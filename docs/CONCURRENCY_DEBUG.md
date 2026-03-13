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
**Status**: ❌ Broken
**Error**: `Call parameter type does not match function signature`
**Example that fails**:
```python
def worker(id):
    return id

task worker(1)  # FAILS - argument passing broken
```
**Example that works**:
```python
def worker():
    return 1

task worker()  # WORKS
```

### 2. AOT stdout Not Showing Output
**Status**: ❌ Broken
**Symptom**: Print statements produce no output in AOT-compiled binaries
**Test case**:
```python
def main():
    print("Hello")  # No output
```
**Verified**:
- C programs work fine with same compiler
- Runtime has proper fflush() calls
- JIT mode also has issue (subprocess isolation)

### 3. Sync Block Behavior Unclear
**Status**: ⚠️ Needs verification
**Symptom**: Fast execution times (~0ms) could mean:
- Tasks aren't actually running, OR
- Fiber scheduler is extremely efficient

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
- `benchmarks/go/13_fiber_bench.go` - Go benchmark

## Next Steps

1. Fix task argument passing in codegen
2. Investigate AOT stdout issue
3. Verify sync block actually waits for tasks (add counter verification)
4. Add proper benchmarking with timing and memory measurement
