//! pheno integration over the production-style backend chain.
//!
//! Same routing chain as `tests/nod_aps.rs`: packaged data first, then the
//! VSOP87/ELP composite, then the JPL snapshot, then fictitious bodies
//! layered over packaged data. Asserts cross-body invariants for all ten
//! majors at J2000.

use pleiades_backend::{CompositeBackend, RoutingBackend};
use pleiades_data::PackagedDataBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::EventEngine;
use pleiades_fict::FictitiousBackend;
use pleiades_jpl::JplSnapshotBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

// Same production routing chain as tests/nod_aps.rs.
fn engine() -> EventEngine<RoutingBackend> {
    EventEngine::new(RoutingBackend::new(vec![
        Box::new(PackagedDataBackend::new()),
        Box::new(CompositeBackend::new(
            Vsop87Backend::new(),
            ElpBackend::new(),
        )),
        Box::new(JplSnapshotBackend::new()),
        Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
    ]))
}

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

#[test]
fn majors_report_finite_outputs_with_magnitude() {
    let e = engine();
    for body in [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
    ] {
        let d = e.pheno(body.clone(), tdb(2_451_545.0)).unwrap();
        assert!(d.phase_angle_deg.is_finite() && (0.0..=180.0).contains(&d.phase_angle_deg));
        assert!((0.0..=1.0).contains(&d.phase_fraction));
        assert!((0.0..=180.0).contains(&d.elongation_deg));
        assert!(d.apparent_diameter_deg.is_finite() && d.apparent_diameter_deg >= 0.0);
        let mag = d.apparent_magnitude.expect("major bodies carry magnitude");
        assert!(mag.is_finite(), "{body:?} mag {mag}");
    }
}

#[test]
fn inner_planet_phase_tracks_phase_angle() {
    // Illuminated fraction must fall as the phase angle grows.
    let e = engine();
    let d = e.pheno(CelestialBody::Venus, tdb(2_451_545.0)).unwrap();
    let expected = (1.0 + d.phase_angle_deg.to_radians().cos()) / 2.0;
    assert!((d.phase_fraction - expected).abs() < 1e-9);
}
