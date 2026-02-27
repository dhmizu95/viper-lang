/**
 * Viper Runtime - Logging Module
 * Thread-safe logger with multiple levels
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <time.h>
#include <stdarg.h>
#include <pthread.h>
#include "viper_stdlib.h"

/* Forward declaration */
static char* json_strdup_local(const char* s, size_t len);

/* ============================================ */
/* Log Levels                                   */
/* ============================================ */

#define LOG_LEVEL_DEBUG     0
#define LOG_LEVEL_INFO      1
#define LOG_LEVEL_WARNING   2
#define LOG_LEVEL_ERROR     3
#define LOG_LEVEL_CRITICAL  4
#define LOG_LEVEL_NOTSET    5

/* ============================================ */
/* Logger Structure                             */
/* ============================================ */

typedef struct ViperLogger {
    char* name;
    int level;
    FILE* stream;
    char* format;
    pthread_mutex_t mutex;
    int64_t handler_count;
} ViperLogger;

/* Global default logger */
static ViperLogger* default_logger = NULL;
static pthread_mutex_t logger_init_mutex = PTHREAD_MUTEX_INITIALIZER;

/* ============================================ */
/* Logger Creation                              */
/* ============================================ */

ViperLogger* vp_logging_create_logger(const char* name, int level) {
    ViperLogger* logger = (ViperLogger*)vp_arc_alloc(sizeof(ViperLogger));
    if (!logger) return NULL;
    
    logger->name = json_strdup_local(name, strlen(name));
    logger->level = level;
    logger->stream = stderr;
    logger->format = "%(levelname)s - %(name)s - %(message)s";
    logger->handler_count = 0;
    pthread_mutex_init(&logger->mutex, NULL);
    
    return logger;
}

void vp_logging_logger_free(ViperLogger* logger) {
    if (!logger) return;
    
    if (logger->name) {
        vp_arc_release(logger->name);
    }
    pthread_mutex_destroy(&logger->mutex);
    vp_arc_release(logger);
}

/* ============================================ */
/* Level Functions                              */
/* ============================================ */

void vp_logging_set_level(ViperLogger* logger, int level) {
    if (!logger) return;
    logger->level = level;
}

int vp_logging_get_level(ViperLogger* logger) {
    return logger ? logger->level : LOG_LEVEL_NOTSET;
}

int64_t vp_logging_enabled_for(ViperLogger* logger, int level) {
    return logger && level >= logger->level ? 1 : 0;
}

/* ============================================ */
/* Format Helpers                               */
/* ============================================ */

static const char* level_to_string(int level) {
    switch (level) {
        case LOG_LEVEL_DEBUG:    return "DEBUG";
        case LOG_LEVEL_INFO:     return "INFO";
        case LOG_LEVEL_WARNING:  return "WARNING";
        case LOG_LEVEL_ERROR:    return "ERROR";
        case LOG_LEVEL_CRITICAL: return "CRITICAL";
        default:                 return "UNKNOWN";
    }
}

static void get_timestamp(char* buffer, size_t size) {
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    strftime(buffer, size, "%Y-%m-%d %H:%M:%S", tm_info);
}

/* ============================================ */
/* Core Logging Function                        */
/* ============================================ */

static void vp_logging_log(ViperLogger* logger, int level, const char* message) {
    if (!logger || level < logger->level) return;
    
    pthread_mutex_lock(&logger->mutex);
    
    char timestamp[32];
    get_timestamp(timestamp, sizeof(timestamp));
    
    const char* level_str = level_to_string(level);
    
    /* Simple format: LEVEL - NAME - MESSAGE */
    fprintf(logger->stream, "%s - %s - %s - %s\n", 
            timestamp, level_str, logger->name ? logger->name : "root", message);
    fflush(logger->stream);
    
    pthread_mutex_unlock(&logger->mutex);
}

/* ============================================ */
/* Level-specific Functions                     */
/* ============================================ */

void vp_logging_debug(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_DEBUG, message);
}

void vp_logging_info(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_INFO, message);
}

void vp_logging_warning(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_WARNING, message);
}

void vp_logging_error(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_ERROR, message);
}

void vp_logging_critical(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_CRITICAL, message);
}

void vp_logging_exception(ViperLogger* logger, const char* message) {
    vp_logging_log(logger, LOG_LEVEL_ERROR, message);
    /* Would add stack trace here */
}

/* ============================================ */
/* Printf-style Logging                         */
/* ============================================ */

void vp_logging_debug_f(ViperLogger* logger, const char* format, ...) {
    if (!logger || LOG_LEVEL_DEBUG < logger->level) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    vp_logging_debug(logger, buffer);
}

void vp_logging_info_f(ViperLogger* logger, const char* format, ...) {
    if (!logger || LOG_LEVEL_INFO < logger->level) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    vp_logging_info(logger, buffer);
}

void vp_logging_warning_f(ViperLogger* logger, const char* format, ...) {
    if (!logger || LOG_LEVEL_WARNING < logger->level) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    vp_logging_warning(logger, buffer);
}

void vp_logging_error_f(ViperLogger* logger, const char* format, ...) {
    if (!logger || LOG_LEVEL_ERROR < logger->level) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    vp_logging_error(logger, buffer);
}

void vp_logging_critical_f(ViperLogger* logger, const char* format, ...) {
    if (!logger || LOG_LEVEL_CRITICAL < logger->level) return;
    
    char buffer[1024];
    va_list args;
    va_start(args, format);
    vsnprintf(buffer, sizeof(buffer), format, args);
    va_end(args);
    
    vp_logging_critical(logger, buffer);
}

/* ============================================ */
/* Handler Management                           */
/* ============================================ */

void vp_logging_add_handler(ViperLogger* logger, FILE* stream) {
    if (!logger || !stream) return;
    
    pthread_mutex_lock(&logger->mutex);
    logger->stream = stream;
    logger->handler_count++;
    pthread_mutex_unlock(&logger->mutex);
}

void vp_logging_remove_handler(ViperLogger* logger) {
    if (!logger) return;
    
    pthread_mutex_lock(&logger->mutex);
    if (logger->handler_count > 0) {
        logger->handler_count--;
        if (logger->handler_count == 0) {
            logger->stream = stderr;
        }
    }
    pthread_mutex_unlock(&logger->mutex);
}

void vp_logging_set_stream(ViperLogger* logger, FILE* stream) {
    if (!logger) return;
    logger->stream = stream;
}

/* ============================================ */
/* Formatter                                    */
/* ============================================ */

void vp_logging_set_format(ViperLogger* logger, const char* format) {
    if (!logger || !format) return;
    
    pthread_mutex_lock(&logger->mutex);
    if (logger->format) {
        vp_arc_release(logger->format);
    }
    logger->format = json_strdup_local(format, strlen(format));
    pthread_mutex_unlock(&logger->mutex);
}

/* ============================================ */
/* Module-level Functions                       */
/* ============================================ */

ViperLogger* vp_logging_get_logger(const char* name) {
    if (!name || strcmp(name, "root") == 0 || strcmp(name, "") == 0) {
        if (!default_logger) {
            pthread_mutex_lock(&logger_init_mutex);
            if (!default_logger) {
                default_logger = vp_logging_create_logger("root", LOG_LEVEL_WARNING);
            }
            pthread_mutex_unlock(&logger_init_mutex);
        }
        return default_logger;
    }
    
    /* Create new logger for named loggers */
    return vp_logging_create_logger(name, LOG_LEVEL_WARNING);
}

void vp_logging_basic_config(int level, const char* format, FILE* stream) {
    pthread_mutex_lock(&logger_init_mutex);
    
    if (!default_logger) {
        default_logger = vp_logging_create_logger("root", level);
    } else {
        vp_logging_set_level(default_logger, level);
    }
    
    if (format) {
        vp_logging_set_format(default_logger, format);
    }
    
    if (stream) {
        vp_logging_set_stream(default_logger, stream);
    }
    
    pthread_mutex_unlock(&logger_init_mutex);
}

void vp_logging_cleanup(void) {
    pthread_mutex_lock(&logger_init_mutex);
    if (default_logger) {
        vp_logging_logger_free(default_logger);
        default_logger = NULL;
    }
    pthread_mutex_unlock(&logger_init_mutex);
}

/* ============================================ */
/* Level Constants                              */
/* ============================================ */

int64_t vp_logging_debug_level(void) { return LOG_LEVEL_DEBUG; }
int64_t vp_logging_info_level(void) { return LOG_LEVEL_INFO; }
int64_t vp_logging_warning_level(void) { return LOG_LEVEL_WARNING; }
int64_t vp_logging_error_level(void) { return LOG_LEVEL_ERROR; }
int64_t vp_logging_critical_level(void) { return LOG_LEVEL_CRITICAL; }
int64_t vp_logging_notset_level(void) { return LOG_LEVEL_NOTSET; }

/* ============================================ */
/* Filter (for future implementation)           */
/* ============================================ */

typedef struct ViperLogFilter {
    char* name;
    int64_t (*filter_fn)(const char* message);
} ViperLogFilter;

ViperLogFilter* vp_logging_create_filter(const char* name) {
    ViperLogFilter* filter = (ViperLogFilter*)vp_arc_alloc(sizeof(ViperLogFilter));
    if (!filter) return NULL;
    
    filter->name = json_strdup_local(name, strlen(name));
    filter->filter_fn = NULL;

    return filter;
}

void vp_logging_filter_free(ViperLogFilter* filter) {
    if (!filter) return;

    if (filter->name) {
        vp_arc_release(filter->name);
    }
    vp_arc_release(filter);
}

int64_t vp_logging_filter_call(ViperLogFilter* filter, const char* message) {
    if (!filter || !filter->filter_fn) return 1;
    return filter->filter_fn(message);
}

/* Helper function */
static char* json_strdup_local(const char* s, size_t len) {
    char* result = (char*)vp_arc_alloc(len + 1);
    if (result) {
        memcpy(result, s, len);
        result[len] = '\0';
    }
    return result;
}
