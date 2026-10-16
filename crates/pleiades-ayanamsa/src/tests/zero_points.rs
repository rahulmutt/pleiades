//! Published zero points: the Swiss Ephemeris `aya_init` anchor table and the
//! catalog descriptors that carry the same epoch/offset pairs.
//!
//! Every expected value is a literal worked out by hand from the cited source
//! (`sweph.h` `ayanamsa[]`, SE 2.10.03), for example `360 - 341.33904 = 18.66096`.

use pleiades_types::Ayanamsa;

use crate::{
    built_in_ayanamsas, release_ayanamsas, se_anchor, AyanamsaDescriptor, IN_SCOPE_ANCHORS,
};

/// Absolute tolerance for decimal literals: far below the smallest operator
/// mutant (a `-` turned into `+` moves a value by at least 2e-3 degrees).
const TOL: f64 = 1.0e-9;

/// `(typed mode, SE_SIDM, t0 JD, ayan_t0 degrees, t0_is_UT)` per `sweph.h`
/// `ayanamsa[SE_SIDM]`. `J1900 = 2415020.0`, `J2000 = 2451545.0` and
/// `B1950 = 2433282.42345905` are the `#define`s at the top of `sweph.h`.
const SWEPH_ROWS: &[(Ayanamsa, i32, f64, f64, bool)] = &[
    // 2: {1721057.5, 0, TRUE, 0}
    (Ayanamsa::DeLuce, 2, 1721057.5, 0.0, true),
    // 4: {J1900, 360 - 341.33904, FALSE, -1}; 360 - 341.33904 = 18.66096
    (Ayanamsa::UshaShashi, 4, 2415020.0, 18.66096, false),
    // 6: {J1900, 360 - 333.0369024, FALSE, 0}; = 26.9630976
    (Ayanamsa::DjwhalKhul, 6, 2415020.0, 26.9630976, false),
    // 7: {J1900, 360 - 338.917778, FALSE, -1}; = 21.082222
    (Ayanamsa::Yukteshwar, 7, 2415020.0, 21.082222, false),
    // 8: {J1900, 360 - 338.634444, FALSE, -1}; = 21.365556
    (Ayanamsa::JnBhasin, 8, 2415020.0, 21.365556, false),
    // 9..12: {1684532.5, -5.66667 | -4.26667 | -3.41667 | -4.46667, TRUE, -1}
    (Ayanamsa::BabylonianKugler1, 9, 1684532.5, -5.66667, true),
    (Ayanamsa::BabylonianKugler2, 10, 1684532.5, -4.26667, true),
    (Ayanamsa::BabylonianKugler3, 11, 1684532.5, -3.41667, true),
    (Ayanamsa::BabylonianHuber, 12, 1684532.5, -4.46667, true),
    // 13: {1673941, -5.079167, TRUE, -1}
    (
        Ayanamsa::BabylonianEtaPiscium,
        13,
        1673941.0,
        -5.079167,
        true,
    ),
    // 14: {1684532.5, -4.44138598, TRUE, 0}
    (
        Ayanamsa::BabylonianAldebaran,
        14,
        1684532.5,
        -4.44138598,
        true,
    ),
    // 15: {1674484.0, -9.33333, TRUE, -1}
    (Ayanamsa::Hipparchus, 15, 1674484.0, -9.33333, true),
    // 16: {1927135.8747793, 0, TRUE, -1}
    (Ayanamsa::Sassanian, 16, 1927135.8747793, 0.0, true),
    // 18, 19, 20: {J2000, 0}, {J1900, 0}, {B1950, 0}
    (Ayanamsa::J2000, 18, 2451545.0, 0.0, false),
    (Ayanamsa::J1900, 19, 2415020.0, 0.0, false),
    (Ayanamsa::B1950, 20, 2433282.42345905, 0.0, false),
    // 21..26: Suryasiddhanta and Aryabhata families
    (Ayanamsa::Suryasiddhanta499, 21, 1903396.8128654, 0.0, true),
    (
        Ayanamsa::Suryasiddhanta499MeanSun,
        22,
        1903396.8128654,
        -0.21463395,
        true,
    ),
    (Ayanamsa::Aryabhata499, 23, 1903396.7895321, 0.0, true),
    (
        Ayanamsa::Aryabhata499MeanSun,
        24,
        1903396.7895321,
        -0.23763238,
        true,
    ),
    (
        Ayanamsa::SuryasiddhantaRevati,
        25,
        1903396.8128654,
        -0.79167046,
        true,
    ),
    (
        Ayanamsa::SuryasiddhantaCitra,
        26,
        1903396.8128654,
        2.11070444,
        true,
    ),
    // 37 (commented "36" in sweph.h): {1911797.740782065, 0, TRUE, 0}
    (Ayanamsa::Aryabhata522, 37, 1911797.740782065, 0.0, true),
    // 38: {1721057.5, -3.2, TRUE, -1}
    (Ayanamsa::BabylonianBritton, 38, 1721057.5, -3.2, true),
    // 42: {1775845.5, -2.9422, TRUE, -1}
    (Ayanamsa::ValensMoon, 42, 1775845.5, -2.9422, true),
    // 43: {J1900, 22.44597222, FALSE, SEMOD_PREC_NEWCOMB}
    (Ayanamsa::Lahiri1940, 43, 2415020.0, 22.44597222, false),
    // 44, 45: zero-ayanamsa epochs
    (Ayanamsa::LahiriVP285, 44, 1825235.2458513028, 0.0, false),
    (
        Ayanamsa::KrishnamurtiVP291,
        45,
        1827424.752255678,
        0.0,
        false,
    ),
    // 46: {2435553.5, 23.25 - 0.00464207, FALSE, SEMOD_PREC_NEWCOMB}; = 23.24535793
    (Ayanamsa::LahiriIcrc, 46, 2435553.5, 23.24535793, false),
];

#[test]
fn se_anchor_table_matches_sweph_h_rows() {
    assert_eq!(
        IN_SCOPE_ANCHORS.len(),
        SWEPH_ROWS.len(),
        "the anchor table and the sweph.h transcription must list the same rows"
    );
    for (mode, sidm, t0, ayan_t0, is_ut) in SWEPH_ROWS {
        let anchor = se_anchor(mode).unwrap_or_else(|| panic!("{mode:?} has no SE anchor"));
        assert_eq!(anchor.se_sidm, *sidm, "{mode:?} SE_SIDM");
        assert!((anchor.t0 - t0).abs() < TOL, "{mode:?} t0 {}", anchor.t0);
        assert!(
            (anchor.ayan_t0 - ayan_t0).abs() < TOL,
            "{mode:?} ayan_t0 {} vs {ayan_t0}",
            anchor.ayan_t0
        );
        assert_eq!(anchor.t0_is_ut, *is_ut, "{mode:?} t0_is_UT");
    }
}

/// Catalog descriptors whose zero point is published in `sweph.h` with an
/// arithmetic expression (so a sign or operator slip is a real error), plus the
/// negative-offset entries. `(mode, epoch JD, offset degrees, tolerance)`.
///
/// Hipparchus and Babylonian (Huber) are not equal to their `sweph.h` anchors:
/// the catalog keeps its own historical value for each (see
/// `pinned_descriptors_agree_with_se_anchors_except_documented_divergences`).
const DESCRIPTOR_ZERO_POINTS: &[(Ayanamsa, f64, f64, f64)] = &[
    (Ayanamsa::LahiriIcrc, 2435553.5, 23.24535793, TOL),
    (Ayanamsa::UshaShashi, 2415020.0, 18.66096, TOL),
    (Ayanamsa::Yukteshwar, 2415020.0, 21.082222, TOL),
    (Ayanamsa::DjwhalKhul, 2415020.0, 26.9630976, TOL),
    (Ayanamsa::JnBhasin, 2415020.0, 21.365556, TOL),
    (
        Ayanamsa::SuryasiddhantaRevati,
        1903396.8128654,
        -0.79167046,
        TOL,
    ),
    (
        Ayanamsa::Suryasiddhanta499MeanSun,
        1903396.8128654,
        -0.21463395,
        TOL,
    ),
    (
        Ayanamsa::Aryabhata499MeanSun,
        1903396.7895321,
        -0.23763238,
        TOL,
    ),
    (Ayanamsa::ValensMoon, 1775845.5, -2.9422, TOL),
    // -28/3 degrees: the catalog keeps the exact third, where `sweph.h` rounds
    // to -9.33333.
    (Ayanamsa::Hipparchus, 1674484.0, -9.333333333333334, 1.0e-12),
    // REGRESSION PIN, NO INDEPENDENT SOURCE. Neither the catalog notes nor any
    // spec or git history (`git log -S`) cite a number for this zero point, and
    // `sweph.h` row 12 is {1684532.5, -4.46667}. The literal is the descriptor's
    // own value (-434/3600 degrees, i.e. -0 deg 07' 14") written out, so this row
    // only guards against sign and operator slips; it does not vouch for the value.
    (
        Ayanamsa::BabylonianHuber,
        1721171.5,
        -0.12055555555555555,
        1.0e-12,
    ),
];

fn find<'a>(list: &'a [AyanamsaDescriptor], mode: &Ayanamsa) -> Option<&'a AyanamsaDescriptor> {
    list.iter().find(|d| d.ayanamsa == *mode)
}

#[test]
fn descriptor_zero_points_match_published_values_in_both_catalog_tables() {
    // The release table and the full built-in table carry separate copies of
    // each descriptor, so both are checked.
    for (name, list) in [
        ("release", release_ayanamsas()),
        ("built-in", built_in_ayanamsas()),
    ] {
        for (mode, epoch, offset, tol) in DESCRIPTOR_ZERO_POINTS {
            let d = find(list, mode).unwrap_or_else(|| panic!("{mode:?} missing from {name}"));
            let got_epoch = d.epoch.expect("epoch").days();
            let got_offset = d.offset_degrees.expect("offset").degrees();
            assert!(
                (got_epoch - epoch).abs() < TOL,
                "{name} {mode:?} epoch {got_epoch} vs {epoch}"
            );
            assert!(
                (got_offset - offset).abs() < *tol,
                "{name} {mode:?} offset {got_offset} vs {offset}"
            );
        }
    }
}

/// Anchors whose catalog descriptor deliberately keeps a different zero point.
const DIVERGENT_FROM_SWEPH: &[Ayanamsa] = &[Ayanamsa::BabylonianHuber, Ayanamsa::Hipparchus];

/// For the zero points pinned above, the descriptor must also equal the
/// `se_anchors` entry, so the two tables cannot drift apart. Other descriptors
/// (for example DeLuce, whose catalog epoch is J2000) are outside this check:
/// the catalog does not claim to mirror every `aya_init` row.
#[test]
fn pinned_descriptors_agree_with_se_anchors_except_documented_divergences() {
    let mut compared = 0;
    for (mode, ..) in DESCRIPTOR_ZERO_POINTS {
        if DIVERGENT_FROM_SWEPH.contains(mode) {
            continue;
        }
        let anchor = se_anchor(mode).unwrap_or_else(|| panic!("{mode:?} has no SE anchor"));
        for list in [release_ayanamsas(), built_in_ayanamsas()] {
            let d = find(list, mode).unwrap_or_else(|| panic!("{mode:?} missing"));
            compared += 1;
            let epoch = d.epoch.expect("epoch").days();
            let offset = d.offset_degrees.expect("offset").degrees();
            assert!(
                (epoch - anchor.t0).abs() < TOL,
                "{mode:?} epoch {epoch} vs anchor {}",
                anchor.t0
            );
            assert!(
                (offset - anchor.ayan_t0).abs() < TOL,
                "{mode:?} offset {offset} vs anchor {}",
                anchor.ayan_t0
            );
        }
    }
    assert_eq!(
        compared,
        2 * (DESCRIPTOR_ZERO_POINTS.len() - DIVERGENT_FROM_SWEPH.len())
    );
}
