# Tail Call Optimization (TCO) Implementation Plan

## Overview

Tail Call Optimization (TCO) allows recursive functions to run in constant stack space by reusing the current stack frame for the next call when the call is in tail position.

## Current State

- **No explicit TCO implementation** - Recursive functions consume stack space for each call
- **Workaround**: Automatic memoization for exponential recursion (Fibonacci-style)
- **Inline hints** exist for small functions but don't handle tail calls

## Implementation Strategy

### 1. Tail Call Detection (Semantic Analysis)

Add detection in `src/semantic/recursion_analysis.rs`:

```rust
/// Detect if a function call is in tail position
fn is_tail_call(stmts: &[Stmt], call_expr: &Expr) -> bool {
    // A call is in tail position if:
    // 1. It's the last statement in a function body
    // 2. The return value of the call is directly returned
    // 3. No operations happen after the call
}
```

### 2. Mark Tail-Call Candidates

Extend `RecursiveFunctionInfo` in `recursion_analysis.rs`:

```rust
pub struct RecursiveFunctionInfo {
    // ... existing fields
    /// Has at least one tail-recursive call site
    pub has_tail_recursive_call: bool,
    /// Call sites that are in tail position
    pub tail_call_sites: Vec<CallSiteInfo>,
}
```

### 3. Codegen Changes

Modify `src/codegen/expressions/calls/dispatch.rs`:

```rust
// Add tail call attribute to function calls in tail position
if is_tail_call {
    let tail_attr = context.create_string_attribute("tail", "");
    callsite.add_attribute(AttributeLoc::CallSite, tail_attr);
}
```

Or use LLVM's `build_tail_call` if available in Inkwell.

### 4. Ensure Proper Function Signature

Tail calls require matching calling conventions:

```rust
// Ensure tail-recursive functions have proper attributes
func_val.add_attribute(
    AttributeLoc::Function, 
    context.create_string_attribute("tailcall", "must")
);
```

## Challenges

1. **Python Compatibility**: Viper aims for Python compatibility - need to ensure TCO doesn't break Python semantics
2. **Memory Management**: TCO reuses stack frames - need to ensure ARC cleanup happens correctly
3. **Non-tail Recursion**: Functions with multiple recursive calls (like Fibonacci) can't be optimized via TCO
4. **Mutual Recursion**: More complex - requires trampolines or continuation-passing style

## Implementation Steps

### Phase 1: Detection (Easy)
- [ ] Add tail call detection to `RecursionAnalyzer`
- [ ] Identify functions with tail-recursive patterns

### Phase 2: Codegen (Medium)
- [ ] Add LLVM tail call attribute support
- [ ] Modify call generation to mark tail calls
- [ ] Add function-level tail call attributes

### Phase 3: Testing (Easy)
- [ ] Create test cases for tail-recursive functions
- [ ] Verify stack usage doesn't grow
- [ ] Compare performance before/after

## Example: Tail-Recursive Factorial

**Before (not tail-recursive)**:
```python
def factorial(n, acc=1):
    if n <= 1:
        return acc
    return factorial(n-1, n*acc)  # Not tail position - can't optimize
```

**After (tail-recursive)**:
```python
def factorial(n, acc=1):
    if n <= 1:
        return acc
    return factorial(n-1, n*acc)  # Tail position - can optimize!
```

## Benchmark Impact

Expected improvements for tail-recursive functions:
- **Factorial**: ~10-100x improvement (stack depth reduced from O(n) to O(1))
- **List sum**: Similar improvements
- **Tree traversal**: Significant improvement for deep trees

## Files to Modify

1. `src/semantic/recursion_analysis.rs` - Add tail call detection
2. `src/codegen/expressions/calls/dispatch.rs` - Add tail call attributes
3. `src/codegen/core/functions.rs` - Add function-level attributes
4. New test file: `tests/unit/tail_call.rs`
