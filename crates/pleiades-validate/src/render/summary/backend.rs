//! Backend capability matrix summary and report rendering.

use std::collections::BTreeMap;
use std::fmt;

use crate::*;

/// Renders a compact summary of the implemented backend capability matrix catalog.
pub fn render_backend_matrix_summary() -> String {
    render_backend_matrix_summary_text()
}

pub(crate) fn native_sidereal_posture_line(native_sidereal_count: usize) -> String {
    match native_sidereal_count {
        0 => "Native sidereal posture: unsupported across first-party backends".to_string(),
        1 => "Native sidereal posture: supported natively by 1 backend".to_string(),
        count => format!("Native sidereal posture: supported natively by {count} backends"),
    }
}

pub(crate) fn render_backend_matrix_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    };
    if let Err(error) = validated_compatibility_profile_for_report() {
        return format!("Backend matrix summary unavailable ({error})");
    }
    let catalog = implemented_backend_catalog();
    let mut family_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut bodies: Vec<String> = Vec::new();
    let mut frames: Vec<String> = Vec::new();
    let mut time_scales: Vec<String> = Vec::new();
    let mut deterministic_count = 0usize;
    let mut offline_count = 0usize;
    let mut batch_count = 0usize;
    let mut native_sidereal_count = 0usize;
    let mut bounded_nominal_range_count = 0usize;
    let mut open_ended_nominal_range_count = 0usize;
    let mut exact_accuracy_count = 0usize;
    let mut high_accuracy_count = 0usize;
    let mut moderate_accuracy_count = 0usize;
    let mut approximate_accuracy_count = 0usize;
    let mut unknown_accuracy_count = 0usize;
    let mut selected_asteroid_count = 0usize;
    let mut data_source_count = 0usize;
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();

    for entry in &catalog {
        *status_counts
            .entry(entry.implementation_status.label().to_string())
            .or_insert(0) += 1;

        *family_counts
            .entry(backend_family_label(&entry.metadata.family))
            .or_insert(0) += 1;
        deterministic_count += usize::from(entry.metadata.deterministic);
        offline_count += usize::from(entry.metadata.offline);
        batch_count += usize::from(entry.metadata.capabilities.batch);
        native_sidereal_count += usize::from(entry.metadata.capabilities.native_sidereal);
        if entry.metadata.nominal_range.start.is_some()
            || entry.metadata.nominal_range.end.is_some()
        {
            bounded_nominal_range_count += 1;
        } else {
            open_ended_nominal_range_count += 1;
        }
        match entry.metadata.accuracy {
            AccuracyClass::Exact => exact_accuracy_count += 1,
            AccuracyClass::High => high_accuracy_count += 1,
            AccuracyClass::Moderate => moderate_accuracy_count += 1,
            AccuracyClass::Approximate => approximate_accuracy_count += 1,
            AccuracyClass::Unknown => unknown_accuracy_count += 1,
            _ => unknown_accuracy_count += 1,
        }
        let entry_bodies = entry.metadata.supported_bodies();
        if selected_asteroid_coverage(&entry_bodies).is_some() {
            selected_asteroid_count += 1;
        }
        if !entry.metadata.provenance.data_sources.is_empty() {
            data_source_count += 1;
        }
        for body in &entry_bodies {
            push_unique(&mut bodies, body.to_string());
        }
        for frame in &entry.metadata.supported_frames {
            push_unique(&mut frames, frame.to_string());
        }
        for scale in &entry.metadata.supported_time_scales {
            push_unique(&mut time_scales, scale.to_string());
        }
    }

    let mut family_entries = family_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    family_entries.sort();

    let mut status_entries = status_counts
        .into_iter()
        .map(|(label, count)| format!("{label}: {count}"))
        .collect::<Vec<_>>();
    status_entries.sort();

    let mut text = String::new();
    text.push_str("Backend matrix summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Backends: ");
    text.push_str(&catalog.len().to_string());
    text.push('\n');
    text.push_str("Families: ");
    text.push_str(&family_entries.join(", "));
    text.push('\n');
    text.push_str("Implementation statuses: ");
    text.push_str(&status_entries.join(", "));
    text.push('\n');
    text.push_str("Deterministic backends: ");
    text.push_str(&deterministic_count.to_string());
    text.push('\n');
    text.push_str("Offline backends: ");
    text.push_str(&offline_count.to_string());
    text.push('\n');
    text.push_str("Batch-capable backends: ");
    text.push_str(&batch_count.to_string());
    text.push('\n');
    text.push_str("Native sidereal backends: ");
    text.push_str(&native_sidereal_count.to_string());
    text.push('\n');
    text.push_str(&native_sidereal_posture_line(native_sidereal_count));
    text.push('\n');
    text.push_str("Nominal ranges: bounded: ");
    text.push_str(&bounded_nominal_range_count.to_string());
    text.push_str(", open-ended: ");
    text.push_str(&open_ended_nominal_range_count.to_string());
    text.push('\n');
    text.push_str("Accuracy classes: Exact: ");
    text.push_str(&exact_accuracy_count.to_string());
    text.push_str(", High: ");
    text.push_str(&high_accuracy_count.to_string());
    text.push_str(", Moderate: ");
    text.push_str(&moderate_accuracy_count.to_string());
    text.push_str(", Approximate: ");
    text.push_str(&approximate_accuracy_count.to_string());
    text.push_str(", Unknown: ");
    text.push_str(&unknown_accuracy_count.to_string());
    text.push('\n');
    text.push_str("Backends with selected asteroid coverage: ");
    text.push_str(&selected_asteroid_count.to_string());
    text.push('\n');
    text.push_str(&selected_asteroid_source_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&selected_asteroid_terminal_boundary_summary_for_report());
    text.push('\n');
    text.push_str("Comparison corpus release-grade guard: ");
    match validated_comparison_corpus_release_guard_summary_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Reference/hold-out overlap: ");
    text.push_str(&render_reference_holdout_overlap_summary_text());
    text.push('\n');
    text.push_str("JPL independent hold-out: ");
    text.push_str(&jpl_independent_holdout_summary_for_report());
    text.push('\n');
    text.push_str("Release-grade body claims: ");
    text.push_str(&format_release_body_claims_summary_for_report());
    text.push('\n');
    text.push_str("Body/date/channel claims: ");
    text.push_str(&format_body_date_channel_claims_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus: ");
    text.push_str(&source_corpus_summary_for_report());
    text.push('\n');
    text.push_str("Source corpus posture: ");
    text.push_str(&source_corpus_posture_summary_for_report());
    text.push('\n');
    text.push_str("JPL source corpus contract: ");
    match required_labelled_summary_payload(
        jpl_source_corpus_contract_summary_for_report(),
        "JPL source corpus contract: ",
        "JPL source corpus contract",
    ) {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Catalog posture: ");
    match current_compatibility_profile().validated_catalog_posture_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target house scope: ");
    match current_compatibility_profile().validated_target_house_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Target ayanamsa scope: ");
    match current_compatibility_profile().validated_target_ayanamsa_scope_summary_line() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Pluto fallback: ");
    match validated_pluto_fallback_summary_line_for_report() {
        Ok(summary) => text.push_str(summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("House code aliases: ");
    match validated_house_code_aliases_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str(&reference_asteroid_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&reference_asteroid_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_boundary_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_sparse_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_pre_bridge_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_dense_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_2451914_major_body_bridge_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_lunar_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_1900_selected_body_boundary_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_high_curvature_epoch_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_source_window_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_manifest_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_equatorial_parity_summary_for_report());
    text.push('\n');
    text.push_str(&reference_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&jpl_snapshot_batch_error_taxonomy_summary_for_report());
    text.push('\n');
    text.push_str(&validated_production_generation_manifest_summary_text_for_report());
    text.push('\n');
    text.push_str("Production generation source revision: ");
    match validated_production_generation_source_revision_summary_for_report() {
        Ok(summary) => text.push_str(&summary),
        Err(error) => return format!("Backend matrix summary unavailable ({error})"),
    }
    text.push('\n');
    text.push_str("Production generation source: ");
    text.push_str(&production_generation_source_summary_for_report());
    text.push('\n');
    text.push_str("Production generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation coverage: ");
    text.push_str(&production_generation_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&production_generation_snapshot_window_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation body-class coverage: ");
    text.push_str(&validated_production_generation_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation corpus shape: ");
    text.push_str(&production_generation_corpus_shape_summary_for_report());
    text.push('\n');
    text.push_str("JPL production-generation boundary request corpus equatorial: ");
    text.push_str(&production_generation_boundary_request_corpus_equatorial_summary_for_report());
    text.push('\n');
    text.push_str(&jpl_source_corpus_contract_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_body_class_coverage_summary_for_report());
    text.push('\n');
    text.push_str(&comparison_snapshot_batch_parity_summary_text());
    text.push('\n');
    text.push_str(&comparison_snapshot_source_summary_for_report());
    text.push('\n');
    text.push_str(&format_comparison_snapshot_manifest_summary());
    text.push('\n');
    if let Ok(report) = build_validation_report(SUMMARY_BENCHMARK_ROUNDS) {
        text.push_str("Comparison audit: compare-backends-audit; ");
        text.push_str(&comparison_audit_summary_for_report(&report.comparison));
        text.push('\n');
    }
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str(&format_request_semantics_summary_for_report(
        &time_scale_policy,
    ));
    text.push_str(&request_surface_summary_for_report());
    text.push('\n');
    text.push_str("Frame policy: ");
    text.push_str(&validated_frame_policy_summary_for_report());
    text.push('\n');
    text.push_str("Mean-obliquity frame round-trip: ");
    text.push_str(&mean_obliquity_frame_round_trip_summary_for_report());
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Backends with external data sources: ");
    text.push_str(&data_source_count.to_string());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_documentation_health_summary());
    text.push('\n');
    text.push_str(&format_vsop87_frame_treatment_summary());
    text.push('\n');
    text.push_str("VSOP87 request policy: ");
    text.push_str(&format_vsop87_request_policy_summary());
    text.push('\n');
    text.push_str(&format_vsop87_source_audit_summary());
    text.push('\n');
    text.push_str(&crate::posture::vsop87::audit::generated_binary_audit_summary_for_report());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_canonical_outlier_note_summary());
    text.push('\n');
    text.push_str(&format_vsop87_equatorial_evidence_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j2000_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j2000_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_ecliptic_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_supported_body_j1900_equatorial_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_mixed_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_j1900_batch_summary());
    text.push('\n');
    text.push_str(&format_vsop87_body_evidence_summary());
    text.push('\n');
    text.push_str(&lunar_theory_catalog_summary_for_report());
    text.push('\n');
    text.push_str(&validated_lunar_theory_catalog_validation_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_source_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_theory_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference\n");
    text.push_str(&lunar_equatorial_reference_evidence_summary_for_report());
    text.push('\n');
    text.push_str("Lunar equatorial reference batch parity\n");
    text.push_str(&lunar_equatorial_reference_batch_parity_summary_for_report());
    text.push('\n');
    text.push_str(&lunar_equatorial_reference_evidence_envelope_for_report());
    text.push('\n');
    text.push_str("Lunar source windows: ");
    text.push_str(&lunar_source_window_summary_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature continuity evidence\n");
    text.push_str(&lunar_high_curvature_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Lunar high-curvature equatorial continuity evidence\n");
    text.push_str(&lunar_high_curvature_equatorial_continuity_evidence_for_report());
    text.push('\n');
    text.push_str("Distinct bodies covered: ");
    text.push_str(&bodies.len().to_string());
    text.push_str(" (");
    text.push_str(&bodies.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct coordinate frames: ");
    text.push_str(&frames.len().to_string());
    text.push_str(" (");
    text.push_str(&frames.join(", "));
    text.push_str(")\n");
    text.push_str("Distinct time scales: ");
    text.push_str(&time_scales.len().to_string());
    text.push_str(" (");
    text.push_str(&time_scales.join(", "));
    text.push_str(")\n");
    let time_scale_policy = time_scale_policy_summary_for_report();
    text.push_str("Time-scale policy: ");
    text.push_str(&format_time_scale_policy_summary_for_report(
        &time_scale_policy,
    ));
    text.push('\n');
    text.push_str("Delta T policy: ");
    text.push_str(&format_delta_t_policy_summary_for_report(
        &delta_t_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Observer policy: ");
    text.push_str(&format_observer_policy_summary_for_report(
        &crate::posture::backend_policy::observer_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Apparentness policy: ");
    text.push_str(&format_apparentness_policy_summary_for_report(
        &crate::posture::backend_policy::apparentness_policy_summary_for_report(),
    ));
    text.push('\n');
    text.push_str("Native sidereal policy: ");
    text.push_str(
        &crate::posture::backend_policy::validated_native_sidereal_policy_summary_for_report(),
    );
    text.push('\n');
    text.push_str("Zodiac policy: ");
    text.push_str(&validated_zodiac_policy_summary_for_report());
    text.push('\n');
    text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
    text.push_str("Release profile identifiers: ");
    text.push_str(&validated_release_profile_identifiers_summary_for_report(
        &release_profiles,
    ));
    text.push('\n');
    text.push_str("API stability summary: api-stability-summary\n");
    text.push_str("Release notes summary: release-notes-summary\n");
    text.push_str("Compatibility profile verification: verify-compatibility-profile\n");
    text.push_str("Release bundle verification: verify-release-bundle\n");
    text.push_str("Packaged-artifact summary: artifact-summary / artifact-posture-summary\n");
    text.push_str("Release checklist summary: release-checklist-summary\n");
    text.push_str("Validation report summary: validation-report-summary / validation-summary / report-summary\n");
    text.push_str("See release-summary for the compact one-screen release overview.\n");

    text
}

/// Renders a compact summary of the API stability posture.
pub fn render_api_stability_summary() -> String {
    render_api_stability_summary_text()
}

pub(crate) fn render_api_stability_summary_text() -> String {
    let release_profiles = match validated_release_profile_identifiers_for_report() {
        Ok(release_profiles) => release_profiles,
        Err(error) => return format!("API stability summary unavailable ({error})"),
    };

    match validated_api_stability_profile_for_report() {
        Ok(profile) => {
            let mut text = String::new();

            text.push_str("API stability summary\n");
            text.push_str("Profile: ");
            text.push_str(profile.profile_id);
            text.push('\n');
            text.push_str("Summary line: ");
            text.push_str(&profile.summary_line());
            text.push('\n');
            text.push_str("Compatibility profile: ");
            text.push_str(release_profiles.compatibility_profile_id);
            text.push('\n');
            text.push_str("Release profile identifiers: ");
            text.push_str(&validated_release_profile_identifiers_summary_for_report(
                &release_profiles,
            ));
            text.push('\n');
            text.push_str("Stable surfaces: ");
            text.push_str(&profile.stable_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Experimental surfaces: ");
            text.push_str(&profile.experimental_surfaces.len().to_string());
            text.push('\n');
            text.push_str("Deprecation policy items: ");
            text.push_str(&profile.deprecation_policy.len().to_string());
            text.push('\n');
            text.push_str("Intentional limits: ");
            text.push_str(&profile.intentional_limits.len().to_string());
            text.push('\n');
            text.push_str("Compatibility profile summary: compatibility-profile-summary\n");
            text.push_str("Backend matrix summary: backend-matrix-summary\n");
            text.push_str("Release notes summary: release-notes-summary\n");
            text.push_str("Release checklist summary: release-checklist-summary\n");
            text.push_str("Release bundle verification: verify-release-bundle\n");
            text.push_str("See release-summary for the compact one-screen release overview.\n");

            text
        }
        Err(error) => format!("API stability summary unavailable ({error})"),
    }
}

pub(crate) fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

pub(crate) fn backend_family_label(family: &BackendFamily) -> String {
    family.to_string()
}

/// Renders a backend capability matrix for the implemented backend catalog.
pub fn render_backend_matrix_report() -> Result<String, EphemerisError> {
    validated_compatibility_profile_for_report().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("backend capability matrix unavailable ({error})"),
        )
    })?;
    let mut rendered = String::new();
    fmt::write(
        &mut rendered,
        format_args!("Implemented backend matrices\n\n"),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    let house_code_aliases =
        validated_house_code_aliases_summary_for_report().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("backend capability matrix unavailable ({error})"),
            )
        })?;

    fmt::write(
        &mut rendered,
        format_args!("House code aliases: {}\n\n", house_code_aliases),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    fmt::write(
        &mut rendered,
        format_args!(
            "Body/date/channel claims: {}\n\n",
            format_body_date_channel_claims_summary_for_report()
        ),
    )
    .map_err(|_| {
        EphemerisError::new(
            EphemerisErrorKind::NumericalFailure,
            "failed to render backend capability matrix",
        )
    })?;

    for entry in implemented_backend_catalog() {
        validate_backend_matrix_entry(&entry)?;
        fmt::write(&mut rendered, format_args!("{}\n", entry.label)).map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
        fmt::write(
            &mut rendered,
            format_args!("{}\n\n", BackendMatrixDisplay(&entry)),
        )
        .map_err(|_| {
            EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "failed to render backend capability matrix",
            )
        })?;
    }

    Ok(rendered)
}

pub(crate) fn validate_backend_matrix_entry(
    entry: &BackendMatrixEntry,
) -> Result<(), EphemerisError> {
    entry.metadata.validate().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "backend matrix entry `{}` has invalid metadata: {error}",
                entry.label
            ),
        )
    })
}

pub(crate) struct BackendMatrixDisplay<'a>(&'a BackendMatrixEntry);

impl fmt::Display for BackendMatrixDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_backend_catalog_entry(f, self.0)
    }
}
