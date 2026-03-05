# Python-Style Integer Implementation Plan

## Goal

Remove all user-facing `BigInt`/`bigint` type, `BigInt()` constructor, `bigint()` function, and `n` suffix.
Use Python-style `int` type that automatically handles arbitrary precision integers.

**User sees only:** `int` (which is internally tagged: small int or BigInt on overflow)

---

## Current State

### What Exists Now:
1. **`n` suffix** in lexer: `123n` → `TokenKind::BigInt("123")`
2. **`BigInt()` constructor**: `BigInt(123)` or `BigInt("123")`
3. **`bigint` type annotation**: `x: bigint = 123`
4. **`BigInt` type annotation**: `x: BigInt = 123`
5. **Helper functions**: `str_bigint()`, `int_bigint()`, `abs_bigint()`, `pow_bigint()`, etc.
6. **Auto-promotion** in type checker: integers can be assigned to BigInt variables

### Type System:
- `Type::I8` - 8-bit integer
- `Type::I16` - 16-bit integer
- `Type::I32` - 32-bit integer
- `Type::I64` - 64-bit integer
- `Type::Int` - Tagged integer (auto-promoting)
- `Type::BigInt` - Arbitrary precision (GMP-based) - **to be removed from user view**

### Files Using `n` Suffix and BigInt():
- Tests: `tests/test_bigint_number_theory.vp`, `tests/bigint_*.vp`, `benchmarks/bigint/bigint_demo.vp`
- Constructor calls in: `src/codegen/expressions/calls.rs`
- Token scanning: `src/lexer/scanner.rs`
- Type parsing: `src/parser/statements/definitions.rs`
- Type compatibility: `src/semantic/type_checker/compatibility.rs`

---

## Desired Python Style

```python
# Type annotation for arbitrary precision int
a: int = 12364546546546546546546546546

# Auto-promotion for large literals (no suffix needed)
large = 123456789012345678901234567890

# Function with type annotations
def factorial(n: int) -> int:
    if n == 0:
        return 1  # Auto-promoted due to return type
    return n * factorial(n - 1)

# Function without type annotations (inferred)
def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n - 1)

# Arithmetic with large integers
a = 123456789012345678901234567890
b = 987654321098765432109876543210
c = a + b  # Works automatically
```

---

## Implementation Strategy

### Phase 1: Remove `bigint`/`BigInt` from User-Facing Type System

**Key Insight:** `Type::BigInt` already exists internally for GMP-based integers.
We need to:
1. Remove `bigint` and `BigInt` as parseable type names
2. Keep `Type::BigInt` internally for codegen
3. Make `int` always map to auto-promoting tagged integers (`Type::Int`)

**Files to modify:**

#### 1. `src/parser/statements/definitions.rs`
Remove `bigint`/`BigInt` as valid type annotations:
```rust
// Remove these lines:
"bigint" => Type::BigInt,
"BigInt" => Type::BigInt,
```

Now `int` is the only user-facing integer type that can be arbitrary precision.

#### 2. `src/ast/types.rs`
- Update `Display` impl to keep `BigInt` for internal error messages only
- Update `is_numeric()`, `is_integer()`, `is_hashable()` to treat `Type::Int` same as `Type::BigInt`

---

### Phase 2: Remove `n` Suffix from Lexer

**File: `src/lexer/scanner.rs`**

Remove the 'n' suffix detection:
```rust
// REMOVE this block:
if let Some('n') = self.peek() {
    self.advance(); // consume 'n'
    Ok(TokenKind::BigInt(s.clone()))
}
```

**Result:**
- `123n` → Syntax error (unexpected identifier)
- `123456789012345678901234567890` → BigInt token (too large for i64)
- `123` → Int token

**File: `src/lexer/tokens.rs`**
Keep `TokenKind::BigInt(String)` for large integer literals that don't fit in i64.

---

### Phase 3: Remove `BigInt()` Constructor

**File: `src/codegen/expressions/calls.rs`**

Remove `generate_bigint_constructor()` function and its usage.

**Migration:**
```python
# Old (removed):
x = BigInt(123)
y = BigInt("123456789012345678901234567890")

# New:
x: int = 123  # or just x = 123 if context infers int
y = 123456789012345678901234567890  # Auto-promoted (too large for i64)
```

---

### Phase 4: Update Type Checker Compatibility

**File: `src/semantic/type_checker/compatibility.rs`**

The key change: Make `Type::Int` the primary arbitrary-precision integer type.

Currently:
```rust
pub(crate) fn is_numeric(&self, t: &Type) -> bool {
    matches!(t, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::Int | Type::F64 | Type::F32 | Type::BigInt)
}
```

This is already correct - `Type::Int` is listed.

Update compatibility rules to ensure `Type::Int` ↔ `Type::BigInt` compatibility:
```rust
pub(crate) fn is_compatible(&self, expected: &Type, actual: &Type) -> bool {
    // ... existing code ...

    // Int (tagged arbitrary precision) is compatible with integer literals
    (Type::Int, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
    (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::Int) => true,

    // BigInt internal type compatibility (for internal use)
    (Type::BigInt, Type::I64 | Type::I32 | Type::I16 | Type::I8) => true,
    (Type::BigInt, Type::BigInt) => true,
    (Type::I64 | Type::I32 | Type::I16 | Type::I8, Type::BigInt) => true,

    // Int and BigInt are compatible (both are arbitrary precision)
    (Type::Int, Type::BigInt) | (Type::BigInt, Type::Int) => true,

    // ... rest of code ...
}
```

---

### Phase 5: Rename Helper Functions to Python Style

Rename all `*_bigint` functions to clean Python-style names:

| Old Name | New Name | Description |
|----------|----------|-------------|
| `str_bigint(x)` | `str(x)` | convert int to string |
| `int_bigint(x)` | `int(x)` | convert to int (from string) |
| `abs_bigint(x)` | `abs(x)` | absolute value |
| `pow_bigint(base, exp)` | `pow(base, exp)` | power operation |
| `sqrt_bigint(x)` | `sqrt(x)` | square root |
| `gcd_bigint(a, b)` | `gcd(a, b)` | greatest common divisor |
| `lcm_bigint(a, b)` | `lcm(a, b)` | least common multiple |
| `factorial_bigint(n)` | `factorial(n)` | factorial |
| `comb_bigint(n, k)` | `comb(n, k)` | combinations |
| `perm_bigint(n, k)` | `perm(n, k)` | permutations |
| `min_bigint(a, b)` | `min(a, b)` | minimum |
| `max_bigint(a, b)` | `max(a, b)` | maximum |
| `is_zero_bigint(x)` | `is_zero(x)` | check if zero |
| `is_negative_bigint(x)` | `is_negative(x)` | check if negative |
| `sign_bigint(x)` | `sign(x)` | sign of value |
| `bit_length_bigint(x)` | `bit_length(x)` | bit length |

**Files to modify:**
- `src/semantic/symbol_table.rs` - Update builtin signatures
- `src/semantic/type_checker/hindley_milner.rs` - Update function type signatures
- `src/codegen/expressions/calls.rs` - Update code generation for renamed functions
- `src/codegen/expressions/builtins.rs` - Update builtin handling
- `src/codegen/expressions/operators/bigint.rs` - Update operator handling
- `src/codegen/statements/assignment.rs` - Update assignment handling
- `src/codegen/statements/declaration.rs` - Update declaration handling
- All test files using these functions

---

## Implementation Tasks

### Task 1: Remove `bigint`/`BigInt` Type from Parser
- [ ] Edit `src/parser/statements/definitions.rs`: Remove `"bigint" => Type::BigInt` and `"BigInt" => Type::BigInt`
- [ ] Verify `int` maps to `Type::Int`

### Task 2: Remove `n` Suffix from Lexer
- [ ] Edit `src/lexer/scanner.rs`: Remove 'n' suffix detection
- [ ] Verify large integers still become BigInt tokens

### Task 3: Remove `BigInt()` Constructor
- [ ] Edit `src/codegen/expressions/calls.rs`: Remove `generate_bigint_constructor()`
- [ ] Remove BigInt handling in call generation

### Task 4: Update Type Checker
- [ ] Edit `src/semantic/type_checker/compatibility.rs`: Ensure `Type::Int` ↔ `Type::BigInt` compatibility
- [ ] Test return type promotion in functions
- [ ] Test variable assignment with type annotations

### Task 5: Update All Type References
- [ ] `src/ast/types.rs`: Update `is_numeric()`, `is_integer()`, `is_hashable()` for `Type::Int`
- [ ] `src/codegen/types.rs`: Ensure `Type::Int` and `Type::BigInt` use same representation
- [ ] `src/semantic/symbol_table.rs`: Update builtin signatures to use `Type::Int`
- [ ] `src/utils/mangling.rs`: Update type mangling for `Type::Int`
- [ ] `src/semantic/monomorphization.rs`: Update monomorphization for `Type::Int`

### Task 6: Update Tests and Documentation
- [ ] Update all `.vp` test files to use `int` instead of `BigInt`/`bigint`
- [ ] Remove `n` suffix from all test files
- [ ] Remove `BigInt()` constructor calls
- [ ] Update documentation

---

## Migration Examples

### Type Annotations
```python
# Before:
a: BigInt = 123
b: bigint = 456

# After:
a: int = 123
b: int = 456
```

### Large Literals
```python
# Before:
x = 123456789012345678901234567890n
y = BigInt("123456789012345678901234567890")

# After:
x = 123456789012345678901234567890  # Auto-promoted
y: int = 123456789012345678901234567890
```

### Function Signatures
```python
# Before:
def factorial(n: BigInt) -> BigInt:
    if n == 0:
        return 1
    return n * factorial(n - 1)

# After:
def factorial(n: int) -> int:
    if n == 0:
        return 1
    return n * factorial(n - 1)

# Or inferred:
def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n - 1)
```

### Helper Functions
```python
# Before and After (unchanged for now):
s = str_bigint(x)
a = abs_bigint(x)
p = pow_bigint(base, exp)
g = gcd_bigint(a, b)
```

---

## Files to Modify

1. `src/lexer/scanner.rs` - Remove 'n' suffix
2. `src/lexer/tokens.rs` - Keep `TokenKind::BigInt` for large literals only
3. `src/parser/statements/definitions.rs` - Remove `bigint`/`BigInt` type
4. `src/codegen/expressions/calls.rs` - Remove `BigInt()` constructor
5. `src/semantic/type_checker/compatibility.rs` - Update compatibility rules
6. `src/ast/types.rs` - Update type predicates
7. `src/codegen/types.rs` - Ensure Int/BigInt codegen consistency
8. `src/semantic/symbol_table.rs` - Update builtin signatures
9. `src/utils/mangling.rs` - Update type mangling
10. `src/semantic/monomorphization.rs` - Update monomorphization
11. All test files (`.vp` files)
12. Documentation files

---

## Files to Keep Unchanged

1. `src/ast/nodes.rs` - `Expr::BigInt` remains for large literal representation
2. Runtime BigInt helper functions (`*_bigint` functions) - kept for now
3. Internal GMP-based BigInt implementation in runtime

---

## Internal Type Representation

**Important:** `Type::BigInt` may still exist internally for codegen purposes.
The key is that users only see and use `int`.

```
User level:     int
                ↓
Internal AST:   Type::Int (tagged) or Type::BigInt (GMP) based on value/usage
                ↓
Codegen:        LLVM pointer type (for both Int and BigInt internally)
```

---

## Testing Strategy

1. **Type annotation test:**
   ```python
   a: int = 123456789012345678901234567890
   ```

2. **Large literal auto-promotion:**
   ```python
   x = 123456789012345678901234567890  # Should work without annotation
   ```

3. **Function return type promotion:**
   ```python
   def f() -> int:
       return 999999999999999999999999999999
   ```

4. **No type annotation inference:**
   ```python
   def f(n):
       return n * 2  # Should work with large integers
   ```

5. **Arithmetic operations:**
   ```python
   a = 123456789012345678901234567890
   b = 987654321098765432109876543210
   c = a + b  # Should work
   ```

6. **Edge cases:**
   - Zero initialization: `x: int = 0`
   - Negative values: `y: int = -123`
   - Mixed operations: `int` with `i64`, `i32`, etc.

---

## Success Criteria

- [ ] No `bigint`/`BigInt` type in user-facing type system
- [ ] No `n` suffix support
- [ ] No `BigInt()` constructor function
- [ ] `int` type supports arbitrary precision
- [ ] Large literals auto-promote to BigInt internally
- [ ] Type annotations with `int` work for large values
- [ ] Function return types drive promotion
- [ ] `Type::Int` ↔ `Type::BigInt` internal compatibility works
- [ ] All tests pass with new syntax
- [ ] Python-like behavior achieved

---

## Risks and Considerations

1. **Breaking change**: All existing code using `n` suffix, `BigInt()`, or `bigint` type will break
2. **Test updates**: Many test files need updating (50+ `.vp` files)
3. **User confusion**: Need clear documentation on when `int` becomes arbitrary precision
4. **Performance**: Ensure auto-promotion doesn't introduce overhead for small integers
5. **Internal consistency**: Ensure all type system components handle `Type::Int` correctly

---

## Timeline Estimate

- **Phase 1 (Remove bigint type from parser)**: 1 hour
- **Phase 2 (Remove n suffix)**: 1 hour
- **Phase 3 (Remove BigInt constructor)**: 1 hour
- **Phase 4 (Type checker updates)**: 2 hours
- **Phase 5 (Update type references)**: 2 hours
- **Phase 6 (Update tests)**: 4-6 hours
- **Phase 7 (Verification)**: 2 hours

**Total**: 13-15 hours

---

## Appendix: Current Type Checker Compatibility Rules

The existing `is_compatible()` function already supports:
- Integer literal → BigInt promotion
- BigInt → integer (may truncate)
- Int ↔ i64 compatibility

These rules need to be extended to ensure `Type::Int` behaves identically to `Type::BigInt` for user-facing purposes.
