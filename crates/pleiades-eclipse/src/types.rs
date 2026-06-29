//! Eclipse domain value types.

use pleiades_types::{Instant, Longitude};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseKind {
    Solar,
    Lunar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolarEclipseType {
    Total,
    Annular,
    Hybrid,
    Partial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarEclipseType {
    Penumbral,
    Partial,
    Total,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseType {
    Solar(SolarEclipseType),
    Lunar(LunarEclipseType),
}

impl EclipseType {
    pub fn kind(&self) -> EclipseKind {
        match self {
            EclipseType::Solar(_) => EclipseKind::Solar,
            EclipseType::Lunar(_) => EclipseKind::Lunar,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseFilter {
    All,
    SolarOnly,
    LunarOnly,
}

impl EclipseFilter {
    pub fn admits(&self, kind: EclipseKind) -> bool {
        match self {
            EclipseFilter::All => true,
            EclipseFilter::SolarOnly => kind == EclipseKind::Solar,
            EclipseFilter::LunarOnly => kind == EclipseKind::Lunar,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    North,
    South,
}

/// A geocentric sub-shadow point on Earth. Deliberately not `ObserverLocation`:
/// the greatest-eclipse point is a position, not an observing site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoLocation {
    pub latitude_degrees: f64,
    pub longitude_degrees: f64,
}

/// A single global/geocentric eclipse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eclipse {
    pub kind: EclipseKind,
    pub eclipse_type: EclipseType,
    pub greatest_eclipse: Instant,
    pub magnitude: f64,
    pub gamma: f64,
    pub saros_series: u32,
    pub eclipsed_longitude: Longitude,
    pub near_node: Node,
    pub greatest_eclipse_location: Option<GeoLocation>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eclipse_type_reports_its_kind() {
        assert_eq!(
            EclipseType::Solar(SolarEclipseType::Total).kind(),
            EclipseKind::Solar
        );
        assert_eq!(
            EclipseType::Lunar(LunarEclipseType::Penumbral).kind(),
            EclipseKind::Lunar
        );
    }

    #[test]
    fn filter_admits_expected_kinds() {
        assert!(EclipseFilter::All.admits(EclipseKind::Solar));
        assert!(EclipseFilter::All.admits(EclipseKind::Lunar));
        assert!(EclipseFilter::SolarOnly.admits(EclipseKind::Solar));
        assert!(!EclipseFilter::SolarOnly.admits(EclipseKind::Lunar));
        assert!(EclipseFilter::LunarOnly.admits(EclipseKind::Lunar));
        assert!(!EclipseFilter::LunarOnly.admits(EclipseKind::Solar));
    }
}
