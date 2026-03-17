/**
 * Viper Runtime - bytearray Implementation
 * Mutable sequence of bytes (0-255)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "viper_stdlib.h"
#include "viper_arc.h"

static void vp_bytearray_destroy(void* ptr) {
    ViperByteArray* ba = (ViperByteArray*)ptr;
    if (ba && ba->data) {
        free(ba->data);
    }
}

ViperByteArray* vp_bytearray_create(void) {
    ViperByteArray* ba = (ViperByteArray*)vp_arc_alloc(sizeof(ViperByteArray));
    ba->length = 0;
    ba->capacity = BYTEARRAY_INITIAL_CAPACITY;
    ba->data = (uint8_t*)malloc(ba->capacity);
    if (!ba->data) {
        fprintf(stderr, "Failed to allocate bytearray data\n");
        exit(1);
    }
    vp_arc_set_destructor(ba, vp_bytearray_destroy);
    return ba;
}

ViperByteArray* vp_bytearray_create_with_capacity(int64_t cap) {
    ViperByteArray* ba = (ViperByteArray*)vp_arc_alloc(sizeof(ViperByteArray));
    ba->length = 0;
    ba->capacity = cap > 0 ? cap : BYTEARRAY_INITIAL_CAPACITY;
    ba->data = (uint8_t*)malloc(ba->capacity);
    if (!ba->data) {
        fprintf(stderr, "Failed to allocate bytearray data\n");
        exit(1);
    }
    vp_arc_set_destructor(ba, vp_bytearray_destroy);
    return ba;
}

ViperByteArray* vp_bytearray_from_bytes(const uint8_t* bytes, int64_t len) {
    ViperByteArray* ba = vp_bytearray_create_with_capacity(len);
    memcpy(ba->data, bytes, len);
    ba->length = len;
    return ba;
}

void vp_bytearray_free(ViperByteArray* ba) {
    if (!ba) return;
    vp_arc_release(ba);
}

int64_t vp_bytearray_len(ViperByteArray* ba) {
    if (!ba) return 0;
    return ba->length;
}

static void vp_bytearray_ensure_capacity(ViperByteArray* ba, int64_t new_cap) {
    if (ba->capacity >= new_cap) return;
    
    int64_t new_capacity = ba->capacity * 2;
    if (new_capacity < new_cap) new_capacity = new_cap;
    
    uint8_t* new_data = (uint8_t*)realloc(ba->data, new_capacity);
    if (!new_data) {
        fprintf(stderr, "Failed to grow bytearray\n");
        exit(1);
    }
    ba->data = new_data;
    ba->capacity = new_capacity;
}

void vp_bytearray_append(ViperByteArray* ba, int64_t value) {
    if (!ba) return;
    if (value < 0 || value > 255) {
        fprintf(stderr, "bytearray value must be 0-255, got %ld\n", (long)value);
        return;
    }
    vp_bytearray_ensure_capacity(ba, ba->length + 1);
    ba->data[ba->length++] = (uint8_t)value;
}

void vp_bytearray_set(ViperByteArray* ba, int64_t index, int64_t value) {
    if (!ba) return;
    if (index < 0 || index >= ba->length) {
        fprintf(stderr, "bytearray index out of range: %ld\n", (long)index);
        return;
    }
    if (value < 0 || value > 255) {
        fprintf(stderr, "bytearray value must be 0-255, got %ld\n", (long)value);
        return;
    }
    ba->data[index] = (uint8_t)value;
}

int64_t vp_bytearray_get(ViperByteArray* ba, int64_t index) {
    if (!ba) return 0;
    if (index < 0 || index >= ba->length) {
        fprintf(stderr, "bytearray index out of range: %ld\n", (long)index);
        return 0;
    }
    return ba->data[index];
}

void vp_bytearray_extend(ViperByteArray* ba, ViperByteArray* other) {
    if (!ba || !other) return;
    vp_bytearray_ensure_capacity(ba, ba->length + other->length);
    memcpy(ba->data + ba->length, other->data, other->length);
    ba->length += other->length;
}

ViperByteArray* vp_bytearray_slice(ViperByteArray* ba, int64_t start, int64_t end, int64_t step) {
    if (!ba) return NULL;
    
    // Normalize negative indices
    if (start < 0) start = ba->length + start;
    if (end < 0) end = ba->length + end;
    if (start < 0) start = 0;
    if (end > ba->length) end = ba->length;
    if (start >= end) return vp_bytearray_create();
    
    // Calculate result length
    int64_t result_len = (end - start + step - 1) / step;
    if (result_len < 0) result_len = 0;
    
    ViperByteArray* result = vp_bytearray_create_with_capacity(result_len);
    for (int64_t i = start; i < end; i += step) {
        vp_bytearray_append(result, ba->data[i]);
    }
    return result;
}

void vp_bytearray_print(ViperByteArray* ba) {
    if (!ba) {
        printf("bytearray()");
        return;
    }
    printf("bytearray(b\"");
    for (int64_t i = 0; i < ba->length; i++) {
        uint8_t c = ba->data[i];
        if (c >= 32 && c < 127 && c != '"' && c != '\\') {
            putchar(c);
        } else {
            printf("\\x%02x", c);
        }
    }
    printf("\")");
}


/**
 * Repeat bytearray n times
 */
ViperByteArray* vp_bytearray_repeat(ViperByteArray* ba, int64_t count) {
    if (!ba || count <= 0) {
        return vp_bytearray_create();
    }
    
    int64_t orig_len = ba->length;
    int64_t new_len = orig_len * count;
    
    ViperByteArray* result = vp_bytearray_create_with_capacity(new_len);
    
    for (int64_t i = 0; i < count; i++) {
        memcpy(result->data + (i * orig_len), ba->data, orig_len);
    }
    result->length = new_len;
    
    return result;
}
