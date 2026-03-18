"""
gsyncio.task - Task/Sync model for fire-and-forget parallelism

This module provides the task/sync model for spawning concurrent
tasks that run in parallel and can be waited on collectively.

Usage:
    import gsyncio as gs
    
    def worker(n):
        result = sum(range(n))
        print(f"Worker completed: {result}")
    
    def main():
        # Spawn multiple tasks
        for i in range(10):
            gs.task(worker, 1000)
        
        # Wait for all tasks to complete
        gs.sync()
        print("All tasks completed")
    
    gs.run(main)
"""

import threading
from typing import Callable, Any, List
from .core import spawn, yield_execution, init_scheduler, shutdown_scheduler


# Global task tracking
_tasks_lock = threading.Lock()
_active_tasks: List[threading.Thread] = []


def task(func: Callable, *args, **kwargs) -> threading.Thread:
    """
    Spawn a new task (fire-and-forget parallel work).
    
    Args:
        func: Function to execute
        *args: Arguments to pass to function
        **kwargs: Keyword arguments to pass to function
    
    Returns:
        Thread object representing the task
    """
    def wrapper():
        try:
            func(*args, **kwargs)
        finally:
            with _tasks_lock:
                # Remove from active tasks (use discard to avoid errors)
                current = threading.current_thread()
                if current in _active_tasks:
                    _active_tasks.remove(current)
    
    t = spawn(wrapper)
    
    with _tasks_lock:
        _active_tasks.append(t)
    
    return t


def sync():
    """
    Wait for all spawned tasks to complete.
    
    This blocks until all tasks created with gs.task() have finished.
    """
    global _active_tasks
    
    with _tasks_lock:
        tasks = list(_active_tasks)
    
    for t in tasks:
        t.join()


def sync_timeout(timeout: float) -> bool:
    """
    Wait for all tasks with a timeout.
    
    Args:
        timeout: Maximum time to wait in seconds
    
    Returns:
        True if all tasks completed, False if timeout occurred
    """
    import time
    
    deadline = time.time() + timeout
    
    while True:
        with _tasks_lock:
            if not _active_tasks:
                return True
            tasks = list(_active_tasks)
        
        remaining = deadline - time.time()
        if remaining <= 0:
            return False
        
        # Join one task with remaining timeout
        for t in tasks:
            t.join(timeout=remaining)
            if not t.is_alive():
                with _tasks_lock:
                    if t in _active_tasks:
                        _active_tasks.remove(t)
                break
        
        if time.time() >= deadline:
            with _tasks_lock:
                return len(_active_tasks) == 0


def task_count() -> int:
    """
    Get the number of active tasks.
    
    Returns:
        Number of currently running tasks
    """
    with _tasks_lock:
        return len(_active_tasks)


def run(func: Callable, *args, **kwargs) -> Any:
    """
    Run a function in the gsyncio runtime.
    
    This initializes the scheduler, runs the function, and shuts down.
    
    Args:
        func: Function to run
        *args: Arguments to pass to function
        **kwargs: Keyword arguments to pass to function
    
    Returns:
        Result of the function
    """
    init_scheduler()
    try:
        return func(*args, **kwargs)
    finally:
        shutdown_scheduler(wait=True)


__all__ = ['task', 'sync', 'sync_timeout', 'task_count', 'run']
