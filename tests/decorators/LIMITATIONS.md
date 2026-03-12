# @lru_cache Implementation Limitations

**Date:** March 12, 2026  
**Status:** Documenting current limitations

---

## Critical Limitations

### 1. ❌ Cache Wrapper Not Fully Implemented

**Issue:** The `define_memoized_function()` in `src/codegen/core/functions.rs` is a **simplified stub** that doesn't generate the actual cache wrapper.

**Current behavior:**
```rust
// Simplified - just calls normal function generation
pub(crate) fn define_memoized_function(...) {
    // Creates cache global (✅)
    // Declares runtime functions (✅)
    // But doesn't wrap function body with cache lookup/insert (❌)
    
    self.define_function(...)  // Just generates normal function
}
```

**What's missing:**
```llvm
; Should generate:
define i64 @fib(i64 %n) {
  ; Cache lookup
  %cached = call i8* @vp_cache_get(%cache, %n)
  %found = icmp ne i8* %cached, null
  br i1 %found, label %hit, label %miss

hit:
  %result = bitcast i8* %cached to i64
  ret i64 %result

miss:
  %result = call i64 @__fib_body(i64 %n)  ; Call renamed original
  call void @vp_cache_set(%cache, %n, %result)
  ret i64 %result
}

define internal i64 @__fib_body(i64 %n) {
  ; Original function body here
}
```

**Impact:** No actual caching happens at runtime - function executes normally.

**Fix required:** Complete the wrapper codegen in `define_memoized_function()`.

---

### 2. ❌ Single Integer Parameter Only

**Issue:** Cache key generation only supports single `i64` parameter.

**Current code:**
```rust
// Only handles 1 parameter
if params.len() != 1 {
    return Err("Memoization currently only supports single-parameter functions");
}

// Only handles integer params
let key_value = if first_param.is_int_value() {
    let key_tuple = self.builder.build_call(memo_funcs.tuple_create1, &[first_param.into()], "cache_key");
    ...
} else {
    return Err("Memoization only supports integer parameters for now");
}
```

**What doesn't work:**
```python
@lru_cache(maxsize=128)
def add(a, b):          # ❌ Two parameters
    return a + b

@lru_cache(maxsize=128)
def process(data: list):  # ❌ Non-integer parameter
    return len(data)

@lru_cache(maxsize=128)
def greet(name: str):   # ❌ String parameter
    return f"Hello, {name}"
```

**Fix required:** 
- Implement multi-parameter tuple key generation
- Add support for string/list/dict hashing
- Handle type conversions for cache keys

---

### 3. ❌ No Thread Safety

**Issue:** Cache data structures have no mutex/lock protection.

**Current C implementation:**
```c
void vp_cache_set(Cache* cache, void* key, void* value) {
    // No locking!
    hashmap_set(cache->map, hash, key, value);
    cache->currsize++;
}

void* vp_cache_get(Cache* cache, void* key) {
    // No locking!
    return hashmap_get(cache->map, hash, key)->value;
}
```

**Race conditions possible:**
```python
@lru_cache(maxsize=128)
def shared_compute(x):
    return x * 2

# Thread 1: shared_compute(5)
# Thread 2: shared_compute(5)  # May corrupt cache
```

**Fix required:** Add `pthread_mutex_t` to cache structs.

---

### 4. ❌ Memory Leak (Unbounded Cache)

**Issue:** `@cache` (unbounded) never frees cached values.

**Current behavior:**
```python
@cache
def factorial(n):
    if n <= 1: return 1
    return n * factorial(n - 1)

# Every unique call allocates memory that's never freed
for i in range(1000000):
    factorial(i)  # Memory grows forever!
```

**C implementation:**
```c
typedef struct Cache {
    size_t currsize;
    HashMap* map;  // Grows forever with @cache
} Cache;
```

**Fix required:**
- Add `cache_clear()` function
- Implement weak references for GC integration
- Add max size limits even for `@cache`

---

### 5. ❌ No cache_info() or cache_clear() Methods

**Issue:** Python's `functools.lru_cache` provides these:

```python
@lru_cache(maxsize=128)
def fib(n):
    ...

print(fib.cache_info())  # ❌ Not implemented
fib.cache_clear()         # ❌ Not implemented
```

**Python equivalent:**
```
CacheInfo(hits=33, misses=36, maxsize=128, currsize=36)
```

**Fix required:**
- Add `cache_info` function to return stats
- Track hits/misses in cache struct
- Add `cache_clear` function

---

### 6. ❌ No Hash Collision Handling

**Issue:** Hash function is simple and collisions aren't properly handled.

**Current hash:**
```c
uint64_t vp_hash_int(int64_t key) {
    uint64_t hash = (uint64_t)key;
    hash = (hash ^ (hash >> 30)) * 0xbf58476d1ce4e5b9ULL;
    hash = (hash ^ (hash >> 27)) * 0x94d049bb133111ebULL;
    hash = hash ^ (hash >> 31);
    return hash;
}
```

**Issue:** The hashmap uses linear comparison:
```c
if (node->key_hash == hash && memcmp(node->key, key, sizeof(int64_t)) == 0)
```

This only compares `sizeof(int64_t)` bytes - won't work for larger keys.

**Fix required:** Proper key comparison for variable-size keys.

---

### 7. ❌ No Decorator Argument Validation

**Issue:** Invalid decorator arguments aren't caught:

```python
@lru_cache(maxsize=-1)    # ❌ Negative maxsize accepted
@lru_cache(maxsize="128") # ❌ String accepted
@lru_cache(unknown=128)   # ❌ Unknown keyword accepted
@lru_cache                # ❌ Missing parentheses may fail
```

**Fix required:** Add semantic validation for decorator arguments.

---

### 8. ❌ No Mutual Recursion Support

**Issue:** Mutually recursive functions aren't properly handled:

```python
@lru_cache(maxsize=128)
def is_even(n):
    if n == 0: return True
    return is_odd(n - 1)

@lru_cache(maxsize=128)
def is_odd(n):
    if n == 0: return False
    return is_even(n - 1)
```

**Current behavior:** Each function gets its own cache, but the recursion analyzer may not properly detect mutual recursion patterns.

**Fix required:** Enhance recursion analysis for mutual recursion.

---

### 9. ❌ No Cache Persistence

**Issue:** Cache is lost between program runs:

```python
# Run 1: fib(50) computed and cached
# Run 2: fib(50) computed again from scratch
```

**Fix required (future):** 
- Serialize cache to disk
- Load cache on module import

---

### 10. ❌ No Integration with GC/ARC

**Issue:** Cached values aren't tracked by Viper's ARC (Automatic Reference Counting):

```python
@lru_cache(maxsize=128)
def create_list(n):
    return [i for i in range(n)]  # List never freed even if unreferenced
```

**Fix required:** Integrate cache with Viper's reference counting system.

---

## Summary Table

| Limitation | Severity | Workaround | Status |
|------------|----------|------------|--------|
| Cache wrapper not implemented | 🔴 Critical | Use iterative approach | ⏳ Pending |
| Single integer parameter only | 🔴 Critical | Restructure code | ⏳ Pending |
| No thread safety | 🟠 High | Single-threaded only | ⏳ Pending |
| Memory leak (unbounded) | 🟠 High | Use `@lru_cache(N)` | ⏳ Pending |
| No cache_info/clear | 🟡 Medium | Track manually | ⏳ Pending |
| Hash collision issues | 🟡 Medium | Small inputs only | ⏳ Pending |
| No argument validation | 🟡 Medium | Careful usage | ⏳ Pending |
| No mutual recursion | 🟡 Medium | Combine functions | ⏳ Pending |
| No persistence | 🟢 Low | N/A | Future |
| No GC integration | 🟠 High | Avoid caching refs | ⏳ Pending |

---

## Recommended Usage (Until Fixed)

```python
# ✅ DO: Use for single-integer recursive functions
@lru_cache(maxsize=256)
def fib(n):
    if n <= 1: return n
    return fib(n-1) + fib(n-2)

# ✅ DO: Use bounded cache to limit memory
@lru_cache(maxsize=1000)
def factorial(n):
    if n <= 1: return 1
    return n * factorial(n-1)

# ❌ DON'T: Multi-parameter (not yet supported)
@lru_cache(maxsize=128)
def add(a, b):
    return a + b

# ❌ DON'T: Non-integer params (not yet supported)
@lru_cache(maxsize=128)
def greet(name: str):
    return f"Hello, {name}"

# ❌ DON'T: Threaded access (not thread-safe)
go compute(x)  # Concurrent cache access unsafe
```

---

## Implementation Priority

1. **P0 - Complete cache wrapper codegen** (unlock performance)
2. **P0 - Multi-parameter support** (usability)
3. **P1 - Thread safety** (correctness)
4. **P1 - GC/ARC integration** (memory safety)
5. **P2 - cache_info/clear** (debugging)
6. **P3 - Persistence** (nice-to-have)

---

*Last Updated: March 12, 2026*  
*Version: 0.5.0*  
*Author: Viper Language Team*
