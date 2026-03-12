# @lru_cache Fix Plan

**Date:** March 12, 2026  
**Priority:** P0 - Critical Performance Feature  
**Estimated Effort:** 2-3 weeks

---

## Executive Summary

This plan addresses all 10 limitations in the current `@lru_cache` implementation. The work is organized into 4 phases:

| Phase | Focus | Duration | Key Deliverable |
|-------|-------|----------|-----------------|
| **1** | Cache Wrapper Codegen | 3-4 days | Actual caching works |
| **2** | Multi-Parameter Support | 3-4 days | Tuple/dict keys |
| **3** | Safety & Correctness | 4-5 days | Thread-safe, GC-integrated |
| **4** | Polish & Debugging | 2-3 days | cache_info/clear, validation |

---

## Phase 1: Complete Cache Wrapper Codegen (P0)

**Goal:** Make `@lru_cache` actually cache function results at runtime.

### Task 1.1: Function Renaming Infrastructure

**File:** `src/codegen/core/functions.rs`

**Changes:**
```rust
pub(crate) fn define_memoized_function(...) {
    // 1. Rename original function to __func_body
    let body_func_name = format!("__{}_body", original_name);
    
    // 2. Create the "body" function (original implementation)
    let body_func = module.add_function(&body_func_name, fn_type, None);
    
    // 3. Create wrapper function with original name
    let wrapper_func = module.add_function(mangled_name, fn_type, None);
    
    // 4. Generate body_func implementation (original logic)
    self.generate_function_body(body_func, body, params, ...);
    
    // 5. Generate wrapper with cache logic
    self.generate_cache_wrapper(wrapper_func, body_func, cache_global, ...);
}
```

**Estimated:** 4 hours

---

### Task 1.2: Cache Lookup Codegen

**File:** `src/codegen/core/functions.rs`

**Generate LLVM IR:**
```llvm
define i64 @fib(i64 %n) {
entry:
  ; Create cache key from arguments
  %key = call i8* @vp_tuple_create1(i64 %n)
  
  ; Cache lookup
  %cached_ptr = call i8* @vp_cache_get(%cache_global, i8* %key)
  %is_null = icmp eq i8* %cached_ptr, null
  br i1 %is_null, label %miss, label %hit

hit:
  ; Cache hit - return cached value
  %cached_val = bitcast i8* %cached_ptr to i64
  call void @vp_free(i8* %key)  ; Free key after lookup
  ret i64 %cached_val

miss:
  ; Cache miss - call original function
  %result = call i64 @__fib_body(i64 %n)
  
  ; Store in cache
  %result_ptr = bitcast i64 %result to i8*
  call void @vp_cache_set(%cache_global, i8* %key, i8* %result_ptr)
  
  ret i64 %result
}
```

**Implementation:**
```rust
fn generate_cache_wrapper<'ctx>(
    &mut self,
    wrapper_func: FunctionValue<'ctx>,
    body_func: FunctionValue<'ctx>,
    cache_global: PointerValue<'ctx>,
    params: &[Param],
) -> Result<(), String> {
    let entry = self.context.append_basic_block(wrapper_func, "entry");
    let hit_block = self.context.append_basic_block(wrapper_func, "hit");
    let miss_block = self.context.append_basic_block(wrapper_func, "miss");
    
    // Build key from parameters
    let key_value = self.build_cache_key(params, wrapper_func)?;
    
    // Cache lookup call
    let cached = self.builder.build_call(
        memo_funcs.cache_get,
        &[cache_global.into(), key_value.into()],
        "cached_ptr",
    );
    
    // Null check
    let is_null = self.builder.build_int_compare(
        IntPredicate::EQ,
        cached.into_pointer_value().into_int_value(),
        self.context.ptr_type(AddressSpace::default()).const_null().into_int_value(),
        "is_null",
    );
    
    // Conditional branch
    self.builder.build_conditional_branch(is_null, miss_block, hit_block)?;
    
    // Generate hit block
    self.builder.position_at_end(hit_block);
    // ... return cached value
    
    // Generate miss block  
    self.builder.position_at_end(miss_block);
    // ... call body_func and cache result
}
```

**Estimated:** 8 hours

---

### Task 1.3: Return Statement Modification

**File:** `src/codegen/statements/core/dispatch.rs` or `src/codegen/core/functions.rs`

**Issue:** Need to intercept return statements in memoized functions to cache the result.

**Approach A - Wrap at return sites:**
```rust
// In memoized functions, modify return codegen:
Stmt::Return { value, .. } => {
    if self.current_function_is_memoized {
        // Generate cache_store before return
        let result_value = generate_expr(self, value)?;
        self.generate_cache_store(result_value)?;
        self.ir_builder.build_return(&self.builder, Some(&result_value));
    } else {
        // Normal return
    }
}
```

**Approach B - Single exit point (simpler):**
```rust
// In generate_cache_wrapper, after calling body_func:
let result = self.builder.build_call(body_func, &arg_values, "result");

// Cache the result
self.builder.build_call(
    memo_funcs.cache_set,
    &[cache_global.into(), key_value.into(), result.into()],
    "",
);

// Return
self.ir_builder.build_return(&self.builder, Some(&result));
```

**Recommended:** Approach B (simpler, single modification point)

**Estimated:** 4 hours

---

### Task 1.4: Testing Phase 1

**Test file:** `tests/decorators/test_phase1_wrapper.vp`

```python
@lru_cache(maxsize=128)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    # Should be instant with caching
    import time
    start = time.time()
    result = fib(40)
    elapsed = time.time() - start
    
    print(f"fib(40) = {result}")
    print(f"Time: {elapsed:.4f}s")
    
    # Verify speedup
    assert elapsed < 0.1, f"Too slow: {elapsed}s"
    assert result == 102334155
    
    return 0

main()
```

**Expected result:** fib(40) in <0.1s (vs ~10s without caching)

**Estimated:** 2 hours

---

**Phase 1 Total:** ~18 hours (3-4 days)

---

## Phase 2: Multi-Parameter Support (P0)

**Goal:** Support functions with multiple parameters and non-integer types.

### Task 2.1: Tuple Key Generation

**File:** `src/codegen/core/functions.rs`

**Current limitation:**
```rust
if params.len() != 1 {
    return Err("Memoization currently only supports single-parameter functions");
}
```

**Fix:**
```rust
fn build_cache_key<'ctx>(
    &mut self,
    params: &[Param],
    func: FunctionValue<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    match params.len() {
        0 => Err("Cannot cache functions with no parameters".to_string()),
        1 => {
            let param_val = func.get_nth_param(0).unwrap();
            self.build_single_param_key(param_val)
        }
        2 => {
            let p0 = func.get_nth_param(0).unwrap();
            let p1 = func.get_nth_param(1).unwrap();
            self.build_two_param_key(p0, p1)
        }
        n => {
            // For 3+ params, create variadic tuple
            let param_vals: Vec<_> = (0..n)
                .map(|i| func.get_nth_param(i as u32).unwrap())
                .collect();
            self.build_multi_param_key(&param_vals)
        }
    }
}
```

**Runtime function additions:**
```c
// runtime/src/memoization.h
void* vp_tuple_create1(int64_t v0);
void* vp_tuple_create2(int64_t v0, int64_t v1);
void* vp_tuple_create3(int64_t v0, int64_t v1, int64_t v2);
void* vp_tuple_create_n(int64_t* values, size_t count);
```

**Estimated:** 6 hours

---

### Task 2.2: Non-Integer Parameter Support

**File:** `src/codegen/core/functions.rs`

**Type conversions needed:**

| Type | Key Strategy |
|------|--------------|
| `int`, `i64` | Direct use |
| `float`, `f64` | Bitcast to i64 |
| `bool` | Convert to 0/1 |
| `str` | Hash string content |
| `list` | Hash elements (if hashable) |
| `dict` | Not hashable (error) |

**Implementation:**
```rust
fn build_single_param_key<'ctx>(
    &mut self,
    param_val: BasicValueEnum<'ctx>,
) -> Result<PointerValue<'ctx>, String> {
    if param_val.is_int_value() {
        // Direct integer key
        Ok(self.builder.build_call(
            memo_funcs.tuple_create1,
            &[param_val.into_int_value().into()],
            "key",
        ).try_as_basic_value().left().unwrap().into_pointer_value())
    }
    else if param_val.is_float_value() {
        // Bitcast float to int for hashing
        let int_val = self.builder.build_bitcast(
            param_val.into_float_value(),
            self.context.i64_type(),
            "float_as_int",
        );
        Ok(self.builder.build_call(
            memo_funcs.tuple_create1,
            &[int_val.into()],
            "key",
        ).try_as_basic_value().left().unwrap().into_pointer_value())
    }
    else if param_val.is_pointer_value() {
        // String/list - need to hash content
        let hash = self.generate_string_hash(param_val.into_pointer_value())?;
        Ok(self.builder.build_call(
            memo_funcs.tuple_create1,
            &[hash.into()],
            "key",
        ).try_as_basic_value().left().unwrap().into_pointer_value())
    }
    else {
        Err(format!("Cannot create cache key for parameter type {:?}", param_val.get_type()))
    }
}
```

**String hash runtime function:**
```c
// runtime/src/memoization.h
uint64_t vp_hash_string(const char* str);
void* vp_tuple_create_str(const char* str);
```

**Estimated:** 8 hours

---

### Task 2.3: Type Validation

**File:** `src/semantic/type_checker/stmts.rs`

**Add validation for memoized functions:**
```rust
fn check_memoized_function_params(func: &Function) -> Result<(), String> {
    for param in &func.params {
        match &param.type_ann {
            Some(Type::Dict(_, _)) => {
                return Err(format!(
                    "Parameter '{}' has dict type which is not hashable for @lru_cache",
                    param.name
                ));
            }
            Some(Type::List(inner)) => {
                // Lists are hashable only if elements are hashable
                // For now, warn but allow
                eprintln!("warning: list parameter '{}' - ensure elements are hashable");
            }
            _ => {}
        }
    }
    Ok(())
}
```

**Estimated:** 3 hours

---

### Task 2.4: Testing Phase 2

**Test file:** `tests/decorators/test_phase2_multiparam.vp`

```python
@lru_cache(maxsize=128)
def add(a, b):
    return a + b

@lru_cache(maxsize=128)
def gcd(a, b):
    if b == 0:
        return a
    return gcd(b, a % b)

@lru_cache(maxsize=128)
def fib_2d(x, y):
    """2D Fibonacci-like function."""
    if x <= 0 or y <= 0:
        return 1
    return fib_2d(x-1, y) + fib_2d(x, y-1)

def main():
    # Test two-parameter functions
    assert add(5, 3) == 8
    assert add(100, 200) == 300
    
    assert gcd(48, 18) == 6
    assert gcd(100, 35) == 5
    
    assert fib_2d(5, 5) == 252
    
    print("All multi-parameter tests passed!")
    return 0

main()
```

**Estimated:** 3 hours

---

**Phase 2 Total:** ~20 hours (4-5 days)

---

## Phase 3: Safety & Correctness (P1)

### Task 3.1: Thread Safety

**File:** `runtime/src/memoization.h`, `runtime/src/memoization.c`

**Changes:**
```c
// Add mutex to cache structs
typedef struct LRUCache {
    pthread_mutex_t lock;  // NEW
    size_t maxsize;
    size_t currsize;
    // ... rest unchanged
} LRUCache;

typedef struct Cache {
    pthread_mutex_t lock;  // NEW
    size_t currsize;
    HashMap* map;
} Cache;
```

**Update all cache operations:**
```c
void* vp_cache_get(Cache* cache, void* key) {
    pthread_mutex_lock(&cache->lock);
    
    void* result = hashmap_get(cache->map, hash, key)->value;
    
    pthread_mutex_unlock(&cache->lock);
    return result;
}

void vp_cache_set(Cache* cache, void* key, void* value) {
    pthread_mutex_lock(&cache->lock);
    
    hashmap_set(cache->map, hash, key, value);
    cache->currsize++;
    
    pthread_mutex_unlock(&cache->lock);
}

void vp_cache_destroy(Cache* cache) {
    pthread_mutex_destroy(&cache->lock);  // NEW
    // ... rest of cleanup
}
```

**Estimated:** 6 hours

---

### Task 3.2: GC/ARC Integration

**File:** `runtime/src/memoization.c`, `src/codegen/runtime/memoization.rs`

**Approach:** Register cached objects with Viper's ARC system.

**C implementation:**
```c
#include "gc.h"  // Viper's GC header

void vp_cache_set(Cache* cache, void* key, void* value) {
    pthread_mutex_lock(&cache->lock);
    
    // Register value with GC
    vp_gc_register_root(value);
    
    hashmap_set(cache->map, hash, key, value);
    cache->currsize++;
    
    pthread_mutex_unlock(&cache->lock);
}

void vp_cache_remove_node(Cache* cache, CacheNode* node) {
    // Unregister from GC before freeing
    vp_gc_unregister_root(node->value);
    
    // ... free node
}
```

**Estimated:** 8 hours

---

### Task 3.3: Memory Leak Prevention

**File:** `runtime/src/memoization.c`

**Add LRU eviction for bounded cache:**
```c
static void lru_cache_evict(LRUCache* cache) {
    if (!cache->tail) return;
    
    LRUCacheNode* node = cache->tail;
    
    // Remove from linked list
    if (node->prev) {
        node->prev->next = NULL;
        cache->tail = node->prev;
    } else {
        cache->head = NULL;
        cache->tail = NULL;
    }
    
    // Remove from hash map
    void* old_value;
    hashmap_remove(cache->map, node->key_hash, node->key, &old_value);
    
    // Unregister from GC and free
    vp_gc_unregister_root(old_value);
    free(node->key);
    free(node);
    
    cache->currsize--;
}
```

**Estimated:** 4 hours

---

### Task 3.4: Testing Phase 3

**Test file:** `tests/decorators/test_phase3_safety.vp`

```python
@lru_cache(maxsize=10)  # Small maxsize to test eviction
def compute(x):
    return x * x

def main():
    # Test eviction works
    for i in range(20):
        result = compute(i)
        print(f"compute({i}) = {result}")
    
    # Cache should have at most 10 entries
    # (would verify with cache_info when implemented)
    
    print("Eviction test passed!")
    return 0

main()
```

**Estimated:** 3 hours

---

**Phase 3 Total:** ~21 hours (4-5 days)

---

## Phase 4: Polish & Debugging (P2)

### Task 4.1: cache_info() Implementation

**File:** `runtime/src/memoization.c`, `src/codegen/runtime/memoization.rs`

**C implementation:**
```c
typedef struct CacheInfo {
    size_t hits;
    size_t misses;
    size_t maxsize;
    size_t currsize;
} CacheInfo;

void vp_cache_info(Cache* cache, CacheInfo* info) {
    pthread_mutex_lock(&cache->lock);
    info->hits = cache->hits;      // Need to add counters
    info->misses = cache->misses;
    info->maxsize = cache->maxsize;
    info->currsize = cache->currsize;
    pthread_mutex_unlock(&cache->lock);
}
```

**LLVM codegen:**
```rust
// Generate cache_info call when user calls fib.cache_info()
fn generate_cache_info_call(...) {
    let info_struct = self.builder.build_alloca(cache_info_type, "info");
    self.builder.build_call(
        memo_funcs.cache_info,
        &[cache_global.into(), info_struct.into()],
        "",
    );
    // Load and return struct
}
```

**Estimated:** 6 hours

---

### Task 4.2: cache_clear() Implementation

**File:** `runtime/src/memoization.c`

```c
void vp_cache_clear(Cache* cache) {
    pthread_mutex_lock(&cache->lock);
    
    // Free all entries
    for (size_t i = 0; i < cache->map->capacity; i++) {
        CacheNode* node = cache->map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            vp_gc_unregister_root(node->value);
            free(node->key);
            free(node);
            node = next;
        }
    }
    
    // Reset map
    memset(cache->map->buckets, 0, cache->map->capacity * sizeof(CacheNode*));
    cache->map->size = 0;
    cache->currsize = 0;
    cache->hits = 0;
    cache->misses = 0;
    
    pthread_mutex_unlock(&cache->lock);
}
```

**Estimated:** 4 hours

---

### Task 4.3: Decorator Argument Validation

**File:** `src/semantic/type_checker/stmts.rs`

```rust
fn validate_lru_cache_decorator(decorator: &Decorator) -> Result<(), String> {
    // Check for unknown keyword arguments
    for (key, _) in &decorator.keywords {
        if key != "maxsize" {
            return Err(format!(
                "Unknown keyword argument '{}' for @lru_cache (expected 'maxsize')",
                key
            ));
        }
    }
    
    // Validate maxsize value
    for (key, value) in &decorator.keywords {
        if key == "maxsize" {
            if let Expr::Int(n, _) = value {
                if *n < 0 {
                    return Err(format!(
                        "maxsize must be non-negative, got {}",
                        n
                    ));
                }
            } else {
                return Err("maxsize must be an integer".to_string());
            }
        }
    }
    
    Ok(())
}
```

**Estimated:** 3 hours

---

### Task 4.4: Testing Phase 4

**Test file:** `tests/decorators/test_phase4_debugging.vp`

```python
@lru_cache(maxsize=128)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    # Test cache_info
    fib(10)
    fib(10)  # Should be cache hit
    fib(11)
    
    info = fib.cache_info()
    print(f"CacheInfo(hits={info.hits}, misses={info.misses}, maxsize={info.maxsize}, currsize={info.currsize})")
    
    assert info.hits == 1, f"Expected 1 hit, got {info.hits}"
    assert info.misses == 2, f"Expected 2 misses, got {info.misses}"
    
    # Test cache_clear
    fib.cache_clear()
    info = fib.cache_info()
    assert info.currsize == 0, f"Expected 0 after clear, got {info.currsize}"
    
    print("Debugging features work!")
    return 0

main()
```

**Estimated:** 3 hours

---

**Phase 4 Total:** ~16 hours (2-3 days)

---

## Summary

### Effort Estimate

| Phase | Tasks | Hours | Days |
|-------|-------|-------|------|
| **1. Cache Wrapper** | 4 tasks | 18 | 3-4 |
| **2. Multi-Param** | 4 tasks | 20 | 4-5 |
| **3. Safety** | 4 tasks | 21 | 4-5 |
| **4. Polish** | 4 tasks | 16 | 2-3 |
| **Total** | 16 tasks | **75** | **13-17** |

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLVM API complexity | Medium | High | Start with simple cases, test incrementally |
| Thread safety bugs | Medium | High | Extensive testing, use TSan |
| GC integration issues | High | Medium | Coordinate with GC team, careful testing |
| Performance regression | Low | Medium | Benchmark before/after each phase |

### Success Criteria

- [ ] `fib(50)` completes in <1 second
- [ ] Multi-parameter functions work correctly
- [ ] No memory leaks in long-running programs
- [ ] Thread-safe cache access
- [ ] `cache_info()` and `cache_clear()` functional
- [ ] All 434+ existing tests still pass

---

*Last Updated: March 12, 2026*  
*Version: 1.0*  
*Author: Viper Language Team*
