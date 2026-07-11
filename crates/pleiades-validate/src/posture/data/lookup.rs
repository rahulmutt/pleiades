//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).
//!
//! Rendering-only prose for packaged-data lookup/request posture. The functional
//! crate keeps the structured `Packaged*Summary` records, the `&'static str`
//! self-description accessors, and all inherent methods.

use pleiades_data::{
    packaged_artifact_access_summary_details, packaged_artifact_storage_summary_details,
    packaged_frame_treatment_summary_details, packaged_lookup_epoch_policy_summary_details,
    packaged_mixed_frame_batch_parity_summary, packaged_mixed_tt_tdb_batch_parity_summary,
    packaged_request_policy_summary_details, PackagedTimeScaleBatchParitySummary,
};

/// Returns the current packaged-data lookup-epoch policy summary after validating the structured posture.
pub(crate) fn packaged_lookup_epoch_policy_summary_for_report() -> String {
    let summary = packaged_lookup_epoch_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged lookup epoch policy: unavailable ({error})"),
    }
}

/// Returns the current packaged-data request policy summary after validating the structured posture.
pub(crate) fn packaged_request_policy_summary_for_report() -> String {
    let summary = packaged_request_policy_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged request policy: unavailable ({error})"),
    }
}

/// Returns the packaged-artifact frame-treatment summary for report rendering.
pub(crate) fn packaged_frame_treatment_summary_for_report() -> String {
    let summary = packaged_frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("Packaged frame treatment unavailable ({error})"),
    }
}

/// Returns the packaged-artifact storage/reconstruction summary for reporting.
pub(crate) fn packaged_artifact_storage_summary_for_report() -> String {
    let summary = packaged_artifact_storage_summary_details();
    match summary.validate() {
        Ok(()) => summary.to_string(),
        Err(error) => format!("Packaged artifact storage/reconstruction: unavailable ({error})"),
    }
}

/// Returns the packaged-artifact access summary for reporting.
pub(crate) fn packaged_artifact_access_summary_for_report() -> String {
    let summary = packaged_artifact_access_summary_details();
    match summary.validated_summary_line() {
        Ok(rendered) => rendered.to_string(),
        Err(error) => format!("Packaged artifact access: unavailable ({error})"),
    }
}

/// Returns the packaged frame-parity summary.
///
/// Reconstructed from the retained structured mixed-frame batch-parity summary
/// so the moved copy no longer depends on the (Task 6 `pub(crate)`-demoted)
/// `pleiades-data` renderer while remaining byte-identical to its output.
pub(crate) fn packaged_frame_parity_summary_for_report() -> String {
    packaged_mixed_frame_batch_parity_summary()
        .as_ref()
        .map(|summary| match summary.validated_summary_line() {
            Ok(line) => line,
            Err(error) => format!("Packaged mixed frame batch parity: unavailable ({error})"),
        })
        .unwrap_or_else(|| "Packaged mixed frame batch parity: unavailable".to_string())
}

fn format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report(
    summary: &PackagedTimeScaleBatchParitySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!("Packaged mixed TT/TDB batch parity: unavailable ({error})"),
    }
}

/// Returns the packaged mixed TT/TDB batch-parity summary.
pub(crate) fn packaged_mixed_tt_tdb_batch_parity_summary_for_report() -> String {
    packaged_mixed_tt_tdb_batch_parity_summary()
        .as_ref()
        .map(format_validated_packaged_mixed_tt_tdb_batch_parity_summary_for_report)
        .unwrap_or_else(|| "Packaged mixed TT/TDB batch parity: unavailable".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_epoch_policy_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/lookup.rs`.
        assert_eq!(
            packaged_lookup_epoch_policy_summary_for_report(),
            packaged_lookup_epoch_policy_summary_details().summary_line()
        );
    }

    #[test]
    fn request_policy_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/lookup.rs`.
        assert_eq!(
            packaged_request_policy_summary_for_report(),
            packaged_request_policy_summary_details().summary_line()
        );
    }

    #[test]
    fn frame_treatment_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/coverage.rs`.
        assert_eq!(
            packaged_frame_treatment_summary_for_report(),
            packaged_frame_treatment_summary_details().to_string()
        );
    }

    #[test]
    fn artifact_storage_report_matches_retained_details() {
        assert_eq!(
            packaged_artifact_storage_summary_for_report(),
            packaged_artifact_storage_summary_details().to_string()
        );
    }

    #[test]
    fn artifact_access_report_matches_retained_details() {
        // Relocated from `pleiades-data` `tests/coverage.rs`.
        assert_eq!(
            packaged_artifact_access_summary_for_report(),
            packaged_artifact_access_summary_details().to_string()
        );
    }

    #[test]
    fn frame_parity_summary_for_report_pins_current_string() {
        // Equality guard closing the drift vector between this reconstruction and
        // the retained `pleiades-data` mixed-frame batch-parity renderer.
        assert_eq!(
            packaged_frame_parity_summary_for_report(),
            "Packaged mixed frame batch parity: 11 requests across 11 bodies, ecliptic requests=6, equatorial requests=5; quality counts: Exact=0, Interpolated=11, Approximate=0, Unknown=0; order=preserved, single-query parity=preserved"
        );
    }

    #[test]
    fn mixed_tt_tdb_batch_parity_summary_for_report_pins_current_string() {
        assert_eq!(
            packaged_mixed_tt_tdb_batch_parity_summary_for_report(),
            "Packaged mixed TT/TDB batch parity: 11 requests across 11 bodies, TT requests=6, TDB requests=5; quality counts: Exact=0, Interpolated=11, Approximate=0, Unknown=0; order=preserved, single-query parity=preserved"
        );
    }
}
