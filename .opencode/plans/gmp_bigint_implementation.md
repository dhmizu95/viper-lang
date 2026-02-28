# Implementing GMP-Based Big Integer in Viper

This document outlines the step-by-step implementation of GMP-based big integers with Python-like syntax.

## Design Decisions

- **Library**: `gmp-mpfr-sys = "1.6.8"` (Rust FFI bindings, bundled)
- **Syntax**: Python-like with auto-inference and methods
- **Memory**: ARC integration with Viper's memory management
- **Hybrid types**: Small literals = i64, overflow = BigInt

---

## Features to Implement

| Feature | Syntax | Implementation |
|---------|--------|----------------|
| BigInt literals | `10**50` | Auto-BigInt when overflows i64 |
| Type annotation | `x: bigint` | Explicit BigInt type |
| Constructor | `big(123)` | Explicit conversion |
| Addition | `+` | BigInt |
| Subtraction | `-` | BigInt |
| Multiplication | `*` | BigInt |
| True division | `/` | **Returns float** (not BigInt!) |
| Floor division | `//` | BigInt |
| Modulo | `%` | BigInt |
| Power operator | `a ** b` | BigInt |
| Power function | `pow(base, exp)` | BigInt |
| Modular pow | `pow(base, exp, mod)` | Efficient crypto |
| Bitwise AND | `&` | BigInt |
| Bitwise OR | `\|` | BigInt |
| Bitwise XOR | `^` | BigInt |
| Bitwise NOT | `~` | BigInt (new!) |
| Left shift | `<<` | BigInt |
| Right shift | `>>` | BigInt |
| Method: bit_length | `.bit_length()` | Returns u64 |
| Method: to_bytes | `.to_bytes(len, order)` | Returns bytes |
| Method: from_bytes | `BigInt.from_bytes(...)` | Class method |
| Mixed ops | `bigint + i64` | Auto-promote i64 → BigInt |

---

## Step 1: Add BigInt Type to AST

**File:** `src/ast/types.rs`

```rust
BigInt,

pub fn is_bigint(&self) -> bool {
    matches!(self, Type::BigInt)
}

pub fn is_integer(&self) -> bool {
    matches!(self, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::BigInt)
}
```

**File:** `src/utils/mangling.rs` - Add BigInt to type mangling

---

## Step 2: Parser

### 2a. Type Keyword

**File:** `src/parser/statements/definitions.rs`

```rust
"bigint" => Some(Type::BigInt),
```

### 2b. Integer Literal Overflow

**File:** `src/parser/expressions/literals.rs`

```rust
// Detect overflow: 10**50 overflows i64 → BigInt
Expr::Pow(base, exp, span) => {
    // Evaluate at parse time if possible
    // If overflows i64, emit as BigIntLiteral
}
```

**New AST node:** `src/ast/nodes.rs`
```rust
BigIntLiteral(String, Span),  // "12345678901234567890"
```

### 2c. Operators

Add to BinOp enum:
- `FloorDiv` for `//`
- `Pow` for `**`
- `Not` for `~` (check if exists)

---

## Step 3: Type Checker

### 3a. Literal Inference

**File:** `src/semantic/type_checker/infer.rs`

```rust
Expr::BigIntLiteral(_, _) => Some(Type::BigInt),
```

### 3b. Binary Operations

**File:** `src/semantic/type_checker/exprs.rs`

```rust
// Addition, Subtraction, Multiplication → BigInt
BinOp::Add | BinOp::Sub | BinOp::Mul => {
    if left.is_bigint() || right.is_bigint() {
        return Ok(Type::BigInt);
    }
}

// True division → Always float (even for BigInt!)
BinOp::Div => {
    return Ok(Type::F64);
}

// Floor division → BigInt
BinOp::FloorDiv => {
    return Ok(Type::BigInt);
}

// Modulo → BigInt
BinOp::Mod => {
    if left.is_bigint() || right.is_bigint() {
        return Ok(Type::BigInt);
    }
}

// Power → BigInt
BinOp::Pow => {
    return Ok(Type::BigInt);
}

// Bitwise → BigInt
BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Shl | BinOp::Shr => {
    if left.is_bigint() || right.is_bigint() {
        return Ok(Type::BigInt);
    }
}

// Bitwise NOT → BigInt (unary)
UnaryOp::Not (or new BitNot) => {
    if operand.is_bigint() {
        return Ok(Type::BigInt);
    }
}
```

---

## Step 4: Codegen Types

**File:** `src/codegen/types.rs`

```rust
Type::BigInt => self.context.ptr_type(AddressSpace::default()).into(),
```

---

## Step 5: JIT Stubs with ARC

**File:** `src/jit_stubs/bigint.rs`

```rust
use gmp_mpfr_sys::gmp::{self, mpz_t};

#[repr(C)]
pub struct ViperBigInt {
    pub mpz: mpz_t,
}

// === CONSTRUCTORS ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_from_i64(val: i64) -> *mut ViperBigInt { ... }

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_from_str(ptr: *const i8) -> *mut ViperBigInt { ... }

// === ARITHMETIC ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_add(a, b) -> *mut ViperBigInt { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_sub(a, b) -> *mut ViperBigInt { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_mul(a, b) -> *mut ViperBigInt { ... }

// TRUE DIVISION → returns float!
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_div(a, b) -> f64 {
    // gmp::mpz_get_d - converts to double
    gmp::mpz_get_d(&(*a).mpz) / gmp::mpz_get_d(&(*b).mpz)
}

// FLOOR DIVISION → BigInt
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_floor_div(a, b) -> *mut ViperBigInt { ... }

// MODULO → BigInt
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_mod(a, b) -> *mut ViperBigInt { ... }

// === POWER ===

// a ** b
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_pow(base: *mut ViperBigInt, exp: u64) -> *mut ViperBigInt {
    let result = allocate_bigint();
    gmp::mpz_pow_ui(&mut (*result).mpz, &(*base).mpz, exp);
    result
}

// pow(a, b, mod) - modular exponentiation
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_pow_mod(base, exp, mod_) -> *mut ViperBigInt {
    let result = allocate_bigint();
    gmp::mpz_powm(&mut (*result).mpz, &(*base).mpz, &(*exp).mpz, &(*mod_).mpz);
    result
}

// === BITWISE ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_and(a, b) -> *mut ViperBigInt { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_or(a, b) -> *mut ViperBigInt { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_xor(a, b) -> *mut ViperBigInt { ... }

// Bitwise NOT: ~a
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_not(a: *mut ViperBigInt) -> *mut ViperBigInt {
    let result = allocate_bigint();
    gmp::mpz_com(&mut (*result).mpz, &(*a).mpz);  // bitwise complement
    result
}

// Left shift: a << n
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_shl(a: *mut ViperBigInt, shift: u64) -> *mut ViperBigInt {
    let result = allocate_bigint();
    gmp::mpz_mul_2exp(&mut (*result).mpz, &(*a).mpz, shift);
    result
}

// Right shift: a >> n
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_shr(a: *mut ViperBigInt, shift: u64) -> *mut ViperBigInt {
    let result = allocate_bigint();
    gmp::mpz_fdiv_q_2exp(&mut (*result).mpz, &(*a).mpz, shift);
    result
}

// === COMPARISON ===
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_cmp(a, b) -> i32 { ... }

// === METHODS ===

// .bit_length()
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_bit_length(a: *mut ViperBigInt) -> u64 {
    gmp::mpz_sizeinbase(&(*a).mpz, 2)
}

// .to_bytes(length, byteorder) -> (bytes, sign)
#[repr(C)]
pub struct ViperBytes {
    pub data: *mut u8,
    pub len: usize,
    pub sign: i32,
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_to_bytes(a, length: u64, byteorder: i32) -> ViperBytes {
    // Convert to bytes
}

// BigInt.from_bytes(bytes, byteorder) - class method
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_from_bytes(data: *const u8, len: usize, byteorder: i32) -> *mut ViperBigInt {
    // Convert from bytes
}

// === CONVERSION ===
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_to_i64(a) -> i64 { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_to_f64(a) -> f64 { ... }

// === ARC ===
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_destructor(ptr) { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_retain(a) { ... }
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_release(a) { ... }
```

---

## Step 6: Register JIT Stubs

**File:** `src/jit_stubs/registry.rs`

Register all functions including:
- `vp_bigint_div` → returns f64 (true division!)
- `vp_bigint_floor_div` → returns BigInt
- `vp_bigint_pow` → standard power
- `vp_bigint_pow_mod` → modular exponentiation
- `vp_bigint_not` → bitwise NOT
- `vp_bigint_bit_length`
- `vp_bigint_to_bytes`
- `vp_bigint_from_bytes`

---

## Step 7: Runtime Codegen

**File:** `src/codegen/runtime/bigint.rs`

Declare all functions in LLVM module.

---

## Step 8: Operator Codegen

**File:** `src/codegen/expressions/operators/mod.rs`

```rust
match op {
    BinOp::Add => "vp_bigint_add",
    BinOp::Sub => "vp_bigint_sub",
    BinOp::Mul => "vp_bigint_mul",
    BinOp::Div => "vp_bigint_div",        // Returns float!
    BinOp::FloorDiv => "vp_bigint_floor_div",  // Returns BigInt
    BinOp::Mod => "vp_bigint_mod",
    BinOp::Pow => "vp_bigint_pow",
    BinOp::And => "vp_bigint_and",
    BinOp::Or => "vp_bigint_or",
    BinOp::Xor => "vp_bigint_xor",
    BinOp::Shl => "vp_bigint_shl",
    BinOp::Shr => "vp_bigint_shr",
}
```

**Handle unary NOT for bitwise NOT:**
```rust
UnaryOp::BitNot => "vp_bigint_not",
```

---

## Step 9: Method Calls

**File:** Handle `.method()` in codegen

```rust
Expr::MethodCall { object, method, args } => {
    match method.as_str() {
        "bit_length" => "vp_bigint_bit_length",
        "to_bytes" => "vp_bigint_to_bytes",
        "from_bytes" => "vp_bigint_from_bytes",  // class method
    }
}
```

---

## Step 10: pow() Built-in

```viper
pow(2, 1000)       # a ** b
pow(a, b, mod)      # modular exponentiation
```

---

## Step 11: State & Build

Track BigInt variables, add `gmp-mpfr-sys` to Cargo.toml.

---

## Step 12: Test

```viper
def demonstrate():
    a = 10**50 + 123456789
    b = 10**40 + 987654321
    
    # Arithmetic
    print(a + b)
    print(a - b)
    print(a * b)
    print(a / b)          # Float! 2.0
    print(a // b)         # BigInt (floor)
    print(a % b)
    
    # Power
    print(pow(2, 1000))
    print(pow(a, 12345, 10**10))
    
    # Bitwise
    print(a & b)
    print(a | b)
    print(a ^ b)
    print(~a)             # Bitwise NOT
    print(a << 10)
    print(a >> 10)
    
    # Methods
    print(a.bit_length())
    print(a.to_bytes(100, 'big'))
```

---

## Implementation Order

1. AST Type
2. Parser (`//`, `**`, overflow literals)
3. Type Checker (true div → float, floor div → BigInt)
4. Codegen Types
5. JIT Stubs (all ops + ARC)
6. Register Stubs
7. Runtime Codegen
8. Operator Codegen
9. Method Calls
10. pow() built-in
11. State & Build
12. Test
