# Viper Language - Phase 1 Release

**Version:** 0.1.0  
**Release Date:** February 24, 2026  
**Status:** Alpha MVP

---

## Overview

Phase 1 delivers the **core Viper compiler** - a working, self-hosted language with Python-like syntax that compiles to native code via LLVM. This is the foundation upon which all future features (memory management, concurrency, OOP) will be built.

### Vision

> A compiled programming language with Python-like syntax and C-level performance.

---

## What's Included

### Language Features

#### Types
- `i64` - 64-bit signed integers
- `f64` - 64-bit floating point numbers
- `bool` - Boolean values (`true`, `false`)
- `str` - String literals

#### Statements
```python
# Variable declaration (type inferred)
x = 42
y = 3.14
name = "viper"

# Function definition with type annotations
def add(a: i64, b: i64) -> i64:
    return a + b

# If/elif/else conditionals
if x > 0:
    print("positive")
elif x == 0:
    print("zero")
else:
    print("negative")

# While loops
while x > 0:
    x = x - 1

# For loops with range()
for i in range(10):
    print(i)

# Return statements
def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)
```

#### Expressions
- **Arithmetic:** `+`, `-`, `*`, `/`, `%`, `//` (floor div), `**` (power)
- **Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical:** `and`, `or`, `not`
- **Unary:** `-x`, `+x`, `not x`
- **Function calls:** `print(x)`, `factorial(5)`

### Compiler Architecture

| Layer | Technology | Status |
|-------|------------|--------|
| Frontend | Rust | ✅ Complete |
| Middle-end | LLVM (via Inkwell) | ✅ Complete |
| Backend | Native code (LLVM) | ✅ Complete |
| Output | Native binary | ✅ Complete |

### Implementation Details

#### Lexer
- Python-style indentation handling (emit `Indent`/`Dedent` tokens)
- String literals with escape sequences
- Source location tracking for error reporting

#### Parser
- Recursive descent with Pratt parsing for expressions
- Proper operator precedence (`*` before `+`, etc.)
- AST with `Box<Expr>` for recursive structures

#### Code Generator
- LLVM IR generation via Inkwell crate
- Automatic `main` entry point generation
- Support for top-level statements via `viper_init`

---

## Installation

### Prerequisites

- **Rust** (latest stable)
- **LLVM 20** (or compatible version)
- **GCC/Clang** (for final linking)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/viper-lang/viper.git
cd viper-lang

# Build the compiler
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Verify Installation

```bash
viper help
```

---

## Usage

### Compile a Program

```bash
viper build source.vp -o output
```

This generates:
- `output.bc` - LLVM bitcode
- Link manually:
  ```bash
  llc output.bc -filetype=obj -o output.o
  gcc output.o -o output -L./runtime -lviper
  ```

### Run Directly (JIT)

```bash
viper run source.vp
```

### Example Session

```bash
$ viper run test_factorial.vp
🐍 Viper Compiler 0.1.0
   Running: test_factorial.vp
   [4/4] Executing via JIT...
120
✅ Execution complete.
```

---

## Example Programs

### Hello World (via print)
```python
def main():
    print(42)
```

### Factorial (Recursive)
```python
def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    result = factorial(5)
    print(result)  # Output: 120
```

### Fibonacci (Iterative)
```python
def main():
    a = 0
    b = 1
    i = 0

    while i < 10:
        print(a)
        temp = a + b
        a = b
        b = temp
        i = i + 1
```

### For Loop
```python
def main():
    for i in range(5):
        print(i)  # Output: 0 1 2 3 4
```

---

## Known Limitations

### Phase 1 Scope (Intentional)

The following features are **not** included in Phase 1:

- ❌ **Memory Management** - No ARC, no garbage collection
- ❌ **Data Structures** - No lists, dicts, tuples (coming in Phase 2)
- ❌ **Concurrency** - No `sync`/`task`, no channels (coming in Phase 3)
- ❌ **OOP** - No classes, inheritance (coming in Phase 4)
- ❌ **Error Handling** - No `try`/`except` (coming in Phase 4)
- ❌ **Modules/Imports** - Single-file programs only
- ❌ **Standard Library** - Only `print()` and `range()` builtins
- ❌ **Type Inference** - Basic inference for literals only
- ❌ **Floating Point Operations** - Limited support

### Technical Limitations

- All variables default to `i64` in expressions
- No support for string concatenation or interpolation
- No support for function overloading
- No support for default arguments
- No support for variadic functions

---

## Testing

### Run Test Suite

```bash
# Individual tests
viper run test_simple.vp
viper run test_factorial.vp
viper run test_fibonacci.vp
viper run test_add.vp
viper run test_swap.vp
viper run test_swap_xor.vp

# Run all tests
cargo test
```

### Success Criteria

✅ `factorial(20)` computes correctly  
✅ `fibonacci(10)` generates correct sequence  
✅ All arithmetic operators work as expected  
✅ Control flow (if/while/for) executes correctly  
✅ Function calls (including recursion) work properly  

---

## Project Structure

```
viper-lang/
├── Cargo.toml              # Rust project configuration
├── build.rs                # LLVM linking configuration
├── Makefile                # Build automation
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library exports
│   ├── cli/                # Command-line interface
│   ├── lexer/              # Lexical analysis
│   ├── parser/             # Syntax analysis
│   ├── ast/                # AST definitions
│   ├── codegen/            # LLVM IR generation
│   └── utils/              # Shared utilities
├── runtime/                # C runtime library (future)
├── tests/                  # Integration tests
└── docs/                   # Documentation
```

---

## Roadmap

### Phase 2: Data Structures + Memory Management
- [ ] Automatic Reference Counting (ARC)
- [ ] Dynamic arrays (`List[T]`)
- [ ] String operations
- [ ] Type inference improvements

### Phase 3: Concurrency
- [ ] M:N threading (work-stealing scheduler)
- [ ] `sync`/`task` primitives
- [ ] Channels (`chan`)
- [ ] Wait groups

### Phase 4: OOP + Advanced Features
- [ ] Classes and inheritance
- [ ] Generics
- [ ] Exception handling
- [ ] Decorators

### Phase 5: Ecosystem Tools
- [ ] Package manager (`vpm`)
- [ ] Language server (`viper-lsp`)
- [ ] Code formatter (`viper-fmt`)
- [ ] Documentation generator (`vdoc`)

---

## Contributing

### Reporting Issues

Found a bug? Have a feature request? Please open an issue on GitHub with:
- Clear description
- Minimal reproduction example
- Expected vs actual behavior

### Code Contributions

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

### Coding Standards

- **Rust:** Use `?` for error propagation, avoid `unsafe` except for LLVM
- **Error Messages:** Include file, line, column, and helpful suggestion
- **Tests:** Unit tests in `#[cfg(test)]` modules, integration tests in `tests/`

---

## License

MIT License - See [LICENSE](../LICENSE) for details.

---

## Acknowledgments

- **Inkwell** - LLVM bindings for Rust
- **LLVM** - Compiler infrastructure
- **Python** - Syntax inspiration
- **Rust** - Implementation language

---

**Happy Coding! 🐍**
