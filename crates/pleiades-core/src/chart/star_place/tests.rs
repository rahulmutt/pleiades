use super::*;
use pleiades_types::{Ayanamsa, Instant, JulianDay, TimeScale};

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn arcsec(ayanamsa: Ayanamsa, jd: f64) -> f64 {
    apparent_star_ayanamsa_correction(&ayanamsa, tt(jd))
        .expect("anchored")
        .degrees()
        * 3600.0
}

// Swiss Ephemeris apparent − geometric ayanamsa (MOSEPH|NONUT), measured
// 2026-10-07 with libswisseph-sys 0.1.2. Away from solar conjunction the
// model holds these to 0.06″ (spec amendment 11).
#[test]
fn matches_swiss_ephemeris_reference_values() {
    let cases = [
        (Ayanamsa::TrueCitra, 2_451_545.0, -4.8521),
        (Ayanamsa::TrueCitra, 2_460_000.5, 13.6521),
        (Ayanamsa::TrueRevati, 2_451_545.0, 3.4348),
        (Ayanamsa::TrueRevati, 2_460_000.5, -14.6800),
        (Ayanamsa::TruePushya, 2_451_545.0, 18.3457),
        (Ayanamsa::TrueMula, 2_451_545.0, -20.6651),
        (Ayanamsa::TrueMula, 2_460_000.5, -7.1331),
        (Ayanamsa::GalacticCenter, 2_451_545.0, -20.3888),
        (Ayanamsa::GalacticCenter, 2_460_000.5, -7.7311),
        (Ayanamsa::GalacticCenterMulaWilhelm, 2_451_545.0, -21.2815),
        (Ayanamsa::GalacticCenterMulaWilhelm, 2_460_000.5, -8.0303),
    ];
    for (ayanamsa, jd, expected) in cases {
        let got = arcsec(ayanamsa.clone(), jd);
        assert!(
            (got - expected).abs() < 0.06,
            "{ayanamsa:?} {jd}: {got} vs {expected}"
        );
    }
}

#[test]
fn modes_sharing_a_star_and_projection_share_the_correction() {
    for jd in [2_420_000.5, 2_451_545.0, 2_480_000.5] {
        assert_eq!(
            arcsec(Ayanamsa::TrueChitra, jd),
            arcsec(Ayanamsa::TrueCitra, jd)
        );
        assert_eq!(
            arcsec(Ayanamsa::TrueSheoran, jd),
            arcsec(Ayanamsa::TruePushya, jd)
        );
        assert_eq!(
            arcsec(Ayanamsa::GalacticCenterCochrane, jd),
            arcsec(Ayanamsa::GalacticCenter, jd)
        );
        assert_eq!(
            arcsec(Ayanamsa::GalacticCenterRgilbrand, jd),
            arcsec(Ayanamsa::GalacticCenter, jd)
        );
    }
}

#[test]
fn unanchored_ayanamsas_have_no_correction() {
    for ayanamsa in [
        Ayanamsa::Lahiri,
        Ayanamsa::GalacticEquatorTrue,
        Ayanamsa::GalacticCenterMardyks,
    ] {
        assert_eq!(
            apparent_star_ayanamsa_correction(&ayanamsa, tt(2_451_545.0)),
            None
        );
    }
}

// ζ Psc's mean longitude crosses 0°/360° in the window; the correction is a
// difference and must stay within about ±25″ (Review Focus 5).
#[test]
fn the_correction_wraps_at_the_seam() {
    let mut jd = 2_415_020.5;
    while jd <= 2_488_069.5 {
        for ayanamsa in [
            Ayanamsa::TrueRevati,
            Ayanamsa::TrueCitra,
            Ayanamsa::GalacticCenterMulaWilhelm,
        ] {
            let got = arcsec(ayanamsa.clone(), jd);
            assert!(got.abs() < 25.0, "{ayanamsa:?} {jd}: {got}");
        }
        jd += 97.3;
    }
}
