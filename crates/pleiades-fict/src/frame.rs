//! Rotate an element-equinox ecliptic Cartesian vector to the J2000 mean ecliptic.

/// Rotate `(x, y, z)` (AU) from the mean ecliptic of the element equinox
/// `equinox_jd` to the J2000 mean ecliptic. J2000 is identity; any other equinox
/// is precessed via
/// `pleiades_apparent::precess_ecliptic_vector_date_to_j2000_long_term`
/// (Vondrák 2011, a rotation, so distance is preserved). That is the model SE
/// applies in `swi_osc_el_plan`; the IAU-1976 expressions drift by about 1.4″
/// at Nibiru's ~370 CE equinox. For `Equinox::OfDate` the caller passes the
/// evaluation JD as `equinox_jd`.
pub fn rotate_ecliptic_to_j2000(x: f64, y: f64, z: f64, equinox_jd: f64) -> (f64, f64, f64) {
    if (equinox_jd - crate::J2000_JD).abs() < 1.0e-6 {
        return (x, y, z);
    }
    let [x, y, z] =
        pleiades_apparent::precess_ecliptic_vector_date_to_j2000_long_term([x, y, z], equinox_jd)
            .expect("fictitious body vector precesses cleanly to J2000");
    (x, y, z)
}

#[cfg(test)]
mod tests;
