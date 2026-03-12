//! Function declaration and definition methods

use crate::ast::{Expr, Stmt, Type};

use crate::codegen::core::context::CodeGen;
use crate::codegen::variables::{VarInfo, VarType};
use crate::utils::mangling::mangle_function_name_with_closure;

impl<'ctx> CodeGen<'ctx> {
    /// Define all functions recursively (including nested functions)
    pub(crate) fn define_all_functions(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        // First pass: declare all functions at this level with closure cell parameters
        for stmt in stmts {
            if let Stmt::Function { name, params, return_type, body, decorators, .. } = stmt {
                // Get closure info to determine nonlocal variables
                let closure_info = self.closure_analyzer.get_closure_info(name);
                let nonlocal_vars: Vec<String> = closure_info
                    .map(|info| info.nonlocal_vars.iter().cloned().collect())
                    .unwrap_or_default();

                // Compute mangled name including closure info
                // Rename user's main to __user_main to match declaration
                let func_name = if name == "main" { "__user_main" } else { name };

                // Use type annotations directly if present, otherwise default to I64
                // This must match the logic in declare_function_with_closure
                let param_types: Vec<Type> = params.iter().map(|p| {
                    if let Some(ref ty) = p.type_ann {
                        ty.clone()
                    } else {
                        // Default to I64 for unannotated parameters (backward compatible)
                        Type::I64
                    }
                }).collect();

                let mangled_name = mangle_function_name_with_closure(func_name, &param_types, &nonlocal_vars);
                
                // Check for memoization decorators
                let is_lru_cache = decorators.iter().any(|d| d.name == "lru_cache");
                let is_cache = decorators.iter().any(|d| d.name == "cache");
                
                if is_lru_cache || is_cache {
                    // Get maxsize from decorator arguments
                    let maxsize = if is_lru_cache {
                        decorators.iter()
                            .find(|d| d.name == "lru_cache")
                            .and_then(|d| {
                                // Check for maxsize keyword argument
                                for (key, val) in &d.keywords {
                                    if key == "maxsize" {
                                        if let Expr::Int(v, _) = val {
                                            return Some(*v);
                                        }
                                    }
                                }
                                // Check for positional argument
                                d.args.first().and_then(|arg| {
                                    if let Expr::Int(v, _) = arg {
                                        return Some(*v);
                                    }
                                    None
                                })
                            })
                            .unwrap_or(128)  // Default maxsize
                    } else {
                        0  // Unbounded for @cache
                    };
                    
                    self.define_memoized_function(&mangled_name, func_name, params, return_type, body, &nonlocal_vars, is_lru_cache, maxsize)?;
                } else {
                    self.define_function(&mangled_name, func_name, params, return_type, body, &nonlocal_vars)?;
                }
            }
        }
        // Second pass: define nested functions in compound statements
        for stmt in stmts {
            match stmt {
                Stmt::Function { body, .. } => {
                    self.define_all_functions(body)?;
                }
                Stmt::If { body, else_body, .. } => {
                    self.define_all_functions(body)?;
                    if let Some(else_stmts) = else_body {
                        self.define_all_functions(else_stmts)?;
                    }
                }
                Stmt::While { body, .. } => {
                    self.define_all_functions(body)?;
                }
                Stmt::For { body, .. } => {
                    self.define_all_functions(body)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Define a function (generate body)
    pub(crate) fn define_function(
        &mut self,
        mangled_name: &str,
        original_name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
        body: &[Stmt],
        _nonlocal_vars_param: &[String],
    ) -> Result<(), String> {
        // Save variables from previous function scope
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_loop_stack = std::mem::take(&mut self.loop_stack);
        let saved_list_vars = std::mem::take(&mut self.list_vars);
        let saved_bigint_vars = std::mem::take(&mut self.bigint_vars);
        let saved_var_types = std::mem::take(&mut self.var_types);
        let saved_current_function = self.current_function.clone();
        // Use original (unmangled) name for escape analysis
        self.current_function = Some(original_name.to_string());

        let func = self.functions.get(mangled_name).copied().unwrap();
        
        // PERFORMANCE OPTIMIZATION: Add inlining attributes for small functions
        // Functions with < 10 statements and < 3 parameters are good candidates for inlining
        // This reduces function call overhead significantly (20-40% for recursive benchmarks)
        if body.len() < 10 && params.len() < 3 {
            let always_inline_attr = self.context.create_string_attribute("alwaysinline", "");
            func.add_attribute(inkwell::attributes::AttributeLoc::Function, always_inline_attr);
        } else if body.len() < 5 {
            // Even smaller functions always benefit from inlining
            let always_inline_attr = self.context.create_string_attribute("alwaysinline", "");
            func.add_attribute(inkwell::attributes::AttributeLoc::Function, always_inline_attr);
        }

        // PERFORMANCE OPTIMIZATION: Add purity attributes for pure functions
        // Pure functions have no side effects and can be optimized aggressively
        if is_pure_function(body, params) {
            let readonly_attr = self.context.create_string_attribute("readonly", "");
            let willreturn_attr = self.context.create_string_attribute("willreturn", "");
            func.add_attribute(inkwell::attributes::AttributeLoc::Function, readonly_attr);
            func.add_attribute(inkwell::attributes::AttributeLoc::Function, willreturn_attr);
        }

        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Get closure info for this function
        let closure_info = self.closure_analyzer.get_closure_info(original_name);
        let nonlocal_vars: Vec<String> = closure_info
            .map(|info| info.nonlocal_vars.iter().cloned().collect())
            .unwrap_or_default();

        // Get variables that need closure cells (captured by nested functions)
        let _captured_vars: Vec<String> = self.closure_analyzer.get_closure_cells_to_create(original_name);

        // Set up parameters with alloca
        let num_regular_params = params.len();
        for (i, param) in params.iter().enumerate() {
            let param_value = func.get_nth_param(i as u32).unwrap();

            // Check escape analysis for this parameter
            let _can_stack_alloc = self.escape_analyzer.can_stack_allocate(original_name, &param.name);

            // Determine if parameter is a reference type (pointer)
            let is_ref_type = param_value.is_pointer_value();

            // Mark parameter as reference type in escape analyzer
            self.escape_analyzer.set_reference_type(original_name, &param.name, is_ref_type);

            // Always allocate on stack for now (escape analysis informs optimization decisions)
            // In a more advanced implementation, we might skip alloca for non-escaping params
            let alloca =
                self.builder.build_alloca(param_value.get_type(), &param.name).expect("alloca");
            self.builder.build_store(alloca, param_value).expect("store");
            // Determine VarType from the type annotation if present, otherwise from the LLVM parameter type
            // This ensures correct type handling even if there's a mismatch in the function signature
            let var_type = if let Some(ref ty) = param.type_ann {
                // Use type annotation to determine VarType
                match ty {
                    Type::F32 | Type::F64 => VarType::Float,
                    Type::Bool => VarType::Bool,
                    Type::Bytes => VarType::Bytes,
                    Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Class(_) | Type::Instance(_)
                    | Type::Fn(_, _) | Type::Optional(_) | Type::Chan(_) | Type::WaitGroup
                    | Type::Future(_) => VarType::Pointer,
                    _ => VarType::Int,
                }
            } else if param_value.is_pointer_value() {
                VarType::Pointer
            } else if param_value.is_float_value() {
                VarType::Float
            } else {
                VarType::Int
            };
            self.variables.insert(param.name.clone(), VarInfo::new_stack(alloca, var_type));

            // Store the parameter's type annotation in var_types for type inference
            if let Some(ref ty) = param.type_ann {
                self.var_types.insert(param.name.clone(), ty.clone());
            }

            // If parameter is a pointer type, mark it as a list or BigInt for indexing purposes
            // This is needed because list/BigInt parameters passed from callers are pointers
            if param_value.is_pointer_value() {
                // Check if it's a BigInt parameter based on type annotation or inferred type
                let is_bigint_param = matches!(param.type_ann, Some(Type::BigInt))
                    || matches!(self.var_types.get(&param.name), Some(Type::BigInt));
                if is_bigint_param {
                    self.bigint_vars.insert(param.name.clone());
                } else {
                    self.list_vars.insert(param.name.clone());
                }
            }
        }

        // Set up closure cell parameters (hidden parameters after regular params)
        // These are for nonlocal variables from enclosing scope
        if !nonlocal_vars.is_empty() {
            for (i, var_name) in nonlocal_vars.iter().enumerate() {
                let cell_param = func.get_nth_param((num_regular_params + i) as u32).unwrap();
                let cell_ptr = cell_param.into_pointer_value();
                
                // Get the value pointer inside the cell
                let i64_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let value_ptr = crate::codegen::closure_cells::get_closure_cell_value(
                    self.context, &self.module, &self.builder,
                    cell_ptr, i64_ptr_type
                ).unwrap_or(cell_ptr);
                
                // Store the closure cell pointer and value pointer
                self.closure_cells.insert(var_name.clone(), crate::codegen::state::ClosureCellInfo {
                    cell_ptr,
                    value_ptr,
                    var_type: VarType::Int,
                });
                // Create a variable entry that points to the closure cell
                self.variables.insert(var_name.clone(), VarInfo::new_closure_cell(
                    cell_ptr, VarType::Int, value_ptr
                ));
            }
        }

        // Note: Closure cells for captured variables are created inline when the variable is assigned
        // This is handled in the assignment codegen when it detects a variable is captured

        // Generate body using escape analysis and closure analysis
        // Create a single state for all statements to preserve closure_cells across statements
        let mut state = crate::codegen::state::CodeGenState::with_closure_analysis(
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
            &mut self.bigint_vars,
            &mut self.var_types,
            &mut self.escape_analyzer,
            original_name,
            &self.closure_analyzer,
            &mut self.closure_cells,
        );
        state.current_class = self.current_class.clone();
        
        for stmt in body {
            crate::codegen::statements::generate_stmt_internal(&mut state, stmt)?;
        }

        // Mark function as containing BigInt if it has BigInt variables
        // mem2reg is safe for pointer-typed allocas; no need for optnone
        if !self.bigint_vars.is_empty() {
            self.bigint_functions.insert(original_name.to_string());
        }

        // Generate ARC cleanup for local variables before return
        // Only generate cleanup if function doesn't already have a terminator
        // (i.e., no explicit return/break/continue at the end)
        let needs_cleanup_and_return = self.builder.get_insert_block().unwrap().get_terminator().is_none();
        
        if needs_cleanup_and_return {
            self.generate_arc_cleanup(original_name);
        }

        // Add implicit return if needed
        if needs_cleanup_and_return {
            // Check actual function return type from LLVM signature
            let func = self.module.get_function(mangled_name).unwrap();
            let return_type_llvm = func.get_type().get_return_type();

            // For methods without explicit return type or with None, use void return
            // For methods with pointer return type, return null pointer
            // For methods with scalar return type, return zero
            match return_type {
                Some(Type::None) => {
                    self.ir_builder.build_return(&self.builder, None);
                }
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
                // For pointer return types (str, list, object, etc.), return null pointer
                Some(Type::Str) | Some(Type::Bytes) | Some(Type::List(_)) | Some(Type::Dict(_, _))
                | Some(Type::Class(_)) | Some(Type::Instance(_)) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.context.ptr_type(inkwell::AddressSpace::default()).const_null()));
                }
                // For Infer type (unannotated methods), return null pointer
                Some(Type::Infer) => {
                    self.ir_builder
                        .build_return(&self.builder, Some(&self.context.ptr_type(inkwell::AddressSpace::default()).const_null()));
                }
                // For functions with no explicit return type annotation,
                // check LLVM signature: main() returns i64, others return void
                None => {
                    if return_type_llvm.is_some() {
                        // Function has non-void return type (e.g., main without annotation)
                        self.ir_builder
                            .build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
                    } else {
                        self.ir_builder.build_return(&self.builder, None);
                    }
                }
                _ => {
                    self.ir_builder.build_return(&self.builder, None);
                }
            }
        }

        // Restore variables for next function
        self.variables = saved_variables;
        self.loop_stack = saved_loop_stack;
        self.list_vars = saved_list_vars;
        self.bigint_vars = saved_bigint_vars;
        self.var_types = saved_var_types;
        self.current_function = saved_current_function;

        Ok(())
    }

    /// Define a memoized function (with @lru_cache or @cache decorator)
    pub(crate) fn define_memoized_function(
        &mut self,
        mangled_name: &str,
        original_name: &str,
        params: &[crate::ast::Param],
        return_type: &Option<Type>,
        body: &[Stmt],
        nonlocal_vars_param: &[String],
        is_lru: bool,
        _maxsize: i64,
    ) -> Result<(), String> {
        use crate::codegen::runtime::memoization;

        // Declare memoization runtime functions
        let memo_funcs = memoization::declare_memoization_functions(self.context, &mut self.module)
            .map_err(|e| format!("Failed to declare memoization functions: {}", e))?;

        // Create global cache for this function
        let cache_global = memoization::create_cache_global(self.context, &mut self.module, original_name, is_lru);
        
        // Store cache global for later use
        self.memoized_functions.insert(original_name.to_string(), cache_global);

        // For now, generate normal function body
        // The cache lookup/insert will be added at call sites
        // This is a simplified implementation - full version would:
        // 1. Create wrapper function that does cache lookup
        // 2. Rename original to __func_body
        // 3. Generate cache miss path that calls __func_body
        
        self.define_function(mangled_name, original_name, params, return_type, body, nonlocal_vars_param)
    }

    /// Check whether any of the given statements contain a direct or indirect call to `main()`.
    /// This is used to decide whether to emit an explicit `__user_main` call in the wrapper.
    fn stmts_call_main(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                // Direct call: main()
                Stmt::Expr(expr) => {
                    if let Expr::Call { func, .. } = expr {
                        if let Expr::Ident(name, _) = func.as_ref() {
                            if name == "main" {
                                return true;
                            }
                        }
                    }
                }
                // if __name__ == "__main__": main()  (or any if-block containing a main() call)
                Stmt::If { body, else_body, .. } => {
                    if Self::stmts_call_main(body) {
                        return true;
                    }
                    if let Some(else_stmts) = else_body {
                        if Self::stmts_call_main(else_stmts) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Generate main function handling top-level statements
    pub(crate) fn generate_main_with_statements(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        let main_type = self.context.i64_type().fn_type(&[], false);

        // Check if user defined main and save it
        let user_main_func = self.functions.remove("main");
        let has_user_main = user_main_func.is_some();

        // Detect whether the module-level statements already call main() explicitly
        // (e.g. via `if __name__ == "__main__": main()` or a bare `main()` call).
        // If they do, we must NOT emit an extra call at wrapper exit — that would double-execute.
        let module_calls_main = Self::stmts_call_main(stmts);

        // Generate viper_init function that only initializes __name__
        let init_type = self.context.void_type().fn_type(&[], false);
        let init_func = self.module.add_function("viper_init", init_type, None);
        let init_entry = self.context.append_basic_block(init_func, "entry");
        self.builder.position_at_end(init_entry);

        // Initialize __name__ builtin
        self.initialize_name_builtin()?;

        // viper_init only initializes __name__, no module-level statements
        self.ir_builder.build_return(&self.builder, None);

        // Generate wrapper main function
        let wrapper_main = self.module.add_function("main", main_type, None);
        let entry = self.context.append_basic_block(wrapper_main, "entry");
        self.builder.position_at_end(entry);

        // Call viper_init first
        let _ = self.builder.build_call(init_func, &[], "call_init");

        // Add user's main back to functions map as __user_main so calls to main() are redirected
        if let Some(user_main) = user_main_func {
            self.functions.insert("__user_main".to_string(), user_main);
        }

        // Generate module-level statements (calls to main() will be redirected to __user_main)
        for stmt in stmts {
            crate::codegen::statements::generate_stmt(
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
                &mut self.bigint_vars,
                &mut self.var_types,
                stmt,
            )?;
        }

        // Generate ARC cleanup for module-level variables
        self.generate_arc_cleanup("__module_level__");

        // Only emit an explicit __user_main call when the module-level statements did NOT
        // already call main() themselves. This handles the implicit-main pattern:
        //   def main(): ...
        //   # (no call at module level)
        // Without this, programs that use `if __name__ == "__main__": main()` would execute
        // main() twice — once through that statement and once through the explicit call here.
        if has_user_main && !module_calls_main {
            if let Some(user_main) = self.functions.get("__user_main") {
                let user_main_func = *user_main;
                let _ = self.builder.build_call(user_main_func, &[], "call_user_main");
            }
        }

        // Return 0
        self.ir_builder.build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));

        Ok(())
    }

    /// Declare all functions recursively (including nested functions)
    pub(crate) fn declare_all_functions(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Function { name, params, return_type, body, .. } => {
                    // Get closure info for this function
                    let closure_info = self.closure_analyzer.get_closure_info(name);
                    let nonlocal_vars: Vec<String> = closure_info
                        .map(|info| info.nonlocal_vars.iter().cloned().collect())
                        .unwrap_or_default();

                    // Rename user's main to __user_main so we can wrap it
                    let func_name = if name == "main" { "__user_main" } else { name };

                    // Declare with closure cell parameters
                    crate::codegen::functions::declare_function_with_closure(
                        self.context,
                        &mut self.module,
                        &self.type_mapper,
                        &mut self.functions,
                        func_name,
                        params,
                        return_type,
                        Some(body),
                        &nonlocal_vars,
                    )?;
                    // Recursively declare nested functions in the body
                    self.declare_all_functions(body)?;
                }
                Stmt::Extern { name, params, return_type, .. } => {
                    let param_types: Vec<Type> =
                        params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::I64)).collect();
                    use crate::utils::mangle_function_name;
                    let mangled_name = mangle_function_name(name, &param_types);
                    crate::codegen::functions::declare_function(
                        self.context,
                        &mut self.module,
                        &self.type_mapper,
                        &mut self.functions,
                        &mangled_name,
                        params,
                        return_type,
                        None,
                    )?;
                }
                // Recursively search for nested functions in compound statements
                Stmt::If { body, else_body, .. } => {
                    self.declare_all_functions(body)?;
                    if let Some(else_stmts) = else_body {
                        self.declare_all_functions(else_stmts)?;
                    }
                }
                Stmt::While { body, .. } => {
                    self.declare_all_functions(body)?;
                }
                Stmt::For { body, .. } => {
                    self.declare_all_functions(body)?;
                }
                Stmt::Task { call, .. } => {
                    // Check if call contains nested function definitions
                    if let Expr::Call { args, .. } = call {
                        // Functions could be defined in lambda args
                        for arg in args {
                            self.declare_functions_in_expr(&arg)?;
                        }
                    }
                }
                Stmt::Class { name: class_name, body, .. } => {
                    // Declare class methods
                    for stmt in body {
                        if let Stmt::Function { name: method_name, params, return_type, .. } = stmt {
                            // Use simple mangled name format for methods
                            let mangled_name = format!("__method_{}_{}", class_name, method_name);

                            // For instance methods, self should be a pointer type
                            // We need to create modified params with self as pointer
                            let mut method_params = params.clone();
                            if !params.is_empty() && params[0].name == "self" {
                                // First param is self - it should be a pointer to the class instance
                                // For now, use a special marker that will be treated as pointer
                                if method_params[0].type_ann.is_none() {
                                    // self without type annotation should be treated as pointer
                                    method_params[0].type_ann = Some(Type::Class(class_name.clone()));
                                }
                            }

                            // For methods without return type annotation, use pointer type as default
                            // This allows the method to return any reference type (str, list, object, etc.)
                            // Exception: __init__ methods should have None (void) return type
                            let method_return_type = if method_name == "__init__" {
                                Some(Type::None)
                            } else if return_type.as_ref().map_or(true, |t| matches!(t, Type::None)) {
                                Some(Type::Str)
                            } else {
                                return_type.clone()
                            };

                            // Just create a forward declaration without body-based name mangling
                            crate::codegen::functions::declare_function_simple(
                                self.context,
                                &mut self.module,
                                &self.type_mapper,
                                &mut self.functions,
                                &mangled_name,
                                &method_params,
                                &method_return_type,
                            )?;
                        }
                    }
                    // Also recursively declare any nested functions in the class body
                    self.declare_all_functions(body)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Declare functions found in expressions (e.g., lambdas)
    fn declare_functions_in_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Lambda { body, .. } => {
                // Lambda body is an expression, check if it contains nested functions
                self.declare_functions_in_expr(body)?;
            }
            Expr::Call { args, func, .. } => {
                for arg in args {
                    self.declare_functions_in_expr(arg)?;
                }
                self.declare_functions_in_expr(func)?;
            }
            Expr::BinOp { left, right, .. } => {
                self.declare_functions_in_expr(left)?;
                self.declare_functions_in_expr(right)?;
            }
            Expr::UnaryOp { operand, .. } => {
                self.declare_functions_in_expr(operand)?;
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.declare_functions_in_expr(condition)?;
                self.declare_functions_in_expr(then_expr)?;
                self.declare_functions_in_expr(else_expr)?;
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.declare_functions_in_expr(elem)?;
                }
            }
            Expr::Attribute { obj, .. } => {
                self.declare_functions_in_expr(obj)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Generate ARC cleanup code for local variables at function exit
    pub(crate) fn generate_arc_cleanup(&mut self, function_name: &str) {
        // Find variables that need cleanup
        let vars_needing_cleanup = self.escape_analyzer.get_vars_needing_cleanup(function_name);

        // OPTIMIZATION 3: Batch Release - Group releases by thread-local vs shared
        let mut local_vars: Vec<inkwell::values::PointerValue> = Vec::new();
        let mut shared_vars: Vec<inkwell::values::PointerValue> = Vec::new();

        for var_name in vars_needing_cleanup {
            if let Some(var_info) = self.variables.get(var_name) {
                if let crate::codegen::variables::VarStorage::Stack(stack_ptr) = &var_info.storage {
                    // Load the pointer value
                    let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                    let value = self.builder.build_load(ptr_type, *stack_ptr, "load_var").unwrap();
                    if let inkwell::values::BasicValueEnum::PointerValue(ptr_val) = value {
                        // Check if shared across threads
                        if self.escape_analyzer.is_thread_shared(function_name, var_name) {
                            shared_vars.push(ptr_val);
                        } else {
                            local_vars.push(ptr_val);
                        }
                    }
                }
            }
        }

        // Use batch release for local variables (more efficient)
        if !local_vars.is_empty() {
            // For small counts, individual releases are fine
            // For large counts, could use vp_release_batch_local
            if local_vars.len() >= 4 {
                // Batch release for 4+ variables
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let array_type = ptr_type.array_type(local_vars.len() as u32);
                let array_alloca = self.builder.build_alloca(array_type, "batch_ptrs").unwrap();

                // Store each pointer in the array
                for (i, ptr) in local_vars.iter().enumerate() {
                    let gep = unsafe {
                        self.builder.build_in_bounds_gep(
                            array_type,
                            array_alloca,
                            &[self.context.i32_type().const_zero(), self.context.i32_type().const_int(i as u64, false)],
                            "ptr_gep"
                        )
                    }.unwrap();
                    self.builder.build_store(gep, *ptr).unwrap();
                }

                // Call batch release
                if let Some(batch_func) = self.module.get_function("vp_release_batch_local") {
                    let array_ptr = self.builder.build_pointer_cast(
                        array_alloca,
                        self.context.ptr_type(inkwell::AddressSpace::default()),
                        "array_cast"
                    ).unwrap();
                    self.builder.build_call(
                        batch_func,
                        &[array_ptr.into(), self.context.i32_type().const_int(local_vars.len() as u64, false).into()],
                        "batch_release"
                    ).unwrap();
                }
            } else {
                // Individual releases for small counts
                for ptr_val in local_vars {
                    if let Some(release_func) = self.module.get_function("vp_release_local") {
                        self.builder.build_call(release_func, &[ptr_val.into()], "release_var").unwrap();
                    }
                }
            }
        }

        // Shared variables need individual releases (different destructor signature)
        let null_ptr = self.context.ptr_type(inkwell::AddressSpace::default()).const_null();
        for ptr_val in shared_vars {
            if let Some(release_func) = self.module.get_function("vp_release") {
                self.builder.build_call(release_func, &[ptr_val.into(), null_ptr.into()], "release_var").unwrap();
            }
        }
    }
}

/// Check if a function is pure (no side effects)
/// Pure functions:
/// - Don't have side effects (no print, no I/O, no mutations of globals)
/// - Only return a value based on their parameters
/// This allows LLVM to apply aggressive optimizations like CSE and DCE
fn is_pure_function(body: &[Stmt], _params: &[crate::ast::Param]) -> bool {
    for stmt in body {
        if !is_pure_statement(stmt) {
            return false;
        }
    }
    true
}

/// Check if a statement is pure (no side effects)
fn is_pure_statement(stmt: &Stmt) -> bool {
    match stmt {
        // Pure statements
        Stmt::Declare { value, .. } => value.as_ref().map_or(true, is_pure_expr),
        Stmt::Assign { value, .. } => is_pure_expr(value),
        
        // Control flow - check nested statements
        Stmt::If { body, else_body, condition, .. } => {
            is_pure_expr(condition)
                && body.iter().all(is_pure_statement)
                && else_body.as_ref().map_or(true, |eb| eb.iter().all(is_pure_statement))
        }
        Stmt::While { condition, body, .. } => {
            is_pure_expr(condition) && body.iter().all(is_pure_statement)
        }
        Stmt::For { iter, body, .. } => {
            is_pure_expr(iter) && body.iter().all(is_pure_statement)
        }
        
        // Return is pure if the value is pure
        Stmt::Return { value, .. } => value.as_ref().map_or(true, is_pure_expr),
        
        // Expressions are pure if the expression is pure
        Stmt::Expr(expr) => is_pure_expr(expr),
        
        // These statements have side effects
        Stmt::AugAssign { .. } => false,  // Mutation
        
        // Functions and classes are declarations, not statements in function body
        Stmt::Function { .. } => false,  // Nested function definition is impure
        Stmt::Class { .. } => false,
        
        // Import statements
        Stmt::Import { .. } => false,
        Stmt::FromImport { .. } => false,
        
        // Break/Continue are control flow, not side effects
        Stmt::Break(_) => true,
        Stmt::Continue(_) => true,
        
        // Pass is pure
        Stmt::Pass(_) => true,
        
        // External function calls are impure (unknown side effects)
        Stmt::Extern { .. } => false,
        
        // Match/Select - check nested statements
        Stmt::Match { cases, subject, .. } => {
            is_pure_expr(subject) && cases.iter().all(|c| c.body.iter().all(is_pure_statement))
        }
        Stmt::Select { cases, .. } => {
            cases.iter().all(|c| c.body.iter().all(is_pure_statement))
        }
        
        // Concurrency primitives have side effects
        Stmt::Task { .. } => false,
        Stmt::Sync { .. } => false,
        Stmt::Chan { .. } => false,
        Stmt::Send { .. } => false,
        Stmt::Recv { .. } => false,
        Stmt::WaitGroup { .. } => false,
        Stmt::WgAdd { .. } => false,
        Stmt::WgDone { .. } => false,
        
        // Exception handling
        Stmt::Try { body, handlers, .. } => {
            body.iter().all(is_pure_statement)
                && handlers.iter().all(|h| h.body.iter().all(is_pure_statement))
        }
        Stmt::Raise { .. } => false,
        Stmt::Assert { .. } => false,
        
        // Other statements
        Stmt::Global { .. } => false,
        Stmt::Nonlocal { .. } => false,
        Stmt::Const { .. } => false,
        Stmt::Struct { .. } => false,
        
        // Additional concurrency and misc statements
        Stmt::WgWait { .. } => false,
        Stmt::TypeAlias { .. } => false,
        Stmt::Delete { .. } => false,
        Stmt::With { .. } => false,
        Stmt::Yield { .. } => false,
    }
}

/// Check if an expression is pure (no side effects)
fn is_pure_expr(expr: &Expr) -> bool {
    match expr {
        // Literals are pure
        Expr::Int(_, _) | Expr::Float(_, _) | Expr::Bool(_, _) | Expr::Str(_, _)
        | Expr::BigInt(_, _) | Expr::None(_) | Expr::Bytes(_, _) | Expr::FString(_, _) => true,
        
        // Identifiers are pure
        Expr::Ident(_, _) => true,
        
        // Binary/unary ops are pure if operands are pure
        Expr::BinOp { left, right, .. } => is_pure_expr(left) && is_pure_expr(right),
        Expr::UnaryOp { operand, .. } => is_pure_expr(operand),
        
        // Index/Attribute access is pure if object is pure
        Expr::Index { obj, index, .. } => is_pure_expr(obj) && is_pure_expr(index),
        Expr::Attribute { obj, .. } => is_pure_expr(obj),
        Expr::Slice { obj, start, end, step, .. } => {
            is_pure_expr(obj)
                && start.as_ref().map_or(true, |s| is_pure_expr(s))
                && end.as_ref().map_or(true, |e| is_pure_expr(e))
                && step.as_ref().map_or(true, |s| is_pure_expr(s))
        }
        
        // Calls are impure unless they're known pure builtins
        Expr::Call { func, args, .. } => {
            // Check if it's a pure builtin
            let is_pure_builtin = if let Expr::Ident(name, _) = func.as_ref() {
                matches!(name.as_str(), 
                    "len" | "abs" | "min" | "max" | "sum" | "range" |
                    "str" | "int" | "float" | "bool" | "repr" |
                    "ord" | "chr" | "hex" | "bin" | "oct" |
                    "hash" | "id" | "type" | "isinstance"
                )
            } else {
                false
            };
            
            is_pure_builtin && args.iter().all(is_pure_expr)
        }
        
        // Lambda is impure (function definition)
        Expr::Lambda { .. } => false,
        
        // Collections are pure if elements are pure
        Expr::List { elements, .. } => elements.iter().all(is_pure_expr),
        Expr::Tuple { elements, .. } => elements.iter().all(is_pure_expr),
        Expr::Dict { pairs, .. } => {
            pairs.iter().all(|(k, v)| is_pure_expr(k) && is_pure_expr(v))
        }
        Expr::Array { elements, .. } => elements.iter().all(is_pure_expr),
        
        // Comprehensions are impure (contain loops)
        Expr::ListComprehension { .. } => false,
        
        // Conditional is pure if all parts are pure
        Expr::Conditional { condition, then_expr, else_expr, .. } => {
            is_pure_expr(condition) && is_pure_expr(then_expr) && is_pure_expr(else_expr)
        }
        
        // Await is impure (side effect of async)
        Expr::Await { .. } => false,
        
        // Assignment expression is impure (mutation)
        Expr::AssignmentExpr { .. } => false,
        
        // Super call is impure
        Expr::Super(_) => false,
    }
}

/// Check if an expression is definitely impure
fn is_impure_expr(expr: &Expr) -> bool {
    match expr {
        // Calls to impure functions
        Expr::Call { func, .. } => {
            if let Expr::Ident(name, _) = func.as_ref() {
                matches!(name.as_str(),
                    "print" | "input" | "exit" | "open" | "read" | "write" |
                    "append" | "pop" | "remove" | "clear" | "sort" | "reverse" |
                    "send" | "recv" | "done" | "wait"
                )
            } else {
                false
            }
        }
        _ => false,
    }
}
