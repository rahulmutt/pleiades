use super::*;

/// Release-state for the packaged-artifact target thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackagedArtifactTargetThresholdState {
    /// Calibrated fit envelope is recorded, but production thresholds are not yet release-ready.
    Draft,
    /// Production thresholds have been finalized for the current profile.
    ProductionReady,
}

/// Validation error for a packaged-artifact target-threshold state that is not release-ready.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactTargetThresholdStateValidationError {
    /// The target-threshold state is still draft.
    Draft,
}

impl fmt::Display for PackagedArtifactTargetThresholdStateValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Draft => write!(
                f,
                "the packaged-artifact target-threshold state is draft; production thresholds are not yet release-ready"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactTargetThresholdStateValidationError {}

impl PackagedArtifactTargetThresholdState {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Draft => {
                "calibrated fit envelope recorded; production thresholds not yet release-ready"
            }
            Self::ProductionReady => "production thresholds recorded",
        }
    }

    /// Returns a compact human-readable line for the target-threshold state.
    pub fn summary_line(self) -> String {
        format!("target-threshold state: {}", self)
    }

    /// Returns whether the target thresholds are finalized for production release.
    pub const fn is_production_ready(self) -> bool {
        matches!(self, Self::ProductionReady)
    }

    /// Returns `Ok(())` when the state is release-ready.
    pub fn validate_production_ready(
        self,
    ) -> Result<(), PackagedArtifactTargetThresholdStateValidationError> {
        if self.is_production_ready() {
            Ok(())
        } else {
            Err(PackagedArtifactTargetThresholdStateValidationError::Draft)
        }
    }

    /// Returns the validated target-threshold state as a compact human-readable line.
    pub fn validated_summary_line(
        self,
    ) -> Result<String, PackagedArtifactTargetThresholdStateValidationError> {
        self.validate_production_ready()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

const PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE: PackagedArtifactTargetThresholdState =
    PackagedArtifactTargetThresholdState::ProductionReady;
pub(crate) const PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES: &[&str] = &[
    "luminaries",
    "major planets",
    "pluto",
    "lunar points",
    "selected asteroids",
    "custom bodies",
];

/// Phase-2 corpus evidence used to keep the packaged-artifact threshold policy aligned
/// with the current reference, fixture-exactness, comparison, hold-out, selected-asteroid, boundary-overlay, and production-generation corpora.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactPhase2CorpusAlignmentSummary {
    /// Source-material evidence from the checked-in reference snapshot.
    pub reference_snapshot_source: pleiades_jpl::ReferenceSnapshotSourceSummary,
    /// Body-class coverage evidence from the checked-in reference snapshot.
    pub reference_snapshot: pleiades_jpl::ReferenceSnapshotBodyClassCoverageSummary,
    /// Exact J2000 fixture-exactness evidence from the checked-in reference snapshot.
    pub reference_snapshot_exact_j2000: pleiades_jpl::ReferenceSnapshotExactJ2000EvidenceSummary,
    /// Source-material evidence from the checked-in comparison snapshot.
    pub comparison_snapshot_source: pleiades_jpl::ComparisonSnapshotSourceSummary,
    /// Body-class coverage evidence from the checked-in comparison snapshot.
    pub comparison_snapshot: pleiades_jpl::ComparisonSnapshotBodyClassCoverageSummary,
    /// Source-material evidence from the checked-in independent hold-out snapshot.
    pub independent_holdout_source: pleiades_jpl::IndependentHoldoutSourceSummary,
    /// Body-class coverage evidence from the checked-in independent hold-out snapshot.
    pub independent_holdout: pleiades_jpl::IndependentHoldoutSnapshotBodyClassCoverageSummary,
    /// Source-backed evidence for the selected-asteroid validation corpus.
    pub selected_asteroid_source: pleiades_jpl::SelectedAsteroidSourceSummary,
    /// Source-backed window evidence for the selected-asteroid validation corpus.
    pub selected_asteroid_source_windows: pleiades_jpl::SelectedAsteroidSourceWindowSummary,
    /// Checked-in request-corpus evidence for the selected-asteroid validation corpus in the ecliptic frame.
    pub selected_asteroid_source_request_corpus:
        pleiades_jpl::SelectedAsteroidSourceRequestCorpusSummary,
    /// Checked-in request-corpus evidence for the selected-asteroid validation corpus in the equatorial frame.
    pub selected_asteroid_source_request_corpus_equatorial:
        pleiades_jpl::SelectedAsteroidSourceRequestCorpusSummary,
    /// Source-material evidence for the checked-in production-generation boundary overlay.
    pub production_generation_boundary_source: pleiades_jpl::IndependentHoldoutSourceSummary,
    /// Body-class coverage evidence for the checked-in production-generation corpus.
    pub production_generation_body_class_coverage:
        pleiades_jpl::ProductionGenerationSnapshotBodyClassCoverageSummary,
    /// Combined provenance for the checked-in production-generation corpus.
    pub production_generation_source: ProductionGenerationSourceSummary,
}

/// Validation error for a phase-2 corpus alignment summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact phase-2 corpus alignment summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactPhase2CorpusAlignmentSummaryValidationError {}

fn phase2_corpus_alignment_validation_field_path(field: &'static str) -> &'static str {
    match field {
        "reference_snapshot_source" => "phase2_corpus_alignment.reference_snapshot_source",
        "reference_snapshot" => "phase2_corpus_alignment.reference_snapshot",
        "reference_snapshot_exact_j2000" => {
            "phase2_corpus_alignment.reference_snapshot_exact_j2000"
        }
        "comparison_snapshot_source" => "phase2_corpus_alignment.comparison_snapshot_source",
        "comparison_snapshot" => "phase2_corpus_alignment.comparison_snapshot",
        "independent_holdout_source" => "phase2_corpus_alignment.independent_holdout_source",
        "independent_holdout" => "phase2_corpus_alignment.independent_holdout",
        "selected_asteroid_source" => "phase2_corpus_alignment.selected_asteroid_source",
        "selected_asteroid_source_windows" => {
            "phase2_corpus_alignment.selected_asteroid_source_windows"
        }
        "selected_asteroid_source_request_corpus" => {
            "phase2_corpus_alignment.selected_asteroid_source_request_corpus"
        }
        "selected_asteroid_source_request_corpus_equatorial" => {
            "phase2_corpus_alignment.selected_asteroid_source_request_corpus_equatorial"
        }
        "production_generation_boundary_source" => {
            "phase2_corpus_alignment.production_generation_boundary_source"
        }
        "production_generation_body_class_coverage" => {
            "phase2_corpus_alignment.production_generation_body_class_coverage"
        }
        "production_generation_source" => "phase2_corpus_alignment.production_generation_source",
        _ => "phase2_corpus_alignment",
    }
}

impl PackagedArtifactPhase2CorpusAlignmentSummary {
    /// Returns the phase-2 corpus alignment posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "reference source={}; reference snapshot={}; reference exact J2000 evidence={}; comparison source={}; comparison snapshot={}; independent hold-out source={}; independent hold-out={}; selected asteroid source evidence={}; selected asteroid source windows={}; selected asteroid source request corpus={}; selected asteroid source request corpus equatorial={}; production generation boundary source={}; production generation body-class coverage={}; production generation source={}",
            self.reference_snapshot_source.summary_line(),
            self.reference_snapshot.summary_line(),
            self.reference_snapshot_exact_j2000.summary_line(),
            self.comparison_snapshot_source.summary_line(),
            self.comparison_snapshot.summary_line(),
            self.independent_holdout_source.summary_line(),
            self.independent_holdout.summary_line(),
            self.selected_asteroid_source.summary_line(),
            self.selected_asteroid_source_windows.summary_line(),
            self.selected_asteroid_source_request_corpus.summary_line(),
            self.selected_asteroid_source_request_corpus_equatorial.summary_line(),
            pleiades_jpl::format_production_generation_boundary_source_summary(
                &self.production_generation_boundary_source,
            ),
            self.production_generation_body_class_coverage.summary_line(),
            self.production_generation_source.summary_line(),
        )
    }

    /// Returns `Ok(())` when the phase-2 corpus evidence still matches the current packaged-artifact posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactPhase2CorpusAlignmentSummaryValidationError> {
        let Some(expected) = packaged_artifact_phase2_corpus_alignment_summary_details() else {
            return Err(
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        };

        let field_out_of_sync = |field| {
            PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync { field }
        };

        if self.reference_snapshot_source != expected.reference_snapshot_source {
            return Err(field_out_of_sync("reference_snapshot_source"));
        }
        self.reference_snapshot_source
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot_source"))?;

        if self.reference_snapshot != expected.reference_snapshot {
            return Err(field_out_of_sync("reference_snapshot"));
        }
        self.reference_snapshot
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot"))?;

        if self.reference_snapshot_exact_j2000 != expected.reference_snapshot_exact_j2000 {
            return Err(field_out_of_sync("reference_snapshot_exact_j2000"));
        }
        self.reference_snapshot_exact_j2000
            .validate()
            .map_err(|_| field_out_of_sync("reference_snapshot_exact_j2000"))?;

        if self.comparison_snapshot_source != expected.comparison_snapshot_source {
            return Err(field_out_of_sync("comparison_snapshot_source"));
        }
        self.comparison_snapshot_source
            .validate()
            .map_err(|_| field_out_of_sync("comparison_snapshot_source"))?;

        if self.comparison_snapshot != expected.comparison_snapshot {
            return Err(field_out_of_sync("comparison_snapshot"));
        }
        self.comparison_snapshot
            .validate()
            .map_err(|_| field_out_of_sync("comparison_snapshot"))?;

        if self.independent_holdout_source != expected.independent_holdout_source {
            return Err(field_out_of_sync("independent_holdout_source"));
        }
        self.independent_holdout_source
            .validate()
            .map_err(|_| field_out_of_sync("independent_holdout_source"))?;

        if self.independent_holdout != expected.independent_holdout {
            return Err(field_out_of_sync("independent_holdout"));
        }
        self.independent_holdout
            .validate()
            .map_err(|_| field_out_of_sync("independent_holdout"))?;

        if self.selected_asteroid_source != expected.selected_asteroid_source {
            return Err(field_out_of_sync("selected_asteroid_source"));
        }
        self.selected_asteroid_source
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source"))?;

        if self.selected_asteroid_source_windows != expected.selected_asteroid_source_windows {
            return Err(field_out_of_sync("selected_asteroid_source_windows"));
        }
        self.selected_asteroid_source_windows
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_windows"))?;

        if self.selected_asteroid_source_request_corpus
            != expected.selected_asteroid_source_request_corpus
        {
            return Err(field_out_of_sync("selected_asteroid_source_request_corpus"));
        }
        self.selected_asteroid_source_request_corpus
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_request_corpus"))?;

        if self.selected_asteroid_source_request_corpus_equatorial
            != expected.selected_asteroid_source_request_corpus_equatorial
        {
            return Err(field_out_of_sync(
                "selected_asteroid_source_request_corpus_equatorial",
            ));
        }
        self.selected_asteroid_source_request_corpus_equatorial
            .validate()
            .map_err(|_| field_out_of_sync("selected_asteroid_source_request_corpus_equatorial"))?;

        if self.production_generation_boundary_source
            != expected.production_generation_boundary_source
        {
            return Err(field_out_of_sync("production_generation_boundary_source"));
        }
        self.production_generation_boundary_source
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_boundary_source"))?;

        if self.production_generation_body_class_coverage
            != expected.production_generation_body_class_coverage
        {
            return Err(field_out_of_sync(
                "production_generation_body_class_coverage",
            ));
        }
        self.production_generation_body_class_coverage
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_body_class_coverage"))?;

        if self.production_generation_source != expected.production_generation_source {
            return Err(field_out_of_sync("production_generation_source"));
        }
        self.production_generation_source
            .validate()
            .map_err(|_| field_out_of_sync("production_generation_source"))?;

        Ok(())
    }

    /// Returns the validated phase-2 corpus alignment posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactPhase2CorpusAlignmentSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactPhase2CorpusAlignmentSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured phase-2 corpus alignment posture used to keep the packaged-artifact threshold policy aligned with the current corpus.
pub fn packaged_artifact_phase2_corpus_alignment_summary_details(
) -> Option<PackagedArtifactPhase2CorpusAlignmentSummary> {
    Some(PackagedArtifactPhase2CorpusAlignmentSummary {
        reference_snapshot_source: pleiades_jpl::reference_snapshot_source_summary(),
        reference_snapshot: pleiades_jpl::reference_snapshot_body_class_coverage_summary()?,
        reference_snapshot_exact_j2000:
            pleiades_jpl::reference_snapshot_exact_j2000_evidence_summary()?,
        comparison_snapshot_source: pleiades_jpl::comparison_snapshot_source_summary(),
        comparison_snapshot: comparison_snapshot_body_class_coverage_summary()?,
        independent_holdout_source: pleiades_jpl::independent_holdout_source_summary(),
        independent_holdout: independent_holdout_snapshot_body_class_coverage_summary()?,
        selected_asteroid_source: pleiades_jpl::selected_asteroid_source_evidence_summary()?,
        selected_asteroid_source_windows: pleiades_jpl::selected_asteroid_source_window_summary()?,
        selected_asteroid_source_request_corpus: selected_asteroid_source_request_corpus_summary(
            CoordinateFrame::Ecliptic,
        )?,
        selected_asteroid_source_request_corpus_equatorial:
            selected_asteroid_source_request_corpus_summary(CoordinateFrame::Equatorial)?,
        production_generation_boundary_source:
            pleiades_jpl::production_generation_boundary_source_summary(),
        production_generation_body_class_coverage:
            pleiades_jpl::production_generation_snapshot_body_class_coverage_summary()?,
        production_generation_source: production_generation_source_summary(),
    })
}

/// Structured target-threshold posture for the packaged artifact generator.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactTargetThresholdSummary {
    /// Stable identifier for the release profile that the thresholds apply to.
    pub profile_id: &'static str,
    /// Current release posture for the production thresholds.
    pub state: PackagedArtifactTargetThresholdState,
    /// Body-class scopes covered by the current threshold policy.
    pub scopes: &'static [&'static str],
    /// Measured fit envelope captured for the current packaged artifact posture.
    pub fit_envelope: PackagedArtifactFitEnvelopeSummary,
    /// Body-class-specific fit envelopes captured for the current packaged artifact posture.
    pub scope_envelopes: PackagedArtifactTargetThresholdScopeEnvelopesSummary,
    /// Phase-2 corpus evidence that keeps the threshold posture aligned with the current corpus.
    pub phase2_corpus_alignment: PackagedArtifactPhase2CorpusAlignmentSummary,
}

/// Validation error for a packaged-artifact target-threshold summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactTargetThresholdSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactTargetThresholdSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact target-threshold summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactTargetThresholdSummaryValidationError {}

impl PackagedArtifactTargetThresholdSummary {
    /// Returns the target-threshold posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "profile id={}; target thresholds: {}; scopes={}; {}; scope envelopes={}; phase 2 corpus alignment={}",
            self.profile_id,
            self.state,
            self.scopes.join(", "),
            self.fit_envelope.summary_line(),
            join_display(&self.scope_envelopes.scope_envelopes),
            self.phase2_corpus_alignment.summary_line(),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current packaged-artifact posture.
    pub fn validate(&self) -> Result<(), PackagedArtifactTargetThresholdSummaryValidationError> {
        if self.profile_id != ARTIFACT_PROFILE_ID {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "profile_id",
                },
            );
        }
        self.state.validate_production_ready().map_err(|_| {
            PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync { field: "state" }
        })?;
        if self.state != PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "state",
                },
            );
        }
        if self.scopes != PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "scopes",
                },
            );
        }

        let expected_fit_envelope = packaged_artifact_fit_envelope_summary_details();
        if self.fit_envelope != expected_fit_envelope {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "fit_envelope",
                },
            );
        }
        if self.state.is_production_ready() {
            let thresholds = packaged_artifact_fit_threshold_summary_details();
            self.fit_envelope
                .validate_against_thresholds(&thresholds)
                .map_err(|_| {
                    PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                        field: "fit_envelope",
                    }
                })?;
        }
        let expected_scope_envelopes =
            packaged_artifact_target_threshold_scope_envelopes_summary_details();
        if self.scope_envelopes != expected_scope_envelopes {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "scope_envelopes",
                },
            );
        }
        self.scope_envelopes.validate().map_err(|_| {
            PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                field: "scope_envelopes",
            }
        })?;

        let expected_phase2_corpus_alignment =
            packaged_artifact_phase2_corpus_alignment_summary_details().ok_or(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            )?;
        self.phase2_corpus_alignment
            .validate()
            .map_err(|error| match error {
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field,
                } => PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: phase2_corpus_alignment_validation_field_path(field),
                },
            })?;
        if self.phase2_corpus_alignment != expected_phase2_corpus_alignment {
            return Err(
                PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        }

        let thresholds = packaged_artifact_fit_threshold_summary_details();
        for scope_envelope in &self.scope_envelopes.scope_envelopes {
            scope_envelope
                .fit_envelope
                .validate_against_thresholds(&thresholds)
                .map_err(|_| {
                    PackagedArtifactTargetThresholdSummaryValidationError::FieldOutOfSync {
                        field: "scope_envelopes",
                    }
                })?;
        }

        Ok(())
    }

    /// Returns the validated target-threshold posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactTargetThresholdSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactTargetThresholdSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact target-threshold summary record.
pub fn packaged_artifact_target_threshold_summary_details() -> PackagedArtifactTargetThresholdSummary
{
    let summary = PackagedArtifactTargetThresholdSummary {
        profile_id: ARTIFACT_PROFILE_ID,
        state: PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE,
        scopes: PACKAGED_ARTIFACT_TARGET_THRESHOLD_SCOPES,
        fit_envelope: packaged_artifact_fit_envelope_summary_details(),
        scope_envelopes: packaged_artifact_target_threshold_scope_envelopes_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}

/// Structured sync summary for the packaged-artifact source-fit and hold-out checks.
#[derive(Clone, Debug, PartialEq)]
pub struct PackagedArtifactSourceFitHoldoutSyncSummary {
    /// Calibrated fit thresholds used by the current packaged-artifact posture.
    pub fit_thresholds: PackagedArtifactFitThresholdSummary,
    /// Release-threshold posture that keeps the phase-2 corpus alignment synchronized.
    pub target_thresholds: PackagedArtifactTargetThresholdSummary,
    /// Phase-2 corpus evidence that anchors the threshold posture to current source coverage.
    pub phase2_corpus_alignment: PackagedArtifactPhase2CorpusAlignmentSummary,
}

/// Validation error for a packaged-artifact source-fit and hold-out sync summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {
    /// A summary field is out of sync with the current packaged-artifact posture.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the packaged artifact source-fit and hold-out sync summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for PackagedArtifactSourceFitHoldoutSyncSummaryValidationError {}

impl PackagedArtifactSourceFitHoldoutSyncSummary {
    /// Returns the source-fit and hold-out sync posture as a compact human-readable line.
    pub fn summary_line(&self) -> String {
        format!(
            "source-fit and hold-out sync: fit thresholds={}; target thresholds={}; phase 2 corpus alignment={}",
            self.fit_thresholds.summary_line(),
            self.target_thresholds.summary_line(),
            self.phase2_corpus_alignment.summary_line(),
        )
    }

    /// Returns `Ok(())` when the sync summary still matches the current packaged-artifact posture.
    pub fn validate(
        &self,
    ) -> Result<(), PackagedArtifactSourceFitHoldoutSyncSummaryValidationError> {
        let expected_fit_thresholds = packaged_artifact_fit_threshold_summary_details();
        if self.fit_thresholds != expected_fit_thresholds {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "fit_thresholds",
                },
            );
        }
        self.fit_thresholds.validate().map_err(|_| {
            PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                field: "fit_thresholds",
            }
        })?;

        let expected_target_thresholds = packaged_artifact_target_threshold_summary_details();
        if self.target_thresholds != expected_target_thresholds {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "target_thresholds",
                },
            );
        }
        self.target_thresholds.validate().map_err(|_| {
            PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                field: "target_thresholds",
            }
        })?;

        let expected_phase2_corpus_alignment =
            packaged_artifact_phase2_corpus_alignment_summary_details().ok_or(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            )?;
        self.phase2_corpus_alignment
            .validate()
            .map_err(|error| match error {
                PackagedArtifactPhase2CorpusAlignmentSummaryValidationError::FieldOutOfSync {
                    field,
                } => PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: phase2_corpus_alignment_validation_field_path(field),
                },
            })?;
        if self.phase2_corpus_alignment != expected_phase2_corpus_alignment {
            return Err(
                PackagedArtifactSourceFitHoldoutSyncSummaryValidationError::FieldOutOfSync {
                    field: "phase2_corpus_alignment",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated source-fit and hold-out sync posture as a compact human-readable line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, PackagedArtifactSourceFitHoldoutSyncSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for PackagedArtifactSourceFitHoldoutSyncSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current packaged-artifact source-fit and hold-out sync summary record.
pub fn packaged_artifact_source_fit_holdout_sync_summary_details(
) -> PackagedArtifactSourceFitHoldoutSyncSummary {
    let summary = PackagedArtifactSourceFitHoldoutSyncSummary {
        fit_thresholds: packaged_artifact_fit_threshold_summary_details(),
        target_thresholds: packaged_artifact_target_threshold_summary_details(),
        phase2_corpus_alignment: packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available"),
    };
    debug_assert!(summary.validate().is_ok());
    summary
}
