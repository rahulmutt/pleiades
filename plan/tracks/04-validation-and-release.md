# Track 4 — Validation and Release

## Role

Ensure remaining implementation work produces evidence strong enough for release claims: accuracy reports, compatibility profiles, backend matrices, artifact summaries, and reproducible bundles.

## Standards

- Validate against authoritative public source data where practical.
- Preserve small golden fixtures for regression tests even after full readers/generators exist.
- Publish tolerances by backend, body class, time range, and coordinate output.
- Include known gaps and constraints in compatibility profiles instead of hiding them.
- Archive release reports and checksums with each release bundle.

## Required report families

- cross-backend comparison reports;
- artifact validation reports;
- benchmark reports;
- compatibility-profile verification reports;
- backend capability matrices;
- release notes and release checklists;
- pure-Rust/native-dependency audit summaries.

## Release readiness rule

A release should not claim production ephemeris accuracy, compressed artifact coverage, or full compatibility-catalog coverage until the generated reports verify those claims from the current source revision.
