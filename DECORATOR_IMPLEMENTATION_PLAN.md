# Viper Language Decorator Implementation Plan

**Date:** March 12, 2026  
**Version:** 0.5.0  
**Status:** Proposed

---

## Executive Summary

This document outlines the comprehensive plan to implement Python-compatible decorators in the Viper Language compiler. Decorators provide a powerful mechanism for modifying function/class behavior at definition time, enabling patterns like memoization, abstraction, and automatic code generation.

### Current State

| Decorator | Status | Implementation |
|-----------|--------|----------------|
| `@staticmethod` | ✅ Implemented | `src/codegen/oop/classes.rs` |
| `@classmethod` | ✅ Implemented | `src/codegen/oop/classes.rs` |
| `@property` | ✅ Implemented | `src/codegen/oop/classes.rs` |
| `@<name>.setter` | ✅ Implemented | `src/codegen/oop/classes.rs` |
| `@dataclass` | ⚠️ Partial | Basic support in `src/codegen/oop/classes.rs` |
| `@lru_cache` / `@memoize` | ❌ Not Implemented | **This Plan** |
| `@cache` | ❌ Not Implemented | **This Plan** |
| `@abstractmethod` | ❌ Not Implemented | **This Plan** |
| `@final` | ❌ Not Implemented | **This Plan** |
| `@wraps` | ❌ Not Implemented | **This Plan** |
| `@singledispatch` | ❌ Not Implemented | **Future** |
| `@total_ordering` | ❌ Not Implemented | **Future** |

---

## Phase 1: Memoization Decorators (Week 1-2)

### 1.1 `@lru_cache(maxsize=128)`

**Priority:** P0 - High  
**Estimated Impact:** 100-1000x improvement on recursive functions

#### Description
Automatically memoize pure functions with LRU (Least Recently Used) eviction policy.

#### Syntax
```python
@lru_cache(maxsize=128)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
```

#### Implementation Plan

1. **Runtime Cache Structure** (`src/codegen/runtime/memoization.rs`)
   - Hash map-based cache with LRU eviction
   - Thread-safe for concurrent access
   - Support for multiple argument types (tuples as keys)

2. **Codegen Integration** (`src/codegen/core/functions.rs`)
   - Detect `@lru_cache` decorator during function generation
   - Generate cache lookup before function body
   - Generate cache insert before return
   - Create per-function cache global

3. **LLVM IR Generation**
   ```llvm
   ; Cache lookup
   %cached = call i8* @vp_lru_cache_get(%cache_ptr, %args_tuple)
   %found = icmp ne i8* %cached, null
   br i1 %found, label %hit, label %miss

   ; Cache hit: return cached value
   hit:
     %result = bitcast i8* %cached to i64
     ret i64 %result

   ; Cache miss: compute and store
   miss:
     %result = call i64 @fib_body(%n)
     call void @vp_lru_cache_set(%cache_ptr, %args_tuple, %result)
     ret i64 %result
   ```

4. **Runtime Functions**
   - `vp_lru_cache_create(maxsize)` - Create new LRU cache
   - `vp_lru_cache_get(cache, key)` - Lookup key, return NULL if miss
   - `vp_lru_cache_set(cache, key, value)` - Insert/update key
   - `vp_lru_cache_destroy(cache)` - Free cache memory

#### Files to Create
- `src/codegen/runtime/memoization.rs` (new)
- `runtime/src/memoization.c` (C runtime implementation)
- `runtime/src/memoization.h` (C header)

#### Files to Modify
- `src/codegen/core/context.rs` (add `memoized_functions` field) ✅ Started
- `src/codegen/core/functions.rs` (add memoization wrapper logic)
- `src/codegen/runtime/mod.rs` (add memoization module)
- `src/semantic/validator.rs` (validate memoized functions are pure)

---

### 1.2 `@cache` (Unbounded Memoization)

**Priority:** P0 - High  
**Estimated Impact:** Same as `@lru_cache` but simpler

#### Description
Simplified unbounded memoization (Python 3.9+). Equivalent to `@lru_cache(maxsize=None)`.

#### Syntax
```python
@cache
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
```

#### Implementation
- Same as `@lru_cache` but without eviction logic
- Simpler hash map (no LRU tracking)
- Slightly better performance (no eviction overhead)

#### Runtime Functions
- `vp_cache_create()` - Create unbounded cache
- `vp_cache_get(cache, key)` - Lookup key
- `vp_cache_set(cache, key, value)` - Insert key
- `vp_cache_destroy(cache)` - Free cache

---

## Phase 2: OOP Decorators (Week 3-4)

### 2.1 `@abstractmethod`

**Priority:** P1 - Medium  
**Use Case:** Abstract base classes, interface definitions

#### Description
Mark methods that must be implemented by subclasses.

#### Syntax
```python
from abc import ABC

class Shape(ABC):
    @abstractmethod
    def area(self):
        pass
    
    @abstractmethod
    def perimeter(self):
        pass

class Rectangle(Shape):
    def area(self):
        return self.width * self.height
    
    def perimeter(self):
        return 2 * (self.width + self.height)
```

#### Implementation Plan

1. **Semantic Analysis** (`src/semantic/abstract_check.rs`)
   - Track abstract methods per class
   - Verify concrete subclasses implement all abstract methods
   - Error if abstract class is instantiated directly

2. **Runtime Support**
   - Mark abstract methods with special flag
   - Runtime error if abstract method called directly

3. **Codegen** (`src/codegen/oop/classes.rs`)
   - Add `is_abstract` flag to method metadata
   - Generate runtime check in abstract methods

#### Files to Create
- `src/semantic/abstract_check.rs` (new)
- `runtime/src/abstract.c` (runtime check)

---

### 2.2 `@final`

**Priority:** P2 - Low  
**Use Case:** Prevent method overriding, seal classes

#### Description
Prevent subclasses from overriding methods or inheriting from final classes.

#### Syntax
```python
class Base:
    @final
    def do_something(self):
        pass

@final
class Sealed:
    pass

# Error: cannot override final method
# Error: cannot inherit from final class
```

#### Implementation
- Compile-time check (no runtime overhead)
- Error during type checking if violated

---

### 2.3 `@dataclass` (Complete Implementation)

**Priority:** P1 - Medium  
**Use Case:** Auto-generate boilerplate for data containers

#### Description
Automatically generate `__init__`, `__repr__`, `__eq__`, and other dunder methods.

#### Syntax
```python
@dataclass
class Point:
    x: int
    y: int

# Auto-generated:
# - __init__(self, x: int, y: int)
# - __repr__(self) -> "Point(x=1, y=2)"
# - __eq__(self, other) -> bool
# - __hash__(self) -> int (if frozen=True)
```

#### Options
```python
@dataclass(init=True, repr=True, eq=True, order=False, frozen=False)
class Point:
    x: int
    y: int
```

#### Implementation Plan

1. **Codegen Enhancement** (`src/codegen/oop/classes.rs`)
   - Currently has partial support
   - Add `__eq__` generation
   - Add `__hash__` generation (when frozen)
   - Add `__lt__`, `__le__`, etc. (when order=True)

2. **Field Processing**
   - Collect annotated fields from class body
   - Generate `__init__` parameters from fields
   - Generate field assignments in `__init__`

---

## Phase 3: Utility Decorators (Week 5-6)

### 3.1 `@wraps(func)`

**Priority:** P2 - Low  
**Use Case:** Preserve metadata in wrapper functions

#### Description
Copy function metadata (name, docstring, module) from wrapped function.

#### Syntax
```python
def my_decorator(f):
    @wraps(f)
    def wrapper(*args, **kwargs):
        return f(*args, **kwargs)
    return wrapper
```

#### Implementation
- Mostly compile-time metadata handling
- Copy `__name__`, `__doc__`, `__module__` attributes
- Important for debugging and introspection

---

### 3.2 `@singledispatch`

**Priority:** P3 - Future  
**Use Case:** Function overloading by argument type

#### Description
Dispatch function calls based on the type of the first argument.

#### Syntax
```python
@singledispatch
def process(arg):
    raise NotImplementedError

@process.register(int)
def _(arg):
    return f"Processing int: {arg}"

@process.register(str)
def _(arg):
    return f"Processing string: {arg}"
```

#### Implementation
- Runtime type dispatch table
- Register type-specific handlers
- Fall back to base implementation

---

### 3.3 `@total_ordering`

**Priority:** P3 - Future  
**Use Case:** Auto-generate comparison methods

#### Description
Generate missing comparison methods from `__eq__` and one other (`__lt__`, `__le__`, etc.).

#### Syntax
```python
@total_ordering
class Version:
    def __init__(self, major, minor):
        self.major = major
        self.minor = minor
    
    def __eq__(self, other):
        return (self.major, self.minor) == (other.major, other.minor)
    
    def __lt__(self, other):
        return (self.major, self.minor) < (other.major, other.minor)

# Auto-generated: __le__, __gt__, __ge__
```

---

## Phase 4: Advanced Decorators (Week 7+)

### 4.1 `@contextmanager`

**Priority:** P2 - Medium  
**Use Case:** Create context managers from generators

#### Syntax
```python
@contextmanager
def managed_resource():
    resource = acquire()
    try:
        yield resource
    finally:
        release(resource)

with managed_resource() as r:
    use(r)
```

---

### 4.2 `@asynccontextmanager`

**Priority:** P2 - Medium  
**Use Case:** Async context managers

---

### 4.3 `@coroutine` / `@asyncio.coroutine`

**Priority:** P3 - Future  
**Use Case:** Legacy coroutine support

---

## Decorator Architecture

### AST Representation

```rust
/// Decorator for functions, classes, and methods
#[derive(Debug, Clone)]
pub struct Decorator {
    pub name: String,              // e.g., "lru_cache", "property"
    pub args: Vec<Expr>,           // Positional arguments
    pub keywords: Vec<(String, Expr)>,  // Keyword arguments (maxsize=128)
    pub span: Span,
}
```

### Decorator Categories

| Category | Decorators | Implementation Complexity |
|----------|------------|--------------------------|
| **Built-in** | `@staticmethod`, `@classmethod`, `@property` | Low (done) |
| **Memoization** | `@lru_cache`, `@cache` | Medium |
| **OOP** | `@abstractmethod`, `@final`, `@dataclass` | Medium |
| **Utility** | `@wraps`, `@singledispatch`, `@total_ordering` | High |
| **Async** | `@contextmanager`, `@asynccontextmanager` | High |

### Implementation Strategy

```
┌─────────────────────────────────────────────────────────────┐
│                     Source Code                              │
│                  @lru_cache(maxsize=256)                     │
│                  def fib(n): ...                             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      Parser                                  │
│         Parse decorator into AST::Decorator node             │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Semantic Analysis                           │
│   - Validate decorator is allowed on this construct         │
│   - Check decorator arguments are valid                     │
│   - For @abstractmethod: mark method as abstract            │
│   - For @lru_cache: verify function is pure                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Code Generation                           │
│   - Generate cache infrastructure (@lru_cache)              │
│   - Generate wrapper functions (@wraps)                     │
│   - Generate dispatch tables (@singledispatch)              │
│   - Add metadata flags (@final, @abstractmethod)            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Runtime Support                            │
│   - Cache operations (get, set, evict)                      │
│   - Dispatch table lookup                                   │
│   - Abstract method checks                                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Runtime Cache Design

### LRU Cache Structure

```c
typedef struct LRUCacheNode {
    void* key;
    void* value;
    struct LRUCacheNode* prev;
    struct LRUCacheNode* next;
} LRUCacheNode;

typedef struct LRUCache {
    size_t maxsize;
    size_t currsize;
    LRUCacheNode* head;  // Most recently used
    LRUCacheNode* tail;  // Least recently used
    HashMap* map;        // key -> node
} LRUCache;
```

### Operations

| Operation | Complexity | Description |
|-----------|------------|-------------|
| `get(key)` | O(1) | Return value, move node to head |
| `set(key, value)` | O(1) | Insert/update, evict tail if full |
| `evict()` | O(1) | Remove tail node |
| `clear()` | O(n) | Free all nodes |

---

## Testing Strategy

### Unit Tests

```python
# tests/decorators/test_lru_cache.vp
@lru_cache(maxsize=128)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def test_fib_memoized():
    assert fib(35) == 9227465
    # Should be instant due to memoization

@cache
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test_factorial_cached():
    assert factorial(100) == 3628800...  # Large number
```

### Integration Tests

```python
# tests/decorators/test_dataclass.vp
@dataclass
class Point:
    x: int
    y: int

def test_dataclass_init():
    p = Point(1, 2)
    assert p.x == 1
    assert p.y == 2

def test_dataclass_repr():
    p = Point(1, 2)
    assert repr(p) == "Point(x=1, y=2)"

def test_dataclass_eq():
    assert Point(1, 2) == Point(1, 2)
    assert Point(1, 2) != Point(2, 1)
```

### Performance Benchmarks

```python
# benchmarks/viper/08_memoization.vp
import time

@lru_cache(maxsize=None)
def fib_memo(n):
    if n <= 1:
        return n
    return fib_memo(n - 1) + fib_memo(n - 2)

def fib_recursive(n):
    if n <= 1:
        return n
    return fib_recursive(n - 1) + fib_recursive(n - 2)

def benchmark():
    start = time.time()
    fib_memo(35)
    memo_time = time.time() - start
    
    start = time.time()
    fib_recursive(35)
    recursive_time = time.time() - start
    
    print(f"Memoized: {memo_time}s")
    print(f"Recursive: {recursive_time}s")
    print(f"Speedup: {recursive_time / memo_time}x")
```

**Expected Results:**

| Benchmark | Without Decorator | With `@lru_cache` | Speedup |
|-----------|-------------------|-------------------|---------|
| `fib(35)` | ~5-10s | <1ms | >5000x |
| `fib(50)` | Timeout | <10ms | ∞ |
| `factorial(1000)` | ~50ms | <5ms | 10x |

---

## Implementation Timeline

| Week | Phase | Tasks |
|------|-------|-------|
| 1-2 | Phase 1 | `@lru_cache`, `@cache` |
| 3-4 | Phase 2 | `@abstractmethod`, `@final`, `@dataclass` |
| 5-6 | Phase 3 | `@wraps`, `@singledispatch`, `@total_ordering` |
| 7+ | Phase 4 | `@contextmanager`, `@asynccontextmanager` |

---

## Success Metrics

### Performance Goals

| Benchmark | Target |
|-----------|--------|
| `fib(35)` with `@lru_cache` | <1ms |
| `fib(50)` with `@cache` | <10ms |
| Memoization overhead (cache miss) | <100ns |
| Memoization overhead (cache hit) | <10ns |

### Code Quality Goals

- Zero regressions on existing tests (434+ tests)
- Full test coverage for all decorators
- Documentation for all decorator features
- Examples in `examples/decorators/`

---

## Risk Mitigation

### Low-Risk Decorators
- `@cache` - Simple hash map, no eviction
- `@final` - Compile-time check only
- `@wraps` - Metadata copy, no runtime impact

### Medium-Risk Decorators
- `@lru_cache` - LRU eviction logic needs testing
- `@dataclass` - Must handle edge cases (defaults, inheritance)
- `@abstractmethod` - Runtime checks add overhead

### High-Risk Decorators
- `@singledispatch` - Complex dispatch logic
- `@total_ordering` - Must handle all comparison combinations

### Mitigation Strategies
1. **Incremental Testing**: Test each decorator independently
2. **Performance Profiling**: Measure overhead of each decorator
3. **Documentation**: Clear examples and limitations
4. **Fallback Path**: Allow disabling decorators at compile time

---

## Appendix: Decorator Reference

### Built-in Decorators (No Import Required)

```python
@staticmethod
@classmethod
@property
@<name>.setter
@<name>.deleter
```

### Standard Library Decorators (Import Required)

```python
from functools import lru_cache, cache, wraps, singledispatch, total_ordering
from dataclasses import dataclass
from abc import ABC, abstractmethod
from typing import final
from contextlib import contextmanager, asynccontextmanager
```

### Viper-Specific Decorators (Future)

```python
@inline          # Force function inlining
@noinline        # Prevent inlining
@cold            # Mark cold function (optimization hint)
@hot             # Mark hot function (optimization hint)
@pure            # Mark pure function (optimization hint)
@thread_safe     # Mark thread-safe function
@unsafe          # Mark unsafe function (bypass checks)
```

---

## References

- Python Decorator Documentation: https://docs.python.org/3/glossary.html#term-decorator
- functools module: https://docs.python.org/3/library/functools.html
- dataclasses module: https://docs.python.org/3/library/dataclasses.html
- abc module: https://docs.python.org/3/library/abc.html
- PEP 318 - Decorators for Functions and Methods
- PEP 3119 - Introducing Abstract Base Classes
- PEP 557 - Data Classes
- PEP 591 - Final Qualifier
- Viper OPTIMIZATION_PLAN.md
- Viper KNOWN_ISSUES_FIX_PLAN.md

---

*Last Updated: March 12, 2026*  
*Version: 0.5.0*  
*Author: Viper Language Team*
