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

/// Exercises the southern-perihelion branch (peri_vec[2] < 0), where
/// `omega = 2*PI - omega` runs. No other test reaches it. Elements are
/// recovered from a state built by the independent forward construction.
#[test]
fn elements_recover_southern_perihelion_argument() {
    let (a, e, incl, node, argp, mu) = (2.0, 0.2, 10.0, 40.0, 210.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, incl, node, argp, 50.0, mu);
    let el = elements_from_state(pos, vel, mu).unwrap();
    assert!((el.node_deg - node).abs() < 1e-9, "node {}", el.node_deg);
    assert!((el.incl_deg - incl).abs() < 1e-9, "incl {}", el.incl_deg);
    assert!(
        (el.peri_lon_deg - (node + argp)).abs() < 1e-9,
        "peri_lon {}",
        el.peri_lon_deg
    );
}

/// A non-finite semi-major axis must be rejected as NonFinite by the input
/// guard, not fall through to the later `a <= 0.0` unbound check. Setting
/// a = -inf with every other field finite distinguishes all four of the
/// guard's `&&` operators at once: each mutant reaches `a <= 0.0` and returns
/// UnboundOrbit instead.
#[test]
fn points_from_elements_rejects_non_finite_semi_major() {
    let el = KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: 0.5,
        semi_major_au: f64::NEG_INFINITY,
    };
    assert_eq!(
        points_from_elements(&el, false).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The eccentricity floor is exclusive: e exactly at MIN_ECCENTRICITY is
/// accepted, e below it is DegenerateOrbit. Here `e` is a caller-supplied
/// field, so the boundary is directly addressable.
#[test]
fn points_from_elements_eccentricity_floor_is_exclusive() {
    let mk = |e: f64| KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: e,
        semi_major_au: 2.0,
    };
    assert!(points_from_elements(&mk(MIN_ECCENTRICITY), false).is_ok());
    assert_eq!(
        points_from_elements(&mk(1e-7), false).unwrap_err(),
        ApsidesError::DegenerateOrbit
    );
}

/// Both halves of `e >= 1.0 || a <= 0.0` independently mean "not an ellipse".
#[test]
fn points_from_elements_rejects_unbound_conics() {
    let mk = |e: f64, a: f64| KeplerianElements {
        node_deg: 40.0,
        peri_lon_deg: 70.0,
        incl_deg: 10.0,
        eccentricity: e,
        semi_major_au: a,
    };
    assert_eq!(
        points_from_elements(&mk(1.5, 2.0), false).unwrap_err(),
        ApsidesError::UnboundOrbit
    );
    assert_eq!(
        points_from_elements(&mk(0.5, -2.0), false).unwrap_err(),
        ApsidesError::UnboundOrbit
    );
}

/// Overflow lens: each component is finite, but the squared norm overflows to
/// +inf. The guard must reject this. Under the mutant the function proceeds
/// and returns Ok, because p[i]/inf = 0 makes both output angles finite.
#[test]
fn to_ecliptic_rejects_overflowing_norm() {
    assert_eq!(
        to_ecliptic([1e200, 1e200, 1e200]).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// Underflow lens -- the mirror of the overflow case above, and the direction
/// this campaign had not been testing. Each component is finite and the norm
/// does NOT overflow; instead `fl(z*z)` UNDERFLOWS to a subnormal, losing bits
/// off the bottom. `sqrt` of that truncated square is then strictly less than
/// |z|, so `p[2] / r` comes out at 1.0000000000000002 -- just above 1 -- and
/// `asin` returns NaN. Meanwhile `atan2(0.0, 0.0)` is 0.0, perfectly finite.
///
/// So exactly ONE of the two output angles is non-finite. That is precisely
/// the region where `||` and `&&` differ, which is why this input kills the
/// output guard's `||` -> `&&` mutant: the original returns Err(NonFinite),
/// the mutant returns Ok carrying a NaN latitude. The line-76 input guard does
/// not catch it either -- r is finite and non-zero.
///
/// The bit pattern is written raw, not as a decimal: the kill depends on the
/// exact ulp at which fl(z*z) lands on the smallest subnormal, which a decimal
/// literal would not reliably round to.
#[test]
fn to_ecliptic_rejects_underflowing_norm() {
    // z = 2.222758749485078e-162; z*z underflows to 5e-324 (subnormal).
    let z = f64::from_bits(0x1e60000000000001);
    assert_eq!(
        to_ecliptic([0.0, 0.0, z]).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The same overflow lens at the `apsides` input boundary.
#[test]
fn apsides_rejects_overflowing_position_norm() {
    assert_eq!(
        apsides([1e200, 1e200, 1e200], [1e-60, 0.0, 0.0], 1.0).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// A non-positive gravitational parameter is not a physical system.
#[test]
fn apsides_rejects_non_positive_mu() {
    assert_eq!(
        apsides([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], -1.0).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// Radial motion has zero angular momentum, so no orbital plane and no node.
///
/// The velocity is crafted, not arbitrary: a plain radial state has e == 1.0
/// exactly, which makes r_peri = a(1-e) = 0 and fails inside apsides() before
/// the h_mag guard is reached. This vx leaves e one ulp below 1.
#[test]
fn radial_motion_has_no_orbital_plane() {
    let pos = [2.0, 0.0, 0.0];
    let vel = [f64::from_bits(0x3f50624dd2f1aa15), 0.0, 0.0];
    let mu = 2.959e-4;

    // Preconditions: the state is radial AND apsides() succeeds, or line 204
    // is unreachable and this test proves nothing.
    assert_eq!(cross(pos, vel), [0.0, 0.0, 0.0], "state must be radial");
    let aps = apsides(pos, vel, mu).expect("crafted state must form an ellipse");
    assert_ne!(aps.perigee.distance_au, 0.0, "r_peri must be non-zero");

    assert_eq!(
        elements_from_state(pos, vel, mu).unwrap_err(),
        ApsidesError::NonFinite
    );
}

/// The node-degeneracy floor is exclusive: a state sitting exactly on
/// n_mag == 1e-12 * h_mag is accepted.
#[test]
fn node_threshold_is_exclusive_at_the_exact_boundary() {
    let pos = [1.0, 0.0, 0.0];
    let vel = [0.0, 1.0, -1e-12];
    let mu = 0.8;

    // Precondition: the crafted state must sit exactly ON the threshold.
    let h = cross(pos, vel);
    let h_mag = norm(h);
    let n_mag = norm([-h[1], h[0], 0.0]);
    assert_eq!(
        n_mag,
        1e-12 * h_mag,
        "crafted state must sit ON the boundary"
    );

    assert!(elements_from_state(pos, vel, mu).is_ok());
}

/// The node floor scales with |h| multiplicatively, not inversely. At this
/// inclination n_mag sits above 1e-12*h_mag but below 1e-12/h_mag, so the two
/// formulations disagree.
#[test]
fn node_threshold_scales_multiplicatively_with_angular_momentum() {
    let (a, e, node, argp, mu) = (2.0, 0.2, 40.0, 30.0, 2.959e-4);
    let (pos, vel) = state_from_elements(a, e, 1e-10, node, argp, 50.0, mu);

    // Precondition: n_mag must lie strictly between the two formulations.
    let h = cross(pos, vel);
    let h_mag = norm(h);
    let n_mag = norm([-h[1], h[0], 0.0]);
    assert!(n_mag > 1e-12 * h_mag, "must be above the correct floor");
    assert!(n_mag < 1e-12 / h_mag, "must be below the mutated floor");

    let el = elements_from_state(pos, vel, mu).unwrap();
    assert!(el.incl_deg < 1e-8, "incl {}", el.incl_deg);
}

/// The eccentricity floor is exclusive: a state whose osculating eccentricity
/// is bit-identical to MIN_ECCENTRICITY is accepted, not rejected as
/// degenerate.
///
/// This test pins only the ON-boundary side; on its own it would also pass if
/// the floor check were deleted entirely. The below-boundary side is held by
/// the sibling `near_circular_orbit_is_degenerate`, and the two together give
/// the exclusivity.
///
/// Unlike `points_from_elements`, `e` is derived here, through a cancellation
/// that makes the reachable grid ~1e6x coarser than the target's precision.
/// This state was found by sweeping r_mag over consecutive doubles so the
/// final c1*r_mag product gets an independent rounding; sweeping powers of two
/// makes that product exact and never hits the boundary.
#[test]
fn apsides_eccentricity_floor_is_exclusive() {
    let rx = f64::from_bits(0x400000000005a740); // 2.0000000001645333
    let vy = f64::from_bits(0x3fe6a09f244b3b60); // 0.707107134710764
    let aps = apsides([rx, 0.0, 0.0], [0.0, vy, 0.0], 1.0).unwrap();

    // Precondition: the crafted state must sit exactly ON the boundary.
    assert_eq!(
        aps.eccentricity, MIN_ECCENTRICITY,
        "crafted state must sit ON the boundary"
    );
}

/// Documents the provenance of MU_EARTH_MOON_AU3_PER_DAY2 by recomputing it
/// from the published constants it cites, outside the code.
///
/// No mutant is attached to this constant (cargo-mutants does not mutate
/// consts); this test exists so the tuned value cannot drift from its
/// documented derivation unnoticed. The 2e-5 tolerance is the *measured*
/// 1.43e-5 gap between the pure derivation and the shipped value, which the
/// rustdoc attributes to tuning against the validate-lilith gate. It is not a
/// tolerance chosen to make the assertion pass.
#[test]
fn mu_matches_its_published_derivation() {
    const GM_EARTH_KM3_S2: f64 = 398_600.441_8;
    const GM_MOON_KM3_S2: f64 = 4_902.800;
    const AU_KM: f64 = 149_597_870.7;
    const DAY_S: f64 = 86_400.0;
    let derived = (GM_EARTH_KM3_S2 + GM_MOON_KM3_S2) * DAY_S * DAY_S / (AU_KM * AU_KM * AU_KM);
    let rel = (MU_EARTH_MOON_AU3_PER_DAY2 / derived - 1.0).abs();
    assert!(rel < 2e-5, "mu drifted from its derivation: rel {rel:e}");
}
