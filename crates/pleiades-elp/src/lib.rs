//! Lunar backend boundary based on a compact pure-Rust analytical model.
//!
//! The full ELP series data is still planned, but this crate now provides a
//! usable Moon-and-lunar-point backend for chart workflows by combining a
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
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, FrameTreatmentSummary, QualityAnnotation,
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
    envelope.summary_line()
}

/// Returns the release-facing lunar high-curvature continuity evidence string.
pub fn lunar_high_curvature_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_continuity_envelope() {
        Some(envelope) if envelope.validate().is_ok() => {
            format_lunar_high_curvature_continuity_evidence_envelope(&envelope)
        }
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
    envelope.summary_line()
}

/// Returns the release-facing lunar high-curvature equatorial continuity evidence string.
pub fn lunar_high_curvature_equatorial_continuity_evidence_for_report() -> String {
    match lunar_high_curvature_equatorial_continuity_envelope() {
        Some(envelope) if envelope.validate().is_ok() => {
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
            .to_equatorial(instant.mean_obliquity())
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
                    lunar_theory_source_family_summary_for_report(),
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

        validate_zodiac_policy(req, "the ELP backend", &[ZodiacMode::Tropical])?;

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
        assert_eq!(theory.summary_line(), summary);
        assert_eq!(theory.to_string(), summary);
        let source = theory.source_selection();
        let source_selection = lunar_theory_source_selection();
        assert_eq!(source_selection, source);
        assert!(summary.contains(theory.model_name));
        assert!(summary.contains(theory.source_identifier));
        assert!(summary.contains(theory.source_family.label()));
        assert!(summary
            .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
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
        let family_summary = lunar_theory_source_family_summary();
        assert_eq!(family_summary.family, theory.source_family);
        assert_eq!(family_summary.family_label, theory.source_family.label());
        assert_eq!(
            family_summary.selected_source_identifier,
            theory.source_identifier
        );
        assert_eq!(family_summary.selected_model_name, theory.model_name);
        assert_eq!(
            family_summary.selected_catalog_key,
            LunarTheoryCatalogKey::SourceIdentifier(theory.source_identifier)
        );
        assert_eq!(
            family_summary.selected_family_key,
            LunarTheoryCatalogKey::SourceFamily(theory.source_family)
        );
        assert_eq!(
            family_summary.selected_alias_count,
            theory.source_aliases.len()
        );
        assert_eq!(family_summary.summary_line(), family_summary.to_string());
        assert_eq!(
            lunar_theory_source_family_summary_for_report(),
            family_summary.summary_line()
        );
        assert_eq!(
            source.catalog_key(),
            LunarTheoryCatalogKey::SourceIdentifier(theory.source_identifier)
        );
        assert_eq!(
            source.family_key(),
            LunarTheoryCatalogKey::SourceFamily(theory.source_family)
        );
        assert_eq!(
            source.catalog_key().to_string(),
            "source identifier=meeus-style-truncated-lunar-baseline"
        );
        assert_eq!(
            source.family_key().to_string(),
            "source family=Meeus-style truncated analytical baseline"
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
        assert_eq!(source_summary.source_family, theory.source_family);
        assert_eq!(
            source_summary.catalog_key,
            theory.source_selection().catalog_key()
        );
        assert_eq!(
            source_summary.source_family_key,
            theory.source_selection().family_key()
        );
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
            source_summary.summary_line(),
            format_lunar_theory_source_summary(&source_summary)
        );
        assert_eq!(source_summary.to_string(), source_summary.summary_line());
        assert!(source_summary.validate().is_ok());
        assert_eq!(
            source_summary.validated_summary_line().unwrap(),
            source_summary.summary_line()
        );
        assert_eq!(
            format_lunar_theory_source_summary(&source_summary),
            lunar_theory_source_summary_for_report()
        );
        let mut drifted_source_summary = source_summary;
        drifted_source_summary.source_identifier = "not-the-current-selection";
        let error = drifted_source_summary
            .validate()
            .expect_err("drifted summary should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar source summary field `source_identifier` is out of sync with the current selection"
        );
        assert_eq!(
            drifted_source_summary.validated_summary_line().unwrap_err().to_string(),
            "the lunar source summary field `source_identifier` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_source_summary),
            "lunar source selection: unavailable (the lunar source summary field `source_identifier` is out of sync with the current selection)"
        );
        let drifted_catalog_key = LunarTheorySourceSummary {
            catalog_key: LunarTheoryCatalogKey::SourceFamily(source.family),
            ..source_summary
        };
        let error = drifted_catalog_key
            .validate()
            .expect_err("drifted catalog key should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar source summary field `catalog_key` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_catalog_key),
            "lunar source selection: unavailable (the lunar source summary field `catalog_key` is out of sync with the current selection)"
        );
        let drifted_family_key = LunarTheorySourceSummary {
            source_family_key: LunarTheoryCatalogKey::SourceIdentifier(source.identifier),
            ..source_summary
        };
        let error = drifted_family_key
            .validate()
            .expect_err("drifted family key should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar source summary field `source_family_key` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_family_key),
            "lunar source selection: unavailable (the lunar source summary field `source_family_key` is out of sync with the current selection)"
        );
        let drifted_family_label = LunarTheorySourceSummary {
            source_family_label: "Drifted family label",
            ..source_summary
        };
        let error = drifted_family_label
            .validate()
            .expect_err("drifted family label should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar source summary field `source_family_label` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_family_label),
            "lunar source selection: unavailable (the lunar source summary field `source_family_label` is out of sync with the current selection)"
        );
        assert!(lunar_theory_source_summary_for_report().contains("lunar source selection: "));
        assert!(lunar_theory_source_summary_for_report()
            .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
        assert!(lunar_theory_source_summary_for_report()
            .contains("family key: source family=Meeus-style truncated analytical baseline"));
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
        assert_eq!(
            theory.request_policy.to_string(),
            theory.request_policy.summary_line()
        );
        assert!(theory.request_policy.validate().is_ok());
        assert_eq!(
            format_validated_lunar_theory_request_policy_for_report(&theory.request_policy),
            theory.request_policy.summary_line()
        );

        let mut drifted_request_policy = theory.request_policy;
        drifted_request_policy.supported_time_scales = &[TimeScale::Tt];
        let error = drifted_request_policy
            .validate()
            .expect_err("drifted request policy should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar theory request policy field `supported_time_scales` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_request_policy_for_report(&drifted_request_policy),
            "lunar theory request policy: unavailable (the lunar theory request policy field `supported_time_scales` is out of sync with the current selection)"
        );
        assert_eq!(lunar_theory_summary_for_report(), theory.summary_line());
        assert!(summary.contains("frames=Ecliptic, Equatorial"));
        assert!(summary.contains("time scales=TT, TDB"));
        assert!(summary.contains("zodiac modes=Tropical"));
        assert!(summary.contains("apparentness=Mean"));
        assert!(summary.contains("topocentric observer=false"));

        let mut drifted_spec = theory;
        drifted_spec.request_policy = LunarTheoryRequestPolicy {
            supported_time_scales: &[TimeScale::Tt],
            ..drifted_spec.request_policy
        };
        let error = drifted_spec
            .validate()
            .expect_err("drifted specification should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar theory specification field `request_policy.supported_time_scales` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_specification_for_report(&drifted_spec),
            "ELP lunar theory specification: unavailable (the lunar theory specification field `request_policy.supported_time_scales` is out of sync with the current selection)"
        );

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
        assert_eq!(catalog_summary.selected_source_family, theory.source_family);
        assert_eq!(
            catalog_summary.selected_source_family_label,
            theory.source_family.label()
        );
        assert_eq!(catalog_summary.selected_catalog_key, source.catalog_key());
        assert_eq!(catalog_summary.selected_family_key, source.family_key());
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
        assert_eq!(catalog_summary.summary_line(), catalog_summary.to_string());
        assert_eq!(
            catalog_summary.validated_summary_line().unwrap(),
            catalog_summary.summary_line()
        );
        assert_eq!(
            format_lunar_theory_catalog_summary(&catalog_summary),
            catalog_summary.summary_line()
        );
        assert_eq!(
            lunar_theory_catalog_summary_for_report(),
            catalog_summary.summary_line()
        );
        let catalog_entry_summary = catalog[0].summary_line();
        assert_eq!(catalog_entry_summary, catalog[0].to_string());
        assert_eq!(
            catalog[0].validated_summary_line().unwrap(),
            catalog_entry_summary
        );
        assert!(catalog_entry_summary.contains("lunar theory catalog entry: selected=true"));
        assert!(catalog_entry_summary.contains(
            "source=meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]"
        ));
        assert!(catalog_entry_summary
            .contains("key=source identifier=meeus-style-truncated-lunar-baseline"));
        assert!(catalog_entry_summary.contains(&format!("aliases={}", theory.source_aliases.len())));
        assert!(catalog_entry_summary.contains(&format!(
            "supported bodies={}",
            theory.supported_bodies.len()
        )));
        assert!(catalog_entry_summary.contains(&format!(
            "unsupported bodies={}",
            theory.unsupported_bodies.len()
        )));
        let mut drifted_catalog_entry = catalog[0];
        drifted_catalog_entry.selected = false;
        let drifted_catalog_entry_summary = drifted_catalog_entry.summary_line();
        assert!(drifted_catalog_entry_summary.contains("selected=false"));
        let drifted_catalog_entry_error = drifted_catalog_entry
            .validated_summary_line()
            .expect_err("unselected catalog entry should fail validation");
        assert_eq!(
            drifted_catalog_entry_error.to_string(),
            "the lunar-theory catalog has no selected entry"
        );
        assert!(catalog[0].validate().is_ok());
        let mut drifted_specification = theory;
        drifted_specification.model_name = "Drifted lunar baseline";
        let spec_error = drifted_specification
            .validate()
            .expect_err("drifted lunar specification should fail validation");
        assert_eq!(
            spec_error.to_string(),
            "the lunar theory specification field `model_name` is out of sync with the current selection"
        );
        let drifted_catalog_validation =
            validate_lunar_theory_catalog_entries(&[LunarTheoryCatalogEntry {
                selected: true,
                specification: drifted_specification,
            }])
            .expect_err("drifted lunar catalog entry should fail validation");
        assert_eq!(
            drifted_catalog_validation.to_string(),
            "the lunar-theory catalog entry `meeus-style-truncated-lunar-baseline` has specification field `model_name` out of sync with the current catalog"
        );
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("lunar theory catalog: 1 entry, 1 selected entry"));
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
        assert!(lunar_theory_catalog_summary_for_report().contains(
            "selected family key: source family=Meeus-style truncated analytical baseline"
        ));
        assert!(catalog_summary.validate().is_ok());
        let mut drifted_catalog_summary = catalog_summary;
        drifted_catalog_summary.selected_alias_count += 1;
        let error = drifted_catalog_summary
            .validate()
            .expect_err("drifted catalog summary should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog"
        );
        assert_eq!(
            drifted_catalog_summary.validated_summary_line().unwrap_err().to_string(),
            "the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog"
        );
        assert_eq!(
            format_validated_lunar_theory_catalog_summary_for_report(&drifted_catalog_summary),
            "lunar theory catalog: unavailable (the lunar catalog summary field `selected_alias_count` is out of sync with the current catalog)"
        );
        let drifted_catalog_key = LunarTheoryCatalogSummary {
            selected_catalog_key: LunarTheoryCatalogKey::SourceFamily(source.family),
            ..catalog_summary
        };
        let catalog_key_error = drifted_catalog_key
            .validate()
            .expect_err("drifted catalog key should fail validation");
        assert_eq!(
            catalog_key_error.to_string(),
            "the lunar catalog summary field `selected_catalog_key` is out of sync with the current catalog"
        );
        assert_eq!(
            format_validated_lunar_theory_catalog_summary_for_report(&drifted_catalog_key),
            "lunar theory catalog: unavailable (the lunar catalog summary field `selected_catalog_key` is out of sync with the current catalog)"
        );
        let drifted_family_key = LunarTheoryCatalogSummary {
            selected_family_key: LunarTheoryCatalogKey::SourceIdentifier(source.identifier),
            ..catalog_summary
        };
        let family_key_error = drifted_family_key
            .validate()
            .expect_err("drifted family key should fail validation");
        assert_eq!(
            family_key_error.to_string(),
            "the lunar catalog summary field `selected_family_key` is out of sync with the current catalog"
        );
        assert_eq!(
            format_validated_lunar_theory_catalog_summary_for_report(&drifted_family_key),
            "lunar theory catalog: unavailable (the lunar catalog summary field `selected_family_key` is out of sync with the current catalog)"
        );
        let catalog_validation_summary = lunar_theory_catalog_validation_summary();
        assert_eq!(catalog_validation_summary.entry_count, 1);
        assert_eq!(catalog_validation_summary.selected_count, 1);
        assert_eq!(catalog_validation_summary.selected_source, Some(source));
        assert!(catalog_validation_summary.validation_result.is_ok());
        assert_eq!(
            catalog_validation_summary.summary_line(),
            format_lunar_theory_catalog_validation_summary(&catalog_validation_summary)
        );
        assert_eq!(
            catalog_validation_summary.to_string(),
            catalog_validation_summary.summary_line()
        );
        assert_eq!(
            format_lunar_theory_catalog_validation_summary(&catalog_validation_summary),
            lunar_theory_catalog_validation_summary_for_report()
        );
        assert!(lunar_theory_catalog_validation_summary_for_report()
            .contains("lunar theory catalog validation: ok (1 entries, 1 selected; selected source: meeus-style-truncated-lunar-baseline [Meeus-style truncated analytical baseline]; selected key: source identifier=meeus-style-truncated-lunar-baseline; selected family key: source family=Meeus-style truncated analytical baseline; aliases=1; specification sync, round-trip, alias uniqueness, body coverage disjointness, and case-insensitive key matching verified)"));
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("selected source: meeus-style-truncated-lunar-baseline"));
        assert!(lunar_theory_catalog_summary_for_report()
            .contains("aliases=1; supported bodies=5; unsupported bodies=2"));
        assert!(lunar_theory_catalog_validation_summary_for_report().contains("aliases=1"));
        assert!(lunar_theory_catalog_validation_summary_for_report()
            .contains("case-insensitive key matching verified"));
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

        let empty_catalog_error = validate_lunar_theory_catalog_entries(&[])
            .expect_err("empty catalog should fail validation");
        assert_eq!(
            empty_catalog_error.to_string(),
            "the lunar-theory catalog is empty"
        );

        let no_selected_catalog_error =
            validate_lunar_theory_catalog_entries(&[LunarTheoryCatalogEntry {
                selected: false,
                specification: theory,
            }])
            .expect_err("catalog without a selected entry should fail validation");
        assert_eq!(
            no_selected_catalog_error.to_string(),
            "the lunar-theory catalog has no selected entry"
        );
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
    fn lunar_theory_catalog_validation_rejects_duplicate_supported_bodies() {
        const DUPLICATE_SUPPORTED_BODIES: &[CelestialBody] =
            &[CelestialBody::Moon, CelestialBody::Moon];

        let duplicate_supported_catalog = [LunarTheoryCatalogEntry {
            selected: true,
            specification: LunarTheorySpecification {
                supported_bodies: DUPLICATE_SUPPORTED_BODIES,
                ..LUNAR_THEORY_SPECIFICATION
            },
        }];

        assert!(matches!(
            validate_lunar_theory_catalog_entries(&duplicate_supported_catalog),
            Err(LunarTheoryCatalogValidationError::DuplicateSupportedBody {
                body: CelestialBody::Moon
            })
        ));
    }

    #[test]
    fn lunar_theory_catalog_validation_rejects_duplicate_unsupported_bodies() {
        const DUPLICATE_UNSUPPORTED_BODIES: &[CelestialBody] =
            &[CelestialBody::TrueApogee, CelestialBody::TrueApogee];

        let duplicate_unsupported_catalog = [LunarTheoryCatalogEntry {
            selected: true,
            specification: LunarTheorySpecification {
                unsupported_bodies: DUPLICATE_UNSUPPORTED_BODIES,
                ..LUNAR_THEORY_SPECIFICATION
            },
        }];

        assert!(matches!(
            validate_lunar_theory_catalog_entries(&duplicate_unsupported_catalog),
            Err(
                LunarTheoryCatalogValidationError::DuplicateUnsupportedBody {
                    body: CelestialBody::TrueApogee
                }
            )
        ));
    }

    #[test]
    fn lunar_theory_catalog_validation_rejects_supported_and_unsupported_overlaps() {
        const OVERLAPPING_SUPPORTED_BODIES: &[CelestialBody] = &[CelestialBody::Moon];
        const OVERLAPPING_UNSUPPORTED_BODIES: &[CelestialBody] = &[CelestialBody::Moon];

        let overlapping_catalog = [LunarTheoryCatalogEntry {
            selected: true,
            specification: LunarTheorySpecification {
                supported_bodies: OVERLAPPING_SUPPORTED_BODIES,
                unsupported_bodies: OVERLAPPING_UNSUPPORTED_BODIES,
                ..LUNAR_THEORY_SPECIFICATION
            },
        }];

        assert!(matches!(
            validate_lunar_theory_catalog_entries(&overlapping_catalog),
            Err(
                LunarTheoryCatalogValidationError::OverlappingSupportedAndUnsupportedBody {
                    body: CelestialBody::Moon
                }
            )
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
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source == &lunar_theory_source_family_summary_for_report()));
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
    fn lunar_theory_source_selection_has_a_stable_summary_line() {
        let selection = lunar_theory_source_selection();
        let summary = selection.summary_line();

        assert_eq!(selection.to_string(), summary);
        assert_eq!(format_lunar_theory_source_selection(&selection), summary);
        assert_eq!(lunar_theory_source_selection_summary(), summary);
        assert_eq!(
            resolve_lunar_theory_by_key(selection.catalog_key()),
            Some(lunar_theory_specification())
        );
        assert_eq!(
            resolve_lunar_theory_by_key(selection.family_key()),
            Some(lunar_theory_specification())
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_selection(selection).map(|entry| entry.specification),
            Some(lunar_theory_specification())
        );
        assert_eq!(
            lunar_theory_catalog_entry_for_current_selection().map(|entry| entry.specification),
            Some(lunar_theory_specification())
        );
        assert!(summary.contains(selection.identifier));
        assert!(summary
            .contains("selected key: source identifier=meeus-style-truncated-lunar-baseline"));
        assert!(
            summary.contains("family key: source family=Meeus-style truncated analytical baseline")
        );
        assert!(summary.contains(selection.family_label()));
        assert!(summary.contains(selection.citation));
        assert!(summary.contains(selection.license_note));
        assert_eq!(selection.validated_summary_line().unwrap(), summary);
        assert_eq!(selection.validate(), Ok(()));
    }

    #[test]
    fn lunar_theory_source_selection_validation_fails_closed_for_drifted_fields() {
        let selection = lunar_theory_source_selection();
        let drifted = LunarTheorySourceSelection {
            identifier: "not-the-current-selection",
            ..selection
        };

        let error = drifted
            .validate()
            .expect_err("drifted source selection should fail validation");
        assert_eq!(
            error.to_string(),
            "the lunar source selection field `identifier` is out of sync with the current selection"
        );
        assert_eq!(
            drifted.validated_summary_line().unwrap_err().to_string(),
            "the lunar source selection field `identifier` is out of sync with the current selection"
        );
        assert_eq!(
            format_lunar_theory_source_selection(&drifted),
            "lunar source selection: unavailable (the lunar source selection field `identifier` is out of sync with the current selection)"
        );
        assert_eq!(
            lunar_theory_source_selection_summary(),
            selection.summary_line()
        );
    }

    #[test]
    fn lunar_theory_source_summary_validation_fails_closed_for_drifted_keys() {
        let source_summary = lunar_theory_source_summary();

        let drifted_catalog_key = LunarTheorySourceSummary {
            catalog_key: LunarTheoryCatalogKey::SourceFamily(source_summary.source_family),
            ..source_summary
        };
        let catalog_error = drifted_catalog_key
            .validate()
            .expect_err("drifted catalog key should fail validation");
        assert_eq!(
            catalog_error.to_string(),
            "the lunar source summary field `catalog_key` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_catalog_key),
            "lunar source selection: unavailable (the lunar source summary field `catalog_key` is out of sync with the current selection)"
        );

        let drifted_family_key = LunarTheorySourceSummary {
            source_family_key: LunarTheoryCatalogKey::SourceIdentifier(
                source_summary.source_identifier,
            ),
            ..source_summary
        };
        let family_error = drifted_family_key
            .validate()
            .expect_err("drifted family key should fail validation");
        assert_eq!(
            family_error.to_string(),
            "the lunar source summary field `source_family_key` is out of sync with the current selection"
        );
        assert_eq!(
            format_validated_lunar_theory_source_summary_for_report(&drifted_family_key),
            "lunar source selection: unavailable (the lunar source summary field `source_family_key` is out of sync with the current selection)"
        );

        assert_eq!(
            lunar_theory_source_summary_for_report(),
            source_summary.summary_line()
        );
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
    fn published_apparent_moon_example_matches_the_shared_mean_obliquity_transform() {
        let sample = lunar_apparent_comparison_evidence()
            .iter()
            .find(|sample| (sample.epoch.julian_day.days() - 2_453_100.5).abs() < f64::EPSILON)
            .expect("NASA RP 1349 sample should exist");

        let equatorial = EquatorialCoordinates::new(
            Angle::from_degrees(sample.apparent_right_ascension_deg),
            Latitude::from_degrees(sample.apparent_declination_deg),
            Some(sample.apparent_distance_au),
        );
        let ecliptic = equatorial.to_ecliptic(sample.epoch.mean_obliquity());

        assert!((ecliptic.longitude.degrees() - sample.apparent_longitude_deg).abs() < 1e-6);
        assert!((ecliptic.latitude.degrees() - sample.apparent_latitude_deg).abs() < 1e-6);
        assert!(
            (ecliptic
                .distance_au
                .expect("apparent Moon distance should exist")
                - sample.apparent_distance_au)
                .abs()
                < 1e-12
        );
        assert!(sample.note.contains("NASA RP 1349"));
        assert!(sample.note.contains("shared mean-obliquity transform"));
    }

    #[test]
    fn published_apparent_moon_comparison_datum_matches_the_reference_slice() {
        let sample = lunar_apparent_comparison_evidence()
            .iter()
            .find(|sample| (sample.epoch.julian_day.days() - 2_448_724.5).abs() < f64::EPSILON)
            .expect("1992 apparent Moon sample should exist");

        assert_eq!(sample.body, CelestialBody::Moon);
        assert_eq!(sample.epoch.julian_day.days(), 2_448_724.5);
        assert!((sample.apparent_longitude_deg - 133.167_264).abs() < 1e-12);
        assert!((sample.apparent_latitude_deg - (-3.229_126)).abs() < 1e-12);
        assert!((sample.apparent_distance_au - (368_409.7 / 149_597_870.700)).abs() < 1e-12);
        assert!((sample.apparent_right_ascension_deg - 134.688_469).abs() < 1e-12);
        assert!((sample.apparent_declination_deg - 13.768_367).abs() < 1e-12);
        assert!(sample
            .note
            .contains("1992-04-12 apparent geocentric Moon example"));
        assert!(sample.note.contains("mean/apparent comparison datum"));
    }

    #[test]
    fn published_eclipsewise_apparent_moon_example_matches_the_shared_mean_obliquity_transform() {
        let sample = lunar_apparent_comparison_evidence()
            .iter()
            .find(|sample| {
                (sample.epoch.julian_day.days() - 2_453_986.285_649).abs() < f64::EPSILON
            })
            .expect("EclipseWise apparent Moon sample should exist");

        let equatorial = EquatorialCoordinates::new(
            Angle::from_degrees(sample.apparent_right_ascension_deg),
            Latitude::from_degrees(sample.apparent_declination_deg),
            Some(sample.apparent_distance_au),
        );
        let ecliptic = equatorial.to_ecliptic(sample.epoch.mean_obliquity());

        assert!((ecliptic.longitude.degrees() - sample.apparent_longitude_deg).abs() < 1e-6);
        assert!((ecliptic.latitude.degrees() - sample.apparent_latitude_deg).abs() < 1e-6);
        assert!(
            (ecliptic
                .distance_au
                .expect("apparent Moon distance should exist")
                - sample.apparent_distance_au)
                .abs()
                < 1e-12
        );
        assert!(sample.note.contains("EclipseWise"));
        assert!(sample.note.contains("shared mean-obliquity transform"));
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
        let envelope = lunar_high_curvature_continuity_envelope().expect("envelope should exist");
        let report = lunar_high_curvature_continuity_evidence_for_report();

        assert_eq!(report, envelope.summary_line());
        assert!(envelope.validate().is_ok());
        assert!(
            report.contains("lunar high-curvature continuity evidence: 6 samples across 1 bodies")
        );
        assert!(report.contains("epoch range JD 2451544.0 (TT) → JD 2451547.0 (TT)"));
        assert!(report.contains("max adjacent Δlon="));
        assert!(report.contains("max adjacent Δlat="));
        assert!(report.contains("max adjacent Δdist="));
        assert!(report.contains("within regression limits=true"));
        assert!(report.contains("Δlon≤20.0°"));
        assert!(report.contains("Δlat≤10.0°"));
        assert!(report.contains("Δdist≤0.02 AU"));
    }

    #[test]
    fn lunar_high_curvature_request_corpus_helpers_match_the_regression_window() {
        let ecliptic_requests = lunar_high_curvature_continuity_requests();
        let equatorial_requests = lunar_high_curvature_equatorial_continuity_requests();

        assert_eq!(
            ecliptic_requests,
            lunar_high_curvature_continuity_request_corpus()
        );
        assert_eq!(
            ecliptic_requests,
            lunar_high_curvature_continuity_batch_parity_requests()
        );
        assert_eq!(
            ecliptic_requests,
            lunar_high_curvature_continuity_batch_parity_request_corpus()
        );
        assert_eq!(
            equatorial_requests,
            lunar_high_curvature_equatorial_continuity_request_corpus()
        );
        assert_eq!(
            equatorial_requests,
            lunar_high_curvature_equatorial_continuity_batch_parity_requests()
        );
        assert_eq!(
            equatorial_requests,
            lunar_high_curvature_equatorial_continuity_batch_parity_request_corpus()
        );

        for (request, expected_epoch) in ecliptic_requests
            .iter()
            .zip(LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS)
        {
            assert_eq!(request.body, CelestialBody::Moon);
            assert_eq!(request.instant, expected_epoch);
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }

        for (request, expected_epoch) in equatorial_requests
            .iter()
            .zip(LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS)
        {
            assert_eq!(request.body, CelestialBody::Moon);
            assert_eq!(request.instant, expected_epoch);
            assert_eq!(request.frame, CoordinateFrame::Equatorial);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
        }
    }

    #[test]
    fn batch_query_preserves_lunar_high_curvature_continuity_order_and_values() {
        let backend = ElpBackend::new();
        let requests = lunar_high_curvature_continuity_batch_parity_requests();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the lunar high-curvature order");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.frame, CoordinateFrame::Ecliptic);

            let batch = result.ecliptic.expect("ecliptic result should exist");
            let single = backend
                .position(request)
                .expect("single high-curvature query should succeed")
                .ecliptic
                .expect("single-query ecliptic result should exist");

            assert_eq!(batch, single);
        }
    }

    #[test]
    fn batch_query_preserves_lunar_high_curvature_equatorial_order_and_values() {
        let backend = ElpBackend::new();
        let requests = lunar_high_curvature_equatorial_continuity_batch_parity_requests();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the lunar high-curvature equatorial order");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);

            let batch = result.equatorial.expect("equatorial result should exist");
            let single = backend
                .position(request)
                .expect("single high-curvature equatorial query should succeed")
                .equatorial
                .expect("single-query equatorial result should exist");

            assert_eq!(batch, single);
        }
    }

    #[test]
    fn lunar_high_curvature_continuity_validation_rejects_stale_counts() {
        let mut envelope =
            lunar_high_curvature_continuity_envelope().expect("envelope should exist");
        envelope.sample_count = 1;

        assert!(matches!(
            envelope.validate(),
            Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
        ));
    }

    #[test]
    fn lunar_high_curvature_continuity_validation_rejects_stale_regression_limit_flag() {
        let mut envelope =
            lunar_high_curvature_continuity_envelope().expect("envelope should exist");
        envelope.within_regression_limits = false;

        assert!(matches!(
            envelope.validate(),
            Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature continuity evidence",
                    within_regression_limits: false,
                    expected_within_regression_limits: true,
                }
            )
        ));
    }

    #[test]
    fn lunar_high_curvature_equatorial_continuity_evidence_is_rendered() {
        let envelope = lunar_high_curvature_equatorial_continuity_envelope()
            .expect("equatorial envelope should exist");
        let report = lunar_high_curvature_equatorial_continuity_evidence_for_report();

        assert_eq!(report, envelope.summary_line());
        assert!(envelope.validate().is_ok());
        assert!(report.contains(
            "lunar high-curvature equatorial continuity evidence: 6 samples across 1 bodies"
        ));
        assert!(report.contains("epoch range JD 2451544.0 (TT) → JD 2451547.0 (TT)"));
        assert!(report.contains("max adjacent ΔRA="));
        assert!(report.contains("max adjacent ΔDec="));
        assert!(report.contains("max adjacent Δdist="));
        assert!(report.contains("within regression limits=true"));
        assert!(report.contains("ΔRA≤20.0°"));
        assert!(report.contains("ΔDec≤10.0°"));
        assert!(report.contains("Δdist≤0.02 AU"));
    }

    #[test]
    fn lunar_high_curvature_equatorial_continuity_validation_rejects_stale_counts() {
        let mut envelope = lunar_high_curvature_equatorial_continuity_envelope()
            .expect("equatorial envelope should exist");
        envelope.sample_count = 1;

        assert!(matches!(
            envelope.validate(),
            Err(LunarHighCurvatureEvidenceValidationError::SampleCountTooSmall { sample_count: 1 })
        ));
    }

    #[test]
    fn lunar_high_curvature_equatorial_continuity_validation_rejects_stale_regression_limit_flag() {
        let mut envelope = lunar_high_curvature_equatorial_continuity_envelope()
            .expect("equatorial envelope should exist");
        envelope.within_regression_limits = false;

        assert!(matches!(
            envelope.validate(),
            Err(
                LunarHighCurvatureEvidenceValidationError::RegressionLimitMismatch {
                    envelope: "lunar high-curvature equatorial continuity evidence",
                    within_regression_limits: false,
                    expected_within_regression_limits: true,
                }
            )
        ));
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
    fn batch_query_preserves_lunar_reference_order_for_mixed_time_scales() {
        let backend = ElpBackend::new();
        let evidence = lunar_reference_evidence();
        let requests = evidence
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                let mut request = mean_request_at(sample.body.clone(), sample.epoch);
                request.instant.scale = if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                };
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch mixed-scale query should preserve the lunar reference order");

        assert_eq!(results.len(), evidence.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant.scale, request.instant.scale);
            let single = backend
                .position(request)
                .expect("single mixed-scale query should preserve the lunar reference order");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant.scale, request.instant.scale);
            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let single_ecliptic = single
                .ecliptic
                .expect("single-query ecliptic result should exist");
            assert_eq!(
                ecliptic.longitude.degrees(),
                single_ecliptic.longitude.degrees()
            );
            assert_eq!(
                ecliptic.latitude.degrees(),
                single_ecliptic.latitude.degrees()
            );
            assert_eq!(ecliptic.distance_au, single_ecliptic.distance_au);
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
            let expected = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
            let equatorial = result.equatorial.expect("equatorial result should exist");

            assert_eq!(equatorial, expected);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn batch_query_preserves_mixed_frame_requests_and_values() {
        let backend = ElpBackend::new();
        let evidence = lunar_reference_evidence();
        let requests = evidence
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                let mut request = mean_request_at(sample.body.clone(), sample.epoch);
                request.frame = if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                };
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("mixed frame batch query should preserve the lunar reference order");

        assert_eq!(results.len(), evidence.len());
        for ((request, result), sample) in requests.iter().zip(results.iter()).zip(evidence.iter())
        {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, sample.epoch);
            assert_eq!(result.frame, request.frame);

            let ecliptic = result.ecliptic.expect("ecliptic result should exist");
            let expected = ecliptic.to_equatorial(sample.epoch.mean_obliquity());
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
            .source_material
            .contains("no ELP coefficient files are bundled in the baseline"));
        assert!(theory
            .redistribution_note
            .contains("does not bundle ELP coefficient tables"));
        assert!(theory.license_note.contains("handwritten pure Rust"));
        assert!(theory.truncation_note.contains("truncated"));
        assert!(theory.unit_note.contains("astronomical units"));
        assert!(theory.date_range_note.contains("1992-04-12"));
        assert!(theory
            .date_range_note
            .contains("1968-12-24 apparent geocentric Moon comparison datum"));
        assert!(theory.date_range_note.contains("J2000 lunar-point anchors"));
        assert!(theory
            .date_range_note
            .contains("2021-03-05 mean-perigee example"));
        assert!(theory.date_range_note.contains("1913-05-27 true-node"));
        assert!(theory.frame_note.contains("mean-obliquity"));
        let frame_summary = lunar_theory_frame_treatment_summary_details();
        assert_eq!(frame_summary.to_string(), frame_summary.summary_line());
        assert_eq!(frame_summary.summary_line(), theory.frame_note);
        assert_eq!(
            lunar_theory_frame_treatment_summary_for_report(),
            frame_summary.to_string()
        );
        assert_eq!(
            lunar_theory_frame_treatment_summary(),
            frame_summary.summary_line()
        );
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
        assert_eq!(capability.source_family, theory.source_family);
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
        assert_eq!(
            capability.summary_line(),
            format_lunar_theory_capability_summary(&capability)
        );
        assert_eq!(
            capability.validated_summary_line().unwrap(),
            capability.summary_line()
        );
        assert_eq!(capability.to_string(), capability.summary_line());
        assert_eq!(
            lunar_theory_capability_summary_for_report(),
            capability.summary_line()
        );
        assert_eq!(capability.validate(), Ok(()));
        let mut drifted_helper = capability;
        drifted_helper.catalog_validation_ok = !drifted_helper.catalog_validation_ok;
        assert_eq!(
            drifted_helper.validated_summary_line().unwrap_err().to_string(),
            "the lunar capability summary field `catalog_validation_ok` is out of sync with the current selection"
        );
        let mut drifted = capability;
        drifted.catalog_validation_ok = !drifted.catalog_validation_ok;
        assert_eq!(
            drifted.validate(),
            Err(
                LunarTheoryCapabilitySummaryValidationError::FieldOutOfSync {
                    field: "catalog_validation_ok",
                }
            )
        );
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
    fn lunar_reference_batch_requests_match_the_canonical_slice() {
        let requests = lunar_reference_batch_requests();
        let samples = lunar_reference_evidence();

        assert_eq!(requests.len(), samples.len());

        for (index, (request, sample)) in requests.iter().zip(samples.iter()).enumerate() {
            assert_eq!(
                request.body, sample.body,
                "request {index} body should match evidence"
            );
            assert_eq!(
                request.instant.julian_day, sample.epoch.julian_day,
                "request {index} epoch should match evidence"
            );
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
            if index % 2 == 0 {
                assert_eq!(request.instant.scale, TimeScale::Tt);
            } else {
                assert_eq!(request.instant.scale, TimeScale::Tdb);
            }
        }

        assert_eq!(
            requests
                .iter()
                .filter(|request| request.instant.scale == TimeScale::Tt)
                .count(),
            5
        );
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.instant.scale == TimeScale::Tdb)
                .count(),
            4
        );
    }

    #[test]
    fn lunar_equatorial_reference_batch_requests_match_the_canonical_slice() {
        let requests = lunar_equatorial_reference_batch_requests();
        let samples = lunar_equatorial_reference_evidence();

        assert_eq!(requests.len(), samples.len());

        for (index, (request, sample)) in requests.iter().zip(samples.iter()).enumerate() {
            assert_eq!(
                request.body, sample.body,
                "request {index} body should match evidence"
            );
            assert_eq!(
                request.instant.julian_day, sample.epoch.julian_day,
                "request {index} epoch should match evidence"
            );
            assert_eq!(request.frame, CoordinateFrame::Equatorial);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
            assert_eq!(request.instant.scale, TimeScale::Tt);
        }
    }

    #[test]
    fn lunar_request_corpus_aliases_remain_the_canonical_slices() {
        assert_eq!(
            lunar_reference_batch_parity_requests(),
            lunar_reference_batch_requests()
        );
        assert_eq!(
            lunar_reference_batch_request_corpus(),
            lunar_reference_batch_parity_requests()
        );
        assert_eq!(
            lunar_reference_request_corpus(),
            lunar_reference_batch_requests()
        );
        assert_eq!(
            lunar_reference_batch_parity_request_corpus(),
            lunar_reference_batch_parity_requests()
        );
        assert_eq!(
            lunar_equatorial_reference_batch_parity_requests(),
            lunar_equatorial_reference_batch_requests()
        );
        assert_eq!(
            lunar_equatorial_reference_batch_request_corpus(),
            lunar_equatorial_reference_batch_parity_requests()
        );
        assert_eq!(
            lunar_equatorial_reference_request_corpus(),
            lunar_equatorial_reference_batch_requests()
        );
        assert_eq!(
            lunar_equatorial_reference_batch_parity_request_corpus(),
            lunar_equatorial_reference_batch_parity_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_request_corpus(),
            lunar_apparent_comparison_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_batch_parity_requests(),
            lunar_apparent_comparison_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_batch_parity_request_corpus(),
            lunar_apparent_comparison_batch_parity_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_equatorial_request_corpus(),
            lunar_apparent_comparison_equatorial_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_equatorial_batch_parity_requests(),
            lunar_apparent_comparison_equatorial_requests()
        );
        assert_eq!(
            lunar_apparent_comparison_equatorial_batch_parity_request_corpus(),
            lunar_apparent_comparison_equatorial_batch_parity_requests()
        );
        assert_eq!(
            lunar_high_curvature_request_corpus(),
            lunar_high_curvature_continuity_requests()
        );
        assert_eq!(
            lunar_high_curvature_continuity_request_corpus(),
            lunar_high_curvature_continuity_requests()
        );
        assert_eq!(
            lunar_high_curvature_equatorial_request_corpus(),
            lunar_high_curvature_equatorial_continuity_requests()
        );
        assert_eq!(
            lunar_high_curvature_equatorial_continuity_request_corpus(),
            lunar_high_curvature_equatorial_continuity_requests()
        );
    }

    #[test]
    fn lunar_apparent_comparison_request_corpora_match_the_canonical_slice() {
        let samples = lunar_apparent_comparison_evidence();
        let ecliptic_requests = lunar_apparent_comparison_requests();
        let equatorial_requests = lunar_apparent_comparison_equatorial_requests();

        assert_eq!(ecliptic_requests.len(), samples.len());
        assert_eq!(equatorial_requests.len(), samples.len());

        for (index, (request, sample)) in ecliptic_requests.iter().zip(samples.iter()).enumerate() {
            assert_eq!(
                request.body, sample.body,
                "ecliptic request {index} body should match evidence"
            );
            assert_eq!(
                request.instant.julian_day, sample.epoch.julian_day,
                "ecliptic request {index} epoch should match evidence"
            );
            assert_eq!(request.frame, CoordinateFrame::Ecliptic);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
            assert_eq!(request.instant.scale, TimeScale::Tt);
        }

        for (index, (request, sample)) in equatorial_requests.iter().zip(samples.iter()).enumerate()
        {
            assert_eq!(
                request.body, sample.body,
                "equatorial request {index} body should match evidence"
            );
            assert_eq!(
                request.instant.julian_day, sample.epoch.julian_day,
                "equatorial request {index} epoch should match evidence"
            );
            assert_eq!(request.frame, CoordinateFrame::Equatorial);
            assert_eq!(request.zodiac_mode, ZodiacMode::Tropical);
            assert_eq!(request.apparent, Apparentness::Mean);
            assert!(request.observer.is_none());
            assert_eq!(request.instant.scale, TimeScale::Tt);
        }
    }

    #[test]
    fn lunar_reference_evidence_summary_matches_the_canonical_slice() {
        let summary = lunar_reference_evidence_summary().expect("reference evidence should exist");

        assert_eq!(summary.sample_count, 9);
        assert_eq!(summary.body_count, 5);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_419_914.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_459_278.5);
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            format_lunar_reference_evidence_summary(&summary),
            summary.summary_line()
        );
        assert!(lunar_reference_evidence_summary_for_report().contains("9 samples across 5 bodies"));
        assert!(lunar_reference_evidence_summary_for_report()
            .contains("JD 2419914.5 (TT) → JD 2459278.5 (TT)"));

        let parity = lunar_reference_batch_parity_summary()
            .expect("mixed-scale batch parity evidence should exist");
        assert_eq!(parity.sample_count, 9);
        assert_eq!(parity.body_count, 5);
        assert_eq!(parity.tt_request_count, 5);
        assert_eq!(parity.tdb_request_count, 4);
        assert!(parity.order_preserved);
        assert!(parity.single_query_parity);
        assert_eq!(parity.summary_line(), parity.to_string());
        assert!(parity.validate().is_ok());
        assert_eq!(
            format_lunar_reference_batch_parity_summary(&parity),
            parity.summary_line()
        );
        assert!(lunar_reference_batch_parity_summary_for_report()
            .contains("lunar reference mixed TT/TDB batch parity: 9 requests across 5 bodies"));
        assert!(lunar_reference_batch_parity_summary_for_report().contains("TT requests=5"));
        assert!(lunar_reference_batch_parity_summary_for_report().contains("TDB requests=4"));
        assert!(lunar_reference_batch_parity_summary_for_report().contains("order=preserved"));
        assert!(lunar_reference_batch_parity_summary_for_report()
            .contains("single-query parity=preserved"));

        let equatorial_parity = lunar_equatorial_reference_batch_parity_summary()
            .expect("equatorial batch parity evidence should exist");
        assert_eq!(equatorial_parity.sample_count, 3);
        assert_eq!(equatorial_parity.body_count, 1);
        assert_eq!(equatorial_parity.frame, CoordinateFrame::Equatorial);
        assert!(equatorial_parity.order_preserved);
        assert!(equatorial_parity.single_query_parity);
        assert_eq!(
            equatorial_parity.summary_line(),
            equatorial_parity.to_string()
        );
        assert!(equatorial_parity.validate().is_ok());
        assert_eq!(
            format_lunar_equatorial_reference_batch_parity_summary(&equatorial_parity),
            equatorial_parity.summary_line()
        );
        assert!(lunar_equatorial_reference_batch_parity_summary_for_report()
            .contains("lunar equatorial reference batch parity: 3 requests across 1 bodies, frame=Equatorial, order=preserved, single-query parity=preserved"));

        let envelope = lunar_reference_evidence_envelope().expect("error envelope should exist");
        assert_eq!(envelope.sample_count, summary.sample_count);
        assert_eq!(envelope.body_count, summary.body_count);
        assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
        assert_eq!(envelope.latest_epoch, summary.latest_epoch);
        assert!(envelope.max_longitude_delta_deg.is_finite());
        assert!(envelope.mean_longitude_delta_deg.is_finite());
        assert!(envelope.median_longitude_delta_deg.is_finite());
        assert!(envelope.percentile_longitude_delta_deg.is_finite());
        assert!(envelope.max_latitude_delta_deg.is_finite());
        assert!(envelope.mean_latitude_delta_deg.is_finite());
        assert!(envelope.median_latitude_delta_deg.is_finite());
        assert!(envelope.percentile_latitude_delta_deg.is_finite());
        assert_eq!(envelope.outside_current_limits_count, 0);
        assert!(envelope.within_current_limits);
        assert!(lunar_reference_evidence_envelope_for_report()
            .contains("lunar reference error envelope"));
        assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("median Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("p95 Δlon="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("max Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("mean Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("median Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("p95 Δlat="));
        assert!(lunar_reference_evidence_envelope_for_report().contains("outliers=none"));
        assert!(lunar_reference_evidence_envelope_for_report().contains("limits: Δlon≤1e-4°"));
        assert!(
            lunar_reference_evidence_envelope_for_report().contains("within current limits=true")
        );
        assert!(lunar_reference_evidence_envelope_for_report().contains("outside current limits=0"));
        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert_eq!(
            format_lunar_reference_evidence_envelope(&envelope),
            envelope.summary_line()
        );
    }

    #[test]
    fn lunar_reference_evidence_summary_validates_against_the_checked_in_slice() {
        let summary = lunar_reference_evidence_summary().expect("reference evidence should exist");

        assert!(summary.validate().is_ok());
        assert_eq!(
            lunar_reference_evidence_summary_for_report(),
            summary.summary_line()
        );

        let mut mutated = summary;
        mutated.sample_count += 1;
        let error = mutated
            .validate()
            .expect_err("mutated reference evidence summaries should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("lunar reference evidence summary mismatch"));
        assert!(error.message.contains("expected"));
        assert!(error.message.contains("found"));
    }

    #[test]
    fn lunar_reference_evidence_envelope_validation_rejects_non_finite_metrics() {
        let mut envelope =
            lunar_reference_evidence_envelope().expect("reference error envelope should exist");
        envelope.mean_longitude_delta_deg = f64::NAN;

        let error = envelope
            .validate()
            .expect_err("mutated reference envelopes should fail validation");

        assert_eq!(
            error,
            LunarEvidenceEnvelopeValidationError::NonFiniteMeasure {
                envelope: "lunar reference error envelope",
                field: "mean_longitude_delta_deg",
            }
        );
    }

    #[test]
    fn lunar_reference_batch_parity_summary_validation_rejects_count_drift() {
        let mut parity = lunar_reference_batch_parity_summary()
            .expect("mixed-scale batch parity evidence should exist");
        parity.tt_request_count -= 1;

        let error = parity
            .validate()
            .expect_err("drifted mixed-scale parity evidence should fail validation");

        assert_eq!(
            error,
            LunarReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                field: "tt_request_count"
            }
        );
    }

    #[test]
    fn lunar_equatorial_reference_batch_parity_summary_validation_rejects_count_drift() {
        let mut parity = lunar_equatorial_reference_batch_parity_summary()
            .expect("equatorial batch parity evidence should exist");
        parity.sample_count += 1;

        let error = parity
            .validate()
            .expect_err("drifted equatorial parity evidence should fail validation");

        assert_eq!(
            error,
            LunarEquatorialReferenceBatchParitySummaryValidationError::FieldOutOfSync {
                field: "sample_count"
            }
        );
    }

    #[test]
    fn lunar_equatorial_reference_evidence_matches_the_canonical_slice() {
        let summary = lunar_equatorial_reference_evidence_summary()
            .expect("equatorial reference evidence should exist");

        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.body_count, 1);
        assert_eq!(summary.earliest_epoch.julian_day.days(), 2_448_724.5);
        assert_eq!(summary.latest_epoch.julian_day.days(), 2_448_724.5);
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            format_lunar_equatorial_reference_evidence_summary(&summary),
            summary.summary_line()
        );
        assert!(lunar_equatorial_reference_evidence_summary_for_report()
            .contains("3 samples across 1 bodies"));
        assert!(lunar_equatorial_reference_evidence_summary_for_report()
            .contains("JD 2448724.5 (TT) → JD 2448724.5 (TT)"));

        let envelope = lunar_equatorial_reference_evidence_envelope()
            .expect("equatorial error envelope should exist");
        assert_eq!(envelope.sample_count, summary.sample_count);
        assert_eq!(envelope.body_count, summary.body_count);
        assert_eq!(envelope.earliest_epoch, summary.earliest_epoch);
        assert_eq!(envelope.latest_epoch, summary.latest_epoch);
        assert!(envelope.max_right_ascension_delta_deg.is_finite());
        assert!(envelope.mean_right_ascension_delta_deg.is_finite());
        assert!(envelope.median_right_ascension_delta_deg.is_finite());
        assert!(envelope.percentile_right_ascension_delta_deg.is_finite());
        assert!(envelope.max_declination_delta_deg.is_finite());
        assert!(envelope.mean_declination_delta_deg.is_finite());
        assert!(envelope.median_declination_delta_deg.is_finite());
        assert!(envelope.percentile_declination_delta_deg.is_finite());
        assert_eq!(envelope.outside_current_limits_count, 0);
        assert!(envelope.within_current_limits);
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("lunar equatorial reference error envelope"));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("median ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("p95 ΔRA="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("max ΔDec="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("mean ΔDec="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("median ΔDec="));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report().contains("p95 ΔDec="));
        assert!(
            lunar_equatorial_reference_evidence_envelope_for_report().contains("limits: ΔRA≤1e-2°")
        );
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("within current limits=true"));
        assert!(lunar_equatorial_reference_evidence_envelope_for_report()
            .contains("outside current limits=0"));
        assert_eq!(envelope.summary_line(), envelope.to_string());
        assert_eq!(
            format_lunar_equatorial_reference_evidence_envelope(&envelope),
            envelope.summary_line()
        );

        for sample in lunar_equatorial_reference_evidence() {
            assert_eq!(sample.body, CelestialBody::Moon);
            let result = ElpBackend::new()
                .position(&EphemerisRequest::new(sample.body.clone(), sample.epoch))
                .expect("equatorial reference sample should remain computable");
            let equatorial = result
                .equatorial
                .expect("equatorial reference sample should include equatorial coordinates");
            assert!(
                (equatorial.right_ascension.degrees()
                    - sample.equatorial.right_ascension.degrees())
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
    }

    #[test]
    fn lunar_equatorial_reference_evidence_summary_validates_against_the_checked_in_slice() {
        let summary = lunar_equatorial_reference_evidence_summary()
            .expect("equatorial reference evidence should exist");

        assert!(summary.validate().is_ok());
        assert_eq!(
            lunar_equatorial_reference_evidence_summary_for_report(),
            summary.summary_line()
        );

        let mut mutated = summary;
        mutated.body_count += 1;
        let error = mutated
            .validate()
            .expect_err("mutated equatorial reference summaries should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("lunar equatorial reference evidence summary mismatch"));
        assert!(error.message.contains("expected"));
        assert!(error.message.contains("found"));
    }

    #[test]
    fn lunar_equatorial_reference_evidence_envelope_validation_rejects_outlier_drift() {
        let mut envelope = lunar_equatorial_reference_evidence_envelope()
            .expect("equatorial error envelope should exist");
        envelope.outside_current_limits_count = 1;
        envelope.within_current_limits = false;
        envelope.outlier_bodies.clear();

        let error = envelope
            .validate()
            .expect_err("mutated equatorial envelopes should fail validation");

        assert_eq!(
            error,
            LunarEvidenceEnvelopeValidationError::OutlierCountMismatch {
                envelope: "lunar equatorial reference error envelope",
                outside_current_limits_count: 1,
                sample_count: envelope.sample_count,
                outlier_bodies_len: 0,
            }
        );
    }

    #[test]
    fn lunar_apparent_comparison_evidence_documents_the_mean_gap() {
        let summary =
            lunar_apparent_comparison_summary().expect("apparent comparison evidence should exist");

        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.body_count, 1);
        assert!((summary.earliest_epoch.julian_day.days() - 2_440_214.916_7).abs() < 1e-9);
        assert!((summary.latest_epoch.julian_day.days() - 2_453_986.285_649).abs() < 1e-9);
        assert!(summary.max_ecliptic_longitude_delta_deg.is_finite());
        assert!(summary.mean_ecliptic_longitude_delta_deg.is_finite());
        assert!(summary.median_ecliptic_longitude_delta_deg.is_finite());
        assert!(summary.percentile_ecliptic_longitude_delta_deg.is_finite());
        assert!(summary.max_ecliptic_latitude_delta_deg.is_finite());
        assert!(summary.mean_ecliptic_latitude_delta_deg.is_finite());
        assert!(summary.median_ecliptic_latitude_delta_deg.is_finite());
        assert!(summary.percentile_ecliptic_latitude_delta_deg.is_finite());
        assert!(summary.max_ecliptic_distance_delta_au.is_finite());
        assert!(summary.mean_ecliptic_distance_delta_au.is_finite());
        assert!(summary.median_ecliptic_distance_delta_au.is_finite());
        assert!(summary.percentile_ecliptic_distance_delta_au.is_finite());
        assert!(summary.max_right_ascension_delta_deg.is_finite());
        assert!(summary.mean_right_ascension_delta_deg.is_finite());
        assert!(summary.median_right_ascension_delta_deg.is_finite());
        assert!(summary.percentile_right_ascension_delta_deg.is_finite());
        assert!(summary.max_declination_delta_deg.is_finite());
        assert!(summary.mean_declination_delta_deg.is_finite());
        assert!(summary.median_declination_delta_deg.is_finite());
        assert!(summary.percentile_declination_delta_deg.is_finite());
        let known_epochs = [2_440_214.916_7, 2_448_724.5, 2_453_100.5, 2_453_986.285_649];
        assert!(known_epochs.contains(&summary.max_ecliptic_longitude_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_ecliptic_latitude_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_ecliptic_distance_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_right_ascension_epoch.julian_day.days()));
        assert!(known_epochs.contains(&summary.max_declination_epoch.julian_day.days()));
        assert!(lunar_apparent_comparison_summary_for_report().contains(
            "lunar apparent comparison evidence: 4 reference-only samples across 1 bodies"
        ));
        assert!(lunar_apparent_comparison_summary_for_report()
            .contains("mean-only gap against the published apparent Moon examples"));
        assert!(lunar_apparent_comparison_summary_for_report().contains("|Δlon| mean/median/p95="));
        assert!(lunar_apparent_comparison_summary_for_report().contains("|ΔDec| mean/median/p95="));
        assert!(lunar_apparent_comparison_summary_for_report().contains("@ JD"));
        assert!(lunar_apparent_comparison_summary_for_report()
            .contains("apparent requests remain unsupported"));
        assert_eq!(summary.summary_line(), summary.to_string());

        let samples = lunar_apparent_comparison_evidence();
        assert_eq!(samples.len(), 4);
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
        assert_eq!(samples[2].body, CelestialBody::Moon);
        assert_eq!(samples[2].epoch.julian_day.days(), 2_453_100.5);
        assert!(samples[2].note.contains("NASA RP 1349"));
        assert!(samples[2].note.contains("shared mean-obliquity transform"));
        assert_eq!(samples[3].body, CelestialBody::Moon);
        assert_eq!(samples[3].epoch.julian_day.days(), 2_453_986.285_649);
        assert!(samples[3].note.contains("EclipseWise"));
        assert!(samples[3].note.contains("shared mean-obliquity transform"));
        assert!(samples[3].summary_line().contains("body=Moon"));
        assert!(samples[3].summary_line().contains("EclipseWise"));
        assert!(samples[3]
            .to_string()
            .contains("shared mean-obliquity transform"));
    }

    #[test]
    fn lunar_evidence_sample_validators_reject_drifted_metadata() {
        let mut reference = lunar_reference_evidence()[0].clone();
        reference.body = CelestialBody::Sun;
        assert_eq!(
            reference.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );

        reference = lunar_reference_evidence()[0].clone();
        reference.epoch = Instant::new(
            pleiades_types::JulianDay::from_days(reference.epoch.julian_day.days()),
            TimeScale::Tdb,
        );
        assert_eq!(
            reference.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );

        let mut equatorial = lunar_equatorial_reference_evidence()[0].clone();
        equatorial.body = CelestialBody::Sun;
        assert_eq!(
            equatorial.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );

        equatorial = lunar_equatorial_reference_evidence()[0].clone();
        equatorial.note = " ";
        assert_eq!(
            equatorial.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );

        let mut apparent = lunar_apparent_comparison_evidence()[0].clone();
        apparent.body = CelestialBody::Sun;
        assert_eq!(
            apparent.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );

        apparent = lunar_apparent_comparison_evidence()[0].clone();
        apparent.apparent_distance_au = f64::NAN;
        assert_eq!(
            apparent.validate().unwrap_err().kind,
            EphemerisErrorKind::InvalidRequest
        );
    }

    #[test]
    fn lunar_reference_rows_expose_compact_summary_lines() {
        let reference = lunar_reference_evidence()[0].clone();
        assert_eq!(reference.summary_line(), reference.to_string());
        assert!(reference
            .summary_line()
            .contains("Published 1992-04-12 geocentric Moon example"));
        assert!(reference.summary_line().contains("lon=133.162655000000°"));

        let equatorial = lunar_equatorial_reference_evidence()[0].clone();
        assert_eq!(equatorial.summary_line(), equatorial.to_string());
        assert!(equatorial
            .summary_line()
            .contains("Published 1992-04-12 geocentric Moon RA/Dec example"));
        assert!(equatorial.summary_line().contains("ra=134.688470000000°"));
    }

    #[test]
    fn lunar_apparent_comparison_summary_validates_against_the_checked_in_slice() {
        let summary =
            lunar_apparent_comparison_summary().expect("apparent comparison evidence should exist");

        assert!(summary.validate().is_ok());
        assert_eq!(
            lunar_apparent_comparison_summary_for_report(),
            summary.summary_line()
        );

        let mut mutated = summary;
        mutated.sample_count += 1;
        let error = mutated
            .validate()
            .expect_err("mutated apparent comparison summaries should fail validation");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error
            .message
            .contains("lunar apparent comparison evidence summary mismatch"));
        assert!(error.message.contains("expected"));
        assert!(error.message.contains("found"));
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
    fn batch_query_rejects_apparent_requests_explicitly() {
        let backend = ElpBackend::new();
        let mut request = mean_request(CelestialBody::Moon);
        request.apparent = Apparentness::Apparent;

        let error = backend
            .positions(&[request])
            .expect_err("apparent batch requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
    }

    #[test]
    fn unsupported_time_scales_are_rejected_explicitly() {
        let backend = ElpBackend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
        );

        let error = backend
            .position(&request)
            .expect_err("UTC requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
    }

    #[test]
    fn batch_query_rejects_unsupported_time_scales_explicitly() {
        let backend = ElpBackend::new();
        let request = EphemerisRequest::new(
            CelestialBody::Moon,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Utc),
        );

        let error = backend
            .positions(&[request])
            .expect_err("UTC batch requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedTimeScale);
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

    #[test]
    fn batch_query_rejects_topocentric_requests_explicitly() {
        let backend = ElpBackend::new();
        let mut request = mean_request(CelestialBody::Moon);
        request.observer = Some(pleiades_types::ObserverLocation::new(
            Latitude::from_degrees(51.5),
            Longitude::from_degrees(0.0),
            None,
        ));

        let error = backend
            .positions(&[request])
            .expect_err("topocentric batch requests should be unsupported");
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
