//! Versioned compatibility profile for the current release line.
//!
//! The profile is intentionally explicit about what the repository ships today
//! versus what remains for later stages. It can be printed by the CLI and used
//! in documentation or release notes so consumers know which built-ins and
//! aliases are actually available.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_ayanamsa::{
    baseline_ayanamsas, built_in_ayanamsas, release_ayanamsas, AyanamsaDescriptor,
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
        profile_id: "pleiades-compatibility-profile/0.5.0",
        summary: "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit, including the first release-specific house-system additions, Carter (poli-equatorial), Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, historical ayanamsa anchor variants, and fixed zodiac-sign house coverage.",
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
            "Release-specific house-system additions now include Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter (poli-equatorial), Horizon/Azimuth, APC, and Krusinski-Pisa-Goelzer.",
            "Release-specific ayanamsa additions now include Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), and Sassanian.",
            "The compatibility profile is intended to be archived with release validation outputs and release notes.",
        ],
        known_gaps: &[
            "Stage 4 validation against external reference data is still the next source of accuracy tightening for house formulas.",
            "Remaining house-system breadth still includes the sinusoidal, Albategnius, Sunshine, and Gauquelin-sector families called out in the stage-6 plan.",
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
            .any(|entry| entry.canonical_name == "Equal (1=Aries)"));
        assert!(profile
            .baseline_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri (ICRC)"));
        assert!(profile
            .release_ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Sassanian"));
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
        assert!(rendered.contains("Equal (MC)"));
        assert!(rendered.contains("Equal (1=Aries)"));
        assert!(rendered.contains("Vehlow Equal"));
        assert!(rendered.contains("Sripati"));
        assert!(rendered.contains("Carter (poli-equatorial)"));
        assert!(rendered.contains("Horizon/Azimuth"));
        assert!(rendered.contains("APC"));
        assert!(rendered.contains("Krusinski-Pisa-Goelzer"));
        assert!(rendered.contains("Lahiri (ICRC)"));
        assert!(rendered.contains("Sassanian"));
        assert!(rendered.contains("Known gaps:"));
        assert!(rendered.contains("Placidus"));
        assert!(rendered.contains("Lahiri"));
    }
}
