use crate::*;

#[test]
fn validate_requests_against_metadata_rejects_unsupported_batch_backends() {
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
            batch: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );

    let error = validate_requests_against_metadata(&[request], &metadata).expect_err(
        "batch requests should be rejected when the backend does not advertise batch support",
    );
    assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    assert_eq!(error.message, "toy backend does not support batch requests");
}

#[test]
fn validate_requests_against_metadata_preserves_mixed_time_scales_and_topocentric_requests_when_supported(
) {
    struct EchoSunBackend;

    impl EphemerisBackend for EchoSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("echo-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("echo Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![CelestialBody::Sun.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: true,
                    apparent: true,
                    topocentric: true,
                    ..BackendCapabilities::default()
                },
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Sun
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("echo-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = EchoSunBackend;
    let metadata = backend.metadata();

    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let mut topocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
    );
    topocentric_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));

    validate_requests_against_metadata(
        &[geocentric_request.clone(), topocentric_request.clone()],
        &metadata,
    )
    .expect("batch preflight should preserve mixed TT/TDB and topocentric requests when supported");

    let results = backend
        .positions(&[geocentric_request, topocentric_request])
        .expect("batch adapter should preserve the validated request shapes");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "echo-sun");
    assert_eq!(results[1].backend_id.as_str(), "echo-sun");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_with_batch_index_prefix() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let tropical_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        ..tropical_request.clone()
    };

    let error =
        validate_requests_against_metadata(&[tropical_request, sidereal_request], &metadata)
            .expect_err("the batch helper should preserve sidereal request failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_with_batch_index_prefix() {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let mean_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        ..mean_request.clone()
    };

    let error = validate_requests_against_metadata(&[mean_request, apparent_request], &metadata)
        .expect_err("the batch helper should preserve apparentness failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(
            error.message,
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        );
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_topocentric_checks() {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        )
    };

    let error = validate_requests_against_metadata(&[request], &metadata)
        .expect_err("apparentness should be checked before the observer policy");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_topocentric_checks_in_batches(
) {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_topocentric_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, apparent_topocentric_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve apparentness failures before observer checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_sidereal_checks_in_batches()
{
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_sidereal_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, apparent_sidereal_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve apparentness failures before sidereal checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error.message.contains("exposes tropical coordinates only"));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_apparent_requests_before_sidereal_and_observer_checks_in_batches(
) {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let apparent_sidereal_topocentric_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
            &[geocentric_request, apparent_sidereal_topocentric_request],
            &metadata,
        )
        .expect_err(
            "the batch helper should preserve apparentness failures before sidereal and observer checks",
        );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert!(error.message.contains(
            "batch request 2: toy backend currently returns mean geometric coordinates only; apparent corrections are not implemented"
        ));
    assert!(!error.message.contains("exposes tropical coordinates only"));
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_apparentness_checks_in_batches(
) {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_apparent_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        apparent: Apparentness::Apparent,
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, unsupported_time_scale_apparent_request],
        &metadata,
    )
    .expect_err("time-scale failures should be reported before apparentness failures in batches");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("apparent corrections are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_apparentness_and_observer_checks_in_batches(
) {
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
            apparent: false,
            ..BackendCapabilities::default()
        },
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_apparent_observer_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        apparent: Apparentness::Apparent,
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
            &[geocentric_request, unsupported_time_scale_apparent_observer_request],
            &metadata,
        )
        .expect_err(
            "time-scale failures should be reported before apparentness and observer failures in batches",
        );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("apparent corrections are not implemented"));
    assert!(!error
        .message
        .contains("request received invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_rejects_unsupported_time_scales_before_invalid_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let unsupported_time_scale_invalid_observer_request = EphemerisRequest {
        instant: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Utc),
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[
            geocentric_request,
            unsupported_time_scale_invalid_observer_request,
        ],
        &metadata,
    )
    .expect_err(
        "time-scale failures should be reported before invalid observer failures in batches",
    );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    assert_eq!(
        error.message,
        "batch request 2: toy backend expects one of [TT] for request instants"
    );
    assert!(!error
        .message
        .contains("request received invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_rejects_topocentric_requests_with_batch_index_prefix() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };
    let topocentric_summary = topocentric_request
        .observer
        .as_ref()
        .expect("observer should be present")
        .summary_line();

    let error =
        validate_requests_against_metadata(&[geocentric_request, topocentric_request], &metadata)
            .expect_err("the batch helper should preserve observer failures");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert!(error
        .message
        .contains("batch request 2: toy backend is geocentric only"));
    assert!(error.message.contains(&topocentric_summary));
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_before_topocentric_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_topocentric_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, sidereal_topocentric_request],
        &metadata,
    )
    .expect_err("the batch helper should preserve sidereal failures before observer checks");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
    assert!(!error
        .message
        .contains("topocentric positions are not implemented"));
}

#[test]
fn validate_requests_against_metadata_rejects_sidereal_requests_before_invalid_observer_checks_in_batches(
) {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let sidereal_invalid_observer_request = EphemerisRequest {
        zodiac_mode: ZodiacMode::Sidereal {
            ayanamsa: pleiades_types::Ayanamsa::FaganBradley,
        },
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(95.0),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = validate_requests_against_metadata(
        &[geocentric_request, sidereal_invalid_observer_request],
        &metadata,
    )
    .expect_err(
        "the batch helper should preserve sidereal failures before invalid observer checks",
    );
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedZodiacMode);
    assert_eq!(
        error.message,
        "batch request 2: toy backend currently exposes tropical coordinates only"
    );
    assert!(!error.message.contains("invalid observer location"));
}

#[test]
fn validate_requests_against_metadata_fails_fast_on_the_first_invalid_request() {
    let metadata = BackendMetadata {
        id: BackendId::new("toy backend"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![CelestialBody::Sun.into()],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    };
    let valid_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    let invalid_request = EphemerisRequest {
        body: CelestialBody::Mars,
        ..valid_request.clone()
    };

    let error = validate_requests_against_metadata(&[valid_request, invalid_request], &metadata)
        .expect_err("the batch helper should stop at the first unsupported body");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(
        error.message,
        "batch request 2: toy backend does not support Mars"
    );
}
