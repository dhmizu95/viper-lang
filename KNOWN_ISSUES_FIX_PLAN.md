# Known Issues Fix Plan

## Overview

This document outlines the plan to fix known issues in the Viper compiler related to literal types and string operations that currently cause segmentation faults.

---

## Issue #1: f-string Literals Segfault ✅ FIXED

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
- `runtime/include/viper_types.h` - `vp_str_concat` implementation (inline)

### Fix Applied
Fixed `vp_str_concat` in `runtime/include/viper_types.h` to:
- Use SSO (Small String Optimization) for strings ≤15 characters
- Use heap allocation for larger strings with proper embedded data layout
- Correctly set the SSO flag bit for small strings

### Status: ✅ FIXED - March 10, 2026

---

## Issue #2: Bytes Literals Segfault ✅ FIXED

### Problem
Bytes literals cause segmentation faults during JIT execution.

**Example:**
```python
def test():
    a = b"hello"
    print(a)  # Segfault
```

### Root Cause
The bytes literal issue was caused by the same string concatenation bug that affected f-strings.

### Status: ✅ FIXED - March 10, 2026

---

## Issue #3: String Concatenation Segfault ✅ FIXED

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

### Status: ✅ FIXED - March 10, 2026

---

## Issue #4: For Loops Only Execute First Iteration ✅ FIXED

### Problem
For loops only execute the first iteration, then segfault.

**Example:**
```python
def test():
    for i in range(3):
        print(i)  # Only prints 0, then segfault
    print(999)    # Never prints
```

### Root Cause Analysis
1. **Alloca Placement**: The counter variable alloca was created in the wrong basic block
2. **Tagged Integers**: Viper uses tagged integers (LSB=0 for small ints, value<<1), but the for loop counter was storing untagged values
3. **Value Mismatch**: When `print(i)` was called, it expected a tagged value but received an untagged value

### Files Involved
- `src/codegen/control_flow/loops.rs` - `generate_for()` function (range() path)

### Fix Applied
1. Moved counter alloca to function entry block for proper dominance
2. Store tagged integer values in the counter (`value << 1`)
3. Properly handle tagged vs untagged start/step values based on range() argument count:
   - 1-arg `range(3)`: start=0 (untagged), step=1 (untagged) - both need tagging
   - 2-arg `range(2, 5)`: start=tagged, step=1 (untagged) - step needs tagging
   - 3-arg `range(0, 10, 2)`: start=tagged, step=tagged - no tagging needed

### Status: ✅ FIXED - March 10, 2026

---

## Issue #5: Multiple Assignment (Future)

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
