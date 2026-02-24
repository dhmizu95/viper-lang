#include <stdio.h>
#include <stdint.h>

void vp_print_i64(int64_t val) {
    printf("%ld\n", val);
}

void vp_print_f64(double val) {
    printf("%.15g\n", val);
}

void vp_print_newline() {
    // newline handled by printf
}
