#ifndef VIPER_STDLIB_H
#define VIPER_STDLIB_H

#include <stdint.h>
#include <stddef.h>

// Print functions - bridge to C stdio
void vp_print_i64(int64_t val);
void vp_print_f64(double val);
void vp_print_str(const char* val);
void vp_print_bool(int val);

// Basic I/O
int64_t vp_read_i64(void);
double vp_read_f64(void);

// String operations
char* vp_str_concat(const char* a, const char* b);
int64_t vp_str_len(const char* s);
char* vp_str_slice(const char* s, int64_t start, int64_t end);

// Memory management (Phase 2)
void* vp_alloc(size_t size);
void vp_free(void* ptr);

#endif // VIPER_STDLIB_H
