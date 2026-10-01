//! Measured-basis ceilings for the `validate-helio-position` gate.

/// Per-channel ceilings for one body group.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ceilings {
    pub(crate) lon_arcsec: f64,
    pub(crate) lat_arcsec: f64,
    pub(crate) dist_rel: f64,
    pub(crate) lon_speed_arcsec_per_day: f64,
    pub(crate) lat_speed_arcsec_per_day: f64,
    pub(crate) dist_speed_au_per_day: f64,
}

// Ceilings are set in Step 4 from the measured maxima. Until then they are
// open so the first run reports the maxima instead of failing.
// Ceilings: 1.5 x the measured maximum of each channel, rounded up to two
// significant figures, measured 2026-10-01 over 25424 rows. The longitude
// residual is dominated by the light-time signature of the planet-minus-Sun
// reconstruction, as in validate-crossings.

/// Mercury-Neptune.
pub(crate) const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 62.0,               // measured max 40.8046"
    lat_arcsec: 7.4,                // measured max 4.90827"
    dist_rel: 5.1e-5,               // measured max 3.37996e-5
    lon_speed_arcsec_per_day: 1.8,  // measured max 1.15027"/day
    lat_speed_arcsec_per_day: 0.85, // measured max 0.566267"/day
    dist_speed_au_per_day: 9.9e-6,  // measured max 6.55583e-6 AU/day
};

/// Pluto (served from a different source than the other planets).
pub(crate) const PLUTO_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 6.7,                 // measured max 4.40740"
    lat_arcsec: 1.7,                 // measured max 1.12878"
    dist_rel: 1.0e-5,                // measured max 6.62453e-6
    lon_speed_arcsec_per_day: 0.046, // measured max 0.0300050"/day
    lat_speed_arcsec_per_day: 0.14,  // measured max 0.0869711"/day
    dist_speed_au_per_day: 3.9e-7,   // measured max 2.58703e-7 AU/day
};
