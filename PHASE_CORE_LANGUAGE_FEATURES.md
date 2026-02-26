# Core Language Features by Phase

This document reorganizes the language features from `CORE_LANGUAGE_FEATURES.md`, grouped by implementation phase rather than feature category.

---

## Phase 1: Core Compiler

Features that form the foundation of the language.

### Lexical & Syntax

| Feature | Description |
|---------|-------------|
| Indentation-based scoping | Python-style Indent/Dedent tokens |
| Significant whitespace | No braces, colon-start blocks |
| Line comments | `# single line comment` |
| String literals | `"hello"`, `'hello'`, `r"raw"`, `f"f-{var}"` |
| Numeric literals | `42`, `3.14`, `0xFF`, `1e-10` |
| Boolean literals | `True`, `False` |
| None literal | `None` |
| Escape sequences | `\n`, `\t`, `\\`, `\"`, `\x41` |

### Type System

| Feature | Description |
|---------|-------------|
| Static typing | Compile-time type checking |
| Type inference | `x = 5` → `i64` automatically |
| Explicit annotations | `x: i64`, `def f() -> str` |
| Basic types | `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool` |
| BigInt type | `BigInt` - arbitrary precision integers |
| String type | `str` (UTF-8, immutable) |
| Void type | `void` for functions |

### Variables & Assignment

| Feature | Description |
|---------|-------------|
| Immutable by default | `x = 5` cannot be reassigned |

### Operators

| Feature | Description |
|---------|-------------|
| Arithmetic | `+`, `-`, `*`, `/`, `//`, `%`, `**` |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Logical | `and`, `or`, `not` (short-circuiting) |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=`, etc. |
| Increment/Decrement | `++`, `--` (prefix and postfix) |

### Control Flow

| Feature | Description |
|---------|-------------|
| If/elif/else | Conditional branching |
| While loop | `while condition:` |
| For loop | `for item in iterable:` |
| Break | Exit loop early |
| Continue | Skip to next iteration |
| Pass | No-op placeholder |

### Functions

| Feature | Description |
|---------|-------------|
| Function definition | `def name(args):` |
| Return values | `return value` |
| Recursion | Self-calling functions |

### Data Structures

#### Lists

| Feature | Description |
|---------|-------------|
| List literals | `[1, 2, 3]` |
| Indexing | `list[0]`, `list[-1]` |
| Length | `len(list)` |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print()` | Output to stdout |
| `len()` | Length of container |
| `range()` | Integer sequence |
| `str()`, `int()`, `float()`, `bool()` | Type conversion |

### Tooling & Ecosystem

| Tool | Purpose |
|------|---------|
| `viper` | Main compiler |
| `viper build` | Compile to binary |

### Optimization

| Feature | Description |
|---------|-------------|
| LLVM O0-O3 | Optimization levels |

### Platform Support

| Platform | Target |
|----------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` |

### Module System

| Module | Description |
|--------|-------------|
| `builtins` | Auto-imported functions |

---

## Phase 2: Data Structures + ARC

Core data structures, memory management, and standard library basics.

### Lexical & Syntax

| Feature | Description |
|---------|-------------|
| Block comments | `"""multi-line"""` |

### Type System

| Feature | Description |
|---------|-------------|
| Optional types | `T?` or `Option[T]` |
| Type aliases | `type MyInt = i64` |
| Named tuples | `tuple[int, str, float]` |

### Variables & Assignment

| Feature | Description |
|---------|-------------|
| Mutable variables | `mut x = 5` allows reassignment |
| Multiple assignment | `a, b = 1, 2` |
| Unpacking | `a, b = my_tuple` |
| Global variables | `global x` declaration |
| Constants | `const PI = 3.14159` |

### Operators

| Feature | Description |
|---------|-------------|
| Bitwise | `&`, `|`, `^`, `~`, `<<`, `>>` |
| Identity | `is`, `is not` |
| Membership | `in`, `not in` |
| Ternary | `x if cond else y` |

### Control Flow

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Functions

| Feature | Description |
|---------|-------------|
| Multiple returns | `return a, b` (tuple) |
| Default arguments | `def f(x=10):` |
| Keyword arguments | `f(x=5, y=10)` |
| Variable arguments | `*args` (tuple) |
| Keyword varargs | `**kwargs` (dict) |
| Lambda expressions | `lambda x: x * 2` |

### Data Structures

#### Lists

| Feature | Description |
|---------|-------------|
| List comprehension | `[x*2 for x in range(10)]` |
| Slicing | `list[0:5]`, `list[::2]` (zero-copy) |
| Concatenation | `list1 + list2` |
| Repetition | `[0] * 10` |
| Membership | `x in list` |
| Append | `list.append(x)` |
| Extend | `list.extend(other)` |
| Insert | `list.insert(i, x)` |
| Remove | `list.remove(x)` |
| Pop | `list.pop()`, `list.pop(i)` |
| Clear | `list.clear()` |
| Index | `list.index(x)` |
| Count | `list.count(x)` |
| Sort | `list.sort()`, `sorted(list)` |
| Reverse | `list.reverse()`, `reversed(list)` |
| Copy | `list.copy()` (shallow) |

#### Tuples

| Feature | Description |
|---------|-------------|
| Tuple literals | `(1, 2, 3)` or `1, 2, 3` |
| Immutable | Cannot modify after creation |
| Hashable | Can be dict keys |

#### Dictionaries

| Feature | Description |
|---------|-------------|
| Dict literals | `{"key": "value"}` |
| Dict comprehension | `{k: v for k, v in pairs}` |
| Key access | `dict["key"]` |
| Get with default | `dict.get("key", default)` |
| Set default | `dict.setdefault(k, v)` |
| Keys/Values/Items | `dict.keys()`, `dict.values()`, `dict.items()` |
| Update | `dict.update(other)` |
| Pop | `dict.pop("key")`, `dict.popitem()` |
| Membership | `"key" in dict` |

#### Sets

| Feature | Description |
|---------|-------------|
| Set literals | `{1, 2, 3}` |
| Set comprehension | `{x for x in range(10)}` |
| Union | `set1 | set2` or `set1.union(set2)` |
| Intersection | `set1 & set2` |
| Difference | `set1 - set2` |
| Symmetric diff | `set1 ^ set2` |
| Subset/Superset | `set1 <= set2`, `set1 >= set2` |
| Add/Remove | `set.add(x)`, `set.remove(x)` |

#### Strings

| Feature | Description |
|---------|-------------|
| String methods | `upper()`, `lower()`, `strip()`, etc. |
| Formatting | `format()`, f-strings |
| Join | `" ".join(list)` |
| Split | `str.split()`, `str.splitlines()` |
| Replace | `str.replace(old, new)` |
| Find/Index | `str.find()`, `str.index()` |
| Startswith/Endswith | `str.startswith()`, `str.endswith()` |
| Isdigit/Isalpha | Type checking methods |
| Slicing | `str[0:5]` (zero-copy view) |

### Memory Management

| Feature | Description |
|---------|-------------|
| ARC (Atomic Reference Counting) | Automatic memory management |
| Deterministic cleanup | No GC pauses |

### Module System

| Feature | Description |
|---------|-------------|
| `import module` | Standard import |
| `from module import name` | Selective import |
| `from module import *` | Wildcard import |
| `import as alias` | Module aliasing |
| Module search path | `VIPER_PATH` environment |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `input()` | Read from stdin |
| `enumerate()` | Index-value pairs |
| `zip()` | Parallel iteration |
| `map()` | Transform iterable |
| `filter()` | Conditional filtering |
| `reduce()` | Cumulative reduction |
| `sum()`, `min()`, `max()` | Numeric aggregates |
| `abs()`, `round()`, `pow()` | Math utilities |
| `divmod()` | Division with remainder |
| `isinstance()` | Type checking |
| `type()` | Get type of object |
| `id()` | Object identity |
| `hash()` | Hash value |
| `repr()` | String representation |
| `list()`, `dict()`, `set()`, `tuple()` | Container construction |
| `open()` | File opening |

### Core Modules

| Module | Features |
|--------|----------|
| `math` | `sin`, `cos`, `tan`, `sqrt`, `log`, `exp`, `pi`, `e` |
| `random` | `random`, `randint`, `choice`, `shuffle`, `seed` |
| `os` | Environment, files, processes, path |
| `sys` | `argv`, `exit`, `path`, `modules`, `stdout`/`stderr` |
| `time` | `sleep`, `time`, `monotonic`, `perf_counter` |
| `string` | Constants, Template |

### Tooling & Ecosystem

| Tool | Purpose |
|------|---------|
| `viper run` | Compile and execute |
| `viper check` | Syntax/type check only |

### FFI & Interop

| Feature | Description |
|---------|-------------|
| `extern` keyword | Declare C functions |
| `extern "C"` | C calling convention |

### Optimization

| Feature | Description |
|---------|-------------|
| Dead code elimination | Remove unused code |
| Inlining | Function inlining |

### Debugging & Profiling

| Feature | Description |
|---------|-------------|
| Assertions | `assert condition` |

### Platform Support

| Platform | Target |
|----------|--------|
| macOS x86_64 | `x86_64-apple-darwin` |
| macOS ARM64 (M1/M2) | `aarch64-apple-darwin` |
| Windows MSVC | `x86_64-pc-windows-msvc` |

---

## Phase 3: Concurrency + OOP

Object-oriented programming, concurrency primitives, and advanced features.

### Lexical & Syntax

| Feature | Description |
|---------|-------------|
| Byte literals | `b"bytes"` |

### Type System

| Feature | Description |
|---------|-------------|
| Union types | `int | str` |
| Generic types | `List[T]`, `Dict[K,V]` |
| Function types | `fn(int) -> str` |

### Variables & Assignment

| Feature | Description |
|---------|-------------|
| Walrus operator | `if (n := len(x)) > 0:` |
| Nonlocal variables | `nonlocal x` for closures |
| Static variables | `static counter = 0` |

### Control Flow

| Feature | Description |
|---------|-------------|
| Match/Case | Pattern matching |
| Loop else | `for...else`, `while...else` |

### Functions

| Feature | Description |
|---------|-------------|
| Nested functions | Inner function definitions |
| Closures | Capture outer scope variables |
| Tail-call optimization | Optimize tail recursion |
| Async functions | `async def` |
| Generators | `yield` for lazy iteration |
| Coroutines | `async/await` with yield |

### Data Structures

#### Lists

| Feature | Description |
|---------|-------------|
| Deep copy | `deepcopy(list)` |

#### Tuples

| Feature | Description |
|---------|-------------|
| Named tuples | `Point(x=1, y=2)` |

#### Strings

| Feature | Description |
|---------|-------------|
| Encoding | `str.encode()`, `bytes.decode()` |
| Regex | `re.match()`, `re.search()`, `re.sub()` |

#### Sets

| Feature | Description |
|---------|-------------|
| Frozen sets | Immutable, hashable sets |

### Object-Oriented Programming

| Feature | Description |
|---------|-------------|
| Class definition | `class Name:` |
| Constructor | `def __init__(self):` |
| Instance methods | `def method(self):` |
| Instance variables | `self.variable = value` |
| Class variables | `ClassName.variable` |
| Inheritance | `class Child(Parent):` |
| Multiple inheritance | `class C(A, B):` |
| Method overriding | Redefine parent method |
| Super calls | `super().method()` |
| Encapsulation | `self._private` convention |
| Properties | `@property`, `@x.setter` |
| Static methods | `@staticmethod` |
| Class methods | `@classmethod` |
| Abstract methods | `@abstractmethod` |
| Abstract base classes | `class ABC(metaclass=ABCMeta):` |
| Dataclasses | `@dataclass` auto-generated methods |
| Special methods | `__str__`, `__repr__`, `__eq__`, etc. |
| Operator overloading | `__add__`, `__mul__`, etc. |
| Container methods | `__getitem__`, `__setitem__`, `__len__` |
| Iterator methods | `__iter__`, `__next__` |
| Context manager | `__enter__`, `__exit__` |
| Callable objects | `__call__` |

### Memory Management

| Feature | Description |
|---------|-------------|
| Weak references | `Weak[T]` for breaking cycles |
| Escape analysis | Stack allocation when safe |

### Concurrency & Parallelism

#### M:N Threading (Green Threads)

| Feature | Description |
|---------|-------------|
| `sync` blocks | Structured concurrency |
| `task` keyword | Spawn lightweight task |
| Work-stealing scheduler | Load-balanced thread pool |
| Task queues | Per-thread deque with stealing |
| Thread pool sizing | Auto or manual configuration |

#### Channels

| Feature | Description |
|---------|-------------|
| `chan[T]` type | Typed communication channel |
| Buffered channels | `chan(100)` with capacity |
| Unbuffered channels | Synchronous send/receive |
| `send(chan, value)` | Send to channel |
| `recv(chan)` | Receive from channel |

#### Synchronization

| Feature | Description |
|---------|-------------|
| WaitGroup | Wait for multiple tasks |
| `add(wg, n)` | Add to wait counter |
| `done(wg)` | Signal completion |
| `wait(wg)` | Block until zero |
| Mutex | Mutual exclusion lock |

#### Async/Await

| Feature | Description |
|---------|-------------|
| `async def` | Async function definition |
| `await` expression | Suspend until ready |
| `asyncio` module | Event loop and utilities |
| `sleep()` | Non-blocking sleep |
| Native polling | `epoll`/`kqueue`/`IOCP` |
| Zero-cost coroutines | State machine transformation |

### Error Handling

| Feature | Description |
|---------|-------------|
| try/except | Exception catching |
| `except SpecificError` | Typed exception handling |
| `except as e` | Bind exception to variable |
| `else` clause | Run if no exception |
| `finally` clause | Always run |
| `raise` statement | Throw exception |
| Custom exceptions | `class MyError(Exception):` |
| Exception hierarchy | Inheritance for catch-all |
| Stack traces | Automatic traceback |
| Zero-cost exceptions | No overhead when not thrown |
| Panic | Unrecoverable error |

### Module System

| Feature | Description |
|---------|-------------|
| `from . import module` | Relative import |
| `from .. import module` | Parent relative import |
| `__init__.vp` | Package initialization |
| `__all__` | Export control |
| `viper_modules/` | Local package directory |
| Interface files (`.vi`) | Compiled module signatures |
| Circular import detection | Error on cycles |

### Standard Library

| Module | Features |
|--------|----------|
| `types` | Type utilities |
| `typing` | Type hints (Generic, Union, etc.) |
| `collections` | `deque`, `Counter`, `OrderedDict`, `defaultdict` |
| `itertools` | `permutations`, `combinations`, `cycle`, `chain` |
| `functools` | `partial`, `reduce`, `lru_cache`, `wraps` |
| `copy` | Shallow/deep copy |
| `json` | JSON parsing/serialization |
| `csv` | CSV reading/writing |
| `cmath` | Complex number math |
| `datetime` | `date`, `time`, `datetime`, `timedelta` |
| `socket` | Low-level TCP/UDP sockets |
| `http` | HTTP client/server |
| `urllib` | URL parsing, encoding |
| `asyncio` | Async networking |
| `glob` | Pattern matching |
| `io` | `StringIO`, `BytesIO`, buffered I/O |
| `pathlib` | Object-oriented paths |
| `hashlib` | MD5, SHA-1, SHA-256, SHA-512 |
| `re` | Regular expressions (JIT compiled) |
| `threading` | Thread-based parallelism |
| `queue` | Thread-safe queues |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `hasattr()`, `getattr()`, `setattr()` | Attribute introspection |

### Tooling & Ecosystem

| Tool | Purpose |
|------|---------|
| `viper test` | Run test suite |

#### Package Manager (vpm)

| Feature | Description |
|---------|-------------|
| `vpm init` | Initialize project |
| `vpm add <pkg>` | Add dependency |
| `vpm remove <pkg>` | Remove dependency |
| `vpm install` | Install dependencies |
| `vpm build` | Build project |
| `vpm test` | Run project tests |
| Semantic versioning | SemVer compliance |
| Lock files | Reproducible builds |
| Git dependencies | `github.com/user/repo` |
| Local dependencies | `path = "../local"` |

### Metaprogramming

| Feature | Description |
|---------|-------------|
| Decorators | `@decorator` syntax |
| Decorator factories | `@decorator(args)` |
| Introspection | Runtime type inspection |

### FFI & Interop

| Feature | Description |
|---------|-------------|
| Struct layout control | `#[repr(C)]` |
| Pointer types | `*T`, `*mut T` |
| Unsafe blocks | `unsafe { ... }` |

### Optimization

| Feature | Description |
|---------|-------------|
| Link-time optimization (LTO) | Cross-module optimization |
| Loop unrolling | Unroll small loops |

### Debugging & Profiling

| Feature | Description |
|---------|-------------|
| DWARF debug info | Source-level debugging |
| GDB/LLDB integration | Debugger support |
| Stack traces | Runtime backtraces |
| Debug prints | `debug!()` macro |

### Platform Support

| Platform | Target |
|----------|--------|
| Linux ARM64 | `aarch64-unknown-linux-gnu` |

---

## Phase 4: Advanced Features + Tooling

Advanced language features, tooling, and ecosystem support.

### Type System

| Feature | Description |
|---------|-------------|
| Decimal type | `Decimal` - arbitrary precision decimals (built on BigInt infrastructure) |

### Variables & Assignment

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Operators

| Feature | Description |
|---------|-------------|
| Null coalescing | `x ?? default` |
| Pipeline | `data |> transform` |

### Control Flow

| Feature | Description |
|---------|-------------|
| Guard clauses | `unless condition:` |

### Functions

| Feature | Description |
|---------|-------------|
| Function overloading | Multiple signatures |
| Pure functions | `pure def` (no side effects) |

### Data Structures

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Memory Management

| Feature | Description |
|---------|-------------|
| Cycle detection | Optional cycle collector |
| Small object optimization | Inline small objects |
| Object pooling | Reuse allocated memory |
| Custom allocators | Pluggable memory allocators |
| Memory profiling | Track allocations |

### Concurrency & Parallelism

#### M:N Threading (Green Threads)

| Feature | Description |
|---------|-------------|
| Task cancellation | Cancel running tasks |
| Task priorities | High/normal/low priority |

#### Channels

| Feature | Description |
|---------|-------------|
| Select statement | `select { case recv(c1): ... }` |
| Channel closing | `close(chan)` |
| Range over channel | `for x in chan:` |

#### Synchronization

| Feature | Description |
|---------|-------------|
| RwLock | Read-write lock |
| Condition | Condition variables |
| Barrier | Synchronization barrier |
| Semaphore | Counted access control |
| Atomic types | `AtomicInt`, `AtomicBool` |

#### Async/Await

| Feature | Description |
|---------|-------------|
| `async for` | Async iteration |
| `async with` | Async context managers |
| `gather()` | Run multiple async tasks |

#### Multiprocessing

| Feature | Description |
|---------|-------------|
| Process type | OS-level process |
| ProcessPool | Managed process pool |
| Shared memory | `SharedMemory` for IPC |
| Memory mapping | `mmap` support |
| Process queues | Cross-process channels |
| Pickle serialization | Data transfer format |

### Error Handling

| Feature | Description |
|---------|-------------|
| `raise from` | Exception chaining |
| `unreachable!` | Assertion for unreachable code |
| Error propagation | `?` operator or `try!` macro |
| Result type | `Result[T, E]` explicit errors |

### Module System

| Feature | Description |
|---------|-------------|
| Module hot-reloading | Runtime module replacement |

### Standard Library

| Module | Features |
|--------|----------|
| `operator` | `itemgetter`, `attrgetter`, `methodcaller` |
| `pickle` | Object serialization |
| `xml` | XML parsing |
| `html` | HTML escaping |
| `statistics` | `mean`, `median`, `mode`, `stdev` |
| `decimal` | Arbitrary precision decimals (built on BigInt) |
| `fractions` | Rational numbers |
| `numbers` | Abstract base classes |
| `shutil` | File operations, archives |
| `tempfile` | Temporary files/directories |
| `fileinput` | Iterate over files |
| `mmap` | Memory-mapped files |
| `calendar` | Calendar operations |
| `zoneinfo` | Timezone support |
| `ssl` | TLS/SSL wrapper |
| `selectors` | I/O multiplexing |
| `multiprocessing` | Process-based parallelism |
| `concurrent.futures` | `ThreadPoolExecutor`, `ProcessPoolExecutor` |
| `hmac` | HMAC authentication |
| `secrets` | Secure random generation |
| `crypto` (custom) | AES-GCM, Argon2 |
| `textwrap` | Text wrapping/filling |
| `unicodedata` | Unicode database |
| `codecs` | Encoding/decoding |
| `sqlite3` | SQLite database interface |
| `dbm` | Key-value database |
| `shelve` | Persistent dictionary |

### Built-in Functions

| Function | Description |
|----------|-------------|
| `help()` | Documentation |
| `dir()` | Namespace introspection |
| `vars()`, `locals()`, `globals()` | Variable inspection |
| `eval()` | Evaluate expression |
| `exec()` | Execute code |
| `compile()` | Compile to code object |
| `breakpoint()` | Debugger hook |

### Tooling & Ecosystem

| Tool | Purpose |
|------|---------|
| `viper bench` | Run benchmarks |
| `viper doc` | Generate documentation |
| `viper fmt` | Format code |
| `viper lint` | Static analysis |
| `viper repl` | Interactive shell |

#### Package Manager (vpm)

| Feature | Description |
|---------|-------------|
| `vpm update` | Update dependencies |
| `vpm publish` | Publish to registry |
| `vpm search` | Search packages |
| Workspace support | Multi-crate projects |

#### IDE Support

| Feature | Description |
|---------|-------------|
| `viper-lsp` | Language server |
| Autocompletion | Context-aware suggestions |
| Hover information | Type/docs on hover |
| Go-to-definition | Navigate to symbol |
| Find references | Find all usages |
| Rename refactoring | Safe symbol rename |
| Real-time diagnostics | Error squiggles |
| Code formatting | `viper fmt` integration |
| Snippets | Code templates |
| VS Code extension | Official extension |
| Vim/Neovim plugin | LSP client config |
| Emacs mode | LSP client config |

#### Documentation

| Feature | Description |
|---------|-------------|
| `vdoc` tool | Documentation generator |
| Docstring extraction | Parse `"""docs"""` |
| Markdown output | `.md` files |
| HTML output | Static site |
| Cross-references | Link between symbols |
| Type signatures | Auto-generated |
| Examples | Runnable code blocks |

### Metaprogramming

| Feature | Description |
|---------|-------------|
| Class decorators | Modify class definition |
| Metaclasses | `metaclass=Meta` |
| Reflection | Modify objects at runtime |
| Code generation | Generate code from templates |

### FFI & Interop

| Feature | Description |
|---------|-------------|
| C header generation | Export Viper to C |

### Optimization

| Feature | Description |
|---------|-------------|
| Vectorization (SIMD) | Auto-vectorization |

### Debugging & Profiling

| Feature | Description |
|---------|-------------|
| Memory profiler | Track allocations |
| CPU profiler | Hot spot detection |
| Coverage analysis | Code coverage |

### Platform Support

| Platform | Target |
|----------|--------|
| Windows MinGW | `x86_64-pc-windows-gnu` |

---

## Phase 5: Ecosystem + Optimization

Performance optimization, external targets, and advanced ecosystem features.

### Lexical & Syntax

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Type System

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Data Structures

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Object-Oriented Programming

| Feature | Description |
|---------|-------------|
| Descriptors | `__get__`, `__set__`, `__delete__` |
| Metaclasses | `class MyClass(metaclass=Meta):` |
| Final classes | `@final` (no inheritance) |
| Sealed methods | `@sealed` (no overriding) |

### Concurrency & Parallelism

| Feature | Description |
|---------|-------------|
| (none additional) | |

### Metaprogramming

| Feature | Description |
|---------|-------------|
| Macros | Compile-time code transformation |
| AST manipulation | Modify AST programmatically |

### FFI & Interop

| Feature | Description |
|---------|-------------|
| `extern "Python"` | Python interop |
| WASM target | WebAssembly compilation |

### Optimization

| Feature | Description |
|---------|-------------|
| Profile-guided optimization | PGO |
| JIT compilation | `eval()` with LLVM JIT |

### Documentation

| Feature | Description |
|---------|-------------|
| Search | Full-text search |

### Standard Library

| Module | Features |
|--------|----------|
| `numpy` (subset) | Array operations, broadcasting |

### Tooling & Ecosystem

| Tool | Purpose |
|------|---------|
| `viper debugger` | Debug support |

### Platform Support

| Platform | Target |
|----------|--------|
| WebAssembly | `wasm32-unknown-unknown` |
| Embedded (no_std) | Custom targets |

---

## Summary

| Phase | Focus | Feature Categories |
|-------|-------|---------------------|
| Phase 1 | Core Compiler | Lexical & Syntax, Type System, Variables & Assignment, Operators, Control Flow, Functions (basics), Lists (basics), Built-in Functions, Tooling, Optimization, Platform |
| Phase 2 | Data Structures + ARC | Type System (optionals), Variables (mutability), Operators (bitwise, ternary), Functions (args, lambdas), Lists (comprehensions, methods), Tuples, Dictionaries, Sets, Strings, Memory (ARC), Module System, Standard Library, FFI, Optimization |
| Phase 3 | Concurrency + OOP | Type System (generics, unions), Variables (advanced), Control Flow (match), Functions (async, generators), Data Structures (deep copy, encoding), OOP (classes, inheritance), Memory (weak refs), Concurrency (tasks, channels, async), Error Handling, Module System, Standard Library, Tooling, Metaprogramming, FFI, Debugging |
| Phase 4 | Advanced Features + Tooling | Operators (pipeline, null coalescing), Control Flow (guard clauses), Functions (overloading, pure), Memory (cycles, pooling), Concurrency (sync, multiprocessing), Error Handling (propagation), Module System (hot-reload), Standard Library (extensive), Tooling (REPL, LSP, doc), Metaprogramming, FFI, Optimization |
| Phase 5 | Ecosystem + Optimization | OOP (descriptors, metaclasses), Metaprogramming (macros, AST), FFI (Python, WASM), Optimization (PGO, JIT), Documentation (search), Standard Library (numpy), Tooling (debugger), Platform (WASM, embedded) |
