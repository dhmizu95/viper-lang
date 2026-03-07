//! Main code generator that translates AST to LLVM IR

use crate::ast::{Expr, Module, Stmt, Type};
use crate::utils::mangle_function_name;
use inkwell::context::Context;
use inkwell::values::{BasicValue, FunctionValue, GlobalValue};
use std::collections::{HashMap, HashSet};

use crate::codegen::builder::IRBuilder;
use crate::codegen::types::TypeMapper;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};
use crate::semantic::escape_analysis::EscapeAnalyzer;
use crate::semantic::closure_analysis::ClosureAnalyzer;
use crate::utils::mangling::mangle_function_name_with_closure;

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
    list_vars: HashSet<String>,
    dict_vars: HashSet<String>,
    bool_list_vars: HashSet<String>,
    bigint_vars: HashSet<String>,
    var_types: HashMap<String, Type>,
    /// Functions that contain BigInt variables (need special optimization handling)
    bigint_functions: HashSet<String>,
    escape_analyzer: EscapeAnalyzer,
    closure_analyzer: ClosureAnalyzer,
    /// Variables that are captured by nested functions
    closure_cells: HashMap<String, crate::codegen::state::ClosureCellInfo<'ctx>>,
    /// Variables captured by this function's nested functions
    captured_vars: HashSet<String>,
    current_function: Option<String>,
    current_class: Option<String>,  // Current class context for super() and methods
    in_classmethod: bool,  // True when generating code for a @classmethod
    /// Module name for __name__ builtin
    module_name: String,
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
            list_vars: HashSet::new(),
            dict_vars: HashSet::new(),
            bool_list_vars: HashSet::new(),
            bigint_vars: HashSet::new(),
            var_types: HashMap::new(),
            bigint_functions: HashSet::new(),
            escape_analyzer: EscapeAnalyzer::new(),
            closure_analyzer: ClosureAnalyzer::new(),
            closure_cells: HashMap::new(),
            captured_vars: HashSet::new(),
            current_function: None,
            current_class: None,
            in_classmethod: false,
            module_name: module_name.to_string(),
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {
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

        // Note: Closure cell runtime is generated on-demand when nested functions with nonlocal are used
        // For now, we skip generating it to avoid linking issues with simple programs
        // crate::codegen::runtime::closure_cells::declare_closure_cell_functions(self.context, &self.module)?;

        // First pass: declare all functions (including class methods and nested functions)
        self.declare_all_functions(&module.statements)?;

        // Generate class definitions (defines class methods)
        self.generate_classes(&module.statements)?;

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
                            &mut self.bigint_vars,
                            &mut self.var_types,
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
                                &mut self.bigint_vars,
                                &mut self.var_types,
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

    /// Define all functions recursively (including nested functions)
    fn define_all_functions(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        // First pass: declare all functions at this level with closure cell parameters
        for stmt in stmts {
            if let Stmt::Function { name, params, return_type, body, .. } = stmt {
                // Get closure info to determine nonlocal variables
                let closure_info = self.closure_analyzer.get_closure_info(name);
                let nonlocal_vars: Vec<String> = closure_info
                    .map(|info| info.nonlocal_vars.iter().cloned().collect())
                    .unwrap_or_default();

                // Compute mangled name including closure info
                // Rename user's main to __user_main to match declaration
                let func_name = if name == "main" { "__user_main" } else { name };
                use crate::codegen::functions::infer_param_types_from_body;
                let param_types = infer_param_types_from_body(params, body);
                let mangled_name = mangle_function_name_with_closure(func_name, &param_types, &nonlocal_vars);
                self.define_function(&mangled_name, func_name, params, return_type, body, &nonlocal_vars)?;
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
    fn define_function(
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
            // Determine VarType from the actual LLVM parameter type, not just the annotation
            // This handles cases where lists/channels are passed without explicit type annotations
            let var_type = if param_value.is_pointer_value() {
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
                // Store the closure cell pointer
                self.closure_cells.insert(var_name.clone(), crate::codegen::state::ClosureCellInfo {
                    cell_ptr: cell_param.into_pointer_value(),
                    value_ptr: cell_param.into_pointer_value(),
                    var_type: VarType::Int,
                });
                // Create a variable entry that points to the closure cell
                let i64_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let value_ptr = crate::codegen::closure_cells::get_closure_cell_value(
                    self.context, &self.module, &self.builder,
                    cell_param.into_pointer_value(), i64_ptr_type
                ).unwrap_or(cell_param.into_pointer_value());
                self.variables.insert(var_name.clone(), VarInfo::new_closure_cell(
                    cell_param.into_pointer_value(), VarType::Int, value_ptr
                ));
            }
        }

        // Note: Closure cells for captured variables are created inline when the variable is assigned
        // This is handled in the assignment codegen when it detects a variable is captured

        // Generate body using escape analysis and closure analysis
        for stmt in body {
            crate::codegen::statements::generate_stmt_with_closure(
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
                &mut self.escape_analyzer,
                original_name,
                &self.closure_analyzer,
                self.current_class.as_deref(),
            )?;
        }

        // Mark function as containing BigInt if it has BigInt variables
        // These functions need special optimization handling (no mem2reg)
        if !self.bigint_vars.is_empty() {
            self.bigint_functions.insert(original_name.to_string());
            // Apply optnone attribute to prevent mem2reg and other optimizations
            // that could break ARC retain/release semantics for BigInt
            func.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                self.context.create_string_attribute("optnone", ""),
            );
        }

        // Generate ARC cleanup for local variables before return
        // Only generate cleanup if function doesn't already have a terminator
        // (i.e., no explicit return/break/continue at the end)
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            self.generate_arc_cleanup(original_name);
        }

        // Add implicit return if needed
        if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
            // Check actual function return type from LLVM signature
            let func = self.module.get_function(mangled_name).unwrap();
            let return_type_llvm = func.get_type().get_return_type();

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
                // For functions with no explicit return type (None),
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
    fn generate_main_with_statements(&mut self, stmts: &[Stmt]) -> Result<(), String> {
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
            crate::codegen::statements::generate_stmt_with_escape(
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
                &mut self.escape_analyzer,
                "__module_level__",
                None,  // No class context for module-level code
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
    fn declare_all_functions(&mut self, stmts: &[Stmt]) -> Result<(), String> {
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
                        if let Stmt::Function { name: method_name, params, return_type, body: inner_body, .. } = stmt {
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
                            
                            // Just create a forward declaration without body-based name mangling
                            crate::codegen::functions::declare_function_simple(
                                self.context,
                                &mut self.module,
                                &self.type_mapper,
                                &mut self.functions,
                                &mangled_name,
                                &method_params,
                                return_type,
                                &Some(inner_body.clone()),
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
    fn generate_arc_cleanup(&mut self, function_name: &str) {
        // Find variables that need cleanup
        let vars_needing_cleanup = self.escape_analyzer.get_vars_needing_cleanup(function_name);
        
        // OPTIMIZATION 3: Batch Release - Group releases by thread-local vs shared
        let mut local_vars: Vec<inkwell::values::PointerValue> = Vec::new();
        let mut shared_vars: Vec<inkwell::values::PointerValue> = Vec::new();

        for var_name in vars_needing_cleanup {
            if let Some(var_info) = self.variables.get(var_name) {
                if let crate::codegen::variables::VarStorage::Stack(stack_ptr) = &var_info.storage {
                    // CRITICAL: Only release pointer-typed variables.
                    // Loading an i64 alloca as ptr_type is UB and causes segfaults.
                    if var_info.var_type != crate::codegen::variables::VarType::Pointer {
                        continue;
                    }
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
                        ptr_type.ptr_type(inkwell::AddressSpace::default()),
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

    /// Create a global constant from a literal expression
    fn create_global_constant(&mut self, name: &str, value: &Expr) -> Result<(), String> {
        let val = match value {
            Expr::Int(n, _) => self.ir_builder.i64_const(*n).as_basic_value_enum(),
            Expr::Float(n, _) => self.ir_builder.f64_const(*n).as_basic_value_enum(),
            Expr::Bool(b, _) => self.ir_builder.bool_const(*b).as_basic_value_enum(),
            Expr::Str(s, _) => self.ir_builder.string_const(&self.module, s).as_basic_value_enum(),
            Expr::Bytes(b, _) => self.ir_builder.bytes_const(&self.module, b).as_basic_value_enum(),
            Expr::None(_) => self.ir_builder.i64_const(0).as_basic_value_enum(),
            _ => return Err(format!("Cannot create global constant from non-literal expression")),
        };

        let global = self.module.add_global(val.get_type(), None, name);
        global.set_constant(true);
        global.set_initializer(&val);
        global.set_unnamed_addr(false);

        self.global_constants.insert(name.to_string(), global);
        Ok(())
    }

    /// Check if an expression can be used as a simple global initializer
    /// Complex types (tuples, lists, dicts, arrays) require runtime allocation
    fn is_simple_initializer_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Int(..)
            | Expr::Float(..)
            | Expr::Bool(..)
            | Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::None(..) => true,
            Expr::UnaryOp { operand, .. } => {
                matches!(operand.as_ref(), Expr::Int(..) | Expr::Float(..))
            }
            _ => false,
        }
    }

    /// Get the generated LLVM module
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    /// Create a global string constant
    pub fn create_global_string(&mut self, s: &str) -> inkwell::values::PointerValue<'ctx> {
        let context = self.context;
        let string_global = self.module.add_global(
            context.i8_type().array_type((s.len() + 1) as u32),
            Some(inkwell::AddressSpace::default()),
            &format!(".str.{}", s.replace(" ", "_").replace("\n", "_n").replace("\"", "_q")),
        );
        string_global.set_constant(true);
        string_global.set_unnamed_addr(true);
        string_global.set_linkage(inkwell::module::Linkage::Private);

        let init_data: Vec<u8> = s.as_bytes().iter().copied().chain(std::iter::once(0)).collect();
        let init_array = context.i8_type().const_array(&init_data.iter()
            .map(|&b| context.i8_type().const_int(b as u64, false))
            .collect::<Vec<_>>());
        string_global.set_initializer(&init_array);

        // GlobalValue is already a pointer, cast it
        string_global.as_basic_value_enum().into_pointer_value()
    }

    /// Generate code for all class definitions in a module
    fn generate_classes(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        // First pass: collect all class metadata
        for stmt in stmts {
            if let Stmt::Class { name, bases, body, span: _, decorators: _, fields, methods } = stmt {
                let metadata = crate::codegen::oop::generate_class_metadata(
                    name, bases, body, fields, methods
                )?;
                crate::codegen::oop::with_class_registry_mut(|reg| {
                    reg.register_class(metadata);
                });
            }
        }

        // Calculate MRO for all classes
        crate::codegen::oop::with_class_registry_mut(|reg| {
            if let Err(e) = crate::codegen::oop::calculate_all_mros(reg) {
                eprintln!("Warning: Failed to calculate MRO: {}", e);
            }
        });

        // Second pass: generate class code and methods
        for stmt in stmts {
            if let Stmt::Class { name, bases: _, body, span: _, decorators: _, fields, methods } = stmt {
                self.generate_class_def(name, body, fields, methods)?;
            }
        }
        Ok(())
    }

    /// Generate code for a single class definition
    fn generate_class_def(
        &mut self,
        name: &str,
        body: &[Stmt],
        _fields: &[(String, Option<Type>, bool)],
        _methods: &[String],
    ) -> Result<(), String> {
        // Get class metadata from registry
        let metadata = crate::codegen::oop::with_class_registry(|reg| {
            reg.get_class(name).cloned()
        }).ok_or_else(|| format!("Class metadata not found for '{}'", name))?;
        
        let context = self.context;
        
        // Create class metadata struct type
        // ViperClass struct layout:
        // - name: i8*
        // - bases: void* (ViperClass**)
        // - base_count: i64
        // - methods: void* (ViperMethod*)
        // - method_count: i64
        // - instance_size: i64
        // - init: function pointer (void*)
        // - dealloc: function pointer (void*)
        let class_struct_type = context.struct_type(&[
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // name
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // bases
            context.i64_type().into(),  // base_count
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // methods
            context.i64_type().into(),  // method_count
            context.i64_type().into(),  // instance_size
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // init
            context.ptr_type(inkwell::AddressSpace::default()).into(),  // dealloc
        ], false);

        // Create class metadata global
        let class_global_name = format!("__viper_class_{}", name);
        let class_global = self.module.add_global(class_struct_type, None, &class_global_name);
        class_global.set_constant(false);
        class_global.set_unnamed_addr(true);

        // Create class name string
        let name_str = self.create_global_string(name);

        // Create initializer values
        let null_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();
        let base_count_val = context.i64_type().const_int(0, false);  // Will be updated with inheritance
        let method_count_val = context.i64_type().const_int(metadata.methods.len() as u64, false);
        let instance_size_val = context.i64_type().const_int(metadata.instance_size as u64, false);
        let init_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();
        let dealloc_ptr = context.ptr_type(inkwell::AddressSpace::default()).const_null();
        
        // Create initializer for class struct
        let class_init = class_struct_type.const_named_struct(&[
            name_str.as_basic_value_enum(),  // name
            null_ptr.as_basic_value_enum(),  // bases
            base_count_val.as_basic_value_enum(),  // base_count
            null_ptr.as_basic_value_enum(),  // methods
            method_count_val.as_basic_value_enum(),  // method_count
            instance_size_val.as_basic_value_enum(),  // instance_size
            init_ptr.as_basic_value_enum(),  // init
            dealloc_ptr.as_basic_value_enum(),  // dealloc
        ]);
        
        class_global.set_initializer(&class_init);
        
        // Generate method functions
        // Save current class context
        let saved_class = self.current_class.clone();
        self.current_class = Some(name.to_string());

        for stmt in body {
            if let Stmt::Function { name: method_name, params, return_type, body: method_body, decorators, .. } = stmt {
                // Check for staticmethod and classmethod decorators
                let is_static = decorators.iter().any(|d| d.name == "staticmethod");
                let is_class_method = decorators.iter().any(|d| d.name == "classmethod");

                // Generate mangled method name
                let mangled_name = format!("__method_{}_{}", name, method_name);

                // Set flag for classmethod
                let saved_classmethod = self.in_classmethod;
                self.in_classmethod = is_class_method;

                // For static methods, generate as regular function
                // For instance methods, the first param is 'self'
                let empty_nonlocal: Vec<String> = Vec::new();
                if is_static {
                    // Static method - no self parameter type injection needed
                    self.define_function(&mangled_name, method_name, params, return_type, method_body, &empty_nonlocal)?;
                } else {
                    // Instance method - already has self parameter in AST
                    // Inject the current class type for the 'self' parameter to aid inference
                    let mut typed_params = params.to_vec();
                    if let Some(first_param) = typed_params.first_mut() {
                        if first_param.name == "self" && first_param.type_ann.is_none() {
                            first_param.type_ann = Some(crate::ast::Type::Instance(name.to_string()));
                        }
                    }
                    self.define_function(&mangled_name, method_name, &typed_params, return_type, method_body, &empty_nonlocal)?;
                }

                // Restore classmethod flag
                self.in_classmethod = saved_classmethod;
            }
        }

        // Restore previous class context
        self.current_class = saved_class;
        
        Ok(())
    }

    /// Generate __name__ builtin constant
    /// For the main module, use "__main__"; for imported modules, use the module name
    fn generate_name_builtin(&mut self) -> Result<(), String> {
        // Create a global pointer variable for __name__ (will be initialized in viper_init)
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let global = self.module.add_global(ptr_type, None, "__name__");
        global.set_constant(false);
        global.set_initializer(&ptr_type.const_null());
        global.set_unnamed_addr(false);
        
        // Store the pointer type for correct loading later
        self.var_types.insert("__name__".to_string(), crate::ast::Type::Str);
        
        // Store in global_constants for lookup
        self.global_constants.insert("__name__".to_string(), global);
        
        Ok(())
    }

    /// Initialize __name__ builtin in viper_init
    fn initialize_name_builtin(&mut self) -> Result<(), String> {
        // For the main module, use "__main__" as the name
        // This allows if __name__ == "__main__" to work correctly
        let name_value = "__main__";
        
        // Create string constant for __name__
        let str_val = self.ir_builder.string_const(&self.module, name_value);
        let create_func = self
            .module
            .get_function("vp_str_create")
            .ok_or_else(|| "vp_str_create not declared".to_string())?;
        
        let result = self
            .ir_builder
            .build_call(
                &mut self.builder,
                create_func,
                &[str_val.into()],
                "__name__",
            )
            .ok_or_else(|| "Failed to create __name__ string".to_string())?;
        
        // Store the result in the __name__ global
        if let Some(global) = self.global_constants.get("__name__") {
            self.builder
                .build_store(global.as_pointer_value(), result)
                .map_err(|e| format!("Failed to store __name__: {:?}", e))?;
        }
        
        Ok(())
    }

    /// Get the list of functions containing BigInt variables
    /// These functions should skip mem2reg optimization
    pub fn bigint_functions(&self) -> &HashSet<String> {
        &self.bigint_functions
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
