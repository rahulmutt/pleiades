# Phase 1 — Accuracy Closure and Request Semantics

## Purpose

Close the remaining production ephemeris gaps so downstream artifact generation and release claims rely on trustworthy, documented source outputs.

The workspace already has the required crate structure, backend trait, chart façade, VSOP87B generated tables for the Sun through Neptune, a compact lunar baseline, a JPL Horizons fixture backend, validation reports, and explicit fail-closed request policy. This phase is only about what remains: accuracy outliers, broader evidence, and advanced request semantics that are either implemented or deliberately deferred with structured errors.

## Spec drivers

- `requirements.md` FR-1, FR-2, FR-3, FR-7, FR-8, NFR-2, NFR-3
- `backend-trait.md` request/result/metadata/error semantics
- `backends.md` JPL, VSOP87, ELP, backend capability matrix expectations
- `api-and-ergonomics.md` time, observer, frame, apparentness, batch, and error behavior
- `validation-and-testing.md` reference comparison, golden tests, regression tests, benchmarks

## Current baseline

Implemented and not re-planned here:

- backend trait, metadata, request validation, composite routing, batch APIs, and structured errors;
- mean geometric geocentric TT/TDB request handling with explicit rejection of unsupported mean/apparent, topocentric, and native-sidereal modes;
- source-backed VSOP87B generated binary tables for Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune;
- compact lunar baseline with validation for Moon, mean/true node, and mean apogee/perigee;
- JPL Horizons fixture/snapshot backend with selected asteroid rows, exact fixture epochs, interpolation transparency, equatorial reconstruction, and batch parity evidence; the interpolation-quality corpus now includes a 2451910.5 boundary sample with quadratic coverage;
- comparison and regression tolerance audits now use body-class-specific release thresholds for the active planetary corpus instead of the earlier flat release posture;
- validation/report commands that expose tolerance posture, source documentation, request policy, frame policy, and benchmark summaries; the shared frame-policy summary now also fails closed when it drifts from the current canonical posture, and the observer/apparentness policy surfaces are now typed and validated in the shared request-semantics formatter so those report lines fail closed too.

Known remaining gaps:

- Pluto remains an approximate mean-elements fallback, and release-facing reports now label it as an explicit approximate fallback rather than a release-grade major-body claim.
- The JPL backend is fixture/snapshot-oriented, not a broad production reader/corpus for artifact generation.
- The lunar backend is a compact Meeus-style baseline, not a full ELP coefficient implementation.
- Built-in Delta T modeling, UTC convenience conversion, apparent-place corrections, and topocentric body positions are not implemented; unsupported behavior is explicit today.
- Production-grade error envelopes are incomplete for full 1500-2500 claims and for selected asteroid expansion beyond checked-in fixture rows.

## Remaining implementation goals

### 1. Resolve Pluto and major-body accuracy outliers

- Keep Pluto as an explicitly downgraded approximate fallback unless a source-backed public-data path is proven and validated for the release profile.
- Add reference comparisons over representative 1500-2500 epochs for all claimed major bodies.
- Keep fallback/approximate paths clearly labeled and excluded from release-grade claims.

### 2. Expand reference-source coverage

- Decide whether `pleiades-jpl` becomes a parser/reader for a broader public JPL-derived corpus or remains a fixture backend paired with a separate generation input path.
- Add enough source/reference rows to validate production artifact generation, including boundary dates and high-curvature windows. The checked-in reference snapshot now surfaces the Moon 2451911.5/2451912.5 high-curvature window as a dedicated report slice, the major-body high-curvature summary now extends through the 2451914.5 boundary day across all comparison bodies, the lunar source-window summary now also includes the reference-only apparent Moon comparison windows, the checked-in reference snapshot source-window summary now has typed body-window validation, and the production-generation boundary overlay now also exposes a per-body window summary; remaining work is to widen source windows beyond lunar-only evidence if production artifact generation needs additional body coverage.
- Preserve pure-Rust parsing, deterministic fixture manifests, checksums, and provenance records.
- Expand selected asteroid evidence for Ceres, Pallas, Juno, Vesta, and any custom/named bodies advertised in release profiles.

### 3. Finish lunar-source posture

- Decide whether the first production release keeps the compact lunar baseline or adds a fuller ELP-style coefficient selection.
- If keeping the compact baseline, publish release-grade limitations and error envelopes by lunar channel.
- If adding coefficient data, implement pure-Rust ingestion/evaluation, provenance, redistribution review, and validation against published references.
- Keep unsupported true apogee/perigee or other lunar points explicit until implemented and validated.

### 4. Decide advanced request semantics

For each area, either implement the behavior or document the release deferral with structured unsupported errors and capability metadata:

- UTC/UT1 convenience conversion and Delta T policy;
- apparent-place corrections;
- topocentric body positions;
- native sidereal backend output versus domain-layer sidereal conversion;
- equatorial/ecliptic frame transformations and precision expectations.

### 5. Strengthen validation evidence

- Add golden/reference tests for all claimed bodies and frames.
- Add boundary-date and out-of-range regression tests.
- Keep batch/single parity tests for mixed frames and mixed TT/TDB requests.
- Make validation reports fail or clearly flag any advertised release profile whose bodies exceed release tolerance.

## Done criteria

Phase 1 is complete when:

- no body advertised as release-grade has unresolved tolerance outliers in validation reports;
- Pluto and any other currently approximate release-scope body is either source-backed or downgraded explicitly and excluded from release-grade claims;
- JPL/reference-source coverage is sufficient for production artifact generation inputs;
- Delta T, UTC/UT1, apparentness, topocentric, sidereal, and frame behavior is implemented or explicitly deferred with metadata and structured errors;
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test --workspace` pass.

## Work intentionally deferred

- Generating and shipping production compressed artifacts belongs to Phase 2.
- Completing catalog formula/reference audits belongs to Phase 3.
- Publishing a final release bundle belongs to Phase 4.
