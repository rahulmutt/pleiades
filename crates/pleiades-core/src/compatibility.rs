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
    /// Human-readable summary of the current stage.
    pub summary: &'static str,
    /// Built-in house systems shipped in this release line.
    pub house_systems: &'static [HouseSystemDescriptor],
    /// Built-in ayanamsas shipped in this release line.
    pub ayanamsas: &'static [AyanamsaDescriptor],
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
        profile_id: "pleiades-compatibility-profile/0.1.0",
        summary: "Stage 3 compatibility scaffold: baseline catalogs are published and tropical/sidereal chart assembly works, while house placement remains to be implemented.",
        house_systems: baseline_house_systems(),
        ayanamsas: baseline_ayanamsas(),
        known_gaps: &[
            "House placement for the baseline compatibility catalog still remains to be implemented.",
            "Compatibility coverage will expand as Stage 3 continues and later stages add validation.",
        ],
    }
}

impl fmt::Display for CompatibilityProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Compatibility profile: {}", self.profile_id)?;
        writeln!(f, "{}", self.summary)?;
        writeln!(f)?;
        writeln!(f, "Built-in house systems:")?;
        for entry in self.house_systems {
            write!(f, "- {}", entry.canonical_name)?;
            if !entry.aliases.is_empty() {
                write!(f, " (aliases: {})", entry.aliases.join(", "))?;
            }
            if entry.latitude_sensitive {
                write!(f, " [latitude-sensitive]")?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        writeln!(f, "Built-in ayanamsas:")?;
        for entry in self.ayanamsas {
            write!(f, "- {}", entry.canonical_name)?;
            if !entry.aliases.is_empty() {
                write!(f, " (aliases: {})", entry.aliases.join(", "))?;
            }
            writeln!(f)?;
        }
        writeln!(f)?;
        writeln!(f, "Known gaps:")?;
        for gap in self.known_gaps {
            writeln!(f, "- {}", gap)?;
        }
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
            .known_gaps
            .iter()
            .any(|gap| gap.contains("House placement")));
    }

    #[test]
    fn display_lists_catalog_sections() {
        let rendered = current_compatibility_profile().to_string();
        assert!(rendered.contains("Built-in house systems:"));
        assert!(rendered.contains("Built-in ayanamsas:"));
        assert!(rendered.contains("Known gaps:"));
    }
}
