#![forbid(unsafe_code)]

use core::fmt;

use pleiades_ayanamsa::{metadata_coverage, AyanamsaDescriptor};
use pleiades_houses::HouseSystemDescriptor;

use super::aliases::{ayanamsa_source_label_aliases, house_source_label_aliases};
use super::profile::CompatibilityProfile;
use super::validation::{
    validate_ayanamsa_descriptor_metadata, validate_catalog_coverage,
    validate_catalog_coverage_ayanamsa, validate_catalog_coverage_labels,
    validate_catalog_label_uniqueness, validate_catalog_partitions_are_disjoint,
    validate_catalog_partitions_are_disjoint_ayanamsa, validate_catalogs_are_disjoint,
    validate_custom_definition_ayanamsa_coverage, validate_house_descriptor_metadata,
    validate_house_formula_families, validate_profile_identifier, validate_profile_summary,
    validate_profile_text_section, validate_profile_text_sections_are_disjoint,
    CompatibilityProfileValidationError,
};

pub(super) fn format_canonical_name_summary(names: &[&'static str]) -> String {
    match names {
        [] => "0 (none)".to_string(),
        [single] => format!("1 ({single})"),
        _ => format!("{} ({})", names.len(), names.join(", ")),
    }
}

pub(super) fn format_string_summary(items: &[String]) -> String {
    match items {
        [] => "0 (none)".to_string(),
        [single] => format!("1 ({single})"),
        _ => format!("{} ({})", items.len(), items.join(", ")),
    }
}

impl CompatibilityProfile {
    /// Validates the profile's internal release-facing metadata.
    pub fn validate(&self) -> Result<(), CompatibilityProfileValidationError> {
        validate_profile_identifier(self.profile_id)?;
        validate_profile_summary(self.summary)?;
        validate_profile_text_section("target-house-scope", self.target_house_scope)?;
        validate_profile_text_section("target-ayanamsa-scope", self.target_ayanamsa_scope)?;
        validate_profile_text_section("release-note", self.release_notes)?;
        validate_profile_text_section(
            "validation-reference-point",
            self.validation_reference_points,
        )?;
        super::validation::validate_custom_definition_labels(self.custom_definition_labels)?;
        validate_custom_definition_ayanamsa_coverage(self.custom_definition_labels)?;
        self.house_code_alias_inventory_summary().validate()?;
        validate_house_descriptor_metadata(self.house_systems)?;
        validate_ayanamsa_descriptor_metadata(self.ayanamsas)?;
        validate_profile_text_section("compatibility-caveat", self.known_gaps)?;
        validate_profile_text_sections_are_disjoint(&[
            ("target-house-scope", self.target_house_scope),
            ("target-ayanamsa-scope", self.target_ayanamsa_scope),
            ("release-note", self.release_notes),
            (
                "validation-reference-point",
                self.validation_reference_points,
            ),
            ("custom-definition", self.custom_definition_labels),
            ("compatibility-caveat", self.known_gaps),
        ])?;
        validate_catalog_label_uniqueness("house-system", self.house_systems)?;
        validate_house_formula_families(self.house_systems)?;
        validate_catalog_label_uniqueness("ayanamsa", self.ayanamsas)?;
        validate_catalogs_are_disjoint(
            "house-system",
            self.house_systems,
            "ayanamsa",
            self.ayanamsas,
        )?;
        validate_catalog_partitions_are_disjoint(
            "house-system",
            self.baseline_house_systems,
            self.release_house_systems,
        )?;
        validate_catalog_partitions_are_disjoint_ayanamsa(
            "ayanamsa",
            self.baseline_ayanamsas,
            self.release_ayanamsas,
        )?;
        validate_catalog_coverage(
            "house-system",
            self.house_systems,
            self.baseline_house_systems,
            self.release_house_systems,
        )?;
        validate_catalog_coverage_labels(
            "house-system",
            self.house_systems,
            self.baseline_house_systems,
            self.release_house_systems,
            |entry| entry.canonical_name,
        )?;
        validate_catalog_coverage_ayanamsa(
            "ayanamsa",
            self.ayanamsas,
            self.baseline_ayanamsas,
            self.release_ayanamsas,
        )?;
        validate_catalog_coverage_labels(
            "ayanamsa",
            self.ayanamsas,
            self.baseline_ayanamsas,
            self.release_ayanamsas,
            |entry| entry.canonical_name,
        )?;
        Ok(())
    }
}

pub(super) fn write_scope_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    lines: &[&'static str],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for line in lines {
        writeln!(f, "- {}", line)?;
    }
    Ok(())
}

/// Trait for catalog entries that carry a canonical name and zero or more aliases.
pub trait AliasProfileEntry {
    fn canonical_name(&self) -> &'static str;
    fn aliases(&self) -> &'static [&'static str];
}

impl AliasProfileEntry for HouseSystemDescriptor {
    fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }
}

impl AliasProfileEntry for AyanamsaDescriptor {
    fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }
}

pub(super) fn write_alias_section<T: AliasProfileEntry>(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    entries: &[T],
) -> fmt::Result {
    let mut has_aliases = false;
    for entry in entries {
        if !entry.aliases().is_empty() {
            has_aliases = true;
            break;
        }
    }

    if !has_aliases {
        return Ok(());
    }

    writeln!(f, "{}", title)?;
    for entry in entries {
        if entry.aliases().is_empty() {
            continue;
        }

        writeln!(
            f,
            "- {} -> {}",
            entry.aliases().join(", "),
            entry.canonical_name()
        )?;
    }
    Ok(())
}

pub(super) fn write_source_label_section<T, F>(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    entries: &[T],
    source_label_aliases: F,
) -> fmt::Result
where
    T: AliasProfileEntry,
    F: Fn(&str) -> &'static [&'static str],
{
    let mut has_source_labels = false;
    for entry in entries {
        if !source_label_aliases(entry.canonical_name()).is_empty() {
            has_source_labels = true;
            break;
        }
    }

    if !has_source_labels {
        return Ok(());
    }

    writeln!(f, "{}", title)?;
    for entry in entries {
        let source_labels = source_label_aliases(entry.canonical_name());
        if source_labels.is_empty() {
            continue;
        }

        writeln!(
            f,
            "- {} -> {}",
            source_labels.join(", "),
            entry.canonical_name()
        )?;
    }
    Ok(())
}

pub(super) fn write_custom_definition_section(
    f: &mut fmt::Formatter<'_>,
    labels: &[&'static str],
    descriptors: &[AyanamsaDescriptor],
) -> fmt::Result {
    writeln!(f, "Custom-definition labels:")?;
    for label in labels {
        if let Some(entry) = descriptors
            .iter()
            .find(|entry| entry.canonical_name.eq_ignore_ascii_case(label))
        {
            writeln!(f, "- {}", entry)?;
        } else {
            writeln!(f, "- {}", label)?;
        }
    }
    Ok(())
}

impl fmt::Display for CompatibilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Err(error) = self.validate() {
            return write!(f, "Compatibility profile unavailable ({error})");
        }

        writeln!(f, "Compatibility profile: {}", self.profile_id)?;
        writeln!(f, "{}", self.summary)?;
        writeln!(f)?;
        write_scope_section(f, "Target compatibility catalog:", self.target_house_scope)?;
        write_scope_section(f, "Target ayanamsa catalog:", self.target_ayanamsa_scope)?;
        writeln!(f)?;
        writeln!(f, "Baseline compatibility milestone:")?;
        writeln!(f, "House systems:")?;
        for entry in self.baseline_house_systems {
            writeln!(f, "- {}", entry)?;
        }
        writeln!(f, "Ayanamsas:")?;
        for entry in self.baseline_ayanamsas {
            writeln!(f, "- {}", entry)?;
        }
        if !self.release_house_systems.is_empty() || !self.release_ayanamsas.is_empty() {
            writeln!(f)?;
            writeln!(f, "Release-specific coverage beyond baseline:")?;
            if !self.release_house_systems.is_empty() {
                writeln!(f, "House systems:")?;
                for entry in self.release_house_systems {
                    writeln!(f, "- {}", entry)?;
                }
            }
            if !self.release_ayanamsas.is_empty() {
                writeln!(f, "Ayanamsas:")?;
                for entry in self.release_ayanamsas {
                    writeln!(f, "- {}", entry)?;
                }
            }
        }
        if !self.release_notes.is_empty() {
            writeln!(f)?;
            write_scope_section(
                f,
                "Release-specific notes beyond baseline:",
                self.release_notes,
            )?;
        }
        writeln!(f)?;
        let coverage = metadata_coverage();
        writeln!(f, "Coverage summary:")?;
        writeln!(
            f,
            "- house systems: {} total ({} baseline, {} release-specific)",
            self.house_systems.len(),
            self.baseline_house_systems.len(),
            self.release_house_systems.len()
        )?;
        writeln!(
            f,
            "- ayanamsas: {} total ({} baseline, {} release-specific)",
            self.ayanamsas.len(),
            self.baseline_ayanamsas.len(),
            self.release_ayanamsas.len()
        )?;
        writeln!(f, "- {}", coverage.summary_line())?;
        writeln!(
            f,
            "- house formula families: {}",
            self.house_formula_families_summary_line()
        )?;
        writeln!(
            f,
            "- latitude-sensitive house systems: {}",
            self.latitude_sensitive_house_systems_summary_line()
        )?;
        writeln!(
            f,
            "- latitude-sensitive house constraints: {}",
            self.latitude_sensitive_house_constraints_summary_line()
        )?;
        if !coverage.custom_definition_only.is_empty() {
            writeln!(
                f,
                "- custom-definition ayanamsas: {} (tracked without sidereal metadata)",
                coverage.custom_definition_only.join(", ")
            )?;
        }
        if coverage.is_complete() {
            writeln!(f, "- no unexpected sidereal-metadata gaps remain.")?;
        } else {
            writeln!(
                f,
                "- missing metadata: {}",
                coverage.without_sidereal_metadata.join(", ")
            )?;
        }
        writeln!(f, "{}", self.catalog_inventory_summary_line())?;
        if !self.custom_definition_labels.is_empty() {
            writeln!(
                f,
                "- custom-definition labels: {}",
                self.custom_definition_labels.len()
            )?;
            writeln!(f)?;
            write_custom_definition_section(f, self.custom_definition_labels, self.ayanamsas)?;
        }
        writeln!(f)?;
        write_alias_section(
            f,
            "Alias mappings for built-in house systems:",
            self.house_systems,
        )?;
        writeln!(f)?;
        write_source_label_section(
            f,
            "Source-label aliases for built-in house systems:",
            self.house_systems,
            house_source_label_aliases,
        )?;
        writeln!(f)?;
        write_source_label_section(
            f,
            "Source-label aliases for built-in ayanamsas:",
            self.ayanamsas,
            ayanamsa_source_label_aliases,
        )?;
        writeln!(f)?;
        write_alias_section(f, "Alias mappings for built-in ayanamsas:", self.ayanamsas)?;
        writeln!(f)?;
        write_scope_section(
            f,
            "Validation reference points:",
            self.validation_reference_points,
        )?;
        writeln!(f)?;
        write_scope_section(f, "Compatibility caveats:", self.known_gaps)?;
        writeln!(
            f,
            "Unsupported modes: {}",
            self.unsupported_modes_summary_line()
        )?;
        Ok(())
    }
}
