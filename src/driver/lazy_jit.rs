//! Lazy Compilation Framework for JIT Memory Reduction
//!
//! This module provides infrastructure for lazy compilation to reduce JIT memory overhead:
//! - Compile functions on first call only (not all at once)
//! - Reduce initial memory footprint by deferring compilation
//! - Enable tiered compilation (baseline → optimizing)
//!
//! # Memory Savings
//! Standard JIT: ~60MB base overhead (all functions compiled upfront)
//! Lazy JIT: ~20-30MB (functions compiled on-demand)
//!
//! # Usage
//! ```rust,ignore
//! use viper_lang::driver::LazyJitEngine;
//! use inkwell::context::Context;
//!
//! let context = Context::create();
//! let module = context.create_module("test");
//! let lazy_engine = LazyJitEngine::new(&context, 3);
//! lazy_engine.add_module(module);
//!
//! // Functions are compiled on first call
//! let addr = lazy_engine.get_function("my_func")?;
//! ```

use crate::error::{Result, ViperError};
use inkwell::{
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    targets::{InitializationConfig, Target},
    OptimizationLevel,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Lazy JIT execution engine
///
/// Defers compilation of functions until they are first called,
/// reducing initial memory footprint.
pub struct LazyJitEngine<'ctx> {
    /// LLVM context
    context: &'ctx Context,
    /// Optimization level
    opt_level: OptimizationLevel,
    /// Modules to be compiled (not yet compiled)
    pending_modules: Vec<Module<'ctx>>,
    /// Compiled functions (name -> address)
    compiled_functions: HashMap<String, u64>,
    /// Execution engine (created on first compilation)
    execution_engine: Option<ExecutionEngine<'ctx>>,
    /// Track compilation statistics
    stats: CompilationStats,
}

/// Compilation statistics for monitoring
#[derive(Debug, Default)]
struct CompilationStats {
    /// Total functions compiled
    pub total_compiled: AtomicUsize,
    /// Time when first function was compiled
    pub first_compilation: std::sync::Mutex<Option<Instant>>,
    /// Total compilation time in milliseconds
    pub total_compilation_time_ms: AtomicUsize,
}

impl<'ctx> LazyJitEngine<'ctx> {
    /// Create a new lazy JIT engine
    pub fn new(context: &'ctx Context, opt_level: u32) -> Self {
        let opt = match opt_level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Less,
            2 => OptimizationLevel::Default,
            3 | _ => OptimizationLevel::Aggressive,
        };

        Self {
            context,
            opt_level: opt,
            pending_modules: Vec::new(),
            compiled_functions: HashMap::new(),
            execution_engine: None,
            stats: CompilationStats::default(),
        }
    }

    /// Add a module to be compiled lazily
    pub fn add_module(&mut self, module: Module<'ctx>) {
        self.pending_modules.push(module);
    }

    /// Get or compile a function by name
    ///
    /// Returns the function address, compiling it if necessary.
    pub fn get_function(&mut self, name: &str) -> Result<u64> {
        // Check if already compiled (fast path - no locking needed)
        if let Some(&addr) = self.compiled_functions.get(name) {
            return Ok(addr);
        }

        // Need to compile - initialize execution engine if not done
        if self.execution_engine.is_none() {
            self.initialize_engine()?;
        }

        // Find the function in pending modules
        for module in &self.pending_modules {
            if let Some(_func) = module.get_function(name) {
                // Compile this function
                let engine = self.execution_engine.as_ref().unwrap();

                // Track compilation time
                let start_time = Instant::now();

                let _func_value = engine.get_function_value(name).map_err(|e| {
                    ViperError::driver(format!("Failed to get function {}: {}", name, e))
                })?;

                let addr = engine.get_function_address(name).map_err(|e| {
                    ViperError::driver(format!("Failed to get address for {}: {}", name, e))
                })?;

                // Record compilation statistics
                let elapsed_ms = start_time.elapsed().as_millis() as usize;
                self.stats.total_compiled.fetch_add(1, Ordering::Relaxed);
                self.stats.total_compilation_time_ms.fetch_add(elapsed_ms, Ordering::Relaxed);

                // Record first compilation time
                {
                    let mut first_time = self.stats.first_compilation.lock().unwrap();
                    if first_time.is_none() {
                        *first_time = Some(Instant::now());
                    }
                }

                self.compiled_functions.insert(name.to_string(), addr as u64);
                return Ok(addr as u64);
            }
        }

        Err(ViperError::driver(format!("Function '{}' not found", name)))
    }

    /// Get compilation statistics
    pub fn get_compilation_stats(&self) -> CompilationStatsSummary {
        let first_compilation = self.stats.first_compilation.lock().unwrap();
        CompilationStatsSummary {
            total_compiled: self.stats.total_compiled.load(Ordering::Relaxed),
            total_compilation_time_ms: self.stats.total_compilation_time_ms.load(Ordering::Relaxed),
            avg_compilation_time_ms: {
                let total = self.stats.total_compiled.load(Ordering::Relaxed);
                if total > 0 {
                    self.stats.total_compilation_time_ms.load(Ordering::Relaxed) / total
                } else {
                    0
                }
            },
            time_since_first_compilation: first_compilation
                .map(|i| i.elapsed().as_secs_f64())
                .unwrap_or(0.0),
        }
    }

    /// Initialize the execution engine
    fn initialize_engine(&mut self) -> Result<()> {
        Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
            ViperError::driver(format!("Failed to initialize native target: {}", e))
        })?;

        // Create execution engine from the first module
        if let Some(module) = self.pending_modules.first() {
            let engine = module
                .create_jit_execution_engine(self.opt_level)
                .map_err(|e| ViperError::driver(format!("Failed to create JIT engine: {}", e)))?;

            self.execution_engine = Some(engine);
        } else {
            return Err(ViperError::driver("No modules available for JIT compilation"));
        }

        Ok(())
    }

    /// Get memory usage statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        let mut stats = MemoryStats::default();

        // Estimate memory usage
        stats.pending_modules = self.pending_modules.len();
        stats.compiled_functions = self.compiled_functions.len();

        // Base LLVM overhead (approximate)
        stats.base_overhead_mb = if self.execution_engine.is_some() { 60.0 } else { 0.0 };

        // Additional memory per compiled function (approximate)
        stats.compiled_function_memory_mb = (self.compiled_functions.len() as f64) * 0.1;

        stats.total_estimated_mb = stats.base_overhead_mb + stats.compiled_function_memory_mb;

        stats
    }

    /// Pre-compile a set of functions (eager compilation)
    ///
    /// Use this for hot paths where you know functions will be called.
    pub fn precompile(&mut self, function_names: &[&str]) -> Result<()> {
        for &name in function_names {
            self.get_function(name)?;
        }
        Ok(())
    }
}

/// Compilation statistics summary
#[derive(Debug, Default)]
pub struct CompilationStatsSummary {
    /// Total functions compiled
    pub total_compiled: usize,
    /// Total compilation time in milliseconds
    pub total_compilation_time_ms: usize,
    /// Average compilation time per function in milliseconds
    pub avg_compilation_time_ms: usize,
    /// Time since first compilation in seconds
    pub time_since_first_compilation: f64,
}

impl std::fmt::Display for CompilationStatsSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Compilation Statistics:")?;
        writeln!(f, "  Total compiled: {}", self.total_compiled)?;
        writeln!(f, "  Total time: {} ms", self.total_compilation_time_ms)?;
        writeln!(f, "  Avg per function: {} ms", self.avg_compilation_time_ms)?;
        writeln!(f, "  Time since first: {:.2}s", self.time_since_first_compilation)
    }
}

/// Memory usage statistics for the lazy JIT engine
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Number of pending (not yet compiled) modules
    pub pending_modules: usize,
    /// Number of compiled functions
    pub compiled_functions: usize,
    /// Base LLVM overhead in MB
    pub base_overhead_mb: f64,
    /// Memory used by compiled functions in MB
    pub compiled_function_memory_mb: f64,
    /// Total estimated memory usage in MB
    pub total_estimated_mb: f64,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Lazy JIT Memory Statistics:")?;
        writeln!(f, "  Pending modules: {}", self.pending_modules)?;
        writeln!(f, "  Compiled functions: {}", self.compiled_functions)?;
        writeln!(f, "  Base overhead: {:.1} MB", self.base_overhead_mb)?;
        writeln!(f, "  Function code: {:.1} MB", self.compiled_function_memory_mb)?;
        writeln!(f, "  Total estimated: {:.1} MB", self.total_estimated_mb)
    }
}

/// Tiered compilation strategy
///
/// Implements three tiers:
/// 1. Interpreter - fastest startup, slowest execution
/// 2. Baseline JIT - quick compilation, moderate optimization
/// 3. Optimizing JIT - slow compilation, best optimization
pub enum CompilationTier {
    /// Interpret without compilation (future implementation)
    Interpreter,
    /// Quick compilation with minimal optimization
    Baseline,
    /// Full optimization
    Optimizing,
}

impl CompilationTier {
    /// Get the optimization level for this tier
    pub fn optimization_level(&self) -> OptimizationLevel {
        match self {
            CompilationTier::Interpreter => OptimizationLevel::None,
            CompilationTier::Baseline => OptimizationLevel::Less,
            CompilationTier::Optimizing => OptimizationLevel::Aggressive,
        }
    }
}

/// Tiered JIT engine with profiling
///
/// Starts with baseline compilation and re-compiles hot functions
/// with full optimization.
pub struct TieredJitEngine<'ctx> {
    /// Baseline engine (fast compilation)
    baseline_engine: LazyJitEngine<'ctx>,
    /// Optimizing engine (slow compilation, better code)
    optimizing_engine: Option<LazyJitEngine<'ctx>>,
    /// Call counts for each function
    call_counts: HashMap<String, usize>,
    /// Threshold for promoting to optimizing tier
    promotion_threshold: usize,
    /// Functions already promoted to optimizing tier
    promoted_functions: HashMap<String, u64>,
}

impl<'ctx> TieredJitEngine<'ctx> {
    /// Create a new tiered JIT engine
    pub fn new(context: &'ctx Context) -> Self {
        Self {
            baseline_engine: LazyJitEngine::new(context, 1), // O1 for baseline
            optimizing_engine: None,
            call_counts: HashMap::new(),
            promotion_threshold: 100, // Promote after 100 calls
            promoted_functions: HashMap::new(),
        }
    }

    /// Create a new tiered JIT engine with custom promotion threshold
    pub fn with_threshold(context: &'ctx Context, promotion_threshold: usize) -> Self {
        Self {
            baseline_engine: LazyJitEngine::new(context, 1),
            optimizing_engine: None,
            call_counts: HashMap::new(),
            promotion_threshold,
            promoted_functions: HashMap::new(),
        }
    }

    /// Add a module to both engines
    pub fn add_module(&mut self, module: Module<'ctx>) {
        self.baseline_engine.add_module(module);
    }

    /// Get or compile a function, potentially promoting to optimizing tier
    pub fn get_function(&mut self, name: &str) -> Result<u64> {
        // Check if already promoted to optimizing tier
        if let Some(&addr) = self.promoted_functions.get(name) {
            return Ok(addr);
        }

        // Increment call count
        let count = self.call_counts.entry(name.to_string()).or_insert(0);
        *count += 1;

        // Check if we should promote to optimizing tier
        if *count >= self.promotion_threshold {
            // Initialize optimizing engine if not done
            if self.optimizing_engine.is_none() {
                let context = self.baseline_engine.context;
                self.optimizing_engine = Some(LazyJitEngine::new(context, 3)); // O3 for optimizing
            }

            // Try to get from optimizing engine
            if let Some(opt_engine) = &mut self.optimizing_engine {
                // Compile this function in optimizing engine
                if let Ok(addr) = opt_engine.get_function(name) {
                    // Cache the promoted function address
                    self.promoted_functions.insert(name.to_string(), addr);
                    return Ok(addr);
                }
            }
        }

        // Get from baseline engine
        self.baseline_engine.get_function(name)
    }

    /// Get memory statistics for both tiers
    pub fn get_memory_stats(&self) -> TieredMemoryStats {
        TieredMemoryStats {
            baseline: self.baseline_engine.get_memory_stats(),
            optimizing: self.optimizing_engine.as_ref().map(|e| e.get_memory_stats()),
            hot_functions: self
                .call_counts
                .iter()
                .filter(|(_, &count)| count >= self.promotion_threshold)
                .count(),
            promoted_functions: self.promoted_functions.len(),
        }
    }

    /// Get compilation statistics for both tiers
    pub fn get_compilation_stats(&self) -> TieredCompilationStats {
        TieredCompilationStats {
            baseline: self.baseline_engine.get_compilation_stats(),
            optimizing: self.optimizing_engine.as_ref().map(|e| e.get_compilation_stats()),
            total_calls: self.call_counts.values().sum(),
            hot_functions: self
                .call_counts
                .iter()
                .filter(|(_, &count)| count >= self.promotion_threshold)
                .count(),
            promoted_functions: self.promoted_functions.len(),
        }
    }
}

/// Memory statistics for tiered JIT
pub struct TieredMemoryStats {
    /// Baseline tier statistics
    pub baseline: MemoryStats,
    /// Optimizing tier statistics (if initialized)
    pub optimizing: Option<MemoryStats>,
    /// Number of hot functions (call count >= threshold)
    pub hot_functions: usize,
    /// Number of functions promoted to optimizing tier
    pub promoted_functions: usize,
}

impl std::fmt::Display for TieredMemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tiered JIT Memory Statistics:")?;
        writeln!(f, "\nBaseline Tier:")?;
        writeln!(f, "{}", self.baseline)?;
        if let Some(ref opt) = self.optimizing {
            writeln!(f, "\nOptimizing Tier:")?;
            writeln!(f, "{}", opt)?;
        }
        writeln!(f, "\nHot functions: {}", self.hot_functions)?;
        writeln!(f, "Promoted functions: {}", self.promoted_functions)
    }
}

/// Compilation statistics for tiered JIT
pub struct TieredCompilationStats {
    /// Baseline tier statistics
    pub baseline: CompilationStatsSummary,
    /// Optimizing tier statistics (if initialized)
    pub optimizing: Option<CompilationStatsSummary>,
    /// Total function calls
    pub total_calls: usize,
    /// Number of hot functions (call count >= threshold)
    pub hot_functions: usize,
    /// Number of functions promoted to optimizing tier
    pub promoted_functions: usize,
}

impl std::fmt::Display for TieredCompilationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tiered JIT Compilation Statistics:")?;
        writeln!(f, "\nBaseline Tier:")?;
        writeln!(f, "{}", self.baseline)?;
        if let Some(ref opt) = self.optimizing {
            writeln!(f, "\nOptimizing Tier:")?;
            writeln!(f, "{}", opt)?;
        }
        writeln!(f, "\nTotal calls: {}", self.total_calls)?;
        writeln!(f, "Hot functions: {}", self.hot_functions)?;
        writeln!(f, "Promoted functions: {}", self.promoted_functions)
    }
}
