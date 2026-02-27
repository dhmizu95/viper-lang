/**
 * Viper Event Loop - Cross-Platform Event Loop
 *
 * Provides async I/O using platform-specific mechanisms:
 * - Linux: epoll + io_uring
 * - macOS/BSD: kqueue
 * - Windows: IOCP
 */

#ifndef VIPER_EVENT_LOOP_H
#define VIPER_EVENT_LOOP_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Event Types                                 */
/* ============================================ */

typedef enum {
    VIPER_EVENT_READ = 1,
    VIPER_EVENT_WRITE = 2,
    VIPER_EVENT_READ_WRITE = 3,
    VIPER_EVENT_ERROR = 4,
    VIPER_EVENT_HUP = 8,
} ViperEventType;

typedef enum {
    VIPER_EVLOOP_OK = 0,
    VIPER_EVLOOP_ERROR = -1,
    VIPER_EVLOOP_TIMEOUT = -2,
    VIPER_EVLOOP_NO_EVENTS = -3,
} ViperEventLoopResult;

/* ============================================ */
/* I/O Callback                               */
/* ============================================ */

typedef void (*ViperIoCallback)(int fd, ViperEventType events, void* user_data);

/* ============================================ */
/* Event Loop                                 */
/* ============================================ */

typedef struct ViperEventLoop ViperEventLoop;

/**
 * Create a new event loop
 * @return New event loop, or NULL on failure
 */
ViperEventLoop* vp_event_loop_create(void);

/**
 * Destroy an event loop
 * @param loop Event loop to destroy
 */
void vp_event_loop_destroy(ViperEventLoop* loop);

/**
 * Add a file descriptor to the event loop
 * @param loop Event loop
 * @param fd File descriptor
 * @param events Event types to monitor
 * @param callback Callback to invoke when events occur
 * @param user_data User data to pass to callback
 * @return 0 on success, -1 on failure
 */
int vp_event_loop_add(ViperEventLoop* loop, int fd, ViperEventType events, 
                      ViperIoCallback callback, void* user_data);

/**
 * Modify events for a file descriptor
 * @param loop Event loop
 * @param fd File descriptor
 * @param events New event types
 * @return 0 on success, -1 on failure
 */
int vp_event_loop_mod(ViperEventLoop* loop, int fd, ViperEventType events);

/**
 * Remove a file descriptor from the event loop
 * @param loop Event loop
 * @param fd File descriptor
 * @return 0 on success, -1 on failure
 */
int vp_event_loop_del(ViperEventLoop* loop, int fd);

/**
 * Run the event loop (blocking)
 * @param loop Event loop
 * @param timeout_ms Timeout in milliseconds (-1 = infinite)
 * @return Number of events processed, or error code
 */
int vp_event_loop_run(ViperEventLoop* loop, int timeout_ms);

/**
 * Stop the event loop
 * @param loop Event loop
 */
void vp_event_loop_stop(ViperEventLoop* loop);

/**
 * Get the global event loop (singleton)
 * @return Global event loop
 */
ViperEventLoop* vp_event_loop_get_global(void);

/* ============================================ */
/* Timer Support                              */
/* ============================================ */

typedef void (*ViperTimerCallback)(void* user_data);

/**
 * Add a timer
 * @param loop Event loop
 * @param timeout_ms Timeout in milliseconds
 * @param callback Callback to invoke
 * @param user_data User data
 * @return Timer ID, or -1 on failure
 */
int64_t vp_event_loop_add_timer(ViperEventLoop* loop, int64_t timeout_ms,
                                 ViperTimerCallback callback, void* user_data);

/**
 * Cancel a timer
 * @param loop Event loop
 * @param timer_id Timer ID
 * @return 0 on success, -1 on failure
 */
int vp_event_loop_cancel_timer(ViperEventLoop* loop, int64_t timer_id);

/* ============================================ */
/* Async I/O Operations                       */
/* ============================================ */

/**
 * Make a file descriptor non-blocking
 * @param fd File descriptor
 * @return 0 on success, -1 on failure
 */
int vp_make_nonblocking(int fd);

/**
 * Read from a file descriptor (async, parks fiber if not ready)
 * @param fd File descriptor
 * @param buf Buffer to read into
 * @param count Number of bytes to read
 * @return Number of bytes read, or -1 on error
 */
int64_t vp_async_read(int fd, void* buf, size_t count);

/**
 * Write to a file descriptor (async, parks fiber if not ready)
 * @param fd File descriptor
 * @param buf Buffer to write from
 * @param count Number of bytes to write
 * @return Number of bytes written, or -1 on error
 */
int64_t vp_async_write(int fd, const void* buf, size_t count);

/* ============================================ */
/* Statistics                                  */
/* ============================================ */

/**
 * Get number of events processed
 * @param loop Event loop
 * @return Events processed
 */
uint64_t vp_event_loop_events_processed(ViperEventLoop* loop);

/**
 * Get number of timers fired
 * @param loop Event loop
 * @return Timers fired
 */
uint64_t vp_event_loop_timers_fired(ViperEventLoop* loop);

/**
 * Get number of pending async operations
 * @param loop Event loop
 * @return Pending operations
 */
int64_t vp_event_loop_pending_ops(ViperEventLoop* loop);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_EVENT_LOOP_H */
