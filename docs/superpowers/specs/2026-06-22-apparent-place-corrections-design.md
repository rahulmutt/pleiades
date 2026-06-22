# Apparent-Place Corrections (Phase 4 sub-project) — Design

Status: approved design, ready for implementation planning.
Date: 2026-06-22
Phase: 4 — Request-Mode Semantics (second of the independent sub-projects, after civil-time)

## Summary

Add built-in apparent-place computation to `pleiades`: callers can request a
chart in apparent mode and receive body positions corrected for light-time
(planetary aberration), annual aberration, and nutation in longitude, referred
to the true equinox of date — together with typed provenance describing which
corrections were applied and how large they were.

This **reverses the current deliberate non-goal for apparent place**. Today the
first-party backends return only **mean geometric, geocentric, tropical**
positions; `Apparentness::Apparent` requests are rejected with a structured
error (`validate_request_policy`), and the apparentness policy summary documents
apparent place as unsupported. This sub-project implements apparent place as a
**chart-layer capability** and updates those policy surfaces to the
now-supported posture.

The work is additive and keeps the backend boundary clean: backends stay
mean-only, raw `EphemerisRequest { apparent: Apparent }` requests are still
rejected, and the chart facade satisfies an apparent request by issuing mean
backend requests and transforming them — the
"expose only mean requests through a higher-level configuration" path the spec
(`spec/api-and-ergonomics.md`) explicitly sanctions.

## Scope of "apparent" (first release)

The implemented correction set is the **astrology-standard subset**:

- **Light-time / planetary aberration** — the body is observed where it was
  τ = distance × light-travel-time-per-AU ago; the geocentric position is
  re-evaluated at the retarded epoch t − τ and iterated to convergence.
- **Annual aberration** — the κ = 20.49552″ displacement from the observer's
  (Earth's) velocity, applied in ecliptic coordinates.
- **Nutation in longitude (Δψ)** — the true-equinox-of-date longitude shift.

Deliberately **omitted** for the first release: relativistic gravitational
light-deflection by the Sun (sub-0.05″ except in the immediate vicinity of the
solar disk) and the full (untruncated) nutation series beyond the terms needed
for sub-arcsecond longitude. These omissions are documented in the apparent
policy summary and absorbed by the validation tolerance, not hidden.

Mean equinox of date vs. nutation: nutation in longitude is a rotation about the
ecliptic pole, so in **ecliptic** coordinates apparent longitude = mean
longitude + Δψ and ecliptic latitude is unchanged to first order. Nutation in
obliquity (Δε) enters only when **equatorial** output is produced (true
obliquity ε₀ + Δε); the chart's primary output is ecliptic.

## Goals

- Compute apparent ecliptic-of-date positions for the **release-grade bodies**
  (the Sun, Moon, and the planets/Pluto/Eros the packaged-data backend serves
  release-grade) across the packaged window (1900–2100 CE).
- Keep the correction math in a standalone, pure, checksum-pinned crate
  (`pleiades-apparent`) depending only on `pleiades-types`, mirroring the
  `pleiades-time` shape.
- Carry typed `ApparentProvenance` on every apparent placement (which
  corrections, their magnitudes, light-time iterations, model sources).
- Never silently downgrade: an apparent request for a non-release-grade body
  returns a structured error rather than a quietly-mean result.
- Satisfy the Phase 4 exit criteria for an implemented request mode: validation
  fixtures, rustdoc/API examples, CLI coverage, policy/metadata surfacing, and
  release-profile entries.

## Non-goals

- Gravitational light-deflection and the full nutation series (see Scope).
- Topocentric body positions (separate Phase 4 slice; apparent is the
  astronomical foundation it will later build on).
- Native sidereal backend output (remains an unsupported non-goal).
- Making backends apparent-aware. Apparent stays a chart-layer capability; the
  mean-only backend contract is unchanged.
- Apparent **motion/speed** output — the SP3 derived-speed channel stays mean;
  apparent motion is out of scope for this slice.

## Architecture

### New crate: `pleiades-apparent`

Pure math, depends only on `pleiades-types`, checksum-pinned data, typed
provenance — the `pleiades-time` pattern. Modules:

- **`error`** — `ApparentPlaceError` with variants `NonConvergentLightTime`,
  `MissingDistance`, `NonFiniteCorrection`, `StaleModelData { kind }`. Each
  exposes `summary_line()` + `Display` + `std::error::Error`.
- **`nutation`** — nutation in longitude Δψ and obliquity Δε from the truncated
  IAU-1980 series. The coefficient table is a committed CSV embedded with
  `include_str!` and FNV-1a checksum-pinned behind a fail-closed gate, matching
  the `leap`/`deltat` table idiom in `pleiades-time`. Public:
  `nutation(jd_tt) -> Result<Nutation { delta_psi_arcsec, delta_eps_arcsec }, ApparentPlaceError>`
  and `mean_obliquity(jd_tt) -> f64`.
- **`aberration`** — annual aberration as a **pure function** of
  (body λ, β, Sun true longitude ⊙, eccentricity e and longitude of perihelion ϖ
  of date). The chart layer supplies ⊙ from its own Sun query, keeping this
  crate free of any ephemeris dependency. Public:
  `annual_aberration(lambda, beta, sun_true_longitude, jd_tt) -> AberrationOffset { d_lambda_arcsec, d_beta_arcsec }`.
- **`lighttime`** — `apparent_via_light_time(instant, f)` where
  `f: Fn(Instant) -> Result<EclipticCoordinates, E>` returns a geocentric
  position carrying `distance_au`. Iterates t − τ
  (τ = distance_au × 0.005_775_518_3 days/AU) until the position converges or a
  fixed iteration cap is hit (→ `NonConvergentLightTime`). Returns the
  light-time-corrected geometric geocentric position plus the iteration count.
- **`apparent`** — orchestrator. Given a light-time-corrected position, the Sun's
  true longitude, and the instant, applies annual aberration then Δψ to produce
  the apparent ecliptic-of-date position. Returns
  `ApparentPosition { ecliptic, provenance }`.
- **`provenance`** — `ApparentProvenance { light_time_days: f64, iterations: u8,
  nutation_longitude_arcsec: f64, aberration_longitude_arcsec: f64,
  corrections_applied: CorrectionSet, model_sources: &'static str }` with
  `summary_line()` + `Display`, mirroring `ConversionProvenance`.
- **`policy`** — `ApparentPlacePolicySummary` describing the implemented subset
  and the deliberately-omitted deflection, with the same
  blank/whitespace/linebreak/out-of-sync `validate()` discipline as the other
  policy summaries.

### Chart-layer orchestration (`pleiades-core`)

When a chart request carries `apparentness == Apparent`, the chart-compute path:

1. Queries the **Sun** (mean) once at the chart instant to obtain its true
   geometric longitude ⊙ for the aberration term.
2. For each **release-grade** body, drives `lighttime::apparent_via_light_time`
   with a closure that re-queries the same backend at the retarded instant
   (mean mode), then applies `aberration` + nutation Δψ via
   `apparent::apparent`.
3. Builds an apparent `EphemerisResult` (`apparent = Apparent`) and attaches the
   `ApparentProvenance` to the `BodyPlacement` via a new field
   `apparent: Option<ApparentProvenance>`.

Light-time requires `distance_au` on the mean result; a release-grade body
missing distance yields `MissingDistance` (surfaced as a structured chart
error). An apparent request for a **non-release-grade** body yields a structured
"apparent place not validated for `<body>`" error — fail-closed, no silent mean
fallback.

### Backend boundary

Unchanged. `validate_request_policy` still rejects
`EphemerisRequest { apparent: Apparent }` for the mean-only backends; the chart
facade only ever issues mean backend requests. Apparent is documented as a
chart-layer capability in the policy summaries.

## CLI

Add `chart --apparent` (mean remains the default). When set, the chart output
appends an apparent-provenance line per body (corrections applied + magnitudes),
mirroring how `--civil` appends conversion provenance. `--apparent` composes
with `--civil`, sidereal, and house options.

## Validation

Both reference sources (per the design decision):

- **Horizons goldens** — apparent ecliptic-of-date longitudes for the
  release-grade bodies at sampled epochs spanning 1900–2100, committed via the
  existing `pleiades-jpl` Horizons ingest path, behind a fail-closed
  cross-check. Per-body tolerance ceilings absorb the omitted light-deflection
  (tighter toward ~0.1″ for inner bodies/Sun/Moon, looser for outer
  planets/Pluto). Ceilings live alongside the existing threshold tables.
- **Meeus / Astronomical Almanac anchors** — 1–2 hand-entered worked
  apparent-place examples committed as offline golden constants (independent of
  Horizons), e.g. a Meeus Chapter 23/25 apparent-place example.
- **`pleiades-apparent` unit tests** — nutation at the Meeus 1987-04-10 epoch
  (Δψ ≈ −3.788″, Δε ≈ +9.443″, ε ≈ 23°26′36.85″); annual-aberration magnitude
  bounded below 20.5″; light-time convergence within the iteration cap; ecliptic
  latitude invariance under the Δψ rotation.

## Docs & policy alignment

- Update `CURRENT_APPARENTNESS_POLICY_SUMMARY_TEXT` and its `validate()`
  expectation to the now-supported chart-layer posture.
- Update the unsupported-modes summary so apparent place is no longer listed as
  unsupported (topocentric and native sidereal remain).
- Update `docs/time-observer-policy.md` apparentness section, the README
  apparent line (currently "rejected"), and the release profiles.
- Update `PLAN.md` and `plan/stages/04-advanced-request-modes.md` to mark
  apparent place complete and narrow the remaining Phase 4 work to topocentric +
  native-sidereal.

## Assumptions to verify before implementation (plan Task 0)

- **Equinox of date** — confirm each release-grade mean position is referred to
  the *mean equinox of date* (precession applied). If any backend returns a
  J2000-frame longitude instead, that body also needs precession-to-date before
  Δψ + aberration; the design adds a precession step only if this check fails.
- **Distance availability** — confirm the packaged-data backend populates
  `distance_au` for every release-grade body, since light-time depends on it.

## Exit criteria (Phase 4, apparent place)

- Apparent requests for release-grade bodies return genuinely computed apparent
  positions with typed provenance; non-release-grade apparent requests fail with
  a structured, documented error.
- Validation fixtures (Horizons goldens + Almanac anchors + crate unit tests)
  pass under a fail-closed gate.
- rustdoc/API examples, CLI coverage, policy/metadata surfacing, and
  release-profile entries are present and consistent.
