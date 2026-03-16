//! CodeGen struct definition and constructor

use crate::ast::Type;
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue, PointerValue};
use std::collections::{HashMap, HashSet};

use crate::codegen::builder::IRBuilder;
use crate::codegen::types::TypeMapper;
use crate::codegen::variables::{LoopContext, VarInfo};
use crate::semantic::closure_analysis::ClosureAnalyzer;
use crate::semantic::escape_analysis::EscapeAnalyzer;

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
    /// Track which runtime symbols are used (for selective linking)
    pub(crate) used_runtime_symbols: HashSet<String>,
    pub(crate) escape_analyzer: EscapeAnalyzer,
    pub(crate) closure_analyzer: ClosureAnalyzer,
    /// Variables that are captured by nested functions
    pub(crate) closure_cells: HashMap<String, crate::codegen::state::ClosureCellInfo<'ctx>>,
    pub(crate) current_function: Option<String>,
    pub(crate) current_class: Option<String>, // Current class context for super() and methods
    pub(crate) in_classmethod: bool,          // True when generating code for a @classmethod
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
            auto_memoize: false, // Disabled by default - users must opt-in via @lru_cache
            memoize_warn: true,  // Warn about non-memoized recursion by default
            used_runtime_symbols: HashSet::new(),
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

    /// Get the set of used runtime symbols (for selective linking)
    pub fn used_runtime_symbols(&self) -> &HashSet<String> {
        &self.used_runtime_symbols
    }

    /// Track a runtime symbol as used
    pub fn track_runtime_symbol(&mut self, symbol: &str) {
        self.used_runtime_symbols.insert(symbol.to_string());
    }

    /// Get the list of required runtime modules based on used symbols
    pub fn get_required_modules(&self) -> Vec<&'static str> {
        let mut modules = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for symbol in &self.used_runtime_symbols {
            let module = symbol_to_module(symbol);
            if seen.insert(module) {
                modules.push(module);
            }
        }

        modules.sort();
        modules
    }
}

/// Maps a runtime symbol to its corresponding module name
fn symbol_to_module(symbol: &str) -> &'static str {
    // Core - always needed
    if symbol.starts_with("vp_alloc") || symbol.starts_with("vp_free")
        || symbol.starts_with("vp_retain") || symbol.starts_with("vp_release")
        || symbol.starts_with("vp_ref_count") || symbol == "malloc" || symbol == "free" {
        return "core";
    }

    // Lists
    if symbol.starts_with("vp_list_") || symbol == "vp_enumerate" || symbol == "vp_zip"
        || symbol.starts_with("vp_list_bool") {
        return "lists";
    }

    // BitVector (bool lists)
    if symbol.starts_with("vp_bitvec_") {
        return "lists";
    }

    // Dicts
    if symbol.starts_with("vp_dict_") {
        return "dicts";
    }

    // Strings
    if symbol.starts_with("vp_str_") || symbol.starts_with("vp_bytes_")
        || symbol.starts_with("vp_char_") {
        return "strings";
    }

    // Tuples
    if symbol.starts_with("vp_tuple_") {
        return "tuples";
    }

    // Math
    if symbol.starts_with("vp_math_") || symbol.starts_with("vp_pow")
        || symbol.starts_with("vp_hash_") {
        return "math";
    }

    // Print
    if symbol.starts_with("vp_print") || symbol == "vp_exit" {
        return "print";
    }

    // Async
    if symbol.starts_with("vp_async_") || symbol.starts_with("vp_future_")
        || symbol.starts_with("vp_fiber_") || symbol.starts_with("vp_iterator") {
        return "async";
    }

    // Concurrency (channels, waitgroups, threads)
    if symbol.starts_with("vp_chan_") || symbol.starts_with("vp_waitgroup")
        || symbol.starts_with("vp_thread") || symbol.starts_with("vp_spawn")
        || symbol.starts_with("vp_scheduler") || symbol.starts_with("vp_submit")
        || symbol.starts_with("vp_init_threadpool") || symbol == "vp_wait_all_tasks" {
        return "concurrency";
    }

    // Memoization/Caching
    if symbol.starts_with("vp_cache_") || symbol.starts_with("vp_lru_") {
        return "memoize";
    }

    // BigInt
    if symbol.starts_with("vp_bigint_") || symbol.starts_with("tagged_int")
        || symbol.starts_with("gmp_") {
        return "bigint";
    }

    // JSON
    if symbol.starts_with("vp_json_") {
        return "json";
    }

    // Regex
    if symbol.starts_with("vp_re_") {
        return "regex";
    }

    // Logging
    if symbol.starts_with("vp_logging_") {
        return "logging";
    }

    // Struct
    if symbol.starts_with("vp_struct_") {
        return "struct";
    }

    // Closure cells
    if symbol.starts_with("vp_closure_cell") {
        return "closure";
    }

    // Default to core if unknown
    "core"
}
