use std::sync::{Arc, Mutex};

use pleiades_backend::{
    AccuracyClass, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, ClaimEvidence, EphemerisBackend, EphemerisError,
    EphemerisErrorKind, EphemerisRequest, EphemerisResult, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, EclipticCoordinates, Latitude, Longitude, ObserverLocation, TimeScale,
};

pub(super) struct ToyChartBackend;

#[derive(Clone)]
pub(super) struct RecordingChartBackend {
    pub(super) observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
}

#[derive(Clone)]
pub(super) struct MeanOnlyRecordingChartBackend {
    pub(super) observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
}

impl EphemerisBackend for ToyChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("toy-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("toy chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into(), CelestialBody::Moon.into()],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun | CelestialBody::Moon)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if request.observer.is_some() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedObserver,
                "toy chart backend is geocentric only",
            ));
        }

        let longitude = match request.body {
            CelestialBody::Sun => Longitude::from_degrees(15.0),
            CelestialBody::Moon => Longitude::from_degrees(45.0),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "unsupported",
                ))
            }
        };
        let mut result = EphemerisResult::new(
            BackendId::new("toy-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(EclipticCoordinates::new(
            longitude,
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

impl EphemerisBackend for RecordingChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("recording-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("recording chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into()],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
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
        matches!(body, CelestialBody::Sun)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.observers
            .lock()
            .expect("observer log should be lockable")
            .push(request.observer.clone());
        let mut result = EphemerisResult::new(
            BackendId::new("recording-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

impl EphemerisBackend for MeanOnlyRecordingChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("mean-only-recording-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("mean-only recording chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into()],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                apparent: false,
                ..BackendCapabilities::default()
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.observers
            .lock()
            .expect("observer log should be lockable")
            .push(request.observer.clone());
        let mut result = EphemerisResult::new(
            BackendId::new("mean-only-recording-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(15.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

#[derive(Clone)]
pub(super) struct BatchRecordingChartBackend {
    pub(super) observers: Arc<Mutex<Vec<Option<ObserverLocation>>>>,
    pub(super) batch_calls: Arc<Mutex<usize>>,
}

impl EphemerisBackend for BatchRecordingChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("batch-recording-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("batch recording chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![CelestialBody::Sun.into(), CelestialBody::Moon.into()],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities {
                batch: true,
                ..BackendCapabilities::default()
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun | CelestialBody::Moon)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.observers
            .lock()
            .expect("observer log should be lockable")
            .push(request.observer.clone());
        let longitude = match request.body {
            CelestialBody::Sun => Longitude::from_degrees(15.0),
            CelestialBody::Moon => Longitude::from_degrees(45.0),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "unsupported",
                ))
            }
        };
        let mut result = EphemerisResult::new(
            BackendId::new("batch-recording-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(EclipticCoordinates::new(
            longitude,
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }

    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        *self
            .batch_calls
            .lock()
            .expect("batch call log should be lockable") += 1;
        reqs.iter().map(|req| self.position(req)).collect()
    }
}

/// A backend that marks the Sun as ReleaseGrade (for apparent-place tests).
/// It returns a fixed ecliptic position with distance_au for light-time computation.
pub(super) struct ApparentChartBackend;

impl EphemerisBackend for ApparentChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("apparent-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("apparent chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![BodyClaim::release_grade(
                CelestialBody::Sun,
                AccuracyClass::Approximate,
                ClaimEvidence::AlgorithmicModel,
            )],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Sun)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let longitude = match request.body {
            CelestialBody::Sun => Longitude::from_degrees(280.0),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "unsupported",
                ))
            }
        };
        let mut result = EphemerisResult::new(
            BackendId::new("apparent-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(EclipticCoordinates::new(
            longitude,
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}

/// A backend where only Moon is available (at Constrained tier). Sun is served
/// in `position()` for the apparent-place aberration term but is NOT declared
/// in `body_claims`. Tests the graceful mean fallback for non-release-grade bodies.
pub(super) struct ConstrainedOnlyChartBackend;

impl EphemerisBackend for ConstrainedOnlyChartBackend {
    fn metadata(&self) -> BackendMetadata {
        BackendMetadata {
            id: BackendId::new("constrained-only-chart"),
            version: "0.1.0".to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance::new("constrained-only chart backend"),
            nominal_range: pleiades_types::TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt],
            body_claims: vec![BodyClaim::constrained(
                CelestialBody::Moon,
                AccuracyClass::Approximate,
                ClaimEvidence::AlgorithmicModel,
            )],
            supported_frames: vec![pleiades_types::CoordinateFrame::Ecliptic],
            capabilities: BackendCapabilities::default(),
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        matches!(body, CelestialBody::Moon)
    }

    fn position(&self, request: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        // Sun is served for the apparent-place aberration term even though it is
        // not declared in body_claims — the Sun query bypasses metadata validation.
        let longitude = match request.body {
            CelestialBody::Sun => Longitude::from_degrees(280.0),
            CelestialBody::Moon => Longitude::from_degrees(45.0),
            _ => {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::UnsupportedBody,
                    "unsupported",
                ))
            }
        };
        let mut result = EphemerisResult::new(
            BackendId::new("constrained-only-chart"),
            request.body.clone(),
            request.instant,
            request.frame,
            request.zodiac_mode.clone(),
            request.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        result.ecliptic = Some(EclipticCoordinates::new(
            longitude,
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        Ok(result)
    }
}
