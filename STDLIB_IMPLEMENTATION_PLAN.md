# Viper Standard Library Implementation Plan

This document outlines the implementation plan for missing standard library modules that are frequently used by developers.

---

## Priority Classification

| Priority | Description | Timeline |
|----------|-------------|----------|
| **P0** | Critical - Essential for daily development | Immediate |
| **P1** | High - Common production requirements | Short-term |
| **P2** | Medium - Productivity boosters | Medium-term |
| **P3** | Low - Specialized use cases | Long-term |

---

## P0: Critical Modules (Immediate)

### 1. `string` - String Constants and Utilities

**File:** `std/core/string.vp`

**Purpose:** Common string constants and template substitution

**API to Implement:**

```vp
# Constants
ascii_letters: str      # 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ'
ascii_lowercase: str    # 'abcdefghijklmnopqrstuvwxyz'
ascii_uppercase: str    # 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'
digits: str             # '0123456789'
hexdigits: str          # '0123456789abcdefABCDEF'
octdigits: str          # '01234567'
punctuation: str        # '!"#$%&\'()*+,-./:;<=>?@[\\]^_`{|}~'
printable: str          # Digits + letters + punctuation + whitespace
whitespace: str         # ' \t\n\r\x0b\x0c'

# Template class
class Template:
    def __init__(self, template: str)
    def substitute(self, mapping, **kwargs) -> str
    def safe_substitute(self, mapping, **kwargs) -> str
    def get_identifiers() -> [str]
```

**Implementation Notes:**
- Constants are pre-defined strings
- Template uses `$identifier` or `${identifier}` syntax
- `safe_substitute` leaves missing keys unchanged

**Dependencies:** None

---

### 2. `base64` - Base64 Encoding/Decoding

**File:** `std/core/base64.vp`

**Purpose:** Encode/decode data using Base64 and variants

**API to Implement:**

```vp
# Standard Base64
def b64encode(data: str) -> str
def b64decode(data: str) -> str

# URL-safe Base64
def urlsafe_b64encode(data: str) -> str
def urlsafe_b64decode(data: str) -> str

# Alternative encodings
def b16encode(data: str) -> str
def b16decode(data: str) -> str
def b32encode(data: str) -> str
def b32decode(data: str) -> str

# Helper
def standard_b64encode(data: str) -> str
def standard_b64decode(data: str) -> str
```

**Implementation Notes:**
- Base64 encoding converts 3 bytes to 4 characters
- Padding with `=` for incomplete groups
- URL-safe uses `-` and `_` instead of `+` and `/`
- Runtime functions needed for actual encoding

**Runtime Functions Required:**
```vp
extern def vp_base64_encode(data: str) -> str
extern def vp_base64_decode(data: str) -> str
extern def vp_base64_urlsafe_encode(data: str) -> str
extern def vp_base64_urlsafe_decode(data: str) -> str
```

**Dependencies:** None

---

### 3. `contextlib` - Context Manager Utilities

**File:** `std/core/contextlib.vp`

**Purpose:** Helpers for creating and working with context managers

**API to Implement:**

```vp
# Decorator for generator-based context managers
def contextmanager(func) -> callable

# Close objects with .close() method
class closing:
    def __init__(self, thing)
    def __enter__(self) -> object
    def __exit__(self, exc_type, exc_val, exc_tb)

# Suppress specified exceptions
class suppress:
    def __init__(self, *exceptions)
    def __enter__(self)
    def __exit__(self, exc_type, exc_val, exc_tb) -> bool

# Redirect stdout/stderr
class redirect_stdout:
    def __init__(self, new_target)
    def __enter__(self)
    def __exit__(self, *args)

class redirect_stderr:
    def __init__(self, new_target)
    def __enter__(self)
    def __exit__(self, *args)

# Context manager for acquiring locks
class nullcontext:
    def __init__(self, enter_result=None)
    def __enter__(self) -> object
    def __exit__(self, *args)
```

**Implementation Notes:**
- `@contextmanager` converts a generator function into a context manager
- `yield` in the generator separates `__enter__` and `__exit__` logic
- `suppress` swallows specified exceptions in `__exit__`

**Dependencies:** None

---

### 4. `dataclasses` - Automatic Class Generation

**File:** `std/core/dataclasses.vp`

**Purpose:** Decorator for automatically adding special methods to classes

**API to Implement:**

```vp
# Main decorator
def dataclass(cls=None, /, *, init=True, repr=True, eq=True,
              order=False, unsafe_hash=False, frozen=False)

# Field definition
def field(*, default, default_factory, repr=True, hash=None,
          compare=True, metadata=None)

# Helper functions
def fields(class_or_instance) -> tuple
def asdict(obj) -> dict
def astuple(obj) -> tuple
def make_dataclass(cls_name, fields, *, bases=(), namespace=None,
                   init=True, repr=True, eq=True, order=False,
                   unsafe_hash=False, frozen=False)
def replace(obj, /, **changes)
def is_dataclass(obj) -> bool
```

**Implementation Notes:**
- `@dataclass` adds: `__init__`, `__repr__`, `__eq__`, comparison methods
- `order=True` adds: `__lt__`, `__le__`, `__gt__`, `__ge__`
- `frozen=True` makes instances immutable
- `field()` allows customization of individual fields
- `default_factory` for mutable defaults (e.g., `list`, `dict`)

**Example Usage:**
```vp
@dataclass
class Point:
    x: int
    y: int
    z: int = 0  # Default value

@dataclass(order=True)
class Student:
    name: str
    grade: int
```

**Dependencies:** None

---

### 5. `subprocess` - Process Management

**File:** `std/core/subprocess.vp`

**Purpose:** Spawn and manage external processes

**API to Implement:**

```vp
# Convenience functions
def run(args, *, stdin=None, input=None, stdout=None, stderr=None,
        shell=False, timeout=None, check=False) -> CompletedProcess

def call(args, *, stdin=None, stdout=None, stderr=None, shell=False) -> int

def check_call(args, *, stdin=None, stdout=None, stderr=None, shell=False) -> int

def check_output(args, *, stdin=None, stderr=None, shell=False,
                 timeout=None) -> str

# Popen class for advanced process control
class Popen:
    def __init__(self, args, *, stdin=None, stdout=None, stderr=None,
                 shell=False, cwd=None, env=None)
    def wait(self, timeout=None) -> int
    def poll(self) -> int or None
    def communicate(self, input=None, timeout=None) -> (str, str)
    def kill()
    def terminate()
    def send_signal(signal)

    # Properties
    pid: int
    stdin
    stdout
    stderr
    returncode: int

# Result class
class CompletedProcess:
    def __init__(self, args, returncode, stdout=None, stderr=None)
    def check_returncode()
```

**Implementation Notes:**
- `run()` is the recommended high-level API
- `Popen` for low-level process control
- `shell=True` executes through shell (security consideration)
- `timeout` raises `TimeoutExpired` exception

**Exceptions:**
```vp
class SubprocessError(Exception)
class CalledProcessError(SubprocessError)
    def __init__(self, returncode, cmd, output=None, stderr=None)
class TimeoutExpired(SubprocessError)
```

**Runtime Functions Required:**
```vp
extern def vp_subprocess_run(cmd: str, shell: bool) -> int
extern def vp_subprocess_popen(cmd: str, shell: bool)
extern def vp_subprocess_wait(handle, timeout: float) -> int
extern def vp_subprocess_output(handle) -> str
extern def vp_subprocess_kill(handle)
```

**Dependencies:** None

---

## P1: High Priority Modules

### 6. `pickle` - Object Serialization

**File:** `std/core/pickle.vp`

**Purpose:** Serialize and deserialize Python objects

**API to Implement:**

```vp
# Dump to file
def dump(obj, file, protocol=None, fix_imports=True)
def dumps(obj, protocol=None, fix_imports=True) -> str

# Load from file
def load(file, fix_imports=True, encoding="ASCII") -> object
def loads(data: str, fix_imports=True, encoding="ASCII") -> object

# Pickler class
class Pickler:
    def __init__(self, file, protocol=None, fix_imports=True)
    def dump(obj)
    def persistent_id(obj)

# Unpickler class
class Unpickler:
    def __init__(self, file, fix_imports=True, encoding="ASCII")
    def load() -> object
    def persistent_load(pid)

# Exceptions
class PickleError(Exception)
class PicklingError(PickleError)
class UnpicklingError(PickleError)
```

**Implementation Notes:**
- Protocol versions for compatibility (0-5)
- Handle circular references
- Support custom `__getstate__` and `__setstate__`
- Security: Never unpickle untrusted data

**Runtime Functions Required:**
```vp
extern def vp_pickle_dump(obj, file)
extern def vp_pickle_dumps(obj) -> str
extern def vp_pickle_load(file) -> object
extern def vp_pickle_loads(data: str) -> object
```

**Dependencies:** `io`

---

### 7. `threading` - Thread-based Parallelism

**File:** `std/core/threading.vp`

**Purpose:** Thread-based parallel execution

**API to Implement:**

```vp
# Thread class
class Thread:
    def __init__(self, target=None, args=(), kwargs=None, name=None)
    def start()
    def run()
    def join(timeout=None)
    def is_alive() -> bool
    def getName() -> str
    def setName(name: str)
    def ident -> int
    def native_id -> int

# Lock primitives
class Lock:
    def acquire(blocking=True, timeout=-1) -> bool
    def release()
    def locked() -> bool

class RLock:  # Reentrant lock
    def acquire(blocking=True, timeout=-1) -> bool
    def release()
    def locked() -> bool

# Other synchronization primitives
class Semaphore:
    def __init__(self, value=1)
    def acquire(blocking=True, timeout=None) -> bool
    def release(n=1)

class BoundedSemaphore:
    def __init__(self, value=1)
    def acquire(blocking=True, timeout=None) -> bool
    def release(n=1)

class Event:
    def __init__(self)
    def is_set() -> bool
    def set()
    def clear()
    def wait(timeout=None) -> bool

class Condition:
    def __init__(self, lock=None)
    def acquire(*args)
    def release()
    def wait(timeout=None) -> bool
    def notify(n=1)
    def notify_all()

# Thread-local data
class local:
    pass

# Utility functions
def current_thread() -> Thread
def active_count() -> int
def enumerate() -> [Thread]
def get_ident() -> int
def get_native_id() -> int
def main_thread() -> Thread
def setprofile(func)
def settrace(func)
```

**Implementation Notes:**
- Maps to native OS threads
- `RLock` can be acquired multiple times by same thread
- `Semaphore` with value > 1 allows multiple acquirers
- `Event` is a simple flag for thread communication

**Runtime Functions Required:**
```vp
extern def vp_thread_create(target, args) -> int
extern def vp_thread_join(thread_id, timeout: float)
extern def vp_thread_lock_create()
extern def vp_thread_lock_acquire(lock, timeout: float) -> int
extern def vp_thread_lock_release(lock)
```

**Dependencies:** None

---

### 8. `queue` - Thread-Safe Queues

**File:** `std/core/queue.vp`

**Purpose:** Synchronized queue classes for multi-threaded programming

**API to Implement:**

```vp
# Base class
class Queue:
    def __init__(self, maxsize=0)
    def qsize() -> int
    def empty() -> bool
    def full() -> bool
    def put(item, block=True, timeout=None)
    def put_nowait(item)
    def get(block=True, timeout=None) -> object
    def get_nowait() -> object
    def task_done()
    def join()

# LIFO Queue
class LifoQueue(Queue):
    pass

# Priority Queue
class PriorityQueue(Queue):
    pass

# Simple FIFO for single producer/consumer
class SimpleQueue:
    def __init__(self)
    def empty() -> bool
    def get(block=True) -> object
    def get_nowait() -> object
    def put(item, block=True)
    def put_nowait(item)
    def qsize() -> int

# Exceptions
class Empty(Exception)
class Full(Exception)
```

**Implementation Notes:**
- `maxsize=0` means infinite queue size
- `put()` blocks when queue is full
- `get()` blocks when queue is empty
- `task_done()` signals completion of processing
- `join()` blocks until all items are processed

**Dependencies:** `threading`

---

### 9. `warnings` - Warning System

**File:** `std/core/warnings.vp`

**Purpose:** Issue and control warning messages

**API to Implement:**

```vp
# Issue warnings
def warn(message, category=Warning, stacklevel=1, source=None)
def warn_explicit(message, category, filename, lineno, module=None,
                  registry=None, module_globals=None, source=None)

# Warning filters
def filterwarnings(action, message="", category=Warning, module="",
                   lineno=0, append=False)
def resetwarnings()

# Context manager
class catch_warnings:
    def __init__(self, *args, **kwargs)
    def __enter__()
    def __exit__(*args)

# Warning categories
class Warning(Exception)
class UserWarning(Warning)
class DeprecationWarning(Warning)
class PendingDeprecationWarning(Warning)
class SyntaxWarning(Warning)
class RuntimeWarning(Warning)
class FutureWarning(Warning)
class ImportWarning(Warning)
class UnicodeWarning(Warning)
class BytesWarning(Warning)
class ResourceWarning(Warning)

# Simple warning display
def simplefilter(action, category=Warning, lineno=0, append=False)
def formatwarning(message, category, filename, lineno, line=None) -> str
def showwarning(message, category, filename, lineno, file=None, line=None)
def defaultaction(message, category, filename, lineno, file=None, line=None)
```

**Implementation Notes:**
- Filter actions: "error", "ignore", "always", "default", "module", "once"
- Warnings are filtered based on module, category, message
- `stacklevel` controls which stack frame is reported
- `catch_warnings` temporarily modifies warning filters

**Dependencies:** None

---

### 10. `struct` - Binary Data Packing

**File:** `std/core/struct.vp`

**Purpose:** Pack and unpack binary data

**API to Implement:**

```vp
# Main functions
def pack(format, *values) -> str
def unpack(format, buffer: str) -> tuple
def pack_into(format, buffer, offset, *values)
def unpack_from(format, buffer, offset=0) -> tuple
def calcsize(format) -> int

# Format string characters:
#   x - pad byte
#   c - char
#   b - signed char
#   B - unsigned char
#   ? - bool
#   h - short
#   H - unsigned short
#   i - int
#   I - unsigned int
#   l - long
#   L - unsigned long
#   q - long long
#   Q - unsigned long long
#   f - float
#   d - double
#   s - char[] (string)
#   p - Pascal string
#   @ - native (default)
#   = - native standard
#   < - little-endian
#   > - big-endian
#   ! - network (= big-endian)

# Exceptions
class error(Exception)
```

**Implementation Notes:**
- Format string specifies byte order and data types
- Prefix numbers for arrays: `4i` = 4 integers
- Strings need explicit length: `10s` = 10-char string

**Runtime Functions Required:**
```vp
extern def vp_struct_pack(format: str, values) -> str
extern def vp_struct_unpack(format: str, buffer: str) -> tuple
extern def vp_struct_calcsize(format: str) -> int
```

**Dependencies:** None

---

### 11. `glob` - Unix Pathname Pattern Matching

**File:** `std/core/glob.vp`

**Purpose:** Find pathnames matching Unix shell-style wildcards

**API to Implement:**

```vp
# Find matching paths
def glob(pathname, *, recursive=False) -> [str]
def iglob(pathname, *, recursive=False) -> iterator
def escape(pathname) -> str

# Pattern characters:
#   *     - matches everything except path separator
#   **    - matches everything including directories (recursive)
#   ?     - matches any single character
#   [seq] - matches any character in seq
#   [!seq]- matches any character not in seq
```

**Implementation Notes:**
- Uses `re` module internally for pattern matching
- `**` pattern requires `recursive=True`
- `escape()` removes special meaning from pattern chars
- Results are sorted alphabetically

**Dependencies:** `re`, `os`

---

## P2: Medium Priority Modules

### 12. `bisect` - Binary Search Algorithms

**File:** `std/core/bisect.vp`

**Purpose:** Maintain sorted lists efficiently

**API to Implement:**

```vp
# Search functions
def bisect_left(a: [object], x, lo=0, hi=None) -> int
def bisect_right(a: [object], x, lo=0, hi=None) -> int
def bisect(a: [object], x, lo=0, hi=None) -> int  # Alias for bisect_right

# Insert functions
def insort_left(a: [object], x, lo=0, hi=None)
def insort_right(a: [object], x, lo=0, hi=None)
def insort(a: [object], x, lo=0, hi=None)  # Alias for insort_right
```

**Implementation Notes:**
- O(log n) search, O(n) insertion (list shift)
- `bisect_left` returns leftmost insertion point
- `bisect_right` returns rightmost insertion point
- List must already be sorted

**Dependencies:** None

---

### 13. `heapq` - Heap Queue Algorithms

**File:** `std/core/heapq.vp`

**Purpose:** Implement min-heap queue algorithms

**API to Implement:**

```vp
# Heap operations
def heappush(heap, item)
def heappop(heap) -> object
def heappushpop(heap, item) -> object
def heapreplace(heap, item) -> object
def heapify(x: [object])

# Utility functions
def merge(*iterables) -> iterator
def nlargest(n, iterable, key=None) -> [object]
def nsmallest(n, iterable, key=None) -> [object]
```

**Implementation Notes:**
- Min-heap: smallest element at `heap[0]`
- `heapify` is O(n), repeated `heappush` is O(n log n)
- `nlargest`/`nsmallest` more efficient than sort for small n
- For max-heap, negate values

**Dependencies:** None

---

### 14. `textwrap` - Text Wrapping

**File:** `std/core/textwrap.vp`

**Purpose:** Wrap and fill text paragraphs

**API to Implement:**

```vp
# Main functions
def fill(text, width=70, **kwargs) -> str
def wrap(text, width=70, **kwargs) -> [str]
def shorten(text, width, placeholder=" [...]") -> str
def dedent(text) -> str
def indent(text, prefix, predicate=None) -> str

# TextWrapper class
class TextWrapper:
    def __init__(self, width=70, initial_indent="", subsequent_indent="",
                 expand_tabs=True, replace_whitespace=True,
                 fix_sentence_endings=False, break_long_words=True,
                 drop_whitespace=True, break_on_hyphens=True,
                 max_lines=None, placeholder=" [...]")
    def wrap(text) -> [str]
    def fill(text) -> str
```

**Implementation Notes:**
- Wraps text to specified width
- `dedent()` removes common leading whitespace
- `shorten()` truncates with placeholder
- Handles tabs, multiple spaces, hyphens

**Dependencies:** None

---

### 15. `shutil` - High-level File Operations

**File:** `std/core/shutil.vp`

**Purpose:** High-level file operations

**API to Implement:**

```vp
# Copy operations
def copy(src, dst) -> str
def copy2(src, dst) -> str  # Preserves metadata
def copyfile(src, dst)
def copyfileobj(fsrc, fdst, length=16*1024)
def copymode(src, dst)
def copystat(src, dst, follow_symlinks=True)
def copytree(src, dst, symlinks=False, ignore=None,
             copy_function=copy2, ignore_dangling_symlinks=False) -> str

# Move operations
def move(src, dst, copy_function=copy2) -> str

# Remove operations
def rmtree(path, ignore_errors=False, onerror=None)

# Archive operations
def make_archive(base_name, format, root_dir=None, base_dir=None,
                 verbose=False, dry_run=False, owner=None, group=None) -> str
def get_archive_formats() -> [(str, str)]
def get_unpack_formats() -> [(str, str)]
def unpack_archive(filename, extract_dir=None, format=None)

# Disk usage
def disk_usage(path) -> tuple
def which(cmd, mode=os.F_OK | os.X_OK, path=None) -> str

# Other utilities
def chown(path, user, group)
def get_terminal_size(fallback=(80, 24)) -> tuple
def get_archive_formats() -> [(str, str)]
```

**Implementation Notes:**
- `copytree()` recursively copies directories
- `rmtree()` recursively removes directories
- `move()` uses rename if possible, copy+remove otherwise
- Archive formats: 'zip', 'tar', 'gztar', 'bztar', 'xztar'

**Dependencies:** `os`, `pathlib`, `tarfile` (optional)

---

## P3: Low Priority Modules

### 16-20. Additional Modules

| Module | File | Description |
|--------|------|-------------|
| `statistics` | `std/statistics.vp` | mean, median, mode, stdev, variance |
| `fractions` | `std/fractions.vp` | Fraction class for rational numbers |
| `decimal` | `std/decimal.vp` | Decimal floating-point arithmetic |
| `sqlite3` | `std/sqlite3.vp` | SQLite database interface |
| `ssl` | `std/ssl.vp` | TLS/SSL wrapper for sockets |

These modules have stub implementations already in place.

---

## Implementation Roadmap

### Phase 1: P0 Critical (Weeks 1-2)
- [ ] `string` - String constants and Template
- [ ] `base64` - Base64 encoding/decoding
- [ ] `contextlib` - Context manager utilities
- [ ] `dataclasses` - Automatic class generation
- [ ] `subprocess` - Process management

### Phase 2: P1 High (Weeks 3-4)
- [ ] `pickle` - Object serialization
- [ ] `threading` - Thread-based parallelism
- [ ] `queue` - Thread-safe queues
- [ ] `warnings` - Warning system
- [ ] `struct` - Binary data packing
- [ ] `glob` - Pathname pattern matching

### Phase 3: P2 Medium (Weeks 5-8)

#### Core Utilities
- [ ] `bisect` - Binary search algorithms
- [ ] `heapq` - Heap queue algorithms
- [ ] `textwrap` - Text wrapping
- [ ] `shutil` - High-level file operations

#### Data & Configuration
- [ ] `configparser` - Configuration file parsing (INI files)
- [ ] `json` enhancements - JSON5 support, custom encoders/decoders
- [ ] `csv` enhancements - Dialect registration, Sniffer class

#### Development Tools
- [ ] `pprint` - Pretty-printing for data structures
- [ ] `reprlib` - Alternative repr with recursion limiting
- [ ] `inspect` - Object inspection and introspection
- [ ] `dis` - Bytecode disassembler

#### Numeric & Math
- [ ] `cmath` - Complex number mathematics
- [ ] `numbers` - Numeric abstract base classes
- [ ] `array` - Space-efficient typed arrays
- [ ] `matrix` - Matrix operations (optional numpy-lite)

#### System & IO Extensions
- [ ] `tempfile` enhancements - NamedTemporaryFile, SpooledTemporaryFile
- [ ] `fileinput` - Iterate over multiple input files
- [ ] `glob` - Unix pathname pattern matching (moved from P1)
- [ ] `fnmatch` - Unix filename pattern matching

#### Functional Programming
- [ ] `functools` enhancements - cached_property, partialmethod
- [ ] `itertools` enhancements - Additional recipes
- [ ] `operator` enhancements - Complete operator set

#### Object & Reference
- [ ] `weakref` - Weak references and finalizers
- [ ] `contextlib` enhancements - asynccontextmanager, AsyncExitStack
- [ ] `abc` - Abstract Base Classes

#### String & Text
- [ ] `string` enhancements - Formatter, Template advanced features
- [ ] `difflib` - Sequence comparison utilities
- [ ] `codecs` - Codec registry and base classes

### Phase 4: P3 Low (Weeks 9-12)

#### Specialized Math & Science
- [ ] `statistics` - Statistical functions (mean, median, mode, stdev)
- [ ] `fractions` - Fraction type for rational numbers
- [ ] `decimal` - Decimal floating-point arithmetic
- [ ] `cmath` enhancements - Advanced complex functions

#### Database & Persistence
- [ ] `sqlite3` - SQLite database interface
- [ ] `dbm` - Simple key-value database
- [ ] `shelve` - Persistent dictionary
- [ ] `pickle` enhancements - Protocol 5, optimization

#### Networking & Security
- [ ] `ssl` - SSL/TLS wrapper for sockets
- [ ] `socketserver` - Socket server framework
- [ ] `http.server` enhancements - Advanced HTTP server features
- [ ] `urllib.parse` enhancements - Complete URL handling

#### Testing & Debugging
- [ ] `unittest` enhancements - Additional assertions, mock support
- [ ] `pdb` - Interactive debugger
- [ ] `coverage` - Code coverage measurement
- [ ] `profile` - Deterministic profiling

#### Platform & System
- [ ] `platform` - Platform identification
- [ ] `ctypes` - Foreign function library
- [ ] `mmap` - Memory-mapped file support
- [ ] `signal` - Signal handling

#### Additional Utilities
- [ ] `calendar` - Calendar utilities
- [ ] `locale` - Internationalization services
- [ ] `gettext` - Translation system
- [ ] `uuid` - UUID generation

---

## Runtime Function Requirements

### Summary of Required C Runtime Functions

```c
// Base64
const char* vp_base64_encode(const char* data, size_t len);
const char* vp_base64_decode(const char* data, size_t len);

// Subprocess
int vp_subprocess_run(const char* cmd, bool shell);
void* vp_subprocess_popen(const char* cmd, bool shell);
int vp_subprocess_wait(void* handle, float timeout);
const char* vp_subprocess_output(void* handle);
void vp_subprocess_kill(void* handle);

// Pickle
void vp_pickle_dump(void* obj, FILE* file);
const char* vp_pickle_dumps(void* obj);
void* vp_pickle_load(FILE* file);
void* vp_pickle_loads(const char* data);

// Threading
int vp_thread_create(void (*target)(void*), void* args);
void vp_thread_join(int thread_id, float timeout);
void* vp_thread_lock_create();
int vp_thread_lock_acquire(void* lock, float timeout);
void vp_thread_lock_release(void* lock);

// Struct
const char* vp_struct_pack(const char* format, void* values);
void* vp_struct_unpack(const char* format, const char* buffer);
int vp_struct_calcsize(const char* format);
```

---

## Testing Strategy

Each module should include:

1. **Unit tests** - Test individual functions
2. **Integration tests** - Test module interactions
3. **Edge case tests** - Test boundary conditions
4. **Compatibility tests** - Verify Python compatibility

Test files should be placed in `tests/test_<module>.vp`

---

## Documentation Requirements

Each module should include:

1. **Module docstring** - Overview of module purpose
2. **Function docstrings** - Args, return values, exceptions
3. **Usage examples** - Common use cases
4. **Compatibility notes** - Differences from Python stdlib

---

## Code Style Guidelines

1. Follow existing Viper stdlib conventions
2. Use type annotations where possible
3. Include comprehensive docstrings
4. Handle errors gracefully with appropriate exceptions
5. Maintain Python stdlib API compatibility

---

## Progress Tracking

| Module | Status | Tests | Docs | Notes |
|--------|--------|-------|------|-------|
| string | ⬜ Pending | ⬜ | ⬜ | |
| base64 | ⬜ Pending | ⬜ | ⬜ | |
| contextlib | ⬜ Pending | ⬜ | ⬜ | |
| dataclasses | ⬜ Pending | ⬜ | ⬜ | |
| subprocess | ⬜ Pending | ⬜ | ⬜ | |
| pickle | ⬜ Pending | ⬜ | ⬜ | |
| threading | ⬜ Pending | ⬜ | ⬜ | |
| queue | ⬜ Pending | ⬜ | ⬜ | |
| warnings | ⬜ Pending | ⬜ | ⬜ | |
| struct | ⬜ Pending | ⬜ | ⬜ | |
| glob | ⬜ Pending | ⬜ | ⬜ | |
| bisect | ⬜ Pending | ⬜ | ⬜ | |
| heapq | ⬜ Pending | ⬜ | ⬜ | |
| textwrap | ⬜ Pending | ⬜ | ⬜ | |
| shutil | ⬜ Pending | ⬜ | ⬜ | |

---

## References

- Python Standard Library: https://docs.python.org/3/library/
- Viper Core Language Features: `CORE_LANGUAGE_FEATURES.md`
- Existing stdlib modules: `std/` directory
