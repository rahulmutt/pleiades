# Phase 3 — Advanced Request Support

## Goal

Close or explicitly defer advanced request semantics required by `spec/api-and-ergonomics.md`, `spec/backend-trait.md`, and `spec/requirements.md` FR-2/FR-3.

## Starting point

Current first-party backends expose mean geometric geocentric positions for TT/TDB-style requests. Built-in UTC/UT1 conversion, Delta T policy, apparent-place corrections, topocentric body positions, and native sidereal backend output are currently deferred or rejected with structured errors.

## Implementation goals

- Decide the first production release policy for UTC input convenience and Delta T handling.
- If implemented, add typed conversion APIs, validation cases, rustdoc, and report summaries.
- If deferred, keep the explicit policy summaries and CLI/report wording current.
- Decide whether any backend will implement apparent-place corrections; reject apparent requests until capability metadata and validation support them.
- Decide whether topocentric body positions are in scope for the release; preserve chart-level observer use for houses without implying topocentric body positions.
- Keep sidereal conversion in the domain/façade layer unless a backend explicitly advertises native equivalent support.
- Add regression tests for precedence among invalid observer, unsupported time scale, unsupported apparentness, and unsupported topocentric requests.

## Completion criteria

Phase 3 is complete when every advanced request dimension is either implemented with validation and capability metadata or explicitly documented as unsupported/deferred in API docs, CLI output, backend matrices, release profiles, and release gates.

## Out of scope

- Improving astronomical source accuracy unless needed for an implemented advanced mode.
- Catalog formula/provenance audits.
