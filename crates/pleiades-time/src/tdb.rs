//! Deterministic TT <-> TDB periodic term (USNO low-precision approximation,
//! magnitude bounded below ~2 ms; peak ~1.7 ms).

/// `TDB − TT` in seconds for a TT Julian Day. Standard USNO low-precision model,
/// which captures only the dominant annual/semi-annual terms (peak amplitude
/// ~1.7 ms, always bounded below ~2 ms) and omits the smaller planetary and
/// lunar terms of the full Fairhead–Bretagnon series:
/// `g = 357.53° + 0.9856003° * (JD_TT − 2451545.0)`,
/// `TDB − TT = 0.001658 sin g + 0.000014 sin 2g`.
pub fn tdb_minus_tt_seconds(jd_tt: f64) -> f64 {
    let g_deg = 357.53 + 0.985_600_3 * (jd_tt - 2451545.0);
    let g = g_deg.to_radians();
    0.001_658 * g.sin() + 0.000_014 * (2.0 * g).sin()
}

#[cfg(test)]
mod tests;
