use pleiades_backend::{
    Apparentness, CelestialBody, CoordinateFrame, EphemerisBackend, EphemerisRequest,
};
use pleiades_data::packaged_backend;
use pleiades_eclipse::{
    EclipseEngine, EclipseFilter, EclipseKind, EclipseType, SolarEclipseType, WINDOW_END_JD,
};
use pleiades_types::{Instant, JulianDay, TimeScale};

fn at(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

#[test]
fn finds_the_1999_august_11_total_solar_eclipse() {
    let engine = EclipseEngine::new(packaged_backend());
    // Search a tight window around 1999-08-11.
    let eclipses = engine
        .eclipses_in_range(at(2_451_400.0), at(2_451_410.0), EclipseFilter::SolarOnly)
        .unwrap();
    let e = eclipses
        .iter()
        .find(|e| e.kind == EclipseKind::Solar)
        .expect("a solar eclipse in this window");
    assert_eq!(e.eclipse_type, EclipseType::Solar(SolarEclipseType::Total));
    // Greatest eclipse was 1999-08-11 ~11:03 UT → JD ≈ 2451401.961, within 1 min.
    // Canon (NASA Five Millennium Canon): greatest eclipse 11:03:04 UT,
    // TT greatest = 11:04:10 → JD(TT) ≈ 2451401.9612.
    let jd = e.greatest_eclipse.julian_day.days();
    let canon_jd = 2_451_401.961_f64;
    let delta_s = (jd - canon_jd).abs() * 86_400.0;
    eprintln!(
        "1999-08-11 eclipse: computed JD={jd}, canon JD={canon_jd}, delta={delta_s:.1}s, type={:?}",
        e.eclipse_type
    );
    assert!(
        delta_s < 60.0,
        "jd was {jd}, delta {delta_s:.1}s exceeds 60s limit"
    );
}

/// Boundary tests: calling `next_eclipse`, `previous_eclipse`, and
/// `eclipses_in_range` with end = WINDOW_END_JD must return Ok, not a backend
/// OutOfRange error. (The syzygy scanner probes one STEP_DAYS past its end;
/// without the clamp in `eclipses_in_range` this would query past the data bound.)
#[test]
fn next_eclipse_near_window_end_does_not_error() {
    let engine = EclipseEngine::new(packaged_backend());
    // Start ~1 year before the end so at least a few syzygies are scanned.
    let start = at(WINDOW_END_JD - 365.0);
    let result = engine.next_eclipse(start, EclipseFilter::All);
    assert!(
        result.is_ok(),
        "next_eclipse near WINDOW_END_JD should not error: {:?}",
        result
    );
}

#[test]
fn previous_eclipse_at_window_end_does_not_error() {
    let engine = EclipseEngine::new(packaged_backend());
    // before = WINDOW_END_JD is the exact failing case before the fix.
    let before = at(WINDOW_END_JD);
    let result = engine.previous_eclipse(before, EclipseFilter::All);
    assert!(
        result.is_ok(),
        "previous_eclipse(WINDOW_END_JD) should not error: {:?}",
        result
    );
    // There must be at least one eclipse before the window end.
    assert!(
        result.unwrap().is_some(),
        "previous_eclipse(WINDOW_END_JD) should find at least one eclipse"
    );
}

#[test]
fn eclipses_in_range_ending_at_window_end_does_not_error() {
    let engine = EclipseEngine::new(packaged_backend());
    // Narrow window ending exactly at WINDOW_END_JD — the other broken path.
    let start = at(WINDOW_END_JD - 365.0);
    let end = at(WINDOW_END_JD);
    let result = engine.eclipses_in_range(start, end, EclipseFilter::All);
    assert!(
        result.is_ok(),
        "eclipses_in_range ending at WINDOW_END_JD should not error: {:?}",
        result
    );
}

/// Evidence test: print mean vs apparent eclipsed_longitude delta.
#[test]
fn apparent_vs_mean_eclipsed_longitude_delta() {
    let backend = packaged_backend();
    let engine = EclipseEngine::new(backend.clone());

    let eclipses = engine
        .eclipses_in_range(at(2_451_400.0), at(2_451_410.0), EclipseFilter::SolarOnly)
        .unwrap();
    let e = eclipses
        .iter()
        .find(|e| e.kind == EclipseKind::Solar)
        .unwrap();

    let jd = e.greatest_eclipse.julian_day.days();
    let apparent_lon = e.eclipsed_longitude.degrees();

    // Compute mean Sun longitude directly from the backend (before apparent correction).
    let req = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: pleiades_types::ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let result = backend.position(&req).unwrap();
    let mean_lon = result.ecliptic.unwrap().longitude.degrees();

    let delta_arcsec = (apparent_lon - mean_lon) * 3600.0;
    eprintln!(
        "1999 eclipse mean_lon={mean_lon:.6}°  apparent_lon={apparent_lon:.6}°  \
         delta={delta_arcsec:+.1}\"  (expect ~-50\": precession + aberration + nutation)"
    );
    // mean_lon is the backend's mean J2000 longitude; apparent_lon is apparent-of-date.
    // The delta is therefore precession(J2000->date) + annual aberration + nutation in
    // longitude, which for 1999 sums to ~-51″. (Aberration is applied ONCE — the earlier
    // ~-71″ reflected a since-removed light-time/aberration double-count.) Assert a loose
    // band to confirm the correction is active and of the right magnitude.
    assert!(
        delta_arcsec.abs() > 30.0 && delta_arcsec.abs() < 90.0,
        "unexpected delta {delta_arcsec:.1}\" — correction may be inactive or wrong"
    );
}

const ONE_SECOND_DAYS: f64 = 1.0 / 86_400.0;

/// Regression for issue #121: `eclipses_in_range` must select on greatest
/// eclipse, not on the syzygy instant. Greatest eclipse and syzygy differ by
/// minutes, so a range whose bound falls between the two used to lose (or
/// wrongly keep) the eclipse depending on which side the syzygy lay.
///
/// With `g` the greatest-eclipse instant found from a wide search, each range
/// that contains `g` at a bound must return the eclipse, and each range that
/// stops one second short of `g` must not.
fn assert_range_selects_on_greatest_eclipse(approx_jd: f64, kind: EclipseKind) {
    let engine = EclipseEngine::new(packaged_backend());
    let filter = match kind {
        EclipseKind::Solar => EclipseFilter::SolarOnly,
        EclipseKind::Lunar => EclipseFilter::LunarOnly,
    };
    let wide = engine
        .eclipses_in_range(at(approx_jd - 2.0), at(approx_jd + 2.0), filter)
        .unwrap();
    assert_eq!(
        wide.len(),
        1,
        "expected exactly one {kind:?} eclipse near JD {approx_jd}"
    );
    let g = wide[0].greatest_eclipse.julian_day.days();
    assert!(
        (g - approx_jd).abs() < 0.01,
        "greatest eclipse {g} is not near the expected JD {approx_jd}"
    );

    let found = |start: f64, end: f64| -> Vec<f64> {
        engine
            .eclipses_in_range(at(start), at(end), filter)
            .unwrap()
            .into_iter()
            .map(|e| e.greatest_eclipse.julian_day.days())
            .collect()
    };

    assert_eq!(found(g - 1.0, g), vec![g], "[g - 1 d, g] must return it");
    assert_eq!(found(g, g + 1.0), vec![g], "[g, g + 1 d] must return it");
    assert!(
        found(g - 1.0, g - ONE_SECOND_DAYS).is_empty(),
        "[g - 1 d, g - 1 s] must not return it"
    );
    assert!(
        found(g + ONE_SECOND_DAYS, g + 1.0).is_empty(),
        "[g + 1 s, g + 1 d] must not return it"
    );
}

#[test]
fn range_selects_solar_eclipse_by_greatest_eclipse() {
    // 2024-04-08 total solar eclipse; its syzygy falls ~211 s after greatest eclipse.
    assert_range_selects_on_greatest_eclipse(2_460_409.262_8, EclipseKind::Solar);
}

#[test]
fn range_selects_lunar_eclipse_by_greatest_eclipse() {
    // 2025-03-14 total lunar eclipse; its syzygy falls before greatest eclipse.
    assert_range_selects_on_greatest_eclipse(2_460_748.791_6, EclipseKind::Lunar);
}
