# Viper Test Status Report

**Date:** March 8, 2026  
**Version:** 0.5.0

---

## Summary

**No, not all tests pass.** Here's the current status:

---

## ✅ Passing Tests (~20)

### Core Language Features
- `test_walrus.vp` - Walrus operator
- `test_global_simple.vp` - Global keyword
- `test_nonlocal_simple.vp` - Nonlocal keyword (basic)
- `test_isinstance.vp` - isinstance() builtin
- `test_neg.vp` - Negation operator
- `test_err_print.vp` - Error printing
- `test_exception_chain.vp` - Exception chaining

### Math & BigInt
- `test_math_simple.vp` - Math functions (gcd, lcm, factorial, isqrt)
- `fib_test.vp` - Fibonacci with BigInt
- `fib_python_style.vp` - Python-style Fibonacci
- `test_abs.vp` - abs() builtin
- `minimal_bigint.vp` - Basic BigInt operations
- `string_func_test.vp` - String functions
- `jit_test.vp` - JIT execution

### OOP & Advanced
- `comprehensive_oop.vp` - OOP (diamond, MRO, multi-level)
- `overload_test.vp` - Function overloading
- `bigint_test.vp` - BigInt operations (after str() fix)
- `bigint_simple_test.vp` - Simple BigInt tests

---

## ❌ Failing Tests (~30+)

### Standard Library Issues (Parser Errors)
| Test | Error |
|------|-------|
| `test_collections.vp` | collections.vp: "Unexpected token: Tuple" |
| `test_contextlib.vp` | contextlib.vp: "Unexpected token: Star" |
| `test_csv.vp` | csv.vp: "Expected In, found Comma" |
| `test_datetime.vp` | datetime.vp: "Inconsistent indentation" |
| `test_coverage_example.vp` | coverage.vp: "Inconsistent indentation" |
| `test_debugger.vp` | pdb.vp: "Expected Colon, found Indent" |
| `test_with.vp` | io.vp: "Expected RParen, found None at line 147" |

### Parser Syntax Issues
| Test | Error |
|------|-------|
| `test_dataclass.vp` | "Expected Def, found Class" |
| `test_functools.vp` | "Expected Def, found Class" |
| `test_exceptions.vp` | "Expected Colon, found Ident" |
| `test_iterator.vp` | "Expected Colon, found Indent" |
| `test_typing.vp` | "Expected identifier, found Optional" |
| `test_pattern_match.vp` | "Undefined variable: r2" |
| `test_loop_else.vp` | "Unexpected token: Indent" |

### Codegen Issues
| Test | Error |
|------|-------|
| `test_factorial_loop.vp` | "Unsupported pointer operator: Mul" |
| `bigint_simple_test.vp` | "Unsupported pointer operator: Sub" |
| `test_while.vp` | "Unsupported pointer operator: Mul" |
| `generic_types.vp` | LLVM type mismatch errors |

### Missing Dependencies
| Test | Error |
|------|-------|
| `test_import.vp` | "Module 'test_module' not found" |
| `test_stdlib_phase*.vp` | Various stdlib loading failures |

### Runtime Issues
| Test | Error |
|------|-------|
| `test_nonlocal.vp` | "Basic Block does not have terminator!" |
| `super_test.vp` | Segmentation fault (JIT) |

---

## Root Causes

### 1. Standard Library Parser Errors (High Impact)
Multiple stdlib files fail to parse due to:
- Indentation handling bugs
- Tuple/star token handling in expressions
- Multi-line function signature issues

**Affected:** 10+ stdlib modules (io, collections, contextlib, csv, datetime, etc.)

### 2. Unsupported Pointer Operators (Medium Impact)
Codegen rejects `Sub` and `Mul` operations on pointer types.

**Affected:** List operations, string operations

### 3. Advanced Syntax Not Supported (Medium Impact)
Parser doesn't handle:
- `@decorator` syntax
- `raise ... from ...` syntax
- `Optional[T]`, `List[T]` typing
- Loop-else syntax

### 4. Closure/Nonlocal Codegen (Low Impact)
Missing terminators in if-continue blocks for closure cells.

---

## Pass Rate

| Category | Passing | Total | Rate |
|----------|---------|-------|------|
| Core Tests | ~20 | ~50 | ~40% |
| Stdlib Tests | 0 | ~10 | 0% |
| Parser Tests | ~5 | ~15 | ~33% |
| Codegen Tests | ~10 | ~15 | ~67% |
| **Overall** | **~20** | **~50+** | **~40%** |

---

## Priority Fixes Needed

### P0 - Blockers
1. **Fix stdlib parser errors** - io.vp, collections.vp, etc.
2. **Fix pointer operator support** - Sub, Mul for lists/strings

### P1 - High Priority
3. **Fix @decorator parsing** - dataclass, etc.
4. **Fix closure cell codegen** - nonlocal terminator issues
5. **Fix typing syntax** - Optional[T], List[T]

### P2 - Medium Priority
6. **Fix exception syntax** - raise ... from ...
7. **Fix loop-else** - Control flow
8. **Fix super() segfault** - JIT stability

---

## Conclusion

**~40% of tests pass.** The compiler has solid foundations but needs:
- Stdlib parser fixes (blocking all stdlib tests)
- Codegen pointer operator support
- Advanced syntax parser support

The fixes implemented in this session resolved:
- Type checker false positives
- Lexer indentation issues  
- String repetition support
- exit() builtin
- Python-standard function names
