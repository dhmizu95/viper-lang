//! Variable management for Viper code generation

use inkwell::values::PointerValue;
pub use crate::codegen::types::VarType;

/// Variable info: stores both the alloca pointer and its LLVM type
pub struct VarInfo<'ctx> {
    pub alloca: PointerValue<'ctx>,
    pub var_type: VarType,
}

impl<'ctx> VarInfo<'ctx> {
    pub fn new(alloca: PointerValue<'ctx>, var_type: VarType) -> Self {
        Self { alloca, var_type }
    }
}

/// Loop context for break/continue support
pub struct LoopContext<'ctx> {
    pub break_block: inkwell::basic_block::BasicBlock<'ctx>,
    pub continue_block: inkwell::basic_block::BasicBlock<'ctx>,
}

impl<'ctx> LoopContext<'ctx> {
    pub fn new(
        break_block: inkwell::basic_block::BasicBlock<'ctx>,
        continue_block: inkwell::basic_block::BasicBlock<'ctx>,
    ) -> Self {
        Self {
            break_block,
            continue_block,
        }
    }
}
