use pleiades_types::CelestialBody;
use std::fmt;
use std::sync::OnceLock;

use crate::profiles::{
    body_catalog_entries, count_vsop87_terms, fnv1a_64, join_display, source_backed_body_order,
    source_text_for_file,
};

use super::request_corpus::checked_in_generated_vsop87b_table_bytes_for_source_file;
use super::spec::source_specifications;

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
        /// Zero-based position of the record in the audit manifest.
        position: usize,
        /// Body named by the record with the blank source file.
        body: CelestialBody,
    },
    /// The audit record references a source file that does not exist in the current source catalog.
    UnknownSourceFile {
        /// Zero-based position of the record in the audit manifest.
        position: usize,
        /// Source-file label absent from the current source catalog.
        source_file: &'static str,
    },
    /// The audit record body/source pairing does not match the current source catalog.
    BodySourceMismatch {
        /// Zero-based position of the record in the audit manifest.
        position: usize,
        /// Body named by the drifted record.
        body: CelestialBody,
        /// Source file named by the drifted record.
        source_file: &'static str,
        /// Body the catalog associates with that source file.
        expected_body: CelestialBody,
    },
    /// A rendered audit field no longer matches the current source text.
    FieldOutOfSync {
        /// Zero-based position of the record in the audit manifest.
        position: usize,
        /// Body named by the record with the drifted field.
        body: CelestialBody,
        /// Source file named by the record with the drifted field.
        source_file: &'static str,
        /// Name of the field that drifted from the source text.
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
    FieldOutOfSync {
        /// Name of the summary field that drifted from the manifest.
        field: &'static str,
    },
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

static SOURCE_AUDITS: OnceLock<Vec<Vsop87SourceAudit>> = OnceLock::new();
static GENERATED_BINARY_AUDITS: OnceLock<Vec<Vsop87GeneratedBlobAudit>> = OnceLock::new();

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
        /// Zero-based position of the record in the blob-audit manifest.
        position: usize,
        /// Body named by the record with the blank source file.
        body: CelestialBody,
    },
    /// The audit record points at an empty generated blob.
    EmptyBlob {
        /// Zero-based position of the record in the blob-audit manifest.
        position: usize,
        /// Source file whose generated blob is empty.
        source_file: &'static str,
    },
    /// The audit record references a source file that does not exist in the current source catalog.
    UnknownSourceFile {
        /// Zero-based position of the record in the blob-audit manifest.
        position: usize,
        /// Source-file label absent from the current source catalog.
        source_file: &'static str,
    },
    /// The audit record references a checked-in blob that is missing from the current source catalog.
    MissingGeneratedBlob {
        /// Zero-based position of the record in the blob-audit manifest.
        position: usize,
        /// Body whose checked-in generated blob is missing.
        body: CelestialBody,
        /// Source file whose checked-in generated blob is missing.
        source_file: &'static str,
    },
    /// The audit record body/source pairing does not match the current source catalog.
    BodySourceMismatch {
        /// Zero-based position of the record in the blob-audit manifest.
        position: usize,
        /// Body named by the drifted record.
        body: CelestialBody,
        /// Source file named by the drifted record.
        source_file: &'static str,
        /// Body the catalog associates with that source file.
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
    FieldOutOfSync {
        /// Name of the summary field that drifted from the manifest.
        field: &'static str,
    },
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

