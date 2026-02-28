//! Common state for code generation (shared across all codegen modules)

use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::{HashMap, HashSet};

use crate::codegen::builder::IRBuilder;
use crate::codegen::variables::{LoopContext, VarInfo};
use crate::semantic::escape_analysis::EscapeAnalyzer;

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
    pub list_vars: &'a mut HashSet<String>,
    pub dict_vars: &'a mut HashSet<String>,
    pub bool_list_vars: &'a mut HashSet<String>, // Track bool-specific lists
    pub bigint_vars: &'a mut HashSet<String>,    // Track BigInt variables
    pub escape_analyzer: Option<&'a mut EscapeAnalyzer>,
    pub current_function: Option<&'a str>,
}

impl<'a, 'ctx> CodeGenState<'a, 'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        context: &'ctx Context,
        module: &'a inkwell::module::Module<'ctx>,
        builder: &'a inkwell::builder::Builder<'ctx>,
        ir_builder: &'a IRBuilder<'ctx>,
        variables: &'a mut HashMap<String, VarInfo<'ctx>>,
        functions: &'a HashMap<String, FunctionValue<'ctx>>,
        global_constants: &'a mut HashMap<String, GlobalValue<'ctx>>,
        loop_stack: &'a mut Vec<LoopContext<'ctx>>,
        list_vars: &'a mut HashSet<String>,
        dict_vars: &'a mut HashSet<String>,
        bool_list_vars: &'a mut HashSet<String>,
        bigint_vars: &'a mut HashSet<String>,
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
            list_vars,
            dict_vars,
            bool_list_vars,
            bigint_vars,
            escape_analyzer: None,
            current_function: None,
        }
    }

    /// Create a new state with escape analysis info
    #[allow(clippy::too_many_arguments)]
    pub fn with_escape_analysis(
        context: &'ctx Context,
        module: &'a inkwell::module::Module<'ctx>,
        builder: &'a inkwell::builder::Builder<'ctx>,
        ir_builder: &'a IRBuilder<'ctx>,
        variables: &'a mut HashMap<String, VarInfo<'ctx>>,
        functions: &'a HashMap<String, FunctionValue<'ctx>>,
        global_constants: &'a mut HashMap<String, GlobalValue<'ctx>>,
        loop_stack: &'a mut Vec<LoopContext<'ctx>>,
        list_vars: &'a mut HashSet<String>,
        dict_vars: &'a mut HashSet<String>,
        bool_list_vars: &'a mut HashSet<String>,
        bigint_vars: &'a mut HashSet<String>,
        escape_analyzer: &'a mut EscapeAnalyzer,
        current_function: &'a str,
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
            list_vars,
            dict_vars,
            bool_list_vars,
            bigint_vars,
            escape_analyzer: Some(escape_analyzer),
            current_function: Some(current_function),
        }
    }

    /// Check if a variable can be stack-allocated based on escape analysis
    pub fn can_stack_allocate(&self, var_name: &str) -> bool {
        if let (Some(analyzer), Some(func)) = (self.escape_analyzer.as_ref(), self.current_function)
        {
            analyzer.can_stack_allocate(func, var_name)
        } else {
            true // Default to stack allocation if no escape analysis info
        }
    }

    /// Check if a variable needs ARC retain/release operations
    pub fn needs_arc(&self, var_name: &str) -> bool {
        if let (Some(analyzer), Some(func)) = (self.escape_analyzer.as_ref(), self.current_function)
        {
            analyzer.needs_arc(func, var_name)
        } else {
            false // Default to no ARC if no escape analysis info
        }
    }

    /// Check if a variable needs ARC cleanup at function exit
    pub fn needs_arc_cleanup(&self, var_name: &str) -> bool {
        if let (Some(analyzer), Some(func)) = (self.escape_analyzer.as_ref(), self.current_function)
        {
            analyzer.needs_arc_cleanup(func, var_name)
        } else {
            false // Default to no cleanup if no escape analysis info
        }
    }

    /// Check if a variable is shared across threads
    pub fn is_thread_shared(&self, var_name: &str) -> bool {
        if let (Some(analyzer), Some(func)) = (self.escape_analyzer.as_ref(), self.current_function)
        {
            analyzer.is_thread_shared(func, var_name)
        } else {
            true // Default to atomic ARC safely
        }
    }

    /// Set reference type flag for a variable
    pub fn set_reference_type(&mut self, var_name: &str, is_ref: bool) {
        if let (Some(analyzer), Some(func)) = (self.escape_analyzer.as_mut(), self.current_function)
        {
            analyzer.set_reference_type(func, var_name, is_ref);
        }
    }

    /// Mark a variable as a list
    pub fn mark_as_list(&mut self, name: String) {
        self.list_vars.insert(name);
    }

    /// Mark a variable as a dict
    pub fn mark_as_dict(&mut self, name: String) {
        self.dict_vars.insert(name);
    }

    /// Check if a variable is a list
    pub fn is_list(&self, name: &str) -> bool {
        self.list_vars.contains(name)
    }

    /// Mark a variable as a bool list
    pub fn mark_as_bool_list(&mut self, name: String) {
        self.bool_list_vars.insert(name.clone());
        self.list_vars.insert(name); // Bool lists are also lists
    }

    /// Check if a variable is a bool list
    pub fn is_bool_list(&self, name: &str) -> bool {
        self.bool_list_vars.contains(name)
    }

    /// Check if a variable is a dict
    pub fn is_dict(&self, name: &str) -> bool {
        self.dict_vars.contains(name)
    }

    /// Mark a variable as BigInt
    pub fn mark_as_bigint(&mut self, name: String) {
        self.bigint_vars.insert(name);
    }

    /// Check if a variable is BigInt
    pub fn is_bigint(&self, name: &str) -> bool {
        self.bigint_vars.contains(name)
    }

    /// Generate ARC retain call for a value
    pub fn build_retain(&self, value: inkwell::values::BasicValueEnum<'ctx>, name: &str) {
        if !self.needs_arc(name) {
            return;
        }

        let func_name = if self.is_thread_shared(name) { "vp_retain" } else { "vp_retain_local" };

        if let Some(retain_func) = self.module.get_function(func_name) {
            self.builder
                .build_call(retain_func, &[value.into()], &format!("retain_{}", name))
                .expect("build retain call");
        }
    }

    /// Generate ARC release call for a value (with null destructor)
    pub fn build_release(&self, value: inkwell::values::BasicValueEnum<'ctx>, name: &str) {
        if !self.needs_arc(name) {
            return;
        }

        if self.is_thread_shared(name) {
            if let Some(release_func) = self.module.get_function("vp_release") {
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let null_ptr = ptr_type.const_null();
                self.builder
                    .build_call(
                        release_func,
                        &[value.into(), null_ptr.into()],
                        &format!("release_{}", name),
                    )
                    .expect("build release call");
            }
        } else {
            if let Some(release_func) = self.module.get_function("vp_release_local") {
                self.builder
                    .build_call(release_func, &[value.into()], &format!("release_{}", name))
                    .expect("build release_local call");
            }
        }
    }

    /// Check if a variable can use register allocation (SSA)
    /// Returns true if the variable does not escape and is not reassigned
    pub fn can_use_register(&self, var_name: &str) -> bool {
        // For register allocation, the variable must:
        // 1. Not escape (based on escape analysis)
        // 2. Not be reassigned (we detect this at assignment time and upgrade to stack)
        // 3. Not be a pointer type (pointers need stack for proper memory management)
        self.can_stack_allocate(var_name)
    }
}
