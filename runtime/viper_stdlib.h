#ifndef VIPER_STDLIB_H
#define VIPER_STDLIB_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

// Print functions - bridge to C stdio
void vp_print_i64(int64_t val);
void vp_print_f64(double val);
void vp_print_bool(int val);

// Basic I/O
int64_t vp_read_i64(void);
double vp_read_f64(void);

// String operations - declarations removed, now static inline in viper_types.h
// The following functions are defined in viper_types.h:
//   - vp_str_create, vp_str_free, vp_str_concat, vp_str_len, vp_str_slice
//   - vp_str_equals, vp_str_from_bool, vp_str_from_i64, vp_str_from_f64

// Memory management (Phase 2)
void* vp_alloc(size_t size);
void vp_free(void* ptr);

// Concurrency (Phase 3) - Opaque types
typedef struct ViperChannel ViperChannel;
typedef struct ViperWaitGroup ViperWaitGroup;

// Channel operations
ViperChannel* vp_chan_create(int64_t capacity);
void vp_chan_destroy(ViperChannel* chan);
void vp_chan_send(ViperChannel* chan, int64_t value);
int64_t vp_chan_recv(ViperChannel* chan);

// WaitGroup operations
ViperWaitGroup* vp_waitgroup_create(void);
void vp_waitgroup_destroy(ViperWaitGroup* wg);
void vp_waitgroup_add(ViperWaitGroup* wg, int64_t n);
void vp_waitgroup_done(ViperWaitGroup* wg);
void vp_waitgroup_wait(ViperWaitGroup* wg);

// Thread pool (global)
void vp_init_threadpool(size_t num_threads);
void vp_shutdown_threadpool(void);
void vp_submit_task(void (*func)(void*), void* data);

#endif // VIPER_STDLIB_H
