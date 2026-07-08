//! Lunar occultations — Swiss Ephemeris `swe_lun_occult_*` analogue: the Moon
//! occulting a planet (small disc) or a catalogued fixed star (point). Local
//! circumstances (`how`), per-observer search (`when_loc`), and global search
//! (`when_glob`). Geocentric/topocentric apparent-of-date, TDB, 1900–2100 CE.

use pleiades_types::{CelestialBody, Instant, Latitude, Longitude};

/// What the Moon is occulting.
#[derive(Clone, Debug, PartialEq)]
pub enum OccultTarget {
    /// A planet, Mercury..=Pluto. Sun and Moon are rejected (Sun ⇒ solar
    /// eclipse; the Moon is the occulter).
    Body(CelestialBody),
    /// A curated fixed-star catalog name (see [`crate::fixed_star_entry`]).
    Star(String),
}

/// What an observer (or the globe, for `when_glob`) actually sees.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccultationType {
    /// Target fully covered at maximum (point star hidden, or planet disc fully
    /// behind the Moon's limb).
    Total,
    /// The Moon's limb crosses the target but never fully covers it.
    Grazing,
    /// No contact (topocentric/geocentric separation never small enough).
    Miss,
}

/// One observer-local contact event: its instant plus the target's horizontal
/// position and visibility there. A contact below the horizon is still timed
/// but flagged `visible == false`, matching SE.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OccultationContact {
    /// Instant of the contact (TDB).
    pub instant: Instant,
    /// Apparent (refracted) altitude of the target, degrees.
    pub altitude_degrees: f64,
    /// Azimuth from south increasing westward, `[0,360)` degrees.
    pub azimuth_degrees: f64,
    /// Whether the target is above the horizon at this instant.
    pub visible: bool,
}

/// Local circumstances of a lunar occultation of one target for one observer
/// (`how` / `when_loc`). Contact fields mirror the SP-2c eclipse C1–C4 layout.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// What THIS observer sees.
    pub occultation_type: OccultationType,
    /// Instant of least topocentric separation.
    pub maximum: OccultationContact,
    /// Covered fraction of the target's diameter at maximum (SE `attr[0]`).
    pub magnitude: f64,
    /// Covered fraction of the target's disc area at maximum (SE `attr[2]`);
    /// `0.0`/`1.0` for a point star.
    pub obscuration: f64,
    /// C1 — disappearance (exterior ingress).
    pub first_contact: OccultationContact,
    /// C2 — target fully hidden (planet disc only; `None` for a star / graze).
    pub second_contact: Option<OccultationContact>,
    /// C3 — target begins to reappear (planet disc only).
    pub third_contact: Option<OccultationContact>,
    /// C4 — reappearance (exterior egress).
    pub fourth_contact: OccultationContact,
    /// Whether the target is above the horizon during any part of the event.
    pub any_phase_visible: bool,
}

/// Global circumstances (`when_glob`): the greatest-occultation instant and the
/// sub-lunar point where it is central/greatest. NOT the full path polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalOccultation {
    /// The occulted target.
    pub target: OccultTarget,
    /// Instant of greatest global occultation (TDB).
    pub maximum: Instant,
    /// Sub-lunar point of maximum occultation: geographic latitude, positive north.
    pub sublunar_latitude: Latitude,
    /// Sub-lunar point of maximum occultation: geographic longitude, positive east.
    pub sublunar_longitude: Longitude,
    /// Whether a central occultation exists somewhere on Earth.
    pub central: bool,
    /// Best-case type over the globe.
    pub occultation_type: OccultationType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{JulianDay, TimeScale};

    fn contact(jd: f64) -> OccultationContact {
        OccultationContact {
            instant: Instant::new(JulianDay::from_days(jd), TimeScale::Tdb),
            altitude_degrees: 42.0,
            azimuth_degrees: 100.0,
            visible: true,
        }
    }

    #[test]
    fn local_occultation_holds_optional_interior_contacts() {
        let star = LocalOccultation {
            target: OccultTarget::Star("Aldebaran".into()),
            occultation_type: OccultationType::Total,
            maximum: contact(2_451_545.0),
            magnitude: 1.0,
            obscuration: 1.0,
            first_contact: contact(2_451_544.98),
            second_contact: None,
            third_contact: None,
            fourth_contact: contact(2_451_545.02),
            any_phase_visible: true,
        };
        assert!(star.second_contact.is_none());
        assert_eq!(star.occultation_type, OccultationType::Total);
    }
}
