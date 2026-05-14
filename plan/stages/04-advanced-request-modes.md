# Phase 4 — Advanced Request Modes and Policy

## Goal

Implement or consistently defer the advanced request behavior described in `spec/api-and-ergonomics.md`, `spec/backend-trait.md`, and `requirements.md` FR-2/FR-3/FR-10.

## Starting point

Current first-party backends support mean geometric geocentric tropical requests for TT/TDB instants where metadata allows them. Caller-supplied UTC/UT1-to-TT/TDB offset helpers exist, but built-in Delta T, leap-second, DUT1, and relativistic conversion models are deliberately out of scope. Apparent-place corrections, topocentric body positions, and native sidereal backend output are rejected unless a backend explicitly advertises support.

## Implementation goals

- Decide whether production releases will add built-in UTC/Delta-T convenience or continue with caller-supplied offsets.
- If built-in time conversion lands, add typed policy objects, validation corpora, rustdoc, CLI behavior, and release reports.
- Implement apparent-place corrections only with clear capability metadata, source references, and validation thresholds.
- Add topocentric body-position support only through an explicit request surface that remains distinct from house observers.
- Keep sidereal conversion in the domain/façade layer unless a backend implements and advertises native equivalent output.
- Preserve structured errors for unsupported apparentness, observer-bearing geocentric-only requests, unsupported time scales, unsupported frames, and malformed observers.
- Add precedence tests whenever invalid and unsupported request dimensions interact.

## Completion criteria

Phase 4 is complete when every advanced request dimension is implemented with evidence or consistently documented as unsupported in rustdoc, CLI output, backend matrices, request-policy reports, release bundles, and tests.

## Out of scope

- Improving base ephemeris accuracy unless required by an implemented advanced mode.
- Changing house/ayanamsa catalog status.
