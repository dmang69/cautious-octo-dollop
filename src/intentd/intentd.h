/*
 * IntentKernel — intentd public API
 *
 * intentd is the Intent Broker: the only entity allowed to issue
 * capability tokens.  An app (or the SDK) calls intentd_issue() to
 * request authority for exactly one privileged action.
 *
 * In-process API (used by securecurl demo and test harness).
 * The standalone daemon in intentd.c also exposes this over a Unix
 * domain socket using the same request/response structs.
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_INTENTD_H
#define IK_INTENTD_H

#include <stdint.h>
#include "token.h"

/* ------------------------------------------------------------------ */
/* Lifecycle                                                           */
/* ------------------------------------------------------------------ */

/*
 * intentd_init — load signing key and start the token counter.
 * Must be called once before intentd_issue().
 * Returns 0 on success, -1 on error.
 */
int intentd_init(void);

void intentd_shutdown(void);

/* ------------------------------------------------------------------ */
/* Token issuance                                                      */
/* ------------------------------------------------------------------ */

/*
 * intentd_issue — issue a one-shot capability token.
 *
 *   scope       : SCOPE_NETWORK_REQUEST etc.
 *   resource_id : IPv4 address as uint32 (network byte order), or
 *                 a file-handle index for file scopes.
 *   ttl_ms      : time-to-live in milliseconds (0 → TOKEN_DEFAULT_TTL_MS)
 *   out         : filled on success
 *
 * Returns 0 on success, -1 on error.
 */
int intentd_issue(uint32_t scope, uint32_t resource_id,
                  uint64_t ttl_ms, struct CapabilityToken *out);

/* ------------------------------------------------------------------ */
/* Utility                                                             */
/* ------------------------------------------------------------------ */

/* Convert "a.b.c.d" to network-byte-order uint32 resource_id.
 * Returns 0 on parse error (0.0.0.0 is not a valid target anyway). */
uint32_t intentd_ip_to_resource(const char *ip_str);

/* Print a human-readable token summary to stdout */
void intentd_token_print(const struct CapabilityToken *tok);

#endif /* IK_INTENTD_H */
