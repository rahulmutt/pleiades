# Plan Overview

`pleiades` has completed the bootstrap/foundation roadmap. The active plan now tracks only production gaps that remain against `SPEC.md` and `spec/*.md`.

## Active phases

1. **Artifact accuracy and packaged-data production** — replace the draft packaged-data fixture with a production 1500-2500 CE artifact.
2. **Reference/source corpus productionization** — provide public, documented, deterministic inputs for validation and artifact fitting.
3. **Body-model completion and claim boundaries** — settle Pluto, fuller lunar theory/lunar points, and selected asteroid release claims.
4. **Advanced request modes and policy** — implement or consistently reject UTC/Delta-T, apparent, topocentric, and native-sidereal modes.
5. **Compatibility catalog evidence** — audit house and ayanamsa formulas, aliases, constraints, and profile status.
6. **Release gate hardening** — make release bundles and gates fail closed on drift or unsupported claims.

## Current priority

Prioritize Phase 1, but do not broaden packaged-data claims until Phase 2-quality inputs are available. The current draft path now passes the calibrated fit thresholds, and the target-threshold policy is recorded as production-ready, but the Phase 2 corpus still needs to be finalized; current artifact manifest, checksum, output-support, cadence, fit-outlier, and benchmark reports are useful diagnostics, not substitutes for release-grade evidence.

## Cross-cutting rules

- Keep the workspace pure Rust and layered according to `spec/architecture.md`.
- Keep unsupported modes as structured errors until implemented and validated.
- Keep release profiles and public docs aligned with current generated evidence.
- Remove completed implementation slices from active status files instead of accumulating historical notes.
