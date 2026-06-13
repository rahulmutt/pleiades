//! jpl posture summaries.

use core::fmt;

use pleiades_backend::{
    CelestialBody, EphemerisBackend, EphemerisErrorKind, EphemerisRequest, FrameTreatmentSummary,
};
use pleiades_types::{Apparentness, CoordinateFrame, Instant, TimeScale, ZodiacMode};

#[allow(unused_imports)]
use crate::reference_summary::*;
#[allow(unused_imports)]
use crate::*;

pub(crate) fn validated_checked_in_snapshot_schema_summary() -> Result<&'static str, String> {
    reference_snapshot_manifest_summary()
        .validate_with_expected_columns(&CHECKED_IN_SNAPSHOT_SCHEMA_COLUMNS)
        .map_err(|error| format!("reference snapshot schema validation failed: {error}"))?;
    independent_holdout_manifest_summary()
        .validate_with_expected_columns(&CHECKED_IN_SNAPSHOT_SCHEMA_COLUMNS)
        .map_err(|error| {
            format!("independent hold-out snapshot schema validation failed: {error}")
        })?;

    Ok("epoch_jd, body, x_km, y_km, z_km")
}

/// Returns the schema shared by the checked-in snapshot fixtures after validating the manifests.
pub fn checked_in_snapshot_schema_summary_for_report() -> String {
    match validated_checked_in_snapshot_schema_summary() {
        Ok(schema) => format!("Checked-in snapshot schema: {schema}"),
        Err(error) => format!("Checked-in snapshot schema: unavailable ({error})"),
    }
}

/// Returns the validated schema shared by the checked-in snapshot fixtures.
pub fn validated_checked_in_snapshot_schema_summary_for_report() -> Result<String, String> {
    validated_checked_in_snapshot_schema_summary().map(str::to_string)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplSnapshotEvidenceClassificationSummary {
    /// Evidence-classification line used by validation and release reports.
    pub text: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplSnapshotEvidenceClassificationSummaryValidationError {
    /// A summary field is out of sync with the current posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSnapshotEvidenceClassificationSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL snapshot evidence classification summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSnapshotEvidenceClassificationSummaryValidationError {}

impl JplSnapshotEvidenceClassificationSummary {
    /// Returns the evidence-classification line used by validation and release reports.
    pub fn summary_line(&self) -> String {
        self.text.to_string()
    }

    /// Returns `Ok(())` when the summary still matches the current posture.
    pub fn validate(&self) -> Result<(), JplSnapshotEvidenceClassificationSummaryValidationError> {
        if self.text != JPL_SNAPSHOT_EVIDENCE_CLASSIFICATION_SUMMARY {
            return Err(
                JplSnapshotEvidenceClassificationSummaryValidationError::FieldOutOfSync {
                    field: "text",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated evidence-classification line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSnapshotEvidenceClassificationSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplSnapshotEvidenceClassificationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the evidence-classification line used by validation and release reports.
pub fn jpl_snapshot_evidence_classification_summary_details(
) -> JplSnapshotEvidenceClassificationSummary {
    let summary = JplSnapshotEvidenceClassificationSummary {
        text: JPL_SNAPSHOT_EVIDENCE_CLASSIFICATION_SUMMARY,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the validated evidence-classification line used by validation and release reports.
pub fn jpl_snapshot_evidence_classification_summary_for_report() -> String {
    let summary = jpl_snapshot_evidence_classification_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL evidence classification: unavailable ({error})"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplSourcePostureSummary {
    /// Source-posture line used by validation and release reports.
    pub text: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplSourcePostureSummaryValidationError {
    /// A summary field is out of sync with the current posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSourcePostureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL source posture summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSourcePostureSummaryValidationError {}

impl JplSourcePostureSummary {
    /// Returns the source-posture line used by validation and release reports.
    pub fn summary_line(&self) -> String {
        self.text.to_string()
    }

    /// Returns `Ok(())` when the summary still matches the current posture.
    pub fn validate(&self) -> Result<(), JplSourcePostureSummaryValidationError> {
        if self.text != JPL_SOURCE_POSTURE_SUMMARY {
            return Err(JplSourcePostureSummaryValidationError::FieldOutOfSync { field: "text" });
        }

        Ok(())
    }

    /// Returns the validated source-posture line.
    pub fn validated_summary_line(&self) -> Result<String, JplSourcePostureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplSourcePostureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the source-posture line used by validation and release reports.
pub fn jpl_source_posture_summary_details() -> JplSourcePostureSummary {
    let summary = JplSourcePostureSummary {
        text: JPL_SOURCE_POSTURE_SUMMARY,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the validated source-posture line used by validation and release reports.
pub fn jpl_source_posture_summary_for_report() -> String {
    let summary = jpl_source_posture_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL source posture: unavailable ({error})"),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplProvenanceOnlySummary {
    /// Provenance-only line used by validation and release reports.
    pub text: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplProvenanceOnlySummaryValidationError {
    /// A summary field is out of sync with the current posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplProvenanceOnlySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL provenance-only summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplProvenanceOnlySummaryValidationError {}

impl JplProvenanceOnlySummary {
    /// Returns the provenance-only line used by validation and release reports.
    pub fn summary_line(&self) -> String {
        self.text.to_string()
    }

    /// Returns `Ok(())` when the summary still matches the current posture.
    pub fn validate(&self) -> Result<(), JplProvenanceOnlySummaryValidationError> {
        if self.text != JPL_PROVENANCE_ONLY_SUMMARY {
            return Err(JplProvenanceOnlySummaryValidationError::FieldOutOfSync { field: "text" });
        }

        Ok(())
    }

    /// Returns the validated provenance-only line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplProvenanceOnlySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplProvenanceOnlySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the provenance-only line used by validation and release reports.
pub fn jpl_provenance_only_summary_details() -> JplProvenanceOnlySummary {
    let summary = JplProvenanceOnlySummary {
        text: JPL_PROVENANCE_ONLY_SUMMARY,
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the validated provenance-only line used by validation and release reports.
pub fn jpl_provenance_only_summary_for_report() -> String {
    let summary = jpl_provenance_only_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL provenance-only evidence: unavailable ({error})"),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JplSourceCorpusContractSummary {
    /// Evidence-classification line for the current corpus contract.
    pub evidence_classification: JplSnapshotEvidenceClassificationSummary,
    /// Source-posture line for the current corpus contract.
    pub source_posture: JplSourcePostureSummary,
    /// Reference-snapshot provenance describing the release-claimed body/channel/frame posture.
    pub reference_summary: ReferenceSnapshotSourceSummary,
    /// Independent hold-out provenance describing the hold-out partition.
    pub boundary_summary: IndependentHoldoutSourceSummary,
    /// Source-window summary for the merged production-generation corpus.
    pub source_windows: ProductionGenerationSnapshotWindowSummary,
    /// Deterministic revision metadata for the checked-in CSV fixtures.
    pub source_revision: ProductionGenerationSourceRevisionSummary,
    /// Ecliptic boundary-request corpus used to keep request-frame posture explicit.
    pub boundary_request_corpus_ecliptic: ProductionGenerationBoundaryRequestCorpusSummary,
    /// Equatorial boundary-request corpus used to keep request-frame posture explicit.
    pub boundary_request_corpus_equatorial: ProductionGenerationBoundaryRequestCorpusSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplSourceCorpusContractSummaryValidationError {
    /// A field is out of sync with the current corpus contract posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSourceCorpusContractSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL source corpus contract summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSourceCorpusContractSummaryValidationError {}

impl JplSourceCorpusContractSummary {
    /// Returns the source-corpus contract line used by validation and release reports.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL source corpus contract: {}; {}; reference={}; hold-out={}; source windows={}; source revision={}; boundary request corpora: ecliptic={}; equatorial={}",
            self.evidence_classification.summary_line(),
            self.source_posture.summary_line(),
            self.reference_summary.summary_line(),
            self.boundary_summary.summary_line(),
            strip_report_prefix(
                &self.source_windows.summary_line(),
                "Production generation source windows: ",
            ),
            self.source_revision.summary_line(),
            strip_report_prefix(
                &self.boundary_request_corpus_ecliptic.summary_line(),
                "Production generation boundary request corpus: ",
            ),
            strip_report_prefix(
                &self.boundary_request_corpus_equatorial.summary_line(),
                "Production generation boundary request corpus: ",
            ),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current posture.
    pub fn validate(&self) -> Result<(), JplSourceCorpusContractSummaryValidationError> {
        if self.evidence_classification != jpl_snapshot_evidence_classification_summary_details() {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "evidence_classification",
                },
            );
        }
        if self.source_posture != jpl_source_posture_summary_details() {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "source_posture",
                },
            );
        }
        if self.reference_summary != reference_snapshot_source_summary() {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "reference_summary",
                },
            );
        }
        if self.boundary_summary != independent_holdout_source_summary() {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "boundary_summary",
                },
            );
        }
        let expected_source_windows = production_generation_snapshot_window_summary().ok_or(
            JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                field: "source_windows",
            },
        )?;
        if self.source_windows != expected_source_windows {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "source_windows",
                },
            );
        }
        if self.source_revision != production_generation_source_revision_summary() {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "source_revision",
                },
            );
        }
        let expected_boundary_request_corpus_ecliptic =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Ecliptic)
                .ok_or(
                    JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                        field: "boundary_request_corpus_ecliptic",
                    },
                )?;
        if self.boundary_request_corpus_ecliptic != expected_boundary_request_corpus_ecliptic {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "boundary_request_corpus_ecliptic",
                },
            );
        }
        let expected_boundary_request_corpus_equatorial =
            production_generation_boundary_request_corpus_summary(CoordinateFrame::Equatorial)
                .ok_or(
                    JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                        field: "boundary_request_corpus_equatorial",
                    },
                )?;
        if self.boundary_request_corpus_equatorial != expected_boundary_request_corpus_equatorial {
            return Err(
                JplSourceCorpusContractSummaryValidationError::FieldOutOfSync {
                    field: "boundary_request_corpus_equatorial",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated source-corpus contract line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSourceCorpusContractSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplSourceCorpusContractSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the source-corpus contract line used by validation and release reports.
pub fn jpl_source_corpus_contract_summary_details() -> JplSourceCorpusContractSummary {
    let summary = JplSourceCorpusContractSummary {
        evidence_classification: jpl_snapshot_evidence_classification_summary_details(),
        source_posture: jpl_source_posture_summary_details(),
        reference_summary: reference_snapshot_source_summary(),
        boundary_summary: independent_holdout_source_summary(),
        source_windows: production_generation_snapshot_window_summary()
            .expect("production generation source windows should exist"),
        source_revision: production_generation_source_revision_summary(),
        boundary_request_corpus_ecliptic: production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )
        .expect("production generation ecliptic boundary request corpus should exist"),
        boundary_request_corpus_equatorial: production_generation_boundary_request_corpus_summary(
            CoordinateFrame::Equatorial,
        )
        .expect("production generation equatorial boundary request corpus should exist"),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Returns the validated source-corpus contract line used by validation and release reports.
pub fn jpl_source_corpus_contract_summary_for_report() -> String {
    let summary = jpl_source_corpus_contract_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("JPL source corpus contract: unavailable ({error})"),
    }
}

/// Returns the combined snapshot evidence summary used by validation and release reports.
pub fn jpl_snapshot_evidence_summary_for_report() -> String {
    [
        jpl_snapshot_evidence_classification_summary_for_report(),
        jpl_source_posture_summary_for_report(),
        jpl_provenance_only_summary_for_report(),
        reference_snapshot_summary_for_report(),
        reference_snapshot_early_major_body_boundary_summary_for_report(),
        reference_snapshot_2451910_major_body_boundary_summary_for_report(),
        reference_snapshot_2451911_major_body_boundary_summary_for_report(),
        reference_snapshot_2451912_major_body_boundary_summary_for_report(),
        reference_snapshot_2451913_major_body_boundary_summary_for_report(),
        reference_snapshot_2451914_major_body_boundary_summary_for_report(),
        reference_snapshot_2451915_major_body_boundary_summary_for_report(),
        reference_snapshot_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_day_summary_for_report(),
        reference_snapshot_2451914_major_body_bridge_summary_for_report(),
        reference_snapshot_1750_major_body_interior_summary_for_report(),
        reference_snapshot_2451916_major_body_interior_summary_for_report(),
        reference_snapshot_2451916_major_body_dense_boundary_summary_for_report(),
        reference_snapshot_2451917_major_body_boundary_summary_for_report(),
        reference_snapshot_2451917_major_body_bridge_summary_for_report(),
        reference_snapshot_mars_jupiter_boundary_summary_for_report(),
        reference_snapshot_2451918_major_body_boundary_summary_for_report(),
        reference_snapshot_2451919_major_body_boundary_summary_for_report(),
        reference_snapshot_2451920_major_body_interior_summary_for_report(),
        reference_snapshot_body_class_coverage_summary_for_report(),
        reference_snapshot_equatorial_parity_summary_for_report(),
        reference_snapshot_batch_parity_summary_for_report(),
        production_generation_snapshot_summary_for_report(),
        production_generation_source_summary_for_report(),
        reference_snapshot_source_summary_for_report(),
        reference_snapshot_source_window_summary_for_report(),
        reference_snapshot_boundary_epoch_coverage_summary_for_report(),
        reference_snapshot_sparse_boundary_summary_for_report(),
        reference_snapshot_major_body_boundary_summary_for_report(),
        reference_holdout_overlap_summary_for_report(),
        independent_holdout_high_curvature_summary_for_report(),
        reference_snapshot_manifest_summary_for_report(),
        production_generation_boundary_source_summary_for_report(),
        production_generation_boundary_window_summary_for_report(),
        production_generation_boundary_body_class_coverage_summary_for_report(),
        production_generation_boundary_request_corpus_summary_for_report(),
        production_generation_boundary_request_corpus_equatorial_summary_for_report(),
        reference_asteroid_evidence_summary_for_report(),
        reference_asteroid_equatorial_evidence_summary_for_report(),
        reference_asteroid_source_window_summary_for_report(),
        selected_asteroid_source_2451917_summary_for_report(),
        selected_asteroid_source_2453000_summary_for_report(),
        selected_asteroid_boundary_summary_for_report(),
        selected_asteroid_bridge_summary_for_report(),
        selected_asteroid_dense_boundary_summary_for_report(),
        selected_asteroid_terminal_boundary_summary_for_report(),
        comparison_snapshot_summary_for_report(),
        comparison_snapshot_body_class_coverage_summary_for_report(),
        comparison_snapshot_source_summary_for_report(),
        comparison_snapshot_source_window_summary_for_report(),
        comparison_snapshot_manifest_summary_for_report(),
        independent_holdout_snapshot_summary_for_report(),
        independent_holdout_snapshot_equatorial_parity_summary_for_report(),
        independent_holdout_snapshot_batch_parity_summary_for_report(),
        independent_holdout_source_summary_for_report(),
        independent_holdout_snapshot_source_window_summary_for_report(),
        independent_holdout_snapshot_quarter_day_boundary_summary_for_report(),
        independent_holdout_manifest_summary_for_report(),
        jpl_independent_holdout_summary_for_report(),
    ]
    .join(" | ")
}

/// Structured request policy for the current JPL snapshot backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JplSnapshotRequestPolicy {
    /// Coordinate frames the current snapshot backend exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current snapshot backend.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current snapshot backend.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current snapshot backend.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current snapshot backend accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
}

/// Validation error for a JPL request-policy summary that drifted from the current backend posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JplSnapshotRequestPolicyValidationError {
    /// One of the request-policy fields differs from the current backend posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSnapshotRequestPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL snapshot request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSnapshotRequestPolicyValidationError {}

impl JplSnapshotRequestPolicy {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
            format_coordinate_frames(self.supported_frames),
            format_time_scales(self.supported_time_scales),
            format_zodiac_modes(self.supported_zodiac_modes),
            format_apparentness_modes(self.supported_apparentness),
            self.supports_topocentric_observer,
        )
    }

    /// Returns the compact summary line after validating the cached request policy.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSnapshotRequestPolicyValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the summary against the current JPL snapshot backend posture.
    pub fn validate(&self) -> Result<(), JplSnapshotRequestPolicyValidationError> {
        if self.supported_frames != JPL_SNAPSHOT_REQUEST_POLICY.supported_frames {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_frames",
            });
        }
        if self.supported_time_scales != JPL_SNAPSHOT_REQUEST_POLICY.supported_time_scales {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_time_scales",
            });
        }
        if self.supported_zodiac_modes != JPL_SNAPSHOT_REQUEST_POLICY.supported_zodiac_modes {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_zodiac_modes",
            });
        }
        if self.supported_apparentness != JPL_SNAPSHOT_REQUEST_POLICY.supported_apparentness {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_apparentness",
            });
        }
        if self.supports_topocentric_observer
            != JPL_SNAPSHOT_REQUEST_POLICY.supports_topocentric_observer
        {
            return Err(JplSnapshotRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer",
            });
        }
        Ok(())
    }
}

impl fmt::Display for JplSnapshotRequestPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current JPL snapshot request policy.
pub const fn jpl_snapshot_request_policy() -> JplSnapshotRequestPolicy {
    JPL_SNAPSHOT_REQUEST_POLICY
}

/// Returns the release-facing JPL snapshot request policy summary string.
pub fn jpl_snapshot_request_policy_summary_for_report() -> String {
    let policy = jpl_snapshot_request_policy();
    match policy.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("JPL snapshot request policy: unavailable ({error})"),
    }
}

/// A compact batch error-taxonomy summary for the current JPL snapshot backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplSnapshotBatchErrorTaxonomySummary {
    /// The body used for the supported batch check.
    pub supported_request_body: CelestialBody,
    /// The body used for the unsupported-body batch check.
    pub unsupported_request_body: CelestialBody,
    /// The error kind observed for the unsupported-body batch check.
    pub unsupported_error_kind: EphemerisErrorKind,
    /// The body used for the out-of-range batch check.
    pub out_of_range_request_body: CelestialBody,
    /// The error kind observed for the out-of-range batch check.
    pub out_of_range_error_kind: EphemerisErrorKind,
}

/// Structured errors for a JPL batch error-taxonomy summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JplSnapshotBatchErrorTaxonomySummaryValidationError {
    /// A summary field is out of sync with the current backend posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplSnapshotBatchErrorTaxonomySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL batch error-taxonomy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for JplSnapshotBatchErrorTaxonomySummaryValidationError {}

impl JplSnapshotBatchErrorTaxonomySummary {
    /// Returns the compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL batch error taxonomy: supported body {}; unsupported body {} -> {}; out-of-range {} -> {}",
            self.supported_request_body,
            self.unsupported_request_body,
            self.unsupported_error_kind,
            self.out_of_range_request_body,
            self.out_of_range_error_kind,
        )
    }

    /// Returns the compact summary line after validating the cached batch taxonomy.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplSnapshotBatchErrorTaxonomySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the summary against the current JPL snapshot backend posture.
    pub fn validate(&self) -> Result<(), JplSnapshotBatchErrorTaxonomySummaryValidationError> {
        if self.supported_request_body != CelestialBody::Ceres {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "supported_request_body",
                },
            );
        }
        if self.unsupported_request_body != CelestialBody::MeanNode {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_request_body",
                },
            );
        }
        if self.unsupported_error_kind != EphemerisErrorKind::UnsupportedBody {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_error_kind",
                },
            );
        }
        if self.out_of_range_request_body != CelestialBody::Ceres {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_request_body",
                },
            );
        }
        if self.out_of_range_error_kind != EphemerisErrorKind::OutOfRangeInstant {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_error_kind",
                },
            );
        }
        Ok(())
    }
}

impl fmt::Display for JplSnapshotBatchErrorTaxonomySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the control-sample batch corpus used by the current JPL batch
/// error taxonomy summary.
///
/// The requests preserve the supported-body, unsupported-body, and
/// out-of-range checks exercised by the release-facing taxonomy summary so
/// downstream tooling can reuse the exact batch shape without reconstructing it
/// inline.
pub fn jpl_snapshot_batch_error_taxonomy_requests() -> Vec<EphemerisRequest> {
    let supported_request = EphemerisRequest {
        body: CelestialBody::Ceres,
        instant: reference_instant(),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let unsupported_body_request = EphemerisRequest {
        body: CelestialBody::MeanNode,
        ..supported_request.clone()
    };
    let out_of_range_request = EphemerisRequest {
        body: CelestialBody::Ceres,
        instant: Instant::new(JulianDay::from_days(2_634_168.0), TimeScale::Tdb),
        ..supported_request.clone()
    };

    vec![
        supported_request,
        unsupported_body_request,
        out_of_range_request,
    ]
}

/// This is a compatibility alias for [`jpl_snapshot_batch_error_taxonomy_requests`].
#[doc(alias = "jpl_snapshot_batch_error_taxonomy_requests")]
pub fn jpl_snapshot_batch_error_taxonomy_request_corpus() -> Vec<EphemerisRequest> {
    jpl_snapshot_batch_error_taxonomy_requests()
}

/// Returns a compact batch error-taxonomy summary for the current JPL snapshot backend.
pub fn jpl_snapshot_batch_error_taxonomy_summary(
) -> Result<JplSnapshotBatchErrorTaxonomySummary, JplSnapshotBatchErrorTaxonomySummaryValidationError>
{
    let backend = JplSnapshotBackend;

    let requests = jpl_snapshot_batch_error_taxonomy_requests();
    let supported_request = requests[0].clone();
    let unsupported_body_request = requests[1].clone();
    let out_of_range_request = requests[2].clone();

    let unsupported_body_error =
        match backend.positions(&[supported_request.clone(), unsupported_body_request]) {
            Ok(_) => {
                return Err(
                    JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                        field: "unsupported_body_batch",
                    },
                );
            }
            Err(error) => error,
        };
    if unsupported_body_error.kind != EphemerisErrorKind::UnsupportedBody {
        return Err(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "unsupported_body_error_kind",
            },
        );
    }

    let out_of_range_error = match backend.positions(&[out_of_range_request]) {
        Ok(_) => {
            return Err(
                JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                    field: "out_of_range_batch",
                },
            );
        }
        Err(error) => error,
    };
    if out_of_range_error.kind != EphemerisErrorKind::OutOfRangeInstant {
        return Err(
            JplSnapshotBatchErrorTaxonomySummaryValidationError::FieldOutOfSync {
                field: "out_of_range_error_kind",
            },
        );
    }

    Ok(JplSnapshotBatchErrorTaxonomySummary {
        supported_request_body: CelestialBody::Ceres,
        unsupported_request_body: CelestialBody::MeanNode,
        unsupported_error_kind: EphemerisErrorKind::UnsupportedBody,
        out_of_range_request_body: CelestialBody::Ceres,
        out_of_range_error_kind: EphemerisErrorKind::OutOfRangeInstant,
    })
}

/// Returns the release-facing batch error-taxonomy summary for the current JPL snapshot backend.
pub fn jpl_snapshot_batch_error_taxonomy_summary_for_report() -> String {
    match jpl_snapshot_batch_error_taxonomy_summary() {
        Ok(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
        },
        Err(error) => format!("JPL batch error taxonomy: unavailable ({error})"),
    }
}

/// Returns the structured JPL snapshot frame-treatment summary.
pub const fn frame_treatment_summary_details() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(
        "checked-in ecliptic snapshot; equatorial coordinates are derived with a mean-obliquity transform",
    )
}

/// Returns the current JPL snapshot frame-treatment summary.
pub fn frame_treatment_summary() -> &'static str {
    frame_treatment_summary_details().summary_line()
}

/// Returns the release-facing frame-treatment summary for the current JPL snapshot backend.
///
/// The backend-owned note is validated before the compact report line is
/// rendered, so a drifted summary becomes an unavailable report rather than a
/// stale cached string.
pub fn frame_treatment_summary_for_report() -> String {
    let summary = frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("JPL frame treatment unavailable ({error})"),
    }
}

/// Returns coarse leave-one-out interpolation checks derived from the checked-in
/// fixture.
///
/// Each sample removes a middle exact fixture epoch from the body-specific
/// snapshot rows, re-runs the backend's current interpolation path, and compares
/// the interpolated result with the held-out exact sample. The current fixture is
/// intentionally sparse, so these values are evidence for report transparency
/// rather than production interpolation tolerances.
pub fn interpolation_quality_samples() -> &'static [InterpolationQualitySample] {
    interpolation_quality_sample_list()
}

/// Returns the exact ecliptic request corpus used to derive the interpolation-quality samples.
///
/// The requests preserve the checked-in sample order and stored epochs from the
/// derivative fixture, so downstream validation and reproducibility tooling can
/// reuse the exact held-out batch slice without reconstructing it from the sample
/// metadata.
pub fn interpolation_quality_sample_requests() -> Option<Vec<EphemerisRequest>> {
    snapshot_entries().map(|_| {
        interpolation_quality_samples()
            .iter()
            .map(|sample| EphemerisRequest {
                body: sample.body.clone(),
                instant: sample.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            })
            .collect()
    })
}

/// Returns the exact ecliptic request corpus used to derive the interpolation-quality samples.
///
/// This is a compatibility alias for [`interpolation_quality_sample_requests`].
#[doc(alias = "interpolation_quality_sample_requests")]
pub fn interpolation_quality_sample_request_corpus() -> Option<Vec<EphemerisRequest>> {
    interpolation_quality_sample_requests()
}

/// Compact release-facing summary for the interpolation-quality sample request corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpolationQualitySampleRequestCorpusSummary {
    /// Total number of generated requests.
    pub request_count: usize,
    /// Number of distinct bodies covered by the request corpus.
    pub body_count: usize,
    /// Bodies covered by the request corpus in first-seen order.
    pub bodies: Vec<pleiades_backend::CelestialBody>,
    /// Number of distinct epochs covered by the request corpus.
    pub epoch_count: usize,
    /// Earliest epoch represented in the request corpus.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the request corpus.
    pub latest_epoch: Instant,
    /// Coordinate frame requested by the corpus.
    pub frame: CoordinateFrame,
    /// Time scale requested by the corpus.
    pub time_scale: TimeScale,
    /// Zodiac mode requested by the corpus.
    pub zodiac_mode: ZodiacMode,
    /// Apparentness requested by the corpus.
    pub apparentness: Apparentness,
}

/// Validation error for an interpolation-quality sample request corpus summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InterpolationQualitySampleRequestCorpusSummaryValidationError {
    /// A summary field is out of sync with the checked-in request corpus.
    FieldOutOfSync { field: &'static str },
}

impl InterpolationQualitySampleRequestCorpusSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FieldOutOfSync { .. } => "field out of sync",
        }
    }
}

impl fmt::Display for InterpolationQualitySampleRequestCorpusSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the interpolation-quality sample request corpus summary field `{field}` is out of sync with the current slice"
            ),
        }
    }
}

impl std::error::Error for InterpolationQualitySampleRequestCorpusSummaryValidationError {}

impl InterpolationQualitySampleRequestCorpusSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "Interpolation-quality sample request corpus: {} requests (frame={}; time scale={}; zodiac mode={}; apparentness={}; observerless) across {} bodies and {} epochs ({}..{}); bodies: {}",
            self.request_count,
            self.frame,
            self.time_scale,
            self.zodiac_mode,
            self.apparentness,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            format_bodies(&self.bodies),
        )
    }

    /// Returns `Ok(())` when the summary still matches the checked-in request corpus.
    pub fn validate(
        &self,
    ) -> Result<(), InterpolationQualitySampleRequestCorpusSummaryValidationError> {
        let Some(expected) = interpolation_quality_sample_request_corpus_summary_details() else {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count",
                },
            );
        };

        if self.request_count != expected.request_count {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "request_count",
                },
            );
        }
        if self.body_count != expected.body_count {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.bodies != expected.bodies {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "bodies",
                },
            );
        }
        if self.epoch_count != expected.epoch_count {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }
        if self.earliest_epoch != expected.earliest_epoch {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "earliest_epoch",
                },
            );
        }
        if self.latest_epoch != expected.latest_epoch {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "latest_epoch",
                },
            );
        }
        if self.frame != expected.frame {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "frame",
                },
            );
        }
        if self.time_scale != expected.time_scale {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "time_scale",
                },
            );
        }
        if self.zodiac_mode != expected.zodiac_mode {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "zodiac_mode",
                },
            );
        }
        if self.apparentness != expected.apparentness {
            return Err(
                InterpolationQualitySampleRequestCorpusSummaryValidationError::FieldOutOfSync {
                    field: "apparentness",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated interpolation-quality request corpus summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, InterpolationQualitySampleRequestCorpusSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for InterpolationQualitySampleRequestCorpusSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn interpolation_quality_sample_request_corpus_summary_details(
) -> Option<InterpolationQualitySampleRequestCorpusSummary> {
    let samples = interpolation_quality_samples();
    let requests = interpolation_quality_sample_request_corpus()?;
    if requests.is_empty() || requests.len() != samples.len() {
        return None;
    }

    let mut bodies = Vec::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let time_scale = requests[0].instant.scale;

    for (request, sample) in requests.iter().zip(samples.iter()) {
        if request.body != sample.body
            || request.instant != sample.epoch
            || request.frame != CoordinateFrame::Ecliptic
            || request.instant.scale != time_scale
            || request.zodiac_mode != ZodiacMode::Tropical
            || request.apparent != Apparentness::Mean
            || request.observer.is_some()
        {
            return None;
        }

        if !bodies.contains(&request.body) {
            bodies.push(request.body.clone());
        }
        epochs.insert(request.instant.julian_day.days().to_bits());
        if request.instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = request.instant;
        }
        if request.instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = request.instant;
        }
    }

    Some(InterpolationQualitySampleRequestCorpusSummary {
        request_count: requests.len(),
        body_count: bodies.len(),
        bodies,
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        frame: CoordinateFrame::Ecliptic,
        time_scale,
        zodiac_mode: ZodiacMode::Tropical,
        apparentness: Apparentness::Mean,
    })
}

/// Returns the interpolation-quality sample request corpus summary.
pub fn interpolation_quality_sample_request_corpus_summary(
) -> Option<InterpolationQualitySampleRequestCorpusSummary> {
    interpolation_quality_sample_request_corpus_summary_details()
}

/// Formats the interpolation-quality sample request corpus for release-facing reporting.
pub fn format_interpolation_quality_sample_request_corpus_summary(
    summary: &InterpolationQualitySampleRequestCorpusSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing interpolation-quality sample request corpus summary string.
pub fn interpolation_quality_sample_request_corpus_summary_for_report() -> String {
    match interpolation_quality_sample_request_corpus_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => {
                format!("Interpolation-quality sample request corpus: unavailable ({error})")
            }
        },
        None => "Interpolation-quality sample request corpus: unavailable".to_string(),
    }
}

/// A compact interpolation-quality summary for the checked-in JPL snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct JplInterpolationQualitySummary {
    /// Total number of interpolation-quality samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Number of distinct epochs represented by the samples.
    pub epoch_count: usize,
    /// Earliest epoch represented by the samples.
    pub earliest_epoch: Instant,
    /// Latest epoch represented by the samples.
    pub latest_epoch: Instant,
    /// Number of samples that used cubic interpolation.
    pub cubic_sample_count: usize,
    /// Number of samples that used quadratic interpolation.
    pub quadratic_sample_count: usize,
    /// Number of samples that used linear fallback interpolation.
    pub linear_sample_count: usize,
    /// Largest bracketing span among the samples.
    pub max_bracket_span_days: f64,
    /// Body associated with the largest bracketing span.
    pub max_bracket_span_body: String,
    /// Held-out epoch associated with the largest bracketing span.
    pub max_bracket_span_epoch: Instant,
    /// Mean bracketing span across the samples.
    pub mean_bracket_span_days: f64,
    /// Median bracketing span across the samples.
    pub median_bracket_span_days: f64,
    /// 95th percentile bracketing span across the samples.
    pub percentile_bracket_span_days: f64,
    /// Largest longitude error among the samples.
    pub max_longitude_error_deg: f64,
    /// Body associated with the largest longitude error.
    pub max_longitude_error_body: String,
    /// Held-out epoch associated with the largest longitude error.
    pub max_longitude_error_epoch: Instant,
    /// Mean longitude error across the samples.
    pub mean_longitude_error_deg: f64,
    /// Median longitude error across the samples.
    pub median_longitude_error_deg: f64,
    /// 95th percentile longitude error across the samples.
    pub percentile_longitude_error_deg: f64,
    /// Root-mean-square longitude error across the samples.
    pub rms_longitude_error_deg: f64,
    /// Largest latitude error among the samples.
    pub max_latitude_error_deg: f64,
    /// Body associated with the largest latitude error.
    pub max_latitude_error_body: String,
    /// Held-out epoch associated with the largest latitude error.
    pub max_latitude_error_epoch: Instant,
    /// Mean latitude error across the samples.
    pub mean_latitude_error_deg: f64,
    /// Median latitude error across the samples.
    pub median_latitude_error_deg: f64,
    /// 95th percentile latitude error across the samples.
    pub percentile_latitude_error_deg: f64,
    /// Root-mean-square latitude error across the samples.
    pub rms_latitude_error_deg: f64,
    /// Largest distance error among the samples.
    pub max_distance_error_au: f64,
    /// Body associated with the largest distance error.
    pub max_distance_error_body: String,
    /// Held-out epoch associated with the largest distance error.
    pub max_distance_error_epoch: Instant,
    /// Mean distance error across the samples.
    pub mean_distance_error_au: f64,
    /// Median distance error across the samples.
    pub median_distance_error_au: f64,
    /// 95th percentile distance error across the samples.
    pub percentile_distance_error_au: f64,
    /// Root-mean-square distance error across the samples.
    pub rms_distance_error_au: f64,
}

impl JplInterpolationQualitySummary {
    /// Returns the compact release-facing interpolation-quality summary line.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch_suffix(body: &str, epoch: Instant) -> String {
            if body.is_empty() {
                String::new()
            } else {
                format!(" ({body} @ {})", format_instant(epoch))
            }
        }

        format!(
            "JPL interpolation quality: {} samples across {} bodies and {} epochs ({} cubic, {} quadratic, {} linear), epoch window {} → {}; leave-one-out runtime interpolation evidence with worst-case bodies named, max bracket span={:.1} d{}; mean bracket span={:.1} d; median bracket span={:.1} d; p95 bracket span={:.1} d; max Δlon={:.12}°{}; mean Δlon={:.12}°; median Δlon={:.12}°; p95 Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}°{}; mean Δlat={:.12}°; median Δlat={:.12}°; p95 Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU{}; mean Δdist={:.12} AU; median Δdist={:.12} AU; p95 Δdist={:.12} AU; rms Δdist={:.12} AU; transparency evidence only, not a production tolerance envelope",
            self.sample_count,
            self.body_count,
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.cubic_sample_count,
            self.quadratic_sample_count,
            self.linear_sample_count,
            self.max_bracket_span_days,
            format_body_epoch_suffix(&self.max_bracket_span_body, self.max_bracket_span_epoch),
            self.mean_bracket_span_days,
            self.median_bracket_span_days,
            self.percentile_bracket_span_days,
            self.max_longitude_error_deg,
            format_body_epoch_suffix(&self.max_longitude_error_body, self.max_longitude_error_epoch),
            self.mean_longitude_error_deg,
            self.median_longitude_error_deg,
            self.percentile_longitude_error_deg,
            self.rms_longitude_error_deg,
            self.max_latitude_error_deg,
            format_body_epoch_suffix(&self.max_latitude_error_body, self.max_latitude_error_epoch),
            self.mean_latitude_error_deg,
            self.median_latitude_error_deg,
            self.percentile_latitude_error_deg,
            self.rms_latitude_error_deg,
            self.max_distance_error_au,
            format_body_epoch_suffix(&self.max_distance_error_body, self.max_distance_error_epoch),
            self.mean_distance_error_au,
            self.median_distance_error_au,
            self.percentile_distance_error_au,
            self.rms_distance_error_au,
        )
    }

    /// Returns the validated compact interpolation-quality summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationQualitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A compact posture summary for the checked-in interpolation-quality evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplInterpolationPostureSummary {
    /// Source attribution for the interpolation-quality evidence posture.
    pub source: String,
    /// Release-facing posture label for the interpolation-quality evidence.
    pub detail: String,
    /// Explicit claim boundary for the interpolation-quality evidence.
    pub envelope: String,
}

impl JplInterpolationPostureSummary {
    /// Returns the compact release-facing interpolation posture summary line.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL interpolation posture: source={}; detail={}; envelope={}",
            self.source, self.detail, self.envelope
        )
    }

    /// Validates that the posture summary still matches the checked-in evidence posture.
    pub fn validate(&self) -> Result<(), JplInterpolationPostureSummaryValidationError> {
        if self.source != JPL_INTERPOLATION_POSTURE_SOURCE {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "source" },
            );
        }
        if self.detail != JPL_INTERPOLATION_POSTURE_DETAIL {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "detail" },
            );
        }
        if self.envelope != JPL_INTERPOLATION_POSTURE_ENVELOPE {
            return Err(
                JplInterpolationPostureSummaryValidationError::FieldOutOfSync { field: "envelope" },
            );
        }
        Ok(())
    }

    /// Returns the validated release-facing interpolation posture summary line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationPostureSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationPostureSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured validation errors for the interpolation-quality summary.
#[derive(Clone, Debug, PartialEq)]
pub enum JplInterpolationQualitySummaryValidationError {
    /// The summary did not expose any samples.
    MissingSamples,
    /// The summary did not expose any bodies.
    MissingBodies,
    /// The summary body count did not match the body list length.
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    /// The summary body list contained a duplicate body label.
    DuplicateBody { body: String },
    /// The summary body list contained a blank entry.
    BlankBody { index: usize },
    /// The summary did not expose any epochs.
    MissingEpochs,
    /// The summary reported an invalid earliest/latest epoch range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// A summary metric was not finite and non-negative.
    MetricOutOfRange { field: &'static str },
    /// A peak-body label was blank despite the corresponding metric being populated.
    BlankPeakBody { field: &'static str },
    /// The interpolation-kind counts did not add up to the total sample count.
    InterpolationKindCountMismatch {
        sample_count: usize,
        kind_count: usize,
    },
    /// The summary no longer matches the derived interpolation evidence.
    DerivedSummaryMismatch,
}

impl JplInterpolationQualitySummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::MissingSamples => "missing samples",
            Self::MissingBodies => "missing bodies",
            Self::BodyCountMismatch { .. } => "body count mismatch",
            Self::DuplicateBody { .. } => "duplicate body",
            Self::BlankBody { .. } => "blank body",
            Self::MissingEpochs => "missing epochs",
            Self::InvalidEpochRange { .. } => "invalid epoch range",
            Self::MetricOutOfRange { .. } => "metric out of range",
            Self::BlankPeakBody { .. } => "blank peak body",
            Self::InterpolationKindCountMismatch { .. } => "interpolation-kind count mismatch",
            Self::DerivedSummaryMismatch => "derived summary mismatch",
        }
    }
}

impl fmt::Display for JplInterpolationQualitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSamples | Self::MissingBodies | Self::MissingEpochs => {
                f.write_str(self.label())
            }
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(
                f,
                "body count {body_count} does not match body list length {bodies_len}"
            ),
            Self::DuplicateBody { body } => {
                write!(f, "body list contains duplicate body label `{body}`")
            }
            Self::BlankBody { index } => {
                write!(f, "body list entry {index} is blank")
            }
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "invalid epoch range: earliest {} is after latest {}",
                format_instant(*earliest_epoch),
                format_instant(*latest_epoch),
            ),
            Self::MetricOutOfRange { field } => write!(
                f,
                "summary metric `{field}` is not a finite non-negative value"
            ),
            Self::BlankPeakBody { field } => {
                write!(f, "summary peak body label `{field}` is blank")
            }
            Self::InterpolationKindCountMismatch {
                sample_count,
                kind_count,
            } => write!(
                f,
                "interpolation-kind count {kind_count} does not match sample count {sample_count}"
            ),
            Self::DerivedSummaryMismatch => {
                f.write_str("summary no longer matches the derived interpolation evidence")
            }
        }
    }
}

impl std::error::Error for JplInterpolationQualitySummaryValidationError {}

/// Structured validation errors for the interpolation posture summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplInterpolationPostureSummaryValidationError {
    /// A summary field is out of sync with the checked-in evidence posture.
    FieldOutOfSync { field: &'static str },
}

impl JplInterpolationPostureSummaryValidationError {
    /// Returns the compact label used in release-facing summaries and tests.
    pub const fn label(&self) -> &'static str {
        match self {
            Self::FieldOutOfSync { .. } => "field out of sync",
        }
    }
}

impl fmt::Display for JplInterpolationPostureSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL interpolation posture summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for JplInterpolationPostureSummaryValidationError {}

/// Returns the release-facing interpolation posture summary for the checked-in evidence slice.
pub fn jpl_interpolation_posture_summary() -> Option<JplInterpolationPostureSummary> {
    Some(JplInterpolationPostureSummary {
        source: JPL_INTERPOLATION_POSTURE_SOURCE.to_string(),
        detail: JPL_INTERPOLATION_POSTURE_DETAIL.to_string(),
        envelope: JPL_INTERPOLATION_POSTURE_ENVELOPE.to_string(),
    })
}

/// Formats the interpolation posture summary for release-facing reports.
pub fn format_jpl_interpolation_posture_summary(
    summary: &JplInterpolationPostureSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing interpolation posture summary string.
pub fn jpl_interpolation_posture_summary_for_report() -> String {
    match jpl_interpolation_posture_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL interpolation posture: unavailable ({error})"),
        },
        None => "JPL interpolation posture: unavailable".to_string(),
    }
}

pub(crate) fn validate_non_negative_metric(
    field: &'static str,
    value: f64,
) -> Result<(), JplInterpolationQualitySummaryValidationError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(JplInterpolationQualitySummaryValidationError::MetricOutOfRange { field })
    }
}

impl JplInterpolationQualitySummary {
    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.epoch_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingEpochs);
        }
        if self.earliest_epoch.julian_day.days() > self.latest_epoch.julian_day.days() {
            return Err(
                JplInterpolationQualitySummaryValidationError::InvalidEpochRange {
                    earliest_epoch: self.earliest_epoch,
                    latest_epoch: self.latest_epoch,
                },
            );
        }
        for (field, value) in [
            ("max_bracket_span_days", self.max_bracket_span_days),
            ("mean_bracket_span_days", self.mean_bracket_span_days),
            ("median_bracket_span_days", self.median_bracket_span_days),
            (
                "percentile_bracket_span_days",
                self.percentile_bracket_span_days,
            ),
            ("max_longitude_error_deg", self.max_longitude_error_deg),
            ("mean_longitude_error_deg", self.mean_longitude_error_deg),
            (
                "median_longitude_error_deg",
                self.median_longitude_error_deg,
            ),
            (
                "percentile_longitude_error_deg",
                self.percentile_longitude_error_deg,
            ),
            ("rms_longitude_error_deg", self.rms_longitude_error_deg),
            ("max_latitude_error_deg", self.max_latitude_error_deg),
            ("mean_latitude_error_deg", self.mean_latitude_error_deg),
            ("median_latitude_error_deg", self.median_latitude_error_deg),
            (
                "percentile_latitude_error_deg",
                self.percentile_latitude_error_deg,
            ),
            ("rms_latitude_error_deg", self.rms_latitude_error_deg),
            ("max_distance_error_au", self.max_distance_error_au),
            ("mean_distance_error_au", self.mean_distance_error_au),
            ("median_distance_error_au", self.median_distance_error_au),
            (
                "percentile_distance_error_au",
                self.percentile_distance_error_au,
            ),
            ("rms_distance_error_au", self.rms_distance_error_au),
        ] {
            validate_non_negative_metric(field, value)?;
        }
        if self.max_bracket_span_days > 0.0 && self.max_bracket_span_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_bracket_span_body",
                },
            );
        }
        if self.max_longitude_error_deg > 0.0 && self.max_longitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_longitude_error_body",
                },
            );
        }
        if self.max_latitude_error_deg > 0.0 && self.max_latitude_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_latitude_error_body",
                },
            );
        }
        if self.max_distance_error_au > 0.0 && self.max_distance_error_body.trim().is_empty() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BlankPeakBody {
                    field: "max_distance_error_body",
                },
            );
        }

        if self.sample_count
            != self.cubic_sample_count + self.quadratic_sample_count + self.linear_sample_count
        {
            return Err(
                JplInterpolationQualitySummaryValidationError::InterpolationKindCountMismatch {
                    sample_count: self.sample_count,
                    kind_count: self.cubic_sample_count
                        + self.quadratic_sample_count
                        + self.linear_sample_count,
                },
            );
        }
        if jpl_interpolation_quality_summary().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Distinct-body coverage for the interpolation-quality hold-out samples.
#[derive(Clone, Debug, PartialEq)]
pub struct JplInterpolationQualityKindCoverage {
    /// Total number of interpolation-quality samples.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Bodies represented by the samples in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct bodies represented by cubic interpolation samples.
    pub cubic_body_count: usize,
    /// Number of distinct bodies represented by quadratic interpolation samples.
    pub quadratic_body_count: usize,
    /// Number of distinct bodies represented by linear interpolation samples.
    pub linear_body_count: usize,
}

/// Returns the release-facing interpolation-quality summary for the checked-in
/// JPL snapshot.
pub fn jpl_interpolation_quality_summary() -> Option<JplInterpolationQualitySummary> {
    let samples = interpolation_quality_samples();
    if samples.is_empty() {
        return None;
    }

    let mut bodies = BTreeSet::new();
    let mut epochs = BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut cubic_sample_count = 0usize;
    let mut quadratic_sample_count = 0usize;
    let mut linear_sample_count = 0usize;
    let mut max_bracket_span_days: f64 = 0.0;
    let mut max_bracket_span_body = String::new();
    let mut max_bracket_span_epoch = samples[0].epoch;
    let mut total_bracket_span_days = 0.0;
    let mut bracket_spans = Vec::new();
    let mut max_longitude_error_deg: f64 = 0.0;
    let mut max_longitude_error_body = String::new();
    let mut max_longitude_error_epoch = samples[0].epoch;
    let mut total_longitude_error_deg = 0.0;
    let mut total_longitude_error_sq_deg = 0.0;
    let mut longitude_errors = Vec::new();
    let mut max_latitude_error_deg: f64 = 0.0;
    let mut max_latitude_error_body = String::new();
    let mut max_latitude_error_epoch = samples[0].epoch;
    let mut total_latitude_error_deg = 0.0;
    let mut total_latitude_error_sq_deg = 0.0;
    let mut latitude_errors = Vec::new();
    let mut max_distance_error_au: f64 = 0.0;
    let mut max_distance_error_body = String::new();
    let mut max_distance_error_epoch = samples[0].epoch;
    let mut total_distance_error_au = 0.0;
    let mut total_distance_error_sq_au = 0.0;
    let mut distance_errors = Vec::new();

    for sample in samples {
        bodies.insert(sample.body.to_string());
        epochs.insert(sample.epoch.julian_day.days().to_bits());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
        match sample.interpolation_kind {
            InterpolationQualityKind::Cubic => cubic_sample_count += 1,
            InterpolationQualityKind::Quadratic => quadratic_sample_count += 1,
            InterpolationQualityKind::Linear => linear_sample_count += 1,
        }
        total_bracket_span_days += sample.bracket_span_days;
        bracket_spans.push(sample.bracket_span_days);
        total_longitude_error_deg += sample.longitude_error_deg;
        total_longitude_error_sq_deg += sample.longitude_error_deg * sample.longitude_error_deg;
        longitude_errors.push(sample.longitude_error_deg);
        total_latitude_error_deg += sample.latitude_error_deg;
        total_latitude_error_sq_deg += sample.latitude_error_deg * sample.latitude_error_deg;
        latitude_errors.push(sample.latitude_error_deg);
        total_distance_error_au += sample.distance_error_au;
        total_distance_error_sq_au += sample.distance_error_au * sample.distance_error_au;
        distance_errors.push(sample.distance_error_au);
        if sample.bracket_span_days > max_bracket_span_days {
            max_bracket_span_days = sample.bracket_span_days;
            max_bracket_span_body = sample.body.to_string();
            max_bracket_span_epoch = sample.epoch;
        }
        if sample.longitude_error_deg > max_longitude_error_deg {
            max_longitude_error_deg = sample.longitude_error_deg;
            max_longitude_error_body = sample.body.to_string();
            max_longitude_error_epoch = sample.epoch;
        }
        if sample.latitude_error_deg > max_latitude_error_deg {
            max_latitude_error_deg = sample.latitude_error_deg;
            max_latitude_error_body = sample.body.to_string();
            max_latitude_error_epoch = sample.epoch;
        }
        if sample.distance_error_au > max_distance_error_au {
            max_distance_error_au = sample.distance_error_au;
            max_distance_error_body = sample.body.to_string();
            max_distance_error_epoch = sample.epoch;
        }
    }

    let sample_count = samples.len() as f64;

    Some(JplInterpolationQualitySummary {
        median_bracket_span_days: median_f64(&mut bracket_spans),
        percentile_bracket_span_days: percentile_f64(&mut bracket_spans, 0.95),
        sample_count: samples.len(),
        body_count: bodies.len(),
        epoch_count: epochs.len(),
        earliest_epoch,
        latest_epoch,
        cubic_sample_count,
        quadratic_sample_count,
        linear_sample_count,
        max_bracket_span_days,
        max_bracket_span_body,
        max_bracket_span_epoch,
        mean_bracket_span_days: total_bracket_span_days / sample_count,
        max_longitude_error_deg,
        max_longitude_error_body,
        max_longitude_error_epoch,
        mean_longitude_error_deg: total_longitude_error_deg / sample_count,
        median_longitude_error_deg: median_f64(&mut longitude_errors),
        percentile_longitude_error_deg: percentile_f64(&mut longitude_errors, 0.95),
        rms_longitude_error_deg: (total_longitude_error_sq_deg / sample_count).sqrt(),
        max_latitude_error_deg,
        max_latitude_error_body,
        max_latitude_error_epoch,
        mean_latitude_error_deg: total_latitude_error_deg / sample_count,
        median_latitude_error_deg: median_f64(&mut latitude_errors),
        percentile_latitude_error_deg: percentile_f64(&mut latitude_errors, 0.95),
        rms_latitude_error_deg: (total_latitude_error_sq_deg / sample_count).sqrt(),
        max_distance_error_au,
        max_distance_error_body,
        max_distance_error_epoch,
        mean_distance_error_au: total_distance_error_au / sample_count,
        median_distance_error_au: median_f64(&mut distance_errors),
        percentile_distance_error_au: percentile_f64(&mut distance_errors, 0.95),
        rms_distance_error_au: (total_distance_error_sq_au / sample_count).sqrt(),
    })
}

/// Returns the distinct-body coverage breakdown for the interpolation-quality
/// hold-out samples.
pub fn jpl_interpolation_quality_kind_coverage() -> Option<JplInterpolationQualityKindCoverage> {
    let samples = interpolation_quality_samples();
    if samples.is_empty() {
        return None;
    }

    let mut all_bodies = BTreeSet::new();
    let mut first_seen_bodies = Vec::new();
    let mut cubic_bodies = BTreeSet::new();
    let mut quadratic_bodies = BTreeSet::new();
    let mut linear_bodies = BTreeSet::new();

    for sample in samples {
        let body = sample.body.to_string();
        if all_bodies.insert(body.clone()) {
            first_seen_bodies.push(body.clone());
        }
        match sample.interpolation_kind {
            InterpolationQualityKind::Cubic => {
                cubic_bodies.insert(body);
            }
            InterpolationQualityKind::Quadratic => {
                quadratic_bodies.insert(body);
            }
            InterpolationQualityKind::Linear => {
                linear_bodies.insert(body);
            }
        }
    }

    Some(JplInterpolationQualityKindCoverage {
        sample_count: samples.len(),
        body_count: all_bodies.len(),
        bodies: first_seen_bodies,
        cubic_body_count: cubic_bodies.len(),
        quadratic_body_count: quadratic_bodies.len(),
        linear_body_count: linear_bodies.len(),
    })
}

impl JplInterpolationQualityKindCoverage {
    /// Returns the compact release-facing coverage summary line.
    pub fn summary_line(&self) -> String {
        let bodies = if self.bodies.is_empty() {
            "none".to_string()
        } else {
            self.bodies.join(", ")
        };

        format!(
            "JPL interpolation quality kind coverage: {} samples across {} bodies [{}] ({} cubic bodies, {} quadratic bodies, {} linear bodies)",
            self.sample_count,
            self.body_count,
            bodies,
            self.cubic_body_count,
            self.quadratic_body_count,
            self.linear_body_count,
        )
    }

    /// Returns a compact summary line after validating the coverage summary.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationQualityKindCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl JplInterpolationQualityKindCoverage {
    /// Validates that the coverage summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySummaryValidationError> {
        if self.sample_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingSamples);
        }
        if self.body_count == 0 {
            return Err(JplInterpolationQualitySummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                JplInterpolationQualitySummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }

        let mut seen_bodies = BTreeSet::new();
        for (index, body) in self.bodies.iter().enumerate() {
            if body.trim().is_empty() {
                return Err(JplInterpolationQualitySummaryValidationError::BlankBody { index });
            }
            if !seen_bodies.insert(body) {
                return Err(
                    JplInterpolationQualitySummaryValidationError::DuplicateBody {
                        body: body.clone(),
                    },
                );
            }
        }

        if jpl_interpolation_quality_kind_coverage().as_ref() != Some(self) {
            return Err(JplInterpolationQualitySummaryValidationError::DerivedSummaryMismatch);
        }

        Ok(())
    }
}

/// Formats the interpolation-quality summary for release-facing reports.
pub fn format_jpl_interpolation_quality_summary(
    summary: &JplInterpolationQualitySummary,
) -> String {
    summary.summary_line()
}

/// Formats the distinct-body interpolation-kind coverage for release-facing reports.
pub fn format_jpl_interpolation_quality_kind_coverage(
    coverage: &JplInterpolationQualityKindCoverage,
) -> String {
    coverage.summary_line()
}

/// Returns the release-facing interpolation-kind coverage summary string.
pub fn jpl_interpolation_quality_kind_coverage_for_report() -> String {
    match jpl_interpolation_quality_kind_coverage() {
        Some(coverage) => match coverage.validated_summary_line() {
            Ok(rendered) => rendered,
            Err(_) => "JPL interpolation quality kind coverage: unavailable".to_string(),
        },
        None => "JPL interpolation quality kind coverage: unavailable".to_string(),
    }
}

/// Backend-owned provenance summary for the interpolation-quality evidence slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JplInterpolationQualitySourceSummary {
    /// Source attribution for the interpolation-quality evidence.
    pub source: String,
    /// Derivation note describing how the evidence slice was produced.
    pub derivation: String,
    /// Number of interpolation-quality samples in the evidence slice.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the evidence slice.
    pub body_count: usize,
    /// Number of distinct epochs represented by the evidence slice.
    pub epoch_count: usize,
}

/// Structured validation errors for an interpolation-quality provenance summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JplInterpolationQualitySourceSummaryValidationError {
    /// The summary did not include a non-empty source label.
    BlankSource,
    /// The summary did not include a non-empty derivation note.
    BlankDerivation,
    /// The summary drifted away from the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for JplInterpolationQualitySourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSource => f.write_str("blank source"),
            Self::BlankDerivation => f.write_str("blank derivation"),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the JPL interpolation-quality source summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for JplInterpolationQualitySourceSummaryValidationError {}

impl JplInterpolationQualitySourceSummary {
    /// Returns a compact release-facing provenance line.
    pub fn summary_line(&self) -> String {
        format!(
            "JPL interpolation quality source: {}; derivation={}; coverage: {} samples across {} bodies and {} epochs",
            self.source, self.derivation, self.sample_count, self.body_count, self.epoch_count,
        )
    }

    /// Returns a compact provenance line after validating the current evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationQualitySourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates that the summary remains internally consistent and still matches the derived evidence.
    pub fn validate(&self) -> Result<(), JplInterpolationQualitySourceSummaryValidationError> {
        if self.source.trim().is_empty() {
            return Err(JplInterpolationQualitySourceSummaryValidationError::BlankSource);
        }
        if self.derivation.trim().is_empty() {
            return Err(JplInterpolationQualitySourceSummaryValidationError::BlankDerivation);
        }

        let reference_source = reference_snapshot_source_summary().source;
        if self.source != reference_source {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "source",
                },
            );
        }
        if self.derivation != JPL_INTERPOLATION_QUALITY_DERIVATION {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "derivation",
                },
            );
        }

        let derived_summary = jpl_interpolation_quality_summary().ok_or(
            JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                field: "derived_summary",
            },
        )?;
        if self.sample_count != derived_summary.sample_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != derived_summary.body_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.epoch_count != derived_summary.epoch_count {
            return Err(
                JplInterpolationQualitySourceSummaryValidationError::FieldOutOfSync {
                    field: "epoch_count",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for JplInterpolationQualitySourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned provenance summary for the interpolation-quality evidence slice.
pub fn jpl_interpolation_quality_source_summary() -> Option<JplInterpolationQualitySourceSummary> {
    let summary = jpl_interpolation_quality_summary()?;
    Some(JplInterpolationQualitySourceSummary {
        source: reference_snapshot_source_summary().source,
        derivation: JPL_INTERPOLATION_QUALITY_DERIVATION.to_string(),
        sample_count: summary.sample_count,
        body_count: summary.body_count,
        epoch_count: summary.epoch_count,
    })
}

/// Returns the release-facing interpolation-quality provenance summary string.
pub fn jpl_interpolation_quality_source_summary_for_report() -> String {
    match jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(summary_line) => summary_line,
            Err(error) => format!("JPL interpolation quality source: unavailable ({error})"),
        },
        None => "JPL interpolation quality source: unavailable".to_string(),
    }
}

/// Formats the interpolation-quality summary together with the distinct-body coverage
/// and sample request corpus lines.
pub fn format_jpl_interpolation_quality_summary_for_report() -> String {
    let source_summary = match jpl_interpolation_quality_source_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(rendered) => rendered,
            Err(_) => return "JPL interpolation quality: unavailable".to_string(),
        },
        None => return "JPL interpolation quality: unavailable".to_string(),
    };

    match jpl_interpolation_quality_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(mut rendered) => {
                rendered.insert_str(0, &format!("{}\n", source_summary));
                rendered.push('\n');
                rendered.push_str(&jpl_interpolation_quality_kind_coverage_for_report());
                rendered.push('\n');
                rendered
                    .push_str(&interpolation_quality_sample_request_corpus_summary_for_report());
                rendered.push('\n');
                rendered.push_str(&jpl_interpolation_body_class_error_envelopes_for_report());
                rendered
            }
            Err(_) => "JPL interpolation quality: unavailable".to_string(),
        },
        None => "JPL interpolation quality: unavailable".to_string(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct JplInterpolationBodyClassErrorEnvelopeSummary {
    /// Body class represented by this envelope.
    pub class: &'static str,
    /// Total number of interpolation-quality samples in the class.
    pub sample_count: usize,
    /// Number of distinct bodies represented by the samples.
    pub body_count: usize,
    /// Bodies represented by the samples in first-seen order.
    pub bodies: Vec<String>,
    /// Number of distinct epochs represented by the samples.
    pub epoch_count: usize,
    /// Earliest epoch represented by the samples.
    pub earliest_epoch: Instant,
    /// Latest epoch represented by the samples.
    pub latest_epoch: Instant,
    /// Largest longitude error among the samples.
    pub max_longitude_error_deg: f64,
    /// Body associated with the largest longitude error.
    pub max_longitude_error_body: String,
    /// Held-out epoch associated with the largest longitude error.
    pub max_longitude_error_epoch: Instant,
    /// Mean longitude error across the samples.
    pub mean_longitude_error_deg: f64,
    /// Root-mean-square longitude error across the samples.
    pub rms_longitude_error_deg: f64,
    /// Largest latitude error among the samples.
    pub max_latitude_error_deg: f64,
    /// Body associated with the largest latitude error.
    pub max_latitude_error_body: String,
    /// Held-out epoch associated with the largest latitude error.
    pub max_latitude_error_epoch: Instant,
    /// Mean latitude error across the samples.
    pub mean_latitude_error_deg: f64,
    /// Root-mean-square latitude error across the samples.
    pub rms_latitude_error_deg: f64,
    /// Largest distance error among the samples.
    pub max_distance_error_au: f64,
    /// Body associated with the largest distance error.
    pub max_distance_error_body: String,
    /// Held-out epoch associated with the largest distance error.
    pub max_distance_error_epoch: Instant,
    /// Mean distance error across the samples.
    pub mean_distance_error_au: f64,
    /// Root-mean-square distance error across the samples.
    pub rms_distance_error_au: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JplInterpolationBodyClassErrorEnvelopeSummaryValidationError {
    /// No interpolation-quality samples were available.
    MissingSamples,
    /// A rendered summary line drifted from the current evidence.
    FieldOutOfSync { class: &'static str },
}

impl fmt::Display for JplInterpolationBodyClassErrorEnvelopeSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSamples => f.write_str(
                "JPL interpolation body-class error envelopes are unavailable",
            ),
            Self::FieldOutOfSync { class } => write!(
                f,
                "the JPL interpolation body-class error envelope for {class} is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for JplInterpolationBodyClassErrorEnvelopeSummaryValidationError {}

impl JplInterpolationBodyClassErrorEnvelopeSummary {
    fn summary_line(&self) -> String {
        format!(
            "JPL interpolation body-class error envelope: {}: {} samples across {} bodies [{}] and {} epochs ({} → {}); max Δlon={:.12}° ({} @ {}); mean Δlon={:.12}°; rms Δlon={:.12}°; max Δlat={:.12}° ({} @ {}); mean Δlat={:.12}°; rms Δlat={:.12}°; max Δdist={:.12} AU ({} @ {}); mean Δdist={:.12} AU; rms Δdist={:.12} AU",
            self.class,
            self.sample_count,
            self.body_count,
            self.bodies.join(", "),
            self.epoch_count,
            format_instant(self.earliest_epoch),
            format_instant(self.latest_epoch),
            self.max_longitude_error_deg,
            self.max_longitude_error_body,
            format_instant(self.max_longitude_error_epoch),
            self.mean_longitude_error_deg,
            self.rms_longitude_error_deg,
            self.max_latitude_error_deg,
            self.max_latitude_error_body,
            format_instant(self.max_latitude_error_epoch),
            self.mean_latitude_error_deg,
            self.rms_latitude_error_deg,
            self.max_distance_error_au,
            self.max_distance_error_body,
            format_instant(self.max_distance_error_epoch),
            self.mean_distance_error_au,
            self.rms_distance_error_au,
        )
    }

    pub(crate) fn validate(
        &self,
    ) -> Result<(), JplInterpolationBodyClassErrorEnvelopeSummaryValidationError> {
        let Some(expected_summaries) = jpl_interpolation_body_class_error_envelopes() else {
            return Err(
                JplInterpolationBodyClassErrorEnvelopeSummaryValidationError::MissingSamples,
            );
        };

        let Some(expected_summary) = expected_summaries
            .iter()
            .find(|summary| summary.class == self.class)
        else {
            return Err(
                JplInterpolationBodyClassErrorEnvelopeSummaryValidationError::FieldOutOfSync {
                    class: self.class,
                },
            );
        };

        if self.summary_line() != expected_summary.summary_line() {
            return Err(
                JplInterpolationBodyClassErrorEnvelopeSummaryValidationError::FieldOutOfSync {
                    class: self.class,
                },
            );
        }

        Ok(())
    }

    fn validated_summary_line(
        &self,
    ) -> Result<String, JplInterpolationBodyClassErrorEnvelopeSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for JplInterpolationBodyClassErrorEnvelopeSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) struct JplInterpolationBodyClassErrorEnvelopeAccumulator {
    class: &'static str,
    sample_count: usize,
    bodies: Vec<String>,
    seen_bodies: BTreeSet<String>,
    epochs: BTreeSet<u64>,
    earliest_epoch: Option<Instant>,
    latest_epoch: Option<Instant>,
    max_longitude_error_deg: f64,
    max_longitude_error_body: String,
    max_longitude_error_epoch: Instant,
    sum_longitude_error_deg: f64,
    sum_longitude_error_sq_deg: f64,
    max_latitude_error_deg: f64,
    max_latitude_error_body: String,
    max_latitude_error_epoch: Instant,
    sum_latitude_error_deg: f64,
    sum_latitude_error_sq_deg: f64,
    max_distance_error_au: f64,
    max_distance_error_body: String,
    max_distance_error_epoch: Instant,
    sum_distance_error_au: f64,
    sum_distance_error_sq_au: f64,
}

impl JplInterpolationBodyClassErrorEnvelopeAccumulator {
    fn new(class: &'static str) -> Self {
        Self {
            class,
            sample_count: 0,
            bodies: Vec::new(),
            seen_bodies: BTreeSet::new(),
            epochs: BTreeSet::new(),
            earliest_epoch: None,
            latest_epoch: None,
            max_longitude_error_deg: 0.0,
            max_longitude_error_body: String::new(),
            max_longitude_error_epoch: reference_instant(),
            sum_longitude_error_deg: 0.0,
            sum_longitude_error_sq_deg: 0.0,
            max_latitude_error_deg: 0.0,
            max_latitude_error_body: String::new(),
            max_latitude_error_epoch: reference_instant(),
            sum_latitude_error_deg: 0.0,
            sum_latitude_error_sq_deg: 0.0,
            max_distance_error_au: 0.0,
            max_distance_error_body: String::new(),
            max_distance_error_epoch: reference_instant(),
            sum_distance_error_au: 0.0,
            sum_distance_error_sq_au: 0.0,
        }
    }

    fn push(&mut self, sample: &InterpolationQualitySample) {
        self.sample_count += 1;

        let body = sample.body.to_string();
        if self.seen_bodies.insert(body.clone()) {
            self.bodies.push(body.clone());
        }
        self.epochs.insert(sample.epoch.julian_day.days().to_bits());
        self.earliest_epoch = Some(match self.earliest_epoch {
            Some(current) if current.julian_day.days() <= sample.epoch.julian_day.days() => current,
            _ => sample.epoch,
        });
        self.latest_epoch = Some(match self.latest_epoch {
            Some(current) if current.julian_day.days() >= sample.epoch.julian_day.days() => current,
            _ => sample.epoch,
        });

        self.sum_longitude_error_deg += sample.longitude_error_deg;
        self.sum_longitude_error_sq_deg += sample.longitude_error_deg * sample.longitude_error_deg;
        self.sum_latitude_error_deg += sample.latitude_error_deg;
        self.sum_latitude_error_sq_deg += sample.latitude_error_deg * sample.latitude_error_deg;
        self.sum_distance_error_au += sample.distance_error_au;
        self.sum_distance_error_sq_au += sample.distance_error_au * sample.distance_error_au;

        if sample.longitude_error_deg >= self.max_longitude_error_deg {
            self.max_longitude_error_deg = sample.longitude_error_deg;
            self.max_longitude_error_body = body.clone();
            self.max_longitude_error_epoch = sample.epoch;
        }
        if sample.latitude_error_deg >= self.max_latitude_error_deg {
            self.max_latitude_error_deg = sample.latitude_error_deg;
            self.max_latitude_error_body = body.clone();
            self.max_latitude_error_epoch = sample.epoch;
        }
        if sample.distance_error_au >= self.max_distance_error_au {
            self.max_distance_error_au = sample.distance_error_au;
            self.max_distance_error_body = body;
            self.max_distance_error_epoch = sample.epoch;
        }
    }

    fn finish(self) -> Option<JplInterpolationBodyClassErrorEnvelopeSummary> {
        let earliest_epoch = self.earliest_epoch?;
        let latest_epoch = self.latest_epoch?;
        let sample_count = self.sample_count as f64;

        Some(JplInterpolationBodyClassErrorEnvelopeSummary {
            class: self.class,
            sample_count: self.sample_count,
            body_count: self.bodies.len(),
            bodies: self.bodies,
            epoch_count: self.epochs.len(),
            earliest_epoch,
            latest_epoch,
            max_longitude_error_deg: self.max_longitude_error_deg,
            max_longitude_error_body: self.max_longitude_error_body,
            max_longitude_error_epoch: self.max_longitude_error_epoch,
            mean_longitude_error_deg: self.sum_longitude_error_deg / sample_count,
            rms_longitude_error_deg: (self.sum_longitude_error_sq_deg / sample_count).sqrt(),
            max_latitude_error_deg: self.max_latitude_error_deg,
            max_latitude_error_body: self.max_latitude_error_body,
            max_latitude_error_epoch: self.max_latitude_error_epoch,
            mean_latitude_error_deg: self.sum_latitude_error_deg / sample_count,
            rms_latitude_error_deg: (self.sum_latitude_error_sq_deg / sample_count).sqrt(),
            max_distance_error_au: self.max_distance_error_au,
            max_distance_error_body: self.max_distance_error_body,
            max_distance_error_epoch: self.max_distance_error_epoch,
            mean_distance_error_au: self.sum_distance_error_au / sample_count,
            rms_distance_error_au: (self.sum_distance_error_sq_au / sample_count).sqrt(),
        })
    }
}

pub(crate) fn interpolation_quality_body_class_index(
    body: &pleiades_backend::CelestialBody,
) -> usize {
    match body {
        pleiades_backend::CelestialBody::Sun | pleiades_backend::CelestialBody::Moon => 0,
        pleiades_backend::CelestialBody::Mercury
        | pleiades_backend::CelestialBody::Venus
        | pleiades_backend::CelestialBody::Mars
        | pleiades_backend::CelestialBody::Jupiter
        | pleiades_backend::CelestialBody::Saturn
        | pleiades_backend::CelestialBody::Uranus
        | pleiades_backend::CelestialBody::Neptune
        | pleiades_backend::CelestialBody::Pluto => 1,
        pleiades_backend::CelestialBody::MeanNode
        | pleiades_backend::CelestialBody::TrueNode
        | pleiades_backend::CelestialBody::MeanApogee
        | pleiades_backend::CelestialBody::TrueApogee
        | pleiades_backend::CelestialBody::MeanPerigee
        | pleiades_backend::CelestialBody::TruePerigee => 2,
        pleiades_backend::CelestialBody::Ceres
        | pleiades_backend::CelestialBody::Pallas
        | pleiades_backend::CelestialBody::Juno
        | pleiades_backend::CelestialBody::Vesta => 3,
        pleiades_backend::CelestialBody::Custom(_) => 4,
        _ => 4,
    }
}

/// Returns the body-class error envelopes for the interpolation-quality samples.
pub fn jpl_interpolation_body_class_error_envelopes(
) -> Option<Vec<JplInterpolationBodyClassErrorEnvelopeSummary>> {
    let samples = interpolation_quality_samples();
    if samples.is_empty() {
        return None;
    }

    let mut accumulators = [
        JplInterpolationBodyClassErrorEnvelopeAccumulator::new("Luminaries"),
        JplInterpolationBodyClassErrorEnvelopeAccumulator::new("Major planets"),
        JplInterpolationBodyClassErrorEnvelopeAccumulator::new("Lunar points"),
        JplInterpolationBodyClassErrorEnvelopeAccumulator::new("Selected asteroids"),
        JplInterpolationBodyClassErrorEnvelopeAccumulator::new("Custom bodies"),
    ];

    for sample in samples {
        accumulators[interpolation_quality_body_class_index(&sample.body)].push(sample);
    }

    let summaries = accumulators
        .into_iter()
        .filter_map(JplInterpolationBodyClassErrorEnvelopeAccumulator::finish)
        .collect::<Vec<_>>();

    if summaries.is_empty() {
        None
    } else {
        Some(summaries)
    }
}

/// Returns the release-facing body-class error envelopes for the interpolation-quality samples.
pub fn jpl_interpolation_body_class_error_envelopes_for_report() -> String {
    match jpl_interpolation_body_class_error_envelopes() {
        Some(summaries) => {
            let mut rendered = String::from("JPL interpolation body-class error envelopes:");
            for summary in summaries {
                match summary.validated_summary_line() {
                    Ok(summary_line) => {
                        rendered.push('\n');
                        rendered.push_str(&summary_line);
                    }
                    Err(error) => {
                        return format!(
                            "JPL interpolation body-class error envelopes: unavailable ({error})"
                        )
                    }
                }
            }
            rendered
        }
        None => "JPL interpolation body-class error envelopes: unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests;
