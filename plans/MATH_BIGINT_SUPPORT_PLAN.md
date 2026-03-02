# Python-Compatible BigInt Math Support Plan

## Overview

Add Python-style BigInt support to Viper's `math` module and built-in functions. Python's `math` module seamlessly handles large integers with functions like `math.isqrt()`, `math.gcd()`, `math.factorial()`, etc. Viper currently has separate `_bigint` suffixed functions that aren't integrated into the standard `math` module.

**Goal:** Achieve Python compatibility where math functions automatically work with BigInt without requiring separate function names.

---

## Current State

### What Exists
- `sqrt_bigint(x: bigint) -> bigint` - in prelude
- `abs_bigint(x: bigint) -> bigint` - in prelude
- `pow_bigint(base: bigint, exp: bigint) -> bigint` - in prelude
- `min_bigint(a: bigint, b: bigint) -> bigint` - in prelude
- `max_bigint(a: bigint, b: bigint) -> bigint` - in prelude
- `int_bigint(x: bigint) -> int` - in prelude

### What's Missing (Python Compatibility Gap)
| Python Function | Viper Equivalent | Status |
|-----------------|------------------|--------|
| `math.isqrt(n)` | ❌ None | Missing |
| `math.gcd(a, b)` | `gcd(a: int, b: int)` | int only |
| `math.lcm(a, b)` | `lcm(a: int, b: int)` | int only |
| `math.factorial(n)` | `factorial(n: int)` | int only, overflows |
| `math.comb(n, k)` | `comb(n: int, k: int)` | int only |
| `math.perm(n, k)` | `perm(n: int, k: int)` | int only |
| `pow(a, b, mod)` | `pow_bigint(base, exp)` | No mod param |
| `abs(x)` | `abs(x: int)` | int only |
| `min(a, b)` | `min(a: int, b: int)` | int only |
| `max(a, b)` | `max(a: int, b: int)` | int only |

---

## Design Principles

1. **Function Overloading**: Single function name works with both `int` and `bigint`
2. **Auto-promotion**: Operations that would overflow `int` automatically promote to `bigint`
3. **Python API Compatibility**: Match Python's `math` module function signatures
4. **Return Type Inference**: Return type matches input type (int→int, bigint→bigint)

---

## Phase 1: Core Integer-Theoretic Functions

### 1.1 `math.isqrt(n: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> import math
>>> math.isqrt(10**100)
10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000
```

**Implementation:**
```viper
# In std/core/math.vp
def isqrt(n: int) -> int:
    """Return the integer square root of nonnegative integer n."""
    if n < 0:
        raise ValueError("isqrt() argument must be nonnegative")
    return vp_math_isqrt(n)

def isqrt_bigint(n: bigint) -> bigint:
    """Return the integer square root of nonnegative bigint n."""
    if n < bigint("0"):
        raise ValueError("isqrt() argument must be nonnegative")
    return sqrt_bigint(n)

# Overloaded version (compiler magic)
def isqrt(n: int | bigint) -> int | bigint:
    # Compiler dispatches based on type
    pass
```

**Runtime function needed:**
```c
// runtime/src/gmp_bridge.c
ViperBigInt* vp_bigint_isqrt(ViperBigInt* n);
```

**LLVM declaration:**
```rust
// src/codegen/runtime/math.rs
let bigint_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
let isqrt_bigint_type = bigint_ptr_type.fn_type(&[bigint_ptr_type.into()], false);
module.add_function("vp_bigint_isqrt", isqrt_bigint_type, None);
```

**Tests:**
```viper
# tests/test_math_bigint.vp
def test_isqrt_int():
    assert math.isqrt(0) == 0
    assert math.isqrt(1) == 1
    assert math.isqrt(16) == 4
    assert math.isqrt(100) == 10

def test_isqrt_bigint():
    n = bigint("100000000000000000000000000000")
    result = math.isqrt(n)
    assert result * result <= n
    assert (result + bigint("1")) * (result + bigint("1")) > n

def test_isqrt_large():
    # 10^100
    n = bigint("1" + "0" * 100)
    result = math.isqrt(n)
    # Should be 10^50
    expected = bigint("1" + "0" * 50)
    assert result == expected
```

---

### 1.2 `math.gcd(a: int | bigint, b: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> math.gcd(10**50, 15**50)
31250000000000000000000000000000000000000000000000
```

**Implementation:**
```viper
def gcd(a: int, b: int) -> int:
    """Greatest common divisor of a and b."""
    return vp_math_gcd(a, b)

def gcd_bigint(a: bigint, b: bigint) -> bigint:
    """Greatest common divisor of bigint a and bigint b."""
    return vp_bigint_gcd(a, b)

def gcd(a: int | bigint, b: int | bigint) -> int | bigint:
    # Compiler dispatches based on type
    pass
```

**Runtime function needed:**
```c
ViperBigInt* vp_bigint_gcd(ViperBigInt* a, ViperBigInt* b);
```

**Tests:**
```viper
def test_gcd_int():
    assert math.gcd(48, 18) == 6
    assert math.gcd(0, 5) == 5
    assert math.gcd(7, 0) == 7

def test_gcd_bigint():
    a = bigint("123456789012345678901234567890")
    b = bigint("987654321098765432109876543210")
    result = math.gcd(a, b)
    assert a % result == bigint("0")
    assert b % result == bigint("0")
```

---

### 1.3 `math.lcm(a: int | bigint, b: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> math.lcm(10**30, 15**30)
# Returns large integer
```

**Implementation:**
```viper
def lcm(a: int, b: int) -> int:
    """Least common multiple of a and b."""
    if a == 0 or b == 0:
        return 0
    return abs(a * b) // math.gcd(a, b)

def lcm_bigint(a: bigint, b: bigint) -> bigint:
    """Least common multiple of bigint a and bigint b."""
    if a == bigint("0") or b == bigint("0"):
        return bigint("0")
    return abs_bigint(a * b) // math.gcd_bigint(a, b)
```

**Tests:**
```viper
def test_lcm_int():
    assert math.lcm(4, 6) == 12
    assert math.lcm(3, 5) == 15

def test_lcm_bigint():
    a = bigint("1000000000000")
    b = bigint("1500000000000")
    result = math.lcm(a, b)
    assert result % a == bigint("0")
    assert result % b == bigint("0")
```

---

### 1.4 `math.factorial(n: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> math.factorial(100)
# Returns 158-digit number (automatically BigInt)
```

**Implementation:**
```viper
def factorial(n: int) -> int:
    """Return n! (n factorial)."""
    if n < 0:
        raise ValueError("factorial() not defined for negative numbers")
    if n > 20:
        # Auto-promote to bigint for large results
        return _factorial_bigint(bigint(n))
    return vp_math_factorial(n)

def factorial_bigint(n: bigint) -> bigint:
    """Return n! (n factorial) for bigint."""
    if n < bigint("0"):
        raise ValueError("factorial() not defined for negative numbers")
    return _factorial_bigint(n)

def _factorial_bigint(n: bigint) -> bigint:
    """Internal factorial using BigInt arithmetic."""
    result = bigint("1")
    i = bigint("1")
    while i <= n:
        result = result * i
        i = i + bigint("1")
    return result
```

**Runtime function (optional optimization):**
```c
ViperBigInt* vp_bigint_factorial(ViperBigInt* n);
```

**Tests:**
```viper
def test_factorial_small():
    assert math.factorial(0) == 1
    assert math.factorial(1) == 1
    assert math.factorial(5) == 120
    assert math.factorial(10) == 3628800

def test_factorial_large():
    # 100! has 158 digits
    result = math.factorial(100)
    # Verify it's a bigint and has correct properties
    assert result > bigint("10") ** bigint("150")
    
def test_factorial_bigint():
    n = bigint("50")
    result = math.factorial(n)
    assert result > bigint("10") ** bigint("60")
```

---

### 1.5 `math.comb(n: int | bigint, k: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> math.comb(1000, 500)
# Very large number
```

**Implementation:**
```viper
def comb(n: int, k: int) -> int:
    """Number of ways to choose k items from n items."""
    if k < 0 or k > n:
        return 0
    if k == 0 or k == n:
        return 1
    if k > n // 2:
        k = n - k
    # Use BigInt for intermediate calculations
    result = bigint("1")
    for i in range(k):
        result = result * bigint(n - i) // bigint(i + 1)
    return int_bigint(result)

def comb_bigint(n: bigint, k: bigint) -> bigint:
    """Number of ways to choose k items from n items (bigint)."""
    if k < bigint("0") or k > n:
        return bigint("0")
    if k == bigint("0") or k == n:
        return bigint("1")
    if k > n // bigint("2"):
        k = n - k
    result = bigint("1")
    i = bigint("0")
    while i < k:
        result = result * (n - i) // (i + bigint("1"))
        i = i + bigint("1")
    return result
```

**Tests:**
```viper
def test_comb_int():
    assert math.comb(5, 2) == 10
    assert math.comb(10, 5) == 252

def test_comb_large():
    result = math.comb(100, 50)
    assert result > 0
```

---

### 1.6 `math.perm(n: int | bigint, k: int | bigint) -> int | bigint`

**Python behavior:**
```python
>>> math.perm(1000, 500)
# Very large number
```

**Implementation:**
```viper
def perm(n: int, k: int) -> int:
    """Number of ways to choose k items from n items with order."""
    if k < 0 or k > n:
        return 0
    result = 1
    for i in range(k):
        result = result * (n - i)
    return result

def perm_bigint(n: bigint, k: bigint) -> bigint:
    """Number of ways to choose k items from n items with order (bigint)."""
    if k < bigint("0") or k > n:
        return bigint("0")
    result = bigint("1")
    i = bigint("0")
    while i < k:
        result = result * (n - i)
        i = i + bigint("1")
    return result
```

---

## Phase 2: Modular Arithmetic

### 2.1 `pow(base: int | bigint, exp: int | bigint, mod: int | bigint = None) -> int | bigint`

**Python behavior:**
```python
>>> pow(2, 1000, 10**9 + 7)
# Modular exponentiation
>>> pow(2, 10000)
# Very large number (BigInt)
```

**Implementation:**
```viper
def pow(base: int, exp: int, mod: int = None) -> int:
    """Return base raised to power exp, modulo mod if given."""
    if mod is not None:
        return vp_pow_mod(base, exp, mod)
    if exp < 0:
        raise ValueError("Negative exponent for int")
    if exp > 30:
        # Auto-promote to bigint
        return pow_bigint(bigint(base), bigint(exp), None)
    return vp_pow_i64(base, exp)

def pow_bigint(base: bigint, exp: bigint, mod: bigint = None) -> bigint:
    """Return base raised to power exp, modulo mod if given."""
    if mod is not None:
        return vp_bigint_pow_mod(base, exp, mod)
    return vp_bigint_pow(base, exp)
```

**Runtime functions needed:**
```c
ViperBigInt* vp_bigint_pow(ViperBigInt* base, ViperBigInt* exp);
ViperBigInt* vp_bigint_pow_mod(ViperBigInt* base, ViperBigInt* exp, ViperBigInt* mod);
int64_t vp_pow_mod(int64_t base, int64_t exp, int64_t mod);
```

**Tests:**
```viper
def test_pow_int():
    assert pow(2, 10) == 1024
    assert pow(3, 4) == 81

def test_pow_mod():
    assert pow(2, 10, 1000) == 24
    assert pow(3, 100, 1000000007) == 10460353208 % 1000000007

def test_pow_bigint():
    result = pow(bigint("2"), bigint("1000"))
    assert result > bigint("10") ** bigint("300")

def test_pow_bigint_mod():
    base = bigint("2")
    exp = bigint("10000")
    mod = bigint("1000000007")
    result = pow(base, exp, mod)
    assert result < mod
```

---

## Phase 3: Overloaded Built-in Functions

### 3.1 Update `abs()`, `min()`, `max()` in Prelude

**Current:**
```viper
def abs(x: int) -> int:
def min(a: int, b: int) -> int:
def max(a: int, b: int) -> int:
```

**Updated:**
```viper
def abs(x: int) -> int:
    if x < 0:
        return -x
    return x

def abs_bigint(x: bigint) -> bigint:
    if x < bigint("0"):
        return -x
    return x

def abs(x: int | bigint) -> int | bigint:
    # Compiler dispatches based on type
    pass

def min(a: int, b: int) -> int:
    if a < b:
        return a
    return b

def min_bigint(a: bigint, b: bigint) -> bigint:
    if a < b:
        return a
    return b

def min(a: int | bigint, b: int | bigint) -> int | bigint:
    # Compiler dispatches based on type
    pass

def max(a: int, b: int) -> int:
    if a > b:
        return a
    return b

def max_bigint(a: bigint, b: bigint) -> bigint:
    if a > b:
        return a
    return b

def max(a: int | bigint, b: int | bigint) -> int | bigint:
    # Compiler dispatches based on type
    pass
```

---

### 3.2 Update `sum()` for BigInt Lists

**Current:**
```viper
def sum(items: [int]) -> int:
    total = 0
    for item in items:
        total = total + item
    return total
```

**Updated:**
```viper
def sum_int(items: [int]) -> int:
    total = 0
    for item in items:
        total = total + item
    return total

def sum_bigint(items: [bigint]) -> bigint:
    total = bigint("0")
    for item in items:
        total = total + item
    return total

def sum(items: [int] | [bigint]) -> int | bigint:
    # Compiler dispatches based on element type
    pass
```

---

## Phase 4: Auto-Promotion in Arithmetic

### 4.1 TaggedInt Auto-Promotion

**Goal:** Arithmetic operations that overflow `int` automatically promote to `bigint`.

**Implementation location:** `runtime/include/tagged_int.h`, `runtime/src/tagged_int.c`

**Example behavior:**
```viper
a: int = 10**18  # Still fits in i64
b: int = a * 10  # Overflows, auto-promotes to bigint
# b is now bigint type
```

**Implementation approach:**
1. Check for overflow after each arithmetic operation
2. If overflow detected, allocate BigInt and recompute
3. Return tagged value with BigInt pointer

**Runtime function:**
```c
TaggedInt tagged_int_mul_checked(TaggedInt a, TaggedInt b);
TaggedInt tagged_int_add_checked(TaggedInt a, TaggedInt b);
// etc.
```

---

## Phase 5: Math Module Integration

### 5.1 Update `std/core/math.vp`

Add all BigInt-capable functions to the math module:

```viper
# At end of math.vp

# ============================================
# BigInt Support (Python compatibility)
# ============================================

def isqrt(n: int | bigint) -> int | bigint:
    """Integer square root."""
    if isinstance(n, bigint):
        return isqrt_bigint(n)
    return isqrt_int(n)

def gcd(a: int | bigint, b: int | bigint) -> int | bigint:
    """Greatest common divisor."""
    if isinstance(a, bigint) or isinstance(b, bigint):
        return gcd_bigint(
            bigint(a) if not isinstance(a, bigint) else a,
            bigint(b) if not isinstance(b, bigint) else b
        )
    return gcd_int(a, b)

def lcm(a: int | bigint, b: int | bigint) -> int | bigint:
    """Least common multiple."""
    if isinstance(a, bigint) or isinstance(b, bigint):
        return lcm_bigint(
            bigint(a) if not isinstance(a, bigint) else a,
            bigint(b) if not isinstance(b, bigint) else b
        )
    return lcm_int(a, b)

def factorial(n: int | bigint) -> int | bigint:
    """Factorial."""
    if isinstance(n, bigint):
        return factorial_bigint(n)
    if n > 20:
        return factorial_bigint(bigint(n))
    return factorial_int(n)

def comb(n: int | bigint, k: int | bigint) -> int | bigint:
    """Combinations."""
    if isinstance(n, bigint) or isinstance(k, bigint):
        return comb_bigint(
            bigint(n) if not isinstance(n, bigint) else n,
            bigint(k) if not isinstance(k, bigint) else k
        )
    return comb_int(n, k)

def perm(n: int | bigint, k: int | bigint) -> int | bigint:
    """Permutations."""
    if isinstance(n, bigint) or isinstance(k, bigint):
        return perm_bigint(
            bigint(n) if not isinstance(n, bigint) else n,
            bigint(k) if not isinstance(k, bigint) else k
        )
    return perm_int(n, k)
```

---

## Implementation Checklist

### Runtime (C)
- [ ] `vp_bigint_isqrt()` - Integer square root
- [ ] `vp_bigint_gcd()` - Greatest common divisor
- [ ] `vp_bigint_pow()` - Power with BigInt
- [ ] `vp_bigint_pow_mod()` - Modular exponentiation
- [ ] `vp_bigint_factorial()` - Factorial (optional optimization)
- [ ] `vp_pow_mod()` - Modular exponentiation for int64

### Codegen (Rust)
- [ ] `src/codegen/runtime/math.rs` - Add BigInt function declarations
- [ ] `src/codegen/expressions/builtins.rs` - Update math builtin dispatch
- [ ] Type checker support for overloaded functions

### Standard Library (Viper)
- [ ] `std/core/math.vp` - Add BigInt versions of all functions
- [ ] `std/prelude.vp` - Update `abs`, `min`, `max`, `sum` for BigInt
- [ ] Add `isqrt` function

### Tests
- [ ] `tests/test_math_bigint.vp` - Comprehensive BigInt math tests
- [ ] `tests/test_math_overloads.vp` - Test int/bigint overloading
- [ ] `tests/test_pow_mod.vp` - Modular exponentiation tests
- [ ] Update existing math tests to verify BigInt compatibility

### Documentation
- [ ] Update `CORE_LANGUAGE_FEATURES.md` - BigInt math support
- [ ] Update `std/core/math.vp` docstrings
- [ ] Add examples to docs/

---

## Testing Strategy

### Unit Tests
```viper
# tests/test_math_bigint.vp

def test_isqrt_perfect_square():
    assert math.isqrt(144) == 12
    assert math.isqrt(bigint("144")) == bigint("12")

def test_isqrt_non_perfect():
    assert math.isqrt(20) == 4  # floor(sqrt(20))
    
def test_gcd_coprime():
    assert math.gcd(17, 19) == 1
    
def test_gcd_bigint():
    a = bigint("12345678901234567890")
    b = bigint("98765432109876543210")
    result = math.gcd(a, b)
    assert result > bigint("0")

def test_factorial_growth():
    f100 = math.factorial(100)
    f101 = math.factorial(101)
    assert f101 == f100 * bigint("101")

def test_pow_mod_correct():
    # Fermat's little theorem: a^(p-1) ≡ 1 (mod p) for prime p
    p = bigint("1000000007")  # Prime
    a = bigint("12345")
    result = pow(a, p - bigint("1"), p)
    assert result == bigint("1")
```

### Integration Tests
```viper
# tests/test_bigint_integration.vp

def test_rsa_style_computation():
    # RSA-style modular exponentiation with large numbers
    p = bigint("61")
    q = bigint("53")
    n = p * q  # 3233
    phi = (p - bigint("1")) * (q - bigint("1"))  # 3120
    e = bigint("17")
    d = pow(e, phi - bigint("2"), phi)  # Modular inverse (simplified)
    message = bigint("65")
    encrypted = pow(message, e, n)
    decrypted = pow(encrypted, d, n)
    assert decrypted == message
```

---

## Performance Considerations

1. **Small integers**: Keep using native i64 operations (fast path)
2. **Large integers**: Use GMP-backed BigInt (correctness over speed)
3. **Modular exponentiation**: Use square-and-multiply algorithm
4. **Factorial**: Cache small factorials, use iterative for large
5. **GCD**: Use binary GCD algorithm (Stein's algorithm) for BigInt

---

## Milestones

| Milestone | Functions | Target |
|-----------|-----------|--------|
| M1: Core | `isqrt`, `gcd`, `lcm` | Week 1 |
| M2: Combinatorics | `factorial`, `comb`, `perm` | Week 2 |
| M3: Modular | `pow(base, exp, mod)` | Week 3 |
| M4: Overloads | `abs`, `min`, `max`, `sum` | Week 4 |
| M5: Auto-promotion | TaggedInt overflow | Week 5-6 |
| M6: Polish | Tests, docs, benchmarks | Week 7 |

---

## Success Criteria

1. ✅ All Python `math` module integer functions work with BigInt
2. ✅ `math.isqrt(10**100)` returns correct result
3. ✅ `math.factorial(1000)` computes without overflow
4. ✅ `pow(2, 10000, 10**9+7)` uses modular exponentiation
5. ✅ Existing int-only code continues to work (no regression)
6. ✅ Performance within 2x of Python for BigInt operations

---

## Related Files to Modify

```
runtime/
├── include/
│   ├── gmp_bridge.h          # Add vp_bigint_isqrt, vp_bigint_gcd, etc.
│   └── tagged_int.h          # Add overflow-checked arithmetic
├── src/
│   ├── gmp_bridge.c          # Implement BigInt math functions
│   └── tagged_int.c          # Implement auto-promotion

src/
├── codegen/
│   ├── runtime/
│   │   └── math.rs           # Add BigInt function declarations
│   └── expressions/
│       └── builtins.rs       # Update math builtin dispatch
└── semantic/
    └── type_checker/
        └── exprs.rs          # Support overloaded functions

std/
├── core/
│   └── math.vp               # Add BigInt versions of all functions
└── prelude.vp                # Update abs, min, max, sum

tests/
├── test_math_bigint.vp       # BigInt math tests
├── test_math_overloads.vp    # Overloading tests
└── test_pow_mod.vp           # Modular exponentiation tests
```

---

## References

- Python `math` module: https://docs.python.org/3/library/math.html
- GMP documentation: https://gmplib.org/manual/
- Viper BigInt implementation: `runtime/src/gmp_bridge.c`
- Python integer square root: `math.isqrt()` (Python 3.8+)
