//! Static analysis over an executable image: where the functions are, which
//! ones are BIOS call stubs, and which strings the code points at.

use std::collections::{BTreeMap, BTreeSet};

use crate::Memory;
use crate::mips::{Op, decode};

/// A BIOS call stub: `li t2, TABLE; jr t2; li t1, FN` (or with the `li t1`
/// before the jump).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BiosStub {
    pub table: u32,
    pub function: u32,
}

/// Recognises the three-instruction BIOS trampolines the PsyQ library uses.
pub fn bios_stub(mem: &Memory, addr: u32) -> Option<BiosStub> {
    let w0 = mem.u32(addr)?;
    let w1 = mem.u32(addr + 4)?;
    let w2 = mem.u32(addr + 8)?;
    let li = |w: u32, reg: u32| {
        let i = decode(w);
        (i.op == Op::Addiu && i.rs() == 0 && i.rt() as u32 == reg).then_some(i.imm() as u32)
    };
    let jr_t2 = decode(w1).op == Op::Jr && decode(w1).rs() == 10;
    if !jr_t2 {
        return None;
    }
    let table = li(w0, 10)?;
    let function = li(w2, 9)?;
    matches!(table, 0xa0 | 0xb0 | 0xc0).then_some(BiosStub { table, function })
}

/// Names for the BIOS functions the PsyQ library reaches through the A0, B0
/// and C0 tables.
pub fn bios_name(stub: BiosStub) -> Option<&'static str> {
    Some(match (stub.table, stub.function) {
        (0xa0, 0x00) => "FileOpen",
        (0xa0, 0x13) => "SaveState",
        (0xa0, 0x14) => "RestoreState",
        (0xa0, 0x17) => "strcmp",
        (0xa0, 0x19) => "strcpy",
        (0xa0, 0x1b) => "strlen",
        (0xa0, 0x25) => "toupper",
        (0xa0, 0x2a) => "memcpy",
        (0xa0, 0x2b) => "memset",
        (0xa0, 0x2f) => "rand",
        (0xa0, 0x30) => "srand",
        (0xa0, 0x33) => "malloc",
        (0xa0, 0x34) => "free",
        (0xa0, 0x39) => "InitHeap",
        (0xa0, 0x3f) => "printf",
        (0xa0, 0x44) => "FlushCache",
        (0xa0, 0x51) => "LoadExec",
        (0xa0, 0x72) => "CdRemove",
        (0xa0, 0x96) => "AddCDROMDevice",
        (0xa0, 0x97) => "AddMemCardDevice",
        (0xa0, 0x99) => "AddDummyTtyDevice",
        (0xa0, 0x9f) => "SetMem",
        (0xa0, 0xa0) => "_boot",
        (0xb0, 0x00) => "SysMalloc",
        (0xb0, 0x07) => "DeliverEvent",
        (0xb0, 0x08) => "OpenEvent",
        (0xb0, 0x09) => "CloseEvent",
        (0xb0, 0x0a) => "WaitEvent",
        (0xb0, 0x0b) => "TestEvent",
        (0xb0, 0x0c) => "EnableEvent",
        (0xb0, 0x0d) => "DisableEvent",
        (0xb0, 0x0e) => "OpenTh",
        (0xb0, 0x0f) => "CloseTh",
        (0xb0, 0x10) => "ChangeTh",
        (0xb0, 0x12) => "InitPAD",
        (0xb0, 0x13) => "StartPAD",
        (0xb0, 0x14) => "StopPAD",
        (0xb0, 0x16) => "PAD_dr",
        (0xb0, 0x17) => "ReturnFromException",
        (0xb0, 0x18) => "ResetEntryInt",
        (0xb0, 0x19) => "HookEntryInt",
        (0xb0, 0x20) => "UnDeliverEvent",
        (0xb0, 0x32) => "open",
        (0xb0, 0x33) => "lseek",
        (0xb0, 0x34) => "read",
        (0xb0, 0x35) => "write",
        (0xb0, 0x36) => "close",
        (0xb0, 0x37) => "ioctl",
        (0xb0, 0x3f) => "puts",
        (0xb0, 0x42) => "firstfile",
        (0xb0, 0x43) => "nextfile",
        (0xb0, 0x44) => "rename",
        (0xb0, 0x45) => "erase",
        (0xb0, 0x47) => "AddDevice",
        (0xb0, 0x4a) => "InitCARD",
        (0xb0, 0x4b) => "StartCARD",
        (0xb0, 0x4c) => "StopCARD",
        (0xb0, 0x4e) => "_card_write",
        (0xb0, 0x4f) => "_card_read",
        (0xb0, 0x50) => "_new_card",
        (0xb0, 0x51) => "Krom2RawAdd",
        (0xb0, 0x54) => "_get_errno",
        (0xb0, 0x55) => "_get_error",
        (0xb0, 0x56) => "GetC0Table",
        (0xb0, 0x57) => "GetB0Table",
        (0xb0, 0x58) => "_card_chan",
        (0xb0, 0x5b) => "ChangeClearPad",
        (0xb0, 0x5c) => "_card_status",
        (0xb0, 0x5d) => "_card_wait",
        (0xc0, 0x00) => "EnqueueTimerAndVblankIrqs",
        (0xc0, 0x01) => "EnqueueSyscallHandler",
        (0xc0, 0x02) => "SysEnqIntRP",
        (0xc0, 0x03) => "SysDeqIntRP",
        (0xc0, 0x07) => "InstallExceptionHandlers",
        (0xc0, 0x08) => "SysInitMemory",
        (0xc0, 0x0a) => "ChangeClearRCnt",
        (0xc0, 0x12) => "InstallDevices",
        (0xc0, 0x1c) => "AdjustA0Table",
        _ => return None,
    })
}

#[derive(Clone, Debug, Default)]
pub struct Functions {
    /// Function start addresses, with how they were found.
    pub starts: BTreeMap<u32, &'static str>,
    /// Call sites: target -> callers.
    pub callers: BTreeMap<u32, BTreeSet<u32>>,
}

/// Finds function starts: `jal` targets, BIOS stubs, and the instruction
/// after a `jr ra` delay slot when it opens a stack frame.
pub fn find_functions(mem: &Memory, entry: u32) -> Functions {
    let mut f = Functions::default();
    f.starts.insert(entry, "entry");
    let mut pc = mem.base;
    while pc + 4 <= mem.end() {
        let insn = decode(mem.u32(pc).unwrap());
        if insn.op == Op::Jal {
            let t = insn.target(pc).unwrap();
            if mem.contains(t) {
                f.starts.entry(t).or_insert("jal");
                f.callers.entry(t).or_default().insert(pc);
            }
        }
        if bios_stub(mem, pc).is_some() {
            f.starts.insert(pc, "bios");
        }
        if insn.op == Op::Jr && insn.rs() == 31 {
            let next = pc + 8;
            if let Some(w) = mem.u32(next) {
                let i = decode(w);
                if i.op == Op::Addiu && i.rs() == 29 && i.rt() == 29 && i.simm() < 0 {
                    f.starts.entry(next).or_insert("prologue");
                }
            }
        }
        pc += 4;
    }
    // Words in the image that hold the address of a known start are function
    // pointers: jump tables, callbacks, method tables.
    let known: BTreeSet<u32> = f.starts.keys().copied().collect();
    let mut pc = mem.base;
    while pc + 4 <= mem.end() {
        let w = mem.u32(pc).unwrap();
        if w & 3 == 0 && mem.contains(w) && !known.contains(&w) {
            // Only count it when the target opens a stack frame.
            if let Some(i) = mem.u32(w).map(decode)
                && i.op == Op::Addiu
                && i.rs() == 29
                && i.rt() == 29
                && i.simm() < 0
            {
                f.starts.entry(w).or_insert("pointer");
            }
        }
        pc += 4;
    }
    f
}

/// Addresses formed by `lui rX, hi` followed (within a few instructions) by
/// `addiu rY, rX, lo` or a load/store off rX, and by `$gp`-relative loads,
/// stores and `addiu` when `gp` is known: the code's references to data.
/// `addu rX, rX, rI` between the two keeps the base, as an indexed access.
pub fn data_refs(mem: &Memory, gp: Option<u32>) -> BTreeMap<u32, BTreeSet<u32>> {
    let mut refs: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    let mut pc = mem.base;
    while pc + 4 <= mem.end() {
        let insn = decode(mem.u32(pc).unwrap());
        if let Some(gp) = gp
            && insn.rs() == 28
            && (insn.is_load_store() || insn.op == Op::Addiu)
        {
            refs.entry(gp.wrapping_add(insn.simm() as u32)).or_default().insert(pc);
        }
        if insn.op == Op::Lui {
            let reg = insn.rt();
            let hi = (insn.imm() as u32) << 16;
            for k in 1..=8 {
                let at = pc + k * 4;
                let Some(w) = mem.u32(at) else { break };
                let i = decode(w);
                let uses = i.rs() == reg;
                let target = match i.op {
                    Op::Addiu | Op::Ori if uses => {
                        let lo = if i.op == Op::Ori { i.imm() as u32 } else { i.simm() as u32 };
                        Some(hi.wrapping_add(lo))
                    }
                    _ if uses && i.is_load_store() => Some(hi.wrapping_add(i.simm() as u32)),
                    _ => None,
                };
                if let Some(t) = target {
                    refs.entry(t).or_default().insert(at);
                }
                // Stop when the register is overwritten or control leaves.
                let writes = match i.op {
                    Op::Lui | Op::Addiu | Op::Ori | Op::Andi | Op::Lw | Op::Lh | Op::Lhu | Op::Lb | Op::Lbu => {
                        i.rt() == reg
                    }
                    // Adding an index to the base keeps it a pointer into the
                    // same object.
                    Op::Addu if i.rd() == reg && (i.rs() == reg || i.rt() == reg) => false,
                    Op::Addu | Op::Or | Op::Subu | Op::And | Op::Sll | Op::Srl | Op::Sra => i.rd() == reg,
                    _ => false,
                };
                if writes || i.has_delay_slot() && k > 1 {
                    break;
                }
            }
        }
        pc += 4;
    }
    refs
}

/// Finds the `lui gp, hi; addiu gp, gp, lo` that crt0 runs, near the entry.
pub fn find_gp(mem: &Memory, entry: u32) -> Option<u32> {
    let mut hi = None;
    for k in 0..128 {
        let i = decode(mem.u32(entry + k * 4)?);
        if i.op == Op::Lui && i.rt() == 28 {
            hi = Some((i.imm() as u32) << 16);
        } else if let Some(h) = hi
            && i.op == Op::Addiu
            && i.rt() == 28
            && i.rs() == 28
        {
            return Some(h.wrapping_add(i.simm() as u32));
        }
    }
    None
}
