//! Time-scale handling for the sidereal-time base of every cusp and angle
//! (issue #56). Sidereal time is a function of UT1, so a TT/TDB-tagged
//! request instant must be converted through ΔT before ARMC is taken, while
//! obliquity and nutation keep using the dynamical day.

use crate::systems::*;
use pleiades_types::{Instant, JulianDay, Latitude, TimeScale};

/// Chennai, 2026-03-21 05:00 UTC expressed as TT (the fixture from #56).
const CHENNAI_JD_TT: f64 = 2_461_120.709_130;

fn chennai() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(13.0827),
        Longitude::from_degrees(80.2707),
        None,
    )
}

fn ut1_from_tt(jd_tt: f64) -> f64 {
    let (delta_t_seconds, _) = pleiades_time::deltat::delta_t(jd_tt).expect("ΔT table available");
    jd_tt - delta_t_seconds / 86_400.0
}

fn signed_wrap(a: f64, b: f64) -> f64 {
    let w = (a - b).rem_euclid(360.0);
    if w > 180.0 {
        w - 360.0
    } else {
        w
    }
}

#[test]
fn tt_request_yields_the_same_cusps_as_the_explicit_ut1_instant() {
    let tt = HouseRequest::new(
        Instant::new(JulianDay::from_days(CHENNAI_JD_TT), TimeScale::Tt),
        chennai(),
        HouseSystem::Placidus,
    );
    let ut1 = HouseRequest::new(
        Instant::new(
            JulianDay::from_days(ut1_from_tt(CHENNAI_JD_TT)),
            TimeScale::Ut1,
        ),
        chennai(),
        HouseSystem::Placidus,
    );
    let from_tt = calculate_houses(&tt).expect("tt houses");
    let from_ut1 = calculate_houses(&ut1).expect("ut1 houses");

    // ARMC is pure sidereal time, so it must agree exactly up to JD rounding.
    assert!(
        signed_wrap(
            from_tt.asc_mc.armc.degrees(),
            from_ut1.asc_mc.armc.degrees()
        )
        .abs()
            < 1e-7,
        "armc tt={} ut1={}",
        from_tt.asc_mc.armc.degrees(),
        from_ut1.asc_mc.armc.degrees()
    );
    // Cusps differ only through the obliquity, which moves ~1e-8° over ΔT.
    for (i, (a, b)) in from_tt.cusps.iter().zip(&from_ut1.cusps).enumerate() {
        assert!(
            signed_wrap(a.degrees(), b.degrees()).abs() < 1e-6,
            "cusp {} tt={} ut1={}",
            i + 1,
            a.degrees(),
            b.degrees()
        );
    }
}

#[test]
fn tt_request_no_longer_reproduces_the_tt_as_ut_ascendant() {
    // 71.267308° is what pleiades 0.5.0 returned for this TT instant: the
    // ascendant Swiss Ephemeris gives when the TT day is *fed as UT*.
    // The correct value (SE at jd_ut) is 70.988218°, ≈0.28° earlier.
    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(CHENNAI_JD_TT), TimeScale::Tt),
        chennai(),
        HouseSystem::WholeSign,
    );
    let asc = calculate_houses(&request)
        .expect("whole-sign houses")
        .angles
        .ascendant
        .degrees();
    assert!(
        signed_wrap(asc, 71.267_308).abs() > 0.2,
        "ascendant {asc} still evaluated at the TT day"
    );
    // The 2026 residual against SE is bounded by the ΔT model, not the
    // house math: pleiades uses the leap-second bound 69.184 s here while
    // SE 2.10.03 interpolates its own table to ≈68.88 s, a 0.3 s ≈ 0.0014°
    // difference in ascendant (FU-11 item 2; measured 0.001408°). Before that
    // bound existed the extrapolated polynomial overshot by ~6 s ≈ 0.026°,
    // which 0.005° rejects.
    assert!(
        signed_wrap(asc, 70.988_218).abs() < 0.005,
        "ascendant {asc} vs SE(jd_ut) 70.988218"
    );
}

#[test]
fn tt_tagged_corpus_row_shifted_by_delta_t_reproduces_swiss_ephemeris_cusps() {
    // Corpus row c4_lat40_e2 / Placidus (SE 2.10.03): jd_ut 2433283, lat 40,
    // lon 30, at an epoch inside the *observed* ΔT table. Re-expressing the
    // UT day as TT must give the same cusps back within the corpus ceiling.
    const JD_UT: f64 = 2_433_283.0;
    const EXPECTED: [f64; 12] = [
        60.830331, 85.166046, 105.987810, 128.147116, 156.178607, 195.112631, 240.830331,
        265.166046, 285.987810, 308.147116, 336.178607, 15.112631,
    ];
    let (delta_t_seconds, _) = pleiades_time::deltat::delta_t(JD_UT).expect("ΔT table");
    let jd_tt = JD_UT + delta_t_seconds / 86_400.0;
    let request = HouseRequest::new(
        Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt),
        ObserverLocation::new(
            Latitude::from_degrees(40.0),
            Longitude::from_degrees(30.0),
            Some(0.0),
        ),
        HouseSystem::Placidus,
    );
    let snapshot = calculate_houses(&request).expect("placidus houses");
    for (i, (actual, expected)) in snapshot.cusps.iter().zip(EXPECTED).enumerate() {
        let arcsec = signed_wrap(actual.degrees(), expected).abs() * 3600.0;
        assert!(
            arcsec < 1.0,
            "cusp {} = {:.6} vs SE {expected:.6} ({arcsec:.3}″)",
            i + 1,
            actual.degrees()
        );
    }
}

#[test]
fn ut1_and_utc_tagged_requests_use_the_day_as_supplied() {
    let observer = chennai();
    let jd = ut1_from_tt(CHENNAI_JD_TT);
    let armc_for = |scale: TimeScale| {
        calculate_houses(&HouseRequest::new(
            Instant::new(JulianDay::from_days(jd), scale),
            observer.clone(),
            HouseSystem::Equal,
        ))
        .expect("equal houses")
        .asc_mc
        .armc
        .degrees()
    };
    assert_eq!(armc_for(TimeScale::Ut1), armc_for(TimeScale::Utc));
}

#[test]
fn chart_points_take_armc_at_ut1_for_a_tt_instant() {
    let tt = Instant::new(JulianDay::from_days(CHENNAI_JD_TT), TimeScale::Tt);
    let ut1 = Instant::new(
        JulianDay::from_days(ut1_from_tt(CHENNAI_JD_TT)),
        TimeScale::Ut1,
    );
    let from_tt = chart_points(tt, &chennai(), None).expect("chart points at tt");
    let from_ut1 = chart_points(ut1, &chennai(), None).expect("chart points at ut1");
    assert!(
        signed_wrap(from_tt.armc.degrees(), from_ut1.armc.degrees()).abs() < 1e-7,
        "armc tt={} ut1={}",
        from_tt.armc.degrees(),
        from_ut1.armc.degrees()
    );
}
