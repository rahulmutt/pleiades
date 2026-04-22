//! Ayanamsa catalog definitions and compatibility metadata.
//!
//! This crate currently focuses on the catalog layer: it enumerates the
//! baseline built-in ayanamsas, their common aliases, and notes about their
//! intended interoperability role. It also carries the baseline epoch/offset
//! metadata used by the chart-layer sidereal conversion helper.
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

use pleiades_types::{Angle, Ayanamsa, Instant, JulianDay};

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
    /// Reference epoch for the published offset, when available.
    pub epoch: Option<JulianDay>,
    /// Reference sidereal offset in degrees at the reference epoch, when available.
    pub offset_degrees: Option<Angle>,
}

impl AyanamsaDescriptor {
    /// Creates a new descriptor.
    pub const fn new(
        ayanamsa: Ayanamsa,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        epoch: Option<JulianDay>,
        offset_degrees: Option<Angle>,
    ) -> Self {
        Self {
            ayanamsa,
            canonical_name,
            aliases,
            notes,
            epoch,
            offset_degrees,
        }
    }

    /// Returns the sidereal offset at the provided instant, when the catalog
    /// entry carries enough metadata to derive one.
    pub fn offset_at(&self, instant: Instant) -> Option<Angle> {
        offset_from_components(self.epoch, self.offset_degrees, instant)
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
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Raman,
        "Raman",
        &["B. V. Raman"],
        "Popular named sidereal offset used in classical astrology software.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(21.014_44)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::Krishnamurti,
        "Krishnamurti",
        &["KP"],
        "Krishnamurti Paddhati ayanamsa.",
        Some(JulianDay::from_days(2_415_020.0)),
        Some(Angle::from_degrees(22.363_889)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::FaganBradley,
        "Fagan/Bradley",
        &["Fagan Bradley", "Fagan-Bradley"],
        "Western sidereal reference used by several astrology packages.",
        Some(JulianDay::from_days(2_433_282.423_46)),
        Some(Angle::from_degrees(24.042_044_444)),
    ),
    AyanamsaDescriptor::new(
        Ayanamsa::TrueChitra,
        "True Chitra",
        &["Chitra"],
        "True Chitra / Chitra-based sidereal variant.",
        Some(JulianDay::from_days(2_435_553.5)),
        Some(Angle::from_degrees(23.245_524_743)),
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

/// Returns the sidereal offset for the provided ayanamsa and instant.
///
/// Built-in catalog entries use the published reference epoch and offset
/// metadata where available. Custom ayanamsas can supply the same information
/// directly on the `CustomAyanamsa` definition.
pub fn sidereal_offset(ayanamsa: &Ayanamsa, instant: Instant) -> Option<Angle> {
    match ayanamsa {
        Ayanamsa::Custom(custom) => {
            offset_from_components(custom.epoch, custom.offset_degrees, instant)
        }
        other => descriptor(other).and_then(|entry| entry.offset_at(instant)),
    }
}

fn offset_from_components(
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

    #[test]
    fn sidereal_offset_is_available_for_baseline_ayanamsas() {
        let instant = Instant::new(
            JulianDay::from_days(2_451_545.0),
            pleiades_types::TimeScale::Tt,
        );
        let offset = sidereal_offset(&Ayanamsa::Lahiri, instant).expect("offset should exist");
        assert!(offset.degrees().is_finite());
    }
}
