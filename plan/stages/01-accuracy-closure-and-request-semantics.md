# Phase 1 — Reference Accuracy and Request Semantics

## Purpose

Close the remaining production ephemeris and request-policy gaps so downstream artifact generation and release claims rely on trustworthy, documented source outputs.

This phase starts from an already-working workspace: backend traits, chart façade, VSOP87B generated tables for Sun-through-Neptune, compact lunar baseline, JPL snapshot fixtures, request-policy reporting, and structured unsupported-mode errors are in place. The remaining work is production evidence and final policy decisions.

## Spec drivers

- `requirements.md` FR-1, FR-2, FR-3, FR-7, FR-8, NFR-2, NFR-3
- `backend-trait.md` request/result/metadata/error semantics
- `backends.md` backend capability matrix expectations
- `api-and-ergonomics.md` time, observer, frame, apparentness, batch, and error behavior
- `validation-and-testing.md` reference comparison, golden tests, regression tests, benchmarks

## Current baseline

Implemented and not re-planned here:

- backend trait, metadata, request validation, composite/routing helpers, batch APIs, and structured errors;
- mean geometric, tropical, geocentric TT/TDB request support in first-party backends;
- explicit rejection/reporting for unsupported time scales, apparent-place requests, observer-bearing topocentric body-position requests, and native sidereal backend requests;
- chart-level observer handling for houses without silently implying topocentric body positions;
- VSOP87B generated binary tables for Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune;
- compact lunar baseline for Moon, mean/true node, and mean apogee/perigee;
- JPL Horizons snapshot and hold-out fixtures with selected asteroid rows, provenance summaries, equatorial reconstruction, interpolation transparency, and batch parity evidence;
- validation/report commands that expose request policy, source documentation, comparison tolerance posture, benchmark summaries, and reference-corpus summaries.

## Remaining implementation goals

### 1. Establish release-grade body claims

- Decide the first release's body claim set by backend and body class.
- Keep Pluto as an explicitly approximate fallback unless a source-backed pure-Rust path is validated for the claimed range and tolerance.
- Ensure any approximate or fixture-only path is excluded from release-grade accuracy claims unless evidence supports it.
- Publish body-class tolerances and validation status for all advertised release-grade bodies.

### 2. Broaden reference/source coverage

- Decide whether `pleiades-jpl` becomes a broader public JPL-derived reader/corpus provider or remains a checked-in fixture backend paired with a separate generation-input path.
- Add enough public source/reference coverage to support production validation and Phase 2 artifact fitting over the advertised 1500-2500 CE range.
- Include boundary dates, high-curvature windows, lunar windows, selected asteroid coverage, and independent hold-out rows.
- Preserve pure-Rust parsing, deterministic manifests, checksum validation, and source provenance.

### 3. Finalize lunar source posture

- Decide whether the first production release keeps the compact Meeus-style lunar baseline or adds a fuller ELP-style coefficient implementation.
- If the compact baseline remains, publish release limitations and measured error envelopes by supported lunar channel.
- If coefficient data is added, implement pure-Rust ingestion/evaluation with provenance, redistribution review, and validation against published references.
- Keep true apogee/perigee and other unsupported lunar points as structured unsupported-body errors until implemented and validated.

### 4. Finalize advanced request semantics

For each area, either implement the behavior or document release deferral through metadata, compatibility/profile summaries, rustdoc, and structured errors:

- built-in Delta T policy and UTC/UT1 convenience conversion;
- apparent-place corrections;
- topocentric body positions;
- native sidereal backend output versus domain-layer sidereal conversion;
- ecliptic/equatorial frame transformation precision and supported time-scale assumptions.

### 5. Strengthen validation gates

- Add golden/reference tests for all release-claimed bodies and frames.
- Add boundary-date, out-of-range, batch/single parity, and unsupported-mode precedence tests where claims expand.
- Make comparison reports fail or clearly block release when advertised body classes exceed tolerance.
- Keep validation evidence separate from provenance-only or interpolation-transparency evidence.

## Done criteria

Phase 1 is complete when:

- every release-grade body claim has source provenance, measured tolerances, and validation status;
- Pluto and other approximate paths are either validated within thresholds or explicitly downgraded/excluded from release-grade claims;
- the reference/generation input corpus is sufficient for Phase 2 artifact fitting;
- Delta T, UTC/UT1, apparentness, topocentric, native sidereal, and frame behavior is either implemented or explicitly deferred with metadata and structured errors;
- standard format, lint, and test checks pass.

## Deferred to later phases

- Generating and shipping production compressed artifacts belongs to Phase 2.
- Completing catalog formula/reference audits belongs to Phase 3.
- Publishing a final release bundle belongs to Phase 4.
