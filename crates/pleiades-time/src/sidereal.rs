//! Greenwich mean sidereal time (IAU-1982, Meeus 12.4) from a UT1 Julian day.

/// Greenwich mean sidereal time in degrees, **unnormalized** — the raw IAU-1982
/// (Meeus eq. 12.4) polynomial, which may lie outside `[0, 360)`. This is the
/// single source of the GMST coefficients; `gmst_degrees` reduces it to `[0,360)`.
pub fn gmst_degrees_raw(jd_ut1: f64) -> f64 {
    let t = (jd_ut1 - 2_451_545.0) / 36_525.0;
    280.460_618_37 + 360.985_647_366_29 * (jd_ut1 - 2_451_545.0) + 0.000_387_933 * t * t
        - (t * t * t) / 38_710_000.0
}

/// Greenwich mean sidereal time in degrees, normalized to `[0, 360)`.
///
/// `jd_ut1` is the Julian day in the UT1 time scale. Formula: Meeus,
/// *Astronomical Algorithms*, eq. 12.4.
pub fn gmst_degrees(jd_ut1: f64) -> f64 {
    gmst_degrees_raw(jd_ut1).rem_euclid(360.0)
}

#[cfg(test)]
mod tests;
