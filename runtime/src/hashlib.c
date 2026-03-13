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

#define MD5_F(x, y, z) (((x) & (y)) | (~(x) & (z)))
#define MD5_G(x, y, z) (((x) & (z)) | ((y) & ~(z)))
#define MD5_H(x, y, z) ((x) ^ (y) ^ (z))
#define MD5_I(x, y, z) ((y) ^ ((x) | ~(z)))
#define MD5_ROTATE_LEFT(x, n) (((x) << (n)) | ((x) >> (32 - (n))))
#define MD5_STEP(func, a, b, c, d, x, s, ac) \
    do { \
        (a) += func((b), (c), (d)) + (x) + (uint32_t)(ac); \
        (a) = MD5_ROTATE_LEFT((a), (s)); \
        (a) += (b); \
    } while (0)

static const uint32_t MD5_K[64] = {
    0xd76aa478U, 0xe8c7b756U, 0x242070dbU, 0xc1bdceeeU,
    0xf57c0fafU, 0x4787c62aU, 0xa8304613U, 0xfd469501U,
    0x698098d8U, 0x8b44f7afU, 0xffff5bb1U, 0x895cd7beU,
    0x6b901122U, 0xfd987193U, 0xa679438eU, 0x49b40821U,
    0xf61e2562U, 0xc040b340U, 0x265e5a51U, 0xe9b6c7aaU,
    0xd62f105dU, 0x02441453U, 0xd8a1e681U, 0xe7d3fbc8U,
    0x21e1cde6U, 0xc33707d6U, 0xf4d50d87U, 0x455a14edU,
    0xa9e3e905U, 0xfcefa3f8U, 0x676f02d9U, 0x8d2a4c8aU,
    0xfffa3942U, 0x8771f681U, 0x6d9d6122U, 0xfde5380cU,
    0xa4beea44U, 0x4bdecfa9U, 0xf6bb4b60U, 0xbebfbc70U,
    0x289b7ec6U, 0xeaa127faU, 0xd4ef3085U, 0x04881d05U,
    0xd9d4d039U, 0xe6db99e5U, 0x1fa27cf8U, 0xc4ac5665U,
    0xf4292244U, 0x432aff97U, 0xab9423a7U, 0xfc93a039U,
    0x655b59c3U, 0x8f0ccc92U, 0xffeff47dU, 0x85845dd1U,
    0x6fa87e4fU, 0xfe2ce6e0U, 0xa3014314U, 0x4e0811a1U,
    0xf7537e82U, 0xbd3af235U, 0x2ad7d2bbU, 0xeb86d391U
};

static void md5_transform(MD5_CTX* ctx, const uint8_t data[64]) {
    static const uint8_t MD5_S[64] = {
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22,
        5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
        4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23,
        6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21
    };
    uint32_t a = ctx->state[0];
    uint32_t b = ctx->state[1];
    uint32_t c = ctx->state[2];
    uint32_t d = ctx->state[3];
    uint32_t x[16];
    int i;

    for (i = 0; i < 16; i++) {
        x[i] = (uint32_t)data[i * 4] |
               ((uint32_t)data[i * 4 + 1] << 8) |
               ((uint32_t)data[i * 4 + 2] << 16) |
               ((uint32_t)data[i * 4 + 3] << 24);
    }

    for (i = 0; i < 64; i++) {
        uint32_t f;
        uint32_t g;
        uint32_t temp = d;

        if (i < 16) {
            f = MD5_F(b, c, d);
            g = (uint32_t)i;
        } else if (i < 32) {
            f = MD5_G(b, c, d);
            g = (uint32_t)((5 * i + 1) % 16);
        } else if (i < 48) {
            f = MD5_H(b, c, d);
            g = (uint32_t)((3 * i + 5) % 16);
        } else {
            f = MD5_I(b, c, d);
            g = (uint32_t)((7 * i) % 16);
        }

        d = c;
        c = b;
        b = b + MD5_ROTATE_LEFT(a + f + MD5_K[i] + x[g], MD5_S[i]);
        a = temp;
    }

    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
}

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
        finalcount[i] = (ctx->count >> (i * 8)) & 0xff;
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

#define ROTR64(x, n) (((x) >> (n)) | ((x) << (64 - (n))))
#define EP0_64(x) (ROTR64((x), 28) ^ ROTR64((x), 34) ^ ROTR64((x), 39))
#define EP1_64(x) (ROTR64((x), 14) ^ ROTR64((x), 18) ^ ROTR64((x), 41))
#define SIG0_64(x) (ROTR64((x), 1) ^ ROTR64((x), 8) ^ ((x) >> 7))
#define SIG1_64(x) (ROTR64((x), 19) ^ ROTR64((x), 61) ^ ((x) >> 6))

static const uint64_t SHA512_K[80] = {
    0x428a2f98d728ae22ULL, 0x7137449123ef65cdULL,
    0xb5c0fbcfec4d3b2fULL, 0xe9b5dba58189dbbcULL,
    0x3956c25bf348b538ULL, 0x59f111f1b605d019ULL,
    0x923f82a4af194f9bULL, 0xab1c5ed5da6d8118ULL,
    0xd807aa98a3030242ULL, 0x12835b0145706fbeULL,
    0x243185be4ee4b28cULL, 0x550c7dc3d5ffb4e2ULL,
    0x72be5d74f27b896fULL, 0x80deb1fe3b1696b1ULL,
    0x9bdc06a725c71235ULL, 0xc19bf174cf692694ULL,
    0xe49b69c19ef14ad2ULL, 0xefbe4786384f25e3ULL,
    0x0fc19dc68b8cd5b5ULL, 0x240ca1cc77ac9c65ULL,
    0x2de92c6f592b0275ULL, 0x4a7484aa6ea6e483ULL,
    0x5cb0a9dcbd41fbd4ULL, 0x76f988da831153b5ULL,
    0x983e5152ee66dfabULL, 0xa831c66d2db43210ULL,
    0xb00327c898fb213fULL, 0xbf597fc7beef0ee4ULL,
    0xc6e00bf33da88fc2ULL, 0xd5a79147930aa725ULL,
    0x06ca6351e003826fULL, 0x142929670a0e6e70ULL,
    0x27b70a8546d22ffcULL, 0x2e1b21385c26c926ULL,
    0x4d2c6dfc5ac42aedULL, 0x53380d139d95b3dfULL,
    0x650a73548baf63deULL, 0x766a0abb3c77b2a8ULL,
    0x81c2c92e47edaee6ULL, 0x92722c851482353bULL,
    0xa2bfe8a14cf10364ULL, 0xa81a664bbc423001ULL,
    0xc24b8b70d0f89791ULL, 0xc76c51a30654be30ULL,
    0xd192e819d6ef5218ULL, 0xd69906245565a910ULL,
    0xf40e35855771202aULL, 0x106aa07032bbd1b8ULL,
    0x19a4c116b8d2d0c8ULL, 0x1e376c085141ab53ULL,
    0x2748774cdf8eeb99ULL, 0x34b0bcb5e19b48a8ULL,
    0x391c0cb3c5c95a63ULL, 0x4ed8aa4ae3418acbULL,
    0x5b9cca4f7763e373ULL, 0x682e6ff3d6b2b8a3ULL,
    0x748f82ee5defb2fcULL, 0x78a5636f43172f60ULL,
    0x84c87814a1f0ab72ULL, 0x8cc702081a6439ecULL,
    0x90befffa23631e28ULL, 0xa4506cebde82bde9ULL,
    0xbef9a3f7b2c67915ULL, 0xc67178f2e372532bULL,
    0xca273eceea26619cULL, 0xd186b8c721c0c207ULL,
    0xeada7dd6cde0eb1eULL, 0xf57d4f7fee6ed178ULL,
    0x06f067aa72176fbaULL, 0x0a637dc5a2c898a6ULL,
    0x113f9804bef90daeULL, 0x1b710b35131c471bULL,
    0x28db77f523047d84ULL, 0x32caab7b40c72493ULL,
    0x3c9ebe0a15c9bebcULL, 0x431d67c49c100d4cULL,
    0x4cc5d4becb3e42b6ULL, 0x597f299cfc657e2aULL,
    0x5fcb6fab3ad6faecULL, 0x6c44198c4a475817ULL
};

static void sha512_transform(SHA512_CTX* ctx, const uint8_t data[128]) {
    uint64_t w[80];
    uint64_t a = ctx->state[0];
    uint64_t b = ctx->state[1];
    uint64_t c = ctx->state[2];
    uint64_t d = ctx->state[3];
    uint64_t e = ctx->state[4];
    uint64_t f = ctx->state[5];
    uint64_t g = ctx->state[6];
    uint64_t h = ctx->state[7];
    int i;

    for (i = 0; i < 16; i++) {
        w[i] = ((uint64_t)data[i * 8] << 56) |
               ((uint64_t)data[i * 8 + 1] << 48) |
               ((uint64_t)data[i * 8 + 2] << 40) |
               ((uint64_t)data[i * 8 + 3] << 32) |
               ((uint64_t)data[i * 8 + 4] << 24) |
               ((uint64_t)data[i * 8 + 5] << 16) |
               ((uint64_t)data[i * 8 + 6] << 8) |
               ((uint64_t)data[i * 8 + 7]);
    }
    for (i = 16; i < 80; i++) {
        w[i] = SIG1_64(w[i - 2]) + w[i - 7] + SIG0_64(w[i - 15]) + w[i - 16];
    }

    for (i = 0; i < 80; i++) {
        uint64_t t1 = h + EP1_64(e) + CH(e, f, g) + SHA512_K[i] + w[i];
        uint64_t t2 = EP0_64(a) + MAJ(a, b, c);
        h = g;
        g = f;
        f = e;
        e = d + t1;
        d = c;
        c = b;
        b = a;
        a = t1 + t2;
    }

    ctx->state[0] += a;
    ctx->state[1] += b;
    ctx->state[2] += c;
    ctx->state[3] += d;
    ctx->state[4] += e;
    ctx->state[5] += f;
    ctx->state[6] += g;
    ctx->state[7] += h;
}

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
    uint64_t bit_len = (uint64_t)len * 8U;

    ctx->count[0] += bit_len;
    if (ctx->count[0] < bit_len) {
        ctx->count[1]++;
    }
    
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
