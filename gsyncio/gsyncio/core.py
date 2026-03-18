"""
gsyncio.core - Core interface for gsyncio

This module provides the core interface to gsyncio's fiber-based
concurrency primitives.
"""

import sys
import threading
from typing import Any, Callable, Coroutine, Optional, List
from contextlib import contextmanager

# Try to import the Cython extension, fall back to pure Python
try:
    from ._gsyncio_core import (
        Future,
        Channel,
        WaitGroup,
        init_scheduler,
        shutdown_scheduler,
        get_scheduler_stats,
        spawn,
        sleep_ms,
        current_fiber_id,
        yield_execution,
        num_workers,
    )
    _HAS_CYTHON = True
except ImportError:
    _HAS_CYTHON = False
    # Pure Python fallback implementations
    class Future:
        """Pure Python Future implementation"""
        def __init__(self):
            self._done = False
            self._result = None
            self._exception = None
            self._callbacks = []
            self._lock = threading.Lock()
        
        @property
        def done(self):
            return self._done
        
        @property
        def cancelled(self):
            return False
        
        def result(self, timeout=None):
            with self._lock:
                if not self._done:
                    # In pure Python, we can't really await
                    raise RuntimeError("Future not done")
                if self._exception:
                    raise self._exception
                return self._result
        
        def exception(self, timeout=None):
            with self._lock:
                if not self._done:
                    raise RuntimeError("Future not done")
                return self._exception
        
        def set_result(self, result):
            with self._lock:
                if self._done:
                    raise RuntimeError("Future already completed")
                self._result = result
                self._done = True
                for cb in self._callbacks:
                    cb(self)
        
        def set_exception(self, exc):
            with self._lock:
                if self._done:
                    raise RuntimeError("Future already completed")
                self._exception = exc
                self._done = True
                for cb in self._callbacks:
                    cb(self)
        
        def add_callback(self, cb):
            with self._lock:
                if self._done:
                    cb(self)
                else:
                    self._callbacks.append(cb)
        
        def __await__(self):
            """Make future awaitable in asyncio context"""
            import asyncio
            if not self._done:
                # Create an asyncio event to wait for
                event = asyncio.Event()
                
                def on_done(f):
                    event.set()
                
                self.add_callback(on_done)
                yield from event.wait().__await__()
            return self.result()
    
    class Channel:
        """Pure Python Channel implementation"""
        def __init__(self, capacity=0):
            self._capacity = capacity
            self._queue = []
            self._closed = False
            self._lock = threading.Lock()
            self._not_empty = threading.Condition(self._lock)
            self._not_full = threading.Condition(self._lock)
        
        @property
        def capacity(self):
            return self._capacity
        
        @property
        def size(self):
            with self._lock:
                return len(self._queue)
        
        @property
        def closed(self):
            with self._lock:
                return self._closed
        
        async def send(self, value):
            with self._not_full:
                while len(self._queue) >= self._capacity and self._capacity > 0:
                    if self._closed:
                        raise RuntimeError("Channel closed")
                    self._not_full.wait()
                
                if self._closed:
                    raise RuntimeError("Channel closed")
                
                self._queue.append(value)
                self._not_empty.notify()
        
        async def recv(self):
            with self._not_empty:
                while len(self._queue) == 0:
                    if self._closed:
                        raise StopAsyncIteration("Channel closed")
                    self._not_empty.wait()
                
                value = self._queue.pop(0)
                self._not_full.notify()
                return value
        
        def send_nowait(self, value):
            with self._lock:
                if self._closed:
                    return False
                if len(self._queue) >= self._capacity and self._capacity > 0:
                    return False
                self._queue.append(value)
                self._not_empty.notify()
                return True
        
        def recv_nowait(self):
            with self._lock:
                if len(self._queue) == 0:
                    return None
                value = self._queue.pop(0)
                self._not_full.notify()
                return value
        
        def close(self):
            with self._lock:
                self._closed = True
                self._not_empty.notify_all()
                self._not_full.notify_all()
        
        def __len__(self):
            with self._lock:
                return len(self._queue)
        
        def __bool__(self):
            return not self.closed
    
    class WaitGroup:
        """Pure Python WaitGroup implementation"""
        def __init__(self):
            self._counter = 0
            self._lock = threading.Lock()
            self._condition = threading.Condition(self._lock)
        
        def add(self, delta=1):
            with self._lock:
                self._counter += delta
                if self._counter < 0:
                    raise RuntimeError("WaitGroup counter would be negative")
        
        def done(self):
            with self._lock:
                self._counter -= 1
                if self._counter <= 0:
                    self._condition.notify_all()
        
        async def wait(self):
            with self._lock:
                while self._counter > 0:
                    self._condition.wait()
        
        @property
        def counter(self):
            with self._lock:
                return self._counter
    
    def init_scheduler(num_workers=0, max_fibers=1000000, work_stealing=True):
        """Pure Python scheduler init (no-op)"""
        pass
    
    def shutdown_scheduler(wait=True):
        """Pure Python scheduler shutdown (no-op)"""
        pass
    
    def get_scheduler_stats():
        """Pure Python scheduler stats"""
        return {
            'total_fibers_created': 0,
            'total_fibers_completed': 0,
            'total_context_switches': 0,
            'total_work_steals': 0,
            'current_active_fibers': 0,
            'current_ready_fibers': 0,
        }
    
    def spawn(func, *args):
        """Pure Python spawn using threading"""
        t = threading.Thread(target=func, args=args)
        t.start()
        return t
    
    def sleep_ms(ms):
        """Sleep for milliseconds"""
        import time
        time.sleep(ms / 1000.0)
    
    def current_fiber_id():
        """Get current thread ID (fiber ID in pure Python)"""
        return threading.current_thread().ident
    
    def yield_execution():
        """Yield execution (no-op in pure Python)"""
        pass
    
    def num_workers():
        """Get number of CPU cores"""
        import os
        return os.cpu_count() or 1


__all__ = [
    'Future',
    'Channel', 
    'WaitGroup',
    'init_scheduler',
    'shutdown_scheduler',
    'get_scheduler_stats',
    'spawn',
    'sleep_ms',
    'current_fiber_id',
    'yield_execution',
    'num_workers',
    '_HAS_CYTHON',
]
