/**
 * Viper Runtime - Main Entry Point
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdarg.h>
#include <ctype.h>
#include "viper_stdlib.h"


/* ============================================ */
/* Basic I/O Functions                          */
/* ============================================ */

void vp_print_i64(int64_t val) {
    printf("%ld", (long)val);
}

void vp_print_f64(double val) {
    printf("%g", val);
}

void vp_print_str(const char* val) {
    if (val) {
        printf("%s", val);
    } else {
        printf("(null)");
    }
}

void vp_print_bool(bool val) {
    printf("%s", val ? "True" : "False");
}

void vp_print_newline(void) {
    printf("\n");
}

/* ============================================ */
/* String Functions                             */
/* ============================================ */

char* vp_str_create(const char* str) {
    if (!str) {
        return NULL;
    }

    size_t len = strlen(str);
    char* new_str = (char*)vp_arc_alloc(len + 1);
    strcpy(new_str, str);
    return new_str;
}

/* ============================================ */
/* String Interning                             */
/* ============================================ */

/* Open-addressing intern table with FNV-1a hashing.
 * Slots: 0 = empty, non-NULL = occupied.
 * The table is sized to a power-of-two and doubles on load factor > 0.75. */

#define INTERN_INITIAL_CAP 256

typedef struct {
    const char** slots;   /* permanent C-string storage */
    size_t       cap;
    size_t       count;
} InternTable;

static InternTable g_intern_table = { NULL, 0, 0 };

/* FNV-1a hash (matches vp_hash_str used elsewhere). */
static uint64_t intern_hash(const char* s) {
    uint64_t h = 14695981039346656037ULL;
    while (*s) { h ^= (uint8_t)*s++; h *= 1099511628211ULL; }
    return h;
}

static void intern_table_ensure_init(void) {
    if (g_intern_table.slots) return;
    g_intern_table.cap   = INTERN_INITIAL_CAP;
    g_intern_table.count = 0;
    g_intern_table.slots = (const char**)calloc(INTERN_INITIAL_CAP, sizeof(char*));
}

/* Grow when load factor exceeds 0.75 */
static void intern_table_grow(void) {
    size_t old_cap   = g_intern_table.cap;
    const char** old = g_intern_table.slots;
    size_t new_cap   = old_cap * 2;
    const char** fresh = (const char**)calloc(new_cap, sizeof(char*));

    for (size_t i = 0; i < old_cap; i++) {
        if (!old[i]) continue;
        uint64_t h   = intern_hash(old[i]);
        size_t   idx = (size_t)(h & (new_cap - 1));
        while (fresh[idx]) idx = (idx + 1) & (new_cap - 1);
        fresh[idx] = old[i];
    }
    free(old);
    g_intern_table.slots = fresh;
    g_intern_table.cap   = new_cap;
}

char* vp_str_intern(const char* str) {
    if (!str) return NULL;
    intern_table_ensure_init();

    /* Grow before insert if load > 75 % */
    if (g_intern_table.count * 4 >= g_intern_table.cap * 3)
        intern_table_grow();

    uint64_t h   = intern_hash(str);
    size_t   cap = g_intern_table.cap;
    size_t   idx = (size_t)(h & (cap - 1));

    while (g_intern_table.slots[idx]) {
        if (strcmp(g_intern_table.slots[idx], str) == 0)
            return (char*)g_intern_table.slots[idx];  /* cache hit */
        idx = (idx + 1) & (cap - 1);
    }

    /* Miss: allocate a permanent copy (plain malloc — never freed). */
    size_t len = strlen(str);
    char* copy = (char*)malloc(len + 1);
    strcpy(copy, str);
    g_intern_table.slots[idx] = copy;
    g_intern_table.count++;
    return copy;
}

void vp_str_intern_cleanup(void) {
    if (!g_intern_table.slots) return;
    for (size_t i = 0; i < g_intern_table.cap; i++) {
        if (g_intern_table.slots[i]) {
            free((void*)g_intern_table.slots[i]);
            g_intern_table.slots[i] = NULL;
        }
    }
    free(g_intern_table.slots);
    g_intern_table.slots = NULL;
    g_intern_table.cap   = 0;
    g_intern_table.count = 0;
}

void vp_str_free(char* str) {
    if (str) {
        vp_arc_release(str);
    }
}

char* vp_str_concat(const char* a, const char* b) {
    if (!a || !b) return NULL;

    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    size_t total = len_a + len_b + 1;

    char* result = (char*)vp_arc_alloc(total);
    strcpy(result, a);
    strcat(result, b);

    return result;
}

int64_t vp_str_len(const char* str) {
    if (!str) return 0;
    return (int64_t)strlen(str);
}

char* vp_str_slice(const char* str, int64_t start, int64_t end) {
    if (!str) return NULL;

    int64_t len = (int64_t)strlen(str);

    /* Handle negative indices */
    if (start < 0) start = len + start;
    if (end < 0) end = len + end;

    /* Clamp to valid range */
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start >= end) return vp_str_create("");

    size_t slice_len = end - start;
    char* result = (char*)vp_arc_alloc(slice_len + 1);

    strncpy(result, str + start, slice_len);
    result[slice_len] = '\0';

    return result;
}

bool vp_str_equals(const char* a, const char* b) {
    if (!a && !b) return true;
    if (!a || !b) return false;
    return strcmp(a, b) == 0;
}

int64_t vp_str_compare(const char* a, const char* b) {
    if (!a && !b) return 0;
    if (!a) return -1;
    if (!b) return 1;
    return (int64_t)strcmp(a, b);
}

/* ============================================ */
/* String Methods                               */
/* ============================================ */

char* vp_str_upper(const char* str) {
    if (!str) return NULL;
    size_t len = strlen(str);
    char* upper = (char*)vp_arc_alloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        upper[i] = (char)toupper((unsigned char)str[i]);
    }
    upper[len] = '\0';
    return upper;
}

char* vp_str_lower(const char* str) {
    if (!str) return NULL;
    size_t len = strlen(str);
    char* lower = (char*)vp_arc_alloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        lower[i] = (char)tolower((unsigned char)str[i]);
    }
    lower[len] = '\0';
    return lower;
}

ViperList* vp_str_split(const char* str, const char* delim) {
    ViperList* list = vp_list_create();
    if (!str || !delim) return list;
    
    char* str_copy = vp_str_create(str); // Mutable copy for strtok
    char* token = strtok(str_copy, delim);
    while (token != NULL) {
        // Here we just append the string directly to the list
        // Note: vp_list_append appends an i64_t, which causes pointer truncation
        // Phase 3 list handles i64 array, so storing a pointer needs casting.
        vp_list_append(list, (int64_t)vp_str_create(token));
        token = strtok(NULL, delim);
    }
    vp_str_free(str_copy);
    return list;
}

char* vp_str_replace(const char* str, const char* old_sub, const char* new_sub) {
    if (!str || !old_sub || !new_sub) return NULL;
    
    size_t old_len = strlen(old_sub);
    size_t new_len = strlen(new_sub);
    
    if (old_len == 0) return vp_str_create(str);
    
    // Count occurrences
    const char* p = str;
    int count = 0;
    while ((p = strstr(p, old_sub)) != NULL) {
        count++;
        p += old_len;
    }
    
    size_t res_len = strlen(str) + count * (new_len - old_len);
    char* result = (char*)vp_arc_alloc(res_len + 1);
    
    char* out = result;
    p = str;
    const char* tmp;
    while ((tmp = strstr(p, old_sub)) != NULL) {
        size_t len = tmp - p;
        strncpy(out, p, len);
        out += len;
        strcpy(out, new_sub);
        out += new_len;
        p = tmp + old_len;
    }
    strcpy(out, p);

    return result;
}

// String format: replaces {} placeholders with arguments
// Args: format_str, args_array (array of char*), arg_count
char* vp_str_format(const char* format_str, const char** args_array, int64_t arg_count) {
    if (!format_str) return NULL;
    
    // Make a copy of the format string to work with
    char* result = vp_str_create(format_str);
    if (!result || arg_count == 0 || !args_array) {
        return result;
    }
    
    // Replace each {} placeholder with corresponding argument
    for (int64_t i = 0; i < arg_count; i++) {
        const char* arg = args_array[i];
        if (!arg) continue;
        
        // Find first {} placeholder
        char* placeholder = strstr(result, "{}");
        if (!placeholder) break;
        
        // Build new string: before_placeholder + arg + after_placeholder
        size_t before_len = placeholder - result;
        size_t arg_len = strlen(arg);
        size_t after_len = strlen(placeholder + 2);
        
        char* new_result = (char*)vp_arc_alloc(before_len + arg_len + after_len + 1);
        
        // Copy before part
        strncpy(new_result, result, before_len);
        new_result[before_len] = '\0';
        
        // Append argument
        strcat(new_result, arg);
        
        // Append after part
        strcat(new_result, placeholder + 2);
        
        // Note: old result will be freed by ARC when refcount reaches 0
        result = new_result;
    }
    
    return result;
}

// Convert bool to string
char* vp_str_from_bool(bool val) {
    return vp_str_create(val ? "True" : "False");
}

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

/* Math functions */
double vp_pow(double base, double exponent) {
    return pow(base, exponent);
}

int64_t vp_pow_i64(int64_t base, int64_t exponent) {
    if (exponent < 0) {
        vp_panic("Negative exponent not supported for integer power");
    }
    if (exponent == 0) {
        return 1;
    }
    
    int64_t result = 1;
    int64_t b = base;
    int64_t e = exponent;
    
    while (e > 0) {
        if (e & 1) {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }
    
    return result;
}

void vp_panic(const char* message) {
    fprintf(stderr, "\nViper Runtime Error: %s\n", message);
    exit(1);
}

void vp_assert(bool condition, const char* message) {
    if (!condition) {
        vp_panic(message);
    }
}

/* ============================================ */
/* Hash Functions                               */
/* ============================================ */

/* FNV-1a hash constants */
#define FNV_OFFSET_BASIS 14695981039346656037ULL
#define FNV_PRIME 1099511628211ULL

/* Hash an i64 value */
int64_t vp_hash_i64(int64_t val) {
    /* Simple hash for integers - use the value itself with some mixing */
    uint64_t hash = (uint64_t)val;
    hash ^= hash >> 33;
    hash *= 0xff51afd7ed558ccdULL;
    hash ^= hash >> 33;
    hash *= 0xc4ceb9fe1a85ec53ULL;
    hash ^= hash >> 33;
    return (int64_t)hash;
}

/* Hash an f64 value */
int64_t vp_hash_f64(double val) {
    /* Hash the bit representation of the float */
    uint64_t bits;
    memcpy(&bits, &val, sizeof(double));
    return vp_hash_i64((int64_t)bits);
}

/* Hash a bool value */
int64_t vp_hash_bool(bool val) {
    return val ? 1 : 0;
}

/* Hash a string using FNV-1a */
int64_t vp_hash_str(const char* str) {
    if (!str) return 0;
    
    uint64_t hash = FNV_OFFSET_BASIS;
    while (*str) {
        hash ^= (uint64_t)(*str);
        hash *= FNV_PRIME;
        str++;
    }
    return (int64_t)hash;
}

/* Hash for None */
int64_t vp_hash_none(void) {
    return 0;
}

/* ============================================ */
/* Range Built-in                               */
/* ============================================ */

ViperList* vp_builtin_range_list(int64_t end) {
    ViperList* list = vp_list_create();

    for (int64_t i = 0; i < end; i++) {
        vp_list_append(list, i);
    }

    return list;
}

/* ============================================ */
/* Type Conversion Functions                    */
/* ============================================ */

/* Convert i64 to string */
char* vp_str_from_i64(int64_t val) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%ld", (long)val);
    return vp_str_create(buffer);
}

/* Convert f64 to string */
char* vp_str_from_f64(double val) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g", val);
    return vp_str_create(buffer);
}

/* Convert string to i64 */
int64_t vp_i64_from_str(const char* str) {
    if (!str) return 0;
    return strtoll(str, NULL, 10);
}

/* Convert string to f64 */
double vp_f64_from_str(const char* str) {
    if (!str) return 0.0;
    return strtod(str, NULL);
}

/* Convert i64 to bool (non-zero = true) */
bool vp_bool_from_i64(int64_t val) {
    return val != 0;
}

/* Convert f64 to bool (non-zero = true) */
bool vp_bool_from_f64(double val) {
    return val != 0.0;
}

/* Convert string to bool */
bool vp_bool_from_str(const char* str) {
    if (!str) return false;
    return (strcmp(str, "True") == 0 || strcmp(str, "true") == 0 ||
            strcmp(str, "1") == 0);
}

/* ============================================ */
/* Math Functions                               */
/* ============================================ */

double vp_math_sqrt(double x) {
    return sqrt(x);
}

double vp_math_abs(double x) {
    return fabs(x);
}

double vp_math_ln(double x) {
    return log(x);
}

double vp_math_floor(double x) {
    return floor(x);
}

/* ============================================ */
/* Dict Wrapper Functions (for LLVM JIT)         */
/* ============================================ */

/* Wrapper for dict set with i64 value */
void vp_dict_set_i64(ViperDict* dict, const char* key, int64_t value) {
    if (!dict || !key) return;

    ViperValue val;
    val.type = VIPER_TYPE_I64;
    val.data.as_i64 = value;

    vp_dict_set(dict, key, val);
}

/* Wrapper for dict set with string key (from vp_str_create) and i64 value */
void vp_dict_set_str_i64(ViperDict* dict, void* str_ptr, int64_t value) {
    if (!dict || !str_ptr) {
        return;
    }

    /* vp_str_create returns a plain char*, not ViperString struct */
    const char* key = (const char*)str_ptr;

    ViperValue val;
    val.type = VIPER_TYPE_I64;
    val.data.as_i64 = value;

    vp_dict_set(dict, key, val);
}

/* Wrapper for dict set with string key and string value (both from vp_str_create) */
void vp_dict_set_str_str(ViperDict* dict, void* key_str, void* value_str) {
    if (!dict || !key_str || !value_str) {
        return;
    }

    /* vp_str_create returns plain char* */
    const char* key = (const char*)key_str;
    char* val_str = (char*)value_str;

    /* Create ViperValue with string type */
    ViperValue val;
    val.type = VIPER_TYPE_STR;
    val.data.as_str = val_str;

    vp_dict_set(dict, key, val);
}

/* Wrapper for dict get returning i64 value */
int64_t vp_dict_get_i64(ViperDict* dict, const char* key) {
    if (!dict || !key) return 0;
    
    ViperValue val = vp_dict_get(dict, key);
    
    if (val.type == VIPER_TYPE_I64) {
        return val.data.as_i64;
    } else if (val.type == VIPER_TYPE_NONE) {
        return 0;
    }
    
    /* Type mismatch - return 0 for now */
    return 0;
}

/* ============================================ */
/* Struct Module (Python-compatible pack/unpack)  */
/* ============================================ */

#include <string.h>

typedef enum {
    FMT_INT8 = 0,
    FMT_UINT8,
    FMT_INT16,
    FMT_UINT16,
    FMT_INT32,
    FMT_UINT32,
    FMT_INT64,
    FMT_UINT64,
    FMT_FLOAT32,
    FMT_FLOAT64,
    FMT_STRING,
} FormatType;

static int get_format_size(char c) {
    switch (c) {
        case 'b': case 'B': return 1;
        case 'h': case 'H': return 2;
        case 'i': case 'I': case 'l': case 'L': case 'f': return 4;
        case 'q': case 'Q': case 'd': return 8;
        default: return 4;
    }
}

/* Calculate total size needed for pack */
char* vp_struct_pack(const char* format, ...) {
    va_list args;
    va_start(args, format);
    
    /* First pass: calculate size */
    int size = 0;
    int count = 1;
    
    for (int i = 0; format[i]; i++) {
        char c = format[i];
        
        if (c >= '0' && c <= '9') {
            count = count * 10 + (c - '0');
            continue;
        }
        
        if (c == 's') {
            /* String: count bytes */
            const char* str = va_arg(args, char*);
            if (str) {
                int len = count < (int)strlen(str) ? count : (int)strlen(str);
                size += len;
            }
            /* Also account for length prefix */
            size += 4;
        } else {
            size += get_format_size(c) * count;
        }
        count = 1;
    }
    
    /* Allocate buffer */
    char* buffer = (char*)vp_arc_alloc(size + 1);
    int offset = 0;
    
    /* Second pass: pack values */
    va_start(args, format);
    count = 1;
    
    for (int i = 0; format[i]; i++) {
        char c = format[i];
        
        if (c >= '0' && c <= '9') {
            count = count * 10 + (c - '0');
            continue;
        }
        
        for (int j = 0; j < count; j++) {
            switch (c) {
                case 'b': {
                    int8_t val = (int8_t)va_arg(args, int);
                    buffer[offset++] = (char)val;
                    break;
                }
                case 'B': {
                    uint8_t val = (uint8_t)va_arg(args, unsigned int);
                    buffer[offset++] = (char)val;
                    break;
                }
                case 'h': {
                    int16_t val = (int16_t)va_arg(args, int);
                    memcpy(&buffer[offset], &val, 2);
                    offset += 2;
                    break;
                }
                case 'H': {
                    uint16_t val = (uint16_t)va_arg(args, unsigned int);
                    memcpy(&buffer[offset], &val, 2);
                    offset += 2;
                    break;
                }
                case 'i': case 'l': {
                    int32_t val = va_arg(args, int32_t);
                    memcpy(&buffer[offset], &val, 4);
                    offset += 4;
                    break;
                }
                case 'I': case 'L': {
                    uint32_t val = va_arg(args, uint32_t);
                    memcpy(&buffer[offset], &val, 4);
                    offset += 4;
                    break;
                }
                case 'q': {
                    int64_t val = va_arg(args, int64_t);
                    memcpy(&buffer[offset], &val, 8);
                    offset += 8;
                    break;
                }
                case 'Q': {
                    uint64_t val = va_arg(args, uint64_t);
                    memcpy(&buffer[offset], &val, 8);
                    offset += 8;
                    break;
                }
                case 'f': {
                    float val = (float)va_arg(args, double);
                    memcpy(&buffer[offset], &val, 4);
                    offset += 4;
                    break;
                }
                case 'd': {
                    double val = va_arg(args, double);
                    memcpy(&buffer[offset], &val, 8);
                    offset += 8;
                    break;
                }
                case 's': {
                    const char* str = va_arg(args, char*);
                    if (str) {
                        int len = count < (int)strlen(str) ? count : (int)strlen(str);
                        /* Write length prefix */
                        memcpy(&buffer[offset], &len, 4);
                        offset += 4;
                        /* Write string */
                        memcpy(&buffer[offset], str, len);
                        offset += len;
                    }
                    break;
                }
            }
        }
        count = 1;
    }
    
    buffer[offset] = '\0';
    va_end(args);
    
    return buffer;
}

/* Unpack values from binary buffer - returns allocated buffer with unpacked data */
/* Note: format parameter reserved for future use */
char* vp_struct_unpack(const char* format, const char* data, int data_len) {
    (void)format; /* Reserved for future use */
    /* Simplified implementation: return a copy of the data */
    /* A full implementation would parse the format and unpack values */
    if (!data || data_len <= 0) return NULL;

    char* buffer = (char*)malloc((size_t)data_len + 1);
    if (!buffer) return NULL;
    
    memcpy(buffer, data, (size_t)data_len);
    buffer[data_len] = '\0';
    return buffer;
}
