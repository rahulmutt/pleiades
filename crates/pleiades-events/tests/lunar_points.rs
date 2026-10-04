//! Lunar orbit points (mean and true node, mean apogee and perigee) in the
//! geocentric frames (issue #118).
//!
//! The ELP backend serves the mean lunar points as directions, without a
//! distance (its osculating true node carries the orbit radius at the node).
//! The chart layer reduces them with precession and nutation only (no
//! light-time, no aberration), and the event engine must evaluate them the
//! same way instead of rejecting them for the missing distance. "Node minus
//! Sun" is meaningless, so the heliocentric frame rejects them.

use pleiades_apparent::nutation::nutation;
use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, CompositeBackend, CoordinateFrame, EphemerisBackend,
    EphemerisError, EphemerisRequest, EphemerisResult,
};
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_data::packaged_backend;
use pleiades_elp::ElpBackend;
use pleiades_events::{CrossingFrame, EventEngine, EventError};
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, TimeRange,
    TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;
const MEAN: CrossingFrame = CrossingFrame::GeocentricMeanOfDate;
const HELIO: CrossingFrame = CrossingFrame::Heliocentric;

/// 2025-03-29 00:00 TT, the instant of the issue's reproduction.
const ISSUE_118_JD: f64 = 2_460_763.5;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn wrap_arcsec(deg: f64) -> f64 {
    ((deg + 180.0).rem_euclid(360.0) - 180.0) * 3600.0
}

fn composite() -> CompositeBackend<ElpBackend, Vsop87Backend> {
    CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new())
}

/// The lunar points the ELP backend serves.
fn elp_lunar_points() -> [CelestialBody; 4] {
    [
        CelestialBody::TrueNode,
        CelestialBody::MeanNode,
        CelestialBody::MeanApogee,
        CelestialBody::MeanPerigee,
    ]
}

fn chart_apparent_longitude_deg<B: EphemerisBackend>(
    backend: B,
    body: &CelestialBody,
    jd: f64,
) -> f64 {
    ChartEngine::new(backend)
        .chart(&ChartRequest::new(tdb(jd)).with_bodies(vec![body.clone()]))
        .expect("chart")
        .placement_for(body)
        .expect("placed")
        .position
        .ecliptic
        .expect("ecliptic")
        .longitude
        .degrees()
}

#[test]
fn composite_lunar_points_evaluate_apparent_like_the_chart_layer() {
    // Issue #118: `longitude_at` failed with "no ecliptic coordinates" for a
    // body the backend does serve, while the chart placed it.
    let engine = EventEngine::new(composite());
    for body in elp_lunar_points() {
        let lon = engine
            .longitude_at(body.clone(), APPARENT, tdb(ISSUE_118_JD))
            .unwrap_or_else(|e| panic!("{body:?} apparent: {e}"))
            .degrees();
        let chart = chart_apparent_longitude_deg(composite(), &body, ISSUE_118_JD);
        assert!(
            wrap_arcsec(lon - chart).abs() < 1e-6,
            "{body:?}: events {lon:.9} vs chart {chart:.9}"
        );
    }
}

#[test]
fn packaged_lunar_points_evaluate_apparent_like_the_chart_layer() {
    // The packaged backend serves the lunar points with a distance; they still
    // take the direction-only reduction, so the engine and the chart agree
    // here too (before #118 the engine put them through the light-time
    // pipeline, which the chart never did).
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::MeanNode, CelestialBody::TrueNode] {
        for jd in [2_415_100.25, 2_451_545.0, 2_487_900.5] {
            let lon = engine
                .longitude_at(body.clone(), APPARENT, tdb(jd))
                .unwrap_or_else(|e| panic!("{body:?} {jd}: {e}"))
                .degrees();
            let chart = chart_apparent_longitude_deg(packaged_backend(), &body, jd);
            assert!(
                wrap_arcsec(lon - chart).abs() < 1e-6,
                "{body:?} {jd}: events {lon:.9} vs chart {chart:.9}"
            );
        }
    }
}

#[test]
fn composite_lunar_points_mean_of_date_is_apparent_without_nutation() {
    // A direction gets precession only in the mean frame and precession plus
    // Δψ in the apparent frame; no light-time or aberration separates them.
    let engine = EventEngine::new(composite());
    let delta_psi_deg = nutation(ISSUE_118_JD).unwrap().delta_psi_arcsec / 3600.0;
    for body in elp_lunar_points() {
        let mean = engine
            .longitude_at(body.clone(), MEAN, tdb(ISSUE_118_JD))
            .unwrap_or_else(|e| panic!("{body:?} mean of date: {e}"))
            .degrees();
        let apparent = engine
            .longitude_at(body.clone(), APPARENT, tdb(ISSUE_118_JD))
            .unwrap()
            .degrees();
        assert!(
            wrap_arcsec(apparent - delta_psi_deg - mean).abs() < 1e-6,
            "{body:?}: apparent {apparent:.9} − Δψ vs mean {mean:.9}"
        );
    }
}

#[test]
fn composite_mean_lunar_point_position_has_no_distance() {
    // The ELP mean node is a direction without a distance; the engine must
    // serve it as such rather than reject it (issue #118).
    let engine = EventEngine::new(composite());
    for frame in [APPARENT, MEAN] {
        let pos = engine
            .position_at(CelestialBody::MeanNode, frame, tdb(ISSUE_118_JD))
            .unwrap_or_else(|e| panic!("{frame:?}: {e}"));
        let lon = engine
            .longitude_at(CelestialBody::MeanNode, frame, tdb(ISSUE_118_JD))
            .unwrap();
        assert_eq!(pos.ecliptic.longitude, lon, "{frame:?}");
        assert_eq!(pos.ecliptic.distance_au, None, "{frame:?}");
        assert!(pos.ecliptic.latitude.degrees().is_finite());
    }
}

#[test]
fn composite_true_node_position_carries_the_orbit_radius_at_the_node() {
    // Since issue #127 the ELP true node is the osculating node and carries
    // the orbit radius at the node, like the packaged node; the engine passes
    // that distance through unchanged in both geocentric frames.
    let engine = EventEngine::new(composite());
    for frame in [APPARENT, MEAN] {
        let pos = engine
            .position_at(CelestialBody::TrueNode, frame, tdb(ISSUE_118_JD))
            .unwrap_or_else(|e| panic!("{frame:?}: {e}"));
        let lon = engine
            .longitude_at(CelestialBody::TrueNode, frame, tdb(ISSUE_118_JD))
            .unwrap();
        assert_eq!(pos.ecliptic.longitude, lon, "{frame:?}");
        let distance = pos
            .ecliptic
            .distance_au
            .unwrap_or_else(|| panic!("{frame:?}: the osculating node should carry a distance"));
        assert!(
            (0.0024..0.0028).contains(&distance),
            "{frame:?}: node distance {distance} AU outside the lunar orbit range"
        );
        assert!(pos.ecliptic.latitude.degrees().is_finite());
    }
}

#[test]
fn composite_true_node_crossings_are_found() {
    // Crossings root-find on the same evaluator; a sign ingress of the node
    // failed with the issue's error.
    let engine = EventEngine::new(composite());
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::TrueNode,
            Longitude::from_degrees(330.0),
            APPARENT,
            tdb(ISSUE_118_JD),
        )
        .expect("search succeeds")
        .expect("the retrograde node reaches 330 deg within the window");
    assert!(crossing.instant.julian_day.days() > ISSUE_118_JD);
}

#[test]
fn packaged_lunar_points_keep_their_distance() {
    // The packaged backend serves the lunar points with a distance; the
    // direction-only reduction passes it through.
    let engine = EventEngine::new(packaged_backend());
    for body in [CelestialBody::MeanNode, CelestialBody::TrueNode] {
        for frame in [APPARENT, MEAN] {
            let pos = engine
                .position_at(body.clone(), frame, tdb(2_451_545.0))
                .unwrap_or_else(|e| panic!("{body:?} {frame:?}: {e}"));
            assert!(pos.ecliptic.distance_au.is_some(), "{body:?} {frame:?}");
        }
    }
}

#[test]
fn heliocentric_lunar_points_are_unsupported() {
    // FU-17 (b): "node minus Sun" is not a place. Both backends reject it the
    // same way, whether or not the point carries a distance.
    let packaged = EventEngine::new(packaged_backend());
    let composite = EventEngine::new(composite());
    for body in [
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
        CelestialBody::MeanPerigee,
        CelestialBody::TruePerigee,
    ] {
        for (label, lon, pos) in [
            (
                "packaged",
                packaged.longitude_at(body.clone(), HELIO, tdb(2_451_545.0)),
                packaged.position_at(body.clone(), HELIO, tdb(2_451_545.0)),
            ),
            (
                "composite",
                composite.longitude_at(body.clone(), HELIO, tdb(2_451_545.0)),
                composite.position_at(body.clone(), HELIO, tdb(2_451_545.0)),
            ),
        ] {
            assert!(
                matches!(lon, Err(EventError::UnsupportedFrame { .. })),
                "{label} {body:?} longitude_at: {lon:?}"
            );
            assert!(
                matches!(pos, Err(EventError::UnsupportedFrame { .. })),
                "{label} {body:?} position_at: {pos:?}"
            );
        }
    }
}

/// Serves the Sun as a direction only: ecliptic longitude and latitude with
/// no distance. A body (not a lunar point) without a distance is a broken
/// backend, and the error must name the distance.
struct DirectionOnlySun;

impl EphemerisBackend for DirectionOnlySun {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("direction-only-sun"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("test backend without distances"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tdb],
            body_claims: vec![BodyClaim::from(CelestialBody::Sun)],
            supported_frames: vec![CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        body == CelestialBody::Sun
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = EphemerisResult::new(
            BackendId::new("direction-only-sun"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        // Any other body comes back without ecliptic coordinates.
        if req.body == CelestialBody::Sun {
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(100.0),
                Latitude::from_degrees(0.0),
                None,
            ));
        }
        Ok(result)
    }
}

#[test]
fn a_body_without_a_distance_is_missing_distance_not_missing_coordinates() {
    let engine = EventEngine::new(DirectionOnlySun);
    for frame in [APPARENT, MEAN] {
        let err = engine
            .longitude_at(CelestialBody::Sun, frame, tdb(2_451_545.0))
            .expect_err("a body needs a distance");
        assert!(
            matches!(
                err,
                EventError::MissingDistance {
                    body_label: "Sun",
                    ..
                }
            ),
            "{frame:?}: {err:?}"
        );
        let message = err.to_string();
        assert!(
            message.contains("distance") && message.contains("Sun"),
            "{frame:?}: {message}"
        );
    }
}

#[test]
fn errors_name_the_lunar_point() {
    // Before #118 every lunar point was labelled "body" in error messages.
    let engine = EventEngine::new(DirectionOnlySun);
    let err = engine
        .longitude_at(CelestialBody::TrueNode, APPARENT, tdb(2_451_545.0))
        .expect_err("this backend has no node");
    assert!(err.to_string().contains("true node"), "{err}");
}
