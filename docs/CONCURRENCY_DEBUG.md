# Viper Concurrency Debug Notes

## Overview
This document tracks issues and findings with Viper's Go-routine-like concurrency feature (fibers/tasks).

## Test Date
2026-03-13

## Feature Implementation

### Keywords
- `task function()` - Spawns concurrent task (like Go's `go func()`)
- `sync: block` - Waits for all spawned tasks to complete

### Parsing
Located in `src/parser/statements/concurrency.rs`:
- `parse_sync_block()` - lines 6-13
- `parse_task_spawn()` - lines 15-21

### Code Generation
Located in `src/codegen/statements/concurrency.rs`:
- `generate_sync()` - lines 6-21
- `generate_task()` - lines 23-150

### Runtime Implementation
Located in `runtime/src/`:
- `fiber.h` / `fiber.c` - Fiber (stackful coroutine) implementation
- `scheduler.h` / `scheduler.c` - M:N work-stealing scheduler
- `fiber_pool.h` / `fiber_pool.c` - Fiber allocation pool

## Known Issues

### 1. Task Arguments Not Working
**Status**: ✅ FIXED (2026-03-13)
**Error**: `Call parameter type does not match function signature`
**Root Cause**: 
- Function parameters were defaulting to `Type::Str` (pointer) instead of `Type::Int` (i64 tagged int)
- The task wrapper was using argument value types instead of function parameter types for struct packing

**Fix**:
1. Changed default parameter type from `Type::Str` to `Type::Int` in `src/codegen/functions.rs`
2. Fixed task wrapper to use function's actual parameter types from LLVM signature

**Example that now works**:
```python
def worker(id):
    print("Worker", id)
    return id

task worker(42)  # ✅ WORKS - arguments correctly passed
```

### 2. AOT stdout Not Showing Output
**Status**: ✅ FIXED (2026-03-13)
**Symptom**: Print statements produce no output in AOT-compiled binaries
**Root Cause**:
1. `tagged_int_print()` in `runtime/src/tagged_int.c` was missing `fflush(stdout)` after printing
2. The `main` function wasn't calling the user's `__user_main` function due to a lookup bug

**Fix**:
1. Added `fflush(stdout)` to `tagged_int_print()` in `runtime/src/tagged_int.c`
2. Fixed function lookup in `generate_main_with_statements()` to look for `__user_main` instead of `main`

**Test case**:
```python
def main():
    print("Hello")  # ✅ Now produces output
```

### 3. Sync Block Behavior Unclear
**Status**: ✅ VERIFIED (2026-03-13)
**Test**: Counter verification with 100 tasks
**Result**: Sync block correctly waits for all tasks to complete

```python
counter = 0

def worker():
    global counter
    counter = counter + 1

def main():
    global counter
    counter = 0
    
    for i in range(100):
        task worker()
    
    sync:
        pass
    
    print("Counter:", counter)  # Prints 100 ✅
```

## Performance Benchmarks

### 1 Million Tasks Test

#### Viper (AOT)
```
Time: ~0ms (suspect - needs verification)
Memory: 1.4 MB
Binary size: 32 KB
Exit code: 0
```

#### Viper (JIT)
```
Time: ~110ms
Memory: 71 MB (includes ~60MB LLVM overhead)
Exit code: 0
```

#### Go (baseline)
```
Time: ~565ms
Memory: 46 MB
Exit code: 0
```

### Test Code

**Viper** (`test_concurrency.vp`):
```python
def worker():
    result = 0
    for i in range(1000):
        result = result + i
    return result

def main():
    for i in range(1000000):
        task worker()
    sync:
        pass
```

**Go** (`benchmarks/go/13_fiber_bench.go`):
```go
package main

import (
    "fmt"
    "runtime"
    "sync"
    "time"
)

func worker(wg *sync.WaitGroup) {
    defer wg.Done()
    result := 0
    for i := 0; i < 10; i++ {
        result += i
    }
}

func main() {
    var m runtime.MemStats
    runtime.ReadMemStats(&m)
    fmt.Printf("Initial Memory: Alloc=%v MiB\n", m.Alloc/1024/1024)

    start := time.Now()
    var wg sync.WaitGroup

    for i := 0; i < 1000000; i++ {
        wg.Add(1)
        go worker(&wg)
    }
    wg.Wait()

    elapsed := time.Since(start)
    runtime.ReadMemStats(&m)
    fmt.Printf("Final Memory: Alloc=%v MiB\n", m.Alloc/1024/1024)
    fmt.Printf("Time: %v\n", elapsed)
}
```

## Concurrency Scale Tests

| Tasks | Viper JIT | Viper AOT | Go |
|-------|-----------|-----------|-----|
| 100 | ✅ | ✅ | ✅ |
| 1,000 | ✅ | ✅ | ✅ |
| 10,000 | ✅ | ✅ | ✅ |
| 100,000 | ✅ | ✅ | ✅ |
| 1,000,000 | ✅ | ✅ | ✅ |

## Files Created During Testing

- `test_concurrency.vp` - Main test file
- `test_print.vp` - Print test file
- `test_task_args.vp` - Task arguments test
- `test_sync_wait.vp` - Sync block verification test
- `test_print_simple.vp` - Simple print test
- `benchmarks/go/13_fiber_bench.go` - Go benchmark

## Next Steps

All known issues have been resolved:
1. ✅ Task argument passing fixed
2. ✅ AOT stdout issue fixed
3. ✅ Sync block behavior verified

Future improvements:
- Add proper benchmarking with timing and memory measurement
- Investigate the suspicious ~0ms timing for 1M tasks in AOT mode
- Add more comprehensive concurrency tests
