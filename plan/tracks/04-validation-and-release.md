# Track 4 — Validation and Release

## Role

Turn generated evidence into release gates that prevent stale artifacts and overstated claims.

## Standards

- Validation should distinguish reference, hold-out, boundary, fixture-exactness, and provenance-only evidence.
- Release profiles must state exact shipped built-ins, aliases, constraints, known gaps, and unsupported modes.
- Bundle verification must compare staged files to live renderers or reproducible source inputs where practical.

## Required report families

- source-corpus provenance and coverage
- backend capability and body-claim matrices
- packaged-artifact profile, accuracy, regeneration, checksum, and benchmarks
- house/ayanamsa compatibility evidence
- request-mode policy summaries
- native dependency/build audit

## Release readiness rule

A release is not ready until validation and bundle verification fail closed on missing inputs, stale generated outputs, artifact threshold failures, unsupported-mode claim drift, and compatibility-profile overclaims.
