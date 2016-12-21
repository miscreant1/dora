use cpu::*;
use baseline::codegen::CondCode;
use masm::*;
use ty::MachineMode;

pub fn emit_orl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_alu_reg_reg(buf, 0, 0x09, src, dest);
}

pub fn emit_andl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_alu_reg_reg(buf, 0, 0x21, src, dest);
}

pub fn emit_xorl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_alu_reg_reg(buf, 0, 0x31, src, dest);
}

fn emit_alu_reg_reg(buf: &mut MacroAssembler, x64: u8, opcode: u8, src: Reg, dest: Reg) {
    if x64 != 0 || src.msb() != 0 || dest.msb() != 0 {
        emit_rex(buf, x64, src.msb(), 0, dest.msb());
    }

    emit_op(buf, opcode);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_movl_imm_reg(buf: &mut MacroAssembler, imm: i32, reg: Reg) {
    if reg.msb() != 0 {
        emit_rex(buf, 0, 0, 0, 1);
    }

    emit_op(buf, (0xB8 as u8) + reg.and7());
    emit_u32(buf, imm as u32);
}

// mov 32bit immediate and sign-extend into 64bit-register
pub fn emit_movq_imm_reg(buf: &mut MacroAssembler, imm: i32, reg: Reg) {
    emit_rex(buf, 1, 0, 0, reg.msb());
    emit_op(buf, 0xc7);
    emit_modrm(buf, 0b11, 0, reg.and7());
    emit_u32(buf, imm as u32);
}

pub fn emit_movb_memq_reg(buf: &mut MacroAssembler, src: Reg, disp: i32, dest: Reg) {
    let rex_prefix = if dest != RAX && dest != RBX && dest != RCX && dest != RDX { 1 } else { 0 };

    emit_mov_memq_reg(buf, rex_prefix, 0, 0x8a, src, disp, dest);
}

pub fn emit_movl_memq_reg(buf: &mut MacroAssembler, src: Reg, disp: i32, dest: Reg) {
    emit_mov_memq_reg(buf, 0, 0, 0x8b, src, disp, dest);
}

pub fn emit_movzbl_memq_reg(buf: &mut MacroAssembler, src: Reg, disp: i32, dest: Reg) {
    let src_msb = if src == RIP { 0 } else { src.msb() };

    if dest.msb() != 0 || src_msb != 0 {
        emit_rex(buf, 0, dest.msb(), 0, src_msb);
    }

    emit_op(buf, 0x0F);
    emit_op(buf, 0xB6);
    emit_membase(buf, src, disp, dest);
}

pub fn emit_movq_memq_reg(buf: &mut MacroAssembler, src: Reg, disp: i32, dest: Reg) {
    emit_mov_memq_reg(buf, 0, 1, 0x8b, src, disp, dest);
}

fn emit_mov_memq_reg(buf: &mut MacroAssembler, rex_prefix: u8, x64: u8,
        opcode: u8, src: Reg, disp: i32, dest: Reg) {
    let src_msb = if src == RIP { 0 } else { src.msb() };

    if src_msb != 0 || dest.msb() != 0 || x64 != 0 || rex_prefix != 0 {
        emit_rex(buf, x64, dest.msb(), 0, src_msb);
    }

    emit_op(buf, opcode);
    emit_membase(buf, src, disp, dest);
}

pub fn emit_movq_reg_memq(buf: &mut MacroAssembler, src: Reg, dest: Reg, disp: i32) {
    emit_mov_reg_memq(buf, 0x89, 1, src, dest, disp);
}

pub fn emit_movl_reg_memq(buf: &mut MacroAssembler, src: Reg, dest: Reg, disp: i32) {
    emit_mov_reg_memq(buf, 0x89, 0, src, dest, disp);
}

pub fn emit_movb_reg_memq(buf: &mut MacroAssembler, src: Reg, dest: Reg, disp: i32) {
    let dest_msb = if dest == RIP { 0 } else { dest.msb() };

    if dest_msb != 0
        || src.msb() != 0
        || (src != RAX && src != RBX && src != RCX && src != RDX) {
        emit_rex(buf, 0, src.msb(), 0, dest.msb());
    }

    emit_op(buf, 0x88);
    emit_membase(buf, dest, disp, src);
}

pub fn emit_movq_ar(buf: &mut MacroAssembler, base: Reg, index: Reg, scale: u8, dest: Reg) {
    emit_mov_ar(buf, 1, 0x8b, base, index, scale, dest);
}

pub fn emit_movl_ar(buf: &mut MacroAssembler, base: Reg, index: Reg, scale: u8, dest: Reg) {
    emit_mov_ar(buf, 0, 0x8b, base, index, scale, dest);
}

pub fn emit_movq_ra(buf: &mut MacroAssembler, src: Reg, base: Reg, index: Reg, scale: u8) {
    emit_mov_ar(buf, 1, 0x89, base, index, scale, src);
}

pub fn emit_movl_ra(buf: &mut MacroAssembler, src: Reg, base: Reg, index: Reg, scale: u8) {
    emit_mov_ar(buf, 0, 0x89, base, index, scale, src);
}

fn emit_mov_ar(buf: &mut MacroAssembler, x64: u8, opcode: u8, base: Reg, index: Reg,
               scale: u8, dest: Reg) {
    assert!(scale == 8 || scale == 4 || scale == 2 || scale == 1);

    if x64 != 0 || dest.msb() != 0 || index.msb() != 0 || base.msb() != 0 {
        emit_rex(buf, x64, dest.msb(), index.msb(), base.msb());
    }

    let scale = match scale {
        8 => 3,
        4 => 2,
        2 => 1,
        _ => 0,
    };

    emit_op(buf, opcode);
    emit_modrm(buf, 0b00, dest.and7(), 0b100);
    emit_sib(buf, scale, index.and7(), base.and7());
}

fn emit_mov_reg_memq(buf: &mut MacroAssembler, opcode: u8, x64: u8, src: Reg, dest: Reg, disp: i32) {
    let dest_msb = if dest == RIP { 0 } else { dest.msb() };

    if dest_msb != 0
        || src.msb() != 0
        || x64 != 0 {
        emit_rex(buf, x64, src.msb(), 0, dest_msb);
    }

    emit_op(buf, opcode);
    emit_membase(buf, dest, disp, src);
}

fn emit_membase(buf: &mut MacroAssembler, base: Reg, disp: i32, dest: Reg) {
    if base == RSP || base == R12 {
        if disp == 0 {
            emit_modrm(buf, 0, dest.and7(), RSP.and7());
            emit_sib(buf, 0, RSP.and7(), RSP.and7());
        } else if fits_i8(disp) {
            emit_modrm(buf, 1, dest.and7(), RSP.and7());
            emit_sib(buf, 0, RSP.and7(), RSP.and7());
            emit_u8(buf, disp as u8);
        } else {
            emit_modrm(buf, 2, dest.and7(), RSP.and7());
            emit_sib(buf, 0, RSP.and7(), RSP.and7());
            emit_u32(buf, disp as u32);
        }

    } else if disp == 0 && base != RBP && base != R13 && base != RIP {
        emit_modrm(buf, 0, dest.and7(), base.and7());

    } else if base == RIP {
        emit_modrm(buf, 0, dest.and7(), RBP.and7());
        emit_u32(buf, disp as u32);

    } else if fits_i8(disp) {
        emit_modrm(buf, 1, dest.and7(), base.and7());
        emit_u8(buf, disp as u8);

    } else {
        emit_modrm(buf, 2, dest.and7(), base.and7());
        emit_u32(buf, disp as u32);
    }
}

pub fn emit_subq_imm_reg(buf: &mut MacroAssembler, imm: i32, reg: Reg) {
    emit_aluq_imm_reg(buf, imm, reg, 0x2d, 0b101);
}

pub fn emit_addq_imm_reg(buf: &mut MacroAssembler, imm: i32, reg: Reg) {
    emit_aluq_imm_reg(buf, imm, reg, 0x05, 0);
}

fn emit_aluq_imm_reg(buf: &mut MacroAssembler, imm: i32, reg: Reg, rax_opcode: u8, modrm_reg: u8) {
    emit_rex(buf, 1, 0, 0, reg.msb());

    if fits_i8(imm) {
        emit_op(buf, 0x83);
        emit_modrm(buf, 0b11, modrm_reg, reg.and7());
        emit_u8(buf, imm as u8);
    } else if reg == RAX {
        emit_op(buf, rax_opcode);
        emit_u32(buf, imm as u32);
    } else {
        emit_op(buf, 0x81);
        emit_modrm(buf, 0b11, modrm_reg, reg.and7());
        emit_u32(buf, imm as u32);
    }
}

pub fn emit_movq_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_mov_reg_reg(buf, 1, src, dest);
}

pub fn emit_movl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_mov_reg_reg(buf, 0, src, dest);
}

fn emit_mov_reg_reg(buf: &mut MacroAssembler, x64: u8, src: Reg, dest: Reg) {
    if x64 != 0 || src.msb() != 0 || dest.msb() != 0 {
        emit_rex(buf, x64, src.msb(), 0, dest.msb());
    }

    emit_op(buf, 0x89);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_negl_reg(buf: &mut MacroAssembler, reg: Reg) {
    emit_alul_reg(buf, 0xf7, 0b11, 0, reg);
}

pub fn emit_notl_reg(buf: &mut MacroAssembler, reg: Reg) {
    emit_alul_reg(buf, 0xf7, 0b10, 0, reg);
}

fn emit_alul_reg(buf: &mut MacroAssembler, opcode: u8, modrm_reg: u8, x64: u8, reg: Reg) {
    if reg.msb() != 0 || x64 != 0 {
        emit_rex(buf, x64, 0, 0, reg.msb());
    }

    emit_op(buf, opcode);
    emit_modrm(buf, 0b11, modrm_reg, reg.and7());
}

pub fn emit_xorb_imm_reg(buf: &mut MacroAssembler, imm: u8, dest: Reg) {
    emit_alub_imm_reg(buf, 0x80, 0x34, 0b110, imm, dest);
}

pub fn emit_andb_imm_reg(buf: &mut MacroAssembler, imm: u8, dest: Reg) {
    emit_alub_imm_reg(buf, 0x80, 0x24, 0b100, imm, dest);
}

fn emit_alub_imm_reg(buf: &mut MacroAssembler, opcode: u8, rax_opcode: u8,
                     modrm_reg: u8, imm: u8, dest: Reg) {
    if dest == RAX {
        emit_op(buf, rax_opcode);
        emit_u8(buf, imm);
    } else {
        if dest.msb() != 0 || !dest.is_basic_reg() {
            emit_rex(buf, 0, 0, 0, dest.msb());
        }

        emit_op(buf, opcode);
        emit_modrm(buf, 0b11, modrm_reg, dest.and7());
        emit_u8(buf, imm);
    }
}

pub fn emit_sub_imm_mem(buf: &mut MacroAssembler, mode: MachineMode, base: Reg, imm: u8) {
    let (x64, opcode) = match mode {
        MachineMode::Ptr => (1, 0x83),
        MachineMode::Int32 => (0, 0x83),
        MachineMode::Int8 => (0, 0x80),
    };

    if x64 != 0 || base.msb() != 0 {
        emit_rex(buf, x64, 0, 0, base.msb());
    }

    emit_op(buf, opcode);
    emit_modrm(buf, 0b00, 0b101, base.and7());
    emit_u8(buf, imm);
}

pub fn emit_pushq_reg(buf: &mut MacroAssembler, reg: Reg) {
    if reg.msb() != 0 {
        emit_rex(buf, 0, 0, 0, 1);
    }

    emit_op(buf, 0x50 + reg.and7());
}

pub fn emit_popq_reg(buf: &mut MacroAssembler, reg: Reg) {
    if reg.msb() != 0 {
        emit_rex(buf, 0, 0, 0, 1);
    }

    emit_op(buf, 0x58 + reg.and7());
}

pub fn emit_retq(buf: &mut MacroAssembler) {
    emit_op(buf, 0xC3);
}

pub fn emit_nop(buf: &mut MacroAssembler) {
    emit_op(buf, 0x90);
}

pub fn emit_u32(buf: &mut MacroAssembler, val: u32) {
    buf.emit_u32(val)
}

pub fn emit_u8(buf: &mut MacroAssembler, val: u8) {
    buf.emit_u8(val)
}

pub fn emit_op(buf: &mut MacroAssembler, opcode: u8) {
    buf.emit_u8(opcode);
}

pub fn emit_rex(buf: &mut MacroAssembler, w: u8, r: u8, x: u8, b: u8) {
    assert!(w == 0 || w == 1);
    assert!(r == 0 || r == 1);
    assert!(x == 0 || x == 1);
    assert!(b == 0 || b == 1);

    buf.emit_u8(0x4 << 4 | w << 3 | r << 2 | x << 1 | b);
}

pub fn emit_modrm(buf: &mut MacroAssembler, mode: u8, reg: u8, rm: u8) {
    assert!(mode < 4);
    assert!(reg < 8);
    assert!(rm < 8);

    buf.emit_u8(mode << 6 | reg << 3 | rm);
}

pub fn emit_sib(buf: &mut MacroAssembler, scale: u8, index: u8, base: u8) {
    assert!(scale < 4);
    assert!(index < 8);
    assert!(base < 8);

    buf.emit_u8(scale << 6 | index << 3 | base);
}

pub fn fits_i8(imm: i32) -> bool {
    imm == (imm as i8) as i32
}

pub fn emit_jcc(buf: &mut MacroAssembler, cond: CondCode, lbl: Label) {
    let opcode = match cond {
        CondCode::Zero | CondCode::Equal => 0x84,
        CondCode::NonZero | CondCode::NotEqual => 0x85,
        CondCode::Greater => 0x8F,
        CondCode::GreaterEq => 0x8D,
        CondCode::Less => 0x8C,
        CondCode::LessEq => 0x8E,
        CondCode::UnsignedGreater => 0x87, // above
        CondCode::UnsignedGreaterEq => 0x83, // above or equal
        CondCode::UnsignedLess => 0x82, // below
        CondCode::UnsignedLessEq => 0x86, // below or equal
    };

    emit_op(buf, 0x0f);
    emit_op(buf, opcode);
    buf.emit_label(lbl);
}

pub fn emit_jmp(buf: &mut MacroAssembler, lbl: Label) {
    emit_op(buf, 0xe9);
    buf.emit_label(lbl);
}

pub fn emit_testl_reg_reg(buf: &mut MacroAssembler, op1: Reg, op2: Reg) {
    if op1.msb() != 0 || op2.msb() != 0 {
        emit_rex(buf, 0, op1.msb(), 0, op2.msb());
    }

    emit_op(buf, 0x85);
    emit_modrm(buf, 0b11, op1.and7(), op2.and7());
}

pub fn emit_testq_reg_reg(buf: &mut MacroAssembler, op1: Reg, op2: Reg) {
    emit_rex(buf, 1, op1.msb(), 0, op2.msb());

    emit_op(buf, 0x85);
    emit_modrm(buf, 0b11, op1.and7(), op2.and7());
}

pub fn emit_addl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    if src.msb() != 0 || dest.msb() != 0 {
        emit_rex(buf, 0, src.msb(), 0, dest.msb());
    }

    emit_op(buf, 0x01);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_addq_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_rex(buf, 1, src.msb(), 0, dest.msb());

    emit_op(buf, 0x01);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_subl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    if src.msb() != 0 || dest.msb() != 0 {
        emit_rex(buf, 0, src.msb(), 0, dest.msb());
    }

    emit_op(buf, 0x29);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_imull_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    if src.msb() != 0 || dest.msb() != 0 {
        emit_rex(buf, 0, dest.msb(), 0, src.msb());
    }

    emit_op(buf, 0x0f);
    emit_op(buf, 0xaf);
    emit_modrm(buf, 0b11, dest.and7(), src.and7());
}

pub fn emit_idivl_reg_reg(buf: &mut MacroAssembler, reg: Reg) {
    if reg.msb() != 0 {
        emit_rex(buf, 0, 0, 0, reg.msb());
    }

    emit_op(buf, 0xf7);
    emit_modrm(buf, 0b11, 0b111, reg.and7());
}

pub fn emit_cmpl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_alu_reg_reg(buf, 0, 0x39, src, dest);
}

pub fn emit_cmpq_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    emit_alu_reg_reg(buf, 1, 0x39, src, dest);
}

pub fn emit_cmp_mem_reg(buf: &mut MacroAssembler, mode: MachineMode,
                        base: Reg, disp: i32, dest: Reg) {
    let base_msb = if base == RIP { 0 } else { base.msb() };

    let (x64, opcode) = match mode {
        MachineMode::Int8 => (0, 0x38),
        MachineMode::Int32 => (0, 0x39),
        MachineMode::Ptr => (1, 0x39),
    };

    if x64 != 0 || dest.msb() != 0 || base_msb != 0 {
        emit_rex(buf, x64, dest.msb(), 0, base_msb);
    }

    emit_op(buf, opcode);
    emit_membase(buf, base, disp, dest);
}

pub fn emit_mov_memindex_reg(buf: &mut MacroAssembler, mode: MachineMode,
                             base: Reg, index: Reg, scale: i32, disp: i32,
                             dest: Reg) {
    assert!(scale == 8 || scale == 4 || scale == 2 || scale == 1);

    let (x64, opcode) = match mode {
        MachineMode::Int8 => (0, 0x8a),
        MachineMode::Int32 => (0, 0x8b),
        MachineMode::Ptr => (1, 0x8b),
    };

    if x64 != 0 || dest.msb() != 0 || index.msb() != 0 || base.msb() != 0 {
        emit_rex(buf, x64, dest.msb(), index.msb(), base.msb());
    }

    emit_op(buf, opcode);
    emit_membase_with_index_and_scale(buf, base, index, scale, disp, dest);
}

pub fn emit_mov_reg_memindex(buf: &mut MacroAssembler, mode: MachineMode, src: Reg,
                             base: Reg, index: Reg, scale: i32, disp: i32) {
    assert!(scale == 8 || scale == 4 || scale == 2 || scale == 1);

    let (x64, opcode) = match mode {
        MachineMode::Int8 => (0, 0x88),
        MachineMode::Int32 => (0, 0x89),
        MachineMode::Ptr => (1, 0x89),
    };

    if x64 != 0 || src.msb() != 0 || index.msb() != 0 || base.msb() != 0 {
        emit_rex(buf, x64, src.msb(), index.msb(), base.msb());
    }

    emit_op(buf, opcode);
    emit_membase_with_index_and_scale(buf, base, index, scale, disp, src);
}

pub fn emit_cmp_mem_imm(buf: &mut MacroAssembler, mode: MachineMode,
                        base: Reg, disp: i32, imm: i32) {
    let base_msb = if base == RIP { 0 } else { base.msb() };

    let opcode = if fits_i8(imm) {
        0x83
    } else {
        0x81
    };

    let (x64, opcode) = match mode {
        MachineMode::Int8 => (0, 0x80),
        MachineMode::Int32 => (0, opcode),
        MachineMode::Ptr => (1, opcode),
    };

    if x64 != 0 || base_msb != 0 {
        emit_rex(buf, x64, 0, 0, base_msb);
    }

    emit_op(buf, opcode);
    emit_membase(buf, base, disp, RDI);

    if fits_i8(imm) {
        emit_u8(buf, imm as u8);
    } else {
        if mode == MachineMode::Int8 {
            panic!("Int8 does not support 32 bit values");
        }

        emit_u32(buf, imm as u32);
    }
}

pub fn emit_cmp_memindex_reg(buf: &mut MacroAssembler, mode: MachineMode,
                             base: Reg, index: Reg, scale: i32, disp: i32,
                             dest: Reg) {
    assert!(scale == 8 || scale == 4 || scale == 2 || scale == 1);

    let (x64, opcode) = match mode {
        MachineMode::Int8 => (0, 0x38),
        MachineMode::Int32 => (0, 0x39),
        MachineMode::Ptr => (1, 0x39),
    };

    if x64 != 0 || dest.msb() != 0 || index.msb() != 0 || base.msb() != 0 {
        emit_rex(buf, x64, dest.msb(), index.msb(), base.msb());
    }

    emit_op(buf, opcode);
    emit_membase_with_index_and_scale(buf, base, index, scale, disp, dest);
}

fn emit_membase_with_index_and_scale(buf: &mut MacroAssembler,
                                     base: Reg, index: Reg, scale: i32, disp: i32,
                                     dest: Reg) {
    assert!(scale == 8 || scale == 4 || scale == 2 || scale == 1);

    let scale = match scale {
        8 => 3,
        4 => 2,
        2 => 1,
        _ => 0,
    };

    if disp == 0 {
        emit_modrm(buf, 0, dest.and7(), 4);
        emit_sib(buf, scale, index.and7(), base.and7());

    } else if fits_i8(disp) {
        emit_modrm(buf, 1, dest.and7(), 4);
        emit_sib(buf, scale, index.and7(), base.and7());
        emit_u8(buf, disp as u8);

    } else {
        emit_modrm(buf, 2, dest.and7(), 4);
        emit_sib(buf, scale, index.and7(), base.and7());
        emit_u32(buf, disp as u32);
    }
}

pub fn emit_cltd(buf: &mut MacroAssembler) {
    emit_op(buf, 0x99);
}

pub fn emit_setb_reg(buf: &mut MacroAssembler, op: CondCode, reg: Reg) {
    if reg.msb() != 0 || !reg.is_basic_reg() {
        emit_rex(buf, 0, 0, 0, reg.msb());
    }

    let op = match op {
        CondCode::Less => 0x9c,
        CondCode::LessEq => 0x9e,
        CondCode::Greater => 0x9f,
        CondCode::GreaterEq => 0x9d,
        CondCode::UnsignedLess => 0x9c,
        CondCode::UnsignedLessEq => 0x9e,
        CondCode::UnsignedGreater => 0x9f,
        CondCode::UnsignedGreaterEq => 0x9d,
        CondCode::Zero | CondCode::Equal => 0x94,
        CondCode::NonZero | CondCode::NotEqual => 0x95,
    };

    emit_op(buf, 0x0f);
    emit_op(buf, op);
    emit_modrm(buf, 0b11, 0, reg.and7());
}

pub fn emit_movb_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    if src.msb() != 0 || dest.msb() != 0 || !src.is_basic_reg() {
        emit_rex(buf, 0, dest.msb(), 0, src.msb());
    }

    emit_op(buf, 0x88);
    emit_modrm(buf, 0b11, src.and7(), dest.and7());
}

pub fn emit_movzbl_reg_reg(buf: &mut MacroAssembler, src: Reg, dest: Reg) {
    if src.msb() != 0 || dest.msb() != 0 || !src.is_basic_reg() {
        emit_rex(buf, 0, dest.msb(), 0, src.msb());
    }

    emit_op(buf, 0x0f);
    emit_op(buf, 0xb6);

    emit_modrm(buf, 0b11, dest.and7(), src.and7());
}

pub fn emit_cmpb_imm_reg(buf: &mut MacroAssembler, imm: u8, dest: Reg) {
    if dest == RAX {
        emit_op(buf, 0x3c);
        emit_u8(buf, imm);
        return;
    }

    if dest.msb() != 0 || !dest.is_basic_reg() {
        emit_rex(buf, 0, 0, 0, dest.msb());
    }

    emit_op(buf, 0x80);
    emit_modrm(buf, 0b11, 0b111, dest.and7());
    emit_u8(buf, imm);
}

pub fn emit_callq_reg(buf: &mut MacroAssembler, dest: Reg) {
    if dest.msb() != 0 {
        emit_rex(buf, 0, 0, 0, dest.msb());
    }

    emit_op(buf, 0xff);
    emit_modrm(buf, 0b11, 0b10, dest.and7());
}

pub fn emit_shlq_reg(buf: &mut MacroAssembler, imm: u8, dest: Reg) {
    emit_rex(buf, 1, 0, 0, dest.msb());
    emit_op(buf, 0xC1);
    emit_modrm(buf, 0b11, 0b100, dest.and7());
    emit_u8(buf, imm);
}

pub fn emit_shll_reg(buf: &mut MacroAssembler, imm: u8, dest: Reg) {
    if dest.msb() != 0 {
        emit_rex(buf, 0, 0, 0, dest.msb());
    }

    emit_op(buf, 0xC1);
    emit_modrm(buf, 0b11, 0b100, dest.and7());
    emit_u8(buf, imm);
}

pub fn emit_shll_reg_cl(buf: &mut MacroAssembler, dest: Reg) {
    if dest.msb() != 0 {
        emit_rex(buf, 0, 0, 0, dest.msb());
    }

    emit_op(buf, 0xD3);
    emit_modrm(buf, 0b11, 0b100, dest.and7());
}

#[cfg(test)]
mod tests {
    use super::*;

    use cpu::*;
    use baseline::codegen::CondCode;
    use masm::MacroAssembler;
    use ty::MachineMode;

    macro_rules! assert_emit {
        (
            $($expr:expr),*;
            $name:ident
        ) => {{
            let mut buf = MacroAssembler::new();
            $name(&mut buf);
            let expected = vec![$($expr,)*];

            assert_eq!(expected, buf.data());
        }};

        (
            $($expr:expr),*;
            $name:ident
            (
                    $($param:expr),+
            )
        ) => {{
            let mut buf = MacroAssembler::new();
            $name(&mut buf, $($param,)*);
            let expected = vec![$($expr,)*];
            let data = buf.data();

            if expected != data {
                print!("exp: ");

                for (ind, val) in expected.iter().enumerate() {
                    if ind > 0 { print!(", "); }

                    print!("{:02x}", val);
                }

                print!("\ngot: ");

                for (ind, val) in data.iter().enumerate() {
                    if ind > 0 { print!(", "); }

                    print!("{:02x}", val);
                }

                println!("");

                panic!("emitted code wrong.");
            }
        }};
    }

    #[test]
    fn test_fits8() {
        assert!(fits_i8(1));
        assert!(fits_i8(0));
        assert!(fits_i8(-1));
        assert!(fits_i8(127));
        assert!(fits_i8(-128));

        assert!(!fits_i8(128));
        assert!(!fits_i8(-129));
    }

    #[test]
    fn test_shlq_reg() {
        assert_emit!(0x48, 0xC1, 0xE0, 0x02; emit_shlq_reg(2, RAX));
        assert_emit!(0x49, 0xC1, 0xE4, 0x07; emit_shlq_reg(7, R12));
    }

    #[test]
    fn test_shll_reg() {
        assert_emit!(0xC1, 0xE0, 0x02; emit_shll_reg(2, RAX));
        assert_emit!(0x41, 0xC1, 0xE4, 0x07; emit_shll_reg(7, R12));
    }

    #[test]
    fn test_emit_retq() {
        assert_emit!(0xc3; emit_retq);
    }

    #[test]
    fn test_emit_popq_reg() {
        assert_emit!(0x58; emit_popq_reg(RAX));
        assert_emit!(0x5c; emit_popq_reg(RSP));
        assert_emit!(0x41, 0x58; emit_popq_reg(R8));
        assert_emit!(0x41, 0x5F; emit_popq_reg(R15));
    }

    #[test]
    fn test_emit_pushq_reg() {
        assert_emit!(0x50; emit_pushq_reg(RAX));
        assert_emit!(0x54; emit_pushq_reg(RSP));
        assert_emit!(0x41, 0x50; emit_pushq_reg(R8));
        assert_emit!(0x41, 0x57; emit_pushq_reg(R15));
    }

    #[test]
    fn test_emit_movq_reg_reg() {
        assert_emit!(0x4c, 0x89, 0xf8; emit_movq_reg_reg(R15, RAX));
        assert_emit!(0x49, 0x89, 0xc7; emit_movq_reg_reg(RAX, R15));
    }

    #[test]
    fn test_emit_movl_reg_reg() {
        assert_emit!(0x44, 0x89, 0xf8; emit_movl_reg_reg(R15, RAX));
        assert_emit!(0x41, 0x89, 0xc7; emit_movl_reg_reg(RAX, R15));
        assert_emit!(0x89, 0xc8; emit_movl_reg_reg(RCX, RAX));
    }

    #[test]
    fn test_emit_movl_imm_reg() {
        assert_emit!(0xb8, 2, 0, 0, 0; emit_movl_imm_reg(2, RAX));
        assert_emit!(0x41, 0xbe, 3, 0, 0, 0; emit_movl_imm_reg(3, R14));
    }

    #[test]
    fn test_emit_movq_imm_reg() {
        assert_emit!(0x48, 0xc7, 0xc0, 1, 0, 0, 0; emit_movq_imm_reg(1, RAX));
        assert_emit!(0x49, 0xc7, 0xc7, 0xFF, 0xFF, 0xFF, 0xFF; emit_movq_imm_reg(-1, R15));
    }

    #[test]
    fn test_emit_subq_imm_reg() {
        assert_emit!(0x48, 0x83, 0xe8, 0x11; emit_subq_imm_reg(0x11, RAX));
        assert_emit!(0x49, 0x83, 0xef, 0x11; emit_subq_imm_reg(0x11, R15));
        assert_emit!(0x48, 0x2d, 0x11, 0x22, 0, 0; emit_subq_imm_reg(0x2211, RAX));
        assert_emit!(0x48, 0x81, 0xe9, 0x11, 0x22, 0, 0; emit_subq_imm_reg(0x2211, RCX));
        assert_emit!(0x49, 0x81, 0xef, 0x11, 0x22, 0, 0; emit_subq_imm_reg(0x2211, R15));
    }

    #[test]
    fn test_emit_addq_imm_reg() {
        assert_emit!(0x48, 0x83, 0xc0, 0x11; emit_addq_imm_reg(0x11, RAX));
        assert_emit!(0x49, 0x83, 0xc7, 0x11; emit_addq_imm_reg(0x11, R15));
        assert_emit!(0x48, 0x05, 0x11, 0x22, 0, 0; emit_addq_imm_reg(0x2211, RAX));
        assert_emit!(0x48, 0x81, 0xc1, 0x11, 0x22, 0, 0; emit_addq_imm_reg(0x2211, RCX));
        assert_emit!(0x49, 0x81, 0xc7, 0x11, 0x22, 0, 0; emit_addq_imm_reg(0x2211, R15));
    }

    #[test]
    fn test_emit_testl_reg_reg() {
        assert_emit!(0x85, 0xc0; emit_testl_reg_reg(RAX, RAX));
        assert_emit!(0x85, 0xc6; emit_testl_reg_reg(RAX, RSI));
        assert_emit!(0x41, 0x85, 0xc7; emit_testl_reg_reg(RAX, R15));
    }

    #[test]
    fn test_emit_testq_reg_reg() {
        assert_emit!(0x48, 0x85, 0xc0; emit_testq_reg_reg(RAX, RAX));
        assert_emit!(0x48, 0x85, 0xc6; emit_testq_reg_reg(RAX, RSI));
        assert_emit!(0x49, 0x85, 0xc7; emit_testq_reg_reg(RAX, R15));
    }

    #[test]
    fn test_emit_jcc_zero() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::Zero, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x84, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_non_zero() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::NonZero, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x85, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_greater() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::Greater, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x8F, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_greater_or_equal() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::GreaterEq, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x8D, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_less() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::Less, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x8C, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_less_or_equal() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::LessEq, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x8E, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_unsigned_greater() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::UnsignedGreater, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x87, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_unsigned_greater_or_equal() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::UnsignedGreaterEq, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x83, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_unsigned_less() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::UnsignedLess, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x82, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jcc_unsigned_less_or_equal() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jcc(&mut buf, CondCode::UnsignedLessEq, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0x0f, 0x86, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_jmp() {
        let mut buf = MacroAssembler::new();
        let lbl = buf.create_label();
        emit_jmp(&mut buf, lbl);
        emit_nop(&mut buf);
        buf.bind_label(lbl);
        assert_eq!(vec![0xe9, 1, 0, 0, 0, 0x90], buf.data());
    }

    #[test]
    fn test_emit_movl_memq_reg() {
        assert_emit!(0x8b, 0x44, 0x24, 1; emit_movl_memq_reg(RSP, 1, RAX));
        assert_emit!(0x8b, 0x04, 0x24; emit_movl_memq_reg(RSP, 0, RAX));

        assert_emit!(0x44, 0x8b, 0x45, 0; emit_movl_memq_reg(RBP, 0, R8));

        assert_emit!(0x8b, 0x05, 0, 0, 0, 0; emit_movl_memq_reg(RIP, 0, RAX));
        assert_emit!(0x8b, 0x0d, 0, 0, 0, 0; emit_movl_memq_reg(RIP, 0, RCX));
    }

    #[test]
    fn test_emit_movzbl_memq_reg() {
        assert_emit!(0x0f, 0xb6, 0x45, 0; emit_movzbl_memq_reg(RBP, 0, RAX));
        assert_emit!(0x0F, 0xB6, 0x71, 0x11; emit_movzbl_memq_reg(RCX, 0x11, RSI));

        assert_emit!(0x0F, 0xB6, 0x3D, 0x00, 0x00, 0x00, 0x00;
            emit_movzbl_memq_reg(RIP, 0, RDI));

        assert_emit!(0x44, 0x0F, 0xB6, 0x94, 0x24, 0x22, 0x11, 0, 0;
            emit_movzbl_memq_reg(RSP, 0x1122, R10));
    }

    #[test]
    fn test_emit_movq_memq_reg() {
        assert_emit!(0x48, 0x8b, 0x44, 0x24, 1; emit_movq_memq_reg(RSP, 1, RAX));

        assert_emit!(0x48, 0x8b, 0x05, 0xff, 0xff, 0xff, 0xff; emit_movq_memq_reg(RIP, -1, RAX));
        assert_emit!(0x48, 0x8b, 0x05, 0, 0, 0, 0; emit_movq_memq_reg(RIP, 0, RAX));
        assert_emit!(0x48, 0x8b, 0x05, 1, 0, 0, 0; emit_movq_memq_reg(RIP, 1, RAX));
        assert_emit!(0x48, 0x8b, 0; emit_movq_memq_reg(RAX, 0, RAX));
    }

    #[test]
    fn test_emit_movb_memq_reg() {
        assert_emit!(0x8a, 0x45, 1; emit_movb_memq_reg(RBP, 1, RAX));
        assert_emit!(0x8a, 0x44, 0x24, 1; emit_movb_memq_reg(RSP, 1, RAX));
        assert_emit!(0x8a, 0x44, 0x24, 0xff; emit_movb_memq_reg(RSP, -1, RAX));
        assert_emit!(0x8a, 0x5d, 1; emit_movb_memq_reg(RBP, 1, RBX));
        assert_emit!(0x8a, 0x4d, 1; emit_movb_memq_reg(RBP, 1, RCX));
        assert_emit!(0x8a, 0x55, 1; emit_movb_memq_reg(RBP, 1, RDX));
        assert_emit!(0x44, 0x8a, 0x7d, 1; emit_movb_memq_reg(RBP, 1, R15));
        assert_emit!(0x40, 0x8a, 0x75, 1; emit_movb_memq_reg(RBP, 1, RSI));
        assert_emit!(0x40, 0x8a, 0x7d, 1; emit_movb_memq_reg(RBP, 1, RDI));
    }

    #[test]
    fn test_emit_movl_reg_memq() {
        assert_emit!(0x89, 0x0d, 0, 0, 0, 0; emit_movl_reg_memq(RCX, RIP, 0));
        assert_emit!(0x89, 0x48, 3; emit_movl_reg_memq(RCX, RAX, 3));
    }

    #[test]
    fn test_emit_movq_reg_memq() {
        assert_emit!(0x48, 0x89, 0x0d, 0, 0, 0, 0; emit_movq_reg_memq(RCX, RIP, 0));
        assert_emit!(0x48, 0x89, 0x48, 3; emit_movq_reg_memq(RCX, RAX, 3));
        assert_emit!(0x4c, 0x89, 0x42, 0x08; emit_movq_reg_memq(R8, RDX, 8));
        assert_emit!(0x4d, 0x89, 0x42, 0x08; emit_movq_reg_memq(R8, R10, 8));
        assert_emit!(0x49, 0x89, 0x42, 0x08; emit_movq_reg_memq(RAX, R10, 8));
        assert_emit!(0x48, 0x89, 0x42, 0x08; emit_movq_reg_memq(RAX, RDX, 8));
    }

    #[test]
    fn test_emit_movb_reg_memq() {
        assert_emit!(0x88, 0x0d, 0, 0, 0, 0; emit_movb_reg_memq(RCX, RIP, 0));
        assert_emit!(0x88, 0x48, 3; emit_movb_reg_memq(RCX, RAX, 3));
        assert_emit!(0x40, 0x88, 0x75, 0xFF; emit_movb_reg_memq(RSI, RBP, -1));

        assert_emit!(0x88, 0x45, 0x01; emit_movb_reg_memq(RAX, RBP, 1));
        assert_emit!(0x88, 0x44, 0x24, 0x01; emit_movb_reg_memq(RAX, RSP, 1));
        assert_emit!(0x88, 0x5d, 0x01; emit_movb_reg_memq(RBX, RBP, 1));
        assert_emit!(0x88, 0x4d, 0x01; emit_movb_reg_memq(RCX, RBP, 1));
        assert_emit!(0x88, 0x55, 0x01; emit_movb_reg_memq(RDX, RBP, 1));

        assert_emit!(0x44, 0x88, 0x42, 0x08; emit_movb_reg_memq(R8, RDX, 8));
        assert_emit!(0x45, 0x88, 0x42, 0x08; emit_movb_reg_memq(R8, R10, 8));
        assert_emit!(0x41, 0x88, 0x42, 0x08; emit_movb_reg_memq(RAX, R10, 8));
        assert_emit!(0x88, 0x42, 0x08; emit_movb_reg_memq(RAX, RDX, 8));
    }

    #[test]
    fn test_negl_reg() {
        assert_emit!(0xf7, 0xd8; emit_negl_reg(RAX));
        assert_emit!(0x41, 0xf7, 0xdf; emit_negl_reg(R15));
    }

    #[test]
    fn test_notl_reg() {
        assert_emit!(0xf7, 0xd0; emit_notl_reg(RAX));
        assert_emit!(0x41, 0xf7, 0xd7; emit_notl_reg(R15));
    }

    #[test]
    fn test_xorb_imm_reg() {
        assert_emit!(0x34, 1; emit_xorb_imm_reg(1, RAX));
        assert_emit!(0x80, 0xf3, 1; emit_xorb_imm_reg(1, RBX));
        assert_emit!(0x80, 0xf1, 1; emit_xorb_imm_reg(1, RCX));
        assert_emit!(0x80, 0xf2, 1; emit_xorb_imm_reg(1, RDX));
        assert_emit!(0x40, 0x80, 0xf6, 1; emit_xorb_imm_reg(1, RSI));
        assert_emit!(0x40, 0x80, 0xf7, 1; emit_xorb_imm_reg(1, RDI));

        assert_emit!(0x41, 0x80, 0xf0, 3; emit_xorb_imm_reg(3, R8));
        assert_emit!(0x41, 0x80, 0xf7, 4; emit_xorb_imm_reg(4, R15));
    }

    #[test]
    fn test_andb_imm_reg() {
        assert_emit!(0x24, 1; emit_andb_imm_reg(1, RAX));
        assert_emit!(0x80, 0xe1, 2; emit_andb_imm_reg(2, RCX));
        assert_emit!(0x41, 0x80, 0xe0, 3; emit_andb_imm_reg(3, R8));
        assert_emit!(0x41, 0x80, 0xe7, 4; emit_andb_imm_reg(4, R15));
    }

    #[test]
    fn test_addl_reg_reg() {
        assert_emit!(0x01, 0xd8; emit_addl_reg_reg(RBX, RAX));
        assert_emit!(0x44, 0x01, 0xf9; emit_addl_reg_reg(R15, RCX));
    }

    #[test]
    fn test_addq_reg_reg() {
        assert_emit!(0x48, 0x01, 0xD8; emit_addq_reg_reg(RBX, RAX));
        assert_emit!(0x4C, 0x01, 0xE0; emit_addq_reg_reg(R12, RAX));
        assert_emit!(0x49, 0x01, 0xC4; emit_addq_reg_reg(RAX, R12));
        assert_emit!(0x49, 0x01, 0xE7; emit_addq_reg_reg(RSP, R15));
    }

    #[test]
    fn test_subl_reg_reg() {
        assert_emit!(0x29, 0xd8; emit_subl_reg_reg(RBX, RAX));
        assert_emit!(0x44, 0x29, 0xf9; emit_subl_reg_reg(R15, RCX));
    }

    #[test]
    fn test_imull_reg_reg() {
        assert_emit!(0x0f, 0xaf, 0xc3; emit_imull_reg_reg(RBX, RAX));
        assert_emit!(0x41, 0x0f, 0xaf, 0xcf; emit_imull_reg_reg(R15, RCX));
    }

    #[test]
    fn test_idivl_reg_reg() {
        assert_emit!(0xf7, 0xf8; emit_idivl_reg_reg(RAX));
        assert_emit!(0x41, 0xf7, 0xff; emit_idivl_reg_reg(R15));
    }

    #[test]
    fn test_cltd() {
        assert_emit!(0x99; emit_cltd);
    }

    #[test]
    fn test_orl_reg_reg() {
        assert_emit!(0x44, 0x09, 0xf8; emit_orl_reg_reg(R15, RAX));
        assert_emit!(0x09, 0xc8; emit_orl_reg_reg(RCX, RAX));
        assert_emit!(0x41, 0x09, 0xc7; emit_orl_reg_reg(RAX, R15));
    }

    #[test]
    fn test_andl_reg_reg() {
        assert_emit!(0x44, 0x21, 0xf8; emit_andl_reg_reg(R15, RAX));
        assert_emit!(0x21, 0xc8; emit_andl_reg_reg(RCX, RAX));
        assert_emit!(0x41, 0x21, 0xc7; emit_andl_reg_reg(RAX, R15));
    }

    #[test]
    fn test_xorl_reg_reg() {
        assert_emit!(0x44, 0x31, 0xf8; emit_xorl_reg_reg(R15, RAX));
        assert_emit!(0x31, 0xc8; emit_xorl_reg_reg(RCX, RAX));
        assert_emit!(0x41, 0x31, 0xc7; emit_xorl_reg_reg(RAX, R15));
    }

    #[test]
    fn test_cmpl_reg_reg() {
        assert_emit!(0x44, 0x39, 0xf8; emit_cmpl_reg_reg(R15, RAX));
        assert_emit!(0x41, 0x39, 0xdf; emit_cmpl_reg_reg(RBX, R15));
        assert_emit!(0x39, 0xd8; emit_cmpl_reg_reg(RBX, RAX));
    }

    #[test]
    fn test_cmpq_reg_reg() {
        assert_emit!(0x4C, 0x39, 0xf8; emit_cmpq_reg_reg(R15, RAX));
        assert_emit!(0x49, 0x39, 0xdf; emit_cmpq_reg_reg(RBX, R15));
        assert_emit!(0x48, 0x39, 0xd8; emit_cmpq_reg_reg(RBX, RAX));
    }

    #[test]
    fn test_setb_reg() {
        assert_emit!(0x0f, 0x94, 0xc0; emit_setb_reg(CondCode::Equal, RAX));
        assert_emit!(0x41, 0x0f, 0x95, 0xc7; emit_setb_reg(CondCode::NotEqual, R15));
        assert_emit!(0x0f, 0x9d, 0xc1; emit_setb_reg(CondCode::GreaterEq, RCX));
        assert_emit!(0x0f, 0x9f, 0xc2; emit_setb_reg(CondCode::Greater, RDX));
        assert_emit!(0x40, 0x0f, 0x9e, 0xc6; emit_setb_reg(CondCode::LessEq, RSI));
        assert_emit!(0x40, 0x0f, 0x9c, 0xc7; emit_setb_reg(CondCode::Less, RDI));
    }

    #[test]
    fn test_movb_reg_reg() {
        assert_emit!(0x88, 0xd8; emit_movb_reg_reg(RBX, RAX));
        assert_emit!(0x88, 0xd1; emit_movb_reg_reg(RDX, RCX));
        assert_emit!(0x45, 0x88, 0xd1; emit_movb_reg_reg(R10, R9));
        assert_emit!(0x40, 0x88, 0xfe; emit_movb_reg_reg(RDI, RSI));
        assert_emit!(0x45, 0x88, 0xf7; emit_movb_reg_reg(R14, R15));
    }

    #[test]
    fn test_movzbl_reg_reg() {
        assert_emit!(0x0f, 0xb6, 0xc0; emit_movzbl_reg_reg(RAX, RAX));
        assert_emit!(0x41, 0x0f, 0xb6, 0xc7; emit_movzbl_reg_reg(R15, RAX));
        assert_emit!(0x44, 0x0f, 0xb6, 0xfb; emit_movzbl_reg_reg(RBX, R15));
        assert_emit!(0x40, 0x0f, 0xb6, 0xce; emit_movzbl_reg_reg(RSI, RCX));
    }

    #[test]
    fn test_emit_cmpb_imm_reg() {
        assert_emit!(0x3c, 0; emit_cmpb_imm_reg(0, RAX));
        assert_emit!(0x80, 0xf9, 0; emit_cmpb_imm_reg(0, RCX));
        assert_emit!(0x41, 0x80, 0xff, 0; emit_cmpb_imm_reg(0, R15));
        assert_emit!(0x40, 0x80, 0xfe, 0; emit_cmpb_imm_reg(0, RSI));
    }

    #[test]
    fn test_callq_reg() {
        assert_emit!(0xff, 0xd0; emit_callq_reg(RAX));
        assert_emit!(0x41, 0xff, 0xd7; emit_callq_reg(R15));
    }

    #[test]
    fn test_movq_ar() {
        assert_emit!(0x48, 0x8b, 0x0C, 0xD8; emit_movq_ar(RAX, RBX, 8, RCX));
        assert_emit!(0x48, 0x8b, 0x1C, 0x81; emit_movq_ar(RCX, RAX, 4, RBX));
        assert_emit!(0x48, 0x8b, 0x04, 0x4B; emit_movq_ar(RBX, RCX, 2, RAX));
        assert_emit!(0x4D, 0x8b, 0x3C, 0x03; emit_movq_ar(R11, RAX, 1, R15));
    }

    #[test]
    fn test_movl_ar() {
        assert_emit!(0x8b, 0x0c, 0xd8; emit_movl_ar(RAX, RBX, 8, RCX));
        assert_emit!(0x8b, 0x1C, 0x81; emit_movl_ar(RCX, RAX, 4, RBX));
        assert_emit!(0x8b, 0x04, 0x4B; emit_movl_ar(RBX, RCX, 2, RAX));
        assert_emit!(0x45, 0x8b, 0x3C, 0x03; emit_movl_ar(R11, RAX, 1, R15));
    }

    #[test]
    fn test_movq_ra() {
        assert_emit!(0x48, 0x89, 0x0C, 0xD8; emit_movq_ra(RCX, RAX, RBX, 8));
        assert_emit!(0x48, 0x89, 0x1C, 0x81; emit_movq_ra(RBX, RCX, RAX, 4));
        assert_emit!(0x48, 0x89, 0x04, 0x4B; emit_movq_ra(RAX, RBX, RCX, 2));
        assert_emit!(0x4D, 0x89, 0x3C, 0x03; emit_movq_ra(R15, R11, RAX, 1));
    }

    #[test]
    fn test_movl_ra() {
        assert_emit!(0x89, 0x0c, 0xd8; emit_movl_ra(RCX, RAX, RBX, 8));
        assert_emit!(0x89, 0x1C, 0x81; emit_movl_ra(RBX, RCX, RAX, 4));
        assert_emit!(0x89, 0x04, 0x4B; emit_movl_ra(RAX, RBX, RCX, 2));
        assert_emit!(0x45, 0x89, 0x3C, 0x03; emit_movl_ra(R15, R11, RAX, 1));
    }

    #[test]
    fn test_shll_reg_reg() {
        assert_emit!(0xD3, 0xE0; emit_shll_reg_cl(RAX));
        assert_emit!(0x41, 0xD3, 0xE1; emit_shll_reg_cl(R9));
    }

    #[test]
    fn test_cmp_memindex_reg() {
        let p = MachineMode::Ptr;

        // cmp [rax+rbx*8+1],rcx
        assert_emit!(0x48, 0x39, 0x4c, 0xd8, 1; emit_cmp_memindex_reg(p, RAX, RBX, 8, 1, RCX));

        // cmp [rax+rbx*8],rcx
        assert_emit!(0x48, 0x39, 0x0c, 0xd8; emit_cmp_memindex_reg(p, RAX, RBX, 8, 0, RCX));

        // cmp [rax+rbx*8+256],rcx
        assert_emit!(0x48, 0x39, 0x8c, 0xd8, 0, 1, 0, 0;
                     emit_cmp_memindex_reg(p, RAX, RBX, 8, 256, RCX));

        // cmp [r8+rbp*1],rsp
        assert_emit!(0x49, 0x39, 0x24, 0x28;
                     emit_cmp_memindex_reg(p, R8, RBP, 1, 0, RSP));

        // cmp [rsi+r9*1],rdi
        assert_emit!(0x4a, 0x39, 0x3c, 0x0e; emit_cmp_memindex_reg(p, RSI, R9, 1, 0, RDI));

        // cmp [rsp+rsi*1],r15
        assert_emit!(0x4c, 0x39, 0x3c, 0x34; emit_cmp_memindex_reg(p, RSP, RSI, 1, 0, R15));

        // cmp [rsp+rbp],rax
        assert_emit!(0x48, 0x39, 0x04, 0x2c; emit_cmp_memindex_reg(p, RSP, RBP, 1, 0, RAX));

    }

    #[test]
    #[should_panic]
    fn test_cmp_memindex_reg_base_rip() {
        let mut buf = MacroAssembler::new();
        emit_cmp_memindex_reg(&mut buf, MachineMode::Ptr, RIP, RAX, 1, 0, RAX);
    }

    #[test]
    #[should_panic]
    fn test_cmp_memindex_reg_index_rip() {
        let mut buf = MacroAssembler::new();
        emit_cmp_memindex_reg(&mut buf, MachineMode::Ptr, RAX, RIP, 1, 0, RAX);
    }

    #[test]
    #[should_panic]
    fn test_cmp_memindex_reg_dest_rip() {
        let mut buf = MacroAssembler::new();
        emit_cmp_memindex_reg(&mut buf, MachineMode::Ptr, RAX, RBX, 1, 0, RIP);
    }

    #[test]
    fn test_cmp_mem_reg() {
        let p = MachineMode::Ptr;

        // cmp [rbx+1],rax
        assert_emit!(0x48, 0x39, 0x43, 1; emit_cmp_mem_reg(p, RBX, 1, RAX));

        // cmp [rbx+256],rax
        assert_emit!(0x48, 0x39, 0x83, 0, 1, 0, 0; emit_cmp_mem_reg(p, RBX, 256, RAX));

        // cmp [rdi+1],rax
        assert_emit!(0x48, 0x39, 0x47, 1; emit_cmp_mem_reg(p, RDI, 1, RAX));

        // cmp [r9+1],rax
        assert_emit!(0x49, 0x39, 0x41, 1; emit_cmp_mem_reg(p, R9, 1, RAX));

        // cmp [rdi+1],r10
        assert_emit!(0x4c, 0x39, 0x57, 1; emit_cmp_mem_reg(p, RDI, 1, R10));

        // cmp [rip+1], rax
        assert_emit!(0x48, 0x39, 0x05, 1, 0, 0, 0; emit_cmp_mem_reg(p, RIP, 1, RAX));

        let i = MachineMode::Int32;

        // cmp [rbx+1], eax
        assert_emit!(0x39, 0x43, 1; emit_cmp_mem_reg(i, RBX, 1, RAX));

        // cmp [rbx+1], r10d
        assert_emit!(0x44, 0x39, 0x53, 1; emit_cmp_mem_reg(i, RBX, 1, R10));
    }

    #[test]
    fn test_cmp_mem_imm() {
        let p = MachineMode::Ptr;

        // cmp [rbx+1], 2
        assert_emit!(0x48, 0x83, 0x7b, 1, 2; emit_cmp_mem_imm(p, RBX, 1, 2));

        // cmp [rbx+256], 2
        assert_emit!(0x48, 0x83, 0xBB, 0, 1, 0, 0, 2; emit_cmp_mem_imm(p, RBX, 256, 2));

        // cmp [rdi+1], 256
        assert_emit!(0x48, 0x81, 0x7F, 1, 0, 1, 0, 0; emit_cmp_mem_imm(p, RDI, 1, 256));

        // cmp [r9+1], 2
        assert_emit!(0x49, 0x83, 0x79, 1, 2; emit_cmp_mem_imm(p, R9, 1, 2));

        let i = MachineMode::Int32;

        // cmp [rbx+1], 2
        assert_emit!(0x83, 0x7B, 1, 2; emit_cmp_mem_imm(i, RBX, 1, 2));

        // cmp [rbx+1], 256
        assert_emit!(0x81, 0x7B, 1, 0, 1, 0, 0; emit_cmp_mem_imm(i, RBX, 1, 256));

        let b = MachineMode::Int8;

        // cmp [rbx+1], 2
        assert_emit!(0x80, 0x7B, 1, 2; emit_cmp_mem_imm(b, RBX, 1, 2));

        // cmp [R15+256], 2
        assert_emit!(0x41, 0x80, 0xBF, 0, 1, 0, 0, 2; emit_cmp_mem_imm(b, R15, 256, 2));
    }

    #[test]
    #[should_panic]
    fn test_cmp_mem_imm_i32_for_i8() {
        let mut buf = MacroAssembler::new();
        emit_cmp_mem_imm(&mut buf, MachineMode::Int8, R15, 256, 256);
    }

    #[test]
    #[should_panic]
    fn test_cmp_mem_reg_dest_rip() {
        let mut buf = MacroAssembler::new();
        emit_cmp_mem_reg(&mut buf, MachineMode::Ptr, RAX, 1, RIP);
    }

    #[test]
    fn test_sub_imm_mem() {
        assert_emit!(0x83, 0x28, 1; emit_sub_imm_mem(MachineMode::Int32, RAX, 1));
        assert_emit!(0x48, 0x83, 0x28, 1; emit_sub_imm_mem(MachineMode::Ptr, RAX, 1));
        assert_emit!(0x49, 0x83, 0x29, 1; emit_sub_imm_mem(MachineMode::Ptr, R9, 1));
        assert_emit!(0x49, 0x83, 0x28, 1; emit_sub_imm_mem(MachineMode::Ptr, R8, 1));
        assert_emit!(0x41, 0x80, 0x28, 1; emit_sub_imm_mem(MachineMode::Int8, R8, 1));
    }

    #[test]
    fn test_mov_memindex_reg() {
        assert_emit!(0x48, 0x8b, 0x54, 0x88, 0x0c;
            emit_mov_memindex_reg(MachineMode::Ptr, RAX, RCX, 4, 12, RDX));
        assert_emit!(0x8b, 0x54, 0x88, 0x0c;
            emit_mov_memindex_reg(MachineMode::Int32, RAX, RCX, 4, 12, RDX));
        assert_emit!(0x8a, 0x54, 0x88, 0x0c;
            emit_mov_memindex_reg(MachineMode::Int8, RAX, RCX, 4, 12, RDX));

        assert_emit!(0x4f, 0x8b, 0x6c, 0xfe, 0x10;
            emit_mov_memindex_reg(MachineMode::Ptr, R14, R15, 8, 16, R13));
    }

    #[test]
    fn test_mov_reg_memindex() {
        assert_emit!(0x48, 0x89, 0x54, 0x88, 0x0c;
            emit_mov_reg_memindex(MachineMode::Ptr, RDX, RAX, RCX, 4, 12));
        assert_emit!(0x89, 0x54, 0x88, 0x0c;
            emit_mov_reg_memindex(MachineMode::Int32, RDX, RAX, RCX, 4, 12));
        assert_emit!(0x88, 0x54, 0x88, 0x0c;
            emit_mov_reg_memindex(MachineMode::Int8, RDX, RAX, RCX, 4, 12));

        assert_emit!(0x4f, 0x89, 0x6c, 0xfe, 0x10;
            emit_mov_reg_memindex(MachineMode::Ptr, R13, R14, R15, 8, 16));
    }
}