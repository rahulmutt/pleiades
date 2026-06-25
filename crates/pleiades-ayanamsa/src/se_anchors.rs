//! SE-sourced ayanamsa anchors, transcribed from Swiss Ephemeris 2.10.03
//! `sweph.h` `static const struct aya_init ayanamsa[SE_NSIDM_PREDEF]`.
//! Each row: { SE_SIDM index, t0 (JD), ayan_t0 (deg at t0), t0_is_UT }.
//! Provenance: libswisseph-sys 0.1.2 vendored SE 2.10.03. SE_SIDM indices
//! per `swephexp.h`. Do not edit values without re-checking the SE source row
//! named in the trailing comment.

use pleiades_types::Ayanamsa;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeAnchor {
    pub se_sidm: i32,
    pub t0: f64,
    pub ayan_t0: f64,
    pub t0_is_ut: bool,
}

pub const IN_SCOPE_ANCHORS: &[(Ayanamsa, SeAnchor)] = &[
    // sweph.h ayanamsa[2]: {1721057.5, 0, TRUE, 0} — DeLuce
    (
        Ayanamsa::DeLuce,
        SeAnchor {
            se_sidm: 2,
            t0: 1721057.5,
            ayan_t0: 0.0,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[4]: {J1900, 360 - 341.33904, FALSE, -1} — Usha/Shashi
    (
        Ayanamsa::UshaShashi,
        SeAnchor {
            se_sidm: 4,
            t0: 2415020.0,
            ayan_t0: 360.0 - 341.33904,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[6]: {J1900, 360 - 333.0369024, FALSE, 0} — Djwhal Khool
    (
        Ayanamsa::DjwhalKhul,
        SeAnchor {
            se_sidm: 6,
            t0: 2415020.0,
            ayan_t0: 360.0 - 333.0369024,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[7]: {J1900, 360 - 338.917778, FALSE, -1} — Shri Yukteshwar
    (
        Ayanamsa::Yukteshwar,
        SeAnchor {
            se_sidm: 7,
            t0: 2415020.0,
            ayan_t0: 360.0 - 338.917778,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[8]: {J1900, 360 - 338.634444, FALSE, -1} — Bhasin
    (
        Ayanamsa::JnBhasin,
        SeAnchor {
            se_sidm: 8,
            t0: 2415020.0,
            ayan_t0: 360.0 - 338.634444,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[9]: {1684532.5, -5.66667, TRUE, -1} — Babylonian, Kugler 1
    (
        Ayanamsa::BabylonianKugler1,
        SeAnchor {
            se_sidm: 9,
            t0: 1684532.5,
            ayan_t0: -5.66667,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[10]: {1684532.5, -4.26667, TRUE, -1} — Babylonian, Kugler 2
    (
        Ayanamsa::BabylonianKugler2,
        SeAnchor {
            se_sidm: 10,
            t0: 1684532.5,
            ayan_t0: -4.26667,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[11]: {1684532.5, -3.41667, TRUE, -1} — Babylonian, Kugler 3
    (
        Ayanamsa::BabylonianKugler3,
        SeAnchor {
            se_sidm: 11,
            t0: 1684532.5,
            ayan_t0: -3.41667,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[12]: {1684532.5, -4.46667, TRUE, -1} — Babylonian, Huber
    (
        Ayanamsa::BabylonianHuber,
        SeAnchor {
            se_sidm: 12,
            t0: 1684532.5,
            ayan_t0: -4.46667,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[13]: {1673941, -5.079167, TRUE, -1} — Babylonian, Mercier/EtaPiscium
    (
        Ayanamsa::BabylonianEtaPiscium,
        SeAnchor {
            se_sidm: 13,
            t0: 1673941.0,
            ayan_t0: -5.079167,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[14]: {1684532.5, -4.44138598, TRUE, 0} — Babylonian/Aldebaran = 15 Tau
    (
        Ayanamsa::BabylonianAldebaran,
        SeAnchor {
            se_sidm: 14,
            t0: 1684532.5,
            ayan_t0: -4.44138598,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[15]: {1674484.0, -9.33333, TRUE, -1} — Hipparchos
    (
        Ayanamsa::Hipparchus,
        SeAnchor {
            se_sidm: 15,
            t0: 1674484.0,
            ayan_t0: -9.33333,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[16]: {1927135.8747793, 0, TRUE, -1} — Sassanian
    (
        Ayanamsa::Sassanian,
        SeAnchor {
            se_sidm: 16,
            t0: 1927135.8747793,
            ayan_t0: 0.0,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[18]: {J2000, 0, FALSE, 0} — J2000
    (
        Ayanamsa::J2000,
        SeAnchor {
            se_sidm: 18,
            t0: 2451545.0,
            ayan_t0: 0.0,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[19]: {J1900, 0, FALSE, 0} — J1900
    (
        Ayanamsa::J1900,
        SeAnchor {
            se_sidm: 19,
            t0: 2415020.0,
            ayan_t0: 0.0,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[20]: {B1950, 0, FALSE, 0} — B1950
    (
        Ayanamsa::B1950,
        SeAnchor {
            se_sidm: 20,
            t0: 2433282.42345905,
            ayan_t0: 0.0,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[21]: {1903396.8128654, 0, TRUE, 0} — Suryasiddhanta
    (
        Ayanamsa::Suryasiddhanta499,
        SeAnchor {
            se_sidm: 21,
            t0: 1903396.8128654,
            ayan_t0: 0.0,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[22]: {1903396.8128654, -0.21463395, TRUE, 0} — Suryasiddhanta, mean Sun
    (
        Ayanamsa::Suryasiddhanta499MeanSun,
        SeAnchor {
            se_sidm: 22,
            t0: 1903396.8128654,
            ayan_t0: -0.21463395,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[23]: {1903396.7895321, 0, TRUE, 0} — Aryabhata
    (
        Ayanamsa::Aryabhata499,
        SeAnchor {
            se_sidm: 23,
            t0: 1903396.7895321,
            ayan_t0: 0.0,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[24]: {1903396.7895321, -0.23763238, TRUE, 0} — Aryabhata, mean Sun
    (
        Ayanamsa::Aryabhata499MeanSun,
        SeAnchor {
            se_sidm: 24,
            t0: 1903396.7895321,
            ayan_t0: -0.23763238,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[25]: {1903396.8128654, -0.79167046, TRUE, 0} — SS Revati
    (
        Ayanamsa::SuryasiddhantaRevati,
        SeAnchor {
            se_sidm: 25,
            t0: 1903396.8128654,
            ayan_t0: -0.79167046,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[26]: {1903396.8128654, 2.11070444, TRUE, 0} — SS Citra
    (
        Ayanamsa::SuryasiddhantaCitra,
        SeAnchor {
            se_sidm: 26,
            t0: 1903396.8128654,
            ayan_t0: 2.11070444,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[37]: {1911797.740782065, 0, TRUE, 0} — 0 ayanamsha in year 522
    (
        Ayanamsa::Aryabhata522,
        SeAnchor {
            se_sidm: 37,
            t0: 1911797.740782065,
            ayan_t0: 0.0,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[38]: {1721057.5, -3.2, TRUE, -1} — Babylonian (Britton 2010)
    (
        Ayanamsa::BabylonianBritton,
        SeAnchor {
            se_sidm: 38,
            t0: 1721057.5,
            ayan_t0: -3.2,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[42]: {1775845.5, -2.9422, TRUE, -1} — Vettius Valens
    (
        Ayanamsa::ValensMoon,
        SeAnchor {
            se_sidm: 42,
            t0: 1775845.5,
            ayan_t0: -2.9422,
            t0_is_ut: true,
        },
    ),
    // sweph.h ayanamsa[43]: {J1900, 22.44597222, FALSE, SEMOD_PREC_NEWCOMB} — Lahiri (1940)
    (
        Ayanamsa::Lahiri1940,
        SeAnchor {
            se_sidm: 43,
            t0: 2415020.0,
            ayan_t0: 22.44597222,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[44]: {1825235.2458513028, 0.0, FALSE, 0} — Lahiri VP285 (1980)
    (
        Ayanamsa::LahiriVP285,
        SeAnchor {
            se_sidm: 44,
            t0: 1825235.2458513028,
            ayan_t0: 0.0,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[45]: {1827424.752255678, 0.0, FALSE, 0} — Krishnamurti VP291
    (
        Ayanamsa::KrishnamurtiVP291,
        SeAnchor {
            se_sidm: 45,
            t0: 1827424.752255678,
            ayan_t0: 0.0,
            t0_is_ut: false,
        },
    ),
    // sweph.h ayanamsa[46]: {2435553.5, 23.25 - 0.00464207, FALSE, SEMOD_PREC_NEWCOMB} — SE_SIDM_LAHIRI_ICRC
    (
        Ayanamsa::LahiriIcrc,
        SeAnchor {
            se_sidm: 46,
            t0: 2435553.5,
            ayan_t0: 23.25 - 0.00464207,
            t0_is_ut: false,
        },
    ),
];

pub fn se_anchor(a: &Ayanamsa) -> Option<SeAnchor> {
    IN_SCOPE_ANCHORS
        .iter()
        .find(|(m, _)| m == a)
        .map(|(_, s)| *s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::Ayanamsa;

    #[test]
    fn j2000_anchor_matches_se_table() {
        // sweph.h ayanamsa[18] = J2000: {J2000, 0, FALSE, ...}
        let a = se_anchor(&Ayanamsa::J2000).expect("J2000 has an SE anchor");
        assert_eq!(a.se_sidm, 18);
        assert!((a.t0 - 2_451_545.0).abs() < 1e-6);
        assert!((a.ayan_t0 - 0.0).abs() < 1e-9);
        assert!(!a.t0_is_ut);
    }

    #[test]
    fn every_anchor_is_finite_and_unique_sidm() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for a in IN_SCOPE_ANCHORS {
            assert!(a.1.t0.is_finite() && a.1.ayan_t0.is_finite());
            assert!(
                seen.insert(a.1.se_sidm),
                "duplicate SE_SIDM {}",
                a.1.se_sidm
            );
        }
    }
}
