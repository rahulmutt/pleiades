//! Swiss Ephemeris mean orbital elements for Mercury–Neptune (+ Earth, used by
//! the Sun per SE semantics), transcribed verbatim from `swecl.c:5000-5063`
//! ("mean elements for Mercury - Neptune from VSOP87 (mean equinox of date)").
//! Rows: 0 Mercury, 1 Venus, 2 Earth, 3 Mars, 4 Jupiter, 5 Saturn, 6 Uranus,
//! 7 Neptune. Columns: polynomial coefficients in `t = (jd_tdb − J2000)/36525`.
//! Angles in degrees, semi-major axis in AU, referred to the MEAN equinox and
//! ecliptic of date.

use pleiades_apsides::KeplerianElements;
use pleiades_types::CelestialBody;

pub(crate) const J2000_JD: f64 = 2_451_545.0;

#[rustfmt::skip]
const EL_NODE: [[f64; 4]; 8] = [
    [ 48.330893,  1.1861890,  0.00017587,  0.000000211],  // Mercury
    [ 76.679920,  0.9011190,  0.00040665, -0.000000080],  // Venus
    [  0.0,       0.0,        0.0,         0.0        ],  // Earth
    [ 49.558093,  0.7720923,  0.00001605,  0.000002325],  // Mars
    [100.464441,  1.0209550,  0.00040117,  0.000000569],  // Jupiter
    [113.665524,  0.8770970, -0.00012067, -0.000002380],  // Saturn
    [ 74.005947,  0.5211258,  0.00133982,  0.000018516],  // Uranus
    [131.784057,  1.1022057,  0.00026006, -0.000000636],  // Neptune
];
#[rustfmt::skip]
const EL_PERI: [[f64; 4]; 8] = [
    [ 77.456119,  1.5564775,  0.00029589,  0.000000056],
    [131.563707,  1.4022188, -0.00107337, -0.000005315],
    [102.937348,  1.7195269,  0.00045962,  0.000000499],
    [336.060234,  1.8410331,  0.00013515,  0.000000318],
    [ 14.331309,  1.6126668,  0.00103127, -0.000004569],
    [ 93.056787,  1.9637694,  0.00083757,  0.000004899],
    [173.005159,  1.4863784,  0.00021450,  0.000000433],
    [ 48.123691,  1.4262677,  0.00037918, -0.000000003],
];
#[rustfmt::skip]
const EL_INCL: [[f64; 4]; 8] = [
    [  7.004986,  0.0018215, -0.00001809,  0.000000053],
    [  3.394662,  0.0010037, -0.00000088, -0.000000007],
    [  0.0,       0.0,        0.0,         0.0        ],
    [  1.849726, -0.0006010,  0.00001276, -0.000000006],
    [  1.303270, -0.0054966,  0.00000465, -0.000000004],
    [  2.488878, -0.0037363, -0.00001516,  0.000000089],
    [  0.773196,  0.0007744,  0.00003749, -0.000000092],
    [  1.769952, -0.0093082, -0.00000708,  0.000000028],
];
#[rustfmt::skip]
const EL_ECCE: [[f64; 4]; 8] = [
    [  0.20563175,  0.000020406, -0.0000000284, -0.00000000017],
    [  0.00677188, -0.000047766,  0.0000000975,  0.00000000044],
    [  0.01670862, -0.000042037, -0.0000001236,  0.00000000004],
    [  0.09340062,  0.000090483, -0.0000000806, -0.00000000035],
    [  0.04849485,  0.000163244, -0.0000004719, -0.00000000197],
    [  0.05550862, -0.000346818, -0.0000006456,  0.00000000338],
    [  0.04629590, -0.000027337,  0.0000000790,  0.00000000025],
    [  0.00898809,  0.000006408, -0.0000000008, -0.00000000005],
];
#[rustfmt::skip]
const EL_SEMA: [[f64; 4]; 8] = [
    [  0.387098310,  0.0,           0.0,           0.0],
    [  0.723329820,  0.0,           0.0,           0.0],
    [  1.000001018,  0.0,           0.0,           0.0],
    [  1.523679342,  0.0,           0.0,           0.0],
    [  5.202603191,  0.0000001913,  0.0,           0.0],
    [  9.554909596,  0.0000021389,  0.0,           0.0],
    [ 19.218446062, -0.0000000372,  0.00000000098, 0.0],
    [ 30.110386869, -0.0000001663,  0.00000000069, 0.0],
];

/// Sun/planet mass ratios (`swecl.c:5051-5062`): Mercury, Venus, Earth+Moon,
/// Mars, Jupiter, Saturn, Uranus, Neptune, Pluto.
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
pub(crate) const SUN_MASS_RATIO: [f64; 9] = [
    6_023_600.0,
    408_523.719,
    328_900.5,
    3_098_703.59,
    1_047.348_644,
    3_497.901_8,
    22_902.98,
    19_412.26,
    136_566_000.0,
];

/// SE physical constants (sweph.h:260-279).
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
pub(crate) const EARTH_MOON_MASS_RATIO: f64 = 81.300_569_074_190_62;
const AUNIT_M: f64 = 1.495_978_707_00e11;
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
const HELGRAVCONST_M3_S2: f64 = 1.327_124_400_179_87e20;
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
const GEOGCONST_M3_S2: f64 = 3.986_004_48e14;
pub(crate) const MOON_MEAN_INCL_DEG: f64 = 5.145_396_4;
pub(crate) const MOON_MEAN_ECC: f64 = 0.054_900_489;
pub(crate) const MOON_MEAN_SEMA_AU: f64 = 384_400_000.0 / AUNIT_M;

/// Solar GM in AU³/day², from SE's HELGRAVCONST.
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
pub(crate) const SUN_GM_AU3_DAY2: f64 =
    HELGRAVCONST_M3_S2 / (AUNIT_M * AUNIT_M * AUNIT_M) * (86_400.0 * 86_400.0);
/// Geocentric Earth+Moon GM in AU³/day², from SE's GEOGCONST.
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
pub(crate) const GEO_GM_AU3_DAY2: f64 = GEOGCONST_M3_S2 / (AUNIT_M * AUNIT_M * AUNIT_M)
    * (86_400.0 * 86_400.0)
    * (1.0 + 1.0 / EARTH_MOON_MASS_RATIO);

/// Row index into the element tables for a mean-capable body; `None` for
/// bodies without SE mean elements. The Sun maps to Earth's elements
/// (`ipl_to_elem[0] = 2`, swecl.c:5063).
pub(crate) fn elem_index(body: &CelestialBody) -> Option<usize> {
    match body {
        CelestialBody::Sun => Some(2),
        CelestialBody::Mercury => Some(0),
        CelestialBody::Venus => Some(1),
        CelestialBody::Mars => Some(3),
        CelestialBody::Jupiter => Some(4),
        CelestialBody::Saturn => Some(5),
        CelestialBody::Uranus => Some(6),
        CelestialBody::Neptune => Some(7),
        _ => None,
    }
}

/// Mass-table index for μ: like `elem_index` but including Pluto (row 8).
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
fn mass_index(body: &CelestialBody) -> Option<usize> {
    match body {
        CelestialBody::Pluto => Some(8),
        other => elem_index(other),
    }
}

/// GM for osculating-element formation (SE `Gmsm`, swecl.c:5258/5266):
/// solar μ scaled by `(1 + m_body/m_sun)` where a mass ratio exists, bare
/// solar μ for massless small/fictitious bodies, geocentric μ for the Moon.
#[allow(dead_code)] // consumed by osculating nod_aps (Task 5)
pub(crate) fn mu_au3_day2(body: &CelestialBody) -> f64 {
    if *body == CelestialBody::Moon {
        return GEO_GM_AU3_DAY2;
    }
    match mass_index(body) {
        Some(i) => SUN_GM_AU3_DAY2 * (1.0 + 1.0 / SUN_MASS_RATIO[i]),
        None => SUN_GM_AU3_DAY2,
    }
}

fn poly(c: &[f64; 4], t: f64) -> f64 {
    c[0] + t * (c[1] + t * (c[2] + t * c[3]))
}

/// Mean elements at `jd_tdb`, referred to the mean equinox/ecliptic of date.
pub(crate) fn mean_elements_of_date(index: usize, jd_tdb: f64) -> KeplerianElements {
    let t = (jd_tdb - J2000_JD) / 36_525.0;
    KeplerianElements {
        node_deg: poly(&EL_NODE[index], t).rem_euclid(360.0),
        peri_lon_deg: poly(&EL_PERI[index], t).rem_euclid(360.0),
        incl_deg: poly(&EL_INCL[index], t),
        eccentricity: poly(&EL_ECCE[index], t),
        semi_major_au: poly(&EL_SEMA[index], t),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_elements_are_the_constant_terms() {
        // t = 0 at J2000 ⇒ every polynomial collapses to its constant column.
        let m = mean_elements_of_date(0, 2_451_545.0);
        assert!((m.node_deg - 48.330893).abs() < 1e-12);
        assert!((m.peri_lon_deg - 77.456119).abs() < 1e-12);
        assert!((m.incl_deg - 7.004986).abs() < 1e-12);
        assert!((m.eccentricity - 0.20563175).abs() < 1e-12);
        assert!((m.semi_major_au - 0.387098310).abs() < 1e-12);
    }

    #[test]
    fn one_century_later_mercury_node_advances() {
        // t = 1: 48.330893 + 1.1861890 + 0.00017587 + 0.000000211
        let m = mean_elements_of_date(0, 2_451_545.0 + 36525.0);
        assert!(
            (m.node_deg - 49.517258081).abs() < 1e-8,
            "node {}",
            m.node_deg
        );
    }

    #[test]
    fn mu_uses_se_mass_ratios() {
        use pleiades_types::CelestialBody;
        let gms = SUN_GM_AU3_DAY2;
        let mercury = mu_au3_day2(&CelestialBody::Mercury);
        assert!((mercury / gms - (1.0 + 1.0 / 6_023_600.0)).abs() < 1e-12);
        // Asteroids/fictitious: massless.
        let chiron = mu_au3_day2(&CelestialBody::Custom(pleiades_types::CustomBodyId::new(
            "asteroid",
            "2060-Chiron",
        )));
        assert!((chiron - gms).abs() < 1e-30);
        // Moon: geocentric Earth+Moon μ, far smaller than solar.
        let moon = mu_au3_day2(&CelestialBody::Moon);
        assert!(moon < 1e-8 && moon > 1e-10, "moon mu {moon}");
    }
}
