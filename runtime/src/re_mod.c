/**
 * Viper Runtime - Regex Module
 * POSIX regex wrappers with simple LRU cache
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <regex.h>
#include <stdint.h>
#include "viper_stdlib.h"

/* ============================================ */
/* LRU Cache for Compiled Patterns              */
/* ============================================ */

#define REGEX_CACHE_SIZE 16

typedef struct {
    char* pattern;
    regex_t compiled;
    int flags;
    int used;
} CacheEntry;

static CacheEntry regex_cache[REGEX_CACHE_SIZE];
static int cache_lru_order[REGEX_CACHE_SIZE];
static int cache_initialized = 0;

static void regex_cache_init(void) {
    if (cache_initialized) return;
    
    for (int i = 0; i < REGEX_CACHE_SIZE; i++) {
        regex_cache[i].pattern = NULL;
        regex_cache[i].used = 0;
        cache_lru_order[i] = i;
    }
    cache_initialized = 1;
}

static int regex_cache_find(const char* pattern, int flags) {
    for (int i = 0; i < REGEX_CACHE_SIZE; i++) {
        if (regex_cache[i].used && 
            regex_cache[i].flags == flags &&
            strcmp(regex_cache[i].pattern, pattern) == 0) {
            return i;
        }
    }
    return -1;
}

static int regex_cache_add(const char* pattern, int flags, regex_t* compiled) {
    /* Find LRU entry (last in order) */
    int lru_idx = cache_lru_order[0];
    
    /* Free old entry if used */
    if (regex_cache[lru_idx].used) {
        regfree(&regex_cache[lru_idx].compiled);
        if (regex_cache[lru_idx].pattern) {
            vp_arc_release(regex_cache[lru_idx].pattern);
        }
    }
    
    /* Add new entry */
    regex_cache[lru_idx].pattern = vp_strdup_slice(pattern, strlen(pattern));
    regex_cache[lru_idx].flags = flags;
    regex_cache[lru_idx].compiled = *compiled;
    regex_cache[lru_idx].used = 1;
    
    /* Move to front of LRU order */
    int pos = 0;
    for (int i = 0; i < REGEX_CACHE_SIZE; i++) {
        if (cache_lru_order[i] == lru_idx) {
            pos = i;
            break;
        }
    }
    
    for (int i = pos; i > 0; i--) {
        cache_lru_order[i] = cache_lru_order[i - 1];
    }
    cache_lru_order[0] = lru_idx;
    
    return lru_idx;
}

static regex_t* regex_cache_get(int idx) {
    if (idx < 0 || idx >= REGEX_CACHE_SIZE) return NULL;
    return &regex_cache[idx].compiled;
}

/* ============================================ */
/* Pattern Object                               */
/* ============================================ */

typedef struct {
    regex_t compiled;
    char* pattern;
    int flags;
    int cache_idx;
} ViperPattern;

ViperPattern* vp_re_compile(const char* pattern, int64_t flags) {
    regex_cache_init();
    
    /* Check cache first */
    int cache_idx = regex_cache_find(pattern, (int)flags);
    if (cache_idx >= 0) {
        ViperPattern* p = (ViperPattern*)vp_arc_alloc(sizeof(ViperPattern));
        if (p) {
            p->pattern = vp_strdup_slice(pattern, strlen(pattern));
            p->flags = (int)flags;
            p->cache_idx = cache_idx;
            p->compiled = *regex_cache_get(cache_idx);
        }
        return p;
    }
    
    /* Compile new pattern */
    int cflags = REG_EXTENDED;
    
    if (flags & 0x01) {  /* IGNORECASE */
        cflags |= REG_ICASE;
    }
    if (flags & 0x02) {  /* MULTILINE */
        cflags |= REG_NEWLINE;
    }
    
    regex_t compiled;
    int ret = regcomp(&compiled, pattern, cflags);
    
    if (ret != 0) {
        return NULL;
    }
    
    /* Add to cache */
    cache_idx = regex_cache_add(pattern, (int)flags, &compiled);
    
    ViperPattern* p = (ViperPattern*)vp_arc_alloc(sizeof(ViperPattern));
    if (p) {
        p->pattern = vp_strdup_slice(pattern, strlen(pattern));
        p->flags = (int)flags;
        p->cache_idx = cache_idx;
        p->compiled = compiled;
    }
    
    return p;
}

void vp_re_pattern_free(ViperPattern* pattern) {
    if (!pattern) return;
    
    /* Don't free compiled regex - it's in cache */
    if (pattern->pattern) {
        vp_arc_release(pattern->pattern);
    }
    vp_arc_release(pattern);
}

/* ============================================ */
/* Match Functions                              */
/* ============================================ */

typedef struct {
    int64_t start;
    int64_t end;
    char* group;
} ViperMatch;

ViperMatch* vp_re_match(ViperPattern* pattern, const char* string, int64_t pos) {
    if (!pattern || !string) return NULL;
    
    regmatch_t matches[10];
    int ret = regexec(&pattern->compiled, string + pos, 10, matches, 0);
    
    if (ret != 0 || matches[0].rm_so == -1) {
        return NULL;
    }
    
    ViperMatch* m = (ViperMatch*)vp_arc_alloc(sizeof(ViperMatch));
    if (m) {
        m->start = pos + matches[0].rm_so;
        m->end = pos + matches[0].rm_eo;
        
        size_t len = matches[0].rm_eo - matches[0].rm_so;
        m->group = vp_strdup_slice(string + pos + matches[0].rm_so, len);
    }
    
    return m;
}

ViperMatch* vp_re_search(ViperPattern* pattern, const char* string, int64_t pos, int64_t endpos) {
    if (!pattern || !string) return NULL;
    
    if (endpos < 0) {
        endpos = strlen(string);
    }
    
    /* Search through string */
    for (size_t i = pos; i < (size_t)endpos; i++) {
        regmatch_t matches[10];
        int ret = regexec(&pattern->compiled, string + i, 10, matches, 0);
        
        if (ret == 0 && matches[0].rm_so != -1) {
            ViperMatch* m = (ViperMatch*)vp_arc_alloc(sizeof(ViperMatch));
            if (m) {
                m->start = i + matches[0].rm_so;
                m->end = i + matches[0].rm_eo;
                
                size_t len = matches[0].rm_eo - matches[0].rm_so;
                m->group = vp_strdup_slice(string + i + matches[0].rm_so, len);
            }
            return m;
        }
    }
    
    return NULL;
}

/* ============================================ */
/* Find All Matches                             */
/* ============================================ */

ViperList* vp_re_findall(ViperPattern* pattern, const char* string) {
    ViperList* results = vp_list_create();
    
    if (!pattern || !string) {
        return results;
    }
    
    size_t pos = 0;
    size_t len = strlen(string);
    
    while (pos < len) {
        regmatch_t matches[10];
        int ret = regexec(&pattern->compiled, string + pos, 10, matches, 0);
        
        if (ret != 0 || matches[0].rm_so == -1) {
            break;
        }
        
        /* Store match start position (simplified) */
        vp_list_append(results, (int64_t)(pos + matches[0].rm_so));
        
        /* Move past this match */
        pos += matches[0].rm_eo;
        
        /* Prevent infinite loop on zero-width matches */
        if (matches[0].rm_eo == 0) {
            pos++;
        }
    }
    
    return results;
}

/* ============================================ */
/* Split Function                               */
/* ============================================ */

ViperList* vp_re_split(ViperPattern* pattern, const char* string) {
    ViperList* results = vp_list_create();
    
    if (!pattern || !string) {
        return results;
    }
    
    size_t last_end = 0;
    size_t pos = 0;
    size_t len = strlen(string);
    
    while (pos < len) {
        regmatch_t matches[10];
        int ret = regexec(&pattern->compiled, string + pos, 10, matches, 0);
        
        if (ret != 0 || matches[0].rm_so == -1) {
            break;
        }
        
        /* Add substring before match */
        size_t match_start = pos + matches[0].rm_so;
        if (match_start > last_end) {
            /* Would add string segment here */
            vp_list_append(results, 1); /* Placeholder */
        }
        
        pos += matches[0].rm_eo;
        last_end = pos;
        
        if (matches[0].rm_eo == 0) {
            pos++;
        }
    }
    
    /* Add remaining string */
    if (last_end < len) {
        vp_list_append(results, 1); /* Placeholder */
    }
    
    return results;
}

/* ============================================ */
/* Substitute Function                          */
/* ============================================ */

char* vp_re_sub(ViperPattern* pattern, const char* repl, const char* string, int64_t count) {
    (void)count;
    if (!pattern || !repl || !string) {
        return NULL;
    }
    
    /* Simplified implementation - just return original string */
    size_t len = strlen(string);
    char* result = vp_strdup_slice(string, len);
    return result;
}

/* ============================================ */
/* Convenience Functions                        */
/* ============================================ */

int64_t vp_re_fullmatch(ViperPattern* pattern, const char* string) {
    if (!pattern || !string) return 0;
    
    regmatch_t matches[1];
    int ret = regexec(&pattern->compiled, string, 1, matches, 0);
    
    if (ret != 0 || matches[0].rm_so == -1) {
        return 0;
    }
    
    /* Check if entire string matched */
    size_t len = strlen(string);
    return (matches[0].rm_so == 0 && (size_t)matches[0].rm_eo == len) ? 1 : 0;
}

char* vp_re_escape(const char* string) {
    if (!string) return NULL;
    
    size_t len = strlen(string);
    /* Worst case: every char needs escaping */
    char* result = (char*)vp_arc_alloc(len * 2 + 1);
    if (!result) return NULL;
    
    size_t j = 0;
    for (size_t i = 0; i < len; i++) {
        char c = string[i];
        if (c == '.' || c == '^' || c == '$' || c == '*' || 
            c == '+' || c == '?' || c == '(' || c == ')' ||
            c == '[' || c == ']' || c == '{' || c == '}' ||
            c == '|' || c == '\\') {
            result[j++] = '\\';
        }
        result[j++] = c;
    }
    result[j] = '\0';
    
    return result;
}

/* ============================================ */
/* Error Handling                               */
/* ============================================ */

char* vp_re_get_error(int errcode) {
    char* buffer = (char*)vp_arc_alloc(256);
    if (!buffer) return NULL;
    
    regerror(errcode, NULL, buffer, 256);
    return buffer;
}

/* ============================================ */
/* Module Initialization                        */
/* ============================================ */

void vp_re_init(void) {
    regex_cache_init();
}

void vp_re_cleanup(void) {
    for (int i = 0; i < REGEX_CACHE_SIZE; i++) {
        if (regex_cache[i].used) {
            regfree(&regex_cache[i].compiled);
            if (regex_cache[i].pattern) {
                vp_arc_release(regex_cache[i].pattern);
            }
        }
    }
}

/* ============================================ */
/* Flag Constants                               */
/* ============================================ */

int64_t vp_re_ignorecase(void) { return 0x01; }
int64_t vp_re_multiline(void) { return 0x02; }
int64_t vp_re_dotall(void) { return 0x04; }
int64_t vp_re_verbose(void) { return 0x08; }
