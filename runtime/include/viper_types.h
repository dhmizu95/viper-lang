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
/* Dynamic List (Array)                         */
/* ============================================ */

struct ViperList {
    int64_t ref_count;      /* Reference count for ARC */
    int64_t length;         /* Current number of elements */
    int64_t capacity;       /* Allocated capacity */
    int64_t* data;          /* Element data (i64 for Phase 2) */
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
