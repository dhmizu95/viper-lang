/**
 * Viper Dictionary (Hash Map) Implementation
 * A hash map with reference counting
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_stdlib.h"

#define DICT_INITIAL_BUCKETS 16
#define DICT_LOAD_FACTOR 0.75

/* ============================================ */
/* Hash Function                                */
/* ============================================ */

static uint64_t vp_dict_hash(const char* key) {
    if (!key) return 0;

    /* FNV-1a hash function */
    uint64_t hash = 14695981039346656037ULL;
    while (*key) {
        hash ^= (uint64_t)(*key);
        hash *= 1099511628211ULL;
        key++;
    }
    return hash;
}

/* ============================================ */
/* DictEntry Internal Functions                 */
/* ============================================ */

static DictEntry* vp_dict_entry_create(const char* key, ViperValue value) {
    DictEntry* entry = (DictEntry*)malloc(sizeof(DictEntry));
    if (!entry) {
        vp_panic("Failed to allocate dict entry");
        return NULL;
    }

    entry->key = vp_str_create(key);
    entry->value = value;
    entry->next = NULL;

    return entry;
}

static void vp_dict_entry_free(DictEntry* entry) {
    if (!entry) return;

    if (entry->key) {
        vp_str_free(entry->key);
    }

    /* Free value if it's a reference type */
    if (entry->value.type == VIPER_TYPE_STR) {
        vp_str_free(entry->value.data.as_str);
    } else if (entry->value.type == VIPER_TYPE_LIST) {
        vp_list_free(entry->value.data.as_list);
    } else if (entry->value.type == VIPER_TYPE_DICT) {
        vp_dict_free(entry->value.data.as_dict);
    }

    free(entry);
}

/* ============================================ */
/* Dict Internal Functions                      */
/* ============================================ */

static void vp_dict_resize(ViperDict* dict, int64_t new_size) {
    if (!dict || new_size <= 0) return;

    DictEntry** new_buckets = (DictEntry**)calloc(new_size, sizeof(DictEntry*));
    if (!new_buckets) {
        vp_panic("Failed to resize dictionary");
        return;
    }

    /* Rehash all entries */
    for (int64_t i = 0; i < dict->size; i++) {
        DictEntry* entry = dict->buckets[i];
        while (entry) {
            DictEntry* next = entry->next;

            uint64_t new_hash = vp_dict_hash(entry->key);
            int64_t new_index = new_hash % new_size;

            entry->next = new_buckets[new_index];
            new_buckets[new_index] = entry;

            entry = next;
        }
    }

    /* Free old buckets array (not entries, they've been moved) */
    free(dict->buckets);

    dict->buckets = new_buckets;
    dict->size = new_size;
}

static void vp_dict_destroy(void* ptr) {
    ViperDict* dict = (ViperDict*)ptr;
    if (!dict) return;

    /* Free all entries */
    for (int64_t i = 0; i < dict->size; i++) {
        DictEntry* entry = dict->buckets[i];
        while (entry) {
            DictEntry* next = entry->next;
            vp_dict_entry_free(entry);
            entry = next;
        }
    }

    /* Free buckets array */
    if (dict->buckets) {
        free(dict->buckets);
        dict->buckets = NULL;
    }
}

/* ============================================ */
/* Dict Public Functions                        */
/* ============================================ */

ViperDict* vp_dict_create(void) {
    ViperDict* dict = (ViperDict*)vp_arc_alloc(sizeof(ViperDict));

    dict->ref_count = 1;
    dict->size = DICT_INITIAL_BUCKETS;
    dict->count = 0;
    dict->buckets = (DictEntry**)calloc(dict->size, sizeof(DictEntry*));

    if (!dict->buckets) {
        vp_panic("Failed to allocate dict buckets");
    }

    vp_arc_set_destructor(dict, vp_dict_destroy);

    return dict;
}

void vp_dict_free(ViperDict* dict) {
    if (!dict) return;
    vp_arc_release(dict);
}

void vp_dict_set(ViperDict* dict, const char* key, ViperValue value) {
    if (!dict || !key) {
        vp_panic("Cannot set on NULL dict or with NULL key");
        return;
    }

    /* Check load factor and resize if needed */
    double load_factor = (double)(dict->count + 1) / dict->size;
    if (load_factor > DICT_LOAD_FACTOR) {
        vp_dict_resize(dict, dict->size * 2);
    }

    uint64_t hash = vp_dict_hash(key);
    int64_t index = hash % dict->size;

    /* Check if key already exists */
    DictEntry* entry = dict->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            /* Update existing value */
            /* Free old value if reference type */
            if (entry->value.type == VIPER_TYPE_STR) {
                vp_str_free(entry->value.data.as_str);
            } else if (entry->value.type == VIPER_TYPE_LIST) {
                vp_list_free(entry->value.data.as_list);
            } else if (entry->value.type == VIPER_TYPE_DICT) {
                vp_dict_free(entry->value.data.as_dict);
            }

            entry->value = value;
            return;
        }
        entry = entry->next;
    }

    /* Insert new entry */
    DictEntry* new_entry = vp_dict_entry_create(key, value);
    new_entry->next = dict->buckets[index];
    dict->buckets[index] = new_entry;
    dict->count++;
}

ViperValue vp_dict_get(ViperDict* dict, const char* key) {
    if (!dict || !key) {
        ViperValue null_val = {0};
        null_val.type = VIPER_TYPE_NONE;
        return null_val;
    }

    uint64_t hash = vp_dict_hash(key);
    int64_t index = hash % dict->size;

    DictEntry* entry = dict->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            return entry->value;
        }
        entry = entry->next;
    }

    /* Key not found */
    ViperValue null_val = {0};
    null_val.type = VIPER_TYPE_NONE;
    return null_val;
}

bool vp_dict_contains(ViperDict* dict, const char* key) {
    if (!dict || !key) return false;

    uint64_t hash = vp_dict_hash(key);
    int64_t index = hash % dict->size;

    DictEntry* entry = dict->buckets[index];
    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            return true;
        }
        entry = entry->next;
    }

    return false;
}

bool vp_dict_remove(ViperDict* dict, const char* key) {
    if (!dict || !key) return false;

    uint64_t hash = vp_dict_hash(key);
    int64_t index = hash % dict->size;

    DictEntry* entry = dict->buckets[index];
    DictEntry* prev = NULL;

    while (entry) {
        if (strcmp(entry->key, key) == 0) {
            if (prev) {
                prev->next = entry->next;
            } else {
                dict->buckets[index] = entry->next;
            }

            vp_dict_entry_free(entry);
            dict->count--;
            return true;
        }
        prev = entry;
        entry = entry->next;
    }

    return false;
}

void vp_dict_clear(ViperDict* dict) {
    if (!dict) return;

    for (int64_t i = 0; i < dict->size; i++) {
        DictEntry* entry = dict->buckets[i];
        while (entry) {
            DictEntry* next = entry->next;
            vp_dict_entry_free(entry);
            entry = next;
        }
        dict->buckets[i] = NULL;
    }

    dict->count = 0;
}

int64_t vp_dict_len(ViperDict* dict) {
    if (!dict) return 0;
    return dict->count;
}

ViperDict* vp_dict_copy(ViperDict* dict) {
    if (!dict) return NULL;

    ViperDict* copy = vp_dict_create();

    for (int64_t i = 0; i < dict->size; i++) {
        DictEntry* entry = dict->buckets[i];
        while (entry) {
            vp_dict_set(copy, entry->key, entry->value);
            entry = entry->next;
        }
    }

    return copy;
}

/* ============================================ */
/* Dict Iterator Functions                      */
/* ============================================ */

ViperDictIter* vp_dict_iter_create(ViperDict* dict) {
    if (!dict) return NULL;

    ViperDictIter* iter = (ViperDictIter*)malloc(sizeof(ViperDictIter));
    if (!iter) return NULL;

    iter->dict = dict;
    iter->bucket_index = 0;
    iter->current = NULL;

    /* Find first entry */
    while (iter->bucket_index < dict->size) {
        if (dict->buckets[iter->bucket_index]) {
            iter->current = dict->buckets[iter->bucket_index];
            return iter;
        }
        iter->bucket_index++;
    }

    return iter;  /* Empty dict */
}

void vp_dict_iter_free(ViperDictIter* iter) {
    if (iter) free(iter);
}

bool vp_dict_iter_next(ViperDictIter* iter, const char** key, ViperValue* value) {
    if (!iter || !iter->dict || !key || !value) return false;

    if (!iter->current) return false;

    *key = iter->current->key;
    *value = iter->current->value;

    /* Move to next entry */
    if (iter->current->next) {
        iter->current = iter->current->next;
    } else {
        /* Move to next bucket */
        iter->bucket_index++;
        iter->current = NULL;

        while (iter->bucket_index < iter->dict->size) {
            if (iter->dict->buckets[iter->bucket_index]) {
                iter->current = iter->dict->buckets[iter->bucket_index];
                break;
            }
            iter->bucket_index++;
        }
    }

    return true;
}
