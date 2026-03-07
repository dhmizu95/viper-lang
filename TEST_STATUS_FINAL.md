# Viper Test Status - Final Report

**Date:** March 8, 2026  
**Version:** 0.5.0

---

## Summary

**Not all tests pass.** Current status: **~14/50+ tests passing (~28%)**

---

## ✅ Passing Tests (14)

### Core Language
- `test_walrus.vp` - Walrus operator ✅
- `test_global_simple.vp` - Global keyword ✅
- `test_isinstance.vp` - isinstance() builtin ✅
- `test_neg.vp` - Negation operator ✅
- `test_err_print.vp` - Error printing ✅

### Math & BigInt
- `test_math_simple.vp` - Math functions ✅
- `fib_test.vp` - Fibonacci ✅
- `fib_python_style.vp` - Python-style Fibonacci ✅
- `test_abs.vp` - abs() builtin ✅
- `minimal_bigint.vp` - Basic BigInt ✅
- `string_func_test.vp` - String functions ✅
- `jit_test.vp` - JIT execution ✅

### OOP & Advanced
- `comprehensive_oop.vp` - OOP (diamond, MRO) ✅
- `overload_test.vp` - Function overloading ✅

---

## ❌ Failing Tests (36+)

### Stdlib Loading Issues (10+ tests)
These tests fail because stdlib modules don't parse correctly:

| Module | Error |
|--------|-------|
| io.vp | Type errors: bytes(), __builtin_open undefined |
| collections.vp | "Unexpected token: Tuple" |
| contextlib.vp | "Unexpected token: Star" |
| csv.vp | "Expected In, found Comma" |
| datetime.vp | "Inconsistent indentation" |
| coverage.vp | "Inconsistent indentation" |
| pdb.vp | "Expected Colon, found Indent" |

**Affected tests:** test_with.vp, test_collections.vp, test_contextlib.vp, test_csv.vp, test_datetime.vp, test_coverage_example.vp, test_debugger.vp

### Parser Syntax Issues (6+ tests)
| Test | Issue |
|------|-------|
| test_dataclass.vp | @decorator syntax not supported |
| test_functools.vp | @decorator syntax not supported |
| test_typing.vp | Optional[T], List[T] not supported |
| test_iterator.vp | Indent handling in class |
| test_exceptions.vp | raise...from syntax |
| test_loop_else.vp | Loop-else syntax |

### Codegen Issues (10+ tests)
| Test | Error |
|------|-------|
| bigint_test.vp | "Unsupported pointer operator: Sub" |
| bigint_simple_test.vp | "Unsupported pointer operator: Sub" |
| test_factorial_loop.vp | "Unsupported pointer operator: Mul" |
| test_while.vp | "Unsupported pointer operator: Mul" |
| generic_types.vp | LLVM type mismatch |

### Runtime Issues
| Test | Issue |
|------|-------|
| test_nonlocal.vp | "Basic Block does not have terminator" |
| super_test.vp | Segmentation fault (JIT) |

### Missing Dependencies
| Test | Issue |
|------|-------|
| test_import.vp | Module not found |
| test_stdlib_phase*.vp | Various stdlib failures |

---

## Fixes Applied This Session

### 1. Parser: Added `bytes` Type ✅
**File:** `src/parser/statements/definitions.rs`

Added `bytes` to recognized type annotations, fixing io.vp parsing.

### 2. Type Checker: Result/Dict/Optional ✅
**File:** `src/semantic/type_checker/compatibility.rs`

Added type compatibility rules for complex types.

### 3. Lexer: Comment & Paren Handling ✅
**File:** `src/lexer/scanner.rs`

- Fixed newline consumption after comments
- Added paren depth tracking for multi-line expressions

### 4. Builtins: exit() Implementation ✅
**Files:** Multiple

Full implementation of `exit()` with codegen and JIT support.

### 5. String Repetition ✅
**Files:** Multiple

Full support for `str * int` and `int * str`.

### 6. Python Standard Names ✅
**Files:** tests/*.vp

Replaced non-standard functions:
- `str_int()` → `str()`
- `abs_int()` → `abs()`
- `sqrt_int()` → `math.isqrt()`
- `pow_int()` → `pow()`

---

## Root Causes

### 1. Stdlib Parser Bugs (High Impact)
Multiple stdlib files have syntax that the parser doesn't handle:
- Complex type annotations
- Multi-line function signatures
- Star expressions
- Tuple expressions in certain contexts

### 2. Tagged Int/BigInt Codegen (Medium Impact)
The tagged integer implementation has gaps:
- Sub/Mul operations on pointer-typed values
- Some comparison operations

### 3. Advanced Syntax Not Implemented (Medium Impact)
- @decorator syntax
- raise...from exception chaining
- Generic type syntax (Optional[T])
- Loop-else

### 4. Closure Cell Codegen (Low Impact)
Missing terminators in nonlocal variable handling.

---

## What Would It Take to Make ALL Tests Pass?

### P0 - Critical (Blocks 20+ tests)
1. **Fix stdlib parser errors** - Estimate: 2-3 days
   - io.vp, collections.vp, contextlib.vp need syntax fixes
   - Or parser needs to handle more Python constructs

2. **Fix tagged int pointer operations** - Estimate: 1-2 days
   - Add Sub, Mul support for pointer-typed tagged ints
   - Fix comparison operations

### P1 - High Priority (Blocks 10+ tests)
3. **Implement @decorator parsing** - Estimate: 1 day
4. **Fix raise...from syntax** - Estimate: 0.5 days
5. **Fix Optional[T] parsing** - Estimate: 0.5 days

### P2 - Medium Priority (Blocks 5+ tests)
6. **Fix closure cell codegen** - Estimate: 1 day
7. **Fix super() segfault** - Estimate: 1-2 days
8. **Fix loop-else** - Estimate: 0.5 days

**Total Estimated Effort: 7-11 days of focused development**

---

## Conclusion

**Current pass rate: ~28% (14/50+ tests)**

The compiler has solid foundations:
- Core language features work
- BigInt operations mostly work
- OOP inheritance works
- Type inference works

But significant work remains:
- Stdlib loading is broken (parser bugs)
- Some codegen operations missing
- Advanced Python syntax not implemented

**Making all tests pass would require approximately 1-2 weeks of focused development effort.**
