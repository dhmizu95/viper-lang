# FFT Benchmark Python Syntax Compatibility Plan

## Problem Analysis

The FFT benchmark (`benchmarks/viper/15_fft.vp`) fails with:
1. **Type inference errors** - mixing `int` and `float` causes type mismatches
2. **Boolean vs int** - Python uses `True/False`, Viper requires `int` for conditionals
3. **Complex number support** - Python's `complex()` not fully implemented
4. **JIT segfault** - Complex array operations crash at runtime
5. **Math functions** - `sin/cos/tan` were missing (now added)

## Goal

Enable 100% Python syntax compatibility for the FFT benchmark:
```python
import cmath
import math

N = 256
PI = math.pi

def fft(x, inverse=False):
    # Python-style code
    pass
```

---

## Phase 1: Core Type System Fixes

### 1.1 Automatic int→float Coercion
**Problem:** Viper requires explicit `float()` casts, Python does automatic coercion.

**Current (broken):**
```python
angle = 2.0 * PI / float(length)  # Requires explicit cast
```

**Target (Python):**
```python
angle = 2 * PI / length  # Automatic coercion
```

**Implementation:**
- Modify `src/semantic/type_checker.rs` to allow implicit int→float conversion in binary ops
- Add coercion in `src/codegen/expressions/binary_ops.rs`

**Files to modify:**
- `src/semantic/type_checker.rs` - Add coercion rules
- `src/codegen/expressions/binary_ops.rs` - Insert float casts

---

### 1.2 Boolean Type Compatibility
**Problem:** Python `True/False` work in conditionals, Viper requires `bool` type.

**Current (broken):**
```python
def fft(x, inverse: int):  # Must use int
    if inverse != 0:       # Must compare to 0
```

**Target (Python):**
```python
def fft(x, inverse=False):  # bool default
    if inverse:             # Direct bool check
```

**Implementation:**
- Ensure `bool` type is properly handled in conditionals
- Add implicit bool→int conversion where needed

**Files to modify:**
- `src/semantic/type_checker.rs` - Bool in conditionals
- `src/codegen/expressions/core.rs` - Bool handling

---

## Phase 2: Complex Number Support

### 2.1 Enable `cmath` Module
**Problem:** `cmath.complex()` class exists but has parser issues.

**Current status:** `std/core/cmath.vp` has `complex` class but parser fails on class syntax.

**Implementation:**
1. Fix class parsing in `std/core/cmath.vp`
2. Ensure `__mul__`, `__add__`, `__sub__` operators work
3. Add `abs()` for complex magnitude

**Files to modify:**
- `std/core/cmath.vp` - Fix class syntax
- `src/parser/expressions.rs` - Class parsing

---

### 2.2 Complex List Support
**Problem:** Lists of complex numbers `[complex(1,0)] * N` need proper initialization.

**Target (Python):**
```python
signal = [complex(0, 0)] * N  # List of complex zeros
```

**Implementation:**
- Ensure list multiplication works with complex type
- Add complex type to list element type inference

**Files to modify:**
- `src/semantic/type_checker.rs` - List type inference
- `src/codegen/containers/lists.rs` - Complex list handling

---

## Phase 3: FFT-Specific Fixes

### 3.1 For Loop with Range
**Problem:** Python `for i in range(n)` should work without type annotations.

**Target (Python):**
```python
for i in range(n):
    t = i / sample_rate  # Auto-convert to float
```

**Implementation:**
- Ensure `range()` returns proper iterator type
- Add int→float coercion in division context

**Files to modify:**
- `src/codegen/expressions/builtins/range.rs`
- `src/semantic/type_checker.rs`

---

### 3.2 In-place Operators
**Problem:** Python `+=`, `-=`, `*=` should work with floats.

**Target (Python):**
```python
total += abs(c)  # Float accumulation
```

**Implementation:**
- Ensure augmented assignment handles float types
- Add proper type inference for compound ops

**Files to modify:**
- `src/codegen/statements/augmented_assign.rs`

---

### 3.3 F-string Formatting
**Problem:** Python `f"{value:.6f}"` formatting.

**Target (Python):**
```python
print(f"{magnitude:.6f}")
```

**Current workaround:**
```python
print("{:.6f}".format(magnitude))
```

**Implementation:**
- Add f-string parsing support
- Implement format spec handling

**Files to modify:**
- `src/parser/expressions.rs` - F-string parsing
- `src/codegen/expressions/strings.rs` - Format implementation

---

## Phase 4: JIT Runtime Fixes

### 4.1 Fix Array Indexing Segfault
**Problem:** Complex array operations cause JIT segfault.

**Root cause:** Likely LLVM type mismatch in array element access.

**Implementation:**
1. Add debug output to trace array access codegen
2. Verify LLVM IR types match expected signatures
3. Check array bounds and pointer types

**Files to modify:**
- `src/codegen/expressions/subscript.rs` - Array access
- `src/codegen/types.rs` - Type lowering

---

### 4.2 Math Function JIT Stubs
**Status:** ✅ Already added `vp_math_sin`, `vp_math_cos`, `vp_math_tan`

**Verify:**
- All math functions have JIT stubs
- Runtime C implementations exist in `runtime/src/math_mod.c`

---

## Phase 5: Testing & Validation

### 5.1 Unit Tests
Add tests for:
- [ ] int→float automatic coercion
- [ ] Complex number arithmetic
- [ ] Complex list operations
- [ ] Math functions (sin, cos, tan)
- [ ] F-string formatting

**Files to create:**
- `tests/unit/float_coercion.rs`
- `tests/integration/complex_numbers.rs`

---

### 5.2 Benchmark Validation
Compare output across languages:
```
C:      384.000000  -0.000000
Python: 384.000000  -0.000000
Viper:  ???         ???
```

**Test command:**
```bash
cd benchmarks
./benchmark_runner.sh 15_fft
```

---

## Implementation Priority

| Priority | Task | Estimated Effort |
|----------|------|------------------|
| P0 | Fix JIT array indexing segfault | 4-8 hours |
| P0 | Automatic int→float coercion | 2-4 hours |
| P1 | Boolean type in conditionals | 1-2 hours |
| P1 | Fix cmath class parsing | 2-4 hours |
| P2 | Complex list support | 2-4 hours |
| P2 | F-string formatting | 2-4 hours |
| P3 | For loop range type inference | 1-2 hours |

**Total estimated effort:** 14-28 hours

---

## Immediate Action Items

1. **Debug JIT segfault** - Run with `RUST_BACKTRACE=1` to get stack trace
2. **Check LLVM IR** - Use `viper build -emit-llvm` to inspect generated IR
3. **Simplify FFT** - Create minimal test case that reproduces the crash
4. **Add coercion** - Implement int→float automatic conversion
5. **Test incrementally** - Verify each fix before moving to next

---

## Minimal Test Case

Create simplified test to isolate the crash:

```python
# test_fft_simple.vp
import math

def test():
    real = [0.0] * 4
    imag = [0.0] * 4
    
    # Simple FFT butterfly
    w_r = 1.0
    w_i = 0.0
    t_r = real[2] * w_r - imag[2] * w_i
    t_i = real[2] * w_i + imag[2] * w_r
    
    real[0] = real[0] + t_r
    imag[0] = imag[0] + t_i
    
    print(real[0])
    print(imag[0])

test()
```

---

## Success Criteria

✅ Viper FFT produces same output as C/Python (384.000000, -0.000000)
✅ No type annotations required in benchmark code
✅ 100% Python syntax compatibility
✅ No JIT crashes or segfaults
✅ Passes `./build.sh --test --benchmark`
