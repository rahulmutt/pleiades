use pleiades_types::{CelestialBody, CoordinateFrame, Instant, TimeScale};
use std::fmt;

use pleiades_backend::EphemerisRequest;

use crate::backend::Vsop87Backend;
use crate::profiles::source_text_for_file;
use crate::tables::vsop87b_earth::generated_vsop87b_table_bytes;

use super::evidence::{
    canonical_epoch_requests, canonical_j2000_batch_parity_requests, requests_for_bodies_at,
};
use super::spec::source_specifications;

const J1900: f64 = 2_415_020.0;
const J2000: f64 = 2_451_545.0;

/// Errors that can occur while regenerating a checked-in VSOP87B binary table
/// from a vendored public source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Vsop87TableGenerationError {
    /// The requested file name does not match one of the vendored public source
    /// files that this crate knows how to regenerate.
    UnknownSourceFile {
        /// File name that did not match a vendored source file.
        source_file: String,
        /// Vendored source files this crate can regenerate.
        supported_source_files: Vec<&'static str>,
    },
    /// The vendored source text could be parsed, but the regeneration step
    /// failed while rebuilding the binary coefficient table.
    Parse {
        /// Source file whose regeneration failed.
        source_file: String,
        /// Human-readable parse/regeneration error detail.
        error: String,
    },
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
        /// Number of entries the current source catalog expects.
        expected: usize,
        /// Number of entries the supplied manifest actually has.
        actual: usize,
        /// Body/source-file pairs the current catalog expects.
        expected_manifest: Vec<(CelestialBody, &'static str)>,
        /// Body/source-file pairs found in the supplied manifest.
        actual_manifest: Vec<(CelestialBody, &'static str)>,
    },
    /// One manifest entry differs from the current source-specification catalog.
    EntryMismatch {
        /// Index of the drifted manifest entry.
        index: usize,
        /// Body the catalog expects at this index.
        expected_body: Box<CelestialBody>,
        /// Body found at this index in the supplied manifest.
        actual_body: Box<CelestialBody>,
        /// Source file the catalog expects at this index.
        expected_source_file: &'static str,
        /// Source file found at this index in the supplied manifest.
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

/// Regenerates the binary VSOP87B blob for a vendored source file, returning
/// `None` instead of an error when the file is unknown or fails to regenerate.
///
/// This is the infallible convenience wrapper over
/// [`try_generated_vsop87b_table_bytes_for_source_file`] for maintainer tooling
/// that only needs the bytes-or-nothing outcome.
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
        "VSOP87B.ear" => Some(include_bytes!("../../data/VSOP87B.ear.bin") as &'static [u8]),
        "VSOP87B.mer" => Some(include_bytes!("../../data/VSOP87B.mer.bin") as &'static [u8]),
        "VSOP87B.ven" => Some(include_bytes!("../../data/VSOP87B.ven.bin") as &'static [u8]),
        "VSOP87B.mar" => Some(include_bytes!("../../data/VSOP87B.mar.bin") as &'static [u8]),
        "VSOP87B.jup" => Some(include_bytes!("../../data/VSOP87B.jup.bin") as &'static [u8]),
        "VSOP87B.sat" => Some(include_bytes!("../../data/VSOP87B.sat.bin") as &'static [u8]),
        "VSOP87B.ura" => Some(include_bytes!("../../data/VSOP87B.ura.bin") as &'static [u8]),
        "VSOP87B.nep" => Some(include_bytes!("../../data/VSOP87B.nep.bin") as &'static [u8]),
        _ => None,
    }
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
