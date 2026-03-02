cat << 'INNER_EOF' >> runtime/src/runtime.c

/* String conversions */
#include <stdlib.h>
#include <math.h>

int64_t vp_str_to_i64(const char* str) {
    if (!str) return 0;
    return strtoll(str, NULL, 10);
}

double vp_str_to_f64(const char* str) {
    if (!str) return 0.0;
    if (strcmp(str, "inf") == 0 || strcmp(str, "+inf") == 0 || strcmp(str, "Infinity") == 0) return INFINITY;
    if (strcmp(str, "-inf") == 0 || strcmp(str, "-Infinity") == 0) return -INFINITY;
    if (strcmp(str, "nan") == 0 || strcmp(str, "NaN") == 0) return NAN;
    return strtod(str, NULL);
}
INNER_EOF

sed -i '/char\* vp_str_from_i64(int64_t val);/a int64_t vp_str_to_i64(const char* str);\ndouble vp_str_to_f64(const char* str);' runtime/include/runtime.h

