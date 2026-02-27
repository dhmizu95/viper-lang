/**
 * Viper Runtime - System Module
 * Provides system-level functionality: argv, exit, getpid, version, platform
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/utsname.h>
#include "viper_stdlib.h"

/* Global storage for argv */
static char** vp_sys_argv_storage = NULL;
static int64_t vp_sys_argc = 0;

/**
 * Initialize sys module with argc/argv from main
 * Called at program startup
 */
void vp_sys_init(int argc, char** argv) {
    vp_sys_argc = argc;
    vp_sys_argv_storage = argv;
}

/**
 * Get command-line arguments as a Viper list
 * Returns: ViperList* containing strings
 */
ViperList* vp_sys_get_argv(void) {
    ViperList* argv_list = vp_list_create();
    
    if (vp_sys_argv_storage == NULL) {
        /* Simulated argv with just program name */
        argv_list = vp_list_create();
        return argv_list;
    }
    
    for (int i = 0; i < vp_sys_argc; i++) {
        /* For now, we just store the count - actual string access 
         * requires integration with Viper string system */
        (void)argv_list; /* Suppress unused warning */
    }
    
    return argv_list;
}

/**
 * Exit the program with a status code
 * @param code Exit code (0 = success)
 */
void vp_sys_exit(int64_t code) {
    exit((int)code);
}

/**
 * Get the current process ID
 * Returns: PID as i64
 */
int64_t vp_sys_getpid(void) {
    return (int64_t)getpid();
}

/**
 * Get the Viper version string
 * Returns: Version string (statically allocated)
 */
const char* vp_sys_get_version(void) {
    return "0.4.1";
}

/**
 * Get the platform identifier
 * Returns: Platform string (e.g., "linux", "darwin", "windows")
 */
const char* vp_sys_get_platform(void) {
#if defined(__linux__)
    return "linux";
#elif defined(__APPLE__) && defined(__MACH__)
    return "darwin";
#elif defined(_WIN32) || defined(_WIN64)
    return "windows";
#elif defined(__FreeBSD__)
    return "freebsd";
#elif defined(__unix__)
    return "unix";
#else
    return "unknown";
#endif
}

/**
 * Get detailed platform info (uname -s format)
 * Returns: System name string
 */
const char* vp_sys_get_sysname(void) {
    static struct utsname uts;
    static int initialized = 0;
    
    if (!initialized) {
        if (uname(&uts) == 0) {
            initialized = 1;
        } else {
            return "Unknown";
        }
    }
    
    return uts.sysname;
}

/**
 * Get machine architecture
 * Returns: Machine hardware name
 */
const char* vp_sys_get_machine(void) {
    static struct utsname uts;
    static int initialized = 0;
    
    if (!initialized) {
        if (uname(&uts) == 0) {
            initialized = 1;
        } else {
            return "unknown";
        }
    }
    
    return uts.machine;
}

/**
 * Get environment variable by name
 * @param name Environment variable name
 * Returns: Value string or NULL if not found
 */
const char* vp_sys_getenv(const char* name) {
    if (!name) return NULL;
    return getenv(name);
}

/**
 * Set environment variable
 * @param name Variable name
 * @param value Variable value
 * @param overwrite If 0, don't overwrite existing
 * Returns: 0 on success, -1 on error
 */
int64_t vp_sys_setenv(const char* name, const char* value, int64_t overwrite) {
    if (!name || !value) return -1;
    return setenv(name, value, (int)overwrite);
}

/**
 * Unset environment variable
 * @param name Variable name
 * Returns: 0 on success, -1 on error
 */
int64_t vp_sys_unsetenv(const char* name) {
    if (!name) return -1;
    return unsetenv(name);
}
