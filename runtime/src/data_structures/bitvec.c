/**
 * Viper Bit Vector Implementation
 * Type-specific list for bool elements (1 bit per element)
 *
 * Memory savings: 8x compared to int8_t bool list, 64x compared to int64_t list
 *
 * Implementation: Uses uint64_t array where each bit represents a boolean value.
 * Bit 0 of word 0 = index 0, Bit 1 of word 0 = index 1, ..., Bit 63 of word 0 = index 63,
 * Bit 0 of word 1 = index 64, etc.
 *
 * SIMD Optimizations:
 * - AVX2: Process 256 bits (4 words) at a time
 * - SSE2: Process 128 bits (2 words) at a time
 * - POPCNT: Hardware population count for fast bit counting
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include "viper_stdlib.h"

/* SIMD intrinsics */
#ifdef __AVX2__
#include <immintrin.h>
#define VP_HAS_AVX2 1
#else
#define VP_HAS_AVX2 0
#endif

#ifdef __SSE2__
#include <emmintrin.h>
#define VP_HAS_SSE2 1
#else
#define VP_HAS_SSE2 0
#endif

#define BITVEC_INITIAL_CAPACITY 64  /* 64 bits = 8 bytes initially */
#define BITVEC_GROWTH_FACTOR 2

/* Number of bits in a uint64_t word */
#define BITS_PER_WORD 64

/* Get the word index for a given bit index */
static inline int64_t bitvec_word_index(int64_t bit_index) {
    return bit_index / BITS_PER_WORD;
}

/* Get the bit mask for a given bit index within its word */
static inline uint64_t bitvec_bit_mask(int64_t bit_index) {
    return (uint64_t)1 << (bit_index % BITS_PER_WORD);
}

/* Calculate number of words needed for n bits */
static inline int64_t bitvec_words_needed(int64_t n_bits) {
    return (n_bits + BITS_PER_WORD - 1) / BITS_PER_WORD;
}

/* ============================================ */
/* Bit Vector Internal Functions                */
/* ============================================ */

static inline void vp_bitvec_grow(ViperList* vec) {
    int64_t new_capacity = vec->capacity * BITVEC_GROWTH_FACTOR;
    int64_t new_words = bitvec_words_needed(new_capacity);
    
    uint64_t* new_data = (uint64_t*)realloc(vec->data.data_bitvec, 
                                             (size_t)new_words * sizeof(uint64_t));

    if (!new_data) {
        vp_panic("Failed to grow bit vector");
    }

    /* Zero out the new words */
    int64_t old_words = bitvec_words_needed(vec->capacity);
    memset(new_data + old_words, 0, (size_t)(new_words - old_words) * sizeof(uint64_t));

    vec->data.data_bitvec = new_data;
    vec->capacity = new_capacity;
}

static void vp_bitvec_destroy(void* ptr) {
    ViperList* vec = (ViperList*)ptr;
    if (vec->data.data_bitvec) {
        free(vec->data.data_bitvec);
        vec->data.data_bitvec = NULL;
    }
}

/* Static inline versions for LTO inlining - used internally */

/* Unchecked version - no bounds checking, for hot loops */
static inline bool vp_bitvec_get_unchecked_inl(ViperList* vec, int64_t index) {
    int64_t word_idx = index / 64;
    uint64_t mask = (uint64_t)1 << (index % 64);
    return (vec->data.data_bitvec[word_idx] & mask) != 0;
}

/* Checked version with branch prediction hint */
static inline bool vp_bitvec_get_inl(ViperList* vec, int64_t index) {
    int64_t idx = index;
    if (__builtin_expect(idx < 0, 0)) idx = vec->length + idx;
    int64_t word_idx = idx / 64;
    uint64_t mask = (uint64_t)1 << (idx % 64);
    return (vec->data.data_bitvec[word_idx] & mask) != 0;
}

static inline void vp_bitvec_set_unchecked_inl(ViperList* vec, int64_t index, bool value) {
    int64_t word_idx = index / 64;
    uint64_t mask = (uint64_t)1 << (index % 64);
    if (value) {
        vec->data.data_bitvec[word_idx] |= mask;
    } else {
        vec->data.data_bitvec[word_idx] &= ~mask;
    }
}

static inline void vp_bitvec_set_inl(ViperList* vec, int64_t index, bool value) {
    int64_t idx = index;
    if (__builtin_expect(idx < 0, 0)) idx = vec->length + idx;
    int64_t word_idx = idx / 64;
    uint64_t mask = (uint64_t)1 << (idx % 64);
    if (value) {
        vec->data.data_bitvec[word_idx] |= mask;
    } else {
        vec->data.data_bitvec[word_idx] &= ~mask;
    }
}

/* Append with branch prediction for growth (rare case) */
static inline void vp_bitvec_append_inl(ViperList* vec, bool value) {
    if (__builtin_expect(vec->length >= vec->capacity, 0)) {
        vp_bitvec_grow(vec);
    }
    int64_t word_idx = vec->length / 64;
    uint64_t mask = (uint64_t)1 << (vec->length % 64);
    if (value) {
        vec->data.data_bitvec[word_idx] |= mask;
    } else {
        vec->data.data_bitvec[word_idx] &= ~mask;
    }
    vec->length++;
}

/* ============================================ */
/* SIMD-Optimized Bit Operations                */
/* ============================================ */

/**
 * SIMD-optimized clear range - clear all bits in a range [start, end)
 * Uses AVX2 to clear 256 bits (4 words) at a time
 */
static inline void vp_bitvec_clear_range_simd(uint64_t* data, int64_t start_word, int64_t end_word) {
#if VP_HAS_AVX2
    // AVX2: Clear 4 words (256 bits) at a time
    __m256i zero = _mm256_setzero_si256();
    
    int64_t i = start_word;
    
    // Align to 4-word boundary
    while (i < end_word && (i % 4) != 0) {
        data[i] = 0;
        i++;
    }
    
    // Process 4 words at a time
    while (i + 4 <= end_word) {
        _mm256_storeu_si256((__m256i*)&data[i], zero);
        i += 4;
    }
    
    // Handle remainder
    while (i < end_word) {
        data[i] = 0;
        i++;
    }
#elif VP_HAS_SSE2
    // SSE2: Clear 2 words (128 bits) at a time
    __m128i zero = _mm_setzero_si128();
    
    int64_t i = start_word;
    
    // Align to 2-word boundary
    while (i < end_word && (i % 2) != 0) {
        data[i] = 0;
        i++;
    }
    
    // Process 2 words at a time
    while (i + 2 <= end_word) {
        _mm_storeu_si128((__m128i*)&data[i], zero);
        i += 2;
    }
    
    // Handle remainder
    while (i < end_word) {
        data[i] = 0;
        i++;
    }
#else
    // Scalar fallback
    for (int64_t i = start_word; i < end_word; i++) {
        data[i] = 0;
    }
#endif
}

/**
 * SIMD-optimized bit counting using POPCNT
 * Returns the number of set bits in the range [0, word_count)
 */
static inline int64_t vp_bitvec_popcount_simd(uint64_t* data, int64_t word_count) {
    int64_t count = 0;
    
#if VP_HAS_AVX2
    // AVX2: Process 4 words at a time
    int64_t i = 0;
    
    while (i + 4 <= word_count) {
        __m256i v = _mm256_loadu_si256((__m256i*)&data[i]);
        uint64_t words[4];
        _mm256_storeu_si256((__m256i*)words, v);
        
        // Use hardware POPCNT
        for (int j = 0; j < 4; j++) {
            count += __builtin_popcountll(words[j]);
        }
        i += 4;
    }
    
    // Handle remainder
    while (i < word_count) {
        count += __builtin_popcountll(data[i]);
        i++;
    }
#elif VP_HAS_SSE2
    // SSE2: Process 2 words at a time
    int64_t i = 0;
    
    while (i + 2 <= word_count) {
        __m128i v = _mm_loadu_si128((__m128i*)&data[i]);
        uint64_t words[2];
        _mm_storeu_si128((__m128i*)words, v);
        
        count += __builtin_popcountll(words[0]);
        count += __builtin_popcountll(words[1]);
        i += 2;
    }
    
    // Handle remainder
    while (i < word_count) {
        count += __builtin_popcountll(data[i]);
        i++;
    }
#else
    // Scalar with POPCNT
    for (int64_t i = 0; i < word_count; i++) {
        count += __builtin_popcountll(data[i]);
    }
#endif
    
    return count;
}

/**
 * SIMD-optimized sieve marking - clear bits at regular intervals
 * For prime sieve: mark multiples of p as composite
 * 
 * Optimized for power-of-2 strides (2, 4, 8, 16, 32) which divide 64 evenly.
 */
static inline void vp_bitvec_mark_multiples_simd(ViperList* vec, int64_t start, int64_t stride, int64_t limit) {
    if (stride <= 0 || start > limit) return;
    
    // Fast path: for large aligned strides, clear whole words at once
    if (stride >= 64 && start % 64 == 0) {
        int64_t start_word = start / 64;
        int64_t end_word = limit / 64 + 1;
        
        if (end_word > vec->length / 64 + 1) {
            end_word = vec->length / 64 + 1;
        }
        
        if (start_word < end_word) {
            vp_bitvec_clear_range_simd(vec->data.data_bitvec, start_word, end_word);
        }
        return;
    }
    
    // Optimized path for power-of-2 strides that divide 64 evenly (2, 4, 8, 16, 32)
    if (stride >= 2 && stride <= 32 && (64 % stride) == 0) {
        uint64_t mask = 0;
        for (int64_t i = 0; i < 64; i += stride) {
            mask |= ((uint64_t)1 << i);
        }
        
        uint64_t* data = vec->data.data_bitvec;
        int64_t word_idx = start / 64;
        int64_t bit_offset = start % 64;
        
        // First word
        if (word_idx < (limit / 64 + 1)) {
            uint64_t first_mask = (mask >> bit_offset);
            if (bit_offset > 0) {
                first_mask |= (mask << (64 - bit_offset));
            }
            data[word_idx] &= ~first_mask;
            word_idx++;
        }
        
        // Full words
        int64_t end_word = limit / 64;
        while (word_idx < end_word) {
            data[word_idx] &= ~mask;
            word_idx++;
        }
        
        // Last word
        if (word_idx <= limit / 64 && word_idx < (vec->length / 64 + 1)) {
            int64_t last_bits = limit % 64;
            uint64_t last_mask = mask & ((uint64_t)(1ULL << (last_bits + 1)) - 1);
            data[word_idx] &= ~last_mask;
        }
        return;
    }
    
    // Fallback: scalar for non-power-of-2 strides (3, 5, 6, 7, 9, 10, etc.)
    for (int64_t i = start; i <= limit; i += stride) {
        vp_bitvec_set_unchecked_inl(vec, i, false);
    }
}

/* Public wrapper for SIMD-optimized strided marking */
void vp_bitvec_mark_multiples(ViperList* vec, int64_t start, int64_t stride, int64_t limit) {
    if (!vec || start < 0 || stride <= 0 || limit < start) {
        return;
    }
    vp_bitvec_mark_multiples_simd(vec, start, stride, limit);
}

/* ============================================ */
/* Bit Vector Public Functions                  */
/* ============================================ */

ViperList* vp_bitvec_create_with_capacity(int64_t cap) {
    ViperList* vec = (ViperList*)vp_arc_alloc(sizeof(ViperList));

    vec->length = 0;
    vec->capacity = cap > 0 ? cap : BITVEC_INITIAL_CAPACITY;
    vec->elem_type = VIPER_LIST_BITVEC;
    
    int64_t words = bitvec_words_needed(vec->capacity);
    vec->data.data_bitvec = (uint64_t*)calloc((size_t)words, sizeof(uint64_t));

    if (!vec->data.data_bitvec) {
        vp_panic("Failed to allocate bit vector data");
    }

    vp_arc_set_destructor(vec, vp_bitvec_destroy);

    return vec;
}

ViperList* vp_bitvec_create(void) {
    return vp_bitvec_create_with_capacity(BITVEC_INITIAL_CAPACITY);
}

/* OPTIMIZED: Create a bit vector with all elements set to the same value
 * This is critical for prime sieve: [1] * (n+1) creates a vector of n+1 true values
 */
ViperList* vp_bitvec_repeat(bool elem, int64_t count) {
    if (count <= 0) {
        return vp_bitvec_create();
    }

    ViperList* vec = vp_bitvec_create_with_capacity(count);
    vec->length = count;
    
    int64_t words = bitvec_words_needed(count);
    
    if (elem) {
        /* Set all bits to 1 */
        /* First, set all full words to all 1s */
        for (int64_t i = 0; i < words; i++) {
            vec->data.data_bitvec[i] = UINT64_MAX;
        }
        
        /* Clear extra bits in the last word */
        int64_t extra_bits = words * BITS_PER_WORD - count;
        if (extra_bits > 0) {
            uint64_t mask = (UINT64_MAX >> extra_bits);
            vec->data.data_bitvec[words - 1] &= mask;
        }
    } else {
        /* All bits already 0 from calloc */
    }

    return vec;
}

void vp_bitvec_free(ViperList* vec) {
    if (!vec) return;
    vp_arc_release(vec);
}

/* OPTIMIZED: Inline version in header - this is fallback for non-inlined calls */
/* vp_bitvec_append - defined inline in viper_stdlib.h */

void vp_bitvec_insert(ViperList* vec, int64_t index, bool value) {
    if (!vec) {
        vp_panic("Cannot insert into NULL bit vector");
        return;
    }

    if (index < 0 || index > vec->length) {
        vp_panic("Bit vector index out of range");
        return;
    }

    if (vec->length >= vec->capacity) {
        vp_bitvec_grow(vec);
    }

    /* Shift bits to the right, starting from the end */
    for (int64_t i = vec->length; i > index; i--) {
        bool bit = vp_bitvec_get_inl(vec, i - 1);
        int64_t word_idx = bitvec_word_index(i);
        uint64_t mask = bitvec_bit_mask(i);
        
        if (bit) {
            vec->data.data_bitvec[word_idx] |= mask;
        } else {
            vec->data.data_bitvec[word_idx] &= ~mask;
        }
    }

    /* Set the new bit */
    int64_t word_idx = bitvec_word_index(index);
    uint64_t mask = bitvec_bit_mask(index);
    
    if (value) {
        vec->data.data_bitvec[word_idx] |= mask;
    } else {
        vec->data.data_bitvec[word_idx] &= ~mask;
    }
    
    vec->length++;
}

bool vp_bitvec_remove(ViperList* vec, int64_t index) {
    if (!vec) {
        vp_panic("Cannot remove from NULL bit vector");
        return false;
    }

    if (index < 0 || index >= vec->length) {
        vp_panic("Bit vector index out of range");
        return false;
    }

    bool value = vp_bitvec_get_inl(vec, index);

    /* Shift bits to the left */
    for (int64_t i = index; i < vec->length - 1; i++) {
        bool bit = vp_bitvec_get_inl(vec, i + 1);
        int64_t word_idx = bitvec_word_index(i);
        uint64_t mask = bitvec_bit_mask(i);
        
        if (bit) {
            vec->data.data_bitvec[word_idx] |= mask;
        } else {
            vec->data.data_bitvec[word_idx] &= ~mask;
        }
    }

    /* Clear the last bit */
    vec->length--;
    int64_t word_idx = bitvec_word_index(vec->length);
    uint64_t mask = bitvec_bit_mask(vec->length);
    vec->data.data_bitvec[word_idx] &= ~mask;
    
    return value;
}

bool vp_bitvec_pop(ViperList* vec) {
    if (!vec) {
        vp_panic("Cannot pop from NULL bit vector");
        return false;
    }

    if (vec->length == 0) {
        vp_panic("Cannot pop from empty bit vector");
        return false;
    }

    vec->length--;
    return vp_bitvec_get_inl(vec, vec->length);
}

void vp_bitvec_clear(ViperList* vec) {
    if (!vec) return;
    
    int64_t words = bitvec_words_needed(vec->capacity);
    memset(vec->data.data_bitvec, 0, (size_t)words * sizeof(uint64_t));
    vec->length = 0;
}

/* Exported functions for AOT linking - call inline versions */
bool vp_bitvec_get(ViperList* vec, int64_t index) {
    return vp_bitvec_get_inl(vec, index);
}

void vp_bitvec_set(ViperList* vec, int64_t index, bool value) {
    vp_bitvec_set_inl(vec, index, value);
}

void vp_bitvec_append(ViperList* vec, bool value) {
    vp_bitvec_append_inl(vec, value);
}

/* Unchecked versions for hot loops - no bounds checking */
bool vp_bitvec_get_unchecked(ViperList* vec, int64_t index) {
    return vp_bitvec_get_unchecked_inl(vec, index);
}

void vp_bitvec_set_unchecked(ViperList* vec, int64_t index, bool value) {
    vp_bitvec_set_unchecked_inl(vec, index, value);
}

bool vp_bitvec_contains(ViperList* vec, bool value) {
    if (!vec || vec->length == 0) return false;
    
    int64_t words = bitvec_words_needed(vec->length);
    
    if (value) {
        /* Check if any bit is set */
        for (int64_t i = 0; i < words; i++) {
            if (vec->data.data_bitvec[i] != 0) {
                return true;
            }
        }
        return false;
    } else {
        /* Check if any bit is clear */
        for (int64_t i = 0; i < words - 1; i++) {
            if (vec->data.data_bitvec[i] != UINT64_MAX) {
                return true;
            }
        }
        /* Check last word with only relevant bits */
        int64_t extra_bits = words * BITS_PER_WORD - vec->length;
        uint64_t mask = (extra_bits > 0) ? (UINT64_MAX >> extra_bits) : UINT64_MAX;
        return (vec->data.data_bitvec[words - 1] & mask) != mask;
    }
}

ViperList* vp_bitvec_copy(ViperList* vec) {
    if (!vec) return NULL;
    
    ViperList* copy = vp_bitvec_create_with_capacity(vec->capacity);
    copy->length = vec->length;
    
    int64_t words = bitvec_words_needed(vec->capacity);
    memcpy(copy->data.data_bitvec, vec->data.data_bitvec, (size_t)words * sizeof(uint64_t));
    
    return copy;
}

ViperList* vp_bitvec_slice(ViperList* vec, int64_t start, int64_t end, int64_t step) {
    if (!vec) return vp_bitvec_create();

    int64_t len = vec->length;

    /* Handle negative indices */
    if (start < 0) start = len + start;
    if (end < 0) end = len + end;

    /* Clamp to valid range */
    if (start < 0) start = 0;
    if (end > len) end = len;
    if (start >= end) return vp_bitvec_create();

    ViperList* result = vp_bitvec_create();

    if (step == 0) step = 1;

    if (step > 0) {
        for (int64_t i = start; i < end; i += step) {
            vp_bitvec_append_inl(result, vp_bitvec_get_inl(vec, i));
        }
    } else {
        for (int64_t i = end - 1; i >= start; i += step) {
            vp_bitvec_append_inl(result, vp_bitvec_get_inl(vec, i));
        }
    }

    return result;
}

void vp_bitvec_print(ViperList* vec) {
    if (!vec) {
        printf("(null)");
        return;
    }
    
    printf("[");
    for (int64_t i = 0; i < vec->length; i++) {
        if (i > 0) printf(", ");
        printf("%s", vp_bitvec_get_inl(vec, i) ? "True" : "False");
    }
    printf("]");
}

int64_t vp_bitvec_len(ViperList* vec) {
    if (!vec) return 0;
    return vec->length;
}

/* ============================================ */
/* Extended Bit Vector Operations               */
/* ============================================ */

void vp_bitvec_extend(ViperList* vec, ViperList* other) {
    if (!vec || !other) return;
    if (other->elem_type != VIPER_LIST_BITVEC) {
        vp_panic("Cannot extend bit vector with non-bit-vector");
        return;
    }
    
    for (int64_t i = 0; i < other->length; i++) {
        vp_bitvec_append_inl(vec, vp_bitvec_get_inl(other, i));
    }
}

int64_t vp_bitvec_index(ViperList* vec, bool value) {
    if (!vec) return -1;
    
    for (int64_t i = 0; i < vec->length; i++) {
        if (vp_bitvec_get_inl(vec, i) == value) {
            return i;
        }
    }
    return -1;
}

int64_t vp_bitvec_count(ViperList* vec, bool value) {
    if (!vec || vec->length == 0) return 0;

    int64_t count = 0;
    int64_t words = bitvec_words_needed(vec->length);

    if (value) {
        /* Count set bits using SIMD-optimized population count */
        count = vp_bitvec_popcount_simd(vec->data.data_bitvec, words - 1);
        
        /* Last word: only count bits within valid range */
        int64_t extra_bits = words * BITS_PER_WORD - vec->length;
        uint64_t mask = (extra_bits > 0) ? (UINT64_MAX >> extra_bits) : UINT64_MAX;
        uint64_t word = vec->data.data_bitvec[words - 1] & mask;
        count += __builtin_popcountll(word);
    } else {
        /* Count clear bits */
        count = vec->length - vp_bitvec_count(vec, true);
    }

    return count;
}

void vp_bitvec_reverse(ViperList* vec) {
    if (!vec) return;
    
    int64_t left = 0;
    int64_t right = vec->length - 1;
    
    while (left < right) {
        bool left_val = vp_bitvec_get_inl(vec, left);
        bool right_val = vp_bitvec_get_inl(vec, right);
        vp_bitvec_set_inl(vec, left, right_val);
        vp_bitvec_set_inl(vec, right, left_val);
        left++;
        right--;
    }
}

ViperList* vp_bitvec_reversed(ViperList* vec) {
    if (!vec) return NULL;
    
    ViperList* result = vp_bitvec_copy(vec);
    vp_bitvec_reverse(result);
    return result;
}

ViperList* vp_bitvec_concat(ViperList* vec1, ViperList* vec2) {
    if (!vec1 || !vec2) return NULL;
    if (vec2->elem_type != VIPER_LIST_BITVEC) {
        vp_panic("Cannot concatenate non-bit-vector to bit vector");
        return NULL;
    }
    
    ViperList* result = vp_bitvec_copy(vec1);
    vp_bitvec_extend(result, vec2);
    return result;
}
