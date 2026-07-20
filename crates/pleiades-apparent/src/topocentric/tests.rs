//! White-box unit tests for the topocentric module.
//!
//! Relocated out of `topocentric.rs` per AGENTS.md ("keep large inline test
//! suites out of the file under test"). These remain white-box unit tests with
//! access to the module's internals — they are deliberately not converted to
//! black-box integration tests.

use super::*;

fn ecl(lon: f64, lat: f64, dist: f64) -> EclipticCoordinates {
    EclipticCoordinates::new(
        Longitude::from_degrees(lon),
        Latitude::from_degrees(lat),
        Some(dist),
    )
}

fn observer(lat: f64) -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(lat),
        Longitude::from_degrees(0.0),
        Some(0.0),
    )
}

#[test]
fn moon_parallax_is_about_one_degree() {
    // Moon at ~0.00257 AU (60.3 Earth radii). For an observer with the Moon
    // near the horizon the parallax approaches ~0.95°. Assert it is large.
    let out = topocentric_position(ecl(100.0, 0.0, 0.002_57), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec)
        / 3600.0;
    assert!(shift > 0.3, "moon parallax {shift}° too small");
}

#[test]
fn distant_body_parallax_is_negligible() {
    // A body at 30 AU: parallax < 1".
    let out = topocentric_position(ecl(100.0, 0.0, 30.0), &observer(0.0), 100.0, 23.4).unwrap();
    let shift = out
        .provenance
        .parallax_longitude_arcsec
        .hypot(out.provenance.parallax_latitude_arcsec);
    assert!(shift < 1.0, "distant parallax {shift}\" too large");
}

#[test]
fn missing_distance_errors() {
    let no_dist = EclipticCoordinates::new(
        Longitude::from_degrees(100.0),
        Latitude::from_degrees(0.0),
        None,
    );
    let err = topocentric_position(no_dist, &observer(0.0), 100.0, 23.4).unwrap_err();
    assert_eq!(err, ApparentPlaceError::MissingDistance);
}

#[test]
fn diurnal_aberration_is_sub_arcsec() {
    let out = topocentric_position(ecl(100.0, 0.0, 1.0), &observer(0.0), 100.0, 23.4).unwrap();
    assert!(
        out.provenance.diurnal_aberration_arcsec < 0.36,
        "diurnal aberration {}\"",
        out.provenance.diurnal_aberration_arcsec
    );
}

/// Meeus ch. 11 worked-example observer (Palomar): φ = +33.356111°, 1706 m.
/// ρcosφ′ ≈ 0.836339 ≠ 1, which is what makes the diurnal-aberration factor
/// mutants (`* rho_cos_phi_prime` → `/`) distinguishable at all.
fn palomar() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(33.356_111),
        Longitude::from_degrees(0.0),
        Some(1706.0),
    )
}

// ---------------------------------------------------------------------------
// FU-9 exact-literal tests. Every expected value below was computed OUTSIDE
// this crate by an independent Python reimplementation of the published
// pipeline — Meeus ch. 11 observer terms (WGS84), ch. 40 rectangular
// diurnal-parallax subtraction, the classical diurnal-aberration terms
// (Δα = 0.3192″ ρcosφ′ cos H / cos δ, Δδ = 0.3192″ ρcosφ′ sin H sin δ), and
// the standard ecliptic↔equatorial rotation. The script is reproduced in
// docs/superpowers/plans/2026-07-20-fu9-topocentric-mutant-triage.md
// (Appendix A). Reference-vs-crate agreement is ~1e-11″, far inside the
// tolerances asserted here; the literals are the reference's output, never
// this crate's own.
// ---------------------------------------------------------------------------

#[test]
fn palomar_moon_matches_independent_meeus_pipeline() {
    // λ=100°, β=+5°, Δ=0.00257 AU, ε=23.44°, LAST=70°, Palomar.
    // Discriminating geometry (spec §6): ρcosφ′≈0.836, dec_topo≈27.9°,
    // H≈328.2° — no factor is 0, 1, or otherwise mutation-degenerate.
    let out = topocentric_position(ecl(100.0, 5.0, 0.002_57), &palomar(), 70.0, 23.44).unwrap();

    let lon = out.ecliptic.longitude.degrees();
    let lat = out.ecliptic.latitude.degrees();
    let dist = out.ecliptic.distance_au.unwrap();
    assert!((lon - 100.430_618_719_114_62).abs() < 1e-9, "lon {lon}");
    assert!((lat - 4.891_647_280_609_852).abs() < 1e-9, "lat {lat}");
    assert!(
        (dist - 0.002_532_223_707_150_349_7).abs() < 1e-12,
        "dist {dist}"
    );

    let p = &out.provenance;
    assert!(
        (p.parallax_longitude_arcsec - 1_550.227_388_812_629_7).abs() < 1e-6,
        "parallax lon {}",
        p.parallax_longitude_arcsec
    );
    assert!(
        (p.parallax_latitude_arcsec - -390.069_789_804_534).abs() < 1e-6,
        "parallax lat {}",
        p.parallax_latitude_arcsec
    );
    assert!(
        (p.diurnal_aberration_arcsec - 0.236_287_334_372_904_58).abs() < 1e-6,
        "diurnal {}",
        p.diurnal_aberration_arcsec
    );
    // Deliberately tight (1e-15): `distance_au_used` is a verbatim pass-through
    // of the input distance, so bit-equality holds — the tight bound is what
    // kills replacement mutants on this field.
    assert!(
        (p.distance_au_used - 0.002_57).abs() < 1e-15,
        "distance used {}",
        p.distance_au_used
    );
}

#[test]
fn parallax_displaces_toward_horizon() {
    // Direction, not just magnitude: for a body above the observer's horizon
    // the observer is closer to it than the geocenter (topocentric distance
    // shrinks) and the ecliptic shift has the reference-predicted signs. The
    // pre-existing magnitude tests are sign-free (`hypot`), which is exactly
    // why the L53–55 subtraction-sign mutants survived until this slice.
    let out = topocentric_position(ecl(100.0, 5.0, 0.002_57), &palomar(), 70.0, 23.44).unwrap();
    assert!(out.ecliptic.distance_au.unwrap() < 0.002_57);
    assert!(out.provenance.parallax_longitude_arcsec > 0.0);
    assert!(out.provenance.parallax_latitude_arcsec < 0.0);
}

#[test]
fn wrap_westward_across_zero() {
    // λ=0.02°: the ~0.93° westward parallax carries the topocentric longitude
    // across 0° to ~359.09°, so raw Δlon = +359.07° and the `> 180 → −360`
    // wrap branch fires. Unwrapped, the provenance would read ~+1.29e6″; the
    // exact literal pins the wrap and its direction.
    let out = topocentric_position(ecl(0.02, 0.0, 0.002_57), &palomar(), 80.0, 23.44).unwrap();
    let lon = out.ecliptic.longitude.degrees();
    assert!((lon - 359.092_861_428_952_8).abs() < 1e-9, "lon {lon}");
    assert!(
        (out.provenance.parallax_longitude_arcsec - -3_337.698_855_769_849_6).abs() < 1e-6,
        "parallax lon {}",
        out.provenance.parallax_longitude_arcsec
    );
    assert!(
        (out.provenance.parallax_latitude_arcsec - -597.126_617_152_460_1).abs() < 1e-6,
        "parallax lat {}",
        out.provenance.parallax_latitude_arcsec
    );
}

#[test]
fn wrap_eastward_across_zero() {
    // λ=359.98°: the ~0.51° eastward parallax carries the topocentric
    // longitude across 360° to ~0.49°, so raw Δlon = −359.49° and the
    // `< −180 → +360` wrap branch fires.
    let out = topocentric_position(ecl(359.98, 0.0, 0.002_57), &palomar(), 280.0, 23.44).unwrap();
    let lon = out.ecliptic.longitude.degrees();
    assert!((lon - 0.492_685_616_714_902_74).abs() < 1e-9, "lon {lon}");
    assert!(
        (out.provenance.parallax_longitude_arcsec - 1_845.668_220_173_502).abs() < 1e-6,
        "parallax lon {}",
        out.provenance.parallax_longitude_arcsec
    );
    assert!(
        (out.provenance.parallax_latitude_arcsec - -2_844.541_308_965_105).abs() < 1e-6,
        "parallax lat {}",
        out.provenance.parallax_latitude_arcsec
    );
}

#[test]
fn non_finite_inputs_fail_closed() {
    // Guard intent: a non-finite LAST or obliquity yields the typed error,
    // never a NaN coordinate. This cannot distinguish the remaining `||`→`&&`
    // guard mutant at the output-finiteness check (L95, documented equivalent
    // — see `overflowing_distance_fails_closed` for the L57 guard, which this
    // does not exercise) — it pins the fail-closed contract itself.
    let err = topocentric_position(ecl(100.0, 5.0, 1.0), &palomar(), f64::NAN, 23.44).unwrap_err();
    assert_eq!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric"
        }
    );
    let err = topocentric_position(ecl(100.0, 5.0, 1.0), &palomar(), 70.0, f64::NAN).unwrap_err();
    assert_eq!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric"
        }
    );
}

#[test]
fn overflowing_distance_fails_closed() {
    // A finite but absurdly large distance overflows the squared-norm sum to
    // +inf. The guard must reject it: under the `||`→`&&` mutant at the
    // topo_distance check, `inf <= 0.0` is false, the guard passes, and
    // `tz / inf == 0.0` keeps every downstream value finite — the function
    // would return Ok with `distance_au = Some(inf)`. This pins fail-closed
    // behavior on the overflow path (found in review; kills the L57 mutant
    // the spec originally classified as equivalent).
    let err = topocentric_position(ecl(100.0, 0.0, 1e301), &palomar(), 70.0, 23.44).unwrap_err();
    assert_eq!(
        err,
        ApparentPlaceError::NonFiniteCorrection {
            stage: "topocentric"
        }
    );
}
