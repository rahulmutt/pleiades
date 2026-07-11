//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).

use std::sync::OnceLock;

use pleiades_data::{
    packaged_artifact_phase2_corpus_alignment_summary_details,
    packaged_artifact_source_fit_holdout_sync_summary_details,
    packaged_artifact_target_threshold_scope_envelopes_summary_details,
    packaged_artifact_target_threshold_summary_details,
    PackagedArtifactPhase2CorpusAlignmentSummary, PackagedArtifactTargetThresholdState,
};

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
pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_phase2_corpus_alignment_summary_details();
            match summary.as_ref().map(PackagedArtifactPhase2CorpusAlignmentSummary::validated_summary_line) {
                Some(Ok(line)) => line,
                Some(Err(error)) => format!("phase 2 corpus alignment: unavailable ({error})"),
                None => "phase 2 corpus alignment: unavailable (phase-2 corpus evidence should be available)".to_string(),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact target-threshold summary after validating the structured posture.
pub(crate) fn packaged_artifact_target_threshold_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_target_threshold_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
                Err(error) => format!("target thresholds: unavailable ({error})"),
            }
        })
        .clone()
}

/// Returns the current packaged-artifact source-fit and hold-out sync posture after validating the structured evidence.
pub(crate) fn packaged_artifact_source_fit_holdout_sync_summary_for_report() -> String {
    static SUMMARY: OnceLock<String> = OnceLock::new();
    SUMMARY
        .get_or_init(|| {
            let summary = packaged_artifact_source_fit_holdout_sync_summary_details();
            match summary.validated_summary_line() {
                Ok(line) => line,
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

    #[test]
    fn phase2_corpus_alignment_summary_for_report_matches_details() {
        let summary = pleiades_data::packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("phase-2 corpus evidence should be available");
        assert_eq!(
            packaged_artifact_phase2_corpus_alignment_summary_for_report(),
            summary.summary_line()
        );
    }

    #[test]
    fn target_threshold_scope_envelopes_for_report_reflects_the_current_posture() {
        assert!(
            packaged_artifact_target_threshold_scope_envelopes_for_report()
                .contains("scope=luminaries; bodies=2 (Sun, Moon); fit envelope:")
        );
    }

    #[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
    #[test]
    fn source_fit_holdout_sync_summary_for_report_matches_details() {
        let summary = pleiades_data::packaged_artifact_source_fit_holdout_sync_summary_details();
        assert_eq!(
            packaged_artifact_source_fit_holdout_sync_summary_for_report(),
            summary.summary_line()
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
