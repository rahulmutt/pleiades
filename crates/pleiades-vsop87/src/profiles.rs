use crate::tables::vsop87b_earth::parse_vsop87b_tables;
use crate::Vsop87CanonicalEpochSample;
use crate::Vsop87SourceSpecification;
use pleiades_backend::{AccuracyClass, Apparentness};
use pleiades_types::{CelestialBody, CoordinateFrame, TimeScale, ZodiacMode};
use std::fmt;
use std::sync::OnceLock;

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
    /// remaining Pluto-specific source path is modeled separately as an
    /// explicit special case.
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
/// [`crate::Vsop87Backend`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vsop87BodySource {
    /// Body covered by this source profile.
    pub body: CelestialBody,
    /// Calculation family used for the heliocentric or geocentric channel.
    pub kind: Vsop87BodySourceKind,
    /// Human-readable provenance detail for this body's calculation path.
    pub provenance: &'static str,
    /// Series-fidelity class of this body's checked-in path: `Exact` means the
    /// generated table reproduces the full, untruncated VSOP87B series for the
    /// body, while `Approximate` marks the mean-element Pluto fallback. This
    /// describes source-reproduction fidelity, not observational position
    /// accuracy: the backend's chart-facing `BodyClaim` for these source-backed
    /// planets is still `Moderate`/constrained (see
    /// [`crate::Vsop87Backend`]), and results remain mean, J2000 ecliptic.
    pub accuracy: AccuracyClass,
}

/// Validation errors for a VSOP87 body source profile that drifted from the
/// current catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Vsop87BodySourceValidationError {
    /// The catalog no longer contains the body.
    UnknownBody {
        /// Body that is absent from the current catalog.
        body: CelestialBody,
    },
    /// The provenance text is blank.
    BlankProvenance {
        /// Body whose profile carries empty provenance text.
        body: CelestialBody,
    },
    /// The provenance text carries surrounding whitespace.
    WhitespacePaddedProvenance {
        /// Body whose provenance text has leading or trailing whitespace.
        body: CelestialBody,
    },
    /// The declared source family no longer matches the catalog.
    SourceKindMismatch {
        /// Body whose source-kind declaration drifted from the catalog.
        body: CelestialBody,
        /// Source kind the current catalog expects for the body.
        expected: Vsop87BodySourceKind,
        /// Source kind found on the drifted profile.
        found: Vsop87BodySourceKind,
    },
}

impl fmt::Display for Vsop87BodySourceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownBody { body } => {
                write!(
                    f,
                    "the VSOP87 body source for {body} is no longer present in the catalog"
                )
            }
            Self::BlankProvenance { body } => {
                write!(f, "the VSOP87 body source for {body} has blank provenance")
            }
            Self::WhitespacePaddedProvenance { body } => write!(
                f,
                "the VSOP87 body source for {body} has surrounding whitespace in its provenance"
            ),
            Self::SourceKindMismatch {
                body,
                expected,
                found,
            } => write!(
                f,
                "the VSOP87 body source for {body} expects kind {expected} but found {found}"
            ),
        }
    }
}

impl std::error::Error for Vsop87BodySourceValidationError {}

impl Vsop87BodySource {
    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        format!(
            "{}: kind={}, accuracy={}, {}",
            self.body, self.kind, self.accuracy, self.provenance
        )
    }

    /// Returns `Ok(())` when the source profile still matches the current
    /// catalog.
    pub fn validate(&self) -> Result<(), Vsop87BodySourceValidationError> {
        if self.provenance.trim().is_empty() {
            return Err(Vsop87BodySourceValidationError::BlankProvenance {
                body: self.body.clone(),
            });
        }
        if self.provenance.trim() != self.provenance {
            return Err(
                Vsop87BodySourceValidationError::WhitespacePaddedProvenance {
                    body: self.body.clone(),
                },
            );
        }

        let expected = source_kind_for_body(self.body.clone()).ok_or(
            Vsop87BodySourceValidationError::UnknownBody {
                body: self.body.clone(),
            },
        )?;
        if expected != self.kind {
            return Err(Vsop87BodySourceValidationError::SourceKindMismatch {
                body: self.body.clone(),
                expected,
                found: self.kind,
            });
        }

        Ok(())
    }
}

impl fmt::Display for Vsop87BodySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the per-body source profiles used by [`crate::Vsop87Backend`].
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

/// Returns the source-backed VSOP87 body profiles used by [`crate::Vsop87Backend`].
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

/// Returns the fallback VSOP87 body profiles used by [`crate::Vsop87Backend`].
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

#[derive(Clone, Debug)]
pub(crate) struct Vsop87BodyCatalogEntry {
    pub(crate) source_profile: Vsop87BodySource,
    pub(crate) source_specification: Option<Vsop87SourceSpecification>,
    pub(crate) canonical_sample: Option<Vsop87CanonicalEpochSample>,
}

static BODY_CATALOG: OnceLock<Vec<Vsop87BodyCatalogEntry>> = OnceLock::new();

pub(crate) fn source_text_for_file(source_file: &str) -> Option<&'static str> {
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

pub(crate) fn count_vsop87_terms(source: &str) -> usize {
    let tables = parse_vsop87b_tables(source).expect("known VSOP87 source file should parse");
    tables
        .longitude
        .iter()
        .chain(tables.latitude.iter())
        .chain(tables.radius.iter())
        .map(Vec::len)
        .sum()
}

pub(crate) fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub(crate) fn join_display<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn format_coordinate_frames(frames: &[CoordinateFrame]) -> String {
    join_display(frames)
}

pub(crate) fn format_time_scales(time_scales: &[TimeScale]) -> String {
    join_display(time_scales)
}

pub(crate) fn format_zodiac_modes(zodiac_modes: &[ZodiacMode]) -> String {
    join_display(zodiac_modes)
}

pub(crate) fn format_apparentness_modes(modes: &[Apparentness]) -> String {
    join_display(modes)
}

pub(crate) fn body_catalog_entries() -> &'static [Vsop87BodyCatalogEntry] {
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
                    "current approximate mean-element fallback special case until a Pluto-specific source path is selected",
                    AccuracyClass::Approximate,
                ),
                source_specification: None,
                canonical_sample: None,
            },
        ]
    })
}

pub(crate) fn body_catalog_entry_for_body(body: &CelestialBody) -> Option<&Vsop87BodyCatalogEntry> {
    body_catalog_entries()
        .iter()
        .find(|entry| entry.source_profile.body == *body)
}

pub(crate) fn source_kind_for_body(body: CelestialBody) -> Option<Vsop87BodySourceKind> {
    body_catalog_entry_for_body(&body).map(|entry| entry.source_profile.kind)
}

pub(crate) fn source_file_for_body(body: &CelestialBody) -> Option<&'static str> {
    body_catalog_entry_for_body(body)
        .and_then(|entry| entry.source_specification.as_ref())
        .map(|spec| spec.source_file)
}
