/**
 * Viper Event Loop - Linux Implementation (epoll)
 * 
 * Uses epoll for efficient I/O multiplexing on Linux.
 */

#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/epoll.h>
#include <sys/socket.h>
#include <fcntl.h>
#include <errno.h>
#include <stdatomic.h>
#include "event_loop.h"

/* ============================================ */
/* Internal Structures                         */
/* ============================================ */

typedef struct {
    int fd;
    ViperEventType events;
    ViperIoCallback callback;
    void* user_data;
} FdEntry;

struct ViperEventLoop {
    /* epoll file descriptor */
    int epoll_fd;
    
    /* Registered file descriptors */
    FdEntry* fds;
    size_t fd_capacity;
    size_t fd_count;
    
    /* Timer wheel (simplified) */
    int64_t* timer_ids;
    int64_t* timer_expiry;
    ViperTimerCallback* timer_callbacks;
    void** timer_data;
    size_t timer_capacity;
    size_t timer_count;
    int64_t next_timer_id;
    
    /* Control */
    _Atomic bool running;
    
    /* Statistics */
    _Atomic uint64_t events_processed;
};

/* ============================================ */
/* Helper Functions                           */
/* ============================================ */

static int set_nonblocking(int fd) {
    int flags = fcntl(fd, F_GETFL, 0);
    if (flags == -1) return -1;
    return fcntl(fd, F_SETFL, flags | O_NONBLOCK);
}

static FdEntry* find_fd_entry(ViperEventLoop* loop, int fd) {
    for (size_t i = 0; i < loop->fd_count; i++) {
        if (loop->fds[i].fd == fd) {
            return &loop->fds[i];
        }
    }
    return NULL;
}

/* ============================================ */
/* Event Loop Implementation                  */
/* ============================================ */

ViperEventLoop* vp_event_loop_create(void) {
    ViperEventLoop* loop = (ViperEventLoop*)malloc(sizeof(ViperEventLoop));
    if (!loop) return NULL;
    
    memset(loop, 0, sizeof(ViperEventLoop));
    
    /* Create epoll instance */
    loop->epoll_fd = epoll_create1(EPOLL_CLOEXEC);
    if (loop->epoll_fd < 0) {
        free(loop);
        return NULL;
    }
    
    /* Initialize fd table */
    loop->fd_capacity = 64;
    loop->fds = (FdEntry*)malloc(sizeof(FdEntry) * loop->fd_capacity);
    if (!loop->fds) {
        close(loop->epoll_fd);
        free(loop);
        return NULL;
    }
    
    /* Initialize timer table */
    loop->timer_capacity = 64;
    loop->timer_ids = (int64_t*)malloc(sizeof(int64_t) * loop->timer_capacity);
    loop->timer_expiry = (int64_t*)malloc(sizeof(int64_t) * loop->timer_capacity);
    loop->timer_callbacks = (ViperTimerCallback*)malloc(sizeof(ViperTimerCallback) * loop->timer_capacity);
    loop->timer_data = (void**)malloc(sizeof(void*) * loop->timer_capacity);
    
    if (!loop->timer_ids || !loop->timer_expiry || !loop->timer_callbacks || !loop->timer_data) {
        close(loop->epoll_fd);
        free(loop->fds);
        free(loop->timer_ids);
        free(loop->timer_expiry);
        free(loop->timer_callbacks);
        free(loop->timer_data);
        free(loop);
        return NULL;
    }
    
    atomic_store(&loop->running, true);
    
    return loop;
}

void vp_event_loop_destroy(ViperEventLoop* loop) {
    if (!loop) return;
    
    atomic_store(&loop->running, false);
    
    if (loop->epoll_fd >= 0) {
        close(loop->epoll_fd);
    }
    
    free(loop->fds);
    free(loop->timer_ids);
    free(loop->timer_expiry);
    free(loop->timer_callbacks);
    free(loop->timer_data);
    free(loop);
}

int vp_event_loop_add(ViperEventLoop* loop, int fd, ViperEventType events,
                      ViperIoCallback callback, void* user_data) {
    if (!loop || fd < 0 || !callback) {
        return -1;
    }
    
    /* Grow fd table if needed */
    if (loop->fd_count >= loop->fd_capacity) {
        size_t new_capacity = loop->fd_capacity * 2;
        FdEntry* new_fds = (FdEntry*)realloc(loop->fds, sizeof(FdEntry) * new_capacity);
        if (!new_fds) return -1;
        loop->fds = new_fds;
        loop->fd_capacity = new_capacity;
    }
    
    /* Set non-blocking */
    set_nonblocking(fd);
    
    /* Add to epoll */
    struct epoll_event ev;
    memset(&ev, 0, sizeof(ev));
    
    if (events & VIPER_EVENT_READ) {
        ev.events |= EPOLLIN;
    }
    if (events & VIPER_EVENT_WRITE) {
        ev.events |= EPOLLOUT;
    }
    
    if (epoll_ctl(loop->epoll_fd, EPOLL_CTL_ADD, fd, &ev) < 0) {
        return -1;
    }
    
    /* Store entry */
    loop->fds[loop->fd_count].fd = fd;
    loop->fds[loop->fd_count].events = events;
    loop->fds[loop->fd_count].callback = callback;
    loop->fds[loop->fd_count].user_data = user_data;
    loop->fd_count++;
    
    return 0;
}

int vp_event_loop_mod(ViperEventLoop* loop, int fd, ViperEventType events) {
    if (!loop || fd < 0) {
        return -1;
    }
    
    FdEntry* entry = find_fd_entry(loop, fd);
    if (!entry) {
        return -1;
    }
    
    struct epoll_event ev;
    memset(&ev, 0, sizeof(ev));
    
    if (events & VIPER_EVENT_READ) {
        ev.events |= EPOLLIN;
    }
    if (events & VIPER_EVENT_WRITE) {
        ev.events |= EPOLLOUT;
    }
    
    entry->events = events;
    
    return epoll_ctl(loop->epoll_fd, EPOLL_CTL_MOD, fd, &ev);
}

int vp_event_loop_del(ViperEventLoop* loop, int fd) {
    if (!loop || fd < 0) {
        return -1;
    }
    
    epoll_ctl(loop->epoll_fd, EPOLL_CTL_DEL, fd, NULL);
    
    /* Remove from table */
    for (size_t i = 0; i < loop->fd_count; i++) {
        if (loop->fds[i].fd == fd) {
            /* Swap with last */
            loop->fds[i] = loop->fds[loop->fd_count - 1];
            loop->fd_count--;
            return 0;
        }
    }
    
    return -1;
}

int vp_event_loop_run(ViperEventLoop* loop, int timeout_ms) {
    if (!loop) return -1;
    
    struct epoll_event events[64];
    int nfds = epoll_wait(loop->epoll_fd, events, 64, timeout_ms);
    
    if (nfds < 0) {
        if (errno == EINTR) {
            return 0;  /* Interrupted, not an error */
        }
        return -1;
    }
    
    /* Process events */
    for (int i = 0; i < nfds; i++) {
        int fd = events[i].data.fd;
        ViperEventType event_type = 0;
        
        if (events[i].events & EPOLLIN) {
            event_type |= VIPER_EVENT_READ;
        }
        if (events[i].events & EPOLLOUT) {
            event_type |= VIPER_EVENT_WRITE;
        }
        if (events[i].events & EPOLLERR) {
            event_type |= VIPER_EVENT_ERROR;
        }
        if (events[i].events & EPOLLHUP) {
            event_type |= VIPER_EVENT_HUP;
        }
        
        /* Find and call callback */
        FdEntry* entry = find_fd_entry(loop, fd);
        if (entry && entry->callback) {
            entry->callback(fd, event_type, entry->user_data);
        }
        
        atomic_fetch_add(&loop->events_processed, 1);
    }
    
    return nfds;
}

void vp_event_loop_stop(ViperEventLoop* loop) {
    if (loop) {
        atomic_store(&loop->running, false);
    }
}

/* ============================================ */
/* Global Event Loop                          */
/* ============================================ */

static ViperEventLoop* g_global_event_loop = NULL;

ViperEventLoop* vp_event_loop_get_global(void) {
    if (!g_global_event_loop) {
        g_global_event_loop = vp_event_loop_create();
    }
    return g_global_event_loop;
}

/* ============================================ */
/* Timer Implementation                       */
/* ============================================ */

int64_t vp_event_loop_add_timer(ViperEventLoop* loop, int64_t timeout_ms,
                                ViperTimerCallback callback, void* user_data) {
    if (!loop || !callback) return -1;
    
    /* Grow timer table if needed */
    if (loop->timer_count >= loop->timer_capacity) {
        size_t new_capacity = loop->timer_capacity * 2;
        
        loop->timer_ids = (int64_t*)realloc(loop->timer_ids, sizeof(int64_t) * new_capacity);
        loop->timer_expiry = (int64_t*)realloc(loop->timer_expiry, sizeof(int64_t) * new_capacity);
        loop->timer_callbacks = (ViperTimerCallback*)realloc(loop->timer_callbacks, sizeof(ViperTimerCallback) * new_capacity);
        loop->timer_data = (void**)realloc(loop->timer_data, sizeof(void*) * new_capacity);
        
        if (!loop->timer_ids || !loop->timer_expiry || !loop->timer_callbacks || !loop->timer_data) {
            return -1;
        }
        
        loop->timer_capacity = new_capacity;
    }
    
    int64_t timer_id = loop->next_timer_id++;
    
    loop->timer_ids[loop->timer_count] = timer_id;
    loop->timer_expiry[loop->timer_count] = timeout_ms;  /* Simplified - would use absolute time */
    loop->timer_callbacks[loop->timer_count] = callback;
    loop->timer_data[loop->timer_count] = user_data;
    loop->timer_count++;
    
    return timer_id;
}

int vp_event_loop_cancel_timer(ViperEventLoop* loop, int64_t timer_id) {
    if (!loop) return -1;
    
    for (size_t i = 0; i < loop->timer_count; i++) {
        if (loop->timer_ids[i] == timer_id) {
            /* Swap with last */
            loop->timer_ids[i] = loop->timer_ids[loop->timer_count - 1];
            loop->timer_expiry[i] = loop->timer_expiry[loop->timer_count - 1];
            loop->timer_callbacks[i] = loop->timer_callbacks[loop->timer_count - 1];
            loop->timer_data[i] = loop->timer_data[loop->timer_count - 1];
            loop->timer_count--;
            return 0;
        }
    }
    
    return -1;
}

/* ============================================ */
/* Async I/O                                  */
/* ============================================ */

int vp_make_nonblocking(int fd) {
    return set_nonblocking(fd);
}

int64_t vp_async_read(int fd, void* buf, size_t count) {
    return read(fd, buf, count);
}

int64_t vp_async_write(int fd, const void* buf, size_t count) {
    return write(fd, buf, count);
}
