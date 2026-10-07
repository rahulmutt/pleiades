use super::*;
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

#[test]
fn exactly_the_swiss_ephemeris_star_modes_have_an_anchor() {
    use AnchorProjection::*;
    use AnchorStar::*;
    let expected = [
        (Ayanamsa::TrueCitra, Spica, EclipticLongitude),
        (Ayanamsa::TrueChitra, Spica, EclipticLongitude),
        (Ayanamsa::TrueRevati, ZetaPiscium, EclipticLongitude),
        (Ayanamsa::TruePushya, DeltaCancri, EclipticLongitude),
        (Ayanamsa::TrueSheoran, DeltaCancri, EclipticLongitude),
        (Ayanamsa::TrueMula, LambdaScorpii, EclipticLongitude),
        (Ayanamsa::GalacticCenter, GalacticCenter, EclipticLongitude),
        (
            Ayanamsa::GalacticCenterRgilbrand,
            GalacticCenter,
            EclipticLongitude,
        ),
        (
            Ayanamsa::GalacticCenterCochrane,
            GalacticCenter,
            EclipticLongitude,
        ),
        (
            Ayanamsa::GalacticCenterMulaWilhelm,
            GalacticCenter,
            PolarRightAscension,
        ),
    ];
    for (ayanamsa, star, projection) in &expected {
        assert_eq!(
            star_anchor(ayanamsa),
            Some(StarAnchor {
                star: *star,
                projection: *projection
            }),
            "{ayanamsa:?}"
        );
    }
    // Swiss Ephemeris applies no aberration to these (measured 0 over 1900–2100).
    for ayanamsa in [
        Ayanamsa::GalacticEquatorIau1958,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticEquatorMula,
        Ayanamsa::GalacticEquatorFiorenza,
        Ayanamsa::GalacticCenterMardyks,
        Ayanamsa::Lahiri,
        Ayanamsa::FaganBradley,
        Ayanamsa::DeLuce,
    ] {
        assert_eq!(star_anchor(&ayanamsa), None, "{ayanamsa:?}");
    }
    let anchored = crate::built_in_ayanamsas()
        .iter()
        .filter(|descriptor| star_anchor(&descriptor.ayanamsa).is_some())
        .count();
    assert_eq!(anchored, expected.len());
}

// Swiss Ephemeris `swe_fixstar`, geometric mean ecliptic and equinox of date
// (MOSEPH|NONUT|TRUEPOS|NOABERR|NOGDEFL), measured 2026-10-07: the latitude
// at J2000 and the identity longitude = primary mode's ayanamsa + anchor.
#[test]
fn mean_places_match_swiss_ephemeris_at_j2000() {
    let cases = [
        (
            AnchorStar::Spica,
            Ayanamsa::TrueCitra,
            180.0,
            -2.054_487_222,
        ),
        (
            AnchorStar::ZetaPiscium,
            Ayanamsa::TrueRevati,
            359.833_333_333_3,
            -0.213_433_452,
        ),
        (
            AnchorStar::DeltaCancri,
            Ayanamsa::TruePushya,
            106.0,
            0.077_172_355,
        ),
        (
            AnchorStar::LambdaScorpii,
            Ayanamsa::TrueMula,
            240.0,
            -13.788_463_334,
        ),
        (
            AnchorStar::GalacticCenter,
            Ayanamsa::GalacticCenter,
            240.0,
            -5.607_686_222,
        ),
    ];
    let j2000 = tt(2_451_545.0);
    for (star, primary, anchor, beta) in cases {
        let place = anchor_star_mean_place(star, j2000).expect("place");
        let ayanamsa = crate::sidereal_offset(&primary, j2000).unwrap().degrees();
        let lon = (ayanamsa + anchor).rem_euclid(360.0);
        assert!((place.longitude_deg - lon).abs() < 1e-12, "{star:?}");
        // The linear fit is within 0.12″ of Swiss Ephemeris everywhere.
        assert!(
            ((place.latitude_deg - beta) * 3600.0).abs() < 0.11,
            "{star:?}"
        );
    }
}

// Issue #226: the latitude's rate term, off J2000. Swiss Ephemeris
// `swe_fixstar` (same flags as above) at 1900 and 2100, printed by
// `tools/se-ayanamsa-reference anchor-places`, measured 2026-10-07.
#[test]
fn mean_place_latitudes_match_swiss_ephemeris_off_j2000() {
    let cases = [
        // (star, β at JD 2415020.5, β at JD 2488069.5), degrees
        (AnchorStar::Spica, -2.046_980_611, -2.062_080_618),
        (AnchorStar::ZetaPiscium, -0.215_952_235, -0.210_821_425),
        (AnchorStar::DeltaCancri, 0.073_987_339, 0.080_268_140),
        (AnchorStar::LambdaScorpii, -13.774_533_942, -13.802_374_841),
        (AnchorStar::GalacticCenter, -5.594_472_070, -5.620_878_154),
    ];
    for (star, beta_1900, beta_2100) in cases {
        for (jd, beta) in [(2_415_020.5, beta_1900), (2_488_069.5, beta_2100)] {
            let place = anchor_star_mean_place(star, tt(jd)).expect("place");
            assert!(
                ((place.latitude_deg - beta) * 3600.0).abs() < 0.12,
                "{star:?} {jd}: {} vs {beta}",
                place.latitude_deg
            );
        }
    }
}

#[test]
fn longitudes_are_normalized() {
    // ζ Psc sits at 359.83° + ayanamsa, past 360° for the whole window.
    for jd in [2_415_020.5, 2_451_545.0, 2_488_069.5] {
        for star in [
            AnchorStar::Spica,
            AnchorStar::ZetaPiscium,
            AnchorStar::DeltaCancri,
            AnchorStar::LambdaScorpii,
            AnchorStar::GalacticCenter,
        ] {
            let lon = anchor_star_mean_place(star, tt(jd)).unwrap().longitude_deg;
            assert!((0.0..360.0).contains(&lon), "{star:?} {jd} {lon}");
        }
    }
}
