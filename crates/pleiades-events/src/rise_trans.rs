//! Rise, set, and meridian-transit finding (`swe_rise_trans`, full-flag).

// `effective()` and some variants here are not yet consumed by an engine
// method; those land in Tasks 9-13. Silence the dead_code lint until then.
#![allow(dead_code)]

use pleiades_types::{CelestialBody, Instant, Latitude, Longitude};

/// Which observer-local event to find.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiseSetEvent {
    /// Body crosses the horizon upward.
    Rise,
    /// Body crosses the horizon downward.
    Set,
    /// Upper (meridian) transit — hour angle 0.
    UpperTransit,
    /// Lower transit — hour angle ±12ʰ.
    LowerTransit,
}

/// Which point of the disc defines the event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscMode {
    /// Disc center.
    Center,
    /// Upper limb (SE default for rise/set).
    UpperLimb,
    /// Lower limb (`SE_BIT_DISC_BOTTOM`).
    LowerLimb,
}

/// The object whose event is sought.
#[derive(Clone, Debug)]
pub enum RiseSetTarget {
    /// A release-grade body.
    Body(CelestialBody),
    /// An arbitrary ecliptic point (longitude, latitude); pair with `no_ecl_lat`
    /// to force latitude 0 (rising of a zodiac degree).
    EclipticPoint(Longitude, Latitude),
    /// A curated fixed star by name.
    FixedStar(String),
}

/// `swe_rise_trans` flag bundle.
#[derive(Clone, Debug)]
pub struct RiseSetOptions {
    /// Disc convention.
    pub disc: DiscMode,
    /// Apply atmospheric refraction (`false` = `SE_BIT_NO_REFRACTION`).
    pub refraction: bool,
    /// Force ecliptic latitude 0 (`SE_BIT_GEOCTR_NO_ECL_LAT`).
    pub no_ecl_lat: bool,
    /// Hindu rising = `DISC_CENTER | NO_REFRACTION | GEOCTR_NO_ECL_LAT`.
    pub hindu: bool,
    /// Freeze semidiameter at mean distance (`SE_BIT_FIXED_DISC_SIZE`).
    pub fixed_disc_size: bool,
    /// Custom local horizon altitude, degrees (`swe_rise_trans_true_hor`).
    pub horizon_altitude_deg: Option<f64>,
}

impl Default for RiseSetOptions {
    fn default() -> Self {
        Self {
            disc: DiscMode::UpperLimb,
            refraction: true,
            no_ecl_lat: false,
            hindu: false,
            fixed_disc_size: false,
            horizon_altitude_deg: None,
        }
    }
}

impl RiseSetOptions {
    /// Resolves `hindu` into its component flags (SE composition).
    pub(crate) fn effective(&self) -> Self {
        if self.hindu {
            Self {
                disc: DiscMode::Center,
                refraction: false,
                no_ecl_lat: true,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

/// A located rise/set/transit event (TDB).
#[derive(Clone, Debug)]
pub struct RiseSet {
    /// Which event this is.
    pub event: RiseSetEvent,
    /// Instant of the event (TDB).
    pub instant: Instant,
    /// The target the event is for.
    pub target: RiseSetTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options_are_upper_limb_refracted() {
        let o = RiseSetOptions::default();
        assert!(matches!(o.disc, DiscMode::UpperLimb));
        assert!(o.refraction);
        assert!(!o.hindu && !o.no_ecl_lat && !o.fixed_disc_size);
        assert!(o.horizon_altitude_deg.is_none());
    }
}
