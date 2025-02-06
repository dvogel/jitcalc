// A simple integer calculator:
// `+` or `-` means add or subtract by 1
// `*` or `/` means multiply or divide by 2

use std::env::args;
use std::iter::Iterator;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "x86_64")]
use x86_64 as native_insns;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "aarch64")]
use aarch64 as native_insns;

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
    native_insns.extend(native_insns::native_insns(&Insn::Reset));
    for insn in insn_seq {
        native_insns.extend(native_insns::native_insns(insn));
    }
    native_insns.extend(native_insns::native_insns(&Insn::Return));
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
