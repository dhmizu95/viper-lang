// String Operations Benchmark - C Implementation
// String concatenation and character scanning

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define REPEATS 400

int main() {
    const char* chunk = "alpha,beta,gamma,delta;";
    size_t chunk_len = strlen(chunk);
    size_t text_len = chunk_len * REPEATS;
    char* text = (char*)malloc(text_len + 1);
    char* cursor = text;

    for (int i = 0; i < REPEATS; i++) {
        memcpy(cursor, chunk, chunk_len);
        cursor += chunk_len;
    }
    text[text_len] = '\0';

    long count_a = 0;
    long count_comma = 0;
    long count_semicolon = 0;
    for (size_t i = 0; i < text_len; i++) {
        if (text[i] == 'a') {
            count_a++;
        } else if (text[i] == ',') {
            count_comma++;
        } else if (text[i] == ';') {
            count_semicolon++;
        }
    }

    long checksum = (long)text_len + count_a + count_comma + count_semicolon;
    printf("string operations checksum: %ld\n", checksum);
    free(text);
    return 0;
}
