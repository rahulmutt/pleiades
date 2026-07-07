//! Osculating lunar apsides — the true (osculating) apogee and perigee of the
//! Moon's instantaneous Kepler ellipse, derived from its geocentric position and
//! velocity. Computes the same osculating-apogee/perigee quantity Swiss
//! Ephemeris exposes as `SE_OSCU_APOG` ("True Black Moon Lilith"), distinct from
//! the smooth mean apogee. This crate is pure two-body geometry; its parity with
//! Swiss Ephemeris depends on the input state's accuracy. End-to-end, when fed
//! the packaged-Moon-derived state via `pleiades-data`, parity with SE is
//! validated to a max longitude residual of ~306″ by the `validate-lilith` gate
//! (3177 samples, 1900–2100).
//!
//! Frame-agnostic: the output ecliptic longitude/latitude are in the same frame
//! as the input Cartesian state — here, geocentric J2000 mean ecliptic.

#![deny(missing_docs)]

/// `G(M⊕ + M☾)` in AU³/day². Derived from GM⊕ = 398600.4418 km³/s² and
/// GM☾ = 4902.800 km³/s² (sum 403503.2418) with 1 AU = 149597870.7 km and
/// 1 day = 86400 s. Starting value; tuned against the `validate-lilith` gate
/// (the apse *direction* depends on μ, so it is the dominant parity knob).
pub const MU_EARTH_MOON_AU3_PER_DAY2: f64 = 8.997_14e-10;

/// One apsis expressed in the input frame's ecliptic longitude/latitude (deg)
/// and geocentric distance (AU).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApsisPoint {
    /// Ecliptic longitude of the apsis, degrees in `[0, 360)`, in the input
    /// state's frame (geocentric J2000 mean ecliptic for the packaged path).
    pub longitude_deg: f64,
    /// Ecliptic latitude of the apsis, degrees in `[-90, 90]`, in the same frame
    /// as [`ApsisPoint::longitude_deg`].
    pub latitude_deg: f64,
    /// Geocentric distance from Earth to the apsis point, in AU.
    pub distance_au: f64,
}

/// The osculating apogee and perigee plus the shape of the osculating ellipse.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Apsides {
    /// Far apsis of the osculating ellipse (True Black Moon Lilith / `SE_OSCU_APOG`).
    pub apogee: ApsisPoint,
    /// Near apsis of the osculating ellipse, 180° opposite the apogee in the
    /// orbital plane.
    pub perigee: ApsisPoint,
    /// Eccentricity of the osculating ellipse (dimensionless, `0 < e < 1` for a
    /// bound orbit).
    pub eccentricity: f64,
    /// Semi-major axis of the osculating ellipse, in AU.
    pub semi_major_au: f64,
}

/// Why an osculating apsis could not be formed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApsidesError {
    /// Eccentricity below the conditioning floor (apse direction ill-defined).
    DegenerateOrbit,
    /// Specific orbital energy is non-negative (not an ellipse).
    UnboundOrbit,
    /// Inclination below the conditioning floor (node direction ill-defined).
    DegenerateNode,
    /// A non-finite intermediate value was produced.
    NonFinite,
}

const MIN_ECCENTRICITY: f64 = 1e-6;

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

fn to_ecliptic(p: [f64; 3]) -> Result<ApsisPoint, ApsidesError> {
    let r = norm(p);
    if !r.is_finite() || r == 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    let longitude_deg = p[1].atan2(p[0]).to_degrees().rem_euclid(360.0);
    let latitude_deg = (p[2] / r).asin().to_degrees();
    if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    Ok(ApsisPoint {
        longitude_deg,
        latitude_deg,
        distance_au: r,
    })
}

/// Computes the osculating apogee and perigee from a geocentric state vector.
///
/// `pos_au` and `vel_au_per_day` are the geocentric position (AU) and velocity
/// (AU/day) of the Moon in the J2000 mean ecliptic frame; `mu` is `G(M⊕+M☾)` in
/// AU³/day².
pub fn apsides(
    pos_au: [f64; 3],
    vel_au_per_day: [f64; 3],
    mu: f64,
) -> Result<Apsides, ApsidesError> {
    let r = pos_au;
    let v = vel_au_per_day;
    let r_mag = norm(r);
    if !r_mag.is_finite() || r_mag == 0.0 || !mu.is_finite() || mu <= 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    let v2 = dot(v, v);
    let rv = dot(r, v);

    // Eccentricity vector: e = ((v·v − μ/r) r − (r·v) v) / μ. Points to perigee.
    let c1 = (v2 - mu / r_mag) / mu;
    let c2 = rv / mu;
    let e_vec = [
        c1 * r[0] - c2 * v[0],
        c1 * r[1] - c2 * v[1],
        c1 * r[2] - c2 * v[2],
    ];
    let e = norm(e_vec);
    if !e.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    if e < MIN_ECCENTRICITY {
        return Err(ApsidesError::DegenerateOrbit);
    }

    // Semi-major axis: a = 1 / (2/r − v²/μ). Non-positive inverse ⇒ unbound.
    let inv_a = 2.0 / r_mag - v2 / mu;
    if !inv_a.is_finite() {
        return Err(ApsidesError::NonFinite);
    }
    if inv_a <= 0.0 {
        return Err(ApsidesError::UnboundOrbit);
    }
    let a = 1.0 / inv_a;

    let e_hat = [e_vec[0] / e, e_vec[1] / e, e_vec[2] / e];
    let r_apo = a * (1.0 + e);
    let r_peri = a * (1.0 - e);
    let apo_pos = [-e_hat[0] * r_apo, -e_hat[1] * r_apo, -e_hat[2] * r_apo];
    let peri_pos = [e_hat[0] * r_peri, e_hat[1] * r_peri, e_hat[2] * r_peri];

    Ok(Apsides {
        apogee: to_ecliptic(apo_pos)?,
        perigee: to_ecliptic(peri_pos)?,
        eccentricity: e,
        semi_major_au: a,
    })
}

/// Osculating Keplerian elements of an elliptical orbit, referred to the input
/// state's frame: longitude of ascending node Ω, longitude of perihelion
/// ϖ = Ω + ω, inclination i (all degrees), eccentricity, semi-major axis (AU).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeplerianElements {
    /// Longitude of the ascending node Ω, degrees in `[0, 360)`.
    pub node_deg: f64,
    /// Longitude of perihelion ϖ = Ω + ω (ω in-plane), degrees in `[0, 360)`.
    pub peri_lon_deg: f64,
    /// Inclination to the reference plane, degrees in `[0, 180)`.
    pub incl_deg: f64,
    /// Eccentricity (`0 < e < 1` for a bound orbit).
    pub eccentricity: f64,
    /// Semi-major axis, AU.
    pub semi_major_au: f64,
}

/// The four singular orbital points of an ellipse: both nodes and both apsides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitalPoints {
    /// Point on the ellipse at the ascending node.
    pub ascending: ApsisPoint,
    /// Point on the ellipse at the descending node.
    pub descending: ApsisPoint,
    /// Near apsis (perihelion/perigee).
    pub perihelion: ApsisPoint,
    /// Far apsis (aphelion/apogee) — or the ellipse's second (empty) focus at
    /// distance `2ae` in the same direction when requested.
    pub aphelion: ApsisPoint,
    /// Eccentricity of the ellipse.
    pub eccentricity: f64,
    /// Semi-major axis, AU.
    pub semi_major_au: f64,
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Osculating elements from a state vector. Frame-agnostic: Ω/ϖ/i are referred
/// to the input frame's reference plane and +x origin of longitude.
pub fn elements_from_state(
    pos_au: [f64; 3],
    vel_au_per_day: [f64; 3],
    mu: f64,
) -> Result<KeplerianElements, ApsidesError> {
    // Reuse the eccentricity/energy machinery (and its error taxonomy).
    let aps = apsides(pos_au, vel_au_per_day, mu)?;
    let h = cross(pos_au, vel_au_per_day);
    let h_mag = norm(h);
    if !h_mag.is_finite() || h_mag == 0.0 {
        return Err(ApsidesError::NonFinite);
    }
    // Node vector n = ẑ × h points to the ascending node.
    let n = [-h[1], h[0], 0.0];
    let n_mag = norm(n);
    if n_mag < 1e-12 * h_mag {
        return Err(ApsidesError::DegenerateNode);
    }
    let node_deg = n[1].atan2(n[0]).to_degrees().rem_euclid(360.0);
    let incl_deg = (h[2] / h_mag).acos().to_degrees();
    // ϖ = Ω + ω where ω is the in-plane angle from the node to perihelion; the
    // perihelion direction is the apsides() perigee point's unit vector.
    let peri = aps.perigee;
    let peri_vec = {
        let lon = peri.longitude_deg.to_radians();
        let lat = peri.latitude_deg.to_radians();
        [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
    };
    let n_hat = [n[0] / n_mag, n[1] / n_mag, 0.0];
    let cos_omega = dot(n_hat, peri_vec).clamp(-1.0, 1.0);
    let mut omega = cos_omega.acos();
    if peri_vec[2] < 0.0 {
        omega = 2.0 * core::f64::consts::PI - omega;
    }
    Ok(KeplerianElements {
        node_deg,
        peri_lon_deg: (node_deg + omega.to_degrees()).rem_euclid(360.0),
        incl_deg,
        eccentricity: aps.eccentricity,
        semi_major_au: aps.semi_major_au,
    })
}

/// The four orbital points from Keplerian elements, in the elements' frame.
/// With `second_focus`, the aphelion slot instead carries the empty focus at
/// distance `2ae` (Swiss Ephemeris `SE_NODBIT_FOPOINT`); direction unchanged.
pub fn points_from_elements(
    elements: &KeplerianElements,
    second_focus: bool,
) -> Result<OrbitalPoints, ApsidesError> {
    let e = elements.eccentricity;
    let a = elements.semi_major_au;
    if !(e.is_finite()
        && a.is_finite()
        && elements.node_deg.is_finite()
        && elements.peri_lon_deg.is_finite()
        && elements.incl_deg.is_finite())
    {
        return Err(ApsidesError::NonFinite);
    }
    if e < MIN_ECCENTRICITY {
        return Err(ApsidesError::DegenerateOrbit);
    }
    if e >= 1.0 || a <= 0.0 {
        return Err(ApsidesError::UnboundOrbit);
    }
    let node = elements.node_deg.to_radians();
    let incl = elements.incl_deg.to_radians();
    let omega = (elements.peri_lon_deg - elements.node_deg).to_radians();
    let p = a * (1.0 - e * e);
    // In-plane point at argument-of-latitude u (angle from the ascending node),
    // rotated into the reference frame.
    let in_plane = |u: f64, r: f64| -> [f64; 3] {
        [
            r * (u.cos() * node.cos() - u.sin() * incl.cos() * node.sin()),
            r * (u.cos() * node.sin() + u.sin() * incl.cos() * node.cos()),
            r * (u.sin() * incl.sin()),
        ]
    };
    // At the ascending node u = 0 and ν = −ω (so cos ν = cos ω); descending
    // node u = π, ν = π − ω.
    let r_asc = p / (1.0 + e * omega.cos());
    let r_dsc = p / (1.0 - e * omega.cos());
    let apo_dist = if second_focus {
        2.0 * a * e
    } else {
        a * (1.0 + e)
    };
    Ok(OrbitalPoints {
        ascending: to_ecliptic(in_plane(0.0, r_asc))?,
        descending: to_ecliptic(in_plane(core::f64::consts::PI, r_dsc))?,
        perihelion: to_ecliptic(in_plane(omega, a * (1.0 - e)))?,
        aphelion: to_ecliptic(in_plane(omega + core::f64::consts::PI, apo_dist))?,
        eccentricity: e,
        semi_major_au: a,
    })
}

#[cfg(test)]
mod tests {
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
}
