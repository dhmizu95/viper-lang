# Fiber-Based Async/Await Implementation Plan

**Date:** 2026-03-14  
**Status:** Ready for Implementation  
**Priority:** High  

---

## Overview

This plan outlines the implementation of full fiber-based async/await support in Viper, integrating the existing fiber scheduler with the async/await runtime to enable true non-blocking concurrency.

**Key Design Decision:** The `task`/`sync` concurrency model remains **unchanged**. Both models coexist and share the same underlying fiber scheduler infrastructure.

---

## Concurrency Models Comparison

| Feature | `task`/`sync` (Existing) | `async`/`await` (New) |
|---------|--------------------------|----------------------|
| **Purpose** | Fire-and-forget parallel work | I/O-bound, results needed |
| **Model** | Go-style goroutines | Python-style futures |
| **Return value** | None (void) | `Future[T]` with result |
| **Syntax** | `task worker()` / `sync:` | `f = worker()` / `await f` |
| **Suspension** | Runs to completion | Can suspend at `await` |
| **Scale** | 10M+ tasks | 10M+ tasks (shared fibers) |
| **Status** | ✅ Working | ❌ Needs implementation |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Viper Concurrency Architecture                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────┐      ┌─────────────────────────┐  │
│  │   task/sync Model       │      │  async/await Model      │  │
│  │   (Fire-and-forget)     │      │  (Future-based)         │  │
│  │                         │      │                         │  │
│  │  task worker(i)         │      │  f = async_worker(i)    │  │
│  │  sync: pass             │      │  r = await f            │  │
│  └───────────┬─────────────┘      └───────────┬─────────────┘  │
│              │                                │                 │
│              │  vp_submit_task()              │  async spawn    │
│              │                                │                 │
│              ▼                                ▼                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Fiber Scheduler (M:N)                       │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │   │
│  │  │ Fiber Queue  │  │ Fiber Queue  │  │ Fiber Queue  │   │   │
│  │  │ (Thread 0)   │  │ (Thread 1)   │  │ (Thread N)   │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘   │   │
│  │                                                           │   │
│  │  Shared Infrastructure:                                   │   │
│  │  - Work-stealing scheduler                                │   │
│  │  - Fiber pool (reuse, no malloc per task)                 │   │
│  │  - 8KB initial stacks, on-demand growth                   │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│              ┌─────────────────────────────┐                   │
│              │  CPU Threads (= core count) │                   │
│              └─────────────────────────────┘                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Current State

### What Works
- ✅ `task`/`sync` concurrency with M:N fiber scheduler
- ✅ Work-stealing scheduler with per-thread queues
- ✅ Fiber pool for efficient allocation (10M+ tasks)
- ✅ Async/await syntax parsing (`async def`, `await expr`)
- ✅ Runtime structures (`ViperFuture`, `ViperTask`, `ViperEventLoop`)

### What's Missing
- ❌ `async def` returns a Future (currently just calls function directly)
- ❌ `await` yields fiber (currently spin-waits)
- ❌ Event loop integration with fiber scheduler
- ❌ Async function state machine transformation
- ❌ `async for` loop support

---

## Implementation Phases

### Phase 1: Fiber Yield Primitive

**Goal:** Add fiber yield capability for async suspension.

#### 1.1 Add `vp_fiber_yield()` to Fiber Runtime

**File:** `runtime/src/fiber.h`

```c
/**
 * Yield current fiber to scheduler.
 * Marks fiber as READY and jumps to scheduler to pick next ready fiber.
 * Used by async/await for non-blocking suspension.
 */
void vp_fiber_yield(void);
```

**File:** `runtime/src/fiber.c`

```c
#include "fiber.h"
#include "scheduler.h"

void vp_fiber_yield(void) {
    ViperFiber* current = vp_fiber_current();
    if (!current) return;
    
    // Mark as ready to run again
    current->state = FIBER_READY;
    
    // Add back to scheduler's ready queue
    vp_scheduler_add_ready(current);
    
    // Jump to scheduler to pick next fiber
    // sched_jump is set when fiber starts running
    if (current->sched_jump != NULL) {
        siglongjmp(*current->sched_jump, 1);
    }
}
```

#### 1.2 Add Scheduler Jump Point to Fiber Struct

**File:** `runtime/src/fiber.h`

```c
typedef struct ViperFiber {
    uint64_t id;
    ViperFiberState state;
    void (*func)(void*);
    void* arg;
    
    /* Stack management */
    void* stack_base;
    size_t stack_size;
    size_t stack_capacity;
    
    /* Context switching */
    sigjmp_buf context;
    sigjmp_buf* sched_jump;  /* NEW: Jump point for yield return */
    
    /* Parent fiber (for nested spawning) */
    struct ViperFiber* parent;
    
    /* Fiber pool reference */
    ViperFiberPool* pool;
} ViperFiber;
```

#### 1.3 Initialize sched_jump in Scheduler

**File:** `runtime/src/scheduler.c`

```c
static void* scheduler_worker(void* arg) {
    SchedulerThread* st = (SchedulerThread*)arg;
    
    while (1) {
        ViperFiber* current = scheduler_get_next_fiber(st);
        
        if (current) {
            // Set up jump point for yield return
            sigjmp_buf jump_buf;
            current->sched_jump = &jump_buf;
            
            if (sigsetjmp(jump_buf, 1) == 0) {
                // First entry - run the fiber
                current->state = FIBER_RUNNING;
                current->func(current->arg);
                current->state = FIBER_COMPLETED;
            } else {
                // Returned via yield - continue loop
            }
            
            current->sched_jump = NULL;
            // ... rest of fiber completion logic
        }
        
        // ... rest of scheduler loop
    }
}
```

**Tests:** See [Test Suite](#test-suite) - Test 1

**Estimated Effort:** 2-3 hours  
**Risk:** Low - builds on existing setjmp/longjmp mechanism

---

### Phase 2: Fix `vp_future_await` to Yield Fiber

**Goal:** Replace spin-wait with fiber yield.

#### 2.1 Add Waiting Fiber to ViperFuture

**File:** `runtime/src/async.c`

```c
typedef struct ViperFuture {
    int64_t ref_count;
    AsyncState state;
    int64_t result;
    void (*callback)(struct ViperFuture*);
    void* user_data;
    ViperFiber* waiting_fiber;      /* NEW: fiber awaiting this future */
    struct ViperFuture* next;       /* NEW: for multiple awaiters */
} ViperFuture;
```

#### 2.2 Update `vp_future_await` to Yield

**File:** `runtime/src/async.c`

```c
int64_t vp_future_await(ViperFuture* future) {
    if (!future) return 0;
    
    // Fast path: future already ready
    if (future->state == ASYNC_COMPLETED || future->state == ASYNC_ERROR) {
        return future->result;
    }
    
    // Register current fiber as waiting on this future
    ViperFiber* current = vp_fiber_current();
    current->waiting_on = future;  // Track what we're waiting on
    
    // Add to future's wait list (for multiple awaiters support)
    future->waiting_fiber = current;
    
    // Wait until future is ready
    while (future->state != ASYNC_COMPLETED && future->state != ASYNC_ERROR) {
        // Yield to scheduler - this is the key change!
        vp_fiber_yield();
    }
    
    current->waiting_on = NULL;
    future->waiting_fiber = NULL;
    return future->result;
}
```

#### 2.3 Wake Waiting Fibers on Completion

**File:** `runtime/src/async.c`

```c
void vp_future_set_result(ViperFuture* future, int64_t result) {
    if (!future) return;
    
    future->result = result;
    future->state = ASYNC_COMPLETED;
    
    // Wake up waiting fiber (if any)
    if (future->waiting_fiber) {
        future->waiting_fiber->state = FIBER_READY;
        vp_scheduler_add_ready(future->waiting_fiber);
    }
    
    // Invoke callback if registered
    if (future->callback) {
        future->callback(future);
    }
}
```

**Tests:** See [Test Suite](#test-suite) - Test 2

**Estimated Effort:** 2-3 hours  
**Risk:** Medium - need to handle edge cases (future already ready, multiple awaiters)

---

### Phase 3: Async Function Code Generation

**Goal:** Transform `async def` into functions that return Futures.

#### 3.1 Update AST for Async Functions

**File:** `src/ast/nodes.rs`

```rust
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub is_async: bool,           // NEW: marks async functions
    pub return_type: Option<Type>,
    pub span: Span,
}

impl FunctionDef {
    pub fn new(
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
        is_async: bool,           // NEW
        return_type: Option<Type>,
        span: Span,
    ) -> Self {
        Self { name, params, body, is_async, return_type, span }
    }
}
```

#### 3.2 Parser: Mark Async Functions

**File:** `src/parser/statements/function.rs`

```rust
pub fn parse_function_def(parser: &mut StatementParser) -> Result<Stmt> {
    // Check for 'async' keyword
    let is_async = parser.peek().kind == TokenKind::Async;
    if is_async {
        parser.next()?;  // consume 'async'
    }
    
    // Expect 'def' keyword
    parser.expect(TokenKind::Def)?;
    
    // Parse function name
    let name_token = parser.expect(TokenKind::Ident)?;
    let name = name_token.lexeme().to_string();
    
    // Parse parameters
    parser.expect(TokenKind::LParen)?;
    let params = parse_parameters(parser)?;
    parser.expect(TokenKind::RParen)?;
    
    // Parse return type annotation (optional)
    let return_type = if parser.peek().kind == TokenKind::Arrow {
        parser.next()?;
        Some(parse_type_annotation(parser)?)
    } else {
        None
    };
    
    // Parse function body
    parser.expect(TokenKind::Colon)?;
    let body = parse_block(parser)?;
    
    let span = Span::merge(name_token.span(), body.last().map_or(name_token.span(), |s| s.span()));
    
    Ok(Stmt::FunctionDef(FunctionDef::new(
        name, params, body, is_async, return_type, span,
    )))
}
```

#### 3.3 Codegen: Async Function Wrapper

**File:** `src/codegen/functions.rs`

```rust
pub fn generate_function_def(
    state: &mut CodeGenState,
    func: &FunctionDef,
) -> Result<()> {
    if func.is_async {
        generate_async_function_wrapper(state, func)
    } else {
        generate_regular_function(state, func)
    }
}

fn generate_async_function_wrapper(
    state: &mut CodeGenState,
    func: &FunctionDef,
) -> Result<()> {
    use inkwell::types::BasicType;
    
    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = state.context.i64_type();
    
    // 1. Create the actual function body (prefixed with __async_body_)
    let body_func_name = format!("__async_body_{}", func.name);
    let body_func_type = i64_type.fn_type(&[ptr_type.into()], false);
    let body_func = state.module.add_function(&body_func_name, body_func_type, None);
    
    // 2. Create wrapper that returns Future
    let wrapper_func_type = ptr_type.fn_type(&func_param_types(state, &func.params), false);
    let wrapper_func = state.module.add_function(
        &mangle_function_name(&func.name, &[]),
        wrapper_func_type,
        None,
    );
    
    // 3. Generate wrapper body
    let entry = state.context.append_basic_block(wrapper_func, "entry");
    let old_block = state.builder.get_insert_block();
    state.builder.position_at_end(entry);
    
    // Create Future
    let future_create = state.module.get_function("vp_future_create").unwrap();
    let future = state.builder.build_call(
        state.builder,
        future_create,
        &[],
        "future",
    ).unwrap().into_pointer_value();
    
    // Create async context (holds args + future)
    let context_type = state.context.struct_type(&[
        ptr_type.into(),  // future
        i64_type.into(),  // arg1 (if any)
        i64_type.into(),  // arg2 (if any)
        // ... more args as needed
    ], false);
    let context_ptr = state.builder.build_alloca(context_type, "context");
    
    // Store future in context
    state.builder.build_store(
        state.builder.build_struct_gep(context_type, context_ptr, 0, "future_gep").unwrap(),
        future,
    );
    
    // Store args in context (simplified - would need proper arg handling)
    // ...
    
    // Spawn fiber to run body
    let async_spawn = state.module.get_function("vp_async_spawn").unwrap();
    let body_func_ptr = body_func.as_global_value().as_pointer_value();
    state.builder.build_call(
        state.builder,
        async_spawn,
        &[body_func_ptr.into(), context_ptr.into()],
        "spawn",
    );
    
    // Return future
    state.builder.build_return(Some(&future));
    
    // Restore builder position
    if let Some(ob) = old_block {
        state.builder.position_at_end(ob);
    }
    
    // 4. Generate body function (calls original logic, sets result)
    generate_async_body_function(state, func, body_func, context_type);
    
    Ok(())
}
```

**Tests:** See [Test Suite](#test-suite) - Test 3

**Estimated Effort:** 6-8 hours  
**Risk:** High - requires careful handling of captured variables and state

---

### Phase 4: Event Loop Integration

**Goal:** Connect event loop with fiber scheduler for async I/O.

#### 4.1 Event Loop Integration in Scheduler

**File:** `runtime/src/scheduler.c`

```c
static void* scheduler_worker(void* arg) {
    SchedulerThread* st = (SchedulerThread*)arg;
    ViperEventLoop* event_loop = vp_event_loop_get_global();
    
    while (1) {
        if (atomic_load(&g_scheduler->shutdown)) {
            break;
        }
        
        ViperFiber* current = scheduler_get_next_fiber(st);
        
        if (current) {
            // Set up jump point for yield return
            sigjmp_buf jump_buf;
            current->sched_jump = &jump_buf;
            
            if (sigsetjmp(jump_buf, 1) == 0) {
                current->state = FIBER_RUNNING;
                current->func(current->arg);
                current->state = FIBER_COMPLETED;
                
                if (current->parent) {
                    vp_scheduler_add_ready(current->parent);
                }
                
                if (current->pool) {
                    vp_fiber_pool_free(current->pool, current);
                } else {
                    vp_fiber_free(current);
                }
            } else {
                // Returned via yield - fiber will be re-scheduled
            }
            
            current->sched_jump = NULL;
            atomic_fetch_add(&st->fibers_run, 1);
            continue;
        }
        
        // No fibers ready - run event loop for async I/O
        if (event_loop && vp_event_loop_pending_ops(event_loop) > 0) {
            vp_event_loop_run(event_loop, 1);  // 1ms timeout
        }
        
        // Wait for work
        scheduler_wait_for_work(st);
    }
    
    return NULL;
}
```

#### 4.2 Async Sleep Example

**File:** `runtime/src/async.c`

```c
typedef struct SleepRequest {
    ViperFiber* waiting_fiber;
    int64_t wake_time;  // nanoseconds since epoch
    bool completed;
} SleepRequest;

int64_t vp_async_sleep(int64_t milliseconds) {
    SleepRequest* req = (SleepRequest*)malloc(sizeof(SleepRequest));
    req->waiting_fiber = vp_fiber_current();
    req->wake_time = vp_current_time_ns() + (milliseconds * 1000000);
    req->completed = false;
    
    // Register with event loop
    vp_event_loop_register_timer(req->wake_time, on_sleep_complete, req);
    
    // Yield until timer fires
    while (!req->completed) {
        vp_fiber_yield();
    }
    
    free(req);
    return 0;
}

static void on_sleep_complete(void* data) {
    SleepRequest* req = (SleepRequest*)data;
    req->completed = true;
    
    if (req->waiting_fiber) {
        req->waiting_fiber->state = FIBER_READY;
        vp_scheduler_add_ready(req->waiting_fiber);
    }
}
```

**Tests:** See [Test Suite](#test-suite) - Test 4

**Estimated Effort:** 4-6 hours  
**Risk:** Medium - integration complexity

---

### Phase 5: Async For Loops

**Goal:** Support `async for` with fiber yielding.

#### 5.1 Runtime: Async Iterator Protocol

**File:** `runtime/src/async.c`

```c
/* Async iterator protocol - follows Python's __aiter__/__anext__ */

void* vp_async_aiter(void* iterable) {
    if (!iterable) return NULL;
    
    // For now, assume iterable is already an async iterator
    // Full implementation would call __aiter__ method
    return iterable;
}

int64_t vp_async_anext(void* iterator) {
    if (!iterator) return -1;  // StopAsyncIteration
    
    // For ViperAsyncRange iterator
    ViperAsyncRange* range = (ViperAsyncRange*)iterator;
    int64_t next_val = vp_async_range_next(range);
    
    if (next_val == -1) {
        // End of iteration
        return -1;
    }
    
    return next_val;
}
```

#### 5.2 Codegen for Async For

**File:** `src/codegen/statements/loops.rs`

```rust
pub fn generate_async_for(
    state: &mut CodeGenState,
    target: &str,
    iter_expr: &Expr,
    body: &[Stmt],
) -> Result<()> {
    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let i64_type = state.context.i64_type();
    
    // Get async iterator
    let iter_val = generate_expr(state, iter_expr)?;
    let aiter_func = state.module.get_function("vp_async_aiter").unwrap();
    let iterator = state.builder.build_call(
        state.builder,
        aiter_func,
        &[iter_val.into()],
        "async_iter",
    ).unwrap();
    
    // Create loop blocks
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let loop_block = state.context.append_basic_block(func, "async_for_loop");
    let body_block = state.context.append_basic_block(func, "async_for_body");
    let end_block = state.context.append_basic_block(func, "async_for_end");
    
    // Branch to loop
    state.builder.build_unconditional_branch(loop_block);
    
    // Loop block: get next item
    state.builder.position_at_end(loop_block);
    let anext_func = state.module.get_function("vp_async_anext").unwrap();
    let item = state.builder.build_call(
        state.builder,
        anext_func,
        &[iterator.into()],
        "async_next",
    ).unwrap().into_int_value();
    
    // Check for StopAsyncIteration (-1)
    let is_end = state.builder.build_int_compare(
        inkwell::IntPredicate::EQ,
        item,
        i64_type.const_int(-1, false),
        "is_end",
    );
    
    state.builder.build_conditional_branch(
        is_end,
        end_block,
        body_block,
    );
    
    // Body block
    state.builder.position_at_end(body_block);
    
    // Store item in target variable
    state.store_variable(target, item.into());
    
    // Generate body statements
    for stmt in body {
        generate_stmt_internal(state, stmt)?;
    }
    
    // Branch back to loop
    state.builder.build_unconditional_branch(loop_block);
    
    // End block
    state.builder.position_at_end(end_block);
    
    // Free iterator
    let free_func = state.module.get_function("vp_async_range_free").unwrap();
    state.builder.build_call(
        state.builder,
        free_func,
        &[iterator.into()],
        "free_iter",
    );
    
    Ok(())
}
```

**Tests:** See [Test Suite](#test-suite) - Test 5

**Estimated Effort:** 3-4 hours  
**Risk:** Low - follows Python's async iterator protocol

---

## Test Suite

### Test 1: Fiber Yield Primitive

**File:** `tests/integration/async_fiber_yield.rs`

```rust
//! Test fiber yield primitive

use viper_lang::driver::run_file;

#[test]
fn test_fiber_yield_basic() {
    let code = r#"
counter = 0

def worker1():
    global counter
    counter = counter + 1
    print("worker1: yield")
    # Implicit yield at end

def worker2():
    global counter
    counter = counter + 1
    print("worker2: yield")

def main():
    global counter
    counter = 0
    task worker1()
    task worker2()
    sync:
        pass
    print("counter:", counter)
"#;
    
    // Should print counter: 2
    let output = run_code(code).unwrap();
    assert!(output.contains("counter: 2"));
}
```

### Test 2: Future Await Yields Fiber

**File:** `tests/integration/async_await_basic.rs`

```rust
//! Test basic async/await with fiber yield

#[test]
fn test_async_def_returns_future() {
    let code = r#"
async def simple():
    return 42

async def main():
    f = simple()
    print("future:", f)
    result = await f
    print("result:", result)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("future:"));
    assert!(output.contains("result: 42"));
}

#[test]
fn test_await_multiple_futures() {
    let code = r#"
async def worker(n):
    return n * 2

async def main():
    f1 = worker(1)
    f2 = worker(2)
    f3 = worker(3)
    
    r1 = await f1
    r2 = await f2
    r3 = await f3
    
    print("results:", r1, r2, r3)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("results: 2 4 6"));
}
```

### Test 3: Async Function Codegen

**File:** `tests/integration/async_codegen.rs`

```rust
//! Test async function code generation

#[test]
fn test_async_with_args() {
    let code = r#"
async def add(a, b):
    return a + b

async def main():
    result = await add(10, 20)
    print("sum:", result)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("sum: 30"));
}

#[test]
fn test_async_concurrent_execution() {
    let code = r#"
async def worker(n):
    # Simulate work
    result = 0
    for i in range(1000):
        result = result + i
    return n * 100 + result

async def main():
    futures = []
    for i in range(10):
        futures.append(worker(i))
    
    total = 0
    for f in futures:
        r = await f
        total = total + r
    
    print("total:", total)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("total:"));
}
```

### Test 4: Async Sleep (Event Loop)

**File:** `tests/integration/async_sleep.rs`

```rust
//! Test async sleep with event loop

#[test]
fn test_async_sleep() {
    let code = r#"
async def sleeper(n):
    await sleep(10)  # 10ms
    return n

async def main():
    futures = []
    for i in range(5):
        futures.append(sleeper(i))
    
    results = []
    for f in futures:
        r = await f
        results.append(r)
    
    print("done:", results)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("done:"));
}

#[test]
fn test_async_sleep_concurrent() {
    let code = r#"
async def sleeper(n):
    await sleep(50)
    return n

async def main():
    # All should complete in ~50ms, not 500ms
    futures = []
    for i in range(10):
        futures.append(sleeper(i))
    
    results = []
    for f in futures:
        results.append(await f)
    
    print("results:", results)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("results:"));
}
```

### Test 5: Async For Loops

**File:** `tests/integration/async_for.rs`

```rust
//! Test async for loops

#[test]
fn test_async_for_range() {
    let code = r#"
async def main():
    total = 0
    async for i in async_range(10):
        total = total + i
    print("total:", total)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("total: 45"));  // 0+1+2+...+9
}

#[test]
fn test_async_for_with_await() {
    let code = r#"
async def process(n):
    await sleep(1)
    return n * 2

async def main():
    results = []
    async for i in async_range(5):
        r = await process(i)
        results.append(r)
    print("results:", results)

main()
"#;
    
    let output = run_code(code).unwrap();
    assert!(output.contains("results: 0 2 4 6 8"));
}
```

### Test 6: Scale Test (10M+ Tasks)

**File:** `tests/integration/async_scale.rs`

```rust
//! Test async/await at scale

#[test]
fn test_async_million_tasks() {
    let code = r#"
async def worker(n):
    return n

async def main():
        futures = []
        for i in range(1_000_000):
            futures.append(worker(i))
        
        total = 0
        for f in futures:
            total = total + await f
        
        print("total:", total)
    
    main()
    "#;
    
    let output = run_code(code).unwrap();
    // Sum of 0..999999 = 499999500000
    assert!(output.contains("total: 499999500000"));
}
```

---

## File Changes Summary

| File | Changes |
|------|---------|
| `runtime/src/fiber.h` | Add `sched_jump`, `waiting_on` fields |
| `runtime/src/fiber.c` | Add `vp_fiber_yield()` |
| `runtime/src/async.c` | Fix `vp_future_await`, add sleep, fix wake |
| `runtime/src/async.h` | Declare new functions |
| `runtime/src/scheduler.c` | Initialize `sched_jump`, event loop integration |
| `src/ast/nodes.rs` | Add `is_async` to `FunctionDef` |
| `src/parser/statements/function.rs` | Parse `async def` |
| `src/codegen/functions.rs` | Async function wrapper |
| `src/codegen/statements/loops.rs` | Async for codegen |
| `src/jit_stubs/concurrency.rs` | JIT stubs for async |
| `tests/integration/async_*.rs` | Test suite |

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
| Multiple awaiters on same Future | Medium | Implement wait list in ViperFuture |

---

## Success Criteria

1. ✅ `async def` returns a Future object
2. ✅ `await` yields fiber (no spin-wait)
3. ✅ Multiple async functions can run concurrently
4. ✅ `async for` loops work correctly
5. ✅ No deadlocks or race conditions in tests
6. ✅ 10M+ async tasks scale (same as task/sync)
7. ✅ `task`/`sync` remains unchanged and working

---

## Future Enhancements

- **Async context managers** (`async with`)
- **Task groups** (structured concurrency) - `async with TaskGroup()`
- **Timeout/cancellation** support - `with_timeout(future, ms)`
- **Async generators** (`async def` with `yield`)
- **Integration with epoll/kqueue** for real async I/O
- **gather()** helper - `await gather(f1, f2, f3)`
