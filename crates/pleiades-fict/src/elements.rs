//! Osculating orbital-element model (polynomial-in-time), transcribed from SE's
//! `seorbel.txt`. Each element is `c0 + c1·T + c2·T²`, `T` in Julian centuries
//! (36525 d) since the element epoch.

use crate::frame::rotate_ecliptic_to_j2000;
use crate::kepler::{orbital_plane_position, solve_kepler};

/// Gaussian mean-motion constant (SE `swi_osc_el_plan`): daily motion in
/// degrees is `MEAN_MOTION_DEG_PER_DAY / a^1.5`.
const MEAN_MOTION_DEG_PER_DAY: f64 = 0.9856076686;
/// Sun / Earth mass ratio (Earth only), SE `SUN_EARTH_MRAT`. Scales the mean
/// motion of geocentric-orbit bodies (central mass is Earth, not the Sun).
const SUN_EARTH_MRAT: f64 = 332946.050895;

/// Reference equinox the angular elements are expressed in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Equinox {
    /// Fixed equinox at a Julian Day (TT): `J1900` (2415020.0), `B1950`, `J2000`
    /// (2451545.0), or an arbitrary JD from `seorbel.txt`. `Fixed(2451545.0)` is
    /// identity (no precession).
    Fixed(f64),
    /// Equinox "of date" (`seorbel.txt` `JDATE`): the equinox is the evaluation
    /// instant; precess from the evaluation JD to J2000.
    OfDate,
}

/// Whether the osculating orbit is centered on the Sun or the Earth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Center {
    /// Heliocentric orbit — caller adds the Earth→Sun vector to geocentricize.
    Heliocentric,
    /// Geocentric orbit (White Moon, Waldemath) — already Earth-centered.
    Geocentric,
}

/// Classical osculating elements with linear/quadratic time terms.
#[derive(Clone, Copy, Debug)]
pub struct KeplerElements {
    /// Element epoch, Julian Day (TT).
    pub epoch_jd_tt: f64,
    /// Semi-major axis polynomial [c0, c1, c2], AU.
    pub a_au: [f64; 3],
    /// Eccentricity polynomial (dimensionless).
    pub e: [f64; 3],
    /// Inclination polynomial, degrees.
    pub incl_deg: [f64; 3],
    /// Longitude of ascending node polynomial, degrees.
    pub node_deg: [f64; 3],
    /// Argument of perihelion polynomial, degrees.
    pub arg_peri_deg: [f64; 3],
    /// Mean anomaly polynomial, degrees. A nonzero c1/c2 (T-term) supplies the
    /// body's mean motion directly; otherwise Kepler's third law is used.
    pub mean_anom_deg: [f64; 3],
    /// Reference equinox of the angular elements.
    pub equinox: Equinox,
    /// Orbit centering.
    pub center: Center,
}

fn poly(c: [f64; 3], t: f64) -> f64 {
    c[0] + c[1] * t + c[2] * t * t
}

impl KeplerElements {
    /// J2000-mean-ecliptic Cartesian position (AU) at `jd_tt`, in the elements'
    /// native centering (helio or geo). Mirrors SE `swi_osc_el_plan`: the
    /// mean-anomaly polynomial supplies any explicit T-term (Vulcan, Selena,
    /// Waldemath); bodies with no T-term advance by the Kepler-third-law mean
    /// motion. See Reconciliation §1.
    pub fn state_at(&self, jd_tt: f64) -> (f64, f64, f64) {
        let t = (jd_tt - self.epoch_jd_tt) / 36_525.0;
        let a = poly(self.a_au, t);
        let e = poly(self.e, t);
        let incl = poly(self.incl_deg, t).to_radians();
        let node = poly(self.node_deg, t).to_radians();
        let argp = poly(self.arg_peri_deg, t).to_radians();

        let mut mean_anom_deg = poly(self.mean_anom_deg, t);
        if self.mean_anom_deg[1] == 0.0 && self.mean_anom_deg[2] == 0.0 {
            // No T-term: advance mean anomaly by the Kepler-third-law daily motion.
            let mut dmot = MEAN_MOTION_DEG_PER_DAY / (a * a.sqrt());
            if self.center == Center::Geocentric {
                dmot /= SUN_EARTH_MRAT.sqrt();
            }
            mean_anom_deg += dmot * (jd_tt - self.epoch_jd_tt);
        }
        let mean_anom = mean_anom_deg.to_radians();

        let ea = solve_kepler(mean_anom, e);
        let (xo, yo) = orbital_plane_position(a, e, ea);

        // Rotate orbital plane → equinox-frame ecliptic by argp, incl, node
        // (classic 3-1-3). The matrix is linear, applied to the position vector.
        let (sa, ca) = argp.sin_cos();
        let (si, ci) = incl.sin_cos();
        let (sn, cn) = node.sin_cos();
        let xp = ca * xo - sa * yo;
        let yp = sa * xo + ca * yo;
        let x = cn * xp - sn * ci * yp;
        let y = sn * xp + cn * ci * yp;
        let z = si * yp;

        let equinox_jd = match self.equinox {
            Equinox::Fixed(jd) => jd,
            Equinox::OfDate => jd_tt,
        };
        rotate_ecliptic_to_j2000(x, y, z, equinox_jd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
