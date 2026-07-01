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
}
