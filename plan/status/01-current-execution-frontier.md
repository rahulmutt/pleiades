# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Reference Accuracy and Request Semantics**.

The repository is past bootstrap, MVP API work, catalog scaffolding, report-surface expansion, and release-bundle rehearsal. The next production blocker is not another CLI summary or fixture-archaeology path; it is the evidence needed to make truthful release-grade ephemeris and packaged-data claims.

## Evidence reviewed

Current implementation status shows:

- all mandatory first-party crates exist and respect the `pleiades-*` naming rule;
- the backend trait, metadata, batch APIs, composite/routing helpers, chart façade, compatibility profiles, and validation/report commands are in place;
- first-party backend request policy is explicit: mean geometric, tropical, geocentric TT/TDB requests are supported; unsupported time scales, apparent-place requests, topocentric body-position requests, and native sidereal backend requests fail with structured errors;
- chart APIs preserve the distinction between house observers and body observers and provide caller-supplied UTC/UT1/TT/TDB offset helpers, but built-in Delta T/UTC conversion policy remains a release decision;
- `pleiades-vsop87` is source-backed for Sun through Neptune via generated VSOP87B tables; Pluto remains an explicitly approximate mean-elements fallback;
- `pleiades-elp` is a compact Meeus-style lunar baseline with validation evidence for supported lunar channels, not a full ELP coefficient implementation;
- `pleiades-jpl` is a checked-in JPL Horizons snapshot/hold-out fixture backend with provenance and selected asteroid evidence, not a broad production reader/corpus;
- `pleiades-data` ships a deterministic prototype artifact with codec/profile/checksum/regeneration support, but it is not a production 1500-2500 CE artifact and current fit posture is not release-grade;
- house and ayanamsa catalogs are broad, but not every release-advertised entry has independent formula/provenance/reference evidence sufficient for full interoperability claims;
- release-bundle generation and verification exist, but final release gates must be rerun over production accuracy evidence, production artifacts, and truthful compatibility profiles.

## Why this frontier comes first

Phase 2 production artifacts require trusted generation inputs and target thresholds. Phase 4 release claims require the same evidence. Therefore maintainers should close source-backed accuracy and request-policy gaps before claiming production packaged-data coverage or broad compatibility.

## Immediate blockers

1. **Reference corpus breadth** — Expand or replace fixture evidence with production-suitable public source/reference data for validation and artifact fitting.
2. **Pluto release posture** — Either source-back Pluto within release thresholds or keep it visibly approximate and excluded from release-grade major-body claims.
3. **Lunar release posture** — Decide whether the first release ships the compact lunar baseline with measured limitations or implements fuller ELP-style coefficient support.
4. **Advanced request semantics** — Decide whether built-in Delta T/UTC conversion, apparent corrections, topocentric body positions, and native sidereal backend output are implemented now or explicitly deferred.

## Recommended next slice

Implement a small, reviewable reference-coverage slice:

- choose a concrete public source input path for production validation/generation;
- add a minimal but representative set of new source rows or parser support across the 1500-2500 target range;
- include at least one boundary or high-curvature window relevant to artifact fitting;
- add validation that distinguishes production tolerance evidence from provenance-only fixture evidence;
- update the relevant backend metadata/report summaries and tests without broadening release claims prematurely.

## Parallel safe work

The following can proceed without blocking Phase 1:

- house/ayanamsa formula, alias, latitude, and custom-definition audits;
- documentation cleanup for already-explicit request policy and known gaps;
- artifact-profile metadata hardening that does not claim production fit accuracy;
- release-bundle smoke-test maintenance that keeps existing rehearsal tooling accurate.

## Constraints

- Preserve pure Rust and first-party crate layering.
- Do not make domain crates depend on concrete backends.
- Do not silently satisfy unsupported apparent/topocentric/native-sidereal requests.
- Do not publish broader accuracy, artifact, or compatibility claims until validation evidence supports them.
