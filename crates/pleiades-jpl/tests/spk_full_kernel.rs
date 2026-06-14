//! Opt-in end-to-end test against a real DE kernel. Skipped unless the env var
//! `PLEIADES_DE_KERNEL` points to a readable `.bsp` file.
//!
//! Run with:
//!   PLEIADES_DE_KERNEL=/path/to/de440.bsp cargo test -p pleiades-jpl --test spk_full_kernel -- --nocapture

use pleiades_backend::{CelestialBody, EphemerisBackend, EphemerisRequest};
use pleiades_jpl::SpkBackend;
use pleiades_types::{Instant, JulianDay, TimeScale};

#[test]
fn de440_reports_coverage_and_resolves_sun() {
    let Ok(path) = std::env::var("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL to a .bsp path to run");
        return;
    };
    let backend = SpkBackend::builder().add_kernel(&path).unwrap().build();
    let meta = backend.metadata();
    assert!(
        meta.nominal_range.start.is_some(),
        "kernel coverage detected"
    );
    assert!(backend.supports_body(CelestialBody::Sun));
    assert!(backend.supports_body(CelestialBody::Jupiter));

    // J2000.0: the Sun's geocentric ecliptic longitude is ~280.5 degrees.
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let res = backend
        .position(&EphemerisRequest::new(CelestialBody::Sun, inst))
        .unwrap();
    let lon = res.ecliptic.unwrap().longitude.degrees();
    assert!((lon - 280.5).abs() < 1.0, "sun lon {lon} near 280.5");
}
