//! Construction, validation, and benchmarking of the packaged artifact
//! inspection report.
//!
//! Moved verbatim from the parent `artifact` module: the cached packaged-report
//! accessors, the `ArtifactInspectionReport` builders and validators, the
//! decode/lookup benchmark builders, the comparison-corpus construction, and the
//! boundary-continuity envelope aggregation.

use std::sync::OnceLock;
use std::time::Instant as StdInstant;

use crate::artifact::*;
use crate::{compare_backends, default_candidate_backend, ValidationCorpus};
use pleiades_compression::CompressedArtifact;
use pleiades_core::{
    Angle, Apparentness, CelestialBody, CoordinateFrame, EclipticCoordinates, EphemerisBackend,
    EphemerisError, EphemerisErrorKind, EphemerisRequest, Instant, JulianDay, ZodiacMode,
};
use pleiades_data::{packaged_artifact, packaged_artifact_bytes, packaged_backend};

fn packaged_artifact_encoded_bytes() -> &'static [u8] {
    packaged_artifact_bytes()
}

fn packaged_artifact_inspection_report() -> &'static ArtifactInspectionReport {
    static REPORT: OnceLock<ArtifactInspectionReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let artifact = packaged_artifact();
        let encoded = packaged_artifact_encoded_bytes();
        ArtifactInspectionReport::from_encoded_artifact(artifact, encoded, encoded.len())
            .expect("packaged artifact inspection report should build")
    })
}

fn packaged_artifact_report_text() -> &'static String {
    static TEXT: OnceLock<String> = OnceLock::new();
    TEXT.get_or_init(|| packaged_artifact_inspection_report().to_string())
}

fn packaged_artifact_summary_text() -> &'static String {
    static TEXT: OnceLock<String> = OnceLock::new();
    TEXT.get_or_init(|| render_artifact_summary_text(packaged_artifact_inspection_report()))
}

impl ArtifactInspectionReport {
    #[cfg(test)]
    pub(crate) fn from_artifact(
        artifact: &CompressedArtifact,
        encoded_bytes: usize,
    ) -> Result<Self, ArtifactInspectionError> {
        let encoded = artifact.encode()?;
        Self::from_encoded_artifact(artifact, &encoded, encoded_bytes)
    }

    fn from_encoded_artifact(
        artifact: &CompressedArtifact,
        encoded: &[u8],
        encoded_bytes: usize,
    ) -> Result<Self, ArtifactInspectionError> {
        let decoded = CompressedArtifact::decode(encoded)?;
        let mut bodies = Vec::with_capacity(decoded.bodies.len());
        let mut segment_count = 0usize;
        let mut earliest: Option<Instant> = None;
        let mut latest: Option<Instant> = None;

        for body in &decoded.bodies {
            let inspection = inspect_body(&decoded, body)?;
            segment_count += inspection.segment_count;
            earliest = Some(match earliest {
                Some(current)
                    if current.julian_day.days() <= inspection.earliest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.earliest,
                None => inspection.earliest,
            });
            latest = Some(match latest {
                Some(current)
                    if current.julian_day.days() >= inspection.latest.julian_day.days() =>
                {
                    current
                }
                Some(_) => inspection.latest,
                None => inspection.latest,
            });
            bodies.push(inspection);
        }

        let comparison_corpus = artifact_model_comparison_corpus(&decoded);
        let model_comparison = compare_backends(
            &default_candidate_backend(),
            &packaged_backend(),
            &comparison_corpus,
        )?;
        let decode_benchmark = benchmark_packaged_artifact_decode(1)?;
        let lookup_benchmark = benchmark_packaged_artifact_lookup(1)?;
        let batch_lookup_benchmark = benchmark_packaged_artifact_batch_lookup(1)?;
        let residual_segment_count = decoded.residual_segment_count();
        let residual_bodies = decoded.residual_bodies();

        let report = Self {
            generation_label: decoded.header.generation_label,
            source: decoded.header.source,
            version: decoded.header.version,
            endian_policy: decoded.header.endian_policy,
            checksum: decoded.checksum,
            encoded_bytes,
            roundtrip_ok: true,
            checksum_ok: true,
            body_count: decoded.bodies.len(),
            segment_count,
            residual_segment_count,
            residual_bodies,
            earliest: earliest.unwrap_or_else(|| artifact_first_instant(artifact)),
            latest: latest.unwrap_or_else(|| artifact_first_instant(artifact)),
            model_comparison,
            decode_benchmark,
            lookup_benchmark,
            batch_lookup_benchmark,
            bodies,
        };
        report.validate()?;
        Ok(report)
    }

    /// Returns a compact one-line summary of the inspection report.
    pub fn summary_line(&self) -> String {
        format!(
            "artifact inspection: {} bundled bodies, {} segments, residual-bearing segments: {}, residual-bearing bodies: {}, body classes: {}; coverage: {} → {}, roundtrip={}, checksum={}, encoded bytes={}",
            self.body_count,
            self.segment_count,
            self.residual_segment_count,
            format_residual_bodies(&self.residual_bodies),
            format_body_class_coverage(self),
            self.earliest,
            self.latest,
            yes_no(self.roundtrip_ok),
            yes_no(self.checksum_ok),
            self.encoded_bytes,
        )
    }

    /// Validates the inspection report before returning the compact summary line.
    pub fn validated_summary_line(&self) -> Result<String, ArtifactInspectionError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    pub fn validate(&self) -> Result<(), ArtifactInspectionError> {
        if !self.roundtrip_ok {
            return Err(report_validation_error(
                "artifact inspection report roundtrip status drifted from the decoded artifact",
            ));
        }

        if !self.checksum_ok {
            return Err(report_validation_error(
                "artifact inspection report checksum status drifted from the decoded artifact",
            ));
        }

        if self.body_count != self.bodies.len() {
            return Err(report_validation_error(
                "artifact inspection report field `body_count` does not match the inspected body set",
            ));
        }

        let expected_segment_count: usize = self.bodies.iter().map(|body| body.segment_count).sum();
        if self.segment_count != expected_segment_count {
            return Err(report_validation_error(
                "artifact inspection report field `segment_count` does not match the inspected body set",
            ));
        }

        if self.residual_segment_count == 0 {
            if !self.residual_bodies.is_empty() {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` must stay empty when no residual segments are present",
                ));
            }
        } else {
            if self.residual_bodies.is_empty() {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` must name the residual-bearing bodies",
                ));
            }
            if self.residual_segment_count > self.segment_count {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_segment_count` exceeds the total inspected segments",
                ));
            }

            let expected_residual_bodies = self
                .bodies
                .iter()
                .filter(|inspection| inspection.residual_segment_count > 0)
                .map(|inspection| inspection.body.clone())
                .collect::<Vec<_>>();

            if self.residual_bodies != expected_residual_bodies {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_bodies` does not match the inspected residual-bearing body set",
                ));
            }

            let expected_residual_segment_count: usize = self
                .bodies
                .iter()
                .map(|inspection| inspection.residual_segment_count)
                .sum();
            if self.residual_segment_count != expected_residual_segment_count {
                return Err(report_validation_error(
                    "artifact inspection report field `residual_segment_count` does not match the inspected residual-bearing segment count",
                ));
            }
        }

        if self
            .residual_bodies
            .iter()
            .enumerate()
            .any(|(index, body)| self.residual_bodies[..index].contains(body))
        {
            return Err(report_validation_error(
                "artifact inspection report field `residual_bodies` contains duplicate entries",
            ));
        }

        if self.residual_bodies.iter().any(|body| {
            !self
                .bodies
                .iter()
                .any(|inspection| inspection.body == *body)
        }) {
            return Err(report_validation_error(
                "artifact inspection report field `residual_bodies` references a body not present in the inspected artifact",
            ));
        }

        if self.earliest.julian_day.days() > self.latest.julian_day.days() {
            return Err(report_validation_error(
                "artifact inspection report coverage bounds are inverted",
            ));
        }

        self.model_comparison.summary.validate()?;
        self.decode_benchmark.validate()?;
        self.lookup_benchmark.validate()?;
        self.batch_lookup_benchmark.validate()?;

        if self.decode_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.decode_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark source does not match the decoded artifact header",
            ));
        }

        if self.decode_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report decode benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        if self.lookup_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.lookup_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark source does not match the decoded artifact header",
            ));
        }

        if self.lookup_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report lookup benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        if self.batch_lookup_benchmark.artifact_label != self.generation_label {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark artifact label does not match the decoded artifact header",
            ));
        }

        if self.batch_lookup_benchmark.source != self.source {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark source does not match the decoded artifact header",
            ));
        }

        if self.batch_lookup_benchmark.encoded_bytes != self.encoded_bytes {
            return Err(report_validation_error(
                "artifact inspection report batch lookup benchmark encoded byte count does not match the decoded artifact",
            ));
        }

        artifact_boundary_envelope_summary(self)
            .validate()
            .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
        Ok(())
    }
}

/// Renders the bundled artifact validation report.
pub fn render_artifact_report() -> Result<String, ArtifactInspectionError> {
    Ok(packaged_artifact_report_text().clone())
}

/// Returns the aggregate packaged-artifact boundary envelope used by reports.
pub fn artifact_boundary_envelope_summary_for_report(
) -> Result<ArtifactBoundaryEnvelopeSummary, ArtifactInspectionError> {
    let report = packaged_artifact_inspection_report();
    let summary = artifact_boundary_envelope_summary(report);
    summary
        .validate()
        .map_err(ArtifactInspectionError::BoundaryEnvelope)?;
    Ok(summary)
}

/// Renders a compact summary of the bundled artifact validation report.
pub fn render_artifact_summary() -> Result<String, ArtifactInspectionError> {
    Ok(packaged_artifact_summary_text().clone())
}

/// Returns the compact artifact inspection summary used by release-facing reports.
pub fn artifact_inspection_summary_for_report() -> Result<String, ArtifactInspectionError> {
    packaged_artifact_inspection_report().validated_summary_line()
}

pub(crate) fn benchmark_packaged_artifact_decode(
    rounds: usize,
) -> Result<ArtifactDecodeBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let start = StdInstant::now();
    for _ in 0..rounds {
        std::hint::black_box(CompressedArtifact::decode(encoded)?);
    }
    let elapsed = start.elapsed();

    let report = ArtifactDecodeBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        rounds,
        sample_count: 1,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn benchmark_packaged_artifact_lookup(
    rounds: usize,
) -> Result<ArtifactLookupBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let corpus = artifact_timing_corpus(artifact);
    let sample_count = corpus.requests.len();
    let start = StdInstant::now();
    for _ in 0..rounds {
        for request in &corpus.requests {
            std::hint::black_box(artifact.lookup_ecliptic(&request.body, request.instant)?);
        }
    }
    let elapsed = start.elapsed();

    let report = ArtifactLookupBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        corpus_name: corpus.name,
        rounds,
        sample_count,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn benchmark_packaged_artifact_batch_lookup(
    rounds: usize,
) -> Result<ArtifactBatchLookupBenchmarkReport, ArtifactInspectionError> {
    let artifact = packaged_artifact();
    let encoded = packaged_artifact_encoded_bytes();
    let corpus = artifact_timing_corpus(artifact);
    let batch_size = corpus.requests.len();
    let backend = packaged_backend();
    let start = StdInstant::now();
    for _ in 0..rounds {
        let results = backend.positions(&corpus.requests)?;
        std::hint::black_box(results);
    }
    let elapsed = start.elapsed();

    let report = ArtifactBatchLookupBenchmarkReport {
        artifact_label: artifact.header.generation_label.clone(),
        source: artifact.header.source.clone(),
        corpus_name: corpus.name,
        rounds,
        batch_size,
        encoded_bytes: encoded.len(),
        elapsed,
    };
    report.validate()?;

    Ok(report)
}

pub(crate) fn packaged_artifact_corpus() -> ValidationCorpus {
    artifact_comparison_corpus(packaged_artifact())
}

fn artifact_timing_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    let mut corpus = artifact_model_comparison_corpus(artifact);
    corpus.name = "Packaged artifact timing subset".to_string();
    corpus.description = "Reduced timing subset of the packaged artifact comparison corpus.";
    corpus.requests.truncate(1);
    corpus
}

fn artifact_model_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |body| !matches!(body, CelestialBody::Custom(_)))
}

fn artifact_comparison_corpus(artifact: &CompressedArtifact) -> ValidationCorpus {
    artifact_comparison_corpus_filtered(artifact, |_| true)
}

fn artifact_comparison_corpus_filtered<F>(
    artifact: &CompressedArtifact,
    include_body: F,
) -> ValidationCorpus
where
    F: Fn(&CelestialBody) -> bool,
{
    let mut requests = Vec::new();

    for body in &artifact.bodies {
        if !include_body(&body.body) {
            continue;
        }

        let (Some(first), Some(last)) = (body.segments.first(), body.segments.last()) else {
            continue;
        };
        let midpoint = midpoint(first.start, last.end);
        for instant in [first.start, midpoint, last.end] {
            requests.push(EphemerisRequest {
                body: body.body.clone(),
                instant,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            });
        }
    }

    ValidationCorpus {
        name: "Packaged artifact error envelope".to_string(),
        description: "Comparison corpus built from packaged artifact body coverage endpoints and midpoints so the bundled data can be measured against the algorithmic baseline without turning compact reports into full validation runs.",
        apparentness: Apparentness::Mean,
        requests,
    }
}

fn representative_indexes(len: usize) -> Vec<usize> {
    match len {
        0 => Vec::new(),
        1 => vec![0],
        2 => vec![0, 1],
        _ => vec![0, len / 2, len - 1],
    }
}

fn inspect_body(
    artifact: &CompressedArtifact,
    body: &pleiades_compression::BodyArtifact,
) -> Result<ArtifactBodyInspection, ArtifactInspectionError> {
    let mut sample_count = 0usize;
    let mut boundary_checks = 0usize;
    let mut sum_boundary_longitude_delta_deg = 0.0;
    let mut sum_boundary_longitude_delta_deg_sq = 0.0;
    let mut sum_boundary_latitude_delta_deg = 0.0;
    let mut sum_boundary_latitude_delta_deg_sq = 0.0;
    let mut sum_boundary_distance_delta_au: Option<f64> = None;
    let mut sum_boundary_distance_delta_au_sq: Option<f64> = None;
    let mut boundary_distance_checks = 0usize;
    let mut max_boundary_longitude_delta_deg: f64 = 0.0;
    let mut max_boundary_latitude_delta_deg: f64 = 0.0;
    let mut max_boundary_distance_delta_au: Option<f64> = None;

    let sampled_segment_indexes = representative_indexes(body.segments.len());
    for segment_index in sampled_segment_indexes {
        let segment = &body.segments[segment_index];
        let midpoint = midpoint(segment.start, segment.end);
        for instant in [segment.start, midpoint, segment.end] {
            artifact.lookup_ecliptic(&body.body, instant)?;
            sample_count += 1;
        }
    }

    let boundary_count = body.segments.len().saturating_sub(1);
    for boundary_index in representative_indexes(boundary_count) {
        let pair = &body.segments[boundary_index..=boundary_index + 1];
        let left = artifact.lookup_ecliptic(&body.body, pair[0].end)?;
        let right = artifact.lookup_ecliptic(&body.body, pair[1].start)?;
        let delta = boundary_delta(&left, &right);
        boundary_checks += 1;
        sum_boundary_longitude_delta_deg += delta.longitude_delta_deg;
        sum_boundary_longitude_delta_deg_sq += delta.longitude_delta_deg.powi(2);
        sum_boundary_latitude_delta_deg += delta.latitude_delta_deg;
        sum_boundary_latitude_delta_deg_sq += delta.latitude_delta_deg.powi(2);
        max_boundary_longitude_delta_deg =
            max_boundary_longitude_delta_deg.max(delta.longitude_delta_deg);
        max_boundary_latitude_delta_deg =
            max_boundary_latitude_delta_deg.max(delta.latitude_delta_deg);
        max_boundary_distance_delta_au =
            match (max_boundary_distance_delta_au, delta.distance_delta_au) {
                (Some(current), Some(next)) => Some(current.max(next)),
                (None, Some(next)) => Some(next),
                (current, None) => current,
            };
        if let Some(distance_delta_au) = delta.distance_delta_au {
            boundary_distance_checks += 1;
            sum_boundary_distance_delta_au =
                Some(sum_boundary_distance_delta_au.unwrap_or(0.0) + distance_delta_au);
            sum_boundary_distance_delta_au_sq =
                Some(sum_boundary_distance_delta_au_sq.unwrap_or(0.0) + distance_delta_au.powi(2));
        }
    }

    let earliest = body
        .segments
        .first()
        .map(|segment| segment.start)
        .unwrap_or_else(|| artifact_first_instant(artifact));
    let latest = body
        .segments
        .last()
        .map(|segment| segment.end)
        .unwrap_or_else(|| artifact_first_instant(artifact));
    let mut min_segment_span_days: f64 = f64::INFINITY;
    let mut max_segment_span_days: f64 = 0.0;
    let mut sum_segment_span_days: f64 = 0.0;
    for segment in &body.segments {
        let span_days = segment.end.julian_day.days() - segment.start.julian_day.days();
        min_segment_span_days = min_segment_span_days.min(span_days);
        max_segment_span_days = max_segment_span_days.max(span_days);
        sum_segment_span_days += span_days;
    }
    if min_segment_span_days.is_infinite() {
        min_segment_span_days = 0.0;
    }
    let mean_segment_span_days = if body.segments.is_empty() {
        0.0
    } else {
        sum_segment_span_days / body.segments.len() as f64
    };

    Ok(ArtifactBodyInspection {
        body: body.body.clone(),
        segment_count: body.segments.len(),
        earliest,
        latest,
        sample_count,
        min_segment_span_days,
        max_segment_span_days,
        mean_segment_span_days,
        residual_segment_count: body
            .segments
            .iter()
            .filter(|segment| !segment.residual_channels.is_empty())
            .count(),
        boundary_checks,
        sum_boundary_longitude_delta_deg,
        sum_boundary_longitude_delta_deg_sq,
        sum_boundary_latitude_delta_deg,
        sum_boundary_latitude_delta_deg_sq,
        sum_boundary_distance_delta_au,
        sum_boundary_distance_delta_au_sq,
        boundary_distance_checks,
        max_boundary_longitude_delta_deg,
        max_boundary_latitude_delta_deg,
        max_boundary_distance_delta_au,
    })
}

fn report_validation_error(message: &'static str) -> ArtifactInspectionError {
    ArtifactInspectionError::Validation(EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        message,
    ))
}

fn midpoint(start: Instant, end: Instant) -> Instant {
    let start_days = start.julian_day.days();
    let end_days = end.julian_day.days();
    Instant::new(
        JulianDay::from_days((start_days + end_days) / 2.0),
        start.scale,
    )
}

fn artifact_first_instant(artifact: &CompressedArtifact) -> Instant {
    artifact
        .bodies
        .iter()
        .flat_map(|body| body.segments.iter())
        .map(|segment| segment.start)
        .min_by(|left, right| {
            left.julian_day
                .days()
                .partial_cmp(&right.julian_day.days())
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .unwrap_or_else(|| {
            Instant::new(
                pleiades_core::JulianDay::from_days(0.0),
                pleiades_core::TimeScale::Tt,
            )
        })
}

struct BoundaryDelta {
    longitude_delta_deg: f64,
    latitude_delta_deg: f64,
    distance_delta_au: Option<f64>,
}

pub(crate) fn artifact_boundary_envelope_summary(
    report: &ArtifactInspectionReport,
) -> ArtifactBoundaryEnvelopeSummary {
    let mut summary = ArtifactBoundaryEnvelopeSummary {
        body_count: report.bodies.len(),
        boundary_check_count: 0,
        sum_boundary_longitude_delta_deg: 0.0,
        sum_boundary_longitude_delta_deg_sq: 0.0,
        sum_boundary_latitude_delta_deg: 0.0,
        sum_boundary_latitude_delta_deg_sq: 0.0,
        sum_boundary_distance_delta_au: None,
        sum_boundary_distance_delta_au_sq: None,
        boundary_distance_check_count: 0,
        max_boundary_longitude_delta_body: None,
        max_boundary_longitude_delta_deg: 0.0,
        max_boundary_latitude_delta_body: None,
        max_boundary_latitude_delta_deg: 0.0,
        max_boundary_distance_delta_body: None,
        max_boundary_distance_delta_au: None,
    };

    for body in &report.bodies {
        summary.boundary_check_count += body.boundary_checks;
        summary.sum_boundary_longitude_delta_deg += body.sum_boundary_longitude_delta_deg;
        summary.sum_boundary_longitude_delta_deg_sq += body.sum_boundary_longitude_delta_deg_sq;
        summary.sum_boundary_latitude_delta_deg += body.sum_boundary_latitude_delta_deg;
        summary.sum_boundary_latitude_delta_deg_sq += body.sum_boundary_latitude_delta_deg_sq;
        if let Some(sum) = body.sum_boundary_distance_delta_au {
            summary.sum_boundary_distance_delta_au =
                Some(summary.sum_boundary_distance_delta_au.unwrap_or(0.0) + sum);
            summary.sum_boundary_distance_delta_au_sq = Some(
                summary.sum_boundary_distance_delta_au_sq.unwrap_or(0.0)
                    + body.sum_boundary_distance_delta_au_sq.unwrap_or(0.0),
            );
            summary.boundary_distance_check_count += body.boundary_distance_checks;
        }
        if body.boundary_checks == 0 {
            continue;
        }

        if body.max_boundary_longitude_delta_deg >= summary.max_boundary_longitude_delta_deg {
            summary.max_boundary_longitude_delta_deg = body.max_boundary_longitude_delta_deg;
            summary.max_boundary_longitude_delta_body = Some(body.body.clone());
        }
        if body.max_boundary_latitude_delta_deg >= summary.max_boundary_latitude_delta_deg {
            summary.max_boundary_latitude_delta_deg = body.max_boundary_latitude_delta_deg;
            summary.max_boundary_latitude_delta_body = Some(body.body.clone());
        }
        match (
            summary.max_boundary_distance_delta_au,
            body.max_boundary_distance_delta_au,
        ) {
            (Some(current), Some(next)) if next < current => {}
            (_, Some(next)) => {
                summary.max_boundary_distance_delta_au = Some(next);
                summary.max_boundary_distance_delta_body = Some(body.body.clone());
            }
            _ => {}
        }
    }

    summary
}

fn boundary_delta(left: &EclipticCoordinates, right: &EclipticCoordinates) -> BoundaryDelta {
    let longitude_delta_deg =
        Angle::from_degrees(left.longitude.degrees() - right.longitude.degrees())
            .normalized_signed()
            .degrees()
            .abs();
    let latitude_delta_deg = (left.latitude.degrees() - right.latitude.degrees()).abs();
    let distance_delta_au = match (left.distance_au, right.distance_au) {
        (Some(left), Some(right)) => Some((left - right).abs()),
        _ => None,
    };

    BoundaryDelta {
        longitude_delta_deg,
        latitude_delta_deg,
        distance_delta_au,
    }
}
