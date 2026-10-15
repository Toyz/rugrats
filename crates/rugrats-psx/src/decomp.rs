//! A small decompiler: MIPS to C-like pseudo-code, one basic block at a
//! time.
//!
//! Each block is executed symbolically: registers hold expression trees, so a
//! run of instructions folds into the expressions it computes. Stores, calls
//! and branches become statements. A register liveness pass decides what is
//! written: a value is shown only where it is used later, and named when it
//! is used more than once. Control flow stays as labels and gotos.
//!
//! Folded idioms: `lui`/`addiu` into addresses (and strings or names), the
//! stack frame (so `$sp` slots read as `local_NN`, and the prologue's register
//! saves and the epilogue's restores disappear), `$gp` slots into globals,
//! GCC's 64-bit fixed-point multiply (`mflo >> n | mfhi << 32-n`) into
//! `fx(a * b >> n)`, and GCC's divide-by-zero and overflow traps (dropped).
//! A call shows as many arguments as the callee reads, found by the same
//! liveness pass run on the callee.
//!
//! It is a reading aid. The port's code is written by hand from it and checked
//! against the original.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::rc::Rc;

use crate::mips::{GTE_CTRL, GTE_DATA, Insn, Op, REG, decode};
use crate::program::Program;

/// HI and LO as pseudo-registers after the 32 general ones.
const HI: usize = 32;
const LO: usize = 33;

fn rname(r: usize) -> &'static str {
    match r {
        HI => "hi",
        LO => "lo",
        _ => REG[r],
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// A register as the block began (or as last written out).
    Reg(usize),
    Const(i64),
    Bin(&'static str, Rc<Expr>, Rc<Expr>),
    Neg(Rc<Expr>),
    Not(Rc<Expr>),
    /// A load of `width` bytes, `signed`, from an address expression.
    Load(u8, bool, Rc<Expr>),
    /// The low or high word of a 64-bit product, signed or not.
    Lo(Rc<Expr>, Rc<Expr>, bool),
    Hi(Rc<Expr>, Rc<Expr>, bool),
    Quot(Rc<Expr>, Rc<Expr>, bool),
    Rem(Rc<Expr>, Rc<Expr>, bool),
    /// `(a * b) >> n` in 64 bits, the fixed-point multiply.
    MulShr(Rc<Expr>, Rc<Expr>, u32),
    Cop2(&'static str),
    /// A named temporary.
    Var(String),
}

use Expr::*;

fn rc(e: Expr) -> Rc<Expr> {
    Rc::new(e)
}

/// Names for addresses: functions, globals, strings.
pub trait Namer {
    fn name(&self, addr: u32) -> Option<String>;
}

impl<F: Fn(u32) -> Option<String>> Namer for F {
    fn name(&self, addr: u32) -> Option<String> {
        self(addr)
    }
}

const SAVED: [usize; 10] = [16, 17, 18, 19, 20, 21, 22, 23, 30, 31];
/// Registers a call may change: at, v0-v1, a0-a3, t0-t9, hi and lo.
const CLOBBERED: [usize; 19] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 24, 25, HI, LO];

struct Ctx<'a> {
    gp: Option<u32>,
    namer: &'a dyn Namer,
}

fn ty(width: u8, signed: bool) -> &'static str {
    match (width, signed) {
        (1, true) => "s8",
        (1, false) => "u8",
        (2, true) => "s16",
        (2, false) => "u16",
        _ => "s32",
    }
}

impl Ctx<'_> {
    fn constant(&self, v: i64) -> String {
        let u = v as u32;
        if (0x8000_0000..0x8020_0000).contains(&u) {
            return self.namer.name(u).unwrap_or(format!("0x{u:08x}"));
        }
        if !(-4096..=4096).contains(&v) { format!("0x{u:x}") } else { v.to_string() }
    }

    fn place(&self, addr: &Expr, width: u8, signed: bool) -> String {
        let t = ty(width, signed);
        let tag = if t == "s32" { String::new() } else { format!(" /*{t}*/") };
        if let Bin("+", a, b) = addr
            && let Const(off) = b.as_ref()
        {
            if let Reg(29) = a.as_ref() {
                return format!("local_{off:x}{tag}");
            }
            if let (Reg(28), Some(gp)) = (a.as_ref(), self.gp) {
                let at = gp.wrapping_add(*off as u32);
                return format!("{}{tag}", self.namer.name(at).unwrap_or(format!("g_{at:08x}")));
            }
            let sign = if *off < 0 { "-" } else { "" };
            return format!("{}->{sign}0x{:x}{tag}", self.expr(a), off.unsigned_abs());
        }
        if let Const(c) = addr {
            let at = *c as u32;
            return format!("{}{tag}", self.namer.name(at).unwrap_or(format!("*({t}*)0x{at:08x}")));
        }
        if let Reg(29) = addr {
            return format!("local_0{tag}");
        }
        if let Reg(_) | Var(_) = addr {
            return format!("{}->0x0{tag}", self.expr(addr));
        }
        // base[index], scaled by the element size.
        if let Bin("+", a, b) = addr
            && let Bin("<<", i, n) = a.as_ref()
            && let Const(n) = n.as_ref()
            && 1 << n == width as i64
        {
            return format!("{}[{}]{tag}", self.expr(b), self.expr(i));
        }
        format!("*({t}*)({})", self.expr(addr))
    }

    fn expr(&self, e: &Expr) -> String {
        match e {
            Reg(r) => rname(*r).to_string(),
            Const(v) => self.constant(*v),
            Bin("+", a, b) if matches!(a.as_ref(), Reg(29)) => match b.as_ref() {
                Const(off) => format!("&local_{off:x}"),
                _ => format!("(sp + {})", self.expr(b)),
            },
            Bin(op, a, b) => format!("({} {op} {})", self.expr(a), self.expr(b)),
            Neg(a) => format!("-{}", self.expr(a)),
            Not(a) => format!("~{}", self.expr(a)),
            Load(w, s, a) => self.place(a, *w, *s),
            Lo(a, b, s) => format!("lo({} *{} {})", self.expr(a), if *s { "" } else { "u" }, self.expr(b)),
            Hi(a, b, s) => format!("hi({} *{} {})", self.expr(a), if *s { "" } else { "u" }, self.expr(b)),
            Quot(a, b, s) => format!("({} /{} {})", self.expr(a), if *s { "" } else { "u" }, self.expr(b)),
            Rem(a, b, s) => format!("({} %{} {})", self.expr(a), if *s { "" } else { "u" }, self.expr(b)),
            MulShr(a, b, n) => format!("fx({} * {} >> {n})", self.expr(a), self.expr(b)),
            Cop2(n) => format!("gte.{n}"),
            Var(n) => n.clone(),
        }
    }

    /// A condition, without its outer parentheses.
    fn cond(&self, e: &Expr) -> String {
        match e {
            Bin(op, a, b) => format!("{} {op} {}", self.expr(a), self.expr(b)),
            _ => self.expr(e),
        }
    }
}

fn simplify(e: Expr) -> Expr {
    match &e {
        Bin(op, a, b) => match (*op, a.as_ref(), b.as_ref()) {
            ("+", Const(x), Const(y)) => Const((*x as i32).wrapping_add(*y as i32) as i64),
            ("|", Const(x), Const(y)) => Const(((*x as u32) | (*y as u32)) as i64),
            ("+" | "|" | "^" | "-" | "<<" | ">>" | "u>>", x, Const(0)) => x.clone(),
            ("+" | "|", Const(0), x) => x.clone(),
            ("<<", Const(x), Const(y)) => Const(((*x as u32) << (*y as u32 & 31)) as i32 as i64),
            // (x + a) + b
            ("+", Bin("+", x, c1), Const(c2)) => match c1.as_ref() {
                Const(c1) => {
                    let c = (*c1 as i32).wrapping_add(*c2 as i32) as i64;
                    if c == 0 { x.as_ref().clone() } else { Bin("+", x.clone(), rc(Const(c))) }
                }
                _ => e,
            },
            ("|", Bin("u>>", l, n1), Bin("<<", h, n2)) => match (l.as_ref(), h.as_ref(), n1.as_ref(), n2.as_ref()) {
                (Lo(a1, b1, true), Hi(a2, b2, true), Const(n), Const(m)) if a1 == a2 && b1 == b2 && n + m == 32 => {
                    MulShr(a1.clone(), b1.clone(), *n as u32)
                }
                _ => e,
            },
            _ => e,
        },
        _ => e,
    }
}

fn mentions(e: &Expr, regs: &[usize]) -> bool {
    match e {
        Reg(r) => regs.contains(r),
        Bin(_, a, b) | Lo(a, b, _) | Hi(a, b, _) | Quot(a, b, _) | Rem(a, b, _) | MulShr(a, b, _) => {
            mentions(a, regs) || mentions(b, regs)
        }
        Neg(a) | Not(a) | Load(_, _, a) => mentions(a, regs),
        _ => false,
    }
}

fn contains_load(e: &Expr) -> bool {
    match e {
        Load(..) | Cop2(_) => true,
        Bin(_, a, b) | Lo(a, b, _) | Hi(a, b, _) | Quot(a, b, _) | Rem(a, b, _) | MulShr(a, b, _) => {
            contains_load(a) || contains_load(b)
        }
        Neg(a) | Not(a) => contains_load(a),
        _ => false,
    }
}

/// `e` with every copy of `from` replaced by `to`.
fn replace(e: &Expr, from: &Expr, to: &Expr) -> Expr {
    if e == from {
        return to.clone();
    }
    let f = |x: &Rc<Expr>| rc(replace(x, from, to));
    match e {
        Bin(op, a, b) => Bin(op, f(a), f(b)),
        Lo(a, b, s) => Lo(f(a), f(b), *s),
        Hi(a, b, s) => Hi(f(a), f(b), *s),
        Quot(a, b, s) => Quot(f(a), f(b), *s),
        Rem(a, b, s) => Rem(f(a), f(b), *s),
        MulShr(a, b, n) => MulShr(f(a), f(b), *n),
        Neg(a) => Neg(f(a)),
        Not(a) => Not(f(a)),
        Load(w, s, a) => Load(*w, *s, f(a)),
        _ => e.clone(),
    }
}

fn size(e: &Expr) -> usize {
    match e {
        Bin(_, a, b) | Lo(a, b, _) | Hi(a, b, _) | Quot(a, b, _) | Rem(a, b, _) | MulShr(a, b, _) => {
            1 + size(a) + size(b)
        }
        Neg(a) | Not(a) | Load(_, _, a) => 1 + size(a),
        _ => 1,
    }
}

fn is_frame(r: usize, e: &Expr) -> bool {
    r == 29 && matches!(e, Bin("+", a, b) if matches!((a.as_ref(), b.as_ref()), (Reg(29), Const(_))))
}

/// An address as base and constant offset.
fn split(a: &Expr) -> (Expr, i64) {
    match a {
        Bin("+", x, c) => match c.as_ref() {
            Const(o) => (x.as_ref().clone(), *o),
            _ => (a.clone(), 0),
        },
        Const(o) => (Const(0), *o),
        _ => (a.clone(), 0),
    }
}

/// Whether a load in `e` may read what a store to `store` writes. Stack
/// slots are taken not to alias anything but stack slots, and the same base
/// at offsets four or more apart not to overlap.
fn aliases(e: &Expr, store: &Expr) -> bool {
    match e {
        Load(_, _, a) => {
            if aliases(a, store) {
                return true;
            }
            let ((b1, o1), (b2, o2)) = (split(a), split(store));
            let (sp1, sp2) = (b1 == Reg(29), b2 == Reg(29));
            if sp1 != sp2 {
                return false;
            }
            if b1 == b2 {
                return (o1 - o2).abs() < 4;
            }
            true
        }
        Cop2(_) => false,
        Bin(_, a, b) | Lo(a, b, _) | Hi(a, b, _) | Quot(a, b, _) | Rem(a, b, _) | MulShr(a, b, _) => {
            aliases(a, store) || aliases(b, store)
        }
        Neg(a) | Not(a) => aliases(a, store),
        _ => false,
    }
}

fn is_break(i: &Insn) -> bool {
    i.op == Op::Break
}

struct Block {
    start: u32,
    end: u32,
    succ: Vec<u32>,
}

/// How many arguments each function reads, from liveness at its entry.
struct Arity<'p> {
    p: &'p Program<'p>,
    cache: RefCell<BTreeMap<u32, usize>>,
}

impl Arity<'_> {
    fn get(&self, addr: u32) -> usize {
        if let Some(&n) = self.cache.borrow().get(&addr) {
            return n;
        }
        if !self.p.funcs.contains_key(&addr) {
            return 4;
        }
        // Recursion reads as all four until it is known.
        self.cache.borrow_mut().insert(addr, 4);
        let n = Flow::new(self.p, addr, self).map_or(4, |f| {
            let entry = &f.live_in[0];
            (4..8).rev().find(|r| entry.contains(r)).map_or(0, |r| r - 3)
        });
        self.cache.borrow_mut().insert(addr, n);
        n
    }
}

/// A function's blocks and register liveness.
struct Flow {
    addrs: BTreeSet<u32>,
    blocks: Vec<Block>,
    index: BTreeMap<u32, usize>,
    leaders: BTreeSet<u32>,
    /// Addresses that get a label: branch and jump targets.
    targets: BTreeSet<u32>,
    /// Branches that only guard a trap.
    guards: BTreeSet<u32>,
    /// Blocks that are only a trap.
    traps: BTreeSet<u32>,
    live_in: Vec<BTreeSet<usize>>,
}

impl Flow {
    fn new(p: &Program, start: u32, arity: &Arity) -> Option<Flow> {
        p.funcs.get(&start)?;
        let addrs: BTreeSet<u32> = p.owner.iter().filter(|(_, s)| **s == start).map(|(a, _)| *a).collect();
        let word = |a: u32| decode(p.mem.u32(a).unwrap_or(0));
        let mut leaders: BTreeSet<u32> = [start].into();
        let mut targets: BTreeSet<u32> = [start].into();
        for &a in &addrs {
            let i = word(a);
            if i.has_delay_slot() {
                if let Some(t) = i.target(a)
                    && addrs.contains(&t)
                    && !i.is_call()
                {
                    leaders.insert(t);
                    targets.insert(t);
                }
                // Calls return into the same block.
                if addrs.contains(&(a + 8)) && !matches!(i.op, Op::Jal | Op::Jalr) {
                    leaders.insert(a + 8);
                }
            }
            if is_break(&i) {
                leaders.insert(a);
                leaders.insert(a + 4);
            }
            if let Some(jt) = p.jump_tables.get(&a) {
                leaders.extend(jt.targets.iter().copied());
                targets.extend(jt.targets.iter().copied());
            }
        }
        let mut blocks: Vec<Block> = Vec::new();
        let mut cur: Option<u32> = None;
        let mut prev = 0;
        for &a in &addrs {
            if let Some(s) = cur
                && (leaders.contains(&a) || a != prev + 4)
            {
                blocks.push(Block { start: s, end: prev + 4, succ: vec![] });
                cur = None;
            }
            if cur.is_none() {
                cur = Some(a);
            }
            prev = a;
        }
        if let Some(s) = cur {
            blocks.push(Block { start: s, end: prev + 4, succ: vec![] });
        }
        // Traps, and the branches that skip them: a conditional branch just
        // before a trap block, to just after it.
        let traps: BTreeSet<u32> =
            blocks.iter().filter(|b| b.end == b.start + 4 && is_break(&word(b.start))).map(|b| b.start).collect();
        let mut guards = BTreeSet::new();
        for &t in &traps {
            for back in 1..=8u32 {
                let at = t.wrapping_sub(back * 4);
                let i = word(at);
                if i.has_delay_slot() && !i.is_call() && !i.ends_block() && i.target(at) == Some(t + 4) {
                    guards.insert(at);
                }
            }
        }
        for b in &mut blocks {
            let (br, at) = if b.end >= b.start + 8 && word(b.end - 8).has_delay_slot() {
                (word(b.end - 8), b.end - 8)
            } else {
                (word(b.end - 4), b.end - 4)
            };
            let fall = b.end;
            if traps.contains(&b.start) {
                b.succ.push(fall);
            } else if br.has_delay_slot() {
                match br.op {
                    Op::Jr if br.rs() == 31 => {}
                    Op::Jr => {
                        if let Some(jt) = p.jump_tables.get(&at) {
                            b.succ.extend(jt.targets.iter().copied());
                        }
                    }
                    Op::J => {
                        if let Some(t) = br.target(at)
                            && addrs.contains(&t)
                        {
                            b.succ.push(t);
                        }
                    }
                    Op::Jal | Op::Jalr => b.succ.push(fall),
                    _ if br.ends_block() => b.succ.push(br.target(at).unwrap()),
                    _ => {
                        b.succ.push(br.target(at).unwrap());
                        b.succ.push(fall);
                    }
                }
            } else {
                b.succ.push(fall);
            }
        }
        // The guards' own targets are labels only through other branches.
        for &g in &guards {
            let t = word(g).target(g).unwrap();
            let other = addrs.iter().any(|&a| {
                let i = word(a);
                !guards.contains(&a) && i.has_delay_slot() && !i.is_call() && i.target(a) == Some(t)
            });
            if !other {
                targets.remove(&t);
            }
        }
        let index = blocks.iter().enumerate().map(|(k, b)| (b.start, k)).collect();
        let mut f = Flow { addrs, blocks, index, leaders, targets, guards, traps, live_in: vec![] };
        f.live_in = vec![BTreeSet::new(); f.blocks.len()];
        loop {
            let mut changed = false;
            for k in (0..f.blocks.len()).rev() {
                let inn = f.live_through(p, k, arity, None);
                if inn != f.live_in[k] {
                    f.live_in[k] = inn;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Some(f)
    }

    fn live_out(&self, k: usize) -> BTreeSet<usize> {
        self.blocks[k]
            .succ
            .iter()
            .filter_map(|s| self.index.get(s))
            .flat_map(|&j| self.live_in[j].iter().copied())
            .collect()
    }

    /// The block's instructions in execution order, as (address, whether
    /// it is a branch whose delay slot follows).
    fn order(&self, p: &Program, k: usize) -> Vec<(u32, bool)> {
        let b = &self.blocks[k];
        let mut v = Vec::new();
        let mut pc = b.start;
        while pc < b.end {
            let branch = decode(p.mem.u32(pc).unwrap_or(0)).has_delay_slot() && pc + 4 < b.end;
            v.push((pc, branch));
            pc += if branch { 8 } else { 4 };
        }
        v
    }

    /// Liveness back through block `k`: its live-in, and if asked, what is
    /// live after each instruction. A branch reads its operands before its
    /// delay slot runs, and a call or return reads its arguments or results
    /// after it.
    fn live_through(
        &self,
        p: &Program,
        k: usize,
        arity: &Arity,
        mut after: Option<&mut BTreeMap<u32, BTreeSet<usize>>>,
    ) -> BTreeSet<usize> {
        let mut live = self.live_out(k);
        let mut note = |pc: u32, live: &BTreeSet<usize>| {
            if let Some(after) = after.as_deref_mut() {
                after.insert(pc, live.clone());
            }
        };
        for (pc, branch) in self.order(p, k).into_iter().rev() {
            note(pc, &live);
            let (early, late, defs) = self.uses_defs(p, pc, arity);
            for r in defs {
                live.remove(&r);
            }
            live.extend(late);
            if branch {
                note(pc + 4, &live);
                let (u, l, d) = self.uses_defs(p, pc + 4, arity);
                for r in d {
                    live.remove(&r);
                }
                live.extend(u);
                live.extend(l);
            }
            live.extend(early);
        }
        live
    }

    /// The registers the instruction at `pc` reads, before and after its
    /// delay slot, and writes.
    fn uses_defs(&self, p: &Program, pc: u32, arity: &Arity) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
        use Op::*;
        let i = decode(p.mem.u32(pc).unwrap_or(0));
        if self.guards.contains(&pc) {
            return (vec![], vec![], vec![]);
        }
        let nz = |v: Vec<usize>| v.into_iter().filter(|&r| r != 0).collect::<Vec<_>>();
        let args = |n: usize| (4..4 + n).collect::<Vec<_>>();
        let (rs, rt) = (i.rs(), i.rt());
        let mut defs: Vec<usize> = i.dest().into_iter().collect();
        let late = match i.op {
            Jr if rs == 31 => vec![2],
            Jr if p.jump_tables.contains_key(&pc) => vec![],
            Jr => args(4),
            Jal => args(arity.get(i.target(pc).unwrap())),
            Jalr | Bltzal | Bgezal => args(4),
            J => match i.target(pc) {
                Some(t) if !self.addrs.contains(&t) => args(arity.get(t)),
                _ => vec![],
            },
            _ => vec![],
        };
        if i.is_call() {
            defs.extend(CLOBBERED);
        }
        let early = match i.op {
            Jr if rs == 31 => vec![],
            Jr | Jalr | Bltzal | Bgezal => vec![rs],
            Jal | J => vec![],
            Sll | Srl | Sra => vec![rt],
            Sllv | Srlv | Srav | Add | Addu | Sub | Subu | And | Or | Xor | Nor | Slt | Sltu | Beq | Bne => {
                vec![rs, rt]
            }
            Mult | Multu | Div | Divu => {
                defs.extend([HI, LO]);
                vec![rs, rt]
            }
            Mfhi => vec![HI],
            Mflo => vec![LO],
            Mthi => {
                defs.push(HI);
                vec![rs]
            }
            Mtlo => {
                defs.push(LO);
                vec![rs]
            }
            Blez | Bgtz | Bltz | Bgez | Addi | Addiu | Slti | Sltiu | Andi | Ori | Xori | Lb | Lh | Lw | Lbu | Lhu
            | Lwc2 | Swc2 => vec![rs],
            Lwl | Lwr | Sb | Sh | Sw | Swl | Swr => vec![rs, rt],
            Mtc2 | Ctc2 | Mtc0 => vec![rt],
            _ => vec![],
        };
        (nz(early), nz(late), defs)
    }
}

/// Pseudo-code for the function starting at `start`.
pub fn decompile(p: &Program, start: u32, namer: &dyn Namer) -> Option<String> {
    let f = p.funcs.get(&start)?;
    let arity = Arity { p, cache: RefCell::new(BTreeMap::new()) };
    let flow = Flow::new(p, start, &arity)?;
    let ctx = Ctx { gp: p.gp, namer };
    let word = |a: u32| decode(p.mem.u32(a).unwrap_or(0));
    // The frame: `addiu sp, sp, -N` first in the function.
    let frame = (0..4)
        .map(|k| word(start + 4 * k))
        .find(|i| i.op == Op::Addiu && i.rs() == 29 && i.rt() == 29 && i.simm() < 0)
        .map_or(0, |i| -i.simm() as i64);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "// {}({}) {start:08x}..{:08x}, {} instructions, frame {frame}",
        namer.name(start).unwrap_or(format!("fn_{start:08x}")),
        (0..arity.get(start)).map(|k| REG[4 + k]).collect::<Vec<_>>().join(", "),
        f.end,
        f.insns
    );
    let mut spills: BTreeMap<i64, usize> = BTreeMap::new();
    for k in 0..flow.blocks.len() {
        if flow.traps.contains(&flow.blocks[k].start) {
            continue;
        }
        let mut after = BTreeMap::new();
        flow.live_through(p, k, &arity, Some(&mut after));
        let mut e = Emitter {
            ctx: &ctx,
            p,
            flow: &flow,
            arity: &arity,
            regs: BTreeMap::new(),
            out: &mut out,
            spills: &mut spills,
            after,
            uses: BTreeMap::new(),
            entry: k == 0,
            inflight: Vec::new(),
            marks: Vec::new(),
        };
        if k == 0 && frame != 0 {
            // At entry sp is the frame plus its size, so after the prologue's
            // adjustment sp-relative addresses are frame offsets.
            e.regs.insert(29, Bin("+", rc(Reg(29)), rc(Const(frame))));
        }
        e.block(k);
    }
    Some(out)
}

struct Emitter<'a, 'b> {
    ctx: &'a Ctx<'a>,
    p: &'a Program<'a>,
    flow: &'a Flow,
    arity: &'a Arity<'a>,
    regs: BTreeMap<usize, Expr>,
    out: &'b mut String,
    spills: &'b mut BTreeMap<i64, usize>,
    /// What is live after each instruction of the block.
    after: BTreeMap<u32, BTreeSet<usize>>,
    /// For each instruction that writes a register, how often the value is
    /// read before it is overwritten.
    uses: BTreeMap<(u32, usize), usize>,
    /// Whether this is the function's first block.
    entry: bool,
    /// Values computed but not yet stored anywhere, kept correct while
    /// registers are written out.
    inflight: Vec<Expr>,
    /// Marks standing for copies of a value being written out.
    marks: Vec<(Expr, Expr)>,
}

impl Emitter<'_, '_> {
    fn word(&self, a: u32) -> Insn {
        decode(self.p.mem.u32(a).unwrap_or(0))
    }

    fn get(&self, r: usize) -> Expr {
        if r == 0 { Const(0) } else { self.regs.get(&r).cloned().unwrap_or(Reg(r)) }
    }

    fn line(&mut self, s: String) {
        let _ = writeln!(self.out, "    {s}");
    }

    /// Writes register `r`'s pending value out. Other pending values that
    /// still read the old `r` are written first; copies of the value inside
    /// other pending or in-flight values come to read `r` instead; and an
    /// in-flight value that reads the old `r` keeps it in a temporary.
    fn materialize(&mut self, r: usize, busy: &mut Vec<usize>) {
        let Some(p) = self.regs.get(&r).cloned() else { return };
        if p == Reg(r) || is_frame(r, &p) {
            return;
        }
        // Copies of the value, marked until `r` holds it.
        let mark = Var(format!("\u{0}{r}"));
        let share = size(&p) > 2;
        if share {
            for (q, x) in self.regs.iter_mut() {
                if *q != r {
                    *x = replace(x, &p, &mark);
                }
            }
            for x in self.inflight.iter_mut() {
                *x = replace(x, &p, &mark);
            }
            self.marks.push((mark.clone(), p.clone()));
        }
        busy.push(r);
        let readers: Vec<usize> =
            self.regs.iter().filter(|(q, x)| **q != r && mentions(x, &[r])).map(|(q, _)| *q).collect();
        for q in readers {
            if busy.contains(&q) {
                // A cycle: the old value goes to a temporary.
                self.keep_old(r);
            } else {
                self.materialize(q, busy);
            }
        }
        busy.pop();
        if self.inflight.iter().any(|x| mentions(x, &[r])) {
            self.keep_old(r);
        }
        let e = self.regs[&r].clone();
        let text = format!("{} = {};", rname(r), self.show(&e));
        self.line(text);
        self.regs.insert(r, Reg(r));
        if share {
            self.marks.retain(|(m, _)| *m != mark);
            for x in self.regs.values_mut() {
                *x = replace(x, &mark, &Reg(r));
            }
            for x in self.inflight.iter_mut() {
                *x = replace(x, &mark, &Reg(r));
            }
        }
    }

    /// Saves `r` in `old_r` and makes every pending and in-flight read of
    /// `r` read that.
    fn keep_old(&mut self, r: usize) {
        let tmp = Var(format!("old_{}", rname(r)));
        self.line(format!("old_{} = {};", rname(r), rname(r)));
        for x in self.regs.values_mut().chain(self.inflight.iter_mut()) {
            *x = replace(x, &Reg(r), &tmp);
        }
    }

    /// A store's target as text, with marked copies spelled out.
    fn place(&self, addr: &Expr, w: u8) -> String {
        let mut a = addr.clone();
        for (m, p) in self.marks.iter().rev() {
            a = replace(&a, m, p);
        }
        self.ctx.place(&a, w, false)
    }

    /// An expression as text, with marked copies spelled out.
    fn show(&self, e: &Expr) -> String {
        let mut e = e.clone();
        for (m, p) in self.marks.iter().rev() {
            e = replace(&e, m, p);
        }
        self.ctx.expr(&e)
    }

    /// Writes out `regs` (those still pending) while `values` are in flight:
    /// built from the registers as they are, and kept correct as registers
    /// are written. Returns the values.
    fn materialize_with(&mut self, regs: &[usize], values: Vec<Expr>) -> Vec<Expr> {
        let base = self.inflight.len();
        self.inflight.extend(values);
        for &r in regs {
            self.materialize(r, &mut vec![]);
        }
        self.inflight.split_off(base)
    }

    fn set(&mut self, pc: u32, r: usize, e: Expr) {
        if r == 0 {
            return;
        }
        // Nothing pending may read the old value once it is replaced: what
        // is still needed is written out, the rest forgotten.
        let live = self.after.get(&pc).cloned().unwrap_or_default();
        let readers: Vec<usize> =
            self.regs.iter().filter(|(q, x)| **q != r && mentions(x, &[r])).map(|(q, _)| *q).collect();
        let (keep, drop): (Vec<usize>, Vec<usize>) = readers.into_iter().partition(|q| live.contains(q));
        for q in drop {
            self.regs.remove(&q);
        }
        let e = self.materialize_with(&keep, vec![e]).pop().unwrap();
        let e = simplify(e);
        let named = self.uses.get(&(pc, r)).is_some_and(|&n| n >= 2) && size(&e) >= 6;
        self.regs.insert(r, e);
        if named {
            self.materialize(r, &mut vec![]);
        }
    }

    /// Writes back registers (those `keep` says) whose value is an
    /// expression, and forgets them. The writes are one parallel assignment,
    /// so they are ordered to read each register before it is overwritten,
    /// with a temporary to break a cycle. `cond`, an expression in the values
    /// from before the writes, comes back rewritten in terms of after them.
    fn flush(&mut self, keep: impl Fn(usize) -> bool, cond: Option<Expr>) -> Option<Expr> {
        let regs = std::mem::take(&mut self.regs);
        let mut pending: Vec<(usize, Expr)> =
            regs.into_iter().filter(|(r, e)| *e != Reg(*r) && keep(*r) && !is_frame(*r, e)).collect();
        let cond = cond.map(|c| {
            let mut by_size = pending.clone();
            by_size.sort_by_key(|(_, e)| std::cmp::Reverse(size(e)));
            // Parts of the condition a write computes read as that register.
            let mut c2 = c.clone();
            for (r, e) in by_size.iter().filter(|(_, e)| size(e) > 1) {
                c2 = replace(&c2, e, &Var(rname(*r).to_string()));
            }
            let targets: Vec<usize> = pending.iter().map(|(r, _)| *r).collect();
            if mentions(&c2, &targets) {
                // It still reads a register the writes change: test it first.
                let text = format!("cond = {};", self.ctx.cond(&c));
                self.line(text);
                return Var("cond".into());
            }
            c2
        });
        while !pending.is_empty() {
            let free = (0..pending.len())
                .find(|&k| !pending.iter().enumerate().any(|(j, (_, e))| j != k && mentions(e, &[pending[k].0])));
            let Some(k) = free else {
                let r = pending[0].0;
                let tmp = Var(format!("old_{}", rname(r)));
                self.line(format!("old_{} = {};", rname(r), rname(r)));
                for (_, e) in pending.iter_mut().skip(1) {
                    *e = replace(e, &Reg(r), &tmp);
                }
                continue;
            };
            let (r, e) = pending.remove(k);
            let text = format!("{} = {};", rname(r), self.ctx.expr(&e));
            self.line(text);
        }
        cond
    }

    /// Forgets pending values nothing reads any more.
    fn prune(&mut self, live: &BTreeSet<usize>) {
        self.regs.retain(|r, e| live.contains(r) || *r == 29 || *e == Reg(*r));
    }

    fn block(&mut self, k: usize) {
        let b = &self.flow.blocks[k];
        let (start, end) = (b.start, b.end);
        if self.flow.targets.contains(&start) {
            let _ = writeln!(self.out, "L_{start:08x}:");
        }
        // How often each written value is read.
        let live_out = self.flow.live_out(k);
        let mut open: BTreeMap<usize, u32> = BTreeMap::new();
        let mut count = |open: &mut BTreeMap<usize, u32>, uses: &[usize], defs: &[(u32, usize)]| {
            for r in uses {
                if let Some(&at) = open.get(r) {
                    *self.uses.entry((at, *r)).or_default() += 1;
                }
            }
            for &(pc, r) in defs {
                open.insert(r, pc);
                self.uses.insert((pc, r), 0);
            }
        };
        for (pc, branch) in self.flow.order(self.p, k) {
            let (early, late, defs) = self.flow.uses_defs(self.p, pc, self.arity);
            count(&mut open, &early, &[]);
            if branch {
                let (u, l, d) = self.flow.uses_defs(self.p, pc + 4, self.arity);
                let d: Vec<_> = d.into_iter().map(|r| (pc + 4, r)).collect();
                count(&mut open, &[u, l].concat(), &d);
            }
            let defs: Vec<_> = defs.into_iter().map(|r| (pc, r)).collect();
            count(&mut open, &late, &defs);
        }
        for (r, at) in open {
            if live_out.contains(&r) {
                *self.uses.entry((at, r)).or_default() += 1;
            }
        }
        let mut pc = start;
        while pc < end {
            let i = self.word(pc);
            if i.has_delay_slot() {
                self.branch(&i, pc);
                pc += 8;
                continue;
            }
            self.step(&i, pc);
            if let Some(live) = self.after.get(&pc).cloned() {
                self.prune(&live);
            }
            pc += 4;
        }
        self.flush(|r| live_out.contains(&r), None);
    }

    fn call_args(&self, n: usize) -> String {
        (4..4 + n).map(|r| self.ctx.expr(&self.get(r))).collect::<Vec<_>>().join(", ")
    }

    fn branch(&mut self, i: &Insn, pc: u32) {
        let (s, t) = (self.get(i.rs()), self.get(i.rt()));
        let bin = |op: &'static str, a: &Expr, b: &Expr| Bin(op, rc(a.clone()), rc(b.clone()));
        let zero = Const(0);
        let cond = match i.op {
            Op::Beq if i.rs() == 0 && i.rt() == 0 => None,
            Op::Beq => Some(bin("==", &s, &t)),
            Op::Bne => Some(bin("!=", &s, &t)),
            Op::Blez => Some(bin("<=", &s, &zero)),
            Op::Bgtz => Some(bin(">", &s, &zero)),
            Op::Bltz | Op::Bltzal => Some(bin("<", &s, &zero)),
            Op::Bgez | Op::Bgezal => Some(bin(">=", &s, &zero)),
            _ => None,
        };
        let delay = self.word(pc + 4);
        self.step(&delay, pc + 4);
        // What is live once the branch is taken or not.
        let live_end = self.after.get(&pc).cloned().unwrap_or_default();
        let target = i.target(pc);
        if self.flow.guards.contains(&pc) {
            self.prune(&live_end);
            return;
        }
        match i.op {
            Op::Jal | Op::Jalr | Op::Bltzal | Op::Bgezal => {
                // Values still needed after the call are written out first:
                // the call may change memory and the registers it clobbers.
                let live: Vec<usize> = self
                    .regs
                    .iter()
                    .filter(|(r, e)| {
                        !CLOBBERED.contains(r) && live_end.contains(r) && (contains_load(e) || mentions(e, &CLOBBERED))
                    })
                    .map(|(r, _)| *r)
                    .collect();
                let mut v = self.materialize_with(&live, vec![s, cond.clone().unwrap_or(Const(0))]);
                let cond = cond.map(|_| v.pop().unwrap());
                let s = v.swap_remove(0);
                let (name, n, result) = match i.op {
                    Op::Jalr => (format!("(*{})", self.show(&s)), 4, format!("r_{pc:x}")),
                    _ => {
                        let a = target.unwrap_or(0);
                        let name = self.ctx.namer.name(a).unwrap_or(format!("fn_{a:08x}"));
                        let result = match self.ctx.namer.name(a) {
                            Some(n) if n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') => format!("r_{n}"),
                            _ => format!("r_{a:x}"),
                        };
                        (name, self.arity.get(a), result)
                    }
                };
                let args = self.call_args(n);
                let call = format!("{name}({args})");
                let used = live_end.contains(&2);
                match cond {
                    Some(ref c) => self.line(format!("if ({}) v0 = {call};", self.ctx.cond(c))),
                    None if used => self.line(format!("{result} = {call};")),
                    None => self.line(format!("{call};")),
                }
                for r in CLOBBERED {
                    self.regs.remove(&r);
                }
                if used && cond.is_none() {
                    self.regs.insert(2, Var(result));
                }
                self.prune(&live_end);
            }
            Op::Jr if i.rs() == 31 => {
                let v = self.get(2);
                self.regs.remove(&2);
                self.flush(|r| r == 3, None);
                let text = format!("return {};", self.ctx.expr(&v));
                self.line(text);
            }
            Op::Jr => {
                let s = self.flush(|r| live_end.contains(&r), Some(s)).unwrap();
                match self.p.jump_tables.get(&pc) {
                    Some(jt) => {
                        let text = format!("switch ({}) {{  // table 0x{:08x}", self.ctx.cond(&s), jt.table);
                        self.line(text);
                        for (k, t) in jt.targets.iter().enumerate() {
                            self.line(format!("    case {k}: goto L_{t:08x};"));
                        }
                        self.line("}".into());
                    }
                    None => {
                        let text = format!("goto *{};", self.ctx.expr(&s));
                        self.line(text);
                    }
                }
            }
            Op::J => {
                let t = target.unwrap_or(0);
                if self.flow.leaders.contains(&t) {
                    self.flush(|r| live_end.contains(&r), None);
                    self.line(format!("goto L_{t:08x};"));
                } else {
                    let name = self.ctx.namer.name(t).unwrap_or(format!("fn_{t:08x}"));
                    let args = self.call_args(self.arity.get(t));
                    self.flush(|_| false, None);
                    self.line(format!("return {name}({args}); // tail call"));
                }
            }
            _ => {
                let cond = self.flush(|r| live_end.contains(&r), cond);
                let t = target.unwrap_or(0);
                match cond {
                    Some(c) => {
                        let text = format!("if ({}) goto L_{t:08x};", self.ctx.cond(&c));
                        self.line(text)
                    }
                    None => self.line(format!("goto L_{t:08x};")),
                }
            }
        }
    }

    fn step(&mut self, i: &Insn, pc: u32) {
        let (s, t) = (self.get(i.rs()), self.get(i.rt()));
        let (rs, rt, rd) = (i.rs(), i.rt(), i.rd());
        let simm = Const(i.simm() as i64);
        let uimm = Const(i.imm() as i64);
        let sa = Const(i.sa() as i64);
        let bin = |op: &'static str, a: &Expr, b: &Expr| Bin(op, rc(a.clone()), rc(b.clone()));
        let addr = simplify(bin("+", &s, &simm));
        let slot = |a: &Expr| match a {
            Bin("+", x, c) if matches!(x.as_ref(), Reg(29)) => match c.as_ref() {
                Const(o) => Some(*o),
                _ => None,
            },
            _ => None,
        };
        match i.op {
            Op::Sll if i.word == 0 => {}
            Op::Sll => self.set(pc, rd, bin("<<", &t, &sa)),
            Op::Srl => self.set(pc, rd, bin("u>>", &t, &sa)),
            Op::Sra => self.set(pc, rd, bin(">>", &t, &sa)),
            Op::Sllv => self.set(pc, rd, bin("<<", &t, &s)),
            Op::Srlv => self.set(pc, rd, bin("u>>", &t, &s)),
            Op::Srav => self.set(pc, rd, bin(">>", &t, &s)),
            Op::Mfhi => {
                let h = self.get(HI);
                self.set(pc, rd, h)
            }
            Op::Mflo => {
                let l = self.get(LO);
                self.set(pc, rd, l)
            }
            Op::Mthi => self.set(pc, HI, s),
            Op::Mtlo => self.set(pc, LO, s),
            Op::Mult | Op::Multu => {
                let sg = i.op == Op::Mult;
                self.set(pc, LO, Lo(rc(s.clone()), rc(t.clone()), sg));
                self.set(pc, HI, Hi(rc(s), rc(t), sg));
            }
            Op::Div | Op::Divu => {
                let sg = i.op == Op::Div;
                self.set(pc, LO, Quot(rc(s.clone()), rc(t.clone()), sg));
                self.set(pc, HI, Rem(rc(s), rc(t), sg));
            }
            Op::Add | Op::Addu => self.set(pc, rd, bin("+", &s, &t)),
            Op::Sub | Op::Subu if rs == 0 => self.set(pc, rd, Neg(rc(t))),
            Op::Sub | Op::Subu => self.set(pc, rd, bin("-", &s, &t)),
            Op::And => self.set(pc, rd, bin("&", &s, &t)),
            Op::Or => self.set(pc, rd, bin("|", &s, &t)),
            Op::Xor => self.set(pc, rd, bin("^", &s, &t)),
            Op::Nor if rt == 0 => self.set(pc, rd, Not(rc(s))),
            Op::Nor => self.set(pc, rd, Not(rc(bin("|", &s, &t)))),
            Op::Slt => self.set(pc, rd, bin("<", &s, &t)),
            Op::Sltu => self.set(pc, rd, bin("u<", &s, &t)),
            Op::Addi | Op::Addiu => self.set(pc, rt, bin("+", &s, &simm)),
            Op::Slti => self.set(pc, rt, bin("<", &s, &simm)),
            Op::Sltiu => self.set(pc, rt, bin("u<", &s, &simm)),
            Op::Andi => self.set(pc, rt, bin("&", &s, &uimm)),
            Op::Ori => self.set(pc, rt, bin("|", &s, &uimm)),
            Op::Xori => self.set(pc, rt, bin("^", &s, &uimm)),
            Op::Lui => self.set(pc, rt, Const(((i.imm() as u32) << 16) as i32 as i64)),
            Op::Lb | Op::Lbu | Op::Lh | Op::Lhu | Op::Lw | Op::Lwl | Op::Lwr => {
                // An epilogue restoring a saved register from its slot.
                if let Some(o) = slot(&addr)
                    && self.spills.get(&o) == Some(&rt)
                {
                    self.regs.insert(rt, Reg(rt));
                    return;
                }
                let (w, sg) = match i.op {
                    Op::Lb => (1, true),
                    Op::Lbu => (1, false),
                    Op::Lh => (2, true),
                    Op::Lhu => (2, false),
                    _ => (4, true),
                };
                self.set(pc, rt, Load(w, sg, rc(addr)));
            }
            Op::Sb | Op::Sh | Op::Sw | Op::Swl | Op::Swr => {
                // A prologue saving a register it will restore: in the entry
                // block, the register still as it came in.
                if let (Some(o), Reg(r)) = (slot(&addr), &t)
                    && self.entry
                    && !self.regs.contains_key(r)
                    && *r == rt
                    && SAVED.contains(r)
                    && i.op == Op::Sw
                {
                    self.spills.insert(o, *r);
                    return;
                }
                let w = match i.op {
                    Op::Sb => 1,
                    Op::Sh => 2,
                    _ => 4,
                };
                // Values read from what the store may change, and still
                // needed, are written out first.
                let live = self.after.get(&pc).cloned().unwrap_or_default();
                let hit: Vec<usize> = self.regs.iter().filter(|(_, e)| aliases(e, &addr)).map(|(r, _)| *r).collect();
                let (keep, drop): (Vec<usize>, Vec<usize>) = hit.into_iter().partition(|r| live.contains(r));
                for r in drop {
                    self.regs.remove(&r);
                }
                let v = self.materialize_with(&keep, vec![addr, t]);
                let (addr, t) = (&v[0], &v[1]);
                let text = format!("{} = {};", self.place(addr, w), self.show(t));
                self.line(text);
            }
            Op::Mfc2 => self.set(pc, rt, Cop2(GTE_DATA[rd])),
            Op::Cfc2 => self.set(pc, rt, Cop2(GTE_CTRL[rd])),
            Op::Mtc2 => {
                let text = format!("gte.{} = {};", GTE_DATA[rd], self.ctx.expr(&t));
                self.line(text);
            }
            Op::Ctc2 => {
                let text = format!("gte.{} = {};", GTE_CTRL[rd], self.ctx.expr(&t));
                self.line(text);
            }
            Op::Lwc2 => {
                let text = format!("gte.{} = {};", GTE_DATA[rt], self.ctx.place(&addr, 4, false));
                self.line(text);
            }
            Op::Swc2 => {
                let text = format!("{} = gte.{};", self.ctx.place(&addr, 4, false), GTE_DATA[rt]);
                self.line(text);
            }
            Op::Gte(_) => {
                // GTE results read later must not be read from the new state.
                let live = self.after.get(&pc).cloned().unwrap_or_default();
                let hit: Vec<usize> =
                    self.regs.iter().filter(|(_, e)| contains_load(e) && mentions_cop2(e)).map(|(r, _)| *r).collect();
                for r in hit {
                    if live.contains(&r) {
                        self.materialize(r, &mut vec![]);
                    }
                }
                self.line(format!("gte_{};", i.render(pc, &|_| None).replace("  ", " ")))
            }
            _ => self.line(format!("/* {} */", i.render(pc, &|_| None))),
        }
    }
}

fn mentions_cop2(e: &Expr) -> bool {
    match e {
        Cop2(_) => true,
        Bin(_, a, b) | Lo(a, b, _) | Hi(a, b, _) | Quot(a, b, _) | Rem(a, b, _) | MulShr(a, b, _) => {
            mentions_cop2(a) || mentions_cop2(b)
        }
        Neg(a) | Not(a) | Load(_, _, a) => mentions_cop2(a),
        _ => false,
    }
}
