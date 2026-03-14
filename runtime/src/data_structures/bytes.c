/**
 * Viper Bytes Implementation
 * Immutable byte sequence with reference counting
 */

#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_stdlib.h"
#include "viper_arc.h"

/* ============================================ */
/* Bytes Functions                              */
/* ============================================ */

static void vp_bytes_destroy(void* ptr) {
    ViperBytes* bytes = (ViperBytes*)ptr;
    if (bytes->data) {
        free(bytes->data);
        bytes->data = NULL;
    }
}

ViperBytes* vp_bytes_create(const uint8_t* data, int64_t len) {
    if (len < 0) {
        vp_panic("Bytes length cannot be negative");
    }
    
    ViperBytes* bytes = (ViperBytes*)vp_arc_alloc(sizeof(ViperBytes));
    if (!bytes) {
        vp_panic("Failed to allocate ViperBytes");
    }
    
    if (len > 0 && data) {
        /* Allocate data using malloc (managed by destructor) */
        bytes->data = (uint8_t*)malloc(len * sizeof(uint8_t));
        if (!bytes->data) {
            vp_arc_release(bytes);
            vp_panic("Failed to allocate bytes data");
        }
        memcpy(bytes->data, data, len);
    } else {
        bytes->data = NULL;
    }
    
    bytes->len = len;
    vp_arc_set_destructor(bytes, vp_bytes_destroy);
    
    return bytes;
}

void vp_bytes_free(ViperBytes* bytes) {
    if (!bytes) return;
    vp_arc_release(bytes);
}

ViperBytes* vp_bytes_concat(ViperBytes* a, ViperBytes* b) {
    if (!a || !b) {
        vp_panic("Cannot concat null bytes");
    }
    
    int64_t new_len = a->len + b->len;
    ViperBytes* result = vp_bytes_create(NULL, new_len);
    
    if (a->len > 0 && a->data) {
        memcpy(result->data, a->data, a->len);
    }
    if (b->len > 0 && b->data) {
        memcpy(result->data + a->len, b->data, b->len);
    }
    
    return result;
}

int64_t vp_bytes_len(ViperBytes* bytes) {
    if (!bytes) {
        vp_panic("Cannot get length of null bytes");
    }
    return bytes->len;
}

uint8_t vp_bytes_get(ViperBytes* bytes, int64_t index) {
    if (!bytes) {
        vp_panic("Cannot get byte from null bytes");
    }
    if (index < 0 || index >= bytes->len) {
        vp_panic("Bytes index out of range");
    }
    return bytes->data[index];
}

void vp_bytes_set(ViperBytes* bytes, int64_t index, uint8_t value) {
    if (!bytes) {
        vp_panic("Cannot set byte in null bytes");
    }
    if (index < 0 || index >= bytes->len) {
        vp_panic("Bytes index out of range");
    }
    bytes->data[index] = value;
}

ViperBytes* vp_bytes_slice(ViperBytes* bytes, int64_t start, int64_t end) {
    if (!bytes) {
        vp_panic("Cannot slice null bytes");
    }
    
    // Handle negative indices
    if (start < 0) start = bytes->len + start;
    if (end < 0) end = bytes->len + end;
    
    // Clamp to valid range
    if (start < 0) start = 0;
    if (end > bytes->len) end = bytes->len;
    if (start > end) start = end;
    
    int64_t slice_len = end - start;
    ViperBytes* result = vp_bytes_create(NULL, slice_len);
    
    if (slice_len > 0 && bytes->data) {
        memcpy(result->data, bytes->data + start, slice_len);
    }
    
    return result;
}

bool vp_bytes_contains(ViperBytes* bytes, uint8_t value) {
    if (!bytes || !bytes->data) {
        return false;
    }
    
    for (int64_t i = 0; i < bytes->len; i++) {
        if (bytes->data[i] == value) {
            return true;
        }
    }
    return false;
}

ViperBytes* vp_bytes_copy(ViperBytes* bytes) {
    if (!bytes) {
        vp_panic("Cannot copy null bytes");
    }
    return vp_bytes_create(bytes->data, bytes->len);
}

void vp_bytes_print(ViperBytes* bytes) {
    if (!bytes) {
        printf("None");
        return;
    }
    
    printf("b\"");
    if (bytes->data) {
        for (int64_t i = 0; i < bytes->len; i++) {
            uint8_t b = bytes->data[i];
            // Print printable ASCII characters directly
            if (b >= 32 && b < 127) {
                if (b == '"' || b == '\\') {
                    putchar('\\');
                }
                putchar(b);
            } else {
                // Print escape sequences for special chars
                switch (b) {
                    case '\n': printf("\\n"); break;
                    case '\t': printf("\\t"); break;
                    case '\r': printf("\\r"); break;
                    default: printf("\\x%02x", b); break;
                }
            }
        }
    }
    printf("\"");
}

int64_t vp_bytes_hash(ViperBytes* bytes) {
    if (!bytes || !bytes->data) {
        return 0;
    }
    
    // FNV-1a hash algorithm (using unsigned arithmetic)
    uint64_t hash = 14695981039346656037ULL;
    for (int64_t i = 0; i < bytes->len; i++) {
        hash ^= bytes->data[i];
        hash *= 1099511628211ULL;
    }
    return (int64_t)hash;
}

bool vp_bytes_equals(ViperBytes* a, ViperBytes* b) {
    if (!a && !b) return true;
    if (!a || !b) return false;
    if (a->len != b->len) return false;
    
    if (a->len == 0) return true;
    
    return memcmp(a->data, b->data, a->len) == 0;
}
