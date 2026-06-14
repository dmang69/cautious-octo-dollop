/*
 * IntentKernel — intentd implementation
 *
 * Responsibilities
 * ----------------
 *  1. Accept intent requests (scope + resource_id + optional TTL)
 *  2. Issue one-shot capability tokens
 *  3. Sign tokens with HMAC-SHA256 stub (placeholder for ML-DSA-87)
 *  4. Enforce TTL
 *  5. Log every issuance to stdout
 *
 * Signing stub
 * ------------
 * Production IntentKernel uses ML-DSA-87 (NIST FIPS 204, Dilithium 5).
 * Until liboqs is integrated this implementation uses HMAC-SHA256 over
 * the token header fields.  The first 32 bytes of the signature field
 * carry the HMAC; bytes 32-2423 are zeroed and reserved.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <time.h>

#include "intentd.h"
#include "sha256.h"

/* ------------------------------------------------------------------ */
/* Private signing key (stub — 32 bytes)                               */
/* Production: load from secure hardware / TPM.                        */
/* ------------------------------------------------------------------ */

static const uint8_t INTENTD_SIGNING_KEY[32] = {
    0x4b, 0x65, 0x79, 0x49, 0x6e, 0x74, 0x65, 0x6e,
    0x74, 0x4b, 0x65, 0x72, 0x6e, 0x65, 0x6c, 0x50,
    0x51, 0x43, 0x53, 0x69, 0x67, 0x6e, 0x69, 0x6e,
    0x67, 0x4b, 0x65, 0x79, 0x32, 0x30, 0x32, 0x35
};

/* ------------------------------------------------------------------ */
/* Token ID counter                                                    */
/* ------------------------------------------------------------------ */

static uint64_t g_token_counter = 0;
static int      g_initialized   = 0;

/* ------------------------------------------------------------------ */
/* Internal helpers                                                    */
/* ------------------------------------------------------------------ */

static uint64_t current_unix_ms(void)
{
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    return (uint64_t)ts.tv_sec * 1000ULL + (uint64_t)(ts.tv_nsec / 1000000);
}

/*
 * sign_token — compute HMAC-SHA256 over the fixed-size token header
 * (all fields except the signature bytes themselves) and store in
 * the first SHA256_DIGEST_SIZE bytes of tok->signature.
 */
static void sign_token(struct CapabilityToken *tok)
{
    /* Message = token_id || issued_at || expires_at || scope || resource_id
     * (28 bytes — everything before the signature field)               */
    const size_t header_len = offsetof(struct CapabilityToken, signature);

    uint8_t mac[SHA256_DIGEST_SIZE];
    hmac_sha256(INTENTD_SIGNING_KEY, sizeof(INTENTD_SIGNING_KEY),
                (const uint8_t *)tok, header_len,
                mac);

    memset(tok->signature, 0, ML_DSA_87_SIG_SIZE);
    memcpy(tok->signature, mac, SHA256_DIGEST_SIZE);
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

int intentd_init(void)
{
    g_token_counter = 1;
    g_initialized   = 1;
    fprintf(stdout, "[intentd] initialized — signing key loaded (HMAC-SHA256 stub)\n");
    return 0;
}

void intentd_shutdown(void)
{
    g_initialized = 0;
    fprintf(stdout, "[intentd] shutdown\n");
}

int intentd_issue(uint32_t scope, uint32_t resource_id,
                  uint64_t ttl_ms, struct CapabilityToken *out)
{
    if (!g_initialized || !out)
        return -1;

    if (ttl_ms == 0)
        ttl_ms = TOKEN_DEFAULT_TTL_MS;

    uint64_t now = current_unix_ms();

    out->token_id           = g_token_counter++;
    out->issued_at_unix_ms  = now;
    out->expires_at_unix_ms = now + ttl_ms;
    out->scope              = scope;
    out->resource_id        = resource_id;

    sign_token(out);

    fprintf(stdout,
        "[intentd] issued token #%llu  scope=0x%02x  resource=0x%08x  "
        "ttl=%llums  expires=%llu\n",
        (unsigned long long)out->token_id,
        (unsigned)out->scope,
        (unsigned)out->resource_id,
        (unsigned long long)ttl_ms,
        (unsigned long long)out->expires_at_unix_ms);

    return 0;
}

uint32_t intentd_ip_to_resource(const char *ip_str)
{
    if (!ip_str)
        return 0;

    unsigned int a = 0, b = 0, c = 0, d = 0;
    if (sscanf(ip_str, "%u.%u.%u.%u", &a, &b, &c, &d) != 4)
        return 0;
    if (a > 255 || b > 255 || c > 255 || d > 255)
        return 0;

    return (uint32_t)((a << 24) | (b << 16) | (c << 8) | d);
}

void intentd_token_print(const struct CapabilityToken *tok)
{
    printf("  token_id       : %llu\n",  (unsigned long long)tok->token_id);
    printf("  issued_at_ms   : %llu\n",  (unsigned long long)tok->issued_at_unix_ms);
    printf("  expires_at_ms  : %llu\n",  (unsigned long long)tok->expires_at_unix_ms);
    printf("  scope          : 0x%08x\n", tok->scope);
    printf("  resource_id    : 0x%08x\n", tok->resource_id);
    printf("  sig[0..7]      :");
    for (int i = 0; i < 8; i++) printf(" %02x", tok->signature[i]);
    printf(" ...\n");
}
