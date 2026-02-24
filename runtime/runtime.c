#include "viper_stdlib.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Print an i64 value
void vp_print_i64(int64_t val) {
    printf("%ld\n", val);
}

// Print an f64 value
void vp_print_f64(double val) {
    printf("%f\n", val);
}

// Print a string value
void vp_print_str(const char* val) {
    printf("%s\n", val ? val : "");
}

// Print a boolean value
void vp_print_bool(int val) {
    printf("%s\n", val ? "True" : "False");
}

// Print newline
void vp_print_newline(void) {
    printf("\n");
}

// Read an i64 from stdin
int64_t vp_read_i64(void) {
    int64_t val;
    if (scanf("%ld", &val) != 1) {
        return 0;
    }
    return val;
}

// Read an f64 from stdin
double vp_read_f64(void) {
    double val;
    if (scanf("%lf", &val) != 1) {
        return 0.0;
    }
    return val;
}

// Concatenate two strings
char* vp_str_concat(const char* a, const char* b) {
    if (!a) a = "";
    if (!b) b = "";
    
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    char* result = (char*)malloc(len_a + len_b + 1);
    
    if (result) {
        memcpy(result, a, len_a);
        memcpy(result + len_a, b, len_b);
        result[len_a + len_b] = '\0';
    }
    
    return result;
}

// Get string length
int64_t vp_str_len(const char* s) {
    return s ? (int64_t)strlen(s) : 0;
}

// Slice a string
char* vp_str_slice(const char* s, int64_t start, int64_t end) {
    if (!s) return NULL;
    
    int64_t len = (int64_t)strlen(s);
    if (start < 0) start = len + start;
    if (end < 0) end = len + end;
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start >= end) return strdup("");
    
    size_t slice_len = (size_t)(end - start);
    char* result = (char*)malloc(slice_len + 1);
    
    if (result) {
        memcpy(result, s + start, slice_len);
        result[slice_len] = '\0';
    }
    
    return result;
}

// Allocate memory
void* vp_alloc(size_t size) {
    return malloc(size);
}

// Free memory
void vp_free(void* ptr) {
    free(ptr);
}
