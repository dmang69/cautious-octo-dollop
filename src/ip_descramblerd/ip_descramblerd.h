/*
 * IntentKernel — ip-descramblerd public API
 *
 * ip-descramblerd is the AI OS Security Engine: it analyzes outbound
 * IP addresses and returns an allow/block verdict before the network
 * request is executed.
 *
 * Analysis pipeline
 * -----------------
 *  1. Validate the capability token (delegates to capd)
 *  2. Look up the IP in the built-in threat database
 *  3. Apply GeoIP / policy rules
 *  4. (Future) Run ONNX ML model for anomaly scoring
 *  5. Return a Verdict
 *
 * Copyright 2025 Daniel Kirk Owings — Apache License 2.0
 */

#ifndef IK_IP_DESCRAMBLERD_H
#define IK_IP_DESCRAMBLERD_H

#include <stdint.h>
#include "intentd/token.h"   /* Verdict typedef + CapabilityToken */

/* ------------------------------------------------------------------ */
/* Lifecycle                                                           */
/* ------------------------------------------------------------------ */

int  ip_descramblerd_init(void);
void ip_descramblerd_shutdown(void);

/* ------------------------------------------------------------------ */
/* Analysis                                                            */
/* ------------------------------------------------------------------ */

/*
 * ip_descramblerd_analyze — analyze an IPv4 address string.
 *
 *   ip_str : dotted-decimal IPv4 address (e.g. "142.250.72.14")
 *   token  : capability token that authorized this network request
 *
 * Returns one of:
 *   VERDICT_ALLOW            — IP appears safe, proceed
 *   VERDICT_BLOCK_MALICIOUS  — IP is in threat database
 *   VERDICT_BLOCK_SUSPICIOUS — IP has elevated risk score
 *   VERDICT_BLOCK_POLICY     — IP blocked by local policy
 *
 * The function validates the token internally.  If the token is
 * invalid it returns VERDICT_BLOCK_POLICY.
 */
Verdict ip_descramblerd_analyze(const char *ip_str,
                                const struct CapabilityToken *token);

/* Human-readable verdict string */
const char *ip_descramblerd_verdict_str(Verdict v);

#endif /* IK_IP_DESCRAMBLERD_H */
