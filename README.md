# Viper Compiler Documentation

## Overview

Viper is an LLVM-based compiler for the Viper programming language, providing both Ahead-of-Time (AOT) compilation to native binaries and Just-in-Time (JIT) execution for rapid development.

**Version:** 0.2.3  
**License:** MIT  
**Repository:** github.com/viper-lang/viper

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/viper-lang/viper.git
cd viper

# Build release binary
cargo build --release

# Install to ~/.cargo/bin
cargo install --path . --force
```

### Requirements

- Rust 1.70+
- LLVM 20.x
- GCC (for linking AOT binaries)
- Viper Runtime Library (included in `runtime/`)

## Quick Start

```bash
# Create a new project
viper init myproject
cd myproject

# Run the default program
viper run src/main.vp

# Build optimized binary
viper build src/main.vp -O 2 -o myapp
./myapp
```

## CLI Commands

### `viper build` - AOT Compilation

Compile Viper source to native object code.

```bash
viper build <FILE> [OPTIONS]

Arguments:
  <FILE>              Source file to compile (.vp)

Options:
  -o, --output <OUT>  Output file name
  -O, --optimize <LVL> Optimization level (0-3) [default: 2]
  -h, --help          Print help
```

**Examples:**

```bash
# Basic compilation
viper build program.vp

# With optimization
viper build program.vp -O 2

# Custom output name
viper build program.vp -o myapp

# Link and run
viper build program.vp -o program
gcc program.o runtime/libviper.a -o program -lm
./program
```

### `viper run` - JIT Execution

Compile and execute Viper source using LLVM JIT.

```bash
viper run <FILE> [OPTIONS]

Arguments:
  <FILE>              Source file to execute (.vp)

Options:
  -O, --optimize <LVL> Optimization level (0-3) [default: 2]
  -h, --help          Print help
```

**Examples:**

```bash
# Quick execution (no optimization)
viper run program.vp

# With optimizations
viper run program.vp -O 3
```

### `viper init` - Project Initialization

Create a new Viper project structure.

```bash
viper init [NAME]

Arguments:
  [NAME]              Project name [default: viper_project]
```

**Creates:**
```
myproject/
├── Cargo.toml        # Rust project file (for builds)
└── src/
    └── main.vp       # Main Viper source file
```

### `viper info` - Compiler Information

Display compiler features and usage.

```bash
viper info
```

## Optimization Levels

| Level | Flag | Description | Use Case |
|-------|------|-------------|----------|
| O0 | `-O 0` | No optimization | Debugging, testing |
| O1 | `-O 1` | Basic optimization | Development builds |
| O2 | `-O 2` | Standard optimization | **Default** - Best performance/speed tradeoff |
| O3 | `-O 3` | Aggressive optimization | Performance-critical production builds |

**Default:** `-O 2` provides the best balance of compilation speed and runtime performance.

## Language Features

### Core Features

- **Static typing** with type inference
- **Python-like syntax** for readability
- **AOT compilation** to native code
- **JIT execution** for rapid iteration

### Data Types

- `i64` - 64-bit signed integers
- `f64` - 64-bit floating point
- `bool` - Boolean values (`True`/`False`)
- `str` - Strings
- `list` - Dynamic arrays
- `None` - Null value

### Control Flow

```python
# Conditionals
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

# Loops
while i < 10:
    i = i + 1

for item in items:
    print(item)
```

### Functions

```python
def greet(name: str) -> str:
    return "Hello, " + name

def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)
```

### Math Builtins

```python
sqrt(16.0)     # Square root → 4.0
abs(-5.5)      # Absolute value → 5.5
ln(2.718)      # Natural log → ~1.0
floor(3.14)    # Floor → 3.0
```

### Lists

```python
# Create
numbers = [1, 2, 3, 4, 5]

# Access
first = numbers[0]

# Modify
numbers[0] = 10

# Methods
numbers.append(6)
numbers.insert(0, 0)
numbers.remove(2)
last = numbers.pop()
numbers.clear()

# Length
count = len(numbers)
```

### Global Constants

Module-level literal assignments are immutable constants:

```python
PI = 3.14159
MAX_SIZE = 1000
NAME = "Viper"
ENABLED = True
```

### Comments

```python
# This is a comment
x = 42  # Inline comment
```

## Runtime Library

Viper programs link against `libviper.a` which provides:

- **I/O functions**: `vp_print_i64`, `vp_print_f64`, `vp_print_str`, `vp_print_bool`
- **List operations**: `vp_list_create`, `vp_list_append`, `vp_list_get`, etc.
- **Memory management**: `vp_retain`, `vp_release` (ARC-based)
- **Math functions**: `vp_math_sqrt`, `vp_math_abs`, `vp_math_ln`, `vp_math_floor`

### Linking AOT Binaries

```bash
# Basic linking
gcc program.o runtime/libviper.a -o program -lm

# With optimization flags
gcc -O2 program.o runtime/libviper.a -o program -lm

# Static linking
gcc -static program.o runtime/libviper.a -o program -lm -lpthread
```

## Examples

### Hello World

```python
# hello.vp
print("Hello, World!")
```

```bash
viper run hello.vp
```

### Prime Sieve

```python
# sieve.vp
def sieve(n: i64) -> i64:
    is_prime = []
    i = 0
    while i <= n:
        is_prime.append(1)
        i = i + 1
    
    is_prime[0] = 0
    is_prime[1] = 0
    
    i = 2
    while i * i <= n:
        if is_prime[i] == 1:
            j = i * i
            while j <= n:
                is_prime[j] = 0
                j = j + i
        i = i + 1
    
    count = 0
    i = 2
    while i <= n:
        if is_prime[i] == 1:
            count = count + 1
        i = i + 1
    
    return count

def main():
    n = 1000000
    print("Primes up to")
    print(n)
    print(":")
    print(sieve(n))
```

```bash
viper build sieve.vp -O 2 -o sieve
gcc sieve.o runtime/libviper.a -o sieve -lm
./sieve
```

### Fibonacci

```python
# fibonacci.vp
def fib(n: i64) -> i64:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    i = 0
    while i < 20:
        print(fib(i))
        i = i + 1
```

## Architecture

### Compiler Pipeline

```
Source (.vp)
    ↓
Lexer → Tokens
    ↓
Parser → AST
    ↓
Semantic Analysis
    ↓
Codegen → LLVM IR
    ↓
Optimizer (optional)
    ↓
Codegen → Object (.o)
    ↓
Linker → Binary
```

### Directory Structure

```
viper-lang/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── cli/             # Command-line interface
│   ├── lexer/           # Lexical analysis
│   ├── parser/          # Parsing to AST
│   ├── semantic/        # Semantic analysis
│   ├── codegen/         # LLVM code generation
│   └── ast/             # AST definitions
├── runtime/
│   ├── src/runtime.c    # Runtime functions
│   ├── include/         # Headers
│   └── libviper.a       # Compiled library
├── benchmark/           # Performance benchmarks
└── tests/               # Test files
```

## Performance

### Benchmark: Prime Sieve (10M)

| Language | Time | Relative |
|----------|------|----------|
| Rust | 53ms | 1.0x |
| C | 69ms | 1.3x |
| Go | 80ms | 1.5x |
| Viper AOT | 200ms* | 3.8x |

*Includes runtime overhead

See `benchmark/RESULTS.md` for detailed analysis.

## Troubleshooting

### Compilation Errors

**"Undefined variable"**
```python
# Wrong
print(x)  # x not defined

# Right
x = 42
print(x)
```

**"Binary operators cannot be applied to pointer values"**
```python
# Wrong - can't add lists
a = [1, 2]
b = [3, 4]
c = a + b

# Right - use append
a.append(3)
a.append(4)
```

### Linking Errors

**"cannot find -lviper"**
```bash
# Use full path to library
gcc program.o runtime/libviper.a -o program -lm
```

**"relocation R_X86_64_32"**
```bash
# Add -no-pie flag
gcc -no-pie program.o runtime/libviper.a -o program -lm
```

### Runtime Errors

**"Segmentation fault"**
- Check array bounds
- Ensure lists are initialized before use
- Verify function signatures match declarations

## Development

### Building from Source

```bash
# Debug build (fast)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check without building
cargo check
```

### Running Benchmarks

```bash
cd benchmark

# Build all implementations
gcc -O2 -o sieve_c sieve.c
rustc -O -o sieve_rust sieve.rs
go build -o sieve_go sieve.go

# Build Viper
viper build sieve.vp -O 2
gcc -no-pie sieve.o runtime/libviper.a -o sieve_viper -lm

# Run comparison
./sieve_c
./sieve_rust
./sieve_go
./sieve_viper
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Acknowledgments

- LLVM Project - Compiler infrastructure
- Inkwell - LLVM bindings for Rust
- Viper Runtime - C runtime library

## Support

- Issues: GitHub Issues
- Discussions: GitHub Discussions
- Documentation: `docs/` directory
