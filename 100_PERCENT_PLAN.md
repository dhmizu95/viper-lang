# Viper Compiler - 100% Test Pass Plan

**Goal:** Make all 113 test files pass

---

## Current Status

**Passing:** ~20-22 tests (~40%)  
**Failing:** ~75 tests (~60%)

---

## Critical Issues to Fix

### 1. Result Type Codegen (Blocks ~5 tests)
**Error:** `Function return type does not match operand type of return inst!`

**Files:** `src/codegen/statements/core/return.rs`, `src/codegen/types.rs`

**Fix:** Ensure Result[T, E] uses consistent LLVM struct type {i8, value} where value is properly typed.

### 2. Closure Cell Terminators (Blocks ~3 tests)
**Error:** `Basic Block does not have terminator!`

**Files:** `src/codegen/statements/core/exceptions.rs`, `src/codegen/closure_analysis.rs`

**Fix:** Add proper branch terminators after closure cell loads in if-else blocks.

### 3. from X import Y.Z Syntax (Blocks ~5 tests)
**Error:** `Expected Import, found Dot`

**Files:** `src/parser/statements/core.rs`

**Fix:** Parse dotted names in from-import statements.

### 4. Stdlib Loading (Blocks ~15 tests)
**Errors:** Various parser errors in io.vp, collections.vp, etc.

**Files:** Multiple stdlib files + parser

**Fix:** Fix stdlib syntax to match parser capabilities.

### 5. Full ViperString Support (Blocks ~5 tests)
**Issue:** str() returns tagged int workaround

**Files:** `runtime/src/tagged_int.c`, `src/codegen/expressions/builtins/str.rs`

**Fix:** Proper ViperString creation and conversion.

### 6. Math Module Integration (Blocks ~3 tests)
**Issue:** Type mismatch in print(str())

**Files:** `std/math.vp`, tests

**Fix:** Ensure math.isqrt returns proper type.

---

## Implementation Plan

### Phase 1: Quick Wins (1-2 days)
- [ ] Fix from X import Y.Z parsing
- [ ] Fix math module type issues
- [ ] Fix test file indentation issues

### Phase 2: Codegen Fixes (2-3 days)
- [ ] Fix Result type codegen
- [ ] Fix closure cell terminators
- [ ] Fix str() ViperString support

### Phase 3: Stdlib Fixes (2-3 days)
- [ ] Fix io.vp parsing
- [ ] Fix collections.vp parsing
- [ ] Fix contextlib.vp parsing
- [ ] Fix csv.vp parsing
- [ ] Fix datetime.vp parsing

### Phase 4: Remaining Issues (2-3 days)
- [ ] Fix Generic[T] in class bases
- [ ] Fix nested class definitions
- [ ] Fix any remaining parser issues

---

## Expected Timeline

**Total:** 7-11 days of focused development

**Risk:** Some issues may require architectural changes (e.g., full ViperString support)

---

## Success Criteria

- All 97 .vp test files pass
- All 12 .rs unit tests pass
- No segfaults or runtime errors
- Clean build with no errors
