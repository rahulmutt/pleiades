//! Versioned compatibility profile for the current release line.
//!
//! The profile is intentionally explicit about what the repository ships today
//! versus what remains for later stages. It can be printed by the CLI and used
//! in documentation or release notes so consumers know which built-ins and
//! aliases are actually available.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_ayanamsa::{
    baseline_ayanamsas, built_in_ayanamsas, metadata_coverage, release_ayanamsas,
    AyanamsaDescriptor,
};
use pleiades_houses::{
    baseline_house_systems, built_in_house_systems, release_house_systems, HouseSystemDescriptor,
};

/// The current compatibility-profile identifier.
pub const CURRENT_COMPATIBILITY_PROFILE_ID: &str = "pleiades-compatibility-profile/0.6.60";

/// Returns the current compatibility-profile identifier.
pub const fn current_compatibility_profile_id() -> &'static str {
    CURRENT_COMPATIBILITY_PROFILE_ID
}

/// A release-scoped compatibility profile.
#[derive(Clone, Copy, Debug)]
pub struct CompatibilityProfile {
    /// Stable profile identifier.
    pub profile_id: &'static str,
    /// Human-readable summary of the current release posture.
    pub summary: &'static str,
    /// Scope note describing the long-term house-system target.
    pub target_house_scope: &'static [&'static str],
    /// Scope note describing the long-term ayanamsa target.
    pub target_ayanamsa_scope: &'static [&'static str],
    /// Built-in house systems shipped in this release line.
    pub house_systems: &'static [HouseSystemDescriptor],
    /// House systems that belong to the published baseline milestone.
    pub baseline_house_systems: &'static [HouseSystemDescriptor],
    /// Release-specific house-system additions beyond the baseline milestone.
    pub release_house_systems: &'static [HouseSystemDescriptor],
    /// Built-in ayanamsas shipped in this release line.
    pub ayanamsas: &'static [AyanamsaDescriptor],
    /// Built-in ayanamsas that belong to the published baseline milestone.
    pub baseline_ayanamsas: &'static [AyanamsaDescriptor],
    /// Release-specific ayanamsa additions beyond the baseline milestone.
    pub release_ayanamsas: &'static [AyanamsaDescriptor],
    /// Explicitly documented release-specific notes beyond the baseline milestone.
    pub release_notes: &'static [&'static str],
    /// Validation reference points that are intentionally surfaced separately
    /// from unresolved compatibility gaps.
    pub validation_reference_points: &'static [&'static str],
    /// Labels that are intentionally surfaced as custom-definition territory
    /// instead of unresolved compatibility gaps.
    pub custom_definition_labels: &'static [&'static str],
    /// Explicitly documented compatibility caveats and follow-up notes.
    pub known_gaps: &'static [&'static str],
}

impl CompatibilityProfile {
    /// Returns a short release note string.
    pub const fn release_note(&self) -> &'static str {
        self.summary
    }
}

/// Returns the current compatibility profile.
pub const fn current_compatibility_profile() -> CompatibilityProfile {
    CompatibilityProfile {
        profile_id: CURRENT_COMPATIBILITY_PROFILE_ID,
        summary: "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit, including the release-specific house-system additions across the Carter, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen, Sunshine, and Gauquelin families, plus the expanded ayanamsa coverage for J2000/J1900/B1950, True Citra and the True Citra Paksha / True Chitra Paksha / True Chitrapaksha interoperability spellings, DeLuce, Yukteshwar including the Sri Yukteshwar / Shri Yukteswar / Shri Yukteshwar transliterations, PVR Pushya-paksha, Sheoran, the true-nakshatra and Suryasiddhanta Revati/Citra reference modes, the Hipparchus/Babylonian/Galactic reference-frame modes, the latest True Pushya, Udayagiri, Lahiri (VP285), Krishnamurti (VP291), Djwhal Khul, JN Bhasin, mean-sun, Valens Moon, and the Valens / Moon sign ayanamsa source spellings, Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane/Mardyks), Galactic Equator (Mula), the Babylonian house/sissy/true-geoc/true-topc/true-obs/house-obs variants, the backfilled True Sheoran, Galactic Center (Rgilbrand), and Galactic Center (Mula/Wilhelm) zero-point metadata, the additional Galactic Equator/Center variants, the exact Swiss Ephemeris source-label aliases for the Babylonian/Kugler family plus the Babylonian 1/2/3 shorthand forms and Babylonian Huber, the galactic-reference, mean-sun, Sassanian/Zij al-Shah, Aryabhata 499/522, and the Surya Siddhanta / Suryasiddhanta 499/499 CE source-form entries, the expanded APC and Horizon/Azimuth interoperability aliases, the baseline Fagan/Bradley and Usha Shashi source-label appendix entries, the Babylonian house-family labels now rendered as explicit custom-definition territory rather than unresolved release gaps, and the `Equal (MC)` / `Equal (1=Aries)` source-label appendix entries for the release-line equal-house variants, plus the Wang, Aries houses, P.V.R. Narasimha Rao, and True Mula (Chandra Hari) source-label appendix entries for the ascendant-anchored equal-house and true-Mula variants, along with the exact Swiss Ephemeris house-table code spellings surfaced in the source-label appendix.",

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
            "Release-specific house-system additions now include Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Krusinski/Pisa/Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, and Gauquelin sectors, with the Whole Sign (house 1 = Aries) label, the Whole sign houses, 1. house = Aries source spelling, Wang alias, Equal MC / Equal Midheaven aliases, Equal (cusp 1 = Asc) source spelling, Equal (MC) and Equal (1=Aries) source-label appendix entries, APC houses / Ascendant Parallel Circle / WvA aliases, Horizon / Horizontal / Azimuthal aliases, the Vehlow-equal source label, the Bob Makransky source label for Sunshine, the Topocentric house system alias, the baseline Placidus and Koch table-of-houses source spellings, the remaining Albategnius / Pullen / Gauquelin source labels, the Swiss Ephemeris single-letter house-table codes P/K/R/C/O/E/W/N/V/A/H/B/M/S/I/G plus the additional T/U/X/Y interoperability codes resolving to their corresponding built-ins, and the exact Swiss Ephemeris house-table code spellings A equal, D equal / MC, E equal = A, N whole sign houses, 1. house = Aries, S sripati, I sunshine, W equal, whole sign, V equal Vehlow, T topocentric, U Krusinski-Pisa-Goelzer, X axial rotation system/ Meridian houses, and Y APC houses.",
            "The compatibility profile now also renders a source-label appendix for the built-in house systems so common Placidus, Koch, Equal, Whole Sign, Topocentric, Vehlow, Meridian, ARMC, Sunshine, APC, and Horizon/Azimuth spellings — including the Swiss Ephemeris \"Equal (cusp 1 = Asc)\" and \"Whole Sign (house 1 = Aries)\" forms — are searchable alongside the ayanamsa appendix, and the latest release-specific house-system label batches now also surface the exact Placidus table of houses, Koch table of houses, Albategnius, Pullen, and Gauquelin search forms.",
            "The compatibility profile now also surfaces the exact Swiss Ephemeris house-table code spellings A equal, D equal / MC, E equal = A, N whole sign houses, 1. house = Aries, S sripati, I sunshine, W equal, whole sign, V equal Vehlow, T topocentric, U Krusinski-Pisa-Goelzer, X axial rotation system/ Meridian houses, and Y APC houses so the code-style interoperability forms remain searchable alongside the canonical house names.",
            "The compatibility profile now also renders source-label appendix entries for Lahiri / Chitrapaksha / Chitra Paksha, True Chitra / Chitra, Krishnamurti Ayanamsha / Krishnamurti (Swiss) / Krishnamurti Paddhati / KP ayanamsa, Fagan/Bradley Ayanamsha / Fagan/Bradley / Fagan Bradley / Fagan-Bradley, Usha Shashi, and the Yukteshwar / Sri Yukteshwar / Shri Yukteshwar transliterations so the baseline sidereal spellings remain searchable alongside the existing Raman appendix entry and the rest of the ayanamsa catalog.",
            "The compatibility profile now also renders source-label appendix entries for P.V.R. Narasimha Rao, Aries houses, and True Mula (Chandra Hari) so the release-facing interoperability labels stay aligned with the documented source spellings for the Pushya-paksha, equal-house, and true-Mula variants.",
            "The compatibility profile now also renders a source-label appendix entry for Raman so the B. V. Raman, B.V. Raman, and B V Raman spellings are searchable alongside the other baseline ayanamsa labels.",
            "The True Citra entry now also accepts the True Citra Paksha and True Chitrapaksha spellings, and the release profile summary highlights that alias batch explicitly so the release-facing source-label appendix stays aligned with common interoperability wording.",
            "Release-specific ayanamsa additions now include J2000, J1900, B1950, True Citra, DeLuce, Yukteshwar (including the Sri Yukteshwar / Shri Yukteswar / Shri Yukteshwar transliterations), PVR Pushya-paksha, Sheoran, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Pushya, Udayagiri, Lahiri (VP285), Krishnamurti (VP291), Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), True Sheoran, Galactic Center, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), and Valens Moon, with explicit zero-point metadata now published for Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Britton), Udayagiri, Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Galactic Center, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Galactic Center (Cochrane), JN Bhasin, Babylonian (Eta Piscium), Babylonian (Aldebaran), Galactic Equator (Mula), Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Aryabhata (522 CE), and the true/modern Galactic Equator entries; the Babylonian house-family source labels now resolve as exact aliases too, Galactic Equator (Fiorenza) continues to carry a J2000.0 reference epoch and 25° zero-point offset for the release profile, the Babylonian house-family labels now render in a separate custom-definition section, and the plain Moon alias also resolves to Valens Moon for compatibility with existing label variants, while the Valens Moon source-label appendix now also includes the Valens, Moon, and Moon sign ayanamsa source spellings, the release profile now surfaces the Aryabhata 499/522 and Surya Siddhanta / Suryasiddhanta 499/499 CE source spellings explicitly, and the release-facing source-label appendix now also calls out the Babylonian 1/2/3 shorthand labels, Babylonian Huber, Aryabhatan Kaliyuga / Aryabhata Kaliyuga spellings, Fagan/Bradley Ayanamsha / Fagan/Bradley spellings, Krishnamurti Ayanamsha / Krishnamurti (Swiss) search forms, and the Usha Shashi search forms explicitly, alongside the new Lahiri / Chitrapaksha and True Chitra / Chitra appendix entries.",
            "Non-standard ayanamsa labels such as True Balarama, Aphoric, and Takra are intentionally treated as custom definitions until a documented source mapping is added.",
            "The compatibility profile is intended to be archived with release validation outputs and release notes.",
        ],
        validation_reference_points: &[
            "The stage-4 validation corpus remains the reference point for tightening house formulas whenever future revisions land.",
        ],
        custom_definition_labels: &[
            "Babylonian (House)",
            "Babylonian (Sissy)",
            "Babylonian (True Geoc)",
            "Babylonian (True Topc)",
            "Babylonian (True Obs)",
            "Babylonian (House Obs)",
            "True Balarama",
            "Aphoric",
            "Takra",
        ],
        known_gaps: &[
            "The newly added historical/reference-frame and formula-variant ayanamsa modes are catalogued and resolvable, and the release line now publishes explicit sidereal metadata for Babylonian (Huber), Babylonian (Britton), Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Galactic Center (Cochrane), Galactic Center (Mardyks), Galactic Center (Rgilbrand), Galactic Center (Mula/Wilhelm), Galactic Equator (IAU 1958), Galactic Equator (Fiorenza), Suryasiddhanta (Revati), Suryasiddhanta (Citra), True Pushya, True Sheoran, Udayagiri, Lahiri (VP285), Krishnamurti (VP291), Djwhal Khul, Valens Moon, and the remaining historical/reference-frame catalog entries; additional metadata/source mapping work remains scheduled for any unreconciled future breadth batches or custom definitions.",
            "Labels outside the published compatibility profile, including ad hoc names such as True Balarama, Aphoric, and Takra, should be modeled as custom ayanamsa definitions rather than assumed to be built-ins.",
        ],
    }
}

fn write_scope_section(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    lines: &[&'static str],
) -> fmt::Result {
    writeln!(f, "{}", title)?;
    for line in lines {
        writeln!(f, "- {}", line)?;
    }
    Ok(())
}

trait AliasProfileEntry {
    fn canonical_name(&self) -> &'static str;
    fn aliases(&self) -> &'static [&'static str];
}

impl AliasProfileEntry for HouseSystemDescriptor {
    fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }
}

impl AliasProfileEntry for AyanamsaDescriptor {
    fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    fn aliases(&self) -> &'static [&'static str] {
        self.aliases
    }
}

fn write_alias_section<T: AliasProfileEntry>(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    entries: &[T],
) -> fmt::Result {
    let mut has_aliases = false;
    for entry in entries {
        if !entry.aliases().is_empty() {
            has_aliases = true;
            break;
        }
    }

    if !has_aliases {
        return Ok(());
    }

    writeln!(f, "{}", title)?;
    for entry in entries {
        if entry.aliases().is_empty() {
            continue;
        }

        writeln!(
            f,
            "- {} -> {}",
            entry.aliases().join(", "),
            entry.canonical_name()
        )?;
    }
    Ok(())
}

fn house_source_label_aliases(canonical_name: &str) -> &'static [&'static str] {
    match canonical_name {
        "Placidus" => &["Placidus house system", "Placidus table of houses"],
        "Koch" => &[
            "Koch house system",
            "Koch table of houses",
            "W. Koch",
            "W Koch",
        ],
        "Porphyry" => &["Equal Quadrant"],
        "Regiomontanus" => &["Regiomontanus houses"],
        "Campanus" => &["Campanus houses"],
        "Equal" => &[
            "A equal",
            "E equal = A",
            "Equal houses",
            "Equal house system",
            "Equal House",
            "Equal table of houses",
            "Wang",
            "Equal (cusp 1 = Asc)",
        ],
        "Whole Sign" => &[
            "W equal, whole sign",
            "Whole Sign houses",
            "Whole Sign table of houses",
            "Whole-sign",
            "Whole Sign system",
        ],
        "Alcabitius" => &["Alcabitius houses"],
        "Meridian" => &[
            "X",
            "Meridian houses",
            "Meridian table of houses",
            "ARMC",
            "X axial rotation system/ Meridian houses",
        ],
        "Axial" => &["Axial variants"],
        "Topocentric" => &[
            "T",
            "Polich-Page",
            "Polich/Page",
            "Polich Page",
            "Polich-Page \"topocentric\" table of houses",
            "T Polich/Page (\"topocentric\")",
            "T topocentric",
            "Topocentric house system",
        ],
        "Morinus" => &["Morinus houses"],
        "Equal (MC)" => &[
            "D equal / MC",
            "Equal from MC",
            "Equal (from MC)",
            "Equal (from MC) table of houses",
            "Equal MC",
            "Equal Midheaven",
            "Equal/MC = 10th",
        ],
        "Equal (1=Aries)" => &[
            "N",
            "Equal/1=Aries",
            "Equal Aries",
            "Aries houses",
            "Whole Sign (house 1 = Aries)",
            "Whole Sign (house 1 = Aries) table of houses",
            "Whole sign houses, 1. house = Aries",
            "Equal/1=0 Aries",
            "Equal (cusp 1 = 0° Aries)",
        ],
        "Vehlow Equal" => &[
            "V equal Vehlow",
            "Vehlow-equal table of houses",
            "Vehlow-equal",
            "Vehlow",
            "Vehlow equal",
        ],
        "Sripati" => &["S", "Śrīpati"],
        "Carter (poli-equatorial)" => &[
            "Carter",
            "Carter's poli-equatorial table of houses",
            "Poli-Equatorial",
            "Poli-equatorial",
        ],
        "Horizon/Azimuth" => &[
            "Horizon/Azimuth",
            "Horizon",
            "Azimuth",
            "Horizontal",
            "Azimuthal",
            "Horizon/Azimuth table of houses",
            "Horizon house system",
            "Azimuthal house system",
            "horizon/azimut",
            "horizon/azimuth",
        ],
        "APC" => &[
            "Y",
            "APC",
            "Ram school",
            "Ram's school",
            "Ramschool",
            "WvA",
            "Y APC houses",
            "APC houses",
            "APC, also known as “Ram school”, table of houses",
            "APC house system",
            "Ascendant Parallel Circle",
        ],
        "Krusinski-Pisa-Goelzer" => &[
            "U",
            "Krusinski",
            "Krusinski-Pisa",
            "Krusinski Pisa",
            "Krusinski/Pisa/Goelzer",
            "Krusinski-Pisa-Goelzer table of houses",
            "U krusinski-pisa-goelzer",
            "Krusinski/Pisa/Goelzer house system",
            "Pisa-Goelzer",
        ],
        "Albategnius" => &[
            "Albategnius table of houses",
            "Savard-A",
            "Savard A",
            "Savard's Albategnius",
        ],
        "Pullen SD" => &[
            "Pullen SD (Neo-Porphyry) table of houses",
            "Neo-Porphyry",
            "Pullen sinusoidal delta",
        ],
        "Pullen SR" => &[
            "Pullen SR (Sinusoidal Ratio) table of houses",
            "Pullen sinusoidal ratio",
        ],
        "Sunshine" => &[
            "I",
            "Sunshine",
            "Sunshine houses",
            "Sunshine house system",
            "Sunshine table of houses, by Bob Makransky",
            "Makransky Sunshine",
            "Bob Makransky",
            "Treindl Sunshine",
        ],
        "Gauquelin sectors" => &[
            "G",
            "Gauquelin",
            "Gauquelin sector",
            "Gauquelin sectors",
            "Gauquelin table of sectors",
        ],
        _ => &[],
    }
}

fn ayanamsa_source_label_aliases(canonical_name: &str) -> &'static [&'static str] {
    match canonical_name {
        "PVR Pushya-paksha" => &[
            "True Pushya (PVRN Rao)",
            "Pushya-paksha",
            "Pushya Paksha",
            "PVR",
            "P.V.R. Narasimha Rao",
        ],
        "Raman" => &["B. V. Raman", "B.V. Raman", "B V Raman"],
        "Krishnamurti" => &["Krishnamurti Ayanamsha", "Krishnamurti (Swiss)"],
        "Fagan/Bradley" => &[
            "Fagan/Bradley Ayanamsha",
            "Fagan/Bradley",
            "Fagan Bradley",
            "Fagan-Bradley",
        ],
        "Lahiri" => &["Chitra Paksha", "Chitrapaksha", "Chitra-paksha"],
        "True Pushya" => &["True Pushya ayanamsa", "Pushya"],
        "True Chitra" => &["Chitra", "True Chitra ayanamsa"],
        "True Citra" => &[
            "True Citra ayanamsa",
            "True Citra Paksha",
            "True Chitra Paksha",
            "True Chitrapaksha",
        ],
        "True Revati" => &["True Revati ayanamsa"],
        "True Mula" => &[
            "True Mula (Chandra Hari)",
            "True Mula ayanamsa",
            "Chandra Hari",
        ],
        "Udayagiri" => &["Udayagiri ayanamsa"],
        "Usha Shashi" => &[
            "Usha Shashi",
            "Ushashashi",
            "Usha-Shashi",
            "Usha/Shashi",
            "Revati",
        ],
        "Lahiri (ICRC)" => &["ICRC Lahiri", "Lahiri ICRC"],
        "Lahiri (1940)" => &["Lahiri original", "Panchanga Darpan Lahiri"],
        "DeLuce" => &["De Luce", "DeLuce ayanamsa"],
        "Yukteshwar" => &[
            "Yukteswar",
            "Sri Yukteswar",
            "Sri Yukteshwar",
            "Shri Yukteswar",
            "Shri Yukteshwar",
        ],
        "J2000" => &["J2000.0"],
        "J1900" => &["J1900.0"],
        "B1950" => &["B1950.0"],
        "Sheoran" => &["Sheoran true", "True Sheoran ayanamsa", "\"Vedic\"/Sheoran"],
        "Djwhal Khul" => &["Djwhal", "Djwhal Khul ayanamsa"],
        "JN Bhasin" => &["J. N. Bhasin", "J.N. Bhasin", "Bhasin"],
        "Suryasiddhanta (Mean Sun)" => &[
            "Suryasiddhanta, mean Sun",
            "Suryasiddhanta mean sun",
            "Suryasiddhanta MSUN",
        ],
        "Suryasiddhanta (499 CE)" => &[
            "Suryasiddhanta",
            "Surya Siddhanta",
            "Suryasiddhanta 499",
            "Surya Siddhanta 499",
            "Suryasiddhanta 499 CE",
            "Surya Siddhanta 499 CE",
        ],
        "Aryabhata (499 CE)" => &[
            "Aryabhata",
            "Aryabhata 499",
            "Aryabhata 499 CE",
            "Aryabhatan Kaliyuga",
            "Aryabhata Kaliyuga",
        ],
        "Aryabhata (Mean Sun)" => &[
            "Aryabhata, mean Sun",
            "Aryabhata mean sun",
            "Aryabhata MSUN",
        ],
        "Aryabhata (522 CE)" => &["Aryabhata 522", "Aryabhata 522 CE"],
        "Babylonian (Kugler 1)" => &["Babylonian/Kugler 1", "Babylonian 1"],
        "Babylonian (Kugler 2)" => &["Babylonian/Kugler 2", "Babylonian 2"],
        "Babylonian (Kugler 3)" => &["Babylonian/Kugler 3", "Babylonian 3"],
        "Babylonian (Britton)" => &["Babylonian/Britton", "Babylonian Britton"],
        "Babylonian (Huber)" => &["Babylonian/Huber", "Babylonian Huber"],
        "Babylonian (Eta Piscium)" => &["Babylonian/Eta Piscium"],
        "Babylonian (Aldebaran)" => &["Babylonian/Aldebaran = 15 Tau"],
        "Lahiri (VP285)" => &["Lahiri VP285", "VP285"],
        "Krishnamurti (VP291)" => &[
            "KP VP291",
            "Krishnamurti VP291",
            "Krishnamurti-Senthilathiban",
            "VP291",
        ],
        "True Sheoran" => &["Sheoran true", "True Sheoran ayanamsa"],
        "Galactic Center (Rgilbrand)" => &[
            "Galactic Center (Gil Brand)",
            "Gil Brand",
            "Rgilbrand",
            "Galactic center Rgilbrand",
        ],
        "Galactic Center (Mardyks)" => &["Skydram (Mardyks)", "Mardyks", "Galactic center Mardyks"],
        "Galactic Center (Mula/Wilhelm)" => &[
            "Dhruva/Gal.Center/Mula (Wilhelm)",
            "Mula Wilhelm",
            "Wilhelm",
            "Galactic center Mula/Wilhelm",
        ],
        "Galactic Center (Cochrane)" => &[
            "Cochrane (Gal.Center = 0 Cap)",
            "Cochrane",
            "Galactic center Cochrane",
        ],
        "Galactic Center" => &["Galact. Center = 0 Sag"],
        "Sassanian" => &["Zij al-Shah"],
        "Galactic Equator (IAU 1958)" => &["Galactic Equator (IAU1958)"],
        "Galactic Equator (True)" => &["True galactic equator"],
        "Galactic Equator (Mula)" => &["Galactic Equator mid-Mula"],
        "Galactic Equator" => &["Gal. Eq."],
        "Galactic Equator (Fiorenza)" => &["Fiorenza"],
        "Valens Moon" => &["Vettius Valens", "Valens", "Moon", "Moon sign ayanamsa"],
        _ => &[],
    }
}

fn write_source_label_section<T, F>(
    f: &mut fmt::Formatter<'_>,
    title: &str,
    entries: &[T],
    source_label_aliases: F,
) -> fmt::Result
where
    T: AliasProfileEntry,
    F: Fn(&str) -> &'static [&'static str],
{
    let mut has_source_labels = false;
    for entry in entries {
        if !source_label_aliases(entry.canonical_name()).is_empty() {
            has_source_labels = true;
            break;
        }
    }

    if !has_source_labels {
        return Ok(());
    }

    writeln!(f, "{}", title)?;
    for entry in entries {
        let source_labels = source_label_aliases(entry.canonical_name());
        if source_labels.is_empty() {
            continue;
        }

        writeln!(
            f,
            "- {} -> {}",
            source_labels.join(", "),
            entry.canonical_name()
        )?;
    }
    Ok(())
}

fn write_custom_definition_section(
    f: &mut fmt::Formatter<'_>,
    labels: &[&'static str],
    descriptors: &[AyanamsaDescriptor],
) -> fmt::Result {
    writeln!(f, "Custom-definition labels:")?;
    for label in labels {
        if let Some(entry) = descriptors
            .iter()
            .find(|entry| entry.canonical_name.eq_ignore_ascii_case(label))
        {
            write!(f, "- {}", entry.canonical_name)?;
            if !entry.aliases.is_empty() {
                write!(f, " (aliases: {})", entry.aliases.join(", "))?;
            }
            if let Some(epoch) = entry.epoch {
                write!(f, " [epoch: {}]", epoch)?;
            }
            if let Some(offset) = entry.offset_degrees {
                write!(f, " [offset: {}]", offset)?;
            }
            writeln!(f, " — {}", entry.notes)?;
        } else {
            writeln!(f, "- {}", label)?;
        }
    }
    Ok(())
}

impl fmt::Display for CompatibilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Compatibility profile: {}", self.profile_id)?;
        writeln!(f, "{}", self.summary)?;
        writeln!(f)?;
        write_scope_section(f, "Target compatibility catalog:", self.target_house_scope)?;
        write_scope_section(f, "Target ayanamsa catalog:", self.target_ayanamsa_scope)?;
        writeln!(f)?;
        writeln!(f, "Baseline compatibility milestone:")?;
        writeln!(f, "House systems:")?;
        for entry in self.baseline_house_systems {
            write!(f, "- {}", entry.canonical_name)?;
            if !entry.aliases.is_empty() {
                write!(f, " (aliases: {})", entry.aliases.join(", "))?;
            }
            if entry.latitude_sensitive {
                write!(f, " [latitude-sensitive]")?;
            }
            writeln!(f, " — {}", entry.notes)?;
        }
        writeln!(f, "Ayanamsas:")?;
        for entry in self.baseline_ayanamsas {
            write!(f, "- {}", entry.canonical_name)?;
            if !entry.aliases.is_empty() {
                write!(f, " (aliases: {})", entry.aliases.join(", "))?;
            }
            if let Some(epoch) = entry.epoch {
                write!(f, " [epoch: {}]", epoch)?;
            }
            if let Some(offset) = entry.offset_degrees {
                write!(f, " [offset: {}]", offset)?;
            }
            writeln!(f, " — {}", entry.notes)?;
        }
        if !self.release_house_systems.is_empty() || !self.release_ayanamsas.is_empty() {
            writeln!(f)?;
            writeln!(f, "Release-specific coverage beyond baseline:")?;
            if !self.release_house_systems.is_empty() {
                writeln!(f, "House systems:")?;
                for entry in self.release_house_systems {
                    write!(f, "- {}", entry.canonical_name)?;
                    if !entry.aliases.is_empty() {
                        write!(f, " (aliases: {})", entry.aliases.join(", "))?;
                    }
                    writeln!(f, " — {}", entry.notes)?;
                }
            }
            if !self.release_ayanamsas.is_empty() {
                writeln!(f, "Ayanamsas:")?;
                for entry in self.release_ayanamsas {
                    write!(f, "- {}", entry.canonical_name)?;
                    if !entry.aliases.is_empty() {
                        write!(f, " (aliases: {})", entry.aliases.join(", "))?;
                    }
                    if let Some(epoch) = entry.epoch {
                        write!(f, " [epoch: {}]", epoch)?;
                    }
                    if let Some(offset) = entry.offset_degrees {
                        write!(f, " [offset: {}]", offset)?;
                    }
                    writeln!(f, " — {}", entry.notes)?;
                }
            }
        }
        if !self.release_notes.is_empty() {
            writeln!(f)?;
            write_scope_section(
                f,
                "Release-specific notes beyond baseline:",
                self.release_notes,
            )?;
        }
        writeln!(f)?;
        let coverage = metadata_coverage();
        writeln!(f, "Coverage summary:")?;
        writeln!(
            f,
            "- house systems: {} total ({} baseline, {} release-specific)",
            self.house_systems.len(),
            self.baseline_house_systems.len(),
            self.release_house_systems.len()
        )?;
        writeln!(
            f,
            "- ayanamsas: {} total ({} baseline, {} release-specific)",
            self.ayanamsas.len(),
            self.baseline_ayanamsas.len(),
            self.release_ayanamsas.len()
        )?;
        writeln!(
            f,
            "- ayanamsa sidereal metadata: {}/{} entries with both a reference epoch and offset",
            coverage.with_sidereal_metadata, coverage.total
        )?;
        if !coverage.custom_definition_only.is_empty() {
            writeln!(
                f,
                "- custom-definition ayanamsas: {} labels are intentionally tracked without sidereal metadata",
                coverage.custom_definition_only.len()
            )?;
        }
        if coverage.is_complete() {
            writeln!(f, "- no unexpected sidereal-metadata gaps remain.")?;
        } else {
            writeln!(
                f,
                "- missing metadata: {}",
                coverage.without_sidereal_metadata.join(", ")
            )?;
        }
        if !self.custom_definition_labels.is_empty() {
            writeln!(
                f,
                "- custom-definition labels: {}",
                self.custom_definition_labels.len()
            )?;
            writeln!(f)?;
            write_custom_definition_section(f, self.custom_definition_labels, self.ayanamsas)?;
        }
        writeln!(f)?;
        write_alias_section(
            f,
            "Alias mappings for built-in house systems:",
            self.house_systems,
        )?;
        writeln!(f)?;
        write_source_label_section(
            f,
            "Source-label aliases for built-in house systems:",
            self.house_systems,
            house_source_label_aliases,
        )?;
        writeln!(f)?;
        write_source_label_section(
            f,
            "Source-label aliases for built-in ayanamsas:",
            self.ayanamsas,
            ayanamsa_source_label_aliases,
        )?;
        writeln!(f)?;
        write_alias_section(f, "Alias mappings for built-in ayanamsas:", self.ayanamsas)?;
        writeln!(f)?;
        write_scope_section(
            f,
            "Validation reference points:",
            self.validation_reference_points,
        )?;
        writeln!(f)?;
        write_scope_section(f, "Compatibility caveats:", self.known_gaps)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_includes_baseline_and_release_catalogs() {
        let profile = current_compatibility_profile();
        assert!(profile
            .house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Equal (MC)"));
        assert!(profile
            .baseline_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Placidus"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Sripati"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Carter (poli-equatorial)"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Horizon/Azimuth"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "APC"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Krusinski-Pisa-Goelzer"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Albategnius"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Pullen SD"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Pullen SR"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Sunshine"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Gauquelin sectors"));
        assert!(profile
            .release_house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Equal (1=Aries)"));
        assert!(profile
            .baseline_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "J2000"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "DeLuce"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Yukteshwar"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "PVR Pushya-paksha"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Sheoran"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "True Revati"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "True Mula"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Suryasiddhanta (Revati)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Suryasiddhanta (Citra)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri (ICRC)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Sassanian"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Hipparchus"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (Kugler 1)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (Kugler 2)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (Aldebaran)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (House)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (Sissy)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (True Geoc)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (True Topc)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (True Obs)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (House Obs)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "True Pushya"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Djwhal Khul"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "JN Bhasin"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Suryasiddhanta (Mean Sun)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Aryabhata (Mean Sun)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Babylonian (Britton)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Aryabhata (522 CE)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri (VP285)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Krishnamurti (VP291)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "True Sheoran"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Center"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Center (Rgilbrand)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Center (Mardyks)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Center (Mula/Wilhelm)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Dhruva Galactic Center (Middle Mula)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Center (Cochrane)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Equator"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Equator (IAU 1958)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Equator (True)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Equator (Mula)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Galactic Equator (Fiorenza)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Valens Moon"));
        assert_eq!(profile.house_systems, built_in_house_systems());
        assert_eq!(profile.baseline_house_systems, baseline_house_systems());
        assert_eq!(profile.release_house_systems, release_house_systems());
        assert_eq!(profile.ayanamsas, built_in_ayanamsas());
        assert_eq!(profile.baseline_ayanamsas, baseline_ayanamsas());
        assert_eq!(profile.release_ayanamsas, release_ayanamsas());
        assert!(profile
            .target_house_scope
            .iter()
            .any(|line| line.contains("Swiss-Ephemeris-class house-system catalog")));
        assert!(profile
            .target_ayanamsa_scope
            .iter()
            .any(|line| line.contains("Swiss-Ephemeris-class ayanamsa catalog")));
        assert!(profile
            .release_notes
            .iter()
            .any(|note| note.contains("Krusinski-Pisa-Goelzer")));
        assert!(profile
            .release_notes
            .iter()
            .any(|note| note.contains("B. V. Raman")));
        assert!(profile
            .release_notes
            .iter()
            .any(|note| note.contains("Whole sign houses, 1. house = Aries")));
        assert!(profile
            .validation_reference_points
            .iter()
            .any(|point| point.contains("validation corpus")));
        assert!(profile
            .validation_reference_points
            .iter()
            .any(|point| point.contains("house formulas")));
        assert!(profile
            .known_gaps
            .iter()
            .all(|gap| !gap.contains("validation corpus")));
        assert!(profile
            .known_gaps
            .iter()
            .all(|gap| !gap.contains("house formulas")));
        assert!(profile
            .custom_definition_labels
            .contains(&"Babylonian (House)"));
        assert!(profile
            .custom_definition_labels
            .contains(&"Babylonian (House Obs)"));
        assert!(profile
            .known_gaps
            .iter()
            .all(|gap| !gap.contains("Babylonian (House)")));
        assert!(profile
            .known_gaps
            .iter()
            .all(|gap| !gap.contains("House Obs")));
    }

    #[test]
    fn display_lists_release_sections() {
        let rendered = current_compatibility_profile().to_string();
        assert!(rendered.contains("Target compatibility catalog:"));
        assert!(rendered.contains("Target ayanamsa catalog:"));
        assert!(rendered.contains("Baseline compatibility milestone:"));
        assert!(rendered.contains("Release-specific coverage beyond baseline:"));
        assert!(rendered.contains("Alias mappings for built-in house systems:"));
        assert!(rendered.contains("Source-label aliases for built-in house systems:"));
        assert!(rendered.contains("Source-label aliases for built-in ayanamsas:"));
        assert!(rendered.contains("Alias mappings for built-in ayanamsas:"));
        assert!(rendered.contains("Coverage summary:"));
        assert!(rendered.contains("house systems:"));
        assert!(rendered.contains("ayanamsas:"));
        assert!(rendered.contains("ayanamsa sidereal metadata:"));
        assert!(rendered.contains("custom-definition ayanamsas:"));
        assert!(rendered.contains("no unexpected sidereal-metadata gaps remain."));
        assert!(rendered.contains("custom-definition labels:"));
        assert!(rendered.contains("Validation reference points:"));
        assert!(rendered.contains("The stage-4 validation corpus remains the reference point for tightening house formulas whenever future revisions land."));
        assert!(rendered.contains("Babylonian (House) (aliases: Babylonian House, BABYL_HOUSE)"));
        assert!(rendered
            .contains("Babylonian (House Obs) (aliases: Babylonian House Obs, BABYL_HOUSE_OBS)"));
        assert!(
            rendered.contains("Babylonian sidereal mode labeled BABYL_HOUSE in Swiss Ephemeris.")
        );
        assert!(rendered
            .contains("Babylonian sidereal mode labeled BABYL_HOUSE_OBS in Swiss Ephemeris."));
        assert!(rendered.contains("Babylonian/Kugler 1, Babylonian 1 -> Babylonian (Kugler 1)"));
        assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal MC, Equal Midheaven, Equal (MC), Equal/MC = 10th -> Equal (MC)"));
        assert!(rendered.contains(
            "N, Equal/1=Aries, Equal Aries, Aries houses, Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"
        ));
        assert!(rendered.contains(
            "Galactic Center (Gil Brand), Gil Brand, Rgilbrand, Galactic center Rgilbrand -> Galactic Center (Rgilbrand)"
        ));
        assert!(rendered.contains("Galact. Center = 0 Sag -> Galactic Center"));
        assert!(rendered.contains("Gal. Eq. -> Galactic Equator"));
        assert!(rendered.contains("Zij al-Shah -> Sassanian"));
        assert!(
            rendered.contains("Vettius Valens, Valens, Moon, Moon sign ayanamsa -> Valens Moon")
        );
        assert!(rendered.contains("Suryasiddhanta, mean Sun"));
        assert!(rendered.contains(
            "Suryasiddhanta, Surya Siddhanta, Suryasiddhanta 499, Surya Siddhanta 499, Suryasiddhanta 499 CE, Surya Siddhanta 499 CE -> Suryasiddhanta (499 CE)"
        ));
        assert!(rendered.contains(
            "Aryabhata, Aryabhata 499, Aryabhata 499 CE, Aryabhatan Kaliyuga, Aryabhata Kaliyuga -> Aryabhata (499 CE)"
        ));
        assert!(rendered.contains("Aryabhata 522, Aryabhata 522 CE -> Aryabhata (522 CE)"));
        assert!(rendered.contains("Suryasiddhanta MSUN -> Suryasiddhanta (Mean Sun)"));
        assert!(rendered.contains("J. N. Bhasin, J.N. Bhasin, Bhasin -> JN Bhasin"));
        assert!(rendered.contains("Lahiri VP285, VP285 -> Lahiri (VP285)"));
        assert!(rendered.contains("KP VP291, Krishnamurti VP291, Krishnamurti-Senthilathiban, VP291 -> Krishnamurti (VP291)"));
        assert!(rendered.contains(
            "True Pushya (PVRN Rao), Pushya-paksha, Pushya Paksha, PVR, P.V.R. Narasimha Rao -> PVR Pushya-paksha"
        ));
        assert!(rendered.contains("True Pushya ayanamsa, Pushya -> True Pushya"));
        assert!(rendered
            .contains("True Citra ayanamsa, True Citra Paksha, True Chitra Paksha, True Chitrapaksha -> True Citra"));
        assert!(rendered.contains("Chitra, True Chitra ayanamsa -> True Chitra"));
        assert!(rendered.contains("True Revati ayanamsa -> True Revati"));
        assert!(rendered
            .contains("True Mula (Chandra Hari), True Mula ayanamsa, Chandra Hari -> True Mula"));
        assert!(rendered.contains("Udayagiri ayanamsa -> Udayagiri"));
        assert!(rendered.contains("ICRC Lahiri, Lahiri ICRC -> Lahiri (ICRC)"));
        assert!(rendered.contains("Lahiri original, Panchanga Darpan Lahiri -> Lahiri (1940)"));
        assert!(rendered.contains("De Luce, DeLuce ayanamsa -> DeLuce"));
        assert!(rendered.contains("Yukteswar, Sri Yukteswar, Sri Yukteshwar, Shri Yukteswar, Shri Yukteshwar -> Yukteshwar"));
        assert!(rendered.contains(
            "T, Polich-Page, Polich/Page, Polich Page, Polich-Page \"topocentric\" table of houses, T Polich/Page (\"topocentric\"), T topocentric, Topocentric house system -> Topocentric"
        ));
        assert!(rendered
            .contains("Horizontal, Azimuthal, Horizon house system, Azimuthal house system"));
        assert!(rendered.contains(
            "X, Meridian houses, Meridian table of houses, ARMC, X axial rotation system/ Meridian houses -> Meridian"
        ));
        assert!(rendered.contains("horizon/azimuth"));
        assert!(rendered
            .contains("Y, APC, Ram school, Ram's school, Ramschool, WvA, Y APC houses, APC houses, APC, also known as “Ram school”, table of houses, APC house system, Ascendant Parallel Circle -> APC"));
        assert!(rendered.contains("Chitra Paksha, Chitrapaksha, Chitra-paksha -> Lahiri"));
        assert!(rendered.contains("source-label appendix entries for Lahiri / Chitrapaksha / Chitra Paksha, True Chitra / Chitra, Krishnamurti Ayanamsha / Krishnamurti (Swiss) / Krishnamurti Paddhati / KP ayanamsa, Fagan/Bradley Ayanamsha / Fagan/Bradley / Fagan Bradley / Fagan-Bradley, Usha Shashi, and the Yukteshwar / Sri Yukteshwar / Shri Yukteshwar transliterations"));
        assert!(rendered.contains("source-label appendix entries for P.V.R. Narasimha Rao, Aries houses, and True Mula (Chandra Hari)"));
        assert!(rendered.contains("B. V. Raman, B.V. Raman, B V Raman -> Raman"));
        assert!(rendered.contains(
            "KP, Krishnamurti Ayanamsha, Krishnamurti (Swiss), Krishnamurti Paddhati, KP ayanamsa -> Krishnamurti"
        ));
        assert!(rendered.contains("Krishnamurti Ayanamsha, Krishnamurti (Swiss) -> Krishnamurti"));
        assert!(rendered.contains(
            "Fagan/Bradley Ayanamsha, Fagan/Bradley, Fagan Bradley, Fagan-Bradley -> Fagan/Bradley"
        ));
        assert!(rendered
            .contains("Usha Shashi, Ushashashi, Usha-Shashi, Usha/Shashi, Revati -> Usha Shashi"));
        assert!(rendered.contains("Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"));
        assert!(rendered.contains(
            "A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"
        ));
        let source_label_section = rendered
            .split("Source-label aliases for built-in house systems:")
            .nth(1)
            .expect("source-label house appendix should be present");
        assert!(source_label_section.contains(
            "A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"
        ));
        assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal MC, Equal Midheaven, Equal (MC), Equal/MC = 10th -> Equal (MC)"));
        assert!(rendered.contains("J2000.0 -> J2000"));
        assert!(rendered.contains("J1900.0 -> J1900"));
        assert!(rendered.contains("B1950.0 -> B1950"));
        assert!(
            rendered.contains("Vettius Valens, Valens, Moon, Moon sign ayanamsa -> Valens Moon")
        );
        assert!(rendered.contains("Equal (MC)"));
        assert!(rendered.contains("Equal (1=Aries)"));
        assert!(rendered.contains("N, Equal/1=Aries, Equal Aries, Aries houses, Whole Sign (house 1 = Aries), Whole Sign (house 1 = Aries) table of houses, Whole sign houses, 1. house = Aries, Equal/1=0 Aries, Equal (cusp 1 = 0° Aries) -> Equal (1=Aries)"));
        assert!(
            rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow-equal, Vehlow-equal table of houses -> Vehlow Equal")
        );
        assert!(rendered.contains("Sripati"));
        assert!(rendered.contains("Carter (poli-equatorial)"));
        assert!(rendered.contains("Horizon/Azimuth"));
        assert!(rendered.contains("APC"));
        assert!(rendered.contains("Krusinski-Pisa-Goelzer"));
        assert!(rendered.contains("U, Krusinski, Krusinski-Pisa, Krusinski Pisa, Krusinski/Pisa/Goelzer, Krusinski-Pisa-Goelzer table of houses, U krusinski-pisa-goelzer, Krusinski/Pisa/Goelzer house system, Pisa-Goelzer -> Krusinski-Pisa-Goelzer"));
        assert!(rendered.contains("Albategnius"));
        assert!(rendered.contains("Savard-A, Savard A, Savard's Albategnius -> Albategnius"));
        assert!(rendered.contains("Pullen SD"));
        assert!(rendered.contains("Neo-Porphyry, Pullen sinusoidal delta -> Pullen SD"));
        assert!(rendered.contains("Pullen SR"));
        assert!(rendered.contains("Pullen sinusoidal ratio -> Pullen SR"));
        assert!(rendered.contains("Babylonian/Kugler 1, Babylonian 1 -> Babylonian (Kugler 1)"));
        assert!(rendered.contains("Babylonian/Kugler 2, Babylonian 2 -> Babylonian (Kugler 2)"));
        assert!(rendered.contains("Babylonian/Kugler 3, Babylonian 3 -> Babylonian (Kugler 3)"));
        assert!(rendered.contains("Babylonian/Huber, Babylonian Huber -> Babylonian (Huber)"));
        assert!(rendered.contains(
            "Aryabhata, Aryabhata 499, Aryabhata 499 CE, Aryabhatan Kaliyuga, Aryabhata Kaliyuga -> Aryabhata (499 CE)"
        ));
        assert!(rendered.contains(
            "I, Sunshine, Sunshine houses, Sunshine house system, Sunshine table of houses, by Bob Makransky, Makransky Sunshine, Bob Makransky, Treindl Sunshine -> Sunshine"
        ));
        assert!(rendered.contains("S, Śrīpati -> Sripati"));
        assert!(rendered.contains("P/K/R/C/O/E/W/N/V/A/H/B/M/S/I/G"));
        assert!(rendered.contains("plus the additional T/U/X/Y interoperability codes"));
        assert!(rendered.contains("A equal, E equal = A, Equal houses, Equal house system, Equal House, Equal table of houses, Wang, Equal (cusp 1 = Asc) -> Equal"));
        assert!(rendered.contains("D equal / MC, Equal from MC, Equal (from MC), Equal (from MC) table of houses, Equal MC, Equal Midheaven, Equal (MC), Equal/MC = 10th -> Equal (MC)"));
        assert!(rendered.contains(
            "W equal, whole sign, Whole Sign houses, Whole Sign table of houses, Whole-sign, Whole Sign system -> Whole Sign"
        ));
        assert!(
            rendered.contains("V equal Vehlow, Vehlow, Vehlow equal, Vehlow-equal, Vehlow-equal table of houses -> Vehlow Equal")
        );
        assert!(rendered.contains(
            "X, Meridian houses, Meridian table of houses, ARMC, X axial rotation system/ Meridian houses -> Meridian"
        ));
        assert!(rendered.contains("Y, APC, Ram school, Ram's school, Ramschool, WvA, Y APC houses, APC houses, APC, also known as “Ram school”, table of houses, APC house system, Ascendant Parallel Circle -> APC"));
        assert!(rendered.contains("T, Polich-Page, Polich/Page, Polich Page, Polich-Page \"topocentric\" table of houses, T Polich/Page (\"topocentric\"), T topocentric, Topocentric house system -> Topocentric"));
        assert!(rendered.contains("Gauquelin sectors"));
        assert!(rendered
            .contains("G, Gauquelin, Gauquelin sector, Gauquelin sectors, Gauquelin table of sectors -> Gauquelin sectors"));
        assert!(rendered.contains("J2000"));
        assert!(rendered.contains("DeLuce"));
        assert!(rendered.contains("Yukteshwar"));
        assert!(rendered.contains("PVR Pushya-paksha"));
        assert!(rendered.contains("Sheoran"));
        assert!(rendered.contains("True Revati"));
        assert!(rendered.contains("True Mula"));
        assert!(rendered.contains("Suryasiddhanta (Revati)"));
        assert!(rendered.contains("Suryasiddhanta (Citra)"));
        assert!(rendered.contains("Lahiri (ICRC)"));
        assert!(rendered.contains("Sassanian"));
        assert!(rendered.contains("Hipparchus"));
        assert!(rendered.contains("Babylonian (Kugler 1)"));
        assert!(rendered.contains("Babylonian (Aldebaran)"));
        assert!(rendered.contains("Babylonian (House)"));
        assert!(rendered.contains("Babylonian (Sissy)"));
        assert!(rendered.contains("Babylonian (True Geoc)"));
        assert!(rendered.contains("Babylonian (True Topc)"));
        assert!(rendered.contains("Babylonian (True Obs)"));
        assert!(rendered.contains("Babylonian (House Obs)"));
        assert!(rendered.contains("Galactic Center"));
        assert!(rendered.contains("Dhruva Galactic Center (Middle Mula)"));
        assert!(rendered.contains("Galactic Equator"));
        assert!(rendered.contains("Compatibility caveats:"));
        assert!(rendered.contains("Placidus house system, Placidus table of houses -> Placidus"));
        assert!(rendered.contains("Equal (cusp 1 = Asc) -> Equal"));
        assert!(
            rendered.contains("Koch house system, Koch table of houses, W. Koch, W Koch -> Koch")
        );
        assert!(rendered.contains("Lahiri"));
        assert!(rendered.contains("Custom-definition labels:"));
        assert!(rendered.contains("- True Balarama"));
        assert!(rendered.contains("- Aphoric"));
        assert!(rendered.contains("- Takra"));
        assert!(rendered.contains("custom definitions"));
        assert!(rendered.contains("house systems: 25 total"));
        assert!(rendered.contains("ayanamsas: 59 total"));
    }
}
