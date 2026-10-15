//! The game's printf, formatted on the host from the call's arguments.

use rrt::emu::psx::R3000;
use rrt::emu::{BusExt, Call};

use crate::bus::PsxBus;

fn cstr(c: &mut Call<'_, R3000, PsxBus>, at: u32) -> String {
    let mut out = Vec::new();
    for k in 0..512 {
        match c.bus.read_u8(at.wrapping_add(k)) {
            Ok(0) | Err(_) => break,
            Ok(b) => out.push(b),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `printf(fmt, ...)`: %d %i %u %x %X %c %s %p with flags, width and `l`.
pub fn format(c: &mut Call<'_, R3000, PsxBus>) -> String {
    format_from(c, 0)
}

/// The same with the format string in argument `first`.
pub fn format_from(c: &mut Call<'_, R3000, PsxBus>, first: usize) -> String {
    let at = c.arg32(first);
    let fmt = cstr(c, at);
    let mut arg = first + 1;
    let mut out = String::new();
    let mut chars = fmt.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        let mut spec = String::new();
        while let Some(&n) = chars.peek() {
            if "-+ #0123456789.l".contains(n) {
                spec.push(n);
                chars.next();
            } else {
                break;
            }
        }
        let Some(conv) = chars.next() else { break };
        let left = spec.contains('-');
        let zero = spec.starts_with('0') || spec.contains("-0");
        let width: usize = spec.trim_matches(|c: char| !c.is_ascii_digit()).split('.').next().unwrap_or("").parse().unwrap_or(0);
        let text = match conv {
            '%' => "%".to_string(),
            'd' | 'i' => {
                let v = c.arg32(arg) as i32;
                arg += 1;
                v.to_string()
            }
            'u' => {
                let v = c.arg32(arg);
                arg += 1;
                v.to_string()
            }
            'x' | 'p' => {
                let v = c.arg32(arg);
                arg += 1;
                format!("{v:x}")
            }
            'X' => {
                let v = c.arg32(arg);
                arg += 1;
                format!("{v:X}")
            }
            'c' => {
                let v = c.arg32(arg);
                arg += 1;
                char::from(v as u8).to_string()
            }
            's' => {
                let at = c.arg32(arg);
                arg += 1;
                cstr(c, at)
            }
            other => format!("%{spec}{other}"),
        };
        let pad = width.saturating_sub(text.len());
        if left {
            out.push_str(&text);
            out.extend(std::iter::repeat_n(' ', pad));
        } else if zero && conv != 's' {
            let (sign, digits) = text.strip_prefix('-').map_or(("", text.as_str()), |d| ("-", d));
            out.push_str(sign);
            out.extend(std::iter::repeat_n('0', pad));
            out.push_str(digits);
        } else {
            out.extend(std::iter::repeat_n(' ', pad));
            out.push_str(&text);
        }
    }
    out
}
