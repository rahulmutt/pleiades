//! Lookup and computation functions for the ayanamsa catalog.

use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay};

use crate::catalog::{BASELINE_AYANAMSAS, BUILT_IN_AYANAMSAS, RELEASE_AYANAMSAS};
use crate::model::{
    AyanamsaCatalogValidationError, AyanamsaCatalogValidationSummary, AyanamsaDescriptor,
    AyanamsaMetadataCoverage, AyanamsaProvenanceSummary,
};

/// Returns the baseline built-in ayanamsa catalog.
pub const fn baseline_ayanamsas() -> &'static [AyanamsaDescriptor] {
    BASELINE_AYANAMSAS
}

/// Returns the release-specific ayanamsa additions beyond the baseline milestone.
pub const fn release_ayanamsas() -> &'static [AyanamsaDescriptor] {
    RELEASE_AYANAMSAS
}

/// Returns the full built-in ayanamsa catalog shipped by this release line.
pub const fn built_in_ayanamsas() -> &'static [AyanamsaDescriptor] {
    &BUILT_IN_AYANAMSAS
}

/// Returns the representative release-facing reference-offset sample set used by validation and reports.
pub const fn reference_offset_sample_ayanamsas() -> &'static [Ayanamsa] {
    &[
        Ayanamsa::Lahiri,
        Ayanamsa::LahiriIcrc,
        Ayanamsa::Lahiri1940,
        Ayanamsa::UshaShashi,
        Ayanamsa::Raman,
        Ayanamsa::Krishnamurti,
        Ayanamsa::FaganBradley,
        Ayanamsa::TrueChitra,
        Ayanamsa::TrueCitra,
        Ayanamsa::SuryasiddhantaRevati,
        Ayanamsa::SuryasiddhantaCitra,
        Ayanamsa::DeLuce,
        Ayanamsa::Yukteshwar,
        Ayanamsa::PvrPushyaPaksha,
        Ayanamsa::J2000,
        Ayanamsa::J1900,
        Ayanamsa::B1950,
        Ayanamsa::TrueRevati,
        Ayanamsa::TrueMula,
        Ayanamsa::TruePushya,
        Ayanamsa::Udayagiri,
        Ayanamsa::LahiriVP285,
        Ayanamsa::KrishnamurtiVP291,
        Ayanamsa::Sheoran,
        Ayanamsa::TrueSheoran,
        Ayanamsa::Hipparchus,
        Ayanamsa::DjwhalKhul,
        Ayanamsa::GalacticCenter,
        Ayanamsa::GalacticCenterRgilbrand,
        Ayanamsa::GalacticCenterMardyks,
        Ayanamsa::GalacticCenterCochrane,
        Ayanamsa::GalacticCenterMulaWilhelm,
        Ayanamsa::DhruvaGalacticCenterMula,
        Ayanamsa::GalacticEquatorIau1958,
        Ayanamsa::GalacticEquatorFiorenza,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticEquatorMula,
        Ayanamsa::ValensMoon,
        Ayanamsa::BabylonianBritton,
        Ayanamsa::BabylonianKugler1,
        Ayanamsa::BabylonianKugler2,
        Ayanamsa::BabylonianKugler3,
        Ayanamsa::BabylonianEtaPiscium,
        Ayanamsa::BabylonianAldebaran,
        Ayanamsa::BabylonianHuber,
        Ayanamsa::Aryabhata499,
        Ayanamsa::Sassanian,
        Ayanamsa::JnBhasin,
        Ayanamsa::GalacticEquator,
        Ayanamsa::Suryasiddhanta499,
        Ayanamsa::Suryasiddhanta499MeanSun,
        Ayanamsa::Aryabhata499MeanSun,
        Ayanamsa::Aryabhata522,
    ]
}

/// Returns a representative release-facing provenance sample set used by validation and reports.
pub const fn provenance_sample_ayanamsas() -> &'static [Ayanamsa] {
    &[
        Ayanamsa::TrueCitra,
        Ayanamsa::TrueRevati,
        Ayanamsa::TrueMula,
        Ayanamsa::TruePushya,
        Ayanamsa::Udayagiri,
        Ayanamsa::TrueSheoran,
        Ayanamsa::BabylonianBritton,
        Ayanamsa::GalacticCenterRgilbrand,
        Ayanamsa::BabylonianKugler1,
        Ayanamsa::GalacticEquator,
        Ayanamsa::Suryasiddhanta499MeanSun,
        Ayanamsa::Aryabhata522,
        Ayanamsa::ValensMoon,
    ]
}

/// Returns a representative release-facing provenance summary.
pub fn provenance_summary() -> AyanamsaProvenanceSummary {
    AyanamsaProvenanceSummary::new()
}

/// Returns the release-facing provenance payload after validation.
pub fn validated_provenance_summary_for_report(
) -> Result<String, crate::model::AyanamsaProvenanceSummaryValidationError> {
    provenance_summary().validated_summary_line()
}

pub(crate) const CUSTOM_DEFINITION_ONLY_AYANAMSAS: &[&str] = &[
    "Babylonian (House)",
    "Babylonian (Sissy)",
    "Babylonian (True Geoc)",
    "Babylonian (True Topc)",
    "Babylonian (True Obs)",
    "Babylonian (House Obs)",
];

/// Ad hoc ayanamsa labels intentionally surfaced as custom-definition territory.
pub const fn custom_definition_example_ayanamsa_labels() -> &'static [&'static str] {
    &["True Balarama", "Aphoric", "Takra"]
}

/// Release-profile custom-definition ayanamsa labels, including the built-in
/// Babylonian custom-definition-only entries and the ad hoc example labels.
pub const fn custom_definition_ayanamsa_labels() -> &'static [&'static str] {
    &[
        "Babylonian (House)",
        "Babylonian (Sissy)",
        "Babylonian (True Geoc)",
        "Babylonian (True Topc)",
        "Babylonian (True Obs)",
        "Babylonian (House Obs)",
        "True Balarama",
        "Aphoric",
        "Takra",
    ]
}

pub(crate) fn is_custom_definition_only_ayanamsa(canonical_name: &str) -> bool {
    CUSTOM_DEFINITION_ONLY_AYANAMSAS
        .iter()
        .any(|name| name.eq_ignore_ascii_case(canonical_name))
}

pub(crate) fn format_ayanamsa_label_list(labels: &[&'static str]) -> String {
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    }
}

/// Returns a coverage summary for the built-in ayanamsa catalog.
///
/// # Examples
///
/// ```
/// use pleiades_ayanamsa::metadata_coverage;
///
/// let coverage = metadata_coverage();
/// assert_eq!(coverage.to_string(), coverage.summary_line());
/// assert!(coverage
///     .summary_line()
///     .contains("entries with both a reference epoch and offset"));
/// ```
pub fn metadata_coverage() -> AyanamsaMetadataCoverage {
    let mut custom_definition_only = Vec::new();
    let mut without_sidereal_metadata = Vec::new();
    let mut with_sidereal_metadata = 0;

    for entry in built_in_ayanamsas() {
        if entry.has_sidereal_metadata() {
            with_sidereal_metadata += 1;
        } else if is_custom_definition_only_ayanamsa(entry.canonical_name) {
            custom_definition_only.push(entry.canonical_name);
        } else {
            without_sidereal_metadata.push(entry.canonical_name);
        }
    }

    AyanamsaMetadataCoverage {
        total: built_in_ayanamsas().len(),
        with_sidereal_metadata,
        custom_definition_only,
        without_sidereal_metadata,
    }
}

pub(crate) fn validate_ayanamsa_catalog_entries(
    entries: &[AyanamsaDescriptor],
) -> Result<usize, AyanamsaCatalogValidationError> {
    use std::collections::BTreeSet;

    if entries.is_empty() {
        return Err(AyanamsaCatalogValidationError::EmptyCatalog);
    }

    let mut labels_checked = 0usize;
    let mut seen_labels = BTreeSet::new();

    for entry in entries {
        labels_checked += 1;
        entry.validate()?;

        if resolve_ayanamsa(entry.canonical_name) != Some(entry.ayanamsa.clone()) {
            return Err(AyanamsaCatalogValidationError::LabelDoesNotRoundTrip {
                label: entry.canonical_name,
                expected_ayanamsa: entry.ayanamsa.clone(),
            });
        }
        if !seen_labels.insert(entry.canonical_name.to_ascii_lowercase()) {
            return Err(AyanamsaCatalogValidationError::DuplicateLabel {
                label: entry.canonical_name,
            });
        }

        for alias in entry.aliases.iter().copied() {
            labels_checked += 1;
            if resolve_ayanamsa(alias) != Some(entry.ayanamsa.clone()) {
                return Err(AyanamsaCatalogValidationError::LabelDoesNotRoundTrip {
                    label: alias,
                    expected_ayanamsa: entry.ayanamsa.clone(),
                });
            }

            if alias.eq_ignore_ascii_case(entry.canonical_name) {
                if alias != entry.canonical_name {
                    continue;
                }

                return Err(AyanamsaCatalogValidationError::DuplicateLabel { label: alias });
            }

            if !seen_labels.insert(alias.to_ascii_lowercase()) {
                return Err(AyanamsaCatalogValidationError::DuplicateLabel { label: alias });
            }
        }
    }

    Ok(labels_checked)
}

/// Validates the built-in ayanamsa catalog for label uniqueness and round-trips.
pub fn validate_ayanamsa_catalog() -> Result<(), AyanamsaCatalogValidationError> {
    validate_ayanamsa_catalog_entries(built_in_ayanamsas()).map(|_| ())
}

/// Returns a compact validation summary for the built-in ayanamsa catalog.
pub fn ayanamsa_catalog_validation_summary() -> AyanamsaCatalogValidationSummary {
    let entry_count = built_in_ayanamsas().len();
    let baseline_entry_count = baseline_ayanamsas().len();
    let release_entry_count = release_ayanamsas().len();
    let mc = metadata_coverage();
    let (label_count, validation_result) =
        match validate_ayanamsa_catalog_entries(built_in_ayanamsas()) {
            Ok(label_count) => (label_count, Ok(())),
            Err(error) => (0, Err(error)),
        };

    AyanamsaCatalogValidationSummary {
        entry_count,
        baseline_entry_count,
        release_entry_count,
        label_count,
        metadata_coverage: mc,
        validation_result,
    }
}

/// Finds the descriptor for a typed ayanamsa selection.
pub fn descriptor(ayanamsa: &Ayanamsa) -> Option<&'static AyanamsaDescriptor> {
    built_in_ayanamsas()
        .iter()
        .find(|entry| entry.ayanamsa == *ayanamsa)
}

/// Resolves an ayanamsa label to a built-in type.
pub fn resolve_ayanamsa(label: &str) -> Option<Ayanamsa> {
    built_in_ayanamsas()
        .iter()
        .find(|entry| entry.matches_label(label))
        .map(|entry| entry.ayanamsa.clone())
}

/// Returns the sidereal offset for the provided ayanamsa and instant.
///
/// Built-in catalog entries use the published reference epoch and offset
/// metadata where available. Custom ayanamsas can supply the same information
/// directly on the `CustomAyanamsa` definition.
///
/// # Examples
///
/// ```
/// use pleiades_ayanamsa::sidereal_offset;
/// use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};
///
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let offset = sidereal_offset(&Ayanamsa::Lahiri, instant)
///     .expect("Lahiri should carry reference sidereal metadata");
///
/// assert!(offset.degrees() > 20.0);
/// ```
pub fn sidereal_offset(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    use crate::thresholds::{ayanamsa_mode_class, AyanamsaModeClass};

    if let Ayanamsa::Custom(custom) = ayanamsa {
        return offset_from_components(custom.epoch, custom.offset_degrees, instant);
    }

    let jd_tt = instant.julian_day.days();
    match ayanamsa_mode_class(ayanamsa) {
        // Offset-defined: documented epoch anchor + IAU-2006 precession drift.
        Some(AyanamsaModeClass::OffsetDefined) => {
            let entry = descriptor(ayanamsa)?;
            let epoch = entry.epoch?;
            let offset = entry.offset_degrees?;
            let drift = crate::precession::precession_delta_degrees(jd_tt, epoch.days());
            Some(Angle::from_degrees(offset.degrees() + drift))
        }
        // True-star: committed cubic fit to Swiss Ephemeris.
        Some(AyanamsaModeClass::TrueStar) => {
            crate::truestar::true_star_offset_degrees(ayanamsa, jd_tt).map(Angle::from_degrees)
        }
        // Galactic: committed cubic fit to Swiss Ephemeris.
        Some(AyanamsaModeClass::Galactic) => {
            crate::galactic::galactic_offset_degrees(ayanamsa, jd_tt).map(Angle::from_degrees)
        }
        // FittedOffset: cubic-fit evaluator wired in a later task; fall through to legacy path.
        Some(AyanamsaModeClass::FittedOffset) => {
            descriptor(ayanamsa).and_then(|entry| entry.offset_at(instant))
        }
        // Not gated: unchanged legacy linear-rate path.
        None => descriptor(ayanamsa).and_then(|entry| entry.offset_at(instant)),
    }
}

pub(crate) fn offset_from_components(
    epoch: Option<JulianDay>,
    offset_degrees: Option<Angle>,
    instant: Instant,
) -> Option<Angle> {
    let offset = offset_degrees?;
    let epoch = epoch?;
    let centuries = (instant.julian_day.days() - epoch.days()) / 36_525.0;
    Some(Angle::from_degrees(
        offset.degrees() + centuries * SIDEREAL_PRECESSION_DEGREES_PER_CENTURY,
    ))
}

const SIDEREAL_PRECESSION_DEGREES_PER_CENTURY: f64 = 1.396_971_277_777_777_8;
