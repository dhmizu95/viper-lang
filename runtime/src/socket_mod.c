/**
 * Viper Runtime - Socket Module
 * POSIX socket wrappers for TCP/UDP networking
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <netdb.h>
#include "viper_stdlib.h"

/* Forward declaration */
static char* json_strdup(const char* s, size_t len);

/* ============================================ */
/* Socket Object                                */
/* ============================================ */

typedef struct ViperSocket {
    int fd;
    int family;      /* AF_INET, AF_INET6 */
    int type;        /* SOCK_STREAM, SOCK_DGRAM */
    int protocol;
    int blocking;
    char* host;
    int port;
} ViperSocket;

/* ============================================ */
/* Socket Creation                              */
/* ============================================ */

ViperSocket* vp_socket_create(int64_t family, int64_t type, int64_t protocol) {
    ViperSocket* sock = (ViperSocket*)vp_arc_alloc(sizeof(ViperSocket));
    if (!sock) return NULL;
    
    int dom = AF_INET;
    switch (family) {
        case 10: dom = AF_INET6; break;  /* AF_INET6 */
        default: dom = AF_INET;
    }
    
    int t = SOCK_STREAM;
    switch (type) {
        case 2: t = SOCK_DGRAM; break;  /* SOCK_DGRAM */
        default: t = SOCK_STREAM;
    }
    
    int proto = (int)protocol;
    if (proto == 0 && t == SOCK_STREAM) proto = IPPROTO_TCP;
    if (proto == 0 && t == SOCK_DGRAM) proto = IPPROTO_UDP;
    
    sock->fd = socket(dom, t, proto);
    if (sock->fd < 0) {
        vp_arc_release(sock);
        return NULL;
    }
    
    sock->family = (int)family;
    sock->type = (int)type;
    sock->protocol = proto;
    sock->blocking = 1;
    sock->host = NULL;
    sock->port = 0;
    
    /* Set default options */
    int reuse = 1;
    setsockopt(sock->fd, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));
    
    return sock;
}

void vp_socket_free(ViperSocket* sock) {
    if (!sock) return;
    
    if (sock->fd >= 0) {
        close(sock->fd);
    }
    if (sock->host) {
        vp_arc_release(sock->host);
    }
    vp_arc_release(sock);
}

/* ============================================ */
/* Socket Options                               */
/* ============================================ */

int64_t vp_socket_setblocking(ViperSocket* sock, int64_t blocking) {
    if (!sock) return -1;
    
    int flags = fcntl(sock->fd, F_GETFL, 0);
    if (flags < 0) return -1;
    
    if (blocking) {
        flags &= ~O_NONBLOCK;
    } else {
        flags |= O_NONBLOCK;
    }
    
    if (fcntl(sock->fd, F_SETFL, flags) < 0) {
        return -1;
    }
    
    sock->blocking = blocking ? 1 : 0;
    return 0;
}

int64_t vp_socket_setsockopt(ViperSocket* sock, int64_t level, int64_t optname, const char* value, int64_t len) {
    if (!sock) return -1;
    
    int lvl = (int)level;
    int opt = (int)optname;
    
    if (level == 6 && optname == 1) {  /* TCP_NODELAY */
        int val = len ? 1 : 0;
        return setsockopt(sock->fd, IPPROTO_TCP, TCP_NODELAY, &val, sizeof(val));
    }
    
    return setsockopt(sock->fd, lvl, opt, value, (socklen_t)len);
}

int64_t vp_socket_getsockopt(ViperSocket* sock, int64_t level, int64_t optname, char* value, int64_t* len) {
    if (!sock || !value || !len) return -1;
    
    socklen_t socklen = (socklen_t)*len;
    int ret = getsockopt(sock->fd, (int)level, (int)optname, value, &socklen);
    *len = (int64_t)socklen;
    return ret;
}

/* ============================================ */
/* Address Binding                              */
/* ============================================ */

int64_t vp_socket_bind(ViperSocket* sock, const char* host, int64_t port) {
    if (!sock || sock->fd < 0) return -1;
    
    struct sockaddr_in addr4;
    struct sockaddr_in6 addr6;
    struct sockaddr* addr;
    socklen_t addrlen;
    
    if (sock->family == 10) {  /* AF_INET6 */
        memset(&addr6, 0, sizeof(addr6));
        addr6.sin6_family = AF_INET6;
        addr6.sin6_port = htons((uint16_t)port);
        
        if (host && strcmp(host, "0.0.0.0") != 0 && strcmp(host, "") != 0) {
            if (inet_pton(AF_INET6, host, &addr6.sin6_addr) != 1) {
                return -1;
            }
        } else {
            addr6.sin6_addr = in6addr_any;
        }
        
        addr = (struct sockaddr*)&addr6;
        addrlen = sizeof(addr6);
    } else {  /* AF_INET */
        memset(&addr4, 0, sizeof(addr4));
        addr4.sin_family = AF_INET;
        addr4.sin_port = htons((uint16_t)port);
        
        if (host && strcmp(host, "0.0.0.0") != 0 && strcmp(host, "") != 0) {
            if (inet_pton(AF_INET, host, &addr4.sin_addr) != 1) {
                return -1;
            }
        } else {
            addr4.sin_addr.s_addr = INADDR_ANY;
        }
        
        addr = (struct sockaddr*)&addr4;
        addrlen = sizeof(addr4);
    }
    
    return bind(sock->fd, addr, addrlen);
}

int64_t vp_socket_listen(ViperSocket* sock, int64_t backlog) {
    if (!sock || sock->fd < 0) return -1;
    return listen(sock->fd, (int)backlog);
}

/* ============================================ */
/* Connection                                   */
/* ============================================ */

int64_t vp_socket_connect(ViperSocket* sock, const char* host, int64_t port) {
    if (!sock || sock->fd < 0 || !host) return -1;
    
    struct sockaddr_in addr4;
    struct sockaddr_in6 addr6;
    struct sockaddr* addr;
    socklen_t addrlen;
    
    if (sock->family == 10) {  /* AF_INET6 */
        memset(&addr6, 0, sizeof(addr6));
        addr6.sin6_family = AF_INET6;
        addr6.sin6_port = htons((uint16_t)port);
        
        if (inet_pton(AF_INET6, host, &addr6.sin6_addr) != 1) {
            /* Try hostname resolution */
            struct addrinfo hints, *res;
            memset(&hints, 0, sizeof(hints));
            hints.ai_family = AF_INET6;
            
            if (getaddrinfo(host, NULL, &hints, &res) != 0) {
                return -1;
            }
            
            memcpy(&addr6.sin6_addr, &((struct sockaddr_in6*)res->ai_addr)->sin6_addr, sizeof(addr6.sin6_addr));
            freeaddrinfo(res);
        }
        
        addr = (struct sockaddr*)&addr6;
        addrlen = sizeof(addr6);
    } else {  /* AF_INET */
        memset(&addr4, 0, sizeof(addr4));
        addr4.sin_family = AF_INET;
        addr4.sin_port = htons((uint16_t)port);
        
        if (inet_pton(AF_INET, host, &addr4.sin_addr) != 1) {
            /* Try hostname resolution */
            struct addrinfo hints, *res;
            memset(&hints, 0, sizeof(hints));
            hints.ai_family = AF_INET;
            
            if (getaddrinfo(host, NULL, &hints, &res) != 0) {
                return -1;
            }
            
            memcpy(&addr4.sin_addr, &((struct sockaddr_in*)res->ai_addr)->sin_addr, sizeof(addr4.sin_addr));
            freeaddrinfo(res);
        }
        
        addr = (struct sockaddr*)&addr4;
        addrlen = sizeof(addr4);
    }
    
    sock->host = json_strdup(host, strlen(host));
    sock->port = (int)port;
    
    return connect(sock->fd, addr, addrlen);
}

ViperSocket* vp_socket_accept(ViperSocket* sock) {
    if (!sock || sock->fd < 0) return NULL;
    
    struct sockaddr_storage addr;
    socklen_t addrlen = sizeof(addr);
    
    int newfd = accept(sock->fd, (struct sockaddr*)&addr, &addrlen);
    if (newfd < 0) {
        return NULL;
    }
    
    ViperSocket* newsock = (ViperSocket*)vp_arc_alloc(sizeof(ViperSocket));
    if (!newsock) {
        close(newfd);
        return NULL;
    }
    
    newsock->fd = newfd;
    newsock->family = sock->family;
    newsock->type = sock->type;
    newsock->protocol = sock->protocol;
    newsock->blocking = sock->blocking;
    newsock->host = NULL;
    newsock->port = 0;
    
    return newsock;
}

/* ============================================ */
/* Send/Receive                                 */
/* ============================================ */

int64_t vp_socket_send(ViperSocket* sock, const char* data, int64_t len) {
    if (!sock || sock->fd < 0 || !data || len <= 0) return -1;
    return send(sock->fd, data, (size_t)len, 0);
}

int64_t vp_socket_sendall(ViperSocket* sock, const char* data, int64_t len) {
    if (!sock || sock->fd < 0 || !data || len <= 0) return -1;
    
    int64_t total = 0;
    while (total < len) {
        int64_t sent = send(sock->fd, data + total, (size_t)(len - total), 0);
        if (sent < 0) {
            return total > 0 ? total : -1;
        }
        total += sent;
    }
    return total;
}

int64_t vp_socket_recv(ViperSocket* sock, char* buffer, int64_t maxlen) {
    if (!sock || sock->fd < 0 || !buffer || maxlen <= 0) return -1;
    return recv(sock->fd, buffer, (size_t)maxlen, 0);
}

int64_t vp_socket_recv_into(ViperSocket* sock, ViperList* buffer) {
    if (!sock || sock->fd < 0 || !buffer) return -1;
    
    char temp[4096];
    int64_t n = recv(sock->fd, temp, sizeof(temp), 0);
    if (n <= 0) return n;
    
    /* Convert bytes to list of integers */
    for (int64_t i = 0; i < n; i++) {
        vp_list_append(buffer, (int64_t)(unsigned char)temp[i]);
    }
    
    return n;
}

/* ============================================ */
/* UDP Functions                                */
/* ============================================ */

int64_t vp_socket_sendto(ViperSocket* sock, const char* data, int64_t len, 
                         const char* host, int64_t port) {
    if (!sock || sock->fd < 0 || !data || len <= 0 || !host) return -1;
    
    struct sockaddr_in addr4;
    struct sockaddr_in6 addr6;
    struct sockaddr* addr;
    socklen_t addrlen;
    
    if (sock->family == 10) {
        memset(&addr6, 0, sizeof(addr6));
        addr6.sin6_family = AF_INET6;
        addr6.sin6_port = htons((uint16_t)port);
        inet_pton(AF_INET6, host, &addr6.sin6_addr);
        addr = (struct sockaddr*)&addr6;
        addrlen = sizeof(addr6);
    } else {
        memset(&addr4, 0, sizeof(addr4));
        addr4.sin_family = AF_INET;
        addr4.sin_port = htons((uint16_t)port);
        inet_pton(AF_INET, host, &addr4.sin_addr);
        addr = (struct sockaddr*)&addr4;
        addrlen = sizeof(addr4);
    }
    
    return sendto(sock->fd, data, (size_t)len, 0, addr, addrlen);
}

int64_t vp_socket_recvfrom(ViperSocket* sock, char* buffer, int64_t maxlen,
                           char* host_out, int64_t* port_out) {
    if (!sock || sock->fd < 0 || !buffer || maxlen <= 0) return -1;
    
    struct sockaddr_storage addr;
    socklen_t addrlen = sizeof(addr);
    
    int64_t n = recvfrom(sock->fd, buffer, (size_t)maxlen, 0, 
                         (struct sockaddr*)&addr, &addrlen);
    
    if (n > 0 && host_out && port_out) {
        if (addr.ss_family == AF_INET6) {
            struct sockaddr_in6* a6 = (struct sockaddr_in6*)&addr;
            inet_ntop(AF_INET6, &a6->sin6_addr, host_out, INET6_ADDRSTRLEN);
            *port_out = ntohs(a6->sin6_port);
        } else {
            struct sockaddr_in* a4 = (struct sockaddr_in*)&addr;
            strcpy(host_out, inet_ntoa(a4->sin_addr));
            *port_out = ntohs(a4->sin_port);
        }
    }
    
    return n;
}

/* ============================================ */
/* Socket Info                                  */
/* ============================================ */

int64_t vp_socket_fileno(ViperSocket* sock) {
    return sock ? sock->fd : -1;
}

int64_t vp_socket_getpeername(ViperSocket* sock, char* host_out, int64_t* port_out) {
    if (!sock || sock->fd < 0) return -1;
    
    struct sockaddr_storage addr;
    socklen_t addrlen = sizeof(addr);
    
    if (getpeername(sock->fd, (struct sockaddr*)&addr, &addrlen) < 0) {
        return -1;
    }
    
    if (addr.ss_family == AF_INET6) {
        struct sockaddr_in6* a6 = (struct sockaddr_in6*)&addr;
        inet_ntop(AF_INET6, &a6->sin6_addr, host_out, INET6_ADDRSTRLEN);
        *port_out = ntohs(a6->sin6_port);
    } else {
        struct sockaddr_in* a4 = (struct sockaddr_in*)&addr;
        strcpy(host_out, inet_ntoa(a4->sin_addr));
        *port_out = ntohs(a4->sin_port);
    }
    
    return 0;
}

int64_t vp_socket_getsockname(ViperSocket* sock, char* host_out, int64_t* port_out) {
    if (!sock || sock->fd < 0) return -1;
    
    struct sockaddr_storage addr;
    socklen_t addrlen = sizeof(addr);
    
    if (getsockname(sock->fd, (struct sockaddr*)&addr, &addrlen) < 0) {
        return -1;
    }
    
    if (addr.ss_family == AF_INET6) {
        struct sockaddr_in6* a6 = (struct sockaddr_in6*)&addr;
        inet_ntop(AF_INET6, &a6->sin6_addr, host_out, INET6_ADDRSTRLEN);
        *port_out = ntohs(a6->sin6_port);
    } else {
        struct sockaddr_in* a4 = (struct sockaddr_in*)&addr;
        strcpy(host_out, inet_ntoa(a4->sin_addr));
        *port_out = ntohs(a4->sin_port);
    }
    
    return 0;
}

void vp_socket_close(ViperSocket* sock) {
    if (!sock) return;
    
    if (sock->fd >= 0) {
        shutdown(sock->fd, SHUT_RDWR);
        close(sock->fd);
        sock->fd = -1;
    }
}

/* ============================================ */
/* Constants                                    */
/* ============================================ */

int64_t vp_socket_af_inet(void) { return AF_INET; }
int64_t vp_socket_af_inet6(void) { return AF_INET6; }
int64_t vp_socket_sock_stream(void) { return SOCK_STREAM; }
int64_t vp_socket_sock_dgram(void) { return SOCK_DGRAM; }
int64_t vp_socket_sol_socket(void) { return SOL_SOCKET; }
int64_t vp_socket_so_reuseaddr(void) { return SO_REUSEADDR; }
int64_t vp_socket_tcp_nodelay(void) { return TCP_NODELAY; }
int64_t vp_socket_shut_rd(void) { return SHUT_RD; }
int64_t vp_socket_shut_wr(void) { return SHUT_WR; }
int64_t vp_socket_shut_RDWR(void) { return SHUT_RDWR; }

/* Helper function */
static char* json_strdup(const char* s, size_t len) {
    char* result = (char*)malloc(len + 1);
    if (result) {
        memcpy(result, s, len);
        result[len] = '\0';
    }
    return result;
}
