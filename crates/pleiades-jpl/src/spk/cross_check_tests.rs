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
    let backend = SpkBackend::builder()
        .add_kernel_bytes(blob, "x")
        .unwrap()
        .build();
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let res = backend
        .position(&pleiades_backend::EphemerisRequest::new(
            CelestialBody::Sun,
            inst,
        ))
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

#[test]
fn spk_reduction_is_j2000_frame_at_non_j2000_epoch() {
    // Same synthetic ICRF geocentric vector as the J2000 test, but evaluated at
    // 1900 where ε_date − ε₀ ≈ 46″. The reduction must use the FIXED ε₀ (J2000),
    // so the ecliptic latitude equals the ε₀ rotation and is epoch-independent.
    let body_icrf = [1.2e8, -3.4e7, 5.6e6];
    let blob = build_daf(&[
        const_seg(10, 0, body_icrf),
        const_seg(399, 3, [0.0, 0.0, 0.0]),
        const_seg(3, 0, [0.0, 0.0, 0.0]),
    ]);
    let backend = SpkBackend::builder()
        .add_kernel_bytes(blob, "x")
        .unwrap()
        .build();
    let inst_1900 = Instant::new(JulianDay::from_days(2_415_025.5), TimeScale::Tt);
    let ec = backend
        .position(&pleiades_backend::EphemerisRequest::new(
            CelestialBody::Sun,
            inst_1900,
        ))
        .unwrap()
        .ecliptic
        .unwrap();

    // Independent J2000 reference: rotate by ε₀, NOT by the of-date obliquity.
    let eps0 = pleiades_types::OBLIQUITY_J2000_DEG.to_radians();
    let (x, ye, ze) = (body_icrf[0], body_icrf[1], body_icrf[2]);
    let y = ye * eps0.cos() + ze * eps0.sin();
    let z = -ye * eps0.sin() + ze * eps0.cos();
    let r = (x * x + y * y + z * z).sqrt();
    let expect_lat = (z / r).asin().to_degrees();
    assert!(
        (ec.latitude.degrees() - expect_lat).abs() < 1e-9,
        "lat {} should equal the ε₀ rotation {} at 1900 (J2000 frame)",
        ec.latitude.degrees(),
        expect_lat
    );

    // And it must DIFFER from the of-date rotation by ~tens of arcsec — proving
    // the test is frame-discriminating, not vacuous.
    let eps_d = inst_1900.mean_obliquity().radians();
    let z_d = -ye * eps_d.sin() + ze * eps_d.cos();
    let of_date_lat = (z_d
        / (x * x
            + (ye * eps_d.cos() + ze * eps_d.sin()).powi(2)
            + z_d * z_d)
            .sqrt())
    .asin()
    .to_degrees();
    assert!(
        (ec.latitude.degrees() - of_date_lat).abs() * 3600.0 > 1.0,
        "frame-blind: J2000 and of-date latitudes are too close to discriminate (gap = {:.4}\")",
        (ec.latitude.degrees() - of_date_lat).abs() * 3600.0
    );
}
