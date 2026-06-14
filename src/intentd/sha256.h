/*
 * IntentKernel — Minimal SHA-256 (FIPS 180-4)
 *
 * Used to produce the HMAC-SHA256 stub that stands in for ML-DSA-87
 * until the liboqs post-quantum library is integrated.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_SHA256_H
#define IK_SHA256_H

#include <stdint.h>
#include <stddef.h>

#define SHA256_DIGEST_SIZE 32
#define SHA256_BLOCK_SIZE  64

typedef struct {
    uint32_t state[8];
    uint64_t count;         /* total bits processed */
    uint8_t  buf[SHA256_BLOCK_SIZE];
    size_t   buf_len;
} SHA256_CTX;

void sha256_init(SHA256_CTX *ctx);
void sha256_update(SHA256_CTX *ctx, const uint8_t *data, size_t len);
void sha256_final(SHA256_CTX *ctx, uint8_t digest[SHA256_DIGEST_SIZE]);

/* One-shot convenience wrapper */
void sha256(const uint8_t *data, size_t len, uint8_t digest[SHA256_DIGEST_SIZE]);

/* HMAC-SHA256 */
void hmac_sha256(const uint8_t *key, size_t key_len,
                 const uint8_t *msg, size_t msg_len,
                 uint8_t mac[SHA256_DIGEST_SIZE]);

#endif /* IK_SHA256_H */
