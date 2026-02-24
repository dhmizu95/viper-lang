viper/
├── Cargo.toml                    # Rust project configuration
├── Cargo.lock                    # Dependency lock file
├── Makefile                      # Build automation
├── build.rs                      # LLVM linking configuration
├── setup_viper.sh               # Project initialization script
├── publish.sh                   # Production release script
├── viper.toml                   # Default project manifest (template)
├── Dockerfile                   # Container configuration
│
├── src/                         # Rust compiler source
│   ├── main.rs                  # CLI entry point & orchestration
│   ├── lib.rs                   # Library exports (for testing)
│   │
│   ├── cli/                     # Command-line interface
│   │   ├── mod.rs
│   │   ├── args.rs              # clap argument definitions
│   │   └── commands.rs          # build, run, test, init, etc.
│   │
│   ├── lexer/                   # Lexical analysis
│   │   ├── mod.rs
│   │   ├── scanner.rs           # Character-by-character scanning
│   │   ├── tokens.rs            # Token enum definitions
│   │   ├── indent_stack.rs      # Indentation tracking
│   │   └── tests.rs             # Unit tests
│   │
│   ├── parser/                  # Syntax analysis
│   │   ├── mod.rs
│   │   ├── recursive_descent.rs # Main parser implementation
│   │   ├── expressions.rs       # Expression parsing (Pratt parser)
│   │   ├── statements.rs        # Statement parsing
│   │   ├── precedence.rs        # Operator precedence table
│   │   └── tests.rs
│   │
│   ├── ast/                     # Abstract Syntax Tree definitions
│   │   ├── mod.rs
│   │   ├── nodes.rs             # Expr, Stmt enums
│   │   ├── types.rs             # Type representations
│   │   ├── visitor.rs           # Visitor trait for tree traversal
│   │   └── printer.rs           # Debug AST printing
│   │
│   ├── semantic/                # Semantic analysis
│   │   ├── mod.rs
│   │   ├── symbol_table.rs      # Scope management
│   │   ├── type_checker.rs      # Type inference & validation
│   │   ├── reachability.rs      # Dead code detection
│   │   ├── ownership.rs         # ARC analysis
│   │   └── diagnostics.rs       # Error reporting
│   │
│   ├── codegen/                 # LLVM IR generation
│   │   ├── mod.rs
│   │   ├── context.rs           # LLVM context wrapper
│   │   ├── module.rs            # LLVM module management
│   │   ├── builder.rs           # IR instruction building
│   │   ├── types.rs             # Viper type → LLVM type mapping
│   │   ├── values.rs            # Value generation (constants, variables)
│   │   ├── expressions.rs       # Expression compilation
│   │   ├── statements.rs        # Statement compilation
│   │   ├── functions.rs         # Function definition/calls
│   │   ├── classes.rs           # OOP: VTables, methods
│   │   ├── concurrency.rs       # sync, task, chan, async/await
│   │   ├── memory.rs            # alloca, load, store, ARC calls
│   │   ├── control_flow.rs      # if/else, loops, branches
│   │   ├── optimizations.rs     # Custom LLVM passes
│   │   └── debug_info.rs        # DWARF generation
│   │
│   ├── linker/                  # Final binary generation
│   │   ├── mod.rs
│   │   ├── object_file.rs       # .o file emission
│   │   ├── system_linker.rs     # cc/clang invocation
│   │   └── static_lib.rs        # libviper.a linking
│   │
│   └── utils/                   # Shared utilities
│       ├── mod.rs
│       ├── source_file.rs       # File loading with encoding
│       ├── span.rs              # Source location tracking
│       ├── interner.rs          # String interning
│       └── config.rs            # Compiler configuration
│
├── runtime/                     # C runtime library
│   ├── include/
│   │   ├── viper_stdlib.h       # Public C API header
│   │   ├── viper_types.h        # Type definitions
│   │   └── viper_atomic.h       # Atomic operations
│   │
│   ├── src/
│   │   ├── runtime.c            # Main runtime initialization
│   │   ├── memory/
│   │   │   ├── arc.c            # Atomic Reference Counting
│   │   │   ├── allocator.c      # malloc/free wrappers
│   │   │   ├── weak_ref.c       # Weak reference support
│   │   │   └── cycle_detector.c # Cycle detection (optional)
│   │   │
│   │   ├── data_structures/
│   │   │   ├── list.c           # ViperList implementation
│   │   │   ├── dict.c           # ViperDict (hash map)
│   │   │   ├── set.c            # ViperSet
│   │   │   ├── tuple.c          # ViperTuple
│   │   │   ├── string.c         # ViperString
│   │   │   └── slice.c          # Zero-copy slicing
│   │   │
│   │   ├── concurrency/
│   │   │   ├── thread_pool.c    # M:N scheduler
│   │   │   ├── task_queue.c     # Work-stealing deque
│   │   │   ├── channel.c        # chan implementation
│   │   │   ├── wait_group.c     # sync block support
│   │   │   ├── mutex.c          # Synchronization primitives
│   │   │   └── async_runtime.c  # Event loop (epoll/kqueue/IOCP)
│   │   │
│   │   ├── io/
│   │   │   ├── file.c           # File operations
│   │   │   ├── socket.c         # Network primitives
│   │   │   └── buffer.c         # I/O buffering
│   │   │
│   │   ├── crypto/
│   │   │   ├── hash.c           # SHA-256, Argon2
│   │   │   └── cipher.c         # AES-GCM
│   │   │
│   │   ├── math/
│   │   │   ├── basic.c          # sqrt, sin, cos, etc.
│   │   │   ├── complex.c        # Complex number ops
│   │   │   └── vector.c         # SIMD wrappers
│   │   │
│   │   └── exception/
│   │       ├── unwinding.c      # Stack unwinding
│   │       └── landing_pad.c    # LLVM personality function
│   │
│   └── Makefile                 # Runtime build configuration
│
├── std/                         # Viper standard library
│   ├── prelude.vp               # Auto-imported builtins
│   ├── builtins/
│   │   ├── print.vp             # print(), format()
│   │   ├── range.vp             # range(), enumerate()
│   │   ├── len.vp               # len(), sizeof()
│   │   ├── typeof.vp            # typeof(), isinstance()
│   │   └── assert.vp            # assert(), test()
│   │
│   ├── core/
│   │   ├── types.vp             # Type definitions
│   │   ├── operators.vp         # Operator overloads
│   │   ├── iterators.vp         # Iterator protocol
│   │   ├── context.vp           # with statements
│   │   └── decorators.vp        # @decorator syntax
│   │
│   ├── collections/
│   │   ├── list.vp              # List methods
│   │   ├── dict.vp              # Dict methods
│   │   ├── set.vp               # Set operations
│   │   ├── tuple.vp             # Tuple utilities
│   │   └── deque.vp             # Double-ended queue
│   │
│   ├── math/
│   │   ├── __init__.vp
│   │   ├── constants.vp         # pi, e, etc.
│   │   ├── trig.vp              # Trigonometric functions
│   │   ├── stats.vp             # Statistics
│   │   └── random.vp            # RNG
│   │
│   ├── io/
│   │   ├── __init__.vp
│   │   ├── file.vp              # File class
│   │   ├── path.vp              # Path manipulation
│   │   ├── buffer.vp            # Buffered I/O
│   │   └── serialization/
│   │       ├── json.vp          # JSON parser
│   │       ├── csv.vp           # CSV parser
│   │       └── binary.vp        # Binary formats
│   │
│   ├── net/
│   │   ├── __init__.vp
│   │   ├── socket.vp            # Low-level sockets
│   │   ├── http.vp              # HTTP client/server
│   │   ├── url.vp               # URL parsing
│   │   └── async.vp             # asyncio module
│   │
│   ├── os/
│   │   ├── __init__.vp
│   │   ├── env.vp               # Environment variables
│   │   ├── process.vp           # Process management
│   │   ├── fs.vp                # Filesystem operations
│   │   └── time.vp              # Clocks, timers
│   │
│   ├── crypto/
│   │   ├── __init__.vp
│   │   ├── hash.vp              # Hashing interface
│   │   ├── cipher.vp            # Encryption
│   │   └── random.vp            # Secure random
│   │
│   ├── concurrency/
│   │   ├── __init__.vp
│   │   ├── sync.vp              # sync, task primitives
│   │   ├── channel.vp           # chan utilities
│   │   ├── atomic.vp            # Atomic types
│   │   └── multiprocessing.vp   # Process pools
│   │
│   ├── re/                      # Regular expressions
│   │   ├── __init__.vp
│   │   ├── pattern.vp
│   │   └── engine.vp
│   │
│   └── testing/
│       ├── __init__.vp
│       ├── test.vp              # Test runner
│       ├── mock.vp              # Mocking
│       └── bench.vp             # Benchmarking
│
├── vpm/                         # Package manager (separate crate)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # vpm CLI
│       ├── commands/
│       │   ├── init.rs          # vpm init
│       │   ├── install.rs       # vpm install
│       │   ├── add.rs           # vpm add
│       │   ├── build.rs         # vpm build
│       │   ├── test.rs          # vpm test
│       │   └── publish.rs       # vpm publish
│       ├── resolver/
│       │   ├── mod.rs
│       │   ├── dependency.rs    # Dependency graph
│       │   ├── version.rs       # SemVer handling
│       │   └── lockfile.rs      # viper.lock generation
│       ├── registry/
│       │   ├── mod.rs
│       │   ├── git.rs           # Git-based packages
│       │   ├── local.rs         # Local path packages
│       │   └── index.rs         # Registry index
│       └── manifest/
│           ├── mod.rs
│           ├── parser.rs        # viper.toml parsing
│           └── validation.rs
│
├── viper-lsp/                   # Language Server Protocol
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── server.rs
│       ├── handlers.rs          # LSP method handlers
│       ├── completion.rs        # Autocompletion
│       ├── hover.rs             # Type info on hover
│       ├── goto.rs              # Go-to-definition
│       ├── diagnostics.rs       # Real-time errors
│       └── symbols.rs           # Document symbols
│
├── viper-fmt/                   # Code formatter
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── rules.rs             # Formatting rules
│       └── indent.rs            # Indentation handling
│
├── vdoc/                        # Documentation generator
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── parser.rs            # Docstring extraction
│       ├── markdown.rs          # Markdown generation
│       └── html.rs              # HTML output
│
├── tests/                       # Integration tests
│   ├── integration/
│   │   ├── lexer_tests.rs
│   │   ├── parser_tests.rs
│   │   ├── codegen_tests.rs
│   │   └── end_to_end.rs
│   │
│   ├── vscripts/                # Viper test programs
│   │   ├── 01_hello.vp
│   │   ├── 02_math.vp
│   │   ├── 03_functions.vp
│   │   ├── 04_classes.vp
│   │   ├── 05_concurrency.vp
│   │   ├── 06_generics.vp
│   │   ├── 07_exceptions.vp
│   │   ├── 08_file_io.vp
│   │   ├── 09_json.vp
│   │   ├── 10_http.vp
│   │   └── stress/
│   │       ├── fibonacci.vp
│   │       ├── million_list.vp
│   │       └── web_server.vp
│   │
│   └── expected/                # Expected outputs
│       ├── 01_hello.out
│       └── ...
│
├── examples/                    # Example projects
│   ├── hello_world/
│   │   ├── viper.toml
│   │   └── src/main.vp
│   │
│   ├── rest_api/
│   │   ├── viper.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.vp
│   │       ├── models.vp
│   │       └── routes.vp
│   │
│   ├── data_processing/
│   │   ├── viper.toml
│   │   └── src/
│   │       ├── main.vp
│   │       └── pipeline.vp
│   │
│   └── concurrent_server/
│       ├── viper.toml
│       └── src/
│           ├── main.vp
│           └── handlers.vp
│
├── docs/                        # Documentation
│   ├── README.md
│   ├── INSTALL.md
│   ├── LANGUAGE_SPEC.md        # Formal specification
│   ├── WHITEPAPER.md
│   ├── QUICK_START.md
│   ├── API_REFERENCE/
│   │   ├── std.md
│   │   └── runtime.md
│   │
│   ├── INTERNALS/
│   │   ├── architecture.md
│   │   ├── memory_model.md
│   │   ├── concurrency.md
│   │   └── codegen.md
│   │
│   └── CONTRIBUTING.md
│
├── scripts/                     # Utility scripts
│   ├── install.sh               # System-wide install
│   ├── uninstall.sh
│   ├── test_runner.sh
│   └── benchmark.sh
│
├── .github/                     # GitHub configuration
│   ├── workflows/
│   │   ├── ci.yml               # Continuous integration
│   │   ├── release.yml
│   │   └── docs.yml
│   ├── ISSUE_TEMPLATE/
│   └── PULL_REQUEST_TEMPLATE.md
│
├── .vscode/                     # VS Code configuration
│   ├── extensions.json
│   ├── settings.json
│   └── launch.json
│
├── .gitignore
├── LICENSE                      # MIT or Apache-2.0
└── README.md                    # Project overview