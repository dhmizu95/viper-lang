# Viper Test Status - Final Summary

**Date:** March 8, 2026  
**Version:** 0.5.0

---

## Summary

**Not all tests pass.** Current status: **~13/50+ tests passing (~26%)**

---

## ✅ Passing Tests (13)

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

## ❌ Failing Tests (37+)

### Tagged Int Arithmetic (2 tests)
| Test | Error |
|------|-------|
| bigint_test.vp | "Unsupported pointer operator: Sub" |
| test_factorial_loop.vp | "Unsupported pointer operator: Mul" |

**Root Cause:** The tagged int codegen expects pointer values, but `generate_expr` for identifiers loads from stack slots, creating a pointer-to-pointer mismatch.

### Stdlib Loading Issues (10+ tests)
| Module | Error |
|--------|-------|
| io.vp | Type errors: list concatenation, __builtin_open undefined |
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

### Runtime Issues
| Test | Issue |
|------|-------|
| test_nonlocal.vp | "Basic Block does not have terminator" |
| super_test.vp | Segmentation fault (JIT) |

---

## Fixes Applied This Session

### 1. Parser: Type Annotation Fixes ✅
**File:** `src/parser/statements/definitions.rs`

- Added `bytes` type to recognized types
- Fixed type aliases to call `parser.advance()` before returning

### 2. Type Checker: Result/Dict/Optional ✅
**File:** `src/semantic/type_checker/compatibility.rs`

Added type compatibility rules for complex types.

### 3. Lexer: Comment & Paren Handling ✅
**File:** `src/lexer/scanner.rs`

- Fixed newline consumption after comments
- Added paren depth tracking for multi-line expressions

### 4. Builtins: exit() and bytes() ✅
**Files:** Multiple

Full implementation of `exit()` and `bytes()` with:
- Type checker registration
- Codegen implementation
- JIT stub (for exit)

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

### 7. Codegen: Type Inference ✅
**File:** `src/codegen/expressions/core.rs`

Fixed `int()` to return `Type::Int` instead of `Type::I64`.

---

## Root Causes

### 1. Tagged Int Arithmetic (CRITICAL)
**Problem:** Tagged int subtraction/multiplication fails with LLVM type mismatch.

**Error:**
```
Call parameter type does not match function signature!
  %a3 = load ptr, ptr %a, align 8
  i64  %tagged_sub = call i64 @tagged_int_sub(ptr %a3, ptr %b4)
```

**Root Cause:** The tagged int codegen expects pointer values, but `generate_expr` for identifiers loads from stack slots, creating a pointer-to-pointer mismatch.

**Fix Required:** Modify `generate_expr` for `Expr::Ident` to return the alloca pointer directly for `Type::Int` variables, not the loaded value.

### 2. Stdlib Parser Bugs (HIGH)
Multiple stdlib files have syntax that the parser doesn't handle:
- Complex type annotations
- Multi-line function signatures
- Star expressions
- Tuple expressions in certain contexts

### 3. Advanced Syntax Not Implemented (MEDIUM)
- @decorator syntax
- raise...from exception chaining
- Generic type syntax (Optional[T])
- Loop-else

### 4. Closure Cell Codegen (LOW)
Missing terminators in nonlocal variable handling.

---

## What Would It Take to Make ALL Tests Pass?

### P0 - Critical (Blocks 2+ tests)
1. **Fix tagged int arithmetic** - Estimate: 2-3 days
   - Modify `generate_expr` for `Expr::Ident` to return alloca pointer for Type::Int
   - Ensure all tagged int operations work correctly

### P1 - High Priority (Blocks 10+ tests)
2. **Fix stdlib parser errors** - Estimate: 2-3 days
   - io.vp, collections.vp, contextlib.vp need syntax fixes
   - Or parser needs to handle more Python constructs

### P2 - Medium Priority (Blocks 6+ tests)
3. **Implement @decorator parsing** - Estimate: 1 day
4. **Fix raise...from syntax** - Estimate: 0.5 days
5. **Fix Optional[T] parsing** - Estimate: 0.5 days

### P3 - Low Priority (Blocks 2+ tests)
6. **Fix closure cell codegen** - Estimate: 1 day
7. **Fix super() segfault** - Estimate: 1-2 days
8. **Fix loop-else** - Estimate: 0.5 days

**Total Estimated Effort: 8-12 days of focused development**

---

## Conclusion

**Current pass rate: ~26% (13/50+ tests)**

The compiler has solid foundations:
- Core language features work
- BigInt operations mostly work (except Sub/Mul)
- OOP inheritance works
- Type inference works

But significant work remains:
- Tagged int arithmetic is broken (Sub, Mul)
- Stdlib loading is broken (parser bugs)
- Advanced Python syntax not implemented

**Making all tests pass would require approximately 1.5-2 weeks of focused development effort.**

---

## Files Modified This Session

1. `src/parser/statements/definitions.rs` - Type annotation fixes
2. `src/semantic/type_checker/compatibility.rs` - Type compatibility
3. `src/lexer/scanner.rs` - Comment handling, paren depth
4. `src/semantic/type_checker/exprs.rs` - int() return type
5. `src/codegen/expressions/core.rs` - Type inference for int()
6. `src/codegen/expressions/operators/mod.rs` - Tagged int check
7. `src/semantic/symbol_table.rs` - bytes() builtin
8. `src/codegen/expressions/builtins/str.rs` - generate_bytes_call
9. `src/codegen/expressions/calls/dispatch.rs` - bytes() dispatch
10. `std/builtins_ext.vp` - Removed non-standard functions
11. `std/prelude.vp` - Added exit()
12. `src/codegen/expressions/builtins/print.rs` - exit() codegen
13. `src/codegen/runtime/print.rs` - vp_exit declaration
14. `src/jit_stubs/strings.rs` - vp_exit_stub, vp_str_repeat_stub
15. `src/jit_stubs/registry/strings.rs` - Stub registration
16. `tests/*.vp` - Python-standard function names (100+ files)
