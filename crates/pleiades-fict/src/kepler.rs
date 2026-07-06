//! Kepler's-equation solver and orbital-plane position.

/// Solve Kepler's equation `M = E − e·sin E` for the eccentric anomaly `E`
/// (radians), by Newton–Raphson. Converges to machine precision in a few
/// iterations for the bounded eccentricities (`e < 1`) of `seorbel.txt`.
pub fn solve_kepler(mean_anomaly_rad: f64, eccentricity: f64) -> f64 {
    let m = mean_anomaly_rad.rem_euclid(std::f64::consts::TAU);
    let mut e = if eccentricity < 0.8 { m } else { std::f64::consts::PI };
    for _ in 0..64 {
        let delta = (e - eccentricity * e.sin() - m) / (1.0 - eccentricity * e.cos());
        e -= delta;
        if delta.abs() < 1.0e-14 {
            break;
        }
    }
    e
}

/// Orbital-plane Cartesian position (AU), focus at the origin, x-axis toward
/// perihelion, from the semi-major axis, eccentricity, and eccentric anomaly.
pub fn orbital_plane_position(a_au: f64, e: f64, eccentric_anomaly_rad: f64) -> (f64, f64) {
    let x = a_au * (eccentric_anomaly_rad.cos() - e);
    let y = a_au * (1.0 - e * e).sqrt() * eccentric_anomaly_rad.sin();
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_kepler_inverts_the_equation() {
        for &e in &[0.0, 0.05, 0.2, 0.5, 0.9] {
            for step in 0..12 {
                let m = step as f64 * 0.5;
                let ea = solve_kepler(m, e);
                let recovered = ea - e * ea.sin();
                let diff = (recovered - m.rem_euclid(std::f64::consts::TAU)).abs();
                assert!(diff < 1.0e-12, "e={e} m={m} diff={diff}");
            }
        }
    }

    #[test]
    fn circular_orbit_has_radius_a() {
        let (x, y) = orbital_plane_position(2.5, 0.0, 1.0);
        assert!((x.hypot(y) - 2.5).abs() < 1.0e-12);
    }
}
