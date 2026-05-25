# Status 2 — Next Slice Candidates

This file lists active implementation slices only. Completed command aliases,
summary wrappers, bundle sidecars, and report-cache changes are intentionally
omitted.

## Phase 1 — Production reference backend and corpus

- Implement the chosen pure-Rust source strategy, either as a public-data
  reader/parser or a reproducible corpus-generation pipeline from public inputs.
  The production-generation manifest summary now validates the derived source,
  coverage, and boundary-request corpus records directly, and the release-facing
  body/date/channel posture now derives from validated corpus evidence.
- Broaden reference and hold-out coverage for luminaries, major planets, Pluto
  policy, lunar/lunar-point channels, baseline asteroids, and representative
  custom/numbered bodies across 1500-2500 CE.
- Store source evidence in a form that keeps reference, fitting, hold-out,
  boundary, fixture-exactness, and provenance-only rows separable; the
  exact-J2000 reference slice now carries an explicit major-body/selected-
  asteroid class split in the source-corpus report, the production-generation
  source summary now exposes an explicit source-class breakdown, and the merged
  production-generation body-class coverage/cadence split is now surfaced too,
  but broader source breadth and the remaining public-input strategy work still
  need to land.

## Phase 2 — Release-grade compressed ephemeris

- Rebase artifact generation on Phase 1 validated inputs.
- Replace draft tolerance posture with enforced production thresholds per body
  class and channel.
- Improve fitting/reconstruction where measured reference and hold-out errors
  exceed thresholds.
- Keep artifact size, checksum, decode, lookup, batch, and chart-workload
  benchmarks current.
- Keep unsupported outputs explicit, especially apparent, topocentric, native
  sidereal, civil-time, and motion policies.

## Phase 3 — Body/backend claim closure

- Resolve Pluto as validated, approximate, constrained, or excluded.
- Decide whether to implement fuller lunar theory or constrain lunar/lunar-point
  claims to the compact Meeus-style baseline.
- Promote selected asteroid support only where source evidence and backend
  metadata are broad enough.
- Audit backend capability metadata against actual supported request shapes.

## Phase 4 — Request-mode semantics

- Decide first-release scope for built-in UTC/UT1 and Delta-T behavior.
- Implement apparent-place support only with documented corrections and fixtures.
- Implement topocentric body positions only with explicit observer semantics and
  tests.
- Keep native sidereal backend output unsupported unless validated native backend
  behavior exists.
- Align motion/speed/retrograde output policy across backends, charts, CLI, and
  artifact profiles.

## Phase 5 — Compatibility and release gates

- Audit house formulas, aliases, source-label mappings, and latitude/numerical
  constraints for release-claimed entries.
- Audit ayanamsa offsets, epochs, formulas, aliases, near-equivalent variants,
  and provenance.
- Ensure descriptor-only, custom-only, constrained, approximate, and unsupported
  entries are not advertised as fully implemented.
- Add any missing release gates for stale generated outputs, missing source
  evidence, threshold failures, native-dependency drift, unsupported-mode claim
  drift, and compatibility-profile overclaims.
