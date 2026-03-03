# Compact Dictionary Implementation Plan

## 🚀 Critical Optimization: String Interning

**Problem**: Without interning, every dict lookup requires `strcmp()` - character-by-character comparison.

**Solution**: String interning makes key comparison a **single pointer comparison** - same speed as integer comparison.

### How It Works

```
String Literal in Code          Global String Pool
     "user_id"  ─────────────►  [0x1000] { hash: 0xABCD, len: 7, data: "user_id" }
     "user_id"  ─────────────►  (returns same 0x1000 pointer - deduplicated!)
     "email"    ─────────────►  [0x2000] { hash: 0x1234, len: 5, data: "email" }
```

### Tagged Pointer Scheme for ViperValue

```
LSB Bits  Type
  00      BigInt (GMP) / Raw i64 (tagged int)
  01      Interned String (heap pointer)
  10      Dictionary / Array
  11      User Class Instance
```

### Fast Dict Lookup with Interned Strings

```c
ViperValue vp_compact_dict_get(CompactDict* dict, ViperValue key) {
    uint64_t hash = get_hash_from_value(key);  // Fast: read struct field
    uint32_t idx = hash & (dict->index_size - 1);

    while (dict->index_map[idx] != INDEX_EMPTY) {
        CompactDictEntry* entry = &dict->entries[dict->index_map[idx]];
        
        // THE SPEED WIN: Single pointer comparison!
        if (entry->key.data.as_ptr == key.data.as_ptr) {
            return entry->value;
        }
        idx = (idx + 1) & (dict->index_size - 1);
    }
    return VIPER_NONE;
}
```

**Benefits:**
- ✅ **Deduplication**: "user_id" 1000× → stored once
- ✅ **O(1) key comparison**: Pointer compare vs `strcmp()`
- ✅ **ARC synergy**: Interned strings have `ref_count = -1` (immortal, no free)

---

## 🧵 String Interning Implementation

### ViperString Structure

```c
/* runtime/include/viper_string.h */

typedef struct {
    int64_t ref_count;      /* -1 = immortal (interned literal) */
    uint64_t hash;          /* Pre-computed hash for dict lookups */
    int64_t length;         /* String length */
    char data[];            /* Flexible array member */
} ViperString;
```

### Global String Pool

```c
/* runtime/strings/intern_pool.c */

#include <pthread.h>

#define STRING_POOL_CAPACITY 65536

typedef struct {
    ViperString** strings;          /* Array of string pointers */
    uint64_t* hashes;               /* Parallel hash array */
    size_t count;                   /* Current string count */
    size_t capacity;                /* Pool capacity */
    pthread_mutex_t lock;           /* Thread-safe */
} StringPool;

/* Global singleton pool */
static StringPool global_pool = {0};
static pthread_mutex_t pool_init_lock = PTHREAD_MUTEX_INITIALIZER;

void vp_string_pool_init(void) {
    pthread_mutex_lock(&pool_init_lock);
    if (global_pool.strings == NULL) {
        global_pool.capacity = STRING_POOL_CAPACITY;
        global_pool.strings = calloc(global_pool.capacity, sizeof(ViperString*));
        global_pool.hashes = calloc(global_pool.capacity, sizeof(uint64_t));
        global_pool.count = 0;
        pthread_mutex_init(&global_pool.lock, NULL);
    }
    pthread_mutex_unlock(&pool_init_lock);
}

/* FNV-1a hash for strings */
static inline uint64_t fnv1a_hash(const char* str, size_t len) {
    uint64_t hash = 14695981039346656037ULL;
    for (size_t i = 0; i < len; i++) {
        hash ^= (uint8_t)str[i];
        hash *= 1099511628211ULL;
    }
    return hash;
}

/* Lookup existing string in pool */
static ViperString* pool_lookup(const char* str, size_t len, uint64_t hash) {
    for (size_t i = 0; i < global_pool.count; i++) {
        if (global_pool.hashes[i] != hash) continue;
        
        ViperString* existing = global_pool.strings[i];
        if (existing->length != len) continue;
        if (memcmp(existing->data, str, len) == 0) {
            return existing;  /* Found! */
        }
    }
    return NULL;
}

/* Intern a string - returns canonical pointer */
ViperString* vp_string_intern(const char* str, size_t len) {
    if (!str) return NULL;
    
    uint64_t hash = fnv1a_hash(str, len);
    
    pthread_mutex_lock(&global_pool.lock);
    
    /* Check if already interned */
    ViperString* existing = pool_lookup(str, len, hash);
    if (existing) {
        pthread_mutex_unlock(&global_pool.lock);
        return existing;
    }
    
    /* Allocate new interned string */
    ViperString* new_str = malloc(sizeof(ViperString) + len + 1);
    new_str->ref_count = -1;  /* Immortal - never freed */
    new_str->hash = hash;
    new_str->length = len;
    memcpy(new_str->data, str, len);
    new_str->data[len] = '\0';
    
    /* Add to pool */
    if (global_pool.count >= global_pool.capacity) {
        /* Grow pool */
        global_pool.capacity *= 2;
        global_pool.strings = realloc(global_pool.strings, 
                                       global_pool.capacity * sizeof(ViperString*));
        global_pool.hashes = realloc(global_pool.hashes,
                                      global_pool.capacity * sizeof(uint64_t));
    }
    
    global_pool.strings[global_pool.count] = new_str;
    global_pool.hashes[global_pool.count] = hash;
    global_pool.count++;
    
    pthread_mutex_unlock(&global_pool.lock);
    return new_str;
}

/* Convenience function for C strings */
ViperString* vp_string_intern_cstr(const char* str) {
    return vp_string_intern(str, strlen(str));
}

/* Get hash from interned string */
static inline uint64_t vp_string_get_hash(ViperString* s) {
    return s ? s->hash : 0;
}

/* Compare two interned strings - O(1) pointer comparison! */
static inline bool vp_string_equals(ViperString* a, ViperString* b) {
    return a == b;  /* Same pointer = same string */
}
```

### Rust Codegen: Emitting Interned Strings

```rust
// src/codegen/literals.rs

use inkwell::values::{BasicValue, PointerValue};

pub struct StringConstants<'ctx> {
    constants: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// Emit a string literal - returns interned, tagged pointer
    pub fn emit_string_literal(&self, value: &str) -> Result<PointerValue<'ctx>, String> {
        // Check if we already emitted this string in this module
        if let Some(&global) = self.string_constants.get(value) {
            return Ok(global);
        }

        // Create ViperString struct type if not exists
        let viper_string_type = self.get_or_create_viper_string_type();
        
        // Create global constant for the string data
        let string_data = self.context.const_string(value.as_bytes(), true);
        let string_data_ptr = self.builder.build_global_string_ptr(value, "str_data");
        
        // Compute hash at compile time (FNV-1a)
        let hash = fnv1a_hash_compile_time(value);
        
        // Build ViperString struct:
        // { ref_count: -1, hash: u64, length: i64, data: [N x i8] }
        let ref_count = self.context.i64_type().const_int(-1i64 as u64, true);  // Immortal
        let hash_val = self.context.i64_type().const_int(hash, false);
        let length = self.context.i64_type().const_int(value.len() as u64, false);
        
        let viper_string = self.builder.build_struct(
            viper_string_type,
            &[ref_count.into(), hash_val.into(), length.into(), string_data_ptr.as_basic_value()],
            "viper_str",
        );
        
        // Create global variable for the ViperString
        let global = self.module.add_global(viper_string_type, None, "vip_str");
        global.set_initializer(&viper_string);
        global.set_constant(true);
        global.set_thread_local_mode(Some(inkwell::module::ThreadLocalMode::GeneralDynamicTLSModel));
        
        // Tag the pointer: LSB = 01 for interned string
        let ptr_as_int = self.builder.build_ptr_to_int(global, self.context.i64_type(), "ptr_int");
        let tag = self.context.i64_type().const_int(0b01, false);  // Interned string tag
        let tagged = self.builder.build_or(ptr_as_int, tag, "tagged_str");
        
        self.string_constants.insert(value.to_string(), global);
        
        Ok(global)
    }
    
    /// Compile-time FNV-1a hash (const fn in Rust)
    fn fnv1a_hash_compile_time(&self, str: &str) -> u64 {
        const FNV_OFFSET: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;
        
        let mut hash = FNV_OFFSET;
        for byte in str.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }
}
```

### Updated ViperValue with String Tag

```c
/* runtime/include/viper_types.h */

/* Type tags encoded in LSB of ViperValue */
typedef enum {
    VIPER_TAG_RAW_I64    = 0b00,  /* Small integer (tagged pointer) */
    VIPER_TAG_INTERNED_STR = 0b01, /* Interned string pointer */
    VIPER_TAG_HEAP_OBJ   = 0b10,  /* Dict, List, BigInt */
    VIPER_TAG_CLASS_INST = 0b11,  /* User class instance */
} ViperTag;

/* Extract tag from ViperValue */
static inline ViperTag vp_value_get_tag(ViperValue val) {
    return (ViperTag)(val.data.as_ptr & 0b11);
}

/* Check if value is an interned string */
static inline bool vp_value_is_interned_str(ViperValue val) {
    return vp_value_get_tag(val) == VIPER_TAG_INTERNED_STR;
}

/* Get ViperString from tagged value */
static inline ViperString* vp_value_to_viper_string(ViperValue val) {
    if (vp_value_is_interned_str(val)) {
        return (ViperString*)(val.data.as_ptr & ~0b11);  /* Clear tag bits */
    }
    return NULL;
}

/* Create ViperValue from interned string */
static inline ViperValue vp_value_from_viper_string(ViperString* s) {
    ViperValue val;
    val.type = VIPER_TYPE_STR;  /* For switch statements */
    val.data.as_ptr = (void*)((uintptr_t)s | VIPER_TAG_INTERNED_STR);
    val._reserved = 0;
    return val;
}
```

### Updated Hash Function for Interned Strings

```c
/* Now hash extraction is O(1) - just read the pre-computed field! */
static inline uint64_t vp_compact_dict_hash(ViperValue key) {
    if (vp_value_is_interned_str(key)) {
        ViperString* s = vp_value_to_viper_string(key);
        return s->hash;  /* Pre-computed, O(1)! */
    }
    
    /* Fallback for non-interned keys */
    switch (key.type) {
        case VIPER_TYPE_I64:
            return fnv1a_hash_bytes((const uint8_t*)&key.data.as_i64, 8);
        case VIPER_TYPE_F64:
            return fnv1a_hash_bytes((const uint8_t*)&key.data.as_f64, 8);
        /* ... other types ... */
        default:
            return 0;
    }
}
```

### Updated Key Comparison - O(1) Pointer Compare

```c
/* Value equality check - optimized for interned strings */
static inline bool vp_values_equal(ViperValue a, ViperValue b) {
    /* Fast path: both are interned strings */
    if (vp_value_is_interned_str(a) && vp_value_is_interned_str(b)) {
        return a.data.as_ptr == b.data.as_ptr;  /* Pointer comparison! */
    }
    
    /* Fast path: both are raw i64 */
    if (vp_value_get_tag(a) == VIPER_TAG_RAW_I64 && 
        vp_value_get_tag(b) == VIPER_TAG_RAW_I64) {
        return a.data.as_i64 == b.data.as_i64;
    }
    
    /* Slow path: type-specific comparison */
    if (a.type != b.type) return false;
    
    switch (a.type) {
        case VIPER_TYPE_I64: return a.data.as_i64 == b.data.as_i64;
        case VIPER_TYPE_F64: return a.data.as_f64 == b.data.as_f64;
        case VIPER_TYPE_BOOL: return a.data.as_bool == b.data.as_bool;
        case VIPER_TYPE_STR: {
            /* Both are strings - check if interned first */
            ViperString* sa = vp_value_to_viper_string(a);
            ViperString* sb = vp_value_to_viper_string(b);
            if (sa && sb) return sa == sb;  /* Interned: pointer compare */
            return strcmp(a.data.as_str, b.data.as_str) == 0;  /* Non-interned: strcmp */
        }
        default: return false;
    }
}
```

---

## 📊 Analysis: Current vs. Compact Dict

### Current Implementation (Chaining with Separate Chaining)

**Structure:**
- Array of buckets (pointers to `DictEntry`)
- Each entry: `key` (8B) + `value` (24B) + `next` (8B) = 40 bytes
- Collision resolution: Linked list chaining

**Problems:**
1. **Memory inefficiency**: 50-75% of bucket array is empty (load factor 0.75)
2. **Cache unfriendly**: Entries scattered across heap, pointer chasing
3. **No insertion order**: Cannot iterate in insertion order
4. **Worst case O(n)**: All keys hash to same bucket → linked list

### Recommended: Compact Dict (Two-Array / Dense Storage)

**Structure:**
```
Index Map (sparse):  [None, 0, None, 1, None, None, 2, None]  (1 byte per slot)
                     ↓      ↓                    ↓
Entries Array (dense): [{"hash", key0, val0}, {"hash", key1, val1}, {"hash", key2, val2}]
                       ↑_______________________________________________↑
                       Linear scan for iteration - no gaps!
```

**Benefits:**
- ✅ **30-40% less memory** - No holes in entry storage
- ✅ **Cache-friendly iteration** - Linear scan, CPU prefetcher works
- ✅ **Insertion order preserved** - Python 3.6+ behavior
- ✅ **Better cache locality** - Entries contiguous in memory

---

## 🏗️ Revised Data Structure Design

### Key Improvements Over Initial Plan

| Feature | Old Plan | New Plan |
|---------|----------|----------|
| Index width | Fixed `int8_t` | Dynamic (8/16/32-bit) |
| Max entries | 254 | 4+ billion |
| Key type | `char*` only | `ViperValue` (any type) |
| Memory overhead | ~24 bytes/entry | ~48 bytes/entry (type-safe) |
| Tombstone handling | Standard tombstones | Vacuum with compaction |

### Data Structures

```c
/* ============================================ */
/* Index Width Enumeration */
/* ============================================ */

typedef enum {
    INDEX_WIDTH_8  = 1,   /* uint8_t  - up to 252 entries */
    INDEX_WIDTH_16 = 2,   /* uint16_t - up to 65,532 entries */
    INDEX_WIDTH_32 = 4,   /* uint32_t - up to 4B entries */
} IndexWidth;

/* ============================================ */
/* CompactDictEntry - 48 bytes per entry */
/* ============================================ */

typedef struct {
    uint64_t hash;          /* 0:  Stored hash for fast comparison */
    ViperValue key;         /* 8:  Tagged pointer (int, float, str, etc.) */
    ViperValue value;       /* 32: Tagged value */
} CompactDictEntry;         /* Total: 48 bytes */

/* ============================================ */
/* CompactDict - 56 bytes header + variable */
/* ============================================ */

typedef struct {
    int64_t ref_count;          /* 0:  ARC reference count */
    int64_t entry_count;        /* 8:  Number of live entries */
    int64_t tombstone_count;    /* 16: Deleted entries needing vacuum */
    int32_t index_size;         /* 20: Size of index map (power of 2) */
    uint8_t index_width;        /* 24: 1=uint8, 2=uint16, 4=uint32 */
    uint8_t _padding[3];        /* 25: Alignment padding */
    void* index_map;            /* 32: Typed based on index_width */
    CompactDictEntry* entries;  /* 40: Dense array of entries */
    int32_t entries_cap;        /* 48: Capacity of entries array */
    uint8_t _reserved[4];       /* 52: Padding for alignment */
} CompactDict;                  /* Total: 56 bytes header */
```

### Memory Layout Diagram

```
CompactDict (56 bytes header)
├─ ref_count:        8 bytes
├─ entry_count:      8 bytes
├─ tombstone_count:  8 bytes
├─ index_size:       4 bytes
├─ index_width:      1 byte
├─ padding:          3 bytes
├─ index_map:        8 bytes (points to external array)
├─ entries:          8 bytes (points to external array)
├─ entries_cap:      4 bytes
└─ reserved:         4 bytes

Index Map (variable width)
├─ uint8_t[256]   = 256 bytes  (small dicts)
├─ uint16_t[512]  = 1024 bytes (medium dicts)
└─ uint32_t[1024] = 4096 bytes (large dicts)

Entries Array (48 bytes per entry)
├─ hash:   8 bytes
├─ key:    24 bytes (ViperValue - tagged pointer)
└─ value:  24 bytes (ViperValue - tagged pointer)
```

**Total per entry:** ~48-56 bytes (depending on index width amortized)

---

## 📊 Performance Comparison

| Metric | Current (Chaining) | Compact Dict | Improvement |
|--------|-------------------|--------------|-------------|
| **Cache Locality** | Poor (pointer chasing) | Excellent (linear scan) | ~10x faster iteration |
| **Insertion Order** | Not guaranteed | Guaranteed | Python 3.6+ compatible |
| **Memory Overhead** | ~48+ bytes/entry | ~48 bytes/entry | 25-40% savings |
| **LLVM Optimization** | Low | High (auto-vectorization) | Better codegen |
| **Worst Case Lookup** | O(n) linked list | O(n) probing | Similar, but rarer |

### Memory Comparison by Scenario

| Scenario | Current (Chaining) | Compact Dict | Savings |
|----------|-------------------|--------------|---------|
| 100 entries, 200 buckets | 200×8B + 100×48B = 6.4KB | 200×1B + 100×48B = 5.0KB | **22%** |
| 1000 entries, 2000 buckets | 2000×8B + 1000×48B = 64KB | 2000×1B + 1000×48B = 50KB | **22%** |
| 10000 entries | ~640KB | ~500KB | **22%** |

---

## 🔧 Implementation Details

### Phase 2: Hash Function with ViperValue Keys

```c
/* Constants for FNV-1a hash */
#define FNV_OFFSET_BASIS 14695981039346656037ULL
#define FNV_PRIME 1099511628211ULL

/* FNV-1a hash for arbitrary ViperValue keys */
static uint64_t vp_compact_dict_hash(ViperValue key) {
    switch (key.type) {
        case VIPER_TYPE_I64:
        case VIPER_TYPE_BOOL:
            /* Direct hash of bits */
            return fnv1a_hash_bytes((const uint8_t*)&key.data.as_i64, 8);
        
        case VIPER_TYPE_F64:
            /* Hash float bits (handles -0.0, NaN consistently) */
            return fnv1a_hash_bytes((const uint8_t*)&key.data.as_f64, 8);
        
        case VIPER_TYPE_STR: {
            /* Hash string contents */
            const char* str = key.data.as_str;
            return fnv1a_hash_bytes((const uint8_t*)str, strlen(str));
        }
        
        case VIPER_TYPE_LIST:
        case VIPER_TYPE_DICT:
        case VIPER_TYPE_OBJECT:
            /* Hash pointer identity (reference equality) */
            return fnv1a_hash_bytes((const uint8_t*)&key.data.as_generic, 8);
        
        case VIPER_TYPE_NONE:
        default:
            return FNV_OFFSET_BASIS;  /* Constant for None */
    }
}

/* FNV-1a helper */
static inline uint64_t fnv1a_hash_bytes(const uint8_t* data, size_t len) {
    uint64_t hash = FNV_OFFSET_BASIS;
    for (size_t i = 0; i < len; i++) {
        hash ^= data[i];
        hash *= FNV_PRIME;
    }
    return hash;
}
```

**Benefits:**
- ✅ **Type-agnostic keys**: int, float, string, bool, objects
- ✅ **Tagged pointer efficiency**: Small ints don't allocate
- ✅ **Reference equality for objects**: Fast pointer comparison

---

### Phase 3: Core Operations

#### Index Map Accessors (Dynamic Width)

```c
/* Constants */
#define INDEX_EMPTY   (-1)
#define INDEX_TOMBSTONE (-2)

/* Get index entry based on width */
static inline int64_t get_index_entry(CompactDict* dict, int32_t idx) {
    switch (dict->index_width) {
        case INDEX_WIDTH_8:
            return ((int8_t*)dict->index_map)[idx];
        case INDEX_WIDTH_16:
            return ((int16_t*)dict->index_map)[idx];
        case INDEX_WIDTH_32:
            return ((int32_t*)dict->index_map)[idx];
        default:
            return INDEX_EMPTY;
    }
}

/* Set index entry based on width */
static inline void set_index_entry(CompactDict* dict, int32_t idx, int64_t value) {
    switch (dict->index_width) {
        case INDEX_WIDTH_8:
            ((int8_t*)dict->index_map)[idx] = (int8_t)value;
            break;
        case INDEX_WIDTH_16:
            ((int16_t*)dict->index_map)[idx] = (int16_t)value;
            break;
        case INDEX_WIDTH_32:
            ((int32_t*)dict->index_map)[idx] = (int32_t)value;
            break;
    }
}
```

#### Lookup with Open Addressing

```c
ViperValue vp_compact_dict_get(CompactDict* dict, ViperValue key) {
    uint64_t hash = vp_compact_dict_hash(key);
    int32_t mask = dict->index_size - 1;
    int32_t idx = hash & mask;
    
    int64_t tombstone_slot = -1;  /* Track first tombstone for insertion */
    
    while (true) {
        int64_t entry_idx = get_index_entry(dict, idx);
        
        if (entry_idx == INDEX_EMPTY) {
            /* Not found - return None */
            ViperValue none = {0};
            none.type = VIPER_TYPE_NONE;
            return none;
        }
        
        if (entry_idx == INDEX_TOMBSTONE) {
            if (tombstone_slot == -1) {
                tombstone_slot = idx;  /* Remember for potential insert */
            }
        } else {
            /* Occupied slot - check hash and key equality */
            CompactDictEntry* entry = &dict->entries[entry_idx];
            if (entry->hash == hash && vp_values_equal(entry->key, key)) {
                return entry->value;  /* Found! */
            }
        }
        
        /* Linear probing with wrap-around */
        idx = (idx + 1) & mask;
    }
}
```

#### Insert/Update Operation

```c
void vp_compact_dict_set(CompactDict* dict, ViperValue key, ViperValue value) {
    /* Check load factor - resize if needed */
    if ((double)(dict->entry_count + 1) / dict->index_size > 0.66) {
        vp_compact_dict_resize(dict);
    }
    
    uint64_t hash = vp_compact_dict_hash(key);
    int32_t mask = dict->index_size - 1;
    int32_t idx = hash & mask;
    
    int64_t tombstone_slot = -1;
    
    while (true) {
        int64_t entry_idx = get_index_entry(dict, idx);
        
        if (entry_idx == INDEX_EMPTY) {
            /* Found empty slot - insert here */
            int32_t insert_slot = (tombstone_slot >= 0) ? tombstone_slot : idx;
            
            /* Add new entry to entries array */
            if (dict->entry_count >= dict->entries_cap) {
                vp_compact_dict_grow_entries(dict);
            }
            
            int32_t new_entry_idx = dict->entry_count;
            CompactDictEntry* new_entry = &dict->entries[new_entry_idx];
            new_entry->hash = hash;
            new_entry->key = key;  /* ViperValue copy (shallow for strings) */
            new_entry->value = value;
            
            dict->entry_count++;
            set_index_entry(dict, insert_slot, new_entry_idx);
            
            if (tombstone_slot >= 0) {
                dict->tombstone_count--;
            }
            return;
        }
        
        if (entry_idx == INDEX_TOMBSTONE) {
            if (tombstone_slot == -1) {
                tombstone_slot = idx;
            }
        } else {
            /* Check for existing key */
            CompactDictEntry* entry = &dict->entries[entry_idx];
            if (entry->hash == hash && vp_values_equal(entry->key, key)) {
                /* Update existing value */
                vp_value_decref(entry->value);  /* Handle old value refcount */
                entry->value = value;
                return;
            }
        }
        
        idx = (idx + 1) & mask;
    }
}
```

---

### Phase 4: Resizing with Index Width Promotion

```c
/* Determine optimal index width for entry count */
static inline uint8_t get_index_width_for_entries(int64_t count) {
    if (count <= 252) return INDEX_WIDTH_8;
    if (count <= 65532) return INDEX_WIDTH_16;
    return INDEX_WIDTH_32;
}

void vp_compact_dict_resize(CompactDict* dict) {
    int32_t new_index_size = dict->index_size * 2;
    uint8_t new_width = get_index_width_for_entries(dict->entry_count);
    
    /* Allocate new index map */
    size_t map_bytes = new_index_size * new_width;
    void* new_index_map = malloc(map_bytes);
    memset(new_index_map, 0xFF, map_bytes);  /* Fill with -1 (EMPTY) */
    
    /* Rehash all live entries */
    int32_t new_mask = new_index_size - 1;
    for (int32_t i = 0; i < dict->entries_cap; i++) {
        CompactDictEntry* entry = &dict->entries[i];
        
        /* Skip tombstoned entries */
        if (entry->key.type == VIPER_TYPE_NONE && entry->key.data.as_i64 == 0) {
            continue;
        }
        
        int32_t idx = entry->hash & new_mask;
        while (true) {
            int64_t existing = get_index_entry_raw(new_index_map, new_width, idx);
            if (existing == INDEX_EMPTY) {
                set_index_entry_raw(new_index_map, new_width, idx, i);
                break;
            }
            idx = (idx + 1) & new_mask;
        }
    }
    
    /* Free old index map */
    free(dict->index_map);
    
    dict->index_map = new_index_map;
    dict->index_size = new_index_size;
    dict->index_width = new_width;
    dict->tombstone_count = 0;  /* All tombstones cleared */
}

/* Grow entries array independently */
void vp_compact_dict_grow_entries(CompactDict* dict) {
    int32_t new_cap = dict->entries_cap * 2;
    CompactDictEntry* new_entries = realloc(dict->entries, new_cap * sizeof(CompactDictEntry));
    
    /* Compact entries while growing - remove tombstones */
    vp_compact_dict_compact(dict, new_entries, new_cap);
}
```

---

### Phase 5: Tombstone-Free Vacuum

```c
/* Vacuum: Remove tombstones, compact entries, rebuild index map */
void vp_compact_dict_vacuum(CompactDict* dict) {
    /* Trigger vacuum when tombstones > 25% of entries */
    if (dict->tombstone_count < dict->entry_count / 4) {
        return;
    }
    
    /* Compact entries array - slide live entries down */
    int32_t write_idx = 0;
    for (int32_t read_idx = 0; read_idx < dict->entries_cap; read_idx++) {
        CompactDictEntry* entry = &dict->entries[read_idx];
        
        /* Check if entry is live (not tombstoned) */
        if (!(entry->key.type == VIPER_TYPE_NONE && entry->key.data.as_i64 == 0)) {
            if (write_idx != read_idx) {
                /* Move entry to new position */
                dict->entries[write_idx] = *entry;
                entry->key.type = VIPER_TYPE_NONE;  /* Mark old as tombstone */
            }
            write_idx++;
        }
    }
    
    dict->entry_count = write_idx;
    dict->tombstone_count = 0;
    
    /* Rebuild index map from scratch */
    memset(dict->index_map, 0xFF, dict->index_size * dict->index_width);
    int32_t mask = dict->index_size - 1;
    
    for (int32_t i = 0; i < write_idx; i++) {
        CompactDictEntry* entry = &dict->entries[i];
        int32_t idx = entry->hash & mask;
        
        while (true) {
            int64_t existing = get_index_entry(dict, idx);
            if (existing == INDEX_EMPTY) {
                set_index_entry(dict, idx, i);
                break;
            }
            idx = (idx + 1) & mask;
        }
    }
}

/* Delete with tombstone */
bool vp_compact_dict_remove(CompactDict* dict, ViperValue key) {
    uint64_t hash = vp_compact_dict_hash(key);
    int32_t mask = dict->index_size - 1;
    int32_t idx = hash & mask;
    
    while (true) {
        int64_t entry_idx = get_index_entry(dict, idx);
        
        if (entry_idx == INDEX_EMPTY) {
            return false;  /* Not found */
        }
        
        if (entry_idx == INDEX_TOMBSTONE) {
            /* Continue probing */
        } else {
            CompactDictEntry* entry = &dict->entries[entry_idx];
            if (entry->hash == hash && vp_values_equal(entry->key, key)) {
                /* Found - mark as tombstone in entries */
                vp_value_decref(entry->key);
                vp_value_decref(entry->value);
                entry->key.type = VIPER_TYPE_NONE;  /* Tombstone marker */
                entry->key.data.as_i64 = 0;
                
                /* Mark index slot as tombstone */
                set_index_entry(dict, idx, INDEX_TOMBSTONE);
                dict->tombstone_count++;
                
                /* Trigger vacuum if needed */
                if (dict->tombstone_count > dict->entry_count / 4) {
                    vp_compact_dict_vacuum(dict);
                }
                
                return true;
            }
        }
        
        idx = (idx + 1) & mask;
    }
}
```

---

### Phase 6: Insertion-Order Iteration

```c
/* Iterator - linear scan of entries array */
typedef struct {
    CompactDict* dict;
    int32_t entry_index;
} CompactDictIter;

CompactDictIter* vp_compact_dict_iter_create(CompactDict* dict) {
    CompactDictIter* iter = malloc(sizeof(CompactDictIter));
    iter->dict = dict;
    iter->entry_index = 0;
    
    /* Skip any leading tombstones */
    while (iter->entry_index < dict->entries_cap) {
        CompactDictEntry* entry = &dict->entries[iter->entry_index];
        if (!(entry->key.type == VIPER_TYPE_NONE && entry->key.data.as_i64 == 0)) {
            break;
        }
        iter->entry_index++;
    }
    
    return iter;
}

bool vp_compact_dict_iter_next(CompactDictIter* iter, ViperValue* key, ViperValue* value) {
    while (iter->entry_index < iter->dict->entries_cap) {
        CompactDictEntry* entry = &iter->dict->entries[iter->entry_index];
        iter->entry_index++;
        
        /* Skip tombstones */
        if (entry->key.type == VIPER_TYPE_NONE && entry->key.data.as_i64 == 0) {
            continue;
        }
        
        *key = entry->key;
        *value = entry->value;
        return true;
    }
    
    return false;  /* No more entries */
}

void vp_compact_dict_iter_free(CompactDictIter* iter) {
    free(iter);
}
```

**Key Benefit:** Pure linear scan - CPU prefetcher can load 10+ entries ahead!

---

## 📅 Implementation Timeline

| Week | Tasks | Deliverable |
|------|-------|-------------|
| 1 | 0.1-0.5 | String interning infrastructure complete |
| 2 | 1, 2, 7 | `viper_types.h` updated, hash function ready |
| 3 | 3, 4, 5 | Core ops + resize + vacuum working |
| 4 | 6, 8, 9 | Iteration + LLVM bindings + JIT |
| 5 | 10, 11 | Tests + benchmarks |

---

## ✅ Task List

### String Interning (Prerequisite)
- [ ] **Task 0.1**: Create `viper_string.h` with ViperString struct
- [ ] **Task 0.2**: Implement global string pool (`intern_pool.c`)
- [ ] **Task 0.3**: Add string interning to Rust codegen (`literals.rs`)
- [ ] **Task 0.4**: Update `viper_types.h` with string tag enums
- [ ] **Task 0.5**: Implement `vp_values_equal()` with fast path for interned strings

### Compact Dictionary Core
- [ ] **Task 1**: Design CompactDict with dynamic index width (uint8_t/uint16_t/uint32_t)
- [ ] **Task 2**: Implement hash function with ViperValue key support (tagged pointers)
- [ ] **Task 3**: Implement CompactDict core operations (create, set, get, contains)
- [ ] **Task 4**: Implement resizing with index width promotion (8→16→32 bit)
- [ ] **Task 5**: Implement tombstone-free vacuum with entry compaction
- [ ] **Task 6**: Implement insertion-order iteration (dense linear scan)
- [ ] **Task 7**: Update `viper_types.h` with CompactDict and CompactDictEntry structs
- [ ] **Task 8**: Update `dicts.rs` codegen bindings for new structure
- [ ] **Task 9**: Update JIT stubs for CompactDict with ViperValue keys
- [ ] **Task 10**: Write comprehensive tests (resize, vacuum, type-specific keys)
- [ ] **Task 11**: Benchmark: memory usage, lookup speed, iteration vs old chaining

---

## 🎯 Key Design Decisions

### 1. Dynamic Index Width
- **Problem**: Fixed `int8_t` limits to 254 entries
- **Solution**: Promote width as dict grows (8→16→32 bit)
- **Benefit**: Memory efficient for small dicts, unlimited for large

### 2. ViperValue Keys (Not `char*`)
- **Problem**: `char*` keys cause memory fragmentation
- **Solution**: Use `ViperValue` for keys (tagged pointers)
- **Benefit**: Any type can be a key, small ints don't allocate

### 3. Tombstone-Free Vacuum
- **Problem**: Tombstones accumulate and slow lookups
- **Solution**: Compact entries array, rebuild index map
- **Benefit**: Dense iteration, no tombstone overhead

### 4. Two-Array Separation
- **Problem**: Traditional hash tables waste space on empty slots
- **Solution**: Separate index map (sparse) from entries (dense)
- **Benefit**: 25-40% memory savings, cache-friendly iteration

---

## 📈 Expected Performance Gains

| Metric | Current | Compact Dict | Compact + Interning | Improvement |
|--------|---------|--------------|---------------------|-------------|
| Memory per entry | ~64 bytes | ~48 bytes | ~48 bytes | **25% reduction** |
| String key lookup | O(n) `strcmp()` | O(n) `strcmp()` | **O(1) pointer compare** | **10-100x faster** |
| String hash | O(n) compute | O(n) compute | **O(1) pre-computed** | **Instant** |
| Lookup (cache hits) | 2-3 | 1-2 | 1-2 | **33% faster** |
| Iteration speed | O(n) with gaps | O(n) linear | O(n) linear | **5-10x faster** |
| String memory | N × len | N × len | **1 × len (dedup)** | **90%+ savings** |
| Insertion order | ❌ | ✅ | ✅ | Python compatible |
| Type support | Strings only | All types | All types | More flexible |

### String Interning Impact Example

```
Code: 1000 dict lookups with "user_id" key

Without Interning:
  - 1000 × hash("user_id") = 1000 × 7 char iterations
  - 1000 × strcmp("user_id", key) = up to 7000 char comparisons
  Total: ~14,000 operations

With Interning:
  - Hash: Pre-computed at compile time = 0 operations
  - Comparison: Single pointer compare = 1 operation
  Total: ~1,000 operations (14x faster!)

Memory Savings:
  - "user_id" appears 1000× in code
  - Without: 1000 × 8 bytes = 8000 bytes
  - With: 1 × 8 bytes = 8 bytes (99.9% savings!)
```

---

## 🔬 Future Optimizations

1. **SIMD Lookup**: Use SIMD to check 8-16 index slots in parallel
2. **Small Dict Optimization**: Embed entries in header for dicts < 4 entries
3. **Robin Hood Hashing**: Track PSL for more predictable lookups
4. **SWAR Techniques**: Use word-level parallelism for tombstone detection
5. **Thread-Local String Pools**: Per-thread pools to reduce lock contention
6. **String Pool Serialization**: Dump/load interned strings for faster startup
7. **Adaptive Index Width**: Grow/shrink width dynamically based on occupancy
8. **Vectorized Rehashing**: SIMD-accelerated rehash during resize

---

**Document Version:** 1.1  
**Created:** 2026-03-04  
**Updated:** 2026-03-04 (Added String Interning)  
**Status:** Ready for implementation
