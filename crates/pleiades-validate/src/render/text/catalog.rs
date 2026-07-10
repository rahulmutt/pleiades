//! Catalog, house, ayanamsa, and release name/scope summary text.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::OnceLock;

use crate::*;

/// Renders the compact compatibility-profile summary used by release tooling.
pub fn render_compatibility_profile_summary() -> String {
    render_compatibility_profile_summary_text()
}

/// Renders the compact compatibility-caveats summary used by release tooling.
pub fn render_compatibility_caveats_summary() -> String {
    render_compatibility_caveats_summary_text()
}

/// Renders the compact latitude-sensitive house failure modes summary used by release tooling.
pub fn render_house_latitude_sensitive_failure_modes_summary() -> String {
    format_latitude_sensitive_house_failure_modes_for_report()
}

/// Renders the compact known-gaps summary used by release tooling.
pub fn render_known_gaps_summary() -> String {
    render_known_gaps_summary_text()
}

/// Renders the compact compatibility catalog inventory summary used by release tooling.
pub fn render_catalog_inventory_summary() -> String {
    render_catalog_inventory_summary_text()
}

pub(crate) fn render_catalog_inventory_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| match validated_catalog_inventory_summary_for_report() {
            Ok(summary) => summary,
            Err(error) => format!("Compatibility catalog inventory unavailable ({error})"),
        })
        .clone()
}

pub(crate) fn render_known_gaps_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(
            || match current_compatibility_profile().validated_known_gaps_summary_line() {
                Ok(summary) => format!("Known gaps: {summary}"),
                Err(error) => format!("Known gaps unavailable ({error})"),
            },
        )
        .clone()
}

/// Renders the compact compatibility catalog posture summary used by release tooling.
pub fn render_catalog_posture_summary() -> String {
    render_catalog_posture_summary_text()
}

pub(crate) fn render_catalog_posture_summary_text() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            match current_compatibility_profile().validated_catalog_posture_summary_line() {
                Ok(summary) => summary,
                Err(error) => format!("Compatibility catalog posture unavailable ({error})"),
            }
        })
        .clone()
}

/// Renders the compact custom-definition ayanamsa label summary used by release tooling.
pub fn render_custom_definition_ayanamsa_labels_summary() -> String {
    format_custom_definition_ayanamsa_labels_for_report()
}

/// Renders the compact release-specific house-system canonical-name summary used by release tooling.
pub fn render_release_house_system_canonical_names_summary() -> String {
    format_release_house_system_canonical_names_for_report()
}

/// Renders the compact release-specific ayanamsa canonical-name summary used by release tooling.
pub fn render_release_ayanamsa_canonical_names_summary() -> String {
    format_release_ayanamsa_canonical_names_for_report()
}

/// Renders the compact ayanamsa audit summary used by release tooling.
pub fn render_ayanamsa_audit_summary() -> String {
    format_ayanamsa_audit_for_report()
}

/// Renders the compact target house-system scope summary used by release tooling.
pub fn render_target_house_scope_summary() -> String {
    match current_compatibility_profile().validated_target_house_scope_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target house scope unavailable ({error})"),
    }
}

/// Renders the compact target ayanamsa scope summary used by release tooling.
pub fn render_target_ayanamsa_scope_summary() -> String {
    match current_compatibility_profile().validated_target_ayanamsa_scope_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("Compatibility profile target ayanamsa scope unavailable ({error})"),
    }
}

/// Renders the release notes used by release tooling.
pub fn render_release_notes() -> String {
    render_release_notes_text()
}

/// Renders the compact release notes summary used by release tooling.
pub fn render_release_notes_summary() -> String {
    render_release_notes_summary_text()
}

/// Renders the release checklist used by release tooling.
pub fn render_release_checklist() -> String {
    render_release_checklist_text()
}

/// Renders the compact release checklist summary used by release tooling.
pub fn render_release_checklist_summary() -> String {
    render_release_checklist_summary_text()
}

/// Renders the compact release summary used by release tooling.
pub fn render_release_summary() -> String {
    render_release_summary_text()
}

/// Renders the compact Delta T policy summary used by validation and release tooling.
pub fn render_delta_t_policy_summary() -> String {
    render_delta_t_policy_summary_text()
}

/// Renders the compact request-policy summary used by validation and release tooling.
pub fn render_request_policy_summary() -> String {
    render_request_policy_summary_text()
}

/// Renders the compact request-surface inventory used by validation and release tooling.
pub fn render_request_surface_summary() -> String {
    render_request_surface_summary_text()
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaReferenceOffsetExample {
    pub(crate) canonical_name: &'static str,
    pub(crate) epoch: JulianDay,
    pub(crate) offset_degrees: Angle,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaReferenceOffsetsSummary {
    pub(crate) examples: Vec<AyanamsaReferenceOffsetExample>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaProvenanceExample {
    pub(crate) canonical_name: &'static str,
    pub(crate) provenance_note: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AyanamsaProvenanceSummary {
    pub(crate) examples: Vec<AyanamsaProvenanceExample>,
}

impl AyanamsaReferenceOffsetsSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa reference offsets",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative zero-point examples: 0 (none)".to_string(),
            [single] => format!(
                "representative zero-point examples: 1 ({}: epoch={}; offset={})",
                single.canonical_name, single.epoch, single.offset_degrees
            ),
            _ => format!(
                "representative zero-point examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{}: epoch={}; offset={}",
                        example.canonical_name, example.epoch, example.offset_degrees
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }
}

impl fmt::Display for AyanamsaReferenceOffsetsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl AyanamsaProvenanceSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence(
            "ayanamsa provenance examples",
            self.examples.iter().map(|example| example.canonical_name),
        )?;

        for example in &self.examples {
            if example.provenance_note.trim().is_empty()
                || example.provenance_note.contains('\n')
                || example.provenance_note.contains('\r')
                || has_surrounding_whitespace(example.provenance_note)
            {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "ayanamsa provenance example `{}` has an unnormalized provenance note",
                        example.canonical_name
                    ),
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.examples.as_slice() {
            [] => "representative provenance examples: 0 (none)".to_string(),
            [single] => format!(
                "representative provenance examples: 1 ({} — {})",
                single.canonical_name, single.provenance_note
            ),
            _ => format!(
                "representative provenance examples: {}",
                self.examples
                    .iter()
                    .map(|example| format!(
                        "{} — {}",
                        example.canonical_name, example.provenance_note
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        }
    }

    pub(crate) fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaProvenanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn summarize_ayanamsa_reference_offsets(
) -> Result<AyanamsaReferenceOffsetsSummary, EphemerisError> {
    let samples = pleiades_ayanamsa::reference_offset_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa reference offsets sample `{sample}` is unavailable"),
            )
        })?;
        let epoch = descriptor.epoch.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference epoch",
                    descriptor.canonical_name
                ),
            )
        })?;
        let offset_degrees = descriptor.offset_degrees.ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "ayanamsa reference offsets sample `{}` is missing its reference offset",
                    descriptor.canonical_name
                ),
            )
        })?;

        examples.push(AyanamsaReferenceOffsetExample {
            canonical_name: descriptor.canonical_name,
            epoch,
            offset_degrees,
        });
    }

    let summary = AyanamsaReferenceOffsetsSummary { examples };
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn validated_ayanamsa_reference_offsets_summary_for_report(
    summary: &AyanamsaReferenceOffsetsSummary,
) -> Result<String, EphemerisError> {
    summary.validate()?;
    Ok(summary.to_string())
}

pub(crate) fn format_ayanamsa_reference_offsets_for_report() -> String {
    match summarize_ayanamsa_reference_offsets() {
        Ok(summary) => match validated_ayanamsa_reference_offsets_summary_for_report(&summary) {
            Ok(summary) => format!("Ayanamsa reference offsets: {summary}"),
            Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa reference offsets: unavailable ({error})"),
    }
}

pub(crate) fn summarize_ayanamsa_provenance() -> Result<AyanamsaProvenanceSummary, EphemerisError> {
    let samples = pleiades_ayanamsa::provenance_sample_ayanamsas();

    let mut examples = Vec::with_capacity(samples.len());
    for sample in samples {
        let descriptor = descriptor(sample).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("ayanamsa provenance sample `{sample}` is unavailable"),
            )
        })?;

        examples.push(AyanamsaProvenanceExample {
            canonical_name: descriptor.canonical_name,
            provenance_note: descriptor.notes,
        });
    }

    let summary = AyanamsaProvenanceSummary { examples };
    summary.validate()?;
    Ok(summary)
}

pub(crate) fn format_ayanamsa_catalog_validation_for_report() -> String {
    match ayanamsa_catalog_validation_summary().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa catalog validation: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_metadata_coverage_for_report() -> String {
    match metadata_coverage().validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("ayanamsa sidereal metadata: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_provenance_for_report() -> String {
    match summarize_ayanamsa_provenance() {
        Ok(summary) => match summary.validated_summary_line() {
            Ok(summary) => format!("Ayanamsa provenance: {summary}"),
            Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
        },
        Err(error) => format!("Ayanamsa provenance: unavailable ({error})"),
    }
}

pub(crate) fn format_ayanamsa_audit_for_report() -> String {
    format!(
        "Ayanamsa audit: {}; {}; {}; {}",
        format_ayanamsa_catalog_validation_for_report(),
        format_ayanamsa_metadata_coverage_for_report(),
        format_ayanamsa_reference_offsets_for_report(),
        format_ayanamsa_provenance_for_report(),
    )
}

pub(crate) fn format_house_code_aliases_for_report() -> String {
    match pleiades_houses::validated_house_system_code_aliases_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("house-code aliases unavailable ({error})"),
    }
}

pub(crate) fn format_house_formula_families_for_report() -> String {
    match current_compatibility_profile().validated_house_formula_families_summary_line() {
        Ok(summary) => format!("House formula families: {summary}"),
        Err(error) => format!("house formula families unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_systems_for_report() -> String {
    match current_compatibility_profile().validated_latitude_sensitive_house_systems_summary_line()
    {
        Ok(summary) => format!("Latitude-sensitive house systems: {summary}"),
        Err(error) => format!("Latitude-sensitive house systems unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_constraints_for_report() -> String {
    match current_compatibility_profile()
        .validated_latitude_sensitive_house_constraints_summary_line()
    {
        Ok(summary) => format!("Latitude-sensitive house constraints: {summary}"),
        Err(error) => format!("Latitude-sensitive house constraints unavailable ({error})"),
    }
}

pub(crate) fn format_latitude_sensitive_house_failure_modes_for_report() -> String {
    let profile = current_compatibility_profile();
    match profile.validate() {
        Ok(()) => format!(
            "Latitude-sensitive house failure modes: {}",
            profile.latitude_sensitive_house_failure_modes_summary_line()
        ),
        Err(error) => format!("Latitude-sensitive house failure modes unavailable ({error})"),
    }
}

pub(crate) fn format_custom_definition_ayanamsa_labels_for_report() -> String {
    match current_compatibility_profile().validated_custom_definition_ayanamsa_labels_summary_line()
    {
        Ok(summary) => summary,
        Err(error) => format!("custom-definition ayanamsa labels unavailable ({error})"),
    }
}

pub(crate) fn validated_release_house_system_canonical_names_for_report() -> Result<String, String>
{
    current_compatibility_profile()
        .validated_release_house_system_canonical_names_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn validated_release_ayanamsa_canonical_names_for_report() -> Result<String, String> {
    current_compatibility_profile()
        .validated_release_ayanamsa_canonical_names_summary_line()
        .map_err(|error| error.to_string())
}

pub(crate) fn format_release_house_system_canonical_names_for_report() -> String {
    match validated_release_house_system_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific house-system canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific house-system canonical names unavailable ({error})")
        }
    }
}

pub(crate) fn format_release_ayanamsa_canonical_names_for_report() -> String {
    match validated_release_ayanamsa_canonical_names_for_report() {
        Ok(summary) => format!("Release-specific ayanamsa canonical names: {summary}"),
        Err(error) => {
            format!("Release-specific ayanamsa canonical names unavailable ({error})")
        }
    }
}

pub(crate) fn validate_name_sequence<'a, I>(
    section_label: &'static str,
    names: I,
) -> Result<(), EphemerisError>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut seen_names = BTreeSet::new();
    let mut seen_names_case_insensitive = BTreeMap::new();

    for name in names {
        if name.trim().is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a blank name"),
            ));
        }

        if has_surrounding_whitespace(name) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} entry '{name}' contains surrounding whitespace"),
            ));
        }

        let normalized_name = name.trim().to_string();
        if !seen_names.insert(normalized_name.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!("{section_label} contains a duplicate name '{name}'"),
            ));
        }

        let normalized_name_case_insensitive = normalized_name.to_ascii_lowercase();
        if let Some(existing_name) =
            seen_names_case_insensitive.get(&normalized_name_case_insensitive)
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "{section_label} contains a case-insensitive duplicate name '{name}' that conflicts with '{existing_name}'"
                ),
            ));
        }
        seen_names_case_insensitive.insert(normalized_name_case_insensitive, normalized_name);
    }

    Ok(())
}

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DescriptorNamesSummary {
    pub(crate) names: Vec<&'static str>,
}

#[cfg(test)]
impl DescriptorNamesSummary {
    pub(crate) fn validate(&self) -> Result<(), EphemerisError> {
        validate_name_sequence("descriptor-name summary", self.names.iter().copied())
    }

    pub(crate) fn summary_line(&self) -> String {
        match self.names.as_slice() {
            [] => "0 (none)".to_string(),
            [single] => format!("1 ({single})"),
            _ => format!("{} ({})", self.names.len(), self.names.join(", ")),
        }
    }
}

#[cfg(test)]
impl fmt::Display for DescriptorNamesSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[cfg(test)]
pub(crate) fn summarize_descriptor_names<T>(
    entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> DescriptorNamesSummary {
    DescriptorNamesSummary {
        names: entries.iter().map(canonical_name).collect::<Vec<_>>(),
    }
}
