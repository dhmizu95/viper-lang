# @lru_cache and Auto-Memoize Limitations

**Date:** March 12, 2026  
**Version:** 0.5.0

---

## Summary

The `@lru_cache` decorator and `--auto-memoize` flag are **production-ready for i64 return values**, but have several limitations for advanced use cases.

---

## ✅ What Works

| Feature | Status | Example |
|---------|--------|---------|
| i64 return values | ✅ Full support | `fib(45)` cached |
| Multi-parameter (1-2) | ✅ Full support | `gcd(a, b)` cached |
| Explicit `@lru_cache` | ✅ Full support | `@lru_cache def f():` |
| Auto-memoize flag | ✅ Full support | `--auto-memoize` |
| Cache hits | ✅ Working | Repeated calls instant |
| LRU eviction | ✅ Working | `maxsize=128` |
| Unbounded cache | ✅ Working | `@cache` or `maxsize=None` |

---

## ❌ Current Limitations

### 1. BigInt Return Values (Partially Fixed)

**Status:** ✅ Auto-detection implemented, pointer caching needs work

**What Works:**
```python
@lru_cache(maxsize=None)
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

factorial(25)  # ✅ Cached and fast
factorial(25)  # ✅ Cache hit - instant!
```

**What Doesn't Work:**
```python
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

fib(75)  # ❌ Still slow - BigInt pointer not properly cached
```

**Root Cause:**
- ✅ BigInt auto-detection now works (analyzes return statements)
- ✅ `is_bigint` flag properly set for BigInt-returning functions
- ❌ BigInt pointer storage/retrieval from cache has issues

**Progress:**
- Auto-detection: ✅ Complete (March 12, 2026)
- Pointer caching: ⏳ In progress

**Workaround:**
```python
# Use iterative for very large BigInt values
def fib_iter(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
```

---

### 2. Three or More Parameters (Low Priority)

**Issue:** Only 1-2 parameters supported.

**Symptom:**
```python
@lru_cache(maxsize=128)
def complex_func(a, b, c):  # ❌ Error!
    return a + b + c
```

**Error:**
```
Error: Memoization supports 1-2 parameters, got 3
```

**Root Cause:**
- Only `vp_tuple_create1` and `vp_tuple_create2` implemented
- No `vp_tuple_create3+` for more parameters

**Workaround:**
```python
# Combine parameters into a single value
@lru_cache(maxsize=128)
def complex_func_packed(abc):
    a = abc // 10000
    b = (abc // 100) % 100
    c = abc % 100
    return a + b + c
```

**Fix Required:**
Add `vp_tuple_create3+` and update codegen.

---

### 3. No Purity Checking (Medium Priority)

**Issue:** Functions with side effects get cached.

**Symptom:**
```python
counter = 0

@lru_cache(maxsize=128)
def bad_func(n):
    global counter
    counter += 1  # Side effect!
    return n * 2

bad_func(5)  # counter = 1
bad_func(5)  # counter = 1 (should be 2!)
```

**Root Cause:**
- No analysis of function body for side effects
- All recursive functions cached with `--auto-memoize`

**Workaround:**
```python
# Don't use @lru_cache for functions with side effects
def bad_func(n):
    global counter
    counter += 1
    return n * 2
```

**Fix Required:**
Implement purity analysis in semantic pass.

---

### 4. No cache_info() / cache_clear() (Medium Priority)

**Issue:** Can't inspect or clear cache.

**Symptom:**
```python
@lru_cache(maxsize=128)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

fib(40)
print(fib.cache_info())  # ❌ Not implemented
fib.cache_clear()        # ❌ Not implemented
```

**Python Equivalent:**
```python
# Python's functools.lru_cache provides:
CacheInfo(hits=33, misses=36, maxsize=128, currsize=36)
```

**Workaround:**
None available.

**Fix Required:**
Implement runtime functions and codegen for cache statistics.

---

### 5. No Thread Safety (Low Priority)

**Issue:** Cache not thread-safe.

**Symptom:**
```python
@lru_cache(maxsize=128)
def shared_func(x):
    return x * 2

# Concurrent calls may corrupt cache
go shared_func(5)
go shared_func(5)  # Race condition!
```

**Root Cause:**
- No mutex/lock in cache operations
- Single-threaded execution assumed

**Workaround:**
Don't use memoization in concurrent code.

**Fix Required:**
Add `pthread_mutex_t` to cache structs.

---

### 6. Memory Leak with @cache (Medium Priority)

**Issue:** Unbounded cache never frees memory.

**Symptom:**
```python
@cache  # Unbounded!
def process(data):
    return heavy_computation(data)

# Memory grows forever with unique inputs
for i in range(1000000):
    process(unique_data[i])  # Cache never evicts
```

**Root Cause:**
- `@cache` has `maxsize=0` (unlimited)
- No GC integration for cached values

**Workaround:**
```python
# Use bounded cache instead
@lru_cache(maxsize=10000)
def process(data):
    return heavy_computation(data)
```

**Fix Required:**
Add GC integration or manual cache clearing.

---

### 7. No Cache Persistence (Low Priority)

**Issue:** Cache lost between program runs.

**Symptom:**
```bash
$ viper run program.vp  # Computes fib(50)
$ viper run program.vp  # Computes fib(50) again (not cached!)
```

**Root Cause:**
- Cache is in-memory only
- No disk persistence

**Workaround:**
None available.

**Fix Required:**
Implement cache serialization/deserialization.

---

### 8. No Opt-Out for Auto-Memoize (Low Priority)

**Issue:** Can't disable auto-memoize for specific functions.

**Symptom:**
```python
# With --auto-memoize, ALL recursive functions cached

def needs_side_effects(n):
    log_call(n)  # Must run every time!
    return n * 2

# No way to disable caching for this function
```

**Workaround:**
Don't use `--auto-memoize`, use explicit `@lru_cache` instead.

**Fix Required:**
Add `@nomemo` decorator.

---

## Limitation Summary Table

| Limitation | Priority | Impact | Workaround |
|------------|----------|--------|------------|
| BigInt returns | Medium | fib(75)+ slow | Use iterative |
| 3+ parameters | Low | Can't cache | Pack params |
| No purity check | Medium | Wrong results | Manual review |
| No cache_info | Medium | Can't debug | N/A |
| No thread safety | Low | Race conditions | Single-thread |
| Memory leak (@cache) | Medium | High memory use | Use @lru_cache |
| No persistence | Low | Recompute on restart | N/A |
| No opt-out | Low | Can't disable | Use explicit |

---

## Recommended Usage

### For i64 Returns (Fully Supported)

```python
# ✅ Recommended: Explicit decorator
@lru_cache(maxsize=256)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

# ✅ Also good: Auto-memoize for testing
# Run with: viper run --auto-memoize program.vp
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)
```

### For BigInt Returns (Limited Support)

```python
# ⚠️ Use iterative for large values
def fib_big(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

# ⚠️ Or accept cache misses for recursive
@lru_cache(maxsize=None)
def fib_recursive(n):
    if n <= 1:
        return n
    return fib_recursive(n - 1) + fib_recursive(n - 2)
# Works for n < 75, slow for n >= 75
```

### For Multi-Parameter (1-2 params only)

```python
# ✅ Works: 1-2 parameters
@lru_cache(maxsize=128)
def gcd(a, b):
    if b == 0:
        return a
    return gcd(b, a % b)

# ❌ Doesn't work: 3+ parameters
# @lru_cache(maxsize=128)
# def func(a, b, c): ...
```

---

## Future Roadmap

### Phase 1 (Completed)
- [x] Basic LRU cache
- [x] Unbounded cache
- [x] Multi-parameter (1-2)
- [x] Auto-memoize flag

### Phase 2 (In Progress)
- [ ] BigInt auto-detection
- [ ] `cache_info()` implementation
- [ ] `cache_clear()` implementation

### Phase 3 (Future)
- [ ] Thread safety (mutex)
- [ ] GC integration
- [ ] 3+ parameter support
- [ ] Cache persistence
- [ ] Purity checking
- [ ] `@nomemo` decorator

---

## Conclusion

The `@lru_cache` decorator and `--auto-memoize` flag are **production-ready for i64 return values with 1-2 parameters**. 

For BigInt values, 3+ parameters, or advanced features, use workarounds or wait for future enhancements.

---

*Last Updated: March 12, 2026*  
*Author: Viper Language Team*
