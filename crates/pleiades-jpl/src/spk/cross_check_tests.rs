//! Cross-checks SPK chaining/reduction against the existing fixture math.

use pleiades_backend::{CelestialBody, EphemerisBackend};
use pleiades_types::{Instant, JulianDay, TimeScale};

use super::backend::SpkBackend;
use super::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
    let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
    let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
    SegmentSpec {
        start_et: -1.0e12,
        stop_et: 1.0e12,
        target,
        center,
        frame: 1,
        data_type: 2,
        data,
        name: "C".to_string(),
    }
}

#[test]
fn spk_reduction_matches_snapshot_entry_ecliptic() {
    // Treat a known ICRF geocentric vector as if it came from a kernel: place a
    // body at an equatorial position and confirm the reduction matches a direct
    // SnapshotEntry-style ecliptic computation (same obliquity, same formula).
    let body_icrf = [1.2e8, -3.4e7, 5.6e6]; // arbitrary equatorial km
    let blob = build_daf(&[
        const_seg(10, 0, body_icrf),
        const_seg(399, 3, [0.0, 0.0, 0.0]),
        const_seg(3, 0, [0.0, 0.0, 0.0]),
    ]);
    let backend = SpkBackend::builder().add_kernel_bytes(blob, "x").unwrap().build();
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let res = backend
        .position(&pleiades_backend::EphemerisRequest::new(CelestialBody::Sun, inst))
        .unwrap();
    let ec = res.ecliptic.unwrap();

    // Independent reference: rotate the same vector by mean obliquity here.
    let eps = inst.mean_obliquity().radians();
    let (x, ye, ze) = (body_icrf[0], body_icrf[1], body_icrf[2]);
    let y = ye * eps.cos() + ze * eps.sin();
    let _z = -ye * eps.sin() + ze * eps.cos();
    let expect_lon = y.atan2(x).to_degrees().rem_euclid(360.0);
    assert!(
        (ec.longitude.degrees() - expect_lon).abs() < 1e-6,
        "lon {} vs {}",
        ec.longitude.degrees(),
        expect_lon
    );
}
