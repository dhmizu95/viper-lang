# BigInt: Arbitrary Precision Integers in Viper

## Background

The user pasted Python BigInt/float educational content as context. Currently in Viper:
- Integers are `i64` in AST (`Expr::Int(i64, Span)`) and LLVM codegen (`i64_const`)
- The lexer uses `i128` internally but the AST truncates to `i64`, silently overflowing
- Large literals beyond `i128::MAX` return a lexer error
- No arbitrary precision support exists

The goal is to implement **Python-style arbitrary precision integers** in Viper so that
integer arithmetic never overflows. This is a core correctness feature.

> [!IMPORTANT]
> **BigInt changes affect the entire integer pipeline**: lexer → AST → parser → codegen → runtime.
> The approach below keeps normal `i64` for small integers and introduces `BigInt` only when needed
> (a literal that overflows `i64`, or arithmetic result that overflowed at compile time).

## Proposed Changes

---

### Runtime C Library

#### [NEW] [bigint.h](file:///home/stl/viper-lang/runtime/include/bigint.h)
Declares `VpBigInt` struct and all runtime BigInt functions. Uses a sign-magnitude array of
`uint32_t` limbs (base 2³²), similar to CPython's implementation.

```c
typedef struct { int sign; size_t len; size_t cap; uint32_t* digits; } VpBigInt;
VpBigInt* vp_bigint_from_i64(int64_t v);
VpBigInt* vp_bigint_from_str(const char* s);
VpBigInt* vp_bigint_add(VpBigInt* a, VpBigInt* b);
VpBigInt* vp_bigint_sub(VpBigInt* a, VpBigInt* b);
VpBigInt* vp_bigint_mul(VpBigInt* a, VpBigInt* b);
VpBigInt* vp_bigint_div(VpBigInt* a, VpBigInt* b);
VpBigInt* vp_bigint_mod(VpBigInt* a, VpBigInt* b);
VpBigInt* vp_bigint_pow(VpBigInt* base, VpBigInt* exp);
int       vp_bigint_cmp(VpBigInt* a, VpBigInt* b);
char*     vp_bigint_to_str(VpBigInt* a);
void      vp_bigint_free(VpBigInt* a);
```

#### [NEW] [bigint.c](file:///home/stl/viper-lang/runtime/src/bigint.c)
Implementation of all BigInt operations. Algorithms:
- **Addition/Subtraction**: Grade-school with carry/borrow across limbs
- **Multiplication**: Grade-school O(n²) — sufficient for now, Karatsuba later
- **Division**: Knuth D algorithm
- **Power**: Binary exponentiation

#### [MODIFY] [Makefile](file:///home/stl/viper-lang/runtime/Makefile)
Add `bigint.c` to the SOURCES list so it gets compiled into the runtime library.

---

### Lexer

#### [MODIFY] [tokens.rs](file:///home/stl/viper-lang/src/lexer/tokens.rs)
Add `BigInt(String)` variant before `Float`:
```diff
+    BigInt(String),  // Arbitrary precision integer literal (as decimal string)
     Float(f64),
```

#### [MODIFY] [scanner.rs](file:///home/stl/viper-lang/src/lexer/scanner.rs)
In [read_number()](file:///home/stl/viper-lang/src/lexer/scanner.rs#510-634), after reading all decimal digits and determining it's an integer:
```diff
-   Ok(TokenKind::Int(s.parse::<i128>().map_err(|_| {
-       format!("Integer literal too large: {}", s)
-   })?))
+   // Try i64 first; fall back to BigInt for large literals
+   if let Ok(v) = s.parse::<i64>() {
+       Ok(TokenKind::Int(v as i128))
+   } else {
+       Ok(TokenKind::BigInt(s))
+   }
```
Do the same for hex/binary/octal literals — if they don't fit in i64, emit `BigInt`.

---

### AST

#### [MODIFY] [ast/types.rs](file:///home/stl/viper-lang/src/ast/types.rs)
Add `BigInt` type after `I64`:
```diff
+    /// Arbitrary precision integer
+    BigInt,
```
Update [is_integer()](file:///home/stl/viper-lang/src/ast/types.rs#60-63), [is_numeric()](file:///home/stl/viper-lang/src/ast/types.rs#53-59), `Display` impl accordingly.

#### [MODIFY] [ast/nodes.rs](file:///home/stl/viper-lang/src/ast/nodes.rs)
Add `BigInt` expression node:
```diff
+    /// BigInt literal (arbitrary precision)
+    BigInt(String, Span),
```
Update the [span()](file:///home/stl/viper-lang/src/ast/nodes.rs#406-439) method match arm.

---

### Parser

#### [MODIFY] Parser (whichever file handles primary expressions)
Map `TokenKind::BigInt(s)` → `Expr::BigInt(s, span)`.

---

### Code Generation

#### [MODIFY] [codegen/runtime.rs](file:///home/stl/viper-lang/src/codegen/runtime.rs)
Declare all BigInt runtime functions as LLVM external functions (returning `i8*`/ptr).

#### [MODIFY] [codegen/types.rs](file:///home/stl/viper-lang/src/codegen/types.rs)
- Add `BigInt` to `VarType` enum
- Map `Type::BigInt` → pointer type in [llvm_type()](file:///home/stl/viper-lang/src/codegen/types.rs#44-67) and [llvm_return_type()](file:///home/stl/viper-lang/src/codegen/types.rs#68-89)

#### [MODIFY] [codegen/expressions/core.rs](file:///home/stl/viper-lang/src/codegen/expressions/core.rs)
- [infer_expr_type](file:///home/stl/viper-lang/src/codegen/expressions/core.rs#10-65): `Expr::BigInt` → `Type::BigInt`
- [generate_expr](file:///home/stl/viper-lang/src/codegen/expressions/core.rs#66-221): `Expr::BigInt(s, _)` → call `vp_bigint_from_str(s_ptr)`
- Handle `Expr::Int` for large values: detect overflow at codegen time → emit BigInt

#### [MODIFY] [codegen/expressions/operators.rs](file:///home/stl/viper-lang/src/codegen/expressions/operators.rs)
In `generate_binop`, when either operand is `BigInt`, route through runtime calls:
- `+` → `vp_bigint_add`, `-` → `vp_bigint_sub`, `*` → `vp_bigint_mul`, etc.
- Comparison via `vp_bigint_cmp`

#### [MODIFY] [codegen/expressions/builtins.rs](file:///home/stl/viper-lang/src/codegen/expressions/builtins.rs)
When `print()` receives a BigInt value, call `vp_bigint_to_str` first then print the string.

---

## Verification Plan

### Automated Tests

**Build command:**
```bash
cd /home/stl/viper-lang && cargo build 2>&1
```

**Runtime build:**
```bash
cd /home/stl/viper-lang/runtime && make 2>&1
```

**Run existing test suite (regression check):**
```bash
cd /home/stl/viper-lang && bash run_tests.sh
```
All previously-passing tests must continue to pass.

**New test:** `tests/test_bigint.vp`
```python
# Test: BigInt arithmetic
a = 9999999999999999999999999999999  # > i64::MAX literal
print(a)

# Large arithmetic
b = 999999999999999999 * 999999999999999999
print(b)

# Factorial using BigInt (30! overflows i64)
def factorial(n: i64) -> BigInt:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

result = factorial(30)
print(result)
```

Run it with:
```bash
cd /home/stl/viper-lang
./target/debug/viper build tests/test_bigint.vp && ./test_bigint_vp_bin
```

Expected output (first two lines):
```
9999999999999999999999999999999
999999999999999999000000000000000001
```

**Float overflow test:** `tests/test_float_overflow.vp`
```python
x = 1.8e308
print(x)  # Should print "inf"
y = -1.8e308
print(y)  # Should print "-inf"
```

---

## Deferred: `decimal` Module

The `decimal` standard library module (arbitrary precision decimals like Python's `Decimal`) is
listed as **Phase 4 Low priority** in [FEATURES_NEED_TO_IMPLEMENTED.md](file:///home/stl/viper-lang/FEATURES_NEED_TO_IMPLEMENTED.md). It will be a follow-up
task built on top of the BigInt infrastructure added here.
