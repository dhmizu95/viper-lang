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
/* Basic I/O Functions                          */
/* ============================================ */

void vp_print_i64(int64_t val);
void vp_print_f64(double val);
void vp_print_str(const char* val);
void vp_print_bool(bool val);
void vp_print_newline(void);

/* ============================================ */
/* Memory Management (ARC)                      */
/* ============================================ */

void* vp_alloc(size_t size);
void vp_free(void* ptr);
void vp_retain(void* ptr);
void vp_release(void* ptr);
int64_t vp_ref_count(void* ptr);

/* ============================================ */
/* List Functions                               */
/* ============================================ */

ViperList* vp_list_create(void);
ViperList* vp_list_create_with_capacity(int64_t cap);
void vp_list_free(ViperList* list);
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
ViperList* vp_list_slice(ViperList* list, int64_t start, int64_t end, int64_t step);
void vp_list_print(ViperList* list);

/* Extended list operations */
void vp_list_extend(ViperList* list, ViperList* other);
int64_t vp_list_index(ViperList* list, int64_t value);
int64_t vp_list_count(ViperList* list, int64_t value);
void vp_list_sort(ViperList* list);
void vp_list_reverse(ViperList* list);
ViperList* vp_list_reversed(ViperList* list);
ViperList* vp_list_sorted(ViperList* list);
ViperList* vp_list_concat(ViperList* list1, ViperList* list2);

/* ============================================ */
/* Dictionary Functions                         */
/* ============================================ */

ViperDict* vp_dict_create(void);
ViperDict* vp_dict_create_with_capacity(int64_t initial_cap);
void vp_dict_free(ViperDict* dict);
void vp_dict_set(ViperDict* dict, const char* key, ViperValue value);
ViperValue vp_dict_get(ViperDict* dict, const char* key);
bool vp_dict_contains(ViperDict* dict, const char* key);
bool vp_dict_remove(ViperDict* dict, const char* key);
void vp_dict_clear(ViperDict* dict);
int64_t vp_dict_len(ViperDict* dict);
ViperDict* vp_dict_copy(ViperDict* dict);
void vp_dict_print(ViperDict* dict);

/* Dict set with ViperString key (for compiler codegen) */
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
bool vp_dict_iter_next(ViperDictIter* iter, const char** key, ViperValue* value);

/* ============================================ */
/* String Functions                             */
/* ============================================ */

char* vp_str_create(const char* str);
void vp_str_free(char* str);
char* vp_str_concat(const char* a, const char* b);
int64_t vp_str_len(const char* str);
char* vp_str_slice(const char* str, int64_t start, int64_t end);
bool vp_str_equals(const char* a, const char* b);
int64_t vp_str_compare(const char* a, const char* b);
char* vp_str_upper(const char* str);
char* vp_str_lower(const char* str);
ViperList* vp_str_split(const char* str, const char* delim);
char* vp_str_replace(const char* str, const char* old_sub, const char* new_sub);


/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

void vp_panic(const char* message);
void vp_assert(bool condition, const char* message);

/* Hash functions */
int64_t vp_hash_i64(int64_t val);
int64_t vp_hash_f64(double val);
int64_t vp_hash_bool(bool val);
int64_t vp_hash_str(const char* str);
int64_t vp_hash_none(void);

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

/* ============================================ */
/* Type Conversion Functions                    */
/* ============================================ */

char* vp_str_from_i64(int64_t val);
char* vp_str_from_f64(double val);
char* vp_str_from_bool(bool val);
int64_t vp_i64_from_str(const char* str);
double vp_f64_from_str(const char* str);
bool vp_bool_from_i64(int64_t val);
bool vp_bool_from_f64(double val);
bool vp_bool_from_str(const char* str);

#endif /* VIPER_STDLIB_H */
