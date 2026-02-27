/**
 * Viper Type Definitions
 * Optimized for LLVM IR generation and inlining
 */

#ifndef VIPER_TYPES_H
#define VIPER_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/* ============================================ */
/* Inline Attributes for Performance            */
/* ============================================ */

#ifdef __GNUC__
    #define VIPER_ALWAYS_INLINE static inline __attribute__((always_inline))
    #define VIPER_NEVER_INLINE __attribute__((noinline))
    #define VIPER_LIKELY(x) __builtin_expect(!!(x), 1)
    #define VIPER_UNLIKELY(x) __builtin_expect(!!(x), 0)
#else
    #define VIPER_ALWAYS_INLINE static inline
    #define VIPER_NEVER_INLINE
    #define VIPER_LIKELY(x) (x)
    #define VIPER_UNLIKELY(x) (x)
#endif

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

/* Generic Viper Value - Unified 24-byte layout */
typedef struct {
    ViperTypeTag type;      /* 8 bytes */
    union {
        int64_t as_i64;
        double as_f64;
        bool as_bool;
        char* as_str;
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
    if (VIPER_UNLIKELY(!list || index < 0 || index >= list->length)) {
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
    if (VIPER_UNLIKELY(!list || index < 0 || index >= list->length)) {
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

/* ============================================ */
/* Dictionary (Hash Map) - Unified Layout       */
/* ============================================ */

typedef struct DictEntry {
    char* key;            /* 0:  Key string */
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
/* String (reference counted) - Unified Layout  */
/* ============================================ */

typedef struct {
    int64_t ref_count;    /* 0:  Reference count */
    int64_t length;       /* 8:  String length */
    char data[];          /* 16: Flexible array member */
} ViperString;            /* Total: 16 + length + 1 bytes */

/* Inline string accessors */
VIPER_ALWAYS_INLINE int64_t vp_str_len_inline(ViperString* s) {
    return s ? s->length : 0;
}

VIPER_ALWAYS_INLINE const char* vp_str_data_inline(ViperString* s) {
    return s ? s->data : "";
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
