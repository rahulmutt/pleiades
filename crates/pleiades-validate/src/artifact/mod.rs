//! Bundled-artifact inspection: report types, construction/validation,
//! human-readable rendering, and error handling.
//!
//! This module was split out of a single `artifact.rs` file into a module tree
//! with no behavioral changes. The pieces are:
//!
//! - [`reports`]: the public report and benchmark data types and their
//!   self-contained methods, validation errors, and `Display` impls.
//! - [`inspection`]: construction and validation of the packaged
//!   [`ArtifactInspectionReport`], the decode/lookup benchmark builders, the
//!   comparison-corpus construction, and the boundary-continuity envelope.
//! - [`rendering`]: the compact summary text, per-body-class envelopes, and the
//!   `Display` rendering of the inspection report.
//! - [`error`]: the [`ArtifactInspectionError`] type and its conversions.
//!
//! Everything is re-exported here so callers continue to use `crate::artifact::*`
//! exactly as before.

mod error;
mod inspection;
mod rendering;
mod reports;

pub use error::*;
pub use inspection::*;
pub(crate) use rendering::*;
pub use reports::*;

#[cfg(test)]
mod tests {
    use super::{
        ArtifactBatchLookupBenchmarkReport, ArtifactBatchLookupBenchmarkReportValidationError,
        ArtifactBodyInspection, ArtifactBoundaryEnvelopeSummary,
        ArtifactBoundaryEnvelopeSummaryValidationError, ArtifactDecodeBenchmarkReport,
        ArtifactDecodeBenchmarkReportValidationError, ArtifactInspectionReport,
        ArtifactLookupBenchmarkReport, ArtifactLookupBenchmarkReportValidationError,
    };
    use pleiades_core::{CelestialBody, Instant, JulianDay, TimeScale};
    use pleiades_data::packaged_artifact;
    use std::time::Duration;

    fn instant(days: f64) -> Instant {
        Instant::new(JulianDay::from_days(days), TimeScale::Tt)
    }

    fn decode_benchmark_report() -> ArtifactDecodeBenchmarkReport {
        ArtifactDecodeBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            rounds: 2,
            sample_count: 3,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(5),
        }
    }

    fn lookup_benchmark_report() -> ArtifactLookupBenchmarkReport {
        ArtifactLookupBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            corpus_name: "packaged artifact lookup corpus".to_string(),
            rounds: 2,
            sample_count: 4,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(8),
        }
    }

    fn batch_lookup_benchmark_report() -> ArtifactBatchLookupBenchmarkReport {
        ArtifactBatchLookupBenchmarkReport {
            artifact_label: "packaged artifact".to_string(),
            source: "public reference snapshot".to_string(),
            corpus_name: "packaged artifact lookup corpus".to_string(),
            rounds: 2,
            batch_size: 4,
            encoded_bytes: 128,
            elapsed: Duration::from_millis(8),
        }
    }

    #[test]
    fn body_inspection_summary_includes_mean_boundary_deltas() {
        let inspection = ArtifactBodyInspection {
            body: CelestialBody::Sun,
            segment_count: 2,
            earliest: instant(1.0),
            latest: instant(2.0),
            sample_count: 6,
            min_segment_span_days: 0.5,
            max_segment_span_days: 1.5,
            mean_segment_span_days: 1.0,
            residual_segment_count: 1,
            boundary_checks: 2,
            sum_boundary_longitude_delta_deg: 0.20,
            sum_boundary_longitude_delta_deg_sq: 0.05,
            sum_boundary_latitude_delta_deg: 0.40,
            sum_boundary_latitude_delta_deg_sq: 0.20,
            sum_boundary_distance_delta_au: Some(0.60),
            sum_boundary_distance_delta_au_sq: Some(0.45),
            boundary_distance_checks: 2,
            max_boundary_longitude_delta_deg: 0.15,
            max_boundary_latitude_delta_deg: 0.30,
            max_boundary_distance_delta_au: Some(0.45),
        };

        let summary = inspection.summary_line();
        assert!(summary.contains("Sun: 2 segments,"));
        assert!(summary.contains("JD 1 TT → JD 2 TT"));
        assert!(summary.contains("6 samples, 2 boundary checks, 1 residual-bearing segments"));
        assert!(summary.contains("span days=0.500000000000..1.500000000000 (mean 1.000000000000)"));
        assert!(summary.contains("mean boundary Δlon=0.100000000000°"));
        assert!(summary.contains("rms boundary Δlon=0.158113883008°"));
        assert!(summary.contains("mean boundary Δlat=0.200000000000°"));
        assert!(summary.contains("rms boundary Δlat=0.316227766017°"));
        assert!(summary.contains("mean boundary Δdist=0.300000000000 AU"));
        assert!(summary.contains("rms boundary Δdist=0.474341649025 AU"));
        assert!(summary.contains("max boundary Δlon=0.150000000000°"));
        assert!(summary.contains("Δlat=0.300000000000°"));
        assert!(summary.contains("Δdist=0.450000000000 AU"));
    }

    #[test]
    fn artifact_lookup_benchmark_report_validated_summary_line_matches_summary_line() {
        let report = lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn artifact_lookup_benchmark_report_validated_summary_line_rejects_drift() {
        let mut report = lookup_benchmark_report();
        report.corpus_name = " ".to_string();

        assert!(matches!(
            report.validated_summary_line(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankCorpusName)
        ));
    }

    #[test]
    fn artifact_decode_benchmark_report_validated_summary_line_matches_summary_line() {
        let report = decode_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn artifact_decode_benchmark_report_validated_summary_line_rejects_drift() {
        let mut report = decode_benchmark_report();
        report.encoded_bytes = 0;

        assert!(matches!(
            report.validated_summary_line(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn artifact_inspection_report_summary_line_includes_residual_bodies_and_checksum_status() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");

        let summary = report.summary_line();
        assert!(summary.contains("artifact inspection:"));
        assert!(summary.contains("residual-bearing segments:"));
        assert!(summary.contains("residual-bearing bodies: asteroid:433-Eros"));
        assert!(summary.contains("body classes: luminaries=2; major planets=8; lunar points=0; built-in asteroids=0; custom bodies=1; other bodies=0"));
        assert!(summary.contains("roundtrip=ok"));
        assert!(summary.contains("checksum=ok"));
        assert!(summary.contains("encoded bytes="));
        assert!(summary.contains(&format!("encoded bytes={}", report.encoded_bytes)));
        assert!(matches!(
            report.validated_summary_line(),
            Ok(rendered) if rendered == summary
        ));
    }

    #[test]
    fn render_artifact_summary_includes_span_caps() {
        let rendered = super::render_artifact_summary().expect("artifact summary should render");

        assert!(rendered.contains("Artifact summary"));
        assert!(rendered.contains("Body-class cadence:"));
        assert!(rendered.contains("Body-class span caps: luminaries=256 days, inner planets=384 days, outer planets=768 days, pluto=1536 days, lunar points=256 days, selected asteroids=256 days, custom bodies=512 days"));
    }

    #[test]
    fn artifact_inspection_report_validated_summary_line_rejects_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.lookup_benchmark.corpus_name = " ".to_string();

        let error = report
            .validated_summary_line()
            .expect_err("lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::LookupBenchmark(
                ArtifactLookupBenchmarkReportValidationError::BlankCorpusName
            )
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_body_count_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.body_count += 1;

        let error = report
            .validate()
            .expect_err("body count drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `body_count`"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_residual_body_coverage_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.residual_bodies.push(CelestialBody::Sun);

        let error = report
            .validate()
            .expect_err("residual body drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `residual_bodies` does not match the inspected residual-bearing body set"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_residual_segment_count_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.residual_segment_count += 1;

        let error = report
            .validate()
            .expect_err("residual segment count drift should fail validation");
        assert!(error
            .to_string()
            .contains("artifact inspection report field `residual_segment_count` does not match the inspected residual-bearing segment count"));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_decode_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.decode_benchmark.encoded_bytes += 1;

        let error = report
            .validate()
            .expect_err("decode benchmark drift should fail validation");
        assert!(error.to_string().contains(
            "artifact inspection report decode benchmark encoded byte count does not match"
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_lookup_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.lookup_benchmark.corpus_name = " ".to_string();

        let error = report
            .validate()
            .expect_err("lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::LookupBenchmark(
                ArtifactLookupBenchmarkReportValidationError::BlankCorpusName
            )
        ));
    }

    #[test]
    fn artifact_inspection_report_validate_rejects_batch_lookup_benchmark_drift() {
        let artifact = packaged_artifact();
        let encoded = artifact.encode().expect("packaged artifact should encode");
        let mut report = ArtifactInspectionReport::from_artifact(artifact, encoded.len())
            .expect("artifact inspection report should build");
        report.batch_lookup_benchmark.batch_size = 0;

        let error = report
            .validate()
            .expect_err("batch lookup benchmark drift should fail validation");
        assert!(matches!(
            error,
            super::ArtifactInspectionError::BatchLookupBenchmark(
                ArtifactBatchLookupBenchmarkReportValidationError::ZeroBatchSize
            )
        ));
    }

    #[test]
    fn boundary_envelope_summary_includes_mean_boundary_deltas() {
        let summary = ArtifactBoundaryEnvelopeSummary {
            body_count: 2,
            boundary_check_count: 3,
            sum_boundary_longitude_delta_deg: 0.30,
            sum_boundary_longitude_delta_deg_sq: 0.07,
            sum_boundary_latitude_delta_deg: 0.60,
            sum_boundary_latitude_delta_deg_sq: 0.29,
            sum_boundary_distance_delta_au: Some(0.90),
            sum_boundary_distance_delta_au_sq: Some(0.63),
            boundary_distance_check_count: 3,
            max_boundary_longitude_delta_body: Some(CelestialBody::Moon),
            max_boundary_longitude_delta_deg: 0.18,
            max_boundary_latitude_delta_body: Some(CelestialBody::Sun),
            max_boundary_latitude_delta_deg: 0.27,
            max_boundary_distance_delta_body: Some(CelestialBody::Moon),
            max_boundary_distance_delta_au: Some(0.33),
        };

        let rendered = summary.summary_line();
        assert!(rendered.contains("Artifact boundary envelope: 3 checks across 2 bundled bodies"));
        assert!(rendered.contains("mean boundary Δlon=0.100000000000°"));
        assert!(rendered.contains("rms boundary Δlon=0.152752523165°"));
        assert!(rendered.contains("mean boundary Δlat=0.200000000000°"));
        assert!(rendered.contains("rms boundary Δlat=0.310912635103°"));
        assert!(rendered.contains("mean boundary Δdist=0.300000000000 AU (3 distance checks)"));
        assert!(rendered.contains("rms boundary Δdist=0.458257569496 AU (3 distance checks)"));
        assert!(rendered.contains("max boundary Δlon=0.180000000000° (Moon)"));
        assert!(rendered.contains("max boundary Δlat=0.270000000000° (Sun)"));
        assert!(rendered.contains("max boundary Δdist=0.330000000000 AU (Moon)"));
        assert_eq!(
            summary
                .validated_summary_line()
                .expect("boundary summary should validate"),
            rendered
        );
    }

    #[test]
    fn boundary_envelope_summary_rejects_inconsistent_distance_channels() {
        let summary = ArtifactBoundaryEnvelopeSummary {
            body_count: 1,
            boundary_check_count: 2,
            sum_boundary_longitude_delta_deg: 0.30,
            sum_boundary_longitude_delta_deg_sq: 0.07,
            sum_boundary_latitude_delta_deg: 0.60,
            sum_boundary_latitude_delta_deg_sq: 0.29,
            sum_boundary_distance_delta_au: Some(0.90),
            sum_boundary_distance_delta_au_sq: None,
            boundary_distance_check_count: 1,
            max_boundary_longitude_delta_body: Some(CelestialBody::Moon),
            max_boundary_longitude_delta_deg: 0.18,
            max_boundary_latitude_delta_body: Some(CelestialBody::Sun),
            max_boundary_latitude_delta_deg: 0.27,
            max_boundary_distance_delta_body: None,
            max_boundary_distance_delta_au: None,
        };

        let error = summary
            .validate()
            .expect_err("inconsistent distance coverage should fail");
        assert!(matches!(
            error,
            ArtifactBoundaryEnvelopeSummaryValidationError::InconsistentDistanceCoverage {
                boundary_distance_check_count: 1,
                has_sum: true,
                has_sum_sq: false,
                has_max: false,
            }
        ));
    }

    #[test]
    fn decode_benchmark_report_validate_accepts_compact_metadata() {
        let report = decode_benchmark_report();

        assert!(report.validate().is_ok());
        assert!((report.nanoseconds_per_decode() - 833_333.3333333334).abs() < 1e-9);
        assert!((report.decodes_per_second() - 1_200.0).abs() < 1e-9);
        assert!(report.to_string().contains("Artifact: packaged artifact"));
    }

    #[test]
    fn decode_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = decode_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("decodes per round=3"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/decode=833333.33"));
        assert!(summary.contains("decodes/s=1200.00"));
    }

    #[test]
    fn decode_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = decode_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = decode_benchmark_report();
        report.source = "\t".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = decode_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = decode_benchmark_report();
        report.sample_count = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroSampleCount)
        ));

        let mut report = decode_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactDecodeBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn lookup_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("lookups per round=4"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/lookup=1000000.00"));
        assert!(summary.contains("lookups/s=1000.00"));
    }

    #[test]
    fn batch_lookup_benchmark_report_summary_line_mentions_the_provenance_and_throughput() {
        let report = batch_lookup_benchmark_report();

        let summary = report.summary_line();
        assert!(summary.contains("artifact=packaged artifact"));
        assert!(summary.contains("source=public reference snapshot"));
        assert!(summary.contains("corpus=packaged artifact lookup corpus"));
        assert!(summary.contains("rounds=2"));
        assert!(summary.contains("lookups per batch=4"));
        assert!(summary.contains("encoded bytes=128"));
        assert!(summary.contains("ns/lookup=1000000.00"));
        assert!(summary.contains("lookups/s=1000.00"));
    }

    #[test]
    fn batch_lookup_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = batch_lookup_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.source = "	".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.corpus_name = " ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::BlankCorpusName)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.batch_size = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroBatchSize)
        ));

        let mut report = batch_lookup_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactBatchLookupBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }

    #[test]
    fn lookup_benchmark_report_validate_rejects_invalid_metadata() {
        let mut report = lookup_benchmark_report();
        report.artifact_label = "   ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankArtifactLabel)
        ));

        let mut report = lookup_benchmark_report();
        report.source = "\t".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankSource)
        ));

        let mut report = lookup_benchmark_report();
        report.corpus_name = " ".to_string();
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::BlankCorpusName)
        ));

        let mut report = lookup_benchmark_report();
        report.rounds = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroRounds)
        ));

        let mut report = lookup_benchmark_report();
        report.sample_count = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroSampleCount)
        ));

        let mut report = lookup_benchmark_report();
        report.encoded_bytes = 0;
        assert!(matches!(
            report.validate(),
            Err(ArtifactLookupBenchmarkReportValidationError::ZeroEncodedBytes)
        ));
    }
}
