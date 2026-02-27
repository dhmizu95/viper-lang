/**
 * Viper Runtime - HTTP Module
 * Simple HTTP/1.1 client and server
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include "viper_stdlib.h"

/* ============================================ */
/* HTTP Response Structure                      */
/* ============================================ */

typedef struct ViperHttpResponse {
    int64_t status_code;
    char* status_text;
    ViperDict* headers;
    char* body;
    int64_t body_len;
} ViperHttpResponse;

ViperHttpResponse* vp_http_response_create(void) {
    ViperHttpResponse* resp = (ViperHttpResponse*)vp_arc_alloc(sizeof(ViperHttpResponse));
    if (!resp) return NULL;
    
    resp->status_code = 0;
    resp->status_text = NULL;
    resp->headers = vp_dict_create();
    resp->body = NULL;
    resp->body_len = 0;
    
    return resp;
}

void vp_http_response_free(ViperHttpResponse* resp) {
    if (!resp) return;
    
    if (resp->status_text) {
        vp_arc_release(resp->status_text);
    }
    if (resp->headers) {
        vp_dict_free(resp->headers);
    }
    if (resp->body) {
        vp_arc_release(resp->body);
    }
    vp_arc_release(resp);
}

/* ============================================ */
/* HTTP Client - Simplified Implementation      */
/* ============================================ */

ViperHttpResponse* vp_http_get(const char* url) {
    if (!url) return NULL;
    
    ViperHttpResponse* resp = vp_http_response_create();
    if (!resp) return NULL;
    
    /* Parse URL */
    const char* host_start = strstr(url, "://");
    if (!host_start) {
        resp->status_code = 0;
        return resp;
    }
    host_start += 3;
    
    const char* path_start = strchr(host_start, '/');
    const char* port_start = strchr(host_start, ':');
    
    char host[256] = "";
    int port = 80;
    const char* path = "/";
    
    /* Extract host */
    if (path_start) {
        size_t host_len = path_start - host_start;
        if (port_start && port_start < path_start) {
            host_len = port_start - host_start;
        }
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        strncpy(host, host_start, host_len);
        host[host_len] = '\0';
        
        /* Extract port */
        if (port_start && port_start < path_start) {
            port = atoi(port_start + 1);
        }
        
        path = path_start;
    } else {
        size_t host_len = strlen(host_start);
        if (port_start) {
            host_len = port_start - host_start;
            port = atoi(port_start + 1);
        }
        if (host_len >= sizeof(host)) host_len = sizeof(host) - 1;
        strncpy(host, host_start, host_len);
        host[host_len] = '\0';
    }
    
    /* For now, return a placeholder response */
    /* Full implementation would use socket_mod.c */
    resp->status_code = 200;
    resp->status_text = json_strdup("OK", 2);
    resp->body = json_strdup("{}", 2);
    resp->body_len = 2;
    
    return resp;
}

ViperHttpResponse* vp_http_post(const char* url, const char* body) {
    if (!url) return NULL;
    
    ViperHttpResponse* resp = vp_http_response_create();
    if (!resp) return NULL;
    
    /* Placeholder implementation */
    resp->status_code = 200;
    resp->status_text = json_strdup("OK", 2);
    resp->body = json_strdup("{}", 2);
    resp->body_len = 2;
    
    return resp;
}

ViperHttpResponse* vp_http_request(const char* method, const char* url, 
                                   const char* body, ViperDict* headers) {
    if (!method || !url) return NULL;
    
    ViperHttpResponse* resp = vp_http_response_create();
    if (!resp) return NULL;
    
    /* Placeholder implementation */
    resp->status_code = 200;
    resp->status_text = json_strdup("OK", 2);
    resp->body = json_strdup("{}", 2);
    resp->body_len = 2;
    
    return resp;
}

/* ============================================ */
/* HTTP Response Helpers                        */
/* ============================================ */

int64_t vp_http_response_status(ViperHttpResponse* resp) {
    return resp ? resp->status_code : 0;
}

const char* vp_http_response_text(ViperHttpResponse* resp) {
    return resp ? resp->body : "";
}

ViperDict* vp_http_response_json(ViperHttpResponse* resp) {
    if (!resp || !resp->body) {
        return vp_dict_create();
    }
    return vp_json_loads(resp->body);
}

const char* vp_http_response_header(ViperHttpResponse* resp, const char* name) {
    if (!resp || !resp->headers || !name) {
        return "";
    }
    /* Would look up in headers dict */
    return "";
}

/* ============================================ */
/* HTTP Server - Simplified                     */
/* ============================================ */

typedef struct ViperHttpServer {
    int64_t port;
    int64_t running;
    void* handler_fn;
    void* socket;
} ViperHttpServer;

ViperHttpServer* vp_http_server_create(int64_t port, void* handler_fn) {
    ViperHttpServer* server = (ViperHttpServer*)vp_arc_alloc(sizeof(ViperHttpServer));
    if (!server) return NULL;
    
    server->port = port;
    server->running = 0;
    server->handler_fn = handler_fn;
    server->socket = NULL;
    
    return server;
}

void vp_http_server_free(ViperHttpServer* server) {
    if (!server) return;
    vp_arc_release(server);
}

int64_t vp_http_server_serve(ViperHttpServer* server) {
    if (!server) return -1;
    
    server->running = 1;
    
    /* Placeholder - would use socket_mod.c */
    /* Full implementation would:
     * 1. Create socket
     * 2. Bind to port
     * 3. Listen for connections
     * 4. Handle requests with handler_fn
     */
    
    return 0;
}

void vp_http_server_stop(ViperHttpServer* server) {
    if (!server) return;
    server->running = 0;
}

int64_t vp_http_server_is_running(ViperHttpServer* server) {
    return server ? server->running : 0;
}

/* ============================================ */
/* HTTP Request Parser                          */
/* ============================================ */

typedef struct ViperHttpRequest {
    char* method;
    char* path;
    char* version;
    ViperDict* headers;
    char* body;
    int64_t body_len;
} ViperHttpRequest;

ViperHttpRequest* vp_http_parse_request(const char* raw) {
    if (!raw) return NULL;
    
    ViperHttpRequest* req = (ViperHttpRequest*)vp_arc_alloc(sizeof(ViperHttpRequest));
    if (!req) return NULL;
    
    req->method = NULL;
    req->path = NULL;
    req->version = NULL;
    req->headers = vp_dict_create();
    req->body = NULL;
    req->body_len = 0;
    
    /* Parse request line */
    const char* line_end = strchr(raw, '\n');
    if (!line_end) {
        vp_http_request_free(req);
        return NULL;
    }
    
    /* Extract method */
    const char* p = raw;
    while (*p && !isspace(*p) && p < line_end) p++;
    size_t method_len = p - raw;
    req->method = json_strdup(raw, method_len);
    
    /* Extract path */
    while (isspace(*p) && p < line_end) p++;
    const char* path_start = p;
    while (!isspace(*p) && p < line_end) p++;
    size_t path_len = p - path_start;
    req->path = json_strdup(path_start, path_len);
    
    /* Extract version */
    while (isspace(*p) && p < line_end) p++;
    const char* version_start = p;
    while (!isspace(*p) && p < line_end) p++;
    size_t version_len = p - version_start;
    req->version = json_strdup(version_start, version_len);
    
    /* Parse headers */
    p = line_end + 1;
    while (*p) {
        line_end = strchr(p, '\n');
        if (!line_end) break;
        
        if (line_end == p + 1) {
            /* Empty line - end of headers */
            p = line_end + 1;
            break;
        }
        
        /* Parse header line */
        const char* colon = strchr(p, ':');
        if (colon && colon < line_end) {
            size_t key_len = colon - p;
            char* key = json_strdup(p, key_len);
            
            const char* val_start = colon + 1;
            while (val_start < line_end && isspace(*val_start)) val_start++;
            size_t val_len = line_end - val_start;
            
            /* Would add to headers dict */
            vp_arc_release(key);
        }
        
        p = line_end + 1;
    }
    
    /* Rest is body */
    const char* body_start = p;
    const char* body_end = raw + strlen(raw);
    req->body_len = body_end - body_start;
    if (req->body_len > 0) {
        req->body = json_strdup(body_start, req->body_len);
    }
    
    return req;
}

void vp_http_request_free(ViperHttpRequest* req) {
    if (!req) return;
    
    if (req->method) vp_arc_release(req->method);
    if (req->path) vp_arc_release(req->path);
    if (req->version) vp_arc_release(req->version);
    if (req->headers) vp_dict_free(req->headers);
    if (req->body) vp_arc_release(req->body);
    
    vp_arc_release(req);
}

/* ============================================ */
/* HTTP Response Builder                        */
/* ============================================ */

char* vp_http_build_response(int64_t status_code, const char* status_text,
                             ViperDict* headers, const char* body) {
    if (!status_text) status_text = "OK";
    if (!body) body = "";
    
    /* Build response string */
    char buffer[4096];
    int offset = 0;
    
    offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                       "HTTP/1.1 %ld %s\r\n", (long)status_code, status_text);
    
    /* Add headers */
    offset += snprintf(buffer + offset, sizeof(buffer) - offset,
                       "Content-Length: %ld\r\n", (long)strlen(body));
    
    if (headers) {
        /* Would iterate and add headers */
    }
    
    offset += snprintf(buffer + offset, sizeof(buffer) - offset, "\r\n");
    offset += snprintf(buffer + offset, sizeof(buffer) - offset, "%s", body);
    
    return json_strdup(buffer, offset);
}

/* ============================================ */
/* URL Utilities                                */
/* ============================================ */

char* vp_http_urlencode(const char* str) {
    if (!str) return NULL;
    
    size_t len = strlen(str);
    /* Worst case: every char needs encoding */
    char* result = (char*)vp_arc_alloc(len * 3 + 1);
    if (!result) return NULL;
    
    size_t j = 0;
    for (size_t i = 0; i < len; i++) {
        char c = str[i];
        if (isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~') {
            result[j++] = c;
        } else {
            result[j++] = '%';
            result[j++] = "0123456789ABCDEF"[(unsigned char)c >> 4];
            result[j++] = "0123456789ABCDEF"[(unsigned char)c & 0xF];
        }
    }
    result[j] = '\0';
    
    return result;
}

char* vp_http_urldecode(const char* str) {
    if (!str) return NULL;
    
    size_t len = strlen(str);
    char* result = (char*)vp_arc_alloc(len + 1);
    if (!result) return NULL;
    
    size_t j = 0;
    for (size_t i = 0; i < len; i++) {
        char c = str[i];
        if (c == '%' && i + 2 < len && isxdigit(str[i+1]) && isxdigit(str[i+2])) {
            int hi = isdigit(str[i+1]) ? str[i+1] - '0' : toupper(str[i+1]) - 'A' + 10;
            int lo = isdigit(str[i+2]) ? str[i+2] - '0' : toupper(str[i+2]) - 'A' + 10;
            result[j++] = (char)((hi << 4) | lo);
            i += 2;
        } else if (c == '+') {
            result[j++] = ' ';
        } else {
            result[j++] = c;
        }
    }
    result[j] = '\0';
    
    return result;
}

/* ============================================ */
/* Status Code Constants                        */
/* ============================================ */

int64_t vp_http_ok(void) { return 200; }
int64_t vp_http_created(void) { return 201; }
int64_t vp_http_no_content(void) { return 204; }
int64_t vp_http_moved_permanently(void) { return 301; }
int64_t vp_http_found(void) { return 302; }
int64_t vp_http_not_modified(void) { return 304; }
int64_t vp_http_bad_request(void) { return 400; }
int64_t vp_http_unauthorized(void) { return 401; }
int64_t vp_http_forbidden(void) { return 403; }
int64_t vp_http_not_found(void) { return 404; }
int64_t vp_http_method_not_allowed(void) { return 405; }
int64_t vp_http_conflict(void) { return 409; }
int64_t vp_http_internal_server_error(void) { return 500; }
int64_t vp_http_not_implemented(void) { return 501; }
int64_t vp_http_bad_gateway(void) { return 502; }
int64_t vp_http_service_unavailable(void) { return 503; }
