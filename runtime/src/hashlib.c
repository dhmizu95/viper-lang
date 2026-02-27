/**
 * Viper Runtime - Hashlib Module
 * SHA-256, MD5, SHA-512 implementations (no external dependencies)
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <arpa/inet.h>
#include "viper_stdlib.h"

/* ============================================ */
/* SHA-256 Implementation                       */
/* ============================================ */

typedef struct {
    uint32_t state[8];
    uint64_t count;
    uint8_t buffer[64];
} SHA256_CTX;

static const uint32_t SHA256_K[64] = {
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
};

#define ROTR(x, n) (((x) >> (n)) | ((x) << (32 - (n))))
#define CH(x, y, z) (((x) & (y)) ^ (~(x) & (z)))
#define MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define EP0(x) (ROTR(x, 2) ^ ROTR(x, 13) ^ ROTR(x, 22))
#define EP1(x) (ROTR(x, 6) ^ ROTR(x, 11) ^ ROTR(x, 25))
#define SIG0(x) (ROTR(x, 7) ^ ROTR(x, 18) ^ ((x) >> 3))
#define SIG1(x) (ROTR(x, 17) ^ ROTR(x, 19) ^ ((x) >> 10))

static void sha256_transform(SHA256_CTX* ctx, const uint8_t data[64]) {
    uint32_t a, b, c, d, e, f, g, h, t1, t2, w[64];
    int i;
    
    for (i = 0; i < 16; i++) {
        w[i] = ((uint32_t)data[i*4] << 24) | ((uint32_t)data[i*4+1] << 16) |
               ((uint32_t)data[i*4+2] << 8) | ((uint32_t)data[i*4+3]);
    }
    for (i = 16; i < 64; i++) {
        w[i] = SIG1(w[i-2]) + w[i-7] + SIG0(w[i-15]) + w[i-16];
    }
    
    a = ctx->state[0]; b = ctx->state[1]; c = ctx->state[2]; d = ctx->state[3];
    e = ctx->state[4]; f = ctx->state[5]; g = ctx->state[6]; h = ctx->state[7];
    
    for (i = 0; i < 64; i++) {
        t1 = h + EP1(e) + CH(e, f, g) + SHA256_K[i] + w[i];
        t2 = EP0(a) + MAJ(a, b, c);
        h = g; g = f; f = e; e = d + t1;
        d = c; c = b; b = a; a = t1 + t2;
    }
    
    ctx->state[0] += a; ctx->state[1] += b; ctx->state[2] += c; ctx->state[3] += d;
    ctx->state[4] += e; ctx->state[5] += f; ctx->state[6] += g; ctx->state[7] += h;
}

static void sha256_init(SHA256_CTX* ctx) {
    ctx->state[0] = 0x6a09e667; ctx->state[1] = 0xbb67ae85;
    ctx->state[2] = 0x3c6ef372; ctx->state[3] = 0xa54ff53a;
    ctx->state[4] = 0x510e527f; ctx->state[5] = 0x9b05688c;
    ctx->state[6] = 0x1f83d9ab; ctx->state[7] = 0x5be0cd19;
    ctx->count = 0;
}

static void sha256_update(SHA256_CTX* ctx, const uint8_t* data, size_t len) {
    size_t i;
    size_t index = (ctx->count / 8) % 64;
    ctx->count += len * 8;
    
    size_t part_len = 64 - index;
    if (len >= part_len) {
        memcpy(&ctx->buffer[index], data, part_len);
        sha256_transform(ctx, ctx->buffer);
        
        for (i = part_len; i + 64 <= len; i += 64) {
            sha256_transform(ctx, &data[i]);
        }
        index = 0;
    } else {
        i = 0;
    }
    
    memcpy(&ctx->buffer[index], &data[i], len - i);
}

static void sha256_final(SHA256_CTX* ctx, uint8_t hash[32]) {
    uint8_t finalcount[8];
    int i;
    
    for (i = 0; i < 8; i++) {
        finalcount[i] = (ctx->count >> ((7 - i) * 8)) & 0xff;
    }
    
    size_t index = (ctx->count / 8) % 64;
    ctx->buffer[index++] = 0x80;
    
    if (index > 56) {
        while (index < 64) ctx->buffer[index++] = 0;
        sha256_transform(ctx, ctx->buffer);
        index = 0;
    }
    
    while (index < 56) ctx->buffer[index++] = 0;
    memcpy(&ctx->buffer[56], finalcount, 8);
    sha256_transform(ctx, ctx->buffer);
    
    for (i = 0; i < 8; i++) {
        hash[i*4] = (ctx->state[i] >> 24) & 0xff;
        hash[i*4+1] = (ctx->state[i] >> 16) & 0xff;
        hash[i*4+2] = (ctx->state[i] >> 8) & 0xff;
        hash[i*4+3] = ctx->state[i] & 0xff;
    }
}

/* ============================================ */
/* MD5 Implementation                           */
/* ============================================ */

typedef struct {
    uint32_t state[4];
    uint64_t count;
    uint8_t buffer[64];
} MD5_CTX;

static void md5_transform(MD5_CTX* ctx, const uint8_t data[64]);

static void md5_init(MD5_CTX* ctx) {
    ctx->state[0] = 0x67452301; ctx->state[1] = 0xefcdab89;
    ctx->state[2] = 0x98badcfe; ctx->state[3] = 0x10325476;
    ctx->count = 0;
}

static void md5_update(MD5_CTX* ctx, const uint8_t* data, size_t len) {
    size_t i;
    size_t index = (ctx->count / 8) % 64;
    ctx->count += len * 8;
    
    size_t part_len = 64 - index;
    if (len >= part_len) {
        memcpy(&ctx->buffer[index], data, part_len);
        md5_transform(ctx, ctx->buffer);
        
        for (i = part_len; i + 64 <= len; i += 64) {
            md5_transform(ctx, &data[i]);
        }
        index = 0;
    } else {
        i = 0;
    }
    
    memcpy(&ctx->buffer[index], &data[i], len - i);
}

static void md5_final(MD5_CTX* ctx, uint8_t hash[16]) {
    uint8_t finalcount[8];
    int i;
    
    for (i = 0; i < 8; i++) {
        finalcount[i] = (ctx->count >> ((7 - i) * 8)) & 0xff;
    }
    
    size_t index = (ctx->count / 8) % 64;
    ctx->buffer[index++] = 0x80;
    
    if (index > 56) {
        while (index < 64) ctx->buffer[index++] = 0;
        md5_transform(ctx, ctx->buffer);
        index = 0;
    }
    
    while (index < 56) ctx->buffer[index++] = 0;
    memcpy(&ctx->buffer[56], finalcount, 8);
    md5_transform(ctx, ctx->buffer);
    
    for (i = 0; i < 4; i++) {
        hash[i*4] = ctx->state[i] & 0xff;
        hash[i*4+1] = (ctx->state[i] >> 8) & 0xff;
        hash[i*4+2] = (ctx->state[i] >> 16) & 0xff;
        hash[i*4+3] = (ctx->state[i] >> 24) & 0xff;
    }
}

/* ============================================ */
/* SHA-512 Implementation                       */
/* ============================================ */

typedef struct {
    uint64_t state[8];
    uint64_t count[2];
    uint8_t buffer[128];
} SHA512_CTX;

static void sha512_transform(SHA512_CTX* ctx, const uint8_t data[128]);

static void sha512_init(SHA512_CTX* ctx) {
    ctx->state[0] = 0x6a09e667f3bcc908ULL; ctx->state[1] = 0xbb67ae8584caa73bULL;
    ctx->state[2] = 0x3c6ef372fe94f82bULL; ctx->state[3] = 0xa54ff53a5f1d36f1ULL;
    ctx->state[4] = 0x510e527fade682d1ULL; ctx->state[5] = 0x9b05688c2b3e6c1fULL;
    ctx->state[6] = 0x1f83d9abfb41bd6bULL; ctx->state[7] = 0x5be0cd19137e2179ULL;
    ctx->count[0] = 0; ctx->count[1] = 0;
}

static void sha512_update(SHA512_CTX* ctx, const uint8_t* data, size_t len) {
    size_t i;
    size_t index = (ctx->count[0] / 8) % 128;
    
    ctx->count[0] += len * 8;
    if (ctx->count[0] < len * 8) ctx->count[1]++;
    ctx->count[1] += len / 9223372036854775808ULL;
    
    size_t part_len = 128 - index;
    if (len >= part_len) {
        memcpy(&ctx->buffer[index], data, part_len);
        sha512_transform(ctx, ctx->buffer);
        
        for (i = part_len; i + 128 <= len; i += 128) {
            sha512_transform(ctx, &data[i]);
        }
        index = 0;
    } else {
        i = 0;
    }
    
    memcpy(&ctx->buffer[index], &data[i], len - i);
}

static void sha512_final(SHA512_CTX* ctx, uint8_t hash[64]) {
    uint8_t finalcount[16];
    int i;
    
    for (i = 0; i < 8; i++) {
        finalcount[i] = (ctx->count[1] >> ((7 - i) * 8)) & 0xff;
        finalcount[i + 8] = (ctx->count[0] >> ((7 - i) * 8)) & 0xff;
    }
    
    size_t index = (ctx->count[0] / 8) % 128;
    ctx->buffer[index++] = 0x80;
    
    if (index > 112) {
        while (index < 128) ctx->buffer[index++] = 0;
        sha512_transform(ctx, ctx->buffer);
        index = 0;
    }
    
    while (index < 112) ctx->buffer[index++] = 0;
    memcpy(&ctx->buffer[112], finalcount, 16);
    sha512_transform(ctx, ctx->buffer);
    
    for (i = 0; i < 8; i++) {
        hash[i*8] = (ctx->state[i] >> 56) & 0xff;
        hash[i*8+1] = (ctx->state[i] >> 48) & 0xff;
        hash[i*8+2] = (ctx->state[i] >> 40) & 0xff;
        hash[i*8+3] = (ctx->state[i] >> 32) & 0xff;
        hash[i*8+4] = (ctx->state[i] >> 24) & 0xff;
        hash[i*8+5] = (ctx->state[i] >> 16) & 0xff;
        hash[i*8+6] = (ctx->state[i] >> 8) & 0xff;
        hash[i*8+7] = ctx->state[i] & 0xff;
    }
}

/* ============================================ */
/* Public API - Hash Objects                    */
/* ============================================ */

typedef struct ViperHash {
    int algo;  /* 0=md5, 1=sha256, 2=sha512 */
    union {
        MD5_CTX md5;
        SHA256_CTX sha256;
        SHA512_CTX sha512;
    } ctx;
    int initialized;
} ViperHash;

ViperHash* vp_hashlib_new(const char* algo) {
    ViperHash* h = (ViperHash*)vp_arc_alloc(sizeof(ViperHash));
    if (!h) return NULL;
    
    if (strcmp(algo, "md5") == 0) {
        h->algo = 0;
        md5_init(&h->ctx.md5);
    } else if (strcmp(algo, "sha256") == 0) {
        h->algo = 1;
        sha256_init(&h->ctx.sha256);
    } else if (strcmp(algo, "sha512") == 0) {
        h->algo = 2;
        sha512_init(&h->ctx.sha512);
    } else {
        vp_arc_release(h);
        return NULL;
    }
    
    h->initialized = 1;
    return h;
}

void vp_hashlib_free(ViperHash* h) {
    if (!h) return;
    vp_arc_release(h);
}

void vp_hashlib_update(ViperHash* h, const char* data, int64_t len) {
    if (!h || !h->initialized || !data) return;
    
    if (h->algo == 0) {
        md5_update(&h->ctx.md5, (const uint8_t*)data, len);
    } else if (h->algo == 1) {
        sha256_update(&h->ctx.sha256, (const uint8_t*)data, len);
    } else if (h->algo == 2) {
        sha512_update(&h->ctx.sha512, (const uint8_t*)data, len);
    }
}

char* vp_hashlib_digest(ViperHash* h) {
    if (!h || !h->initialized) return NULL;
    
    uint8_t hash[64];
    int hash_len = 0;
    
    /* Copy context to avoid modifying original */
    if (h->algo == 0) {
        MD5_CTX ctx;
        memcpy(&ctx, &h->ctx.md5, sizeof(MD5_CTX));
        md5_final(&ctx, hash);
        hash_len = 16;
    } else if (h->algo == 1) {
        SHA256_CTX ctx;
        memcpy(&ctx, &h->ctx.sha256, sizeof(SHA256_CTX));
        sha256_final(&ctx, hash);
        hash_len = 32;
    } else if (h->algo == 2) {
        SHA512_CTX ctx;
        memcpy(&ctx, &h->ctx.sha512, sizeof(SHA512_CTX));
        sha512_final(&ctx, hash);
        hash_len = 64;
    }
    
    /* Convert to hex string */
    static const char hex[] = "0123456789abcdef";
    char* result = (char*)vp_arc_alloc(hash_len * 2 + 1);
    if (!result) return NULL;
    
    for (int i = 0; i < hash_len; i++) {
        result[i*2] = hex[(hash[i] >> 4) & 0xf];
        result[i*2+1] = hex[hash[i] & 0xf];
    }
    result[hash_len * 2] = '\0';
    
    return result;
}

char* vp_hashlib_hexdigest(ViperHash* h) {
    return vp_hashlib_digest(h);
}

/* ============================================ */
/* Convenience Functions                        */
/* ============================================ */

char* vp_hash_sha256(const char* data, int64_t len) {
    SHA256_CTX ctx;
    sha256_init(&ctx);
    sha256_update(&ctx, (const uint8_t*)data, len);
    
    uint8_t hash[32];
    sha256_final(&ctx, hash);
    
    static const char hex[] = "0123456789abcdef";
    char* result = (char*)vp_arc_alloc(65);
    if (!result) return NULL;
    
    for (int i = 0; i < 32; i++) {
        result[i*2] = hex[(hash[i] >> 4) & 0xf];
        result[i*2+1] = hex[hash[i] & 0xf];
    }
    result[64] = '\0';
    
    return result;
}

char* vp_hash_md5(const char* data, int64_t len) {
    MD5_CTX ctx;
    md5_init(&ctx);
    md5_update(&ctx, (const uint8_t*)data, len);
    
    uint8_t hash[16];
    md5_final(&ctx, hash);
    
    static const char hex[] = "0123456789abcdef";
    char* result = (char*)vp_arc_alloc(33);
    if (!result) return NULL;
    
    for (int i = 0; i < 16; i++) {
        result[i*2] = hex[(hash[i] >> 4) & 0xf];
        result[i*2+1] = hex[hash[i] & 0xf];
    }
    result[32] = '\0';
    
    return result;
}

char* vp_hash_sha512(const char* data, int64_t len) {
    SHA512_CTX ctx;
    sha512_init(&ctx);
    sha512_update(&ctx, (const uint8_t*)data, len);
    
    uint8_t hash[64];
    sha512_final(&ctx, hash);
    
    static const char hex[] = "0123456789abcdef";
    char* result = (char*)vp_arc_alloc(129);
    if (!result) return NULL;
    
    for (int i = 0; i < 64; i++) {
        result[i*2] = hex[(hash[i] >> 4) & 0xf];
        result[i*2+1] = hex[hash[i] & 0xf];
    }
    result[128] = '\0';
    
    return result;
}

/* ============================================ */
/* Hash constants                               */
/* ============================================ */

int64_t vp_hashlib_block_size_md5(void) { return 64; }
int64_t vp_hashlib_block_size_sha256(void) { return 64; }
int64_t vp_hashlib_block_size_sha512(void) { return 128; }

int64_t vp_hashlib_digest_size_md5(void) { return 16; }
int64_t vp_hashlib_digest_size_sha256(void) { return 32; }
int64_t vp_hashlib_digest_size_sha512(void) { return 64; }
