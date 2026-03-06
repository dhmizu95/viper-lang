/**
 * Viper String Interning Implementation
 * 
 * Hash table-based string interning with reference counting.
 * Uses open addressing with linear probing for cache efficiency.
 */

#include "viper_string_intern.h"
#include "viper_arc.h"
#include <string.h>
#include <stdlib.h>

/* ============================================ */
/* Internal Data Structures                     */
/* ============================================ */

/**
 * Interned string entry
 * Stored in hash table with open addressing
 */
typedef struct {
    const char* str;      /* Pointer to interned string */
    size_t len;           /* String length */
    int64_t ref_count;    /* Reference count */
    uint64_t hash;        /* Cached hash value */
    bool occupied;        /* Slot is occupied */
} InternEntry;

/**
 * String intern hash table
 */
typedef struct {
    InternEntry* entries;     /* Hash table entries */
    int64_t capacity;         /* Table capacity */
    int64_t count;            /* Number of interned strings */
    int64_t memory_used;      /* Total memory usage */
} StringInternTable;

/* Global intern table */
static StringInternTable g_intern_table = {0};

/* ============================================ */
/* Hash Function                                */
/* ============================================ */

/**
 * FNV-1a hash function for strings
 * Fast and good distribution for string keys
 */
static inline uint64_t hash_string(const char* str, size_t len) {
    const uint64_t FNV_OFFSET = 14695981039346656037ULL;
    const uint64_t FNV_PRIME = 1099511628211ULL;
    
    uint64_t hash = FNV_OFFSET;
    for (size_t i = 0; i < len; i++) {
        hash ^= (uint8_t)str[i];
        hash *= FNV_PRIME;
    }
    return hash;
}

/**
 * Get hash table index with masking
 */
static inline int64_t get_index(uint64_t hash, int64_t capacity) {
    return (int64_t)(hash & (capacity - 1));
}

/* ============================================ */
/* Core Functions                               */
/* ============================================ */

void vp_str_intern_init(void) {
    if (g_intern_table.entries != NULL) {
        return;  /* Already initialized */
    }
    
    g_intern_table.capacity = VIPER_INTERN_TABLE_SIZE;
    g_intern_table.count = 0;
    g_intern_table.memory_used = 0;
    
    /* Allocate hash table (array of entries) */
    g_intern_table.entries = (InternEntry*)calloc(
        (size_t)g_intern_table.capacity, 
        sizeof(InternEntry)
    );
}

void vp_str_intern_shutdown(void) {
    if (g_intern_table.entries == NULL) {
        return;  /* Not initialized */
    }
    
    /* Free all interned strings */
    for (int64_t i = 0; i < g_intern_table.capacity; i++) {
        if (g_intern_table.entries[i].occupied && g_intern_table.entries[i].str) {
            free((void*)g_intern_table.entries[i].str);
        }
    }
    
    /* Free hash table */
    free(g_intern_table.entries);
    g_intern_table.entries = NULL;
    g_intern_table.capacity = 0;
    g_intern_table.count = 0;
    g_intern_table.memory_used = 0;
}

/**
 * Find or insert a string in the intern table
 */
static const char* intern_insert(const char* str, size_t len, uint64_t hash) {
    /* Linear probing for open addressing */
    int64_t index = get_index(hash, g_intern_table.capacity);
    int64_t start_index = index;
    
    do {
        InternEntry* entry = &g_intern_table.entries[index];
        
        if (!entry->occupied) {
            /* Empty slot - insert here */
            char* new_str = (char*)malloc(len + 1);
            if (!new_str) {
                return NULL;  /* Allocation failed */
            }
            
            memcpy(new_str, str, len);
            new_str[len] = '\0';
            
            entry->str = new_str;
            entry->len = len;
            entry->ref_count = 1;
            entry->hash = hash;
            entry->occupied = true;
            
            g_intern_table.count++;
            g_intern_table.memory_used += (int64_t)(len + 1);
            
            return new_str;
        }
        
        /* Check if this is a match */
        if (entry->hash == hash && entry->len == len) {
            if (memcmp(entry->str, str, len) == 0) {
                /* Found existing interned string */
                entry->ref_count++;
                return entry->str;
            }
        }
        
        /* Move to next slot (linear probing) */
        index = (index + 1) % g_intern_table.capacity;
        
    } while (index != start_index);
    
    /* Table is full - need to resize */
    /* For simplicity, we'll just return a new allocation */
    /* A production implementation should resize the table */
    char* new_str = (char*)malloc(len + 1);
    if (!new_str) {
        return NULL;
    }
    memcpy(new_str, str, len);
    new_str[len] = '\0';
    return new_str;
}

const char* vp_str_intern(const char* str) {
    if (!str) {
        return NULL;
    }
    
    size_t len = strlen(str);
    return vp_str_intern_len(str, len);
}

const char* vp_str_intern_len(const char* str, size_t len) {
    if (!str) {
        return NULL;
    }
    
    /* Don't intern very long strings */
    if (len > VIPER_INTERN_MAX_LENGTH) {
        char* new_str = (char*)malloc(len + 1);
        if (!new_str) return NULL;
        memcpy(new_str, str, len);
        new_str[len] = '\0';
        return new_str;
    }
    
    /* Initialize if needed */
    if (g_intern_table.entries == NULL) {
        vp_str_intern_init();
    }
    
    /* Compute hash */
    uint64_t hash = hash_string(str, len);
    
    /* Find or insert */
    return intern_insert(str, len, hash);
}

const char* vp_str_intern_find(const char* str) {
    if (!str || g_intern_table.entries == NULL) {
        return NULL;
    }
    
    size_t len = strlen(str);
    uint64_t hash = hash_string(str, len);
    
    /* Linear probing search */
    int64_t index = get_index(hash, g_intern_table.capacity);
    int64_t start_index = index;
    
    do {
        InternEntry* entry = &g_intern_table.entries[index];
        
        if (!entry->occupied) {
            return NULL;  /* Not found */
        }
        
        if (entry->hash == hash && entry->len == len) {
            if (memcmp(entry->str, str, len) == 0) {
                return entry->str;
            }
        }
        
        index = (index + 1) % g_intern_table.capacity;
        
    } while (index != start_index);
    
    return NULL;
}

int64_t vp_str_intern_count(void) {
    return g_intern_table.count;
}

int64_t vp_str_intern_memory_usage(void) {
    return g_intern_table.memory_used + 
           (g_intern_table.capacity * (int64_t)sizeof(InternEntry));
}

bool vp_str_intern_eq(const char* a, const char* b) {
    if (a == b) {
        return true;  /* Same pointer - definitely equal */
    }
    
    if (!a || !b) {
        return false;
    }
    
    /* Different pointers - compare contents */
    return strcmp(a, b) == 0;
}
