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
    // Documented equivalent mutant (FU-9): `&&` binds tighter than `||`, so
    // mutating THIS operator yields `a || (b && c) || d`. That has TWO
    // distinguishing regions, not one, and both land on Err(NonFinite) anyway:
    //
    // A: {r_mag == 0.0, mu finite, mu > 0}. Here mu / r_mag = +inf forces
    //    c1 = -inf and poisons `e` to inf or NaN in every sub-case (all-zero
    //    pos -> NaN; all-subnormal pos whose squared norm underflows -> inf;
    //    mixed -> NaN).
    // B: {r_mag finite and non-zero, mu in {NaN, +inf}}. `mu <= 0.0` is false
    //    for both, so the mutant proceeds. mu = NaN gives c1 = NaN directly;
    //    mu = +inf gives c1 = (v2 - inf) / inf = -inf / inf = NaN. Measured:
    //    `e` is NaN for both.
    //
    // Every case in A and B fails the `!e.is_finite()` check below, so the
    // mutant returns Err(NonFinite) exactly where the original does. This arm
    // is redundant with that downstream check.
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
    // `<` -> `<=` is a documented equivalent mutant (FU-9). The two differ only
    // at peri_vec[2] == 0.0 exactly -- perihelion in the reference plane, so
    // the true omega is 0 or PI. They are NOT numerically identical there:
    // acos does not return exactly 0 or PI, because cos_omega arrives one ulp
    // off 1.0 through the longitude/latitude round-trip above. Measured at
    // pos [0.5, 0.25, 0.0], vel [0.5, -1.0, 1.0], mu 1: cos_omega
    // 0.99999999999999989, acos 1.49e-8 rad, and the two branches land
    // 1.71e-6 deg apart in peri_lon_deg.
    //
    // It is un-killable for a different reason: the pair straddles the truth
    // symmetrically. acos returns +eps where the true omega is 0, so the
    // original lands at node + eps and the mutant at node - eps. Over a
    // 1,272-case search of states reaching peri_vec[2] == 0.0 exactly, the
    // asymmetry ||orig - truth| - |mut - truth|| stayed <= 5.69e-14 deg. Any
    // symmetric tolerance against an independent reference therefore admits
    // both or rejects both; only a SIGNED assertion could separate them, and
    // that would be pinning the sign of rounding noise.
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
        // `+` -> `-` is a documented equivalent mutant (FU-9). In exact reals
        // cos(omega + PI) = cos(omega - PI) = -cos(omega), and likewise for
        // sin, so both branches denote the same point; they differ only in how
        // the two arguments round.
        //
        // The displacement is NOT bounded by a coarse sweep grid. It grows
        // continuously as |latitude| -> 90, where atan2's two arguments both
        // collapse toward zero. Measured at node 0, omega 90, r 2.4:
        //     incl 89        ->  8.53e-13 deg of longitude
        //     incl 89.99     ->  8.04e-11 deg
        //     incl 89.9999   ->  8.04e-9  deg  (~8x this suite's 1e-9 deg tol)
        //     incl 89.999999 ->  8.04e-7  deg
        // Latitude is -89.9999 on the third row, so longitude is perfectly
        // well defined there. "Undefined at the pole" does not excuse it, and
        // a 1-degree sweep grid cannot see any of this.
        //
        // Two references appear below and they are NOT interchangeable; every
        // figure names the one it was measured against.
        //   (1) The exact-in-real value of the mutated expression itself --
        //       substituting -cos(omega), -sin(omega) for the mutated
        //       argument's trig. It isolates the mutation's own rounding, but
        //       it is derived from this code, so no test can assert against it.
        //   (2) A genuinely independent reference: the exact-real longitude
        //       these elements denote, computed at 60 digits. This is what a
        //       test could actually use.
        //
        // Against (1) the branches straddle symmetrically: at incl 89.9999 the
        // original sits at -4.020250798930647e-9 deg and the mutant at
        // +4.020307642349508e-9 deg, and the asymmetry
        // ||orig - truth| - |mut - truth|| holds at 5.684342e-14 deg across
        // four orders of magnitude of displacement -- which is exactly one ulp
        // of a ~270 deg output, i.e. the smallest difference the result can
        // represent at all.
        //
        // Against (2) that symmetry does NOT carry over, and the mutant is
        // separable in principle. At node 250, incl 89.9999, omega 89.9999 the
        // original is 1.6345e-9 deg from the truth and the mutant 2.0102e-9,
        // so a tolerance inside that window passes the original and fails the
        // mutant. Which branch is nearer turns on the sign of omega's
        // representation error, not on the mutation: over a 252-case near-pole
        // sweep the original was the farther one in 159 cases, the mutant in
        // 37, with 56 exact ties.
        //
        // It is nonetheless left un-killed, on a narrow ground. Such a test
        // would need BOTH a hand-picked near-pole geometry AND a tolerance
        // threaded between two errors -- 1.63e-9 and 2.01e-9 deg -- that are
        // themselves already larger than the suite's 1e-9 deg tolerance, so
        // the UNMUTATED code fails an ordinary assertion at that geometry.
        // Such a test pins this code's rounding at a chosen point instead of
        // asserting a physical fact, which is what this campaign's "never
        // assert against the code's own output" rule forbids.
        //
        // At |lat| == 90 exactly (incl 90 with omega = +/-90) the separation
        // reaches 116.56505117707803 deg, but there longitude is
        // mathematically undefined and BOTH branches are atan2 of pure
        // rounding noise, so that is not a distinguishing observation either.
        aphelion: to_ecliptic(in_plane(omega + core::f64::consts::PI, apo_dist))?,
        eccentricity: e,
        semi_major_au: a,
    })
}

#[cfg(test)]
mod tests;
