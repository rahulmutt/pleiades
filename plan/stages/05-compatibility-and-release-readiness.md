# Phase 5 — Compatibility and Release Readiness

## Goal

Make compatibility catalog claims and release artifacts trustworthy enough for production consumers.

## Starting point

The workspace has broad house and ayanamsa descriptors, baseline calculations, custom-definition support, release compatibility profiles, explicit latitude-sensitive house-constraint reporting, report helpers, audits, and release-bundle rehearsal. The remaining gap is evidence and gate strictness, not catalog scaffolding.

## Implementation goals

- Audit house-system formulas, aliases, latitude/numerical constraints, and failure modes for entries promoted as implemented; the release bundle now also carries the house-code-aliases summary alongside the house-formula-families and house-latitude-sensitive audit summaries.
- Audit ayanamsa epochs, offsets, formulas, aliases, and provenance for entries promoted as implemented.
- Keep descriptor-only, constrained, approximate, custom-only, and unsupported entries distinct in compatibility profiles.
- Require release bundles to contain current profiles, validation reports, manifests, checksums, source revisions, tool versions, benchmark summaries, and release notes.
- Gate releases on format, clippy, tests, native-dependency audit, artifact validation, compatibility-profile verification, benchmark/report generation, and bundle verification.

Progress update: release-bundle verification now also re-checks the compact release checklist summary and catalog inventory summary against the current renderer, so the release gate fails closed on stale checklist prose, compatibility-catalog drift, and checksum drift. The target house-system and target ayanamsa scope notes also now have direct report helpers, and release bundles now also carry those target-scope summaries with verification, so release tooling can inspect the long-term compatibility horizon without parsing the full profile summary. The core compatibility profile now also exposes a dedicated ayanamsa provenance summary helper, keeping the representative provenance payload reusable across report surfaces and downstream consumers. Release-bundle verification now also re-checks the release-specific house-system and ayanamsa canonical-name summaries against the current renderer, so those release-label sidecars fail closed on semantic drift too. The CLI front-end now also dispatches catalog-posture and known-gaps directly, keeping the compact compatibility posture and caveat summaries first-class alongside the existing validate fallback path. House-validation snapshots now also render compact one-line diagnostics for direct review alongside the existing request and validation summaries. The CLI front-end now also directly dispatches `house-latitude-sensitive-constraints` with parity coverage, keeping the latitude-sensitive house constraints audit surface explicit from both binaries. The ayanamsa audit has now also absorbed the spaced `Fagan / Bradley` alias spelling, and now also normalizes spaced `Usha / Shashi` and `Vedic / Sheoran` spellings, keeping the Western sidereal alias coverage aligned with the catalog audit. Release-bundle verification now also re-checks the full validation report text against the current renderer after normalization, so full validation-report sidecars fail closed on semantic drift as well.

## Completion criteria

- A clean checkout can produce and verify release artifacts without hidden tooling or network requirements.
- Release gates fail on stale generated outputs, unsupported-mode claim drift, native-dependency regressions, artifact threshold failures, and overbroad compatibility/backend claims.
