//! Caller assertions and provenance for external ingestion.

use pleiades_types::{CoordinateFrame, TimeScale};

/// Output units the reader can normalize from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Units {
    /// Kilometers (position) — the corpus storage unit.
    Km,
    /// Astronomical units — converted to km on normalize.
    Au,
}

/// Supported coordinate centers/origins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Center {
    /// Solar System Barycenter (Horizons `500@0` / `@0`).
    SolarSystemBarycenter,
}

/// Caller-asserted attributes used to fill genuine source silences.
///
/// Every field is optional: assert only what the source omits. A field that is
/// `Some` and contradicts the source is a hard error; a field that is `Some`
/// and fills a silent source is recorded as `Provenance::Asserted`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExpectedProfile {
    /// Asserted coordinate frame.
    pub frame: Option<CoordinateFrame>,
    /// Asserted time scale.
    pub time_scale: Option<TimeScale>,
    /// Asserted output units.
    pub units: Option<Units>,
    /// Asserted center/origin.
    pub center: Option<Center>,
}

/// Whether a normalized attribute came from the source or a caller assertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    /// Read from the source headers.
    Read,
    /// Filled from `ExpectedProfile`.
    Asserted,
}

/// Per-attribute provenance of a normalized corpus.
#[derive(Clone, Debug, PartialEq)]
pub struct IngestProvenance {
    /// Frame provenance.
    pub frame: Provenance,
    /// Time-scale provenance.
    pub time_scale: Provenance,
    /// Units provenance.
    pub units: Provenance,
    /// Center provenance.
    pub center: Provenance,
    /// Free-form source label, if any.
    pub source_label: Option<String>,
    /// The fetch URL, when the bytes came from a live fetch.
    pub request_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{CoordinateFrame, TimeScale};

    #[test]
    fn expected_profile_defaults_to_all_silent() {
        let p = ExpectedProfile::default();
        assert!(p.frame.is_none());
        assert!(p.time_scale.is_none());
        assert!(p.units.is_none());
        assert!(p.center.is_none());
    }

    #[test]
    fn provenance_records_read_vs_asserted() {
        let prov = IngestProvenance {
            frame: Provenance::Read,
            time_scale: Provenance::Asserted,
            units: Provenance::Read,
            center: Provenance::Read,
            source_label: Some("JPL Horizons".to_string()),
            request_url: None,
        };
        assert_eq!(prov.time_scale, Provenance::Asserted);
    }

    #[test]
    fn units_and_center_are_distinct_values() {
        assert_ne!(Units::Km, Units::Au);
        let _ = ExpectedProfile {
            frame: Some(CoordinateFrame::Ecliptic),
            time_scale: Some(TimeScale::Tdb),
            units: Some(Units::Km),
            center: Some(Center::SolarSystemBarycenter),
        };
    }
}
