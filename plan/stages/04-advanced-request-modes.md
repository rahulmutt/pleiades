# Phase 4 — Advanced Request Modes

## Goal

Close the specification gap for request modes that astrology applications commonly need while preserving explicit semantics.

## Starting point

The type layer can represent UTC/UT1/TT/TDB and caller-supplied offsets. Current first-party backends support mean geometric geocentric tropical requests in TT/TDB where metadata permits. Apparent place, topocentric body positions, native sidereal backend output, and built-in civil-time/Delta-T modeling are not implemented as automatic backend behavior.

## Implementation goals

- Decide whether built-in UTC/civil-time and Delta-T modeling belongs in the first production release.
- If implemented, add explicit policies, tests, docs, and validation fixtures for UTC/UT1/TT/TDB conversions.
- Implement apparent-place corrections only when light-time, aberration, nutation, and related assumptions are specified and validated.
- Implement topocentric body positions only with observer validation, backend/domain capability metadata, and regression tests.
- Keep sidereal conversion in the domain layer unless a backend explicitly advertises native sidereal output and documents equivalence.

Progress update: observer and apparentness policy summaries now ship through the release bundle, manifest, and verification path, keeping the request-mode policy surfaces aligned with the CLI help text.

## Completion criteria

- Every request mode is either implemented with evidence or consistently rejected with a structured error.
- Direct backend APIs, chart façade APIs, CLI help, docs, tests, and release reports describe the same behavior.
