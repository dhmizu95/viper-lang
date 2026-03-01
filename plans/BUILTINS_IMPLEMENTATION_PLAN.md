# Python Builtins Implementation Plan

## Overview

This plan outlines the implementation of missing Python built-in functions in Viper. Currently **28 builtins** are implemented (including BigInt-specific and concurrency functions). This plan adds **~40 more** to reach comprehensive Python compatibility.

## Priority Tiers

### Tier 1: High Priority (Essential for usability)
- Collection constructors: `list()`, `dict()`, `tuple()`, `set()`
- Iteration helpers: `enumerate()`, `zip()`
- Numeric utilities: `min()`, `max()`, `sum()`
- Introspection: `type()`, `repr()`

### Tier 2: Medium Priority (Common Python patterns)
- Functional: `map()`, `filter()`, `any()`, `all()`
- Attribute access: `getattr()`, `setattr()`, `hasattr()`
- Conversion: `bin()`, `oct()`, `hex()`, `chr()`, `ord()`

### Tier 3: Lower Priority (Specialized use cases)
- Advanced introspection: `dir()`, `globals()`, `locals()`, `vars()`
- I/O: `open()`, `input()`
- Metaprogramming: `callable()`, `issubclass()`, `super()`

---

## Phase 1: Essential Collection Builtins

**Goal:** Enable dynamic collection construction from iterables

### 1.1 `list(iterable)` 
**Location:** `src/codegen/expressions/collections.rs`

```rust
/// Generate list() call - convert iterable to list
pub fn generate_list_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function needed:** `vp_list_from_iterable(*mut void) -> *mut VpList`
- Handle string → list of characters
- Handle existing list → copy
- Handle range → create list

**Tests:**
```python
list("hello")      # ['h', 'e', 'l', 'l', 'o']
list(range(5))     # [0, 1, 2, 3, 4]
list([1, 2, 3])    # [1, 2, 3] (copy)
```

### 1.2 `tuple(iterable)`
**Location:** `src/codegen/expressions/core.rs`

```rust
/// Generate tuple() call - convert iterable to tuple
pub fn generate_tuple_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_tuple_from_iterable(*mut void) -> *mut VpTuple`

### 1.3 `dict(iterable)` 
**Location:** `src/codegen/expressions/collections.rs`

```rust
/// Generate dict() call - create dict from iterable of pairs
pub fn generate_dict_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_dict_from_pairs(*mut void) -> *mut VpDict`
- Accept list of (key, value) tuples
- Accept keyword arguments (future)

**Tests:**
```python
dict([("a", 1), ("b", 2)])  # {"a": 1, "b": 2}
dict()                       # {}
```

### 1.4 `set(iterable)`
**Location:** `src/codegen/expressions/collections.rs`

```rust
/// Generate set() call - create set from iterable
pub fn generate_set_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_set_from_iterable(*mut void) -> *mut VpSet`

---

## Phase 2: Iteration Builtins

### 2.1 `enumerate(iterable, start=0)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
/// Generate enumerate() call - returns iterator of (index, value) tuples
pub fn generate_enumerate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_enumerate_create(*mut void, i64) -> *mut VpEnumerator`

**Tests:**
```python
for i, v in enumerate(["a", "b", "c"]):
    print(i, v)  # 0 a, 1 b, 2 c

for i, v in enumerate(["x", "y"], start=1):
    print(i, v)  # 1 x, 2 y
```

### 2.2 `zip(iter1, iter2, ...)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
/// Generate zip() call - combines multiple iterables
pub fn generate_zip_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_zip_create(*mut void**, i64 count) -> *mut VpZipper`

**Tests:**
```python
for a, b in zip([1, 2, 3], ["a", "b", "c"]):
    print(a, b)  # 1 a, 2 b, 3 c
```

### 2.3 `iter(obj)` and `next(iterator, default?)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_iter_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_next_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_iter_create(*mut void) -> *mut VpIterator`
- `vp_next(*mut VpIterator) -> *mut void`
- `vp_next_default(*mut VpIterator, *mut void) -> *mut void`

---

## Phase 3: Functional Builtins

### 3.1 `map(func, iterable, ...)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
/// Generate map() call - applies function to each element
pub fn generate_map_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_map_create(fn_ptr, *mut void** iters, i64 count) -> *mut VpMapper`

**Tests:**
```python
squares = map(lambda x: x * x, range(5))
list(squares)  # [0, 1, 4, 9, 16]
```

### 3.2 `filter(func, iterable)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
/// Generate filter() call - filters elements by predicate
pub fn generate_filter_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_filter_create(fn_ptr, *mut void) -> *mut VpFilterer`

**Tests:**
```python
evens = filter(lambda x: x % 2 == 0, range(10))
list(evens)  # [0, 2, 4, 6, 8]
```

### 3.3 `sum(iterable, start=0)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
/// Generate sum() call - sums all elements
pub fn generate_sum_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_list_sum(*mut VpList) -> i64` (for int lists)
**Runtime function:** `vp_list_sum_f64(*mut VpList) -> f64` (for float lists)

**Tests:**
```python
sum(range(10))        # 45
sum([1.5, 2.5, 3.0])  # 7.0
sum([[1], [2]], [])   # [1, 2] (concatenation)
```

### 3.4 `any(iterable)` and `all(iterable)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_any_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_all_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_list_any(*mut VpList) -> i1`
- `vp_list_all(*mut VpList) -> i1`

**Tests:**
```python
any([False, False, True])   # True
all([1, 2, 3, 4])           # True
any([0, 0, 0])              # False
all([1, 2, 0, 4])           # False
```

---

## Phase 4: Numeric Builtins

### 4.1 `min(iterable, key?, default?)` and `max(iterable, key?, default?)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_min_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_max_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_list_min(*mut VpList) -> i64`
- `vp_list_max(*mut VpList) -> i64`
- `vp_list_min_f64(*mut VpList) -> f64`
- `vp_list_max_f64(*mut VpList) -> f64`

**Tests:**
```python
min([3, 1, 4, 1, 5])  # 1
max([3, 1, 4, 1, 5])  # 5
min(3, 7, 1, 9)       # 1 (varargs)
```

### 4.2 `round(number, ndigits=0)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_round_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_round_f64(f64, i64) -> f64`

### 4.3 `divmod(a, b)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_divmod_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** Returns tuple (quotient, remainder)

**Tests:**
```python
divmod(17, 5)   # (3, 2)
divmod(17.5, 5) # (3.0, 2.5)
```

### 4.4 `pow(base, exp, mod?)`
**Location:** `src/codegen/expressions/builtins.rs`

Note: `pow_bigint()` already exists for BigInt-specific power

```rust
pub fn generate_pow_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_pow_i64(i64, i64) -> i64`
- `vp_pow_f64(f64, f64) -> f64`
- `vp_pow_mod(i64, i64, i64) -> i64` (modular exponentiation)

---

## Phase 5: Introspection Builtins

### 5.1 `type(obj)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_type_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_type_of(*mut void) -> *mut VpType`

**Tests:**
```python
type(42)      # <class 'int'>
type("hi")    # <class 'str'>
type([1, 2])  # <class 'list'>
```

### 5.2 `id(obj)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_id_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_object_id(*mut void) -> i64` (returns pointer as int)

### 5.3 `repr(obj)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_repr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_repr_i64(i64) -> *mut VpStr`
- `vp_repr_f64(f64) -> *mut VpStr`
- `vp_repr_str(*mut VpStr) -> *mut VpStr` (adds quotes)
- `vp_repr_list(*mut VpList) -> *mut VpStr`

**Tests:**
```python
repr(42)       # "42"
repr("hello")  # "'hello'"
repr([1, 2])   # "[1, 2]"
```

### 5.4 `dir(obj?)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_dir_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_dir(*mut void) -> *mut VpList` (returns list of attribute names)

### 5.5 `globals()`, `locals()`, `vars(obj?)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_globals_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_locals_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_vars_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_globals_create() -> *mut VpDict`
- `vp_locals_create() -> *mut VpDict`

---

## Phase 6: Attribute Builtins

### 6.1 `getattr(obj, name, default?)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_getattr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_getattr(*mut void, *mut VpStr, *mut void default) -> *mut void`

### 6.2 `setattr(obj, name, value)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_setattr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_setattr(*mut void, *mut VpStr, *mut void) -> void`

### 6.3 `hasattr(obj, name)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_hasattr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_hasattr(*mut void, *mut VpStr) -> i1`

### 6.4 `delattr(obj, name)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_delattr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_delattr(*mut void, *mut VpStr) -> void`

---

## Phase 7: I/O Builtins

### 7.1 `open(file, mode='r', ...)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_open_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_fopen(*mut VpStr, *mut VpStr) -> *mut VpFile`

**Tests:**
```python
f = open("test.txt", "r")
content = f.read()
f.close()
```

### 7.2 `input(prompt?)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_input_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_input(*mut VpStr prompt) -> *mut VpStr`

---

## Phase 8: Conversion Builtins

### 8.1 `bin(n)`, `oct(n)`, `hex(n)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_bin_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_oct_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_hex_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_bin_i64(i64) -> *mut VpStr`  # "0b1010"
- `vp_oct_i64(i64) -> *mut VpStr`  # "0o12"
- `vp_hex_i64(i64) -> *mut VpStr`  # "0xa"

### 8.2 `chr(n)` and `ord(s)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_chr_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
pub fn generate_ord_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime functions:**
- `vp_chr_i64(i64) -> *mut VpStr`  # single character string
- `vp_ord_str(*mut VpStr) -> i64`  # Unicode code point

---

## Phase 9: Advanced Builtins

### 9.1 `callable(obj)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_callable_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_is_callable(*mut void) -> i1`

### 9.2 `issubclass(cls, classinfo)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_issubclass_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_is_subclass(*mut VpType, *mut VpType) -> i1`

### 9.3 `super()` and `super(cls, obj)`
**Location:** `src/codegen/expressions/calls.rs`

Note: `super().method()` already partially supported in method calls

```rust
pub fn generate_super_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

---

## Phase 10: Remaining Builtins

### 10.1 `slice(start, stop, step)`
**Location:** `src/codegen/expressions/calls.rs`

```rust
pub fn generate_slice_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime struct:** `VpSlice { start: i64, stop: i64, step: i64 }`

### 10.2 `frozenset(iterable?)`
**Location:** `src/codegen/expressions/collections.rs`

```rust
pub fn generate_frozenset_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_frozenset_create(*mut void) -> *mut VpFrozenSet`

### 10.3 `format(value, format_spec?)`
**Location:** `src/codegen/expressions/builtins.rs`

```rust
pub fn generate_format_call<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String>
```

**Runtime function:** `vp_format_value(*mut void, *mut VpStr) -> *mut VpStr`

---

## Implementation Order (Recommended)

| Week | Phase | Builtins | Priority |
|------|-------|----------|----------|
| 1 | Phase 1 | `list()`, `tuple()`, `dict()`, `set()` | 🔴 Critical |
| 2 | Phase 2 | `enumerate()`, `zip()` | 🔴 Critical |
| 3 | Phase 3 | `map()`, `filter()`, `sum()`, `any()`, `all()` | 🟠 High |
| 4 | Phase 4 | `min()`, `max()`, `round()`, `divmod()`, `pow()` | 🟠 High |
| 5 | Phase 5 | `type()`, `id()`, `repr()` | 🟡 Medium |
| 6 | Phase 6 | `getattr()`, `setattr()`, `hasattr()` | 🟡 Medium |
| 7 | Phase 7 | `open()`, `input()` | 🟡 Medium |
| 8 | Phase 8 | `bin()`, `oct()`, `hex()`, `chr()`, `ord()` | 🟢 Low |
| 9 | Phase 9 | `callable()`, `issubclass()`, `super()` | 🟢 Low |
| 10 | Phase 10 | `slice()`, `frozenset()`, `format()` | 🟢 Low |

---

## File Structure

```
src/codegen/expressions/
├── builtins.rs      # Core builtins (print, len, str, math, etc.)
├── calls.rs         # Function call dispatch + new builtins
├── collections.rs   # list(), dict(), set(), tuple()
├── core.rs          # tuple(), basic expressions
└── iterators.rs     # NEW: enumerate, zip, map, filter (Phase 2-3)

src/codegen/runtime/
├── lists.rs         # vp_list_* functions
├── dicts.rs         # vp_dict_* functions
├── sets.rs          # vp_set_* functions (NEW)
├── tuples.rs        # vp_tuple_* functions (NEW)
├── iterators.rs     # vp_enumerate_*, vp_zip_* (NEW)
├── builtins.rs      # vp_min, vp_max, vp_sum, etc. (NEW)
└── introspect.rs    # vp_type_of, vp_dir, vp_getattr (NEW)
```

---

## Testing Strategy

For each builtin, create tests in:

```
tests/builtins/
├── test_collection_builtins.vp    # list, dict, tuple, set
├── test_iteration_builtins.vp     # enumerate, zip, iter, next
├── test_functional_builtins.vp    # map, filter, sum, any, all
├── test_numeric_builtins.vp       # min, max, round, divmod, pow
├── test_introspection_builtins.vp # type, id, repr, dir
├── test_attribute_builtins.vp     # getattr, setattr, hasattr
├── test_io_builtins.vp            # open, input
├── test_conversion_builtins.vp    # bin, oct, hex, chr, ord
└── test_advanced_builtins.vp      # callable, issubclass, super
```

Run tests with:
```bash
./run_tests.sh tests/builtins/
```

---

## Runtime Dependencies

Many builtins require new C runtime functions. These go in:

```
src/codegen/runtime/
├── lists.c          # Existing list operations
├── dicts.c          # Existing dict operations
├── sets.c           # NEW: set operations
├── tuples.c         # NEW: tuple operations
├── iterators.c      # NEW: enumerate, zip, map, filter
├── builtins.c       # NEW: min, max, sum, any, all
├── introspect.c     # NEW: type, dir, getattr
└── io.c             # NEW: file I/O, input
```

---

## Success Metrics

- [ ] All 68 Python builtins implemented
- [ ] 100% test coverage for each builtin
- [ ] Documentation in `CORE_LANGUAGE_FEATURES.md` updated
- [ ] Examples in `examples/builtins/` directory
- [ ] Performance benchmarks for iteration builtins

---

## Notes

1. **Iterator protocol**: Many builtins return lazy iterators. Consider whether to:
   - Materialize immediately (simpler, more memory)
   - Return iterator objects (more Pythonic, lazy)

2. **Type specialization**: For performance, create specialized versions:
   - `vp_list_sum_i64()` vs `vp_list_sum_f64()`
   - `vp_list_min_i64()` vs `vp_list_min_f64()`

3. **Error handling**: Follow Python semantics:
   - `min([])` → raises ValueError
   - `next(iterator, default)` → returns default if exhausted

4. **BigInt support**: Ensure numeric builtins work with BigInt:
   - `sum()` of BigInts
   - `min()`/`max()` of BigInts
