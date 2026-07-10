//! Compatibility-profile verification summaries and checks.

use std::fmt;

use crate::*;

/// A compact summary of the compatibility-profile verification posture.
const RELEASE_POSTURE_SUMMARY: &str = "Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented";

#[derive(Clone, Debug)]
pub struct CompatibilityProfileVerificationSummary {
    /// Release profile identifier that was verified.
    pub profile_id: String,
    /// Number of house-system descriptors checked.
    pub house_system_descriptor_count: usize,
    /// Number of house-system labels checked, including aliases.
    pub house_system_label_count: usize,
    /// Number of house-system aliases checked.
    pub house_system_alias_count: usize,
    /// Number of Swiss Ephemeris house-table code aliases checked.
    pub house_code_alias_count: usize,
    /// Compact mapping of the Swiss Ephemeris house-table code aliases.
    pub house_code_aliases_summary: String,
    /// Latitude-sensitive house systems exposed by the profile.
    pub latitude_sensitive_house_systems: Vec<String>,
    /// Number of ayanamsa descriptors checked.
    pub ayanamsa_descriptor_count: usize,
    /// Number of ayanamsa labels checked, including aliases.
    pub ayanamsa_label_count: usize,
    /// Number of ayanamsa aliases checked.
    pub ayanamsa_alias_count: usize,
    /// Number of baseline house-system descriptors.
    pub baseline_house_system_count: usize,
    /// Number of release-specific house-system descriptors.
    pub release_house_system_count: usize,
    /// Number of baseline ayanamsa descriptors.
    pub baseline_ayanamsa_count: usize,
    /// Number of release-specific ayanamsa descriptors.
    pub release_ayanamsa_count: usize,
    /// Release-specific house-system canonical names.
    pub release_house_canonical_names: String,
    /// Release-specific ayanamsa canonical names.
    pub release_ayanamsa_canonical_names: String,
    /// Built-in house formula families surfaced by the profile.
    pub house_formula_family_names: String,
    /// Human-readable release posture summary line.
    pub release_posture: String,
    /// Number of built-in ayanamsa descriptors that carry reference epoch/offset metadata.
    pub ayanamsa_metadata_count: usize,
    /// Number of built-in ayanamsa descriptors that still lack reference epoch/offset metadata.
    pub ayanamsa_metadata_gap_count: usize,
    /// Release-specific custom-definition label names.
    pub custom_definition_label_names: String,
    /// Number of custom-definition ayanamsa labels checked.
    pub custom_definition_ayanamsa_label_count: usize,
    /// Custom-definition ayanamsa label names.
    pub custom_definition_ayanamsa_label_names: String,
    /// Number of release notes documented in the profile.
    pub release_note_count: usize,
    /// Number of validation reference points documented in the profile.
    pub validation_reference_point_count: usize,
    /// Number of custom-definition labels checked.
    pub custom_definition_label_count: usize,
    /// Number of documented compatibility caveats.
    pub compatibility_caveat_count: usize,
}

impl CompatibilityProfileVerificationSummary {
    /// Validates that the summary still matches the current release profile.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let profile = current_compatibility_profile();
        profile.validate().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile validation failed: {error}"),
            )
        })?;
        let release_profiles = validated_release_profile_identifiers_for_report()
            .map_err(|error| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile verification summary release profile identifiers invalid: {error}"
                    ),
                )
            })?;

        if self.profile_id != release_profiles.compatibility_profile_id {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary profile id mismatch: expected {}, found {}",
                    release_profiles.compatibility_profile_id, self.profile_id
                ),
            ));
        }

        let house_labels_checked = verify_house_system_aliases(profile.house_systems)?;
        if self.house_system_descriptor_count != profile.house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-system descriptor count mismatch: expected {}, found {}",
                    profile.house_systems.len(), self.house_system_descriptor_count
                ),
            ));
        }
        if self.house_system_label_count != house_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-system label count mismatch: expected {}, found {}",
                    house_labels_checked, self.house_system_label_count
                ),
            ));
        }
        let expected_house_system_alias_count = house_labels_checked - profile.house_systems.len();
        if self.house_system_alias_count != expected_house_system_alias_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-system alias count mismatch: expected {}, found {}",
                    expected_house_system_alias_count, self.house_system_alias_count
                ),
            ));
        }
        if self.house_code_alias_count != profile.house_code_alias_count() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-code alias count mismatch: expected {}, found {}",
                    profile.house_code_alias_count(), self.house_code_alias_count
                ),
            ));
        }
        let expected_house_code_aliases = profile.house_code_aliases_summary_line();
        if self.house_code_aliases_summary != expected_house_code_aliases {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house-code aliases mismatch: expected [{}], found [{}]",
                    expected_house_code_aliases, self.house_code_aliases_summary
                ),
            ));
        }

        let expected_latitude_sensitive = profile.latitude_sensitive_house_systems();
        validate_name_sequence(
            "compatibility profile latitude-sensitive house systems",
            expected_latitude_sensitive.iter().copied(),
        )?;
        if self.latitude_sensitive_house_systems != expected_latitude_sensitive {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary latitude-sensitive house systems mismatch: expected [{}], found [{}]",
                    expected_latitude_sensitive.join(", "),
                    self.latitude_sensitive_house_systems.join(", ")
                ),
            ));
        }

        let ayanamsa_labels_checked = verify_ayanamsa_aliases(profile.ayanamsas)?;
        if self.ayanamsa_descriptor_count != profile.ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa descriptor count mismatch: expected {}, found {}",
                    profile.ayanamsas.len(), self.ayanamsa_descriptor_count
                ),
            ));
        }
        if self.ayanamsa_label_count != ayanamsa_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa label count mismatch: expected {}, found {}",
                    ayanamsa_labels_checked, self.ayanamsa_label_count
                ),
            ));
        }
        let expected_ayanamsa_alias_count = ayanamsa_labels_checked - profile.ayanamsas.len();
        if self.ayanamsa_alias_count != expected_ayanamsa_alias_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa alias count mismatch: expected {}, found {}",
                    expected_ayanamsa_alias_count, self.ayanamsa_alias_count
                ),
            ));
        }

        if self.baseline_house_system_count != profile.baseline_house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary baseline house-system count mismatch: expected {}, found {}",
                    profile.baseline_house_systems.len(), self.baseline_house_system_count
                ),
            ));
        }
        if self.release_house_system_count != profile.release_house_systems.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system count mismatch: expected {}, found {}",
                    profile.release_house_systems.len(), self.release_house_system_count
                ),
            ));
        }
        if self.baseline_ayanamsa_count != profile.baseline_ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary baseline ayanamsa count mismatch: expected {}, found {}",
                    profile.baseline_ayanamsas.len(), self.baseline_ayanamsa_count
                ),
            ));
        }
        if self.release_ayanamsa_count != profile.release_ayanamsas.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa count mismatch: expected {}, found {}",
                    profile.release_ayanamsas.len(), self.release_ayanamsa_count
                ),
            ));
        }

        let expected_release_house_names =
            profile.validated_release_house_system_canonical_names_summary_line().map_err(
                |error| {
                    EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "compatibility profile verification summary release house-system canonical names invalid: {error}"
                        ),
                    )
                },
            )?;
        if self.release_house_canonical_names != expected_release_house_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system canonical names mismatch: expected {}, found {}",
                    expected_release_house_names, self.release_house_canonical_names
                ),
            ));
        }

        let expected_release_ayanamsa_names =
            profile.validated_release_ayanamsa_canonical_names_summary_line().map_err(
                |error| {
                    EphemerisError::new(
                        EphemerisErrorKind::InvalidRequest,
                        format!(
                            "compatibility profile verification summary release ayanamsa canonical names invalid: {error}"
                        ),
                    )
                },
            )?;
        if self.release_ayanamsa_canonical_names != expected_release_ayanamsa_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa canonical names mismatch: expected {}, found {}",
                    expected_release_ayanamsa_names, self.release_ayanamsa_canonical_names
                ),
            ));
        }

        let expected_house_formula_family_names = profile.house_formula_family_names();
        validate_name_sequence(
            "compatibility profile house formula families",
            expected_house_formula_family_names
                .iter()
                .map(String::as_str),
        )?;
        let expected_house_formula_family_names = expected_house_formula_family_names.join(", ");
        if self.house_formula_family_names != expected_house_formula_family_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary house formula families mismatch: expected {}, found {}",
                    expected_house_formula_family_names, self.house_formula_family_names
                ),
            ));
        }
        if self.release_posture != RELEASE_POSTURE_SUMMARY {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release posture mismatch: expected {}, found {}",
                    RELEASE_POSTURE_SUMMARY, self.release_posture
                ),
            ));
        }

        let expected_ayanamsa_metadata_count = profile
            .ayanamsas
            .iter()
            .filter(|entry| entry.has_sidereal_metadata())
            .count();
        let expected_ayanamsa_metadata_gap_count =
            profile.ayanamsas.len() - expected_ayanamsa_metadata_count;
        if self.ayanamsa_metadata_count != expected_ayanamsa_metadata_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa metadata count mismatch: expected {}, found {}",
                    expected_ayanamsa_metadata_count, self.ayanamsa_metadata_count
                ),
            ));
        }
        if self.ayanamsa_metadata_gap_count != expected_ayanamsa_metadata_gap_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary ayanamsa metadata gap count mismatch: expected {}, found {}",
                    expected_ayanamsa_metadata_gap_count, self.ayanamsa_metadata_gap_count
                ),
            ));
        }

        let expected_custom_definition_label_names = profile.custom_definition_labels.join(", ");
        if self.custom_definition_label_names != expected_custom_definition_label_names {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition label names mismatch: expected {}, found {}",
                    expected_custom_definition_label_names, self.custom_definition_label_names
                ),
            ));
        }
        let expected_custom_definition_ayanamsa_labels =
            profile.custom_definition_ayanamsa_labels();
        if self.custom_definition_ayanamsa_label_count
            != expected_custom_definition_ayanamsa_labels.len()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition ayanamsa label count mismatch: expected {}, found {}",
                    expected_custom_definition_ayanamsa_labels.len(), self.custom_definition_ayanamsa_label_count
                ),
            ));
        }
        let expected_custom_definition_ayanamsa_label_names =
            expected_custom_definition_ayanamsa_labels.join(", ");
        if self.custom_definition_ayanamsa_label_names
            != expected_custom_definition_ayanamsa_label_names
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition ayanamsa label names mismatch: expected {}, found {}",
                    expected_custom_definition_ayanamsa_label_names, self.custom_definition_ayanamsa_label_names
                ),
            ));
        }

        if self.release_note_count != profile.release_notes.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release note count mismatch: expected {}, found {}",
                    profile.release_notes.len(), self.release_note_count
                ),
            ));
        }
        if self.validation_reference_point_count != profile.validation_reference_points.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary validation reference point count mismatch: expected {}, found {}",
                    profile.validation_reference_points.len(), self.validation_reference_point_count
                ),
            ));
        }
        let custom_definition_labels_checked =
            verify_custom_definition_labels(profile.custom_definition_labels)?;
        if self.custom_definition_label_count != custom_definition_labels_checked {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary custom-definition label count mismatch: expected {}, found {}",
                    custom_definition_labels_checked, self.custom_definition_label_count
                ),
            ));
        }
        if self.compatibility_caveat_count != profile.known_gaps.len() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary compatibility caveat count mismatch: expected {}, found {}",
                    profile.known_gaps.len(), self.compatibility_caveat_count
                ),
            ));
        }

        verify_profile_text_section("target-house-scope", profile.target_house_scope)?;
        verify_profile_text_section("target-ayanamsa-scope", profile.target_ayanamsa_scope)?;
        verify_profile_text_section("release-note", profile.release_notes)?;
        verify_profile_text_section(
            "validation-reference-point",
            profile.validation_reference_points,
        )?;
        verify_profile_text_section("compatibility-caveat", profile.known_gaps)?;
        verify_profile_text_sections_are_disjoint(&[
            ("target-house-scope", profile.target_house_scope),
            ("target-ayanamsa-scope", profile.target_ayanamsa_scope),
            ("release-note", profile.release_notes),
            (
                "validation-reference-point",
                profile.validation_reference_points,
            ),
            ("compatibility-caveat", profile.known_gaps),
        ])?;

        Ok(())
    }

    /// Returns the validated compact summary line for the verification posture.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Renders the verification summary as compact release-facing text.
    pub fn summary_line(&self) -> String {
        let mut text = String::new();
        text.push_str("Compatibility profile verification\n");
        text.push_str("Profile: ");
        text.push_str(&self.profile_id);
        text.push('\n');
        text.push_str("House systems verified: ");
        text.push_str(&self.house_system_descriptor_count.to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.house_system_label_count.to_string());
        text.push_str(" labels\n");
        text.push_str("House code aliases verified: ");
        text.push_str(&self.house_code_alias_count.to_string());
        text.push_str(" short-form labels\n");
        text.push_str("House code aliases: ");
        text.push_str(&self.house_code_aliases_summary);
        text.push('\n');
        text.push_str("Alias uniqueness checks: house=");
        text.push_str(&self.house_system_alias_count.to_string());
        text.push_str(" aliases, ayanamsa=");
        text.push_str(&self.ayanamsa_alias_count.to_string());
        text.push_str(" aliases; exact and case-insensitive labels verified\n");
        text.push_str("Latitude-sensitive house systems verified: ");
        text.push_str(&self.latitude_sensitive_house_systems.len().to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.latitude_sensitive_house_systems.len().to_string());
        text.push_str(" labels");
        if !self.latitude_sensitive_house_systems.is_empty() {
            text.push_str(" (");
            text.push_str(&self.latitude_sensitive_house_systems.join(", "));
            text.push(')');
        } else {
            text.push_str(" (none)");
        }
        text.push('\n');
        text.push_str("Ayanamsas verified: ");
        text.push_str(&self.ayanamsa_descriptor_count.to_string());
        text.push_str(" descriptors, ");
        text.push_str(&self.ayanamsa_label_count.to_string());
        text.push_str(" labels\n");
        text.push_str("Baseline/release slices: ");
        text.push_str(&self.baseline_house_system_count.to_string());
        text.push_str(" house baseline + ");
        text.push_str(&self.release_house_system_count.to_string());
        text.push_str(" house release, ");
        text.push_str(&self.baseline_ayanamsa_count.to_string());
        text.push_str(" ayanamsa baseline + ");
        text.push_str(&self.release_ayanamsa_count.to_string());
        text.push_str(" ayanamsa release\n");
        text.push_str("Release-specific house-system canonical names verified: ");
        text.push_str(&self.release_house_canonical_names);
        text.push('\n');
        text.push_str("Release-specific ayanamsa canonical names verified: ");
        text.push_str(&self.release_ayanamsa_canonical_names);
        text.push('\n');
        text.push_str("House formula families verified: ");
        text.push_str(&self.house_formula_family_names);
        text.push('\n');
        text.push_str(&self.release_posture);
        text.push('\n');
        text.push_str("Ayanamsa reference metadata verified: ");
        text.push_str(&self.ayanamsa_metadata_count.to_string());
        text.push_str(" descriptors with epoch/offset metadata, ");
        text.push_str(&self.ayanamsa_metadata_gap_count.to_string());
        text.push_str(" metadata gaps\n");
        text.push_str(&current_compatibility_profile().catalog_posture_summary_line());
        text.push('\n');
        text.push_str("Release posture: baseline milestone preserved, release additions explicit, custom definitions tracked, caveats documented\n");
        text.push_str("Release notes documented: ");
        text.push_str(&self.release_note_count.to_string());
        text.push_str(" entries\n");
        text.push_str("Validation reference points documented: ");
        text.push_str(&self.validation_reference_point_count.to_string());
        text.push_str(" entries\n");
        text.push_str("Custom-definition labels verified: ");
        text.push_str(&self.custom_definition_label_count.to_string());
        text.push_str(" labels, all remain custom-definition territory\n");
        text.push_str("Custom-definition label names verified: ");
        text.push_str(&self.custom_definition_label_names);
        text.push('\n');
        text.push_str("Custom-definition ayanamsa labels verified: ");
        text.push_str(&self.custom_definition_ayanamsa_label_count.to_string());
        text.push_str(" labels, all remain custom-definition territory\n");
        text.push_str("Custom-definition ayanamsa label names verified: ");
        text.push_str(&self.custom_definition_ayanamsa_label_names);
        text.push('\n');
        text.push_str("Compatibility caveats documented: ");
        text.push_str(&self.compatibility_caveat_count.to_string());
        text.push('\n');
        text
    }
}

impl fmt::Display for CompatibilityProfileVerificationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured compatibility-profile verification posture.
pub fn compatibility_profile_verification_summary(
) -> Result<CompatibilityProfileVerificationSummary, EphemerisError> {
    let profile = current_compatibility_profile();
    let release_profiles = validated_release_profile_identifiers_for_report().map_err(|error| {
        EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile verification summary release profile identifiers invalid: {error}"
            ),
        )
    })?;

    if profile.profile_id != release_profiles.compatibility_profile_id {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile identifier mismatch: profile {} does not match release profile {}",
                profile.profile_id, release_profiles.compatibility_profile_id
            ),
        ));
    }

    ensure_profile_slice_matches(
        "house-system catalog",
        profile.house_systems,
        built_in_house_systems(),
    )?;
    ensure_profile_slice_matches(
        "baseline house-system slice",
        profile.baseline_house_systems,
        baseline_house_systems(),
    )?;
    ensure_profile_slice_matches(
        "release house-system slice",
        profile.release_house_systems,
        release_house_systems(),
    )?;
    ensure_profile_slice_matches("ayanamsa catalog", profile.ayanamsas, built_in_ayanamsas())?;
    ensure_profile_slice_matches(
        "baseline ayanamsa slice",
        profile.baseline_ayanamsas,
        baseline_ayanamsas(),
    )?;
    ensure_profile_slice_matches(
        "release ayanamsa slice",
        profile.release_ayanamsas,
        release_ayanamsas(),
    )?;

    verify_profile_catalog_partitions_are_disjoint(
        "house-system",
        profile.baseline_house_systems,
        profile.release_house_systems,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )?;
    verify_profile_catalog_partitions_are_disjoint(
        "ayanamsa",
        profile.baseline_ayanamsas,
        profile.release_ayanamsas,
        |entry| entry.canonical_name,
        |entry| entry.aliases,
    )?;

    let house_labels_checked = verify_house_system_aliases(profile.house_systems)?;
    let ayanamsa_labels_checked = verify_ayanamsa_aliases(profile.ayanamsas)?;
    let custom_definition_labels_checked =
        verify_custom_definition_labels(profile.custom_definition_labels)?;
    verify_profile_text_section("release-note", profile.release_notes)?;
    verify_profile_text_section(
        "validation-reference-point",
        profile.validation_reference_points,
    )?;
    verify_profile_text_section("compatibility-caveat", profile.known_gaps)?;
    verify_profile_text_sections_are_disjoint(&[
        ("release-note", profile.release_notes),
        (
            "validation-reference-point",
            profile.validation_reference_points,
        ),
        ("compatibility-caveat", profile.known_gaps),
    ])?;

    let release_house_names =
        profile.validated_release_house_system_canonical_names_summary_line().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release house-system canonical names invalid: {error}"
                ),
            )
        })?;
    let release_ayanamsa_names =
        profile.validated_release_ayanamsa_canonical_names_summary_line().map_err(|error| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile verification summary release ayanamsa canonical names invalid: {error}"
                ),
            )
        })?;
    let house_formula_family_names = profile.house_formula_family_names().join(", ");
    let ayanamsa_metadata_count = profile
        .ayanamsas
        .iter()
        .filter(|entry| entry.has_sidereal_metadata())
        .count();
    let custom_definition_ayanamsa_labels = profile.custom_definition_ayanamsa_labels();

    Ok(CompatibilityProfileVerificationSummary {
        profile_id: release_profiles.compatibility_profile_id.to_string(),
        house_system_descriptor_count: profile.house_systems.len(),
        house_system_label_count: house_labels_checked,
        house_code_alias_count: profile.house_code_alias_count(),
        house_system_alias_count: house_labels_checked - profile.house_systems.len(),
        house_code_aliases_summary: profile.house_code_aliases_summary_line(),
        latitude_sensitive_house_systems: profile
            .latitude_sensitive_house_systems()
            .into_iter()
            .map(|label| label.to_string())
            .collect(),
        ayanamsa_descriptor_count: profile.ayanamsas.len(),
        ayanamsa_label_count: ayanamsa_labels_checked,
        ayanamsa_alias_count: ayanamsa_labels_checked - profile.ayanamsas.len(),
        baseline_house_system_count: profile.baseline_house_systems.len(),
        release_house_system_count: profile.release_house_systems.len(),
        baseline_ayanamsa_count: profile.baseline_ayanamsas.len(),
        release_ayanamsa_count: profile.release_ayanamsas.len(),
        release_house_canonical_names: release_house_names,
        release_ayanamsa_canonical_names: release_ayanamsa_names,
        house_formula_family_names,
        release_posture: RELEASE_POSTURE_SUMMARY.to_string(),
        ayanamsa_metadata_count,
        ayanamsa_metadata_gap_count: profile.ayanamsas.len() - ayanamsa_metadata_count,
        release_note_count: profile.release_notes.len(),
        validation_reference_point_count: profile.validation_reference_points.len(),
        custom_definition_label_count: custom_definition_labels_checked,
        custom_definition_label_names: profile.custom_definition_labels.join(", "),
        custom_definition_ayanamsa_label_count: custom_definition_ayanamsa_labels.len(),
        custom_definition_ayanamsa_label_names: custom_definition_ayanamsa_labels.join(", "),
        compatibility_caveat_count: profile.known_gaps.len(),
    })
}

/// Verifies that the release compatibility profile stays synchronized with the
/// canonical house-system and ayanamsa catalogs.
pub fn verify_compatibility_profile() -> Result<String, EphemerisError> {
    if let Err(violations) = crate::claims::audit_compat_claims() {
        let messages: Vec<String> = violations.iter().map(|v| v.to_string()).collect();
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility overclaim audit failed:\n{}",
                messages.join("\n")
            ),
        ));
    }
    compatibility_profile_verification_summary()?.validated_summary_line()
}

fn ensure_profile_slice_matches<T>(
    label: &str,
    actual: &[T],
    expected: &[T],
) -> Result<(), EphemerisError>
where
    T: PartialEq + fmt::Debug,
{
    if actual != expected {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {label} mismatch: expected {} entries, found {}",
                expected.len(),
                actual.len()
            ),
        ));
    }

    Ok(())
}

pub(crate) fn verify_profile_catalog_partitions_are_disjoint<T>(
    catalog_label: &str,
    baseline_entries: &[T],
    release_entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
    aliases: impl Fn(&T) -> &'static [&'static str],
) -> Result<(), EphemerisError> {
    let mut baseline_labels = BTreeSet::new();

    for entry in baseline_entries {
        baseline_labels.insert(canonical_name(entry).trim().to_ascii_lowercase());
        for alias in aliases(entry) {
            baseline_labels.insert(alias.trim().to_ascii_lowercase());
        }
    }

    for entry in release_entries {
        for label in std::iter::once(canonical_name(entry)).chain(aliases(entry).iter().copied()) {
            let normalized_label = label.trim().to_ascii_lowercase();
            if baseline_labels.contains(&normalized_label) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile {catalog_label} baseline and release slices overlap on label '{label}'",
                    ),
                ));
            }
        }
    }

    Ok(())
}

pub(crate) fn verify_house_system_aliases(
    entries: &[pleiades_houses::HouseSystemDescriptor],
) -> Result<usize, EphemerisError> {
    if let Err(error) = validate_house_catalog() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("house catalog validation failed: {error}"),
        ));
    }
    if let Err(error) = pleiades_houses::validate_house_system_code_aliases() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("house-code alias validation failed: {error}"),
        ));
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("house-system", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label(
            "house-system",
            entry.canonical_name,
            entry.canonical_name,
            &mut seen_labels,
            &mut seen_labels_case_insensitive,
        )?;
        if resolve_house_system(entry.canonical_name) != Some(entry.system.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile house-system alias mismatch: canonical label '{}' should resolve to {}",
                    entry.canonical_name, entry.system
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            if has_surrounding_whitespace(alias) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile house-system descriptor '{}' contains surrounding whitespace in its label",
                        alias
                    ),
                ));
            }
            ensure_unique_profile_label(
                "house-system",
                alias,
                entry.canonical_name,
                &mut seen_labels,
                &mut seen_labels_case_insensitive,
            )?;
            if resolve_house_system(alias) != Some(entry.system.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile house-system alias mismatch: alias '{}' should resolve to {}",
                        alias, entry.system
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

fn format_ayanamsa_label(ayanamsa: &pleiades_core::Ayanamsa) -> String {
    pleiades_ayanamsa::descriptor(ayanamsa)
        .map(|descriptor| descriptor.canonical_name.to_owned())
        .unwrap_or_else(|| ayanamsa.to_string())
}

pub(crate) fn verify_ayanamsa_aliases(
    entries: &[pleiades_ayanamsa::AyanamsaDescriptor],
) -> Result<usize, EphemerisError> {
    if let Err(error) = validate_ayanamsa_catalog() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("ayanamsa catalog validation failed: {error}"),
        ));
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();
    let mut seen_labels_case_insensitive = BTreeMap::new();

    for entry in entries {
        ensure_profile_descriptor_metadata("ayanamsa", entry.canonical_name, entry.notes)?;

        labels_checked += 1;
        ensure_unique_profile_label(
            "ayanamsa",
            entry.canonical_name,
            entry.canonical_name,
            &mut seen_labels,
            &mut seen_labels_case_insensitive,
        )?;
        if resolve_ayanamsa(entry.canonical_name) != Some(entry.ayanamsa.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile ayanamsa alias mismatch: canonical label '{}' should resolve to {}",
                    entry.canonical_name,
                    format_ayanamsa_label(&entry.ayanamsa)
                ),
            ));
        }

        for alias in entry.aliases {
            labels_checked += 1;
            if has_surrounding_whitespace(alias) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile ayanamsa descriptor '{}' contains surrounding whitespace in its label",
                        alias
                    ),
                ));
            }
            ensure_unique_profile_label(
                "ayanamsa",
                alias,
                entry.canonical_name,
                &mut seen_labels,
                &mut seen_labels_case_insensitive,
            )?;
            if resolve_ayanamsa(alias) != Some(entry.ayanamsa.clone()) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile ayanamsa alias mismatch: alias '{}' should resolve to {}",
                        alias,
                        format_ayanamsa_label(&entry.ayanamsa)
                    ),
                ));
            }
        }
    }

    Ok(labels_checked)
}

#[cfg(test)]
pub(crate) const INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS: &[&str] = &[
    "Babylonian (House)",
    "Babylonian (Sissy)",
    "Babylonian (True Geoc)",
    "Babylonian (True Topc)",
    "Babylonian (True Obs)",
    "Babylonian (House Obs)",
];

#[cfg(test)]
pub(crate) fn is_intentional_custom_definition_ayanamsa_homograph(label: &str) -> bool {
    INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS.contains(&label)
}

pub(crate) fn verify_custom_definition_labels(
    labels: &[&'static str],
) -> Result<usize, EphemerisError> {
    validate_custom_definition_labels(labels)
        .map_err(|error| EphemerisError::new(EphemerisErrorKind::InvalidRequest, error.to_string()))
}

pub(crate) fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
}

pub(crate) fn verify_profile_text_section(
    section_label: &str,
    entries: &[&str],
) -> Result<usize, EphemerisError> {
    let mut entries_checked = 0usize;
    let mut seen_entries = BTreeSet::new();
    let mut seen_entries_case_insensitive = BTreeMap::new();

    for entry in entries {
        entries_checked += 1;
        if entry.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile {section_label} entry is blank"),
            ));
        }

        if has_surrounding_whitespace(entry) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {section_label} entry '{}' contains surrounding whitespace",
                    entry
                ),
            ));
        }

        let normalized_entry = entry.trim().to_string();
        if !seen_entries.insert(normalized_entry.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("compatibility profile {section_label} entries are not unique: duplicate entry '{}'", entry),
            ));
        }

        let normalized_case_insensitive = normalized_entry.to_ascii_lowercase();
        if let Some(existing_entry) =
            seen_entries_case_insensitive.get(&normalized_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {section_label} entries are not unique ignoring case: duplicate entry '{}' conflicts with '{}'",
                    entry, existing_entry
                ),
            ));
        }
        seen_entries_case_insensitive.insert(normalized_case_insensitive, normalized_entry);
    }

    Ok(entries_checked)
}

pub(crate) fn verify_profile_text_sections_are_disjoint(
    sections: &[(&'static str, &'static [&'static str])],
) -> Result<(), EphemerisError> {
    let mut seen_entries = BTreeMap::<String, &'static str>::new();
    let mut seen_entries_case_insensitive = BTreeMap::<String, (&'static str, String)>::new();

    for (section_label, entries) in sections {
        for entry in *entries {
            if entry.trim().is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("compatibility profile {section_label} entry is blank"),
                ));
            }

            if has_surrounding_whitespace(entry) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile {section_label} entry '{}' contains surrounding whitespace",
                        entry
                    ),
                ));
            }

            let normalized_entry = entry.trim().to_string();
            if let Some(existing_section) = seen_entries.get(&normalized_entry) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile text sections are not unique: duplicate entry '{}' appears in both {} and {}",
                        entry, existing_section, section_label
                    ),
                ));
            }

            let normalized_case_insensitive = normalized_entry.to_ascii_lowercase();
            if let Some((existing_section, existing_entry)) =
                seen_entries_case_insensitive.get(&normalized_case_insensitive)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "compatibility profile text sections are not unique ignoring case: duplicate entry '{}' appears in both {} and {} (conflicts with '{}')",
                        entry, existing_section, section_label, existing_entry
                    ),
                ));
            }

            seen_entries.insert(normalized_entry.clone(), section_label);
            seen_entries_case_insensitive.insert(
                normalized_case_insensitive,
                (section_label, normalized_entry),
            );
        }
    }

    Ok(())
}

pub(crate) fn ensure_profile_descriptor_metadata(
    catalog_label: &str,
    canonical_name: &str,
    notes: &str,
) -> Result<(), EphemerisError> {
    if canonical_name.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility profile {catalog_label} descriptor is missing a canonical name"),
        ));
    }

    if has_surrounding_whitespace(canonical_name) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{canonical_name}' contains surrounding whitespace in its canonical name",
            ),
        ));
    }

    if notes.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' is missing notes metadata",
                canonical_name
            ),
        ));
    }

    if has_surrounding_whitespace(notes) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' contains surrounding whitespace in its notes metadata",
                canonical_name
            ),
        ));
    }

    Ok(())
}

pub(crate) fn ensure_unique_profile_label(
    catalog_label: &str,
    label: &str,
    item_identity: &str,
    seen_labels: &mut BTreeSet<String>,
    seen_labels_case_insensitive: &mut BTreeMap<String, String>,
) -> Result<(), EphemerisError> {
    if label.trim().is_empty() {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility profile {catalog_label} descriptor contains a blank label"),
        ));
    }

    if has_surrounding_whitespace(label) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} descriptor '{}' contains surrounding whitespace in its label",
                item_identity
            ),
        ));
    }

    let normalized = label.trim().to_string();
    if !seen_labels.insert(normalized.clone()) {
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!(
                "compatibility profile {catalog_label} labels are not unique: duplicate label '{}'",
                label
            ),
        ));
    }

    let normalized_case_insensitive = normalized.to_ascii_lowercase();
    match seen_labels_case_insensitive.get(&normalized_case_insensitive) {
        Some(existing_identity) if existing_identity == item_identity => {}
        Some(_) => {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "compatibility profile {catalog_label} labels are not unique ignoring case: duplicate label '{}'",
                    label
                ),
            ));
        }
        None => {
            seen_labels_case_insensitive
                .insert(normalized_case_insensitive, item_identity.to_string());
        }
    }

    Ok(())
}

fn validation_reference_point_summary(point: &str) -> String {
    if point.contains("stage-4 validation corpus") {
        "stage-4 validation corpus".to_string()
    } else {
        point.to_string()
    }
}

pub(crate) fn summarize_validation_reference_points(points: &[&str]) -> String {
    match points {
        [] => "0".to_string(),
        [point] => format!("1 ({})", validation_reference_point_summary(point)),
        _ => format!(
            "{} ({})",
            points.len(),
            points
                .iter()
                .map(|point| validation_reference_point_summary(point))
                .collect::<Vec<_>>()
                .join("; ")
        ),
    }
}
