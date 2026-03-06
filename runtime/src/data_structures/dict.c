/**
 * Viper Dictionary (Hash Map) Implementation
 * Open addressing with linear probing
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "viper_stdlib.h"

#define DICT_INITIAL_CAPACITY 16
#define DICT_LOAD_FACTOR 0.75

/* Control bytes */
#define CTRL_EMPTY   0x00
#define CTRL_DELETED 0x01
#define CTRL_FULL    0x02

/* ============================================ */
/* Hash Function                                */
/* ============================================ */

static uint64_t vp_dict_hash(const char* key) {
    if (!key) return 0;
    uint64_t hash = 14695981039346656037ULL;
    while (*key) {
        hash ^= (uint64_t)(*key);
        hash *= 1099511628211ULL;
        key++;
    }
    return hash;
}

/* ============================================ */
/* Dict Internal Functions                      */
/* ============================================ */

static void free_value(ViperValue* val) {
    if (val->type == VIPER_TYPE_STR && val->data.as_str) {
        vp_str_free(val->data.as_str);
    } else if (val->type == VIPER_TYPE_LIST && val->data.as_list) {
        vp_list_free(val->data.as_list);
    } else if (val->type == VIPER_TYPE_DICT && val->data.as_dict) {
        vp_dict_free(val->data.as_dict);
    }
}

static void vp_dict_resize(ViperDict* dict, int64_t new_size) {
    if (!dict || new_size <= 0) return;

    DictEntry* old_entries = dict->entries;
    uint8_t* old_ctrl = dict->ctrl;
    int64_t old_size = dict->size;

    dict->entries = (DictEntry*)calloc(new_size, sizeof(DictEntry));
    dict->ctrl = (uint8_t*)calloc(new_size, sizeof(uint8_t)); // All 0 = EMPTY
    if (!dict->entries || !dict->ctrl) {
        vp_panic("Failed to resize dictionary");
        return;
    }

    dict->size = new_size;
    dict->count = 0; // Will be incremented properly below

    /* Rehash all entries */
    for (int64_t i = 0; i < old_size; i++) {
        if (old_ctrl[i] == CTRL_FULL) {
            /* We bypass vp_dict_set to avoid freeing old_entries or duplicating strings */
            uint64_t hash = vp_dict_hash(old_entries[i].key);
            int64_t idx = hash & (new_size - 1); // Assuming size is power of 2
            
            while (dict->ctrl[idx] == CTRL_FULL) {
                idx = (idx + 1) & (new_size - 1);
            }
            
            dict->entries[idx] = old_entries[i];
            dict->ctrl[idx] = CTRL_FULL;
            dict->count++;
        }
    }

    if (old_entries) free(old_entries);
    if (old_ctrl) free(old_ctrl);
}

static void vp_dict_destroy(void* ptr) {
    ViperDict* dict = (ViperDict*)ptr;
    if (!dict) return;

    if (dict->ctrl && dict->entries) {
        for (int64_t i = 0; i < dict->size; i++) {
            if (dict->ctrl[i] == CTRL_FULL) {
                if (dict->entries[i].key) {
                    vp_str_free(dict->entries[i].key);
                }
                free_value(&dict->entries[i].value);
            }
        }
    }

    if (dict->entries) {
        free(dict->entries);
        dict->entries = NULL;
    }
    if (dict->ctrl) {
        free(dict->ctrl);
        dict->ctrl = NULL;
    }
}

/* ============================================ */
/* Dict Public Functions                        */
/* ============================================ */

ViperDict* vp_dict_create(void) {
    return vp_dict_create_with_capacity(DICT_INITIAL_CAPACITY);
}

ViperDict* vp_dict_create_with_capacity(int64_t initial_cap) {
    ViperDict* dict = (ViperDict*)vp_arc_alloc(sizeof(ViperDict));

    dict->ref_count = 1;
    
    int64_t bucket_size = DICT_INITIAL_CAPACITY;
    while (bucket_size < initial_cap * 2) { // x2 to account for load factor
        bucket_size *= 2;
    }
    
    dict->size = bucket_size;
    dict->count = 0;
    dict->entries = (DictEntry*)calloc(dict->size, sizeof(DictEntry));
    dict->ctrl = (uint8_t*)calloc(dict->size, sizeof(uint8_t));

    if (!dict->entries || !dict->ctrl) {
        vp_panic("Failed to allocate dict memory");
    }

    vp_arc_set_destructor(dict, vp_dict_destroy);

    return dict;
}

void vp_dict_free(ViperDict* dict) {
    if (!dict) return;
    vp_arc_release(dict);
}

void vp_dict_set(ViperDict* dict, const char* key, ViperValue value) {
    if (!dict || !key) vp_panic("Cannot set on NULL dict or with NULL key");

    if ((double)(dict->count + 1) > dict->size * DICT_LOAD_FACTOR) {
        vp_dict_resize(dict, dict->size * 2);
    }

    uint64_t hash = vp_dict_hash(key);
    int64_t idx = hash & (dict->size - 1);
    int64_t first_deleted = -1;

    while (1) {
        if (dict->ctrl[idx] == CTRL_EMPTY) {
            break;
        } else if (dict->ctrl[idx] == CTRL_DELETED) {
            if (first_deleted == -1) first_deleted = idx;
        } else if (dict->ctrl[idx] == CTRL_FULL) {
            if (strcmp(dict->entries[idx].key, key) == 0) {
                /* Update existing value */
                free_value(&dict->entries[idx].value);
                dict->entries[idx].value = value;
                return;
            }
        }
        idx = (idx + 1) & (dict->size - 1);
    }

    /* Insert new entry */
    int64_t target_idx = (first_deleted != -1) ? first_deleted : idx;
    dict->entries[target_idx].key = vp_str_create(key);
    dict->entries[target_idx].value = value;
    dict->ctrl[target_idx] = CTRL_FULL;
    dict->count++;
}

ViperValue vp_dict_get(ViperDict* dict, const char* key) {
    ViperValue null_val = {0};
    null_val.type = VIPER_TYPE_NONE;
    if (!dict || !key) return null_val;

    uint64_t hash = vp_dict_hash(key);
    int64_t idx = hash & (dict->size - 1);

    while (dict->ctrl[idx] != CTRL_EMPTY) {
        if (dict->ctrl[idx] == CTRL_FULL && strcmp(dict->entries[idx].key, key) == 0) {
            return dict->entries[idx].value;
        }
        idx = (idx + 1) & (dict->size - 1);
    }

    return null_val;
}

bool vp_dict_contains(ViperDict* dict, const char* key) {
    if (!dict || !key) return false;

    uint64_t hash = vp_dict_hash(key);
    int64_t idx = hash & (dict->size - 1);

    while (dict->ctrl[idx] != CTRL_EMPTY) {
        if (dict->ctrl[idx] == CTRL_FULL && strcmp(dict->entries[idx].key, key) == 0) {
            return true;
        }
        idx = (idx + 1) & (dict->size - 1);
    }

    return false;
}

bool vp_dict_remove(ViperDict* dict, const char* key) {
    if (!dict || !key) return false;

    uint64_t hash = vp_dict_hash(key);
    int64_t idx = hash & (dict->size - 1);

    while (dict->ctrl[idx] != CTRL_EMPTY) {
        if (dict->ctrl[idx] == CTRL_FULL && strcmp(dict->entries[idx].key, key) == 0) {
            vp_str_free(dict->entries[idx].key);
            dict->entries[idx].key = NULL;
            free_value(&dict->entries[idx].value);
            dict->ctrl[idx] = CTRL_DELETED;
            dict->count--;
            return true;
        }
        idx = (idx + 1) & (dict->size - 1);
    }

    return false;
}

void vp_dict_clear(ViperDict* dict) {
    if (!dict) return;

    for (int64_t i = 0; i < dict->size; i++) {
        if (dict->ctrl[i] == CTRL_FULL) {
            if (dict->entries[i].key) vp_str_free(dict->entries[i].key);
            free_value(&dict->entries[i].value);
        }
        dict->ctrl[i] = CTRL_EMPTY;
    }
    dict->count = 0;
}

int64_t vp_dict_len(ViperDict* dict) {
    if (!dict) return 0;
    return dict->count;
}

ViperDict* vp_dict_copy(ViperDict* dict) {
    if (!dict) return NULL;

    ViperDict* copy = vp_dict_create_with_capacity(dict->size);

    for (int64_t i = 0; i < dict->size; i++) {
        if (dict->ctrl[i] == CTRL_FULL) {
            vp_dict_set(copy, dict->entries[i].key, dict->entries[i].value);
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
    iter->iter_index = 0;
    return iter;
}

void vp_dict_iter_free(ViperDictIter* iter) {
    if (iter) free(iter);
}

bool vp_dict_iter_next(ViperDictIter* iter, const char** key, ViperValue* value) {
    if (!iter || !iter->dict || !key || !value) return false;

    while (iter->iter_index < iter->dict->size) {
        if (iter->dict->ctrl[iter->iter_index] == CTRL_FULL) {
            *key = iter->dict->entries[iter->iter_index].key;
            *value = iter->dict->entries[iter->iter_index].value;
            iter->iter_index++;
            return true;
        }
        iter->iter_index++;
    }

    return false;
}

/* ============================================ */
/* Dict Print Function                          */
/* ============================================ */

void vp_dict_print(ViperDict* dict) {
    if (!dict || !dict->entries) {
        printf("{}");
        return;
    }

    printf("{");
    
    bool first = true;
    for (int64_t i = 0; i < dict->size; i++) {
        if (dict->ctrl[i] != CTRL_FULL) continue;
        
        if (!first) {
            printf(", ");
        }
        first = false;
        
        DictEntry* entry = &dict->entries[i];
        
        if (entry->key) {
            printf("'%s': ", entry->key);
        } else {
            printf("<null_key>: ");
        }
        
        switch (entry->value.type) {
            case VIPER_TYPE_I64:
                printf("%ld", (long)entry->value.data.as_i64);
                break;
            case VIPER_TYPE_F64:
                printf("%f", entry->value.data.as_f64);
                break;
            case VIPER_TYPE_BOOL:
                printf("%s", entry->value.data.as_bool ? "True" : "False");
                break;
            case VIPER_TYPE_STR:
                if (entry->value.data.as_str) {
                    printf("'%s'", entry->value.data.as_str);
                } else {
                    printf("<null_str>");
                }
                break;
            case VIPER_TYPE_NONE:
                printf("None");
                break;
            case VIPER_TYPE_LIST:
                printf("<list>");
                break;
            case VIPER_TYPE_DICT:
                printf("<dict>");
                break;
            default:
                printf("<unknown>");
                break;
        }
    }
    
    printf("}");
}

