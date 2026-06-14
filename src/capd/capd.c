/*
 * IntentKernel — capd implementation
 *
 * Validates capability tokens: signature, TTL, one-shot enforcement.
 *
 * One-shot tracking uses a fixed-size ring buffer of recently consumed
 * token IDs.  In production this would be a persistent store to survive
 * restarts, but for the reference implementation an in-memory set is
 * sufficient for tokens with a 5-second TTL.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <string.h>
#include <time.h>

#include "capd.h"
#include "../intentd/sha256.h"

/* ------------------------------------------------------------------ */
/* Shared verification key (must match intentd's signing key)          */
/* ------------------------------------------------------------------ */

static const uint8_t CAPD_VERIFY_KEY[32] = {
    0x4b, 0x65, 0x79, 0x49, 0x6e, 0x74, 0x65, 0x6e,
    0x74, 0x4b, 0x65, 0x72, 0x6e, 0x65, 0x6c, 0x50,
    0x51, 0x43, 0x53, 0x69, 0x67, 0x6e, 0x69, 0x6e,
    0x67, 0x4b, 0x65, 0x79, 0x32, 0x30, 0x32, 0x35
};

/* ------------------------------------------------------------------ */
/* One-shot token registry                                             */
/* ------------------------------------------------------------------ */

#define CONSUMED_RING_SIZE 4096

static uint64_t g_consumed[CONSUMED_RING_SIZE];
static int      g_ring_head   = 0;
static int      g_initialized = 0;

static int is_consumed(uint64_t token_id)
{
    for (int i = 0; i < CONSUMED_RING_SIZE; i++) {
        if (g_consumed[i] == token_id && token_id != 0)
            return 1;
    }
    return 0;
}

static void mark_consumed(uint64_t token_id)
{
    g_consumed[g_ring_head] = token_id;
    g_ring_head = (g_ring_head + 1) % CONSUMED_RING_SIZE;
}

/* ------------------------------------------------------------------ */
/* Constant-time byte comparison (prevents timing side-channels)       */
/* ------------------------------------------------------------------ */

static int ct_memcmp(const void *a, const void *b, size_t n)
{
    const volatile uint8_t *x = (const volatile uint8_t *)a;
    const volatile uint8_t *y = (const volatile uint8_t *)b;
    uint8_t diff = 0;
    for (size_t i = 0; i < n; i++)
        diff |= x[i] ^ y[i];
    return (int)diff;
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

int capd_init(void)
{
    memset(g_consumed, 0, sizeof(g_consumed));
    g_ring_head   = 0;
    g_initialized = 1;
    fprintf(stdout, "[capd] initialized — token registry ready\n");
    return 0;
}

void capd_shutdown(void)
{
    g_initialized = 0;
    fprintf(stdout, "[capd] shutdown\n");
}

int capd_validate(const struct CapabilityToken *token)
{
    if (!g_initialized || !token) {
        fprintf(stderr, "[capd] not initialized or null token\n");
        return -1;
    }

    /* 1. Expiry check */
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    uint64_t now_ms = (uint64_t)ts.tv_sec * 1000ULL
                    + (uint64_t)(ts.tv_nsec / 1000000);

    if (now_ms >= token->expires_at_unix_ms) {
        fprintf(stderr, "[capd] DENIED token #%llu — EXPIRED (now=%llu exp=%llu)\n",
                (unsigned long long)token->token_id,
                (unsigned long long)now_ms,
                (unsigned long long)token->expires_at_unix_ms);
        return -1;
    }

    /* 2. Replay check */
    if (is_consumed(token->token_id)) {
        fprintf(stderr, "[capd] DENIED token #%llu — REPLAYED\n",
                (unsigned long long)token->token_id);
        return -1;
    }

    /* 3. Signature verification */
    const size_t header_len = offsetof(struct CapabilityToken, signature);
    uint8_t expected_mac[SHA256_DIGEST_SIZE];
    hmac_sha256(CAPD_VERIFY_KEY, sizeof(CAPD_VERIFY_KEY),
                (const uint8_t *)token, header_len,
                expected_mac);

    if (ct_memcmp(expected_mac, token->signature, SHA256_DIGEST_SIZE) != 0) {
        fprintf(stderr, "[capd] DENIED token #%llu — BAD_SIG\n",
                (unsigned long long)token->token_id);
        return -1;
    }

    /* 4. Consume the token (one-shot) */
    mark_consumed(token->token_id);

    fprintf(stdout, "[capd] ALLOW  token #%llu  scope=0x%02x\n",
            (unsigned long long)token->token_id,
            (unsigned)token->scope);

    return (int)token->scope;
}
