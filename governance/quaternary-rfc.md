# RFC: Quaternary Computing Subsystem

**Status:** Draft
**Author:** Daniel Kirk Owings (dmang69), M13THCO Research Division
**Date:** 2026-05-30
**DOI:** 10.5281/zenodo.19332771

## Summary
Add a quaternary (base-4) computing subsystem to IntentKernel that
reduces capability token bus wire count by 2x while maintaining
full binary software compatibility through binary-overlay encoding.

## Motivation
IntentKernel's capability token model generates significant
inter-daemon bus traffic. Binary encoding requires N wires for
N bits. Quaternary encoding requires N/2 wires for the same N bits.
At the IntentKernel scale (10 daemons, continuous capability
token exchange), this halves physical routing complexity.

## Proposal
1. Add `intentkernel/subsystems/qalu/` with Q-ALU, Q-RAM, QCPU
2. Add `docs/quaternary-encoding.md` specification
3. Add CI verification workflow
4. Phase 2: wrap as capability tokens via capd

## Verification
77,000+ test assertions, zero failures.
Q-ALU: 65,536 arithmetic tests.
GF(4) ECC: 960 single-error correction tests.
QCPU: 6 integration programs including factorial.

## Compatibility
Full binary backward compatibility via binary-overlay encoding.
No changes to existing daemon interfaces in Phase 1.
