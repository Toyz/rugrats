//! The PsyQ library's BIOS call stubs, and names for what they call. rrt-dis
//! reads a stub as a tail call to 0xa0, 0xb0 or 0xc0; which function it asks
//! for (t1) is the console's, so it is named here.

use rrt::dis::analysis::Image;
use rrt::dis::{Endian, Operand, Reg};

use crate::MIPS;

/// A BIOS call stub, `li t2, TABLE; jr t2; li t1, FN`: (TABLE, FN).
pub fn stub(image: Image, addr: u32) -> Option<(u32, u32)> {
    let insn = |k: u32| {
        let at = addr.checked_add(4 * k)?;
        Some(MIPS.decode_word(image.word(at, Endian::Little)?, at))
    };
    // `li reg, imm` is `addiu reg, zero, imm`.
    let li = |k: u32, reg: u8| {
        let i = insn(k)?;
        match i.operands[..] {
            [Operand::Reg(r), Operand::Reg(z), Operand::Imm(v)]
                if i.mnemonic == "addiu" && r == Reg::gpr(reg) && z == Reg::gpr(0) =>
            {
                Some(v as u16 as u32)
            }
            _ => None,
        }
    };
    let jr = insn(1)?;
    if jr.mnemonic != "jr" || jr.operands[..] != [Operand::Reg(Reg::gpr(10))] {
        return None;
    }
    let table = li(0, 10)?;
    let function = li(2, 9)?;
    matches!(table, 0xa0 | 0xb0 | 0xc0).then_some((table, function))
}

/// Names for the BIOS functions the PsyQ library reaches through the A0, B0
/// and C0 tables.
pub fn name((table, function): (u32, u32)) -> Option<&'static str> {
    Some(match (table, function) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_stub_and_names_it() {
        // li t2, 0xa0; jr t2; li t1, 0x3f
        let words = [0x240a_00a0u32, 0x0140_0008, 0x2409_003f, 0];
        let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
        let image = Image { base: 0x8001_0000, bytes: &bytes };
        assert_eq!(stub(image, 0x8001_0000), Some((0xa0, 0x3f)));
        assert_eq!(stub(image, 0x8001_0000).and_then(name), Some("printf"));
        assert_eq!(stub(image, 0x8001_0004), None);
    }
}
