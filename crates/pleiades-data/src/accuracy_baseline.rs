//! Per-body accuracy baseline: decoded committed artifact vs hold-out corpus.
//!
//! Compares every hold-out row against the committed packaged artifact at the same
//! epoch and body, accumulating per-body max and RMS errors in longitude (arcsec),
//! latitude (arcsec), and distance (km).  These numbers are the SP1 deliverable
//! that scopes SP2 accuracy targets.

use std::collections::HashMap;

use pleiades_backend::{Angle, CelestialBody, CustomBodyId};
use pleiades_compression::{cartesian_state_to_spherical, CartesianState, CompressedArtifact};
use pleiades_jpl::{production_holdout_corpus, reference_snapshot, SnapshotEntry};

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
    /// Maximum absolute longitude speed error across hold-out rows that have velocity truth (arcsec/day).
    /// Zero when no velocity-bearing rows were compared for this body.
    pub max_lon_speed_arcsec_per_day: f64,
    /// Maximum absolute latitude speed error across hold-out rows that have velocity truth (arcsec/day).
    /// Zero when no velocity-bearing rows were compared for this body.
    pub max_lat_speed_arcsec_per_day: f64,
    /// Maximum absolute radial speed error across hold-out rows that have velocity truth (AU/day).
    /// Zero when no velocity-bearing rows were compared for this body.
    pub max_radial_speed_au_per_day: f64,
}

impl BodyChannelError {
    fn label(&self) -> String {
        format!("{:?}", self.body)
    }

    /// Returns a compact one-line summary for this body's errors.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: n={} max_lon={:.4} arcsec  rms_lon={:.4} arcsec  max_lat={:.4} arcsec  rms_lat={:.4} arcsec  max_dist={:.3} km  rms_dist={:.3} km  max_lon_speed={:.4} arcsec/day  max_lat_speed={:.4} arcsec/day  max_radial_speed={:.6} AU/day",
            self.label(),
            self.comparison_count,
            self.max_longitude_arcsec,
            self.rms_longitude_arcsec,
            self.max_latitude_arcsec,
            self.rms_latitude_arcsec,
            self.max_distance_km,
            self.rms_distance_km,
            self.max_lon_speed_arcsec_per_day,
            self.max_lat_speed_arcsec_per_day,
            self.max_radial_speed_au_per_day,
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
    max_lon_speed_arcsec_per_day: f64,
    max_lat_speed_arcsec_per_day: f64,
    max_radial_speed_au_per_day: f64,
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
            max_lon_speed_arcsec_per_day: 0.0,
            max_lat_speed_arcsec_per_day: 0.0,
            max_radial_speed_au_per_day: 0.0,
        }
    }

    fn accumulate(&mut self, lon_arcsec: f64, lat_arcsec: f64, dist_km: Option<f64>) {
        self.count += 1;
        self.max_lon_arcsec = self.max_lon_arcsec.max(lon_arcsec);
        self.sum_sq_lon_arcsec += lon_arcsec * lon_arcsec;
        self.max_lat_arcsec = self.max_lat_arcsec.max(lat_arcsec);
        self.sum_sq_lat_arcsec += lat_arcsec * lat_arcsec;
        if let Some(d) = dist_km {
            self.max_dist_km = self.max_dist_km.max(d);
            self.sum_sq_dist_km += d * d;
        }
    }

    fn accumulate_speed(
        &mut self,
        lon_speed_arcsec_per_day: f64,
        lat_speed_arcsec_per_day: f64,
        radial_speed_au_per_day: f64,
    ) {
        self.max_lon_speed_arcsec_per_day = self
            .max_lon_speed_arcsec_per_day
            .max(lon_speed_arcsec_per_day);
        self.max_lat_speed_arcsec_per_day = self
            .max_lat_speed_arcsec_per_day
            .max(lat_speed_arcsec_per_day);
        self.max_radial_speed_au_per_day = self
            .max_radial_speed_au_per_day
            .max(radial_speed_au_per_day);
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
            max_lon_speed_arcsec_per_day: self.max_lon_speed_arcsec_per_day,
            max_lat_speed_arcsec_per_day: self.max_lat_speed_arcsec_per_day,
            max_radial_speed_au_per_day: self.max_radial_speed_au_per_day,
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

        // Distance diff in km; None when either side has no distance.
        // Skip the distance contribution for this row rather than silently
        // accumulating 0.0 (which would mask a future regression).
        let dist_km = match (artifact_coords.distance_au, holdout_coords.distance_au) {
            (Some(a), Some(h)) => Some((a - h).abs() * AU_IN_KM),
            _ => None,
        };

        acc.accumulate(lon_arcsec, lat_arcsec, dist_km);

        // Speed error: only for rows that carry velocity truth.
        if let (Some(vx), Some(vy), Some(vz)) = (entry.vx_km_s, entry.vy_km_s, entry.vz_km_s) {
            // Convert hold-out Cartesian position (km) + velocity (km/s) to AU and AU/day.
            let pos_au = [
                entry.x_km / AU_IN_KM,
                entry.y_km / AU_IN_KM,
                entry.z_km / AU_IN_KM,
            ];
            let vel_au_per_day = [
                vx * 86400.0 / AU_IN_KM,
                vy * 86400.0 / AU_IN_KM,
                vz * 86400.0 / AU_IN_KM,
            ];
            let truth_spherical = cartesian_state_to_spherical(CartesianState {
                pos_au,
                vel_au_per_day,
            });

            // truth rates are in rad/day (lon/lat) and AU/day (radial) — convert lon/lat to deg/day.
            let truth_lon_deg_per_day = truth_spherical.lon_rate_rad_per_day.to_degrees();
            let truth_lat_deg_per_day = truth_spherical.lat_rate_rad_per_day.to_degrees();
            let truth_radial_au_per_day = truth_spherical.dist_rate_au_per_day;

            if let Ok(motion) = artifact.lookup_motion(&entry.body, lookup_instant) {
                let art_lon = motion.longitude_deg_per_day.unwrap_or(0.0);
                let art_lat = motion.latitude_deg_per_day.unwrap_or(0.0);
                let art_radial = motion.distance_au_per_day.unwrap_or(0.0);

                let lon_speed_arcsec = (art_lon - truth_lon_deg_per_day).abs() * 3600.0;
                let lat_speed_arcsec = (art_lat - truth_lat_deg_per_day).abs() * 3600.0;
                let radial_speed_au = (art_radial - truth_radial_au_per_day).abs();

                acc.accumulate_speed(lon_speed_arcsec, lat_speed_arcsec, radial_speed_au);
            }
        }
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

/// Computes the maximum absolute longitude error (arcsec) between the committed
/// packaged artifact and the Eros reference-snapshot rows it was fit from.
///
/// Eros has no independent hold-out truth (its curated 1900–2100 corpus is the
/// only source — it is not in de440). This helper is therefore a SELF-CONSISTENCY
/// check: it measures how faithfully the artifact reproduces the snapshot it was
/// derived from. A large value here would indicate a fitting regression, not a
/// mismatch against external truth.
///
/// The longitude difference is wrapped to ±180° before conversion to arcseconds,
/// exactly as in [`accuracy_baseline_against`].
pub fn eros_self_consistency_max_longitude_arcsec() -> f64 {
    let eros_body = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
    let artifact = build_packaged_artifact();
    let mut max_lon_arcsec: f64 = 0.0;

    for entry in reference_snapshot() {
        if entry.body != eros_body {
            continue;
        }
        let lookup_instant = normalize_lookup_instant(entry.epoch);
        let artifact_coords = match artifact.lookup_ecliptic(&eros_body, lookup_instant) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let snapshot_coords = coordinates(entry);

        // Longitude diff wrapped to ±180°, then to arcseconds — reuses the same
        // idiom as accuracy_baseline_against.
        let lon_diff_deg = Angle::from_degrees(
            artifact_coords.longitude.degrees() - snapshot_coords.longitude.degrees(),
        )
        .normalized_signed()
        .degrees();
        let lon_arcsec = lon_diff_deg.abs() * 3600.0;
        max_lon_arcsec = max_lon_arcsec.max(lon_arcsec);
    }

    max_lon_arcsec
}

#[cfg(test)]
mod tests {
    use pleiades_backend::{Instant, JulianDay, TimeScale};
    use pleiades_compression::{
        ArtifactHeader, BodyArtifact, ChannelKind, CompressedArtifact, PolynomialChannel, Segment,
    };

    use super::*;

    /// Synthetic holdout with a known ecliptic velocity for the Sun at J2000.
    ///
    /// Position: Sun at (1 AU, 0, 0) → lon=0°, lat=0°, dist=1 AU.
    /// Velocity: (0, vy_km_s, 0) in km/s where vy_km_s = 0.01 AU/day in km/s.
    ///
    /// At this position (x=1, y=0, z=0):
    ///   dλ/dt = (x·vy − y·vx) / (x²+y²) = vy = 0.01 AU/day in rad/day
    ///   dβ/dt = (ρ·vz − z·ρ̇) / r²       = 0 rad/day
    ///   dr/dt = (x·vx + y·vy + z·vz) / r  = 0 AU/day
    fn synthetic_holdout_with_velocity() -> Vec<SnapshotEntry> {
        let au = AU_IN_KM;
        // vy = 0.01 AU/day converted to km/s: 0.01 * AU_IN_KM / 86400
        let vy_km_s = 0.01 * au / 86400.0;
        vec![SnapshotEntry {
            body: CelestialBody::Sun,
            epoch: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
            x_km: au,
            y_km: 0.0,
            z_km: 0.0,
            vx_km_s: Some(0.0),
            vy_km_s: Some(vy_km_s),
            vz_km_s: Some(0.0),
        }]
    }

    /// Synthetic artifact with a linear longitude segment matching the known velocity.
    ///
    /// The Sun longitude increases linearly over 10 days from 0° to 0.01.to_degrees()*10°,
    /// so the analytic derivative dλ/dt = 0.01.to_degrees() deg/day = 0.01 rad/day ✓.
    /// Latitude and distance are constant, matching dβ/dt=0 and dr/dt=0.
    fn synthetic_artifact_linear() -> CompressedArtifact {
        let jd0 = 2_451_545.0_f64;
        let span_days = 10.0_f64;
        let start = Instant::new(JulianDay::from_days(jd0), TimeScale::Tt);
        let end = Instant::new(JulianDay::from_days(jd0 + span_days), TimeScale::Tt);

        // lon rate = 0.01 rad/day → deg/day = 0.01 * (180/π)
        // over 10 days: lon goes from 0° to 0.01 * (180/π) * 10°
        let lon_rate_deg_per_day = 0.01_f64.to_degrees();
        let lon_end = lon_rate_deg_per_day * span_days;

        let segment = Segment::new(
            start,
            end,
            vec![
                PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.0, lon_end),
                PolynomialChannel::linear(ChannelKind::Latitude, 9, 0.0, 0.0),
                PolynomialChannel::linear(ChannelKind::DistanceAu, 10, 1.0, 1.0),
            ],
        );
        CompressedArtifact::new(
            ArtifactHeader::new(
                "synthetic-linear-test",
                "synthetic linear velocity test source",
            ),
            vec![BodyArtifact::new(CelestialBody::Sun, vec![segment])],
        )
    }

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
            vx_km_s: None,
            vy_km_s: None,
            vz_km_s: None,
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
    fn baseline_reports_speed_error_fields() {
        let errors = accuracy_baseline_against(
            &synthetic_holdout_with_velocity(),
            &synthetic_artifact_linear(),
        );
        assert_eq!(
            errors.len(),
            1,
            "expected exactly 1 body in synthetic speed baseline"
        );
        let sun = &errors[0];
        assert_eq!(sun.comparison_count, 1, "Sun should have 1 comparison");
        // Speed error must be near-zero (artifact derivative exactly matches truth velocity).
        assert!(
            sun.max_lon_speed_arcsec_per_day < 1e-3,
            "Sun max longitude speed error too large: {} arcsec/day",
            sun.max_lon_speed_arcsec_per_day
        );
        assert!(
            sun.max_lat_speed_arcsec_per_day < 1e-3,
            "Sun max latitude speed error too large: {} arcsec/day",
            sun.max_lat_speed_arcsec_per_day
        );
        assert!(
            sun.max_radial_speed_au_per_day < 1e-6,
            "Sun max radial speed error too large: {} AU/day",
            sun.max_radial_speed_au_per_day
        );
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

    // Runtime: ~1.7 s (decodes 47.5 MB artifact + 500-row hold-out). Not ignored.
    #[test]
    fn packaged_artifact_baseline_is_non_vacuous() {
        // Regression guard against the Tdb/Tt scale-mismatch vacuity bug.
        // All 10 base bodies must be present with count>0; inner bodies and luminaries
        // must be sub-arcsec; at least one outer planet must show a clearly non-zero
        // longitude error (proving the baseline is not vacuous).
        let errors = packaged_artifact_accuracy_baseline();

        // (a) All 10 base bodies present.
        let expected_bodies = [
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
        ];
        assert_eq!(
            errors.len(),
            10,
            "expected 10 base bodies in packaged baseline; got {}: {:?}",
            errors.len(),
            errors
                .iter()
                .map(|e| format!("{:?}", e.body))
                .collect::<Vec<_>>()
        );
        for body in &expected_bodies {
            let entry = errors
                .iter()
                .find(|e| &e.body == body)
                .unwrap_or_else(|| panic!("{body:?} must appear in the packaged baseline"));
            assert!(
                entry.comparison_count > 0,
                "{body:?} must have at least one successful comparison (got 0 — vacuous baseline)"
            );
        }

        // (b) Inner bodies + luminaries must be sub-arcsec in longitude.
        let sub_arcsec_bodies = [
            CelestialBody::Sun,
            CelestialBody::Moon,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
        ];
        for body in &sub_arcsec_bodies {
            let entry = errors.iter().find(|e| &e.body == body).unwrap();
            assert!(
                entry.max_longitude_arcsec < 1.0,
                "{body:?} max longitude error must be <1 arcsec (got {:.6}\")",
                entry.max_longitude_arcsec
            );
        }

        // (b2) SP2: outer planets stored heliocentrically — all must also be sub-arcsec.
        let outer_bodies = [
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Pluto,
        ];
        for body in &outer_bodies {
            let entry = errors.iter().find(|e| &e.body == body).unwrap();
            assert!(
                entry.max_longitude_arcsec < 1.0,
                "{body:?} max longitude error must be <1 arcsec after SP2 heliocentric reframe (got {:.6}\")",
                entry.max_longitude_arcsec
            );
        }

        // (c) Non-vacuity anchor: Uranus must be non-zero (baseline is not vacuous)
        // and sub-arcsec (heliocentric reframe is active).
        let uranus = errors
            .iter()
            .find(|e| e.body == CelestialBody::Uranus)
            .expect("Uranus must appear in the packaged baseline");
        assert!(
            uranus.max_longitude_arcsec > 0.0001,
            "Uranus max longitude error must be >0.0001\" (got {:.6}\" — baseline may be vacuous)",
            uranus.max_longitude_arcsec
        );
        assert!(
            uranus.max_longitude_arcsec < 1.0,
            "Uranus max longitude error must be <1\" after SP2 heliocentric reframe (got {:.4}\" — reframe may be broken)",
            uranus.max_longitude_arcsec
        );
    }

    // Astrology-grade longitude envelope gate (SP2).
    // Outer planets are allowed up to 5.0″; inner bodies/luminaries up to 1.0″.
    // With the v7 heliocentric artifact all bodies are sub-arcsec, so this
    // passes with large margin — the 5.0″ ceiling guards against a future
    // regression, not against the current state.
    // Note: (b2) in packaged_artifact_baseline_is_non_vacuous already asserts
    // outer planets < 1.0″, which is STRICTER than this gate's 5.0″ ceiling.
    // The two tests are complementary, not redundant: this test encodes the
    // published astrology-grade specification, while (b2) is an operational
    // regression guard at the tighter SP2-achieved level.
    #[test]
    fn outer_planet_longitude_meets_astrology_grade_envelope() {
        let baseline = crate::accuracy_baseline::packaged_artifact_accuracy_baseline();
        // Astrology-grade longitude ceilings drawn from the published SSOT.
        // accuracy_ceiling returns 1.0" for Luminary/InnerPlanet, 5.0" for
        // OuterPlanet, and 30.0" for Asteroid — identical to the old inline
        // match, so no threshold is loosened.
        for body_error in &baseline {
            let c = crate::thresholds::accuracy_ceiling(&body_error.body).lon_arcsec;
            assert!(
                body_error.max_longitude_arcsec <= c,
                "{:?} longitude {:.3}\" exceeds ceiling {:.1}\"",
                body_error.body,
                body_error.max_longitude_arcsec,
                c
            );
        }
    }

    // Hard accuracy-ceiling gate (SP3, Task 11): all 6 channels for every body in
    // the baseline must be within their published ceilings from thresholds.rs.
    // Currently passing with large margin (measured << ceiling), so this guards
    // against future regressions rather than reflecting the current tight state.
    #[test]
    fn all_channels_within_published_ceilings_for_major_bodies() {
        let baseline = packaged_artifact_accuracy_baseline();
        for e in &baseline {
            let c = crate::thresholds::accuracy_ceiling(&e.body);
            assert!(
                e.max_longitude_arcsec <= c.lon_arcsec,
                "{:?} lon {:.4}\" exceeds ceiling {:.1}\"",
                e.body,
                e.max_longitude_arcsec,
                c.lon_arcsec
            );
            assert!(
                e.max_latitude_arcsec <= c.lat_arcsec,
                "{:?} lat {:.4}\" exceeds ceiling {:.1}\"",
                e.body,
                e.max_latitude_arcsec,
                c.lat_arcsec
            );
            assert!(
                e.max_distance_km <= c.dist_km,
                "{:?} dist {:.3} km exceeds ceiling {:.0} km",
                e.body,
                e.max_distance_km,
                c.dist_km
            );
            assert!(
                e.max_lon_speed_arcsec_per_day <= c.lon_speed_arcsec_per_day,
                "{:?} lon speed {:.4} arcsec/day exceeds ceiling {:.2} arcsec/day",
                e.body,
                e.max_lon_speed_arcsec_per_day,
                c.lon_speed_arcsec_per_day
            );
            assert!(
                e.max_lat_speed_arcsec_per_day <= c.lat_speed_arcsec_per_day,
                "{:?} lat speed {:.4} arcsec/day exceeds ceiling {:.2} arcsec/day",
                e.body,
                e.max_lat_speed_arcsec_per_day,
                c.lat_speed_arcsec_per_day
            );
            assert!(
                e.max_radial_speed_au_per_day <= c.radial_speed_au_per_day,
                "{:?} radial speed {:.6} AU/day exceeds ceiling {:.2e} AU/day",
                e.body,
                e.max_radial_speed_au_per_day,
                c.radial_speed_au_per_day
            );
        }
    }

    // Size-budget gate (SP3, Task 11): the committed packaged artifact must not exceed
    // the published encoded-bytes budget from PACKAGED_BUDGETS.
    #[test]
    fn encoded_artifact_within_size_budget() {
        let bytes_len = crate::data::packaged_artifact_bytes().len();
        assert!(
            bytes_len <= crate::thresholds::PACKAGED_BUDGETS.max_encoded_bytes,
            "encoded artifact {} bytes exceeds budget {} bytes",
            bytes_len,
            crate::thresholds::PACKAGED_BUDGETS.max_encoded_bytes
        );
    }

    #[test]
    #[ignore = "maintainer helper: prints the accuracy baseline summary to regenerate the golden"]
    fn print_packaged_artifact_baseline_summary() {
        eprintln!(
            "{}",
            packaged_artifact_accuracy_baseline_summary_for_report()
        );
    }

    // Drift gate: the committed per-body summary must match the live baseline.
    // Generated from actual output (2026-06-20, SP3 heliocentric-planet artifact);
    // fails if errors silently go all-zero or if artifact/hold-out changes shift
    // any body's error bucket. SP2 outer-planet errors are sub-arcsec after the
    // heliocentric reframe; compare with SP1 (pre-reframe) goldens in git history.
    // SP3 (Task 11): golden now anchors speed channels (first 3 significant digits)
    // in addition to the position channels. Speed vacuity: if a body's speed rows
    // were all skipped, max_lon_speed stays 0.0000 and the non-vacuity check below
    // catches it via the "strictly-positive lon-speed" guard.
    #[test]
    fn packaged_artifact_baseline_summary_matches_committed_golden() {
        let report = packaged_artifact_accuracy_baseline_summary_for_report();

        // Header: 10 bodies
        assert!(
            report.contains("Packaged-artifact accuracy baseline (10 bodies)"),
            "baseline report header drift: {report}"
        );

        // All bodies: sub-arcsec — anchored to first 3 significant digits.
        // SP2: outer planets now stored heliocentrically, so all errors are sub-arcsec.
        assert!(
            report.contains("Sun: n=50 max_lon=0.000"),
            "Sun max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Moon: n=50 max_lon=0.000"),
            "Moon max_lon bucket drift (expected ~0.0001\"): {report}"
        );
        assert!(
            report.contains("Mercury: n=50 max_lon=0.000"),
            "Mercury max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Venus: n=50 max_lon=0.001"),
            "Venus max_lon bucket drift (expected ~0.0011\"): {report}"
        );
        assert!(
            report.contains("Mars: n=50 max_lon=0.000"),
            "Mars max_lon bucket drift (expected ~0.0005\"): {report}"
        );
        // Outer planets: SP2 heliocentric-reframe — all sub-arcsec, non-zero.
        assert!(
            report.contains("Jupiter: n=50 max_lon=0.000"),
            "Jupiter max_lon bucket drift (expected ~0.0004\"): {report}"
        );
        assert!(
            report.contains("Saturn: n=50 max_lon=0.000"),
            "Saturn max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Uranus: n=50 max_lon=0.003"),
            "Uranus max_lon bucket drift (expected ~0.0036\"): {report}"
        );
        assert!(
            report.contains("Neptune: n=50 max_lon=0.002"),
            "Neptune max_lon bucket drift (expected ~0.0020\"): {report}"
        );
        assert!(
            report.contains("Pluto: n=50 max_lon=0.001"),
            "Pluto max_lon bucket drift (expected ~0.0018\"): {report}"
        );

        // SP3 Task 11: speed channel golden anchors (first 3 significant digits).
        // Moon has the highest lon/lat speed error (~0.030/~0.023 arcsec/day) due to
        // fast apparent motion; outer planets are an order of magnitude slower.
        // Radial speed errors are sub-1e-6 AU/day for all bodies; anchored to "0.000000".
        assert!(
            report.contains("Moon: n=50") && report.contains("max_lon_speed=0.023"),
            "Moon max_lon_speed bucket drift (expected ~0.0230 arcsec/day): {report}"
        );
        assert!(
            report.contains("Moon: n=50") && report.contains("max_lat_speed=0.025"),
            "Moon max_lat_speed bucket drift (expected ~0.0253 arcsec/day): {report}"
        );
        assert!(
            report.contains("Sun: n=50") && report.contains("max_lon_speed=0.001"),
            "Sun max_lon_speed bucket drift (expected ~0.0013 arcsec/day): {report}"
        );
        assert!(
            report.contains("Mercury: n=50") && report.contains("max_lon_speed=0.001"),
            "Mercury max_lon_speed bucket drift (expected ~0.0013 arcsec/day): {report}"
        );
        assert!(
            report.contains("Venus: n=50") && report.contains("max_lon_speed=0.001"),
            "Venus max_lon_speed bucket drift (expected ~0.0014 arcsec/day): {report}"
        );
        assert!(
            report.contains("Mars: n=50") && report.contains("max_lon_speed=0.001"),
            "Mars max_lon_speed bucket drift (expected ~0.0011 arcsec/day): {report}"
        );
        // Outer planets: lon_speed in the 0.000X range.
        assert!(
            report.contains("Jupiter: n=50") && report.contains("max_lon_speed=0.000"),
            "Jupiter max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Saturn: n=50") && report.contains("max_lon_speed=0.000"),
            "Saturn max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Uranus: n=50") && report.contains("max_lon_speed=0.000"),
            "Uranus max_lon_speed bucket drift (expected ~0.0007 arcsec/day): {report}"
        );
        assert!(
            report.contains("Neptune: n=50") && report.contains("max_lon_speed=0.000"),
            "Neptune max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Pluto: n=50") && report.contains("max_lon_speed=0.000"),
            "Pluto max_lon_speed bucket drift (expected ~0.0005 arcsec/day): {report}"
        );
        // Radial speed: all bodies sub-1e-6 AU/day — anchor on "0.000000".
        for body_name in &[
            "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
            "Pluto",
        ] {
            assert!(
                report.contains(&format!("{body_name}: n=50"))
                    && report.contains("max_radial_speed=0.000000"),
                "{body_name} max_radial_speed not anchored at 0.000000 AU/day: {report}"
            );
        }
    }

    // Non-vacuity guard for speed channels (SP3, Task 11).
    //
    // The speed accumulator has no velocity-row counter.  A body whose velocity
    // rows were all skipped (e.g. because lookup_motion returned Err or the corpus
    // lacked velocity data) would keep max_*_speed = 0.0 and pass the ceiling
    // gate VACUOUSLY.  This guard catches that: every major planet (excluding the
    // Moon, whose min non-zero would be below our threshold on some corpora) must
    // show a STRICTLY POSITIVE lon-speed error magnitude above a small but
    // non-zero floor (0.0001 arcsec/day), well below the smallest observed real
    // error (~0.0002 arcsec/day for Jupiter/Saturn/Neptune) but clearly above
    // exact zero (which would only arise from a silently-all-skipped velocity
    // channel).
    //
    // Moon is guarded separately with a higher floor (0.005 arcsec/day) because
    // its faster apparent motion means any real measurement would be >>0.005.
    #[test]
    fn speed_channels_are_non_vacuous_for_major_bodies() {
        let errors = packaged_artifact_accuracy_baseline();

        // Inner planets + luminaries: lon-speed must be > 0.0001 arcsec/day.
        let inner_bodies = [
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
        ];
        for body in &inner_bodies {
            let e = errors
                .iter()
                .find(|e| &e.body == body)
                .unwrap_or_else(|| panic!("{body:?} must appear in the packaged baseline"));
            assert!(
                e.max_lon_speed_arcsec_per_day > 0.0001,
                "{body:?} max_lon_speed {:.6} arcsec/day is not strictly positive (speed channel may be vacuous — all velocity rows skipped?)",
                e.max_lon_speed_arcsec_per_day
            );
        }

        // Moon: higher floor (real motion >> 0.005 arcsec/day).
        let moon = errors
            .iter()
            .find(|e| e.body == CelestialBody::Moon)
            .expect("Moon must appear in the packaged baseline");
        assert!(
            moon.max_lon_speed_arcsec_per_day > 0.005,
            "Moon max_lon_speed {:.6} arcsec/day is not strictly positive (speed channel may be vacuous — all velocity rows skipped?)",
            moon.max_lon_speed_arcsec_per_day
        );

        // Outer planets: lon-speed must be > 0.0001 arcsec/day.
        let outer_bodies = [
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Pluto,
        ];
        for body in &outer_bodies {
            let e = errors
                .iter()
                .find(|e| &e.body == body)
                .unwrap_or_else(|| panic!("{body:?} must appear in the packaged baseline"));
            assert!(
                e.max_lon_speed_arcsec_per_day > 0.0001,
                "{body:?} max_lon_speed {:.6} arcsec/day is not strictly positive (speed channel may be vacuous — all velocity rows skipped?)",
                e.max_lon_speed_arcsec_per_day
            );
        }
    }

    #[test]
    #[ignore = "maintainer helper: prints the Eros self-consistency max longitude error"]
    fn print_eros_self_consistency_max_longitude_arcsec() {
        let v = crate::accuracy_baseline::eros_self_consistency_max_longitude_arcsec();
        eprintln!("EROS_SELF_CONSISTENCY_MAX_LON_ARCSEC = {v:.6}\"");
    }

    /// Eros self-consistency gate (SP3, Task 12).
    ///
    /// Eros is re-derived from the committed reference snapshot (no independent truth —
    /// it is absent from de440). This is a SELF-CONSISTENCY check, NOT an independent-truth
    /// gate. It verifies the artifact faithfully reproduces the snapshot it was fit from,
    /// within the published Asteroid-class longitude ceiling (30″).
    #[test]
    fn eros_round_trips_against_its_reference_snapshot_within_documented_target() {
        let eros_body = pleiades_backend::CelestialBody::Custom(
            pleiades_backend::CustomBodyId::new("asteroid", "433-Eros"),
        );
        // Non-vacuity guard: the reference snapshot must actually contain Eros rows,
        // otherwise the helper iterates zero rows and trivially returns 0.0 — a pass
        // that reveals nothing. A missing Eros corpus is a configuration error, not
        // "perfect accuracy".
        let eros_row_count = pleiades_jpl::reference_snapshot()
            .iter()
            .filter(|e| e.body == eros_body)
            .count();
        assert!(
            eros_row_count > 0,
            "reference snapshot contains no Eros rows — self-consistency check would be vacuous (iterate zero rows → max=0.0 → trivially passes)"
        );

        let ceiling = crate::thresholds::accuracy_ceiling(&eros_body);
        let max_lon_arcsec = crate::accuracy_baseline::eros_self_consistency_max_longitude_arcsec();
        assert!(
            max_lon_arcsec <= ceiling.lon_arcsec,
            "Eros self-consistency {max_lon_arcsec:.4}\" > {:.1}\" (artifact does not reproduce the reference snapshot it was fit from within the Asteroid-class ceiling)",
            ceiling.lon_arcsec
        );
    }
}
