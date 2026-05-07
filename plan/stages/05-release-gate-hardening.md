# Phase 5 — Release Gate Hardening

## Goal

Make production release claims reproducible and enforceable from a clean checkout, as required by `spec/validation-and-testing.md`.

## Starting point

CLI and validation tooling can already render audits, summaries, artifact reports, benchmark matrices, compatibility checks, and release-bundle rehearsals. These gates must become blocking for production claims and must be rerun after artifact/reference/catalog work lands.

## Implementation goals

- Ensure `cargo fmt`, clippy, tests, pure-Rust/native-dependency audits, artifact validation, compatibility-profile verification, benchmarks, and release-bundle verification are all reproducible locally and in CI.
- Stage release bundles with current compatibility profiles, backend matrices, request-policy summaries, validation reports, benchmark summaries, artifact manifests, checksums, and release notes.
- Make release gates fail on stale generated summaries, artifact threshold violations, unsupported-mode claim drift, or native dependency regressions.
- Archive enough source revision, tool version, generation parameters, and checksum metadata to reproduce shipped artifacts.
- Keep public docs aligned with the exact release compatibility profile and known gaps.

## Completion criteria

Phase 5 is complete when maintainers can cut a release from a clean checkout and verify that every published claim is backed by current generated evidence.

## Out of scope

- Inventing new feature scope during release hardening.
- Relaxing thresholds or profile claims to make a release pass without corresponding evidence.
