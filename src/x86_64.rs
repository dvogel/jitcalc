use super::Insn;

// All instructions assume the value to operate on is in EAX and the result should be stored in
// EAX as well. This aligns with the x86 conventions.

// Format of the REX prefix byte, from https://pyokagan.name/blog/2019-09-20-x86encoding/
// 0100	4 bits	Fixed bit pattern
// W	1 bit	When 1, a 64-bit operand size is used. Otherwise, when 0, the default operand size is used (which is 32-bit for most but not all instructions)
// R	1 bit	This 1-bit value is an extension to the MODRM.reg field.
// X	1 bit	This 1-bit value is an extension to the SIB.index field.
// B	1 bit	This 1-bit value is an extension to the MODRM.rm field or the SIB.base field.

#[allow(dead_code)]
enum Rex {
    W,
    R,
    X,
    B,
}

fn rex(opts: &[Rex]) -> u8 {
    let mut rex: u8 = 0b01000000;
    for opt in opts {
        let shift = match opt {
            Rex::W => 3,
            Rex::R => 2,
            Rex::X => 1,
            Rex::B => 0,
        };
        rex = rex | 0x01 << shift;
    }
    rex
}

enum ModRM {
    Mod(u8),
    Reg(u8),
    RM(u8),
}

fn modrm(parts: &[ModRM]) -> u8 {
    // mod, reg, r/m
    // mm, rrr, bbb
    let mut modrm: u8 = 0x0;

    for part in parts {
        match part {
            ModRM::Mod(m) => {
                modrm = modrm | ((m & 0b00000011) << 6);
            }
            ModRM::Reg(r) => {
                modrm = modrm | ((r & 0b00000111) << 3);
            }
            ModRM::RM(b) => {
                modrm = modrm | (b & 0b00000111);
            }
        }
    }
    modrm
}

// From https://cs.wellesley.edu/~cs342/fall12/papers/isa.pdf
// A slash followed by a digit, such as /2, indicates that one of the operands to the instruction
// is a memory address or register (denoted mem or r/m, with an optional size). This is to be
// encoded as an effective address, with a ModR/M byte, an optional SIB byte, and an optional
// displacement, and the spare (register) field of the ModR/M byte should be the digit given
// (which will be from 0 to 7, so it fits in three bits). The encoding of effective addresses
// is given in section A.2.3.
//
// A.2.3
// An effective address is encoded in up to three parts: a ModR/M byte, an optional SIB byte,
// and an optional byte, word or doubleword displacement field.
//
// The ModR/M byte consists of three fields: the mod field, ranging from 0 to 3, in the upper
// two bits of the byte, the r/m field, ranging from 0 to 7, in the lower three bits, and the
// spare (register) field in the middle (bit 3 to bit 5). The spare field is not relevant to
// the effective address being encoded, and either contains an extension to the instruction
// opcode or the register value of another operand.

fn reset_accum() -> Vec<u8> {
    // XOR r/m64, r64
    // REX.W + 31 /r
    vec![
        rex(&[Rex::W]),
        0x31,
        modrm(&[
            ModRM::Mod(0x3),
            ModRM::Reg(0x00), // rax
        ]),
    ]
}

fn func_return() -> Vec<u8> {
    // RET
    vec![0xC3]
}

fn incr() -> Vec<u8> {
    vec![
        // ADD r/m64, imm8
        // REX.W + 83 /0 ib
        rex(&[Rex::W]),
        0x83,
        modrm(&[ModRM::Mod(0x3)]),
        0x01,
    ]
}

fn decr() -> Vec<u8> {
    // SUB r/m64, imm32
    // REX.W + 81 /5 id
    vec![
        rex(&[Rex::W]),
        0x81,
        modrm(&[ModRM::Mod(0x3), ModRM::Reg(0x05), ModRM::RM(0x00)]),
        0x01,
        0x00,
        0x00,
        0x00,
    ]
}

fn double() -> Vec<u8> {
    vec![
        // MOV rcx, DWORD 0x02
        // REX.W + C7 /0 io
        rex(&[Rex::W]),
        0xC7,
        modrm(&[ModRM::Mod(0x3), ModRM::RM(0x01)]),
        0x02,
        0x00,
        0x00,
        0x00,
        //
        // IMUL r/m64 -> REX.W + F7 /5
        // RDX:RAX ← RAX ∗ r/m64.
        // IMUL rcx
        rex(&[Rex::W]),
        0xF7,
        modrm(&[
            ModRM::Mod(0x3),  // register addressing
            ModRM::Reg(0x05), // literal,
            ModRM::RM(0x01),  // rcx
        ]),
    ]
}

fn halve() -> Vec<u8> {
    // Signed divide EDX:EAX by r/m32, with result stored in EAX ← Quotient, EDX ← Remainder.
    // IDIV r/m32 -> 0xF7 /7 -> ???
    // op1 -> ModRM:r/m (r)
    vec![
        // MOV rcx, DWORD 0x02
        // REX.W + C7 /0 io
        rex(&[Rex::W]),
        0xC7,
        modrm(&[ModRM::Mod(0x3), ModRM::RM(0x01)]),
        0x02,
        0x00,
        0x00,
        0x00,
        //
        // IDIV REX.W + F7 /7
        rex(&[Rex::W]),
        0xF7,
        modrm(&[
            ModRM::Mod(0x3),  // register addressing
            ModRM::Reg(0x07), // literal
            ModRM::RM(0x01),  // rcx
        ]),
    ]
}

pub fn native_insns(insn: &Insn) -> Vec<u8> {
    match insn {
        Insn::Reset => reset_accum(),
        Insn::Return => func_return(),
        Insn::Incr => incr(),
        Insn::Decr => decr(),
        Insn::Double => double(),
        Insn::Halve => halve(),
    }
}
