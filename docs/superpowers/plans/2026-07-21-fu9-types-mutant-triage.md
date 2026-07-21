# FU-9 Final Slice — `pleiades-types` + `provenance.rs` Mutant Triage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the 44 remaining FU-9 surviving mutants (41 in `pleiades-types`, 3 in `pleiades-apparent/src/provenance.rs`) to zero with intent-expressing white-box tests, closing the FU-9 mutation-testing baseline.

**Architecture:** Tests-only slice. First a mechanical, behavior-preserving relocation of `pleiades-types`'s monolithic `src/tests.rs` into a per-module `src/tests/` directory (AGENTS.md "split before adding"). Then new white-box tests, grouped by survivor shape, added into the relocated submodules. No production code changes; the mutants tier stays report-only.

**Tech Stack:** Rust (stable), `cargo-nextest`, `cargo-mutants` 27.x, `mise` task runner.

**Spec:** `docs/superpowers/specs/2026-07-21-fu9-types-mutant-triage-design.md`

## Global Constraints

- **Tests-only:** no change to any production `.rs` in the 11 target files other than moving each file's inline `#[cfg(test)] mod tests` out to a co-located test file. Copied verbatim from spec §2.
- **Report-only tier:** no `validate-*` gate file is touched; no mutation score gates anything.
- **No suppression:** no `#[mutants::skip]` attribute is added anywhere.
- **Independent references:** every expected value is a literal stated in the test, derived from a published formula / documented contract / crafted input — never read from the code's own output.
- **Reference independence for the one numeric file:** the `mean_obliquity` expected values come from the published IAU-1976 arcsec coefficients evaluated outside the code (see Task 3 mirror), confirmed to match the crate at `< 1e-12` before pinning.
- **Per-file acceptance command (pleiades-types):** `cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false --file crates/pleiades-types/src/<file>.rs` must report `0 missed`.
- **Per-file acceptance command (provenance):** `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file crates/pleiades-apparent/src/provenance.rs` must report `0 missed`.
- **Green CI:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `cargo test -p pleiades-types -p pleiades-apparent` all pass.
- **Branch:** `fu9-types-mutant-triage` (already created; design doc already committed on it).

---

## Reference: exact literals used across tasks

These are computed/derived once here so every task can reference them.

**Independent `mean_obliquity` mirror (Task 3)** — Python, from published IAU-1976 coefficients:

```python
eps0 = 23 + 26/60 + 21.448/3600      # 23.43929111111111
c1 = -46.8150/3600                    # per Julian century
c2 = -0.00059/3600
c3 =  0.001813/3600
def eps(t): return eps0 + c1*t + c2*t*t + c3*t*t*t
# eps(+1.0) = 23.42628728416667   (t = +1 -> JD 2488070.0, year 2100)
# eps(-1.0) = 23.452294610277775  (t = -1 -> JD 2415020.0, year 1900)
# eps( 0.0) = 23.43929111111111   (already asserted at t = 0)
```

Both off-epoch values reproduce the crate's own degree-literal evaluation exactly (verified during planning); they are pinned with a `1e-12` absolute tolerance.

**Exact rendered strings** (read from the target `format!`/`match` sources, stated as literals):

| Item | Expected string |
| --- | --- |
| `ZodiacSign::{Aries..Pisces}.name()` | `"Aries"`,`"Taurus"`,`"Gemini"`,`"Cancer"`,`"Leo"`,`"Virgo"`,`"Libra"`,`"Scorpio"`,`"Sagittarius"`,`"Capricorn"`,`"Aquarius"`,`"Pisces"` |
| `MotionDirection` | `"Direct"`, `"Stationary"`, `"Retrograde"` |
| `MotionValidationError::NonFiniteSpeed{field:"longitude_deg_per_day", value: NaN}` | `` "motion field `longitude_deg_per_day` must be finite, got NaN" `` |
| `Apparentness` | `"Apparent"`, `"Mean"` |
| `CoordinateValidationError::LatitudeOutOfRange{coordinate:"ecliptic",field:"latitude",value:91.0}` | `` "ecliptic coordinate field `latitude` must stay within [-90, 90], got 91" `` |
| `ObserverLocationValidationError::LatitudeOutOfRange{value:91.0}` | `"observer latitude must stay within [-90, 90], got 91"` |
| `TopocentricProvenance{12.5,-3.25,0.3192,1.5}.summary_line()` | `` `topocentric parallax_lon=12.500" parallax_lat=-3.250" diurnal_aberration=0.3192" distance_au=1.500000` `` |

**Zodiac band mid-points** (each `30·k + 15`): 15→Aries, 45→Taurus, 75→Gemini, 105→Cancer, 135→Leo, 165→Virgo, 195→Libra, 225→Scorpio, 255→Sagittarius, 285→Capricorn, 315→Aquarius, 345→Pisces.

---

## Task 1: Relocate `pleiades-types` tests into a per-module `src/tests/` directory

**Files:**
- Create: `crates/pleiades-types/src/tests/mod.rs`
- Create: `crates/pleiades-types/src/tests/{angles,ayanamsa,bodies,coordinates,custom_bodies,frames,house_systems,motion,observer,time,time_range,zodiac}.rs`
- Delete: `crates/pleiades-types/src/tests.rs`
- Unchanged: `crates/pleiades-types/src/lib.rs` (`#[cfg(test)] mod tests;` now resolves to `tests/mod.rs`)

**Interfaces:**
- Consumes: nothing (mechanical move).
- Produces: `crate::tests::{angles,ayanamsa,…,zodiac}` submodules; each subsequent task adds its new test(s) into one of these files.

**This is a pure move.** No test body is edited; the goal is byte-identical test behavior with the code split by subject. Each moved test keeps its exact body; only its containing module and `use` header change.

- [ ] **Step 1: Create `tests/mod.rs` with the submodule declarations**

Create `crates/pleiades-types/src/tests/mod.rs`:

```rust
//! Unit tests for the `pleiades-types` crate, split by the source module
//! under test. Relocated from the former monolithic `src/tests.rs` per
//! AGENTS.md ("split a large file before adding to it").

mod angles;
mod ayanamsa;
mod bodies;
mod coordinates;
mod custom_bodies;
mod frames;
mod house_systems;
mod motion;
mod observer;
mod time;
mod time_range;
mod zodiac;
```

- [ ] **Step 2: Move each existing test into its subject file**

For each existing test function in the old `src/tests.rs`, move its **verbatim body** into the target file below. Each target file begins with `use crate::*;` (the crate root re-exports the full public vocabulary), plus the local imports that test needs (`use core::time::Duration;` for the time-scale conversion tests; `use proptest::prelude::*;` for the angle proptests). Mapping (test name → target file):

| Target file | Tests to move |
| --- | --- |
| `angles.rs` | `angle_normalization_wraps_correctly`, `longitude_is_always_normalized`, and the `mod angle_properties { … }` proptest module (verbatim) |
| `bodies.rs` | `celestial_body_classes_cover_the_built_in_catalog`, `built_in_body_names_are_stable`, `celestial_body_validate_reuses_custom_body_checks` |
| `custom_bodies.rs` | `custom_body_id_validate_rejects_blank_padding_and_separators` |
| `coordinates.rs` | `ecliptic_to_equatorial_preserves_zero_obliquity_identity`, `ecliptic_to_equatorial_rotates_by_mean_obliquity`, `equatorial_to_ecliptic_round_trip_uses_the_same_obliquity`, `mean_obliquity_round_trip_stays_stable_across_quadrants`, `mean_obliquity_round_trip_stays_stable_near_the_poles`, `ecliptic_to_equatorial_normalizes_negative_right_ascension`, `equatorial_to_ecliptic_round_trip_preserves_negative_declination_near_wraparound`, `equatorial_to_ecliptic_treats_negative_right_ascension_as_normalized_angle`, `equatorial_to_ecliptic_treats_full_turn_right_ascension_as_normalized_angle`, `coordinate_validation_rejects_non_finite_and_out_of_range_values` |
| `time.rs` | `instant_mean_obliquity_matches_the_shared_cubic_approximation`, `instant_has_a_compact_display`, `time_scales_have_stable_display_names`, `time_scale_conversion_errors_use_stable_display_labels`, all `caller_supplied_time_scale_offsets_*` (13 tests), `time_scale_helpers_reject_the_wrong_source_scale`, `signed_time_scale_helpers_reject_non_finite_offsets`, `time_scale_conversion_errors_render_stable_summary_lines`, all `time_scale_conversion_policy_*` (5 tests), `instant_validate_time_scale_conversion_rejects_mismatched_scales_and_non_finite_offsets`, `instant_with_time_scale_offset_checked_rejects_non_finite_offsets` |
| `time_range.rs` | `time_range_checks_scale_and_julian_day`, `time_range_validation_rejects_non_finite_bounds_and_invalid_order` |
| `zodiac.rs` | `zodiac_modes_have_stable_display_names`, `zodiac_signs_follow_longitude_bands` |
| `ayanamsa.rs` | `custom_ayanamsas_have_stable_display_names`, `custom_ayanamsa_enum_validate_reuses_the_structured_validator`, `custom_ayanamsa_validate_rejects_padded_or_incomplete_definitions`, `custom_ayanamsa_validate_against_reserved_labels_rejects_builtin_collisions` |
| `house_systems.rs` | `custom_house_system_display_includes_aliases_and_notes`, `custom_house_system_validate_rejects_whitespace_and_duplicate_aliases`, `custom_house_system_validate_against_reserved_labels_rejects_builtin_collisions`, `house_systems_have_stable_display_names`, `house_system_validate_reuses_the_structured_validator` |
| `frames.rs` | `coordinate_frames_have_stable_display_names` |
| `motion.rs` | `motion_accessors_return_the_original_speed_components`, `motion_summary_line_matches_display`, `motion_direction_tracks_the_sign_of_longitude_speed`, `motion_validation_rejects_non_finite_components` |
| `observer.rs` | `observer_location_has_a_compact_display`, `observer_location_validation_rejects_invalid_values` |

Then delete the old `crates/pleiades-types/src/tests.rs`.

> Note on imports: the old file used `use super::*;` (crate-root glob). In the new submodules `use crate::*;` is the equivalent. If any moved test references a name the glob does not resolve (e.g. a `pub(crate)` helper), add the explicit `use crate::<module>::<item>;` — but the existing 69 tests all exercise the public API, so `use crate::*;` plus the two local imports above should suffice.

- [ ] **Step 3: Verify the move is behavior-neutral**

Run: `cargo test -p pleiades-types 2>&1 | tail -20`
Expected: PASS with the **same test count as before** (69 unit tests plus the proptests). Compare against the pre-move count captured with `git stash && cargo test -p pleiades-types 2>&1 | grep 'test result' ; git stash pop` if in doubt.

- [ ] **Step 4: Format + lint**

Run: `cargo fmt -p pleiades-types && cargo clippy -p pleiades-types --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-types/src/tests.rs crates/pleiades-types/src/tests/
git commit -m "test(types): relocate tests.rs into per-module src/tests/"
```

---

## Task 2: `zodiac.rs` triage (12 → 0)

**Files:**
- Test: `crates/pleiades-types/src/tests/zodiac.rs`

**Interfaces:**
- Consumes: `ZodiacSign` (public), `Longitude::from_degrees` (public).
- Produces: nothing downstream.

Kills: `Display for ZodiacSign` (103), `name → ""/"xyzzy"` (84 ×2), and the 9 deleted match arms (69–77) in `from_longitude`.

- [ ] **Step 1: Add the two failing-safe tests**

Append to `crates/pleiades-types/src/tests/zodiac.rs`:

```rust
#[test]
fn zodiac_sign_names_and_display_are_stable() {
    let cases = [
        (ZodiacSign::Aries, "Aries"),
        (ZodiacSign::Taurus, "Taurus"),
        (ZodiacSign::Gemini, "Gemini"),
        (ZodiacSign::Cancer, "Cancer"),
        (ZodiacSign::Leo, "Leo"),
        (ZodiacSign::Virgo, "Virgo"),
        (ZodiacSign::Libra, "Libra"),
        (ZodiacSign::Scorpio, "Scorpio"),
        (ZodiacSign::Sagittarius, "Sagittarius"),
        (ZodiacSign::Capricorn, "Capricorn"),
        (ZodiacSign::Aquarius, "Aquarius"),
        (ZodiacSign::Pisces, "Pisces"),
    ];
    for (sign, expected) in cases {
        assert_eq!(sign.name(), expected);
        assert_eq!(sign.to_string(), expected);
    }
}

#[test]
fn zodiac_signs_cover_every_thirty_degree_band() {
    let bands = [
        (15.0, ZodiacSign::Aries),
        (45.0, ZodiacSign::Taurus),
        (75.0, ZodiacSign::Gemini),
        (105.0, ZodiacSign::Cancer),
        (135.0, ZodiacSign::Leo),
        (165.0, ZodiacSign::Virgo),
        (195.0, ZodiacSign::Libra),
        (225.0, ZodiacSign::Scorpio),
        (255.0, ZodiacSign::Sagittarius),
        (285.0, ZodiacSign::Capricorn),
        (315.0, ZodiacSign::Aquarius),
        (345.0, ZodiacSign::Pisces),
    ];
    for (deg, expected) in bands {
        assert_eq!(
            ZodiacSign::from_longitude(Longitude::from_degrees(deg)),
            expected,
            "longitude {deg} should map to {expected:?}"
        );
    }
    // Wraparound: normalization must reduce a full turn before band dispatch.
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(360.0)),
        ZodiacSign::Aries
    );
    assert_eq!(
        ZodiacSign::from_longitude(Longitude::from_degrees(750.0)),
        ZodiacSign::Gemini
    );
}
```

- [ ] **Step 2: Run the tests — expect PASS (they characterize existing correct behavior)**

Run: `cargo test -p pleiades-types zodiac 2>&1 | tail -15`
Expected: PASS.

- [ ] **Step 3: Confirm the survivors are killed**

Run: `cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false --file crates/pleiades-types/src/zodiac.rs 2>&1 | tail -3`
Expected: `... 0 missed ...` (was 12 missed).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-types/src/tests/zodiac.rs
git commit -m "test(types): FU-9 zodiac.rs mutant triage (12 -> 0)"
```

---

## Task 3: `time.rs` `mean_obliquity` triage (10 → 0)

**Files:**
- Test: `crates/pleiades-types/src/tests/time.rs`

**Interfaces:**
- Consumes: `Instant::new`, `JulianDay::from_days`, `TimeScale`, `Instant::mean_obliquity` (all public).
- Produces: nothing downstream.

Kills all 10 arithmetic-operator swaps in the mean-obliquity cubic (349–354). The existing t = 0 test cannot see them; two off-epochs at t = ±1 with pinned exact values do.

- [ ] **Step 1: (Independence) confirm the mirror values against the crate**

Run this one-off check (not committed) to confirm the pinned literals equal the crate output before hard-coding them:

```bash
python3 -c "
eps0=23+26/60+21.448/3600; c1=-46.8150/3600; c2=-0.00059/3600; c3=0.001813/3600
f=lambda t: eps0+c1*t+c2*t*t+c3*t*t*t
print(repr(f(1.0)), repr(f(-1.0)))
"
```
Expected: `23.42628728416667 23.452294610277775`.

- [ ] **Step 2: Add the failing-safe test**

Append to `crates/pleiades-types/src/tests/time.rs`:

```rust
#[test]
fn mean_obliquity_matches_published_cubic_off_epoch() {
    // t = (jd - 2451545.0) / 36525.0. These two epochs make jd - 2451545
    // exactly +/-36525.0, so t is exactly +/-1.0 (JD-grid representable).
    // Expected values come from the published IAU-1976 mean-obliquity cubic
    // evaluated outside the code (23d26m21.448s - 46.8150"t - 0.00059"t^2
    // + 0.001813"t^3), not from mean_obliquity itself.
    let plus_one = Instant::new(JulianDay::from_days(2_488_070.0), TimeScale::Tt);
    assert!(
        (plus_one.mean_obliquity().degrees() - 23.426_287_284_166_67).abs() < 1e-12,
        "t=+1 obliquity was {}",
        plus_one.mean_obliquity().degrees()
    );

    let minus_one = Instant::new(JulianDay::from_days(2_415_020.0), TimeScale::Tt);
    assert!(
        (minus_one.mean_obliquity().degrees() - 23.452_294_610_277_775).abs() < 1e-12,
        "t=-1 obliquity was {}",
        minus_one.mean_obliquity().degrees()
    );
}
```

- [ ] **Step 3: Run the test — expect PASS**

Run: `cargo test -p pleiades-types mean_obliquity_matches_published_cubic_off_epoch 2>&1 | tail -10`
Expected: PASS. (If it fails, the pinned literal is wrong — re-check Step 1; do NOT copy the code's output into the test.)

- [ ] **Step 4: Confirm the survivors are killed**

Run: `cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false --file crates/pleiades-types/src/time.rs 2>&1 | tail -3`
Expected: `0 missed` (was 10 missed).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-types/src/tests/time.rs
git commit -m "test(types): FU-9 time.rs mean_obliquity mutant triage (10 -> 0)"
```

---

## Task 4: Guard + conversion triage — `time_range`, `coordinates`, `ayanamsa`, `house_systems`, `angles` (15 → 0)

**Files:**
- Test: `crates/pleiades-types/src/tests/time_range.rs`, `coordinates.rs`, `ayanamsa.rs`, `house_systems.rs`, `angles.rs`

**Interfaces:**
- Consumes: `TimeRange`, `Instant`, `JulianDay`, `TimeScale`, `EclipticCoordinates`, `Longitude`, `Latitude`, `CoordinateValidationError`, `CustomAyanamsa`, `Ayanamsa`, `CustomHouseSystem`, `HouseSystem`, `Angle` (all public).
- Produces: nothing downstream.

Kills: `time_range` `&&`→`||` ×3 (32/35/37) + `>`→`>=` (61); `coordinates` `<`→`<=` (266) + `validate_finite_coordinate_value`→`Ok(())` (216) + `Display`→`Ok(default)` (205); `ayanamsa` delete-`!` (222) + reserved-label dispatch (167/168); `house_systems` reserved-label dispatch (121/122); `angles` `is_finite`→const ×2 (52) + `From<Angle> for Latitude`→default (125).

- [ ] **Step 1: `time_range.rs` — boundary test**

Append to `crates/pleiades-types/src/tests/time_range.rs`:

```rust
#[test]
fn time_range_contains_and_validate_respect_boundaries() {
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tt);
    let range = TimeRange::new(Some(start), Some(end));

    // Same scale, but outside each bound: the && form excludes these; the ||
    // mutant would wrongly include a value that fails exactly one clause.
    let before = Instant::new(JulianDay::from_days(2_451_544.5), TimeScale::Tt);
    let after = Instant::new(JulianDay::from_days(2_451_546.5), TimeScale::Tt);
    assert!(!range.contains(before), "instant before start must not be contained");
    assert!(!range.contains(after), "instant after end must not be contained");
    // A same-scale in-range instant IS contained (guards the 37 && overall).
    assert!(range.contains(Instant::new(JulianDay::from_days(2_451_545.5), TimeScale::Tt)));

    // Degenerate range start == end must validate Ok; the >= mutant flags it
    // as out-of-order.
    let point = TimeRange::new(Some(start), Some(start));
    assert_eq!(point.validate(), Ok(()));
}
```

- [ ] **Step 2: `coordinates.rs` — zero-distance + non-finite-angle + Display tests**

Append to `crates/pleiades-types/src/tests/coordinates.rs`:

```rust
#[test]
fn coordinate_validation_accepts_zero_distance_and_rejects_non_finite_angle() {
    // distance exactly 0.0 is valid; the `<` -> `<=` mutant would reject it.
    let zero_distance = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(12.0),
        Some(0.0),
    );
    assert_eq!(zero_distance.validate(), Ok(()));

    // A non-finite longitude survives Longitude normalization (NaN.rem_euclid
    // = NaN) and must be rejected by validate_finite_coordinate_value; the
    // `-> Ok(())` mutant would skip that check.
    let nan_longitude = EclipticCoordinates::new(
        Longitude::from_degrees(f64::NAN),
        Latitude::from_degrees(12.0),
        None,
    );
    assert!(matches!(
        nan_longitude.validate(),
        Err(CoordinateValidationError::NonFiniteValue {
            coordinate: "ecliptic",
            field: "longitude",
            value,
        }) if value.is_nan()
    ));
}

#[test]
fn coordinate_validation_error_display_matches_summary_line() {
    let err = CoordinateValidationError::LatitudeOutOfRange {
        coordinate: "ecliptic",
        field: "latitude",
        value: 91.0,
    };
    // Ties the Display wrapper (205) to the already-pinned summary_line.
    assert_eq!(err.to_string(), err.summary_line());
    assert_eq!(
        err.to_string(),
        "ecliptic coordinate field `latitude` must stay within [-90, 90], got 91"
    );
}
```

- [ ] **Step 3: `ayanamsa.rs` — finite-offset guard + enum reserved-label dispatch**

Append to `crates/pleiades-types/src/tests/ayanamsa.rs`:

```rust
#[test]
fn custom_ayanamsa_accepts_finite_offset_pair_and_rejects_non_finite() {
    // Both epoch and a FINITE offset present -> Ok. The deleted `!` in
    // `if !offset.is_finite()` would reject this finite pair.
    let finite = CustomAyanamsa {
        name: "Local Calibration".to_string(),
        description: None,
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: Some(Angle::from_degrees(24.0)),
    };
    assert_eq!(finite.validate(), Ok(()));

    // Both present but offset non-finite -> Err. The deleted `!` would accept it.
    let non_finite = CustomAyanamsa {
        name: "Local Calibration".to_string(),
        description: None,
        epoch: Some(JulianDay::from_days(2_451_545.0)),
        offset_degrees: Some(Angle::from_degrees(f64::INFINITY)),
    };
    assert!(non_finite.validate().is_err());
}

#[test]
fn ayanamsa_enum_validate_against_reserved_labels_checks_wrapped_custom() {
    // The enum method (and its Self::Custom arm) must forward to the wrapped
    // custom's reserved-label check. Existing tests call the struct method
    // directly and never traverse this arm.
    let wrapped = Ayanamsa::Custom(CustomAyanamsa::new("Lahiri"));
    assert!(
        wrapped
            .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Lahiri"))
            .is_err()
    );
    // Built-in variants are always Ok (documents the `_ => Ok(())` arm).
    assert_eq!(
        Ayanamsa::Lahiri.validate_against_reserved_labels(|_| true),
        Ok(())
    );
}
```

- [ ] **Step 4: `house_systems.rs` — enum reserved-label dispatch**

Append to `crates/pleiades-types/src/tests/house_systems.rs`:

```rust
#[test]
fn house_system_enum_validate_against_reserved_labels_checks_wrapped_custom() {
    let wrapped = HouseSystem::Custom(CustomHouseSystem::new("Equal"));
    assert!(
        wrapped
            .validate_against_reserved_labels(|label| label.eq_ignore_ascii_case("Equal"))
            .is_err()
    );
    assert_eq!(
        HouseSystem::Placidus.validate_against_reserved_labels(|_| true),
        Ok(())
    );
}
```

- [ ] **Step 5: `angles.rs` — is_finite + Latitude conversion**

Append to `crates/pleiades-types/src/tests/angles.rs`:

```rust
#[test]
fn angle_is_finite_and_latitude_from_angle_preserve_values() {
    // is_finite must track the value, not a constant (kills -> true / -> false).
    assert!(Angle::from_degrees(42.0).is_finite());
    assert!(!Angle::from_degrees(f64::NAN).is_finite());
    assert!(!Angle::from_degrees(f64::INFINITY).is_finite());

    // From<Angle> for Latitude must carry the value, not Default (0.0).
    let lat = Latitude::from(Angle::from_degrees(-33.5));
    assert!((lat.degrees() - (-33.5)).abs() < 1e-12);
}
```

- [ ] **Step 6: Run all five files' tests — expect PASS**

Run: `cargo test -p pleiades-types 2>&1 | tail -15`
Expected: PASS (all new + existing tests green).

- [ ] **Step 7: Confirm the survivors are killed in each file**

Run each and expect `0 missed`:

```bash
for f in time_range coordinates ayanamsa house_systems angles; do
  echo "== $f =="
  cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false \
    --file crates/pleiades-types/src/$f.rs 2>&1 | tail -2
done
```
Expected: each reports `0 missed` (was time_range 4, coordinates 3, ayanamsa 3, house_systems 2, angles 3).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-types/src/tests/{time_range,coordinates,ayanamsa,house_systems,angles}.rs
git commit -m "test(types): FU-9 guard + conversion mutant triage (time_range/coordinates/ayanamsa/house_systems/angles, 15 -> 0)"
```

---

## Task 5: `Display` triage — `motion`, `observer`, `frames` (4 → 0)

**Files:**
- Test: `crates/pleiades-types/src/tests/motion.rs`, `observer.rs`, `frames.rs`

**Interfaces:**
- Consumes: `MotionDirection`, `MotionValidationError`, `ObserverLocationValidationError`, `Apparentness` (all public).
- Produces: nothing downstream.

Kills the `Display → Ok(default)` mutants at `motion.rs:19`, `motion.rs:54`, `observer.rs:130`, `frames.rs:39`.

- [ ] **Step 1: `motion.rs` test**

Append to `crates/pleiades-types/src/tests/motion.rs`:

```rust
#[test]
fn motion_direction_and_error_render_stable_strings() {
    assert_eq!(MotionDirection::Direct.to_string(), "Direct");
    assert_eq!(MotionDirection::Stationary.to_string(), "Stationary");
    assert_eq!(MotionDirection::Retrograde.to_string(), "Retrograde");

    let err = MotionValidationError::NonFiniteSpeed {
        field: "longitude_deg_per_day",
        value: f64::NAN,
    };
    assert_eq!(
        err.to_string(),
        "motion field `longitude_deg_per_day` must be finite, got NaN"
    );
}
```

- [ ] **Step 2: `observer.rs` test**

Append to `crates/pleiades-types/src/tests/observer.rs`:

```rust
#[test]
fn observer_location_validation_errors_render_stable_strings() {
    // The existing test asserts summary_line; this asserts the Display wrapper
    // (observer.rs:130), which the -> Ok(default) mutant empties.
    assert_eq!(
        ObserverLocationValidationError::LatitudeOutOfRange { value: 91.0 }.to_string(),
        "observer latitude must stay within [-90, 90], got 91"
    );
    assert_eq!(
        ObserverLocationValidationError::NonFiniteElevation {
            value: f64::INFINITY
        }
        .to_string(),
        "observer elevation must be finite, got inf"
    );
}
```

- [ ] **Step 3: `frames.rs` test**

Append to `crates/pleiades-types/src/tests/frames.rs`:

```rust
#[test]
fn apparentness_displays_stable_labels() {
    assert_eq!(Apparentness::Apparent.to_string(), "Apparent");
    assert_eq!(Apparentness::Mean.to_string(), "Mean");
}
```

- [ ] **Step 4: Run the tests — expect PASS**

Run: `cargo test -p pleiades-types 2>&1 | tail -10`
Expected: PASS.

- [ ] **Step 5: Confirm the survivors are killed**

```bash
for f in motion observer frames; do
  echo "== $f =="
  cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false \
    --file crates/pleiades-types/src/$f.rs 2>&1 | tail -2
done
```
Expected: each reports `0 missed` (was motion 2, observer 1, frames 1).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-types/src/tests/{motion,observer,frames}.rs
git commit -m "test(types): FU-9 Display mutant triage (motion/observer/frames, 4 -> 0)"
```

---

## Task 6: `provenance.rs` triage (3 → 0)

**Files:**
- Modify: `crates/pleiades-apparent/src/provenance.rs` (relocate its inline test module)
- Create: `crates/pleiades-apparent/src/provenance/tests.rs`

**Interfaces:**
- Consumes: `TopocentricProvenance` (public), plus the existing `ApparentProvenance`/`CorrectionSet`/`MODEL_SOURCES` used by the relocated test.
- Produces: nothing downstream.

Kills `TopocentricProvenance::summary_line → String::new()/"xyzzy"` (96 ×2) and `Display for TopocentricProvenance → Ok(default)` (108).

- [ ] **Step 1: Relocate the inline test module (pure move)**

In `crates/pleiades-apparent/src/provenance.rs`, replace the inline
`#[cfg(test)] mod tests { … }` block (from `#[cfg(test)]` to its closing brace)
with a single line:

```rust
#[cfg(test)]
mod tests;
```

Create `crates/pleiades-apparent/src/provenance/tests.rs` containing `use super::*;` followed by the moved body of the former inline `mod tests` (the existing `summary_line_is_nonempty_and_matches_display` test and any siblings), verbatim.

- [ ] **Step 2: Add the `TopocentricProvenance` string test**

Append to `crates/pleiades-apparent/src/provenance/tests.rs`:

```rust
#[test]
fn topocentric_provenance_summary_line_and_display_are_stable() {
    let p = TopocentricProvenance {
        parallax_longitude_arcsec: 12.5,
        parallax_latitude_arcsec: -3.25,
        diurnal_aberration_arcsec: 0.3192,
        distance_au_used: 1.5,
    };
    assert_eq!(
        p.summary_line(),
        "topocentric parallax_lon=12.500\" parallax_lat=-3.250\" diurnal_aberration=0.3192\" distance_au=1.500000"
    );
    assert_eq!(p.to_string(), p.summary_line());
}
```

- [ ] **Step 3: Run the tests — expect PASS**

Run: `cargo test -p pleiades-apparent provenance 2>&1 | tail -10`
Expected: PASS. (If the `summary_line` literal mismatches, re-derive it from the `format!` in `provenance.rs:96` — do not paste the code's output.)

- [ ] **Step 4: Confirm the survivors are killed**

Run: `cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file crates/pleiades-apparent/src/provenance.rs 2>&1 | tail -3`
Expected: `0 missed` (was 3 missed).

- [ ] **Step 5: Format, lint, commit**

```bash
cargo fmt -p pleiades-apparent && cargo clippy -p pleiades-apparent --all-targets -- -D warnings
git add crates/pleiades-apparent/src/provenance.rs crates/pleiades-apparent/src/provenance/tests.rs
git commit -m "test(apparent): FU-9 provenance.rs mutant triage (3 -> 0)"
```

---

## Task 7: Record FU-9 closure + final verification

**Files:**
- Modify: `docs/follow-ups.md` (FU-9 section)

**Interfaces:**
- Consumes: the confirmed per-file `0 missed` results from Tasks 2–6.
- Produces: the FU-9 closure record.

- [ ] **Step 1: Full-crate mutant re-check (both crates)**

```bash
cargo mutants -p pleiades-types --test-tool nextest --test-workspace=false 2>&1 | tail -2
cargo mutants -p pleiades-apparent --test-tool nextest --test-workspace=false --file crates/pleiades-apparent/src/provenance.rs 2>&1 | tail -2
```
Expected: `pleiades-types` reports `0 missed`; `provenance.rs` reports `0 missed`.

- [ ] **Step 2: Green blocking CI**

Run: `mise run ci 2>&1 | tail -20`
Expected: PASS (fmt, clippy `-D warnings`, tests).

- [ ] **Step 3: Append the FU-9 progress/closure note**

Add a dated progress entry to the FU-9 section of `docs/follow-ups.md`, following the format of the prior slice entries. It must state:
- `pleiades-types` triaged from 41 → 0 across 10 files, and `provenance.rs` 3 → 0 (folded in — it was recorded in the baseline notes but omitted from every prior remaining-slices line);
- this slice **closes the FU-9 baseline** — every file in the 2026-07-18 three-crate measurement now reaches 0 surviving mutants or a documented equivalent;
- the total documented-equivalent tally across all nine slices (nutation 1, refraction 3, topocentric 3, precession 2 = 9 equivalents; apparent/aberration/sidereal/time/types all genuine 0);
- tests-only; the test split of `tests.rs` into `src/tests/`; no parity gate touched; tier stays report-only; `mise run ci` green;
- **Remaining slices: none** for the original three-crate baseline; FU-9 stays open only as a standing posture entry for any future `mise run mutants` expansion to unmeasured crates.

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): FU-9 pleiades-types + provenance triage — baseline complete"
```

- [ ] **Step 5: Open the PR**

```bash
git push -u origin fu9-types-mutant-triage
gh pr create --fill --title "test: FU-9 final slice — pleiades-types + provenance mutant triage (44 -> 0)"
```

---

## Self-Review notes

- **Spec coverage:** every §4 shape maps to a task — Shape 1 (Display) → Tasks 2/4/5/6; Shape 2 (zodiac arms) → Task 2; Shape 3 (mean_obliquity) → Task 3; Shape 4 (guards/dispatch) → Task 4; Shape 5 (accessor/conversion) → Task 4. §7 relocations → Tasks 1 and 6. §9 deliverables map 1:1 to Tasks 1–7 commits.
- **Count check:** kills per task = 12 (T2) + 10 (T3) + 15 (T4) + 4 (T5) + 3 (T6) = 44. Matches spec.
- **No placeholders:** every test body is complete; every command has an expected result; the one numeric literal set is computed in the plan header and re-confirmed in Task 3 Step 1.
- **Type consistency:** `validate_against_reserved_labels` closure signature (`impl Fn(&str) -> bool`), `CustomAyanamsa`/`CustomHouseSystem::new`, `EclipticCoordinates::new(Longitude, Latitude, Option<f64>)`, and `CoordinateValidationError::NonFiniteValue { coordinate, field, value }` all match the source signatures inspected during design.
