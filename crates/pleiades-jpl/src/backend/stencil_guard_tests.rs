//! The snapshot backend answers only where its rows support the answer
//! (issue #158).

use super::*;
use crate::test_support::mean_request;
use crate::{asteroid_reference_corpus, reference_snapshot};
use pleiades_backend::{
    CelestialBody, CustomBodyId, EphemerisBackend, EphemerisErrorKind, QualityAnnotation,
};
use pleiades_types::{CoordinateFrame, TimeScale};

/// Largest disagreement allowed between a served position and the
/// `sb441-n373s` row. At JD 2453000.5 the Horizons row and the kernel row
/// differ by up to 1.0″ in longitude and 1.9″ in latitude (measured
/// 2026-10-06).
const TRUTH_TOLERANCE_ARCSEC: f64 = 5.0;
/// The one epoch a snapshot row and a truth row share.
const SHARED_EPOCH_JD: f64 = 2_453_000.5;
/// Between J2000 and the January 2001 cluster.
const MID_GAP_JD: f64 = 2_451_700.0;
const CLUSTER_FIRST_JD: f64 = 2_451_910.5;
const CLUSTER_LAST_JD: f64 = 2_451_919.5;

fn eros() -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
}

fn apophis() -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
}

fn truth_bodies() -> [CelestialBody; 5] {
    [
        CelestialBody::Ceres,
        CelestialBody::Pallas,
        CelestialBody::Juno,
        CelestialBody::Vesta,
        eros(),
    ]
}

/// Longitude (scaled by cos latitude) and latitude differences, arcseconds.
fn separation_arcsec(got: &EclipticCoordinates, truth: &EclipticCoordinates) -> (f64, f64) {
    let longitude = angular_degrees_delta(got.longitude.degrees(), truth.longitude.degrees())
        * truth.latitude.degrees().to_radians().cos()
        * 3600.0;
    let latitude = (got.latitude.degrees() - truth.latitude.degrees()).abs() * 3600.0;
    (longitude, latitude)
}

#[test]
fn every_truth_epoch_is_served_within_tolerance_or_refused() {
    let backend = JplSnapshotBackend;
    for body in truth_bodies() {
        let mut served = Vec::new();
        let mut refused = 0_usize;
        for row in asteroid_reference_corpus()
            .iter()
            .filter(|row| row.body == body)
        {
            let jd = row.epoch.julian_day.days();
            match backend.position(&mean_request(body.clone(), jd)) {
                Ok(result) => {
                    let got = result
                        .ecliptic
                        .expect("a served row has ecliptic coordinates");
                    let (longitude, latitude) = separation_arcsec(&got, &row.ecliptic());
                    assert!(
                        longitude <= TRUTH_TOLERANCE_ARCSEC && latitude <= TRUTH_TOLERANCE_ARCSEC,
                        "{body} at JD {jd}: served {longitude:.3}″ / {latitude:.3}″ from truth"
                    );
                    served.push(jd);
                }
                Err(error) => {
                    assert_eq!(
                        error.kind,
                        EphemerisErrorKind::OutOfRangeInstant,
                        "{body} at JD {jd}: {error}"
                    );
                    refused += 1;
                }
            }
        }
        assert_eq!(served, vec![SHARED_EPOCH_JD], "{body}: served epochs");
        assert_eq!(refused, 406, "{body}: refused epochs");
    }
}

#[test]
fn a_request_between_j2000_and_the_cluster_is_refused_for_every_asteroid() {
    let backend = JplSnapshotBackend;
    for body in truth_bodies().into_iter().chain([apophis()]) {
        let error = backend
            .position(&mean_request(body.clone(), MID_GAP_JD))
            .expect_err("a mid-gap request must be refused");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant, "{body}");
        let message = error.to_string();
        assert!(message.contains("SpkBackend"), "{body}: {message}");
        assert!(message.contains("2451545"), "{body}: {message}");
    }
}

#[test]
fn a_request_inside_the_cluster_interpolates_between_its_neighbours() {
    let backend = JplSnapshotBackend;
    let longitude_at = |jd: f64| {
        let result = backend
            .position(&mean_request(CelestialBody::Ceres, jd))
            .expect("the cluster supports this instant");
        (
            result.ecliptic.expect("ecliptic").longitude.degrees(),
            result.quality,
        )
    };
    let (before, before_quality) = longitude_at(2_451_914.0);
    let (middle, middle_quality) = longitude_at(2_451_914.25);
    let (after, after_quality) = longitude_at(2_451_914.5);
    assert_eq!(before_quality, QualityAnnotation::Exact);
    assert_eq!(after_quality, QualityAnnotation::Exact);
    assert_eq!(middle_quality, QualityAnnotation::Interpolated);
    let chord = (before + after) / 2.0;
    assert!(
        (middle - chord).abs() * 3600.0 < 1.0,
        "Ceres at the half-day point: {middle} against the chord {chord}"
    );
}

#[test]
fn a_request_just_outside_the_cluster_is_refused() {
    let backend = JplSnapshotBackend;
    for jd in [
        CLUSTER_FIRST_JD - 0.1,
        CLUSTER_LAST_JD + 0.1,
        CLUSTER_LAST_JD + 1e-6,
    ] {
        let error = backend
            .position(&mean_request(CelestialBody::Ceres, jd))
            .expect_err("no row brackets this instant within the stencil");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant, "JD {jd}");
    }
}

#[test]
fn a_light_time_step_off_an_isolated_row_is_refused() {
    let error = JplSnapshotBackend
        .position(&mean_request(CelestialBody::Ceres, 2_451_545.0 - 0.013))
        .expect_err("the J2000 row stands alone");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn an_exact_row_is_served_whatever_its_time_scale_tag() {
    let mut request = mean_request(CelestialBody::Ceres, 2_451_545.0);
    request.instant.scale = TimeScale::Tt;
    let result = JplSnapshotBackend
        .position(&request)
        .expect("an exact row is served");
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn the_frame_does_not_bypass_the_guard() {
    let mut request = mean_request(CelestialBody::Ceres, MID_GAP_JD);
    request.frame = CoordinateFrame::Equatorial;
    let error = JplSnapshotBackend
        .position(&request)
        .expect_err("an equatorial request is refused like an ecliptic one");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn a_batch_holding_a_refused_request_returns_the_refusal() {
    let served = mean_request(CelestialBody::Ceres, 2_451_545.0);
    let refused = mean_request(CelestialBody::Ceres, MID_GAP_JD);
    let error = JplSnapshotBackend
        .positions(&[served, refused])
        .expect_err("the batch must not return a partial result");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn a_request_past_the_last_row_keeps_its_message() {
    let error = JplSnapshotBackend
        .position(&mean_request(CelestialBody::Ceres, 2_634_168.0))
        .expect_err("past the last row");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
    assert!(error
        .to_string()
        .contains("outside adjacent JPL fixture samples"));
}

/// Holds every asteroid row out in turn. Where the remaining rows pass the
/// guard, the interpolation must reproduce the held-out row. This is the
/// measurement behind `MAX_STENCIL_SPAN_DAYS`. Major bodies are left out:
/// some of their cluster rows are not geocentric ecliptic positions, and the
/// Moon moves too fast for a cubic through day-spaced rows (issue #200).
#[test]
fn stencils_the_guard_admits_reproduce_held_out_rows() {
    let entries = reference_snapshot();
    let is_asteroid = |body: &CelestialBody| truth_bodies().contains(body) || *body == apophis();
    let mut asteroid_cases = 0_usize;
    let mut worst_asteroid = 0.0_f64;
    for held_out in entries.iter().filter(|entry| is_asteroid(&entry.body)) {
        let jd = held_out.epoch.julian_day.days();
        let rest = entries
            .iter()
            .filter(|entry| entry.body != held_out.body || entry.epoch.julian_day.days() != jd)
            .cloned()
            .collect::<Vec<_>>();
        if !stencil_supports(&rest, &held_out.body, jd) {
            continue;
        }
        let interpolated = interpolate_fixture_state(&rest, held_out.body.clone(), jd)
            .expect("an admitted stencil interpolates");
        let (longitude, latitude) =
            separation_arcsec(&interpolated.ecliptic(), &held_out.ecliptic());
        asteroid_cases += 1;
        worst_asteroid = worst_asteroid.max(longitude.max(latitude));
    }
    assert!(asteroid_cases >= 40, "only {asteroid_cases} asteroid cases");
    assert!(
        worst_asteroid <= ADMITTED_ASTEROID_CEILING_ARCSEC,
        "asteroids: {worst_asteroid:.4}″"
    );
}

/// Measured 2026-10-06 over 48 admitted asteroid cases: the worst is 0.0032″,
/// Juno at JD 2451918.5.
const ADMITTED_ASTEROID_CEILING_ARCSEC: f64 = 0.05;
