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
            list_vars: HashSet::new(),
            dict_vars: HashSet::new(),
            bool_list_vars: HashSet::new(),
            bigint_vars: HashSet::new(),
            var_types: HashMap::new(),
            bigint_functions: HashSet::new(),
            escape_analyzer: EscapeAnalyzer::new(),
            current_function: None,
        }
    }

    /// Generate code for a complete module
    pub fn generate(&mut self, module: &Module) -> Result<(), String> {
        // Run escape analysis first
        self.escape_analyzer.analyze_module(module);

        // Initialize class registry for OOP
        crate::codegen::oop::init_class_registry();

        // Declare runtime functions first
        crate::codegen::runtime::declare_runtime_functions(self.context, &self.module)?;

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
        for stmt in stmts {
            if let Stmt::Function { name, params, return_type, body, .. } = stmt {
                // Compute mangled name using the same logic as declare_function
                use crate::codegen::functions::infer_param_types_from_body;
                let param_types = infer_param_types_from_body(params, body);
                let mangled_name = mangle_function_name(name, &param_types);
                self.define_function(&mangled_name, name, params, return_type, body)?;
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
    ) -> Result<(), String> {
        // Save variables from previous function scope
        let saved_variables = std::mem::take(&mut self.variables);
        let saved_loop_stack = std::mem::take(&mut self.loop_stack);
        let saved_list_vars = std::mem::take(&mut self.list_vars);
        let saved_bigint_vars = std::mem::take(&mut self.bigint_vars);
        let saved_current_function = self.current_function.clone();
        // Use original (unmangled) name for escape analysis
        self.current_function = Some(original_name.to_string());

        let func = self.functions.get(mangled_name).copied().unwrap();
        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        // Set up parameters with alloca
        // Use escape analysis to determine if parameter needs stack allocation
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

            // If parameter is a pointer type, mark it as a list for indexing purposes
            // This is needed because list parameters passed from callers are pointers
            if param_value.is_pointer_value() {
                self.list_vars.insert(param.name.clone());
            }
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
                &mut self.list_vars,
                &mut self.dict_vars,
                &mut self.bool_list_vars,
                &mut self.bigint_vars,
                stmt,
                &mut self.escape_analyzer,
                original_name,
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
                &mut self.list_vars,
                &mut self.dict_vars,
                &mut self.bool_list_vars,
                &mut self.bigint_vars,
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

            self.ir_builder.build_return(&self.builder, Some(&self.ir_builder.i64_const(0)));
        }

        Ok(())
    }

    /// Declare all functions recursively (including nested functions)
    fn declare_all_functions(&mut self, stmts: &[Stmt]) -> Result<(), String> {
        for stmt in stmts {
            match stmt {
                Stmt::Function { name, params, return_type, body, .. } => {
                    crate::codegen::functions::declare_function(
                        self.context,
                        &mut self.module,
                        &self.type_mapper,
                        &mut self.functions,
                        name,
                        params,
                        return_type,
                        Some(body),
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
                            
                            // Just create a forward declaration without body-based name mangling
                            crate::codegen::functions::declare_function_simple(
                                self.context,
                                &mut self.module,
                                &self.type_mapper,
                                &mut self.functions,
                                &mangled_name,
                                &method_params,
                                return_type,
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
        if vars_needing_cleanup.is_empty() {
            return;
        }

        let mut local_vars = Vec::new();
        let mut shared_vars = Vec::new();

        for var_name in vars_needing_cleanup {
            if let Some(var_info) = self.variables.get(var_name) {
                // If the variable might be shared across threads, it needs atomic release
                if self.escape_analyzer.is_thread_shared(function_name, var_name) {
                    shared_vars.push(var_info.clone());
                } else {
                    local_vars.push(var_info.clone());
                }
            }
        }

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let null_ptr = ptr_type.const_null();

        // Generate individual release calls for each variable
        // (batch release would be more efficient but requires runtime support)
        for var in &local_vars {
            if let crate::codegen::variables::VarStorage::Stack(stack_ptr) = &var.storage {
                let value = self.builder.build_load(ptr_type, *stack_ptr, "load_var").unwrap();
                if let inkwell::values::BasicValueEnum::PointerValue(ptr_val) = value {
                    if let Some(release_func) = self.module.get_function("vp_release_local") {
                        self.builder.build_call(release_func, &[ptr_val.into()], "release_var").unwrap();
                    }
                }
            }
        }

        for var in &shared_vars {
            if let crate::codegen::variables::VarStorage::Stack(stack_ptr) = &var.storage {
                let value = self.builder.build_load(ptr_type, *stack_ptr, "load_var").unwrap();
                if let inkwell::values::BasicValueEnum::PointerValue(ptr_val) = value {
                    if let Some(release_func) = self.module.get_function("vp_release") {
                        self.builder.build_call(release_func, &[ptr_val.into(), null_ptr.into()], "release_var").unwrap();
                    }
                }
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
            context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),  // name
            context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),  // bases
            context.i64_type().into(),  // base_count
            context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),  // methods
            context.i64_type().into(),  // method_count
            context.i64_type().into(),  // instance_size
            context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),  // init
            context.i8_type().ptr_type(inkwell::AddressSpace::default()).into(),  // dealloc
        ], false);
        
        // Create class metadata global
        let class_global_name = format!("__viper_class_{}", name);
        let class_global = self.module.add_global(class_struct_type, None, &class_global_name);
        class_global.set_constant(false);
        class_global.set_unnamed_addr(true);
        
        // Create class name string
        let name_str = self.create_global_string(name);
        
        // Create initializer values
        let null_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
        let base_count_val = context.i64_type().const_int(0, false);  // Will be updated with inheritance
        let method_count_val = context.i64_type().const_int(metadata.methods.len() as u64, false);
        let instance_size_val = context.i64_type().const_int(metadata.instance_size as u64, false);
        let init_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
        let dealloc_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default()).const_null();
        
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
        for stmt in body {
            if let Stmt::Function { name: method_name, params, return_type, body: method_body, decorators, .. } = stmt {
                // Check for staticmethod decorator
                let is_static = decorators.iter().any(|d| d.name == "staticmethod");
                
                // Generate mangled method name
                let mangled_name = format!("__method_{}_{}", name, method_name);
                
                // For static methods, generate as regular function
                // For instance methods, the first param is 'self'
                if is_static {
                    // Static method - no self parameter
                    self.define_function(&mangled_name, method_name, params, return_type, method_body)?;
                } else {
                    // Instance method - already has self parameter in AST
                    self.define_function(&mangled_name, method_name, params, return_type, method_body)?;
                }
            }
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
