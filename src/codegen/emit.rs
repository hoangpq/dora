use codegen::buffer::*;
use codegen::x64::emit::*;
use codegen::x64::reg::*;
use codegen::x64::reg::Reg::*;

use parser::ast::ctxt::*;

use sym::BuiltinType;

pub fn var_store(buf: &mut Buffer, ctxt: &Context, src: Reg, var: VarInfoId) {
    let var_infos = ctxt.var_infos.borrow();
    let var = &var_infos[var.0];

    match var.data_type {
        BuiltinType::Bool => emit_movb_reg_memq(buf, src, RBP, var.offset),
        BuiltinType::Int => emit_movl_reg_memq(buf, src, RBP, var.offset),
        BuiltinType::Str => emit_movq_reg_memq(buf, src, RBP, var.offset),
        BuiltinType::Unit => {},
    }
}

pub fn var_load(buf: &mut Buffer, ctxt: &Context, var: VarInfoId, dest: Reg) {
    let var_infos = ctxt.var_infos.borrow();
    let var = &var_infos[var.0];

    match var.data_type {
        BuiltinType::Bool => emit_movb_memq_reg(buf, RBP, var.offset, dest),
        BuiltinType::Int => emit_movl_memq_reg(buf, RBP, var.offset, dest),
        BuiltinType::Str => emit_movq_memq_reg(buf, RBP, var.offset, dest),
        BuiltinType::Unit => {},
    }
}