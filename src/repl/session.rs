use std::collections::HashMap;
use std::ffi::c_void;
use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;

use crate::codegen::CodeGen;

pub struct ReplSession {
    chunk_counter: usize,
    // Persistent shadow store of REPL variables
    int_vars: HashMap<String, i64>,
    float_vars: HashMap<String, f64>,
    bool_vars: HashMap<String, bool>,
    str_vars: HashMap<String, *mut c_void>, // raw pointers!
    // We could store other pointers (lists, channels) here later
}

unsafe impl Send for ReplSession {}
unsafe impl Sync for ReplSession {}

impl ReplSession {
    pub fn new() -> Self {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        Self {
            chunk_counter: 0,
            int_vars: HashMap::new(),
            float_vars: HashMap::new(),
            bool_vars: HashMap::new(),
            str_vars: HashMap::new(),
        }
    }

    pub fn execute_chunk(&mut self, source: &str) -> Result<(), String> {
        self.chunk_counter += 1;
        
        // 1. Wrap source with declarations for our shadowed variables
        let wrapped_source = self.build_wrapped_source(source);

        // 2. Tokenize and parse
        let mut lexer = crate::lexer::Lexer::new(&wrapped_source);
        let tokens = lexer.tokenize()?;

        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse()?;

        // Type checking
        let mut type_checker = crate::semantic::type_checker::TypeChecker::new();
        type_checker.check(&ast).map_err(|e| {
            format!(
                "Type errors found:\n{}",
                e.iter().map(|err| format!(" - {}", err)).collect::<Vec<_>>().join("\n")
            )
        })?;

        // 3. JIT Compile
        let context = Context::create();
        let module_name = format!("repl_chunk_{}", self.chunk_counter);
        
        let mut codegen = CodeGen::new(&context, &module_name);
        codegen.generate(&ast)?;
        codegen.verify()?;

        let execution_engine = codegen
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

        crate::jit_stubs::register_stubs(&execution_engine, codegen.module());

        // 4. Run the code
        unsafe {
            let init_func_name = "__module_level__";
            if let Some(func) = codegen.module().get_function(init_func_name) {
                // Remove main if exists since we run __module_level__
                let func_val = execution_engine
                    .get_function_value(init_func_name)
                    .map_err(|e| format!("Failed to find JIT init function: {}", e))?;

                execution_engine.run_function(func_val, &[]);
            } else if let Some(func) = codegen.module().get_function("main") {
                let func_val = execution_engine
                    .get_function_value("main")
                    .map_err(|e| format!("Failed to find JIT main function: {}", e))?;

                execution_engine.run_function(func_val, &[]);
            }

            // 5. Read back new generic globals (using inkwell API)
            // Note: Since viper variables are often declared inside `__module_level__` as allocas 
            // for simple local run, they might not be globals!
            // However, Viper's `CodeGen` might use globals for global constants.
            // If they are local allocas inside `__module_level__` they are lost.
            // For a complete interactive REPL, Viper's CodeGen needs a special mode 
            // to treat REPL top-level vars as globals, but for now we might not be able to 
            // easily read them if they are local. 
        }

        Ok(())
    }

    fn build_wrapped_source(&self, source: &str) -> String {
        // We will prepend user's new assignments with earlier variable state.
        // Wait, if we prepend `x = 10`, it resets it. 
        // For a true REPL, we want the old vars to be available.
        let mut preamble = String::new();

        for (name, val) in &self.int_vars {
            preamble.push_str(&format!("{} = {}\n", name, val));
        }
        for (name, val) in &self.float_vars {
            preamble.push_str(&format!("{} = {}\n", name, val));
        }
        for (name, val) in &self.bool_vars {
            preamble.push_str(&format!("{} = {}\n", name, if *val { "True" } else { "False" }));
        }

        preamble.push_str(source);
        preamble
    }

    pub fn reset(&mut self) {
        self.chunk_counter = 0;
        self.int_vars.clear();
        self.float_vars.clear();
        self.bool_vars.clear();
        self.str_vars.clear();
    }
}
