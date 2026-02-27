# BigInt Implementation Guide

## Overview

Viper now supports arbitrary-precision integers (BigInt) using the GNU Multiple Precision Arithmetic Library (GMP). This provides:

- **Unlimited precision**: Integers limited only by available memory
- **High performance**: GMP uses assembly-optimized algorithms for all CPU architectures
- **Complete arithmetic**: All standard operations (+, -, *, /, %, **, bitwise ops)
- **Seamless integration**: Works alongside native i64 types

## Architecture

```
Viper Source Code
       ↓
BigInt Literal: BigInt("123...")
       ↓
AST: Expr::BigInt(String, Span)
       ↓
Type: Type::BigInt
       ↓
LLVM: Pointer to ViperBigInt struct
       ↓
Runtime: GMP mpz_t operations
```

## Type System

### AST Representation

```rust
// In src/ast/nodes.rs
pub enum Expr {
    // ...
    BigInt(String, Span),  // BigInt literal from string
    // ...
}

// In src/ast/types.rs
pub enum Type {
    // ...
    BigInt,  // Arbitrary precision integer type
    // ...
}
```

### LLVM Representation

BigInt values are represented as pointers to `ViperBigInt` structs:

```llvm
%ViperBigInt = type { i64, ptr, i8, [7 x i8], mpz_t }
```

The struct contains:
- ARC header (ref_count, destructor, flags)
- GMP `mpz_t` value (variable-size limb array)

## Runtime Integration

### C Runtime Bridge (`runtime/src/gmp_bridge.c`)

All BigInt operations are implemented in C, calling GMP functions:

```c
// Create BigInt from string
ViperBigInt* vp_bigint_from_str(const char* str);

// Arithmetic operations
void vp_bigint_add(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_sub(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_mul(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_div(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_mod(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

// Bitwise operations
void vp_bigint_and(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_or(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_xor(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_lshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_rshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

// Comparison operations
bool vp_bigint_eq(ViperBigInt* a, ViperBigInt* b);
bool vp_bigint_lt(ViperBigInt* a, ViperBigInt* b);
bool vp_bigint_gt(ViperBigInt* a, ViperBigInt* b);
```

### Memory Management

BigInt values are managed by Viper's ARC (Automatic Reference Counting) system:

1. **Creation**: `vp_bigint_from_str()` returns a new BigInt with ref_count=1
2. **Usage**: ARC retains when copied, releases when out of scope
3. **Destruction**: When ref_count reaches 0, `vp_bigint_destroy()` is called
4. **Cleanup**: GMP's `mpz_clear()` frees limb memory

## Usage

### Python-like Syntax

Viper supports natural, Python-like syntax for BigInt operations:

```python
# Method 1: Explicit 'n' suffix (like JavaScript BigInt)
x = 123456789012345678901234567890n

# Method 2: Automatic promotion (integers too large for i64)
y = 123456789012345678901234567890  # Automatically BigInt

# Method 3: BigInt constructor (explicit)
z = BigInt("123456789012345678901234567890")

# All arithmetic operators work naturally
a = 100000000000000000000n
b = 50000000000000000000n

c = a + b  # Addition
d = a * b  # Multiplication
e = a ** b # Power (use pow_bigint for very large exponents)

# Comparisons work as expected
if a > b:
    print("a is larger")

# Bitwise operations
x = 0xFF00FF00n
y = 0x00FF00FFn
z = x & y  # AND

# Different number bases
hex_big = 0xDEADBEEFCAFEn
bin_big = 0b1111111111111111n
oct_big = 0o777777777777n
```

### Basic Usage (Legacy)

```python
# Create BigInt from string literal
x = BigInt("123456789012345678901234567890")

# Arithmetic operations
a = BigInt("1000000000000000000")
b = BigInt("2000000000000000000")
c = a + b  # Addition
d = a * b  # Multiplication
e = b / a  # Division
f = b % BigInt("3")  # Modulo

# Comparison
if a < b:
    print("a is smaller")

# Bitwise operations
x = BigInt("0xFF00FF00")
y = BigInt("0x00FF00FF")
z = x & y  # AND
```

### Standard Library Functions

```python
# Convert to string
s = str_bigint(x)

# Convert to i64 (may overflow)
n = int_bigint(x)

# Absolute value
abs_x = abs_bigint(x)

# Power and square root
result = pow_bigint(base, exp)
sqrt_x = sqrt_bigint(x)

# Min/Max
min_val = min_bigint(a, b)
max_val = max_bigint(a, b)
```

### Large Number Example

```python
# Calculate 2^1000
def power_of_two():
    two = BigInt("2")
    exp = BigInt("1000")
    result = pow_bigint(two, exp)
    print(str_bigint(result))
    # Output: 107150860718626732094842504906000181056140481170553360744375038837...
```

## Code Generation

### Literal Generation

```rust
// In src/codegen/expressions/core.rs
Expr::BigInt(s, _) => {
    let str_val = state.ir_builder.string_const(state.module, s);
    let create_func = state.module
        .get_function("vp_bigint_from_str")
        .ok_or_else(|| "vp_bigint_from_str not declared")?;
    let result = state.ir_builder.build_call(
        state.builder, 
        create_func, 
        &[str_val.into()], 
        "bigint_create"
    )?;
    Ok(result.into())
}
```

### Binary Operation Generation

```rust
// In src/codegen/expressions/operators/bigint.rs
if lhs_type == Type::BigInt || rhs_type == Type::BigInt {
    return bigint::generate_bigint_binop(state, lhs_val, rhs_val, op);
}

// Example: Addition
BinOp::Add => {
    let result_ptr = state.builder.build_alloca(lhs_ptr.get_type(), "result")?;
    let add_func = state.module.get_function("vp_bigint_add")?;
    state.ir_builder.build_call(
        state.builder,
        add_func,
        &[result_ptr.into(), lhs_ptr.into(), rhs_ptr.into()],
        "bigint_add"
    )?;
    Ok(state.builder.build_load(lhs_ptr.get_type(), result_ptr, "result")?.into())
}
```

## Building

### Prerequisites

Install GMP development libraries:

```bash
# Debian/Ubuntu/WSL
sudo apt install libgmp-dev pkg-config

# macOS
brew install gmp pkg-config

# Fedora/RHEL
sudo dnf install gmp-devel pkg-config

# Arch Linux
sudo pacman -S gmp pkg-config
```

Verify installation:
```bash
pkg-config --libs --cflags gmp
# Should output something like: -lgmp
```

### Build Runtime

```bash
cd runtime
make
```

The Makefile automatically:
- Detects GMP via pkg-config
- Compiles `gmp_bridge.c` with `-lgmp`
- Links into `libviper.a`

### Build Compiler

```bash
cargo build
```

The `build.rs` script:
- Uses pkg-config to find GMP
- Links `-lgmp` to the compiler

## Performance Considerations

### When to Use BigInt

**Use BigInt when:**
- Values exceed i64 range (±9.2 × 10¹⁸)
- Cryptographic calculations
- Financial calculations requiring exact precision
- Mathematical computations with very large numbers

**Use i64 when:**
- Values fit in 64 bits
- Performance is critical (loops, counters)
- Interfacing with C APIs

### Performance Comparison

| Operation | i64 (cycles) | BigInt (cycles) | Overhead |
|-----------|--------------|-----------------|----------|
| Add | ~1 | ~50-100 | 50-100× |
| Multiply | ~3 | ~100-500 | 30-150× |
| Divide | ~20-50 | ~200-1000 | 10-20× |
| Compare | ~1 | ~20-50 | 20-50× |

**Note:** BigInt overhead is constant per operation, but enables calculations impossible with fixed-width types.

## Future Enhancements

### Planned Features

1. **Automatic Promotion**: Automatically promote i64 to BigInt on overflow
2. **Tagged Pointers**: Small integer optimization (SIO) for values < 2⁶³
3. **BigInt Literals**: Syntax like `123n` for BigInt literals
4. **Mixed Arithmetic**: Seamless i64 + BigInt operations
5. **GCD/LCM**: Number theory functions
6. **Modular Exponentiation**: `pow_mod(base, exp, mod)` for cryptography

### Optimization Opportunities

1. **Register Caching**: Cache small BigInt values in registers
2. **Limb Inlining**: Store small values directly in struct (no heap allocation)
3. **Batch Operations**: Vectorized operations for arrays of BigInts
4. **JIT Compilation**: Generate optimized code for common patterns

## Testing

### Run Tests

```bash
# Run BigInt test suite
cargo run -- run tests/bigint_test.vp

# Run integration tests
cargo test bigint
```

### Example Test Output

```
=== Viper BigInt Test Suite ===

Test 1: BigInt creation
a = 123456789012345678901234567890
b = 987654321098765432109876543210

Test 2: BigInt addition
a + b = 1111111110111111111011111111100

Test 10: Very large numbers
2^1000 = 107150860718626732094842504906000181056140481170553360744375038837...

=== All tests completed ===
```

## Troubleshooting

### "libgmp not found"

```bash
# Install GMP
sudo apt install libgmp-dev

# Verify installation
pkg-config --libs gmp
# Should output: -lgmp
```

### "vp_bigint_from_str not declared"

Ensure runtime library is built and linked:

```bash
cd runtime && make
cargo build
```

### Memory Leaks

BigInt values should be automatically freed by ARC. To verify:

```bash
valgrind --leak-check=full ./program_vp
```

## References

- [GMP Documentation](https://gmplib.org/manual/)
- [GMP Implementation Guide](runtime/GMP_IMPLEMENTATION.md)
- [Viper Runtime](runtime/src/gmp_bridge.c)
- [Viper Codegen](src/codegen/expressions/operators/bigint.rs)
