# Sidereal Mean Chart Frame Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A sidereal chart placement that stays mean is reported on the mean ecliptic and equinox of date less the ayanamsa, with a Swiss Ephemeris gate behind it (issue #164 (b)).

**Architecture:** Two private helpers in `pleiades-core`'s `chart/sidereal.rs` precess a J2000 mean place to the equinox of date and derive its speed from the backend's own speed; `chart/mod.rs` calls them where it already applies the ayanamsa to a mean placement. A new out-of-workspace tool writes a Swiss Ephemeris geometric sidereal corpus, and a new `validate-sidereal-position` gate in `pleiades-validate` compares mean sidereal charts with it.

**Tech Stack:** Rust (stable, workspace toolchain from `mise.toml`), `pleiades-apparent` precession and motion helpers, Swiss Ephemeris 2.10.03 through `libswisseph-sys` 0.1.2 for the reference tool only.

**Spec:** `docs/superpowers/specs/2026-10-06-sidereal-mean-chart-frame-design.md`

## Global Constraints

- No public type or function signature changes in any `pleiades-*` crate.
- No extra backend reads: `crates/pleiades-core/src/chart/query_count_tests.rs` must pass unchanged.
- Tropical mean charts and all apparent charts must not move: `bit_identity_tests.rs` and the existing `sidereal_tests.rs` apparent tests pass unchanged.
- The reference tool stays outside the Cargo workspace (own `Cargo.lock`, `publish = false`, listed in the root `[workspace].exclude`). The workspace lockfile stays free of `-sys` crates.
- Corpus epochs are TT Julian days ending in `.5`, printed with one decimal.
- Longitude and latitude ceilings: `ceil(1.4 × measured max)` in whole arcseconds. Speed ceilings: `1.4 × measured max` rounded up to two significant figures, because a whole-arcsecond speed ceiling would hide the 0.14″/day this change is about.
- Do not touch any other gate's corpus, ceilings or tolerances.
- Do not bump versions or edit `CHANGELOG.md`; release-plz does both.
- Code is `rustfmt`-clean and passes `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- This worktree session's Bash refuses heredocs, loops and `/usr/bin/time`. Create and edit files with the Write and Edit tools and run plain commands.
- Do not edit source or commit while a background test run is in progress.

## Review Focus

1. **A placement with no speed.** A backend that reports no motion, or no longitude speed, must still get a precessed place and must keep its missing channels missing. Pinned in Task 2 (`a_missing_speed_channel_stays_missing`).
2. **A body near 0° of longitude.** Precession to 1913 carries a J2000 longitude of 0.2° back past 360°. The place must wrap and the speed must not jump by 360°/day. Pinned in Tasks 1 and 2 (`a_place_near_the_equinox_precesses_across_the_zero_of_longitude`, `a_speed_across_the_zero_of_longitude_is_continuous`).
3. **A body at high ecliptic latitude.** The precession shift in longitude depends on latitude; Pluto (17°) and the Moon (5°) are in the formula test of Task 1 and in the corpus of Task 4.
4. **A chart at the edge of the backend's range.** The speed must not need a neighbouring backend read, which would fail at the window's first instant. Pinned in Task 2 (`a_sidereal_mean_chart_at_the_window_start_has_a_speed`).
5. **A mean fallback inside an apparent sidereal chart.** It must report the same place as a requested mean chart. Pinned in Task 1 (`a_mean_fallback_in_a_sidereal_chart_reports_the_requested_mean_place`).

Not covered and left as found: an apparent **topocentric** sidereal chart applies the topocentric correction to a fallback body's already-sidereal longitude. That predates this change and is outside the spec.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/pleiades-core/src/chart/sidereal.rs` (modify) | Gains `mean_place_of_date` and `mean_motion_of_date`, the only new production code. |
| `crates/pleiades-core/src/chart/mod.rs` (modify) | Calls the two helpers in the existing mean-placement sidereal steps; declares the new test module. |
| `crates/pleiades-core/src/chart/sidereal_mean_tests.rs` (create) | All tests of the sidereal mean frame. |
| `crates/pleiades-core/src/chart/sidereal_tests.rs` (modify) | Shares its helpers with the new test file; loses the one test whose premise changes. |
| `crates/pleiades-core/src/chart/house_placement_tests.rs` (modify) | One pin for sidereal mean houses. |
| `crates/pleiades-events/tests/reference.rs` (modify) | The ignored diagnostic becomes an asserting cross-crate test. |
| `crates/pleiades-core/src/chart/request.rs`, `crates/pleiades-core/README.md`, `crates/pleiades-events/README.md`, `docs/cli.md` (modify) | Say which equinox each combination is on. |
| `tools/se-sidereal-position-reference/` (create) | Swiss Ephemeris reference tool: `Cargo.toml`, `Cargo.lock`, `LICENSE-NOTES.md`, `src/main.rs`. |
| `Cargo.toml` (modify) | Adds the tool to `[workspace].exclude`. |
| `crates/pleiades-validate/data/sidereal-position-corpus/` (create) | `sidereal-position.csv`, `manifest.txt`. |
| `crates/pleiades-validate/src/sidereal_position_validation.rs` (create) | The gate. |
| `crates/pleiades-validate/src/sidereal_position_thresholds.rs` (create) | Its measured ceilings. |
| `crates/pleiades-validate/src/sidereal_position_validation/tests.rs` (create) | Gate tests. |
| `crates/pleiades-validate/src/lib.rs`, `crates/pleiades-validate/src/render/cli.rs`, `crates/pleiades-validate/src/tests/validate_gates.rs`, `crates/pleiades-cli/src/cli.rs`, `docs/status.md` (modify) | Register, expose and document the gate. |

---

### Task 1: Sidereal mean place on the mean equinox of date

**Files:**
- Modify: `crates/pleiades-core/src/chart/sidereal.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs` (test module list near line 34; pre-apparent sidereal block near lines 461–479)
- Modify: `crates/pleiades-core/src/chart/sidereal_tests.rs` (helper visibility only)
- Modify: `crates/pleiades-core/src/chart/house_placement_tests.rs`
- Create: `crates/pleiades-core/src/chart/sidereal_mean_tests.rs`

**Interfaces:**
- Consumes: `pleiades_apparent::precess_ecliptic_j2000_to_date(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> Result<PrecessedEcliptic, ApparentPlaceError>` (fields `longitude_deg`, `latitude_deg`); `map_apparent_place_error(ApparentPlaceError) -> EphemerisError` in `chart/mod.rs`.
- Produces: `pub(super) fn mean_place_of_date(j2000: EclipticCoordinates, julian_day: f64) -> Result<EclipticCoordinates, EphemerisError>` in `chart/sidereal.rs`. Test helpers `composite_backend`, `lahiri`, `tt`, `wrap_arcsec`, `lahiri_deg`, `speed_deg_per_day`, `rate` become `pub(super)` in `sidereal_tests.rs`. The new test file defines, privately, `EPOCHS_JD_TT: [f64; 3]`, `bodies() -> Vec<CelestialBody>`, `mean_charts(jd_tt: f64, bodies: Vec<CelestialBody>) -> (ChartSnapshot, ChartSnapshot)` (tropical, sidereal), `ecliptic(&ChartSnapshot, &CelestialBody) -> EclipticCoordinates` and `general_precession_arcsec(jd_tt: f64) -> f64`; Task 2 adds tests to the same file and uses them.

- [ ] **Step 1: Share the existing test helpers**

In `crates/pleiades-core/src/chart/sidereal_tests.rs`, change these seven definitions from `fn` to `pub(super) fn` and nothing else: `composite_backend`, `lahiri`, `wrap_arcsec`, `tt`, `speed_deg_per_day`, `lahiri_deg`, `rate`. For example:

```rust
pub(super) fn composite_backend() -> CompositeBackend<Vsop87Backend, ElpBackend> {
    CompositeBackend::new(Vsop87Backend::new(), ElpBackend::new())
}
```

- [ ] **Step 2: Declare the new test module**

In `crates/pleiades-core/src/chart/mod.rs`, directly above the existing

```rust
#[cfg(test)]
mod sidereal_tests;
```

add

```rust
#[cfg(test)]
mod sidereal_mean_tests;
```

- [ ] **Step 3: Write the failing tests**

Create `crates/pleiades-core/src/chart/sidereal_mean_tests.rs`:

```rust
//! A sidereal placement the chart leaves mean is on the mean ecliptic and
//! equinox of date, less the mean ayanamsa (issue #164 (b)).
//!
//! The backends report a mean place on the J2000 equinox. An ayanamsa is the
//! distance from the sidereal zero point to the equinox of date, so before
//! this fix a sidereal mean longitude kept the precession accumulated since
//! J2000: 1.2 degrees at 1913.

use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude};

use super::sidereal::mean_place_of_date;
use super::sidereal_tests::{composite_backend, lahiri, lahiri_deg, tt, wrap_arcsec};
use super::test_support::AbsurdDistanceReleaseGradeBackend;
use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// 1913, J2000 and 2077: near both ends of the supported window, and the
/// epoch at which the two equinoxes coincide.
const EPOCHS_JD_TT: [f64; 3] = [2_420_000.5, 2_451_545.0, 2_480_000.5];

/// A body on the ecliptic, the Moon at 5 degrees of latitude, a slow planet,
/// Pluto at 17 degrees, and a lunar point.
fn bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
        CelestialBody::MeanNode,
    ]
}

/// The tropical and the Lahiri mean chart of `bodies` at `jd_tt`.
fn mean_charts(jd_tt: f64, bodies: Vec<CelestialBody>) -> (ChartSnapshot, ChartSnapshot) {
    let engine = ChartEngine::new(composite_backend());
    let request = ChartRequest::new(tt(jd_tt))
        .with_bodies(bodies)
        .with_apparentness(Apparentness::Mean);
    let tropical = engine.chart(&request).expect("tropical mean chart");
    let sidereal = engine
        .chart(&request.with_zodiac_mode(lahiri()))
        .expect("sidereal mean chart");
    (tropical, sidereal)
}

fn ecliptic(snapshot: &ChartSnapshot, body: &CelestialBody) -> EclipticCoordinates {
    snapshot
        .placement_for(body)
        .expect("body is placed")
        .position
        .ecliptic
        .expect("ecliptic coordinates")
}

/// General precession in longitude accumulated since J2000, in arcseconds:
/// Lieske et al. 1977 (IAU 1976), `p_A = 5029.0966″ T + 1.11113″ T² −
/// 0.000006″ T³`, with `T` in Julian centuries of TT from J2000.
fn general_precession_arcsec(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    5029.0966 * t + 1.11113 * t * t - 0.000_006 * t * t * t
}

#[test]
fn a_sidereal_mean_place_is_the_precessed_j2000_place_less_the_ayanamsa() {
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, bodies());
        for body in bodies() {
            // A tropical mean chart reports the backend's J2000 place.
            let j2000 = ecliptic(&tropical, &body);
            let of_date = precess_ecliptic_j2000_to_date(
                j2000.longitude.degrees(),
                j2000.latitude.degrees(),
                jd,
            )
            .expect("precession");
            let got = ecliptic(&sidereal, &body);
            let longitude_off =
                wrap_arcsec(got.longitude.degrees() - (of_date.longitude_deg - lahiri_deg(jd)));
            let latitude_off = (got.latitude.degrees() - of_date.latitude_deg) * 3600.0;
            assert!(
                longitude_off.abs() < 1e-6,
                "{body:?} at {jd}: longitude is {longitude_off}″ off"
            );
            assert!(
                latitude_off.abs() < 1e-6,
                "{body:?} at {jd}: latitude is {latitude_off}″ off"
            );
            assert_eq!(got.distance_au, j2000.distance_au, "{body:?} at {jd}");
        }
    }
}

// An anchor that does not come from the code under test: the Sun stays on
// the ecliptic, so the equinox of date differs from the J2000 one by the
// general precession in longitude and nothing else.
#[test]
fn the_suns_sidereal_mean_longitude_carries_the_general_precession() {
    let sun = CelestialBody::Sun;
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, vec![sun.clone()]);
        let shift = wrap_arcsec(
            ecliptic(&sidereal, &sun).longitude.degrees() + lahiri_deg(jd)
                - ecliptic(&tropical, &sun).longitude.degrees(),
        );
        let want = general_precession_arcsec(jd);
        assert!(
            (shift - want).abs() < 0.05,
            "at {jd}: the equinox moved {shift}″, general precession is {want}″"
        );
    }
    // 86 years before J2000 that is well over a degree.
    assert!(general_precession_arcsec(EPOCHS_JD_TT[0]) < -4300.0);
}

// The ecliptic of date is tilted against the J2000 one by 47″ per century,
// so a latitude changes by at most that tilt, and by how much depends on
// where the body is along the ecliptic.
#[test]
fn a_sidereal_mean_latitude_is_on_the_ecliptic_of_date() {
    let jd = EPOCHS_JD_TT[0];
    let (tropical, sidereal) = mean_charts(jd, bodies());
    let tilt = 47.1 * (jd - 2_451_545.0).abs() / 36_525.0;
    let mut largest = 0.0_f64;
    for body in bodies() {
        let moved = (ecliptic(&sidereal, &body).latitude.degrees()
            - ecliptic(&tropical, &body).latitude.degrees())
            * 3600.0;
        assert!(moved.abs() < tilt, "{body:?}: latitude moved {moved}″");
        largest = largest.max(moved.abs());
    }
    // Five bodies spread around the ecliptic cannot all sit on the line the
    // two ecliptics cross along.
    assert!(largest > 5.0, "no latitude moved more than {largest}″");
}

#[test]
fn a_place_near_the_equinox_precesses_across_the_zero_of_longitude() {
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(0.2),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    let of_date = mean_place_of_date(j2000, EPOCHS_JD_TT[0]).expect("precession");
    // 0.2° less 1.206° of precession.
    let longitude = of_date.longitude.degrees();
    assert!((358.9..359.1).contains(&longitude), "{longitude}");
    assert_eq!(of_date.distance_au, Some(1.0));
    // At J2000 the two equinoxes are the same.
    let unchanged = mean_place_of_date(j2000, 2_451_545.0).expect("precession");
    assert!((unchanged.longitude.degrees() - 0.2).abs() < 1e-9);
}

// Mars falls back to its mean place in the apparent chart (its distance
// trips the light-time sanity cap). The fallback and a requested mean chart
// go through one step, and must keep doing so.
#[test]
fn a_mean_fallback_in_a_sidereal_chart_reports_the_requested_mean_place() {
    let chart = |apparentness: Apparentness| {
        ChartEngine::new(AbsurdDistanceReleaseGradeBackend)
            .chart(
                &ChartRequest::new(tt(EPOCHS_JD_TT[0]))
                    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
                    .with_apparentness(apparentness)
                    .with_zodiac_mode(lahiri()),
            )
            .expect("chart succeeds")
    };
    let apparent = chart(Apparentness::Apparent);
    let mean = chart(Apparentness::Mean);
    let mars = CelestialBody::Mars;
    assert!(
        apparent
            .mean_fallback_placements()
            .any(|placement| placement.body == mars),
        "Mars must fall back for this test to mean anything"
    );
    let (fallback, requested) = (ecliptic(&apparent, &mars), ecliptic(&mean, &mars));
    assert_eq!(
        fallback.longitude.degrees(),
        requested.longitude.degrees()
    );
    assert_eq!(fallback.latitude.degrees(), requested.latitude.degrees());
}
```

In `crates/pleiades-core/src/chart/house_placement_tests.rs`, directly after `mean_houses_follow_the_reported_longitude`, add:

```rust
#[test]
fn sidereal_mean_houses_follow_the_reported_longitude() {
    let charts = sweep(|request| {
        request
            .with_apparentness(Apparentness::Mean)
            .with_zodiac_mode(ZodiacMode::Sidereal {
                ayanamsa: Ayanamsa::Lahiri,
            })
    });
    assert_eq!(assert_houses_match_reported_longitudes(&charts), 1260);
}
```

- [ ] **Step 4: Run the tests to verify they fail**

Run: `cargo test -p pleiades-core --lib sidereal_mean`
Expected: a compile error, `cannot find function mean_place_of_date in module super::sidereal`.

- [ ] **Step 5: Add the helper**

In `crates/pleiades-core/src/chart/sidereal.rs`, replace the three `use` lines at the top with:

```rust
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude, ZodiacMode};
```

and append to the end of the file:

```rust
/// The mean J2000 place `j2000` on the mean ecliptic and equinox of date
/// `julian_day` (TT). The distance passes through.
///
/// The backends report a mean place on the J2000 equinox, and an ayanamsa is
/// counted from the equinox of date, so a mean place takes this step before
/// [`sidereal_longitude`] (issue #164). It is the IAU 1976 precession the
/// apparent reduction and `pleiades-events`' mean-of-date frame use.
pub(super) fn mean_place_of_date(
    j2000: EclipticCoordinates,
    julian_day: f64,
) -> Result<EclipticCoordinates, EphemerisError> {
    let of_date = precess_ecliptic_j2000_to_date(
        j2000.longitude.degrees(),
        j2000.latitude.degrees(),
        julian_day,
    )
    .map_err(super::map_apparent_place_error)?;
    Ok(EclipticCoordinates::new(
        Longitude::from_degrees(of_date.longitude_deg),
        Latitude::from_degrees(of_date.latitude_deg),
        j2000.distance_au,
    ))
}
```

Also extend the doc comment of `sidereal_longitude` in the same file. Replace

```rust
/// resolved ayanamsa for the provided instant. The longitude is taken to be
/// on the mean equinox; the chart layer moves an apparent (true-equinox)
/// longitude to the mean equinox first, so nutation does not move a body
/// through a sidereal zodiac (issue #120).
```

with

```rust
/// resolved ayanamsa for the provided instant. The longitude is taken to be
/// on the mean equinox of date. The chart layer brings its own longitudes
/// there first: an apparent (true-equinox) one has nutation removed, so
/// nutation does not move a body through a sidereal zodiac (issue #120), and
/// a mean one, which the backends report on the J2000 equinox, is precessed
/// to the equinox of date (issue #164).
```

- [ ] **Step 6: Call it where a mean placement becomes sidereal**

In `crates/pleiades-core/src/chart/mod.rs`, replace

```rust
                    let instant = position.instant;
                    let longitude = position.ecliptic.as_mut().map(|coords| &mut coords.longitude);
                    let longitude = longitude.ok_or_else(|| {
                        EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            "sidereal chart assembly requires ecliptic coordinates from the backend",
                        )
                    })?;
                    *longitude = sidereal_longitude(*longitude, instant, &request.zodiac_mode)?;
                    Some(pleiades_types::ZodiacSign::from_longitude(*longitude))
```

with

```rust
                    let instant = position.instant;
                    let ecliptic = position.ecliptic.as_mut().ok_or_else(|| {
                        EphemerisError::new(
                            EphemerisErrorKind::InvalidRequest,
                            "sidereal chart assembly requires ecliptic coordinates from the backend",
                        )
                    })?;
                    // The backend's place is on the J2000 equinox and the
                    // ayanamsa is counted from the equinox of date, so the
                    // place is precessed there first (issue #164). A
                    // successful apparent reduction below replaces it.
                    *ecliptic =
                        sidereal::mean_place_of_date(*ecliptic, instant.julian_day.days())?;
                    ecliptic.longitude =
                        sidereal_longitude(ecliptic.longitude, instant, &request.zodiac_mode)?;
                    Some(pleiades_types::ZodiacSign::from_longitude(ecliptic.longitude))
```

- [ ] **Step 7: Run the tests to verify they pass**

Run: `cargo test -p pleiades-core --lib sidereal_mean`
Expected: 5 passed.

Run: `cargo test -p pleiades-core --lib chart::`
Expected: all pass except `sidereal_mean_speed_drops_by_the_rate_of_the_ayanamsa`, which may still pass here and is replaced in Task 2. `sidereal_mean_houses_follow_the_reported_longitude`, the `bit_identity_tests` and the `query_count_tests` pass.

If `a_mean_fallback_in_a_sidereal_chart_reports_the_requested_mean_place` fails on its first assertion (Mars did not fall back at this epoch), change its epoch to `2_451_545.0`, the epoch `apparentness_applied_tests.rs` uses, and keep the rest.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core/src/chart
git commit -m "fix(core): put a sidereal mean placement on the mean equinox of date (#164)"
```

---

### Task 2: Speed of a sidereal mean placement

**Files:**
- Modify: `crates/pleiades-core/src/chart/sidereal.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs` (mean speed block near lines 533–549)
- Modify: `crates/pleiades-core/src/chart/sidereal_tests.rs` (delete one test)
- Modify: `crates/pleiades-core/src/chart/sidereal_mean_tests.rs`

**Interfaces:**
- Consumes: `mean_place_of_date` (Task 1); `pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS}`, where `Correction::between(corrected: &EclipticCoordinates, base: &EclipticCoordinates) -> Correction`, `CorrectionSample { julian_day: f64, correction: Correction }`, `apparent_motion(base: Motion, earlier: &CorrectionSample, later: &CorrectionSample) -> Motion`, `HALF_SPAN_DAYS = 0.5`; test helpers `EPOCHS_JD_TT`, `mean_charts`, `ecliptic` from Task 1 and `speed_deg_per_day`, `rate`, `lahiri_deg` from `sidereal_tests.rs`.
- Produces: `pub(super) fn mean_motion_of_date(j2000: EclipticCoordinates, base: Motion, julian_day: f64) -> Result<Motion, EphemerisError>` in `chart/sidereal.rs`.

- [ ] **Step 1: Write the failing tests**

In `crates/pleiades-core/src/chart/sidereal_mean_tests.rs`, replace the import block

```rust
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude};

use super::sidereal::mean_place_of_date;
use super::sidereal_tests::{composite_backend, lahiri, lahiri_deg, tt, wrap_arcsec};
```

with

```rust
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::Apparentness;
use pleiades_data::packaged_backend;
use pleiades_types::{CelestialBody, EclipticCoordinates, Latitude, Longitude, Motion};

use super::sidereal::{mean_motion_of_date, mean_place_of_date};
use super::sidereal_tests::{
    composite_backend, lahiri, lahiri_deg, rate, speed_deg_per_day, tt, wrap_arcsec,
};
```

and append:

```rust
/// Bodies the composite backend reports a longitude speed for.
fn moving_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ]
}

/// Rate of [`general_precession_arcsec`], in degrees per day.
fn general_precession_rate_deg_per_day(jd_tt: f64) -> f64 {
    let t = (jd_tt - 2_451_545.0) / 36_525.0;
    (5029.0966 + 2.0 * 1.11113 * t - 3.0 * 0.000_006 * t * t) / 3600.0 / 36_525.0
}

// The Sun stays on the ecliptic, so its longitude on the equinox of date
// moves faster than its J2000 longitude by the precession rate, 0.138″ a
// day, and the ayanamsa takes its own rate off. Before the fix only the
// ayanamsa's rate came off, and the speed was 3.8e-5 deg/day too low.
#[test]
fn the_suns_sidereal_mean_speed_gains_the_precession_rate_and_loses_the_ayanamsas() {
    let sun = CelestialBody::Sun;
    for jd in EPOCHS_JD_TT {
        let (tropical, sidereal) = mean_charts(jd, vec![sun.clone()]);
        let gain = speed_deg_per_day(&sidereal, &sun) - speed_deg_per_day(&tropical, &sun);
        let want = general_precession_rate_deg_per_day(jd) - rate(lahiri_deg, jd);
        assert!(
            (gain - want).abs() < 5e-7,
            "at {jd}: sidereal − tropical speed is {gain}, expected {want} deg/day"
        );
    }
    assert!(general_precession_rate_deg_per_day(2_451_545.0) > 3.8e-5);
}

// For every body the reported speed is the rate of the reported longitude.
// Off the ecliptic the precession step also depends on where the body is, so
// the Moon's speed changes by up to 0.8″ a day at 1913; a difference of the
// charts' own longitudes a day apart sees that too. The tolerance covers the
// truncation of that difference for the Moon (5e-7 deg/day).
#[test]
fn a_sidereal_mean_speed_is_the_rate_of_the_reported_longitude() {
    let jd = EPOCHS_JD_TT[0];
    let step_longitudes = |jd_tt: f64| {
        let (tropical, sidereal) = mean_charts(jd_tt, moving_bodies());
        moving_bodies()
            .into_iter()
            .map(|body| {
                ecliptic(&sidereal, &body).longitude.degrees()
                    - ecliptic(&tropical, &body).longitude.degrees()
            })
            .collect::<Vec<f64>>()
    };
    let (earlier, later) = (step_longitudes(jd - 0.5), step_longitudes(jd + 0.5));
    let (tropical, sidereal) = mean_charts(jd, moving_bodies());
    for (index, body) in moving_bodies().into_iter().enumerate() {
        let gain = speed_deg_per_day(&sidereal, &body) - speed_deg_per_day(&tropical, &body);
        let want = wrap_arcsec(later[index] - earlier[index]) / 3600.0;
        assert!(
            (gain - want).abs() < 2e-6,
            "{body:?}: sidereal − tropical speed is {gain}, the step's rate is {want} deg/day"
        );
    }
}

#[test]
fn a_missing_speed_channel_stays_missing() {
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(120.0),
        Latitude::from_degrees(4.0),
        Some(1.0),
    );
    let jd = EPOCHS_JD_TT[0];
    let none = mean_motion_of_date(j2000, Motion::new(None, None, None), jd).expect("motion");
    assert_eq!(none, Motion::new(None, None, None));
    let latitude_only =
        mean_motion_of_date(j2000, Motion::new(None, Some(0.01), None), jd).expect("motion");
    assert_eq!(latitude_only.longitude_deg_per_day, None);
    assert_eq!(latitude_only.distance_au_per_day, None);
    let latitude_speed = latitude_only.latitude_deg_per_day.expect("latitude speed");
    assert!((latitude_speed - 0.01).abs() < 1e-6, "{latitude_speed}");
    // A distance speed is not a matter of frame.
    let with_distance =
        mean_motion_of_date(j2000, Motion::new(Some(1.0), Some(0.0), Some(1e-4)), jd)
            .expect("motion");
    assert_eq!(with_distance.distance_au_per_day, Some(1e-4));
}

#[test]
fn a_speed_across_the_zero_of_longitude_is_continuous() {
    // The place precesses from 0.2° back past 360° at 1913 (Task 1's test).
    let j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(0.2),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    let jd = EPOCHS_JD_TT[0];
    let motion = mean_motion_of_date(j2000, Motion::new(Some(1.0), Some(0.0), None), jd)
        .expect("motion");
    let speed = motion.longitude_deg_per_day.expect("longitude speed");
    let want = 1.0 + general_precession_rate_deg_per_day(jd);
    assert!((speed - want).abs() < 2e-6, "{speed} vs {want}");
}

// The speed needs no neighbouring backend read, so the first instant the
// packaged backend serves has one.
#[test]
fn a_sidereal_mean_chart_at_the_window_start_has_a_speed() {
    let request = ChartRequest::new(tt(2_415_020.5))
        .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
        .with_apparentness(Apparentness::Mean)
        .with_zodiac_mode(lahiri());
    let chart = ChartEngine::new(packaged_backend())
        .chart(&request)
        .expect("chart at the window start");
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        assert!(speed_deg_per_day(&chart, &body) > 0.9, "{body:?}");
    }
}
```

If `Motion` does not implement `PartialEq`, the first `assert_eq!` in `a_missing_speed_channel_stays_missing` will not compile; compare its three channels with `None` one by one instead.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-core --lib sidereal_mean`
Expected: a compile error, `cannot find function mean_motion_of_date in module super::sidereal`.

- [ ] **Step 3: Add the helper**

In `crates/pleiades-core/src/chart/sidereal.rs`, replace the `use` block at the top with:

```rust
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::{EphemerisError, EphemerisErrorKind};
use pleiades_types::{EclipticCoordinates, Instant, Latitude, Longitude, Motion, ZodiacMode};
```

and append to the end of the file:

```rust
/// Speed of [`mean_place_of_date`] at `julian_day`, from `base`, the speed of
/// the J2000 place `j2000` itself.
///
/// The precession step is differenced centrally over ±[`HALF_SPAN_DAYS`] and
/// added to `base`. Its two samples are taken at the J2000 place carried
/// along its own speed, so no backend read is needed and a chart at the edge
/// of a backend's range still gets a speed. A channel `base` leaves empty
/// stays empty, and the distance speed is unchanged.
pub(super) fn mean_motion_of_date(
    j2000: EclipticCoordinates,
    base: Motion,
    julian_day: f64,
) -> Result<Motion, EphemerisError> {
    let sample = |offset_days: f64| -> Result<CorrectionSample, EphemerisError> {
        let carried = EclipticCoordinates::new(
            Longitude::from_degrees(
                j2000.longitude.degrees()
                    + base.longitude_deg_per_day.unwrap_or(0.0) * offset_days,
            ),
            Latitude::from_degrees(
                j2000.latitude.degrees() + base.latitude_deg_per_day.unwrap_or(0.0) * offset_days,
            ),
            j2000.distance_au,
        );
        let of_date = mean_place_of_date(carried, julian_day + offset_days)?;
        Ok(CorrectionSample {
            julian_day: julian_day + offset_days,
            correction: Correction::between(&of_date, &carried),
        })
    };
    Ok(apparent_motion(
        base,
        &sample(-HALF_SPAN_DAYS)?,
        &sample(HALF_SPAN_DAYS)?,
    ))
}
```

- [ ] **Step 4: Use it for a mean placement's speed**

In `crates/pleiades-core/src/chart/mod.rs`, replace

```rust
                // A mean placement (requested, or the fallback above) reports the
                // backend's longitude less the ayanamsa, so its speed is the
                // backend's less the ayanamsa's rate (issue #141). An apparent
                // placement's speed took the sidereal step in `apparent_motion`.
                if let (None, Some(zodiac_mode)) = (&apparent, chart_sidereal_mode) {
                    if let Some(speed) = position
```

with

```rust
                // A mean placement (requested, or the fallback above) reports the
                // backend's place precessed to the equinox of date, less the
                // ayanamsa. Its speed is the backend's plus the rate of that
                // precession step (issue #164), less the ayanamsa's rate
                // (issue #141). An apparent placement's speed took the sidereal
                // step in `apparent_motion`.
                if let (None, Some(zodiac_mode)) = (&apparent, chart_sidereal_mode) {
                    // `batch_mean` is the J2000 place the backend's speed describes.
                    if let (Some(j2000), Some(base)) = (batch_mean, position.motion) {
                        position.motion = Some(sidereal::mean_motion_of_date(
                            j2000,
                            base,
                            request.instant.julian_day.days(),
                        )?);
                    }
                    if let Some(speed) = position
```

Leave the rest of that block (the `ayanamsa_rate_deg_per_day` subtraction) as it is.

- [ ] **Step 5: Remove the test whose premise changed**

In `crates/pleiades-core/src/chart/sidereal_tests.rs`, delete the whole test `sidereal_mean_speed_drops_by_the_rate_of_the_ayanamsa` (its `#[test]` attribute, signature and body). The tests added in Step 1 replace it. If `issue_141_charts` or `issue_141_bodies` then has no remaining caller in that file, clippy will say so in Step 6; they are still used by the two apparent speed tests, so no further change is expected.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p pleiades-core --lib chart::`
Expected: all pass, including the ten `sidereal_mean` tests and the unchanged `query_count_tests` and `bit_identity_tests`.

Run: `cargo clippy -p pleiades-core --all-targets --all-features -- -D warnings`
Expected: no warnings.

If `a_sidereal_mean_speed_is_the_rate_of_the_reported_longitude` fails with a difference above 2e-6 deg/day, do not widen the tolerance: the speed is then not the rate of the reported longitude, and `mean_motion_of_date` or its call site is wrong.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core/src/chart
git commit -m "fix(core): give a sidereal mean placement the speed of its place of date (#164)"
```

---

### Task 3: Cross-crate agreement and documentation

**Files:**
- Modify: `crates/pleiades-events/tests/reference.rs` (the ignored test `measure_chart_sidereal_conventions`, near line 753)
- Modify: `crates/pleiades-core/src/chart/request.rs` (`with_zodiac_mode`, `with_apparentness`)
- Modify: `crates/pleiades-core/README.md`, `crates/pleiades-events/README.md`, `docs/cli.md`

**Interfaces:**
- Consumes: the chart behaviour of Tasks 1 and 2; `EventEngine::position_at(body, reference, instant)` returning a position with `ecliptic: EclipticCoordinates` and `motion: Motion`; the file's existing helpers `lahiri(frame)`, `tdb(jd)`, `wrap(deg)` and constant `MEAN`.
- Produces: nothing later tasks use.

- [ ] **Step 1: Turn the diagnostic into a test**

In `crates/pleiades-events/tests/reference.rs`, replace the whole item that starts with the doc comment `/// Diagnostic for issue #164 (b): how a sidereal mean chart differs from a` and ends with the closing brace of `fn measure_chart_sidereal_conventions()` with:

```rust
/// Issue #164 (b): a sidereal mean chart reports the place this crate's
/// mean-of-date sidereal reference reports. Before the fix the chart
/// subtracted the ayanamsa from a J2000 longitude and the two differed by the
/// precession since J2000: 4342.5″ at the first epoch here.
#[test]
fn a_sidereal_mean_chart_reports_the_mean_of_date_sidereal_place() {
    let engine = EventEngine::new(packaged_backend());
    let charts = ChartEngine::new(packaged_backend());
    let zodiac = ZodiacMode::Sidereal {
        ayanamsa: Ayanamsa::Lahiri,
    };
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Saturn,
    ];
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        let request = ChartRequest::new(tdb(jd))
            .with_bodies(bodies.to_vec())
            .with_apparentness(Apparentness::Mean)
            .with_zodiac_mode(zodiac.clone());
        let chart = charts.chart(&request).expect("chart");
        for body in &bodies {
            let placed = &chart.placement_for(body).expect("placed").position;
            let ecliptic = placed.ecliptic.as_ref().expect("ecliptic");
            let want = engine
                .position_at(body.clone(), lahiri(MEAN), tdb(jd))
                .expect("position");
            let longitude_off =
                wrap(ecliptic.longitude.degrees() - want.ecliptic.longitude.degrees()) * 3600.0;
            let latitude_off =
                (ecliptic.latitude.degrees() - want.ecliptic.latitude.degrees()) * 3600.0;
            assert!(
                longitude_off.abs() < 1e-6 && latitude_off.abs() < 1e-6,
                "{body:?} at {jd}: chart − events is {longitude_off}″ in longitude, {latitude_off}″ in latitude"
            );
            // The chart carries the J2000 place along its own speed where
            // this crate reads the backend again, so the speeds agree
            // closely, not to the bit.
            let speed = placed
                .motion
                .and_then(|motion| motion.longitude_deg_per_day)
                .expect("chart speed");
            let want_speed = want.motion.longitude_deg_per_day.expect("events speed");
            assert!(
                (speed - want_speed).abs() < 1e-6,
                "{body:?} at {jd}: chart speed {speed}, events speed {want_speed} deg/day"
            );
        }
    }
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p pleiades-events --test reference a_sidereal_mean_chart`
Expected: 1 passed. (On `main`, without Tasks 1 and 2, the same test fails at the first epoch with `chart − events is 4342.5…″ in longitude`.)

Run: `cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings`
Expected: no warnings. If `nutation` or `APPARENT` is now reported unused in this file, remove it from the imports or constants; both are expected to keep other users.

- [ ] **Step 3: Document the frames on the request builders**

In `crates/pleiades-core/src/chart/request.rs`, replace

```rust
    /// Sets the zodiac mode.
    pub fn with_zodiac_mode(mut self, zodiac_mode: ZodiacMode) -> Self {
```

with

```rust
    /// Sets the zodiac mode.
    ///
    /// A sidereal longitude is the longitude on the mean equinox of date
    /// less the mean ayanamsa, whatever the apparentness. An apparent place
    /// has nutation in longitude removed first. A mean place, which the
    /// backends report on the J2000 equinox, is precessed to the equinox of
    /// date first, and its latitude and speed are those of that place.
    pub fn with_zodiac_mode(mut self, zodiac_mode: ZodiacMode) -> Self {
```

and replace

```rust
    /// Sets the preferred apparentness.
    pub fn with_apparentness(mut self, apparentness: Apparentness) -> Self {
```

with

```rust
    /// Sets the preferred apparentness.
    ///
    /// `Apparent`, the default, is the apparent place on the true equinox of
    /// date. `Mean` is the backend's geometric place: on the J2000 equinox
    /// in the tropical zodiac, and on the mean equinox of date in a sidereal
    /// one (see [`ChartRequest::with_zodiac_mode`]). House cusps and angles
    /// are on the equinox of date in every chart.
    pub fn with_apparentness(mut self, apparentness: Apparentness) -> Self {
```

- [ ] **Step 4: Update the READMEs and the CLI guide**

In `crates/pleiades-core/README.md`, in the paragraph that begins `Experimental, pre-1.0:`, insert this sentence directly before `See the [workspace README]`:

```markdown
A mean chart (`Apparentness::Mean`) reports that geometric place on the J2000 equinox in the tropical zodiac; in a sidereal zodiac it is precessed to the mean equinox of date before the ayanamsa comes off, as Swiss Ephemeris and `pleiades-events` do.
```

In `crates/pleiades-events/README.md`, replace

```markdown
  date. This is not the J2000 longitude a `pleiades-core` mean chart reports.
```

with

```markdown
  date. This is not the J2000 longitude a `pleiades-core` tropical mean chart
  reports. In a sidereal zodiac the two agree: a sidereal mean chart is
  precessed to the equinox of date before the ayanamsa comes off.
```

In `docs/cli.md`, in the `Notes:` list that follows the `chart ... --mean` example, add this item as the first bullet:

```markdown
- `--mean` reports the geometric place: on the J2000 equinox in the tropical zodiac, and on the mean equinox of date less the ayanamsa when `--ayanamsa` is given.
```

- [ ] **Step 5: Check the docs build**

Run: `mise run docs`
Expected: finishes with `Generated ...` and no warnings (the task denies rustdoc warnings).

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates/pleiades-events/tests/reference.rs crates/pleiades-core/src/chart/request.rs crates/pleiades-core/README.md crates/pleiades-events/README.md docs/cli.md
git commit -m "test(events): hold a sidereal mean chart to the mean-of-date sidereal place; document the frames (#164)"
```

---

### Task 4: Swiss Ephemeris reference tool and corpus

**Files:**
- Create: `tools/se-sidereal-position-reference/Cargo.toml`
- Create: `tools/se-sidereal-position-reference/LICENSE-NOTES.md`
- Create: `tools/se-sidereal-position-reference/src/main.rs`
- Create (generated): `tools/se-sidereal-position-reference/Cargo.lock`
- Modify: `Cargo.toml` (`[workspace].exclude`)
- Create (generated): `crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`
- Create: `crates/pleiades-validate/data/sidereal-position-corpus/manifest.txt`

**Interfaces:**
- Consumes: nothing from earlier tasks.
- Produces: a CSV whose data rows are `jd_tt,ayanamsa,body,lon_deg,lat_deg,lon_speed_deg_per_day` (6 fields), with comment lines starting `#` and one header line starting `jd_tt`. Ayanamsa names: `Lahiri`, `TrueCitra`, `GalacticCenter`, `DeLuce`. Body names: `Sun`, `Moon`, `Mercury`, `Venus`, `Mars`, `Jupiter`, `Saturn`, `Uranus`, `Neptune`, `Pluto`. 67 epochs × 4 ayanamsas × 10 bodies = 2680 rows. A manifest line `slice sidereal-position file=sidereal-position.csv role=sidereal-position rows=2680 checksum=<fnv1a64 of the CSV>`.

- [ ] **Step 1: Create the tool's manifest**

Create `tools/se-sidereal-position-reference/Cargo.toml`:

```toml
[package]
name = "se-sidereal-position-reference"
version = "0.0.0"
edition = "2021"
publish = false

# Standalone: this tool is excluded from the root workspace, but when built
# from inside a git worktree nested under the repo root, cargo walks up and
# finds the root Cargo.toml (whose `exclude` lists the canonical
# `tools/se-sidereal-position-reference` path, not the nested worktree path),
# causing a "believes it's in a workspace when it's not" error. An empty
# [workspace] table makes this package its own workspace root and resolves
# that unconditionally.
[workspace]

[dependencies]
swisseph = "0.1.1"
libswisseph-sys = "0.1.2"
```

- [ ] **Step 2: Create the licence notes**

Create `tools/se-sidereal-position-reference/LICENSE-NOTES.md`:

```markdown
# License notes — `se-sidereal-position-reference`

This crate is a **build-time verification harness only**. It is **not shipped**
and is deliberately kept **outside the Cargo workspace** (its own `Cargo.lock`,
`publish = false`, listed in the root `[workspace].exclude`). Nothing in the
shipped `pleiades-*` crates depends on it, and the workspace lockfile therefore
stays pure-Rust (no `-sys`/FFI), which the `workspace-audit` gate enforces.

Its sole purpose is to link Swiss Ephemeris (via `swisseph` / `libswisseph-sys`)
to **generate a geometric sidereal position reference corpus** (longitude,
latitude and longitude speed of the Sun, the Moon and Mercury–Pluto in four
sidereal zodiacs) used to validate the mean sidereal charts of the pure-Rust
`pleiades-core` chart layer. It runs the Moshier ephemeris (`SEFLG_MOSEPH`), so
no Swiss Ephemeris data files are bundled or distributed.

## Swiss Ephemeris licensing

Swiss Ephemeris (© Astrodienst AG) is dual-licensed: AGPL, or a separate
commercial/professional license. Because this tool is used **only internally to
produce verification fixtures** and is **never distributed as part of the
product**, no Swiss Ephemeris code, binaries, or data files enter the shipped
artifacts. The generated CSV corpus contains numeric reference values only, not
Swiss Ephemeris source or data.

Anyone building this tool locally must have libclang available
(`LIBCLANG_PATH`) and is responsible for their own compliance with the Swiss
Ephemeris license terms for their use. See the sibling `se-lilith-reference`
tool, which follows the same isolated, verification-only posture.
```

- [ ] **Step 3: Write the tool**

Create `tools/se-sidereal-position-reference/src/main.rs`:

```rust
//! Emits a Swiss Ephemeris geometric sidereal position corpus to STDOUT as
//! CSV: longitude, latitude and longitude speed of the Sun, the Moon and
//! Mercury–Pluto from
//! `swe_calc(jd_tt, body, SEFLG_MOSEPH | SEFLG_SIDEREAL | SEFLG_TRUEPOS |
//! SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_SPEED)` after `swe_set_sid_mode`
//! (Swiss Ephemeris' "ET" argument is TT).
//!
//! Place: the geometric geocentric place (no light-time, no aberration, no
//! deflection) on the mean ecliptic and equinox of date, less the ayanamsa.
//! Swiss Ephemeris drops nutation from a sidereal position. This is the place
//! a `pleiades-core` mean chart reports in a sidereal zodiac (issue #164).
//!
//! The geometric flags also keep a star-anchored ayanamsa (True Citra,
//! Galactic Center) the mean one: under apparent flags Swiss Ephemeris takes
//! the anchoring star's apparent place and folds its annual aberration, up to
//! about 20″, into the ayanamsa. `check_decomposition` asserts on every row
//! that the sidereal longitude is the mean-equinox longitude minus the mean
//! ayanamsa.
//!
//! Ayanamsas: one per `pleiades-ayanamsa` computation class, the four the
//! crossings corpus uses. Grid: every 1087 days from 1901-01-01, 67 epochs
//! across the packaged 1900–2100 window; the step shares no period with the
//! year or the month. Ephemeris: Moshier (SEFLG_MOSEPH), no data files.
//!
//! Two build/run caveats: under devenv's gcc the build needs
//! `CFLAGS=-std=gnu17` (libswisseph-sys otherwise fails with a conflicting
//! `getenv` declaration), and `devenv shell` prints a banner line to stdout,
//! so build inside devenv and run the built binary directly:
//!
//! `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release \
//!    --manifest-path tools/se-sidereal-position-reference/Cargo.toml`
//! `tools/se-sidereal-position-reference/target/release/se-sidereal-position-reference \
//!    > crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use libswisseph_sys::raw::{swe_calc, swe_get_ayanamsa_ex, swe_set_sid_mode};

const SEFLG_MOSEPH: c_int = 4;
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_SPEED: c_int = 256;
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

/// Geometric place: no light-time, aberration or deflection.
const GEOMETRIC: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

/// (Swiss Ephemeris body id, name as written to the CSV).
const BODIES: [(c_int, &str); 10] = [
    (0, "Sun"),
    (1, "Moon"),
    (2, "Mercury"),
    (3, "Venus"),
    (4, "Mars"),
    (5, "Jupiter"),
    (6, "Saturn"),
    (7, "Uranus"),
    (8, "Neptune"),
    (9, "Pluto"),
];

/// (name as written to the CSV, SE_SIDM id): OffsetDefined, TrueStar,
/// Galactic, FittedOffset.
const AYANAMSAS: [(&str, c_int); 4] = [
    ("Lahiri", 1),
    ("TrueCitra", 27),
    ("GalacticCenter", 17),
    ("DeLuce", 2),
];

const JD_FIRST_TT: f64 = 2_415_385.5; // 1901-01-01
const JD_END_TT: f64 = 2_488_069.5; //   pleiades-events WINDOW_END_JD
const STEP_DAYS: f64 = 1087.0;

fn serr_string(serr: &[c_char]) -> String {
    unsafe { CStr::from_ptr(serr.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn se_state(jd_tt: f64, body: c_int, name: &str, iflag: c_int) -> [f64; 6] {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(
            jd_tt,
            body,
            iflag,
            xx.as_mut_ptr(),
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_calc({name}, iflag={iflag}) failed at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    assert!(
        xx.iter().all(|v| v.is_finite()),
        "non-finite SE {name} result at jd_tt={jd_tt}"
    );
    xx[0] = xx[0].rem_euclid(360.0);
    xx
}

/// Mean ayanamsa (degrees) of the sidereal mode last set with
/// `swe_set_sid_mode`.
fn mean_ayanamsa(jd_tt: f64) -> f64 {
    let mut daya = 0.0_f64;
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_get_ayanamsa_ex(
            jd_tt,
            SEFLG_MOSEPH | SEFLG_NONUT | GEOMETRIC,
            &mut daya,
            serr.as_mut_ptr() as *mut c_char,
        )
    };
    if ret < 0 {
        panic!(
            "swe_get_ayanamsa_ex failed at jd_tt={jd_tt}: {}",
            serr_string(&serr)
        );
    }
    daya
}

/// Asserts that the sidereal longitude about to be written is the geometric
/// mean-equinox longitude minus the mean ayanamsa.
fn check_decomposition(jd_tt: f64, body: c_int, name: &str, ayanamsa: &str, sidereal_lon: f64) {
    let tropical = se_state(jd_tt, body, name, SEFLG_MOSEPH | GEOMETRIC | SEFLG_NONUT)[0];
    let decomposed = (tropical - mean_ayanamsa(jd_tt)).rem_euclid(360.0);
    let diff = ((sidereal_lon - decomposed + 180.0).rem_euclid(360.0) - 180.0).abs();
    assert!(
        diff < 1e-6,
        "{ayanamsa} {name} at jd_tt={jd_tt}: sidereal {sidereal_lon} vs decomposed {decomposed} differ by {diff} deg"
    );
}

fn main() {
    println!("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), swe_calc bodies 0..=9 (Sun..Pluto),");
    println!("# iflag=SEFLG_MOSEPH|SEFLG_SIDEREAL|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_SPEED after swe_set_sid_mode.");
    println!("# Place: geometric geocentric, mean ecliptic and equinox of date, less the mean ayanamsa (nutation-free).");
    println!("# Each row is checked against the geometric mean-equinox longitude minus swe_get_ayanamsa_ex. jd_tt is TT.");
    println!("# Columns: sidereal longitude and latitude (deg), longitude speed (deg/day).");
    println!("jd_tt,ayanamsa,body,lon_deg,lat_deg,lon_speed_deg_per_day");
    let place = SEFLG_MOSEPH | SEFLG_SIDEREAL | GEOMETRIC | SEFLG_SPEED;
    for (ayanamsa, sid_mode) in AYANAMSAS {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        let mut jd = JD_FIRST_TT;
        while jd < JD_END_TT {
            for (body, name) in BODIES {
                let state = se_state(jd, body, name, place);
                check_decomposition(jd, body, name, ayanamsa, state[0]);
                println!(
                    "{jd:.1},{ayanamsa},{name},{:.9},{:.9},{:.12}",
                    state[0], state[1], state[3]
                );
            }
            jd += STEP_DAYS;
        }
    }
}
```

- [ ] **Step 4: Exclude the tool from the workspace**

In the root `Cargo.toml`, in the `exclude = [...]` list of `[workspace]`, add `"tools/se-sidereal-position-reference"` as the last element, after `"tools/se-aspects-reference"`.

- [ ] **Step 5: Build the tool**

Run: `devenv shell -- env CFLAGS=-std=gnu17 cargo build --release --offline --manifest-path tools/se-sidereal-position-reference/Cargo.toml`
Expected: `Finished release profile`. The build writes `tools/se-sidereal-position-reference/Cargo.lock`. If `--offline` cannot resolve the two crates, drop `--offline`.

- [ ] **Step 6: Generate the corpus**

Run: `mkdir -p crates/pleiades-validate/data/sidereal-position-corpus`

Run: `tools/se-sidereal-position-reference/target/release/se-sidereal-position-reference > crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`
Expected: exit code 0 and no panic. A panic from `check_decomposition` means the reference does not mean what its header says; stop and investigate before going on.

Run: `grep -c -v "^#\|^jd_tt" crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`
Expected: `2680`.

Run: `head -8 crates/pleiades-validate/data/sidereal-position-corpus/sidereal-position.csv`
Expected: five `#` lines, the header `jd_tt,ayanamsa,body,lon_deg,lat_deg,lon_speed_deg_per_day`, then rows beginning `2415385.5,Lahiri,Sun,` and `2415385.5,Lahiri,Moon,`. The first line must be the `# Source:` line.

- [ ] **Step 7: Write the manifest with a provisional checksum**

Create `crates/pleiades-validate/data/sidereal-position-corpus/manifest.txt` with this single line (Task 5 replaces the `0` with the real checksum, which the gate reports):

```text
slice sidereal-position file=sidereal-position.csv role=sidereal-position rows=2680 checksum=0
```

- [ ] **Step 8: Confirm the workspace is still pure Rust**

Run: `git status --short`
Expected: the new tool directory (four files, no `target/`), the corpus directory and `Cargo.toml`; the root `Cargo.lock` is not modified.

Run: `mise run audit`
Expected: passes.

- [ ] **Step 9: Commit**

```bash
git add tools/se-sidereal-position-reference Cargo.toml crates/pleiades-validate/data/sidereal-position-corpus
git commit -m "chore(validate): add a Swiss Ephemeris geometric sidereal position corpus (#164)"
```

---

### Task 5: The `validate-sidereal-position` gate

**Files:**
- Create: `crates/pleiades-validate/src/sidereal_position_thresholds.rs`
- Create: `crates/pleiades-validate/src/sidereal_position_validation.rs`
- Create: `crates/pleiades-validate/src/sidereal_position_validation/tests.rs`
- Modify: `crates/pleiades-validate/src/lib.rs`
- Modify: `crates/pleiades-validate/src/render/cli.rs` (battery list near line 151, command match near line 484, help text near line 2449)
- Modify: `crates/pleiades-validate/src/tests/validate_gates.rs`
- Modify: `crates/pleiades-cli/src/cli.rs` (passthrough list near line 808)
- Modify: `crates/pleiades-validate/data/sidereal-position-corpus/manifest.txt`
- Modify: `docs/status.md`

**Interfaces:**
- Consumes: the corpus and manifest of Task 4; the chart behaviour of Tasks 1 and 2; `pleiades_core::{ChartEngine, ChartRequest}`, `pleiades_data::packaged_backend`, `pleiades_apparent::fnv1a64`.
- Produces: `pub fn validate_sidereal_position_corpus() -> Result<SiderealPositionReport, SiderealPositionError>`, `SiderealPositionReport::summary_line(&self) -> &str`, public field `rows_validated: usize`; CLI commands `validate-sidereal-position` and `sidereal-position-gate`.

- [ ] **Step 1: Write the thresholds file with open ceilings**

Create `crates/pleiades-validate/src/sidereal_position_thresholds.rs`. The ceilings start infinite so that Step 6 can measure; Step 7 replaces them.

```rust
//! Measured-basis ceilings for the `validate-sidereal-position` gate.

/// Per-channel ceilings for one body class.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) lon_arcsec: f64,
    pub(crate) lat_arcsec: f64,
    pub(crate) lon_speed_arcsec_per_day: f64,
}

const UNMEASURED: Ceilings = Ceilings {
    lon_arcsec: f64::INFINITY,
    lat_arcsec: f64::INFINITY,
    lon_speed_arcsec_per_day: f64::INFINITY,
};

pub(crate) const SUN_CEILINGS: Ceilings = UNMEASURED;
pub(crate) const MOON_CEILINGS: Ceilings = UNMEASURED;
/// Mercury–Pluto.
pub(crate) const PLANET_CEILINGS: Ceilings = UNMEASURED;
```

- [ ] **Step 2: Write the gate**

Create `crates/pleiades-validate/src/sidereal_position_validation.rs`:

```rust
//! Fail-closed gate: a mean sidereal `ChartEngine` chart on the packaged
//! backend vs the committed Swiss Ephemeris geometric sidereal reference
//! corpus (`SEFLG_SIDEREAL | SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL |
//! SEFLG_SPEED`; the Sun, the Moon and Mercury–Pluto; four ayanamsas;
//! 1901–2100), for longitude, latitude and longitude speed (issue #164 (b)).
//!
//! Both sides are the geometric place on the mean ecliptic and equinox of
//! date less the mean ayanamsa, so the residual is the Moshier-vs-DE440
//! ephemeris difference plus the ayanamsa gate's residual for the class. A
//! chart that subtracted the ayanamsa from a J2000 longitude, as charts did
//! before #164, is off by the precession since J2000: 83′ at the first epoch.
//! See `sidereal_position_thresholds` for the basis of the ceilings.

use crate::sidereal_position_thresholds::{
    Ceilings, MOON_CEILINGS, PLANET_CEILINGS, SUN_CEILINGS,
};
use pleiades_apparent::fnv1a64;
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_types::{
    Apparentness, Ayanamsa, CelestialBody, Instant, JulianDay, TimeScale, ZodiacMode,
};

const CORPUS_CSV: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/sidereal-position-corpus/sidereal-position.csv"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/sidereal-position-corpus/manifest.txt"
));

/// Fail-closed floor on validated rows: every corpus row is inside the
/// packaged backend's window, so none may be skipped.
const MIN_ROWS_VALIDATED: usize = 2680;

#[derive(Clone, Debug)]
struct Row {
    jd_tt: f64,
    ayanamsa: Ayanamsa,
    ayanamsa_name: &'static str,
    body: CelestialBody,
    body_name: &'static str,
    lon_deg: f64,
    lat_deg: f64,
    lon_speed: f64,
}

#[derive(Debug)]
pub enum SiderealPositionError {
    MalformedRow(String),
    MalformedManifest(String),
    ChecksumMismatch {
        got: u64,
        want: u64,
    },
    ManifestDrift {
        rows_csv: usize,
        rows_manifest: usize,
    },
    TooFewRowsValidated {
        validated: usize,
        floor: usize,
    },
    CalculationFailed {
        ayanamsa: &'static str,
        body: &'static str,
        jd_tt: f64,
        reason: String,
    },
    CeilingExceeded {
        ayanamsa: &'static str,
        body: &'static str,
        jd_tt: f64,
        kind: &'static str,
        got: f64,
        want: f64,
        residual: f64,
        ceiling: f64,
    },
}

impl std::fmt::Display for SiderealPositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedRow(s) => write!(f, "malformed corpus row: {s}"),
            Self::MalformedManifest(s) => write!(f, "malformed manifest: {s}"),
            Self::ChecksumMismatch { got, want } => {
                write!(f, "corpus checksum mismatch: got {got} want {want}")
            }
            Self::ManifestDrift {
                rows_csv,
                rows_manifest,
            } => write!(
                f,
                "manifest drift: csv has {rows_csv} rows, manifest says {rows_manifest}"
            ),
            Self::TooFewRowsValidated { validated, floor } => {
                write!(f, "only {validated} rows validated, floor is {floor}")
            }
            Self::CalculationFailed {
                ayanamsa,
                body,
                jd_tt,
                reason,
            } => write!(
                f,
                "{ayanamsa} {body} chart failed at jd_tt={jd_tt}: {reason}"
            ),
            Self::CeilingExceeded {
                ayanamsa,
                body,
                jd_tt,
                kind,
                got,
                want,
                residual,
                ceiling,
            } => write!(
                f,
                "{ayanamsa} {body} {kind} ceiling exceeded at jd_tt={jd_tt}: got {got:.12} want {want:.12} residual {residual:.6e} > ceiling {ceiling:.6e}"
            ),
        }
    }
}

impl std::error::Error for SiderealPositionError {}

/// Largest absolute residuals seen for one body class across the corpus.
#[derive(Clone, Copy, Debug, Default)]
pub struct SiderealMaxima {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub lon_speed_arcsec_per_day: f64,
}

#[derive(Debug)]
pub struct SiderealPositionReport {
    pub rows_validated: usize,
    pub sun_maxima: SiderealMaxima,
    pub moon_maxima: SiderealMaxima,
    /// Mercury–Pluto.
    pub planet_maxima: SiderealMaxima,
    summary_line: String,
}

impl SiderealPositionReport {
    pub fn summary_line(&self) -> &str {
        &self.summary_line
    }
}

fn body_from_name(name: &str) -> Option<(CelestialBody, &'static str)> {
    Some(match name {
        "Sun" => (CelestialBody::Sun, "Sun"),
        "Moon" => (CelestialBody::Moon, "Moon"),
        "Mercury" => (CelestialBody::Mercury, "Mercury"),
        "Venus" => (CelestialBody::Venus, "Venus"),
        "Mars" => (CelestialBody::Mars, "Mars"),
        "Jupiter" => (CelestialBody::Jupiter, "Jupiter"),
        "Saturn" => (CelestialBody::Saturn, "Saturn"),
        "Uranus" => (CelestialBody::Uranus, "Uranus"),
        "Neptune" => (CelestialBody::Neptune, "Neptune"),
        "Pluto" => (CelestialBody::Pluto, "Pluto"),
        _ => return None,
    })
}

fn ayanamsa_from_name(name: &str) -> Option<(Ayanamsa, &'static str)> {
    Some(match name {
        "Lahiri" => (Ayanamsa::Lahiri, "Lahiri"),
        "TrueCitra" => (Ayanamsa::TrueCitra, "TrueCitra"),
        "GalacticCenter" => (Ayanamsa::GalacticCenter, "GalacticCenter"),
        "DeLuce" => (Ayanamsa::DeLuce, "DeLuce"),
        _ => return None,
    })
}

fn parse_corpus(csv: &str) -> Result<Vec<Row>, SiderealPositionError> {
    let malformed = |what: String| SiderealPositionError::MalformedRow(what);
    let mut rows = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("jd_tt") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 6 {
            return Err(malformed(format!(
                "expected 6 fields, got {} in {line}",
                fields.len()
            )));
        }
        let num = |i: usize| -> Result<f64, SiderealPositionError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let (ayanamsa, ayanamsa_name) = ayanamsa_from_name(fields[1])
            .ok_or_else(|| malformed(format!("unknown ayanamsa {} in {line}", fields[1])))?;
        let (body, body_name) = body_from_name(fields[2])
            .ok_or_else(|| malformed(format!("unknown body {} in {line}", fields[2])))?;
        rows.push(Row {
            jd_tt: num(0)?,
            ayanamsa,
            ayanamsa_name,
            body,
            body_name,
            lon_deg: num(3)?,
            lat_deg: num(4)?,
            lon_speed: num(5)?,
        });
    }
    Ok(rows)
}

fn parse_manifest(manifest: &str) -> Result<(usize, u64), SiderealPositionError> {
    let malformed = |what: String| SiderealPositionError::MalformedManifest(what);
    let line = manifest
        .lines()
        .find(|l| l.trim_start().starts_with("slice"))
        .ok_or_else(|| malformed("no slice line".into()))?;
    let mut rows = None;
    let mut checksum = None;
    for tok in line.split_whitespace() {
        if let Some(v) = tok.strip_prefix("rows=") {
            rows = Some(
                v.parse::<usize>()
                    .map_err(|e| malformed(format!("rows: {e}")))?,
            );
        } else if let Some(v) = tok.strip_prefix("checksum=") {
            checksum = Some(
                v.parse::<u64>()
                    .map_err(|e| malformed(format!("checksum: {e}")))?,
            );
        }
    }
    Ok((
        rows.ok_or_else(|| malformed("rows= missing".into()))?,
        checksum.ok_or_else(|| malformed("checksum= missing".into()))?,
    ))
}

fn wrap_deg(got_deg: f64, want_deg: f64) -> f64 {
    (got_deg - want_deg + 180.0).rem_euclid(360.0) - 180.0
}

fn validate(csv: &str, manifest: &str) -> Result<SiderealPositionReport, SiderealPositionError> {
    let (manifest_rows, manifest_checksum) = parse_manifest(manifest)?;
    let got_checksum = fnv1a64(csv);
    if got_checksum != manifest_checksum {
        return Err(SiderealPositionError::ChecksumMismatch {
            got: got_checksum,
            want: manifest_checksum,
        });
    }
    let rows = parse_corpus(csv)?;
    if rows.len() != manifest_rows {
        return Err(SiderealPositionError::ManifestDrift {
            rows_csv: rows.len(),
            rows_manifest: manifest_rows,
        });
    }

    let engine = ChartEngine::new(packaged_backend());
    let mut sun_maxima = SiderealMaxima::default();
    let mut moon_maxima = SiderealMaxima::default();
    let mut planet_maxima = SiderealMaxima::default();
    let mut validated = 0usize;

    for row in &rows {
        let failed = |reason: String| SiderealPositionError::CalculationFailed {
            ayanamsa: row.ayanamsa_name,
            body: row.body_name,
            jd_tt: row.jd_tt,
            reason,
        };
        // The corpus epoch is TT; the chart reads the Julian day as TDB. The
        // two differ by under 2 ms, far below every ceiling here.
        let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tdb);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![row.body.clone()])
            .with_apparentness(Apparentness::Mean)
            .with_zodiac_mode(ZodiacMode::Sidereal {
                ayanamsa: row.ayanamsa.clone(),
            });
        let chart = engine.chart(&request).map_err(|e| failed(e.to_string()))?;
        let position = &chart
            .placement_for(&row.body)
            .ok_or_else(|| failed("body not placed".into()))?
            .position;
        let ecliptic = position
            .ecliptic
            .ok_or_else(|| failed("no ecliptic coordinates".into()))?;
        let got_lon_speed = position
            .motion
            .and_then(|motion| motion.longitude_deg_per_day)
            .ok_or_else(|| failed("no longitude speed".into()))?;
        let got_lon = ecliptic.longitude.degrees();
        let got_lat = ecliptic.latitude.degrees();

        let (ceilings, maxima): (Ceilings, &mut SiderealMaxima) = match row.body {
            CelestialBody::Sun => (SUN_CEILINGS, &mut sun_maxima),
            CelestialBody::Moon => (MOON_CEILINGS, &mut moon_maxima),
            _ => (PLANET_CEILINGS, &mut planet_maxima),
        };
        let checks = [
            (
                "longitude_arcsec",
                got_lon,
                row.lon_deg,
                (wrap_deg(got_lon, row.lon_deg) * 3600.0).abs(),
                ceilings.lon_arcsec,
            ),
            (
                "latitude_arcsec",
                got_lat,
                row.lat_deg,
                ((got_lat - row.lat_deg) * 3600.0).abs(),
                ceilings.lat_arcsec,
            ),
            (
                "longitude_speed_arcsec_per_day",
                got_lon_speed,
                row.lon_speed,
                ((got_lon_speed - row.lon_speed) * 3600.0).abs(),
                ceilings.lon_speed_arcsec_per_day,
            ),
        ];
        for (kind, got, want, residual, ceiling) in checks {
            // A NaN residual must fail closed too.
            if residual.is_nan() || residual > ceiling {
                return Err(SiderealPositionError::CeilingExceeded {
                    ayanamsa: row.ayanamsa_name,
                    body: row.body_name,
                    jd_tt: row.jd_tt,
                    kind,
                    got,
                    want,
                    residual,
                    ceiling,
                });
            }
        }
        maxima.lon_arcsec = maxima.lon_arcsec.max(checks[0].3);
        maxima.lat_arcsec = maxima.lat_arcsec.max(checks[1].3);
        maxima.lon_speed_arcsec_per_day = maxima.lon_speed_arcsec_per_day.max(checks[2].3);
        validated += 1;
    }
    let floor = MIN_ROWS_VALIDATED.min(manifest_rows);
    if validated < floor {
        return Err(SiderealPositionError::TooFewRowsValidated { validated, floor });
    }

    let class = |m: &SiderealMaxima| {
        format!(
            "lon {:.3}\" lat {:.3}\" speed lon {:.4}\"/d",
            m.lon_arcsec, m.lat_arcsec, m.lon_speed_arcsec_per_day
        )
    };
    let summary_line = format!(
        "Sidereal-position gate: {validated} mean sidereal chart placements validated vs Swiss Ephemeris \
         SEFLG_SIDEREAL|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_SPEED, \
         Sun max {}; Moon max {}; Mercury-Pluto max {}",
        class(&sun_maxima),
        class(&moon_maxima),
        class(&planet_maxima),
    );
    Ok(SiderealPositionReport {
        rows_validated: validated,
        sun_maxima,
        moon_maxima,
        planet_maxima,
        summary_line,
    })
}

pub fn validate_sidereal_position_corpus(
) -> Result<SiderealPositionReport, SiderealPositionError> {
    validate(CORPUS_CSV, MANIFEST)
}

#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Write the gate tests**

Create `crates/pleiades-validate/src/sidereal_position_validation/tests.rs`:

```rust
use super::*;

#[test]
fn sidereal_position_gate_passes_within_ceilings() {
    let report = validate_sidereal_position_corpus().expect("sidereal-position gate passes");
    assert_eq!(report.rows_validated, 2680);
    eprintln!("{}", report.summary_line());
}

#[test]
fn tampered_corpus_fails_the_checksum() {
    let tampered = CORPUS_CSV.replacen("2415385.5,", "2415385.5, ", 1);
    assert!(matches!(
        validate(&tampered, MANIFEST),
        Err(SiderealPositionError::ChecksumMismatch { .. })
    ));
}

#[test]
fn manifest_row_count_drift_fails_closed() {
    let checksum = fnv1a64(CORPUS_CSV);
    let manifest = format!(
        "slice sidereal-position file=sidereal-position.csv role=sidereal-position rows=2679 checksum={checksum}"
    );
    assert!(matches!(
        validate(CORPUS_CSV, &manifest),
        Err(SiderealPositionError::ManifestDrift {
            rows_csv: 2680,
            rows_manifest: 2679
        })
    ));
}

#[test]
fn manifest_without_a_slice_line_is_rejected() {
    assert!(matches!(
        validate(CORPUS_CSV, "rows=2680"),
        Err(SiderealPositionError::MalformedManifest(_))
    ));
}

#[test]
fn malformed_rows_are_rejected() {
    for bad in [
        "2451545.0,Lahiri,Mars,1.0,0.0",
        "2451545.0,Lahiri,Mars,NaN,0.0,0.5",
        "2451545.0,Lahiri,Vulcan,1.0,0.0,0.5",
        "2451545.0,Nonesuch,Mars,1.0,0.0,0.5",
    ] {
        let manifest = format!("slice x rows=1 checksum={}", fnv1a64(bad));
        assert!(
            matches!(
                validate(bad, &manifest),
                Err(SiderealPositionError::MalformedRow(_))
            ),
            "{bad}"
        );
    }
}

/// The first corpus row of `body` (1901, Lahiri) with field `index` replaced
/// by `f(value)`, and a manifest for that one row.
fn shifted_row(body: &str, index: usize, f: impl Fn(f64) -> f64) -> (String, String) {
    let line = CORPUS_CSV
        .lines()
        .find(|l| l.starts_with("2415385.5,Lahiri,") && l.split(',').nth(2) == Some(body))
        .expect("a 1901 Lahiri row");
    let mut fields: Vec<String> = line.split(',').map(str::to_string).collect();
    let value: f64 = fields[index].parse().unwrap();
    fields[index] = format!("{:.12}", f(value));
    let csv = fields.join(",");
    let manifest = format!("slice x rows=1 checksum={}", fnv1a64(&csv));
    (csv, manifest)
}

fn exceeded_kind(result: Result<SiderealPositionReport, SiderealPositionError>) -> &'static str {
    match result {
        Err(SiderealPositionError::CeilingExceeded { kind, .. }) => kind,
        other => panic!("expected a ceiling to be exceeded, got {other:?}"),
    }
}

#[test]
fn an_unshifted_row_passes_alone() {
    let (csv, manifest) = shifted_row("Saturn", 3, |lon| lon);
    assert_eq!(validate(&csv, &manifest).unwrap().rows_validated, 1);
}

// Before issue #164 the chart took the ayanamsa off a J2000 longitude, which
// at 1901 is 4978″ of precession away from the reference. Moving the
// reference by that much puts today's chart the same distance from it.
#[test]
fn a_longitude_left_on_the_j2000_equinox_fails() {
    for body in ["Sun", "Moon", "Saturn"] {
        let (csv, manifest) = shifted_row(body, 3, |lon| lon - 4978.0 / 3600.0);
        assert_eq!(
            exceeded_kind(validate(&csv, &manifest)),
            "longitude_arcsec",
            "{body}"
        );
    }
}

#[test]
fn a_latitude_left_on_the_j2000_ecliptic_fails() {
    // The two ecliptics are tilted by 46″ at 1901.
    let (csv, manifest) = shifted_row("Saturn", 4, |lat| lat + 46.0 / 3600.0);
    assert_eq!(exceeded_kind(validate(&csv, &manifest)), "latitude_arcsec");
}

// A speed that misses the precession rate is off by 0.138″/day.
#[test]
fn a_speed_without_the_precession_rate_fails() {
    for body in ["Sun", "Saturn"] {
        let (csv, manifest) = shifted_row(body, 5, |speed| speed + 0.138 / 3600.0);
        assert_eq!(
            exceeded_kind(validate(&csv, &manifest)),
            "longitude_speed_arcsec_per_day",
            "{body}"
        );
    }
}
```

- [ ] **Step 4: Register the modules**

In `crates/pleiades-validate/src/lib.rs`, directly after the two lines

```rust
mod helio_position_thresholds;
mod helio_position_validation;
```

add the two lines below. If the surrounding `mod` lines are in alphabetical order, put them at their alphabetical place instead (before `mod stations_thresholds;`).

```rust
mod sidereal_position_thresholds;
mod sidereal_position_validation;
```

and directly after the `pub use helio_position_validation::{ ... };` item add:

```rust
pub use sidereal_position_validation::{
    validate_sidereal_position_corpus, SiderealMaxima, SiderealPositionError,
    SiderealPositionReport,
};
```

- [ ] **Step 5: Run the gate test to get the checksum**

Run: `cargo test -p pleiades-validate --lib sidereal_position_validation::tests::sidereal_position_gate_passes_within_ceilings -- --nocapture`
Expected: FAIL with `ChecksumMismatch { got: <number>, want: 0 }`.

Put that number in `crates/pleiades-validate/data/sidereal-position-corpus/manifest.txt`, replacing the `0` after `checksum=`.

- [ ] **Step 6: Measure**

Run: `cargo test -p pleiades-validate --lib sidereal_position_validation::tests::sidereal_position_gate_passes_within_ceilings -- --nocapture`
Expected: PASS, printing `Sidereal-position gate: 2680 mean sidereal chart placements validated ... Sun max lon …" lat …" speed lon …"/d; Moon max …; Mercury-Pluto max …`.

Record the nine maxima. Three checks before any ceiling is set (the spec's stop condition):

- Expected longitudes, from the crossings gate's sidereal groups: Sun about 0.3″, Moon up to about 3″, planets under 1″. A longitude maximum of tens of arcseconds or more in any class means the chart and the reference are not the same place: stop and find the cause.
- Sun and planet speed maxima must be below 0.055″/day. A row shifted by 0.138″/day then still exceeds a ceiling of 1.4 × max (0.138 − max > 1.4 × max), which `a_speed_without_the_precession_rate_fails` needs. If either is larger, stop and investigate the speed before setting anything.
- If one class's maximum comes from a single ayanamsa and is several times the others', check that ayanamsa against `validate-ayanamsa`'s ceiling for its class before accepting it.

- [ ] **Step 7: Set the ceilings**

In `crates/pleiades-validate/src/sidereal_position_thresholds.rs`, delete the `UNMEASURED` constant and write the three constants out with the measured values. Longitude and latitude: `ceil(1.4 × max)` in whole arcseconds. Speed: `1.4 × max` rounded up to two significant figures. Record each measured maximum beside its ceiling, and replace the module doc with the basis. The file's final shape, with `<…>` standing for the numbers Step 6 printed and the ceilings computed from them:

```rust
//! Measured-basis ceilings for the `validate-sidereal-position` gate.
//!
//! Both sides of the comparison are the geometric geocentric place on the
//! mean ecliptic and equinox of date less the mean ayanamsa, so the residual
//! is the Moshier-vs-DE440 ephemeris difference plus the ayanamsa's own
//! residual. Measured 2026-10-06 over all 2680 rows (67 epochs, four
//! ayanamsas, ten bodies).
//!
//! Longitude and latitude ceilings are `ceil(1.4 × max)` in whole
//! arcseconds, the crossings corpus rule. Speed ceilings are `1.4 × max`
//! rounded up to two significant figures: a whole-arcsecond speed ceiling
//! would pass a speed that misses the precession rate, 0.138″/day, which is
//! half of what issue #164 corrected.

/// Per-channel ceilings for one body class.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) lon_arcsec: f64,
    pub(crate) lat_arcsec: f64,
    pub(crate) lon_speed_arcsec_per_day: f64,
}

pub(crate) const SUN_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lat_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lon_speed_arcsec_per_day: <1.4 × max, 2 s.f.>, // measured max <…>"/day
};

pub(crate) const MOON_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lat_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lon_speed_arcsec_per_day: <1.4 × max, 2 s.f.>, // measured max <…>"/day
};

/// Mercury–Pluto.
pub(crate) const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lat_arcsec: <ceil(1.4 × max)>,                // measured max <…>"
    lon_speed_arcsec_per_day: <1.4 × max, 2 s.f.>, // measured max <…>"/day
};
```

Every `<…>` is replaced by a number in this step; none may remain in the committed file.

- [ ] **Step 8: Run the gate tests**

Run: `cargo test -p pleiades-validate --lib sidereal_position`
Expected: 9 passed. Note the reported time of the test binary run; it bounds what the gate adds to the release battery. If it is more than a few seconds, say so in the pull request.

- [ ] **Step 9: Write the failing CLI tests**

In `crates/pleiades-validate/src/tests/validate_gates.rs`, add at the end of the file:

```rust
#[test]
fn validate_sidereal_position_and_alias_report_the_summary() {
    let report = crate::validate_sidereal_position_corpus().expect("gate passes");
    for command in ["validate-sidereal-position", "sidereal-position-gate"] {
        let out = render_cli(&[command]).expect("gate passes via the CLI");
        assert_eq!(out, report.summary_line(), "{command}");
    }
    assert!(render_cli(&["validate-sidereal-position", "extra"]).is_err());
}

#[test]
fn help_text_mentions_validate_sidereal_position() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(help.contains("validate-sidereal-position"));
    assert!(help.contains("sidereal-position-gate"));
}
```

Run: `cargo test -p pleiades-validate --lib sidereal_position`
Expected: the two new tests FAIL (unknown command; help text lacks the names).

- [ ] **Step 10: Register the command, the battery entry and the help text**

In `crates/pleiades-validate/src/render/cli.rs`:

1. In the battery list, directly after the entry

```rust
    ("helio-position gate failed", || {
        crate::validate_helio_position_corpus()
            .map(drop)
            .map_err(|e| e.to_string())
    }),
```

add

```rust
    ("sidereal-position gate failed", || {
        crate::validate_sidereal_position_corpus()
            .map(drop)
            .map_err(|e| e.to_string())
    }),
```

2. In the command match, directly after the `Some("validate-helio-position") | Some("helio-position-gate") => { ... }` arm, add

```rust
        Some("validate-sidereal-position") | Some("sidereal-position-gate") => {
            ensure_no_extra_args(&args[1..], "validate-sidereal-position")?;
            crate::validate_sidereal_position_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

3. In the help text string, find `  helio-position-gate       Alias for validate-helio-position\n` and insert directly after it:

```text
  validate-sidereal-position  Run the fail-closed sidereal mean chart gate (Swiss Ephemeris SEFLG_SIDEREAL geometric place: longitude, latitude and longitude speed of a mean sidereal chart, arcsecond ceilings) over the committed sidereal-position corpus\n  sidereal-position-gate    Alias for validate-sidereal-position\n
```

In `crates/pleiades-cli/src/cli.rs`, directly after the line

```rust
        Some("validate-helio-position") | Some("helio-position-gate") => validate_render_cli(args),
```

add

```rust
        Some("validate-sidereal-position") | Some("sidereal-position-gate") => {
            validate_render_cli(args)
        }
```

- [ ] **Step 11: Run the tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib sidereal_position`
Expected: 11 passed.

Run: `cargo run -q -p pleiades-validate -- validate-sidereal-position`
Expected: the `Sidereal-position gate: 2680 mean sidereal chart placements validated ...` line.

Run: `cargo run -q -p pleiades-cli -- validate-sidereal-position`
Expected: the same line.

- [ ] **Step 12: Add the gate to the status table**

In `docs/status.md`, directly after the row that begins `| Apparent equatorial (RA/Dec) |`, add:

```markdown
| Sidereal mean chart (mean equinox of date, less the ayanamsa) | [`pleiades-core`](https://docs.rs/pleiades-core) | `validate-sidereal-position` | arcsecond-class against Swiss Ephemeris |
```

- [ ] **Step 13: Lint and commit**

Run: `cargo fmt --all`
Run: `cargo clippy -p pleiades-validate -p pleiades-cli --all-targets --all-features -- -D warnings`
Expected: no warnings.

```bash
git add crates/pleiades-validate crates/pleiades-cli/src/cli.rs docs/status.md
git commit -m "feat(validate): gate mean sidereal charts against Swiss Ephemeris (#164)"
```

---

### Task 6: Whole-branch verification and pull request

**Files:**
- No source changes expected. Any pinned value the blocking tier turns up is updated in the file that pins it.

**Interfaces:**
- Consumes: everything above.
- Produces: an open pull request.

- [ ] **Step 1: Run the blocking tier**

Run: `mise run ci`
Expected: every task passes (`fmt`, `lint`, `docs`, `audit`, `deny`, `secrets`, `claims-audit`, `test`, `doctest`, `release-smoke`). Run it in the foreground and make no edits or commits while it runs.

If a test outside this plan fails because it pinned a sidereal mean chart value (the spec found none, and this step is where that is confirmed), read the test: if it encodes "J2000 longitude less the ayanamsa", update it to the mean-of-date rule and say so in the pull request. Do not change any gate's ceiling or tolerance to make it pass.

If `claims-audit` or `audit` reports that the new gate or tool is missing from a list it checks, add the entry it names.

- [ ] **Step 2: Run the gates that read sidereal places**

Run: `cargo run -q -p pleiades-validate -- validate-crossings`
Expected: `329 SE crossing fixtures`, unchanged maxima (this gate does not go through the chart layer).

Run: `cargo run -q -p pleiades-validate -- validate-sidereal-position`
Expected: passes with 2680 rows.

- [ ] **Step 3: Push and open the pull request**

```bash
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin fix/sidereal-mean-chart-frame
```

Open a pull request against `main` titled `fix(core): put a sidereal mean chart on the mean equinox of date (#164)`. Its body states: the defect with the measured table from the spec; the rule adopted and the two rejected; what moved (longitude, latitude, speed) and what did not (tropical mean, apparent, equatorial); the new gate with its measured maxima and ceilings; the commands run in Steps 1 and 2 with their results; and `Refs #164` (item (c) stays open).

- [ ] **Step 4: After the checks pass**

Wait for the pull request's checks to finish and confirm they are green with `gh pr checks <number>`; then squash-merge with `gh pr merge <number> --squash --delete-branch` (never `--auto`). Comment on issue #164 that item (b) is done, with the measured maxima, and that item (c) remains open.
