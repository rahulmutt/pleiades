use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, JulianDay, Latitude, TimeScale};

#[test]
fn apparent_midheaven_declination_pins_the_product_form() {
    // Independent reference (houses-reference.py `mid_dec`): atan(sin(st)*tan(eps)).
    // st=57, eps=23.4392811 chosen so sin*tan != sin+tan != sin/tan, killing the
    // 1651:43 `* -> +` and `* -> /` survivors. Literals emitted by the reference.
    assert_close_degrees(
        apparent_midheaven_declination(57.0, 23.4392811),
        19.98167206592639,
    );
    assert_close_degrees(
        apparent_midheaven_declination(205.0, 23.4392811),
        -10.382982940241511,
    );
}

#[test]
fn apparent_solar_declination_pins_the_published_sun_series() {
    // Independent reference (houses-reference.py `solar_declination`), the
    // published NOAA/USNO low-precision Sun. Two non-degenerate instants
    // (d=55 and d=3455 days from J2000) make every `*d` series term and both
    // sin(g)/sin(2g) terms observable, killing all 20 arith survivors.
    let obl = Angle::from_degrees(23.4392811);
    let dec = |jd: f64| {
        apparent_solar_declination(Instant::new(JulianDay::from_days(jd), TimeScale::Tt), obl)
            .degrees()
    };
    assert_close_degrees(dec(2_451_600.0), -9.23948196138383);
    assert_close_degrees(dec(2_455_000.0), 23.39101729022372);
}

#[test]
fn sunshine_offsets_pins_the_semi_arc_trisection() {
    // Independent reference (houses-reference.py `sunshine_offsets`). Two
    // geometries with ad != 0 (so nsa != dsa) make every +/-, 2*, and /3 term
    // observable, killing all 34 arith survivors. Only the 8 non-zero house
    // indices carry signal; the other 5 are hard 0.0 and untested here.
    let check = |lat: f64, sundec: f64, want: [(usize, f64); 8]| {
        let got = sunshine_offsets(lat, sundec);
        for (idx, expected) in want {
            assert_close_degrees(got[idx], expected);
        }
    };
    check(
        52.0,
        -10.0,
        [
            (2, -68.69556843044369),
            (3, -34.34778421522184),
            (5, 34.34778421522184),
            (6, 68.69556843044369),
            (8, -51.30443156955631),
            (9, -25.652215784778154),
            (11, 25.652215784778154),
            (12, 51.30443156955631),
        ],
    );
    check(
        -33.0,
        15.0,
        [
            (2, -66.68063265912221),
            (3, -33.340316329561105),
            (5, 33.340316329561105),
            (6, 66.68063265912221),
            (8, -53.31936734087779),
            (9, -26.659683670438895),
            (11, 26.659683670438895),
            (12, 53.31936734087779),
        ],
    );
}

/// Independent recomposition of `sunshine_houses` (published Sunshine/solar-arc
/// algorithm), threading `st`/`sundec`/`offsets` from the un-mutated (and
/// independently-pinned) helpers. Separate transcription => a mutant inside
/// `sunshine_houses` is not mirrored here, so equality kills it. Returns the 12
/// cusp longitudes in degrees.
fn recompose_sunshine(
    instant: Instant,
    observer: &ObserverLocation,
    obliquity: Angle,
    angles: HouseAngles,
) -> [f64; 12] {
    let sidereal_time = local_sidereal_time(instant, observer.longitude).degrees();
    let latitude = observer.latitude.degrees();
    let obliquity_deg = obliquity.degrees();
    let sundec = apparent_solar_declination(instant, obliquity).degrees();
    let mc_under_horizon = latitude.signum() != 0.0
        && (latitude - apparent_midheaven_declination(sidereal_time, obliquity_deg)).abs() > 90.0;

    let mut cusps = [0.0_f64; 12];
    let mut ascendant = angles.ascendant;
    let mut midheaven = angles.midheaven;
    let acmc = signed_longitude_difference(ascendant.degrees(), midheaven.degrees());
    if acmc < 0.0 {
        ascendant = longitude_opposite(ascendant);
        midheaven = longitude_opposite(midheaven); // KEEP_MC_SOUTH is const false
    }
    cusps[0] = ascendant.degrees();
    cusps[3] = longitude_opposite(midheaven).degrees();
    cusps[6] = longitude_opposite(ascendant).degrees();
    cusps[9] = midheaven.degrees();

    let offsets = sunshine_offsets(latitude, sundec);
    let sin_ecl = obliquity_deg.to_radians().sin();
    let cos_ecl = obliquity_deg.to_radians().cos();

    for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
        let offset = offsets[house];
        let xhs = 2.0
            * (sundec.to_radians().cos() * (offset.to_radians() / 2.0).sin())
                .asin()
                .to_degrees();
        let cosa = (sundec.to_radians().tan() * (xhs.to_radians() / 2.0).tan()).clamp(-1.0, 1.0);
        let alph = cosa.acos().to_degrees();
        let (alpha2, b) = if house > 7 {
            (180.0 - alph, 90.0 - latitude + sundec)
        } else {
            (alph, 90.0 - latitude - sundec)
        };
        let cosc = xhs.to_radians().cos() * b.to_radians().cos()
            + xhs.to_radians().sin() * b.to_radians().sin() * alpha2.to_radians().cos();
        let c = cosc.clamp(-1.0, 1.0).acos().to_degrees();
        let sinzd = if c.abs() < f64::EPSILON {
            0.0
        } else {
            xhs.to_radians().sin() * alpha2.to_radians().sin() / c.to_radians().sin()
        };
        let zd = sinzd.clamp(-1.0, 1.0).asin().to_degrees();
        let rax = (latitude.to_radians().cos() * zd.to_radians().tan())
            .atan()
            .to_degrees();
        let pole = (sinzd * latitude.to_radians().sin())
            .clamp(-1.0, 1.0)
            .asin()
            .to_degrees();
        let pole = if house <= 6 { -pole } else { pole };
        let a = if house <= 6 {
            sidereal_time + 180.0 + rax
        } else {
            sidereal_time + rax
        };
        cusps[house - 1] = asc1(a, pole, sin_ecl, cos_ecl).degrees();
    }

    if mc_under_horizon {
        for house in [2usize, 3, 5, 6, 8, 9, 11, 12] {
            cusps[house - 1] =
                longitude_opposite(Longitude::from_degrees(cusps[house - 1])).degrees();
        }
    }
    cusps
}

/// Builds the `(Instant, ObserverLocation, Angle, HouseAngles)` argument tuple
/// for a Sunshine geometry, so Tasks 4-5 share one constructor.
fn sun_geom(
    jd: f64,
    lat: f64,
    lon: f64,
    obl: f64,
    asc: f64,
    mc: f64,
) -> (Instant, ObserverLocation, Angle, HouseAngles) {
    (
        Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(lon),
            None,
        ),
        Angle::from_degrees(obl),
        gc_angles(asc, mc),
    )
}

#[test]
fn sunshine_houses_matches_independent_recomposition() {
    // Geometry table: row A mid-lat acmc>0 (no axis flip, mc above horizon);
    // row B mid-lat acmc<0 (exercises the 1552 axis flip + 1554 mc flip);
    // row C high-lat (66 deg), exercising the per-house loop under a different
    // hemisphere sign. Note: mc_under_horizon can NEVER be true here — or at any
    // latitude — because apparent_midheaven_declination is bounded to
    // +/-obliquity (~23.44 deg), so |lat - mc_dec| maxes at 66 + 23.44 = 89.44,
    // under the 90 deg guard threshold. The dedicated under-horizon flip rows
    // live in `sunshine_houses_under_horizon_guards_match_recomposition` below.
    // All rows have non-degenerate loop terms, so the 51 loop-body swaps and the
    // 2 axis-flip guards are all killed.
    let obl = 23.4392811;
    let rows = [
        sun_geom(2_451_600.0, 52.0, 10.0, obl, 100.0, 15.0), // A: acmc = 85 > 0
        sun_geom(2_455_000.0, 40.0, -75.0, obl, 15.0, 100.0), // B: acmc = -85 < 0
        sun_geom(2_451_600.0, 66.0, 200.0, obl, 300.0, 210.0), // C: high lat
    ];
    for (i, (instant, observer, obliquity, angles)) in rows.iter().enumerate() {
        let got = sunshine_houses(*instant, observer, *obliquity, *angles);
        let want = recompose_sunshine(*instant, observer, *obliquity, *angles);
        for h in 0..12 {
            assert!(
                (got[h].degrees() - want[h]).abs() < 1.0e-9,
                "row {i} cusp[{h}]: crate {} != recompose {}",
                got[h].degrees(),
                want[h]
            );
        }
    }
}

#[test]
fn sunshine_houses_under_horizon_guard_is_exercised_both_ways() {
    // Precondition: the crafted rows must land on opposite sides of the guard,
    // else the comparison-swap mutants would survive vacuously.
    let obl = 23.4392811;
    let st_true = local_sidereal_time(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        Longitude::from_degrees(295.0),
    )
    .degrees();
    let mc_dec_true = apparent_midheaven_declination(st_true, obl);
    assert!(
        (80.0_f64 - mc_dec_true).abs() > 90.0,
        "expected mc_under_horizon TRUE row: |80 - {mc_dec_true}| must exceed 90"
    );
    let st_false = local_sidereal_time(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        Longitude::from_degrees(20.0),
    )
    .degrees();
    let mc_dec_false = apparent_midheaven_declination(st_false, obl);
    assert!(
        (80.0_f64 - mc_dec_false).abs() < 90.0,
        "expected mc_under_horizon FALSE row: |80 - {mc_dec_false}| must be under 90"
    );
}

#[test]
fn sunshine_houses_under_horizon_guards_match_recomposition() {
    // Rows chosen (Step 1) to straddle mc_under_horizon and to include a lat==0
    // row, so every 1545/1546/1607/1609 guard mutant changes at least one cusp
    // vs the correct recomposition. lon sets sidereal time (mc_dec sign). Row 1
    // (TRUE, latitude=80 != 0) alone already kills 1545:46 (`!= 0.0` guard);
    // the lat==0 row is not needed to pin that mutant specifically -- it adds a
    // separate, non-degenerate equality check on the `latitude.signum() == 0.0`
    // branch (guard short-circuits to false regardless of the mc_dec term).
    let obl = 23.4392811;
    let rows = [
        sun_geom(2_451_600.0, 80.0, 295.0, obl, 300.0, 210.0), // mc_under_horizon = true
        sun_geom(2_451_600.0, 80.0, 20.0, obl, 100.0, 15.0),   // mc_under_horizon = false
        sun_geom(2_451_600.0, 0.0, 20.0, obl, 100.0, 15.0),    // lat == 0 -> false
    ];
    for (i, (instant, observer, obliquity, angles)) in rows.iter().enumerate() {
        let got = sunshine_houses(*instant, observer, *obliquity, *angles);
        let want = recompose_sunshine(*instant, observer, *obliquity, *angles);
        for h in 0..12 {
            assert!(
                (got[h].degrees() - want[h]).abs() < 1.0e-9,
                "row {i} cusp[{h}]: crate {} != recompose {}",
                got[h].degrees(),
                want[h]
            );
        }
    }
}

#[test]
fn sunshine_houses_axis_flip_zero_boundary_matches_recomposition() {
    // acmc = signed_longitude_difference(asc, mc) is exactly 0.0 when asc == mc.
    // At HEAD, `acmc < 0.0` is false there (no axis flip), and
    // `recompose_sunshine` uses the same `< 0.0` guard, so crate == recompose.
    // The `acmc < 0.0` -> `acmc <= 0.0` mutant flips the axis at this boundary,
    // diverging from recompose and killing 1552:13.
    let obl = 23.4392811;
    let (instant, observer, obliquity, angles) =
        sun_geom(2_451_600.0, 45.0, 30.0, obl, 120.0, 120.0);
    let got = sunshine_houses(instant, &observer, obliquity, angles);
    let want = recompose_sunshine(instant, &observer, obliquity, angles);
    for h in 0..12 {
        assert!(
            (got[h].degrees() - want[h]).abs() < 1.0e-9,
            "cusp[{h}]: crate {} != recompose {}",
            got[h].degrees(),
            want[h]
        );
    }
}

#[test]
fn sunshine_houses_under_horizon_exact_boundary_is_killed() {
    // Review fix (Task 5 follow-up): 1546:92 `> 90.0` -> `>= 90.0` was previously
    // documented equivalent, but `latitude` is a free input, so
    // `|latitude - mc_dec|` can be driven to EXACTLY 90.0 by construction:
    // set `latitude = mc_dec + 90.0`, i.e. `latitude - mc_dec == (mc_dec + 90.0)
    // - mc_dec`. At `lon = 356.5` this f64 subtraction round-trips exactly to
    // 90.0 (verified by the precondition assert below, not assumed).
    let obl = 23.4392811;
    let jd = 2_451_600.0;
    let lon = 356.5;
    let st = local_sidereal_time(
        Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        Longitude::from_degrees(lon),
    )
    .degrees();
    let mc_dec = apparent_midheaven_declination(st, obl);
    let latitude = mc_dec + 90.0;
    // Precondition: |latitude - mc_dec| must be exactly 90.0, or this test would
    // vacuously pass without ever exercising the mutant's differing branch.
    assert_eq!(
        (latitude - mc_dec).abs(),
        90.0,
        "exact-90 boundary precondition failed: mc_dec={mc_dec} latitude={latitude}"
    );

    // At HEAD, `> 90.0` is false here (mc_under_horizon = false, no flip), and
    // `recompose_sunshine` uses the identical `> 90.0` guard, so crate ==
    // recompose. The `>= 90.0` mutant makes mc_under_horizon = true, flipping
    // the 8 loop-house cusps and diverging from recompose -- killing 1546:92.
    let (instant, observer, obliquity, angles) = sun_geom(jd, latitude, lon, obl, 300.0, 210.0);
    let got = sunshine_houses(instant, &observer, obliquity, angles);
    let want = recompose_sunshine(instant, &observer, obliquity, angles);
    for h in 0..12 {
        assert!(
            (got[h].degrees() - want[h]).abs() < 1.0e-9,
            "cusp[{h}]: crate {} != recompose {}",
            got[h].degrees(),
            want[h]
        );
    }
}

#[test]
fn sunshine_houses_degenerate_semi_arc_guard_is_killed() {
    // Review fix (Task 5 follow-up): 1585:32 `c.abs() < f64::EPSILON` -> `== `
    // was previously documented equivalent, but `latitude = 90.0 + sundec`
    // drives the house>7 branch's `b = 90 - latitude + sundec` to exactly 0.0,
    // which collapses `cosc = cos(xhs)*cos(b) + sin(xhs)*sin(b)*cos(alpha2)` to
    // `cos(xhs)` (since cos(0)=1, sin(0)=0). At jd = 2_451_525.0 this also drives
    // `xhs` for houses 8/9/11/12 to (near-)zero, so `cosc` rounds to exactly
    // 1.0 in f64 and `c = acos(1.0) == 0.0` bit-exact.
    let obl = 23.4392811;
    let obl_angle = Angle::from_degrees(obl);
    let jd = 2_451_525.0;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let sundec = apparent_solar_declination(instant, obl_angle).degrees();
    let latitude = 90.0 + sundec;

    // Precondition: independently re-derive `c` for house 9 (the house>7
    // branch) via the same trig chain as `sunshine_houses`/`recompose_sunshine`
    // and confirm it lands on exactly 0.0 -- proving the guard fires at HEAD
    // and this test isn't a vacuous pass.
    let b = 90.0 - latitude + sundec;
    let offsets = sunshine_offsets(latitude, sundec);
    let offset = offsets[9];
    let xhs = 2.0
        * (sundec.to_radians().cos() * (offset.to_radians() / 2.0).sin())
            .asin()
            .to_degrees();
    let cosa = (sundec.to_radians().tan() * (xhs.to_radians() / 2.0).tan()).clamp(-1.0, 1.0);
    let alph = cosa.acos().to_degrees();
    let alpha2 = 180.0 - alph;
    let cosc = xhs.to_radians().cos() * b.to_radians().cos()
        + xhs.to_radians().sin() * b.to_radians().sin() * alpha2.to_radians().cos();
    let c = cosc.clamp(-1.0, 1.0).acos().to_degrees();
    assert_eq!(c, 0.0, "c==0 precondition failed for house 9: c={c}");

    // At HEAD, `c.abs() < f64::EPSILON` fires in both the crate and
    // `recompose_sunshine` (both use the identical guard), so both take
    // sinzd=0 and crate == recompose. The `c.abs() == f64::EPSILON` mutant is
    // false at c==0.0, so the crate falls through to `.../ sin(0deg).to_radians()`
    // == division by zero == NaN, while recompose stays finite -- the equality
    // assertion below fails on NaN, killing 1585:32.
    let (instant, observer, obliquity, angles) = sun_geom(jd, latitude, 40.0, obl, 130.0, 20.0);
    let got = sunshine_houses(instant, &observer, obliquity, angles);
    let want = recompose_sunshine(instant, &observer, obliquity, angles);
    for h in 0..12 {
        assert!(
            (got[h].degrees() - want[h]).abs() < 1.0e-9,
            "cusp[{h}]: crate {} != recompose {}",
            got[h].degrees(),
            want[h]
        );
    }
}

/// Census of the Sunshine/solar-arc-family mutants left as *documented
/// equivalents* after this FU-9 slice: 5 surviving mutants, each unobservable
/// through the public API (killing any would require a production behavior
/// change, out of scope for this tests-only slice). Left visible (never
/// `#[mutants::skip]`) so a future reader sees the reachability argument.
/// Measured by the authoritative scoped family re-run (Task 6 Step 3).
///
/// --- nutation_for (2) ---
/// (NUT-1) 600:30 `delta_psi_arcsec / 3600.0 -> * 3600.0` and
/// (NUT-2) 600:30 `delta_psi_arcsec / 3600.0 -> % 3600.0`: both mutate the FIRST
///   tuple element of `nutation_for(instant) -> Result<(delta_psi_deg,
///   delta_eps_deg), HouseError>` (the `Ok` payload's `.0`).
///   BOTH call sites discard it -- `asc_mc` (mod.rs:268, `let (_dpsi, deps) =
///   ...`) and `validated_obliquity` (mod.rs:610, `let (_delta_psi_deg,
///   delta_eps_deg) = ...`). Only `delta_eps` feeds obliquity (observed, caught
///   by validate-houses/validate-angles). `delta_psi` reaches no public output,
///   so its arithmetic is unobservable => equivalent.
///
/// --- sunshine_houses (3) ---
/// (SUN-1) 1576:36 `house > 7 -> house >= 7` (below/above-horizon branch split):
///   the per-house loop iterates the fixed set [2,3,5,6,8,9,11,12] (mod.rs:1568);
///   `house` is never 7, so `> 7` and `>= 7` agree on every reachable value
///   (false for {2,3,5,6}, true for {8,9,11,12}). No reachable input
///   distinguishes them => equivalent.
/// (SUN-2) 1585:32 `c.abs() < f64::EPSILON -> <=` (semi-arc div-by-zero guard):
///   differs only at c.abs() == f64::EPSILON (~2.22e-16 deg) exactly. c =
///   acos(clamp(cosc)).to_degrees(); by the acos singularity near cosc=1 the
///   reachable set of c.abs() is {0.0} U [~8.5e-7 deg, ...] -- the next
///   representable f64 below 1.0 already yields c ~ sqrt(2*ulp) ~ 8.5e-7 deg. So
///   EPSILON sits in an unreachable structural gap; `<` and `<=` agree on every
///   reachable input => equivalent. (The `< -> ==` sibling IS killable --
///   c.abs()==0.0 is reachable at a house-8 degenerate semi-arc -- and is killed
///   by `sunshine_houses_degenerate_semi_arc_guard_is_killed`.)
/// (SUN-3) 1600:27 `sidereal_time + 180.0 -> sidereal_time - 180.0` (RAMC
///   offset, house<=6): the two arguments differ by exactly 360 deg; asc1
///   normalizes its first arg mod 360 (mod.rs:1794), so the physical ascendant
///   is identical for every input. The only f64-level divergence is a wraparound
///   representation seam at a measure-zero input set where normalization rounds x
///   and x-360 to distinct representatives of the SAME angle (e.g. 1e-12 deg vs
///   359.999999999999 deg); these are one modular Longitude differing by ~2e-12
///   deg -- within the 1e-9 recomposition tolerance. The mutant cannot change the
///   meaningful (modular) house cusp => equivalent. (A raw-subtraction "kill"
///   would only pin `recompose_sunshine`'s non-modular comparison seam, not
///   intent; unlike SUN-2's `==` sibling, the output here is behaviorally
///   unchanged.)
#[test]
fn sunshine_family_equivalent_mutants_are_documented() {
    // Liveness: assert the OBSERVABLE half (delta_eps) is genuinely exercised,
    // so the equivalence claim rests on a live path rather than dead code. A
    // None-obliquity Sunshine request drives validated_obliquity ->
    // nutation_for -> delta_eps -> cusps.
    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_600.0), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(48.0),
            Longitude::from_degrees(12.0),
            None,
        ),
        HouseSystem::Sunshine,
    );
    // obliquity defaulted (None) => mean_obliquity + delta_eps from nutation_for.
    assert!(request.obliquity.is_none());
    let snapshot = calculate_houses(&request).expect("sunshine houses should work");
    assert_eq!(snapshot.cusps.len(), 12);
}
