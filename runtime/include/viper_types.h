/**
 * Viper Type Definitions
 * Optimized for LLVM IR generation and inlining
 */

#ifndef VIPER_TYPES_H
#define VIPER_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>

/* Include optimization macros (branch prediction, inlining, etc.) */
#include "viper_optimize.h"

/* Include ARC memory management */
#include "viper_arc.h"

/* ============================================ */
/* Viper Value Types - Unified Layout           */
/* ============================================ */

typedef enum {
    VIPER_TYPE_NONE = 0,
    VIPER_TYPE_I64,
    VIPER_TYPE_F64,
    VIPER_TYPE_BOOL,
    VIPER_TYPE_STR,
    VIPER_TYPE_LIST,
    VIPER_TYPE_DICT,
    VIPER_TYPE_TUPLE,
    VIPER_TYPE_OBJECT,
} ViperTypeTag;

/* Forward declarations */
typedef struct ViperList ViperList;
typedef struct ViperDict ViperDict;
typedef struct ViperObject ViperObject;
typedef struct ViperString ViperString;

/* Generic Viper Value - Unified 24-byte layout */
typedef struct {
    ViperTypeTag type;      /* 8 bytes */
    union {
        int64_t as_i64;
        double as_f64;
        bool as_bool;
        ViperString* as_str;
        ViperList* as_list;
        ViperDict* as_dict;
        ViperObject* as_object;
    } data;                 /* 8 bytes */
    uint64_t _reserved;     /* 8 bytes padding for alignment */
} ViperValue;               /* Total: 24 bytes */

/* ============================================ */
/* Dynamic List (Array) - Typed                 */
/* ============================================ */

typedef enum {
    VIPER_LIST_I64 = 0,     /* int64_t elements */
    VIPER_LIST_F64,         /* double elements */
    VIPER_LIST_BOOL,        /* bool (int8_t) elements - legacy, use BITVEC */
    VIPER_LIST_I32,         /* int32_t elements */
    VIPER_LIST_I16,         /* int16_t elements */
    VIPER_LIST_I8,          /* int8_t elements */
    VIPER_LIST_GENERIC,     /* void* elements (objects) */
    VIPER_LIST_BITVEC,      /* Bit vector (1 bit per boolean) */
} ViperListType;

/* ViperList - Unified 40-byte layout for predictable LLVM IR */
struct ViperList {
    int64_t ref_count;      /* 0:  Reference count for ARC */
    int64_t length;         /* 8:  Current number of elements */
    int64_t capacity;       /* 16: Allocated capacity (in elements) */
    ViperListType elem_type;/* 24: Element type for type-specific access */
    union {
        int64_t* data_i64;   /* i64 list data */
        double*  data_f64;   /* f64 list data */
        int8_t*  data_bool;  /* bool list data (1 byte per element) - legacy */
        int32_t* data_i32;   /* i32 list data */
        int16_t* data_i16;   /* i16 list data */
        int8_t*  data_i8;    /* i8 list data */
        void**   data_generic; /* generic pointer data */
        uint64_t* data_bitvec; /* bit vector data (1 bit per boolean) */
    } data;                 /* 32: Union of data pointers */
};                          /* Total: 40 bytes */

/* Inline list accessors for LLVM optimization */
VIPER_ALWAYS_INLINE int64_t vp_list_len_inline(ViperList* list) {
    return list ? list->length : 0;
}

VIPER_ALWAYS_INLINE int64_t vp_list_get_inline(ViperList* list, int64_t index) {
    if VIPER_BOUNDS_CHECK_UNLIKELY_FAIL(index, list->length) {
        return 0;
    }
    switch (list->elem_type) {
        case VIPER_LIST_I64: return list->data.data_i64[index];
        case VIPER_LIST_I32: return list->data.data_i32[index];
        case VIPER_LIST_I16: return list->data.data_i16[index];
        case VIPER_LIST_I8:  return list->data.data_i8[index];
        case VIPER_LIST_BOOL: return list->data.data_bool[index];
        default: return 0;
    }
}

VIPER_ALWAYS_INLINE void vp_list_set_inline(ViperList* list, int64_t index, int64_t value) {
    if VIPER_BOUNDS_CHECK_UNLIKELY_FAIL(index, list->length) {
        return;
    }
    switch (list->elem_type) {
        case VIPER_LIST_I64: list->data.data_i64[index] = value; break;
        case VIPER_LIST_I32: list->data.data_i32[index] = (int32_t)value; break;
        case VIPER_LIST_I16: list->data.data_i16[index] = (int16_t)value; break;
        case VIPER_LIST_I8:  list->data.data_i8[index] = (int8_t)value; break;
        case VIPER_LIST_BOOL: list->data.data_bool[index] = (int8_t)value; break;
        default: break;
    }
}

/* Unchecked versions for hot loops - use only when bounds are guaranteed */
VIPER_ALWAYS_INLINE int64_t vp_list_get_unchecked(ViperList* list, int64_t index) {
    switch (list->elem_type) {
        case VIPER_LIST_I64: return list->data.data_i64[index];
        case VIPER_LIST_I32: return list->data.data_i32[index];
        case VIPER_LIST_I16: return list->data.data_i16[index];
        case VIPER_LIST_I8:  return list->data.data_i8[index];
        case VIPER_LIST_BOOL: return list->data.data_bool[index];
        default: return 0;
    }
}

VIPER_ALWAYS_INLINE void vp_list_set_unchecked(ViperList* list, int64_t index, int64_t value) {
    switch (list->elem_type) {
        case VIPER_LIST_I64: list->data.data_i64[index] = value; break;
        case VIPER_LIST_I32: list->data.data_i32[index] = (int32_t)value; break;
        case VIPER_LIST_I16: list->data.data_i16[index] = (int16_t)value; break;
        case VIPER_LIST_I8:  list->data.data_i8[index] = (int8_t)value; break;
        case VIPER_LIST_BOOL: list->data.data_bool[index] = (int8_t)value; break;
        default: break;
    }
}

/* ============================================ */
/* Dictionary (Hash Map) - Unified Layout       */
/* ============================================ */

typedef struct DictEntry {
    ViperString* key;     /* 0:  Key string */
    ViperValue value;     /* 8:  Value (24 bytes) */
    struct DictEntry* next;/* 32: Next entry in chain */
} DictEntry;              /* Total: 40 bytes per entry */

/* ViperDict - Unified 40-byte layout */
struct ViperDict {
    int64_t ref_count;    /* 0:  Reference count */
    int64_t size;         /* 8:  Bucket array size */
    int64_t count;        /* 16: Number of entries */
    DictEntry** buckets;  /* 24: Bucket array pointer */
    uint64_t _reserved;   /* 32: Padding for alignment */
};                        /* Total: 40 bytes */

/* Inline dict accessors */
VIPER_ALWAYS_INLINE int64_t vp_dict_len_inline(ViperDict* dict) {
    return dict ? dict->count : 0;
}

/* ============================================ */
/* String (reference counted) with SSO          */
/* ============================================ */

/* Small String Optimization (SSO) threshold */
#define VIPER_SSO_CAPACITY 15  /* Strings <= 15 chars use inline storage */

/**
 * ViperString with Small String Optimization
 * 
 * Layout for small strings (length <= 15):
 *   - is_sso: 1 (high bit of length field)
 *   - length: 7 bits (0-127, but we use 0-15)
 *   - data: inline storage in union
 * 
 * Layout for large strings (length > 15):
 *   - is_sso: 0
 *   - length: full 64-bit length
 *   - data: heap pointer
 * 
 * Total size: 24 bytes (same for both small and large)
 */
typedef struct ViperString {
    union {
        /* Large string (heap-allocated) */
        struct {
            int64_t ref_count;      /* 0:  Reference count for ARC */
            int64_t length;         /* 8:  String length (positive = heap) */
            char* heap_data;        /* 16: Pointer to heap data */
        } heap;
        
        /* Small string (inline storage, SSO) */
        struct {
            int64_t _unused;        /* 0:  Unused (for alignment) */
            int8_t sso_length;      /* 8:  Length (0-127, stored as-is) */
            char sso_data[15];      /* 9-23: Inline storage (15 bytes) */
        } sso;
    } data;
} ViperString;            /* Total: 24 bytes */

/* SSO flag: high bit of length indicates heap vs inline */
#define VIPER_SSO_FLAG 0x80

/* Check if string uses SSO (inline storage) */
static inline int vp_str_is_sso_inline(ViperString* s) {
    if (!s) return 0;
    return (s->data.heap.length & VIPER_SSO_FLAG) != 0;
}

/* Get string length (works for both SSO and heap) */
static inline int64_t vp_str_len_inline(ViperString* s) {
    if (!s) return 0;
    if (vp_str_is_sso_inline(s)) {
        return s->data.sso.sso_length & ~VIPER_SSO_FLAG;
    }
    return s->data.heap.length;
}

/* Get string data pointer (works for both SSO and heap) */
static inline const char* vp_str_data_inline(ViperString* s) {
    if (!s) return "";
    if (vp_str_is_sso_inline(s)) {
        return s->data.sso.sso_data;
    }
    return s->data.heap.heap_data;
}

/* Get first character from string (returns 0 for empty string) */
static inline int64_t vp_str_get_first_inline(ViperString* s) {
    if (!s) return 0;
    if (vp_str_len_inline(s) == 0) return 0;
    return (int64_t)(unsigned char)vp_str_data_inline(s)[0];
}

/* Create a small string with inline storage */
static inline ViperString* vp_str_create_sso_small(const char* str, int64_t len) {
    ViperString* s = (ViperString*)vp_arc_alloc_local(sizeof(ViperString));
    s->data.heap.ref_count = 1;
    s->data.heap.length = len | VIPER_SSO_FLAG;  /* Set SSO flag */
    memcpy(s->data.sso.sso_data, str, (size_t)len);
    s->data.sso.sso_data[len] = '\0';
    return s;
}

/* Create a large string with heap storage */
static inline ViperString* vp_str_create_heap_large(const char* str, int64_t len) {
    ViperString* s = (ViperString*)vp_arc_alloc(sizeof(ViperString) + (size_t)len + 1);
    s->data.heap.ref_count = 1;
    s->data.heap.length = len;  /* No SSO flag */
    s->data.heap.heap_data = (char*)((char*)s + sizeof(ViperString));
    memcpy(s->data.heap.heap_data, str, (size_t)len + 1);
    return s;
}

/* Create a ViperString (automatically chooses SSO or heap) */
static inline ViperString* vp_str_create(const char* str) {
    if (!str) return NULL;
    int64_t len = (int64_t)strlen(str);
    if (len <= VIPER_SSO_CAPACITY) {
        return vp_str_create_sso_small(str, len);
    }
    return vp_str_create_heap_large(str, len);
}

/* Free a ViperString */
static inline void vp_str_free(ViperString* s) {
    if (s) {
        vp_arc_release(s);
    }
}

/* Check if two strings are equal */
bool vp_str_equals(ViperString* a, ViperString* b);

/* Concatenate two strings */
static inline ViperString* vp_str_concat(ViperString* a, ViperString* b) {
    if (!a && !b) return NULL;
    if (!a) return b;
    if (!b) return a;

    int64_t len_a = vp_str_len_inline(a);
    int64_t len_b = vp_str_len_inline(b);
    int64_t total_len = len_a + len_b;

    const char* data_a = vp_str_data_inline(a);
    const char* data_b = vp_str_data_inline(b);

    /* Allocate result string with exact size needed */
    ViperString* result;
    
    /* Use SSO for small strings, heap for large */
    if (total_len <= VIPER_SSO_CAPACITY) {
        /* Small string optimization */
        result = (ViperString*)vp_arc_alloc_local(sizeof(ViperString));
        if (!result) {
            return NULL;
        }
        result->data.heap.ref_count = 1;
        result->data.heap.length = total_len | VIPER_SSO_FLAG;  /* Set SSO flag */
        memcpy(result->data.sso.sso_data, data_a, (size_t)len_a);
        memcpy(result->data.sso.sso_data + len_a, data_b, (size_t)len_b);
        result->data.sso.sso_data[total_len] = '\0';
    } else {
        /* Large string - allocate with embedded data */
        result = (ViperString*)vp_arc_alloc(sizeof(ViperString) + (size_t)total_len + 1);
        if (!result) {
            return NULL;
        }
        result->data.heap.ref_count = 1;
        result->data.heap.length = total_len;  /* No SSO flag = heap string */
        result->data.heap.heap_data = (char*)((char*)result + sizeof(ViperString));

        /* Copy data from both strings */
        if (len_a > 0) {
            memcpy(result->data.heap.heap_data, data_a, (size_t)len_a);
        }
        if (len_b > 0) {
            memcpy(result->data.heap.heap_data + len_a, data_b, (size_t)len_b);
        }
        result->data.heap.heap_data[total_len] = '\0';
    }

    return result;
}

/* ============================================ */
/* Object (for OOP) - Unified Layout            */
/* ============================================ */

struct ViperObject {
    int64_t ref_count;    /* 0:  Reference count */
    void* vtable;         /* 8:  Virtual method table */
    void* data;           /* 16: Object data */
    uint64_t _reserved;   /* 24: Padding for alignment */
};                        /* Total: 32 bytes */

/* Inline object accessors */
VIPER_ALWAYS_INLINE void* vp_object_data_inline(ViperObject* obj) {
    return obj ? obj->data : NULL;
}

/* ============================================ */
/* Tuple (fixed-size heterogeneous collection)  */
/* ============================================ */

/* ViperTuple - Heap-allocated tuple with tagged values
 * Layout: Header (24 bytes) + elements (8 bytes each)
 * Total: 24 + (size * 8) bytes
 */
typedef struct ViperTuple {
    int64_t ref_count;    /* 0:  Reference count for ARC */
    int64_t size;         /* 8:  Number of elements */
    int64_t* elements;    /* 16: Array of ViperValue (tagged i64 values) */
    uint64_t _reserved;   /* 24: Padding for alignment */
} ViperTuple;             /* Total: 32 bytes header + elements */

/* Inline tuple accessors for LLVM optimization */
VIPER_ALWAYS_INLINE int64_t vp_tuple_len_inline(ViperTuple* tuple) {
    return tuple ? tuple->size : 0;
}

VIPER_ALWAYS_INLINE int64_t vp_tuple_get_inline(ViperTuple* tuple, int64_t index) {
    if (VIPER_UNLIKELY(!tuple || index < 0 || index >= tuple->size)) {
        return 0;
    }
    return tuple->elements[index];
}

VIPER_ALWAYS_INLINE void vp_tuple_set_inline(ViperTuple* tuple, int64_t index, int64_t value) {
    if (VIPER_UNLIKELY(!tuple || index < 0 || index >= tuple->size)) {
        return;
    }
    tuple->elements[index] = value;
}

/* Tuple contains check */
VIPER_ALWAYS_INLINE int64_t vp_tuple_contains_inline(ViperTuple* tuple, int64_t value) {
    if (!tuple) return 0;
    for (int64_t i = 0; i < tuple->size; i++) {
        if (tuple->elements[i] == value) return 1;
    }
    return 0;
}

/* ============================================ */
/* Bounds Check Macros for Release Mode         */
/* ============================================ */

#ifdef NDEBUG
    /* Release mode: skip bounds checks */
    #define VIPER_BOUNDS_CHECK(cond) ((void)0)
    #define VIPER_NULL_CHECK(ptr) ((void)0)
#else
    /* Debug mode: full bounds checks */
    #define VIPER_BOUNDS_CHECK(cond) \
        do { if (!(cond)) { \
            fprintf(stderr, "Bounds check failed at %s:%d\n", __FILE__, __LINE__); \
        } } while(0)
    #define VIPER_NULL_CHECK(ptr) \
        do { if (!(ptr)) { \
            fprintf(stderr, "Null pointer check failed at %s:%d\n", __FILE__, __LINE__); \
        } } while(0)
#endif

#endif /* VIPER_TYPES_H */
