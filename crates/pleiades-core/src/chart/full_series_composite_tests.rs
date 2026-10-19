//! The VSOP87 full-series backend composes with ELP like the default one
//! (issue #281).

use pleiades_backend::{Apparentness, CompositeBackend, EphemerisBackend, EphemerisRequest};
use pleiades_elp::ElpBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87FullSeriesBackend;

#[test]
fn full_series_backend_composes() {
    let mut request = EphemerisRequest::new(
        CelestialBody::Mars,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    request.apparent = Apparentness::Mean;

    let composite = CompositeBackend::new(ElpBackend::new(), Vsop87FullSeriesBackend::new());
    let composed = composite.position(&request).expect("composite Mars");
    let direct = Vsop87FullSeriesBackend::new()
        .position(&request)
        .expect("full-series Mars");
    assert_eq!(composed, direct);
}
