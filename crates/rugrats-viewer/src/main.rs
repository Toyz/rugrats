//! rugrats-viewer: fly around a database's rooms.
//!
//! ```text
//! rugrats-viewer DB [--cue DISC.cue] [--at X,Y,Z] [--look YAW,PITCH] [--shot OUT.png] [--models] [--hier] [--anim H,A] [--cine F]
//! ```
//!
//! DB is 0-31. Keys: W A S D move, R F up and down, arrows turn, Shift
//! faster, P logs where the camera is; on a pad the left stick moves, the
//! right stick turns, R1 is faster, R2 and L2 up and down. `--shot` renders
//! one frame offscreen and writes it as PNG. `--models` lays every model of
//! the database out in rows from the origin; `--hier` every hierarchy at
//! rest; `--anim H,A` hierarchy H at each key frame of animation A.

#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::path::PathBuf;

use rrt::glam::{self, Mat4, Vec3};
use rrt::gpu::{Gpu, Target};
use rrt::prelude::*;
use rrt::winit::event::{ElementState, WindowEvent};
use rrt::winit::keyboard::{KeyCode, PhysicalKey};
use rugrats::disc::GameDisc;
use rugrats_render::{Renderer, Vram, Vtx};

/// The database's files: models, rooms, its texture image; and the
/// characters' texture image every database shares.
const MOD: usize = 242;
const LEVEL: usize = 434;
const A_RAW: usize = 499;
const CHARACTR_RAW: usize = 498;
const HIE: usize = 370;
const ANM: usize = 274;
const FRM: usize = 306;
const ROT: usize = 338;

struct Camera {
    pos: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    /// The world is y down, as the PlayStation's.
    fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.cos() * self.pitch.cos(), -self.pitch.sin(), self.yaw.sin() * self.pitch.cos())
    }

    fn matrix(&self, aspect: f32) -> Mat4 {
        let view = glam::camera::rh::view::look_to_mat4(self.pos, self.forward(), Vec3::NEG_Y);
        rrt::gpu::projection(60f32.to_radians(), aspect, 16.0, 200_000.0) * view
    }
}

struct Scene {
    vram: Vram,
    triangles: Vec<Vtx>,
    start: Vec3,
    /// A cinematic frame's camera, when one is shown: its view-projection.
    cine_camera: Option<Mat4>,
}

impl Scene {
    fn load(
        disc: &GameDisc,
        db: usize,
        models: bool,
        hier: bool,
        anim: Option<(usize, usize)>,
        cine: Option<usize>,
    ) -> Result<Scene, String> {
        let level = rugrats_data::level::parse(&disc.file(LEVEL + db)?).map_err(|e| e.to_string())?;
        let raw = disc.file(A_RAW + db)?;
        let mut vram = Vram::new();
        let words: Vec<u16> = raw.chunks_exact(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
        // load_rooms (0x8002a208) uploads it to (0, 256), 576 x 256.
        vram.load(0, 256, 576, 256, &words);
        // main loads it at boot (0x800186d8) to (576, 256), 384 x 256.
        let chars = disc.file(CHARACTR_RAW)?;
        let words: Vec<u16> = chars.chunks_exact(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
        vram.load(576, 256, 384, 256, &words);
        // RUGRATS_VRAM=PATH: VRAM's lower half from a dump (the oracle's
        // cine_shot CINE_VRAM), to see what the port's uploads miss.
        if let Some(path) = std::env::var_os("RUGRATS_VRAM") {
            let raw = std::fs::read(path).map_err(|e| e.to_string())?;
            let words: Vec<u16> = raw.chunks_exact(2).map(|h| u16::from_le_bytes([h[0], h[1]])).collect();
            vram.load(0, 256, 1024, 256, &words[1024 * 256..]);
        }
        let mut triangles = Vec::new();
        let mut cine_camera = None;
        if let Some(want) = cine {
            // Frame `want` of the opening's stream (cine 0) on its rooms.
            let set = rugrats::cine::Set::load(disc, db)?;
            let stream = rugrats::cine::Stream::read(disc, 0)?;
            let (_, frame) = stream.frames.get(want.saturating_sub(1)).ok_or(format!("no cine frame {want}"))?;
            triangles = set.rooms.clone();
            triangles.extend(set.objects(frame));
            cine_camera = Some(rugrats::cine::camera(frame));
            rrt::tracing::info!("cine frame {}: camera at {:?}, {} poses", frame.number, frame.camera_pos, frame.poses.len());
        } else if let Some((hn, an)) = anim {
            // Hierarchy hn at each key frame of animation an, in a row.
            let all = rugrats_data::model::Models::parse(&disc.file(MOD + db)?).map_err(|e| e.to_string())?;
            let hs = rugrats_data::hierarchy::parse_file(&disc.file(HIE + db)?).map_err(|e| e.to_string())?;
            let h = hs.get(hn).and_then(|h| h.as_ref()).ok_or(format!("no hierarchy {hn}"))?;
            let anims = rugrats_data::anim::Anims::parse(&disc.file(ANM + db)?, &disc.file(FRM + db)?, &disc.file(ROT + db)?)
                .map_err(|e| e.to_string())?;
            let frames = anims.frames_of(an);
            rrt::tracing::info!("hierarchy {hn}, animation {an}: {} frames", frames.len());
            for (k, f) in frames.iter().enumerate() {
                let mut local = vec![None; h.parts.len()];
                for key in anims.keys_of(f) {
                    if let rugrats_data::anim::Key::Rotation { channel, q } = *key {
                        let part = h.header.get(0x14 + channel as usize).copied().unwrap_or(0xff);
                        if let Some(slot) = local.get_mut(part as usize) {
                            *slot = Some(rugrats_render::models::quat_matrix(q));
                        }
                    }
                }
                let at = Vec3::new(k as f32 * 900.0, 0.0, 0.0);
                rugrats_render::models::posed_triangles(h, &all, &local, glam::Mat3::IDENTITY, at, &mut triangles);
            }
        } else if hier {
            // Every hierarchy at rest, in a row from the origin.
            let all = rugrats_data::model::Models::parse(&disc.file(MOD + db)?).map_err(|e| e.to_string())?;
            let hs = rugrats_data::hierarchy::parse_file(&disc.file(HIE + db)?).map_err(|e| e.to_string())?;
            for (k, h) in hs.iter().enumerate().filter_map(|(k, h)| h.as_ref().map(|h| (k, h))) {
                let base = Vec3::new((k % 12) as f32 * 1200.0, 0.0, (k / 12) as f32 * 1500.0);
                for (part, joint) in h.parts.iter().zip(h.rest()) {
                    let Some(m) = all.get(part.model) else { continue };
                    let at = base + Vec3::new(joint[0] as f32, joint[1] as f32, joint[2] as f32);
                    rugrats_render::models::model_triangles(m, glam::Mat3::IDENTITY, at, &mut triangles);
                }
            }
        } else if models {
            // Every model and nothing else, in rows of 16, 600 apart, from
            // the origin.
            let all = rugrats_data::model::Models::parse(&disc.file(MOD + db)?).map_err(|e| e.to_string())?;
            for (k, (_, m)) in all.iter().enumerate() {
                let at = Vec3::new((k % 16) as f32 * 600.0, 0.0, (k / 16) as f32 * 600.0);
                rugrats_render::models::model_triangles(m, glam::Mat3::IDENTITY, at, &mut triangles);
            }
        } else {
            triangles = rugrats_render::rooms::level_triangles(&level);
        }
        // Start in the first room, low, at one end.
        let start = level.rooms.first().map_or(Vec3::ZERO, |r| {
            let (lo, hi) = r.bounds();
            Vec3::new(lo[0] as f32 + 512.0, hi[1] as f32 - 1024.0, (lo[2] + hi[2]) as f32 / 2.0)
        });
        rrt::tracing::info!("database {db}: {} rooms, {} triangles", level.rooms.len(), triangles.len() / 3);
        Ok(Scene { vram, triangles, start, cine_camera })
    }

    fn renderer(&self, device: &rrt::wgpu::Device, queue: &rrt::wgpu::Queue, format: rrt::wgpu::TextureFormat) -> Renderer {
        let mut r = Renderer::new(device, queue, format, &self.vram, &self.triangles);
        // The cinematic's backdrop: play_cine (0x8007475c) sets it.
        r.clear = if self.cine_camera.is_some() { [217, 175, 141] } else { [64, 64, 80] };
        r
    }

    fn shot(&self, width: u32, height: u32, mvp: Mat4) -> Result<Vec<u8>, String> {
        let gpu = Gpu::headless().map_err(|e| e.to_string())?;
        let target = Target::new(&gpu.device, width, height);
        let mut renderer = self.renderer(&gpu.device, &gpu.queue, Target::FORMAT);
        let cmd = renderer.draw(&gpu.device, &gpu.queue, &target.view, (width, height), mvp);
        gpu.queue.submit([cmd]);
        Ok(target.read_back(&gpu).rgba)
    }
}

fn parse_list(s: &str) -> Option<Vec<f32>> {
    s.split(',').map(|v| v.trim().parse().ok()).collect()
}

struct Viewer {
    scene: Scene,
    renderer: Option<Renderer>,
    camera: Camera,
    keys: HashSet<KeyCode>,
}

impl Game for Viewer {
    fn init(&mut self, ctx: &mut Init<'_>) {
        self.renderer = Some(self.scene.renderer(&ctx.gpu.device, &ctx.gpu.queue, ctx.plain_format));
    }

    fn event(&mut self, event: &WindowEvent) -> bool {
        if let WindowEvent::KeyboardInput { event, .. } = event
            && let PhysicalKey::Code(k) = event.physical_key
        {
            if event.state == ElementState::Pressed {
                if k == KeyCode::KeyP {
                    let c = &self.camera;
                    rrt::tracing::info!("--at {:.0},{:.0},{:.0} --look {:.3},{:.3}", c.pos.x, c.pos.y, c.pos.z, c.yaw, c.pitch);
                }
                self.keys.insert(k);
            } else {
                self.keys.remove(&k);
            }
        }
        false
    }

    fn tick(&mut self, t: &mut Tick) {
        let dt = t.dt.as_secs_f32();
        let pad = t.pad;
        let held = |k: KeyCode| self.keys.contains(&k);
        let axis = |b: u8| {
            let v = (b as f32 - 128.0) / 127.5;
            if v.abs() < 0.15 { 0.0 } else { v }
        };
        let fast = held(KeyCode::ShiftLeft) || pad.held(Buttons::R1);
        let speed = if fast { 8000.0 } else { 2000.0 } * dt;
        let mut fwd = -axis(pad.left.y);
        let mut side = axis(pad.left.x);
        let mut up = 0.0;
        for (k, f, s, u) in [
            (KeyCode::KeyW, 1.0, 0.0, 0.0),
            (KeyCode::KeyS, -1.0, 0.0, 0.0),
            (KeyCode::KeyD, 0.0, 1.0, 0.0),
            (KeyCode::KeyA, 0.0, -1.0, 0.0),
            (KeyCode::KeyR, 0.0, 0.0, 1.0),
            (KeyCode::KeyF, 0.0, 0.0, -1.0),
        ] {
            if held(k) {
                fwd += f;
                side += s;
                up += u;
            }
        }
        if pad.held(Buttons::R2) {
            up += 1.0;
        }
        if pad.held(Buttons::L2) {
            up -= 1.0;
        }
        let mut turn = axis(pad.right.x) * 2.0;
        let mut tilt = -axis(pad.right.y) * 1.5;
        for (k, tu, ti) in [
            (KeyCode::ArrowLeft, -1.5, 0.0),
            (KeyCode::ArrowRight, 1.5, 0.0),
            (KeyCode::ArrowUp, 0.0, 1.0),
            (KeyCode::ArrowDown, 0.0, -1.0),
        ] {
            if held(k) {
                turn += tu;
                tilt += ti;
            }
        }
        let c = &mut self.camera;
        c.yaw += turn * dt;
        c.pitch = (c.pitch + tilt * dt).clamp(-1.5, 1.5);
        let right = Vec3::new(-c.yaw.sin(), 0.0, c.yaw.cos());
        c.pos += (c.forward() * fwd + right * side + Vec3::NEG_Y * up) * speed;
    }

    fn draw(&mut self, d: &mut Draw<'_>) {
        let Some(renderer) = &mut self.renderer else { return };
        let size = (d.width, d.height);
        let mvp = self.camera.matrix(size.0 as f32 / size.1 as f32);
        let cmd = renderer.draw(&d.gpu.device, &d.gpu.queue, d.plain, size, mvp);
        d.submit(cmd);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let (mut db, mut cue, mut at, mut look, mut out) = (22usize, None::<PathBuf>, None, None, None::<PathBuf>);
    let mut models = false;
    let mut hier = false;
    let mut anim = None;
    let mut cine = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--cue" => cue = args.next().map(PathBuf::from),
            "--at" => at = args.next().and_then(|s| parse_list(&s)),
            "--look" => look = args.next().and_then(|s| parse_list(&s)),
            "--shot" => out = args.next().map(PathBuf::from),
            "--models" => models = true,
            "--hier" => hier = true,
            "--cine" => cine = args.next().and_then(|s| s.parse().ok()),
            "--anim" => {
                anim = args.next().and_then(|s| {
                    let (h, a) = s.split_once(',')?;
                    Some((h.parse().ok()?, a.parse().ok()?))
                })
            }
            n if !n.starts_with('-') => db = n.parse().map_err(|_| format!("{n}: not a database number"))?,
            _ => return Err("usage: rugrats-viewer DB [--cue DISC.cue] [--at X,Y,Z] [--look YAW,PITCH] [--shot OUT.png]".into()),
        }
    }
    let cue = match cue {
        Some(c) => c,
        None => rrt::disc::Image::find(std::path::Path::new("work/disc")).map_err(|e| e.to_string())?,
    };
    let scene = Scene::load(&GameDisc::open(&cue)?, db, models, hier, anim, cine)?;
    let mut camera = Camera { pos: scene.start, yaw: 0.0, pitch: 0.0 };
    if let Some(p) = at.filter(|p| p.len() == 3) {
        camera.pos = Vec3::new(p[0], p[1], p[2]);
    }
    if let Some(l) = look.filter(|l| l.len() == 2) {
        camera.yaw = l[0];
        camera.pitch = l[1];
    }
    if let Some(out) = out {
        let (w, h) = (1280, 960);
        let mvp = scene.cine_camera.unwrap_or_else(|| camera.matrix(w as f32 / h as f32));
        let rgba = scene.shot(w, h, mvp)?;
        std::fs::write(&out, rrt::image::png::encode(w, h, &rgba)).map_err(|e| e.to_string())?;
        rrt::tracing::info!("-> {}", out.display());
        return Ok(());
    }
    let config = Config::default().title("rugrats-viewer").size((1280, 960)).hz(60.0);
    rrt::app::run(config, Viewer { scene, renderer: None, camera, keys: Default::default() }).map_err(|e| e.to_string())
}

fn main() {
    rrt::app::init_logging(&Config::default().log);
    if let Err(e) = run() {
        rrt::tracing::error!("{e}");
        std::process::exit(1);
    }
}
