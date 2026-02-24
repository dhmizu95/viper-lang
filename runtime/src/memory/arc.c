/**
 * Viper ARC (Automatic Reference Counting) Implementation
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_arc.h"

/* ============================================ */
/* Core ARC Functions                           */
/* ============================================ */

void* vp_arc_alloc(size_t size) {
    /* Allocate header + object */
    size_t total_size = sizeof(ViperHeader) + size;
    void* memory = malloc(total_size);
    
    if (!memory) {
        fprintf(stderr, "Viper ARC: Out of memory (requested %zu bytes)\n", size);
        exit(1);
    }
    
    /* Initialize header */
    ViperHeader* header = (ViperHeader*)memory;
    atomic_store(&header->ref_count, 1);
    header->destructor = NULL;
    
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
    free(header);
}

void vp_arc_retain(void* ptr) {
    if (!ptr) return;
    
    ViperHeader* header = VP_GET_HEADER(ptr);
    atomic_fetch_add(&header->ref_count, 1);
}

void vp_arc_release(void* ptr) {
    if (!ptr) return;
    
    ViperHeader* header = VP_GET_HEADER(ptr);
    int64_t old_count = atomic_fetch_sub(&header->ref_count, 1);
    
    if (old_count == 1) {
        /* Reference count reached zero, free the object */
        vp_arc_free(ptr);
    }
}

int64_t vp_arc_ref_count(void* ptr) {
    if (!ptr) return 0;
    
    ViperHeader* header = VP_GET_HEADER(ptr);
    return atomic_load(&header->ref_count);
}

void vp_arc_set_destructor(void* ptr, void (*destructor)(void*)) {
    if (!ptr) return;
    
    ViperHeader* header = VP_GET_HEADER(ptr);
    header->destructor = destructor;
}

/* ============================================ */
/* Wrapper Functions (for stdlib.h)             */
/* ============================================ */

void* vp_alloc(size_t size) {
    return vp_arc_alloc(size);
}

void vp_free(void* ptr) {
    vp_arc_release(ptr);
}

void vp_retain(void* ptr) {
    vp_arc_retain(ptr);
}

void vp_release(void* ptr) {
    vp_arc_release(ptr);
}

int64_t vp_ref_count(void* ptr) {
    return vp_arc_ref_count(ptr);
}
