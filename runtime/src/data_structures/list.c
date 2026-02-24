/**
 * Viper Dynamic List Implementation
 * A resizable array with reference counting
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_stdlib.h"

#define LIST_INITIAL_CAPACITY 8
#define LIST_GROWTH_FACTOR 2

/* ============================================ */
/* List Internal Functions                      */
/* ============================================ */

static void vp_list_grow(ViperList* list) {
    int64_t new_capacity = list->capacity * LIST_GROWTH_FACTOR;
    int64_t* new_data = (int64_t*)realloc(list->data, new_capacity * sizeof(int64_t));
    
    if (!new_data) {
        vp_panic("Failed to grow list");
    }
    
    list->data = new_data;
    list->capacity = new_capacity;
}

static void vp_list_destroy(void* ptr) {
    ViperList* list = (ViperList*)ptr;
    if (list->data) {
        free(list->data);
        list->data = NULL;
    }
}

/* ============================================ */
/* List Public Functions                        */
/* ============================================ */

ViperList* vp_list_create(void) {
    ViperList* list = (ViperList*)vp_arc_alloc(sizeof(ViperList));
    
    list->ref_count = 1;
    list->length = 0;
    list->capacity = LIST_INITIAL_CAPACITY;
    list->data = (int64_t*)malloc(list->capacity * sizeof(int64_t));
    
    if (!list->data) {
        vp_panic("Failed to allocate list data");
    }
    
    vp_arc_set_destructor(list, vp_list_destroy);
    
    return list;
}

void vp_list_free(ViperList* list) {
    if (!list) return;
    vp_arc_release(list);
}

void vp_list_append(ViperList* list, int64_t value) {
    if (!list) {
        vp_panic("Cannot append to NULL list");
        return;
    }
    
    if (list->length >= list->capacity) {
        vp_list_grow(list);
    }
    
    list->data[list->length] = value;
    list->length++;
}

void vp_list_insert(ViperList* list, int64_t index, int64_t value) {
    if (!list) {
        vp_panic("Cannot insert into NULL list");
        return;
    }
    
    if (index < 0 || index > list->length) {
        vp_panic("List index out of range");
        return;
    }
    
    if (list->length >= list->capacity) {
        vp_list_grow(list);
    }
    
    /* Shift elements to the right */
    for (int64_t i = list->length; i > index; i--) {
        list->data[i] = list->data[i - 1];
    }
    
    list->data[index] = value;
    list->length++;
}

int64_t vp_list_remove(ViperList* list, int64_t index) {
    if (!list) {
        vp_panic("Cannot remove from NULL list");
        return 0;
    }
    
    if (index < 0 || index >= list->length) {
        vp_panic("List index out of range");
        return 0;
    }
    
    int64_t value = list->data[index];
    
    /* Shift elements to the left */
    for (int64_t i = index; i < list->length - 1; i++) {
        list->data[i] = list->data[i + 1];
    }
    
    list->length--;
    return value;
}

int64_t vp_list_pop(ViperList* list) {
    if (!list) {
        vp_panic("Cannot pop from NULL list");
        return 0;
    }
    
    if (list->length == 0) {
        vp_panic("Cannot pop from empty list");
        return 0;
    }
    
    list->length--;
    return list->data[list->length];
}

void vp_list_clear(ViperList* list) {
    if (!list) return;
    list->length = 0;
}

int64_t vp_list_get(ViperList* list, int64_t index) {
    if (!list) {
        vp_panic("Cannot get from NULL list");
        return 0;
    }
    
    if (index < 0 || index >= list->length) {
        vp_panic("List index out of range");
        return 0;
    }
    
    return list->data[index];
}

void vp_list_set(ViperList* list, int64_t index, int64_t value) {
    if (!list) {
        vp_panic("Cannot set on NULL list");
        return;
    }
    
    if (index < 0 || index >= list->length) {
        vp_panic("List index out of range");
        return;
    }
    
    list->data[index] = value;
}

int64_t vp_list_len(ViperList* list) {
    if (!list) return 0;
    return list->length;
}

bool vp_list_contains(ViperList* list, int64_t value) {
    if (!list) return false;
    
    for (int64_t i = 0; i < list->length; i++) {
        if (list->data[i] == value) {
            return true;
        }
    }
    
    return false;
}

ViperList* vp_list_copy(ViperList* list) {
    if (!list) return NULL;
    
    ViperList* copy = vp_list_create();
    
    if (list->length > 0) {
        /* Ensure capacity */
        while (copy->capacity < list->length) {
            vp_list_grow(copy);
        }
        
        memcpy(copy->data, list->data, list->length * sizeof(int64_t));
        copy->length = list->length;
    }
    
    return copy;
}
