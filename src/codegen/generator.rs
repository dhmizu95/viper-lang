//! Main code generator that translates AST to LLVM IR

use crate::ast::{Module, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::FunctionValue;
use std::collections::HashMap;

use crate::codegen::builder::IRBuilder;
use crate::codegen::types::TypeMapper;
use crate::codegen::variables::{VarInfo, VarType, LoopContext};

/// Main code generator that translates AST to LLVM IR
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    ir_builder: IRBuilder<'ctx>,
    type_mapper: TypeMapper<'ctx>,
    variables: HashMap<String, VarInfo<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    loop_stack: Vec<LoopContext<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        let ir_builder = IRBuilder::new(context, &module);
        let type_mapper = TypeMapper::new(context);

        Self {
            context,
            module,
            builder,
            ir_builder,
            type_mapper,
            variables: HashMap::new(),
            functions: HashMap::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {
        // Declare runtime functions first
        crate::codegen::runtime::declare_runtime_functions(self.context, &self.module)?;

        // First pass: declare all functions
        for stmt in &module.statements {
            if let Stmt::Function {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                crate::codegen::functions::declare_function(
                    self.context,
                    &mut self.module,
                    &self.type_mapper,
                    &mut self.functions,
                    name,
                    params,
                    return_type,
                )?;
            }
        }

        // Second pass: define all functions
        let mut top_level_stmts = Vec::new();
        for stmt in &module.statements {
            if let Stmt::Function {
                name,
                params,
                return_type,
                body,
                ..
            } = stmt
            {
                self.define_function(name, params, return_type, body)?;
            } else {
                top_level_stmts.push(stmt.clone());
            }
        }

        // Generate main handling top-level statements
        self.generate_main_with_statements(&top_level_stmts)?;

        Ok(())
    }

    /// Define a function (generate body)
    fn define_function(
        &mut self,
        name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
        body: &[Stmt],
    ) -> Result<(), String> {
        // Save variables from previous function scope
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_loop_stack = std::mem::take(&mut self.loop_stack);

        let func = self.functions.get(name).copied().unwrap();
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Set up parameters with alloca
        for (i, param) in params.iter().enumerate() {
            let param_value = func.get_nth_param(i as u32).unwrap();
            let alloca = self
                .builder
                .build_alloca(param_value.get_type(), &param.name)
                .expect("alloca");
            self.builder
                .build_store(alloca, param_value)
                .expect("store");
            let var_type = if param_value.is_float_value() {
                VarType::Float
            } else if param_value.is_pointer_value() {
                VarType::Pointer
            } else {
                VarType::Int
            };
            self.variables.insert(param.name.clone(), VarInfo { alloca, var_type });
        }

        // Generate body
        for stmt in body {
            crate::codegen::statements::generate_stmt(
                self.context,
                &self.module,
                &self.builder,
                &self.ir_builder,
                &mut self.variables,
                &self.functions,
                &mut self.loop_stack,
                stmt,
            )?;
        }

        // Add implicit return if needed
        if self
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            match return_type {
                Some(Type::I8) | Some(Type::I16) | Some(Type::I32) | Some(Type::I64) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
                }
                Some(Type::F32) | Some(Type::F64) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.f64_const(0.0)));
                }
                Some(Type::Bool) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.ir_builder.bool_const(false)));
                }
                _ => {
                    self.ir_builder.build_return(&self.builder, None);
                }
            }
        }

        // Restore variables for next function
        self.variables = saved_variables;
        self.loop_stack = saved_loop_stack;

        Ok(())
    }

    /// Generate main function handling top-level statements
    fn generate_main_with_statements(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        let main_type = self.context.i64_type().fn_type(&[], false);

        let has_user_main = self.functions.contains_key("main");

        // Generate viper_init function for top-level statements
        let init_type = self.context.void_type().fn_type(&[], false);
        let init_func = self.module.add_function("viper_init", init_type, None);
        let init_entry = self.context.append_basic_block(init_func, "entry");
        self.builder.position_at_end(init_entry);

        // Generate top-level statements into init
        for stmt in stmts {
            crate::codegen::statements::generate_stmt(
                self.context,
                &self.module,
                &self.builder,
                &self.ir_builder,
                &mut self.variables,
                &self.functions,
                &mut self.loop_stack,
                stmt,
            )?;
        }
        self.ir_builder.build_return(&self.builder, None);

        // If user didn't define main, we define it
        if !has_user_main {
            let main_func = self.module.add_function("main", main_type, None);
            let entry = self.context.append_basic_block(main_func, "entry");
            self.builder.position_at_end(entry);

            // Call viper_init
            let _ = self.builder.build_call(init_func, &[], "call_init");

            self.ir_builder
                .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
        }

        Ok(())
    }

    /// Get the generated LLVM module
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    /// Verify the generated code
    pub fn verify(&self) -> Result<(), String> {
        self.module.verify().map_err(|e| e.to_string())
    }

    /// Print the generated IR
    #[allow(dead_code)]
    pub fn print_ir(&self) -> String {
        self.module.to_string().to_string()
    }
}
