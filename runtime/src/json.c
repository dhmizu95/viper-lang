/**
 * Viper Runtime - JSON Module
 * Recursive-descent JSON parser (no external dependencies)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <stdint.h>
#include "viper_stdlib.h"
#include "viper_types.h"

/* ============================================ */
/* JSON Parser State                            */
/* ============================================ */

typedef struct {
    const char* input;
    size_t pos;
    size_t len;
    char* error;
} JsonParser;

/* Forward declarations */
static ViperValue json_parse_value(JsonParser* p);
static void json_skip_whitespace(JsonParser* p);

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

static void json_skip_whitespace(JsonParser* p) {
    while (p->pos < p->len && isspace((unsigned char)p->input[p->pos])) {
        p->pos++;
    }
}

static char json_peek(JsonParser* p) {
    if (p->pos >= p->len) return '\0';
    return p->input[p->pos];
}

static char json_advance(JsonParser* p) {
    if (p->pos >= p->len) return '\0';
    return p->input[p->pos++];
}

static int json_match(JsonParser* p, const char* expected) {
    size_t len = strlen(expected);
    if (p->pos + len > p->len) return 0;
    
    for (size_t i = 0; i < len; i++) {
        if (p->input[p->pos + i] != expected[i]) return 0;
    }
    
    p->pos += len;
    return 1;
}

static char* json_strdup(const char* s, size_t len) {
    char* result = (char*)vp_arc_alloc(len + 1);
    if (result) {
        memcpy(result, s, len);
        result[len] = '\0';
    }
    return result;
}

/* ============================================ */
/* String Parsing                               */
/* ============================================ */

static char* json_parse_string(JsonParser* p) {
    if (json_peek(p) != '"') {
        p->error = "Expected '\"' at start of string";
        return NULL;
    }
    
    json_advance(p); /* Skip opening quote */
    
    size_t start = p->pos;
    size_t len = 0;
    
    /* First pass: calculate length */
    while (p->pos < p->len && p->input[p->pos] != '"') {
        if (p->input[p->pos] == '\\' && p->pos + 1 < p->len) {
            p->pos += 2;
            len++;
        } else {
            p->pos++;
            len++;
        }
    }
    
    if (p->pos >= p->len || p->input[p->pos] != '"') {
        p->error = "Unterminated string";
        return NULL;
    }
    
    /* Second pass: copy with escape handling */
    char* result = (char*)vp_arc_alloc(len + 1);
    if (!result) {
        p->error = "Memory allocation failed";
        return NULL;
    }
    
    p->pos = start;
    size_t out = 0;
    
    while (p->pos < p->len && p->input[p->pos] != '"') {
        if (p->input[p->pos] == '\\' && p->pos + 1 < p->len) {
            p->pos++;
            char escaped = p->input[p->pos++];
            switch (escaped) {
                case '"': result[out++] = '"'; break;
                case '\\': result[out++] = '\\'; break;
                case '/': result[out++] = '/'; break;
                case 'b': result[out++] = '\b'; break;
                case 'f': result[out++] = '\f'; break;
                case 'n': result[out++] = '\n'; break;
                case 'r': result[out++] = '\r'; break;
                case 't': result[out++] = '\t'; break;
                case 'u':
                    /* Unicode escape - simplified handling */
                    if (p->pos + 4 <= p->len) {
                        /* Parse 4 hex digits */
                        unsigned int codepoint = 0;
                        for (int i = 0; i < 4; i++) {
                            char c = p->input[p->pos++];
                            codepoint <<= 4;
                            if (c >= '0' && c <= '9') codepoint |= (c - '0');
                            else if (c >= 'a' && c <= 'f') codepoint |= (c - 'a' + 10);
                            else if (c >= 'A' && c <= 'F') codepoint |= (c - 'A' + 10);
                        }
                        /* Simple ASCII range */
                        if (codepoint < 128) {
                            result[out++] = (char)codepoint;
                        } else {
                            result[out++] = '?';
                        }
                    }
                    break;
                default:
                    result[out++] = escaped;
            }
        } else {
            result[out++] = p->input[p->pos++];
        }
    }
    
    result[out] = '\0';
    json_advance(p); /* Skip closing quote */
    
    return result;
}

/* ============================================ */
/* Number Parsing                               */
/* ============================================ */

static ViperValue json_parse_number(JsonParser* p) {
    ViperValue value;
    value.type = VIPER_TYPE_I64;
    value.data.as_i64 = 0;
    
    size_t start = p->pos;
    int has_decimal = 0;
    
    if (p->input[p->pos] == '-') {
        p->pos++;
    }
    
    /* Integer part */
    if (p->pos < p->len && p->input[p->pos] == '0') {
        p->pos++;
    } else {
        while (p->pos < p->len && isdigit((unsigned char)p->input[p->pos])) {
            p->pos++;
        }
    }
    
    /* Decimal part */
    if (p->pos < p->len && p->input[p->pos] == '.') {
        has_decimal = 1;
        p->pos++;
        while (p->pos < p->len && isdigit((unsigned char)p->input[p->pos])) {
            p->pos++;
        }
    }
    
    /* Exponent part */
    if (p->pos < p->len && (p->input[p->pos] == 'e' || p->input[p->pos] == 'E')) {
        has_decimal = 1;
        p->pos++;
        if (p->pos < p->len && (p->input[p->pos] == '+' || p->input[p->pos] == '-')) {
            p->pos++;
        }
        while (p->pos < p->len && isdigit((unsigned char)p->input[p->pos])) {
            p->pos++;
        }
    }
    
    /* Parse the number */
    char* num_str = json_strdup(p->input + start, p->pos - start);
    if (has_decimal) {
        value.type = VIPER_TYPE_F64;
        value.data.as_f64 = strtod(num_str, NULL);
    } else {
        value.type = VIPER_TYPE_I64;
        value.data.as_i64 = strtoll(num_str, NULL, 10);
    }
    
    vp_arc_release(num_str);
    return value;
}

/* ============================================ */
/* Value Parsing                                */
/* ============================================ */

static ViperValue json_parse_value(JsonParser* p) {
    ViperValue value;
    value.type = VIPER_TYPE_NONE;
    value.data.as_i64 = 0;
    
    json_skip_whitespace(p);
    
    char c = json_peek(p);
    
    if (c == '"') {
        /* String */
        char* str = json_parse_string(p);
        if (str) {
            value.type = VIPER_TYPE_STR;
            value.data.as_str = vp_str_create(str);
            vp_arc_release(str);
        }
    } else if (c == '{') {
        /* Object - simplified: return NULL dict */
        json_advance(p);
        json_skip_whitespace(p);
        
        ViperDict* dict = vp_dict_create();
        value.type = VIPER_TYPE_DICT;
        value.data.as_dict = dict;
        
        if (json_peek(p) != '}') {
            while (1) {
                json_skip_whitespace(p);
                char* key = json_parse_string(p);
                if (!key) break;
                
                json_skip_whitespace(p);
                if (json_peek(p) != ':') {
                    p->error = "Expected ':' after key";
                    vp_arc_release(key);
                    break;
                }
                json_advance(p);
                
                ViperValue val = json_parse_value(p);
                if (val.type == VIPER_TYPE_STR) {
                    vp_dict_set_str_str(dict, key, val.data.as_str);
                } else if (val.type == VIPER_TYPE_I64) {
                    /* Need to convert key to proper format */
                }
                
                vp_arc_release(key);
                
                json_skip_whitespace(p);
                if (json_peek(p) == ',') {
                    json_advance(p);
                } else {
                    break;
                }
            }
        }
        
        json_skip_whitespace(p);
        if (json_peek(p) != '}') {
            p->error = "Expected '}' at end of object";
        }
        json_advance(p);
        
    } else if (c == '[') {
        /* Array - simplified: return empty list */
        json_advance(p);
        json_skip_whitespace(p);
        
        ViperList* list = vp_list_create();
        value.type = VIPER_TYPE_LIST;
        value.data.as_list = list;
        
        if (json_peek(p) != ']') {
            while (1) {
                ViperValue item = json_parse_value(p);
                if (item.type == VIPER_TYPE_I64) {
                    vp_list_append(list, item.data.as_i64);
                }
                
                json_skip_whitespace(p);
                if (json_peek(p) == ',') {
                    json_advance(p);
                } else {
                    break;
                }
            }
        }
        
        json_skip_whitespace(p);
        if (json_peek(p) != ']') {
            p->error = "Expected ']' at end of array";
        }
        json_advance(p);
        
    } else if (c == 't' && json_match(p, "true")) {
        value.type = VIPER_TYPE_BOOL;
        value.data.as_bool = true;
        
    } else if (c == 'f' && json_match(p, "false")) {
        value.type = VIPER_TYPE_BOOL;
        value.data.as_bool = false;
        
    } else if (c == 'n' && json_match(p, "null")) {
        value.type = VIPER_TYPE_NONE;
        
    } else if (c == '-' || isdigit((unsigned char)c)) {
        value = json_parse_number(p);
        
    } else {
        p->error = "Unexpected character in JSON";
    }
    
    return value;
}

/* ============================================ */
/* Public API                                   */
/* ============================================ */

/**
 * Parse JSON string into ViperDict
 * @param json_str JSON string
 * Returns: ViperDict* or NULL on error
 */
ViperDict* vp_json_loads(const char* json_str) {
    if (!json_str) return NULL;
    
    JsonParser parser = {
        .input = json_str,
        .pos = 0,
        .len = strlen(json_str),
        .error = NULL
    };
    
    ViperValue result = json_parse_value(&parser);
    
    if (parser.error) {
        return NULL;
    }
    
    if (result.type == VIPER_TYPE_DICT) {
        return (ViperDict*)result.data.as_dict;
    }
    
    return NULL;
}

/**
 * Convert ViperDict to JSON string
 * @param dict ViperDict to serialize
 * Returns: JSON string (caller must free with vp_arc_release)
 */
char* vp_json_dumps(ViperDict* dict) {
    if (!dict) {
        char* result = (char*)vp_arc_alloc(5);
        if (result) strcpy(result, "null");
        return result;
    }
    
    /* Simplified implementation - returns "{}" for now */
    char* result = (char*)vp_arc_alloc(3);
    if (result) {
        strcpy(result, "{}");
    }
    return result;
}

/**
 * Get last JSON parse error
 * Returns: Error message or NULL
 */
const char* vp_json_get_error(void) {
    /* Thread-local storage would be needed for proper implementation */
    static const char* last_error = NULL;
    return last_error;
}

/**
 * Parse JSON file
 * @param filename Path to JSON file
 * Returns: ViperDict* or NULL on error
 */
ViperDict* vp_json_load_file(const char* filename) {
    if (!filename) return NULL;
    
    FILE* f = fopen(filename, "r");
    if (!f) return NULL;
    
    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);
    
    char* buffer = (char*)malloc(size + 1);
    if (!buffer) {
        fclose(f);
        return NULL;
    }
    
    size_t read_size = fread(buffer, 1, size, f);
    buffer[read_size] = '\0';
    fclose(f);
    
    ViperDict* result = vp_json_loads(buffer);
    free(buffer);
    
    return result;
}

/**
 * Write ViperDict to JSON file
 * @param dict ViperDict to serialize
 * @param filename Output file path
 * Returns: 0 on success, -1 on error
 */
int64_t vp_json_dump_file(ViperDict* dict, const char* filename) {
    if (!filename) return -1;
    
    char* json = vp_json_dumps(dict);
    if (!json) return -1;
    
    FILE* f = fopen(filename, "w");
    if (!f) {
        vp_arc_release(json);
        return -1;
    }
    
    fprintf(f, "%s", json);
    fclose(f);
    vp_arc_release(json);
    
    return 0;
}
