//! Whole-program analysis by recursive descent: functions with their real
//! extents, the calls between them, jump tables, and the code nothing seeded.
//!
//! Seeds come from [`crate::analysis`] (the entry, `jal` targets, BIOS stubs,
//! frames opened after a `jr ra`). Each function is walked from its start
//! along both arms of every branch until `jr ra`, a tail jump, or a jump
//! table. What remains unowned between functions is examined for code, and
//! words in the data that point at unowned code become seeds too; this repeats
//! until nothing new is found.

use std::collections::{BTreeMap, BTreeSet};

use crate::Memory;
use crate::analysis::{bios_stub, find_functions};
use crate::mips::{Insn, Op, decode};

#[derive(Clone, Debug, Default)]
pub struct Func {
    pub start: u32,
    /// One past the highest instruction the walk reached.
    pub end: u32,
    /// How the start was found: entry, jal or call (reached by a call from
    /// walked code), bios, and the heuristic ones: scan (a `jal` word
    /// anywhere), prologue, pointer, gap.
    pub how: &'static str,
    /// Instructions owned.
    pub insns: u32,
    /// Direct `jal` targets.
    pub calls: BTreeSet<u32>,
    /// `j` to another function's start.
    pub tail_calls: BTreeSet<u32>,
    /// Number of `jalr` sites (calls through a register).
    pub indirect_calls: u32,
    /// `jr` through a register that is not `ra` and not a resolved table.
    pub unresolved_jumps: u32,
    /// The walk hit an invalid instruction.
    pub hit_invalid: bool,
}

#[derive(Clone, Debug)]
pub struct JumpTable {
    /// The `jr` that dispatches through it.
    pub site: u32,
    pub table: u32,
    pub targets: Vec<u32>,
}

pub struct Program<'a> {
    pub mem: Memory<'a>,
    pub gp: Option<u32>,
    pub funcs: BTreeMap<u32, Func>,
    /// Instruction address -> owning function start.
    pub owner: BTreeMap<u32, u32>,
    pub jump_tables: BTreeMap<u32, JumpTable>,
    /// Callee -> call sites.
    pub callers: BTreeMap<u32, BTreeSet<u32>>,
    /// Where the code ends: one past the highest owned instruction.
    pub code_end: u32,
    /// Every instruction each function's walk reached.
    reach: BTreeMap<u32, Vec<u32>>,
    /// Fixed memory words the code stores function addresses into (interface
    /// tables filled at init): slot -> the functions stored there.
    pub slots: BTreeMap<u32, BTreeSet<u32>>,
    /// `jalr` sites whose register was loaded from a fixed slot: site -> slot.
    pub indirect: BTreeMap<u32, u32>,
}

fn opens_frame(i: &Insn) -> bool {
    i.op == Op::Addiu && i.rs() == 29 && i.rt() == 29 && i.simm() < 0
}

/// Plausible as the first instruction of a function found in a gap.
fn plausible_start(i: &Insn) -> bool {
    i.op != Op::Invalid && i.word != 0
}

impl<'a> Program<'a> {
    pub fn analyze(mem: Memory<'a>, entry: u32, gp: Option<u32>) -> Program<'a> {
        let seeds = find_functions(&mem, entry);
        let mut p = Program {
            mem,
            gp,
            funcs: BTreeMap::new(),
            owner: BTreeMap::new(),
            jump_tables: BTreeMap::new(),
            callers: BTreeMap::new(),
            code_end: mem.base,
            reach: BTreeMap::new(),
            slots: BTreeMap::new(),
            indirect: BTreeMap::new(),
        };
        // Call targets, BIOS stubs and the entry are certain. Walking them
        // first fixes where the code ends; the heuristic seeds are only
        // believed below that, which keeps data tables that happen to decode
        // out of the code.
        // Everything reachable by direct calls from the entry is certain.
        // Walking that first fixes where the code ends; the heuristic seeds
        // (including `jal` words the linear scan found, which may sit in data)
        // are only believed below that, which keeps data tables that happen
        // to decode out of the code.
        let mut reach = vec![entry];
        while let Some(start) = reach.pop() {
            if p.funcs.contains_key(&start) || !p.mem.contains(start) {
                continue;
            }
            let how = if start == entry {
                "entry"
            } else if bios_stub(&p.mem, start).is_some() {
                "bios"
            } else {
                "jal"
            };
            p.walk(start, how);
            let f = &p.funcs[&start];
            reach.extend(f.calls.iter().chain(f.tail_calls.iter()).copied());
        }
        p.code_end = p.funcs.values().map(|f| f.end).max().unwrap_or(p.mem.base);
        // A `jal` word the linear scan found but no walked code executes may
        // be data; it is a heuristic seed like the others.
        let mut pending: Vec<(u32, &'static str)> =
            seeds.starts.iter().map(|(&a, &h)| (a, if h == "jal" { "scan" } else { h })).collect();
        pending.sort_by_key(|&(a, h)| (h != "scan", a));
        let mut round = 0;
        while !pending.is_empty() && round < 32 {
            round += 1;
            for (start, how) in std::mem::take(&mut pending) {
                if p.funcs.contains_key(&start) {
                    continue;
                }
                // A heuristic start inside an already walked function is a
                // false positive (a frame opened mid-function, say).
                if how != "entry" && p.owner.contains_key(&start) {
                    continue;
                }
                if start >= p.code_end && how != "call" {
                    continue;
                }
                p.walk(start, how);
            }
            // Calls made by anything walked are as certain as the caller.
            let called: BTreeSet<u32> = p
                .funcs
                .values()
                .flat_map(|f| f.calls.iter().chain(f.tail_calls.iter()).copied())
                .filter(|t| p.mem.contains(*t) && !p.funcs.contains_key(t))
                .collect();
            pending = called.into_iter().map(|t| (t, "call")).collect();
            if pending.is_empty() {
                p.code_end = p.code_end.max(p.funcs.values().map(|f| f.end).max().unwrap_or(p.mem.base));
                pending = p.new_seeds();
            }
        }
        p.prune();
        p.resolve_slots();
        p
    }

    /// Drops heuristic starts that another function's walk reaches (case
    /// labels of a switch found before the switch, a frame opened
    /// mid-function), then rebuilds instruction ownership, certain functions
    /// first.
    fn prune(&mut self) {
        let weak = |h: &str| matches!(h, "pointer" | "gap" | "prologue" | "scan");
        let mut inside = BTreeSet::new();
        for (&start, addrs) in &self.reach {
            for a in addrs {
                if *a != start && self.funcs.get(a).is_some_and(|f| weak(f.how)) {
                    inside.insert(*a);
                }
            }
        }
        for a in &inside {
            self.funcs.remove(a);
            self.reach.remove(a);
        }
        self.owner.clear();
        let mut order: Vec<u32> = self.funcs.keys().copied().collect();
        order.sort_by_key(|a| (weak(self.funcs[a].how), *a));
        for start in order {
            for &a in &self.reach[&start] {
                self.owner.entry(a).or_insert(start);
            }
        }
    }

    fn is_start(&self, addr: u32) -> bool {
        self.funcs.contains_key(&addr)
    }

    fn walk(&mut self, start: u32, how: &'static str) {
        let mut f = Func { start, end: start, how, ..Func::default() };
        let mut seen: BTreeSet<u32> = BTreeSet::new();
        let mut work = vec![start];
        // A bios stub is three instructions and ends at its `jr`.
        let is_bios = bios_stub(&self.mem, start).is_some();
        while let Some(mut pc) = work.pop() {
            let mut sequential = false;
            loop {
                if seen.contains(&pc) || !self.mem.contains(pc) {
                    break;
                }
                // Heuristic functions stay inside the code the calls fixed.
                if how != "call" && self.code_end > self.mem.base && pc >= self.code_end {
                    f.hit_invalid = true;
                    break;
                }
                // Fell into the next function: no return was seen. Only a
                // certain start stops the walk; a heuristic one may be a
                // false start inside this function, which prune() removes.
                if sequential
                    && self.funcs.get(&pc).is_some_and(|f| !matches!(f.how, "pointer" | "gap" | "prologue" | "scan"))
                {
                    break;
                }
                sequential = true;
                let insn = decode(self.mem.u32(pc).unwrap());
                if insn.op == Op::Invalid {
                    f.hit_invalid = true;
                    break;
                }
                seen.insert(pc);
                let delay = pc + 4;
                let record_delay = |seen: &mut BTreeSet<u32>| {
                    seen.insert(delay);
                };
                match insn.op {
                    Op::Jal => {
                        let t = insn.target(pc).unwrap();
                        f.calls.insert(t);
                        self.callers.entry(t).or_default().insert(pc);
                        pc += 4;
                    }
                    Op::Jalr => {
                        f.indirect_calls += 1;
                        pc += 4;
                    }
                    Op::J => {
                        record_delay(&mut seen);
                        let t = insn.target(pc).unwrap();
                        if t != start && (self.is_start(t) || t < start) {
                            f.tail_calls.insert(t);
                            self.callers.entry(t).or_default().insert(pc);
                        } else {
                            work.push(t);
                        }
                        break;
                    }
                    Op::Jr => {
                        record_delay(&mut seen);
                        if insn.rs() != 31 && !is_bios {
                            match self.jump_table(pc, insn.rs()) {
                                Some(jt) => {
                                    work.extend(jt.targets.iter().copied());
                                    self.jump_tables.insert(pc, jt);
                                }
                                None => f.unresolved_jumps += 1,
                            }
                        }
                        break;
                    }
                    // The trap after a divide is always branched around.
                    Op::Break => break,
                    _ if insn.ends_block() => {
                        // `b` (beq zero, zero)
                        record_delay(&mut seen);
                        work.push(insn.target(pc).unwrap());
                        break;
                    }
                    _ if insn.has_delay_slot() => {
                        work.push(insn.target(pc).unwrap());
                        pc += 4;
                    }
                    _ => pc += 4,
                }
            }
        }
        f.insns = seen.len() as u32;
        f.end = seen.iter().next_back().map_or(start, |&a| a + 4);
        for &a in &seen {
            self.owner.entry(a).or_insert(start);
        }
        self.reach.insert(start, seen.into_iter().collect());
        self.funcs.insert(start, f);
    }

    /// The fixed address `reg` was last loaded from, before `site - k*4`:
    /// a `lw reg, off(base)` with a constant base or `$gp`.
    fn loaded_from(&self, site: u32, k: u32, reg: usize) -> Option<u32> {
        let mut k = k;
        while k < 16 {
            k += 1;
            let i = decode(self.mem.u32(site.checked_sub(k * 4)?)?);
            if i.dest() != Some(reg) {
                continue;
            }
            if i.op != Op::Lw {
                return None;
            }
            let base = if i.rs() == 28 { self.gp? } else { self.const_before(site, k, i.rs(), 0)? };
            return Some(base.wrapping_add(i.simm() as u32));
        }
        None
    }

    /// Finds the interface tables: stores of a function's address to a fixed
    /// word. Then names the callee of each `jalr` that loads its target from
    /// one, when exactly one function is ever stored there.
    fn resolve_slots(&mut self) {
        let mut slots: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
        for &pc in self.owner.keys() {
            let i = decode(self.mem.u32(pc).unwrap());
            if i.op != Op::Sw {
                continue;
            }
            let Some(value) = self.const_before(pc, 0, i.rt(), 0) else { continue };
            if !self.funcs.contains_key(&value) {
                continue;
            }
            let base = if i.rs() == 28 { self.gp } else { self.const_before(pc, 0, i.rs(), 0) };
            if let Some(base) = base {
                slots.entry(base.wrapping_add(i.simm() as u32)).or_default().insert(value);
            }
        }
        let mut indirect = BTreeMap::new();
        for (&start, addrs) in &self.reach {
            for &pc in addrs {
                let i = decode(self.mem.u32(pc).unwrap());
                if i.op != Op::Jalr {
                    continue;
                }
                // The delay slot runs before the jump but cannot change the
                // target register's value already latched.
                if let Some(slot) = self.loaded_from(pc, 0, i.rs()) {
                    indirect.insert(pc, slot);
                    if let Some(targets) = slots.get(&slot)
                        && targets.len() == 1
                    {
                        let t = *targets.iter().next().unwrap();
                        let _ = start;
                        self.callers.entry(t).or_default().insert(pc);
                    }
                }
            }
        }
        // Record resolved callees on the calling functions too.
        for (&site, slot) in &indirect {
            if let (Some(&owner), Some(targets)) = (self.owner.get(&site), slots.get(slot))
                && targets.len() == 1
                && let Some(f) = self.funcs.get_mut(&owner)
            {
                f.calls.insert(*targets.iter().next().unwrap());
            }
        }
        self.slots = slots;
        self.indirect = indirect;
    }

    /// The constant in `reg` on entry to the callee of the call at `site`
    /// (after its delay slot), when the few instructions before build one.
    pub fn arg_at_call(&self, site: u32, reg: usize) -> Option<u32> {
        self.const_before(site + 8, 0, reg, 0)
    }

    /// The constant a register holds just before `site - k*4`, if the last
    /// write to it within a few instructions builds one from `lui`,
    /// `addiu` and `ori`.
    fn const_before(&self, site: u32, mut k: u32, reg: usize, depth: u32) -> Option<u32> {
        if reg == 0 {
            return Some(0);
        }
        while k < 160 && depth < 6 {
            k += 1;
            let i = decode(self.mem.u32(site.checked_sub(k * 4)?)?);
            let writes = i.dest() == Some(reg);
            if !writes {
                continue;
            }
            return match i.op {
                Op::Lui => Some((i.imm() as u32) << 16),
                Op::Addiu => Some(self.const_before(site, k, i.rs(), depth + 1)?.wrapping_add(i.simm() as u32)),
                Op::Ori => Some(self.const_before(site, k, i.rs(), depth + 1)? | i.imm() as u32),
                _ => None,
            };
        }
        None
    }

    /// Recognises GCC's switch dispatch ending at `jr reg` at `site`: a
    /// `lw reg, off(b)` where `b` is the table address, possibly plus a
    /// scaled index added with `addu`, and an optional `sltiu` bound:
    ///
    /// ```text
    /// sltiu  t, idx, N              sltiu  t, idx, N
    /// sll    o, idx, 2              lui    b, %hi(table)
    /// lui    b, %hi(table)          addiu  b, b, %lo(table)
    /// addu   b, b, o                sll    o, idx, 2
    /// lw     reg, %lo(table)(b)     addu   o, o, b
    /// jr     reg                    lw     reg, 0(o)
    /// ```
    fn jump_table(&self, site: u32, reg: usize) -> Option<JumpTable> {
        let back = |k: u32| self.mem.u32(site.checked_sub(k * 4)?).map(decode);
        let (k, lw) = (1..=6).find_map(|k| {
            let i = back(k)?;
            (i.op == Op::Lw && i.rt() == reg).then_some((k, i))
        })?;
        let off = lw.simm() as u32;
        // The base register: a constant itself, or an addu of a constant and
        // an index.
        let base = lw.rs();
        let table = match self.const_before(site, k, base, 0) {
            Some(c) => c.wrapping_add(off),
            None => {
                let (ka, add) = (k + 1..k + 12).find_map(|ka| {
                    let i = back(ka)?;
                    (i.op == Op::Addu && i.rd() == base).then_some((ka, i))
                })?;
                let c =
                    self.const_before(site, ka, add.rs(), 0).or_else(|| self.const_before(site, ka, add.rt(), 0))?;
                c.wrapping_add(off)
            }
        };
        let bound = (k + 1..k + 24).find_map(|kb| {
            let i = back(kb)?;
            (i.op == Op::Sltiu).then_some(i.imm() as u32)
        });
        let limit = bound.unwrap_or(256).min(1024);
        let mut targets = Vec::new();
        for n in 0..limit {
            let Some(t) = self.mem.u32(table.wrapping_add(n * 4)) else { break };
            if t & 3 != 0 || !self.mem.contains(t) || t >= table {
                break;
            }
            targets.push(t);
        }
        (!targets.is_empty()).then_some(JumpTable { site, table, targets })
    }

    /// New seeds: the start of each unowned stretch of code below
    /// `code_end`, and data words that point at unowned code.
    fn new_seeds(&self) -> Vec<(u32, &'static str)> {
        let mut out = Vec::new();
        let mut pc = self.mem.base;
        while pc < self.code_end {
            if self.owner.contains_key(&pc) {
                pc += 4;
                continue;
            }
            // Skip zero padding between functions.
            let word = self.mem.u32(pc).unwrap();
            if word == 0 {
                pc += 4;
                continue;
            }
            let i = decode(word);
            if plausible_start(&i) && (opens_frame(&i) || self.owner.contains_key(&(pc - 4)) || pc == self.mem.base) {
                out.push((pc, "gap"));
            }
            // Move past this unowned stretch.
            while pc < self.code_end && !self.owner.contains_key(&pc) {
                pc += 4;
            }
        }
        let mut pc = self.code_end;
        while pc + 4 <= self.mem.end() {
            let w = self.mem.u32(pc).unwrap();
            if w & 3 == 0
                && w >= self.mem.base
                && w < self.code_end
                && !self.owner.contains_key(&w)
                && let Some(i) = self.mem.u32(w).map(decode)
                && plausible_start(&i)
            {
                out.push((w, "pointer"));
            }
            pc += 4;
        }
        out.sort();
        out.dedup_by_key(|s| s.0);
        out
    }

    /// The function that owns `addr`.
    pub fn func_of(&self, addr: u32) -> Option<&Func> {
        self.owner.get(&addr).and_then(|s| self.funcs.get(s))
    }

    /// Words of the image below `code_end` that no walk reached.
    pub fn unowned_words(&self) -> u32 {
        let mut n = 0;
        let mut pc = self.mem.base;
        while pc < self.code_end {
            if !self.owner.contains_key(&pc) && self.mem.u32(pc) != Some(0) {
                n += 1;
            }
            pc += 4;
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// main calls f; f dispatches a two-case switch through a jump table.
    #[test]
    fn walks_calls_and_switches() {
        let words: [u32; 26] = [
            0x0c00_4008, // 80010000 main: jal f
            0,           //          nop
            0x03e0_0008, //          jr ra
            0,
            0,
            0,
            0,
            0,
            0x2c82_0002, // 80010020 f: sltiu v0, a0, 2
            0x3c01_8001, //          lui at, 0x8001
            0x0004_1880, //          sll v1, a0, 2
            0x0023_0821, //          addu at, at, v1
            0x8c23_0060, //          lw v1, 0x60(at)
            0,
            0x0060_0008, // 80010038 jr v1
            0,
            0x03e0_0008, // 80010040 case 0: jr ra
            0x2402_0001, //          li v0, 1
            0x03e0_0008, // 80010048 case 1: jr ra
            0x2402_0002, //          li v0, 2
            0,
            0,
            0,
            0,
            0x8001_0040, // 80010060 the table
            0x8001_0048,
        ];
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        let mem = Memory { base: 0x8001_0000, bytes: &bytes };
        let p = Program::analyze(mem, 0x8001_0000, None);
        assert_eq!(p.funcs.keys().copied().collect::<Vec<_>>(), vec![0x8001_0000, 0x8001_0020]);
        let jt = &p.jump_tables[&0x8001_0038];
        assert_eq!(jt.table, 0x8001_0060);
        assert_eq!(jt.targets, vec![0x8001_0040, 0x8001_0048]);
        assert_eq!(p.funcs[&0x8001_0020].end, 0x8001_0050);
        assert_eq!(p.code_end, 0x8001_0050);
        assert_eq!(p.callers[&0x8001_0020].iter().copied().collect::<Vec<_>>(), vec![0x8001_0000]);
    }
}
