//! Formula-based planetary backend boundary built around VSOP87-style series
//! evaluation, low-precision orbital elements, and geocentric coordinate
//! transforms.
//!
//! This crate now provides a working pure-Rust algorithmic backend for the Sun
//! and major planets. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus,
//! and Neptune paths evaluate public IMCCE VSOP87B sources (heliocentric
//! spherical variables, J2000 ecliptic/equinox) transformed to geocentric
//! chart-facing coordinates. The Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune paths
//! now use generated binary tables derived from their vendored source files.
//! A maintainer-facing regeneration helper and `regenerate-vsop87b-tables`
//! binary keep those checked-in blobs reproducible from the public source text.
//! The backend accepts both TT and TDB requests as dynamical-time inputs and
//! still rejects UT-based requests explicitly. Pluto still uses compact
//! Keplerian orbital elements,
//! a geocentric reduction step, and central-difference motion estimates so the
//! workspace has an end-to-end tropical chart path while the remaining
//! generated VSOP87 tables and Pluto-specific source selection are added
//! incrementally.

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
    validate_observer_policy, validate_request_policy, AccuracyClass, Apparentness,
    BackendCapabilities, BackendFamily, BackendId, BackendMetadata, BackendProvenance,
    EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest, EphemerisResult,
    QualityAnnotation,
};
use pleiades_types::{
    Angle, CelestialBody, CoordinateFrame, EclipticCoordinates, EquatorialCoordinates, Instant,
    Latitude, Longitude, Motion, TimeRange, TimeScale, ZodiacMode,
};
use std::sync::OnceLock;

use crate::vsop87b_earth::{generated_vsop87b_table_bytes, parse_vsop87b_tables};

const PACKAGE_NAME: &str = "pleiades-vsop87";
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
    /// complete VSOP87 coefficient path is still pending.
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
/// sync as the backend moves from vendored source files toward generated tables.
pub fn body_source_profiles() -> Vec<Vsop87BodySource> {
    body_catalog_entries()
        .iter()
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

/// Summary metrics for the current VSOP87 source-documentation catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceDocumentationSummary {
    /// Number of source specifications described by the catalog.
    pub source_specification_count: usize,
    /// Number of source-backed body profiles described by the catalog.
    pub source_backed_profile_count: usize,
    /// Number of vendored full-file body profiles.
    pub vendored_full_file_profile_count: usize,
    /// Number of generated-binary body profiles.
    pub generated_binary_profile_count: usize,
    /// Number of truncated-slice body profiles.
    pub truncated_profile_count: usize,
    /// Number of fallback mean-element body profiles.
    pub fallback_profile_count: usize,
}

/// Canonical J2000 reference samples for the source-backed VSOP87B paths.
///
/// These values are the same full-file public IMCCE VSOP87B reference points
/// exercised by the backend regression tests. The validation tooling uses them
/// to render measured deltas against the checked-in source-backed coefficient
/// paths while the complete generated-table pipeline is still pending.
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
    /// Whether the body is within the current interim limits.
    pub within_interim_limits: bool,
}

/// Public summary of the canonical J2000 error envelope.
#[derive(Clone, Debug, PartialEq)]
pub struct Vsop87CanonicalEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Body with the maximum absolute geocentric longitude delta.
    pub max_longitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum longitude delta body.
    pub max_longitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum longitude delta body.
    pub max_longitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric longitude delta in degrees.
    pub max_longitude_delta_deg: f64,
    /// Body with the maximum absolute geocentric latitude delta.
    pub max_latitude_delta_body: CelestialBody,
    /// Calculation family behind the maximum latitude delta body.
    pub max_latitude_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum latitude delta body.
    pub max_latitude_delta_source_file: &'static str,
    /// Maximum absolute geocentric latitude delta in degrees.
    pub max_latitude_delta_deg: f64,
    /// Body with the maximum absolute geocentric distance delta.
    pub max_distance_delta_body: CelestialBody,
    /// Calculation family behind the maximum distance delta body.
    pub max_distance_delta_source_kind: Vsop87BodySourceKind,
    /// Public source file behind the maximum distance delta body.
    pub max_distance_delta_source_file: &'static str,
    /// Maximum absolute geocentric distance delta in astronomical units.
    pub max_distance_delta_au: f64,
    /// Whether every measured body remained within the interim limits.
    pub within_interim_limits: bool,
}

#[derive(Clone, Debug)]
struct Vsop87BodyCatalogEntry {
    source_profile: Vsop87BodySource,
    source_specification: Option<Vsop87SourceSpecification>,
    canonical_sample: Option<Vsop87CanonicalEpochSample>,
}

static BODY_CATALOG: OnceLock<Vec<Vsop87BodyCatalogEntry>> = OnceLock::new();
static SOURCE_AUDITS: OnceLock<Vec<Vsop87SourceAudit>> = OnceLock::new();

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
    let tables = parse_vsop87b_tables(source);
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
                    "compact mean orbital elements fallback pending source-backed VSOP87 coefficient tables",
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

/// Returns the current frame-treatment summary for VSOP87-backed results.
pub fn frame_treatment_summary() -> &'static str {
    "VSOP87 frame treatment: J2000 ecliptic/equinox inputs; equatorial coordinates are derived with a mean-obliquity transform"
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
    Vsop87SourceAuditSummary {
        source_count: audits.len(),
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

/// Returns a summary of the current VSOP87 source-documentation catalog.
pub fn source_documentation_summary() -> Vsop87SourceDocumentationSummary {
    let source_specs = source_specifications();
    let source_backed_profiles = body_source_profiles();

    Vsop87SourceDocumentationSummary {
        source_specification_count: source_specs.len(),
        source_backed_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| {
                matches!(
                    profile.kind,
                    Vsop87BodySourceKind::TruncatedVsop87b
                        | Vsop87BodySourceKind::VendoredVsop87b
                        | Vsop87BodySourceKind::GeneratedBinaryVsop87b
                )
            })
            .count(),
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
        fallback_profile_count: source_backed_profiles
            .iter()
            .filter(|profile| profile.kind == Vsop87BodySourceKind::MeanOrbitalElements)
            .count(),
    }
}

/// Backend-owned summary of the canonical VSOP87 body evidence envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87SourceBodyEvidenceSummary {
    /// Number of canonical samples measured.
    pub sample_count: usize,
    /// Number of samples within the interim limits.
    pub within_interim_limits_count: usize,
    /// Number of vendored full-file source-backed samples.
    pub vendored_full_file_count: usize,
    /// Number of generated-binary source-backed samples.
    pub generated_binary_count: usize,
    /// Number of truncated-slice source-backed samples.
    pub truncated_count: usize,
    /// Bodies outside the current interim limits.
    pub outside_interim_limit_bodies: Vec<CelestialBody>,
}

/// Returns a backend-owned summary of the canonical VSOP87 body evidence.
pub fn source_body_evidence_summary() -> Option<Vsop87SourceBodyEvidenceSummary> {
    let evidence = canonical_epoch_body_evidence()?;
    Some(Vsop87SourceBodyEvidenceSummary {
        sample_count: evidence.len(),
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
        outside_interim_limit_bodies: evidence
            .into_iter()
            .filter(|row| !row.within_interim_limits)
            .map(|row| row.body)
            .collect(),
    })
}

/// Regenerates the checked-in binary VSOP87B coefficient blob for a vendored
/// public source file.
///
/// This helper is used by the maintainer-facing regeneration tool and the
/// reproducibility tests to keep the checked-in `.bin` files aligned with the
/// vendored public IMCCE/CELMECH source inputs.
pub fn generated_vsop87b_table_bytes_for_source_file(source_file: &str) -> Option<Vec<u8>> {
    source_text_for_file(source_file).map(generated_vsop87b_table_bytes)
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
            within_interim_limits,
        });
    }

    Some(evidence)
}

/// Returns the canonical J2000 error envelope summary used by release-facing
/// validation reports.
pub fn canonical_epoch_evidence_summary() -> Option<Vsop87CanonicalEvidenceSummary> {
    let body_evidence = canonical_epoch_body_evidence()?;
    let first = body_evidence.first()?;
    let mut sample_count = 0usize;
    let mut max_longitude_delta_body = first.body.clone();
    let mut max_longitude_delta_source_kind = first.source_kind;
    let mut max_longitude_delta_source_file = first.source_file;
    let mut max_longitude_delta_deg = first.longitude_delta_deg;
    let mut max_latitude_delta_body = first.body.clone();
    let mut max_latitude_delta_source_kind = first.source_kind;
    let mut max_latitude_delta_source_file = first.source_file;
    let mut max_latitude_delta_deg = first.latitude_delta_deg;
    let mut max_distance_delta_body = first.body.clone();
    let mut max_distance_delta_source_kind = first.source_kind;
    let mut max_distance_delta_source_file = first.source_file;
    let mut max_distance_delta_au = first.distance_delta_au;
    let mut within_interim_limits = true;

    for evidence in &body_evidence {
        sample_count += 1;
        if evidence.longitude_delta_deg >= max_longitude_delta_deg {
            max_longitude_delta_deg = evidence.longitude_delta_deg;
            max_longitude_delta_body = evidence.body.clone();
            max_longitude_delta_source_kind = evidence.source_kind;
            max_longitude_delta_source_file = evidence.source_file;
        }
        if evidence.latitude_delta_deg >= max_latitude_delta_deg {
            max_latitude_delta_deg = evidence.latitude_delta_deg;
            max_latitude_delta_body = evidence.body.clone();
            max_latitude_delta_source_kind = evidence.source_kind;
            max_latitude_delta_source_file = evidence.source_file;
        }
        if evidence.distance_delta_au >= max_distance_delta_au {
            max_distance_delta_au = evidence.distance_delta_au;
            max_distance_delta_body = evidence.body.clone();
            max_distance_delta_source_kind = evidence.source_kind;
            max_distance_delta_source_file = evidence.source_file;
        }
        within_interim_limits &= evidence.within_interim_limits;
    }

    Some(Vsop87CanonicalEvidenceSummary {
        sample_count,
        max_longitude_delta_body,
        max_longitude_delta_source_kind,
        max_longitude_delta_source_file,
        max_longitude_delta_deg,
        max_latitude_delta_body,
        max_latitude_delta_source_kind,
        max_latitude_delta_source_file,
        max_latitude_delta_deg,
        max_distance_delta_body,
        max_distance_delta_source_kind,
        max_distance_delta_source_file,
        max_distance_delta_au,
        within_interim_limits,
    })
}

fn source_kind_for_body(body: CelestialBody) -> Option<Vsop87BodySourceKind> {
    body_catalog_entries()
        .iter()
        .find(|entry| entry.source_profile.body == body)
        .map(|entry| entry.source_profile.kind)
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
        Self::to_ecliptic(coords)
            .to_equatorial(Angle::from_degrees(Self::mean_obliquity_degrees(instant)))
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
                        "Paul Schlyter-style mean orbital elements for planets not yet backed by VSOP87 coefficient tables".to_string(),
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
        if req.zodiac_mode != ZodiacMode::Tropical {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "the VSOP87 MVP backend currently exposes tropical coordinates only",
            ));
        }

        validate_request_policy(
            req,
            "the VSOP87 MVP backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            false,
        )?;

        validate_observer_policy(req, "the VSOP87 MVP backend", false)?;

        let days = Self::days_since_j2000(req.instant);
        let geocentric = Self::geocentric_coordinates(req.body.clone(), days).ok_or_else(|| {
            EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "requested body is not implemented in the VSOP87 MVP backend",
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
    fn source_documentation_summary_tracks_catalog_counts() {
        let summary = source_documentation_summary();

        assert_eq!(summary.source_specification_count, 8);
        assert_eq!(summary.source_backed_profile_count, 8);
        assert_eq!(summary.generated_binary_profile_count, 8);
        assert_eq!(summary.vendored_full_file_profile_count, 0);
        assert_eq!(summary.truncated_profile_count, 0);
        assert_eq!(summary.fallback_profile_count, 1);
    }

    #[test]
    fn source_body_evidence_summary_matches_the_canonical_body_evidence() {
        let evidence = canonical_epoch_body_evidence().expect("evidence should exist");
        let summary = source_body_evidence_summary().expect("summary should exist");

        assert_eq!(summary.sample_count, evidence.len());
        assert_eq!(summary.within_interim_limits_count, evidence.len());
        assert_eq!(summary.vendored_full_file_count, 0);
        assert_eq!(summary.generated_binary_count, evidence.len());
        assert_eq!(summary.truncated_count, 0);
        assert!(summary.outside_interim_limit_bodies.is_empty());
        assert!(evidence.iter().all(|row| row.within_interim_limits));
    }

    #[test]
    fn regenerated_binary_tables_match_the_checked_in_artifacts() {
        for spec in source_specifications() {
            let regenerated = generated_vsop87b_table_bytes_for_source_file(spec.source_file)
                .expect("source-backed tables should regenerate");
            let expected = match spec.source_file {
                "VSOP87B.ear" => include_bytes!("../data/VSOP87B.ear.bin").as_slice(),
                "VSOP87B.mer" => include_bytes!("../data/VSOP87B.mer.bin").as_slice(),
                "VSOP87B.ven" => include_bytes!("../data/VSOP87B.ven.bin").as_slice(),
                "VSOP87B.mar" => include_bytes!("../data/VSOP87B.mar.bin").as_slice(),
                "VSOP87B.jup" => include_bytes!("../data/VSOP87B.jup.bin").as_slice(),
                "VSOP87B.sat" => include_bytes!("../data/VSOP87B.sat.bin").as_slice(),
                "VSOP87B.ura" => include_bytes!("../data/VSOP87B.ura.bin").as_slice(),
                "VSOP87B.nep" => include_bytes!("../data/VSOP87B.nep.bin").as_slice(),
                other => panic!("unexpected VSOP87B source file {other}"),
            };
            assert_eq!(
                regenerated.as_slice(),
                expected,
                "regenerated blob should match {}",
                spec.source_file
            );
        }
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
        let mut request = EphemerisRequest::new(
            body,
            Instant::new(pleiades_types::JulianDay::from_days(J2000), TimeScale::Tt),
        );
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
