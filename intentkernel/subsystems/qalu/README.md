# Q-ALU Quaternary Computing Subsystem

Replaces binary bus encoding with quaternary (base-4) signaling
inside IntentKernel. Each capability token wire carries 2 bits
instead of 1 — halving physical trace count for the same bandwidth.

## Results
- 2x wire count reduction on capability token buses
- 2.67x total switching activity reduction
- GF(4) [5,3] ECC corrects any single-symbol level-shift error
- 77,000+ test assertions — zero failures

## How it connects to IntentKernel
- capd: Q-ISA 8-quat instruction format maps to capability token fields
- intentd: quaternary intent encoding doubles token information density
- qsimd: Q-ALU digit-wise MIN/MAX/TSUM as SIMD lane operations
- quantumd: Q-RAM replaces binary volatile store with 2x density
- qproofd: GF(4) ECC provides hardware integrity beneath proof layer

## Run tests
    bash tests/run_all.sh

## Thesis
Beyond Binary: Architectures of the Fourth State
DOI: 10.5281/zenodo.19332771
