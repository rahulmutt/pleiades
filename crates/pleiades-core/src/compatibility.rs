//! Versioned compatibility profile for the current release line.
//!
//! The profile is intentionally explicit about what the repository ships today
//! versus what remains for later stages. It can be printed by the CLI and used
//! in documentation or release notes so consumers know which built-ins and
//! aliases are actually available.

#![forbid(unsafe_code)]

use core::fmt;

use pleiades_ayanamsa::{baseline_ayanamsas, AyanamsaDescriptor};
use pleiades_houses::{baseline_house_systems, HouseSystemDescriptor};

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
    /// Built-in ayanamsas shipped in this release line.
    pub ayanamsas: &'static [AyanamsaDescriptor],
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
        profile_id: "pleiades-compatibility-profile/0.2.0",
        summary: "Stage 6 release profile: the baseline catalogs remain published as a routine release artifact while the target Swiss-Ephemeris-class compatibility catalog stays explicit.",
        target_house_scope: &[
            "Target house scope: the full Swiss-Ephemeris-class house-system catalog remains the long-term compatibility goal.",
            "Baseline milestone: Placidus, Koch, Porphyry, Regiomontanus, Campanus, Equal, Whole Sign, Alcabitius, Meridian/Axial variants, Topocentric, and Morinus are shipped today.",
        ],
        target_ayanamsa_scope: &[
            "Target ayanamsa scope: the full Swiss-Ephemeris-class ayanamsa catalog remains the long-term compatibility goal.",
            "Baseline milestone: Lahiri, Raman, Krishnamurti, Fagan/Bradley, True Chitra, and documented aliases/custom variants are shipped today.",
        ],
        house_systems: baseline_house_systems(),
        ayanamsas: baseline_ayanamsas(),
        release_notes: &[
            "No additional catalog breadth beyond the baseline milestone is claimed in this release line yet.",
            "The compatibility profile is intended to be archived with release validation outputs and release notes.",
        ],
        known_gaps: &[
            "Stage 4 validation against external reference data is still the next source of accuracy tightening for house formulas.",
            "Later stages will continue to expand catalog breadth, packaged data, and release hardening.",
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
        for entry in self.house_systems {
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
        for entry in self.ayanamsas {
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
        writeln!(f)?;
        write_scope_section(
            f,
            "Release-specific coverage beyond baseline:",
            self.release_notes,
        )?;
        writeln!(f)?;
        write_scope_section(f, "Known gaps:", self.known_gaps)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_includes_baseline_catalogs() {
        let profile = current_compatibility_profile();
        assert!(profile
            .house_systems
            .iter()
            .any(|entry| entry.canonical_name == "Placidus"));
        assert!(profile
            .ayanamsas
            .iter()
            .any(|entry| entry.canonical_name == "Lahiri"));
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
            .any(|note| note.contains("release notes")));
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
        assert!(rendered.contains("Known gaps:"));
        assert!(rendered.contains("Placidus"));
        assert!(rendered.contains("Lahiri"));
    }
}
