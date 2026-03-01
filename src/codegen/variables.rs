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
#[derive(Debug, Clone)]
pub struct VarInfo<'ctx> {
    pub storage: VarStorage<'ctx>,
    pub var_type: VarType,
    /// Optional class name for instance variables (OOP support)
    pub class_name: Option<String>,
}

impl<'ctx> VarInfo<'ctx> {
    /// Create a new variable with stack allocation
    pub fn new_stack(alloca: PointerValue<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Stack(alloca), var_type, class_name: None }
    }

    /// Create a new variable with register allocation
    pub fn new_register(value: BasicValueEnum<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Register(value), var_type, class_name: None }
    }

    /// Create a new variable with stack allocation and class name
    pub fn new_stack_with_class(alloca: PointerValue<'ctx>, var_type: VarType, class_name: String) -> Self {
        Self { storage: VarStorage::Stack(alloca), var_type, class_name: Some(class_name) }
    }

    /// Create a new variable with register allocation and class name
    pub fn new_register_with_class(value: BasicValueEnum<'ctx>, var_type: VarType, class_name: String) -> Self {
        Self { storage: VarStorage::Register(value), var_type, class_name: Some(class_name) }
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
