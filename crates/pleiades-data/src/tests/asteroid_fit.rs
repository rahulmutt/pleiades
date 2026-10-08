//! Kernel-gated maintainer measurements for the dense asteroid fits
//! (issue #201). Run in release with both kernels:
//!
//! PLEIADES_DE_KERNEL=… PLEIADES_AST_KERNEL=… cargo test --release -p pleiades-data \
//!     --lib asteroid_fit -- --ignored --nocapture

use crate::packaged_asteroids;
use crate::regenerate::fit_dense_body_artifact;
use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_compression::{ArtifactHeader, CompressedArtifact};
use pleiades_jpl::spk::corpus_spec::CoverageWindow;
use pleiades_jpl::SpkBackend;

const SAMPLE_STEP_DAYS: f64 = 0.5;

fn two_kernel_reference() -> Option<SpkBackend> {
    let (Ok(de), Ok(ast)) = (
        std::env::var("PLEIADES_DE_KERNEL"),
        std::env::var("PLEIADES_AST_KERNEL"),
    ) else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL to run");
        return None;
    };
    Some(
        SpkBackend::builder()
            .add_kernel(&de)
            .expect("de440 kernel loads")
            .add_kernel(&ast)
            .expect("sb441-n373s kernel loads")
            .build(),
    )
}

fn artifact_of(bodies: Vec<pleiades_compression::BodyArtifact>) -> CompressedArtifact {
    let mut artifact =
        CompressedArtifact::new(ArtifactHeader::new("measurement", "measurement"), bodies);
    artifact.checksum = artifact.checksum().expect("checksum");
    artifact
}

/// Wrapped longitude difference × cos(lat) and latitude difference, arcsec.
fn error_arcsec(
    got: &pleiades_backend::EclipticCoordinates,
    want: &pleiades_backend::EclipticCoordinates,
) -> (f64, f64) {
    let dlon =
        (got.longitude.degrees() - want.longitude.degrees() + 180.0).rem_euclid(360.0) - 180.0;
    let lon = dlon.abs() * want.latitude.degrees().to_radians().cos() * 3600.0;
    let lat = (got.latitude.degrees() - want.latitude.degrees()).abs() * 3600.0;
    (lon, lat)
}

#[test]
#[ignore = "maintainer measurement: needs PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL; run in release"]
fn asteroid_fit_error_and_size_against_the_kernel() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let Some(reference) = two_kernel_reference() else {
        return;
    };
    let window = CoverageWindow::default().as_tuple();
    let sun = fit_dense_body_artifact(&CelestialBody::Sun, window, &reference);
    let sun_only_bytes = artifact_of(vec![sun.clone()])
        .encode()
        .expect("encode")
        .len();
    let mut total_added = 0_usize;
    for body in packaged_asteroids() {
        let fitted = fit_dense_body_artifact(body, window, &reference);
        let segments = fitted.segments.len();
        let artifact = artifact_of(vec![sun.clone(), fitted]);
        let added = artifact.encode().expect("encode").len() - sun_only_bytes;
        total_added += added;
        let (mut max_lon, mut lon_jd, mut max_lat, mut lat_jd) = (0.0_f64, 0.0, 0.0_f64, 0.0);
        let mut jd = window.0;
        while jd < window.1 {
            let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
            // Segments are tagged Tt (see fit_segment_within_span); the
            // reference is queried in Tdb, a ~2 ms difference.
            let artifact_instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
            let got = artifact
                .lookup_ecliptic(body, artifact_instant)
                .expect("artifact lookup");
            let want = reference
                .position(&EphemerisRequest::new(body.clone(), instant))
                .expect("kernel position")
                .ecliptic
                .expect("kernel ecliptic");
            let (lon, lat) = error_arcsec(&got, &want);
            if lon > max_lon {
                (max_lon, lon_jd) = (lon, jd);
            }
            if lat > max_lat {
                (max_lat, lat_jd) = (lat, jd);
            }
            jd += SAMPLE_STEP_DAYS;
        }
        eprintln!(
            "{body}: span {} d, {segments} segments, +{added} bytes, \
             max lon {max_lon:.4}\" at JD {lon_jd}, max lat {max_lat:.4}\" at JD {lat_jd}",
            crate::coverage::fitting_segment_span_days(body)
        );
    }
    let committed = crate::packaged_artifact_bytes();
    let mut without_eros = CompressedArtifact::decode(committed).expect("decode committed");
    without_eros
        .bodies
        .retain(|b| !matches!(&b.body, CelestialBody::Custom(_)));
    without_eros.checksum = without_eros.checksum().expect("checksum");
    let old_eros = committed.len() - without_eros.encode().expect("encode").len();
    eprintln!(
        "projected artifact: {} bytes (committed {} - old Eros {old_eros} + added {total_added})",
        committed.len() - old_eros + total_added,
        committed.len()
    );
    eprintln!("total added by the five asteroids: {total_added} bytes");
}
