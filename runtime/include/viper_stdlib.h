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

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

void vp_panic(const char* message);
void vp_assert(bool condition, const char* message);

/* ============================================ */
/* Built-in Functions (callable from Viper)     */
/* ============================================ */

int64_t vp_builtin_range_start(int64_t start, int64_t end);
int64_t vp_builtin_range_next(int64_t* state);
ViperList* vp_builtin_range_list(int64_t end);

#endif /* VIPER_STDLIB_H */
