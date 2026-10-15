//! The GPU as the game drives it: GP0 commands and GP1 control, VRAM, and a
//! software rasterizer that follows the PlayStation's rules closely enough to
//! show what the original draws: affine texturing through 4-, 8- and 15-bit
//! texture pages, texel times colour over 128, the four semi-transparency
//! modes, the drawing area and offset. Dithering and mask bits are not
//! modelled.

pub const VRAM_W: usize = 1024;
pub const VRAM_H: usize = 512;

#[derive(Clone, Copy, Debug, Default)]
pub struct Display {
    /// Top left of the displayed area in VRAM.
    pub x: u16,
    pub y: u16,
    /// Horizontal resolution: 256, 320, 368, 512 or 640.
    pub width: u16,
    /// 240 or 480.
    pub height: u16,
    pub enabled: bool,
    /// 24-bit colour (GP1 0x08 bit 4): three bytes a pixel, as the movies
    /// are shown.
    pub rgb24: bool,
}

pub struct Gpu {
    pub vram: Vec<u16>,
    /// A rectangle of the drawing area (x0, y0, x1, y1) whose polygons
    /// are printed as they are drawn.
    pub trace: Option<[i32; 4]>,
    /// GP0 words of the command being assembled.
    cmd: Vec<u32>,
    /// A CPU-to-VRAM transfer in progress: rectangle and halfwords done.
    upload: Option<(u16, u16, u16, u16, usize)>,
    /// A VRAM-to-CPU transfer in progress.
    download: Option<(u16, u16, u16, u16, usize)>,
    // Drawing environment.
    texpage: u16,
    area: (i32, i32, i32, i32),
    offset: (i32, i32),
    pub display: Display,
    /// Primitives drawn since the counter was last read.
    pub drawn: u64,
    field: bool,
    /// Status reads left that report the GPU busy after drawing. Drawing is
    /// instant here, but the game waits for the GPU to have *started*
    /// (`while (DrawSync(1) == 0)` at 0x80013a64), which on hardware is
    /// immediately true.
    busy: u32,
}

impl Default for Gpu {
    fn default() -> Self {
        Gpu {
            vram: vec![0; VRAM_W * VRAM_H],
            trace: None,
            cmd: Vec::new(),
            upload: None,
            download: None,
            texpage: 0,
            area: (0, 0, 1023, 511),
            offset: (0, 0),
            display: Display { x: 0, y: 0, width: 320, height: 240, enabled: false, rgb24: false },
            drawn: 0,
            field: false,
            busy: 0,
        }
    }
}

fn sext(v: u32, bits: u32) -> i32 {
    ((v << (32 - bits)) as i32) >> (32 - bits)
}

#[derive(Clone, Copy, Debug, Default)]
struct Vert {
    x: i32,
    y: i32,
    rgb: [i32; 3],
    u: i32,
    v: i32,
}

/// How a primitive samples and blends.
#[derive(Clone, Copy, Debug)]
struct Mode {
    textured: bool,
    raw: bool,
    semi: bool,
    texpage: u16,
    clut: u16,
}

fn put_xfer(w: &mut crate::state::Writer, t: &Option<(u16, u16, u16, u16, usize)>) {
    match t {
        Some((x, y, wd, h, n)) => {
            w.u8(1);
            for v in [*x, *y, *wd, *h] {
                w.u16(v);
            }
            w.u32(*n as u32);
        }
        None => w.u8(0),
    }
}

/// A VRAM transfer in progress: x, y, width, height and halfwords done.
type Xfer = (u16, u16, u16, u16, usize);

fn get_xfer(r: &mut crate::state::Reader) -> crate::state::Result<Option<Xfer>> {
    Ok(if r.u8()? != 0 { Some((r.u16()?, r.u16()?, r.u16()?, r.u16()?, r.u32()? as usize)) } else { None })
}

impl Gpu {
    pub fn save(&self, w: &mut crate::state::Writer) {
        w.u16s(&self.vram);
        w.u32s(&self.cmd);
        put_xfer(w, &self.upload);
        put_xfer(w, &self.download);
        w.u16(self.texpage);
        for v in [self.area.0, self.area.1, self.area.2, self.area.3, self.offset.0, self.offset.1] {
            w.u32(v as u32);
        }
        let d = self.display;
        for v in [d.x, d.y, d.width, d.height] {
            w.u16(v);
        }
        w.bool(d.enabled);
        w.bool(d.rgb24);
        w.u64(self.drawn);
        w.bool(self.field);
        w.u32(self.busy);
    }

    pub fn load(&mut self, r: &mut crate::state::Reader) -> crate::state::Result<()> {
        self.vram = r.u16s()?;
        self.cmd = r.u32s()?;
        self.upload = get_xfer(r)?;
        self.download = get_xfer(r)?;
        self.texpage = r.u16()?;
        let mut a = [0i32; 6];
        for v in &mut a {
            *v = r.u32()? as i32;
        }
        self.area = (a[0], a[1], a[2], a[3]);
        self.offset = (a[4], a[5]);
        self.display =
            Display { x: r.u16()?, y: r.u16()?, width: r.u16()?, height: r.u16()?, enabled: r.bool()?, rgb24: r.bool()? };
        self.drawn = r.u64()?;
        self.field = r.bool()?;
        self.busy = r.u32()?;
        Ok(())
    }

    pub fn status(&mut self) -> u32 {
        self.field = !self.field;
        let mut s = 0x1c00_0000 | (self.texpage as u32 & 0x7ff);
        if self.busy > 0 {
            self.busy -= 1;
            s &= !(1 << 26);
        }
        if self.display.height == 480 {
            s |= 1 << 19 | 1 << 22;
        }
        if self.field {
            s |= 1 << 31;
        }
        s
    }

    pub fn read(&mut self) -> u32 {
        let Some((x, y, w, h, done)) = self.download else { return 0 };
        let mut out = 0u32;
        let mut n = done;
        for k in 0..2 {
            let (px, py) = (x as usize + n % w as usize, y as usize + n / w as usize);
            out |= (self.vram[(py % VRAM_H) * VRAM_W + px % VRAM_W] as u32) << (16 * k);
            n += 1;
        }
        self.download = if n >= w as usize * h as usize { None } else { Some((x, y, w, h, n)) };
        out
    }

    pub fn gp1(&mut self, v: u32) {
        match v >> 24 {
            0x00 => {
                self.cmd.clear();
                self.upload = None;
                self.display.enabled = false;
            }
            0x01 => {
                self.cmd.clear();
                self.upload = None;
            }
            0x03 => self.display.enabled = v & 1 == 0,
            0x05 => {
                self.display.x = (v & 0x3ff) as u16;
                self.display.y = ((v >> 10) & 0x1ff) as u16;
            }
            0x08 => {
                self.display.width = match (v & 3, v & 0x40 != 0) {
                    (_, true) => 368,
                    (0, _) => 256,
                    (1, _) => 320,
                    (2, _) => 512,
                    _ => 640,
                };
                self.display.height = if v & 4 != 0 { 480 } else { 240 };
                self.display.rgb24 = v & 0x10 != 0;
            }
            _ => {}
        }
    }

    pub fn gp0(&mut self, w: u32) {
        if let Some((x, y, wd, h, done)) = self.upload {
            let mut n = done;
            for half in [w as u16, (w >> 16) as u16] {
                if n >= wd as usize * h as usize {
                    break;
                }
                let (px, py) = (x as usize + n % wd as usize, y as usize + n / wd as usize);
                self.vram[(py % VRAM_H) * VRAM_W + px % VRAM_W] = half;
                n += 1;
            }
            self.upload = if n >= wd as usize * h as usize { None } else { Some((x, y, wd, h, n)) };
            return;
        }
        self.cmd.push(w);
        let op = self.cmd[0] >> 24;
        // Poly-lines end with a terminator word.
        if (0x48..0x60).contains(&op) && op & 0x08 != 0 {
            if self.cmd.len() > 3 && (w & 0xf000_f000) == 0x5000_5000 {
                let c = std::mem::take(&mut self.cmd);
                self.polyline(&c);
            }
            return;
        }
        if self.cmd.len() < command_len(op) {
            return;
        }
        let c = std::mem::take(&mut self.cmd);
        if (0x20..0x80).contains(&op) || op == 0x02 {
            self.busy = 2;
        }
        self.execute(&c);
    }

    fn execute(&mut self, c: &[u32]) {
        let op = c[0] >> 24;
        // RUGRATS_GPU_FILLS: print fills and rectangles, to find backdrops.
        if (op == 0x02 || (0x60..0x80).contains(&op)) && std::env::var_os("RUGRATS_GPU_FILLS").is_some() {
            eprintln!("gpu {op:02x} {:08x?}", c);
        }
        match op {
            0x02 => {
                let rgb = c[0];
                let (x, y) = ((c[1] & 0x3f0) as usize, ((c[1] >> 16) & 0x1ff) as usize);
                let (w, h) = ((((c[2] & 0x3ff) + 15) & !15) as usize, ((c[2] >> 16) & 0x1ff) as usize);
                let px = to15([(rgb & 0xff) as i32, ((rgb >> 8) & 0xff) as i32, ((rgb >> 16) & 0xff) as i32]);
                for yy in y..y + h {
                    for xx in x..x + w {
                        self.vram[(yy % VRAM_H) * VRAM_W + xx % VRAM_W] = px;
                    }
                }
            }
            0x20..=0x3f => {
                self.polygon(c);
                self.drawn += 1;
            }
            0x40..=0x5f => self.polyline(c),
            0x60..=0x7f => {
                self.rect(c);
                self.drawn += 1;
            }
            0x80..=0x9f => {
                let (sx, sy) = ((c[1] & 0x3ff) as usize, ((c[1] >> 16) & 0x1ff) as usize);
                let (dx, dy) = ((c[2] & 0x3ff) as usize, ((c[2] >> 16) & 0x1ff) as usize);
                let (w, h) = (((c[3] & 0xffff).max(1)) as usize, ((c[3] >> 16).max(1)) as usize);
                for yy in 0..h {
                    for xx in 0..w {
                        let p = self.vram[((sy + yy) % VRAM_H) * VRAM_W + (sx + xx) % VRAM_W];
                        self.vram[((dy + yy) % VRAM_H) * VRAM_W + (dx + xx) % VRAM_W] = p;
                    }
                }
            }
            0xa0..=0xbf => {
                let (x, y) = ((c[1] & 0x3ff) as u16, ((c[1] >> 16) & 0x1ff) as u16);
                let (w, h) = (((c[2] & 0xffff) as u16).max(1), ((c[2] >> 16) as u16).max(1));
                self.upload = Some((x, y, w, h, 0));
            }
            0xc0..=0xdf => {
                let (x, y) = ((c[1] & 0x3ff) as u16, ((c[1] >> 16) & 0x1ff) as u16);
                let (w, h) = (((c[2] & 0xffff) as u16).max(1), ((c[2] >> 16) as u16).max(1));
                self.download = Some((x, y, w, h, 0));
            }
            0xe1 => self.texpage = (c[0] & 0x7ff) as u16,
            0xe3 => self.area = (sext(c[0] & 0x3ff, 32), sext((c[0] >> 10) & 0x1ff, 32), self.area.2, self.area.3),
            0xe4 => self.area = (self.area.0, self.area.1, (c[0] & 0x3ff) as i32, ((c[0] >> 10) & 0x1ff) as i32),
            0xe5 => self.offset = (sext(c[0] & 0x7ff, 11), sext((c[0] >> 11) & 0x7ff, 11)),
            _ => {}
        }
    }

    fn polygon(&mut self, c: &[u32]) {
        let op = c[0] >> 24;
        let gouraud = op & 0x10 != 0;
        let quad = op & 0x08 != 0;
        let textured = op & 0x04 != 0;
        let n = if quad { 4 } else { 3 };
        let mut verts = [Vert::default(); 4];
        let mut at = 1;
        let mut clut = 0;
        let mut page = self.texpage;
        let base = rgb_of(c[0]);
        for (i, v) in verts.iter_mut().enumerate().take(n) {
            v.rgb = if gouraud && i > 0 {
                let col = rgb_of(c[at]);
                at += 1;
                col
            } else {
                base
            };
            v.x = sext(c[at] & 0x7ff, 11) + self.offset.0;
            v.y = sext((c[at] >> 16) & 0x7ff, 11) + self.offset.1;
            at += 1;
            if textured {
                let t = c[at];
                at += 1;
                v.u = (t & 0xff) as i32;
                v.v = ((t >> 8) & 0xff) as i32;
                match i {
                    0 => clut = (t >> 16) as u16,
                    1 => {
                        page = (t >> 16) as u16;
                        self.texpage = (self.texpage & !0x1ff) | (page & 0x1ff);
                    }
                    _ => {}
                }
            }
        }
        let mode = Mode { textured, raw: op & 0x01 != 0, semi: op & 0x02 != 0, texpage: page, clut };
        // `trace`: print the polygons with a corner in that part of the
        // drawing area, to find what draws something.
        if let Some(r) = self.trace
            && verts[..n].iter().any(|v| v.x >= r[0] && v.x < r[2] && v.y >= r[1] && v.y < r[3])
        {
            let xy: Vec<_> = verts[..n].iter().map(|v| (v.x, v.y, v.u, v.v)).collect();
            eprintln!("gpu poly {op:02x} page {page:04x} clut {clut:04x} {xy:?}");
        }
        self.triangle(&verts[0], &verts[1], &verts[2], mode, gouraud);
        if quad {
            self.triangle(&verts[1], &verts[2], &verts[3], mode, gouraud);
        }
    }

    fn polyline(&mut self, c: &[u32]) {
        let op = c[0] >> 24;
        let gouraud = op & 0x10 != 0;
        let mut pts = Vec::new();
        let mut at = 1;
        let mut col = rgb_of(c[0]);
        while at < c.len() {
            if (c[at] & 0xf000_f000) == 0x5000_5000 && pts.len() >= 2 {
                break;
            }
            if gouraud && !pts.is_empty() {
                col = rgb_of(c[at]);
                at += 1;
                if at >= c.len() {
                    break;
                }
            }
            let x = sext(c[at] & 0x7ff, 11) + self.offset.0;
            let y = sext((c[at] >> 16) & 0x7ff, 11) + self.offset.1;
            pts.push((x, y, col));
            at += 1;
        }
        for w in pts.windows(2) {
            let ((x0, y0, c0), (x1, y1, _)) = (w[0], w[1]);
            let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
            for s in 0..=steps {
                let x = x0 + (x1 - x0) * s / steps;
                let y = y0 + (y1 - y0) * s / steps;
                self.plot(x, y, to15(c0), op & 0x02 != 0);
            }
        }
    }

    fn rect(&mut self, c: &[u32]) {
        let op = c[0] >> 24;
        let textured = op & 0x04 != 0;
        let x = sext(c[1] & 0x7ff, 11) + self.offset.0;
        let y = sext((c[1] >> 16) & 0x7ff, 11) + self.offset.1;
        let mut at = 2;
        let (mut u0, mut v0, mut clut) = (0, 0, 0);
        if textured {
            u0 = (c[at] & 0xff) as i32;
            v0 = ((c[at] >> 8) & 0xff) as i32;
            clut = (c[at] >> 16) as u16;
            at += 1;
        }
        let (w, h) = match (op >> 3) & 3 {
            0 => ((c[at] & 0x3ff) as i32, ((c[at] >> 16) & 0x1ff) as i32),
            1 => (1, 1),
            2 => (8, 8),
            _ => (16, 16),
        };
        let mode = Mode { textured, raw: op & 0x01 != 0, semi: op & 0x02 != 0, texpage: self.texpage, clut };
        let rgb = rgb_of(c[0]);
        for yy in 0..h {
            for xx in 0..w {
                let v = Vert { x: x + xx, y: y + yy, rgb, u: u0 + xx, v: v0 + yy };
                self.shade(v.x, v.y, v.rgb, v.u, v.v, mode);
            }
        }
    }

    fn triangle(&mut self, a: &Vert, b: &Vert, c: &Vert, mode: Mode, gouraud: bool) {
        let area = (b.x - a.x) as i64 * (c.y - a.y) as i64 - (c.x - a.x) as i64 * (b.y - a.y) as i64;
        if area == 0 {
            return;
        }
        // The GPU refuses polygons wider than 1023 or taller than 511.
        let (minx, maxx) = (a.x.min(b.x).min(c.x), a.x.max(b.x).max(c.x));
        let (miny, maxy) = (a.y.min(b.y).min(c.y), a.y.max(b.y).max(c.y));
        if maxx - minx > 1023 || maxy - miny > 511 {
            return;
        }
        let (x0, x1) = (minx.max(self.area.0), maxx.min(self.area.2));
        let (y0, y1) = (miny.max(self.area.1), maxy.min(self.area.3));
        let edge = |p: &Vert, q: &Vert, x: i32, y: i32| {
            (q.x - p.x) as i64 * (y - p.y) as i64 - (q.y - p.y) as i64 * (x - p.x) as i64
        };
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (w0, w1, w2) = (edge(b, c, x, y), edge(c, a, x, y), edge(a, b, x, y));
                let inside = if area > 0 { w0 >= 0 && w1 >= 0 && w2 >= 0 } else { w0 <= 0 && w1 <= 0 && w2 <= 0 };
                if !inside {
                    continue;
                }
                // The right and bottom edges are not drawn.
                if (w0 == 0 && excluded(b, c, area))
                    || (w1 == 0 && excluded(c, a, area))
                    || (w2 == 0 && excluded(a, b, area))
                {
                    continue;
                }
                let lerp =
                    |pa: i32, pb: i32, pc: i32| ((w0 * pa as i64 + w1 * pb as i64 + w2 * pc as i64) / area) as i32;
                let rgb = if gouraud {
                    [
                        lerp(a.rgb[0], b.rgb[0], c.rgb[0]),
                        lerp(a.rgb[1], b.rgb[1], c.rgb[1]),
                        lerp(a.rgb[2], b.rgb[2], c.rgb[2]),
                    ]
                } else {
                    a.rgb
                };
                let (u, v) = if mode.textured { (lerp(a.u, b.u, c.u), lerp(a.v, b.v, c.v)) } else { (0, 0) };
                self.shade(x, y, rgb, u, v, mode);
            }
        }
    }

    fn texel(&self, mode: Mode, u: i32, v: i32) -> u16 {
        let (u, v) = ((u & 0xff) as usize, (v & 0xff) as usize);
        let bx = (mode.texpage as usize & 15) * 64;
        let by = ((mode.texpage as usize >> 4) & 1) * 256;
        let (cx, cy) = ((mode.clut as usize & 63) * 16, (mode.clut as usize >> 6) & 511);
        let at = |x: usize, y: usize| self.vram[(y % VRAM_H) * VRAM_W + x % VRAM_W];
        match (mode.texpage >> 7) & 3 {
            0 => {
                let idx = (at(bx + u / 4, by + v) >> ((u % 4) * 4)) & 15;
                at(cx + idx as usize, cy)
            }
            1 => {
                let idx = (at(bx + u / 2, by + v) >> ((u % 2) * 8)) & 255;
                at(cx + idx as usize, cy)
            }
            _ => at(bx + u, by + v),
        }
    }

    fn shade(&mut self, x: i32, y: i32, rgb: [i32; 3], u: i32, v: i32, mode: Mode) {
        if x < self.area.0 || x > self.area.2 || y < self.area.1 || y > self.area.3 {
            return;
        }
        let (px, semi) = if mode.textured {
            let t = self.texel(mode, u, v);
            if t == 0 {
                return;
            }
            let px = if mode.raw {
                t
            } else {
                let ch = |k: u32| (((t >> (5 * k)) & 31) as i32 * rgb[k as usize] / 128).min(31) as u16;
                ch(0) | ch(1) << 5 | ch(2) << 10 | (t & 0x8000)
            };
            (px, mode.semi && t & 0x8000 != 0)
        } else {
            (to15(rgb), mode.semi)
        };
        self.blend(x, y, px, semi, mode.texpage);
    }

    fn plot(&mut self, x: i32, y: i32, px: u16, semi: bool) {
        if x < self.area.0 || x > self.area.2 || y < self.area.1 || y > self.area.3 {
            return;
        }
        self.blend(x, y, px, semi, self.texpage);
    }

    fn blend(&mut self, x: i32, y: i32, px: u16, semi: bool, texpage: u16) {
        let at = (y as usize % VRAM_H) * VRAM_W + x as usize % VRAM_W;
        let out = if semi {
            let back = self.vram[at];
            let ch = |p: u16, k: u32| ((p >> (5 * k)) & 31) as i32;
            let mix = |k: u32| -> u16 {
                let (b, f) = (ch(back, k), ch(px, k));
                (match (texpage >> 5) & 3 {
                    0 => (b + f) / 2,
                    1 => b + f,
                    2 => b - f,
                    _ => b + f / 4,
                })
                .clamp(0, 31) as u16
            };
            mix(0) | mix(1) << 5 | mix(2) << 10 | (px & 0x8000)
        } else {
            px
        };
        self.vram[at] = out;
    }

    /// The displayed picture as RGBA8.
    pub fn screen(&self) -> (usize, usize, Vec<u8>) {
        let d = self.display;
        let (w, h) = (d.width as usize, d.height as usize);
        let mut out = Vec::with_capacity(w * h * 4);
        if d.rgb24 {
            for y in 0..h {
                let row = (d.y as usize + y) % VRAM_H * VRAM_W;
                let byte = |b: usize| {
                    let half = self.vram[row + (d.x as usize + b / 2) % VRAM_W];
                    (half >> (8 * (b % 2))) as u8
                };
                for x in 0..w {
                    out.extend([byte(3 * x), byte(3 * x + 1), byte(3 * x + 2), 255]);
                }
            }
            return (w, h, out);
        }
        for y in 0..h {
            for x in 0..w {
                let p = self.vram[((d.y as usize + y) % VRAM_H) * VRAM_W + (d.x as usize + x) % VRAM_W];
                let ex = |v: u16| ((v & 31) << 3 | (v & 31) >> 2) as u8;
                out.extend([ex(p), ex(p >> 5), ex(p >> 10), 255]);
            }
        }
        (w, h, out)
    }

    /// All of VRAM as RGBA8, 1024 x 512, for looking at what was uploaded.
    pub fn vram_rgba(&self) -> Vec<u8> {
        let ex = |v: u16| ((v & 31) << 3 | (v & 31) >> 2) as u8;
        self.vram.iter().flat_map(|&p| [ex(p), ex(p >> 5), ex(p >> 10), 255]).collect()
    }
}

/// Words in a GP0 command, by its first byte (poly-lines aside).
fn command_len(op: u32) -> usize {
    match op {
        0x02 => 3,
        0x20..=0x3f => {
            let n = if op & 0x08 != 0 { 4 } else { 3 };
            let tex = (op & 0x04 != 0) as usize;
            let g = (op & 0x10 != 0) as usize;
            1 + n * (1 + tex) + g * (n - 1)
        }
        0x40..=0x5f => {
            if op & 0x10 != 0 {
                4
            } else {
                3
            }
        }
        0x60..=0x7f => 2 + (op & 0x04 != 0) as usize + ((op >> 3) & 3 == 0) as usize,
        0x80..=0x9f => 4,
        0xa0..=0xdf => 3,
        _ => 1,
    }
}

fn rgb_of(w: u32) -> [i32; 3] {
    [(w & 0xff) as i32, ((w >> 8) & 0xff) as i32, ((w >> 16) & 0xff) as i32]
}

fn to15(c: [i32; 3]) -> u16 {
    let ch = |v: i32| (v.clamp(0, 255) >> 3) as u16;
    ch(c[0]) | ch(c[1]) << 5 | ch(c[2]) << 10
}

/// Top-left rule: an edge is excluded when it is a right or bottom edge.
fn excluded(p: &Vert, q: &Vert, area: i64) -> bool {
    let (dx, dy) = (q.x - p.x, q.y - p.y);
    let (dx, dy) = if area > 0 { (dx, dy) } else { (-dx, -dy) };
    // For counter-clockwise (area > 0 in screen space, y down) winding, an
    // edge going up or going right horizontally is a left or top edge.
    !(dy < 0 || (dy == 0 && dx > 0))
}
