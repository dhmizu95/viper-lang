# Fiber-Based Async/Await Implementation Plan

**Date:** 2026-03-14  
**Status:** Proposed  
**Priority:** High  

---

## Overview

This plan outlines the implementation of full fiber-based async/await support in Viper, integrating the existing fiber scheduler with the async/await runtime to enable true non-blocking concurrency.

## Current State

### What Works
- ✅ `task`/`sync` concurrency with M:N fiber scheduler
- ✅ Work-stealing scheduler with per-thread queues
- ✅ Fiber pool for efficient allocation
- ✅ Async/await syntax parsing (`async def`, `await expr`)
- ✅ Runtime structures (`ViperFuture`, `ViperTask`, `ViperEventLoop`)

### What's Missing
- ❌ `async def` returns a Future (currently just calls function directly)
- ❌ `await` yields fiber (currently spin-waits)
- ❌ Event loop integration with fiber scheduler
- ❌ Async function state machine transformation
- ❌ `async for` loop support

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Viper Async/Fiber Integration               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  async def foo():                    task foo():                 │
│  ┌────────────────────────┐          ┌────────────────┐          │
│  │  Async Function        │          │  Regular Func  │          │
│  │  (returns Future)      │          │  (void return) │          │
│  └───────────┬────────────┘          └───────┬────────┘          │
│              │                               │                   │
│              ▼                               ▼                   │
│  ┌────────────────────────┐          ┌────────────────┐          │
│  │  vp_async_spawn        │          │  vp_submit_task│          │
│  │  (creates Future)      │          │  (fire-forget) │          │
│  └───────────┬────────────┘          └───────┬────────┘          │
│              │                               │                   │
│              ▼                               ▼                   │
│  ┌─────────────────────────────────────────────────────┐        │
│  │           Fiber Scheduler (M:N)                     │        │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │        │
│  │  │ Fiber Queue │  │ Fiber Queue │  │ Fiber Queue │  │        │
│  │  │ (Thread 0)  │  │ (Thread 1)  │  │ (Thread N)  │  │        │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  │        │
│  └─────────────────────────────────────────────────────┘        │
│                                                                  │
│  await future:                          Event Loop:              │
│  ┌────────────────────────┐          ┌────────────────┐          │
│  │  vp_future_await       │          │  vp_event_loop │          │
│  │  (yields fiber)        │          │  (async I/O)   │          │
│  └───────────┬────────────┘          └───────┬────────┘          │
│              │                               │                   │
│              └───────────────┬───────────────┘                   │
│                              │                                   │
│                              ▼                                   │
│                  ┌─────────────────────┐                         │
│                  │  Fiber Suspension   │                         │
│                  │  (setjmp/longjmp)   │                         │
│                  └─────────────────────┘                         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Fiber Yield Primitive

**Goal:** Add fiber yield capability for async suspension.

#### 1.1 Add `vp_fiber_yield()` to Fiber Runtime

**File:** `runtime/src/fiber.c`, `runtime/src/fiber.h`

```c
// runtime/src/fiber.h
void vp_fiber_yield(void);  // Yield current fiber, run next ready

// runtime/src/fiber.c
void vp_fiber_yield(void) {
    ViperFiber* current = vp_fiber_current();
    if (!current) return;
    
    // Mark as ready
    current->state = FIBER_READY;
    
    // Add to scheduler queue
    vp_scheduler_add_ready(current);
    
    // Jump to scheduler to pick next fiber
    if (current->sched_jump != NULL) {
        siglongjmp(*current->sched_jump, 1);
    }
}
```

#### 1.2 Add Scheduler Jump Point

**File:** `runtime/src/scheduler.c`

```c
// Store jump point in fiber for yield return
struct ViperFiber {
    // ... existing fields ...
    sigjmp_buf* sched_jump;  // Jump point for yield return
};
```

**Estimated Effort:** 2-3 hours  
**Risk:** Low - builds on existing setjmp/longjmp mechanism

---

### Phase 2: Fix `vp_future_await` to Yield Fiber

**Goal:** Replace spin-wait with fiber yield.

#### 2.1 Update `vp_future_await`

**File:** `runtime/src/async.c`

```c
int64_t vp_future_await(ViperFuture* future) {
    if (!future) return 0;

    // Register current fiber as waiting on this future
    ViperFiber* current = vp_fiber_current();
    future->waiting_fiber = current;
    
    while (future->state != ASYNC_COMPLETED && future->state != ASYNC_ERROR) {
        // Yield to scheduler instead of spin-wait
        vp_fiber_yield();
    }
    
    future->waiting_fiber = NULL;
    return future->result;
}
```

#### 2.2 Add Waiting Fiber to ViperFuture

**File:** `runtime/src/async.c`

```c
typedef struct ViperFuture {
    int64_t ref_count;
    AsyncState state;
    int64_t result;
    void (*callback)(struct ViperFuture*);
    void* user_data;
    ViperFiber* waiting_fiber;  // NEW: fiber awaiting this future
} ViperFuture;
```

#### 2.3 Wake Waiting Fiber on Completion

**File:** `runtime/src/async.c`

```c
void vp_future_set_result(ViperFuture* future, int64_t result) {
    if (!future) return;
    future->result = result;
    future->state = ASYNC_COMPLETED;

    // Wake up waiting fiber
    if (future->waiting_fiber) {
        future->waiting_fiber->state = FIBER_READY;
        vp_scheduler_add_ready(future->waiting_fiber);
    }
    
    if (future->callback) {
        future->callback(future);
    }
}
```

**Estimated Effort:** 2-3 hours  
**Risk:** Medium - need to handle edge cases (future already ready, multiple awaiters)

---

### Phase 3: Async Function Code Generation

**Goal:** Transform `async def` into functions that return Futures.

#### 3.1 Update AST for Async Functions

**File:** `src/ast/nodes.rs`

```rust
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub is_async: bool,  // NEW: marks async functions
    pub return_type: Option<Type>,
    pub span: Span,
}
```

#### 3.2 Parser: Mark Async Functions

**File:** `src/parser/statements/function.rs`

```rust
pub fn parse_function_def(parser: &mut StatementParser) -> Result<Stmt> {
    let is_async = parser.peek().kind == TokenKind::Async;
    if is_async {
        parser.next()?;  // consume 'async'
    }
    
    // ... parse function ...
    
    Ok(Stmt::FunctionDef(FunctionDef {
        is_async,
        // ... other fields ...
    }))
}
```

#### 3.3 Codegen: Async Function Returns Future

**File:** `src/codegen/functions.rs`

```rust
pub fn generate_function_def(
    state: &mut CodeGenState,
    func: &FunctionDef,
) -> Result<()> {
    if func.is_async {
        // Async function: wrap in Future
        generate_async_function_wrapper(state, func)
    } else {
        // Regular function
        generate_regular_function(state, func)
    }
}

fn generate_async_function_wrapper(
    state: &mut CodeGenState,
    func: &FunctionDef,
) -> Result<()> {
    // 1. Create the actual function body (prefixed with __async_body_)
    let body_func_name = format!("__async_body_{}", func.name);
    
    // 2. Create wrapper that:
    //    a. Creates a Future
    //    b. Spawns fiber to run body
    //    c. Returns Future
    let wrapper_func = state.module.add_function(
        &mangle_function_name(&func.name, &[]),
        future_return_type(state.context),
        None,
    );
    
    // Wrapper body:
    //   %future = call ptr @vp_future_create()
    //   %ctx = alloca AsyncContext
    //   store %future, %ctx.future
    //   call void @vp_async_spawn(ptr @__async_body_X, ptr %ctx)
    //   ret ptr %future
}
```

#### 3.4 Async Context Structure

For async functions that need to suspend/resume, create a context struct:

```rust
// LLVM struct: { future, local1, local2, state, ... }
struct AsyncContext {
    future: *mut ViperFuture,
    locals: Vec<LLVMType>,  // Captured locals
    state: i32,             // State machine state
}
```

**Estimated Effort:** 6-8 hours  
**Risk:** High - requires careful handling of captured variables and state

---

### Phase 4: Event Loop Integration

**Goal:** Connect event loop with fiber scheduler for async I/O.

#### 4.1 Event Loop Runs on Scheduler Thread

**File:** `runtime/src/scheduler.c`

```c
static void* scheduler_worker(void* arg) {
    SchedulerThread* st = (SchedulerThread*)arg;
    ViperEventLoop* event_loop = vp_event_loop_get_global();

    while (1) {
        ViperFiber* current = scheduler_get_next_fiber(st);
        
        if (current) {
            // Run fiber
            current->state = FIBER_RUNNING;
            current->func(current->arg);
            current->state = FIBER_COMPLETED;
            continue;
        }
        
        // No fibers ready - run event loop for async I/O
        if (event_loop && vp_event_loop_pending_ops(event_loop) > 0) {
            vp_event_loop_run(event_loop, 1);  // 1ms timeout
        }
        
        // Wait for work
        scheduler_wait_for_work(st);
    }
}
```

#### 4.2 Async I/O Wakes Waiting Fibers

**File:** `runtime/src/event_loop.h`, `runtime/src/event_loop.c`

```c
typedef struct AsyncIORequest {
    ViperFiber* waiting_fiber;
    int fd;
    void* buffer;
    size_t count;
    int64_t result;
    bool completed;
} AsyncIORequest;

// When I/O completes:
void on_io_complete(AsyncIORequest* req) {
    req->completed = true;
    req->waiting_fiber->state = FIBER_READY;
    vp_scheduler_add_ready(req->waiting_fiber);
}
```

**Estimated Effort:** 4-6 hours  
**Risk:** Medium - integration complexity

---

### Phase 5: Async For Loops

**Goal:** Support `async for` with fiber yielding.

#### 5.1 Syntax Support (Already Exists)

```python
async for item in async_iterable:
    print(item)
```

#### 5.2 Codegen for Async For

**File:** `src/codegen/statements/loops.rs`

```rust
pub fn generate_async_for(
    state: &mut CodeGenState,
    iter_expr: &Expr,
    body: &[Stmt],
) -> Result<()> {
    // 1. Get async iterator: __aiter__
    let iter = call_aiter(state, iter_expr);
    
    // 2. Loop: await __anext__()
    let loop_block = state.context.append_basic_block(..., "async_for_loop");
    let end_block = state.context.append_basic_block(..., "async_for_end");
    
    // Loop body:
    //   %item = call ptr @vp_async_next(ptr %iter)
    //   %done = icmp eq ptr %item, null
    //   br %done, label %end, label %body
}
```

#### 5.3 Runtime: Async Iterator Protocol

**File:** `runtime/src/async.c`

```c
void* vp_async_aiter(void* iterable) {
    // Call __aiter__ method on iterable
    // Return async iterator
}

int64_t vp_async_anext(void* iterator) {
    // Call __anext__ method
    // Returns value or -1 for StopAsyncIteration
}
```

**Estimated Effort:** 3-4 hours  
**Risk:** Low - follows Python's async iterator protocol

---

### Phase 6: Testing & Benchmarks

#### 6.1 Unit Tests

**File:** `tests/unit/async.rs`

```rust
#[test]
fn test_async_def_returns_future() {
    // async def foo(): return 42
    // f = foo()
    // assert isinstance(f, Future)
}

#[test]
fn test_await_yields_fiber() {
    // Verify fiber yield during await
}
```

#### 6.2 Integration Tests

**File:** `tests/integration/async_await.rs`

```python
async def fetch_data(id):
    await sleep(100)  # Simulate I/O
    return id * 2

async def main():
    futures = []
    for i in range(100):
        futures.append(fetch_data(i))
    
    results = []
    for f in futures:
        results.append(await f)
    
    print(sum(results))
```

#### 6.3 Benchmarks

Compare:
- `task`/`sync` vs `async`/`await` for CPU-bound work
- Async I/O vs threaded I/O
- Memory usage per async task vs fiber task

**Estimated Effort:** 4-6 hours  
**Risk:** Low

---

## File Changes Summary

| File | Changes |
|------|---------|
| `runtime/src/fiber.c` | Add `vp_fiber_yield()` |
| `runtime/src/fiber.h` | Declare yield function |
| `runtime/src/async.c` | Fix `vp_future_await`, add waiting fiber |
| `runtime/src/scheduler.c` | Store sched_jump, integration |
| `src/ast/nodes.rs` | Add `is_async` to FunctionDef |
| `src/parser/statements/function.rs` | Parse `async def` |
| `src/codegen/functions.rs` | Async function wrapper |
| `src/codegen/statements/loops.rs` | Async for codegen |
| `src/jit_stubs/concurrency.rs` | JIT stubs for async |
| `tests/integration/async_await.rs` | Integration tests |

---

## Timeline

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Fiber yield primitive | 2-3 hours |
| 2 | Fix `vp_future_await` | 2-3 hours |
| 3 | Async function codegen | 6-8 hours |
| 4 | Event loop integration | 4-6 hours |
| 5 | Async for loops | 3-4 hours |
| 6 | Testing & benchmarks | 4-6 hours |
| **Total** | | **21-30 hours** |

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| State machine complexity | High | Start with simple async functions (no suspension points) |
| Fiber stack corruption | High | Thorough testing with stack-heavy async functions |
| Event loop deadlock | Medium | Add timeouts and debug logging |
| JIT/AOT behavior divergence | Medium | Test both modes in parallel |

---

## Success Criteria

1. ✅ `async def` returns a Future object
2. ✅ `await` yields fiber (no spin-wait)
3. ✅ Multiple async functions can run concurrently
4. ✅ `async for` loops work correctly
5. ✅ No deadlocks or race conditions in tests
6. ✅ Performance comparable to `task`/`sync` for similar workloads

---

## Future Enhancements

- **Async context managers** (`async with`)
- **Task groups** (structured concurrency)
- **Timeout/cancellation** support
- **Async generators** (`async def` with `yield`)
- **Integration with epoll/kqueue** for real async I/O
