/*
 * IntentKernel capability_core.c
 *
 * Educational placeholder reference skeleton.
 * NOT production-ready. NOT a complete security implementation.
 */

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint64_t not_before;
    uint64_t expires_at;
    uint32_t action;
    uint32_t resource_id;
    uint16_t remaining_uses;
    bool revoked;
} ik_capability_t;

/* Platform hooks (to be provided by runtime integration). */
extern uint64_t ik_now_monotonic_ms(void);
extern bool ik_verify_token_signature(const ik_capability_t *cap);

/* Validate capability bounds. */
bool ik_validate_capability(const ik_capability_t *cap,
                            uint32_t requested_action,
                            uint32_t requested_resource_id) {
    uint64_t now;

    if (cap == NULL) {
        return false;
    }

    if (!ik_verify_token_signature(cap)) {
        return false;
    }

    now = ik_now_monotonic_ms();
    if (now < cap->not_before || now > cap->expires_at) {
        return false;
    }

    if (cap->revoked || cap->remaining_uses == 0) {
        return false;
    }

    if (cap->action != requested_action || cap->resource_id != requested_resource_id) {
        return false;
    }

    return true;
}

/* Consume one use on successful authorization. */
bool ik_consume_capability(ik_capability_t *cap) {
    if (cap == NULL || cap->remaining_uses == 0 || cap->revoked) {
        return false;
    }

    cap->remaining_uses--;
    return true;
}

/* Explicit revocation helper. */
void ik_revoke_capability(ik_capability_t *cap) {
    if (cap != NULL) {
        cap->revoked = true;
        cap->remaining_uses = 0;
    }
}
