# Osculating (true) lunar apsides: "True Lilith" (`TrueApogee` + `TruePerigee`)

**Date:** 2026-06-30
**Status:** Design approved, pending spec review
**Origin:** Follow-up sub-project recorded at the end of
`docs/superpowers/specs/2026-06-29-eclipse-subsystem-design.md` ("True
(osculating) lunar apogee"); the standing gap is catalogued in
`docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md` (the two
`GAP (unsupported)` rows) and in `PLAN.md` ("True Apogee/Perigee remain
unsupported").

## Problem

The `CelestialBody` enum already carries `TrueApogee` and `TruePerigee`
(`crates/pleiades-types/src/bodies.rs:75,79`), but every backend refuses them:
`pleiades-elp` marks both `BodyClaim::unsupported`
(`crates/pleiades-elp/src/backend.rs:41-42`) and the packaged-data artifact does
not bundle them. Only the **mean** apsides exist today (`MeanApogee` =
`mean_perigee + 180°`, a smooth analytic series in
`crates/pleiades-elp/src/backend.rs:80-82`).

The osculating ("true") apogee — Swiss Ephemeris `SE_OSCU_APOG`, the astrologer's
"True Black Moon Lilith" — is a fundamentally different quantity. It is not a
mean element but the **apoapsis direction of the Moon's instantaneous osculating
Kepler ellipse**, derived from the Moon's geocentric position *and velocity*
vector at the instant. It oscillates tens of degrees around the mean apogee, so
it cannot be expressed as a low-order time polynomial; it must be computed from a
live lunar state vector.

## Goals

1. Serve `TrueApogee` and `TruePerigee` as **ReleaseGrade** bodies, reaching
   Swiss Ephemeris `SE_OSCU_APOG` parity (perigee = the opposite apse).
2. Compute them from the **release-grade packaged Moon** state, not the
   constrained compact-ELP Moon, since the osculating apse direction is sensitive
   to the velocity vector.
3. Lock the result in behind a new fail-closed `validate-lilith` gate against a
   committed Swiss Ephemeris reference corpus, wired into the release-gate set.
4. Keep the orbital math in an isolated, independently unit-testable unit.

### Non-goals (YAGNI)

- Equatorial / declination output for the apsides (that is the **next** queued
  follow-up sub-project, designed separately).
- Swiss Ephemeris `SE_INTP_APOG` (interpolated/"natural" apogee) and topocentric
  apsides.
- Any change to the constrained mean apsides or to the ELP backend's lunar
  theory.
- Re-deriving the apsides from the compact ELP Moon (explicitly rejected: its
  velocity error would put the osculating longitude off by potentially degrees).

## Key facts established from the codebase

- The production stack is a `RoutingBackend`
  (`crates/pleiades-cli/src/commands/chart.rs:559-566`) with
  `PackagedDataBackend` **first**; it serves the release-grade Moon and shadows
  the compact ELP Moon. `RoutingBackend` returns the first provider whose
  `supports_body` is true (`crates/pleiades-backend/src/traits.rs:265-297`).
- `PackagedDataBackend` exposes the Moon's full geocentric state at any instant
  in `[1900, 2100]`: 3D position via `lookup_ecliptic` (λ, β, dist) **and an
  analytic velocity** via `lookup_motion` (`SpeedPolicy::FittedDerivative`) —
  `crates/pleiades-data/src/backend.rs:139-184`. No finite-differencing of the
  Moon is required. Lookup is continuous, not snapped to a sample grid.
- The Moon state is **geocentric J2000 mean ecliptic** at the backend boundary
  (`apparent: false, mean: true`); all apparent corrections (light-time,
  precession, nutation, aberration) live in the chart layer via
  `pleiades-apparent` and are already applied **per body** — the chart already
  special-cases the Sun (FU-1) at `crates/pleiades-core/src/chart/mod.rs` near
  the apparent stage.
- `pleiades-compression` already provides spherical↔Cartesian state conversion
  (`frame_recombine.rs:47-121`, `SphericalState`/`CartesianState`).
- The repo already vendors Swiss Ephemeris (`libswisseph-sys 0.1.2`, SE 2.10.03)
  and generates committed SE reference corpora via small Rust tools
  (`tools/se-ayanamsa-reference`, `tools/se-house-reference`) feeding fail-closed
  gates in `pleiades-validate`.

## Approach (chosen: ① extend `PackagedDataBackend` + isolated math module)

Rejected alternatives: a dedicated `pleiades-apsides` crate/backend (cleanest
separation but needs dependency wiring to reach the packaged Moon state, and
another crate to publish + gate); inline computation in the chart engine
(bloats the already-large `chart/mod.rs` and breaks the "every body is a backend
body" consistency that even `MeanApogee` honors).

### Data flow

```
PackagedDataBackend::position(TrueApogee | TruePerigee, instant)
  1. query its OWN Moon state at `instant`:
       lookup_ecliptic → (λ, β, d)        [J2000 mean ecliptic, geocentric]
       lookup_motion   → (λ̇, β̇, ḋ)        [analytic, FittedDerivative]
  2. spherical state → Cartesian (r, v)    [reuse frame_recombine.rs]
  3. osculating::apsides(r, v, μ) → {apogee, perigee} (λ, β, d)   [still J2000 mean]
  4. return EclipticCoordinates(apo|peri) as a MEAN-J2000 geometric direction;
     Motion via finite-difference of the apsis longitude at t±0.5d (Derived);
     Apparentness::Mean, frame = Ecliptic.

ChartEngine apparent stage (crates/pleiades-core/src/chart/mod.rs, near Sun branch)
  special-case TrueApogee/TruePerigee → GEOMETRIC of-date path:
     precession (J2000 → mean equinox of date) + nutation Δψ (→ true equinox of date) ONLY
     NO light-time, NO annual aberration  (it is a geometric direction, not a body)

ChartSnapshot → façade applies ayanamsa for sidereal, like every other body.
```

### The osculating math (new isolated unit)

A pure module/crate (working name `pleiades-apsides` / `osculating`) with a
single public entry point and no I/O or frame logic beyond its inputs:

```
apsides(state: CartesianState /* r,v in J2000 mean ecliptic */, mu: f64) -> Apsides
// Apsides { apogee: EclipticDirection(λ,β,d), perigee: EclipticDirection(λ,β,d), ecc, semi_major }
```

Newtonian two-body geometry:

- Eccentricity vector `e_vec = ((v·v − μ/|r|)·r − (r·v)·v) / μ`. It points to
  **perigee**; **apogee direction = −e_vec**.
- `a = 1 / (2/|r| − |v|²/μ)`, `e = |e_vec|`; apogee distance `a(1+e)`, perigee
  distance `a(1−e)`.
- The apse line lies in the inclined lunar orbit, so the apogee carries a real
  ecliptic latitude (~±5°); we return full (λ, β, d), matching `SE_OSCU_APOG`.
- `TruePerigee` is the same `e_vec`, opposite sign — free once `TrueApogee` is
  computed.

**μ is the main accuracy knob.** The apse *direction* depends on
`μ = G(M⊕ + M☾)` (the two terms of `e_vec` scale differently with μ), so μ must
match Swiss Ephemeris's convention to reach parity. The exact μ value is tuned
against the `validate-lilith` gate during implementation.

The Moon's osculating eccentricity (≈0.04–0.07) is always safely non-degenerate,
but the module documents the near-circular domain limit (`e → 0` makes the apse
direction ill-conditioned).

### Frame handling

- Input state: geocentric J2000 mean ecliptic (straight from the backend).
- Apse direction is computed in that inertial frame, then the **chart** rotates
  it by precession (J2000 → mean equinox of date) + nutation Δψ (→ true equinox
  of date), reusing the existing `pleiades-apparent` rotations **minus**
  light-time and aberration.
- Output frame therefore matches Swiss Ephemeris's default for `SE_OSCU_APOG`:
  **true ecliptic of date, nutation on, no aberration** (the SE-default target
  confirmed during design).
- This preserves the "backends are mean-only J2000" invariant: the backend emits
  a legitimate mean-J2000 geometric vector; all of-date work stays in the chart
  layer beside the Sun special-case.
- Sidereal longitude is obtained by the façade's ayanamsa subtraction, exactly
  as for every other body.

## Validation

### Reference tool — `tools/se-lilith-reference` (new)

Mirrors `tools/se-ayanamsa-reference`: links `libswisseph-sys`, calls
`swe_calc(jd, SE_OSCU_APOG /* =13 */, iflag)` over a date grid, emits a committed
golden CSV (λ, β, dist for the apogee; perigee is the opposite apse).

- **Ephemeris flag:** start with `SEFLG_SWIEPH` (SE's own DE-fit files — what the
  existing SE tools already use; no extra kernel wiring). The residual
  Moon-model difference vs the DE440-sourced packaged Moon is absorbed into the
  tolerance budget. **Empirical fallback** (the one open implementation detail,
  settled during the plan): if the band is too loose to be meaningful, switch to
  `SEFLG_JPLEPH` against the pinned DE440 to isolate the osculating math.
- **Frame flags:** tropical, of-date, nutation on — matching the chart output
  frame exactly.

### Gate — `validate-lilith` (new, fail-closed)

Modeled on the apparent/ayanamsa gates in `pleiades-validate`: load the committed
corpus, compute our `TrueApogee`/`TruePerigee` end-to-end through the chart
of-date path, assert max residual ≤ tolerance, and wire it into the release-gate
set (alongside the house/ayanamsa/apparent/topocentric/corpus gates).

- **Tolerance is set empirically** from the first measured run (expected
  arcsec-to-arcmin floor from μ convention + velocity fit + Moon-model
  difference), documented with a per-source rationale header like
  `crates/pleiades-validate/data/apparent-goldens.csv`.

## Claims, enum, and artifact

- `PackagedDataBackend`: add `TrueApogee`/`TruePerigee` to its body set
  (`crates/pleiades-data/src/lib.rs:173-187`) and `supports_body`, with
  **ReleaseGrade** `BodyClaim`s. These are **derived at lookup**, not stored, so
  there is **no `ARTIFACT_VERSION` bump and no artifact regeneration**.
- `pleiades-elp`: retains its own `unsupported` claim (per-backend claims are
  independent; the claims-audit is per-backend, and the packaged backend wins
  routing). Update the ELP rustdoc/`lib.rs` note so it no longer reads as a
  *global* gap — the true apsides are now served release-grade by the packaged
  backend ahead of ELP.

## Testing (TDD)

1. **Unit (math module):** closed-form ellipses — known `(r, v, μ)` → known apse
   direction / eccentricity / semi-major axis; perigee = apogee + 180°
   symmetry; near-circular domain guard.
2. **Backend:** `supports_body` true for both; `position` returns an apsis with a
   plausible latitude (~±5°) and a `Derived` motion of the correct sign.
3. **Chart:** requesting `TrueApogee` yields it in the snapshot; the apparent
   path applies **precession + nutation only** — an explicit regression guard
   asserting the result does **not** carry the ~20″ annual-aberration term (the
   FU-1 lesson, applied preventively to a new geometric point).
4. **Golden gate:** the `validate-lilith` corpus passes within tolerance.

## Docs to update on landing

- `README.md` and `PLAN.md` — drop "True Apogee/Perigee remain unsupported";
  record release-grade true apsides via the packaged backend.
- `docs/lunar-theory-policy.md`.
- `docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md` — flip
  the two `GAP (unsupported)` rows to `EXISTS`.
- The eclipse design's follow-up list, and a `docs/follow-ups.md` entry marking
  this sub-project done.

## Risks / open items

- **μ / SE convention parity** is the dominant accuracy risk; resolved
  empirically against the gate, with the `SEFLG_JPLEPH`/DE440 reference as a
  fallback to separate model error from math error.
- **Motion for the apsis** is finite-differenced (the apse longitude is not a
  stored channel), consistent with the ELP lunar-point motion convention; its
  large oscillation rate is expected and not an error.
