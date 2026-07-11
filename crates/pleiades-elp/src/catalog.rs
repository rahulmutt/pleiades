use core::fmt;

use pleiades_types::CelestialBody;

use crate::source::{
    lunar_theory_source_selection, LunarTheoryCatalogKey, LunarTheorySourceFamily,
    LunarTheorySourceSelection,
};
use crate::specification::{
    lunar_theory_specification, LunarTheorySpecification, LunarTheorySpecificationValidationError,
    LUNAR_THEORY_SPECIFICATION,
};

const LUNAR_THEORY_CATALOG: &[LunarTheoryCatalogEntry] = &[LunarTheoryCatalogEntry {
    selected: true,
    specification: LUNAR_THEORY_SPECIFICATION,
}];

/// Returns the structured catalog of lunar-theory selections.
pub fn lunar_theory_catalog() -> &'static [LunarTheoryCatalogEntry] {
    LUNAR_THEORY_CATALOG
}

/// Structured description of a catalog entry in the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryCatalogEntry {
    /// Whether this catalog entry is the currently selected lunar-theory baseline.
    pub selected: bool,
    /// Structured description of the selected lunar-theory baseline.
    pub specification: LunarTheorySpecification,
}

impl LunarTheoryCatalogEntry {
    /// Returns a compact summary line for a catalog entry.
    pub fn summary_line(&self) -> String {
        let source = self.specification.source_selection();

        format!(
            "lunar theory catalog entry: selected={}; source={} [{}]; key={}; aliases={}; supported bodies={}; unsupported bodies={}",
            self.selected,
            self.specification.source_identifier,
            self.specification.source_family.label(),
            source.catalog_key(),
            self.specification.source_aliases.len(),
            self.specification.supported_bodies.len(),
            self.specification.unsupported_bodies.len(),
        )
    }

    /// Returns `Ok(())` when the entry still matches the current baseline specification.
    pub fn validate(&self) -> Result<(), LunarTheorySpecificationValidationError> {
        self.specification.validate()
    }

    /// Returns the compact summary line after validating the entry against the current catalog.
    pub fn validated_summary_line(&self) -> Result<String, LunarTheoryCatalogValidationError> {
        validate_lunar_theory_catalog_entries(std::slice::from_ref(self))?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for LunarTheoryCatalogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation errors for the structured lunar-theory catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryCatalogValidationError {
    /// The catalog unexpectedly contains no entries.
    EmptyCatalog,
    /// The catalog unexpectedly contains no selected entry.
    NoSelectedEntry,
    /// The catalog unexpectedly contains more than one selected entry.
    MultipleSelectedEntries {
        /// Number of selected entries observed during validation.
        selected_count: usize,
    },
    /// Two catalog entries share the same source identifier.
    DuplicateSourceIdentifier {
        /// Source identifier that collided.
        source_identifier: &'static str,
    },
    /// Two catalog entries share the same model name.
    DuplicateModelName {
        /// Model name that collided.
        model_name: &'static str,
    },
    /// Two catalog entries share the same family label.
    DuplicateFamilyLabel {
        /// Family label that collided.
        family_label: &'static str,
    },
    /// A supported body appears more than once in the same catalog entry.
    DuplicateSupportedBody {
        /// Supported body that collided.
        body: CelestialBody,
    },
    /// An unsupported body appears more than once in the same catalog entry.
    DuplicateUnsupportedBody {
        /// Unsupported body that collided.
        body: CelestialBody,
    },
    /// A body appears in both the supported and unsupported lists.
    OverlappingSupportedAndUnsupportedBody {
        /// Body that was listed in both coverage sets.
        body: CelestialBody,
    },
    /// Two catalog entries share the same documented alias.
    DuplicateAlias {
        /// Alias that collided.
        alias: &'static str,
    },
    /// The selected entry does not round-trip through the typed selection helper.
    SelectedEntryDoesNotRoundTrip {
        /// Source identifier of the selected entry.
        source_identifier: &'static str,
    },
    /// A catalog entry's specification drifted away from the current baseline.
    SpecificationFieldOutOfSync {
        /// Source identifier of the drifting entry.
        source_identifier: &'static str,
        /// Field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheoryCatalogValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalog => f.write_str("the lunar-theory catalog is empty"),
            Self::NoSelectedEntry => f.write_str("the lunar-theory catalog has no selected entry"),
            Self::MultipleSelectedEntries { selected_count } => write!(
                f,
                "the lunar-theory catalog has {selected_count} selected entries"
            ),
            Self::DuplicateSourceIdentifier { source_identifier } => write!(
                f,
                "the lunar-theory catalog contains duplicate source identifier `{source_identifier}`"
            ),
            Self::DuplicateModelName { model_name } => write!(
                f,
                "the lunar-theory catalog contains duplicate model name `{model_name}`"
            ),
            Self::DuplicateFamilyLabel { family_label } => write!(
                f,
                "the lunar-theory catalog contains duplicate family label `{family_label}`"
            ),
            Self::DuplicateSupportedBody { body } => write!(
                f,
                "the lunar-theory catalog contains duplicate supported body `{body}`"
            ),
            Self::DuplicateUnsupportedBody { body } => write!(
                f,
                "the lunar-theory catalog contains duplicate unsupported body `{body}`"
            ),
            Self::OverlappingSupportedAndUnsupportedBody { body } => write!(
                f,
                "the lunar-theory catalog lists body `{body}` in both the supported and unsupported sets"
            ),
            Self::DuplicateAlias { alias } => write!(
                f,
                "the lunar-theory catalog contains duplicate alias `{alias}`"
            ),
            Self::SelectedEntryDoesNotRoundTrip { source_identifier } => write!(
                f,
                "the selected lunar-theory catalog entry `{source_identifier}` does not round-trip through the typed selection helper"
            ),
            Self::SpecificationFieldOutOfSync {
                source_identifier,
                field,
            } => write!(
                f,
                "the lunar-theory catalog entry `{source_identifier}` has specification field `{field}` out of sync with the current catalog"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryCatalogValidationError {}

pub(crate) fn validate_lunar_theory_catalog_entries(
    catalog: &[LunarTheoryCatalogEntry],
) -> Result<(), LunarTheoryCatalogValidationError> {
    if catalog.is_empty() {
        return Err(LunarTheoryCatalogValidationError::EmptyCatalog);
    }

    let selected_entries = catalog.iter().filter(|entry| entry.selected).count();
    match selected_entries {
        0 => return Err(LunarTheoryCatalogValidationError::NoSelectedEntry),
        1 => {}
        selected_count => {
            return Err(LunarTheoryCatalogValidationError::MultipleSelectedEntries {
                selected_count,
            });
        }
    }

    for (index, entry) in catalog.iter().enumerate() {
        if let Err(error) = entry.validate() {
            return Err(match error {
                LunarTheorySpecificationValidationError::FieldOutOfSync { field } => {
                    LunarTheoryCatalogValidationError::SpecificationFieldOutOfSync {
                        source_identifier: entry.specification.source_identifier,
                        field,
                    }
                }
            });
        }

        for (body_index, body) in entry.specification.supported_bodies.iter().enumerate() {
            if entry.specification.supported_bodies[..body_index].contains(body) {
                return Err(LunarTheoryCatalogValidationError::DuplicateSupportedBody {
                    body: body.clone(),
                });
            }
            if entry.specification.unsupported_bodies.contains(body) {
                return Err(
                    LunarTheoryCatalogValidationError::OverlappingSupportedAndUnsupportedBody {
                        body: body.clone(),
                    },
                );
            }
        }

        for (body_index, body) in entry.specification.unsupported_bodies.iter().enumerate() {
            if entry.specification.unsupported_bodies[..body_index].contains(body) {
                return Err(
                    LunarTheoryCatalogValidationError::DuplicateUnsupportedBody {
                        body: body.clone(),
                    },
                );
            }
        }

        for (alias_index, alias) in entry.specification.source_aliases.iter().enumerate() {
            if alias.eq_ignore_ascii_case(entry.specification.source_identifier)
                || alias.eq_ignore_ascii_case(entry.specification.model_name)
                || alias.eq_ignore_ascii_case(entry.specification.source_family.label())
            {
                return Err(LunarTheoryCatalogValidationError::DuplicateAlias { alias });
            }

            for other_alias in entry
                .specification
                .source_aliases
                .iter()
                .skip(alias_index + 1)
            {
                if alias.eq_ignore_ascii_case(other_alias) {
                    return Err(LunarTheoryCatalogValidationError::DuplicateAlias { alias });
                }
            }
        }

        for other in catalog.iter().skip(index + 1) {
            if entry
                .specification
                .source_identifier
                .eq_ignore_ascii_case(other.specification.source_identifier)
            {
                return Err(
                    LunarTheoryCatalogValidationError::DuplicateSourceIdentifier {
                        source_identifier: entry.specification.source_identifier,
                    },
                );
            }
            if entry
                .specification
                .model_name
                .eq_ignore_ascii_case(other.specification.model_name)
            {
                return Err(LunarTheoryCatalogValidationError::DuplicateModelName {
                    model_name: entry.specification.model_name,
                });
            }
            if entry
                .specification
                .source_family
                .label()
                .eq_ignore_ascii_case(other.specification.source_family.label())
            {
                return Err(LunarTheoryCatalogValidationError::DuplicateFamilyLabel {
                    family_label: entry.specification.source_family.label(),
                });
            }
            for alias in entry.specification.source_aliases {
                if other.specification.matches_alias(alias)
                    || other
                        .specification
                        .source_identifier
                        .eq_ignore_ascii_case(alias)
                    || other.specification.model_name.eq_ignore_ascii_case(alias)
                    || other
                        .specification
                        .source_family
                        .label()
                        .eq_ignore_ascii_case(alias)
                {
                    return Err(LunarTheoryCatalogValidationError::DuplicateAlias { alias });
                }
            }
            for alias in other.specification.source_aliases {
                if entry.specification.matches_alias(alias)
                    || entry
                        .specification
                        .source_identifier
                        .eq_ignore_ascii_case(alias)
                    || entry.specification.model_name.eq_ignore_ascii_case(alias)
                    || entry
                        .specification
                        .source_family
                        .label()
                        .eq_ignore_ascii_case(alias)
                {
                    return Err(LunarTheoryCatalogValidationError::DuplicateAlias { alias });
                }
            }
        }
    }

    let selected_entry = catalog
        .iter()
        .find(|entry| entry.selected)
        .ok_or(LunarTheoryCatalogValidationError::NoSelectedEntry)?;
    let selection = selected_entry.specification.source_selection();
    if lunar_theory_catalog_entry_for_selection(selection) != Some(*selected_entry) {
        return Err(
            LunarTheoryCatalogValidationError::SelectedEntryDoesNotRoundTrip {
                source_identifier: selected_entry.specification.source_identifier,
            },
        );
    }

    Ok(())
}

/// Validates the structured lunar-theory catalog for round-trip, alias/core-label uniqueness,
/// and disjoint supported/unsupported body coverage.
pub fn validate_lunar_theory_catalog() -> Result<(), LunarTheoryCatalogValidationError> {
    validate_lunar_theory_catalog_entries(lunar_theory_catalog())
}

/// A compact validation summary for the current lunar-theory catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarTheoryCatalogValidationSummary {
    /// Number of catalog entries.
    pub entry_count: usize,
    /// Number of selected catalog entries.
    pub selected_count: usize,
    /// The selected lunar-theory source selection, when one is present.
    pub selected_source: Option<LunarTheorySourceSelection>,
    /// Result of validating the structured lunar-theory catalog.
    pub validation_result: Result<(), LunarTheoryCatalogValidationError>,
}

/// Returns a compact validation summary for the current lunar-theory catalog.
pub fn lunar_theory_catalog_validation_summary() -> LunarTheoryCatalogValidationSummary {
    let catalog = lunar_theory_catalog();
    let selected_count = catalog.iter().filter(|entry| entry.selected).count();
    let selected_source = catalog
        .iter()
        .find(|entry| entry.selected)
        .map(|entry| entry.specification.source_selection());

    LunarTheoryCatalogValidationSummary {
        entry_count: catalog.len(),
        selected_count,
        selected_source,
        validation_result: validate_lunar_theory_catalog(),
    }
}

impl LunarTheoryCatalogValidationSummary {
    /// Returns the compact release-facing summary line for the catalog validation state.
    pub fn summary_line(&self) -> String {
        let selected_source_summary = self
            .selected_source
            .map(|source| format!("{} [{}]", source.identifier, source.family_label()))
            .unwrap_or_else(|| "none".to_string());
        let selected_catalog_key = self
            .selected_source
            .map(|source| source.catalog_key().to_string())
            .unwrap_or_else(|| "none".to_string());
        let selected_family_key = self
            .selected_source
            .map(|source| source.family_key().to_string())
            .unwrap_or_else(|| "none".to_string());
        let selected_alias_count = self
            .selected_source
            .map(|source| source.source_aliases.len())
            .unwrap_or(0);

        match &self.validation_result {
            Ok(()) => format!(
                "lunar theory catalog validation: ok ({} entries, {} selected; selected source: {}; selected key: {}; selected family key: {}; aliases={}; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)",
                self.entry_count,
                self.selected_count,
                selected_source_summary,
                selected_catalog_key,
                selected_family_key,
                selected_alias_count,
            ),
            Err(error) => format!(
                "lunar theory catalog validation: error: {} ({} entries, {} selected; selected source: {}; selected key: {}; selected family key: {}; aliases={})",
                error,
                self.entry_count,
                self.selected_count,
                selected_source_summary,
                selected_catalog_key,
                selected_family_key,
                selected_alias_count,
            ),
        }
    }
}

impl fmt::Display for LunarTheoryCatalogValidationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the catalog entry matching the provided source identifier, when present.
pub fn lunar_theory_catalog_entry_for_source_identifier(
    source_identifier: &str,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceIdentifier(source_identifier))
}

/// Returns the catalog entry matching the provided model name, when present.
pub fn lunar_theory_catalog_entry_for_model_name(
    model_name: &str,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::ModelName(model_name))
}

/// Returns the catalog entry matching the structured source family, when present.
pub fn lunar_theory_catalog_entry_for_source_family(
    source_family: LunarTheorySourceFamily,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceFamily(source_family))
}

/// Returns the catalog entry matching the provided source-family label, when present.
pub fn lunar_theory_catalog_entry_for_family_label(
    family_label: &str,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::FamilyLabel(family_label))
}

/// Returns the catalog entry matching the provided alias, when present.
pub fn lunar_theory_catalog_entry_for_alias(alias: &str) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::Alias(alias))
}

/// Returns the catalog entry matching the provided typed lookup key, when present.
pub fn lunar_theory_catalog_entry_for_key(
    key: LunarTheoryCatalogKey<'_>,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog()
        .iter()
        .copied()
        .find(|entry| key.matches(entry))
}

/// Returns the catalog entry matching the provided label, when present.
pub fn lunar_theory_catalog_entry_for_label(label: &str) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_source_identifier(label)
        .or_else(|| lunar_theory_catalog_entry_for_model_name(label))
        .or_else(|| lunar_theory_catalog_entry_for_family_label(label))
        .or_else(|| lunar_theory_catalog_entry_for_alias(label))
}

/// Returns the current lunar-theory specification matching the provided label, when present.
pub fn resolve_lunar_theory(label: &str) -> Option<LunarTheorySpecification> {
    lunar_theory_catalog_entry_for_label(label).map(|entry| entry.specification)
}

/// Returns the catalog entry matching the provided source selection, when present.
pub fn lunar_theory_catalog_entry_for_selection(
    selection: LunarTheorySourceSelection,
) -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_key(selection.catalog_key())
}

/// Returns the catalog entry matching the current lunar-theory selection, when present.
pub fn lunar_theory_catalog_entry_for_current_selection() -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_selection(lunar_theory_source_selection())
}

/// Returns the current lunar-theory catalog entry, when present.
#[doc(alias = "lunar_theory_catalog_entry_for_current_selection")]
pub fn current_lunar_theory_catalog_entry() -> Option<LunarTheoryCatalogEntry> {
    lunar_theory_catalog_entry_for_current_selection()
}

/// Returns the current lunar-theory specification matching the provided alias, when present.
pub fn resolve_lunar_theory_by_alias(alias: &str) -> Option<LunarTheorySpecification> {
    lunar_theory_catalog_entry_for_alias(alias).map(|entry| entry.specification)
}

/// Returns the current lunar-theory specification matching the structured source family, when present.
pub fn resolve_lunar_theory_by_family(
    source_family: LunarTheorySourceFamily,
) -> Option<LunarTheorySpecification> {
    lunar_theory_catalog_entry_for_source_family(source_family).map(|entry| entry.specification)
}

/// Returns the current lunar-theory specification matching the provided typed lookup key, when present.
pub fn resolve_lunar_theory_by_key(
    key: LunarTheoryCatalogKey<'_>,
) -> Option<LunarTheorySpecification> {
    lunar_theory_catalog_entry_for_key(key).map(|entry| entry.specification)
}

/// Returns the current lunar-theory specification matching the provided source selection, when present.
pub fn resolve_lunar_theory_by_selection(
    selection: LunarTheorySourceSelection,
) -> Option<LunarTheorySpecification> {
    resolve_lunar_theory_by_key(selection.catalog_key())
}

/// Returns the bodies/channels the current lunar-theory baseline explicitly supports.
pub fn lunar_theory_supported_bodies() -> &'static [CelestialBody] {
    lunar_theory_specification().supported_bodies
}

/// Returns the bodies/channels the current lunar-theory baseline explicitly rejects.
pub fn lunar_theory_unsupported_bodies() -> &'static [CelestialBody] {
    lunar_theory_specification().unsupported_bodies
}

/// A compact summary of the current lunar-theory catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryCatalogSummary {
    /// Number of catalog entries.
    pub entry_count: usize,
    /// Number of selected catalog entries.
    pub selected_count: usize,
    /// Identifier of the selected lunar-theory baseline.
    pub selected_source_identifier: &'static str,
    /// Structured source family of the selected lunar-theory baseline.
    pub selected_source_family: LunarTheorySourceFamily,
    /// Human-readable source family label of the selected lunar-theory baseline.
    pub selected_source_family_label: &'static str,
    /// Typed lookup key for the selected lunar-theory baseline.
    pub selected_catalog_key: LunarTheoryCatalogKey<'static>,
    /// Typed family lookup key for the selected lunar-theory baseline.
    pub selected_family_key: LunarTheoryCatalogKey<'static>,
    /// Number of aliases documented for the selected baseline.
    pub selected_alias_count: usize,
    /// Number of bodies/channels explicitly supported by the selected baseline.
    pub selected_supported_body_count: usize,
    /// Number of bodies/channels explicitly unsupported by the selected baseline.
    pub selected_unsupported_body_count: usize,
}

/// Validation error for a lunar-theory catalog summary that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryCatalogSummaryValidationError {
    /// A rendered summary field no longer matches the current lunar-theory catalog.
    FieldOutOfSync {
        /// Name of the rendered field that drifted from its current backend-owned value.
        field: &'static str,
    },
    /// The current catalog no longer contains a selected entry.
    MissingSelectedEntry,
}

impl fmt::Display for LunarTheoryCatalogSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar catalog summary field `{field}` is out of sync with the current catalog"
            ),
            Self::MissingSelectedEntry => {
                f.write_str("the lunar catalog no longer contains a selected entry")
            }
        }
    }
}

impl std::error::Error for LunarTheoryCatalogSummaryValidationError {}

/// Returns a compact summary of the current lunar-theory catalog.
pub fn lunar_theory_catalog_summary() -> LunarTheoryCatalogSummary {
    let catalog = lunar_theory_catalog();
    let selected_entry = catalog
        .iter()
        .find(|entry| entry.selected)
        .copied()
        .unwrap_or(LUNAR_THEORY_CATALOG[0]);

    let selected_source = selected_entry.specification.source_selection();

    LunarTheoryCatalogSummary {
        entry_count: catalog.len(),
        selected_count: catalog.iter().filter(|entry| entry.selected).count(),
        selected_source_identifier: selected_entry.specification.source_identifier,
        selected_source_family: selected_entry.specification.source_family,
        selected_source_family_label: selected_entry.specification.source_family.label(),
        selected_catalog_key: selected_source.catalog_key(),
        selected_family_key: selected_source.family_key(),
        selected_alias_count: selected_entry.specification.source_aliases.len(),
        selected_supported_body_count: selected_entry.specification.supported_bodies.len(),
        selected_unsupported_body_count: selected_entry.specification.unsupported_bodies.len(),
    }
}

impl LunarTheoryCatalogSummary {
    /// Returns `Ok(())` when the summary still matches the current lunar-theory catalog.
    pub fn validate(&self) -> Result<(), LunarTheoryCatalogSummaryValidationError> {
        let catalog = lunar_theory_catalog();
        let selected_count = catalog.iter().filter(|entry| entry.selected).count();
        let Some(selected_entry) = catalog.iter().find(|entry| entry.selected) else {
            return Err(LunarTheoryCatalogSummaryValidationError::MissingSelectedEntry);
        };

        if self.entry_count != catalog.len() {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "entry_count",
            });
        }
        if self.selected_count != selected_count {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_count",
            });
        }
        if self.selected_source_identifier != selected_entry.specification.source_identifier {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_source_identifier",
            });
        }
        if self.selected_source_family != selected_entry.specification.source_family {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_source_family",
            });
        }
        if self.selected_source_family_label != selected_entry.specification.source_family.label() {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_source_family_label",
            });
        }
        if self.selected_catalog_key
            != selected_entry
                .specification
                .source_selection()
                .catalog_key()
        {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_catalog_key",
            });
        }
        if self.selected_family_key != selected_entry.specification.source_selection().family_key()
        {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_family_key",
            });
        }
        if self.selected_alias_count != selected_entry.specification.source_aliases.len() {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_alias_count",
            });
        }
        if self.selected_supported_body_count != selected_entry.specification.supported_bodies.len()
        {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_supported_body_count",
            });
        }
        if self.selected_unsupported_body_count
            != selected_entry.specification.unsupported_bodies.len()
        {
            return Err(LunarTheoryCatalogSummaryValidationError::FieldOutOfSync {
                field: "selected_unsupported_body_count",
            });
        }

        Ok(())
    }

    /// Returns the compact release-facing summary line for the current lunar catalog.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarTheoryCatalogSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the compact release-facing summary line for the current lunar catalog.
    pub fn summary_line(&self) -> String {
        let entry_label = if self.entry_count == 1 {
            "entry"
        } else {
            "entries"
        };
        let selected_label = if self.selected_count == 1 {
            "selected entry"
        } else {
            "selected entries"
        };

        format!(
            "lunar theory catalog: {} {}, {} {}; selected source: {} [{}]; selected key: {}; selected family key: {}; aliases={}; supported bodies={}; unsupported bodies={}",
            self.entry_count,
            entry_label,
            self.selected_count,
            selected_label,
            self.selected_source_identifier,
            self.selected_source_family_label,
            self.selected_catalog_key,
            self.selected_family_key,
            self.selected_alias_count,
            self.selected_supported_body_count,
            self.selected_unsupported_body_count,
        )
    }
}

impl fmt::Display for LunarTheoryCatalogSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A compact capability summary for the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryCapabilitySummary {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Stable identifier for the selected lunar-theory baseline.
    pub source_identifier: &'static str,
    /// Structured source family for the selected lunar-theory baseline.
    pub source_family: LunarTheorySourceFamily,
    /// Human-readable source family label.
    pub source_family_label: &'static str,
    /// Bodies/channels the current baseline explicitly supports.
    pub supported_bodies: &'static [CelestialBody],
    /// Bodies/channels that are explicitly unsupported by this baseline.
    pub unsupported_bodies: &'static [CelestialBody],
    /// Number of supported lunar bodies/channels.
    pub supported_body_count: usize,
    /// Number of explicitly unsupported lunar bodies/channels.
    pub unsupported_body_count: usize,
    /// Number of supported coordinate frames.
    pub supported_frame_count: usize,
    /// Number of supported time scales.
    pub supported_time_scale_count: usize,
    /// Number of supported zodiac modes.
    pub supported_zodiac_mode_count: usize,
    /// Number of supported apparentness modes.
    pub supported_apparentness_count: usize,
    /// Whether the current baseline accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
    /// Structured validation window represented by the current evidence slice.
    pub validation_window: pleiades_types::TimeRange,
    /// Whether the current lunar-theory catalog validates cleanly.
    pub catalog_validation_ok: bool,
}

/// Validation error for a lunar-theory capability summary that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryCapabilitySummaryValidationError {
    /// A rendered summary field no longer matches the current lunar-theory selection.
    FieldOutOfSync {
        /// Name of the rendered field that drifted from its current backend-owned value.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheoryCapabilitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar capability summary field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryCapabilitySummaryValidationError {}

/// Returns the compact capability summary for the current lunar-theory selection.
pub fn lunar_theory_capability_summary() -> LunarTheoryCapabilitySummary {
    let theory = lunar_theory_specification();
    LunarTheoryCapabilitySummary {
        model_name: theory.model_name,
        source_identifier: theory.source_identifier,
        source_family: theory.source_family,
        source_family_label: theory.source_family.label(),
        supported_bodies: theory.supported_bodies,
        unsupported_bodies: theory.unsupported_bodies,
        supported_body_count: theory.supported_bodies.len(),
        unsupported_body_count: theory.unsupported_bodies.len(),
        supported_frame_count: theory.supported_frames.len(),
        supported_time_scale_count: theory.supported_time_scales.len(),
        supported_zodiac_mode_count: theory.supported_zodiac_modes.len(),
        supported_apparentness_count: theory.supported_apparentness.len(),
        supports_topocentric_observer: theory.request_policy.supports_topocentric_observer,
        validation_window: theory.validation_window,
        catalog_validation_ok: validate_lunar_theory_catalog().is_ok(),
    }
}

impl LunarTheoryCapabilitySummary {
    /// Returns the compact release-facing summary line for the current lunar selection.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarTheoryCapabilitySummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the compact release-facing summary line for the current lunar selection.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar capability summary: {} [{}; family: {}] bodies={} ({}); unsupported={} ({}); frames={} time scales={} zodiac modes={} apparentness={} topocentric observer={} validation window={}; catalog validation={}",
            self.model_name,
            self.source_identifier,
            self.source_family_label,
            self.supported_body_count,
            crate::format_bodies(self.supported_bodies),
            self.unsupported_body_count,
            crate::format_bodies(self.unsupported_bodies),
            self.supported_frame_count,
            self.supported_time_scale_count,
            self.supported_zodiac_mode_count,
            self.supported_apparentness_count,
            self.supports_topocentric_observer,
            self.validation_window,
            if self.catalog_validation_ok { "ok" } else { "error" },
        )
    }

    /// Returns `Ok(())` when the summary still matches the current lunar-theory selection.
    pub fn validate(&self) -> Result<(), LunarTheoryCapabilitySummaryValidationError> {
        let theory = lunar_theory_specification();

        if self.model_name != theory.model_name {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "model_name",
                },
            );
        }
        if self.source_identifier != theory.source_identifier {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "source_identifier",
                },
            );
        }
        if self.source_family != theory.source_family {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "source_family",
                },
            );
        }
        if self.source_family_label != theory.source_family.label() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "source_family_label",
                },
            );
        }
        if self.supported_bodies != theory.supported_bodies {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_bodies",
                },
            );
        }
        if self.unsupported_bodies != theory.unsupported_bodies {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_bodies",
                },
            );
        }
        if self.supported_body_count != theory.supported_bodies.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_body_count",
                },
            );
        }
        if self.unsupported_body_count != theory.unsupported_bodies.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "unsupported_body_count",
                },
            );
        }
        if self.supported_frame_count != theory.supported_frames.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_frame_count",
                },
            );
        }
        if self.supported_time_scale_count != theory.supported_time_scales.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_time_scale_count",
                },
            );
        }
        if self.supported_zodiac_mode_count != theory.supported_zodiac_modes.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_zodiac_mode_count",
                },
            );
        }
        if self.supported_apparentness_count != theory.supported_apparentness.len() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supported_apparentness_count",
                },
            );
        }
        if self.supports_topocentric_observer != theory.request_policy.supports_topocentric_observer
        {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "supports_topocentric_observer",
                },
            );
        }
        if self.validation_window != theory.validation_window {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "validation_window",
                },
            );
        }
        if self.catalog_validation_ok != validate_lunar_theory_catalog().is_ok() {
            return Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "catalog_validation_ok",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarTheoryCapabilitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A compact release-facing summary for the current lunar-theory limitations posture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryLimitationsSummary {
    /// Human-readable baseline label.
    pub baseline_label: &'static str,
    /// Built-in lunar channels currently supported by the baseline.
    pub supported_bodies: &'static [CelestialBody],
    /// Built-in lunar channels intentionally left unsupported.
    pub unsupported_bodies: &'static [CelestialBody],
    /// Release-facing error-envelope label for the reference channel.
    pub reference_envelope_label: &'static str,
    /// Release-facing error-envelope label for the equatorial channel.
    pub equatorial_envelope_label: &'static str,
}

/// Validation error for a lunar-theory limitations summary that drifted from the current baseline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryLimitationsSummaryValidationError {
    /// A rendered limitations field no longer matches the current baseline.
    FieldOutOfSync {
        /// Name of the rendered field that drifted from its current backend-owned value.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheoryLimitationsSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar theory limitations summary field `{field}` is out of sync with the current baseline"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryLimitationsSummaryValidationError {}

impl LunarTheoryLimitationsSummary {
    /// Returns the compact release-facing limitations line.
    ///
    /// This mirrors the release-facing free-function renderer relocated to
    /// `pleiades-validate`'s posture module (report-surface relocation
    /// program, Slice B); the two evidence-envelope lines below are rebuilt
    /// from the retained `crate::evidence` constructors and their pub
    /// `validated_summary_line()` bridge (unchanged wording) because this
    /// inherent method (and `Display`) must keep working without a
    /// dependency on `pleiades-validate`.
    pub fn summary_line(&self) -> String {
        let reference_envelope = match crate::evidence::lunar_reference_evidence_envelope() {
            Some(envelope) => match envelope.validated_summary_line() {
                Ok(summary_line) => summary_line,
                Err(error) => format!("lunar reference error envelope: unavailable ({error})"),
            },
            None => "lunar reference error envelope: unavailable".to_string(),
        };
        let equatorial_envelope =
            match crate::evidence::lunar_equatorial_reference_evidence_envelope() {
                Some(envelope) => match envelope.validated_summary_line() {
                    Ok(summary_line) => summary_line,
                    Err(error) => {
                        format!("lunar equatorial reference error envelope: unavailable ({error})")
                    }
                },
                None => "lunar equatorial reference error envelope: unavailable".to_string(),
            };

        format!(
            "lunar theory limitations: {}; supported bodies: {}; unsupported bodies: {}; release-grade evidence by channel: {}; {}",
            self.baseline_label,
            crate::format_bodies(self.supported_bodies),
            crate::format_bodies(self.unsupported_bodies),
            reference_envelope,
            equatorial_envelope,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current lunar baseline.
    pub fn validate(&self) -> Result<(), LunarTheoryLimitationsSummaryValidationError> {
        let theory = lunar_theory_specification();
        let expected = lunar_theory_limitations_summary();

        if self.baseline_label != expected.baseline_label {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "baseline_label",
                },
            );
        }
        if self.supported_bodies != expected.supported_bodies {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "supported_bodies",
                },
            );
        }
        if self.unsupported_bodies != expected.unsupported_bodies {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "unsupported_bodies",
                },
            );
        }
        if self.reference_envelope_label != expected.reference_envelope_label {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "reference_envelope_label",
                },
            );
        }
        if self.equatorial_envelope_label != expected.equatorial_envelope_label {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "equatorial_envelope_label",
                },
            );
        }
        if self.supported_bodies != theory.supported_bodies {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "supported_bodies_vs_specification",
                },
            );
        }
        if self.unsupported_bodies != theory.unsupported_bodies {
            return Err(
                LunarTheoryLimitationsSummaryValidationError::FieldOutOfSync {
                    field: "unsupported_bodies_vs_specification",
                },
            );
        }

        Ok(())
    }

    /// Returns the compact limitations line after validating the current baseline.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarTheoryLimitationsSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for LunarTheoryLimitationsSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the structured lunar-theory limitations posture.
pub fn lunar_theory_limitations_summary() -> LunarTheoryLimitationsSummary {
    let theory = lunar_theory_specification();

    LunarTheoryLimitationsSummary {
        baseline_label: theory.model_name,
        supported_bodies: theory.supported_bodies,
        unsupported_bodies: theory.unsupported_bodies,
        reference_envelope_label: "lunar reference error envelope",
        equatorial_envelope_label: "lunar equatorial reference error envelope",
    }
}
