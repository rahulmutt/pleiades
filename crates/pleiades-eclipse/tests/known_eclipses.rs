use pleiades_data::packaged_backend;
use pleiades_eclipse::{EclipseEngine, EclipseFilter, EclipseKind, EclipseType, SolarEclipseType};
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
