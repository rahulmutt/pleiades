//! Per-body accuracy baseline: decoded committed artifact vs hold-out corpus.
//!
//! Compares every hold-out row against the committed packaged artifact at the same
//! epoch and body, accumulating per-body max and RMS errors in longitude (arcsec),
//! latitude (arcsec), and distance (km).  These numbers are the SP1 deliverable
//! that scopes SP2 accuracy targets.

use std::collections::HashMap;

use pleiades_backend::{Angle, CelestialBody, Instant};
use pleiades_compression::CompressedArtifact;
use pleiades_jpl::{production_holdout_corpus, SnapshotEntry};

use crate::regenerate::{build_packaged_artifact, coordinates};
use crate::AU_IN_KM;

/// Per-body accuracy summary comparing the artifact to an independent hold-out corpus.
#[derive(Clone, Debug)]
pub struct BodyChannelError {
    /// Body these errors apply to.
    pub body: CelestialBody,
    /// Maximum absolute longitude error across all hold-out rows for this body (arcseconds).
    pub max_longitude_arcsec: f64,
    /// Root-mean-square longitude error across all hold-out rows for this body (arcseconds).
    pub rms_longitude_arcsec: f64,
    /// Maximum absolute latitude error across all hold-out rows for this body (arcseconds).
    pub max_latitude_arcsec: f64,
    /// Root-mean-square latitude error across all hold-out rows for this body (arcseconds).
    pub rms_latitude_arcsec: f64,
    /// Maximum absolute distance error across all hold-out rows for this body (km).
    pub max_distance_km: f64,
    /// Root-mean-square distance error across all hold-out rows for this body (km).
    pub rms_distance_km: f64,
}

impl BodyChannelError {
    fn label(&self) -> String {
        format!("{:?}", self.body)
    }

    /// Returns a compact one-line summary for this body's errors.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: max_lon={:.4} arcsec  rms_lon={:.4} arcsec  max_lat={:.4} arcsec  rms_lat={:.4} arcsec  max_dist={:.3} km  rms_dist={:.3} km",
            self.label(),
            self.max_longitude_arcsec,
            self.rms_longitude_arcsec,
            self.max_latitude_arcsec,
            self.rms_latitude_arcsec,
            self.max_distance_km,
            self.rms_distance_km,
        )
    }
}

struct BodyAccumulator {
    body: CelestialBody,
    count: usize,
    max_lon_arcsec: f64,
    sum_sq_lon_arcsec: f64,
    max_lat_arcsec: f64,
    sum_sq_lat_arcsec: f64,
    max_dist_km: f64,
    sum_sq_dist_km: f64,
}

impl BodyAccumulator {
    fn new(body: CelestialBody) -> Self {
        Self {
            body,
            count: 0,
            max_lon_arcsec: 0.0,
            sum_sq_lon_arcsec: 0.0,
            max_lat_arcsec: 0.0,
            sum_sq_lat_arcsec: 0.0,
            max_dist_km: 0.0,
            sum_sq_dist_km: 0.0,
        }
    }

    fn accumulate(&mut self, lon_arcsec: f64, lat_arcsec: f64, dist_km: f64) {
        self.count += 1;
        self.max_lon_arcsec = self.max_lon_arcsec.max(lon_arcsec);
        self.sum_sq_lon_arcsec += lon_arcsec * lon_arcsec;
        self.max_lat_arcsec = self.max_lat_arcsec.max(lat_arcsec);
        self.sum_sq_lat_arcsec += lat_arcsec * lat_arcsec;
        self.max_dist_km = self.max_dist_km.max(dist_km);
        self.sum_sq_dist_km += dist_km * dist_km;
    }

    fn finish(self) -> BodyChannelError {
        let n = self.count as f64;
        let rms = |sum_sq: f64| if n > 0.0 { (sum_sq / n).sqrt() } else { 0.0 };
        BodyChannelError {
            body: self.body,
            max_longitude_arcsec: self.max_lon_arcsec,
            rms_longitude_arcsec: rms(self.sum_sq_lon_arcsec),
            max_latitude_arcsec: self.max_lat_arcsec,
            rms_latitude_arcsec: rms(self.sum_sq_lat_arcsec),
            max_distance_km: self.max_dist_km,
            rms_distance_km: rms(self.sum_sq_dist_km),
        }
    }
}

/// Computes per-body accuracy errors between a hold-out slice and an artifact.
///
/// For each hold-out row the artifact is queried at the same epoch and body.
/// Rows where the artifact lookup fails (body missing or out of range) are skipped
/// silently.
///
/// The hold-out epoch's time scale is preserved as-is when building the lookup
/// instant.  The dense de440-backed artifact (v5+) stores segments tagged with
/// `Tdb`; the `Segment::contains` check requires the scale to match, so passing
/// the raw `Tdb`-tagged epoch is the correct path for the current committed
/// artifact.
///
/// Longitude differences are wrapped to ±180° before conversion to arcseconds.
/// Latitude differences are taken directly.  Distance differences are converted
/// from AU to km.
pub fn accuracy_baseline_against(
    holdout: &[SnapshotEntry],
    artifact: &CompressedArtifact,
) -> Vec<BodyChannelError> {
    let mut accumulators: HashMap<String, BodyAccumulator> = HashMap::new();

    for entry in holdout {
        let key = format!("{:?}", entry.body);
        let acc = accumulators
            .entry(key)
            .or_insert_with(|| BodyAccumulator::new(entry.body.clone()));

        // Preserve the entry's time scale exactly so that segment range checks
        // pass.  The dense de440 artifact stores segments with Tdb-tagged instants;
        // converting to Tt here would cause all lookups to fail.
        let lookup_instant = Instant::new(entry.epoch.julian_day, entry.epoch.scale);

        let artifact_result = artifact.lookup_ecliptic(&entry.body, lookup_instant);
        let artifact_coords = match artifact_result {
            Ok(coords) => coords,
            Err(_) => continue, // body missing or out of range — skip this row
        };

        let holdout_coords = coordinates(entry);

        // Longitude diff wrapped to ±180°, then to arcseconds.
        let lon_diff_deg = Angle::from_degrees(
            artifact_coords.longitude.degrees() - holdout_coords.longitude.degrees(),
        )
        .normalized_signed()
        .degrees();
        let lon_arcsec = lon_diff_deg.abs() * 3600.0;

        // Latitude diff (no wrapping needed, bounded ±90°).
        let lat_arcsec =
            (artifact_coords.latitude.degrees() - holdout_coords.latitude.degrees()).abs() * 3600.0;

        // Distance diff in km.
        let dist_km = match (artifact_coords.distance_au, holdout_coords.distance_au) {
            (Some(a), Some(h)) => (a - h).abs() * AU_IN_KM,
            _ => 0.0,
        };

        acc.accumulate(lon_arcsec, lat_arcsec, dist_km);
    }

    // Emit results in a deterministic order (body debug-name alphabetical).
    let mut results: Vec<BodyChannelError> =
        accumulators.into_values().map(|acc| acc.finish()).collect();
    results.sort_by_key(|a| a.label());
    results
}

/// Computes per-body accuracy errors for the committed packaged artifact against
/// the production hold-out corpus.
///
/// This is the SP1 baseline deliverable.
pub fn packaged_artifact_accuracy_baseline() -> Vec<BodyChannelError> {
    let holdout = production_holdout_corpus();
    let artifact = build_packaged_artifact();
    accuracy_baseline_against(holdout, &artifact)
}

/// Returns a deterministic one-line-per-body summary of the packaged-artifact accuracy baseline.
///
/// The string is recomputed on every call and compared to the committed baseline when
/// [`validate_packaged_artifact_accuracy_baseline_summary`] is used.
pub fn packaged_artifact_accuracy_baseline_summary_for_report() -> String {
    let errors = packaged_artifact_accuracy_baseline();
    if errors.is_empty() {
        return "Packaged-artifact accuracy baseline: no hold-out rows matched".to_string();
    }
    let lines: Vec<String> = errors.iter().map(|e| e.summary_line()).collect();
    format!(
        "Packaged-artifact accuracy baseline ({} bodies):\n{}",
        errors.len(),
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use pleiades_backend::{Instant, JulianDay, TimeScale};
    use pleiades_compression::{
        ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel, Segment,
    };

    use super::*;

    fn synthetic_holdout() -> Vec<SnapshotEntry> {
        // Single Sun entry at J2000.0 in ecliptic Cartesian: place Sun at 1 AU along x-axis,
        // giving lon=0°, lat=0°, dist=1 AU.
        let au = AU_IN_KM;
        vec![SnapshotEntry {
            body: CelestialBody::Sun,
            epoch: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            x_km: au,
            y_km: 0.0,
            z_km: 0.0,
        }]
    }

    fn synthetic_artifact() -> CompressedArtifact {
        // Build an artifact whose Sun segment reproduces the same lon/lat/dist exactly.
        let jd = JulianDay::from_days(2_451_545.0);
        let instant = Instant::new(jd, TimeScale::Tt);
        // lon=0°, lat=0°, dist_au=1.0 — constant segment.
        let segment = Segment::new(
            instant,
            instant,
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, 0.0),
                PolynomialChannel::linear(ChannelKind::Latitude, 9, 0.0, 0.0),
                PolynomialChannel::linear(ChannelKind::DistanceAu, 10, 1.0, 1.0),
            ],
        );
        CompressedArtifact::new(
            ArtifactHeader::new("synthetic-test", "synthetic test source"),
            vec![BodyArtifact::new(CelestialBody::Sun, vec![segment])],
        )
    }

    #[test]
    fn baseline_reports_zero_error_for_an_artifact_that_matches_holdout() {
        let errors = accuracy_baseline_against(&synthetic_holdout(), &synthetic_artifact());
        // Should have exactly one body (Sun) with ~zero errors.
        assert_eq!(
            errors.len(),
            1,
            "expected exactly 1 body in synthetic baseline"
        );
        let sun = &errors[0];
        assert!(
            sun.max_longitude_arcsec < 1e-3,
            "Sun max longitude error too large: {} arcsec",
            sun.max_longitude_arcsec
        );
        assert!(
            sun.max_latitude_arcsec < 1e-3,
            "Sun max latitude error too large: {} arcsec",
            sun.max_latitude_arcsec
        );
        assert!(
            sun.max_distance_km < 1.0,
            "Sun max distance error too large: {} km",
            sun.max_distance_km
        );
    }
}
