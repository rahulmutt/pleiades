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
    /// Explicitly documented gaps that remain for later stages.
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
        profile_id: "pleiades-compatibility-profile/0.6.9",
        summary: "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit, including the release-specific house-system additions across the Carter, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen, Sunshine, and Gauquelin families, plus the expanded ayanamsa coverage for J2000/J1900/B1950, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, the true-nakshatra and Suryasiddhanta Revati/Citra reference modes, the Hipparchus/Babylonian/Galactic reference-frame modes, the latest True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, mean-sun, VP285/VP291, Valens Moon, Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), the Babylonian house/sissy/true-geoc/true-topc/true-obs/house-obs variants, and additional Galactic Equator/Center variants.",

        target_house_scope: &[
            "Target house scope: the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal.",
            "Baseline milestone: Placidus, Koch, Porphyry, Regiomontanus, Campanus, Equal, Whole Sign, Alcabitius, Meridian/Axial variants, Topocentric, and Morinus are shipped today.",
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
            "Release-specific house-system additions now include Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD, Pullen SR, Sunshine, and Gauquelin sectors, with the Whole Sign (house 1 = Aries) label and Wang alias also resolving as interoperability aliases.",
            "Release-specific ayanamsa additions now include J2000, J1900, B1950, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, True Revati, True Mula, Suryasiddhanta (Revati), Suryasiddhanta (Citra), Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, Hipparchus, Babylonian (Kugler 1), Babylonian (Kugler 2), Babylonian (Kugler 3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), Babylonian (House Obs), True Pushya, Udayagiri, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, Valens Moon, Galactic Center (Rgilbrand), Galactic Center (Mardyks), Galactic Center (Mula/Wilhelm), Dhruva Galactic Center (Middle Mula), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Galactic Equator (True), Galactic Equator (Mula), Galactic Equator (Fiorenza), Galactic Center, and Galactic Equator, and the plain Moon alias also resolves to Valens Moon for compatibility with existing label variants.",
            "Non-standard ayanamsa labels such as True Balarama, Aphoric, and Takra are intentionally treated as custom definitions until a documented source mapping is added.",
            "The compatibility profile is intended to be archived with release validation outputs and release notes.",
        ],
        known_gaps: &[
            "Stage 4 validation against external reference data is still the next source of accuracy tightening for house formulas.",
            "Additional Swiss Ephemeris ayanamsa modes remain scheduled for future release-breadth batches, even after adding the Hipparchus, Babylonian house/sissy/true-geoc/true-topc/true-obs/house-obs, Galactic, True Pushya, Djwhal Khul, JN Bhasin, mean-sun, and VP285/VP291 families to the catalog.",
            "The newly added historical/reference-frame and formula-variant ayanamsa modes are catalogued and resolvable, but most do not yet carry sidereal offset metadata for chart-layer conversion; Babylonian (Huber), Galactic Center (Cochrane), Galactic Equator (IAU 1958), Suryasiddhanta (Revati), Suryasiddhanta (Citra), True Pushya, Djwhal Khul, Sheoran, and Valens Moon now do.",
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
        writeln!(f, "Ayanamsa sidereal metadata coverage:")?;
        writeln!(
            f,
            "- entries with both a reference epoch and offset: {}/{}",
            coverage.with_sidereal_metadata, coverage.total
        )?;
        if coverage.is_complete() {
            writeln!(
                f,
                "- all built-in ayanamsas can participate in chart-layer sidereal conversion."
            )?;
        } else {
            writeln!(
                f,
                "- missing metadata: {}",
                coverage.without_sidereal_metadata.join(", ")
            )?;
        }
        writeln!(f)?;
        write_alias_section(
            f,
            "Alias mappings for built-in house systems:",
            self.house_systems,
        )?;
        writeln!(f)?;
        write_alias_section(f, "Alias mappings for built-in ayanamsas:", self.ayanamsas)?;
        writeln!(f)?;
        write_scope_section(f, "Known gaps:", self.known_gaps)?;
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
            .known_gaps
            .iter()
            .any(|gap| gap.contains("validation")));
    }

    #[test]
    fn display_lists_release_sections() {
        let rendered = current_compatibility_profile().to_string();
        assert!(rendered.contains("Target compatibility catalog:"));
        assert!(rendered.contains("Target ayanamsa catalog:"));
        assert!(rendered.contains("Baseline compatibility milestone:"));
        assert!(rendered.contains("Release-specific coverage beyond baseline:"));
        assert!(rendered.contains("Alias mappings for built-in house systems:"));
        assert!(rendered.contains("Alias mappings for built-in ayanamsas:"));
        assert!(rendered.contains("Ayanamsa sidereal metadata coverage:"));
        assert!(rendered.contains("entries with both a reference epoch and offset"));
        assert!(rendered.contains("missing metadata:"));
        assert!(rendered.contains("Polich-Page, Polich Page -> Topocentric"));
        assert!(rendered.contains("Chitrapaksha -> Lahiri"));
        assert!(rendered.contains("Whole Sign (house 1 = Aries) -> Equal (1=Aries)"));
        assert!(rendered.contains("Wang -> Equal"));
        assert!(rendered.contains("Valens, Moon, Moon sign ayanamsa -> Valens Moon"));
        assert!(rendered.contains("Equal (MC)"));
        assert!(rendered.contains("Equal (1=Aries)"));
        assert!(rendered.contains("Vehlow Equal"));
        assert!(rendered.contains("Sripati"));
        assert!(rendered.contains("Carter (poli-equatorial)"));
        assert!(rendered.contains("Horizon/Azimuth"));
        assert!(rendered.contains("APC"));
        assert!(rendered.contains("Krusinski-Pisa-Goelzer"));
        assert!(rendered.contains("Albategnius"));
        assert!(rendered.contains("Pullen SD"));
        assert!(rendered.contains("Pullen SR"));
        assert!(rendered.contains("Sunshine"));
        assert!(rendered.contains("Gauquelin sectors"));
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
        assert!(rendered.contains("Known gaps:"));
        assert!(rendered.contains("Placidus"));
        assert!(rendered.contains("Lahiri"));
        assert!(rendered.contains("True Balarama"));
        assert!(rendered.contains("custom definitions"));
    }
}
