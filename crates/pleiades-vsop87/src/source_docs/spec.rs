use pleiades_backend::{Apparentness, FrameTreatmentSummary};
use pleiades_types::{CelestialBody, CoordinateFrame, TimeScale, ZodiacMode};
use std::collections::BTreeSet;
use std::fmt;

use crate::profiles::{
    body_catalog_entries, body_catalog_entry_for_body, format_apparentness_modes,
    format_coordinate_frames, format_time_scales, format_zodiac_modes, join_display,
    source_text_for_file,
};

/// Provenance metadata for one source-backed VSOP87B body path.
///
/// Captures the public coefficient file, frame, units, and reduction/transform
/// notes for a single body so the mixed generated-binary and mean-element
/// provenance stays auditable. Frame is J2000 ecliptic/equinox at the backend
/// boundary; this describes the mean, first-party path (not an apparent place).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceSpecification {
    /// Body covered by the source-backed slice.
    pub body: CelestialBody,
    /// Public coefficient file backing the body.
    pub source_file: &'static str,
    /// Source series variant.
    pub variant: &'static str,
    /// Coordinate family represented by the coefficients.
    pub coordinate_family: &'static str,
    /// Reference frame for the coefficients.
    pub frame: &'static str,
    /// Measurement units used by the coefficients.
    pub units: &'static str,
    /// How the coefficients are reduced to a geocentric chart-facing result.
    pub reduction: &'static str,
    /// Frame-transform note describing how equatorial coordinates are derived.
    pub transform_note: &'static str,
    /// How much of the public source file is currently retained.
    pub truncation_policy: &'static str,
    /// Current date-range note for the retained slice.
    pub date_range: &'static str,
}

impl Vsop87SourceSpecification {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 source spec: body={}, file={}, variant={}, family={}, frame={}, units={}, reduction={}, transform={}, truncation={}, date range={}",
            self.body,
            self.source_file,
            self.variant,
            self.coordinate_family,
            self.frame,
            self.units,
            self.reduction,
            self.transform_note,
            self.truncation_policy,
            self.date_range,
        )
    }

    /// Validates the source specification and returns its compact report line.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceSpecificationValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Vsop87SourceSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a VSOP87 source specification that contains blank, unknown, or drifted
/// metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceSpecificationValidationError {
    /// The rendered field is blank or whitespace-only for the named body.
    BlankField {
        /// Body whose specification carries the blank field.
        body: CelestialBody,
        /// Name of the field that is blank.
        field: &'static str,
    },
    /// The specification names a body that is not backed by the current source catalog.
    UnknownBody {
        /// Body absent from the current source catalog.
        body: CelestialBody,
    },
    /// The specification names a public source file that is not part of the current source catalog.
    UnknownSourceFile {
        /// Body whose specification references the unknown file.
        body: CelestialBody,
        /// Public source-file label that is not part of the catalog.
        source_file: &'static str,
    },
    /// The rendered field no longer matches the current canonical catalog value.
    FieldOutOfSync {
        /// Body whose specification field drifted from the catalog.
        body: CelestialBody,
        /// Name of the drifted field.
        field: &'static str,
        /// Canonical value expected by the current catalog.
        expected: &'static str,
        /// Value found on the drifted specification.
        found: &'static str,
    },
    /// The public source file label appears more than once in the catalog.
    DuplicateSourceFile {
        /// Public source-file label that is duplicated in the catalog.
        source_file: &'static str,
    },
}

impl fmt::Display for Vsop87SourceSpecificationValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankField { body, field } => {
                write!(
                    f,
                    "the VSOP87 source specification for {body} has a blank `{field}` field"
                )
            }
            Self::UnknownBody { body } => write!(
                f,
                "the VSOP87 source specification for {body} is no longer backed by the current source catalog"
            ),
            Self::UnknownSourceFile { body, source_file } => write!(
                f,
                "the VSOP87 source specification for {body} references unknown public source file `{source_file}`"
            ),
            Self::FieldOutOfSync {
                body,
                field,
                expected,
                found,
            } => write!(
                f,
                "the VSOP87 source specification for {body} has `{field}` = `{found}`, but expected `{expected}`"
            ),
            Self::DuplicateSourceFile { source_file } => write!(
                f,
                "the VSOP87 source specification catalog lists public source file `{source_file}` more than once"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceSpecificationValidationError {}

impl Vsop87SourceSpecification {
    /// Returns `Ok(())` when the specification still carries the current canonical public metadata.
    pub fn validate(&self) -> Result<(), Vsop87SourceSpecificationValidationError> {
        for (field, value) in [
            ("source_file", self.source_file),
            ("variant", self.variant),
            ("coordinate_family", self.coordinate_family),
            ("frame", self.frame),
            ("units", self.units),
            ("reduction", self.reduction),
            ("transform_note", self.transform_note),
            ("truncation_policy", self.truncation_policy),
            ("date_range", self.date_range),
        ] {
            if value.trim().is_empty() {
                return Err(Vsop87SourceSpecificationValidationError::BlankField {
                    body: self.body.clone(),
                    field,
                });
            }
        }

        if source_text_for_file(self.source_file).is_none() {
            return Err(
                Vsop87SourceSpecificationValidationError::UnknownSourceFile {
                    body: self.body.clone(),
                    source_file: self.source_file,
                },
            );
        }

        let Some(expected) = body_catalog_entry_for_body(&self.body)
            .and_then(|entry| entry.source_specification.as_ref())
        else {
            return Err(Vsop87SourceSpecificationValidationError::UnknownBody {
                body: self.body.clone(),
            });
        };

        if expected.source_file != self.source_file {
            return Err(Vsop87SourceSpecificationValidationError::FieldOutOfSync {
                body: self.body.clone(),
                field: "source_file",
                expected: expected.source_file,
                found: self.source_file,
            });
        }

        for (field, expected, found) in [
            ("variant", expected.variant, self.variant),
            (
                "coordinate_family",
                expected.coordinate_family,
                self.coordinate_family,
            ),
            ("frame", expected.frame, self.frame),
            ("units", expected.units, self.units),
            ("reduction", expected.reduction, self.reduction),
            (
                "transform_note",
                expected.transform_note,
                self.transform_note,
            ),
            (
                "truncation_policy",
                expected.truncation_policy,
                self.truncation_policy,
            ),
            ("date_range", expected.date_range, self.date_range),
        ] {
            if found != expected {
                return Err(Vsop87SourceSpecificationValidationError::FieldOutOfSync {
                    body: self.body.clone(),
                    field,
                    expected,
                    found,
                });
            }
        }

        Ok(())
    }
}

/// Returns the source-backed VSOP87B specifications, one per body with a public
/// coefficient file (Sun through Neptune); the mean-element Pluto fallback has
/// no source specification and is excluded.
pub fn source_specifications() -> Vec<Vsop87SourceSpecification> {
    body_catalog_entries()
        .iter()
        .filter_map(|entry| entry.source_specification.clone())
        .collect()
}

/// Formats a single VSOP87 source specification for reporting.
pub fn format_source_specification(spec: &Vsop87SourceSpecification) -> String {
    match spec.validated_summary_line() {
        Ok(summary) => summary,
        Err(error) => format!("VSOP87 source specification unavailable ({error})"),
    }
}

/// Formats the current VSOP87 source-specification catalog for reporting.
pub fn format_source_specifications(specs: &[Vsop87SourceSpecification]) -> String {
    join_display(specs)
}

/// Validates that the supplied VSOP87 source-specification catalog carries non-blank metadata
/// and unique public source-file labels.
pub fn validate_source_specifications(
    specs: &[Vsop87SourceSpecification],
) -> Result<(), Vsop87SourceSpecificationValidationError> {
    let mut seen_source_files = BTreeSet::new();

    for spec in specs {
        let source_file = spec.source_file.trim();
        if !seen_source_files.insert(source_file.to_string()) {
            return Err(
                Vsop87SourceSpecificationValidationError::DuplicateSourceFile {
                    source_file: spec.source_file.trim(),
                },
            );
        }
    }

    for spec in specs {
        spec.validate()?;
    }

    Ok(())
}

/// Returns the structured frame-treatment summary for VSOP87-backed results.
pub const fn frame_treatment_summary_details() -> FrameTreatmentSummary {
    FrameTreatmentSummary::new(
        "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform",
    )
}

/// Returns the current frame-treatment summary for VSOP87-backed results.
pub fn frame_treatment_summary() -> &'static str {
    frame_treatment_summary_details().summary_line()
}

/// Structured request policy for the current VSOP87 backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vsop87RequestPolicy {
    /// Coordinate frames the current backend exposes.
    pub supported_frames: &'static [CoordinateFrame],
    /// Time scales accepted by the current backend.
    pub supported_time_scales: &'static [TimeScale],
    /// Zodiac modes accepted by the current backend.
    pub supported_zodiac_modes: &'static [ZodiacMode],
    /// Apparentness modes accepted by the current backend.
    pub supported_apparentness: &'static [Apparentness],
    /// Whether the current backend accepts topocentric observer requests.
    pub supports_topocentric_observer: bool,
}

/// Validation error for a VSOP87 request-policy summary that drifted from the current backend posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vsop87RequestPolicyValidationError {
    /// One of the request-policy fields differs from the current backend posture.
    FieldOutOfSync {
        /// Name of the request-policy field that drifted from the posture.
        field: &'static str,
    },
}

impl fmt::Display for Vsop87RequestPolicyValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 request-policy summary field `{field}` is out of sync with the current posture"
            ),
        }
    }
}

impl std::error::Error for Vsop87RequestPolicyValidationError {}

impl Vsop87RequestPolicy {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "frames={}; time scales={}; zodiac modes={}; apparentness={}; topocentric observer={}",
            format_coordinate_frames(self.supported_frames),
            format_time_scales(self.supported_time_scales),
            format_zodiac_modes(self.supported_zodiac_modes),
            format_apparentness_modes(self.supported_apparentness),
            self.supports_topocentric_observer,
        )
    }

    /// Validates the summary against the current VSOP87 backend posture.
    pub fn validate(&self) -> Result<(), Vsop87RequestPolicyValidationError> {
        if self.supported_frames != VSOP87_REQUEST_POLICY.supported_frames {
            return Err(Vsop87RequestPolicyValidationError::FieldOutOfSync {
                field: "supported_frames",
            });
        }
        if self.supported_time_scales != VSOP87_REQUEST_POLICY.supported_time_scales {
            return Err(Vsop87RequestPolicyValidationError::FieldOutOfSync {
                field: "supported_time_scales",
            });
        }
        if self.supported_zodiac_modes != VSOP87_REQUEST_POLICY.supported_zodiac_modes {
            return Err(Vsop87RequestPolicyValidationError::FieldOutOfSync {
                field: "supported_zodiac_modes",
            });
        }
        if self.supported_apparentness != VSOP87_REQUEST_POLICY.supported_apparentness {
            return Err(Vsop87RequestPolicyValidationError::FieldOutOfSync {
                field: "supported_apparentness",
            });
        }
        if self.supports_topocentric_observer != VSOP87_REQUEST_POLICY.supports_topocentric_observer
        {
            return Err(Vsop87RequestPolicyValidationError::FieldOutOfSync {
                field: "supports_topocentric_observer",
            });
        }
        Ok(())
    }
}

impl fmt::Display for Vsop87RequestPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

const VSOP87_REQUEST_POLICY: Vsop87RequestPolicy = Vsop87RequestPolicy {
    supported_frames: &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
    supported_time_scales: &[TimeScale::Tt, TimeScale::Tdb],
    supported_zodiac_modes: &[ZodiacMode::Tropical],
    supported_apparentness: &[Apparentness::Mean],
    supports_topocentric_observer: false,
};

/// Returns the current VSOP87 request policy.
pub const fn vsop87_request_policy() -> Vsop87RequestPolicy {
    VSOP87_REQUEST_POLICY
}
