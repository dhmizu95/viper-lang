# Unimplemented Features - Viper Language

## Phase 3 - Still Missing (~40%)

### Concurrency - Channels
| Feature | Description | Priority |
|---------|-------------|----------|
| Select statement | `select { case recv(c1): ... case recv(c2): ... }` | High |
| Channel closing | `close(chan)` | High |
| Range over channel | `for x in chan:` | Medium |

### Concurrency - Tasks
| Feature | Description | Priority |
|---------|-------------|----------|
| Task cancellation | Cancel running tasks | Medium |
| Task priorities | High/normal/low priority | Low |

### Async/Await
| Feature | Description | Priority |
|---------|-------------|----------|
| Async iteration | `async for` | Medium |
| Async context managers | `async with` | Medium |
| `gather()` | Run multiple async tasks | Medium |

### Operators
| Feature | Description | Priority |
|---------|-------------|----------|
| Pipeline operator | `data \|> transform` | Low |

### Control Flow
| Feature | Description | Priority |
|---------|-------------|----------|
| Guard clauses | `unless condition:` | Medium |

### Type System
| Feature | Description | Priority |
|---------|-------------|----------|
| Union types | `int \| str` | High |
| Generic types | `List[T]`, `Dict[K,V]` | High |
| Function types | `fn(int) -> str` | Medium |

---

## Phase 4 - Not Started (~80 features)

### Type System
| Feature | Description | Priority |
|---------|-------------|----------|
| Function overloading | Multiple signatures | Medium |
| Parametric polymorphism | Generic functions | High |

### Control Flow
| Feature | Description | Priority |
|---------|-------------|----------|
| Pattern matching | `match/case` with patterns | High |
| Guard clauses | `unless condition:` | Medium |

### Data Structures
| Feature | Description | Priority |
|---------|-------------|----------|
| Named tuples | `Point(x=1, y=2)` | Medium |
| Frozen sets | Immutable, hashable sets | Low |

### Concurrency - Synchronization
| Feature | Description | Priority |
|---------|-------------|----------|
| RwLock | Read-write lock | Medium |
| Condition | Condition variables | Low |
| Barrier | Synchronization barrier | Low |
| Semaphore | Counted access control | Low |
| Atomic types | `AtomicInt`, `AtomicBool` | Medium |

### Error Handling
| Feature | Description | Priority |
|---------|-------------|----------|
| Error propagation | `?` operator | High |
| Result type | `Result[T, E]` | High |
| `unreachable!` | Assertion for unreachable code | Low |

### Module System
| Feature | Description | Priority |
|---------|-------------|----------|
| Module hot-reloading | Runtime module replacement | Low |

### Built-in Functions
| Feature | Description | Priority |
|---------|-------------|----------|
| `help()` | Documentation | Low |
| `dir()` | Namespace introspection | Low |
| `vars()`, `locals()`, `globals()` | Variable inspection | Low |
| `eval()` | Evaluate expression | Low |
| `exec()` | Execute code | Low |
| `compile()` | Compile to code object | Low |

### Standard Library
| Module | Features | Priority |
|--------|----------|----------|
| `operator` | `itemgetter`, `attrgetter` | Low |
| `pickle` | Object serialization | Low |
| `xml` | XML parsing | Low |
| `html` | HTML escaping | Low |
| `statistics` | `mean`, `median`, `stdev` | Low |
| `decimal` | Arbitrary precision | Low |
| `fractions` | Rational numbers | Low |
| `shutil` | File operations | Low |
| `tempfile` | Temporary files | Low |
| `fileinput` | Iterate over files | Low |
| `calendar` | Calendar operations | Low |
| `zoneinfo` | Timezone support | Low |
| `ssl` | TLS/SSL wrapper | Medium |
| `selectors` | I/O multiplexing | Low |
| `multiprocessing` | Process-based parallelism | Medium |
| `concurrent.futures` | Executors | Low |
| `hmac` | HMAC authentication | Low |
| `secrets` | Secure random | Low |
| `crypto` | AES-GCM, Argon2 | Low |
| `textwrap` | Text wrapping | Low |
| `unicodedata` | Unicode database | Low |
| `codecs` | Encoding/decoding | Low |
| `sqlite3` | SQLite database | Low |
| `dbm` | Key-value database | Low |
| `shelve` | Persistent dictionary | Low |

### Tooling
| Tool | Purpose | Priority |
|------|---------|----------|
| `vpm` | Package manager | High |
| `viper-lsp` | Language server | High |
| `viper fmt` | Code formatter | Medium (partial) |
| `viper lint` | Static analysis | Medium |
| `viper debugger` | Debug support | Low |

### IDE Support
| Feature | Description | Priority |
|---------|-------------|----------|
| Autocompletion | Context-aware suggestions | High |
| Hover information | Type/docs on hover | High |
| Go-to-definition | Navigate to symbol | High |
| Find references | Find all usages | Medium |
| Rename refactoring | Safe symbol rename | Medium |
| Real-time diagnostics | Error squiggles | High |
| VS Code extension | Official extension | Medium |
| Vim/Neovim plugin | LSP client config | Low |
| Emacs mode | LSP client config | Low |

### Documentation
| Feature | Description | Priority |
|---------|-------------|----------|
| `vdoc` tool | Documentation generator | Medium |
| Docstring extraction | Parse `"""docs"""` | Medium |
| Markdown output | `.md` files | Medium |
| HTML output | Static site | Low |
| Cross-references | Link between symbols | Low |

### OOP
| Feature | Description | Priority |
|---------|-------------|----------|
| Descriptors | `__get__`, `__set__`, `__delete__` | Low |
| Metaclasses | `class MyClass(metaclass=Meta):` | Low |
| Final classes | `@final` | Low |
| Sealed methods | `@sealed` | Low |

### Metaprogramming
| Feature | Description | Priority |
|---------|-------------|----------|
| Class decorators | Modify class definition | Medium |
| Reflection | Modify objects at runtime | Low |
| Code generation | Generate code from templates | Low |

### FFI & Interop
| Feature | Description | Priority |
|---------|-------------|----------|
| C header generation | Export Viper to C | Low |

### Memory Management
| Feature | Description | Priority |
|---------|-------------|----------|
| Cycle detection | Optional cycle collector | Medium |
| Small object optimization | Inline small objects | Low |
| Object pooling | Reuse allocated memory | Low |
| Custom allocators | Pluggable memory allocators | Low |
| Memory profiling | Track allocations | Low |

### Optimization
| Feature | Description | Priority |
|---------|-------------|----------|
| Vectorization (SIMD) | Auto-vectorization | Low |
| Profile-guided optimization | PGO | Done in Phase 3 |
| JIT compilation | `eval()` with LLVM JIT | Low |

### Debugging & Profiling
| Feature | Description | Priority |
|---------|-------------|----------|
| Memory profiler | Track allocations | Low |
| CPU profiler | Hot spot detection | Low |
| Coverage analysis | Code coverage | Low |

---

## Priority Implementation Order

### High Priority (Phase 3 completion)
1. Pattern matching (`match/case`)
2. Select statement for channels
3. Union types (`int | str`)
4. Generic types (`List[T]`)
5. Error propagation (`?` operator)
6. `Result[T, E]` type

### Medium Priority (Phase 4 core)
7. Guard clauses (`unless`)
8. Function overloading
9. RwLock, Atomics
10. Package manager (vpm)
11. Language server (viper-lsp)
12. Async iteration

### Low Priority (Phase 4+)
13. Remaining stdlib modules
14. IDE integrations
15. Documentation tools
16. Advanced memory features
