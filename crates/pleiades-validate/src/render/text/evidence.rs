//! VSOP87 evidence summaries and mean-obliquity frame round-trip rendering.

use std::fmt;

use crate::*;

pub(crate) fn vsop87_canonical_body_evidence(
) -> Option<Vec<pleiades_vsop87::Vsop87CanonicalBodyEvidence>> {
    pleiades_vsop87::canonical_epoch_body_evidence()
}

pub(crate) fn format_vsop87_canonical_evidence_summary() -> String {
    crate::posture::vsop87::evidence::canonical_epoch_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_equatorial_evidence_summary() -> String {
    crate::posture::vsop87::evidence::canonical_epoch_equatorial_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_j2000_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::canonical_j2000_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j2000_ecliptic_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::supported_body_j2000_ecliptic_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j2000_equatorial_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::supported_body_j2000_equatorial_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j1900_ecliptic_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::supported_body_j1900_ecliptic_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_supported_body_j1900_equatorial_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::supported_body_j1900_equatorial_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_mixed_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::canonical_mixed_time_scale_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_j1900_batch_summary() -> String {
    crate::posture::vsop87::batch_parity::canonical_j1900_batch_parity_summary_for_report()
}

pub(crate) fn format_vsop87_body_evidence_summary() -> String {
    crate::posture::vsop87::evidence::source_body_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_source_body_class_evidence_summary() -> String {
    crate::posture::vsop87::batch_parity::source_body_class_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_equatorial_body_class_evidence_summary() -> String {
    crate::posture::vsop87::evidence::canonical_epoch_equatorial_body_class_evidence_summary_for_report()
}

pub(crate) fn format_vsop87_canonical_outlier_note_summary() -> String {
    crate::posture::vsop87::evidence::canonical_epoch_outlier_note_for_report()
}

pub(crate) fn format_validated_vsop87_source_documentation_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

pub(crate) fn format_validated_vsop87_source_documentation_health_summary_for_report(
    summary: &pleiades_vsop87::Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

pub(crate) fn format_vsop87_source_documentation_summary() -> String {
    format_validated_vsop87_source_documentation_summary_for_report(&source_documentation_summary())
}

pub(crate) fn format_vsop87_source_documentation_health_summary() -> String {
    format_validated_vsop87_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}

pub(crate) fn format_vsop87_frame_treatment_summary() -> String {
    crate::posture::vsop87::spec::frame_treatment_summary_for_report()
}

pub(crate) fn format_jpl_frame_treatment_summary() -> String {
    jpl_frame_treatment_summary_for_report()
}

/// Compact validation evidence for the shared mean-obliquity frame round-trip samples.
#[derive(Clone, Debug, PartialEq)]
pub struct MeanObliquityFrameRoundTripSummary {
    pub(crate) sample_count: usize,
    pub(crate) max_longitude_delta_deg: f64,
    pub(crate) max_latitude_delta_deg: f64,
    pub(crate) max_distance_delta_au: f64,
    pub(crate) mean_longitude_delta_deg: f64,
    pub(crate) mean_latitude_delta_deg: f64,
    pub(crate) mean_distance_delta_au: f64,
    pub(crate) percentile_longitude_delta_deg: f64,
    pub(crate) percentile_latitude_delta_deg: f64,
    pub(crate) percentile_distance_delta_au: f64,
}

impl MeanObliquityFrameRoundTripSummary {
    /// Validates the stored round-trip envelope.
    pub fn validate(&self) -> Result<(), String> {
        if self.sample_count == 0 {
            return Err("mean-obliquity frame round-trip summary has no samples".to_string());
        }

        for (label, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "mean-obliquity frame round-trip summary field `{label}` must be a finite non-negative value"
                ));
            }
        }

        let expected = expected_mean_obliquity_frame_round_trip_summary()?;
        if *self != expected {
            return Err(
                "mean-obliquity frame round-trip summary drifted from the canonical sample set"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        format!(
            "{} samples, max |Δlon|={:.12}°, mean |Δlon|={:.12}°, p95 |Δlon|={:.12}°, max |Δlat|={:.12}°, mean |Δlat|={:.12}°, p95 |Δlat|={:.12}°, max |Δdist|={:.12} AU, mean |Δdist|={:.12} AU, p95 |Δdist|={:.12} AU",
            self.sample_count,
            self.max_longitude_delta_deg,
            self.mean_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            self.mean_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            self.max_distance_delta_au,
            self.mean_distance_delta_au,
            self.percentile_distance_delta_au,
        )
    }

    /// Returns the compact round-trip summary line after validating the canonical sample set.
    pub fn validated_summary_line(&self) -> Result<String, String> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for MeanObliquityFrameRoundTripSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical sample corpus used to validate the shared mean-obliquity frame round-trip envelope.
///
/// Downstream tooling can reuse this exact input set instead of reconstructing it from report text.
/// The corpus intentionally covers a near-polar wraparound case so the report evidence exercises the
/// same precision edge that the frame regression tests pin.
pub fn mean_obliquity_frame_round_trip_sample_corpus() -> [(EclipticCoordinates, Instant); 7] {
    [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(123.45),
                pleiades_core::Latitude::from_degrees(-6.75),
                Some(0.123),
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(90.0),
                pleiades_core::Latitude::from_degrees(0.0),
                Some(1.0),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                pleiades_core::Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(315.0),
                pleiades_core::Latitude::from_degrees(18.0),
                Some(4.25),
            ),
            Instant::new(JulianDay::from_days(2_440_587.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(5.0),
                pleiades_core::Latitude::from_degrees(66.0),
                Some(0.75),
            ),
            Instant::new(JulianDay::from_days(2_500_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.875),
                pleiades_core::Latitude::from_degrees(89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(180.0),
                pleiades_core::Latitude::from_degrees(-89.25),
                Some(0.5),
            ),
            Instant::new(JulianDay::from_days(2_450_000.5), TimeScale::Tt),
        ),
    ]
}

pub(crate) fn validate_mean_obliquity_frame_round_trip_sample_corpus(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<(), String> {
    if samples.len() != 7 {
        return Err(format!(
            "mean-obliquity frame round-trip sample corpus must contain 7 samples, found {}",
            samples.len()
        ));
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() > 80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a northern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees() < -80.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a southern polar sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.longitude.degrees() > 350.0)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include a wraparound longitude sample"
                .to_string(),
        );
    }

    if !samples
        .iter()
        .any(|(coordinates, _)| coordinates.latitude.degrees().abs() < 1e-12)
    {
        return Err(
            "mean-obliquity frame round-trip sample corpus must include an equatorial sample"
                .to_string(),
        );
    }

    Ok(())
}

pub(crate) fn mean_obliquity_frame_round_trip_summary_from_samples(
    samples: &[(EclipticCoordinates, Instant)],
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    validate_mean_obliquity_frame_round_trip_sample_corpus(samples)?;

    let mut sample_count = 0usize;
    let mut max_longitude_delta_deg: f64 = 0.0;
    let mut max_latitude_delta_deg: f64 = 0.0;
    let mut max_distance_delta_au: f64 = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut distance_deltas = Vec::with_capacity(samples.len());

    for (ecliptic, instant) in samples.iter().copied() {
        let obliquity = instant.mean_obliquity();
        let round_trip = ecliptic.to_equatorial(obliquity).to_ecliptic(obliquity);
        let longitude_delta_deg =
            (round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs();
        let latitude_delta_deg =
            (round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs();
        let distance_delta_au = (round_trip.distance_au.unwrap_or_default()
            - ecliptic.distance_au.unwrap_or_default())
        .abs();

        if !longitude_delta_deg.is_finite()
            || !latitude_delta_deg.is_finite()
            || !distance_delta_au.is_finite()
        {
            return Err("non-finite round-trip delta".to_string());
        }

        max_longitude_delta_deg = max_longitude_delta_deg.max(longitude_delta_deg);
        max_latitude_delta_deg = max_latitude_delta_deg.max(latitude_delta_deg);
        max_distance_delta_au = max_distance_delta_au.max(distance_delta_au);
        longitude_deltas.push(longitude_delta_deg);
        latitude_deltas.push(latitude_delta_deg);
        distance_deltas.push(distance_delta_au);
        sample_count += 1;
    }

    Ok(MeanObliquityFrameRoundTripSummary {
        sample_count,
        max_longitude_delta_deg,
        max_latitude_delta_deg,
        max_distance_delta_au,
        mean_longitude_delta_deg: arithmetic_mean(&longitude_deltas),
        mean_latitude_delta_deg: arithmetic_mean(&latitude_deltas),
        mean_distance_delta_au: arithmetic_mean(&distance_deltas),
        percentile_longitude_delta_deg: percentile_linear_interpolation(&longitude_deltas, 0.95),
        percentile_latitude_delta_deg: percentile_linear_interpolation(&latitude_deltas, 0.95),
        percentile_distance_delta_au: percentile_linear_interpolation(&distance_deltas, 0.95),
    })
}

pub(crate) fn expected_mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )
}

pub(crate) fn arithmetic_mean(values: &[f64]) -> f64 {
    values.iter().copied().sum::<f64>() / values.len() as f64
}

pub(crate) fn percentile_linear_interpolation(values: &[f64], percentile: f64) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    if sorted.len() == 1 {
        return sorted[0];
    }

    let clamped = percentile.clamp(0.0, 1.0);
    let position = clamped * (sorted.len() - 1) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;

    if lower_index == upper_index {
        sorted[lower_index]
    } else {
        let lower_value = sorted[lower_index];
        let upper_value = sorted[upper_index];
        let fraction = position - lower_index as f64;
        lower_value + (upper_value - lower_value) * fraction
    }
}

/// Computes the shared mean-obliquity frame round-trip validation summary.
pub fn mean_obliquity_frame_round_trip_summary(
) -> Result<MeanObliquityFrameRoundTripSummary, String> {
    let summary = mean_obliquity_frame_round_trip_summary_from_samples(
        &mean_obliquity_frame_round_trip_sample_corpus(),
    )?;
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn format_mean_obliquity_frame_round_trip_summary_for_report(
    summary: &MeanObliquityFrameRoundTripSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}

pub(crate) fn mean_obliquity_frame_round_trip_summary_for_report() -> String {
    match mean_obliquity_frame_round_trip_summary() {
        Ok(summary) => format_mean_obliquity_frame_round_trip_summary_for_report(&summary),
        Err(error) => format!("mean-obliquity frame round-trip unavailable ({error})"),
    }
}
