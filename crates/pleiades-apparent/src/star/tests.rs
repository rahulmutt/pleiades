use super::*;

// Swiss Ephemeris, measured 2026-10-07 (libswisseph-sys 0.1.2, Moshier):
// the anchor stars' geometric places at J2000 (swe_fixstar,
// MOSEPH|NONUT|TRUEPOS|NOABERR|NOGDEFL) and their apparent − geometric
// ayanamsa there (swe_get_ayanamsa_ex, MOSEPH|NONUT) — an independent
// reference for deflection + aberration.
#[test]
fn anchor_stars_match_swiss_ephemeris_at_j2000() {
    let jd = 2_451_545.0;
    let cases = [
        // (λ, β, SE correction ″)
        (203.841_362_759, -2.054_487_222, -4.8521), // Spica (True Citra)
        (19.877_543_860, -0.213_433_452, 3.4348),   // ζ Psc (True Revati)
        (128.721_995_524, 0.077_172_355, 18.3457),  // δ Cnc (True Pushya)
        (264.585_713_184, -13.788_463_334, -20.6651), // λ Sco (True Mula)
        (266.851_709_361, -5.607_686_222, -20.3888), // Sgr A* (Galactic Center)
    ];
    for (lambda, beta, expected) in cases {
        let (app, _) = apparent_star_place(lambda, beta, jd);
        let got = ((app - lambda + 540.0).rem_euclid(360.0) - 180.0) * 3600.0;
        assert!(
            (got - expected).abs() < 0.06,
            "{lambda}: {got} vs {expected}"
        );
    }
}

#[test]
fn polar_projection_is_the_identity_on_the_ecliptic_at_the_equinoxes_and_solstices() {
    for lambda in [0.0_f64, 90.0, 180.0, 270.0] {
        let p = polar_projection_deg(lambda, 0.0, 23.44);
        assert!(
            (p - lambda).abs() < 1e-9 || (p - lambda).abs() > 359.999_999,
            "{lambda} {p}"
        );
    }
}

#[test]
fn polar_projection_differs_from_longitude_off_the_ecliptic() {
    // A point 5° south of the ecliptic at λ = 240° has a different right
    // ascension than the ecliptic point at 240°, so its projection moves.
    let p = polar_projection_deg(240.0, -5.6, 23.44);
    assert!((p - 240.0).abs() > 0.5, "{p}");
}
