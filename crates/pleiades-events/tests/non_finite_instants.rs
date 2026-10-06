//! Every first-party backend, and the production-style chain over them,
//! refuses a request whose instant is NaN or infinite with
//! `EphemerisErrorKind::InvalidRequest` (issue #201). The policy lives in
//! `validate_request_policy`; this guards against a backend that stops
//! calling it.

use pleiades_backend::{
    CompositeBackend, EphemerisBackend, EphemerisErrorKind, EphemerisRequest, RoutingBackend,
};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

fn assert_refuses_non_finite(label: &str, backend: &dyn EphemerisBackend, body: CelestialBody) {
    for days in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let request = EphemerisRequest::new(
            body.clone(),
            Instant::new(JulianDay::from_days(days), TimeScale::Tt),
        );
        let error = backend
            .position(&request)
            .expect_err("a non-finite instant must be refused");
        assert_eq!(
            error.kind,
            EphemerisErrorKind::InvalidRequest,
            "{label} at JD {days}: {error}"
        );
    }
}

#[test]
fn every_first_party_backend_refuses_non_finite_instants() {
    assert_refuses_non_finite("packaged", &PackagedDataBackend::new(), CelestialBody::Sun);
    assert_refuses_non_finite("vsop87", &Vsop87Backend::new(), CelestialBody::Sun);
    assert_refuses_non_finite("elp", &ElpBackend::new(), CelestialBody::Moon);
    assert_refuses_non_finite(
        "jpl snapshot",
        &JplSnapshotBackend::new(),
        CelestialBody::Sun,
    );
    assert_refuses_non_finite(
        "fictitious",
        &FictitiousBackend::new(PackagedDataBackend::new()),
        CelestialBody::Cupido,
    );
}

#[test]
fn the_production_chain_refuses_non_finite_instants() {
    let chain = RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]);
    assert_refuses_non_finite("chain", &chain, CelestialBody::Sun);
}
