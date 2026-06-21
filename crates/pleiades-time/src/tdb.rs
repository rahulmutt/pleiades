//! Deterministic TT <-> TDB periodic term (USNO approximation, sub-millisecond).

/// `TDB − TT` in seconds for a TT Julian Day. Standard low-precision model:
/// `g = 357.53° + 0.9856003° * (JD_TT − 2451545.0)`,
/// `TDB − TT = 0.001658 sin g + 0.000014 sin 2g`.
pub fn tdb_minus_tt_seconds(jd_tt: f64) -> f64 {
    let g_deg = 357.53 + 0.985_600_3 * (jd_tt - 2451545.0);
    let g = g_deg.to_radians();
    0.001_658 * g.sin() + 0.000_014 * (2.0 * g).sin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_below_two_milliseconds() {
        for offset in [-36525.0, -18262.0, 0.0, 18262.0, 36525.0] {
            let v = tdb_minus_tt_seconds(2451545.0 + offset);
            assert!(
                v.abs() < 0.002,
                "TDB-TT {v} out of bound at offset {offset}"
            );
        }
    }
}
