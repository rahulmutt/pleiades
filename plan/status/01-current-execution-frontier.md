# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Accuracy Closure and Request Semantics**.

The repository is past bootstrap and release-rehearsal scaffolding. The next production blocker is closing the remaining accuracy and request-policy gaps that prevent truthful release-grade ephemeris and packaged-data claims.

## Evidence reviewed

Current summaries show:

- all mandatory crates and release/reporting commands exist;
- VSOP87B source-backed generated binary paths cover the Sun through Neptune;
- Pluto remains an approximate mean-elements fallback and is explicitly downgraded out of release-grade major-body claims;
- the compact lunar baseline has documented reference evidence for supported lunar channels but is not a full ELP coefficient implementation;
- the JPL backend is a checked-in fixture/snapshot backend with selected asteroid rows, not a broad production reader/corpus;
- the packaged artifact is deterministic and validated as a prototype, but current fit errors are not release-grade;
- request policy is explicit: mean, tropical, geocentric TT/TDB requests are supported; apparent, topocentric body-position, native sidereal backend output, and built-in Delta T are not implemented today.

## Why this frontier comes first

Phase 2 production artifacts require trusted generation inputs and tolerances. Phase 4 release claims require the same evidence. Therefore the next work should close source-backed accuracy gaps before expanding packaged-data or compatibility claims that depend on those outputs.

## Immediate blockers

1. **Pluto accuracy posture** — Replace, source-back, or explicitly downgrade Pluto so validation reports no longer advertise an unqualified release-grade outlier.
2. **Reference corpus breadth** — Expand source/reference data enough to support production validation and artifact generation, not only fixture exactness.
3. **Advanced request semantics** — Decide whether Delta T/UTC convenience, apparent corrections, and topocentric body positions are implemented for the first release or intentionally deferred with metadata and structured errors.
4. **Release thresholds** — Convert interim broad tolerance posture into body-class-specific release thresholds for claimed scopes.

## Recommended next slice

Start with **Pluto downgrade and corpus cleanup**:

- keep Pluto explicitly downgraded and out of release-grade claims until a validated source-backed path exists;
- remove Pluto from release-grade corpus/tolerance assertions where it is only approximate;
- update backend metadata, capability summaries, and comparison reports to say exactly what is and is not supported;
- make release tolerance audits fail closed if Pluto is advertised beyond its evidence.

This slice keeps the release posture truthful without overstating approximate paths.

## Parallel safe work

The following can proceed without distracting from accuracy closure:

- house/ayanamsa formula and alias audits;
- documentation cleanup for already-explicit request policy;
- release-bundle smoke-test maintenance;
- artifact-profile design that does not claim production fit accuracy yet.

## Constraints

- Preserve pure Rust and first-party crate layering.
- Do not couple domain crates to concrete backends.
- Do not loosen unsupported-mode errors to silently satisfy apparent/topocentric/native-sidereal requests.
- Do not publish broader compatibility or accuracy claims until validation evidence supports them.
