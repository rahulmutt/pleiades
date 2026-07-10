#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use pleiades_ayanamsa::{metadata_coverage, AyanamsaDescriptor};
use pleiades_houses::{
    HouseCatalogValidationError, HouseFormulaFamily, HouseSystemCodeAlias, HouseSystemDescriptor,
};
use pleiades_types::HouseSystem;

use super::report::AliasProfileEntry;
use super::CURRENT_COMPATIBILITY_PROFILE_SUMMARY;

/// A validation error emitted when the compatibility profile's internal
/// release-facing metadata drifts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompatibilityProfileValidationError {
    /// The profile identifier is blank.
    BlankProfileIdentifier,
    /// The summary is blank.
    BlankSummary,
    /// The summary no longer describes the baseline/release split explicitly.
    SummaryDoesNotDescribeReleaseSplit,
    /// A text section is empty or contains a blank entry.
    BlankTextSectionEntry {
        /// Section that failed validation.
        section_label: &'static str,
    },
    /// A text section contains an entry with surrounding whitespace.
    WhitespaceTextSectionEntry {
        /// Section that failed validation.
        section_label: &'static str,
        /// Entry that contained surrounding whitespace.
        entry: &'static str,
    },
    /// A text section contains a duplicate entry.
    DuplicateTextSectionEntry {
        /// Section that failed validation.
        section_label: &'static str,
        /// Duplicate entry.
        entry: &'static str,
    },
    /// Two different text sections contain the same entry.
    DuplicateTextSectionEntryAcrossSections {
        /// Duplicate entry.
        entry: &'static str,
        /// First section that contained the entry.
        first_section: &'static str,
        /// Second section that contained the entry.
        second_section: &'static str,
    },
    /// A custom-definition label resolves to a built-in house system or ayanamsa.
    CustomDefinitionLabelResolvesToBuiltIn {
        /// Label that should remain unresolved as a custom definition.
        label: &'static str,
    },
    /// A documented custom-definition-only ayanamsa is missing from the profile's custom-definition label list.
    MissingCustomDefinitionAyanamsaCoverageLabel {
        /// Label that should be surfaced as an intentional custom-definition ayanamsa.
        label: &'static str,
    },
    /// A built-in house-system descriptor failed its own normalization or alias checks.
    HouseDescriptorValidationFailed {
        /// Underlying descriptor validation error.
        error: HouseCatalogValidationError,
    },
    /// A built-in ayanamsa descriptor failed its own normalization or alias checks.
    AyanamsaDescriptorValidationFailed {
        /// Underlying descriptor validation error rendered as a stable summary string.
        error: String,
    },
    /// A Swiss-Ephemeris house-code alias does not resolve back to its typed house system.
    HouseCodeAliasDoesNotRoundTrip {
        /// Alias label that failed validation.
        label: &'static str,
        /// Typed system that the alias should resolve to.
        expected_system: HouseSystem,
    },
    /// Baseline and release partitions overlap on a catalog label.
    CatalogPartitionOverlap {
        /// Catalog that drifted.
        catalog_label: &'static str,
        /// Label that appears in both partitions.
        label: &'static str,
    },
    /// The published catalog contains a duplicate canonical name or alias.
    CatalogLabelCollision {
        /// Catalog that drifted.
        catalog_label: &'static str,
        /// Duplicate label that appeared more than once.
        label: &'static str,
    },
    /// A house-system descriptor resolved to an unknown formula family.
    HouseFormulaFamilyUnknown {
        /// House-system label that drifted.
        label: &'static str,
    },
    /// Two published catalogs share a canonical name or alias.
    CrossCatalogLabelCollision {
        /// First catalog that provided the duplicate label.
        first_catalog_label: &'static str,
        /// Second catalog that re-used the duplicate label.
        second_catalog_label: &'static str,
        /// Duplicate label that appeared in both catalogs.
        label: &'static str,
    },
    /// The total catalog coverage does not equal the baseline plus release partitions.
    CatalogCoverageMismatch {
        /// Catalog that drifted.
        catalog_label: &'static str,
        /// Total entries in the catalog.
        total_count: usize,
        /// Entries in the baseline partition.
        baseline_count: usize,
        /// Entries in the release partition.
        release_count: usize,
    },
    /// The total catalog labels do not match the baseline and release partitions exactly.
    CatalogCoverageLabelMismatch {
        /// Catalog that drifted.
        catalog_label: &'static str,
        /// Label that appears in the total catalog but not in the partitions.
        missing_label: &'static str,
        /// Label that appears in the partitions but not in the total catalog.
        unexpected_label: &'static str,
    },
}

impl core::fmt::Display for CompatibilityProfileValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BlankProfileIdentifier => {
                f.write_str("compatibility profile identifier is blank")
            }
            Self::BlankSummary => f.write_str("compatibility profile summary is blank"),
            Self::SummaryDoesNotDescribeReleaseSplit => f.write_str(
                "compatibility profile summary no longer describes the baseline/release split explicitly",
            ),
            Self::BlankTextSectionEntry { section_label } => {
                write!(f, "compatibility profile {section_label} entry is blank")
            }
            Self::WhitespaceTextSectionEntry { section_label, entry } => write!(
                f,
                "compatibility profile {section_label} entry '{}' contains surrounding whitespace",
                entry
            ),
            Self::DuplicateTextSectionEntry { section_label, entry } => write!(
                f,
                "compatibility profile {section_label} entries are not unique: duplicate entry '{}'",
                entry
            ),
            Self::DuplicateTextSectionEntryAcrossSections {
                entry,
                first_section,
                second_section,
            } => write!(
                f,
                "compatibility profile text sections are not unique: duplicate entry '{}' appears in both {} and {}",
                entry, first_section, second_section
            ),
            Self::CustomDefinitionLabelResolvesToBuiltIn { label } => write!(
                f,
                "compatibility profile custom-definition label '{}' should remain unresolved as a built-in house system or ayanamsa",
                label
            ),
            Self::MissingCustomDefinitionAyanamsaCoverageLabel { label } => write!(
                f,
                "compatibility profile custom-definition ayanamsa coverage is missing the documented custom-definition-only label '{}'",
                label
            ),
            Self::HouseDescriptorValidationFailed { error } => write!(
                f,
                "compatibility profile house-system descriptor is invalid: {}",
                error
            ),
            Self::AyanamsaDescriptorValidationFailed { error } => write!(
                f,
                "compatibility profile ayanamsa descriptor is invalid: {}",
                error
            ),
            Self::HouseCodeAliasDoesNotRoundTrip {
                label,
                expected_system,
            } => write!(
                f,
                "compatibility profile house-code alias '{}' should resolve to {}",
                label, expected_system
            ),
            Self::CatalogPartitionOverlap { catalog_label, label } => write!(
                f,
                "compatibility profile {catalog_label} baseline and release slices overlap on label '{}'",
                label
            ),
            Self::CatalogLabelCollision { catalog_label, label } => write!(
                f,
                "compatibility profile {catalog_label} catalog contains duplicate label '{}'",
                label
            ),
            Self::HouseFormulaFamilyUnknown { label } => write!(
                f,
                "compatibility profile house-system '{label}' resolves to an unknown formula family"
            ),
            Self::CrossCatalogLabelCollision {
                first_catalog_label,
                second_catalog_label,
                label,
            } => write!(
                f,
                "compatibility profile {first_catalog_label} and {second_catalog_label} catalogs share duplicate label '{}'",
                label
            ),
            Self::CatalogCoverageMismatch {
                catalog_label,
                total_count,
                baseline_count,
                release_count,
            } => write!(
                f,
                "compatibility profile {catalog_label} catalog coverage mismatch: total={}, baseline={}, release={}",
                total_count, baseline_count, release_count
            ),
            Self::CatalogCoverageLabelMismatch {
                catalog_label,
                missing_label,
                unexpected_label,
            } => write!(
                f,
                "compatibility profile {catalog_label} catalog coverage mismatch: missing label '{}', unexpected label '{}'",
                missing_label, unexpected_label
            ),
        }
    }
}

impl std::error::Error for CompatibilityProfileValidationError {}

pub(super) fn validate_profile_identifier(
    profile_id: &str,
) -> Result<(), CompatibilityProfileValidationError> {
    if profile_id.trim().is_empty() {
        return Err(CompatibilityProfileValidationError::BlankProfileIdentifier);
    }

    Ok(())
}

pub(super) fn validate_profile_summary(
    summary: &str,
) -> Result<(), CompatibilityProfileValidationError> {
    if summary.trim().is_empty() {
        return Err(CompatibilityProfileValidationError::BlankSummary);
    }

    if summary != CURRENT_COMPATIBILITY_PROFILE_SUMMARY {
        return Err(CompatibilityProfileValidationError::SummaryDoesNotDescribeReleaseSplit);
    }

    Ok(())
}

fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
}

fn normalized_profile_text_entry(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub(super) fn validate_profile_text_section(
    section_label: &'static str,
    entries: &[&'static str],
) -> Result<(), CompatibilityProfileValidationError> {
    if entries.is_empty() {
        return Err(CompatibilityProfileValidationError::BlankTextSectionEntry { section_label });
    }

    let mut seen_entries = BTreeSet::new();
    for entry in entries {
        if entry.trim().is_empty() {
            return Err(CompatibilityProfileValidationError::BlankTextSectionEntry {
                section_label,
            });
        }

        if has_surrounding_whitespace(entry) {
            return Err(
                CompatibilityProfileValidationError::WhitespaceTextSectionEntry {
                    section_label,
                    entry,
                },
            );
        }

        if !seen_entries.insert(normalized_profile_text_entry(entry)) {
            return Err(
                CompatibilityProfileValidationError::DuplicateTextSectionEntry {
                    section_label,
                    entry,
                },
            );
        }
    }

    Ok(())
}

pub(super) fn validate_profile_text_sections_are_disjoint(
    sections: &[(&'static str, &'static [&'static str])],
) -> Result<(), CompatibilityProfileValidationError> {
    let mut seen_entries = BTreeMap::<String, &'static str>::new();
    for (section_label, entries) in sections {
        for entry in *entries {
            let normalized_entry = normalized_profile_text_entry(entry);
            if let Some(existing_section) = seen_entries.get(&normalized_entry) {
                if *existing_section != *section_label {
                    return Err(
                        CompatibilityProfileValidationError::DuplicateTextSectionEntryAcrossSections {
                            entry,
                            first_section: existing_section,
                            second_section: section_label,
                        },
                    );
                }
            } else {
                seen_entries.insert(normalized_entry, section_label);
            }
        }
    }

    Ok(())
}

const INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS: &[&str] = &[
    "Babylonian (House)",
    "Babylonian (Sissy)",
    "Babylonian (True Geoc)",
    "Babylonian (True Topc)",
    "Babylonian (True Obs)",
    "Babylonian (House Obs)",
];

fn is_intentional_custom_definition_ayanamsa_homograph(label: &str) -> bool {
    INTENTIONAL_CUSTOM_DEFINITION_AYANAMSA_HOMOGRAPHS.contains(&label)
}

/// Validates the custom-definition labels of a compatibility profile.
///
/// Each label must be non-empty, free of surrounding whitespace, unique under
/// the profile's text-normalization rules, and must not collide with a built-in
/// house-system or ayanamsa name (except for the intentionally reserved
/// homographs). Returns the number of labels checked on success.
pub fn validate_custom_definition_labels(
    labels: &[&'static str],
) -> Result<usize, CompatibilityProfileValidationError> {
    if labels.is_empty() {
        return Err(CompatibilityProfileValidationError::BlankTextSectionEntry {
            section_label: "custom-definition",
        });
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for label in labels {
        labels_checked += 1;

        if label.trim().is_empty() {
            return Err(CompatibilityProfileValidationError::BlankTextSectionEntry {
                section_label: "custom-definition",
            });
        }

        if has_surrounding_whitespace(label) {
            return Err(
                CompatibilityProfileValidationError::WhitespaceTextSectionEntry {
                    section_label: "custom-definition",
                    entry: label,
                },
            );
        }

        let normalized_label = normalized_profile_text_entry(label);
        if !seen_labels.insert(normalized_label) {
            return Err(
                CompatibilityProfileValidationError::DuplicateTextSectionEntry {
                    section_label: "custom-definition",
                    entry: label,
                },
            );
        }

        if pleiades_houses::resolve_house_system(label).is_some()
            || (pleiades_ayanamsa::resolve_ayanamsa(label).is_some()
                && !is_intentional_custom_definition_ayanamsa_homograph(label))
        {
            return Err(
                CompatibilityProfileValidationError::CustomDefinitionLabelResolvesToBuiltIn {
                    label,
                },
            );
        }
    }

    Ok(labels_checked)
}

pub(super) fn validate_custom_definition_ayanamsa_coverage(
    labels: &[&'static str],
) -> Result<usize, CompatibilityProfileValidationError> {
    let coverage = metadata_coverage();
    let mut seen_labels = BTreeSet::new();
    for label in labels {
        seen_labels.insert(normalized_profile_text_entry(label));
    }

    let mut checked = 0usize;
    for label in coverage.custom_definition_only {
        checked += 1;
        if !seen_labels.contains(&normalized_profile_text_entry(label)) {
            return Err(
                CompatibilityProfileValidationError::MissingCustomDefinitionAyanamsaCoverageLabel {
                    label,
                },
            );
        }
    }

    Ok(checked)
}

pub(super) fn validate_house_descriptor_metadata(
    entries: &[HouseSystemDescriptor],
) -> Result<usize, CompatibilityProfileValidationError> {
    let mut checked = 0usize;
    for entry in entries {
        checked += 1;
        entry.validate().map_err(|error| {
            CompatibilityProfileValidationError::HouseDescriptorValidationFailed { error }
        })?;
    }

    Ok(checked)
}

pub(super) fn validate_ayanamsa_descriptor_metadata(
    entries: &[AyanamsaDescriptor],
) -> Result<usize, CompatibilityProfileValidationError> {
    let mut checked = 0usize;
    for entry in entries {
        checked += 1;
        entry.validate().map_err(|error| {
            CompatibilityProfileValidationError::AyanamsaDescriptorValidationFailed {
                error: error.to_string(),
            }
        })?;
    }

    Ok(checked)
}

pub(super) fn validate_house_code_aliases(
    aliases: &[HouseSystemCodeAlias],
) -> Result<usize, CompatibilityProfileValidationError> {
    if aliases.is_empty() {
        return Err(CompatibilityProfileValidationError::BlankTextSectionEntry {
            section_label: "house-code-alias",
        });
    }

    let mut aliases_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for alias in aliases {
        aliases_checked += 1;

        if alias.label.trim().is_empty() {
            return Err(CompatibilityProfileValidationError::BlankTextSectionEntry {
                section_label: "house-code-alias",
            });
        }

        if has_surrounding_whitespace(alias.label) {
            return Err(
                CompatibilityProfileValidationError::WhitespaceTextSectionEntry {
                    section_label: "house-code-alias",
                    entry: alias.label,
                },
            );
        }

        let normalized_label = normalized_profile_text_entry(alias.label);
        if !seen_labels.insert(normalized_label) {
            return Err(
                CompatibilityProfileValidationError::DuplicateTextSectionEntry {
                    section_label: "house-code-alias",
                    entry: alias.label,
                },
            );
        }

        match pleiades_houses::resolve_house_system(alias.label) {
            Some(system) if system == alias.system => {}
            _ => {
                return Err(
                    CompatibilityProfileValidationError::HouseCodeAliasDoesNotRoundTrip {
                        label: alias.label,
                        expected_system: alias.system.clone(),
                    },
                );
            }
        }
    }

    Ok(aliases_checked)
}

pub(super) fn validate_catalog_partitions_are_disjoint(
    catalog_label: &'static str,
    baseline_entries: &[HouseSystemDescriptor],
    release_entries: &[HouseSystemDescriptor],
) -> Result<(), CompatibilityProfileValidationError> {
    let mut baseline_labels = BTreeSet::new();
    for entry in baseline_entries {
        baseline_labels.insert(entry.canonical_name.trim().to_ascii_lowercase());
        for alias in entry.aliases {
            baseline_labels.insert(alias.trim().to_ascii_lowercase());
        }
    }

    for entry in release_entries {
        for label in std::iter::once(entry.canonical_name).chain(entry.aliases.iter().copied()) {
            if baseline_labels.contains(&label.trim().to_ascii_lowercase()) {
                return Err(
                    CompatibilityProfileValidationError::CatalogPartitionOverlap {
                        catalog_label,
                        label,
                    },
                );
            }
        }
    }

    Ok(())
}

pub(super) fn validate_catalog_partitions_are_disjoint_ayanamsa(
    catalog_label: &'static str,
    baseline_entries: &[AyanamsaDescriptor],
    release_entries: &[AyanamsaDescriptor],
) -> Result<(), CompatibilityProfileValidationError> {
    let mut baseline_labels = BTreeSet::new();
    for entry in baseline_entries {
        baseline_labels.insert(entry.canonical_name.trim().to_ascii_lowercase());
        for alias in entry.aliases {
            baseline_labels.insert(alias.trim().to_ascii_lowercase());
        }
    }

    for entry in release_entries {
        for label in std::iter::once(entry.canonical_name).chain(entry.aliases.iter().copied()) {
            if baseline_labels.contains(&label.trim().to_ascii_lowercase()) {
                return Err(
                    CompatibilityProfileValidationError::CatalogPartitionOverlap {
                        catalog_label,
                        label,
                    },
                );
            }
        }
    }

    Ok(())
}

fn is_case_only_alias_variant(canonical_name: &str, label: &str) -> bool {
    label != canonical_name && label.eq_ignore_ascii_case(canonical_name)
}

pub(super) fn validate_catalog_label_uniqueness<T: AliasProfileEntry>(
    catalog_label: &'static str,
    entries: &[T],
) -> Result<(), CompatibilityProfileValidationError> {
    let mut seen_labels = BTreeSet::new();

    for entry in entries {
        for label in std::iter::once(entry.canonical_name()).chain(entry.aliases().iter().copied())
        {
            // Some built-in entries intentionally retain case-only alias variants,
            // such as Vehlow Equal / vehlow equal and the galactic-reference
            // labels that keep both title-case and lowercase spellings visible in
            // the release profile. Keep those forms allowed while still rejecting
            // genuine duplicate labels.
            if is_case_only_alias_variant(entry.canonical_name(), label) {
                continue;
            }

            let normalized_label = label.trim().to_ascii_lowercase();
            if !seen_labels.insert(normalized_label) {
                return Err(CompatibilityProfileValidationError::CatalogLabelCollision {
                    catalog_label,
                    label,
                });
            }
        }
    }

    Ok(())
}

fn collect_catalog_labels<T: AliasProfileEntry>(entries: &[T]) -> BTreeSet<String> {
    let mut labels = BTreeSet::new();

    for entry in entries {
        labels.insert(entry.canonical_name().trim().to_ascii_lowercase());
        for alias in entry.aliases() {
            labels.insert(alias.trim().to_ascii_lowercase());
        }
    }

    labels
}

pub(super) fn validate_catalogs_are_disjoint<T: AliasProfileEntry, U: AliasProfileEntry>(
    first_catalog_label: &'static str,
    first_entries: &[T],
    second_catalog_label: &'static str,
    second_entries: &[U],
) -> Result<(), CompatibilityProfileValidationError> {
    let first_labels = collect_catalog_labels(first_entries);

    for entry in second_entries {
        for label in std::iter::once(entry.canonical_name()).chain(entry.aliases().iter().copied())
        {
            let normalized_label = label.trim().to_ascii_lowercase();
            if first_labels.contains(&normalized_label) {
                return Err(
                    CompatibilityProfileValidationError::CrossCatalogLabelCollision {
                        first_catalog_label,
                        second_catalog_label,
                        label,
                    },
                );
            }
        }
    }

    Ok(())
}

pub(super) fn validate_catalog_coverage(
    catalog_label: &'static str,
    total_entries: &[HouseSystemDescriptor],
    baseline_entries: &[HouseSystemDescriptor],
    release_entries: &[HouseSystemDescriptor],
) -> Result<(), CompatibilityProfileValidationError> {
    if total_entries.len() != baseline_entries.len() + release_entries.len() {
        return Err(
            CompatibilityProfileValidationError::CatalogCoverageMismatch {
                catalog_label,
                total_count: total_entries.len(),
                baseline_count: baseline_entries.len(),
                release_count: release_entries.len(),
            },
        );
    }

    Ok(())
}

pub(super) fn validate_house_formula_families(
    entries: &[HouseSystemDescriptor],
) -> Result<usize, CompatibilityProfileValidationError> {
    let mut checked = 0usize;

    for entry in entries {
        checked += 1;
        if entry.formula_family() == HouseFormulaFamily::Unknown {
            return Err(
                CompatibilityProfileValidationError::HouseFormulaFamilyUnknown {
                    label: entry.canonical_name,
                },
            );
        }
    }

    Ok(checked)
}

pub(super) fn validate_catalog_coverage_ayanamsa(
    catalog_label: &'static str,
    total_entries: &[AyanamsaDescriptor],
    baseline_entries: &[AyanamsaDescriptor],
    release_entries: &[AyanamsaDescriptor],
) -> Result<(), CompatibilityProfileValidationError> {
    if total_entries.len() != baseline_entries.len() + release_entries.len() {
        return Err(
            CompatibilityProfileValidationError::CatalogCoverageMismatch {
                catalog_label,
                total_count: total_entries.len(),
                baseline_count: baseline_entries.len(),
                release_count: release_entries.len(),
            },
        );
    }

    Ok(())
}

fn collect_catalog_names<T>(
    entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    for entry in entries {
        names.insert(canonical_name(entry).trim().to_ascii_lowercase());
    }

    names
}

pub(super) fn validate_catalog_coverage_labels<T>(
    catalog_label: &'static str,
    total_entries: &[T],
    baseline_entries: &[T],
    release_entries: &[T],
    canonical_name: impl Fn(&T) -> &'static str,
) -> Result<(), CompatibilityProfileValidationError> {
    let total_names = collect_catalog_names(total_entries, &canonical_name);
    let mut partition_names = collect_catalog_names(baseline_entries, &canonical_name);
    partition_names.extend(collect_catalog_names(release_entries, &canonical_name));

    if total_names == partition_names {
        return Ok(());
    }

    let missing_label = total_entries
        .iter()
        .map(&canonical_name)
        .find(|label| !partition_names.contains(&label.trim().to_ascii_lowercase()))
        .unwrap_or_else(|| {
            total_entries
                .first()
                .map_or("", |entry| canonical_name(entry))
        });
    let unexpected_label = baseline_entries
        .iter()
        .chain(release_entries.iter())
        .map(&canonical_name)
        .find(|label| !total_names.contains(&label.trim().to_ascii_lowercase()))
        .unwrap_or_else(|| {
            baseline_entries
                .first()
                .map_or("", |entry| canonical_name(entry))
        });

    Err(
        CompatibilityProfileValidationError::CatalogCoverageLabelMismatch {
            catalog_label,
            missing_label,
            unexpected_label,
        },
    )
}
