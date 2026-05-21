# Status 2 — Next Slice Candidates

This file lists only active implementation slices. Completed report aliases, summary wrappers, cache optimizations, and bundle cross-check additions are intentionally omitted.

## Phase 1 — Production reference/source corpus

- Expand source coverage for all release-claimed major bodies, lunar channels, Pluto policy, and selected asteroids across 1500-2500 CE.
- Keep corpus provenance surfaces aligned; JPL reference, hold-out, and boundary-overlay summaries now surface explicit evidence-class labels and explicit frame-treatment/time-scale posture alongside the existing source revision checksums, the production-generation corpus-shape summary now validates both ecliptic and equatorial boundary request corpora, the production-generation source summary now also validates its explicit evidence-class fragment, the source-corpus cadence now fails closed when ecliptic/equatorial boundary-request epoch counts diverge, the selected-asteroid source evidence/window summaries now carry the same explicit posture, the packaged-artifact phase-2 corpus alignment summary now carries both selected-asteroid request-corpus frames, the JPL source corpus contract now has a typed release-facing summary, the consolidated source-corpus posture now also has its own standalone summary/alias surface with an explicit shared schema label, and the comparison-corpus summary now re-validates against the current renderer during bundle verification.
- The production-generation boundary request corpus now also has an equatorial summary surface and CLI alias, so the boundary overlay corpus can be inspected directly in both frames without relying on ad hoc report prose.
- The backend matrix summary now also pulls in validated production-generation coverage, body-class coverage, corpus shape, source-corpus contract lines, and the consolidated source-corpus posture line so release-facing matrix output exposes the current corpus claims directly.
- The comparison snapshot manifest summary now fails closed on redistribution drift so provenance posture stays explicit in release-facing validation.
- The release bundle now carries independent-holdout body-class coverage alongside the source-window evidence so hold-out coverage is explicit in staged artifacts.
- Make backend matrices and release profiles derive body/date/channel claims from validated corpus evidence; the backend matrix summary now also carries the comparison corpus release-grade guard, reference/hold-out overlap, independent hold-out, release-grade body claims, and Pluto fallback posture.
- The release summary now also mirrors validated production-generation body-class coverage and corpus-shape lines, so the condensed release overview stays anchored to current corpus evidence.
- The shared celestial-body class taxonomy now drives the release-facing body-class coverage summaries, reducing duplicated body-class matching across report families.
- Keep the production-generation source summary explicit about both license, redistribution, and schema posture so the checked-in fixture corpus stays audit-friendly; its cadence fragment now derives from the checked-in source-window and boundary-request corpus counts instead of a hardcoded prose pair, the source summary now also records the apparentness posture alongside frame/time-scale/column provenance, and the production-generation source revision line now appears in the release summary and backend matrix as part of the same provenance block.
- JPL provenance-only evidence now renders as its own report line so provenance-only rows stay separate from tolerance, hold-out, and fixture-exactness evidence.

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

- Audit house formulas, aliases, and latitude/numerical failure constraints; the compatibility profile and report surfaces now expose explicit latitude-sensitive house-constraint summaries.
- Audit ayanamsa offsets, epochs, formula/provenance notes, aliases, and near-equivalent variants; the compatibility inventory now also surfaces representative ayanamsa provenance examples so those audits remain visible in release-facing summaries.
- Keep compatibility profiles exact about shipped built-ins, descriptor-only entries, constraints, aliases, and gaps; the catalog inventory line now also surfaces the current ayanamsa metadata-gap count alongside custom-definition labels, and the release notes summary now carries the house-validation corpus plus the ayanamsa catalog validation and sidereal-metadata coverage lines.
- Make release gates fail on stale generated artifacts, overbroad claims, missing evidence, native-dependency drift, and threshold failures.
