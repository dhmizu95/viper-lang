/**
 * Viper Runtime - OS Module
 * POSIX wrappers for filesystem and OS operations
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <errno.h>
#include <limits.h>
#include <pwd.h>
#include "viper_stdlib.h"

/**
 * Get current working directory
 * Returns: Path string (caller must free with vp_arc_release)
 */
char* vp_os_getcwd(void) {
    char* buffer = (char*)vp_arc_alloc(PATH_MAX);
    if (!buffer) return NULL;
    
    if (getcwd(buffer, PATH_MAX) == NULL) {
        vp_arc_release(buffer);
        return NULL;
    }
    
    return buffer;
}

/**
 * Change current working directory
 * @param path Directory path
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_chdir(const char* path) {
    if (!path) return -1;
    return chdir(path);
}

/**
 * List directory contents
 * @param path Directory path
 * Returns: ViperList* of filename strings
 */
ViperList* vp_os_listdir(const char* path) {
    ViperList* result = vp_list_create();
    DIR* dir;
    struct dirent* entry;
    
    if (!path) {
        path = ".";
    }
    
    dir = opendir(path);
    if (!dir) {
        return result; /* Return empty list on error */
    }
    
    while ((entry = readdir(dir)) != NULL) {
        /* Skip . and .. */
        if (strcmp(entry->d_name, ".") == 0 || 
            strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        
        /* For now, we just count entries
         * Full implementation would create Viper strings */
        vp_list_append(result, 1);
    }
    
    closedir(dir);
    return result;
}

/**
 * Join path components
 * @param a First path component
 * @param b Second path component
 * Returns: Joined path string (caller must free)
 */
char* vp_os_path_join(const char* a, const char* b) {
    if (!a) a = "";
    if (!b) b = "";
    
    size_t len_a = strlen(a);
    size_t len_b = strlen(b);
    size_t total = len_a + len_b + 2; /* +1 for /, +1 for \0 */
    
    /* Check if a already ends with / */
    int needs_slash = (len_a > 0 && a[len_a - 1] != '/');
    
    char* result = (char*)vp_arc_alloc(total + (needs_slash ? 1 : 0));
    if (!result) return NULL;
    
    strcpy(result, a);
    if (needs_slash && len_b > 0) {
        strcat(result, "/");
    }
    strcat(result, b);
    
    return result;
}

/**
 * Get environment variable
 * @param name Variable name
 * Returns: Value string or NULL if not found
 */
const char* vp_os_getenv(const char* name) {
    if (!name) return NULL;
    return getenv(name);
}

/**
 * Create directory
 * @param path Directory path
 * @param mode Permissions (e.g., 0755)
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_mkdir(const char* path, int64_t mode) {
    if (!path) return -1;
    
    if (mode == 0) {
        mode = 0755; /* Default permissions */
    }
    
    return mkdir(path, (mode_t)mode);
}

/**
 * Create directory recursively (mkdir -p)
 * @param path Directory path
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_makedirs(const char* path) {
    if (!path) return -1;
    
    char tmp[PATH_MAX];
    char* p = NULL;
    size_t len;
    
    snprintf(tmp, sizeof(tmp), "%s", path);
    len = strlen(tmp);
    
    /* Remove trailing slash */
    if (tmp[len - 1] == '/') {
        tmp[len - 1] = 0;
    }
    
    /* Create each directory component */
    for (p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = 0;
            if (mkdir(tmp, 0755) != 0 && errno != EEXIST) {
                return -1;
            }
            *p = '/';
        }
    }
    
    if (mkdir(tmp, 0755) != 0 && errno != EEXIST) {
        return -1;
    }
    
    return 0;
}

/**
 * Remove file or directory
 * @param path Path to remove
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_remove(const char* path) {
    if (!path) return -1;
    
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    
    if (S_ISDIR(st.st_mode)) {
        return rmdir(path);
    } else {
        return unlink(path);
    }
}

/**
 * Check if path exists
 * @param path File/directory path
 * Returns: 1 if exists, 0 if not
 */
int64_t vp_os_path_exists(const char* path) {
    if (!path) return 0;
    
    struct stat st;
    return (stat(path, &st) == 0) ? 1 : 0;
}

/**
 * Check if path is a file
 * @param path File path
 * Returns: 1 if file, 0 if not
 */
int64_t vp_os_path_isfile(const char* path) {
    if (!path) return 0;
    
    struct stat st;
    if (stat(path, &st) != 0) {
        return 0;
    }
    return S_ISREG(st.st_mode) ? 1 : 0;
}

/**
 * Check if path is a directory
 * @param path Directory path
 * Returns: 1 if directory, 0 if not
 */
int64_t vp_os_path_isdir(const char* path) {
    if (!path) return 0;
    
    struct stat st;
    if (stat(path, &st) != 0) {
        return 0;
    }
    return S_ISDIR(st.st_mode) ? 1 : 0;
}

/**
 * Get file size in bytes
 * @param path File path
 * Returns: Size in bytes, or -1 on error
 */
int64_t vp_os_path_getsize(const char* path) {
    if (!path) return -1;
    
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    
    return (int64_t)st.st_size;
}

/**
 * Get absolute path
 * @param path Relative or absolute path
 * Returns: Absolute path string (caller must free)
 */
char* vp_os_path_abspath(const char* path) {
    if (!path) return NULL;
    
    char* resolved = realpath(path, NULL);
    if (!resolved) {
        /* If path doesn't exist, try to resolve parent */
        char cwd[PATH_MAX];
        if (getcwd(cwd, sizeof(cwd)) == NULL) {
            return NULL;
        }
        
        size_t len_cwd = strlen(cwd);
        size_t len_path = strlen(path);
        char* result = (char*)vp_arc_alloc(len_cwd + len_path + 2);
        if (!result) return NULL;
        
        strcpy(result, cwd);
        if (path[0] != '/') {
            strcat(result, "/");
        }
        strcat(result, path);
        return result;
    }
    
    char* result = (char*)vp_arc_alloc(strlen(resolved) + 1);
    if (result) {
        strcpy(result, resolved);
    }
    free(resolved); /* realpath uses malloc */
    
    return result;
}

/**
 * Get the basename of a path
 * @param path File path
 * Returns: Basename string (caller must free)
 */
char* vp_os_path_basename(const char* path) {
    if (!path) return NULL;
    
    const char* base = strrchr(path, '/');
    if (base) {
        base++; /* Skip the slash */
    } else {
        base = path;
    }
    
    char* result = (char*)vp_arc_alloc(strlen(base) + 1);
    if (result) {
        strcpy(result, base);
    }
    return result;
}

/**
 * Get the dirname of a path
 * @param path File path
 * Returns: Directory name string (caller must free)
 */
char* vp_os_path_dirname(const char* path) {
    if (!path) return NULL;
    
    char* result = vp_os_path_abspath(path);
    if (!result) return NULL;
    
    /* Find last slash */
    char* last_slash = strrchr(result, '/');
    if (last_slash && last_slash != result) {
        *last_slash = '\0';
    } else if (last_slash == result) {
        /* Root directory */
        result[1] = '\0';
    }
    
    return result;
}

/**
 * Rename/move file or directory
 * @param src Source path
 * @param dst Destination path
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_rename(const char* src, const char* dst) {
    if (!src || !dst) return -1;
    return rename(src, dst);
}

/**
 * Copy file content
 * @param src Source path
 * @param dst Destination path
 * Returns: 0 on success, -1 on error
 */
int64_t vp_os_copy(const char* src, const char* dst) {
    if (!src || !dst) return -1;
    
    FILE* fsrc = fopen(src, "rb");
    if (!fsrc) return -1;
    
    FILE* fdst = fopen(dst, "wb");
    if (!fdst) {
        fclose(fsrc);
        return -1;
    }
    
    char buffer[4096];
    size_t bytes;
    
    while ((bytes = fread(buffer, 1, sizeof(buffer), fsrc)) > 0) {
        if (fwrite(buffer, 1, bytes, fdst) != bytes) {
            fclose(fsrc);
            fclose(fdst);
            return -1;
        }
    }
    
    fclose(fsrc);
    fclose(fdst);
    return 0;
}

/**
 * Get user's home directory
 * Returns: Home directory path (caller must free)
 */
char* vp_os_get_home(void) {
    const char* home = getenv("HOME");
    if (home) {
        char* result = (char*)vp_arc_alloc(strlen(home) + 1);
        if (result) {
            strcpy(result, home);
        }
        return result;
    }
    
    /* Fallback to passwd */
    struct passwd* pw = getpwuid(getuid());
    if (pw && pw->pw_dir) {
        char* result = (char*)vp_arc_alloc(strlen(pw->pw_dir) + 1);
        if (result) {
            strcpy(result, pw->pw_dir);
        }
        return result;
    }
    
    return NULL;
}

/**
 * Get file stat info
 * Returns a struct-like data via output parameters
 */
int64_t vp_os_stat(
    const char* path,
    int64_t* size,
    int64_t* mode,
    int64_t* mtime,
    int64_t* is_dir,
    int64_t* is_file
) {
    if (!path) return -1;
    
    struct stat st;
    if (stat(path, &st) != 0) {
        return -1;
    }
    
    if (size) *size = (int64_t)st.st_size;
    if (mode) *mode = (int64_t)st.st_mode;
    if (mtime) *mtime = (int64_t)st.st_mtime;
    if (is_dir) *is_dir = S_ISDIR(st.st_mode) ? 1 : 0;
    if (is_file) *is_file = S_ISREG(st.st_mode) ? 1 : 0;
    
    return 0;
}
