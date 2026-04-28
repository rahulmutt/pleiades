//! House-system catalog definitions and compatibility metadata.
//!
//! This crate focuses on the catalog layer and the baseline chart house-
//! placement helpers: it enumerates the built-in house systems, their common
//! aliases, formula-family tags, latitude-sensitive notes, and the Stage 3
//! baseline house formulas that power the chart workflow. It also carries the first Stage 6
//! compatibility-expansion additions so release profiles can distinguish the
//! baseline milestone from newer catalog breadth. The resolver additionally
//! accepts the common Swiss Ephemeris house-system letter codes used by
//! interoperability tables.
//!
//! # Examples
//!
//! ```
//! use pleiades_houses::{baseline_house_systems, resolve_house_system};
//!
//! let systems = baseline_house_systems();
//! assert!(systems.iter().any(|entry| entry.canonical_name == "Placidus"));
//!
//! assert_eq!(resolve_house_system("Polich-Page"), Some(pleiades_types::HouseSystem::Topocentric));
//! ```
//!
//! ```
//! use pleiades_houses::{calculate_houses, HouseRequest};
//! use pleiades_types::{HouseSystem, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};
//!
//! let request = HouseRequest::new(
//!     Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
//!     ObserverLocation::new(Latitude::from_degrees(0.0), Longitude::from_degrees(0.0), None),
//!     HouseSystem::WholeSign,
//! );
//! let houses = calculate_houses(&request).expect("house calculation should work");
//! assert_eq!(houses.cusps.len(), 12);
//! ```

#![forbid(unsafe_code)]

mod houses;

pub use houses::{
    calculate_houses, house_for_longitude, HouseAngles, HouseError, HouseErrorKind, HouseRequest,
    HouseSnapshot,
};

use core::fmt;
use std::collections::BTreeSet;

use pleiades_types::HouseSystem;

/// A catalog entry for a built-in house system.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Creates a new descriptor.
    pub const fn new(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
        }
    }

    /// Validates the descriptor-local metadata invariants.
    pub fn validate(&self) -> Result<(), HouseCatalogValidationError> {
        if self.canonical_name.trim().is_empty() || has_surrounding_whitespace(self.canonical_name)
        {
            return Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                label: self.canonical_name,
                field: "canonical name",
            });
        }

        for alias in self.aliases {
            if alias.trim().is_empty() || has_surrounding_whitespace(alias) {
                return Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                    label: alias,
                    field: "alias",
                });
            }
        }

        if self.notes.trim().is_empty()
            || (!self.notes.is_empty() && self.notes.trim() != self.notes)
        {
            return Err(HouseCatalogValidationError::DescriptorNotesNotNormalized {
                label: self.canonical_name,
            });
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
                "the house catalog descriptor {field} for `{label}` is blank or contains surrounding whitespace"
            ),
            Self::DescriptorNotesNotNormalized { label } => write!(
                f,
                "the house catalog descriptor note for `{label}` is blank or contains surrounding whitespace"
            ),
        }
    }
}

impl std::error::Error for HouseCatalogValidationError {}

fn has_surrounding_whitespace(value: &str) -> bool {
    !value.is_empty() && value.trim() != value
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

        match &self.validation_result {
            Ok(()) => format!(
                "house catalog validation: ok ({} entries, {} labels checked; baseline={}, release={}; latitude-sensitive={}/{} entries; labels: {}; round-trip, alias uniqueness, and notes verified)",
                self.entry_count,
                self.label_count,
                self.baseline_entry_count,
                self.release_entry_count,
                latitude_sensitive_count,
                self.entry_count,
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
    HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
];

const RELEASE_HOUSE_SYSTEMS: &[HouseSystemDescriptor] = &[
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as “Ram school”, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
    ),
];

static BUILT_IN_HOUSE_SYSTEMS: [HouseSystemDescriptor; 25] = [
    HouseSystemDescriptor::new(
        HouseSystem::Placidus,
        "Placidus",
        &["Placidus house system", "Placidus table of houses"],
        "Quadrant system; can fail or become unstable at extreme latitudes.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Koch,
        "Koch",
        &["Koch houses", "Koch house system", "house system of the birth place", "Koch table of houses", "W. Koch", "W Koch"],
        "Quadrant system with documented high-latitude pathologies.",
        true,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Porphyry,
        "Porphyry",
        &[
            "Equal Quadrant",
            "Porphyry house system",
            "Porphyry table of houses",
        ],
        "Simple quadrant division used as a robust fallback.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Regiomontanus,
        "Regiomontanus",
        &[
            "Regiomontanus houses",
            "Regiomontanus house system",
            "Regiomontanus table of houses",
        ],
        "Classical quadrant system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Campanus,
        "Campanus",
        &[
            "Campanus houses",
            "Campanus house system",
            "Campanus table of houses",
        ],
        "Great-circle division system.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Apc,
        "APC",
        &[
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as “Ram school”, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "APC (Ram school) houses with non-opposite quadrant pairs and polar adjustments.",
        true,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Albategnius,
        "Albategnius",
        &["Savard-A", "Savard A", "Savard's Albategnius"],
        "Quartered latitude-circle variant associated with Savard's Albategnius proposal.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Alcabitius,
        "Alcabitius",
        &[
            "Alcabitius houses",
            "Alcabitius house system",
            "Alcabitius table of houses",
        ],
        "Classical semi-arc family system.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Axial,
        "Axial",
        &["Axial variants", "A"],
        "Documented axial variants used by some astrology packages.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Morinus,
        "Morinus",
        &["Morinus houses", "Morinus house system"],
        "Morinus house system with historical interoperability value.",
        false,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Gauquelin,
        "Gauquelin sectors",
        &["G", "Gauquelin", "Gauquelin sector", "Gauquelin table of sectors"],
        "Thirty-six sectors used by the Gauquelin-sector family.",
        true,
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
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
    ),
    HouseSystemDescriptor::new(
        HouseSystem::Sripati,
        "Sripati",
        &["S sripati", "Śrīpati", "Sripati house system", "Sripati table of houses"],
        "Midpoint variant of the Porphyry quadrants used in Jyotiṣa.",
        false,
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
mod tests {
    use super::*;

    #[test]
    fn baseline_catalog_includes_required_milestone_entries() {
        let names: Vec<_> = baseline_house_systems()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Placidus",
            "Koch",
            "Porphyry",
            "Regiomontanus",
            "Campanus",
            "Equal",
            "Whole Sign",
            "Alcabitius",
            "Meridian",
            "Axial",
            "Topocentric",
            "Morinus",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn descriptor_summary_line_includes_aliases_formula_family_latitude_and_notes() {
        let descriptor = HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Equal",
            &["Alias One", "Alias Two"],
            "Summary note",
            true,
        );

        let expected =
            "Equal (aliases: Alias One, Alias Two) [formula: Equal] [latitude-sensitive] — Summary note";
        assert_eq!(descriptor.summary_line(), expected);
        assert_eq!(descriptor.to_string(), expected);
    }

    #[test]
    fn formula_family_groups_the_built_in_house_systems_by_shape() {
        let equal = descriptor(&HouseSystem::Equal).expect("equal should be catalogued");
        let whole_sign =
            descriptor(&HouseSystem::WholeSign).expect("whole sign should be catalogued");
        let quadrant = descriptor(&HouseSystem::Placidus).expect("placidus should be catalogued");
        let equatorial = descriptor(&HouseSystem::Meridian).expect("meridian should be catalogued");
        let great_circle = descriptor(&HouseSystem::Horizon).expect("horizon should be catalogued");
        let solar_arc = descriptor(&HouseSystem::Sunshine).expect("sunshine should be catalogued");
        let sector = descriptor(&HouseSystem::Gauquelin).expect("gauquelin should be catalogued");

        assert_eq!(equal.formula_family(), HouseFormulaFamily::Equal);
        assert_eq!(whole_sign.formula_family(), HouseFormulaFamily::WholeSign);
        assert_eq!(quadrant.formula_family(), HouseFormulaFamily::Quadrant);
        assert_eq!(
            equatorial.formula_family(),
            HouseFormulaFamily::EquatorialProjection
        );
        assert_eq!(
            great_circle.formula_family(),
            HouseFormulaFamily::GreatCircle
        );
        assert_eq!(solar_arc.formula_family(), HouseFormulaFamily::SolarArc);
        assert_eq!(sector.formula_family(), HouseFormulaFamily::Sector);
    }

    #[test]
    fn validation_errors_use_stable_house_system_display_names() {
        let error = HouseCatalogValidationError::LabelDoesNotRoundTrip {
            label: "Equal (MC) table of houses",
            expected_system: HouseSystem::EqualMidheaven,
        };

        assert_eq!(
            error.to_string(),
            "the house catalog label `Equal (MC) table of houses` does not round-trip to Equal (MC)"
        );
    }

    #[test]
    fn aliases_resolve_to_builtin_systems() {
        assert_eq!(
            resolve_house_system("Polich-Page"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Polich/Page"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Topocentric house system"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Topocentric table of houses"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Polich-Page \"topocentric\" table of houses"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Equal table of houses"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Equal (from MC) table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC) table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC) house system"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Whole Sign table of houses"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign (house 1 = Aries) table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (1=Aries) table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=Aries table of houses"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (1=Aries) house system"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Vehlow-equal table of houses"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow Equal table of houses"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow equal"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial table of houses"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("APC, also known as “Ram school”, table of houses"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(
            resolve_house_system("Krusinski-Pisa-Goelzer table of houses"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Sunshine table of houses"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Sunshine table of houses, by Bob Makransky"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("I sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Gauquelin table of sectors"),
            Some(HouseSystem::Gauquelin)
        );
        assert_eq!(
            resolve_house_system("whole sign houses"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign system"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Whole Sign house system"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("Placidus table of houses"),
            Some(HouseSystem::Placidus)
        );
        assert_eq!(
            resolve_house_system("Koch table of houses"),
            Some(HouseSystem::Koch)
        );
        assert_eq!(resolve_house_system("w. koch"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("Koch houses"), Some(HouseSystem::Koch));
        assert_eq!(
            resolve_house_system("house system of the birth place"),
            Some(HouseSystem::Koch)
        );
        assert_eq!(resolve_house_system("W Koch"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("ARMC"), Some(HouseSystem::Meridian));
        assert_eq!(
            resolve_house_system("Axial Rotation"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(
            resolve_house_system("Axial rotation system"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("Zariel"), Some(HouseSystem::Meridian));
        assert_eq!(
            resolve_house_system("Meridian house system"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("D"), Some(HouseSystem::EqualMidheaven));
        assert_eq!(resolve_house_system("A equal"), Some(HouseSystem::Equal));
        assert_eq!(
            resolve_house_system("D equal / MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("E equal = A"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("W equal, whole sign"),
            Some(HouseSystem::WholeSign)
        );
        assert_eq!(
            resolve_house_system("V equal Vehlow"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("X axial rotation system/ Meridian houses"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("Y APC houses"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("T Polich/Page (\"topocentric\")"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(resolve_house_system("P"), Some(HouseSystem::Placidus));
        assert_eq!(resolve_house_system("K"), Some(HouseSystem::Koch));
        assert_eq!(resolve_house_system("R"), Some(HouseSystem::Regiomontanus));
        assert_eq!(resolve_house_system("C"), Some(HouseSystem::Campanus));
        assert_eq!(resolve_house_system("O"), Some(HouseSystem::Porphyry));
        assert_eq!(resolve_house_system("E"), Some(HouseSystem::Equal));
        assert_eq!(resolve_house_system("W"), Some(HouseSystem::WholeSign));
        assert_eq!(resolve_house_system("N"), Some(HouseSystem::EqualAries));
        assert_eq!(resolve_house_system("V"), Some(HouseSystem::Vehlow));
        assert_eq!(resolve_house_system("A"), Some(HouseSystem::Axial));
        assert_eq!(resolve_house_system("H"), Some(HouseSystem::Horizon));
        assert_eq!(resolve_house_system("B"), Some(HouseSystem::Alcabitius));
        assert_eq!(resolve_house_system("M"), Some(HouseSystem::Morinus));
        assert_eq!(resolve_house_system("S"), Some(HouseSystem::Sripati));
        assert_eq!(resolve_house_system("I"), Some(HouseSystem::Sunshine));
        assert_eq!(resolve_house_system("G"), Some(HouseSystem::Gauquelin));
        assert_eq!(resolve_house_system("T"), Some(HouseSystem::Topocentric));
        assert_eq!(
            resolve_house_system("U"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(resolve_house_system("X"), Some(HouseSystem::Meridian));
        assert_eq!(resolve_house_system("Y"), Some(HouseSystem::Apc));
        assert_eq!(resolve_house_system("Carter"), Some(HouseSystem::Carter));
        assert_eq!(
            resolve_house_system("Carter's poli-equatorial"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("T topocentric"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("U krusinski-pisa-goelzer"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Equal (from MC)"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC house system"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven house system"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Midheaven table of houses"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal (MC)"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal/MC = 10th"),
            Some(HouseSystem::EqualMidheaven)
        );
        assert_eq!(
            resolve_house_system("Equal Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=Aries house system"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal/1=0 Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (cusp 1 = 0° Aries)"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(resolve_house_system("vehlow"), Some(HouseSystem::Vehlow));
        assert_eq!(
            resolve_house_system("Vehlow house system"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow Equal house system"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(
            resolve_house_system("Vehlow-equal"),
            Some(HouseSystem::Vehlow)
        );
        assert_eq!(resolve_house_system("Wang"), Some(HouseSystem::Equal));
        assert_eq!(
            resolve_house_system("Equal house system"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Equal House"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(
            resolve_house_system("Whole Sign (house 1 = Aries)"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("N whole sign houses, 1. house = Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Whole sign houses, 1. house = Aries"),
            Some(HouseSystem::EqualAries)
        );
        assert_eq!(
            resolve_house_system("Equal (cusp 1 = Asc)"),
            Some(HouseSystem::Equal)
        );
        assert_eq!(resolve_house_system("Azimuth"), Some(HouseSystem::Horizon));
        assert_eq!(
            resolve_house_system("Horizontal"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuthal"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizontal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizontal table of houses"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuthal table of houses"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("horizon/azimuth"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("horizon/azimut"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(resolve_house_system("Ram school"), Some(HouseSystem::Apc));
        assert_eq!(resolve_house_system("Ram's school"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("APC house system"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(resolve_house_system("WvA"), Some(HouseSystem::Apc));
        assert_eq!(
            resolve_house_system("Ascendant Parallel Circle"),
            Some(HouseSystem::Apc)
        );
        assert_eq!(
            resolve_house_system("Krusinski"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Krusinski/Pisa/Goelzer"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Krusinski/Pisa/Goelzer house system"),
            Some(HouseSystem::KrusinskiPisaGoelzer)
        );
        assert_eq!(
            resolve_house_system("Horizon house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizon/Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizontal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuth house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Horizon/Azimuth table of houses"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Azimuthal house system"),
            Some(HouseSystem::Horizon)
        );
        assert_eq!(
            resolve_house_system("Sunshine house system"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(resolve_house_system("Śrīpati"), Some(HouseSystem::Sripati));
        assert_eq!(
            resolve_house_system("S sripati"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sripati house system"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sripati table of houses"),
            Some(HouseSystem::Sripati)
        );
        assert_eq!(
            resolve_house_system("Sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Bob Makransky"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(
            resolve_house_system("Treindl Sunshine"),
            Some(HouseSystem::Sunshine)
        );
        assert_eq!(resolve_house_system("G"), Some(HouseSystem::Gauquelin));
        assert_eq!(
            resolve_house_system("Gauquelin sectors"),
            Some(HouseSystem::Gauquelin)
        );
        assert_eq!(
            resolve_house_system("Savard-A"),
            Some(HouseSystem::Albategnius)
        );
        assert_eq!(
            resolve_house_system("Neo-Porphyry"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen (Sinusoidal Delta)"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen SD (Sinusoidal Delta)"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen SD table of houses"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen SD (Neo-Porphyry) table of houses"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen SD (Neo-Porphyry)"),
            Some(HouseSystem::PullenSd)
        );
        assert_eq!(
            resolve_house_system("Pullen (Sinusoidal Ratio)"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen sinusoidal ratio"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen SR table of houses"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen SR (Sinusoidal Ratio) table of houses"),
            Some(HouseSystem::PullenSr)
        );
        assert_eq!(
            resolve_house_system("Pullen SR (Sinusoidal Ratio)"),
            Some(HouseSystem::PullenSr)
        );
    }

    #[test]
    fn release_additions_are_merged_into_the_built_in_catalog() {
        let names: Vec<_> = built_in_house_systems()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Equal (MC)",
            "Equal (1=Aries)",
            "Vehlow Equal",
            "Sripati",
            "Carter (poli-equatorial)",
            "Horizon/Azimuth",
            "APC",
            "Krusinski-Pisa-Goelzer",
            "Albategnius",
            "Pullen SD",
            "Pullen SR",
            "Sunshine",
            "Gauquelin sectors",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn release_descriptor_aliases_do_not_repeat_canonical_labels() {
        assert!(built_in_house_systems()
            .iter()
            .all(|entry| { !entry.aliases.contains(&entry.canonical_name) }));
    }

    #[test]
    fn house_catalog_round_trips_all_built_ins_and_aliases() {
        use std::collections::HashSet;

        let built_in = built_in_house_systems();
        let mut unique_names = HashSet::new();

        assert_eq!(
            built_in.len(),
            baseline_house_systems().len() + release_house_systems().len()
        );

        for entry in baseline_house_systems()
            .iter()
            .chain(release_house_systems().iter())
        {
            assert!(
                unique_names.insert(entry.canonical_name),
                "duplicate canonical house-system name {}",
                entry.canonical_name
            );
            assert_eq!(
                descriptor(&entry.system).map(|d| d.canonical_name),
                Some(entry.canonical_name)
            );
            assert_eq!(
                resolve_house_system(entry.canonical_name),
                Some(entry.system.clone())
            );
            for alias in entry.aliases {
                assert_eq!(resolve_house_system(alias), Some(entry.system.clone()));
            }
        }

        for entry in built_in {
            assert!(unique_names.contains(entry.canonical_name));
        }
    }

    #[test]
    fn additional_release_house_aliases_resolve_to_builtin_systems() {
        assert_eq!(
            resolve_house_system("Polich Page"),
            Some(HouseSystem::Topocentric)
        );
        assert_eq!(
            resolve_house_system("Poli-Equatorial"),
            Some(HouseSystem::Carter)
        );
        assert_eq!(
            resolve_house_system("Equal Quadrant"),
            Some(HouseSystem::Porphyry)
        );
        assert_eq!(
            resolve_house_system("Meridian table of houses"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(
            resolve_house_system("Whole-sign"),
            Some(HouseSystem::WholeSign)
        );
    }

    #[test]
    fn house_catalog_validation_summary_reports_catalog_health() {
        let summary = house_catalog_validation_summary();
        let expected_latitude_sensitive_labels = built_in_house_systems()
            .iter()
            .filter(|entry| entry.latitude_sensitive)
            .map(|entry| entry.canonical_name)
            .collect::<Vec<_>>()
            .join(", ");

        assert_eq!(summary.entry_count, built_in_house_systems().len());
        assert_eq!(summary.baseline_entry_count, baseline_house_systems().len());
        assert_eq!(summary.release_entry_count, release_house_systems().len());
        assert!(summary.validation_result.is_ok());
        assert!(summary
            .summary_line()
            .contains("house catalog validation: ok"));
        assert!(summary.summary_line().contains("latitude-sensitive="));
        assert!(summary
            .summary_line()
            .contains(&expected_latitude_sensitive_labels));
        assert!(summary
            .summary_line()
            .contains("round-trip, alias uniqueness, and notes verified"));
    }

    #[test]
    fn house_catalog_validation_rejects_duplicate_labels_and_round_trip_mismatches() {
        let duplicate_alias_entries = [HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Equal",
            &["Wang", "wang"],
            "notes",
            false,
        )];

        assert!(matches!(
            validate_house_catalog_entries(&duplicate_alias_entries),
            Err(HouseCatalogValidationError::DescriptorLabelCollision {
                label: "wang",
                canonical_name: "Equal"
            })
        ));

        let mismatched_entry = [HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Not Equal",
            &[],
            "notes",
            false,
        )];

        assert!(matches!(
            validate_house_catalog_entries(&mismatched_entry),
            Err(HouseCatalogValidationError::LabelDoesNotRoundTrip {
                label: "Not Equal",
                expected_system: HouseSystem::Equal,
            })
        ));

        let blank_name_descriptor =
            HouseSystemDescriptor::new(HouseSystem::Equal, "   ", &[], "notes", false);
        assert!(matches!(
            blank_name_descriptor.validate(),
            Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                label: "   ",
                field: "canonical name"
            })
        ));

        let padded_alias_descriptor =
            HouseSystemDescriptor::new(HouseSystem::Equal, "Equal", &[" Alias "], "notes", false);
        assert!(matches!(
            padded_alias_descriptor.validate(),
            Err(HouseCatalogValidationError::DescriptorLabelNotNormalized {
                label: " Alias ",
                field: "alias"
            })
        ));

        let blank_notes_descriptor =
            HouseSystemDescriptor::new(HouseSystem::Equal, "Equal", &[], "   ", false);
        assert!(matches!(
            blank_notes_descriptor.validate(),
            Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
        ));

        let duplicate_alias_descriptor = HouseSystemDescriptor::new(
            HouseSystem::Equal,
            "Equal",
            &["Wang", "wang"],
            "notes",
            false,
        );
        assert!(matches!(
            duplicate_alias_descriptor.validate(),
            Err(HouseCatalogValidationError::DescriptorLabelCollision {
                label: "wang",
                canonical_name: "Equal"
            })
        ));

        let blank_notes_entry = [blank_notes_descriptor];
        assert!(matches!(
            validate_house_catalog_entries(&blank_notes_entry),
            Err(HouseCatalogValidationError::DescriptorNotesNotNormalized { label: "Equal" })
        ));
    }

    #[test]
    fn swiss_ephemeris_house_system_code_aliases_are_unique_and_round_trip() {
        let aliases = house_system_code_aliases();
        let mut seen = std::collections::BTreeSet::new();

        for alias in aliases {
            assert!(seen.insert(alias.label.to_ascii_lowercase()));
            assert_eq!(
                resolve_house_system(alias.label),
                Some(alias.system.clone())
            );
        }

        assert_eq!(aliases.len(), 22);
        assert_eq!(
            resolve_house_system("axial rotation"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(
            resolve_house_system("axial rotation system"),
            Some(HouseSystem::Meridian)
        );
        assert_eq!(resolve_house_system("X"), Some(HouseSystem::Meridian));
        assert_eq!(resolve_house_system("Y"), Some(HouseSystem::Apc));
    }
}
