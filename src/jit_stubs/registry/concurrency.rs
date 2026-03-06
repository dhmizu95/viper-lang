//! Concurrency JIT stub registration - Channel, Task, Asyncio, and WaitGroup functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_concurrency_stubs(ee: &ExecutionEngine, module: &Module) {
    // Channel functions
    register_stubs!(ee, module, [
        "vp_chan_create" => super::super::concurrency::vp_chan_create,
        "vp_chan_destroy" => super::super::concurrency::vp_chan_destroy,
        "vp_chan_send" => super::super::concurrency::vp_chan_send,
        "vp_chan_recv" => super::super::concurrency::vp_chan_recv,
    ]);

    // WaitGroup functions
    register_stubs!(ee, module, [
        "vp_waitgroup_create" => super::super::concurrency::vp_waitgroup_create,
        "vp_waitgroup_destroy" => super::super::concurrency::vp_waitgroup_destroy,
        "vp_waitgroup_add" => super::super::concurrency::vp_waitgroup_add,
        "vp_waitgroup_done" => super::super::concurrency::vp_waitgroup_done,
        "vp_waitgroup_wait" => super::super::concurrency::vp_waitgroup_wait,
    ]);

    // Future functions
    register_stubs!(ee, module, [
        "vp_future_await" => super::super::concurrency::vp_future_await,
    ]);

    // Async iteration runtime functions
    register_stubs!(ee, module, [
        "vp_async_range_create" => super::super::concurrency::vp_async_range_create,
        "vp_async_range_next" => super::super::concurrency::vp_async_range_next,
        "vp_async_iter" => super::super::concurrency::vp_async_iter,
        "vp_async_next" => super::super::concurrency::vp_async_next,
        "vp_async_spawn" => super::super::concurrency::vp_async_spawn,
        "vp_async_run_loop" => super::super::concurrency::vp_async_run_loop,
    ]);

    // Thread pool functions
    register_stubs!(ee, module, [
        "vp_init_threadpool" => super::super::concurrency::vp_init_threadpool,
        "vp_shutdown_threadpool" => super::super::concurrency::vp_shutdown_threadpool,
        "vp_submit_task" => super::super::concurrency::vp_submit_task,
        "vp_wait_all_tasks" => super::super::concurrency::vp_wait_all_tasks,
    ]);

    // Asyncio functions
    register_stubs!(ee, module, [
        "vp_asyncio_init" => super::super::asyncio_mod::vp_asyncio_init,
        "vp_asyncio_cleanup" => super::super::asyncio_mod::vp_asyncio_cleanup,
        "vp_asyncio_sleep" => super::super::asyncio_mod::vp_asyncio_sleep,
        "vp_asyncio_create_task" => super::super::asyncio_mod::vp_asyncio_create_task,
        "vp_asyncio_task_done" => super::super::asyncio_mod::vp_asyncio_task_done,
        "vp_asyncio_task_cancelled" => super::super::asyncio_mod::vp_asyncio_task_cancelled,
        "vp_asyncio_task_cancel" => super::super::asyncio_mod::vp_asyncio_task_cancel,
        "vp_asyncio_gather" => super::super::asyncio_mod::vp_asyncio_gather,
        "vp_asyncio_wait" => super::super::asyncio_mod::vp_asyncio_wait,
        "vp_asyncio_run" => super::super::asyncio_mod::vp_asyncio_run,
        "vp_asyncio_stop" => super::super::asyncio_mod::vp_asyncio_stop,
        "vp_asyncio_lock_create" => super::super::asyncio_mod::vp_asyncio_lock_create,
        "vp_asyncio_lock_free" => super::super::asyncio_mod::vp_asyncio_lock_free,
        "vp_asyncio_lock_acquire" => super::super::asyncio_mod::vp_asyncio_lock_acquire,
        "vp_asyncio_lock_release" => super::super::asyncio_mod::vp_asyncio_lock_release,
        "vp_asyncio_event_create" => super::super::asyncio_mod::vp_asyncio_event_create,
        "vp_asyncio_event_free" => super::super::asyncio_mod::vp_asyncio_event_free,
        "vp_asyncio_event_is_set" => super::super::asyncio_mod::vp_asyncio_event_is_set,
        "vp_asyncio_event_set" => super::super::asyncio_mod::vp_asyncio_event_set,
        "vp_asyncio_event_clear" => super::super::asyncio_mod::vp_asyncio_event_clear,
        "vp_asyncio_event_wait" => super::super::asyncio_mod::vp_asyncio_event_wait,
        "vp_asyncio_queue_create" => super::super::asyncio_mod::vp_asyncio_queue_create,
        "vp_asyncio_queue_free" => super::super::asyncio_mod::vp_asyncio_queue_free,
        "vp_asyncio_queue_size" => super::super::asyncio_mod::vp_asyncio_queue_size,
        "vp_asyncio_queue_empty" => super::super::asyncio_mod::vp_asyncio_queue_empty,
        "vp_asyncio_queue_full" => super::super::asyncio_mod::vp_asyncio_queue_full,
        "vp_asyncio_queue_put" => super::super::asyncio_mod::vp_asyncio_queue_put,
        "vp_asyncio_queue_get" => super::super::asyncio_mod::vp_asyncio_queue_get,
        "vp_asyncio_semaphore_create" => super::super::asyncio_mod::vp_asyncio_semaphore_create,
        "vp_asyncio_semaphore_free" => super::super::asyncio_mod::vp_asyncio_semaphore_free,
        "vp_asyncio_semaphore_acquire" => super::super::asyncio_mod::vp_asyncio_semaphore_acquire,
        "vp_asyncio_semaphore_release" => super::super::asyncio_mod::vp_asyncio_semaphore_release,
        "vp_asyncio_timeout_create" => super::super::asyncio_mod::vp_asyncio_timeout_create,
        "vp_asyncio_timeout_free" => super::super::asyncio_mod::vp_asyncio_timeout_free,
        "vp_asyncio_timeout_expired" => super::super::asyncio_mod::vp_asyncio_timeout_expired,
    ]);
}
