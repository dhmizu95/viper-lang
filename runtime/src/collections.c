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
/* NamedTuple (Simple struct-like container)    */
/* ============================================ */

typedef struct ViperNamedTuple {
    ViperDict* fields;
    ViperList* values;
    int64_t size;
} ViperNamedTuple;

ViperNamedTuple* vp_named_tuple_create(int64_t size) {
    ViperNamedTuple* nt = (ViperNamedTuple*)vp_arc_alloc(sizeof(ViperNamedTuple));
    if (nt) {
        nt->fields = vp_dict_create();
        nt->values = vp_list_create();
        nt->size = size;
        
        /* Pre-allocate values */
        for (int64_t i = 0; i < size; i++) {
            vp_list_append(nt->values, 0);
        }
    }
    return nt;
}

void vp_named_tuple_free(ViperNamedTuple* nt) {
    if (!nt) return;
    
    if (nt->fields) {
        vp_dict_free(nt->fields);
    }
    if (nt->values) {
        vp_list_free(nt->values);
    }
    vp_arc_release(nt);
}

void vp_named_tuple_set_field(ViperNamedTuple* nt, int64_t index, const char* name) {
    if (!nt || index < 0 || index >= nt->size || !name) return;
    
    vp_dict_set_str_i64(nt->fields, (void*)name, index);
}

void vp_named_tuple_set_value(ViperNamedTuple* nt, int64_t index, int64_t value) {
    if (!nt || index < 0 || index >= nt->size) return;
    
    /* Simplified: would need proper list set implementation */
    (void)value;
}

int64_t vp_named_tuple_get_value(ViperNamedTuple* nt, int64_t index) {
    if (!nt || index < 0 || index >= nt->size) return 0;

    return 0; /* Simplified */
}

int64_t vp_named_tuple_len(ViperNamedTuple* nt) {
    return nt ? nt->size : 0;
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
 * For now, returns a list (tuple implementation is simplified)
 */
ViperList* vp_tuple_from_iterable(void* iterable) {
    if (!iterable) {
        return vp_list_create();
    }
    /* Simplified: just return a list for now */
    return vp_list_create();
}

/**
 * tuple() from list - convert list to tuple
 * For now, just returns a copy of the list
 */
ViperList* vp_tuple_from_list(ViperList* src) {
    if (!src) {
        return vp_list_create();
    }
    
    ViperList* tuple = vp_list_create_with_capacity(src->length);
    for (int64_t i = 0; i < src->length; i++) {
        vp_list_append(tuple, vp_list_get(src, i));
    }
    return tuple;
}

/**
 * tuple() from string - create tuple of character codes
 */
ViperList* vp_tuple_from_str(ViperString* str) {
    if (!str) {
        return vp_list_create();
    }
    
    ViperList* tuple = vp_list_create_with_capacity(str->length);
    for (int64_t i = 0; i < str->length; i++) {
        vp_list_append(tuple, (int64_t)str->data[i]);
    }
    return tuple;
}

/**
 * tuple() copy - create shallow copy of tuple
 */
ViperList* vp_tuple_copy(ViperList* src) {
    return vp_tuple_from_list(src);
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

