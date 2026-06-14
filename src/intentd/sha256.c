/*
 * IntentKernel — SHA-256 / HMAC-SHA256 implementation (FIPS 180-4)
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#include <string.h>
#include "sha256.h"

/* ------------------------------------------------------------------ */
/* SHA-256 round constants (first 32 bits of cube roots of primes)    */
/* ------------------------------------------------------------------ */
static const uint32_t K[64] = {
    0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u,
    0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
    0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u,
    0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
    0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu,
    0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
    0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u,
    0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
    0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u,
    0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
    0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u,
    0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
    0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u,
    0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
    0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u,
    0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
};

#define ROTR32(x, n) (((x) >> (n)) | ((x) << (32 - (n))))

#define CH(x, y, z)  (((x) & (y)) ^ (~(x) & (z)))
#define MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define EP0(x)  (ROTR32(x,  2) ^ ROTR32(x, 13) ^ ROTR32(x, 22))
#define EP1(x)  (ROTR32(x,  6) ^ ROTR32(x, 11) ^ ROTR32(x, 25))
#define SIG0(x) (ROTR32(x,  7) ^ ROTR32(x, 18) ^ ((x) >>  3))
#define SIG1(x) (ROTR32(x, 17) ^ ROTR32(x, 19) ^ ((x) >> 10))

/* Initial hash values (first 32 bits of square roots of first 8 primes) */
static const uint32_t H0[8] = {
    0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
    0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
};

static void sha256_transform(uint32_t state[8], const uint8_t block[64])
{
    uint32_t w[64];
    uint32_t a, b, c, d, e, f, g, h;

    for (int i = 0; i < 16; i++) {
        w[i] = ((uint32_t)block[i*4  ] << 24)
             | ((uint32_t)block[i*4+1] << 16)
             | ((uint32_t)block[i*4+2] <<  8)
             | ((uint32_t)block[i*4+3]      );
    }
    for (int i = 16; i < 64; i++) {
        w[i] = SIG1(w[i-2]) + w[i-7] + SIG0(w[i-15]) + w[i-16];
    }

    a = state[0]; b = state[1]; c = state[2]; d = state[3];
    e = state[4]; f = state[5]; g = state[6]; h = state[7];

    for (int i = 0; i < 64; i++) {
        uint32_t t1 = h + EP1(e) + CH(e, f, g) + K[i] + w[i];
        uint32_t t2 = EP0(a) + MAJ(a, b, c);
        h = g; g = f; f = e; e = d + t1;
        d = c; c = b; b = a; a = t1 + t2;
    }

    state[0] += a; state[1] += b; state[2] += c; state[3] += d;
    state[4] += e; state[5] += f; state[6] += g; state[7] += h;
}

void sha256_init(SHA256_CTX *ctx)
{
    for (int i = 0; i < 8; i++) ctx->state[i] = H0[i];
    ctx->count   = 0;
    ctx->buf_len = 0;
    memset(ctx->buf, 0, SHA256_BLOCK_SIZE);
}

void sha256_update(SHA256_CTX *ctx, const uint8_t *data, size_t len)
{
    ctx->count += (uint64_t)len * 8;

    for (size_t i = 0; i < len; i++) {
        ctx->buf[ctx->buf_len++] = data[i];
        if (ctx->buf_len == SHA256_BLOCK_SIZE) {
            sha256_transform(ctx->state, ctx->buf);
            ctx->buf_len = 0;
        }
    }
}

void sha256_final(SHA256_CTX *ctx, uint8_t digest[SHA256_DIGEST_SIZE])
{
    size_t i = ctx->buf_len;
    ctx->buf[i++] = 0x80;

    if (i > 56) {
        while (i < 64) ctx->buf[i++] = 0;
        sha256_transform(ctx->state, ctx->buf);
        i = 0;
    }
    while (i < 56) ctx->buf[i++] = 0;

    /* Append bit-length as big-endian 64-bit */
    ctx->buf[56] = (uint8_t)(ctx->count >> 56);
    ctx->buf[57] = (uint8_t)(ctx->count >> 48);
    ctx->buf[58] = (uint8_t)(ctx->count >> 40);
    ctx->buf[59] = (uint8_t)(ctx->count >> 32);
    ctx->buf[60] = (uint8_t)(ctx->count >> 24);
    ctx->buf[61] = (uint8_t)(ctx->count >> 16);
    ctx->buf[62] = (uint8_t)(ctx->count >>  8);
    ctx->buf[63] = (uint8_t)(ctx->count      );
    sha256_transform(ctx->state, ctx->buf);

    for (int j = 0; j < 8; j++) {
        digest[j*4  ] = (uint8_t)(ctx->state[j] >> 24);
        digest[j*4+1] = (uint8_t)(ctx->state[j] >> 16);
        digest[j*4+2] = (uint8_t)(ctx->state[j] >>  8);
        digest[j*4+3] = (uint8_t)(ctx->state[j]      );
    }
    /* Scrub state */
    memset(ctx, 0, sizeof(*ctx));
}

void sha256(const uint8_t *data, size_t len, uint8_t digest[SHA256_DIGEST_SIZE])
{
    SHA256_CTX ctx;
    sha256_init(&ctx);
    sha256_update(&ctx, data, len);
    sha256_final(&ctx, digest);
}

/* ------------------------------------------------------------------ */
/* HMAC-SHA256 (RFC 2104)                                              */
/* ------------------------------------------------------------------ */
void hmac_sha256(const uint8_t *key, size_t key_len,
                 const uint8_t *msg, size_t msg_len,
                 uint8_t mac[SHA256_DIGEST_SIZE])
{
    uint8_t k[SHA256_BLOCK_SIZE];
    uint8_t ipad[SHA256_BLOCK_SIZE];
    uint8_t opad[SHA256_BLOCK_SIZE];
    uint8_t inner[SHA256_DIGEST_SIZE];
    SHA256_CTX ctx;

    memset(k, 0, SHA256_BLOCK_SIZE);
    if (key_len > SHA256_BLOCK_SIZE) {
        sha256(key, key_len, k);
    } else {
        memcpy(k, key, key_len);
    }

    for (int i = 0; i < SHA256_BLOCK_SIZE; i++) {
        ipad[i] = k[i] ^ 0x36u;
        opad[i] = k[i] ^ 0x5cu;
    }

    sha256_init(&ctx);
    sha256_update(&ctx, ipad, SHA256_BLOCK_SIZE);
    sha256_update(&ctx, msg, msg_len);
    sha256_final(&ctx, inner);

    sha256_init(&ctx);
    sha256_update(&ctx, opad, SHA256_BLOCK_SIZE);
    sha256_update(&ctx, inner, SHA256_DIGEST_SIZE);
    sha256_final(&ctx, mac);
}
