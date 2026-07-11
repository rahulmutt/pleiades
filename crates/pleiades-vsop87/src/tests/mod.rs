use super::*;
use crate::source_docs::{
    build_generated_binary_audits_with_lookup, source_documentation_health_issues,
    CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL, CANONICAL_EVIDENCE_SUMMARY_LABEL,
};

mod backend;
mod documentation;
mod evidence;
mod profiles;

#[test]
fn package_name_is_stable() {
    assert_eq!(PACKAGE_NAME, "pleiades-vsop87");
}

#[test]
fn backend_reports_major_planets() {
    let backend = Vsop87Backend::new();
    assert!(backend.supports_body(CelestialBody::Sun));
    assert!(backend.supports_body(CelestialBody::Mars));
    assert!(!backend.supports_body(CelestialBody::Moon));
}

fn mean_request(body: CelestialBody) -> EphemerisRequest {
    mean_request_at(
        body,
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    )
}

fn mean_request_at(body: CelestialBody, instant: Instant) -> EphemerisRequest {
    let mut request = EphemerisRequest::new(body, instant);
    request.apparent = Apparentness::Mean;
    request
}

fn assert_degrees_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = signed_longitude_delta_degrees(expected, actual).abs();
    assert!(
        delta <= tolerance,
        "expected {actual}° to be within {tolerance}° of {expected}°; delta was {delta}°"
    );
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}; delta was {delta}"
    );
}
