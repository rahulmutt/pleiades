//! Ayanamsa catalog definitions and compatibility metadata.
//!
//! This crate currently focuses on the catalog layer: it enumerates the
//! baseline built-in ayanamsas, their common aliases, and notes about their
//! intended interoperability role. The numerical sidereal conversion layer will
//! be added on top of these identifiers in later stages.
//!
//! # Examples
//!
//! ```
//! use pleiades_ayanamsa::{baseline_ayanamsas, resolve_ayanamsa};
//!
//! let catalog = baseline_ayanamsas();
//! assert!(catalog.iter().any(|entry| entry.canonical_name == "Lahiri"));
//!
//! assert_eq!(resolve_ayanamsa("KP"), Some(pleiades_types::Ayanamsa::Krishnamurti));
//! ```

#![forbid(unsafe_code)]

use pleiades_types::Ayanamsa;

/// A catalog entry for a built-in ayanamsa.
#[derive(Clone, Debug, PartialEq)]
pub struct AyanamsaDescriptor {
    /// The strongly typed ayanamsa identifier.
    pub ayanamsa: Ayanamsa,
    /// The canonical name used in compatibility profiles.
    pub canonical_name: &'static str,
    /// Alternate names or software-specific aliases.
    pub aliases: &'static [&'static str],
    /// Short notes about the definition or interoperability constraints.
    pub notes: &'static str,
}

impl AyanamsaDescriptor {
    /// Creates a new descriptor.
    pub const fn new(
        ayanamsa: Ayanamsa,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
    ) -> Self {
        Self {
            ayanamsa,
            canonical_name,
            aliases,
            notes,
        }
    }

    /// Returns `true` if the provided label matches the canonical name or one
    /// of the documented aliases.
    pub fn matches_label(&self, label: &str) -> bool {
        self.canonical_name.eq_ignore_ascii_case(label)
            || self
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(label))
    }
}

const BASELINE_AYANAMSAS: &[AyanamsaDescriptor] = &[
    AyanamsaDescriptor::new(
        Ayanamsa::Lahiri,
        "Lahiri",
        &["Chitrapaksha"],
        "Default Indian sidereal standard in many astrology workflows.",
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["B. V. Raman"],
        "Popular named sidereal offset used in classical astrology software.",
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Krishnamurti",
        &["KP"],
        "Krishnamurti Paddhati ayanamsa.",
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::FaganBradley,
        "Fagan/Bradley",
        &["Fagan Bradley", "Fagan-Bradley"],
        "Western sidereal reference used by several astrology packages.",
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueChitra,
        "True Chitra",
        &["Chitra"],
        "True Chitra / Chitra-based sidereal variant.",
    ),
];

/// Returns the baseline built-in ayanamsa catalog.
pub const fn baseline_ayanamsas() -> &'static [AyanamsaDescriptor] {
    BASELINE_AYANAMSAS
}

/// Finds the descriptor for a typed ayanamsa selection.
pub fn descriptor(ayanamsa: &Ayanamsa) -> Option<&'static AyanamsaDescriptor> {
    BASELINE_AYANAMSAS
        .iter()
        .find(|entry| entry.ayanamsa == *ayanamsa)
}

/// Resolves an ayanamsa label to a built-in type.
pub fn resolve_ayanamsa(label: &str) -> Option<Ayanamsa> {
    BASELINE_AYANAMSAS
        .iter()
        .find(|entry| entry.matches_label(label))
        .map(|entry| entry.ayanamsa.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_catalog_includes_required_milestone_entries() {
        let names: Vec<_> = baseline_ayanamsas()
            .iter()
            .map(|entry| entry.canonical_name)
            .collect();

        for expected in [
            "Lahiri",
            "Raman",
            "Krishnamurti",
            "Fagan/Bradley",
            "True Chitra",
        ] {
            assert!(names.contains(&expected), "missing {expected}");
        }
    }

    #[test]
    fn aliases_resolve_to_builtin_ayanamsas() {
        assert_eq!(resolve_ayanamsa("KP"), Some(Ayanamsa::Krishnamurti));
        assert_eq!(
            resolve_ayanamsa("fagan-bradley"),
            Some(Ayanamsa::FaganBradley)
        );
        assert_eq!(resolve_ayanamsa("chitrapaksha"), Some(Ayanamsa::Lahiri));
    }
}
