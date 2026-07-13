//! Relocated `pleiades-data` phase-2 corpus alignment renderer copied from
//! `pleiades-data::coverage::target::PackagedArtifactPhase2CorpusAlignmentSummary::summary_line`
//! (report-surface relocation program, Slice D, Task 12). Rendering only —
//! `pleiades-data` keeps the structured `PackagedArtifactPhase2CorpusAlignmentSummary`
//! struct, its `*_details()` constructor, and `validate()`; its own
//! `summary_line`/`Display` rendering stays in place until the Task 14
//! contract sweep.
//!
//! Each of the 13 jpl-evidence-struct fields is rewired to the local
//! validate posture/jpl free renderer for that field's jpl type (relocated
//! in Tasks 4-11), and `production_generation_boundary_source` is rewired to
//! the local `production_generation::format_production_generation_boundary_source_summary`
//! (relocated in Task 10b). `production_generation_source` is left calling
//! its inherent `.summary_line()` — that field's rendering is out of scope
//! for this relocation and is untouched by Task 14 as well.

use pleiades_data::PackagedArtifactPhase2CorpusAlignmentSummary;

use crate::posture::jpl::comparison::{
    comparison_snapshot_body_class_coverage_summary_line, comparison_snapshot_source_summary_line,
};
use crate::posture::jpl::holdout::{
    independent_holdout_snapshot_body_class_coverage_summary_summary_line,
    independent_holdout_source_summary_summary_line,
};
use crate::posture::jpl::production_generation::{
    format_production_generation_boundary_source_summary,
    production_generation_snapshot_body_class_coverage_summary_line,
};
use crate::posture::jpl::reference_snapshot::core::coverage::reference_snapshot_body_class_coverage_summary_line;
use crate::posture::jpl::reference_snapshot::core::evidence::reference_snapshot_exact_j2000_evidence_summary_line;
use crate::posture::jpl::reference_snapshot::core::general_b::reference_snapshot_source_summary_line;
use crate::posture::jpl::selected_asteroid::{
    selected_asteroid_source_request_corpus_summary_line, selected_asteroid_source_summary_line,
    selected_asteroid_source_window_summary_line,
};

/// Compact release-facing summary line for the packaged-artifact phase-2
/// corpus alignment posture. Verbatim copy of
/// `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line`
/// (`pleiades-data/src/coverage/target.rs:181-200`), with each of the 13
/// jpl-evidence-struct fields' `self.<field>.summary_line()` rewired to the
/// corresponding local validate free renderer, and
/// `pleiades_jpl::format_production_generation_boundary_source_summary(&self.production_generation_boundary_source)`
/// rewired to the local `format_production_generation_boundary_source_summary`.
/// `self.production_generation_source.summary_line()` is left as-is (out of
/// scope for this relocation).
pub(crate) fn packaged_artifact_phase2_corpus_alignment_summary_for_report(
    s: &PackagedArtifactPhase2CorpusAlignmentSummary,
) -> String {
    format!(
        "reference source={}; reference snapshot={}; reference exact J2000 evidence={}; comparison source={}; comparison snapshot={}; independent hold-out source={}; independent hold-out={}; selected asteroid source evidence={}; selected asteroid source windows={}; selected asteroid source request corpus={}; selected asteroid source request corpus equatorial={}; production generation boundary source={}; production generation body-class coverage={}; production generation source={}",
        reference_snapshot_source_summary_line(&s.reference_snapshot_source),
        reference_snapshot_body_class_coverage_summary_line(&s.reference_snapshot),
        reference_snapshot_exact_j2000_evidence_summary_line(&s.reference_snapshot_exact_j2000),
        comparison_snapshot_source_summary_line(&s.comparison_snapshot_source),
        comparison_snapshot_body_class_coverage_summary_line(&s.comparison_snapshot),
        independent_holdout_source_summary_summary_line(&s.independent_holdout_source),
        independent_holdout_snapshot_body_class_coverage_summary_summary_line(
            &s.independent_holdout
        ),
        selected_asteroid_source_summary_line(&s.selected_asteroid_source),
        selected_asteroid_source_window_summary_line(&s.selected_asteroid_source_windows),
        selected_asteroid_source_request_corpus_summary_line(
            &s.selected_asteroid_source_request_corpus
        ),
        selected_asteroid_source_request_corpus_summary_line(
            &s.selected_asteroid_source_request_corpus_equatorial
        ),
        format_production_generation_boundary_source_summary(
            &s.production_generation_boundary_source,
        ),
        production_generation_snapshot_body_class_coverage_summary_line(
            &s.production_generation_body_class_coverage
        ),
        s.production_generation_source.summary_line(),
    )
}

#[cfg(test)]
mod golden {
    use super::*;

    #[test]
    fn packaged_artifact_phase2_corpus_alignment_summary_matches_pleiades_data() {
        let details = pleiades_data::packaged_artifact_phase2_corpus_alignment_summary_details()
            .expect("packaged artifact phase-2 corpus alignment summary should exist");
        assert_eq!(
            details.summary_line(),
            packaged_artifact_phase2_corpus_alignment_summary_for_report(&details)
        );
    }
}
