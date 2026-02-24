# Viper Language - Project Structure

```
viper-lang/
├── Cargo.toml                    # Rust project configuration
├── Cargo.lock                    # Dependency lock file
├── Makefile                      # Build automation
├── build.rs                      # LLVM linking configuration
├── setup_viper.sh                # Project initialization script
├── publish.sh                    # Production release script
├── Dockerfile                    # Container configuration
│
├── src/                          # Rust compiler source
│   ├── main.rs                   # CLI entry point & orchestration
│   ├── lib.rs                    # Library exports (for testing)
│   │
│   ├── cli/                      # Command-line interface
│   │   ├── mod.rs
│   │   ├── args.rs               # clap argument definitions
│   │   └── commands.rs           # build, run, test, init, etc.
│   │
│   ├── lexer/                    # Lexical analysis
│   │   ├── mod.rs
│   │   ├── scanner.rs            # Character-by-character scanning
│   │   ├── tokens.rs             # Token enum definitions
│   │   ├── indent_stack.rs       # Indentation tracking
│   │   └── tests.rs              # Unit tests
│   │
│   ├── parser/                   # Syntax analysis
│   │   ├── mod.rs
│   │   ├── recursive_descent.rs  # Main parser implementation
│   │   ├── expressions.rs        # Expression parsing (Pratt parser)
│   │   ├── statements.rs         # Statement parsing
│   │   ├── precedence.rs         # Operator precedence table
│   │   └── tests.rs
│   │
│   ├── ast/                      # Abstract Syntax Tree definitions
│   │   ├── mod.rs
│   │   ├── nodes.rs              # Expr, Stmt enums
│   │   ├── types.rs              # Type representations
│   │   ├── visitor.rs            # Visitor trait for tree traversal
│   │   └── printer.rs            # Debug AST printing
│   │
│   ├── semantic/                 # Semantic analysis
│   │   ├── mod.rs
│   │   ├── symbol_table.rs       # Scope management
│   │   ├── type_checker.rs       # Type inference & validation
│   │   ├── reachability.rs       # Dead code detection
│   │   ├── ownership.rs          # ARC analysis
│   │   └── diagnostics.rs        # Error reporting
│   │
│   ├── codegen/                  # LLVM IR generation
│   │   ├── mod.rs
│   │   ├── context.rs            # LLVM context wrapper
│   │   ├── builder.rs            # IR instruction building
│   │   ├── dce.rs                # Dead code elimination
│   │   └── types.rs              # Viper type → LLVM type mapping
│   │
│   └── utils/                    # Shared utilities
│       ├── mod.rs
│       ├── span.rs               # Source location tracking
│       └── source_file.rs        # File loading with encoding
│
├── runtime/                      # C runtime library
│   ├── include/
│   │   ├── viper_stdlib.h        # Public C API header
│   │   ├── viper_types.h         # Type definitions
│   │   └── viper_arc.h           # ARC operations
│   │
│   ├── src/
│   │   ├── runtime.c             # Main runtime initialization
│   │   ├── memory/
│   │   │   └── arc.c             # Atomic Reference Counting
│   │   │
│   │   └── data_structures/
│   │       ├── list.c            # ViperList implementation
│   │       └── dict.c            # ViperDict (hash map)
│   │
│   └── Makefile                  # Runtime build configuration
│
├── std/                          # Viper standard library
│   ├── prelude.vp                # Auto-imported builtins
│   ├── core/
│   │   ├── types.vp
│   │   └── operators.vp
│   │
│   ├── collections/
│   │   ├── list.vp
│   │   └── dict.vp
│   │
│   └── io/
│       ├── file.vp
│       └── path.vp
│
├── tests/                        # Integration tests
│   ├── integration/
│   │   ├── lexer_tests.rs
│   │   ├── parser_tests.rs
│   │   └── codegen_tests.rs
│   │
│   └── *.vp                      # Viper test programs
│
├── examples/                     # Example projects
│   ├── hello_world/
│   ├── rest_api/
│   └── data_processing/
│
├── docs/                         # Documentation
│   ├── README.md
│   ├── INSTALL.md
│   ├── RELEASE_PHASE1.md         # Phase 1 release notes
│   ├── RELEASE_PHASE2.md         # Phase 2 release notes
│   └── CONTRIBUTING.md
│
├── scripts/                      # Utility scripts
│   ├── install.sh
│   └── test_runner.sh
│
├── .github/                      # GitHub configuration
│   └── workflows/
│       └── ci.yml
│
├── .vscode/                      # VS Code configuration
│   ├── extensions.json
│   └── settings.json
│
├── .gitignore
├── LICENSE
├── README.md
├── PROJECT_OVERVIEW.md
└── PROJECT_STRUCTURE.md
```

## Directory Descriptions

### `src/` - Compiler Source Code

| Directory | Purpose |
|-----------|---------|
| `cli/` | Command-line interface with `clap` |
| `lexer/` | Lexical analysis, tokenization |
| `parser/` | Syntax analysis, AST construction |
| `ast/` | Abstract Syntax Tree node definitions |
| `semantic/` | Type checking, symbol table management |
| `codegen/` | LLVM IR generation |
| `utils/` | Shared utilities (spans, file loading) |

### `runtime/` - C Runtime Library

| Directory | Purpose |
|-----------|---------|
| `include/` | Public C API headers |
| `src/memory/` | ARC memory management |
| `src/data_structures/` | Lists, dictionaries |
| `src/runtime.c` | Runtime initialization, builtins |

### `std/` - Viper Standard Library

| Directory | Purpose |
|-----------|---------|
| `prelude.vp` | Auto-imported functions |
| `core/` | Core language utilities |
| `collections/` | Data structure implementations |
| `io/` | File and path operations |

### `tests/` - Test Suite

| Directory | Purpose |
|-----------|---------|
| `integration/` | Rust integration tests |
| `*.vp` | Viper program test files |

### `docs/` - Documentation

| File | Purpose |
|------|---------|
| `README.md` | Project overview |
| `INSTALL.md` | Installation guide |
| `RELEASE_PHASE1.md` | Phase 1 release notes |
| `RELEASE_PHASE2.md` | Phase 2 release notes |
| `CONTRIBUTING.md` | Contribution guidelines |

## Build Artifacts

| File | Generated By |
|------|--------------|
| `target/` | `cargo build` |
| `runtime/*.o` | `make -C runtime` |
| `runtime/libviper.a` | `make -C runtime` |
| `*.bc` | `viper build` |
| `*.opt.bc` | `viper build -O2/-O3` |
