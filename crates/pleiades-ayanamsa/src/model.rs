//! Domain types for the ayanamsa catalog: descriptors, provenance, and validation.

use core::fmt;
use std::collections::BTreeSet;

use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay};

use crate::lookup::{
    built_in_ayanamsas, descriptor, metadata_coverage, provenance_sample_ayanamsas,
    validate_ayanamsa_catalog_entries,
};

/// A catalog entry for a built-in ayanamsa.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaDescriptor {
    /// The strongly typed ayanamsa identifier.
    pub ayanamsa: Ayanamsa,
    /// The canonical name used in compatibility profiles.
    pub canonical_name: &'static str,
    /// Alternate names or software-specific aliases.
    pub aliases: &'static [&'static str],
    /// Short notes about the definition or interoperability constraints.
    pub notes: &'static str,
    /// Reference epoch for the published offset, when available.
    pub epoch: Option<JulianDay>,
    /// Reference sidereal offset in degrees at the reference epoch, when available.
    pub offset_degrees: Option<Angle>,
    /// The compatibility claim tier for this built-in entry.
    pub claim_tier: pleiades_types::CompatibilityClaimTier,
}

impl AyanamsaDescriptor {
    /// Creates a descriptor that makes no numeric compatibility claim.
    pub const fn new(
        ayanamsa: Ayanamsa,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        epoch: Option<JulianDay>,
        offset_degrees: Option<Angle>,
    ) -> Self {
        Self {
            ayanamsa,
            canonical_name,
            aliases,
            notes,
            epoch,
            offset_degrees,
            claim_tier: pleiades_types::CompatibilityClaimTier::DescriptorOnly,
        }
    }

    /// Creates a descriptor that asserts release-grade numeric compatibility.
    /// Use only for entries with passing SE numeric-gate evidence.
    pub const fn new_release_grade(
        ayanamsa: Ayanamsa,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        epoch: Option<JulianDay>,
        offset_degrees: Option<Angle>,
    ) -> Self {
        Self {
            ayanamsa,
            canonical_name,
            aliases,
            notes,
            epoch,
            offset_degrees,
            claim_tier: pleiades_types::CompatibilityClaimTier::ReleaseGradeNumeric,
        }
    }

    /// Validates the descriptor-local metadata invariants.
    pub fn validate(&self) -> Result<(), AyanamsaCatalogValidationError> {
        if self.canonical_name.trim().is_empty()
            || has_surrounding_whitespace(self.canonical_name)
            || contains_line_break(self.canonical_name)
        {
            return Err(
                AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                    label: self.canonical_name,
                    field: "canonical name",
                },
            );
        }

        for alias in self.aliases {
            if alias.trim().is_empty()
                || has_surrounding_whitespace(alias)
                || contains_line_break(alias)
            {
                return Err(
                    AyanamsaCatalogValidationError::DescriptorLabelNotNormalized {
                        label: alias,
                        field: "alias",
                    },
                );
            }
        }

        if self.notes.trim().is_empty()
            || (!self.notes.is_empty() && self.notes.trim() != self.notes)
            || contains_line_break(self.notes)
        {
            return Err(
                AyanamsaCatalogValidationError::DescriptorNotesNotNormalized {
                    label: self.canonical_name,
                },
            );
        }

        let mut seen_labels = BTreeSet::new();
        let mut saw_canonical_case_variant = false;
        for alias in self.aliases {
            if alias.eq_ignore_ascii_case(self.canonical_name) {
                if alias == &self.canonical_name || saw_canonical_case_variant {
                    return Err(AyanamsaCatalogValidationError::DescriptorLabelCollision {
                        label: alias,
                        canonical_name: self.canonical_name,
                    });
                }
                saw_canonical_case_variant = true;
                continue;
            }

            if !seen_labels.insert(alias.to_ascii_lowercase()) {
                return Err(AyanamsaCatalogValidationError::DescriptorLabelCollision {
                    label: alias,
                    canonical_name: self.canonical_name,
                });
            }
        }

        if self.epoch.is_some() ^ self.offset_degrees.is_some() {
            return Err(AyanamsaCatalogValidationError::PartialSiderealMetadata {
                label: self.canonical_name,
            });
        }

        Ok(())
    }

    /// Returns the sidereal offset at the provided instant, when the catalog
    /// entry carries enough metadata to derive one.
    pub fn offset_at(&self, instant: Instant) -> Option<Angle> {
        crate::lookup::offset_from_components(self.epoch, self.offset_degrees, instant)
    }

    /// Returns `true` if the provided label matches the canonical name or one
    /// of the documented aliases.
    pub fn matches_label(&self, label: &str) -> bool {
        self.canonical_name.eq_ignore_ascii_case(label)
            || self
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(label))
    }

    /// Returns `true` when both reference metadata fields are present.
    pub fn has_sidereal_metadata(&self) -> bool {
        self.epoch.is_some() && self.offset_degrees.is_some()
    }

    /// Returns a compact one-line rendering of the descriptor.
    pub fn summary_line(&self) -> String {
        let mut text = String::from(self.canonical_name);
        if !self.aliases.is_empty() {
            text.push_str(" (aliases: ");
            text.push_str(&self.aliases.join(", "));
            text.push(')');
        }
        if let Some(epoch) = self.epoch {
            text.push_str(" [epoch: ");
            text.push_str(&epoch.to_string());
            text.push(']');
        }
        if let Some(offset) = self.offset_degrees {
            text.push_str(" [offset: ");
            text.push_str(&offset.to_string());
            text.push(']');
        }
        text.push_str(" — ");
        text.push_str(self.notes);
        text
    }

    /// Returns the descriptor summary after validating the entry first.
    pub fn validated_summary_line(&self) -> Result<String, AyanamsaCatalogValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A single provenance example surfaced in release-facing reports.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaProvenanceExample {
    /// The canonical ayanamsa name.
    pub canonical_name: &'static str,
    /// The provenance note surfaced for the example.
    pub provenance_note: &'static str,
}

/// A compact provenance summary for representative ayanamsa examples.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaProvenanceSummary {
    /// Representative provenance examples.
    pub examples: Vec<AyanamsaProvenanceExample>,
}

/// Validation errors for the representative ayanamsa provenance summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AyanamsaProvenanceSummaryValidationError {
    /// A summary field drifted from the documented sample set.
    FieldOutOfSync {
        /// Name of the summary field that no longer matches the documented posture.
        field: &'static str,
    },
}

impl fmt::Display for AyanamsaProvenanceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the ayanamsa provenance summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for AyanamsaProvenanceSummaryValidationError {}

impl Default for AyanamsaProvenanceSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl AyanamsaProvenanceSummary {
    /// Creates the representative provenance summary from the documented sample set.
    pub fn new() -> Self {
        let examples = provenance_sample_ayanamsas()
            .iter()
            .map(|ayanamsa| {
                let d = descriptor(ayanamsa)
                    .expect("provenance sample ayanamsa should exist in the built-in catalog");
                AyanamsaProvenanceExample {
                    canonical_name: d.canonical_name,
                    provenance_note: d.notes,
                }
            })
            .collect();

        Self { examples }
    }

    /// Validates that the provenance summary still matches the documented sample set.
    pub fn validate(&self) -> Result<(), AyanamsaProvenanceSummaryValidationError> {
        if self.examples != AyanamsaProvenanceSummary::new().examples {
            return Err(AyanamsaProvenanceSummaryValidationError::FieldOutOfSync {
                field: "examples",
            });
        }

        Ok(())
    }

    /// Returns the compact provenance payload used in release-facing reports.
    pub fn summary_line(&self) -> String {
        let examples = self
            .examples
            .iter()
            .map(|example| format!("{} — {}", example.canonical_name, example.provenance_note))
            .collect::<Vec<_>>()
            .join("; ");
        format!("representative provenance examples: {examples}")
    }

    /// Returns the compact provenance payload after validation.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, AyanamsaProvenanceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaProvenanceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A summary of which built-in ayanamsas have sidereal reference metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AyanamsaMetadataCoverage {
    /// Total number of built-in ayanamsas.
    pub total: usize,
    /// Built-in entries that provide both a reference epoch and a reference offset.
    pub with_sidereal_metadata: usize,
    /// Built-in entries that intentionally model custom-definition labels and
    /// therefore omit sidereal metadata.
    pub custom_definition_only: Vec<&'static str>,
    /// Canonical names for built-in entries that are still missing one or both fields.
    pub without_sidereal_metadata: Vec<&'static str>,
}

/// Errors returned when validating a sidereal-metadata coverage summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AyanamsaMetadataCoverageValidationError {
    /// The recorded counts do not add up to the expected total.
    CountsDoNotSum {
        /// Total number of built-in ayanamsas.
        total: usize,
        /// Built-in entries that provide both a reference epoch and a reference offset.
        with_sidereal_metadata: usize,
        /// Built-in entries that intentionally model custom-definition labels.
        custom_definition_only: usize,
        /// Built-in entries still missing one or both fields.
        without_sidereal_metadata: usize,
    },
    /// A custom-definition-only label does not belong in that bucket.
    UnexpectedCustomDefinitionLabel {
        /// Label that drifted.
        label: &'static str,
    },
    /// A missing-metadata label does not belong in the incomplete bucket.
    UnexpectedMissingMetadataLabel {
        /// Label that drifted.
        label: &'static str,
    },
    /// The custom-definition-only bucket does not match the documented release profile.
    CustomDefinitionOnlyLabelsDoNotMatch {
        /// Expected release-profile labels.
        expected: &'static [&'static str],
        /// Labels observed in the coverage summary.
        actual: Vec<&'static str>,
    },
    /// The missing-metadata bucket does not match the documented release profile.
    WithoutSiderealMetadataLabelsDoNotMatch {
        /// Expected release-profile labels.
        expected: &'static [&'static str],
        /// Labels observed in the coverage summary.
        actual: Vec<&'static str>,
    },
    /// A label appeared in more than one bucket.
    DuplicateLabel {
        /// Label that drifted.
        label: &'static str,
    },
}

impl fmt::Display for AyanamsaMetadataCoverageValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountsDoNotSum {
                total,
                with_sidereal_metadata,
                custom_definition_only,
                without_sidereal_metadata,
            } => write!(
                f,
                "coverage counts do not sum to the total ({with_sidereal_metadata} + {custom_definition_only} + {without_sidereal_metadata} != {total})"
            ),
            Self::UnexpectedCustomDefinitionLabel { label } => write!(
                f,
                "label `{label}` is not a documented custom-definition-only ayanamsa"
            ),
            Self::UnexpectedMissingMetadataLabel { label } => write!(
                f,
                "label `{label}` is not a documented sidereal-metadata gap"
            ),
            Self::CustomDefinitionOnlyLabelsDoNotMatch { expected, actual } => write!(
                f,
                "custom-definition-only labels do not match the documented release profile (expected: {}; actual: {})",
                crate::lookup::format_ayanamsa_label_list(expected),
                crate::lookup::format_ayanamsa_label_list(actual)
            ),
            Self::WithoutSiderealMetadataLabelsDoNotMatch { expected, actual } => write!(
                f,
                "missing sidereal-metadata labels do not match the documented release profile (expected: {}; actual: {})",
                crate::lookup::format_ayanamsa_label_list(expected),
                crate::lookup::format_ayanamsa_label_list(actual)
            ),
            Self::DuplicateLabel { label } => write!(
                f,
                "label `{label}` appears in more than one sidereal-metadata bucket"
            ),
        }
    }
}

impl std::error::Error for AyanamsaMetadataCoverageValidationError {}

impl AyanamsaMetadataCoverage {
    /// Returns `true` when every built-in ayanamsa that is meant to carry
    /// sidereal metadata does so.
    pub fn is_complete(&self) -> bool {
        self.without_sidereal_metadata.is_empty()
    }

    /// Validates the derived coverage record before it is rendered in release-facing output.
    pub fn validate(&self) -> Result<(), AyanamsaMetadataCoverageValidationError> {
        if self.with_sidereal_metadata
            + self.custom_definition_only.len()
            + self.without_sidereal_metadata.len()
            != self.total
        {
            return Err(AyanamsaMetadataCoverageValidationError::CountsDoNotSum {
                total: self.total,
                with_sidereal_metadata: self.with_sidereal_metadata,
                custom_definition_only: self.custom_definition_only.len(),
                without_sidereal_metadata: self.without_sidereal_metadata.len(),
            });
        }

        let mut seen_labels = BTreeSet::new();
        for label in &self.custom_definition_only {
            if !crate::lookup::is_custom_definition_only_ayanamsa(label) {
                return Err(
                    AyanamsaMetadataCoverageValidationError::UnexpectedCustomDefinitionLabel {
                        label,
                    },
                );
            }
            if !seen_labels.insert(label.to_ascii_lowercase()) {
                return Err(AyanamsaMetadataCoverageValidationError::DuplicateLabel { label });
            }
        }

        for label in &self.without_sidereal_metadata {
            if crate::lookup::is_custom_definition_only_ayanamsa(label) {
                return Err(
                    AyanamsaMetadataCoverageValidationError::UnexpectedMissingMetadataLabel {
                        label,
                    },
                );
            }
            if !seen_labels.insert(label.to_ascii_lowercase()) {
                return Err(AyanamsaMetadataCoverageValidationError::DuplicateLabel { label });
            }
        }

        if self.custom_definition_only.as_slice() != crate::lookup::CUSTOM_DEFINITION_ONLY_AYANAMSAS
        {
            return Err(
                AyanamsaMetadataCoverageValidationError::CustomDefinitionOnlyLabelsDoNotMatch {
                    expected: crate::lookup::CUSTOM_DEFINITION_ONLY_AYANAMSAS,
                    actual: self.custom_definition_only.clone(),
                },
            );
        }

        if !self.without_sidereal_metadata.is_empty() {
            return Err(
                AyanamsaMetadataCoverageValidationError::WithoutSiderealMetadataLabelsDoNotMatch {
                    expected: &[],
                    actual: self.without_sidereal_metadata.clone(),
                },
            );
        }

        Ok(())
    }

    /// Returns the compact release-facing summary line for the metadata coverage state.
    pub fn summary_line(&self) -> String {
        match self.validate() {
            Ok(()) => self.render_summary_line(),
            Err(error) => format!("ayanamsa sidereal metadata: unavailable ({error})"),
        }
    }

    /// Returns the compact release-facing summary line after validation.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, AyanamsaMetadataCoverageValidationError> {
        self.validate()?;
        Ok(self.render_summary_line())
    }

    fn render_summary_line(&self) -> String {
        let custom_definition_only_labels = if self.custom_definition_only.is_empty() {
            "none".to_string()
        } else {
            self.custom_definition_only.join(", ")
        };
        let without_sidereal_metadata_labels = if self.without_sidereal_metadata.is_empty() {
            "none".to_string()
        } else {
            self.without_sidereal_metadata.join(", ")
        };

        format!(
            "ayanamsa sidereal metadata: {}/{} entries with both a reference epoch and offset; custom-definition-only={} labels: {}; missing-sidereal-metadata={}",
            self.with_sidereal_metadata,
            self.total,
            self.custom_definition_only.len(),
            custom_definition_only_labels,
            without_sidereal_metadata_labels,
        )
    }
}

impl fmt::Display for AyanamsaMetadataCoverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Errors returned when validating the built-in ayanamsa catalog.
#[derive(Clone, Debug, PartialEq)]
pub enum AyanamsaCatalogValidationError {
    /// The catalog did not contain any entries.
    EmptyCatalog,
    /// A canonical label or alias was repeated.
    DuplicateLabel {
        /// Repeated label.
        label: &'static str,
    },
    /// A canonical label or alias did not resolve back to the expected entry.
    LabelDoesNotRoundTrip {
        /// Label that failed to resolve.
        label: &'static str,
        /// Expected typed ayanamsa.
        expected_ayanamsa: Ayanamsa,
    },
    /// A descriptor label duplicates another label within the same entry.
    DescriptorLabelCollision {
        /// Colliding label.
        label: &'static str,
        /// Canonical name of the descriptor that owns the collision.
        canonical_name: &'static str,
    },
    /// A descriptor label is blank or whitespace-padded.
    DescriptorLabelNotNormalized {
        /// Label that drifted.
        label: &'static str,
        /// Field that drifted.
        field: &'static str,
    },
    /// A descriptor note is blank or whitespace-padded.
    DescriptorNotesNotNormalized {
        /// Label whose descriptor note drifted.
        label: &'static str,
    },
    /// Exactly one of the reference epoch or offset fields was populated.
    PartialSiderealMetadata {
        /// Label whose metadata was incomplete.
        label: &'static str,
    },
}

impl fmt::Display for AyanamsaCatalogValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalog => f.write_str("catalog is empty"),
            Self::DuplicateLabel { label } => {
                write!(f, "duplicate label '{label}'")
            }
            Self::LabelDoesNotRoundTrip {
                label,
                expected_ayanamsa,
            } => write!(f, "label '{label}' should resolve to {expected_ayanamsa}",),
            Self::DescriptorLabelCollision {
                label,
                canonical_name,
            } => write!(
                f,
                "the ayanamsa catalog descriptor label `{label}` collides with another label on `{canonical_name}`"
            ),
            Self::DescriptorLabelNotNormalized { label, field } => write!(
                f,
                "the ayanamsa catalog descriptor {field} for `{label}` is blank, contains surrounding whitespace, or contains line breaks"
            ),
            Self::DescriptorNotesNotNormalized { label } => write!(
                f,
                "the ayanamsa catalog descriptor note for `{label}` is blank, contains surrounding whitespace, or contains line breaks"
            ),
            Self::PartialSiderealMetadata { label } => write!(
                f,
                "the ayanamsa catalog descriptor for `{label}` has only one of the reference epoch or offset fields populated"
            ),
        }
    }
}

impl std::error::Error for AyanamsaCatalogValidationError {}

/// A compact validation summary for the built-in ayanamsa catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaCatalogValidationSummary {
    /// Total number of built-in ayanamsa entries.
    pub entry_count: usize,
    /// Number of baseline entries.
    pub baseline_entry_count: usize,
    /// Number of release-specific entries.
    pub release_entry_count: usize,
    /// Number of canonical labels plus aliases checked.
    pub label_count: usize,
    /// Metadata coverage for the current built-in catalog.
    pub metadata_coverage: AyanamsaMetadataCoverage,
    /// Result of validating the built-in ayanamsa catalog.
    pub validation_result: Result<(), AyanamsaCatalogValidationError>,
}

/// Validation error for an ayanamsa catalog validation summary that drifted from the current posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AyanamsaCatalogValidationSummaryValidationError {
    /// A summary field is out of sync with the current ayanamsa catalog posture.
    FieldOutOfSync {
        /// Name of the summary field that no longer matches the current catalog posture.
        field: &'static str,
    },
}

impl fmt::Display for AyanamsaCatalogValidationSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the ayanamsa catalog validation summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for AyanamsaCatalogValidationSummaryValidationError {}

impl AyanamsaCatalogValidationSummary {
    /// Returns the compact release-facing summary line for the ayanamsa catalog validation state.
    pub fn summary_line(&self) -> String {
        match &self.validation_result {
            Ok(()) => format!(
                "ayanamsa catalog validation: ok ({} entries, {} labels checked; baseline={}, release={}; {}; implementation posture: {} baseline entries, {} release-specific entries, {} custom-definition-only labels; round-trip, alias uniqueness, and notes verified)",
                self.entry_count,
                self.label_count,
                self.baseline_entry_count,
                self.release_entry_count,
                self.metadata_coverage.summary_line(),
                self.baseline_entry_count,
                self.release_entry_count,
                self.metadata_coverage.custom_definition_only.len(),
            ),
            Err(error) => format!(
                "ayanamsa catalog validation: error: {} ({} entries; baseline={}, release={})",
                error,
                self.entry_count,
                self.baseline_entry_count,
                self.release_entry_count,
            ),
        }
    }

    /// Returns `Ok(())` when the summary still matches the current ayanamsa catalog posture.
    pub fn validate(&self) -> Result<(), AyanamsaCatalogValidationSummaryValidationError> {
        let expected_entry_count = built_in_ayanamsas().len();
        if self.entry_count != expected_entry_count {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "entry_count",
                },
            );
        }

        let expected_baseline_entry_count = crate::lookup::baseline_ayanamsas().len();
        if self.baseline_entry_count != expected_baseline_entry_count {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "baseline_entry_count",
                },
            );
        }

        let expected_release_entry_count = crate::lookup::release_ayanamsas().len();
        if self.release_entry_count != expected_release_entry_count {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "release_entry_count",
                },
            );
        }

        let expected_metadata_coverage = metadata_coverage();
        if self.metadata_coverage != expected_metadata_coverage {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "metadata_coverage",
                },
            );
        }
        self.metadata_coverage.validate().map_err(|_| {
            AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                field: "metadata_coverage",
            }
        })?;

        let (expected_label_count, expected_validation_result) =
            match validate_ayanamsa_catalog_entries(built_in_ayanamsas()) {
                Ok(label_count) => (label_count, Ok(())),
                Err(error) => (0, Err(error)),
            };

        if self.label_count != expected_label_count {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "label_count",
                },
            );
        }
        if self.validation_result != expected_validation_result {
            return Err(
                AyanamsaCatalogValidationSummaryValidationError::FieldOutOfSync {
                    field: "validation_result",
                },
            );
        }

        Ok(())
    }

    /// Returns the validated release-facing summary line for the ayanamsa catalog validation state.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, AyanamsaCatalogValidationSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for AyanamsaCatalogValidationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
}

pub(crate) fn contains_line_break(value: &str) -> bool {
    value.chars().any(|ch| matches!(ch, '\n' | '\r'))
}
