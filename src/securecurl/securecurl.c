/*
 * securecurl — IntentKernel end-to-end demo application
 *
 * Demonstrates the complete IntentKernel security model:
 *
 *   securecurl https://example.com
 *
 *   → libintentkernel: network_request("example.com")
 *   → intentd:         issue one-shot capability token (ML-DSA-87 signed)
 *   → capd:            validate token (signature + TTL + one-shot)
 *   → ip-descramblerd: analyze IP (threat DB + heuristics)
 *   → If ALLOW  → perform HTTPS/HTTP request
 *   → If BLOCK  → deny with reason, exit non-zero
 *
 * This is the first real IntentKernel application.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "../libintentkernel/libintentkernel.h"

static void print_banner(void)
{
    printf("╔══════════════════════════════════════════════════════╗\n");
    printf("║          securecurl — IntentKernel Demo              ║\n");
    printf("║  Capability-secure network requests (no ambient      ║\n");
    printf("║  authority, PQC-signed tokens, 5s TTL, one-shot)     ║\n");
    printf("╚══════════════════════════════════════════════════════╝\n\n");
}

static void print_usage(const char *argv0)
{
    fprintf(stderr, "Usage: %s <url> [url2 ...]\n", argv0);
    fprintf(stderr, "  Example: %s https://example.com\n", argv0);
}

int main(int argc, char **argv)
{
    print_banner();

    if (argc < 2) {
        print_usage(argv[0]);
        return 1;
    }

    /* Initialize all IntentKernel subsystems */
    printf("[securecurl] initializing IntentKernel subsystems...\n\n");
    if (ik_init() != IK_OK) {
        fprintf(stderr, "[securecurl] FATAL: IntentKernel init failed\n");
        return 1;
    }

    int overall_rc = 0;

    /* Process each URL argument */
    for (int i = 1; i < argc; i++) {
        printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        printf("[securecurl] REQUEST %d: %s\n", i, argv[i]);

        int rc = ik_network_request(argv[i]);

        printf("\n[securecurl] RESULT: ");
        switch (rc) {
        case IK_OK:
            printf("SUCCESS ✓\n");
            break;
        case IK_ERR_ACCESS_DENIED:
            printf("ACCESS DENIED ✗\n");
            overall_rc = 1;
            break;
        case IK_ERR_INVALID_URL:
            printf("INVALID URL ✗\n");
            overall_rc = 1;
            break;
        case IK_ERR_TOKEN_FAILED:
            printf("TOKEN ISSUANCE FAILED ✗\n");
            overall_rc = 1;
            break;
        default:
            printf("UNKNOWN ERROR %d ✗\n", rc);
            overall_rc = 1;
        }
    }

    printf("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    ik_shutdown();

    return overall_rc;
}
