/**
 * Viper Standard Library Header
 * Phase 2: Memory Management and Data Structures
 */

#ifndef VIPER_STDLIB_H
#define VIPER_STDLIB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "viper_types.h"
#include "viper_arc.h"

/* ============================================ */
/* Basic I/O Functions - Defined later with ViperString* */
/* ============================================ */

/* ============================================ */
/* Memory Management (ARC)                      */
/* ============================================ */

void* vp_alloc(size_t size);
void vp_free(void* ptr);
void vp_retain(void* ptr);
void vp_release(void* ptr);
int64_t vp_ref_count(void* ptr);

static inline char* vp_strdup_slice(const char* s, size_t len) {
    char* result = (char*)vp_arc_alloc(len + 1);
    if (!result) return NULL;
    memcpy(result, s, len);
    result[len] = '\0';
    return result;
}

/* ============================================ */
/* List Functions                               */
/* ============================================ */

/* Generic list functions (i64) - for backward compatibility */
ViperList* vp_list_create(void);
ViperList* vp_list_create_with_capacity(int64_t cap);
void vp_list_free(ViperList* list);
void vp_list_grow(ViperList* list);  /* Exposed for inline codegen */
void vp_list_reserve(ViperList* list, int64_t capacity);  /* Pre-allocate capacity */
void vp_list_append(ViperList* list, int64_t value);
void vp_list_insert(ViperList* list, int64_t index, int64_t value);
int64_t vp_list_remove(ViperList* list, int64_t index);
int64_t vp_list_pop(ViperList* list);
void vp_list_clear(ViperList* list);
int64_t vp_list_get(ViperList* list, int64_t index);
void vp_list_set(ViperList* list, int64_t index, int64_t value);
int64_t vp_list_len(ViperList* list);
bool vp_list_contains(ViperList* list, int64_t value);
ViperList* vp_list_copy(ViperList* list);
ViperList* vp_list_repeat(int64_t elem, int64_t count);
ViperList* vp_list_zeros(int64_t count);       /* Optimized [0] * n */
ViperList* vp_list_ones(int64_t count);        /* Optimized [1] * n */
ViperList* vp_list_slice(ViperList* list, int64_t start, int64_t end, int64_t step);
void vp_list_print(ViperList* list);

/* Tuple Functions */
ViperTuple* vp_tuple_create(int64_t size);
void vp_tuple_free(ViperTuple* tuple);
int64_t vp_tuple_get(ViperTuple* tuple, int64_t index);
void vp_tuple_print(ViperTuple* tuple);
ViperTuple* vp_tuple_from_list(ViperList* list);
ViperTuple* vp_tuple_from_iterable(void* iterable);
ViperTuple* vp_tuple_from_str(ViperString* str);
ViperString* vp_tuple_to_str(ViperTuple* tuple);

/* Float list functions */
ViperList* vp_list_create_f64(void);
void vp_list_append_f64(ViperList* list, double value);
void vp_list_set_f64(ViperList* list, int64_t index, double value);

/* Extended list operations (i64) */
void vp_list_extend(ViperList* list, ViperList* other);
int64_t vp_list_index(ViperList* list, int64_t value);
int64_t vp_list_count(ViperList* list, int64_t value);
void vp_list_sort(ViperList* list);
void vp_list_reverse(ViperList* list);
ViperList* vp_list_reversed(ViperList* list);
ViperList* vp_list_sorted(ViperList* list);
ViperList* vp_list_concat(ViperList* list1, ViperList* list2);

/* Built-in iteration/functional functions */
ViperList* vp_enumerate(ViperList* iterable, int64_t start);
ViperList* vp_zip(ViperList* iter1, ViperList* iter2);
int64_t vp_list_sum(ViperList* list);
int64_t vp_list_min(ViperList* list);
int64_t vp_list_max(ViperList* list);
int vp_list_any(ViperList* list);
int vp_list_all(ViperList* list);
ViperString* vp_type_of(void* obj);

/* Bool list functions (type-specific, memory efficient) */
ViperList* vp_list_bool_create(void);
ViperList* vp_list_bool_create_with_capacity(int64_t cap);
void vp_list_bool_free(ViperList* list);
void vp_list_bool_append(ViperList* list, bool value);
void vp_list_bool_insert(ViperList* list, int64_t index, bool value);
bool vp_list_bool_remove(ViperList* list, int64_t index);
bool vp_list_bool_pop(ViperList* list);
void vp_list_bool_clear(ViperList* list);
bool vp_list_bool_get(ViperList* list, int64_t index);
void vp_list_bool_set(ViperList* list, int64_t index, bool value);
bool vp_list_bool_contains(ViperList* list, bool value);
ViperList* vp_list_bool_copy(ViperList* list);
ViperList* vp_list_bool_repeat(bool elem, int64_t count);
ViperList* vp_list_bool_slice(ViperList* list, int64_t start, int64_t end, int64_t step);
void vp_list_bool_print(ViperList* list);

/* Extended bool list operations */
void vp_list_bool_extend(ViperList* list, ViperList* other);
int64_t vp_list_bool_index(ViperList* list, bool value);
int64_t vp_list_bool_count(ViperList* list, bool value);
void vp_list_bool_reverse(ViperList* list);
ViperList* vp_list_bool_reversed(ViperList* list);
ViperList* vp_list_bool_concat(ViperList* list1, ViperList* list2);

/* Bit vector functions (1 bit per boolean - 8x memory savings) */
ViperList* vp_bitvec_create(void);
ViperList* vp_bitvec_create_with_capacity(int64_t cap);
ViperList* vp_bitvec_repeat(bool elem, int64_t count);
void vp_bitvec_free(ViperList* vec);
void vp_bitvec_append(ViperList* vec, bool value);  /* always_inline in .c */
void vp_bitvec_insert(ViperList* vec, int64_t index, bool value);
bool vp_bitvec_remove(ViperList* vec, int64_t index);
bool vp_bitvec_pop(ViperList* vec);
void vp_bitvec_clear(ViperList* vec);

/* Bounds-checked versions (default) */
bool vp_bitvec_get(ViperList* vec, int64_t index);  /* always_inline in .c */
void vp_bitvec_set(ViperList* vec, int64_t index, bool value);  /* always_inline in .c */

/* Unchecked versions for hot loops - no bounds checking */
bool vp_bitvec_get_unchecked(ViperList* vec, int64_t index);
void vp_bitvec_set_unchecked(ViperList* vec, int64_t index, bool value);

bool vp_bitvec_contains(ViperList* vec, bool value);
ViperList* vp_bitvec_copy(ViperList* vec);
ViperList* vp_bitvec_slice(ViperList* vec, int64_t start, int64_t end, int64_t step);
void vp_bitvec_print(ViperList* vec);
int64_t vp_bitvec_len(ViperList* vec);

/* Extended bit vector operations */
void vp_bitvec_extend(ViperList* vec, ViperList* other);
int64_t vp_bitvec_index(ViperList* vec, bool value);
int64_t vp_bitvec_count(ViperList* vec, bool value);
void vp_bitvec_reverse(ViperList* vec);
ViperList* vp_bitvec_reversed(ViperList* vec);
ViperList* vp_bitvec_concat(ViperList* vec1, ViperList* vec2);

/* ============================================ */
/* Dictionary Functions                         */
/* ============================================ */

ViperDict* vp_dict_create(void);
ViperDict* vp_dict_create_with_capacity(int64_t initial_cap);
void vp_dict_free(ViperDict* dict);
void vp_dict_set(ViperDict* dict, ViperString* key, ViperValue value);
ViperValue vp_dict_get(ViperDict* dict, ViperString* key);
bool vp_dict_contains(ViperDict* dict, ViperString* key);
bool vp_dict_remove(ViperDict* dict, ViperString* key);
void vp_dict_clear(ViperDict* dict);
int64_t vp_dict_len(ViperDict* dict);
ViperDict* vp_dict_copy(ViperDict* dict);
void vp_dict_print(ViperDict* dict);
ViperDict* vp_json_loads(const char* json_str);

/* Dict set with ViperString key (for compiler codegen) */
void vp_dict_set_i64(ViperDict* dict, const char* key, int64_t value);
void vp_dict_set_str_i64(ViperDict* dict, void* viper_str, int64_t value);
void vp_dict_set_str_str(ViperDict* dict, void* viper_str, void* value_str);

/* Dictionary Iterator */
typedef struct ViperDictIter ViperDictIter;
struct ViperDictIter {
    ViperDict* dict;
    int64_t bucket_index;
    DictEntry* current;
};

ViperDictIter* vp_dict_iter_create(ViperDict* dict);
void vp_dict_iter_free(ViperDictIter* iter);
bool vp_dict_iter_next(ViperDictIter* iter, ViperString** key, ViperValue* value);

/* ============================================ */
/* String functions (implemented in runtime.c) */
ViperString* vp_str_create(const char* str);
ViperString* vp_str_create_with_len(const char* str, int64_t len);
void vp_str_free(ViperString* s);
ViperString* vp_str_concat(ViperString* a, ViperString* b);
int64_t vp_str_len(ViperString* s);
bool vp_str_equals(ViperString* a, ViperString* b);
const char* vp_str_data(ViperString* s);

/* Additional string functions (implemented in runtime.c) */
ViperString* vp_str_upper(ViperString* str);
ViperString* vp_str_lower(ViperString* str);
ViperList* vp_str_split(ViperString* str, ViperString* delim);
ViperString* vp_str_replace(ViperString* str, ViperString* old_sub, ViperString* new_sub);
ViperString* vp_str_format(ViperString* format_str, ViperString** args_array, int64_t arg_count);
ViperString* vp_str_from_bool(bool val);
ViperString* vp_str_from_i64(int64_t val);
ViperString* vp_str_from_f64(double val);

/* ============================================ */
/* Bytes Functions                              */
/* ============================================ */

/* ViperBytes is now defined in viper_types.h */

ViperBytes* vp_bytes_create(const uint8_t* data, int64_t len);
void vp_bytes_free(ViperBytes* bytes);
ViperBytes* vp_bytes_concat(ViperBytes* a, ViperBytes* b);
int64_t vp_bytes_len(ViperBytes* bytes);
uint8_t vp_bytes_get(ViperBytes* bytes, int64_t index);
void vp_bytes_set(ViperBytes* bytes, int64_t index, uint8_t value);
ViperBytes* vp_bytes_slice(ViperBytes* bytes, int64_t start, int64_t end);
bool vp_bytes_contains(ViperBytes* bytes, uint8_t value);
ViperBytes* vp_bytes_copy(ViperBytes* bytes);
void vp_bytes_print(ViperBytes* bytes);
int64_t vp_bytes_hash(ViperBytes* bytes);
bool vp_bytes_equals(ViperBytes* a, ViperBytes* b);

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

void vp_panic(const char* message);
void vp_assert(bool condition, const char* message);

/* Hash functions */
int64_t vp_hash_i64(int64_t val);
int64_t vp_hash_f64(double val);
int64_t vp_hash_bool(bool val);
int64_t vp_hash_str(ViperString* str);
int64_t vp_hash_none(void);

/* Print functions */
void vp_print_i64(int64_t val);
void vp_print_f64(double val);
void vp_print_str(ViperString* str);
void vp_print_viper_str(ViperString* str);
void vp_print_bool(bool val);
void vp_print_newline(void);
void vp_print_list(ViperList* list);
void vp_print_dict(ViperDict* dict);
void vp_print_bytes(ViperBytes* bytes);

/* Math functions */
double vp_pow(double base, double exponent);
int64_t vp_pow_i64(int64_t base, int64_t exponent);

/* ============================================ */
/* Built-in Functions (callable from Viper)     */
/* ============================================ */

int64_t vp_builtin_range_start(int64_t start, int64_t end);
int64_t vp_builtin_range_next(int64_t* state);
ViperList* vp_builtin_range_list(int64_t end);

/* Async iteration types */
typedef struct ViperAsyncRange ViperAsyncRange;

/* Async iteration functions */
ViperAsyncRange* vp_async_range_create(int64_t start, int64_t end, int64_t step);
int64_t vp_async_range_next(ViperAsyncRange* range);
void vp_async_range_free(ViperAsyncRange* range);
void* vp_async_iter(void* obj);
int64_t vp_async_next(void* iterator);

/* Async future functions */
struct ViperFuture;
struct ViperFuture* vp_future_create(void);
int64_t vp_future_await(struct ViperFuture* future);
int64_t vp_future_await_and_release(struct ViperFuture* future);
void vp_future_set_result(struct ViperFuture* future, int64_t result);
bool vp_future_is_ready(struct ViperFuture* future);
void vp_future_retain(struct ViperFuture* future);
void vp_future_release(struct ViperFuture* future);
int64_t vp_future_gather(int64_t* futures_ptr, int64_t count);
void vp_future_gather_free(int64_t results_ptr, int64_t count);

/* Async context manager functions */
int64_t vp_async_context_enter(void* context);
int64_t vp_async_context_exit(void* context, int64_t exc_type, int64_t exc_val, int64_t exc_tb);

/* ============================================ */
/* Type Conversion Functions                    */
/* ============================================ */

ViperString* vp_str_from_i64(int64_t val);
ViperString* vp_str_from_f64(double val);
ViperString* vp_str_from_bool(bool val);
int64_t vp_i64_from_str(ViperString* str);
double vp_f64_from_str(ViperString* str);
bool vp_bool_from_i64(int64_t val);
bool vp_bool_from_f64(double val);
bool vp_bool_from_str(ViperString* str);

#endif /* VIPER_STDLIB_H */

/* ============================================ */
/* bytearray functions                          */
/* ============================================ */

typedef struct ViperByteArray ViperByteArray;

ViperByteArray* vp_bytearray_create(void);
ViperByteArray* vp_bytearray_create_with_capacity(int64_t cap);
ViperByteArray* vp_bytearray_from_bytes(const uint8_t* bytes, int64_t len);
void vp_bytearray_free(ViperByteArray* ba);
int64_t vp_bytearray_len(ViperByteArray* ba);
void vp_bytearray_append(ViperByteArray* ba, int64_t value);
void vp_bytearray_set(ViperByteArray* ba, int64_t index, int64_t value);
int64_t vp_bytearray_get(ViperByteArray* ba, int64_t index);
void vp_bytearray_extend(ViperByteArray* ba, ViperByteArray* other);
ViperByteArray* vp_bytearray_slice(ViperByteArray* ba, int64_t start, int64_t end, int64_t step);
void vp_bytearray_print(ViperByteArray* ba);
ViperByteArray* vp_bytearray_repeat(int64_t value, int64_t count);
ViperByteArray* vp_bytearray_from_list(ViperList* list);
