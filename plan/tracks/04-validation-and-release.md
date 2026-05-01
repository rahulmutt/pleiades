# Track 4 — Validation and Release

## Role

Ensure remaining implementation work produces evidence strong enough for release claims: accuracy reports, compatibility profiles, backend matrices, artifact summaries, benchmarks, audits, and reproducible bundles.

## Standards

- Validate against authoritative public source data where practical.
- Preserve small golden fixtures for regression tests even after full readers/generators exist.
- Publish tolerances by backend, body class, time range, coordinate output, and artifact profile.
- Include known gaps and constraints in compatibility profiles instead of hiding them.
- Archive release reports, artifact metadata, manifests, and checksums with each release bundle.
- Make release-facing commands fail closed when generated evidence contradicts advertised claims.

## Required report families

- cross-backend comparison reports;
- JPL/reference corpus and source-provenance reports;
- artifact validation and benchmark reports;
- compatibility-profile verification reports;
- backend capability matrices;
- API stability posture reports;
- release notes, release checklists, release summaries, and bundle manifests;
- pure-Rust/native-dependency audit summaries.

## Release readiness rule

A release must not claim production ephemeris accuracy, compressed artifact coverage, apparent/topocentric/native-sidereal support, or full compatibility-catalog coverage until generated reports verify those claims from the current source revision.
