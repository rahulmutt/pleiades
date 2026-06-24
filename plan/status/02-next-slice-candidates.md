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

## Phase 2 — Release-grade compressed ephemeris (complete)

SP1, SP2, and SP3 have all landed. The packaged artifact (ARTIFACT_VERSION 7,
1900–2100 CE, ~10.0 MB) passes per-body-class accuracy ceilings (defined in
`crates/pleiades-data/src/thresholds.rs`), the hard size gate (≤ 12,000,000
bytes), and speed ceilings. Latency targets are tracked in `PACKAGED_BUDGETS`
(opt-in enforcement via `PLEIADES_ENFORCE_LATENCY`). Motion output is
`Motion = Derived` (SpeedPolicy::FittedDerivative), measured and gated.

Summary of completed SP2/SP3 outcomes:
- SP2: heliocentric-planet reframe — all bodies sub-arcsec (Uranus ~0.0036″,
  Neptune ~0.0020″, Pluto ~0.0018″, Saturn ~0.0009″, Jupiter ~0.0004″).
- SP3: published per-body-class ceilings enforced; hard size gate active; latency
  tracked; motion (FittedDerivative) gated against lon/lat/radial speed ceilings.

No open Phase 2 slices remain.

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

Phase 5 compatibility-audit pair is complete:

- House-system numeric gate **done** (`validate-houses`, 60-row SE corpus,
  per-formula-family arcsecond ceilings set from measured residuals; all 12
  baseline house systems validated). Audit: house formulas, aliases, source-label
  mappings, latitude/numerical constraints for release-claimed entries — complete.
- Ayanamsa epoch/offset/formula/alias/provenance audit **done** via the numeric gate
  (`validate-ayanamsa`, 60-row SE mean corpus, per-mode-class ceilings set from
  measured residuals; 6 release-claimed modes — Lahiri, Raman, Krishnamurti,
  Fagan/Bradley ≤ 2.0″; True Chitra, True Citra ≤ 1.0″). Remaining ~48 built-in
  ayanamsa variants are not-yet-gated (descriptor tests only; no claim broadening).

Remaining Phase 5 candidates:

- Release-gate hardening: audit any remaining generated artifacts whose stale
  output, missing input, unsupported-mode claim drift, or threshold failure is not
  yet checked by a release gate.
- Compatibility-profile overclaim checks: ensure no descriptor-only, custom-only,
  constrained, approximate, or unsupported entry is advertised as fully implemented
  in compatibility profiles or public surfaces.

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
