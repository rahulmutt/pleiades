//! A placement's house is the house of the longitude it reports (issue #182).
//!
//! The house used to be assigned from the backend's mean J2000 longitude,
//! before the apparent, topocentric and sidereal steps moved the placement,
//! and was never recomputed, although the sign was. A body within the
//! reduction's shift of a cusp (mostly the precession since J2000, about
//! 0.37° in 2026) was then placed one house off.

use pleiades_backend::{Apparentness, CompositeBackend, ZodiacMode};
use pleiades_elp::ElpBackend;
use pleiades_houses::house_for_longitude;
use pleiades_types::{
    Ayanamsa, CelestialBody, HouseSystem, Instant, JulianDay, Latitude, Longitude,
    ObserverLocation, TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

fn bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
    ]
}

fn observer(latitude_deg: f64) -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(latitude_deg),
        Longitude::from_degrees(80.2707),
        None,
    )
}

/// Charts over 1968–2060 at three latitudes, each built by `configure`.
fn sweep(configure: impl Fn(ChartRequest) -> ChartRequest) -> Vec<ChartSnapshot> {
    let engine = ChartEngine::new(CompositeBackend::new(
        Vsop87Backend::new(),
        ElpBackend::new(),
    ));
    let mut charts = Vec::new();
    for step in 0..60 {
        let jd_tt = 2_440_000.5 + 563.3 * f64::from(step);
        for latitude_deg in [-45.0, 13.0827, 51.5] {
            let request =
                ChartRequest::new(Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt))
                    .with_observer(observer(latitude_deg))
                    .with_house_system(HouseSystem::Placidus)
                    .with_bodies(bodies());
            charts.push(engine.chart(&configure(request)).expect("chart succeeds"));
        }
    }
    charts
}

/// Asserts every placement's house is the house its reported longitude falls
/// in, and returns how many placements were checked.
fn assert_houses_match_reported_longitudes(charts: &[ChartSnapshot]) -> usize {
    let mut checked = 0;
    for chart in charts {
        let cusps = &chart.houses.as_ref().expect("houses are computed").cusps;
        for placement in &chart.placements {
            let longitude = placement
                .position
                .ecliptic
                .expect("ecliptic coordinates")
                .longitude;
            assert_eq!(
                placement.house,
                Some(house_for_longitude(longitude, cusps)),
                "{:?} at {} (JD {})",
                placement.body,
                longitude,
                chart.instant.julian_day
            );
            checked += 1;
        }
    }
    checked
}

#[test]
fn apparent_tropical_houses_follow_the_reported_longitude() {
    let charts = sweep(|request| request);
    assert!(charts
        .iter()
        .all(|chart| chart.apparentness_applied() == Apparentness::Apparent));
    assert_eq!(assert_houses_match_reported_longitudes(&charts), 1260);
}

#[test]
fn apparent_sidereal_houses_follow_the_reported_longitude() {
    let charts = sweep(|request| {
        request.with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        })
    });
    assert_eq!(assert_houses_match_reported_longitudes(&charts), 1260);
}

#[test]
fn topocentric_houses_follow_the_reported_longitude() {
    let charts = sweep(|request| request.with_topocentric(true));
    assert!(charts.iter().all(|chart| chart
        .placements
        .iter()
        .all(|placement| placement.topocentric.is_some())));
    assert_eq!(assert_houses_match_reported_longitudes(&charts), 1260);
}

#[test]
fn mean_houses_follow_the_reported_longitude() {
    let charts = sweep(|request| request.with_apparentness(Apparentness::Mean));
    assert_eq!(assert_houses_match_reported_longitudes(&charts), 1260);
}
