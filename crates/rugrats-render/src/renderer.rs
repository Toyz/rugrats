//! Draws triangles with the PlayStation's texturing: the texture page and
//! CLUT of each polygon select texels and colours out of a copy of VRAM, the
//! colour 0x0000 is transparent, and the texel is modulated by the vertex
//! colour with 128 as 1.0. A semi-transparent polygon (bit 31 of its mode)
//! blends its texels that have the top bit set by its page's mode (0 half
//! and half, 1 added, 2 subtracted, 3 a quarter added) and draws the rest
//! as any other. An untextured polygon (bit 30) is its colour as it is, and
//! blends whole when semi-transparent.

use rrt::gpu::{DepthBuffer, GrowBuffer};
use rrt::kit::{Pack, Staging};
use rrt::wgpu::util::DeviceExt;
use rrt::{glam, wgpu};

use crate::mesh::{VRAM_H, VRAM_W, Vram, Vtx};

const SHADER: &str = r#"
struct Camera { mvp: mat4x4f }
@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var vram: texture_2d<u32>;

struct In {
    @location(0) pos: vec3f,
    @location(1) colour: u32,
    @location(2) uv: u32,
    @location(3) mode: u32,
    @location(4) window: u32,
}

struct Out {
    @builtin(position) pos: vec4f,
    @location(0) colour: vec3f,
    @location(1) uv: vec2f,
    @location(2) @interpolate(flat) mode: u32,
    @location(3) @interpolate(flat) window: u32,
}

@vertex
fn vs(v: In) -> Out {
    var o: Out;
    o.pos = camera.mvp * vec4f(v.pos, 1.0);
    o.colour = vec3f(f32(v.colour & 255u), f32((v.colour >> 8u) & 255u), f32((v.colour >> 16u) & 255u));
    o.uv = vec2f(f32(v.uv & 0xffffu), f32(v.uv >> 16u));
    o.mode = v.mode;
    o.window = v.window;
    return o;
}

fn word(x: u32, y: u32) -> u32 {
    return textureLoad(vram, vec2u(x & 1023u, y & 511u), 0).r;
}

fn texel(i: Out) -> u32 {
    let tpage = (i.mode >> 16u) & 0x1fffu;
    let clut = i.mode & 0xffffu;
    let bx = (tpage & 15u) * 64u;
    let by = ((tpage >> 4u) & 1u) * 256u;
    let depth = (tpage >> 7u) & 3u;
    let cx = (clut & 63u) * 16u;
    let cy = clut >> 6u;
    var u = u32(clamp(i.uv.x, 0.0, 255.0));
    var v = u32(clamp(i.uv.y, 0.0, 255.0));
    if ((i.mode >> 29u) & 1u) == 1u {
        let mu = i.window & 255u;
        let mv = (i.window >> 8u) & 255u;
        u = (u & ~mu) | ((i.window >> 16u) & mu);
        v = (v & ~mv) | ((i.window >> 24u) & mv);
    } else {
        u = clamp(u, i.window & 255u, (i.window >> 16u) & 255u);
        v = clamp(v, (i.window >> 8u) & 255u, i.window >> 24u);
    }
    var c: u32;
    if depth == 0u {
        let idx = (word(bx + u / 4u, by + v) >> ((u % 4u) * 4u)) & 15u;
        c = word(cx + idx, cy);
    } else if depth == 1u {
        let idx = (word(bx + u / 2u, by + v) >> ((u % 2u) * 8u)) & 255u;
        c = word(cx + idx, cy);
    } else {
        c = word(bx + u, by + v);
    }
    return c;
}

fn shade(i: Out, c: u32) -> vec4f {
    let t = vec3f(f32(c & 31u), f32((c >> 5u) & 31u), f32((c >> 10u) & 31u)) * 8.0;
    let rgb = clamp(t * i.colour / 128.0, vec3f(0.0), vec3f(255.0)) / 255.0;
    return vec4f(rgb, 1.0);
}

fn flat(i: Out) -> bool {
    return ((i.mode >> 30u) & 1u) == 1u;
}

@fragment
fn fs(i: Out) -> @location(0) vec4f {
    if flat(i) {
        if (i.mode >> 31u) == 1u {
            discard;
        }
        return vec4f(i.colour / 255.0, 1.0);
    }
    let c = texel(i);
    if c == 0u || ((i.mode >> 31u) == 1u && (c & 0x8000u) != 0u) {
        discard;
    }
    return shade(i, c);
}

@fragment
fn fs_semi(i: Out) -> @location(0) vec4f {
    if flat(i) {
        return vec4f(i.colour / 255.0, 1.0);
    }
    let c = texel(i);
    if (c & 0x8000u) == 0u {
        discard;
    }
    return shade(i, c);
}
"#;

pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    /// VRAM as the shader reads it.
    vram: wgpu::Texture,
    group: wgpu::BindGroup,
    camera: wgpu::Buffer,
    /// The HUD over the world, in the screen's pixels, replaced each frame
    /// by `set_overlay`.
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_group: wgpu::BindGroup,
    overlay_camera: wgpu::Buffer,
    overlay: GrowBuffer,
    overlay_count: u32,
    overlay_staging: Staging,
    /// The overlay's semi-transparent shapes, by blend mode, drawn over the
    /// world and under the overlay (the wreck's flash).
    overlay_semi_pipelines: [wgpu::RenderPipeline; 4],
    overlay_semi: GrowBuffer,
    overlay_semi_ranges: [(u32, u32); 4],
    overlay_semi_staging: Staging,
    /// Shapes in the PlayStation screen's pixels drawn in their order under
    /// the world (the back of the ordering table: a menu's backdrop), as
    /// runs of one kind: opaque, or semi-transparent by blend mode.
    under: GrowBuffer,
    under_runs: Vec<(u32, u32, Option<usize>)>,
    under_staging: Staging,
    /// What does not move (the track), uploaded once.
    vertices: wgpu::Buffer,
    count: u32,
    /// What moves (the cars), replaced each frame by `set_moving`, and how
    /// many vertices.
    moving: GrowBuffer,
    moving_count: u32,
    /// Where the moving triangles are packed each frame.
    staging: Staging,
    /// The semi-transparent triangles, by blend mode, each frame; one
    /// pipeline a mode.
    semi_pipelines: [wgpu::RenderPipeline; 4],
    semi: GrowBuffer,
    semi_ranges: [(u32, u32); 4],
    semi_staging: Staging,
    depth: DepthBuffer,
    pub clear: [u8; 3],
    /// The PlayStation screen the overlay is in, in pixels (512 by 240
    /// unless set).
    pub screen: (f32, f32),
}

pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// The game's screen, in pixels.
pub const SCREEN: (f32, f32) = (512.0, 240.0);

/// From the PlayStation screen's pixels (`screen` of them) to a 4:3 box in
/// the middle of a target of `size`, as a television shows it.
pub fn overlay_matrix(size: (u32, u32), screen: (f32, f32)) -> glam::Mat4 {
    let (w, h) = (size.0.max(1) as f32, size.1.max(1) as f32);
    let (bw, bh) = if w * 3.0 > h * 4.0 { (h * 4.0 / 3.0, h) } else { (w, w * 3.0 / 4.0) };
    let (sx, sy) = (bw / w, bh / h);
    glam::Mat4::from_cols_array(&[
        2.0 * sx / screen.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -2.0 * sy / screen.1,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        -sx,
        sy,
        0.0,
        1.0,
    ])
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        vram: &Vram,
        tris: &[Vtx],
    ) -> Renderer {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("world"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let vram_tex = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("vram"),
                size: wgpu::Extent3d { width: VRAM_W as u32, height: VRAM_H as u32, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R16Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &vram.words.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>(),
        );
        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("world"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let view = vram_tex.create_view(&Default::default());
        let overlay_camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let overlay_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("overlay"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: overlay_camera.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            ],
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("world"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&view) },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("world"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = make_pipeline(device, &pipeline_layout, &shader, format, true, None);
        // The HUD: drawn over everything, both faces.
        let overlay_pipeline = make_pipeline(device, &pipeline_layout, &shader, format, false, None);
        let overlay_semi_pipelines =
            std::array::from_fn(|abr| make_pipeline(device, &pipeline_layout, &shader, format, false, Some(abr as u8)));
        let semi_pipelines =
            std::array::from_fn(|abr| make_pipeline(device, &pipeline_layout, &shader, format, true, Some(abr as u8)));
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("triangles"),
            contents: Staging::new().pack(tris),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Renderer {
            pipeline,
            vram: vram_tex,
            group,
            camera,
            overlay_pipeline,
            overlay_group,
            overlay_camera,
            overlay: GrowBuffer::new("overlay", wgpu::BufferUsages::VERTEX),
            overlay_count: 0,
            overlay_staging: Staging::new(),
            overlay_semi_pipelines,
            overlay_semi: GrowBuffer::new("overlay semi", wgpu::BufferUsages::VERTEX),
            overlay_semi_ranges: [(0, 0); 4],
            overlay_semi_staging: Staging::new(),
            under: GrowBuffer::new("under", wgpu::BufferUsages::VERTEX),
            under_runs: Vec::new(),
            under_staging: Staging::new(),
            vertices,
            count: tris.len() as u32,
            moving: GrowBuffer::new("moving", wgpu::BufferUsages::VERTEX),
            moving_count: 0,
            staging: Staging::new(),
            semi_pipelines,
            semi: GrowBuffer::new("semi", wgpu::BufferUsages::VERTEX),
            semi_ranges: [(0, 0); 4],
            semi_staging: Staging::new(),
            depth: DepthBuffer::new(DEPTH),
            clear: [0, 0, 0],
            screen: SCREEN,
        }
    }

    /// Replaces VRAM along one row from (`x`, `y`) with `words` (a
    /// LoadImage of one line, as for a palette).
    pub fn load_vram(&self, queue: &wgpu::Queue, x: u16, y: u16, words: &[u16]) {
        self.load_vram_rect(queue, x, y, words.len() as u16, 1, words);
    }

    /// Replaces the `w` x `h` words of VRAM at (`x`, `y`) with `words`, row
    /// by row (a LoadImage).
    pub fn load_vram_rect(&self, queue: &wgpu::Queue, x: u16, y: u16, w: u16, h: u16, words: &[u16]) {
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.vram,
                mip_level: 0,
                origin: wgpu::Origin3d { x: x as u32, y: y as u32, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(2 * w as u32), rows_per_image: None },
            wgpu::Extent3d { width: w as u32, height: h as u32, depth_or_array_layers: 1 },
        );
    }

    /// The triangles that move, drawn after the fixed ones until replaced.
    pub fn set_moving(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tris: &[Vtx]) {
        self.moving.write(device, queue, self.staging.pack(tris));
        self.moving_count = tris.len() as u32;
    }

    /// The semi-transparent triangles' blended texels, drawn after the
    /// opaque ones (give them to `set_moving` too, for their other texels).
    pub fn set_semi(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tris: &[Vtx]) {
        let abr = |v: &Vtx| ((v.mode >> 21) & 3) as usize;
        let mut sorted = Vec::with_capacity(tris.len());
        for k in 0..4 {
            let from = sorted.len() as u32;
            for t in tris.as_chunks::<3>().0.iter().filter(|t| abr(&t[0]) == k) {
                sorted.extend_from_slice(t);
            }
            self.semi_ranges[k] = (from, sorted.len() as u32);
        }
        self.semi.write(device, queue, self.semi_staging.pack(&sorted));
    }

    /// The overlay's semi-transparent triangles, in the PlayStation screen's
    /// pixels, each by its page's blend mode.
    pub fn set_overlay_semi(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tris: &[Vtx]) {
        let abr = |v: &Vtx| ((v.mode >> 21) & 3) as usize;
        let mut sorted = Vec::with_capacity(tris.len());
        for k in 0..4 {
            let from = sorted.len() as u32;
            for t in tris.as_chunks::<3>().0.iter().filter(|t| abr(&t[0]) == k) {
                sorted.extend_from_slice(t);
            }
            self.overlay_semi_ranges[k] = (from, sorted.len() as u32);
        }
        self.overlay_semi.write(device, queue, self.overlay_semi_staging.pack(&sorted));
    }

    /// Shapes drawn under the world in this order, in the PlayStation
    /// screen's pixels. A semi-transparent triangle (mode bit 31) blends
    /// its marked texels by its page's mode, as on the PlayStation.
    pub fn set_under(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tris: &[Vtx]) {
        let kind = |v: &Vtx| (v.mode >> 31 == 1).then_some(((v.mode >> 21) & 3) as usize);
        self.under_runs.clear();
        for (k, t) in tris.as_chunks::<3>().0.iter().enumerate() {
            let at = 3 * k as u32;
            match self.under_runs.last_mut() {
                Some(run) if run.2 == kind(&t[0]) && run.1 == at => run.1 = at + 3,
                _ => self.under_runs.push((at, at + 3, kind(&t[0]))),
            }
        }
        self.under.write(device, queue, self.under_staging.pack(tris));
    }

    /// The overlay's triangles, in the PlayStation screen's pixels.
    pub fn set_overlay(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, tris: &[Vtx]) {
        self.overlay.write(device, queue, self.overlay_staging.pack(tris));
        self.overlay_count = tris.len() as u32;
    }

    /// Draws into `target` with the camera's view-projection matrix.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: &wgpu::TextureView,
        size: (u32, u32),
        mvp: glam::Mat4,
    ) -> wgpu::CommandBuffer {
        let depth_view = self.depth.view(device, size.0, size.1).clone();
        let bytes: Vec<u8> = mvp.to_cols_array().iter().flat_map(|f| f.to_le_bytes()).collect();
        queue.write_buffer(&self.camera, 0, &bytes);
        let overlay = overlay_matrix(size, self.screen);
        let bytes: Vec<u8> = overlay.to_cols_array().iter().flat_map(|f| f.to_le_bytes()).collect();
        queue.write_buffer(&self.overlay_camera, 0, &bytes);
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let c = self.clear.map(|v| v as f64 / 255.0);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("world"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: c[0], g: c[1], b: c[2], a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some(slice) = self.under.slice() {
                pass.set_vertex_buffer(0, slice);
                for &(from, to, kind) in &self.under_runs {
                    pass.set_pipeline(&self.overlay_pipeline);
                    pass.set_bind_group(0, &self.overlay_group, &[]);
                    pass.draw(from..to, 0..1);
                    if let Some(k) = kind {
                        pass.set_pipeline(&self.overlay_semi_pipelines[k]);
                        pass.set_bind_group(0, &self.overlay_group, &[]);
                        pass.set_blend_constant(wgpu::Color { r: BLEND[k], g: BLEND[k], b: BLEND[k], a: BLEND[k] });
                        pass.draw(from..to, 0..1);
                    }
                }
            }
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.group, &[]);
            if self.count > 0 {
                pass.set_vertex_buffer(0, self.vertices.slice(..));
                pass.draw(0..self.count, 0..1);
            }
            if let Some(slice) = self.moving.slice()
                && self.moving_count > 0
            {
                pass.set_vertex_buffer(0, slice);
                pass.draw(0..self.moving_count, 0..1);
            }
            if let Some(slice) = self.semi.slice() {
                pass.set_vertex_buffer(0, slice);
                for (k, &(from, to)) in self.semi_ranges.iter().enumerate() {
                    if from == to {
                        continue;
                    }
                    pass.set_pipeline(&self.semi_pipelines[k]);
                    pass.set_bind_group(0, &self.group, &[]);
                    pass.set_blend_constant(wgpu::Color { r: BLEND[k], g: BLEND[k], b: BLEND[k], a: BLEND[k] });
                    pass.draw(from..to, 0..1);
                }
            }
            if let Some(slice) = self.overlay_semi.slice() {
                pass.set_vertex_buffer(0, slice);
                for (k, &(from, to)) in self.overlay_semi_ranges.iter().enumerate() {
                    if from == to {
                        continue;
                    }
                    pass.set_pipeline(&self.overlay_semi_pipelines[k]);
                    pass.set_bind_group(0, &self.overlay_group, &[]);
                    pass.set_blend_constant(wgpu::Color { r: BLEND[k], g: BLEND[k], b: BLEND[k], a: BLEND[k] });
                    pass.draw(from..to, 0..1);
                }
            }
            if let Some(slice) = self.overlay.slice()
                && self.overlay_count > 0
            {
                pass.set_pipeline(&self.overlay_pipeline);
                pass.set_bind_group(0, &self.overlay_group, &[]);
                pass.set_vertex_buffer(0, slice);
                pass.draw(0..self.overlay_count, 0..1);
            }
        }
        encoder.finish()
    }
}

/// The blend constant each semi-transparent mode uses (its share of the new
/// colour: half, or a quarter; the others ignore it).
const BLEND: [f64; 4] = [0.5, 1.0, 1.0, 0.25];

/// A semi-transparent mode as a blend: half old and half new, old plus new,
/// old less new, old plus a quarter of new.
fn blend_of(abr: u8) -> wgpu::BlendState {
    use wgpu::{BlendComponent as C, BlendFactor as F, BlendOperation as O};
    let colour = match abr {
        0 => C { src_factor: F::Constant, dst_factor: F::OneMinusConstant, operation: O::Add },
        1 => C { src_factor: F::One, dst_factor: F::One, operation: O::Add },
        2 => C { src_factor: F::One, dst_factor: F::One, operation: O::ReverseSubtract },
        _ => C { src_factor: F::Constant, dst_factor: F::One, operation: O::Add },
    };
    wgpu::BlendState { color: colour, alpha: C::REPLACE }
}

/// The pipeline for the world (depth-tested, back faces culled) or for the
/// overlay (neither); or, with a semi-transparent mode, for the world's
/// blended texels (depth-tested but not written, both faces).
fn make_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    world: bool,
    semi: Option<u8>,
) -> wgpu::RenderPipeline {
    let attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Uint32, 3 => Uint32, 4 => Uint32];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if semi.is_some() {
            "semi"
        } else if world {
            "world"
        } else {
            "overlay"
        }),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: <Vtx as Pack>::SIZE as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attrs,
            })],
        },
        // The game draws a face only when it is front-facing; the
        // triangles are wound counter-clockwise for front.
        primitive: wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: (world && semi.is_none()).then_some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH,
            depth_write_enabled: Some(world && semi.is_none()),
            depth_compare: Some(match (world, semi) {
                // Equal depths go to the later polygon, as the ordering
                // table draws a coplanar decal over its floor.
                (true, None) => wgpu::CompareFunction::LessEqual,
                (true, Some(_)) => wgpu::CompareFunction::LessEqual,
                _ => wgpu::CompareFunction::Always,
            }),
            stencil: Default::default(),
            // The game sorts by depth rather than testing it, so marks on
            // the ground show; here they are pulled a little forward.
            bias: if semi.is_some() {
                wgpu::DepthBiasState { constant: -4, slope_scale: -2.0, clamp: 0.0 }
            } else {
                Default::default()
            },
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(if semi.is_some() { "fs_semi" } else { "fs" }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: semi.map(blend_of),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
