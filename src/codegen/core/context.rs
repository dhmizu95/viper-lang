//! CodeGen struct definition and constructor

use crate::ast::Type;
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};
use std::collections::{HashMap, HashSet};

use crate::codegen::builder::IRBuilder;
use crate::codegen::types::TypeMapper;
use crate::codegen::variables::{LoopContext, VarInfo};
use crate::semantic::escape_analysis::EscapeAnalyzer;
use crate::semantic::closure_analysis::ClosureAnalyzer;

/// Main code generator that translates AST to LLVM IR
pub struct CodeGen<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: inkwell::module::Module<'ctx>,
    pub(crate) builder: inkwell::builder::Builder<'ctx>,
    pub(crate) ir_builder: IRBuilder<'ctx>,
    pub(crate) type_mapper: TypeMapper<'ctx>,
    pub(crate) variables: HashMap<String, VarInfo<'ctx>>,
    pub(crate) functions: HashMap<String, FunctionValue<'ctx>>,
    pub(crate) global_constants: HashMap<String, GlobalValue<'ctx>>,
    pub(crate) loop_stack: Vec<LoopContext<'ctx>>,
    pub(crate) list_vars: HashSet<String>,
    pub(crate) dict_vars: HashSet<String>,
    pub(crate) bool_list_vars: HashSet<String>,
    pub(crate) bigint_vars: HashSet<String>,
    pub(crate) var_types: HashMap<String, Type>,
    /// Functions that contain BigInt variables (need special optimization handling)
    pub(crate) bigint_functions: HashSet<String>,
    pub(crate) escape_analyzer: EscapeAnalyzer,
    pub(crate) closure_analyzer: ClosureAnalyzer,
    /// Variables that are captured by nested functions
    pub(crate) closure_cells: HashMap<String, crate::codegen::state::ClosureCellInfo<'ctx>>,
    pub(crate) current_function: Option<String>,
    pub(crate) current_class: Option<String>,  // Current class context for super() and methods
    pub(crate) in_classmethod: bool,  // True when generating code for a @classmethod
    /// Functions decorated with @lru_cache or @memoize - maps function name to cache global pointer
    pub(crate) memoized_functions: HashMap<String, PointerValue<'ctx>>,
    /// Enable automatic memoization for pure recursive functions
    pub(crate) auto_memoize: bool,
    /// Warn about non-memoized recursive functions (default: true)
    pub(crate) memoize_warn: bool,
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
            current_function: None,
            current_class: None,
            in_classmethod: false,
            memoized_functions: HashMap::new(),
            auto_memoize: false,  // Disabled by default - users must opt-in via @lru_cache
            memoize_warn: true,   // Warn about non-memoized recursion by default
        }
    }

    /// Get the generated LLVM module
    pub fn module(&self) -> &inkwell::module::Module<'ctx> {
        &self.module
    }

    /// Get the list of functions containing BigInt variables
    /// These functions should skip mem2reg optimization
    pub fn bigint_functions(&self) -> &HashSet<String> {
        &self.bigint_functions
    }
}
