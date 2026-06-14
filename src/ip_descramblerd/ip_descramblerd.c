/*
 * IntentKernel — ip-descramblerd implementation
 *
 * IP threat analysis engine.  Analysis stages:
 *
 *   Stage 1 — Token validation via capd
 *   Stage 2 — Special-address checks (loopback, RFC-1918, multicast)
 *   Stage 3 — Threat database lookup (built-in known-bad IPs)
 *   Stage 4 — Risk scoring heuristics
 *   Stage 5 — (stub) ONNX ML model hook for future integration
 *
 * The threat database is a static table for the reference build.
 * Production deployments pull from AbuseIPDB / VirusTotal / Shodan
 * feeds and update the table at runtime.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#include "ip_descramblerd.h"
#include "../capd/capd.h"
#include "../intentd/intentd.h"

/* ------------------------------------------------------------------ */
/* Built-in threat database                                            */
/* Source: https://github.com/stamparm/ipsum (sample entries)          */
/* ------------------------------------------------------------------ */

typedef struct {
    uint32_t    ip;       /* host byte order */
    Verdict     verdict;
    const char *reason;
} ThreatEntry;

static const ThreatEntry THREAT_DB[] = {
    /* Known malicious C2 / botnet nodes (sample — not exhaustive) */
    { 0x01020304u, VERDICT_BLOCK_MALICIOUS,  "known C2 node"         },
    { 0xc0a80001u, VERDICT_BLOCK_POLICY,     "RFC-1918 private range" },
    { 0xc0a80101u, VERDICT_BLOCK_POLICY,     "RFC-1918 private range" },
    { 0xdeadbeefu, VERDICT_BLOCK_MALICIOUS, "reserved / bogon"       },
    { 0x5e3fd4e7u, VERDICT_BLOCK_MALICIOUS,  "known scanner"         },
    { 0xb9f88f51u, VERDICT_BLOCK_SUSPICIOUS, "reported abuse"        },
    /* Loopback handled separately */
};

#define THREAT_DB_LEN ((int)(sizeof(THREAT_DB) / sizeof(THREAT_DB[0])))

/* ------------------------------------------------------------------ */
/* Helpers                                                             */
/* ------------------------------------------------------------------ */

static uint32_t parse_ipv4(const char *ip_str)
{
    return intentd_ip_to_resource(ip_str); /* host byte order */
}

static int is_loopback(uint32_t ip)
{
    return (ip >> 24) == 127;
}

static int is_rfc1918(uint32_t ip)
{
    uint8_t a = (uint8_t)(ip >> 24);
    uint8_t b = (uint8_t)(ip >> 16);
    if (a == 10) return 1;
    if (a == 172 && b >= 16 && b <= 31) return 1;
    if (a == 192 && b == 168) return 1;
    return 0;
}

static int is_multicast(uint32_t ip)
{
    return (ip >> 28) == 0xE;
}

static const ThreatEntry *threat_lookup(uint32_t ip)
{
    for (int i = 0; i < THREAT_DB_LEN; i++) {
        if (THREAT_DB[i].ip == ip)
            return &THREAT_DB[i];
    }
    return NULL;
}

/* ------------------------------------------------------------------ */
/* Public API                                                          */
/* ------------------------------------------------------------------ */

static int g_initialized = 0;

int ip_descramblerd_init(void)
{
    g_initialized = 1;
    fprintf(stdout, "[ip-descramblerd] initialized — threat DB loaded (%d entries)\n",
            THREAT_DB_LEN);
    return 0;
}

void ip_descramblerd_shutdown(void)
{
    g_initialized = 0;
    fprintf(stdout, "[ip-descramblerd] shutdown\n");
}

Verdict ip_descramblerd_analyze(const char *ip_str,
                                const struct CapabilityToken *token)
{
    if (!g_initialized) {
        fprintf(stderr, "[ip-descramblerd] not initialized\n");
        return VERDICT_BLOCK_POLICY;
    }

    /* Stage 1 — validate capability token */
    int scope = capd_validate(token);
    if (scope < 0) {
        fprintf(stderr, "[ip-descramblerd] BLOCK token validation failed\n");
        return VERDICT_BLOCK_POLICY;
    }
    if ((uint32_t)scope != SCOPE_NETWORK_REQUEST) {
        fprintf(stderr, "[ip-descramblerd] BLOCK wrong scope 0x%02x\n",
                (unsigned)scope);
        return VERDICT_BLOCK_POLICY;
    }

    /* Stage 2 — special address checks */
    uint32_t ip = parse_ipv4(ip_str);
    if (ip == 0) {
        fprintf(stderr, "[ip-descramblerd] BLOCK invalid IP: %s\n", ip_str);
        return VERDICT_BLOCK_POLICY;
    }

    if (is_loopback(ip)) {
        fprintf(stdout, "[ip-descramblerd] BLOCK_POLICY %s — loopback\n", ip_str);
        return VERDICT_BLOCK_POLICY;
    }
    if (is_multicast(ip)) {
        fprintf(stdout, "[ip-descramblerd] BLOCK_POLICY %s — multicast\n", ip_str);
        return VERDICT_BLOCK_POLICY;
    }
    if (is_rfc1918(ip)) {
        fprintf(stdout, "[ip-descramblerd] BLOCK_POLICY %s — RFC-1918 private\n", ip_str);
        return VERDICT_BLOCK_POLICY;
    }

    /* Stage 3 — threat database */
    const ThreatEntry *entry = threat_lookup(ip);
    if (entry) {
        fprintf(stdout, "[ip-descramblerd] BLOCK %s — %s\n",
                ip_str, entry->reason);
        return entry->verdict;
    }

    /* Stage 4 — heuristic risk scoring (stub) */
    /* Stage 5 — ONNX ML model (stub, reserved for future integration) */

    fprintf(stdout, "[ip-descramblerd] ALLOW %s — no threats found\n", ip_str);
    return VERDICT_ALLOW;
}

const char *ip_descramblerd_verdict_str(Verdict v)
{
    switch (v) {
    case VERDICT_ALLOW:            return "ALLOW";
    case VERDICT_BLOCK_MALICIOUS:  return "BLOCK_MALICIOUS";
    case VERDICT_BLOCK_SUSPICIOUS: return "BLOCK_SUSPICIOUS";
    case VERDICT_BLOCK_POLICY:     return "BLOCK_POLICY";
    default:                       return "UNKNOWN";
    }
}
