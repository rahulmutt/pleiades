# Plan Overview

`pleiades` has completed the original workspace bootstrap: mandatory crates exist, core typed APIs are in place, catalogs are broad, validation and release-rehearsal tooling exists, and a draft packaged-data artifact is checked in.

The active plan now starts after that foundation. It focuses on closing the remaining specification gaps needed before production release claims are truthful.

## Active phases

1. [Production compressed data](stages/01-production-compressed-data.md)
2. [Production reference inputs](stages/02-production-reference-inputs.md)
3. [Advanced request support](stages/03-advanced-request-support.md)
4. [Compatibility catalog evidence](stages/04-compatibility-catalog-evidence.md)
5. [Release gate hardening](stages/05-release-gate-hardening.md)

## Current priority

The next implementation work should prioritize **Phase 1: Production compressed data**. The draft artifact is reproducible and inspectable, but its current fit errors are far outside production thresholds. Work should improve the fitting strategy, generation inputs, validation thresholds, and benchmarks without broadening user-facing accuracy claims prematurely.

## Cross-cutting rules

- Keep all first-party code pure Rust.
- Preserve crate layering from `spec/architecture.md`.
- Treat checked-in fixtures and hold-out rows as evidence classes, not as a substitute for production source coverage.
- Keep unsupported advanced modes as structured errors until implemented and validated.
- Keep compatibility profiles aligned with implemented behavior and known gaps.
