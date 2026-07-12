//! Human-readable rendering of the artifact inspection report.
//!
//! Moved verbatim from the parent `artifact` module: the compact summary-text
//! builder, the per-body-class error/cadence envelopes, the `Display`
//! implementation for `ArtifactInspectionReport`, and the small formatting
//! helpers they share.

use core::fmt;

use crate::artifact::*;
use crate::posture::data::coverage::profile::{
    packaged_artifact_generation_manifest_for_report,
    packaged_artifact_output_support_summary_for_report,
    packaged_artifact_production_profile_summary_for_report,
};
use crate::posture::data::coverage::regen::packaged_artifact_regeneration_summary_for_report;
use crate::posture::data::coverage::target::{
    packaged_artifact_phase2_corpus_alignment_summary_for_report,
    packaged_artifact_source_fit_holdout_sync_summary_for_report,
};
use crate::posture::data::lookup::{
    packaged_artifact_storage_summary_for_report, packaged_frame_treatment_summary_for_report,
    packaged_request_policy_summary_for_report,
};
use crate::posture::data::regenerate::packaged_artifact_body_class_span_cap_summary_for_report;
use crate::ComparisonSample;
use pleiades_compression::join_display;
use pleiades_core::{BackendFamily, CelestialBody, CelestialBodyClass};
use pleiades_data::packaged_artifact_profile_summary_with_body_coverage;

pub(crate) fn format_residual_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        join_display(bodies)
    }
}

pub(crate) fn format_body_class_coverage(report: &ArtifactInspectionReport) -> String {
    let mut luminaries = 0usize;
    let mut major_planets = 0usize;
    let mut lunar_points = 0usize;
    let mut built_in_asteroids = 0usize;
    let mut custom_bodies = 0usize;
    let mut other_bodies = 0usize;

    for body in &report.bodies {
        match body.body.class() {
            CelestialBodyClass::Luminary => luminaries += 1,
            CelestialBodyClass::MajorPlanet => major_planets += 1,
            CelestialBodyClass::LunarPoint => lunar_points += 1,
            CelestialBodyClass::BuiltInAsteroid => built_in_asteroids += 1,
            CelestialBodyClass::Custom => custom_bodies += 1,
            _ => other_bodies += 1,
        }
    }

    format!(
        "luminaries={luminaries}; major planets={major_planets}; lunar points={lunar_points}; built-in asteroids={built_in_asteroids}; custom bodies={custom_bodies}; other bodies={other_bodies}"
    )
}

pub(crate) fn render_artifact_summary_text(report: &ArtifactInspectionReport) -> String {
    let mut text = String::new();

    text.push_str("Artifact summary\n");
    text.push_str("  label: ");
    text.push_str(&report.generation_label);
    text.push('\n');
    text.push_str("  source: ");
    text.push_str(&report.source);
    text.push('\n');
    text.push_str("  regeneration provenance: ");
    text.push_str(&packaged_artifact_regeneration_summary_for_report());
    text.push('\n');
    text.push_str("  version: ");
    text.push_str(&report.version.to_string());
    text.push('\n');
    text.push_str("  byte order: ");
    text.push_str(report.endian_policy.label());
    text.push('\n');
    text.push_str(&format!("  checksum: 0x{:016x}\n", report.checksum));
    text.push_str("  encoded bytes: ");
    text.push_str(&report.encoded_bytes.to_string());
    text.push('\n');
    text.push_str("  Artifact profile: ");
    text.push_str(&packaged_artifact_profile_summary_with_body_coverage());
    text.push('\n');
    text.push_str("  Artifact output support: ");
    text.push_str(&packaged_artifact_output_support_summary_for_report());
    text.push('\n');
    text.push_str("  Body classes: ");
    text.push_str(&format_body_class_coverage(report));
    text.push('\n');
    text.push_str("  Body-class cadence: ");
    text.push_str(&format_body_class_cadence(report));
    text.push('\n');
    text.push_str("  Body-class span caps: ");
    text.push_str(&format_body_class_span_caps());
    text.push('\n');
    text.push_str("  Production profile skeleton: ");
    text.push_str(&packaged_artifact_production_profile_summary_for_report());
    text.push('\n');
    text.push_str("  Generation manifest: ");
    text.push_str(&packaged_artifact_generation_manifest_for_report());
    text.push('\n');
    text.push_str("  Packaged-artifact phase-2 corpus alignment: ");
    text.push_str(&packaged_artifact_phase2_corpus_alignment_summary_for_report());
    text.push('\n');
    text.push_str("  Packaged-artifact source-fit and hold-out sync: ");
    text.push_str(&packaged_artifact_source_fit_holdout_sync_summary_for_report());
    text.push('\n');
    text.push_str("  Artifact request policy: ");
    text.push_str(&packaged_request_policy_summary_for_report());
    text.push('\n');
    text.push_str("  Artifact storage: ");
    text.push_str(&packaged_artifact_storage_summary_for_report());
    text.push('\n');
    text.push_str("  Packaged frame treatment: ");
    text.push_str(&packaged_frame_treatment_summary_for_report());
    text.push('\n');
    text.push_str("  coverage: ");
    text.push_str(&report.earliest.julian_day.to_string());
    text.push_str(" → ");
    text.push_str(&report.latest.julian_day.to_string());
    text.push('\n');
    text.push_str("  bodies: ");
    text.push_str(&report.body_count.to_string());
    text.push_str(" total\n");
    text.push_str("  segments: ");
    text.push_str(&report.segment_count.to_string());
    text.push_str(" total\n");
    text.push_str("  residual-bearing segments: ");
    text.push_str(&report.residual_segment_count.to_string());
    text.push('\n');
    text.push_str("  residual-bearing bodies: ");
    text.push_str(&format_residual_bodies(&report.residual_bodies));
    text.push('\n');
    text.push_str("  ");
    text.push_str(
        &artifact_boundary_envelope_summary(report)
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact boundary envelope: unavailable ({error})")),
    );
    text.push('\n');
    text.push_str("  roundtrip decode: ");
    text.push_str(yes_no(report.roundtrip_ok));
    text.push('\n');
    text.push_str("  checksum verified: ");
    text.push_str(yes_no(report.checksum_ok));
    text.push('\n');
    if report
        .bodies
        .iter()
        .any(|body| matches!(body.body, CelestialBody::Custom(_)))
    {
        text.push_str(
            "  note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.\n",
        );
    }
    text.push('\n');
    text.push_str("Model error envelope\n");
    text.push_str("  baseline backend: ");
    text.push_str(&report.model_comparison.reference_backend.id.to_string());
    text.push('\n');
    text.push_str("  candidate backend: ");
    text.push_str(&report.model_comparison.candidate_backend.id.to_string());
    text.push('\n');
    text.push_str("  corpus: ");
    text.push_str(&report.model_comparison.corpus_name);
    text.push('\n');
    text.push_str("  samples: ");
    text.push_str(&report.model_comparison.summary.sample_count.to_string());
    text.push('\n');
    text.push_str(&format!(
        "  max longitude delta: {:.12}°\n",
        report.model_comparison.summary.max_longitude_delta_deg
    ));
    text.push_str(&format!(
        "  mean longitude delta: {:.12}°\n",
        report.model_comparison.summary.mean_longitude_delta_deg
    ));
    let median = crate::comparison_median_envelope(&report.model_comparison.samples)
        .expect("median envelope should exist");
    let percentile = crate::comparison_percentile_envelope(&report.model_comparison.samples, 0.95);
    text.push_str(&format!(
        "  median longitude delta: {:.12}°\n",
        median.longitude_delta_deg
    ));
    text.push_str(&format!(
        "  95th percentile longitude delta: {:.12}°\n",
        percentile.longitude_delta_deg
    ));
    text.push_str(&format!(
        "  rms longitude delta: {:.12}°\n",
        report.model_comparison.summary.rms_longitude_delta_deg
    ));
    text.push_str(&format!(
        "  max latitude delta: {:.12}°\n",
        report.model_comparison.summary.max_latitude_delta_deg
    ));
    text.push_str(&format!(
        "  mean latitude delta: {:.12}°\n",
        report.model_comparison.summary.mean_latitude_delta_deg
    ));
    text.push_str(&format!(
        "  median latitude delta: {:.12}°\n",
        median.latitude_delta_deg
    ));
    text.push_str(&format!(
        "  95th percentile latitude delta: {:.12}°\n",
        percentile.latitude_delta_deg
    ));
    text.push_str(&format!(
        "  rms latitude delta: {:.12}°\n",
        report.model_comparison.summary.rms_latitude_delta_deg
    ));
    if let Some(value) = report.model_comparison.summary.max_distance_delta_au {
        text.push_str(&format!("  max distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = report.model_comparison.summary.mean_distance_delta_au {
        text.push_str(&format!("  mean distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = median.distance_delta_au {
        text.push_str(&format!("  median distance delta: {:.12} AU\n", value));
    }
    if let Some(value) = percentile.distance_delta_au {
        text.push_str(&format!(
            "  95th percentile distance delta: {:.12} AU\n",
            value
        ));
    }
    if let Some(value) = report.model_comparison.summary.rms_distance_delta_au {
        text.push_str(&format!("  rms distance delta: {:.12} AU\n", value));
    }
    text.push_str("\nExpected tolerance status\n");
    let tolerance_summaries = report.model_comparison.tolerance_summaries();
    if tolerance_summaries.is_empty() {
        text.push_str("  none\n");
    } else {
        for summary in &tolerance_summaries {
            text.push_str("  ");
            text.push_str(&summary.body.to_string());
            text.push_str(": backend family=");
            text.push_str(backend_family_label(&summary.tolerance.backend_family));
            text.push_str(", profile=");
            text.push_str(summary.tolerance.profile);
            text.push_str(", status=");
            text.push_str(if summary.within_tolerance {
                "within"
            } else {
                "exceeded"
            });
            text.push_str(", limit Δlon≤");
            text.push_str(&format!(
                "{:.6}°",
                summary.tolerance.max_longitude_delta_deg
            ));
            text.push_str(", margin Δlon=");
            text.push_str(&format!("{:+.12}°", summary.longitude_margin_deg));
            text.push_str(", limit Δlat≤");
            text.push_str(&format!("{:.6}°", summary.tolerance.max_latitude_delta_deg));
            text.push_str(", margin Δlat=");
            text.push_str(&format!("{:+.12}°", summary.latitude_margin_deg));
            text.push_str(", limit Δdist=");
            text.push_str(
                &summary
                    .tolerance
                    .max_distance_delta_au
                    .map(|value| format!("{value:.6} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
            );
            text.push_str(", margin Δdist=");
            text.push_str(
                &summary
                    .distance_margin_au
                    .map(|value| format!("{value:+.12} AU"))
                    .unwrap_or_else(|| "n/a".to_string()),
            );
            text.push('\n');
        }
    }
    let within_tolerance_body_count = tolerance_summaries
        .iter()
        .filter(|summary| summary.within_tolerance)
        .count();
    let outside_tolerance_body_count = tolerance_summaries.len() - within_tolerance_body_count;
    let regression_count = report.model_comparison.notable_regressions().len();
    text.push_str("\nComparison tolerance audit\n");
    text.push_str("  bodies checked: ");
    text.push_str(&tolerance_summaries.len().to_string());
    text.push('\n');
    text.push_str("  within tolerance bodies: ");
    text.push_str(&within_tolerance_body_count.to_string());
    text.push('\n');
    text.push_str("  outside tolerance bodies: ");
    text.push_str(&outside_tolerance_body_count.to_string());
    text.push('\n');
    text.push_str("  notable regressions: ");
    text.push_str(&regression_count.to_string());
    text.push('\n');
    text.push_str("\nArtifact lookup benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .lookup_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact lookup benchmark: unavailable ({error})")),
    );
    text.push('\n');
    text.push_str("\nArtifact batch lookup benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .batch_lookup_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| {
                format!("Artifact batch lookup benchmark: unavailable ({error})")
            }),
    );
    text.push('\n');
    text.push_str("\nArtifact decode benchmark\n");
    text.push_str("  ");
    text.push_str(
        &report
            .decode_benchmark
            .validated_summary_line()
            .unwrap_or_else(|error| format!("Artifact decode benchmark: unavailable ({error})")),
    );
    text.push('\n');

    text.push_str("\nArtifact fit outliers by channel\n");
    text.push_str("  ");
    text.push_str(
        &crate::posture::data::coverage::fit::packaged_artifact_fit_channel_outlier_summary_for_report(),
    );
    text.push('\n');

    text.push_str("\nRelease summary: release-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Workspace audit: workspace-audit / audit\n");
    text.push_str(
        "See validate-artifact for the full body-class envelopes and regression details.\nSee release-summary for the compact one-screen release overview.\n",
    );

    text
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyClass {
    Luminary,
    MajorPlanet,
    LunarPoint,
    Asteroid,
    Custom,
}

impl BodyClass {
    const ALL: [Self; 5] = [
        Self::Luminary,
        Self::MajorPlanet,
        Self::LunarPoint,
        Self::Asteroid,
        Self::Custom,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminaries",
            Self::MajorPlanet => "Major planets",
            Self::LunarPoint => "Lunar points",
            Self::Asteroid => "Asteroids",
            Self::Custom => "Custom bodies",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Luminary => 0,
            Self::MajorPlanet => 1,
            Self::LunarPoint => 2,
            Self::Asteroid => 3,
            Self::Custom => 4,
        }
    }
}

fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury
        | CelestialBody::Venus
        | CelestialBody::Mars
        | CelestialBody::Jupiter
        | CelestialBody::Saturn
        | CelestialBody::Uranus
        | CelestialBody::Neptune
        | CelestialBody::Pluto => BodyClass::MajorPlanet,
        CelestialBody::MeanNode
        | CelestialBody::TrueNode
        | CelestialBody::MeanApogee
        | CelestialBody::TrueApogee
        | CelestialBody::MeanPerigee
        | CelestialBody::TruePerigee => BodyClass::LunarPoint,
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => BodyClass::Asteroid,
        CelestialBody::Custom(_) => BodyClass::Custom,
        _ => BodyClass::Custom,
    }
}

#[derive(Clone, Debug)]
struct BodyClassSummary {
    class: BodyClass,
    sample_count: usize,
    max_longitude_delta_body: Option<CelestialBody>,
    max_longitude_delta_deg: f64,
    sum_longitude_delta_deg: f64,
    max_latitude_delta_body: Option<CelestialBody>,
    max_latitude_delta_deg: f64,
    sum_latitude_delta_deg: f64,
    max_distance_delta_body: Option<CelestialBody>,
    max_distance_delta_au: Option<f64>,
    sum_distance_delta_au: f64,
    distance_count: usize,
}

impl BodyClassSummary {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            sample_count: 0,
            max_longitude_delta_body: None,
            max_longitude_delta_deg: 0.0,
            sum_longitude_delta_deg: 0.0,
            max_latitude_delta_body: None,
            max_latitude_delta_deg: 0.0,
            sum_latitude_delta_deg: 0.0,
            max_distance_delta_body: None,
            max_distance_delta_au: None,
            sum_distance_delta_au: 0.0,
            distance_count: 0,
        }
    }

    fn update(&mut self, sample: &ComparisonSample) {
        self.sample_count += 1;
        self.sum_longitude_delta_deg += sample.longitude_delta_deg;
        if sample.longitude_delta_deg >= self.max_longitude_delta_deg {
            self.max_longitude_delta_deg = sample.longitude_delta_deg;
            self.max_longitude_delta_body = Some(sample.body.clone());
        }
        self.sum_latitude_delta_deg += sample.latitude_delta_deg;
        if sample.latitude_delta_deg >= self.max_latitude_delta_deg {
            self.max_latitude_delta_deg = sample.latitude_delta_deg;
            self.max_latitude_delta_body = Some(sample.body.clone());
        }

        if let Some(distance_delta_au) = sample.distance_delta_au {
            match self.max_distance_delta_au {
                Some(current) if distance_delta_au < current => {}
                _ => {
                    self.max_distance_delta_au = Some(distance_delta_au);
                    self.max_distance_delta_body = Some(sample.body.clone());
                }
            }
            self.sum_distance_delta_au += distance_delta_au;
            self.distance_count += 1;
        }
    }

    fn mean_longitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_longitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_latitude_delta_deg(&self) -> f64 {
        if self.sample_count == 0 {
            0.0
        } else {
            self.sum_latitude_delta_deg / self.sample_count as f64
        }
    }

    fn mean_distance_delta_au(&self) -> Option<f64> {
        if self.distance_count == 0 {
            None
        } else {
            Some(self.sum_distance_delta_au / self.distance_count as f64)
        }
    }

    fn render(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sample_count == 0 {
            return Ok(());
        }

        writeln!(f, "  {}", self.class.label())?;
        writeln!(f, "    samples: {}", self.sample_count)?;
        if let (Some(body), value) = (
            self.max_longitude_delta_body.as_ref(),
            self.max_longitude_delta_deg,
        ) {
            writeln!(f, "    max longitude delta: {:.12}° ({})", value, body)?;
        }
        writeln!(
            f,
            "    mean longitude delta: {:.12}°",
            self.mean_longitude_delta_deg()
        )?;
        if let (Some(body), value) = (
            self.max_latitude_delta_body.as_ref(),
            self.max_latitude_delta_deg,
        ) {
            writeln!(f, "    max latitude delta: {:.12}° ({})", value, body)?;
        }
        writeln!(
            f,
            "    mean latitude delta: {:.12}°",
            self.mean_latitude_delta_deg()
        )?;
        if let (Some(body), Some(value)) = (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_au,
        ) {
            writeln!(f, "    max distance delta: {:.12} AU ({})", value, body)?;
        }
        if let Some(value) = self.mean_distance_delta_au() {
            writeln!(f, "    mean distance delta: {:.12} AU", value)?;
        }

        Ok(())
    }
}

fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;

    let mut summaries = BodyClass::ALL.map(BodyClassSummary::new);
    for sample in samples {
        summaries[body_class(&sample.body).index()].update(sample);
    }

    let mut has_entries = false;
    for summary in &summaries {
        if summary.sample_count > 0 {
            has_entries = true;
            summary.render(f)?;
        }
    }

    if !has_entries {
        writeln!(f, "  none")?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct BodyClassCadenceAccumulator {
    class: BodyClass,
    body_count: usize,
    segment_count: usize,
    min_segment_span_days: f64,
    max_segment_span_days: f64,
    sum_segment_span_days: f64,
}

impl BodyClassCadenceAccumulator {
    const fn new(class: BodyClass) -> Self {
        Self {
            class,
            body_count: 0,
            segment_count: 0,
            min_segment_span_days: f64::INFINITY,
            max_segment_span_days: 0.0,
            sum_segment_span_days: 0.0,
        }
    }

    fn push(&mut self, body: &ArtifactBodyInspection) {
        self.body_count += 1;
        self.segment_count += body.segment_count;
        if body.segment_count == 0 {
            return;
        }

        self.min_segment_span_days = self.min_segment_span_days.min(body.min_segment_span_days);
        self.max_segment_span_days = self.max_segment_span_days.max(body.max_segment_span_days);
        self.sum_segment_span_days += body.mean_segment_span_days * body.segment_count as f64;
    }

    fn finish(self) -> Option<BodyClassCadenceSummary> {
        if self.body_count == 0 {
            return None;
        }

        Some(BodyClassCadenceSummary {
            class: self.class,
            body_count: self.body_count,
            segment_count: self.segment_count,
            min_segment_span_days: if self.min_segment_span_days.is_infinite() {
                0.0
            } else {
                self.min_segment_span_days
            },
            max_segment_span_days: self.max_segment_span_days,
            mean_segment_span_days: if self.segment_count == 0 {
                0.0
            } else {
                self.sum_segment_span_days / self.segment_count as f64
            },
        })
    }
}

#[derive(Clone, Debug)]
struct BodyClassCadenceSummary {
    class: BodyClass,
    body_count: usize,
    segment_count: usize,
    min_segment_span_days: f64,
    max_segment_span_days: f64,
    mean_segment_span_days: f64,
}

impl BodyClassCadenceSummary {
    fn summary_line(&self) -> String {
        format!(
            "{}: {} bodies, {} segments, span days={:.12}..{:.12} (mean {:.12})",
            self.class.label(),
            self.body_count,
            self.segment_count,
            self.min_segment_span_days,
            self.max_segment_span_days,
            self.mean_segment_span_days,
        )
    }
}

fn body_class_cadence_summaries(report: &ArtifactInspectionReport) -> Vec<BodyClassCadenceSummary> {
    let mut accumulators = BodyClass::ALL.map(BodyClassCadenceAccumulator::new);
    for body in &report.bodies {
        accumulators[body_class(&body.body).index()].push(body);
    }

    accumulators
        .into_iter()
        .filter_map(BodyClassCadenceAccumulator::finish)
        .collect()
}

pub(crate) fn format_body_class_cadence(report: &ArtifactInspectionReport) -> String {
    let summaries = body_class_cadence_summaries(report);
    if summaries.is_empty() {
        "none".to_string()
    } else {
        summaries
            .into_iter()
            .map(|summary| summary.summary_line())
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub(crate) fn format_body_class_span_caps() -> String {
    let summary = packaged_artifact_body_class_span_cap_summary_for_report();
    summary
        .strip_prefix("body-class span caps: ")
        .unwrap_or(&summary)
        .to_string()
}

impl fmt::Display for ArtifactInspectionReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Artifact validation report")?;
        writeln!(f, "  label: {}", self.generation_label)?;
        writeln!(f, "  source: {}", self.source)?;
        writeln!(f, "  version: {}", self.version)?;
        writeln!(f, "  byte order: {}", self.endian_policy.label())?;
        writeln!(f, "  checksum: 0x{:016x}", self.checksum)?;
        writeln!(f, "  encoded bytes: {}", self.encoded_bytes)?;
        writeln!(f, "  roundtrip decode: {}", yes_no(self.roundtrip_ok))?;
        writeln!(f, "  checksum verified: {}", yes_no(self.checksum_ok))?;
        writeln!(f, "  bodies: {}", self.body_count)?;
        writeln!(f, "  segments: {}", self.segment_count)?;
        writeln!(
            f,
            "  residual-bearing segments: {}",
            self.residual_segment_count
        )?;
        writeln!(
            f,
            "  residual-bearing bodies: {}",
            format_residual_bodies(&self.residual_bodies)
        )?;
        writeln!(
            f,
            "  coverage: {} → {}",
            self.earliest.julian_day, self.latest.julian_day
        )?;
        writeln!(f, "  {}", artifact_boundary_envelope_summary(self))?;
        writeln!(
            f,
            "  Artifact request policy: {}",
            packaged_request_policy_summary_for_report()
        )?;
        writeln!(
            f,
            "  Artifact output support: {}",
            packaged_artifact_output_support_summary_for_report()
        )?;
        writeln!(f)?;
        writeln!(f, "Bodies")?;
        for body in &self.bodies {
            writeln!(f, "  {}", body)?;
        }

        writeln!(f)?;
        if self
            .bodies
            .iter()
            .any(|body| matches!(body.body, CelestialBody::Custom(_)))
        {
            writeln!(f, "Note: custom bodies are included in decode and boundary checks, but omitted from the algorithmic comparison corpus.")?;
            writeln!(f)?;
        }
        writeln!(f, "Model error envelope")?;
        writeln!(
            f,
            "  baseline backend: {}",
            self.model_comparison.reference_backend.id
        )?;
        writeln!(
            f,
            "  candidate backend: {}",
            self.model_comparison.candidate_backend.id
        )?;
        writeln!(f, "  corpus: {}", self.model_comparison.corpus_name)?;
        writeln!(
            f,
            "  samples: {}",
            self.model_comparison.summary.sample_count
        )?;
        writeln!(
            f,
            "  max longitude delta: {:.12}°",
            self.model_comparison.summary.max_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean longitude delta: {:.12}°",
            self.model_comparison.summary.mean_longitude_delta_deg
        )?;
        writeln!(
            f,
            "  max latitude delta: {:.12}°",
            self.model_comparison.summary.max_latitude_delta_deg
        )?;
        writeln!(
            f,
            "  mean latitude delta: {:.12}°",
            self.model_comparison.summary.mean_latitude_delta_deg
        )?;
        if let Some(value) = self.model_comparison.summary.max_distance_delta_au {
            writeln!(f, "  max distance delta: {:.12} AU", value)?;
        }
        if let Some(value) = self.model_comparison.summary.mean_distance_delta_au {
            writeln!(f, "  mean distance delta: {:.12} AU", value)?;
        }

        writeln!(f)?;
        write_body_class_envelopes(f, &self.model_comparison.samples)?;
        writeln!(f, "Body-class cadence")?;
        writeln!(f, "  {}", format_body_class_cadence(self))?;
        writeln!(f, "Body-class span caps")?;
        writeln!(f, "  {}", format_body_class_span_caps())?;
        writeln!(f)?;

        let notable_regressions = self.model_comparison.notable_regressions();
        writeln!(f, "  notable regressions")?;
        if notable_regressions.is_empty() {
            writeln!(f, "    none")?;
        } else {
            for finding in notable_regressions {
                writeln!(f, "    {}", finding.summary_line())?;
            }
        }

        writeln!(f)?;
        writeln!(f, "Artifact lookup benchmark")?;
        writeln!(f, "  {}", self.lookup_benchmark.summary_line())?;
        writeln!(f, "  artifact: {}", self.lookup_benchmark.artifact_label)?;
        writeln!(f, "  source: {}", self.lookup_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.lookup_benchmark.rounds)?;
        writeln!(
            f,
            "  lookups per round: {}",
            self.lookup_benchmark.sample_count
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.lookup_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            crate::format_duration(self.lookup_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per lookup: {:.2}",
            self.lookup_benchmark.nanoseconds_per_lookup()
        )?;
        writeln!(
            f,
            "  lookups per second: {:.2}",
            self.lookup_benchmark.lookups_per_second()
        )?;

        writeln!(f)?;
        writeln!(f, "Artifact batch lookup benchmark")?;
        writeln!(f, "  {}", self.batch_lookup_benchmark.summary_line())?;
        writeln!(
            f,
            "  artifact: {}",
            self.batch_lookup_benchmark.artifact_label
        )?;
        writeln!(f, "  source: {}", self.batch_lookup_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.batch_lookup_benchmark.rounds)?;
        writeln!(
            f,
            "  lookups per batch: {}",
            self.batch_lookup_benchmark.batch_size
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.batch_lookup_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            crate::format_duration(self.batch_lookup_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per lookup: {:.2}",
            self.batch_lookup_benchmark.nanoseconds_per_lookup()
        )?;
        writeln!(
            f,
            "  lookups per second: {:.2}",
            self.batch_lookup_benchmark.lookups_per_second()
        )?;

        writeln!(f)?;
        writeln!(f, "Artifact decode benchmark")?;
        writeln!(f, "  {}", self.decode_benchmark.summary_line())?;
        writeln!(f, "  artifact: {}", self.decode_benchmark.artifact_label)?;
        writeln!(f, "  source: {}", self.decode_benchmark.source)?;
        writeln!(f, "  rounds: {}", self.decode_benchmark.rounds)?;
        writeln!(
            f,
            "  decodes per round: {}",
            self.decode_benchmark.sample_count
        )?;
        writeln!(
            f,
            "  encoded bytes: {}",
            self.decode_benchmark.encoded_bytes
        )?;
        writeln!(
            f,
            "  elapsed: {}",
            crate::format_duration(self.decode_benchmark.elapsed)
        )?;
        writeln!(
            f,
            "  nanoseconds per decode: {:.2}",
            self.decode_benchmark.nanoseconds_per_decode()
        )?;
        writeln!(
            f,
            "  decodes per second: {:.2}",
            self.decode_benchmark.decodes_per_second()
        )?;

        Ok(())
    }
}

pub(crate) fn yes_no(value: bool) -> &'static str {
    if value {
        "ok"
    } else {
        "failed"
    }
}

fn backend_family_label(family: &BackendFamily) -> &'static str {
    match family {
        BackendFamily::Algorithmic => "algorithmic",
        BackendFamily::ReferenceData => "reference data",
        BackendFamily::CompressedData => "compressed data",
        BackendFamily::Composite => "composite",
        BackendFamily::Other(_) => "other",
        _ => "other",
    }
}
