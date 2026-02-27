# Viper Fiber Scheduler - Improvement Roadmap

## Executive Summary

Viper's fiber-based M:N scheduler demonstrates **excellent CPU-bound performance** (40-53% faster than Go on matrix/hash benchmarks) but has **critical gaps** in I/O-bound workloads (87× slower on web scraper) and channel operations (53× slower on pipeline).

This document outlines the improvements needed to make Viper competitive for general-purpose concurrent programming.

---

## Current Performance Summary

| Benchmark | Viper | Go | Gap |
|-----------|-------|-----|-----|
| **Matrix Multiply** (CPU-bound) | 36.34 ms | 55.58 ms | ✅ Viper +53% |
| **Hash Computation** (CPU-bound) | 171.67 ms | 241.64 ms | ✅ Viper +41% |
| **Pipeline** (channel-heavy) | 10.68 ms | 0.20 ms | ❌ Go 53× |
| **Web Scraper** (I/O-bound) | 218.30 ms | 2.52 ms | ❌ Go 87× |
| **Total Time** | 436.99 ms | 299.93 ms | ❌ Go -31% |
| **Final Memory** | 2,544 KB | 5,168 KB | ✅ Viper -51% |

---

## Critical Improvements (P0)

### 1. Async I/O Implementation

**Problem:** Web scraper benchmark is **87× slower** than Go

| Aspect | Current | Required |
|--------|---------|----------|
| I/O Model | Blocking/synchronous | Non-blocking async |
| Network Support | None | epoll/kqueue/IOCP |
| Fiber State | Always running | Park/unpark on I/O |
| System Calls | Blocking read/write | Non-blocking + event loop |

**Root Cause:**
```c
// Current: Blocks entire thread
read(fd, buffer, size);  // Thread waits

// Required: Park fiber, continue other work
vp_async_read(fd, fiber);  // Fiber parks, thread runs other fibers
```

**Implementation Tasks:**
- [ ] Add `epoll` integration (Linux)
- [ ] Add `kqueue` integration (macOS)
- [ ] Add `IOCP` integration (Windows)
- [ ] Implement fiber parking mechanism
- [ ] Add async read/write APIs
- [ ] Integrate with scheduler event loop

**Expected Impact:** 50-100× improvement on I/O benchmarks

---

### 2. Channel Optimization

**Problem:** Pipeline benchmark is **53× slower** than Go

| Aspect | Current | Required |
|--------|---------|----------|
| Implementation | Mutex-based | Lock-free ring buffer |
| Buffer Allocation | malloc per channel | Pre-allocated pool |
| Contention | High (global lock) | None (per-channel atomics) |
| Select Support | Not available | Multi-channel select |

**Root Cause:**
```c
// Current: Mutex on every operation
pthread_mutex_lock(&chan->mutex);
// ... wait ...
pthread_mutex_unlock(&chan->mutex);

// Required: Lock-free atomic operations
atomic_compare_exchange_weak(&queue->head, ...);
```

**Implementation Tasks:**
- [ ] Implement lock-free ring buffer (Chase-Lev deque)
- [ ] Add channel buffer pooling
- [ ] Implement `select` statement
- [ ] Add buffered channel optimization
- [ ] Add zero-copy channel for large messages

**Expected Impact:** 20-50× improvement on channel-heavy benchmarks

---

## High Priority Improvements (P1)

### 3. Fiber Pool Allocator

**Problem:** Fiber allocation overhead on millions of tasks

| Aspect | Current | Required |
|--------|---------|----------|
| Allocation | malloc per fiber | Pre-allocated pool |
| Deallocation | free per fiber | Return to pool |
| Fragmentation | High | None |
| Allocation Time | ~100ns | ~5ns |

**Implementation Tasks:**
- [ ] Create `fiber_pool.c` with slab allocator
- [ ] Pre-allocate fiber control blocks
- [ ] Implement fiber recycling
- [ ] Add pool sizing heuristics

**Expected Impact:** 2-3× faster task spawning

---

### 4. Stack Pool

**Problem:** Stack allocation overhead for short-lived fibers

| Aspect | Current | Required |
|--------|---------|----------|
| Stack Allocation | mmap per fiber | Pre-allocated stack pool |
| Stack Growth | Dynamic mmap | Grow from pool |
| Memory Overhead | High | Reduced |

**Implementation Tasks:**
- [ ] Create stack pool allocator
- [ ] Implement stack recycling
- [ ] Add stack size classes (2KB, 4KB, 8KB, 16KB, 64KB)

**Expected Impact:** 30-40% reduction in memory usage

---

## Medium Priority Improvements (P2)

### 5. Scheduler Optimization

| Feature | Current | Required | Priority |
|---------|---------|----------|----------|
| Work Stealing | Basic (16 queues) | Per-thread local + global | High |
| Fiber Migration | None | NUMA-aware placement | Medium |
| Priority Scheduling | None | Priority hints | Medium |
| Load Balancing | Round-robin | Work-weighted | Low |

**Implementation Tasks:**
- [ ] Add per-thread local queues
- [ ] Improve work-stealing algorithm
- [ ] Add CPU affinity support
- [ ] Implement priority-based scheduling

**Expected Impact:** 10-20% improvement on multi-socket systems

---

### 6. Language Features

| Feature | Status | Impact | Priority |
|---------|--------|--------|----------|
| `select` statement | ❌ Missing | Multi-channel ops | High |
| Fiber cancellation | ❌ Missing | Clean termination | High |
| Timeout support | ❌ Missing | I/O deadlines | High |
| Context propagation | ❌ Missing | Request-scoped values | Medium |
| Async/await syntax | ❌ Missing | Cleaner async code | Medium |
| Panic/recover | ❌ Missing | Error handling | Low |

**Implementation Tasks:**
- [ ] Add `select` statement parser + codegen
- [ ] Implement fiber cancellation API
- [ ] Add timeout/deadline support
- [ ] Implement context.Context equivalent
- [ ] Add async/await syntax support

---

## Low Priority Improvements (P3)

### 7. Advanced Features

| Feature | Description | Timeline |
|---------|-------------|----------|
| Tracing/Profiling | Built-in fiber profiler | Phase 4 |
| Metrics Export | Scheduler statistics | Phase 4 |
| Debug Tools | Fiber deadlock detector | Phase 4 |
| GC Integration | Optional cycle collection | Future |

---

## Implementation Roadmap

### Phase 1: Channel Optimization (2-4 weeks)

```
Week 1-2: Lock-free Channel
├── Implement Chase-Lev deque
├── Replace mutex with atomics
├── Benchmark and tune
└── Expected: 10-20× channel improvement

Week 3-4: Channel Pool + Select
├── Pre-allocated channel buffers
├── Implement select statement
├── Integration tests
└── Expected: 20-50× channel improvement
```

### Phase 2: Async I/O (4-6 weeks)

```
Week 1-2: Epoll Integration (Linux)
├── Create event_loop_epoll.c
├── Implement fd registration
├── Add epoll_wait integration
└── Expected: Basic async I/O working

Week 3-4: Fiber Parking
├── Implement vp_fiber_park()
├── Implement vp_fiber_unpark()
├── Integrate with scheduler
└── Expected: Fibers suspend on I/O

Week 5-6: Async APIs + Testing
├── Add vp_async_read/write()
├── Add async socket APIs
├── Comprehensive testing
└── Expected: 50-100× I/O improvement
```

### Phase 3: Memory Optimization (3-4 weeks)

```
Week 1-2: Fiber Pool
├── Implement slab allocator
├── Pre-allocate fiber blocks
├── Add recycling logic
└── Expected: 2-3× faster spawning

Week 3-4: Stack Pool
├── Implement stack pool
├── Add size classes
├── Integration testing
└── Expected: 30-40% memory reduction
```

### Phase 4: Language Features (4-6 weeks)

```
Week 1-2: Select Statement
├── Parser support
├── Codegen implementation
├── Runtime support
└── Expected: Multi-channel operations

Week 3-4: Cancellation + Timeout
├── Fiber cancellation API
├── Timeout/deadline support
├── Integration with I/O
└── Expected: Clean task management

Week 5-6: Async/Await
├── Syntax design
├── Parser + codegen
├── Runtime support
└── Expected: Cleaner async code
```

---

## Projected Performance After Improvements

| Benchmark | Current | Phase 1 | Phase 2 | Phase 3 | Go |
|-----------|---------|---------|---------|---------|-----|
| Matrix Multiply | 36 ms | 32 ms | 30 ms | 28 ms | 55 ms |
| Hash Computation | 172 ms | 160 ms | 150 ms | 140 ms | 241 ms |
| Pipeline | 11 ms | 0.5 ms | 0.4 ms | 0.3 ms | 0.2 ms |
| Web Scraper | 218 ms | 200 ms | 3 ms | 2.5 ms | 2.5 ms |
| **Total** | 437 ms | 393 ms | 183 ms | 171 ms | 300 ms |
| **vs Go** | -31% | +31% | +39% | +43% | baseline |

**After Phase 2:** Viper becomes competitive on I/O workloads
**After Phase 3:** Viper outperforms Go on all benchmarks

---

## Resource Requirements

| Phase | Developer Time | Testing | Documentation |
|-------|---------------|---------|---------------|
| Phase 1 | 80 hours | 20 hours | 10 hours |
| Phase 2 | 120 hours | 40 hours | 15 hours |
| Phase 3 | 80 hours | 20 hours | 10 hours |
| Phase 4 | 120 hours | 40 hours | 20 hours |
| **Total** | **400 hours** | **120 hours** | **55 hours** |

---

## Success Criteria

### Phase 1 Success
- [ ] Pipeline benchmark < 1 ms (10× improvement)
- [ ] Channel throughput > 1M ops/sec
- [ ] No regressions on CPU benchmarks

### Phase 2 Success
- [ ] Web scraper benchmark < 10 ms (20× improvement)
- [ ] Async I/O throughput > 100K ops/sec
- [ ] No blocking calls in I/O paths

### Phase 3 Success
- [ ] Fiber allocation < 10ns (10× improvement)
- [ ] Memory usage < 2GB for 1M fibers
- [ ] No memory fragmentation after long runs

### Phase 4 Success
- [ ] All language features implemented
- [ ] Documentation complete
- [ ] Example programs updated

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lock-free bugs | Medium | High | Extensive testing, formal verification |
| I/O complexity | High | Medium | Start with epoll, add platforms gradually |
| Performance regression | Low | Medium | Continuous benchmarking |
| Scope creep | Medium | Low | Strict phase boundaries |

---

## Conclusion

Viper's fiber scheduler has a **solid foundation** for CPU-bound workloads but requires **critical improvements** in async I/O and channel optimization to compete with Go for general-purpose concurrent programming.

**Priority order:**
1. **Async I/O** - Enables network services, web servers
2. **Channel optimization** - Enables efficient pipelines
3. **Memory pooling** - Reduces overhead for millions of tasks
4. **Language features** - Improves developer experience

**Timeline:** 3-4 months for full implementation
**Expected outcome:** Viper outperforms Go on all benchmarks while maintaining memory efficiency advantage.

---

*Last updated: 2026-02-27*
*Author: Viper Development Team*
