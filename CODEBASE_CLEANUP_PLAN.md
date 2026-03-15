# Viper Language Codebase Cleanup Plan

**Generated**: March 6, 2026  
**Total Estimated Effort**: 4-5 weeks (19-24 days)  
**Current State**: ~41K lines of Rust code, 139 source files

---

## Executive Summary

The Viper compiler codebase shows signs of rapid iterative development with several architectural issues:

- **3 critical files** >2000 lines requiring immediate splitting
- **5 high-priority files** >800 lines needing refactoring
- **18+ occurrences** of duplicated code patterns
- **231 debug prints** in production code
- **9+ TODO/FIXME** comments indicating incomplete features
- **4 files** with `#[allow(dead_code)]` suppressing warnings
- **~50% feature completion** for Phase 3 (Concurrency + OOP)

---

## 1. Large Files Analysis

### Critical Priority (>2000 lines)

| File | Lines | Domain | Action |
|------|-------|--------|--------|
| `src/codegen/expressions/calls.rs` | 2,810 | Function/method call codegen | **SPLIT IMMEDIATELY** |
| `src/jit_stubs/registry.rs` | 2,549 | JIT stub registration | **SPLIT + MACRO** |

### High Priority (>800 lines)

| File | Lines | Domain | Action |
|------|-------|--------|--------|
| `src/codegen/expressions/collections.rs` | 1,167 | Collection codegen | Split by type |
| `src/codegen/statements/core.rs` | 1,099 | Statement codegen | Split by category |
| `src/codegen/generator.rs` | 1,090 | Main orchestrator | Acceptable but complex |
| `src/parser/statements/primary.rs` | 995 | Expression parsing | Move to expressions/ |
| `src/codegen/oop/classes.rs` | 936 | OOP class codegen | Well-organized |
| `src/lexer/scanner.rs` | 896 | Lexical analysis | Acceptable |
| `src/codegen/expressions/builtins.rs` | 820 | Builtin codegen | Split by category |
| `src/codegen/control_flow/loops.rs` | 814 | Loop codegen | Split by loop type |

---

## 2. Detailed Refactoring Plans

### 2.1 Split `calls.rs` (2,810 lines) ⭐⭐⭐

**Current**: Single monolithic file handling all function calls  
**Problem**: Mixes call dispatch, builtin routing, method calls, and collection constructors  
**Location**: `src/codegen/expressions/calls.rs`

**New Structure**:
```
src/codegen/expressions/calls/
├── mod.rs              # Re-export all submodules
├── dispatch.rs         # Main generate_call() dispatch (lines 1-500)
├── builtins_io.rs      # print, input (lines 120-250)
├── builtins_types.rs   # len, type, isinstance, id (lines 250-400)
├── builtins_conv.rs    # str, int, float, bool (lines 400-600)
├── math.rs             # Math BigInt functions (lines 500-700)
├── methods.rs          # Method call handling (lines 1000-1500)
├── collections.rs      # list, tuple, set, dict (lines 1600-1800)
└── functional.rs       # sum, min, max, any, all (lines 2000-2500)
```

**Dead Code to Remove**:
- Lines 160-180: Deprecated BigInt functions (commented out but still there)
- Lines 437-450: Commented-out closure cell code

**Steps**:
1. Create directory structure
2. Extract each section to separate file
3. Create mod.rs with re-exports
4. Update imports in parent modules
5. Run tests to verify

---

### 2.2 Refactor `registry.rs` (2,549 lines) ⭐⭐⭐

**Current**: Repetitive registration code for 100+ JIT stub functions  
**Problem**: Identical pattern repeated 100+ times  
**Location**: `src/jit_stubs/registry.rs`

**Solution**: Create registration macros

**Before**:
```rust
if let Some(func) = module.get_function("vp_list_create") {
    execution_engine.add_global_mapping(&func.as_global_value(), vp_list_create_stub as *const () as usize);
}
if let Some(func) = module.get_function("vp_list_append") {
    execution_engine.add_global_mapping(&func.as_global_value(), vp_list_append_stub as *const () as usize);
}
// ... repeated 100+ times
```

**After**:
```rust
// Macro definition in registry/mod.rs
macro_rules! register_stub {
    ($ee:expr, $mod:expr, $func:ident, $stub:ident) => {
        if let Some(func) = $mod.get_function(stringify!($func)) {
            $ee.add_global_mapping(&func.as_global_value(), $stub as *const () as usize);
        }
    };
}

// Usage - much cleaner
register_stub!(execution_engine, module, vp_list_create, vp_list_create_stub);
register_stub!(execution_engine, module, vp_list_append, vp_list_append_stub);
register_stub_group!(execution_engine, module, [
    (vp_list_get, vp_list_get_stub),
    (vp_list_set, vp_list_set_stub),
    (vp_list_len, vp_list_len_stub),
]);
```

**New Structure**:
```
src/jit_stubs/registry/
├── mod.rs           # Macros and main registry function
├── lists.rs         # List stub registrations
├── bitvec.rs        # Bit vector registrations
├── strings.rs       # String stub registrations
├── bigint.rs        # BigInt stub registrations
├── tagged_int.rs    # Tagged int registrations
├── collections.rs   # Dict/set/tuple registrations
└── concurrency.rs   # Channel/task registrations
```

---

### 2.3 Split `collections.rs` (1,167 lines) ⭐⭐

**Location**: `src/codegen/expressions/collections.rs`

**New Structure**:
```
src/codegen/expressions/collections/
├── mod.rs       # Re-exports
├── lists.rs     # List creation, comprehension (lines 1-200)
├── dicts.rs     # Dict creation (lines 200-400)
├── arrays.rs    # Array creation (lines 400-550)
├── index.rs     # Index access logic (lines 550-800)
└── slice.rs     # Slice operations (lines 800-1167)
```

**Issue Found**: Line 603 has disabled inline operations with comment:
> "Inline operations disabled due to JIT/AOT struct layout differences"

**Action**: Either fix or remove the disabled code.

---

### 2.4 Split `statements/core.rs` (1,099 lines) ⭐⭐

**Location**: `src/codegen/statements/core.rs`

**New Structure**:
```
src/codegen/statements/
├── mod.rs
├── dispatch.rs      # Main statement dispatch (lines 1-200)
├── exceptions.rs    # Exception handling (lines 200-400)
├── patterns.rs      # Match statements (lines 400-600)
├── context.rs       # With statements (lines 600-800)
└── concurrency.rs   # Concurrency statements (lines 800-1099)
```

**Incomplete Implementations** (TODO comments):
- Line 317: `Stmt::Assert` - needs runtime panic
- Line 324: `Stmt::Delete` - needs implementation
- Line 344: `Stmt::Yield` - generator yield (Phase 3 feature)
- Line 618: `generate_try_except` - simplified implementation

---

### 2.5 Split `builtins.rs` (820 lines) ⭐⭐

**Location**: `src/codegen/expressions/builtins.rs`

**New Structure**:
```
src/codegen/expressions/builtins/
├── mod.rs     # Re-exports
├── io.rs      # print, input
├── inspect.rs # len, type, id, repr
├── convert.rs # str, int, float, bool
└── misc.rs    # Other builtins
```

---

## 3. Code Quality Issues

### 3.1 Dead Code to Remove

| Location | Lines | Description |
|----------|-------|-------------|
| `codegen/expressions/calls.rs` | 160-180 | Deprecated BigInt functions |
| `codegen/expressions/calls.rs` | 437-450 | Commented closure cell code |
| `codegen/generator.rs` | 99 | Commented closure runtime code |
| `codegen/control_flow/loops.rs` | 547 | Debug println statements |
| Multiple files | Various | `#![allow(dead_code)]` attributes |

**Action**: Run `cargo clippy -- -W dead_code` to identify all instances, then remove.

---

### 3.2 Duplicated Code Patterns

#### Pattern 1: Function Parent Retrieval (18 occurrences)

**Found in**:
- `src/codegen/statements/assignment.rs:362`
- `src/codegen/control_flow/conditional.rs:12,110`
- `src/codegen/control_flow/core.rs:15,25`
- `src/codegen/statements/declaration.rs:201,339`
- `src/codegen/expressions/operators/incdec.rs:89`
- `src/codegen/expressions/operators/logical.rs:18,62`
- `src/codegen/control_flow/loops.rs:28,128,258,419,595`
- `src/codegen/statements/core.rs:245,705,913`

**Current Code**:
```rust
let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
```

**Fix**: Add helper method to `CodeGenState`:
```rust
impl<'a, 'ctx> CodeGenState<'a, 'ctx> {
    pub fn current_function(&self) -> Result<FunctionValue<'ctx>, String> {
        self.builder.get_insert_block()
            .ok_or("No insertion block".to_string())?
            .get_parent()
            .ok_or("No parent function".to_string())
    }
}
```

---

#### Pattern 2: List Type Checking (15+ occurrences)

**Current Code**:
```rust
let is_list = match obj {
    Expr::Ident(name, _) => state.is_list(name),
    Expr::List { .. } | Expr::ListComprehension { .. } => true,
    _ => false,
};
```

**Fix**: Consolidate into single method on `CodeGenState`.

---

#### Pattern 3: Bool List Detection (8+ occurrences)

**Current Code**:
```rust
let is_bool_list = match obj {
    Expr::Ident(obj_name, _) => state.is_bool_list(obj_name),
    Expr::List { elements, .. } => elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false),
    _ => false,
};
```

**Fix**: Consolidate into helper method.

---

#### Pattern 4: Runtime Function Retrieval (100+ occurrences)

**Current Code**:
```rust
let func = state.module.get_function("vp_list_get")
    .ok_or_else(|| "vp_list_get not declared".to_string())?;
```

**Fix**: Add helper with better error messages:
```rust
impl<'a, 'ctx> CodeGenState<'a, 'ctx> {
    pub fn get_runtime_function(&self, name: &str) -> Result<FunctionValue<'ctx>, String> {
        self.module.get_function(name)
            .ok_or_else(|| format!("Runtime function '{}' not declared", name))
    }
}
```

---

### 3.3 Debug Code in Production

**Found**: 231 `println!`/`eprintln!`/`dbg!` calls

**Problematic Examples**:
```rust
// src/codegen/control_flow/loops.rs:547
println!("ITER_TYPE: {:?}, iter: {:?}", iter_type, iter);
if let Expr::Ident(name, _) = iter {
    println!("In var_types: {:?}", state.var_types.get(name));
}
```

**Action**:
1. Remove all debug prints from production code
2. Replace with `log` crate for necessary logging
3. Add `log` dependency to `Cargo.toml`
4. Use `env_logger` for CLI output control

---

### 3.4 Files with Suppressed Warnings

| File | Line | Warning Type |
|------|------|--------------|
| `src/lexer/scanner.rs` | 1 | `#![allow(dead_code)]` (entire module) |
| `src/semantic/type_checker/mod.rs` | 40 | `#[allow(dead_code)]` |
| `src/driver/utils.rs` | 75 | `#[allow(dead_code)]` |
| `src/driver/aot.rs` | 10 | `#[allow(dead_code)]` |
| `src/codegen/generator.rs` | 1085 | `#[allow(dead_code)]` |

**Action**: Remove suppressed code instead of silencing warnings.

---

## 4. Incomplete Features

### 4.1 Partially Implemented (Decide: Complete or Remove)

| Feature | Location | Status | Decision Needed |
|---------|----------|--------|-----------------|
| **Closures** | `semantic/closure_analysis.rs`, `codegen/expressions/calls.rs:437` | Analysis done, codegen disabled | Complete or remove |
| **Generators** | `codegen/statements/core.rs:344` | TODO comment | Complete or remove |
| **Monomorphization** | `semantic/monomorphization.rs:209` | `stmt.clone()  // TODO` | Complete or remove |
| **Exception Handling** | `codegen/statements/core.rs:618` | Simplified implementation | Complete LLVM integration |
| **DCE** | `codegen/dce.rs` | Implemented but not used | Integrate or remove |
| **Inline Lists** | `codegen/inline_lists.rs` | Disabled (line 603) | Fix or remove |

---

### 4.2 Documentation vs Implementation Gaps

| Feature | Documented | Actual | Gap |
|---------|------------|--------|-----|
| **Sets** | Phase 2 (full operations) | Not implemented | No `Expr::Set` in AST |
| **BigInt/Int** | `int` is arbitrary precision | Both `Expr::Int` and `Expr::BigInt` exist | Confused implementation |
| **Pattern Matching** | Phase 3 (full match/case) | Basic stub only | No pattern support |
| **Decorators** | Phase 3 (full support) | Only 3 decorators | Missing `@lru_cache`, etc. |
| **Generic Types** | Phase 3 | Monomorphization stub | Not functional |
| **Async/Await** | Phase 3 | Basic implementation | Missing `async for`, `async with` |

**Action**: Update `CORE_LANGUAGE_FEATURES.md` to reflect actual status.

---

## 5. Recommended Module Structure

### Final Target Architecture

```
src/
├── lib.rs
├── main.rs
├── error.rs
│
├── ast/                          # ✅ KEEP AS-IS
│   ├── mod.rs
│   ├── nodes.rs                  # Expr, Stmt enums
│   └── types.rs                  # Type enum
│
├── lexer/                        # ✅ KEEP AS-IS
│   ├── mod.rs
│   ├── scanner.rs
│   ├── tokens.rs
│   └── indent_stack.rs
│
├── parser/                       # 🔄 REORGANIZE
│   ├── mod.rs
│   ├── lexer_bridge.rs           # Token handling utilities
│   ├── expressions/              # NEW
│   │   ├── mod.rs
│   │   ├── pratt.rs              # Current expressions.rs
│   │   ├── primary.rs            # Move from statements/primary.rs
│   │   └── operators.rs
│   ├── statements/               # 🔄 SPLIT
│   │   ├── mod.rs
│   │   ├── simple.rs             # Assignments, declarations
│   │   ├── control.rs            # If, while, for, match
│   │   ├── definitions.rs        # Function, class definitions
│   │   └── concurrency.rs        # sync, task, channels
│   └── precedence.rs
│
├── semantic/                     # 🔄 REORGANIZE
│   ├── mod.rs
│   ├── analysis/                 # NEW
│   │   ├── mod.rs
│   │   ├── escape.rs             # Move from escape_analysis.rs
│   │   └── closure.rs            # Move from closure_analysis.rs
│   ├── types/                    # NEW
│   │   ├── mod.rs
│   │   ├── checker.rs            # Type checking logic
│   │   ├── inference.rs          # Type inference
│   │   └── compatibility.rs      # Type compatibility
│   ├── symbols/                  # NEW
│   │   ├── mod.rs
│   │   └── table.rs              # Symbol table
│   └── generics/                 # NEW
│       └── monomorph.rs          # Monomorphization (complete or remove)
│
├── codegen/                      # 🔄 MAJOR REORGANIZATION
│   ├── mod.rs
│   ├── core/                     # NEW
│   │   ├── mod.rs
│   │   ├── generator.rs          # Main CodeGen struct
│   │   ├── state.rs              # CodeGenState
│   │   └── builder_ext.rs        # IRBuilder extensions
│   ├── expressions/              # 🔄 SPLIT
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   ├── calls/                # ⭐ NEW (split from calls.rs)
│   │   │   ├── mod.rs
│   │   │   ├── dispatch.rs
│   │   │   ├── builtins_io.rs
│   │   │   ├── builtins_types.rs
│   │   │   ├── builtins_conv.rs
│   │   │   ├── math.rs
│   │   │   ├── methods.rs
│   │   │   └── functional.rs
│   │   ├── builtins/             # ⭐ NEW (split from builtins.rs)
│   │   │   ├── mod.rs
│   │   │   ├── io.rs
│   │   │   ├── inspect.rs
│   │   │   ├── convert.rs
│   │   │   └── misc.rs
│   │   ├── collections/          # ⭐ NEW (split from collections.rs)
│   │   │   ├── mod.rs
│   │   │   ├── lists.rs
│   │   │   ├── dicts.rs
│   │   │   ├── arrays.rs
│   │   │   ├── index.rs
│   │   │   └── slice.rs
│   │   └── operators/
│   │       ├── mod.rs
│   │       ├── arithmetic.rs
│   │       ├── comparison.rs
│   │       ├── logical.rs
│   │       └── bitwise.rs
│   ├── statements/               # 🔄 SPLIT
│   │   ├── mod.rs
│   │   ├── dispatch.rs
│   │   ├── declarations.rs
│   │   ├── assignments.rs
│   │   ├── control_flow.rs
│   │   ├── exceptions.rs
│   │   └── concurrency.rs
│   ├── oop/
│   │   └── classes.rs
│   ├── runtime/                  # Runtime function declarations
│   │   ├── mod.rs
│   │   ├── lists.rs
│   │   ├── dicts.rs
│   │   ├── strings.rs
│   │   ├── bigint.rs
│   │   └── concurrency.rs
│   └── optimization/             # NEW
│       ├── mod.rs
│       ├── dce.rs                # Move from dce.rs (integrate or remove)
│       └── inline.rs             # Move from inline_lists.rs (fix or remove)
│
├── jit_stubs/                    # 🔄 MAJOR REORGANIZATION
│   ├── mod.rs
│   ├── registry/                 # ⭐ NEW (split from registry.rs)
│   │   ├── mod.rs                # With macros
│   │   ├── lists.rs
│   │   ├── strings.rs
│   │   ├── bigint.rs
│   │   └── collections.rs
│   ├── core/
│   │   ├── memory.rs
│   │   ├── gc.rs
│   │   └── tagged_int.rs
│   ├── collections/
│   │   ├── lists.rs
│   │   ├── dicts.rs
│   │   ├── sets.rs               # Create if implementing sets
│   │   └── bitvec.rs
│   ├── strings/
│   │   ├── strings.rs
│   │   └── format.rs
│   ├── math/
│   │   ├── bigint.rs
│   │   └── math.rs
│   ├── concurrency/
│   │   ├── channels.rs
│   │   └── tasks.rs
│   └── stdlib/
│       ├── os.rs
│       ├── sys.rs
│       ├── time.rs
│       └── ...
│
├── driver/                       # ✅ KEEP AS-IS
│   ├── mod.rs
│   ├── aot.rs
│   ├── jit.rs
│   └── utils.rs
│
├── cli/                          # ✅ KEEP AS-IS
├── repl/                         # ✅ KEEP AS-IS
├── lsp/                          # ✅ KEEP AS-IS
└── vpm/                          # ✅ KEEP AS-IS
```

---

## 6. Implementation Timeline

### Week 1-2: Critical Refactoring

**Day 1-2**: Dead Code Removal
- [ ] Remove commented-out closure code (calls.rs:437, generator.rs:99)
- [ ] Remove deprecated BigInt functions (calls.rs:160-180)
- [ ] Remove debug prints (search for `println!`, `dbg!`)
- [ ] Remove `#[allow(dead_code)]` attributes and fix underlying issues
- [ ] Run `cargo clippy -- -W dead_code` and fix all warnings

**Day 3-5**: Split `calls.rs`
- [ ] Create directory structure
- [ ] Extract dispatch logic to `dispatch.rs`
- [ ] Extract builtin functions to submodules
- [ ] Extract method call handling
- [ ] Update all imports
- [ ] Run tests

**Day 6-8**: Refactor `registry.rs`
- [ ] Create registration macros
- [ ] Split by domain (lists, strings, bigint, etc.)
- [ ] Test JIT mode thoroughly

**Day 9-10**: Consolidate Duplicated Patterns
- [ ] Add `CodeGenState::current_function()` helper
- [ ] Add `CodeGenState::is_collection_type()` helper
- [ ] Add `CodeGenState::get_runtime_function()` helper
- [ ] Replace all occurrences
- [ ] Test

---

### Week 3-4: High Priority Refactoring

**Day 11-12**: Split `collections.rs`
- [ ] Create directory structure
- [ ] Split by collection type
- [ ] Fix or remove disabled inline operations

**Day 13-14**: Split `statements/core.rs`
- [ ] Create directory structure
- [ ] Split by statement category
- [ ] Address TODO comments (decide: implement or remove)

**Day 15-16**: Split `builtins.rs`
- [ ] Create directory structure
- [ ] Split by builtin category

**Day 17-18**: Reorganize `jit_stubs/`
- [ ] Create new directory structure
- [ ] Group by domain
- [ ] Fix naming inconsistencies (`_mod` suffix)

**Day 19-20**: Add Logging Infrastructure
- [ ] Add `log` and `env_logger` to `Cargo.toml`
- [ ] Replace remaining `println!` with `log` macros
- [ ] Add logging configuration to CLI

---

### Week 5: Medium Priority

**Day 21-22**: Reorganize `parser/`
- [ ] Move primary expressions to `expressions/`
- [ ] Split statements by category

**Day 23-24**: Reorganize `semantic/`
- [ ] Group analysis modules
- [ ] Split type checker
- [ ] Fix or remove monomorphization

**Day 25**: Update Documentation
- [ ] Update `CORE_LANGUAGE_FEATURES.md` with actual status
- [ ] Document incomplete features
- [ ] Create architecture decision records

---

### Week 6: Decision Points

**Day 26-28**: Incomplete Features
- [ ] **Closures**: Complete implementation or remove
- [ ] **Generators**: Complete implementation or remove
- [ ] **Monomorphization**: Complete implementation or remove
- [ ] **DCE**: Integrate into pipeline or remove
- [ ] **Inline Lists**: Fix struct layout issue or remove

**Day 29-30**: Final Cleanup
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Run `cargo fmt`
- [ ] Update README with new structure
- [ ] Create migration guide for contributors

---

## 7. Testing Strategy

### Before Each Refactoring

1. **Run full test suite**: `cargo test`
2. **Build release**: `cargo build --release`
3. **Test interpreter**: Run sample Viper programs
4. **Test JIT mode**: Run programs with JIT
5. **Snapshot git**: Create branch before major changes

### After Each Refactoring

1. **Compile check**: `cargo check`
2. **Run tests**: `cargo test`
3. **Build release**: `cargo build --release`
4. **Integration tests**: Run Viper test programs
5. **Compare binaries**: Ensure no behavioral changes

### Test Commands

```bash
# Full test suite
cargo test --all

# Build release
cargo build --release

# Clippy (after cleanup)
cargo clippy -- -W dead_code -W unused_imports

# Format check
cargo fmt -- --check

# Integration tests (if available)
./run_python_tests.sh  # Or equivalent Viper test runner
```

---

## 8. Risk Mitigation

### High Risk Changes

| Change | Risk | Mitigation |
|--------|------|------------|
| Splitting `calls.rs` | Medium | Small commits, frequent tests |
| Refactoring `registry.rs` | High | Keep old code commented initially |
| Removing dead code | Medium | Git backup, incremental removal |
| Incomplete features | High | Document decisions clearly |

### Rollback Plan

1. **Create feature branch** for each major change
2. **Commit frequently** (every 30-60 minutes)
3. **Test after each commit**
4. **Keep previous version** in separate directory
5. **Document all changes** in commit messages

---

## 9. Success Metrics

### Code Quality Metrics

| Metric | Before | Target | After |
|--------|--------|--------|-------|
| Largest file (lines) | 2,810 | <1,000 | TBD |
| Files >1000 lines | 5 | 0 | TBD |
| `#[allow(dead_code)]` | 4 | 0 | TBD |
| Debug prints | 231 | <10 | TBD |
| Duplicated patterns | 18+ | 0 | TBD |
| TODO comments | 9 | <5 | TBD |

### Maintainability Metrics

| Metric | Before | Target |
|--------|--------|--------|
| Average file size | ~300 lines | <200 lines |
| Module depth | 2-3 levels | 3-4 levels |
| Public API surface | Large | Minimal |
| Test coverage | Unknown | >70% |

---

## 10. Quick Start

### Immediate Actions (Day 1)

```bash
# 1. Create backup branch
git checkout -b cleanup-backup-$(date +%Y%m%d)

# 2. Identify dead code
cargo clippy -- -W dead_code -W unused_imports

# 3. Find all debug prints
rg 'println!|eprintln!|dbg!' -t rs

# 4. Find all TODO comments
rg 'TODO|FIXME|XXX' -t rs

# 5. Create cleanup branch
git checkout -b cleanup-phase1
```

### First Refactoring (Example: Remove Dead Code)

```bash
# 1. Remove commented BigInt functions
# Edit: src/codegen/expressions/calls.rs lines 160-180

# 2. Remove debug prints
# Edit: src/codegen/control_flow/loops.rs line 547

# 3. Test
cargo check
cargo test

# 4. Commit
git add -A
git commit -m "refactor: remove dead BigInt and debug code"
```

---

## 11. Appendix: File Size Reference

### Complete File Size List (Top 30)

```
   2809 src/codegen/expressions/calls.rs
   2548 src/jit_stubs/registry.rs
   1166 src/codegen/expressions/collections.rs
   1098 src/codegen/statements/core.rs
   1089 src/codegen/generator.rs
    994 src/parser/statements/primary.rs
    935 src/codegen/oop/classes.rs
    895 src/lexer/scanner.rs
    819 src/codegen/expressions/builtins.rs
    813 src/codegen/control_flow/loops.rs
    801 src/parser/expressions.rs
    789 src/codegen/functions.rs
    744 src/codegen/statements/assignment.rs
    708 src/semantic/escape_analysis.rs
    693 src/repl/session.rs
    687 src/semantic/type_checker/stmts.rs
    623 src/jit_stubs/bitvec.rs
    617 src/semantic/type_checker/hindley_milner.rs
    565 src/vpm/cli/commands.rs
    556 src/semantic/closure_analysis.rs
    556 src/codegen/expressions/operators/mod.rs
    544 src/codegen/dce.rs
    543 src/semantic/symbol_table.rs
    503 src/jit_stubs/collections.rs
    500 src/codegen/expressions/core.rs
    493 src/parser/statements/definitions.rs
    476 src/parser/statements/control_flow.rs
    470 src/driver/aot.rs
    457 src/codegen/inline_lists.rs
```

---

## 12. Contact & Questions

For questions about this cleanup plan:
1. Check existing issues in repository
2. Review `CORE_LANGUAGE_FEATURES.md` for feature context
3. Consult `BUG_JIT_NAME_MAIN_SEGFAULT.md` for known issues
4. Review recent commit messages for context

---

**Last Updated**: March 6, 2026  
**Version**: 1.0  
**Status**: Ready for implementation
