/**
 * Viper ARC (Automatic Reference Counting) Header
 */

#ifndef VIPER_ARC_H
#define VIPER_ARC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdatomic.h>

/* ============================================ */
/* ARC Header Structure                         */
/* ============================================ */

/*
 * All heap-allocated objects have this header prepended.
 * The header contains the reference count.
 *
 * Memory layout:
 *   [ViperHeader][Object Data...]
 *    ^           ^
 *    |           |
 *  returned    user gets
 *  from        this pointer
 *  malloc
 *
 * Flags field:
 *   Bit 0: VIPER_ARC_FLAG_SHARED - object may be shared across threads (uses atomic ref count)
 *   Bit 1: VIPER_ARC_FLAG_POOL   - object allocated from pool allocator
 *   Bit 2: VIPER_ARC_FLAG_LOCAL  - object is thread-local (uses non-atomic ref count)
 *   Bits 3-7: Reserved for future use
 */

#define VIPER_ARC_FLAG_SHARED 0x01
#define VIPER_ARC_FLAG_POOL   0x02
#define VIPER_ARC_FLAG_LOCAL  0x04

#ifndef VIPER_POOL_MAX_SIZE
#define VIPER_POOL_MAX_SIZE 256
#endif
#ifndef VIPER_POOL_CAPACITY
#define VIPER_POOL_CAPACITY 1024
#endif

typedef struct {
    union {
        _Atomic int64_t ref_count_atomic;  /* For shared objects */
        int64_t ref_count;                  /* For local objects */
    };
    void (*destructor)(void*);
    uint8_t flags;  /* Object flags (shared, pooled, local) */
    uint8_t reserved[7];  /* Padding for alignment */
} ViperHeader;

/* ============================================ */
/* ARC Core Functions                           */
/* ============================================ */

/**
 * Allocate memory with ARC header
 * @param size Size of the object (excluding header)
 * @return Pointer to the object (after header)
 */
void* vp_arc_alloc(size_t size);

/**
 * Allocate memory with ARC header (thread-local, non-atomic fast path)
 * @param size Size of the object (excluding header)
 * @return Pointer to the object (after header)
 */
void* vp_arc_alloc_local(size_t size);

/**
 * Free memory with ARC header
 * @param ptr Pointer to the object (after header)
 */
void vp_arc_free(void* ptr);

/**
 * Increment reference count (atomic, thread-safe)
 * @param ptr Pointer to the object
 */
void vp_arc_retain(void* ptr);

/**
 * Increment reference count (non-atomic, thread-local only)
 * @param ptr Pointer to the object
 * @note Only use when object is guaranteed to not be shared across threads
 */
void vp_arc_retain_local(void* ptr);

/**
 * Decrement reference count and free if zero (atomic, thread-safe)
 * @param ptr Pointer to the object
 */
void vp_arc_release(void* ptr);

/**
 * Decrement reference count and free if zero (non-atomic, thread-local only)
 * @param ptr Pointer to the object
 * @note Only use when object is guaranteed to not be shared across threads
 */
void vp_arc_release_local(void* ptr);

/**
 * Batch release multiple objects (reduces function call overhead)
 * @param ptrs Array of object pointers
 * @param count Number of pointers
 */
void vp_arc_release_batch(void** ptrs, size_t count);

/**
 * Batch release multiple local objects (returns to pool / free without atomics)
 * @param ptrs Array of object pointers
 * @param count Number of pointers
 */
void vp_arc_release_batch_local(void** ptrs, size_t count);

/**
 * Get current reference count
 * @param ptr Pointer to the object
 * @return Current reference count
 */
int64_t vp_arc_ref_count(void* ptr);

/**
 * Set destructor callback for when object is freed
 * @param ptr Pointer to the object
 * @param destructor Function to call before freeing
 */
void vp_arc_set_destructor(void* ptr, void (*destructor)(void*));

/**
 * Mark object as shared (may be accessed across threads)
 * @param ptr Pointer to the object
 */
void vp_arc_mark_shared(void* ptr);

/**
 * Check if object is marked as shared
 * @param ptr Pointer to the object
 * @return true if shared, false otherwise
 */
bool vp_arc_is_shared(void* ptr);

/* ============================================ */
/* Helper Macros                                */
/* ============================================ */

/* Get header from object pointer */
#define VP_GET_HEADER(ptr) ((ViperHeader*)(ptr) - 1)

/* Get object pointer from header */
#define VP_GET_OBJECT(header) ((void*)((ViperHeader*)(header) + 1))

/* Atomic increment with relaxed ordering */
#define VP_ATOMIC_INC(ptr) atomic_fetch_add_explicit((ptr), 1, memory_order_relaxed)

/* Atomic decrement with release ordering */
#define VP_ATOMIC_DEC(ptr) atomic_fetch_sub_explicit((ptr), 1, memory_order_release)

/* Atomic load with relaxed ordering */
#define VP_ATOMIC_LOAD(ptr) atomic_load_explicit((ptr), memory_order_relaxed)

/* Atomic store with relaxed ordering */
#define VP_ATOMIC_STORE(ptr, val) atomic_store_explicit((ptr), (val), memory_order_relaxed)

/* Non-atomic increment (for thread-local objects) */
#define VP_NON_ATOMIC_INC(ptr) do { (*(ptr))++; } while (0)

/* Non-atomic decrement (for thread-local objects) */
#define VP_NON_ATOMIC_DEC(ptr) do { (*(ptr))--; } while (0)

/* Check if object is shared */
#define VP_IS_SHARED(header) ((header)->flags & VIPER_ARC_FLAG_SHARED)

/* Mark object as shared */
#define VP_MARK_SHARED(header) do { (header)->flags |= VIPER_ARC_FLAG_SHARED; } while (0)

/* Check if object is local (non-atomic ref count) */
#define VP_IS_LOCAL(header) ((header)->flags & VIPER_ARC_FLAG_LOCAL)

/* Mark object as local (non-atomic ref count) */
#define VP_MARK_LOCAL(header) do { (header)->flags |= VIPER_ARC_FLAG_LOCAL; } while (0)

/* Check if object is pooled */
#define VP_IS_POOLED(header) ((header)->flags & VIPER_ARC_FLAG_POOL)

/* Mark object as pooled */
#define VP_MARK_POOLED(header) do { (header)->flags |= VIPER_ARC_FLAG_POOL; } while (0)

/* Get atomic ref count (for shared objects) */
#define VP_GET_ATOMIC_REF(header) (&(header)->ref_count_atomic)

/* Get non-atomic ref count (for local objects) */
#define VP_GET_REF(header) (&(header)->ref_count)

#endif /* VIPER_ARC_H */
