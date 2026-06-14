/*
 * IntentKernel — Capability Token Wire Format
 *
 * Binary-safe, fixed-size token transmitted between intentd, capd,
 * ip-descramblerd, and SDK callers.
 *
 * The signature field carries an ML-DSA-87 signature (2420 bytes per
 * NIST FIPS 204, zero-padded here to 2424 for 8-byte alignment).
 * In this reference implementation the first 32 bytes hold an
 * HMAC-SHA256 stub; the remaining 2392 bytes are reserved for the
 * actual PQC signature once liboqs is integrated.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_TOKEN_H
#define IK_TOKEN_H

#include <stdint.h>

/* ------------------------------------------------------------------ */
/* Constants                                                           */
/* ------------------------------------------------------------------ */

#define ML_DSA_87_SIG_SIZE   2424    /* ML-DSA-87 sig (FIPS 204) padded to 8-byte align */
#define TOKEN_DEFAULT_TTL_MS 5000ULL /* 5-second one-shot default */

/* Capability scopes */
#define SCOPE_NETWORK_REQUEST 0x01u
#define SCOPE_FILE_READ       0x02u
#define SCOPE_FILE_WRITE      0x03u
#define SCOPE_PROCESS_SPAWN   0x04u

/* ------------------------------------------------------------------ */
/* CapabilityToken — the atomic unit of authority in IntentKernel      */
/* ------------------------------------------------------------------ */

struct CapabilityToken {
    uint64_t token_id;                    /* Monotonically increasing   */
    uint64_t issued_at_unix_ms;           /* Unix epoch, milliseconds   */
    uint64_t expires_at_unix_ms;          /* issued_at + TTL            */
    uint32_t scope;                       /* SCOPE_* constant           */
    uint32_t resource_id;                 /* IPv4 addr or file handle   */
    uint8_t  signature[ML_DSA_87_SIG_SIZE]; /* ML-DSA-87 / stub        */
} __attribute__((packed));

/* ------------------------------------------------------------------ */
/* Verdict returned by ip-descramblerd                                 */
/* ------------------------------------------------------------------ */

typedef enum {
    VERDICT_ALLOW            = 0,
    VERDICT_BLOCK_MALICIOUS  = 1,
    VERDICT_BLOCK_SUSPICIOUS = 2,
    VERDICT_BLOCK_POLICY     = 3
} Verdict;

/* ------------------------------------------------------------------ */
/* IPC socket paths (Unix domain)                                      */
/* ------------------------------------------------------------------ */

#define INTENTD_SOCK_PATH       "/tmp/intentd.sock"
#define CAPD_SOCK_PATH          "/tmp/capd.sock"
#define IP_DESCRAMBLERD_SOCK    "/tmp/ip_descramblerd.sock"

#endif /* IK_TOKEN_H */
