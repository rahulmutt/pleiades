# Phase 6 — Release Gate Hardening

## Goal

Make production release claims reproducible and enforceable from a clean checkout, as required by `spec/validation-and-testing.md`.

## Starting point

CLI and validation tooling can render audits, summaries, artifact reports, benchmark matrices, compatibility checks, API stability reports, release-house-validation summaries, and release-bundle rehearsals. These gates must become blocking once artifact, reference, body-claim, request-policy, and catalog evidence are production-ready.

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
