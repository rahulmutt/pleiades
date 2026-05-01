# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Accuracy Closure and Request Semantics**.

The repository is past bootstrap and release-rehearsal scaffolding. The next production blocker is closing the remaining accuracy and request-policy gaps that prevent truthful release-grade ephemeris and packaged-data claims.

## Evidence reviewed

Current summaries show:

- all mandatory crates and release/reporting commands exist;
- VSOP87B source-backed generated binary paths cover the Sun through Neptune;
- Pluto remains an approximate mean-elements fallback in the backend catalog, while release-grade comparison and tolerance reports now exclude Pluto from evidence;
- the compact lunar baseline has documented reference evidence for supported lunar channels but is not a full ELP coefficient implementation;
- the JPL backend is a checked-in fixture/snapshot backend with expanded selected asteroid rows (J2000 plus 2001-01-01, 2132-08-31, and 2500-01-01) and expanded interpolation-quality coverage around JD 2451910.5, not a broad production reader/corpus;
- the JPL backend now exposes a production-generation boundary overlay corpus that appends the full independent hold-out validation snapshot to the checked-in reference snapshot for validation and artifact-generation work, and the evidence report now also surfaces the boundary-overlay provenance alongside that broader coverage slice; the overlay now also has a standalone request corpus, inventory summary, and provenance summary for generation tooling; the interpolation-quality corpus also now carries a 2451910.5 boundary sample with quadratic coverage;
- the packaged artifact is deterministic and validated as a prototype, and the checked-in fixture has been regenerated from the updated reference snapshot, but current fit errors are not release-grade;
- request policy is explicit: mean, tropical, geocentric TT/TDB requests are supported; apparent-place corrections, topocentric body-position requests, native sidereal backend output, and built-in Delta T/UTC convenience conversion are not implemented today.

## Why this frontier comes first

Phase 2 production artifacts require trusted generation inputs and tolerances. Phase 4 release claims require the same evidence. Therefore the next work should close source-backed accuracy gaps before expanding packaged-data or compatibility claims that depend on those outputs.

## Immediate blockers

1. **Reference corpus breadth** — Expand source/reference data enough to support production validation and artifact generation, not only fixture exactness.
2. **Advanced request semantics** — Decide whether Delta T/UTC convenience, apparent corrections, and topocentric body positions are implemented for the first release or intentionally deferred with metadata and structured errors.
3. **Release thresholds** — Convert interim broad tolerance posture into body-class-specific release thresholds for claimed scopes.

## Recommended next slice

Pluto downgrade and corpus cleanup is complete:

- keep Pluto explicitly labeled as approximate in backend metadata and fallback provenance;
- use the Pluto-excluded release-grade corpus for comparison/tolerance evidence;
- preserve the full snapshot corpus for provenance and validation archaeology;
- keep release tolerance audits fail-closed if Pluto is advertised beyond its evidence.

The next slice should focus on broader reference coverage and the remaining request-semantics decisions.

## Parallel safe work

The following can proceed without distracting from accuracy closure:

- house/ayanamsa formula and alias audits;
- documentation cleanup for already-explicit request policy;
- release-bundle smoke-test maintenance;
- artifact-profile identifier groundwork and production-profile skeleton plumbing that do not claim production fit accuracy yet.

## Constraints

- Preserve pure Rust and first-party crate layering.
- Do not couple domain crates to concrete backends.
- Do not loosen unsupported-mode errors to silently satisfy apparent/topocentric/native-sidereal requests.
- Do not publish broader compatibility or accuracy claims until validation evidence supports them.
