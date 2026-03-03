/**
 * Viper Runtime - Collections Module
 * Advanced data structures: Deque, Counter, OrderedDict
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include "viper_stdlib.h"

/* ============================================ */
/* Deque (Double-ended Queue)                   */
/* ============================================ */

typedef struct DequeNode {
    int64_t value;
    struct DequeNode* next;
    struct DequeNode* prev;
} DequeNode;

typedef struct ViperDeque {
    DequeNode* head;
    DequeNode* tail;
    int64_t size;
} ViperDeque;

ViperDeque* vp_deque_create(void) {
    ViperDeque* dq = (ViperDeque*)vp_arc_alloc(sizeof(ViperDeque));
    if (dq) {
        dq->head = NULL;
        dq->tail = NULL;
        dq->size = 0;
    }
    return dq;
}

void vp_deque_free(ViperDeque* dq) {
    if (!dq) return;
    
    DequeNode* current = dq->head;
    while (current) {
        DequeNode* next = current->next;
        vp_arc_release(current);
        current = next;
    }
    
    vp_arc_release(dq);
}

void vp_deque_append(ViperDeque* dq, int64_t value) {
    if (!dq) return;
    
    DequeNode* node = (DequeNode*)vp_arc_alloc(sizeof(DequeNode));
    if (!node) return;
    
    node->value = value;
    node->next = NULL;
    node->prev = dq->tail;
    
    if (dq->tail) {
        dq->tail->next = node;
    } else {
        dq->head = node;
    }
    
    dq->tail = node;
    dq->size++;
}

void vp_deque_appendleft(ViperDeque* dq, int64_t value) {
    if (!dq) return;
    
    DequeNode* node = (DequeNode*)vp_arc_alloc(sizeof(DequeNode));
    if (!node) return;
    
    node->value = value;
    node->prev = NULL;
    node->next = dq->head;
    
    if (dq->head) {
        dq->head->prev = node;
    } else {
        dq->tail = node;
    }
    
    dq->head = node;
    dq->size++;
}

int64_t vp_deque_pop(ViperDeque* dq) {
    if (!dq || !dq->tail) return 0;
    
    DequeNode* node = dq->tail;
    int64_t value = node->value;
    
    dq->tail = node->prev;
    if (dq->tail) {
        dq->tail->next = NULL;
    } else {
        dq->head = NULL;
    }
    
    dq->size--;
    vp_arc_release(node);
    
    return value;
}

int64_t vp_deque_popleft(ViperDeque* dq) {
    if (!dq || !dq->head) return 0;
    
    DequeNode* node = dq->head;
    int64_t value = node->value;
    
    dq->head = node->next;
    if (dq->head) {
        dq->head->prev = NULL;
    } else {
        dq->tail = NULL;
    }
    
    dq->size--;
    vp_arc_release(node);
    
    return value;
}

int64_t vp_deque_get(ViperDeque* dq, int64_t index) {
    if (!dq || index < 0 || index >= dq->size) return 0;
    
    DequeNode* node;
    if (index < dq->size / 2) {
        node = dq->head;
        for (int64_t i = 0; i < index; i++) {
            node = node->next;
        }
    } else {
        node = dq->tail;
        for (int64_t i = dq->size - 1; i > index; i--) {
            node = node->prev;
        }
    }
    
    return node->value;
}

int64_t vp_deque_len(ViperDeque* dq) {
    return dq ? dq->size : 0;
}

void vp_deque_clear(ViperDeque* dq) {
    if (!dq) return;
    
    DequeNode* current = dq->head;
    while (current) {
        DequeNode* next = current->next;
        vp_arc_release(current);
        current = next;
    }
    
    dq->head = NULL;
    dq->tail = NULL;
    dq->size = 0;
}

/* ============================================ */
/* Counter (Dict-backed frequency counter)      */
/* ============================================ */

typedef struct ViperCounter {
    ViperDict* dict;
    int64_t total;
} ViperCounter;

ViperCounter* vp_counter_create(void) {
    ViperCounter* counter = (ViperCounter*)vp_arc_alloc(sizeof(ViperCounter));
    if (counter) {
        counter->dict = vp_dict_create();
        counter->total = 0;
    }
    return counter;
}

void vp_counter_free(ViperCounter* counter) {
    if (!counter) return;
    
    if (counter->dict) {
        vp_dict_free(counter->dict);
    }
    vp_arc_release(counter);
}

void vp_counter_add(ViperCounter* counter, const char* key, int64_t count) {
    if (!counter || !key) return;
    
    /* Simplified: just track total */
    counter->total += count;
}

int64_t vp_counter_get(ViperCounter* counter, const char* key) {
    if (!counter || !key) return 0;
    
    /* Simplified implementation */
    return 0;
}

void vp_counter_set(ViperCounter* counter, const char* key, int64_t count) {
    if (!counter || !key) return;
    
    /* Simplified implementation */
    (void)count;
}

int64_t vp_counter_total(ViperCounter* counter) {
    return counter ? counter->total : 0;
}

ViperList* vp_counter_most_common(ViperCounter* counter, int64_t n) {
    ViperList* result = vp_list_create();
    
    if (!counter || n <= 0) {
        return result;
    }
    
    /* Simplified: return empty list */
    return result;
}

/* ============================================ */
/* OrderedDict (Dict with insertion order)      */
/* ============================================ */

typedef struct OrderNode {
    char* key;
    struct OrderNode* next;
    struct OrderNode* prev;
} OrderNode;

typedef struct ViperOrderedDict {
    ViperDict* dict;
    OrderNode* head;
    OrderNode* tail;
    int64_t size;
} ViperOrderedDict;

ViperOrderedDict* vp_ordered_dict_create(void) {
    ViperOrderedDict* od = (ViperOrderedDict*)vp_arc_alloc(sizeof(ViperOrderedDict));
    if (od) {
        od->dict = vp_dict_create();
        od->head = NULL;
        od->tail = NULL;
        od->size = 0;
    }
    return od;
}

void vp_ordered_dict_free(ViperOrderedDict* od) {
    if (!od) return;
    
    if (od->dict) {
        vp_dict_free(od->dict);
    }
    
    OrderNode* current = od->head;
    while (current) {
        OrderNode* next = current->next;
        if (current->key) {
            vp_arc_release(current->key);
        }
        vp_arc_release(current);
        current = next;
    }
    
    vp_arc_release(od);
}

void vp_ordered_dict_set(ViperOrderedDict* od, const char* key, int64_t value) {
    if (!od || !key) return;
    
    /* Check if key exists */
    if (!vp_dict_contains(od->dict, key)) {
        /* Add to order list */
        OrderNode* node = (OrderNode*)vp_arc_alloc(sizeof(OrderNode));
        if (node) {
            node->key = json_strdup(key, strlen(key));
            node->next = NULL;
            node->prev = od->tail;
            
            if (od->tail) {
                od->tail->next = node;
            } else {
                od->head = node;
            }
            
            od->tail = node;
            od->size++;
        }
    }
    
    /* Set in dict */
    vp_dict_set_str_i64(od->dict, (void*)key, value);
}

int64_t vp_ordered_dict_get(ViperOrderedDict* od, const char* key) {
    if (!od || !key) return 0;
    
    /* Simplified: just return 0 */
    (void)key;
    return 0;
}

int64_t vp_ordered_dict_len(ViperOrderedDict* od) {
    return od ? od->size : 0;
}

void vp_ordered_dict_clear(ViperOrderedDict* od) {
    if (!od) return;
    
    if (od->dict) {
        vp_dict_free(od->dict);
        od->dict = vp_dict_create();
    }
    
    OrderNode* current = od->head;
    while (current) {
        OrderNode* next = current->next;
        if (current->key) {
            vp_arc_release(current->key);
        }
        vp_arc_release(current);
        current = next;
    }
    
    od->head = NULL;
    od->tail = NULL;
    od->size = 0;
}

ViperList* vp_ordered_dict_keys(ViperOrderedDict* od) {
    ViperList* result = vp_list_create();
    
    if (!od) return result;
    
    /* Simplified: return empty list */
    return result;
}

ViperList* vp_ordered_dict_values(ViperOrderedDict* od) {
    ViperList* result = vp_list_create();
    
    if (!od) return result;
    
    /* Simplified: return empty list */
    return result;
}

/* ============================================ */
/* Defaultdict (Dict with default factory)      */
/* ============================================ */

typedef struct ViperDefaultDict {
    ViperDict* dict;
    int64_t default_value;
} ViperDefaultDict;

ViperDefaultDict* vp_default_dict_create(int64_t default_value) {
    ViperDefaultDict* dd = (ViperDefaultDict*)vp_arc_alloc(sizeof(ViperDefaultDict));
    if (dd) {
        dd->dict = vp_dict_create();
        dd->default_value = default_value;
    }
    return dd;
}

void vp_default_dict_free(ViperDefaultDict* dd) {
    if (!dd) return;
    
    if (dd->dict) {
        vp_dict_free(dd->dict);
    }
    vp_arc_release(dd);
}

int64_t vp_default_dict_get(ViperDefaultDict* dd, const char* key) {
    if (!dd || !key) return 0;
    
    /* Return default if key doesn't exist */
    return dd->default_value;
}

void vp_default_dict_set(ViperDefaultDict* dd, const char* key, int64_t value) {
    if (!dd || !key) return;
    
    vp_dict_set_str_i64(dd->dict, (void*)key, value);
}

/* ============================================ */
/* Tuple (fixed-size heterogeneous collection)  */
/* ============================================ */

/**
 * vp_tuple_create - Create a new tuple with given size
 * @size: Number of elements in the tuple
 * 
 * Returns: Pointer to newly created ViperTuple, or NULL on failure
 * 
 * Memory layout:
 *   - ViperTuple header (32 bytes): ref_count, size, elements pointer
 *   - elements array (size * 8 bytes): tagged i64 values
 */
ViperTuple* vp_tuple_create(int64_t size) {
    if (size < 0) return NULL;
    
    // Allocate tuple header
    ViperTuple* tuple = (ViperTuple*)vp_arc_alloc(sizeof(ViperTuple));
    if (!tuple) return NULL;
    
    tuple->ref_count = 1;
    tuple->size = size;
    
    // Allocate elements array if size > 0
    if (size > 0) {
        tuple->elements = (int64_t*)vp_arc_alloc(size * sizeof(int64_t));
        if (!tuple->elements) {
            vp_arc_release(tuple);
            return NULL;
        }
        // Initialize elements to 0
        for (int64_t i = 0; i < size; i++) {
            tuple->elements[i] = 0;
        }
    } else {
        tuple->elements = NULL;
    }
    
    return tuple;
}

/**
 * vp_tuple_free - Free a tuple and its elements
 * @tuple: Tuple to free
 */
void vp_tuple_free(ViperTuple* tuple) {
    if (!tuple) return;
    
    // Free elements array if present
    if (tuple->elements) {
        vp_arc_release(tuple->elements);
        tuple->elements = NULL;
    }
    
    // Free the tuple header
    vp_arc_release(tuple);
}

/**
 * vp_tuple_get - Get element at index from tuple
 * @tuple: Tuple to access
 * @index: Index of element (supports negative indices)
 * 
 * Returns: Element value (tagged i64), or 0 on error
 */
int64_t vp_tuple_get(ViperTuple* tuple, int64_t index) {
    if (!tuple) return 0;
    
    // Handle negative indices
    if (index < 0) {
        index = tuple->size + index;
    }
    
    if (index < 0 || index >= tuple->size) {
        return 0;
    }
    return tuple->elements[index];
}

/**
 * vp_tuple_set - Set element at index in tuple
 * @tuple: Tuple to modify
 * @index: Index of element
 * @value: Value to set (tagged i64)
 */
void vp_tuple_set(ViperTuple* tuple, int64_t index, int64_t value) {
    if (!tuple || index < 0 || index >= tuple->size) {
        return;
    }
    tuple->elements[index] = value;
}

/**
 * vp_tuple_len - Get length of tuple
 * @tuple: Tuple to query
 * 
 * Returns: Number of elements in tuple
 */
int64_t vp_tuple_len(ViperTuple* tuple) {
    return tuple ? tuple->size : 0;
}

/**
 * vp_tuple_contains - Check if value is in tuple
 * @tuple: Tuple to search
 * @value: Value to find (tagged i64)
 *
 * Returns: 1 if found, 0 otherwise
 */
int64_t vp_tuple_contains(ViperTuple* tuple, int64_t value) {
    if (!tuple) return 0;
    
    for (int64_t i = 0; i < tuple->size; i++) {
        if (tuple->elements[i] == value) {
            return 1;
        }
    }
    return 0;
}

/**
 * vp_tuple_hash - Compute hash of tuple
 * @tuple: Tuple to hash
 * 
 * Returns: Hash value (FNV-1a variant)
 * 
 * Note: Only works for tuples with hashable elements
 * (integers, floats, bools, strings, bytes, and other hashable tuples)
 */
int64_t vp_tuple_hash(ViperTuple* tuple) {
    if (!tuple) return 0;
    
    // FNV-1a hash parameters
    const uint64_t FNV_OFFSET = 14695981039346656037ULL;
    const uint64_t FNV_PRIME = 1099511628211ULL;
    
    uint64_t hash = FNV_OFFSET;
    
    // Include size in hash
    for (size_t i = 0; i < sizeof(int64_t); i++) {
        hash ^= (tuple->size >> (i * 8)) & 0xFF;
        hash *= FNV_PRIME;
    }
    
    // Include elements in hash
    for (int64_t i = 0; i < tuple->size; i++) {
        int64_t elem = tuple->elements[i];
        for (size_t j = 0; j < sizeof(int64_t); j++) {
            hash ^= (elem >> (j * 8)) & 0xFF;
            hash *= FNV_PRIME;
        }
    }
    
    return (int64_t)hash;
}

/**
 * vp_tuple_slice - Create a slice of a tuple
 * @tuple: Source tuple
 * @start: Start index (supports negative)
 * @end: End index (supports negative, exclusive)
 * 
 * Returns: New tuple containing sliced elements, or NULL on error
 */
ViperTuple* vp_tuple_slice(ViperTuple* tuple, int64_t start, int64_t end) {
    if (!tuple) return NULL;
    
    int64_t size = tuple->size;
    
    // Handle negative indices
    if (start < 0) start = size + start;
    if (end < 0) end = size + end;
    
    // Clamp to valid range
    if (start < 0) start = 0;
    if (end > size) end = size;
    if (start > end) start = end;
    
    int64_t new_size = end - start;
    ViperTuple* result = vp_tuple_create(new_size);
    if (!result) return NULL;
    
    // Copy elements
    for (int64_t i = 0; i < new_size; i++) {
        result->elements[i] = tuple->elements[start + i];
    }
    
    return result;
}

/**
 * vp_tuple_concat - Concatenate two tuples
 * @a: First tuple
 * @b: Second tuple
 * 
 * Returns: New tuple containing all elements from both tuples
 */
ViperTuple* vp_tuple_concat(ViperTuple* a, ViperTuple* b) {
    if (!a && !b) return vp_tuple_create(0);
    if (!a) return a;  // Return b, but caller needs to handle ref counting
    if (!b) return b;
    
    int64_t new_size = a->size + b->size;
    ViperTuple* result = vp_tuple_create(new_size);
    if (!result) return NULL;
    
    // Copy elements from first tuple
    for (int64_t i = 0; i < a->size; i++) {
        result->elements[i] = a->elements[i];
    }
    
    // Copy elements from second tuple
    for (int64_t i = 0; i < b->size; i++) {
        result->elements[a->size + i] = b->elements[i];
    }
    
    return result;
}

/**
 * vp_tuple_count - Count occurrences of value in tuple
 * @tuple: Tuple to search
 * @value: Value to count
 * 
 * Returns: Number of times value appears in tuple
 */
int64_t vp_tuple_count(ViperTuple* tuple, int64_t value) {
    if (!tuple) return 0;
    
    int64_t count = 0;
    for (int64_t i = 0; i < tuple->size; i++) {
        if (tuple->elements[i] == value) {
            count++;
        }
    }
    return count;
}

/**
 * vp_tuple_index - Find first index of value in tuple
 * @tuple: Tuple to search
 * @value: Value to find
 * 
 * Returns: Index of first occurrence, or -1 if not found
 */
int64_t vp_tuple_index(ViperTuple* tuple, int64_t value) {
    if (!tuple) return -1;
    
    for (int64_t i = 0; i < tuple->size; i++) {
        if (tuple->elements[i] == value) {
            return i;
        }
    }
    return -1;
}

/* ============================================ */
/* Collection Built-in Functions                */
/* ============================================ */

/**
 * list() builtin - create list from iterable
 * For now, handles: list() -> empty list, list(list) -> copy
 */
ViperList* vp_list_from_iterable(void* iterable) {
    if (!iterable) {
        return vp_list_create();
    }
    /* Simplified: just return a copy for now */
    /* Full implementation would handle various iterable types */
    return vp_list_create();
}

/**
 * list() from string - create list of character codes
 */
ViperList* vp_list_from_str(ViperString* str) {
    if (!str) {
        return vp_list_create();
    }
    
    ViperList* list = vp_list_create_with_capacity(str->length);
    for (int64_t i = 0; i < str->length; i++) {
        vp_list_append(list, (int64_t)str->data[i]);
    }
    return list;
}

/**
 * list() copy - create shallow copy of list
 */
ViperList* vp_list_copy_from_list(ViperList* src) {
    if (!src) {
        return vp_list_create();
    }
    
    ViperList* copy = vp_list_create_with_capacity(src->length);
    for (int64_t i = 0; i < src->length; i++) {
        vp_list_append(copy, vp_list_get(src, i));
    }
    return copy;
}

/**
 * tuple() builtin - create tuple from iterable
 * Currently handles: tuple() -> empty tuple, tuple(list) -> convert list to tuple
 */
ViperTuple* vp_tuple_from_iterable(void* iterable) {
    if (!iterable) {
        return vp_tuple_create(0);
    }
    /* Simplified: just return empty tuple for now */
    /* Full implementation would handle various iterable types */
    return vp_tuple_create(0);
}

/**
 * tuple() from list - convert list to tuple
 */
ViperTuple* vp_tuple_from_list(ViperList* src) {
    if (!src) {
        return vp_tuple_create(0);
    }

    ViperTuple* tuple = vp_tuple_create(src->length);
    if (!tuple) return NULL;
    
    for (int64_t i = 0; i < src->length; i++) {
        tuple->elements[i] = vp_list_get(src, i);
    }
    return tuple;
}

/**
 * tuple() from string - create tuple of character codes
 */
ViperTuple* vp_tuple_from_str(ViperString* str) {
    if (!str) {
        return vp_tuple_create(0);
    }
    
    ViperTuple* tuple = vp_tuple_create(str->length);
    if (!tuple) return NULL;
    
    for (int64_t i = 0; i < str->length; i++) {
        tuple->elements[i] = (int64_t)str->data[i];
    }
    return tuple;
}

/**
 * tuple() copy - create shallow copy of tuple
 */
ViperTuple* vp_tuple_copy(ViperTuple* src) {
    if (!src) {
        return vp_tuple_create(0);
    }
    
    ViperTuple* copy = vp_tuple_create(src->size);
    if (!copy) return NULL;
    
    for (int64_t i = 0; i < src->size; i++) {
        copy->elements[i] = src->elements[i];
    }
    return copy;
}

/**
 * set() builtin - create set from iterable
 * For now, returns a list (set implementation is simplified)
 */
ViperList* vp_set_from_iterable(void* iterable) {
    if (!iterable) {
        return vp_list_create();
    }
    /* Simplified: just return a list for now */
    return vp_list_create();
}

/**
 * set() from list - convert list to set (removes duplicates)
 */
ViperList* vp_set_from_list(ViperList* src) {
    if (!src) {
        return vp_list_create();
    }
    
    /* Simple implementation: just copy (no deduplication yet) */
    /* Full implementation would use hash table for O(1) lookup */
    ViperList* set = vp_list_create_with_capacity(src->length);
    for (int64_t i = 0; i < src->length; i++) {
        int64_t val = vp_list_get(src, i);
        /* Check if already in set */
        int found = 0;
        for (int64_t j = 0; j < set->length; j++) {
            if (vp_list_get(set, j) == val) {
                found = 1;
                break;
            }
        }
        if (!found) {
            vp_list_append(set, val);
        }
    }
    return set;
}

/**
 * set() copy - create shallow copy of set
 */
ViperList* vp_set_copy(ViperList* src) {
    return vp_set_from_list(src);
}

/**
 * set() add element
 */
void vp_set_add(ViperList* set, int64_t value) {
    if (!set) return;
    
    /* Check if already in set */
    for (int64_t i = 0; i < set->length; i++) {
        if (vp_list_get(set, i) == value) {
            return; /* Already exists */
        }
    }
    vp_list_append(set, value);
}

/**
 * set() contains
 */
int vp_set_contains(ViperList* set, int64_t value) {
    if (!set) return 0;
    
    for (int64_t i = 0; i < set->length; i++) {
        if (vp_list_get(set, i) == value) {
            return 1;
        }
    }
    return 0;
}

/**
 * set() len
 */
int64_t vp_set_len(ViperList* set) {
    return set ? set->length : 0;
}

/**
 * set() print
 */
void vp_set_print(ViperList* set) {
    if (!set) {
        printf("set()\n");
        return;
    }
    
    printf("{");
    for (int64_t i = 0; i < set->length; i++) {
        if (i > 0) printf(", ");
        printf("%ld", (long)vp_list_get(set, i));
    }
    printf("}\n");
}

/* ============================================ */
/* Iteration Builtins                           */
/* ============================================ */

/**
 * enumerate(iterable, start=0) - returns list of (index, value) tuples
 * Simplified: returns list of indices for now
 */
ViperList* vp_enumerate(ViperList* iterable, int64_t start) {
    if (!iterable) {
        return vp_list_create();
    }
    
    /* For now, just return a list of indices - proper tuple support needed */
    ViperList* result = vp_list_create_with_capacity(iterable->length);
    for (int64_t i = 0; i < iterable->length; i++) {
        vp_list_append(result, start + i);
    }
    return result;
}

/**
 * zip(iter1, iter2) - returns list of paired elements
 * Simplified: returns first list for now
 */
ViperList* vp_zip(ViperList* iter1, ViperList* iter2) {
    if (!iter1 || !iter2) {
        return vp_list_create();
    }
    
    /* For now, return iter1 copy - proper tuple support needed */
    return vp_list_copy_from_list(iter1);
}

/* ============================================ */
/* Functional Builtins                          */
/* ============================================ */

/**
 * sum(iterable) - sum of all elements
 */
int64_t vp_list_sum(ViperList* list) {
    if (!list) return 0;
    
    int64_t total = 0;
    for (int64_t i = 0; i < list->length; i++) {
        total += vp_list_get(list, i);
    }
    return total;
}

/**
 * min(iterable) - minimum element
 */
int64_t vp_list_min(ViperList* list) {
    if (!list || list->length == 0) return 0;
    
    int64_t min_val = vp_list_get(list, 0);
    for (int64_t i = 1; i < list->length; i++) {
        int64_t val = vp_list_get(list, i);
        if (val < min_val) {
            min_val = val;
        }
    }
    return min_val;
}

/**
 * max(iterable) - maximum element
 */
int64_t vp_list_max(ViperList* list) {
    if (!list || list->length == 0) return 0;
    
    int64_t max_val = vp_list_get(list, 0);
    for (int64_t i = 1; i < list->length; i++) {
        int64_t val = vp_list_get(list, i);
        if (val > max_val) {
            max_val = val;
        }
    }
    return max_val;
}

/**
 * any(iterable) - true if any element is truthy
 */
int vp_list_any(ViperList* list) {
    if (!list) return 0;
    
    for (int64_t i = 0; i < list->length; i++) {
        if (vp_list_get(list, i) != 0) {
            return 1;
        }
    }
    return 0;
}

/**
 * all(iterable) - true if all elements are truthy
 */
int vp_list_all(ViperList* list) {
    if (!list) return 1;
    
    for (int64_t i = 0; i < list->length; i++) {
        if (vp_list_get(list, i) == 0) {
            return 0;
        }
    }
    return 1;
}

/* ============================================ */
/* Introspection Builtins                       */
/* ============================================ */

/**
 * type_of(obj) - returns type name as string
 * Simplified: returns "object" for all types
 */
ViperString* vp_type_of(void* obj) {
    (void)obj;  /* Unused for now */
    return vp_str_create("object");
}

/**
 * object_id(obj) - returns pointer as integer
 */
int64_t vp_object_id(void* obj) {
    return (int64_t)obj;
}

/**
 * repr() functions - string representations
 */
ViperString* vp_repr_i64(int64_t val) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%ld", (long)val);
    return vp_str_create(buffer);
}

ViperString* vp_repr_f64(double val) {
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%g", val);
    return vp_str_create(buffer);
}

ViperString* vp_repr_str(ViperString* str) {
    if (!str) return vp_str_create("None");
    /* Add quotes */
    char* quoted = malloc(str->length + 3);
    quoted[0] = '\'';
    memcpy(quoted + 1, str->data, str->length);
    quoted[str->length + 1] = '\'';
    quoted[str->length + 2] = '\0';
    ViperString* result = vp_str_create(quoted);
    free(quoted);
    return result;
}

ViperString* vp_repr_bool(int val) {
    return vp_str_create(val ? "True" : "False");
}

/* ============================================ */
/* Conversion Builtins                          */
/* ============================================ */

/**
 * bin(n) - binary representation
 */
ViperString* vp_bin_i64(int64_t n) {
    char buffer[65];
    char* result = buffer + 64;
    *result = '\0';
    
    int64_t abs_n = n < 0 ? -n : n;
    do {
        *--result = '0' + (abs_n % 2);
        abs_n /= 2;
    } while (abs_n > 0);
    
    if (n < 0) *--result = '-';
    *--result = 'b';
    *--result = '0';
    
    return vp_str_create(result);
}

/**
 * oct(n) - octal representation
 */
ViperString* vp_oct_i64(int64_t n) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "0%lo", (unsigned long)n);
    return vp_str_create(buffer);
}

/**
 * hex(n) - hexadecimal representation
 */
ViperString* vp_hex_i64(int64_t n) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "0x%lx", (unsigned long)n);
    return vp_str_create(buffer);
}

/**
 * chr(n) - character from Unicode code point
 */
ViperString* vp_chr_i64(int64_t n) {
    char buffer[8];
    if (n >= 0 && n <= 127) {
        buffer[0] = (char)n;
        buffer[1] = '\0';
    } else {
        buffer[0] = '?';
        buffer[1] = '\0';
    }
    return vp_str_create(buffer);
}

/**
 * ord(s) - Unicode code point of first character
 */
int64_t vp_ord_str(ViperString* str) {
    if (!str || str->length == 0) return 0;
    return (int64_t)(unsigned char)str->data[0];
}

/* ============================================ */
/* Numeric Builtins                             */
/* ============================================ */

/**
 * round(n, ndigits) - round to ndigits decimal places
 */
double vp_round_f64(double n, int64_t ndigits) {
    double multiplier = 1.0;
    for (int64_t i = 0; i < ndigits; i++) {
        multiplier *= 10.0;
    }
    double temp = n * multiplier;
    temp = (temp > 0) ? (int64_t)(temp + 0.5) : (int64_t)(temp - 0.5);
    return temp / multiplier;
}

/**
 * divmod(a, b) - returns tuple (quotient, remainder)
 * Simplified: returns quotient only for now
 */
ViperList* vp_divmod_i64(int64_t a, int64_t b) {
    ViperList* result = vp_list_create_with_capacity(2);
    if (b == 0) {
        vp_list_append(result, 0);
        vp_list_append(result, 0);
    } else {
        vp_list_append(result, a / b);
        vp_list_append(result, a % b);
    }
    return result;
}

/**
 * pow(base, exp) - power function
 */
double vp_pow_f64(double base, double exp) {
    return pow(base, exp);
}

/* ============================================ */
/* Attribute Builtins                           */
/* ============================================ */

/**
 * hasattr(obj, name) - check if object has attribute
 * Simplified: always returns 0 (false) for now
 */
int vp_hasattr(void* obj, ViperString* name) {
    (void)obj;
    (void)name;
    return 0;  /* Not implemented */
}

/* ============================================ */
/* Callable Builtin                             */
/* ============================================ */

/**
 * is_callable(obj) - check if object is callable
 * Simplified: always returns 0 (false) for now
 */
int vp_is_callable(void* obj) {
    (void)obj;
    return 0;  /* Not implemented */
}

/* ============================================ */
/* I/O Builtins                                 */
/* ============================================ */

/**
 * input(prompt) - read line from stdin
 */
ViperString* vp_input(ViperString* prompt) {
    if (prompt && prompt->length > 0) {
        printf("%s", prompt->data);
        fflush(stdout);
    }
    
    char buffer[1024];
    if (fgets(buffer, sizeof(buffer), stdin) == NULL) {
        return vp_str_create("");
    }
    
    /* Remove trailing newline */
    size_t len = strlen(buffer);
    if (len > 0 && buffer[len-1] == '\n') {
        buffer[len-1] = '\0';
    }
    
    return vp_str_create(buffer);
}

/* ============================================ */
/* Dict Builtin                                 */
/* ============================================ */

/**
 * dict() - create empty dict
 */
ViperDict* vp_dict_create_empty(void) {
    return vp_dict_create();
}


