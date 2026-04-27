//! Formula-based planetary backend boundary built around VSOP87-style series
//! evaluation, low-precision orbital elements, and geocentric coordinate
//! transforms.
//!
//! This crate now provides a working pure-Rust algorithmic backend for the Sun
//! and major planets. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
//! and Neptune paths evaluate public IMCCE VSOP87B sources (heliocentric
//! spherical variables, J2000 ecliptic/equinox) transformed to geocentric
//! chart-facing coordinates. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
//! and Neptune paths now use generated binary tables derived from their vendored
//! source files. A maintainer-facing regeneration helper and
//! `regenerate-vsop87b-tables` binary keep those checked-in blobs reproducible
//! from the public source text. The backend accepts both TT and TDB requests as
//! dynamical-time inputs and still rejects UT-based requests explicitly. It
//! exposes mean-only tropical geocentric ecliptic/equatorial results and rejects
//! topocentric observer or apparent-place requests with structured errors. The source-backed
//! and fallback body-profile helpers are public so reproducibility tooling can reuse
//! the backend-owned catalog partition directly. Pluto still uses compact
//! Keplerian orbital elements,
//! a geocentric reduction step, and central-difference motion estimates so the
//! workspace has an end-to-end tropical chart path while the remaining Pluto-
//! specific source selection is added incrementally.

#![forbid(unsafe_code)]

mod vsop87b_earth;
mod vsop87b_jupiter;
mod vsop87b_mars;
mod vsop87b_mercury;
mod vsop87b_neptune;
mod vsop87b_saturn;
mod vsop87b_uranus;
mod vsop87b_venus;

use pleiades_backend::{
    validate_observer_policy, validate_request_policy, validate_zodiac_policy, AccuracyClass,
    Apparentness, BackendCapabilities, BackendFamily, BackendId, BackendMetadata,
    BackendProvenance, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult, FrameTreatmentSummary, QualityAnnotation,
};
use pleiades_types::{
    CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant, Latitude,
    Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};
use std::sync::OnceLock;

use crate::vsop87b_earth::{generated_vsop87b_table_bytes, parse_vsop87b_tables};
use std::fmt;

const PACKAGE_NAME: &str = "pleiades-vsop87";
const BACKEND_LABEL: &str = "the VSOP87 backend";
const J1900: f64 = 2_415_020.0;
const J2000: f64 = 2_451_545.0;

/// Calculation family currently used for an individual VSOP87 backend body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vsop87BodySourceKind {
    /// Heliocentric spherical coordinates are evaluated from a checked-in VSOP87B
    /// coefficient slice.
    TruncatedVsop87b,
    /// Heliocentric spherical coordinates are evaluated directly from a
    /// vendored public IMCCE/CELMECH VSOP87B source file.
    VendoredVsop87b,
    /// Heliocentric spherical coordinates are evaluated from a generated
    /// binary table derived from a vendored public IMCCE/CELMECH VSOP87B
    /// source file.
    GeneratedBinaryVsop87b,
    /// Coordinates are produced from compact mean orbital elements while the
    /// remaining Pluto-specific source path is modeled separately.
    MeanOrbitalElements,
}

impl Vsop87BodySourceKind {
    /// Human-readable label used in release-facing summaries.
    pub const fn label(self) -> &'static str {
        match self {
            Self::TruncatedVsop87b => "truncated VSOP87B slice",
            Self::VendoredVsop87b => "vendored full-file VSOP87B",
            Self::GeneratedBinaryVsop87b => "generated binary VSOP87B",
            Self::MeanOrbitalElements => "mean orbital elements fallback",
        }
    }
}

impl fmt::Display for Vsop87BodySourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Per-body source profile for the current implementation state of
/// [`Vsop87Backend`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87BodySource {
    /// Body covered by this source profile.
    pub body: CelestialBody,
    /// Calculation family used for the heliocentric or geocentric channel.
    pub kind: Vsop87BodySourceKind,
    /// Human-readable provenance detail for this body's calculation path.
    pub provenance: &'static str,
    /// Current published accuracy class for this body path.
    pub accuracy: AccuracyClass,
}

/// Returns the per-body source profiles used by [`Vsop87Backend`].
///
/// The returned list is derived from the unified VSOP87 body catalog so the
/// source profile, source documentation, and canonical J2000 evidence stay in
/// sync as the backend continues expanding the generated-table pipeline.
pub fn body_source_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
        .map(|entry| entry.source_profile.clone())
        .collect()
}

/// Returns the source-backed VSOP87 body profiles used by [`Vsop87Backend`].
///
/// This is the public reproducibility-friendly subset of [`body_source_profiles()`]
/// that excludes the remaining mean-element Pluto fallback.
pub fn source_backed_body_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
        .filter(|entry| entry.source_profile.kind != Vsop87BodySourceKind::MeanOrbitalElements)
        .map(|entry| entry.source_profile.clone())
        .collect()
}

/// Returns the canonical body order for the source-backed VSOP87 profiles.
///
/// The returned order is the same reproducibility order used by the source
/// catalog and release-facing summaries: Sun through Neptune, excluding the
/// mean-element Pluto fallback.
pub fn source_backed_body_order() -> Vec<CelestialBody> {
    source_backed_body_profiles()
        .into_iter()
        .map(|profile| profile.body)
        .collect()
}

/// Returns the fallback VSOP87 body profiles used by [`Vsop87Backend`].
///
/// The current catalog keeps Pluto in this separate mean-element bucket until a
/// Pluto-specific source path is selected.
pub fn fallback_body_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
        .filter(|entry| entry.source_profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
        .map(|entry| entry.source_profile.clone())
        .collect()
}

/// Structured source documentation for the current VSOP87B-backed bodies.
///
/// These records make the current implementation explicit for release reports
/// and future generated-table work: the source-backed paths all use public
/// IMCCE/CELMECH VSOP87B spherical coefficients in the J2000 ecliptic/equinox
/// frame, with longitude/latitude in degrees and radius in astronomical units.
/// The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune paths
/// now use generated binary tables derived from their vendored public source
/// files. Pluto remains a mean orbital-elements fallback until a Pluto-specific
/// source path is selected.
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
}

impl fmt::Display for Vsop87SourceSpecification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
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
    /// Number of fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
    /// Unique date-range notes carried by the source specifications.
    pub date_ranges: Vec<&'static str>,
}

impl Vsop87SourceDocumentationSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_summary(self)
    }
}

impl fmt::Display for Vsop87SourceDocumentationSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Consistency check for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationHealthSummary {
    /// Whether the catalog counts line up with the internal body catalog.
    pub consistent: bool,
    /// Whether the documented source metadata stays aligned with the current
    /// VSOP87B policy for variant, frame, units, truncation, and date range.
    pub documentation_consistent: bool,
    /// Structured labels describing any catalog inconsistencies.
    pub issues: Vec<&'static str>,
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
    /// Number of fallback mean-element body profiles.
    pub fallback_profile_count: usize,
    /// Bodies that still use the fallback mean-element path.
    pub fallback_bodies: Vec<CelestialBody>,
}

impl Vsop87SourceDocumentationHealthSummary {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format_source_documentation_health_summary(self)
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
}

impl fmt::Display for Vsop87CanonicalEquatorialEvidenceSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

#[derive(Clone, Debug)]
struct Vsop87BodyCatalogEntry {
    source_profile: Vsop87BodySource,
    source_specification: Option<Vsop87SourceSpecification>,
    canonical_sample: Option<Vsop87CanonicalEpochSample>,
}

static BODY_CATALOG: OnceLock<Vec<Vsop87BodyCatalogEntry>> = OnceLock::new();
static SOURCE_AUDITS: OnceLock<Vec<Vsop87SourceAudit>> = OnceLock::new();
static GENERATED_BINARY_AUDITS: OnceLock<Vec<Vsop87GeneratedBlobAudit>> = OnceLock::new();

fn source_text_for_file(source_file: &str) -> Option<&'static str> {
    match source_file {
        "VSOP87B.ear" => Some(include_str!("../data/VSOP87B.ear")),
        "VSOP87B.mer" => Some(include_str!("../data/VSOP87B.mer")),
        "VSOP87B.ven" => Some(include_str!("../data/VSOP87B.ven")),
        "VSOP87B.mar" => Some(include_str!("../data/VSOP87B.mar")),
        "VSOP87B.jup" => Some(include_str!("../data/VSOP87B.jup")),
        "VSOP87B.sat" => Some(include_str!("../data/VSOP87B.sat")),
        "VSOP87B.ura" => Some(include_str!("../data/VSOP87B.ura")),
        "VSOP87B.nep" => Some(include_str!("../data/VSOP87B.nep")),
        _ => None,
    }
}

fn count_vsop87_terms(source: &str) -> usize {
    let tables = parse_vsop87b_tables(source).expect("known VSOP87 source file should parse");
    tables
        .longitude
        .iter()
        .chain(tables.latitude.iter())
        .chain(tables.radius.iter())
        .map(Vec::len)
        .sum()
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_coordinate_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

fn format_time_scales(time_scales: &[TimeScale]) -> String {
    join_display(time_scales)
}

fn format_zodiac_modes(zodiac_modes: &[ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

fn body_catalog_entries() -> &'static [Vsop87BodyCatalogEntry] {
    BODY_CATALOG.get_or_init(|| {
        let earth_date_range = "full public source file; J2000 canonical reference sample";
        let vendored_truncation_policy = "vendored full source file";
        let generated_binary_truncation_policy =
            "generated binary coefficient table derived from vendored full source file";
        let variant = "VSOP87B";
        let coordinate_family = "heliocentric spherical variables";
        let frame = "J2000 ecliptic/equinox";
        let units = "degrees and astronomical units";
        let solar_reduction = "geocentric solar reduction from Earth coefficients";
        let planetary_reduction = "geocentric planetary reduction against Earth coefficients";
        let transform_note =
            "J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform";

        let source_profile = |body: CelestialBody,
                              kind: Vsop87BodySourceKind,
                              provenance: &'static str,
                              accuracy: AccuracyClass| Vsop87BodySource {
            body,
            kind,
            provenance,
            accuracy,
        };

        let source_specification = |
            body: CelestialBody,
            source_file: &'static str,
            reduction: &'static str,
        | {
            let truncation_policy = match body {
                CelestialBody::Sun
                | CelestialBody::Mercury
                | CelestialBody::Venus
                | CelestialBody::Mars
                | CelestialBody::Jupiter
                | CelestialBody::Saturn
                | CelestialBody::Uranus
                | CelestialBody::Neptune => generated_binary_truncation_policy,
                _ => vendored_truncation_policy,
            };
            Some(Vsop87SourceSpecification {
                body,
                source_file,
                variant,
                coordinate_family,
                frame,
                units,
                reduction,
                transform_note,
                truncation_policy,
                date_range: earth_date_range,
            })
        };

        let canonical_sample = |
            body: CelestialBody,
            expected_longitude_deg: f64,
            expected_latitude_deg: f64,
            expected_distance_au: f64,
            max_longitude_delta_deg: f64,
            max_latitude_delta_deg: f64,
            max_distance_delta_au: f64,
        | {
            Some(Vsop87CanonicalEpochSample {
                body,
                expected_longitude_deg,
                expected_latitude_deg,
                expected_distance_au,
                max_longitude_delta_deg,
                max_latitude_delta_deg,
                max_distance_delta_au,
            })
        };

        vec![
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Sun,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "geocentric Sun reduced from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Earth source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Sun,
                    "VSOP87B.ear",
                    solar_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Sun,
                    280.377_843_416_648_5,
                    0.000_227_210_514_369_001,
                    0.983_327_682_322_294_2,
                    0.001,
                    0.000_01,
                    0.000_01,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Mercury,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Mercury heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Mercury source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Mercury,
                    "VSOP87B.mer",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Mercury,
                    271.904_744_694_147_67,
                    -0.995_553_498_474_437_4,
                    1.415_524_982_482_968,
                    0.000_000_001,
                    0.000_000_001,
                    0.000_000_000_001,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Venus,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Venus geocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Venus source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Venus,
                    "VSOP87B.ven",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Venus,
                    241.576_729_276_029_5,
                    2.066_187_460_260_189,
                    1.137_689_108_663_588,
                    0.001,
                    0.000_1,
                    0.000_01,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Mars,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Mars heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Mars source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Mars,
                    "VSOP87B.mar",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Mars,
                    327.974_906_233_385_87,
                    -1.067_660_978_531_137_7,
                    1.849_603_891_293_057_7,
                    0.000_000_001,
                    0.000_000_001,
                    0.000_000_000_001,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Jupiter,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Jupiter heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Jupiter source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Jupiter,
                    "VSOP87B.jup",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Jupiter,
                    25.258_084_319_944_018,
                    -1.262_035_369_214_697_3,
                    4.621_126_218_764_805,
                    0.004,
                    0.000_2,
                    0.000_1,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Saturn,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Saturn heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Saturn source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Saturn,
                    "VSOP87B.sat",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Saturn,
                    40.398_572_276_886_384,
                    -2.444_625_745_599_142_3,
                    8.652_748_862_003_302,
                    0.004,
                    0.000_2,
                    0.000_5,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Uranus,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Uranus heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Uranus source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Uranus,
                    "VSOP87B.ura",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Uranus,
                    314.819_126_206_595_1,
                    -0.658_295_956_624_516_5,
                    20.727_185_531_715_136,
                    0.006,
                    0.000_1,
                    0.000_1,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Neptune,
                    Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                    "Neptune heliocentric channel from a generated binary coefficient table derived from the vendored full IMCCE/CELMECH VSOP87B Neptune source file",
                    AccuracyClass::Exact,
                ),
                source_specification: source_specification(
                    CelestialBody::Neptune,
                    "VSOP87B.nep",
                    planetary_reduction,
                ),
                canonical_sample: canonical_sample(
                    CelestialBody::Neptune,
                    303.203_423_517_050_34,
                    0.234_955_476_702_893_77,
                    31.024_432_860_406_91,
                    0.001,
                    0.000_1,
                    0.000_1,
                ),
            },
            Vsop87BodyCatalogEntry {
                source_profile: source_profile(
                    CelestialBody::Pluto,
                    Vsop87BodySourceKind::MeanOrbitalElements,
                    "current mean-element fallback body until a Pluto-specific source path is selected",
                    AccuracyClass::Approximate,
                ),
                source_specification: None,
                canonical_sample: None,
            },
        ]
    })
}

/// Returns the structured source documentation for the current VSOP87-backed bodies.
pub fn source_specifications() -> Vec<Vsop87SourceSpecification> {
    body_catalog_entries()
        .iter()
        .filter_map(|entry| entry.source_specification.clone())
        .collect()
}

/// Formats a single VSOP87 source specification for reporting.
pub fn format_source_specification(spec: &Vsop87SourceSpecification) -> String {
    spec.summary_line()
}

/// Formats the current VSOP87 source-specification catalog for reporting.
pub fn format_source_specifications(specs: &[Vsop87SourceSpecification]) -> String {
    join_display(specs)
}

/// Returns the release-facing source-specification catalog string.
pub fn source_specifications_for_report() -> String {
    format_source_specifications(&source_specifications())
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
    vsop87_request_policy().summary_line()
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

/// Returns the release-facing reproducibility audit summary string.
pub fn source_audit_summary_for_report() -> String {
    source_audit_summary().to_string()
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
            source_specifications()
                .into_iter()
                .map(|spec| {
                    let blob =
                        checked_in_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
                            .expect("known VSOP87 generated blob");
                    Vsop87GeneratedBlobAudit {
                        body: spec.body,
                        source_file: spec.source_file,
                        byte_length: blob.len(),
                        fingerprint: fnv1a_64(blob),
                    }
                })
                .collect()
        })
        .clone()
}

/// Returns a small reproducibility summary for the current generated VSOP87B blobs.
pub fn generated_binary_audit_summary() -> Vsop87GeneratedBlobAuditSummary {
    let audits = generated_binary_audits();
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

/// Formats the checked-in generated VSOP87B blob audit for reporting.
pub fn format_generated_binary_audit_summary(summary: &Vsop87GeneratedBlobAuditSummary) -> String {
    summary.summary_line()
}

/// Returns the release-facing generated binary audit summary string.
pub fn generated_binary_audit_summary_for_report() -> String {
    generated_binary_audit_summary().to_string()
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
    format!(
        "VSOP87 source documentation: {} source specs, {} source-backed body profiles, {} fallback mean-element body profile{} ({}); source-backed bodies: {}; source files: {}; source-backed breakdown: {} generated binary bodies ({}), {} vendored full-file bodies ({}), {} truncated slice bodies ({}); date ranges: {}",
        summary.source_specification_count,
        summary.source_backed_profile_count,
        summary.fallback_profile_count,
        if summary.fallback_profile_count == 1 {
            ""
        } else {
            "s"
        },
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
pub fn source_documentation_summary_for_report() -> String {
    source_documentation_summary().summary_line()
}

/// Returns a consistency check for the current VSOP87 source-documentation catalog.
fn source_documentation_fields_are_consistent(source_specs: &[Vsop87SourceSpecification]) -> bool {
    source_specs.iter().all(|spec| {
        spec.variant == "VSOP87B"
            && spec.coordinate_family == "heliocentric spherical variables"
            && spec.frame == "J2000 ecliptic/equinox"
            && spec.units == "degrees and astronomical units"
            && spec.reduction.contains("geocentric")
            && spec.transform_note.contains("mean-obliquity transform")
            && spec.truncation_policy
                == "generated binary coefficient table derived from vendored full source file"
            && spec.date_range == "full public source file; J2000 canonical reference sample"
    })
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
        "VSOP87 source documentation health: {} ({} source specs, {} source files, {} source-backed profiles, {} body profiles; {} generated binary profiles ({}), {} vendored full-file profiles ({}), {} truncated profiles ({}), {} fallback profiles ({}); source files: {}; source-backed order: {}; source-backed partition order: {}; fallback order: {}; documented fields: {}){}",
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

fn source_documentation_health_issues(
    summary: &Vsop87SourceDocumentationSummary,
    source_specs: &[Vsop87SourceSpecification],
    body_profile_count: usize,
    source_file_count: usize,
) -> Vec<&'static str> {
    let expected_source_files = source_specs
        .iter()
        .map(|spec| spec.source_file)
        .collect::<Vec<_>>();
    let expected_source_backed_bodies = source_documentation_partition_bodies(summary);
    let mut issues = Vec::new();

    if summary.source_specification_count != source_file_count {
        issues.push("source specification/file count mismatch");
    }
    if summary.source_files != expected_source_files {
        issues.push("source file order mismatch");
    }
    if summary.source_backed_bodies != expected_source_backed_bodies {
        issues.push("source-backed body order mismatch");
    }
    if summary.source_backed_profile_count
        != summary.generated_binary_profile_count
            + summary.vendored_full_file_profile_count
            + summary.truncated_profile_count
    {
        issues.push("source-backed profile partition mismatch");
    }
    if summary.source_backed_profile_count + summary.fallback_profile_count != body_profile_count {
        issues.push("body profile coverage mismatch");
    }
    if summary.source_specification_count != source_specs.len() {
        issues.push("source specification catalog count mismatch");
    }
    if !source_documentation_fields_are_consistent(source_specs) {
        issues.push("documented field mismatch");
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

fn format_issue_labels(issues: &[&'static str]) -> String {
    if issues.is_empty() {
        "none".to_string()
    } else {
        issues.join(", ")
    }
}

fn format_source_files(source_files: &[&'static str]) -> String {
    if source_files.is_empty() {
        "none".to_string()
    } else {
        source_files.join(", ")
    }
}

/// Returns the release-facing source-documentation health string.
pub fn source_documentation_health_summary_for_report() -> String {
    source_documentation_health_summary().summary_line()
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
}

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

/// Returns the release-facing canonical VSOP87 J2000 evidence summary string.
pub fn canonical_epoch_evidence_summary_for_report() -> String {
    match canonical_epoch_evidence_summary() {
        Some(summary) => summary.summary_line(),
        None => "VSOP87 canonical J2000 source-backed evidence: unavailable".to_string(),
    }
}

/// Returns a concise note describing any canonical J2000 bodies outside the
/// current interim limits.
pub fn canonical_epoch_outlier_note_for_report() -> String {
    match canonical_epoch_body_evidence() {
        Some(evidence) => {
            let outliers = evidence
                .into_iter()
                .filter(|row| !row.within_interim_limits)
                .map(|row| row.body)
                .collect::<Vec<_>>();

            if outliers.is_empty() {
                "VSOP87 canonical J2000 interim outliers: none".to_string()
            } else {
                format!(
                    "VSOP87 canonical J2000 interim outliers: {}",
                    format_celestial_bodies(&outliers)
                )
            }
        }
        None => "VSOP87 canonical J2000 interim outliers: unavailable".to_string(),
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
}

impl fmt::Display for Vsop87CanonicalJ1900BatchParitySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the backend-owned canonical J1900 batch-path regression summary.
pub fn canonical_j1900_batch_parity_summary() -> Option<Vsop87CanonicalJ1900BatchParitySummary> {
    let backend = Vsop87Backend::new();
    let reference_epoch = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
    let requests = Vsop87Backend::supported_bodies()
        .iter()
        .cloned()
        .map(|body| {
            let mut request = EphemerisRequest::new(body, reference_epoch);
            request.frame = CoordinateFrame::Equatorial;
            request
        })
        .collect::<Vec<_>>();
    let results = backend.positions(&requests).ok()?;

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

    Some(Vsop87CanonicalJ1900BatchParitySummary {
        sample_count: results.len(),
        sample_bodies,
        reference_epoch,
        frame: CoordinateFrame::Equatorial,
        exact_count,
        interpolated_count,
        approximate_count,
        unknown_count,
    })
}

/// Returns the release-facing canonical J1900 batch-path regression summary string.
pub fn canonical_j1900_batch_parity_summary_for_report() -> String {
    match canonical_j1900_batch_parity_summary() {
        Some(summary) => summary.summary_line(),
        None => "VSOP87 canonical J1900 batch parity: unavailable".to_string(),
    }
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
pub fn source_body_evidence_summary_for_report() -> String {
    match source_body_evidence_summary() {
        Some(summary) => summary.summary_line(),
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
pub fn source_body_class_evidence_summary_for_report() -> String {
    match source_body_class_evidence_summary() {
        Some(summary) => format_source_body_class_evidence_summary(&summary),
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

/// Returns the supported vendored VSOP87B source files in source-spec order.
///
/// This list is primarily used by maintainer-facing regeneration tooling and
/// reproducibility checks so downstream code can discover the expected public
/// input files without hardcoding the table-specific match block.
pub fn supported_source_files() -> Vec<&'static str> {
    source_specifications()
        .into_iter()
        .map(|spec| spec.source_file)
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

fn canonical_epoch_requests() -> Vec<EphemerisRequest> {
    canonical_epoch_samples()
        .into_iter()
        .map(|sample| {
            let mut request = EphemerisRequest::new(
                sample.body,
                Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
            );
            request.apparent = Apparentness::Mean;
            request
        })
        .collect()
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

        evidence.push(Vsop87CanonicalBodyEvidence {
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
        });
    }

    Some(evidence)
}

/// Returns the canonical J2000 error envelope summary used by release-facing
/// validation reports.
pub fn canonical_epoch_evidence_summary() -> Option<Vsop87CanonicalEvidenceSummary> {
    let body_evidence = canonical_epoch_body_evidence()?;
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

    for evidence in &body_evidence {
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

/// Returns the canonical J2000 equatorial companion evidence summary used by
/// release-facing validation reports.
pub fn canonical_epoch_equatorial_evidence_summary(
) -> Option<Vsop87CanonicalEquatorialEvidenceSummary> {
    let body_evidence = canonical_epoch_equatorial_body_evidence()?;
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

    for evidence in &body_evidence {
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

/// Returns the release-facing canonical VSOP87 equatorial companion evidence
/// summary string.
pub fn canonical_epoch_equatorial_evidence_summary_for_report() -> String {
    match canonical_epoch_equatorial_evidence_summary() {
        Some(summary) => summary.summary_line(),
        None => "VSOP87 canonical J2000 equatorial companion evidence: unavailable".to_string(),
    }
}

fn source_kind_for_body(body: CelestialBody) -> Option<Vsop87BodySourceKind> {
    body_catalog_entries()
        .iter()
        .find(|entry| entry.source_profile.body == body)
        .map(|entry| entry.source_profile.kind)
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

/// A pure-Rust planetary backend.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vsop87Backend;

impl Vsop87Backend {
    /// Creates a new backend instance.
    pub const fn new() -> Self {
        Self
    }

    fn days_since_j2000(instant: Instant) -> f64 {
        instant.julian_day.days() - J2000
    }

    fn supported_bodies() -> &'static [CelestialBody] {
        &[
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
            CelestialBody::Pluto,
        ]
    }

    fn earth_elements(days: f64) -> OrbitalElements {
        OrbitalElements::new(
            0.0,
            0.0,
            282.9404 + 4.70935e-5 * days,
            1.000000,
            0.016709 - 1.151e-9 * days,
            356.0470 + 0.985_600_258_5 * days,
        )
    }

    fn orbital_elements(body: CelestialBody, days: f64) -> Option<OrbitalElements> {
        match body {
            CelestialBody::Mercury => Some(OrbitalElements::new(
                48.3313 + 3.24587e-5 * days,
                7.0047 + 5.00e-8 * days,
                29.1241 + 1.01444e-5 * days,
                0.387098,
                0.205635 + 5.59e-10 * days,
                168.6562 + 4.092_334_436_8 * days,
            )),
            CelestialBody::Venus => Some(OrbitalElements::new(
                76.6799 + 2.46590e-5 * days,
                3.3946 + 2.75e-8 * days,
                54.8910 + 1.38374e-5 * days,
                0.72333,
                0.006773 - 1.302e-9 * days,
                48.0052 + 1.602_130_224_4 * days,
            )),
            CelestialBody::Mars => Some(OrbitalElements::new(
                49.5574 + 2.11081e-5 * days,
                1.8497 - 1.78e-8 * days,
                286.5016 + 2.92961e-5 * days,
                1.523688,
                0.093405 + 2.516e-9 * days,
                18.6021 + 0.524_020_776_6 * days,
            )),
            CelestialBody::Jupiter => Some(OrbitalElements::new(
                100.4542 + 2.76854e-5 * days,
                1.3030 - 1.557e-7 * days,
                273.8777 + 1.64505e-5 * days,
                5.20256,
                0.048498 + 4.469e-9 * days,
                19.8950 + 0.083_085_300_1 * days,
            )),
            CelestialBody::Saturn => Some(OrbitalElements::new(
                113.6634 + 2.38980e-5 * days,
                2.4886 - 1.081e-7 * days,
                339.3939 + 2.97661e-5 * days,
                9.55475,
                0.055546 - 9.499e-9 * days,
                316.9670 + 0.033_444_228_2 * days,
            )),
            CelestialBody::Uranus => Some(OrbitalElements::new(
                74.0005 + 1.3978e-5 * days,
                0.7733 + 1.9e-8 * days,
                96.6612 + 3.0565e-5 * days,
                19.18171 - 1.55e-8 * days,
                0.047318 + 7.45e-9 * days,
                142.5905 + 0.011_725_806 * days,
            )),
            CelestialBody::Neptune => Some(OrbitalElements::new(
                131.7806 + 3.0173e-5 * days,
                1.7700 - 2.55e-7 * days,
                272.8461 - 6.027e-6 * days,
                30.05826 + 3.313e-8 * days,
                0.008606 + 2.15e-9 * days,
                260.2471 + 0.005_995_147 * days,
            )),
            CelestialBody::Pluto => Some(OrbitalElements::new(
                110.30347,
                17.14175,
                113.76329,
                39.481_686_77,
                0.248_807_66,
                14.53 + 0.003_96 * days,
            )),
            _ => None,
        }
    }

    fn heliocentric_coordinates(elements: OrbitalElements) -> HeliocentricCoordinates {
        let mean_anomaly = normalize_degrees(elements.mean_anomaly);
        let eccentric_anomaly = solve_kepler(mean_anomaly, elements.eccentricity);
        let true_anomaly = true_anomaly_from_eccentric(eccentric_anomaly, elements.eccentricity);
        let eccentric_anomaly_rad = eccentric_anomaly.to_radians();
        let radius =
            elements.semi_major_axis * (1.0 - elements.eccentricity * eccentric_anomaly_rad.cos());

        let node = elements.ascending_node.to_radians();
        let inclination = elements.inclination.to_radians();
        let perihelion = elements.argument_of_perihelion.to_radians();
        let lon = (true_anomaly + perihelion).to_radians();

        let xh = radius * (node.cos() * lon.cos() - node.sin() * lon.sin() * inclination.cos());
        let yh = radius * (node.sin() * lon.cos() + node.cos() * lon.sin() * inclination.cos());
        let zh = radius * (lon.sin() * inclination.sin());

        HeliocentricCoordinates { xh, yh, zh }
    }

    fn geocentric_coordinates(body: CelestialBody, days: f64) -> Option<HeliocentricCoordinates> {
        if body == CelestialBody::Sun {
            return Some(Self::geocentric_sun_from_vsop87b(days));
        }

        if matches!(
            body,
            CelestialBody::Mercury
                | CelestialBody::Venus
                | CelestialBody::Mars
                | CelestialBody::Jupiter
                | CelestialBody::Saturn
                | CelestialBody::Uranus
                | CelestialBody::Neptune
        ) {
            let earth = Self::heliocentric_earth_from_vsop87b(days);
            let target = match body {
                CelestialBody::Mercury => Self::heliocentric_mercury_from_vsop87b(days),
                CelestialBody::Venus => Self::heliocentric_venus_from_vsop87b(days),
                CelestialBody::Mars => Self::heliocentric_mars_from_vsop87b(days),
                CelestialBody::Jupiter => Self::heliocentric_jupiter_from_vsop87b(days),
                CelestialBody::Saturn => Self::heliocentric_saturn_from_vsop87b(days),
                CelestialBody::Uranus => Self::heliocentric_uranus_from_vsop87b(days),
                CelestialBody::Neptune => Self::heliocentric_neptune_from_vsop87b(days),
                _ => unreachable!("body was checked above"),
            };
            return Some(HeliocentricCoordinates {
                xh: target.xh - earth.xh,
                yh: target.yh - earth.yh,
                zh: target.zh - earth.zh,
            });
        }

        let earth = Self::heliocentric_coordinates(Self::earth_elements(days));
        let target = Self::heliocentric_coordinates(Self::orbital_elements(body, days)?);
        Some(HeliocentricCoordinates {
            xh: target.xh - earth.xh,
            yh: target.yh - earth.yh,
            zh: target.zh - earth.zh,
        })
    }

    fn geocentric_sun_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let earth = Self::heliocentric_earth_from_vsop87b(days);
        HeliocentricCoordinates {
            xh: -earth.xh,
            yh: -earth.yh,
            zh: -earth.zh,
        }
    }

    fn heliocentric_earth_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let earth = vsop87b_earth::earth_lbr(J2000 + days);
        spherical_lbr_to_cartesian(earth.longitude_rad, earth.latitude_rad, earth.radius_au)
    }

    fn heliocentric_mercury_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let mercury = vsop87b_mercury::mercury_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            mercury.longitude_rad,
            mercury.latitude_rad,
            mercury.radius_au,
        )
    }

    fn heliocentric_venus_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let venus = vsop87b_venus::venus_lbr(J2000 + days);
        spherical_lbr_to_cartesian(venus.longitude_rad, venus.latitude_rad, venus.radius_au)
    }

    fn heliocentric_mars_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let mars = vsop87b_mars::mars_lbr(J2000 + days);
        spherical_lbr_to_cartesian(mars.longitude_rad, mars.latitude_rad, mars.radius_au)
    }

    fn heliocentric_jupiter_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let jupiter = vsop87b_jupiter::jupiter_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            jupiter.longitude_rad,
            jupiter.latitude_rad,
            jupiter.radius_au,
        )
    }

    fn heliocentric_saturn_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let saturn = vsop87b_saturn::saturn_lbr(J2000 + days);
        spherical_lbr_to_cartesian(saturn.longitude_rad, saturn.latitude_rad, saturn.radius_au)
    }

    fn heliocentric_uranus_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let uranus = vsop87b_uranus::uranus_lbr(J2000 + days);
        spherical_lbr_to_cartesian(uranus.longitude_rad, uranus.latitude_rad, uranus.radius_au)
    }

    fn heliocentric_neptune_from_vsop87b(days: f64) -> HeliocentricCoordinates {
        let neptune = vsop87b_neptune::neptune_lbr(J2000 + days);
        spherical_lbr_to_cartesian(
            neptune.longitude_rad,
            neptune.latitude_rad,
            neptune.radius_au,
        )
    }

    fn distance_au(coords: HeliocentricCoordinates) -> f64 {
        coords.xh.hypot(coords.yh.hypot(coords.zh))
    }

    fn to_ecliptic(coords: HeliocentricCoordinates) -> EclipticCoordinates {
        let longitude = Longitude::from_degrees(coords.yh.atan2(coords.xh).to_degrees());
        let latitude = Latitude::from_degrees(
            coords
                .zh
                .atan2((coords.xh * coords.xh + coords.yh * coords.yh).sqrt())
                .to_degrees(),
        );
        EclipticCoordinates::new(longitude, latitude, Some(Self::distance_au(coords)))
    }

    fn to_equatorial(coords: HeliocentricCoordinates, instant: Instant) -> EquatorialCoordinates {
        Self::to_ecliptic(coords).to_equatorial(instant.mean_obliquity())
    }

    fn motion(body: CelestialBody, days: f64) -> Option<Motion> {
        // A symmetric one-day span gives stable chart-facing daily rates while
        // keeping the preliminary element model simple and deterministic. These
        // are finite-difference estimates of the same mean geocentric model, not
        // apparent velocities from a full VSOP87/light-time reduction.
        const HALF_SPAN_DAYS: f64 = 0.5;
        const FULL_SPAN_DAYS: f64 = HALF_SPAN_DAYS * 2.0;

        let before = Self::to_ecliptic(Self::geocentric_coordinates(
            body.clone(),
            days - HALF_SPAN_DAYS,
        )?);
        let after = Self::to_ecliptic(Self::geocentric_coordinates(body, days + HALF_SPAN_DAYS)?);

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
}

impl EphemerisBackend for Vsop87Backend {
    fn metadata(&self) -> BackendMetadata {
        let source_profiles = body_source_profiles();
        let vendored_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::VendoredVsop87b)
            .count();
        let generated_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b)
            .count();
        let truncated_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::TruncatedVsop87b)
            .count();
        let fallback_count = source_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
            .count();

        let vendored_path_label = pluralize_body_path(vendored_count);
        let generated_path_label = pluralize_body_path(generated_count);
        let truncated_path_label = pluralize_body_path(truncated_count);
        let fallback_path_label = pluralize_body_path(fallback_count);

        BackendMetadata {
            id: BackendId::new(PACKAGE_NAME),
            version: env!("CARGO_PKG_VERSION").to_string(),
            family: BackendFamily::Algorithmic,
            provenance: BackendProvenance {
                summary: if generated_count == 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if vendored_count == 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {generated_count} generated binary VSOP87B {generated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if generated_count > 0 && truncated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {generated_count} generated binary VSOP87B {generated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else if generated_count == 0 {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {truncated_count} source-backed truncated VSOP87B {truncated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                } else {
                    format!(
                        "Mixed pure-Rust planetary backend: {vendored_count} vendored full-file VSOP87B {vendored_path_label}, {generated_count} generated binary VSOP87B {generated_path_label}, {truncated_count} source-backed truncated VSOP87B {truncated_path_label}, {fallback_count} fallback mean-element {fallback_path_label}, and geocentric reduction."
                    )
                },
                data_sources: source_specifications()
                    .into_iter()
                    .map(|spec| {
                        format!(
                            "{}: IMCCE/CELMECH {} {} ({}, {}, {}, {}, {}, {}, {})",
                            spec.body,
                            spec.variant,
                            spec.source_file,
                            spec.coordinate_family,
                            spec.frame,
                            spec.units,
                            spec.reduction,
                            spec.transform_note,
                            spec.truncation_policy,
                            spec.date_range,
                        )
                    })
                    .chain([
                        "Paul Schlyter-style mean orbital elements for planets outside the source-backed VSOP87 coefficient tables".to_string(),
                        "Meeus-style coordinate transforms for geocentric reduction".to_string(),
                    ])
                    .collect(),
            },
            nominal_range: TimeRange::new(None, None),
            supported_time_scales: vec![TimeScale::Tt, TimeScale::Tdb],
            body_coverage: Self::supported_bodies().to_vec(),
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
        Self::supported_bodies().contains(&body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        validate_zodiac_policy(req, BACKEND_LABEL, &[ZodiacMode::Tropical])?;

        validate_request_policy(
            req,
            BACKEND_LABEL,
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            false,
        )?;

        validate_observer_policy(req, BACKEND_LABEL, false)?;

        let days = Self::days_since_j2000(req.instant);
        let geocentric = Self::geocentric_coordinates(req.body.clone(), days).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                format!("requested body is not implemented in {BACKEND_LABEL}"),
            )
        })?;

        let mut result = EphemerisResult::new(
            BackendId::new(PACKAGE_NAME),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.quality = match source_kind_for_body(req.body.clone()) {
            Some(Vsop87BodySourceKind::VendoredVsop87b)
            | Some(Vsop87BodySourceKind::GeneratedBinaryVsop87b) => QualityAnnotation::Exact,
            Some(Vsop87BodySourceKind::TruncatedVsop87b)
            | Some(Vsop87BodySourceKind::MeanOrbitalElements)
            | None => QualityAnnotation::Approximate,
        };
        result.ecliptic = Some(Self::to_ecliptic(geocentric));
        result.equatorial = Some(Self::to_equatorial(geocentric, req.instant));
        result.motion = Self::motion(req.body.clone(), days);
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug)]
struct OrbitalElements {
    ascending_node: f64,
    inclination: f64,
    argument_of_perihelion: f64,
    semi_major_axis: f64,
    eccentricity: f64,
    mean_anomaly: f64,
}

impl OrbitalElements {
    const fn new(
        ascending_node: f64,
        inclination: f64,
        argument_of_perihelion: f64,
        semi_major_axis: f64,
        eccentricity: f64,
        mean_anomaly: f64,
    ) -> Self {
        Self {
            ascending_node,
            inclination,
            argument_of_perihelion,
            semi_major_axis,
            eccentricity,
            mean_anomaly,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct HeliocentricCoordinates {
    xh: f64,
    yh: f64,
    zh: f64,
}

fn spherical_lbr_to_cartesian(
    longitude_rad: f64,
    latitude_rad: f64,
    radius_au: f64,
) -> HeliocentricCoordinates {
    let cos_latitude = latitude_rad.cos();
    HeliocentricCoordinates {
        xh: radius_au * cos_latitude * longitude_rad.cos(),
        yh: radius_au * cos_latitude * longitude_rad.sin(),
        zh: radius_au * latitude_rad.sin(),
    }
}

fn normalize_degrees(angle: f64) -> f64 {
    angle.rem_euclid(360.0)
}

fn signed_longitude_delta_degrees(start: f64, end: f64) -> f64 {
    (end - start + 180.0).rem_euclid(360.0) - 180.0
}

fn pluralize_body_path(count: usize) -> &'static str {
    if count == 1 {
        "body path"
    } else {
        "body paths"
    }
}

fn solve_kepler(mean_anomaly_degrees: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly_degrees.to_radians();
    let mut e = m + eccentricity * m.sin() * (1.0 + eccentricity * m.cos());
    for _ in 0..10 {
        let delta = (e - eccentricity * e.sin() - m) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1e-12 {
            break;
        }
    }
    e.to_degrees()
}

fn true_anomaly_from_eccentric(eccentric_anomaly_degrees: f64, eccentricity: f64) -> f64 {
    let e = eccentric_anomaly_degrees.to_radians();
    let numerator = (1.0 + eccentricity).sqrt() * (e / 2.0).sin();
    let denominator = (1.0 - eccentricity).sqrt() * (e / 2.0).cos();
    (2.0 * numerator.atan2(denominator))
        .to_degrees()
        .rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_name_is_stable() {
        assert_eq!(PACKAGE_NAME, "pleiades-vsop87");
    }

    #[test]
    fn backend_reports_major_planets() {
        let backend = Vsop87Backend::new();
        assert!(backend.supports_body(CelestialBody::Sun));
        assert!(backend.supports_body(CelestialBody::Mars));
        assert!(!backend.supports_body(CelestialBody::Moon));
    }

    #[test]
    fn j2000_sun_position_uses_vendored_vsop87b_earth_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Sun);
        let result = backend.position(&request).expect("sun query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Earth file evaluated
        // at J2000 and converted to geometric geocentric solar coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 280.377_843_416_648_5, 0.001);
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            0.000_227_210_514_369_001,
            0.000_01,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            0.983_327_682_322_294_2,
            0.000_01,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_mercury_position_uses_vendored_vsop87b_mercury_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Mercury);
        let result = backend
            .position(&request)
            .expect("Mercury query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Mercury and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            271.904_744_694_147_67,
            0.000_000_001,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            -0.995_553_498_474_437_4,
            0.000_000_001,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            1.415_524_982_482_968,
            0.000_000_000_001,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_venus_position_uses_vendored_vsop87b_venus_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Venus);
        let result = backend.position(&request).expect("Venus query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Venus and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 241.576_729_276_029_5, 0.001);
        assert_degrees_close(ecliptic.latitude.degrees(), 2.066_187_460_260_189, 0.000_1);
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            1.137_689_108_663_588,
            0.000_01,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_mars_position_uses_generated_vsop87b_mars_table() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Mars);
        let result = backend.position(&request).expect("Mars query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Mars and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates. The runtime path now reaches them through the generated
        // binary Mars table derived from the vendored Mars source file.
        assert_degrees_close(
            ecliptic.longitude.degrees(),
            327.974_906_233_385_87,
            0.000_000_001,
        );
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            -1.067_660_978_531_137_7,
            0.000_000_001,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            1.849_603_891_293_057_7,
            0.000_000_000_001,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_jupiter_position_uses_full_vsop87b_jupiter_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Jupiter);
        let result = backend
            .position(&request)
            .expect("Jupiter query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Jupiter and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 25.258_084_319_944_018, 0.004);
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            -1.262_035_369_214_697_3,
            0.000_2,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            4.621_126_218_764_805,
            0.000_1,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_saturn_position_uses_full_vsop87b_saturn_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Saturn);
        let result = backend
            .position(&request)
            .expect("Saturn query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Saturn and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 40.398_572_276_886_384, 0.004);
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            -2.444_625_745_599_142_3,
            0.000_2,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            8.652_748_862_003_302,
            0.000_5,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_uranus_position_uses_full_vsop87b_uranus_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Uranus);
        let result = backend
            .position(&request)
            .expect("Uranus query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Uranus and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 314.819_126_206_595_1, 0.006);
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            -0.658_295_956_624_516_5,
            0.000_1,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            20.727_185_531_715_136,
            0.000_1,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn j2000_neptune_position_uses_full_vsop87b_neptune_file() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Neptune);
        let result = backend
            .position(&request)
            .expect("Neptune query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");

        // Golden values are the full public IMCCE VSOP87B Neptune and Earth
        // files evaluated at J2000 and reduced to geometric geocentric ecliptic
        // coordinates.
        assert_degrees_close(ecliptic.longitude.degrees(), 303.203_423_517_050_34, 0.001);
        assert_degrees_close(
            ecliptic.latitude.degrees(),
            0.234_955_476_702_893_77,
            0.000_1,
        );
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            31.024_432_860_406_91,
            0.000_1,
        );
        assert_eq!(result.quality, QualityAnnotation::Exact);
    }

    #[test]
    fn batch_query_covers_all_source_backed_vsop87_paths() {
        let backend = Vsop87Backend::new();
        let samples = canonical_epoch_samples();
        let requests = samples
            .iter()
            .map(|sample| mean_request(sample.body.clone()))
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch query should work for every source-backed body");

        assert_eq!(results.len(), samples.len());
        for (sample, result) in samples.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );
            let expected_quality = QualityAnnotation::Exact;
            assert_eq!(result.quality, expected_quality);
        }
    }

    #[test]
    fn batch_query_covers_all_supported_vsop87_paths() {
        let backend = Vsop87Backend::new();
        let requests = Vsop87Backend::supported_bodies()
            .iter()
            .cloned()
            .map(mean_request)
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch query should work for every supported body");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            match result.body {
                CelestialBody::Pluto => {
                    assert_eq!(result.quality, QualityAnnotation::Approximate);
                }
                _ => {
                    assert_eq!(result.quality, QualityAnnotation::Exact);
                }
            }

            let single = backend
                .position(request)
                .expect("single query should work for every supported body");
            assert_eq!(single.body, result.body);
            assert_eq!(single.quality, result.quality);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            let single_ecliptic = single
                .ecliptic
                .as_ref()
                .expect("single-query ecliptic result should exist");
            assert_eq!(
                ecliptic.longitude.degrees(),
                single_ecliptic.longitude.degrees()
            );
            assert_eq!(
                ecliptic.latitude.degrees(),
                single_ecliptic.latitude.degrees()
            );
            assert_eq!(
                ecliptic.distance_au.expect("distance should exist"),
                single_ecliptic
                    .distance_au
                    .expect("single-query distance should exist")
            );

            if let Some(sample) = canonical_epoch_samples()
                .into_iter()
                .find(|sample| sample.body == result.body)
            {
                assert_degrees_close(
                    ecliptic.longitude.degrees(),
                    sample.expected_longitude_deg,
                    sample.max_longitude_delta_deg,
                );
                assert_degrees_close(
                    ecliptic.latitude.degrees(),
                    sample.expected_latitude_deg,
                    sample.max_latitude_delta_deg,
                );
                assert_close(
                    ecliptic.distance_au.expect("distance should exist"),
                    sample.expected_distance_au,
                    sample.max_distance_delta_au,
                );
            }
        }
    }

    #[test]
    fn batch_query_preserves_supported_vsop87_paths_for_tdb_requests() {
        let backend = Vsop87Backend::new();
        let requests = Vsop87Backend::supported_bodies()
            .iter()
            .cloned()
            .map(|body| {
                let mut request = mean_request(body);
                request.instant.scale = TimeScale::Tdb;
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch TDB query should work for every supported body");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant.scale, TimeScale::Tdb);
            match result.body {
                CelestialBody::Pluto => {
                    assert_eq!(result.quality, QualityAnnotation::Approximate);
                }
                _ => {
                    assert_eq!(result.quality, QualityAnnotation::Exact);
                }
            }

            let single = backend
                .position(request)
                .expect("single TDB query should work for every supported body");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant.scale, TimeScale::Tdb);
            assert_eq!(single.quality, result.quality);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            let single_ecliptic = single
                .ecliptic
                .as_ref()
                .expect("single-query ecliptic result should exist");
            assert_eq!(
                ecliptic.longitude.degrees(),
                single_ecliptic.longitude.degrees()
            );
            assert_eq!(
                ecliptic.latitude.degrees(),
                single_ecliptic.latitude.degrees()
            );
            assert_eq!(
                ecliptic.distance_au.expect("distance should exist"),
                single_ecliptic
                    .distance_au
                    .expect("single-query distance should exist")
            );

            if let Some(sample) = canonical_epoch_samples()
                .into_iter()
                .find(|sample| sample.body == result.body)
            {
                assert_degrees_close(
                    ecliptic.longitude.degrees(),
                    sample.expected_longitude_deg,
                    sample.max_longitude_delta_deg,
                );
                assert_degrees_close(
                    ecliptic.latitude.degrees(),
                    sample.expected_latitude_deg,
                    sample.max_latitude_delta_deg,
                );
                assert_close(
                    ecliptic.distance_au.expect("distance should exist"),
                    sample.expected_distance_au,
                    sample.max_distance_delta_au,
                );
            }
        }
    }

    #[test]
    fn batch_query_preserves_supported_vsop87_paths_for_mixed_time_scales() {
        let backend = Vsop87Backend::new();
        let requests = Vsop87Backend::supported_bodies()
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, body)| {
                let mut request = mean_request(body);
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
            .expect("batch mixed-scale query should work for every supported body");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant.scale, request.instant.scale);
            let single = backend
                .position(request)
                .expect("single mixed-scale query should work for every supported body");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant.scale, request.instant.scale);
            assert_eq!(single.quality, result.quality);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            let single_ecliptic = single
                .ecliptic
                .as_ref()
                .expect("single-query ecliptic result should exist");
            assert_eq!(
                ecliptic.longitude.degrees(),
                single_ecliptic.longitude.degrees()
            );
            assert_eq!(
                ecliptic.latitude.degrees(),
                single_ecliptic.latitude.degrees()
            );
            assert_eq!(
                ecliptic.distance_au.expect("distance should exist"),
                single_ecliptic
                    .distance_au
                    .expect("single-query distance should exist")
            );
        }
    }

    #[test]
    fn batch_query_preserves_canonical_sample_order_for_source_backed_paths() {
        let backend = Vsop87Backend::new();
        let mut requests = canonical_epoch_requests();
        let mut samples = canonical_epoch_samples();
        requests.reverse();
        samples.reverse();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve input order for every source-backed body");

        assert_eq!(results.len(), samples.len());
        for (sample, result) in samples.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );
        }
    }

    #[test]
    fn batch_query_preserves_equatorial_frame_and_values() {
        let backend = Vsop87Backend::new();
        let mut requests = canonical_epoch_requests();
        let mut samples = canonical_epoch_samples();
        requests.reverse();
        samples.reverse();
        for request in &mut requests {
            request.frame = CoordinateFrame::Equatorial;
        }

        let results = backend
            .positions(&requests)
            .expect("batch equatorial query should preserve the canonical sample order");

        assert_eq!(results.len(), samples.len());
        for (sample, result) in samples.iter().zip(results.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );

            let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial result should exist");

            assert_eq!(equatorial, &expected);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn batch_query_preserves_supported_vsop87_paths_at_the_j1900_reference_epoch() {
        let backend = Vsop87Backend::new();
        let instant = Instant::new(pleiades_types::JulianDay::from_days(J1900), TimeScale::Tdb);
        let requests = Vsop87Backend::supported_bodies()
            .iter()
            .cloned()
            .map(|body| {
                let mut request = mean_request_at(body, instant);
                request.frame = CoordinateFrame::Equatorial;
                request
            })
            .collect::<Vec<_>>();

        let results = backend
            .positions(&requests)
            .expect("batch query should preserve the supported planetary set at J1900");

        assert_eq!(results.len(), requests.len());
        for (request, result) in requests.iter().zip(results.iter()) {
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.frame, CoordinateFrame::Equatorial);
            match result.body {
                CelestialBody::Pluto => {
                    assert_eq!(result.quality, QualityAnnotation::Approximate);
                }
                _ => {
                    assert_eq!(result.quality, QualityAnnotation::Exact);
                }
            }

            let single = backend
                .position(request)
                .expect("single query should match the J1900 batch path");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant, result.instant);
            assert_eq!(single.frame, result.frame);
            assert_eq!(single.quality, result.quality);
            assert_eq!(single.ecliptic, result.ecliptic);
            assert_eq!(single.equatorial, result.equatorial);
            assert_eq!(single.motion, result.motion);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert!(ecliptic.longitude.degrees().is_finite());
            assert!(ecliptic.latitude.degrees().is_finite());
            assert!(ecliptic
                .distance_au
                .expect("distance should exist")
                .is_finite());

            let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial result should exist");

            assert_eq!(equatorial, &expected);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn batch_query_preserves_mixed_frame_requests_and_values() {
        let backend = Vsop87Backend::new();
        let requests = canonical_epoch_requests()
            .into_iter()
            .enumerate()
            .map(|(index, mut request)| {
                request.frame = if index % 2 == 0 {
                    CoordinateFrame::Ecliptic
                } else {
                    CoordinateFrame::Equatorial
                };
                request
            })
            .collect::<Vec<_>>();
        let samples = canonical_epoch_samples();

        let results = backend
            .positions(&requests)
            .expect("mixed frame batch query should preserve the canonical sample order");

        assert_eq!(results.len(), samples.len());
        for ((request, result), sample) in requests.iter().zip(results.iter()).zip(samples.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.frame, request.frame);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );

            let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
            let equatorial = result
                .equatorial
                .as_ref()
                .expect("equatorial result should exist");

            assert_eq!(equatorial, &expected);
            assert!(equatorial.right_ascension.degrees().is_finite());
            assert!(equatorial.declination.degrees().is_finite());
        }
    }

    #[test]
    fn finite_difference_motion_is_reported_for_supported_bodies() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Mars);
        let result = backend.position(&request).expect("Mars query should work");
        let motion = result.motion.expect("motion should be populated");

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
    }

    #[test]
    fn topocentric_requests_are_rejected_explicitly() {
        let backend = Vsop87Backend::new();
        let mut request = mean_request(CelestialBody::Mars);
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
        let backend = Vsop87Backend::new();
        let mut request = mean_request(CelestialBody::Mars);
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

    #[test]
    fn apparent_requests_are_rejected_explicitly() {
        let backend = Vsop87Backend::new();
        let mut request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        request.apparent = Apparentness::Apparent;

        let error = backend
            .position(&request)
            .expect_err("apparent requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains(BACKEND_LABEL));
    }

    #[test]
    fn batch_query_rejects_apparent_requests_explicitly() {
        let backend = Vsop87Backend::new();
        let mut request = EphemerisRequest::new(
            CelestialBody::Sun,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
        request.apparent = Apparentness::Apparent;

        let error = backend
            .positions(&[request])
            .expect_err("apparent batch requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::InvalidRequest);
        assert!(error.message.contains(BACKEND_LABEL));
    }

    #[test]
    fn unsupported_bodies_report_the_current_backend_label() {
        let backend = Vsop87Backend::new();
        let request = mean_request(CelestialBody::Moon);

        let error = backend
            .position(&request)
            .expect_err("moon requests should be unsupported");
        assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
        assert!(error.message.contains(BACKEND_LABEL));
    }

    #[test]
    fn tdb_requests_are_accepted_like_tt_requests() {
        let backend = Vsop87Backend::new();
        let tt_request = mean_request(CelestialBody::Mars);
        let tdb_request = EphemerisRequest::new(
            CelestialBody::Mars,
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
    fn batch_query_preserves_mixed_time_scales_and_values() {
        let backend = Vsop87Backend::new();
        let requests = canonical_epoch_requests()
            .into_iter()
            .enumerate()
            .map(|(index, mut request)| {
                request.instant.scale = if index % 2 == 0 {
                    TimeScale::Tt
                } else {
                    TimeScale::Tdb
                };
                request
            })
            .collect::<Vec<_>>();
        let samples = canonical_epoch_samples();

        let results = backend
            .positions(&requests)
            .expect("mixed-scale batch query should preserve the canonical sample order");

        assert_eq!(results.len(), samples.len());
        for ((request, result), sample) in requests.iter().zip(results.iter()).zip(samples.iter()) {
            assert_eq!(result.body, sample.body);
            assert_eq!(result.body, request.body);
            assert_eq!(result.instant, request.instant);
            assert_eq!(result.instant.scale, request.instant.scale);

            let single = backend
                .position(request)
                .expect("single mixed-scale query should preserve the canonical sample order");
            assert_eq!(single.body, result.body);
            assert_eq!(single.instant, result.instant);
            assert_eq!(single.ecliptic, result.ecliptic);
            assert_eq!(single.equatorial, result.equatorial);
            assert_eq!(single.motion, result.motion);

            let ecliptic = result
                .ecliptic
                .as_ref()
                .expect("ecliptic result should exist");
            assert_degrees_close(
                ecliptic.longitude.degrees(),
                sample.expected_longitude_deg,
                sample.max_longitude_delta_deg,
            );
            assert_degrees_close(
                ecliptic.latitude.degrees(),
                sample.expected_latitude_deg,
                sample.max_latitude_delta_deg,
            );
            assert_close(
                ecliptic.distance_au.expect("distance should exist"),
                sample.expected_distance_au,
                sample.max_distance_delta_au,
            );
        }
    }

    #[test]
    fn metadata_identifies_source_backed_planet_vsop87b_paths() {
        let metadata = Vsop87Backend::new().metadata();
        assert!(metadata
            .provenance
            .summary
            .contains("8 generated binary VSOP87B body paths"));
        assert!(!metadata
            .provenance
            .summary
            .contains("vendored full-file VSOP87B body paths"));
        assert!(metadata
            .provenance
            .summary
            .contains("1 fallback mean-element body path"));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Sun: IMCCE/CELMECH VSOP87B VSOP87B.ear")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("mean-obliquity transform")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("outside the source-backed VSOP87 coefficient tables")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Mercury: IMCCE/CELMECH VSOP87B VSOP87B.mer")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Venus: IMCCE/CELMECH VSOP87B VSOP87B.ven")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Mars: IMCCE/CELMECH VSOP87B VSOP87B.mar")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Jupiter: IMCCE/CELMECH VSOP87B VSOP87B.jup")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Saturn: IMCCE/CELMECH VSOP87B VSOP87B.sat")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Uranus: IMCCE/CELMECH VSOP87B VSOP87B.ura")));
        assert!(metadata
            .provenance
            .data_sources
            .iter()
            .any(|source| source.contains("Neptune: IMCCE/CELMECH VSOP87B VSOP87B.nep")));
        assert_eq!(
            metadata.supported_time_scales,
            vec![TimeScale::Tt, TimeScale::Tdb]
        );
    }

    #[test]
    fn body_source_profiles_identify_generated_binary_and_full_file_paths() {
        let profiles = body_source_profiles();
        assert_eq!(profiles.len(), Vsop87Backend::supported_bodies().len());

        for body in [
            CelestialBody::Sun,
            CelestialBody::Mercury,
            CelestialBody::Venus,
            CelestialBody::Mars,
            CelestialBody::Jupiter,
            CelestialBody::Saturn,
            CelestialBody::Uranus,
            CelestialBody::Neptune,
        ] {
            let profile = profiles
                .iter()
                .find(|profile| profile.body == body)
                .expect("source profile should exist");
            assert_eq!(profile.kind, Vsop87BodySourceKind::GeneratedBinaryVsop87b);
            assert_eq!(profile.accuracy, AccuracyClass::Exact);
            assert!(profile
                .provenance
                .contains("vendored full IMCCE/CELMECH VSOP87B"));
        }

        let pluto = profiles
            .iter()
            .find(|profile| profile.body == CelestialBody::Pluto)
            .expect("Pluto profile should exist");
        assert_eq!(pluto.kind, Vsop87BodySourceKind::MeanOrbitalElements);
        assert!(pluto.provenance.contains("fallback"));
    }

    #[test]
    fn canonical_epoch_samples_cover_source_backed_paths() {
        let samples = canonical_epoch_samples();
        assert_eq!(samples.len(), 8);
        assert!(samples
            .iter()
            .any(|sample| sample.body == CelestialBody::Sun));
        assert!(samples
            .iter()
            .any(|sample| sample.body == CelestialBody::Mercury));
        assert!(samples
            .iter()
            .any(|sample| sample.body == CelestialBody::Neptune));
        assert!(samples
            .iter()
            .all(|sample| sample.max_longitude_delta_deg > 0.0));
        assert!(samples
            .iter()
            .all(|sample| sample.max_latitude_delta_deg > 0.0));
        assert!(samples
            .iter()
            .all(|sample| sample.max_distance_delta_au > 0.0));
    }

    #[test]
    fn canonical_epoch_error_envelope_matches_the_public_sample_catalog() {
        let samples = canonical_epoch_samples();
        let body_evidence = canonical_epoch_body_evidence().expect("evidence should exist");
        let summary = canonical_epoch_evidence_summary().expect("summary should exist");

        assert_eq!(body_evidence.len(), samples.len());
        assert_eq!(summary.sample_count, samples.len());
        assert_eq!(
            summary.sample_bodies,
            samples
                .iter()
                .map(|sample| sample.body.clone())
                .collect::<Vec<_>>()
        );
        assert!(summary.within_interim_limits);
        assert!(body_evidence
            .iter()
            .all(|evidence| evidence.within_interim_limits));
        assert!(body_evidence
            .iter()
            .any(|evidence| evidence.source_kind == Vsop87BodySourceKind::GeneratedBinaryVsop87b));
        assert!(summary.max_longitude_delta_deg > 0.0);
        assert!(summary.max_latitude_delta_deg > 0.0);
        assert!(summary.max_distance_delta_au > 0.0);
        assert!(summary.mean_longitude_delta_deg > 0.0);
        assert!(summary.median_longitude_delta_deg > 0.0);
        assert!(summary.rms_longitude_delta_deg > 0.0);
        assert!(summary.percentile_longitude_delta_deg > 0.0);
        assert!(summary.mean_latitude_delta_deg > 0.0);
        assert!(summary.median_latitude_delta_deg > 0.0);
        assert!(summary.percentile_latitude_delta_deg > 0.0);
        assert!(summary.rms_latitude_delta_deg > 0.0);
        assert!(summary.mean_distance_delta_au > 0.0);
        assert!(summary.median_distance_delta_au > 0.0);
        assert!(summary.percentile_distance_delta_au > 0.0);
        assert!(summary.rms_distance_delta_au > 0.0);
        assert_eq!(summary.out_of_limit_count, 0);
        assert!(body_evidence
            .iter()
            .any(|evidence| evidence.body == summary.max_longitude_delta_body));
        assert!(body_evidence
            .iter()
            .any(|evidence| evidence.body == summary.max_latitude_delta_body));
        assert!(body_evidence
            .iter()
            .any(|evidence| evidence.body == summary.max_distance_delta_body));
        let max_longitude = body_evidence
            .iter()
            .find(|evidence| evidence.body == summary.max_longitude_delta_body)
            .expect("max longitude body should exist");
        let max_latitude = body_evidence
            .iter()
            .find(|evidence| evidence.body == summary.max_latitude_delta_body)
            .expect("max latitude body should exist");
        let max_distance = body_evidence
            .iter()
            .find(|evidence| evidence.body == summary.max_distance_delta_body)
            .expect("max distance body should exist");
        assert_eq!(
            summary.max_longitude_delta_source_kind,
            max_longitude.source_kind
        );
        assert_eq!(
            summary.max_longitude_delta_source_file,
            max_longitude.source_file
        );
        assert_eq!(
            summary.max_latitude_delta_source_kind,
            max_latitude.source_kind
        );
        assert_eq!(
            summary.max_latitude_delta_source_file,
            max_latitude.source_file
        );
        assert_eq!(
            summary.max_distance_delta_source_kind,
            max_distance.source_kind
        );
        assert_eq!(
            summary.max_distance_delta_source_file,
            max_distance.source_file
        );
    }

    #[test]
    fn source_specifications_document_variant_frames_units_and_range() {
        let specs = source_specifications();
        assert_eq!(specs.len(), 8);
        assert!(specs.iter().all(|spec| spec.variant == "VSOP87B"));
        assert!(specs
            .iter()
            .all(|spec| spec.frame == "J2000 ecliptic/equinox"));
        assert!(specs
            .iter()
            .all(|spec| spec.units == "degrees and astronomical units"));
        assert!(specs
            .iter()
            .any(|spec| spec.reduction.contains("solar reduction")));
        assert!(specs
            .iter()
            .all(|spec| spec.reduction.contains("geocentric")));
        assert!(specs.iter().all(|spec| {
            spec.truncation_policy
                == "generated binary coefficient table derived from vendored full source file"
        }));
        assert!(!specs
            .iter()
            .any(|spec| spec.truncation_policy == "vendored full source file"));
        assert!(specs.iter().all(|spec| spec
            .date_range
            .contains("full public source file; J2000 canonical reference sample")));
        assert!(specs
            .iter()
            .all(|spec| spec.transform_note.contains("mean-obliquity transform")));
        assert!(specs.iter().any(|spec| spec.source_file == "VSOP87B.nep"));
    }

    #[test]
    fn source_audit_manifest_tracks_all_vendored_inputs() {
        let audits = source_audits();
        let summary = source_audit_summary();

        assert_eq!(audits.len(), 8);
        assert_eq!(summary.source_count, 8);
        assert_eq!(summary.source_bodies, source_backed_body_order());
        assert_eq!(
            summary.source_files,
            source_specifications()
                .iter()
                .map(|spec| spec.source_file)
                .collect::<Vec<_>>()
        );
        assert_eq!(summary.vendored_full_file_count, 8);
        assert_eq!(summary.fingerprint_count, 8);
        assert!(summary.total_term_count > 0);
        assert!(summary.max_byte_length > 0);
        assert!(summary.max_line_count > 0);

        let mut fingerprints = audits
            .iter()
            .map(|audit| audit.fingerprint)
            .collect::<Vec<_>>();
        fingerprints.sort_unstable();
        fingerprints.dedup();
        assert_eq!(fingerprints.len(), audits.len());

        let earth = audits
            .iter()
            .find(|audit| audit.body == CelestialBody::Sun)
            .expect("Sun audit should exist");
        assert_eq!(earth.source_file, "VSOP87B.ear");
        assert_eq!(earth.term_count, 2_564);
    }

    #[test]
    fn source_audit_report_matches_the_backend_formatter() {
        let summary = source_audit_summary();
        assert_eq!(
            source_audit_summary_for_report(),
            "VSOP87 source audit: 8 source-backed bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune) across 8 source files (VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep); 8 vendored full-file inputs, 35080 total terms, max source size 949753 bytes / 7141 lines, 8 deterministic fingerprints"
        );
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            source_audit_summary_for_report(),
            format_source_audit_summary(&summary)
        );
    }

    #[test]
    fn generated_binary_audit_manifest_tracks_all_checked_in_blobs() {
        let audits = generated_binary_audits();
        let summary = generated_binary_audit_summary();

        assert_eq!(audits.len(), 8);
        assert_eq!(summary.blob_count, 8);
        assert_eq!(summary.source_file_count, 8);
        assert_eq!(summary.fingerprint_count, 8);
        assert_eq!(summary.source_bodies, source_backed_body_order());
        assert_eq!(
            summary.source_files,
            source_specifications()
                .iter()
                .map(|spec| spec.source_file)
                .collect::<Vec<_>>()
        );
        assert!(summary.total_byte_length > 0);
        assert!(summary.max_byte_length > 0);
        assert_eq!(
            audits
                .iter()
                .map(|audit| audit.source_file)
                .collect::<Vec<_>>(),
            summary.source_files
        );

        let mut fingerprints = audits
            .iter()
            .map(|audit| audit.fingerprint)
            .collect::<Vec<_>>();
        fingerprints.sort_unstable();
        fingerprints.dedup();
        assert_eq!(fingerprints.len(), audits.len());

        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            generated_binary_audit_summary_for_report(),
            format_generated_binary_audit_summary(&summary)
        );
        assert!(generated_binary_audit_summary_for_report().contains(
            "VSOP87 generated binary audit: 8 checked-in blobs across 8 source files (bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep)"
        ));
    }

    #[test]
    fn source_documentation_summary_tracks_catalog_counts() {
        let summary = source_documentation_summary();

        assert_eq!(summary.source_specification_count, 8);
        assert_eq!(summary.source_backed_profile_count, 8);
        assert_eq!(
            summary.source_backed_bodies,
            vec![
                CelestialBody::Sun,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        assert_eq!(
            summary.source_files,
            vec![
                "VSOP87B.ear",
                "VSOP87B.mer",
                "VSOP87B.ven",
                "VSOP87B.mar",
                "VSOP87B.jup",
                "VSOP87B.sat",
                "VSOP87B.ura",
                "VSOP87B.nep",
            ]
        );
        assert_eq!(
            summary.generated_binary_bodies,
            summary.source_backed_bodies
        );
        assert_eq!(
            summary.generated_binary_bodies,
            vec![
                CelestialBody::Sun,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        assert_eq!(
            summary.generated_binary_bodies,
            vec![
                CelestialBody::Sun,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        assert!(summary.vendored_full_file_bodies.is_empty());
        assert!(summary.truncated_bodies.is_empty());
        assert_eq!(summary.generated_binary_profile_count, 8);
        assert_eq!(summary.vendored_full_file_profile_count, 0);
        assert_eq!(summary.truncated_profile_count, 0);
        assert_eq!(summary.fallback_profile_count, 1);
        assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
        assert_eq!(
            summary.date_ranges,
            vec!["full public source file; J2000 canonical reference sample"]
        );
    }

    #[test]
    fn source_specification_summary_is_typed_and_reusable() {
        let specs = source_specifications();
        let first = &specs[0];
        let expected_joined = specs
            .iter()
            .map(format_source_specification)
            .collect::<Vec<_>>()
            .join(", ");

        assert_eq!(first.summary_line(), first.to_string());
        assert_eq!(format_source_specification(first), first.summary_line());
        assert!(first.summary_line().contains("body=Sun"));
        assert!(first.summary_line().contains("file=VSOP87B.ear"));
        assert!(first.summary_line().contains("variant=VSOP87B"));
        assert!(first
            .summary_line()
            .contains("date range=full public source file; J2000 canonical reference sample"));
        assert_eq!(format_source_specifications(&specs), expected_joined);
        assert_eq!(source_specifications_for_report(), expected_joined);
        assert!(source_specifications_for_report().contains("body=Neptune"));
    }

    #[test]
    fn source_documentation_health_summary_confirms_catalog_partitioning() {
        let documentation_summary = source_documentation_summary();
        let summary = source_documentation_health_summary();

        assert!(summary.consistent);
        assert!(summary.documentation_consistent);
        assert!(summary.issues.is_empty());
        assert_eq!(summary.source_specification_count, 8);
        assert_eq!(summary.source_file_count, 8);
        assert_eq!(
            summary.source_files,
            vec![
                "VSOP87B.ear",
                "VSOP87B.mer",
                "VSOP87B.ven",
                "VSOP87B.mar",
                "VSOP87B.jup",
                "VSOP87B.sat",
                "VSOP87B.ura",
                "VSOP87B.nep",
            ]
        );
        assert_eq!(summary.source_backed_profile_count, 8);
        assert_eq!(summary.body_profile_count, 9);
        assert_eq!(
            summary.generated_binary_bodies,
            vec![
                CelestialBody::Sun,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        assert!(summary.vendored_full_file_bodies.is_empty());
        assert!(summary.truncated_bodies.is_empty());
        assert_eq!(summary.generated_binary_profile_count, 8);
        assert_eq!(summary.vendored_full_file_profile_count, 0);
        assert_eq!(summary.truncated_profile_count, 0);
        assert_eq!(summary.fallback_profile_count, 1);
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(
            source_documentation_health_summary_for_report(),
            summary.summary_line()
        );
        assert_eq!(
            summary.source_backed_bodies,
            vec![
                CelestialBody::Sun,
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        assert_eq!(
            summary.source_backed_partition_bodies,
            summary.source_backed_bodies
        );
        assert_eq!(
            source_documentation_partition_bodies(&documentation_summary),
            documentation_summary.source_backed_bodies
        );
        assert_eq!(summary.fallback_bodies, vec![CelestialBody::Pluto]);
        assert_eq!(
            format_source_documentation_health_summary(&summary),
            "VSOP87 source documentation health: ok (8 source specs, 8 source files, 8 source-backed profiles, 9 body profiles; 8 generated binary profiles (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep; source-backed order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; source-backed partition order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune; fallback order: Pluto; documented fields: variant, coordinate family, frame, units, reduction, transform note, truncation policy, and date range)"
        );
        assert_eq!(
            source_documentation_health_summary_for_report(),
            format_source_documentation_health_summary(&summary)
        );
    }

    #[test]
    fn source_documentation_health_summary_lists_issues_when_inconsistent() {
        let summary = Vsop87SourceDocumentationHealthSummary {
            consistent: false,
            documentation_consistent: false,
            issues: vec![
                "source specification/file count mismatch",
                "documented field mismatch",
            ],
            source_specification_count: 1,
            source_file_count: 2,
            source_files: vec!["VSOP87B.ear"],
            source_backed_profile_count: 1,
            source_backed_bodies: vec![CelestialBody::Sun],
            source_backed_partition_bodies: vec![CelestialBody::Sun],
            generated_binary_bodies: vec![CelestialBody::Sun],
            vendored_full_file_bodies: vec![],
            truncated_bodies: vec![],
            body_profile_count: 2,
            generated_binary_profile_count: 1,
            vendored_full_file_profile_count: 0,
            truncated_profile_count: 0,
            fallback_profile_count: 1,
            fallback_bodies: vec![CelestialBody::Pluto],
        };

        assert_eq!(
            format_source_documentation_health_summary(&summary),
            "VSOP87 source documentation health: needs attention (1 source specs, 2 source files, 1 source-backed profiles, 2 body profiles; 1 generated binary profiles (Sun), 0 vendored full-file profiles (none), 0 truncated profiles (none), 1 fallback profiles (Pluto); source files: VSOP87B.ear; source-backed order: Sun; source-backed partition order: Sun; fallback order: Pluto; documented fields: needs attention); issues: source specification/file count mismatch, documented field mismatch"
        );
    }

    #[test]
    fn source_documentation_health_issues_detect_partition_order_drift() {
        let mut summary = source_documentation_summary();
        summary.source_backed_bodies.reverse();
        let source_specs = source_specifications();

        let issues = source_documentation_health_issues(
            &summary,
            &source_specs,
            body_catalog_entries().len(),
            summary.source_files.len(),
        );

        assert!(issues.contains(&"source-backed body order mismatch"));
    }

    #[test]
    fn request_policy_summary_tracks_the_public_backend_posture() {
        let policy = vsop87_request_policy();

        assert_eq!(
            policy.summary_line(),
            "frames=Ecliptic, Equatorial; time scales=TT, TDB; zodiac modes=Tropical; apparentness=Mean; topocentric observer=false"
        );
        assert_eq!(
            policy.supported_frames,
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial]
        );
        assert_eq!(
            policy.supported_time_scales,
            &[TimeScale::Tt, TimeScale::Tdb]
        );
        assert_eq!(policy.supported_zodiac_modes, &[ZodiacMode::Tropical]);
        assert_eq!(policy.supported_apparentness, &[Apparentness::Mean]);
        assert!(!policy.supports_topocentric_observer);
    }

    #[test]
    fn source_kind_display_labels_match_the_release_facing_labels() {
        let cases = [
            (
                Vsop87BodySourceKind::TruncatedVsop87b,
                "truncated VSOP87B slice",
            ),
            (
                Vsop87BodySourceKind::VendoredVsop87b,
                "vendored full-file VSOP87B",
            ),
            (
                Vsop87BodySourceKind::GeneratedBinaryVsop87b,
                "generated binary VSOP87B",
            ),
            (
                Vsop87BodySourceKind::MeanOrbitalElements,
                "mean orbital elements fallback",
            ),
        ];

        for (kind, expected) in cases {
            assert_eq!(kind.label(), expected);
            assert_eq!(kind.to_string(), expected);
        }
    }

    #[test]
    fn source_documentation_report_matches_the_backend_formatter() {
        let summary = source_documentation_summary();
        let rendered = source_documentation_summary_for_report();
        assert_eq!(rendered, format_source_documentation_summary(&summary));
        assert_eq!(summary.summary_line(), rendered);
        assert_eq!(summary.to_string(), rendered);
        assert!(rendered.contains("source files: VSOP87B.ear, VSOP87B.mer, VSOP87B.ven, VSOP87B.mar, VSOP87B.jup, VSOP87B.sat, VSOP87B.ura, VSOP87B.nep"));
        assert!(rendered.contains("source-backed breakdown: 8 generated binary bodies (Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune), 0 vendored full-file bodies (none), 0 truncated slice bodies (none)"));
    }

    #[test]
    fn source_body_evidence_summary_matches_the_canonical_body_evidence() {
        let evidence = canonical_epoch_body_evidence().expect("evidence should exist");
        let summary = source_body_evidence_summary().expect("summary should exist");

        assert_eq!(summary.sample_count, evidence.len());
        assert_eq!(
            summary.sample_bodies,
            evidence
                .iter()
                .map(|row| row.body.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(summary.within_interim_limits_count, evidence.len());
        assert_eq!(summary.vendored_full_file_count, 0);
        assert_eq!(summary.generated_binary_count, evidence.len());
        assert_eq!(summary.truncated_count, 0);
        assert_eq!(summary.outside_interim_limit_count, 0);
        assert!(summary.outside_interim_limit_bodies.is_empty());
        assert!(evidence.iter().all(|row| row.within_interim_limits));
        assert!(format_source_body_evidence_summary(&summary).contains(
            "source-backed body order: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"
        ));
    }

    #[test]
    fn source_body_evidence_report_matches_the_backend_formatter() {
        let summary = source_body_evidence_summary().expect("summary should exist");
        assert_eq!(
            source_body_evidence_summary_for_report(),
            format_source_body_evidence_summary(&summary)
        );
        assert_eq!(
            summary.summary_line(),
            source_body_evidence_summary_for_report()
        );
        assert_eq!(
            summary.to_string(),
            source_body_evidence_summary_for_report()
        );
    }

    #[test]
    fn source_body_class_evidence_report_matches_the_backend_formatter() {
        let summary = source_body_class_evidence_summary().expect("summary should exist");
        assert_eq!(summary.len(), 2);
        assert_eq!(summary[0].class, Vsop87SourceBodyClass::Luminary);
        assert_eq!(summary[0].sample_count, 1);
        assert_eq!(summary[0].sample_bodies, vec![CelestialBody::Sun]);
        assert_eq!(summary[1].class, Vsop87SourceBodyClass::MajorPlanet);
        assert_eq!(summary[1].sample_count, 7);
        assert_eq!(
            summary[1].sample_bodies,
            vec![
                CelestialBody::Mercury,
                CelestialBody::Venus,
                CelestialBody::Mars,
                CelestialBody::Jupiter,
                CelestialBody::Saturn,
                CelestialBody::Uranus,
                CelestialBody::Neptune,
            ]
        );
        let rendered = source_body_class_evidence_summary_for_report();
        assert_eq!(
            rendered,
            format_source_body_class_evidence_summary(&summary)
        );
        assert_eq!(summary[0].summary_line(), summary[0].to_string());
        assert_eq!(summary[1].summary_line(), summary[1].to_string());
        assert!(rendered.contains("Luminary: samples=1, bodies: Sun"));
        assert!(rendered.contains("median Δlon="));
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("median Δlat="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("median Δdist="));
        assert!(rendered.contains("p95 Δdist="));
        assert!(rendered.contains("Major planets: samples=7, bodies: Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    }

    #[test]
    fn canonical_evidence_report_matches_the_backend_formatter() {
        let summary = canonical_epoch_evidence_summary().expect("summary should exist");
        let rendered = canonical_epoch_evidence_summary_for_report();
        assert_eq!(rendered, format_canonical_epoch_evidence_summary(&summary));
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("p95 Δlon="));
        assert!(rendered.contains("p95 Δlat="));
        assert!(rendered.contains("p95 Δdist="));
    }

    #[test]
    fn canonical_evidence_outlier_note_reports_the_current_interim_status() {
        assert_eq!(
            canonical_epoch_outlier_note_for_report(),
            "VSOP87 canonical J2000 interim outliers: none"
        );
    }

    #[test]
    fn frame_treatment_summary_has_a_displayable_summary_line() {
        let summary = frame_treatment_summary_details();

        assert_eq!(summary.to_string(), summary.summary_line());
        assert_eq!(
            summary.summary_line(),
            "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
        );
        assert_eq!(frame_treatment_summary(), summary.summary_line());
        assert!(summary.summary_line().contains("mean-obliquity transform"));
    }

    #[test]
    fn canonical_equatorial_evidence_report_matches_the_backend_formatter() {
        let summary =
            canonical_epoch_equatorial_evidence_summary().expect("equatorial summary should exist");
        let rendered = canonical_epoch_equatorial_evidence_summary_for_report();
        assert_eq!(
            rendered,
            format_canonical_equatorial_evidence_summary(&summary)
        );
        assert_eq!(summary.summary_line(), summary.to_string());
        assert_eq!(rendered, summary.summary_line());
        assert!(rendered.contains("p95 Δra="));
        assert!(rendered.contains("p95 Δdec="));
        assert!(rendered.contains("p95 Δdist="));
    }

    #[test]
    fn canonical_evidence_report_lists_the_measured_bodies() {
        let rendered = canonical_epoch_evidence_summary_for_report();
        assert!(rendered
            .contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    }

    #[test]
    fn canonical_equatorial_evidence_report_lists_the_measured_bodies() {
        let rendered = canonical_epoch_equatorial_evidence_summary_for_report();
        assert!(rendered
            .contains("bodies: Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune"));
    }

    #[test]
    fn regenerated_binary_tables_match_the_checked_in_artifacts() {
        for spec in source_specifications() {
            let regenerated = generated_vsop87b_table_bytes_for_source_file(spec.source_file)
                .expect("source-backed tables should regenerate");
            let expected =
                checked_in_generated_vsop87b_table_bytes_for_source_file(spec.source_file)
                    .expect("supported source files should have a checked-in generated blob");
            assert_eq!(
                regenerated.as_slice(),
                expected,
                "regenerated blob should match {}",
                spec.source_file
            );
        }
    }

    #[test]
    fn checked_in_generated_tables_cover_the_supported_source_file_set() {
        for source_file in supported_source_files() {
            assert!(
                checked_in_generated_vsop87b_table_bytes_for_source_file(source_file).is_some(),
                "supported source file {source_file} should have a checked-in generated blob"
            );
        }
        assert!(checked_in_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu").is_none());
    }

    #[test]
    fn supported_source_files_are_exposed_for_reproducibility_tooling() {
        assert_eq!(
            supported_source_files(),
            source_documentation_summary().source_files
        );
    }

    #[test]
    fn source_backed_and_fallback_body_profiles_are_exposed_for_reproducibility_tooling() {
        let source_backed_profiles = source_backed_body_profiles();
        let fallback_profiles = fallback_body_profiles();
        let summary = source_documentation_summary();

        assert_eq!(
            source_backed_profiles.len(),
            summary.source_backed_profile_count
        );
        assert_eq!(fallback_profiles.len(), summary.fallback_profile_count);
        assert_eq!(source_backed_body_order(), summary.source_backed_bodies);
        assert_eq!(
            source_backed_profiles
                .iter()
                .map(|profile| profile.body.clone())
                .collect::<Vec<_>>(),
            summary.source_backed_bodies
        );
        assert_eq!(
            fallback_profiles
                .iter()
                .map(|profile| profile.body.clone())
                .collect::<Vec<_>>(),
            summary.fallback_bodies
        );
        assert!(fallback_profiles
            .iter()
            .all(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements));
        assert!(source_backed_profiles
            .iter()
            .all(|profile| profile.kind != Vsop87BodySourceKind::MeanOrbitalElements));
        assert_eq!(
            source_backed_profiles.len() + fallback_profiles.len(),
            body_source_profiles().len()
        );
    }

    #[test]
    fn regeneration_helper_reports_unknown_source_files_explicitly() {
        let error = try_generated_vsop87b_table_bytes_for_source_file("VSOP87B.plu")
            .expect_err("unsupported source files should be rejected");

        assert_eq!(
            error,
            Vsop87TableGenerationError::UnknownSourceFile {
                source_file: "VSOP87B.plu".to_string(),
                supported_source_files: vec![
                    "VSOP87B.ear",
                    "VSOP87B.mer",
                    "VSOP87B.ven",
                    "VSOP87B.mar",
                    "VSOP87B.jup",
                    "VSOP87B.sat",
                    "VSOP87B.ura",
                    "VSOP87B.nep",
                ],
            }
        );
        assert!(error
            .to_string()
            .contains("no vendored VSOP87B source text found for VSOP87B.plu"));
    }

    #[test]
    fn unified_body_catalog_keeps_profiles_specs_and_samples_aligned() {
        let catalog = body_catalog_entries();
        assert_eq!(catalog.len(), Vsop87Backend::supported_bodies().len());

        let source_backed = catalog
            .iter()
            .filter(|entry| {
                matches!(
                    entry.source_profile.kind,
                    Vsop87BodySourceKind::TruncatedVsop87b
                        | Vsop87BodySourceKind::VendoredVsop87b
                        | Vsop87BodySourceKind::GeneratedBinaryVsop87b
                )
            })
            .count();
        let fallback = catalog
            .iter()
            .filter(|entry| entry.source_profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
            .count();
        assert_eq!(source_backed, 8);
        assert_eq!(fallback, 1);

        let pluto = catalog
            .iter()
            .find(|entry| entry.source_profile.body == CelestialBody::Pluto)
            .expect("Pluto entry should exist");
        assert!(pluto.source_specification.is_none());
        assert!(pluto.canonical_sample.is_none());

        let sun = catalog
            .iter()
            .find(|entry| entry.source_profile.body == CelestialBody::Sun)
            .expect("Sun entry should exist");
        assert_eq!(
            sun.source_profile.kind,
            Vsop87BodySourceKind::GeneratedBinaryVsop87b
        );
        assert!(sun.source_specification.is_some());
        assert!(sun.canonical_sample.is_some());
    }

    #[test]
    fn signed_longitude_delta_wraps_across_zero_aries() {
        assert_eq!(signed_longitude_delta_degrees(359.5, 0.5), 1.0);
        assert_eq!(signed_longitude_delta_degrees(0.5, 359.5), -1.0);
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

    fn assert_degrees_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = signed_longitude_delta_degrees(expected, actual).abs();
        assert!(
            delta <= tolerance,
            "expected {actual}° to be within {tolerance}° of {expected}°; delta was {delta}°"
        );
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {actual} to be within {tolerance} of {expected}; delta was {delta}"
        );
    }
}
