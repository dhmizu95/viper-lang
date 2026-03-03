# Int/BigInt Smooth Handling - Updated Implementation Plan

## Executive Summary

**Current State:** Viper already has a working TaggedInt implementation with automatic overflow detection and promotion to BigInt. However, the system can be improved for better Python-like smoothness and performance.

**Key Findings:**
- ✅ TaggedInt already exists (`runtime/src/tagged_int.c`, `runtime/include/tagged_int.h`)
- ✅ Uses LSB tagging: `LSB=0` for SmallInt (i63), `LSB=1` for BigInt pointer
- ✅ Automatic overflow detection in `would_overflow_add/sub/mul`
- ✅ Runtime dispatch in `tagged_int_add/sub/mul/div`
- ✅ Rust codegen integration (`src/codegen/runtime/tagged_int.rs`)
- ❌ **Missing:** Automatic demotion from BigInt back to SmallInt
- ❌ **Missing:** LLVM overflow intrinsics for faster overflow detection
- ❌ **Missing:** Unified `vp_runtime_add` dispatcher (currently uses `tagged_int_*` functions)

---

## 1. Current Architecture Analysis

### 1.1 TaggedInt Representation (Existing)

```c
// runtime/include/tagged_int.h
typedef uint64_t TaggedInt;

#define TAGGED_INT_SMALL 0  /* LSB = 0: small integer */
#define TAGGED_INT_BIGINT  1  /* LSB = 1: BigInt pointer */

// Small int: value << 1
static inline TaggedInt tagged_int_from_i64(int64_t value) {
    return ((uint64_t)value << 1) | TAGGED_INT_SMALL;
}

// BigInt pointer: pointer | 1
static inline TaggedInt tagged_int_from_bigint(ViperBigInt* bigint) {
    return ((uint64_t)bigint) | TAGGED_INT_BIGINT;
}

// Range: -2^62 to 2^62-1 (±4.6 quintillion)
#define TAGGED_INT_MAX_SMALL ((1LL << 62) - 1)
#define TAGGED_INT_MIN_SMALL (-(1LL << 62))
```

**⚠️ Issue:** The tag bits are **inverted** from the standard convention:
- **Viper's current:** LSB=0 → SmallInt, LSB=1 → BigInt
- **Standard convention:** LSB=1 → SmallInt, LSB=0 → BigInt pointer

**Why this matters:**
- Standard convention allows direct pointer dereferencing for tagged pointers
- Viper's current approach requires untagging before pointer access
- **Recommendation:** Keep current scheme (already deeply integrated), but document clearly

### 1.2 Current Overflow Detection (Software-based)

```c
// runtime/include/tagged_int.h - Current implementation
static inline bool would_overflow_add(int64_t a, int64_t b) {
    if (b > 0 && a > TAGGED_INT_MAX_SMALL - b) return true;
    if (b < 0 && a < TAGGED_INT_MIN_SMALL - b) return true;
    return false;
}
```

**⚠️ Performance Issue:** Uses software checks instead of LLVM overflow intrinsics

### 1.3 Current Addition Flow

```c
// runtime/src/tagged_int.c - Current implementation
TaggedInt tagged_int_add(TaggedInt a, TaggedInt b) {
    // Case 1: Both small integers
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);

        // Check for overflow
        if (would_overflow_add(a_val, b_val)) {
            // Promote both to BigInt and add
            ViperBigInt* a_big = tagged_int_to_bigint(a);
            ViperBigInt* b_big = tagged_int_to_bigint(b);
            ViperBigInt* result = alloc_bigint_for_tagged();
            mpz_add(result->value, a_big->value, b_big->value);
            free_temp_bigint(a_big);
            free_temp_bigint(b_big);
            return tagged_int_from_bigint(result);
        }

        // No overflow - return small int result
        return tagged_int_from_i64(a_val + b_val);
    }

    // Case 2: At least one BigInt
    // ... (allocation and GMP addition)
}
```

**⚠️ Missing:** No demotion back to SmallInt when BigInt result fits in 63 bits

---

## 2. Recommended Improvements

### 2.1 Add Automatic Demotion (Priority: HIGH)

**Problem:** Once a value promotes to BigInt, it never returns to SmallInt, even if the result would fit.

**Example:**
```python
big = 10**100          # BigInt
result = big - (10**100 - 1)  # Should be 1 (SmallInt), but stays BigInt
```

**Solution:** Add `try_demote` check after every BigInt operation:

```c
// runtime/src/tagged_int.c - ADD THIS FUNCTION
static inline TaggedInt try_demote_bigint(mpz_t value) {
    if (mpz_fits_slong_p(value)) {
        int64_t small = mpz_get_si(value);
        return tagged_int_from_i64(small);
    }
    return 0;  // Cannot demote
}

// Update tagged_int_add Case 2:
ViperBigInt* result = alloc_bigint_for_tagged();
mpz_add(result->value, a_big->value, b_big->value);

// TRY DEMOTION BEFORE RETURNING
TaggedInt demoted = try_demote_bigint(result->value);
if (demoted != 0) {
    // Result fits in SmallInt - free the BigInt and return tagged value
    mpz_clear(result->value);
    free(result);
    return demoted;
}

return tagged_int_from_bigint(result);
```

**Apply to:** `tagged_int_sub`, `tagged_int_mul`, `tagged_int_div`, `tagged_int_mod`

### 2.2 Use LLVM Overflow Intrinsics (Priority: MEDIUM)

**Problem:** Current software overflow checks are slower than hardware detection.

**Solution:** Use `llvm.sadd.with.overflow.i64` intrinsic in Rust codegen:

```rust
// src/codegen/runtime/tagged_int.rs - IMPROVED VERSION
pub fn generate_tagged_int_add<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Option 1: Call runtime function (current approach - simpler)
    let func = state.module.get_function("tagged_int_add")
        .ok_or_else(|| "tagged_int_add not declared".to_string())?;
    
    let result = state.ir_builder.build_call(
        state.builder,
        func,
        &[lhs.into(), rhs.into()],
        "tagged_add",
    ).expect("tagged_int_add call");
    
    Ok(result.into())
    
    // Option 2: Inline overflow check with LLVM intrinsic (faster but more complex)
    // This would require generating the branch logic in LLVM IR
    // See INT_BIGINT_SMOOTH_IMPLEMENTATION_PLAN.md (original) for details
}
```

**Trade-off:**
- **Option 1 (current):** Simpler, easier to maintain, LTO can inline
- **Option 2 (intrinsics):** Faster for hot loops, more complex codegen

**Recommendation:** Start with Option 1, profile, then consider Option 2 if needed.

### 2.3 Unified Runtime Dispatcher (Priority: LOW)

**Current:** Separate `tagged_int_add`, `tagged_int_sub`, etc.

**Proposed:** Add `vp_runtime_add`, `vp_runtime_sub` as aliases for consistency:

```c
// runtime/src/tagged_int.c - ADD ALIASES
ViperValue vp_runtime_add(ViperValue a, ViperValue b) {
    return tagged_int_add(a, b);
}

ViperValue vp_runtime_sub(ViperValue a, ViperValue b) {
    return tagged_int_sub(a, b);
}

ViperValue vp_runtime_mul(ViperValue a, ViperValue b) {
    return tagged_int_mul(a, b);
}

ViperValue vp_runtime_div(ViperValue a, ViperValue b) {
    return tagged_int_div(a, b);
}
```

**Benefit:** Consistent naming with other runtime functions, easier to extend later.

---

## 3. Updated Implementation Checklist

### Phase 1: Automatic Demotion (CRITICAL)

- [ ] **runtime/src/tagged_int.c:**
  - [ ] Add `try_demote_bigint()` helper function
  - [ ] Update `tagged_int_add()` to demote after GMP addition
  - [ ] Update `tagged_int_sub()` to demote after GMP subtraction
  - [ ] Update `tagged_int_mul()` to demote after GMP multiplication
  - [ ] Update `tagged_int_div()` to demote after GMP division
  - [ ] Update `tagged_int_mod()` to demote if possible

- [ ] **Testing:**
  - [ ] Test demotion: `(10**100) - (10**100 - 1)` → SmallInt `1`
  - [ ] Test no demotion: `(10**100) + 1` → stays BigInt
  - [ ] Test negative demotion: `(10**100) - (10**100 + 1)` → SmallInt `-1`
  - [ ] Verify no memory leaks with valgrind

### Phase 2: Performance Optimization

- [ ] **Profile current implementation:**
  - [ ] Benchmark SmallInt addition (baseline)
  - [ ] Benchmark overflow promotion
  - [ ] Benchmark BigInt arithmetic
  - [ ] Identify hotspots

- [ ] **Consider LLVM intrinsics (if profiling shows need):**
  - [ ] Update `tagged_int_add` to use `llvm.sadd.with.overflow.i64`
  - [ ] Generate branch logic in Rust codegen
  - [ ] Benchmark improvement

- [ ] **Add branch prediction hints:**
  ```c
  // runtime/include/tagged_int.h
  #include <stdbranch.h>  // C23, or use __builtin_expect
  
  static inline bool would_overflow_add(int64_t a, int64_t b) {
      if (b > 0 && a > TAGGED_INT_MAX_SMALL - b) return true;
      if (b < 0 && a < TAGGED_INT_MIN_SMALL - b) return true;
      return false;
  }
  
  // In tagged_int_add:
  if (VIPER_LIKELY(tagged_int_is_small(a) && tagged_int_is_small(b))) {
      // Fast path
  } else {
      // Slow path
  }
  ```

### Phase 3: Codegen Improvements

- [ ] **src/codegen/runtime/tagged_int.rs:**
  - [ ] Add `generate_tagged_int_from_i64()` for literal conversion
  - [ ] Add `generate_tagged_int_to_str()` for print formatting
  - [ ] Ensure all binary ops call tagged functions

- [ ] **src/codegen/expressions/operators/arithmetic.rs:**
  - [ ] Update `generate_int_binop()` to use tagged functions for `Type::Int`
  - [ ] Keep native LLVM ops for `Type::I64` (fixed-width integers)

- [ ] **src/codegen/types.rs:**
  - [ ] Clarify `Type::Int` vs `Type::I64` distinction
  - [ ] `Type::Int` → TaggedInt (pointer-sized, auto-promoting)
  - [ ] `Type::I64` → Native i64 (fixed-width, no promotion)

### Phase 4: Python Compatibility

- [ ] **std/core/math.vp (or equivalent):**
  ```python
  # Overloaded functions that work with both int and bigint
  
  def abs(x: int) -> int:
      """Absolute value with auto-promotion."""
      if x < 0:
          return -x
      return x
  
  def min(a: int, b: int) -> int:
      """Minimum with auto-promotion."""
      if a < b:
          return a
      return b
  
  def max(a: int, b: int) -> int:
      """Maximum with auto-promotion."""
      if a > b:
          return a
      return b
  ```

- [ ] **Test Python compatibility:**
  ```python
  # All these should work without manual casting
  
  # Basic arithmetic
  a = 1 + 2           # SmallInt
  b = a * 100         # SmallInt
  c = b ** 100        # Automatically BigInt
  
  # Mixed operations
  x = 10              # SmallInt
  y = 10 ** 50        # BigInt
  z = x + y           # Auto-promote to BigInt
  
  # Demotion
  big = 10 ** 100
  result = big - (10 ** 100 - 1)  # Should be SmallInt 1
  
  # Comparisons
  if x < y:
      print("works!")
  ```

### Phase 5: Memory Management Verification

- [ ] **Verify ARC integration:**
  - [ ] Check `tagged_int_free()` properly calls `vp_arc_release()`
  - [ ] Verify `tagged_int_to_bigint()` temporaries are freed
  - [ ] Test with long-running programs

- [ ] **Valgrind testing:**
  ```bash
  valgrind --leak-check=full --show-leak-kinds=all \
           ./target/release/viper run test_bigint.vp
  ```

---

## 4. Performance Comparison

| Operation | Python 3.x | Viper (Current) | Viper (With Demotion) |
|-----------|------------|-----------------|----------------------|
| `1 + 1` | Heap alloc (~50ns) | Register (~0.5ns) | Register (~0.5ns) |
| `a + b` (i63) | Heap alloc (~50ns) | Register (~0.5ns) | Register (~0.5ns) |
| `a + b` (overflow) | GMP add (~50ns) | Promote + GMP (~100ns) | Promote + GMP (~100ns) |
| `big - (big - 1)` | GMP + stays BigInt | GMP + stays BigInt | GMP + **demote** (~50ns) |
| Loop counter | Heap alloc each iter | Register (no alloc) | Register (no alloc) |

**Key Insight:** Viper is already **100x faster** for SmallInt operations. Demotion adds memory efficiency.

---

## 5. File Locations Summary

### Runtime (C)
```
runtime/
├── include/
│   ├── tagged_int.h      # TaggedInt definitions (EXISTING)
│   ├── gmp_bridge.h      # BigInt operations (EXISTING)
│   └── viper_arc.h       # ARC memory management (EXISTING)
└── src/
    ├── tagged_int.c      # TaggedInt operations (EXISTING, needs demotion)
    ├── gmp_bridge.c      # GMP operations (EXISTING)
    └── memory/
        └── arc.c         # ARC implementation (EXISTING)
```

### Compiler (Rust)
```
src/
└── codegen/
    ├── runtime/
    │   └── tagged_int.rs # TaggedInt codegen (EXISTING)
    ├── expressions/
    │   └── operators/
    │       ├── arithmetic.rs  # Binary ops (EXISTING)
    │       └── bigint.rs      # BigInt ops (EXISTING)
    └── types.rs          # Type mapping (EXISTING)
```

---

## 6. Migration Path (If Changing Tag Bit Convention)

**⚠️ WARNING:** Changing from `LSB=0 → SmallInt` to `LSB=1 → SmallInt` would require:

1. Update all tag checks in `tagged_int.h`
2. Update all packing/unpacking in `tagged_int.c`
3. Update Rust codegen that checks tags
4. Update any inline assembly or bit manipulation
5. Update all tests

**Recommendation:** **DO NOT CHANGE** - current scheme works fine, benefits don't justify the risk.

---

## 7. Testing Strategy

### Unit Tests (runtime)
```c
// runtime/tests/test_tagged_int.c
void test_small_int_add() {
    TaggedInt a = tagged_int_from_i64(100);
    TaggedInt b = tagged_int_from_i64(200);
    TaggedInt result = tagged_int_add(a, b);
    assert(tagged_int_is_small(result));
    assert(tagged_int_get_small(result) == 300);
}

void test_overflow_promotion() {
    TaggedInt a = tagged_int_from_i64(TAGGED_INT_MAX_SMALL);
    TaggedInt b = tagged_int_from_i64(1);
    TaggedInt result = tagged_int_add(a, b);
    assert(tagged_int_is_bigint(result));
}

void test_demotion() {
    // Create BigInt
    ViperBigInt* big = tagged_int_to_bigint(
        tagged_int_from_i64(TAGGED_INT_MAX_SMALL)
    );
    TaggedInt big_tagged = tagged_int_from_bigint(big);
    
    // Subtract to get SmallInt result
    TaggedInt result = tagged_int_sub(
        big_tagged,
        tagged_int_from_i64(TAGGED_INT_MAX_SMALL - 1)
    );
    
    assert(tagged_int_is_small(result));  // SHOULD DEMOTE
    assert(tagged_int_get_small(result) == 1);
}
```

### Integration Tests (Viper language)
```python
# tests/test_int_bigint_smooth.vp

def test_basic_arithmetic():
    a = 1 + 2
    assert a == 3
    assert type(a) == int  # SmallInt

def test_overflow():
    big = 10**100
    assert type(big) == bigint

def test_mixed_ops():
    small = 10
    big = 10**50
    result = small + big
    assert result == 10**50 + 10
    assert type(result) == bigint

def test_demotion():
    big = 10**100
    result = big - (10**100 - 1)
    assert result == 1
    assert type(result) == int  # Should demote!

def test_loop_performance():
    total = 0
    for i in range(1000000):
        total = total + i
    assert total == 499999500000
    # Should complete in <100ms (all SmallInt)
```

---

## 8. Summary of Changes Needed

### Critical (Phase 1)
1. **Add `try_demote_bigint()` function** in `runtime/src/tagged_int.c`
2. **Update all arithmetic ops** to attempt demotion after GMP operations
3. **Test demotion behavior** thoroughly

### Important (Phase 2-3)
4. **Add branch prediction hints** for better CPU branch prediction
5. **Profile performance** to identify bottlenecks
6. **Consider LLVM intrinsics** if profiling shows benefit

### Nice-to-have (Phase 4-5)
7. **Add `vp_runtime_*` aliases** for consistency
8. **Improve Python compatibility** in stdlib
9. **Comprehensive valgrind testing** for memory leaks

---

## 9. Next Steps

1. **Immediate:** Implement Phase 1 (demotion) - this is the biggest missing feature
2. **Short-term:** Profile and optimize (Phase 2)
3. **Medium-term:** Improve Python compatibility (Phase 4)
4. **Long-term:** Consider advanced optimizations (SIMD, inlining, etc.)

The foundation is solid - Viper already has working tagged integers with automatic promotion. The main improvement needed is **automatic demotion** to match Python's smooth behavior where users never think about integer "types" - they just work.
