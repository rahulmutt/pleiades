//! Unit tests for the osculating-apsides geometry.

use super::*;

// A planar bound orbit with perigee on +x. At perigee the eccentricity
// vector points to +x, so the perigee longitude is 0° and the apogee is
// 180° at distance a(1+e).
fn perigee_on_x_state(a: f64, e: f64, mu: f64) -> ([f64; 3], [f64; 3]) {
    let r_peri = a * (1.0 - e);
    let v_peri = (mu / a * (1.0 + e) / (1.0 - e)).sqrt();
    ([r_peri, 0.0, 0.0], [0.0, v_peri, 0.0])
}

#[test]
fn apogee_is_opposite_perigee_at_correct_distance() {
    let a = 0.00257;
    let e = 0.05;
    let mu = MU_EARTH_MOON_AU3_PER_DAY2;
    let (pos, vel) = perigee_on_x_state(a, e, mu);
    let aps = apsides(pos, vel, mu).unwrap();

    assert!(
        (aps.eccentricity - e).abs() < 1e-9,
        "ecc {}",
        aps.eccentricity
    );
    assert!(
        (aps.semi_major_au - a).abs() < 1e-12,
        "a {}",
        aps.semi_major_au
    );
    assert!(
        (aps.perigee.longitude_deg - 0.0).abs() < 1e-6,
        "peri lon {}",
        aps.perigee.longitude_deg
    );
    assert!(
        (aps.apogee.longitude_deg - 180.0).abs() < 1e-6,
        "apo lon {}",
        aps.apogee.longitude_deg
    );
    assert!(
        (aps.apogee.distance_au - a * (1.0 + e)).abs() < 1e-12,
        "apo dist {}",
        aps.apogee.distance_au
    );
    assert!(
        (aps.perigee.distance_au - a * (1.0 - e)).abs() < 1e-12,
        "peri dist {}",
        aps.perigee.distance_au
    );
    assert!(aps.apogee.latitude_deg.abs() < 1e-9);
}

#[test]
fn near_circular_orbit_is_degenerate() {
    let a = 0.00257;
    let mu = MU_EARTH_MOON_AU3_PER_DAY2;
    let v_circ = (mu / a).sqrt();
    let err = apsides([a, 0.0, 0.0], [0.0, v_circ, 0.0], mu).unwrap_err();
    assert_eq!(err, ApsidesError::DegenerateOrbit);
}

#[test]
fn unbound_orbit_is_rejected() {
    let a = 0.00257;
    let mu = MU_EARTH_MOON_AU3_PER_DAY2;
    // Far above escape velocity → unbound (1/a <= 0).
    let v_escape = (2.0 * mu / a).sqrt() * 1.5;
    let err = apsides([a, 0.0, 0.0], [0.0, v_escape, 0.0], mu).unwrap_err();
    assert_eq!(err, ApsidesError::UnboundOrbit);
}

// An inclined orbit with perihelion placed at argument-of-perihelion ω from
// the +x-aligned ascending node: Ω = 40°, i = 10°, ω = 30°, e = 0.2, a = 2 AU.
fn inclined_elements() -> KeplerianElements {
    KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0, // ϖ = Ω + ω
        incl_deg: 10.0,
        eccentricity: 0.2,
        semi_major_au: 2.0,
    }
}

#[test]
fn points_from_elements_places_nodes_and_apsides() {
    let el = inclined_elements();
    let pts = points_from_elements(&el, false).unwrap();
    let p = el.semi_major_au * (1.0 - el.eccentricity * el.eccentricity);
    let omega = (el.peri_lon_deg - el.node_deg).to_radians();
    // Nodes lie in the reference plane at Ω / Ω+180 with the ellipse radius
    // at true anomaly ∓ω (argument of latitude 0 / π).
    assert!((pts.ascending.longitude_deg - 40.0).abs() < 1e-9);
    assert!(pts.ascending.latitude_deg.abs() < 1e-12);
    let r_asc = p / (1.0 + el.eccentricity * omega.cos());
    assert!((pts.ascending.distance_au - r_asc).abs() < 1e-12);
    assert!((pts.descending.longitude_deg - 220.0).abs() < 1e-9);
    let r_dsc = p / (1.0 - el.eccentricity * omega.cos());
    assert!((pts.descending.distance_au - r_dsc).abs() < 1e-12);
    // Perihelion at r = a(1−e), latitude sin β = sin i · sin ω.
    assert!((pts.perihelion.distance_au - 2.0 * 0.8).abs() < 1e-12);
    let beta = (el.incl_deg.to_radians().sin() * omega.sin())
        .asin()
        .to_degrees();
    assert!((pts.perihelion.latitude_deg - beta).abs() < 1e-9);
    // Aphelion opposite at r = a(1+e).
    assert!((pts.aphelion.distance_au - 2.0 * 1.2).abs() < 1e-12);
    assert!((pts.aphelion.latitude_deg + beta).abs() < 1e-9);
}

#[test]
fn second_focus_replaces_aphelion_distance_only() {
    let el = inclined_elements();
    let apo = points_from_elements(&el, false).unwrap().aphelion;
    let foc = points_from_elements(&el, true).unwrap().aphelion;
    assert!((foc.distance_au - 2.0 * el.semi_major_au * el.eccentricity).abs() < 1e-12);
    assert!((foc.longitude_deg - apo.longitude_deg).abs() < 1e-9);
    assert!((foc.latitude_deg - apo.latitude_deg).abs() < 1e-9);
}

#[test]
fn elements_from_state_round_trips_through_points() {
    // Build the state at perihelion of the inclined orbit analytically,
    // then recover the elements and check the perihelion point matches.
    let el = inclined_elements();
    let mu = 2.959e-4;
    let a = el.semi_major_au;
    let e = el.eccentricity;
    let r_peri = a * (1.0 - e);
    let v_peri = (mu / a * (1.0 + e) / (1.0 - e)).sqrt();
    let node = el.node_deg.to_radians();
    let incl = el.incl_deg.to_radians();
    let omega = (el.peri_lon_deg - el.node_deg).to_radians();
    // In-plane basis: P̂ toward perihelion, Q̂ 90° ahead in the motion.
    let rot = |u: f64, s: f64| -> [f64; 3] {
        [
            s * (u.cos() * node.cos() - u.sin() * incl.cos() * node.sin()),
            s * (u.cos() * node.sin() + u.sin() * incl.cos() * node.cos()),
            s * (u.sin() * incl.sin()),
        ]
    };
    let pos = rot(omega, r_peri);
    let vel = rot(omega + core::f64::consts::FRAC_PI_2, v_peri);
    let got = elements_from_state(pos, vel, mu).unwrap();
    assert!(
        (got.node_deg - el.node_deg).abs() < 1e-6,
        "node {}",
        got.node_deg
    );
    assert!((got.incl_deg - el.incl_deg).abs() < 1e-6);
    assert!((got.eccentricity - e).abs() < 1e-9);
    assert!((got.semi_major_au - a).abs() < 1e-9);
    assert!((got.peri_lon_deg - el.peri_lon_deg).abs() < 1e-6);
}

#[test]
fn zero_inclination_state_is_degenerate_node() {
    let mu = 2.959e-4;
    let (pos, vel) = perigee_on_x_state(2.0, 0.2, mu);
    let err = elements_from_state(pos, vel, mu).unwrap_err();
    assert_eq!(err, ApsidesError::DegenerateNode);
}
