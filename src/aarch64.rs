use super::Insn;

// Surely this could be done with transmute() or something similar...
fn arm_u32_insn_to_bytes(encoded: u32) -> Vec<u8> {
    vec![
        (encoded & 0xff) as u8,
        ((encoded & 0xff00) >> 8) as u8,
        ((encoded & 0xff0000) >> 16) as u8,
        ((encoded & 0xff000000) >> 24) as u8,
    ]
}

fn reset_accum() -> Vec<u8> {
    // MOVZ x0, 0x0000, LSL 0
    // 010100101--imm16	Rd(5)
    // -:-:-:-:-:-:-:-:-:-:-:imm16::::::::::::::::Rd::::
    arm_u32_insn_to_bytes(0x52800000)
}

fn func_return() -> Vec<u8> {
    // RET x30
    arm_u32_insn_to_bytes(0xD65F0000 | (30 << 5))
}

fn incr() -> Vec<u8> {
    // ADDS x0, x0, 0x01
    // -:-:-:-:-:-:-:-:-:-:imm12::::::::::::Rn:::::Rd::::
    arm_u32_insn_to_bytes(0xB1000000 | (0x01 << 10))
}

fn decr() -> Vec<u8> {
    // SUBS x0, x0, 0x01
    // -:-:-:-:-:-:-:-:-:-:imm12::::::::::::Rn:::::Rd::::
    arm_u32_insn_to_bytes(0xF1000000 | (0x01 << 10))
}

fn double() -> Vec<u8> {
    // MOVZ x1, 0x0002, LSL 0
    // -:-:-:-:-:-:-:-:-:-:-:imm16::::::::::::::::Rd::::
    let mut insns = arm_u32_insn_to_bytes(0x52800000 | (0x02 << 5) | 0x01);

    // MOVZ x2, 0x0000, LSL 0
    // -:-:-:-:-:-:-:-:-:-:-:imm16::::::::::::::::Rd::::
    insns.extend(arm_u32_insn_to_bytes(0x52800000 | 0x02));

    // MADD x0, x0, x1, x2
    // -:-:-:-:-:-:-:-:-:-:-:Rm:::::-:Ra:::::Rn:::::Rd::::
    insns.extend(arm_u32_insn_to_bytes(
        0x9B000000 | (0x01 << 16) | (0x02 << 10),
    ));

    insns
}

fn halve() -> Vec<u8> {
    // MOVZ x1, 0x0002, LSL 0
    // -:-:-:-:-:-:-:-:-:-:-:imm16::::::::::::::::Rd::::
    let mut insns = arm_u32_insn_to_bytes(0x52800000 | (0x02 << 5) | 0x01);

    // SDIV x0, x0, x1
    // -:-:-:-:-:-:-:-:-:-:-:Rm:::::-:-:-:-:-:-:Rn:::::Rd::::
    insns.extend(arm_u32_insn_to_bytes(0x9AC00C00 | (0x01 << 16)));

    insns
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
