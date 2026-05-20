# Status 2 — Next Slice Candidates

This file lists only active implementation slices. Completed report aliases, summary wrappers, cache optimizations, and bundle cross-check additions are intentionally omitted.

## Phase 1 — Production reference/source corpus

- Expand source coverage for all release-claimed major bodies, lunar channels, Pluto policy, and selected asteroids across 1500-2500 CE.
- Keep corpus provenance surfaces aligned; JPL reference, hold-out, and boundary-overlay summaries now surface explicit evidence-class labels alongside the existing source revision checksums, and the JPL source corpus contract now has a typed release-facing summary.
- Create independent hold-out coverage that is not consumed by artifact fitting.
- The release bundle now carries independent-holdout body-class coverage alongside the source-window evidence so hold-out coverage is explicit in staged artifacts.
- Make backend matrices and release profiles derive body/date/channel claims from validated corpus evidence; the backend matrix summary now also carries the comparison corpus release-grade guard, reference/hold-out overlap, independent hold-out, release-grade body claims, and Pluto fallback posture.
- Keep corpus provenance surfaces aligned; source revision checksums now have a dedicated `production-generation-source-revision-summary` report command and are bundled/verified as a first-class release artifact.

## Phase 2 — Production compressed ephemeris

- Rebase artifact generation on the Phase 1 corpus.
- Replace draft tolerance posture with enforced production thresholds per body class and channel.
- Continue fitting/reconstruction work only where it improves measured reference and hold-out errors.
- Keep artifact size/decode/lookup/batch/chart benchmarks current.
- Keep unsupported outputs explicit, especially apparent, topocentric, native sidereal, and motion policy.

## Phase 3 — Body and backend claim completion

- Resolve Pluto status before any production compatibility claim.
- Decide whether to implement fuller lunar theory or constrain lunar/lunar-point claims.
- Promote Ceres, Pallas, Juno, Vesta, and any custom asteroid support only where evidence is broad enough.
- Ensure backend capability metadata rejects unsupported request shapes before computation.

## Phase 4 — Advanced request modes

- Decide whether built-in UTC/Delta-T convenience is in scope for the first production release.
- Implement apparent-place support only with documented corrections and validation fixtures.
- Implement topocentric body positions only with clear observer semantics and tests.
- Keep native sidereal backend output unsupported unless a backend provides validated native behavior.

## Phase 5 — Compatibility and release readiness

- Audit house formulas, aliases, and latitude/numerical failure constraints.
- Audit ayanamsa offsets, epochs, formula/provenance notes, aliases, and near-equivalent variants.
- Keep compatibility profiles exact about shipped built-ins, descriptor-only entries, constraints, aliases, and gaps.
- Make release gates fail on stale generated artifacts, overbroad claims, missing evidence, native-dependency drift, and threshold failures.
