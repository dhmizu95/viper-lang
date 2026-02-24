#![allow(dead_code)]
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module as LLVMModule;

/// LLVM code generation context
pub struct CodeGenContext<'ctx> {
    pub context: &'ctx Context,
    pub module: LLVMModule<'ctx>,
    pub builder: Builder<'ctx>,
}

impl<'ctx> CodeGenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
        }
    }

    /// Get the LLVM module
    pub fn module(&self) -> &LLVMModule<'ctx> {
        &self.module
    }

    /// Verify the module
    pub fn verify(&self) -> Result<(), String> {
        self.module.verify().map_err(|e| e.to_string())
    }

    /// Print the LLVM IR
    pub fn print_ir(&self) -> String {
        self.module.to_string().to_string()
    }
}
