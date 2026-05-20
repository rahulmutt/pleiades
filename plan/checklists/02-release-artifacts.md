# Checklist 2 — Release Artifacts

A production release bundle must include current, reproducible evidence for the claims it makes.

## Required artifact families

- Compatibility profile and release notes.
- Backend capability matrix and body-claim summaries.
- Source-corpus provenance, coverage, checksums, and hold-out reports.
- Packaged-artifact binary, manifest, checksum sidecars, profile, validation reports, and benchmarks.
- House and ayanamsa catalog evidence summaries, including aliases and constraints.
- Request-mode policy summaries for time scale, observer/topocentric, apparentness, frame, zodiac/sidereal, and unsupported modes.
- Native dependency/build-hook audit and workspace tool-version provenance.

## Release readiness rule

Do not publish a release bundle when any generated artifact is stale, any production threshold is exceeded, any public profile overclaims support, or any required source/provenance checksum cannot be reproduced from the documented inputs.
