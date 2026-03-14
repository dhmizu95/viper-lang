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

    printf("{");
    bool first = true;
    for (int64_t i = 0; i < dict->size; i++) {
        DictEntry* entry = dict->buckets[i];
        while (entry) {
            if (!first) printf(", ");
            first = false;

            if (entry->key) {
                printf("'%.*s': ", (int)vp_str_len_inline(entry->key), vp_str_data_inline(entry->key));
            } else {
                printf("None: ");
            }

            switch (entry->value.type) {
                case VIPER_TYPE_I64: printf("%ld", (long)entry->value.data.as_i64); break;
                case VIPER_TYPE_F64: printf("%f", entry->value.data.as_f64); break;
                case VIPER_TYPE_BOOL: printf("%s", entry->value.data.as_bool ? "True" : "False"); break;
                case VIPER_TYPE_STR: 
                    if (entry->value.data.as_str)
                        printf("'%.*s'", (int)vp_str_len_inline(entry->value.data.as_str), vp_str_data_inline(entry->value.data.as_str));
                    else
                        printf("None");
                    break;
                case VIPER_TYPE_LIST: printf("[...]"); break;
                case VIPER_TYPE_DICT: printf("{...}"); break;
                case VIPER_TYPE_NONE: printf("None"); break;
                default: printf("<?>"); break;
            }
            entry = entry->next;
        }
    }
    printf("}");
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
/* String functions (implemented in runtime.c) */
/* ============================================ */

/* Exported versions of inline string functions for AOT code generation */
int64_t vp_str_len(ViperString* s) {
    return vp_str_len_inline(s);
}

int64_t vp_str_get_first(ViperString* s) {
    return vp_str_get_first_inline(s);
}

const char* vp_str_data(ViperString* s) {
    return vp_str_data_inline(s);
}

ViperString* vp_str_create(const char* s) {
    return vp_str_create_inline(s);
}

ViperString* vp_str_create_with_len(const char* s, int64_t len) {
    if (len <= VIPER_SSO_CAPACITY) {
        return vp_str_create_sso_small(s, len);
    }
    return vp_str_create_heap_large(s, len);
}

void vp_str_free(ViperString* s) {
    vp_str_free_inline(s);
}

ViperString* vp_str_concat(ViperString* a, ViperString* b) {
    return vp_str_concat_inline(a, b);
}

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
ViperString* vp_str_format(ViperString* format_str, ViperString** args_array, int64_t arg_count) {
    if (!format_str) return NULL;

    const char* format_data = vp_str_data_inline(format_str);
    if (!format_data) return NULL;

    // Make a copy of the format string to work with
    char* result_data = strdup(format_data);
    if (!result_data || !args_array || arg_count == 0) {
        ViperString* result = vp_str_create(result_data);
        free(result_data);
        return result;
    }

    // Replace each {...} placeholder with corresponding argument
    for (int64_t i = 0; i < arg_count; i++) {
        if (!args_array[i]) continue;
        const char* arg_data = vp_str_data_inline(args_array[i]);
        if (!arg_data) continue;

        // Find first {
        char* start_placeholder = strchr(result_data, '{');
        if (!start_placeholder) break;
        
        // Find matching }
        char* end_placeholder = strchr(start_placeholder, '}');
        if (!end_placeholder) break;

        // Extract specifier if present (e.g., {:.9f} -> .9f)
        char spec[32] = {0};
        bool has_spec = false;
        if (end_placeholder - start_placeholder > 2 && start_placeholder[1] == ':') {
            has_spec = true;
            size_t spec_len = end_placeholder - start_placeholder - 2;
            if (spec_len > 31) spec_len = 31;
            strncpy(spec, start_placeholder + 2, spec_len);
            spec[spec_len] = '\0';
        }

        // Apply specifier if it's a known one (e.g., .9f)
        char formatted_arg[128];
        const char* final_arg_data = arg_data;
        if (has_spec && strcmp(spec, ".9f") == 0) {
            double val = atof(arg_data);
            snprintf(formatted_arg, sizeof(formatted_arg), "%.9f", val);
            final_arg_data = formatted_arg;
        }

        // Build new string: before_placeholder + arg + after_placeholder
        size_t before_len = start_placeholder - result_data;
        size_t arg_len = strlen(final_arg_data);
        size_t after_len = strlen(end_placeholder + 1);

        char* new_result_data = (char*)malloc(before_len + arg_len + after_len + 1);
        if (!new_result_data) break;

        // Copy before part
        strncpy(new_result_data, result_data, before_len);
        new_result_data[before_len] = '\0';

        // Append argument
        strcat(new_result_data, final_arg_data);

        // Append after part
        strcat(new_result_data, end_placeholder + 1);

        free(result_data);
        result_data = new_result_data;
    }

    ViperString* final_result = vp_str_create(result_data);
    free(result_data);
    return final_result;
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
    // Use higher precision to match Rust/Python behavior
    snprintf(buffer, sizeof(buffer), "%.16g", val);
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

    ViperString* key_str = vp_str_create(key);
    if (!key_str) return;

    ViperValue val;
    val.type = VIPER_TYPE_I64;
    val.data.as_i64 = value;

    bool existed = vp_dict_contains(dict, key_str);
    vp_dict_set(dict, key_str, val);
    if (existed) {
        vp_str_free(key_str);
    }
}

/* Wrapper for dict set with string key (from vp_str_create) and i64 value */
void vp_dict_set_str_i64(ViperDict* dict, void* str_ptr, int64_t value) {
    if (!dict || !str_ptr) {
        return;
    }

    ViperString* key = (ViperString*)str_ptr;

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

    ViperString* key = (ViperString*)key_str;
    ViperString* val_str = (ViperString*)value_str;

    /* Create ViperValue with string type */
    ViperValue val;
    val.type = VIPER_TYPE_STR;
    val.data.as_str = val_str;

    vp_dict_set(dict, key, val);
}

/* Helper for optimized dict lookup */
static uint64_t vp_runtime_hash_cstr(const char* data) {
    if (!data) return 0;
    uint64_t hash = 14695981039346656037ULL;
    for (int i = 0; data[i]; i++) {
        hash ^= (uint64_t)data[i];
        hash *= 1099511628211ULL;
    }
    return hash;
}

/* Optimized wrapper for dict get returning i64 value - avoids ViperString allocation */
int64_t vp_dict_get_i64(ViperDict* dict, const char* key) {
    if (!dict || !key) return 0;

    uint64_t hash = vp_runtime_hash_cstr(key);
    int64_t index = hash % dict->size;

    DictEntry* entry = dict->buckets[index];
    while (entry) {
        if (entry->key) {
            const char* entry_data = vp_str_data_inline(entry->key);
            if (strcmp(entry_data, key) == 0) {
                if (entry->value.type == VIPER_TYPE_I64) {
                    return entry->value.data.as_i64;
                }
                return 0;
            }
        }
        entry = entry->next;
    }
    
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
bool vp_str_equals(ViperString* a, ViperString* b) {
    if (!a && !b) return true;
    if (!a || !b) return false;

    int64_t len_a = vp_str_len_inline(a);
    int64_t len_b = vp_str_len_inline(b);

    if (len_a != len_b) return false;

    const char* data_a = vp_str_data_inline(a);
    const char* data_b = vp_str_data_inline(b);

    return memcmp(data_a, data_b, (size_t)len_a) == 0;
}

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

ViperList* vp_struct_unpack(const char* format, const char* buffer) {
    if (!format || !buffer) return NULL;
    
    ViperList* result = vp_list_create();
    int offset = 0;
    int count = 1;
    bool has_count = false;
    
    for (int i = 0; format[i]; i++) {
        char c = format[i];
        
        if (c >= '0' && c <= '9') {
            if (!has_count) {
                count = 0;
                has_count = true;
            }
            count = count * 10 + (c - '0');
            continue;
        }
        
        has_count = false;
        
        for (int j = 0; j < count; j++) {
            switch (c) {
                case 'b': {
                    int8_t val = (int8_t)buffer[offset++];
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'B': {
                    uint8_t val = (uint8_t)buffer[offset++];
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'h': {
                    int16_t val;
                    memcpy(&val, &buffer[offset], 2);
                    offset += 2;
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'H': {
                    uint16_t val;
                    memcpy(&val, &buffer[offset], 2);
                    offset += 2;
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'i': case 'l': {
                    int32_t val;
                    memcpy(&val, &buffer[offset], 4);
                    offset += 4;
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'I': case 'L': {
                    uint32_t val;
                    memcpy(&val, &buffer[offset], 4);
                    offset += 4;
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'q': {
                    int64_t val;
                    memcpy(&val, &buffer[offset], 8);
                    offset += 8;
                    vp_list_append(result, val);
                    break;
                }
                case 'Q': {
                    uint64_t val;
                    memcpy(&val, &buffer[offset], 8);
                    offset += 8;
                    vp_list_append(result, (int64_t)val);
                    break;
                }
                case 'f': {
                    float val;
                    memcpy(&val, &buffer[offset], 4);
                    offset += 4;
                    ViperValue v;
                    v.type = VIPER_TYPE_F64;
                    v.data.as_f64 = (double)val;
                    vp_list_append(result, v.data.as_i64);
                    break;
                }
                case 'd': {
                    double val;
                    memcpy(&val, &buffer[offset], 8);
                    offset += 8;
                    ViperValue v;
                    v.type = VIPER_TYPE_F64;
                    v.data.as_f64 = val;
                    vp_list_append(result, v.data.as_i64);
                    break;
                }
                case 's': {
                    int len;
                    memcpy(&len, &buffer[offset], 4);
                    offset += 4;
                    char* str_data = (char*)malloc(len + 1);
                    memcpy(str_data, &buffer[offset], len);
                    str_data[len] = '\0';
                    offset += len;
                    ViperString* vs = vp_str_create(str_data);
                    free(str_data);
                    ViperValue v;
                    v.type = VIPER_TYPE_STR;
                    v.data.as_str = vs;
                    vp_list_append(result, v.data.as_i64);
                    j = count; 
                    break;
                }
            }
        }
        count = 1;
    }
    return result;
}

// Float list specific functions
ViperList* vp_list_create_f64(void) {
    return vp_list_create();
}

void vp_list_append_f64(ViperList* list, double value) {
    if (!list) return;
    if (list->length >= list->capacity) vp_list_grow(list);
    list->data.data_f64[list->length++] = value;
}

void vp_list_set_f64(ViperList* list, int64_t index, double value) {
    if (!list || index < 0 || index >= list->length) {
        vp_panic("List index out of bounds (f64)");
    }
    list->data.data_f64[index] = value;
}
