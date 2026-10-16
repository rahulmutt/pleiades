use super::*;

#[test]
fn table_has_all_nineteen_bodies() {
    assert_eq!(TABLE.len(), 19);
}

#[test]
fn geocentric_bodies_are_earth_centered() {
    assert_eq!(
        elements_for(CelestialBody::WhiteMoon).unwrap().center,
        Center::Geocentric
    );
    assert_eq!(
        elements_for(CelestialBody::Waldemath).unwrap().center,
        Center::Geocentric
    );
    assert_eq!(
        elements_for(CelestialBody::Cupido).unwrap().center,
        Center::Heliocentric
    );
}

#[test]
fn non_fictitious_body_has_no_elements() {
    assert!(elements_for(CelestialBody::Mars).is_none());
}

fn circular_j2000(a: f64) -> KeplerElements {
    KeplerElements {
        epoch_jd_tt: crate::J2000_JD,
        a_au: [a, 0.0, 0.0],
        e: [0.0, 0.0, 0.0],
        incl_deg: [0.0, 0.0, 0.0],
        node_deg: [0.0, 0.0, 0.0],
        arg_peri_deg: [0.0, 0.0, 0.0],
        mean_anom_deg: [0.0, 0.0, 0.0],
        equinox: Equinox::Fixed(crate::J2000_JD),
        center: Center::Heliocentric,
    }
}

#[test]
fn zero_inclination_circular_orbit_lies_in_ecliptic_at_radius_a() {
    // At the epoch (t=0) the Kepler dmot term is zero, so the body sits at
    // mean anomaly 0 on a circle of radius a in the J2000 ecliptic plane.
    let (x, y, z) = circular_j2000(3.0).state_at(crate::J2000_JD);
    assert!(z.abs() < 1.0e-12, "z={z}");
    assert!((x.hypot(y) - 3.0).abs() < 1.0e-12);
}

#[test]
fn kepler_mean_motion_advances_a_non_t_term_body() {
    // Guard against the stationary-body bug: a body with no mean-anomaly
    // T-term must advance via Kepler's third law (dmot ∝ a^-1.5).
    let el = circular_j2000(3.0);
    let (x0, y0, _) = el.state_at(crate::J2000_JD);
    let (x1, y1, _) = el.state_at(crate::J2000_JD + 365.25);
    let sep = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();
    assert!(sep > 1.0e-3, "non-T-term body should advance; sep={sep}");
}
