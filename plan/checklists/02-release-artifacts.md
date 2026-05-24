# Checklist 2 — Release Artifacts

A production release bundle must include current, reproducible evidence for the
claims it makes.

## Required artifact families

- Compatibility profile and release notes.
- Backend capability matrix and body-claim summaries derived from validated
  evidence.
- Source-corpus provenance, coverage, source revisions, checksums, and hold-out
  reports.
- Packaged-artifact binary, manifest, checksum sidecars, profile, validation
  reports, regeneration record, and benchmarks.
- House and ayanamsa catalog evidence summaries, including aliases,
  source-label mappings, provenance, and constraints.
- Request-mode policy summaries for time scale, observer/topocentric,
  apparentness, frame, zodiac/sidereal, speed/motion, and unsupported modes.
- Native dependency/build-hook audit and workspace tool-version provenance.

## Release readiness rule

Do not publish a release bundle when any generated artifact is stale, any
production threshold is exceeded, any public profile overclaims support, any
required source/provenance checksum cannot be reproduced, or any required gate is
missing from bundle verification.
