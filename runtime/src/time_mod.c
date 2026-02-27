/**
 * Viper Runtime - Time Module
 * Time-related functions: wall clock, monotonic, sleep, perf counter
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include "viper_stdlib.h"

#if defined(__linux__) || defined(__APPLE__) || defined(__FreeBSD__)
    #define HAS_CLOCK_GETTIME 1
#else
    #define HAS_CLOCK_GETTIME 0
#endif

/**
 * Get wall clock time (Unix timestamp)
 * Returns: Seconds since epoch as f64
 */
double vp_time_time(void) {
    struct timespec ts;
    
#if HAS_CLOCK_GETTIME
    if (clock_gettime(CLOCK_REALTIME, &ts) == 0) {
        return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    }
#endif
    
    /* Fallback to time() */
    return (double)time(NULL);
}

/**
 * Get monotonic time (cannot go backwards)
 * Returns: Monotonic time in seconds as f64
 */
double vp_time_monotonic(void) {
    struct timespec ts;
    
#if HAS_CLOCK_GETTIME
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    }
#endif
    
    /* Fallback - not truly monotonic but works */
    return (double)time(NULL);
}

/**
 * Get high-resolution performance counter
 * Returns: Time in seconds as f64 (best resolution available)
 */
double vp_time_perf_counter(void) {
    struct timespec ts;
    
#if HAS_CLOCK_GETTIME
#if defined(_POSIX_MONOTONIC_RAW)
    /* Raw monotonic time - not adjusted by NTP */
    if (clock_gettime(CLOCK_MONOTONIC_RAW, &ts) == 0) {
        return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    }
#endif
    if (clock_gettime(CLOCK_MONOTONIC, &ts) == 0) {
        return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    }
#endif
    
    /* Fallback */
    return (double)time(NULL);
}

/**
 * Sleep for specified duration
 * @param seconds Sleep duration in seconds (f64)
 */
void vp_time_sleep(double seconds) {
    if (seconds <= 0) return;
    
    struct timespec ts;
    ts.tv_sec = (time_t)seconds;
    ts.tv_nsec = (long)((seconds - (double)ts.tv_sec) * 1e9);
    
#if HAS_CLOCK_GETTIME
    nanosleep(&ts, NULL);
#else
    /* Fallback to usleep (microseconds) */
    usleep((useconds_t)(seconds * 1e6));
#endif
}

/**
 * Get current local time as a struct-like data
 * Output parameters for year, month, day, hour, minute, second
 */
void vp_time_localtime(
    double timestamp,
    int64_t* year,
    int64_t* month,
    int64_t* day,
    int64_t* hour,
    int64_t* minute,
    int64_t* second
) {
    time_t t = (time_t)timestamp;
    struct tm* tm_info = localtime(&t);
    
    if (!tm_info) {
        if (year) *year = 1970;
        if (month) *month = 1;
        if (day) *day = 1;
        if (hour) *hour = 0;
        if (minute) *minute = 0;
        if (second) *second = 0;
        return;
    }
    
    if (year) *year = (int64_t)(tm_info->tm_year + 1900);
    if (month) *month = (int64_t)(tm_info->tm_mon + 1);
    if (day) *day = (int64_t)tm_info->tm_mday;
    if (hour) *hour = (int64_t)tm_info->tm_hour;
    if (minute) *minute = (int64_t)tm_info->tm_min;
    if (second) *second = (int64_t)tm_info->tm_sec;
}

/**
 * Get current UTC time as a struct-like data
 */
void vp_time_gmtime(
    double timestamp,
    int64_t* year,
    int64_t* month,
    int64_t* day,
    int64_t* hour,
    int64_t* minute,
    int64_t* second
) {
    time_t t = (time_t)timestamp;
    struct tm* tm_info = gmtime(&t);
    
    if (!tm_info) {
        if (year) *year = 1970;
        if (month) *month = 1;
        if (day) *day = 1;
        if (hour) *hour = 0;
        if (minute) *minute = 0;
        if (second) *second = 0;
        return;
    }
    
    if (year) *year = (int64_t)(tm_info->tm_year + 1900);
    if (month) *month = (int64_t)(tm_info->tm_mon + 1);
    if (day) *day = (int64_t)tm_info->tm_mday;
    if (hour) *hour = (int64_t)tm_info->tm_hour;
    if (minute) *minute = (int64_t)tm_info->tm_min;
    if (second) *second = (int64_t)tm_info->tm_sec;
}

/**
 * Format timestamp to string
 * @param timestamp Unix timestamp
 * @param format Format string (strftime format)
 * Returns: Formatted time string (caller must free)
 */
char* vp_time_strftime(double timestamp, const char* format) {
    if (!format) format = "%Y-%m-%d %H:%M:%S";
    
    time_t t = (time_t)timestamp;
    struct tm* tm_info = localtime(&t);
    
    if (!tm_info) {
        char* result = (char*)vp_arc_alloc(20);
        if (result) {
            strcpy(result, "1970-01-01 00:00:00");
        }
        return result;
    }
    
    char buffer[64];
    size_t len = strftime(buffer, sizeof(buffer), format, tm_info);
    
    if (len == 0) {
        char* result = (char*)vp_arc_alloc(1);
        if (result) {
            result[0] = '\0';
        }
        return result;
    }
    
    char* result = (char*)vp_arc_alloc(len + 1);
    if (result) {
        strcpy(result, buffer);
    }
    return result;
}

/**
 * Get timezone offset in seconds
 * Returns: Offset from UTC in seconds (positive = east of Greenwich)
 */
int64_t vp_time_timezone_offset(void) {
    time_t now = time(NULL);
    struct tm* tm_local = localtime(&now);
    struct tm* tm_utc = gmtime(&now);
    
    if (!tm_local || !tm_utc) {
        return 0;
    }
    
    /* Calculate difference in hours */
    time_t local_time = mktime(tm_local);
    time_t utc_time = mktime(tm_utc);
    
    return (int64_t)difftime(local_time, utc_time);
}

/**
 * Check if daylight saving time is in effect
 * Returns: 1 if DST, 0 if not
 */
int64_t vp_time_isdst(void) {
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    
    if (!tm_info) {
        return 0;
    }
    
    return tm_info->tm_isdst > 0 ? 1 : 0;
}

/**
 * Get number of days in a month
 * @param year Year
 * @param month Month (1-12)
 * Returns: Days in month (28-31)
 */
int64_t vp_time_days_in_month(int64_t year, int64_t month) {
    static const int days[] = {0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
    
    if (month < 1 || month > 12) {
        return 30; /* Default */
    }
    
    /* Check for leap year */
    if (month == 2) {
        int is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        return is_leap ? 29 : 28;
    }
    
    return days[month];
}
