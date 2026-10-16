//! Tests against independent references: the element polynomial by hand, the
//! propagated state against a textbook Kepler solution (bisection on
//! `E - e sin E = M`, 3-1-3 rotation matrices), and the parsed table against
//! the `seorbel.txt` rows the CSV transcribes.

use super::*;
use crate::test_support::{
    assert_xyz, geo_kepler_third_law, geo_linear_mean_anomaly, geo_quadratic_mean_anomaly,
    helio_kepler_third_law, HALF_CENTURY_DAYS, MINUS_0_3_CENTURY_DAYS,
};

#[test]
fn poly_is_c0_plus_c1_t_plus_c2_t_squared() {
    // t = 2: 1 + 2*2 + 3*2*2 = 17.  t = -1: 1 - 2 + 3 = 2.
    assert_eq!(poly([1.0, 2.0, 3.0], 2.0), 17.0);
    assert_eq!(poly([1.0, 2.0, 3.0], -1.0), 2.0);
    assert_eq!(poly([4.0, 0.0, 0.0], 9.0), 4.0);
}

/// Positions of the hand-built element sets at three instants, from an
/// independent textbook solution (see `test_support`).
#[test]
fn state_at_matches_the_textbook_kepler_solution() {
    let j = crate::J2000_JD;
    let cases = [
        // Kepler-third-law branch, heliocentric: M += 0.9856076686 / a^1.5 * dt.
        (
            "helio third law t=+0.5",
            helio_kepler_third_law(),
            j + HALF_CENTURY_DAYS,
            [5.437398434253267, -3.681490629649334, -0.899407750125753],
        ),
        (
            "helio third law t=-0.3",
            helio_kepler_third_law(),
            j + MINUS_0_3_CENTURY_DAYS,
            [2.7891781647294955, -5.4997465160827055, -0.3119204514759507],
        ),
        (
            "helio third law dt=1000 d",
            helio_kepler_third_law(),
            j + 1000.0,
            [1.866723361138678, -5.690277707544131, -0.14924626112035555],
        ),
        // Same orbit, Earth-centred: the daily motion is divided by sqrt(332946.050895).
        (
            "geo third law t=+0.5",
            geo_kepler_third_law(),
            j + HALF_CENTURY_DAYS,
            [-3.367097275068979, -1.9260122409620681, 0.7253771374808393],
        ),
        (
            "geo third law t=-0.3",
            geo_kepler_third_law(),
            j + MINUS_0_3_CENTURY_DAYS,
            [-3.6599017449088507, -0.9412743279780411, 0.6260396615603739],
        ),
        (
            "geo third law dt=1000 d",
            geo_kepler_third_law(),
            j + 1000.0,
            [-3.587810029275123, -1.3121634325323666, 0.6670968458062394],
        ),
        // Linear mean-anomaly T-term (c1 != 0, c2 == 0): no third-law motion on top.
        (
            "linear M t=+0.5",
            geo_linear_mean_anomaly(),
            j + HALF_CENTURY_DAYS,
            [
                -1.2768405686458082,
                -0.9027280114438142,
                0.058470069661990066,
            ],
        ),
        (
            "linear M t=-0.3",
            geo_linear_mean_anomaly(),
            j + MINUS_0_3_CENTURY_DAYS,
            [1.3498906655118699, -0.8574832792105451, -0.5440321659391396],
        ),
        (
            "linear M dt=1000 d",
            geo_linear_mean_anomaly(),
            j + 1000.0,
            [-0.6424954585516662, -1.7052379996793825, -0.325064647765959],
        ),
        // Quadratic-only mean anomaly (c1 == 0, c2 != 0): still a T-term.
        (
            "quadratic M t=+0.5",
            geo_quadratic_mean_anomaly(),
            j + HALF_CENTURY_DAYS,
            [-0.8792852339310652, 0.5955128798993754, 0.3837187036427429],
        ),
        (
            "quadratic M t=-0.3",
            geo_quadratic_mean_anomaly(),
            j + MINUS_0_3_CENTURY_DAYS,
            [-0.5905830068700167, 0.8183869843286865, 0.3606499166466236],
        ),
        (
            "quadratic M dt=1000 d",
            geo_quadratic_mean_anomaly(),
            j + 1000.0,
            [-0.48376876554988213, 0.8778553390623682, 0.3584063531532478],
        ),
    ];
    for (what, el, jd, want) in cases {
        assert_xyz(el.state_at(jd), want, 1.0e-9, what);
    }
}

#[test]
fn the_state_at_the_epoch_is_the_mean_anomaly_zero_point_of_a_circular_orbit() {
    // At the epoch t = 0 the third-law term vanishes: a circular, uninclined
    // orbit with M0 = 0 sits at (a, 0, 0).
    let el = crate::test_support::circular_helio_at_epoch(3.0);
    assert_xyz(
        el.state_at(crate::J2000_JD),
        [3.0, 0.0, 0.0],
        1.0e-12,
        "epoch",
    );
}

#[test]
fn jd_tokens_resolve_to_the_seorbel_keywords_and_bare_julian_days() {
    assert_eq!(jd_from_token("J1900"), 2_415_020.0);
    assert_eq!(jd_from_token("B1950"), 2_433_282.423_459_05);
    assert_eq!(jd_from_token("J2000"), 2_451_545.0);
    assert_eq!(jd_from_token("  J2000 "), 2_451_545.0);
    assert_eq!(jd_from_token("2368547.66"), 2_368_547.66);
    assert_eq!(jd_from_token(" 1856113.380954 "), 1_856_113.380_954);
}

#[test]
fn equinox_tokens_distinguish_jdate_from_fixed_equinoxes() {
    assert_eq!(equinox_from_token("JDATE"), Equinox::OfDate);
    assert_eq!(equinox_from_token(" JDATE "), Equinox::OfDate);
    assert_eq!(equinox_from_token("J1900"), Equinox::Fixed(2_415_020.0));
    assert_eq!(equinox_from_token("2431456.5"), Equinox::Fixed(2_431_456.5));
}

#[test]
fn the_table_carries_the_transcribed_seorbel_rows() {
    // Rows below are from tools/se-fictitious-reference/data/seorbel.txt
    // (lines 44, 46, 48, 55, 57 quoted per body). The CSV's own comment quotes
    // seorbel.txt line 28:
    //   "J1900, J1900, 163.7409, 40.99837, 0.00460, 171.4333, 129.8325, 1.0833, Cupido"
    // i.e. epoch, equinox, M0, a, e, argp, node, incl.
    let cupido = elements_for(CelestialBody::Cupido).unwrap();
    assert_eq!(cupido.epoch_jd_tt, 2_415_020.0);
    assert_eq!(cupido.equinox, Equinox::Fixed(2_415_020.0));
    assert_eq!(cupido.a_au, [40.99837, 0.0, 0.0]);
    assert_eq!(cupido.e, [0.00460, 0.0, 0.0]);
    assert_eq!(cupido.incl_deg, [1.0833, 0.0, 0.0]);
    assert_eq!(cupido.node_deg, [129.8325, 0.0, 0.0]);
    assert_eq!(cupido.arg_peri_deg, [171.4333, 0.0, 0.0]);
    assert_eq!(cupido.mean_anom_deg, [163.7409, 0.0, 0.0]);
    assert_eq!(cupido.center, Center::Heliocentric);

    // Transpluto (seorbel set 9): arbitrary-JD epoch and equinox.
    // tools/se-fictitious-reference/data/seorbel.txt line 44:
    //   "2368547.66, 2431456.5, 0.0, 77.775, 0.3, 0.7, 0, 0, Isis-Transpluto"
    let transpluto = elements_for(CelestialBody::Transpluto).unwrap();
    assert_eq!(transpluto.epoch_jd_tt, 2_368_547.66);
    assert_eq!(transpluto.equinox, Equinox::Fixed(2_431_456.5));
    assert_eq!(transpluto.a_au[0], 77.775);
    assert_eq!(transpluto.e[0], 0.3);
    assert_eq!(transpluto.arg_peri_deg[0], 0.7);

    // Nibiru (set 10): negative node, epoch == equinox.
    // seorbel.txt line 46: "1856113.380954, 1856113.380954, 0.0, 234.8921,
    //   0.981092, 103.966, -44.567, 158.708, Nibiru"
    let nibiru = elements_for(CelestialBody::Nibiru).unwrap();
    assert_eq!(nibiru.epoch_jd_tt, 1_856_113.380_954);
    assert_eq!(nibiru.equinox, Equinox::Fixed(1_856_113.380_954));
    assert_eq!(nibiru.node_deg[0], -44.567);
    assert_eq!(nibiru.e[0], 0.981092);

    // Harrington (set 11): numeric epoch, keyword equinox.
    // seorbel.txt line 48: "2374696.5, J2000, 0.0, 101.2, 0.411, 208.5,
    //   275.4, 32.4, Harrington"
    let harrington = elements_for(CelestialBody::Harrington).unwrap();
    assert_eq!(harrington.epoch_jd_tt, 2_374_696.5);
    assert_eq!(harrington.equinox, Equinox::Fixed(2_451_545.0));

    // Vulcan (set 16): JDATE equinox with T-terms in node, argp and M.
    // seorbel.txt line 55: "J1900,JDATE, 252.8987988 + 707550.7341 * T,
    //   0.13744, 0.019, 322.212069+1670.056*T, 47.787931-1670.056*T, 7.5, Vulcan"
    let vulcan = elements_for(CelestialBody::Vulcan).unwrap();
    assert_eq!(vulcan.equinox, Equinox::OfDate);
    assert_eq!(vulcan.node_deg, [47.787931, -1670.056, 0.0]);
    assert_eq!(vulcan.arg_peri_deg, [322.212069, 1670.056, 0.0]);
    assert_eq!(vulcan.mean_anom_deg, [252.8987988, 707550.7341, 0.0]);

    // White Moon (set 17): Earth-centred, epoch J2000.
    // seorbel.txt line 57: "J2000,JDATE, 242.2205555 + 5143.5418158 * T,
    //   0.05280098949, 0.0, 0.0, 0.0, 0.0, Selena/White Moon, geo"
    let white_moon = elements_for(CelestialBody::WhiteMoon).unwrap();
    assert_eq!(white_moon.center, Center::Geocentric);
    assert_eq!(white_moon.epoch_jd_tt, 2_451_545.0);
    assert_eq!(white_moon.equinox, Equinox::OfDate);
    assert_eq!(white_moon.mean_anom_deg, [242.2205555, 5143.5418158, 0.0]);
}
