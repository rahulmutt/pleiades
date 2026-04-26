//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The full ELP series data is still planned, but this crate now provides a
//! usable Moon-and-lunar-point backend for the chart MVP by combining a
//! compact Meeus-style truncated lunar position series with geocentric
//! coordinate transforms, Meeus-style mean node/perigee/apogee formulae, and
//! finite-difference mean-motion estimates. The backend accepts both TT and
//! TDB requests as dynamical-time inputs and still rejects UT-based requests
//! explicitly.
//!
//! The current lunar-theory selection is intentionally explicit: the backend
//! exposes the Moon plus the mean/true node and mean apogee/perigee channels
//! through a small structured specification, and it also lists true apogee /
//! true perigee as unsupported bodies, so future source-backed ELP work can
//! attach provenance, supported channels, unsupported channels, and date-range
//! notes without changing the public API shape. The current catalog also has
//! typed lookup helpers by source identifier, model name, structured source
//! family, and family label so future source-backed lunar variants can slot
//! into the same resolution path.
//!
//! See `docs/lunar-theory-policy.md` for the current baseline, validation
//! scope, and source/provenance posture.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, AccuracyClass, Apparentness,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    QualityAnnotation,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant,
    Latitude, Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};

mod moonposition;

const PACKAGE_NAME: &str = "pleiades-elp";
const J2000: f64 = 2_451_545.0;

/// Structured request policy for the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryRequestPolicy {
    /// Coordinate frames the current baseline exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current baseline.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current baseline.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current baseline.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current baseline accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
}

impl LunarTheoryRequestPolicy {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
            format_frames(self.supported_frames),
            format_time_scales(self.supported_time_scales),
            format_zodiac_modes(self.supported_zodiac_modes),
            format_apparentness_modes(self.supported_apparentness),
            self.supports_topocentric_observer,
        )
    }
}

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
}

/// Compact source-selection summary for the current lunar-theory baseline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheorySourceSummary {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Stable identifier for the current baseline.
    pub source_identifier: &'static str,
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

/// Returns the current lunar-theory source family.
pub const fn lunar_theory_source_family() -> LunarTheorySourceFamily {
    LunarTheorySourceFamily::MeeusStyleTruncatedAnalyticalBaseline
}

/// Returns a compact source-selection summary for the current lunar theory.
pub fn lunar_theory_source_summary() -> LunarTheorySourceSummary {
    let theory = lunar_theory_specification();
    let source = lunar_theory_source_selection();

    LunarTheorySourceSummary {
        model_name: theory.model_name,
        source_identifier: source.identifier,
        source_family_label: source.family_label(),
        source_aliases: source.source_aliases,
        citation: source.citation,
        provenance: source.material,
        validation_window: theory.validation_window,
        redistribution_note: source.redistribution_note,
        license_note: source.license_note,
    }
}

/// Returns the current lunar-theory request policy.
pub const fn lunar_theory_request_policy() -> LunarTheoryRequestPolicy {
    LUNAR_THEORY_REQUEST_POLICY
}

/// Structured description of a catalog entry in the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryCatalogEntry {
    /// Whether this catalog entry is the currently selected lunar-theory baseline.
    pub selected: bool,
    /// Structured description of the selected lunar-theory baseline.
    pub specification: LunarTheorySpecification,
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
    pub fn matches(self, entry: &LunarTheoryCatalogEntry) -> bool {
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

/// Structured description of the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheorySpecification {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Structured source family for the selected lunar-theory baseline.
    pub source_family: LunarTheorySourceFamily,
    /// Stable identifier for the selected lunar-theory baseline.
    pub source_identifier: &'static str,
    /// Canonical bibliographic citation for the selected baseline.
    pub source_citation: &'static str,
    /// Human-readable source/provenance note for the selected lunar baseline.
    pub source_material: &'static str,
    /// Alternate names or documented aliases for the selected baseline.
    pub source_aliases: &'static [&'static str],
    /// Redistribution or licensing posture for the selected baseline.
    pub redistribution_note: &'static str,
    /// Bodies/channels the current lunar baseline explicitly covers.
    pub supported_bodies: &'static [CelestialBody],
    /// Bodies/channels that are explicitly unsupported by this baseline.
    pub unsupported_bodies: &'static [CelestialBody],
    /// Structured request policy for the current baseline.
    pub request_policy: LunarTheoryRequestPolicy,
    /// Coordinate frames the current baseline exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current baseline.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current baseline.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current baseline.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current baseline accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
    /// Notes the truncation/scope policy for the current baseline series.
    pub truncation_note: &'static str,
    /// Notes on the physical output units used by the baseline.
    pub unit_note: &'static str,
    /// Notes the effective validation window or date-range posture.
    pub date_range_note: &'static str,
    /// Notes on the coordinate-frame treatment used by the baseline.
    pub frame_note: &'static str,
    /// Structured validation window represented by the current evidence slice.
    pub validation_window: TimeRange,
    /// Licensing or redistribution summary for the selected baseline source.
    pub license_note: &'static str,
}

impl LunarTheorySpecification {
    /// Returns the structured source selection for the current lunar baseline.
    pub const fn source_selection(self) -> LunarTheorySourceSelection {
        LunarTheorySourceSelection {
            family: self.source_family,
            source_aliases: self.source_aliases,
            identifier: self.source_identifier,
            citation: self.source_citation,
            material: self.source_material,
            redistribution_note: self.redistribution_note,
            license_note: self.license_note,
        }
    }

    /// Returns `true` when the provided label matches one of the documented aliases.
    pub fn matches_alias(self, label: &str) -> bool {
        self.source_aliases
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(label))
    }

    /// Returns `true` when the provided label matches the built-in lunar selection.
    pub fn matches_label(self, label: &str) -> bool {
        self.source_identifier.eq_ignore_ascii_case(label)
            || self.model_name.eq_ignore_ascii_case(label)
            || self.source_family.label().eq_ignore_ascii_case(label)
            || self.matches_alias(label)
    }
}

const SUPPORTED_LUNAR_BODIES: &[CelestialBody] = &[
    CelestialBody::Moon,
    CelestialBody::MeanNode,
    CelestialBody::TrueNode,
    CelestialBody::MeanPerigee,
    CelestialBody::MeanApogee,
];
const UNSUPPORTED_LUNAR_BODIES: &[CelestialBody] =
    &[CelestialBody::TrueApogee, CelestialBody::TruePerigee];
const LUNAR_THEORY_SOURCE_ALIASES: &[&str] = &["Meeus-style truncated lunar baseline"];

const SUPPORTED_LUNAR_FRAMES: &[CoordinateFrame] =
    &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial];
const SUPPORTED_LUNAR_TIME_SCALES: &[TimeScale] = &[TimeScale::Tt, TimeScale::Tdb];
const SUPPORTED_LUNAR_ZODIAC_MODES: &[ZodiacMode] = &[ZodiacMode::Tropical];
const SUPPORTED_LUNAR_APPARENTNESS: &[Apparentness] = &[Apparentness::Mean];
const LUNAR_THEORY_REQUEST_POLICY: LunarTheoryRequestPolicy = LunarTheoryRequestPolicy {
    supported_frames: SUPPORTED_LUNAR_FRAMES,
    supported_time_scales: SUPPORTED_LUNAR_TIME_SCALES,
    supported_zodiac_modes: SUPPORTED_LUNAR_ZODIAC_MODES,
    supported_apparentness: SUPPORTED_LUNAR_APPARENTNESS,
    supports_topocentric_observer: false,
};
const LUNAR_THEORY_VALIDATION_WINDOW: TimeRange = TimeRange::new(
    Some(Instant::new(
        pleiades_types::JulianDay::from_days(2_448_724.5),
        TimeScale::Tt,
    )),
    Some(Instant::new(
        pleiades_types::JulianDay::from_days(2_459_278.5),
        TimeScale::Tt,
    )),
);

const LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS: [Instant; 4] = [
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 - 1.0),
        TimeScale::Tt,
    ),
    Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 + 1.0),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 + 2.0),
        TimeScale::Tt,
    ),
];
const LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG: f64 = 20.0;
const LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG: f64 = 10.0;
const LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU: f64 = 0.02;

const LUNAR_THEORY_SPECIFICATION: LunarTheorySpecification = LunarTheorySpecification {
    model_name: "Compact Meeus-style truncated lunar baseline",
    source_family: LunarTheorySourceFamily::MeeusStyleTruncatedAnalyticalBaseline,
    source_identifier: "meeus-style-truncated-lunar-baseline",
    source_citation: "Jean Meeus, Astronomical Algorithms, 2nd edition, truncated lunar position and lunar node/perigee/apogee formulae adapted into a compact pure-Rust baseline",
    source_material:
        "Published lunar position, node, and mean-point formulas implemented as the current pure-Rust baseline; no vendored ELP coefficient files are used yet while full ELP coefficient selection remains pending",
    source_aliases: LUNAR_THEORY_SOURCE_ALIASES,
    redistribution_note:
        "No external coefficient-file redistribution constraints apply to the current baseline because the implementation does not vendor ELP coefficient tables yet",
    supported_bodies: SUPPORTED_LUNAR_BODIES,
    unsupported_bodies: UNSUPPORTED_LUNAR_BODIES,
    request_policy: LUNAR_THEORY_REQUEST_POLICY,
    supported_frames: SUPPORTED_LUNAR_FRAMES,
    supported_time_scales: SUPPORTED_LUNAR_TIME_SCALES,
    supported_zodiac_modes: SUPPORTED_LUNAR_ZODIAC_MODES,
    supported_apparentness: SUPPORTED_LUNAR_APPARENTNESS,
    supports_topocentric_observer: false,
    truncation_note:
        "The baseline is intentionally truncated to the Moon, mean/true node, and mean apogee/perigee channels currently exercised by validation; it is not a full ELP coefficient selection",
    unit_note:
        "Angular outputs are reported in degrees and distance outputs, when present, are reported in astronomical units",
    date_range_note:
        "Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example; no full ELP coefficient range has been published yet",
    frame_note:
        "Geocentric ecliptic coordinates are produced directly from the truncated lunar series; equatorial coordinates are derived with a mean-obliquity transform",
    validation_window: LUNAR_THEORY_VALIDATION_WINDOW,
    license_note:
        "The current baseline is handwritten pure Rust and does not redistribute external coefficient tables; any future source-backed lunar theory selection will need its own provenance and redistribution review",
};

const LUNAR_THEORY_CATALOG: &[LunarTheoryCatalogEntry] = &[LunarTheoryCatalogEntry {
    selected: true,
    specification: LUNAR_THEORY_SPECIFICATION,
}];

/// Returns the structured catalog of lunar-theory selections.
pub fn lunar_theory_catalog() -> &'static [LunarTheoryCatalogEntry] {
    LUNAR_THEORY_CATALOG
}

/// Validation errors for the structured lunar-theory catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
            Self::DuplicateAlias { alias } => write!(
                f,
                "the lunar-theory catalog contains duplicate alias `{alias}`"
            ),
            Self::SelectedEntryDoesNotRoundTrip { source_identifier } => write!(
                f,
                "the selected lunar-theory catalog entry `{source_identifier}` does not round-trip through the typed selection helper"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryCatalogValidationError {}

fn validate_lunar_theory_catalog_entries(
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

/// Validates the structured lunar-theory catalog for round-trip and alias/core-label uniqueness.
pub fn validate_lunar_theory_catalog() -> Result<(), LunarTheoryCatalogValidationError> {
    validate_lunar_theory_catalog_entries(lunar_theory_catalog())
}

/// A compact validation summary for the current lunar-theory catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// Formats the compact validation summary for release-facing reporting.
pub fn format_lunar_theory_catalog_validation_summary(
    summary: &LunarTheoryCatalogValidationSummary,
) -> String {
    let selected_source_summary = summary
        .selected_source
        .map(|source| format!("{} [{}]", source.identifier, source.family_label()))
        .unwrap_or_else(|| "none".to_string());
    let selected_alias_count = summary
        .selected_source
        .map(|source| source.source_aliases.len())
        .unwrap_or(0);

    match summary.validation_result {
        Ok(()) => format!(
            "lunar theory catalog validation: ok ({} entries, {} selected; selected source: {}; aliases={}; round-trip and alias uniqueness verified)",
            summary.entry_count,
            summary.selected_count,
            selected_source_summary,
            selected_alias_count,
        ),
        Err(error) => format!(
            "lunar theory catalog validation: error: {} ({} entries, {} selected; selected source: {}; aliases={})",
            error,
            summary.entry_count,
            summary.selected_count,
            selected_source_summary,
            selected_alias_count,
        ),
    }
}

/// Returns a compact release-facing summary of the lunar-theory catalog validation state.
pub fn lunar_theory_catalog_validation_summary_for_report() -> String {
    format_lunar_theory_catalog_validation_summary(&lunar_theory_catalog_validation_summary())
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

/// Returns the currently selected compact lunar-theory specification.
pub fn lunar_theory_specification() -> LunarTheorySpecification {
    lunar_theory_catalog()
        .iter()
        .find(|entry| entry.selected)
        .map(|entry| entry.specification)
        .unwrap_or(LUNAR_THEORY_SPECIFICATION)
}

/// Returns the structured source selection for the current lunar-theory baseline.
pub fn lunar_theory_source_selection() -> LunarTheorySourceSelection {
    lunar_theory_specification().source_selection()
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
    /// Human-readable source family label of the selected lunar-theory baseline.
    pub selected_source_family_label: &'static str,
    /// Number of aliases documented for the selected baseline.
    pub selected_alias_count: usize,
    /// Number of bodies/channels explicitly supported by the selected baseline.
    pub selected_supported_body_count: usize,
    /// Number of bodies/channels explicitly unsupported by the selected baseline.
    pub selected_unsupported_body_count: usize,
}

/// Returns a compact summary of the current lunar-theory catalog.
pub fn lunar_theory_catalog_summary() -> LunarTheoryCatalogSummary {
    let catalog = lunar_theory_catalog();
    let selected_entry = catalog
        .iter()
        .find(|entry| entry.selected)
        .unwrap_or(&catalog[0]);

    LunarTheoryCatalogSummary {
        entry_count: catalog.len(),
        selected_count: catalog.iter().filter(|entry| entry.selected).count(),
        selected_source_identifier: selected_entry.specification.source_identifier,
        selected_source_family_label: selected_entry.specification.source_family.label(),
        selected_alias_count: selected_entry.specification.source_aliases.len(),
        selected_supported_body_count: selected_entry.specification.supported_bodies.len(),
        selected_unsupported_body_count: selected_entry.specification.unsupported_bodies.len(),
    }
}

/// Formats the catalog summary for release-facing reporting.
pub fn format_lunar_theory_catalog_summary(summary: &LunarTheoryCatalogSummary) -> String {
    let entry_label = if summary.entry_count == 1 {
        "entry"
    } else {
        "entries"
    };
    let selected_label = if summary.selected_count == 1 {
        "selected entry"
    } else {
        "selected entries"
    };

    format!(
        "lunar theory catalog: {} {}, {} {}; selected source: {} [{}]; aliases={}; supported bodies={}; unsupported bodies={}",
        summary.entry_count,
        entry_label,
        summary.selected_count,
        selected_label,
        summary.selected_source_identifier,
        summary.selected_source_family_label,
        summary.selected_alias_count,
        summary.selected_supported_body_count,
        summary.selected_unsupported_body_count,
    )
}

/// Returns the release-facing catalog summary string for the current lunar-theory selection.
pub fn lunar_theory_catalog_summary_for_report() -> String {
    format_lunar_theory_catalog_summary(&lunar_theory_catalog_summary())
}

/// A compact capability summary for the current lunar-theory selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarTheoryCapabilitySummary {
    /// Human-readable model name.
    pub model_name: &'static str,
    /// Stable identifier for the selected lunar-theory baseline.
    pub source_identifier: &'static str,
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
    pub validation_window: TimeRange,
    /// Whether the current lunar-theory catalog validates cleanly.
    pub catalog_validation_ok: bool,
}

/// Returns the compact capability summary for the current lunar-theory selection.
pub fn lunar_theory_capability_summary() -> LunarTheoryCapabilitySummary {
    let theory = lunar_theory_specification();
    LunarTheoryCapabilitySummary {
        model_name: theory.model_name,
        source_identifier: theory.source_identifier,
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

/// Formats the capability summary for release-facing reporting.
pub fn format_lunar_theory_capability_summary(summary: &LunarTheoryCapabilitySummary) -> String {
    format!(
        "lunar capability summary: {} [{}; family: {}] bodies={} ({}); unsupported={} ({}); frames={} time scales={} zodiac modes={} apparentness={} topocentric observer={} validation window={}; catalog validation={}",
        summary.model_name,
        summary.source_identifier,
        summary.source_family_label,
        summary.supported_body_count,
        format_bodies(summary.supported_bodies),
        summary.unsupported_body_count,
        format_bodies(summary.unsupported_bodies),
        summary.supported_frame_count,
        summary.supported_time_scale_count,
        summary.supported_zodiac_mode_count,
        summary.supported_apparentness_count,
        summary.supports_topocentric_observer,
        format_time_range(&summary.validation_window),
        if summary.catalog_validation_ok { "ok" } else { "error" },
    )
}

/// Formats the release-facing one-line summary for a lunar-theory specification.
///
/// Validation and release tooling use this helper so lunar provenance stays
/// owned by the backend crate rather than duplicated in reporting layers.
pub fn format_lunar_theory_specification(theory: &LunarTheorySpecification) -> String {
    let source = theory.source_selection();
    format!(
        "ELP lunar theory specification: {} [{}; family: {}] ({} supported bodies: {}; {} unsupported bodies: {}); request policy: {}; citation: {}; provenance: {}; redistribution: {}; truncation: {}; units: {}; validation window: {}; date-range note: {}; frame treatment: {}; license: {}",
        theory.model_name,
        source.identifier,
        source.family,
        theory.supported_bodies.len(),
        format_bodies(theory.supported_bodies),
        theory.unsupported_bodies.len(),
        format_bodies(theory.unsupported_bodies),
        theory.request_policy.summary_line(),
        source.citation,
        source.material,
        source.redistribution_note,
        theory.truncation_note,
        theory.unit_note,
        format_time_range(&theory.validation_window),
        theory.date_range_note,
        theory.frame_note,
        source.license_note,
    )
}

/// Returns the release-facing one-line summary for the current lunar-theory selection.
///
/// The validation and release tooling uses this helper so the lunar provenance
/// summary is defined in the backend crate rather than duplicated in reporting
/// layers.
pub fn lunar_theory_summary() -> String {
    format_lunar_theory_specification(&lunar_theory_specification())
}

/// Formats the compact lunar source-selection summary for release-facing reporting.
pub fn format_lunar_theory_source_summary(summary: &LunarTheorySourceSummary) -> String {
    let aliases = if summary.source_aliases.is_empty() {
        "none".to_string()
    } else {
        summary.source_aliases.join(", ")
    };

    format!(
        "lunar source selection: {} [{}; family: {}]; aliases: {}; citation: {}; provenance: {}; validation window: {}; redistribution: {}; license: {}",
        summary.model_name,
        summary.source_identifier,
        summary.source_family_label,
        aliases,
        summary.citation,
        summary.provenance,
        format_time_range(&summary.validation_window),
        summary.redistribution_note,
        summary.license_note,
    )
}

/// Returns the release-facing one-line summary for the current lunar source selection.
pub fn lunar_theory_source_summary_for_report() -> String {
    format_lunar_theory_source_summary(&lunar_theory_source_summary())
}

/// Returns the current lunar-theory request policy summary.
pub fn lunar_theory_request_policy_summary() -> String {
    lunar_theory_request_policy().summary_line()
}

/// Returns the current lunar-theory frame-treatment summary.
pub fn lunar_theory_frame_treatment_summary() -> &'static str {
    lunar_theory_specification().frame_note
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(|body| body.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_frames(frames: &[CoordinateFrame]) -> String {
    frames
        .iter()
        .map(|frame| match frame {
            CoordinateFrame::Ecliptic => "Ecliptic",
            CoordinateFrame::Equatorial => "Equatorial",
            _ => "Other",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_time_scales(scales: &[TimeScale]) -> String {
    scales
        .iter()
        .map(|scale| match scale {
            TimeScale::Utc => "UTC",
            TimeScale::Ut1 => "UT1",
            TimeScale::Tt => "TT",
            TimeScale::Tdb => "TDB",
            _ => "Other",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_zodiac_modes(modes: &[ZodiacMode]) -> String {
    modes
        .iter()
        .map(|mode| match mode {
            ZodiacMode::Tropical => "Tropical".to_string(),
            ZodiacMode::Sidereal { ayanamsa } => format!("Sidereal ({ayanamsa:?})"),
            _ => "Other".to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    modes
        .iter()
        .map(|mode| mode.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_time_range(range: &TimeRange) -> String {
    match (range.start, range.end) {
        (Some(start), Some(end)) => format!("{} → {}", format_instant(start), format_instant(end)),
        (Some(start), None) => format!("from {}", format_instant(start)),
        (None, Some(end)) => format!("through {}", format_instant(end)),
        (None, None) => "unbounded".to_string(),
    }
}

fn format_instant(instant: Instant) -> String {
    let scale = match instant.scale {
        TimeScale::Utc => "UTC",
        TimeScale::Ut1 => "UT1",
        TimeScale::Tt => "TT",
        TimeScale::Tdb => "TDB",
        _ => "Other",
    };
    format!("JD {:.1} ({scale})", instant.julian_day.days())
}

/// A single canonical lunar evidence sample used by validation and reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarReferenceSample {
    /// Body or lunar point covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Ecliptic longitude in degrees.
    pub longitude_deg: f64,
    /// Ecliptic latitude in degrees.
    pub latitude_deg: f64,
    /// Geocentric distance in astronomical units, if available.
    pub distance_au: Option<f64>,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

/// Returns the canonical lunar evidence samples used by validation and reporting.
pub fn lunar_reference_evidence() -> &'static [LunarReferenceSample] {
    const SAMPLES: &[LunarReferenceSample] = &[
        LunarReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            longitude_deg: 133.162_655,
            latitude_deg: -3.229_126,
            distance_au: Some(368_409.7 / 149_597_870.700),
            note: "Published 1992-04-12 geocentric Moon example used as the compact lunar baseline regression sample",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
            longitude_deg: 125.044_547_9,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean node reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::TrueNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
            longitude_deg: 123.926_171_368_400_46,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 true node reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_419_914.5), TimeScale::Tt),
            longitude_deg: 0.0,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1913-05-27 mean ascending node example used to anchor the lunar node model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_436_909.5), TimeScale::Tt),
            longitude_deg: 180.0,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1959-12-07 mean ascending node example used to anchor the lunar node model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanPerigee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_459_278.5), TimeScale::Tt),
            longitude_deg: 224.891_94,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 2021-03-05 mean perigee example used to anchor the lunar perigee model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanPerigee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
            longitude_deg: 83.353_246_5,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean perigee reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::MeanApogee,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
            longitude_deg: 263.353_246_5,
            latitude_deg: 0.0,
            distance_au: None,
            note: "J2000 mean apogee reference used to anchor the lunar point model",
        },
        LunarReferenceSample {
            body: CelestialBody::TrueNode,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_419_914.5), TimeScale::Tt),
            longitude_deg: 0.876_3,
            latitude_deg: 0.0,
            distance_au: None,
            note: "Published 1913-05-27 true ascending node example used to anchor the lunar node model",
        },
    ];

    SAMPLES
}

/// A single canonical lunar equatorial evidence sample used by validation and reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarEquatorialReferenceSample {
    /// Body covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Expected equatorial coordinates.
    pub equatorial: EquatorialCoordinates,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

/// Returns the canonical lunar equatorial evidence samples used by validation and reporting.
pub fn lunar_equatorial_reference_evidence() -> &'static [LunarEquatorialReferenceSample] {
    const SAMPLES: &[LunarEquatorialReferenceSample] = &[LunarEquatorialReferenceSample {
        body: CelestialBody::Moon,
        epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
        equatorial: EquatorialCoordinates::new(
            Angle::from_degrees(134.688_470),
            Latitude::from_degrees(13.768_368),
            Some(368_409.7 / 149_597_870.700),
        ),
        note: "Published 1992-04-12 geocentric Moon RA/Dec example used to anchor the mean-obliquity equatorial transform",
    }];

    SAMPLES
}

/// A compact summary of the canonical lunar reference evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarReferenceEvidenceSummary {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
}

/// Returns a compact summary of the canonical lunar reference evidence slice.
pub fn lunar_reference_evidence_summary() -> Option<LunarReferenceEvidenceSummary> {
    let samples = lunar_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarReferenceEvidenceSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Formats the lunar reference evidence summary for release-facing reporting.
pub fn format_lunar_reference_evidence_summary(summary: &LunarReferenceEvidenceSummary) -> String {
    format!(
        "lunar reference evidence: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, validated against the published 1992-04-12 Moon example plus J2000 lunar-point anchors, including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example",
        summary.sample_count,
        summary.body_count,
        summary.earliest_epoch.julian_day.days(),
        summary.latest_epoch.julian_day.days(),
    )
}

/// Returns the release-facing lunar reference evidence summary string.
pub fn lunar_reference_evidence_summary_for_report() -> String {
    match lunar_reference_evidence_summary() {
        Some(summary) => format_lunar_reference_evidence_summary(&summary),
        None => "lunar reference evidence: unavailable".to_string(),
    }
}

/// A compact summary of the canonical lunar equatorial reference evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarEquatorialReferenceEvidenceSummary {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
}

/// Returns a compact summary of the canonical lunar equatorial reference evidence slice.
pub fn lunar_equatorial_reference_evidence_summary(
) -> Option<LunarEquatorialReferenceEvidenceSummary> {
    let samples = lunar_equatorial_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarEquatorialReferenceEvidenceSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Formats the lunar equatorial reference evidence summary for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_summary(
    summary: &LunarEquatorialReferenceEvidenceSummary,
) -> String {
    format!(
        "lunar equatorial reference evidence: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, validated against the published 1992-04-12 geocentric Moon RA/Dec example",
        summary.sample_count,
        summary.body_count,
        summary.earliest_epoch.julian_day.days(),
        summary.latest_epoch.julian_day.days(),
    )
}

/// Returns the release-facing lunar equatorial reference evidence summary string.
pub fn lunar_equatorial_reference_evidence_summary_for_report() -> String {
    match lunar_equatorial_reference_evidence_summary() {
        Some(summary) => format_lunar_equatorial_reference_evidence_summary(&summary),
        None => "lunar equatorial reference evidence: unavailable".to_string(),
    }
}

/// A reference-only apparent Moon comparison sample.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarApparentComparisonSample {
    /// Body covered by the sample.
    pub body: CelestialBody,
    /// Reference epoch used for the sample.
    pub epoch: Instant,
    /// Published apparent ecliptic longitude in degrees.
    pub apparent_longitude_deg: f64,
    /// Published apparent ecliptic latitude in degrees.
    pub apparent_latitude_deg: f64,
    /// Published apparent geocentric distance in astronomical units.
    pub apparent_distance_au: f64,
    /// Published apparent right ascension in degrees.
    pub apparent_right_ascension_deg: f64,
    /// Published apparent declination in degrees.
    pub apparent_declination_deg: f64,
    /// Human-readable note describing the provenance of the sample.
    pub note: &'static str,
}

/// Returns the reference-only apparent Moon comparison sample used by validation and reporting.
pub fn lunar_apparent_comparison_evidence() -> &'static [LunarApparentComparisonSample] {
    const SAMPLES: &[LunarApparentComparisonSample] = &[
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            apparent_longitude_deg: 133.167_264,
            apparent_latitude_deg: -3.229_126,
            apparent_distance_au: 368_409.7 / 149_597_870.700,
            apparent_right_ascension_deg: 134.688_469,
            apparent_declination_deg: 13.768_367,
            note: "Published 1992-04-12 apparent geocentric Moon example used as a reference-only mean/apparent comparison datum",
        },
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_440_214.916_7),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 336.242_307,
            apparent_latitude_deg: -2.480_685,
            apparent_distance_au: 376_090.0 / 149_597_870.700,
            apparent_right_ascension_deg: 331.293_23,
            apparent_declination_deg: -14.412_95,
            note: "Published 1968-12-24 low-accuracy Meeus-style geocentric Moon example at 10:00 UT used as a second reference-only mean/apparent comparison datum",
        },
    ];

    SAMPLES
}

/// A compact summary of the reference-only apparent Moon comparison evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarApparentComparisonSummary {
    /// Number of evidence samples in the checked-in comparison slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the comparison slice.
    pub body_count: usize,
    /// Earliest epoch covered by the comparison slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the comparison slice.
    pub latest_epoch: Instant,
    /// Epoch that produces the maximum apparent minus mean ecliptic longitude delta.
    pub max_ecliptic_longitude_epoch: Instant,
    /// Maximum apparent minus mean ecliptic longitude delta in degrees.
    pub max_ecliptic_longitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean ecliptic latitude delta.
    pub max_ecliptic_latitude_epoch: Instant,
    /// Maximum apparent minus mean ecliptic latitude delta in degrees.
    pub max_ecliptic_latitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean distance delta.
    pub max_ecliptic_distance_epoch: Instant,
    /// Maximum apparent minus mean distance delta in astronomical units.
    pub max_ecliptic_distance_delta_au: f64,
    /// Epoch that produces the maximum apparent minus mean right ascension delta.
    pub max_right_ascension_epoch: Instant,
    /// Maximum apparent minus mean right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean declination delta.
    pub max_declination_epoch: Instant,
    /// Maximum apparent minus mean declination delta in degrees.
    pub max_declination_delta_deg: f64,
}

fn lunar_apparent_comparison_baseline_sample(
    body: CelestialBody,
    epoch: Instant,
) -> Option<&'static LunarReferenceSample> {
    lunar_reference_evidence()
        .iter()
        .find(|sample| sample.body == body && sample.epoch == epoch)
}

fn lunar_apparent_comparison_equatorial_baseline_sample(
    body: CelestialBody,
    epoch: Instant,
) -> Option<&'static LunarEquatorialReferenceSample> {
    lunar_equatorial_reference_evidence()
        .iter()
        .find(|sample| sample.body == body && sample.epoch == epoch)
}

/// Returns a compact summary of the reference-only apparent Moon comparison evidence.
pub fn lunar_apparent_comparison_summary() -> Option<LunarApparentComparisonSummary> {
    let samples = lunar_apparent_comparison_evidence();
    if samples.is_empty() {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_delta_deg = 0.0;
    let mut max_ecliptic_longitude_delta_abs = 0.0;
    let mut max_ecliptic_latitude_epoch = samples[0].epoch;
    let mut max_ecliptic_latitude_delta_deg = 0.0;
    let mut max_ecliptic_latitude_delta_abs = 0.0;
    let mut max_ecliptic_distance_epoch = samples[0].epoch;
    let mut max_ecliptic_distance_delta_au = 0.0;
    let mut max_ecliptic_distance_delta_abs = 0.0;
    let mut max_right_ascension_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_deg = 0.0;
    let mut max_right_ascension_delta_abs = 0.0;
    let mut max_declination_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut max_declination_delta_abs = 0.0;

    let mut compared_sample_count = 0usize;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let Some(mean_ecliptic) =
            lunar_apparent_comparison_baseline_sample(sample.body.clone(), sample.epoch)
        else {
            continue;
        };
        let Some(mean_equatorial) =
            lunar_apparent_comparison_equatorial_baseline_sample(sample.body.clone(), sample.epoch)
        else {
            continue;
        };

        compared_sample_count += 1;
        let longitude_delta = sample.apparent_longitude_deg - mean_ecliptic.longitude_deg;
        let latitude_delta = sample.apparent_latitude_deg - mean_ecliptic.latitude_deg;
        let distance_delta =
            sample.apparent_distance_au - mean_ecliptic.distance_au.unwrap_or_default();
        let right_ascension_delta = sample.apparent_right_ascension_deg
            - mean_equatorial.equatorial.right_ascension.degrees();
        let declination_delta =
            sample.apparent_declination_deg - mean_equatorial.equatorial.declination.degrees();

        let longitude_delta_abs = longitude_delta.abs();
        if longitude_delta_abs >= max_ecliptic_longitude_delta_abs {
            max_ecliptic_longitude_delta_abs = longitude_delta_abs;
            max_ecliptic_longitude_delta_deg = longitude_delta;
            max_ecliptic_longitude_epoch = sample.epoch;
        }
        let latitude_delta_abs = latitude_delta.abs();
        if latitude_delta_abs >= max_ecliptic_latitude_delta_abs {
            max_ecliptic_latitude_delta_abs = latitude_delta_abs;
            max_ecliptic_latitude_delta_deg = latitude_delta;
            max_ecliptic_latitude_epoch = sample.epoch;
        }
        let distance_delta_abs = distance_delta.abs();
        if distance_delta_abs >= max_ecliptic_distance_delta_abs {
            max_ecliptic_distance_delta_abs = distance_delta_abs;
            max_ecliptic_distance_delta_au = distance_delta;
            max_ecliptic_distance_epoch = sample.epoch;
        }
        let right_ascension_delta_abs = right_ascension_delta.abs();
        if right_ascension_delta_abs >= max_right_ascension_delta_abs {
            max_right_ascension_delta_abs = right_ascension_delta_abs;
            max_right_ascension_delta_deg = right_ascension_delta;
            max_right_ascension_epoch = sample.epoch;
        }
        let declination_delta_abs = declination_delta.abs();
        if declination_delta_abs >= max_declination_delta_abs {
            max_declination_delta_abs = declination_delta_abs;
            max_declination_delta_deg = declination_delta;
            max_declination_epoch = sample.epoch;
        }
    }

    if compared_sample_count == 0 {
        return None;
    }

    Some(LunarApparentComparisonSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_ecliptic_longitude_epoch,
        max_ecliptic_longitude_delta_deg,
        max_ecliptic_latitude_epoch,
        max_ecliptic_latitude_delta_deg,
        max_ecliptic_distance_epoch,
        max_ecliptic_distance_delta_au,
        max_right_ascension_epoch,
        max_right_ascension_delta_deg,
        max_declination_epoch,
        max_declination_delta_deg,
    })
}

/// Formats the reference-only apparent Moon comparison summary for release-facing reporting.
pub fn format_lunar_apparent_comparison_summary(
    summary: &LunarApparentComparisonSummary,
) -> String {
    format!(
        "lunar apparent comparison evidence: {} reference-only samples across {} bodies, epoch range JD {:.1}..{:.1}, mean-only gap against the published apparent Moon examples: Δlon={:+.6}° @ JD {:.1}, Δlat={:+.6}° @ JD {:.1}, Δdist={:+.12} AU @ JD {:.1}, ΔRA={:+.6}° @ JD {:.1}, ΔDec={:+.6}° @ JD {:.1}; apparent requests remain unsupported",
        summary.sample_count,
        summary.body_count,
        summary.earliest_epoch.julian_day.days(),
        summary.latest_epoch.julian_day.days(),
        summary.max_ecliptic_longitude_delta_deg,
        summary.max_ecliptic_longitude_epoch.julian_day.days(),
        summary.max_ecliptic_latitude_delta_deg,
        summary.max_ecliptic_latitude_epoch.julian_day.days(),
        summary.max_ecliptic_distance_delta_au,
        summary.max_ecliptic_distance_epoch.julian_day.days(),
        summary.max_right_ascension_delta_deg,
        summary.max_right_ascension_epoch.julian_day.days(),
        summary.max_declination_delta_deg,
        summary.max_declination_epoch.julian_day.days(),
    )
}

/// Returns the release-facing one-line apparent comparison summary.
pub fn lunar_apparent_comparison_summary_for_report() -> String {
    match lunar_apparent_comparison_summary() {
        Some(summary) => format_lunar_apparent_comparison_summary(&summary),
        None => "lunar apparent comparison evidence: unavailable".to_string(),
    }
}

/// A compact summary of the lunar equatorial reference error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarEquatorialReferenceEvidenceEnvelope {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Epoch for the maximum absolute right ascension delta.
    pub max_right_ascension_delta_epoch: Instant,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Mean absolute right ascension delta in degrees across the evidence slice.
    pub mean_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Epoch for the maximum absolute declination delta.
    pub max_declination_delta_epoch: Instant,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Mean absolute declination delta in degrees across the evidence slice.
    pub mean_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Number of samples outside the current regression limits.
    pub outside_current_limits_count: usize,
    /// Bodies associated with samples outside the current regression limits.
    pub outlier_bodies: Vec<CelestialBody>,
    /// Whether every sample stayed within the current regression limits.
    pub within_current_limits: bool,
}

/// Returns the current lunar equatorial reference error envelope measured against the
/// checked-in reference slice.
pub fn lunar_equatorial_reference_evidence_envelope(
) -> Option<LunarEquatorialReferenceEvidenceEnvelope> {
    let samples = lunar_equatorial_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_body = samples[0].body.clone();
    let mut max_right_ascension_delta_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_deg = 0.0;
    let mut total_right_ascension_delta_deg = 0.0;
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_declination_delta_body = samples[0].body.clone();
    let mut max_declination_delta_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut total_declination_delta_deg = 0.0;
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_delta_sample_count = 0usize;
    let mut outside_current_limits_count = 0usize;
    let mut within_current_limits = true;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let result = backend
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("the canonical lunar equatorial evidence samples should remain computable");
        let equatorial = result.equatorial.expect(
            "the canonical lunar equatorial evidence samples should include equatorial coordinates",
        );

        let right_ascension_delta_deg = signed_longitude_delta_degrees(
            sample.equatorial.right_ascension.degrees(),
            equatorial.right_ascension.degrees(),
        )
        .abs();
        let declination_delta_deg =
            (equatorial.declination.degrees() - sample.equatorial.declination.degrees()).abs();
        let distance_delta_au = match (sample.equatorial.distance_au, equatorial.distance_au) {
            (Some(expected), Some(actual)) => Some((actual - expected).abs()),
            _ => None,
        };

        let right_ascension_limit = 1e-2;
        let declination_limit = 1e-2;
        let distance_limit = 1e-8;
        let sample_within_limits = right_ascension_delta_deg <= right_ascension_limit
            && declination_delta_deg <= declination_limit
            && match distance_delta_au {
                Some(delta) => delta <= distance_limit,
                None => true,
            };
        within_current_limits &= sample_within_limits;
        if !sample_within_limits {
            outside_current_limits_count += 1;
            if !outlier_bodies.contains(&sample.body) {
                outlier_bodies.push(sample.body.clone());
            }
        }

        total_right_ascension_delta_deg += right_ascension_delta_deg;
        total_declination_delta_deg += declination_delta_deg;
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_delta_sample_count += 1;
        }

        if right_ascension_delta_deg > max_right_ascension_delta_deg {
            max_right_ascension_delta_deg = right_ascension_delta_deg;
            max_right_ascension_delta_body = sample.body.clone();
            max_right_ascension_delta_epoch = sample.epoch;
        }
        if declination_delta_deg > max_declination_delta_deg {
            max_declination_delta_deg = declination_delta_deg;
            max_declination_delta_body = sample.body.clone();
            max_declination_delta_epoch = sample.epoch;
        }
        if let Some(delta) = distance_delta_au {
            match max_distance_delta_au {
                Some(current_max) if delta <= current_max => {}
                _ => {
                    max_distance_delta_body = Some(sample.body.clone());
                    max_distance_delta_epoch = Some(sample.epoch);
                    max_distance_delta_au = Some(delta);
                }
            }
        }
    }

    Some(LunarEquatorialReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_right_ascension_delta_body,
        max_right_ascension_delta_epoch,
        max_right_ascension_delta_deg,
        mean_right_ascension_delta_deg: total_right_ascension_delta_deg / samples.len() as f64,
        max_declination_delta_body,
        max_declination_delta_epoch,
        max_declination_delta_deg,
        mean_declination_delta_deg: total_declination_delta_deg / samples.len() as f64,
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

/// Formats the lunar equatorial reference error envelope for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_envelope(
    envelope: &LunarEquatorialReferenceEvidenceEnvelope,
) -> String {
    fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
        format!("{} @ {}", body, format_instant(epoch))
    }

    let distance = match (
        envelope.max_distance_delta_body.as_ref(),
        envelope.max_distance_delta_epoch,
        envelope.max_distance_delta_au,
    ) {
        (Some(body), Some(epoch), Some(delta)) => {
            format!(
                "; max Δdist={delta:.12} AU ({})",
                format_body_epoch(body, epoch)
            )
        }
        _ => String::new(),
    };
    let mean_distance = envelope
        .mean_distance_delta_au
        .map(|value| format!("; mean Δdist={value:.12} AU"))
        .unwrap_or_default();
    let limit_note = "; limits: ΔRA≤1e-2°, ΔDec≤1e-2°, Δdist≤1e-8 AU";
    let outlier_note = if envelope.outlier_bodies.is_empty() {
        "; outliers=none".to_string()
    } else {
        format!("; outliers={}", format_bodies(&envelope.outlier_bodies))
    };

    format!(
        "lunar equatorial reference error envelope: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, max ΔRA={:.12}° ({}), mean ΔRA={:.12}°, max ΔDec={:.12}° ({}), mean ΔDec={:.12}°{}{}{}{}; outside current limits={}; within current limits={}",
        envelope.sample_count,
        envelope.body_count,
        envelope.earliest_epoch.julian_day.days(),
        envelope.latest_epoch.julian_day.days(),
        envelope.max_right_ascension_delta_deg,
        format_body_epoch(&envelope.max_right_ascension_delta_body, envelope.max_right_ascension_delta_epoch),
        envelope.mean_right_ascension_delta_deg,
        envelope.max_declination_delta_deg,
        format_body_epoch(&envelope.max_declination_delta_body, envelope.max_declination_delta_epoch),
        envelope.mean_declination_delta_deg,
        distance,
        mean_distance,
        limit_note,
        outlier_note,
        envelope.outside_current_limits_count,
        envelope.within_current_limits,
    )
}

/// Returns the release-facing lunar equatorial reference error envelope string.
pub fn lunar_equatorial_reference_evidence_envelope_for_report() -> String {
    match lunar_equatorial_reference_evidence_envelope() {
        Some(envelope) => format_lunar_equatorial_reference_evidence_envelope(&envelope),
        None => "lunar equatorial reference error envelope: unavailable".to_string(),
    }
}

/// A compact summary of the lunar high-curvature continuity evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
struct LunarHighCurvatureContinuityEnvelope {
    /// Number of continuity samples in the regression slice.
    sample_count: usize,
    /// Number of distinct bodies covered by the regression slice.
    body_count: usize,
    /// Earliest epoch covered by the regression slice.
    earliest_epoch: Instant,
    /// Latest epoch covered by the regression slice.
    latest_epoch: Instant,
    /// Epoch at the start of the largest adjacent longitude step.
    max_longitude_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent longitude step.
    max_longitude_step_end_epoch: Instant,
    /// Largest adjacent longitude step in degrees.
    max_longitude_step_deg: f64,
    /// Epoch at the start of the largest adjacent latitude step.
    max_latitude_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent latitude step.
    max_latitude_step_end_epoch: Instant,
    /// Largest adjacent latitude step in degrees.
    max_latitude_step_deg: f64,
    /// Epoch at the start of the largest adjacent distance step.
    max_distance_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent distance step.
    max_distance_step_end_epoch: Instant,
    /// Largest adjacent distance step in astronomical units.
    max_distance_step_au: f64,
    /// Whether every adjacent step stayed within the current regression limits.
    within_regression_limits: bool,
}

fn lunar_high_curvature_continuity_envelope() -> Option<LunarHighCurvatureContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut latest_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_longitude_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_longitude_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_longitude_step_deg = 0.0;
    let mut max_latitude_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_latitude_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_latitude_step_deg = 0.0;
    let mut max_distance_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_distance_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for instant in LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS {
        let result = backend
            .position(&EphemerisRequest::new(CelestialBody::Moon, instant))
            .expect("the high-curvature lunar continuity samples should remain computable");
        let ecliptic = result.ecliptic.expect(
            "the high-curvature lunar continuity samples should include ecliptic coordinates",
        );
        let longitude = ecliptic.longitude.degrees();
        let latitude = ecliptic.latitude.degrees();
        let distance = ecliptic
            .distance_au
            .expect("the high-curvature lunar continuity samples should include distance");

        bodies.insert(result.body.to_string());
        if instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = instant;
        }
        if instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = instant;
        }

        if let Some((previous_epoch, previous_longitude, previous_latitude, previous_distance)) =
            previous_sample
        {
            let longitude_step =
                signed_longitude_delta_degrees(previous_longitude, longitude).abs();
            let latitude_step = (latitude - previous_latitude).abs();
            let distance_step = (distance - previous_distance).abs();

            within_regression_limits &= longitude_step <= LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG
                && latitude_step <= LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG
                && distance_step <= LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;

            if longitude_step > max_longitude_step_deg {
                max_longitude_step_deg = longitude_step;
                max_longitude_step_start_epoch = previous_epoch;
                max_longitude_step_end_epoch = instant;
            }
            if latitude_step > max_latitude_step_deg {
                max_latitude_step_deg = latitude_step;
                max_latitude_step_start_epoch = previous_epoch;
                max_latitude_step_end_epoch = instant;
            }
            if distance_step > max_distance_step_au {
                max_distance_step_au = distance_step;
                max_distance_step_start_epoch = previous_epoch;
                max_distance_step_end_epoch = instant;
            }
        }

        previous_sample = Some((instant, longitude, latitude, distance));
    }

    Some(LunarHighCurvatureContinuityEnvelope {
        sample_count: LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_step_start_epoch,
        max_longitude_step_end_epoch,
        max_longitude_step_deg,
        max_latitude_step_start_epoch,
        max_latitude_step_end_epoch,
        max_latitude_step_deg,
        max_distance_step_start_epoch,
        max_distance_step_end_epoch,
        max_distance_step_au,
        within_regression_limits,
    })
}

fn format_lunar_high_curvature_continuity_evidence_envelope(
    envelope: &LunarHighCurvatureContinuityEnvelope,
) -> String {
    format!(
        "lunar high-curvature continuity evidence: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, max adjacent Δlon={:.12}° ({} → {}), max adjacent Δlat={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: Δlon≤{:.1}°, Δlat≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
        envelope.sample_count,
        envelope.body_count,
        envelope.earliest_epoch.julian_day.days(),
        envelope.latest_epoch.julian_day.days(),
        envelope.max_longitude_step_deg,
        format_instant(envelope.max_longitude_step_start_epoch),
        format_instant(envelope.max_longitude_step_end_epoch),
        envelope.max_latitude_step_deg,
        format_instant(envelope.max_latitude_step_start_epoch),
        format_instant(envelope.max_latitude_step_end_epoch),
        envelope.max_distance_step_au,
        format_instant(envelope.max_distance_step_start_epoch),
        format_instant(envelope.max_distance_step_end_epoch),
        LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG,
        LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG,
        LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
        envelope.within_regression_limits,
    )
}

/// Returns the release-facing lunar high-curvature continuity evidence string.
pub fn lunar_high_curvature_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_continuity_envelope() {
        Some(envelope) => format_lunar_high_curvature_continuity_evidence_envelope(&envelope),
        None => "lunar high-curvature continuity evidence: unavailable".to_string(),
    }
}

const LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG: f64 = 20.0;
const LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG: f64 = 10.0;

/// A compact summary of the lunar high-curvature equatorial continuity evidence slice.
#[derive(Clone, Copy, Debug, PartialEq)]
struct LunarHighCurvatureEquatorialContinuityEnvelope {
    /// Number of continuity samples in the regression slice.
    sample_count: usize,
    /// Number of distinct bodies covered by the regression slice.
    body_count: usize,
    /// Earliest epoch covered by the regression slice.
    earliest_epoch: Instant,
    /// Latest epoch covered by the regression slice.
    latest_epoch: Instant,
    /// Epoch at the start of the largest adjacent right ascension step.
    max_right_ascension_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent right ascension step.
    max_right_ascension_step_end_epoch: Instant,
    /// Largest adjacent right ascension step in degrees.
    max_right_ascension_step_deg: f64,
    /// Epoch at the start of the largest adjacent declination step.
    max_declination_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent declination step.
    max_declination_step_end_epoch: Instant,
    /// Largest adjacent declination step in degrees.
    max_declination_step_deg: f64,
    /// Epoch at the start of the largest adjacent distance step.
    max_distance_step_start_epoch: Instant,
    /// Epoch at the end of the largest adjacent distance step.
    max_distance_step_end_epoch: Instant,
    /// Largest adjacent distance step in astronomical units.
    max_distance_step_au: f64,
    /// Whether every adjacent step stayed within the current regression limits.
    within_regression_limits: bool,
}

fn lunar_high_curvature_equatorial_continuity_envelope(
) -> Option<LunarHighCurvatureEquatorialContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut latest_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_right_ascension_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_right_ascension_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_right_ascension_step_deg = 0.0;
    let mut max_declination_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_declination_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_declination_step_deg = 0.0;
    let mut max_distance_step_start_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_distance_step_end_epoch = LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS[0];
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for instant in LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS {
        let result = backend
            .position(&EphemerisRequest::new(CelestialBody::Moon, instant))
            .expect("the high-curvature lunar continuity samples should remain computable");
        let equatorial = result.equatorial.expect(
            "the high-curvature lunar continuity samples should include equatorial coordinates",
        );
        let right_ascension = equatorial.right_ascension.degrees();
        let declination = equatorial.declination.degrees();
        let distance = equatorial
            .distance_au
            .expect("the high-curvature lunar continuity samples should include distance");

        bodies.insert(result.body.to_string());
        if instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = instant;
        }
        if instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = instant;
        }

        if let Some((
            previous_epoch,
            previous_right_ascension,
            previous_declination,
            previous_distance,
        )) = previous_sample
        {
            let right_ascension_step =
                signed_longitude_delta_degrees(previous_right_ascension, right_ascension).abs();
            let declination_step = (declination - previous_declination).abs();
            let distance_step = (distance - previous_distance).abs();

            within_regression_limits &= right_ascension_step
                <= LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG
                && declination_step <= LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG
                && distance_step <= LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;

            if right_ascension_step > max_right_ascension_step_deg {
                max_right_ascension_step_deg = right_ascension_step;
                max_right_ascension_step_start_epoch = previous_epoch;
                max_right_ascension_step_end_epoch = instant;
            }
            if declination_step > max_declination_step_deg {
                max_declination_step_deg = declination_step;
                max_declination_step_start_epoch = previous_epoch;
                max_declination_step_end_epoch = instant;
            }
            if distance_step > max_distance_step_au {
                max_distance_step_au = distance_step;
                max_distance_step_start_epoch = previous_epoch;
                max_distance_step_end_epoch = instant;
            }
        }

        previous_sample = Some((instant, right_ascension, declination, distance));
    }

    Some(LunarHighCurvatureEquatorialContinuityEnvelope {
        sample_count: LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_right_ascension_step_start_epoch,
        max_right_ascension_step_end_epoch,
        max_right_ascension_step_deg,
        max_declination_step_start_epoch,
        max_declination_step_end_epoch,
        max_declination_step_deg,
        max_distance_step_start_epoch,
        max_distance_step_end_epoch,
        max_distance_step_au,
        within_regression_limits,
    })
}

fn format_lunar_high_curvature_equatorial_continuity_evidence_envelope(
    envelope: &LunarHighCurvatureEquatorialContinuityEnvelope,
) -> String {
    format!(
        "lunar high-curvature equatorial continuity evidence: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, max adjacent ΔRA={:.12}° ({} → {}), max adjacent ΔDec={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: ΔRA≤{:.1}°, ΔDec≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
        envelope.sample_count,
        envelope.body_count,
        envelope.earliest_epoch.julian_day.days(),
        envelope.latest_epoch.julian_day.days(),
        envelope.max_right_ascension_step_deg,
        format_instant(envelope.max_right_ascension_step_start_epoch),
        format_instant(envelope.max_right_ascension_step_end_epoch),
        envelope.max_declination_step_deg,
        format_instant(envelope.max_declination_step_start_epoch),
        format_instant(envelope.max_declination_step_end_epoch),
        envelope.max_distance_step_au,
        format_instant(envelope.max_distance_step_start_epoch),
        format_instant(envelope.max_distance_step_end_epoch),
        LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG,
        LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG,
        LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
        envelope.within_regression_limits,
    )
}

/// Returns the release-facing lunar high-curvature equatorial continuity evidence string.
pub fn lunar_high_curvature_equatorial_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_equatorial_continuity_envelope() {
        Some(envelope) => {
            format_lunar_high_curvature_equatorial_continuity_evidence_envelope(&envelope)
        }
        None => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
    }
}

/// A compact summary of the lunar reference error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarReferenceEvidenceEnvelope {
    /// Number of evidence samples in the checked-in reference slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the evidence slice.
    pub body_count: usize,
    /// Earliest epoch covered by the evidence slice.
    pub earliest_epoch: Instant,
    /// Latest epoch covered by the evidence slice.
    pub latest_epoch: Instant,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Epoch for the maximum absolute longitude delta.
    pub max_longitude_delta_epoch: Instant,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Mean absolute longitude delta in degrees across the evidence slice.
    pub mean_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Epoch for the maximum absolute latitude delta.
    pub max_latitude_delta_epoch: Instant,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta in degrees across the evidence slice.
    pub mean_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Number of samples outside the current regression limits.
    pub outside_current_limits_count: usize,
    /// Bodies associated with samples outside the current regression limits.
    pub outlier_bodies: Vec<CelestialBody>,
    /// Whether every sample stayed within the current regression limits.
    pub within_current_limits: bool,
}

/// Returns the current lunar reference error envelope measured against the
/// checked-in reference slice.
pub fn lunar_reference_evidence_envelope() -> Option<LunarReferenceEvidenceEnvelope> {
    let samples = lunar_reference_evidence();
    if samples.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_longitude_delta_body = samples[0].body.clone();
    let mut max_longitude_delta_epoch = samples[0].epoch;
    let mut max_longitude_delta_deg = 0.0;
    let mut total_longitude_delta_deg = 0.0;
    let mut max_latitude_delta_body = samples[0].body.clone();
    let mut max_latitude_delta_epoch = samples[0].epoch;
    let mut max_latitude_delta_deg = 0.0;
    let mut total_latitude_delta_deg = 0.0;
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_delta_sample_count = 0usize;
    let mut outside_current_limits_count = 0usize;
    let mut within_current_limits = true;

    for sample in samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }

        let result = backend
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("the canonical lunar evidence samples should remain computable");
        let ecliptic = result
            .ecliptic
            .expect("the canonical lunar evidence samples should include ecliptic coordinates");

        let longitude_delta_deg =
            signed_longitude_delta_degrees(sample.longitude_deg, ecliptic.longitude.degrees())
                .abs();
        let latitude_delta_deg = (ecliptic.latitude.degrees() - sample.latitude_deg).abs();
        let distance_delta_au = match (sample.distance_au, ecliptic.distance_au) {
            (Some(expected), Some(actual)) => Some((actual - expected).abs()),
            _ => None,
        };

        let longitude_limit = if sample.body == CelestialBody::MeanNode {
            1e-1
        } else {
            1e-4
        };
        let latitude_limit = 1e-4;
        let distance_limit = 1e-8;
        let sample_within_limits = longitude_delta_deg <= longitude_limit
            && latitude_delta_deg <= latitude_limit
            && match distance_delta_au {
                Some(delta) => delta <= distance_limit,
                None => true,
            };
        within_current_limits &= sample_within_limits;
        if !sample_within_limits {
            outside_current_limits_count += 1;
            if !outlier_bodies.contains(&sample.body) {
                outlier_bodies.push(sample.body.clone());
            }
        }

        total_longitude_delta_deg += longitude_delta_deg;
        total_latitude_delta_deg += latitude_delta_deg;
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_delta_sample_count += 1;
        }

        if longitude_delta_deg > max_longitude_delta_deg {
            max_longitude_delta_deg = longitude_delta_deg;
            max_longitude_delta_body = sample.body.clone();
            max_longitude_delta_epoch = sample.epoch;
        }
        if latitude_delta_deg > max_latitude_delta_deg {
            max_latitude_delta_deg = latitude_delta_deg;
            max_latitude_delta_body = sample.body.clone();
            max_latitude_delta_epoch = sample.epoch;
        }
        if let Some(delta) = distance_delta_au {
            match max_distance_delta_au {
                Some(current_max) if delta <= current_max => {}
                _ => {
                    max_distance_delta_body = Some(sample.body.clone());
                    max_distance_delta_epoch = Some(sample.epoch);
                    max_distance_delta_au = Some(delta);
                }
            }
        }
    }

    Some(LunarReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_delta_body,
        max_longitude_delta_epoch,
        max_longitude_delta_deg,
        mean_longitude_delta_deg: total_longitude_delta_deg / samples.len() as f64,
        max_latitude_delta_body,
        max_latitude_delta_epoch,
        max_latitude_delta_deg,
        mean_latitude_delta_deg: total_latitude_delta_deg / samples.len() as f64,
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

/// Formats the lunar reference error envelope for release-facing reporting.
pub fn format_lunar_reference_evidence_envelope(
    envelope: &LunarReferenceEvidenceEnvelope,
) -> String {
    fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
        format!("{} @ {}", body, format_instant(epoch))
    }

    let distance = match (
        envelope.max_distance_delta_body.as_ref(),
        envelope.max_distance_delta_epoch,
        envelope.max_distance_delta_au,
    ) {
        (Some(body), Some(epoch), Some(delta)) => {
            format!(
                "; max Δdist={delta:.12} AU ({})",
                format_body_epoch(body, epoch)
            )
        }
        _ => String::new(),
    };
    let mean_distance = envelope
        .mean_distance_delta_au
        .map(|value| format!("; mean Δdist={value:.12} AU"))
        .unwrap_or_default();
    let limit_note = "; limits: Δlon≤1e-4° (1e-1° for mean node), Δlat≤1e-4°, Δdist≤1e-8 AU";
    let outlier_note = if envelope.outlier_bodies.is_empty() {
        "; outliers=none".to_string()
    } else {
        format!("; outliers={}", format_bodies(&envelope.outlier_bodies))
    };

    format!(
        "lunar reference error envelope: {} samples across {} bodies, epoch range JD {:.1}..{:.1}, max Δlon={:.12}° ({}), mean Δlon={:.12}°, max Δlat={:.12}° ({}), mean Δlat={:.12}°{}{}{}{}; outside current limits={}; within current limits={}",
        envelope.sample_count,
        envelope.body_count,
        envelope.earliest_epoch.julian_day.days(),
        envelope.latest_epoch.julian_day.days(),
        envelope.max_longitude_delta_deg,
        format_body_epoch(&envelope.max_longitude_delta_body, envelope.max_longitude_delta_epoch),
        envelope.mean_longitude_delta_deg,
        envelope.max_latitude_delta_deg,
        format_body_epoch(&envelope.max_latitude_delta_body, envelope.max_latitude_delta_epoch),
        envelope.mean_latitude_delta_deg,
        distance,
        mean_distance,
        limit_note,
        outlier_note,
        envelope.outside_current_limits_count,
        envelope.within_current_limits,
    )
}

/// Returns the release-facing lunar reference error envelope string.
pub fn lunar_reference_evidence_envelope_for_report() -> String {
    match lunar_reference_evidence_envelope() {
        Some(envelope) => format_lunar_reference_evidence_envelope(&envelope),
        None => "lunar reference error envelope: unavailable".to_string(),
    }
}

/// A pure-Rust lunar backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct ElpBackend;

impl ElpBackend {
    /// Creates a new backend instance.
    pub const fn new() -> Self {
        Self
    }

    fn days_since_j2000(instant: Instant) -> f64 {
        instant.julian_day.days() - J2000
    }

    fn julian_centuries(instant: Instant) -> f64 {
        Self::days_since_j2000(instant) / 36_525.0
    }

    fn mean_obliquity_degrees(instant: Instant) -> f64 {
        let t = Self::julian_centuries(instant);
        23.439_291_111_111_11
            - 0.013_004_166_666_666_667 * t
            - 0.000_000_163_888_888_888_888_88 * t * t
            + 0.000_000_503_611_111_111_111_1 * t * t * t
    }

    fn moon_ecliptic_coordinates(days: f64) -> EclipticCoordinates {
        let (longitude, latitude, distance_au) = moonposition::position(J2000 + days);
        EclipticCoordinates::new(longitude, latitude, Some(distance_au))
    }

    fn mean_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        normalize_degrees(
            125.044_547_9
                + (-1_934.136_289_1 + (0.002_075_4 + (1.0 / 476_441.0 - t / 60_616_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_perigee_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        normalize_degrees(
            83.353_246_5
                + (4_069.013_728_7 + (-0.010_32 + (-1.0 / 80_053.0 + t / 18_999_000.0) * t) * t)
                    * t,
        )
    }

    fn mean_apogee_longitude(days: f64) -> f64 {
        normalize_degrees(Self::mean_perigee_longitude(days) + 180.0)
    }

    fn true_node_longitude(days: f64) -> f64 {
        let t = days / 36_525.0;
        let mean_node = Self::mean_node_longitude(days).to_radians();
        let mean_elongation = normalize_degrees(
            297.850_192_1
                + (445_267.111_403_4
                    + (-0.001_881_9 + (1.0 / 545_868.0 - t / 113_065_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let solar_anomaly = normalize_degrees(
            357.529_109_2 + (35_999.050_290_9 + (-0.000_153_6 + t / 24_490_000.0) * t) * t,
        )
        .to_radians();
        let lunar_anomaly = normalize_degrees(
            134.963_396_4
                + (477_198.867_505_5 + (0.008_741_4 + (1.0 / 69_699.9 + t / 14_712_000.0) * t) * t)
                    * t,
        )
        .to_radians();
        let latitude_argument = normalize_degrees(
            93.272_095_0
                + (483_202.017_523_3
                    + (-0.003_653_9 + (-1.0 / 3_526_000.0 + t / 863_310_000.0) * t) * t)
                    * t,
        )
        .to_radians();

        normalize_degrees(
            mean_node.to_degrees()
                + (-1.4979 * (2.0 * (mean_elongation - latitude_argument)).sin()
                    - 0.15 * solar_anomaly.sin()
                    - 0.1226 * (2.0 * mean_elongation).sin()
                    + 0.1176 * (2.0 * latitude_argument).sin()
                    - 0.0801 * (2.0 * (lunar_anomaly - latitude_argument)).sin()),
        )
    }

    fn ecliptic_for_body(body: CelestialBody, days: f64) -> Option<EclipticCoordinates> {
        match body {
            CelestialBody::Moon => Some(Self::moon_ecliptic_coordinates(days)),
            CelestialBody::MeanNode => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_node_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::TrueNode => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::true_node_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::MeanApogee => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_apogee_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            CelestialBody::MeanPerigee => Some(EclipticCoordinates::new(
                Longitude::from_degrees(Self::mean_perigee_longitude(days)),
                Latitude::from_degrees(0.0),
                None,
            )),
            _ => None,
        }
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // Match the planetary backend's chart-facing convention: these are
        // symmetric finite-difference rates for the same mean geometric model,
        // not apparent velocities from a full lunar theory.
        const HALF_SPAN_DAYS: f64 = 0.5;
        const FULL_SPAN_DAYS: f64 = HALF_SPAN_DAYS * 2.0;

        let before = Self::ecliptic_for_body(body.clone(), days - HALF_SPAN_DAYS)?;
        let after = Self::ecliptic_for_body(body, days + HALF_SPAN_DAYS)?;

        let longitude_speed =
            signed_longitude_delta_degrees(before.longitude.degrees(), after.longitude.degrees())
                / FULL_SPAN_DAYS;
        let latitude_speed =
            (after.latitude.degrees() - before.latitude.degrees()) / FULL_SPAN_DAYS;
        let distance_speed = match (before.distance_au, after.distance_au) {
            (Some(before), Some(after)) => Some((after - before) / FULL_SPAN_DAYS),
            _ => None,
        };

        Some(Motion::new(
            Some(longitude_speed),
            Some(latitude_speed),
            distance_speed,
        ))
    }

    fn ecliptic_point_to_equatorial(
        longitude: Longitude,
        latitude: Latitude,
        instant: Instant,
        distance_au: Option<f64>,
    ) -> EquatorialCoordinates {
        EclipticCoordinates::new(longitude, latitude, distance_au)
            .to_equatorial(Angle::from_degrees(Self::mean_obliquity_degrees(instant)))
    }
}

impl EphemerisBackend for ElpBackend {
    fn metadata(&self) -> BackendMetadata {
        let theory = lunar_theory_specification();
        let source = theory.source_selection();
        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: format!(
                    "{} [{}; family: {}] {} The backend exposes the Moon plus mean/true node and mean apogee/perigee channels as an explicit lunar-theory selection, while explicitly leaving true apogee/perigee unsupported for now; {}",
                    theory.model_name,
                    source.identifier,
                    source.family,
                    source.citation,
                    source.license_note,
                ),
                data_sources: vec![
                    "Meeus-style truncated lunar orbit formulas implemented in pure Rust; see docs/lunar-theory-policy.md for the current baseline scope".to_string(),
                    source.identifier.to_string(),
                    source.citation.to_string(),
                    source.material.to_string(),
                    source.redistribution_note.to_string(),
                    theory.truncation_note.to_string(),
                    theory.unit_note.to_string(),
                    source.license_note.to_string(),
                    theory.date_range_note.to_string(),
                    theory.frame_note.to_string(),
                ],
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: lunar_theory_supported_bodies().to_vec(),
            supported_frames: vec![CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            capabilities: BackendCapabilities {
                geocentric: true,
                topocentric: false,
                apparent: false,
                mean: true,
                batch: true,
                native_sidereal: false,
            },
            accuracy: AccuracyClass::Approximate,
            deterministic: true,
            offline: true,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        lunar_theory_supported_bodies().contains(&body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if !self.supports_body(req.body.clone()) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "the ELP backend currently serves the Moon, lunar nodes, and mean lunar apogee/perigee only",
            ));
        }

        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the ELP backend currently exposes tropical coordinates only",
            ));
        }

        validate_request_policy(
            req,
            "the ELP backend",
            SUPPORTED_LUNAR_TIME_SCALES,
            SUPPORTED_LUNAR_FRAMES,
            false,
        )?;

        validate_observer_policy(req, "the ELP backend", false)?;

        let days = Self::days_since_j2000(req.instant);
        let body = req.body.clone();
        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = QualityAnnotation::Approximate;
        match body {
            CelestialBody::Moon => {
                let coords = Self::moon_ecliptic_coordinates(days);
                result.ecliptic = Some(coords);
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    coords.longitude,
                    coords.latitude,
                    req.instant,
                    coords.distance_au,
                ));
            }
            CelestialBody::MeanNode => {
                let longitude = Longitude::from_degrees(Self::mean_node_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::TrueNode => {
                let longitude = Longitude::from_degrees(Self::true_node_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::MeanApogee => {
                let longitude = Longitude::from_degrees(Self::mean_apogee_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            CelestialBody::MeanPerigee => {
                let longitude = Longitude::from_degrees(Self::mean_perigee_longitude(days));
                let latitude = Latitude::from_degrees(0.0);
                result.ecliptic = Some(EclipticCoordinates::new(longitude, latitude, None));
                result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                    longitude,
                    latitude,
                    req.instant,
                    None,
                ));
            }
            _ => unreachable!("body support should be validated before position queries"),
        }
        result.motion = Self::motion(body, days);
        Ok(result)
    }
}

fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(PACKAGE_NAME, "pleiades-elp");
    }

    #[test]
    fn lunar_theory_summary_mentions_the_selected_lunar_theory() {
        let summary = lunar_theory_summary();
        let theory = lunar_theory_specification();
        let formatted = format_lunar_theory_specification(&theory);

        assert_eq!(summary, formatted);
        let source = theory.source_selection();
        let source_selection = lunar_theory_source_selection();
        assert_eq!(source_selection, source);
        assert!(summary.contains(theory.model_name));
        assert!(summary.contains(theory.source_identifier));
        assert!(summary.contains(theory.source_family.label()));
        assert_eq!(theory.source_family, lunar_theory_source_family());
        assert_eq!(
            theory.source_family.to_string(),
            theory.source_family.label()
        );
        assert_eq!(source.family, theory.source_family);
        assert_eq!(source.source_aliases, theory.source_aliases);
        assert_eq!(source.identifier, theory.source_identifier);
        assert_eq!(source.citation, theory.source_citation);
        assert_eq!(
            source_selection.family_label(),
            theory.source_family.label()
        );
        assert_eq!(source.material, theory.source_material);
        assert_eq!(source.redistribution_note, theory.redistribution_note);
        assert_eq!(source.license_note, theory.license_note);
        assert_eq!(source.family_label(), theory.source_family.label());
        assert_eq!(
            source.catalog_key(),
            LunarTheoryCatalogKey::SourceIdentifier(theory.source_identifier)
        );
        assert_eq!(
            source.family_key(),
            LunarTheoryCatalogKey::SourceFamily(theory.source_family)
        );
        assert_eq!(resolve_lunar_theory_by_selection(source), Some(theory));
        let catalog = lunar_theory_catalog();
        assert_eq!(
            resolve_lunar_theory("Meeus-style truncated lunar baseline"),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_alias(theory.source_aliases[0]),
            Some(theory)
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_alias(theory.source_aliases[0]),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::Alias(
                theory.source_aliases[0],
            )),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(theory.source_aliases[0]),
            Some(catalog[0])
        );
        assert!(resolve_lunar_theory_by_alias("not-a-lunar-alias").is_none());

        let source_summary = lunar_theory_source_summary();
        assert_eq!(source_summary.model_name, theory.model_name);
        assert_eq!(source_summary.source_identifier, theory.source_identifier);
        assert_eq!(
            source_summary.source_family_label,
            theory.source_family.label()
        );
        assert_eq!(source_summary.citation, theory.source_citation);
        assert_eq!(source_summary.provenance, theory.source_material);
        assert_eq!(source_summary.validation_window, theory.validation_window);
        assert_eq!(source_summary.source_aliases, theory.source_aliases);
        assert_eq!(
            source_summary.redistribution_note,
            theory.redistribution_note
        );
        assert_eq!(source_summary.license_note, theory.license_note);
        assert_eq!(
            format_lunar_theory_source_summary(&source_summary),
            lunar_theory_source_summary_for_report()
        );
        assert!(lunar_theory_source_summary_for_report().contains("lunar source selection: "));
        assert!(lunar_theory_source_summary_for_report()
            .contains("aliases: Meeus-style truncated lunar baseline"));
        assert!(lunar_theory_source_summary_for_report().contains(theory.model_name));
        assert!(lunar_theory_source_summary_for_report().contains(theory.source_identifier));
        assert!(lunar_theory_source_summary_for_report()
            .contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(summary.contains(source.citation));
        assert!(summary.contains("Moon, Mean Node, True Node, Mean Perigee, Mean Apogee"));
        assert!(summary.contains("unsupported bodies: True Apogee, True Perigee"));
        assert!(summary.contains("validation window: JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(summary.contains(
            "geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform"
        ));
        assert!(summary.contains("mean apogee"));
        assert_eq!(
            lunar_theory_request_policy_summary(),
            theory.request_policy.summary_line()
        );
        assert!(summary.contains("frames=Ecliptic, Equatorial"));
        assert!(summary.contains("time scales=TT, TDB"));
        assert!(summary.contains("zodiac modes=Tropical"));
        assert!(summary.contains("apparentness=Mean"));
        assert!(summary.contains("topocentric observer=false"));

        let catalog_summary = lunar_theory_catalog_summary();
        assert_eq!(catalog.len(), 1);
        assert!(catalog[0].selected);
        assert_eq!(catalog[0].specification, theory);
        assert_eq!(catalog_summary.entry_count, 1);
        assert_eq!(catalog_summary.selected_count, 1);
        assert_eq!(
            catalog_summary.selected_source_identifier,
            theory.source_identifier
        );
        assert_eq!(
            catalog_summary.selected_source_family_label,
            theory.source_family.label()
        );
        assert_eq!(
            catalog_summary.selected_alias_count,
            theory.source_aliases.len()
        );
        assert_eq!(
            catalog_summary.selected_supported_body_count,
            theory.supported_bodies.len()
        );
        assert_eq!(
            catalog_summary.selected_unsupported_body_count,
            theory.unsupported_bodies.len()
        );
        assert_eq!(
            format_lunar_theory_catalog_summary(&catalog_summary),
            lunar_theory_catalog_summary_for_report()
        );
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("lunar theory catalog: 1 entry, 1 selected entry"));
        let catalog_validation_summary = lunar_theory_catalog_validation_summary();
        assert_eq!(catalog_validation_summary.entry_count, 1);
        assert_eq!(catalog_validation_summary.selected_count, 1);
        assert_eq!(catalog_validation_summary.selected_source, Some(source));
        assert!(catalog_validation_summary.validation_result.is_ok());
        assert_eq!(
            format_lunar_theory_catalog_validation_summary(&catalog_validation_summary),
            lunar_theory_catalog_validation_summary_for_report()
        );
        assert!(lunar_theory_catalog_validation_summary_for_report()
            .contains("lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; aliases=1"));
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("selected source: meeus-style-truncated-lunar-baseline"));
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("aliases=1; supported bodies=5; unsupported bodies=2"));
        assert!(lunar_theory_catalog_validation_summary_for_report().contains("aliases=1"));
        assert_eq!(
            lunar_theory_catalog_entry_for_source_identifier(theory.source_identifier),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_model_name(theory.model_name),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_source_family(theory.source_family),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_family_label(theory.source_family.label()),
            Some(catalog[0])
        );
        assert_eq!(resolve_lunar_theory(theory.source_identifier), Some(theory));
        assert_eq!(resolve_lunar_theory(theory.model_name), Some(theory));
        assert_eq!(
            resolve_lunar_theory_by_family(theory.source_family),
            Some(theory)
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(theory.source_family.label()),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceIdentifier(
                theory.source_identifier,
            )),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(
                LunarTheoryCatalogKey::ModelName(theory.model_name,)
            ),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::FamilyLabel(
                theory.source_family.label(),
            )),
            Some(catalog[0])
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceIdentifier(
                theory.source_identifier,
            )),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::ModelName(theory.model_name)),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceFamily(theory.source_family)),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel(
                theory.source_family.label(),
            )),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::Alias(
                "Meeus-style truncated lunar baseline",
            )),
            Some(theory)
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_selection(source),
            Some(catalog[0])
        );
        assert!(resolve_lunar_theory("not-a-lunar-theory").is_none());
        assert!(resolve_lunar_theory_by_family(
            LunarTheorySourceFamily::MeeusStyleTruncatedAnalyticalBaseline
        )
        .is_some());
        assert!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel("not-a-family"))
                .is_none()
        );
        assert!(validate_lunar_theory_catalog().is_ok());
    }

    #[test]
    fn lunar_theory_catalog_resolvers_are_case_insensitive_across_key_families() {
        let theory = lunar_theory_specification();
        let catalog = lunar_theory_catalog();
        let source_identifier_upper = theory.source_identifier.to_uppercase();
        let model_name_upper = theory.model_name.to_uppercase();
        let family_label_upper = theory.source_family.label().to_uppercase();
        let alias_upper = theory.source_aliases[0].to_uppercase();

        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::SourceIdentifier(
                source_identifier_upper.as_str(),
            )),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::ModelName(
                model_name_upper.as_str(),
            )),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::FamilyLabel(
                family_label_upper.as_str(),
            )),
            Some(theory)
        );
        assert_eq!(
            resolve_lunar_theory_by_key(LunarTheoryCatalogKey::Alias(alias_upper.as_str())),
            Some(theory)
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::SourceIdentifier(
                source_identifier_upper.as_str(),
            )),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::ModelName(
                model_name_upper.as_str(),
            )),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::FamilyLabel(
                family_label_upper.as_str(),
            )),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_key(LunarTheoryCatalogKey::Alias(alias_upper.as_str())),
            Some(catalog[0])
        );
        assert_eq!(
            resolve_lunar_theory_by_alias(alias_upper.as_str()),
            Some(theory)
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(source_identifier_upper.as_str()),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(model_name_upper.as_str()),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(family_label_upper.as_str()),
            Some(catalog[0])
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_label(alias_upper.as_str()),
            Some(catalog[0])
        );
    }

    #[test]
    fn lunar_theory_catalog_validation_rejects_duplicate_aliases_within_entry() {
        let duplicate_alias_catalog = [LunarTheoryCatalogEntry {
            selected: true,
            specification: LunarTheorySpecification {
                source_aliases: &[
                    "Meeus-style truncated lunar baseline",
                    "meeus-style truncated lunar baseline",
                ],
                ..LUNAR_THEORY_SPECIFICATION
            },
        }];

        assert!(matches!(
            validate_lunar_theory_catalog_entries(&duplicate_alias_catalog),
            Err(LunarTheoryCatalogValidationError::DuplicateAlias {
                alias: "Meeus-style truncated lunar baseline"
            })
        ));
    }

    #[test]
    fn lunar_theory_catalog_validation_rejects_alias_collisions_with_core_labels() {
        let colliding_alias_catalog = [LunarTheoryCatalogEntry {
            selected: true,
            specification: LunarTheorySpecification {
                source_aliases: &["meeus-style-truncated-lunar-baseline"],
                ..LUNAR_THEORY_SPECIFICATION
            },
        }];

        assert!(matches!(
            validate_lunar_theory_catalog_entries(&colliding_alias_catalog),
            Err(LunarTheoryCatalogValidationError::DuplicateAlias {
                alias: "meeus-style-truncated-lunar-baseline"
            })
        ));
    }

    #[test]
    fn metadata_mentions_the_selected_lunar_theory() {
        let metadata = ElpBackend::new().metadata();
        let theory = lunar_theory_specification();

        let source = theory.source_selection();
        assert_eq!(lunar_theory_source_selection(), source);
        assert!(metadata.provenance.summary.contains(theory.model_name));
        assert!(metadata.provenance.summary.contains(source.identifier));
        assert!(metadata
            .provenance
            .summary
            .contains(theory.source_family.label()));
        assert_eq!(theory.source_family, lunar_theory_source_family());
        assert_eq!(
            theory.source_family.to_string(),
            theory.source_family.label()
        );
        assert_eq!(source.family, theory.source_family);
        assert_eq!(source.source_aliases, theory.source_aliases);
        assert_eq!(source.identifier, theory.source_identifier);
        assert_eq!(source.citation, theory.source_citation);
        assert_eq!(source.material, theory.source_material);
        assert_eq!(source.redistribution_note, theory.redistribution_note);
        assert_eq!(source.license_note, theory.license_note);
        assert_eq!(source.family_label(), theory.source_family.label());
        assert!(metadata.provenance.summary.contains(source.citation));
        assert!(metadata
            .provenance
            .summary
            .contains("true apogee/perigee unsupported"));
        assert!(metadata.provenance.data_sources.iter().any(
            |source| source.contains("Published lunar position, node, and mean-point formulas")
        ));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains(theory.truncation_note)));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains(theory.unit_note)));
        assert_eq!(
            metadata.supported_time_scales,
            vec![TimeScale::Tt, TimeScale::Tdb]
        );
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains(theory.source_identifier)));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains(theory.source_citation)));
        assert!(metadata.provenance.data_sources.iter().any(
            |source| source.contains("No external coefficient-file redistribution constraints")
        ));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("pure Rust")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("J2000 lunar-point anchors")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("2021-03-05 mean-perigee example")));
    }

    #[test]
    fn backend_supports_the_moon_and_lunar_nodes() {
        let backend = ElpBackend::new();
        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(!backend.supports_body(CelestialBody::Sun));
    }

    #[test]
    fn published_moon_example_matches_reference() {
        let backend = ElpBackend::new();
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2_448_724.5),
            TimeScale::Tt,
        );
        let result = backend
            .position(&mean_request_at(CelestialBody::Moon, instant))
            .expect("moon query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let motion = result.motion.expect("motion should be populated");

        assert!((ecliptic.longitude.degrees() - 133.162_655).abs() < 1e-6);
        assert!((ecliptic.latitude.degrees() - -3.229_126).abs() < 1e-6);
        assert!(
            (ecliptic.distance_au.expect("moon distance should exist") * 149_597_870.700
                - 368_409.7)
                .abs()
                < 0.5
        );
        assert!(motion
            .longitude_deg_per_day
            .expect("longitude speed should exist")
            .is_finite());
        assert!(motion
            .latitude_deg_per_day
            .expect("latitude speed should exist")
            .is_finite());
        assert!(motion
            .distance_au_per_day
            .expect("distance speed should exist")
            .is_finite());
        assert_eq!(result.quality, QualityAnnotation::Approximate);
    }

    #[test]
    fn published_true_node_example_matches_reference() {
        let backend = ElpBackend::new();
        let instant = Instant::new(
            pleiades_types::JulianDay::from_days(2_419_914.5),
            TimeScale::Tt,
        );
        let result = backend
            .position(&mean_request_at(CelestialBody::TrueNode, instant))
            .expect("true node query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        let motion = result.motion.expect("motion should be populated");

        assert!((ecliptic.longitude.degrees() - 0.876_3).abs() < 1e-4);
        assert_eq!(ecliptic.latitude.degrees(), 0.0);
        assert_eq!(ecliptic.distance_au, None);
        assert!(motion
            .longitude_deg_per_day
            .expect("longitude speed should exist")
            .is_finite());
        assert_eq!(motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(motion.distance_au_per_day, None);
        assert_eq!(result.quality, QualityAnnotation::Approximate);
    }

    #[test]
    fn moon_samples_remain_finite_across_high_curvature_window() {
        let backend = ElpBackend::new();
        let instants = [J2000 - 1.0, J2000, J2000 + 1.0, J2000 + 2.0]
            .map(|days| Instant::new(pleiades_types::JulianDay::from_days(days), TimeScale::Tt));

        let mut previous_longitude: Option<f64> = None;
        let mut previous_distance: Option<f64> = None;

        for instant in instants {
            let result = backend
                .position(&mean_request_at(CelestialBody::Moon, instant))
                .expect("moon query should work");
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let motion = result.motion.expect("motion should be populated");

            assert!(ecliptic.longitude.degrees().is_finite());
            assert!(ecliptic.latitude.degrees().is_finite());
            assert!(ecliptic
                .distance_au
                .expect("moon distance should exist")
                .is_finite());
            assert!(motion
                .longitude_deg_per_day
                .expect("longitude speed should exist")
                .is_finite());
            assert!(motion
                .latitude_deg_per_day
                .expect("latitude speed should exist")
                .is_finite());
            assert!(motion
                .distance_au_per_day
                .expect("distance speed should exist")
                .is_finite());
            assert!(motion.longitude_deg_per_day.unwrap().abs() < 20.0);
            assert!(motion.latitude_deg_per_day.unwrap().abs() < 10.0);

            if let Some(previous_longitude) = previous_longitude {
                let delta = signed_longitude_delta_degrees(
                    previous_longitude,
                    ecliptic.longitude.degrees(),
                );
                assert!(delta.abs() > 1.0);
                assert!(delta.abs() < 20.0);
            }

            if let Some(previous_distance) = previous_distance {
                assert!(
                    (ecliptic.distance_au.expect("moon distance should exist") - previous_distance)
                        .abs()
                        < 0.02
                );
            }

            previous_longitude = Some(ecliptic.longitude.degrees());
            previous_distance = ecliptic.distance_au;
        }

        assert!(previous_longitude.is_some());
        assert!(previous_distance.is_some());
    }

    #[test]
    fn lunar_high_curvature_continuity_evidence_is_rendered() {
        let report = lunar_high_curvature_continuity_evidence_for_report();

        assert!(
            report.contains("lunar high-curvature continuity evidence: 4 samples across 1 bodies")
        );
        assert!(report.contains("epoch range JD 2451544.0..2451547.0"));
        assert!(report.contains("max adjacent Δlon="));
        assert!(report.contains("max adjacent Δlat="));
        assert!(report.contains("max adjacent Δdist="));
        assert!(report.contains("within regression limits=true"));
        assert!(report.contains("Δlon≤20.0°"));
        assert!(report.contains("Δlat≤10.0°"));
        assert!(report.contains("Δdist≤0.02 AU"));
    }

    #[test]
    fn lunar_high_curvature_equatorial_continuity_evidence_is_rendered() {
        let report = lunar_high_curvature_equatorial_continuity_evidence_for_report();

        assert!(report.contains(
            "lunar high-curvature equatorial continuity evidence: 4 samples across 1 bodies"
        ));
        assert!(report.contains("epoch range JD 2451544.0..2451547.0"));
        assert!(report.contains("max adjacent ΔRA="));
        assert!(report.contains("max adjacent ΔDec="));
        assert!(report.contains("max adjacent Δdist="));
        assert!(report.contains("within regression limits=true"));
        assert!(report.contains("ΔRA≤20.0°"));
        assert!(report.contains("ΔDec≤10.0°"));
        assert!(report.contains("Δdist≤0.02 AU"));
    }

    #[test]
    fn j2000_mean_and_true_nodes_are_available() {
        let backend = ElpBackend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

        let mean = backend
            .position(&mean_request_at(CelestialBody::MeanNode, instant))
            .expect("mean node query should work");
        let mean_ecliptic = mean.ecliptic.expect("mean node ecliptic should exist");
        assert!((mean_ecliptic.longitude.degrees() - 125.044_547_9).abs() < 1e-9);
        assert_eq!(mean_ecliptic.latitude.degrees(), 0.0);
        assert!(mean.equatorial.is_some());
        let mean_motion = mean.motion.expect("mean node motion should be populated");
        assert!(mean_motion
            .longitude_deg_per_day
            .expect("mean node longitude speed should exist")
            .is_finite());
        assert_eq!(mean_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(mean_motion.distance_au_per_day, None);

        let true_node = backend
            .position(&mean_request_at(CelestialBody::TrueNode, instant))
            .expect("true node query should work");
        let true_ecliptic = true_node.ecliptic.expect("true node ecliptic should exist");
        assert!((true_ecliptic.longitude.degrees() - 123.926_171_368_400_46).abs() < 1e-9);
        assert_eq!(true_ecliptic.latitude.degrees(), 0.0);
        assert!(true_node.equatorial.is_some());
        let true_motion = true_node
            .motion
            .expect("true node motion should be populated");
        assert!(true_motion
            .longitude_deg_per_day
            .expect("true node longitude speed should exist")
            .is_finite());
        assert_eq!(true_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(true_motion.distance_au_per_day, None);
    }

    #[test]
    fn j2000_mean_apogee_and_perigee_are_available() {
        let backend = ElpBackend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);

        let perigee = backend
            .position(&mean_request_at(CelestialBody::MeanPerigee, instant))
            .expect("mean perigee query should work");
        let perigee_ecliptic = perigee
            .ecliptic
            .expect("mean perigee ecliptic should exist");
        assert!((perigee_ecliptic.longitude.degrees() - 83.353_246_5).abs() < 1e-9);
        assert_eq!(perigee_ecliptic.latitude.degrees(), 0.0);
        assert_eq!(perigee_ecliptic.distance_au, None);
        assert!(perigee.equatorial.is_some());
        let perigee_motion = perigee
            .motion
            .expect("mean perigee motion should be populated");
        assert!(perigee_motion
            .longitude_deg_per_day
            .expect("mean perigee longitude speed should exist")
            .is_finite());
        assert_eq!(perigee_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(perigee_motion.distance_au_per_day, None);

        let apogee = backend
            .position(&mean_request_at(CelestialBody::MeanApogee, instant))
            .expect("mean apogee query should work");
        let apogee_ecliptic = apogee.ecliptic.expect("mean apogee ecliptic should exist");
        assert!((apogee_ecliptic.longitude.degrees() - 263.353_246_5).abs() < 1e-9);
        assert_eq!(apogee_ecliptic.latitude.degrees(), 0.0);
        assert_eq!(apogee_ecliptic.distance_au, None);
        assert!(apogee.equatorial.is_some());
        let apogee_motion = apogee
            .motion
            .expect("mean apogee motion should be populated");
        assert!(apogee_motion
            .longitude_deg_per_day
            .expect("mean apogee longitude speed should exist")
            .is_finite());
        assert_eq!(apogee_motion.latitude_deg_per_day, Some(0.0));
        assert_eq!(apogee_motion.distance_au_per_day, None);
    }

    #[test]
    fn batch_query_preserves_lunar_reference_order_and_values() {
        let backend = ElpBackend::new();
        let evidence = lunar_reference_evidence();
        let requests = evidence
            .iter()
            .map(|sample| mean_request_at(sample.body.clone(), sample.epoch))
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the lunar reference order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let longitude_tolerance = match sample.body {
                CelestialBody::MeanNode => 1e-1,
                _ => 1e-4,
            };
            assert!(
                (ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance
            );
            assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
            assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
            if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
                assert!((actual - expected).abs() < 1e-8);
            }
        }
    }

    #[test]
    fn batch_query_preserves_lunar_reference_order_for_tdb_requests() {
        let backend = ElpBackend::new();
        let evidence = lunar_reference_evidence();
        let requests = evidence
            .iter()
            .map(|sample| {
                let mut request = mean_request_at(sample.body.clone(), sample.epoch);
                request.instant.scale = TimeScale::Tdb;
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch TDB query should preserve the lunar reference order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant.scale, TimeScale::Tdb);
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let longitude_tolerance = match sample.body {
                CelestialBody::MeanNode => 1e-1,
                _ => 1e-4,
            };
            assert!(
                (ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance
            );
            assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
            assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
            if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
                assert!((actual - expected).abs() < 1e-8);
            }
        }
    }

    #[test]
    fn batch_query_rejects_unsupported_lunar_bodies_with_structured_errors() {
        let backend = ElpBackend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
        let requests = [CelestialBody::TrueApogee, CelestialBody::TruePerigee]
            .into_iter()
            .map(|body| mean_request_at(body, instant))
            .collect::<Vec<_>>();

        let error = backend
            .positions(&requests)
            .expect_err("unsupported lunar bodies should fail explicitly through batch requests");

        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    }

    #[test]
    fn batch_query_preserves_equatorial_frame_and_values() {
        let backend = ElpBackend::new();
        let evidence = lunar_reference_evidence();
        let requests = evidence
            .iter()
            .map(|sample| {
                let mut request = mean_request_at(sample.body.clone(), sample.epoch);
                request.frame = CoordinateFrame::Equatorial;
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch equatorial query should preserve the lunar reference order");

        assert_eq!(results.len(), evidence.len());
        for (sample, result) in evidence.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);

            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let expected = ecliptic.to_equatorial(Angle::from_degrees(
                ElpBackend::mean_obliquity_degrees(sample.epoch),
            ));
            let equatorial = result.equatorial.expect("equatorial result should exist");

            assert_eq!(equatorial, expected);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn backend_supports_lunar_points() {
        let backend = ElpBackend::new();
        let theory = lunar_theory_specification();

        assert_eq!(
            theory.model_name,
            "Compact Meeus-style truncated lunar baseline"
        );
        assert_eq!(
            theory.source_identifier,
            "meeus-style-truncated-lunar-baseline"
        );
        assert_eq!(
            theory.source_citation,
            "Jean Meeus, Astronomical Algorithms, 2nd edition, truncated lunar position and lunar node/perigee/apogee formulae adapted into a compact pure-Rust baseline"
        );
        assert!(theory
            .source_material
            .contains("Published lunar position, node, and mean-point formulas"));
        assert!(theory
            .redistribution_note
            .contains("No external coefficient-file redistribution constraints"));
        assert!(theory.license_note.contains("handwritten pure Rust"));
        assert!(theory.truncation_note.contains("truncated"));
        assert!(theory.unit_note.contains("astronomical units"));
        assert!(theory.date_range_note.contains("1992-04-12"));
        assert!(theory.date_range_note.contains("J2000 lunar-point anchors"));
        assert!(theory
            .date_range_note
            .contains("2021-03-05 mean-perigee example"));
        assert!(theory.date_range_note.contains("1913-05-27 true-node"));
        assert!(theory.frame_note.contains("mean-obliquity"));
        assert_eq!(
            theory.validation_window,
            TimeRange::new(
                Some(Instant::new(
                    pleiades_types::JulianDay::from_days(2_448_724.5),
                    TimeScale::Tt,
                )),
                Some(Instant::new(
                    pleiades_types::JulianDay::from_days(2_459_278.5),
                    TimeScale::Tt,
                )),
            )
        );
        let capability = lunar_theory_capability_summary();
        assert_eq!(capability.model_name, theory.model_name);
        assert_eq!(capability.source_identifier, theory.source_identifier);
        assert_eq!(
            capability.source_family_label,
            lunar_theory_source_family().label()
        );
        assert_eq!(capability.supported_bodies, theory.supported_bodies);
        assert_eq!(capability.unsupported_bodies, theory.unsupported_bodies);
        assert_eq!(
            capability.supported_body_count,
            theory.supported_bodies.len()
        );
        assert_eq!(
            capability.unsupported_body_count,
            theory.unsupported_bodies.len()
        );
        assert_eq!(
            capability.supported_frame_count,
            theory.supported_frames.len()
        );
        assert_eq!(
            capability.supported_time_scale_count,
            theory.supported_time_scales.len()
        );
        assert_eq!(
            capability.supported_zodiac_mode_count,
            theory.supported_zodiac_modes.len()
        );
        assert_eq!(
            capability.supported_apparentness_count,
            theory.supported_apparentness.len()
        );
        assert_eq!(
            capability.supports_topocentric_observer,
            theory.request_policy.supports_topocentric_observer
        );
        assert_eq!(capability.validation_window, theory.validation_window);
        assert!(format_lunar_theory_capability_summary(&capability).contains("bodies=5"));
        assert!(format_lunar_theory_capability_summary(&capability)
            .contains("Moon, Mean Node, True Node, Mean Perigee, Mean Apogee"));
        assert!(format_lunar_theory_capability_summary(&capability).contains("unsupported=2"));
        assert!(format_lunar_theory_capability_summary(&capability)
            .contains("True Apogee, True Perigee"));
        assert!(format_lunar_theory_capability_summary(&capability).contains("frames=2"));
        assert!(format_lunar_theory_capability_summary(&capability).contains("time scales=2"));
        assert!(format_lunar_theory_capability_summary(&capability).contains("zodiac modes=1"));
        assert!(format_lunar_theory_capability_summary(&capability).contains("apparentness=1"));
        assert!(format_lunar_theory_capability_summary(&capability)
            .contains("topocentric observer=false"));
        assert!(format_lunar_theory_capability_summary(&capability)
            .contains("validation window=JD 2448724.5 (TT) → JD 2459278.5 (TT)"));
        assert!(
            format_lunar_theory_capability_summary(&capability).contains("catalog validation=ok")
        );
        assert_eq!(theory.supported_bodies, lunar_theory_supported_bodies());
        assert_eq!(theory.unsupported_bodies, lunar_theory_unsupported_bodies());
        assert_eq!(
            theory.supported_bodies,
            &[
                CelestialBody::Moon,
                CelestialBody::MeanNode,
                CelestialBody::TrueNode,
                CelestialBody::MeanPerigee,
                CelestialBody::MeanApogee,
            ]
        );
        assert_eq!(
            theory.unsupported_bodies,
            &[CelestialBody::TrueApogee, CelestialBody::TruePerigee]
        );
        assert_eq!(
            theory.request_policy.supported_frames,
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
        );
        assert_eq!(
            theory.request_policy.supported_time_scales,
            &[TimeScale::Tt, TimeScale::Tdb]
        );
        assert_eq!(
            theory.request_policy.supported_zodiac_modes,
            &[ZodiacMode::Tropical]
        );
        assert_eq!(
            theory.request_policy.supported_apparentness,
            &[Apparentness::Mean]
        );
        assert!(!theory.request_policy.supports_topocentric_observer);
        assert!(theory
            .license_note
            .contains("future source-backed lunar theory selection"));

        assert!(backend.supports_body(CelestialBody::Moon));
        assert!(backend.supports_body(CelestialBody::MeanNode));
        assert!(backend.supports_body(CelestialBody::TrueNode));
        assert!(backend.supports_body(CelestialBody::MeanApogee));
        assert!(backend.supports_body(CelestialBody::MeanPerigee));
        assert!(!backend.supports_body(CelestialBody::TrueApogee));
        assert!(!backend.supports_body(CelestialBody::TruePerigee));
        assert!(!backend.supports_body(CelestialBody::Sun));
        assert_eq!(
            backend.metadata().body_coverage,
            lunar_theory_supported_bodies()
        );

        let evidence = lunar_reference_evidence();
        assert_eq!(evidence.len(), 9);
        assert_eq!(evidence[0].body, CelestialBody::Moon);
        assert_eq!(evidence[0].epoch.julian_day.days(), 2_448_724.5);
        assert_eq!(evidence[1].body, CelestialBody::MeanNode);
        assert_eq!(evidence[1].epoch.julian_day.days(), J2000);
        assert_eq!(evidence[2].body, CelestialBody::TrueNode);
        assert_eq!(evidence[2].epoch.julian_day.days(), J2000);
        assert_eq!(evidence[3].body, CelestialBody::MeanNode);
        assert_eq!(evidence[3].epoch.julian_day.days(), 2_419_914.5);
        assert_eq!(evidence[4].body, CelestialBody::MeanNode);
        assert_eq!(evidence[4].epoch.julian_day.days(), 2_436_909.5);
        assert_eq!(evidence[5].body, CelestialBody::MeanPerigee);
        assert_eq!(evidence[5].epoch.julian_day.days(), 2_459_278.5);
        assert_eq!(evidence[6].body, CelestialBody::MeanPerigee);
        assert_eq!(evidence[6].epoch.julian_day.days(), J2000);
        assert_eq!(evidence[7].body, CelestialBody::MeanApogee);
        assert_eq!(evidence[7].epoch.julian_day.days(), J2000);
        assert_eq!(evidence[8].body, CelestialBody::TrueNode);
        assert_eq!(evidence[8].epoch.julian_day.days(), 2_419_914.5);
        for body in theory.supported_bodies {
            assert!(evidence.iter().any(|sample| sample.body == *body));
        }

        for sample in evidence {
            let result = backend
                .position(&mean_request_at(sample.body.clone(), sample.epoch))
                .expect("lunar reference sample should be computable");
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let longitude_tolerance = match sample.body {
                CelestialBody::MeanNode => 1e-1,
                _ => 1e-4,
            };
            assert!(
                (ecliptic.longitude.degrees() - sample.longitude_deg).abs() < longitude_tolerance
            );
            assert!((ecliptic.latitude.degrees() - sample.latitude_deg).abs() < 1e-4);
            assert_eq!(ecliptic.distance_au.is_some(), sample.distance_au.is_some());
            if let (Some(actual), Some(expected)) = (ecliptic.distance_au, sample.distance_au) {
                assert!((actual - expected).abs() < 1e-8);
            }
        }

        let instant = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
        for body in [
            CelestialBody::TrueApogee,
            CelestialBody::TruePerigee,
            CelestialBody::Sun,
        ] {
            let error = backend
                .position(&mean_request_at(body, instant))
                .expect_err("unsupported lunar bodies should fail explicitly");
            assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        }
    }

    #[test]
    fn lunar_reference_evidence_summary_matches_the_canonical_slice() {
        let summary = lunar_reference_evidence_summary().expect("reference evidence should exist");

        assert_eq!(summary.sample_count, 9);
        assert_eq!(summary.body_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_419_914.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_459_278.5);
        assert!(lunar_reference_evidence_summary_for_report().contains("9 samples across 5 bodies"));
        assert!(lunar_reference_evidence_summary_for_report().contains("JD 2419914.5..2459278.5"));

        let envelope = lunar_reference_evidence_envelope().expect("error envelope should exist");
        assert_eq!(envelope.sample_count, summary.sample_count);
        assert_eq!(envelope.body_count, summary.body_count);
        assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
        assert_eq!(envelope.latest_epoch, summary.latest_epoch);
        assert!(envelope.max_longitude_delta_deg.is_finite());
        assert!(envelope.mean_longitude_delta_deg.is_finite());
        assert!(envelope.max_latitude_delta_deg.is_finite());
        assert!(envelope.mean_latitude_delta_deg.is_finite());
        assert_eq!(envelope.outside_current_limits_count, 0);
        assert!(envelope.within_current_limits);
        assert!(lunar_reference_evidence_envelope_for_report()
            .contains("lunar reference error envelope"));
        assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("outliers=none"));
        assert!(lunar_reference_evidence_envelope_for_report().contains("limits: Δlon≤1e-4°"));
        assert!(
            lunar_reference_evidence_envelope_for_report().contains("within current limits=true")
        );
        assert!(lunar_reference_evidence_envelope_for_report().contains("outside current limits=0"));
    }

    #[test]
    fn lunar_equatorial_reference_evidence_matches_the_canonical_slice() {
        let summary = lunar_equatorial_reference_evidence_summary()
            .expect("equatorial reference evidence should exist");

        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.body_count, 1);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_448_724.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_448_724.5);
        assert!(lunar_equatorial_reference_evidence_summary_for_report()
            .contains("1 samples across 1 bodies"));
        assert!(lunar_equatorial_reference_evidence_summary_for_report()
            .contains("JD 2448724.5..2448724.5"));

        let envelope = lunar_equatorial_reference_evidence_envelope()
            .expect("equatorial error envelope should exist");
        assert_eq!(envelope.sample_count, summary.sample_count);
        assert_eq!(envelope.body_count, summary.body_count);
        assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
        assert_eq!(envelope.latest_epoch, summary.latest_epoch);
        assert!(envelope.max_right_ascension_delta_deg.is_finite());
        assert!(envelope.mean_right_ascension_delta_deg.is_finite());
        assert!(envelope.max_declination_delta_deg.is_finite());
        assert!(envelope.mean_declination_delta_deg.is_finite());
        assert_eq!(envelope.outside_current_limits_count, 0);
        assert!(envelope.within_current_limits);
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("lunar equatorial reference error envelope"));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔDec="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔDec="));
        assert!(
            lunar_equatorial_reference_evidence_envelope_for_report().contains("limits: ΔRA≤1e-2°")
        );
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("within current limits=true"));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("outside current limits=0"));

        let sample = &lunar_equatorial_reference_evidence()[0];
        assert_eq!(sample.body, CelestialBody::Moon);
        assert_eq!(sample.epoch.julian_day.days(), 2_448_724.5);
        let result = ElpBackend::new()
            .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
            .expect("equatorial reference sample should remain computable");
        let equatorial = result
            .equatorial
            .expect("equatorial reference sample should include equatorial coordinates");
        assert!(
            (equatorial.right_ascension.degrees() - sample.equatorial.right_ascension.degrees())
                .abs()
                < 1e-2
        );
        assert!(
            (equatorial.declination.degrees() - sample.equatorial.declination.degrees()).abs()
                < 1e-2
        );
        assert_eq!(
            equatorial.distance_au.is_some(),
            sample.equatorial.distance_au.is_some()
        );
        if let (Some(actual), Some(expected)) =
            (equatorial.distance_au, sample.equatorial.distance_au)
        {
            assert!((actual - expected).abs() < 1e-8);
        }
    }

    #[test]
    fn lunar_apparent_comparison_evidence_documents_the_mean_gap() {
        let summary =
            lunar_apparent_comparison_summary().expect("apparent comparison evidence should exist");

        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.body_count, 1);
        assert!((summary.earliest_epoch.julian_day.days() - 2_440_214.916_7).abs() < 1e-9);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_448_724.5);
        assert!(summary.max_ecliptic_longitude_delta_deg.is_finite());
        assert!(summary.max_ecliptic_latitude_delta_deg.is_finite());
        assert!(summary.max_ecliptic_distance_delta_au.is_finite());
        assert!(summary.max_right_ascension_delta_deg.is_finite());
        assert!(summary.max_declination_delta_deg.is_finite());
        let known_epochs = [2_440_214.916_7, 2_448_724.5];
        assert!(known_epochs.contains(&summary.max_ecliptic_longitude_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_ecliptic_latitude_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_ecliptic_distance_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_right_ascension_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_declination_epoch.julian_day.days()));
        assert!(lunar_apparent_comparison_summary_for_report().contains(
            "lunar apparent comparison evidence: 2 reference-only samples across 1 bodies"
        ));
        assert!(lunar_apparent_comparison_summary_for_report()
            .contains("mean-only gap against the published apparent Moon examples"));
        assert!(lunar_apparent_comparison_summary_for_report().contains("@ JD"));
        assert!(lunar_apparent_comparison_summary_for_report()
            .contains("apparent requests remain unsupported"));

        let samples = lunar_apparent_comparison_evidence();
        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].body, CelestialBody::Moon);
        assert_eq!(samples[0].epoch.julian_day.days(), 2_448_724.5);
        assert!(samples[0]
            .note
            .contains("reference-only mean/apparent comparison datum"));
        assert_eq!(samples[1].body, CelestialBody::Moon);
        assert_eq!(samples[1].epoch.julian_day.days(), 2_440_214.916_7);
        assert!(samples[1]
            .note
            .contains("second reference-only mean/apparent comparison datum"));
    }

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = ElpBackend::new();
        let mut request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        request.apparent = Apparentness::Apparent;

        let error = backend
            .position(&request)
            .expect_err("apparent requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    }

    #[test]
    fn tdb_requests_are_accepted_like_tt_requests() {
        let backend = ElpBackend::new();
        let tt_request = mean_request(CelestialBody::Moon);
        let tdb_request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        );

        let tt_result = backend
            .position(&tt_request)
            .expect("TT request should be supported");
        let tdb_result = backend
            .position(&tdb_request)
            .expect("TDB request should be supported");

        assert_eq!(tt_result.body, tdb_result.body);
        assert_eq!(tt_result.instant.scale, TimeScale::Tt);
        assert_eq!(tdb_result.instant.scale, TimeScale::Tdb);
        assert_eq!(tt_result.ecliptic, tdb_result.ecliptic);
        assert_eq!(tt_result.equatorial, tdb_result.equatorial);
        assert_eq!(tt_result.motion, tdb_result.motion);
    }

    #[test]
    fn topocentric_requests_are_rejected_explicitly() {
        let backend = ElpBackend::new();
        let mut request = mean_request(CelestialBody::Moon);
        request.observer = Some(pleiades_types::ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(0.0),
            None,
        ));

        let error = backend
            .position(&request)
            .expect_err("topocentric requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidObserver);
    }

    fn mean_request(body: CelestialBody) -> EphemerisRequest {
        mean_request_at(
            body,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        )
    }

    fn mean_request_at(body: CelestialBody, instant: Instant) -> EphemerisRequest {
        let mut request = EphemerisRequest::new(body, instant);
        request.apparent = Apparentness::Mean;
        request
    }
}
