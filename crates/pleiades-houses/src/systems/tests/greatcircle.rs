//! The great-circle projection systems: Horizon (azimuth/altitude), APC
//! (`apc_sector`/`apc_houses`), and Krusinski-Pisa-Goelzer.

use super::support::*;
use crate::systems::*;
use pleiades_types::{Angle, JulianDay, Latitude, TimeScale};

/// SE-anchored regression for the Horizon ('H') azimuth convention fix.
///
/// Before the fix `horizon_houses` disagreed with Swiss-Ephemeris by ~55–115°
/// per cusp (missing the +180° post-rotation, an extra +90° azimuth quarter-turn
/// via `ascendant_for`, and a `>= 0` latitude branch that flipped cusp 1 at the
/// equator). The fixtures below are the exact SE 2.10.03 'H' reference rows from
/// the house corpus (grep ',Horizon,' cusps.csv), covering the equator, mid-
/// latitudes, the lat-66° support bound, and an elevated off-J2000 epoch.
#[test]
fn horizon_houses_match_swiss_ephemeris_reference() {
    // (lat, lon, jd, [c1..c12])
    let fixtures: [(f64, f64, f64, [f64; 12]); 5] = [
        (
            0.0,
            0.0,
            2451545.0,
            [
                180.0, 131.019922, 111.942372, 99.611088, 86.327570, 62.021662, 0.0, 311.019922,
                291.942372, 279.611088, 266.327570, 242.021662,
            ],
        ),
        (
            40.0,
            0.0,
            2451545.0,
            [
                7.512627, 40.957940, 71.860911, 99.611088, 126.477720, 155.300954, 187.512627,
                220.957940, 251.860911, 279.611088, 306.477720, 335.300954,
            ],
        ),
        (
            55.0,
            0.0,
            2451545.0,
            [
                8.738716, 39.436172, 69.870644, 99.611088, 128.927706, 158.489993, 188.738716,
                219.436172, 249.870644, 279.611088, 308.927706, 338.489993,
            ],
        ),
        (
            66.0,
            0.0,
            2451545.0,
            [
                9.545351, 39.500052, 69.532860, 99.611088, 129.656324, 159.623396, 189.545351,
                219.500052, 249.532860, 279.611088, 309.656324, 339.623396,
            ],
        ),
        (
            40.0,
            30.0,
            2433283.0,
            [
                29.044129, 64.514574, 99.194140, 128.147116, 153.317114, 178.927456, 209.044129,
                244.514574, 279.194140, 308.147116, 333.317114, 358.927456,
            ],
        ),
    ];
    let mut worst = 0.0f64;
    for (lat, lon, jd, want) in fixtures {
        let req = HouseRequest::new(
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
            ObserverLocation::new(
                Latitude::from_degrees(lat),
                Longitude::from_degrees(lon),
                None,
            ),
            HouseSystem::Horizon,
        );
        let snap = calculate_houses(&req).expect("horizon houses should compute");
        // MC must be the true meridian; cusp 10 is anchored to it directly.
        assert_eq!(snap.cusps[9], snap.angles.midheaven);
        for (i, (cusp, expected)) in snap.cusps.iter().zip(want.iter()).enumerate() {
            let got = cusp.degrees();
            let mut diff = (got - expected).rem_euclid(360.0);
            if diff > 180.0 {
                diff -= 360.0;
            }
            let arcsec = diff.abs() * 3600.0;
            if arcsec > worst {
                worst = arcsec;
            }
            // Tight per-cusp tolerance well inside the GreatCircle 5.0″ ceiling.
            assert!(
                arcsec < 1.0,
                "Horizon lat={lat} cusp {} = {got:.6}° differs from SE {expected:.6}° by {arcsec:.4}\u{2033}",
                i + 1,
            );
        }
    }
    // Whole-corpus Horizon worst residual stays far below the GreatCircle ceiling.
    assert!(
        worst < 1.0,
        "worst Horizon residual {worst:.4}\u{2033} exceeded 1\u{2033}"
    );
}

#[test]
fn krusinski_pisa_goelzer_release_system_preserves_the_documented_opposite_pairs() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::KrusinskiPisaGoelzer))
        .expect("krusinski-pisa-goelzer houses should work");

    assert_eq!(snapshot.cusps.len(), 12);
    assert!(snapshot.cusps.iter().all(|cusp| cusp.degrees().is_finite()));

    for index in 0..6 {
        let diff = (snapshot.cusps[index + 6].degrees() - snapshot.cusps[index].degrees())
            .rem_euclid(360.0);
        assert!(
            (diff - 180.0).abs() < 1.0e-10,
            "krusinski cusp {} and cusp {} should be opposite (diff {diff:.15} should be ~180°)",
            index + 1,
            index + 7,
        );
    }
}

// ===== FU-9 Great-circle PR: apc_sector / apc_houses / horizon / krusinski =====

#[test]
fn apc_sector_pins_all_twelve_against_independent_reference() {
    // Independent reference (houses-reference.py `apc_sector`, published APC
    // algorithm) at lat=52°, obl=23.4366°, sidereal=45° — non-degenerate, so
    // every `*`/`+`/`-`/`/` swap and the `n < 8` split is observable. Pinning
    // all 12 sectors kills all 58 arith survivors (measured 59/59 caught).
    let lat = 52.0_f64.to_radians();
    let obl = 23.4366_f64.to_radians();
    let sid = 45.0_f64.to_radians();
    let expected = [
        148.587_249_395_771,
        166.495_240_772_036,
        189.747_228_099_578,
        227.463_595_280_938,
        275.481_343_990_138,
        308.273_382_675_614,
        328.587_249_395_771,
        350.866_340_446_180,
        14.729_289_455_955,
        47.463_595_280_938,
        88.169_411_590_291,
        122.556_859_248_324,
    ];
    for (i, e) in expected.iter().enumerate() {
        let got = apc_sector(i + 1, lat, obl, sid).degrees();
        assert!(
            (got - e).abs() < 1e-9,
            "apc_sector({}) = {got}, want {e}",
            i + 1
        );
    }
}

fn gc_instant() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

#[test]
fn apc_houses_calls_apc_sector_with_one_indexed_sectors() {
    // Recomposition (independent of apc_houses's own indexing arithmetic): each
    // non-angle cusp i must equal the pure apc_sector for sector n = i+1. The
    // `index + 1 -> index * 1` mutant makes cusp[i] = apc_sector(i), differing
    // from apc_sector(i+1) at every reachable geometry. `st` is threaded from
    // the un-mutated local_sidereal_time (not mutated in this slice), so the
    // check is non-circular; apc_sector's own arithmetic is pinned in Task 1.
    let instant = gc_instant();
    let obs = ObserverLocation::new(
        Latitude::from_degrees(52.0),
        Longitude::from_degrees(0.0),
        None,
    );
    let obl = Angle::from_degrees(23.4366);
    let angles = gc_angles(10.0, 280.0);
    let cusps = apc_houses(instant, &obs, obl, angles);
    let st_rad = local_sidereal_time(instant, obs.longitude)
        .degrees()
        .to_radians();
    let (lat_rad, obl_rad) = (52.0_f64.to_radians(), 23.4366_f64.to_radians());
    for i in [1usize, 2, 3, 4, 5, 6, 7, 8, 10, 11] {
        let want = apc_sector(i + 1, lat_rad, obl_rad, st_rad).degrees();
        assert!(
            (cusps[i].degrees() - want).abs() < 1e-9,
            "apc cusp[{i}] = {}, want apc_sector({}) = {want}",
            cusps[i].degrees(),
            i + 1
        );
    }
    assert_eq!(cusps[0], angles.ascendant);
    assert_eq!(cusps[9], angles.midheaven);
}

// Independent recomposition of the SE 'H' horizon body from the published
// formula (azimuth origin Asc1(th+90) via ascendant_for(th,·), +180 post-fold
// via longitude_opposite, strict `>0` hemisphere branch, VERY_SMALL pole clamp).
// Calls only shared primitives already pinned in Foundation; `st` is threaded
// from the un-mutated local_sidereal_time. Confirmed to equal the crate at HEAD
// to 1e-9 during plan authoring, so any structural mutant that diverges dies.
fn recompose_horizon(
    instant: Instant,
    obs: &ObserverLocation,
    obl_deg: f64,
    mc: Longitude,
) -> [Longitude; 12] {
    let st = (local_sidereal_time(instant, obs.longitude).degrees() + 180.0).rem_euclid(360.0);
    let obl_rad = obl_deg.to_radians();
    let lat = obs.latitude.degrees();
    let mut tl = if lat > 0.0 { 90.0 - lat } else { -90.0 - lat };
    const VS: f64 = 1e-10;
    if (tl.abs() - 90.0).abs() < VS {
        tl = if tl < 0.0 { -90.0 + VS } else { 90.0 - VS };
    }
    let tlr = tl.to_radians();
    let fh1 = (tlr.sin() / 2.0).asin().to_degrees();
    let fh2 = ((3.0_f64).sqrt() / 2.0 * tlr.sin()).asin().to_degrees();
    let cosfi = tlr.cos();
    let (xh1, xh2) = if cosfi == 0.0 {
        if tl > 0.0 {
            (90.0, 90.0)
        } else {
            (270.0, 270.0)
        }
    } else {
        (
            (3.0_f64.sqrt() / cosfi).atan().to_degrees(),
            (1.0 / 3.0_f64.sqrt() / cosfi).atan().to_degrees(),
        )
    };
    let mut c = [Longitude::from_degrees(0.0); 12];
    c[0] = longitude_opposite(ascendant_for(st, tl, obl_rad));
    c[9] = mc;
    c[10] = longitude_opposite(ascendant_for(st - xh1, fh1, obl_rad));
    c[11] = longitude_opposite(ascendant_for(st - xh2, fh2, obl_rad));
    c[1] = longitude_opposite(ascendant_for(st + xh2, fh2, obl_rad));
    c[2] = longitude_opposite(ascendant_for(st + xh1, fh1, obl_rad));
    c[3] = longitude_opposite(c[9]);
    c[4] = longitude_opposite(c[10]);
    c[5] = longitude_opposite(c[11]);
    c[6] = longitude_opposite(c[0]);
    c[7] = longitude_opposite(c[1]);
    c[8] = longitude_opposite(c[2]);
    c
}

#[test]
fn horizon_houses_match_independent_recomposition_across_geometries() {
    let instant = gc_instant();
    let obl = Angle::from_degrees(23.4366);
    // lat=-33 kills the `-90 - lat` southern-hemisphere sign mutant (1089);
    // lat=1e-11 (N, near equator) reaches the +90-VS clamp target and flips
    //   cosfi's sign under the 1098 mutants (killing both);
    // lat=90 (pole) makes the `/90` clamp-guard mutant (1094:36 `/`) clamp
    //   where HEAD does not (cosfi 1 -> 1.7e-12), killing it.
    for lat in [-33.0_f64, 0.0, 40.0, 1e-11, 90.0] {
        let obs = ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(0.0),
            None,
        );
        let angles = gc_angles(10.0, 280.0);
        let got = horizon_houses(instant, &obs, obl, angles);
        let want = recompose_horizon(instant, &obs, 23.4366, angles.midheaven);
        for i in 0..12 {
            let mut d = (got[i].degrees() - want[i].degrees()).rem_euclid(360.0);
            if d > 180.0 {
                d -= 360.0;
            }
            assert!(d.abs() < 1e-9, "horizon lat={lat} cusp[{i}] diff {d}");
        }
    }
}

#[test]
fn horizon_pole_singularity_equivalent_mutants_are_documented() {
    // FU-9 Great-circle residual: 8 surviving mutants in horizon_houses, all in
    // the pole-singularity handling, each an EQUIVALENT MUTANT left visible (no
    // #[mutants::skip]). Magnitudes below are MEASURED, not claimed exact.
    //
    // (A) Sub-tolerance mod-360 antipode — 1082:69 `+ 180.0 -> - 180.0`:
    //   sidereal_time = (LST ± 180).rem_euclid(360). (x+180)%360 and (x-180)%360
    //   are equal in real arithmetic (differ by 360) and differ in f64 by at most
    //   ~5.68e-14° over a fine LST sweep — far below the 1e-9° pin tolerance, so
    //   no white-box pin can distinguish them. (Same shape as the Foundation
    //   `armc ± 180` finding.)
    //
    // (B) Clamp-guard mutants only reachable where the clamp is sub-tolerance:
    //   1094:36 `(|tl|-90) -> (|tl|+90)`  — `|tl|+90 >= 90` so the guard never
    //     fires (never clamps); differs from HEAD only at lat≈0 (tl≈±90), where
    //     the clamp alters tl by ≤1e-10° and atan(√3/cosφ) has already saturated
    //     to ±90°, so the cusp displacement is <1e-9°. (Its `/90` sibling was
    //     killable at lat=90 and IS caught — see the recomposition test.)
    //   1094:50 `< -> ==` and `< -> <=` — the guard fires only at exactly
    //     value == 1e-10 (measure-zero); the reachable difference (lat≈0) is the
    //     same sub-tolerance clamp effect as above.
    //   1095:56 `tl < 0.0 -> tl <= 0.0` — this branch is entered only when the
    //     clamp fires, i.e. |tl|≈90, so tl is ±90 and never 0; the operators
    //     differ only at tl == 0.0, unreachable here.
    //
    // (C) Structurally dead branch — 1108:33 `> -> ==`, `-> <`, `-> >=`:
    //   these steer the `if cosfi == 0.0 { ... }` arm. cos(tl_rad) is never
    //   exactly 0.0 for any reachable tl (min |cos| over the clamp-guaranteed
    //   range incl. raw ±90 is 6.123e-17), so the whole arm is unreachable and
    //   all three mutants are equivalent regardless of geometry.
    //
    // This test asserts the two facts the arguments rest on, so the reasoning is
    // guarded against a future code change that would make a survivor killable.
    // Fact (A): the antipode difference stays sub-tolerance.
    let mut worst = 0.0_f64;
    let mut lst = 0.0_f64;
    while lst < 360.0 {
        let a = (lst + 180.0).rem_euclid(360.0);
        let b = (lst - 180.0).rem_euclid(360.0);
        let mut d = (a - b).rem_euclid(360.0);
        if d > 180.0 {
            d -= 360.0;
        }
        worst = worst.max(d.abs());
        lst += 0.0007;
    }
    assert!(
        worst < 1e-9,
        "antipode (x±180)%360 diff {worst} must stay sub-tolerance"
    );
    // Fact (C): cos(tl_rad) never hits exactly 0 across the reachable tl range.
    for tl in [
        90.0_f64 - 1e-10,
        -90.0_f64 + 1e-10,
        90.0,
        -90.0,
        0.0,
        45.0,
        -57.0,
    ] {
        assert!(
            tl.to_radians().cos() != 0.0,
            "cosfi==0.0 arm is dead (tl={tl})"
        );
    }
}

// Independent recomposition of the Krusinski-Pisa-Goelzer body from the
// published great-circle projection: flip the ascendant when it trails the MC,
// rotate the house-circle anchor into the horizon frame, then step 30° arcs and
// project back to ecliptic longitude. Calls only Foundation-pinned primitives;
// `st` threaded from the un-mutated local_sidereal_time. Confirmed == crate at
// HEAD to 1e-9 during plan authoring.
fn recompose_krusinski(
    instant: Instant,
    obs: &ObserverLocation,
    obl_deg: f64,
    angles: HouseAngles,
) -> [Longitude; 12] {
    let st = local_sidereal_time(instant, obs.longitude).degrees();
    let lat = obs.latitude.degrees();
    let mut ascendant = angles.ascendant;
    if signed_longitude_difference(ascendant.degrees(), angles.midheaven.degrees()) < 0.0 {
        ascendant = longitude_opposite(ascendant);
    }
    let mut hcp = [ascendant.degrees(), 0.0, 1.0];
    spherical_cotrans(&mut hcp, -obl_deg);
    hcp[0] = normalize_degrees(hcp[0] - (st - 90.0));
    spherical_cotrans(&mut hcp, -(90.0 - lat));
    let horizon_offset = hcp[0];
    let mut c = [Longitude::from_degrees(0.0); 12];
    for index in 0..6 {
        let mut p = [30.0 * index as f64, 0.0, 1.0];
        spherical_cotrans(&mut p, 90.0);
        p[0] = normalize_degrees(p[0] + horizon_offset);
        spherical_cotrans(&mut p, 90.0 - lat);
        p[0] = normalize_degrees(p[0] + (st - 90.0));
        c[index] = ecliptic_longitude_from_ra(p[0], obl_deg.to_radians());
        c[index + 6] = longitude_opposite(c[index]);
    }
    c[0] = ascendant;
    c[6] = longitude_opposite(ascendant);
    c
}

#[test]
fn krusinski_houses_match_independent_recomposition_across_geometries() {
    let instant = gc_instant();
    let obl = Angle::from_degrees(23.4366);
    // (asc=10, mc=100): signed_diff = -90 < 0 -> flip fires.
    // (asc=200, mc=100): signed_diff = +100 > 0 -> flip does not fire.
    // (-33, ...): southern observer.
    // (asc=100, mc=100): signed_diff == 0 -> kills the `< -> <=` boundary mutant.
    for (lat, asc, mc) in [
        (52.0_f64, 10.0, 100.0),
        (52.0, 200.0, 100.0),
        (-33.0, 10.0, 100.0),
        (52.0, 100.0, 100.0),
    ] {
        let obs = ObserverLocation::new(
            Latitude::from_degrees(lat),
            Longitude::from_degrees(0.0),
            None,
        );
        let angles = gc_angles(asc, mc);
        let got = krusinski_pisa_goelzer_houses(instant, &obs, obl, angles);
        let want = recompose_krusinski(instant, &obs, 23.4366, angles);
        for i in 0..12 {
            let mut d = (got[i].degrees() - want[i].degrees()).rem_euclid(360.0);
            if d > 180.0 {
                d -= 360.0;
            }
            assert!(
                d.abs() < 1e-9,
                "krusinski lat={lat} asc={asc} cusp[{i}] diff {d}"
            );
        }
    }
}
