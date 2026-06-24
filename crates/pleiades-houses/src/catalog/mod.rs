//! House-system catalog definitions and compatibility metadata.
//!
//! Enumerates the built-in house systems, their common aliases, formula-family
//! tags, latitude-sensitive notes, and validation utilities.

use core::fmt;
use std::collections::BTreeSet;

use pleiades_types::HouseSystem;

/// A catalog entry for a built-in house system.
#[derive(Clone, Debug, PartialEq)]
pub struct HouseSystemDescriptor {
    /// The strongly typed system identifier.
    pub system: HouseSystem,
    /// The canonical name used in compatibility profiles.
    pub canonical_name: &'static str,
    /// Alternate names or software-specific aliases.
    pub aliases: &'static [&'static str],
    /// Short notes about formula family or interoperability constraints.
    pub notes: &'static str,
    /// Whether the system is known to have latitude-sensitive failure modes.
    pub latitude_sensitive: bool,
    /// Maximum |geographic latitude| (degrees) at which this system yields a
    /// well-defined cusp set. `Some(bound)` only for latitude-sensitive systems;
    /// beyond it `calculate_houses` returns `InvalidLatitude` under the strict
    /// default. `None` for systems that are defined at every latitude.
    pub max_abs_latitude_deg: Option<f64>,
    /// The compatibility claim tier for this built-in entry.
    pub claim_tier: pleiades_types::CompatibilityClaimTier,
}

/// Coarse formula-family tags for the built-in house catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum HouseFormulaFamily {
    /// Equal-house variants anchored to different reference points.
    Equal,
    /// Whole-sign house placement.
    WholeSign,
    /// Quadrant-style house placement with intermediate cusps.
    Quadrant,
    /// Equatorial-projection variants.
    EquatorialProjection,
    /// Great-circle or horizon-style variants.
    GreatCircle,
    /// Solar-arc or Sun-centered variants.
    SolarArc,
    /// Sector-based variants.
    Sector,
    /// Custom, user-defined house systems.
    Custom,
    /// A future built-in family that is not modeled yet.
    Unknown,
}

impl fmt::Display for HouseFormulaFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Equal => "Equal",
            Self::WholeSign => "Whole Sign",
            Self::Quadrant => "Quadrant",
            Self::EquatorialProjection => "Equatorial projection",
            Self::GreatCircle => "Great-circle",
            Self::SolarArc => "Solar arc",
            Self::Sector => "Sector",
            Self::Custom => "Custom",
            Self::Unknown => "Unknown",
        };
        f.write_str(label)
    }
}

impl HouseSystemDescriptor {
    /// Creates a descriptor that makes no numeric compatibility claim.
    pub const fn new(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
        max_abs_latitude_deg: Option<f64>,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
            max_abs_latitude_deg,
            claim_tier: pleiades_types::CompatibilityClaimTier::DescriptorOnly,
        }
    }

    /// Creates a descriptor that asserts release-grade numeric compatibility.
    /// Use only for entries with passing SE numeric-gate evidence.
    pub const fn new_release_grade(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
        max_abs_latitude_deg: Option<f64>,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
            max_abs_latitude_deg,
            claim_tier: pleiades_types::CompatibilityClaimTier::ReleaseGradeNumeric,
        }
    }

    /// Validates the descriptor-local metadata invariants.
    pub fn validate(&self) -> Result<(), HouseCatalogValidationError> {
        if self.canonical_name.trim().is_empty()
            || has_surrounding_whitespace(self.canonical_name)
            || contains_line_break(self.canonical_name)
        {
            return Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                label: self.canonical_name,
                field: "canonical name",
            });
        }

        for alias in self.aliases {
            if alias.trim().is_empty()
                || has_surrounding_whitespace(alias)
                || contains_line_break(alias)
            {
                return Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                    label: alias,
                    field: "alias",
                });
            }
        }

        if self.notes.trim().is_empty()
            || (!self.notes.is_empty() && self.notes.trim() != self.notes)
            || contains_line_break(self.notes)
        {
            return Err(HouseCatalogValidationError::DescriptorNotesNotNormalized {
                label: self.canonical_name,
            });
        }

        if self.formula_family() == HouseFormulaFamily::Unknown {
            return Err(
                HouseCatalogValidationError::DescriptorFormulaFamilyUnknown {
                    label: self.canonical_name,
                },
            );
        }

        let mut seen_labels = BTreeSet::new();
        let mut saw_canonical_case_variant = false;
        for alias in self.aliases {
            if alias.eq_ignore_ascii_case(self.canonical_name) {
                if alias == &self.canonical_name || saw_canonical_case_variant {
                    return Err(HouseCatalogValidationError::DescriptorLabelCollision {
                        label: alias,
                        canonical_name: self.canonical_name,
                    });
                }
                saw_canonical_case_variant = true;
                continue;
            }

            if !seen_labels.insert(alias.to_ascii_lowercase()) {
                return Err(HouseCatalogValidationError::DescriptorLabelCollision {
                    label: alias,
                    canonical_name: self.canonical_name,
                });
            }
        }

        Ok(())
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

    /// Returns the coarse formula family used by this built-in system.
    pub fn formula_family(&self) -> HouseFormulaFamily {
        match self.system {
            HouseSystem::Equal
            | HouseSystem::EqualMidheaven
            | HouseSystem::EqualAries
            | HouseSystem::Vehlow => HouseFormulaFamily::Equal,
            HouseSystem::WholeSign => HouseFormulaFamily::WholeSign,
            HouseSystem::Placidus
            | HouseSystem::Koch
            | HouseSystem::Porphyry
            | HouseSystem::Sripati
            | HouseSystem::Alcabitius
            | HouseSystem::Topocentric => HouseFormulaFamily::Quadrant,
            HouseSystem::Regiomontanus
            | HouseSystem::Campanus
            | HouseSystem::Carter
            | HouseSystem::Meridian
            | HouseSystem::Axial
            | HouseSystem::Morinus => HouseFormulaFamily::EquatorialProjection,
            HouseSystem::Horizon | HouseSystem::Apc | HouseSystem::KrusinskiPisaGoelzer => {
                HouseFormulaFamily::GreatCircle
            }
            HouseSystem::Sunshine => HouseFormulaFamily::SolarArc,
            HouseSystem::Albategnius
            | HouseSystem::PullenSd
            | HouseSystem::PullenSr
            | HouseSystem::Gauquelin => HouseFormulaFamily::Sector,
            HouseSystem::Custom(_) => HouseFormulaFamily::Custom,
            _ => HouseFormulaFamily::Unknown,
        }
    }

    /// Returns a compact one-line rendering of the descriptor.
    pub fn summary_line(&self) -> String {
        let mut text = String::from(self.canonical_name);
        if !self.aliases.is_empty() {
            text.push_str(" (aliases: ");
            text.push_str(&self.aliases.join(", "));
            text.push(')');
        }
        text.push_str(" [formula: ");
        text.push_str(&self.formula_family().to_string());
        text.push(']');
        if self.latitude_sensitive {
            text.push_str(" [latitude-sensitive]");
        }
        text.push_str(" — ");
        text.push_str(self.notes);
        text
    }

    /// Returns the compact failure-mode note for the descriptor.
    pub fn failure_mode_summary_line(&self) -> String {
        format!("{}: {}", self.canonical_name, self.notes)
    }

    /// Returns the descriptor summary after validating the entry first.
    pub fn validated_summary_line(&self) -> Result<String, HouseCatalogValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for HouseSystemDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// A Swiss-Ephemeris-style short-form label accepted by the house resolver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HouseSystemCodeAlias {
    /// The label accepted by the resolver.
    pub label: &'static str,
    /// The typed house system resolved by the label.
    pub system: HouseSystem,
}

impl HouseSystemCodeAlias {
    /// Returns a compact one-line rendering of the alias mapping.
    pub fn summary_line(&self) -> String {
        format!("{} -> {}", self.label, self.system)
    }

    /// Returns the alias mapping after validating the entry first.
    pub fn validated_summary_line(&self) -> Result<String, HouseSystemCodeAliasValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Validates the alias normalization and round-trip behavior for one entry.
    pub fn validate(&self) -> Result<(), HouseSystemCodeAliasValidationError> {
        if self.label.trim().is_empty() || has_surrounding_whitespace(self.label) {
            return Err(HouseSystemCodeAliasValidationError::LabelNotNormalized {
                label: self.label,
            });
        }
        if resolve_house_system(self.label) != Some(self.system.clone()) {
            return Err(HouseSystemCodeAliasValidationError::LabelDoesNotRoundTrip {
                label: self.label,
                expected_system: self.system.clone(),
            });
        }

        Ok(())
    }
}

impl fmt::Display for HouseSystemCodeAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const SWISS_EPHEMERIS_HOUSE_SYSTEM_CODE_ALIASES: &[HouseSystemCodeAlias] = &[
    HouseSystemCodeAlias {
        label: "P",
        system: HouseSystem::Placidus,
    },
    HouseSystemCodeAlias {
        label: "K",
        system: HouseSystem::Koch,
    },
    HouseSystemCodeAlias {
        label: "R",
        system: HouseSystem::Regiomontanus,
    },
    HouseSystemCodeAlias {
        label: "C",
        system: HouseSystem::Campanus,
    },
    HouseSystemCodeAlias {
        label: "O",
        system: HouseSystem::Porphyry,
    },
    HouseSystemCodeAlias {
        label: "D",
        system: HouseSystem::EqualMidheaven,
    },
    HouseSystemCodeAlias {
        label: "E",
        system: HouseSystem::Equal,
    },
    HouseSystemCodeAlias {
        label: "W",
        system: HouseSystem::WholeSign,
    },
    HouseSystemCodeAlias {
        label: "V",
        system: HouseSystem::Vehlow,
    },
    HouseSystemCodeAlias {
        label: "A",
        system: HouseSystem::Axial,
    },
    HouseSystemCodeAlias {
        label: "H",
        system: HouseSystem::Horizon,
    },
    HouseSystemCodeAlias {
        label: "B",
        system: HouseSystem::Alcabitius,
    },
    HouseSystemCodeAlias {
        label: "M",
        system: HouseSystem::Morinus,
    },
    HouseSystemCodeAlias {
        label: "S",
        system: HouseSystem::Sripati,
    },
    HouseSystemCodeAlias {
        label: "I",
        system: HouseSystem::Sunshine,
    },
    HouseSystemCodeAlias {
        label: "G",
        system: HouseSystem::Gauquelin,
    },
    HouseSystemCodeAlias {
        label: "T",
        system: HouseSystem::Topocentric,
    },
    HouseSystemCodeAlias {
        label: "U",
        system: HouseSystem::KrusinskiPisaGoelzer,
    },
    HouseSystemCodeAlias {
        label: "Axial Rotation",
        system: HouseSystem::Meridian,
    },
    HouseSystemCodeAlias {
        label: "Axial rotation system",
        system: HouseSystem::Meridian,
    },
    HouseSystemCodeAlias {
        label: "X",
        system: HouseSystem::Meridian,
    },
    HouseSystemCodeAlias {
        label: "Y",
        system: HouseSystem::Apc,
    },
];

/// Returns the Swiss-Ephemeris-style short-form house labels accepted by the resolver.
pub const fn house_system_code_aliases() -> &'static [HouseSystemCodeAlias] {
    SWISS_EPHEMERIS_HOUSE_SYSTEM_CODE_ALIASES
}

/// Returns a compact one-line rendering of the Swiss-Ephemeris house-code alias table.
///
/// # Examples
///
/// ```
/// use pleiades_houses::house_system_code_aliases_summary_line;
///
/// let summary = house_system_code_aliases_summary_line();
/// assert!(summary.contains("P -> Placidus"));
/// ```
pub fn house_system_code_aliases_summary_line() -> String {
    house_system_code_aliases()
        .iter()
        .map(HouseSystemCodeAlias::summary_line)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the alias table summary after validating the built-in alias inventory.
pub fn validated_house_system_code_aliases_summary_line(
) -> Result<String, HouseSystemCodeAliasValidationError> {
    validate_house_system_code_aliases()?;
    Ok(house_system_code_aliases_summary_line())
}

/// Errors emitted when validating the Swiss-Ephemeris-style house-code alias table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HouseSystemCodeAliasValidationError {
    /// The alias table unexpectedly contains no entries.
    EmptyAliasTable,
    /// A short label is blank or whitespace-padded.
    LabelNotNormalized {
        /// Label that drifted.
        label: &'static str,
    },
    /// Two short labels resolve to the same case-insensitive spelling.
    DuplicateLabel {
        /// Label that collided.
        label: &'static str,
    },
    /// A short label does not round-trip to the expected house system.
    LabelDoesNotRoundTrip {
        /// Label that failed to resolve.
        label: &'static str,
        /// Typed system that the label was expected to resolve to.
        expected_system: HouseSystem,
    },
}

impl fmt::Display for HouseSystemCodeAliasValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAliasTable => f.write_str("the house-code alias table is empty"),
            Self::LabelNotNormalized { label } => write!(
                f,
                "the house-code alias label `{label}` is blank, contains surrounding whitespace, or contains line breaks"
            ),
            Self::DuplicateLabel { label } => write!(
                f,
                "the house-code alias table contains duplicate label `{label}`"
            ),
            Self::LabelDoesNotRoundTrip {
                label,
                expected_system,
            } => write!(
                f,
                "the house-code alias label `{label}` does not round-trip to {expected_system}"
            ),
        }
    }
}

impl std::error::Error for HouseSystemCodeAliasValidationError {}

fn validate_house_system_code_alias_entries(
    entries: &[HouseSystemCodeAlias],
) -> Result<usize, HouseSystemCodeAliasValidationError> {
    if entries.is_empty() {
        return Err(HouseSystemCodeAliasValidationError::EmptyAliasTable);
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for alias in entries {
        labels_checked += 1;

        alias.validate()?;
        if !seen_labels.insert(alias.label.to_ascii_lowercase()) {
            return Err(HouseSystemCodeAliasValidationError::DuplicateLabel { label: alias.label });
        }
    }

    Ok(labels_checked)
}

/// Validates the built-in Swiss-Ephemeris-style house-code alias table.
pub fn validate_house_system_code_aliases() -> Result<(), HouseSystemCodeAliasValidationError> {
    validate_house_system_code_alias_entries(house_system_code_aliases()).map(|_| ())
}

/// Errors emitted when validating the built-in house-system catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HouseCatalogValidationError {
    /// The built-in house catalog unexpectedly contains no entries.
    EmptyCatalog,
    /// Two catalog labels resolve to the same case-insensitive spelling.
    DuplicateLabel {
        /// Label that collided.
        label: &'static str,
    },
    /// A canonical name or alias does not round-trip to its typed house system.
    LabelDoesNotRoundTrip {
        /// Label that failed to resolve.
        label: &'static str,
        /// Typed system that the label was expected to resolve to.
        expected_system: HouseSystem,
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
    /// A built-in descriptor mapped to an unknown formula family.
    DescriptorFormulaFamilyUnknown {
        /// Label whose formula family drifted.
        label: &'static str,
    },
}

impl fmt::Display for HouseCatalogValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCatalog => f.write_str("the house catalog is empty"),
            Self::DuplicateLabel { label } => {
                write!(f, "the house catalog contains duplicate label `{label}`")
            }
            Self::LabelDoesNotRoundTrip {
                label,
                expected_system,
            } => write!(
                f,
                "the house catalog label `{label}` does not round-trip to {expected_system}"
            ),
            Self::DescriptorLabelCollision {
                label,
                canonical_name,
            } => write!(
                f,
                "the house catalog descriptor label `{label}` collides with another label on `{canonical_name}`"
            ),
            Self::DescriptorLabelNotNormalized { label, field } => write!(
                f,
                "the house catalog descriptor {field} for `{label}` is blank, contains surrounding whitespace, or contains line breaks"
            ),
            Self::DescriptorNotesNotNormalized { label } => write!(
                f,
                "the house catalog descriptor note for `{label}` is blank, contains surrounding whitespace, or contains line breaks"
            ),
            Self::DescriptorFormulaFamilyUnknown { label } => write!(
                f,
                "the house catalog descriptor for `{label}` resolves to an unknown formula family"
            ),
        }
    }
}

impl std::error::Error for HouseCatalogValidationError {}

fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
}

fn contains_line_break(value: &str) -> bool {
    value.chars().any(|ch| matches!(ch, '\n' | '\r'))
}

/// A compact validation summary for the built-in house-system catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseCatalogValidationSummary {
    /// Total number of built-in house-system entries.
    pub entry_count: usize,
    /// Number of baseline entries.
    pub baseline_entry_count: usize,
    /// Number of release-specific entries.
    pub release_entry_count: usize,
    /// Number of canonical labels plus aliases checked.
    pub label_count: usize,
    /// Result of validating the built-in house-system catalog.
    pub validation_result: Result<(), HouseCatalogValidationError>,
}

impl HouseCatalogValidationSummary {
    /// Returns the compact release-facing summary line for the house catalog validation state.
    pub fn summary_line(&self) -> String {
        let formula_families = house_formula_families_summary_line();
        let latitude_sensitive_labels = built_in_house_systems()
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>();
        let latitude_sensitive_count = latitude_sensitive_labels.len();
        let latitude_sensitive_labels = if latitude_sensitive_labels.is_empty() {
            "none".to_string()
        } else {
            latitude_sensitive_labels.join(", ")
        };

        let failure_modes = latitude_sensitive_house_failure_modes_summary_line();

        match &self.validation_result {
            Ok(()) => format!(
                "house catalog validation: ok ({} entries, {} labels checked; baseline={}, release={}; formula families: {}; latitude-sensitive={}/{} entries; failure modes: {}; labels: {}; round-trip, alias uniqueness, and notes verified)",
                self.entry_count,
                self.label_count,
                self.baseline_entry_count,
                self.release_entry_count,
                formula_families,
                latitude_sensitive_count,
                self.entry_count,
                failure_modes,
                latitude_sensitive_labels,
            ),
            Err(error) => format!(
                "house catalog validation: error: {} ({} entries; baseline={}, release={})",
                error,
                self.entry_count,
                self.baseline_entry_count,
                self.release_entry_count,
            ),
        }
    }
}

impl fmt::Display for HouseCatalogValidationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_house_catalog_entries(
    entries: &[HouseSystemDescriptor],
) -> Result<usize, HouseCatalogValidationError> {
    if entries.is_empty() {
        return Err(HouseCatalogValidationError::EmptyCatalog);
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for entry in entries {
        labels_checked += 1;
        entry.validate()?;

        if resolve_house_system(entry.canonical_name) != Some(entry.system.clone()) {
            return Err(HouseCatalogValidationError::LabelDoesNotRoundTrip {
                label: entry.canonical_name,
                expected_system: entry.system.clone(),
            });
        }
        if !seen_labels.insert(entry.canonical_name.to_ascii_lowercase()) {
            return Err(HouseCatalogValidationError::DuplicateLabel {
                label: entry.canonical_name,
            });
        }

        for alias in entry.aliases {
            labels_checked += 1;
            if resolve_house_system(alias) != Some(entry.system.clone()) {
                return Err(HouseCatalogValidationError::LabelDoesNotRoundTrip {
                    label: alias,
                    expected_system: entry.system.clone(),
                });
            }

            if alias.eq_ignore_ascii_case(entry.canonical_name) {
                if alias != &entry.canonical_name {
                    continue;
                }

                return Err(HouseCatalogValidationError::DuplicateLabel { label: alias });
            }

            if !seen_labels.insert(alias.to_ascii_lowercase()) {
                return Err(HouseCatalogValidationError::DuplicateLabel { label: alias });
            }
        }
    }

    Ok(labels_checked)
}

/// Validates the built-in house-system catalog for label uniqueness and round-trips.
pub fn validate_house_catalog() -> Result<(), HouseCatalogValidationError> {
    validate_house_catalog_entries(built_in_house_systems()).map(|_| ())
}

fn collect_house_formula_families(entries: &[HouseSystemDescriptor]) -> Vec<HouseFormulaFamily> {
    let mut families = Vec::new();

    for entry in entries {
        let family = entry.formula_family();
        if family == HouseFormulaFamily::Custom || family == HouseFormulaFamily::Unknown {
            continue;
        }
        if !families.contains(&family) {
            families.push(family);
        }
    }

    families.sort_by_key(house_formula_family_sort_key);
    families
}

fn house_formula_family_sort_key(family: &HouseFormulaFamily) -> u8 {
    match family {
        HouseFormulaFamily::Equal => 0,
        HouseFormulaFamily::WholeSign => 1,
        HouseFormulaFamily::Quadrant => 2,
        HouseFormulaFamily::EquatorialProjection => 3,
        HouseFormulaFamily::GreatCircle => 4,
        HouseFormulaFamily::SolarArc => 5,
        HouseFormulaFamily::Sector => 6,
        HouseFormulaFamily::Custom => 7,
        HouseFormulaFamily::Unknown => 8,
    }
}

/// Returns the distinct formula families represented by the built-in house catalog.
pub fn house_formula_families() -> Vec<HouseFormulaFamily> {
    collect_house_formula_families(built_in_house_systems())
}

/// Returns a compact one-line rendering of the distinct built-in house formula families.
pub fn house_formula_families_summary_line() -> String {
    let families = house_formula_families();

    match families.as_slice() {
        [] => "none".to_string(),
        [single] => single.to_string(),
        _ => families
            .iter()
            .map(HouseFormulaFamily::to_string)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn format_string_summary(items: &[String]) -> String {
    match items {
        [] => "none".to_string(),
        [single] => single.clone(),
        _ => items.join(", "),
    }
}

/// Returns the release-facing failure-mode notes for the latitude-sensitive built-in house systems.
pub fn latitude_sensitive_house_failure_modes() -> Vec<String> {
    built_in_house_systems()
        .iter()
        .filter(|entry| entry.latitude_sensitive)
        .map(HouseSystemDescriptor::failure_mode_summary_line)
        .collect()
}

/// Returns a compact one-line rendering of the latitude-sensitive failure-mode notes.
pub fn latitude_sensitive_house_failure_modes_summary_line() -> String {
    format_string_summary(&latitude_sensitive_house_failure_modes())
}

/// Returns a compact validation summary for the built-in house-system catalog.
pub fn house_catalog_validation_summary() -> HouseCatalogValidationSummary {
    let entry_count = built_in_house_systems().len();
    let baseline_entry_count = baseline_house_systems().len();
    let release_entry_count = release_house_systems().len();
    let (label_count, validation_result) =
        match validate_house_catalog_entries(built_in_house_systems()) {
            Ok(label_count) => (label_count, Ok(())),
            Err(error) => (0, Err(error)),
        };

    HouseCatalogValidationSummary {
        entry_count,
        baseline_entry_count,
        release_entry_count,
        label_count,
        validation_result,
    }
}

const BASELINE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Equal,
        "Equal",
        &[
            "A equal",
            "E equal = A",
            "Equal houses",
            "Equal house system",
            "Equal House",
            "Equal table of houses",
            "Wang",
            "Equal (cusp 1 = Asc)",
        ],
        "Equal-house system anchored on the ascendant; Wang and the Swiss Ephemeris \"Equal (cusp 1 = Asc)\" label are treated as interoperability aliases for the equal-house-from-Ascendant convention.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::WholeSign,
        "Whole Sign",
        &[
            "W equal, whole sign",
            "Whole Sign houses",
            "Whole Sign table of houses",
            "Whole-sign",
            "Whole Sign system",
            "Whole Sign house system",
        ],
        "Whole-sign system anchored on the rising sign.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Meridian,
        "Meridian",
        &[
            "Meridian houses",
            "Meridian table of houses",
            "Meridian house system",
            "ARMC",
            "Axial Rotation",
            "Axial rotation system",
            "Zariel",
            "X axial rotation system/ Meridian houses",
        ],
        "Meridian-style systems and documented axial variants.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Topocentric,
        "Topocentric",
        &[
            "Polich-Page",
            "Polich/Page",
            "Polich Page",
            "Polich-Page \"topocentric\" table of houses",
            "T Polich/Page (\"topocentric\")",
            "T topocentric",
            "Topocentric house system",
            "Topocentric table of houses",
        ],
        "Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
        None,
    ),
];

const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &[
            "D equal / MC",
            "Equal from MC",
            "Equal (from MC)",
            "Equal (from MC) table of houses",
            "Equal (MC) table of houses",
            "Equal/MC table of houses",
            "Equal (MC) house system",
            "Equal/MC house system",
            "Equal MC",
            "Equal/MC",
            "Equal Midheaven",
            "Equal Midheaven house system",
            "Equal Midheaven table of houses",
            "Equal/MC = 10th",
        ],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &[
            "N",
            "Equal/1=Aries",
            "Equal Aries",
            "Aries houses",
            "Whole Sign (house 1 = Aries)",
            "Whole Sign (house 1 = Aries) table of houses",
            "Equal (1=Aries) table of houses",
            "Equal/1=Aries table of houses",
            "Equal (1=Aries) house system",
            "Equal/1=Aries house system",
            "N whole sign houses, 1. house = Aries",
            "Whole sign houses, 1. house = Aries",
            "Equal/1=0 Aries",
            "Equal (cusp 1 = 0° Aries)",
        ],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &[
            "V equal Vehlow",
            "Vehlow",
            "Vehlow equal",
            "Vehlow house system",
            "Vehlow Equal house system",
            "Vehlow-equal",
            "Vehlow-equal table of houses",
            "Vehlow Equal table of houses",
        ],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &[
            "Carter",
            "Carter's poli-equatorial",
            "Carter's poli-equatorial table of houses",
            "Poli-Equatorial",
        ],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
        None,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &[
            "Horizon",
            "Azimuth",
            "Horizontal",
            "Azimuthal",
            "Horizon table of houses",
            "Horizontal table of houses",
            "Azimuthal table of houses",
            "Horizon house system",
            "Horizon/Azimuth house system",
            "Horizontal house system",
            "Azimuth house system",
            "Horizon/Azimuth table of houses",
            "Azimuthal house system",
            "horizon/azimut",
        ],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as \u{201C}Ram school\u{201D}, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::KrusinskiPisaGoelzer,
        "Krusinski-Pisa-Goelzer",
        &[
            "Krusinski",
            "Krusinski-Pisa",
            "Krusinski Pisa",
            "Krusinski/Pisa/Goelzer",
            "Krusinski-Pisa-Goelzer table of houses",
            "U krusinski-pisa-goelzer",
            "Krusinski/Pisa/Goelzer house system",
            "Pisa-Goelzer",
        ],
        "Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::PullenSd,
        "Pullen SD",
        &[
            "Pullen SD table of houses",
            "Pullen SD (Neo-Porphyry) table of houses",
            "Pullen SD (Neo-Porphyry)",
            "Neo-Porphyry",
            "Pullen (Sinusoidal Delta)",
            "Pullen SD (Sinusoidal Delta)",
            "Pullen sinusoidal delta",
        ],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::PullenSr,
        "Pullen SR",
        &[
            "Pullen SR table of houses",
            "Pullen SR (Sinusoidal Ratio) table of houses",
            "Pullen SR (Sinusoidal Ratio)",
            "Pullen (Sinusoidal Ratio)",
            "Pullen sinusoidal ratio",
        ],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Sunshine,
        "Sunshine",
        &[
            "I sunshine",
            "Sunshine houses",
            "Sunshine house system",
            "Sunshine table of houses",
            "Sunshine table of houses, by Bob Makransky",
            "Makransky Sunshine",
            "Bob Makransky",
            "Treindl Sunshine",
        ],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
        Some(66.0),
    ),
];

static BUILT_IN_HOUSE_SYSTEMS: [HouseSystemDescriptor; 25] = [
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Carter,
        "Carter (poli-equatorial)",
        &[
            "Carter",
            "Carter's poli-equatorial",
            "Carter's poli-equatorial table of houses",
            "Poli-Equatorial",
        ],
        "Equal right-ascension segments anchored on the Ascendant's meridian.",
        false,
        None,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Horizon,
        "Horizon/Azimuth",
        &[
            "Horizon",
            "Azimuth",
            "Horizontal",
            "Azimuthal",
            "Horizon table of houses",
            "Horizontal table of houses",
            "Azimuthal table of houses",
            "Horizon house system",
            "Horizon/Azimuth house system",
            "Horizontal house system",
            "Azimuth house system",
            "Horizon/Azimuth table of houses",
            "Azimuthal house system",
            "horizon/azimut",
        ],
        "Azimuthal house system that anchors house 1 due East and house 10 at the MC.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as \u{201C}Ram school\u{201D}, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::KrusinskiPisaGoelzer,
        "Krusinski-Pisa-Goelzer",
        &[
            "Krusinski",
            "Krusinski-Pisa",
            "Krusinski Pisa",
            "Krusinski/Pisa/Goelzer",
            "Krusinski-Pisa-Goelzer table of houses",
            "U krusinski-pisa-goelzer",
            "Krusinski/Pisa/Goelzer house system",
            "Pisa-Goelzer",
        ],
        "Great-circle house system centered on the ascendant and zenith; latitude-sensitive near the poles.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::PullenSd,
        "Pullen SD",
        &[
            "Pullen SD table of houses",
            "Pullen SD (Neo-Porphyry) table of houses",
            "Pullen SD (Neo-Porphyry)",
            "Neo-Porphyry",
            "Pullen (Sinusoidal Delta)",
            "Pullen SD (Sinusoidal Delta)",
            "Pullen sinusoidal delta",
        ],
        "Sinusoidal-delta variant that smooths quadrant spacing toward the angles.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::PullenSr,
        "Pullen SR",
        &[
            "Pullen SR table of houses",
            "Pullen SR (Sinusoidal Ratio) table of houses",
            "Pullen SR (Sinusoidal Ratio)",
            "Pullen (Sinusoidal Ratio)",
            "Pullen sinusoidal ratio",
        ],
        "Sinusoidal-ratio variant with ratio-derived house spacing.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Equal,
        "Equal",
        &[
            "A equal",
            "E equal = A",
            "Equal houses",
            "Equal house system",
            "Equal House",
            "Equal table of houses",
            "Wang",
            "Equal (cusp 1 = Asc)",
        ],
        "Equal-house system anchored on the ascendant; Wang and the Swiss Ephemeris \"Equal (cusp 1 = Asc)\" label are treated as interoperability aliases for the equal-house-from-Ascendant convention.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::WholeSign,
        "Whole Sign",
        &[
            "W equal, whole sign",
            "Whole Sign houses",
            "Whole Sign table of houses",
            "Whole-sign",
            "Whole Sign system",
            "Whole Sign house system",
        ],
        "Whole-sign system anchored on the rising sign.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Meridian,
        "Meridian",
        &[
            "Meridian houses",
            "Meridian table of houses",
            "Meridian house system",
            "ARMC",
            "Axial Rotation",
            "Axial rotation system",
            "Zariel",
            "X axial rotation system/ Meridian houses",
        ],
        "Meridian-style systems and documented axial variants.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Topocentric,
        "Topocentric",
        &[
            "Polich-Page",
            "Polich/Page",
            "Polich Page",
            "Polich-Page \"topocentric\" table of houses",
            "T Polich/Page (\"topocentric\")",
            "T topocentric",
            "Topocentric house system",
            "Topocentric table of houses",
        ],
        "Topocentric (Polich-Page) house system with geodetic-to-geocentric latitude correction.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Sunshine,
        "Sunshine",
        &[
            "I sunshine",
            "Sunshine houses",
            "Sunshine house system",
            "Sunshine table of houses",
            "Sunshine table of houses, by Bob Makransky",
            "Makransky Sunshine",
            "Bob Makransky",
            "Treindl Sunshine",
        ],
        "Sunshine house system based on the Sun's diurnal and nocturnal arcs; the 1st house is the Ascendant and the 10th house is the MC.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
        Some(66.0),
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::EqualMidheaven,
        "Equal (MC)",
        &[
            "D equal / MC",
            "Equal from MC",
            "Equal (from MC)",
            "Equal (from MC) table of houses",
            "Equal (MC) table of houses",
            "Equal/MC table of houses",
            "Equal (MC) house system",
            "Equal/MC house system",
            "Equal MC",
            "Equal/MC",
            "Equal Midheaven",
            "Equal Midheaven house system",
            "Equal Midheaven table of houses",
            "Equal/MC = 10th",
        ],
        "Equal houses anchored at the Midheaven instead of the Ascendant.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::EqualAries,
        "Equal (1=Aries)",
        &[
            "N",
            "Equal/1=Aries",
            "Equal Aries",
            "Aries houses",
            "Whole Sign (house 1 = Aries)",
            "Whole Sign (house 1 = Aries) table of houses",
            "Equal (1=Aries) table of houses",
            "Equal/1=Aries table of houses",
            "Equal (1=Aries) house system",
            "Equal/1=Aries house system",
            "N whole sign houses, 1. house = Aries",
            "Whole sign houses, 1. house = Aries",
            "Equal/1=0 Aries",
            "Equal (cusp 1 = 0° Aries)",
        ],
        "Fixed zodiac-sign houses anchored at 0° Aries.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Vehlow,
        "Vehlow Equal",
        &[
            "V equal Vehlow",
            "Vehlow",
            "Vehlow equal",
            "Vehlow house system",
            "Vehlow Equal house system",
            "Vehlow-equal",
            "Vehlow-equal table of houses",
            "Vehlow Equal table of houses",
        ],
        "Equal-house variant with the Ascendant centered in house 1.",
        false,
        None,
    ),
    HouseSystemDescriptor::new_release_grade(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
        None,
    ),
];

/// Returns the baseline built-in house-system catalog.
pub const fn baseline_house_systems() -> &'static [HouseSystemDescriptor] {
    BASELINE_HOUSE_SYSTEMS
}

/// Returns the release-specific house-system additions beyond the baseline milestone.
pub const fn release_house_systems() -> &'static [HouseSystemDescriptor] {
    RELEASE_HOUSE_SYSTEMS
}

/// Returns the full built-in house-system catalog shipped by this release line.
pub const fn built_in_house_systems() -> &'static [HouseSystemDescriptor] {
    &BUILT_IN_HOUSE_SYSTEMS
}

/// Finds the descriptor for a typed house-system selection.
pub fn descriptor(system: &HouseSystem) -> Option<&'static HouseSystemDescriptor> {
    built_in_house_systems()
        .iter()
        .find(|entry| entry.system == *system)
}

fn resolve_house_system_code(label: &str) -> Option<HouseSystem> {
    let normalized = label.trim();
    house_system_code_aliases()
        .iter()
        .find(|entry| entry.label.eq_ignore_ascii_case(normalized))
        .map(|entry| entry.system.clone())
}

/// Resolves a house-system label to a built-in type.
pub fn resolve_house_system(label: &str) -> Option<HouseSystem> {
    resolve_house_system_code(label).or_else(|| {
        built_in_house_systems()
            .iter()
            .find(|entry| entry.matches_label(label))
            .map(|entry| entry.system.clone())
    })
}

#[cfg(test)]
mod tests;
