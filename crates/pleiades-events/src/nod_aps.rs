//! Planetary/lunar orbital nodes and apsides — Swiss Ephemeris `swe_nod_aps`
//! analogue. See `EventEngine::nod_aps`.

/// How the orbit is modeled — Swiss Ephemeris `SE_NODBIT_*` analogues.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodApsMethod {
    /// Mean orbital elements (`SE_NODBIT_MEAN`). Moon + Sun + Mercury–Neptune.
    Mean,
    /// Osculating ellipse from the instantaneous state (`SE_NODBIT_OSCU`).
    Osculating,
    /// Osculating ellipse about the solar-system barycenter for bodies beyond
    /// ~6 AU heliocentric distance (`SE_NODBIT_OSCU_BAR`); inside 6 AU this
    /// falls back to the heliocentric ellipse, matching Swiss Ephemeris.
    OsculatingBarycentric,
}

/// What the fourth point means — aphelion or the ellipse's second focus
/// (`SE_NODBIT_FOPOINT`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApsisConvention {
    /// Far apsis at distance `a(1+e)`.
    Aphelion,
    /// Second (empty) focus at distance `2ae`, same direction as the aphelion.
    SecondFocus,
}

/// One orbital point: geocentric true-ecliptic-of-date position and
/// central-difference speeds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodApsPoint {
    /// Ecliptic longitude, degrees in `[0, 360)`, true equinox of date.
    pub longitude_deg: f64,
    /// Ecliptic latitude, degrees in `[-90, 90]`.
    pub latitude_deg: f64,
    /// Geocentric distance, AU.
    pub distance_au: f64,
    /// dλ/dt, degrees/day (central difference over ±0.5 day).
    pub longitude_speed_deg_per_day: f64,
    /// dβ/dt, degrees/day.
    pub latitude_speed_deg_per_day: f64,
    /// d(distance)/dt, AU/day.
    pub distance_speed_au_per_day: f64,
}

/// The four orbital points returned by [`EventEngine::nod_aps`]
/// (ascending node, descending node, perihelion, aphelion-or-focus).
///
/// [`EventEngine::nod_aps`]: crate::EventEngine::nod_aps
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodesApsides {
    /// Ascending-node point.
    pub ascending: NodApsPoint,
    /// Descending-node point.
    pub descending: NodApsPoint,
    /// Near apsis (perihelion; perigee for the Moon).
    pub perihelion: NodApsPoint,
    /// Far apsis — or second focus under [`ApsisConvention::SecondFocus`].
    pub aphelion: NodApsPoint,
    /// The method that actually served the request.
    pub method: NodApsMethod,
}
