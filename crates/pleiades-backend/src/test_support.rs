#![allow(dead_code)]

use crate::{
    AccuracyClass, Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, BodyClaim, EphemerisRequest, TimeRange,
};
use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode};

pub(crate) fn toy_metadata() -> BackendMetadata {
    BackendMetadata {
        id: BackendId::new("toy"),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("toy backend for tests"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims: vec![BodyClaim::from(CelestialBody::Sun)],
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    }
}

/// Builds a minimal valid [`BackendMetadata`] with the given id and body claims.
///
/// The returned metadata passes [`BackendMetadata::validate()`]. Callers must
/// supply at least one unique body claim (non-empty, no duplicate bodies).
pub(crate) fn metadata_with_claims(id: &str, body_claims: Vec<BodyClaim>) -> BackendMetadata {
    BackendMetadata {
        id: BackendId::new(id),
        version: "0.1.0".to_string(),
        family: BackendFamily::Algorithmic,
        provenance: BackendProvenance::new("test backend"),
        nominal_range: TimeRange::new(None, None),
        supported_time_scales: vec![TimeScale::Tt],
        body_claims,
        supported_frames: vec![CoordinateFrame::Ecliptic],
        capabilities: BackendCapabilities::default(),
        accuracy: AccuracyClass::Approximate,
        deterministic: true,
        offline: true,
    }
}

pub(crate) fn tt_instant() -> Instant {
    Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt)
}

pub(crate) fn toy_sun_request() -> EphemerisRequest {
    EphemerisRequest {
        body: CelestialBody::Sun,
        instant: tt_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }
}
