//! Eclipse domain value types.

use pleiades_types::{Instant, Longitude};

/// Whether an eclipse is of the Sun (Moon between Earth and Sun, at new moon)
/// or of the Moon (Earth between Sun and Moon, at full moon).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseKind {
    /// Solar eclipse: the Moon occults the Sun at a new moon near a node.
    Solar,
    /// Lunar eclipse: the Moon passes through Earth's shadow at a full moon near a node.
    Lunar,
}

/// Geometric classification of a solar eclipse at its point of greatest eclipse.
///
/// The distinction is topocentric per the NASA canon convention (see
/// [`crate`] validation notes): `Total`/`Annular`/`Hybrid` require the shadow
/// axis to meet Earth; `Partial` is assigned when the axis misses the ellipsoid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolarEclipseType {
    /// The Moon fully covers the Sun's disk at greatest eclipse.
    Total,
    /// The Moon is angularly smaller than the Sun, leaving a bright ring.
    Annular,
    /// Annular-total: annular at the path ends but total near the sub-solar point.
    Hybrid,
    /// The shadow axis misses Earth; only a partial phase is seen anywhere.
    Partial,
}

/// Geometric classification of a lunar eclipse from the Moon's penetration of
/// Earth's shadow at greatest eclipse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LunarEclipseType {
    /// The Moon enters only the penumbra (partial shadow); no umbral contact.
    Penumbral,
    /// Part of the Moon enters the umbra (full shadow).
    Partial,
    /// The entire Moon is within the umbra at greatest eclipse.
    Total,
}

/// An eclipse type tagged by kind: either a [`SolarEclipseType`] or a
/// [`LunarEclipseType`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseType {
    /// A solar eclipse and its sub-classification.
    Solar(SolarEclipseType),
    /// A lunar eclipse and its sub-classification.
    Lunar(LunarEclipseType),
}

impl EclipseType {
    /// Returns the [`EclipseKind`] (solar or lunar) this type belongs to.
    pub fn kind(&self) -> EclipseKind {
        match self {
            EclipseType::Solar(_) => EclipseKind::Solar,
            EclipseType::Lunar(_) => EclipseKind::Lunar,
        }
    }
}

/// Selects which eclipse kinds a search returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EclipseFilter {
    /// Return both solar and lunar eclipses.
    All,
    /// Return solar eclipses only.
    SolarOnly,
    /// Return lunar eclipses only.
    LunarOnly,
}

impl EclipseFilter {
    /// Returns `true` if this filter admits the given [`EclipseKind`].
    pub fn admits(&self, kind: EclipseKind) -> bool {
        match self {
            EclipseFilter::All => true,
            EclipseFilter::SolarOnly => kind == EclipseKind::Solar,
            EclipseFilter::LunarOnly => kind == EclipseKind::Lunar,
        }
    }
}

/// Which lunar node the eclipse occurs near, derived from the sign of the
/// Moon's ecliptic latitude change through the syzygy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Node {
    /// Ascending node: the Moon's ecliptic latitude is increasing through zero.
    North,
    /// Descending node: the Moon's ecliptic latitude is decreasing through zero.
    South,
}

/// A geocentric sub-shadow point on Earth. Deliberately not `ObserverLocation`:
/// the greatest-eclipse point is a position, not an observing site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoLocation {
    /// Geographic (geodetic) latitude of the point, degrees, positive north.
    pub latitude_degrees: f64,
    /// Geographic longitude of the point, degrees, positive east.
    pub longitude_degrees: f64,
}

/// A single global/geocentric eclipse and its computed circumstances.
///
/// All quantities are geocentric except the solar magnitude and type, which
/// follow the NASA canon's topocentric-at-greatest-eclipse convention. No
/// per-observer (local) circumstances are provided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eclipse {
    /// Solar or lunar.
    pub kind: EclipseKind,
    /// The classified eclipse type (matches `kind`).
    pub eclipse_type: EclipseType,
    /// Instant of greatest eclipse, in the TDB time scale.
    pub greatest_eclipse: Instant,
    /// Eclipse magnitude (fraction of the eclipsed body's diameter covered at
    /// greatest eclipse): topocentric for solar, umbral/penumbral for lunar.
    pub magnitude: f64,
    /// Gamma: least distance of the shadow axis from Earth's center, in
    /// equatorial Earth radii (signed; sign follows the Moon's latitude).
    pub gamma: f64,
    /// Saros series number this eclipse belongs to.
    pub saros_series: u32,
    /// Ecliptic longitude of the eclipsed body at greatest eclipse (apparent
    /// tropical ecliptic of date, no ayanamsa): the Sun for solar eclipses,
    /// the Moon (Sun + 180°) for lunar eclipses.
    pub eclipsed_longitude: Longitude,
    /// The lunar node (ascending/descending) the eclipse occurs near.
    pub near_node: Node,
    /// Sub-shadow point of greatest eclipse for solar eclipses; always `None`
    /// for lunar eclipses (which have no single surface location).
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
