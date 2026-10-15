//! The R3000A instruction set (MIPS I) with the PlayStation's coprocessors:
//! COP0 for exceptions and the GTE as COP2.
//!
//! [`decode`] turns a word into an [`Insn`] whose fields are read off the
//! encoding on demand; [`Insn::render`] prints it the way the docs and the
//! worklog quote code: register names, pseudo-ops for the common idioms,
//! branch targets as absolute addresses.

use std::fmt::Write;

pub const REG: [&str; 32] = [
    "zero", "at", "v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2",
    "s3", "s4", "s5", "s6", "s7", "t8", "t9", "k0", "k1", "gp", "sp", "fp", "ra",
];

/// The GTE data registers, as COP2 registers 0-31.
pub const GTE_DATA: [&str; 32] = [
    "vxy0", "vz0", "vxy1", "vz1", "vxy2", "vz2", "rgbc", "otz", "ir0", "ir1", "ir2", "ir3", "sxy0", "sxy1", "sxy2",
    "sxyp", "sz0", "sz1", "sz2", "sz3", "rgb0", "rgb1", "rgb2", "res1", "mac0", "mac1", "mac2", "mac3", "irgb", "orgb",
    "lzcs", "lzcr",
];

/// The GTE control registers, as COP2 control registers 0-31.
pub const GTE_CTRL: [&str; 32] = [
    "r11r12", "r13r21", "r22r23", "r31r32", "r33", "trx", "try", "trz", "l11l12", "l13l21", "l22l23", "l31l32", "l33",
    "rbk", "gbk", "bbk", "lr1lr2", "lr3lg1", "lg2lg3", "lb1lb2", "lb3", "rfc", "gfc", "bfc", "ofx", "ofy", "h", "dqa",
    "dqb", "zsf3", "zsf4", "flag",
];

pub const COP0: [&str; 32] = [
    "c0r0", "c0r1", "c0r2", "bpc", "c0r4", "bda", "jumpdest", "dcic", "badvaddr", "bdam", "c0r10", "bpcm", "sr",
    "cause", "epc", "prid", "c0r16", "c0r17", "c0r18", "c0r19", "c0r20", "c0r21", "c0r22", "c0r23", "c0r24", "c0r25",
    "c0r26", "c0r27", "c0r28", "c0r29", "c0r30", "c0r31",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Op {
    // SPECIAL
    Sll,
    Srl,
    Sra,
    Sllv,
    Srlv,
    Srav,
    Jr,
    Jalr,
    Syscall,
    Break,
    Mfhi,
    Mthi,
    Mflo,
    Mtlo,
    Mult,
    Multu,
    Div,
    Divu,
    Add,
    Addu,
    Sub,
    Subu,
    And,
    Or,
    Xor,
    Nor,
    Slt,
    Sltu,
    // REGIMM
    Bltz,
    Bgez,
    Bltzal,
    Bgezal,
    // I- and J-type
    J,
    Jal,
    Beq,
    Bne,
    Blez,
    Bgtz,
    Addi,
    Addiu,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Lui,
    Lb,
    Lh,
    Lwl,
    Lw,
    Lbu,
    Lhu,
    Lwr,
    Sb,
    Sh,
    Swl,
    Sw,
    Swr,
    // COP0
    Mfc0,
    Mtc0,
    Rfe,
    // COP2 (GTE)
    Mfc2,
    Cfc2,
    Mtc2,
    Ctc2,
    Lwc2,
    Swc2,
    Gte(Gte),
    Invalid,
}

/// The GTE commands, by their function field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gte {
    Rtps,
    Nclip,
    Op,
    Dpcs,
    Intpl,
    Mvmva,
    Ncds,
    Cdp,
    Ncdt,
    Nccs,
    Cc,
    Ncs,
    Nct,
    Sqr,
    Dcpl,
    Dpct,
    Avsz3,
    Avsz4,
    Rtpt,
    Gpf,
    Gpl,
    Ncct,
    Unknown(u8),
}

impl Gte {
    fn from_funct(f: u32) -> Gte {
        match f {
            0x01 => Gte::Rtps,
            0x06 => Gte::Nclip,
            0x0c => Gte::Op,
            0x10 => Gte::Dpcs,
            0x11 => Gte::Intpl,
            0x12 => Gte::Mvmva,
            0x13 => Gte::Ncds,
            0x14 => Gte::Cdp,
            0x16 => Gte::Ncdt,
            0x1b => Gte::Nccs,
            0x1c => Gte::Cc,
            0x1e => Gte::Ncs,
            0x20 => Gte::Nct,
            0x28 => Gte::Sqr,
            0x29 => Gte::Dcpl,
            0x2a => Gte::Dpct,
            0x2d => Gte::Avsz3,
            0x2e => Gte::Avsz4,
            0x30 => Gte::Rtpt,
            0x3d => Gte::Gpf,
            0x3e => Gte::Gpl,
            0x3f => Gte::Ncct,
            other => Gte::Unknown(other as u8),
        }
    }

    pub fn name(self) -> String {
        match self {
            Gte::Unknown(f) => format!("gte_{f:02x}"),
            other => format!("{other:?}").to_lowercase(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Insn {
    pub word: u32,
    pub op: Op,
}

pub fn decode(word: u32) -> Insn {
    use Op::*;
    let opcode = word >> 26;
    let rs = (word >> 21) & 31;
    let rt = (word >> 16) & 31;
    let funct = word & 63;
    let op = match opcode {
        0 => match funct {
            0x00 => Sll,
            0x02 => Srl,
            0x03 => Sra,
            0x04 => Sllv,
            0x06 => Srlv,
            0x07 => Srav,
            0x08 => Jr,
            0x09 => Jalr,
            0x0c => Syscall,
            0x0d => Break,
            0x10 => Mfhi,
            0x11 => Mthi,
            0x12 => Mflo,
            0x13 => Mtlo,
            0x18 => Mult,
            0x19 => Multu,
            0x1a => Div,
            0x1b => Divu,
            0x20 => Add,
            0x21 => Addu,
            0x22 => Sub,
            0x23 => Subu,
            0x24 => And,
            0x25 => Or,
            0x26 => Xor,
            0x27 => Nor,
            0x2a => Slt,
            0x2b => Sltu,
            _ => Invalid,
        },
        // The R3000A decodes REGIMM on bit 16 (bgez vs bltz) and on bits
        // 17-20 all being 1000 (link); other rt values alias, but no compiler
        // emits them.
        1 => match rt {
            0x00 => Bltz,
            0x01 => Bgez,
            0x10 => Bltzal,
            0x11 => Bgezal,
            _ => Invalid,
        },
        0x02 => J,
        0x03 => Jal,
        0x04 => Beq,
        0x05 => Bne,
        0x06 => Blez,
        0x07 => Bgtz,
        0x08 => Addi,
        0x09 => Addiu,
        0x0a => Slti,
        0x0b => Sltiu,
        0x0c => Andi,
        0x0d => Ori,
        0x0e => Xori,
        0x0f => Lui,
        0x10 => match rs {
            0x00 => Mfc0,
            0x04 => Mtc0,
            0x10 if funct == 0x10 => Rfe,
            _ => Invalid,
        },
        0x12 => match rs {
            0x00 => Mfc2,
            0x02 => Cfc2,
            0x04 => Mtc2,
            0x06 => Ctc2,
            r if r & 0x10 != 0 => Gte(self::Gte::from_funct(funct)),
            _ => Invalid,
        },
        0x20 => Lb,
        0x21 => Lh,
        0x22 => Lwl,
        0x23 => Lw,
        0x24 => Lbu,
        0x25 => Lhu,
        0x26 => Lwr,
        0x28 => Sb,
        0x29 => Sh,
        0x2a => Swl,
        0x2b => Sw,
        0x2e => Swr,
        0x32 => Lwc2,
        0x3a => Swc2,
        _ => Invalid,
    };
    Insn { word, op }
}

impl Insn {
    pub fn rs(&self) -> usize {
        ((self.word >> 21) & 31) as usize
    }
    pub fn rt(&self) -> usize {
        ((self.word >> 16) & 31) as usize
    }
    pub fn rd(&self) -> usize {
        ((self.word >> 11) & 31) as usize
    }
    pub fn sa(&self) -> u32 {
        (self.word >> 6) & 31
    }
    pub fn imm(&self) -> u16 {
        self.word as u16
    }
    pub fn simm(&self) -> i32 {
        self.word as u16 as i16 as i32
    }
    pub fn target26(&self) -> u32 {
        self.word & 0x03ff_ffff
    }

    /// The branch or jump target when executed at `pc`, for the direct forms.
    pub fn target(&self, pc: u32) -> Option<u32> {
        use Op::*;
        match self.op {
            J | Jal => Some((pc.wrapping_add(4) & 0xf000_0000) | (self.target26() << 2)),
            Beq | Bne | Blez | Bgtz | Bltz | Bgez | Bltzal | Bgezal => {
                Some(pc.wrapping_add(4).wrapping_add((self.simm() << 2) as u32))
            }
            _ => None,
        }
    }

    /// Instructions with a delay slot.
    pub fn has_delay_slot(&self) -> bool {
        use Op::*;
        matches!(self.op, J | Jal | Jr | Jalr | Beq | Bne | Blez | Bgtz | Bltz | Bgez | Bltzal | Bgezal)
    }

    pub fn is_call(&self) -> bool {
        matches!(self.op, Op::Jal | Op::Jalr | Op::Bltzal | Op::Bgezal)
    }

    /// Control does not fall through to the next instruction after the delay slot.
    pub fn ends_block(&self) -> bool {
        use Op::*;
        match self.op {
            J | Jr => true,
            Beq => self.rs() == 0 && self.rt() == 0,
            _ => false,
        }
    }

    /// The general register the instruction writes, if any (never `zero`).
    pub fn dest(&self) -> Option<usize> {
        use Op::*;
        let r = match self.op {
            Sll | Srl | Sra | Sllv | Srlv | Srav | Jalr | Mfhi | Mflo | Add | Addu | Sub | Subu | And | Or | Xor
            | Nor | Slt | Sltu => self.rd(),
            Addi | Addiu | Slti | Sltiu | Andi | Ori | Xori | Lui | Lb | Lh | Lwl | Lw | Lbu | Lhu | Lwr | Mfc0
            | Mfc2 | Cfc2 => self.rt(),
            Jal | Bltzal | Bgezal => 31,
            _ => return None,
        };
        (r != 0).then_some(r)
    }

    pub fn is_load_store(&self) -> bool {
        use Op::*;
        matches!(self.op, Lb | Lh | Lwl | Lw | Lbu | Lhu | Lwr | Sb | Sh | Swl | Sw | Swr | Lwc2 | Swc2)
    }

    pub fn mnemonic(&self) -> String {
        match self.op {
            Op::Gte(g) => g.name(),
            Op::Invalid => ".word".into(),
            op => format!("{op:?}").to_lowercase(),
        }
    }

    /// Renders the instruction at `pc`. `label` may name an absolute address
    /// (a branch target or a call); returning `None` prints it as hex.
    pub fn render(&self, pc: u32, label: &dyn Fn(u32) -> Option<String>) -> String {
        use Op::*;
        let r = |i: usize| REG[i];
        let addr = |a: u32| label(a).unwrap_or_else(|| format!("0x{a:08x}"));
        let (rs, rt, rd) = (self.rs(), self.rt(), self.rd());
        let mut s = String::new();
        let mut put = |m: &str, args: String| {
            if args.is_empty() {
                s.push_str(m);
            } else {
                let _ = write!(s, "{m:<8}{args}");
            }
        };
        match self.op {
            Sll if self.word == 0 => put("nop", String::new()),
            Sll | Srl | Sra => put(&self.mnemonic(), format!("{}, {}, {}", r(rd), r(rt), self.sa())),
            Sllv | Srlv | Srav => put(&self.mnemonic(), format!("{}, {}, {}", r(rd), r(rt), r(rs))),
            Jr => put("jr", r(rs).to_string()),
            Jalr if rd == 31 => put("jalr", r(rs).to_string()),
            Jalr => put("jalr", format!("{}, {}", r(rd), r(rs))),
            Syscall | Break => {
                let code = (self.word >> 6) & 0xfffff;
                put(&self.mnemonic(), if code == 0 { String::new() } else { format!("0x{code:x}") })
            }
            Mfhi | Mflo => put(&self.mnemonic(), r(rd).to_string()),
            Mthi | Mtlo => put(&self.mnemonic(), r(rs).to_string()),
            Mult | Multu | Div | Divu => put(&self.mnemonic(), format!("{}, {}", r(rs), r(rt))),
            Addu | Or if rt == 0 => put("move", format!("{}, {}", r(rd), r(rs))),
            Addu | Or if rs == 0 => put("move", format!("{}, {}", r(rd), r(rt))),
            Subu if rs == 0 => put("negu", format!("{}, {}", r(rd), r(rt))),
            Nor if rt == 0 => put("not", format!("{}, {}", r(rd), r(rs))),
            Add | Addu | Sub | Subu | And | Or | Xor | Nor | Slt | Sltu => {
                put(&self.mnemonic(), format!("{}, {}, {}", r(rd), r(rs), r(rt)))
            }
            Bltz | Bgez | Bltzal | Bgezal | Blez | Bgtz => {
                put(&self.mnemonic(), format!("{}, {}", r(rs), addr(self.target(pc).unwrap())))
            }
            J => put("j", addr(self.target(pc).unwrap())),
            Jal => put("jal", addr(self.target(pc).unwrap())),
            Beq if rs == 0 && rt == 0 => put("b", addr(self.target(pc).unwrap())),
            Beq | Bne if rt == 0 => put(
                if self.op == Beq { "beqz" } else { "bnez" },
                format!("{}, {}", r(rs), addr(self.target(pc).unwrap())),
            ),
            Beq | Bne => put(&self.mnemonic(), format!("{}, {}, {}", r(rs), r(rt), addr(self.target(pc).unwrap()))),
            Addiu | Addi if rs == 0 => put("li", format!("{}, {}", r(rt), self.simm())),
            Ori if rs == 0 => put("li", format!("{}, 0x{:x}", r(rt), self.imm())),
            Addi | Addiu | Slti | Sltiu => put(&self.mnemonic(), format!("{}, {}, {}", r(rt), r(rs), self.simm())),
            Andi | Ori | Xori => put(&self.mnemonic(), format!("{}, {}, 0x{:x}", r(rt), r(rs), self.imm())),
            Lui => put("lui", format!("{}, 0x{:x}", r(rt), self.imm())),
            Lb | Lh | Lwl | Lw | Lbu | Lhu | Lwr | Sb | Sh | Swl | Sw | Swr => {
                put(&self.mnemonic(), format!("{}, {}({})", r(rt), self.simm(), r(rs)))
            }
            Lwc2 | Swc2 => put(&self.mnemonic(), format!("{}, {}({})", GTE_DATA[rt], self.simm(), r(rs))),
            Mfc0 | Mtc0 => put(&self.mnemonic(), format!("{}, {}", r(rt), COP0[rd])),
            Rfe => put("rfe", String::new()),
            Mfc2 | Mtc2 => put(&self.mnemonic(), format!("{}, {}", r(rt), GTE_DATA[rd])),
            Cfc2 | Ctc2 => put(&self.mnemonic(), format!("{}, {}", r(rt), GTE_CTRL[rd])),
            Gte(g) => {
                let sf = (self.word >> 19) & 1;
                let lm = (self.word >> 10) & 1;
                let mut args = format!("sf={sf}, lm={lm}");
                if g == self::Gte::Mvmva {
                    let mx = (self.word >> 17) & 3;
                    let v = (self.word >> 15) & 3;
                    let cv = (self.word >> 13) & 3;
                    let _ = write!(args, ", mx={mx}, v={v}, cv={cv}");
                }
                put(&g.name(), args)
            }
            Invalid => put(".word", format!("0x{:08x}", self.word)),
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(word: u32, pc: u32) -> String {
        decode(word).render(pc, &|_| None).split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn common_forms() {
        assert_eq!(text(0x0000_0000, 0), "nop");
        assert_eq!(text(0x27bd_ffe8, 0), "addiu sp, sp, -24");
        assert_eq!(text(0xafbf_0010, 0), "sw ra, 16(sp)");
        assert_eq!(text(0x03e0_0008, 0), "jr ra");
        assert_eq!(text(0x3c02_8001, 0), "lui v0, 0x8001");
        assert_eq!(text(0x0080_1021, 0), "move v0, a0");
        assert_eq!(text(0x2402_0005, 0), "li v0, 5");
        // jal 0x80012340 from 0x80010000
        assert_eq!(text(0x0c00_48d0, 0x8001_0000), "jal 0x80012340");
        // beq zero, zero, -1 instructions: branch to itself
        assert_eq!(text(0x1000_ffff, 0x8001_0000), "b 0x80010000");
        assert_eq!(text(0x1440_0003, 0x8001_0000), "bnez v0, 0x80010010");
    }

    #[test]
    fn gte() {
        // RTPS is cop2 0x0180001
        assert_eq!(text(0x4a18_0001, 0), "rtps sf=1, lm=0");
        assert_eq!(text(0x4a28_0030, 0), "rtpt sf=1, lm=0");
        assert_eq!(text(0x4b40_0006, 0), "nclip sf=0, lm=0");
        assert_eq!(text(0x4888_4800, 0), "mtc2 t0, ir1");
        assert_eq!(text(0xc880_0000, 0), "lwc2 vxy0, 0(a0)");
    }

    #[test]
    fn cop0() {
        assert_eq!(text(0x4002_6000, 0), "mfc0 v0, sr");
        assert_eq!(text(0x4200_0010, 0), "rfe");
    }
}
