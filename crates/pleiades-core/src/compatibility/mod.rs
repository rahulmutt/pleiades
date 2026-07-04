//! Versioned compatibility profile for the current release line.
//!
//! The profile is intentionally explicit about what the repository ships today
//! versus what remains for later stages. It can be printed by the CLI and used
//! in documentation or release notes so consumers know which built-ins and
//! aliases are actually available.

#![forbid(unsafe_code)]

mod aliases;
mod profile;
mod report;
#[cfg(test)]
mod tests;
mod validation;

pub use profile::{CompatibilityProfile, HouseCodeAliasInventorySummary};
pub use validation::{validate_custom_definition_labels, CompatibilityProfileValidationError};

use pleiades_ayanamsa::{
    baseline_ayanamsas, built_in_ayanamsas, custom_definition_ayanamsa_labels, release_ayanamsas,
};
use pleiades_houses::{baseline_house_systems, built_in_house_systems, release_house_systems};

/// The current compatibility-profile identifier.
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.7.6";

/// FNV-1a/64 checksum (via [`pleiades_time::fnv1a64`]) of the fully rendered
/// `current_compatibility_profile()` text.
///
/// This couples the profile id above to the bytes it actually renders: any edit
/// that changes the rendered profile (a descriptor string, a summary, a release
/// note, a catalog entry) changes this checksum and trips
/// `rendered_profile_matches_pinned_content_checksum`. When that test fails,
/// bump `CURRENT_COMPATIBILITY_PROFILE_ID` and update this value in the same
/// commit so the version can never silently diverge from the content it names.
#[cfg(test)]
const CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM: u64 = 0x43f1_5a28_f1c1_204a;

/// The current compatibility-profile release summary.
pub const CURRENT_COMPATIBILITY_PROFILE_SUMMARY: &str = "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit, including the release-specific house-system additions across the Carter, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen, including the exact Pullen SD table of houses, Pullen SD (Neo-Porphyry) table of houses, Pullen SD (Neo-Porphyry), Neo-Porphyry, Pullen SD (Sinusoidal Delta), Pullen SR table of houses, Pullen SR (Sinusoidal Ratio) table of houses, and Pullen SR (Sinusoidal Ratio) spellings, Sunshine, including the Bob Makransky, Makransky Sunshine, and Treindl Sunshine source labels, and Gauquelin families, plus the expanded ayanamsa coverage for J2000/J1900/B1950, True Citra and the True Citra Paksha / True Chitra Paksha / True Chitrapaksha interoperability spellings, DeLuce, Yukteshwar including the Sri Yukteshwar / Shri Yukteswar / Shri Yukteshwar transliterations, PVR Pushya-paksha, including the exact PVR Pushya Paksha spelling, Sheoran, and the Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa source spellings, the true-nakshatra and Suryasiddhanta Revati/Citra reference modes, the Hipparchus/Babylonian/Galactic reference-frame modes, the latest True Pushya, Udayagiri, Lahiri (VP285), Krishnamurti (VP291), Krishnamurti ayanamsa, Djwhal Khul, JN Bhasin, mean-sun, Valens Moon, and the Valens / Moon / Moon sign / Moon sign ayanamsa / Valens Moon ayanamsa source spellings, Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane/Mardyks) with the David Cochrane source name, Galactic Equator (Mula), the Babylonian house/sissy/true-geoc/true-topc/true-obs/house-obs variants, the backfilled True Sheoran, Galactic Center (Rgilbrand), the Skydram / Skydram/Galactic Alignment / Skydram (Mardyks) source spellings, and Galactic Center (Mula/Wilhelm) zero-point metadata, including the Dhruva/Gal.Center/Mula (Wilhelm), Mula Wilhelm, and Wilhelm source spellings, the additional Galactic Equator/Center variants including Galactic Equator (True) / True galactic equator / Galactic equator true and the `Gal. Center = 0 Sag` and `Gal. Center = 0 Cap` spellings, the exact Swiss Ephemeris source-label aliases for the Babylonian/Kugler family plus the Babylonian Kugler 1/2/3 plain spellings, the Babylonian 1/2/3 shorthand forms, and Babylonian Huber, the galactic-reference, mean-sun, Sassanian/Sasanian/Zij al-Shah, Aryabhata 499/522, and the Surya Siddhanta / Suryasiddhanta 499/499 CE source-form entries, the expanded APC and Horizon/Azimuth interoperability aliases, the Topocentric house-system alias and the exact Polich-Page \"topocentric\" table of houses, Polich/Page, Polich Page, and T Polich/Page (\"topocentric\") source spellings, the baseline Fagan/Bradley and Usha Shashi source-label appendix entries, the Babylonian house-family labels now rendered as explicit custom-definition territory rather than unresolved release gaps, and the `Equal (MC)` / `Equal (1=Aries)` source-label appendix entries for the release-line equal-house variants, including the `Equal from MC`, `Equal (from MC)`, `Equal (from MC) table of houses`, and `Equal/MC = 10th` spellings alongside the `Equal (MC)` table of houses, `Equal Midheaven table of houses`, `Equal (1=Aries)` table of houses, `Equal/1=0 Aries`, and `Equal (cusp 1 = 0° Aries)` spellings, plus the Wang, Aries houses, P.V.R. Narasimha Rao, and True Mula (Chandra Hari) source-label appendix entries for the ascendant-anchored equal-house and true-Mula variants, along with the exact Swiss Ephemeris house-table code spellings surfaced in the source-label appendix and the Equal table of houses, Whole Sign system, and Morinus house system spellings now called out explicitly in the quick-audit text, plus the Nick Anthony Fiorenza source name for Galactic Equator (Fiorenza). Unsupported modes remain explicit: built-in UTC convenience remains out of scope; built-in Delta T remains out of scope; chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported; apparent-place corrections are rejected unless a backend explicitly advertises support; native sidereal backend output remains unsupported unless a backend explicitly advertises it. SP-1 (angles and sidereal time) additions: public sidereal-time helpers (GMST, GAST, local sidereal time via pleiades_apparent::sidereal_time and SiderealTime) and AscMc chart-point extras (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant via pleiades_houses::AscMc, chart_points, and chart_points_from_armc) are now part of the stable chart surface; HouseSnapshot::asc_mc carries AscMc on every house snapshot; HouseSnapshot is now #[non_exhaustive] as a deliberate one-time 0.2.x breaking change; the validate-angles numeric gate is now part of the release gate set. SP-2a (longitude crossings) additions: a new pleiades-events crate ships a longitude-crossing engine — CrossingEngine with next_sun_crossing/next_moon_crossing (Swiss-Ephemeris solcross/mooncross analogues), general geocentric-apparent-of-date body crossings, heliocentric helio_cross crossings, and a CrossingEngine::longitude_at evaluator — over the 1900-2100 TDB window, exposed via the validate-crossings CLI (aliases crossings / crossings-gate) and not re-exported from pleiades-core. The fail-closed validate-crossings gate is two-tier over a committed corpus covering geocentric and heliocentric bodies Mercury-Pluto (plus Sun/Moon geocentric): Tier 1 recomputes each crossing and holds it to a sub-second self-consistency ceiling vs a committed engine golden column; Tier 2 evaluates the engine's longitude at the Swiss-Ephemeris crossing time and holds it to per-body arcsecond ceilings (documented cross-theory floors vs SE Moshier; Pluto rides the shared planet/heliocentric ceilings and agrees within the same arcsecond floor as the other planets, so it needs no wider declared boundary; precedent: validate-lilith accepts an SE-vs-ours floor of ~306\"). The corpus is checksum-guarded (fnv1a64) and pinned by row count.";

/// Returns the current compatibility-profile identifier.
pub const fn current_compatibility_profile_id() -> &'static str {
    CURRENT_COMPATIBILITY_PROFILE_ID
}

/// Returns the current compatibility profile.
pub const fn current_compatibility_profile() -> CompatibilityProfile {
    CompatibilityProfile {
        profile_id: CURRENT_COMPATIBILITY_PROFILE_ID,
        summary: CURRENT_COMPATIBILITY_PROFILE_SUMMARY,

        target_house_scope: &[
            "Target house scope: the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal.",
            "Baseline milestone: Placidus, Koch, Porphyry, Regiomontanus, Campanus, Equal, Whole Sign, Alcabitius, Meridian/ARMC/Axial variants, Topocentric, and Morinus are shipped today.",
        ],
        target_ayanamsa_scope: &[
            "Target ayanamsa scope: the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal.",
            "Baseline milestone: Lahiri, Raman, Krishnamurti, Fagan/Bradley, True Chitra, and documented aliases/custom variants are shipped today.",
        ],
        house_systems: built_in_house_systems(),
        baseline_house_systems: baseline_house_systems(),
        release_house_systems: release_house_systems(),
        ayanamsas: built_in_ayanamsas(),
        baseline_ayanamsas: baseline_ayanamsas(),
        release_ayanamsas: release_ayanamsas(),
        release_notes: &[
            "The JPL snapshot backend preserves selected asteroid coverage, including the source-backed custom body asteroid:433-Eros, and the validation report surfaces that subset separately from the planetary comparison corpus.",
            "Release-specific house-system additions now include Equal (MC), Equal (1=Aries), Vehlow Equal, Vehlow house system, Vehlow Equal house system, Sripati, Carter (poli-equatorial), including Carter's poli-equatorial, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Krusinski/Pisa/Goelzer, Albategnius, Pullen SD, Pullen SR, including the exact Pullen SD table of houses, Pullen SD (Neo-Porphyry) table of houses, Pullen SD (Neo-Porphyry), Neo-Porphyry, Pullen SD (Sinusoidal Delta), Pullen SR table of houses, Pullen SR (Sinusoidal Ratio) table of houses, and Pullen SR (Sinusoidal Ratio) spellings, Sunshine, including the Bob Makransky, Makransky Sunshine, and Treindl Sunshine source labels for Sunshine, and Gauquelin sectors, with the Whole Sign (house 1 = Aries) label, the Whole sign houses, 1. house = Aries source spelling, Wang alias, Equal MC / Equal/MC / Equal Midheaven / Equal Midheaven house system aliases, Equal (cusp 1 = Asc) source spelling, Equal (MC) and Equal (1=Aries) source-label appendix entries, including the Equal from MC, Equal (from MC), Equal (from MC) table of houses, and Equal/MC = 10th spellings alongside the Equal (MC) table of houses, Equal (MC) house system, Equal/MC house system, Equal (1=Aries) table of houses, Equal/1=Aries house system spelling, and Equal (1=Aries) house system spellings, plus the exact Equal/1=0 Aries and Equal (cusp 1 = 0° Aries) source-label forms, APC houses / Ascendant Parallel Circle / WvA aliases, Horizon / Horizontal / Azimuthal aliases, the exact Topocentric source labels `Polich-Page \"topocentric\" table of houses`, `Polich/Page`, `Polich Page`, and `T Polich/Page (\"topocentric\")`, the `Horizon/Azimuth house system` and `Horizon/Azimuth table of houses` source labels, the Vehlow-equal source label and the Vehlow house system / Vehlow Equal house system / Vehlow Equal table of houses search forms, the Bob Makransky source label for Sunshine, the Topocentric house system alias, the baseline Placidus and Koch table-of-houses source spellings, the remaining Albategnius / Pullen SD (Sinusoidal Delta) / Pullen SR (Sinusoidal Ratio) / Gauquelin source labels, the Swiss Ephemeris single-letter house-table codes P/K/R/C/O/E/W/N/V/A/H/B/M/S/I/G plus the additional T/U/X/Y interoperability codes resolving to their corresponding built-ins, and the exact Swiss Ephemeris house-table code spellings A equal, D equal / MC, E equal = A, N whole sign houses, 1. house = Aries, S sripati, I sunshine, W equal, whole sign, V equal Vehlow, T topocentric, U Krusinski-Pisa-Goelzer, Zariel, X axial rotation system/ Meridian houses, and Y APC houses, plus the explicit Meridian house system, Horizontal house system, and Azimuth house system spellings.",
            "The compatibility profile now also renders a source-label appendix for the built-in house systems so common Placidus, Koch, Equal, Whole Sign, Topocentric, Vehlow, Meridian, Zariel, ARMC, Sunshine, APC, and Horizon/Azimuth spellings — including the Swiss Ephemeris \"Equal (cusp 1 = Asc)\", \"Whole Sign (house 1 = Aries)\", \"Polich-Page \\\"topocentric\\\" table of houses\", \"T Polich/Page (\\\"topocentric\\\")\", \"Horizon/Azimuth house system\", and \"Horizon/Azimuth table of houses\" forms — are searchable alongside the ayanamsa appendix, and the latest release-specific house-system label batches now also surface the exact Placidus table of houses, Koch table of houses, Koch houses, house system of the birth place, Albategnius, Pullen, Vehlow house system, Vehlow Equal house system, and Gauquelin search forms, plus the exact Equal table of houses, Whole Sign system, and Morinus house system spellings now called out explicitly in the quick-audit text.",
            "The compatibility profile now also surfaces the exact Swiss Ephemeris house-table code spellings A equal, D equal / MC, E equal = A, N whole sign houses, 1. house = Aries, S sripati, I sunshine, W equal, whole sign, V equal Vehlow, T topocentric, U Krusinski-Pisa-Goelzer, Zariel, X axial rotation system/ Meridian houses, and Y APC houses so the code-style interoperability forms remain searchable alongside the canonical house names.",
            "The Equal (MC) and Equal (1=Aries) release-line house entries now also accept the plain Equal (MC) house system, Equal Midheaven table of houses, and Equal (1=Aries) house system spellings, keeping the release-facing alias batch aligned with common source-label wording.",
            "The compatibility profile now also renders source-label appendix entries for Lahiri / Chitrapaksha / Chitra Paksha, True Chitra / Chitra, Krishnamurti Ayanamsha / Krishnamurti Ayanamsa / Krishnamurti ayanamsa / Krishnamurti (Swiss) / Krishnamurti Paddhati / KP ayanamsa, Fagan/Bradley Ayanamsha / Fagan/Bradley / Fagan Bradley / Fagan / Bradley / Fagan-Bradley, Usha Shashi / Usha / Shashi, and the Yukteshwar / Sri Yukteshwar / Shri Yukteshwar transliterations so the baseline sidereal spellings remain searchable alongside the existing Raman appendix entry and the rest of the ayanamsa catalog.",
            "The compatibility profile now also renders source-label appendix entries for P.V.R. Narasimha Rao, Aries houses, and True Mula (Chandra Hari) so the release-facing interoperability labels stay aligned with the documented source spellings for the Pushya-paksha, equal-house, and true-Mula variants.",
            "The compatibility profile now also renders source-label appendix entries for the Galactic equator, IAU 1958, true, Mula, and Fiorenza spellings, including the David Cochrane and Nick Anthony Fiorenza source names for the Cochrane and Fiorenza galactic-reference entries, so the release-facing galactic-reference labels stay aligned with the resolver aliases.",
            "The compatibility profile now also renders a source-label appendix entry for Raman so the B. V. Raman, B.V. Raman, B V Raman, Raman Ayanamsha, and Raman ayanamsa spellings are searchable alongside the other baseline ayanamsa labels.",
            "The True Citra entry now also accepts the True Citra Paksha and True Chitrapaksha spellings, and the release profile summary highlights that alias batch explicitly so the release-facing source-label appendix stays aligned with common interoperability wording.",
            "Release-specific ayanamsa additions now include J2000, J1900, B1950, True Citra, DeLuce, Yukteshwar (including the Sri Yukteshwar / Shri Yukteswar / Shri Yukteshwar transliterations), PVR Pushya-paksha, Sheoran, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran) with the Babylonian/Aldebaran = 15 Tau source form, Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Pushya, Udayagiri, Lahiri (VP285), Krishnamurti (VP291) with the Krishnamurti-Senthilathiban source form, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), the Surya Siddhanta mean-sun source forms, the Aryabhata mean-sun source forms, Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), True Sheoran, Galactic Center, Galactic Center (Rgilbrand), Galactic Center (Mardyks) with the Skydram / Skydram/Galactic Alignment / Skydram (Mardyks) source spellings, Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), and Valens Moon, with explicit zero-point metadata now published for Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Britton), Udayagiri, Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center, Galactic Center (Rgilbrand), Galactic Center (Mardyks) with the Skydram / Skydram/Galactic Alignment / Skydram (Mardyks) source spellings, Galactic Center (Mula/Wilhelm) including the Dhruva/Gal.Center/Mula (Wilhelm), Mula Wilhelm, and Wilhelm source spellings, Galactic Center (Cochrane), JN Bhasin, Babylonian (Eta Piscium), Babylonian (Aldebaran), Galactic Equator (Mula), Suryasiddhanta (Mean Sun), the Surya Siddhanta mean-sun source forms, the Aryabhata mean-sun source forms, Aryabhata (Mean Sun), Aryabhata (522 CE), Galactic Equator (True) / True galactic equator / Galactic equator true entries; the Babylonian house-family source labels now resolve as exact aliases too, Galactic Equator (Fiorenza) continues to carry a J2000.0 reference epoch and 25° zero-point offset for the release profile, the Babylonian house-family labels now render in a separate custom-definition section, and the plain Moon alias also resolves to Valens Moon for compatibility with existing label variants, while the Valens Moon source-label appendix now also includes the Valens, Moon, Moon sign, Moon sign ayanamsa, and Valens Moon ayanamsa source spellings, the release profile now surfaces the Aryabhata 499/522 and Surya Siddhanta / Suryasiddhanta 499/499 CE source spellings explicitly, and the release-facing source-label appendix now also calls out the Babylonian 1/2/3 shorthand labels, Babylonian Huber, Aryabhatan Kaliyuga / Aryabhata Kaliyuga spellings, Fagan/Bradley Ayanamsha / Fagan/Bradley spellings, Krishnamurti Ayanamsha / Krishnamurti (Swiss) search forms, the Sunil Sheoran / Vedic Sheoran / Sheoran ayanamsa spellings, and the Usha Shashi search forms explicitly, alongside the new Lahiri / Chitrapaksha and True Chitra / Chitra appendix entries.",
            "Non-standard ayanamsa labels such as True Balarama, Aphoric, and Takra are intentionally treated as custom definitions until a documented source mapping is added.",
            "The compatibility profile is intended to be archived with release validation outputs and release notes.",
            "SP-1 (angles and sidereal time): public sidereal-time helpers (GMST/GAST/local via pleiades_apparent::sidereal_time and SiderealTime, plus greenwich_mean_sidereal_time_degrees, equation_of_equinoxes_degrees, and the shared equation_of_equinoxes helper) and AscMc chart-point extras (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant via pleiades_houses::AscMc, chart_points, and chart_points_from_armc) are now part of the stable chart surface; HouseSnapshot::asc_mc carries AscMc on every house snapshot; HouseSnapshot is now #[non_exhaustive] as a deliberate one-time 0.2.x breaking change; ChartSnapshot::asc_mc() re-exposes AscMc at the facade layer; the validate-angles numeric gate is wired into run_all_numeric_gates.",
            "SP-2a (longitude crossings): a new pleiades-events crate ships a longitude-crossing engine — CrossingEngine with next_sun_crossing/next_moon_crossing (Swiss-Ephemeris solcross/mooncross analogues), general geocentric-apparent-of-date body crossings, heliocentric helio_cross crossings, and a CrossingEngine::longitude_at evaluator — over the 1900-2100 TDB window, exposed via the validate-crossings CLI (aliases crossings / crossings-gate) and not re-exported from pleiades-core. The fail-closed validate-crossings gate is two-tier over a committed corpus covering geocentric and heliocentric bodies Mercury-Pluto (plus Sun/Moon geocentric): Tier 1 recomputes each crossing and holds it to a sub-second self-consistency ceiling vs a committed engine golden column; Tier 2 evaluates the engine's longitude at the Swiss-Ephemeris crossing time and holds it to per-body arcsecond ceilings (documented cross-theory floors vs SE Moshier; Pluto rides the shared planet/heliocentric ceilings and agrees within the same arcsecond floor as the other planets, so it needs no wider declared boundary; precedent: validate-lilith accepts an SE-vs-ours floor of ~306\"). The corpus is checksum-guarded (fnv1a64) and pinned by row count.",
        ],
        validation_reference_points: &[
            "The stage-4 validation corpus remains the reference point for tightening house formulas whenever future revisions land.",
        ],
        custom_definition_labels: custom_definition_ayanamsa_labels(),
        known_gaps: &[
            "The newly added historical/reference-frame and formula-variant ayanamsa modes are catalogued and resolvable, and the release line now publishes explicit sidereal metadata for Babylonian (Huber), Babylonian (Britton), Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Galactic Center (Cochrane), Galactic Center (Mardyks), Galactic Center (Rgilbrand), Galactic Center (Mula/Wilhelm), Galactic Equator (IAU 1958), Galactic Equator (Fiorenza), Suryasiddhanta (Revati), Suryasiddhanta (Citra), True Pushya, True Sheoran, Udayagiri, Lahiri (VP285), Krishnamurti (VP291), Djwhal Khul, Valens Moon, and the remaining historical/reference-frame catalog entries; additional metadata/source mapping work remains scheduled for any unreconciled future breadth batches or custom definitions.",
            "Labels outside the published compatibility profile, including ad hoc names such as True Balarama, Aphoric, and Takra, should be modeled as custom ayanamsa definitions rather than assumed to be built-ins.",
        ],
    }
}

/// Returns the compatibility-profile house formula family summary for report surfaces.
pub fn house_formula_families_summary_for_report() -> String {
    current_compatibility_profile().house_formula_families_summary_line()
}

/// Returns the compatibility-profile house formula family summary after validating the profile.
pub fn validated_house_formula_families_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_house_formula_families_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house-system summary for report surfaces.
pub fn latitude_sensitive_house_systems_summary_for_report() -> String {
    current_compatibility_profile().latitude_sensitive_house_systems_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house-system summary after validating the profile.
pub fn validated_latitude_sensitive_house_systems_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_latitude_sensitive_house_systems_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house-constraint summary for report surfaces.
pub fn latitude_sensitive_house_constraints_summary_for_report() -> String {
    current_compatibility_profile().latitude_sensitive_house_constraints_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house-constraint summary after validating the profile.
pub fn validated_latitude_sensitive_house_constraints_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_latitude_sensitive_house_constraints_summary_line()
}

/// Returns the compatibility-profile custom-definition ayanamsa summary for report surfaces.
pub fn custom_definition_ayanamsa_labels_summary_for_report() -> String {
    current_compatibility_profile().custom_definition_ayanamsa_labels_summary_line()
}

/// Returns the compatibility-profile custom-definition ayanamsa summary after validating the profile.
pub fn validated_custom_definition_ayanamsa_labels_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_custom_definition_ayanamsa_labels_summary_line()
}

/// Returns the compatibility-profile catalog inventory summary for report surfaces.
pub fn catalog_inventory_summary_for_report() -> String {
    current_compatibility_profile().catalog_inventory_summary_line()
}

/// Returns the compatibility-profile catalog-posture summary for report surfaces.
pub fn catalog_posture_summary_for_report() -> String {
    current_compatibility_profile().catalog_posture_summary_line()
}

/// Returns the compatibility-profile catalog-posture summary after validating the profile.
pub fn validated_catalog_posture_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_catalog_posture_summary_line()
}

/// Returns the compatibility-profile known-gaps summary for report surfaces.
pub fn known_gaps_summary_for_report() -> String {
    current_compatibility_profile().known_gaps_summary_line()
}

/// Returns the compatibility-profile known-gaps summary after validating the profile.
pub fn validated_known_gaps_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_known_gaps_summary_line()
}

/// Returns the compatibility-profile catalog inventory summary after validating the profile.
pub fn validated_catalog_inventory_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_catalog_inventory_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house failure modes summary for report surfaces.
pub fn latitude_sensitive_house_failure_modes_summary_for_report() -> String {
    current_compatibility_profile().latitude_sensitive_house_failure_modes_summary_line()
}

/// Returns the compatibility-profile latitude-sensitive house failure modes summary after validating the profile.
pub fn validated_latitude_sensitive_house_failure_modes_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validate()?;
    Ok(current_compatibility_profile().latitude_sensitive_house_failure_modes_summary_line())
}

/// Returns the compatibility-caveats summary for report surfaces.
pub fn compatibility_caveats_summary_for_report(
    profile: &CompatibilityProfile,
    release_profiles: &crate::release_profiles::ReleaseProfileIdentifiers,
) -> String {
    let mut text = String::new();

    text.push_str("Compatibility caveats summary\n");
    text.push_str("Profile: ");
    text.push_str(release_profiles.compatibility_profile_id);
    text.push('\n');
    text.push_str("Compatibility caveats: ");
    text.push_str(&profile.known_gaps.len().to_string());
    text.push('\n');
    text.push_str("House formula families: ");
    text.push_str(&profile.house_formula_families_summary_line());
    text.push('\n');
    text.push_str("Latitude-sensitive house systems: ");
    text.push_str(&profile.latitude_sensitive_house_systems_summary_line());
    text.push('\n');
    text.push_str("Latitude-sensitive house constraints: ");
    text.push_str(&profile.latitude_sensitive_house_constraints_summary_line());
    text.push('\n');
    text.push_str("Latitude-sensitive house failure modes: ");
    text.push_str(&profile.latitude_sensitive_house_failure_modes_summary_line());
    text.push('\n');
    text.push_str("Descriptor-only ayanamsa labels: ");
    text.push_str(&profile.custom_definition_ayanamsa_labels_summary_line());
    text.push('\n');
    for gap in profile.known_gaps {
        text.push_str("- ");
        text.push_str(gap);
        text.push('\n');
    }

    text
}

/// Returns the house-code alias inventory for report surfaces.
pub fn house_code_aliases_summary_for_report() -> String {
    current_compatibility_profile().house_code_aliases_summary_line()
}

/// Returns the house-code alias inventory after validating the current profile.
pub fn validated_house_code_aliases_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_house_code_aliases_summary_line()
}

/// Returns the release-specific house-system canonical names summary for report surfaces.
pub fn release_house_system_canonical_names_summary_for_report() -> String {
    current_compatibility_profile().release_house_system_canonical_names_summary_line()
}

/// Returns the release-specific house-system canonical names summary after validating the profile.
pub fn validated_release_house_system_canonical_names_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_release_house_system_canonical_names_summary_line()
}

/// Returns the release-specific ayanamsa canonical names summary for report surfaces.
pub fn release_ayanamsa_canonical_names_summary_for_report() -> String {
    current_compatibility_profile().release_ayanamsa_canonical_names_summary_line()
}

/// Returns the release-specific ayanamsa canonical names summary after validating the profile.
pub fn validated_release_ayanamsa_canonical_names_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_release_ayanamsa_canonical_names_summary_line()
}

/// Returns the target house-system scope summary for report surfaces.
pub fn target_house_scope_summary_for_report() -> String {
    current_compatibility_profile().target_house_scope_summary_line()
}

/// Returns the target house-system scope summary after validating the profile.
pub fn validated_target_house_scope_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_target_house_scope_summary_line()
}

/// Returns the target ayanamsa scope summary for report surfaces.
pub fn target_ayanamsa_scope_summary_for_report() -> String {
    current_compatibility_profile().target_ayanamsa_scope_summary_line()
}

/// Returns the target ayanamsa scope summary after validating the profile.
pub fn validated_target_ayanamsa_scope_summary_for_report(
) -> Result<String, CompatibilityProfileValidationError> {
    current_compatibility_profile().validated_target_ayanamsa_scope_summary_line()
}
