//! Pluto: the Meeus Table 37.A periodic-term fit inside 1885–2099 (issue
//! #129) and the mean-element orbit outside it (issue #119).

use super::*;
use crate::elements::OrbitalElements;
use crate::pluto::{PlutoPath, PLUTO_FIT_END_JD, PLUTO_FIT_START_JD};

/// Swiss Ephemeris 2.10 (Moshier) geometric J2000 ecliptic place of Pluto,
/// quoted in issue #119: `(JD TT, longitude deg, latitude deg, distance AU)`.
/// The longitudes are rounded to 0.01°; the latitude and distance are the
/// issue's apparent-of-date values, which differ from J2000 geometric by
/// seconds of arc and the fourth decimal of an AU.
const SWISS_EPHEMERIS_PLUTO_J2000: [(f64, f64, f64, f64); 2] = [
    (2_451_545.0, 251.46, 10.8552, 31.064),
    (2_460_763.5, 303.16, -3.4414, 35.639),
];

#[test]
fn pluto_fit_lands_on_the_rounded_swiss_ephemeris_places() {
    let backend = Vsop87Backend::new();
    for (jd_tt, lon, lat, dist) in SWISS_EPHEMERIS_PLUTO_J2000 {
        let instant = Instant::new(pleiades_types::JulianDay::from_days(jd_tt), TimeScale::Tt);
        let result = backend
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("Pluto query should work");
        let ecliptic = result.ecliptic.expect("ecliptic result should exist");
        assert_eq!(result.quality, QualityAnnotation::Exact);
        assert_degrees_close(ecliptic.longitude.degrees(), lon, 0.01);
        assert_close(ecliptic.latitude.degrees(), lat, 0.01);
        assert_close(
            ecliptic.distance_au.expect("distance should exist"),
            dist,
            0.01,
        );
    }
}

/// Ceilings for the packaged-reference sweep: ceil(1.5 × measured max), measured
/// 2026-10-04 by this test's own 30-day grid (2435 samples): 3.287″ longitude,
/// 0.342″ latitude, 3.27e-4 AU. The packaged (DE440-fitted) Pluto is itself about
/// 1″ from Swiss Ephemeris (`validate-helio-position`).
const SWEEP_LON_CEILING_ARCSEC: f64 = 5.0;
const SWEEP_LAT_CEILING_ARCSEC: f64 = 0.6;
const SWEEP_DIST_CEILING_AU: f64 = 5.0e-4;

#[test]
fn pluto_fit_tracks_the_packaged_pluto_across_1900_2099() {
    // Regression for issue #129: the mean-element Pluto sat 0.4-0.6° from Swiss
    // Ephemeris at every epoch (35.7′ max over 1972-2099).
    let vsop87 = Vsop87Backend::new();
    let packaged = pleiades_data::packaged_backend();
    let (mut max_lon, mut max_lat, mut max_dist) = (0.0_f64, 0.0_f64, 0.0_f64);
    let mut samples = 0_usize;
    let mut jd = 2_415_030.5; // 1900-01-10, clear of the packaged window edge
    while jd < 2_488_060.0 {
        let instant = Instant::new(pleiades_types::JulianDay::from_days(jd), TimeScale::Tt);
        let ours = vsop87
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("VSOP87 Pluto")
            .ecliptic
            .expect("ecliptic");
        let reference = packaged
            .position(&mean_request_at(CelestialBody::Pluto, instant))
            .expect("packaged Pluto")
            .ecliptic
            .expect("ecliptic");
        let lon =
            signed_longitude_delta_degrees(reference.longitude.degrees(), ours.longitude.degrees())
                .abs()
                * 3600.0;
        let lat = (ours.latitude.degrees() - reference.latitude.degrees()).abs() * 3600.0;
        let dist =
            (ours.distance_au.expect("distance") - reference.distance_au.expect("distance")).abs();
        max_lon = max_lon.max(lon);
        max_lat = max_lat.max(lat);
        max_dist = max_dist.max(dist);
        samples += 1;
        jd += 30.0;
    }
    let summary = format!(
        "Pluto vs packaged over {samples} samples: max lon {max_lon:.3}″, lat {max_lat:.3}″, dist {max_dist:.2e} AU"
    );
    // Printed so the ceilings can be re-derived, as the corpus gates do.
    eprintln!("{summary}");
    assert!(samples >= 2_430, "only {samples} samples");
    assert!(
        max_lon <= SWEEP_LON_CEILING_ARCSEC
            && max_lat <= SWEEP_LAT_CEILING_ARCSEC
            && max_dist <= SWEEP_DIST_CEILING_AU,
        "{summary}"
    );
}

fn pluto_at(jd_tt: f64, scale: TimeScale) -> pleiades_backend::EphemerisResult {
    let instant = Instant::new(pleiades_types::JulianDay::from_days(jd_tt), scale);
    Vsop87Backend::new()
        .position(&mean_request_at(CelestialBody::Pluto, instant))
        .expect("Pluto query should work")
}

#[test]
fn the_window_starts_at_1885_inclusive_and_ends_at_2100_exclusive() {
    assert_eq!(PLUTO_FIT_START_JD, 2_409_542.5);
    assert_eq!(PLUTO_FIT_END_JD, 2_488_069.5);
    let cases = [
        (
            PLUTO_FIT_START_JD - 1e-4,
            PlutoPath::MeanElements,
            QualityAnnotation::Approximate,
        ),
        (
            PLUTO_FIT_START_JD,
            PlutoPath::PeriodicTermFit,
            QualityAnnotation::Exact,
        ),
        (
            PLUTO_FIT_END_JD - 1e-4,
            PlutoPath::PeriodicTermFit,
            QualityAnnotation::Exact,
        ),
        (
            PLUTO_FIT_END_JD,
            PlutoPath::MeanElements,
            QualityAnnotation::Approximate,
        ),
    ];
    for (jd, path, quality) in cases {
        assert_eq!(PlutoPath::for_julian_day(jd), path, "{jd}");
        assert_eq!(pluto_at(jd, TimeScale::Tt).quality, quality, "{jd}");
    }
}

#[test]
fn speed_near_a_window_edge_does_not_difference_across_the_jump() {
    // At 2099-12-31 12h the +0.5-day neighbour is outside the window; the centre
    // instant picks the fit for all three samples, so the speed matches a day
    // earlier to well under the ~0.6°/day a jump between paths would add.
    for (edge_side, interior) in [
        (PLUTO_FIT_END_JD - 0.5, PLUTO_FIT_END_JD - 1.5),
        (PLUTO_FIT_START_JD + 0.25, PLUTO_FIT_START_JD + 1.25),
    ] {
        let near = pluto_at(edge_side, TimeScale::Tt).motion.expect("motion");
        let inside = pluto_at(interior, TimeScale::Tt).motion.expect("motion");
        let delta = (near.longitude_deg_per_day.expect("speed")
            - inside.longitude_deg_per_day.expect("speed"))
        .abs();
        assert!(
            delta < 1e-3,
            "speed jumped by {delta}°/day near {edge_side}"
        );
    }
}

#[test]
fn tdb_and_tt_requests_take_the_same_path_bit_for_bit() {
    for jd in [PLUTO_FIT_START_JD, 2_451_545.0, PLUTO_FIT_END_JD] {
        let tt = pluto_at(jd, TimeScale::Tt);
        let tdb = pluto_at(jd, TimeScale::Tdb);
        assert_eq!(tt.quality, tdb.quality, "{jd}");
        assert_eq!(tt.ecliptic, tdb.ecliptic, "{jd}");
        assert_eq!(tt.motion, tdb.motion, "{jd}");
    }
}

#[test]
fn a_batch_mixing_window_sides_keeps_per_instant_quality() {
    let backend = Vsop87Backend::new();
    let requests = [
        PLUTO_FIT_START_JD - 10.0,
        2_451_545.0,
        PLUTO_FIT_END_JD + 10.0,
    ]
    .map(|jd| {
        mean_request_at(
            CelestialBody::Pluto,
            Instant::new(pleiades_types::JulianDay::from_days(jd), TimeScale::Tt),
        )
    });
    let batch = backend.positions(&requests).expect("batch");
    let qualities = batch.iter().map(|r| r.quality).collect::<Vec<_>>();
    assert_eq!(
        qualities,
        vec![
            QualityAnnotation::Approximate,
            QualityAnnotation::Exact,
            QualityAnnotation::Approximate
        ]
    );
    for (request, result) in requests.iter().zip(&batch) {
        let single = backend.position(request).expect("single");
        assert_eq!(single.ecliptic, result.ecliptic);
        assert_eq!(single.quality, result.quality);
    }
}

#[test]
fn far_outside_the_window_pluto_is_still_served_approximately() {
    let result = pluto_at(2_305_447.5, TimeScale::Tt); // 1600-01-01
    assert_eq!(result.quality, QualityAnnotation::Approximate);
    let ecliptic = result.ecliptic.expect("ecliptic");
    assert!(ecliptic.longitude.degrees().is_finite());
    assert!(ecliptic.distance_au.expect("distance").is_finite());
    assert!(result
        .motion
        .expect("motion")
        .longitude_deg_per_day
        .expect("speed")
        .is_finite());
}

#[test]
fn mean_element_orbit_longitude_includes_the_argument_of_perihelion() {
    // A circular orbit in the ecliptic plane, node at 0 deg, perihelion at
    // 90 deg, body at perihelion: the heliocentric longitude is 90 deg. The
    // argument of perihelion is the only non-zero angle, so a path that drops
    // or mis-scales it cannot land there.
    let elements = OrbitalElements::new(0.0, 0.0, 90.0, 1.0, 0.0, 0.0);
    let coords = Vsop87Backend::heliocentric_coordinates(elements);
    let longitude = coords.yh.atan2(coords.xh).to_degrees().rem_euclid(360.0);
    assert_degrees_close(longitude, 90.0, 1e-9);
    assert_close(coords.xh.hypot(coords.yh), 1.0, 1e-12);
}

/// Meeus, Astronomical Algorithms (2nd ed.), Example 37.a: 1992-10-13 0h TD.
#[test]
fn meeus_table_37a_reproduces_example_37a() {
    let lbr = crate::tables::pluto_meeus::pluto_lbr(2_448_908.5);
    assert_degrees_close(
        lbr.longitude_rad.to_degrees().rem_euclid(360.0),
        232.740_71,
        1e-5,
    );
    assert_close(lbr.latitude_rad.to_degrees(), 14.587_82, 1e-5);
    assert_close(lbr.radius_au, 29.711_111, 1e-6);
}

#[test]
fn meeus_table_37a_coefficients_are_pinned() {
    // FNV-1a/64 over the little-endian IEEE-754 bits of all 387 coefficients,
    // row-major. Pinned from the transcription that agrees row for row with
    // two independent implementations of Table 37.A.
    let terms = &crate::tables::pluto_meeus::PLUTO_TERMS;
    assert_eq!(terms.len(), 43);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for row in terms {
        for value in row {
            for byte in value.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    assert_eq!(hash, 0x800f_2aac_998a_fb08);
}

#[test]
fn the_position_switches_path_exactly_at_each_window_bound() {
    // The two paths sit ~0.4-0.6° apart, so a position that kept the wrong path
    // on either side of a bound shows up as a missing jump.
    for bound in [PLUTO_FIT_START_JD, PLUTO_FIT_END_JD] {
        let longitude = |jd| {
            pluto_at(jd, TimeScale::Tt)
                .ecliptic
                .expect("ecliptic")
                .longitude
                .degrees()
        };
        let jump = signed_longitude_delta_degrees(longitude(bound - 1e-4), longitude(bound)).abs();
        assert!(jump > 0.1, "longitude jumped only {jump}° at {bound}");
    }
}

#[test]
fn the_speed_inside_the_window_differences_the_fit_even_next_to_a_bound() {
    for jd in [PLUTO_FIT_START_JD + 0.25, PLUTO_FIT_END_JD - 0.5] {
        let expected =
            Vsop87Backend::motion(CelestialBody::Pluto, jd - J2000, PlutoPath::PeriodicTermFit);
        assert_eq!(pluto_at(jd, TimeScale::Tt).motion, expected, "{jd}");
    }
}
