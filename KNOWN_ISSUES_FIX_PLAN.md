# Known Issues Fix Plan

## Overview

This document outlines the plan to fix known issues in the Viper compiler related to literal types and string operations that currently cause segmentation faults.

---

## Issue #1: f-string Literals Segfault

### Problem
f-string literals with interpolation cause segmentation faults during JIT execution.

**Example:**
```python
def test():
    name = "World"
    a = f"Hello {name}"
    print(a)  # Segfault
```

### Root Cause Analysis
1. **Codegen**: f-string code generation in `src/codegen/expressions/core.rs` creates string elements and concatenates them
2. **Runtime**: `vp_str_concat` function in the C runtime library has memory management issues
3. **Symptom**: String concatenation works for simple cases but fails with interpolated expressions

### Files Involved
- `src/codegen/expressions/core.rs` - FString code generation (lines 329-382)
- `src/codegen/expressions/operators/strings.rs` - `generate_str_concat` function
- `runtime/src/strings.c` - `vp_str_concat` implementation
- `runtime/include/viper_stdlib.h` - Function declarations

### Fix Plan

#### Phase 1: Debug Runtime (2-3 days)
1. Add debug logging to `vp_str_concat` to trace memory operations
2. Use valgrind to detect memory leaks/invalid accesses
3. Check reference counting on string objects
4. Verify null terminator handling

#### Phase 2: Fix Runtime (1-2 days)
1. Fix memory allocation in `vp_str_concat`
2. Ensure proper null termination
3. Add bounds checking
4. Test with various string lengths

#### Phase 3: Test (1 day)
1. Add comprehensive f-string tests
2. Test edge cases (empty strings, multiple interpolations, nested expressions)
3. Verify no memory leaks

### Estimated Time: 4-6 days

---

## Issue #2: Bytes Literals Segfault

### Problem
Bytes literals cause segmentation faults during JIT execution.

**Example:**
```python
def test():
    a = b"hello"
    print(a)  # Segfault
```

### Root Cause Analysis
1. **Codegen**: `bytes_const` in `src/codegen/builder.rs` creates LLVM global
2. **Runtime**: `vp_bytes_create` may have issues with pointer handling
3. **Type System**: Bytes type may not be properly integrated with print system

### Files Involved
- `src/codegen/expressions/core.rs` - Bytes code generation (lines 311-322)
- `src/codegen/builder.rs` - `bytes_const` function (lines 55-77)
- `runtime/src/data_structures/bytes.c` - `vp_bytes_create` implementation
- `runtime/include/viper_stdlib.h` - ViperBytes struct and functions
- `src/codegen/expressions/builtins/print.rs` - Bytes printing support

### Fix Plan

#### Phase 1: Debug Codegen (1-2 days)
1. Verify `bytes_const` creates correct LLVM IR
2. Check pointer type alignment
3. Verify global initializer is correct

#### Phase 2: Debug Runtime (2-3 days)
1. Add debug logging to `vp_bytes_create`
2. Check ViperBytes struct layout matches LLVM type
3. Verify memory allocation and initialization
4. Test bytes printing separately

#### Phase 3: Integration (1 day)
1. Test bytes literals in various contexts
2. Test bytes operations (concatenation, slicing)
3. Verify no memory leaks

### Estimated Time: 4-6 days

---

## Issue #3: String Concatenation Segfault

### Problem
String concatenation with `+` operator causes segmentation faults.

**Example:**
```python
def test():
    a = "Hello " + "World"
    print(a)  # Segfault
```

### Root Cause
Same as Issue #1 - `vp_str_concat` runtime function

### Fix Plan
Will be fixed as part of Issue #1 resolution.

### Estimated Time: Included in Issue #1

---

## Issue #4: Multiple Assignment (Future)

### Problem
Tuple unpacking in assignment causes segmentation faults.

**Example:**
```python
def test():
    a, b = 1, 2  # Segfault
    print(a)
    print(b)
```

### Status
Lower priority - requires tuple literal support first.

### Estimated Time: 5-7 days (after tuple support)

---

## Testing Strategy

### Unit Tests
- Add tests for each fixed feature in appropriate test files
- Test edge cases and error conditions

### Integration Tests
- Add end-to-end tests in `test_literals.rs`
- Test combinations of features

### Memory Testing
- Run with valgrind to detect leaks
- Test with large strings/bytes objects

### Regression Testing
- Ensure existing tests still pass
- Add tests to prevent future regressions

---

## Priority Order

1. **High Priority**: f-string literals (commonly used feature)
2. **High Priority**: String concatenation (blocks f-strings)
3. **Medium Priority**: Bytes literals (less commonly used)
4. **Low Priority**: Multiple assignment (requires tuple support)

---

## Success Criteria

- [ ] All f-string tests pass without segfaults
- [ ] All bytes literal tests pass without segfaults
- [ ] String concatenation works reliably
- [ ] No memory leaks detected by valgrind
- [ ] All existing tests still pass
- [ ] Documentation updated

---

## Timeline

| Week | Task |
|------|------|
| 1 | Debug and fix string concatenation runtime |
| 2 | Fix f-string codegen and add tests |
| 3 | Debug and fix bytes literals |
| 4 | Testing, documentation, and cleanup |

**Total Estimated Time: 3-4 weeks**

---

## Notes

- Some issues may be deeper than initially apparent
- Timeline may adjust based on root cause complexity
- Consider adding sanitizers (ASan, UBSan) for debugging
- Document all changes for future maintainers

---

*Last Updated: March 10, 2026*
*Version: 0.5.0*
