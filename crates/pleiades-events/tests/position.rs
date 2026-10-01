//! `EventEngine::position_at`: longitude identity with `longitude_at`, guards,
//! speed behaviour at the window edges and without backend motion, and
//! agreement with the chart layer's apparent place.

use pleiades_backend::{
    BackendMetadata, EphemerisBackend, EphemerisError, EphemerisRequest, EphemerisResult,
};
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_events::{
    CrossingFrame, EclipticPosition, EventEngine, EventError, WINDOW_END_JD, WINDOW_START_JD,
};
use pleiades_types::{Apparentness, CelestialBody, Instant, JulianDay, Motion, TimeScale};

const GEO: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap(deg: f64) -> f64 {
    (deg + 180.0).rem_euclid(360.0) - 180.0
}

/// Delegates to an inner backend, optionally dropping the motion it reports.
struct StripMotion<B> {
    inner: B,
}

impl<B: EphemerisBackend> EphemerisBackend for StripMotion<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = self.inner.position(req)?;
        result.motion = None;
        Ok(result)
    }
}

fn position(body: CelestialBody, frame: CrossingFrame, jd: f64) -> EclipticPosition {
    EventEngine::new(packaged_backend())
        .position_at(body, frame, tdb(jd))
        .expect("position")
}

#[test]
fn longitude_is_bit_identical_to_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ];
    for jd in [2_415_100.25, 2_451_545.0, 2_460_000.5, 2_487_900.5] {
        for body in &bodies {
            for frame in [GEO, HELIO] {
                if frame == HELIO && matches!(body, CelestialBody::Sun | CelestialBody::Moon) {
                    continue;
                }
                let lon = engine.longitude_at(body.clone(), frame, tdb(jd)).unwrap();
                let pos = engine.position_at(body.clone(), frame, tdb(jd)).unwrap();
                assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{body:?} {frame:?} {jd}"
                );
                assert_eq!(pos.body, *body);
                assert_eq!(pos.frame, frame);
                assert_eq!(pos.instant, tdb(jd));
                assert!(pos.ecliptic.distance_au.is_some());
            }
        }
    }
}

#[test]
fn position_and_longitude_agree_or_fail_together() {
    // Lunar points and anything else the longitude evaluator rejects must be
    // rejected by position_at too, and anything it serves must match.
    let engine = EventEngine::new(packaged_backend());
    let bodies = [
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
    ];
    for body in bodies {
        for frame in [GEO, HELIO] {
            let lon = engine.longitude_at(body.clone(), frame, tdb(2_451_545.0));
            let pos = engine.position_at(body.clone(), frame, tdb(2_451_545.0));
            match (lon, pos) {
                (Ok(lon), Ok(pos)) => assert_eq!(
                    pos.ecliptic.longitude.degrees().to_bits(),
                    lon.degrees().to_bits(),
                    "{body:?} {frame:?}"
                ),
                (Err(_), Err(_)) => {}
                (lon, pos) => {
                    panic!("{body:?} {frame:?}: longitude_at {lon:?} vs position_at {pos:?}")
                }
            }
        }
    }
}

#[test]
fn guards_match_longitude_at() {
    let engine = EventEngine::new(packaged_backend());
    for jd in [WINDOW_START_JD - 0.001, WINDOW_END_JD + 0.001] {
        assert!(matches!(
            engine.position_at(CelestialBody::Mars, GEO, tdb(jd)),
            Err(EventError::OutOfWindow { .. })
        ));
    }
    for body in [CelestialBody::Sun, CelestialBody::Moon] {
        assert!(matches!(
            engine.position_at(body, HELIO, tdb(2_451_545.0)),
            Err(EventError::UnsupportedFrame { .. })
        ));
    }
}

#[test]
fn speed_matches_a_small_central_difference_of_the_position() {
    // Independent check of all three channels in both frames: difference the
    // engine's own position over ±0.01 day. Truncation at that step is far
    // below the tolerance; the tolerance absorbs packaged interpolation noise.
    let engine = EventEngine::new(packaged_backend());
    let h = 0.01;
    for (body, frame) in [
        (CelestialBody::Mars, GEO),
        (CelestialBody::Jupiter, GEO),
        (CelestialBody::Mars, HELIO),
        (CelestialBody::Mercury, HELIO),
        (CelestialBody::Neptune, HELIO),
    ] {
        let jd = 2_455_000.5;
        let at = |jd: f64| {
            engine
                .position_at(body.clone(), frame, tdb(jd))
                .unwrap()
                .ecliptic
        };
        let (a, b) = (at(jd - h), at(jd + h));
        let lon_rate = wrap(b.longitude.degrees() - a.longitude.degrees()) / (2.0 * h);
        let lat_rate = (b.latitude.degrees() - a.latitude.degrees()) / (2.0 * h);
        let dist_rate = (b.distance_au.unwrap() - a.distance_au.unwrap()) / (2.0 * h);
        let motion = engine
            .position_at(body.clone(), frame, tdb(jd))
            .unwrap()
            .motion;
        // 1e-4 deg/day = 0.36"/day.
        assert!(
            (motion.longitude_deg_per_day.unwrap() - lon_rate).abs() < 1e-4,
            "{body:?} {frame:?} lon {:?} vs {lon_rate}",
            motion.longitude_deg_per_day
        );
        assert!(
            (motion.latitude_deg_per_day.unwrap() - lat_rate).abs() < 1e-4,
            "{body:?} {frame:?} lat {:?} vs {lat_rate}",
            motion.latitude_deg_per_day
        );
        assert!(
            (motion.distance_au_per_day.unwrap() - dist_rate).abs() < 1e-5,
            "{body:?} {frame:?} dist {:?} vs {dist_rate}",
            motion.distance_au_per_day
        );
    }
}

#[test]
fn heliocentric_neptune_speed_is_in_its_orbital_range() {
    // Neptune's mean motion is 0.00598 deg/day; the of-date speed adds general
    // precession (3.82e-5 deg/day) and the nutation rate. This is a coarse
    // sanity bound; whether the precession rate is included is decided by the
    // signed-mean diagnostic of validate-helio-position.
    let motion = position(CelestialBody::Neptune, HELIO, 2_451_545.0).motion;
    let lon_rate = motion.longitude_deg_per_day.unwrap();
    assert!(
        (0.0055..0.0068).contains(&lon_rate),
        "Neptune helio speed {lon_rate}"
    );
}

#[test]
fn window_edges_use_a_one_sided_difference() {
    for jd in [WINDOW_START_JD, WINDOW_END_JD] {
        for (body, frame) in [(CelestialBody::Mars, HELIO), (CelestialBody::Sun, GEO)] {
            // Heliocentric Mars needs no light-time re-query and the
            // geocentric Sun none either, so both are served at the exact edge.
            let at_edge = EventEngine::new(packaged_backend())
                .position_at(body.clone(), frame, tdb(jd))
                .expect("a position at the window edge");
            let inside = if jd == WINDOW_START_JD {
                jd + 1.0
            } else {
                jd - 1.0
            };
            let near = position(body.clone(), frame, inside);
            let (edge_rate, near_rate) = (
                at_edge
                    .motion
                    .longitude_deg_per_day
                    .expect("one-sided speed, not None"),
                near.motion.longitude_deg_per_day.unwrap(),
            );
            // One day apart, a planet's or the Sun's speed changes by well under 0.05 deg/day.
            assert!(
                (edge_rate - near_rate).abs() < 0.05,
                "{body:?} {frame:?} edge {edge_rate} vs inside {near_rate}"
            );
        }
    }
}

#[test]
fn backend_without_motion_gives_none_speeds() {
    let engine = EventEngine::new(StripMotion {
        inner: packaged_backend(),
    });
    for (body, frame) in [(CelestialBody::Mars, GEO), (CelestialBody::Mars, HELIO)] {
        let pos = engine.position_at(body, frame, tdb(2_451_545.0)).unwrap();
        assert_eq!(pos.motion, Motion::new(None, None, None));
        assert!(pos.ecliptic.distance_au.is_some());
    }
}

#[test]
fn speed_is_continuous_across_the_zero_degree_wrap() {
    // Find a heliocentric Mars crossing of 0° and evaluate the speed on it:
    // the samples at ±0.5 day sit on either side of the 0°/360° seam.
    let engine = EventEngine::new(packaged_backend());
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Mars,
            pleiades_types::Longitude::from_degrees(0.0),
            HELIO,
            tdb(2_451_545.0),
        )
        .unwrap()
        .expect("Mars crosses 0° heliocentric");
    let jd = crossing.instant.julian_day.days();
    let on = position(CelestialBody::Mars, HELIO, jd)
        .motion
        .longitude_deg_per_day
        .unwrap();
    let before = position(CelestialBody::Mars, HELIO, jd - 2.0)
        .motion
        .longitude_deg_per_day
        .unwrap();
    // Heliocentric Mars moves 0.43–0.64 deg/day and changes slowly.
    assert!((0.4..0.7).contains(&on), "speed on the seam {on}");
    assert!((on - before).abs() < 0.01, "seam {on} vs before {before}");
}

#[test]
fn geocentric_position_and_speed_match_the_chart_layer() {
    // The chart's apparent tropical geocentric placement is gated against JPL
    // Horizons by validate-apparent; position_at must report the same place
    // and the same speed. Both run the same computation, so they agree
    // exactly: every channel's difference measured 0.0 at every body and
    // epoch below (2026-10-01).
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Pluto,
    ];
    let engine = EventEngine::new(packaged_backend());
    for jd in [
        WINDOW_START_JD + 10.0,
        2_451_545.0,
        2_460_000.5,
        WINDOW_END_JD - 10.0,
    ] {
        let request = ChartRequest::new(tdb(jd))
            .with_bodies(bodies.to_vec())
            .with_apparentness(Apparentness::Apparent);
        let chart = ChartEngine::new(packaged_backend())
            .chart(&request)
            .expect("chart");
        for body in &bodies {
            let placed = &chart.placement_for(body).expect("placed").position;
            assert_eq!(placed.apparent, Apparentness::Apparent, "{body:?} {jd}");
            let chart_ecl = placed.ecliptic.as_ref().expect("chart ecliptic");
            let chart_motion = placed.motion.expect("chart motion");
            let pos = engine.position_at(body.clone(), GEO, tdb(jd)).unwrap();
            assert_eq!(
                pos.ecliptic.longitude.degrees(),
                chart_ecl.longitude.degrees(),
                "{body:?} {jd} lon"
            );
            assert_eq!(
                pos.ecliptic.latitude.degrees(),
                chart_ecl.latitude.degrees(),
                "{body:?} {jd} lat"
            );
            assert_eq!(
                pos.ecliptic.distance_au, chart_ecl.distance_au,
                "{body:?} {jd} dist"
            );
            for (name, got, want) in [
                (
                    "lon",
                    pos.motion.longitude_deg_per_day,
                    chart_motion.longitude_deg_per_day,
                ),
                (
                    "lat",
                    pos.motion.latitude_deg_per_day,
                    chart_motion.latitude_deg_per_day,
                ),
                (
                    "dist",
                    pos.motion.distance_au_per_day,
                    chart_motion.distance_au_per_day,
                ),
            ] {
                let (got, want) = (got.expect(name), want.expect(name));
                assert_eq!(got, want, "{body:?} {jd} {name} speed");
            }
        }
    }
}
