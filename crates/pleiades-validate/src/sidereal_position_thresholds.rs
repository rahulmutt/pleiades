//! Measured-basis ceilings for the `validate-sidereal-position` gate.
//!
//! Both sides of the comparison are the geometric geocentric place on the
//! mean ecliptic and equinox of date less the mean ayanamsa, so the residual
//! is the Moshier-vs-DE440 ephemeris difference plus the ayanamsa's own
//! residual. Measured 2026-10-06 over all 2680 rows (67 epochs, four
//! ayanamsas, ten bodies).
//!
//! Longitude and latitude ceilings are `ceil(1.4 × max)` in whole
//! arcseconds, the crossings corpus rule. Speed ceilings are `1.4 × max`
//! rounded up to two significant figures: a whole-arcsecond speed ceiling
//! would pass a speed that misses the precession rate, 0.138″/day, which is
//! half of what issue #164 corrected.

/// Per-channel ceilings for one body class.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) lon_arcsec: f64,
    pub(crate) lat_arcsec: f64,
    pub(crate) lon_speed_arcsec_per_day: f64,
}

pub(crate) const SUN_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 1.0,                 // measured max 0.415"
    lat_arcsec: 1.0,                 // measured max 0.059"
    lon_speed_arcsec_per_day: 0.020, // measured max 0.0139"/day
};

pub(crate) const MOON_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 3.0,               // measured max 2.132"
    lat_arcsec: 4.0,               // measured max 2.454"
    lon_speed_arcsec_per_day: 2.1, // measured max 1.4772"/day
};

/// Mercury–Pluto.
pub(crate) const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 4.0,                 // measured max 2.360"
    lat_arcsec: 1.0,                 // measured max 0.595"
    lon_speed_arcsec_per_day: 0.054, // measured max 0.0384"/day
};
