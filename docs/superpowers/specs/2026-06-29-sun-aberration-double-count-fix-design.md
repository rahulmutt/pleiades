# Fix the geocentric-Sun aberration double-count in chart apparent place (FU-1)

**Date:** 2026-06-29
**Status:** Design approved, pending spec review
**Origin:** `docs/follow-ups.md` FU-1, flagged during `phase6-eclipse-subsystem`.

## Problem

For the **Sun observed geocentrically**, light-time retardation and annual
(stellar) aberration are the *same* ~20.5″ Earth-orbital reflex effect, not two
independent corrections (Meeus, *Astronomical Algorithms* §25).

The chart apparent path in `crates/pleiades-core/src/chart/mod.rs` (~lines
304–313) applies **both** for the Sun:

1. The `query` closure passed to `apparent_position` re-queries the **geocentric**
   Sun at each light-time-retarded epoch (`t − τ`, τ ≈ 499 s), which already
   displaces it ~20.5″.
2. `apparent_position` (`crates/pleiades-apparent/src/apparent.rs:50`) then adds
   the annual-aberration term *on top*.

Result: a systematic ~+20″ (~0.006°) error in the apparent solar ecliptic
longitude. It is currently masked — the Sun golden tolerance is 26″
(`crates/pleiades-validate/data/apparent-goldens.csv`) — but it is a real
inaccuracy in a release-grade body.

This is **Sun-specific**. For the planets, light-time and stellar aberration are
genuinely distinct ("planetary aberration" = both), so `apparent_position` is
correct for them. The Moon shares Earth's heliocentric velocity (annual
aberration applies) *and* has its own geocentric motion (light-time re-query
applies); those are distinct, so the Moon path is also correct and is left
unchanged here.

### Evidence

`pleiades-eclipse` proved the *same* packaged backend matches an independent
Skyfield 1.54 + DE440 apparent solar longitude to **~0.5″** once aberration is
applied **once** — see `crates/pleiades-eclipse/src/ephemeris.rs::apparent_sun_longitude_deg`
and the `validate-eclipses` gate (≤1.0″ longitude tolerance passing on all 908
in-coverage rows). The eclipse result strongly indicates the chart's observed
~15–25″ Sun residual is the double-count, not ephemeris-fit error.

## Goals

1. Apply the Sun apparent-place aberration/light-time correction **exactly once**
   in the chart path.
2. Eliminate the duplicated Sun apparent-place math: chart and eclipse share one
   routine in `pleiades-apparent`.
3. Lock the fix in by tightening the Sun golden tolerance.

## Non-goals

- No change to the planet or Moon apparent paths.
- No change to the Moon's 44″ residual (separate ELP/aberration-model matter).
- No new accuracy claims beyond what the tightened gate demonstrates.

## Design

### 1. New shared routine in `pleiades-apparent`

Add a dedicated, self-contained public function alongside `apparent_position`:

```rust
/// Apparent ecliptic-of-date position of the **geocentric Sun**, applying
/// annual aberration exactly once (no light-time re-query). For the Sun,
/// light-time retardation and annual aberration are the same ~20.5″ effect, so
/// re-querying at `t − τ` *and* adding aberration would double-count it.
pub fn apparent_sun_position(
    instant: Instant,
    sun_geocentric_j2000: EclipticCoordinates, // already queried, Mean/J2000
) -> Result<ApparentPosition, ApparentPlaceError>
```

Logic (mirrors `pleiades-eclipse::apparent_sun_longitude_deg`, but returns the
full `ApparentPosition`):

1. Precess J2000 → mean equinox/ecliptic of date → Sun's true longitude of date
   `λ` and latitude `β`.
2. Add nutation in longitude Δψ.
3. Add annual aberration **once**, with `⊙ = λ` (the Sun is its own aberration
   argument). **No** light-time re-query.
4. Distance passes through unchanged (constant over the Sun's own light-time).

It is a pure function: no `query` closure, no `max_iterations`, no external
`sun_lon` argument. Fully unit-testable without a backend.

### 2. Chart call site (`chart/mod.rs` ~308)

Inside the existing `release_grade.contains(&body)` block, branch on
`body == CelestialBody::Sun`:

- **Sun:** query the Sun's Mean/J2000 ecliptic once via `query_mean_ecliptic`
  (geocentric — `observer = None`), then call `apparent_sun_position`.
- **All other release-grade bodies:** unchanged — the existing
  `apparent_position` closure path (light-time re-query + planetary aberration).

The current `Err → Mean` graceful fallback is preserved for both branches: if the
Sun query or correction fails, fall back to the mean position already stored
(matching `release_grade_body_falls_back_to_mean_when_apparent_unavailable`).

The `query_sun_longitude_of_date` / `sun_true_longitude_of_date` plumbing remains
— planets and the Moon still need the Sun's true longitude of date as their
aberration argument. The Sun branch simply no longer consumes it.

### 3. Unify eclipse onto the shared routine

Refactor `crates/pleiades-eclipse/src/ephemeris.rs::apparent_sun_longitude_deg`:

```rust
pub(crate) fn apparent_sun_longitude_deg<B: EphemerisBackend>(
    backend: &B,
    julian_day: f64,
) -> Result<f64, EclipseError> {
    let (lon, lat, dist) = read(backend, CelestialBody::Sun, "Sun", julian_day)?;
    let j2000 = EclipticCoordinates::new(/* lon, lat, Some(dist) */);
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    let apparent = apparent_sun_position(instant, j2000)
        .map_err(|e| EclipseError::Backend(format!("Sun apparent place failed: {e}")))?;
    Ok(apparent.ecliptic.longitude.degrees())
}
```

The duplicated precess/nutation/aberration block in `ephemeris.rs` is deleted.
The existing `validate-eclipses` gate (≤1.0″ on 908 rows) is the safety net
proving the refactor is behaviour-preserving. The crate's existing doc-comments
explaining the once-only aberration are retained / pointed at the shared routine.

### 4. Provenance representation for the Sun

`apparent_sun_position` fills `ApparentProvenance` as:

| field | value | rationale |
|---|---|---|
| `light_time_days` | `0.0` | no light-time iteration performed |
| `iterations` | `0` | same |
| `corrections.light_time` | `false` | honest: no re-query; aberration embodies the effect |
| `corrections.annual_aberration` | `true` | applied once |
| `corrections.precession` | `true` | applied |
| `corrections.nutation_longitude` | `true` | applied |
| `corrections.diurnal_parallax` | `false` | geocentric |
| `corrections.diurnal_aberration` | `false` | geocentric |
| `precession_longitude_arcsec` | `λ − λ_j2000` (wrapped) | as in `apparent_position` |
| `nutation_longitude_arcsec` | `Δψ` | applied value |
| `aberration_longitude_arcsec` | single applied Δλ | the once-only term |
| `model_sources` | `MODEL_SOURCES` | unchanged |

A doc-comment explains why `light_time = false` despite this being an apparent
place (the aberration term *is* the light-time displacement for the Sun).

### 5. Goldens + validation

The Sun longitudes in `apparent-goldens.csv` are independent JPL Horizons Q31
values and **do not change** — only the engine's computed value moves ~20″
closer. Steps:

1. Measure the post-fix Sun residual empirically (run the apparent validation
   gate).
2. Tighten the 5 Sun rows' `tolerance_arcsec` from `26.0` to (max observed
   residual + small margin) — expected single-digit arcsec.
3. Update the CSV header and `regen-apparent-goldens.sh` tolerance rationale:
   remove the "Sun residuals dominated by polynomial-fit error" claim and note
   aberration is applied once for the Sun.
4. Moon and planet rows unchanged.

## Testing

- **Unit** (`pleiades-apparent`): `apparent_sun_position` applies aberration once
  (no ~+20″ double-count vs a known reference), preserves distance, and sets
  provenance flags per §4.
- **Eclipse gate** (`validate-eclipses`): unchanged, proves §3 is
  behaviour-preserving.
- **Apparent gate** (`apparent_validation`): Sun rows now pass at the tightened
  tolerance.
- **Regression**: `release_grade_body_falls_back_to_mean_when_apparent_unavailable`
  still passes (graceful fallback intact).

## Files touched

- `crates/pleiades-apparent/src/apparent.rs` — new `apparent_sun_position` + unit tests.
- `crates/pleiades-core/src/chart/mod.rs` — Sun branch at the apparent call site.
- `crates/pleiades-eclipse/src/ephemeris.rs` — delegate to shared routine.
- `crates/pleiades-validate/data/apparent-goldens.csv` — tighten Sun tolerance + header.
- `crates/pleiades-validate/scripts/regen-apparent-goldens.sh` — tolerance rationale.
- `docs/follow-ups.md` — mark FU-1 resolved.

## Risks

- Touching the release-merged, validated eclipse crate (§3). Mitigated by the
  908-row ≤1.0″ eclipse gate, which fails loudly on any regression.
- Tightened Sun tolerance could be brittle near coverage boundaries (1900 / 2100
  epochs). Mitigated by setting the tolerance from the *observed* max residual
  plus margin rather than an aspirational value.
