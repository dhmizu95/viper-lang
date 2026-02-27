/**
 * Viper Type Definitions
 */

#ifndef VIPER_TYPES_H
#define VIPER_TYPES_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/* ============================================ */
/* Viper Value Types                            */
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

/* Generic Viper Value */
typedef struct {
    ViperTypeTag type;
    union {
        int64_t as_i64;
        double as_f64;
        bool as_bool;
        char* as_str;
        ViperList* as_list;
        ViperDict* as_dict;
        ViperObject* as_object;
    } data;
} ViperValue;

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

struct ViperList {
    int64_t ref_count;      /* Reference count for ARC */
    int64_t length;         /* Current number of elements */
    int64_t capacity;       /* Allocated capacity (in elements) */
    ViperListType elem_type; /* Element type for type-specific access */
    union {
        int64_t* data_i64;   /* i64 list data */
        double*  data_f64;   /* f64 list data */
        int8_t*  data_bool;  /* bool list data (1 byte per element) - legacy */
        int32_t* data_i32;   /* i32 list data */
        int16_t* data_i16;   /* i16 list data */
        int8_t*  data_i8;    /* i8 list data */
        void**   data_generic; /* generic pointer data */
        uint64_t* data_bitvec; /* bit vector data (1 bit per boolean) */
    } data;
};

/* ============================================ */
/* Dictionary (Hash Map) - Phase 2 placeholder  */
/* ============================================ */

typedef struct DictEntry {
    char* key;
    ViperValue value;
    struct DictEntry* next;
} DictEntry;

struct ViperDict {
    int64_t ref_count;
    int64_t size;
    int64_t count;
    DictEntry** buckets;
};

/* ============================================ */
/* String (reference counted)                   */
/* ============================================ */

typedef struct {
    int64_t ref_count;
    int64_t length;
    char data[];  /* Flexible array member */
} ViperString;

/* ============================================ */
/* Object (for OOP - Phase 4)                   */
/* ============================================ */

struct ViperObject {
    int64_t ref_count;
    void* vtable;
    void* data;
};

#endif /* VIPER_TYPES_H */
