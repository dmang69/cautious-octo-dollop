/*
 * IntentKernel — libintentkernel SDK public API
 *
 * This is the library that applications link against.  It presents
 * the nine primitive IntentKernel APIs; only network_request() is
 * implemented in this initial reference build.
 *
 * network_request() flow
 * ----------------------
 *   1. Parse URL → extract hostname/IP
 *   2. Request one-shot capability from intentd
 *   3. Submit token to capd for validation
 *   4. Submit (token, IP) to ip-descramblerd for threat analysis
 *   5. Perform HTTP request only if verdict == ALLOW
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_LIBINTENTKERNEL_H
#define IK_LIBINTENTKERNEL_H

/* ------------------------------------------------------------------ */
/* Lifecycle                                                           */
/* ------------------------------------------------------------------ */

/*
 * ik_init — initialize all IntentKernel subsystems.
 * Must be called once per process before any SDK function.
 */
int  ik_init(void);
void ik_shutdown(void);

/* ------------------------------------------------------------------ */
/* Error codes                                                         */
/* ------------------------------------------------------------------ */

#define IK_OK               0
#define IK_ERR_ACCESS_DENIED  (-1)
#define IK_ERR_INVALID_URL    (-2)
#define IK_ERR_TOKEN_FAILED   (-3)
#define IK_ERR_NOT_INIT       (-4)

/* ------------------------------------------------------------------ */
/* SDK Primitives (v1 — network_request implemented)                   */
/* ------------------------------------------------------------------ */

/*
 * ik_network_request — make exactly one outbound network request.
 *
 *   url : HTTP or HTTPS URL, e.g. "https://example.com"
 *
 * Returns IK_OK on success (request performed), or a negative
 * IK_ERR_* code on failure.
 *
 * No app can call this without a valid capability.
 * No capability exists without user intent.
 * No capability survives past its TTL.
 */
int ik_network_request(const char *url);

#endif /* IK_LIBINTENTKERNEL_H */
