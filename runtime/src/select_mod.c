/**
 * Viper Runtime - Select Module
 * Wrapper for select()/epoll() for I/O multiplexing
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <errno.h>
#include <sys/select.h>
#include <sys/time.h>
#include <sys/types.h>
#include "viper_stdlib.h"

/* ============================================ */
/* File Descriptor Set                          */
/* ============================================ */

typedef struct ViperFdSet {
    fd_set set;
    int max_fd;
    ViperList* fds;
} ViperFdSet;

ViperFdSet* vp_select_fdset_create(void) {
    ViperFdSet* fdset = (ViperFdSet*)vp_arc_alloc(sizeof(ViperFdSet));
    if (!fdset) return NULL;
    
    FD_ZERO(&fdset->set);
    fdset->max_fd = -1;
    fdset->fds = vp_list_create();
    
    return fdset;
}

void vp_select_fdset_free(ViperFdSet* fdset) {
    if (!fdset) return;
    
    if (fdset->fds) {
        vp_list_free(fdset->fds);
    }
    vp_arc_release(fdset);
}

void vp_select_fdset_add(ViperFdSet* fdset, int64_t fd) {
    if (!fdset || fd < 0) return;
    
    FD_SET(fd, &fdset->set);
    if ((int)fd > fdset->max_fd) {
        fdset->max_fd = (int)fd;
    }
    vp_list_append(fdset->fds, fd);
}

void vp_select_fdset_remove(ViperFdSet* fdset, int64_t fd) {
    if (!fdset || fd < 0) return;
    
    FD_CLR(fd, &fdset->set);
    
    /* Update max_fd */
    fdset->max_fd = -1;
    for (int64_t i = 0; i < vp_list_len(fdset->fds); i++) {
        int64_t f = vp_list_get(fdset->fds, i);
        if (FD_ISSET(f, &fdset->set) && (int)f > fdset->max_fd) {
            fdset->max_fd = (int)f;
        }
    }
}

int64_t vp_select_fdset_contains(ViperFdSet* fdset, int64_t fd) {
    if (!fdset || fd < 0) return 0;
    return FD_ISSET(fd, &fdset->set) ? 1 : 0;
}

void vp_select_fdset_clear(ViperFdSet* fdset) {
    if (!fdset) return;
    
    FD_ZERO(&fdset->set);
    fdset->max_fd = -1;
    vp_list_clear(fdset->fds);
}

ViperList* vp_select_fdset_get_fds(ViperFdSet* fdset) {
    return fdset ? fdset->fds : NULL;
}

/* ============================================ */
/* Select Function                              */
/* ============================================ */

typedef struct ViperSelectResult {
    ViperList* readable;
    ViperList* writable;
    ViperList* error;
    int64_t count;
} ViperSelectResult;

ViperSelectResult* vp_select_select(ViperFdSet* read_fds, 
                                    ViperFdSet* write_fds,
                                    ViperFdSet* error_fds,
                                    double timeout) {
    ViperSelectResult* result = (ViperSelectResult*)vp_arc_alloc(sizeof(ViperSelectResult));
    if (!result) return NULL;
    
    result->readable = vp_list_create();
    result->writable = vp_list_create();
    result->error = vp_list_create();
    result->count = 0;
    
    /* Prepare fd_set copies */
    fd_set read_set, write_set, err_set;
    int max_fd = -1;
    
    if (read_fds) {
        read_set = read_fds->set;
        if (read_fds->max_fd > max_fd) {
            max_fd = read_fds->max_fd;
        }
    }
    
    if (write_fds) {
        write_set = write_fds->set;
        if (write_fds->max_fd > max_fd) {
            max_fd = write_fds->max_fd;
        }
    }
    
    if (error_fds) {
        err_set = error_fds->set;
        if (error_fds->max_fd > max_fd) {
            max_fd = error_fds->max_fd;
        }
    }
    
    /* Prepare timeout */
    struct timeval tv;
    struct timeval* tvp = NULL;
    
    if (timeout >= 0) {
        tv.tv_sec = (time_t)timeout;
        tv.tv_usec = (suseconds_t)((timeout - (double)tv.tv_sec) * 1e6);
        tvp = &tv;
    }
    
    /* Call select */
    int ret = select(max_fd + 1,
                     read_fds ? &read_set : NULL,
                     write_fds ? &write_set : NULL,
                     error_fds ? &err_set : NULL,
                     tvp);
    
    if (ret < 0) {
        result->count = -1;
        return result;
    }
    
    result->count = ret;
    
    /* Collect readable fds */
    if (read_fds && ret > 0) {
        for (int64_t i = 0; i < vp_list_len(read_fds->fds); i++) {
            int64_t fd = vp_list_get(read_fds->fds, i);
            if (FD_ISSET(fd, &read_set)) {
                vp_list_append(result->readable, fd);
            }
        }
    }
    
    /* Collect writable fds */
    if (write_fds && ret > 0) {
        for (int64_t i = 0; i < vp_list_len(write_fds->fds); i++) {
            int64_t fd = vp_list_get(write_fds->fds, i);
            if (FD_ISSET(fd, &write_set)) {
                vp_list_append(result->writable, fd);
            }
        }
    }
    
    /* Collect error fds */
    if (error_fds && ret > 0) {
        for (int64_t i = 0; i < vp_list_len(error_fds->fds); i++) {
            int64_t fd = vp_list_get(error_fds->fds, i);
            if (FD_ISSET(fd, &err_set)) {
                vp_list_append(result->error, fd);
            }
        }
    }
    
    return result;
}

void vp_select_result_free(ViperSelectResult* result) {
    if (!result) return;
    
    if (result->readable) vp_list_free(result->readable);
    if (result->writable) vp_list_free(result->writable);
    if (result->error) vp_list_free(result->error);
    
    vp_arc_release(result);
}

/* ============================================ */
/* Poll (alternative to select)                 */
/* ============================================ */

#ifdef HAVE_POLL

#include <poll.h>

typedef struct ViperPollFd {
    int64_t fd;
    int16_t events;
    int16_t revents;
} ViperPollFd;

typedef struct ViperPollResult {
    ViperList* fds;
    int64_t count;
} ViperPollResult;

ViperPollResult* vp_poll_poll(ViperList* pollfds, double timeout) {
    ViperPollResult* result = (ViperPollResult*)vp_arc_alloc(sizeof(ViperPollResult));
    if (!result) return NULL;
    
    result->fds = vp_list_create();
    result->count = 0;
    
    int64_t n = vp_list_len(pollfds);
    if (n <= 0) {
        return result;
    }
    
    struct pollfd* pfds = malloc(n * sizeof(struct pollfd));
    if (!pfds) {
        return result;
    }
    
    /* Copy pollfds */
    for (int64_t i = 0; i < n; i++) {
        ViperPollFd* vpf = (ViperPollFd*)vp_list_get(pollfds, i);
        pfds[i].fd = (int)vpf->fd;
        pfds[i].events = (short)vpf->events;
        pfds[i].revents = 0;
    }
    
    /* Call poll */
    int ret = poll(pfds, n, timeout < 0 ? -1 : (int)(timeout * 1000));
    
    if (ret < 0) {
        result->count = -1;
        free(pfds);
        return result;
    }
    
    result->count = ret;
    
    /* Collect results */
    for (int64_t i = 0; i < n; i++) {
        if (pfds[i].revents != 0) {
            ViperPollFd* rpf = (ViperPollFd*)vp_arc_alloc(sizeof(ViperPollFd));
            if (rpf) {
                rpf->fd = pfds[i].fd;
                rpf->events = pfds[i].events;
                rpf->revents = pfds[i].revents;
                vp_list_append(result->fds, (int64_t)rpf);
            }
        }
    }
    
    free(pfds);
    return result;
}

void vp_poll_result_free(ViperPollResult* result) {
    if (!result) return;
    
    if (result->fds) {
        for (int64_t i = 0; i < vp_list_len(result->fds); i++) {
            ViperPollFd* pf = (ViperPollFd*)vp_list_get(result->fds, i);
            if (pf) vp_arc_release(pf);
        }
        vp_list_free(result->fds);
    }
    
    vp_arc_release(result);
}

#endif /* HAVE_POLL */

/* ============================================ */
/* Epoll (Linux-specific)                       */
/* ============================================ */

#ifdef __linux__

#include <sys/epoll.h>

typedef struct ViperEpoll {
    int epfd;
    int64_t max_events;
} ViperEpoll;

ViperEpoll* vp_epoll_create(void) {
    ViperEpoll* ep = (ViperEpoll*)vp_arc_alloc(sizeof(ViperEpoll));
    if (!ep) return NULL;
    
    ep->epfd = epoll_create1(0);
    if (ep->epfd < 0) {
        vp_arc_release(ep);
        return NULL;
    }
    
    ep->max_events = 1024;
    
    return ep;
}

void vp_epoll_free(ViperEpoll* ep) {
    if (!ep) return;
    
    if (ep->epfd >= 0) {
        close(ep->epfd);
    }
    vp_arc_release(ep);
}

int64_t vp_epoll_ctl(ViperEpoll* ep, int64_t op, int64_t fd, uint32_t events) {
    if (!ep || ep->epfd < 0) return -1;
    
    struct epoll_event ev;
    ev.events = events;
    ev.data.fd = (int)fd;
    
    return epoll_ctl(ep->epfd, (int)op, (int)fd, &ev);
}

ViperList* vp_epoll_wait(ViperEpoll* ep, int64_t timeout_ms) {
    ViperList* result = vp_list_create();
    
    if (!ep || ep->epfd < 0) {
        return result;
    }
    
    int max_events = (int)ep->max_events;
    struct epoll_event* events = malloc(max_events * sizeof(struct epoll_event));
    if (!events) {
        return result;
    }
    
    int n = epoll_wait(ep->epfd, events, max_events, (int)timeout_ms);
    
    if (n > 0) {
        for (int i = 0; i < n; i++) {
            vp_list_append(result, (int64_t)events[i].data.fd);
        }
    }
    
    free(events);
    return result;
}

/* Epoll event constants */
int64_t vp_epollin(void) { return EPOLLIN; }
int64_t vp_epollout(void) { return EPOLLOUT; }
int64_t vp_epollerr(void) { return EPOLLERR; }
int64_t vp_epollhup(void) { return EPOLLHUP; }
int64_t vp_epollet(void) { return EPOLLET; }

/* Epoll control operations */
int64_t vp_epoll_ctl_add(void) { return EPOLL_CTL_ADD; }
int64_t vp_epoll_ctl_mod(void) { return EPOLL_CTL_MOD; }
int64_t vp_epoll_ctl_del(void) { return EPOLL_CTL_DEL; }

#endif /* __linux__ */

/* ============================================ */
/* Convenience Functions                        */
/* ============================================ */

int64_t vp_select_can_read(int64_t fd, double timeout) {
    ViperFdSet* read_fds = vp_select_fdset_create();
    if (!read_fds) return 0;
    
    vp_select_fdset_add(read_fds, fd);
    
    ViperSelectResult* result = vp_select_select(read_fds, NULL, NULL, timeout);
    int64_t can_read = 0;
    
    if (result && result->count > 0 && vp_list_len(result->readable) > 0) {
        can_read = 1;
    }
    
    if (result) vp_select_result_free(result);
    vp_select_fdset_free(read_fds);
    
    return can_read;
}

int64_t vp_select_can_write(int64_t fd, double timeout) {
    ViperFdSet* write_fds = vp_select_fdset_create();
    if (!write_fds) return 0;
    
    vp_select_fdset_add(write_fds, fd);
    
    ViperSelectResult* result = vp_select_select(NULL, write_fds, NULL, timeout);
    int64_t can_write = 0;
    
    if (result && result->count > 0 && vp_list_len(result->writable) > 0) {
        can_write = 1;
    }
    
    if (result) vp_select_result_free(result);
    vp_select_fdset_free(write_fds);
    
    return can_write;
}

/* ============================================ */
/* Error Handling                               */
/* ============================================ */

int64_t vp_select_get_error(void) {
    return errno;
}

const char* vp_select_strerror(int64_t err) {
    return strerror((int)err);
}
