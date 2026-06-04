# IKRL Specification (IntentKernel Relief Layer)

## Purpose

**IKRL** is the compatibility and deployment layer that applies IntentKernel capability enforcement concepts to existing operating systems without immediate host replacement.

## Supported Platform Classes

- Windows
- Linux
- Android
- macOS
- Embedded/RTOS environments

## Responsibilities

1. Intercept protected host resource access paths.
2. Require valid capability presentation for mediated actions.
3. Enforce TTL/use constraints and revocation outcomes.
4. Normalize host resources into UCCS-compatible descriptors.
5. Emit auditable enforcement outcomes.

## Staged Adoption Model

- **Stage 1 (Compatibility runtime):** user-space/service-level broker + interception hooks.
- **Stage 2 (Kernel-assisted enforcement):** deeper host integration for stronger mediation guarantees.
- **Stage 3 (Hardened deployments):** broader platform coverage and policy tooling.
- **Stage 4 (Native-oriented transition):** reduced dependence on ambient-authority host interfaces.

## Deployment Notes

IKRL is explicitly designed for incremental rollout in existing fleets. It does not claim immediate full replacement of host OS permission models.
