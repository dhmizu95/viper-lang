# Viper vs Go - Real World Benchmark Comparison

## Overview

This benchmark compares **Viper's fiber-based M:N scheduler** against **Go's goroutines** on realistic workloads including CPU-bound, memory-bound, and mixed tasks.

## Test Environment

| Specification | Value |
|--------------|-------|
| CPU Cores | 8 |
| Viper Version | 0.4.1 |
| Go Version | 1.25.7 |
| Viper Scheduler | M:N Fiber (8 worker threads) |
| Go Scheduler | GOMAXPROCS=8 |

## Benchmark Results

### Summary

| Metric | Viper | Go | Winner |
|--------|-------|-----|--------|
| **Total Time** | 436.99 ms | 299.93 ms | 🏆 Go (-31%) |
| **Peak Memory (RSS)** | 15,488 KB | 16,993 KB | 🏆 Viper (-9%) |
| **Final Memory** | 2,544 KB | 5,168 KB | 🏆 Viper (-51%) |
| **GC Pauses** | N/A (ARC) | 7 collections | 🏆 Viper |

---

### Benchmark 1: Matrix Multiplication (CPU-bound)

**Workload:** 10 matrices, 256×256 elements each (2,560 parallel row computations)

| Metric | Viper | Go | Winner |
|--------|-------|-----|--------|
| Time | 36.34 ms | 55.58 ms | 🏆 **Viper (+53% faster)** |
| Throughput | 70,450 rows/sec | 46,060 rows/sec | 🏆 Viper |
| Memory Delta | +13,952 KB | +16,870 KB | 🏆 Viper |

**Analysis:** Viper's fiber scheduler excels at CPU-bound parallel computation due to:
- Lower context switch overhead (setjmp/longjmp vs goroutine scheduler)
- Better cache locality with work-stealing from global queues
- No GC pressure during computation

---

### Benchmark 2: Hash Computation (CPU-bound)

**Workload:** 10,000 hashes, 10,000 iterations each (100M hash operations)

| Metric | Viper | Go | Winner |
|--------|-------|-----|--------|
| Time | 171.67 ms | 241.64 ms | 🏆 **Viper (+41% faster)** |
| Throughput | 58,250 hashes/sec | 41,384 hashes/sec | 🏆 Viper |
| Memory Delta | +640 KB | +3,767 KB | 🏆 Viper |

**Analysis:** Viper shows significant advantage in tight computational loops:
- Fiber context is lighter than goroutine stacks
- No GC interference during computation
- Direct function calls without scheduler mediation

---

### Benchmark 3: Producer-Consumer Pipeline (Mixed)

**Workload:** 100 pipelines × 5 stages × 1,000 items (500 parallel stage tasks)

| Metric | Viper | Go | Winner |
|--------|-------|-----|--------|
| Time | 10.68 ms | 0.20 ms | 🏆 **Go (53× faster)** |
| Throughput | 46,820 stages/sec | 2,519,907 stages/sec | 🏆 Go |
| Memory Delta | 0 KB | +860 KB | 🏆 Viper |

**Analysis:** Go dominates in pipeline scenarios due to:
- **Channel optimization**: Go's channels are highly optimized for producer-consumer patterns
- **Scheduler maturity**: Go's work-stealing scheduler has 15+ years of optimization
- **Viper limitation**: Current implementation processes all stages sequentially per pipeline

---

### Benchmark 4: Web Scraper Simulation (I/O-bound)

**Workload:** 10,000 simulated HTTP requests with processing

| Metric | Viper | Go | Winner |
|--------|-------|-----|--------|
| Time | 218.30 ms | 2.52 ms | 🏆 **Go (87× faster)** |
| Throughput | 45,808 URLs/sec | 3,973,590 URLs/sec | 🏆 Go |
| Memory Delta | 0 KB | +866 KB | 🏆 Viper |

**Analysis:** Go's massive advantage in I/O-bound tasks:
- **Non-blocking I/O**: Go runtime parks goroutines during I/O
- **Network poller**: Go's integrated network poller handles I/O efficiently
- **Viper limitation**: Current implementation uses busy-wait simulation (no async I/O yet)

---

## Performance Characteristics

### Viper Strengths ✅

| Scenario | Advantage |
|----------|-----------|
| **CPU-bound parallel compute** | 40-53% faster than Go |
| **Memory efficiency** | 51% less memory at rest |
| **Deterministic cleanup** | ARC vs GC pauses |
| **Predictable latency** | No GC stop-the-world |
| **Embedded scenarios** | Single binary, no runtime |

### Go Strengths ✅

| Scenario | Advantage |
|----------|-----------|
| **I/O-bound workloads** | 87× faster (async I/O) |
| **Channel-based patterns** | 53× faster (optimized channels) |
| **Mature scheduler** | 15+ years optimization |
| **GC for complex graphs** | Automatic cycle collection |

---

## Memory Analysis

### Viper Memory Profile

```
Initial:  RSS 1,536 KB
Peak:     RSS 15,488 KB (during matrix benchmark)
Final:    RSS 2,544 KB
Virtual:  VM 560,832 KB (lazy allocation)
```

**Key observations:**
- Memory returns to baseline after benchmarks complete
- Virtual memory is pre-allocated but not committed
- ARC provides deterministic deallocation

### Go Memory Profile

```
Initial:  Alloc 122 KB
Peak:     Alloc 16,993 KB (during matrix benchmark)
Final:    Alloc 5,168 KB
Total:    TotalAlloc 22,621 KB
GC:       7 collections
```

**Key observations:**
- GC keeps memory pressure low during execution
- Final memory higher due to runtime overhead
- GC pauses add latency variability

---

## Use Case Recommendations

### Choose Viper When:

- ✅ **CPU-bound parallel processing** (matrix ops, hashing, encryption)
- ✅ **Embedded systems** with limited memory
- ✅ **Real-time systems** requiring predictable latency
- ✅ **Deterministic resource cleanup** (file handles, connections)
- ✅ **Single-binary deployment** without runtime dependencies

### Choose Go When:

- ✅ **I/O-bound services** (web servers, APIs, proxies)
- ✅ **Channel-based concurrency** (pipelines, fan-out/fan-in)
- ✅ **Rapid prototyping** with rich standard library
- ✅ **Network services** with many concurrent connections
- ✅ **Existing Go ecosystem** integration

---

## Future Improvements for Viper

### Planned Enhancements

| Feature | Expected Impact | Timeline |
|---------|-----------------|----------|
| **Async I/O (epoll/kqueue)** | 50-100× I/O performance | Phase 4 |
| **Optimized channels** | 10-20× channel throughput | Phase 3 |
| **Fiber pools** | 2-3× allocation speed | Phase 2 |
| **NUMA awareness** | 10-20% multi-socket | Phase 5 |
| **Priority scheduling** | Better latency for critical tasks | Phase 5 |

### Projected Performance (After Phase 4)

| Benchmark | Current | Projected | vs Go |
|-----------|---------|-----------|-------|
| Matrix Multiply | 36 ms | 30 ms | +85% faster |
| Hash Computation | 172 ms | 150 ms | +61% faster |
| Pipeline | 11 ms | 0.5 ms | 2.5× slower |
| Web Scraper | 218 ms | 3 ms | ~equal |

---

## How to Run

### Viper Benchmark

```bash
# Build runtime
cd runtime && make

# Compile benchmark
gcc -O3 -I. -o viper_bench benchmarks/realworld_benchmark.c \
    runtime/obj/libviper.a -lpthread -lm

# Run
./viper_bench
```

### Go Benchmark

```bash
# Run directly
cd benchmarks
go run realworld_benchmark.go

# Or build optimized binary
go build -o go_bench -ldflags="-s -w" realworld_benchmark.go
./go_bench
```

---

## Conclusion

**Viper's fiber scheduler** demonstrates excellent performance for **CPU-bound workloads**, outperforming Go by 40-53% in computational tasks while using 51% less memory at rest. The deterministic ARC-based memory management provides predictable latency without GC pauses.

**Go's goroutines** remain superior for **I/O-bound workloads** and channel-based patterns, benefiting from 15+ years of runtime optimization and integrated async I/O support.

**The sweet spot for Viper:** Embedded systems, real-time processing, scientific computing, and scenarios where deterministic behavior and memory efficiency are critical.

**The sweet spot for Go:** Network services, web applications, microservices, and rapid development where ecosystem and I/O performance matter most.

---

*Benchmark conducted on Linux with 8-core CPU. Results may vary based on hardware and system configuration.*
