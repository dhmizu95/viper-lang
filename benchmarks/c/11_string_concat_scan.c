// String Concat and Scan Benchmark - C Implementation

#include <stdio.h>
#include <stdlib.h>

int main() {
    const int n = 250;
    const size_t capacity = 4096;
    char* text = (char*)malloc(capacity);
    size_t len = 0;

    for (int i = 0; i < n; i++) {
        len += (size_t)snprintf(text + len, capacity - len, "item=%d;", i);
    }

    long digits = 0;
    long equals = 0;
    long semicolons = 0;
    for (size_t i = 0; i < len; i++) {
        char ch = text[i];
        if (ch >= '0' && ch <= '9') {
            digits++;
        } else if (ch == '=') {
            equals++;
        } else if (ch == ';') {
            semicolons++;
        }
    }

    printf("string concat checksum: %ld\n", (long)len + digits + equals + semicolons);
    free(text);
    return 0;
}
