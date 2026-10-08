//! The packaged asteroids against every JPL `sb441-n373s` row of
//! `asteroid_reference.csv` (issue #201): geocentric ecliptic J2000,
//! geometric. The rows come from the same kernel the fits sample, at
//! instants the fits do not, so this measures fit error.

use crate::packaged_artifact;
use crate::packaged_asteroids;
use crate::thresholds::ASTEROID_CORPUS_CEILING;
use crate::PackagedDataBackend;
use pleiades_backend::{
    CelestialBody, CustomBodyId, EphemerisBackend, EphemerisErrorKind, EphemerisRequest, Instant,
    JulianDay, TimeScale,
};
use pleiades_jpl::SnapshotEntry;

const ROWS_PER_BODY: usize = 407;

fn truth_longitude_latitude(row: &SnapshotEntry) -> (f64, f64) {
    let radius = (row.x_km * row.x_km + row.y_km * row.y_km + row.z_km * row.z_km).sqrt();
    (
        row.y_km.atan2(row.x_km).to_degrees(),
        (row.z_km / radius).clamp(-1.0, 1.0).asin().to_degrees(),
    )
}

#[test]
fn packaged_asteroids_match_the_sb441_rows() {
    let backend = PackagedDataBackend::new();
    for body in packaged_asteroids() {
        let rows: Vec<&SnapshotEntry> = pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| &row.body == body)
            .collect();
        assert_eq!(rows.len(), ROWS_PER_BODY, "{body}: reference rows");
        let (mut max_lon, mut max_lat) = (0.0_f64, 0.0_f64);
        for row in rows {
            let got = backend
                .position(&EphemerisRequest::new(body.clone(), row.epoch))
                .unwrap_or_else(|e| panic!("{body} at {}: {e}", row.epoch.julian_day.days()))
                .ecliptic
                .expect("ecliptic");
            let (lon, lat) = truth_longitude_latitude(row);
            let dlon = (got.longitude.degrees() - lon + 180.0).rem_euclid(360.0) - 180.0;
            max_lon = max_lon.max(dlon.abs() * lat.to_radians().cos() * 3600.0);
            max_lat = max_lat.max((got.latitude.degrees() - lat).abs() * 3600.0);
        }
        eprintln!("{body}: max lon {max_lon:.4}\", max lat {max_lat:.4}\"");
        assert!(
            max_lon <= ASTEROID_CORPUS_CEILING.lon_arcsec
                && max_lat <= ASTEROID_CORPUS_CEILING.lat_arcsec,
            "{body}: {max_lon:.4}\" / {max_lat:.4}\" over the ceiling"
        );
    }
}

#[test]
fn packaged_asteroids_are_served_at_both_window_ends_and_refused_past_them() {
    let backend = PackagedDataBackend::new();
    let range = backend.metadata().nominal_range;
    let start = range.start.expect("window start").julian_day.days();
    let end = range.end.expect("window end").julian_day.days();
    for body in packaged_asteroids() {
        for jd in [start, end] {
            let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
            assert!(
                backend
                    .position(&EphemerisRequest::new(body.clone(), at))
                    .is_ok(),
                "{body} at JD {jd}"
            );
        }
        let past = Instant::new(JulianDay::from_days(end + 1.0), TimeScale::Tdb);
        assert!(backend
            .position(&EphemerisRequest::new(body.clone(), past))
            .is_err());
    }
}

#[test]
fn an_artifact_without_asteroids_refuses_them_cleanly() {
    let mut artifact = packaged_artifact().clone();
    artifact
        .bodies
        .retain(|series| !packaged_asteroids().contains(&series.body));
    artifact.checksum = artifact.checksum().expect("checksum");
    let backend = PackagedDataBackend::from_artifact(artifact);
    let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let error = backend
        .position(&EphemerisRequest::new(CelestialBody::Ceres, at))
        .expect_err("no Ceres segments");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody, "{error}");
}

#[test]
fn the_packaged_backend_does_not_claim_apophis() {
    let apophis = CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"));
    assert!(!PackagedDataBackend::new().supports_body(apophis));
}
