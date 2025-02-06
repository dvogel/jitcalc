use super::Insn;

fn reset_accum() -> Vec<u8> {
    panic!("Not implemented yet");
}

fn func_return() -> Vec<u8> {
    panic!("Not implemented yet");
}

fn incr() -> Vec<u8> {
    panic!("Not implemented yet");
}

fn decr() -> Vec<u8> {
    panic!("Not implemented yet");
}

fn double() -> Vec<u8> {
    panic!("Not implemented yet");
}

fn halve() -> Vec<u8> {
    panic!("Not implemented yet");
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
