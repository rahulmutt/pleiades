# Status 2 — Next Slice Candidates

This file lists active implementation slices only. Completed command aliases,
summary wrappers, bundle sidecars, and report-cache changes are intentionally
omitted.

## Phase 1 — Production reference backend and corpus

Phase 1 is complete. The reproducible de440 generation pipeline produces a
real, broad corpus (~25,659 data rows across boundary, interior, fast-cluster,
hold-out, and independent fixture-golden slices, sampled per-body at each
body's own cadence) committed under `crates/pleiades-jpl/data/corpus/` with
real non-zero checksums and a pinned kernel SHA-256. A clean checkout verifies
kernel-free via `pleiades-validate validate-corpus` and reproduces all slices
from de440 with `PLEIADES_DE_KERNEL` set; the live fail-closed gate covers
missing bodies/roles, schema/checksum drift, malformed/non-finite rows,
placeholder SHA, and an independent Horizons fixture-golden cross-check (600 km
tolerance for giant planets, which resolve to de440 system barycenters). The
broad public-data reader (`pleiades-jpl::ingest`) and the curated asteroid
corpus (Tier A main-belt core from `sb441-n16`, Tier B constrained set from
Horizons over 1900-2100) are also complete. No open Phase 1 slices remain.

## Phase 2 — Release-grade compressed ephemeris

SP1 (dense de440-backed generation source + accuracy baseline) has landed:
artifact generation now fits least-squares polynomials sampled densely from
de440 within each per-body segment span, kernel-gated behind
`PLEIADES_DE_KERNEL` (same gate as corpus_regen). ARTIFACT_VERSION is now 5;
the regenerated artifact is ~201,873 segments / ~49.78 MB. A per-body accuracy
baseline vs the committed de440-derived hold-out is in
`crates/pleiades-data/src/accuracy_baseline.rs`; inner bodies + Sun + Moon are
sub-arcsec; outer planets are draft-level (Uranus ~156″, Neptune ~90″,
Pluto ~62″, Saturn ~11″, Jupiter ~1.7″). The constrained asteroid (433-Eros)
is re-derived from the committed reference snapshot (absent from de440 and
sb441-n16), constrained to 1900-2100. The artifact remains explicitly
draft-grade. Remaining Phase 2 slices:

- SP2: accuracy tuning — per-body span and degree tuning against the measured
  baseline, especially outer planets.
- SP3: enforce published accuracy thresholds per body class and channel; define
  size and latency budgets.
- Improve fitting/reconstruction where measured errors exceed thresholds.
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
