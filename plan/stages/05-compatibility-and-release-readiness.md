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

Progress update: release-bundle verification now also re-checks the compact release checklist summary and catalog inventory summary against the current renderer, so the release gate fails closed on stale checklist prose, compatibility-catalog drift, and checksum drift. The target house-system and target ayanamsa scope notes also now have direct report helpers, so release tooling can inspect the long-term compatibility horizon without parsing the full profile summary.

## Completion criteria

- A clean checkout can produce and verify release artifacts without hidden tooling or network requirements.
- Release gates fail on stale generated outputs, unsupported-mode claim drift, native-dependency regressions, artifact threshold failures, and overbroad compatibility/backend claims.
