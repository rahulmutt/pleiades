use pleiades_backend::{
    Apparentness, EphemerisBackend, EphemerisRequest, FrameTreatmentSummary, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, Instant, Latitude, Longitude, TimeScale,
    ZodiacMode,
};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::OnceLock;

use crate::backend::Vsop87Backend;
use crate::profiles::{
    body_catalog_entries, body_catalog_entry_for_body, body_source_profiles, count_vsop87_terms,
    fallback_body_profiles, fnv1a_64, format_apparentness_modes, format_coordinate_frames,
    format_time_scales, format_zodiac_modes, join_display, source_backed_body_order,
    source_backed_body_profiles, source_file_for_body, source_kind_for_body, source_text_for_file,
    Vsop87BodySourceKind,
};
use crate::tables::vsop87b_earth::generated_vsop87b_table_bytes;
use crate::transforms::signed_longitude_delta_degrees;

const J1900: f64 = 2_415_020.0;
const J2000: f64 = 2_451_545.0;

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
        body: CelestialBody,
        field: &'static str,
    },
    /// The specification names a body that is not backed by the current source catalog.
    UnknownBody { body: CelestialBody },
    /// The specification names a public source file that is not part of the current source catalog.
    UnknownSourceFile {
        body: CelestialBody,
        source_file: &'static str,
    },
    /// The rendered field no longer matches the current canonical catalog value.
    FieldOutOfSync {
        body: CelestialBody,
        field: &'static str,
        expected: &'static str,
        found: &'static str,
    },
    /// The public source file label appears more than once in the catalog.
    DuplicateSourceFile { source_file: &'static str },
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

/// Reproducibility audit details for a vendored VSOP87B source file.
///
/// These records give the generated-table work a stable, deterministic
/// fingerprint of the public inputs that back each source-backed body. They do
/// not replace the coefficient tables themselves; instead they document the
/// exact source material, size, and parse shape that a future generated-table
/// pipeline must reproduce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceAudit {
    /// Body covered by this source audit.
    pub body: CelestialBody,
    /// Public coefficient file backing the body.
    pub source_file: &'static str,
    /// Raw source byte length.
    pub byte_length: usize,
    /// Raw source line count.
    pub line_count: usize,
    /// Total parsed coefficient term count across all series.
    pub term_count: usize,
    /// Deterministic 64-bit fingerprint of the vendored source text.
    pub fingerprint: u64,
}

/// Validation error for a VSOP87 source-audit record that drifted from the current manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceAuditValidationError {
    /// The audit record does not name a source file.
    BlankSourceFile {
        position: usize,
        body: CelestialBody,
    },
    /// The audit record references a source file that does not exist in the current source catalog.
    UnknownSourceFile {
        position: usize,
        source_file: &'static str,
    },
    /// The audit record body/source pairing does not match the current source catalog.
    BodySourceMismatch {
        position: usize,
        body: CelestialBody,
        source_file: &'static str,
        expected_body: CelestialBody,
    },
    /// A rendered audit field no longer matches the current source text.
    FieldOutOfSync {
        position: usize,
        body: CelestialBody,
        source_file: &'static str,
        field: &'static str,
    },
}

impl fmt::Display for Vsop87SourceAuditValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSourceFile { position, body } => write!(
                f,
                "source audit record #{position} has a blank source file for {body}"
            ),
            Self::UnknownSourceFile { position, source_file } => write!(
                f,
                "source audit record #{position} references unknown source file `{source_file}`"
            ),
            Self::BodySourceMismatch {
                position,
                body,
                source_file,
                expected_body,
            } => write!(
                f,
                "source audit record #{position} uses source file `{source_file}`, which belongs to {expected_body} rather than {body}"
            ),
            Self::FieldOutOfSync {
                position,
                body,
                source_file,
                field,
            } => write!(
                f,
                "source audit record #{position} for {body} and source file `{source_file}` has a stale `{field}` field"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceAuditValidationError {}

impl Vsop87SourceAudit {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 source audit record: body={}, file={}, bytes={}, lines={}, terms={}, fingerprint=0x{:016x}",
            self.body,
            self.source_file,
            self.byte_length,
            self.line_count,
            self.term_count,
            self.fingerprint
        )
    }

    fn validate_at_position(
        &self,
        position: usize,
    ) -> Result<(), Vsop87SourceAuditValidationError> {
        if self.source_file.trim().is_empty() {
            return Err(Vsop87SourceAuditValidationError::BlankSourceFile {
                position,
                body: self.body.clone(),
            });
        }

        let Some(source) = source_text_for_file(self.source_file) else {
            return Err(Vsop87SourceAuditValidationError::UnknownSourceFile {
                position,
                source_file: self.source_file,
            });
        };

        let Some(expected_body) = source_specifications()
            .into_iter()
            .find(|spec| spec.source_file == self.source_file)
            .map(|spec| spec.body)
        else {
            return Err(Vsop87SourceAuditValidationError::UnknownSourceFile {
                position,
                source_file: self.source_file,
            });
        };

        if expected_body != self.body {
            return Err(Vsop87SourceAuditValidationError::BodySourceMismatch {
                position,
                body: self.body.clone(),
                source_file: self.source_file,
                expected_body,
            });
        }

        if self.byte_length != source.len() {
            return Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
                position,
                body: self.body.clone(),
                source_file: self.source_file,
                field: "byte_length",
            });
        }
        if self.line_count != source.lines().count() {
            return Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
                position,
                body: self.body.clone(),
                source_file: self.source_file,
                field: "line_count",
            });
        }
        if self.term_count != count_vsop87_terms(source) {
            return Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
                position,
                body: self.body.clone(),
                source_file: self.source_file,
                field: "term_count",
            });
        }
        if self.fingerprint != fnv1a_64(source.as_bytes()) {
            return Err(Vsop87SourceAuditValidationError::FieldOutOfSync {
                position,
                body: self.body.clone(),
                source_file: self.source_file,
                field: "fingerprint",
            });
        }

        Ok(())
    }

    /// Returns `Ok(())` when the record still matches the current source text.
    pub fn validate(&self) -> Result<(), Vsop87SourceAuditValidationError> {
        self.validate_at_position(1)
    }
}

impl fmt::Display for Vsop87SourceAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary metrics for the current VSOP87 source audit manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceAuditSummary {
    /// Number of source-backed bodies represented in the audit manifest.
    pub source_count: usize,
    /// Source-backed bodies represented in the audit manifest, in release-facing body order.
    pub source_bodies: Vec<CelestialBody>,
    /// Public source files represented in the audit manifest.
    pub source_files: Vec<&'static str>,
    /// Number of vendored full-file source entries.
    pub vendored_full_file_count: usize,
    /// Number of deterministic fingerprints recorded in the audit manifest.
    pub fingerprint_count: usize,
    /// Total parsed coefficient term count across all audited sources.
    pub total_term_count: usize,
    /// Maximum raw source line count across the audited files.
    pub max_line_count: usize,
    /// Maximum raw source byte length across the audited files.
    pub max_byte_length: usize,
}

/// Validation error for a VSOP87 source-audit summary that drifted from the current manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceAuditSummaryValidationError {
    /// A rendered summary field no longer matches the current source-audit manifest.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87SourceAuditSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source audit summary field `{field}` is out of sync with the current manifest"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceAuditSummaryValidationError {}

impl Vsop87SourceAuditSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 source audit: {} source-backed bodies ({}) across {} source files ({}); {} vendored full-file inputs, {} total terms, max source size {} bytes / {} lines, {} deterministic fingerprints",
            self.source_count,
            join_display(&self.source_bodies),
            self.source_files.len(),
            join_display(&self.source_files),
            self.vendored_full_file_count,
            self.total_term_count,
            self.max_byte_length,
            self.max_line_count,
            self.fingerprint_count
        )
    }

    /// Returns the rendered summary line after validating the cached manifest snapshot.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceAuditSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current source-audit manifest.
    pub fn validate(&self) -> Result<(), Vsop87SourceAuditSummaryValidationError> {
        let audits = source_audits();
        validate_source_audits(&audits).map_err(|_| {
            Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "audit_records",
            }
        })?;
        let source_specs = source_specifications();
        let expected_source_files = source_specs
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>();

        if self.source_count != audits.len() {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "source_count",
            });
        }
        if self.source_bodies != source_backed_body_order() {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "source_bodies",
            });
        }
        if self.source_files != expected_source_files {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "source_files",
            });
        }
        if self.vendored_full_file_count
            != audits
                .iter()
                .filter(|audit| audit.source_file.starts_with("VSOP87B."))
                .count()
        {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "vendored_full_file_count",
            });
        }
        if self.fingerprint_count != audits.len() {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "fingerprint_count",
            });
        }
        if self.total_term_count != audits.iter().map(|audit| audit.term_count).sum::<usize>() {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "total_term_count",
            });
        }
        if self.max_line_count
            != audits
                .iter()
                .map(|audit| audit.line_count)
                .max()
                .unwrap_or(0)
        {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "max_line_count",
            });
        }
        if self.max_byte_length
            != audits
                .iter()
                .map(|audit| audit.byte_length)
                .max()
                .unwrap_or(0)
        {
            return Err(Vsop87SourceAuditSummaryValidationError::FieldOutOfSync {
                field: "max_byte_length",
            });
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87SourceAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Summary metrics for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationSummary {
    /// Number of source specifications described by the catalog.
    pub source_specification_count: usize,
    /// Number of source-backed body profiles described by the catalog.
    pub source_backed_profile_count: usize,
    /// Bodies that still use a source-backed planetary path rather than the fallback mean-element path.
    pub source_backed_bodies: Vec<CelestialBody>,
    /// Public source files currently represented by the catalog.
    pub source_files: Vec<&'static str>,
    /// Bodies currently served by generated-binary VSOP87B tables.
    pub generated_binary_bodies: Vec<CelestialBody>,
    /// Bodies currently served by vendored full-file source paths.
    pub vendored_full_file_bodies: Vec<CelestialBody>,
    /// Bodies currently served by truncated source slices.
    pub truncated_bodies: Vec<CelestialBody>,
    /// Number of vendored full-file body profiles.
    pub vendored_full_file_profile_count: usize,
    /// Number of generated-binary body profiles.
    pub generated_binary_profile_count: usize,
    /// Number of truncated-slice body profiles.
    pub truncated_profile_count: usize,
    /// Number of approximate fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the approximate fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
    /// Unique date-range notes carried by the source specifications.
    pub date_ranges: Vec<&'static str>,
}

impl Vsop87SourceDocumentationSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_summary(self)
    }

    /// Returns the rendered summary line after validating the cached catalog snapshot.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceDocumentationSummaryValidationError> {
        self.validate()?;
        source_documentation_health_summary()
            .validate()
            .map_err(
                |_| Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_documentation_health",
                },
            )?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for Vsop87SourceDocumentationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error returned when the VSOP87 source-documentation summary drifts from the current
/// catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceDocumentationSummaryValidationError {
    /// A rendered summary field no longer matches the current source-documentation catalog.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87SourceDocumentationSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source documentation summary field `{field}` is out of sync with the current source catalog"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceDocumentationSummaryValidationError {}

impl Vsop87SourceDocumentationSummary {
    /// Returns `Ok(())` when the summary still matches the current source-documentation catalog.
    pub fn validate(&self) -> Result<(), Vsop87SourceDocumentationSummaryValidationError> {
        let source_specs = source_specifications();
        let source_backed_profiles = source_backed_body_profiles();
        let fallback_profiles = fallback_body_profiles();
        let expected_source_files = source_specs
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>();
        let expected_date_ranges = {
            let mut date_ranges = source_specs
                .iter()
                .map(|spec| spec.date_range)
                .collect::<Vec<_>>();
            date_ranges.sort_unstable();
            date_ranges.dedup();
            date_ranges
        };
        let expected_generated_binary_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_vendored_full_file_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_truncated_bodies = source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();
        let expected_fallback_bodies = fallback_profiles
            .iter()
            .map(|profile| profile.body.clone())
            .collect::<Vec<_>>();

        if self.source_specification_count != source_specs.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_specification_count",
                },
            );
        }
        if self.source_backed_profile_count != source_backed_profiles.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_backed_profile_count",
                },
            );
        }
        if self.source_backed_bodies != source_backed_body_order() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_backed_bodies",
                },
            );
        }
        if self.source_files != expected_source_files {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "source_files",
                },
            );
        }
        if self.generated_binary_bodies != expected_generated_binary_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "generated_binary_bodies",
                },
            );
        }
        if self.vendored_full_file_bodies != expected_vendored_full_file_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "vendored_full_file_bodies",
                },
            );
        }
        if self.truncated_bodies != expected_truncated_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "truncated_bodies",
                },
            );
        }
        if self.vendored_full_file_profile_count != expected_vendored_full_file_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "vendored_full_file_profile_count",
                },
            );
        }
        if self.generated_binary_profile_count != expected_generated_binary_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "generated_binary_profile_count",
                },
            );
        }
        if self.truncated_profile_count != expected_truncated_bodies.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "truncated_profile_count",
                },
            );
        }
        if self.fallback_profile_count != fallback_profiles.len() {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "fallback_profile_count",
                },
            );
        }
        if self.fallback_bodies != expected_fallback_bodies {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "fallback_bodies",
                },
            );
        }
        if self.date_ranges != expected_date_ranges {
            return Err(
                Vsop87SourceDocumentationSummaryValidationError::FieldOutOfSync {
                    field: "date_ranges",
                },
            );
        }

        Ok(())
    }
}

/// Structured issue labels used by the VSOP87 source-documentation health check.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Vsop87SourceDocumentationHealthIssue {
    /// The number of source specifications does not match the number of public source files.
    SourceSpecificationFileCountMismatch,
    /// The documented source file order does not match the release-facing catalog order.
    SourceFileOrderMismatch,
    /// The source-backed body order does not match the current generated/vendored/truncated partition order.
    SourceBackedBodyOrderMismatch,
    /// The source-backed profile partition counts do not add up to the total source-backed profile count.
    SourceBackedProfilePartitionMismatch,
    /// The source-backed body list contains a duplicate body entry.
    SourceBackedBodyDuplicate,
    /// The fallback body list contains a duplicate body entry.
    FallbackBodyDuplicate,
    /// A source-backed body also appears in the fallback body list.
    SourceBackedFallbackBodyOverlap,
    /// The source-backed and fallback body profiles do not cover the full body catalog.
    BodyProfileCoverageMismatch,
    /// The source specification catalog count does not match the parsed source specification list.
    SourceSpecificationCatalogCountMismatch,
    /// The documented variant, coordinate family, frame, units, reduction, transform note, truncation policy, or date range drifted.
    DocumentedFieldMismatch,
}

impl Vsop87SourceDocumentationHealthIssue {
    /// Returns the compact label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::SourceSpecificationFileCountMismatch => {
                "source specification/file count mismatch"
            }
            Self::SourceFileOrderMismatch => "source file order mismatch",
            Self::SourceBackedBodyOrderMismatch => "source-backed body order mismatch",
            Self::SourceBackedProfilePartitionMismatch => {
                "source-backed profile partition mismatch"
            }
            Self::SourceBackedBodyDuplicate => "source-backed body duplicate",
            Self::FallbackBodyDuplicate => "fallback body duplicate",
            Self::SourceBackedFallbackBodyOverlap => "source-backed/fallback body overlap",
            Self::BodyProfileCoverageMismatch => "body profile coverage mismatch",
            Self::SourceSpecificationCatalogCountMismatch => {
                "source specification catalog count mismatch"
            }
            Self::DocumentedFieldMismatch => "documented field mismatch",
        }
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Validation error returned when the VSOP87 source-documentation health summary
/// reports an inconsistent catalog state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationHealthError {
    summary: Vsop87SourceDocumentationHealthSummary,
}

impl Vsop87SourceDocumentationHealthError {
    /// Returns the inconsistent summary that triggered the validation failure.
    pub fn summary(&self) -> &Vsop87SourceDocumentationHealthSummary {
        &self.summary
    }

    /// Returns the release-facing summary line for the inconsistent catalog state.
    pub fn summary_line(&self) -> String {
        self.summary.summary_line()
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

impl std::error::Error for Vsop87SourceDocumentationHealthError {}

/// Consistency check for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationHealthSummary {
    /// Whether the catalog counts line up with the internal body catalog.
    pub consistent: bool,
    /// Whether the documented source metadata stays aligned with the current
    /// VSOP87B policy for variant, frame, units, truncation, and date range.
    pub documentation_consistent: bool,
    /// Structured labels describing any catalog inconsistencies.
    pub issues: Vec<Vsop87SourceDocumentationHealthIssue>,
    /// Number of source specifications described by the catalog.
    pub source_specification_count: usize,
    /// Number of public source files represented by the catalog.
    pub source_file_count: usize,
    /// Public source files represented by the catalog in release-facing order.
    pub source_files: Vec<&'static str>,
    /// Number of source-backed body profiles described by the catalog.
    pub source_backed_profile_count: usize,
    /// Bodies that still use a source-backed planetary path rather than the fallback mean-element path.
    pub source_backed_bodies: Vec<CelestialBody>,
    /// Bodies in the current source-backed partition order.
    pub source_backed_partition_bodies: Vec<CelestialBody>,
    /// Bodies currently served by generated-binary VSOP87B tables.
    pub generated_binary_bodies: Vec<CelestialBody>,
    /// Bodies currently served by vendored full-file source paths.
    pub vendored_full_file_bodies: Vec<CelestialBody>,
    /// Bodies currently served by truncated source slices.
    pub truncated_bodies: Vec<CelestialBody>,
    /// Total number of body profiles in the internal catalog.
    pub body_profile_count: usize,
    /// Number of generated-binary body profiles.
    pub generated_binary_profile_count: usize,
    /// Number of vendored full-file body profiles.
    pub vendored_full_file_profile_count: usize,
    /// Number of truncated-slice body profiles.
    pub truncated_profile_count: usize,
    /// Number of approximate fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the approximate fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
}

impl Vsop87SourceDocumentationHealthSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_health_summary(self)
    }

    /// Validates that the summary represents a healthy, internally consistent
    /// VSOP87 source-documentation catalog.
    pub fn validate(&self) -> Result<(), Box<Vsop87SourceDocumentationHealthError>> {
        let source_backed_profile_partition_count = self.generated_binary_profile_count
            + self.vendored_full_file_profile_count
            + self.truncated_profile_count;
        let source_backed_list_count = self.generated_binary_bodies.len()
            + self.vendored_full_file_bodies.len()
            + self.truncated_bodies.len();
        let body_profile_list_count = source_backed_list_count + self.fallback_bodies.len();
        let structural_consistency = self.source_file_count == self.source_files.len()
            && self.source_specification_count == self.source_file_count
            && self.source_backed_profile_count == self.source_backed_bodies.len()
            && self.source_backed_profile_count == self.source_backed_partition_bodies.len()
            && self.source_backed_profile_count == source_backed_profile_partition_count
            && self.source_backed_profile_count == source_backed_list_count
            && self.body_profile_count
                == self.source_backed_profile_count + self.fallback_profile_count
            && self.body_profile_count == body_profile_list_count
            && self.generated_binary_profile_count == self.generated_binary_bodies.len()
            && self.vendored_full_file_profile_count == self.vendored_full_file_bodies.len()
            && self.truncated_profile_count == self.truncated_bodies.len()
            && self.fallback_profile_count == self.fallback_bodies.len()
            && self.source_backed_bodies == self.source_backed_partition_bodies
            && body_labels_are_unique(&self.source_backed_bodies)
            && body_labels_are_unique(&self.fallback_bodies)
            && body_lists_are_disjoint(&self.source_backed_bodies, &self.fallback_bodies);

        if self.consistent
            && self.documentation_consistent
            && self.issues.is_empty()
            && structural_consistency
        {
            Ok(())
        } else {
            Err(Box::new(Vsop87SourceDocumentationHealthError {
                summary: self.clone(),
            }))
        }
    }
}

impl fmt::Display for Vsop87SourceDocumentationHealthSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Canonical J2000 reference samples for the source-backed VSOP87B paths.
///
/// These values are the same full-file public IMCCE VSOP87B reference points
/// exercised by the backend regression tests. The validation tooling uses them
/// to render measured deltas against the checked-in source-backed coefficient
/// paths while the generated-table pipeline continues to expand.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEpochSample {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Reference geocentric ecliptic longitude in degrees.
    pub expected_longitude_deg: f64,
    /// Reference geocentric ecliptic latitude in degrees.
    pub expected_latitude_deg: f64,
    /// Reference geocentric distance in astronomical units.
    pub expected_distance_au: f64,
    /// Maximum acceptable geocentric longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Maximum acceptable geocentric latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Maximum acceptable geocentric distance delta in astronomical units.
    pub max_distance_delta_au: f64,
}

/// Public release-facing error envelope for one body at the canonical J2000
/// comparison epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalBodyEvidence {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Calculation family used for the body.
    pub source_kind: Vsop87BodySourceKind,
    /// Public coefficient file backing the body.
    pub source_file: &'static str,
    /// Human-readable provenance detail for the body.
    pub provenance: &'static str,
    /// Absolute geocentric longitude delta in degrees.
    pub longitude_delta_deg: f64,
    /// Absolute geocentric latitude delta in degrees.
    pub latitude_delta_deg: f64,
    /// Absolute geocentric distance delta in astronomical units.
    pub distance_delta_au: f64,
    /// Interim longitude delta limit used for this body.
    pub longitude_limit_deg: f64,
    /// Interim latitude delta limit used for this body.
    pub latitude_limit_deg: f64,
    /// Interim distance delta limit used for this body.
    pub distance_limit_au: f64,
    /// Whether the body is within the current interim limits.
    pub within_interim_limits: bool,
}

impl Vsop87CanonicalBodyEvidence {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: kind={}, source={}, provenance={}, Δlon={:.12}°/limit {:.12}°/margin {:+.12}°, Δlat={:.12}°/limit {:.12}°/margin {:+.12}°, Δdist={:.12} AU/limit {:.12} AU/margin {:+.12} AU, status {}",
            self.body,
            self.source_kind,
            self.source_file,
            self.provenance,
            self.longitude_delta_deg,
            self.longitude_limit_deg,
            self.longitude_limit_deg - self.longitude_delta_deg,
            self.latitude_delta_deg,
            self.latitude_limit_deg,
            self.latitude_limit_deg - self.latitude_delta_deg,
            self.distance_delta_au,
            self.distance_limit_au,
            self.distance_limit_au - self.distance_delta_au,
            if self.within_interim_limits {
                "within interim limits"
            } else {
                "outside interim limits"
            },
        )
    }

    /// Returns `Ok(())` when the row still matches the current canonical evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBodyEvidenceValidationError> {
        let entry = body_catalog_entry_for_body(&self.body).ok_or(
            Vsop87CanonicalBodyEvidenceValidationError::UnknownBody {
                body: self.body.clone(),
            },
        )?;

        if self.source_kind != entry.source_profile.kind {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::SourceKindMismatch {
                    body: self.body.clone(),
                    expected: entry.source_profile.kind,
                    found: self.source_kind,
                },
            );
        }

        let specification = entry.source_specification.as_ref().ok_or(
            Vsop87CanonicalBodyEvidenceValidationError::MissingSourceSpecification {
                body: self.body.clone(),
            },
        )?;

        if self.source_file != specification.source_file {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::SourceFileMismatch {
                    body: self.body.clone(),
                    expected: specification.source_file,
                    found: self.source_file,
                },
            );
        }
        if self.provenance != entry.source_profile.provenance {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::ProvenanceMismatch {
                    body: self.body.clone(),
                    expected: entry.source_profile.provenance,
                    found: self.provenance,
                },
            );
        }

        for (field, value) in [
            ("longitude_delta_deg", self.longitude_delta_deg),
            ("latitude_delta_deg", self.latitude_delta_deg),
            ("distance_delta_au", self.distance_delta_au),
            ("longitude_limit_deg", self.longitude_limit_deg),
            ("latitude_limit_deg", self.latitude_limit_deg),
            ("distance_limit_au", self.distance_limit_au),
        ] {
            if !value.is_finite() {
                return Err(
                    Vsop87CanonicalBodyEvidenceValidationError::NonFiniteMetric {
                        body: self.body.clone(),
                        field,
                    },
                );
            }
            if value < 0.0 {
                return Err(Vsop87CanonicalBodyEvidenceValidationError::NegativeMetric {
                    body: self.body.clone(),
                    field,
                });
            }
        }

        let within_limits = self.longitude_delta_deg <= self.longitude_limit_deg
            && self.latitude_delta_deg <= self.latitude_limit_deg
            && self.distance_delta_au <= self.distance_limit_au;
        if within_limits != self.within_interim_limits {
            return Err(
                Vsop87CanonicalBodyEvidenceValidationError::InterimLimitStatusMismatch {
                    body: self.body.clone(),
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalBodyEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Validation error for a canonical VSOP87 body-evidence row that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalBodyEvidenceValidationError {
    /// The catalog no longer contains the body.
    UnknownBody { body: CelestialBody },
    /// The declared source family drifted out of sync with the current catalog.
    SourceKindMismatch {
        body: CelestialBody,
        expected: Vsop87BodySourceKind,
        found: Vsop87BodySourceKind,
    },
    /// The catalog no longer exposes a source specification for the body.
    MissingSourceSpecification { body: CelestialBody },
    /// The public source file drifted out of sync with the current catalog.
    SourceFileMismatch {
        body: CelestialBody,
        expected: &'static str,
        found: &'static str,
    },
    /// The provenance text drifted out of sync with the current catalog.
    ProvenanceMismatch {
        body: CelestialBody,
        expected: &'static str,
        found: &'static str,
    },
    /// A numeric field is not finite.
    NonFiniteMetric {
        body: CelestialBody,
        field: &'static str,
    },
    /// A numeric field is negative.
    NegativeMetric {
        body: CelestialBody,
        field: &'static str,
    },
    /// The derived interim-limit status drifted away from the current metrics.
    InterimLimitStatusMismatch { body: CelestialBody },
}

impl fmt::Display for Vsop87CanonicalBodyEvidenceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBody { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} is no longer present in the catalog"
            ),
            Self::SourceKindMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects kind {expected} but found {found}"
            ),
            Self::MissingSourceSpecification { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} no longer has a source specification in the catalog"
            ),
            Self::SourceFileMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects source file `{expected}` but found `{found}`"
            ),
            Self::ProvenanceMismatch { body, expected, found } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} expects provenance `{expected}` but found `{found}`"
            ),
            Self::NonFiniteMetric { body, field } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a non-finite `{field}` value"
            ),
            Self::NegativeMetric { body, field } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a negative `{field}` value"
            ),
            Self::InterimLimitStatusMismatch { body } => write!(
                f,
                "the VSOP87 canonical J2000 source-backed body evidence row for {body} has a mismatched interim-limit status"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalBodyEvidenceValidationError {}

/// Public summary of the canonical J2000 error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute geocentric longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum longitude delta body.
    pub max_longitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum longitude delta body.
    pub max_longitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Interim longitude delta limit for the body that drives the maximum.
    pub max_longitude_delta_limit_deg: f64,
    /// Body with the maximum absolute geocentric latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum latitude delta body.
    pub max_latitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum latitude delta body.
    pub max_latitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Interim latitude delta limit for the body that drives the maximum.
    pub max_latitude_delta_limit_deg: f64,
    /// Body with the maximum absolute geocentric distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute geocentric distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Interim distance delta limit for the body that drives the maximum.
    pub max_distance_delta_limit_au: f64,
    /// Mean absolute geocentric longitude delta in degrees.
    pub mean_longitude_delta_deg: f64,
    /// Median absolute geocentric longitude delta in degrees.
    pub median_longitude_delta_deg: f64,
    /// 95th percentile absolute geocentric longitude delta in degrees.
    pub percentile_longitude_delta_deg: f64,
    /// Root-mean-square geocentric longitude delta in degrees.
    pub rms_longitude_delta_deg: f64,
    /// Mean absolute geocentric latitude delta in degrees.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute geocentric latitude delta in degrees.
    pub median_latitude_delta_deg: f64,
    /// 95th percentile absolute geocentric latitude delta in degrees.
    pub percentile_latitude_delta_deg: f64,
    /// Root-mean-square geocentric latitude delta in degrees.
    pub rms_latitude_delta_deg: f64,
    /// Mean absolute geocentric distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute geocentric distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute geocentric distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square geocentric distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
    /// Number of samples that exceeded at least one interim limit.
    pub out_of_limit_count: usize,
    /// Whether every measured body remained within the interim limits.
    pub within_interim_limits: bool,
}

impl Vsop87CanonicalEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_canonical_epoch_evidence_summary(self)
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_body",
            &self.max_longitude_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_body",
            &self.max_latitude_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_source_file",
            self.max_longitude_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_source_file",
            self.max_latitude_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_longitude_delta_source_kind",
            "max_longitude_delta_source_file",
            &self.max_longitude_delta_body,
            self.max_longitude_delta_source_kind,
            self.max_longitude_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_latitude_delta_source_kind",
            "max_latitude_delta_source_file",
            &self.max_latitude_delta_body,
            self.max_latitude_delta_source_kind,
            self.max_latitude_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            (
                "max_longitude_delta_limit_deg",
                self.max_longitude_delta_limit_deg,
            ),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            (
                "max_latitude_delta_limit_deg",
                self.max_latitude_delta_limit_deg,
            ),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "max_distance_delta_limit_au",
                self.max_distance_delta_limit_au,
            ),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            (
                "median_longitude_delta_deg",
                self.median_longitude_delta_deg,
            ),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("median_latitude_delta_deg", self.median_latitude_delta_deg),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            validate_finite_non_negative_measure(CANONICAL_EVIDENCE_SUMMARY_LABEL, field, value)?;
        }

        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_longitude_delta_deg",
            median_field: "median_longitude_delta_deg",
            percentile_field: "percentile_longitude_delta_deg",
            rms_field: "rms_longitude_delta_deg",
            mean: self.mean_longitude_delta_deg,
            median: self.median_longitude_delta_deg,
            percentile: self.percentile_longitude_delta_deg,
            rms: self.rms_longitude_delta_deg,
            max: self.max_longitude_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_latitude_delta_deg",
            median_field: "median_latitude_delta_deg",
            percentile_field: "percentile_latitude_delta_deg",
            rms_field: "rms_latitude_delta_deg",
            mean: self.mean_latitude_delta_deg,
            median: self.median_latitude_delta_deg,
            percentile: self.percentile_latitude_delta_deg,
            rms: self.rms_latitude_delta_deg,
            max: self.max_latitude_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_distance_delta_au",
            median_field: "median_distance_delta_au",
            percentile_field: "percentile_distance_delta_au",
            rms_field: "rms_distance_delta_au",
            mean: self.mean_distance_delta_au,
            median: self.median_distance_delta_au,
            percentile: self.percentile_distance_delta_au,
            rms: self.rms_distance_delta_au,
            max: self.max_distance_delta_au,
        })?;

        let Some(body_evidence) = canonical_epoch_body_evidence() else {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "sample_count",
                },
            );
        };
        let expected_out_of_limit_count = body_evidence
            .iter()
            .filter(|evidence| !evidence.within_interim_limits)
            .count();

        if self.out_of_limit_count != expected_out_of_limit_count {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "out_of_limit_count",
                },
            );
        }
        if self.within_interim_limits != (expected_out_of_limit_count == 0) {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                    summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                    field: "within_interim_limits",
                },
            );
        }

        validate_canonical_epoch_evidence_summary_against_current_evidence(self)?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Backend-owned summary for the canonical J2000 equatorial companion evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialBodyEvidence {
    /// Body measured at the canonical epoch.
    pub body: CelestialBody,
    /// Calculation family used for the body.
    pub source_kind: Vsop87BodySourceKind,
    /// Public source file backing the body.
    pub source_file: &'static str,
    /// Human-readable provenance detail for the body.
    pub provenance: &'static str,
    /// Absolute right ascension delta in degrees.
    pub right_ascension_delta_deg: f64,
    /// Absolute declination delta in degrees.
    pub declination_delta_deg: f64,
    /// Absolute distance delta in astronomical units.
    pub distance_delta_au: f64,
}

/// Public summary of the canonical J2000 equatorial companion evidence.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Calculation family behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_file: &'static str,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Calculation family behind the maximum declination delta body.
    pub max_declination_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum declination delta body.
    pub max_declination_delta_source_file: &'static str,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Mean absolute right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th percentile absolute right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Root-mean-square right ascension delta in degrees.
    pub rms_right_ascension_delta_deg: f64,
    /// Mean absolute declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th percentile absolute declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
    /// Root-mean-square declination delta in degrees.
    pub rms_declination_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87CanonicalEquatorialEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_canonical_equatorial_evidence_summary(self)
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_body",
            &self.max_right_ascension_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_body",
            &self.max_declination_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_file",
            self.max_right_ascension_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_file",
            self.max_declination_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_kind",
            "max_right_ascension_delta_source_file",
            &self.max_right_ascension_delta_body,
            self.max_right_ascension_delta_source_kind,
            self.max_right_ascension_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_kind",
            "max_declination_delta_source_file",
            &self.max_declination_delta_body,
            self.max_declination_delta_source_kind,
            self.max_declination_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            (
                "max_right_ascension_delta_deg",
                self.max_right_ascension_delta_deg,
            ),
            ("max_declination_delta_deg", self.max_declination_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
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
            (
                "rms_right_ascension_delta_deg",
                self.rms_right_ascension_delta_deg,
            ),
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
            ("rms_declination_delta_deg", self.rms_declination_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            validate_finite_non_negative_measure(
                CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
                field,
                value,
            )?;
        }

        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_right_ascension_delta_deg",
            median_field: "median_right_ascension_delta_deg",
            percentile_field: "percentile_right_ascension_delta_deg",
            rms_field: "rms_right_ascension_delta_deg",
            mean: self.mean_right_ascension_delta_deg,
            median: self.median_right_ascension_delta_deg,
            percentile: self.percentile_right_ascension_delta_deg,
            rms: self.rms_right_ascension_delta_deg,
            max: self.max_right_ascension_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_declination_delta_deg",
            median_field: "median_declination_delta_deg",
            percentile_field: "percentile_declination_delta_deg",
            rms_field: "rms_declination_delta_deg",
            mean: self.mean_declination_delta_deg,
            median: self.median_declination_delta_deg,
            percentile: self.percentile_declination_delta_deg,
            rms: self.rms_declination_delta_deg,
            max: self.max_declination_delta_deg,
        })?;
        validate_canonical_evidence_summary_metric_order(&Vsop87MetricOrderingValidation {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            mean_field: "mean_distance_delta_au",
            median_field: "median_distance_delta_au",
            percentile_field: "percentile_distance_delta_au",
            rms_field: "rms_distance_delta_au",
            mean: self.mean_distance_delta_au,
            median: self.median_distance_delta_au,
            percentile: self.percentile_distance_delta_au,
            rms: self.rms_distance_delta_au,
            max: self.max_distance_delta_au,
        })?;

        validate_canonical_epoch_equatorial_evidence_summary_against_current_evidence(self)?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEquatorialEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

pub(crate) const CANONICAL_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 source-backed evidence";
pub(crate) const CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 equatorial companion evidence";
const CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL: &str =
    "VSOP87 canonical J2000 equatorial body-class evidence";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync {
        summary: &'static str,
        field: &'static str,
    },
    /// The sample body list now contains a duplicate entry.
    DuplicateBody {
        summary: &'static str,
        body: CelestialBody,
    },
    /// A reported peak body is absent from the sample body list.
    PeakBodyNotInSamples {
        summary: &'static str,
        field: &'static str,
        body: CelestialBody,
    },
    /// A reported source file is blank or whitespace only.
    BlankSourceFile {
        summary: &'static str,
        field: &'static str,
    },
}

impl fmt::Display for Vsop87CanonicalEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { summary, field } => write!(
                f,
                "the {summary} summary field `{field}` is out of sync with the current canonical evidence"
            ),
            Self::DuplicateBody { summary, body } => write!(
                f,
                "the {summary} summary lists body `{body}` more than once"
            ),
            Self::PeakBodyNotInSamples {
                summary,
                field,
                body,
            } => write!(
                f,
                "the {summary} summary field `{field}` points at body `{body}` which is absent from the sample body list"
            ),
            Self::BlankSourceFile { summary, field } => write!(
                f,
                "the {summary} summary field `{field}` is blank"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalEvidenceSummaryValidationError {}

fn validate_canonical_evidence_summary_bodies(
    summary: &'static str,
    sample_count: usize,
    sample_bodies: &[CelestialBody],
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if sample_count == 0 {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: "sample_count",
            },
        );
    }
    if sample_count != sample_bodies.len() {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: "sample_count",
            },
        );
    }

    for (index, body) in sample_bodies.iter().enumerate() {
        if sample_bodies[..index].contains(body) {
            return Err(
                Vsop87CanonicalEvidenceSummaryValidationError::DuplicateBody {
                    summary,
                    body: body.clone(),
                },
            );
        }
    }

    Ok(())
}

fn validate_canonical_evidence_summary_peak_body(
    summary: &'static str,
    field: &'static str,
    body: &CelestialBody,
    sample_bodies: &[CelestialBody],
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if sample_bodies.contains(body) {
        Ok(())
    } else {
        Err(
            Vsop87CanonicalEvidenceSummaryValidationError::PeakBodyNotInSamples {
                summary,
                field,
                body: body.clone(),
            },
        )
    }
}

fn validate_non_empty_source_file(
    summary: &'static str,
    field: &'static str,
    source_file: &'static str,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if source_file.trim().is_empty() {
        Err(Vsop87CanonicalEvidenceSummaryValidationError::BlankSourceFile { summary, field })
    } else {
        Ok(())
    }
}

fn validate_finite_non_negative_measure(
    summary: &'static str,
    field: &'static str,
    value: f64,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync { summary, field })
    }
}

struct Vsop87MetricOrderingValidation {
    summary: &'static str,
    mean_field: &'static str,
    median_field: &'static str,
    percentile_field: &'static str,
    rms_field: &'static str,
    mean: f64,
    median: f64,
    percentile: f64,
    rms: f64,
    max: f64,
}

fn validate_canonical_evidence_summary_metric_order(
    metric: &Vsop87MetricOrderingValidation,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    if metric.mean > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.mean_field,
            },
        );
    }
    if metric.median > metric.percentile {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.median_field,
            },
        );
    }
    if metric.percentile > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.percentile_field,
            },
        );
    }
    if metric.rms > metric.max {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary: metric.summary,
                field: metric.rms_field,
            },
        );
    }

    Ok(())
}

fn validate_canonical_evidence_summary_peak_source_metadata(
    summary: &'static str,
    field_kind: &'static str,
    field_file: &'static str,
    body: &CelestialBody,
    source_kind: Vsop87BodySourceKind,
    source_file: &'static str,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let expected_source_kind = source_kind_for_body(body.clone()).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary,
            field: field_kind,
        },
    )?;
    let expected_source_file = source_file_for_body(body).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary,
            field: field_file,
        },
    )?;

    if expected_source_kind != source_kind {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: field_kind,
            },
        );
    }
    if expected_source_file != source_file {
        return Err(
            Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                summary,
                field: field_file,
            },
        );
    }

    Ok(())
}

static SOURCE_AUDITS: OnceLock<Vec<Vsop87SourceAudit>> = OnceLock::new();
static GENERATED_BINARY_AUDITS: OnceLock<Vec<Vsop87GeneratedBlobAudit>> = OnceLock::new();

/// Returns the structured source documentation for the current VSOP87-backed bodies.
///
/// Each entry captures the current public source file, VSOP87 variant, frame,
/// units, reduction note, transform note, truncation policy, and date-range
/// metadata for one body in the source-backed catalog.
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

/// Returns the release-facing source-specification catalog string.
pub fn source_specifications_for_report() -> String {
    let specs = source_specifications();
    match validate_source_specifications(&specs) {
        Ok(()) => format_source_specifications(&specs),
        Err(error) => format!("VSOP87 source specifications: unavailable ({error})"),
    }
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

/// Returns the release-facing frame-treatment summary for VSOP87-backed results.
///
/// The backend-owned note is validated before the compact report line is
/// rendered, so a drifted summary becomes an unavailable report rather than a
/// stale cached string.
pub fn frame_treatment_summary_for_report() -> String {
    let summary = frame_treatment_summary_details();
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line.to_string(),
        Err(error) => format!("VSOP87 frame treatment unavailable ({error})"),
    }
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
    FieldOutOfSync { field: &'static str },
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

/// Returns the release-facing VSOP87 request policy summary string.
pub fn vsop87_request_policy_summary_for_report() -> String {
    let policy = vsop87_request_policy();
    match policy.validate() {
        Ok(()) => policy.to_string(),
        Err(error) => format!("VSOP87 request policy: unavailable ({error})"),
    }
}

/// Returns the reproducibility audit records for the current VSOP87-backed bodies.
pub fn source_audits() -> Vec<Vsop87SourceAudit> {
    SOURCE_AUDITS
        .get_or_init(|| {
            body_catalog_entries()
                .iter()
                .filter_map(|entry| {
                    entry.source_specification.as_ref().map(|spec| {
                        let source = source_text_for_file(spec.source_file)
                            .expect("known VSOP87 source file");
                        Vsop87SourceAudit {
                            body: spec.body.clone(),
                            source_file: spec.source_file,
                            byte_length: source.len(),
                            line_count: source.lines().count(),
                            term_count: count_vsop87_terms(source),
                            fingerprint: fnv1a_64(source.as_bytes()),
                        }
                    })
                })
                .collect()
        })
        .clone()
}

/// Validates the reproducibility audit records for the current VSOP87-backed bodies.
pub fn validate_source_audits(
    audits: &[Vsop87SourceAudit],
) -> Result<(), Vsop87SourceAuditValidationError> {
    for (position, audit) in audits.iter().enumerate() {
        audit.validate_at_position(position + 1)?;
    }

    Ok(())
}

/// Returns a small reproducibility summary for the current VSOP87-backed bodies.
pub fn source_audit_summary() -> Vsop87SourceAuditSummary {
    let audits = source_audits();
    let source_specs = source_specifications();
    Vsop87SourceAuditSummary {
        source_count: audits.len(),
        source_bodies: source_backed_body_order(),
        source_files: source_specs.iter().map(|spec| spec.source_file).collect(),
        vendored_full_file_count: audits
            .iter()
            .filter(|audit| audit.source_file.starts_with("VSOP87B."))
            .count(),
        fingerprint_count: audits.len(),
        total_term_count: audits.iter().map(|audit| audit.term_count).sum(),
        max_line_count: audits
            .iter()
            .map(|audit| audit.line_count)
            .max()
            .unwrap_or(0),
        max_byte_length: audits
            .iter()
            .map(|audit| audit.byte_length)
            .max()
            .unwrap_or(0),
    }
}

/// Formats the current VSOP87 reproducibility audit for reporting.
pub fn format_source_audit_summary(summary: &Vsop87SourceAuditSummary) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_source_audit_summary_for_report(
    summary: &Vsop87SourceAuditSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 source audit: unavailable ({error})"),
    }
}

/// Returns the release-facing reproducibility audit summary string.
pub fn source_audit_summary_for_report() -> String {
    format_validated_source_audit_summary_for_report(&source_audit_summary())
}

/// A reproducibility audit record for one checked-in generated VSOP87B blob.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87GeneratedBlobAudit {
    /// Body covered by the generated blob.
    pub body: CelestialBody,
    /// Public coefficient file backing the body.
    pub source_file: &'static str,
    /// Checked-in generated blob byte length.
    pub byte_length: usize,
    /// Deterministic 64-bit fingerprint of the checked-in generated blob.
    pub fingerprint: u64,
}

/// Validation errors for a checked-in generated VSOP87B blob audit record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87GeneratedBlobAuditValidationError {
    /// The audit record does not name a source file.
    BlankSourceFile {
        position: usize,
        body: CelestialBody,
    },
    /// The audit record points at an empty generated blob.
    EmptyBlob {
        position: usize,
        source_file: &'static str,
    },
    /// The audit record references a source file that does not exist in the current source catalog.
    UnknownSourceFile {
        position: usize,
        source_file: &'static str,
    },
    /// The audit record references a checked-in blob that is missing from the current source catalog.
    MissingGeneratedBlob {
        position: usize,
        body: CelestialBody,
        source_file: &'static str,
    },
    /// The audit record body/source pairing does not match the current source catalog.
    BodySourceMismatch {
        position: usize,
        body: CelestialBody,
        source_file: &'static str,
        expected_body: CelestialBody,
    },
}

impl fmt::Display for Vsop87GeneratedBlobAuditValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlankSourceFile { position, body } => write!(
                f,
                "generated binary audit record #{position} has a blank source file for {body}"
            ),
            Self::EmptyBlob { position, source_file } => write!(
                f,
                "generated binary audit record #{position} has an empty blob for source file `{source_file}`"
            ),
            Self::UnknownSourceFile { position, source_file } => write!(
                f,
                "generated binary audit record #{position} references unknown source file `{source_file}`"
            ),
            Self::MissingGeneratedBlob {
                position,
                body,
                source_file,
            } => write!(
                f,
                "generated binary audit record #{position} is missing the checked-in blob for {body} at source file `{source_file}`"
            ),
            Self::BodySourceMismatch {
                position,
                body,
                source_file,
                expected_body,
            } => write!(
                f,
                "generated binary audit record #{position} uses source file `{source_file}`, which belongs to {expected_body} rather than {body}"
            ),
        }
    }
}

impl std::error::Error for Vsop87GeneratedBlobAuditValidationError {}

impl Vsop87GeneratedBlobAudit {
    fn validate_at_position(
        &self,
        position: usize,
    ) -> Result<(), Vsop87GeneratedBlobAuditValidationError> {
        if self.source_file.trim().is_empty() {
            return Err(Vsop87GeneratedBlobAuditValidationError::BlankSourceFile {
                position,
                body: self.body.clone(),
            });
        }
        if self.byte_length == 0 {
            return Err(Vsop87GeneratedBlobAuditValidationError::EmptyBlob {
                position,
                source_file: self.source_file,
            });
        }

        let Some(expected_body) = source_specifications()
            .into_iter()
            .find(|spec| spec.source_file == self.source_file)
            .map(|spec| spec.body)
        else {
            return Err(Vsop87GeneratedBlobAuditValidationError::UnknownSourceFile {
                position,
                source_file: self.source_file,
            });
        };

        if expected_body != self.body {
            return Err(
                Vsop87GeneratedBlobAuditValidationError::BodySourceMismatch {
                    position,
                    body: self.body.clone(),
                    source_file: self.source_file,
                    expected_body,
                },
            );
        }

        Ok(())
    }

    /// Validates the generated blob audit record against the current source catalog.
    pub fn validate(&self) -> Result<(), Vsop87GeneratedBlobAuditValidationError> {
        self.validate_at_position(1)
    }
}

pub(crate) fn build_generated_binary_audits_with_lookup(
    lookup: impl Fn(&str) -> Option<&'static [u8]>,
) -> Result<Vec<Vsop87GeneratedBlobAudit>, Vsop87GeneratedBlobAuditValidationError> {
    source_specifications()
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let position = index + 1;
            let Some(blob) = lookup(spec.source_file) else {
                return Err(
                    Vsop87GeneratedBlobAuditValidationError::MissingGeneratedBlob {
                        position,
                        body: spec.body,
                        source_file: spec.source_file,
                    },
                );
            };

            Ok(Vsop87GeneratedBlobAudit {
                body: spec.body,
                source_file: spec.source_file,
                byte_length: blob.len(),
                fingerprint: fnv1a_64(blob),
            })
        })
        .collect()
}

/// Summary metrics for the current VSOP87 generated-blob audit manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87GeneratedBlobAuditSummary {
    /// Number of checked-in generated blobs represented in the audit manifest.
    pub blob_count: usize,
    /// Bodies represented in the audit manifest.
    pub source_bodies: Vec<CelestialBody>,
    /// Source files represented in the audit manifest.
    pub source_files: Vec<&'static str>,
    /// Number of source files represented in the audit manifest.
    pub source_file_count: usize,
    /// Total checked-in blob byte length across the manifest.
    pub total_byte_length: usize,
    /// Maximum checked-in blob byte length across the manifest.
    pub max_byte_length: usize,
    /// Number of deterministic fingerprints recorded in the audit manifest.
    pub fingerprint_count: usize,
}

/// Validation error for a VSOP87 generated-blob summary that drifted from the current manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87GeneratedBlobAuditSummaryValidationError {
    /// A rendered summary field no longer matches the current generated-blob manifest.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87GeneratedBlobAuditSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 generated binary audit summary field `{field}` is out of sync with the current manifest"
            ),
        }
    }
}

impl std::error::Error for Vsop87GeneratedBlobAuditSummaryValidationError {}

impl Vsop87GeneratedBlobAuditSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 generated binary audit: {} checked-in blobs across {} source files (bodies: {}; files: {}); {} total bytes, max blob size {} bytes, {} deterministic fingerprints",
            self.blob_count,
            self.source_file_count,
            join_display(&self.source_bodies),
            join_display(&self.source_files),
            self.total_byte_length,
            self.max_byte_length,
            self.fingerprint_count
        )
    }

    /// Returns the rendered summary line after validating the cached manifest snapshot.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87GeneratedBlobAuditSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current generated-blob manifest.
    pub fn validate(&self) -> Result<(), Vsop87GeneratedBlobAuditSummaryValidationError> {
        let audits = generated_binary_audits();
        validate_generated_binary_audits(&audits).map_err(|_| {
            Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                field: "audit_records",
            }
        })?;
        let source_specs = source_specifications();
        let expected_source_files = source_specs
            .iter()
            .map(|spec| spec.source_file)
            .collect::<Vec<_>>();

        if self.blob_count != audits.len() {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "blob_count",
                },
            );
        }
        if self.source_bodies != source_backed_body_order() {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "source_bodies",
                },
            );
        }
        if self.source_files != expected_source_files {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "source_files",
                },
            );
        }
        if self.source_file_count != audits.len() {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "source_file_count",
                },
            );
        }
        if self.total_byte_length != audits.iter().map(|audit| audit.byte_length).sum::<usize>() {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "total_byte_length",
                },
            );
        }
        if self.max_byte_length
            != audits
                .iter()
                .map(|audit| audit.byte_length)
                .max()
                .unwrap_or(0)
        {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "max_byte_length",
                },
            );
        }
        if self.fingerprint_count != audits.len() {
            return Err(
                Vsop87GeneratedBlobAuditSummaryValidationError::FieldOutOfSync {
                    field: "fingerprint_count",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87GeneratedBlobAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the reproducibility audit records for the current checked-in generated blobs.
pub fn generated_binary_audits() -> Vec<Vsop87GeneratedBlobAudit> {
    GENERATED_BINARY_AUDITS
        .get_or_init(|| {
            build_generated_binary_audits_with_lookup(
                checked_in_generated_vsop87b_table_bytes_for_source_file,
            )
            .expect("known VSOP87 generated blob")
        })
        .clone()
}

fn generated_binary_audit_summary_from_audits(
    audits: &[Vsop87GeneratedBlobAudit],
) -> Vsop87GeneratedBlobAuditSummary {
    Vsop87GeneratedBlobAuditSummary {
        blob_count: audits.len(),
        source_bodies: audits.iter().map(|audit| audit.body.clone()).collect(),
        source_files: audits.iter().map(|audit| audit.source_file).collect(),
        source_file_count: audits.len(),
        total_byte_length: audits.iter().map(|audit| audit.byte_length).sum(),
        max_byte_length: audits
            .iter()
            .map(|audit| audit.byte_length)
            .max()
            .unwrap_or(0),
        fingerprint_count: audits.len(),
    }
}

/// Returns a small reproducibility summary for the current generated VSOP87B blobs.
pub fn generated_binary_audit_summary() -> Vsop87GeneratedBlobAuditSummary {
    let audits = generated_binary_audits();
    generated_binary_audit_summary_from_audits(&audits)
}

/// Validates the generated blob records against the current source catalog.
pub fn validate_generated_binary_audits(
    audits: &[Vsop87GeneratedBlobAudit],
) -> Result<(), Vsop87GeneratedBlobAuditValidationError> {
    for (index, audit) in audits.iter().enumerate() {
        audit.validate_at_position(index + 1)?;
    }

    Ok(())
}

/// Formats the checked-in generated VSOP87B blob audit for reporting.
pub fn format_generated_binary_audit_summary(summary: &Vsop87GeneratedBlobAuditSummary) -> String {
    summary.summary_line()
}

pub(crate) fn format_validated_generated_binary_audit_summary_for_report(
    summary: &Vsop87GeneratedBlobAuditSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 generated binary audit: unavailable ({error})"),
    }
}

/// Returns the release-facing generated binary audit summary string.
pub fn generated_binary_audit_summary_for_report() -> String {
    let audits = match build_generated_binary_audits_with_lookup(
        checked_in_generated_vsop87b_table_bytes_for_source_file,
    ) {
        Ok(audits) => audits,
        Err(error) => return format!("VSOP87 generated binary audit: unavailable ({error})"),
    };

    if let Err(error) = validate_generated_binary_audits(&audits) {
        return format!("VSOP87 generated binary audit: unavailable ({error})");
    }

    let summary = generated_binary_audit_summary_from_audits(&audits);
    format_validated_generated_binary_audit_summary_for_report(&summary)
}

/// Returns a summary of the current VSOP87 source-documentation catalog.
pub fn source_documentation_summary() -> Vsop87SourceDocumentationSummary {
    let source_specs = source_specifications();
    let source_backed_profiles = source_backed_body_profiles();
    let fallback_profiles = fallback_body_profiles();

    let fallback_bodies = fallback_profiles
        .iter()
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();

    let mut date_ranges = source_specs
        .iter()
        .map(|spec| spec.date_range)
        .collect::<Vec<_>>();
    date_ranges.sort_unstable();
    date_ranges.dedup();

    let source_backed_bodies = source_backed_body_order();
    let generated_binary_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let vendored_full_file_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let truncated_bodies = source_backed_profiles
        .iter()
        .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
        .map(|profile| profile.body.clone())
        .collect::<Vec<_>>();
    let source_files = source_specs
        .iter()
        .map(|spec| spec.source_file)
        .collect::<Vec<_>>();

    Vsop87SourceDocumentationSummary {
        source_specification_count: source_specs.len(),
        source_backed_profile_count: source_backed_profiles.len(),
        source_backed_bodies,
        source_files,
        generated_binary_bodies,
        vendored_full_file_bodies,
        truncated_bodies,
        vendored_full_file_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count(),
        generated_binary_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count(),
        truncated_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count(),
        fallback_profile_count: fallback_profiles.len(),
        fallback_bodies,
        date_ranges,
    }
}

fn format_celestial_bodies(bodies: &[CelestialBody]) -> String {
    bodies
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Formats the current VSOP87 source-documentation catalog for reporting.
pub fn format_source_documentation_summary(summary: &Vsop87SourceDocumentationSummary) -> String {
    let source_backed_bodies = if summary.source_backed_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.source_backed_bodies)
    };
    let fallback_bodies = if summary.fallback_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.fallback_bodies)
    };
    let source_files = if summary.source_files.is_empty() {
        "none".to_string()
    } else {
        summary.source_files.join(", ")
    };
    let date_ranges = if summary.date_ranges.is_empty() {
        "none".to_string()
    } else {
        summary.date_ranges.join("; ")
    };
    let generated_binary_bodies = if summary.generated_binary_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.generated_binary_bodies)
    };
    let vendored_full_file_bodies = if summary.vendored_full_file_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.vendored_full_file_bodies)
    };
    let truncated_bodies = if summary.truncated_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.truncated_bodies)
    };
    let fallback_profile_label = if summary.fallback_profile_count == 1 {
        "approximate fallback mean-element body profile"
    } else {
        "approximate fallback mean-element body profiles"
    };

    format!(
        "VSOP87 source documentation: {} source specs, {} source-backed body profiles, {} {} ({}); source-backed bodies: {}; source files: {}; source-backed breakdown: {} generated binary bodies ({}), {} vendored full-file bodies ({}), {} truncated slice bodies ({}); date ranges: {}",
        summary.source_specification_count,
        summary.source_backed_profile_count,
        summary.fallback_profile_count,
        fallback_profile_label,
        fallback_bodies,
        source_backed_bodies,
        source_files,
        summary.generated_binary_profile_count,
        generated_binary_bodies,
        summary.vendored_full_file_profile_count,
        vendored_full_file_bodies,
        summary.truncated_profile_count,
        truncated_bodies,
        date_ranges,
    )
}

/// Returns the release-facing summary string for the current VSOP87 source-documentation catalog.
///
/// The compact provenance line is rendered only after the catalog-health gate
/// confirms that the source/file/body partitioning still matches the current
/// canonical VSOP87 inputs.
pub fn source_documentation_summary_for_report() -> String {
    format_validated_source_documentation_summary_for_report(&source_documentation_summary())
}

/// Returns a consistency check for the current VSOP87 source-documentation catalog.
fn source_documentation_fields_are_consistent(source_specs: &[Vsop87SourceSpecification]) -> bool {
    source_specs.iter().all(|spec| spec.validate().is_ok())
}

pub fn source_documentation_health_summary() -> Vsop87SourceDocumentationHealthSummary {
    let summary = source_documentation_summary();
    let source_specs = source_specifications();
    let body_profile_count = body_catalog_entries().len();
    let source_file_count = summary.source_files.len();
    let issues = source_documentation_health_issues(
        &summary,
        &source_specs,
        body_profile_count,
        source_file_count,
    );

    let consistent = issues.is_empty();
    let documentation_consistent = summary.source_specification_count == source_specs.len()
        && source_documentation_fields_are_consistent(&source_specs);

    let source_backed_partition_bodies = source_documentation_partition_bodies(&summary);

    Vsop87SourceDocumentationHealthSummary {
        consistent,
        documentation_consistent,
        issues,
        source_specification_count: summary.source_specification_count,
        source_file_count,
        source_files: summary.source_files,
        source_backed_profile_count: summary.source_backed_profile_count,
        source_backed_bodies: summary.source_backed_bodies,
        source_backed_partition_bodies,
        generated_binary_bodies: summary.generated_binary_bodies,
        vendored_full_file_bodies: summary.vendored_full_file_bodies,
        truncated_bodies: summary.truncated_bodies,
        body_profile_count,
        generated_binary_profile_count: summary.generated_binary_profile_count,
        vendored_full_file_profile_count: summary.vendored_full_file_profile_count,
        truncated_profile_count: summary.truncated_profile_count,
        fallback_profile_count: summary.fallback_profile_count,
        fallback_bodies: summary.fallback_bodies,
    }
}

/// Formats the current VSOP87 source-documentation health check for reporting.
pub fn format_source_documentation_health_summary(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    let issues = if summary.issues.is_empty() {
        String::new()
    } else {
        format!("; issues: {}", format_issue_labels(&summary.issues))
    };

    format!(
        "VSOP87 source documentation health: {} ({} source specs, {} source files, {} source-backed profiles, {} body profiles; {} generated binary profiles ({}), {} vendored full-file profiles ({}), {} truncated profiles ({}), {} approximate fallback profiles ({}); source files: {}; source-backed order: {}; source-backed partition order: {}; fallback order: {}; documented fields: {}){}",
        if summary.consistent { "ok" } else { "needs attention" },
        summary.source_specification_count,
        summary.source_file_count,
        summary.source_backed_profile_count,
        summary.body_profile_count,
        summary.generated_binary_profile_count,
        format_bodies(&summary.generated_binary_bodies),
        summary.vendored_full_file_profile_count,
        format_bodies(&summary.vendored_full_file_bodies),
        summary.truncated_profile_count,
        format_bodies(&summary.truncated_bodies),
        summary.fallback_profile_count,
        format_bodies(&summary.fallback_bodies),
        format_source_files(&summary.source_files),
        format_bodies(&summary.source_backed_bodies),
        format_bodies(&summary.source_backed_partition_bodies),
        format_bodies(&summary.fallback_bodies),
        if summary.documentation_consistent {
            "variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range"
        } else {
            "needs attention"
        },
        issues,
    )
}

/// Returns the source-backed partition order used by the VSOP87 source
/// documentation health check.
///
/// The generated-binary, vendored full-file, and truncated slices are kept in
/// this order so regeneration tooling and release reports can reuse the same
/// backend-owned partitioning without reconstructing it locally.
pub fn source_documentation_partition_bodies(
    summary: &Vsop87SourceDocumentationSummary,
) -> Vec<CelestialBody> {
    summary
        .generated_binary_bodies
        .iter()
        .chain(summary.vendored_full_file_bodies.iter())
        .chain(summary.truncated_bodies.iter())
        .cloned()
        .collect()
}

pub(crate) fn source_documentation_health_issues(
    summary: &Vsop87SourceDocumentationSummary,
    source_specs: &[Vsop87SourceSpecification],
    body_profile_count: usize,
    source_file_count: usize,
) -> Vec<Vsop87SourceDocumentationHealthIssue> {
    let expected_source_files = source_specs
        .iter()
        .map(|spec| spec.source_file)
        .collect::<Vec<_>>();
    let expected_source_backed_bodies = source_documentation_partition_bodies(summary);
    let mut issues = Vec::new();

    if summary.source_specification_count != source_file_count {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceSpecificationFileCountMismatch);
    }
    if summary.source_files != expected_source_files {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceFileOrderMismatch);
    }
    if summary.source_backed_bodies != expected_source_backed_bodies {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedBodyOrderMismatch);
    }
    if !body_labels_are_unique(&summary.source_backed_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedBodyDuplicate);
    }
    if !body_labels_are_unique(&summary.fallback_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::FallbackBodyDuplicate);
    }
    if !body_lists_are_disjoint(&summary.source_backed_bodies, &summary.fallback_bodies) {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedFallbackBodyOverlap);
    }
    if summary.source_backed_profile_count
        != summary.generated_binary_profile_count
            + summary.vendored_full_file_profile_count
            + summary.truncated_profile_count
    {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceBackedProfilePartitionMismatch);
    }
    if summary.source_backed_profile_count + summary.fallback_profile_count != body_profile_count {
        issues.push(Vsop87SourceDocumentationHealthIssue::BodyProfileCoverageMismatch);
    }
    if summary.source_specification_count != source_specs.len() {
        issues.push(Vsop87SourceDocumentationHealthIssue::SourceSpecificationCatalogCountMismatch);
    }
    if !source_documentation_fields_are_consistent(source_specs) {
        issues.push(Vsop87SourceDocumentationHealthIssue::DocumentedFieldMismatch);
    }

    issues
}

fn format_bodies(bodies: &[CelestialBody]) -> String {
    if bodies.is_empty() {
        "none".to_string()
    } else {
        bodies
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_issue_labels<T: fmt::Display>(issues: &[T]) -> String {
    if issues.is_empty() {
        "none".to_string()
    } else {
        join_display(issues)
    }
}

fn body_labels_are_unique(bodies: &[CelestialBody]) -> bool {
    let mut seen = BTreeSet::new();

    for body in bodies {
        if !seen.insert(body.to_string()) {
            return false;
        }
    }

    true
}

fn body_lists_are_disjoint(left: &[CelestialBody], right: &[CelestialBody]) -> bool {
    let mut seen = BTreeSet::new();

    for body in left {
        seen.insert(body.to_string());
    }

    right.iter().all(|body| !seen.contains(&body.to_string()))
}

fn format_source_files(source_files: &[&'static str]) -> String {
    if source_files.is_empty() {
        "none".to_string()
    } else {
        source_files.join(", ")
    }
}

pub(crate) fn format_validated_source_documentation_health_summary_for_report(
    summary: &Vsop87SourceDocumentationHealthSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source documentation health: unavailable ({error})"),
    }
}

pub(crate) fn format_validated_source_documentation_summary_for_report(
    summary: &Vsop87SourceDocumentationSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(summary_line) => summary_line,
        Err(error) => format!("VSOP87 source documentation: unavailable ({error})"),
    }
}

/// Returns the release-facing source-documentation health string.
pub fn source_documentation_health_summary_for_report() -> String {
    format_validated_source_documentation_health_summary_for_report(
        &source_documentation_health_summary(),
    )
}

/// Backend-owned summary of the canonical VSOP87 body evidence envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceBodyEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Number of samples within the interim limits.
    pub within_interim_limits_count: usize,
    /// Number of vendored full-file source-backed samples.
    pub vendored_full_file_count: usize,
    /// Number of generated-binary source-backed samples.
    pub generated_binary_count: usize,
    /// Number of truncated-slice source-backed samples.
    pub truncated_count: usize,
    /// Number of bodies outside the current interim limits.
    pub outside_interim_limit_count: usize,
    /// Bodies outside the current interim limits.
    pub outside_interim_limit_bodies: Vec<CelestialBody>,
}

impl Vsop87SourceBodyEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_body_evidence_summary(self)
    }

    /// Returns the validated summary line used by release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceBodyEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived counts.
    pub fn validate(&self) -> Result<(), Vsop87SourceBodyEvidenceSummaryValidationError> {
        let Some(expected) = source_body_evidence_summary() else {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        };

        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_count == 0 {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.within_interim_limits_count != expected.within_interim_limits_count {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "within_interim_limits_count",
                },
            );
        }
        if self.outside_interim_limit_count != expected.outside_interim_limit_count {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_count",
                },
            );
        }
        if self.within_interim_limits_count + self.outside_interim_limit_count != self.sample_count
        {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "interim_limit_counts",
                },
            );
        }
        if self.outside_interim_limit_count != self.outside_interim_limit_bodies.len() {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.outside_interim_limit_bodies != expected.outside_interim_limit_bodies {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.sample_bodies) {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.outside_interim_limit_bodies) {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.vendored_full_file_count + self.generated_binary_count + self.truncated_count
            != self.sample_count
        {
            return Err(
                Vsop87SourceBodyEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "source_kind_counts",
                },
            );
        }

        Ok(())
    }
}

/// Validation error for a VSOP87 source-backed body-evidence summary that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceBodyEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87SourceBodyEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source-backed body evidence summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceBodyEvidenceSummaryValidationError {}

impl fmt::Display for Vsop87SourceBodyEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a backend-owned summary of the canonical VSOP87 body evidence.
pub fn source_body_evidence_summary() -> Option<Vsop87SourceBodyEvidenceSummary> {
    let evidence = canonical_epoch_body_evidence()?;
    Some(Vsop87SourceBodyEvidenceSummary {
        sample_count: evidence.len(),
        sample_bodies: evidence.iter().map(|row| row.body.clone()).collect(),
        within_interim_limits_count: evidence
            .iter()
            .filter(|row| row.within_interim_limits)
            .count(),
        vendored_full_file_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count(),
        generated_binary_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count(),
        truncated_count: evidence
            .iter()
            .filter(|row| row.source_kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count(),
        outside_interim_limit_count: evidence
            .iter()
            .filter(|row| !row.within_interim_limits)
            .count(),
        outside_interim_limit_bodies: evidence
            .into_iter()
            .filter(|row| !row.within_interim_limits)
            .map(|row| row.body)
            .collect(),
    })
}

/// Formats the canonical VSOP87 J2000 evidence summary for reporting.
pub fn format_canonical_epoch_evidence_summary(summary: &Vsop87CanonicalEvidenceSummary) -> String {
    format!(
        "VSOP87 canonical J2000 source-backed evidence: {} samples, bodies: {}, status {}, mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, out-of-limit samples {}, max Δlon={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δlat={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δdist={:.12} AU (limit {:.12} AU, margin {:+.12} AU; {}; {}; {})",
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        if summary.within_interim_limits {
            "within interim limits"
        } else {
            "outside interim limits"
        },
        summary.mean_longitude_delta_deg,
        summary.median_longitude_delta_deg,
        summary.percentile_longitude_delta_deg,
        summary.rms_longitude_delta_deg,
        summary.mean_latitude_delta_deg,
        summary.median_latitude_delta_deg,
        summary.percentile_latitude_delta_deg,
        summary.rms_latitude_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.out_of_limit_count,
        summary.max_longitude_delta_deg,
        summary.max_longitude_delta_limit_deg,
        summary.max_longitude_delta_limit_deg - summary.max_longitude_delta_deg,
        summary.max_longitude_delta_body,
        summary.max_longitude_delta_source_kind,
        summary.max_longitude_delta_source_file,
        summary.max_latitude_delta_deg,
        summary.max_latitude_delta_limit_deg,
        summary.max_latitude_delta_limit_deg - summary.max_latitude_delta_deg,
        summary.max_latitude_delta_body,
        summary.max_latitude_delta_source_kind,
        summary.max_latitude_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_limit_au,
        summary.max_distance_delta_limit_au - summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

pub(crate) fn format_validated_canonical_epoch_evidence_summary_for_report(
    summary: &Vsop87CanonicalEvidenceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("{CANONICAL_EVIDENCE_SUMMARY_LABEL}: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical VSOP87 J2000 evidence summary string.
pub fn canonical_epoch_evidence_summary_for_report() -> String {
    match canonical_epoch_evidence_summary() {
        Some(summary) => format_validated_canonical_epoch_evidence_summary_for_report(&summary),
        None => format!("{CANONICAL_EVIDENCE_SUMMARY_LABEL}: unavailable"),
    }
}

const CANONICAL_OUTLIER_NOTE_LABEL: &str = "VSOP87 canonical J2000 interim outliers";

/// Public summary of the canonical J2000 interim-outlier note.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalOutlierSummary {
    /// Bodies outside the current interim limits.
    pub outlier_bodies: Vec<CelestialBody>,
}

/// Structured validation errors for a canonical J2000 interim-outlier summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalOutlierSummaryValidationError {
    /// The summary drifted away from the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87CanonicalOutlierSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 canonical outlier summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalOutlierSummaryValidationError {}

impl Vsop87CanonicalOutlierSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        if self.outlier_bodies.is_empty() {
            format!("{CANONICAL_OUTLIER_NOTE_LABEL}: none")
        } else {
            format!(
                "{CANONICAL_OUTLIER_NOTE_LABEL}: {}",
                format_celestial_bodies(&self.outlier_bodies)
            )
        }
    }

    /// Returns the validated compact summary line used in release-facing reporting.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87CanonicalOutlierSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalOutlierSummaryValidationError> {
        let Some(expected) = canonical_epoch_outlier_summary() else {
            return Err(
                Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                    field: "outlier_bodies",
                },
            );
        };

        if self.outlier_bodies != expected.outlier_bodies {
            return Err(
                Vsop87CanonicalOutlierSummaryValidationError::FieldOutOfSync {
                    field: "outlier_bodies",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalOutlierSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the release-facing canonical VSOP87 J2000 interim-outlier summary.
pub fn canonical_epoch_outlier_summary() -> Option<Vsop87CanonicalOutlierSummary> {
    let outlier_bodies = canonical_epoch_body_evidence().map(|evidence| {
        evidence
            .into_iter()
            .filter(|row| !row.within_interim_limits)
            .map(|row| row.body)
            .collect::<Vec<_>>()
    })?;

    Some(Vsop87CanonicalOutlierSummary { outlier_bodies })
}

/// Returns a concise note describing any canonical J2000 bodies outside the
/// current interim limits.
pub fn canonical_epoch_outlier_note_for_report() -> String {
    match canonical_epoch_outlier_summary() {
        Some(summary) => match summary.validated_summary_line() {
            Ok(line) => line,
            Err(error) => format!("{CANONICAL_OUTLIER_NOTE_LABEL}: unavailable ({error})"),
        },
        None => format!("{CANONICAL_OUTLIER_NOTE_LABEL}: unavailable"),
    }
}

fn canonical_batch_parity_counts(
    backend: &Vsop87Backend,
    requests: &[EphemerisRequest],
) -> Option<(Vec<CelestialBody>, usize, usize, usize, usize)> {
    let results = backend.positions(requests).ok()?;

    if results.len() != requests.len() {
        return None;
    }

    let mut sample_bodies = Vec::with_capacity(results.len());
    let mut exact_count = 0usize;
    let mut interpolated_count = 0usize;
    let mut approximate_count = 0usize;
    let mut unknown_count = 0usize;

    for (request, result) in requests.iter().zip(results.iter()) {
        let single = backend.position(request).ok()?;
        if single != *result {
            return None;
        }

        sample_bodies.push(result.body.clone());
        match result.quality {
            QualityAnnotation::Exact => exact_count += 1,
            QualityAnnotation::Interpolated => interpolated_count += 1,
            QualityAnnotation::Approximate => approximate_count += 1,
            QualityAnnotation::Unknown => unknown_count += 1,
            _ => unknown_count += 1,
        }
    }

    Some((
        sample_bodies,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    ))
}

fn validate_batch_parity_quality_counts(
    actual_counts: (usize, usize, usize, usize),
    expected_counts: (usize, usize, usize, usize),
) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
    if actual_counts != expected_counts {
        return Err(
            Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "quality_counts",
            },
        );
    }

    Ok(())
}

fn canonical_j2000_batch_parity_expected_bodies() -> Vec<CelestialBody> {
    canonical_epoch_samples()
        .iter()
        .map(|sample| sample.body.clone())
        .collect()
}

fn canonical_j1900_batch_parity_expected_bodies() -> Vec<CelestialBody> {
    Vsop87Backend::supported_bodies().to_vec()
}

/// Validation error for a VSOP87 canonical batch-parity summary that drifted
/// from the current backend-derived counts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87CanonicalBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87CanonicalBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 canonical batch parity summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87CanonicalBatchParitySummaryValidationError {}

/// Backend-owned summary for the canonical J2000 batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalJ2000BatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalJ2000BatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J2000 batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j2000_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_epoch_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalJ2000BatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 batch-path regression summary.
pub fn canonical_j2000_batch_parity_summary() -> Option<Vsop87CanonicalJ2000BatchParitySummary> {
    let backend = Vsop87Backend::new();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    let requests = canonical_epoch_requests();
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87CanonicalJ2000BatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_canonical_j2000_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalJ2000BatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical J2000 batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical J2000 batch-path regression summary string.
pub fn canonical_j2000_batch_parity_summary_for_report() -> String {
    match canonical_j2000_batch_parity_summary() {
        Some(summary) => format_validated_canonical_j2000_batch_parity_summary_for_report(&summary),
        None => "VSOP87 canonical J2000 batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the canonical mixed TT/TDB batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of TT-tagged results observed in the batch regression.
    pub tt_request_count: usize,
    /// Number of TDB-tagged results observed in the batch regression.
    pub tdb_request_count: usize,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical mixed TT/TDB batch parity: {} requests across {} bodies ({}) at JD {:.1} (TT/TDB mix) in {} frame; TT requests={}, TDB requests={}, quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.frame,
            self.tt_request_count,
            self.tdb_request_count,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    ///
    /// The canonical alternating TT/TDB request counts already imply the mixed-slice posture for
    /// this fixed 11-body slice, so the validation path keeps its focus on the exact counts,
    /// bodies, epoch, and frame rather than introducing a separate degenerate-mix guard.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j2000_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        let expected_tt_request_count = self.sample_count.div_ceil(2);
        let expected_tdb_request_count = self.sample_count / 2;
        if self.tt_request_count != expected_tt_request_count {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tt_request_count",
                },
            );
        }
        if self.tdb_request_count != expected_tdb_request_count {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "tdb_request_count",
                },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_mixed_time_scale_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalMixedTimeScaleBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical mixed TT/TDB batch-path regression summary.
pub fn canonical_mixed_time_scale_batch_parity_summary(
) -> Option<Vsop87CanonicalMixedTimeScaleBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt);
    let requests = canonical_mixed_time_scale_batch_parity_requests();
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;
    let tt_request_count = requests
        .iter()
        .filter(|request| request.instant.scale == TimeScale::Tt)
        .count();
    let tdb_request_count = requests.len() - tt_request_count;

    Some(Vsop87CanonicalMixedTimeScaleBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        tt_request_count,
        tdb_request_count,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalMixedTimeScaleBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical mixed TT/TDB batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical mixed TT/TDB batch-path regression summary string.
pub fn canonical_mixed_time_scale_batch_parity_summary_for_report() -> String {
    match canonical_mixed_time_scale_batch_parity_summary() {
        Some(summary) => {
            format_validated_canonical_mixed_time_scale_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 canonical mixed TT/TDB batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the canonical J1900 batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalJ1900BatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87CanonicalJ1900BatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 canonical J1900 batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != canonical_j1900_batch_parity_expected_bodies() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = canonical_j1900_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalJ1900BatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J1900 batch-path regression summary.
pub fn canonical_j1900_batch_parity_summary() -> Option<Vsop87CanonicalJ1900BatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = canonical_j1900_equatorial_batch_parity_requests();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87CanonicalJ1900BatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_canonical_j1900_batch_parity_summary_for_report(
    summary: &Vsop87CanonicalJ1900BatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 canonical J1900 batch parity: unavailable ({error})"),
    }
}

/// Returns the release-facing canonical J1900 batch-path regression summary string.
pub fn canonical_j1900_batch_parity_summary_for_report() -> String {
    match canonical_j1900_batch_parity_summary() {
        Some(summary) => format_validated_canonical_j1900_batch_parity_summary_for_report(&summary),
        None => "VSOP87 canonical J1900 batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the supported-body J2000 ecliptic batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J2000 ecliptic batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j2000_ecliptic_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J2000 ecliptic batch-path regression summary.
pub fn supported_body_j2000_ecliptic_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ2000EclipticBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j2000_ecliptic_batch_parity_request_corpus();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ2000EclipticBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ2000EclipticBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J2000 ecliptic batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J2000 ecliptic batch-path regression summary string.
pub fn supported_body_j2000_ecliptic_batch_parity_summary_for_report() -> String {
    match supported_body_j2000_ecliptic_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j2000_ecliptic_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body J2000 ecliptic batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the supported-body J2000 equatorial batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J2000 equatorial batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j2000_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J2000 equatorial batch-path regression summary.
pub fn supported_body_j2000_equatorial_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ2000EquatorialBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j2000_equatorial_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ2000EquatorialBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ2000EquatorialBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J2000 equatorial batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J2000 equatorial batch-path regression summary string.
pub fn supported_body_j2000_equatorial_batch_parity_summary_for_report() -> String {
    match supported_body_j2000_equatorial_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j2000_equatorial_batch_parity_summary_for_report(
                &summary,
            )
        }
        None => "VSOP87 supported-body J2000 equatorial batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the supported-body J1900 ecliptic batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J1900 ecliptic batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Ecliptic {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j1900_ecliptic_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J1900 ecliptic batch-path regression summary.
pub fn supported_body_j1900_ecliptic_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ1900EclipticBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j1900_ecliptic_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ1900EclipticBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Ecliptic,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ1900EclipticBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J1900 ecliptic batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J1900 ecliptic batch-path regression summary string.
pub fn supported_body_j1900_ecliptic_batch_parity_summary_for_report() -> String {
    match supported_body_j1900_ecliptic_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j1900_ecliptic_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body J1900 ecliptic batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the supported-body J1900 equatorial batch-path regression.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    /// Number of requests exercised through the batch regression.
    pub sample_count: usize,
    /// Bodies exercised through the batch regression in release-facing order.
    pub sample_bodies: Vec<CelestialBody>,
    /// Reference epoch used by the batch regression.
    pub reference_epoch: Instant,
    /// Coordinate frame used by the batch regression.
    pub frame: CoordinateFrame,
    /// Number of exact-quality results observed in the batch regression.
    pub exact_count: usize,
    /// Number of interpolated-quality results observed in the batch regression.
    pub interpolated_count: usize,
    /// Number of approximate-quality results observed in the batch regression.
    pub approximate_count: usize,
    /// Number of unknown-quality results observed in the batch regression.
    pub unknown_count: usize,
}

impl Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "VSOP87 supported-body J1900 equatorial batch parity: {} requests across {} bodies ({}) at JD {:.1} ({}) in {} frame; quality counts: Exact={}, Interpolated={}, Approximate={}, Unknown={}; batch/single parity preserved",
            self.sample_count,
            self.sample_bodies.len(),
            format_celestial_bodies(&self.sample_bodies),
            self.reference_epoch.julian_day.days(),
            self.reference_epoch.scale,
            self.frame,
            self.exact_count,
            self.interpolated_count,
            self.approximate_count,
            self.unknown_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalBatchParitySummaryValidationError> {
        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != Vsop87Backend::supported_bodies().to_vec() {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.reference_epoch
            != Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb)
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "reference_epoch",
                },
            );
        }
        if self.frame != CoordinateFrame::Equatorial {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync { field: "frame" },
            );
        }
        if (self.exact_count
            + self.interpolated_count
            + self.approximate_count
            + self.unknown_count)
            != self.sample_count
        {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        }
        let backend = Vsop87Backend::new();
        let requests = supported_body_j1900_equatorial_batch_parity_requests();
        let Some((
            _,
            expected_exact_count,
            expected_interpolated_count,
            expected_approximate_count,
            expected_unknown_count,
        )) = canonical_batch_parity_counts(&backend, &requests)
        else {
            return Err(
                Vsop87CanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "quality_counts",
                },
            );
        };
        validate_batch_parity_quality_counts(
            (
                self.exact_count,
                self.interpolated_count,
                self.approximate_count,
                self.unknown_count,
            ),
            (
                expected_exact_count,
                expected_interpolated_count,
                expected_approximate_count,
                expected_unknown_count,
            ),
        )?;

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body J1900 equatorial batch-path regression summary.
pub fn supported_body_j1900_equatorial_batch_parity_summary(
) -> Option<Vsop87SupportedBodyJ1900EquatorialBatchParitySummary> {
    let backend = Vsop87Backend::new();
    let requests = supported_body_j1900_equatorial_batch_parity_requests();
    let reference_epoch = requests.first()?.instant;
    let (sample_bodies, exact_count, interpolated_count, approximate_count, unknown_count) =
        canonical_batch_parity_counts(&backend, &requests)?;

    Some(Vsop87SupportedBodyJ1900EquatorialBatchParitySummary {
        sample_count: requests.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

pub(crate) fn format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyJ1900EquatorialBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body J1900 equatorial batch parity: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body J1900 equatorial batch-path regression summary string.
pub fn supported_body_j1900_equatorial_batch_parity_summary_for_report() -> String {
    match supported_body_j1900_equatorial_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_j1900_equatorial_batch_parity_summary_for_report(
                &summary,
            )
        }
        None => "VSOP87 supported-body J1900 equatorial batch parity: unavailable".to_string(),
    }
}

/// Backend-owned summary for the supported-body canonical batch matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SupportedBodyCanonicalBatchParitySummary {
    /// Number of supported bodies exercised by each batch slice.
    pub supported_body_count: usize,
    /// Supported-body J2000 ecliptic batch regression summary.
    pub j2000_ecliptic: Vsop87SupportedBodyJ2000EclipticBatchParitySummary,
    /// Supported-body J2000 equatorial batch regression summary.
    pub j2000_equatorial: Vsop87SupportedBodyJ2000EquatorialBatchParitySummary,
    /// Supported-body J1900 ecliptic batch regression summary.
    pub j1900_ecliptic: Vsop87SupportedBodyJ1900EclipticBatchParitySummary,
    /// Supported-body J1900 equatorial batch regression summary.
    pub j1900_equatorial: Vsop87SupportedBodyJ1900EquatorialBatchParitySummary,
}

/// Structured validation errors for a supported-body canonical batch matrix summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 supported-body canonical batch matrix field `{field}` is out of sync with the current supported-body batch evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError {}

impl Vsop87SupportedBodyCanonicalBatchParitySummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let bodies = if self.j2000_ecliptic.sample_bodies.is_empty() {
            "none".to_string()
        } else {
            format_celestial_bodies(&self.j2000_ecliptic.sample_bodies)
        };

        format!(
            "VSOP87 supported-body canonical batch matrix: {} bodies ({}); slices: J2000 ecliptic {} requests, J2000 equatorial {} requests, J1900 ecliptic {} requests, J1900 equatorial {} requests; batch/single parity preserved across the supported planetary set",
            self.supported_body_count,
            bodies,
            self.j2000_ecliptic.sample_count,
            self.j2000_equatorial.sample_count,
            self.j1900_ecliptic.sample_count,
            self.j1900_equatorial.sample_count,
        )
    }

    /// Returns `Ok(())` when the summary still matches the current derived batch evidence.
    pub fn validate(
        &self,
    ) -> Result<(), Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError> {
        self.j2000_ecliptic.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j2000_ecliptic",
            }
        })?;
        self.j2000_equatorial.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j2000_equatorial",
            }
        })?;
        self.j1900_ecliptic.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j1900_ecliptic",
            }
        })?;
        self.j1900_equatorial.validate().map_err(|_| {
            Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                field: "j1900_equatorial",
            }
        })?;

        let supported_bodies = Vsop87Backend::supported_bodies();
        if self.supported_body_count != supported_bodies.len() {
            return Err(
                Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "supported_body_count",
                },
            );
        }

        let expected_bodies = supported_bodies.to_vec();
        macro_rules! validate_slice {
            ($field:literal, $summary:expr) => {
                let summary = &$summary;
                if summary.sample_count != self.supported_body_count {
                    return Err(
                        Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                            field: $field,
                        },
                    );
                }
                if summary.sample_bodies != expected_bodies {
                    return Err(
                        Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                            field: $field,
                        },
                    );
                }
            };
        }
        validate_slice!("j2000_ecliptic", self.j2000_ecliptic);
        validate_slice!("j2000_equatorial", self.j2000_equatorial);
        validate_slice!("j1900_ecliptic", self.j1900_ecliptic);
        validate_slice!("j1900_equatorial", self.j1900_equatorial);

        if self.j2000_ecliptic.sample_bodies != self.j2000_equatorial.sample_bodies
            || self.j2000_ecliptic.sample_bodies != self.j1900_ecliptic.sample_bodies
            || self.j2000_ecliptic.sample_bodies != self.j1900_equatorial.sample_bodies
        {
            return Err(
                Vsop87SupportedBodyCanonicalBatchParitySummaryValidationError::FieldOutOfSync {
                    field: "body_order",
                },
            );
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87SupportedBodyCanonicalBatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned supported-body canonical batch matrix summary.
pub fn supported_body_canonical_batch_parity_summary(
) -> Option<Vsop87SupportedBodyCanonicalBatchParitySummary> {
    Some(Vsop87SupportedBodyCanonicalBatchParitySummary {
        supported_body_count: Vsop87Backend::supported_bodies().len(),
        j2000_ecliptic: supported_body_j2000_ecliptic_batch_parity_summary()?,
        j2000_equatorial: supported_body_j2000_equatorial_batch_parity_summary()?,
        j1900_ecliptic: supported_body_j1900_ecliptic_batch_parity_summary()?,
        j1900_equatorial: supported_body_j1900_equatorial_batch_parity_summary()?,
    })
}

pub(crate) fn format_validated_supported_body_canonical_batch_parity_summary_for_report(
    summary: &Vsop87SupportedBodyCanonicalBatchParitySummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("VSOP87 supported-body canonical batch matrix: unavailable ({error})")
        }
    }
}

/// Returns the release-facing supported-body canonical batch matrix summary string.
pub fn supported_body_canonical_batch_parity_summary_for_report() -> String {
    match supported_body_canonical_batch_parity_summary() {
        Some(summary) => {
            format_validated_supported_body_canonical_batch_parity_summary_for_report(&summary)
        }
        None => "VSOP87 supported-body canonical batch matrix: unavailable".to_string(),
    }
}

/// Returns the supported-body canonical batch matrix request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the supported-body order and concatenate the J2000/J1900
/// ecliptic and equatorial slices so validation and reproducibility tooling can
/// reuse the full supported-planet matrix without reconstructing it from the
/// summary metadata.
pub fn supported_body_canonical_batch_parity_requests() -> Vec<EphemerisRequest> {
    let mut requests = supported_body_j2000_ecliptic_batch_parity_requests();
    requests.extend(supported_body_j2000_equatorial_batch_parity_requests());
    requests.extend(supported_body_j1900_ecliptic_batch_parity_requests());
    requests.extend(supported_body_j1900_equatorial_batch_parity_requests());
    requests
}

/// Returns the supported-body canonical batch matrix request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`supported_body_canonical_batch_parity_requests`].
#[doc(alias = "supported_body_canonical_batch_parity_requests")]
#[doc(alias = "supported_body_canonical_batch_matrix_request_corpus")]
pub fn supported_body_canonical_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_parity_requests()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_parity_request_corpus`].
#[doc(alias = "supported_body_canonical_batch_parity_request_corpus")]
pub fn supported_body_canonical_batch_matrix_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_parity_request_corpus()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_matrix_request_corpus`].
#[doc(alias = "supported_body_canonical_batch_matrix_request_corpus")]
pub fn supported_body_canonical_batch_matrix_requests() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_matrix_request_corpus()
}

/// This is a compatibility alias for [`supported_body_canonical_batch_matrix_requests`].
#[doc(alias = "supported_body_canonical_batch_matrix_requests")]
pub fn supported_body_canonical_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_canonical_batch_matrix_requests()
}

/// Formats the canonical VSOP87 J2000 equatorial companion summary for reporting.
pub fn format_canonical_equatorial_evidence_summary(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> String {
    format!(
        "VSOP87 canonical J2000 equatorial companion evidence: {} samples, bodies: {}, mean Δra={:.12}°, median Δra={:.12}°, p95 Δra={:.12}°, rms Δra={:.12}°, mean Δdec={:.12}°, median Δdec={:.12}°, p95 Δdec={:.12}°, rms Δdec={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δra={:.12}° ({}; {}; {}), max Δdec={:.12}° ({}; {}; {}), max Δdist={:.12} AU ({}; {}; {})",
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.mean_right_ascension_delta_deg,
        summary.median_right_ascension_delta_deg,
        summary.percentile_right_ascension_delta_deg,
        summary.rms_right_ascension_delta_deg,
        summary.mean_declination_delta_deg,
        summary.median_declination_delta_deg,
        summary.percentile_declination_delta_deg,
        summary.rms_declination_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_right_ascension_delta_deg,
        summary.max_right_ascension_delta_body,
        summary.max_right_ascension_delta_source_kind,
        summary.max_right_ascension_delta_source_file,
        summary.max_declination_delta_deg,
        summary.max_declination_delta_body,
        summary.max_declination_delta_source_kind,
        summary.max_declination_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

/// Formats the current VSOP87 body-evidence envelope for reporting.
pub fn format_source_body_evidence_summary(summary: &Vsop87SourceBodyEvidenceSummary) -> String {
    let outside_note = if summary.outside_interim_limit_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.outside_interim_limit_bodies)
    };

    let bodies = format_celestial_bodies(&summary.sample_bodies);

    if summary.generated_binary_count == 0 && summary.truncated_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else if summary.generated_binary_count > 0 && summary.truncated_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.generated_binary_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else if summary.generated_binary_count == 0 {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.truncated_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    } else {
        format!(
            "VSOP87 source-backed body evidence: {} body profiles ({} vendored full-file, {} generated binary, {} truncated slice), source-backed body order: {}, {} within interim limits, {} outside interim limits; outside interim limits: {}",
            summary.sample_count,
            summary.vendored_full_file_count,
            summary.generated_binary_count,
            summary.truncated_count,
            bodies,
            summary.within_interim_limits_count,
            summary.outside_interim_limit_count,
            outside_note,
        )
    }
}

/// Returns the release-facing source-body evidence summary string.
fn format_validated_source_body_evidence_summary_for_report(
    summary: &Vsop87SourceBodyEvidenceSummary,
) -> String {
    match summary.validated_summary_line() {
        Ok(rendered) => rendered,
        Err(error) => format!("VSOP87 source-backed body evidence: unavailable ({error})"),
    }
}

/// Returns the release-facing source-body evidence summary string.
pub fn source_body_evidence_summary_for_report() -> String {
    match source_body_evidence_summary() {
        Some(summary) => format_validated_source_body_evidence_summary_for_report(&summary),
        None => "VSOP87 source-backed body evidence: unavailable".to_string(),
    }
}

/// Body classes used for source-backed VSOP87 error-envelope rollups.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vsop87SourceBodyClass {
    /// The source-backed solar body.
    Luminary,
    /// The source-backed planetary bodies.
    MajorPlanet,
}

impl Vsop87SourceBodyClass {
    const ALL: [Self; 2] = [Self::Luminary, Self::MajorPlanet];

    /// Human-readable label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Luminary => "Luminary",
            Self::MajorPlanet => "Major planets",
        }
    }
}

impl fmt::Display for Vsop87SourceBodyClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

fn source_body_class(body: &CelestialBody) -> Vsop87SourceBodyClass {
    match body {
        CelestialBody::Sun => Vsop87SourceBodyClass::Luminary,
        _ => Vsop87SourceBodyClass::MajorPlanet,
    }
}

/// Backend-owned summary for the canonical J2000 source-backed body classes.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87SourceBodyClassEvidenceSummary {
    /// Body class covered by this summary.
    pub class: Vsop87SourceBodyClass,
    /// Number of canonical samples measured for the class.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order for the class.
    pub sample_bodies: Vec<CelestialBody>,
    /// Number of samples within the interim limits.
    pub within_interim_limits_count: usize,
    /// Number of samples outside the current interim limits.
    pub outside_interim_limit_count: usize,
    /// Bodies outside the current interim limits.
    pub outside_interim_limit_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum longitude delta body.
    pub max_longitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum longitude delta body.
    pub max_longitude_delta_source_file: &'static str,
    /// Maximum absolute longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Interim longitude delta limit for the body that drives the maximum.
    pub max_longitude_delta_limit_deg: f64,
    /// Body with the maximum absolute latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum latitude delta body.
    pub max_latitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum latitude delta body.
    pub max_latitude_delta_source_file: &'static str,
    /// Maximum absolute latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Interim latitude delta limit for the body that drives the maximum.
    pub max_latitude_delta_limit_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Interim distance delta limit for the body that drives the maximum.
    pub max_distance_delta_limit_au: f64,
    /// Mean absolute longitude delta in degrees.
    pub mean_longitude_delta_deg: f64,
    /// Median absolute longitude delta in degrees.
    pub median_longitude_delta_deg: f64,
    /// 95th-percentile absolute longitude delta in degrees.
    pub percentile_longitude_delta_deg: f64,
    /// Root-mean-square longitude delta in degrees.
    pub rms_longitude_delta_deg: f64,
    /// Mean absolute latitude delta in degrees.
    pub mean_latitude_delta_deg: f64,
    /// Median absolute latitude delta in degrees.
    pub median_latitude_delta_deg: f64,
    /// 95th-percentile absolute latitude delta in degrees.
    pub percentile_latitude_delta_deg: f64,
    /// Root-mean-square latitude delta in degrees.
    pub rms_latitude_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th-percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87SourceBodyClassEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_body_class_evidence_entry(self)
    }

    /// Returns the validated compact summary line when the class evidence still matches
    /// the current canonical evidence.
    pub fn validated_summary_line(
        &self,
    ) -> Result<String, Vsop87SourceBodyClassEvidenceSummaryValidationError> {
        self.validate()?;
        Ok(self.summary_line())
    }

    /// Returns `Ok(())` when the summary still matches the current derived counts.
    pub fn validate(&self) -> Result<(), Vsop87SourceBodyClassEvidenceSummaryValidationError> {
        let Some(expected) = source_body_class_evidence_summary().and_then(|summaries| {
            summaries
                .into_iter()
                .find(|summary| summary.class == self.class)
        }) else {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        };

        if self.sample_count != self.sample_bodies.len() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_count",
                },
            );
        }
        if self.sample_bodies != expected.sample_bodies {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if self.within_interim_limits_count != expected.within_interim_limits_count {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "within_interim_limits_count",
                },
            );
        }
        if self.outside_interim_limit_count != expected.outside_interim_limit_count {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_count",
                },
            );
        }
        if self.within_interim_limits_count + self.outside_interim_limit_count != self.sample_count
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "interim_limit_counts",
                },
            );
        }
        if self.outside_interim_limit_count != self.outside_interim_limit_bodies.len() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self.outside_interim_limit_bodies != expected.outside_interim_limit_bodies {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.sample_bodies) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "sample_bodies",
                },
            );
        }
        if !body_labels_are_unique(&self.outside_interim_limit_bodies) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if self
            .outside_interim_limit_bodies
            .iter()
            .any(|body| !self.sample_bodies.contains(body))
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "outside_interim_limit_bodies",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_longitude_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_longitude_delta_body",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_latitude_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_latitude_delta_body",
                },
            );
        }
        if !self.sample_bodies.contains(&self.max_distance_delta_body) {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_distance_delta_body",
                },
            );
        }
        if self.max_longitude_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_longitude_delta_source_file",
                },
            );
        }
        if self.max_latitude_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_latitude_delta_source_file",
                },
            );
        }
        if self.max_distance_delta_source_file.trim().is_empty() {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "max_distance_delta_source_file",
                },
            );
        }
        validate_source_body_class_summary_peak_source_metadata(
            "max_longitude_delta_source_kind",
            "max_longitude_delta_source_file",
            &self.max_longitude_delta_body,
            self.max_longitude_delta_source_kind,
            self.max_longitude_delta_source_file,
        )?;
        validate_source_body_class_summary_peak_source_metadata(
            "max_latitude_delta_source_kind",
            "max_latitude_delta_source_file",
            &self.max_latitude_delta_body,
            self.max_latitude_delta_source_kind,
            self.max_latitude_delta_source_file,
        )?;
        validate_source_body_class_summary_peak_source_metadata(
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            ("max_longitude_delta_deg", self.max_longitude_delta_deg),
            (
                "max_longitude_delta_limit_deg",
                self.max_longitude_delta_limit_deg,
            ),
            ("max_latitude_delta_deg", self.max_latitude_delta_deg),
            (
                "max_latitude_delta_limit_deg",
                self.max_latitude_delta_limit_deg,
            ),
            ("max_distance_delta_au", self.max_distance_delta_au),
            (
                "max_distance_delta_limit_au",
                self.max_distance_delta_limit_au,
            ),
            ("mean_longitude_delta_deg", self.mean_longitude_delta_deg),
            (
                "median_longitude_delta_deg",
                self.median_longitude_delta_deg,
            ),
            (
                "percentile_longitude_delta_deg",
                self.percentile_longitude_delta_deg,
            ),
            ("rms_longitude_delta_deg", self.rms_longitude_delta_deg),
            ("mean_latitude_delta_deg", self.mean_latitude_delta_deg),
            ("median_latitude_delta_deg", self.median_latitude_delta_deg),
            (
                "percentile_latitude_delta_deg",
                self.percentile_latitude_delta_deg,
            ),
            ("rms_latitude_delta_deg", self.rms_latitude_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(
                    Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field },
                );
            }
        }
        if self.mean_longitude_delta_deg > self.max_longitude_delta_deg
            || self.median_longitude_delta_deg > self.percentile_longitude_delta_deg
            || self.percentile_longitude_delta_deg > self.max_longitude_delta_deg
            || self.rms_longitude_delta_deg > self.max_longitude_delta_deg
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_longitude_delta_deg",
                },
            );
        }
        if self.mean_latitude_delta_deg > self.max_latitude_delta_deg
            || self.median_latitude_delta_deg > self.percentile_latitude_delta_deg
            || self.percentile_latitude_delta_deg > self.max_latitude_delta_deg
            || self.rms_latitude_delta_deg > self.max_latitude_delta_deg
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_latitude_delta_deg",
                },
            );
        }
        if self.mean_distance_delta_au > self.max_distance_delta_au
            || self.median_distance_delta_au > self.percentile_distance_delta_au
            || self.percentile_distance_delta_au > self.max_distance_delta_au
            || self.rms_distance_delta_au > self.max_distance_delta_au
        {
            return Err(
                Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                    field: "median_distance_delta_au",
                },
            );
        }

        Ok(())
    }
}

/// Validation error for a VSOP87 source-backed body-class evidence summary that drifted
/// from the current canonical evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87SourceBodyClassEvidenceSummaryValidationError {
    /// A rendered summary field no longer matches the current derived evidence.
    FieldOutOfSync { field: &'static str },
}

impl fmt::Display for Vsop87SourceBodyClassEvidenceSummaryValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOutOfSync { field } => write!(
                f,
                "the VSOP87 source-backed body-class evidence summary field `{field}` is out of sync with the current canonical evidence"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceBodyClassEvidenceSummaryValidationError {}

fn validate_source_body_class_summary_peak_source_metadata(
    field_kind: &'static str,
    field_file: &'static str,
    body: &CelestialBody,
    source_kind: Vsop87BodySourceKind,
    source_file: &'static str,
) -> Result<(), Vsop87SourceBodyClassEvidenceSummaryValidationError> {
    let expected_source_kind = source_kind_for_body(body.clone()).ok_or(
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field: field_kind },
    )?;
    let expected_source_file = source_file_for_body(body).ok_or(
        Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync { field: field_file },
    )?;

    if expected_source_kind != source_kind {
        return Err(
            Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                field: field_kind,
            },
        );
    }
    if expected_source_file != source_file {
        return Err(
            Vsop87SourceBodyClassEvidenceSummaryValidationError::FieldOutOfSync {
                field: field_file,
            },
        );
    }

    Ok(())
}

impl fmt::Display for Vsop87SourceBodyClassEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 source-backed body-class evidence.
pub fn source_body_class_evidence_summary() -> Option<Vec<Vsop87SourceBodyClassEvidenceSummary>> {
    let evidence = canonical_epoch_body_evidence()?;
    let mut summaries = Vec::new();

    for class in Vsop87SourceBodyClass::ALL {
        let class_rows: Vec<_> = evidence
            .iter()
            .filter(|row| source_body_class(&row.body) == class)
            .collect();
        if class_rows.is_empty() {
            continue;
        }

        let sample_bodies = class_rows
            .iter()
            .map(|row| row.body.clone())
            .collect::<Vec<_>>();
        let mut longitude_values = Vec::with_capacity(class_rows.len());
        let mut latitude_values = Vec::with_capacity(class_rows.len());
        let mut distance_values = Vec::with_capacity(class_rows.len());
        let mut max_longitude_delta_body = class_rows[0].body.clone();
        let mut max_longitude_delta_source_kind = class_rows[0].source_kind;
        let mut max_longitude_delta_source_file = class_rows[0].source_file;
        let mut max_longitude_delta_deg = class_rows[0].longitude_delta_deg;
        let mut max_longitude_delta_limit_deg = class_rows[0].longitude_limit_deg;
        let mut max_latitude_delta_body = class_rows[0].body.clone();
        let mut max_latitude_delta_source_kind = class_rows[0].source_kind;
        let mut max_latitude_delta_source_file = class_rows[0].source_file;
        let mut max_latitude_delta_deg = class_rows[0].latitude_delta_deg;
        let mut max_latitude_delta_limit_deg = class_rows[0].latitude_limit_deg;
        let mut max_distance_delta_body = class_rows[0].body.clone();
        let mut max_distance_delta_source_kind = class_rows[0].source_kind;
        let mut max_distance_delta_source_file = class_rows[0].source_file;
        let mut max_distance_delta_au = class_rows[0].distance_delta_au;
        let mut max_distance_delta_limit_au = class_rows[0].distance_limit_au;
        let mut within_interim_limits_count = 0usize;
        let mut outside_interim_limit_bodies = Vec::new();

        for row in &class_rows {
            longitude_values.push(row.longitude_delta_deg);
            latitude_values.push(row.latitude_delta_deg);
            distance_values.push(row.distance_delta_au);
            if row.within_interim_limits {
                within_interim_limits_count += 1;
            } else {
                outside_interim_limit_bodies.push(row.body.clone());
            }

            if row.longitude_delta_deg > max_longitude_delta_deg {
                max_longitude_delta_body = row.body.clone();
                max_longitude_delta_source_kind = row.source_kind;
                max_longitude_delta_source_file = row.source_file;
                max_longitude_delta_deg = row.longitude_delta_deg;
                max_longitude_delta_limit_deg = row.longitude_limit_deg;
            }
            if row.latitude_delta_deg > max_latitude_delta_deg {
                max_latitude_delta_body = row.body.clone();
                max_latitude_delta_source_kind = row.source_kind;
                max_latitude_delta_source_file = row.source_file;
                max_latitude_delta_deg = row.latitude_delta_deg;
                max_latitude_delta_limit_deg = row.latitude_limit_deg;
            }
            if row.distance_delta_au > max_distance_delta_au {
                max_distance_delta_body = row.body.clone();
                max_distance_delta_source_kind = row.source_kind;
                max_distance_delta_source_file = row.source_file;
                max_distance_delta_au = row.distance_delta_au;
                max_distance_delta_limit_au = row.distance_limit_au;
            }
        }

        let sample_count = class_rows.len();
        let mut longitude_values_for_median = longitude_values.clone();
        let mut longitude_values_for_percentile = longitude_values;
        let mut latitude_values_for_median = latitude_values.clone();
        let mut latitude_values_for_percentile = latitude_values;
        let mut distance_values_for_median = distance_values.clone();
        let mut distance_values_for_percentile = distance_values;
        summaries.push(Vsop87SourceBodyClassEvidenceSummary {
            class,
            sample_count,
            sample_bodies,
            within_interim_limits_count,
            outside_interim_limit_count: sample_count - within_interim_limits_count,
            outside_interim_limit_bodies,
            max_longitude_delta_body,
            max_longitude_delta_source_kind,
            max_longitude_delta_source_file,
            max_longitude_delta_deg,
            max_longitude_delta_limit_deg,
            max_latitude_delta_body,
            max_latitude_delta_source_kind,
            max_latitude_delta_source_file,
            max_latitude_delta_deg,
            max_latitude_delta_limit_deg,
            max_distance_delta_body,
            max_distance_delta_source_kind,
            max_distance_delta_source_file,
            max_distance_delta_au,
            max_distance_delta_limit_au,
            mean_longitude_delta_deg: longitude_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_longitude_delta_deg: median_f64(&mut longitude_values_for_median),
            percentile_longitude_delta_deg: percentile_f64(
                &mut longitude_values_for_percentile,
                0.95,
            ),
            rms_longitude_delta_deg: rms_f64(&longitude_values_for_percentile),
            mean_latitude_delta_deg: latitude_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_latitude_delta_deg: median_f64(&mut latitude_values_for_median),
            percentile_latitude_delta_deg: percentile_f64(
                &mut latitude_values_for_percentile,
                0.95,
            ),
            rms_latitude_delta_deg: rms_f64(&latitude_values_for_percentile),
            mean_distance_delta_au: distance_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_distance_delta_au: median_f64(&mut distance_values_for_median),
            percentile_distance_delta_au: percentile_f64(&mut distance_values_for_percentile, 0.95),
            rms_distance_delta_au: rms_f64(&distance_values_for_percentile),
        });
    }

    Some(summaries)
}

/// Formats a single canonical VSOP87 body-class evidence envelope.
fn format_source_body_class_evidence_entry(
    summary: &Vsop87SourceBodyClassEvidenceSummary,
) -> String {
    let outside_note = if summary.outside_interim_limit_bodies.is_empty() {
        "none".to_string()
    } else {
        format_celestial_bodies(&summary.outside_interim_limit_bodies)
    };

    format!(
        "{}: samples={}, bodies: {}, within interim limits {}, outside interim limits {}; out-of-limit bodies: {}; mean Δlon={:.12}°, median Δlon={:.12}°, p95 Δlon={:.12}°, rms Δlon={:.12}°, mean Δlat={:.12}°, median Δlat={:.12}°, p95 Δlat={:.12}°, rms Δlat={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δlon={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δlat={:.12}° (limit {:.12}°, margin {:+.12}°; {}; {}; {}), max Δdist={:.12} AU (limit {:.12} AU, margin {:+.12} AU; {}; {}; {})",
        summary.class,
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.within_interim_limits_count,
        summary.outside_interim_limit_count,
        outside_note,
        summary.mean_longitude_delta_deg,
        summary.median_longitude_delta_deg,
        summary.percentile_longitude_delta_deg,
        summary.rms_longitude_delta_deg,
        summary.mean_latitude_delta_deg,
        summary.median_latitude_delta_deg,
        summary.percentile_latitude_delta_deg,
        summary.rms_latitude_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_longitude_delta_deg,
        summary.max_longitude_delta_limit_deg,
        summary.max_longitude_delta_limit_deg - summary.max_longitude_delta_deg,
        summary.max_longitude_delta_body,
        summary.max_longitude_delta_source_kind,
        summary.max_longitude_delta_source_file,
        summary.max_latitude_delta_deg,
        summary.max_latitude_delta_limit_deg,
        summary.max_latitude_delta_limit_deg - summary.max_latitude_delta_deg,
        summary.max_latitude_delta_body,
        summary.max_latitude_delta_source_kind,
        summary.max_latitude_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_limit_au,
        summary.max_distance_delta_limit_au - summary.max_distance_delta_au,
        summary.max_distance_delta_body,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

/// Formats the canonical VSOP87 body-class evidence for reporting.
pub fn format_source_body_class_evidence_summary(
    summaries: &[Vsop87SourceBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 source-backed body-class envelopes: unavailable".to_string();
    }

    let rendered = summaries
        .iter()
        .map(Vsop87SourceBodyClassEvidenceSummary::summary_line)
        .collect::<Vec<_>>()
        .join(" | ");

    format!("VSOP87 source-backed body-class envelopes: {rendered}")
}

/// Returns the release-facing source-body-class evidence summary string.
pub(crate) fn format_validated_source_body_class_evidence_summary_for_report(
    summaries: &[Vsop87SourceBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 source-backed body-class envelopes: unavailable".to_string();
    }

    let mut rendered = Vec::with_capacity(summaries.len());
    for summary in summaries {
        match summary.validated_summary_line() {
            Ok(line) => rendered.push(line),
            Err(error) => {
                return format!("VSOP87 source-backed body-class envelopes: unavailable ({error})");
            }
        }
    }

    format!(
        "VSOP87 source-backed body-class envelopes: {}",
        rendered.join(" | ")
    )
}

/// Returns the release-facing source-body-class evidence summary string.
pub fn source_body_class_evidence_summary_for_report() -> String {
    match source_body_class_evidence_summary() {
        Some(summary) => format_validated_source_body_class_evidence_summary_for_report(&summary),
        None => "VSOP87 source-backed body-class envelopes: unavailable".to_string(),
    }
}

/// Errors that can occur while regenerating a checked-in VSOP87B binary table
/// from a vendored public source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vsop87TableGenerationError {
    /// The requested file name does not match one of the vendored public source
    /// files that this crate knows how to regenerate.
    UnknownSourceFile {
        source_file: String,
        supported_source_files: Vec<&'static str>,
    },
    /// The vendored source text could be parsed, but the regeneration step
    /// failed while rebuilding the binary coefficient table.
    Parse { source_file: String, error: String },
}

impl core::fmt::Display for Vsop87TableGenerationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownSourceFile {
                source_file,
                supported_source_files,
            } => {
                write!(
                    f,
                    "no vendored VSOP87B source text found for {source_file}; supported source files: {}",
                    supported_source_files.join(", ")
                )
            }
            Self::Parse { source_file, error } => {
                write!(
                    f,
                    "failed to regenerate VSOP87B table for {source_file}: {error}"
                )
            }
        }
    }
}

impl std::error::Error for Vsop87TableGenerationError {}

/// Returns the source/body manifest in source-spec order.
///
/// This list is primarily used by maintainer-facing regeneration tooling and
/// reproducibility checks so downstream code can discover the expected public
/// input files and their bodies without hardcoding the table-specific match block.
pub fn source_manifest() -> Vec<(CelestialBody, &'static str)> {
    source_specifications()
        .into_iter()
        .map(|spec| (spec.body, spec.source_file))
        .collect()
}

/// Borrowed summary of a VSOP87 source manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vsop87SourceManifestSummary<'a> {
    /// Source/body pairs in source-spec order.
    pub manifest: &'a [(CelestialBody, &'static str)],
}

impl Vsop87SourceManifestSummary<'_> {
    /// Returns `Ok(())` when the manifest still matches the current source catalog.
    pub fn validate(&self) -> Result<(), Vsop87SourceManifestValidationError> {
        validate_source_manifest(self.manifest)
    }

    /// Returns a compact one-line rendering of the current source manifest.
    pub fn summary_line(&self) -> String {
        let entries = self
            .manifest
            .iter()
            .map(|(body, source_file)| format!("{body} / {source_file}"))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "VSOP87 source manifest: {} entries ({entries})",
            self.manifest.len()
        )
    }
}

impl fmt::Display for Vsop87SourceManifestSummary<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns a borrowed summary for the source/body manifest in source-spec order.
pub fn source_manifest_summary<'a>(
    manifest: &'a [(CelestialBody, &'static str)],
) -> Vsop87SourceManifestSummary<'a> {
    Vsop87SourceManifestSummary { manifest }
}

/// Formats a VSOP87 source-manifest summary for release-facing reporting.
pub fn format_source_manifest_summary(summary: &Vsop87SourceManifestSummary<'_>) -> String {
    summary.summary_line()
}

/// Returns the release-facing source-manifest summary for the current source catalog.
pub fn source_manifest_summary_for_report() -> String {
    let manifest = source_manifest();
    let summary = source_manifest_summary(&manifest);

    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => format!("VSOP87 source manifest: unavailable ({error})"),
    }
}

/// Validation errors for a VSOP87 source manifest that drifted from the
/// current source-specification catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vsop87SourceManifestValidationError {
    /// The manifest length differs from the current source-specification list.
    LengthMismatch {
        expected: usize,
        actual: usize,
        expected_manifest: Vec<(CelestialBody, &'static str)>,
        actual_manifest: Vec<(CelestialBody, &'static str)>,
    },
    /// One manifest entry differs from the current source-specification catalog.
    EntryMismatch {
        index: usize,
        expected_body: Box<CelestialBody>,
        actual_body: Box<CelestialBody>,
        expected_source_file: &'static str,
        actual_source_file: &'static str,
    },
}

impl fmt::Display for Vsop87SourceManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch {
                expected,
                actual,
                expected_manifest,
                actual_manifest,
            } => write!(
                f,
                "the VSOP87 source manifest length is out of sync with the current source catalog (expected {expected} entries [{}], got {actual} entries [{}])",
                format_source_manifest_pairs(expected_manifest),
                format_source_manifest_pairs(actual_manifest)
            ),
            Self::EntryMismatch {
                index,
                expected_body,
                actual_body,
                expected_source_file,
                actual_source_file,
            } => write!(
                f,
                "the VSOP87 source manifest entry {index} is out of sync with the current source catalog (expected {expected_body} / {expected_source_file}, got {actual_body} / {actual_source_file})"
            ),
        }
    }
}

impl std::error::Error for Vsop87SourceManifestValidationError {}

/// Validates that a VSOP87 source manifest still matches the current source
/// catalog order.
pub fn validate_source_manifest(
    manifest: &[(CelestialBody, &'static str)],
) -> Result<(), Vsop87SourceManifestValidationError> {
    let expected_manifest = source_specifications();
    let expected_manifest_pairs = expected_manifest
        .iter()
        .map(|spec| (spec.body.clone(), spec.source_file))
        .collect::<Vec<_>>();

    if manifest.len() != expected_manifest.len() {
        return Err(Vsop87SourceManifestValidationError::LengthMismatch {
            expected: expected_manifest.len(),
            actual: manifest.len(),
            expected_manifest: expected_manifest_pairs,
            actual_manifest: manifest.to_vec(),
        });
    }

    for (index, ((actual_body, actual_source_file), expected_spec)) in
        manifest.iter().zip(expected_manifest.iter()).enumerate()
    {
        if *actual_body != expected_spec.body || *actual_source_file != expected_spec.source_file {
            return Err(Vsop87SourceManifestValidationError::EntryMismatch {
                index,
                expected_body: Box::new(expected_spec.body.clone()),
                actual_body: Box::new(actual_body.clone()),
                expected_source_file: expected_spec.source_file,
                actual_source_file,
            });
        }
    }

    Ok(())
}

fn format_source_manifest_pairs(manifest: &[(CelestialBody, &'static str)]) -> String {
    manifest
        .iter()
        .map(|(body, source_file)| format!("{body} / {source_file}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Returns the supported vendored VSOP87B source files in source-spec order.
///
/// This list is primarily used by maintainer-facing regeneration tooling and
/// reproducibility checks so downstream code can discover the expected public
/// input files without hardcoding the table-specific match block.
pub fn supported_source_files() -> Vec<&'static str> {
    source_manifest()
        .into_iter()
        .map(|(_, source_file)| source_file)
        .collect()
}

/// Regenerates the checked-in binary VSOP87B coefficient blob for a vendored
/// public source file.
///
/// This helper is used by the maintainer-facing regeneration tool and the
/// reproducibility tests to keep the checked-in `.bin` files aligned with the
/// vendored public IMCCE/CELMECH source inputs.
pub fn try_generated_vsop87b_table_bytes_for_source_file(
    source_file: &str,
) -> Result<Vec<u8>, Vsop87TableGenerationError> {
    let source = source_text_for_file(source_file).ok_or_else(|| {
        Vsop87TableGenerationError::UnknownSourceFile {
            source_file: source_file.to_string(),
            supported_source_files: supported_source_files(),
        }
    })?;
    generated_vsop87b_table_bytes(source).map_err(|error| Vsop87TableGenerationError::Parse {
        source_file: source_file.to_string(),
        error: error.to_string(),
    })
}

pub fn generated_vsop87b_table_bytes_for_source_file(source_file: &str) -> Option<Vec<u8>> {
    try_generated_vsop87b_table_bytes_for_source_file(source_file).ok()
}

/// Returns the checked-in generated binary blob for a supported VSOP87B source file.
///
/// This helper keeps the source-file-to-binary mapping explicit for the
/// regeneration tests and maintainer-facing tooling while the runtime path
/// continues to load the generated blobs from `include_bytes!`.
pub fn checked_in_generated_vsop87b_table_bytes_for_source_file(
    source_file: &str,
) -> Option<&'static [u8]> {
    match source_file {
        "VSOP87B.ear" => Some(include_bytes!("../data/VSOP87B.ear.bin") as &'static [u8]),
        "VSOP87B.mer" => Some(include_bytes!("../data/VSOP87B.mer.bin") as &'static [u8]),
        "VSOP87B.ven" => Some(include_bytes!("../data/VSOP87B.ven.bin") as &'static [u8]),
        "VSOP87B.mar" => Some(include_bytes!("../data/VSOP87B.mar.bin") as &'static [u8]),
        "VSOP87B.jup" => Some(include_bytes!("../data/VSOP87B.jup.bin") as &'static [u8]),
        "VSOP87B.sat" => Some(include_bytes!("../data/VSOP87B.sat.bin") as &'static [u8]),
        "VSOP87B.ura" => Some(include_bytes!("../data/VSOP87B.ura.bin") as &'static [u8]),
        "VSOP87B.nep" => Some(include_bytes!("../data/VSOP87B.nep.bin") as &'static [u8]),
        _ => None,
    }
}

/// Returns the canonical J2000 source-backed VSOP87B samples used by
/// validation reporting.
pub fn canonical_epoch_samples() -> Vec<Vsop87CanonicalEpochSample> {
    body_catalog_entries()
        .iter()
        .filter_map(|entry| entry.canonical_sample.clone())
        .collect()
}

/// Builds a batch request corpus for the provided bodies at a shared instant
/// and coordinate frame.
///
/// The requests preserve the iterator order and default to tropical mean
/// geometric geocentric output, which makes the helper suitable for canonical
/// batch regressions and reproducibility tooling.
pub fn requests_for_bodies_at<I>(
    bodies: I,
    instant: Instant,
    frame: CoordinateFrame,
) -> Vec<EphemerisRequest>
where
    I: IntoIterator<Item = CelestialBody>,
{
    bodies
        .into_iter()
        .map(|body| {
            let mut request = EphemerisRequest::new(body, instant);
            request.frame = frame;
            request
        })
        .collect()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the source-backed body order, use the shared J2000 TT
/// instant, and keep the geocentric ecliptic frame so validation and
/// reproducibility tooling can reuse the exact canonical batch slice without
/// reconstructing it from the sample metadata.
#[doc(alias = "canonical_epoch_requests")]
pub fn canonical_j2000_batch_parity_requests() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        canonical_epoch_samples()
            .into_iter()
            .map(|sample| sample.body),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        CoordinateFrame::Ecliptic,
    )
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_j2000_batch_parity_requests`].
pub fn canonical_epoch_requests() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_epoch_requests`].
#[doc(alias = "canonical_epoch_requests")]
#[doc(alias = "canonical_j2000_batch_parity_requests")]
pub fn canonical_epoch_request_corpus() -> Vec<EphemerisRequest> {
    canonical_epoch_requests()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_j2000_batch_parity_requests`].
#[doc(alias = "canonical_j2000_batch_parity_requests")]
pub fn canonical_j2000_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_j2000_batch_parity_requests`].
#[doc(alias = "canonical_j2000_batch_parity_requests")]
pub fn canonical_epoch_batch_parity_requests() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_epoch_batch_parity_requests`].
#[doc(alias = "canonical_epoch_batch_parity_requests")]
pub fn canonical_epoch_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_epoch_batch_parity_requests()
}

/// Returns the canonical J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`canonical_j2000_batch_parity_requests`].
#[doc(alias = "canonical_j2000_batch_parity_requests")]
pub fn canonical_j2000_request_corpus() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the source-backed body order, use the shared J2000 TT
/// instant, and keep the geocentric ecliptic frame so validation and
/// reproducibility tooling can reuse the exact source-backed batch slice without
/// reconstructing it from the sample metadata.
#[doc(alias = "canonical_j2000_batch_parity_requests")]
pub fn source_backed_body_j2000_batch_parity_requests() -> Vec<EphemerisRequest> {
    canonical_j2000_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`source_backed_body_j2000_batch_parity_requests`].
#[doc(alias = "source_backed_body_j2000_batch_parity_requests")]
pub fn source_backed_body_j2000_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for [`source_backed_body_j2000_batch_parity_requests`].
#[doc(alias = "source_backed_body_j2000_batch_parity_requests")]
pub fn source_backed_body_j2000_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// The requests preserve the source-backed body order, use the shared J2000 TT
/// instant, and keep the geocentric ecliptic frame so validation and
/// reproducibility tooling can reuse the exact source-backed batch slice without
/// reconstructing it from the sample metadata.
#[doc(alias = "source_backed_body_j2000_batch_parity_requests")]
pub fn source_backed_body_j2000_ecliptic_batch_parity_requests() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch-path evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j2000_ecliptic_batch_parity_requests`].
#[doc(alias = "source_backed_body_j2000_ecliptic_batch_parity_requests")]
pub fn source_backed_body_j2000_ecliptic_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_ecliptic_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j2000_ecliptic_batch_parity_request_corpus`].
#[doc(alias = "source_backed_body_j2000_ecliptic_batch_parity_request_corpus")]
pub fn source_backed_body_j2000_ecliptic_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_ecliptic_batch_parity_request_corpus()
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 batch-parity evidence.
///
/// The requests preserve the supported-body order, use the shared J2000 TDB
/// instant, and keep the mean-obliquity equatorial frame so validation and
/// reproducibility tooling can reuse the exact canonical batch slice without
/// reconstructing it from the summary metadata.
pub fn supported_body_j2000_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        CoordinateFrame::Equatorial,
    )
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 batch-parity evidence.
///
/// This is a compatibility alias for
/// [`supported_body_j2000_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j2000_equatorial_batch_parity_requests")]
pub fn supported_body_j2000_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j2000_equatorial_batch_parity_requests()
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 batch-parity evidence.
///
/// This is a compatibility alias for
/// [`supported_body_j2000_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j2000_equatorial_batch_parity_requests")]
pub fn supported_body_j2000_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j2000_equatorial_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for [`supported_body_j2000_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j2000_equatorial_batch_parity_requests")]
pub fn source_backed_body_j2000_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    supported_body_j2000_equatorial_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j2000_equatorial_batch_parity_requests`].
#[doc(alias = "source_backed_body_j2000_equatorial_batch_parity_requests")]
pub fn source_backed_body_j2000_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_equatorial_batch_parity_requests()
}

/// Returns the source-backed J2000 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j2000_equatorial_batch_parity_request_corpus`].
#[doc(alias = "source_backed_body_j2000_equatorial_batch_parity_request_corpus")]
pub fn source_backed_body_j2000_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j2000_equatorial_batch_parity_request_corpus()
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 supported-body batch evidence.
///
/// The requests preserve the supported-body order, use the shared J2000 TDB
/// instant, and keep the ecliptic frame so validation and reproducibility
/// tooling can reuse the exact supported-body batch slice without
/// reconstructing it from the summary metadata.
pub fn supported_body_j2000_ecliptic_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tdb),
        CoordinateFrame::Ecliptic,
    )
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for
/// [`supported_body_j2000_ecliptic_batch_parity_request_corpus`].
#[doc(alias = "supported_body_j2000_ecliptic_batch_parity_request_corpus")]
pub fn supported_body_j2000_ecliptic_batch_parity_requests() -> Vec<EphemerisRequest> {
    supported_body_j2000_ecliptic_batch_parity_request_corpus()
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for
/// [`supported_body_j2000_ecliptic_batch_parity_request_corpus`].
#[doc(alias = "supported_body_j2000_ecliptic_batch_parity_request_corpus")]
pub fn supported_body_j2000_ecliptic_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j2000_ecliptic_batch_parity_request_corpus()
}

/// Returns the supported-body J2000 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for [`supported_body_j2000_ecliptic_request_corpus`].
#[doc(alias = "supported_body_j2000_ecliptic_request_corpus")]
pub fn supported_body_j2000_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j2000_ecliptic_request_corpus()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 supported-body batch evidence.
///
/// The requests preserve the supported-body order, use the shared J1900 TDB
/// instant, and keep the mean-obliquity equatorial frame so validation and
/// reproducibility tooling can reuse the exact supported-body batch slice without
/// reconstructing it from the sample metadata.
pub fn supported_body_j1900_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb),
        CoordinateFrame::Equatorial,
    )
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is the explicit frame-qualified alias for
/// [`supported_body_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "canonical_j1900_batch_parity_requests")]
pub fn canonical_j1900_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    supported_body_j1900_equatorial_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`canonical_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "canonical_j1900_equatorial_batch_parity_requests")]
pub fn canonical_j1900_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_j1900_equatorial_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_equatorial_batch_parity_requests")]
pub fn supported_body_j1900_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j1900_equatorial_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_equatorial_batch_parity_requests")]
pub fn supported_body_j1900_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j1900_equatorial_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 supported-body batch evidence.
///
/// The requests preserve the supported-body order, use the shared J1900 TDB
/// instant, and keep the ecliptic frame so validation and reproducibility
/// tooling can reuse the exact supported-body batch slice without reconstructing it
/// from the sample metadata.
pub fn supported_body_j1900_ecliptic_batch_parity_requests() -> Vec<EphemerisRequest> {
    requests_for_bodies_at(
        Vsop87Backend::supported_bodies().iter().cloned(),
        Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb),
        CoordinateFrame::Ecliptic,
    )
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_ecliptic_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_ecliptic_batch_parity_requests")]
pub fn source_backed_body_j1900_ecliptic_batch_parity_requests() -> Vec<EphemerisRequest> {
    supported_body_j1900_ecliptic_batch_parity_requests()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j1900_ecliptic_batch_parity_requests`].
#[doc(alias = "source_backed_body_j1900_ecliptic_batch_parity_requests")]
pub fn source_backed_body_j1900_ecliptic_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j1900_ecliptic_batch_parity_requests()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j1900_ecliptic_batch_parity_request_corpus`].
#[doc(alias = "source_backed_body_j1900_ecliptic_batch_parity_request_corpus")]
pub fn source_backed_body_j1900_ecliptic_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j1900_ecliptic_batch_parity_request_corpus()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for [`source_backed_body_j1900_ecliptic_request_corpus`].
#[doc(alias = "source_backed_body_j1900_ecliptic_request_corpus")]
pub fn source_backed_body_j1900_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j1900_ecliptic_request_corpus()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_equatorial_batch_parity_requests")]
pub fn source_backed_body_j1900_equatorial_batch_parity_requests() -> Vec<EphemerisRequest> {
    supported_body_j1900_equatorial_batch_parity_requests()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j1900_equatorial_batch_parity_requests`].
#[doc(alias = "source_backed_body_j1900_equatorial_batch_parity_requests")]
pub fn source_backed_body_j1900_equatorial_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j1900_equatorial_batch_parity_requests()
}

/// Returns the source-backed J1900 request corpus used by the VSOP87 batch evidence.
///
/// This is a compatibility alias for
/// [`source_backed_body_j1900_equatorial_batch_parity_request_corpus`].
#[doc(alias = "source_backed_body_j1900_equatorial_batch_parity_request_corpus")]
pub fn source_backed_body_j1900_equatorial_request_corpus() -> Vec<EphemerisRequest> {
    source_backed_body_j1900_equatorial_batch_parity_request_corpus()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_ecliptic_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_ecliptic_batch_parity_requests")]
pub fn supported_body_j1900_ecliptic_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j1900_ecliptic_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_ecliptic_batch_parity_requests`].
#[doc(alias = "supported_body_j1900_ecliptic_batch_parity_requests")]
pub fn supported_body_j1900_ecliptic_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j1900_ecliptic_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 supported-body batch evidence.
///
/// This is a compatibility alias for [`supported_body_j1900_ecliptic_request_corpus`].
#[doc(alias = "supported_body_j1900_ecliptic_request_corpus")]
pub fn supported_body_j1900_request_corpus() -> Vec<EphemerisRequest> {
    supported_body_j1900_ecliptic_request_corpus()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`canonical_j1900_equatorial_batch_parity_requests`].
pub fn canonical_j1900_batch_parity_requests() -> Vec<EphemerisRequest> {
    canonical_j1900_equatorial_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`canonical_j1900_batch_parity_requests`].
#[doc(alias = "canonical_j1900_batch_parity_requests")]
pub fn canonical_j1900_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_j1900_batch_parity_requests()
}

/// Returns the supported-body J1900 request corpus used by the VSOP87 canonical batch evidence.
///
/// This is a compatibility alias for [`canonical_j1900_batch_parity_requests`].
#[doc(alias = "canonical_j1900_batch_parity_requests")]
pub fn canonical_j1900_request_corpus() -> Vec<EphemerisRequest> {
    canonical_j1900_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// The requests preserve the canonical source-backed body order and the shared
/// J2000 ecliptic frame while alternating TT and TDB labels per request.
pub fn canonical_mixed_time_scale_batch_parity_requests() -> Vec<EphemerisRequest> {
    let mut requests = canonical_j2000_batch_parity_requests();
    for (index, request) in requests.iter_mut().enumerate() {
        request.instant.scale = if index % 2 == 0 {
            TimeScale::Tt
        } else {
            TimeScale::Tdb
        };
    }
    requests
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// This is a compatibility alias for [`canonical_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "canonical_mixed_time_scale_batch_parity_requests")]
pub fn canonical_mixed_time_scale_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_mixed_time_scale_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// This is a compatibility alias for [`canonical_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "canonical_mixed_time_scale_batch_parity_requests")]
pub fn canonical_mixed_tt_tdb_batch_parity_requests() -> Vec<EphemerisRequest> {
    canonical_mixed_time_scale_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// This is a compatibility alias for [`canonical_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "canonical_mixed_time_scale_batch_parity_requests")]
pub fn canonical_mixed_tt_tdb_batch_parity_request_corpus() -> Vec<EphemerisRequest> {
    canonical_mixed_time_scale_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// This is a compatibility alias for [`canonical_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "canonical_mixed_time_scale_batch_parity_requests")]
pub fn canonical_mixed_time_scale_request_corpus() -> Vec<EphemerisRequest> {
    canonical_mixed_time_scale_batch_parity_requests()
}

/// Returns the canonical mixed TT/TDB request corpus used by the batch-parity evidence.
///
/// This is a compatibility alias for [`canonical_mixed_time_scale_batch_parity_requests`].
#[doc(alias = "canonical_mixed_time_scale_batch_parity_requests")]
pub fn canonical_mixed_tt_tdb_request_corpus() -> Vec<EphemerisRequest> {
    canonical_mixed_time_scale_batch_parity_requests()
}

/// Returns the canonical per-body error envelope used by release-facing
/// validation reports.
///
/// The evidence is derived from one batch query over the canonical source-backed
/// sample set so the validation layer exercises the backend batch path as well
/// as the single-body query path.
pub fn canonical_epoch_body_evidence() -> Option<Vec<Vsop87CanonicalBodyEvidence>> {
    let backend = Vsop87Backend::new();
    let profiles = body_source_profiles();
    let specs = source_specifications();
    let samples = canonical_epoch_samples();
    let requests = canonical_epoch_requests();
    let results = backend.positions(&requests).ok()?;

    if results.len() != samples.len() {
        return None;
    }

    let mut evidence = Vec::with_capacity(samples.len());

    for (sample, result) in samples.into_iter().zip(results) {
        if result.body != sample.body {
            return None;
        }

        let profile = profiles
            .iter()
            .find(|profile| profile.body == sample.body)?;
        profile.validate().ok()?;
        let spec = specs.iter().find(|spec| spec.body == sample.body)?;
        let ecliptic = result.ecliptic?;
        let distance = ecliptic.distance_au?;

        let longitude_delta = signed_longitude_delta_degrees(
            sample.expected_longitude_deg,
            ecliptic.longitude.degrees(),
        )
        .abs();
        let latitude_delta = (ecliptic.latitude.degrees() - sample.expected_latitude_deg).abs();
        let distance_delta = (distance - sample.expected_distance_au).abs();
        let within_interim_limits = longitude_delta <= sample.max_longitude_delta_deg
            && latitude_delta <= sample.max_latitude_delta_deg
            && distance_delta <= sample.max_distance_delta_au;

        let row = Vsop87CanonicalBodyEvidence {
            body: sample.body,
            source_kind: profile.kind,
            source_file: spec.source_file,
            provenance: profile.provenance,
            longitude_delta_deg: longitude_delta,
            latitude_delta_deg: latitude_delta,
            distance_delta_au: distance_delta,
            longitude_limit_deg: sample.max_longitude_delta_deg,
            latitude_limit_deg: sample.max_latitude_delta_deg,
            distance_limit_au: sample.max_distance_delta_au,
            within_interim_limits,
        };
        row.validate().ok()?;
        evidence.push(row);
    }

    Some(evidence)
}

fn derive_canonical_epoch_evidence_summary(
    body_evidence: &[Vsop87CanonicalBodyEvidence],
) -> Option<Vsop87CanonicalEvidenceSummary> {
    let sample_bodies = body_evidence
        .iter()
        .map(|evidence| evidence.body.clone())
        .collect::<Vec<_>>();
    let first = body_evidence.first()?;
    let mut sample_count = 0usize;
    let mut max_longitude_delta_body = first.body.clone();
    let mut max_longitude_delta_source_kind = first.source_kind;
    let mut max_longitude_delta_source_file = first.source_file;
    let mut max_longitude_delta_deg = first.longitude_delta_deg;
    let mut max_longitude_delta_limit_deg = first.longitude_limit_deg;
    let mut max_latitude_delta_body = first.body.clone();
    let mut max_latitude_delta_source_kind = first.source_kind;
    let mut max_latitude_delta_source_file = first.source_file;
    let mut max_latitude_delta_deg = first.latitude_delta_deg;
    let mut max_latitude_delta_limit_deg = first.latitude_limit_deg;
    let mut max_distance_delta_body = first.body.clone();
    let mut max_distance_delta_source_kind = first.source_kind;
    let mut max_distance_delta_source_file = first.source_file;
    let mut max_distance_delta_au = first.distance_delta_au;
    let mut max_distance_delta_limit_au = first.distance_limit_au;
    let mut total_longitude_delta_deg = 0.0;
    let mut total_latitude_delta_deg = 0.0;
    let mut total_distance_delta_au = 0.0;
    let mut longitude_values = Vec::with_capacity(body_evidence.len());
    let mut latitude_values = Vec::with_capacity(body_evidence.len());
    let mut distance_values = Vec::with_capacity(body_evidence.len());
    let mut out_of_limit_count = 0usize;
    let mut within_interim_limits = true;

    for evidence in body_evidence {
        sample_count += 1;
        total_longitude_delta_deg += evidence.longitude_delta_deg;
        total_latitude_delta_deg += evidence.latitude_delta_deg;
        total_distance_delta_au += evidence.distance_delta_au;
        longitude_values.push(evidence.longitude_delta_deg);
        latitude_values.push(evidence.latitude_delta_deg);
        distance_values.push(evidence.distance_delta_au);
        if !evidence.within_interim_limits {
            out_of_limit_count += 1;
        }
        if evidence.longitude_delta_deg >= max_longitude_delta_deg {
            max_longitude_delta_deg = evidence.longitude_delta_deg;
            max_longitude_delta_body = evidence.body.clone();
            max_longitude_delta_source_kind = evidence.source_kind;
            max_longitude_delta_source_file = evidence.source_file;
            max_longitude_delta_limit_deg = evidence.longitude_limit_deg;
        }
        if evidence.latitude_delta_deg >= max_latitude_delta_deg {
            max_latitude_delta_deg = evidence.latitude_delta_deg;
            max_latitude_delta_body = evidence.body.clone();
            max_latitude_delta_source_kind = evidence.source_kind;
            max_latitude_delta_source_file = evidence.source_file;
            max_latitude_delta_limit_deg = evidence.latitude_limit_deg;
        }
        if evidence.distance_delta_au >= max_distance_delta_au {
            max_distance_delta_au = evidence.distance_delta_au;
            max_distance_delta_body = evidence.body.clone();
            max_distance_delta_source_kind = evidence.source_kind;
            max_distance_delta_source_file = evidence.source_file;
            max_distance_delta_limit_au = evidence.distance_limit_au;
        }
        within_interim_limits &= evidence.within_interim_limits;
    }

    Some(Vsop87CanonicalEvidenceSummary {
        sample_count,
        sample_bodies,
        max_longitude_delta_body,
        max_longitude_delta_source_kind,
        max_longitude_delta_source_file,
        max_longitude_delta_deg,
        max_longitude_delta_limit_deg,
        max_latitude_delta_body,
        max_latitude_delta_source_kind,
        max_latitude_delta_source_file,
        max_latitude_delta_deg,
        max_latitude_delta_limit_deg,
        max_distance_delta_body,
        max_distance_delta_source_kind,
        max_distance_delta_source_file,
        max_distance_delta_au,
        max_distance_delta_limit_au,
        mean_longitude_delta_deg: total_longitude_delta_deg / sample_count as f64,
        median_longitude_delta_deg: median_f64(&mut longitude_values),
        percentile_longitude_delta_deg: percentile_f64(&mut longitude_values, 0.95),
        rms_longitude_delta_deg: rms_f64(&longitude_values),
        mean_latitude_delta_deg: total_latitude_delta_deg / sample_count as f64,
        median_latitude_delta_deg: median_f64(&mut latitude_values),
        percentile_latitude_delta_deg: percentile_f64(&mut latitude_values, 0.95),
        rms_latitude_delta_deg: rms_f64(&latitude_values),
        mean_distance_delta_au: total_distance_delta_au / sample_count as f64,
        median_distance_delta_au: median_f64(&mut distance_values),
        percentile_distance_delta_au: percentile_f64(&mut distance_values, 0.95),
        rms_distance_delta_au: rms_f64(&distance_values),
        out_of_limit_count,
        within_interim_limits,
    })
}

fn validate_canonical_epoch_evidence_summary_against_current_evidence(
    summary: &Vsop87CanonicalEvidenceSummary,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let body_evidence = canonical_epoch_body_evidence().ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;
    let expected = derive_canonical_epoch_evidence_summary(&body_evidence).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;

    macro_rules! check_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Err(
                    Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                        summary: CANONICAL_EVIDENCE_SUMMARY_LABEL,
                        field: $field,
                    },
                );
            }
        };
    }

    check_field!("sample_count", summary.sample_count, expected.sample_count);
    check_field!(
        "sample_bodies",
        summary.sample_bodies,
        expected.sample_bodies
    );
    check_field!(
        "max_longitude_delta_body",
        summary.max_longitude_delta_body,
        expected.max_longitude_delta_body
    );
    check_field!(
        "max_longitude_delta_source_kind",
        summary.max_longitude_delta_source_kind,
        expected.max_longitude_delta_source_kind
    );
    check_field!(
        "max_longitude_delta_source_file",
        summary.max_longitude_delta_source_file,
        expected.max_longitude_delta_source_file
    );
    check_field!(
        "max_longitude_delta_deg",
        summary.max_longitude_delta_deg,
        expected.max_longitude_delta_deg
    );
    check_field!(
        "max_longitude_delta_limit_deg",
        summary.max_longitude_delta_limit_deg,
        expected.max_longitude_delta_limit_deg
    );
    check_field!(
        "max_latitude_delta_body",
        summary.max_latitude_delta_body,
        expected.max_latitude_delta_body
    );
    check_field!(
        "max_latitude_delta_source_kind",
        summary.max_latitude_delta_source_kind,
        expected.max_latitude_delta_source_kind
    );
    check_field!(
        "max_latitude_delta_source_file",
        summary.max_latitude_delta_source_file,
        expected.max_latitude_delta_source_file
    );
    check_field!(
        "max_latitude_delta_deg",
        summary.max_latitude_delta_deg,
        expected.max_latitude_delta_deg
    );
    check_field!(
        "max_latitude_delta_limit_deg",
        summary.max_latitude_delta_limit_deg,
        expected.max_latitude_delta_limit_deg
    );
    check_field!(
        "max_distance_delta_body",
        summary.max_distance_delta_body,
        expected.max_distance_delta_body
    );
    check_field!(
        "max_distance_delta_source_kind",
        summary.max_distance_delta_source_kind,
        expected.max_distance_delta_source_kind
    );
    check_field!(
        "max_distance_delta_source_file",
        summary.max_distance_delta_source_file,
        expected.max_distance_delta_source_file
    );
    check_field!(
        "max_distance_delta_au",
        summary.max_distance_delta_au,
        expected.max_distance_delta_au
    );
    check_field!(
        "max_distance_delta_limit_au",
        summary.max_distance_delta_limit_au,
        expected.max_distance_delta_limit_au
    );
    check_field!(
        "mean_longitude_delta_deg",
        summary.mean_longitude_delta_deg,
        expected.mean_longitude_delta_deg
    );
    check_field!(
        "median_longitude_delta_deg",
        summary.median_longitude_delta_deg,
        expected.median_longitude_delta_deg
    );
    check_field!(
        "percentile_longitude_delta_deg",
        summary.percentile_longitude_delta_deg,
        expected.percentile_longitude_delta_deg
    );
    check_field!(
        "rms_longitude_delta_deg",
        summary.rms_longitude_delta_deg,
        expected.rms_longitude_delta_deg
    );
    check_field!(
        "mean_latitude_delta_deg",
        summary.mean_latitude_delta_deg,
        expected.mean_latitude_delta_deg
    );
    check_field!(
        "median_latitude_delta_deg",
        summary.median_latitude_delta_deg,
        expected.median_latitude_delta_deg
    );
    check_field!(
        "percentile_latitude_delta_deg",
        summary.percentile_latitude_delta_deg,
        expected.percentile_latitude_delta_deg
    );
    check_field!(
        "rms_latitude_delta_deg",
        summary.rms_latitude_delta_deg,
        expected.rms_latitude_delta_deg
    );
    check_field!(
        "mean_distance_delta_au",
        summary.mean_distance_delta_au,
        expected.mean_distance_delta_au
    );
    check_field!(
        "median_distance_delta_au",
        summary.median_distance_delta_au,
        expected.median_distance_delta_au
    );
    check_field!(
        "percentile_distance_delta_au",
        summary.percentile_distance_delta_au,
        expected.percentile_distance_delta_au
    );
    check_field!(
        "rms_distance_delta_au",
        summary.rms_distance_delta_au,
        expected.rms_distance_delta_au
    );
    check_field!(
        "out_of_limit_count",
        summary.out_of_limit_count,
        expected.out_of_limit_count
    );
    check_field!(
        "within_interim_limits",
        summary.within_interim_limits,
        expected.within_interim_limits
    );

    Ok(())
}

/// Returns the canonical J2000 error envelope summary used by release-facing
/// validation reports.
pub fn canonical_epoch_evidence_summary() -> Option<Vsop87CanonicalEvidenceSummary> {
    let body_evidence = canonical_epoch_body_evidence()?;
    derive_canonical_epoch_evidence_summary(&body_evidence)
}

/// Returns the canonical J2000 equatorial companion evidence used by
/// validation reporting.
pub fn canonical_epoch_equatorial_body_evidence(
) -> Option<Vec<Vsop87CanonicalEquatorialBodyEvidence>> {
    let backend = Vsop87Backend::new();
    let profiles = body_source_profiles();
    let specs = source_specifications();
    let samples = canonical_epoch_samples();
    let requests = canonical_epoch_requests()
        .into_iter()
        .map(|mut request| {
            request.frame = CoordinateFrame::Equatorial;
            request
        })
        .collect::<Vec<_>>();
    let results = backend.positions(&requests).ok()?;
    let reference_obliquity =
        Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt).mean_obliquity();

    if results.len() != samples.len() {
        return None;
    }

    let mut evidence = Vec::with_capacity(samples.len());

    for (sample, result) in samples.into_iter().zip(results) {
        if result.body != sample.body {
            return None;
        }

        let profile = profiles
            .iter()
            .find(|profile| profile.body == sample.body)?;
        profile.validate().ok()?;
        let spec = specs.iter().find(|spec| spec.body == sample.body)?;
        let expected_ecliptic = EclipticCoordinates::new(
            Longitude::from_degrees(sample.expected_longitude_deg),
            Latitude::from_degrees(sample.expected_latitude_deg),
            Some(sample.expected_distance_au),
        );
        let expected_equatorial = expected_ecliptic.to_equatorial(reference_obliquity);
        let actual_equatorial = result.equatorial?;

        evidence.push(Vsop87CanonicalEquatorialBodyEvidence {
            body: sample.body,
            source_kind: profile.kind,
            source_file: spec.source_file,
            provenance: profile.provenance,
            right_ascension_delta_deg: signed_longitude_delta_degrees(
                expected_equatorial.right_ascension.degrees(),
                actual_equatorial.right_ascension.degrees(),
            )
            .abs(),
            declination_delta_deg: (actual_equatorial.declination.degrees()
                - expected_equatorial.declination.degrees())
            .abs(),
            distance_delta_au: (actual_equatorial.distance_au? - expected_equatorial.distance_au?)
                .abs(),
        });
    }

    Some(evidence)
}

fn derive_canonical_epoch_equatorial_evidence_summary(
    body_evidence: &[Vsop87CanonicalEquatorialBodyEvidence],
) -> Option<Vsop87CanonicalEquatorialEvidenceSummary> {
    let sample_bodies = body_evidence
        .iter()
        .map(|evidence| evidence.body.clone())
        .collect::<Vec<_>>();
    let first = body_evidence.first()?;
    let mut sample_count = 0usize;
    let mut max_right_ascension_delta_body = first.body.clone();
    let mut max_right_ascension_delta_source_kind = first.source_kind;
    let mut max_right_ascension_delta_source_file = first.source_file;
    let mut max_right_ascension_delta_deg = first.right_ascension_delta_deg;
    let mut max_declination_delta_body = first.body.clone();
    let mut max_declination_delta_source_kind = first.source_kind;
    let mut max_declination_delta_source_file = first.source_file;
    let mut max_declination_delta_deg = first.declination_delta_deg;
    let mut max_distance_delta_body = first.body.clone();
    let mut max_distance_delta_source_kind = first.source_kind;
    let mut max_distance_delta_source_file = first.source_file;
    let mut max_distance_delta_au = first.distance_delta_au;
    let mut total_right_ascension_delta_deg = 0.0;
    let mut total_declination_delta_deg = 0.0;
    let mut total_distance_delta_au = 0.0;
    let mut right_ascension_values = Vec::with_capacity(body_evidence.len());
    let mut declination_values = Vec::with_capacity(body_evidence.len());
    let mut distance_values = Vec::with_capacity(body_evidence.len());

    for evidence in body_evidence {
        sample_count += 1;
        total_right_ascension_delta_deg += evidence.right_ascension_delta_deg;
        total_declination_delta_deg += evidence.declination_delta_deg;
        total_distance_delta_au += evidence.distance_delta_au;
        right_ascension_values.push(evidence.right_ascension_delta_deg);
        declination_values.push(evidence.declination_delta_deg);
        distance_values.push(evidence.distance_delta_au);
        if evidence.right_ascension_delta_deg >= max_right_ascension_delta_deg {
            max_right_ascension_delta_deg = evidence.right_ascension_delta_deg;
            max_right_ascension_delta_body = evidence.body.clone();
            max_right_ascension_delta_source_kind = evidence.source_kind;
            max_right_ascension_delta_source_file = evidence.source_file;
        }
        if evidence.declination_delta_deg >= max_declination_delta_deg {
            max_declination_delta_deg = evidence.declination_delta_deg;
            max_declination_delta_body = evidence.body.clone();
            max_declination_delta_source_kind = evidence.source_kind;
            max_declination_delta_source_file = evidence.source_file;
        }
        if evidence.distance_delta_au >= max_distance_delta_au {
            max_distance_delta_au = evidence.distance_delta_au;
            max_distance_delta_body = evidence.body.clone();
            max_distance_delta_source_kind = evidence.source_kind;
            max_distance_delta_source_file = evidence.source_file;
        }
    }

    Some(Vsop87CanonicalEquatorialEvidenceSummary {
        sample_count,
        sample_bodies,
        max_right_ascension_delta_body,
        max_right_ascension_delta_source_kind,
        max_right_ascension_delta_source_file,
        max_right_ascension_delta_deg,
        max_declination_delta_body,
        max_declination_delta_source_kind,
        max_declination_delta_source_file,
        max_declination_delta_deg,
        max_distance_delta_body,
        max_distance_delta_source_kind,
        max_distance_delta_source_file,
        max_distance_delta_au,
        mean_right_ascension_delta_deg: total_right_ascension_delta_deg / sample_count as f64,
        median_right_ascension_delta_deg: median_f64(&mut right_ascension_values),
        percentile_right_ascension_delta_deg: percentile_f64(&mut right_ascension_values, 0.95),
        rms_right_ascension_delta_deg: rms_f64(&right_ascension_values),
        mean_declination_delta_deg: total_declination_delta_deg / sample_count as f64,
        median_declination_delta_deg: median_f64(&mut declination_values),
        percentile_declination_delta_deg: percentile_f64(&mut declination_values, 0.95),
        rms_declination_delta_deg: rms_f64(&declination_values),
        mean_distance_delta_au: total_distance_delta_au / sample_count as f64,
        median_distance_delta_au: median_f64(&mut distance_values),
        percentile_distance_delta_au: percentile_f64(&mut distance_values, 0.95),
        rms_distance_delta_au: rms_f64(&distance_values),
    })
}

fn validate_canonical_epoch_equatorial_evidence_summary_against_current_evidence(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
    let body_evidence = canonical_epoch_equatorial_body_evidence().ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;
    let expected = derive_canonical_epoch_equatorial_evidence_summary(&body_evidence).ok_or(
        Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
            summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
            field: "sample_count",
        },
    )?;

    macro_rules! check_field {
        ($field:literal, $left:expr, $right:expr) => {
            if $left != $right {
                return Err(
                    Vsop87CanonicalEvidenceSummaryValidationError::FieldOutOfSync {
                        summary: CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL,
                        field: $field,
                    },
                );
            }
        };
    }

    check_field!("sample_count", summary.sample_count, expected.sample_count);
    check_field!(
        "sample_bodies",
        summary.sample_bodies,
        expected.sample_bodies
    );
    check_field!(
        "max_right_ascension_delta_body",
        summary.max_right_ascension_delta_body,
        expected.max_right_ascension_delta_body
    );
    check_field!(
        "max_right_ascension_delta_source_kind",
        summary.max_right_ascension_delta_source_kind,
        expected.max_right_ascension_delta_source_kind
    );
    check_field!(
        "max_right_ascension_delta_source_file",
        summary.max_right_ascension_delta_source_file,
        expected.max_right_ascension_delta_source_file
    );
    check_field!(
        "max_right_ascension_delta_deg",
        summary.max_right_ascension_delta_deg,
        expected.max_right_ascension_delta_deg
    );
    check_field!(
        "max_declination_delta_body",
        summary.max_declination_delta_body,
        expected.max_declination_delta_body
    );
    check_field!(
        "max_declination_delta_source_kind",
        summary.max_declination_delta_source_kind,
        expected.max_declination_delta_source_kind
    );
    check_field!(
        "max_declination_delta_source_file",
        summary.max_declination_delta_source_file,
        expected.max_declination_delta_source_file
    );
    check_field!(
        "max_declination_delta_deg",
        summary.max_declination_delta_deg,
        expected.max_declination_delta_deg
    );
    check_field!(
        "max_distance_delta_body",
        summary.max_distance_delta_body,
        expected.max_distance_delta_body
    );
    check_field!(
        "max_distance_delta_source_kind",
        summary.max_distance_delta_source_kind,
        expected.max_distance_delta_source_kind
    );
    check_field!(
        "max_distance_delta_source_file",
        summary.max_distance_delta_source_file,
        expected.max_distance_delta_source_file
    );
    check_field!(
        "max_distance_delta_au",
        summary.max_distance_delta_au,
        expected.max_distance_delta_au
    );
    check_field!(
        "mean_right_ascension_delta_deg",
        summary.mean_right_ascension_delta_deg,
        expected.mean_right_ascension_delta_deg
    );
    check_field!(
        "median_right_ascension_delta_deg",
        summary.median_right_ascension_delta_deg,
        expected.median_right_ascension_delta_deg
    );
    check_field!(
        "percentile_right_ascension_delta_deg",
        summary.percentile_right_ascension_delta_deg,
        expected.percentile_right_ascension_delta_deg
    );
    check_field!(
        "rms_right_ascension_delta_deg",
        summary.rms_right_ascension_delta_deg,
        expected.rms_right_ascension_delta_deg
    );
    check_field!(
        "mean_declination_delta_deg",
        summary.mean_declination_delta_deg,
        expected.mean_declination_delta_deg
    );
    check_field!(
        "median_declination_delta_deg",
        summary.median_declination_delta_deg,
        expected.median_declination_delta_deg
    );
    check_field!(
        "percentile_declination_delta_deg",
        summary.percentile_declination_delta_deg,
        expected.percentile_declination_delta_deg
    );
    check_field!(
        "rms_declination_delta_deg",
        summary.rms_declination_delta_deg,
        expected.rms_declination_delta_deg
    );
    check_field!(
        "mean_distance_delta_au",
        summary.mean_distance_delta_au,
        expected.mean_distance_delta_au
    );
    check_field!(
        "median_distance_delta_au",
        summary.median_distance_delta_au,
        expected.median_distance_delta_au
    );
    check_field!(
        "percentile_distance_delta_au",
        summary.percentile_distance_delta_au,
        expected.percentile_distance_delta_au
    );
    check_field!(
        "rms_distance_delta_au",
        summary.rms_distance_delta_au,
        expected.rms_distance_delta_au
    );

    Ok(())
}

/// Returns the canonical J2000 equatorial companion evidence summary used by
/// release-facing validation reports.
pub fn canonical_epoch_equatorial_evidence_summary(
) -> Option<Vsop87CanonicalEquatorialEvidenceSummary> {
    let body_evidence = canonical_epoch_equatorial_body_evidence()?;
    derive_canonical_epoch_equatorial_evidence_summary(&body_evidence)
}

fn format_validated_canonical_epoch_equatorial_evidence_summary_for_report(
    summary: &Vsop87CanonicalEquatorialEvidenceSummary,
) -> String {
    match summary.validate() {
        Ok(()) => summary.summary_line(),
        Err(error) => {
            format!("{CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL}: unavailable ({error})")
        }
    }
}

/// Returns the release-facing canonical VSOP87 equatorial companion evidence
/// summary string.
pub fn canonical_epoch_equatorial_evidence_summary_for_report() -> String {
    match canonical_epoch_equatorial_evidence_summary() {
        Some(summary) => {
            format_validated_canonical_epoch_equatorial_evidence_summary_for_report(&summary)
        }
        None => format!("{CANONICAL_EQUATORIAL_EVIDENCE_SUMMARY_LABEL}: unavailable"),
    }
}

/// Backend-owned summary for the canonical J2000 equatorial body classes.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    /// Body class covered by this summary.
    pub class: Vsop87SourceBodyClass,
    /// Number of canonical samples measured for the class.
    pub sample_count: usize,
    /// Canonical bodies measured in source-backed order for the class.
    pub sample_bodies: Vec<CelestialBody>,
    /// Body with the maximum absolute right ascension delta.
    pub max_right_ascension_delta_body: CelestialBody,
    /// Calculation family behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum right ascension delta body.
    pub max_right_ascension_delta_source_file: &'static str,
    /// Maximum absolute right ascension delta in degrees.
    pub max_right_ascension_delta_deg: f64,
    /// Body with the maximum absolute declination delta.
    pub max_declination_delta_body: CelestialBody,
    /// Calculation family behind the maximum declination delta body.
    pub max_declination_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum declination delta body.
    pub max_declination_delta_source_file: &'static str,
    /// Maximum absolute declination delta in degrees.
    pub max_declination_delta_deg: f64,
    /// Body with the maximum absolute distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Mean absolute right ascension delta in degrees.
    pub mean_right_ascension_delta_deg: f64,
    /// Median absolute right ascension delta in degrees.
    pub median_right_ascension_delta_deg: f64,
    /// 95th percentile absolute right ascension delta in degrees.
    pub percentile_right_ascension_delta_deg: f64,
    /// Root-mean-square right ascension delta in degrees.
    pub rms_right_ascension_delta_deg: f64,
    /// Mean absolute declination delta in degrees.
    pub mean_declination_delta_deg: f64,
    /// Median absolute declination delta in degrees.
    pub median_declination_delta_deg: f64,
    /// 95th percentile absolute declination delta in degrees.
    pub percentile_declination_delta_deg: f64,
    /// Root-mean-square declination delta in degrees.
    pub rms_declination_delta_deg: f64,
    /// Mean absolute distance delta in astronomical units.
    pub mean_distance_delta_au: f64,
    /// Median absolute distance delta in astronomical units.
    pub median_distance_delta_au: f64,
    /// 95th percentile absolute distance delta in astronomical units.
    pub percentile_distance_delta_au: f64,
    /// Root-mean-square distance delta in astronomical units.
    pub rms_distance_delta_au: f64,
}

impl Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_canonical_equatorial_body_class_evidence_entry(self)
    }

    /// Returns `Ok(())` when the summary still matches the current derived evidence.
    pub fn validate(&self) -> Result<(), Vsop87CanonicalEvidenceSummaryValidationError> {
        validate_canonical_evidence_summary_bodies(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            self.sample_count,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_body",
            &self.max_right_ascension_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_body",
            &self.max_declination_delta_body,
            &self.sample_bodies,
        )?;
        validate_canonical_evidence_summary_peak_body(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_body",
            &self.max_distance_delta_body,
            &self.sample_bodies,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_file",
            self.max_right_ascension_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_file",
            self.max_declination_delta_source_file,
        )?;
        validate_non_empty_source_file(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_file",
            self.max_distance_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_right_ascension_delta_source_kind",
            "max_right_ascension_delta_source_file",
            &self.max_right_ascension_delta_body,
            self.max_right_ascension_delta_source_kind,
            self.max_right_ascension_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_declination_delta_source_kind",
            "max_declination_delta_source_file",
            &self.max_declination_delta_body,
            self.max_declination_delta_source_kind,
            self.max_declination_delta_source_file,
        )?;
        validate_canonical_evidence_summary_peak_source_metadata(
            CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
            "max_distance_delta_source_kind",
            "max_distance_delta_source_file",
            &self.max_distance_delta_body,
            self.max_distance_delta_source_kind,
            self.max_distance_delta_source_file,
        )?;
        for (field, value) in [
            (
                "max_right_ascension_delta_deg",
                self.max_right_ascension_delta_deg,
            ),
            ("max_declination_delta_deg", self.max_declination_delta_deg),
            ("max_distance_delta_au", self.max_distance_delta_au),
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
            (
                "rms_right_ascension_delta_deg",
                self.rms_right_ascension_delta_deg,
            ),
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
            ("rms_declination_delta_deg", self.rms_declination_delta_deg),
            ("mean_distance_delta_au", self.mean_distance_delta_au),
            ("median_distance_delta_au", self.median_distance_delta_au),
            (
                "percentile_distance_delta_au",
                self.percentile_distance_delta_au,
            ),
            ("rms_distance_delta_au", self.rms_distance_delta_au),
        ] {
            validate_finite_non_negative_measure(
                CANONICAL_EQUATORIAL_BODY_CLASS_EVIDENCE_SUMMARY_LABEL,
                field,
                value,
            )?;
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J2000 equatorial body-class evidence.
pub fn canonical_epoch_equatorial_body_class_evidence_summary(
) -> Option<Vec<Vsop87CanonicalEquatorialBodyClassEvidenceSummary>> {
    let evidence = canonical_epoch_equatorial_body_evidence()?;
    let mut summaries = Vec::new();

    for class in Vsop87SourceBodyClass::ALL {
        let class_rows: Vec<_> = evidence
            .iter()
            .filter(|row| source_body_class(&row.body) == class)
            .collect();
        if class_rows.is_empty() {
            continue;
        }

        let sample_bodies = class_rows
            .iter()
            .map(|row| row.body.clone())
            .collect::<Vec<_>>();
        let mut right_ascension_values = Vec::with_capacity(class_rows.len());
        let mut declination_values = Vec::with_capacity(class_rows.len());
        let mut distance_values = Vec::with_capacity(class_rows.len());
        let mut max_right_ascension_delta_body = class_rows[0].body.clone();
        let mut max_right_ascension_delta_source_kind = class_rows[0].source_kind;
        let mut max_right_ascension_delta_source_file = class_rows[0].source_file;
        let mut max_right_ascension_delta_deg = class_rows[0].right_ascension_delta_deg;
        let mut max_declination_delta_body = class_rows[0].body.clone();
        let mut max_declination_delta_source_kind = class_rows[0].source_kind;
        let mut max_declination_delta_source_file = class_rows[0].source_file;
        let mut max_declination_delta_deg = class_rows[0].declination_delta_deg;
        let mut max_distance_delta_body = class_rows[0].body.clone();
        let mut max_distance_delta_source_kind = class_rows[0].source_kind;
        let mut max_distance_delta_source_file = class_rows[0].source_file;
        let mut max_distance_delta_au = class_rows[0].distance_delta_au;

        for row in &class_rows {
            right_ascension_values.push(row.right_ascension_delta_deg);
            declination_values.push(row.declination_delta_deg);
            distance_values.push(row.distance_delta_au);

            if row.right_ascension_delta_deg > max_right_ascension_delta_deg {
                max_right_ascension_delta_body = row.body.clone();
                max_right_ascension_delta_source_kind = row.source_kind;
                max_right_ascension_delta_source_file = row.source_file;
                max_right_ascension_delta_deg = row.right_ascension_delta_deg;
            }
            if row.declination_delta_deg > max_declination_delta_deg {
                max_declination_delta_body = row.body.clone();
                max_declination_delta_source_kind = row.source_kind;
                max_declination_delta_source_file = row.source_file;
                max_declination_delta_deg = row.declination_delta_deg;
            }
            if row.distance_delta_au > max_distance_delta_au {
                max_distance_delta_body = row.body.clone();
                max_distance_delta_source_kind = row.source_kind;
                max_distance_delta_source_file = row.source_file;
                max_distance_delta_au = row.distance_delta_au;
            }
        }

        let sample_count = class_rows.len();
        let mut right_ascension_values_for_median = right_ascension_values.clone();
        let mut right_ascension_values_for_percentile = right_ascension_values;
        let mut declination_values_for_median = declination_values.clone();
        let mut declination_values_for_percentile = declination_values;
        let mut distance_values_for_median = distance_values.clone();
        let mut distance_values_for_percentile = distance_values;
        summaries.push(Vsop87CanonicalEquatorialBodyClassEvidenceSummary {
            class,
            sample_count,
            sample_bodies,
            max_right_ascension_delta_body,
            max_right_ascension_delta_source_kind,
            max_right_ascension_delta_source_file,
            max_right_ascension_delta_deg,
            max_declination_delta_body,
            max_declination_delta_source_kind,
            max_declination_delta_source_file,
            max_declination_delta_deg,
            max_distance_delta_body,
            max_distance_delta_source_kind,
            max_distance_delta_source_file,
            max_distance_delta_au,
            mean_right_ascension_delta_deg: right_ascension_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_right_ascension_delta_deg: median_f64(&mut right_ascension_values_for_median),
            percentile_right_ascension_delta_deg: percentile_f64(
                &mut right_ascension_values_for_percentile,
                0.95,
            ),
            rms_right_ascension_delta_deg: rms_f64(&right_ascension_values_for_percentile),
            mean_declination_delta_deg: declination_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_declination_delta_deg: median_f64(&mut declination_values_for_median),
            percentile_declination_delta_deg: percentile_f64(
                &mut declination_values_for_percentile,
                0.95,
            ),
            rms_declination_delta_deg: rms_f64(&declination_values_for_percentile),
            mean_distance_delta_au: distance_values_for_median.iter().sum::<f64>()
                / sample_count as f64,
            median_distance_delta_au: median_f64(&mut distance_values_for_median),
            percentile_distance_delta_au: percentile_f64(&mut distance_values_for_percentile, 0.95),
            rms_distance_delta_au: rms_f64(&distance_values_for_percentile),
        });
    }

    Some(summaries)
}

/// Formats a single canonical VSOP87 equatorial body-class evidence envelope.
fn format_canonical_equatorial_body_class_evidence_entry(
    summary: &Vsop87CanonicalEquatorialBodyClassEvidenceSummary,
) -> String {
    format!(
        "{}: samples={}, bodies: {}, mean Δra={:.12}°, median Δra={:.12}°, p95 Δra={:.12}°, rms Δra={:.12}°, mean Δdec={:.12}°, median Δdec={:.12}°, p95 Δdec={:.12}°, rms Δdec={:.12}°, mean Δdist={:.12} AU, median Δdist={:.12} AU, p95 Δdist={:.12} AU, rms Δdist={:.12} AU, max Δra={:.12}° ({}; {}), max Δdec={:.12}° ({}; {}), max Δdist={:.12} AU ({}; {})",
        summary.class,
        summary.sample_count,
        format_celestial_bodies(&summary.sample_bodies),
        summary.mean_right_ascension_delta_deg,
        summary.median_right_ascension_delta_deg,
        summary.percentile_right_ascension_delta_deg,
        summary.rms_right_ascension_delta_deg,
        summary.mean_declination_delta_deg,
        summary.median_declination_delta_deg,
        summary.percentile_declination_delta_deg,
        summary.rms_declination_delta_deg,
        summary.mean_distance_delta_au,
        summary.median_distance_delta_au,
        summary.percentile_distance_delta_au,
        summary.rms_distance_delta_au,
        summary.max_right_ascension_delta_deg,
        summary.max_right_ascension_delta_source_kind,
        summary.max_right_ascension_delta_source_file,
        summary.max_declination_delta_deg,
        summary.max_declination_delta_source_kind,
        summary.max_declination_delta_source_file,
        summary.max_distance_delta_au,
        summary.max_distance_delta_source_kind,
        summary.max_distance_delta_source_file,
    )
}

/// Formats the canonical VSOP87 equatorial body-class evidence for reporting.
pub fn format_canonical_equatorial_body_class_evidence_summary(
    summaries: &[Vsop87CanonicalEquatorialBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string();
    }

    let rendered = summaries
        .iter()
        .map(format_canonical_equatorial_body_class_evidence_entry)
        .collect::<Vec<_>>()
        .join(" | ");

    format!("VSOP87 canonical J2000 equatorial body-class envelopes: {rendered}")
}

/// Returns the release-facing equatorial body-class evidence summary string.
fn format_validated_canonical_equatorial_body_class_evidence_summary_for_report(
    summaries: &[Vsop87CanonicalEquatorialBodyClassEvidenceSummary],
) -> String {
    if summaries.is_empty() {
        return "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string();
    }

    if let Some(error) = summaries
        .iter()
        .find_map(|summary| summary.validate().err())
    {
        return format!(
            "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable ({error})"
        );
    }

    format_canonical_equatorial_body_class_evidence_summary(summaries)
}

/// Returns the release-facing equatorial body-class evidence summary string.
pub fn canonical_epoch_equatorial_body_class_evidence_summary_for_report() -> String {
    match canonical_epoch_equatorial_body_class_evidence_summary() {
        Some(summary) => {
            format_validated_canonical_equatorial_body_class_evidence_summary_for_report(&summary)
        }
        None => "VSOP87 canonical J2000 equatorial body-class envelopes: unavailable".to_string(),
    }
}

fn median_f64(values: &mut [f64]) -> f64 {
    values.sort_by(|left, right| left.total_cmp(right));
    let mid = values.len() / 2;
    if values.len().is_multiple_of(2) {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn percentile_f64(values: &mut [f64], percentile: f64) -> f64 {
    values.sort_by(|left, right| left.total_cmp(right));
    let percentile = percentile.clamp(0.0, 1.0);
    if values.len() == 1 {
        return values[0];
    }
    let position = percentile * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        values[lower]
    } else {
        let fraction = position - lower as f64;
        values[lower] + (values[upper] - values[lower]) * fraction
    }
}

fn rms_f64(values: &[f64]) -> f64 {
    let mean_square = values.iter().map(|value| value * value).sum::<f64>() / values.len() as f64;
    mean_square.sqrt()
}
