# Viper Language Test Report

**Date:** March 8, 2026  
**Version:** 0.4.7  
**Compiler:** JIT -O2 mode

---

## Executive Summary

| Category | Status |
|----------|--------|
| Core Language Features | ⚠️ Partial |
| Python Compatibility | ⚠️ Partial |
| Standard Library | ❌ Needs Work |
| Testing Framework | ❌ Not Functional |

---

## Test Results Summary

### ✅ Passing Tests (13)

| Test File | Description |
|-----------|-------------|
| `test_global_simple.vp` | Global keyword support |
| `test_literal.vp` | BigInt literal support |
| `test_nonlocal_simple.vp` | Nonlocal keyword (basic) |
| `test_math_simple.vp` | Math functions (gcd, lcm, factorial, isqrt) |
| `string_func_test.vp` | String functions |
| `jit_test.vp` | JIT execution |
| `fib_test.vp` | Fibonacci with BigInt |
| `fib_python_style.vp` | Python-style Fibonacci |
| `comprehensive_oop.vp` | OOP (diamond, MRO, multi-level inheritance) |
| `overload_test.vp` | Function overloading |
| `test_isinstance.vp` | isinstance() builtin |
| `test_neg.vp` | Negation operator |
| `test_factorial_loop.vp` | Factorial with loops |

### ❌ Failing Tests (Categorized by Issue)

#### 1. Parser Issues

| Test File | Error |
|-----------|-------|
| `test_dataclass.vp` | Expected Def, found Class at line 4 |
| `test_iterator.vp` | Expected Colon, found Indent at line 72 |
| `test_typing.vp` | Expected identifier, found Optional at line 3 |
| `generics.vp` | Expected RParen, found Arrow at line 4 |
| `result_type.vp` | Expected RBracket, found Colon at line 7 |
| `test_loop_else.vp` | Unexpected token in expression: Indent |
| `test_exceptions.vp` | Expected Colon, found Ident at line 8 |

#### 2. Type Checker Issues

| Test File | Error |
|-----------|-------|
| `test_walrus.vp` | Arithmetic operators require numeric types, got str and int |
| `test_nonlocal.vp` | Arithmetic operators require numeric types, got str and int |
| `test_with.vp` | Failed to load module 'io': Inconsistent indentation |
| `test_collections.vp` | Failed to parse module 'collections': Unexpected token |
| `generic_types.vp` | Cannot assign {str: int} to {str: int} |
| `result_simple.vp` | Return type mismatch: Result[int, str] |
| `minimal_bigint.vp` | Undefined function 'str_int' |
| `bigint_literal_test.vp` | Undefined function 'str_int' |
| `nested_call_test.vp` | Undefined function 'str_int' |
| `test_abs.vp` | Undefined function 'abs_int' |
| `test_sqrt.vp` | Undefined function 'sqrt_int' |
| `test_import.vp` | Module 'test_module' not found |

#### 3. Codegen Issues

| Test File | Error |
|-----------|-------|
| `simple_oop.vp` | Binary operators cannot be applied to pointer values (lists) |
| `bigint_ops_test.vp` | Unsupported pointer operator: Sub |
| `test_while.vp` | Unsupported pointer operator: Mul |
| `test_stdlib_phase2.vp` | Unexpected token in expression: Eq |
| `test_math_builtins.vp` | Undefined variable: math |

#### 4. Runtime Issues (Segfaults)

| Test File | Issue |
|-----------|-------|
| `super_test.vp` | Segmentation fault (core dumped) |

---

## Phase-by-Phase Analysis

### Phase 1: Foundation Tests

| Feature | Status | Notes |
|---------|--------|-------|
| Walrus Operator (`:=`) | ⚠️ Partial | Basic support works, type checker issues in complex cases |
| Nonlocal Keyword | ✅ Working | Basic and simple nested cases pass |
| Context Managers (`with`) | ❌ Broken | std/io.vp has indentation parsing issue |
| Exception Chaining | ❌ Broken | Parser doesn't support `raise ... from ...` syntax |

### Phase 2: Python Parity Tests

| Feature | Status | Notes |
|---------|--------|-------|
| @dataclass Decorator | ❌ Broken | Parser issue with class syntax |
| Iterator Protocol | ❌ Broken | Parser issue with indentation |
| Typing Module | ❌ Broken | Parser doesn't recognize Optional, etc. |
| Functools Module | Not Tested | Depends on typing module |
| Itertools Module | Not Tested | Depends on typing module |
| Collections Module | ❌ Broken | std/collections.vp has parser issues |

### Phase 3: Standard Library Tests

| Module | Status | Notes |
|--------|--------|-------|
| csv | Not Tested | Depends on stdlib loading |
| datetime | Not Tested | Depends on stdlib loading |
| string | Not Tested | Depends on stdlib loading |
| contextlib | Not Tested | Depends on stdlib loading |
| pathlib | Not Tested | Depends on stdlib loading |
| io | ❌ Broken | Indentation parsing error at line 3 |

### Phase 4: Testing Tools Tests

| Tool | Status | Notes |
|------|--------|-------|
| unittest | Not Tested | Depends on stdlib loading |
| mock | Not Tested | Depends on unittest |
| coverage | Not Tested | Depends on stdlib loading |
| debugger (pdb) | Not Tested | Interactive, depends on stdlib |

---

## Critical Issues Identified

### 1. Standard Library Parsing Failures

**Issue:** Multiple stdlib modules fail to parse due to indentation/tokenization errors.

**Affected Files:**
- `std/io.vp` - "Inconsistent indentation: expected 0, got 4"
- `std/collections.vp` - "Unexpected token in expression: Tuple"

**Impact:** Blocks all stdlib-dependent tests and features.

### 2. Type Checker False Positives

**Issue:** Type checker reports errors for valid type assignments.

**Examples:**
- "Cannot assign {str: int} to {str: int}"
- "Return type mismatch: expected Result[int, str], got Result[int, str]"
- "Arithmetic operators require numeric types, got str and int" (in print statements)

**Impact:** Prevents valid code from compiling.

### 3. Missing Builtin Functions

**Issue:** Several builtin functions are undefined.

**Missing Functions:**
- `str_int` - String to int conversion
- `abs_int` - Absolute value for int
- `sqrt_int` - Square root for int
- `exit` - Program exit

### 4. Codegen Pointer Operator Issues

**Issue:** "Unsupported pointer operator" errors for basic operations.

**Examples:**
- "Unsupported pointer operator: Sub"
- "Unsupported pointer operator: Mul"
- "Binary operators cannot be applied to pointer values (lists)"

**Impact:** Breaks list operations and arithmetic in certain contexts.

### 5. Runtime Segfaults

**Issue:** Segmentation faults in JIT mode.

**Affected:**
- `super_test.vp` - Segfault during super() call
- Related to BUG_JIT_NAME_MAIN_SEGFAULT.md issue

---

## Recommendations

### Immediate (P0)

1. **Fix std/io.vp indentation** - This blocks context managers and many stdlib features
2. **Fix std/collections.vp parser** - Tuple parsing issue
3. **Add missing builtins** - `str_int`, `abs_int`, `sqrt_int`, `exit`
4. **Fix type checker equality** - Same types should be assignable

### Short-term (P1)

1. **Fix parser for advanced syntax:**
   - `raise ... from ...` (exception chaining)
   - `@dataclass` decorator
   - Type hints with Optional, List, Dict
   - Loop-else syntax

2. **Fix codegen pointer operators:**
   - Support Sub and Mul for pointer types
   - Fix list binary operations

3. **Investigate segfaults:**
   - `super_test.vp` segfault
   - Related to JIT name/main issue

### Medium-term (P2)

1. **Enable full stdlib:**
   - csv, datetime, string, pathlib
   - functools, itertools

2. **Testing framework:**
   - unittest framework
   - mock framework
   - coverage tool

---

## Test Commands Used

```bash
# Build
cargo build --release

# Run individual tests
./target/release/viper run tests/<test_file>.vp

# Example passing test
./target/release/viper run tests/fib_test.vp

# Example failing test
./target/release/viper run tests/test_walrus.vp
```

---

## Conclusion

The Viper compiler has solid foundations with working:
- BigInt support
- Basic OOP (inheritance, MRO)
- Function overloading
- Core math functions
- isinstance() and basic builtins

However, significant work is needed on:
- Standard library parsing/loading
- Type checker accuracy
- Parser support for Python syntax
- Codegen pointer operations
- JIT runtime stability

**Estimated Test Coverage:** ~30% of planned features are functional.
