//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use std::sync::OnceLock;

use pleiades_compression::join_display;
use pleiades_data::{
    packaged_artifact_phase2_corpus_alignment_summary_details,
    packaged_artifact_source_fit_holdout_sync_summary_details,
    packaged_artifact_target_threshold_scope_envelopes_summary_details,
    packaged_artifact_target_threshold_summary_details,
    PackagedArtifactSourceFitHoldoutSyncSummary, PackagedArtifactTargetThresholdState,
    PackagedArtifactTargetThresholdSummary,
};

use crate::posture::jpl::data_phase2_alignment::packaged_artifact_phase2_corpus_alignment_summary_for_report as jpl_phase2_corpus_alignment_summary_for_report;

/// Relocated `pleiades-data` target-threshold renderer copied verbatim
/// (self→s) from
/// `pleiades-data::coverage::target::PackagedArtifactTargetThresholdSummary::summary_line`
/// (`pleiades-data/src/coverage/target.rs:413-423`, report-surface
/// relocation program, Slice D, Task 14a2), with
/// `self.phase2_corpus_alignment.summary_line()` rewired to the Task 12
/// validate copy (`jpl_phase2_corpus_alignment_summary_for_report`). The
/// `fit_envelope`/`scope_envelopes` fields stay verbatim, calling their own
/// (non-jpl) pleiades-data inherent renderers, which are not part of this
/// relocation and are not scheduled for deletion.
pub(crate) fn packaged_artifact_target_threshold_summary_line(
    s: &PackagedArtifactTargetThresholdSummary,
) -> String {
    format!(
        "profile id={}; target thresholds: {}; scopes={}; {}; scope envelopes={}; phase 2 corpus alignment={}",
        s.profile_id,
        s.state,
        s.scopes.join(", "),
        s.fit_envelope.summary_line(),
        join_display(&s.scope_envelopes.scope_envelopes),
        jpl_phase2_corpus_alignment_summary_for_report(&s.phase2_corpus_alignment),
    )
}

/// Relocated `pleiades-data` source-fit/hold-out-sync renderer copied
/// verbatim (self→s) from
/// `pleiades-data::coverage::target::PackagedArtifactSourceFitHoldoutSyncSummary::summary_line`
/// (`pleiades-data/src/coverage/target.rs:587-594`, Slice D, Task 14a2), with
/// `self.phase2_corpus_alignment.summary_line()` rewired to the Task 12
/// validate copy. `self.target_thresholds.summary_line()` is additionally
/// rewired to the sibling `packaged_artifact_target_threshold_summary_line`
/// copy above — that field is itself a `PackagedArtifactTargetThresholdSummary`
/// whose own rendering transitively renders jpl evidence via its nested
/// `phase2_corpus_alignment` field, so it is in-chain for this relocation
/// the same way the direct `phase2_corpus_alignment` field is.
/// `self.fit_thresholds.summary_line()` stays verbatim (non-jpl, not
/// scheduled for deletion).
pub(crate) fn packaged_artifact_source_fit_holdout_sync_summary_line(
    s: &PackagedArtifactSourceFitHoldoutSyncSummary,
) -> String {
    format!(
        "source-fit and hold-out sync: fit thresholds={}; target thresholds={}; phase 2 corpus alignment={}",
        s.fit_thresholds.summary_line(),
        packaged_artifact_target_threshold_summary_line(&s.target_thresholds),
        jpl_phase2_corpus_alignment_summary_for_report(&s.phase2_corpus_alignment),
    )
}

// Relocated with the `packaged_artifact_target_threshold_state_for_report` renderer.
// The private `PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE` const stays in `pleiades-data`
// (retained items still reference it); it is mirrored here so the moved renderer body
// stays byte-identical.
const PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE: PackagedArtifactTargetThresholdState =
    PackagedArtifactTargetThresholdState::ProductionReady;

/// Returns the current packaged-artifact target-threshold state after validating the release posture.
pub(crate) fn packaged_artifact_target_threshold_state_for_report() -> String {
    match PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("target-threshold state: unavailable ({error})"),
    }
}

/// Returns the current packaged-artifact phase-2 corpus alignment posture after validating the structured evidence.
///
/// Repointed (Slice D, Task 14a2) from the pleiades-data inherent
/// `PackagedArtifactPhase2CorpusAlignmentSummary::validated_summary_line` to
/// `validate()` (kept in pleiades-data) followed by the Task 12 validate
/// render copy, so nothing here still calls the pleiades-data inherent
/// renderer that Task 14b will delete.
pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_phase2_corpus_alignment_summary_details();
            match summary {
                Some(summary) => match summary.validate() {
                    Ok(()) => jpl_phase2_corpus_alignment_summary_for_report(&summary),
                    Err(error) => format!("phase 2 corpus alignment: unavailable ({error})"),
                },
                None => "phase 2 corpus alignment: unavailable (phase-2 corpus evidence should be available)".to_string(),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact target-threshold summary after validating the structured posture.
///
/// Repointed (Slice D, Task 14a2) from the pleiades-data inherent
/// `PackagedArtifactTargetThresholdSummary::validated_summary_line` to
/// `validate()` (kept in pleiades-data) followed by the local
/// `packaged_artifact_target_threshold_summary_line` copy above.
pub(crate) fn packaged_artifact_target_threshold_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_target_threshold_summary_details();
            match summary.validate() {
                Ok(()) => packaged_artifact_target_threshold_summary_line(&summary),
                Err(error) => format!("target thresholds: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact source-fit and hold-out sync posture after validating the structured evidence.
///
/// Repointed (Slice D, Task 14a2) from the pleiades-data inherent
/// `PackagedArtifactSourceFitHoldoutSyncSummary::validated_summary_line` to
/// `validate()` (kept in pleiades-data) followed by the local
/// `packaged_artifact_source_fit_holdout_sync_summary_line` copy above.
pub(crate) fn packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_source_fit_holdout_sync_summary_details();
            match summary.validate() {
                Ok(()) => packaged_artifact_source_fit_holdout_sync_summary_line(&summary),
                Err(error) => format!("source-fit and hold-out sync: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact body-class target-threshold envelopes after validating the structured posture.
pub(crate) fn packaged_artifact_target_threshold_scope_envelopes_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_target_threshold_scope_envelopes_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!("scope envelopes: unavailable ({error})"),
            }
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase2_corpus_alignment_summary_for_report_is_validated() {
        let rendered = packaged_artifact_phase2_corpus_alignment_summary_for_report();
        assert!(rendered.contains("reference snapshot="));
        assert!(rendered.contains("reference exact J2000 evidence="));
        assert!(rendered.contains("comparison snapshot="));
        assert!(rendered.contains("independent hold-out="));
        assert!(rendered.contains(
            "production generation body-class coverage=Production generation body-class coverage:"
        ));
        assert!(rendered.contains("production generation source=Production generation source:"));
        assert!(rendered.contains("Reference snapshot body-class coverage"));
        assert!(rendered.contains("Reference snapshot exact J2000 evidence"));
        assert!(rendered.contains("Independent hold-out body-class coverage"));
        assert!(rendered.contains(
            "selected asteroid source request corpus=Selected asteroid source request corpus:"
        ));
        assert!(rendered.contains(
            "selected asteroid source request corpus equatorial=Selected asteroid source request corpus:"
        ));
        assert!(rendered.contains(
            "production generation boundary source=Production generation boundary overlay source:"
        ));
    }

    // Repointed (Slice D, Task 14a2): this test previously compared against
    // the pleiades-data inherent `summary.summary_line()`, which Task 14b
    // will delete. It now compares against the Task 12 validate render copy
    // instead (`jpl_phase2_corpus_alignment_summary_for_report`), which is
    // itself pinned byte-exact against the (still-present) pleiades-data
    // rendering by the golden test in `posture/jpl/data_phase2_alignment.rs`.
    #[test]
    fn phase2_corpus_alignment_summary_for_report_matches_details() {
        let summary = pleiades_data::packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available");
        assert_eq!(
            packaged_artifact_phase2_corpus_alignment_summary_for_report(),
            jpl_phase2_corpus_alignment_summary_for_report(&summary)
        );
    }

    #[test]
    fn target_threshold_scope_envelopes_for_report_reflects_the_current_posture() {
        assert!(
            packaged_artifact_target_threshold_scope_envelopes_for_report()
                .contains("scope=luminaries; bodies=2 (Sun, Moon); fit envelope:")
        );
    }

    // Repointed (Slice D, Task 14a2): previously compared against the
    // pleiades-data inherent `summary.summary_line()`, which Task 14b will
    // delete. Now compares against the local
    // `packaged_artifact_source_fit_holdout_sync_summary_line` copy above.
    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn source_fit_holdout_sync_summary_for_report_matches_details() {
        let summary = pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details();
        assert_eq!(
            packaged_artifact_source_fit_holdout_sync_summary_for_report(),
            packaged_artifact_source_fit_holdout_sync_summary_line(&summary)
        );
    }

    // Drift-guard: `PACKAGED_ARTIFACT_TARGET_THRESHOLD_STATE` is mirrored here from the
    // private const retained in `pleiades-data`. Pin the renderer's exact output so any
    // future divergence between the two copies fails this test.
    #[test]
    fn target_threshold_state_for_report_is_pinned() {
        assert_eq!(
            packaged_artifact_target_threshold_state_for_report(),
            "target-threshold state: production thresholds recorded"
        );
    }
}
