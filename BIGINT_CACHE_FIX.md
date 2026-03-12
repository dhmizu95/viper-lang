# BigInt Caching Limitations and Fix Plan

**Date:** March 12, 2026  
**Status:** Infrastructure Complete, Auto-Detection Pending

---

## Current Status

### ✅ Working
- i64 return values: Full caching support
- Multi-parameter (1-2 params): Working
- BigInt infrastructure: Union type, is_bigint flag

### ⏳ Pending
- BigInt auto-detection: Manual flag only
- BigInt pointer management: GC integration needed

---

## Problem Analysis

### Test Case: fib(75)
```python
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

# fib(75) = 2111485077978050 (exceeds i64 range)
# Viper auto-promotes to BigInt
```

### Current Behavior
1. fib(50), fib(60), fib(70) work (fit in i64)
2. fib(75) times out (BigInt, cache not working)

### Root Cause
```rust
// Current codegen (functions.rs)
// For now, assume i64 return (is_bigint = 0)
// TODO: Detect BigInt return type and set is_bigint = 1
let is_bigint_val = self.context.i32_type().const_int(0, false);
```

The cache stores BigInt pointers as i64 values, causing:
1. Pointer truncation (64-bit pointer stored as i64)
2. Cache misses (truncated value doesn't match)
3. Repeated computation (no cache benefit)

---

## Solution Options

### Option A: Type Annotation Detection (Recommended)

**Implementation:**
```rust
// In define_memoized_function
let is_bigint_return = return_type.as_ref()
    .map_or(false, |t| matches!(t, Type::BigInt));

let is_bigint_val = self.context.i32_type().const_int(
    if is_bigint_return { 1 } else { 0 }, 
    false
);
```

**Pros:**
- Simple, compile-time detection
- No runtime overhead
- Works with explicit type annotations

**Cons:**
- Requires type annotation: `def fib(n) -> BigInt:`
- Doesn't work with inferred BigInt

---

### Option B: Runtime Type Check

**Implementation:**
```rust
// After calling body function
let result_type = self.get_result_type(body_call);
let is_bigint = self.builder.build_int_compare(
    IntPredicate::EQ,
    result_type,
    BIGINT_TYPE_ID,
    "is_bigint"
);
```

**Pros:**
- Works with inferred BigInt
- No type annotation needed

**Cons:**
- Runtime overhead
- Complex implementation

---

### Option C: Conservative Caching (Temporary)

**Implementation:**
```rust
// Always set is_bigint = 0 for now
// Document limitation: BigInt results not cached
```

**Pros:**
- Simple
- i64 caching works perfectly

**Cons:**
- BigInt functions get no cache benefit
- fib(75) still slow

---

## Recommended Fix (Option A)

### Step 1: Check Return Type Annotation

```rust
// In define_memoized_function
let is_bigint_return = match return_type {
    Some(Type::BigInt) => true,
    Some(Type::Infer) => {
        // For inferred types, check if function body uses BigInt operations
        // This requires type inference results from semantic analysis
        false  // Conservative default
    }
    _ => false,
};
```

### Step 2: Pass is_bigint to Cache

```rust
let is_bigint_val = self.context.i32_type().const_int(
    if is_bigint_return { 1 } else { 0 }, 
    false
);

self.builder.build_call(
    set_func,
    &[
        loaded_cache.into(),
        key_value.into(),
        result_value.into(),  // Works for both i64 and pointers
        key_size_val.into(),
        is_bigint_val.into(),
    ],
    "",
);
```

### Step 3: Handle BigInt in Cache Hit

```rust
// In cache hit block
if is_bigint_return {
    // For BigInt, cached_value is a pointer
    // Return directly (pointer to tagged BigInt)
    self.builder.build_return(Some(&cached_value)).expect("return");
} else {
    // For i64, cached_value is the actual value
    self.builder.build_return(Some(&cached_value)).expect("return");
}
```

---

## Quick Test (After Fix)

```python
@lru_cache(maxsize=None)
def fib(n) -> BigInt:  # Explicit type annotation
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    # Should be instant with caching
    print("fib(75) =", fib(75))
    print("fib(75) again =", fib(75))  # Cache hit!
```

---

## Alternative: Use @cache for BigInt

For now, users can work around this by:

1. Using iterative implementation for large Fibonacci
2. Using `@lru_cache` only for i64-range inputs
3. Waiting for auto-detection implementation

---

## Implementation Priority

| Task | Priority | Effort |
|------|----------|--------|
| Type annotation detection | P1 | 2 hours |
| Inferred BigInt detection | P2 | 1 day |
| GC integration for BigInt | P1 | 4 hours |
| Test suite for BigInt | P1 | 2 hours |

---

*Last Updated: March 12, 2026*  
*Author: Viper Language Team*
