use pleiades_data::packaged_backend;
use pleiades_events::{CrossingEngine, CrossingFrame};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

#[test]
fn heliocentric_jupiter_crossing_is_found() {
    let engine = CrossingEngine::new(packaged_backend());
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let end = Instant::new(JulianDay::from_days(2_451_545.0 + 4400.0), TimeScale::Tdb); // ~1 Jupiter orbit
    let out = engine
        .longitude_crossings_in_range(
            CelestialBody::Jupiter,
            Longitude::from_degrees(0.0),
            CrossingFrame::Heliocentric,
            start,
            end,
        )
        .expect("heliocentric crossing search");
    assert!(
        !out.is_empty(),
        "expected a heliocentric Jupiter crossing of 0°"
    );
}
