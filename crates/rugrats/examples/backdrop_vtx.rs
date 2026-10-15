fn main() {
    let mut out = Vec::new();
    rugrats::front::Backdrop::default().triangles(&mut out);
    for v in out.iter().take(6) {
        println!(
            "{:?} uv {:04x} u {} v {} mode {:08x} window {:08x}",
            v.pos,
            v.uv,
            v.uv & 255,
            (v.uv >> 8) & 255,
            v.mode,
            v.window
        );
    }
}
