use crate::*;

#[test]
fn ephemeris_result_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Sun,
        instant,
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Mean,
    );
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(12.5),
        Latitude::from_degrees(-3.25),
        Some(1.234),
    ));
    result.equatorial = Some(EquatorialCoordinates::new(
        Angle::from_degrees(98.0),
        Latitude::from_degrees(0.5),
        None,
    ));
    result.motion = Some(Motion::new(Some(0.1), Some(-0.2), Some(0.003)));
    result.quality = QualityAnnotation::Exact;

    assert_eq!(result.to_string(), result.summary_line());
    assert_eq!(
            result.summary_line(),
            "backend=toy; body=Sun; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; quality=Exact; ecliptic=longitude=12.5°, latitude=-3.25°, distance=1.234 AU; equatorial=right_ascension=98°, declination=0.5°, distance=n/a; motion=longitude_speed=0.1 deg/day, latitude_speed=-0.2 deg/day, distance_speed=0.003 AU/day"
        );
    assert!(result.summary_line().contains("backend=toy"));
    assert!(result.summary_line().contains("quality=Exact"));
    assert!(result.summary_line().contains("ecliptic=longitude=12.5°"));
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip_preserves_requests_and_results() {
    let request = EphemerisRequest {
        body: CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        },
        apparent: Apparentness::Mean,
    };
    let request_roundtrip: EphemerisRequest =
        serde_json::from_value(serde_json::to_value(&request).expect("request should serialize"))
            .expect("request should deserialize");
    assert_eq!(request_roundtrip, request);

    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Apparent,
    );
    result.quality = QualityAnnotation::Interpolated;
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(123.0),
        Latitude::from_degrees(2.5),
        Some(1.0),
    ));
    result.motion = Some(Motion::new(Some(0.12), Some(-0.01), None));

    let result_roundtrip: EphemerisResult =
        serde_json::from_value(serde_json::to_value(&result).expect("result should serialize"))
            .expect("result should deserialize");
    assert_eq!(result_roundtrip, result);
}

#[test]
fn ephemeris_result_validation_rejects_invalid_coordinate_and_motion_samples() {
    let mut result = EphemerisResult::new(
        BackendId::new("toy"),
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Mean,
    );
    result.ecliptic = Some(EclipticCoordinates::new(
        Longitude::from_degrees(12.5),
        Latitude::from_degrees(2.5),
        Some(1.0),
    ));
    result.equatorial = Some(EquatorialCoordinates::new(
        Angle::from_degrees(f64::NAN),
        Latitude::from_degrees(1.0),
        Some(1.0),
    ));
    result.motion = Some(Motion::new(Some(f64::INFINITY), None, None));

    let error = result
        .validate()
        .expect_err("invalid equatorial coordinates should fail validation");
    assert!(matches!(
        error,
        EphemerisResultValidationError::InvalidEquatorial(
            CoordinateValidationError::NonFiniteValue {
                coordinate: "equatorial",
                field: "right_ascension",
                value,
            }
        ) if value.is_nan()
    ));
    assert!(error
            .to_string()
            .contains("backend result equatorial is invalid: equatorial coordinate field `right_ascension` must be finite"));

    result.equatorial = None;
    let error = result
        .validated_summary_line()
        .expect_err("invalid motion should fail validation");
    assert!(matches!(
        error,
        EphemerisResultValidationError::InvalidMotion(MotionValidationError::NonFiniteSpeed {
            field: "longitude_deg_per_day",
            value,
        }) if value.is_infinite()
    ));
    assert!(error.to_string().contains(
        "backend result motion is invalid: motion field `longitude_deg_per_day` must be finite"
    ));
}
