/**
 * Viper Event Loop - Linux Implementation (epoll)
 *
 * Uses epoll for efficient I/O multiplexing on Linux.
 * Integrates with fiber scheduler for async I/O operations.
 */

#define _GNU_SOURCE
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/epoll.h>
#include <sys/socket.h>
#include <fcntl.h>
#include <errno.h>
#include <stdatomic.h>
#include <time.h>
#include "event_loop.h"
#include "fiber.h"
#include "scheduler.h"

/* ============================================ */
/* Configuration                               */
/* ============================================ */

#define MAX_EVENTS 256
#define INITIAL_FD_CAPACITY 64
#define TIMER_WHEEL_SIZE 1024

/* ============================================ */
/* Internal Structures                         */
/* ============================================ */

typedef struct {
    int fd;
    ViperEventType events;
    ViperIoCallback callback;
    void* user_data;
    ViperFiber* waiting_fiber;  /* Fiber waiting on this fd */
    int result;                  /* Result of I/O operation */
} FdEntry;

typedef struct {
    int64_t expiry;
    ViperTimerCallback callback;
    void* user_data;
    bool active;
} TimerEntry;

struct ViperEventLoop {
    /* epoll file descriptor */
    int epoll_fd;
    
    /* Registered file descriptors */
    FdEntry* fds;
    size_t fd_capacity;
    size_t fd_count;
    
    /* Timer wheel */
    TimerEntry* timers;
    size_t timer_capacity;
    int64_t next_timer_id;
    
    /* Control */
    _Atomic bool running;
    _Atomic int64_t pending_ops;
    
    /* Statistics */
    _Atomic uint64_t events_processed;
    _Atomic uint64_t timers_fired;
};

/* Global event loop singleton */
static ViperEventLoop* g_event_loop = NULL;

/* ============================================ */
/* Helper Functions                            */
/* ============================================ */

int vp_make_nonblocking(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags == -1) return -1;
    return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
}

static int64_t get_time_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (int64_t)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

static FdEntry* find_fd_entry(ViperEventLoop* loop, int fd) {
    for (size_t i = 0; i < loop->fd_count; i++) {
        if (loop->fds[i].fd == fd) {
            return &loop->fds[i];
        }
    }
    return NULL;
}

static FdEntry* get_or_create_fd(ViperEventLoop* loop, int fd);
static void check_timers(ViperEventLoop* loop);

static FdEntry* get_or_create_fd(ViperEventLoop* loop, int fd) {
    FdEntry* entry = find_fd_entry(loop, fd);
    if (entry) return entry;
    
    /* Need to allocate new entry */
    if (loop->fd_count >= loop->fd_capacity) {
        size_t new_cap = loop->fd_capacity * 2;
        FdEntry* new_fds = realloc(loop->fds, sizeof(FdEntry) * new_cap);
        if (!new_fds) return NULL;
        loop->fds = new_fds;
        loop->fd_capacity = new_cap;
    }
    
    entry = &loop->fds[loop->fd_count++];
    entry->fd = fd;
    entry->events = 0;
    entry->callback = NULL;
    entry->user_data = NULL;
    entry->waiting_fiber = NULL;
    entry->result = 0;
    
    return entry;
}

/* ============================================ */
/* Event Loop Implementation                   */
/* ============================================ */

ViperEventLoop* vp_event_loop_create(void) {
    ViperEventLoop* loop = calloc(1, sizeof(ViperEventLoop));
    if (!loop) return NULL;
    
    /* Create epoll instance */
    loop->epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (loop->epoll_fd < 0) {
        free(loop);
        return NULL;
    }
    
    /* Initialize fd table */
    loop->fd_capacity = INITIAL_FD_CAPACITY;
    loop->fds = calloc(loop->fd_capacity, sizeof(FdEntry));
    if (!loop->fds) {
        close(loop->epoll_fd);
        free(loop);
        return NULL;
    }
    
    /* Initialize timer wheel */
    loop->timer_capacity = TIMER_WHEEL_SIZE;
    loop->timers = calloc(loop->timer_capacity, sizeof(TimerEntry));
    if (!loop->timers) {
        free(loop->fds);
        close(loop->epoll_fd);
        free(loop);
        return NULL;
    }
    
    loop->next_timer_id = 1;
    atomic_store(&loop->running, true);
    atomic_store(&loop->pending_ops, 0);
    
    return loop;
}

void vp_event_loop_destroy(ViperEventLoop* loop) {
    if (!loop) return;
    
    atomic_store(&loop->running, false);
    
    /* Close all registered fds */
    for (size_t i = 0; i < loop->fd_count; i++) {
        close(loop->fds[i].fd);
    }
    
    free(loop->fds);
    free(loop->timers);
    close(loop->epoll_fd);
    free(loop);
}

ViperEventLoop* vp_event_loop_get_global(void) {
    if (!g_event_loop) {
        g_event_loop = vp_event_loop_create();
    }
    return g_event_loop;
}

/* ============================================ */
/* I/O Registration                            */
/* ============================================ */

int vp_event_loop_add(ViperEventLoop* loop, int fd, ViperEventType events,
                      ViperIoCallback callback, void* user_data) {
    if (!loop || fd < 0) return -1;
    
    /* Set non-blocking */
    if (vp_make_nonblocking(fd) < 0) {
        return -1;
    }
    
    FdEntry* entry = get_or_create_fd(loop, fd);
    if (!entry) return -1;
    
    entry->callback = callback;
    entry->user_data = user_data;
    entry->events = events;
    
    /* Register with epoll */
    struct epoll_event ev;
    ev.events = 0;
    if (events & VIPER_EVENT_READ) ev.events |= EPOLLIN;
    if (events & VIPER_EVENT_WRITE) ev.events |= EPOLLOUT;
    if (events & VIPER_EVENT_ERROR) ev.events |= EPOLLERR;
    if (events & VIPER_EVENT_HUP) ev.events |= EPOLLHUP;
    ev.data.ptr = entry;
    
    if (epoll_ctl(loop->epoll_fd, EPOLL_CTL_ADD, fd, &ev) < 0) {
        return -1;
    }
    
    return 0;
}

int vp_event_loop_mod(ViperEventLoop* loop, int fd, ViperEventType events) {
    if (!loop || fd < 0) return -1;
    
    FdEntry* entry = find_fd_entry(loop, fd);
    if (!entry) return -1;
    
    entry->events = events;
    
    struct epoll_event ev;
    ev.events = 0;
    if (events & VIPER_EVENT_READ) ev.events |= EPOLLIN;
    if (events & VIPER_EVENT_WRITE) ev.events |= EPOLLOUT;
    if (events & VIPER_EVENT_ERROR) ev.events |= EPOLLERR;
    if (events & VIPER_EVENT_HUP) ev.events |= EPOLLHUP;
    ev.data.ptr = entry;
    
    return epoll_ctl(loop->epoll_fd, EPOLL_CTL_MOD, fd, &ev);
}

int vp_event_loop_del(ViperEventLoop* loop, int fd) {
    if (!loop || fd < 0) return -1;
    
    /* Remove from epoll */
    epoll_ctl(loop->epoll_fd, EPOLL_CTL_DEL, fd, NULL);
    
    /* Remove from fd table */
    for (size_t i = 0; i < loop->fd_count; i++) {
        if (loop->fds[i].fd == fd) {
            /* Shift remaining entries */
            for (size_t j = i; j < loop->fd_count - 1; j++) {
                loop->fds[j] = loop->fds[j + 1];
            }
            loop->fd_count--;
            return 0;
        }
    }
    
    return -1;
}

/* ============================================ */
/* Async I/O Operations with Fiber Integration */
/* ============================================ */

typedef struct {
    ViperEventLoop* loop;
    int fd;
    void* buffer;
    size_t count;
    ViperFiber* fiber;
    int result;
} AsyncIoContext;

static void io_callback_wrapper(int fd, ViperEventType events, void* user_data) {
    AsyncIoContext* ctx = (AsyncIoContext*)user_data;
    
    if (events & VIPER_EVENT_READ) {
        ctx->result = read(fd, ctx->buffer, ctx->count);
    } else if (events & VIPER_EVENT_WRITE) {
        ctx->result = write(fd, ctx->buffer, ctx->count);
    } else {
        ctx->result = -1;
    }
    
    /* Resume the waiting fiber */
    if (ctx->fiber) {
        vp_fiber_resume(ctx->fiber);
    }
    
    atomic_fetch_sub(&ctx->loop->pending_ops, 1);
}

int64_t vp_async_read(int fd, void* buf, size_t count) {
    ViperEventLoop* loop = vp_event_loop_get_global();
    if (!loop) return -1;
    
    FdEntry* entry = find_fd_entry(loop, fd);
    if (!entry) {
        /* Register fd for reading */
        if (vp_event_loop_add(loop, fd, VIPER_EVENT_READ, NULL, NULL) < 0) {
            return -1;
        }
        entry = find_fd_entry(loop, fd);
    }
    
    /* Try non-blocking read first */
    ssize_t n = read(fd, buf, count);
    if (n >= 0) {
        return (int64_t)n;  /* Data already available */
    }
    
    if (errno != EAGAIN && errno != EWOULDBLOCK) {
        return -1;  /* Real error */
    }
    
    /* Need to wait - park current fiber */
    ViperFiber* fiber = vp_fiber_current();
    if (!fiber) return -1;
    
    atomic_fetch_add(&loop->pending_ops, 1);
    
    /* Store fiber info for resumption */
    entry->waiting_fiber = fiber;
    entry->user_data = buf;
    entry->result = 0;
    
    /* Update epoll to watch for read (edge-triggered) */
    struct epoll_event ev;
    ev.events = EPOLLIN | EPOLLET;
    ev.data.ptr = entry;
    epoll_ctl(loop->epoll_fd, EPOLL_CTL_MOD, fd, &ev);
    
    /* Park fiber - will be resumed by event loop */
    vp_fiber_park();
    
    return (int64_t)entry->result;
}

int64_t vp_async_write(int fd, const void* buf, size_t count) {
    ViperEventLoop* loop = vp_event_loop_get_global();
    if (!loop) return -1;
    
    FdEntry* entry = find_fd_entry(loop, fd);
    if (!entry) {
        /* Register fd for writing */
        if (vp_event_loop_add(loop, fd, VIPER_EVENT_WRITE, NULL, NULL) < 0) {
            return -1;
        }
        entry = find_fd_entry(loop, fd);
    }
    
    /* Try non-blocking write first */
    ssize_t n = write(fd, buf, count);
    if (n >= 0) {
        return (int64_t)n;  /* Successfully wrote */
    }
    
    if (errno != EAGAIN && errno != EWOULDBLOCK) {
        return -1;  /* Real error */
    }
    
    /* Need to wait - park current fiber */
    ViperFiber* fiber = vp_fiber_current();
    if (!fiber) return -1;
    
    atomic_fetch_add(&loop->pending_ops, 1);
    
    /* Store fiber info for resumption */
    entry->waiting_fiber = fiber;
    entry->user_data = (void*)buf;
    entry->result = 0;
    
    /* Update epoll to watch for write (edge-triggered) */
    struct epoll_event ev;
    ev.events = EPOLLOUT | EPOLLET;
    ev.data.ptr = entry;
    epoll_ctl(loop->epoll_fd, EPOLL_CTL_MOD, fd, &ev);
    
    /* Park fiber - will be resumed by event loop */
    vp_fiber_park();
    
    return (int64_t)entry->result;
}

/* ============================================ */
/* Event Loop Execution                        */
/* ============================================ */

int vp_event_loop_run(ViperEventLoop* loop, int timeout_ms) {
    if (!loop) return VIPER_EVLOOP_ERROR;
    
    struct epoll_event events[MAX_EVENTS];
    int timeout = timeout_ms;
    
    int nfds = epoll_wait(loop->epoll_fd, events, MAX_EVENTS, timeout);
    if (nfds < 0) {
        if (errno == EINTR) return 0;  /* Interrupted, try again */
        return VIPER_EVLOOP_ERROR;
    }
    
    for (int i = 0; i < nfds; i++) {
        FdEntry* entry = (FdEntry*)events[i].data.ptr;
        ViperEventType viper_events = 0;
        
        if (events[i].events & EPOLLIN) viper_events |= VIPER_EVENT_READ;
        if (events[i].events & EPOLLOUT) viper_events |= VIPER_EVENT_WRITE;
        if (events[i].events & EPOLLERR) viper_events |= VIPER_EVENT_ERROR;
        if (events[i].events & EPOLLHUP) viper_events |= VIPER_EVENT_HUP;
        
        /* Check if a fiber is waiting on this fd */
        if (entry->waiting_fiber && vp_fiber_is_parked(entry->waiting_fiber)) {
            /* Perform the actual I/O operation */
            if (viper_events & VIPER_EVENT_READ) {
                entry->result = (int)read(entry->fd, entry->user_data, 4096);
            } else if (viper_events & VIPER_EVENT_WRITE) {
                entry->result = (int)write(entry->fd, entry->user_data, 4096);
            }
            
            /* Resume the waiting fiber */
            vp_fiber_unpark(entry->waiting_fiber);
            entry->waiting_fiber = NULL;
        } else if (entry->callback) {
            /* Use callback if no fiber waiting */
            entry->callback(entry->fd, viper_events, entry->user_data);
        }
        
        atomic_fetch_add(&loop->events_processed, 1);
    }
    
    /* Check and fire timers */
    check_timers(loop);
    
    return nfds;
}

void vp_event_loop_stop(ViperEventLoop* loop) {
    if (!loop) return;
    atomic_store(&loop->running, false);
}

/* ============================================ */
/* Timer Support                               */
/* ============================================ */

int64_t vp_event_loop_add_timer(ViperEventLoop* loop, int64_t timeout_ms,
                                 ViperTimerCallback callback, void* user_data) {
    if (!loop || !callback) return -1;
    
    /* Find free timer slot */
    int64_t timer_id = -1;
    for (size_t i = 0; i < loop->timer_capacity; i++) {
        if (!loop->timers[i].active) {
            timer_id = loop->next_timer_id++;
            loop->timers[i].expiry = get_time_ms() + timeout_ms;
            loop->timers[i].callback = callback;
            loop->timers[i].user_data = user_data;
            loop->timers[i].active = true;
            break;
        }
    }
    
    return timer_id;
}

int vp_event_loop_cancel_timer(ViperEventLoop* loop, int64_t timer_id) {
    if (!loop) return -1;
    
    for (size_t i = 0; i < loop->timer_capacity; i++) {
        if (loop->timers[i].active && 
            (timer_id < 0 || loop->timers[i].expiry == timer_id)) {
            loop->timers[i].active = false;
            return 0;
        }
    }
    
    return -1;
}

static void check_timers(ViperEventLoop* loop) {
    int64_t now = get_time_ms();
    
    for (size_t i = 0; i < loop->timer_capacity; i++) {
        if (loop->timers[i].active && loop->timers[i].expiry <= now) {
            loop->timers[i].active = false;
            loop->timers[i].callback(loop->timers[i].user_data);
            atomic_fetch_add(&loop->timers_fired, 1);
        }
    }
}

/* ============================================ */
/* Statistics                                  */
/* ============================================ */

uint64_t vp_event_loop_events_processed(ViperEventLoop* loop) {
    return loop ? atomic_load(&loop->events_processed) : 0;
}

uint64_t vp_event_loop_timers_fired(ViperEventLoop* loop) {
    return loop ? atomic_load(&loop->timers_fired) : 0;
}

int64_t vp_event_loop_pending_ops(ViperEventLoop* loop) {
    return loop ? atomic_load(&loop->pending_ops) : 0;
}
