//! Main code generator that translates AST to LLVM IR

use crate::ast::{Expr, Module, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::{BasicValue, FunctionValue, GlobalValue};
use std::collections::HashMap;

use crate::codegen::builder::IRBuilder;
use crate::codegen::types::TypeMapper;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};
use crate::semantic::escape_analysis::EscapeAnalyzer;

/// Main code generator that translates AST to LLVM IR
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: inkwell::module::Module<'ctx>,
    builder: inkwell::builder::Builder<'ctx>,
    ir_builder: IRBuilder<'ctx>,
    type_mapper: TypeMapper<'ctx>,
    variables: HashMap<String, VarInfo<'ctx>>,
    functions: HashMap<String, FunctionValue<'ctx>>,
    global_constants: HashMap<String, GlobalValue<'ctx>>,
    loop_stack: Vec<LoopContext<'ctx>>,
    escape_analyzer: EscapeAnalyzer,
    current_function: Option<String>,
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
            global_constants: HashMap::new(),
            loop_stack: Vec::new(),
            escape_analyzer: EscapeAnalyzer::new(),
            current_function: None,
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {
        // Run escape analysis first
        self.escape_analyzer.analyze_module(module);

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

        // Second pass: process module-level constant assignments first
        // Module-level assignments like "PI = 3.14" are treated as immutable constants
        for stmt in &module.statements {
            if let Stmt::Assign { target, value, .. } = stmt {
                if let Expr::Ident(name, _) = target.as_ref() {
                    // Check if it's a simple literal value (int, float, string, bool)
                    let is_literal = matches!(
                        value.as_ref(),
                        Expr::Int(..)
                            | Expr::Float(..)
                            | Expr::Str(..)
                            | Expr::Bool(..)
                            | Expr::None(..)
                    );

                    if is_literal {
                        self.create_global_constant(name, value)?;
                    }
                }
            }
        }

        // Third pass: define all functions
        let mut top_level_stmts = Vec::new();
        for stmt in &module.statements {
            if let Stmt::Function {
                name,
                params,
                return_type,
                body,
                is_async: _,
                ..
            } = stmt
            {
                // Async functions are not fully implemented yet - treat as regular functions
                // TODO: Implement proper async/await with state machine transformation
                self.define_function(name, params, return_type, body)?;
            } else {
                // Skip constant assignments - they're already handled
                let is_constant_assign = match stmt {
                    Stmt::Assign { target, value, .. } => {
                        if let Expr::Ident(name, _) = target.as_ref() {
                            self.global_constants.contains_key(name)
                                && matches!(
                                    value.as_ref(),
                                    Expr::Int(..)
                                        | Expr::Float(..)
                                        | Expr::Str(..)
                                        | Expr::Bool(..)
                                        | Expr::None(..)
                                )
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if !is_constant_assign {
                    // Skip type declarations (Class, Struct)
                    let is_type_decl = matches!(stmt, Stmt::Class { .. } | Stmt::Struct { .. });

                    if !is_type_decl {
                        top_level_stmts.push(stmt.clone());
                    }
                }
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
        let saved_current_function = self.current_function.clone();
        self.current_function = Some(name.to_string());

        let func = self.functions.get(name).copied().unwrap();
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Set up parameters with alloca
        // Use escape analysis to determine if parameter needs stack allocation
        for (i, param) in params.iter().enumerate() {
            let param_value = func.get_nth_param(i as u32).unwrap();

            // Check escape analysis for this parameter
            let _can_stack_alloc = self.escape_analyzer.can_stack_allocate(name, &param.name);

            // Determine if parameter is a reference type (pointer)
            let is_ref_type = param_value.is_pointer_value();

            // Mark parameter as reference type in escape analyzer
            self.escape_analyzer
                .set_reference_type(name, &param.name, is_ref_type);

            // Always allocate on stack for now (escape analysis informs optimization decisions)
            // In a more advanced implementation, we might skip alloca for non-escaping params
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
            self.variables
                .insert(param.name.clone(), VarInfo::new_stack(alloca, var_type));
        }

        // Generate body using escape analysis
        for stmt in body {
            crate::codegen::statements::generate_stmt_with_escape(
                self.context,
                &self.module,
                &self.builder,
                &self.ir_builder,
                &mut self.variables,
                &self.functions,
                &mut self.global_constants,
                &mut self.loop_stack,
                stmt,
                &mut self.escape_analyzer,
                name,
            )?;
        }

        // Generate ARC cleanup for local variables before return
        self.generate_arc_cleanup(name);

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
        self.current_function = saved_current_function;

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
        // For top-level code, use a pseudo-function name
        for stmt in stmts {
            crate::codegen::statements::generate_stmt_with_escape(
                self.context,
                &self.module,
                &self.builder,
                &self.ir_builder,
                &mut self.variables,
                &self.functions,
                &mut self.global_constants,
                &mut self.loop_stack,
                stmt,
                &mut self.escape_analyzer,
                "__module_level__",
            )?;
        }

        // Generate ARC cleanup for module-level variables
        self.generate_arc_cleanup("__module_level__");

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

    /// Generate ARC cleanup code for local variables at function exit
    fn generate_arc_cleanup(&mut self, _function_name: &str) {
        // ARC cleanup is disabled: channels, lists, and other pointer types currently
        // don't have an ARC header, so calling vp_release on them causes a segfault.
        // When proper ARC allocation is wired up for user objects, re-enable this.
    }

    /// Create a global constant from a literal expression
    fn create_global_constant(&mut self, name: &str, value: &Expr) -> Result<(), String> {
        let val = match value {
            Expr::Int(n, _) => self.ir_builder.i64_const(*n).as_basic_value_enum(),
            Expr::Float(n, _) => self.ir_builder.f64_const(*n).as_basic_value_enum(),
            Expr::Bool(b, _) => self.ir_builder.bool_const(*b).as_basic_value_enum(),
            Expr::Str(s, _) => self
                .ir_builder
                .string_const(&self.module, s)
                .as_basic_value_enum(),
            Expr::None(_) => self.ir_builder.i64_const(0).as_basic_value_enum(),
            _ => {
                return Err(format!(
                    "Cannot create global constant from non-literal expression"
                ))
            }
        };

        let global = self.module.add_global(val.get_type(), None, name);
        global.set_constant(true);
        global.set_initializer(&val);
        global.set_unnamed_addr(false);

        self.global_constants.insert(name.to_string(), global);
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
