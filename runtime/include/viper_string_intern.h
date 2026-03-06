/**
 * Viper String Interning Support
 * 
 * Provides string deduplication through interning:
 * - Repeated strings share the same memory
 * - Fast equality comparison (pointer comparison)
 * - Reference counted for automatic cleanup
 * 
 * Usage:
 *   const char* s1 = vp_str_intern("hello");
 *   const char* s2 = vp_str_intern("hello");  // Returns same pointer as s1
 *   if (s1 == s2) { ... }  // Fast pointer comparison
 */

#ifndef VIPER_STRING_INTERN_H
#define VIPER_STRING_INTERN_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* String Interning Configuration               */
/* ============================================ */

/* Default hash table size for string interning */
#define VIPER_INTERN_TABLE_SIZE 1024

/* Maximum string length for interning (longer strings not interned) */
#define VIPER_INTERN_MAX_LENGTH 4096

/* ============================================ */
/* String Interning API                         */
/* ============================================ */

/**
 * Initialize the string interning system
 * Call once at program startup
 */
void vp_str_intern_init(void);

/**
 * Shutdown the string interning system
 * Call once at program cleanup
 */
void vp_str_intern_shutdown(void);

/**
 * Intern a string - returns a canonical copy
 * 
 * @param str The string to intern
 * @return Pointer to the interned string (same pointer for equal strings)
 * 
 * The returned string is reference counted and will be freed
 * automatically when the interning system shuts down.
 */
const char* vp_str_intern(const char* str);

/**
 * Intern a string with explicit length
 * 
 * @param str The string to intern
 * @param len The length of the string
 * @return Pointer to the interned string
 */
const char* vp_str_intern_len(const char* str, size_t len);

/**
 * Check if a string is already interned
 * 
 * @param str The string to check
 * @return The interned pointer if found, NULL otherwise
 */
const char* vp_str_intern_find(const char* str);

/**
 * Get the number of interned strings
 * 
 * @return Count of unique interned strings
 */
int64_t vp_str_intern_count(void);

/**
 * Get memory usage of string interning system
 * 
 * @return Total bytes used by interned strings
 */
int64_t vp_str_intern_memory_usage(void);

/**
 * Check if two pointers point to the same interned string
 * 
 * @param a First string pointer
 * @param b Second string pointer
 * @return true if both point to the same interned string
 */
bool vp_str_intern_eq(const char* a, const char* b);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_STRING_INTERN_H */
