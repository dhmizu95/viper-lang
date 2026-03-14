/**
 * Viper Bool List Implementation
 * Type-specific list for bool elements (1 byte per element)
 * 
 * Memory savings: 8x compared to generic int64_t list
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_stdlib.h"

#define LIST_INITIAL_CAPACITY 8
#define LIST_GROWTH_FACTOR 2

/* ============================================ */
/* Bool List Internal Functions                 */
/* ============================================ */

static inline void vp_list_bool_grow(ViperList* list) {
    int64_t new_capacity = list->capacity * LIST_GROWTH_FACTOR;
    int8_t* new_data = (int8_t*)realloc(list->data.data_bool, new_capacity * sizeof(int8_t));

    if (!new_data) {
        vp_panic("Failed to grow bool list");
    }

    list->data.data_bool = new_data;
    list->capacity = new_capacity;
}

static void vp_list_bool_destroy(void* ptr) {
    ViperList* list = (ViperList*)ptr;
    if (list->data.data_bool) {
        free(list->data.data_bool);
        list->data.data_bool = NULL;
    }
}

/* ============================================ */
/* Bool List Public Functions                   */
/* ============================================ */

ViperList* vp_list_bool_create_with_capacity(int64_t cap) {
    ViperList* list = (ViperList*)vp_arc_alloc(sizeof(ViperList));
    
    list->length = 0;
    list->capacity = cap > 0 ? cap : LIST_INITIAL_CAPACITY;
    list->elem_type = VIPER_LIST_BOOL;
    list->data.data_bool = (int8_t*)malloc(list->capacity * sizeof(int8_t));
    
    if (!list->data.data_bool) {
        vp_panic("Failed to allocate bool list data");
    }
    
    vp_arc_set_destructor(list, vp_list_bool_destroy);
    
    return list;
}

ViperList* vp_list_bool_create(void) {
    return vp_list_bool_create_with_capacity(LIST_INITIAL_CAPACITY);
}

void vp_list_bool_free(ViperList* list) {
    if (!list) return;
    vp_arc_release(list);
}

/* OPTIMIZED: Minimal checks for hot path */
void vp_list_bool_append(ViperList* list, bool value) {
    if (list->length >= list->capacity) {
        vp_list_bool_grow(list);
    }
    list->data.data_bool[list->length++] = value ? 1 : 0;
}

void vp_list_bool_insert(ViperList* list, int64_t index, bool value) {
    if (!list) {
        vp_panic("Cannot insert into NULL list");
        return;
    }
    
    if (index < 0 || index > list->length) {
        vp_panic("Bool list index out of range");
        return;
    }
    
    if (list->length >= list->capacity) {
        vp_list_bool_grow(list);
    }
    
    /* Shift elements to the right */
    for (int64_t i = list->length; i > index; i--) {
        list->data.data_bool[i] = list->data.data_bool[i - 1];
    }
    
    list->data.data_bool[index] = value ? 1 : 0;
    list->length++;
}

bool vp_list_bool_remove(ViperList* list, int64_t index) {
    if (!list) {
        vp_panic("Cannot remove from NULL list");
        return false;
    }
    
    if (index < 0 || index >= list->length) {
        vp_panic("Bool list index out of range");
        return false;
    }
    
    bool value = list->data.data_bool[index] != 0;
    
    /* Shift elements to the left */
    for (int64_t i = index; i < list->length - 1; i++) {
        list->data.data_bool[i] = list->data.data_bool[i + 1];
    }
    
    list->length--;
    return value;
}

bool vp_list_bool_pop(ViperList* list) {
    if (!list) {
        vp_panic("Cannot pop from NULL list");
        return false;
    }
    
    if (list->length == 0) {
        vp_panic("Cannot pop from empty bool list");
        return false;
    }
    
    list->length--;
    return list->data.data_bool[list->length] != 0;
}

void vp_list_bool_clear(ViperList* list) {
    if (!list) return;
    list->length = 0;
}

/* OPTIMIZED: Minimal checks for hot path - assume valid index */
/* Supports negative indexing: list[-1] = last element */
bool vp_list_bool_get(ViperList* list, int64_t index) {
    /* Handle negative indexing */
    if (index < 0) {
        index = list->length + index;
    }
    return list->data.data_bool[index] != 0;
}

/* OPTIMIZED: Minimal checks for hot path - assume valid index */
/* Supports negative indexing: list[-1] = last element */
void vp_list_bool_set(ViperList* list, int64_t index, bool value) {
    /* Handle negative indexing */
    if (index < 0) {
        index = list->length + index;
    }
    list->data.data_bool[index] = value ? 1 : 0;
}

bool vp_list_bool_contains(ViperList* list, bool value) {
    if (!list) return false;
    
    int8_t target = value ? 1 : 0;
    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_bool[i] == target) {
            return true;
        }
    }
    
    return false;
}

ViperList* vp_list_bool_copy(ViperList* list) {
    if (!list) return NULL;
    
    ViperList* copy = vp_list_bool_create_with_capacity(list->capacity);
    copy->length = list->length;
    memcpy(copy->data.data_bool, list->data.data_bool, list->length * sizeof(int8_t));
    
    return copy;
}

ViperList* vp_list_bool_repeat(bool elem, int64_t count) {
    ViperList* list = vp_list_bool_create_with_capacity(count);
    list->length = count;
    
    int8_t value = elem ? 1 : 0;
    for (int64_t i = 0; i < count; i++) {
        list->data.data_bool[i] = value;
    }
    
    return list;
}

void vp_list_bool_init_stack(ViperList* list, int8_t* buffer, int64_t count, bool elem) {
    if (!list || !buffer) return;
    list->length = count;
    list->capacity = count;
    list->elem_type = VIPER_LIST_BOOL;
    list->data.data_bool = buffer;
    
    int8_t value = elem ? 1 : 0;
    memset(buffer, value, count); // Fast initialization
}

void vp_list_bool_print(ViperList* list) {
    if (!list) {
        printf("None");
        return;
    }
    
    printf("[");
    for (int64_t i = 0; i < list->length; i++) {
        if (i > 0) printf(", ");
        printf(list->data.data_bool[i] ? "True" : "False");
    }
    printf("]");
}

/* Extended bool list operations */
void vp_list_bool_extend(ViperList* list, ViperList* other) {
    if (!list || !other) return;
    
    while (list->length + other->length > list->capacity) {
        vp_list_bool_grow(list);
    }
    
    memcpy(list->data.data_bool + list->length, 
           other->data.data_bool, 
           other->length * sizeof(int8_t));
    list->length += other->length;
}

int64_t vp_list_bool_index(ViperList* list, bool value) {
    if (!list) return -1;
    
    int8_t target = value ? 1 : 0;
    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_bool[i] == target) {
            return i;
        }
    }
    
    return -1;
}

int64_t vp_list_bool_count(ViperList* list, bool value) {
    if (!list) return 0;
    
    int8_t target = value ? 1 : 0;
    int64_t count = 0;
    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_bool[i] == target) {
            count++;
        }
    }
    
    return count;
}

void vp_list_bool_reverse(ViperList* list) {
    if (!list) return;
    
    int64_t left = 0;
    int64_t right = list->length - 1;
    while (left < right) {
        int8_t temp = list->data.data_bool[left];
        list->data.data_bool[left] = list->data.data_bool[right];
        list->data.data_bool[right] = temp;
        left++;
        right--;
    }
}

ViperList* vp_list_bool_reversed(ViperList* list) {
    if (!list) return NULL;
    
    ViperList* copy = vp_list_bool_copy(list);
    vp_list_bool_reverse(copy);
    return copy;
}

ViperList* vp_list_bool_slice(ViperList* list, int64_t start, int64_t end, int64_t step) {
    if (!list) return NULL;
    
    // Handle negative indices
    if (start < 0) start = list->length + start;
    if (end < 0) end = list->length + end;
    
    // Clamp to valid range
    if (start < 0) start = 0;
    if (end > list->length) end = list->length;
    
    if (step == 0) step = 1;
    
    // Calculate result size
    int64_t result_len = 0;
    if (step > 0) {
        if (start < end) result_len = (end - start + step - 1) / step;
    } else {
        if (start > end) result_len = (start - end - step - 1) / (-step);
    }
    
    ViperList* result = vp_list_bool_create_with_capacity(result_len);
    
    if (step > 0) {
        for (int64_t i = start; i < end && i < list->length; i += step) {
            vp_list_bool_append(result, list->data.data_bool[i] != 0);
        }
    } else {
        for (int64_t i = start; i > end && i >= 0; i += step) {
            vp_list_bool_append(result, list->data.data_bool[i] != 0);
        }
    }
    
    return result;
}

ViperList* vp_list_bool_concat(ViperList* list1, ViperList* list2) {
    if (!list1 || !list2) return NULL;
    
    ViperList* result = vp_list_bool_create_with_capacity(list1->length + list2->length);
    vp_list_bool_extend(result, list1);
    vp_list_bool_extend(result, list2);
    
    return result;
}
