//! Formatter writers, backend catalog, and value formatting helpers.

use std::fmt;

use crate::*;

pub(crate) fn write_corpus_summary(
    f: &mut fmt::Formatter<'_>,
    corpus: &CorpusSummary,
) -> fmt::Result {
    if let Err(error) = corpus.validate() {
        writeln!(f, "  corpus summary unavailable ({error})")?;
        return Ok(());
    }

    writeln!(f, "  name: {}", corpus.name)?;
    writeln!(f, "  description: {}", corpus.description)?;
    writeln!(f, "  Apparentness: {}", corpus.apparentness)?;
    writeln!(f, "  requests: {}", corpus.request_count)?;
    writeln!(f, "  epochs: {}", corpus.epoch_count)?;
    writeln!(f, "  epoch labels: {}", format_instant_list(&corpus.epochs))?;
    writeln!(f, "  bodies: {}", corpus.body_count)?;
    writeln!(
        f,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    )
}

pub(crate) fn write_corpus_summary_text(text: &mut String, corpus: &CorpusSummary) {
    use std::fmt::Write as _;

    if let Err(error) = corpus.validate() {
        let _ = writeln!(text, "  corpus summary unavailable ({error})");
        return;
    }

    let _ = writeln!(text, "  name: {}", corpus.name);
    let _ = writeln!(text, "  description: {}", corpus.description);
    let _ = writeln!(text, "  Apparentness: {}", corpus.apparentness);
    let _ = writeln!(text, "  requests: {}", corpus.request_count);
    let _ = writeln!(text, "  epochs: {}", corpus.epoch_count);
    let _ = writeln!(
        text,
        "  epoch labels: {}",
        format_instant_list(&corpus.epochs)
    );
    let _ = writeln!(text, "  bodies: {}", corpus.body_count);
    let _ = writeln!(
        text,
        "  julian day span: {:.1} → {:.1}",
        corpus.earliest_julian_day, corpus.latest_julian_day
    );
}

pub(crate) fn write_backend_matrix(
    f: &mut fmt::Formatter<'_>,
    backend: &BackendMetadata,
) -> fmt::Result {
    writeln!(
        f,
        "  summary: {}",
        backend
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    writeln!(f, "  id: {}", backend.id)?;
    writeln!(f, "  version: {}", backend.version)?;
    writeln!(f, "  family: {}", backend.family)?;
    writeln!(f, "  family posture: {}", backend.family.posture())?;
    writeln!(f, "  accuracy: {}", backend.accuracy)?;
    writeln!(f, "  deterministic: {}", backend.deterministic)?;
    writeln!(f, "  offline: {}", backend.offline)?;
    writeln!(f, "  nominal range: {}", backend.nominal_range)?;
    writeln!(
        f,
        "  time scales: {}",
        format_time_scales(&backend.supported_time_scales)
    )?;
    let bodies = backend.supported_bodies();
    writeln!(f, "  bodies: {}", format_bodies(&bodies))?;
    if let Some(asteroids) = selected_asteroid_coverage(&bodies) {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(&asteroids)
        )?;
        if backend.id.as_str() == "jpl-snapshot" {
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                selected_asteroid_source_window_summary_for_report()
            )?;
            writeln!(f, "  {}", selected_asteroid_boundary_summary_for_report())?;
            writeln!(f, "  {}", selected_asteroid_bridge_summary_for_report())?;
            let evidence = reference_asteroid_evidence();
            if let Some(first) = evidence.first() {
                writeln!(
                    f,
                    "  exact J2000 evidence: {} bodies at JD {:.1}",
                    evidence.len(),
                    first.epoch.julian_day.days()
                )?;
                for sample in evidence {
                    writeln!(
                        f,
                        "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                        sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                    )?;
                }
            }
            writeln!(
                f,
                "  {}",
                reference_snapshot_exact_j2000_evidence_summary_for_report()
            )?;
            writeln!(
                f,
                "  {}",
                reference_snapshot_major_body_bridge_summary_for_report()
            )?;
        }
    }
    writeln!(f, "  frames: {}", format_frames(&backend.supported_frames))?;
    writeln!(
        f,
        "  capabilities: {}",
        format_capabilities(&backend.capabilities)
    )?;
    writeln!(
        f,
        "  provenance: {}",
        backend
            .provenance
            .validated_summary_line()
            .unwrap_or_else(|error| format!("unavailable ({error})"))
    )?;
    if !backend.provenance.data_sources.is_empty() {
        writeln!(
            f,
            "  provenance sources: {}",
            backend.provenance.data_sources.join("; ")
        )?;
    }
    Ok(())
}

pub(crate) fn write_backend_catalog_entry(
    f: &mut fmt::Formatter<'_>,
    entry: &BackendMatrixEntry,
) -> fmt::Result {
    write_backend_matrix(f, &entry.metadata)?;
    writeln!(
        f,
        "  implementation status: {}",
        entry.implementation_status.label()
    )?;
    writeln!(f, "  implementation note: {}", entry.status_note)?;
    if entry.metadata.id.as_str() == "pleiades-vsop87" {
        writeln!(f, "  body source profiles:")?;
        for profile in body_source_profiles() {
            writeln!(f, "    {}", profile.summary_line())?;
        }

        writeln!(f, "  source documentation:")?;
        for spec in source_specifications() {
            writeln!(
                f,
                "    {}: {} {} | {} | {} | {} | {} | {} | {} | {}",
                spec.body,
                spec.variant,
                spec.source_file,
                spec.coordinate_family,
                spec.frame,
                spec.units,
                spec.reduction,
                spec.transform_note,
                spec.truncation_policy,
                spec.date_range
            )?;
        }

        writeln!(f, "  source audit:")?;
        for audit in source_audits() {
            writeln!(
                f,
                "    {}: {} bytes, {} lines, {} terms, 0x{:016x}",
                audit.body,
                audit.byte_length,
                audit.line_count,
                audit.term_count,
                audit.fingerprint
            )?;
        }

        writeln!(f, "  generated binary audit:")?;
        writeln!(
            f,
            "    {}",
            crate::posture::vsop87::audit::generated_binary_audit_summary_for_report()
        )?;

        writeln!(f, "  canonical J2000 VSOP87B evidence:")?;
        match vsop87_canonical_body_evidence() {
            Some(body_evidence) => {
                for evidence in body_evidence {
                    writeln!(
                        f,
                        "    {}: kind={} from {} — {} — Δlon={:.12}° / limit {:.12}° / margin {:+.12}°, Δlat={:.12}° / limit {:.12}° / margin {:+.12}°, Δdist={:.12} AU / limit {:.12} AU / margin {:+.12} AU",
                        evidence.body,
                        evidence.source_kind,
                        evidence.source_file,
                        if evidence.within_interim_limits {
                            evidence.provenance
                        } else {
                            "outside interim limits"
                        },
                        evidence.longitude_delta_deg,
                        evidence.longitude_limit_deg,
                        evidence.longitude_limit_deg - evidence.longitude_delta_deg,
                        evidence.latitude_delta_deg,
                        evidence.latitude_limit_deg,
                        evidence.latitude_limit_deg - evidence.latitude_delta_deg,
                        evidence.distance_delta_au,
                        evidence.distance_limit_au,
                        evidence.distance_limit_au - evidence.distance_delta_au
                    )?;
                }
            }
            None => {
                writeln!(f, "    unavailable")?;
            }
        }
        writeln!(
            f,
            "  body profile evidence summary: {}",
            format_vsop87_body_evidence_summary()
        )?;
    } else if entry.metadata.id.as_str() == "pleiades-elp" {
        let theory = lunar_theory_specification();
        writeln!(f, "  lunar theory specification:")?;
        writeln!(
            f,
            "    catalog summary: {}",
            crate::posture::elp::catalog::lunar_theory_catalog_summary_for_report()
        )?;
        writeln!(
            f,
            "    catalog validation: {}",
            validated_lunar_theory_catalog_validation_summary_for_report()
        )?;
        writeln!(f, "    model: {}", theory.model_name)?;
        writeln!(
            f,
            "    source family: {}",
            pleiades_elp::lunar_theory_source_family().label()
        )?;
        writeln!(
            f,
            "    capability summary: {}",
            crate::posture::elp::catalog::lunar_theory_capability_summary_for_report()
        )?;
        writeln!(
            f,
            "    specification summary: {}",
            crate::posture::elp::lib_summaries::lunar_theory_summary_for_report()
        )?;
        writeln!(f, "    source identifier: {}", theory.source_identifier)?;
        writeln!(f, "    source citation: {}", theory.source_citation)?;
        writeln!(f, "    source material: {}", theory.source_material)?;
        writeln!(f, "    redistribution note: {}", theory.redistribution_note)?;
        writeln!(f, "    license note: {}", theory.license_note)?;
        writeln!(
            f,
            "    supported bodies: {}",
            format_bodies(theory.supported_bodies)
        )?;
        writeln!(
            f,
            "    unsupported bodies: {}",
            format_bodies(theory.unsupported_bodies)
        )?;
        writeln!(
            f,
            "    request policy: {}",
            crate::posture::elp::lib_summaries::lunar_theory_request_policy_summary()
        )?;
        writeln!(f, "    validation window: {}", theory.validation_window)?;
        writeln!(f, "    date-range note: {}", theory.date_range_note)?;
        writeln!(f, "    frame note: {}", theory.frame_note)?;
        write_lunar_reference_evidence(f)?;
        write_lunar_equatorial_reference_evidence(f)?;
        write_lunar_apparent_comparison_evidence(f)?;
        write_lunar_source_window_evidence(f)?;
        writeln!(f, "  Lunar high-curvature continuity evidence:")?;
        writeln!(
            f,
            "    {}",
            crate::posture::elp::evidence::lunar_high_curvature_continuity_evidence_for_report()
        )?;
        write_lunar_high_curvature_equatorial_continuity_evidence(f)?;
    }
    if entry.metadata.id.as_str() == "jpl-snapshot" {
        write_jpl_interpolation_quality(f)?;
        writeln!(
            f,
            "    {}",
            jpl_snapshot_batch_error_taxonomy_summary_for_report()
        )?;
    }
    writeln!(
        f,
        "  expected error classes: {}",
        format_error_kinds(entry.expected_error_kinds)
    )?;
    if entry.required_data_files.is_empty() {
        writeln!(f, "  required external data files: none")?;
    } else {
        writeln!(
            f,
            "  required external data files: {}",
            format_data_files(entry.required_data_files)
        )?;
    }
    Ok(())
}

pub(crate) fn write_jpl_interpolation_quality(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  interpolation quality checks:")?;
    let Some(summary) = jpl_interpolation_quality_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        format_jpl_interpolation_quality_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        jpl_interpolation_quality_kind_coverage_for_report()
    )?;
    writeln!(f, "    {}", jpl_interpolation_posture_summary_for_report())?;
    writeln!(f, "    {}", jpl_independent_holdout_summary_for_report())?;
    writeln!(f, "    {}", render_reference_holdout_overlap_summary_text())?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_body_class_coverage_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        independent_holdout_snapshot_batch_parity_summary_text()
    )?;
    writeln!(
        f,
        "    {}",
        jpl_independent_holdout_snapshot_equatorial_parity_summary_for_report()
    )?;
    for sample in interpolation_quality_samples() {
        writeln!(
            f,
            "    {}",
            crate::posture::jpl::backend::interpolation_quality_sample_summary_line(sample)
        )?;
    }
    writeln!(
        f,
        "    {}",
        jpl_interpolation_body_class_error_envelopes_for_report()
    )?;
    Ok(())
}

pub(crate) fn jpl_interpolation_quality_summary(
) -> Option<pleiades_jpl::JplInterpolationQualitySummary> {
    pleiades_jpl::jpl_interpolation_quality_summary()
}

pub(crate) fn format_jpl_interpolation_quality_summary(
    summary: &pleiades_jpl::JplInterpolationQualitySummary,
) -> String {
    crate::posture::jpl::jpl_interpolation_quality_summary_line(summary)
}

pub(crate) fn write_lunar_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar reference:")?;
    let Some(summary) = lunar_reference_evidence_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::format_lunar_reference_evidence_summary(&summary)
    )?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_reference_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_reference_evidence_envelope_for_report()
    )?;
    for sample in lunar_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

pub(crate) fn write_lunar_equatorial_reference_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar equatorial reference:")?;
    if lunar_equatorial_reference_evidence_summary().is_none() {
        writeln!(f, "    none")?;
        return Ok(());
    }

    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_evidence_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_batch_parity_summary_for_report()
    )?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_equatorial_reference_evidence_envelope_for_report()
    )?;
    for sample in lunar_equatorial_reference_evidence() {
        writeln!(f, "    {}", sample)?;
    }
    Ok(())
}

pub(crate) fn write_lunar_apparent_comparison_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar apparent comparison:")?;
    let Some(summary) = lunar_apparent_comparison_summary() else {
        writeln!(f, "    none")?;
        return Ok(());
    };

    writeln!(f, "    {}", summary.summary_line())?;
    for sample in lunar_apparent_comparison_evidence() {
        writeln!(
            f,
            "    {} at JD {:.1}: apparent lon={:.12}°, apparent lat={:.12}°, apparent dist={:.12} AU, apparent RA={:.12}°, apparent Dec={:.12}°, note={}",
            sample.body,
            sample.epoch.julian_day.days(),
            sample.apparent_longitude_deg,
            sample.apparent_latitude_deg,
            sample.apparent_distance_au,
            sample.apparent_right_ascension_deg,
            sample.apparent_declination_deg,
            sample.note
        )?;
    }
    Ok(())
}

pub(crate) fn write_lunar_source_window_evidence(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "  Lunar source windows:")?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_source_window_summary_for_report()
    )?;
    Ok(())
}

pub(crate) fn write_lunar_high_curvature_equatorial_continuity_evidence(
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    writeln!(f, "  Lunar high-curvature equatorial continuity evidence:")?;
    writeln!(
        f,
        "    {}",
        crate::posture::elp::evidence::lunar_high_curvature_equatorial_continuity_evidence_for_report()
    )?;
    Ok(())
}

pub(crate) fn write_comparison_summary(
    f: &mut fmt::Formatter<'_>,
    report: &ComparisonReport,
) -> fmt::Result {
    let summary = &report.summary;
    let comparison_envelope = comparison_envelope_summary(summary, &report.samples);
    let median = comparison_envelope.median;

    writeln!(f, "  samples: {}", summary.sample_count)?;
    writeln!(
        f,
        "  max longitude delta: {:.12}°",
        summary.max_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean longitude delta: {:.12}°",
        summary.mean_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  median longitude delta: {:.12}°",
        median.longitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms longitude delta: {:.12}°",
        summary.rms_longitude_delta_deg
    )?;
    writeln!(
        f,
        "  max latitude delta: {:.12}°",
        summary.max_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  mean latitude delta: {:.12}°",
        summary.mean_latitude_delta_deg
    )?;
    writeln!(
        f,
        "  median latitude delta: {:.12}°",
        median.latitude_delta_deg
    )?;
    writeln!(
        f,
        "  rms latitude delta: {:.12}°",
        summary.rms_latitude_delta_deg
    )?;
    if let Some(value) = summary.max_distance_delta_au {
        writeln!(f, "  max distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.mean_distance_delta_au {
        writeln!(f, "  mean distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = median.distance_delta_au {
        writeln!(f, "  median distance delta: {:.12} AU", value)?;
    }
    if let Some(value) = summary.rms_distance_delta_au {
        writeln!(f, "  rms distance delta: {:.12} AU", value)?;
    }
    match comparison_envelope.validated_percentile_line(&report.samples) {
        Ok(line) => writeln!(f, "  {line}")?,
        Err(error) => writeln!(f, "  comparison percentile envelope unavailable ({error})")?,
    }
    Ok(())
}

pub(crate) fn format_body_comparison_summary_for_report(summary: &BodyComparisonSummary) -> String {
    match summary.validated_summary_line() {
        Ok(line) => line,
        Err(error) => format!(
            "body comparison summary for {} unavailable ({error})",
            summary.body
        ),
    }
}

pub(crate) fn write_body_comparison_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyComparisonSummary],
) -> fmt::Result {
    writeln!(f, "Body comparison summaries")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        writeln!(
            f,
            "  {}",
            format_body_comparison_summary_for_report(summary)
        )?;
    }
    Ok(())
}

pub(crate) fn write_body_class_envelopes(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
) -> fmt::Result {
    writeln!(f, "Body-class error envelopes")?;
    let summaries = body_class_summaries(samples);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

pub(crate) fn write_body_class_tolerance_posture(
    f: &mut fmt::Formatter<'_>,
    samples: &[ComparisonSample],
    backend_family: &BackendFamily,
) -> fmt::Result {
    writeln!(f, "Body-class tolerance posture")?;
    let summaries = body_class_tolerance_summaries(samples, backend_family);
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        summary.render(f)?;
    }
    Ok(())
}

pub(crate) fn tolerance_backend_family_label(family: &BackendFamily) -> String {
    match family {
        BackendFamily::Algorithmic => "algorithmic".to_string(),
        BackendFamily::ReferenceData => "reference data".to_string(),
        BackendFamily::CompressedData => "compressed data".to_string(),
        BackendFamily::Composite => "composite".to_string(),
        BackendFamily::Other(value) => format!("other ({value})"),
        _ => "other (unknown)".to_string(),
    }
}

pub(crate) fn write_tolerance_summaries(
    f: &mut fmt::Formatter<'_>,
    summaries: &[BodyToleranceSummary],
) -> fmt::Result {
    writeln!(f, "Expected tolerance status")?;
    if summaries.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(
                f,
                "  body tolerance summary for {} unavailable ({error})",
                summary.body
            ),
        }?;
    }
    Ok(())
}

pub(crate) fn write_regression_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    findings: &[RegressionFinding],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    if findings.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in findings {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

pub(crate) fn write_regression_archive_section(
    f: &mut fmt::Formatter<'_>,
    archive: &RegressionArchive,
) -> fmt::Result {
    writeln!(f, "Archived regression cases")?;
    writeln!(f, "  corpus: {}", archive.corpus_name)?;
    if archive.cases.is_empty() {
        writeln!(f, "  none")?;
        return Ok(());
    }

    for finding in &archive.cases {
        match finding.validated_summary_line() {
            Ok(line) => writeln!(f, "  {line}"),
            Err(error) => writeln!(f, "  regression finding unavailable ({error})"),
        }?;
    }
    Ok(())
}

pub(crate) fn write_reference_asteroid_section(f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Selected asteroid coverage")?;
    let asteroids = reference_asteroids();
    if asteroids.is_empty() {
        writeln!(f, "  none")?;
    } else {
        writeln!(
            f,
            "  {}",
            selected_asteroid_coverage_summary_for_report(asteroids)
        )?;
        let evidence = reference_asteroid_evidence();
        if evidence.is_empty() {
            writeln!(f, "  exact J2000 evidence: unavailable")?;
        } else {
            writeln!(
                f,
                "  exact J2000 evidence: {} bodies at JD {:.1}",
                evidence.len(),
                evidence[0].epoch.julian_day.days()
            )?;
            for sample in evidence {
                writeln!(
                    f,
                    "    {}: lon={:.12}°, lat={:.12}°, dist={:.12} AU",
                    sample.body, sample.longitude_deg, sample.latitude_deg, sample.distance_au
                )?;
            }
        }
        writeln!(
            f,
            "  note: comparison reports stay on the planetary subset while the JPL snapshot preserves selected asteroid coverage."
        )?;
    }
    Ok(())
}

pub(crate) fn regression_finding(
    sample: &ComparisonSample,
    backend_family: &BackendFamily,
) -> Option<RegressionFinding> {
    let tolerance = comparison_tolerance_for_body(&sample.body, backend_family);
    let mut notes = Vec::new();
    if sample.longitude_delta_deg >= tolerance.max_longitude_delta_deg {
        notes.push(format!(
            "longitude delta exceeds {:.1}°",
            tolerance.max_longitude_delta_deg
        ));
    }
    if sample.latitude_delta_deg >= tolerance.max_latitude_delta_deg {
        notes.push(format!(
            "latitude delta exceeds {:.2}°",
            tolerance.max_latitude_delta_deg
        ));
    }
    if sample
        .distance_delta_au
        .is_some_and(|value| value >= tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY))
    {
        notes.push(format!(
            "distance delta exceeds {:.3} AU",
            tolerance.max_distance_delta_au.unwrap_or(f64::INFINITY)
        ));
    }

    if notes.is_empty() {
        return None;
    }

    Some(RegressionFinding {
        body: sample.body.clone(),
        longitude_delta_deg: sample.longitude_delta_deg,
        latitude_delta_deg: sample.latitude_delta_deg,
        distance_delta_au: sample.distance_delta_au,
        note: notes.join(", "),
    })
}

pub(crate) const JPL_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
];
pub(crate) const JPL_REQUIRED_DATA_FILES: &[&str] = &[
    "crates/pleiades-jpl/data/reference_snapshot.csv",
    "crates/pleiades-jpl/data/j2000_snapshot.csv",
];
pub(crate) const VSOP87_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
pub(crate) const ELP_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidRequest,
];
pub(crate) const PACKAGED_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];
pub(crate) const COMPOSITE_EXPECTED_ERROR_KINDS: &[EphemerisErrorKind] = &[
    EphemerisErrorKind::UnsupportedBody,
    EphemerisErrorKind::UnsupportedCoordinateFrame,
    EphemerisErrorKind::UnsupportedTimeScale,
    EphemerisErrorKind::InvalidObserver,
    EphemerisErrorKind::InvalidRequest,
    EphemerisErrorKind::MissingDataset,
    EphemerisErrorKind::OutOfRangeInstant,
    EphemerisErrorKind::NumericalFailure,
];

pub(crate) fn implemented_backend_catalog() -> Vec<BackendMatrixEntry> {
    vec![
        BackendMatrixEntry {
            label: "JPL snapshot reference backend",
            metadata: default_reference_backend().metadata(),
            implementation_status: BackendImplementationStatus::FixtureReference,
            status_note: "checked-in public-input derivative fixture with exact lookup and cubic interpolation on four-sample windows when available, with quadratic and linear fallbacks for sparser bodies; reference corpus now spans 277 rows across 16 bodies and 23 epochs with expanded bridge and boundary coverage, while the broader production reader remains planned",
            expected_error_kinds: JPL_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
        BackendMatrixEntry {
            label: "VSOP87 planetary backend",
            metadata: Vsop87Backend::new().metadata(),
            implementation_status: BackendImplementationStatus::PartialSourceBacked,
            status_note: "Sun through Neptune now use generated binary VSOP87B source tables derived from the vendored full-file inputs, and Pluto remains the current approximate mean-element fallback special case until a Pluto-specific source path is selected",
            expected_error_kinds: VSOP87_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "ELP lunar backend (Moon and lunar nodes)",
            metadata: ElpBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::PreliminaryAlgorithm,
            status_note: "compact lunar and lunar-point formulas provide the current deterministic baseline while documented production lunar-theory ingestion remains open",
            expected_error_kinds: ELP_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Packaged data backend",
            metadata: PackagedDataBackend::new().metadata(),
            implementation_status: BackendImplementationStatus::DraftArtifact,
            status_note: "sample packaged artifact exercises lookup and profile plumbing; generated 1600-2600 production artifacts are Phase 2 work",
            expected_error_kinds: PACKAGED_EXPECTED_ERROR_KINDS,
            required_data_files: &[],
        },
        BackendMatrixEntry {
            label: "Composite routed backend",
            metadata: default_candidate_backend().metadata(),
            implementation_status: BackendImplementationStatus::RoutingFacade,
            status_note: "routes current planetary and lunar implementations for chart-facing validation without increasing underlying backend accuracy claims",
            expected_error_kinds: COMPOSITE_EXPECTED_ERROR_KINDS,
            required_data_files: JPL_REQUIRED_DATA_FILES,
        },
    ]
}

pub(crate) struct BackendMatrixEntry {
    pub(crate) label: &'static str,
    pub(crate) metadata: BackendMetadata,
    pub(crate) implementation_status: BackendImplementationStatus,
    pub(crate) status_note: &'static str,
    pub(crate) expected_error_kinds: &'static [EphemerisErrorKind],
    pub(crate) required_data_files: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BackendImplementationStatus {
    FixtureReference,
    PartialSourceBacked,
    PreliminaryAlgorithm,
    DraftArtifact,
    RoutingFacade,
}

impl BackendImplementationStatus {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::FixtureReference => "fixture-reference",
            Self::PartialSourceBacked => "partial-source-backed",
            Self::PreliminaryAlgorithm => "preliminary-algorithm",
            Self::DraftArtifact => "draft-artifact",
            Self::RoutingFacade => "routing-facade",
        }
    }
}

pub(crate) fn write_backend_catalog(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    catalog: &[BackendMatrixEntry],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for entry in catalog {
        writeln!(f, "{}", entry.label)?;
        write_backend_catalog_entry(f, entry)?;
        writeln!(f)?;
    }
    Ok(())
}

pub(crate) fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn selected_asteroid_coverage(bodies: &[CelestialBody]) -> Option<Vec<CelestialBody>> {
    let asteroids = bodies
        .iter()
        .filter(|body| is_selected_asteroid(body))
        .cloned()
        .collect::<Vec<_>>();

    if asteroids.is_empty() {
        None
    } else {
        Some(asteroids)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedAsteroidCoverageSummary {
    pub(crate) body_count: usize,
    pub(crate) bodies: Vec<CelestialBody>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SelectedAsteroidCoverageSummaryValidationError {
    MissingBodies,
    BodyCountMismatch {
        body_count: usize,
        bodies_len: usize,
    },
    DuplicateBody {
        first_index: usize,
        second_index: usize,
        body: String,
    },
    UnsupportedBody {
        index: usize,
        body: String,
    },
}

impl fmt::Display for SelectedAsteroidCoverageSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBodies => f.write_str("missing bodies"),
            Self::BodyCountMismatch {
                body_count,
                bodies_len,
            } => write!(f, "body count {body_count} does not match body list length {bodies_len}"),
            Self::DuplicateBody {
                first_index,
                second_index,
                body,
            } => write!(f, "duplicate body '{body}' at index {second_index} (first seen at index {first_index})"),
            Self::UnsupportedBody { index, body } => write!(f, "body '{body}' at index {index} is not a selected asteroid"),
        }
    }
}

impl std::error::Error for SelectedAsteroidCoverageSummaryValidationError {}

impl SelectedAsteroidCoverageSummary {
    pub(crate) fn summary_line(&self) -> String {
        format!(
            "selected asteroid coverage: {} bodies ({})",
            self.body_count,
            format_bodies(&self.bodies)
        )
    }

    pub(crate) fn validate(&self) -> Result<(), SelectedAsteroidCoverageSummaryValidationError> {
        if self.body_count == 0 || self.bodies.is_empty() {
            return Err(SelectedAsteroidCoverageSummaryValidationError::MissingBodies);
        }
        if self.body_count != self.bodies.len() {
            return Err(
                SelectedAsteroidCoverageSummaryValidationError::BodyCountMismatch {
                    body_count: self.body_count,
                    bodies_len: self.bodies.len(),
                },
            );
        }
        for (index, body) in self.bodies.iter().enumerate() {
            if self.bodies[..index].iter().any(|other| other == body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::DuplicateBody {
                        first_index: self.bodies[..index]
                            .iter()
                            .position(|other| other == body)
                            .expect("duplicate body should have a first index"),
                        second_index: index,
                        body: body.to_string(),
                    },
                );
            }
            if !is_selected_asteroid(body) {
                return Err(
                    SelectedAsteroidCoverageSummaryValidationError::UnsupportedBody {
                        index,
                        body: body.to_string(),
                    },
                );
            }
        }

        Ok(())
    }

    pub(crate) fn validated_summary_line(
        &self,
    ) -> Result<String, SelectedAsteroidCoverageSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

pub(crate) fn selected_asteroid_coverage_summary(
    bodies: &[CelestialBody],
) -> Option<SelectedAsteroidCoverageSummary> {
    selected_asteroid_coverage(bodies).map(|bodies| SelectedAsteroidCoverageSummary {
        body_count: bodies.len(),
        bodies,
    })
}

pub(crate) fn selected_asteroid_coverage_summary_for_report(bodies: &[CelestialBody]) -> String {
    match selected_asteroid_coverage_summary(bodies) {
        Some(summary) => summary
            .validated_summary_line()
            .unwrap_or_else(|error| format!("selected asteroid coverage: unavailable ({error})")),
        None => "selected asteroid coverage: unavailable".to_string(),
    }
}

pub(crate) fn is_selected_asteroid(body: &CelestialBody) -> bool {
    match body {
        CelestialBody::Ceres
        | CelestialBody::Pallas
        | CelestialBody::Juno
        | CelestialBody::Vesta => true,
        CelestialBody::Custom(custom) => custom.catalog == "asteroid",
        _ => false,
    }
}

pub(crate) fn format_frames(frames: &[CoordinateFrame]) -> String {
    frames
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_capabilities(capabilities: &BackendCapabilities) -> String {
    match capabilities.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("unavailable ({error})"),
    }
}

pub(crate) fn format_error_kinds(kinds: &[EphemerisErrorKind]) -> String {
    kinds
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_data_files(files: &[&str]) -> String {
    files.join("; ")
}

pub(crate) fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

pub(crate) fn format_instant_list(instants: &[Instant]) -> String {
    if instants.is_empty() {
        return "none".to_string();
    }

    instants
        .iter()
        .copied()
        .map(format_instant)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_ns(value: f64) -> String {
    format!("{value:.2}")
}

pub(crate) fn format_duration(duration: std::time::Duration) -> String {
    format!("{:.6}s", duration.as_secs_f64())
}
