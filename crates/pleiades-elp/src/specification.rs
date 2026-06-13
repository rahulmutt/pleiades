use core::fmt;

use pleiades_backend::Apparentness;
use pleiades_types::{CelestialBody, CoordinateFrame, Instant, TimeRange, TimeScale, ZodiacMode};

use crate::request_policy::LunarTheoryRequestPolicy;
use crate::source::LunarTheorySourceFamily;

pub(crate) const SUPPORTED_LUNAR_BODIES: &[CelestialBody] = &[
    CelestialBody::Moon,
    CelestialBody::MeanNode,
    CelestialBody::TrueNode,
    CelestialBody::MeanPerigee,
    CelestialBody::MeanApogee,
];
pub(crate) const UNSUPPORTED_LUNAR_BODIES: &[CelestialBody] =
    &[CelestialBody::TrueApogee, CelestialBody::TruePerigee];
pub(crate) const LUNAR_THEORY_SOURCE_ALIASES: &[&str] = &["Meeus-style truncated lunar baseline"];

pub(crate) const SUPPORTED_LUNAR_FRAMES: &[CoordinateFrame] =
    &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial];
pub(crate) const SUPPORTED_LUNAR_TIME_SCALES: &[TimeScale] = &[TimeScale::Tt, TimeScale::Tdb];
pub(crate) const SUPPORTED_LUNAR_ZODIAC_MODES: &[ZodiacMode] = &[ZodiacMode::Tropical];
pub(crate) const SUPPORTED_LUNAR_APPARENTNESS: &[Apparentness] = &[Apparentness::Mean];
pub(crate) const LUNAR_THEORY_REQUEST_POLICY: LunarTheoryRequestPolicy = LunarTheoryRequestPolicy {
    supported_frames: SUPPORTED_LUNAR_FRAMES,
    supported_time_scales: SUPPORTED_LUNAR_TIME_SCALES,
    supported_zodiac_modes: SUPPORTED_LUNAR_ZODIAC_MODES,
    supported_apparentness: SUPPORTED_LUNAR_APPARENTNESS,
    supports_topocentric_observer: false,
};
pub(crate) const LUNAR_THEORY_VALIDATION_WINDOW: TimeRange = TimeRange::new(
    Some(Instant::new(
        pleiades_types::JulianDay::from_days(2_448_724.5),
        TimeScale::Tt,
    )),
    Some(Instant::new(
        pleiades_types::JulianDay::from_days(2_459_278.5),
        TimeScale::Tt,
    )),
);

pub(crate) const LUNAR_HIGH_CURVATURE_WINDOW_EPOCHS: [Instant; 6] = [
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000 - 1.0),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000 - 0.5),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000 + 0.5),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000 + 1.0),
        TimeScale::Tt,
    ),
    Instant::new(
        pleiades_types::JulianDay::from_days(crate::J2000 + 2.0),
        TimeScale::Tt,
    ),
];
pub(crate) const LUNAR_HIGH_CURVATURE_LONGITUDE_LIMIT_DEG: f64 = 20.0;
pub(crate) const LUNAR_HIGH_CURVATURE_LATITUDE_LIMIT_DEG: f64 = 10.0;
pub(crate) const LUNAR_HIGH_CURVATURE_DISTANCE_LIMIT_AU: f64 = 0.02;

pub(crate) const LUNAR_THEORY_SPECIFICATION: LunarTheorySpecification = LunarTheorySpecification {
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
    pub const fn source_selection(self) -> crate::source::LunarTheorySourceSelection {
        crate::source::LunarTheorySourceSelection {
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
        crate::format_lunar_theory_specification(self)
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

impl fmt::Display for LunarTheorySpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the currently selected compact lunar-theory specification.
pub fn lunar_theory_specification() -> LunarTheorySpecification {
    crate::catalog::lunar_theory_catalog()
        .iter()
        .find(|entry| entry.selected)
        .map(|entry| entry.specification)
        .unwrap_or(LUNAR_THEORY_SPECIFICATION)
}
