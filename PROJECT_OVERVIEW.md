# Viper Language - Project Overview

**Version:** 0.2.3  
**Status:** Phase 1 Complete

Create Viper 2.0, a compiled programming language with Python-like syntax and C-level performance. The compiler uses Rust for the frontend/LLVM integration and C for the runtime. Target: native binaries via LLVM.

## Architecture Summary

| Layer | Technology | Purpose |
|-------|------------|---------|
| Frontend | Rust | Lexer, Parser, Semantic Analysis |
| Middle-end | LLVM (via Inkwell) | Optimization, IR generation |
| Backend | C + LLVM | Runtime (ARC, threading, I/O) |
| Output | Native binary | `.exe`/`.elf`/`.macho` |

---

## Phase 1: Core Compiler (MVP) ✅

Implement the foundation. No concurrency, no OOP yet—just working compiler for basic programs.

### 1.1 Project Structure

```
viper-lang/
├── Cargo.toml
├── build.rs
├── Makefile
├── src/
│   ├── main.rs              # CLI entry
│   ├── cli/
│   │   └── mod.rs           # clap-based args: viper build <file>
│   ├── lexer/
│   │   ├── mod.rs           # Token enum, Lexer struct
│   │   ├── tokens.rs        # Token::Ident, Int, Float, etc.
│   │   └── scanner.rs       # Indentation-aware tokenization
│   ├── parser/
│   │   ├── mod.rs           # Parser struct
│   │   ├── recursive_descent.rs
│   │   └── expressions.rs   # Pratt parser for precedence
│   ├── ast/
│   │   └── mod.rs           # Expr, Stmt enums with Debug
│   ├── codegen/
│   │   ├── mod.rs           # CodeGen struct with LLVM context
│   │   ├── context.rs       # Inkwell setup
│   │   └── builder.rs       # IR generation methods
│   └── utils/
│       └── span.rs          # Source location tracking
├── runtime/
│   ├── viper_stdlib.h
│   └── runtime.c            # malloc/free, printf bridge
└── std/
    └── prelude.vp           # print(), range() builtins
```

### 1.2 Language Features (Phase 1)

**Types:** `i64`, `f64`, `bool`, `str` (basic)

**Statements:**

```python
# Variable declaration (type inferred)
x = 42
y = 3.14
name = "viper"

# Function definition
def add(a: i64, b: i64) -> i64:
    return a + b

# If/else
if x > 0:
    print("positive")
else:
    print("non-positive")

# While loop
while x > 0:
    x = x - 1

# For loop (range only)
for i in range(10):
    print(i)
```

**Expressions:** Arithmetic, comparison, logical operators, function calls

### 1.3 Key Implementation Details

**Lexer Requirements:**
- Handle Python-style indentation (emit `Indent`/`Dedent` tokens)
- Support strings with escape sequences
- Track source locations for error reporting

**Parser Requirements:**
- Recursive descent with Pratt parsing for expressions
- Proper operator precedence: `*` before `+`, etc.
- Build AST with `Box<Expr>` for recursive structures

**CodeGen Requirements:**
- Use Inkwell crate for LLVM bindings
- Generate `define i64 @main()` entry point
- Link with C runtime for `printf`

**Runtime Requirements:**
- `void vp_print_i64(int64_t val)`
- `void vp_print_f64(double val)`
- `void vp_print_str(const char* val)`

### 1.4 Build System

**Cargo.toml:**

```toml
[package]
name = "viper-lang"
version = "0.2.3"
edition = "2021"

[dependencies]
inkwell = { git = "https://github.com/TheDan64/inkwell", branch = "master", features = ["llvm20-1"] }
clap = { version = "4.4", features = ["derive"] }
```

**Makefile:**

```makefile
all: runtime compiler

runtime:
	gcc -c -O3 runtime/runtime.c -o runtime.o
	ar rcs libviper.a runtime.o

compiler:
	cargo build --release

clean:
	rm -f *.o *.a
	cargo clean
```

### 1.5 Success Criteria

This program must compile and run:

```python
# test.vp
def factorial(n: i64) -> i64:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    result = factorial(5)
    print(result)  # Output: 120
```

Command: `./viper build test.vp -o test && ./test`

---

## Phase 2: Data Structures + Memory Management 🚧

Add ARC-based memory management and core data structures.

### 2.1 New Components

```
src/
├── semantic/
│   ├── mod.rs
│   ├── symbol_table.rs      # Scope tracking
│   └── type_checker.rs      # Basic inference
└── codegen/
    └── memory.rs            # ARC insertion, alloca/store/load

runtime/
└── src/
    ├── memory/
    │   ├── arc.c            # Atomic ref counting
    │   └── allocator.c
    └── data_structures/
        ├── list.c           # Dynamic array
        └── string.c         # RC strings
```

### 2.2 New Language Features

```python
# Lists with type inference
nums = [1, 2, 3, 4, 5]
nums.append(6)
first = nums[0]  # Indexing

# String operations
msg = "Hello, " + "World!"  # Concatenation
print(msg[0:5])              # Slicing (zero-copy)

# Mutable variables (explicit)
mut counter = 0
counter = counter + 1
```

### 2.3 ARC Implementation

**Runtime structure:**

```c
typedef struct {
    _Atomic int64_t ref_count;
} ViperHeader;

#define GET_HEADER(ptr) ((ViperHeader*)ptr - 1)

void vp_retain(void* ptr);
void vp_release(void* ptr, void (*destructor)(void*));
```

**Compiler inserts:**
- `vp_retain` on assignment (shared ownership)
- `vp_release` at end of scope (unless returned)

---

## Phase 3: Concurrency (M:N Threading)

Add the concurrency system that makes Viper unique.

### 3.1 New Components

```
src/
└── codegen/
    └── concurrency.rs       # sync, task, chan codegen

runtime/
└── src/
    └── concurrency/
        ├── thread_pool.c    # Work-stealing scheduler
        ├── task_queue.c     # Chase-Lev deque
        ├── channel.c        # Buffered channels
        └── wait_group.c     # sync block support
```

### 3.2 New Language Features

```python
# Structured concurrency
def worker(id: i64, out_chan):
    result = id * id
    send(out_chan, result)

def main():
    c = chan(10)      # Channel with buffer 10
    wg = WaitGroup()
    add(wg, 3)

    sync:             # Wait for all tasks inside
        task worker(1, c)
        task worker(2, c)
        task worker(3, c)

    # Collect results
    for _ in range(3):
        print(recv(c))
```

### 3.3 Runtime Requirements

- Fixed thread pool (N = CPU cores)
- Task queue per thread with work-stealing
- `pthread_mutex_t` + `pthread_cond_t` for channels
- Atomic operations for wait groups

---

## Phase 4: OOP + Advanced Features

Add classes, inheritance, generics, exceptions.

### 4.1 New Components

```
src/
├── parser/
│   └── classes.rs           # Class/method parsing
├── ast/
│   └── oop.rs               # ClassDef, MethodCall nodes
├── codegen/
│   └── classes.rs           # VTable generation
└── semantic/
    └── mro.rs               # C3 linearization

runtime/
└── src/
    └── oop/
        ├── vtable.c         # Virtual method tables
        └── object.c         # Object layout
```

### 4.2 New Language Features

```python
# Classes with inheritance
class Animal:
    def __init__(self, name: str):
        self.name = name

    def speak(self):
        pass

class Dog(Animal):
    def speak(self):
        print(f"{self.name} says: Woof!")

# Generics
class Stack[T]:
    def __init__(self):
        self.items = []

    def push(self, item: T):
        self.items.append(item)

    def pop(self) -> T:
        return self.items.pop()

# Exceptions
try:
    risky_operation()
except ValueError as e:
    print(f"Error: {e}")
finally:
    cleanup()
```

---

## Phase 5: Ecosystem Tools

### 5.1 Package Manager (vpm)

```
vpm/
├── Cargo.toml
└── src/
    ├── main.rs              # vpm CLI
    ├── commands/
    │   ├── init.rs          # vpm init
    │   ├── add.rs           # vpm add <repo>
    │   └── build.rs         # vpm build
    └── resolver/
        └── mod.rs           # Dependency resolution
```

**viper.toml format:**

```toml
[package]
name = "my_project"
version = "0.1.0"

[dependencies]
http = "github.com/viper-lang/http@v1.2.0"
```

### 5.2 LSP Server (viper-lsp)

**Basic handlers:**
- `textDocument/completion` - Autocomplete
- `textDocument/hover` - Type info
- `textDocument/definition` - Go-to-def
- `textDocument/diagnostic` - Real-time errors

---

## Coding Standards

| Area | Standard |
|------|----------|
| Rust | Use `?` operator for error propagation, avoid `unsafe` except for LLVM |
| C Runtime | All public functions prefixed with `vp_`, use `_Atomic` for thread-safety |
| Error Messages | Include file, line, column, and helpful suggestion |
| Tests | Unit tests in `#[cfg(test)]` modules, integration tests in `tests/` |

---

## Deliverables Per Phase

| Phase | Deliverable | Test |
|-------|-------------|------|
| 1 | Working compiler for basic programs | `factorial(20)` computes correctly |
| 2 | ARC working, no memory leaks | Valgrind shows no leaks on list operations |
| 3 | 1000 tasks complete correctly | `sync` block with 1000 tasks finishes |
| 4 | Class inheritance works | Dog calls overridden `speak()` |
| 5 | vpm installs and builds deps | `vpm add` + `vpm build` works |

---

## Reference Implementation Guidance

When stuck, refer to these existing projects:

| Project | Reference For |
|---------|---------------|
| Inkwell examples | LLVM IR generation patterns |
| Rustc lexer | Indentation handling (rust-lang/rust) |
| Go runtime | M:N scheduler design |
| Swift ARC | Reference counting optimization techniques |

> **Begin with Phase 1. Do not proceed to Phase 2 until Phase 1 tests pass.**
