// A simple integer calculator:
// `+` or `-` means add or subtract by 1
// `*` or `/` means multiply or divide by 2

use std::env::args;
use std::iter::Iterator;

fn main() {
    // let mut accumulator = 0;
    let mut program_tokens: Vec<String> = vec![];
    for arg in args().skip(1) {
        program_tokens.push(arg);
    }

    let program: String = program_tokens.join(" ");
    println!("Program: {}", program);

    let insn_seq = parse(&program);
    println!("{:?}", insn_seq);

    let native_insn_seq = jit(&insn_seq);
    println!("{:?}", native_insn_seq);

    match exec(&native_insn_seq) {
        Ok(r) => println!("Result: {}", r),
        Err(e) => eprintln!("Error: {}", e),
    };
}

#[derive(PartialEq, Eq, Debug)]
enum Insn {
    // Used by compiler
    Reset,
    Return,
    // Used in program text
    Incr,
    Decr,
    Double,
    Halve,
}

fn parse(program: &str) -> Vec<Insn> {
    program
        .chars()
        .map(|ch| match ch {
            '+' => Some(Insn::Incr),
            '-' => Some(Insn::Decr),
            '*' => Some(Insn::Double),
            '/' => Some(Insn::Halve),
            _ => None,
        })
        .filter_map(|insn_opt| insn_opt)
        .collect()
}

fn exec(insn_seq: &Vec<u8>) -> Result<i64, mmap_rs::Error> {
    use mmap_rs::MmapOptions;

    let mut code_mem = MmapOptions::new(insn_seq.len())?.map_mut()?;

    unsafe {
        std::ptr::copy(insn_seq.as_ptr(), code_mem.as_mut_ptr(), insn_seq.len());
    }

    let code_mem1 = match code_mem.make_read_only() {
        Ok(m) => m,
        Err((_, e)) => return Err(e),
    };

    let code_mem2 = match code_mem1.make_exec() {
        Ok(m) => m,
        Err((_, e)) => return Err(e),
    };

    let code_ptr = code_mem2.as_ptr() as *const ();
    let code_func: extern "C" fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
    let result = (code_func)();

    return Ok(result);
}

mod x86_64 {
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
}

fn interpret(insn_seq: &Vec<Insn>) -> u64 {
    let mut accum = 0;
    for insn in insn_seq {
        match insn {
            Insn::Reset => {
                accum = 0;
            }
            Insn::Return => {
                break;
            }
            Insn::Incr => {
                accum += 1;
            }
            Insn::Decr => {
                accum -= 1;
            }
            Insn::Double => {
                accum *= 2;
            }
            Insn::Halve => {
                accum /= 2;
            }
        }
    }
    accum
}

fn jit(insn_seq: &Vec<Insn>) -> Vec<u8> {
    let mut native_insns = Vec::new();
    native_insns.extend(x86_64::native_insns(&Insn::Reset));
    for insn in insn_seq {
        native_insns.extend(x86_64::native_insns(insn));
    }
    native_insns.extend(x86_64::native_insns(&Insn::Return));
    native_insns
}

#[cfg(test)]
mod tests {
    use super::{exec, interpret, jit, parse, Insn};

    #[test]
    fn test_canonical() {
        let program = "+ + * - /";
        let instructions = parse(program);
        assert_eq!(5, instructions.len());
        assert_eq!(Insn::Incr, instructions[0]);
        assert_eq!(Insn::Incr, instructions[1]);
        assert_eq!(Insn::Double, instructions[2]);
        assert_eq!(Insn::Decr, instructions[3]);
        assert_eq!(Insn::Halve, instructions[4]);

        let int_result = interpret(&instructions);
        assert_eq!(1, int_result);
    }

    #[test]
    fn test_exec_empty() {
        let result = exec(&jit(&Vec::new())).expect("mmap failure.");
        assert_eq!(0, result);
    }

    #[test]
    fn test_exec_incr_one() {
        let result = exec(&jit(&vec![Insn::Incr])).expect("mmap failure.");
        assert_eq!(1, result);
    }

    #[test]
    fn test_exec_decr_one() {
        let result = exec(&jit(&vec![Insn::Decr])).expect("mmap failure.");
        assert_eq!(-1, result);
    }

    #[test]
    fn test_exec_incr_double_double() {
        let result =
            exec(&jit(&vec![Insn::Incr, Insn::Double, Insn::Double])).expect("mmap failure.");
        assert_eq!(4, result);
    }

    #[test]
    fn test_exec_decr_double_double() {
        let result =
            exec(&jit(&vec![Insn::Decr, Insn::Double, Insn::Double])).expect("mmap failure.");
        assert_eq!(-4, result);
    }

    #[test]
    fn test_exec_simple() {
        let result = exec(&jit(&vec![
            Insn::Incr,   // 1
            Insn::Double, // 2
            Insn::Double, // 4
            Insn::Double, // 8
            Insn::Decr,   // 7
            Insn::Decr,   // 6
            Insn::Halve,  // 3
        ]))
        .expect("mmap failure.");

        assert_eq!(3, result);
    }
}
