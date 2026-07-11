//! Compression report/summary prose relocated from `pleiades-compression`
//! (report-surface relocation program, Slice B). Rendering only — the
//! functional crate keeps the structured data and its inherent methods.
//!
//! `ArtifactResidualBodyCoverageSummary::validated_summary_line_with_body_count`
//! could not be deleted from `pleiades-compression`: `pleiades-data`
//! (`coverage::generation::packaged_artifact_generation_residual_bodies_summary_for_report`)
//! and `pleiades-compression`'s own retained tests call it directly, and
//! `pleiades-compression` cannot depend on `pleiades-validate`. This module
//! carries a byte-identical free-function copy for `pleiades-validate`'s sole
//! runtime call site
//! (`render::summary::artifact::validate_packaged_artifact_generation_residual_bodies_summary`),
//! reusing the public `pleiades_compression::join_display` helper (rather than
//! duplicating it) so the separator can never drift.

// Verbatim relocation of a report-prose surface: some renderers are exercised
// only by this module's own tests or have no current in-crate caller.
#![allow(dead_code)]

use pleiades_compression::{
    join_display, ArtifactResidualBodyCoverageSummary, CompressedArtifact, CompressionError,
};

/// Returns the residual-body coverage as a compact human-readable line.
///
/// Byte-identical copy of the private rendering body inside
/// `ArtifactResidualBodyCoverageSummary::summary_line`
/// (`pleiades-compression/src/format.rs`).
fn residual_body_coverage_summary_line(summary: &ArtifactResidualBodyCoverageSummary) -> String {
    match summary.bodies.as_slice() {
        [] => "residual bodies: none".to_string(),
        bodies => format!("residual bodies: {}", join_display(bodies)),
    }
}

/// Returns the "N bundled body/bodies" suffix used by the residual-body
/// coverage line. Byte-identical copy of the private
/// `ArtifactResidualBodyCoverageSummary::body_count_suffix` helper.
fn residual_body_count_suffix(summary: &ArtifactResidualBodyCoverageSummary) -> String {
    match summary.body_count {
        1 => "1 bundled body".to_string(),
        count => format!("{count} bundled bodies"),
    }
}

/// Returns the residual-body coverage line after validating the artifact.
///
/// Byte-identical reimplementation of
/// `ArtifactResidualBodyCoverageSummary::validated_summary_line_with_body_count`
/// for `pleiades-validate`'s sole runtime call site.
pub(crate) fn validated_residual_body_coverage_summary_line_with_body_count(
    summary: &ArtifactResidualBodyCoverageSummary,
    artifact: &CompressedArtifact,
) -> Result<String, CompressionError> {
    summary.validate(artifact)?;
    Ok(format!(
        "{}; applies to {}",
        residual_body_coverage_summary_line(summary),
        residual_body_count_suffix(summary),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_compression::{ArtifactHeader, BodyArtifact, ChannelKind, PolynomialChannel, Segment};
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};

    /// Byte-identity guard against
    /// `ArtifactResidualBodyCoverageSummary::{summary_line, validated_summary_line_with_body_count}`
    /// (`pleiades-compression/src/format.rs`), the surface this module
    /// reimplements for the moved validate call site. Mirrors the fixture and
    /// assertions of `pleiades-compression`'s own retained
    /// `artifact_residual_body_coverage_summary_tracks_artifact_residual_bodies`
    /// test (`pleiades-compression/src/tests.rs`), which stays in
    /// `pleiades-compression` because it also exercises the struct's
    /// `validate`/drift-rejection behavior that this module does not own.
    #[test]
    fn residual_body_coverage_summary_line_reports_body_list_and_count() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("residual coverage demo", "unit test residual coverage"),
            vec![
                BodyArtifact::new(
                    CelestialBody::Sun,
                    vec![Segment::new(
                        Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                        Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                        vec![PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0)],
                    )],
                ),
                BodyArtifact::new(
                    CelestialBody::Moon,
                    vec![Segment::with_residual_channels(
                        Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                        Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                        vec![PolynomialChannel::linear(ChannelKind::Longitude, 9, 20.0, 21.0)],
                        vec![PolynomialChannel::linear(ChannelKind::Longitude, 9, 0.1, 0.2)],
                    )],
                ),
            ],
        );

        let summary = artifact.residual_body_coverage_summary();
        assert_eq!(summary.body_count, 1);
        assert_eq!(summary.bodies, vec![CelestialBody::Moon]);

        assert_eq!(
            residual_body_coverage_summary_line(&summary),
            "residual bodies: Moon"
        );
        assert_eq!(
            validated_residual_body_coverage_summary_line_with_body_count(&summary, &artifact),
            Ok("residual bodies: Moon; applies to 1 bundled body".to_string())
        );

        // Drift guard: cross-check against the still-live
        // `pleiades-compression` methods this module reimplements, so a
        // future change to either side's rendering fails this test instead
        // of silently drifting apart.
        assert_eq!(
            residual_body_coverage_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            validated_residual_body_coverage_summary_line_with_body_count(&summary, &artifact),
            summary.validated_summary_line_with_body_count(&artifact)
        );
    }

    #[test]
    fn residual_body_coverage_summary_line_reports_none_when_empty() {
        let artifact = CompressedArtifact::new(
            ArtifactHeader::new("no residuals demo", "unit test no residual coverage"),
            vec![BodyArtifact::new(
                CelestialBody::Sun,
                vec![Segment::new(
                    Instant::new(JulianDay::from_days(0.0), TimeScale::Tt),
                    Instant::new(JulianDay::from_days(1.0), TimeScale::Tt),
                    vec![PolynomialChannel::linear(ChannelKind::Longitude, 9, 10.0, 11.0)],
                )],
            )],
        );

        let summary = artifact.residual_body_coverage_summary();
        assert_eq!(
            residual_body_coverage_summary_line(&summary),
            "residual bodies: none"
        );
        assert_eq!(
            validated_residual_body_coverage_summary_line_with_body_count(&summary, &artifact),
            Ok("residual bodies: none; applies to 0 bundled bodies".to_string())
        );

        // Drift guard: cross-check against the still-live
        // `pleiades-compression` methods this module reimplements, so a
        // future change to either side's rendering fails this test instead
        // of silently drifting apart.
        assert_eq!(
            residual_body_coverage_summary_line(&summary),
            summary.summary_line()
        );
        assert_eq!(
            validated_residual_body_coverage_summary_line_with_body_count(&summary, &artifact),
            summary.validated_summary_line_with_body_count(&artifact)
        );
    }
}
