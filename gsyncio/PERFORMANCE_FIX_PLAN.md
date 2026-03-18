# gsyncio Performance Fix Plan

## Priority 1: Critical Fixes (1-2 days)

### 1. Fix `sleep_ms()` to be Non-Blocking
**File:** `gsyncio/_gsyncio_core.pyx`

```cython
# BEFORE (blocks event loop)
def sleep_ms(int ms):
    import time
    time.sleep(ms / 1000.0)

# AFTER (use asyncio for async context)
def sleep_ms(int ms):
    # Return a future that completes after delay
    # Use timer + fiber park in C
    pass
```

### 2. Fix `spawn()` to Use C Scheduler
**File:** `gsyncio/_gsyncio_core.pyx`

```cython
# BEFORE (creates OS thread)
def spawn(func, *args):
    import threading
    t = threading.Thread(target=lambda: func(*args))
    t.start()
    return t

# AFTER (uses C fiber scheduler)  
def spawn(func, *args):
    # Wrap Python callable for C scheduler
    cdef object wrapper = lambda: func(*args)
    # Use scheduler_spawn with proper Python GIL handling
    return scheduler_spawn(wrapper_wrapper, <void*>wrapper)
```

### 3. Fix Channel to be Non-Blocking
**File:** `gsyncio/core.py`

```python
# BEFORE (blocks with threading.Condition)
async def send(self, value):
    with self._not_full:
        while len(self._queue) >= self._capacity:
            self._not_full.wait()  # BLOCKS!

# AFTER (async wait)
async def send(self, value):
    while not self._channel.send_nowait(value):
        await _async_wait_for_space(self)
```

---

## Priority 2: High Impact Fixes (2-3 days)

### 4. Remove Global Lock in task.py
**File:** `gsyncio/task.py`

```python
# BEFORE (global lock, O(n) removal)
_tasks_lock = threading.Lock()
_active_tasks = []

def task(func):
    with _tasks_lock:
        _active_tasks.append(t)

# AFTER (lock-free with WeakSet)
import weakref
_active_tasks = weakref.WeakSet()

def task(func):
    _active_tasks.add(t)  # No lock needed
```

### 5. Add Fiber-to-Coroutine Bridge
**File:** `gsyncio/_gsyncio_core.pyx`

```cython
# Add function to run Python coroutine on fiber
def run_coroutine_on_fiber(coro):
    """Run Python coroutine on C fiber"""
    cdef Future result_future = Future()
    
    def fiber_runner():
        try:
            # Drive coroutine to completion
            loop = asyncio.new_event_loop()
            result = loop.run_until_complete(coro)
            result_future.set_result(result)
        except Exception as e:
            result_future.set_exception(e)
    
    scheduler_spawn(fiber_runner, NULL)
    return result_future
```

### 6. Fix sync() to Use Fiber Wait
**File:** `gsyncio/task.py`

```python
# BEFORE (blocks with thread.join)
def sync():
    for t in _active_tasks:
        t.join()  # BLOCKS!

# AFTER (uses fiber wait)
def sync():
    # Wait using scheduler_wait_all or fiber-based wait
    scheduler_wait_all()
```

---

## Priority 3: Optimization (3-5 days)

### 7. Remove Redundant channel.py Wrapper
**File:** `gsyncio/channel.py`

Delete the file and export `core.Channel` directly from `__init__.py`.

### 8. Add Fast Path for Completed Futures
**File:** `gsyncio/core.py`

```python
def result(self, timeout=None):
    # Fast path - no lock if already done
    if self._done:
        if self._exception:
            raise self._exception
        return self._result
    
    # Slow path with lock
    with self._lock:
        # ... existing logic
```

### 9. Batch Operations for Channels
**File:** `gsyncio/core.py`

```python
def send_all(self, values):
    """Send multiple values efficiently"""
    for v in values:
        self.send_nowait(v)

def recv_all(self, count):
    """Receive multiple values efficiently"""
    return [self.recv_nowait() for _ in range(count)]
```

---

## Expected Performance After Fixes

| Operation | Current | After Fix | Target |
|-----------|---------|-----------|--------|
| Task spawn | 473μs | ~5μs | 100x faster |
| Sleep (1ms) | 125ms | ~5ms | 25x faster |
| Context switch | 0.47μs | 0.47μs | ✅ Already fast |
| WaitGroup | 6.6ms | ~2ms | 3x faster |
| Overall ratio | 32x | ~2x | 16x improvement |

---

## Testing After Each Fix

1. Run `pytest tests/test_gsyncio.py -v` after each change
2. Run `python benchmark.py` to measure improvement
3. Verify examples still work: `python examples/task_example.py`

---

## Long-Term Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Python Application                      │
├─────────────────────────────────────────────────────────┤
│  gsyncio Python Layer (thin wrappers)                   │
│  - Direct calls to C extension                          │
│  - No threading, no blocking                            │
├─────────────────────────────────────────────────────────┤
│  Cython Layer (_gsyncio_core.pyx)                       │
│  - Python ↔ C bridge with GIL management                │
│  - Coroutine-to-fiber adapter                           │
├─────────────────────────────────────────────────────────┤
│  C Core (csrc/)                                         │
│  ├── fiber.c      - Context switching (0.47μs) ✅       │
│  ├── scheduler.c  - M:N scheduling ✅                   │
│  ├── future.c     - Future implementation ✅            │
│  ├── channel.c    - Channel ops (needs non-blocking) ⚠️ │
│  └── timer.c      - Async timers (MISSING) ❌          │
└─────────────────────────────────────────────────────────┘
```

---

## Files to Modify

1. `gsyncio/_gsyncio_core.pyx` - Critical (spawn, sleep)
2. `gsyncio/core.py` - Critical (Channel blocking)
3. `gsyncio/task.py` - High (lock removal)
4. `gsyncio/async_.py` - Medium (remove wrappers)
5. `gsyncio/channel.py` - Low (delete file)
6. `csrc/channel.c` - Add non-blocking wait support
7. `csrc/timer.c` - NEW FILE (async timers)

---

## Success Criteria

- [ ] All 20 tests pass
- [ ] Task spawn < 10μs (currently 473μs)
- [ ] Sleep overhead < 2x asyncio (currently 32x)
- [ ] Context switch stays < 1μs ✅
- [ ] Examples run without warnings
- [ ] No blocking calls in async paths
