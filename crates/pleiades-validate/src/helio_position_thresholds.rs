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

// Basis: the engine's heliocentric place is geometric (no light-time, no
// aberration), true ecliptic and equinox of date, and the reference corpus is
// Swiss Ephemeris `SEFLG_MOSEPH|SEFLG_HELCTR|SEFLG_TRUEPOS|SEFLG_SPEED` — the
// same geometric place, so the comparison is like-for-like and the residual is
// the Moshier-vs-DE440 ephemeris difference.
//
// Latitude speed: for Mars–Pluto the residual is a near-uniform ≈0.085″/day
// that regresses on dΔε/dt · sin λ (slope −1.001, 99.7 % of the variance;
// ≤ 0.0065″/day remains once it is removed). Swiss Ephemeris' latitude speed
// carries a nutation-in-obliquity rate term that is not the derivative of its
// own latitude; pleiades' latitude speed matches the central difference of its
// own latitude to < 1e-5″/day (final-review probe, 2026-10-01). The gate
// deliberately does not model that term, so it sits inside the latitude-speed
// ceilings below.
//
// Ceilings: 1.5 x the measured maximum of each channel, rounded up to two
// significant figures, measured 2026-10-01 over all 25424 rows.

/// Mercury-Neptune.
pub(crate) const PLANET_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 3.5,                // measured max 2.29414" (Neptune)
    lat_arcsec: 0.36,               // measured max 0.236643"
    dist_rel: 4.7e-6,               // measured max 3.09697e-6
    lon_speed_arcsec_per_day: 0.12, // measured max 0.0795273"/day
    lat_speed_arcsec_per_day: 0.14, // measured max 0.0898433"/day
    dist_speed_au_per_day: 2.8e-7,  // measured max 1.85688e-7 AU/day
};

/// Pluto (served from a different source than the other planets).
pub(crate) const PLUTO_CEILINGS: Ceilings = Ceilings {
    lon_arcsec: 1.8,                 // measured max 1.19402"
    lat_arcsec: 0.91,                // measured max 0.606051"
    dist_rel: 6.5e-6,                // measured max 4.26899e-6
    lon_speed_arcsec_per_day: 0.046, // measured max 0.0300398"/day
    lat_speed_arcsec_per_day: 0.14,  // measured max 0.0869396"/day
    dist_speed_au_per_day: 3.0e-7,   // measured max 1.95533e-7 AU/day
};

/// Bound on the mean signed longitude-speed residual over Jupiter–Neptune
/// (arcsec/day). A reference and an engine that disagree on whether the
/// longitude speed includes the general-precession rate differ by
/// ≈ ±0.137″/day on every row; the measured mean is +0.000232″/day
/// (2026-10-01), so 0.02″/day separates the two cases with wide margin.
pub(crate) const OUTER_LON_SPEED_MEAN_SIGNED_BOUND_ARCSEC_PER_DAY: f64 = 0.02;
