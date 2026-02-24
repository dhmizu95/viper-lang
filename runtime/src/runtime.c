/**
 * Viper Runtime - Main Entry Point
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
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
    if (!str) return NULL;

    size_t len = strlen(str);
    char* new_str = (char*)vp_arc_alloc(len + 1);
    strcpy(new_str, str);
    return new_str;
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
/* Utility Functions                            */
/* ============================================ */

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

/* Convert bool to string */
char* vp_str_from_bool(bool val) {
    return vp_str_create(val ? "True" : "False");
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
