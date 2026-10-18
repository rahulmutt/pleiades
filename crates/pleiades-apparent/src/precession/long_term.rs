//! Long-term precession and mean obliquity of Vondrák, Capitaine & Wallace,
//! "New precession expressions, valid for long time intervals", A&A 534, A22
//! (2011), with the A&A 541, C1 (2012) corrigendum. Valid for ±200 000 years
//! around J2000, against the few centuries of the IAU-1976 expressions in the
//! parent module.
//!
//! This is the default precession model of Swiss Ephemeris since 2.0, and the
//! port follows its `swephlib.c` (`swi_ldp_peps`, `pre_pecl`, `pre_pequ`,
//! `pre_pmat`) term for term, so that bodies defined against SE (the
//! `seorbel.txt` fictitious bodies, whose element equinoxes reach back to
//! about 370 CE) are carried to J2000 the way SE carries them.

use crate::error::ApparentPlaceError;

const J2000: f64 = 2_451_545.0;
const ARCSEC_TO_RAD: f64 = std::f64::consts::PI / (180.0 * 3600.0);
/// Obliquity at J2000 used to orient the ecliptic-pole series, arcseconds.
const EPS0_ARCSEC: f64 = 84_381.406;

/// Obliquity series (Vondrák 2011 Table 1): general precession in longitude
/// and obliquity, polynomial coefficients `[p, ε]` in arcseconds per T^k.
#[rustfmt::skip]
const PEPOL: [[f64; 2]; 4] = [
    [ 8134.017132,   84028.206305],
    [ 5043.0520035,      0.3624445],
    [   -0.00710733,    -0.00004039],
    [    0.000000271,   -0.000000110],
];
/// Periodic terms: period (centuries), then cos p, cos ε, sin p, sin ε.
#[rustfmt::skip]
const PEPER: [[f64; 10]; 5] = [
    [409.90, 396.15, 537.22, 402.90, 417.15, 288.92, 4043.00, 306.00, 277.00, 203.00],
    [-6908.287473, -3198.706291, 1453.674527, -857.748557, 1173.231614, -156.981465, 371.836550, -216.619040, 193.691479, 11.891524],
    [753.872780, -247.805823, 379.471484, -53.880558, -90.109153, -353.600190, -63.115353, -28.248187, 17.703387, 38.911307],
    [-2845.175469, 449.844989, -1255.915323, 886.736783, 418.887514, 997.912441, -240.979710, 76.541307, -36.788069, -170.964086],
    [-1704.720302, -862.308358, 447.832178, -889.571909, 190.402846, -56.564991, -296.222622, -75.859952, 67.473503, 3.014055],
];

/// Ecliptic-pole series (Table 2): `[P_A, Q_A]` polynomial, arcseconds.
#[rustfmt::skip]
const PQPOL: [[f64; 2]; 4] = [
    [5851.607687, -1600.886300],
    [  -0.1189000,    1.1689818],
    [  -0.00028913,  -0.00000020],
    [   0.000000101, -0.000000437],
];
/// Periodic terms: period, cos P, cos Q, sin P, sin Q. The 198.296701 in the
/// cos Q row is the 2012 corrigendum's value (the 2011 paper printed 198.296071).
#[rustfmt::skip]
const PQPER: [[f64; 8]; 5] = [
    [708.15, 2309.0, 1620.0, 492.2, 1183.0, 622.0, 882.0, 547.0],
    [-5486.751211, -17.127623, -617.517403, 413.44294, 78.614193, -180.732815, -87.676083, 46.140315],
    [-684.66156, 2446.28388, 399.671049, -356.652376, -186.387003, -316.80007, 198.296701, 101.135679],
    [667.66673, -2354.886252, -428.152441, 376.202861, 184.778874, 335.321713, -185.138669, -120.97283],
    [-5523.863691, -549.74745, -310.998056, 421.535876, -36.776172, -145.278396, -34.74445, 22.885731],
];

/// Equator-pole series (Table 3): `[X_A, Y_A]` polynomial, arcseconds.
#[rustfmt::skip]
const XYPOL: [[f64; 2]; 4] = [
    [5453.282155, -73750.930350],
    [   0.4252841,     -0.7675452],
    [  -0.00037173,    -0.00018725],
    [  -0.000000152,    0.000000231],
];
/// Periodic terms: period, cos X, cos Y, sin X, sin Y.
#[rustfmt::skip]
const XYPER: [[f64; 14]; 5] = [
    [256.75, 708.15, 274.2, 241.45, 2309.0, 492.2, 396.1, 288.9, 231.1, 1610.0, 620.0, 157.87, 220.3, 1200.0],
    [-819.940624, -8444.676815, 2600.009459, 2755.17563, -167.659835, 871.855056, 44.769698, -512.313065, -819.415595, -538.071099, -189.793622, -402.922932, 179.516345, -9.814756],
    [75004.344875, 624.033993, 1251.136893, -1102.212834, -2660.66498, 699.291817, 153.16722, -950.865637, 499.754645, -145.18821, 558.116553, -23.923029, -165.405086, 9.344131],
    [81491.287984, 787.163481, 1251.296102, -1257.950837, -2966.79973, 639.744522, 131.600209, -445.040117, 584.522874, -89.756563, 524.42963, -13.549067, -210.157124, -44.919798],
    [1558.515853, 7774.939698, -2219.534038, -2523.969396, 247.850422, -846.485643, -1393.124055, 368.526116, 749.045012, 444.704518, 235.934465, 374.049623, -171.33018, -22.899655],
];

/// Evaluates one Vondrák series pair at `t` Julian centuries from J2000:
/// periodic terms first, then the cubic, in arcseconds (SE's summation order).
fn series<const N: usize>(pol: &[[f64; 2]; 4], per: &[[f64; N]; 5], t: f64) -> (f64, f64) {
    let (mut a, mut b) = (0.0, 0.0);
    for i in 0..N {
        let (s, c) = (std::f64::consts::TAU * t / per[0][i]).sin_cos();
        a += c * per[1][i] + s * per[3][i];
        b += c * per[2][i] + s * per[4][i];
    }
    let mut w = 1.0;
    for row in pol {
        a += row[0] * w;
        b += row[1] * w;
        w *= t;
    }
    (a, b)
}

fn centuries(jd_tt: f64) -> f64 {
    (jd_tt - J2000) / 36_525.0
}

/// Mean obliquity of the ecliptic at `jd_tt` (TT), radians, from the Vondrák
/// 2011 long-term series. At J2000 it is 84381.406″, the IAU 2006 value (the
/// IAU-1976 [`crate::nutation::mean_obliquity_degrees`] gives 84381.448″).
pub fn long_term_mean_obliquity_radians(jd_tt: f64) -> f64 {
    series(&PEPOL, &PEPER, centuries(jd_tt)).1 * ARCSEC_TO_RAD
}

/// Rows of the precession matrix from J2000 to `jd_tt`: the mean equinox,
/// the equator's 90° point and the equator pole of date, in J2000 equatorial
/// coordinates (SE `pre_pmat`).
fn precession_matrix(jd_tt: f64) -> [[f64; 3]; 3] {
    let t = centuries(jd_tt);

    let (x, y) = series(&XYPOL, &XYPER, t);
    let (x, y) = (x * ARCSEC_TO_RAD, y * ARCSEC_TO_RAD);
    let w = x * x + y * y;
    let equator_pole = [x, y, if w < 1.0 { (1.0 - w).sqrt() } else { 0.0 }];

    let (p, q) = series(&PQPOL, &PQPER, t);
    let (p, q) = (p * ARCSEC_TO_RAD, q * ARCSEC_TO_RAD);
    let z = (1.0 - p * p - q * q).max(0.0).sqrt();
    let (s, c) = (EPS0_ARCSEC * ARCSEC_TO_RAD).sin_cos();
    let ecliptic_pole = [p, -q * c - z * s, -q * s + z * c];

    let v = cross(equator_pole, ecliptic_pole);
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let equinox = [v[0] / n, v[1] / n, v[2] / n];
    [equinox, cross(equator_pole, equinox), equator_pole]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Rotates about the x-axis from equatorial to ecliptic coordinates by the
/// obliquity `eps` (radians); `-eps` goes the other way (SE `swi_coortrf`).
fn rotate_x(v: [f64; 3], eps: f64) -> [f64; 3] {
    let (s, c) = eps.sin_cos();
    [v[0], v[1] * c + v[2] * s, -v[1] * s + v[2] * c]
}

/// Precesses an ecliptic Cartesian vector from the mean ecliptic and equinox
/// of `equinox_jd_tt` (TT) to the J2000 mean ecliptic and equinox with the
/// Vondrák 2011 long-term model.
///
/// The vector goes to the equator of date with the long-term obliquity at
/// `equinox_jd_tt`, through the precession matrix to the J2000 equator, and
/// back to the ecliptic with the long-term obliquity at J2000 (84381.406″), as
/// Swiss Ephemeris does for osculating elements (`swi_osc_el_plan`). It is a
/// rotation, so it applies to positions and velocities alike and preserves
/// length. At exactly J2000 the vector is returned unchanged.
///
/// Use it for epochs far from J2000, where the IAU-1976
/// [`crate::precess_ecliptic_date_to_j2000`] drifts: 16 centuries back the two
/// differ by about 1.4″.
///
/// # Errors
///
/// Returns [`ApparentPlaceError::NonFiniteCorrection`] if a component of `v`
/// or `equinox_jd_tt` is not finite.
///
/// # Examples
///
/// ```
/// use pleiades_apparent::precess_ecliptic_vector_date_to_j2000_long_term;
///
/// // A unit vector toward the J1900 equinox lands about 1.4° east of the J2000
/// // equinox (a century of general precession, ~5029″).
/// let v = precess_ecliptic_vector_date_to_j2000_long_term([1.0, 0.0, 0.0], 2_415_020.0).unwrap();
/// let lon_arcsec = v[1].atan2(v[0]).to_degrees() * 3600.0;
/// assert!((lon_arcsec - 5029.0).abs() < 2.0);
/// ```
pub fn precess_ecliptic_vector_date_to_j2000_long_term(
    v: [f64; 3],
    equinox_jd_tt: f64,
) -> Result<[f64; 3], ApparentPlaceError> {
    if !equinox_jd_tt.is_finite() || v.iter().any(|c| !c.is_finite()) {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "precession",
        });
    }
    if equinox_jd_tt == J2000 {
        return Ok(v);
    }
    let equatorial_of_date = rotate_x(v, -long_term_mean_obliquity_radians(equinox_jd_tt));
    let m = precession_matrix(equinox_jd_tt);
    // Date -> J2000 is the transpose: sum each row weighted by the component.
    let mut equatorial_j2000 = [0.0; 3];
    for (k, out) in equatorial_j2000.iter_mut().enumerate() {
        *out = equatorial_of_date[0] * m[0][k]
            + equatorial_of_date[1] * m[1][k]
            + equatorial_of_date[2] * m[2][k];
    }
    Ok(rotate_x(
        equatorial_j2000,
        long_term_mean_obliquity_radians(J2000),
    ))
}

#[cfg(test)]
mod tests;
