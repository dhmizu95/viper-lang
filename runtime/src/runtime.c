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
    fflush(stdout);
}

void vp_print_f64(double val) {
    printf("%g", val);
    fflush(stdout);
}

void vp_print_str(ViperString* val) {
    if (!val) {
        printf("(null)");
        fflush(stdout);
        return;
    }

    const char* data = vp_str_data_inline(val);
    printf("%s", data);
    fflush(stdout);
}

void vp_print_bool(bool val) {
    printf("%s", val ? "True" : "False");
    fflush(stdout);
}

void vp_print_newline(void) {
    printf("\n");
    fflush(stdout);
}

void vp_print_list(ViperList* list) {
    if (!list) {
        printf("[]");
        return;
    }
    
    printf("[");
    int64_t len = vp_list_len(list);
    for (int64_t i = 0; i < len; i++) {
        if (i > 0) printf(", ");
        printf("%ld", (long)vp_list_get(list, i));
    }
    printf("]");
}

void vp_print_dict(ViperDict* dict) {
    if (!dict) {
        printf("{}");
        return;
    }
    printf("{...}");  // Simplified dict print
}

void vp_print_bytes(ViperBytes* bytes) {
    if (!bytes) {
        printf("b''");
        return;
    }
    
    printf("b'");
    for (int64_t i = 0; i < bytes->len; i++) {
        printf("\\x%02x", bytes->data[i]);
    }
    printf("'");
}

/* ============================================ */
/* String Functions - Now defined as static inline in viper_types.h */
/* ============================================ */
/* The following functions are now in viper_types.h as static inline:
 * - vp_str_create()
 * - vp_str_free()
 * - vp_str_concat()
 * - vp_str_len()
 * - vp_str_slice()
 * - vp_str_equals()
 * - vp_str_compare()
 */

/* ============================================ */
/* String Methods                               */
/* ============================================ */

ViperString* vp_str_upper(ViperString* str) {
    if (!str) return NULL;
    int64_t len = vp_str_len_inline(str);
    const char* data = vp_str_data_inline(str);
    
    ViperString* result = vp_str_create(data);
    char* result_data = (char*)vp_str_data_inline(result);
    for (int64_t i = 0; i < len; i++) {
        result_data[i] = (char)toupper((unsigned char)data[i]);
    }
    return result;
}

ViperString* vp_str_lower(ViperString* str) {
    if (!str) return NULL;
    int64_t len = vp_str_len_inline(str);
    const char* data = vp_str_data_inline(str);
    
    ViperString* result = vp_str_create(data);
    char* result_data = (char*)vp_str_data_inline(result);
    for (int64_t i = 0; i < len; i++) {
        result_data[i] = (char)tolower((unsigned char)data[i]);
    }
    return result;
}

ViperList* vp_str_split(ViperString* str, ViperString* delim) {
    ViperList* list = vp_list_create();
    if (!str || !delim) return list;

    const char* str_data = vp_str_data_inline(str);
    const char* delim_data = vp_str_data_inline(delim);
    
    // Simple split implementation
    const char* p = str_data;
    const char* tmp;
    while ((tmp = strstr(p, delim_data)) != NULL) {
        size_t len = tmp - p;
        char* token = (char*)malloc(len + 1);
        strncpy(token, p, len);
        token[len] = '\0';
        vp_list_append(list, (int64_t)vp_str_create(token));
        free(token);
        p = tmp + strlen(delim_data);
    }
    // Add last token
    vp_list_append(list, (int64_t)vp_str_create(p));
    
    return list;
}

ViperString* vp_str_replace(ViperString* str, ViperString* old_sub, ViperString* new_sub) {
    if (!str || !old_sub || !new_sub) return NULL;

    const char* str_data = vp_str_data_inline(str);
    const char* old_data = vp_str_data_inline(old_sub);
    const char* new_data = vp_str_data_inline(new_sub);
    
    size_t old_len = strlen(old_data);
    size_t new_len = strlen(new_data);

    if (old_len == 0) return vp_str_create(str_data);

    // Count occurrences
    const char* p = str_data;
    int count = 0;
    while ((p = strstr(p, old_data)) != NULL) {
        count++;
        p += old_len;
    }

    size_t res_len = strlen(str_data) + count * (new_len - old_len);
    char* result_data = (char*)malloc(res_len + 1);

    char* out = result_data;
    p = str_data;
    const char* tmp;
    while ((tmp = strstr(p, old_data)) != NULL) {
        size_t len = tmp - p;
        strncpy(out, p, len);
        out += len;
        strcpy(out, new_data);
        out += new_len;
        p = tmp + old_len;
    }
    strcpy(out, p);
    result_data[res_len] = '\0';
    
    ViperString* result = vp_str_create(result_data);
    free(result_data);
    return result;
}

// String format: replaces {} placeholders with arguments
// Args: format_str, args (ViperList of ViperString*)
ViperString* vp_str_format(ViperString* format_str, ViperList* args) {
    if (!format_str) return NULL;

    const char* format_data = vp_str_data_inline(format_str);
    
    // Make a copy of the format string to work with
    char* result_data = strdup(format_data);
    if (!result_data || !args || vp_list_len(args) == 0) {
        ViperString* result = vp_str_create(result_data);
        free(result_data);
        return result;
    }

    // Replace each {} placeholder with corresponding argument
    int64_t arg_count = vp_list_len(args);
    for (int64_t i = 0; i < arg_count; i++) {
        const char* arg_data = vp_str_data_inline((ViperString*)(intptr_t)vp_list_get(args, i));
        if (!arg_data) continue;

        // Find first {} placeholder
        char* placeholder = strstr(result_data, "{}");
        if (!placeholder) break;
        
        // Build new string: before_placeholder + arg + after_placeholder
        size_t before_len = placeholder - result_data;
        size_t arg_len = strlen(arg_data);
        size_t after_len = strlen(placeholder + 2);

        char* new_result_data = (char*)malloc(before_len + arg_len + after_len + 1);

        // Copy before part
        strncpy(new_result_data, result_data, before_len);
        new_result_data[before_len] = '\0';

        // Append argument
        strcat(new_result_data, arg_data);

        // Append after part
        strcat(new_result_data, placeholder + 2);

        free(result_data);
        result_data = new_result_data;
    }

    ViperString* result = vp_str_create(result_data);
    free(result_data);
    return result;
}

// Convert bool to string
ViperString* vp_str_from_bool(bool val) {
    return vp_str_create(val ? "True" : "False");
}

// Convert i64 to string
ViperString* vp_str_from_i64(int64_t val) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%ld", (long)val);
    return vp_str_create(buffer);
}

// Convert f64 to string
ViperString* vp_str_from_f64(double val) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g", val);
    return vp_str_create(buffer);
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
int64_t vp_hash_str(ViperString* str) {
    if (!str) return 0;

    const char* data = vp_str_data_inline(str);
    int64_t len = vp_str_len_inline(str);
    
    uint64_t hash = FNV_OFFSET_BASIS;
    for (int64_t i = 0; i < len; i++) {
        hash ^= (uint64_t)data[i];
        hash *= FNV_PRIME;
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

/* Convert i64 to bool (non-zero = true) */
bool vp_bool_from_i64(int64_t val) {
    return val != 0;
}

/* Convert f64 to bool (non-zero = true) */
bool vp_bool_from_f64(double val) {
    return val != 0.0;
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
