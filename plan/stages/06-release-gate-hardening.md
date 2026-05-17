# Phase 6 — Release Gate Hardening

## Goal

Make production release claims reproducible and enforceable from a clean checkout, as required by `spec/validation-and-testing.md`.

## Starting point

CLI and validation tooling can render audits, summaries, artifact reports, benchmark matrices, compatibility checks, API stability reports, release-house-validation summaries, packaged-artifact phase-2 corpus alignment summaries, and release-bundle rehearsals. These gates must become blocking once artifact, reference, body-claim, request-policy, and catalog evidence are production-ready. The backend matrix summary now validates the compatibility profile before rendering so release matrix output fails closed on profile drift, the release bundle generator now validates the release-grade body-claims and Pluto-fallback summaries before serializing them, the comparison-corpus release-grade guard summary now validates before rendering in comparison/report surfaces, the release bundle verifier now cross-checks the Pluto fallback summary against the release-grade body claims posture before accepting a staged bundle, the benchmark provenance block now records cargo version alongside source revision, workspace status, and rustc version for reproducibility, the validation-report and packaged-data backend provenance surfaces now route request-policy and frame-treatment through validated report helpers, the release bundle keeps the native-dependency audit summary bookkeeping explicit instead of reusing workspace-audit byte counts, and the verify-release-bundle display now includes the packaged-artifact phase-2 corpus alignment summary alongside the other staged release artifacts. The release bundle now also stages and verifies the lunar-theory catalog validation summary alongside the lunar-theory limitations summary, keeping the body-model claim-boundary evidence explicit in the manifest and directory checks.

## Implementation goals

- Ensure formatting, clippy, tests, pure-Rust/native-dependency audits, artifact validation, compatibility-profile verification, benchmarks, and bundle verification run reproducibly locally and in CI.
- Stage release bundles with current compatibility profiles, backend matrices, request-policy summaries, validation reports, benchmark summaries, artifact manifests, checksums, source revisions, tool versions, and release notes.
- Fail on stale generated outputs, artifact threshold violations, unsupported-mode claim drift, profile mismatches, missing release files, checksum drift, or native dependency regressions.
- Archive enough generation parameters and input checksums to reproduce shipped artifacts.
- Keep README, docs, release notes, and compatibility profiles aligned with the exact release evidence.

## Completion criteria

Phase 6 is complete when maintainers can cut a release from a clean checkout and verify that every published claim is backed by current generated evidence.

## Out of scope

- Inventing new feature scope during release hardening.
- Relaxing thresholds or claims to make a release pass without corresponding evidence.
