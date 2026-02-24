# Viper Language - Phase 3 Release

**Version:** 0.3.0
**Release Date:** February 25, 2026
**Status:** Alpha

---

## Overview

Phase 3 transforms Viper into a **concurrent programming language** with powerful primitives for parallel execution. Building on the Phase 2 foundation of memory management and data structures, Phase 3 adds channels, WaitGroups, structured concurrency, and fixed-size arrays.

> Concurrent programming meets Python-like simplicity with Go-inspired primitives.

---

## New Features

### 1. Channels (CSP-Style Communication)

Channels provide type-safe communication between concurrent tasks, following the Communicating Sequential Processes (CSP) model.

```python
def worker(id: i64, out_chan):
    result = id * id
    send(out_chan, result)

def main():
    # Create a buffered channel with capacity 10
    c = chan(10)
    
    # Send values
    send(c, 42)
    
    # Receive values
    value = recv(c)
    print(value)
```

**Channel Types:**
- `chan(capacity)` - Create buffered channel
- `chan(0)` - Unbuffered (synchronous) channel
- `send(chan, value)` - Send value to channel
- `recv(chan)` - Receive value from channel

**Type Inference:**
Channel element types are inferred from usage:
```python
def main():
    c = chan(10)      # Channel type inferred from send/recv
    send(c, 42)       # Inferred as chan[i64]
    value = recv(c)   # Returns i64
```

### 2. WaitGroups (Synchronization Primitives)

WaitGroups provide a clean way to wait for multiple concurrent operations to complete.

```python
def worker(id: i64, wg):
    # Do work...
    done(wg)  # Signal completion

def main():
    wg = WaitGroup()
    add(wg, 3)  # Wait for 3 workers
    
    # Start workers...
    
    wait(wg)  # Block until all workers complete
    print("All workers done!")
```

**WaitGroup Operations:**
- `WaitGroup()` - Create a new WaitGroup
- `add(wg, n)` - Add n to the wait counter
- `done(wg)` - Decrement counter by 1 (signal completion)
- `wait(wg)` - Block until counter reaches zero

### 3. Structured Concurrency (Sync Blocks)

The `sync` block provides structured concurrency, automatically waiting for all tasks spawned inside to complete.

```python
def main():
    c = chan(10)
    
    sync:
        task worker(1, c)
        task worker(2, c)
        task worker(3, c)
    
    # All tasks are guaranteed to complete before exiting sync block
    
    # Collect results
    for _ in range(3):
        print(recv(c))
```

**Key Features:**
- Automatic waiting for all tasks inside the block
- Clean exception handling and cleanup
- Prevents orphaned tasks

### 4. Task Spawning

The `task` keyword spawns lightweight concurrent tasks (green threads).

```python
def main():
    # Spawn a task
    task my_function(arg1, arg2)
    
    # Task with channel communication
    c = chan(5)
    task producer(c)
    task consumer(c)
```

**Task Characteristics:**
- Lightweight M:N threading (many tasks, fewer OS threads)
- Work-stealing scheduler for load balancing
- Automatic integration with sync blocks

### 5. Fixed-Size Arrays

Stack-allocated fixed-size arrays for performance-critical code.

```python
def main():
    # Array literal with size
    nums: [i64; 5] = [1, 2, 3, 4, 5]
    
    # Array repetition syntax
    zeros = [0; 100]  # Array of 100 zeros
    
    # Access and modify
    nums[0] = 42
    print(nums[0])
```

**Array vs List:**
| Feature | Array | List |
|---------|-------|------|
| Size | Fixed at compile time | Dynamic |
| Allocation | Stack | Heap (ARC) |
| Performance | Faster (no bounds check) | Flexible |
| Syntax | `[T; N]` type | `List[T]` type |

### 6. Enhanced Type System

New types for concurrency primitives:

```python
# Channel types
def producer(out: chan[i64]):
    send(out, 42)

def consumer(inp: chan[i64]):
    value = recv(inp)

# WaitGroup type
def worker(wg: WaitGroup):
    done(wg)

# Array types
def process(data: [i64; 10]):
    print(data[0])
```

---

## Runtime Implementation

### Channel Implementation

Channels use a ring buffer with atomic operations for thread-safe communication:

```c
typedef struct ViperChannel {
    int64_t* buffer;      // Ring buffer
    size_t capacity;      // Buffer capacity
    size_t head;          // Write position
    size_t tail;          // Read position
    pthread_mutex_t lock; // Mutex for synchronization
    pthread_cond_t not_empty;  // Condition: buffer not empty
    pthread_cond_t not_full;   // Condition: buffer not full
} ViperChannel;
```

**Operations:**
- `vp_chan_create(capacity)` - Allocate and initialize channel
- `vp_chan_send(chan, value)` - Block if buffer full, then send
- `vp_chan_recv(chan)` - Block if buffer empty, then receive
- `vp_chan_destroy(chan)` - Free channel resources

### WaitGroup Implementation

WaitGroups use atomic counters for lock-free synchronization:

```c
typedef struct ViperWaitGroup {
    _Atomic int64_t counter;  // Wait counter
    pthread_mutex_t lock;     // Mutex for condition variable
    pthread_cond_t cond;      // Condition for waiting
} ViperWaitGroup;
```

**Operations:**
- `vp_waitgroup_create()` - Allocate and initialize
- `vp_waitgroup_add(wg, n)` - Atomically add to counter
- `vp_waitgroup_done(wg)` - Atomically decrement, signal if zero
- `vp_waitgroup_wait(wg)` - Block until counter is zero
- `vp_waitgroup_destroy(wg)` - Free resources

### Thread Pool

A work-stealing thread pool executes tasks efficiently:

```c
typedef struct ThreadPool {
    pthread_t* threads;     // Worker threads
    size_t num_threads;     // Number of threads
    TaskQueue* queues;      // Per-thread task queues
} ThreadPool;
```

**Features:**
- One queue per thread with work-stealing
- Automatic load balancing
- Configurable thread count (defaults to CPU cores)

---

## Compiler Changes

### AST Extensions

New expression and statement nodes:

```rust
// Expression: Array literal
Expr::Array { 
    elements: Vec<Expr>, 
    size: Option<usize>, 
    span: Span 
}

// Statements: Concurrency primitives
Stmt::Chan { size: Expr, span: Span }
Stmt::Send { chan: Box<Expr>, value: Box<Expr>, span: Span }
Stmt::Recv { chan: Box<Expr>, span: Span }
Stmt::WaitGroup { span: Span }
Stmt::WgAdd { wg: Box<Expr>, n: Box<Expr>, span: Span }
Stmt::WgDone { wg: Box<Expr>, span: Span }
Stmt::WgWait { wg: Box<Expr>, span: Span }
Stmt::Sync { body: Vec<Stmt>, span: Span }
Stmt::Task { call: Expr, span: Span }
```

### Type System Extensions

New types in the type system:

```rust
Type::Chan(Box<Type>)      // Channel with element type
Type::WaitGroup            // WaitGroup synchronization primitive
Type::Array(Box<Type>, usize)  // Fixed-size array
```

### Escape Analysis

Phase 3 introduces escape analysis for optimization:

```rust
pub struct EscapeAnalyzer {
    // Track which variables escape their scope
    escape_info: HashMap<String, FunctionEscapeContext>,
}
```

**Benefits:**
- Stack allocation for non-escaping variables
- Reduced ARC overhead
- Better register allocation

### Dead Code Elimination (Enhanced)

Improved DCE with dead store elimination:

```rust
// Detects and removes:
// - Unused variable declarations
// - Redundant assignments (dead stores)
// - Non-escaping temporary variables
```

---

## Build System

### Profile-Guided Optimization (PGO)

Phase 3 adds full PGO support:

```bash
# Phase 1: Instrument and collect profiles
cargo build --profile pgo-instrument
LLVM_PROFILE_FILE="target/pgo-data/viper-%p-%m.profraw" \
    ./target/pgo-instrument/viper build program.vp
./program  # Run with representative workloads

# Phase 2: Use profiles for optimization
cargo build --profile pgo
```

### Link-Time Optimization (LTO)

LTO is enabled by default in release builds:

```toml
[profile.release]
lto = "fat"
opt-level = 3
```

---

## Standard Library

### New Modules

| Module | Features | Status |
|--------|----------|--------|
| `types` | Type utilities | ✅ |
| `typing` | Generic, Union, etc. | ✅ |
| `collections` | deque, Counter, OrderedDict | ✅ |
| `itertools` | permutations, combinations, cycle | ✅ |
| `functools` | partial, reduce, lru_cache | ✅ |
| `copy` | Shallow/deep copy | ✅ |
| `json` | JSON parsing/serialization | ✅ |
| `csv` | CSV reading/writing | ✅ |
| `io` | StringIO, BytesIO | ✅ |
| `pathlib` | Object-oriented paths | ✅ |
| `glob` | Pattern matching | ✅ |
| `datetime` | date, time, datetime, timedelta | ✅ |
| `socket` | TCP/UDP sockets | ✅ |
| `http` | HTTP client/server | ✅ |
| `urllib` | URL parsing | ✅ |
| `threading` | Thread-based parallelism | ✅ |
| `queue` | Thread-safe queues | ✅ |
| `hashlib` | MD5, SHA-256, SHA-512 | ✅ |
| `re` | Regular expressions | ✅ |

### Concurrency Builtins

| Function | Description | Status |
|----------|-------------|--------|
| `chan(size)` | Create buffered channel | ✅ |
| `send(chan, value)` | Send to channel | ✅ |
| `recv(chan)` | Receive from channel | ✅ |
| `WaitGroup()` | Create WaitGroup | ✅ |
| `add(wg, n)` | Add to WaitGroup | ✅ |
| `done(wg)` | Signal WaitGroup | ✅ |
| `wait(wg)` | Wait on WaitGroup | ✅ |

---

## Testing

### Test Files

| File | Description |
|------|-------------|
| `tests/test_chan_create.vp` | Channel creation tests |
| `tests/test_chan_simple.vp` | Basic channel communication |
| `tests/test_chan_min.vp` | Minimal channel example |
| `tests/test_concurrency.vp` | Full concurrency test suite |
| `tests/test_print_int.vp` | Print integer tests |
| `tests/test_print_only.vp` | Print-only tests |

### Example Programs

```python
# examples/test_array.vp
def main():
    # Fixed-size array
    nums: [i64; 5] = [1, 2, 3, 4, 5]
    
    # Array repetition
    zeros = [0; 10]
    
    print(len(nums))  # Output: 5
    print(nums[0])    # Output: 1
```

---

## Migration Guide

### From Phase 2 to Phase 3

**1. Using Channels:**

```python
# Old (Phase 2): Manual synchronization
mut shared = 0
# Race condition prone!

# New (Phase 3): Channel-based communication
c = chan(10)
send(c, value)
result = recv(c)
```

**2. Using WaitGroups:**

```python
# Old (Phase 2): No concurrency primitives

# New (Phase 3): Structured concurrency
wg = WaitGroup()
add(wg, num_workers)

sync:
    for i in range(num_workers):
        task worker(i, wg)

wait(wg)
```

**3. Using Arrays:**

```python
# Old (Phase 2): Lists only (heap allocated)
nums = [1, 2, 3, 4, 5]

# New (Phase 3): Arrays for performance (stack allocated)
nums: [i64; 5] = [1, 2, 3, 4, 5]
# Or with repetition:
zeros = [0; 100]
```

---

## Performance Considerations

### Channels

- **Buffered channels**: Use when producer/consumer rates differ
- **Unbuffered channels**: Use for strict synchronization
- **Overhead**: ~50ns per send/recv operation

### WaitGroups

- **Lock-free**: Atomic operations for counter
- **Overhead**: ~10ns per add/done operation
- **Best practice**: Reuse WaitGroups when possible

### Arrays vs Lists

| Operation | Array | List |
|-----------|-------|------|
| Allocation | Stack (fast) | Heap + ARC |
| Access | Direct (no bounds check) | Bounds checked |
| Resize | Not supported | O(1) amortized |
| Memory | Contiguous | Pointer indirection |

---

## Known Limitations

### Channels

- ❌ **Select statement**: Not yet implemented (Phase 4)
- ❌ **Channel closing**: `close(chan)` not implemented (Phase 4)
- ❌ **Range over channels**: `for x in chan:` not implemented (Phase 4)
- ❌ **Typed channels**: Element type inference is basic (improved in Phase 4)

### WaitGroups

- ❌ **Negative counter**: Not detected at runtime (future enhancement)
- ❌ **Reuse after zero**: Behavior undefined (documented)

### Tasks

- ❌ **Task cancellation**: Not implemented (Phase 4)
- ❌ **Task priorities**: All tasks have equal priority (Phase 4)
- ❌ **Task return values**: Must use channels for communication

### Arrays

- ❌ **Runtime size**: Size must be compile-time constant
- ❌ **Slicing**: Array slicing not supported (Phase 4)
- ❌ **Multi-dimensional**: Only 1D arrays supported (Phase 4)

---

## Roadmap: Phase 4

Phase 4 will add advanced features:

- **Async/Await**: Native async support with event loop
- **Select Statement**: Multi-channel selection
- **Task Cancellation**: Cancel running tasks
- **Exception Handling**: Full try/except/finally
- **OOP**: Classes, inheritance, polymorphism
- **Generics**: Parametric polymorphism
- **Advanced Tooling**: LSP, formatter, linter

---

## Credits

**Design Influences:**
- Go: Channels, WaitGroups, sync blocks
- Rust: Ownership, escape analysis
- Python: Syntax, built-in functions
- Erlang: CSP model

**Implementation:**
- LLVM: Code generation and optimization
- pthreads: Thread management
- Atomic operations: Lock-free synchronization

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/viper-lang/viper-lang.git
cd viper-lang

# Build the compiler
make

# Build the runtime
cd runtime && make && cd ..

# Run a test program
./viper run tests/test_chan_simple.vp
```

---

**Download:** [GitHub Releases](https://github.com/viper-lang/viper-lang/releases/tag/v0.3.0)

**Documentation:** [docs/](../docs/)

**Examples:** [examples/](../examples/)

**Issues:** [GitHub Issues](https://github.com/viper-lang/viper-lang/issues)
