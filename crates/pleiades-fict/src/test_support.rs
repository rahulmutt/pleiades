//! Shared fixtures for the unit tests: hand-built element sets whose expected
//! positions were worked out independently of this crate (textbook Kepler
//! solution by bisection, 3-1-3 rotation matrices), and a closeness helper.
//!
//! Every set is epoched at J2000 with a J2000 equinox, so the equinox rotation
//! is the identity and only the Kepler/orientation arithmetic is exercised.
//! The polynomials are chosen with non-zero `c1` and `c2` terms and generic
//! angles, so no element, rotation term or time term can drop out unnoticed.

use crate::elements::{Center, Equinox, KeplerElements};
use crate::J2000_JD;

/// Julian centuries are 36525 days; `J2000_JD + 0.5 * 36525` is `t = 0.5`.
pub const HALF_CENTURY_DAYS: f64 = 18_262.5;
/// `t = -0.3` centuries.
pub const MINUS_0_3_CENTURY_DAYS: f64 = -10_957.5;

fn set(
    a: [f64; 3],
    e: [f64; 3],
    incl: [f64; 3],
    node: [f64; 3],
    argp: [f64; 3],
    mean_anom: [f64; 3],
    center: Center,
) -> KeplerElements {
    KeplerElements {
        epoch_jd_tt: J2000_JD,
        a_au: a,
        e,
        incl_deg: incl,
        node_deg: node,
        arg_peri_deg: argp,
        mean_anom_deg: mean_anom,
        equinox: Equinox::Fixed(J2000_JD),
        center,
    }
}

/// Heliocentric, no mean-anomaly T-term: advances by the Kepler-third-law
/// daily motion `0.9856076686 / a^1.5` deg/day.
pub fn helio_kepler_third_law() -> KeplerElements {
    set(
        [5.0, 0.1, 0.01],
        [0.3, 0.02, 0.01],
        [10.0, 2.0, 0.5],
        [100.0, 3.0, 1.0],
        [45.0, 4.0, 2.0],
        [30.0, 0.0, 0.0],
        Center::Heliocentric,
    )
}

/// The same orbit as [`helio_kepler_third_law`], Earth-centred: the daily
/// motion is further divided by `sqrt(332946.050895)`.
pub fn geo_kepler_third_law() -> KeplerElements {
    KeplerElements {
        center: Center::Geocentric,
        ..helio_kepler_third_law()
    }
}

/// Geocentric with a linear mean-anomaly T-term (`c1 = 3 * 36525`, i.e. 3
/// deg/day), so the third-law motion must not be added on top.
pub fn geo_linear_mean_anomaly() -> KeplerElements {
    set(
        [1.5, 0.05, 0.0],
        [0.3, 0.01, 0.0],
        [20.0, 1.0, 0.0],
        [40.0, 2.0, 0.0],
        [60.0, 3.0, 0.0],
        [10.0, 109_575.0, 0.0],
        Center::Geocentric,
    )
}

/// Geocentric with only a quadratic mean-anomaly term (`c1 = 0`, `c2 = 50`):
/// still a T-term, so the third-law motion must not be added.
pub fn geo_quadratic_mean_anomaly() -> KeplerElements {
    KeplerElements {
        mean_anom_deg: [10.0, 0.0, 50.0],
        ..geo_linear_mean_anomaly()
    }
}

/// A circular, uninclined heliocentric orbit of radius `a` at mean anomaly 0
/// at J2000: at the epoch the body is at `(a, 0, 0)`.
pub fn circular_helio_at_epoch(a: f64) -> KeplerElements {
    set(
        [a, 0.0, 0.0],
        [0.0; 3],
        [0.0; 3],
        [0.0; 3],
        [0.0; 3],
        [0.0; 3],
        Center::Heliocentric,
    )
}

/// Assert `|actual - expected| <= tol`, naming the quantity.
#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tol: f64, what: &str) {
    assert!(
        (actual - expected).abs() <= tol,
        "{what}: {actual} vs {expected} (tol {tol})"
    );
}

/// Assert each component of an `(x, y, z)` triple is within `tol`.
#[track_caller]
pub fn assert_xyz(actual: (f64, f64, f64), expected: [f64; 3], tol: f64, what: &str) {
    assert_close(actual.0, expected[0], tol, &format!("{what} x"));
    assert_close(actual.1, expected[1], tol, &format!("{what} y"));
    assert_close(actual.2, expected[2], tol, &format!("{what} z"));
}
