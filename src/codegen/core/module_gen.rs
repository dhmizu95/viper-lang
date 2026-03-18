//! Module generation - generate() method for complete modules

use crate::ast::{Expr, Module, Stmt};

use crate::codegen::core::context::CodeGen;

impl<'ctx> CodeGen<'ctx> {
    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> crate::codegen::Result<()> {
        // Run escape analysis first
        self.escape_analyzer.analyze_module(module);

        // Run closure analysis to identify captured variables
        self.closure_analyzer.analyze_module(module);

        // Initialize class registry for OOP
        crate::codegen::oop::init_class_registry();

        // Declare runtime functions first
        crate::codegen::runtime::declare_runtime_functions(self.context, &self.module)?;

        // Generate __name__ builtin constant
        // For the main module, use "__main__"; for imported modules, use the module name
        self.generate_name_builtin()?;

        // First pass: declare all functions (including class methods and nested functions)
        self.declare_all_functions(&module.statements)?;

        // Generate class definitions (defines class methods)
        self.generate_classes(&module.statements)?;

        // PRE-PASS: Process imports to make them available in all functions
        for stmt in &module.statements {
            match stmt {
                Stmt::Import { module, alias, .. } => {
                    let mut closure_cells = std::collections::HashMap::new();
                    let mut state = crate::codegen::state::CodeGenState::new(
                        self.context,
                        &self.module,
                        &self.builder,
                        &self.ir_builder,
                        &mut self.variables,
                        &self.functions,
                        &mut self.global_constants,
                        &mut self.loop_stack,
                        &mut self.list_vars,
                        &mut self.dict_vars,
                        &mut self.bool_list_vars,
                        &mut self.bytearray_vars,
                        &mut self.bigint_vars,
                        &mut self.var_types,
                        &mut closure_cells,
                    );
                    crate::codegen::statements::core::imports::generate_import(&mut state, module, alias.as_deref())?;
                }
                Stmt::FromImport { module, names, .. } => {
                    let mut closure_cells = std::collections::HashMap::new();
                    let mut state = crate::codegen::state::CodeGenState::new(
                        self.context,
                        &self.module,
                        &self.builder,
                        &self.ir_builder,
                        &mut self.variables,
                        &self.functions,
                        &mut self.global_constants,
                        &mut self.loop_stack,
                        &mut self.list_vars,
                        &mut self.dict_vars,
                        &mut self.bool_list_vars,
                        &mut self.bytearray_vars,
                        &mut self.bigint_vars,
                        &mut self.var_types,
                        &mut closure_cells,
                    );
                    crate::codegen::statements::core::imports::generate_from_import(&mut state, module, names)?;
                }
                _ => {}
            }
        }

        // Second pass: Process module-level constants and variables
        // Module-level assignments create immutable constants by default (Python UPPER_CASE convention)
        // Note: Complex types (tuples, lists, dicts, arrays) cannot be global initializers
        // and will be handled as regular statements in viper_init
        for stmt in &module.statements {
            match stmt {
                Stmt::Const { name, value, .. } => {
                    // Only simple types can be global constants
                    if !Self::is_simple_initializer_expr(value) {
                        continue; // Will be handled as regular statement in viper_init
                    }
                    // Create a true constant (explicit const keyword)
                    // Note: We use set_constant(false) to allow runtime access,
                    // immutability is enforced by the type checker
                    let mut closure_cells = std::collections::HashMap::new();
                    let val = crate::codegen::expressions::generate_expr(
                        &mut crate::codegen::state::CodeGenState::new(
                            self.context,
                            &self.module,
                            &self.builder,
                            &self.ir_builder,
                            &mut self.variables,
                            &self.functions,
                            &mut self.global_constants,
                            &mut self.loop_stack,
                            &mut self.list_vars,
                            &mut self.dict_vars,
                            &mut self.bool_list_vars,
                            &mut self.bytearray_vars,
                            &mut self.bigint_vars,
                            &mut self.var_types,
                            &mut closure_cells,
                        ),
                        value,
                    )?;
                    let ty = val.get_type();
                    let global = self.module.add_global(ty, None, name);
                    global.set_constant(false); // Mutable at LLVM level (type checker enforces)
                    global.set_initializer(&val);
                    global.set_unnamed_addr(false);
                    self.global_constants.insert(name.clone(), global);
                }
                Stmt::Assign { target, value, .. } => {
                    // Module-level assignment creates an immutable constant by default
                    // This follows Python UPPER_CASE convention for constants
                    // Note: We use set_constant(false) to allow 'global' to work,
                    // immutability is enforced by the type checker
                    if let Expr::Ident(name, _) = target.as_ref() {
                        // Only simple types can be global initializers
                        if !Self::is_simple_initializer_expr(value) {
                            continue; // Will be handled as regular statement in viper_init
                        }
                        let mut closure_cells = std::collections::HashMap::new();
                        let val = crate::codegen::expressions::generate_expr(
                            &mut crate::codegen::state::CodeGenState::new(
                                self.context,
                                &self.module,
                                &self.builder,
                                &self.ir_builder,
                                &mut self.variables,
                                &self.functions,
                                &mut self.global_constants,
                                &mut self.loop_stack,
                                &mut self.list_vars,
                                &mut self.dict_vars,
                                &mut self.bool_list_vars,
                                &mut self.bytearray_vars,
                                &mut self.bigint_vars,
                                &mut self.var_types,
                                &mut closure_cells,
                            ),
                            value,
                        )?;
                        let ty = val.get_type();
                        let global = self.module.add_global(ty, None, name);
                        global.set_constant(false); // Mutable at LLVM level (type checker enforces)
                        global.set_initializer(&val);
                        global.set_unnamed_addr(false);
                        self.global_constants.insert(name.clone(), global);
                    }
                }
                _ => {}
            }
        }

        // Third pass: define all functions (including nested ones)
        self.define_all_functions(&module.statements)?;

        // Collect top-level statements (non-function statements)
        let mut top_level_stmts = Vec::new();
        for stmt in &module.statements {
            match stmt {
                Stmt::Function { .. } | Stmt::Extern { .. } => {}
                _ => {
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
                        let is_type_or_extern_decl = matches!(
                            stmt,
                            Stmt::Class { .. } | Stmt::Struct { .. } | Stmt::Extern { .. }
                        );

                        if !is_type_or_extern_decl {
                            top_level_stmts.push(stmt.clone());
                        }
                    }
                }
            }
        }

        // Generate main handling top-level statements
        self.generate_main_with_statements(&top_level_stmts)?;

        Ok(())
    }
}
