# Viper Language - Phase 2 Release

**Version:** 0.2.0  
**Release Date:** February 24, 2026  
**Status:** Alpha

---

## Overview

Phase 2 builds upon the Phase 1 foundation by adding **memory management** and **data structures**. The key addition is Automatic Reference Counting (ARC) for safe memory management, along with dynamic lists (arrays) and improved type handling.

### What's New in Phase 2

> Memory management meets Python-like simplicity with ARC.

---

## New Features

### 1. Automatic Reference Counting (ARC)

Viper now uses ARC for automatic memory management. Objects are automatically freed when no longer referenced.

```python
def main():
    # Objects are automatically managed
    data = [1, 2, 3, 4, 5]
    # No manual memory management needed!
```

**Runtime Implementation:**
- Thread-safe reference counting using atomic operations
- Header-based metadata prepended to all heap objects
- Automatic destructor calls when reference count reaches zero

### 2. Dynamic Lists

Lists are now a first-class data structure with full support:

```python
def main():
    # List literals
    nums = [1, 2, 3, 4, 5]
    
    # Empty list
    empty = []
    
    # Nested lists
    matrix = [[1, 2], [3, 4]]
```

**Supported Operations:**
- `append(value)` - Add element to end
- `insert(index, value)` - Insert at position
- `remove(index)` - Remove and return element
- `pop()` - Remove and return last element
- `clear()` - Remove all elements
- `len(list)` - Get length
- `list[i]` - Index access
- `list[i] = v` - Index assignment

### 3. Mutable Variables

Explicit mutability with the `mut` keyword:

```python
def main():
    # Immutable by default
    x = 10
    # x = 20  # Error: cannot reassign immutable variable
    
    # Mutable variable
    mut counter = 0
    counter = counter + 1  # OK
```

### 4. None Type

First-class support for null-like values:

```python
def main():
    nothing = None
    
    if nothing == None:
        print("It's nothing!")
```

### 5. Tuple Literals

Basic tuple support:

```python
def main():
    # Tuple with trailing comma syntax
    point = (1, 2)
    single = (42,)
```

### 6. Improved Type System

Enhanced type inference and checking:
- List types: `[T]`
- Dictionary types: `{K: V}`
- Tuple types: `(T1, T2, ...)`
- Better type error messages

---

## Runtime Architecture

### Memory Layout

```
+------------------+------------------+
| ViperHeader      | Object Data      |
| - ref_count      | - actual data    |
| - destructor     |                  |
+------------------+------------------+
       ^                    ^
       |                    |
   internal             user-facing
   pointer              pointer
```

### ARC API

```c
// Allocate with ARC header
void* vp_arc_alloc(size_t size);

// Reference counting
void vp_arc_retain(void* ptr);
void vp_arc_release(void* ptr);
int64_t vp_arc_ref_count(void* ptr);

// Destructor callback
void vp_arc_set_destructor(void* ptr, void (*destructor)(void*));
```

### List API

```c
// Creation/destruction
ViperList* vp_list_create(void);
void vp_list_free(ViperList* list);

// Modification
void vp_list_append(ViperList* list, int64_t value);
void vp_list_insert(ViperList* list, int64_t index, int64_t value);
int64_t vp_list_remove(ViperList* list, int64_t index);

// Access
int64_t vp_list_get(ViperList* list, int64_t index);
void vp_list_set(ViperList* list, int64_t index, int64_t value);
int64_t vp_list_len(ViperList* list);
```

---

## Semantic Analysis

Phase 2 introduces a proper semantic analysis phase:

### Symbol Table

- Scope tracking with nested scopes
- Built-in function registration
- Variable/function/parameter distinction
- Mutability tracking

### Type Checker

- Type inference for literals
- Type compatibility checking
- Expression type tracking
- Comprehensive error reporting

```
Source → Lexer → Parser → Semantic Analysis → CodeGen → LLVM IR
                              ↑
                    New in Phase 2
```

---

## Example Programs

### List Operations

```python
# test_lists.vp
def sum_list(nums: [i64]) -> i64:
    total = 0
    i = 0
    while i < len(nums):
        total = total + nums[i]
        i = i + 1
    return total

def main():
    nums = [1, 2, 3, 4, 5]
    result = sum_list(nums)
    print(result)  # Output: 15
```

### Mutable Variables

```python
# test_mut.vp
def main():
    mut counter = 0
    
    while counter < 10:
        print(counter)
        counter = counter + 1
    
    print("Done!")
```

### None Handling

```python
# test_none.vp
def maybe_return(flag: bool) -> None:
    if flag:
        return None
    print("Still here")

def main():
    maybe_return(True)
    maybe_return(False)
```

---

## Installation

### Build from Source

```bash
# Clone and build
git clone https://github.com/viper-lang/viper.git
cd viper-lang

# Build runtime library
cd runtime && make && cd ..

# Build compiler
cargo build --release

# Install (optional)
cargo install --path .
```

### Dependencies

- **Rust** (latest stable)
- **LLVM 20**
- **GCC/Clang** (for runtime compilation)

---

## Usage

### Compile with Runtime Linking

```bash
# Build runtime first
make -C runtime

# Compile Viper program
viper build source.vp -o output

# Link with runtime
llc output.bc -filetype=obj -o output.o
gcc output.o -o output -L./runtime -lviper

# Run
./output
```

### Run with JIT

```bash
viper run source.vp
```

---

## Known Limitations

### Phase 2 Scope

The following are **not** included:

- ❌ **Dictionary Implementation** - Type exists but runtime not complete
- ❌ **String Lists** - Lists only support i64 in Phase 2
- ❌ **Generics** - Type parameters not yet implemented
- ❌ **Cycle Detection** - ARC cannot handle reference cycles
- ❌ **Weak References** - Coming in Phase 3
- ❌ **Concurrency** - Coming in Phase 3
- ❌ **OOP** - Coming in Phase 4

### Technical Limitations

- Lists only store i64 values (no heterogeneous lists)
- No list slicing syntax
- No list comprehensions
- No iterator protocol
- Type inference is basic (no Hindley-Milner)

---

## Testing

### Run Test Suite

```bash
# Phase 1 tests (should still pass)
viper run test_factorial.vp
viper run test_fibonacci.vp

# Phase 2 tests
viper run test_list.vp
viper run test_mut.vp
```

### Success Criteria

✅ List literals parse and generate correct IR  
✅ `mut` keyword enforces mutability  
✅ `None` is a valid expression  
✅ ARC runtime compiles without errors  
✅ Symbol table tracks scopes correctly  
✅ Type checker catches type errors  

---

## Project Structure Changes

```
viper-lang/
├── src/
│   ├── semantic/           # NEW: Semantic analysis
│   │   ├── mod.rs
│   │   ├── symbol_table.rs
│   │   └── type_checker.rs
│   └── ...
├── runtime/                # NEW: C runtime library
│   ├── include/
│   │   ├── viper_stdlib.h
│   │   ├── viper_types.h
│   │   └── viper_arc.h
│   ├── src/
│   │   ├── runtime.c
│   │   ├── memory/
│   │   │   └── arc.c
│   │   └── data_structures/
│   │       └── list.c
│   └── Makefile
└── ...
```

---

## Migration from Phase 1

Phase 2 is backward compatible with Phase 1 code. All Phase 1 programs should compile and run without changes.

### New Syntax

```python
# Phase 1 code (still valid)
x = 42

# Phase 2 additions
mut counter = 0      # Explicit mutability
nothing = None       # None value
nums = [1, 2, 3]     # List literals
point = (1, 2)       # Tuple literals
```

---

## Roadmap

### Phase 3: Concurrency (Next)
- [ ] M:N threading with work-stealing
- [ ] `sync`/`task` primitives
- [ ] Channels (`chan`)
- [ ] Weak references for ARC

### Phase 4: OOP
- [ ] Classes and inheritance
- [ ] Virtual method tables
- [ ] Exception handling

### Phase 5: Ecosystem
- [ ] Package manager (`vpm`)
- [ ] Language server (`viper-lsp`)
- [ ] Standard library expansion

---

## Performance Notes

### ARC Overhead

- Reference count operations use atomic instructions
- Typical overhead: 5-10% for reference-heavy code
- Minimal overhead for short-lived objects

### List Performance

- Amortized O(1) append
- O(n) insert/remove at arbitrary position
- O(1) index access
- Growth factor: 2x (doubling strategy)

---

## Contributing

### Areas Needing Help

1. **Dictionary Implementation** - Complete the dict runtime
2. **Heterogeneous Lists** - Support mixed-type lists
3. **Cycle Detection** - Implement backup GC for cycles
4. **String Lists** - Add string element support

### Reporting Issues

Found a bug? Please include:
- Viper version (`viper --version`)
- Minimal reproduction case
- Expected vs actual behavior

---

## License

MIT License - See [LICENSE](../LICENSE) for details.

---

**Happy Coding! 🐍**
