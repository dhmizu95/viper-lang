# Viper Language Compiler - Project Overview

## Project Overview

Viper is a Python-inspired compiled language that aims for:
- **Near-100% Python compatibility** at the language level
- **C-like performance** on hot paths through LLVM compilation and runtime optimization

The project consists of:
- **Rust compiler** (`src/`) - Lexer, parser, semantic analyzer, and LLVM codegen
- **C runtime** (`runtime/`) - Native runtime library for lists, dicts, strings, concurrency, etc.
- **Standard library** (`std/`) - Viper modules for math, time, random, etc.
- **Benchmarks** (`benchmarks/`) - Performance test suite
- **Tests** (`tests/`) - Unit and integration tests

### Architecture

```
src/
├── lexer/          # Tokenization and scanning
├── parser/         # Parsing to AST
├── semantic/       # Type checking, escape analysis, closure analysis
├── codegen/        # LLVM IR generation
│   ├── expressions/
│   ├── statements/
│   ├── runtime/
│   └── core/
├── driver/         # JIT and AOT execution drivers
└── module/         # Module loading and imports

runtime/
├── src/            # C runtime implementation
├── include/        # Header files
└── obj/            # Compiled object files
```

## Building and Running

### Prerequisites

- Rust toolchain (edition 2021)
- LLVM 21
- GCC (for AOT linking)
- GMP library (for BigInt support)

### Build Commands

```bash
# Build the compiler
make build
# or: cargo build

# Build the runtime library
make runtime
# or: cd runtime && make

# Fast compile check
make check
# or: cargo check

# Format code
make fmt
# or: cargo fmt

# Run tests
make test
# or: cargo test

# Clean build artifacts
make clean
```

### Running Viper Code

```bash
# JIT mode (default, -O2 optimization)
cargo run --bin viper -- run myfile.vp

# AOT mode (compile to native binary)
cargo run --bin viper -- build myfile.vp
# Output: myfile_vp_bin

# Debug mode (-O0 for easier debugging)
make dev
# or: cargo run --bin viper -- run -O0 myfile.vp

# Run with specific optimization level
cargo run --bin viper -- run -O3 myfile.vp
```

### Performance Builds

```bash
# Release build with LTO
cargo build --release

# PGO (Profile-Guided Optimization) - 10-30% performance improvement
make pgo              # Full PGO build with profiling
make pgo-quick        # Quick PGO build using existing profiles
make pgo-clean        # Clean PGO data
```

## Development Conventions

### Coding Style

- **Rust**: Use `rustfmt` defaults (`cargo fmt`)
- **C runtime**: 4-space indentation, consistent with surrounding files
- **Naming**:
  - Rust: `snake_case` for functions/variables, `CamelCase` for enums/types
  - C runtime: `snake_case` with `vp_` prefix (e.g., `vp_list_create`)

### Error Handling

- Use `Result<T, ViperError>` pattern throughout
- Prefer explicit error propagation
- Use `expect()` with descriptive messages in codegen
- Runtime errors should use `vp_panic()` for consistency

### Testing Practices

- Add unit tests near the subsystem you change
- Integration tests for cross-boundary behavior
- Test files use `test_*` naming convention
- Run `make test` before submitting changes
- For performance changes, run `make bench-safe-one` or targeted benchmarks

### Commit Guidelines

- Short imperative subjects
- Use conventional prefixes: `refactor(parser):`, `fix(codegen):`, `feat(runtime):`
- Include:
  - Problem statement
  - Implementation summary
  - Tests and benchmarks run
  - Performance notes (if applicable)

### Project Principles

1. **Python compatibility first** - Preserve Python semantics for `int` (arbitrary precision)
2. **Performance through optimization** - Not by weakening semantics
3. **Explicit opt-in for low-level types** - `i64` is an escape hatch, not the default
4. **Fast because implementation is good** - Not because semantics were weakened

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point |
| `src/lib.rs` | Library root |
| `src/codegen/expressions/core.rs` | Main expression codegen |
| `src/codegen/statements/core/dispatch.rs` | Statement codegen dispatch |
| `src/parser/expressions.rs` | Expression parsing |
| `runtime/src/runtime.c` | Core runtime functions |
| `runtime/include/viper_stdlib.h` | Runtime function declarations |
| `std/core/math.vp` | Math module |
| `std/core/time.vp` | Time module |
| `std/core/random.vp` | Random module |

## Implementation Status

See `plans/IMPLEMENTATION_STATUS.md` for detailed tracking of Python compatibility features.

### Recently Completed

- ✅ List/bytearray repetition (`[elem] * n` syntax)
- ✅ bytearray type (runtime + codegen)
- ✅ f-string format specs (parser + codegen)
- ✅ Default argument support (parser + AST)
- ✅ math.isqrt(), time.perf_counter(), random.randint()

### Known Issues

- List printing may show incorrect values in some cases
- Multi-element list repetition uses loop-based extend (performance optimization pending)

## Useful Commands

```bash
# Run a specific benchmark
make bench-safe-one
# or: cd benchmarks && ./benchmark_runner.sh -i 1 01_fibonacci

# Check for unused variables/imports
cargo build 2>&1 | grep "unused"

# Debug compiler output
RUST_BACKTRACE=1 cargo run --bin viper -- run myfile.vp

# Generate LLVM IR (for debugging)
cargo run --bin viper -- build myfile.vp
llvm-dis myfile_vp.o  # Requires llvm-dis tool
```

## Module System

Viper modules are loaded from:
1. Current directory
2. `std/` directory
3. `std/core/` for core modules
4. `VIPERPATH` environment variable (colon-separated paths)

Import syntax:
```python
import math
from math import sqrt, isqrt
```

## Runtime Functions

Runtime functions follow naming convention:
- `vp_<module>_<function>()` - e.g., `vp_list_create()`, `vp_str_concat()`
- Declared in `runtime/include/viper_stdlib.h`
- Implemented in `runtime/src/*.c`

Common runtime modules:
- `list.c` - Dynamic lists
- `dict.c` - Dictionaries
- `str.c` - Strings (via runtime.c)
- `bitvec.c` - Bit vectors for bool lists
- `bytearray.c` - Mutable byte arrays
- `concurrency.c` - Channels, waitgroups
- `math_mod.c` - Math functions
