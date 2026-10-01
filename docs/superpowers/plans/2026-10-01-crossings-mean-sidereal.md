# Longitude Crossings in a Mean Place or a Sidereal Zodiac (issue #88) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let `pleiades-events` find longitude crossings, and read longitudes and positions, in a geocentric mean place of date and in any sidereal zodiac, without changing a bit of what existing calls return.

**Architecture:** A new unit variant `CrossingFrame::GeocentricMeanOfDate` plus a small value type `CrossingReference { frame, zodiac }` that every crossing/position method accepts through `impl Into<CrossingReference>`. One new module, `src/reference.rs`, is the single place a frame and zodiac become a position; the root-finder closure calls it, so nutation and the ayanamsa are evaluated at every trial instant. The existing `validate-crossings` gate and its Swiss Ephemeris corpus are extended, not duplicated.

**Tech Stack:** Rust 1.99.0 (stable), cargo-nextest via `mise run test`, Swiss Ephemeris via `libswisseph-sys` for the reference tool only (built under `devenv shell`).

**Spec:** `docs/superpowers/specs/2026-10-01-crossings-mean-sidereal-design.md`

## Global Constraints

- Every existing call must compile unchanged and return the same bits. The 86 committed golden crossings must not move.
- `CrossingFrame` keeps `Copy`, `Eq`, `Hash`. No variant is added to `EventError` (it is not `#[non_exhaustive]`); new failures use `EventError::UnsupportedFrame { detail }`.
- No silent tropical fallback: heliocentric + sidereal, and an ayanamsa with no offset data, are errors.
- Mean place = backend J2000 geometric place precessed to the mean equinox of date. No light-time, aberration or nutation.
- Sidereal longitude = longitude on the mean equinox of date − mean ayanamsa. For the apparent frame that means subtracting Δψ first.
- Corpus ceilings are measured: `ceil(1.4 × group max)`. Never widen a ceiling to absorb an unexplained systematic offset (see Task 3 stop condition).
- The Swiss Ephemeris corpus is produced only by `tools/se-crossings-reference`. It must not be hand-written or derived from pleiades.
- Do not bump crate versions by hand; release-plz does it.
- Before each commit run, at minimum: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the task's own tests. The final task runs `mise run ci`.
- Run test commands in the foreground. Make no source edits and no commits while a test run is in progress (release-bundle tests compare git provenance).
- If working in a worktree session: no heredocs, shell loops or process substitution in Bash; use the Edit/Write tools and plain commands.

## Review Focus

Inputs the spec implies but does not list a test for. Each has a test in Task 2.

1. **A sidereal target next to 0°/360°.** The ayanamsa shift carries the longitude across the wrap; a crossing of 0° and of 359.9° sidereal must be found with a small residual.
2. **Backward search in the new references.** `previous_longitude_crossing` in a sidereal and a mean reference must equal the last element of the range search, as it does today for tropical.
3. **Reusing one reference across calls.** `CrossingReference` is not `Copy`; a caller looping over targets must be able to pass `&reference` without cloning by hand.
4. **A custom ayanamsa that does carry an epoch and offset.** It must work, not be rejected along with the data-less one.
5. **`position_at` in a sidereal reference at the window edge.** At `WINDOW_END_JD` the difference is one-sided; the speed must still be `Some`.

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `crates/pleiades-events/src/ephemeris.rs` | modify | add `geocentric_mean_of_date_ecliptic` |
| `crates/pleiades-events/src/reference.rs` | create | `CrossingReference`; `check_supported`, `ecliptic_in`, `sampled_place` |
| `crates/pleiades-events/src/reference/tests.rs` | create | white-box bit-identity tests |
| `crates/pleiades-events/src/crossings.rs` | modify | new variant; methods take `impl Into<CrossingReference>`; `Crossing::zodiac` |
| `crates/pleiades-events/src/position.rs` | modify | `position_at` takes a reference; `EclipticPosition::zodiac`; `sample` delegates |
| `crates/pleiades-events/src/position/tests.rs` | modify | call-site update |
| `crates/pleiades-events/src/lib.rs` | modify | `mod reference; pub use reference::CrossingReference;` |
| `crates/pleiades-events/Cargo.toml` | modify | `pleiades-ayanamsa` dependency |
| `crates/pleiades-events/tests/reference.rs` | create | black-box tests of the new frame and zodiacs |
| `tools/se-crossings-reference/src/main.rs` | modify | `zodiac` column; mean and sidereal rows |
| `crates/pleiades-validate/data/crossings-corpus/{crossings.csv,manifest.txt}` | regenerate | corpus |
| `crates/pleiades-validate/src/crossings_validation.rs` | modify | 8-column schema, reference parsing, new ceilings |
| `crates/pleiades-validate/src/render/cli.rs` | modify | golden helpers for the 7/8-column forms; help text |
| `README.md`, `crates/pleiades-events/README.md`, `crates/pleiades-core/src/compatibility/mod.rs`, `docs/follow-ups.md`, the spec | modify | documentation |

---

### Task 1: `CrossingFrame::GeocentricMeanOfDate`

**Files:**
- Modify: `crates/pleiades-events/src/ephemeris.rs` (after `geocentric_apparent_longitude_deg`, about line 153)
- Modify: `crates/pleiades-events/src/crossings.rs:13-18` (enum), `:68-83` (`longitude_deg`)
- Modify: `crates/pleiades-events/src/position.rs:78-108` (`sample`)
- Create: `crates/pleiades-events/tests/reference.rs`

**Interfaces:**
- Consumes: `read_mean_ecliptic`, `read_mean_ecliptic_with_motion`, `pleiades_apparent::precess_ecliptic_j2000_to_date` (all existing).
- Produces:
  - `CrossingFrame::GeocentricMeanOfDate` (public unit variant).
  - `pub(crate) fn geocentric_mean_of_date_ecliptic<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<(f64, f64, f64), EventError>` returning `(longitude_deg in [0,360), latitude_deg, distance_au)`.

- [ ] **Step 1: Write the failing tests**

Create `crates/pleiades-events/tests/reference.rs`:

```rust
//! Crossings, longitudes and positions in the mean place of date and in a
//! sidereal zodiac (issue #88).

use pleiades_apparent::nutation::nutation;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine, WINDOW_START_JD};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

#[test]
fn sun_mean_of_date_differs_from_apparent_by_aberration_and_nutation() {
    // The Sun's apparent place is its geometric place moved by annual
    // aberration (-20.4898"/R, R in AU) and referred to the true equinox
    // (+Δψ). This checks the mean frame against two independently known
    // quantities, not against the engine's own formula.
    let engine = EventEngine::new(packaged_backend());
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        let apparent = engine.position_at(CelestialBody::Sun, APPARENT, tdb(jd)).unwrap();
        let mean = engine.longitude_at(CelestialBody::Sun, MEAN, tdb(jd)).unwrap();
        let got_arcsec =
            wrap(apparent.ecliptic.longitude.degrees() - mean.degrees()) * 3600.0;
        let delta_psi = nutation(jd).unwrap().delta_psi_arcsec;
        let aberration = -20.4898 / apparent.ecliptic.distance_au.unwrap();
        assert!(
            (got_arcsec - (delta_psi + aberration)).abs() < 0.2,
            "jd {jd}: apparent-mean {got_arcsec}\" vs Δψ {delta_psi}\" + aberration {aberration}\""
        );
    }
}

#[test]
fn mean_frame_reads_succeed_at_the_range_start() {
    // No light-time re-query, so the range-start failure of FU-17(c) does
    // not apply to the mean frame.
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::Mars, CelestialBody::Moon] {
        engine
            .longitude_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
        engine
            .position_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
    }
}

#[test]
fn mean_position_longitude_is_bit_identical_to_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mars, CelestialBody::Pluto] {
        for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
            let pos = engine.position_at(body.clone(), MEAN, tdb(jd)).unwrap();
            let lon = engine.longitude_at(body.clone(), MEAN, tdb(jd)).unwrap();
            assert_eq!(
                pos.ecliptic.longitude.degrees().to_bits(),
                lon.degrees().to_bits(),
                "{body:?} {jd}"
            );
            assert_eq!(pos.frame, MEAN);
        }
    }
}

#[test]
fn mean_speed_matches_a_central_difference_of_the_longitude() {
    let engine = EventEngine::new(packaged_backend());
    let h = 0.5;
    for body in [CelestialBody::Sun, CelestialBody::Mars, CelestialBody::Jupiter] {
        let jd = 2_455_000.5;
        let before = engine.longitude_at(body.clone(), MEAN, tdb(jd - h)).unwrap();
        let after = engine.longitude_at(body.clone(), MEAN, tdb(jd + h)).unwrap();
        let rate = wrap(after.degrees() - before.degrees()) / (2.0 * h);
        let speed = engine
            .position_at(body.clone(), MEAN, tdb(jd))
            .unwrap()
            .motion
            .longitude_deg_per_day
            .expect("speed");
        assert!((speed - rate).abs() < 1e-4, "{body:?}: speed {speed} vs {rate}");
    }
}

#[test]
fn mean_crossing_lands_on_the_target_from_the_settled_side() {
    // The returned instant trails the crossing by < 0.5 s and never precedes
    // it. Both bodies move direct, so the residual is in [0, speed × 0.5 s).
    let engine = EventEngine::new(packaged_backend());
    for (body, bound_deg) in [(CelestialBody::Sun, 1.0e-5), (CelestialBody::Moon, 1.0e-4)] {
        for target_deg in [0.0, 137.5, 359.9] {
            let crossing = engine
                .next_longitude_crossing(
                    body.clone(),
                    Longitude::from_degrees(target_deg),
                    MEAN,
                    tdb(2_451_545.0),
                )
                .unwrap()
                .expect("crossing");
            assert_eq!(crossing.frame, MEAN);
            let lon = engine.longitude_at(body.clone(), MEAN, crossing.instant).unwrap();
            let residual = wrap(lon.degrees() - target_deg);
            assert!(
                (0.0..bound_deg).contains(&residual),
                "{body:?} target {target_deg}: residual {residual}"
            );
        }
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-events --test reference`
Expected: FAIL to compile — `no variant named GeocentricMeanOfDate`.

- [ ] **Step 3: Add the variant**

In `crates/pleiades-events/src/crossings.rs`, extend the enum:

```rust
pub enum CrossingFrame {
    /// Geocentric apparent tropical ecliptic of date (SE `solcross`/`mooncross`).
    GeocentricApparentOfDate,
    /// Heliocentric ecliptic (SE `helio_cross`); planets only.
    Heliocentric,
    /// Geocentric geometric place in the mean ecliptic and equinox of date:
    /// the backend's J2000 place precessed to date, with no light-time, no
    /// aberration and no nutation (SE `SEFLG_TRUEPOS | SEFLG_NOABERR |
    /// SEFLG_NOGDEFL | SEFLG_NONUT`). This is not the J2000 longitude a
    /// `pleiades-core` mean chart reports.
    GeocentricMeanOfDate,
}
```

- [ ] **Step 4: Add the ephemeris helper**

In `crates/pleiades-events/src/ephemeris.rs`, after `geocentric_apparent_longitude_deg`:

```rust
/// Geocentric geometric ecliptic `(longitude_deg, latitude_deg, distance_au)`
/// in the mean ecliptic and equinox of date: the backend's J2000 place
/// precessed to date. No light-time, no aberration, no nutation.
pub(crate) fn geocentric_mean_of_date_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let (lon, lat, dist) = read_mean_ecliptic(backend, body, body_label, julian_day)?;
    let precessed = precess_ecliptic_j2000_to_date(lon, lat, julian_day)
        .map_err(|e| EventError::Backend(format!("{body_label} precession failed: {e}")))?;
    Ok((
        precessed.longitude_deg.rem_euclid(360.0),
        precessed.latitude_deg,
        dist,
    ))
}
```

Update the module doc comment's first lines to name the third longitude: `geocentric apparent-of-date, geocentric mean-of-date, and heliocentric`.

- [ ] **Step 5: Wire the variant into the two frame matches**

In `crossings.rs`, add the import `geocentric_mean_of_date_ecliptic` to the `use crate::ephemeris::{...}` line and add an arm to `longitude_deg`:

```rust
            CrossingFrame::GeocentricMeanOfDate => Ok(geocentric_mean_of_date_ecliptic(
                &self.backend,
                body.clone(),
                body_label(body),
                jd,
            )?
            .0),
```

In `position.rs`, add the same import and an arm to `sample`:

```rust
        CrossingFrame::GeocentricMeanOfDate => {
            let (mean, base_motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let of_date =
                geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?;
            Ok(Sample {
                base: coordinates(mean),
                corrected: coordinates(of_date),
                base_motion,
            })
        }
```

Add one row to the table in `position.rs`'s module doc:

```
//! | geocentric mean of date | backend mean J2000 place and its motion | precessed to the mean equinox of date |
```

and one bullet to the `position_at` rustdoc, after the `Heliocentric` bullet:

```rust
    /// - [`CrossingFrame::GeocentricMeanOfDate`]: the geometric place from the
    ///   Earth's centre (no light-time, no aberration, no nutation) in the
    ///   mean ecliptic and equinox of date.
```

In `longitude_at`'s rustdoc, change the sentence listing the frames to also name `[`CrossingFrame::GeocentricMeanOfDate`]` as "geocentric geometric, mean equinox of date".

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p pleiades-events --test reference`
Expected: 5 passed.

If `sun_mean_of_date_differs_from_apparent_by_aberration_and_nutation` fails, print the difference and STOP: a disagreement above 0.2″ means the mean place is not what the spec defines. Do not loosen the tolerance.

Run: `cargo test -p pleiades-events`
Expected: all pass (existing pinned-bit tests included).

Run: `cargo test -p pleiades-cli crossings`
Expected: PASS (Tier-1 golden unchanged).

- [ ] **Step 7: Lint and commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-events
git commit -m "feat(events): geocentric mean-of-date crossing frame (#88)"
```

---

### Task 2: `CrossingReference` and sidereal zodiacs

**Files:**
- Modify: `crates/pleiades-events/Cargo.toml`
- Create: `crates/pleiades-events/src/reference.rs`, `crates/pleiades-events/src/reference/tests.rs`
- Modify: `crates/pleiades-events/src/crossings.rs` (struct `Crossing`, `longitude_deg` removed, four methods)
- Modify: `crates/pleiades-events/src/position.rs` (`EclipticPosition`, `sample`, `motion`, `position_at`)
- Modify: `crates/pleiades-events/src/position/tests.rs`
- Modify: `crates/pleiades-events/src/lib.rs:110-131`
- Modify: `crates/pleiades-events/tests/reference.rs`

**Interfaces:**
- Consumes: `geocentric_mean_of_date_ecliptic` (Task 1); `geocentric_apparent_ecliptic`, `heliocentric_j2000`, `heliocentric_of_date`, `j2000_spherical`, `read_mean_ecliptic_with_motion` (existing); `pleiades_apparent::nutation::nutation(jd) -> Result<Nutation, _>` with field `delta_psi_arcsec`; `pleiades_ayanamsa::sidereal_offset(&Ayanamsa, Instant) -> Option<Angle>`.
- Produces:
  - `pub struct CrossingReference { pub frame: CrossingFrame, pub zodiac: ZodiacMode }` (`#[non_exhaustive]`), with `CrossingReference::tropical(frame)`, `CrossingReference::sidereal(frame, ayanamsa)`, `From<CrossingFrame>`, `From<&CrossingReference>`.
  - `pub(crate) fn check_supported(body: &CelestialBody, reference: &CrossingReference, julian_day: f64, what: &str) -> Result<(), EventError>`
  - `pub(crate) fn ecliptic_in<B: EphemerisBackend>(backend: &B, body: &CelestialBody, reference: &CrossingReference, julian_day: f64) -> Result<(f64, f64, f64), EventError>`
  - `pub(crate) struct SampledPlace { pub(crate) base: (f64, f64, f64), pub(crate) base_motion: Option<Motion>, pub(crate) corrected: (f64, f64, f64) }` and `pub(crate) fn sampled_place<B>(backend, body, reference, julian_day) -> Result<SampledPlace, EventError>`
  - `Crossing::zodiac: ZodiacMode`, `EclipticPosition::zodiac: ZodiacMode`.
  - The five public methods take `reference: impl Into<CrossingReference>` in the position `frame: CrossingFrame` occupies today.

- [ ] **Step 1: Add the dependency**

In `crates/pleiades-events/Cargo.toml`, under `[dependencies]`, after `pleiades-apparent`:

```toml
pleiades-ayanamsa = { workspace = true }
```

Run: `cargo tree -p pleiades-events -i pleiades-events --depth 0` (sanity) and `cargo check -p pleiades-events`.
Expected: builds; no dependency cycle error.

- [ ] **Step 2: Write the failing black-box tests**

In `crates/pleiades-events/tests/reference.rs`, replace the `use` block with:

```rust
use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, CrossingReference, EventEngine, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_types::{
    Ayanamsa, CelestialBody, CustomAyanamsa, Instant, JulianDay, Longitude, TimeScale, ZodiacMode,
};

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;

fn lahiri(frame: CrossingFrame) -> CrossingReference {
    CrossingReference::sidereal(frame, Ayanamsa::Lahiri)
}

fn ayanamsa_deg(ayanamsa: &Ayanamsa, jd: f64) -> f64 {
    sidereal_offset(ayanamsa, Instant::new(JulianDay::from_days(jd), TimeScale::Tt))
        .expect("offset")
        .degrees()
}

/// Every new reference, for tests that must hold in all of them.
fn new_references() -> Vec<CrossingReference> {
    vec![
        CrossingReference::tropical(MEAN),
        lahiri(APPARENT),
        lahiri(MEAN),
        CrossingReference::sidereal(APPARENT, Ayanamsa::TrueCitra),
        CrossingReference::sidereal(MEAN, Ayanamsa::GalacticCenter),
    ]
}
```

(Keep the existing `tdb` and `wrap` helpers and the five Task 1 tests.) Append:

```rust
#[test]
fn a_frame_converts_to_the_tropical_reference() {
    let reference: CrossingReference = APPARENT.into();
    assert_eq!(reference.frame, APPARENT);
    assert_eq!(reference.zodiac, ZodiacMode::Tropical);
    assert_eq!(reference, CrossingReference::tropical(APPARENT));
    assert_eq!(
        lahiri(MEAN).zodiac,
        ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri }
    );
}

#[test]
fn sidereal_longitude_is_the_mean_equinox_longitude_minus_the_ayanamsa() {
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Saturn] {
        for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
            let ayanamsa = ayanamsa_deg(&Ayanamsa::Lahiri, jd);
            let delta_psi_deg = nutation(jd).unwrap().delta_psi_arcsec / 3600.0;

            let apparent = engine.longitude_at(body.clone(), APPARENT, tdb(jd)).unwrap();
            let sidereal = engine.longitude_at(body.clone(), lahiri(APPARENT), tdb(jd)).unwrap();
            let expected = apparent.degrees() - delta_psi_deg - ayanamsa;
            assert!(
                wrap(sidereal.degrees() - expected).abs() < 1e-10,
                "{body:?} {jd} apparent sidereal"
            );

            let mean = engine.longitude_at(body.clone(), MEAN, tdb(jd)).unwrap();
            let sidereal = engine.longitude_at(body.clone(), lahiri(MEAN), tdb(jd)).unwrap();
            assert!(
                wrap(sidereal.degrees() - (mean.degrees() - ayanamsa)).abs() < 1e-10,
                "{body:?} {jd} mean sidereal"
            );
            assert!((0.0..360.0).contains(&sidereal.degrees()));
        }
    }
}

#[test]
fn heliocentric_sidereal_is_unsupported() {
    let engine = EventEngine::new(packaged_backend());
    let t = tdb(2_451_545.0);
    let target = Longitude::from_degrees(0.0);
    let reference = lahiri(HELIO);
    for err in [
        engine.longitude_at(CelestialBody::Mars, &reference, t).unwrap_err(),
        engine.position_at(CelestialBody::Mars, &reference, t).unwrap_err(),
        engine
            .next_longitude_crossing(CelestialBody::Mars, target, &reference, t)
            .unwrap_err(),
        engine
            .previous_longitude_crossing(CelestialBody::Mars, target, &reference, t)
            .unwrap_err(),
        engine
            .longitude_crossings_in_range(CelestialBody::Mars, target, &reference, t, tdb(2_451_645.0))
            .unwrap_err(),
    ] {
        assert!(matches!(err, EventError::UnsupportedFrame { .. }), "{err:?}");
    }
}

#[test]
fn an_ayanamsa_without_offset_data_is_unsupported() {
    // No epoch, no offset: `sidereal_offset` returns None. There must be no
    // silent tropical fallback.
    let engine = EventEngine::new(packaged_backend());
    let t = tdb(2_451_545.0);
    let reference =
        CrossingReference::sidereal(APPARENT, Ayanamsa::Custom(CustomAyanamsa::new("no data")));
    for err in [
        engine.longitude_at(CelestialBody::Sun, &reference, t).unwrap_err(),
        engine.position_at(CelestialBody::Sun, &reference, t).unwrap_err(),
        engine
            .next_longitude_crossing(CelestialBody::Sun, Longitude::from_degrees(0.0), &reference, t)
            .unwrap_err(),
    ] {
        assert!(matches!(err, EventError::UnsupportedFrame { .. }), "{err:?}");
    }
}

#[test]
fn a_custom_ayanamsa_with_epoch_and_offset_works() {
    let engine = EventEngine::new(packaged_backend());
    let mut custom = CustomAyanamsa::new("fixed 24 at J2000");
    custom.epoch = Some(JulianDay::from_days(2_451_545.0));
    custom.offset_degrees = Some(pleiades_types::Angle::from_degrees(24.0));
    let reference = CrossingReference::sidereal(MEAN, Ayanamsa::Custom(custom));
    let jd = 2_451_545.0;
    let mean = engine.longitude_at(CelestialBody::Sun, MEAN, tdb(jd)).unwrap();
    let sidereal = engine.longitude_at(CelestialBody::Sun, &reference, tdb(jd)).unwrap();
    assert!(wrap(sidereal.degrees() - (mean.degrees() - 24.0)).abs() < 1e-9);
}

#[test]
fn crossings_land_on_the_target_from_the_settled_side_in_every_new_reference() {
    let engine = EventEngine::new(packaged_backend());
    for reference in new_references() {
        for (body, bound_deg) in [(CelestialBody::Sun, 1.0e-5), (CelestialBody::Moon, 1.0e-4)] {
            // 0.0 and 359.9 sit on the wrap seam of the shifted longitude.
            for target_deg in [0.0, 137.5, 359.9] {
                let crossing = engine
                    .next_longitude_crossing(
                        body.clone(),
                        Longitude::from_degrees(target_deg),
                        &reference,
                        tdb(2_451_545.0),
                    )
                    .unwrap()
                    .expect("crossing");
                assert_eq!(crossing.frame, reference.frame);
                assert_eq!(crossing.zodiac, reference.zodiac);
                let lon = engine.longitude_at(body.clone(), &reference, crossing.instant).unwrap();
                let residual = wrap(lon.degrees() - target_deg);
                assert!(
                    (0.0..bound_deg).contains(&residual),
                    "{reference:?} {body:?} target {target_deg}: residual {residual}"
                );
            }
        }
    }
}

#[test]
fn mars_2003_loop_gives_three_sidereal_crossings() {
    // Tropical 337.0° is crossed direct, retrograde, direct in the 2003
    // opposition loop (the corpus's triple crossing). The same point in the
    // Lahiri zodiac is 337.0° minus the ayanamsa.
    let engine = EventEngine::new(packaged_backend());
    let target = Longitude::from_degrees(337.0 - ayanamsa_deg(&Ayanamsa::Lahiri, 2_452_850.0));
    let crossings = engine
        .longitude_crossings_in_range(
            CelestialBody::Mars,
            target,
            lahiri(APPARENT),
            tdb(2_452_791.5),
            tdb(2_453_000.5),
        )
        .unwrap();
    assert_eq!(crossings.len(), 3, "{crossings:?}");
    assert!(crossings
        .windows(2)
        .all(|pair| pair[1].instant.julian_day.days() > pair[0].instant.julian_day.days() + 1.0));
}

#[test]
fn next_after_a_returned_sidereal_crossing_is_the_following_one() {
    let engine = EventEngine::new(packaged_backend());
    let reference = lahiri(APPARENT);
    for target_deg in [7.3, 127.3, 247.3] {
        let target = Longitude::from_degrees(target_deg);
        let found = engine
            .next_longitude_crossing(CelestialBody::Moon, target, &reference, tdb(2_451_545.0))
            .unwrap()
            .expect("the Moon crosses every longitude monthly");
        let following = engine
            .next_longitude_crossing(CelestialBody::Moon, target, &reference, found.instant)
            .unwrap()
            .expect("the Moon crosses every longitude monthly");
        let gap = following.instant.julian_day.days() - found.instant.julian_day.days();
        assert!((27.0..28.0).contains(&gap), "target {target_deg}: gap {gap} days");
    }
}

#[test]
fn previous_equals_the_last_crossing_in_range_in_the_new_references() {
    let engine = EventEngine::new(packaged_backend());
    let target = Longitude::from_degrees(100.0);
    let before = tdb(WINDOW_START_JD + 800.0);
    for reference in [CrossingReference::tropical(MEAN), lahiri(APPARENT), lahiri(MEAN)] {
        let all = engine
            .longitude_crossings_in_range(
                CelestialBody::Sun,
                target,
                &reference,
                tdb(WINDOW_START_JD),
                before,
            )
            .unwrap();
        assert!(all.len() >= 2, "{reference:?}: {}", all.len());
        let last = all.last().unwrap();
        let previous = engine
            .previous_longitude_crossing(CelestialBody::Sun, target, &reference, before)
            .unwrap()
            .expect("previous crossing");
        assert!(
            (previous.instant.julian_day.days() - last.instant.julian_day.days()).abs() < 1e-6,
            "{reference:?}"
        );
    }
}

#[test]
fn position_longitude_is_bit_identical_to_longitude_at_in_every_new_reference() {
    let engine = EventEngine::new(packaged_backend());
    for reference in new_references() {
        for body in [CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mars] {
            for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
                let pos = engine.position_at(body.clone(), &reference, tdb(jd)).unwrap();
                let lon = engine.longitude_at(body.clone(), &reference, tdb(jd)).unwrap();
                assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{reference:?} {body:?} {jd}"
                );
                assert_eq!(pos.frame, reference.frame);
                assert_eq!(pos.zodiac, reference.zodiac);
            }
        }
    }
}

#[test]
fn sidereal_speed_is_the_tropical_speed_minus_the_ayanamsa_rate() {
    // Lahiri drifts with general precession, 3.82e-5 deg/day. In the apparent
    // frame the removed nutation adds a rate bounded by 6.0e-5 deg/day.
    const PRECESSION_DEG_PER_DAY: f64 = 3.82e-5;
    let engine = EventEngine::new(packaged_backend());
    let speed = |reference: CrossingReference, jd: f64| {
        engine
            .position_at(CelestialBody::Mars, reference, tdb(jd))
            .unwrap()
            .motion
            .longitude_deg_per_day
            .expect("speed")
    };
    for jd in [2_430_000.5, 2_451_545.0, 2_470_000.5] {
        let mean_drop = speed(MEAN.into(), jd) - speed(lahiri(MEAN), jd);
        assert!(
            (mean_drop - PRECESSION_DEG_PER_DAY).abs() < 2.0e-6,
            "jd {jd}: mean drop {mean_drop:e}"
        );
        let apparent_drop = speed(APPARENT.into(), jd) - speed(lahiri(APPARENT), jd);
        assert!(
            (apparent_drop - PRECESSION_DEG_PER_DAY).abs() < 6.0e-5,
            "jd {jd}: apparent drop {apparent_drop:e}"
        );
    }
}

#[test]
fn sidereal_position_has_a_speed_at_the_window_end() {
    // The later neighbour is outside the window, so the difference is
    // one-sided; the speed must still be reported.
    let engine = EventEngine::new(packaged_backend());
    let pos = engine
        .position_at(CelestialBody::Sun, lahiri(MEAN), tdb(WINDOW_END_JD))
        .unwrap();
    assert!(pos.motion.longitude_deg_per_day.is_some());
}

#[test]
fn one_reference_serves_many_calls_by_borrow() {
    let engine = EventEngine::new(packaged_backend());
    let reference = lahiri(APPARENT);
    let count = [0.0, 90.0, 180.0, 270.0]
        .into_iter()
        .filter_map(|target_deg| {
            engine
                .next_longitude_crossing(
                    CelestialBody::Sun,
                    Longitude::from_degrees(target_deg),
                    &reference,
                    tdb(2_451_545.0),
                )
                .unwrap()
        })
        .count();
    assert_eq!(count, 4);
}
```

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p pleiades-events --test reference`
Expected: FAIL to compile — `CrossingReference` not found in `pleiades_events`.

- [ ] **Step 4: Create `src/reference.rs`**

```rust
//! The frame and zodiac a longitude is measured in, and the one place both
//! become a position.
//!
//! A tropical reference returns the frame's place untouched, which keeps the
//! pre-existing frames bit-identical. A sidereal reference moves the longitude
//! to the mean equinox of date (removing nutation in longitude where the frame
//! carries it) and subtracts the mean ayanamsa — the Swiss Ephemeris
//! `SEFLG_SIDEREAL` convention. Latitude and distance never change.

use crate::crossings::{body_label, CrossingFrame};
use crate::ephemeris::{
    geocentric_apparent_ecliptic, geocentric_mean_of_date_ecliptic, heliocentric_j2000,
    heliocentric_of_date, j2000_spherical, read_mean_ecliptic_with_motion,
};
use crate::error::EventError;
use crate::state_vector::spherical_rates;
use pleiades_apparent::nutation::nutation;
use pleiades_ayanamsa::sidereal_offset;
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    Ayanamsa, CelestialBody, Instant, JulianDay, Motion, TimeScale, ZodiacMode,
};

/// `(longitude_deg, latitude_deg, distance_au)`.
type EclipticTriple = (f64, f64, f64);

/// The frame and zodiac a longitude is measured in.
///
/// A [`CrossingFrame`] converts into the tropical reference, so every method
/// that takes a reference also takes a bare frame:
///
/// ```
/// use pleiades_events::{CrossingFrame, CrossingReference};
/// use pleiades_types::{Ayanamsa, ZodiacMode};
///
/// let tropical: CrossingReference = CrossingFrame::GeocentricApparentOfDate.into();
/// assert_eq!(tropical.zodiac, ZodiacMode::Tropical);
///
/// let sidereal =
///     CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri);
/// assert_eq!(sidereal.frame, CrossingFrame::GeocentricApparentOfDate);
/// ```
///
/// A sidereal longitude is the longitude on the **mean** equinox of date minus
/// the mean ayanamsa (`pleiades_ayanamsa::sidereal_offset`), so nutation does
/// not move a body through a sidereal zodiac. This is the Swiss Ephemeris
/// `SEFLG_SIDEREAL` convention. A sidereal `pleiades-core` apparent chart
/// keeps nutation and can differ from it by up to about 17″.
///
/// The heliocentric frame is tropical only.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct CrossingReference {
    /// The coordinate and centre convention.
    pub frame: CrossingFrame,
    /// The zodiac longitudes are read in.
    pub zodiac: ZodiacMode,
}

impl CrossingReference {
    /// `frame` in the tropical zodiac; the same as `frame.into()`.
    pub fn tropical(frame: CrossingFrame) -> Self {
        Self {
            frame,
            zodiac: ZodiacMode::Tropical,
        }
    }

    /// `frame` in the sidereal zodiac of `ayanamsa`.
    pub fn sidereal(frame: CrossingFrame, ayanamsa: Ayanamsa) -> Self {
        Self {
            frame,
            zodiac: ZodiacMode::Sidereal { ayanamsa },
        }
    }
}

impl From<CrossingFrame> for CrossingReference {
    fn from(frame: CrossingFrame) -> Self {
        Self::tropical(frame)
    }
}

impl From<&CrossingReference> for CrossingReference {
    fn from(reference: &CrossingReference) -> Self {
        reference.clone()
    }
}

fn unsupported(detail: impl Into<String>) -> EventError {
    EventError::UnsupportedFrame {
        detail: detail.into(),
    }
}

/// Mean ayanamsa in degrees at `julian_day`.
fn ayanamsa_deg(ayanamsa: &Ayanamsa, julian_day: f64) -> Result<f64, EventError> {
    // `sidereal_offset` reads the day as TT; the engine's day is TDB. The two
    // differ by under 2 ms, which moves the ayanamsa by less than 1e-9″.
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tt);
    sidereal_offset(ayanamsa, instant)
        .map(|offset| offset.degrees())
        .ok_or_else(|| unsupported(format!("ayanamsa {ayanamsa} has no sidereal offset data")))
}

/// Fails for a body/frame/zodiac combination that is not defined. `what`
/// completes "heliocentric {what} undefined for {body}" (for example
/// `"crossings are"`). `julian_day` is the instant the ayanamsa is probed at.
pub(crate) fn check_supported(
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
    what: &str,
) -> Result<(), EventError> {
    let heliocentric = matches!(reference.frame, CrossingFrame::Heliocentric);
    if heliocentric && matches!(body, CelestialBody::Sun | CelestialBody::Moon) {
        return Err(unsupported(format!(
            "heliocentric {what} undefined for {body:?}"
        )));
    }
    match &reference.zodiac {
        ZodiacMode::Tropical => Ok(()),
        ZodiacMode::Sidereal { .. } if heliocentric => Err(unsupported(
            "a sidereal zodiac is not supported in the heliocentric frame",
        )),
        ZodiacMode::Sidereal { ayanamsa } => ayanamsa_deg(ayanamsa, julian_day).map(|_| ()),
        _ => Err(unsupported("unsupported zodiac mode")),
    }
}

/// Moves a frame's tropical place into the reference's zodiac.
fn in_zodiac(
    (lon, lat, dist): EclipticTriple,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let ayanamsa = match &reference.zodiac {
        // No arithmetic: the tropical frames stay bit-identical.
        ZodiacMode::Tropical => return Ok((lon, lat, dist)),
        ZodiacMode::Sidereal { ayanamsa } => ayanamsa,
        _ => return Err(unsupported("unsupported zodiac mode")),
    };
    // Nutation slides the equinox along the ecliptic, so removing Δψ from a
    // true-equinox longitude gives the mean-equinox longitude exactly.
    let nutation_deg = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            nutation(julian_day)
                .map_err(|e| EventError::Backend(format!("sidereal nutation failed: {e}")))?
                .delta_psi_arcsec
                / 3600.0
        }
        CrossingFrame::GeocentricMeanOfDate => 0.0,
        CrossingFrame::Heliocentric => {
            return Err(unsupported(
                "a sidereal zodiac is not supported in the heliocentric frame",
            ))
        }
    };
    let shift = nutation_deg + ayanamsa_deg(ayanamsa, julian_day)?;
    Ok(((lon - shift).rem_euclid(360.0), lat, dist))
}

/// Ecliptic `(longitude_deg, latitude_deg, distance_au)` of `body` in
/// `reference` at `julian_day` (TDB). The crossing engine root-finds on the
/// longitude; `longitude_at` and `position_at` report it.
pub(crate) fn ecliptic_in<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let label = body_label(body);
    let tropical = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            heliocentric_of_date(helio.position, julian_day)?
        }
        CrossingFrame::GeocentricMeanOfDate => {
            geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?
        }
    };
    in_zodiac(tropical, reference, julian_day)
}

/// A place with the base place and speed its own speed is derived from.
pub(crate) struct SampledPlace {
    /// The J2000 place the backend's speed describes.
    pub(crate) base: EclipticTriple,
    /// Speed of `base`, when the backend reports one.
    pub(crate) base_motion: Option<Motion>,
    /// The place in the reference; equal to [`ecliptic_in`].
    pub(crate) corrected: EclipticTriple,
}

/// [`ecliptic_in`] together with its base place and speed, for `position_at`.
pub(crate) fn sampled_place<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<SampledPlace, EventError> {
    let label = body_label(body);
    let (base, base_motion, tropical) = match reference.frame {
        CrossingFrame::GeocentricApparentOfDate => {
            let (mean, motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let apparent = geocentric_apparent_ecliptic(backend, body.clone(), label, julian_day)?;
            (mean, motion, apparent)
        }
        CrossingFrame::Heliocentric => {
            let helio = heliocentric_j2000(backend, body.clone(), label, julian_day)?;
            let of_date = heliocentric_of_date(helio.position, julian_day)?;
            (
                j2000_spherical(helio.position),
                helio
                    .velocity
                    .map(|velocity| spherical_rates(helio.position, velocity)),
                of_date,
            )
        }
        CrossingFrame::GeocentricMeanOfDate => {
            let (mean, motion) =
                read_mean_ecliptic_with_motion(backend, body.clone(), label, julian_day)?;
            let of_date =
                geocentric_mean_of_date_ecliptic(backend, body.clone(), label, julian_day)?;
            (mean, motion, of_date)
        }
    };
    Ok(SampledPlace {
        base,
        base_motion,
        corrected: in_zodiac(tropical, reference, julian_day)?,
    })
}

#[cfg(test)]
mod tests;
```

`Ayanamsa` implements `Display` (used in the error message). If `{ayanamsa}` does not compile, use `{ayanamsa:?}`.

- [ ] **Step 5: Create `src/reference/tests.rs`**

```rust
//! White-box checks that the reference module reproduces the pre-existing
//! frame functions bit for bit, and that its two entry points agree.

use super::{ecliptic_in, sampled_place, CrossingReference};
use crate::crossings::CrossingFrame;
use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use pleiades_data::packaged_backend;
use pleiades_types::{Ayanamsa, CelestialBody};

const CASES: [(CelestialBody, &str, f64); 4] = [
    (CelestialBody::Mercury, "Mercury", 2_415_100.25),
    (CelestialBody::Mars, "Mars", 2_451_545.0),
    (CelestialBody::Saturn, "Saturn", 2_439_500.066527),
    (CelestialBody::Pluto, "Pluto", 2_487_900.5),
];

#[test]
fn tropical_references_match_the_frame_wrappers_bitwise() {
    // `validate-crossings` root-finds on these values; a tropical reference
    // must not move them by a single bit.
    let backend = packaged_backend();
    for (body, label, jd) in CASES {
        let apparent =
            ecliptic_in(&backend, &body, &CrossingFrame::GeocentricApparentOfDate.into(), jd)
                .unwrap()
                .0;
        let wrapper = geocentric_apparent_longitude_deg(&backend, body.clone(), label, jd).unwrap();
        assert_eq!(apparent.to_bits(), wrapper.to_bits(), "{label} apparent");

        let helio = ecliptic_in(&backend, &body, &CrossingFrame::Heliocentric.into(), jd)
            .unwrap()
            .0;
        let wrapper = heliocentric_longitude_deg(&backend, body.clone(), label, jd).unwrap();
        assert_eq!(helio.to_bits(), wrapper.to_bits(), "{label} helio");
    }
}

#[test]
fn sampled_place_agrees_with_ecliptic_in() {
    let backend = packaged_backend();
    let references = [
        CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate),
        CrossingReference::tropical(CrossingFrame::Heliocentric),
        CrossingReference::tropical(CrossingFrame::GeocentricMeanOfDate),
        CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri),
        CrossingReference::sidereal(CrossingFrame::GeocentricMeanOfDate, Ayanamsa::Lahiri),
    ];
    for reference in &references {
        for (body, label, jd) in CASES {
            let place = sampled_place(&backend, &body, reference, jd).unwrap();
            let direct = ecliptic_in(&backend, &body, reference, jd).unwrap();
            assert_eq!(place.corrected.0.to_bits(), direct.0.to_bits(), "{reference:?} {label} lon");
            assert_eq!(place.corrected.1.to_bits(), direct.1.to_bits(), "{reference:?} {label} lat");
            assert_eq!(place.corrected.2.to_bits(), direct.2.to_bits(), "{reference:?} {label} dist");
        }
    }
}
```

- [ ] **Step 6: Register the module and re-export**

In `crates/pleiades-events/src/lib.rs`, add `mod reference;` between `mod position;` and `mod rise_trans;`, and after the `pub use position::EclipticPosition;` line add:

```rust
pub use reference::CrossingReference;
```

- [ ] **Step 7: Rewrite the crossing methods over a reference**

In `crates/pleiades-events/src/crossings.rs`:

1. Imports: replace the `use crate::ephemeris::{...}` line with
   `use crate::reference::{check_supported, ecliptic_in, CrossingReference};`
   and add `ZodiacMode` to the `pleiades_types` import. Inside `mod tests`, add
   `use crate::ephemeris::geocentric_apparent_longitude_deg;` (the tests still use it).
2. Delete the private `longitude_deg` method.
3. Add the field to `Crossing`, after `frame`:

```rust
    /// The zodiac `target_longitude` is read in.
    pub zodiac: ZodiacMode,
```

   and change the doc of `target_longitude` to `/// The ecliptic longitude that was crossed, in `zodiac`.`

4. Add a private constructor below `check_window`:

```rust
    fn crossing(
        body: &CelestialBody,
        target: Longitude,
        reference: &CrossingReference,
        jd: f64,
    ) -> Crossing {
        Crossing {
            body: body.clone(),
            target_longitude: target,
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            frame: reference.frame,
            zodiac: reference.zodiac.clone(),
        }
    }
```

5. Replace the bodies of the four methods. Each keeps its existing doc comment, to which this paragraph is appended:

```rust
    ///
    /// `reference` is a [`CrossingFrame`] (tropical zodiac) or a
    /// [`CrossingReference`] carrying a sidereal zodiac; `target` is read in
    /// that zodiac. A sidereal zodiac in the heliocentric frame, and an
    /// ayanamsa with no offset data, are [`EventError::UnsupportedFrame`].
```

```rust
    pub fn longitude_crossings_in_range(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Crossing>, EventError> {
        let reference = reference.into();
        let start_jd = start.julian_day.days();
        let end_jd = end.julian_day.days();
        self.check_window(start_jd)?;
        self.check_window(end_jd)?;
        check_supported(&body, &reference, start_jd, "crossings are")?;
        let step = Self::step_days(&body);
        // Clamp like the eclipse engine: keep retarded/aberration queries in-window.
        let scan_start = start_jd.max(WINDOW_START_JD + step);
        let scan_end = end_jd.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let roots = crossings_in_range(
            |jd| Ok(wrap180(ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(roots
            .into_iter()
            .map(|jd| Self::crossing(&body, target, &reference, jd))
            .collect())
    }

    pub fn next_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let reference = reference.into();
        let after_jd = after.julian_day.days();
        // Same checks, in the same order, as `longitude_crossings_in_range`.
        self.check_window(after_jd)?;
        self.check_window(WINDOW_END_JD)?;
        check_supported(&body, &reference, after_jd, "crossings are")?;
        let step = Self::step_days(&body);
        // Same clamps as `longitude_crossings_in_range` over `[after, WINDOW_END]`.
        let scan_start = after_jd.max(WINDOW_START_JD + step);
        let scan_end = WINDOW_END_JD.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let root = first_crossing_after(
            |jd| Ok(wrap180(ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(root
            .filter(|&jd| jd > after_jd)
            .map(|jd| Self::crossing(&body, target, &reference, jd)))
    }

    pub fn previous_longitude_crossing(
        &self,
        body: CelestialBody,
        target: Longitude,
        reference: impl Into<CrossingReference>,
        before: Instant,
    ) -> Result<Option<Crossing>, EventError> {
        let reference = reference.into();
        let before_jd = before.julian_day.days();
        // Same checks, in the same order, as `longitude_crossings_in_range`.
        self.check_window(WINDOW_START_JD)?;
        self.check_window(before_jd)?;
        check_supported(&body, &reference, before_jd, "crossings are")?;
        let step = Self::step_days(&body);
        // Same clamps as `longitude_crossings_in_range` over `[WINDOW_START, before]`.
        let scan_start = WINDOW_START_JD.max(WINDOW_START_JD + step);
        let scan_end = before_jd.min(WINDOW_END_JD - step);
        let target_deg = target.degrees();
        let root = last_crossing_before(
            |jd| Ok(wrap180(ecliptic_in(&self.backend, &body, &reference, jd)?.0 - target_deg)),
            scan_start,
            scan_end,
            step,
        )?;
        Ok(root
            .filter(|&jd| jd < before_jd)
            .map(|jd| Self::crossing(&body, target, &reference, jd)))
    }

    pub fn longitude_at(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        instant: Instant,
    ) -> Result<Longitude, EventError> {
        let reference = reference.into();
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        check_supported(&body, &reference, jd, "longitude is")?;
        let (deg, _, _) = ecliptic_in(&self.backend, &body, &reference, jd)?;
        Ok(Longitude::from_degrees(deg))
    }
```

   In the three search methods' doc comments, the phrase "`longitude_crossings_in_range(body, target, frame, …)`" becomes "`…(body, target, reference, …)`".

6. Add a sidereal example to `next_longitude_crossing`'s doc comment:

```rust
    /// ```
    /// use pleiades_data::packaged_backend;
    /// use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};
    /// use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale};
    ///
    /// // The Sun's next entry into sidereal Aries (Lahiri), after J2000.
    /// let engine = EventEngine::new(packaged_backend());
    /// let lahiri =
    ///     CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri);
    /// let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    /// let ingress = engine
    ///     .next_longitude_crossing(CelestialBody::Sun, Longitude::from_degrees(0.0), lahiri, after)
    ///     .unwrap()
    ///     .expect("the Sun enters sidereal Aries every year");
    /// // Mid-April, about 24 days after the tropical equinox.
    /// let days = ingress.instant.julian_day.days() - 2_451_545.0;
    /// assert!((100.0..110.0).contains(&days), "{days}");
    /// ```
```

- [ ] **Step 8: Rewrite `position.rs` over a reference**

In `crates/pleiades-events/src/position.rs`:

1. Imports become:

```rust
use crate::crossings::{CrossingFrame, EventEngine};
use crate::error::{EventError, WINDOW_END_JD, WINDOW_START_JD};
use crate::reference::{check_supported, sampled_place, CrossingReference};
use pleiades_apparent::motion::{apparent_motion, Correction, CorrectionSample, HALF_SPAN_DAYS};
use pleiades_backend::EphemerisBackend;
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, Latitude, Longitude, Motion, ZodiacMode,
};
```

   (`CrossingFrame` is still the type of `EclipticPosition::frame`.)

2. Add to `EclipticPosition`, after `frame`:

```rust
    /// The zodiac `ecliptic.longitude` is read in.
    pub zodiac: ZodiacMode,
```

3. Replace `sample` with:

```rust
fn sample<B: EphemerisBackend>(
    backend: &B,
    body: &CelestialBody,
    reference: &CrossingReference,
    julian_day: f64,
) -> Result<Sample, EventError> {
    let place = sampled_place(backend, body, reference, julian_day)?;
    Ok(Sample {
        base: coordinates(place.base),
        corrected: coordinates(place.corrected),
        base_motion: place.base_motion,
    })
}
```

4. In `motion`, change the parameter `frame: CrossingFrame` to `reference: &CrossingReference` and the inner call to `sample(backend, body, reference, jd)`.

5. Replace the body of `position_at` (signature: `reference: impl Into<CrossingReference>` in place of `frame: CrossingFrame`):

```rust
        let reference = reference.into();
        let jd = instant.julian_day.days();
        self.check_window(jd)?;
        check_supported(&body, &reference, jd, "position is")?;
        let centre = sample(&self.backend, &body, &reference, jd)?;
        let motion = motion(&self.backend, &body, &reference, jd, &centre);
        Ok(EclipticPosition {
            body,
            frame: reference.frame,
            zodiac: reference.zodiac,
            instant,
            ecliptic: centre.corrected,
            motion,
        })
```

6. Add to the module-doc table:

```
//! | any geocentric frame, sidereal zodiac | as the frame | the frame's place − Δψ (apparent only) − mean ayanamsa |
```

   and to the `position_at` rustdoc, after the frame bullets:

```rust
    /// With a sidereal [`CrossingReference`] the longitude is the frame's
    /// longitude on the mean equinox of date minus the mean ayanamsa, and the
    /// speed drops by the ayanamsa's rate. `# Errors` gains: a sidereal zodiac
    /// in the heliocentric frame, and an ayanamsa with no offset data, are
    /// [`EventError::UnsupportedFrame`].
```

7. In `crates/pleiades-events/src/position/tests.rs`, bind the reference once and pass it by borrow:

```rust
    let helio = CrossingFrame::Heliocentric.into();
    // ...
        let centre = sample(&backend, &body, &helio, jd).expect("sample");
    // ...
        let of_date = motion(&backend, &body, &helio, jd, &centre)
```

   with `let helio: crate::reference::CrossingReference = ...` if inference needs the annotation.

- [ ] **Step 9: Build and resolve dead code**

Run: `cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings`

If clippy reports `heliocentric_longitude_deg` (in `ephemeris.rs`) as unused outside tests, put `#[cfg(test)]` on that function and extend its doc comment with: "Test-only since the reference module took over the crossing path; kept as the pinned-bits oracle." Do not delete it — `heliocentric_longitude_bits_are_pinned` and the new white-box test use it. Remove any other now-unused import the lint names.

- [ ] **Step 10: Run the tests**

Run: `cargo test -p pleiades-events`
Expected: all pass, including `reference::tests::*`, `--test reference` (18 tests), `--test position`, `--test heliocentric` and the doctests.

If `mars_2003_loop_gives_three_sidereal_crossings` finds a count other than 3, print the crossings and the tropical triple (`CrossingFrame::GeocentricApparentOfDate`, target 337.0, same range) before changing anything: the sidereal count must equal the tropical count.

Run: `cargo test -p pleiades-events --features serde`
Expected: PASS.

Run: `cargo test -p pleiades-validate crossings_validation && cargo test -p pleiades-cli crossings`
Expected: PASS — the 86-row golden is unchanged.

Run: `cargo check --workspace --all-targets --all-features`
Expected: builds. Any other crate that names `Crossing { .. }` or `EclipticPosition { .. }` exhaustively cannot exist (both are `#[non_exhaustive]`); a failure here is a call site passing something that is not `Into<CrossingReference>`.

- [ ] **Step 11: Lint and commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add crates/pleiades-events Cargo.lock
git commit -m "feat(events): crossings and positions in a sidereal zodiac via CrossingReference (#88)"
```

---

### Task 3: Swiss Ephemeris reference rows and the `validate-crossings` gate

**Files:**
- Modify: `tools/se-crossings-reference/src/main.rs`
- Regenerate: `crates/pleiades-validate/data/crossings-corpus/crossings.csv`, `manifest.txt`
- Modify: `crates/pleiades-validate/src/crossings_validation.rs`
- Modify: `crates/pleiades-validate/src/render/cli.rs:11-97` (golden helpers), `:442-475` (comment), help text near `:2281`, tests near `:2878-2960`

**Interfaces:**
- Consumes: `CrossingReference::tropical`, `CrossingReference::sidereal`, `CrossingFrame::GeocentricMeanOfDate` (Tasks 1–2).
- Produces:
  - CSV schema, SE form (7 columns): `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac`; corpus form adds an 8th column `pleiades_jd_tdb`. `frame` ∈ `geo`, `helio`, `geo-mean`. `zodiac` ∈ `tropical`, `Lahiri`, `TrueCitra`, `GalacticCenter`, `DeLuce`.
  - `pub(crate) fn parse_reference(frame: &str, zodiac: &str) -> Option<CrossingReference>` in `crossings_validation.rs`, used by `render/cli.rs`.

The zodiac column sits after `crossing_jd_tdb` so that fields 0–5 keep their positions.

- [ ] **Step 1: Save the committed corpus for the unchanged-rows check**

```bash
mkdir -p target/crossings-check
git show main:crates/pleiades-validate/data/crossings-corpus/crossings.csv > target/crossings-check/old.csv
```

(`target/` is git-ignored.)

- [ ] **Step 2: Extend the reference tool**

In `tools/se-crossings-reference/src/main.rs`:

1. Update the module doc: add after the `Frame helio` paragraph

```rust
//! Frame `geo-mean`: geocentric geometric longitude on the mean equinox of
//!   date (SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL | SEFLG_NONUT),
//!   bisected on `swe_calc` like the geocentric planets.
//! Column `zodiac`: `tropical`, or an ayanamsa name for rows computed with
//!   SEFLG_SIDEREAL after `swe_set_sid_mode`. Swiss Ephemeris drops nutation
//!   from sidereal positions, so a sidereal longitude is the mean-equinox
//!   longitude minus the mean ayanamsa.
//!
//! Two build/run caveats: under devenv's gcc the build needs `CFLAGS=-std=gnu17`
//! (libswisseph-sys otherwise fails with a conflicting `getenv` declaration), and
//! `devenv shell` prints a banner line to stdout that must be removed from the
//! top of the CSV (the file must start with the `# Source:` line).
```

2. Import `swe_set_sid_mode` alongside the other `libswisseph_sys::raw` items and add constants:

```rust
const SEFLG_TRUEPOS: c_int = 16; // geometric: no light-time
const SEFLG_NONUT: c_int = 64; // mean equinox of date
const SEFLG_NOGDEFL: c_int = 512; // no gravitational deflection
const SEFLG_NOABERR: c_int = 1024; // no annual aberration
const SEFLG_SIDEREAL: c_int = 64 * 1024;

/// Geometric place: the `geo-mean` frame before the equinox choice.
const GEOMETRIC: c_int = SEFLG_TRUEPOS | SEFLG_NOABERR | SEFLG_NOGDEFL;

const SE_SUN: c_int = 0;
const SE_MOON: c_int = 1;
```

3. Generalise the longitude and bisection functions over the flags, keeping the existing ones as wrappers so the 86 existing rows are computed by the same arithmetic:

```rust
/// Geocentric longitude of `ipl` (degrees, [0,360)) at TDB under `iflag`.
fn flagged_longitude(jd_tdb: f64, ipl: c_int, iflag: c_int) -> f64 {
    let mut xx = [0.0_f64; 6];
    let mut serr = [0_i8; 256];
    let ret = unsafe {
        swe_calc(jd_tdb, ipl, iflag, xx.as_mut_ptr(), serr.as_mut_ptr() as *mut c_char)
    };
    if ret < 0 {
        panic!(
            "swe_calc(ipl={ipl}, iflag={iflag}) failed at jd_tdb={jd_tdb}: {}",
            serr_string(&serr)
        );
    }
    assert!(xx[0].is_finite(), "non-finite longitude at jd_tdb={jd_tdb}");
    xx[0].rem_euclid(360.0)
}

fn geo_longitude(jd_tdb: f64, ipl: c_int) -> f64 {
    flagged_longitude(jd_tdb, ipl, SEFLG_MOSEPH)
}
```

   Rename the body of `geo_planet_cross_tdb` to `flagged_cross_tdb(ipl: c_int, iflag: c_int, target_deg: f64, start_tdb: f64) -> f64`, replacing its three `geo_longitude(x, ipl)` calls with `flagged_longitude(x, ipl, iflag)` and its panic text with one that prints `iflag`, then:

```rust
fn geo_planet_cross_tdb(ipl: c_int, target_deg: f64, start_tdb: f64) -> f64 {
    flagged_cross_tdb(ipl, SEFLG_MOSEPH, target_deg, start_tdb)
}
```

4. `emit` gains a `zodiac: &str` parameter printed as the last field:

```rust
    println!("{frame},{body},{target:.6},{start_tdb:.6},fwd,{crossing_tdb:.9},{zodiac}");
```

   Every existing `emit(...)` call gets `"tropical"` as its new last argument. The header line becomes
   `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac`, and two comment lines are added to the header block:

```rust
    println!("# geo-mean: bisection on swe_calc with SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT.");
    println!("# zodiac != tropical: swe_set_sid_mode + SEFLG_SIDEREAL (nutation-free; mean ayanamsa).");
```

5. At the end of `main`, after the heliocentric loop, append the new rows:

```rust
    // --- geo-mean tropical: geometric place, mean equinox of date. ---
    let mean_tropical = SEFLG_MOSEPH | GEOMETRIC | SEFLG_NONUT;
    for &start in &sun_starts {
        for &t in &[0.0_f64, 90.0, 137.5] {
            emit("geo-mean", "Sun", t, start, flagged_cross_tdb(SE_SUN, mean_tropical, t, start), "tropical");
        }
    }
    for &start in &moon_starts {
        for &t in &[0.0_f64, 180.0, 45.0] {
            emit("geo-mean", "Moon", t, start, flagged_cross_tdb(SE_MOON, mean_tropical, t, start), "tropical");
        }
    }
    let mut mars_prev = f64::NEG_INFINITY;
    for &start in &mars_starts {
        let c = flagged_cross_tdb(SE_MARS, mean_tropical, mars_target, start);
        assert!(c > mars_prev + 1.0, "geo-mean Mars crossings not distinct: {c} vs {mars_prev}");
        mars_prev = c;
        emit("geo-mean", "Mars", mars_target, start, c, "tropical");
    }
    for &t in &geo_planet_targets {
        let c = flagged_cross_tdb(SE_JUPITER, mean_tropical, t, geo_planet_start);
        emit("geo-mean", "Jupiter", t, geo_planet_start, c, "tropical");
    }

    // --- sidereal: one ayanamsa per pleiades-ayanamsa computation class. ---
    // (name, SE_SIDM): OffsetDefined, TrueStar, Galactic, FittedOffset.
    let ayanamsas: [(&str, c_int); 4] = [
        ("Lahiri", 1),
        ("TrueCitra", 27),
        ("GalacticCenter", 17),
        ("DeLuce", 2),
    ];
    let places: [(&str, c_int); 2] = [
        ("geo", SEFLG_MOSEPH | SEFLG_SIDEREAL),
        ("geo-mean", SEFLG_MOSEPH | SEFLG_SIDEREAL | GEOMETRIC),
    ];
    for &(zodiac, sid_mode) in &ayanamsas {
        unsafe { swe_set_sid_mode(sid_mode, 0.0, 0.0) };
        for &(frame, iflag) in &places {
            for &start in &[2_416_000.5_f64, 2_470_000.5] {
                for &t in &[0.0_f64, 137.5] {
                    emit(frame, "Sun", t, start, flagged_cross_tdb(SE_SUN, iflag, t, start), zodiac);
                }
            }
            for &start in &[2_420_000.5_f64, 2_480_000.5] {
                emit(frame, "Moon", 45.0, start, flagged_cross_tdb(SE_MOON, iflag, 45.0, start), zodiac);
            }
            let c = flagged_cross_tdb(SE_JUPITER, iflag, 120.0, geo_planet_start);
            emit(frame, "Jupiter", 120.0, geo_planet_start, c, zodiac);
        }
    }

    // --- sidereal Mars retrograde triple-crossing (Lahiri, apparent). ---
    // Tropical 337.0 deg is 313.1 deg in the Lahiri zodiac in 2003
    // (ayanamsa 23.9 deg), so the same three starts bracket the same loop.
    unsafe { swe_set_sid_mode(1, 0.0, 0.0) };
    let sidereal_mars_target = 313.1_f64;
    let mut mars_prev = f64::NEG_INFINITY;
    for &start in &mars_starts {
        let c = flagged_cross_tdb(SE_MARS, SEFLG_MOSEPH | SEFLG_SIDEREAL, sidereal_mars_target, start);
        assert!(c > mars_prev + 1.0, "sidereal Mars crossings not distinct: {c} vs {mars_prev}");
        mars_prev = c;
        emit("geo", "Mars", sidereal_mars_target, start, c, "Lahiri");
    }
```

   `sun_starts`, `moon_starts`, `mars_starts`, `mars_target`, `geo_planet_targets` and `geo_planet_start` are the existing locals of `main`; the first `mars_prev` already exists, so the two new ones shadow it (allowed).

   Row count: 86 existing + 24 `geo-mean` tropical (9 Sun, 9 Moon, 3 Mars, 3 Jupiter) + 56 sidereal (4 ayanamsas × 2 places × 7) + 3 sidereal Mars = **169**.

- [ ] **Step 3: Generate the SE reference**

```bash
devenv shell -- env CFLAGS=-std=gnu17 cargo run --release \
  --manifest-path tools/se-crossings-reference/Cargo.toml \
  > target/crossings-check/se.csv
```

If `devenv` is not available in this environment, STOP and report: the corpus cannot be produced another way.

Open `target/crossings-check/se.csv` and delete any banner line above `# Source:`.

Run: `grep -vc '^#' target/crossings-check/se.csv`
Expected: `170` (169 data rows + the header).

- [ ] **Step 4: Verify the 86 existing rows are unchanged**

```bash
grep -v '^#' target/crossings-check/old.csv | cut -d, -f1-6 | tail -n +2 > target/crossings-check/old6.csv
grep -v '^#' target/crossings-check/se.csv | grep ',tropical$' | grep -v '^geo-mean,' | cut -d, -f1-6 > target/crossings-check/new6.csv
diff target/crossings-check/old6.csv target/crossings-check/new6.csv
```

Expected: no output and 86 lines in each file (`wc -l`). Any difference means the refactor of the tool changed an existing reference value: STOP and fix the tool.

- [ ] **Step 5: Write the failing gate tests**

In `crates/pleiades-validate/src/crossings_validation.rs`, in `mod tests`:

1. Change every literal CSV in the existing tests to the 8-column form: the header becomes
   `frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb`
   and each data row gains `,tropical` before its last field, for example
   `geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,tropical,2416199.301931810`,
   `geo,Sun,10.000000,2416000.500000,fwd,2416195.301931810,tropical,PLEIADES`,
   `geo,Sun,0.0,2416000.5,bwd,2416195.3,tropical,2416195.3`, and the short row
   `geo,Sun,0.0,2416000.5,fwd,2416195.3,tropical` (7 fields, still a schema error).
2. In `fill_golden_for_test`, replace the `let frame = if f[0] == "geo" {...}` block with
   `let reference = parse_reference(f[0], f[6]).unwrap();` and pass `reference` to `next_longitude_crossing`; drop the inner `use pleiades_events::{CrossingFrame, EventEngine};`.
3. Add:

```rust
    #[test]
    fn parse_reference_reads_frames_and_zodiacs() {
        use pleiades_types::{Ayanamsa, ZodiacMode};
        let geo = parse_reference("geo", "tropical").unwrap();
        assert_eq!(geo, CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate));
        let mean = parse_reference("geo-mean", "Lahiri").unwrap();
        assert_eq!(mean.frame, CrossingFrame::GeocentricMeanOfDate);
        assert_eq!(mean.zodiac, ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri });
        for name in ["TrueCitra", "GalacticCenter", "DeLuce"] {
            assert!(parse_reference("geo", name).is_some(), "{name}");
        }
        assert!(parse_reference("geo", "Nonesuch").is_none());
        assert!(parse_reference("lunar", "tropical").is_none());
        // The heliocentric frame is tropical only.
        assert!(parse_reference("helio", "Lahiri").is_none());
    }

    #[test]
    fn unknown_zodiac_is_a_schema_error() {
        let csv = "geo,Sun,0.0,2416000.5,fwd,2416195.3,Nonesuch,2416195.3\n";
        assert!(matches!(
            validate_crossings_csv(csv).unwrap_err(),
            CrossingsCorpusError::Schema { .. }
        ));
    }

    #[test]
    fn tier2_honours_the_zodiac_column() {
        // The SE time below is the tropical 0° crossing. Labelled Lahiri, the
        // engine's sidereal longitude there is about 24° short of the target.
        let csv = "\
frame,body,target_longitude_deg,start_jd_tdb,direction,crossing_jd_tdb,zodiac,pleiades_jd_tdb
geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,Lahiri,PLEIADES
";
        let csv = fill_golden_for_test(csv);
        let err = validate_crossings_csv(&csv).unwrap_err();
        assert!(matches!(err, CrossingsCorpusError::ParityExceeded { .. }), "{err:?}");
    }
```

Run: `cargo test -p pleiades-validate crossings_validation`
Expected: FAIL to compile — `parse_reference` not found.

- [ ] **Step 6: Update the gate**

In `crates/pleiades-validate/src/crossings_validation.rs`:

1. Imports: `use pleiades_events::{CrossingFrame, CrossingReference, EventEngine};` and add `Ayanamsa, ZodiacMode` to the `pleiades_types` import.
2. Module doc: change "committed `pleiades_jd_tdb` golden" context to mention the `zodiac` column and the `geo-mean` frame in one added sentence: `Rows carry a frame (`geo`, `helio`, `geo-mean`) and a zodiac (`tropical` or an ayanamsa name).`
3. Add the parser:

```rust
/// Reads a corpus row's `frame` and `zodiac` fields. `None` for an unknown
/// name, and for a sidereal heliocentric row (that frame is tropical only).
pub(crate) fn parse_reference(frame: &str, zodiac: &str) -> Option<CrossingReference> {
    let frame = match frame {
        "geo" => CrossingFrame::GeocentricApparentOfDate,
        "helio" => CrossingFrame::Heliocentric,
        "geo-mean" => CrossingFrame::GeocentricMeanOfDate,
        _ => return None,
    };
    let ayanamsa = match zodiac {
        "tropical" => return Some(CrossingReference::tropical(frame)),
        "Lahiri" => Ayanamsa::Lahiri,
        "TrueCitra" => Ayanamsa::TrueCitra,
        "GalacticCenter" => Ayanamsa::GalacticCenter,
        "DeLuce" => Ayanamsa::DeLuce,
        _ => return None,
    };
    if frame == CrossingFrame::Heliocentric {
        return None;
    }
    Some(CrossingReference::sidereal(frame, ayanamsa))
}
```

4. In `validate_crossings_csv`: the doc says "8-column"; the arity check becomes `f.len() != 8`; the `let frame = match f[0] {...}` block becomes

```rust
        let reference =
            parse_reference(f[0], f[6].trim()).ok_or_else(|| CrossingsCorpusError::Schema {
                row: line.to_string(),
            })?;
```

   the golden is read from `f[7]`; both engine calls take `&reference`; and the ceiling lookup becomes `arcsec_ceiling_for(&reference, &body)`.

5. Replace `arcsec_ceiling_for` so that the tropical `geo`/`helio` groups keep today's constants and the new groups get their own. Use these provisional values so the measurement in Step 8 can run; Step 8 replaces them:

```rust
// Provisional until measured in this change; see the measured block below.
const GEO_MEAN_SUN_ARCSEC: f64 = 60.0;
const GEO_MEAN_MOON_ARCSEC: f64 = 60.0;
const GEO_MEAN_PLANET_ARCSEC: f64 = 60.0;
const SIDEREAL_SUN_ARCSEC: f64 = 60.0;
const SIDEREAL_MOON_ARCSEC: f64 = 60.0;
const SIDEREAL_PLANET_ARCSEC: f64 = 60.0;

fn arcsec_ceiling_for(reference: &CrossingReference, body: &CelestialBody) -> f64 {
    let sidereal = !matches!(reference.zodiac, ZodiacMode::Tropical);
    match (reference.frame, sidereal) {
        (CrossingFrame::Heliocentric, _) => match body {
            CelestialBody::Pluto => PLUTO_ARCSEC,
            _ => HELIO_ARCSEC,
        },
        (CrossingFrame::GeocentricApparentOfDate, false) => match body {
            CelestialBody::Sun => GEO_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MOON_ARCSEC,
            CelestialBody::Pluto => PLUTO_ARCSEC,
            _ => GEO_PLANET_ARCSEC,
        },
        (CrossingFrame::GeocentricMeanOfDate, false) => match body {
            CelestialBody::Sun => GEO_MEAN_SUN_ARCSEC,
            CelestialBody::Moon => GEO_MEAN_MOON_ARCSEC,
            _ => GEO_MEAN_PLANET_ARCSEC,
        },
        // Sidereal rows of either geocentric place.
        (_, true) => match body {
            CelestialBody::Sun => SIDEREAL_SUN_ARCSEC,
            CelestialBody::Moon => SIDEREAL_MOON_ARCSEC,
            _ => SIDEREAL_PLANET_ARCSEC,
        },
        // `CrossingFrame` is `#[non_exhaustive]`; a future frame falls back to
        // the planet ceiling until it gets rows of its own.
        _ => GEO_PLANET_ARCSEC,
    }
}
```

6. `EXPECTED_ROWS` becomes `169`.
7. In `measure_per_group_parity`: parse with `parse_reference(f[0], f[6].trim()).expect("reference")`, pass `&reference` to `longitude_at`, and group by `format!("{}/{}/{}", f[0], f[6].trim(), f[1])`.

- [ ] **Step 7: Update the golden helpers**

In `crates/pleiades-validate/src/render/cli.rs`:

1. `append_golden_column`: doc says "Input is the 7-column SE CSV; output is the 8-column corpus"; the arity check becomes `f.len() != 7` with the message `expected 7 fields`; replace the `let frame = match f[0] {...}` block with

```rust
        let reference = crate::crossings_validation::parse_reference(f[0], f[6].trim())
            .ok_or_else(|| format!("unknown frame or zodiac: {trimmed}"))?;
```

   pass `reference` to `next_longitude_crossing`, and reduce the inner `use` to `use pleiades_events::EventEngine;`.
2. `strip_golden_column`: `if f.len() == 8 { f.truncate(7); }`, doc "back to the 7-column SE form".
3. The `crossings-golden` arm's comment: "Strip any existing 8th column back to the 7-column SE form"; rename the local `six` to `se_form`.
4. Tests near line 2878: every literal header gains `,zodiac`; every data row gains `,tropical` (`geo,Sun,0.000000,2416000.500000,fwd,2416195.301931810,tropical`); the malformed row stays 5 fields; `assert_eq!(fields.len(), 7, ...)` becomes `8` and `fields[6]` becomes `fields[7]`; the stripped-header assertion expects `,zodiac` as the last column in place of `,crossing_jd_tdb` (update its message and the comment above it); rename the locals `six`/`seven`/`seven2` to `se_form`/`corpus`/`corpus2`.
5. Help text near line 2281: replace `over the committed 86-row geo+helio crossings corpus` with `over the committed crossings corpus (geocentric apparent and mean of date, heliocentric; tropical and sidereal)`.

Run: `grep -rn "86-row\|geo+helio" crates --include=*.rs`
Expected: only the dated measurement comment in `crossings_validation.rs` (line 36), which is rewritten in Step 8.

- [ ] **Step 8: Install the corpus, write the golden, measure, set ceilings**

```bash
cp target/crossings-check/se.csv crates/pleiades-validate/data/crossings-corpus/crossings.csv
cargo run -q -p pleiades-validate -- crossings-golden --regenerate
```

Expected output: `crossings-golden: wrote …/crossings.csv; manifest checksum= <N>`.

Confirm the 86 existing goldens did not move:

```bash
grep -v '^#' target/crossings-check/old.csv | tail -n +2 | cut -d, -f7 > target/crossings-check/old-golden.txt
grep -v '^#' crates/pleiades-validate/data/crossings-corpus/crossings.csv | grep ',tropical,' | grep -v '^geo-mean,' | cut -d, -f8 > target/crossings-check/new-golden.txt
diff target/crossings-check/old-golden.txt target/crossings-check/new-golden.txt
```

Expected: no output. A difference means Tasks 1–2 moved an existing crossing: STOP.

Edit `crates/pleiades-validate/data/crossings-corpus/manifest.txt`:

- `rows: 169`
- `checksum=<N>` from the command output
- `frames: geo (apparent of date), geo-mean (geometric, mean equinox of date), helio (SEFLG_HELCTR)`
- add `zodiacs: tropical; sidereal Lahiri, TrueCitra, GalacticCenter, DeLuce (SEFLG_SIDEREAL, nutation-free, mean ayanamsa)`
- add the two `#` comment lines the tool now prints to the comment block at the top.

Measure:

Run: `cargo test -p pleiades-validate crossings_validation::tests::measure_per_group_parity -- --nocapture --ignored`

Record the printed per-group maxima.

**Stop condition (spec).** For each sidereal group, compare its maximum with its tropical counterpart (the same frame and body with zodiac `tropical`) plus the ayanamsa gate's ceiling for the class: Lahiri 3.0″ (`OffsetDefined`), TrueCitra 1.0″, GalacticCenter 1.0″, DeLuce 1.0″ (`pleiades_ayanamsa::ayanamsa_mode_ceiling`). If any sidereal group exceeds that sum, STOP and report the group, its maximum and the row: do not set a ceiling over it. The likely causes are a nutation term (a signature near ±17″ that varies with an 18.6-year period) or an aberration term (≈ 20″), and either one means the engine and Swiss Ephemeris disagree about the convention.

Likewise, if a `geo-mean` tropical group exceeds roughly twice its `geo` tropical counterpart, STOP and report.

Otherwise set each of the six provisional constants to `ceil(1.4 × the largest group maximum it covers)` (for the three `SIDEREAL_*` constants, the largest over all ayanamsas and both places for that body class), and replace the "Provisional" comment with a measured block in the style of the existing one:

```rust
// Measured group maxima for the mean-of-date and sidereal rows (169-row corpus,
// <date>): geo-mean Sun <x>", geo-mean Moon <x>", geo-mean planets <x>";
// sidereal Sun <x>", sidereal Moon <x>", sidereal planets <x>" (largest over
// Lahiri, TrueCitra, GalacticCenter, DeLuce and both geocentric places).
// Ceilings are ceil(1.4x each).
```

with the measured numbers written in. In the existing comment at line 36, change "86-row corpus" to "the 86 tropical geo/helio rows".

- [ ] **Step 9: Run the gate and its tests**

Run: `cargo test -p pleiades-validate crossings_validation`
Expected: PASS, `validate_crossings_passes_over_committed_corpus` checking 169 rows.

Run: `cargo test -p pleiades-validate render::cli`
Expected: PASS.

Run: `cargo run -q -p pleiades-validate -- crossings-golden --check`
Expected: `crossings-golden: committed golden column is current`.

Run: `cargo run -q -p pleiades-validate -- validate-crossings`
Expected: the summary line with `169 SE crossing fixtures`.

Run: `cargo test -p pleiades-cli crossings`
Expected: PASS.

- [ ] **Step 10: Lint and commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
git add tools/se-crossings-reference crates/pleiades-validate
git commit -m "feat(validate): mean-of-date and sidereal rows in the crossings corpus (#88)" \
  -m "Measured maxima: <the per-group lines from Step 8>"
```

---

### Task 4: Documentation, compatibility profile, follow-ups and full verification

**Files:**
- Modify: `README.md:35`
- Modify: `crates/pleiades-events/README.md`
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id line 26, checksum constant, release-note list near line 112)
- Modify: `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440`, `crates/pleiades-validate/src/tests/render_request.rs:333`
- Modify: `crates/pleiades-events/tests/reference.rs` (ignored diagnostic)
- Modify: `docs/follow-ups.md` (new FU-18 at the end)
- Modify: `docs/superpowers/specs/2026-10-01-crossings-mean-sidereal-design.md` (status line)

**Interfaces:**
- Consumes: the measured maxima from Task 3.
- Produces: nothing code-facing.

- [ ] **Step 1: Measure the two chart-layer differences**

The follow-up entry must state measured numbers. Append to `crates/pleiades-events/tests/reference.rs` (add `use pleiades_core::{ChartEngine, ChartRequest};` and `Apparentness` to the `pleiades_types` import):

```rust
/// Diagnostic for FU-18: how a sidereal chart differs from a sidereal
/// crossing reference. Run with
/// `cargo test -p pleiades-events --test reference measure_chart_sidereal_conventions -- --nocapture --ignored`
#[test]
#[ignore]
fn measure_chart_sidereal_conventions() {
    let engine = EventEngine::new(packaged_backend());
    let zodiac = ZodiacMode::Sidereal { ayanamsa: Ayanamsa::Lahiri };
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        for (label, apparentness, frame) in [
            ("apparent", Apparentness::Apparent, APPARENT),
            ("mean", Apparentness::Mean, MEAN),
        ] {
            let request = ChartRequest::new(tdb(jd))
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(apparentness)
                .with_zodiac_mode(zodiac.clone());
            let chart = ChartEngine::new(packaged_backend()).chart(&request).expect("chart");
            let chart_lon = chart
                .placement_for(&CelestialBody::Sun)
                .expect("placed")
                .position
                .ecliptic
                .as_ref()
                .expect("ecliptic")
                .longitude
                .degrees();
            let engine_lon = engine
                .longitude_at(CelestialBody::Sun, lahiri(frame), tdb(jd))
                .unwrap()
                .degrees();
            let delta_psi = nutation(jd).unwrap().delta_psi_arcsec;
            eprintln!(
                "jd {jd} {label}: chart - crossing = {:.3}\" (Δψ = {delta_psi:.3}\")",
                wrap(chart_lon - engine_lon) * 3600.0
            );
        }
    }
}
```

Run: `cargo test -p pleiades-events --test reference measure_chart_sidereal_conventions -- --nocapture --ignored`

Record the eight lines. Expected shape, to be confirmed by the output: the `apparent` difference equals Δψ at each epoch; the `mean` difference is the precession accumulated since J2000 (about −50.3″ per year from 2000, so roughly +4000″ in 1913 and −4000″ in 2077) with the sign showing which side of J2000 the epoch is on. Write the entry in Step 5 from what is actually printed, not from this expectation. If a difference is zero, that follow-up item does not exist: leave it out and say so in the final summary.

- [ ] **Step 2: README capability table**

In `README.md`, replace the row beginning `| Longitude crossings |` (line 35) with:

```markdown
| Longitude crossings (geocentric apparent or mean of date, heliocentric; tropical or sidereal) | [`pleiades-events`](crates/pleiades-events) | `validate-crossings` | arcsecond-class |
```

If Task 3 measured any new group above 10″, write the accuracy cell as `arcsecond-class tropical; <N>″ sidereal` with the measured ceiling.

- [ ] **Step 3: Crate README**

In `crates/pleiades-events/README.md`, add a bullet after the `Heliocentric` bullet:

```markdown
- `GeocentricMeanOfDate`: the geometric place from the Earth's centre (no
  light-time, no aberration, no nutation) in the mean ecliptic and equinox of
  date. This is not the J2000 longitude a `pleiades-core` mean chart reports.
```

and this paragraph after the line `A speed channel is `None` when the backend reports no speed to derive it from.`:

```markdown
Every crossing and position method takes a `CrossingFrame` (tropical zodiac) or
a `CrossingReference`, which adds a zodiac:

    let lahiri = CrossingReference::sidereal(
        CrossingFrame::GeocentricApparentOfDate,
        Ayanamsa::Lahiri,
    );
    let ingress = engine.next_longitude_crossing(
        CelestialBody::Sun,
        Longitude::from_degrees(0.0),
        lahiri,
        after,
    )?;

A sidereal longitude is the longitude on the mean equinox of date minus the
mean ayanamsa, the Swiss Ephemeris `SEFLG_SIDEREAL` convention: nutation does
not move a body through a sidereal zodiac. The ayanamsa is evaluated at every
trial instant of the search. The heliocentric frame is tropical only, and an
ayanamsa without offset data is an error, never a silent tropical result.
Mean-of-date and sidereal crossings are gated by `validate-crossings`.
```

- [ ] **Step 4: Compatibility profile**

In `crates/pleiades-core/src/compatibility/mod.rs`:

- Line 26: change `pleiades-compatibility-profile/0.7.17` to `pleiades-compatibility-profile/0.7.18`.
- After the release-note string that begins `"Ecliptic position with latitude and speed (issue #89) additions:` add a new element in the same list:

```rust
            "Mean-place and sidereal crossings (issue #88) additions: CrossingFrame::GeocentricMeanOfDate is the geocentric geometric place in the mean ecliptic and equinox of date (no light-time, aberration or nutation; Swiss Ephemeris SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT). CrossingReference pairs a CrossingFrame with a ZodiacMode, and EventEngine's longitude_crossings_in_range, next_longitude_crossing, previous_longitude_crossing, longitude_at and position_at accept either a bare frame (tropical, results unchanged) or a reference. A sidereal longitude is the mean-equinox longitude minus the mean ayanamsa (Swiss Ephemeris SEFLG_SIDEREAL), evaluated at every trial instant. Heliocentric sidereal references and ayanamsas without offset data are rejected. Gated by validate-crossings against Swiss Ephemeris mean-of-date and sidereal rows (Lahiri, TrueCitra, GalacticCenter, DeLuce).",
```

- In `crates/pleiades-cli/src/cli/tests/summary_commands.rs:440` and `crates/pleiades-validate/src/tests/render_request.rs:333`, change `0.7.17` to `0.7.18`.

Run: `cargo test -p pleiades-core rendered_profile_matches_pinned_content_checksum`
Expected: FAIL, reporting the new checksum. Set `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` to the reported value (keep the `0x____ ____ ____ ____` underscore grouping), following the instructions in that constant's doc comment, and re-run.
Expected: PASS.

Run: `grep -rn "0\.7\.17" --include=*.rs --include=*.md . | grep -v "^./target" | grep -v "docs/superpowers"`
Expected: no output.

- [ ] **Step 5: Follow-ups entry and spec status**

Append to `docs/follow-ups.md`, with the numbers from Step 1 written in place of each bracketed measurement and with any item whose difference measured zero removed:

```markdown
---

## FU-18: Sidereal chart conventions differ from sidereal crossings (issue #88)

**Status:** open · **Opened:** 2026-10-01

Issue #88 gave `pleiades-events` sidereal crossings in the Swiss Ephemeris
convention: the longitude on the mean equinox of date minus the mean ayanamsa.
The chart layer (`pleiades-core` `src/chart/sidereal.rs`) subtracts the same
mean ayanamsa from whatever longitude the chart holds, which gives two
differences. Both were measured for the Sun with Lahiri by
`measure_chart_sidereal_conventions` in
`crates/pleiades-events/tests/reference.rs`:

- **(a) A sidereal apparent chart keeps nutation.** The chart longitude is on
  the true equinox, so it exceeds the crossing engine's sidereal longitude by
  Δψ: measured [four values with their epochs], equal to Δψ at each epoch.
  Swiss Ephemeris drops nutation from sidereal positions.
- **(b) A sidereal mean chart mixes frames.** A mean chart's longitude is the
  backend's J2000 place, and the of-date ayanamsa is subtracted from it:
  measured [four values with their epochs] against the mean-of-date sidereal
  longitude, which is the precession accumulated since J2000.

Either fix changes chart output and needs its own decision, a regenerated
chart golden and a gate against a Swiss Ephemeris sidereal position corpus.

**Severity:** (a) convention, up to about 17″; (b) frame correctness, growing
with distance from J2000
```

In `docs/superpowers/specs/2026-10-01-crossings-mean-sidereal-design.md`, change the status line to `**Status:** implemented (2026-10-01) ·` (use the actual completion date). If Task 3's row count or ayanamsa picks differ from the spec's "about 70 rows", add a one-line amendment at the end of the spec stating the final count (169) and the four ayanamsas.

- [ ] **Step 6: Full verification**

Run: `mise run ci`
Expected: PASS. Run it in the foreground; make no edits and no commits until it finishes.

Run: `cargo test -p pleiades-events --features serde`
Expected: PASS.

Run: `mise run package-check`
Expected: PASS — `pleiades-events` packages with its new `pleiades-ayanamsa` dependency.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all --check
git add README.md crates docs
git commit -m "docs: mean-of-date and sidereal crossings; FU-18 chart sidereal conventions (#88)"
```
