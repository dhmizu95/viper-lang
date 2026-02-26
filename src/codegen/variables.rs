//! Variable management for Viper code generation

pub use crate::codegen::types::VarType;
use inkwell::values::{BasicValueEnum, PointerValue};

/// Storage strategy for a variable
#[derive(Debug, Clone)]
pub enum VarStorage<'ctx> {
    /// Stack allocation using alloca (for escaping variables)
    Stack(PointerValue<'ctx>),
    /// Register allocation using SSA value (for non-escaping variables)
    Register(BasicValueEnum<'ctx>),
}

/// Variable info: stores the storage strategy and LLVM type
pub struct VarInfo<'ctx> {
    pub storage: VarStorage<'ctx>,
    pub var_type: VarType,
}

impl<'ctx> VarInfo<'ctx> {
    /// Create a new variable with stack allocation
    pub fn new_stack(alloca: PointerValue<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Stack(alloca), var_type }
    }

    /// Create a new variable with register allocation
    pub fn new_register(value: BasicValueEnum<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Register(value), var_type }
    }

    /// Get the alloca pointer if this variable uses stack allocation
    pub fn get_alloca(&self) -> Option<PointerValue<'ctx>> {
        match &self.storage {
            VarStorage::Stack(alloca) => Some(*alloca),
            VarStorage::Register(_) => None,
        }
    }

    /// Get the register value if this variable uses register allocation
    pub fn get_register(&self) -> Option<BasicValueEnum<'ctx>> {
        match &self.storage {
            VarStorage::Stack(_) => None,
            VarStorage::Register(value) => Some(*value),
        }
    }

    /// Check if this variable uses register allocation
    pub fn is_register(&self) -> bool {
        matches!(self.storage, VarStorage::Register(_))
    }

    /// Check if this variable uses stack allocation
    pub fn is_stack(&self) -> bool {
        matches!(self.storage, VarStorage::Stack(_))
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
        Self { break_block, continue_block }
    }
}
