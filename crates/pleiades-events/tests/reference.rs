//! Crossings, longitudes and positions in the mean place of date and in a
//! sidereal zodiac (issue #88).

use pleiades_apparent::nutation::nutation;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, EventEngine, WINDOW_START_JD};
use pleiades_types::{CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

#[test]
fn sun_mean_of_date_differs_from_apparent_by_aberration_and_nutation() {
    // The Sun's apparent place is its geometric place moved by annual
    // aberration (-20.4898"/R, R in AU) and referred to the true equinox
    // (+Δψ). This checks the mean frame against two independently known
    // quantities, not against the engine's own formula.
    let engine = EventEngine::new(packaged_backend());
    for jd in [2_420_000.5, 2_451_545.0, 2_460_000.5, 2_480_000.5] {
        let apparent = engine
            .position_at(CelestialBody::Sun, APPARENT, tdb(jd))
            .unwrap();
        let mean = engine
            .longitude_at(CelestialBody::Sun, MEAN, tdb(jd))
            .unwrap();
        let got_arcsec = wrap(apparent.ecliptic.longitude.degrees() - mean.degrees()) * 3600.0;
        let delta_psi = nutation(jd).unwrap().delta_psi_arcsec;
        let aberration = -20.4898 / apparent.ecliptic.distance_au.unwrap();
        assert!(
            (got_arcsec - (delta_psi + aberration)).abs() < 0.2,
            "jd {jd}: apparent-mean {got_arcsec}\" vs Δψ {delta_psi}\" + aberration {aberration}\""
        );
    }
}

#[test]
fn mean_frame_reads_succeed_at_the_range_start() {
    // No light-time re-query, so the range-start failure of FU-17(c) does
    // not apply to the mean frame.
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::Mars, CelestialBody::Moon] {
        engine
            .longitude_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
        engine
            .position_at(body.clone(), MEAN, tdb(WINDOW_START_JD))
            .unwrap_or_else(|e| panic!("{body:?}: {e}"));
    }
}

#[test]
fn mean_position_longitude_is_bit_identical_to_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Pluto,
    ] {
        for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
            let pos = engine.position_at(body.clone(), MEAN, tdb(jd)).unwrap();
            let lon = engine.longitude_at(body.clone(), MEAN, tdb(jd)).unwrap();
            assert_eq!(
                pos.ecliptic.longitude.degrees().to_bits(),
                lon.degrees().to_bits(),
                "{body:?} {jd}"
            );
            assert_eq!(pos.frame, MEAN);
        }
    }
}

#[test]
fn mean_speed_matches_a_central_difference_of_the_longitude() {
    let engine = EventEngine::new(packaged_backend());
    let h = 0.5;
    for body in [
        CelestialBody::Sun,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
    ] {
        let jd = 2_455_000.5;
        let before = engine
            .longitude_at(body.clone(), MEAN, tdb(jd - h))
            .unwrap();
        let after = engine
            .longitude_at(body.clone(), MEAN, tdb(jd + h))
            .unwrap();
        let rate = wrap(after.degrees() - before.degrees()) / (2.0 * h);
        let speed = engine
            .position_at(body.clone(), MEAN, tdb(jd))
            .unwrap()
            .motion
            .longitude_deg_per_day
            .expect("speed");
        assert!(
            (speed - rate).abs() < 1e-4,
            "{body:?}: speed {speed} vs {rate}"
        );
    }
}

#[test]
fn mean_crossing_lands_on_the_target_from_the_settled_side() {
    // The returned instant trails the crossing by < 0.5 s and never precedes
    // it. Both bodies move direct, so the residual is in [0, speed × 0.5 s).
    let engine = EventEngine::new(packaged_backend());
    for (body, bound_deg) in [(CelestialBody::Sun, 1.0e-5), (CelestialBody::Moon, 1.0e-4)] {
        for target_deg in [0.0, 137.5, 359.9] {
            let crossing = engine
                .next_longitude_crossing(
                    body.clone(),
                    Longitude::from_degrees(target_deg),
                    MEAN,
                    tdb(2_451_545.0),
                )
                .unwrap()
                .expect("crossing");
            assert_eq!(crossing.frame, MEAN);
            let lon = engine
                .longitude_at(body.clone(), MEAN, crossing.instant)
                .unwrap();
            let residual = wrap(lon.degrees() - target_deg);
            assert!(
                (0.0..bound_deg).contains(&residual),
                "{body:?} target {target_deg}: residual {residual}"
            );
        }
    }
}
