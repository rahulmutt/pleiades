use crate::*;
use std::time::Duration;

#[test]
fn ephemeris_request_has_a_compact_display() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let request = EphemerisRequest::new(CelestialBody::Mars, instant);
    let request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            None,
        )),
        ..request
    };

    assert_eq!(request.to_string(), request.summary_line());
    assert_eq!(
            request.summary_line(),
            "body=Mars; instant=JD 2451545 TT; frame=Ecliptic; zodiac=Tropical; apparent=Mean; observer=latitude=51.5°, longitude=359.9°, elevation=n/a"
        );
    assert!(request.summary_line().contains("body=Mars"));
    assert!(request.summary_line().contains("observer="));
}

#[test]
fn request_policy_helpers_reject_unsupported_shapes() {
    let time_scale_request = EphemerisRequest {
        body: CelestialBody::Sun,
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        frame: CoordinateFrame::Equatorial,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Apparent,
    };
    let conversion = TimeScaleConversion::new(TimeScale::Utc, TimeScale::Tt, 64.184);
    assert!(time_scale_request
        .validate_time_scale_conversion(conversion)
        .is_ok());
    let converted = time_scale_request
        .clone()
        .with_time_scale_conversion(conversion)
        .expect("UTC request should convert with the caller-supplied policy");
    assert_eq!(converted.instant.scale, TimeScale::Tt);
    assert_eq!(converted.body, time_scale_request.body);
    assert_eq!(converted.observer, time_scale_request.observer);
    assert_eq!(converted.frame, time_scale_request.frame);
    assert_eq!(converted.zodiac_mode, time_scale_request.zodiac_mode);
    assert_eq!(converted.apparent, time_scale_request.apparent);

    let checked_offset = time_scale_request
        .clone()
        .with_instant_time_scale_offset_checked(TimeScale::Tt, 64.184)
        .expect("UTC request should accept the checked offset helper");
    assert_eq!(checked_offset.instant.scale, TimeScale::Tt);
    assert_eq!(checked_offset.body, time_scale_request.body);
    assert_eq!(checked_offset.observer, time_scale_request.observer);
    assert_eq!(checked_offset.frame, time_scale_request.frame);
    assert_eq!(checked_offset.zodiac_mode, time_scale_request.zodiac_mode);
    assert_eq!(checked_offset.apparent, time_scale_request.apparent);

    let tt_from_tdb_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        ..time_scale_request.clone()
    };
    let tt_from_tdb = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb(-0.001_657)
        .expect("TDB request should convert back to TT with a caller-supplied offset");
    assert_eq!(tt_from_tdb.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb.frame, tt_from_tdb_request.frame);
    assert_eq!(tt_from_tdb.zodiac_mode, tt_from_tdb_request.zodiac_mode);
    assert_eq!(tt_from_tdb.apparent, tt_from_tdb_request.apparent);

    let tt_from_tdb_signed = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb_signed(-0.001_657)
        .expect("TDB request should convert back to TT with a signed offset");
    assert_eq!(tt_from_tdb_signed.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb_signed.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb_signed.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb_signed.frame, tt_from_tdb_request.frame);
    assert_eq!(
        tt_from_tdb_signed.zodiac_mode,
        tt_from_tdb_request.zodiac_mode
    );
    assert_eq!(tt_from_tdb_signed.apparent, tt_from_tdb_request.apparent);

    let tt_from_tdb_unsigned = tt_from_tdb_request
        .clone()
        .with_tt_from_tdb(0.001_657)
        .expect("TDB request should convert back to TT with a duration offset");
    assert_eq!(tt_from_tdb_unsigned.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_tdb_unsigned.body, tt_from_tdb_request.body);
    assert_eq!(tt_from_tdb_unsigned.observer, tt_from_tdb_request.observer);
    assert_eq!(tt_from_tdb_unsigned.frame, tt_from_tdb_request.frame);
    assert_eq!(
        tt_from_tdb_unsigned.zodiac_mode,
        tt_from_tdb_request.zodiac_mode
    );
    assert_eq!(tt_from_tdb_unsigned.apparent, tt_from_tdb_request.apparent);

    let tt_from_ut1_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
        ..time_scale_request.clone()
    };
    let tt_from_ut1 = tt_from_ut1_request
        .clone()
        .with_tt_from_ut1_signed(64.184)
        .expect("UT1 request should convert to TT with a signed offset");
    assert_eq!(tt_from_ut1.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_ut1.body, tt_from_ut1_request.body);
    assert_eq!(tt_from_ut1.observer, tt_from_ut1_request.observer);
    assert_eq!(tt_from_ut1.frame, tt_from_ut1_request.frame);
    assert_eq!(tt_from_ut1.zodiac_mode, tt_from_ut1_request.zodiac_mode);
    assert_eq!(tt_from_ut1.apparent, tt_from_ut1_request.apparent);

    let tt_from_ut1_unsigned = tt_from_ut1_request
        .clone()
        .with_tt_from_ut1(Duration::from_secs_f64(64.184))
        .expect("UT1 request should convert to TT with a duration offset");
    assert_eq!(tt_from_ut1_unsigned.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_ut1_unsigned.body, tt_from_ut1_request.body);
    assert_eq!(tt_from_ut1_unsigned.observer, tt_from_ut1_request.observer);
    assert_eq!(tt_from_ut1_unsigned.frame, tt_from_ut1_request.frame);
    assert_eq!(
        tt_from_ut1_unsigned.zodiac_mode,
        tt_from_ut1_request.zodiac_mode
    );
    assert_eq!(tt_from_ut1_unsigned.apparent, tt_from_ut1_request.apparent);

    let tt_from_utc_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        ..time_scale_request.clone()
    };
    let tt_from_utc = tt_from_utc_request
        .clone()
        .with_tt_from_utc_signed(64.184)
        .expect("UTC request should convert to TT with a signed offset");
    assert_eq!(tt_from_utc.instant.scale, TimeScale::Tt);
    assert_eq!(tt_from_utc.body, tt_from_utc_request.body);
    assert_eq!(tt_from_utc.observer, tt_from_utc_request.observer);
    assert_eq!(tt_from_utc.frame, tt_from_utc_request.frame);
    assert_eq!(tt_from_utc.zodiac_mode, tt_from_utc_request.zodiac_mode);
    assert_eq!(tt_from_utc.apparent, tt_from_utc_request.apparent);

    let tdb_from_tt_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ..time_scale_request.clone()
    };
    let tdb_from_tt = tdb_from_tt_request
        .clone()
        .with_tdb_from_tt_signed(-0.001_657)
        .expect("TT request should convert to TDB with a signed offset");
    assert_eq!(tdb_from_tt.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_tt.body, tdb_from_tt_request.body);
    assert_eq!(tdb_from_tt.observer, tdb_from_tt_request.observer);
    assert_eq!(tdb_from_tt.frame, tdb_from_tt_request.frame);
    assert_eq!(tdb_from_tt.zodiac_mode, tdb_from_tt_request.zodiac_mode);
    assert_eq!(tdb_from_tt.apparent, tdb_from_tt_request.apparent);

    let tdb_from_tt_unsigned = tdb_from_tt_request
        .clone()
        .with_tdb_from_tt(Duration::from_secs_f64(0.001_657))
        .expect("TT request should convert to TDB with a duration offset");
    assert_eq!(tdb_from_tt_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_tt_unsigned.body, tdb_from_tt_request.body);
    assert_eq!(tdb_from_tt_unsigned.observer, tdb_from_tt_request.observer);
    assert_eq!(tdb_from_tt_unsigned.frame, tdb_from_tt_request.frame);
    assert_eq!(
        tdb_from_tt_unsigned.zodiac_mode,
        tdb_from_tt_request.zodiac_mode
    );
    assert_eq!(tdb_from_tt_unsigned.apparent, tdb_from_tt_request.apparent);

    let tdb_from_ut1_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Ut1),
        ..time_scale_request.clone()
    };
    let tdb_from_ut1 = tdb_from_ut1_request
        .clone()
        .with_tdb_from_ut1_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UT1 request should convert to TDB with caller-supplied offsets");
    assert_eq!(tdb_from_ut1.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_ut1.body, tdb_from_ut1_request.body);
    assert_eq!(tdb_from_ut1.observer, tdb_from_ut1_request.observer);
    assert_eq!(tdb_from_ut1.frame, tdb_from_ut1_request.frame);
    assert_eq!(tdb_from_ut1.zodiac_mode, tdb_from_ut1_request.zodiac_mode);
    assert_eq!(tdb_from_ut1.apparent, tdb_from_ut1_request.apparent);

    let tdb_from_ut1_unsigned = tdb_from_ut1_request
        .clone()
        .with_tdb_from_ut1(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UT1 request should convert to TDB with duration offsets");
    assert_eq!(tdb_from_ut1_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_ut1_unsigned.body, tdb_from_ut1_request.body);
    assert_eq!(
        tdb_from_ut1_unsigned.observer,
        tdb_from_ut1_request.observer
    );
    assert_eq!(tdb_from_ut1_unsigned.frame, tdb_from_ut1_request.frame);
    assert_eq!(
        tdb_from_ut1_unsigned.zodiac_mode,
        tdb_from_ut1_request.zodiac_mode
    );
    assert_eq!(
        tdb_from_ut1_unsigned.apparent,
        tdb_from_ut1_request.apparent
    );

    let tdb_from_utc_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Utc),
        ..time_scale_request.clone()
    };
    let tdb_from_utc = tdb_from_utc_request
        .clone()
        .with_tdb_from_utc_signed(Duration::from_secs_f64(64.184), -0.001_657)
        .expect("UTC request should convert to TDB with caller-supplied offsets");
    assert_eq!(tdb_from_utc.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_utc.body, tdb_from_utc_request.body);
    assert_eq!(tdb_from_utc.observer, tdb_from_utc_request.observer);
    assert_eq!(tdb_from_utc.frame, tdb_from_utc_request.frame);
    assert_eq!(tdb_from_utc.zodiac_mode, tdb_from_utc_request.zodiac_mode);
    assert_eq!(tdb_from_utc.apparent, tdb_from_utc_request.apparent);

    let tdb_from_utc_unsigned = tdb_from_utc_request
        .clone()
        .with_tdb_from_utc(
            Duration::from_secs_f64(64.184),
            Duration::from_secs_f64(0.001_657),
        )
        .expect("UTC request should convert to TDB with duration offsets");
    assert_eq!(tdb_from_utc_unsigned.instant.scale, TimeScale::Tdb);
    assert_eq!(tdb_from_utc_unsigned.body, tdb_from_utc_request.body);
    assert_eq!(
        tdb_from_utc_unsigned.observer,
        tdb_from_utc_request.observer
    );
    assert_eq!(tdb_from_utc_unsigned.frame, tdb_from_utc_request.frame);
    assert_eq!(
        tdb_from_utc_unsigned.zodiac_mode,
        tdb_from_utc_request.zodiac_mode
    );
    assert_eq!(
        tdb_from_utc_unsigned.apparent,
        tdb_from_utc_request.apparent
    );

    let error = time_scale_request
        .clone()
        .with_instant_time_scale_offset_checked(TimeScale::Tt, f64::NAN)
        .expect_err("non-finite offsets should be rejected at the request layer");
    assert_eq!(error, TimeScaleConversionError::NonFiniteOffset);

    let error = time_scale_request
        .validate_time_scale_conversion(TimeScaleConversion::new(
            TimeScale::Tt,
            TimeScale::Tt,
            64.184,
        ))
        .expect_err("mismatched source scales should fail validation before retagging");
    assert_eq!(
        error,
        TimeScaleConversionError::Expected {
            expected: TimeScale::Tt,
            actual: TimeScale::Utc
        }
    );
    let error = validate_request_policy(
        &time_scale_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("UTC should be rejected when only TT is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "toy backend expects one of [TT] for request instants"
    );

    let frame_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        observer: None,
        frame: CoordinateFrame::Equatorial,
        apparent: Apparentness::Mean,
        ..time_scale_request.clone()
    };
    let error = validate_request_policy(
        &frame_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("equatorial frame should be rejected when only ecliptic is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedCoordinateFrame);
    assert_eq!(
        error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let apparent_request = EphemerisRequest {
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Apparent,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_policy(
        &apparent_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        true,
        false,
    )
    .expect_err("apparent requests should be rejected when only mean output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let invalid_observer_apparent_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let unsupported_apparent_metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: true,
            batch: true,
            native_sidereal: false,
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let error = validate_request_against_metadata(
        &invalid_observer_apparent_request,
        &unsupported_apparent_metadata,
    )
    .expect_err("unsupported apparentness should be reported before observer validation");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let batch_seed_request = EphemerisRequest {
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let batch_error = validate_requests_against_metadata(
            &[batch_seed_request, invalid_observer_apparent_request.clone()],
            &unsupported_apparent_metadata,
        )
        .expect_err("batch validation should report unsupported apparentness before invalid observer validation");
    assert_eq!(
        batch_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(
            batch_error.message,
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let mean_request = EphemerisRequest {
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let error = validate_request_policy(
        &mean_request,
        "toy backend",
        &[TimeScale::Tt],
        &[CoordinateFrame::Ecliptic],
        false,
        true,
    )
    .expect_err("mean requests should be rejected when only apparent output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "toy backend currently returns apparent coordinates only; mean geometric coordinates are not implemented"
        );

    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities {
            geocentric: true,
            topocentric: false,
            apparent: false,
            mean: true,
            batch: true,
            native_sidereal: false,
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };

    let frame_error = validate_request_against_metadata(&frame_request, &metadata)
        .expect_err("equatorial requests should still be rejected through metadata preflight");
    assert_eq!(
        frame_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        frame_error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let metadata_frame_error = metadata
        .validate_request(&frame_request)
        .expect_err("metadata request validation should match the shared preflight");
    assert_eq!(metadata_frame_error.kind, frame_error.kind);
    assert_eq!(metadata_frame_error.message, frame_error.message);

    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        apparent: Apparentness::Mean,
        ..apparent_request.clone()
    };
    let error = validate_request_against_metadata(&topocentric_request, &metadata)
        .expect_err("metadata preflight should reject observer-bearing geocentric requests");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error.message.contains("toy backend is geocentric only"));
    assert!(error.message.contains(
        &topocentric_request
            .observer
            .as_ref()
            .unwrap()
            .summary_line()
    ));

    let invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let routing_metadata = BackendMetadata {
        family: BackendFamily::Composite,
        capabilities: BackendCapabilities {
            topocentric: true,
            ..metadata.capabilities.clone()
        },
        ..metadata.clone()
    };
    let routing_error =
        validate_request_against_metadata(&invalid_observer_request, &routing_metadata)
            .expect_err("routing metadata should still reject invalid observer locations");
    assert_eq!(routing_error.kind, EphemerisErrorKind::InvalidObserver);
    assert!(routing_error
        .message
        .contains("request received invalid observer location"));

    let geocentric_only_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let geocentric_only_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_only_request.clone()
    };
    let geocentric_only_error = validate_request_against_metadata(
            &geocentric_only_invalid_observer_request,
            &metadata,
        )
        .expect_err("geocentric-only metadata should still reject malformed observer locations as invalid input");
    assert_eq!(
        geocentric_only_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(geocentric_only_error
        .message
        .contains("request received invalid observer location"));

    let geocentric_only_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            geocentric_only_invalid_observer_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve invalid observer precedence for geocentric-only requests",
    );
    assert_eq!(
        geocentric_only_batch_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(geocentric_only_batch_error
        .message
        .contains("batch request 2: request received invalid observer location"));

    let invalid_observer_frame_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..frame_request.clone()
    };
    let invalid_observer_frame_error =
        validate_request_against_metadata(&invalid_observer_frame_request, &metadata)
            .expect_err("frame policy should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_frame_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        invalid_observer_frame_error.message,
        "toy backend only returns [Ecliptic] coordinates"
    );

    let invalid_observer_time_scale_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_only_request.clone()
    };
    let invalid_observer_time_scale_error =
        validate_request_against_metadata(&invalid_observer_time_scale_request, &metadata)
            .expect_err("time-scale policy should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_time_scale_error.kind,
        EphemerisErrorKind::UnsupportedTimeScale
    );
    assert_eq!(
        invalid_observer_time_scale_error.message,
        "toy backend expects one of [TT] for request instants"
    );

    let invalid_observer_apparent_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let invalid_observer_apparent_error =
        validate_request_against_metadata(&invalid_observer_apparent_request, &metadata)
            .expect_err("apparentness should still win before malformed observer validation");
    assert_eq!(
        invalid_observer_apparent_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(
            invalid_observer_apparent_error.message,
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );

    let invalid_observer_frame_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            invalid_observer_frame_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve frame precedence before invalid observer validation",
    );
    assert_eq!(
        invalid_observer_frame_batch_error.kind,
        EphemerisErrorKind::UnsupportedCoordinateFrame
    );
    assert_eq!(
        invalid_observer_frame_batch_error.message,
        "batch request 2: toy backend only returns [Ecliptic] coordinates"
    );

    let unsupported_body_request = EphemerisRequest {
        body: CelestialBody::Mars,
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Mean,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_against_metadata(&unsupported_body_request, &metadata)
        .expect_err("metadata preflight should reject bodies outside the declared coverage");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(error.message, "toy backend does not support Mars");

    let unsupported_body_with_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..unsupported_body_request.clone()
    };
    let unsupported_body_with_invalid_observer_error = validate_request_against_metadata(
        &unsupported_body_with_invalid_observer_request,
        &metadata,
    )
    .expect_err("observer validation should win before unsupported body coverage is reported");
    assert_eq!(
        unsupported_body_with_invalid_observer_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(unsupported_body_with_invalid_observer_error
        .message
        .contains("request received invalid observer location"));

    let unsupported_body_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            unsupported_body_with_invalid_observer_request.clone(),
        ],
        &metadata,
    )
    .expect_err(
        "batch metadata should preserve invalid observer precedence over unsupported bodies",
    );
    assert_eq!(
        unsupported_body_batch_error.kind,
        EphemerisErrorKind::InvalidObserver
    );
    assert!(unsupported_body_batch_error
        .message
        .contains("batch request 2: request received invalid observer location"));

    let sidereal_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Mean,
        observer: None,
        ..frame_request.clone()
    };
    let error = validate_request_against_metadata(&sidereal_request, &metadata)
        .expect_err("sidereal requests should be rejected when metadata stays tropical-only");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));

    let sidereal_invalid_observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..sidereal_request.clone()
    };
    let error = validate_request_against_metadata(&sidereal_invalid_observer_request, &metadata)
        .expect_err("sidereal routing should win before malformed observer validation");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));
    assert!(!error.message.contains("invalid observer location"));

    let unsupported_body_apparent_request = EphemerisRequest {
        body: CelestialBody::Mars,
        frame: CoordinateFrame::Ecliptic,
        apparent: Apparentness::Apparent,
        ..frame_request.clone()
    };
    let unsupported_body_apparent_error =
        validate_request_against_metadata(&unsupported_body_apparent_request, &metadata)
            .expect_err("apparentness should win before unsupported body coverage");
    assert_eq!(
        unsupported_body_apparent_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(unsupported_body_apparent_error.message, "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented");

    let unsupported_body_apparent_batch_error = validate_requests_against_metadata(
        &[
            geocentric_only_request.clone(),
            unsupported_body_apparent_request.clone(),
        ],
        &metadata,
    )
    .expect_err("batch validation should preserve apparentness precedence before body coverage");
    assert_eq!(
        unsupported_body_apparent_batch_error.kind,
        EphemerisErrorKind::UnsupportedApparentness
    );
    assert_eq!(unsupported_body_apparent_batch_error.message, "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented");

    let sidereal_metadata = BackendMetadata {
        capabilities: BackendCapabilities {
            native_sidereal: true,
            ..metadata.capabilities.clone()
        },
        ..metadata.clone()
    };
    assert!(validate_request_against_metadata(&sidereal_request, &sidereal_metadata).is_ok());

    let invalid_custom_body_request = EphemerisRequest {
        body: CelestialBody::Custom(pleiades_types::CustomBodyId::new("asteroid", " 433-Eros ")),
        ..frame_request.clone()
    };
    let invalid_custom_body_error = validate_request_against_metadata(
        &invalid_custom_body_request,
        &BackendMetadata {
            body_claims: vec![CelestialBody::Custom(pleiades_types::CustomBodyId::new(
                "asteroid",
                " 433-Eros ",
            ))
            .into()],
            ..metadata.clone()
        },
    )
    .expect_err("custom body identifiers should validate before metadata dispatch");
    assert_eq!(
        invalid_custom_body_error.kind,
        EphemerisErrorKind::InvalidRequest
    );
    assert!(invalid_custom_body_error
            .message
            .contains("request body is invalid: custom body id designation must not have leading or trailing whitespace"));

    let invalid_custom_ayanamsa_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::Custom(pleiades_types::CustomAyanamsa {
                name: "  ".to_string(),
                description: Some("local calibration".to_string()),
                epoch: Some(pleiades_types::JulianDay::from_days(2451545.0)),
                offset_degrees: Some(pleiades_types::Angle::from_degrees(24.0)),
            }),
        },
        ..frame_request.clone()
    };
    let invalid_custom_ayanamsa_error =
        validate_request_against_metadata(&invalid_custom_ayanamsa_request, &sidereal_metadata)
            .expect_err("custom ayanamsas should validate before sidereal request dispatch");
    assert_eq!(
        invalid_custom_ayanamsa_error.kind,
        EphemerisErrorKind::InvalidRequest
    );
    assert!(invalid_custom_ayanamsa_error
        .message
        .contains("sidereal ayanamsa is invalid: custom ayanamsa name must not be blank"));

    let error = validate_zodiac_policy(&sidereal_request, "toy backend", &[ZodiacMode::Tropical])
        .expect_err("sidereal requests should be rejected when only tropical output is supported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert!(error.message.contains("tropical coordinates only"));
    let request_policy = current_request_policy_summary();
    assert_eq!(
            request_policy.time_scale,
            "direct backend requests accept TT/TDB; civil UTC/UT1 inputs convert via the pleiades-time crate or caller-supplied offsets; the ephemeris backends carry no internal Delta T or UTC convenience model"
        );
    assert_eq!(
            request_policy.observer,
            "chart houses use observer locations; chart body observers stay separate; body requests stay geocentric; geocentric-only backends reject observer-bearing requests with UnsupportedObserver; malformed observer coordinates remain InvalidObserver; topocentric body positions remain unsupported"
        );
    assert_eq!(
            request_policy.apparentness,
            "backends remain mean-only and J2000 at the backend boundary; apparent place of date (chart layer, default): light-time + precession-to-date + annual aberration + nutation-in-longitude, release-grade bodies; gravitational light-deflection omitted"
        );
    assert_eq!(
            request_policy.frame,
            "ecliptic body positions are the default request shape; equatorial output is backend-specific and derived via mean-obliquity transforms when supported; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"
        );
    assert_eq!(
        time_scale_policy_summary_for_report().summary_line(),
        request_policy.time_scale
    );
    assert_eq!(
        observer_policy_summary_for_report().summary_line(),
        request_policy.observer
    );
    assert_eq!(
        apparentness_policy_summary_for_report().summary_line(),
        request_policy.apparentness
    );
    assert_eq!(frame_policy_summary_for_report(), request_policy.frame);
    assert_eq!(
        zodiac_policy_summary_for_report(&[ZodiacMode::Tropical]),
        "tropical only"
    );

    let observer_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..apparent_request.clone()
    };
    let error = validate_observer_policy(&observer_request, "toy backend", false)
        .expect_err("topocentric requests should be rejected when unsupported");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error.message.contains("toy backend is geocentric only"));
    assert!(error
        .message
        .contains(&observer_request.observer.as_ref().unwrap().summary_line()));
}
