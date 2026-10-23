use super::*;

#[test]
fn solve_kepler_inverts_the_equation() {
    for &e in &[0.0, 0.05, 0.2, 0.5, 0.9] {
        for step in 0..12 {
            let m = step as f64 * 0.5;
            let ea = solve_kepler(m, e);
            let recovered = ea - e * libm::sin(ea);
            let diff = (recovered - m.rem_euclid(std::f64::consts::TAU)).abs();
            assert!(diff < 1.0e-12, "e={e} m={m} diff={diff}");
        }
    }
}

#[test]
fn solve_kepler_converges_for_the_high_eccentricity_orbits() {
    // Substitution into `E - e sin E = M`. Nibiru's e = 0.981092 (seorbel
    // set 10). Newton started at E = M diverges for this (e, M) -- the first
    // step is (M - e sin M - M) / (1 - e cos M) ~ -0.13 / 0.019 -- so the
    // start at pi for e >= 0.8 is what makes it converge.
    for &e in &[0.8, 0.9, 0.981_092, 0.99] {
        for degrees in [0.5, 7.5, 9.5, 15.0, 17.0, 90.0, 180.0, 270.0, 343.0, 359.5] {
            let m: f64 = f64::to_radians(degrees);
            let ea = solve_kepler(m, e);
            let diff = (ea - e * libm::sin(ea) - m).abs();
            assert!(diff < 1.0e-12, "e={e} M={degrees} deg diff={diff}");
        }
    }
}

#[test]
fn solve_kepler_of_a_circular_orbit_is_the_mean_anomaly() {
    // e = 0: E = M exactly (reduced into [0, 2 pi)).
    assert!((solve_kepler(1.0, 0.0) - 1.0).abs() < 1.0e-14);
    assert!((solve_kepler(1.0 + std::f64::consts::TAU, 0.0) - 1.0).abs() < 1.0e-12);
    // M = pi is a fixed point for every e: sin(pi) = 0.
    assert!((solve_kepler(std::f64::consts::PI, 0.9) - std::f64::consts::PI).abs() < 1.0e-12);
}

#[test]
fn circular_orbit_has_radius_a() {
    let (x, y) = orbital_plane_position(2.5, 0.0, 1.0);
    assert!((libm::hypot(x, y) - 2.5).abs() < 1.0e-12);
}

#[test]
fn orbital_plane_position_is_a_cos_e_minus_e_and_a_sqrt_1_minus_e2_sin_e() {
    // a = 2, e = 0.6, E = 1 rad: sqrt(1 - 0.36) = 0.8,
    //   x = 2 (cos 1 - 0.6) = 2 (0.5403023058681398 - 0.6) = -0.1193953882637204,
    //   y = 2 * 0.8 * sin 1 = 1.6 * 0.8414709848078965 = 1.3463535756926344.
    let (x, y) = orbital_plane_position(2.0, 0.6, 1.0);
    assert!((x - -0.119_395_388_263_720_4).abs() < 1.0e-12, "x={x}");
    assert!((y - 1.346_353_575_692_634_4).abs() < 1.0e-12, "y={y}");
    // Perihelion (E = 0): (a (1 - e), 0).  Aphelion (E = pi): (-a (1 + e), ~0).
    let (xp, yp) = orbital_plane_position(2.0, 0.6, 0.0);
    assert!((xp - 0.8).abs() < 1.0e-15 && yp == 0.0);
    let (xa, ya) = orbital_plane_position(2.0, 0.6, std::f64::consts::PI);
    assert!((xa - -3.2).abs() < 1.0e-12 && ya.abs() < 1.0e-12);
}
