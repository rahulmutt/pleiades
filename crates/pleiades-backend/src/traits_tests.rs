use crate::*;
use core::sync::atomic::{AtomicUsize, Ordering};

struct ToyBackend;

impl EphemerisBackend for ToyBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("toy"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy backend for tests"),
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into()],
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
        if req.body != CelestialBody::Sun {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "only the Sun is supported",
            ));
        }

        let mut result = EphemerisResult::new(
            BackendId::new("toy"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Exact;
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(120.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

#[test]
fn routing_metadata_defers_request_shape_checks_to_the_selected_provider() {
    struct RejectingSunBackend;

    impl EphemerisBackend for RejectingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("rejecting-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("rejecting Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: false,
                    apparent: false,
                    topocentric: false,
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
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "rejecting Sun backend is geocentric only",
                ));
            }

            if req.apparent == Apparentness::Apparent {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "rejecting Sun backend only returns mean geometric coordinates",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("rejecting-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct AcceptingSunBackend;

    impl EphemerisBackend for AcceptingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("accepting-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("accepting Sun backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
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
                BackendId::new("accepting-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(RejectingSunBackend),
        Box::new(AcceptingSunBackend),
    ]);
    let mut request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));
    request.apparent = Apparentness::Apparent;

    let metadata = routing.metadata();
    assert!(metadata.family.is_routing());
    assert!(!metadata.capabilities.batch);

    validate_requests_against_metadata(&[request.clone()], &metadata)
        .expect("routing metadata should defer request-shape checks to the selected provider");

    let result = routing
        .positions(&[request])
        .expect("routing should recover through the secondary provider");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].backend_id.as_str(), "accepting-sun");
}

#[test]
fn routing_backend_batch_metadata_defers_observer_and_apparentness_checks_to_the_selected_provider()
{
    struct RejectingSunBackend;

    impl EphemerisBackend for RejectingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("rejecting-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("rejecting Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![CelestialBody::Sun.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
                    batch: false,
                    apparent: false,
                    topocentric: false,
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
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "rejecting Sun batch backend is geocentric only",
                ));
            }

            if req.apparent == Apparentness::Apparent {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "rejecting Sun batch backend only returns mean geometric coordinates",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("rejecting-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct AcceptingSunBackend;

    impl EphemerisBackend for AcceptingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("accepting-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("accepting Sun batch backend"),
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
                BackendId::new("accepting-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(RejectingSunBackend),
        Box::new(AcceptingSunBackend),
    ]);
    let mut geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    geocentric_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));
    let mut apparent_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
    );
    apparent_request.apparent = Apparentness::Apparent;

    let metadata = routing.metadata();
    assert!(metadata.family.is_routing());
    assert!(!metadata.capabilities.batch);

    validate_requests_against_metadata(
        &[geocentric_request.clone(), apparent_request.clone()],
        &metadata,
    )
    .expect(
        "routing metadata should defer observer and apparentness checks to the selected provider",
    );

    let results = routing
        .positions(&[geocentric_request, apparent_request])
        .expect("routing should recover through the secondary provider for batch requests");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "accepting-sun-batch");
    assert_eq!(results[1].backend_id.as_str(), "accepting-sun-batch");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}

#[test]
fn batch_query_preserves_apparent_request_rejection() {
    struct MeanOnlyBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for MeanOnlyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mean-only"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mean-only test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into()],
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
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_request_policy(
                req,
                "mean-only test backend",
                &[TimeScale::Tt],
                &[CoordinateFrame::Ecliptic],
                true,
                false,
            )?;

            Ok(EphemerisResult::new(
                BackendId::new("mean-only"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = MeanOnlyBackend {
        calls: AtomicUsize::new(0),
    };
    let mean_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let apparent_request = EphemerisRequest {
        apparent: Apparentness::Apparent,
        ..mean_request.clone()
    };

    let error = backend
        .positions(&[mean_request, apparent_request])
        .expect_err("batch requests should preserve apparentness rejections");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedApparentness);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn batch_query_preserves_observer_request_rejection() {
    struct GeocentricOnlyBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for GeocentricOnlyBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("geocentric-only"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("geocentric-only test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into()],
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
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_observer_policy(req, "geocentric-only test backend", false)?;

            Ok(EphemerisResult::new(
                BackendId::new("geocentric-only"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = GeocentricOnlyBackend {
        calls: AtomicUsize::new(0),
    };
    let geocentric_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let topocentric_request = EphemerisRequest {
        observer: Some(ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(-0.1),
            Some(45.0),
        )),
        ..geocentric_request.clone()
    };

    let error = backend
        .positions(&[geocentric_request, topocentric_request])
        .expect_err("batch requests should preserve observer rejections");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedObserver);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn batch_query_preserves_mixed_time_scales() {
    struct MixedScaleBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for MixedScaleBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("mixed-scale"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("mixed-scale test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![CelestialBody::Sun.into()],
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
            self.calls.fetch_add(1, Ordering::SeqCst);

            validate_request_policy(
                req,
                "mixed-scale test backend",
                &[TimeScale::Tt, TimeScale::Tdb],
                &[CoordinateFrame::Ecliptic],
                true,
                false,
            )?;

            Ok(EphemerisResult::new(
                BackendId::new("mixed-scale"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = MixedScaleBackend {
        calls: AtomicUsize::new(0),
    };
    let tt_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let tdb_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
    );

    let results = backend
        .positions(&[tt_request, tdb_request])
        .expect("batch requests should preserve mixed time-scale labels");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn batch_query_uses_single_query_behavior() {
    let backend = ToyBackend;
    let request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    let result = backend
        .positions(&[request])
        .expect("toy backend should succeed");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].quality, QualityAnnotation::Exact);
    assert!(result[0].ecliptic.is_some());
}

#[test]
fn batch_query_short_circuits_on_the_first_error() {
    struct CountingBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for CountingBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("counting"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("counting test backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into(), CelestialBody::Moon.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, _body: CelestialBody) -> bool {
            true
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.calls.fetch_add(1, Ordering::SeqCst);

            if req.body == CelestialBody::Moon {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "Moon requests fail in the counting backend",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("counting"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let backend = CountingBackend {
        calls: AtomicUsize::new(0),
    };
    let requests = [
        EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
        EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
        EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
        ),
    ];

    let error = backend
        .positions(&requests)
        .expect_err("batch requests should fail fast on the first error");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[test]
fn metadata_is_reported() {
    let backend = ToyBackend;
    let metadata = backend.metadata();
    assert_eq!(metadata.id.as_str(), "toy");
    assert!(metadata.supported_bodies().contains(&CelestialBody::Sun));
}

#[test]
fn composite_backend_routes_by_body() {
    struct MoonBackend;

    impl EphemerisBackend for MoonBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("moon"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("moon backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Moon.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Moon
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("moon"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let composite = CompositeBackend::new(ToyBackend, MoonBackend);
    let sun_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let moon_request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    assert_eq!(
        composite
            .position(&sun_request)
            .unwrap()
            .backend_id
            .as_str(),
        "toy"
    );
    assert_eq!(
        composite
            .position(&moon_request)
            .unwrap()
            .backend_id
            .as_str(),
        "moon"
    );
}

#[test]
fn routing_backend_tries_later_providers_and_merges_metadata() {
    struct FailingSunBackend;

    impl EphemerisBackend for FailingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("fail-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("failing Sun backend"),
                nominal_range: TimeRange::new(
                    Some(Instant::new(
                        JulianDay::from_days(2_451_545.0),
                        TimeScale::Tt,
                    )),
                    Some(Instant::new(
                        JulianDay::from_days(2_451_546.0),
                        TimeScale::Tt,
                    )),
                ),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into()],
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

        fn position(&self, _req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedCoordinateFrame,
                "retry with the next provider",
            ))
        }
    }

    struct RecoverySunBackend;

    impl EphemerisBackend for RecoverySunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recovery-sun"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recovery Sun backend"),
                nominal_range: TimeRange::new(
                    Some(Instant::new(
                        JulianDay::from_days(2_451_545.5),
                        TimeScale::Tdb,
                    )),
                    Some(Instant::new(
                        JulianDay::from_days(2_451_546.5),
                        TimeScale::Tdb,
                    )),
                ),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Sun.into()],
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
                BackendId::new("recovery-sun"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            );
            result.quality = QualityAnnotation::Exact;
            result.ecliptic = Some(EclipticCoordinates::new(
                Longitude::from_degrees(10.0),
                Latitude::from_degrees(0.0),
                Some(1.0),
            ));
            Ok(result)
        }
    }

    struct MoonBackend;

    impl EphemerisBackend for MoonBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("moon"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("moon backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt],
                body_claims: vec![CelestialBody::Moon.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities::default(),
                accuracy: AccuracyClass::Approximate,
                deterministic: true,
                offline: true,
            }
        }

        fn supports_body(&self, body: CelestialBody) -> bool {
            body == CelestialBody::Moon
        }

        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            Ok(EphemerisResult::new(
                BackendId::new("moon"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(FailingSunBackend),
        Box::new(RecoverySunBackend),
        Box::new(MoonBackend),
    ]);
    let sun_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let moon_request = EphemerisRequest::new(
        CelestialBody::Moon,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );

    let metadata = routing.metadata();
    assert!(metadata.supported_bodies().contains(&CelestialBody::Sun));
    assert!(metadata.supported_bodies().contains(&CelestialBody::Moon));
    assert!(metadata.provenance.summary.contains("3 provider(s)"));
    assert_eq!(metadata.nominal_range.validate(), Ok(()));
    assert_eq!(
        metadata
            .nominal_range
            .start
            .expect("routing range start should exist")
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        metadata
            .nominal_range
            .end
            .expect("routing range end should exist")
            .scale,
        TimeScale::Tt
    );
    assert_eq!(
        metadata.nominal_range.summary_line(),
        "JD 2451545.5 (TT) → JD 2451546.0 (TT)"
    );

    assert_eq!(
        routing.position(&sun_request).unwrap().backend_id.as_str(),
        "recovery-sun"
    );
    assert_eq!(
        routing.position(&moon_request).unwrap().backend_id.as_str(),
        "moon"
    );
}

#[test]
fn routing_backend_batch_positions_preserve_mixed_time_scales_and_topocentric_observers_after_fallback(
) {
    struct FailingSunBackend;

    impl EphemerisBackend for FailingSunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("fail-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("failing Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![CelestialBody::Sun.into()],
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
            if req.observer.is_some() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedObserver,
                    "retry with the next provider",
                ));
            }

            Ok(EphemerisResult::new(
                BackendId::new("fail-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    struct RecoverySunBackend {
        calls: AtomicUsize,
    }

    impl EphemerisBackend for RecoverySunBackend {
        fn metadata(&self) -> BackendMetadata {
            BackendMetadata {
                id: BackendId::new("recovery-sun-batch"),
                version: "0.1.0".to_string(),
                family: BackendFamily::Algorithmic,
                provenance: BackendProvenance::new("recovery Sun batch backend"),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
                body_claims: vec![CelestialBody::Sun.into()],
                supported_frames: vec![CoordinateFrame::Ecliptic],
                capabilities: BackendCapabilities {
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
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(EphemerisResult::new(
                BackendId::new("recovery-sun-batch"),
                req.body.clone(),
                req.instant,
                req.frame,
                req.zodiac_mode.clone(),
                req.apparent,
            ))
        }
    }

    let routing = RoutingBackend::new(vec![
        Box::new(FailingSunBackend),
        Box::new(RecoverySunBackend {
            calls: AtomicUsize::new(0),
        }),
    ]);
    let tt_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tt),
    );
    let mut tdb_request = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2451545.0), TimeScale::Tdb),
    );
    tdb_request.observer = Some(ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(12.5),
        Some(0.0),
    ));

    let metadata = routing.metadata();
    validate_requests_against_metadata(&[tt_request.clone(), tdb_request.clone()], &metadata)
            .expect("routing metadata should keep mixed TT/TDB and topocentric requests aligned with the selected provider");

    let results = routing
            .positions(&[tt_request.clone(), tdb_request.clone()])
            .expect("routing should preserve mixed batch scales while falling back to the secondary provider");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend_id.as_str(), "fail-sun-batch");
    assert_eq!(results[1].backend_id.as_str(), "recovery-sun-batch");
    assert_eq!(results[0].instant.scale, TimeScale::Tt);
    assert_eq!(results[1].instant.scale, TimeScale::Tdb);
}
