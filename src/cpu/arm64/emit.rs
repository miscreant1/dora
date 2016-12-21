use cpu::arm64::asm::*;
use cpu::arm64::*;
use cpu::{Mem, Reg, trap};
use cpu::arm64::reg::*;
use baseline::codegen::CondCode;
use lexer::position::Position;
use masm::*;
use mem::ptr_width;
use object::IntArray;
use ty::MachineMode;
use vtable::VTable;

pub fn prolog(buf: &mut MacroAssembler, stacksize: i32) {
    buf.emit_u32(stp_pre(1, REG_FP, REG_LR, REG_SP, -2));
    buf.emit_u32(add_reg(1, REG_FP, REG_SP, REG_ZERO));

    if stacksize > 0 {
        let scratch = get_scratch();
        load_int_const(buf, MachineMode::Ptr, scratch, stacksize);
        buf.emit_u32(sub_reg(1, REG_SP, REG_SP, scratch));
    }
}

pub fn epilog(buf: &mut MacroAssembler, stacksize: i32) {
    if stacksize > 0 {
        let scratch = get_scratch();
        load_int_const(buf, MachineMode::Ptr, scratch, stacksize);
        buf.emit_u32(add_reg(1, REG_SP, REG_SP, scratch));
    }

    buf.emit_u32(add_reg(1, REG_SP, REG_FP, REG_ZERO));
    buf.emit_u32(ldp_post(1, REG_FP, REG_LR, REG_SP, 2));
    buf.emit_u32(ret());
}

pub fn direct_call(buf: &mut MacroAssembler, ptr: *const u8) {
    let disp = buf.add_addr(ptr);
    let pos = buf.pos() as i32;

    let scratch = get_scratch();

    load_constpool(buf, REG_RESULT, disp + pos);
    br(scratch);
}

pub fn indirect_call(buf: &mut MacroAssembler, index: u32) {
    let obj = REG_PARAMS[0];

    // REG_RESULT = [obj] (load vtable)
    load_mem(buf, MachineMode::Ptr, REG_RESULT, Mem::Base(obj, 0));

    // calculate offset of VTable entry
    let disp = VTable::offset_of_method_table() + (index as i32) * ptr_width();

    // load vtable entry
    load_mem(buf, MachineMode::Ptr, REG_RESULT, Mem::Base(REG_RESULT, disp));

    // call *REG_RESULT
    buf.emit_u32(blr(REG_RESULT));
}

pub fn load_array_elem(buf: &mut MacroAssembler, mode: MachineMode, dest: Reg, array: Reg, index: Reg) {
    assert!(mode == MachineMode::Int32);

    load_mem(buf, mode, dest, Mem::Index(array, index, mode.size(), IntArray::offset_of_data()));
}

pub fn store_array_elem(buf: &mut MacroAssembler, mode: MachineMode, array: Reg, index: Reg, value: Reg) {
    assert!(mode == MachineMode::Int32);

    store_mem(buf, MachineMode::Int32,
              Mem::Index(array, index, 4, IntArray::offset_of_data()), value);
}

pub fn test_if_nil_bailout(buf: &mut MacroAssembler, pos: Position, reg: Reg) {
    cmp_reg(buf, MachineMode::Ptr, reg, reg);

    let lbl = buf.create_label();
    jump_if(buf, CondCode::Zero, lbl);
    buf.emit_bailout(lbl, trap::NIL, pos);
}

pub fn test_if_nil(buf: &mut MacroAssembler, reg: Reg) -> Label {
    cmp_reg(buf, MachineMode::Ptr, reg, reg);

    let lbl = buf.create_label();
    jump_if(buf, CondCode::Zero, lbl);

    lbl
}

pub fn set(buf: &mut MacroAssembler, dest: Reg, op: CondCode) {
    buf.emit_u32(cmp_imm(0, dest, 0, 0));
    buf.emit_u32(cset(0, dest, CondCode::NotEqual.into()));
}

pub fn cmp_mem(buf: &mut MacroAssembler, mode: MachineMode, mem: Mem, rhs: Reg) {
    let scratch = get_scratch();

    load_mem(buf, mode, scratch, mem);
    cmp_reg(buf, mode, scratch, rhs);
}

pub fn cmp_mem_imm(buf: &mut MacroAssembler, mode: MachineMode, mem: Mem, imm: i32) {
    let (scratch1, scratch2) = get_scratch_registers();

    load_mem(buf, mode, scratch1, mem);
    load_int_const(buf, mode, scratch2, imm);

    cmp_reg(buf, mode, scratch1, scratch2);
}

pub fn cmp_reg(buf: &mut MacroAssembler, mode: MachineMode, lhs: Reg, rhs: Reg) {
    buf.emit_u32(cmp_shreg(size_flag(mode), lhs, rhs, Shift::LSL, 0));
}

pub fn test_and_jump_if(buf: &mut MacroAssembler, cond: CondCode, reg: Reg, lbl: Label) {
    assert!(cond == CondCode::Zero || cond == CondCode::NonZero);

    unimplemented!();
}

pub fn jump_if(buf: &mut MacroAssembler, cond: CondCode, lbl: Label) {
    unimplemented!();
}

pub fn jump(buf: &mut MacroAssembler, lbl: Label) {
    unimplemented!();
}

pub fn int_div(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(sdiv(0, dest, lhs, rhs));
}

pub fn int_mod(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(sdiv(0, dest, lhs, rhs));
    buf.emit_u32(msub(0, dest, dest, rhs, lhs));
}

pub fn int_mul(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(mul(0, dest, lhs, rhs));
}

pub fn int_add(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(add_reg(0, dest, lhs, rhs));
}

pub fn int_sub(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(sub_reg(0, dest, lhs, rhs));
}

pub fn int_shl(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(lslv(0, dest, lhs, rhs));
}

pub fn int_or(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(orr_shreg(0, dest, lhs, rhs, Shift::LSL, 0));
}

pub fn int_and(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(and_shreg(0, dest, lhs, rhs, Shift::LSL, 0));
}

pub fn int_xor(buf: &mut MacroAssembler, dest: Reg, lhs: Reg, rhs: Reg) {
    buf.emit_u32(eor_shreg(0, dest, lhs, rhs, Shift::LSL, 0));
}

pub fn check_index_out_of_bounds(buf: &mut MacroAssembler, pos: Position, array: Reg,
                                 index: Reg, temp: Reg) {
    load_mem(buf, MachineMode::Int32, temp,
             Mem::Base(array, IntArray::offset_of_length()));
    cmp_reg(buf, MachineMode::Int32, index, temp);

    let lbl = buf.create_label();
    jump_if(buf, CondCode::UnsignedGreaterEq, lbl);
    buf.emit_bailout(lbl, trap::INDEX_OUT_OF_BOUNDS, pos);
}

pub fn load_nil(buf: &mut MacroAssembler, dest: Reg) {
    buf.emit_u32(add_imm(1, dest, REG_ZERO, 0, 0));
}

pub fn load_mem(buf: &mut MacroAssembler, mode: MachineMode, dest: Reg, mem: Mem) {
    unimplemented!();
}

pub fn store_mem(buf: &mut MacroAssembler, mode: MachineMode, mem: Mem, src: Reg) {
    unimplemented!();
}

pub fn copy_reg(buf: &mut MacroAssembler, mode: MachineMode, dest: Reg, src: Reg) {
    buf.emit_u32(orr_shreg(size_flag(mode), dest, REG_ZERO, src, Shift::LSL, 0));
}

pub fn load_constpool(buf: &mut MacroAssembler, dest: Reg, disp: i32) {
    unimplemented!();
}

pub fn call_reg(buf: &mut MacroAssembler, reg: Reg) {
    buf.emit_u32(blr(reg));
}

pub fn debug(buf: &mut MacroAssembler) {
    unimplemented!();
}

pub fn load_int_const(buf: &mut MacroAssembler, mode: MachineMode, dest: Reg, imm: i32) {
    let sf = size_flag(mode);
    let register_size = match mode {
        MachineMode::Int8 => unimplemented!(),
        MachineMode::Int32 => 32,
        MachineMode::Ptr => 64,
    };
    let imm = imm as i64 as u64;

    if fits_movz(imm, register_size) {
        buf.emit_u32(movz(sf, dest, imm as u32, shift_movz(imm)));

    } else if fits_movn(imm, register_size) {
        buf.emit_u32(movn(sf, dest, imm as u32, shift_movn(imm)));

    } else {
        unimplemented!();
    }
}

pub fn load_true(buf: &mut MacroAssembler, dest: Reg) {
    buf.emit_u32(movz(0, dest, 1, 0));
}

pub fn load_false(buf: &mut MacroAssembler, dest: Reg) {
    buf.emit_u32(movz(0, dest, 0, 0));
}

pub fn int_neg(buf: &mut MacroAssembler, dest: Reg, src: Reg) {
    buf.emit_u32(sub_reg(0, dest, REG_ZERO, src));
}

pub fn int_not(buf: &mut MacroAssembler, dest: Reg, src: Reg) {
    buf.emit_u32(orn_shreg(0, dest, REG_ZERO, src, Shift::LSL, 0));
}

pub fn bool_not(buf: &mut MacroAssembler, dest: Reg, src: Reg) {
    let scratch = get_scratch();

    buf.emit_u32(movz(0, scratch, 1, 0));
    buf.emit_u32(eor_shreg(0, dest, src, scratch, Shift::LSL, 0));
    buf.emit_u32(uxtb(dest, dest));
}

fn size_flag(mode: MachineMode) -> u32 {
    match mode {
        MachineMode::Int8 | MachineMode::Int32 => 0,
        MachineMode::Ptr => 1,
    }
}

fn get_scratch() -> Reg {
    SCRATCH[0]
}

fn get_scratch_registers() -> (Reg, Reg) {
    (SCRATCH[0], SCRATCH[1])
}