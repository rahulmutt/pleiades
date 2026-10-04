//! The per-process memoized renderers return exactly what their uncached
//! bodies render (FU-23: release-bundle verification re-renders them on every
//! call, so they are memoized; these tests pin that the memo is not stale).

use super::*;

#[test]
fn memoized_release_notes_summary_matches_the_uncached_rendering() {
    let uncached = crate::release::notes::render_release_notes_summary_text_uncached();
    assert_eq!(render_release_notes_summary_text(), uncached);
    assert_eq!(render_release_notes_summary_text(), uncached);
}

#[test]
fn memoized_backend_matrix_summary_matches_the_uncached_rendering() {
    let uncached = crate::render::summary::render_backend_matrix_summary_text_uncached();
    assert_eq!(render_backend_matrix_summary(), uncached);
    assert_eq!(render_backend_matrix_summary(), uncached);
}

#[test]
fn memoized_target_threshold_summary_matches_the_uncached_rendering() {
    let uncached =
        crate::render::summary::validated_packaged_artifact_target_threshold_summary_uncached();
    assert_eq!(
        validated_packaged_artifact_target_threshold_summary_for_report(),
        uncached
    );
    assert_eq!(
        validated_packaged_artifact_target_threshold_summary_for_report(),
        uncached
    );
}
