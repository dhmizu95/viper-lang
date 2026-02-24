//! Common state for code generation (shared across all codegen modules)

use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::HashMap;

use crate::codegen::builder::IRBuilder;
use crate::codegen::variables::{VarInfo, LoopContext};

/// State needed for code generation (shared across modules)
pub struct CodeGenState<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a inkwell::module::Module<'ctx>,
    pub builder: &'a inkwell::builder::Builder<'ctx>,
    pub ir_builder: &'a IRBuilder<'ctx>,
    pub variables: &'a mut HashMap<String, VarInfo<'ctx>>,
    pub functions: &'a HashMap<String, FunctionValue<'ctx>>,
    pub global_constants: &'a mut HashMap<String, GlobalValue<'ctx>>,
    pub loop_stack: &'a mut Vec<LoopContext<'ctx>>,
}

impl<'a, 'ctx> CodeGenState<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &'a inkwell::module::Module<'ctx>,
        builder: &'a inkwell::builder::Builder<'ctx>,
        ir_builder: &'a IRBuilder<'ctx>,
        variables: &'a mut HashMap<String, VarInfo<'ctx>>,
        functions: &'a HashMap<String, FunctionValue<'ctx>>,
        global_constants: &'a mut HashMap<String, GlobalValue<'ctx>>,
        loop_stack: &'a mut Vec<LoopContext<'ctx>>,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            ir_builder,
            variables,
            functions,
            global_constants,
            loop_stack,
        }
    }
}
