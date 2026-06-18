//! Per-body accuracy baseline: decoded committed artifact vs hold-out corpus.
//!
//! Compares every hold-out row against the committed packaged artifact at the same
//! epoch and body, accumulating per-body max and RMS errors in longitude (arcsec),
//! latitude (arcsec), and distance (km).  These numbers are the SP1 deliverable
//! that scopes SP2 accuracy targets.

use std::collections::HashMap;

use pleiades_backend::{Angle, CelestialBody};
use pleiades_compression::CompressedArtifact;
use pleiades_jpl::{production_holdout_corpus, SnapshotEntry};

use crate::regenerate::{build_packaged_artifact, coordinates, normalize_lookup_instant};
use crate::AU_IN_KM;

/// Per-body accuracy summary comparing the artifact to an independent hold-out corpus.
#[derive(Clone, Debug)]
pub struct BodyChannelError {
    /// Body these errors apply to.
    pub body: CelestialBody,
    /// Number of hold-out rows that were successfully compared for this body.
    /// A value of zero would indicate a vacuous baseline (no rows matched), but
    /// `accuracy_baseline_against` excludes bodies with zero comparisons entirely.
    pub comparison_count: usize,
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
            "{}: n={} max_lon={:.4} arcsec  rms_lon={:.4} arcsec  max_lat={:.4} arcsec  rms_lat={:.4} arcsec  max_dist={:.3} km  rms_dist={:.3} km",
            self.label(),
            self.comparison_count,
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
            comparison_count: self.count,
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
/// silently — but if ALL rows for a body are skipped, that body is excluded from the
/// result rather than silently reporting zero error.  A caller asserting non-zero
/// error on a specific body will detect a vacuous baseline.
///
/// The lookup instant is normalized to `TimeScale::Tt` (same julian_day, Tdb
/// relabelled Tt) to match the committed artifact's segment tag convention.
/// `normalize_lookup_instant` mirrors the runtime packaged-lookup path and
/// `Segment::contains` requires scale equality.
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

        // Normalize to Tt (same julian_day) — the committed artifact's segment
        // boundaries are Tt-tagged; Segment::contains requires scale equality.
        let lookup_instant = normalize_lookup_instant(entry.epoch);

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

    // Vacuity guard: only emit bodies where at least one row was successfully compared.
    // Bodies whose every lookup failed (e.g. due to a scale mismatch) are excluded rather
    // than silently emitted as all-zero error — zero here means "not measured", not "perfect".
    let mut results: Vec<BodyChannelError> = accumulators
        .into_values()
        .filter(|acc| acc.count > 0)
        .map(|acc| acc.finish())
        .collect();
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
        assert_eq!(sun.comparison_count, 1, "Sun should have 1 comparison");
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

    #[test]
    fn baseline_excludes_body_when_all_lookups_fail() {
        // Vacuity guard: if the artifact has NO segment for the holdout body, the body
        // must be absent from results rather than silently appearing as zero error.
        let holdout = synthetic_holdout(); // Sun at J2000.0
                                           // Empty artifact — lookup_ecliptic will fail with MissingBody for Sun.
        let empty_artifact = CompressedArtifact::new(
            ArtifactHeader::new("empty-test", "empty test source"),
            vec![],
        );
        let errors = accuracy_baseline_against(&holdout, &empty_artifact);
        assert!(
            errors.is_empty(),
            "vacuity guard must exclude bodies with zero successful comparisons; got {} entries",
            errors.len()
        );
    }

    #[test]
    #[ignore = "reads 47.5MB artifact + 500-row hold-out; run with -- --ignored"]
    fn packaged_artifact_baseline_is_non_vacuous() {
        // Regression guard against the Tdb/Tt scale-mismatch vacuity bug:
        // the real packaged baseline must include Uranus with a measurable
        // (non-zero) longitude error.  A vacuous baseline would return no Uranus
        // entry (all lookups skipped) or a zero error.
        let errors = packaged_artifact_accuracy_baseline();
        let uranus = errors
            .iter()
            .find(|e| e.body == CelestialBody::Uranus)
            .expect("Uranus must appear in the packaged baseline (hold-out covers 10 base bodies)");
        assert!(
            uranus.comparison_count > 0,
            "Uranus must have at least one successful comparison (got 0 — vacuous baseline)"
        );
        assert!(
            uranus.max_longitude_arcsec > 1.0,
            "Uranus max longitude error must be >1 arcsec (expected ~156\" for SP1 draft; got {:.4}\" — baseline may be vacuous)",
            uranus.max_longitude_arcsec
        );
    }
}
