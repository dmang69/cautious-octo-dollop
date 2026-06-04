# IntentKernel Compatibility Model

## Overview

In compatibility deployments, host operating systems are treated as **resource providers** rather than root trust anchors for authorization semantics.

## Model

- Applications interact with host APIs through compatibility enforcement points.
- An `interceptor` mediates protected operations and requires a valid capability.
- Host-native handles are mapped to scoped capability context.
- Expiry, revocation, and use limits are enforced before allowing host operation completion.

## Enforcement Strategy (High Level)

1. Capture operation request at interception boundary.
2. Extract/present capability token or registered handle.
3. Validate signature, lifecycle state, scope, freshness, and policy constraints.
4. Permit or deny host call.
5. Record audit decision metadata.

## Interception Considerations

- Enforcement depth may vary by platform stage (user space vs kernel-assisted).
- Coverage gaps must be documented explicitly in deployment profiles.
- Compatibility mode is a transition mechanism, not proof of complete host mediation.
