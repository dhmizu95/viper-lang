# Codegen Optimization Implementation Plan

**Date:** March 12, 2026  
**Version:** 0.5.0  
**Status:** Partial Implementation - Phase 2 In Progress  

---

## Executive Summary

This plan details the implementation strategy for LLVM-level codegen optimizations in the Viper compiler. These optimizations focus on reducing function call overhead, improving inlining, and leveraging LLVM's optimization capabilities.

### Goal

Close the performance gap with C by implementing aggressive codegen optimizations while maintaining correctness and compile-time efficiency.

### Current Performance Gap

| Benchmark | Viper AOT-O3 | C -O3 | Gap |
|-----------|--------------|-------|-----|
| Fibonacci | 71ms | 13ms | **5.5x** |
| Prime Sieve | 4ms | 1ms | **4x** |
| Matrix Mul | 6ms | 1ms | **6x** |

### Target After Optimizations

| Benchmark | Current | Target | Improvement |
|-----------|---------|--------|-------------|
| Fibonacci | 71ms | 25ms | 2.8x faster |
| Prime Sieve | 4ms | 1.5ms | 2.7x faster |
| Matrix Mul | 6ms | 2ms | 3x faster |

---

## Optimization Techniques Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Codegen Optimizations                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │  Inlining   │  │   Purity    │  │   Direct    │            │
│  │             │  │  Analysis   │  │   Calls     │            │
│  │  • always   │  │             │  │             │            │
│  │    inline   │  │  • readonly │  │  • Named    │            │
│  │  • inline   │  │  • willret  │  │    calls    │            │
│  │    hint     │  │  • argmem   │  │  • No       │            │
│  │             │  │    only     │  │    dispatch │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │  List Ops   │  │   Escape    │  │     LTO     │            │
│  │   Inline    │  │  Analysis   │  │             │            │
│  │             │  │             │  │             │            │
│  │  • Direct   │  │  • Stack    │  │  • Cross-   │            │
│  │    GEP      │  │    alloc    │  │    module   │            │
│  │  • No call  │  │  • No ARC   │  │    inline   │            │
│  │    overhead │  │    needed   │  │             │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Status

| Optimization | Status | File | Completion |
|--------------|--------|------|------------|
| Function Inlining | ✅ Partial | `src/codegen/core/functions.rs` | 70% |
| Purity Attributes | ✅ Partial | `src/codegen/core/functions.rs` | 60% |
| Direct Calls | ✅ Partial | `src/codegen/expressions/calls/dispatch.rs` | 80% |
| Inline List Ops | ✅ Implemented | `src/codegen/inline_lists.rs` | 90% |
| Escape Analysis | ✅ Implemented | `src/semantic/escape_analysis.rs` | 100% |
| LTO Support | ✅ Configured | `Cargo.toml` | 100% |
| Constant Folding | ❌ Not Started | - | 0% |
| Loop Invariant Code Motion | ❌ Not Started | - | 0% |

---

## Phase 1: Complete Existing Implementations (Week 1-2)

### 1.1 Complete Function Inlining

**Current State:**
- Basic inlining hints implemented
- Small functions (< 10 statements) marked with `alwaysinline`
- No recursive function handling

**TODO:**
1. Add recursive function detection to prevent infinite inlining
2. Implement inline threshold based on LLVM IR instruction count
3. Add `@inline` and `@noinline` user decorators
4. Track inlined functions for better debugging

**Files to Modify:**
- `src/codegen/core/functions.rs` - Add inline logic
- `src/semantic/recursion_analysis.rs` - Use for recursive detection
- `src/ast/decorators.rs` - Add `@inline` decorator support

**Implementation:**
```rust
/// Enhanced inlining heuristics
fn should_inline(func_val: FunctionValue, body: &[Stmt]) -> bool {
    // Check for user decorators first
    if has_noinline_decorator(func_val) {
        return false;
    }
    if has_inline_decorator(func_val) {
        return true;
    }
    
    // Prevent infinite inlining for recursive functions
    if is_recursive_function(func_val) {
        // Only inline if very small (< 5 statements)
        return body.len() < 5;
    }
    
    // Size-based heuristic
    let instruction_count = estimate_ir_instructions(body);
    instruction_count < 15
}
```

**Testing:**
- Unit test: Verify small functions get inlined
- Unit test: Verify recursive functions handled correctly
- Benchmark: Measure fibonacci before/after

---

### 1.2 Complete Purity Analysis

**Current State:**
- Basic `is_pure_function()` implemented
- `readonly` and `willreturn` attributes added
- No argument-based purity checking

**TODO:**
1. Implement `argmemonly` for functions that only read/write through pointers
2. Add `nounwind` attribute (functions that can't unwind)
3. Track purity across function call graph
4. Handle builtin function purity

**Files to Modify:**
- `src/codegen/core/functions.rs` - Enhance purity detection
- `src/semantic/purity_analysis.rs` - New module for call-graph purity

**Implementation:**
```rust
/// Enhanced purity analysis
fn analyze_function_purity(body: &[Stmt], call_graph: &CallGraph) -> PurityLevel {
    match body {
        // Pure: only computations, no side effects
        _ if is_fully_pure(body) => PurityLevel::Pure,
        
        // ArgMemOnly: only reads/writes through argument pointers
        _ if only_accesses_args(body) => PurityLevel::ArgMemOnly,
        
        // ReadOnly: doesn't modify memory
        _ if doesnt_modify_memory(body) => PurityLevel::ReadOnly,
        
        // Impure: has side effects
        _ => PurityLevel::Impure,
    }
}

enum PurityLevel {
    Pure,           // readonly + willreturn
    ArgMemOnly,     // argmemonly + willreturn
    ReadOnly,       // readonly + willreturn
    Impure,         // no attributes
}
```

**LLVM Attributes Mapping:**
| Purity Level | LLVM Attributes |
|--------------|-----------------|
| Pure | `readonly`, `willreturn`, `nounwind` |
| ArgMemOnly | `argmemonly`, `willreturn`, `nounwind` |
| ReadOnly | `readonly`, `willreturn` |
| Impure | (none) |

---

### 1.3 Complete Direct Call Optimization

**Current State:**
- Direct calls implemented for named functions
- Inline hints added for small functions
- No overload resolution optimization

**TODO:**
1. Cache function lookups for repeated calls
2. Optimize overload resolution at compile time
3. Add call site inlining hints based on hotness

**Files to Modify:**
- `src/codegen/expressions/calls/dispatch.rs` - Optimize dispatch

**Implementation:**
```rust
/// Optimized call generation
fn generate_optimized_call(state: &mut State, name: &str, args: &[Expr]) -> Result {
    // Fast path: direct named call
    if let Some(func_val) = state.known_functions.get(name) {
        // Add inline hint for hot call sites
        if is_hot_call_site(name) {
            add_inline_hint(func_val);
        }
        
        // Generate direct call
        return build_direct_call(func_val, args);
    }
    
    // Slow path: runtime dispatch
    build_indirect_call(name, args)
}
```

---

## Phase 2: New Optimizations (Week 3-5)

### 2.1 Constant Folding for Tagged Values

**Priority:** P0 - High  
**Impact:** 10-30% on arithmetic-heavy code  

**Description:**
Evaluate constant expressions at compile time, handling Viper's tagged integer representation.

**Files to Create:**
- `src/semantic/constant_folding.rs`

**Implementation:**
```rust
/// Constant folder for tagged integers
pub struct ConstantFolder<'ctx> {
    context: &'ctx LLVMContext,
}

impl<'ctx> ConstantFolder<'ctx> {
    pub fn fold_expr(&mut self, expr: &Expr) -> Option<Expr> {
        match expr {
            // Fold arithmetic on tagged integers
            Expr::BinOp { left, right, op } => {
                if let (Some(l), Some(r)) = (self.fold_expr(left), self.fold_expr(right)) {
                    if let (Some(l_val), Some(r_val)) = (l.as_int(), r.as_int()) {
                        let result = match op {
                            BinOp::Add => l_val.wrapping_add(r_val),
                            BinOp::Sub => l_val.wrapping_sub(r_val),
                            BinOp::Mul => l_val.wrapping_mul(r_val),
                            _ => return None,
                        };
                        return Some(Expr::Int(result, Span::default()));
                    }
                }
            }
            _ => {}
        }
        None
    }
}
```

**Integration:**
```rust
// In driver/aot.rs compilation pipeline
let ast = parser.parse()?;
let ast = type_checker.check(ast)?;
let ast = constant_folder.fold(ast);  // ← New pass
let ast = dce.eliminate(ast);
codegen.generate(ast)?;
```

---

### 2.2 Inline List Operations Enhancement

**Priority:** P0 - High  
**Impact:** 30-50% on list-heavy code  

**Current State:**
- `inline_lists.rs` has basic get/set/append
- Only i64, f64, bool types supported

**TODO:**
1. Add inline operations for all list types
2. Inline list length operations
3. Inline list iteration (for loops over lists)

**Files to Modify:**
- `src/codegen/inline_lists.rs` - Extend coverage

**Implementation:**
```rust
/// Inline list length - direct struct field access
pub fn inline_list_len<'ctx>(
    state: &mut CodeGenState<'ctx>,
    list: PointerValue<'ctx>,
) -> IntValue<'ctx> {
    // List struct: { ptr, len, capacity }
    // len is at index 1
    let len_ptr = unsafe {
        state.builder.build_gep(
            list.get_type().get_element_type().into_pointer_type(),
            list,
            &[state.context.i32_type().const_zero(), state.context.i32_type().const_int(1, false)],
            "len_ptr",
        )
    };
    state.builder.build_load(len_ptr.get_type().get_element_type(), len_ptr, "len")
        .into_int_value()
}
```

---

### 2.3 Loop Invariant Code Motion (LICM)

**Priority:** P1 - Medium  
**Impact:** 2-5x on loop-heavy code  

**Description:**
Move computations outside loops if their operands don't change.

**Files to Create:**
- `src/codegen/loop_analysis.rs`
- `src/codegen/licm.rs`

**Implementation:**
```rust
/// Detect loop-invariant expressions
pub struct LoopInvariantAnalyzer<'ctx> {
    state: &'ctx CodeGenState<'ctx>,
}

impl<'ctx> LoopInvariantAnalyzer<'ctx> {
    /// Find expressions that can be hoisted out of loop
    pub fn find_invariants(&self, loop_body: &[Stmt], loop_vars: &[&str]) -> Vec<&Stmt> {
        let mut invariants = Vec::new();
        
        for stmt in loop_body {
            if self.is_loop_invariant(stmt, loop_vars) {
                invariants.push(stmt);
            }
        }
        
        invariants
    }
    
    fn is_loop_invariant(&self, stmt: &Stmt, loop_vars: &[&str]) -> bool {
        // An expression is invariant if:
        // 1. All variables used are not loop variables
        // 2. No function calls that might have side effects
        // 3. No memory operations that might change
        match stmt {
            Stmt::Declare { value, .. } => {
                let vars = self.extract_variables(value);
                vars.iter().all(|v| !loop_vars.contains(&v.as_str()))
            }
            _ => false,
        }
    }
}
```

---

## Phase 3: Advanced Optimizations (Week 6-8)

### 3.1 Profile-Guided Optimization (PGO) Integration

**Priority:** P2 - Medium  
**Impact:** 10-30% overall  

**Current State:**
- `Cargo.toml` has PGO profiles defined
- No runtime profiling workflow

**TODO:**
1. Create PGO instrumentation build command
2. Create benchmark runner for profile collection
3. Automate profile merging and application

**Files to Create:**
- `scripts/pgo_instrument.sh`
- `scripts/pgo_collect.sh`
- `scripts/pgo_build.sh`

**Workflow:**
```bash
# Step 1: Build instrumented binary
cargo build --profile pgo-instrument

# Step 2: Run benchmarks to collect profiles
./scripts/pgo_collect.sh

# Step 3: Merge profiles
llvm-profdata merge -sparse target/pgo-data/*.profraw -o merged.profdata

# Step 4: Build with PGO
RUSTFLAGS="-Cprofile-use=merged.profdata" cargo build --release
```

---

### 3.2 Type Specialization

**Priority:** P1 - Medium  
**Impact:** 20-40% for numeric code  

**Description:**
Generate specialized untagged code for monomorphic numeric functions.

**Files to Create:**
- `src/codegen/type_specialization.rs`

**Implementation:**
```rust
/// Generate specialized version for numeric types
pub fn specialize_numeric_function(
    state: &mut CodeGen,
    func_name: &str,
    param_types: &[Type],
    return_type: &Type,
    body: &[Stmt],
) -> Result<(), String> {
    // Check if all params and return are numeric
    if !all_numeric(param_types) || !is_numeric(return_type) {
        return Ok(());  // Not a candidate for specialization
    }
    
    // Generate specialized version with untagged math
    let specialized_name = format!("{}_spec_{}", func_name, mangle_types(param_types));
    
    // Create function with untagged i64/f64 params
    let func = create_specialized_function(state, &specialized_name, param_types, return_type);
    
    // Generate body using direct LLVM math (no tagging)
    generate_untagged_body(state, func, body)?;
    
    // Update call sites
    redirect_calls_to_specialized(state, func_name, &specialized_name)?;
    
    Ok(())
}
```

**Example Transformation:**
```python
# Original (tagged)
def add(a: int, b: int) -> int:
    return a + b  # Tagged add with overflow checks

# Specialized (untagged)
def add_spec_i64_i64(a: i64, b: i64) -> i64:
    return add i64 a, b  # Direct LLVM add
```

---

### 3.3 Escape Analysis Enhancement

**Priority:** P1 - Medium  
**Impact:** 15-25% on allocation-heavy code  

**Current State:**
- Basic escape analysis implemented
- Stack allocation for non-escaping variables

**TODO:**
1. Add scalar replacement of aggregates (SROA) hints
2. Optimize parameter passing (by-value vs by-reference)
3. Track escape through function returns

**Files to Modify:**
- `src/semantic/escape_analysis.rs`

---

## Phase 4: Platform-Specific Optimizations (Week 9-12)

### 4.1 SIMD Vectorization

**Priority:** P3 - Low  
**Impact:** 4-16x on vectorizable code  

**Description:**
Use LLVM SIMD for list operations.

**Target Operations:**
- List arithmetic: `[1,2,3] + [4,5,6]`
- Scalar broadcast: `[1,2,3] * 2`
- Reductions: `sum([1,2,3,4])`

---

### 4.2 Cache-Aware Optimizations

**Priority:** P3 - Low  
**Impact:** 2-5x on memory-bound code  

**Description:**
Optimize data layout and access patterns for CPU cache.

---

## LLVM Pass Pipeline Configuration

### Current Pipeline
```rust
// In src/driver/aot.rs
let pass_manager = PassManager::create(&module);
pass_manager.add_instruction_combining_pass();
pass_manager.add_reassociate_pass();
pass_manager.add_gvn_pass();
pass_manager.add_cfg_simplification_pass();
pass_manager.add_basic_alias_analysis_pass();
pass_manager.add_promote_memory_to_register_pass();  // mem2reg
pass_manager.add_instruction_combining_pass();
pass_manager.add_reassociate_pass();
```

### Enhanced Pipeline (-O2)
```rust
let pass_manager = PassManager::create(&module);

// Early cleanup
pass_manager.add_instruction_combining_pass();
pass_manager.add_reassociate_pass();

// Main optimization passes
pass_manager.add_gvn_pass();                    // Global Value Numbering
pass_manager.add_licm_pass();                   // Loop Invariant Code Motion
pass_manager.add_loop_unroll_pass();            // Loop Unrolling
pass_manager.add_sccp_pass();                   // Sparse Conditional Constant Propagation
pass_manager.add_aggressive_dce_pass();         // Dead Code Elimination

// Alias analysis
pass_manager.add_basic_alias_analysis_pass();
pass_manager.add_cfg_simplification_pass();

// Memory optimization
pass_manager.add_promote_memory_to_register_pass();  // mem2reg (critical!)
pass_manager.add_sroa_pass();                   // Scalar Replacement of Aggregates

// Late cleanup
pass_manager.add_instruction_combining_pass();
pass_manager.add_reassociate_pass();
```

---

## Benchmark Suite

### Performance Tracking Benchmarks

| Benchmark | File | Current | Target | Priority |
|-----------|------|---------|--------|----------|
| Fibonacci | `benchmarks/01_fibonacci.vp` | 71ms | 25ms | P0 |
| Prime Sieve | `benchmarks/02_prime_sieve.vp` | 4ms | 1.5ms | P0 |
| Matrix Mul | `benchmarks/03_matrix_mul.vp` | 6ms | 2ms | P0 |
| Quicksort | `benchmarks/04_quicksort.vp` | 1ms | 0.5ms | P1 |
| List Ops | `benchmarks/05_list_ops.vp` | TBD | TBD | P0 |
| String Ops | `benchmarks/06_string_ops.vp` | 2ms | 1ms | P2 |

### Benchmark Commands
```bash
# Run all benchmarks
make benchmarks

# Run single benchmark
./target/release/viper run benchmarks/01_fibonacci.vp

# Compare with C
make benchmark_compare
```

---

## Testing Strategy

### Unit Tests
Each optimization pass has dedicated tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_inline_small_function() {
        // Verify functions < 10 statements get inlined
    }
    
    #[test]
    fn test_pure_function_attributes() {
        // Verify readonly + willreturn added to pure functions
    }
    
    #[test]
    fn test_constant_fold_arithmetic() {
        // Verify 2 + 3 → 5 at compile time
    }
}
```

### Integration Tests
```rust
#[test]
fn test_fibonacci_optimized() {
    let code = r#"
        def fib(n: int) -> int:
            if n <= 1: return n
            return fib(n-1) + fib(n-2)
    "#;
    
    let result = compile_and_run(code);
    assert_eq!(result, "55");  // fib(10)
    assert_execution_time("<10ms");
}
```

### Regression Tests
- Run full test suite (434+ tests) after each optimization
- Verify no correctness regressions
- Track performance regressions with CI

---

## Risk Mitigation

### Low-Risk Optimizations
| Optimization | Why Low Risk |
|--------------|--------------|
| Inlining | Well-understood, LLVM handles edge cases |
| Purity attributes | Declarative, doesn't change semantics |
| Constant folding | Local transformations, easy to verify |

### Medium-Risk Optimizations
| Optimization | Mitigation |
|--------------|------------|
| LICM | Extensive testing with loop benchmarks |
| Type specialization | Fallback to generic version if issues |
| PGO | Keep non-PGO build as fallback |

### High-Risk Optimizations
| Optimization | Mitigation |
|--------------|------------|
| SIMD | Optional, runtime detection |
| Parallelization | Extensive concurrency testing |

---

## Success Metrics

### Performance Goals

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Fibonacci | 71ms | 25ms | `make bench_fib` |
| Prime Sieve | 4ms | 1.5ms | `make bench_prime` |
| Matrix Mul | 6ms | 2ms | `make bench_matrix` |
| Overall gap to C | 5x | 2x | Geometric mean |

### Compile Time Goals

| Optimization Level | Max Overhead |
|--------------------|--------------|
| -O1 | < 20% |
| -O2 | < 50% |
| -O3 | < 100% |

### Correctness Goals

- Zero test failures (434+ tests)
- Zero memory leaks (valgrind clean)
- Zero undefined behavior (UBSan clean)

---

## Implementation Timeline

```
Week 1-2: Phase 1 - Complete Existing
├── Function inlining completion
├── Purity analysis enhancement
└── Direct call optimization

Week 3-5: Phase 2 - New Optimizations
├── Constant folding
├── Inline list ops enhancement
└── Loop invariant code motion

Week 6-8: Phase 3 - Advanced
├── PGO integration
├── Type specialization
└── Escape analysis enhancement

Week 9-12: Phase 4 - Platform-Specific
├── SIMD vectorization
└── Cache-aware optimizations
```

---

## Files Summary

### Files to Create
| File | Purpose | Phase |
|------|---------|-------|
| `src/semantic/constant_folding.rs` | Constant folding | 2 |
| `src/semantic/purity_analysis.rs` | Call-graph purity | 1 |
| `src/codegen/loop_analysis.rs` | Loop detection | 2 |
| `src/codegen/licm.rs` | Loop invariant motion | 2 |
| `src/codegen/type_specialization.rs` | Type specialization | 3 |
| `scripts/pgo_*.sh` | PGO workflow | 3 |

### Files to Modify
| File | Changes | Phase |
|------|---------|-------|
| `src/codegen/core/functions.rs` | Inlining, purity | 1 |
| `src/codegen/expressions/calls/dispatch.rs` | Direct calls | 1 |
| `src/codegen/inline_lists.rs` | Extend coverage | 2 |
| `src/semantic/escape_analysis.rs` | Enhance analysis | 3 |
| `src/driver/aot.rs` | Pipeline integration | 1-3 |
| `src/driver/jit.rs` | Pipeline integration | 1-3 |

---

## References

- LLVM Passes: https://llvm.org/docs/Passes.html
- LLVM Language Reference: https://llvm.org/docs/LangRef.html
- Engineering a Compiler (Cooper & Torczon)
- Existing: `OPTIMIZATION_PLAN.md`, `PERFORMANCE_ANALYSIS.md`

---

*Last Updated: March 12, 2026*  
*Version: 0.5.0*  
*Author: Viper Language Team*
