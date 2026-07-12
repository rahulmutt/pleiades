//! Relocated from `pleiades-data` (report-surface relocation program, Slice C).
//!
//! Rendering-only prose: a deterministic one-line-per-body summary of the
//! packaged-artifact accuracy baseline. The functional crate keeps the
//! measurement core (`accuracy_baseline_against`, `packaged_artifact_accuracy_baseline`,
//! `BodyChannelError::summary_line`) and the Eros self-consistency check.

use pleiades_data::packaged_artifact_accuracy_baseline;

/// Returns a deterministic one-line-per-body summary of the packaged-artifact accuracy baseline.
///
/// The string is recomputed on every call; the drift-gate test
/// `packaged_artifact_baseline_summary_matches_committed_golden` compares it
/// against the committed golden buckets.
pub(crate) fn packaged_artifact_accuracy_baseline_summary_for_report() -> String {
    let errors = packaged_artifact_accuracy_baseline();
    if errors.is_empty() {
        return "Packaged-artifact accuracy baseline: no hold-out rows matched".to_string();
    }
    let lines: Vec<String> = errors.iter().map(|e| e.summary_line()).collect();
    format!(
        "Packaged-artifact accuracy baseline ({} bodies):\n{}",
        errors.len(),
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "maintainer helper: prints the accuracy baseline summary to regenerate the golden"]
    fn print_packaged_artifact_baseline_summary() {
        eprintln!(
            "{}",
            packaged_artifact_accuracy_baseline_summary_for_report()
        );
    }

    // Drift gate: the committed per-body summary must match the live baseline.
    // Generated from actual output (2026-06-20, SP3 heliocentric-planet artifact);
    // fails if errors silently go all-zero or if artifact/hold-out changes shift
    // any body's error bucket. SP2 outer-planet errors are sub-arcsec after the
    // heliocentric reframe; compare with SP1 (pre-reframe) goldens in git history.
    // SP3 (Task 11): golden now anchors speed channels (first 3 significant digits)
    // in addition to the position channels. Speed vacuity: if a body's speed rows
    // were all skipped, max_lon_speed stays 0.0000 and the non-vacuity check below
    // catches it via the "strictly-positive lon-speed" guard.
    #[test]
    fn packaged_artifact_baseline_summary_matches_committed_golden() {
        let report = packaged_artifact_accuracy_baseline_summary_for_report();

        // Header: 10 bodies
        assert!(
            report.contains("Packaged-artifact accuracy baseline (10 bodies)"),
            "baseline report header drift: {report}"
        );

        // All bodies: sub-arcsec — anchored to first 3 significant digits.
        // SP2: outer planets now stored heliocentrically, so all errors are sub-arcsec.
        assert!(
            report.contains("Sun: n=50 max_lon=0.000"),
            "Sun max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Moon: n=50 max_lon=0.000"),
            "Moon max_lon bucket drift (expected ~0.0001\"): {report}"
        );
        assert!(
            report.contains("Mercury: n=50 max_lon=0.000"),
            "Mercury max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Venus: n=50 max_lon=0.001"),
            "Venus max_lon bucket drift (expected ~0.0011\"): {report}"
        );
        assert!(
            report.contains("Mars: n=50 max_lon=0.000"),
            "Mars max_lon bucket drift (expected ~0.0005\"): {report}"
        );
        // Outer planets: SP2 heliocentric-reframe — all sub-arcsec, non-zero.
        assert!(
            report.contains("Jupiter: n=50 max_lon=0.000"),
            "Jupiter max_lon bucket drift (expected ~0.0004\"): {report}"
        );
        assert!(
            report.contains("Saturn: n=50 max_lon=0.000"),
            "Saturn max_lon bucket drift (expected ~0.0009\"): {report}"
        );
        assert!(
            report.contains("Uranus: n=50 max_lon=0.003"),
            "Uranus max_lon bucket drift (expected ~0.0036\"): {report}"
        );
        assert!(
            report.contains("Neptune: n=50 max_lon=0.002"),
            "Neptune max_lon bucket drift (expected ~0.0020\"): {report}"
        );
        assert!(
            report.contains("Pluto: n=50 max_lon=0.001"),
            "Pluto max_lon bucket drift (expected ~0.0018\"): {report}"
        );

        // SP3 Task 11: speed channel golden anchors (first 3 significant digits).
        // Moon has the highest lon/lat speed error (~0.030/~0.023 arcsec/day) due to
        // fast apparent motion; outer planets are an order of magnitude slower.
        // Radial speed errors are sub-1e-6 AU/day for all bodies; anchored to "0.000000".
        assert!(
            report.contains("Moon: n=50") && report.contains("max_lon_speed=0.023"),
            "Moon max_lon_speed bucket drift (expected ~0.0230 arcsec/day): {report}"
        );
        assert!(
            report.contains("Moon: n=50") && report.contains("max_lat_speed=0.025"),
            "Moon max_lat_speed bucket drift (expected ~0.0253 arcsec/day): {report}"
        );
        assert!(
            report.contains("Sun: n=50") && report.contains("max_lon_speed=0.001"),
            "Sun max_lon_speed bucket drift (expected ~0.0013 arcsec/day): {report}"
        );
        assert!(
            report.contains("Mercury: n=50") && report.contains("max_lon_speed=0.001"),
            "Mercury max_lon_speed bucket drift (expected ~0.0013 arcsec/day): {report}"
        );
        assert!(
            report.contains("Venus: n=50") && report.contains("max_lon_speed=0.001"),
            "Venus max_lon_speed bucket drift (expected ~0.0014 arcsec/day): {report}"
        );
        assert!(
            report.contains("Mars: n=50") && report.contains("max_lon_speed=0.001"),
            "Mars max_lon_speed bucket drift (expected ~0.0011 arcsec/day): {report}"
        );
        // Outer planets: lon_speed in the 0.000X range.
        assert!(
            report.contains("Jupiter: n=50") && report.contains("max_lon_speed=0.000"),
            "Jupiter max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Saturn: n=50") && report.contains("max_lon_speed=0.000"),
            "Saturn max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Uranus: n=50") && report.contains("max_lon_speed=0.000"),
            "Uranus max_lon_speed bucket drift (expected ~0.0007 arcsec/day): {report}"
        );
        assert!(
            report.contains("Neptune: n=50") && report.contains("max_lon_speed=0.000"),
            "Neptune max_lon_speed bucket drift (expected ~0.0002 arcsec/day): {report}"
        );
        assert!(
            report.contains("Pluto: n=50") && report.contains("max_lon_speed=0.000"),
            "Pluto max_lon_speed bucket drift (expected ~0.0005 arcsec/day): {report}"
        );
        // Radial speed: all bodies sub-1e-6 AU/day — anchor on "0.000000".
        for body_name in &[
            "Sun", "Moon", "Mercury", "Venus", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune",
            "Pluto",
        ] {
            assert!(
                report.contains(&format!("{body_name}: n=50"))
                    && report.contains("max_radial_speed=0.000000"),
                "{body_name} max_radial_speed not anchored at 0.000000 AU/day: {report}"
            );
        }
    }
}
