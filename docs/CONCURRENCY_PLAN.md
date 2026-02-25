# Implementation Plan: 10M+ Concurrent Tasks

## Targets
- **I/O**: HTTP, WebSocket, database, files, network sockets - all
- **Memory**: ~25GB budget (2KB initial stack × 10M tasks, grows on demand)
- **Platforms**: Linux, macOS, Windows

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                     Viper Program                                │
│  task worker(1)  task worker(2)  ... task worker(10M)        │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│              Task Spawner                                         │
│  • Allocate from task pool (pre-allocated)                      │
│  • Capture args into closure                                     │
│  • Submit to fiber scheduler                                     │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│              Fiber Manager (M:N Scheduling)                      │
│  • 10M fibers across N threads                                  │
│  • Stack: 2KB initial, grows to 64KB (lazy allocation)        │
│  • Suspend on await, resume on completion                       │
│  • Context switch: ~100ns (vs 10μs for OS thread)              │
└──────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────────────┐
│  Work-Stealing Pool     │     │       Event Loop                │
│  • 100-1000 threads     │     │  • epoll (Linux)               │
│  • Chase-Lev deque      │     │  • kqueue (macOS)              │
│  • NUMA-aware           │     │  • IOCP (Windows)              │
│  • Adaptive sizing      │     │  • Timer wheel                  │
└─────────────────────────┘     └─────────────────────────────────┘
```

---

## Memory Budget (~25GB)

| Component | Per-Item | 10M Items | Notes |
|-----------|----------|-----------|-------|
| Fiber Stack | 2KB (lazy) | 20GB max | Grows to 64KB on demand |
| Task Control Block | 64 bytes | 640MB | Pre-allocated pool |
| Thread Stack | 2MB | 2GB | 1000 threads |
| Event Loop | - | 100MB | Per-instance |
| Scheduler | - | 50MB | Global |
| **Total** | - | **~25GB max** | At full utilization |

*Note: Stacks are lazy-allocated with guard pages, actual memory at rest ~2-3GB*

---

## Implementation Phases

### Phase 1: Basic Concurrent Tasks (Day 1)
**Goal**: Tasks run on thread pool, not inline

| Task | File | Change |
|------|------|--------|
| Add `vp_submit_task` declaration | `src/codegen/runtime.rs` | Add function type |
| Update task codegen | `src/codegen/statements.rs` | Call `vp_submit_task` instead of inline exec |
| Test | - | Verify tasks run concurrently |

**Success Criteria**: 1,000+ tasks run concurrently

---

### Phase 2: Remove Limits + Memory Pooling (Days 2-3)
**Goal**: No artificial limits, efficient allocation

| Task | File | Change |
|------|------|--------|
| Remove 256 task limit | `runtime/src/async.c` | Use dynamic allocation |
| Add task pool | `runtime/src/async.c` | Pre-allocate task blocks |
| Increase max threads | `runtime/src/concurrency/thread_pool.c` | 64 → 512 |
| Add task pool allocator | `runtime/src/task_pool.c` | New: slab allocator |
| Test | - | Run 10K+ tasks |

**Success Criteria**: 10,000+ tasks run concurrently

---

### Phase 3: Fiber Runtime (Days 4-10)
**Goal**: Goroutine-like cheap context switching

| Task | File | Status |
|------|------|--------|
| Create fiber.h | `runtime/src/fiber.h` | ✅ Complete |
| Create fiber.c | `runtime/src/fiber.c` | ✅ Complete |
| Create scheduler.c | `runtime/src/scheduler.c` | ✅ Complete |
| Integrate with task spawn | Both | 🔄 In Progress |
| Dynamic stack growth | `runtime/src/fiber.c` | ✅ Complete |
| Guard pages | `runtime/src/fiber.c` | ✅ Complete |
| Test | - | ⏳ Pending |

**Success Criteria**: 100,000+ fibers run concurrently

---

### Phase 4: Async I/O (Days 11-20)
**Goal**: Handle all I/O patterns efficiently

| Task | File | Change |
|------|------|--------|
| Create event_loop.c | `runtime/src/event_loop.c` | Cross-platform event loop |
| epoll integration (Linux) | `runtime/src/event_loop_epoll.c` | Linux async I/O |
| kqueue integration (macOS) | `runtime/src/event_loop_kqueue.c` | macOS async I/O |
| IOCP integration (Windows) | `runtime/src/event_loop_iocp.c` | Windows async I/O |
| Async file I/O | `runtime/src/async_file.c` | aio_read/write |
| Async sockets | `runtime/src/async_net.c` | TCP/UDP/Unix sockets |
| Async DNS | `runtime/src/async_dns.c` | Non-blocking resolve |
| Test | - | HTTP server with 100K connections |

**Success Criteria**: 100,000+ concurrent network connections

---

### Phase 5: Scheduler Optimization (Days 21-28)
**Goal**: Efficient at 10M scale

| Task | File | Change |
|------|------|--------|
| NUMA-aware scheduling | `runtime/src/scheduler.c` | Locality-aware steal |
| Adaptive thread count | `runtime/src/thread_pool.c` | Scale threads based on load |
| Task affinity | `runtime/src/scheduler.c` | Pin tasks to cores |
| Metrics/Profiling | `runtime/src/metrics.c` | Track scheduler stats |
| Test | - | 1M task benchmark |

**Success Criteria**: 1,000,000+ tasks run concurrently

---

### Phase 6: Integration & Testing (Days 29-30)
**Goal**: Production-ready

| Task | File | Change |
|------|------|--------|
| stdlib async I/O | `std/async.vp` | High-level async primitives |
| HTTP server | `std/http.vp` | Built on async I/O |
| WebSocket | `std/websocket.vp` | Full-duplex communication |
| Database drivers | `std/db/*.vp` | Async DB connections |
| Cross-platform build | `build.rs` | Platform-specific compilation |
| Stress test | - | 10M concurrent tasks |

**Success Criteria**: 10,000,000+ tasks run concurrently

---

## Platform-Specific I/O

| Platform | I/O Mechanism | Header |
|----------|--------------|--------|
| Linux | epoll + io_uring | `<sys/epoll.h>`, `<io_uring.h>` |
| macOS | kqueue | `<sys/event.h>` |
| Windows | IOCP | `<windows.h>` + `<mswsock.h>` |

---

## API (After Implementation)

```viper
# Task spawning (already works)
task worker(1, c)
task worker(2, c)

# Channels (already works)
c = chan(100)
send(c, value)
recv(c)

# Sync block (already works)
sync:
    task worker(1)

# Future/await (to implement)
async def fetch(url):
    response = await http_get(url)
    return response

# Async context manager
async with open("file.txt") as f:
    data = await f.read()

# Async iteration
async for item in stream:
    print(item)
```

---

## Key Files

### New Files (12 files)
- `runtime/src/fiber.h` - Fiber definitions
- `runtime/src/fiber.c` - Fiber implementation
- `runtime/src/scheduler.c` - Fiber scheduler
- `runtime/src/task_pool.c` - Task memory pool
- `runtime/src/event_loop.c` - Cross-platform event loop
- `runtime/src/event_loop_epoll.c` - Linux event loop
- `runtime/src/event_loop_kqueue.c` - macOS event loop
- `runtime/src/event_loop_iocp.c` - Windows event loop
- `runtime/src/async_file.c` - Async file I/O
- `runtime/src/async_net.c` - Async network I/O
- `runtime/src/async_dns.c` - Async DNS
- `runtime/src/metrics.c` - Scheduler metrics

### Modified Files (6 files)
- `src/codegen/statements.rs` - Task spawn codegen
- `src/codegen/runtime.rs` - Runtime declarations
- `runtime/src/async.c` - Remove limits, add pooling
- `runtime/src/concurrency/thread_pool.c` - Dynamic scaling
- `build.rs` - Platform-specific compilation
- `docs/CONCURRENCY.md` - This documentation

---

## Testing Milestones

| Scale | Phase | Target |
|-------|-------|--------|
| 1,000 | Phase 1 | Basic concurrent tasks |
| 10,000 | Phase 2 | Remove limits |
| 100,000 | Phase 3 | Fiber runtime |
| 1,000,000 | Phase 5 | Scheduler optimization |
| 10,000,000 | Phase 6 | Full stress test |

---

## Stack Growth

```
Initial: 2KB (reserved, not committed)
         │
         ▼ (on stack overflow)
Growth:  4KB (16KB committed)
         │
         ▼
Max:    64KB (64KB committed, guard page after)
```

- **Lazy allocation**: Memory is only committed when used
- **Guard pages**: Detect stack overflow, prevent corruption
- **Overflow handling**: Signal SIGSEGV caught, stack grows
