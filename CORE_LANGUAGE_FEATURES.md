# Core Language Features

## Lexical & Syntax

| Feature | Description | Status |
|---------|-------------|--------|
| Indentation-based scoping | Python-style Indent/Dedent tokens | Phase 1 |
| Significant whitespace | No braces, colon-start blocks | Phase 1 |
| Line comments | `# single line comment` | Phase 1 |
| Block comments | `"""multi-line"""` | Phase 2 |
| String literals | `"hello"`, `'hello'`, `r"raw"`, `f"f-{var}"` | Phase 1-2 |
| Numeric literals | `42`, `3.14`, `0xFF`, `1e-10` | Phase 1 |
| Boolean literals | `True`, `False` | Phase 1 |
| None literal | `None` | Phase 1 |
| Escape sequences | `\n`, `\t`, `\\`, `\"`, `\x41` | Phase 1 |
| Byte literals | `b"bytes"` | Phase 3 |

## Type System

| Feature | Description | Status |
|---------|-------------|--------|
| Static typing | Compile-time type checking | Phase 1 |
| Type inference | `x = 5` → `i64` automatically | Phase 1 |
| Explicit annotations | `x: i64`, `def f() -> str` | Phase 1 |
| Basic types | `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `bool` | Phase 1 |
| String type | `str` (UTF-8, immutable) | Phase 1 |
| Void type | `void` for functions | Phase 1 |
| Optional types | `T?` or `Option[T]` | Phase 2 |
| Union types | `int | str` | Phase 3 |
| Generic types | `List[T]`, `Dict[K,V]` | Phase 3 |
| Type aliases | `type MyInt = i64` | Phase 2 |
| Named tuples | `tuple[int, str, float]` | Phase 2 |
| Function types | `fn(int) -> str` | Phase 3 |

## Variables & Assignment

| Feature | Description | Status |
|---------|-------------|--------|
| Immutable by default | `x = 5` cannot be reassigned | Phase 1 |
| Mutable variables | `mut x = 5` allows reassignment | Phase 2 |
| Multiple assignment | `a, b = 1, 2` | Phase 2 |
| Unpacking | `a, b = my_tuple` | Phase 2 |
| Walrus operator | `if (n := len(x)) > 0:` | Phase 3 |
| Global variables | `global x` declaration | Phase 2 |
| Nonlocal variables | `nonlocal x` for closures | Phase 3 |
| Constants | `const PI = 3.14159` | Phase 2 |
| Static variables | `static counter = 0` | Phase 3 |

## Operators

| Feature | Description | Status |
|---------|-------------|--------|
| Arithmetic | `+`, `-`, `*`, `/`, `//`, `%`, `**` | Phase 1 |
| Comparison | `==`, `!=`, `<`, `>`, `<=`, `>=` | Phase 1 |
| Logical | `and`, `or`, `not` (short-circuiting) | Phase 1 |
| Bitwise | `&`, `|`, `^`, `~`, `<<`, `>>` | Phase 2 |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=`, etc. | Phase 1 |
| Identity | `is`, `is not` | Phase 2 |
| Membership | `in`, `not in` | Phase 2 |
| Ternary | `x if cond else y` | Phase 2 |
| Null coalescing | `x ?? default` | Phase 3 |
| Pipeline | `data |> transform` | Phase 4 |

## Control Flow

| Feature | Description | Status |
|---------|-------------|--------|
| If/elif/else | Conditional branching | Phase 1 |
| While loop | `while condition:` | Phase 1 |
| For loop | `for item in iterable:` | Phase 1 |
| Break | Exit loop early | Phase 1 |
| Continue | Skip to next iteration | Phase 1 |
| Pass | No-op placeholder | Phase 1 |
| Match/Case | Pattern matching | Phase 3 |
| Guard clauses | `unless condition:` | Phase 4 |
| Loop else | `for...else`, `while...else` | Phase 3 |

## Functions

| Feature | Description | Status |
|---------|-------------|--------|
| Function definition | `def name(args):` | Phase 1 |
| Return values | `return value` | Phase 1 |
| Multiple returns | `return a, b` (tuple) | Phase 2 |
| Default arguments | `def f(x=10):` | Phase 2 |
| Keyword arguments | `f(x=5, y=10)` | Phase 2 |
| Variable arguments | `*args` (tuple) | Phase 2 |
| Keyword varargs | `**kwargs` (dict) | Phase 2 |
| Lambda expressions | `lambda x: x * 2` | Phase 2 |
| Nested functions | Inner function definitions | Phase 3 |
| Closures | Capture outer scope variables | Phase 3 |
| Recursion | Self-calling functions | Phase 1 |
| Tail-call optimization | Optimize tail recursion | Phase 3 |
| Function overloading | Multiple signatures | Phase 4 |
| Pure functions | `pure def` (no side effects) | Phase 4 |
| Async functions | `async def` | Phase 3 |
| Generators | `yield` for lazy iteration | Phase 3 |
| Coroutines | `async/await` with yield | Phase 3 |

## Data Structures

### Lists

| Feature | Description | Status |
|---------|-------------|--------|
| List literals | `[1, 2, 3]` | Phase 1 |
| List comprehension | `[x*2 for x in range(10)]` | Phase 2 |
| Indexing | `list[0]`, `list[-1]` | Phase 1 |
| Slicing | `list[0:5]`, `list[::2]` (zero-copy) | Phase 2 |
| Concatenation | `list1 + list2` | Phase 2 |
| Repetition | `[0] * 10` | Phase 2 |
| Membership | `x in list` | Phase 2 |
| Length | `len(list)` | Phase 1 |
| Append | `list.append(x)` | Phase 2 |
| Extend | `list.extend(other)` | Phase 2 |
| Insert | `list.insert(i, x)` | Phase 2 |
| Remove | `list.remove(x)` | Phase 2 |
| Pop | `list.pop()`, `list.pop(i)` | Phase 2 |
| Clear | `list.clear()` | Phase 2 |
| Index | `list.index(x)` | Phase 2 |
| Count | `list.count(x)` | Phase 2 |
| Sort | `list.sort()`, `sorted(list)` | Phase 2 |
| Reverse | `list.reverse()`, `reversed(list)` | Phase 2 |
| Copy | `list.copy()` (shallow) | Phase 2 |
| Deep copy | `deepcopy(list)` | Phase 3 |

### Tuples

| Feature | Description | Status |
|---------|-------------|--------|
| Tuple literals | `(1, 2, 3)` or `1, 2, 3` | Phase 2 |
| Named tuples | `Point(x=1, y=2)` | Phase 3 |
| Immutable | Cannot modify after creation | Phase 2 |
| Hashable | Can be dict keys | Phase 2 |

### Dictionaries

| Feature | Description | Status |
|---------|-------------|--------|
| Dict literals | `{"key": "value"}` | Phase 2 |
| Dict comprehension | `{k: v for k, v in pairs}` | Phase 2 |
| Key access | `dict["key"]` | Phase 2 |
| Get with default | `dict.get("key", default)` | Phase 2 |
| Set default | `dict.setdefault(k, v)` | Phase 2 |
| Keys/Values/Items | `dict.keys()`, `dict.values()`, `dict.items()` | Phase 2 |
| Update | `dict.update(other)` | Phase 2 |
| Pop | `dict.pop("key")`, `dict.popitem()` | Phase 2 |
| Membership | `"key" in dict` | Phase 2 |

### Sets

| Feature | Description | Status |
|---------|-------------|--------|
| Set literals | `{1, 2, 3}` | Phase 2 |
| Set comprehension | `{x for x in range(10)}` | Phase 2 |
| Union | `set1 | set2` or `set1.union(set2)` | Phase 2 |
| Intersection | `set1 & set2` | Phase 2 |
| Difference | `set1 - set2` | Phase 2 |
| Symmetric diff | `set1 ^ set2` | Phase 2 |
| Subset/Superset | `set1 <= set2`, `set1 >= set2` | Phase 2 |
| Add/Remove | `set.add(x)`, `set.remove(x)` | Phase 2 |
| Frozen sets | Immutable, hashable sets | Phase 3 |

### Strings

| Feature | Description | Status |
|---------|-------------|--------|
| String methods | `upper()`, `lower()`, `strip()`, etc. | Phase 2 |
| Formatting | `format()`, f-strings | Phase 2 |
| Join | `" ".join(list)` | Phase 2 |
| Split | `str.split()`, `str.splitlines()` | Phase 2 |
| Replace | `str.replace(old, new)` | Phase 2 |
| Find/Index | `str.find()`, `str.index()` | Phase 2 |
| Startswith/Endswith | `str.startswith()`, `str.endswith()` | Phase 2 |
| Isdigit/Isalpha | Type checking methods | Phase 2 |
| Slicing | `str[0:5]` (zero-copy view) | Phase 2 |
| Encoding | `str.encode()`, `bytes.decode()` | Phase 3 |
| Regex | `re.match()`, `re.search()`, `re.sub()` | Phase 3 |

## Object-Oriented Programming

| Feature | Description | Status |
|---------|-------------|--------|
| Class definition | `class Name:` | Phase 3 |
| Constructor | `def __init__(self):` | Phase 3 |
| Instance methods | `def method(self):` | Phase 3 |
| Instance variables | `self.variable = value` | Phase 3 |
| Class variables | `ClassName.variable` | Phase 3 |
| Inheritance | `class Child(Parent):` | Phase 3 |
| Multiple inheritance | `class C(A, B):` | Phase 3 |
| Method overriding | Redefine parent method | Phase 3 |
| Super calls | `super().method()` | Phase 3 |
| Encapsulation | `self._private` convention | Phase 3 |
| Properties | `@property`, `@x.setter` | Phase 3 |
| Static methods | `@staticmethod` | Phase 3 |
| Class methods | `@classmethod` | Phase 3 |
| Abstract methods | `@abstractmethod` | Phase 3 |
| Abstract base classes | `class ABC(metaclass=ABCMeta):` | Phase 3 |
| Dataclasses | `@dataclass` auto-generated methods | Phase 3 |
| Special methods | `__str__`, `__repr__`, `__eq__`, etc. | Phase 3 |
| Operator overloading | `__add__`, `__mul__`, etc. | Phase 3 |
| Container methods | `__getitem__`, `__setitem__`, `__len__` | Phase 3 |
| Iterator methods | `__iter__`, `__next__` | Phase 3 |
| Context manager | `__enter__`, `__exit__` | Phase 3 |
| Callable objects | `__call__` | Phase 3 |
| Descriptors | `__get__`, `__set__`, `__delete__` | Phase 4 |
| Metaclasses | `class MyClass(metaclass=Meta):` | Phase 4 |
| Final classes | `@final` (no inheritance) | Phase 4 |
| Sealed methods | `@sealed` (no overriding) | Phase 4 |

## Memory Management

| Feature | Description | Status |
|---------|-------------|--------|
| ARC (Atomic Reference Counting) | Automatic memory management | Phase 2 |
| Deterministic cleanup | No GC pauses | Phase 2 |
| Weak references | `Weak[T]` for breaking cycles | Phase 3 |
| Cycle detection | Optional cycle collector | Phase 4 |
| Escape analysis | Stack allocation when safe | Phase 3 |
| Small object optimization | Inline small objects | Phase 4 |
| Object pooling | Reuse allocated memory | Phase 4 |
| Custom allocators | Pluggable memory allocators | Phase 4 |
| Memory profiling | Track allocations | Phase 4 |

## Concurrency & Parallelism

### M:N Threading (Green Threads)

| Feature | Description | Status |
|---------|-------------|--------|
| `sync` blocks | Structured concurrency | Phase 3 |
| `task` keyword | Spawn lightweight task | Phase 3 |
| Work-stealing scheduler | Load-balanced thread pool | Phase 3 |
| Task queues | Per-thread deque with stealing | Phase 3 |
| Thread pool sizing | Auto or manual configuration | Phase 3 |
| Task cancellation | Cancel running tasks | Phase 4 |
| Task priorities | High/normal/low priority | Phase 4 |

### Channels

| Feature | Description | Status |
|---------|-------------|--------|
| `chan[T]` type | Typed communication channel | Phase 3 |
| Buffered channels | `chan(100)` with capacity | Phase 3 |
| Unbuffered channels | Synchronous send/receive | Phase 3 |
| `send(chan, value)` | Send to channel | Phase 3 |
| `recv(chan)` | Receive from channel | Phase 3 |
| Select statement | `select { case recv(c1): ... }` | Phase 4 |
| Channel closing | `close(chan)` | Phase 4 |
| Range over channel | `for x in chan:` | Phase 4 |

### Synchronization

| Feature | Description | Status |
|---------|-------------|--------|
| WaitGroup | Wait for multiple tasks | Phase 3 |
| `add(wg, n)` | Add to wait counter | Phase 3 |
| `done(wg)` | Signal completion | Phase 3 |
| `wait(wg)` | Block until zero | Phase 3 |
| Mutex | Mutual exclusion lock | Phase 3 |
| RwLock | Read-write lock | Phase 4 |
| Condition | Condition variables | Phase 4 |
| Barrier | Synchronization barrier | Phase 4 |
| Semaphore | Counted access control | Phase 4 |
| Atomic types | `AtomicInt`, `AtomicBool` | Phase 4 |

### Async/Await

| Feature | Description | Status |
|---------|-------------|--------|
| `async def` | Async function definition | Phase 3 |
| `await` expression | Suspend until ready | Phase 3 |
| `asyncio` module | Event loop and utilities | Phase 3 |
| `async for` | Async iteration | Phase 4 |
| `async with` | Async context managers | Phase 4 |
| `gather()` | Run multiple async tasks | Phase 4 |
| `sleep()` | Non-blocking sleep | Phase 3 |
| Native polling | `epoll`/`kqueue`/`IOCP` | Phase 3 |
| Zero-cost coroutines | State machine transformation | Phase 3 |

### Multiprocessing

| Feature | Description | Status |
|---------|-------------|--------|
| Process type | OS-level process | Phase 4 |
| ProcessPool | Managed process pool | Phase 4 |
| Shared memory | `SharedMemory` for IPC | Phase 4 |
| Memory mapping | `mmap` support | Phase 4 |
| Process queues | Cross-process channels | Phase 4 |
| Pickle serialization | Data transfer format | Phase 4 |

## Error Handling

| Feature | Description | Status |
|---------|-------------|--------|
| try/except | Exception catching | Phase 3 |
| `except SpecificError` | Typed exception handling | Phase 3 |
| `except as e` | Bind exception to variable | Phase 3 |
| `else` clause | Run if no exception | Phase 3 |
| `finally` clause | Always run | Phase 3 |
| `raise` statement | Throw exception | Phase 3 |
| `raise from` | Exception chaining | Phase 4 |
| Custom exceptions | `class MyError(Exception):` | Phase 3 |
| Exception hierarchy | Inheritance for catch-all | Phase 3 |
| Stack traces | Automatic traceback | Phase 3 |
| Zero-cost exceptions | No overhead when not thrown | Phase 3 |
| Panic | Unrecoverable error | Phase 3 |
| `unreachable!` | Assertion for unreachable code | Phase 4 |
| Error propagation | `?` operator or `try!` macro | Phase 4 |
| Result type | `Result[T, E]` explicit errors | Phase 4 |

## Module System

| Feature | Description | Status |
|---------|-------------|--------|
| `import module` | Standard import | Phase 2 |
| `from module import name` | Selective import | Phase 2 |
| `from module import *` | Wildcard import | Phase 2 |
| `import as alias` | Module aliasing | Phase 2 |
| `from . import module` | Relative import | Phase 3 |
| `from .. import module` | Parent relative import | Phase 3 |
| `__init__.vp` | Package initialization | Phase 3 |
| `__all__` | Export control | Phase 3 |
| Module search path | `VIPER_PATH` environment | Phase 2 |
| `viper_modules/` | Local package directory | Phase 3 |
| Interface files (`.vi`) | Compiled module signatures | Phase 3 |
| Circular import detection | Error on cycles | Phase 3 |
| Module hot-reloading | Runtime module replacement | Phase 4 |

## Standard Library

### Built-in Functions

| Function | Description | Status |
|----------|-------------|--------|
| `print()` | Output to stdout | Phase 1 |
| `input()` | Read from stdin | Phase 2 |
| `len()` | Length of container | Phase 1 |
| `range()` | Integer sequence | Phase 1 |
| `enumerate()` | Index-value pairs | Phase 2 |
| `zip()` | Parallel iteration | Phase 2 |
| `map()` | Transform iterable | Phase 2 |
| `filter()` | Conditional filtering | Phase 2 |
| `reduce()` | Cumulative reduction | Phase 2 |
| `sum()`, `min()`, `max()` | Numeric aggregates | Phase 2 |
| `abs()`, `round()`, `pow()` | Math utilities | Phase 2 |
| `divmod()` | Division with remainder | Phase 2 |
| `isinstance()` | Type checking | Phase 2 |
| `hasattr()`, `getattr()`, `setattr()` | Attribute introspection | Phase 3 |
| `type()` | Get type of object | Phase 2 |
| `id()` | Object identity | Phase 2 |
| `hash()` | Hash value | Phase 2 |
| `repr()` | String representation | Phase 2 |
| `str()`, `int()`, `float()`, `bool()` | Type conversion | Phase 1 |
| `list()`, `dict()`, `set()`, `tuple()` | Container construction | Phase 2 |
| `open()` | File opening | Phase 2 |
| `help()` | Documentation | Phase 4 |
| `dir()` | Namespace introspection | Phase 4 |
| `vars()`, `locals()`, `globals()` | Variable inspection | Phase 4 |
| `eval()` | Evaluate expression | Phase 4 |
| `exec()` | Execute code | Phase 4 |
| `compile()` | Compile to code object | Phase 4 |
| `breakpoint()` | Debugger hook | Phase 4 |

### Core Modules

| Module | Features | Status |
|--------|----------|--------|
| `builtins` | Auto-imported functions | Phase 1 |
| `types` | Type utilities | Phase 3 |
| `typing` | Type hints (Generic, Union, etc.) | Phase 3 |
| `collections` | `deque`, `Counter`, `OrderedDict`, `defaultdict` | Phase 3 |
| `itertools` | `permutations`, `combinations`, `cycle`, `chain` | Phase 3 |
| `functools` | `partial`, `reduce`, `lru_cache`, `wraps` | Phase 3 |
| `operator` | `itemgetter`, `attrgetter`, `methodcaller` | Phase 4 |
| `copy` | Shallow/deep copy | Phase 3 |
| `pickle` | Object serialization | Phase 4 |
| `json` | JSON parsing/serialization | Phase 3 |
| `csv` | CSV reading/writing | Phase 3 |
| `xml` | XML parsing | Phase 4 |
| `html` | HTML escaping | Phase 4 |

### Math & Numbers

| Module | Features | Status |
|--------|----------|--------|
| `math` | `sin`, `cos`, `tan`, `sqrt`, `log`, `exp`, `pi`, `e` | Phase 2 |
| `cmath` | Complex number math | Phase 3 |
| `random` | `random`, `randint`, `choice`, `shuffle`, `seed` | Phase 2 |
| `statistics` | `mean`, `median`, `mode`, `stdev` | Phase 4 |
| `decimal` | Arbitrary precision decimals | Phase 4 |
| `fractions` | Rational numbers | Phase 4 |
| `numbers` | Abstract base classes | Phase 4 |
| `numpy` (subset) | Array operations, broadcasting | Phase 5 |

### System & I/O

| Module | Features | Status |
|--------|----------|--------|
| `os` | Environment, files, processes, path | Phase 2 |
| `sys` | `argv`, `exit`, `path`, `modules`, `stdout`/`stderr` | Phase 2 |
| `io` | `StringIO`, `BytesIO`, buffered I/O | Phase 3 |
| `pathlib` | Object-oriented paths | Phase 3 |
| `shutil` | File operations, archives | Phase 4 |
| `glob` | Pattern matching | Phase 3 |
| `tempfile` | Temporary files/directories | Phase 4 |
| `fileinput` | Iterate over files | Phase 4 |
| `mmap` | Memory-mapped files | Phase 4 |

### Time & Date

| Module | Features | Status |
|--------|----------|--------|
| `time` | `sleep`, `time`, `monotonic`, `perf_counter` | Phase 2 |
| `datetime` | `date`, `time`, `datetime`, `timedelta` | Phase 3 |
| `calendar` | Calendar operations | Phase 4 |
| `zoneinfo` | Timezone support | Phase 4 |

### Networking

| Module | Features | Status |
|--------|----------|--------|
| `socket` | Low-level TCP/UDP sockets | Phase 3 |
| `http` | HTTP client/server | Phase 3 |
| `urllib` | URL parsing, encoding | Phase 3 |
| `ssl` | TLS/SSL wrapper | Phase 4 |
| `asyncio` | Async networking | Phase 3 |
| `selectors` | I/O multiplexing | Phase 4 |

### Concurrency

| Module | Features | Status |
|--------|----------|--------|
| `threading` | Thread-based parallelism | Phase 3 |
| `multiprocessing` | Process-based parallelism | Phase 4 |
| `concurrent.futures` | `ThreadPoolExecutor`, `ProcessPoolExecutor` | Phase 4 |
| `asyncio` | Coroutines, event loop, streams | Phase 3 |
| `queue` | Thread-safe queues | Phase 3 |

### Cryptography & Security

| Module | Features | Status |
|--------|----------|--------|
| `hashlib` | MD5, SHA-1, SHA-256, SHA-512 | Phase 3 |
| `hmac` | HMAC authentication | Phase 4 |
| `secrets` | Secure random generation | Phase 4 |
| `crypto` (custom) | AES-GCM, Argon2 | Phase 4 |

### Text Processing

| Module | Features | Status |
|--------|----------|--------|
| `re` | Regular expressions (JIT compiled) | Phase 3 |
| `string` | Constants, Template | Phase 2 |
| `textwrap` | Text wrapping/filling | Phase 4 |
| `unicodedata` | Unicode database | Phase 4 |
| `codecs` | Encoding/decoding | Phase 4 |

### Data Persistence

| Module | Features | Status |
|--------|----------|--------|
| `sqlite3` | SQLite database interface | Phase 4 |
| `dbm` | Key-value database | Phase 4 |
| `shelve` | Persistent dictionary | Phase 4 |

## Tooling & Ecosystem

### Compiler Toolchain

| Tool | Purpose | Status |
|------|---------|--------|
| `viper` | Main compiler | Phase 1 |
| `viper build` | Compile to binary | Phase 1 |
| `viper run` | Compile and execute | Phase 2 |
| `viper check` | Syntax/type check only | Phase 2 |
| `viper test` | Run test suite | Phase 3 |
| `viper bench` | Run benchmarks | Phase 4 |
| `viper doc` | Generate documentation | Phase 4 |
| `viper fmt` | Format code | Phase 4 |
| `viper lint` | Static analysis | Phase 4 |
| `viper repl` | Interactive shell | Phase 4 |
| `viper debugger` | Debug support | Phase 5 |

### Package Manager (vpm)

| Feature | Description | Status |
|---------|-------------|--------|
| `vpm init` | Initialize project | Phase 3 |
| `vpm add <pkg>` | Add dependency | Phase 3 |
| `vpm remove <pkg>` | Remove dependency | Phase 3 |
| `vpm install` | Install dependencies | Phase 3 |
| `vpm update` | Update dependencies | Phase 4 |
| `vpm build` | Build project | Phase 3 |
| `vpm test` | Run project tests | Phase 3 |
| `vpm publish` | Publish to registry | Phase 4 |
| `vpm search` | Search packages | Phase 4 |
| Semantic versioning | SemVer compliance | Phase 3 |
| Lock files | Reproducible builds | Phase 3 |
| Git dependencies | `github.com/user/repo` | Phase 3 |
| Local dependencies | `path = "../local"` | Phase 3 |
| Workspace support | Multi-crate projects | Phase 4 |

### IDE Support

| Feature | Description | Status |
|---------|-------------|--------|
| `viper-lsp` | Language server | Phase 4 |
| Autocompletion | Context-aware suggestions | Phase 4 |
| Hover information | Type/docs on hover | Phase 4 |
| Go-to-definition | Navigate to symbol | Phase 4 |
| Find references | Find all usages | Phase 4 |
| Rename refactoring | Safe symbol rename | Phase 4 |
| Real-time diagnostics | Error squiggles | Phase 4 |
| Code formatting | `viper fmt` integration | Phase 4 |
| Snippets | Code templates | Phase 4 |
| VS Code extension | Official extension | Phase 4 |
| Vim/Neovim plugin | LSP client config | Phase 4 |
| Emacs mode | LSP client config | Phase 4 |

### Documentation

| Feature | Description | Status |
|---------|-------------|--------|
| `vdoc` tool | Documentation generator | Phase 4 |
| Docstring extraction | Parse `"""docs"""` | Phase 4 |
| Markdown output | `.md` files | Phase 4 |
| HTML output | Static site | Phase 4 |
| Cross-references | Link between symbols | Phase 4 |
| Type signatures | Auto-generated | Phase 4 |
| Examples | Runnable code blocks | Phase 4 |
| Search | Full-text search | Phase 5 |

## Advanced Features

### Metaprogramming

| Feature | Description | Status |
|---------|-------------|--------|
| Decorators | `@decorator` syntax | Phase 3 |
| Decorator factories | `@decorator(args)` | Phase 3 |
| Class decorators | Modify class definition | Phase 4 |
| Metaclasses | `metaclass=Meta` | Phase 4 |
| Introspection | Runtime type inspection | Phase 3 |
| Reflection | Modify objects at runtime | Phase 4 |
| Code generation | Generate code from templates | Phase 4 |
| Macros | Compile-time code transformation | Phase 5 |
| AST manipulation | Modify AST programmatically | Phase 5 |

### FFI & Interop

| Feature | Description | Status |
|---------|-------------|--------|
| `extern` keyword | Declare C functions | Phase 2 |
| `extern "C"` | C calling convention | Phase 2 |
| `extern "Python"` | Python interop | Phase 5 |
| Struct layout control | `#[repr(C)]` | Phase 3 |
| Pointer types | `*T`, `*mut T` | Phase 3 |
| Unsafe blocks | `unsafe { ... }` | Phase 3 |
| C header generation | Export Viper to C | Phase 4 |
| WASM target | WebAssembly compilation | Phase 5 |

### Optimization

| Feature | Description | Status |
|---------|-------------|--------|
| LLVM O0-O3 | Optimization levels | Phase 1 |
| Link-time optimization (LTO) | Cross-module optimization | Phase 3 |
| Dead code elimination | Remove unused code | Phase 2 |
| Inlining | Function inlining | Phase 2 |
| Loop unrolling | Unroll small loops | Phase 3 |
| Vectorization (SIMD) | Auto-vectorization | Phase 4 |
| Profile-guided optimization | PGO | Phase 5 |
| JIT compilation | `eval()` with LLVM JIT | Phase 5 |

### Debugging & Profiling

| Feature | Description | Status |
|---------|-------------|--------|
| DWARF debug info | Source-level debugging | Phase 3 |
| GDB/LLDB integration | Debugger support | Phase 3 |
| Stack traces | Runtime backtraces | Phase 3 |
| Assertions | `assert condition` | Phase 2 |
| Debug prints | `debug!()` macro | Phase 3 |
| Memory profiler | Track allocations | Phase 4 |
| CPU profiler | Hot spot detection | Phase 4 |
| Coverage analysis | Code coverage | Phase 4 |

## Platform Support

| Platform | Target | Status |
|----------|--------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Phase 1 |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | Phase 3 |
| macOS x86_64 | `x86_64-apple-darwin` | Phase 2 |
| macOS ARM64 (M1/M2) | `aarch64-apple-darwin` | Phase 2 |
| Windows MSVC | `x86_64-pc-windows-msvc` | Phase 2 |
| Windows MinGW | `x86_64-pc-windows-gnu` | Phase 4 |
| WebAssembly | `wasm32-unknown-unknown` | Phase 5 |
| Embedded (no_std) | Custom targets | Phase 5 |

## Summary by Phase

| Phase | Focus | Feature Count |
|-------|-------|---------------|
| Phase 1 | Core Compiler | ~25 features |
| Phase 2 | Data Structures + ARC | ~60 features |
| Phase 3 | Concurrency + OOP | ~90 features |
| Phase 4 | Advanced Features + Tooling | ~80 features |
| Phase 5 | Ecosystem + Optimization | ~50 features |
| **Total** | | **~305 features** |

---

This list represents a complete, production-ready programming language competitive with Python, Go, and Rust. Prioritize based on your goals—Phase 1-2 gives you a usable language, Phase 3 makes it powerful, Phase 4-5 makes it professional.
