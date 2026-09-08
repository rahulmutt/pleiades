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

/// Independent forward reference: Keplerian elements -> state vector, via the
/// published perifocal basis and the R3(-node)R1(-incl)R3(-argp) rotation.
/// Deliberately NOT the crate's inverse formulation, so a sign or operator
/// error in `apsides` cannot be masked by a shared expression.
fn state_from_elements(
    a: f64,
    e: f64,
    i_deg: f64,
    node_deg: f64,
    argp_deg: f64,
    nu_deg: f64,
    mu: f64,
) -> ([f64; 3], [f64; 3]) {
    let (i, o, w, nu) = (
        i_deg.to_radians(),
        node_deg.to_radians(),
        argp_deg.to_radians(),
        nu_deg.to_radians(),
    );
    let p = a * (1.0 - e * e);
    let r = p / (1.0 + e * nu.cos());
    let rp = [r * nu.cos(), r * nu.sin(), 0.0];
    let k = (mu / p).sqrt();
    let vp = [-k * nu.sin(), k * (e + nu.cos()), 0.0];
    let (co, so, ci, si, cw, sw) = (o.cos(), o.sin(), i.cos(), i.sin(), w.cos(), w.sin());
    let m = [
        [co * cw - so * sw * ci, -co * sw - so * cw * ci, so * si],
        [so * cw + co * sw * ci, -so * sw + co * cw * ci, -co * si],
        [sw * si, cw * si, ci],
    ];
    let rot = |q: [f64; 3]| -> [f64; 3] {
        [
            m[0][0] * q[0] + m[0][1] * q[1] + m[0][2] * q[2],
            m[1][0] * q[0] + m[1][1] * q[1] + m[1][2] * q[2],
            m[2][0] * q[0] + m[2][1] * q[1] + m[2][2] * q[2],
        ]
    };
    (rot(rp), rot(vp))
}

/// Pins both apsides at a geometry that is neither apsidal (r.v != 0, so the
/// c2 term is live) nor axis-aligned (all three e_hat components non-zero),
/// against the independent forward construction. Also checks the bifocal sum
/// r_apo + r_peri = 2a -- a defining property of the ellipse that the crate
/// never evaluates.
#[test]
fn apsides_match_forward_construction_at_non_degenerate_geometry() {
    let (a, e, incl, node, argp, mu) = (2.0, 0.2, 10.0, 40.0, 30.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, incl, node, argp, 50.0, mu);

    // Precondition: this state must break BOTH halves of the blind spot.
    let rv = pos[0] * vel[0] + pos[1] * vel[1] + pos[2] * vel[2];
    assert!(rv.abs() > 1e-6, "state must be non-apsidal, r.v = {rv}");

    let aps = apsides(pos, vel, mu).unwrap();
    assert!(
        (aps.eccentricity - e).abs() < 1e-12,
        "ecc {}",
        aps.eccentricity
    );
    assert!(
        (aps.semi_major_au - a).abs() < 1e-12,
        "a {}",
        aps.semi_major_au
    );

    // Independent expected apsis directions: forward-rotate the in-plane points
    // at true anomaly 0 (perihelion) and 180 (aphelion).
    let (pp, _) = state_from_elements(a, e, incl, node, argp, 0.0, mu);
    let (pa, _) = state_from_elements(a, e, incl, node, argp, 180.0, mu);
    for (got, want) in [(aps.perigee, pp), (aps.apogee, pa)] {
        let r = (want[0] * want[0] + want[1] * want[1] + want[2] * want[2]).sqrt();
        let lon = want[1].atan2(want[0]).to_degrees().rem_euclid(360.0);
        let lat = (want[2] / r).asin().to_degrees();
        assert!(
            (got.longitude_deg - lon).abs() < 1e-10,
            "lon {}",
            got.longitude_deg
        );
        assert!(
            (got.latitude_deg - lat).abs() < 1e-10,
            "lat {}",
            got.latitude_deg
        );
        assert!(
            (got.distance_au - r).abs() < 1e-12,
            "dist {}",
            got.distance_au
        );
    }

    // Conic invariant the crate never computes.
    assert!(
        (aps.apogee.distance_au + aps.perigee.distance_au - 2.0 * a).abs() < 1e-12,
        "bifocal sum"
    );
}
