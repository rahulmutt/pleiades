# Status 2 — Next Slice Candidates

This file lists active implementation slices only. Completed command aliases,
summary wrappers, bundle sidecars, and report-cache changes are intentionally
omitted.

## Phase 1 — Production reference backend and corpus

The reproducible de440 generation pipeline now produces a real, broad corpus
(~25,659 data rows across boundary, interior, fast-cluster, hold-out, and
independent fixture-golden slices, sampled per-body at each body's own cadence)
committed under `crates/pleiades-jpl/data/corpus/` with real non-zero checksums
and a pinned kernel SHA-256. A clean checkout verifies kernel-free via
`pleiades-validate validate-corpus` and reproduces all slices from de440 with
`PLEIADES_DE_KERNEL` set; the live fail-closed gate covers missing bodies/roles,
schema/checksum drift, malformed/non-finite rows, placeholder SHA, and an
independent Horizons fixture-golden cross-check (600 km tolerance for giant
planets, which resolve to de440 system barycenters). The remaining slices are:

- Add a broad public-data reader for arbitrary external JPL-style data products,
  beyond the pinned de440 kernel and checked-in fixtures, on top of the existing
  combined, split-source, and path-backed split-source loaders.
- Adopt a small-body asteroid SPK kernel for broader selected-asteroid source
  coverage and record its provenance in `docs/spk-kernel-sourcing.md`.

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

## Phase 6 — Target catalog completion and expansion (deferred, post-first-release)

These slices are end-state work and are not part of the active first-release
frontier. They are listed so the full target catalog is not treated as complete
once Phase 5 audits pass.

- Implement remaining `compatibility-catalog.md` house systems beyond the
  baseline 11, each with formula, aliases, constraints, and provenance.
- Grow the ayanamsa catalog from the baseline 5 toward the full Swiss Ephemeris
  `SE_SIDM_*` set.
- Expand selected-asteroid coverage where source evidence supports it.
- Add optional chart utilities: aspects/orb-ready angular separations and
  dignities, built above the core domain layer.
