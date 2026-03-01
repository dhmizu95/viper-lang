# Viper Language Compiler - Project Context

## Project Overview

**Viper** is an LLVM-based compiler for a Python-like programming language that compiles to native binaries with C-level performance. The compiler is written in Rust and uses a C runtime library for memory management and I/O operations.

**Version:** 0.3.7  
**License:** MIT  
**Architecture:** Frontend (Rust) → LLVM IR → Native Binary (via GCC)

### Key Design Goals
- Python-like syntax with static typing
- AOT compilation to native code + JIT execution support
- ARC (Automatic Reference Counting) memory management
- M:N threading with work-stealing scheduler (planned)
- Zero-cost abstractions where possible

---

## Building and Running

### Prerequisites
- Rust 1.70+
- LLVM 20.x (at `/usr/lib/llvm-20`)
- GCC (for linking AOT binaries)

### Build Commands

```bash
# Debug build
cargo build

# Release build (with LTO)
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test <test_name>

# Run integration tests
./run_tests.sh

# Lint and format
cargo clippy
cargo fmt
```

### Running Viper Programs

```bash
# JIT compile and run
cargo run -- run src/main.vp
cargo run -- run program.vp -O 2

# AOT compile to binary
cargo run -- build program.vp -o program
cargo run -- build program.vp -O 2 -o program

# With LTO
cargo run -- build program.vp -O 2 --lto -o program

# Emit LLVM IR
cargo run -- build program.vp --emit-llvm

# PGO instrumentation
cargo run -- build program.vp --pgo=instrument -o program
# Run binary to collect profiles
./program_vp_bin
# Rebuild with profiles
cargo run -- build program.vp --pgo=use -o program

# Initialize new project
cargo run -- init myproject

# Show compiler info
cargo run -- info
```

### Optimization Levels

| Level | Flag | Description |
|-------|------|-------------|
| O0 | `-O 0` | No optimization (debug) |
| O1 | `-O 1` | Basic optimization |
| O2 | `-O 2` | Standard optimization (default) |
| O3 | `-O 3` | Aggressive optimization |

---

## Project Structure

```
viper-lang/
├── Cargo.toml              # Rust project config
├── build.rs                # LLVM linking config
├── install.sh              # Installation script
├── run_tests.sh            # Integration test runner
│
├── src/                    # Compiler source (Rust)
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library exports
│   ├── cli/                # Command-line interface
│   │   ├── mod.rs
│   │   ├── args.rs         # clap definitions
│   │   ├── bench.rs        # Benchmarking
│   │   ├── fmt.rs          # Code formatting
│   │   ├── lint.rs         # Static analysis
│   │   ├── repl.rs         # Interactive REPL
│   │   └── doc.rs          # Documentation gen
│   ├── lexer/              # Lexical analysis
│   │   ├── mod.rs
│   │   ├── scanner.rs      # Token scanning
│   │   ├── tokens.rs       # Token definitions
│   │   └── indent_stack.rs # Indentation tracking
│   ├── parser/             # Syntax analysis
│   │   ├── mod.rs
│   │   ├── recursive_descent.rs
│   │   └── expressions.rs  # Pratt parser
│   ├── ast/                # AST definitions
│   │   ├── mod.rs
│   │   ├── nodes.rs        # Expr, Stmt enums
│   │   └── types.rs        # Type definitions
│   ├── semantic/           # Semantic analysis
│   │   ├── mod.rs
│   │   ├── symbol_table.rs
│   │   ├── type_checker.rs
│   │   ├── escape_analysis.rs  # Stack vs heap
│   │   └── ownership.rs    # ARC analysis
│   ├── codegen/            # LLVM IR generation
│   │   ├── mod.rs
│   │   ├── context.rs      # LLVM context
│   │   ├── builder.rs      # IR building
│   │   ├── dce.rs          # Dead code elimination
│   │   ├── types.rs        # Type mapping
│   │   └── runtime.rs      # Runtime bindings
│   ├── lsp/                # Language server (planned)
│   └── utils/              # Utilities
│       ├── span.rs         # Source locations
│       └── mangling.rs     # Name mangling
│
├── runtime/                # C runtime library
│   ├── Makefile
│   ├── viper_stdlib.h      # Public API
│   ├── include/            # Headers
│   ├── src/                # Implementation
│   └── obj/                # Compiled objects
│
├── std/                    # Viper standard library
│   └── prelude.vp          # Auto-imported builtins
│
├── tests/                  # Test suite
│   ├── *.vp                # Viper test programs
│   └── integration/        # Rust integration tests
│
├── benchmark/              # Performance benchmarks
├── examples/               # Example programs
└── docs/                   # Documentation
```

---

## Language Features

### Core Types
- `i64` - 64-bit signed integers
- `f64` - 64-bit floating point
- `bool` - Boolean (`True`/`False`)
- `str` - Strings
- `list[T]` - Dynamic arrays

### Syntax Example

```python
# Variable declaration (type inferred)
x = 42
y = 3.14
name = "Viper"

# Function with type annotations
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# Lists
nums = [1, 2, 3]
nums.append(4)
first = nums[0]

# Control flow
if x > 0:
    print("positive")
elif x < 0:
    print("negative")
else:
    print("zero")

while i < 10:
    i = i + 1

for item in items:
    print(item)

# Pattern matching
match value:
    case 0:
        print("zero")
    case 1:
        print("one")
    case _:
        print("other")

# Pipeline operator
result = data |> transform |> filter |> process
result = data >> transform >> filter >> process
```

### Built-in Functions
- `print()` - Output to stdout
- `len()` - Length of container
- `range()` - Integer sequence
- `str()`, `int()`, `float()`, `bool()` - Type conversion
- Math: `sqrt()`, `abs()`, `ln()`, `floor()`

### Memory Management
- **ARC (Automatic Reference Counting)** - Deterministic cleanup
- **Escape Analysis** - Stack allocation for non-escaping variables
- **Optimizations:**
  - Register allocation for non-escaping variables
  - Skipping ARC retain/release for stack variables
  - Dead code elimination

---

## Compiler Architecture

### Pipeline

```
Source (.vp)
    ↓
Lexer → Tokens (with Indent/Dedent)
    ↓
Parser → AST (recursive descent + Pratt)
    ↓
Semantic Analysis (type checking, escape analysis)
    ↓
CodeGen → LLVM IR
    ↓
DCE Optimization
    ↓
LLVM opt (external) → Optimized IR
    ↓
LLVM llc → Object file
    ↓
GCC → Binary (with libviper.a)
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `lexer` | Python-style indentation, tokenization |
| `parser` | Recursive descent with Pratt parsing |
| `semantic` | Type inference, escape analysis, symbol tables |
| `codegen` | LLVM IR generation via Inkwell |
| `dce` | Dead code elimination optimization |

---

## Development Conventions

### Code Style

```rust
// Imports: stdlib, external, internal
use std::collections::HashMap;
use inkwell::values::BasicValueEnum;
use crate::ast::{Expr, Stmt};

// Public interfaces have explicit types
pub fn generate_stmt<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String>

// Use Result<T, String> for codegen errors
// Use ? operator for propagation
let value = some_operation()?;

// Use .expect() for operations that should never fail
let func = module.get_function("main").expect("main not found");

// LLVM operations often need unsafe
let elem_ptr = unsafe {
    builder.build_in_bounds_gep(...)
}?;
```

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_behavior() {
        // Test implementation
    }
}
```

### Error Messages
- Include file, line, column
- Provide helpful suggestions

---

## Runtime Library

The C runtime (`libviper.a`) provides:

### I/O Functions
- `vp_print_i64(i64)`
- `vp_print_f64(f64)`
- `vp_print_str(const char*)`
- `vp_print_bool(bool)`

### List Operations
- `vp_list_create()`, `vp_list_free()`
- `vp_list_append()`, `vp_list_get()`, `vp_list_set()`
- `vp_list_insert()`, `vp_list_remove()`, `vp_list_pop()`

### Memory Management
- `vp_retain(void*)` - Increment ref count
- `vp_release(void*, destructor)` - Decrement ref count

### Math Functions
- `vp_math_sqrt(f64)`, `vp_math_abs(f64)`
- `vp_math_ln(f64)`, `vp_math_floor(f64)`

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `viper build <file>` | AOT compile to binary |
| `viper run <file>` | JIT compile and execute |
| `viper init [name]` | Create new project |
| `viper info` | Show compiler info |
| `viper bench <file>` | Run benchmarks |
| `viper fmt <file>` | Format code |
| `viper lint <file>` | Static analysis |
| `viper repl` | Interactive shell |
| `viper doc` | Generate documentation |

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `README.md` | User documentation |
| `PROJECT_OVERVIEW.md` | Architecture and roadmap |
| `PROJECT_STRUCTURE.md` | Detailed directory structure |
| `CORE_LANGUAGE_FEATURES.md` | Complete feature list |
| `OPTIMIZATIONS.md` | Optimization implementation |
| `UNIMPLEMENTED_FEATURES.md` | Feature backlog |
| `AGENTS.md` | Development guidelines |

---

## Common Tasks

### Adding a New Language Feature

1. **Lexer** (`src/lexer/`): Add token type and scanning logic
2. **Parser** (`src/parser/`): Add parsing to AST node
3. **AST** (`src/ast/`): Define new node type
4. **Semantic** (`src/semantic/`): Add type checking rules
5. **CodeGen** (`src/codegen/`): Generate LLVM IR

### Debugging

```bash
# Enable debug output in code
eprintln!("Debug: value = {:?}", value);

# Emit LLVM IR for inspection
cargo run -- build program.vp --emit-llvm
# View in program.ll

# Run with JIT for faster iteration
cargo run -- run program.vp
```

### Performance Profiling

```bash
# Build with PGO instrumentation
cargo run -- build program.vp --pgo=instrument -o program

# Run with representative workload
./program_vp_bin

# Rebuild using profiles
cargo run -- build program.vp --pgo=use -o program
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `inkwell` | LLVM bindings (LLVM 20) |
| `clap` | CLI argument parsing |
| `thiserror` | Error handling |
| `which` | Find executables in PATH |
| `cc` | Build-time C compilation |

---

## Troubleshooting

### "Runtime object files not found"
```bash
cd runtime && make
```

### "LLVM not found"
Ensure LLVM 20 is installed and in PATH:
```bash
export PATH="/usr/lib/llvm-20/bin:$PATH"
```

### Linking errors
```bash
# Rebuild runtime
cd runtime && make clean && make

# Reinstall
./install.sh
```

### Memory leaks
Run with Valgrind:
```bash
valgrind --leak-check=full ./program_vp_bin
```
