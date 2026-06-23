use super::test_support::*;
use super::*;

use core::time::Duration;
use std::sync::{Arc, Mutex};

use pleiades_backend::{
    request_policy_summary_for_report, time_scale_policy_summary_for_report,
    validated_frame_policy_summary_for_report, Apparentness, BackendId, EphemerisResult,
    EphemerisResultValidationError,
};
use pleiades_types::{
    Angle, CelestialBody, EclipticCoordinates, HouseSystem, Instant, Latitude, Longitude,
    MotionDirection, ObserverLocation, ObserverLocationValidationError, TimeScale,
    TimeScaleConversion, TimeScaleConversionError, ZodiacSign,
};

use super::houses::HouseSummaryValidationError;
use super::observer::ObserverSummaryValidationError;
use super::placement::BodyPlacementValidationError;
use super::signs::SignSummaryValidationError;
use super::snapshot::ChartSnapshotValidationError;

#[test]
fn default_body_list_contains_the_luminaries() {
    let bodies = default_chart_bodies();
    assert!(bodies.contains(&CelestialBody::Sun));
    assert!(bodies.contains(&CelestialBody::Moon));
}

#[test]
fn sidereal_longitude_applies_ayanamsa() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let sidereal = sidereal_longitude(
        Longitude::from_degrees(5.0),
        instant,
        &ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Lahiri,
        },
    )
    .expect("sidereal conversion should work");

    assert_eq!(ZodiacSign::from_longitude(sidereal), ZodiacSign::Pisces);
}

#[test]
fn sidereal_longitude_wraps_around_when_the_ayanamsa_crosses_zero() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let sidereal = sidereal_longitude(
        Longitude::from_degrees(5.0),
        instant,
        &ZodiacMode::Sidereal {
            ayanamsa: crate::Ayanamsa::Custom(pleiades_types::CustomAyanamsa {
                name: "wraparound offset".to_string(),
                description: Some("sidereal wraparound regression sample".to_string()),
                epoch: Some(pleiades_types::JulianDay::from_days(2451545.0)),
                offset_degrees: Some(Angle::from_degrees(6.0)),
            }),
        },
    )
    .expect("sidereal conversion should normalize into the longitude range");

    assert_eq!(sidereal.degrees(), 359.0);
    assert_eq!(ZodiacSign::from_longitude(sidereal), ZodiacSign::Pisces);
}

#[test]
fn chart_snapshot_assigns_signs() {
    let engine = ChartEngine::new(ToyChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

    let chart = engine.chart(&request).expect("chart should render");
    assert_eq!(chart.backend_id.as_str(), "toy-chart");
    assert_eq!(chart.len(), 2);
    assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Aries));
    assert_eq!(chart.placements[1].sign, Some(ZodiacSign::Taurus));
    assert_eq!(chart.sign_summary().aries, 1);
    assert_eq!(chart.sign_summary().taurus, 1);
    let rendered = chart.to_string();
    assert!(rendered.contains("Sun"));
    assert!(rendered.contains("Moon"));
    assert!(rendered.contains("Sign summary: 1 Aries, 1 Taurus"));
    assert!(rendered.contains("Instant: JD 2451545 (TT)"));
    assert!(rendered.contains(&format!(
        "Frame policy: {}",
        validated_frame_policy_summary_for_report()
    )));
    assert!(rendered.contains(&format!(
        "Apparentness policy: {}",
        request_policy_summary_for_report().apparentness
    )));
}

#[test]
fn chart_snapshot_without_observer_renders_geocentric_policy_line() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let chart = ChartSnapshot {
        backend_id: BackendId::new("toy-chart"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: vec![],
    };

    let rendered = chart.to_string();
    assert_eq!(
        chart.summary_line(),
        "backend=toy-chart; instant=JD 2451545 (TT); placements=0; zodiac=Tropical; apparentness=Mean; observer=geocentric; observer location=none; body observer=none; house system=none; house cusps=0"
    );
    assert!(
        rendered.contains("Observer policy: geocentric body positions; no house observer supplied")
    );
    assert!(!rendered
        .contains("Observer policy: used for houses only; body positions remain geocentric"));
}

#[test]
fn chart_snapshot_exposes_dominant_sign_and_house_summaries() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );

    let chart = ChartSnapshot {
        backend_id: BackendId::new("toy-chart"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Apparent,
        houses: None,
        placements: vec![
            BodyPlacement {
                body: CelestialBody::Sun,
                position: EphemerisResult::new(
                    BackendId::new("toy-chart"),
                    CelestialBody::Sun,
                    instant,
                    pleiades_types::CoordinateFrame::Ecliptic,
                    ZodiacMode::Tropical,
                    Apparentness::Apparent,
                ),
                sign: Some(ZodiacSign::Aries),
                house: Some(1),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Moon,
                position: EphemerisResult::new(
                    BackendId::new("toy-chart"),
                    CelestialBody::Moon,
                    instant,
                    pleiades_types::CoordinateFrame::Ecliptic,
                    ZodiacMode::Tropical,
                    Apparentness::Apparent,
                ),
                sign: Some(ZodiacSign::Aries),
                house: Some(1),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Mars,
                position: EphemerisResult::new(
                    BackendId::new("toy-chart"),
                    CelestialBody::Mars,
                    instant,
                    pleiades_types::CoordinateFrame::Ecliptic,
                    ZodiacMode::Tropical,
                    Apparentness::Apparent,
                ),
                sign: Some(ZodiacSign::Taurus),
                house: Some(2),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Mercury,
                position: EphemerisResult::new(
                    BackendId::new("toy-chart"),
                    CelestialBody::Mercury,
                    instant,
                    pleiades_types::CoordinateFrame::Ecliptic,
                    ZodiacMode::Tropical,
                    Apparentness::Apparent,
                ),
                sign: Some(ZodiacSign::Taurus),
                house: Some(2),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Jupiter,
                position: EphemerisResult::new(
                    BackendId::new("toy-chart"),
                    CelestialBody::Jupiter,
                    instant,
                    pleiades_types::CoordinateFrame::Ecliptic,
                    ZodiacMode::Tropical,
                    Apparentness::Apparent,
                ),
                sign: Some(ZodiacSign::Gemini),
                house: Some(8),
                apparent: None,
                topocentric: None,
            },
        ],
    };

    assert_eq!(
        chart.dominant_sign_summary(),
        SignSummary {
            aries: 2,
            taurus: 2,
            gemini: 0,
            cancer: 0,
            leo: 0,
            virgo: 0,
            libra: 0,
            scorpio: 0,
            sagittarius: 0,
            capricorn: 0,
            aquarius: 0,
            pisces: 0,
        }
    );
    assert_eq!(
        chart.dominant_house_summary(),
        HouseSummary {
            first: 2,
            second: 2,
            third: 0,
            fourth: 0,
            fifth: 0,
            sixth: 0,
            seventh: 0,
            eighth: 0,
            ninth: 0,
            tenth: 0,
            eleventh: 0,
            twelfth: 0,
            unknown: 0,
        }
    );

    let rendered = chart.to_string();
    assert!(rendered.contains("Dominant sign summary: 2 Aries, 2 Taurus"));
    assert!(rendered.contains("Dominant house summary: 2 in 1st house, 2 in 2nd house"));
}

#[test]
fn chart_snapshot_supports_sidereal_signs() {
    let engine = ChartEngine::new(ToyChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    });

    let chart = engine
        .chart(&request)
        .expect("sidereal chart should render");
    assert_eq!(chart.zodiac_mode, request.zodiac_mode);
    assert_eq!(
        chart.placements[0].position.zodiac_mode,
        request.zodiac_mode
    );
    assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Pisces));
    let rendered = chart.to_string();
    assert!(rendered.contains("Zodiac mode: Sidereal (Lahiri)"));
}

#[test]
fn chart_snapshot_preserves_apparentness_choice() {
    // Use ApparentChartBackend which declares Sun as ReleaseGrade, so the
    // engine applies actual apparent-place corrections and the placement
    // reflects Apparentness::Apparent. ToyChartBackend has Constrained bodies
    // only and would fall back to Mean (tested by non_release_grade_body_falls_back_to_mean).
    let engine = ChartEngine::new(ApparentChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun])
    .with_apparentness(Apparentness::Apparent);

    let chart = engine
        .chart(&request)
        .expect("apparent chart should render");
    assert_eq!(
        chart.placements[0].position.apparent,
        Apparentness::Apparent
    );
    assert_eq!(chart.apparentness, Apparentness::Apparent);
    let rendered = chart.to_string();
    assert!(rendered.contains("Apparentness: Apparent"));
    assert!(rendered.contains(&format!(
        "Apparentness policy: {}",
        request_policy_summary_for_report().apparentness
    )));
    assert!(rendered.contains(&format!(
        "Time-scale policy: {}",
        time_scale_policy_summary_for_report().summary_line()
    )));
}

#[test]
fn chart_snapshot_supports_house_placement() {
    let engine = ChartEngine::new(ToyChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(pleiades_types::ObserverLocation::new(
        Latitude::from_degrees(0.0),
        Longitude::from_degrees(0.0),
        None,
    ))
    .with_house_system(crate::HouseSystem::WholeSign)
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

    let chart = engine
        .chart(&request)
        .expect("house-aware chart should render");
    assert!(chart.houses.is_some());
    assert_eq!(chart.placements.len(), 2);
    assert!(chart
        .placements
        .iter()
        .all(|placement| placement.house.is_some()));
    let rendered = chart.to_string();
    assert_eq!(chart.observer_policy(), ObserverPolicy::HouseOnly);
    assert_eq!(
        chart.summary_line(),
        "backend=toy-chart; instant=JD 2451545 (TT); placements=2; zodiac=Tropical; apparentness=Apparent; observer=house-only; observer location=latitude=0°, longitude=0°, elevation=n/a; body observer=none; house system=Whole Sign; house cusps=12"
    );
    assert!(rendered.contains("House system: Whole Sign"));
    assert!(rendered
        .contains("Observer policy: used for houses only; body positions remain geocentric"));
}

#[test]
fn chart_snapshot_keeps_house_observer_out_of_body_position_requests() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(125.0),
    ))
    .with_house_system(crate::HouseSystem::WholeSign)
    .with_bodies(vec![CelestialBody::Sun]);

    let chart = engine
        .chart(&request)
        .expect("chart should render with a house observer");

    assert!(chart.houses.is_some());
    assert_eq!(chart.observer, request.observer);
    let observers = observers.lock().expect("observer log should be lockable");
    assert_eq!(observers.len(), 1);
    assert!(observers.iter().all(Option::is_none));
}

#[test]
fn chart_snapshot_passes_body_observers_through_topocentric_requests() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(35.0),
        Longitude::from_degrees(-80.0),
        Some(50.0),
    );
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_body_observer(body_observer.clone())
    .with_bodies(vec![CelestialBody::Sun]);

    let chart = engine
        .chart(&request)
        .expect("chart should render with a topocentric body observer");

    assert_eq!(chart.body_observer, request.body_observer);
    assert_eq!(chart.observer, request.observer);
    assert!(chart.summary_line().contains("observer=geocentric"));
    assert!(chart.summary_line().contains("body observer=latitude=35°"));
    let observers = observers.lock().expect("observer log should be lockable");
    assert_eq!(observers.as_slice(), &[Some(body_observer)]);
}

#[test]
fn chart_snapshot_keeps_house_and_body_observers_on_distinct_channels() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let house_observer = ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    );
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(-33.9),
        Longitude::from_degrees(151.2),
        None,
    );
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(house_observer.clone())
    .with_body_observer(body_observer.clone())
    .with_house_system(crate::HouseSystem::WholeSign)
    .with_bodies(vec![CelestialBody::Sun]);

    let chart = engine
        .chart(&request)
        .expect("chart should keep the house and body observers separate");

    assert_eq!(chart.observer, request.observer);
    assert_eq!(chart.body_observer, request.body_observer);
    assert!(chart
        .summary_line()
        .contains("observer=house-only; observer location=latitude=12.5°"));
    assert!(chart
        .summary_line()
        .contains("body observer=latitude=-33.9°"));

    let observers = observers.lock().expect("observer log should be lockable");
    assert_eq!(observers.as_slice(), &[Some(body_observer)]);
}

#[test]
fn chart_snapshot_rejects_unsupported_body_before_backend_position() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Moon]);

    let error = engine
        .chart(&request)
        .expect_err("chart should reject unsupported body coverage before dispatch");

    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert!(error
        .message
        .contains("recording-chart does not support Moon"));
    let observers = observers.lock().expect("observer log should be lockable");
    assert!(observers.is_empty());
}

#[test]
fn chart_snapshot_with_observer_but_without_houses_stays_geocentric() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        None,
    ))
    .with_bodies(vec![CelestialBody::Sun]);

    let chart = engine
        .chart(&request)
        .expect("chart should render without house calculations");

    assert!(chart.houses.is_none());
    assert_eq!(chart.observer, request.observer);
    assert_eq!(chart.observer_policy(), ObserverPolicy::Geocentric);
    assert_eq!(
        chart.summary_line(),
        "backend=recording-chart; instant=JD 2451545 (TT); placements=1; zodiac=Tropical; apparentness=Apparent; observer=geocentric; observer location=latitude=51.5°, longitude=359.9°, elevation=n/a; body observer=none; house system=none; house cusps=0"
    );
    assert!(chart
        .to_string()
        .contains("Observer policy: geocentric body positions; no house observer supplied"));
    let observers = observers.lock().expect("observer log should be lockable");
    assert_eq!(observers.len(), 1);
    assert!(observers.iter().all(Option::is_none));
}

#[test]
fn chart_snapshot_rejects_non_finite_house_observer_elevation() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(f64::NAN),
    ))
    .with_house_system(crate::HouseSystem::Topocentric)
    .with_bodies(vec![CelestialBody::Sun]);

    let error = engine
        .chart(&request)
        .expect_err("chart should reject non-finite house observer elevation");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error.message.contains("observer elevation must be finite"));
    let observers = observers.lock().expect("observer log should be lockable");
    assert!(observers.is_empty());
}

#[test]
fn chart_snapshot_rejects_non_finite_house_observer_longitude() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(f64::NAN),
        None,
    ))
    .with_house_system(crate::HouseSystem::Equal)
    .with_bodies(vec![CelestialBody::Sun]);

    let error = engine
        .chart(&request)
        .expect_err("chart should reject non-finite house observer longitude");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error.message.contains("observer longitude must be finite"));
    let observers = observers.lock().expect("observer log should be lockable");
    assert!(observers.is_empty());
}

#[test]
fn chart_snapshot_uses_backend_batch_queries_for_body_positions() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let batch_calls = Arc::new(Mutex::new(0));
    let engine = ChartEngine::new(BatchRecordingChartBackend {
        observers: Arc::clone(&observers),
        batch_calls: Arc::clone(&batch_calls),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

    let chart = engine
        .chart(&request)
        .expect("chart should render through the backend batch path");

    assert_eq!(chart.placements.len(), 2);
    assert_eq!(chart.placements[0].body, CelestialBody::Sun);
    assert_eq!(chart.placements[1].body, CelestialBody::Moon);
    assert_eq!(chart.placements[0].sign, Some(ZodiacSign::Aries));
    assert_eq!(chart.placements[1].sign, Some(ZodiacSign::Taurus));
    assert_eq!(
        *batch_calls
            .lock()
            .expect("batch call log should be lockable"),
        1
    );
    let observers = observers.lock().expect("observer log should be lockable");
    assert_eq!(observers.len(), 2);
    assert!(observers.iter().all(Option::is_none));
}

#[test]
fn chart_request_summary_line_reflects_the_default_request_shape() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ));

    assert_eq!(request.observer_policy(), ObserverPolicy::Geocentric);
    assert_eq!(
        request.summary_line(),
        "instant=JD 2451545 (TT); bodies=10; zodiac=Tropical; apparentness=Apparent; observer=geocentric; observer location=none; body observer=none; house system=none"
    );
    assert_eq!(request.to_string(), request.summary_line());
}

#[test]
fn observer_summary_renders_the_policy_location_and_body_location() {
    let summary = ObserverSummary::new(
        ObserverPolicy::HouseOnly,
        Some(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        )),
    )
    .with_body_location(Some(ObserverLocation::new(
        Latitude::from_degrees(-33.9),
        Longitude::from_degrees(151.2),
        None,
    )));

    assert_eq!(summary.policy, ObserverPolicy::HouseOnly);
    assert_eq!(
        summary.location_label(),
        "latitude=12.5°, longitude=45°, elevation=100.000 m"
    );
    assert_eq!(
        summary.body_location_label(),
        "latitude=-33.9°, longitude=151.2°, elevation=n/a"
    );
    assert_eq!(
        summary
            .validated_location_label()
            .expect("valid observer summary location"),
        "latitude=12.5°, longitude=45°, elevation=100.000 m"
    );
    assert_eq!(
        summary
            .validated_summary_line()
            .expect("valid observer summary"),
        summary.summary_line()
    );
    assert_eq!(
        summary.summary_line(),
        "observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=latitude=-33.9°, longitude=151.2°, elevation=n/a"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
}

#[test]
fn observer_summary_validate_rejects_house_only_without_location() {
    let summary = ObserverSummary::new(ObserverPolicy::HouseOnly, None);

    let error = summary
        .validate()
        .expect_err("house-only summaries should require an observer location");
    assert_eq!(
        error,
        ObserverSummaryValidationError::HouseOnlyMissingObserver
    );
    assert_eq!(
        error.to_string(),
        "observer summary for house-only posture requires an observer location"
    );
}

#[test]
fn observer_summary_validate_rejects_invalid_locations() {
    let summary = ObserverSummary::new(
        ObserverPolicy::Geocentric,
        Some(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            Some(100.0),
        )),
    );

    let error = summary
        .validate()
        .expect_err("observer summaries should reject invalid locations");
    assert!(matches!(
        error,
        ObserverSummaryValidationError::InvalidObserverLocation(
            ObserverLocationValidationError::NonFiniteLongitude { value }
        ) if value.is_nan()
    ));
    assert!(error
        .to_string()
        .contains("observer summary location is invalid: observer longitude must be finite"));
}

#[test]
fn observer_summary_validated_body_location_label_returns_rendered_label() {
    let summary = ObserverSummary::new(ObserverPolicy::Geocentric, None).with_body_location(Some(
        ObserverLocation::new(
            Latitude::from_degrees(-33.9),
            Longitude::from_degrees(151.2),
            None,
        ),
    ));

    assert_eq!(
        summary.validated_body_location_label(),
        Ok("latitude=-33.9°, longitude=151.2°, elevation=n/a".to_string())
    );
}

#[test]
fn observer_summary_validated_body_location_label_rejects_invalid_locations() {
    let summary = ObserverSummary::new(ObserverPolicy::Geocentric, None).with_body_location(Some(
        ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            Some(100.0),
        ),
    ));

    let error = summary
        .validated_body_location_label()
        .expect_err("observer summaries should reject invalid body locations");
    assert!(matches!(
        error,
        ObserverSummaryValidationError::InvalidBodyPosition(
            ObserverLocationValidationError::NonFiniteLongitude { value }
        ) if value.is_nan()
    ));
    assert!(error
        .to_string()
        .contains("observer summary body location is invalid: observer longitude must be finite"));
}

#[test]
fn chart_request_observer_summary_matches_the_rendered_policy_line() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ))
    .with_house_system(crate::HouseSystem::WholeSign);

    let observer = request.observer_summary();

    assert_eq!(observer.policy, ObserverPolicy::HouseOnly);
    assert_eq!(
        observer.summary_line(),
        "observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none"
    );
    assert_eq!(
        request
            .validated_body_observer_label()
            .expect("valid chart request body observer"),
        "none"
    );
    assert!(request
        .summary_line()
        .contains(observer.summary_line().as_str()));
}

#[test]
fn chart_request_validated_body_observer_label_returns_the_rendered_label() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_body_observer(ObserverLocation::new(
        Latitude::from_degrees(-33.9),
        Longitude::from_degrees(151.2),
        None,
    ));

    assert_eq!(
        request.validated_body_observer_label(),
        Ok("latitude=-33.9°, longitude=151.2°, elevation=n/a".to_string())
    );
}

#[test]
fn chart_snapshot_observer_summary_matches_the_rendered_policy_line() {
    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant: Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(45.0),
            Some(100.0),
        )),
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: Vec::new(),
    };

    let observer = snapshot.observer_summary();

    assert_eq!(observer.policy, ObserverPolicy::Geocentric);
    assert_eq!(
        observer.summary_line(),
        "observer=geocentric; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none"
    );
    assert_eq!(
        snapshot
            .validated_body_observer_label()
            .expect("valid chart snapshot body observer"),
        "none"
    );
    assert!(snapshot
        .summary_line()
        .contains(observer.summary_line().as_str()));
}

#[test]
fn chart_snapshot_validated_body_observer_label_returns_the_rendered_label() {
    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant: Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
        observer: None,
        body_observer: Some(ObserverLocation::new(
            Latitude::from_degrees(-33.9),
            Longitude::from_degrees(151.2),
            None,
        )),
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: Vec::new(),
    };

    assert_eq!(
        snapshot.validated_body_observer_label(),
        Ok("latitude=-33.9°, longitude=151.2°, elevation=n/a".to_string())
    );
}

#[test]
fn chart_snapshot_validated_summary_line_rejects_invalid_observer_locations() {
    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant: Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(12.5),
            Longitude::from_degrees(f64::NAN),
            Some(100.0),
        )),
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: Vec::new(),
    };

    let error = snapshot
        .validated_summary_line()
        .expect_err("validated snapshot summaries should reject invalid locations");
    assert!(matches!(
        error,
        ChartSnapshotValidationError::InvalidObserverSummary(
            ObserverSummaryValidationError::InvalidObserverLocation(
                ObserverLocationValidationError::NonFiniteLongitude { value }
            )
        ) if value.is_nan()
    ));
}

#[test]
fn chart_snapshot_validate_rejects_house_assignments_without_house_snapshot() {
    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant: Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: vec![BodyPlacement {
            body: CelestialBody::Sun,
            position: EphemerisResult::new(
                crate::BackendId::new("demo"),
                CelestialBody::Sun,
                Instant::new(
                    pleiades_types::JulianDay::from_days(2_451_545.0),
                    TimeScale::Tt,
                ),
                CoordinateFrame::Ecliptic,
                ZodiacMode::Tropical,
                Apparentness::Mean,
            ),
            sign: Some(ZodiacSign::Aries),
            house: Some(1),
            apparent: None,
            topocentric: None,
        }],
    };

    let error = snapshot
        .validate()
        .expect_err("placement houses should require a house snapshot");
    assert!(matches!(
        error,
        ChartSnapshotValidationError::PlacementHasHouseWithoutSnapshot {
            placement: 1,
            body: CelestialBody::Sun,
            house: 1,
        }
    ));
    assert!(error.to_string().contains(
        "chart snapshot placement 1 for body Sun carries house 1 without a house snapshot"
    ));
}

#[test]
fn chart_snapshot_validate_rejects_out_of_range_house_assignments() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    );
    let observer = ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    );
    let houses = calculate_houses(&HouseRequest::new(
        instant,
        observer.clone(),
        HouseSystem::WholeSign,
    ))
    .expect("house snapshot should validate");
    assert_eq!(houses.cusps.len(), 12);

    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant,
        observer: Some(observer),
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: Some(houses),
        placements: vec![BodyPlacement {
            body: CelestialBody::Sun,
            position: EphemerisResult::new(
                crate::BackendId::new("demo"),
                CelestialBody::Sun,
                instant,
                CoordinateFrame::Ecliptic,
                ZodiacMode::Tropical,
                Apparentness::Mean,
            ),
            sign: Some(ZodiacSign::Aries),
            house: Some(13),
            apparent: None,
            topocentric: None,
        }],
    };

    let error = snapshot
        .validate()
        .expect_err("house numbers beyond the snapshot cusp count should fail");
    assert!(matches!(
        error,
        ChartSnapshotValidationError::PlacementHouseOutOfRange {
            placement: 1,
            body: CelestialBody::Sun,
            house: 13,
            house_count: 12,
        }
    ));
    assert!(error.to_string().contains(
        "chart snapshot placement 1 for body Sun carries house 13 but the house snapshot only has 12 cusps"
    ));
}

#[test]
fn chart_snapshot_validate_rejects_house_snapshots_without_observers() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    );
    let observer = ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    );
    let houses = calculate_houses(&HouseRequest::new(
        instant,
        observer.clone(),
        HouseSystem::WholeSign,
    ))
    .expect("house snapshot should validate");

    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: Some(houses),
        placements: Vec::new(),
    };

    let error = snapshot
        .validate()
        .expect_err("house snapshots should require an observer location");
    assert!(matches!(
        error,
        ChartSnapshotValidationError::HouseSnapshotMissingObserver
    ));
    assert_eq!(
        error.to_string(),
        "chart snapshot includes houses but no observer location"
    );
}

#[test]
fn body_placement_validate_rejects_zero_house_numbers() {
    let placement = BodyPlacement {
        body: CelestialBody::Sun,
        position: EphemerisResult::new(
            crate::BackendId::new("demo"),
            CelestialBody::Sun,
            Instant::new(
                pleiades_types::JulianDay::from_days(2_451_545.0),
                TimeScale::Tt,
            ),
            CoordinateFrame::Ecliptic,
            ZodiacMode::Tropical,
            Apparentness::Mean,
        ),
        sign: Some(ZodiacSign::Aries),
        house: Some(0),
        apparent: None,
        topocentric: None,
    };

    let error = placement
        .validate()
        .expect_err("body placements should reject zero-based house numbers");
    assert!(matches!(
        error,
        BodyPlacementValidationError::InvalidHouseNumber {
            body: CelestialBody::Sun,
            house: 0,
        }
    ));
    assert!(error
        .to_string()
        .contains("body Sun placement house must be one-based, got 0"));
}

#[test]
fn chart_snapshot_validate_rejects_invalid_body_placements() {
    let snapshot = ChartSnapshot {
        backend_id: crate::BackendId::new("demo"),
        instant: Instant::new(
            pleiades_types::JulianDay::from_days(2_451_545.0),
            TimeScale::Tt,
        ),
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
        houses: None,
        placements: vec![BodyPlacement {
            body: CelestialBody::Sun,
            position: EphemerisResult::new(
                crate::BackendId::new("demo"),
                CelestialBody::Sun,
                Instant::new(
                    pleiades_types::JulianDay::from_days(2_451_545.0),
                    TimeScale::Tt,
                ),
                CoordinateFrame::Ecliptic,
                ZodiacMode::Tropical,
                Apparentness::Mean,
            ),
            sign: Some(ZodiacSign::Aries),
            house: None,
            apparent: None,
            topocentric: None,
        }],
    };
    let mut invalid_snapshot = snapshot.clone();
    invalid_snapshot.placements[0].position.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(f64::NAN),
        Latitude::from_degrees(2.5),
        Some(1.0),
    ));

    let error = invalid_snapshot
        .validate()
        .expect_err("invalid placement positions should fail validation");
    assert!(matches!(
        error,
        ChartSnapshotValidationError::InvalidPlacement {
            placement: 1,
            error: BodyPlacementValidationError::InvalidPosition {
                body: CelestialBody::Sun,
                error: EphemerisResultValidationError::InvalidEcliptic(
                    pleiades_types::CoordinateValidationError::NonFiniteValue {
                        coordinate: "ecliptic",
                        field: "longitude",
                        value,
                    }
                ),
            },
        } if value.is_nan()
    ));
    assert!(error
        .to_string()
        .contains("chart snapshot placement 1 is invalid: body Sun placement result is invalid: backend result ecliptic is invalid: ecliptic coordinate field `longitude` must be finite"));
}

#[test]
fn chart_request_validated_summary_line_rejects_invalid_observer_locations() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(f64::NAN),
        Some(100.0),
    ));

    let error = request
        .validated_summary_line()
        .expect_err("validated request summaries should reject invalid locations");
    assert!(matches!(
        error,
        ObserverSummaryValidationError::InvalidObserverLocation(
            ObserverLocationValidationError::NonFiniteLongitude { value }
        ) if value.is_nan()
    ));
}

#[test]
fn chart_request_summary_line_reflects_observer_and_house_system_policy() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ))
    .with_house_system(crate::HouseSystem::WholeSign)
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    })
    .with_apparentness(Apparentness::Apparent);

    assert_eq!(request.observer_policy(), ObserverPolicy::HouseOnly);
    assert_eq!(request.observer_summary().policy, ObserverPolicy::HouseOnly);
    assert_eq!(ObserverPolicy::HouseOnly.summary_line(), "house-only");
    assert_eq!(
        request.summary_line(),
        "instant=JD 2451545 (UTC); bodies=2; zodiac=Sidereal (Lahiri); apparentness=Apparent; observer=house-only; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none; house system=Whole Sign"
    );
}

#[test]
fn chart_request_summary_line_stays_geocentric_without_a_house_system() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);

    assert_eq!(request.observer_policy(), ObserverPolicy::Geocentric);
    assert_eq!(
        request.summary_line(),
        "instant=JD 2451545 (TT); bodies=2; zodiac=Tropical; apparentness=Apparent; observer=geocentric; observer location=latitude=12.5°, longitude=45°, elevation=100.000 m; body observer=none; house system=none"
    );
}

#[test]
fn chart_request_summary_line_preserves_custom_house_system_details() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("My Alias".to_string());
    custom.notes = Some("uses a local calibration".to_string());

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_house_system(crate::HouseSystem::Custom(custom));

    assert_eq!(
        request.summary_line(),
        "instant=JD 2451545 (TT); bodies=10; zodiac=Tropical; apparentness=Apparent; observer=geocentric; observer location=none; body observer=none; house system=My Custom Houses [aliases: My Alias] (uses a local calibration)"
    );
}

#[test]
fn chart_request_validation_rejects_invalid_custom_definitions_before_backend_dispatch() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let mut custom = pleiades_types::CustomHouseSystem::new("   ");
    custom.aliases.push("MCH".to_string());

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        None,
    ))
    .with_house_system(crate::HouseSystem::Custom(custom));

    let validation_error = engine
        .validate_chart_request(&request)
        .expect_err("blank custom house systems should be rejected before metadata checks");
    assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(validation_error
        .message
        .contains("chart house system is invalid: custom house system name must not be blank"));

    let chart_error = engine
        .chart(&request)
        .expect_err("blank custom house systems should be rejected before backend dispatch");
    assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(chart_error
        .message
        .contains("chart house system is invalid: custom house system name must not be blank"));
    assert!(observers
        .lock()
        .expect("observer log should be lockable")
        .is_empty());
}

#[test]
fn chart_request_validation_rejects_custom_definitions_that_collide_with_builtins() {
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::new(Mutex::new(Vec::new())),
    });

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        None,
    ))
    .with_house_system(crate::HouseSystem::Custom(
        pleiades_types::CustomHouseSystem::new("Equal"),
    ));

    let validation_error = engine
        .validate_chart_request(&request)
        .expect_err("built-in house names should be rejected for custom house systems");
    assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(validation_error
        .message
        .contains("chart house system is invalid: custom house system name must not match a built-in label: Equal"));

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        None,
    ))
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Custom(pleiades_types::CustomAyanamsa::new("Lahiri")),
    });

    let validation_error = engine
        .validate_chart_request(&request)
        .expect_err("built-in ayanamsa names should be rejected for custom ayanamsas");
    assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
    assert!(validation_error
        .message
        .contains("sidereal ayanamsa is invalid: custom ayanamsa name must not match a built-in label: Lahiri"));
}

#[test]
fn chart_request_validation_accepts_apparent_for_mean_only_backends() {
    // Apparent-place corrections are now applied in the engine layer, not the
    // backend. Validation no longer rejects Apparent for backends that declare
    // `apparent: false` — the engine always sends Mean to the backend and
    // applies corrections itself (for ReleaseGrade bodies) or falls back
    // gracefully to Mean (for Constrained bodies). MeanOnlyRecordingChartBackend
    // has Constrained-tier bodies so the placement falls back to Mean, but the
    // chart-level apparentness reflects the caller's Apparent request.
    let observers = Arc::new(Mutex::new(Vec::new()));
    let apparent_calls = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(MeanOnlyRecordingChartBackend {
        observers: Arc::clone(&observers),
        apparent_calls: Arc::clone(&apparent_calls),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun])
    .with_apparentness(Apparentness::Apparent);

    // Validation now succeeds — the engine handles apparent internally.
    engine
        .validate_chart_request(&request)
        .expect("apparent chart requests should pass validation for mean-only backends");

    // The chart assembles successfully; chart-level apparentness is Apparent.
    let snapshot = engine
        .chart(&request)
        .expect("apparent chart should succeed for mean-only backends");
    assert_eq!(snapshot.apparentness, Apparentness::Apparent);

    // The backend always receives Mean requests regardless of the chart apparentness.
    let observed = observers.lock().expect("observer log should be lockable");
    assert!(!observed.is_empty(), "backend should have been called");

    // Core invariant: every EphemerisRequest the engine sent to the backend must
    // carry Apparentness::Mean — apparent corrections are composed in the engine
    // layer, never delegated to the backend.
    let recorded_apparent = apparent_calls
        .lock()
        .expect("apparent call log should be lockable");
    assert!(
        recorded_apparent.iter().all(|a| *a == Apparentness::Mean),
        "engine must send Apparentness::Mean to the backend; got: {:?}",
        *recorded_apparent,
    );
}

#[test]
fn chart_request_validation_rejects_house_requests_without_observers_before_backend_dispatch() {
    let observers = Arc::new(Mutex::new(Vec::new()));
    let engine = ChartEngine::new(RecordingChartBackend {
        observers: Arc::clone(&observers),
    });
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun])
    .with_house_system(crate::HouseSystem::WholeSign);

    let validation_error = engine
        .validate_chart_request(&request)
        .expect_err("house requests should require an observer before backend dispatch");
    assert_eq!(validation_error.kind, EphemerisErrorKind::InvalidRequest);
    assert_eq!(
        validation_error.message,
        "house placement requires an observer location"
    );

    let chart_error = engine
        .chart(&request)
        .expect_err("house requests should require an observer before backend dispatch");
    assert_eq!(chart_error.kind, EphemerisErrorKind::InvalidRequest);
    assert_eq!(
        chart_error.message,
        "house placement requires an observer location"
    );
    assert!(observers
        .lock()
        .expect("observer log should be lockable")
        .is_empty());
}

#[test]
fn chart_request_can_apply_a_caller_supplied_time_scale_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_instant_time_scale_offset(TimeScale::Tt, 64.184);

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_apply_a_checked_caller_supplied_time_scale_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
    .expect("UT1 chart request should accept a checked caller-supplied offset");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_apply_a_caller_supplied_time_scale_conversion_policy() {
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_time_scale_conversion(policy)
    .expect("UT1 chart request should accept a caller-supplied policy");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
    assert_eq!(
        policy.summary_line(),
        "source=UT1; target=TDB; offset_seconds=64.184 s"
    );
}

#[test]
fn chart_request_caller_supplied_conversion_preserves_custom_house_metadata() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My UTC Custom Houses");
    custom.aliases.push("My UTC Alias".to_string());
    custom.notes = Some("uses a local UTC calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(34.5),
        Longitude::from_degrees(-118.25),
        Some(75.0),
    );
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(-33.9),
        Longitude::from_degrees(151.2),
        None,
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_observer(observer)
    .with_body_observer(body_observer.clone())
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    })
    .with_apparentness(Apparentness::Apparent);

    let converted = request
        .clone()
        .with_time_scale_conversion(TimeScaleConversion::new(
            TimeScale::Utc,
            TimeScale::Tdb,
            64.184,
        ))
        .expect("UTC chart request should accept a caller-supplied policy");

    assert_eq!(converted.instant.scale, TimeScale::Tdb);
    assert_eq!(converted.observer, request.observer);
    assert_eq!(converted.body_observer, request.body_observer);
    assert_eq!(converted.bodies, request.bodies);
    assert_eq!(converted.zodiac_mode, request.zodiac_mode);
    assert_eq!(converted.apparentness, request.apparentness);
    assert_eq!(converted.house_system, request.house_system);

    let summary = converted.summary_line();
    assert!(summary.contains("(TDB);"));
    assert!(summary.contains("bodies=2;"));
    assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
    assert!(summary.contains("apparentness=Apparent;"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains("body observer=latitude=-33.9°, longitude=151.2°, elevation=n/a"));
    assert!(summary.contains(
        "house system=My UTC Custom Houses [aliases: My UTC Alias] (uses a local UTC calibration)"
    ));
}

#[test]
fn chart_request_can_convert_ut1_to_tt() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_tt_from_ut1(Duration::from_secs_f64(64.184))
    .expect("UT1 chart request should convert to TT");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_ut1_to_tt_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_tt_from_ut1_signed(64.184)
    .expect("UT1 chart request should accept signed TT offsets");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_signed_tt_helpers_preserve_body_and_house_observers() {
    let mut ut1_custom = pleiades_types::CustomHouseSystem::new("My UT1 Signed Custom Houses");
    ut1_custom.aliases.push("My UT1 Signed Alias".to_string());
    ut1_custom.notes = Some("uses a signed UT1 calibration".to_string());

    let ut1_observer = ObserverLocation::new(
        Latitude::from_degrees(-14.6),
        Longitude::from_degrees(34.9),
        Some(1100.0),
    );
    let ut1_body_observer = ObserverLocation::new(
        Latitude::from_degrees(19.8),
        Longitude::from_degrees(155.5),
        None,
    );

    let ut1_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_observer(ut1_observer)
    .with_body_observer(ut1_body_observer.clone())
    .with_house_system(HouseSystem::Custom(ut1_custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_apparentness(Apparentness::Apparent);

    let ut1_converted = ut1_request
        .clone()
        .with_tt_from_ut1_signed(64.184)
        .expect("UT1 chart request should accept signed TT conversion helpers");

    assert_eq!(ut1_converted.instant.scale, TimeScale::Tt);
    assert_eq!(ut1_converted.observer, ut1_request.observer);
    assert_eq!(ut1_converted.body_observer, ut1_request.body_observer);
    assert_eq!(ut1_converted.house_system, ut1_request.house_system);
    assert_eq!(ut1_converted.bodies, ut1_request.bodies);
    assert_eq!(ut1_converted.apparentness, ut1_request.apparentness);

    let ut1_summary = ut1_converted.summary_line();
    assert!(ut1_summary.contains("(TT);"));
    assert!(ut1_summary.contains("observer=house-only;"));
    assert!(ut1_summary.contains("body observer=latitude=19.8°, longitude=155.5°, elevation=n/a"));
    assert!(ut1_summary.contains(
        "house system=My UT1 Signed Custom Houses [aliases: My UT1 Signed Alias] (uses a signed UT1 calibration)"
    ));

    let mut utc_custom = pleiades_types::CustomHouseSystem::new("My UTC Signed Custom Houses");
    utc_custom.aliases.push("My UTC Signed Alias".to_string());
    utc_custom.notes = Some("uses a signed UTC calibration".to_string());

    let utc_observer = ObserverLocation::new(
        Latitude::from_degrees(23.1),
        Longitude::from_degrees(-82.3),
        Some(15.0),
    );
    let utc_body_observer = ObserverLocation::new(
        Latitude::from_degrees(-31.9),
        Longitude::from_degrees(115.9),
        None,
    );

    let utc_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_observer(utc_observer)
    .with_body_observer(utc_body_observer.clone())
    .with_house_system(HouseSystem::Custom(utc_custom))
    .with_bodies(vec![CelestialBody::Mercury, CelestialBody::Venus])
    .with_apparentness(Apparentness::Apparent);

    let utc_converted = utc_request
        .clone()
        .with_tt_from_utc_signed(64.184)
        .expect("UTC chart request should accept signed TT conversion helpers");

    assert_eq!(utc_converted.instant.scale, TimeScale::Tt);
    assert_eq!(utc_converted.observer, utc_request.observer);
    assert_eq!(utc_converted.body_observer, utc_request.body_observer);
    assert_eq!(utc_converted.house_system, utc_request.house_system);
    assert_eq!(utc_converted.bodies, utc_request.bodies);
    assert_eq!(utc_converted.apparentness, utc_request.apparentness);

    let utc_summary = utc_converted.summary_line();
    assert!(utc_summary.contains("(TT);"));
    assert!(utc_summary.contains("observer=house-only;"));
    assert!(utc_summary.contains("body observer=latitude=-31.9°, longitude=115.9°, elevation=n/a"));
    assert!(utc_summary.contains(
        "house system=My UTC Signed Custom Houses [aliases: My UTC Signed Alias] (uses a signed UTC calibration)"
    ));
}

#[test]
fn chart_request_can_convert_utc_to_tt() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_tt_from_utc(Duration::from_secs_f64(64.184))
    .expect("UTC chart request should convert to TT");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_utc_to_tt_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_tt_from_utc_signed(64.184)
    .expect("UTC chart request should accept signed TT offsets");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 + 64.184 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_tt_to_tdb() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
    .expect("TT chart request should convert to TDB");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + 0.001_657 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_tt_to_tdb_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_tdb_from_tt_signed(-0.001_657)
    .expect("TT chart request should accept signed TDB offsets");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_tdb_to_tt() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tdb,
    ))
    .with_tt_from_tdb(-0.001_657)
    .expect("TDB chart request should convert to TT");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_tdb_to_tt_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tdb,
    ))
    .with_tt_from_tdb_signed(-0.001_657)
    .expect("TDB chart request should accept signed TT offsets");

    assert_eq!(request.instant.scale, TimeScale::Tt);
    let expected = 2_451_545.0 - 0.001_657 / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_ut1_to_tdb() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_tdb_from_ut1(
        Duration::from_secs_f64(64.184),
        Duration::from_secs_f64(0.001_657),
    )
    .expect("UT1 chart request should convert to TDB");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_ut1_to_tdb_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
    .expect("UT1 chart request should accept signed TDB offsets");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_utc_to_tdb() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_tdb_from_utc(
        Duration::from_secs_f64(64.184),
        Duration::from_secs_f64(0.001_657),
    )
    .expect("UTC chart request should convert to TDB");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 + 0.001_657) / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_can_convert_utc_to_tdb_with_signed_offset() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
    .expect("UTC chart request should accept signed TDB offsets");

    assert_eq!(request.instant.scale, TimeScale::Tdb);
    let expected = 2_451_545.0 + (64.184 - 0.001_657) / 86_400.0;
    assert!((request.instant.julian_day.days() - expected).abs() < 1e-9);
}

#[test]
fn chart_request_utc_conversion_helpers_preserve_body_and_house_observers() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My UTC Custom Houses");
    custom.aliases.push("My UTC Alias".to_string());
    custom.notes = Some("uses a local UTC calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(34.5),
        Longitude::from_degrees(-118.25),
        Some(75.0),
    );
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(-33.9),
        Longitude::from_degrees(151.2),
        None,
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_observer(observer)
    .with_body_observer(body_observer.clone())
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_apparentness(Apparentness::Apparent);

    let tt_converted = request
        .clone()
        .with_tt_from_utc(Duration::from_secs_f64(64.184))
        .expect("UTC chart request should accept TT conversion helpers");

    assert_eq!(tt_converted.instant.scale, TimeScale::Tt);
    assert_eq!(tt_converted.observer, request.observer);
    assert_eq!(tt_converted.body_observer, request.body_observer);
    assert_eq!(tt_converted.house_system, request.house_system);
    assert_eq!(tt_converted.bodies, request.bodies);
    assert_eq!(tt_converted.apparentness, request.apparentness);

    let tdb_converted = request
        .clone()
        .with_tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC chart request should accept TDB conversion helpers");

    assert_eq!(tdb_converted.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_converted.observer, request.observer);
    assert_eq!(tdb_converted.body_observer, request.body_observer);
    assert_eq!(tdb_converted.house_system, request.house_system);
    assert_eq!(tdb_converted.bodies, request.bodies);
    assert_eq!(tdb_converted.apparentness, request.apparentness);

    let signed_converted = request
        .clone()
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC chart request should accept signed TDB offsets");

    assert_eq!(signed_converted.instant.scale, TimeScale::Tdb);
    assert_eq!(signed_converted.observer, request.observer);
    assert_eq!(signed_converted.body_observer, request.body_observer);
    assert_eq!(signed_converted.house_system, request.house_system);
    assert_eq!(signed_converted.bodies, request.bodies);
    assert_eq!(signed_converted.apparentness, request.apparentness);

    let summary = signed_converted.summary_line();
    assert!(summary.contains("(TDB);"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains("body observer=latitude=-33.9°, longitude=151.2°, elevation=n/a"));
    assert!(summary.contains(
        "house system=My UTC Custom Houses [aliases: My UTC Alias] (uses a local UTC calibration)"
    ));
}

#[test]
fn chart_request_ut1_conversion_helpers_preserve_body_and_house_observers() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My UT1 Custom Houses");
    custom.aliases.push("My UT1 Alias".to_string());
    custom.notes = Some("uses a local UT1 calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(40.7),
        Longitude::from_degrees(-74.0),
        Some(10.0),
    );
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(35.7),
        Longitude::from_degrees(139.7),
        None,
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_observer(observer)
    .with_body_observer(body_observer.clone())
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_apparentness(Apparentness::Apparent);

    let tt_converted = request
        .clone()
        .with_tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 chart request should accept TT conversion helpers");

    assert_eq!(tt_converted.instant.scale, TimeScale::Tt);
    assert_eq!(tt_converted.observer, request.observer);
    assert_eq!(tt_converted.body_observer, request.body_observer);
    assert_eq!(tt_converted.house_system, request.house_system);
    assert_eq!(tt_converted.bodies, request.bodies);
    assert_eq!(tt_converted.apparentness, request.apparentness);

    let tdb_converted = request
        .clone()
        .with_tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 chart request should accept TDB conversion helpers");

    assert_eq!(tdb_converted.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_converted.observer, request.observer);
    assert_eq!(tdb_converted.body_observer, request.body_observer);
    assert_eq!(tdb_converted.house_system, request.house_system);
    assert_eq!(tdb_converted.bodies, request.bodies);
    assert_eq!(tdb_converted.apparentness, request.apparentness);

    let summary = tdb_converted.summary_line();
    assert!(summary.contains("(TDB);"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains("body observer=latitude=35.7°, longitude=139.7°, elevation=n/a"));
    assert!(summary.contains(
        "house system=My UT1 Custom Houses [aliases: My UT1 Alias] (uses a local UT1 calibration)"
    ));
}

#[test]
fn chart_request_tdb_conversion_helpers_preserve_body_and_house_observers() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My TDB Custom Houses");
    custom.aliases.push("My TDB Alias".to_string());
    custom.notes = Some("uses a local TDB calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(48.8),
        Longitude::from_degrees(2.3),
        Some(35.0),
    );
    let body_observer = ObserverLocation::new(
        Latitude::from_degrees(-23.5),
        Longitude::from_degrees(-46.6),
        None,
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tdb,
    ))
    .with_observer(observer)
    .with_body_observer(body_observer.clone())
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Mercury, CelestialBody::Venus])
    .with_apparentness(Apparentness::Apparent);

    let tt_converted = request
        .clone()
        .with_tt_from_tdb(-0.001_657)
        .expect("TDB chart request should accept TT conversion helpers");

    assert_eq!(tt_converted.instant.scale, TimeScale::Tt);
    assert_eq!(tt_converted.observer, request.observer);
    assert_eq!(tt_converted.body_observer, request.body_observer);
    assert_eq!(tt_converted.house_system, request.house_system);
    assert_eq!(tt_converted.bodies, request.bodies);
    assert_eq!(tt_converted.apparentness, request.apparentness);

    let signed_tt_converted = request
        .clone()
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB chart request should accept signed TT conversion helpers");

    assert_eq!(signed_tt_converted.instant.scale, TimeScale::Tt);
    assert_eq!(signed_tt_converted.observer, request.observer);
    assert_eq!(signed_tt_converted.body_observer, request.body_observer);
    assert_eq!(signed_tt_converted.house_system, request.house_system);
    assert_eq!(signed_tt_converted.bodies, request.bodies);
    assert_eq!(signed_tt_converted.apparentness, request.apparentness);

    let summary = signed_tt_converted.summary_line();
    assert!(summary.contains("(TT);"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains("body observer=latitude=-23.5°, longitude=313.4°, elevation=n/a"));
    assert!(summary.contains(
        "house system=My TDB Custom Houses [aliases: My TDB Alias] (uses a local TDB calibration)"
    ));
}

#[test]
fn chart_request_signed_time_scale_helpers_reject_non_finite_offsets() {
    let checked_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ));
    let checked_error = checked_request
        .with_instant_time_scale_offset_checked(TimeScale::Tt, f64::NAN)
        .expect_err("UT1 chart request should reject non-finite checked offsets");
    assert!(matches!(
        checked_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let tt_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ));
    let tt_error = tt_request
        .with_tdb_from_tt_signed(f64::NAN)
        .expect_err("TT chart request should reject non-finite TDB offsets");
    assert!(matches!(
        tt_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let ut1_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ));
    let ut1_error = ut1_request
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), f64::INFINITY)
        .expect_err("UT1 chart request should reject non-finite TDB offsets");
    assert!(matches!(
        ut1_error,
        TimeScaleConversionError::NonFiniteOffset
    ));

    let utc_request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ));
    let utc_error = utc_request
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), f64::NEG_INFINITY)
        .expect_err("UTC chart request should reject non-finite TDB offsets");
    assert!(matches!(
        utc_error,
        TimeScaleConversionError::NonFiniteOffset
    ));
}

#[test]
fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My Custom Houses");
    custom.aliases.push("My Alias".to_string());
    custom.notes = Some("uses a local calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Utc,
    ))
    .with_observer(observer)
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    })
    .with_apparentness(Apparentness::Apparent);

    let converted = request
        .clone()
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC chart request should accept signed TDB offsets");

    assert_eq!(converted.instant.scale, TimeScale::Tdb);
    assert_eq!(converted.observer, request.observer);
    assert_eq!(converted.bodies, request.bodies);
    assert_eq!(converted.zodiac_mode, request.zodiac_mode);
    assert_eq!(converted.apparentness, request.apparentness);
    assert_eq!(converted.house_system, request.house_system);
    let summary = converted.summary_line();
    assert!(summary.contains("(TDB);"));
    assert!(summary.contains("bodies=2;"));
    assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
    assert!(summary.contains("apparentness=Apparent;"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary
        .contains("house system=My Custom Houses [aliases: My Alias] (uses a local calibration)"));
}

#[test]
fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape_for_ut1() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My UT1 Custom Houses");
    custom.aliases.push("My UT1 Alias".to_string());
    custom.notes = Some("uses a local UT1 calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(-22.75),
        Longitude::from_degrees(135.5),
        Some(250.0),
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ))
    .with_observer(observer)
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Mars, CelestialBody::Saturn])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    })
    .with_apparentness(Apparentness::Apparent);

    let converted = request
        .clone()
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 chart request should accept signed TDB offsets");

    assert_eq!(converted.instant.scale, TimeScale::Tdb);
    assert_eq!(converted.observer, request.observer);
    assert_eq!(converted.bodies, request.bodies);
    assert_eq!(converted.zodiac_mode, request.zodiac_mode);
    assert_eq!(converted.apparentness, request.apparentness);
    assert_eq!(converted.house_system, request.house_system);
    let summary = converted.summary_line();
    assert!(summary.contains("(TDB);"));
    assert!(summary.contains("bodies=2;"));
    assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
    assert!(summary.contains("apparentness=Apparent;"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains(
        "house system=My UT1 Custom Houses [aliases: My UT1 Alias] (uses a local UT1 calibration)"
    ));
}

#[test]
fn chart_request_validate_time_scale_conversion_matches_the_policy_helper() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Ut1,
    ));
    let policy = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tdb, 64.184);

    assert!(request.validate_time_scale_conversion(policy).is_ok());

    let mismatched = TimeScaleConversion::new(TimeScale::Tt, TimeScale::Tdb, 64.184);
    let error = request
        .validate_time_scale_conversion(mismatched)
        .expect_err("chart request should reject the wrong source scale");
    assert!(matches!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tt,
            actual: TimeScale::Ut1,
        }
    ));

    let non_finite = TimeScaleConversion::new(TimeScale::Ut1, TimeScale::Tt, f64::NAN);
    assert!(matches!(
        request.validate_time_scale_conversion(non_finite),
        Err(TimeScaleConversionError::NonFiniteOffset)
    ));
}

#[test]
fn chart_request_validate_house_observer_policy_requires_an_observer_for_house_requests() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_house_system(crate::HouseSystem::WholeSign);

    let error = request
        .validate_house_observer_policy()
        .expect_err("house requests should require an observer location");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert_eq!(
        error.message,
        "house placement requires an observer location"
    );
}

#[test]
fn chart_request_validate_house_observer_policy_allows_house_requests_with_observers() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ))
    .with_house_system(crate::HouseSystem::WholeSign);

    assert!(request.validate_house_observer_policy().is_ok());
}

#[test]
fn chart_request_validate_observer_location_rejects_invalid_observers_without_houses() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(95.0),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ));

    let error = request
        .validate_observer_location()
        .expect_err("observer validation should reject out-of-range latitude");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error
        .message
        .contains("observer latitude must stay within [-90, 90], got 95"));
}

#[test]
fn chart_request_validate_observer_location_rejects_invalid_body_observers_without_houses() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_body_observer(ObserverLocation::new(
        Latitude::from_degrees(95.0),
        Longitude::from_degrees(45.0),
        Some(100.0),
    ));

    let error = request
        .validate_observer_location()
        .expect_err("body-observer validation should reject out-of-range latitude");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error
        .message
        .contains("observer latitude must stay within [-90, 90], got 95"));
}

#[test]
fn chart_request_validate_house_observer_policy_rejects_invalid_observers_without_houses() {
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_observer(ObserverLocation::new(
        Latitude::from_degrees(12.5),
        Longitude::from_degrees(f64::NAN),
        Some(100.0),
    ));

    let error = request
        .validate_house_observer_policy()
        .expect_err("house-observer policy should reject invalid observer coordinates");
    assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(error.message.contains("observer longitude must be finite"));
}

#[test]
fn chart_request_time_scale_conversions_preserve_the_rest_of_the_request_shape_for_tdb() {
    let mut custom = pleiades_types::CustomHouseSystem::new("My TDB Custom Houses");
    custom.aliases.push("My TDB Alias".to_string());
    custom.notes = Some("uses a local TDB calibration".to_string());

    let observer = ObserverLocation::new(
        Latitude::from_degrees(34.5),
        Longitude::from_degrees(-118.25),
        Some(75.0),
    );

    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tdb,
    ))
    .with_observer(observer)
    .with_house_system(HouseSystem::Custom(custom))
    .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon])
    .with_zodiac_mode(ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    })
    .with_apparentness(Apparentness::Apparent);

    let converted = request
        .clone()
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB chart request should accept signed TT offsets");

    assert_eq!(converted.instant.scale, TimeScale::Tt);
    assert_eq!(converted.observer, request.observer);
    assert_eq!(converted.bodies, request.bodies);
    assert_eq!(converted.zodiac_mode, request.zodiac_mode);
    assert_eq!(converted.apparentness, request.apparentness);
    assert_eq!(converted.house_system, request.house_system);
    let summary = converted.summary_line();
    assert!(summary.contains("(TT);"));
    assert!(summary.contains("bodies=2;"));
    assert!(summary.contains("zodiac=Sidereal (Lahiri);"));
    assert!(summary.contains("apparentness=Apparent;"));
    assert!(summary.contains("observer=house-only;"));
    assert!(summary.contains(
        "house system=My TDB Custom Houses [aliases: My TDB Alias] (uses a local TDB calibration)"
    ));
}

#[test]
fn body_placement_exposes_motion_direction() {
    let mut result = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Mars,
        Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ),
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.motion = Some(pleiades_types::Motion::new(
        Some(-0.012),
        Some(0.004),
        Some(0.0012),
    ));

    let placement = BodyPlacement {
        body: CelestialBody::Mars,
        position: result,
        sign: None,
        house: None,
        apparent: None,
        topocentric: None,
    };

    assert_eq!(
        placement.motion_direction(),
        Some(MotionDirection::Retrograde)
    );
    assert_eq!(placement.longitude_speed(), Some(-0.012));
    assert_eq!(placement.latitude_speed(), Some(0.004));
    assert_eq!(placement.distance_speed(), Some(0.0012));
    assert_eq!(
        placement
            .motion()
            .and_then(|motion| motion.longitude_deg_per_day),
        Some(-0.012)
    );
}

#[test]
fn body_placement_treats_non_finite_motion_as_unknown() {
    let mut result = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Mars,
        Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ),
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.motion = Some(pleiades_types::Motion::new(
        Some(0.012),
        Some(f64::NAN),
        Some(0.0012),
    ));

    let placement = BodyPlacement {
        body: CelestialBody::Mars,
        position: result,
        sign: None,
        house: None,
        apparent: None,
        topocentric: None,
    };

    assert_eq!(placement.motion_direction(), None);
}

#[test]
fn motion_summary_validates_against_placement_count() {
    let summary = MotionSummary {
        direct: 2,
        stationary: 1,
        retrograde: 3,
        unknown: 0,
    };

    assert_eq!(summary.validate(6), Ok(()));
    assert_eq!(
        summary.summary_line(),
        "2 direct, 1 stationary, 3 retrograde, 0 unknown"
    );

    let error = MotionSummary {
        direct: 1,
        stationary: 1,
        retrograde: 0,
        unknown: 1,
    }
    .validate(4)
    .expect_err("mismatched motion summary counts should fail validation");

    assert_eq!(
        error,
        MotionSummaryValidationError::PlacementCountMismatch {
            expected: 4,
            actual: 3,
        }
    );
}

#[test]
fn body_placement_summary_line_matches_display() {
    let mut result = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Venus,
        Instant::new(
            pleiades_types::JulianDay::from_days(2451545.0),
            TimeScale::Tt,
        ),
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(123.25),
        Latitude::from_degrees(-5.5),
        Some(0.987),
    ));
    result.motion = Some(pleiades_types::Motion::new(
        Some(-0.012),
        Some(0.004),
        Some(0.0012),
    ));

    let placement = BodyPlacement {
        body: CelestialBody::Venus,
        position: result,
        sign: Some(ZodiacSign::Leo),
        house: Some(7),
        apparent: None,
        topocentric: None,
    };

    let summary = placement.summary_line();
    assert_eq!(summary, placement.to_string());
    assert!(summary.contains("Venus"));
    assert!(summary.contains("123.25°"));
    assert!(summary.contains("Leo"));
    assert!(summary.contains("7"));
    assert!(summary.contains("Retrograde"));
    assert!(summary.contains(&placement.position.quality.to_string()));
}

#[test]
fn chart_snapshot_supports_body_lookup_and_retrograde_summary() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let mut retrograde = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Mars,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    retrograde.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        Latitude::from_degrees(0.0),
        None,
    ));
    retrograde.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

    let mut direct = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Sun,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    direct.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(15.0),
        Latitude::from_degrees(0.0),
        None,
    ));
    direct.motion = Some(pleiades_types::Motion::new(Some(0.01), None, None));

    let mut stationary = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Mercury,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    stationary.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(45.0),
        Latitude::from_degrees(0.0),
        None,
    ));
    stationary.motion = Some(pleiades_types::Motion::new(Some(0.0), None, None));

    let mut unknown = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Jupiter,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    unknown.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(105.0),
        Latitude::from_degrees(0.0),
        None,
    ));

    let chart = ChartSnapshot {
        backend_id: BackendId::new("toy-chart"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Apparent,
        houses: None,
        placements: vec![
            BodyPlacement {
                body: CelestialBody::Sun,
                position: direct,
                sign: Some(ZodiacSign::Aries),
                house: Some(1),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Mercury,
                position: stationary,
                sign: Some(ZodiacSign::Taurus),
                house: Some(2),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Mars,
                position: retrograde,
                sign: Some(ZodiacSign::Cancer),
                house: Some(8),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Jupiter,
                position: unknown,
                sign: Some(ZodiacSign::Leo),
                house: Some(9),
                apparent: None,
                topocentric: None,
            },
        ],
    };

    assert!(chart.placement_for(&CelestialBody::Sun).is_some());
    assert_eq!(
        chart.sign_for_body(&CelestialBody::Sun),
        Some(ZodiacSign::Aries)
    );
    assert_eq!(
        chart.sign_for_body(&CelestialBody::Mars),
        Some(ZodiacSign::Cancer)
    );
    assert_eq!(chart.house_for_body(&CelestialBody::Sun), Some(1));
    assert_eq!(chart.house_for_body(&CelestialBody::Mars), Some(8));
    assert_eq!(chart.house_for_body(&CelestialBody::Mercury), Some(2));
    assert_eq!(
        chart.occupied_signs(),
        vec![
            ZodiacSign::Aries,
            ZodiacSign::Taurus,
            ZodiacSign::Cancer,
            ZodiacSign::Leo,
        ]
    );
    assert_eq!(chart.occupied_houses(), vec![1, 2, 8, 9]);
    assert_eq!(
        chart.motion_direction_for(&CelestialBody::Mars),
        Some(MotionDirection::Retrograde)
    );
    assert_eq!(chart.longitude_speed_for(&CelestialBody::Sun), Some(0.01));
    assert_eq!(chart.latitude_speed_for(&CelestialBody::Sun), None);
    assert_eq!(chart.distance_speed_for(&CelestialBody::Sun), None);
    assert_eq!(
        chart
            .motion_for_body(&CelestialBody::Mars)
            .and_then(|motion| motion.longitude_deg_per_day),
        Some(-0.01)
    );
    assert_eq!(
        chart
            .placements_in_sign(ZodiacSign::Cancer)
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Mars]
    );
    assert!(chart
        .placements_in_sign(ZodiacSign::Pisces)
        .next()
        .is_none());
    assert_eq!(
        chart
            .placements_in_house(2)
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Mercury]
    );
    assert_eq!(
        chart
            .placements_in_house(8)
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Mars]
    );
    assert!(chart.placements_in_house(12).next().is_none());
    assert_eq!(chart.stationary_placements().count(), 1);
    assert_eq!(chart.unknown_motion_placements().count(), 1);
    assert_eq!(chart.retrograde_placements().count(), 1);
    assert_eq!(
        chart.house_summary(),
        HouseSummary {
            first: 1,
            second: 1,
            third: 0,
            fourth: 0,
            fifth: 0,
            sixth: 0,
            seventh: 0,
            eighth: 1,
            ninth: 1,
            tenth: 0,
            eleventh: 0,
            twelfth: 0,
            unknown: 0,
        }
    );
    assert_eq!(chart.sign_summary().validate(4), Ok(()));
    assert_eq!(chart.house_summary().validate(4), Ok(()));
    assert_eq!(
        chart.sign_summary().validated_summary_line(4),
        Ok(chart.sign_summary().summary_line())
    );
    assert_eq!(
        chart.house_summary().validated_summary_line(4),
        Ok(chart.house_summary().summary_line())
    );
    assert_eq!(
        chart.motion_summary(),
        MotionSummary {
            direct: 1,
            stationary: 1,
            retrograde: 1,
            unknown: 1,
        }
    );
    assert_eq!(
        chart.motion_summary().summary_line(),
        chart.motion_summary().to_string()
    );
    assert_eq!(
        chart
            .placements_with_motion_direction(MotionDirection::Direct)
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Sun]
    );
    assert_eq!(
        chart
            .placements_with_motion_direction(MotionDirection::Stationary)
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Mercury]
    );
    assert_eq!(
        chart
            .direct_placements()
            .map(|placement| placement.body.clone())
            .collect::<Vec<_>>(),
        vec![CelestialBody::Sun]
    );
    let rendered = chart.to_string();
    assert!(rendered
        .contains("House summary: 1 in 1st house, 1 in 2nd house, 1 in 8th house, 1 in 9th house"));
    assert!(rendered.contains("Motion summary: 1 direct, 1 stationary, 1 retrograde, 1 unknown"));
    assert!(rendered.contains("Stationary bodies: Mercury"));
    assert!(rendered.contains("Unknown motion bodies: Jupiter"));
    assert!(rendered.contains("Retrograde bodies: Mars"));
}

#[test]
fn motion_summary_summary_line_matches_display_order() {
    let summary = MotionSummary {
        direct: 2,
        stationary: 1,
        retrograde: 3,
        unknown: 4,
    };

    assert_eq!(
        summary.summary_line(),
        "2 direct, 1 stationary, 3 retrograde, 4 unknown"
    );
    assert_eq!(summary.to_string(), summary.summary_line());
    assert!(summary.has_known_motion());
}

#[test]
fn sign_summary_validation_rejects_count_mismatch() {
    let summary = SignSummary {
        aries: 1,
        taurus: 1,
        gemini: 0,
        cancer: 0,
        leo: 0,
        virgo: 0,
        libra: 0,
        scorpio: 0,
        sagittarius: 0,
        capricorn: 0,
        aquarius: 0,
        pisces: 0,
    };

    assert_eq!(summary.validate(2), Ok(()));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line(2),
        Ok(summary.summary_line())
    );
    assert_eq!(
        summary.validate(3),
        Err(SignSummaryValidationError::PlacementCountMismatch {
            expected: 3,
            actual: 2
        })
    );
}

#[test]
fn house_summary_validation_rejects_count_mismatch() {
    let summary = HouseSummary {
        first: 1,
        second: 1,
        third: 0,
        fourth: 0,
        fifth: 0,
        sixth: 0,
        seventh: 0,
        eighth: 0,
        ninth: 0,
        tenth: 0,
        eleventh: 0,
        twelfth: 0,
        unknown: 1,
    };

    assert_eq!(summary.validate(3), Ok(()));
    assert_eq!(summary.summary_line(), summary.to_string());
    assert_eq!(
        summary.validated_summary_line(3),
        Ok(summary.summary_line())
    );
    assert_eq!(
        summary.validate(2),
        Err(HouseSummaryValidationError::PlacementCountMismatch {
            expected: 2,
            actual: 3
        })
    );
}

#[test]
fn aspect_summary_validation_rejects_count_mismatch() {
    let summary = AspectSummary {
        conjunction: 1,
        sextile: 0,
        square: 0,
        trine: 1,
        opposition: 0,
    };

    assert_eq!(summary.validate(2), Ok(()));
    assert_eq!(
        summary.validate(3),
        Err(AspectSummaryValidationError::PlacementCountMismatch {
            expected: 3,
            actual: 2
        })
    );
}

#[test]
fn chart_snapshot_exposes_major_aspects_and_angular_separation() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let mut sun = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Sun,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    sun.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(15.0),
        Latitude::from_degrees(0.0),
        None,
    ));

    let mut moon = EphemerisResult::new(
        BackendId::new("toy-chart"),
        CelestialBody::Moon,
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    moon.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(75.0),
        Latitude::from_degrees(0.0),
        None,
    ));

    let chart = ChartSnapshot {
        backend_id: BackendId::new("toy-chart"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Apparent,
        houses: None,
        placements: vec![
            BodyPlacement {
                body: CelestialBody::Sun,
                position: sun,
                sign: Some(ZodiacSign::Aries),
                house: Some(1),
                apparent: None,
                topocentric: None,
            },
            BodyPlacement {
                body: CelestialBody::Moon,
                position: moon,
                sign: Some(ZodiacSign::Taurus),
                house: Some(2),
                apparent: None,
                topocentric: None,
            },
        ],
    };

    assert_eq!(
        chart.angular_separation(&CelestialBody::Sun, &CelestialBody::Moon),
        Some(Angle::from_degrees(60.0))
    );

    let aspects = chart.major_aspects();
    assert_eq!(aspects.len(), 1);
    let aspect = &aspects[0];
    assert_eq!(aspect.left, CelestialBody::Sun);
    assert_eq!(aspect.right, CelestialBody::Moon);
    assert_eq!(aspect.kind, AspectKind::Sextile);
    assert_eq!(aspect.separation, Angle::from_degrees(60.0));
    assert_eq!(aspect.orb, Angle::from_degrees(0.0));
    assert_eq!(
        chart.aspect_summary(),
        AspectSummary {
            conjunction: 0,
            sextile: 1,
            square: 0,
            trine: 0,
            opposition: 0,
        }
    );
    assert_eq!(chart.aspect_summary().summary_line(), "1 Sextile");
    assert_eq!(chart.aspect_summary().to_string(), "1 Sextile");
    assert_eq!(chart.aspect_summary().validate(aspects.len()), Ok(()));
    let rendered = chart.to_string();
    assert!(rendered.contains("Aspect summary: 1 Sextile"));
    assert!(rendered.contains("Aspects:"));
    assert!(rendered.contains("Sun Sextile Moon"));
}

#[test]
fn chart_snapshot_renders_custom_body_identifiers() {
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2451545.0),
        TimeScale::Tt,
    );
    let custom_body =
        CelestialBody::Custom(pleiades_types::CustomBodyId::new("asteroid", "433-Eros"));
    let mut result = EphemerisResult::new(
        BackendId::new("toy-chart"),
        custom_body.clone(),
        instant,
        pleiades_types::CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.motion = Some(pleiades_types::Motion::new(Some(-0.01), None, None));

    let chart = ChartSnapshot {
        backend_id: BackendId::new("toy-chart"),
        instant,
        observer: None,
        body_observer: None,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Apparent,
        houses: None,
        placements: vec![BodyPlacement {
            body: custom_body,
            position: result,
            sign: None,
            house: None,
            apparent: None,
            topocentric: None,
        }],
    };

    let rendered = chart.to_string();
    assert!(rendered.contains("asteroid:433-Eros"));
    assert!(rendered.contains("Retrograde bodies: asteroid:433-Eros"));
}

#[test]
fn aspect_definition_summary_line_and_validation_remain_typed() {
    let definition = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    assert_eq!(
        definition.summary_line(),
        "kind=Sextile; exact_degrees=60°; orb_degrees=4°"
    );
    assert_eq!(definition.to_string(), definition.summary_line());
}

#[test]
fn aspect_definition_validation_rejects_non_finite_and_out_of_range_values() {
    let valid = AspectDefinition::new(AspectKind::Sextile, 60.0, 4.0);
    assert!(valid.validate().is_ok());
    assert!(validate_aspect_definitions(&[valid]).is_ok());

    let exact_nan = AspectDefinition::new(AspectKind::Conjunction, f64::NAN, 8.0);
    let exact_error = exact_nan
        .validate()
        .expect_err("NaN exact degrees should be rejected");
    assert!(exact_error
        .summary_line()
        .contains("exact_degrees must be finite and between 0 and 180 degrees"));
    assert_eq!(exact_error.to_string(), exact_error.summary_line());

    let orb_negative = AspectDefinition::new(AspectKind::Square, 90.0, -1.0);
    let orb_error = orb_negative
        .validate()
        .expect_err("negative orbs should be rejected");
    assert!(orb_error
        .summary_line()
        .contains("orb_degrees must be finite and between 0 and 180 degrees"));
    assert_eq!(orb_error.to_string(), orb_error.summary_line());

    let exact_out_of_range = AspectDefinition::new(AspectKind::Trine, 181.0, 6.0);
    assert!(exact_out_of_range
        .validate()
        .expect_err("angles outside the supported half-circle should be rejected")
        .summary_line()
        .contains("found 181"));

    assert!(validate_aspect_definitions(&[
        valid,
        AspectDefinition::new(AspectKind::Opposition, 180.0, 8.0),
    ])
    .is_ok());
}

#[test]
fn default_chart_applies_apparent_for_release_grade_body() {
    let engine = ChartEngine::new(ApparentChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun]);
    let snapshot = engine
        .chart(&request)
        .expect("default apparent chart should succeed");
    let placement = snapshot.placement_for(&CelestialBody::Sun).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    assert!(
        placement.apparent.is_some(),
        "apparent provenance should be attached"
    );
}

#[test]
fn non_release_grade_body_falls_back_to_mean() {
    let engine = ChartEngine::new(ConstrainedOnlyChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Moon]);
    let snapshot = engine
        .chart(&request)
        .expect("non-release-grade falls back, not errors");
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(
        placement.apparent.is_none(),
        "no apparent provenance on fallback"
    );
}

#[test]
fn explicit_mean_mode_returns_raw_j2000() {
    let engine = ChartEngine::new(ApparentChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Sun])
    .with_apparentness(Apparentness::Mean);
    let snapshot = engine.chart(&request).unwrap();
    let placement = snapshot.placement_for(&CelestialBody::Sun).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(placement.apparent.is_none());
}

#[test]
fn sidereal_apparent_chart_applies_ayanamsa_to_apparent_longitude() {
    // Regression guard for C1: sidereal charts with a release-grade body must
    // store the sidereal (ayanamsa-adjusted) apparent longitude, not the raw
    // tropical apparent longitude.
    //
    // ApparentChartBackend returns Sun at tropical 280° with distance_au=1.0
    // (release-grade claim). After the apparent-place correction the tropical
    // apparent longitude will be shifted slightly but will remain close to 280°.
    // After re-applying the Lahiri ayanamsa (~23.85° at J2000) the sidereal
    // apparent longitude must differ from the tropical apparent longitude by
    // approximately that ayanamsa offset.
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    );
    let zodiac_mode = ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    };

    // Build the apparent sidereal chart.
    let engine = ChartEngine::new(ApparentChartBackend);
    let request = ChartRequest::new(instant)
        .with_bodies(vec![CelestialBody::Sun])
        .with_zodiac_mode(zodiac_mode.clone());
    let snapshot = engine
        .chart(&request)
        .expect("sidereal apparent chart should succeed");
    let placement = snapshot.placement_for(&CelestialBody::Sun).unwrap();

    // The placement must have been corrected to apparent.
    assert_eq!(placement.position.apparent, Apparentness::Apparent);

    // Retrieve the stored longitude.
    let sidereal_apparent_lon = placement
        .position
        .ecliptic
        .expect("placement must have ecliptic coordinates")
        .longitude;

    // Build a mean (no apparent correction) tropical chart to get the raw
    // tropical longitude the backend serves, then derive what the expected
    // sidereal apparent longitude should be.
    let tropical_engine = ChartEngine::new(ApparentChartBackend);
    let tropical_request = ChartRequest::new(instant)
        .with_bodies(vec![CelestialBody::Sun])
        .with_apparentness(Apparentness::Apparent); // tropical, apparent
    let tropical_snapshot = tropical_engine
        .chart(&tropical_request)
        .expect("tropical apparent chart should succeed");
    let tropical_apparent_lon = tropical_snapshot
        .placement_for(&CelestialBody::Sun)
        .unwrap()
        .position
        .ecliptic
        .unwrap()
        .longitude;

    // The expected sidereal apparent longitude is the tropical apparent longitude
    // with the ayanamsa applied.
    let expected_sidereal = sidereal_longitude(tropical_apparent_lon, instant, &zodiac_mode)
        .expect("sidereal conversion of apparent longitude should succeed");

    // The stored longitude must match the ayanamsa-adjusted apparent longitude.
    assert!(
        (sidereal_apparent_lon.degrees() - expected_sidereal.degrees()).abs() < 1e-9,
        "sidereal apparent longitude {:.6}° must equal tropical apparent {:.6}° minus ayanamsa = {:.6}°",
        sidereal_apparent_lon.degrees(),
        tropical_apparent_lon.degrees(),
        expected_sidereal.degrees(),
    );

    // The offset between tropical apparent and sidereal apparent must be
    // approximately the Lahiri ayanamsa (~20-25° at J2000).
    let offset =
        (tropical_apparent_lon.degrees() - sidereal_apparent_lon.degrees()).rem_euclid(360.0);
    assert!(
        offset > 20.0 && offset < 30.0,
        "ayanamsa offset should be ~20-25° at J2000, got {offset:.4}°"
    );

    // The derived sign must be sidereal (Pisces for ~256° sidereal, not tropical Capricorn near 280°).
    assert_ne!(
        placement.sign,
        tropical_snapshot
            .placement_for(&CelestialBody::Sun)
            .unwrap()
            .sign,
        "sidereal and tropical apparent charts must produce different signs"
    );
}

fn sample_apparent_moon_chart(
    observer: pleiades_types::ObserverLocation,
    topocentric: bool,
) -> ChartSnapshot {
    let engine = ChartEngine::new(ApparentMoonChartBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Moon])
    .with_apparentness(Apparentness::Apparent)
    .with_observer(observer)
    .with_topocentric(topocentric);
    engine
        .chart(&request)
        .expect("apparent moon chart should succeed")
}

#[test]
fn topocentric_moon_differs_from_geocentric() {
    // Build a geocentric apparent Moon chart and a topocentric one for the same
    // instant/observer; the longitudes must differ by the lunar parallax (>0.1°).
    let observer = pleiades_types::ObserverLocation::new(
        pleiades_types::Latitude::from_degrees(40.0),
        pleiades_types::Longitude::from_degrees(-3.7),
        Some(650.0),
    );
    let geocentric = sample_apparent_moon_chart(observer.clone(), false);
    let topocentric = sample_apparent_moon_chart(observer, true);
    let geo_lon = geocentric.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();
    let topo_lon = topocentric.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();
    let mut diff = (topo_lon - geo_lon).abs();
    if diff > 180.0 {
        diff = 360.0 - diff;
    }
    assert!(
        diff > 0.1,
        "lunar parallax {diff}° too small (geo {geo_lon}, topo {topo_lon})"
    );
}

#[test]
fn sidereal_topocentric_applies_ayanamsa_once() {
    // Regression guard: in sidereal+topocentric mode the ayanamsa must be applied
    // EXACTLY ONCE to the topocentric tropical apparent longitude. Before this fix
    // the topocentric block received an already-sidereal longitude (wrong frame)
    // and then applied the ayanamsa a SECOND time (double subtraction).
    //
    // Two properties are asserted:
    //
    //   (A) Topocentric effect is real: the sidereal+topocentric Moon longitude
    //       differs from the sidereal+geocentric Moon longitude by > 0.1° (lunar
    //       parallax is measurable).
    //
    //   (B) Ayanamsa is applied exactly once: the sidereal+topocentric longitude
    //       equals (tropical+topocentric longitude − ayanamsa) to within 1 arcsec.
    //       Double subtraction would produce a value off by ~23° (the full ayanamsa).
    let instant = Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    );
    let observer = pleiades_types::ObserverLocation::new(
        pleiades_types::Latitude::from_degrees(40.0),
        pleiades_types::Longitude::from_degrees(-3.7),
        Some(650.0),
    );
    let zodiac_mode = ZodiacMode::Sidereal {
        ayanamsa: crate::Ayanamsa::Lahiri,
    };

    // --- (A) Topocentric parallax is applied in sidereal mode ------------------

    // Sidereal geocentric apparent Moon.
    let geo_sidereal = {
        let engine = ChartEngine::new(ApparentMoonChartBackend);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent)
            .with_observer(observer.clone())
            .with_zodiac_mode(zodiac_mode.clone());
        engine
            .chart(&request)
            .expect("sidereal geocentric apparent Moon chart should succeed")
    };

    // Sidereal topocentric apparent Moon.
    let topo_sidereal = {
        let engine = ChartEngine::new(ApparentMoonChartBackend);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent)
            .with_observer(observer.clone())
            .with_zodiac_mode(zodiac_mode.clone())
            .with_topocentric(true);
        engine
            .chart(&request)
            .expect("sidereal topocentric apparent Moon chart should succeed")
    };

    let geo_sid_lon = geo_sidereal.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();
    let topo_sid_lon = topo_sidereal.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude
        .degrees();

    let mut sid_parallax = (topo_sid_lon - geo_sid_lon).abs();
    if sid_parallax > 180.0 {
        sid_parallax = 360.0 - sid_parallax;
    }
    assert!(
        sid_parallax > 0.1,
        "sidereal+topocentric Moon parallax {sid_parallax}° too small \
         (geo_sid={geo_sid_lon:.4}°, topo_sid={topo_sid_lon:.4}°) — \
         topocentric correction not applied in sidereal mode"
    );

    // --- (B) Ayanamsa is applied exactly once -----------------------------------

    // Tropical topocentric apparent Moon (same observer/instant, no sidereal).
    let topo_tropical = {
        let engine = ChartEngine::new(ApparentMoonChartBackend);
        let request = ChartRequest::new(instant)
            .with_bodies(vec![CelestialBody::Moon])
            .with_apparentness(Apparentness::Apparent)
            .with_observer(observer.clone())
            .with_topocentric(true);
        engine
            .chart(&request)
            .expect("tropical topocentric apparent Moon chart should succeed")
    };

    let topo_trop_lon = topo_tropical.placements[0]
        .position
        .ecliptic
        .as_ref()
        .unwrap()
        .longitude;

    // The expected sidereal+topocentric longitude is: tropical+topocentric − ayanamsa (once).
    let expected_sid_topo = sidereal_longitude(topo_trop_lon, instant, &zodiac_mode)
        .expect("sidereal conversion of tropical topocentric longitude should succeed");

    // Allow a generous 2 arcsec tolerance for floating-point rounding.
    let tol_deg = 2.0 / 3600.0;
    let err = (topo_sid_lon - expected_sid_topo.degrees())
        .abs()
        .min((topo_sid_lon - expected_sid_topo.degrees() + 360.0).abs())
        .min((topo_sid_lon - expected_sid_topo.degrees() - 360.0).abs());
    assert!(
        err < tol_deg,
        "sidereal+topocentric longitude {topo_sid_lon:.6}° must equal \
         tropical+topocentric {:.6}° − ayanamsa = {:.6}° (err {:.4} arcsec); \
         a large error (~23°) indicates double ayanamsa subtraction",
        topo_trop_lon.degrees(),
        expected_sid_topo.degrees(),
        err * 3600.0,
    );
}

#[test]
fn release_grade_body_falls_back_to_mean_when_apparent_unavailable() {
    // Regression guard: when apparent_position() fails for a release-grade body
    // (here because the backend returns an absurd 50,000 AU distance that trips
    // the light-time sanity cap), the engine must fall back gracefully to the
    // mean position rather than propagating the error. The chart must succeed,
    // and the placement must carry Apparentness::Mean with no apparent provenance.
    //
    // This covers the 433-Eros scenario where the packaged distance channel is
    // unreliable and the light-time iterator would otherwise diverge or converge
    // to a physically implausible value.
    let engine = ChartEngine::new(AbsurdDistanceReleaseGradeBackend);
    let request = ChartRequest::new(Instant::new(
        pleiades_types::JulianDay::from_days(2_451_545.0),
        TimeScale::Tt,
    ))
    .with_bodies(vec![CelestialBody::Mars])
    .with_apparentness(Apparentness::Apparent);

    let snapshot = engine.chart(&request).expect(
        "chart must succeed even when apparent-place computation fails for a release-grade body",
    );

    let placement = snapshot
        .placement_for(&CelestialBody::Mars)
        .expect("Mars must be present in the chart");

    assert_eq!(
        placement.position.apparent,
        Apparentness::Mean,
        "release-grade body with unavailable apparent must fall back to Mean"
    );
    assert!(
        placement.apparent.is_none(),
        "no apparent provenance must be attached on mean fallback"
    );
}
