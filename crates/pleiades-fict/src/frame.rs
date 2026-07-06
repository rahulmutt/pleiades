//! Rotate an element-equinox ecliptic Cartesian vector to the J2000 mean ecliptic.

/// Rotate `(x, y, z)` (AU) from the mean ecliptic of the element equinox
/// `equinox_jd` to the J2000 mean ecliptic. J2000 is identity; any other equinox
/// is precessed via `pleiades_apparent::precess_ecliptic_date_to_j2000`
/// (ecliptic-frame IAU-1976 precession; distance is preserved). For
/// `Equinox::OfDate` the caller passes the evaluation JD as `equinox_jd`.
pub fn rotate_ecliptic_to_j2000(x: f64, y: f64, z: f64, equinox_jd: f64) -> (f64, f64, f64) {
    if (equinox_jd - crate::J2000_JD).abs() < 1.0e-6 {
        return (x, y, z);
    }
    let r = (x * x + y * y + z * z).sqrt();
    if r == 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let lon = y.atan2(x).to_degrees().rem_euclid(360.0);
    let lat = (z / r).clamp(-1.0, 1.0).asin().to_degrees();
    let p = pleiades_apparent::precess_ecliptic_date_to_j2000(lon, lat, equinox_jd)
        .expect("fictitious body lon/lat precess cleanly to J2000");
    let lon_r = p.longitude_deg.to_radians();
    let lat_r = p.latitude_deg.to_radians();
    (
        r * lat_r.cos() * lon_r.cos(),
        r * lat_r.cos() * lon_r.sin(),
        r * lat_r.sin(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_equinox_is_identity() {
        let (x, y, z) = rotate_ecliptic_to_j2000(1.0, 2.0, 0.5, crate::J2000_JD);
        assert!((x - 1.0).abs() < 1.0e-12);
        assert!((y - 2.0).abs() < 1.0e-12);
        assert!((z - 0.5).abs() < 1.0e-12);
    }

    #[test]
    fn precession_preserves_distance() {
        // Ecliptic precession is a pure rotation: |v| is invariant. J1900 = 2415020.0.
        let (x, y, z) = rotate_ecliptic_to_j2000(3.0, 0.0, 0.0, 2_415_020.0);
        let r = (x * x + y * y + z * z).sqrt();
        assert!((r - 3.0).abs() < 1.0e-9, "distance changed: r={r}");
    }
}
