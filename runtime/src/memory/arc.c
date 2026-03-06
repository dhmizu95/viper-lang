/**
 * Viper ARC (Automatic Reference Counting) Implementation
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_arc.h"
#include "pool.h"

/* ============================================ */
/* Thread-Local Pool                            */
/* ============================================ */

static __thread VpObjectPool* arc_pool = NULL;
static __thread bool arc_pool_initialized = false;
static __thread size_t arc_pool_max_size = VIPER_POOL_MAX_SIZE;

static void ensure_pool_initialized(void) {
    if (arc_pool_initialized) {
        return;
    }
    
    arc_pool_initialized = true;
    
    const char* env_size = getenv("VIPER_ARC_POOL_SIZE");
    if (env_size) {
        int size = atoi(env_size);
        if (size <= 0) {
            arc_pool_max_size = 0; // Disable pool
            return;
        }
        arc_pool_max_size = (size_t)size;
    }
    
    size_t block_size = sizeof(ViperHeader) + arc_pool_max_size;
    arc_pool = vp_pool_create(block_size, VIPER_POOL_CAPACITY);
}

/* ============================================ */
/* Core ARC Functions                           */
/* ============================================ */

void* vp_arc_alloc(size_t size) {
    ensure_pool_initialized();

    /* Allocate header + object */
    size_t total_size = sizeof(ViperHeader) + size;
    void* memory = NULL;
    bool from_pool = false;

    if (arc_pool && size <= arc_pool_max_size) {
        memory = vp_pool_alloc(arc_pool);
        if (memory) {
            from_pool = true;
        }
    }

    if (!memory) {
        memory = malloc(total_size);
        from_pool = false;
    }

    if (!memory) {
        fprintf(stderr, "Viper ARC: Out of memory (requested %zu bytes)\n", size);
        exit(1);
    }

    /* Initialize header */
    ViperHeader* header = (ViperHeader*)memory;
    atomic_store_explicit(&header->ref_count_atomic, 1, memory_order_relaxed);
    header->destructor = NULL;
    header->flags = 0;  /* Not shared by default */
    if (from_pool) {
        VP_MARK_POOLED(header);
    }

    /* Return pointer to object (after header) */
    return VP_GET_OBJECT(header);
}

void* vp_arc_alloc_local(size_t size) {
    ensure_pool_initialized();

    /* Allocate header + object */
    size_t total_size = sizeof(ViperHeader) + size;
    void* memory = NULL;
    bool from_pool = false;

    if (arc_pool && size <= arc_pool_max_size) {
        memory = vp_pool_alloc(arc_pool);
        if (memory) {
            from_pool = true;
        }
    }

    if (!memory) {
        memory = malloc(total_size);
        from_pool = false;
    }

    if (!memory) {
        fprintf(stderr, "Viper ARC: Out of memory (requested %zu bytes)\n", size);
        exit(1);
    }

    /* Initialize header - non-atomic ref count for thread-local objects */
    ViperHeader* header = (ViperHeader*)memory;
    header->ref_count = 1;  /* Non-atomic store */
    header->destructor = NULL;
    header->flags = VIPER_ARC_FLAG_LOCAL;  /* Mark as local */
    if (from_pool) {
        VP_MARK_POOLED(header);
    }

    /* Return pointer to object (after header) */
    return VP_GET_OBJECT(header);
}

void vp_arc_free(void* ptr) {
    if (!ptr) return;
    
    ViperHeader* header = VP_GET_HEADER(ptr);
    
    /* Call destructor if set */
    if (header->destructor) {
        header->destructor(ptr);
    }
    
    /* Free the memory */
    if (VP_IS_POOLED(header)) {
        ensure_pool_initialized();
        if (arc_pool) {
            vp_pool_free(arc_pool, header);
        }
    } else {
        free(header);
    }
}

void vp_arc_retain(void* ptr) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);
    atomic_fetch_add_explicit(&header->ref_count_atomic, 1, memory_order_relaxed);
}

void vp_arc_retain_local(void* ptr) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);
    /* Non-atomic increment for thread-local objects */
    header->ref_count++;
}

void vp_arc_release(void* ptr) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);

    /* Use release ordering to ensure all prior writes are visible */
    int64_t old_count = atomic_fetch_sub_explicit(&header->ref_count_atomic, 1, memory_order_release);

    if (old_count == 1) {
        /* Reference count reached zero, need acquire fence before destroy */
        atomic_thread_fence(memory_order_acquire);

        /* Call destructor if set */
        if (header->destructor) {
            header->destructor(ptr);
        }

        /* Free the memory */
        if (!VP_IS_STACK(header)) {
            if (VP_IS_POOLED(header)) {
                ensure_pool_initialized();
                if (arc_pool) {
                    vp_pool_free(arc_pool, header);
                }
            } else {
                free(header);
            }
        }
    }
}

void vp_arc_release_local(void* ptr) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);

    /* Non-atomic decrement for thread-local objects */
    header->ref_count--;

    if (header->ref_count == 0) {
        /* Call destructor if set */
        if (header->destructor) {
            header->destructor(ptr);
        }

        /* Free the memory */
        if (!VP_IS_STACK(header)) {
            if (VP_IS_POOLED(header)) {
                ensure_pool_initialized();
                if (arc_pool) {
                    vp_pool_free(arc_pool, header);
                }
            } else {
                free(header);
            }
        }
    }
}

void vp_arc_release_batch(void** ptrs, size_t count) {
    if (!ptrs || count == 0) return;

    for (size_t i = 0; i < count; i++) {
        vp_arc_release(ptrs[i]);
    }
}

void vp_arc_release_batch_local(void** ptrs, size_t count) {
    if (!ptrs || count == 0) return;

    for (size_t i = 0; i < count; i++) {
        vp_arc_release_local(ptrs[i]);
    }
}

int64_t vp_arc_ref_count(void* ptr) {
    if (!ptr) return 0;

    ViperHeader* header = VP_GET_HEADER(ptr);
    return atomic_load_explicit(&header->ref_count_atomic, memory_order_relaxed);
}

void vp_arc_set_destructor(void* ptr, void (*destructor)(void*)) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);
    header->destructor = destructor;
}

void vp_arc_mark_shared(void* ptr) {
    if (!ptr) return;

    ViperHeader* header = VP_GET_HEADER(ptr);
    header->flags |= VIPER_ARC_FLAG_SHARED;
}

bool vp_arc_is_shared(void* ptr) {
    if (!ptr) return false;

    ViperHeader* header = VP_GET_HEADER(ptr);
    return (header->flags & VIPER_ARC_FLAG_SHARED) != 0;
}

/* ============================================ */
/* Wrapper Functions (for stdlib.h)             */
/* ============================================ */

void* vp_alloc(size_t size) {
    return vp_arc_alloc(size);
}

void* vp_alloc_local(size_t size) {
    return vp_arc_alloc_local(size);
}

void vp_free(void* ptr) {
    vp_arc_release(ptr);
}

void vp_retain(void* ptr) {
    vp_arc_retain(ptr);
}

void vp_retain_local(void* ptr) {
    vp_arc_retain_local(ptr);
}

void vp_release(void* ptr) {
    vp_arc_release(ptr);
}

void vp_release_local(void* ptr) {
    vp_arc_release_local(ptr);
}

void vp_release_batch(void** ptrs, size_t count) {
    vp_arc_release_batch(ptrs, count);
}

void vp_release_batch_local(void** ptrs, size_t count) {
    vp_arc_release_batch_local(ptrs, count);
}

int64_t vp_ref_count(void* ptr) {
    return vp_arc_ref_count(ptr);
}
