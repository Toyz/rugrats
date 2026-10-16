//! rugrats-re: the reverse engineering toolbox for Rugrats: Search for Reptar.

#![forbid(unsafe_code)]

mod bios;
mod docs;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use rrt::dis::analysis::{self, Image, find_gp};
use rrt::dis::exe::PsxExe;
use rrt::dis::listing::Listing;
use rrt::dis::mips::Mips;
use rrt::dis::pseudo::Pseudo;
use rrt::dis::{Endian, Symbols};
use rrt::disc::{self, TrackKind};

const USAGE: &str = "usage: rugrats-re <command> ...

  disc info                 tracks, volume descriptor, sector kinds
  disc ls                   every file: LBA, size, XA attributes
  disc extract [DIR]        files to DIR (default work/fs); Form 2 files as raw 2352-byte sectors
  disc cat PATH             one file's Form 1 contents to stdout
  disc audio [DIR]          CD-DA tracks as WAV (default work/audio)
  tim FILE|DIR [OUT] [--clut N]
                            TIM to PNG; a directory converts every TIM under it (default work/png)
  vab DIR|VH [OUT]          check every VAB (VH with its VB) under DIR, or write one bank's samples as WAV
  exe info FILE             PS-X EXE header
  files                     the executable's file table, each entry against the ISO directory
  level FILE [--rooms]      a LEVELnn.BIN's sections and rooms
  disasm FILE [--from ADDR] [--to ADDR] [--count N] [--gp ADDR]
                            disassemble a PS-X EXE (or a raw image with --base ADDR),
                            with function labels, BIOS calls, and the strings the code points at
  disasm FILE --func ADDR|NAME
                            one whole function
  funcs FILE                functions found by recursive descent: size, calls, callers, flags
                            (i indirect calls, u unresolved jr, ! hit invalid, t tail calls)
  calls FILE [ADDR|NAME] [--depth N]
                            the call tree under a function (default the entry)
  callers FILE ADDR|NAME [--depth N]
                            the tree of callers above a function, through interface slots too
  xrefs FILE ADDR|NAME      calls, jumps, data references and pointers to an address
  slots FILE                interface slots: fixed words holding function addresses, and their jalr uses
  pseudo FILE ADDR|NAME     C-like pseudo-code for a function (a reading aid)
  pseudo FILE --all [--dump]
                            check that every function decompiles; --dump prints them all
  strings FILE              strings the code references, with the functions that use them
  docs index|check          the reference index under docs/

The analysis, listings and pseudo-code are rrt-dis's. Names come from
symbols/<stem>.txt (rrt-dis's symbol file: `0xADDRESS name [func|data|label]
[size=N] # (sure|likely|guess) note`, and `@data START END` for data inside the
code); functions it leaves unnamed are bios_NAME for a BIOS stub, else fn_ADDRESS.

FILE may be `disc:PATH` to read straight off the CD. The disc is the one .cue
under work/disc unless --cue is given.";

type Result<T> = std::result::Result<T, String>;

/// The repository root: the nearest directory holding cairns.toml.
fn root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join("cairns.toml").exists() {
            return dir;
        }
        if !dir.pop() {
            return std::env::current_dir().unwrap();
        }
    }
}

struct Args {
    items: Vec<String>,
}

impl Args {
    /// Removes `--name VALUE` and returns VALUE.
    fn take(&mut self, name: &str) -> Option<String> {
        let at = self.items.iter().position(|a| a == name)?;
        self.items.remove(at);
        (at < self.items.len()).then(|| self.items.remove(at))
    }

    /// Removes the flag `name`, if given.
    fn take_flag(&mut self, name: &str) -> bool {
        let at = self.items.iter().position(|a| a == name);
        at.map(|at| self.items.remove(at)).is_some()
    }
}

fn parse_num(text: &str) -> Result<u32> {
    let t = text.trim_start_matches("0x");
    if t.len() != text.len() || t.chars().any(|c| c.is_ascii_alphabetic()) {
        u32::from_str_radix(t, 16).map_err(|e| format!("{text}: {e}"))
    } else {
        t.parse().map_err(|e| format!("{text}: {e}"))
    }
}

fn open_disc(cue: &Option<String>) -> Result<disc::Image> {
    let path = match cue {
        Some(p) => PathBuf::from(p),
        None => disc::Image::find(&root().join("work/disc")).map_err(|e| e.to_string())?,
    };
    disc::Image::open(&path).map_err(|e| e.to_string())
}

fn read_input(file: &str, cue: &Option<String>) -> Result<Vec<u8>> {
    if let Some(path) = file.strip_prefix("disc:") {
        let disc = open_disc(cue)?;
        let iso = disc.iso().map_err(|e| e.to_string())?;
        let entry = iso.find(path).map_err(|e| e.to_string())?;
        iso.read(&entry).map_err(|e| e.to_string())
    } else {
        std::fs::read(file).map_err(|e| format!("{file}: {e}"))
    }
}

fn xa_text(attr: Option<u16>) -> String {
    use rrt::disc::iso::xa;
    let Some(a) = attr else { return "-".into() };
    let mut parts = Vec::new();
    for (bit, name) in
        [(xa::FORM1, "f1"), (xa::FORM2, "f2"), (xa::INTERLEAVED, "il"), (xa::CDDA, "da"), (xa::DIRECTORY, "dir")]
    {
        if a & bit != 0 {
            parts.push(name);
        }
    }
    format!("{a:04x}:{}", parts.join("+"))
}

fn disc_cmd(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let sub = if args.items.is_empty() { String::new() } else { args.items.remove(0) };
    let disc = open_disc(cue)?;
    let iso = disc.iso().map_err(|e| e.to_string())?;
    match sub.as_str() {
        "info" => {
            let cue = disc.cue.as_ref().ok_or("the disc was not opened through a CUE sheet")?;
            println!("cue        {}", cue.path.display());
            for t in &cue.tracks {
                let sectors = t.file_sectors().map_err(|e| e.to_string())?;
                let secs = sectors as f64 / 75.0;
                println!(
                    "track {:02}   {:?}  {} sectors ({:.0}:{:05.2})  index01 at {}",
                    t.number,
                    t.kind,
                    sectors,
                    (secs / 60.0).floor(),
                    secs % 60.0,
                    t.index1
                );
            }
            let p = &iso.pvd;
            println!("system     {:?}", p.system_id);
            println!("volume     {:?}  {} blocks", p.volume_id, p.volume_blocks);
            println!("publisher  {:?}", p.publisher);
            println!("preparer   {:?}", p.preparer);
            println!("app        {:?}", p.application);
            println!("created    {:?}", p.created);
            // Sector kinds across the data track, by submode.
            let mut kinds = std::collections::BTreeMap::new();
            for lba in 0..disc.sectors() {
                let s = disc.sector(lba).map_err(|e| e.to_string())?;
                *kinds.entry((s.mode(), s.submode() & 0x7e)).or_insert(0u32) += 1;
            }
            println!("data track {} sectors", disc.sectors());
            for ((mode, sm), n) in kinds {
                println!("  mode {mode} submode {sm:02x}: {n}");
            }
            // Runs of sectors whose submode differs from plain Form 1 data,
            // with the file that owns them.
            let files = iso.walk().map_err(|e| e.to_string())?;
            let owner = |lba: u32| {
                files
                    .iter()
                    .find(|f| !f.is_dir && lba >= f.lba && lba < f.lba + f.sectors())
                    .map_or("(no file)", |f| f.path.as_str())
            };
            let mut lba = 0;
            while lba < disc.sectors() {
                let sub = disc.sector(lba).map_err(|e| e.to_string())?.subheader();
                if sub[2] & 0x7e == 0x08 {
                    lba += 1;
                    continue;
                }
                let start = lba;
                while lba < disc.sectors() && disc.sector(lba).map_err(|e| e.to_string())?.subheader() == sub {
                    lba += 1;
                }
                println!("  {start}..{lba}: subheader {:02x?}  {}", sub, owner(start));
            }
        }
        "ls" => {
            println!("{:>7} {:>10} {:>6} {:>14}  path", "lba", "size", "secs", "xa");
            for e in iso.walk().map_err(|e| e.to_string())? {
                let d = e.date;
                println!(
                    "{:>7} {:>10} {:>6} {:>14}  {}{}   {}-{:02}-{:02} {:02}:{:02}",
                    e.lba,
                    e.size,
                    e.sectors(),
                    xa_text(e.xa_attr),
                    e.path,
                    if e.is_dir { "/" } else { "" },
                    1900 + d[0] as u32,
                    d[1],
                    d[2],
                    d[3],
                    d[4]
                );
            }
        }
        "extract" => {
            let out = args.items.first().map(PathBuf::from).unwrap_or_else(|| root().join("work/fs"));
            let mut files = 0;
            let mut raw = 0;
            for e in iso.walk().map_err(|e| e.to_string())? {
                let dest = out.join(e.path.trim_start_matches('/'));
                if e.is_dir {
                    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
                    continue;
                }
                // CD-DA files are directory entries pointing into the audio
                // tracks; `disc audio` writes those.
                if e.xa_attr.is_some_and(|a| a & rrt::disc::iso::xa::CDDA != 0) {
                    continue;
                }
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                // A file is written raw when any of its sectors is Form 2,
                // whatever its directory record claims.
                let form2 = e.is_form2()
                    || (0..e.sectors()).any(|i| disc.sector(e.lba + i).map(|s| s.is_form2()).unwrap_or(false));
                let bytes = if form2 {
                    raw += 1;
                    let mut raw = Vec::with_capacity(e.sectors() as usize * rrt::disc::RAW_SECTOR);
                    for i in 0..e.sectors() {
                        raw.extend_from_slice(&disc.sector(e.lba + i).map_err(|e| e.to_string())?.raw[..]);
                    }
                    raw
                } else {
                    iso.read(e).map_err(|e| e.to_string())?
                };
                std::fs::write(&dest, bytes).map_err(|err| format!("{}: {err}", dest.display()))?;
                files += 1;
            }
            println!("{files} files ({raw} raw 2352-byte) -> {}", out.display());
        }
        "cat" => {
            let path = args.items.first().ok_or("disc cat PATH")?;
            let e = iso.find(path).map_err(|e| e.to_string())?;
            let bytes = iso.read(&e).map_err(|e| e.to_string())?;
            std::io::stdout().write_all(&bytes).map_err(|e| e.to_string())?;
        }
        "audio" => {
            let out = args.items.first().map(PathBuf::from).unwrap_or_else(|| root().join("work/audio"));
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            let cue = disc.cue.as_ref().ok_or("the disc was not opened through a CUE sheet")?;
            for t in cue.tracks.iter().filter(|t| t.kind == TrackKind::Audio) {
                let pcm = std::fs::read(&t.file).map_err(|e| format!("{}: {e}", t.file.display()))?;
                let pcm = &pcm[t.index1 as usize * rrt::disc::RAW_SECTOR..];
                let dest = out.join(format!("track{:02}.wav", t.number));
                write_wav(&dest, pcm).map_err(|e| e.to_string())?;
                println!("{} ({:.1} s)", dest.display(), pcm.len() as f64 / 176400.0);
            }
        }
        _ => return Err(USAGE.into()),
    }
    Ok(())
}

/// 44.1 kHz 16-bit stereo little-endian, as CD-DA is stored.
fn write_wav(path: &Path, pcm: &[u8]) -> std::io::Result<()> {
    let mut w = std::io::BufWriter::new(std::fs::File::create(path)?);
    let len = pcm.len() as u32;
    w.write_all(b"RIFF")?;
    w.write_all(&(36 + len).to_le_bytes())?;
    w.write_all(b"WAVEfmt ")?;
    w.write_all(&16u32.to_le_bytes())?;
    w.write_all(&1u16.to_le_bytes())?;
    w.write_all(&2u16.to_le_bytes())?;
    w.write_all(&44100u32.to_le_bytes())?;
    w.write_all(&(44100u32 * 4).to_le_bytes())?;
    w.write_all(&4u16.to_le_bytes())?;
    w.write_all(&16u16.to_le_bytes())?;
    w.write_all(b"data")?;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(pcm)?;
    w.flush()
}

fn tim_cmd(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let clut = args.take("--clut").map(|s| parse_num(&s)).transpose()?.unwrap_or(0) as usize;
    let input = args.items.first().ok_or("tim FILE|DIR [OUT]")?.clone();
    let convert = |bytes: &[u8], out: &Path| -> Result<String> {
        let tim = rugrats_data::Tim::parse(bytes).map_err(|e| e.to_string())?;
        let png = rrt::image::png::encode(tim.width() as u32, tim.height() as u32, &tim.to_rgba(clut));
        std::fs::write(out, png).map_err(|e| e.to_string())?;
        Ok(format!(
            "{:?} {}x{} at vram ({}, {}){}",
            tim.mode,
            tim.width(),
            tim.height(),
            tim.rect.x,
            tim.rect.y,
            tim.clut.as_ref().map_or(String::new(), |(r, _)| format!(", {} palettes at ({}, {})", r.h, r.x, r.y))
        ))
    };
    let path = PathBuf::from(&input);
    if path.is_dir() {
        let out = args.items.get(1).map(PathBuf::from).unwrap_or_else(|| root().join("work/png"));
        let mut stack = vec![path.clone()];
        let (mut ok, mut bad) = (0, 0);
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                let name = p.file_name().unwrap().to_string_lossy().to_uppercase();
                if !name.ends_with("TIM") {
                    continue;
                }
                let rel = p.strip_prefix(&path).unwrap();
                let dest = out.join(rel).with_extension("png");
                std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
                match convert(&std::fs::read(&p).map_err(|e| e.to_string())?, &dest) {
                    Ok(_) => ok += 1,
                    Err(e) => {
                        bad += 1;
                        eprintln!("{}: {e}", p.display());
                    }
                }
            }
        }
        println!("{ok} TIMs converted, {bad} failed -> {}", out.display());
    } else {
        let out = args.items.get(1).map(PathBuf::from).unwrap_or_else(|| path.with_extension("png"));
        println!("{}", convert(&read_input(&input, cue)?, &out)?);
    }
    Ok(())
}

/// A mono 16-bit WAV file.
fn wav(samples: &[i16], rate: u32) -> Vec<u8> {
    let len = samples.len() as u32 * 2;
    let mut w = Vec::with_capacity(44 + len as usize);
    w.extend(b"RIFF");
    w.extend((36 + len).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend((rate * 2).to_le_bytes());
    w.extend(2u16.to_le_bytes());
    w.extend(16u16.to_le_bytes());
    w.extend(b"data");
    w.extend(len.to_le_bytes());
    for s in samples {
        w.extend(s.to_le_bytes());
    }
    w
}

fn vab_cmd(args: &mut Args) -> Result<()> {
    use rrt::kit::libsnd::vab;
    let input = PathBuf::from(args.items.first().ok_or("vab DIR|VH [OUT]")?);
    let body = |vh: &Path| {
        let name = vh.file_name().unwrap().to_string_lossy();
        vh.with_file_name(format!("{}VB", &name[..name.len() - 2]))
    };
    if input.is_dir() {
        let mut stack = vec![input];
        let (mut ok, mut bad, mut samples) = (0, 0, 0);
        while let Some(dir) = stack.pop() {
            for e in std::fs::read_dir(&dir).map_err(|e| e.to_string())?.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if !p.file_name().unwrap().to_string_lossy().ends_with("VH") {
                    continue;
                }
                let vh = std::fs::read(&p).map_err(|e| e.to_string())?;
                let vb = std::fs::read(body(&p)).unwrap_or_default();
                match vab::Vab::parse(&vh) {
                    Ok(v) => {
                        let total: u32 = v.vag_sizes.iter().sum();
                        if total as usize != vb.len() {
                            bad += 1;
                            eprintln!("{}: samples total {total}, VB is {}", p.display(), vb.len());
                        } else {
                            ok += 1;
                            samples += v.vag_sizes.len() - 1;
                        }
                    }
                    Err(e) => {
                        bad += 1;
                        eprintln!("{}: {e}", p.display());
                    }
                }
            }
        }
        println!("{ok} banks parse and match their VB ({samples} samples), {bad} do not");
    } else {
        let vh = std::fs::read(&input).map_err(|e| e.to_string())?;
        let vb = std::fs::read(body(&input)).map_err(|e| e.to_string())?;
        let v = vab::Vab::parse(&vh)?;
        let out = args.items.get(1).map(PathBuf::from).unwrap_or_else(|| root().join("work/wav"));
        std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        println!("{} programs, {} tones, {} samples", v.programs.len(), v.tones.len(), v.vag_sizes.len() - 1);
        for t in &v.tones {
            println!(
                "  program {:>3} vag {:>3} notes {:>3}-{:>3} center {:>3}.{:<3} vol {:>3} pan {:>3} adsr {:04x} {:04x}",
                t.program, t.vag, t.min_note, t.max_note, t.center, t.shift, t.volume, t.pan, t.adsr1, t.adsr2
            );
        }
        let stem = input.file_name().unwrap().to_string_lossy().to_string();
        for n in 1..v.vag_sizes.len() {
            let pcm = rrt::kit::adpcm::spu::decode(v.vag(&vb, n).ok_or("sample past the VB")?);
            // A VAB does not store the recording rate; 22050 Hz is assumed.
            std::fs::write(out.join(format!("{stem}_{n:02}.wav")), wav(&pcm, 22050)).map_err(|e| e.to_string())?;
        }
        println!("-> {}", out.display());
    }
    Ok(())
}

fn exe_info(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let file = args.items.get(1).ok_or("exe info FILE")?;
    let exe = PsxExe::parse(&read_input(file, cue)?)?;
    let t_size = exe.image.len() as u32;
    println!("pc0     0x{:08x}", exe.pc0);
    println!("gp0     0x{:08x}", exe.gp0);
    println!("text    0x{:08x} .. 0x{:08x} ({t_size} bytes)", exe.t_addr, exe.t_addr + t_size);
    println!("data    0x{:08x} ({} bytes)", exe.d_addr, exe.d_size);
    println!("bss     0x{:08x} .. 0x{:08x} ({} bytes)", exe.b_addr, exe.b_addr + exe.b_size, exe.b_size);
    println!("stack   0x{:08x} + 0x{:x}", exe.s_addr, exe.s_size);
    println!("marker  {:?}", exe.marker);
    Ok(())
}

/// The symbol file `symbols/<stem>.txt`, empty when there is none.
fn load_symbols(file: &str) -> Result<Symbols> {
    let stem = file.trim_start_matches("disc:").rsplit(['/', '\\']).next().unwrap_or(file);
    let stem = stem.split('.').next().unwrap_or(stem).to_lowercase();
    let path = root().join("symbols").join(format!("{stem}.txt"));
    let Ok(text) = std::fs::read_to_string(&path) else { return Ok(Symbols::new()) };
    Symbols::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes a command's output, ending quietly when the reader has gone
/// (`| head`).
fn emit(text: &str) -> Result<()> {
    match std::io::stdout().lock().write_all(text.as_bytes()) {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => Err(e.to_string()),
        _ => Ok(()),
    }
}

/// The R3000A, which the analysis borrows for the life of the command.
static MIPS: Mips = Mips::r3000();

/// An executable analysed by rrt-dis. Its listing names what the symbol
/// file names, then BIOS stubs as `bios_NAME` and other functions as
/// `fn_XXXXXXXX`, so every command and the pseudo-code agree.
struct Program {
    entry: u32,
    names: &'static Symbols,
    a: &'static analysis::Program<'static, Mips>,
    listing: Listing<'static, 'static, Mips>,
}

impl Program {
    fn load(args: &mut Args, cue: &Option<String>) -> Result<Program> {
        let base = args.take("--base").map(|s| parse_num(&s)).transpose()?;
        let gp_arg = args.take("--gp").map(|s| parse_num(&s)).transpose()?;
        let file = args.items.first().ok_or("FILE")?.clone();
        let bytes = read_input(&file, cue)?;
        let (base, image, entry, exe_gp) = match base {
            Some(b) => (b, bytes, b, None),
            None => {
                let exe = PsxExe::parse(&bytes)?;
                (exe.t_addr, exe.image, exe.pc0, (exe.gp0 != 0).then_some(exe.gp0))
            }
        };
        // The analysis, its listing and the names live as long as the
        // command.
        let image = Image { base, bytes: Box::leak(image.into_boxed_slice()) };
        let gp = gp_arg.or(exe_gp).or_else(|| find_gp(&MIPS, image, entry));
        let mut names = load_symbols(&file)?;
        let a: &'static _ = Box::leak(Box::new(analysis::Program::analyze(&MIPS, image, entry, gp, &names)));
        for &f in a.funcs.keys() {
            if names.at(f).is_some() {
                continue;
            }
            let name = match bios::stub(image, f).and_then(bios::name) {
                Some(n) if names.get(&format!("bios_{n}")).is_none() => format!("bios_{n}"),
                _ => format!("fn_{f:08x}"),
            };
            names.insert(name, f);
        }
        let names: &'static Symbols = Box::leak(Box::new(names));
        Ok(Program { entry, names, a, listing: Listing::new(a, names) })
    }

    fn image(&self) -> Image<'static> {
        self.a.image
    }

    fn name(&self, addr: u32) -> String {
        self.listing.name(addr)
    }

    /// An address given as a name or a number.
    fn resolve(&self, text: &str) -> Result<u32> {
        self.names.addr(text).map_or_else(|| parse_num(text), Ok)
    }

    /// The function starting at, or holding, an address given as a name or
    /// a number.
    fn func(&self, text: &str) -> Result<u32> {
        let addr = self.resolve(text)?;
        match self.a.funcs.get(&addr).or_else(|| self.a.func_of(addr)) {
            Some(f) => Ok(f.start),
            None => Err(format!("no function at 0x{addr:08x}")),
        }
    }

    /// The name of the function holding `site`, else the site.
    fn func_name(&self, site: u32) -> String {
        self.a.func_of(site).map_or_else(|| format!("{site:08x}"), |f| self.name(f.start))
    }
}

fn disasm(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let from = args.take("--from").map(|s| parse_num(&s)).transpose()?;
    let to = args.take("--to").map(|s| parse_num(&s)).transpose()?;
    let count = args.take("--count").map(|s| parse_num(&s)).transpose()?;
    let func = args.take("--func");
    let p = Program::load(args, cue)?;
    let image = p.image();
    let text = match func {
        Some(f) => p.listing.function(p.func(&f)?).unwrap_or_default(),
        None => {
            let start = from.unwrap_or(image.base) & !3;
            let end = match (to, count) {
                (Some(t), _) => t,
                (None, Some(n)) => start.saturating_add(n.saturating_mul(4)),
                (None, None) => image.end(),
            };
            p.listing.range(start, end.min(image.end()))
        }
    };
    emit(&text)
}

fn funcs(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let p = Program::load(args, cue)?;
    let a = p.a;
    let mut by: BTreeMap<String, usize> = BTreeMap::new();
    for f in a.funcs.values() {
        *by.entry(format!("{:?}", f.found).to_lowercase()).or_default() += 1;
    }
    let by: Vec<String> = by.iter().map(|(k, n)| format!("{k} {n}")).collect();
    let regions: Vec<String> = a.code.iter().map(|(s, e)| format!("0x{s:08x}..0x{e:08x}")).collect();
    let resolved = a.indirect.values().filter(|s| a.slots.get(s).is_some_and(|t| t.len() == 1)).count();
    let unresolved: u32 = a.funcs.values().map(|f| f.unresolved_jumps).sum();
    let mut out = String::new();
    let _ = writeln!(out, "entry        0x{:08x}", p.entry);
    let _ = writeln!(out, "gp           {}", a.gp.map_or("unknown".into(), |g| format!("0x{g:08x}")));
    let _ = writeln!(out, "code         {}, {} words unowned", regions.join(" "), a.unowned_words());
    let _ = writeln!(out, "functions    {} ({})", a.funcs.len(), by.join(", "));
    let _ = writeln!(out, "jump tables  {}", a.jump_tables.len());
    let _ = writeln!(
        out,
        "slots        {} hold functions; {} jalr sites load from one, {resolved} resolved to one callee",
        a.slots.len(),
        a.indirect.len()
    );
    let _ = writeln!(out, "unresolved   {unresolved} jumps");
    let _ = writeln!(out, "{:>8} {:>6} {:>5} {:>5} {:>4}  name", "start", "bytes", "calls", "calrs", "flags");
    for f in a.funcs.values() {
        let callers = a.callers.get(&f.start).map_or(0, |c| c.len());
        let mut flags = String::new();
        for (on, c) in [
            (f.indirect_calls > 0, 'i'),
            (f.unresolved_jumps > 0, 'u'),
            (f.hit_invalid, '!'),
            (!f.tail_calls.is_empty(), 't'),
        ] {
            if on {
                flags.push(c);
            }
        }
        let _ = writeln!(
            out,
            "{:08x} {:>6} {:>5} {:>5} {:>4}  {}",
            f.start,
            f.end - f.start,
            f.calls.len(),
            callers,
            flags,
            p.name(f.start)
        );
    }
    emit(&out)
}

fn strings(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let p = Program::load(args, cue)?;
    let mut out = String::new();
    for (target, (s, sites)) in p.a.strings(3) {
        let users: BTreeSet<String> = sites.iter().map(|&site| p.func_name(site)).collect();
        let users: Vec<String> = users.into_iter().collect();
        let _ = writeln!(out, "{target:08x} {:<50} {}", format!("{s:?}"), users.join(" "));
    }
    emit(&out)
}

/// The call tree under a function, each callee once.
fn calls(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let depth = args.take("--depth").map(|s| parse_num(&s)).transpose()?.unwrap_or(4);
    let root_arg = args.items.get(1).cloned();
    let p = Program::load(args, cue)?;
    let root = match root_arg {
        Some(r) => p.resolve(&r)?,
        None => p.entry,
    };
    fn go(p: &Program, out: &mut String, f: u32, d: u32, max: u32, seen: &mut BTreeSet<u32>) {
        let Some(func) = p.a.funcs.get(&f) else { return };
        let again = !seen.insert(f);
        let _ = writeln!(
            out,
            "{}{} 0x{f:08x}{}{}",
            "  ".repeat(d as usize),
            p.name(f),
            if func.indirect_calls > 0 { format!(" [{} indirect]", func.indirect_calls) } else { String::new() },
            if again { " ..." } else { "" }
        );
        if again || d >= max {
            return;
        }
        for &c in func.calls.iter().chain(func.tail_calls.iter()) {
            go(p, out, c, d + 1, max, seen);
        }
    }
    let mut out = String::new();
    go(&p, &mut out, root, 0, depth, &mut BTreeSet::new());
    emit(&out)
}

/// Interface slots: fixed words holding function addresses, the functions
/// stored in each, and how many `jalr` sites call through it.
fn slots(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let p = Program::load(args, cue)?;
    let mut uses: BTreeMap<u32, u32> = BTreeMap::new();
    for &slot in p.a.indirect.values() {
        *uses.entry(slot).or_default() += 1;
    }
    let all: BTreeSet<u32> = p.a.slots.keys().chain(uses.keys()).copied().collect();
    let mut out = String::new();
    for slot in all {
        let fns =
            p.a.slots.get(&slot).map_or("-".into(), |t| t.iter().map(|&f| p.name(f)).collect::<Vec<_>>().join(" "));
        let _ = writeln!(out, "{slot:08x} {:>4} calls  {} {fns}", uses.get(&slot).copied().unwrap_or(0), p.name(slot));
    }
    emit(&out)
}

/// The tree of callers above a function: who calls it, who calls them.
/// Calls through resolved interface slots count.
fn callers(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let depth = args.take("--depth").map(|s| parse_num(&s)).transpose()?.unwrap_or(4);
    let target = args.items.get(1).cloned().ok_or("callers FILE ADDR|NAME")?;
    let p = Program::load(args, cue)?;
    let root = p.resolve(&target)?;
    // The jalr sites calling through a slot that may hold each function;
    // rrt-dis's callers have those whose slot only ever holds one.
    let mut via_slot: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (&site, slot) in &p.a.indirect {
        for &f in p.a.slots.get(slot).into_iter().flatten() {
            via_slot.entry(f).or_default().push(site);
        }
    }
    type Via = BTreeMap<u32, Vec<u32>>;
    fn go(p: &Program, via: &Via, out: &mut String, f: u32, d: u32, max: u32, seen: &mut BTreeSet<u32>) {
        let again = !seen.insert(f);
        let _ = writeln!(out, "{}{} 0x{f:08x}{}", "  ".repeat(d as usize), p.name(f), if again { " ..." } else { "" });
        if again || d >= max {
            return;
        }
        let sites = p.a.callers.get(&f).into_iter().flatten().chain(via.get(&f).into_iter().flatten());
        let up: BTreeSet<u32> = sites.filter_map(|&site| p.a.func_of(site)).map(|c| c.start).collect();
        for c in up {
            go(p, via, out, c, d + 1, max, seen);
        }
    }
    let mut out = String::new();
    go(&p, &via_slot, &mut out, root, 0, depth, &mut BTreeSet::new());
    emit(&out)
}

/// C-like pseudo-code for a function (rrt-dis's decompiler); with `--all` a
/// check that every function decompiles, and with `--all --dump` every
/// function's text.
fn pseudo(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let dump = args.take_flag("--dump");
    let target = args.items.get(1).cloned().ok_or("pseudo FILE ADDR|NAME|--all [--dump]")?;
    let p = Program::load(args, cue)?;
    let pseudo = Pseudo::new(&p.listing);
    if target != "--all" {
        return emit(&pseudo.function(p.func(&target)?).unwrap_or_default());
    }
    let (mut ok, mut bad) = (0, 0);
    let mut out = String::new();
    for &f in p.a.funcs.keys() {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| pseudo.function(f))) {
            Ok(Some(t)) => {
                ok += 1;
                if dump {
                    out.push_str(&t);
                }
            }
            Ok(None) => {
                bad += 1;
                let _ = writeln!(out, "{f:08x}: no blocks");
            }
            Err(_) => {
                bad += 1;
                let _ = writeln!(out, "{f:08x}: panicked");
            }
        }
    }
    let _ = writeln!(out, "{ok} functions decompiled, {bad} failed");
    emit(&out)
}

/// Everything that refers to an address: calls, jumps, data references
/// (rrt-dis's `Ref`s), calls through a slot holding it, and words holding it.
fn xrefs(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let target = args.items.get(1).cloned().ok_or("xrefs FILE ADDR")?;
    let p = Program::load(args, cue)?;
    let target = p.resolve(&target)?;
    let image = p.image();
    let mut sites: BTreeMap<u32, String> = BTreeMap::new();
    for &(site, how) in p.a.refs.get(&target).into_iter().flatten() {
        sites.insert(site, format!("{how:?}").to_lowercase());
    }
    for &site in p.a.callers.get(&target).into_iter().flatten() {
        sites.entry(site).or_insert_with(|| "slot call".into());
    }
    let mut out = String::new();
    for (site, how) in sites {
        let mnemonic = image.word(site, Endian::Little).map_or("", |w| MIPS.decode_word(w, site).mnemonic);
        let _ = writeln!(out, "{site:08x} {how:<9} {mnemonic:<6}  in {}", p.func_name(site));
    }
    // Words in the image holding the address.
    let mut a = image.base;
    while a + 4 <= image.end() {
        if image.word(a, Endian::Little) == Some(target) {
            let _ = writeln!(out, "{a:08x} pointer");
        }
        a += 4;
    }
    emit(&out)
}

/// The executable's file table against the ISO directory: each entry's file,
/// then the files no entry reaches.
fn files_cmd(cue: &Option<String>) -> Result<()> {
    let disc = open_disc(cue)?;
    let iso = disc.iso().map_err(|e| e.to_string())?;
    let exe =
        PsxExe::parse(&iso.read(&iso.find("/SLUS_006.50").map_err(|e| e.to_string())?).map_err(|e| e.to_string())?)?;
    let table = rugrats_data::files::table(&exe.image, exe.t_addr).ok_or("the file table is not in the image")?;
    let all: Vec<_> = iso.walk().map_err(|e| e.to_string())?.iter().filter(|e| !e.is_dir).collect();
    let mut reached = BTreeSet::new();
    println!("{:>4} {:>7} {:>9}  path", "file", "lba", "size");
    for (i, e) in table.iter().enumerate() {
        if !e.is_used() {
            println!("{i:>4} {:>7} {:>9}  -", 0, e.size);
            continue;
        }
        let names: Vec<&str> = all.iter().filter(|f| f.lba == e.lba).map(|f| f.path.as_str()).collect();
        names.iter().for_each(|n| {
            reached.insert(n.to_string());
        });
        let name = if names.is_empty() { "(no file starts here)".to_string() } else { names.join(" ") };
        println!("{i:>4} {:>7} {:>9}  {name}", e.lba, e.size);
    }
    println!("\nnot in the table:");
    for f in all.iter().filter(|f| !reached.contains(&f.path)) {
        println!("     {:>7} {:>9}  {}", f.lba, f.size, f.path);
    }
    Ok(())
}

/// A LEVELnn.BIN's sections, as the room loader walks them.
fn level_cmd(args: &mut Args, cue: &Option<String>) -> Result<()> {
    let file = args.items.first().ok_or("level FILE")?.clone();
    let bytes = read_input(&file, cue)?;
    let l = rugrats_data::level::parse(&bytes).map_err(|e| e.to_string())?;
    let blobs: usize = l.rooms.iter().map(|r| r.blobs.len()).sum();
    println!(
        "{file}: {} a, {} quads, {} c, {} rooms ({blobs} blobs), tail count {} + {} bytes",
        l.a.len(),
        l.quads.len(),
        l.c.len(),
        l.rooms.len(),
        l.tail_count,
        l.tail.len()
    );
    if args.take_flag("--rooms") {
        for (i, r) in l.rooms.iter().enumerate() {
            let sizes: Vec<usize> = r.blobs.iter().map(Vec::len).collect();
            println!("  room {i:2}: quads {} boxes {} list {:?} blobs {sizes:?}", r.quads(), r.boxes(), r.list);
        }
    }
    Ok(())
}

fn main() {
    let mut args = Args { items: std::env::args().skip(1).collect() };
    let cue = args.take("--cue");
    let command = if args.items.is_empty() { String::new() } else { args.items.remove(0) };
    let result = match command.as_str() {
        "disc" => disc_cmd(&mut args, &cue),
        "exe" => exe_info(&mut args, &cue),
        "files" => files_cmd(&cue),
        "level" => level_cmd(&mut args, &cue),
        "tim" => tim_cmd(&mut args, &cue),
        "vab" => vab_cmd(&mut args),
        "disasm" => disasm(&mut args, &cue),
        "funcs" => funcs(&mut args, &cue),
        "strings" => strings(&mut args, &cue),
        "calls" => calls(&mut args, &cue),
        "xrefs" => xrefs(&mut args, &cue),
        "pseudo" => pseudo(&mut args, &cue),
        "callers" => callers(&mut args, &cue),
        "slots" => slots(&mut args, &cue),
        "docs" => docs::run(&root(), &args.items),
        _ => Err(USAGE.into()),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The symbol file is rrt-dis's: it parses, writes back byte for byte,
    /// and keeps the rodata between the asm and the C out of the code.
    #[test]
    fn the_symbol_file_is_rrt_dis_format() {
        let text = include_str!("../../../symbols/slus_006.txt");
        let symbols = Symbols::parse(text).unwrap();
        assert_eq!(symbols.to_text(), text);
        assert_eq!(symbols.data_ranges(), &[(0x8001_2760, 0x8001_7c0c)]);
        assert_eq!(symbols.addr("main"), Some(0x8001_85f8));
    }
}
