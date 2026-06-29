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
         delta={delta_arcsec:+.1}\"  (expect ~20-25\" for aberration+nutation)"
    );
    // The delta should be in the ~20–25″ range (aberration ~20.5″ + nutation ±17″ net ~20–25″).
    // We just assert the sign/magnitude is non-trivial to confirm the correction is active.
    assert!(
        delta_arcsec.abs() > 5.0 && delta_arcsec.abs() < 100.0,
        "unexpected delta {delta_arcsec:.1}\" — correction may be inactive or wrong"
    );
}
