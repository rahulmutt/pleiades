use core::fmt;

use pleiades_types::TimeRange;

use crate::specification::lunar_theory_specification;

/// Structured source family for the current lunar-theory selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LunarTheorySourceFamily {
    /// Compact Meeus-style truncated analytical baseline.
    MeeusStyleTruncatedAnalyticalBaseline,
}

impl LunarTheorySourceFamily {
    /// Human-readable label for the current source family.
    pub const fn label(self) -> &'static str {
        match self {
            Self::MeeusStyleTruncatedAnalyticalBaseline => {
                "Meeus-style truncated analytical baseline"
            }
        }
    }
}

impl fmt::Display for LunarTheorySourceFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Typed lookup keys for resolving a lunar-theory catalog entry.
///
/// This keeps future source-backed lunar catalogs explicit about which label
/// family is being matched instead of relying only on the generic string helper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LunarTheoryCatalogKey<'a> {
    /// Match the stable source identifier.
    SourceIdentifier(&'a str),
    /// Match the human-readable model name.
    ModelName(&'a str),
    /// Match the structured source-family enum.
    SourceFamily(LunarTheorySourceFamily),
    /// Match the source-family label.
    FamilyLabel(&'a str),
    /// Match a documented short alias.
    Alias(&'a str),
}

impl<'a> LunarTheoryCatalogKey<'a> {
    /// Returns `true` when this key matches the provided catalog entry.
    pub fn matches(self, entry: &crate::catalog::LunarTheoryCatalogEntry) -> bool {
        match self {
            Self::SourceIdentifier(source_identifier) => entry
                .specification
                .source_identifier
                .eq_ignore_ascii_case(source_identifier),
            Self::ModelName(model_name) => entry
                .specification
                .model_name
                .eq_ignore_ascii_case(model_name),
            Self::SourceFamily(source_family) => entry.specification.source_family == source_family,
            Self::FamilyLabel(family_label) => entry
                .specification
                .source_family
                .label()
                .eq_ignore_ascii_case(family_label),
            Self::Alias(alias) => entry.specification.matches_alias(alias),
        }
    }
}

impl fmt::Display for LunarTheoryCatalogKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceIdentifier(source_identifier) => {
                write!(f, "source identifier={source_identifier}")
            }
            Self::ModelName(model_name) => write!(f, "model name={model_name}"),
            Self::SourceFamily(source_family) => write!(f, "source family={source_family}"),
            Self::FamilyLabel(family_label) => write!(f, "family label={family_label}"),
            Self::Alias(alias) => write!(f, "alias={alias}"),
        }
    }
}

/// Structured source selection for the current lunar-theory baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheorySourceSelection {
    /// Structured source family for the current baseline.
    pub family: LunarTheorySourceFamily,
    /// Alternate names or documented aliases for the current baseline.
    pub source_aliases: &'static [&'static str],
    /// Stable identifier for the current baseline.
    pub identifier: &'static str,
    /// Canonical bibliographic citation for the current baseline.
    pub citation: &'static str,
    /// Human-readable source/provenance note for the current baseline.
    pub material: &'static str,
    /// Redistribution or licensing posture for the current baseline.
    pub redistribution_note: &'static str,
    /// Licensing or provenance summary for the current baseline.
    pub license_note: &'static str,
}

impl LunarTheorySourceSelection {
    /// Returns the human-readable family label for the current source selection.
    pub const fn family_label(self) -> &'static str {
        self.family.label()
    }

    /// Returns the typed catalog key for the current source selection.
    pub const fn catalog_key(self) -> LunarTheoryCatalogKey<'static> {
        LunarTheoryCatalogKey::SourceIdentifier(self.identifier)
    }

    /// Returns the typed family key for the current source selection.
    pub const fn family_key(self) -> LunarTheoryCatalogKey<'static> {
        LunarTheoryCatalogKey::SourceFamily(self.family)
    }

    /// Returns a compact summary line for the current source selection.
    pub fn summary_line(&self) -> String {
        let aliases = if self.source_aliases.is_empty() {
            "none".to_string()
        } else {
            self.source_aliases.join(", ")
        };

        format!(
            "lunar source selection: {} [selected key: {}; family key: {}; family: {}]; aliases: {}; citation: {}; provenance: {}; redistribution: {}; license: {}",
            self.identifier,
            self.catalog_key(),
            self.family_key(),
            self.family_label(),
            aliases,
            self.citation,
            self.material,
            self.redistribution_note,
            self.license_note,
        )
    }

    /// Returns the compact summary line after validating the selection against the current baseline.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarTheorySourceSelectionValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the selection still matches the current lunar-theory baseline.
    pub fn validate(&self) -> Result<(), LunarTheorySourceSelectionValidationError> {
        let source = lunar_theory_source_selection();

        if self.family != source.family {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "family",
            });
        }
        if self.source_aliases != source.source_aliases {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "source_aliases",
            });
        }
        if self.identifier != source.identifier {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "identifier",
            });
        }
        if self.citation != source.citation {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "citation",
            });
        }
        if self.material != source.material {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "material",
            });
        }
        if self.redistribution_note != source.redistribution_note {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "redistribution_note",
            });
        }
        if self.license_note != source.license_note {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "license_note",
            });
        }
        if crate::catalog::resolve_lunar_theory_by_key(self.catalog_key())
            != Some(lunar_theory_specification())
        {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "catalog_key",
            });
        }
        if crate::catalog::resolve_lunar_theory_by_key(self.family_key())
            != Some(lunar_theory_specification())
        {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "family_key",
            });
        }

        Ok(())
    }
}

impl fmt::Display for LunarTheorySourceSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a lunar-theory source selection that drifted from the current baseline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheorySourceSelectionValidationError {
    /// A rendered source-selection field no longer matches the current lunar-theory selection.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarTheorySourceSelectionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar source selection field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheorySourceSelectionValidationError {}

/// Compact source-selection summary for the current lunar-theory baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheorySourceSummary {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Stable identifier for the current baseline.
    pub source_identifier: &'static str,
    /// Typed catalog key for the current source selection.
    pub catalog_key: LunarTheoryCatalogKey<'static>,
    /// Typed family key for the current source selection.
    pub source_family_key: LunarTheoryCatalogKey<'static>,
    /// Structured source family for the current source selection.
    pub source_family: LunarTheorySourceFamily,
    /// Human-readable family label for the current source selection.
    pub source_family_label: &'static str,
    /// Alternate names or documented aliases for the current baseline.
    pub source_aliases: &'static [&'static str],
    /// Canonical bibliographic citation for the current baseline.
    pub citation: &'static str,
    /// Human-readable source/provenance note for the current baseline.
    pub provenance: &'static str,
    /// Structured validation window represented by the current evidence slice.
    pub validation_window: TimeRange,
    /// Redistribution or licensing posture for the current baseline.
    pub redistribution_note: &'static str,
    /// Licensing or provenance summary for the current baseline.
    pub license_note: &'static str,
}

/// Validation error for a lunar-theory source summary that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheorySourceSummaryValidationError {
    /// A rendered summary field no longer matches the current lunar-theory selection.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarTheorySourceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar source summary field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheorySourceSummaryValidationError {}

impl LunarTheorySourceSummary {
    /// Returns `Ok(())` when the summary still matches the current lunar-theory selection.
    pub fn validate(&self) -> Result<(), LunarTheorySourceSummaryValidationError> {
        let theory = lunar_theory_specification();
        let source = lunar_theory_source_selection();

        if self.model_name != theory.model_name {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "model_name",
            });
        }
        if self.source_identifier != source.identifier {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "source_identifier",
            });
        }
        if self.catalog_key != source.catalog_key() {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "catalog_key",
            });
        }
        if self.source_family != source.family {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "source_family",
            });
        }
        if self.source_family_key != source.family_key() {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "source_family_key",
            });
        }
        if self.source_family_label != source.family_label() {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "source_family_label",
            });
        }
        if self.source_aliases != source.source_aliases {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "source_aliases",
            });
        }
        if self.citation != source.citation {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "citation",
            });
        }
        if self.provenance != source.material {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "provenance",
            });
        }
        if self.validation_window != theory.validation_window {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "validation_window",
            });
        }
        if self.redistribution_note != source.redistribution_note {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "redistribution_note",
            });
        }
        if self.license_note != source.license_note {
            return Err(LunarTheorySourceSummaryValidationError::FieldOutOfSync {
                field: "license_note",
            });
        }

        Ok(())
    }

    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarTheorySourceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns the compact release-facing summary line for the current lunar source selection.
    pub fn summary_line(&self) -> String {
        let aliases = if self.source_aliases.is_empty() {
            "none".to_string()
        } else {
            self.source_aliases.join(", ")
        };

        format!(
            "lunar source selection: {} [selected key: {}; family key: {}; family: {}]; aliases: {}; citation: {}; provenance: {}; validation window: {}; redistribution: {}; license: {}",
            self.model_name,
            self.catalog_key,
            self.source_family_key,
            self.source_family_label,
            aliases,
            self.citation,
            self.provenance,
            self.validation_window,
            self.redistribution_note,
            self.license_note,
        )
    }
}

impl fmt::Display for LunarTheorySourceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Structured summary of the current lunar-theory source family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheorySourceFamilySummary {
    /// Structured source family for the current lunar baseline.
    pub family: LunarTheorySourceFamily,
    /// Human-readable source family label for the current lunar baseline.
    pub family_label: &'static str,
    /// Stable identifier for the selected lunar baseline.
    pub selected_source_identifier: &'static str,
    /// Human-readable model name for the selected lunar baseline.
    pub selected_model_name: &'static str,
    /// Typed catalog key for the selected lunar baseline.
    pub selected_catalog_key: LunarTheoryCatalogKey<'static>,
    /// Typed family key for the selected lunar baseline.
    pub selected_family_key: LunarTheoryCatalogKey<'static>,
    /// Number of aliases documented for the selected lunar baseline.
    pub selected_alias_count: usize,
}

/// Validation error for a lunar-theory source-family summary that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheorySourceFamilySummaryValidationError {
    /// A rendered summary field no longer matches the current lunar-theory family.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarTheorySourceFamilySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar source-family summary field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheorySourceFamilySummaryValidationError {}

impl LunarTheorySourceFamilySummary {
    /// Returns a compact summary line for the current lunar source family.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar source family: {} [selected source={}; selected model={}; selected key={}; selected family key={}; aliases={}]",
            self.family_label,
            self.selected_source_identifier,
            self.selected_model_name,
            self.selected_catalog_key,
            self.selected_family_key,
            self.selected_alias_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current lunar-theory family.
    pub fn validate(&self) -> Result<(), LunarTheorySourceFamilySummaryValidationError> {
        let theory = lunar_theory_specification();
        let source = lunar_theory_source_selection();

        if self.family != theory.source_family {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync { field: "family" },
            );
        }
        if self.family_label != theory.source_family.label() {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "family_label",
                },
            );
        }
        if self.selected_source_identifier != source.identifier {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "selected_source_identifier",
                },
            );
        }
        if self.selected_model_name != theory.model_name {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "selected_model_name",
                },
            );
        }
        if self.selected_catalog_key != source.catalog_key() {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "selected_catalog_key",
                },
            );
        }
        if self.selected_family_key != source.family_key() {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "selected_family_key",
                },
            );
        }
        if self.selected_alias_count != theory.source_aliases.len() {
            return Err(
                LunarTheorySourceFamilySummaryValidationError::FieldOutOfSync {
                    field: "selected_alias_count",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarTheorySourceFamilySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the current lunar-theory source family.
pub const fn lunar_theory_source_family() -> LunarTheorySourceFamily {
    LunarTheorySourceFamily::MeeusStyleTruncatedAnalyticalBaseline
}

/// Returns the structured source selection for the current lunar-theory baseline.
pub fn lunar_theory_source_selection() -> LunarTheorySourceSelection {
    lunar_theory_specification().source_selection()
}

/// Returns a compact source-selection summary for the current lunar theory.
pub fn lunar_theory_source_summary() -> LunarTheorySourceSummary {
    let theory = lunar_theory_specification();
    let source = lunar_theory_source_selection();

    LunarTheorySourceSummary {
        model_name: theory.model_name,
        source_identifier: source.identifier,
        catalog_key: source.catalog_key(),
        source_family_key: source.family_key(),
        source_family: source.family,
        source_family_label: source.family_label(),
        source_aliases: source.source_aliases,
        citation: source.citation,
        provenance: source.material,
        validation_window: theory.validation_window,
        redistribution_note: source.redistribution_note,
        license_note: source.license_note,
    }
}

/// Returns a structured summary of the current lunar source family.
pub fn lunar_theory_source_family_summary() -> LunarTheorySourceFamilySummary {
    let theory = lunar_theory_specification();
    let source = lunar_theory_source_selection();

    LunarTheorySourceFamilySummary {
        family: theory.source_family,
        family_label: theory.source_family.label(),
        selected_source_identifier: source.identifier,
        selected_model_name: theory.model_name,
        selected_catalog_key: source.catalog_key(),
        selected_family_key: source.family_key(),
        selected_alias_count: theory.source_aliases.len(),
    }
}
