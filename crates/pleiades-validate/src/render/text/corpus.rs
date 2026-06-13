//! Comparison and source corpus summary text and validation.

use std::fmt;
use std::sync::OnceLock;

use crate::*;

pub(crate) fn comparison_corpus_release_guard_summary() -> &'static str {
    "Pluto excluded from tolerance evidence"
}

pub(crate) fn validated_comparison_corpus_release_guard_summary_for_report(
) -> Result<&'static str, String> {
    const EXPECTED: &str = "Pluto excluded from tolerance evidence";
    let summary = comparison_corpus_release_guard_summary();

    if summary == EXPECTED {
        Ok(summary)
    } else {
        Err(format!(
            "comparison corpus release-grade guard mismatch: expected {EXPECTED}, found {summary}"
        ))
    }
}

pub(crate) fn render_comparison_corpus_summary_text() -> String {
    use std::fmt::Write as _;

    let corpus = release_grade_corpus();
    let summary = corpus.summary();
    let mut text = String::from("Comparison corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => return format!("Comparison corpus summary unavailable ({error})"),
    };
    let _ = writeln!(text, "  release-grade guard: {release_grade_guard}");
    text.push('\n');
    text
}

pub(crate) fn ensure_comparison_corpus_summary_matches_current_rendering(
    comparison_corpus_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_summary_text != render_comparison_corpus_summary_text() {
        return Err(ReleaseBundleError::Verification(
            "comparison corpus summary no longer matches the current comparison-corpus posture"
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn required_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    summary
        .strip_prefix(prefix)
        .map(str::to_string)
        .ok_or_else(|| {
            format!("source corpus summary field `{field}` is out of sync with the current posture")
        })
}

pub(crate) fn required_labelled_summary_payload(
    summary: String,
    prefix: &str,
    field: &'static str,
) -> Result<String, String> {
    let payload = required_summary_payload(summary, prefix, field)?;
    if payload.starts_with(prefix) {
        return Err(format!(
            "source corpus summary field `{field}` is out of sync with the current posture"
        ));
    }

    Ok(payload)
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SourceCorpusSummary {
    pub(crate) comparison_corpus_release_grade_guard: String,
    pub(crate) jpl_source_corpus_contract: String,
    pub(crate) jpl_evidence_classification: String,
    pub(crate) jpl_provenance_only: String,
    pub(crate) lunar_source_window: String,
    pub(crate) shared_schema: String,
    pub(crate) generation_command: String,
    pub(crate) production_generation_source: String,
    pub(crate) production_generation_source_revision: String,
    pub(crate) production_generation_coverage: String,
    pub(crate) production_generation_source_windows: String,
    pub(crate) production_generation_body_class_coverage: String,
    pub(crate) production_generation_date_range: String,
    pub(crate) production_generation_quarter_day_boundary_samples: String,
    pub(crate) coverage_posture: String,
    pub(crate) production_generation_boundary_window: String,
    pub(crate) production_generation_boundary_source: String,
    pub(crate) production_generation_boundary_request_corpus: String,
    pub(crate) production_generation_boundary_request_corpus_equatorial: String,
    pub(crate) reference_snapshot_sparse_boundary: String,
    pub(crate) reference_snapshot_exact_j2000_evidence: String,
    pub(crate) reference_snapshot_exact_j2000_body_class_coverage: String,
    pub(crate) reference_snapshot_equatorial_parity: String,
    pub(crate) reference_snapshot_body_class_coverage: String,
    pub(crate) reference_snapshot_manifest: String,
    pub(crate) comparison_snapshot_manifest: String,
    pub(crate) independent_holdout_body_class_coverage: String,
    pub(crate) independent_holdout_source_window: String,
    pub(crate) pluto_fallback: String,
    pub(crate) release_grade_body_claims: String,
    pub(crate) body_date_channel_claims: String,
    pub(crate) phase2_corpus_alignment: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SourceCorpusSummaryValidationError {
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for SourceCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the source corpus summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for SourceCorpusSummaryValidationError {}

impl SourceCorpusSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "comparison corpus release-grade guard: {}; JPL source corpus contract: {}; evidence classification={}; provenance-only={}; lunar source windows={}; shared schema={}; generation command={}; production generation source={}; production generation source revision={}; production generation coverage={}; production generation source windows={}; production generation body-class coverage={}; production generation date range={}; production generation quarter-day boundary samples={}; coverage posture={}; production generation boundary window={}; production generation boundary source={}; production generation boundary request corpus={}; production generation boundary request corpus equatorial={}; reference snapshot sparse boundary={}; reference snapshot exact J2000 evidence={}; reference snapshot exact J2000 body-class coverage={}; reference snapshot equatorial parity={}; reference snapshot body-class coverage={}; reference snapshot manifest={}; comparison snapshot manifest={}; independent-holdout body-class coverage={}; independent-holdout source window={}; pluto fallback={}; release-grade body claims={}; body-date-channel claims={}; phase-2 corpus alignment: {}",
            self.comparison_corpus_release_grade_guard,
            self.jpl_source_corpus_contract,
            self.jpl_evidence_classification,
            self.jpl_provenance_only,
            self.lunar_source_window,
            self.shared_schema,
            self.generation_command,
            self.production_generation_source,
            self.production_generation_source_revision,
            self.production_generation_coverage,
            self.production_generation_source_windows,
            self.production_generation_body_class_coverage,
            self.production_generation_date_range,
            self.production_generation_quarter_day_boundary_samples,
            self.coverage_posture,
            self.production_generation_boundary_window,
            self.production_generation_boundary_source,
            self.production_generation_boundary_request_corpus,
            self.production_generation_boundary_request_corpus_equatorial,
            self.reference_snapshot_sparse_boundary,
            self.reference_snapshot_exact_j2000_evidence,
            self.reference_snapshot_exact_j2000_body_class_coverage,
            self.reference_snapshot_equatorial_parity,
            self.reference_snapshot_body_class_coverage,
            self.reference_snapshot_manifest,
            self.comparison_snapshot_manifest,
            self.independent_holdout_body_class_coverage,
            self.independent_holdout_source_window,
            self.pluto_fallback,
            self.release_grade_body_claims,
            self.body_date_channel_claims,
            self.phase2_corpus_alignment,
        )
    }

    pub(crate) fn validate(&self) -> Result<(), SourceCorpusSummaryValidationError> {
        let expected = source_corpus_summary_details().ok_or(
            SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "source_corpus_summary",
            },
        )?;

        if self.comparison_corpus_release_grade_guard
            != expected.comparison_corpus_release_grade_guard
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "comparison_corpus_release_grade_guard",
            });
        }
        if self.jpl_source_corpus_contract != expected.jpl_source_corpus_contract {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_source_corpus_contract",
            });
        }
        if self.jpl_evidence_classification != expected.jpl_evidence_classification {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_evidence_classification",
            });
        }
        if self.jpl_provenance_only != expected.jpl_provenance_only {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "jpl_provenance_only",
            });
        }
        if self.lunar_source_window != expected.lunar_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "lunar_source_window",
            });
        }
        if self.shared_schema != expected.shared_schema {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "shared_schema",
            });
        }
        if self.generation_command != expected.generation_command {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "generation_command",
            });
        }
        if self.production_generation_source != expected.production_generation_source {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source",
            });
        }
        if self.production_generation_source_revision
            != expected.production_generation_source_revision
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_revision",
            });
        }
        if self.production_generation_coverage != expected.production_generation_coverage {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_coverage",
            });
        }
        if self.production_generation_source_windows
            != expected.production_generation_source_windows
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_source_windows",
            });
        }
        if self.production_generation_body_class_coverage
            != expected.production_generation_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_body_class_coverage",
            });
        }
        if self.production_generation_date_range != expected.production_generation_date_range {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_date_range",
            });
        }
        if self.production_generation_quarter_day_boundary_samples
            != expected.production_generation_quarter_day_boundary_samples
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_quarter_day_boundary_samples",
            });
        }
        if self.coverage_posture != expected.coverage_posture {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "coverage_posture",
            });
        }
        if self.production_generation_boundary_window
            != expected.production_generation_boundary_window
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_window",
            });
        }
        if self.production_generation_boundary_source
            != expected.production_generation_boundary_source
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_source",
            });
        }
        if self.production_generation_boundary_request_corpus
            != expected.production_generation_boundary_request_corpus
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus",
            });
        }
        if self.production_generation_boundary_request_corpus_equatorial
            != expected.production_generation_boundary_request_corpus_equatorial
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "production_generation_boundary_request_corpus_equatorial",
            });
        }
        if self.reference_snapshot_sparse_boundary != expected.reference_snapshot_sparse_boundary {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_sparse_boundary",
            });
        }
        if self.reference_snapshot_exact_j2000_evidence
            != expected.reference_snapshot_exact_j2000_evidence
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_evidence",
            });
        }
        if self.reference_snapshot_exact_j2000_body_class_coverage
            != expected.reference_snapshot_exact_j2000_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_exact_j2000_body_class_coverage",
            });
        }
        if self.reference_snapshot_equatorial_parity
            != expected.reference_snapshot_equatorial_parity
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_equatorial_parity",
            });
        }
        if self.reference_snapshot_body_class_coverage
            != expected.reference_snapshot_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_body_class_coverage",
            });
        }
        if self.reference_snapshot_manifest != expected.reference_snapshot_manifest {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "reference_snapshot_manifest",
            });
        }
        if self.independent_holdout_body_class_coverage
            != expected.independent_holdout_body_class_coverage
        {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_body_class_coverage",
            });
        }
        if self.independent_holdout_source_window != expected.independent_holdout_source_window {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "independent_holdout_source_window",
            });
        }
        if self.pluto_fallback != expected.pluto_fallback {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "pluto_fallback",
            });
        }
        if self.release_grade_body_claims != expected.release_grade_body_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "release_grade_body_claims",
            });
        }
        if self.body_date_channel_claims != expected.body_date_channel_claims {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "body_date_channel_claims",
            });
        }
        if self.phase2_corpus_alignment != expected.phase2_corpus_alignment {
            return Err(SourceCorpusSummaryValidationError::FieldOutOfSync {
                field: "phase2_corpus_alignment",
            });
        }

        Ok(())
    }

    pub(crate) fn validated_summary_line(
        &self,
    ) -> Result<String, SourceCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn source_corpus_summary_details() -> Option<SourceCorpusSummary> {
    let comparison_corpus_release_grade_guard =
        validated_comparison_corpus_release_guard_summary_for_report()
            .ok()?
            .to_string();
    let jpl_source_corpus_contract = required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    )
    .ok()?;
    let jpl_evidence_classification = required_labelled_summary_payload(
        jpl_snapshot_evidence_classification_summary_for_report(),
        "JPL evidence classification: ",
        "JPL evidence classification",
    )
    .ok()?;
    let jpl_provenance_only = required_labelled_summary_payload(
        jpl_provenance_only_summary_for_report(),
        "JPL provenance-only evidence: ",
        "JPL provenance-only evidence",
    )
    .ok()?;
    let release_grade_body_claims = validated_release_body_claims_summary_line_for_report()
        .ok()?
        .to_string();
    let lunar_source_window = required_summary_payload(
        lunar_source_window_summary_for_report(),
        "lunar source windows: ",
        "lunar source window",
    )
    .ok()?;
    let reference_snapshot_sparse_boundary = required_summary_payload(
        reference_snapshot_sparse_boundary_summary_for_report(),
        "Reference snapshot boundary day: ",
        "reference snapshot sparse boundary",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_evidence = required_summary_payload(
        reference_snapshot_exact_j2000_evidence_summary_for_report(),
        "Reference snapshot exact J2000 evidence: ",
        "reference snapshot exact J2000 evidence",
    )
    .ok()?;
    let reference_snapshot_exact_j2000_body_class_coverage = required_summary_payload(
        pleiades_jpl::reference_snapshot_exact_j2000_body_class_coverage_summary_for_report(),
        "Reference snapshot exact J2000 body-class coverage: ",
        "reference snapshot exact J2000 body-class coverage",
    )
    .ok()?;
    let reference_snapshot_equatorial_parity = required_summary_payload(
        reference_snapshot_equatorial_parity_summary_for_report(),
        "JPL reference snapshot equatorial parity: ",
        "reference snapshot equatorial parity",
    )
    .ok()?;
    let reference_snapshot_body_class_coverage = required_summary_payload(
        reference_snapshot_body_class_coverage_summary_for_report(),
        "Reference snapshot body-class coverage: ",
        "reference snapshot body-class coverage",
    )
    .ok()?;
    let reference_snapshot_manifest = required_summary_payload(
        reference_snapshot_manifest_summary_for_report(),
        "Reference snapshot manifest: ",
        "reference snapshot manifest",
    )
    .ok()?;
    let comparison_snapshot_manifest = required_summary_payload(
        validated_comparison_snapshot_manifest_summary_for_report().ok()?,
        "Comparison snapshot manifest: ",
        "comparison snapshot manifest",
    )
    .ok()?;
    let independent_holdout_body_class_coverage = required_summary_payload(
        independent_holdout_snapshot_body_class_coverage_summary_for_report(),
        "Independent hold-out body-class coverage: ",
        "independent-holdout body-class coverage",
    )
    .ok()?;
    let independent_holdout_source_window = required_summary_payload(
        independent_holdout_snapshot_source_window_summary_for_report(),
        "Independent hold-out source windows: ",
        "independent-holdout source window",
    )
    .ok()?;
    let phase2_corpus_alignment =
        validated_packaged_artifact_phase2_corpus_alignment_summary_for_report();
    let pluto_fallback = required_summary_payload(
        format!(
            "Pluto fallback: {}",
            validated_pluto_fallback_summary_line_for_report().ok()?
        ),
        "Pluto fallback: ",
        "pluto fallback",
    )
    .ok()?;
    let production_generation_date_range = production_generation_date_range_for_report()?;
    let production_generation_quarter_day_boundary_samples = required_summary_payload(
        pleiades_jpl::production_generation_quarter_day_boundary_summary_for_report(),
        "Production generation quarter-day boundary samples: ",
        "production generation quarter-day boundary samples",
    )
    .ok()?;

    Some(SourceCorpusSummary {
        comparison_corpus_release_grade_guard,
        jpl_source_corpus_contract,
        jpl_evidence_classification,
        jpl_provenance_only,
        lunar_source_window,
        shared_schema: validated_checked_in_snapshot_schema_summary_for_report().ok()?,
        generation_command: "generate-packaged-artifact --check".to_string(),
        production_generation_source: required_summary_payload(
            validated_production_generation_source_summary_for_report().ok()?,
            "Production generation source: ",
            "production generation source",
        )
        .ok()?,
        production_generation_source_revision:
            validated_production_generation_source_revision_summary_for_report().ok()?,
        production_generation_coverage: required_summary_payload(
            production_generation_snapshot_summary_for_report(),
            "Production generation coverage: ",
            "production generation coverage",
        )
        .ok()?,
        production_generation_source_windows: required_summary_payload(
            production_generation_snapshot_window_summary_for_report(),
            "Production generation source windows: ",
            "production generation source windows",
        )
        .ok()?,
        production_generation_body_class_coverage: required_summary_payload(
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary_for_report(),
            "Production generation body-class coverage: ",
            "production generation body-class coverage",
        )
        .ok()?,
        production_generation_date_range,
        production_generation_quarter_day_boundary_samples,
        coverage_posture: production_generation_coverage_posture_for_report()?,
        production_generation_boundary_window: required_summary_payload(
            production_generation_boundary_window_summary_for_report(),
            "Production generation boundary windows: ",
            "production generation boundary window",
        )
        .ok()?,
        production_generation_boundary_source: required_summary_payload(
            production_generation_boundary_source_summary_for_report(),
            "Production generation boundary overlay source: ",
            "production generation boundary source",
        )
        .ok()?,
        production_generation_boundary_request_corpus: required_summary_payload(
            production_generation_boundary_request_corpus_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus",
        )
        .ok()?,
        production_generation_boundary_request_corpus_equatorial: required_summary_payload(
            production_generation_boundary_request_corpus_equatorial_summary_for_report(),
            "Production generation boundary request corpus: ",
            "production generation boundary request corpus equatorial",
        )
        .ok()?,
        reference_snapshot_sparse_boundary,
        reference_snapshot_exact_j2000_evidence,
        reference_snapshot_exact_j2000_body_class_coverage,
        reference_snapshot_equatorial_parity,
        reference_snapshot_body_class_coverage,
        reference_snapshot_manifest,
        comparison_snapshot_manifest,
        independent_holdout_body_class_coverage,
        independent_holdout_source_window,
        pluto_fallback,
        release_grade_body_claims,
        body_date_channel_claims: body_date_channel_claims_summary_details()?
            .validated_summary_line()
            .ok()?,
        phase2_corpus_alignment,
    })
}

pub(crate) fn validated_source_corpus_summary_for_report() -> Result<String, String> {
    let summary =
        source_corpus_summary_details().ok_or_else(|| "source corpus unavailable".to_string())?;
    summary
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn source_corpus_summary_for_report() -> String {
    match validated_source_corpus_summary_for_report() {
        Ok(summary) => summary,
        Err(error) => format!("Source corpus unavailable ({error})"),
    }
}

pub(crate) fn source_corpus_posture_summary_for_report() -> String {
    source_corpus_summary_for_report()
}

pub(crate) fn render_comparison_corpus_release_guard_summary_text() -> String {
    let release_grade_guard = match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(guard) => guard,
        Err(error) => {
            return format!("Comparison corpus release-grade guard summary unavailable ({error})")
        }
    };
    format!(
        "Comparison corpus release-grade guard summary\nRelease-grade guard: {release_grade_guard}\n",
    )
}

pub(crate) fn ensure_comparison_corpus_release_guard_summary_matches_current_rendering(
    comparison_corpus_release_guard_summary_text: &str,
) -> Result<(), ReleaseBundleError> {
    if comparison_corpus_release_guard_summary_text
        != render_comparison_corpus_release_guard_summary_text()
    {
        return Err(ReleaseBundleError::Verification(
            "comparison-corpus release-guard summary no longer matches the current comparison-corpus release-guard posture"
                .to_string(),
        ));
    }

    Ok(())
}

pub(crate) fn validated_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let corpus = benchmark_corpus();
    let summary = corpus.summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

pub(crate) fn validated_chart_benchmark_corpus_summary_for_report() -> Result<String, String> {
    let summary = chart_benchmark_corpus_summary();
    summary.validate().map_err(|error| error.to_string())?;

    let mut text = String::from("Chart benchmark corpus summary\n");
    write_corpus_summary_text(&mut text, &summary);
    text.push('\n');
    Ok(text)
}

pub(crate) fn render_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(|| match validated_benchmark_corpus_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Benchmark corpus summary unavailable ({error})\n"),
        })
        .clone()
}

pub(crate) fn render_chart_benchmark_corpus_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();

    CACHE
        .get_or_init(
            || match validated_chart_benchmark_corpus_summary_for_report() {
                Ok(summary) => summary,
                Err(error) => format!("Chart benchmark corpus summary unavailable ({error})\n"),
            },
        )
        .clone()
}
