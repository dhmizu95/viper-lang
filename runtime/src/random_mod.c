/**
 * Viper Runtime - Random Module
 * PCG64 Pseudo-Random Number Generator
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include <math.h>
#include "viper_stdlib.h"

/* ============================================ */
/* PCG64 State                                  */
/* ============================================ */

typedef struct {
    uint64_t state;
    uint64_t inc;
} PCG64State;

static PCG64State pcg_state = {0, 0};
static int pcg_initialized = 0;

/* ============================================ */
/* PCG64 Implementation                         */
/* ============================================ */

static void pcg_init(uint64_t seed, uint64_t seq) {
    pcg_state.state = 0;
    pcg_state.inc = (seq << 1) | 1;
    
    /* Advance to initialize */
    uint64_t old_state = pcg_state.state;
    pcg_state.state = old_state * 6364136223846793005ULL + pcg_state.inc;
    
    pcg_state.state = old_state + seed;
    old_state = pcg_state.state;
    pcg_state.state = old_state * 6364136223846793005ULL + pcg_state.inc;
}

static uint64_t pcg_next(void) {
    uint64_t old_state = pcg_state.state;
    pcg_state.state = old_state * 6364136223846793005ULL + pcg_state.inc;
    
    /* XSH-RR output function */
    uint32_t xorshifted = ((old_state >> 18u) ^ old_state) >> 27u;
    uint32_t rot = old_state >> 59u;
    return (uint64_t)((xorshifted >> rot) | (xorshifted << ((-rot) & 31)));
}

static void pcg_ensure_initialized(void) {
    if (!pcg_initialized) {
        /* Seed from /dev/urandom or time */
        FILE* f = fopen("/dev/urandom", "rb");
        if (f) {
            uint64_t seed, seq;
            if (fread(&seed, sizeof(seed), 1, f) == 1 &&
                fread(&seq, sizeof(seq), 1, f) == 1) {
                pcg_init(seed, seq);
            } else {
                pcg_init((uint64_t)time(NULL), 1);
            }
            fclose(f);
        } else {
            pcg_init((uint64_t)time(NULL), 1);
        }
        pcg_initialized = 1;
    }
}

/* ============================================ */
/* Public API                                   */
/* ============================================ */

/**
 * Generate random float in [0, 1)
 * Returns: f64 in range [0, 1)
 */
double vp_random_random(void) {
    pcg_ensure_initialized();
    uint64_t r = pcg_next();
    return (double)(r >> 11) * (1.0 / 9007199254740992.0);
}

/**
 * Generate random integer in [a, b]
 * Returns: i64 in range [a, b]
 */
int64_t vp_random_randint(int64_t a, int64_t b) {
    pcg_ensure_initialized();
    
    if (a > b) {
        int64_t tmp = a;
        a = b;
        b = tmp;
    }
    
    uint64_t range = (uint64_t)(b - a + 1);
    uint64_t r = pcg_next();
    
    return a + (int64_t)(r % range);
}

/**
 * Seed the random number generator
 * @param seed Seed value
 */
void vp_random_seed(int64_t seed) {
    pcg_init((uint64_t)seed, 1);
    pcg_initialized = 1;
}

/**
 * Seed from secure source
 */
void vp_random_seed_secure(void) {
    FILE* f = fopen("/dev/urandom", "rb");
    if (f) {
        uint64_t seed, seq;
        if (fread(&seed, sizeof(seed), 1, f) == 1 &&
            fread(&seq, sizeof(seq), 1, f) == 1) {
            pcg_init(seed, seq);
        }
        fclose(f);
    }
    pcg_initialized = 1;
}

/**
 * Choose random element from list
 * @param list ViperList to choose from
 * Returns: Random element
 */
int64_t vp_random_choice(ViperList* list) {
    if (!list || vp_list_len(list) == 0) {
        return 0;
    }
    
    int64_t idx = vp_random_randint(0, vp_list_len(list) - 1);
    return vp_list_get(list, idx);
}

/**
 * Shuffle list in place
 * @param list ViperList to shuffle
 */
void vp_random_shuffle(ViperList* list) {
    if (!list || vp_list_len(list) <= 1) {
        return;
    }
    
    int64_t n = vp_list_len(list);
    
    /* Fisher-Yates shuffle */
    for (int64_t i = n - 1; i > 0; i--) {
        int64_t j = vp_random_randint(0, i);
        
        int64_t a = vp_list_get(list, i);
        int64_t b = vp_list_get(list, j);
        
        vp_list_set(list, i, b);
        vp_list_set(list, j, a);
    }
}

/**
 * Generate random float in [a, b)
 * Returns: f64 in range [a, b)
 */
double vp_random_uniform(double a, double b) {
    return a + vp_random_random() * (b - a);
}

/**
 * Generate random float from normal distribution
 * Uses Box-Muller transform
 * Returns: f64 with mean 0, stddev 1
 */
double vp_random_gauss(void) {
    static int has_spare = 0;
    static double spare;
    
    if (has_spare) {
        has_spare = 0;
        return spare;
    }
    
    double u, v, s;
    do {
        u = vp_random_random() * 2.0 - 1.0;
        v = vp_random_random() * 2.0 - 1.0;
        s = u * u + v * v;
    } while (s >= 1.0 || s == 0.0);
    
    s = sqrt(-2.0 * log(s) / s);
    
    spare = v * s;
    has_spare = 1;
    
    return u * s;
}

/**
 * Generate random float from normal distribution with given mean and stddev
 * Returns: f64 with given mean and stddev
 */
double vp_random_normal(double mean, double stddev) {
    return mean + stddev * vp_random_gauss();
}

/**
 * Generate random float from exponential distribution
 * Returns: f64 with given lambda
 */
double vp_random_exp(double lambd) {
    if (lambd <= 0) return 0;
    return -log(1.0 - vp_random_random()) / lambd;
}

/**
 * Generate random sample without replacement
 * @param list Source list
 * @param k Number of samples
 * Returns: New list with k samples
 */
ViperList* vp_random_sample(ViperList* list, int64_t k) {
    ViperList* result = vp_list_create();
    
    if (!list || k <= 0) {
        return result;
    }
    
    int64_t n = vp_list_len(list);
    if (k > n) {
        k = n;
    }
    
    /* Create index list */
    ViperList* indices = vp_list_create();
    for (int64_t i = 0; i < n; i++) {
        vp_list_append(indices, i);
    }
    
    /* Shuffle and take first k */
    vp_random_shuffle(indices);
    
    for (int64_t i = 0; i < k; i++) {
        int64_t idx = vp_list_get(indices, i);
        vp_list_append(result, vp_list_get(list, idx));
    }
    
    vp_list_free(indices);
    return result;
}

/**
 * Generate random boolean with given probability
 * @param probability Probability of true (0.0 to 1.0)
 * Returns: 1 if true, 0 if false
 */
int64_t vp_random_bool(double probability) {
    return vp_random_random() < probability ? 1 : 0;
}

/**
 * Get current state (for debugging)
 * Returns: State as i64 (truncated)
 */
int64_t vp_random_get_state(void) {
    return (int64_t)pcg_state.state;
}

/**
 * Set state directly (for reproducibility)
 * @param state State value
 */
void vp_random_set_state(int64_t state) {
    pcg_state.state = (uint64_t)state;
    pcg_initialized = 1;
}

/**
 * Generate random bytes
 * @param buffer Output buffer
 * @param length Number of bytes
 */
void vp_random_bytes(char* buffer, int64_t length) {
    if (!buffer || length <= 0) return;
    
    for (int64_t i = 0; i < length; i++) {
        buffer[i] = (char)(pcg_next() & 0xFF);
    }
}

/**
 * Check if generator is initialized
 * Returns: 1 if initialized, 0 otherwise
 */
int64_t vp_random_is_initialized(void) {
    return pcg_initialized ? 1 : 0;
}
