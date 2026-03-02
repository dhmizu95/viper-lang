/**
 * Viper Dynamic List Implementation
 * A resizable array with reference counting
 *
 * OPTIMIZED VERSION: Minimal overhead for hot paths
 * Uses inline operations for predictable LLVM IR
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

/* Inline-friendly grow function - called only when needed */
/* Made non-static for inline codegen */
VIPER_NEVER_INLINE void vp_list_grow(ViperList* list) {
    int64_t new_capacity = list->capacity * LIST_GROWTH_FACTOR;
    int64_t* new_data = (int64_t*)realloc(list->data.data_i64, new_capacity * sizeof(int64_t));

    if (!new_data) {
        vp_panic("Failed to grow list");
    }

    list->data.data_i64 = new_data;
    list->capacity = new_capacity;
}

static void vp_list_destroy(void* ptr) {
    ViperList* list = (ViperList*)ptr;
    if (list->data.data_i64) {
        free(list->data.data_i64);
        list->data.data_i64 = NULL;
    }
}

/* ============================================ */
/* List Public Functions - HOT PATH OPTIMIZED   */
/* ============================================ */

ViperList* vp_list_create_with_capacity(int64_t cap) {
    ViperList* list = (ViperList*)vp_arc_alloc(sizeof(ViperList));

    list->ref_count = 1;
    list->length = 0;
    list->capacity = cap > 0 ? cap : LIST_INITIAL_CAPACITY;
    list->elem_type = VIPER_LIST_I64;
    list->data.data_i64 = (int64_t*)malloc(list->capacity * sizeof(int64_t));

    if (!list->data.data_i64) {
        vp_panic("Failed to allocate list data");
    }

    vp_arc_set_destructor(list, vp_list_destroy);

    return list;
}

ViperList* vp_list_create(void) {
    ViperList* list = (ViperList*)vp_arc_alloc(sizeof(ViperList));

    list->ref_count = 1;
    list->length = 0;
    list->capacity = LIST_INITIAL_CAPACITY;
    list->elem_type = VIPER_LIST_I64;
    list->data.data_i64 = (int64_t*)malloc(list->capacity * sizeof(int64_t));

    if (!list->data.data_i64) {
        vp_panic("Failed to allocate list data");
    }

    vp_arc_set_destructor(list, vp_list_destroy);

    return list;
}

void vp_list_free(ViperList* list) {
    if (!list) return;
    vp_arc_release(list);
}

/* Reserve capacity - pre-allocate memory for efficient append */
void vp_list_reserve(ViperList* list, int64_t capacity) {
    if (!list) {
        vp_panic("Cannot reserve capacity for NULL list");
        return;
    }
    if (capacity <= 0) {
        return;  /* Nothing to reserve */
    }
    if (capacity <= list->capacity) {
        return;  /* Already have enough capacity */
    }

    int64_t* new_data = (int64_t*)realloc(list->data.data_i64, capacity * sizeof(int64_t));
    if (!new_data) {
        vp_panic("Failed to reserve list capacity");
    }

    list->data.data_i64 = new_data;
    list->capacity = capacity;
}

/* OPTIMIZED: Minimal checks for hot path */
void vp_list_append(ViperList* list, int64_t value) {
    if (list->length >= list->capacity) {
        vp_list_grow(list);
    }
    list->data.data_i64[list->length++] = value;
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
        list->data.data_i64[i] = list->data.data_i64[i - 1];
    }

    list->data.data_i64[index] = value;
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

    int64_t value = list->data.data_i64[index];

    /* Shift elements to the left */
    for (int64_t i = index; i < list->length - 1; i++) {
        list->data.data_i64[i] = list->data.data_i64[i + 1];
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
    return list->data.data_i64[list->length];
}

void vp_list_clear(ViperList* list) {
    if (!list) return;
    list->length = 0;
}

/* OPTIMIZED: Minimal checks for hot path - assume valid index */
/* Supports negative indexing: list[-1] = last element */
int64_t vp_list_get(ViperList* list, int64_t index) {
    /* Handle negative indexing */
    if (index < 0) {
        index = list->length + index;
    }
    if (list->elem_type == VIPER_LIST_BOOL) {
        return list->data.data_bool[index];
    }
    return list->data.data_i64[index];
}

/* OPTIMIZED: Minimal checks for hot path - assume valid index */
/* Supports negative indexing: list[-1] = last element */
void vp_list_set(ViperList* list, int64_t index, int64_t value) {
    /* Handle negative indexing */
    if (index < 0) {
        index = list->length + index;
    }
    if (list->elem_type == VIPER_LIST_BOOL) {
        list->data.data_bool[index] = value ? 1 : 0;
    } else {
        list->data.data_i64[index] = value;
    }
}

int64_t vp_list_len(ViperList* list) {
    if (!list) return 0;
    return list->length;
}

bool vp_list_contains(ViperList* list, int64_t value) {
    if (!list) return false;

    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_i64[i] == value) {
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

        memcpy(copy->data.data_i64, list->data.data_i64, list->length * sizeof(int64_t));
        copy->length = list->length;
    }

    return copy;
}

/**
 * Print a list in format [elem1, elem2, ...]
 */
void vp_list_print(ViperList* list) {
    if (!list) {
        printf("[]");
        return;
    }

    printf("[");
    for (int64_t i = 0; i < list->length; i++) {
        if (i > 0) {
            printf(", ");
        }
        printf("%ld", (long)list->data.data_i64[i]);
    }
    printf("]");
}

/**
 * Slice a list: list[start:end] or list[start:end:step]
 * Returns a new list containing elements from start to end (exclusive)
 * with the given step
 */
ViperList* vp_list_slice(ViperList* list, int64_t start, int64_t end, int64_t step) {
    if (!list) return NULL;

    /* Normalize negative indices */
    if (start < 0) start = (start + list->length < 0) ? 0 : start + list->length;
    if (end < 0) end = end + list->length;

    /* Clamp to valid range */
    if (start < 0) start = 0;
    if (end > list->length) end = list->length;
    if (start >= end) return vp_list_create();

    /* Calculate result length */
    int64_t result_len = (end - start + step - 1) / step;
    if (result_len < 0) result_len = 0;

    ViperList* result = vp_list_create_with_capacity(result_len);

    for (int64_t i = start; i < end; i += step) {
        vp_list_append(result, list->data.data_i64[i]);
    }

    return result;
}

/**
 * Extend a list by appending all elements from another list
 * list.extend(other)
 */
void vp_list_extend(ViperList* list, ViperList* other) {
    if (!list) {
        vp_panic("Cannot extend NULL list");
        return;
    }
    if (!other) {
        return;  /* Extending with NULL is a no-op */
    }

    /* Ensure capacity */
    int64_t new_length = list->length + other->length;
    while (list->capacity < new_length) {
        vp_list_grow(list);
    }

    /* Copy elements */
    memcpy(list->data.data_i64 + list->length, other->data.data_i64, other->length * sizeof(int64_t));
    list->length = new_length;
}

/**
 * Find the index of the first occurrence of a value
 * list.index(x) - returns index or -1 if not found
 */
int64_t vp_list_index(ViperList* list, int64_t value) {
    if (!list) return -1;

    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_i64[i] == value) {
            return i;
        }
    }

    return -1;  /* Not found */
}

/**
 * Count occurrences of a value in the list
 * list.count(x)
 */
int64_t vp_list_count(ViperList* list, int64_t value) {
    if (!list) return 0;

    int64_t count = 0;
    for (int64_t i = 0; i < list->length; i++) {
        if (list->data.data_i64[i] == value) {
            count++;
        }
    }

    return count;
}

/* Comparison function for qsort */
static int compare_i64(const void* a, const void* b) {
    int64_t val_a = *(const int64_t*)a;
    int64_t val_b = *(const int64_t*)b;
    if (val_a < val_b) return -1;
    if (val_a > val_b) return 1;
    return 0;
}

/**
 * Sort the list in-place (ascending order)
 * list.sort()
 */
void vp_list_sort(ViperList* list) {
    if (!list || list->length <= 1) return;
    qsort(list->data.data_i64, list->length, sizeof(int64_t), compare_i64);
}

/**
 * Reverse the list in-place
 * list.reverse()
 */
void vp_list_reverse(ViperList* list) {
    if (!list || list->length <= 1) return;

    int64_t left = 0;
    int64_t right = list->length - 1;
    while (left < right) {
        int64_t temp = list->data.data_i64[left];
        list->data.data_i64[left] = list->data.data_i64[right];
        list->data.data_i64[right] = temp;
        left++;
        right--;
    }
}

/**
 * Create a reversed copy of the list
 * reversed(list) - returns new list
 */
ViperList* vp_list_reversed(ViperList* list) {
    if (!list) return vp_list_create();

    ViperList* result = vp_list_copy(list);
    vp_list_reverse(result);
    return result;
}

/**
 * Create a sorted copy of the list
 * sorted(list) - returns new list
 */
ViperList* vp_list_sorted(ViperList* list) {
    if (!list) return vp_list_create();

    ViperList* result = vp_list_copy(list);
    vp_list_sort(result);
    return result;
}

/**
 * Concatenate two lists: list1 + list2
 * Returns a new list
 */
ViperList* vp_list_concat(ViperList* list1, ViperList* list2) {
    if (!list1 && !list2) return vp_list_create();
    if (!list1) return vp_list_copy(list2);
    if (!list2) return vp_list_copy(list1);

    int64_t total_len = list1->length + list2->length;
    ViperList* result = vp_list_create_with_capacity(total_len);

    /* Copy first list */
    if (list1->length > 0) {
        memcpy(result->data.data_i64, list1->data.data_i64, list1->length * sizeof(int64_t));
        result->length = list1->length;
    }

    /* Copy second list */
    if (list2->length > 0) {
        memcpy(result->data.data_i64 + list1->length, list2->data.data_i64, list2->length * sizeof(int64_t));
        result->length = total_len;
    }

    return result;
}

/**
 * Create a list by repeating an element n times
 * Optimized for [1] * n pattern (e.g., sieve initialization)
 * Uses memset for byte-sized elements
 */
ViperList* vp_list_repeat(int64_t elem, int64_t count) {
    if (count <= 0) {
        return vp_list_create();
    }
    
    ViperList* list = vp_list_create_with_capacity(count);
    
    /* Fast path: use memset for small integer types */
    if (elem >= 0 && elem <= 255 && list->elem_type == VIPER_LIST_I64) {
        /* For i64 lists with small values, use optimized fill */
        int64_t* data = list->data.data_i64;
        for (int64_t i = 0; i < count; i++) {
            data[i] = elem;
        }
    } else {
        /* Standard fill */
        for (int64_t i = 0; i < count; i++) {
            vp_list_append(list, elem);
        }
    }
    
    return list;
}

/**
 * Create a list of zeros with given capacity
 * Optimized for is_prime = [0] * n pattern
 */
ViperList* vp_list_zeros(int64_t count) {
    return vp_list_repeat(0, count);
}

/**
 * Create a list of ones with given capacity
 * Optimized for is_prime = [1] * n pattern
 */
ViperList* vp_list_ones(int64_t count) {
    return vp_list_repeat(1, count);
}

/**
 * range(start, end) - create a list of integers from start to end-1
 * Python-style range: range(5) -> [0, 1, 2, 3, 4]
 */
ViperList* vp_range(int64_t start, int64_t end) {
    int64_t count = end - start;
    if (count <= 0) {
        return vp_list_create();
    }
    
    ViperList* list = vp_list_create_with_capacity(count);
    for (int64_t i = start; i < end; i++) {
        vp_list_append(list, i);
    }
    return list;
}
