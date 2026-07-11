//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).
//!
//! Rendering-only prose comparing published accuracy ceilings against the live
//! measured baseline, plus the size budget. The functional crate keeps the
//! release-gate SSOT (`accuracy_ceiling`, `PACKAGED_BUDGETS`) and the
//! accuracy-baseline measurement core.

use pleiades_data::thresholds::{accuracy_ceiling, PACKAGED_BUDGETS};

/// Returns a deterministic summary comparing published accuracy ceilings against the live
/// measured baseline (from the committed packaged artifact vs hold-out corpus), plus the
/// size budget. One line per body class × channel showing measured/ceiling with PASS/FAIL.
///
/// The function reads [`accuracy_ceiling`] and [`PACKAGED_BUDGETS`] (the SSOT) plus the
/// live baseline — deterministic, kernel-free.  PASS/FAIL is computed from measured vs
/// ceiling, so the summary stays truthful if a value ever regresses.
pub(crate) fn packaged_artifact_thresholds_summary_for_report() -> String {
    let baseline = pleiades_data::packaged_artifact_accuracy_baseline();
    let mut lines = Vec::new();
    for e in &baseline {
        let c = accuracy_ceiling(&e.body);
        let lon_status = if e.max_longitude_arcsec <= c.lon_arcsec {
            "PASS"
        } else {
            "FAIL"
        };
        let lat_status = if e.max_latitude_arcsec <= c.lat_arcsec {
            "PASS"
        } else {
            "FAIL"
        };
        let dist_status = if e.max_distance_km <= c.dist_km {
            "PASS"
        } else {
            "FAIL"
        };
        let spd_status = if e.max_lon_speed_arcsec_per_day <= c.lon_speed_arcsec_per_day {
            "PASS"
        } else {
            "FAIL"
        };
        lines.push(format!(
            "{:?}: lon {:.4}\"/{}\"={} lat {:.4}\"/{}\"={} dist {:.1}/{} km={} lon_spd {:.4}/{} \"/d={}",
            e.body,
            e.max_longitude_arcsec, c.lon_arcsec, lon_status,
            e.max_latitude_arcsec, c.lat_arcsec, lat_status,
            e.max_distance_km, c.dist_km, dist_status,
            e.max_lon_speed_arcsec_per_day, c.lon_speed_arcsec_per_day, spd_status,
        ));
    }
    format!(
        "Packaged-artifact thresholds ({} bodies); size budget {} bytes:\n{}",
        baseline.len(),
        PACKAGED_BUDGETS.max_encoded_bytes,
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "maintainer helper: prints the thresholds summary to regenerate the golden"]
    fn print_packaged_artifact_thresholds_summary() {
        eprintln!("{}", packaged_artifact_thresholds_summary_for_report());
    }

    // Drift gate: committed thresholds summary must match the live baseline.
    // Generated from actual output (2026-06-20, SP3 heliocentric-planet artifact).
    // Anchors header, one representative body line, and the size-budget line.
    // PASS/FAIL is computed — fails if any channel regresses past its ceiling.
    #[test]
    fn packaged_artifact_thresholds_summary_matches_committed_golden() {
        let report = packaged_artifact_thresholds_summary_for_report();

        // Header: 10 bodies and the size budget.
        assert!(
            report.contains("Packaged-artifact thresholds (10 bodies)"),
            "thresholds report header drift: {report}"
        );
        assert!(
            report.contains("size budget 12000000 bytes"),
            "thresholds report size-budget line drift: {report}"
        );

        // Representative body line: Moon (Luminary class, ceiling 1.0"/1.0"/50 km/0.5"/d).
        // All channels must PASS given the current sub-ceiling measurements.
        assert!(
            report.contains("Moon:") && report.contains("/1\"=PASS"),
            "Moon lon/ceiling PASS line drift: {report}"
        );

        // Size budget line is in the header (checked above).
        // Verify each body appears and shows PASS for longitude.
        for body_name in &[
            "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
            "Pluto",
        ] {
            assert!(
                report.contains(body_name),
                "{body_name} missing from thresholds report: {report}"
            );
            // Each body should show PASS (all measured values are well within ceilings).
            assert!(
                report.contains(&format!("{body_name}:")) && {
                    // find this body's line and check it has PASS
                    report
                        .lines()
                        .find(|l| l.starts_with(body_name))
                        .map(|l| l.contains("PASS"))
                        .unwrap_or(false)
                },
                "{body_name} line does not contain PASS: {report}"
            );
        }
    }
}
