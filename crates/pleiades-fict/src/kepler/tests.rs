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
