//! Shared geometry primitives used by every house-system family:
//! `chart_points_from_armc`, `spherical_cotrans`, `asc1`/`asc2`,
//! `asc_mc_from`, `interpolate_longitude`, `signed_longitude_difference`,
//! `right_ascension_from_ecliptic_longitude`, `longitude_in_arc`, and
//! `longitude_opposite`.

use crate::systems::*;

#[test]
fn chart_points_from_armc_mc_is_analytic_at_cardinal_armc() {
    use pleiades_types::{Angle, Latitude, Longitude};
    // With obliquity ε, MC longitude satisfies tan(λ_MC)=tan(ARMC)/cos(ε);
    // at ARMC = 0/90/180/270 the MC equals the ARMC exactly.
    let obl = Angle::from_degrees(23.4392911);
    for armc in [0.0_f64, 90.0, 180.0, 270.0] {
        let pts = chart_points_from_armc(
            Longitude::from_degrees(armc),
            Latitude::from_degrees(40.0),
            obl,
        )
        .expect("defined at 40N");
        let diff = (pts.midheaven.degrees() - armc).rem_euclid(360.0);
        let diff = diff.min(360.0 - diff);
        assert!(diff < 1e-6, "ARMC {armc}: MC {}", pts.midheaven.degrees());
    }
}

#[test]
fn chart_points_from_armc_mc_obliquity_coefficient_is_pinned() {
    use pleiades_types::{Angle, Latitude, Longitude};
    // Independent check of the MC's obliquity dependence at a NON-cardinal ARMC,
    // where the cardinal-value test cannot distinguish cos(ε) from any other
    // coefficient. The MC satisfies tan(λ_MC)·cos(ε) = tan(ARMC); a regression
    // swapping cos(ε) for sin(ε) (or any wrong coefficient) breaks this identity.
    let eps = Angle::from_degrees(23.4392911);
    for armc in [37.0_f64, 123.4, 210.0, 316.7] {
        let pts = chart_points_from_armc(
            Longitude::from_degrees(armc),
            Latitude::from_degrees(40.0),
            eps,
        )
        .expect("defined at 40N");
        let lhs = pts.midheaven.degrees().to_radians().tan() * eps.degrees().to_radians().cos();
        let rhs = armc.to_radians().tan();
        assert!(
            (lhs - rhs).abs() < 1e-9,
            "ARMC {armc}: tan(MC)·cos(ε)={lhs} vs tan(ARMC)={rhs}"
        );
    }
}

#[test]
fn chart_points_invariants_hold() {
    use pleiades_types::{Angle, Latitude, Longitude};
    let pts = chart_points_from_armc(
        Longitude::from_degrees(123.4),
        Latitude::from_degrees(51.5),
        Angle::from_degrees(23.4392911),
    )
    .expect("defined at 51.5N");
    let opp = |a: f64, b: f64| {
        let d = (a - b).rem_euclid(360.0);
        (d - 180.0).abs() < 1e-6
    };
    assert!(opp(pts.ascendant.degrees(), pts.descendant.degrees()));
    assert!(opp(pts.midheaven.degrees(), pts.imum_coeli.degrees()));
    assert!(opp(pts.vertex.degrees(), pts.antivertex.degrees()));
    for p in [
        pts.armc,
        pts.vertex,
        pts.equatorial_ascendant,
        pts.coascendant_koch,
        pts.coascendant_munkasey,
        pts.polar_ascendant,
    ] {
        assert!(
            (0.0..360.0).contains(&p.degrees()),
            "unnormalized {}",
            p.degrees()
        );
    }
    // ARMC round-trips the input.
    let d = (pts.armc.degrees() - 123.4).rem_euclid(360.0);
    assert!(d.min(360.0 - d) < 1e-9);
}

#[test]
fn chart_points_uses_true_obliquity_by_default() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.0),
        Longitude::from_degrees(-74.0),
        None,
    );
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let pts = chart_points(inst, &observer, None).expect("defined");
    // Ascendant matches the value derive_angles produces for the same inputs.
    let req = HouseRequest::new(inst, observer.clone(), HouseSystem::Placidus);
    let snap = calculate_houses(&req).expect("houses");
    assert!((pts.ascendant.degrees() - snap.angles.ascendant.degrees()).abs() < 1e-9);
    assert!((pts.midheaven.degrees() - snap.angles.midheaven.degrees()).abs() < 1e-9);
}

#[test]
fn house_snapshot_carries_asc_mc_consistent_with_angles() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    let observer = ObserverLocation::new(
        Latitude::from_degrees(48.85),
        Longitude::from_degrees(2.35),
        None,
    );
    let req = HouseRequest::new(
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        observer,
        HouseSystem::Placidus,
    );
    let snap = calculate_houses(&req).expect("houses");
    assert_eq!(snap.asc_mc.ascendant, snap.angles.ascendant);
    assert_eq!(snap.asc_mc.midheaven, snap.angles.midheaven);
    assert!((0.0..360.0).contains(&snap.asc_mc.vertex.degrees()));
}

#[test]
fn porphyry_fallback_snapshot_carries_consistent_asc_mc() {
    use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
    // Latitude 75° is beyond Placidus's polar bound, so with the SE fallback
    // policy calculate_houses takes the early-return Porphyry-fallback branch.
    let observer = ObserverLocation::new(
        Latitude::from_degrees(75.0),
        Longitude::from_degrees(10.0),
        None,
    );
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let req = HouseRequest::new(instant, observer.clone(), HouseSystem::Placidus)
        .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);

    let snap = calculate_houses(&req).expect("Porphyry fallback should produce a snapshot");

    // It really took the fallback: Porphyry yields 12 quadrant cusps.
    assert_eq!(snap.cusps.len(), 12);

    // The fallback site's asc_mc must equal an independent recomputation.
    let expected = asc_mc_from(
        local_sidereal_time(instant, observer.longitude).degrees(),
        observer.latitude.degrees(),
        snap.obliquity.degrees(),
    )
    .expect("asc_mc_from");
    assert_eq!(snap.asc_mc, expected);
}

// --- FU-9 Foundation: shared geometry primitives ---

#[test]
fn spherical_cotrans_matches_independent_x_axis_rotation() {
    // Independent reference (houses-reference.py `spherical_cotrans`): a pure
    // x-axis rotation of (lon,lat,r) -> Cartesian -> rotate by `angle` -> back.
    // Geometry avoids every degeneracy (no 0°/90° angle, non-unit radius) so
    // each `*`/`+` term is observable. Cross-validated to 1e-12 vs the crate.
    let mut coord = [40.0_f64, 25.0, 2.0];
    spherical_cotrans(&mut coord, 15.0);
    assert!(
        (coord[0] - 44.070_120_506_012).abs() < 1e-9,
        "lon' = {}",
        coord[0]
    );
    assert!(
        (coord[1] - 14.918_178_485_226).abs() < 1e-9,
        "lat' = {}",
        coord[1]
    );
    assert!(
        (coord[2] - 2.000_000_000_000).abs() < 1e-9,
        "r' = {}",
        coord[2]
    );
}

#[test]
fn asc2_matches_independent_swehouse_kernel() {
    // Independent reference (houses-reference.py `asc2`, swehouse.c Asc2) at
    // pole height 52°, obliquity sine/cosine. Four x values, one per asc1
    // quadrant; each takes the normal atan branch. Cross-validated to 1e-12.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 20.983_664_735_370),
        (300.0, 86.674_198_092_798),
    ];
    for (x, expected) in cases {
        let got = asc2(x, 52.0, sine, cose);
        assert!((got - expected).abs() < 1e-9, "asc2({x}) = {got}");
    }
}

#[test]
fn asc1_dispatches_each_quadrant_to_independent_reference() {
    // houses-reference.py `asc1` (swehouse.c Asc1): quadrant fold into Asc2.
    // 30° -> Q1, 120° -> Q2, 210° -> Q3, 300° -> Q4 exercise all four match
    // arms and the ±pole / (180-x)/(x-180)/(360-x) argument folding.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    let cases = [
        (30.0_f64, 60.273_411_210_075),
        (120.0, 138.177_359_444_927),
        (210.0, 200.983_664_735_370),
        (300.0, 266.674_198_092_798),
    ];
    for (x, expected) in cases {
        let got = asc1(x, 52.0, sine, cose).degrees();
        assert!((got - expected).abs() < 1e-9, "asc1({x}) = {got}");
    }
}

#[test]
fn asc_mc_from_pins_all_points_across_pole_and_flip_branches() {
    // Independent reference (houses-reference.py `asc_mc_from`, swehouse.c
    // swe_houses_armc). obl = 23.4366°. Three geometries cover:
    //   G1 lat>obl  -> f_pole = 90-lat, vertex flip inactive
    //   G2 0<lat<=obl -> flip branch active (vemc>0 path)
    //   G3 lat<0    -> f_pole = -90-lat branch
    // Every literal cross-validated to 1e-12 against the crate.
    let eps = 23.4366_f64;
    let check = |armc: f64, lat: f64, exp: [f64; 7]| {
        let p = asc_mc_from(armc, lat, eps).expect("finite");
        let got = [
            p.ascendant.degrees(),
            p.midheaven.degrees(),
            p.vertex.degrees(),
            p.equatorial_ascendant.degrees(),
            p.coascendant_koch.degrees(),
            p.coascendant_munkasey.degrees(),
            p.polar_ascendant.degrees(),
        ];
        for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
            assert!(
                (g - e).abs() < 1e-8,
                "armc={armc} lat={lat} point[{i}] = {g}, want {e}"
            );
        }
    };
    // G1: armc=45, lat=52 (> obl) — non-flip, f_pole = 38.
    check(
        45.0,
        52.0,
        [
            148.587_249_395_771,
            47.463_595_280_938,
            295.549_781_631_009,
            132.536_404_719_062,
            101.175_335_496_703,
            143.611_940_830_436,
            281.175_335_496_703,
        ],
    );
    // G2: armc=200, lat=10 (0<lat<=obl) — vertex flip active.
    check(
        200.0,
        10.0,
        [
            284.537_224_659_332,
            201.638_102_932_963,
            159.911_766_495_904,
            288.466_379_243_755,
            292.223_701_037_155,
            205.822_995_524_640,
            112.223_701_037_155,
        ],
    );
    // G3: armc=100, lat=-33 (< 0) — f_pole = -90-lat = -57.
    check(
        100.0,
        -33.0,
        [
            195.061_993_029_707,
            99.189_697_612_154,
            6.534_310_124_205,
            190.878_573_217_375,
            188.500_387_274_068,
            210.816_655_151_624,
            8.500_387_274_068,
        ],
    );
}

#[test]
fn interpolate_longitude_wraps_and_scales() {
    // span = (end-start).rem_euclid(360) = 30; start + span*frac = 357.5.
    // start=350,end=20,frac=0.25 keeps every mutant (- -> +//, + -> *//-,
    // * -> +//) observably wrong. Hand-computed.
    let got = interpolate_longitude(
        Longitude::from_degrees(350.0),
        Longitude::from_degrees(20.0),
        0.25,
    )
    .degrees();
    assert!((got - 357.5).abs() < 1e-9, "interp = {got}");
}

#[test]
fn signed_longitude_difference_both_branches() {
    // delta<180 branch: (10-350).rem_euclid(360)=20 -> 20.
    assert!((signed_longitude_difference(10.0, 350.0) - 20.0).abs() < 1e-9);
    // delta>=180 branch: (200-10).rem_euclid(360)=190 -> 190-360 = -170.
    assert!((signed_longitude_difference(200.0, 10.0) + 170.0).abs() < 1e-9);
}

#[test]
fn right_ascension_from_ecliptic_longitude_matches_reference() {
    // atan2(sinλ·cosε, cosλ) at λ=60°, ε=23.4366°. Independent reference.
    let eps = 23.4366_f64;
    let got =
        right_ascension_from_ecliptic_longitude(Longitude::from_degrees(60.0), eps.to_radians());
    assert!((got - 57.819_266_732_173).abs() < 1e-9, "ra = {got}");
}

#[test]
fn longitude_in_arc_handles_wraparound() {
    // Wraparound arc [350,10): membership is `lon>=350 || lon<10`. A point at
    // 355 is in via the first disjunct only, so || -> && flips it to false.
    assert!(longitude_in_arc(355.0, 350.0, 10.0), "355 in [350,10)");
    assert!(longitude_in_arc(5.0, 350.0, 10.0), "5 in [350,10)");
    // Non-wrap arc [10,20): 15 in, 25 out.
    assert!(longitude_in_arc(15.0, 10.0, 20.0));
    assert!(!longitude_in_arc(25.0, 10.0, 20.0));
}

#[test]
fn longitude_opposite_is_the_antipode() {
    // `longitude_opposite(x) = from_degrees(x + 180)`. NOTE: the cargo-mutants
    // survivor `+ -> -` here is a DOCUMENTED EQUIVALENT MUTANT, not a coverage
    // hole: from_degrees normalizes mod 360 and x+180 ≡ x-180 (mod 360) for all
    // x, so no reachable input distinguishes `+` from `-`. It is left visible
    // (no #[mutants::skip]) per FU-9 posture. This test pins the antipode
    // intent; it cannot and does not claim to kill the equivalent mutant.
    assert!((longitude_opposite(Longitude::from_degrees(50.0)).degrees() - 230.0).abs() < 1e-9);
    assert!((longitude_opposite(Longitude::from_degrees(300.0)).degrees() - 120.0).abs() < 1e-9);
}

#[test]
fn asc2_degenerate_sinx_zero_branch_pins() {
    // The four quadrant inputs in `asc2_matches_independent_swehouse_kernel`
    // all take the normal `atan(sinx/value)` path, leaving asc2's degenerate
    // guard branch (`sinx.abs() < 1e-12`) uncovered. x on the sinx~0 axis
    // reaches it. Independent reference (houses-reference.py `asc2`):
    //   asc2(0)   -> value>0, so the branch returns the +1e-12 sentinel exactly.
    //   asc2(180) -> value<0, so it returns -1e-12, folded by `+180` to ~180.
    // Kills the sinx~0 guard/sign mutants: 1811 `< -> ==`, 1812 `< -> ==`/`>`,
    // and `delete -` on the -1e-12 sentinel (1813).
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());
    // asc2(0) is the +1e-12 sentinel exactly; tolerance below the sentinel so a
    // mutant that skips the branch (returns 0.0) or flips the sign (returns
    // ~180) fails.
    let a0 = asc2(0.0, 52.0, sine, cose);
    assert!((a0 - 1e-12).abs() < 1e-13, "asc2(0) = {a0}");
    // asc2(180) folds -1e-12 to ~180; mutants that skip the branch return ~1e-12.
    let a180 = asc2(180.0, 52.0, sine, cose);
    assert!(
        (a180 - 179.999_999_999_999).abs() < 1e-9,
        "asc2(180) = {a180}"
    );
}

#[test]
fn asc_mc_from_vertex_flip_actually_fires() {
    // G2 in `asc_mc_from_pins_all_points_across_pole_and_flip_branches` enters
    // the `|lat| <= obl` flip block but vemc <= 0, so the vertex never rotates
    // and the flip guards/arithmetic stay uncovered. This geometry
    // (armc=15, lat=5) makes vemc > 0 so the flip FIRES, rotating the vertex by
    // 180. Independent reference (houses-reference.py `asc_mc_from`, G4). Kills
    // the flip-trigger mutants: 202 `<= -> >`, 207 `> -> ==`, 208 `+ -> *`.
    let eps = 23.4366_f64;
    let p = asc_mc_from(15.0, 5.0, eps).expect("finite");
    let got = [
        p.ascendant.degrees(),
        p.midheaven.degrees(),
        p.vertex.degrees(),
        p.equatorial_ascendant.degrees(),
        p.coascendant_koch.degrees(),
        p.coascendant_munkasey.degrees(),
        p.polar_ascendant.degrees(),
    ];
    let exp = [
        105.741_462_639_643,
        16.280_047_054_689,
        12.635_804_712_026,
        103.811_888_609_499,
        101.849_837_950_278,
        168.584_056_229_968,
        281.849_837_950_278,
    ];
    for (i, (g, e)) in got.iter().zip(exp.iter()).enumerate() {
        assert!((g - e).abs() < 1e-8, "point[{i}] = {g}, want {e}");
    }
}

#[test]
fn asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants() {
    // Final-review fix (2026-07-23): the original Foundation PR's equivalence
    // sweep sampled lat in [-66,-50,-33,-20,-10,-5,5,10,20,23,33,50,66] and
    // never tried lat = 0. At lat = 0 the pole height `f_pole` used by the
    // vertex/coascendant_munkasey branch is exactly +-90 deg, where `tan` is
    // NOT 180-periodic in f64 (`tan(90 deg) = +1.633...e16` vs
    // `tan(-90 deg) = -1.633...e16`, not equal). That makes several mutants
    // that were documented as "180-periodicity of tan" equivalents actually
    // observable at the equator. Independent reference (houses-reference.py
    // `asc_mc_from`), cross-validated to 1e-12.
    let eps = 23.4366_f64;

    // C1: kills mod.rs `lat_deg >= 0.0 -> < 0.0` (the f_pole branch select).
    // At lat=0.0 HEAD takes the `>=` arm (f_pole = 90-lat = 90); the mutant
    // takes the `<` arm's `else` (f_pole = -90-lat = -90), giving tan(pole)
    // the opposite huge-magnitude sign and moving coascendant_munkasey from
    // 180.0 to ~6.2e-15 -- a ~180 degree miss.
    let c1 = asc_mc_from(45.0, 0.0, eps).expect("finite");
    let got = c1.coascendant_munkasey.degrees();
    assert!(
        signed_longitude_difference(got, 180.0).abs() < 1e-9,
        "coascendant_munkasey = {got}, want ~180.0"
    );

    // C2: kills mod.rs `vemc > 180.0 -> >= 180.0`. At armc=0, lat=0 the raw
    // vemc is exactly 180.0. HEAD's strict `>` leaves it unfolded and the
    // vertex then flips (vemc > 0.0) to 0.0; the mutant's `>=` folds it to
    // -180.0 first, so the flip guard sees a negative vemc and never fires,
    // leaving the vertex at 180.0 -- a ~180 degree miss.
    let c2 = asc_mc_from(0.0, 0.0, eps).expect("finite");
    let got = c2.vertex.degrees();
    assert!(
        signed_longitude_difference(got, 0.0).abs() < 1e-9,
        "vertex = {got}, want ~0.0"
    );

    // C3: kills mod.rs `vemc > 0.0 -> >= 0.0`. At armc=180, lat=0 the raw vemc
    // is exactly 0.0. HEAD's strict `>` does not flip, leaving vertex at
    // 180.0; the mutant's `>=` flips it to 0.0 -- a ~180 degree miss.
    let c3 = asc_mc_from(180.0, 0.0, eps).expect("finite");
    let got = c3.vertex.degrees();
    assert!(
        signed_longitude_difference(got, 180.0).abs() < 1e-9,
        "vertex = {got}, want ~180.0"
    );

    // I1: kills mod.rs `delete -` turning `-90.0 - lat` into `90.0 - lat` in
    // the southern (`lat < 0`) branch. lat = -1e-16 is a hair below zero, so
    // it takes that branch: HEAD computes f_pole = -90.0 - (-1e-16) ~ -90,
    // giving coascendant_munkasey ~6.2e-15; the mutant computes
    // f_pole = 90.0 - (-1e-16) ~ +90, giving ~180.0 -- again a ~180 degree
    // miss, mirroring C1 but through the southern branch.
    let i1 = asc_mc_from(45.0, -1e-16, eps).expect("finite");
    let got = i1.coascendant_munkasey.degrees();
    assert!(
        signed_longitude_difference(got, 0.0).abs() < 1e-9,
        "coascendant_munkasey = {got}, want ~0.0"
    );
}

#[test]
fn asc2_value_zero_guard_reachable_at_exact_equality() {
    // Final-review fix (2026-07-23): the old equivalence writeup claimed
    // "no representable input hits equality" for asc2's `1.0e-12` guard
    // thresholds. That is wrong: the guard at mod.rs `value.abs() < 1.0e-12`
    // ASSIGNS `value = 0.0`, and pole = 90 - EPS = 66.5634 (the f_pole of an
    // observer at latitude == obliquity) drives `value` to exactly 0.0 at
    // x = 0, making the downstream `value < 0.0` comparison reachable at
    // equality too. Independent reference (houses-reference.py `asc2`),
    // cross-validated to 1e-12: both calls below equal 1e-12 exactly.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());

    // Kills mod.rs `value < 0.0 -> <= 0.0`: value is exactly 0.0 here, so
    // HEAD's strict `<` takes the `else` arm (+1e-12 sentinel); the mutant's
    // `<=` takes the `if` arm (-1e-12), which the final `< 0.0 -> += 180.0`
    // fold turns into ~180.0 -- a ~180 degree miss.
    let at_pole = asc2(0.0, 66.5634, sine, cose);
    assert!(
        (at_pole - 1e-12).abs() < 1e-13,
        "asc2(0, 66.5634) = {at_pole}"
    );

    // Kills mod.rs `value.abs() < 1.0e-12 -> == 1.0e-12`: at this neighboring
    // pole, the raw `value` is a tiny unrounded epsilon, not exactly 1e-12,
    // so the mutant guard never fires and value is never snapped to 0.0. The
    // sign of that unrounded epsilon leaks through to the same ~180 degree
    // miss as above.
    let near_pole = asc2(0.0, 66.5634000000001, sine, cose);
    assert!(
        (near_pole - 1e-12).abs() < 1e-13,
        "asc2(0, 66.5634000000001) = {near_pole}"
    );
}

#[test]
fn asc_geometry_equivalent_mutants_are_documented() {
    // The measured cargo-mutants residual for the Foundation functions is a
    // set of DOCUMENTED EQUIVALENT MUTANTS, left visible (no
    // `#[mutants::skip]`) per the FU-9 posture.
    //
    // CORRECTION (final-review fix, 2026-07-23): an earlier version of this
    // test claimed six of these were equivalent by "180-periodicity of tan in
    // the pole height" and "unreachable exact-equality boundaries". Both
    // claims were wrong — the equivalence sweep never sampled lat = 0 (pole
    // height exactly +-90, where f64 `tan` is NOT 180-periodic) and never
    // noticed that asc2's `value.abs() < 1e-12` guard EXPLICITLY assigns
    // `value = 0.0`, making the downstream `value < 0.0` reachable at
    // equality. Those six mutants (mod.rs 192, 195, 204, 207, 1807, 1812) are
    // now KILLED by `asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants`
    // and `asc2_value_zero_guard_reachable_at_exact_equality` above, and are
    // no longer documented as equivalent here. The remaining residual splits
    // into two honestly-distinguished categories:
    //
    // All 13 measured survivors are enumerated below, in three honestly
    // distinguished categories.
    //
    // (A) STRUCTURALLY UNREACHABLE / BIT-IDENTICAL — no representable input can
    // distinguish the operators, independent of tolerance. [5 mutants]
    //   * asc2 1818 `< -> ==`, `< -> >`, `< -> <=`, and 1819 `delete -`: this
    //     `else if value == 0.0` arm is reached ONLY when `sinx.abs() >= 1e-12`
    //     (the 1811 guard consumed the small-sinx case), so `sinx` is never 0
    //     here — the arm is instead reached because the 1807 guard ASSIGNED
    //     `value = 0.0`. Whichever way these mutants steer the `sinx < 0.0`
    //     test, the result is `-90.0` or `+90.0`, and the 1826 fold maps
    //     `-90.0 + 180.0` to exactly `90.0`. So every variant returns bit-
    //     identical `90.0`. (NOTE: an earlier revision of this comment stated
    //     the inverted premise "the 1811 guard already forced sinx == 0" — the
    //     conclusion held but the reason did not.)
    //   * asc2 1826 `< -> <=` (final `longitude < 0.0` fold): `longitude ==
    //     0.0` is unreachable from all three producing branches.
    //
    // (B) BELOW THE 1e-9 PARITY TOLERANCE, BUT MEASURABLY NOT BIT-IDENTICAL
    // (I2 correction — do not claim "differ by exactly 360" or "no
    // representable input hits equality" here; state the measured magnitude).
    // Each magnitude is a sweep MAXIMUM, not a proven bound. [6 mutants]
    //   * asc1 `delete match arm 3` (1799): arm 3 and the `_` arm are
    //     ALGEBRAICALLY identical, but not f64-identical — `(180-u)*pi/180`
    //     and `pi - u*pi/180` differ in the last bits, so the trig arguments
    //     differ. Measured max diff ~5.68e-14 (at x1 ~ 180.315, pole -52).
    //     This was previously mis-filed under (A) as "bit-identical".
    //   * asc1 arm-3 `x1 - 180 -> x1 + 180` (360-periodicity of asc2 in x):
    //     measured max diff ~2.56e-13 over a sweep, not exactly 0.
    //   * asc_mc `armc - 180 -> + 180` at 201 and 215: measured max circular
    //     diff ~1.31e-12 over a lat/armc sweep, not exactly 0.
    //   * asc_mc vertex flip `+180 -> -180` (208) and `longitude_opposite`'s
    //     `+ -> -` (1833): measured max circular diff ~5.68e-14 over a sweep,
    //     not exactly 0.
    //
    // (C) asc2's REMAINING 1e-12 GUARD THRESHOLDS — below tolerance under
    // generic inputs; explicitly NOT a strict-unreachability claim. [2 mutants]
    //   * asc2 1807 `< -> <=` (`value.abs() < 1e-12`) and 1811 `< -> <=`
    //     (`sinx.abs() < 1e-12`): under generic (non-adversarial) inputs the
    //     reachable boundary difference is ~1.4e-10 — below the 1e-9 parity
    //     tolerance. An adversarial input sitting exactly on the threshold
    //     could exceed it, so these two are best read as "not proven
    //     equivalent, not currently killable", and are flagged as such in the
    //     docs/follow-ups.md note rather than asserted to be unreachable.
    //
    // 5 + 6 + 2 = 13, matching the measured `mutants.out/missed.txt`.
    //
    // All measured magnitudes above are independently confirmed by
    // `houses-reference.py`'s sanity-check sweeps, never by running the
    // crate. The identities below are asserted so the reasoning itself is
    // regression-tested.
    let eps = 23.4366_f64;
    let (sine, cose) = (eps.to_radians().sin(), eps.to_radians().cos());

    // 360-periodicity of asc2 in x (asc1 arm-3 `x1-180 -> x1+180`): below
    // tolerance, not bit-identical (see comment above).
    assert!(
        (asc2(30.0 - 180.0, -52.0, sine, cose) - asc2(30.0 + 180.0, -52.0, sine, cose)).abs()
            < 1e-9
    );

    // 180-periodicity of tan(pole) holds well away from the poles (asc_mc
    // pole-branch 90-lat vs -90-lat, both far from +-90):
    assert!((asc2(70.0, 38.0, sine, cose) - asc2(70.0, 38.0 - 180.0, sine, cose)).abs() < 1e-9);
    // ...but the identity FAILS exactly at the degenerate pole (lat = 0,
    // pole = +-90): tan(90 deg) and tan(-90 deg) are not equal in f64, so the
    // difference blows well past the parity tolerance instead of vanishing.
    // This is precisely the case
    // `asc_mc_from_equator_pole_asymmetry_kills_tan_periodicity_mutants`
    // exercises to kill mutants 192/195/204/207 above — it is not
    // re-documented as equivalent here.
    assert!(
        (asc2(0.0, 90.0, sine, cose) - asc2(0.0, -90.0, sine, cose)).abs() > 1.0,
        "the tan(pole) periodicity identity must FAIL at pole = +-90"
    );

    // Vertex-fold `+180 -> -180`: exact for these particular literals (pure
    // rem_euclid arithmetic on 200.0, no trig involved)...
    assert_eq!(
        (200.0_f64 + 180.0).rem_euclid(360.0),
        (200.0_f64 - 180.0).rem_euclid(360.0)
    );
    // ...but not universally bit-identical once trig-derived vertex values are
    // involved: x = 332.3 is a measured near-worst-case (~5.68e-14 diff),
    // still comfortably below the 1e-9 tolerance.
    assert!(
        ((332.3_f64 + 180.0).rem_euclid(360.0) - (332.3_f64 - 180.0).rem_euclid(360.0)).abs()
            < 1e-9
    );

    // asc_mc `armc - 180 -> + 180` (201/215): below tolerance, not exact.
    // armc=88.9, lat=66.0 is a measured near-worst-case (~1.3e-12 diff).
    let obl = eps.to_radians();
    let a = ascendant_for(88.9 - 180.0, 66.0, obl).degrees();
    let b = ascendant_for(88.9 + 180.0, 66.0, obl).degrees();
    assert!(
        signed_longitude_difference(a, b).abs() < 1e-9,
        "a={a} b={b}"
    );
}
