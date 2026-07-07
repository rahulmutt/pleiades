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

Built-in civil-time UTC/UT1 + Delta-T conversion, apparent-place corrections,
topocentric body positions, and motion/speed output are all implemented and
gated. Only one slice remains:

- Keep native sidereal backend output unsupported unless validated native backend
  behavior exists.

## Phase 5 — Compatibility and release gates

Phase 5 compatibility-audit pair is complete:

- House-system numeric gate **done** (`validate-houses`, 138-row SE corpus over
  6 charts × 23 systems, per-formula-family arcsecond ceilings set from measured
  residuals — tightest families ≤ 1–2″, Quadrant ≤ 12″, SolarArc/Sunshine ≤ 66″
  at the lat-66° bound). Audit: house formulas, aliases, source-label mappings,
  latitude/numerical constraints for release-claimed entries — complete.
- Ayanamsa epoch/offset/formula/alias/provenance audit **done** via the numeric gate
  (`validate-ayanamsa`, 480-row SE mean corpus, per-mode-class ceilings set from
  measured residuals; 48 gated modes across 4 classes — OffsetDefined ≤ 3.0″
  (Lahiri, Raman, Krishnamurti, Fagan/Bradley, …); TrueStar ≤ 1.0″ (True Chitra,
  True Citra, …); Galactic ≤ 1.0″; FittedOffset ≤ 1.0″).

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

- All target `compatibility-catalog.md` house systems are shipped (25 built-ins,
  24 gated); only Albategnius corpus-gating remains (optional; beyond the SE
  23-code target).
- Finish the ayanamsa catalog: 48 of 59 built-ins are SE-gated; give the 11
  descriptor-only modes real metadata + gating and add any missing `SE_SIDM_*`
  modes.
- Expand selected-asteroid coverage where source evidence supports it.
- Add dignities, built above the core domain layer. (Aspects / orb-ready angular
  separations are already implemented.)

## Event-engine track (SP series, parallel to the phases above)

SP-1 (angles/sidereal time), SP-2a (longitude crossings, plus the SP-2a-FU
`validate-crossings` hardening), SP-2b (rise/set/transit + horizontal
coordinates, gated by `validate-rise-trans`), SP-2c (local per-observer
eclipse circumstances — `EclipseEngine::local_circumstances` /
`next_local_eclipse` / `previous_local_eclipse` for both solar and lunar
eclipses, contact times/magnitude/obscuration/azimuth/altitude/visibility,
gated by `validate-eclipses-local`), SP-3 (fictitious/hypothetical bodies
via the new `pleiades-fict` crate's `FictitiousBackend`, serving the
Swiss-Ephemeris `seorbel.txt` bodies 40-58 — Uranian/Hamburg planets,
Transpluto, Vulcan, White Moon, Waldemath, and the historical pre-discovery
Neptune/Pluto predictions — as unperturbed Kepler orbits rotated to the J2000
mean ecliptic, definitional claim tier, gated by the fail-closed two-tier
`validate-fictitious` gate over a committed 570-row Swiss-Ephemeris corpus;
all 18 non-Nibiru bodies sub-arcsecond, Nibiru carries a documented per-body
carve-out for its ~370 AD reference equinox), and SP-4 (planetary nodes and
apsides — `EventEngine::nod_aps`/`nod_aps_default` (`swe_nod_aps` analogues)
over three methods (Mean, Osculating, OsculatingBarycentric) and two
`ApsisConvention`s (Aphelion, SecondFocus); `pleiades-apsides` generalized
into shared `elements_from_state`/`points_from_elements` helpers; Mean covers
Moon/Sun/Mercury-Neptune (Earth not addressable, Sun's slots zeroed per SE's
own behavior), Osculating/OsculatingBarycentric cover any body the backend
chain can supply a state vector for; gated by the fail-closed `validate-nod-aps`
gate (alias `nod-aps-gate`) over a committed 184-row Swiss-Ephemeris corpus;
MEAN_PLANET/MEAN_MOON sub-arcsecond, OSCU_PLANET arcminute-class (Neptune
perihelion eccentricity amplification), OSCU_MOON cross-theory; fictitious/
asteroid `nod_aps` is engine-covered but gate-unreferenced since SE's own
`swe_nod_aps` omits fictitious bodies and offline backends can't sample
small-body osculating states) are all **done**. Next candidate slices:

- `swe_pheno` (phase, phase angle, and magnitude for Sun/Moon/planets).
- Custom fictitious-body orbital elements (user-supplied, beyond the
  committed `seorbel.txt` set).
- Occultations.
- Central-path cartography for solar eclipses.
