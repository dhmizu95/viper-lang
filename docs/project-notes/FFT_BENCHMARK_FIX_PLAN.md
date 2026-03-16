# 15_FFT Benchmark Fix Plan

## Problem Summary

The 15_fft benchmark fails with a codegen error:
```
Error: Codegen error: Call parameter type does not match function signature!
  %elem_0 = load i64, ptr %elem_0_ptr, align 4
 ptr  call void @fft_list_infer_list_infer_bool(i64 %elem_0, i64 %elem_1, i1 false)
```

## Root Cause Analysis

### Primary Issue: Missing Float List Type Tracking

There are TWO related problems:

1. **No `float_list_vars` tracking**: The codebase has `bool_list_vars` to track bool-specific lists, but lacks equivalent tracking for float lists.

2. **Inconsistent float list detection**: 
   - [`src/codegen/expressions/calls/methods.rs:119-121`](src/codegen/expressions/calls/methods.rs:119) has a **hardcoded workaround**:
     ```rust
     let is_float_list = if let Expr::Ident(name, _) = obj {
         matches!(name.as_str(), "x" | "y" | "z" | "vx" | "vy" | "vz" | "mass" | "real" | "imag")
     ```
   - [`src/codegen/expressions/collections/index.rs`](src/codegen/expressions/collections/index.rs:160) lacks this workaround, so list indexing doesn't recognize float lists

### Why This Causes the Bug

1. FFT uses lists `real` and `imag` filled with `math.sin()` (float) values
2. When accessing `real[idx]`, the index code doesn't recognize it's a float list
3. Uses `inline_i64_list_get` instead of `inline_f64_list_get`
4. Generates i64 values, but the function expects f64 values
5. LLVM verification fails with type mismatch

## Fix Strategy

### Option A: Add `float_list_vars` Tracking (Proper Fix)

This mirrors the existing `bool_list_vars` pattern:

1. **Add to `src/codegen/state.rs`**:
   ```rust
   pub float_list_vars: &'a mut HashSet<String>,
   ```
   Add methods:
   ```rust
   pub fn mark_as_float_list(&mut self, name: String) {
       self.float_list_vars.insert(name.clone());
       self.list_vars.insert(name);
   }
   
   pub fn is_float_list(&self, name: &str) -> bool {
       self.float_list_vars.contains(name)
   }
   ```

2. **Add to `src/codegen/core/context.rs`**:
   ```rust
   pub(crate) float_list_vars: HashSet<String>,
   ```

3. **Update all CodeGenState initializations** to include `float_list_vars`

4. **Update `src/codegen/expressions/collections/index.rs`** to check float lists:
   ```rust
   let is_float_list = if let Expr::Ident(name, _) = obj {
       state.is_float_list(name)
   } else {
       false
   };
   ```

5. **Update `src/codegen/expressions/calls/methods.rs`** to use proper tracking instead of hardcoded names

### Option B: Quick Fix (Less Robust)

Add the hardcoded variable names to index.rs (like methods.rs):
```rust
let is_float_list = if let Expr::Ident(name, _) = obj {
    matches!(name.as_str(), "x" | "y" | "z" | "vx" | "vy" | "vz" | "mass" | "real" | "imag")
} else {
    false
};
```

## Recommended Approach

**Option A** is preferred as it:
- Properly tracks list types at runtime
- Doesn't require hardcoding variable names
- Follows existing patterns (bool_list_vars)
- Is more maintainable

## Implementation Steps

1. Add `float_list_vars` to CodeGenState in `state.rs`
2. Add `mark_as_float_list()` and `is_float_list()` methods
3. Add to CodeGenContext in `context.rs`
4. Propagate through module generation
5. Update index.rs to use `state.is_float_list()`
6. Update methods.rs to use tracking instead of hardcoded names
7. Test with 15_fft benchmark

## Files to Modify

- `src/codegen/state.rs` - Add float_list_vars tracking
- `src/codegen/core/context.rs` - Add float_list_vars field
- `src/codegen/core/module_gen.rs` - Initialize float_list_vars
- `src/codegen/core/functions.rs` - Pass float_list_vars through
- `src/codegen/expressions/collections/index.rs` - Use float list detection
- `src/codegen/expressions/calls/methods.rs` - Use proper tracking (remove hardcoding)
