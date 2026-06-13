//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The first release intentionally keeps a compact Moon-and-lunar-point
//! backend for chart workflows by combining a compact Meeus-style truncated
//! lunar position series with geocentric coordinate transforms, Meeus-style
//! mean node/perigee/apogee formulae, and finite-difference mean-motion
//! estimates. The backend accepts both TT and TDB requests as dynamical-time
//! inputs and still rejects UT-based requests explicitly.
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
    Apparentness, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    FrameTreatmentSummary,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, EquatorialCoordinates, Instant, Latitude, TimeRange,
    TimeScale, ZodiacMode,
};

mod backend;
pub(crate) mod data;
pub(crate) mod series;

pub use backend::ElpBackend;

use series::signed_longitude_delta_degrees;

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

/// Validation error for a lunar-theory request policy that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryRequestPolicyValidationError {
    /// A rendered policy field no longer matches the current lunar-theory selection.
    FieldOutOfSync {
        /// Field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheoryRequestPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar theory request policy field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheoryRequestPolicyValidationError {}

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

    /// Returns `Ok(())` when the policy still matches the current lunar-theory selection.
    pub fn validate(&self) -> Result<(), LunarTheoryRequestPolicyValidationError> {
        let theory = lunar_theory_specification();

        if self.supported_frames != theory.request_policy.supported_frames {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_frames",
            });
        }
        if self.supported_time_scales != theory.request_policy.supported_time_scales {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_time_scales",
            });
        }
        if self.supported_zodiac_modes != theory.request_policy.supported_zodiac_modes {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_zodiac_modes",
            });
        }
        if self.supported_apparentness != theory.request_policy.supported_apparentness {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supported_apparentness",
            });
        }
        if self.supports_topocentric_observer != theory.request_policy.supports_topocentric_observer
        {
            return Err(LunarTheoryRequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer",
            });
        }

        Ok(())
    }
}

impl fmt::Display for LunarTheoryRequestPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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

impl LunarTheorySourceSelection {
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
        if resolve_lunar_theory_by_key(self.catalog_key()) != Some(lunar_theory_specification()) {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "catalog_key",
            });
        }
        if resolve_lunar_theory_by_key(self.family_key()) != Some(lunar_theory_specification()) {
            return Err(LunarTheorySourceSelectionValidationError::FieldOutOfSync {
                field: "family_key",
            });
        }

        Ok(())
    }
}

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

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_lunar_theory_specification(self)
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

    /// Returns `Ok(())` when the specification still matches the current selection.
    pub fn validate(&self) -> Result<(), LunarTheorySpecificationValidationError> {
        let source = self.source_selection();

        if self.source_family != source.family {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "source_family",
            });
        }
        if self.model_name != LUNAR_THEORY_SPECIFICATION.model_name {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "model_name",
            });
        }
        if self.source_aliases != source.source_aliases {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "source_aliases",
            });
        }
        if self.source_identifier != source.identifier {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "source_identifier",
            });
        }
        if self.source_citation != source.citation {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "source_citation",
            });
        }
        if self.source_material != source.material {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "source_material",
            });
        }
        if self.redistribution_note != source.redistribution_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "redistribution_note",
            });
        }
        if self.license_note != source.license_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "license_note",
            });
        }
        if self.request_policy.supported_frames != self.supported_frames {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "request_policy.supported_frames",
            });
        }
        if self.request_policy.supported_time_scales != self.supported_time_scales {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "request_policy.supported_time_scales",
            });
        }
        if self.request_policy.supported_zodiac_modes != self.supported_zodiac_modes {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "request_policy.supported_zodiac_modes",
            });
        }
        if self.request_policy.supported_apparentness != self.supported_apparentness {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "request_policy.supported_apparentness",
            });
        }
        if self.request_policy.supports_topocentric_observer != self.supports_topocentric_observer {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "request_policy.supports_topocentric_observer",
            });
        }
        if self.truncation_note != LUNAR_THEORY_SPECIFICATION.truncation_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "truncation_note",
            });
        }
        if self.unit_note != LUNAR_THEORY_SPECIFICATION.unit_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "unit_note",
            });
        }
        if self.date_range_note != LUNAR_THEORY_SPECIFICATION.date_range_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "date_range_note",
            });
        }
        if self.frame_note != LUNAR_THEORY_SPECIFICATION.frame_note {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "frame_note",
            });
        }
        if self.validation_window != LUNAR_THEORY_SPECIFICATION.validation_window {
            return Err(LunarTheorySpecificationValidationError::FieldOutOfSync {
                field: "validation_window",
            });
        }
        Ok(())
    }
}

/// Validation errors for the structured lunar-theory specification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheorySpecificationValidationError {
    /// A rendered specification field no longer matches the current selection.
    FieldOutOfSync {
        /// Field that drifted out of sync.
        field: &'static str,
    },
}

impl fmt::Display for LunarTheorySpecificationValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar theory specification field `{field}` is out of sync with the current selection"
            ),
        }
    }
}

impl std::error::Error for LunarTheorySpecificationValidationError {}

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

const LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS: [Instant; 6] = [
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 - 1.0),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 - 0.5),
        TimeScale::Tt,
    ),
    Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
    Instant::new(
        pleiades_types::JulianDay::from_days(J2000 + 0.5),
        TimeScale::Tt,
    ),
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
        "Published lunar position, node, and mean-point formulas implemented as the current pure-Rust baseline; no ELP coefficient files are bundled in the baseline",
    source_aliases: LUNAR_THEORY_SOURCE_ALIASES,
    redistribution_note:
        "No external coefficient-file redistribution constraints apply because the baseline does not bundle ELP coefficient tables",
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
        "Validated against the published 1992-04-12 geocentric Moon example, the published 1992-04-12 geocentric Moon RA/Dec example used for the mean-obliquity equatorial transform, the reference-only published 1968-12-24 apparent geocentric Moon comparison datum, the reference-only published 2004-04-01 NASA RP 1349 apparent Moon table row, the reference-only published 2006-09-07 EclipseWise apparent Moon coordinate row, J2000 lunar-point anchors including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example; no full ELP coefficient range has been published yet",
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

/// Formats the compact validation summary for release-facing reporting.
pub fn format_lunar_theory_catalog_validation_summary(
    summary: &LunarTheoryCatalogValidationSummary,
) -> String {
    summary.summary_line()
}

/// Returns a compact release-facing summary of the lunar-theory catalog validation state.
pub fn lunar_theory_catalog_validation_summary_for_report() -> String {
    lunar_theory_catalog_validation_summary().summary_line()
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

fn format_validated_lunar_theory_source_selection_for_report(
    selection: &LunarTheorySourceSelection,
) -> String {
    match selection.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source selection: unavailable ({error})"),
    }
}

/// Returns the compact source-selection summary for the current lunar baseline.
///
/// The helper validates the backend-owned source-selection record first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_source_selection_summary() -> String {
    format_validated_lunar_theory_source_selection_for_report(&lunar_theory_source_selection())
}

/// Returns the compact source-selection summary for report-facing consumers.
///
/// This keeps the report-facing naming aligned with the other typed lunar
/// provenance helpers used by `pleiades-validate`.
pub fn lunar_theory_source_selection_summary_for_report() -> String {
    lunar_theory_source_selection_summary()
}

/// Returns the validated compact source-selection summary for report-facing consumers.
pub fn validated_lunar_theory_source_selection_summary_for_report() -> Result<String, String> {
    let selection = lunar_theory_source_selection();
    selection
        .validated_summary_line()
        .map_err(|error| error.to_string())
}

/// Formats a structured lunar source selection for reporting.
///
/// The formatter validates the provided selection against the current lunar-theory
/// baseline first so callers get the same fail-closed summary wording as the
/// report-facing accessor when a drifted selection is reused.
pub fn format_lunar_theory_source_selection(selection: &LunarTheorySourceSelection) -> String {
    format_validated_lunar_theory_source_selection_for_report(selection)
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

fn format_validated_lunar_theory_source_family_summary_for_report(
    summary: &LunarTheorySourceFamilySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("lunar source family: unavailable ({error})"),
    }
}

/// Returns the compact source-family summary for the current lunar baseline.
///
/// The helper validates the backend-owned source-family record first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_source_family_summary_for_report() -> String {
    format_validated_lunar_theory_source_family_summary_for_report(
        &lunar_theory_source_family_summary(),
    )
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
    FieldOutOfSync { field: &'static str },
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

/// Formats the catalog summary for release-facing reporting.
pub fn format_lunar_theory_catalog_summary(summary: &LunarTheoryCatalogSummary) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_catalog_summary_for_report(
    summary: &LunarTheoryCatalogSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar theory catalog: unavailable ({error})"),
    }
}

/// Returns the release-facing catalog summary string for the current lunar-theory selection.
///
/// The report helper validates the backend-owned catalog summary first so any
/// future drift in the rendered selection fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_catalog_summary_for_report() -> String {
    format_validated_lunar_theory_catalog_summary_for_report(&lunar_theory_catalog_summary())
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
    pub validation_window: TimeRange,
    /// Whether the current lunar-theory catalog validates cleanly.
    pub catalog_validation_ok: bool,
}

/// Validation error for a lunar-theory capability summary that drifted from the current selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarTheoryCapabilitySummaryValidationError {
    /// A rendered summary field no longer matches the current lunar-theory selection.
    FieldOutOfSync { field: &'static str },
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
            format_bodies(self.supported_bodies),
            self.unsupported_body_count,
            format_bodies(self.unsupported_bodies),
            self.supported_frame_count,
            self.supported_time_scale_count,
            self.supported_zodiac_mode_count,
            self.supported_apparentness_count,
            self.supports_topocentric_observer,
            self.validation_window,
            if self.catalog_validation_ok { "ok" } else { "error" },
        )
    }
}

impl fmt::Display for LunarTheoryCapabilitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl LunarTheoryCapabilitySummary {
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

/// Formats the capability summary for release-facing reporting.
pub fn format_lunar_theory_capability_summary(summary: &LunarTheoryCapabilitySummary) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_capability_summary_for_report(
    summary: &LunarTheoryCapabilitySummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar capability summary: unavailable ({error})"),
    }
}

/// Returns the release-facing capability summary string for the current lunar-theory selection.
///
/// The report helper validates the backend-owned capability summary first so any
/// future drift in the rendered selection fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_capability_summary_for_report() -> String {
    format_validated_lunar_theory_capability_summary_for_report(&lunar_theory_capability_summary())
}

/// Formats the release-facing one-line summary for a lunar-theory specification.
///
/// Validation and release tooling use this helper so lunar provenance stays
/// owned by the backend crate rather than duplicated in reporting layers.
pub fn format_lunar_theory_specification(theory: &LunarTheorySpecification) -> String {
    let source = theory.source_selection();
    format!(
        "ELP lunar theory specification: {} [{}; family: {}; selected key: {}] ({} supported bodies: {}; {} unsupported bodies: {}); request policy: {}; citation: {}; provenance: {}; redistribution: {}; truncation: {}; units: {}; validation window: {}; date-range note: {}; frame treatment: {}; license: {}",
        theory.model_name,
        source.identifier,
        source.family,
        source.catalog_key(),
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
        theory.validation_window,
        theory.date_range_note,
        theory.frame_note,
        source.license_note,
    )
}

fn format_validated_lunar_theory_specification_for_report(
    theory: &LunarTheorySpecification,
) -> String {
    match theory.validate() {
        Ok(()) => theory.summary_line(),
        Err(error) => format!("ELP lunar theory specification: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar-theory selection.
///
/// The validation helper checks the backend-owned specification first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_summary_for_report() -> String {
    format_validated_lunar_theory_specification_for_report(&lunar_theory_specification())
}

impl fmt::Display for LunarTheorySpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the raw one-line summary for the current lunar-theory selection.
///
/// Validation and release tooling should prefer
/// [`lunar_theory_summary_for_report()`] so the backend-owned specification is
/// validated before the compact provenance line is rendered.
pub fn lunar_theory_summary() -> String {
    lunar_theory_specification().summary_line()
}

impl LunarTheorySourceSummary {
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

/// Formats the compact lunar source-selection summary for release-facing reporting.
pub fn format_lunar_theory_source_summary(summary: &LunarTheorySourceSummary) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_source_summary_for_report(
    summary: &LunarTheorySourceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source selection: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar source selection.
///
/// The report helper validates the backend-owned source summary first so any
/// future drift in the rendered provenance fields shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_source_summary_for_report() -> String {
    format_validated_lunar_theory_source_summary_for_report(&lunar_theory_source_summary())
}

fn format_validated_lunar_theory_request_policy_for_report(
    policy: &LunarTheoryRequestPolicy,
) -> String {
    match policy.validate() {
        Ok(()) => policy.summary_line(),
        Err(error) => format!("lunar theory request policy: unavailable ({error})"),
    }
}

/// Returns the current lunar-theory request policy summary.
///
/// The report helper validates the backend-owned request-policy summary first so
/// any future drift in the supported frames, time scales, zodiac modes,
/// apparentness, or topocentric-observer posture shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_request_policy_summary() -> String {
    format_validated_lunar_theory_request_policy_for_report(&lunar_theory_request_policy())
}

/// Returns the structured lunar-theory frame-treatment summary.
pub fn lunar_theory_frame_treatment_summary_details() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(lunar_theory_specification().frame_note)
}

/// Returns the current lunar-theory frame-treatment summary for reports.
///
/// The report helper validates the backend-owned frame-treatment summary first
/// so any future drift in the mean-obliquity wording shows up as an unavailable
/// report line instead of a silently stale summary.
pub fn lunar_theory_frame_treatment_summary_for_report() -> String {
    let summary = lunar_theory_frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("ELP frame treatment unavailable ({error})"),
    }
}

/// Compact release-facing summary for the current lunar-theory limitations posture.
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
    FieldOutOfSync { field: &'static str },
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
    pub fn summary_line(&self) -> String {
        let reference_envelope = lunar_reference_evidence_envelope_for_report();
        let equatorial_envelope = lunar_equatorial_reference_evidence_envelope_for_report();

        format!(
            "lunar theory limitations: {}; supported bodies: {}; unsupported bodies: {}; release-grade evidence by channel: {}; {}",
            self.baseline_label,
            format_bodies(self.supported_bodies),
            format_bodies(self.unsupported_bodies),
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

/// Formats the compact lunar-theory limitations summary for release-facing reporting.
pub fn format_lunar_theory_limitations_summary(summary: &LunarTheoryLimitationsSummary) -> String {
    summary.summary_line()
}

fn format_validated_lunar_theory_limitations_summary_for_report(
    summary: &LunarTheoryLimitationsSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar theory limitations: unavailable ({error})"),
    }
}

/// Returns the release-facing one-line summary for the current lunar-theory limitations posture.
pub fn lunar_theory_limitations_summary_for_report() -> String {
    format_validated_lunar_theory_limitations_summary_for_report(&lunar_theory_limitations_summary())
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

/// Returns the current lunar-theory frame-treatment summary.
pub fn lunar_theory_frame_treatment_summary() -> &'static str {
    lunar_theory_frame_treatment_summary_details().summary_line()
}

fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    join_display(bodies)
}

fn format_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

fn format_time_scales(scales: &[TimeScale]) -> String {
    join_display(scales)
}

fn format_zodiac_modes(modes: &[ZodiacMode]) -> String {
    join_display(modes)
}

fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

fn format_instant(instant: Instant) -> String {
    format!("JD {:.1} ({})", instant.julian_day.days(), instant.scale)
}

fn format_epoch_range(start: Instant, end: Instant) -> String {
    TimeRange::new(Some(start), Some(end)).summary_line()
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

impl LunarReferenceSample {
    /// Returns `Ok(())` when the sample still represents a valid lunar evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if !matches!(
            self.body,
            CelestialBody::Moon
                | CelestialBody::MeanNode
                | CelestialBody::TrueNode
                | CelestialBody::MeanApogee
                | CelestialBody::MeanPerigee
        ) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use a supported lunar body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use TT epochs",
            ));
        }
        if !self.longitude_deg.is_finite() || !self.latitude_deg.is_finite() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample must use finite ecliptic coordinates",
            ));
        }
        if let Some(distance_au) = self.distance_au {
            if !distance_au.is_finite() || distance_au <= 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "lunar reference sample must use a positive finite distance when present",
                ));
            }
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at JD {:.1}: lon={:.12}°, lat={:.12}°, dist={}, note={}",
            self.body,
            self.epoch.julian_day.days(),
            self.longitude_deg,
            self.latitude_deg,
            self.distance_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }
}

impl fmt::Display for LunarReferenceSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
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

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
pub fn lunar_reference_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_reference_evidence()
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.instant.scale = if index % 2 == 0 {
                TimeScale::Tt
            } else {
                TimeScale::Tdb
            };
            request
        })
        .collect()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_requests() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
}

/// Returns the canonical lunar request corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_reference_batch_requests`].
#[doc(alias = "lunar_reference_requests")]
#[doc(alias = "lunar_reference_batch_requests")]
#[doc(alias = "lunar_reference_batch_request_corpus")]
pub fn lunar_reference_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by lunar batch-parity validation.
///
/// This is a compatibility alias for [`lunar_reference_batch_parity_requests`].
#[doc(alias = "lunar_reference_batch_parity_requests")]
pub fn lunar_reference_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_reference_batch_parity_requests()
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

impl LunarEquatorialReferenceSample {
    /// Returns `Ok(())` when the sample still represents a valid lunar equatorial evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use the Moon body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use TT epochs",
            ));
        }
        if !self.equatorial.right_ascension.degrees().is_finite()
            || !self.equatorial.declination.degrees().is_finite()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample must use finite equatorial coordinates",
            ));
        }
        if let Some(distance_au) = self.equatorial.distance_au {
            if !distance_au.is_finite() || distance_au <= 0.0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "lunar equatorial reference sample must use a positive finite distance when present",
                ));
            }
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "{} at JD {:.1}: ra={:.12}°, dec={:.12}°, dist={}, note={}",
            self.body,
            self.epoch.julian_day.days(),
            self.equatorial.right_ascension.degrees(),
            self.equatorial.declination.degrees(),
            self.equatorial
                .distance_au
                .map(|value| format!("{value:.12} AU"))
                .unwrap_or_else(|| "n/a".to_string()),
            self.note,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the canonical lunar equatorial evidence samples used by validation and reporting.
pub fn lunar_equatorial_reference_evidence() -> &'static [LunarEquatorialReferenceSample] {
    const SAMPLES: &[LunarEquatorialReferenceSample] = &[
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.688_470),
                Latitude::from_degrees(13.768_368),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Published 1992-04-12 geocentric Moon RA/Dec example used to anchor the mean-obliquity equatorial transform",
        },
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.683_861_811_039_18),
                Latitude::from_degrees(13.769_414_994_266_76),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Derived equatorial companion from the published 1992-04-12 geocentric Moon example using the shared mean-obliquity transform",
        },
        LunarEquatorialReferenceSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(pleiades_types::JulianDay::from_days(2_448_724.5), TimeScale::Tt),
            equatorial: EquatorialCoordinates::new(
                Angle::from_degrees(134.688_469),
                Latitude::from_degrees(13.768_367),
                Some(368_409.7 / 149_597_870.700),
            ),
            note: "Reference-only published 1992-04-12 apparent geocentric Moon comparison datum reused as an additional equatorial cross-check",
        },
    ];

    SAMPLES
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
pub fn lunar_equatorial_reference_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_evidence()
        .iter()
        .map(|sample| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.frame = CoordinateFrame::Equatorial;
            request.instant.scale = TimeScale::Tt;
            request.zodiac_mode = ZodiacMode::Tropical;
            request.apparent = Apparentness::Mean;
            request
        })
        .collect()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_requests() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// Returns the canonical equatorial lunar request corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_requests`].
#[doc(alias = "lunar_equatorial_reference_requests")]
#[doc(alias = "lunar_equatorial_reference_batch_requests")]
#[doc(alias = "lunar_equatorial_reference_batch_request_corpus")]
pub fn lunar_equatorial_reference_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_requests()
}

/// Returns the canonical equatorial lunar batch-parity corpus used by validation and reporting.
///
/// This is a compatibility alias for [`lunar_equatorial_reference_batch_parity_requests`].
#[doc(alias = "lunar_equatorial_reference_batch_parity_requests")]
pub fn lunar_equatorial_reference_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_equatorial_reference_batch_parity_requests()
}

/// A compact summary of the canonical lunar equatorial reference batch-parity slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarEquatorialReferenceBatchParitySummary {
    /// Number of evidence samples in the checked-in batch slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the batch slice.
    pub body_count: usize,
    /// Coordinate frame covered by the batch slice.
    pub frame: CoordinateFrame,
    /// Whether the batch results preserved request order.
    pub order_preserved: bool,
    /// Whether batch results matched the corresponding single-query lookups.
    pub single_query_parity: bool,
}

/// Validation error for a lunar equatorial batch-parity summary that drifted from the current reference evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarEquatorialReferenceBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current evidence slice.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarEquatorialReferenceBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar equatorial reference batch parity summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarEquatorialReferenceBatchParitySummaryValidationError {}

/// Returns a compact summary of the canonical lunar equatorial reference batch-parity slice.
pub fn lunar_equatorial_reference_batch_parity_summary(
) -> Option<LunarEquatorialReferenceBatchParitySummary> {
    let requests = lunar_equatorial_reference_batch_requests();
    if requests.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let bodies = requests
        .iter()
        .map(|request| request.body.to_string())
        .collect::<std::collections::BTreeSet<_>>();

    let results = backend.positions(&requests).ok()?;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for (request, result) in requests.iter().zip(results.iter()) {
        order_preserved &= result.body == request.body;

        let single = backend.position(request).ok()?;
        single_query_parity &= result.body == single.body
            && result.instant == single.instant
            && result.frame == single.frame
            && result.quality == single.quality
            && result.ecliptic == single.ecliptic
            && result.equatorial == single.equatorial
            && result.motion == single.motion;
    }

    Some(LunarEquatorialReferenceBatchParitySummary {
        sample_count: requests.len(),
        body_count: bodies.len(),
        frame: CoordinateFrame::Equatorial,
        order_preserved,
        single_query_parity,
    })
}

impl LunarEquatorialReferenceBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(
        &self,
    ) -> Result<(), LunarEquatorialReferenceBatchParitySummaryValidationError> {
        let samples = lunar_equatorial_reference_evidence();
        let expected_sample_count = samples.len();
        let expected_body_count = samples
            .iter()
            .map(|sample| sample.body.to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .len();

        if self.sample_count != expected_sample_count {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected_body_count {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "frame",
                },
            );
        }
        if !self.order_preserved {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "order_preserved",
                },
            );
        }
        if !self.single_query_parity {
            return Err(
                LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "single_query_parity",
                },
            );
        }

        Ok(())
    }

    /// Returns the release-facing one-line lunar equatorial batch parity summary.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity {
            "preserved"
        } else {
            "needs attention"
        };

        format!(
            "lunar equatorial reference batch parity: {} requests across {} bodies, frame={}, order={}, single-query parity={}",
            self.sample_count,
            self.body_count,
            self.frame,
            order,
            parity,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial batch-parity evidence for release-facing reporting.
pub fn format_lunar_equatorial_reference_batch_parity_summary(
    summary: &LunarEquatorialReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial batch-parity summary string.
pub fn lunar_equatorial_reference_batch_parity_summary_for_report() -> String {
    match lunar_equatorial_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar equatorial reference batch parity: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference batch parity: unavailable".to_string(),
    }
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
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
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

impl LunarReferenceEvidenceSummary {
    /// Returns the release-facing one-line lunar reference evidence summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar reference evidence: {} samples across {} bodies, epoch range {}, validated against the published 1992-04-12 Moon example plus J2000 lunar-point anchors, including the mean apogee and mean perigee references, published 1913-05-27 true-node and 1959-12-07 mean-node examples, and a published 2021-03-05 mean-perigee example",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_reference_evidence_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar reference evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar reference evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for LunarReferenceEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar reference evidence summary for release-facing reporting.
pub fn format_lunar_reference_evidence_summary(summary: &LunarReferenceEvidenceSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar reference evidence summary string.
pub fn lunar_reference_evidence_summary_for_report() -> String {
    match lunar_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_evidence_summary(&summary),
            Err(error) => format!("lunar reference evidence: unavailable ({error})"),
        },
        None => "lunar reference evidence: unavailable".to_string(),
    }
}

fn lunar_exact_source_window_requests() -> Vec<EphemerisRequest> {
    let mut requests = lunar_reference_evidence()
        .iter()
        .filter(|sample| sample.body == CelestialBody::Moon)
        .map(|sample| EphemerisRequest::new(sample.body.clone(), sample.epoch))
        .collect::<Vec<_>>();
    requests.extend(lunar_high_curvature_continuity_requests());
    requests
}

fn lunar_source_window_requests() -> Vec<EphemerisRequest> {
    let mut requests = lunar_exact_source_window_requests();
    requests.extend(lunar_apparent_comparison_requests());
    requests
}

/// This is a compatibility alias for `lunar_source_window_requests`.
#[doc(alias = "lunar_source_window_requests")]
pub fn lunar_source_window_request_corpus() -> Vec<EphemerisRequest> {
    lunar_source_window_requests()
}

/// A compact summary of the broader lunar source-window evidence slice.
#[derive(Clone, Debug, PartialEq)]
pub struct LunarSourceWindowSummary {
    /// Number of exact Moon samples across the exact source windows.
    pub exact_sample_count: usize,
    /// Number of reference-only apparent Moon samples across the apparent source windows.
    pub apparent_sample_count: usize,
    /// Number of distinct bodies covered by the broader source windows.
    pub body_count: usize,
    /// Number of exact source windows contributing to the broader slice.
    pub exact_window_count: usize,
    /// Number of apparent source windows contributing to the broader slice.
    pub apparent_window_count: usize,
    /// Earliest epoch represented in the broader source windows.
    pub earliest_epoch: Instant,
    /// Latest epoch represented in the broader source windows.
    pub latest_epoch: Instant,
}

/// Validation error for a lunar source-window summary that drifted from the current evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarSourceWindowSummaryValidationError {
    /// The summary no longer matches the current lunar source-window evidence.
    Unavailable,
    /// A rendered summary field no longer matches the current lunar source-window evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarSourceWindowSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => write!(
                f,
                "the lunar source-window summary is unavailable from the current evidence"
            ),
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar source-window summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarSourceWindowSummaryValidationError {}

impl LunarSourceWindowSummary {
    /// Returns the release-facing one-line lunar source-window summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar source windows: {} exact Moon samples across {} bodies in {} exact windows; {} reference-only apparent Moon samples across {} bodies in {} apparent windows, epoch range {}; exact windows: published 1992-04-12 geocentric Moon example; J2000 high-curvature continuity window; apparent windows: published 1992-04-12 apparent geocentric Moon comparison datum; published 1968-12-24 low-accuracy Meeus-style geocentric Moon example; published 2004-04-01 NASA RP 1349 apparent Moon table row; published 2006-09-07 EclipseWise apparent Moon coordinate row",
            self.exact_sample_count,
            self.body_count,
            self.exact_window_count,
            self.apparent_sample_count,
            self.body_count,
            self.apparent_window_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current lunar evidence.
    pub fn validate(&self) -> Result<(), LunarSourceWindowSummaryValidationError> {
        let Some(current) = lunar_source_window_summary() else {
            return Err(LunarSourceWindowSummaryValidationError::Unavailable);
        };

        if self.exact_sample_count != current.exact_sample_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "exact_sample_count",
            });
        }
        if self.apparent_sample_count != current.apparent_sample_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "apparent_sample_count",
            });
        }
        if self.body_count != current.body_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "body_count",
            });
        }
        if self.exact_window_count != current.exact_window_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "exact_window_count",
            });
        }
        if self.apparent_window_count != current.apparent_window_count {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "apparent_window_count",
            });
        }
        if self.earliest_epoch != current.earliest_epoch {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "earliest_epoch",
            });
        }
        if self.latest_epoch != current.latest_epoch {
            return Err(LunarSourceWindowSummaryValidationError::FieldOutOfSync {
                field: "latest_epoch",
            });
        }

        Ok(())
    }

    /// Returns the release-facing one-line summary when the current evidence still matches.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarSourceWindowSummaryValidationError> {
        self.validate().map(|()| self.summary_line())
    }
}

impl fmt::Display for LunarSourceWindowSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a compact summary of the broader lunar source-window evidence slice.
pub fn lunar_source_window_summary() -> Option<LunarSourceWindowSummary> {
    let exact_requests = lunar_exact_source_window_requests();
    let apparent_samples = lunar_apparent_comparison_evidence();
    if exact_requests.is_empty() && apparent_samples.is_empty() {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = exact_requests
        .first()
        .map(|request| request.instant)
        .or_else(|| apparent_samples.first().map(|sample| sample.epoch))
        .expect("lunar source window evidence should not be empty after guard");
    let mut latest_epoch = earliest_epoch;

    for request in &exact_requests {
        bodies.insert(request.body.to_string());
        if request.instant.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = request.instant;
        }
        if request.instant.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = request.instant;
        }
    }

    for sample in apparent_samples {
        bodies.insert(sample.body.to_string());
        if sample.epoch.julian_day.days() < earliest_epoch.julian_day.days() {
            earliest_epoch = sample.epoch;
        }
        if sample.epoch.julian_day.days() > latest_epoch.julian_day.days() {
            latest_epoch = sample.epoch;
        }
    }

    Some(LunarSourceWindowSummary {
        exact_sample_count: exact_requests.len(),
        apparent_sample_count: apparent_samples.len(),
        body_count: bodies.len(),
        exact_window_count: 2,
        apparent_window_count: apparent_samples.len(),
        earliest_epoch,
        latest_epoch,
    })
}

/// Formats the broader lunar source-window summary for release-facing reporting.
fn format_validated_lunar_source_window_summary_for_report(
    summary: &LunarSourceWindowSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("lunar source windows: unavailable ({error})"),
    }
}

/// Formats the broader lunar source-window summary for release-facing reporting.
pub fn format_lunar_source_window_summary(summary: &LunarSourceWindowSummary) -> String {
    format_validated_lunar_source_window_summary_for_report(summary)
}

/// Returns the validated release-facing broader lunar source-window summary string.
pub fn validated_lunar_source_window_summary_for_report() -> Result<String, String> {
    lunar_source_window_summary()
        .ok_or_else(|| {
            "the lunar source-window summary is unavailable from the current evidence".to_string()
        })
        .and_then(|summary| {
            summary
                .validated_summary_line()
                .map_err(|error| error.to_string())
        })
}

/// Returns the release-facing broader lunar source-window summary string.
pub fn lunar_source_window_summary_for_report() -> String {
    match lunar_source_window_summary() {
        Some(summary) => format_validated_lunar_source_window_summary_for_report(&summary),
        None => "lunar source windows: unavailable".to_string(),
    }
}

/// A compact summary of the mixed TT/TDB lunar reference batch-parity evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LunarReferenceBatchParitySummary {
    /// Number of batch requests in the mixed-scale regression slice.
    pub sample_count: usize,
    /// Number of distinct bodies covered by the mixed-scale regression slice.
    pub body_count: usize,
    /// Number of TT-tagged requests in the mixed-scale regression slice.
    pub tt_request_count: usize,
    /// Number of TDB-tagged requests in the mixed-scale regression slice.
    pub tdb_request_count: usize,
    /// Whether the batch results preserved request order.
    pub order_preserved: bool,
    /// Whether batch results matched the corresponding single-query lookups.
    pub single_query_parity: bool,
}

/// Validation error for a lunar mixed TT/TDB batch-parity summary that drifted
/// from the current reference evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LunarReferenceBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current evidence slice.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for LunarReferenceBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the lunar reference mixed TT/TDB batch parity summary field `{field}` is out of sync with the current evidence"
            ),
        }
    }
}

impl std::error::Error for LunarReferenceBatchParitySummaryValidationError {}

/// Returns a compact summary of the mixed TT/TDB lunar reference batch parity slice.
pub fn lunar_reference_batch_parity_summary() -> Option<LunarReferenceBatchParitySummary> {
    let requests = lunar_reference_batch_requests();
    if requests.is_empty() {
        return None;
    }

    let backend = ElpBackend::new();
    let bodies = requests
        .iter()
        .map(|request| request.body.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let tt_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tt)
        .count();
    let tdb_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tdb)
        .count();

    let results = backend.positions(&requests).ok()?;
    let mut order_preserved = true;
    let mut single_query_parity = true;

    for (request, result) in requests.iter().zip(results.iter()) {
        order_preserved &= result.body == request.body;

        let single = backend.position(request).ok()?;
        single_query_parity &= result.body == single.body
            && result.instant == single.instant
            && result.frame == single.frame
            && result.quality == single.quality
            && result.ecliptic == single.ecliptic
            && result.equatorial == single.equatorial
            && result.motion == single.motion;
    }

    Some(LunarReferenceBatchParitySummary {
        sample_count: requests.len(),
        body_count: bodies.len(),
        tt_request_count,
        tdb_request_count,
        order_preserved,
        single_query_parity,
    })
}

impl LunarReferenceBatchParitySummary {
    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), LunarReferenceBatchParitySummaryValidationError> {
        let samples = lunar_reference_evidence();
        let expected_sample_count = samples.len();
        let expected_body_count = samples
            .iter()
            .map(|sample| sample.body.to_string())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let expected_tt_request_count = expected_sample_count.div_ceil(2);
        let expected_tdb_request_count = expected_sample_count / 2;

        if self.sample_count != expected_sample_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.body_count != expected_body_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_count",
                },
            );
        }
        if self.tt_request_count != expected_tt_request_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tt_request_count",
                },
            );
        }
        if self.tdb_request_count != expected_tdb_request_count {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tdb_request_count",
                },
            );
        }
        if !self.order_preserved {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "order_preserved",
                },
            );
        }
        if !self.single_query_parity {
            return Err(
                LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "single_query_parity",
                },
            );
        }

        Ok(())
    }

    /// Returns the release-facing one-line lunar mixed-scale batch parity summary.
    pub fn summary_line(&self) -> String {
        let order = if self.order_preserved {
            "preserved"
        } else {
            "needs attention"
        };
        let parity = if self.single_query_parity {
            "preserved"
        } else {
            "needs attention"
        };

        format!(
            "lunar reference mixed TT/TDB batch parity: {} requests across {} bodies, TT requests={}, TDB requests={}, order={}, single-query parity={}",
            self.sample_count,
            self.body_count,
            self.tt_request_count,
            self.tdb_request_count,
            order,
            parity,
        )
    }
}

impl fmt::Display for LunarReferenceBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar mixed TT/TDB batch-parity evidence for release-facing reporting.
pub fn format_lunar_reference_batch_parity_summary(
    summary: &LunarReferenceBatchParitySummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar mixed TT/TDB batch-parity summary string.
pub fn lunar_reference_batch_parity_summary_for_report() -> String {
    match lunar_reference_batch_parity_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_reference_batch_parity_summary(&summary),
            Err(error) => {
                format!("lunar reference mixed TT/TDB batch parity: unavailable ({error})")
            }
        },
        None => "lunar reference mixed TT/TDB batch parity: unavailable".to_string(),
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
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
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

impl LunarEquatorialReferenceEvidenceSummary {
    /// Returns the release-facing one-line lunar equatorial reference evidence summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar equatorial reference evidence: {} samples across {} bodies, epoch range {}, validated against the published 1992-04-12 geocentric Moon RA/Dec example, a derived 1992 equatorial companion built from the published 1992 geocentric Moon example via the shared mean-obliquity transform, and a reference-only published 1992-04-12 apparent geocentric Moon comparison datum",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
        )
    }

    /// Returns `Ok(())` when the summary still matches the current reference evidence.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_equatorial_reference_evidence_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar equatorial reference evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar equatorial reference evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for LunarEquatorialReferenceEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial reference evidence summary for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_summary(
    summary: &LunarEquatorialReferenceEvidenceSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing lunar equatorial reference evidence summary string.
pub fn lunar_equatorial_reference_evidence_summary_for_report() -> String {
    match lunar_equatorial_reference_evidence_summary() {
        Some(summary) => match summary.validate() {
            Ok(()) => format_lunar_equatorial_reference_evidence_summary(&summary),
            Err(error) => format!("lunar equatorial reference evidence: unavailable ({error})"),
        },
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

impl LunarApparentComparisonSample {
    /// Returns `Ok(())` when the sample still represents a valid apparent Moon evidence row.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        if self.body != CelestialBody::Moon {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use the Moon body",
            ));
        }
        if self.epoch.scale != TimeScale::Tt {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use TT epochs",
            ));
        }
        if !self.apparent_longitude_deg.is_finite()
            || !self.apparent_latitude_deg.is_finite()
            || !self.apparent_distance_au.is_finite()
            || self.apparent_distance_au <= 0.0
            || !self.apparent_right_ascension_deg.is_finite()
            || !self.apparent_declination_deg.is_finite()
        {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample must use finite apparent coordinates",
            ));
        }
        if self.note.trim().is_empty() || self.note.trim() != self.note {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison sample note must be a non-blank canonical line",
            ));
        }

        Ok(())
    }

    /// Returns a compact, release-facing summary of the published sample.
    pub fn summary_line(&self) -> String {
        format!(
            "body={}; epoch={}; apparent lon/lat/dist={:+.6}°/{:+.6}°/{:.12} AU; apparent RA/Dec={:+.6}°/{:+.6}°; note={}",
            self.body,
            format_instant(self.epoch),
            self.apparent_longitude_deg,
            self.apparent_latitude_deg,
            self.apparent_distance_au,
            self.apparent_right_ascension_deg,
            self.apparent_declination_deg,
            self.note,
        )
    }
}

impl fmt::Display for LunarApparentComparisonSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_apparent_comparison_requests_for_frame(frame: CoordinateFrame) -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_evidence()
        .iter()
        .map(|sample| {
            let mut request = EphemerisRequest::new(sample.body.clone(), sample.epoch);
            request.frame = frame;
            request.instant.scale = TimeScale::Tt;
            request.zodiac_mode = ZodiacMode::Tropical;
            request.apparent = Apparentness::Mean;
            request
        })
        .collect()
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
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_453_100.5),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 135.576_300,
            apparent_latitude_deg: 5.203_935,
            apparent_distance_au: 391_200.0 / 149_597_870.700,
            apparent_right_ascension_deg: 139.685_833_333_333_33,
            apparent_declination_deg: 21.130_333_333_333_333,
            note: "Published 2004-04-01 geocentric Moon table row from NASA RP 1349; apparent equatorial coordinates are published directly and the ecliptic row is derived from them using the shared mean-obliquity transform",
        },
        LunarApparentComparisonSample {
            body: CelestialBody::Moon,
            epoch: Instant::new(
                pleiades_types::JulianDay::from_days(2_453_986.285_649),
                TimeScale::Tt,
            ),
            apparent_longitude_deg: 345.100_191_937_146_6,
            apparent_latitude_deg: -0.943_574_954_873_848_7,
            apparent_distance_au: 0.002_388_348_345_388_321_4,
            apparent_right_ascension_deg: 346.648_333_333_333_3,
            apparent_declination_deg: -6.740_444_444_444_445,
            note: "Published 2006-09-07 geocentric Moon coordinate row from EclipseWise; apparent equatorial coordinates are published directly and the ecliptic row is derived from them using the shared mean-obliquity transform",
        },
    ];

    SAMPLES
}

/// Returns the canonical ecliptic apparent-comparison request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests_for_frame(CoordinateFrame::Ecliptic)
}

/// This is a compatibility alias for [`lunar_apparent_comparison_requests`].
#[doc(alias = "lunar_apparent_comparison_requests")]
pub fn lunar_apparent_comparison_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests()
}

/// Returns the canonical ecliptic apparent-comparison batch-parity request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests()
}

/// This is a compatibility alias for [`lunar_apparent_comparison_batch_parity_requests`].
#[doc(alias = "lunar_apparent_comparison_batch_parity_requests")]
pub fn lunar_apparent_comparison_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_batch_parity_requests()
}

/// Returns the canonical equatorial apparent-comparison request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_equatorial_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_requests_for_frame(CoordinateFrame::Equatorial)
}

/// This is a compatibility alias for [`lunar_apparent_comparison_equatorial_requests`].
#[doc(alias = "lunar_apparent_comparison_equatorial_requests")]
pub fn lunar_apparent_comparison_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_requests()
}

/// Returns the canonical equatorial apparent-comparison batch-parity request corpus used by validation and reporting.
pub fn lunar_apparent_comparison_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_requests()
}

/// This is a compatibility alias for [`lunar_apparent_comparison_equatorial_batch_parity_requests`].
#[doc(alias = "lunar_apparent_comparison_equatorial_batch_parity_requests")]
pub fn lunar_apparent_comparison_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_apparent_comparison_equatorial_batch_parity_requests()
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
    /// Mean absolute apparent minus mean ecliptic longitude delta in degrees.
    pub mean_ecliptic_longitude_delta_deg: f64,
    /// Median absolute apparent minus mean ecliptic longitude delta in degrees.
    pub median_ecliptic_longitude_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean ecliptic longitude delta in degrees.
    pub percentile_ecliptic_longitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean ecliptic latitude delta.
    pub max_ecliptic_latitude_epoch: Instant,
    /// Maximum apparent minus mean ecliptic latitude delta in degrees.
    pub max_ecliptic_latitude_delta_deg: f64,
    /// Mean absolute apparent minus mean ecliptic latitude delta in degrees.
    pub mean_ecliptic_latitude_delta_deg: f64,
    /// Median absolute apparent minus mean ecliptic latitude delta in degrees.
    pub median_ecliptic_latitude_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean ecliptic latitude delta in degrees.
    pub percentile_ecliptic_latitude_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean distance delta.
    pub max_ecliptic_distance_epoch: Instant,
    /// Maximum apparent minus mean distance delta in astronomical units.
    pub max_ecliptic_distance_delta_au: f64,
    /// Mean absolute apparent minus mean distance delta in astronomical units.
    pub mean_ecliptic_distance_delta_au: f64,
    /// Median absolute apparent minus mean distance delta in astronomical units.
    pub median_ecliptic_distance_delta_au: f64,
    /// 95th-percentile absolute apparent minus mean distance delta in astronomical units.
    pub percentile_ecliptic_distance_delta_au: f64,
    /// Epoch that produces the maximum apparent minus mean right ascension delta.
    pub max_right_ascension_epoch: Instant,
    /// Maximum apparent minus mean right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Mean absolute apparent minus mean right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute apparent minus mean right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Epoch that produces the maximum apparent minus mean declination delta.
    pub max_declination_epoch: Instant,
    /// Maximum apparent minus mean declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Mean absolute apparent minus mean declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute apparent minus mean declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th-percentile absolute apparent minus mean declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
}

impl LunarApparentComparisonSummary {
    /// Validates that the summary still matches the checked-in apparent evidence slice.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let Some(expected) = lunar_apparent_comparison_summary() else {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "lunar apparent comparison evidence is unavailable",
            ));
        };

        if self != &expected {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "lunar apparent comparison evidence summary mismatch: expected {}, found {}",
                    expected.summary_line(),
                    self.summary_line()
                ),
            ));
        }

        Ok(())
    }

    /// Returns the release-facing one-line apparent comparison summary.
    pub fn summary_line(&self) -> String {
        format!(
            "lunar apparent comparison evidence: {} reference-only samples across {} bodies, epoch range {}, mean-only gap against the published apparent Moon examples, including the 1992-04-12, 1968-12-24, 2004-04-01, and 2006-09-07 examples: Δlon={:+.6}° @ {}; |Δlon| mean/median/p95={:.6}/{:.6}/{:.6}°; Δlat={:+.6}° @ {}; |Δlat| mean/median/p95={:.6}/{:.6}/{:.6}°; Δdist={:+.12} AU @ {}; |Δdist| mean/median/p95={:.12}/{:.12}/{:.12} AU; ΔRA={:+.6}° @ {}; |ΔRA| mean/median/p95={:.6}/{:.6}/{:.6}°; ΔDec={:+.6}° @ {}; |ΔDec| mean/median/p95={:.6}/{:.6}/{:.6}°; apparent requests remain unsupported",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_ecliptic_longitude_delta_deg,
            format_instant(self.max_ecliptic_longitude_epoch),
            self.mean_ecliptic_longitude_delta_deg,
            self.median_ecliptic_longitude_delta_deg,
            self.percentile_ecliptic_longitude_delta_deg,
            self.max_ecliptic_latitude_delta_deg,
            format_instant(self.max_ecliptic_latitude_epoch),
            self.mean_ecliptic_latitude_delta_deg,
            self.median_ecliptic_latitude_delta_deg,
            self.percentile_ecliptic_latitude_delta_deg,
            self.max_ecliptic_distance_delta_au,
            format_instant(self.max_ecliptic_distance_epoch),
            self.mean_ecliptic_distance_delta_au,
            self.median_ecliptic_distance_delta_au,
            self.percentile_ecliptic_distance_delta_au,
            self.max_right_ascension_delta_deg,
            format_instant(self.max_right_ascension_epoch),
            self.mean_right_ascension_delta_deg,
            self.median_right_ascension_delta_deg,
            self.percentile_right_ascension_delta_deg,
            self.max_declination_delta_deg,
            format_instant(self.max_declination_epoch),
            self.mean_declination_delta_deg,
            self.median_declination_delta_deg,
            self.percentile_declination_delta_deg,
        )
    }
}

impl fmt::Display for LunarApparentComparisonSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
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
    if samples.is_empty() || samples.iter().any(|sample| sample.validate().is_err()) {
        return None;
    }

    let mut bodies = std::collections::BTreeSet::new();
    let mut earliest_epoch = samples[0].epoch;
    let mut latest_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_epoch = samples[0].epoch;
    let mut max_ecliptic_longitude_delta_deg = 0.0;
    let mut max_ecliptic_longitude_delta_abs = 0.0;
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut max_ecliptic_latitude_epoch = samples[0].epoch;
    let mut max_ecliptic_latitude_delta_deg = 0.0;
    let mut max_ecliptic_latitude_delta_abs = 0.0;
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut max_ecliptic_distance_epoch = samples[0].epoch;
    let mut max_ecliptic_distance_delta_au = 0.0;
    let mut max_ecliptic_distance_delta_abs = 0.0;
    let mut distance_deltas = Vec::with_capacity(samples.len());
    let mut max_right_ascension_epoch = samples[0].epoch;
    let mut max_right_ascension_delta_deg = 0.0;
    let mut max_right_ascension_delta_abs = 0.0;
    let mut right_ascension_deltas = Vec::with_capacity(samples.len());
    let mut max_declination_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut max_declination_delta_abs = 0.0;
    let mut declination_deltas = Vec::with_capacity(samples.len());

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
        longitude_deltas.push(longitude_delta_abs);
        if longitude_delta_abs >= max_ecliptic_longitude_delta_abs {
            max_ecliptic_longitude_delta_abs = longitude_delta_abs;
            max_ecliptic_longitude_delta_deg = longitude_delta;
            max_ecliptic_longitude_epoch = sample.epoch;
        }
        let latitude_delta_abs = latitude_delta.abs();
        latitude_deltas.push(latitude_delta_abs);
        if latitude_delta_abs >= max_ecliptic_latitude_delta_abs {
            max_ecliptic_latitude_delta_abs = latitude_delta_abs;
            max_ecliptic_latitude_delta_deg = latitude_delta;
            max_ecliptic_latitude_epoch = sample.epoch;
        }
        let distance_delta_abs = distance_delta.abs();
        distance_deltas.push(distance_delta_abs);
        if distance_delta_abs >= max_ecliptic_distance_delta_abs {
            max_ecliptic_distance_delta_abs = distance_delta_abs;
            max_ecliptic_distance_delta_au = distance_delta;
            max_ecliptic_distance_epoch = sample.epoch;
        }
        let right_ascension_delta_abs = right_ascension_delta.abs();
        right_ascension_deltas.push(right_ascension_delta_abs);
        if right_ascension_delta_abs >= max_right_ascension_delta_abs {
            max_right_ascension_delta_abs = right_ascension_delta_abs;
            max_right_ascension_delta_deg = right_ascension_delta;
            max_right_ascension_epoch = sample.epoch;
        }
        let declination_delta_abs = declination_delta.abs();
        declination_deltas.push(declination_delta_abs);
        if declination_delta_abs >= max_declination_delta_abs {
            max_declination_delta_abs = declination_delta_abs;
            max_declination_delta_deg = declination_delta;
            max_declination_epoch = sample.epoch;
        }
    }

    if compared_sample_count == 0 {
        return None;
    }

    let mut longitude_deltas_for_median = longitude_deltas.clone();
    let mut longitude_deltas_for_percentile = longitude_deltas.clone();
    let mut latitude_deltas_for_median = latitude_deltas.clone();
    let mut latitude_deltas_for_percentile = latitude_deltas.clone();
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas.clone();
    let mut right_ascension_deltas_for_median = right_ascension_deltas.clone();
    let mut right_ascension_deltas_for_percentile = right_ascension_deltas.clone();
    let mut declination_deltas_for_median = declination_deltas.clone();
    let mut declination_deltas_for_percentile = declination_deltas.clone();

    Some(LunarApparentComparisonSummary {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_ecliptic_longitude_epoch,
        max_ecliptic_longitude_delta_deg,
        mean_ecliptic_longitude_delta_deg: mean_value(&longitude_deltas).unwrap_or(0.0),
        median_ecliptic_longitude_delta_deg: median_value(&mut longitude_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_longitude_delta_deg: percentile_value(
            &mut longitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_ecliptic_latitude_epoch,
        max_ecliptic_latitude_delta_deg,
        mean_ecliptic_latitude_delta_deg: mean_value(&latitude_deltas).unwrap_or(0.0),
        median_ecliptic_latitude_delta_deg: median_value(&mut latitude_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_latitude_delta_deg: percentile_value(
            &mut latitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_ecliptic_distance_epoch,
        max_ecliptic_distance_delta_au,
        mean_ecliptic_distance_delta_au: mean_value(&distance_deltas).unwrap_or(0.0),
        median_ecliptic_distance_delta_au: median_value(&mut distance_deltas_for_median)
            .unwrap_or(0.0),
        percentile_ecliptic_distance_delta_au: percentile_value(
            &mut distance_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_right_ascension_epoch,
        max_right_ascension_delta_deg,
        mean_right_ascension_delta_deg: mean_value(&right_ascension_deltas).unwrap_or(0.0),
        median_right_ascension_delta_deg: median_value(&mut right_ascension_deltas_for_median)
            .unwrap_or(0.0),
        percentile_right_ascension_delta_deg: percentile_value(
            &mut right_ascension_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
        max_declination_epoch,
        max_declination_delta_deg,
        mean_declination_delta_deg: mean_value(&declination_deltas).unwrap_or(0.0),
        median_declination_delta_deg: median_value(&mut declination_deltas_for_median)
            .unwrap_or(0.0),
        percentile_declination_delta_deg: percentile_value(
            &mut declination_deltas_for_percentile,
            0.95,
        )
        .unwrap_or(0.0),
    })
}

/// Formats the reference-only apparent Moon comparison summary for release-facing reporting.
pub fn format_lunar_apparent_comparison_summary(
    summary: &LunarApparentComparisonSummary,
) -> String {
    summary.summary_line()
}

/// Returns the release-facing one-line apparent comparison summary.
pub fn lunar_apparent_comparison_summary_for_report() -> String {
    match lunar_apparent_comparison_summary() {
        Some(summary) if summary.validate().is_ok() => {
            format_lunar_apparent_comparison_summary(&summary)
        }
        Some(_) | None => "lunar apparent comparison evidence: unavailable".to_string(),
    }
}

fn mean_value(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    Some(values.iter().copied().sum::<f64>() / values.len() as f64)
}

fn median_value(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let midpoint = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Some((values[midpoint - 1] + values[midpoint]) / 2.0)
    } else {
        Some(values[midpoint])
    }
}

fn percentile_value(values: &mut [f64], percentile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let percentile = percentile.clamp(0.0, 1.0);
    let position = percentile * (values.len().saturating_sub(1)) as f64;
    let lower_index = position.floor() as usize;
    let upper_index = position.ceil() as usize;
    if lower_index == upper_index {
        Some(values[lower_index])
    } else {
        let weight = position - lower_index as f64;
        Some(values[lower_index] * (1.0 - weight) + values[upper_index] * weight)
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
    /// Median absolute right ascension delta in degrees across the evidence slice.
    pub median_right_ascension_delta_deg: f64,
    /// 95th-percentile absolute right ascension delta in degrees across the evidence slice.
    pub percentile_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Epoch for the maximum absolute declination delta.
    pub max_declination_delta_epoch: Instant,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Mean absolute declination delta in degrees across the evidence slice.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees across the evidence slice.
    pub median_declination_delta_deg: f64,
    /// 95th-percentile absolute declination delta in degrees across the evidence slice.
    pub percentile_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Median absolute distance delta in astronomical units across the samples that include distance.
    pub median_distance_delta_au: Option<f64>,
    /// 95th-percentile absolute distance delta in astronomical units across the samples that include distance.
    pub percentile_distance_delta_au: Option<f64>,
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
    let mut right_ascension_deltas = Vec::with_capacity(samples.len());
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_declination_delta_body = samples[0].body.clone();
    let mut max_declination_delta_epoch = samples[0].epoch;
    let mut max_declination_delta_deg = 0.0;
    let mut total_declination_delta_deg = 0.0;
    let mut declination_deltas = Vec::with_capacity(samples.len());
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_deltas = Vec::new();
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
        right_ascension_deltas.push(right_ascension_delta_deg);
        total_declination_delta_deg += declination_delta_deg;
        declination_deltas.push(declination_delta_deg);
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_deltas.push(delta);
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

    let mut right_ascension_deltas_for_median = right_ascension_deltas.clone();
    let mut right_ascension_deltas_for_percentile = right_ascension_deltas;
    let mut declination_deltas_for_median = declination_deltas.clone();
    let mut declination_deltas_for_percentile = declination_deltas;
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas;

    Some(LunarEquatorialReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_right_ascension_delta_body,
        max_right_ascension_delta_epoch,
        max_right_ascension_delta_deg,
        mean_right_ascension_delta_deg: total_right_ascension_delta_deg / samples.len() as f64,
        median_right_ascension_delta_deg: median_value(&mut right_ascension_deltas_for_median)
            .unwrap_or_default(),
        percentile_right_ascension_delta_deg: percentile_value(
            &mut right_ascension_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_declination_delta_body,
        max_declination_delta_epoch,
        max_declination_delta_deg,
        mean_declination_delta_deg: total_declination_delta_deg / samples.len() as f64,
        median_declination_delta_deg: median_value(&mut declination_deltas_for_median)
            .unwrap_or_default(),
        percentile_declination_delta_deg: percentile_value(
            &mut declination_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        median_distance_delta_au: median_value(&mut distance_deltas_for_median),
        percentile_distance_delta_au: percentile_value(&mut distance_deltas_for_percentile, 0.95),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

#[derive(Clone, Debug, PartialEq)]
enum LunarEvidenceEnvelopeValidationError {
    SampleCountTooSmall {
        envelope: &'static str,
        sample_count: usize,
    },
    BodyCountTooSmall {
        envelope: &'static str,
        body_count: usize,
    },
    InvalidEpochRange {
        envelope: &'static str,
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    NonFiniteMeasure {
        envelope: &'static str,
        field: &'static str,
    },
    DuplicateOutlierBody {
        envelope: &'static str,
        body: CelestialBody,
    },
    OutlierCountMismatch {
        envelope: &'static str,
        outside_current_limits_count: usize,
        sample_count: usize,
        outlier_bodies_len: usize,
    },
    WithinCurrentLimitsMismatch {
        envelope: &'static str,
        outside_current_limits_count: usize,
        within_current_limits: bool,
    },
}

impl fmt::Display for LunarEvidenceEnvelopeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SampleCountTooSmall {
                envelope,
                sample_count,
            } => write!(f, "{envelope} has no samples ({sample_count})"),
            Self::BodyCountTooSmall {
                envelope,
                body_count,
            } => write!(f, "{envelope} has no bodies ({body_count})"),
            Self::InvalidEpochRange {
                envelope,
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "{envelope} has an invalid epoch range {}",
                TimeRange::new(Some(*earliest_epoch), Some(*latest_epoch)).summary_line()
            ),
            Self::NonFiniteMeasure { envelope, field } => {
                write!(f, "{envelope} field `{field}` is not finite")
            }
            Self::DuplicateOutlierBody { envelope, body } => {
                write!(f, "{envelope} has duplicate outlier body {body}")
            }
            Self::OutlierCountMismatch {
                envelope,
                outside_current_limits_count,
                sample_count,
                outlier_bodies_len,
            } => write!(
                f,
                "{envelope} reports {outside_current_limits_count} samples outside the limits across {sample_count} samples with {outlier_bodies_len} unique outlier bodies"
            ),
            Self::WithinCurrentLimitsMismatch {
                envelope,
                outside_current_limits_count,
                within_current_limits,
            } => write!(
                f,
                "{envelope} reports within_current_limits={within_current_limits} but outside_current_limits_count={outside_current_limits_count}"
            ),
        }
    }
}

impl std::error::Error for LunarEvidenceEnvelopeValidationError {}

#[allow(clippy::too_many_arguments)]
fn validate_lunar_evidence_envelope(
    envelope: &'static str,
    sample_count: usize,
    body_count: usize,
    earliest_epoch: Instant,
    latest_epoch: Instant,
    numeric_fields: &[(&'static str, f64)],
    distance_fields: &[(&'static str, Option<f64>)],
    outlier_bodies: &[CelestialBody],
    outside_current_limits_count: usize,
    within_current_limits: bool,
) -> Result<(), LunarEvidenceEnvelopeValidationError> {
    if sample_count == 0 {
        return Err(LunarEvidenceEnvelopeValidationError::SampleCountTooSmall {
            envelope,
            sample_count,
        });
    }
    if body_count == 0 {
        return Err(LunarEvidenceEnvelopeValidationError::BodyCountTooSmall {
            envelope,
            body_count,
        });
    }
    if earliest_epoch.julian_day.days() > latest_epoch.julian_day.days() {
        return Err(LunarEvidenceEnvelopeValidationError::InvalidEpochRange {
            envelope,
            earliest_epoch,
            latest_epoch,
        });
    }

    for (field, value) in numeric_fields {
        if !value.is_finite() {
            return Err(LunarEvidenceEnvelopeValidationError::NonFiniteMeasure { envelope, field });
        }
    }

    for (field, value) in distance_fields {
        if let Some(value) = value {
            if !value.is_finite() {
                return Err(LunarEvidenceEnvelopeValidationError::NonFiniteMeasure {
                    envelope,
                    field,
                });
            }
        }
    }

    let has_outlier_samples = outside_current_limits_count > 0;
    if outside_current_limits_count > sample_count
        || outlier_bodies.is_empty() == has_outlier_samples
    {
        return Err(LunarEvidenceEnvelopeValidationError::OutlierCountMismatch {
            envelope,
            outside_current_limits_count,
            sample_count,
            outlier_bodies_len: outlier_bodies.len(),
        });
    }
    if within_current_limits == has_outlier_samples {
        return Err(
            LunarEvidenceEnvelopeValidationError::WithinCurrentLimitsMismatch {
                envelope,
                outside_current_limits_count,
                within_current_limits,
            },
        );
    }

    let mut seen_outlier_bodies = Vec::with_capacity(outlier_bodies.len());
    for body in outlier_bodies {
        if seen_outlier_bodies.iter().any(|seen| seen == body) {
            return Err(LunarEvidenceEnvelopeValidationError::DuplicateOutlierBody {
                envelope,
                body: body.clone(),
            });
        }
        seen_outlier_bodies.push(body.clone());
    }

    Ok(())
}

impl LunarEquatorialReferenceEvidenceEnvelope {
    /// Returns `Ok(())` when the equatorial reference envelope remains internally consistent.
    pub(crate) fn validate(&self) -> Result<(), LunarEvidenceEnvelopeValidationError> {
        validate_lunar_evidence_envelope(
            "lunar equatorial reference error envelope",
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            &[
                (
                    "max_right_ascension_delta_deg",
                    self.max_right_ascension_delta_deg,
                ),
                (
                    "mean_right_ascension_delta_deg",
                    self.mean_right_ascension_delta_deg,
                ),
                (
                    "median_right_ascension_delta_deg",
                    self.median_right_ascension_delta_deg,
                ),
                (
                    "percentile_right_ascension_delta_deg",
                    self.percentile_right_ascension_delta_deg,
                ),
                ("max_declination_delta_deg", self.max_declination_delta_deg),
                (
                    "mean_declination_delta_deg",
                    self.mean_declination_delta_deg,
                ),
                (
                    "median_declination_delta_deg",
                    self.median_declination_delta_deg,
                ),
                (
                    "percentile_declination_delta_deg",
                    self.percentile_declination_delta_deg,
                ),
            ],
            &[
                ("max_distance_delta_au", self.max_distance_delta_au),
                ("mean_distance_delta_au", self.mean_distance_delta_au),
                ("median_distance_delta_au", self.median_distance_delta_au),
                (
                    "percentile_distance_delta_au",
                    self.percentile_distance_delta_au,
                ),
            ],
            &self.outlier_bodies,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }

    /// Returns the release-facing one-line lunar equatorial reference error envelope.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
            format!("{} @ {}", body, format_instant(epoch))
        }

        let distance = match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_epoch,
            self.max_distance_delta_au,
        ) {
            (Some(body), Some(epoch), Some(delta)) => {
                format!(
                    "; max Δdist={delta:.12} AU ({})",
                    format_body_epoch(body, epoch)
                )
            }
            _ => String::new(),
        };
        let mean_distance = self
            .mean_distance_delta_au
            .map(|value| format!("; mean Δdist={value:.12} AU"))
            .unwrap_or_default();
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("; median Δdist={value:.12} AU"))
            .unwrap_or_default();
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("; p95 Δdist={value:.12} AU"))
            .unwrap_or_default();
        let limit_note = "; limits: ΔRA≤1e-2°, ΔDec≤1e-2°, Δdist≤1e-8 AU";
        let outlier_note = if self.outlier_bodies.is_empty() {
            "; outliers=none".to_string()
        } else {
            format!("; outliers={}", format_bodies(&self.outlier_bodies))
        };

        format!(
            "lunar equatorial reference error envelope: {} samples across {} bodies, epoch range {}, max ΔRA={:.12}° ({}), mean ΔRA={:.12}°, median ΔRA={:.12}°, p95 ΔRA={:.12}°, max ΔDec={:.12}° ({}), mean ΔDec={:.12}°, median ΔDec={:.12}°, p95 ΔDec={:.12}°{}{}{}{}{}{}; outside current limits={}; within current limits={}",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_right_ascension_delta_deg,
            format_body_epoch(&self.max_right_ascension_delta_body, self.max_right_ascension_delta_epoch),
            self.mean_right_ascension_delta_deg,
            self.median_right_ascension_delta_deg,
            self.percentile_right_ascension_delta_deg,
            self.max_declination_delta_deg,
            format_body_epoch(&self.max_declination_delta_body, self.max_declination_delta_epoch),
            self.mean_declination_delta_deg,
            self.median_declination_delta_deg,
            self.percentile_declination_delta_deg,
            distance,
            mean_distance,
            median_distance,
            percentile_distance,
            limit_note,
            outlier_note,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }
}

impl fmt::Display for LunarEquatorialReferenceEvidenceEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar equatorial reference error envelope for release-facing reporting.
pub fn format_lunar_equatorial_reference_evidence_envelope(
    envelope: &LunarEquatorialReferenceEvidenceEnvelope,
) -> String {
    envelope.summary_line()
}

/// Returns the release-facing lunar equatorial reference error envelope string.
pub fn lunar_equatorial_reference_evidence_envelope_for_report() -> String {
    match lunar_equatorial_reference_evidence_envelope() {
        Some(envelope) => match envelope.validate() {
            Ok(()) => format_lunar_equatorial_reference_evidence_envelope(&envelope),
            Err(error) => {
                format!("lunar equatorial reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar equatorial reference error envelope: unavailable".to_string(),
    }
}

/// Validation error for a lunar high-curvature continuity evidence slice.
#[derive(Clone, Debug, PartialEq)]
enum LunarHighCurvatureEvidenceValidationError {
    /// The regression slice does not include enough samples to justify a continuity envelope.
    SampleCountTooSmall { sample_count: usize },
    /// The regression slice unexpectedly lost all bodies.
    BodyCountTooSmall { body_count: usize },
    /// The stored epoch bounds no longer describe a monotonic range.
    InvalidEpochRange {
        earliest_epoch: Instant,
        latest_epoch: Instant,
    },
    /// A stored step metric is not finite.
    NonFiniteMeasure { field: &'static str },
    /// A stored step window is reversed.
    ReversedStepWindow { field: &'static str },
    /// The stored regression-limit flag drifted away from the derived thresholds.
    RegressionLimitMismatch {
        envelope: &'static str,
        within_regression_limits: bool,
        expected_within_regression_limits: bool,
    },
}

impl fmt::Display for LunarHighCurvatureEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SampleCountTooSmall { sample_count } => write!(
                f,
                "the lunar high-curvature continuity evidence has too few samples ({sample_count})"
            ),
            Self::BodyCountTooSmall { body_count } => write!(
                f,
                "the lunar high-curvature continuity evidence has no bodies ({body_count})"
            ),
            Self::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            } => write!(
                f,
                "the lunar high-curvature continuity evidence has an invalid epoch range {}",
                TimeRange::new(Some(*earliest_epoch), Some(*latest_epoch)).summary_line()
            ),
            Self::NonFiniteMeasure { field } => write!(
                f,
                "the lunar high-curvature continuity evidence field `{field}` is not finite"
            ),
            Self::ReversedStepWindow { field } => write!(
                f,
                "the lunar high-curvature continuity evidence field `{field}` has a reversed step window"
            ),
            Self::RegressionLimitMismatch {
                envelope,
                within_regression_limits,
                expected_within_regression_limits,
            } => write!(
                f,
                "the {envelope} stored regression-limit flag `{within_regression_limits}` does not match the derived limits (`{expected_within_regression_limits}`)"
            ),
        }
    }
}

impl std::error::Error for LunarHighCurvatureEvidenceValidationError {}

fn validate_high_curvature_continuity_window(
    sample_count: usize,
    body_count: usize,
    earliest_epoch: Instant,
    latest_epoch: Instant,
    step_fields: [(&'static str, Instant, Instant, f64); 3],
) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
    if sample_count < 2 {
        return Err(
            LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count },
        );
    }
    if body_count == 0 {
        return Err(LunarHighCurvatureEvidenceValidationError::BodyCountTooSmall { body_count });
    }
    if earliest_epoch.julian_day.days() > latest_epoch.julian_day.days() {
        return Err(
            LunarHighCurvatureEvidenceValidationError::InvalidEpochRange {
                earliest_epoch,
                latest_epoch,
            },
        );
    }

    for (field, start_epoch, end_epoch, measure) in step_fields {
        if !measure.is_finite() {
            return Err(LunarHighCurvatureEvidenceValidationError::NonFiniteMeasure { field });
        }
        if start_epoch.julian_day.days() > end_epoch.julian_day.days() {
            return Err(LunarHighCurvatureEvidenceValidationError::ReversedStepWindow { field });
        }
    }

    Ok(())
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

impl LunarHighCurvatureContinuityEnvelope {
    /// Returns the compact release-facing summary line for this continuity evidence slice.
    fn summary_line(&self) -> String {
        format!(
            "lunar high-curvature continuity evidence: {} samples across {} bodies, epoch range {}, max adjacent Δlon={:.12}° ({} → {}), max adjacent Δlat={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: Δlon≤{:.1}°, Δlat≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_longitude_step_deg,
            format_instant(self.max_longitude_step_start_epoch),
            format_instant(self.max_longitude_step_end_epoch),
            self.max_latitude_step_deg,
            format_instant(self.max_latitude_step_start_epoch),
            format_instant(self.max_latitude_step_end_epoch),
            self.max_distance_step_au,
            format_instant(self.max_distance_step_start_epoch),
            format_instant(self.max_distance_step_end_epoch),
            LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG,
            LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG,
            LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
            self.within_regression_limits,
        )
    }

    /// Returns the compact summary line after validating the continuity evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarHighCurvatureEvidenceValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    fn validate(&self) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
        validate_high_curvature_continuity_window(
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            [
                (
                    "max_longitude_step_deg",
                    self.max_longitude_step_start_epoch,
                    self.max_longitude_step_end_epoch,
                    self.max_longitude_step_deg,
                ),
                (
                    "max_latitude_step_deg",
                    self.max_latitude_step_start_epoch,
                    self.max_latitude_step_end_epoch,
                    self.max_latitude_step_deg,
                ),
                (
                    "max_distance_step_au",
                    self.max_distance_step_start_epoch,
                    self.max_distance_step_end_epoch,
                    self.max_distance_step_au,
                ),
            ],
        )?;

        let expected_within_regression_limits = self.max_longitude_step_deg
            <= LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG
            && self.max_latitude_step_deg <= LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG
            && self.max_distance_step_au <= LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;
        if self.within_regression_limits != expected_within_regression_limits {
            return Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature continuity evidence",
                    within_regression_limits: self.within_regression_limits,
                    expected_within_regression_limits,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarHighCurvatureContinuityEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_high_curvature_requests(frame: CoordinateFrame) -> Vec<EphemerisRequest> {
    LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS
        .into_iter()
        .map(|instant| {
            let mut request = EphemerisRequest::new(CelestialBody::Moon, instant);
            request.frame = frame;
            request
        })
        .collect()
}

/// Returns the lunar high-curvature continuity request corpus used by the nearby-motion regression slice.
pub fn lunar_high_curvature_continuity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_requests(CoordinateFrame::Ecliptic)
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
pub fn lunar_high_curvature_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_request_corpus")]
pub fn lunar_high_curvature_continuity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_continuity_request_corpus")]
pub fn lunar_high_curvature_continuity_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_continuity_requests`].
#[doc(alias = "lunar_high_curvature_continuity_requests")]
#[doc(alias = "lunar_high_curvature_continuity_request_corpus")]
pub fn lunar_high_curvature_continuity_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_continuity_requests()
}

/// Returns the lunar high-curvature equatorial continuity request corpus used by the nearby-motion regression slice.
pub fn lunar_high_curvature_equatorial_continuity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_requests(CoordinateFrame::Equatorial)
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
pub fn lunar_high_curvature_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_request_corpus() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_continuity_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_batch_parity_requests() -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

/// This is a compatibility alias for [`lunar_high_curvature_equatorial_continuity_requests`].
#[doc(alias = "lunar_high_curvature_equatorial_continuity_requests")]
#[doc(alias = "lunar_high_curvature_equatorial_continuity_request_corpus")]
pub fn lunar_high_curvature_equatorial_continuity_batch_parity_request_corpus(
) -> Vec<EphemerisRequest> {
    lunar_high_curvature_equatorial_continuity_requests()
}

fn lunar_high_curvature_continuity_envelope() -> Option<LunarHighCurvatureContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let requests = lunar_high_curvature_continuity_requests();
    let sample_count = requests.len();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_longitude_step_start_epoch = requests[0].instant;
    let mut max_longitude_step_end_epoch = requests[0].instant;
    let mut max_longitude_step_deg = 0.0;
    let mut max_latitude_step_start_epoch = requests[0].instant;
    let mut max_latitude_step_end_epoch = requests[0].instant;
    let mut max_latitude_step_deg = 0.0;
    let mut max_distance_step_start_epoch = requests[0].instant;
    let mut max_distance_step_end_epoch = requests[0].instant;
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for request in &requests {
        let instant = request.instant;
        let result = backend
            .position(request)
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
        sample_count,
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
    match envelope.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(_) => "lunar high-curvature continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature continuity evidence string.
pub fn lunar_high_curvature_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_continuity_envelope() {
        Some(envelope) => format_lunar_high_curvature_continuity_evidence_envelope(&envelope),
        _ => "lunar high-curvature continuity evidence: unavailable".to_string(),
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

impl LunarHighCurvatureEquatorialContinuityEnvelope {
    /// Returns the compact release-facing summary line for this equatorial evidence slice.
    fn summary_line(&self) -> String {
        format!(
            "lunar high-curvature equatorial continuity evidence: {} samples across {} bodies, epoch range {}, max adjacent ΔRA={:.12}° ({} → {}), max adjacent ΔDec={:.12}° ({} → {}), max adjacent Δdist={:.12} AU ({} → {}), regression limits: ΔRA≤{:.1}°, ΔDec≤{:.1}°, Δdist≤{:.2} AU; within regression limits={}",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_right_ascension_step_deg,
            format_instant(self.max_right_ascension_step_start_epoch),
            format_instant(self.max_right_ascension_step_end_epoch),
            self.max_declination_step_deg,
            format_instant(self.max_declination_step_start_epoch),
            format_instant(self.max_declination_step_end_epoch),
            self.max_distance_step_au,
            format_instant(self.max_distance_step_start_epoch),
            format_instant(self.max_distance_step_end_epoch),
            LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG,
            LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG,
            LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU,
            self.within_regression_limits,
        )
    }

    /// Returns the compact summary line after validating the equatorial continuity evidence slice.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, LunarHighCurvatureEvidenceValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    fn validate(&self) -> Result<(), LunarHighCurvatureEvidenceValidationError> {
        validate_high_curvature_continuity_window(
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            [
                (
                    "max_right_ascension_step_deg",
                    self.max_right_ascension_step_start_epoch,
                    self.max_right_ascension_step_end_epoch,
                    self.max_right_ascension_step_deg,
                ),
                (
                    "max_declination_step_deg",
                    self.max_declination_step_start_epoch,
                    self.max_declination_step_end_epoch,
                    self.max_declination_step_deg,
                ),
                (
                    "max_distance_step_au",
                    self.max_distance_step_start_epoch,
                    self.max_distance_step_end_epoch,
                    self.max_distance_step_au,
                ),
            ],
        )?;

        let expected_within_regression_limits = self.max_right_ascension_step_deg
            <= LUNAR_HIGH_CURVATURE_RIGHT_ASCENSION_LIMIT_DEG
            && self.max_declination_step_deg <= LUNAR_HIGH_CURVATURE_DECLINATION_LIMIT_DEG
            && self.max_distance_step_au <= LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU;
        if self.within_regression_limits != expected_within_regression_limits {
            return Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature equatorial continuity evidence",
                    within_regression_limits: self.within_regression_limits,
                    expected_within_regression_limits,
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for LunarHighCurvatureEquatorialContinuityEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn lunar_high_curvature_equatorial_continuity_envelope(
) -> Option<LunarHighCurvatureEquatorialContinuityEnvelope> {
    let backend = ElpBackend::new();
    let mut bodies = std::collections::BTreeSet::new();
    let requests = lunar_high_curvature_equatorial_continuity_requests();
    let sample_count = requests.len();
    let mut earliest_epoch = requests[0].instant;
    let mut latest_epoch = requests[0].instant;
    let mut previous_sample: Option<(Instant, f64, f64, f64)> = None;
    let mut max_right_ascension_step_start_epoch = requests[0].instant;
    let mut max_right_ascension_step_end_epoch = requests[0].instant;
    let mut max_right_ascension_step_deg = 0.0;
    let mut max_declination_step_start_epoch = requests[0].instant;
    let mut max_declination_step_end_epoch = requests[0].instant;
    let mut max_declination_step_deg = 0.0;
    let mut max_distance_step_start_epoch = requests[0].instant;
    let mut max_distance_step_end_epoch = requests[0].instant;
    let mut max_distance_step_au = 0.0;
    let mut within_regression_limits = true;

    for request in &requests {
        let instant = request.instant;
        let result = backend
            .position(request)
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
        sample_count,
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
    match envelope.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(_) => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
    }
}

/// Returns the release-facing lunar high-curvature equatorial continuity evidence string.
pub fn lunar_high_curvature_equatorial_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_equatorial_continuity_envelope() {
        Some(envelope) => {
            format_lunar_high_curvature_equatorial_continuity_evidence_envelope(&envelope)
        }
        _ => "lunar high-curvature equatorial continuity evidence: unavailable".to_string(),
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
    /// Median absolute longitude delta in degrees across the evidence slice.
    pub median_longitude_delta_deg: f64,
    /// 95th-percentile absolute longitude delta in degrees across the evidence slice.
    pub percentile_longitude_delta_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Epoch for the maximum absolute latitude delta.
    pub max_latitude_delta_epoch: Instant,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Mean absolute latitude delta in degrees across the evidence slice.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute latitude delta in degrees across the evidence slice.
    pub median_latitude_delta_deg: f64,
    /// 95th-percentile absolute latitude delta in degrees across the evidence slice.
    pub percentile_latitude_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: Option<CelestialBody>,
    /// Epoch for the maximum absolute distance delta.
    pub max_distance_delta_epoch: Option<Instant>,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: Option<f64>,
    /// Mean absolute distance delta in astronomical units across the samples that include distance.
    pub mean_distance_delta_au: Option<f64>,
    /// Median absolute distance delta in astronomical units across the samples that include distance.
    pub median_distance_delta_au: Option<f64>,
    /// 95th-percentile absolute distance delta in astronomical units across the samples that include distance.
    pub percentile_distance_delta_au: Option<f64>,
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
    let mut longitude_deltas = Vec::with_capacity(samples.len());
    let mut max_latitude_delta_body = samples[0].body.clone();
    let mut max_latitude_delta_epoch = samples[0].epoch;
    let mut max_latitude_delta_deg = 0.0;
    let mut total_latitude_delta_deg = 0.0;
    let mut latitude_deltas = Vec::with_capacity(samples.len());
    let mut outlier_bodies: Vec<CelestialBody> = Vec::new();
    let mut max_distance_delta_body = None;
    let mut max_distance_delta_epoch = None;
    let mut max_distance_delta_au = None;
    let mut total_distance_delta_au = 0.0;
    let mut distance_deltas = Vec::new();
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
        longitude_deltas.push(longitude_delta_deg);
        total_latitude_delta_deg += latitude_delta_deg;
        latitude_deltas.push(latitude_delta_deg);
        if let Some(delta) = distance_delta_au {
            total_distance_delta_au += delta;
            distance_deltas.push(delta);
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

    let mut longitude_deltas_for_median = longitude_deltas.clone();
    let mut longitude_deltas_for_percentile = longitude_deltas;
    let mut latitude_deltas_for_median = latitude_deltas.clone();
    let mut latitude_deltas_for_percentile = latitude_deltas;
    let mut distance_deltas_for_median = distance_deltas.clone();
    let mut distance_deltas_for_percentile = distance_deltas;

    Some(LunarReferenceEvidenceEnvelope {
        sample_count: samples.len(),
        body_count: bodies.len(),
        earliest_epoch,
        latest_epoch,
        max_longitude_delta_body,
        max_longitude_delta_epoch,
        max_longitude_delta_deg,
        mean_longitude_delta_deg: total_longitude_delta_deg / samples.len() as f64,
        median_longitude_delta_deg: median_value(&mut longitude_deltas_for_median)
            .unwrap_or_default(),
        percentile_longitude_delta_deg: percentile_value(
            &mut longitude_deltas_for_percentile,
            0.95,
        )
        .unwrap_or_default(),
        max_latitude_delta_body,
        max_latitude_delta_epoch,
        max_latitude_delta_deg,
        mean_latitude_delta_deg: total_latitude_delta_deg / samples.len() as f64,
        median_latitude_delta_deg: median_value(&mut latitude_deltas_for_median)
            .unwrap_or_default(),
        percentile_latitude_delta_deg: percentile_value(&mut latitude_deltas_for_percentile, 0.95)
            .unwrap_or_default(),
        max_distance_delta_body,
        max_distance_delta_epoch,
        max_distance_delta_au,
        mean_distance_delta_au: (distance_delta_sample_count > 0)
            .then_some(total_distance_delta_au / distance_delta_sample_count as f64),
        median_distance_delta_au: median_value(&mut distance_deltas_for_median),
        percentile_distance_delta_au: percentile_value(&mut distance_deltas_for_percentile, 0.95),
        outside_current_limits_count,
        outlier_bodies,
        within_current_limits,
    })
}

impl LunarReferenceEvidenceEnvelope {
    /// Returns `Ok(())` when the reference envelope remains internally consistent.
    pub(crate) fn validate(&self) -> Result<(), LunarEvidenceEnvelopeValidationError> {
        validate_lunar_evidence_envelope(
            "lunar reference error envelope",
            self.sample_count,
            self.body_count,
            self.earliest_epoch,
            self.latest_epoch,
            &[
                ("max_longitude_delta_deg", self.max_longitude_delta_deg),
                ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
                (
                    "median_longitude_delta_deg",
                    self.median_longitude_delta_deg,
                ),
                (
                    "percentile_longitude_delta_deg",
                    self.percentile_longitude_delta_deg,
                ),
                ("max_latitude_delta_deg", self.max_latitude_delta_deg),
                ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
                ("median_latitude_delta_deg", self.median_latitude_delta_deg),
                (
                    "percentile_latitude_delta_deg",
                    self.percentile_latitude_delta_deg,
                ),
            ],
            &[
                ("max_distance_delta_au", self.max_distance_delta_au),
                ("mean_distance_delta_au", self.mean_distance_delta_au),
                ("median_distance_delta_au", self.median_distance_delta_au),
                (
                    "percentile_distance_delta_au",
                    self.percentile_distance_delta_au,
                ),
            ],
            &self.outlier_bodies,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }

    /// Returns the release-facing one-line lunar reference error envelope.
    pub fn summary_line(&self) -> String {
        fn format_body_epoch(body: &CelestialBody, epoch: Instant) -> String {
            format!("{} @ {}", body, format_instant(epoch))
        }

        let distance = match (
            self.max_distance_delta_body.as_ref(),
            self.max_distance_delta_epoch,
            self.max_distance_delta_au,
        ) {
            (Some(body), Some(epoch), Some(delta)) => {
                format!(
                    "; max Δdist={delta:.12} AU ({})",
                    format_body_epoch(body, epoch)
                )
            }
            _ => String::new(),
        };
        let mean_distance = self
            .mean_distance_delta_au
            .map(|value| format!("; mean Δdist={value:.12} AU"))
            .unwrap_or_default();
        let median_distance = self
            .median_distance_delta_au
            .map(|value| format!("; median Δdist={value:.12} AU"))
            .unwrap_or_default();
        let percentile_distance = self
            .percentile_distance_delta_au
            .map(|value| format!("; p95 Δdist={value:.12} AU"))
            .unwrap_or_default();
        let limit_note = "; limits: Δlon≤1e-4° (1e-1° for mean node), Δlat≤1e-4°, Δdist≤1e-8 AU";
        let outlier_note = if self.outlier_bodies.is_empty() {
            "; outliers=none".to_string()
        } else {
            format!("; outliers={}", format_bodies(&self.outlier_bodies))
        };

        format!(
            "lunar reference error envelope: {} samples across {} bodies, epoch range {}, max Δlon={:.12}° ({}), mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, max Δlat={:.12}° ({}), mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°{}{}{}{}{}{}; outside current limits={}; within current limits={}",
            self.sample_count,
            self.body_count,
            format_epoch_range(self.earliest_epoch, self.latest_epoch),
            self.max_longitude_delta_deg,
            format_body_epoch(&self.max_longitude_delta_body, self.max_longitude_delta_epoch),
            self.mean_longitude_delta_deg,
            self.median_longitude_delta_deg,
            self.percentile_longitude_delta_deg,
            self.max_latitude_delta_deg,
            format_body_epoch(&self.max_latitude_delta_body, self.max_latitude_delta_epoch),
            self.mean_latitude_delta_deg,
            self.median_latitude_delta_deg,
            self.percentile_latitude_delta_deg,
            distance,
            mean_distance,
            median_distance,
            percentile_distance,
            limit_note,
            outlier_note,
            self.outside_current_limits_count,
            self.within_current_limits,
        )
    }
}

impl fmt::Display for LunarReferenceEvidenceEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Formats the lunar reference error envelope for release-facing reporting.
pub fn format_lunar_reference_evidence_envelope(
    envelope: &LunarReferenceEvidenceEnvelope,
) -> String {
    envelope.summary_line()
}

/// Returns the release-facing lunar reference error envelope string.
pub fn lunar_reference_evidence_envelope_for_report() -> String {
    match lunar_reference_evidence_envelope() {
        Some(envelope) => match envelope.validate() {
            Ok(()) => format_lunar_reference_evidence_envelope(&envelope),
            Err(error) => {
                format!("lunar reference error envelope: unavailable ({error})")
            }
        },
        None => "lunar reference error envelope: unavailable".to_string(),
    }
}

// ElpBackend is defined in backend.rs and re-exported above.

#[cfg(test)]
mod tests;
