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
    /// Closure cell - heap-allocated box for variables shared with nested functions
    /// The cell contains a pointer to the actual value
    ClosureCell(PointerValue<'ctx>),  // Pointer to the cell structure
}

/// Variable info: stores the storage strategy and LLVM type
#[derive(Debug, Clone)]
pub struct VarInfo<'ctx> {
    pub storage: VarStorage<'ctx>,
    pub var_type: VarType,
    /// Optional class name for instance variables (OOP support)
    pub class_name: Option<String>,
    /// If this variable is captured by a nested function, stores the cell pointer
    /// This is the actual value storage for closure cells
    pub closure_value_ptr: Option<PointerValue<'ctx>>,
}

impl<'ctx> VarInfo<'ctx> {
    /// Create a new variable with stack allocation
    pub fn new_stack(alloca: PointerValue<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Stack(alloca), var_type, class_name: None, closure_value_ptr: None }
    }

    /// Create a new variable with register allocation
    pub fn new_register(value: BasicValueEnum<'ctx>, var_type: VarType) -> Self {
        Self { storage: VarStorage::Register(value), var_type, class_name: None, closure_value_ptr: None }
    }

    /// Create a new variable with stack allocation and class name
    pub fn new_stack_with_class(alloca: PointerValue<'ctx>, var_type: VarType, class_name: String) -> Self {
        Self { storage: VarStorage::Stack(alloca), var_type, class_name: Some(class_name), closure_value_ptr: None }
    }

    /// Create a new variable with register allocation and class name
    pub fn new_register_with_class(value: BasicValueEnum<'ctx>, var_type: VarType, class_name: String) -> Self {
        Self { storage: VarStorage::Register(value), var_type, class_name: Some(class_name), closure_value_ptr: None }
    }

    /// Create a new variable with closure cell storage
    pub fn new_closure_cell(cell_ptr: PointerValue<'ctx>, var_type: VarType, value_ptr: PointerValue<'ctx>) -> Self {
        Self { storage: VarStorage::ClosureCell(cell_ptr), var_type, class_name: None, closure_value_ptr: Some(value_ptr) }
    }

    /// Get the alloca pointer if this variable uses stack allocation
    pub fn get_alloca(&self) -> Option<PointerValue<'ctx>> {
        match &self.storage {
            VarStorage::Stack(alloca) => Some(*alloca),
            VarStorage::Register(_) => None,
            VarStorage::ClosureCell(_) => None,
        }
    }

    /// Get the register value if this variable uses register allocation
    pub fn get_register(&self) -> Option<BasicValueEnum<'ctx>> {
        match &self.storage {
            VarStorage::Stack(_) => None,
            VarStorage::Register(value) => Some(*value),
            VarStorage::ClosureCell(_) => None,
        }
    }

    /// Get the closure cell pointer if this variable uses closure cell storage
    pub fn get_closure_cell(&self) -> Option<PointerValue<'ctx>> {
        match &self.storage {
            VarStorage::ClosureCell(cell_ptr) => Some(*cell_ptr),
            _ => None,
        }
    }

    /// Get the value pointer for closure cells (the actual storage inside the cell)
    pub fn get_closure_value_ptr(&self) -> Option<PointerValue<'ctx>> {
        self.closure_value_ptr
    }

    /// Check if this variable uses register allocation
    pub fn is_register(&self) -> bool {
        matches!(self.storage, VarStorage::Register(_))
    }

    /// Check if this variable uses stack allocation
    pub fn is_stack(&self) -> bool {
        matches!(self.storage, VarStorage::Stack(_))
    }

    /// Check if this variable uses closure cell storage
    pub fn is_closure_cell(&self) -> bool {
        matches!(self.storage, VarStorage::ClosureCell(_))
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
