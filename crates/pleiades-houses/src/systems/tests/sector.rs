//! The sector/quadrant-division systems: Gauquelin (36-sector),
//! `solve_gauquelin_sector`, and the Pullen family (`pullen_sr_houses`,
//! `pullen_sd_houses`/`albategnius_houses`, which are byte-identical).

use super::support::*;
use crate::systems::*;
use pleiades_types::Latitude;

#[test]
fn gauquelin_release_system_exposes_thirty_six_sectors() {
    let snapshot = calculate_houses(&sample_request(HouseSystem::Gauquelin))
        .expect("gauquelin sectors should work");
    assert_eq!(snapshot.cusps.len(), 36);
    assert_eq!(snapshot.cusps[0], snapshot.angles.ascendant);
    assert_eq!(snapshot.cusps[9], snapshot.angles.midheaven);
    assert_eq!(
        snapshot.cusps[18],
        longitude_opposite(snapshot.angles.ascendant)
    );
    assert_eq!(
        snapshot.cusps[27],
        longitude_opposite(snapshot.angles.midheaven)
    );
    assert_eq!(snapshot.cusp(36), Some(snapshot.cusps[35]));
    // Gauquelin sectors are a Placidus-family semi-arc division, so the lower
    // hemisphere is the exact antipode of the upper hemisphere:
    // s(k + 18) = opposite(s(k)). (Numeric SE agreement is asserted separately
    // in `gauquelin_sectors_match_swiss_ephemeris_reference`.)
    for k in 0..18 {
        let upper = snapshot.cusps[k].degrees();
        let lower = snapshot.cusps[k + 18].degrees();
        let diff = (((upper + 180.0) - lower + 180.0).rem_euclid(360.0)) - 180.0;
        assert!(
            diff.abs() < 1.0e-9,
            "Gauquelin sector {} should be the antipode of sector {}",
            k + 19,
            k + 1
        );
    }
}

#[test]
fn gauquelin_sectors_match_swiss_ephemeris_reference() {
    // Ground-truth Swiss-Ephemeris 36-sector values (sectors.csv fixtures):
    // (jd_ut, lat_deg, lon_deg, [s1..s36]).
    // Gauquelin sectors are a Placidus-family semi-arc division (each diurnal/
    // nocturnal quadrant split into ninths), NOT a longitude lerp. These values
    // exercise the equator, mid-latitudes, and the near-polar (66°) regime.
    struct Fixture {
        id: &'static str,
        jd: f64,
        lat: f64,
        lon: f64,
        sectors: [f64; 36],
    }
    let fixtures = [
        Fixture {
            id: "c0_lat00",
            jd: 2451545.0,
            lat: 0.0,
            lon: 0.0,
            sectors: [
                11.373900, 0.498173, 349.616833, 338.849424, 328.295170, 318.017909, 308.040522,
                298.347719, 288.893685, 279.611088, 270.419362, 261.231657, 251.960854, 242.525543,
                232.856860, 222.906648, 212.656467, 202.125490, 191.373900, 180.498173, 169.616833,
                158.849424, 148.295170, 138.017909, 128.040522, 118.347719, 108.893685, 99.611088,
                90.419362, 81.231657, 71.960854, 62.525543, 52.856860, 42.906648, 32.656467,
                22.125490,
            ],
        },
        Fixture {
            id: "c1_lat40",
            jd: 2451545.0,
            lat: 40.0,
            lon: 0.0,
            sectors: [
                17.706103, 0.736222, 345.587073, 332.464496, 321.131729, 311.221449, 302.382578,
                294.320768, 286.796477, 279.611088, 272.591552, 265.575907, 258.399152, 250.877889,
                242.791680, 233.858979, 223.707487, 211.848972, 197.706103, 180.736222, 165.587073,
                152.464496, 141.131729, 131.221449, 122.382578, 114.320768, 106.796477, 99.611088,
                92.591552, 85.575907, 78.399152, 70.877889, 62.791680, 53.858979, 43.707487,
                31.848972,
            ],
        },
        Fixture {
            id: "c2_lat55",
            jd: 2451545.0,
            lat: 55.0,
            lon: 0.0,
            sectors: [
                28.505186, 1.107822, 340.254713, 325.231349, 313.955164, 305.003978, 297.529337,
                291.013310, 285.119432, 279.611088, 274.305089, 269.042114, 263.663645, 257.987911,
                251.775970, 244.671608, 236.075863, 224.846914, 208.505186, 181.107822, 160.254713,
                145.231349, 133.955164, 125.003978, 117.529337, 111.013310, 105.119432, 99.611088,
                94.305089, 89.042114, 83.663645, 77.987911, 71.775970, 64.671608, 56.075863,
                44.846914,
            ],
        },
        Fixture {
            id: "c3_lat66",
            jd: 2451545.0,
            lat: 66.0,
            lon: 0.0,
            sectors: [
                87.195607, 3.702693, 318.826001, 303.196619, 295.102092, 290.005092, 286.407601,
                283.668075, 281.463357, 279.611088, 277.998875, 276.551926, 275.216522, 273.950574,
                272.717332, 271.480026, 270.195365, 268.802302, 267.195607, 183.702693, 138.826001,
                123.196619, 115.102092, 110.005092, 106.407601, 103.668075, 101.463357, 99.611088,
                97.998875, 96.551926, 95.216522, 93.950574, 92.717332, 91.480026, 90.195365,
                88.802302,
            ],
        },
        Fixture {
            id: "c4_lat40_e2",
            jd: 2433283.0,
            lat: 40.0,
            lon: 30.0,
            sectors: [
                60.830331, 45.921003, 30.402853, 15.112631, 0.775651, 347.777129, 336.178607,
                325.848219, 316.577373, 308.147116, 300.355307, 293.022440, 285.987810, 279.101589,
                272.214688, 265.166046, 257.765393, 249.767458, 240.830331, 225.921003, 210.402853,
                195.112631, 180.775651, 167.777129, 156.178607, 145.848219, 136.577373, 128.147116,
                120.355307, 113.022440, 105.987810, 99.101589, 92.214688, 85.166046, 77.765393,
                69.767458,
            ],
        },
    ];

    // Sector-family ceiling is 2.0″ (see thresholds.rs); the test tolerance is
    // the same so the gate and this unit test agree (measured max is ~0.49″).
    let tolerance_arcsec = 2.0;
    let mut overall_max = 0.0_f64;
    for fx in &fixtures {
        let request = HouseRequest::new(
            Instant::new(
                pleiades_types::JulianDay::from_days(fx.jd),
                pleiades_types::TimeScale::Tt,
            ),
            ObserverLocation::new(
                Latitude::from_degrees(fx.lat),
                Longitude::from_degrees(fx.lon),
                None,
            ),
            HouseSystem::Gauquelin,
        );
        let snapshot = calculate_houses(&request).expect("gauquelin sectors should compute");
        assert_eq!(snapshot.cusps.len(), 36);
        for (i, &want) in fx.sectors.iter().enumerate() {
            let got = snapshot.cusps[i].degrees();
            let resid = (((got - want + 180.0).rem_euclid(360.0)) - 180.0).abs() * 3600.0;
            overall_max = overall_max.max(resid);
            assert!(
                resid <= tolerance_arcsec,
                "{}: sector {} got {got:.6} want {want:.6} resid {resid:.3}\"",
                fx.id,
                i + 1
            );
        }
    }
    eprintln!("gauquelin max residual vs SE = {overall_max:.4} arcsec");
}

// ===== FU-9 Sector PR: pullen_sr / pullen_sd / albategnius / gauquelin =====
// Independent reference: docs/superpowers/specs/notes/2026-07-22-houses-reference.py
// (`pullen_sd`, `pullen_sr`), cross-validated against the crate to ~1e-12 during
// plan authoring. gauquelin_houses already reaches 0 surviving mutants via the
// validate-houses / validate-angles parity gates, so it needs no new unit test;
// only solve_gauquelin_sector's guard survivors are addressed here.

fn assert_sector_cusps(got: &[Longitude; 12], want: &[f64; 12], label: &str) {
    for i in 0..12 {
        let mut d = (got[i].degrees() - want[i]).rem_euclid(360.0);
        if d > 180.0 {
            d -= 360.0;
        }
        assert!(
            d.abs() < 1e-9,
            "{label} cusp[{i}] = {}, want {}",
            got[i].degrees(),
            want[i]
        );
    }
}

#[test]
fn pullen_sd_and_albategnius_pin_all_cusps_against_independent_reference() {
    // pullen_sd_houses and albategnius_houses are byte-identical in the crate
    // (same equal-quadrant split); the same reference pins both. Geometries:
    //  200/100 acmc=100 -> both `else` quadrant-split branches (d != 0);
    //  120/100 acmc=20  -> MC-side bisect branch (acmc <= 30);
    //  260/100 acmc=160 -> ASC-side bisect branch (q1 = 180-acmc <= 30);
    //  10/100  acmc<0   -> ascendant flip branch (kills `< 0 -> == 0`);
    //  100/100 acmc=0   -> flip-guard equality (kills `< 0 -> <= 0`).
    let cases: [(f64, f64, [f64; 12]); 5] = [
        (
            200.0,
            100.0,
            [
                200.0, 227.5, 252.5, 280.0, 312.5, 347.5, 20.0, 47.5, 72.5, 100.0, 132.5, 167.5,
            ],
        ),
        (
            120.0,
            100.0,
            [
                120.0, 167.5, 232.5, 280.0, 290.0, 290.0, 300.0, 347.5, 52.5, 100.0, 110.0, 110.0,
            ],
        ),
        (
            260.0,
            100.0,
            [
                260.0, 270.0, 270.0, 280.0, 327.5, 32.5, 80.0, 90.0, 90.0, 100.0, 147.5, 212.5,
            ],
        ),
        (
            10.0,
            100.0,
            [
                190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0,
            ],
        ),
        (
            100.0,
            100.0,
            [
                100.0, 152.5, 227.5, 280.0, 280.0, 280.0, 280.0, 332.5, 47.5, 100.0, 100.0, 100.0,
            ],
        ),
    ];
    for (asc, mc, want) in cases {
        let angles = gc_angles(asc, mc);
        assert_sector_cusps(
            &pullen_sd_houses(angles),
            &want,
            &format!("pullen_sd asc={asc}"),
        );
        assert_sector_cusps(
            &albategnius_houses(angles),
            &want,
            &format!("albategnius asc={asc}"),
        );
    }
}

#[test]
fn pullen_sr_pins_all_cusps_against_independent_reference() {
    // Independent reference (houses-reference.py `pullen_sr`): ratio r solved as
    // the positive root of r^4 + 2r^3 - 2c*r - c = 0 (c=(180-q)/q) by bisection+
    // Newton, a different method than the crate's Ferrari closed form; matched to
    // ~1e-12 during plan authoring. Geometries:
    //  200/100 acmc=100 -> q>90 reduction (q=80) AND acmc>90 placement branch;
    //  140/100 acmc=40  -> no reduction AND acmc<=90 placement branch;
    //  10/100  acmc<0   -> flip -> acmc=90 (r=1 exactly);
    //  100/100 acmc=0   -> q<1e-30 guard branch (x=xr=xr3=0, xr4=180).
    let cases: [(f64, f64, [f64; 12]); 4] = [
        (
            200.0,
            100.0,
            [
                200.0,
                227.399778974511,
                252.600221025489,
                280.0,
                312.391037626774,
                347.608962373226,
                20.0,
                47.399778974511,
                72.600221025489,
                100.0,
                132.391037626774,
                167.608962373226,
            ],
        ),
        (
            140.0,
            100.0,
            [
                140.0,
                178.908843802504,
                241.091156197496,
                280.0,
                295.233904915732,
                304.766095084268,
                320.0,
                358.908843802504,
                61.091156197496,
                100.0,
                115.233904915732,
                124.766095084268,
            ],
        ),
        (
            10.0,
            100.0,
            [
                190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0,
            ],
        ),
        (
            100.0,
            100.0,
            [
                100.0, 100.0, 280.0, 280.0, 280.0, 280.0, 280.0, 280.0, 100.0, 100.0, 100.0, 100.0,
            ],
        ),
    ];
    for (asc, mc, want) in cases {
        assert_sector_cusps(
            &pullen_sr_houses(gc_angles(asc, mc)),
            &want,
            &format!("pullen_sr asc={asc}"),
        );
    }
}

#[test]
fn solve_gauquelin_sector_fails_closed_on_nonconvergence() {
    // Kills 1341 `!converged || !q.is_finite()` -> `&&`: at lat=80, obl=23.4366,
    // fraction=1/9, sign=+1, ramc=30 the Newton iteration does not converge in 64
    // steps but q stays finite (~52.4). HEAD: `!converged(true) || ...` -> Err;
    // the `&&` mutant: `true && !finite(false)` -> false -> Ok(unconverged). So
    // HEAD MUST return Err here for the `&&` mutant to be observable. (Geometry
    // found by a lat/obl/fraction/ramc sweep of the crate's Newton during plan
    // authoring: 22 non-converged-but-finite candidates, this is the first.)
    let r = solve_gauquelin_sector(30.0, 80.0, 23.4366, 1.0 / 9.0, 1.0);
    assert!(r.is_err(), "expected non-convergence Err, got {r:?}");
    // A physical Gauquelin geometry converges to a finite Ok (the live path).
    let ok = solve_gauquelin_sector(280.4570696, 52.0, 23.4366, 8.0 / 9.0, 1.0);
    assert!(ok.is_ok(), "expected convergence Ok, got {ok:?}");
}

/// Kills `solve_gauquelin_sector` 1327:21 `gp.abs() < 1e-12 -> ==`, withdrawing
/// the GQ-1 equivalent classification recorded by PR 3.
///
/// PR 3 argued the two operators were indistinguishable because both exit with
/// `Err(NumericalFailure)` and "the campaign does not pin error-message text".
/// That premise was wrong — the suite pins message text in several places, and
/// PR 5 killed the structurally identical `solve_placidian_cusp` 1741 `<` ->
/// `==` mutant exactly this way. The two exits carry DIFFERENT messages:
/// HEAD trips the zero-derivative guard; the `==` mutant falls through, divides
/// by a ~4e-18 derivative, and exits via the non-convergence branch.
///
/// Geometry: `gp` on the first iteration is linear in `tan(lat)` because
/// `sign = +1` fixes `arg = 90°`, so
///   `gp·(180/π) = -(1/fraction) + tan(lat)·tan(obl)·cos(ramc + fraction·90)`.
/// With `fraction = 8/9` and `ramc = 280°`, `alpha = 360° ≡ 0°` so `cos = 1`,
/// and the root is `lat = atan((9/8)/tan(23.4366°))`. Measured `|gp| = 3.875e-18`.
#[test]
fn solve_gauquelin_sector_fails_closed_on_a_zero_derivative() {
    let err = solve_gauquelin_sector(280.0, 68.926_784_442_096_97, 23.4366, 8.0 / 9.0, 1.0)
        .expect_err("the zero-derivative guard must fire at this geometry");

    assert_eq!(
        err.message, "gauquelin sector iteration encountered a zero derivative",
        "HEAD must exit via the zero-derivative guard, not the non-convergence branch",
    );
    assert_eq!(err.kind, HouseErrorKind::NumericalFailure);
}

#[test]
fn sector_equivalent_mutants_are_documented() {
    // FU-9 Sector residual: 5 surviving mutants, each an EQUIVALENT MUTANT left
    // visible (no #[mutants::skip]), enumerated with a reachability argument.
    // 233 tested per PR 3's Sector-family scoped run; 5 missed / 228 caught after
    // this PR's confirmed GQ-1 kill (46-mutant solve_gauquelin_sector-scoped rerun,
    // 2 missed: the <= variants at 1327 and 1335; no other in-function survivor moved).
    //
    // --- pullen_sr_houses (3) ---
    // (SR-1) 1437:10 `q > 90.0 -> q >= 90.0` (quadrant reduction): differs only at
    //   q == 90.0, where HEAD keeps q=90 and the mutant sets q=180-90=90 -- same q,
    //   same output. Reachable (acmc=90 via the asc=10/mc=100 flip) but coincident.
    // (SR-2) 1458:13 `acmc > 90.0 -> acmc >= 90.0` (placement branch): differs only
    //   at acmc == 90.0, where q=90 -> c=1 -> r=1 exactly, so xr == xr3 and x == xr4
    //   and the `if`/`else` placements produce bit-identical cusps.
    // (SR-3) 1441:34 `q < 1e-30 -> q <= 1e-30` (degenerate-quadrant guard): differs
    //   only at q == 1e-30 exactly -- a measure-zero boundary q (from
    //   signed_longitude_difference of f64 degrees) cannot reach.
    //
    // At the acmc=90 flip geometry the SR division is the r=1 equal 30-degree split,
    // so both the reduction and placement branches coincide (kills SR-1/SR-2 intent):
    let acmc90 = pullen_sr_houses(gc_angles(10.0, 100.0));
    let equal = [
        190.0, 220.0, 250.0, 280.0, 310.0, 340.0, 10.0, 40.0, 70.0, 100.0, 130.0, 160.0,
    ];
    assert_sector_cusps(&acmc90, &equal, "SR acmc=90 is the r=1 equal split");
    // The q<1e-30 guard is reached only at acmc=0 (asc==mc); signature x=xr=xr3=0
    // gives cusp[1]==asc and cusp[2]==desc (SR-3 boundary is this degenerate point):
    let guard = pullen_sr_houses(gc_angles(100.0, 100.0));
    assert!(
        (guard[1].degrees() - 100.0).abs() < 1e-9,
        "SR guard cusp[1]==asc"
    );
    assert!(
        (guard[2].degrees() - 280.0).abs() < 1e-9,
        "SR guard cusp[2]==desc"
    );
    //
    // --- solve_gauquelin_sector (2) ---
    // GQ-1 (1327:21 `gp.abs() < 1e-12 -> ==`) was withdrawn on 2026-07-25 and
    // killed by solve_gauquelin_sector_fails_closed_on_a_zero_derivative above;
    // the label is retired, not reused, so the two survivors below keep their
    // original numbers.
    // (GQ-2) 1327:21 `gp.abs() < 1e-12 -> <=`: differs only at gp.abs()==1e-12
    //   exactly -- measure-zero, unreachable.
    // (GQ-3) 1335:24 `delta.abs() < 1e-9 -> <=` (convergence): differs only at
    //   delta.abs()==1e-9 exactly -- a Newton iterate shrinking quadratically past
    //   1e-9 does not land on it; measure-zero, unreachable.
    //
    // The reachable non-convergence exit is a real Err (pinned by
    // solve_gauquelin_sector_fails_closed_on_nonconvergence) and a physical geometry
    // converges to Ok -- the live path both operators share:
    assert!(solve_gauquelin_sector(280.4570696, 52.0, 23.4366, 8.0 / 9.0, 1.0).is_ok());
}
