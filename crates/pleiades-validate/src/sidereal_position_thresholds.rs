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
//! half of what issue #164 corrected. The Sun's and the planets' speed
//! ceilings are below that rate and catch such a speed; the Moon's, 2.1″/day,
//! is not. The Moon's speed is held instead by
//! `a_sidereal_mean_chart_reports_the_mean_of_date_sidereal_place` in
//! `pleiades-events/tests/reference.rs`, to the events engine's within
//! 1e-6 deg/day.
//!
//! The rows that drive the maxima:
//!
//! - Moon: all three maxima come from one epoch, JD 2425168.5 (October
//!   1927): longitude −1.99″ to −2.13″ across the four ayanamsas, latitude
//!   −2.454″, speed +1.477″/day. The runners-up are 1.92″, 1.12″ and
//!   1.00″/day. The packaged Moon agrees with the DE440 corpus to 0.0001″,
//!   so this is Swiss Ephemeris' Moshier Moon against DE440.
//! - Planets: longitude 2.360″ is Neptune under Lahiri at JD 2483866.5 (the
//!   helio gate records the same Moshier-vs-DE440 Neptune difference,
//!   2.294″); Mercury–Uranus stay within 0.89″ and Pluto within 1.19″.
//!   Latitude 0.595″ is Pluto at JD 2427342.5. Speed 0.0384″/day is Mercury
//!   under True Citra at JD 2471909.5.
//!
//! No group carries a systematic longitude offset above its class's
//! ayanamsa-gate ceiling. The largest mean signed residual is Neptune's,
//! +0.50″ to +0.64″, alike under all four ayanamsas, so it is the
//! ephemeris and not an ayanamsa. The Sun's is +0.123″ under Lahiri and
//! within ±0.022″ under the other three.

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
