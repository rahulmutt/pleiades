//! Tests against independent references: the geometric transforms are checked
//! against values worked out by hand from the formula each function
//! implements, the assembled geocentric place and the finite-difference speed
//! against a textbook Kepler solution, and the real element table against rows
//! of the committed Swiss Ephemeris corpus.
//!
//! No expected value here was captured from this crate's output.

use super::*;
use crate::test_support::{
    assert_close, circular_helio_at_epoch, geo_linear_mean_anomaly, helio_kepler_third_law,
    HALF_CENTURY_DAYS,
};

/// A Sun source that places the Sun at a fixed geocentric (lon, lat, distance).
#[derive(Debug, Clone)]
struct FixedSun {
    longitude_deg: f64,
    latitude_deg: f64,
    distance_au: f64,
}

impl EphemerisBackend for FixedSun {
    fn metadata(&self) -> BackendMetadata {
        StubSun.metadata()
    }
    fn supports_body(&self, body: CelestialBody) -> bool {
        body == CelestialBody::Sun
    }
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut r = StubSun.position(req)?;
        r.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(self.longitude_deg),
            Latitude::from_degrees(self.latitude_deg),
            Some(self.distance_au),
        ));
        Ok(r)
    }
}

fn instant(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn sun_at_120_30_distance_2() -> FixedSun {
    FixedSun {
        longitude_deg: 120.0,
        latitude_deg: 30.0,
        distance_au: 2.0,
    }
}

#[test]
fn the_backend_lists_the_nineteen_fictitious_bodies_in_table_order() {
    let bodies = fictitious_bodies();
    assert_eq!(bodies.len(), 19);
    assert_eq!(bodies[0], CelestialBody::Cupido);
    assert_eq!(bodies[18], CelestialBody::Waldemath);
    let claims = fictitious_body_claims();
    assert_eq!(claims.len(), 19);
    let claimed: Vec<CelestialBody> = claims.into_iter().map(|c| c.body).collect();
    assert_eq!(claimed, bodies);
}

#[test]
fn spherical_to_cartesian_matches_the_hand_computed_components() {
    // x = r cos(lat) cos(lon), y = r cos(lat) sin(lon), z = r sin(lat).
    let case = |lon: f64, lat: f64, r: Option<f64>| {
        spherical_to_cartesian(&EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            r,
        ))
    };
    let check = |got: (f64, f64, f64), want: [f64; 3], what: &str| {
        assert_close(got.0, want[0], 1.0e-12, &format!("{what} x"));
        assert_close(got.1, want[1], 1.0e-12, &format!("{what} y"));
        assert_close(got.2, want[2], 1.0e-12, &format!("{what} z"));
    };
    let sqrt3 = 3.0_f64.sqrt();
    // (30 deg, 0, 2) -> (2 cos 30, 2 sin 30, 0) = (sqrt 3, 1, 0).
    check(case(30.0, 0.0, Some(2.0)), [sqrt3, 1.0, 0.0], "30/0/2");
    // (60, 30, 2): cos 30 = sqrt3/2 -> (2 * sqrt3/2 * 1/2, 2 * sqrt3/2 * sqrt3/2, 2 * 1/2).
    check(
        case(60.0, 30.0, Some(2.0)),
        [sqrt3 / 2.0, 1.5, 1.0],
        "60/30/2",
    );
    // Poles: z = +-r, x = y = 0 whatever the longitude.
    check(case(77.0, 90.0, Some(4.0)), [0.0, 0.0, 4.0], "north pole");
    check(case(77.0, -90.0, Some(4.0)), [0.0, 0.0, -4.0], "south pole");
    // Longitude just below 360 with a southern latitude: (359, -45, 3).
    check(
        case(359.0, -45.0, Some(3.0)),
        [
            2.120_997_256_196_473,
            -0.037_022_144_819_480_886,
            -2.121_320_343_559_642_4,
        ],
        "359/-45/3",
    );
    // No distance: the unit sphere.
    check(case(90.0, 0.0, None), [0.0, 1.0, 0.0], "unit sphere");
}

#[test]
fn cartesian_to_ecliptic_matches_the_hand_computed_spherical_triple() {
    let check = |x: f64, y: f64, z: f64, lon: f64, lat: f64, r: f64| {
        let ecl = cartesian_to_ecliptic(x, y, z);
        assert_close(ecl.longitude.degrees(), lon, 1.0e-12, "longitude");
        assert_close(ecl.latitude.degrees(), lat, 1.0e-12, "latitude");
        assert_close(ecl.distance_au.unwrap(), r, 1.0e-12, "distance");
    };
    // (1, 2, 2): r = sqrt(1 + 4 + 4) = 3, lon = atan2(2, 1), lat = asin(2/3).
    check(
        1.0,
        2.0,
        2.0,
        63.434_948_822_922_01,
        41.810_314_895_778_596,
        3.0,
    );
    // Third quadrant, negative z: longitude wraps into [0, 360).
    check(
        -1.0,
        -2.0,
        -2.0,
        243.434_948_822_922_02,
        -41.810_314_895_778_596,
        3.0,
    );
    // (3, -4, 12): r = 13, lon = atan2(-4, 3) + 360, lat = asin(12/13).
    check(
        3.0,
        -4.0,
        12.0,
        306.869_897_645_844_05,
        67.380_135_051_959_58,
        13.0,
    );
}

#[test]
fn cartesian_to_ecliptic_of_the_origin_is_latitude_zero_without_nan() {
    let ecl = cartesian_to_ecliptic(0.0, 0.0, 0.0);
    assert_eq!(ecl.latitude.degrees(), 0.0);
    assert_eq!(ecl.longitude.degrees(), 0.0);
    assert_eq!(ecl.distance_au, Some(0.0));
}

#[test]
fn earth_is_the_negated_sun_vector() {
    // Sun at (120, 30, 2): (2 cos 30 cos 120, 2 cos 30 sin 120, 2 sin 30)
    //   = (-sqrt3/2, 3/2, 1), so Earth heliocentric = (sqrt3/2, -3/2, -1).
    let backend = FictitiousBackend::new(sun_at_120_30_distance_2());
    let earth = backend
        .earth_heliocentric(instant(crate::J2000_JD))
        .unwrap();
    assert_close(earth[0], 3.0_f64.sqrt() / 2.0, 1.0e-12, "x");
    assert_close(earth[1], -1.5, 1.0e-12, "y");
    assert_close(earth[2], -1.0, 1.0e-12, "z");
}

#[test]
fn a_heliocentric_body_is_geocentricised_by_subtracting_earth() {
    // Body at (3, 0, 0) heliocentric, Earth at (sqrt3/2, -3/2, -1):
    // geocentric = (3 - sqrt3/2, 3/2, 1), |g| = sqrt(7.8038...) = 2.79353...,
    // lon = atan2(1.5, 2.13397...) = 35.1039 deg, lat = asin(1 / 2.79353...).
    let backend = FictitiousBackend::new(sun_at_120_30_distance_2());
    let ecl = backend
        .geocentric_ecliptic(&circular_helio_at_epoch(3.0), instant(crate::J2000_JD))
        .unwrap();
    assert_close(ecl.longitude.degrees(), 35.103_909_361_017_1, 1.0e-9, "lon");
    assert_close(ecl.latitude.degrees(), 20.975_527_369_423_12, 1.0e-9, "lat");
    assert_close(
        ecl.distance_au.unwrap(),
        2.793_536_750_660_955_6,
        1.0e-12,
        "r",
    );
}

#[test]
fn a_geocentric_orbit_is_returned_without_the_sun() {
    // The Sun's place must not enter: any Sun gives the same geocentric place.
    let el = geo_linear_mean_anomaly();
    let at = instant(crate::J2000_JD + 1000.0);
    for sun in [
        sun_at_120_30_distance_2(),
        FixedSun {
            longitude_deg: 10.0,
            latitude_deg: -5.0,
            distance_au: 0.98,
        },
    ] {
        let ecl = FictitiousBackend::new(sun)
            .geocentric_ecliptic(&el, at)
            .unwrap();
        assert_close(
            ecl.longitude.degrees(),
            249.354_715_410_538_43,
            1.0e-9,
            "lon",
        );
        assert_close(
            ecl.latitude.degrees(),
            -10.114_335_877_421_63,
            1.0e-9,
            "lat",
        );
        assert_close(
            ecl.distance_au.unwrap(),
            1.851_027_842_858_465_7,
            1.0e-12,
            "r",
        );
    }
}

#[test]
fn motion_is_the_central_difference_over_one_day() {
    // Reference: (place(jd + 0.5) - place(jd - 0.5)) / 1.0 d of the textbook
    // Kepler solution for `geo_linear_mean_anomaly` at J2000 + 1000 d.
    let backend = FictitiousBackend::new(StubSun);
    let motion = backend
        .motion(
            &geo_linear_mean_anomaly(),
            instant(crate::J2000_JD + 1000.0),
        )
        .unwrap();
    assert_close(
        motion.longitude_deg_per_day.unwrap(),
        1.825_237_734_511_517_8,
        1.0e-9,
        "lon speed",
    );
    assert_close(
        motion.latitude_deg_per_day.unwrap(),
        -0.562_300_688_310_635_6,
        1.0e-9,
        "lat speed",
    );
    assert_close(
        motion.distance_au_per_day.unwrap(),
        0.012_087_231_102_868_579,
        1.0e-10,
        "distance speed",
    );
}

#[test]
fn motion_across_the_longitude_seam_is_the_short_signed_arc() {
    // `geo_linear_mean_anomaly` at JD 2452598.8125: longitude 357.0989 deg a
    // half day before, 0.0764 deg a half day after (textbook Kepler solution).
    // The speed is the +2.9775 deg/day short arc, not -356.9 deg/day.
    let motion = FictitiousBackend::new(StubSun)
        .motion(&geo_linear_mean_anomaly(), instant(2_452_598.812_5))
        .unwrap();
    assert_close(
        motion.longitude_deg_per_day.unwrap(),
        2.977_469_650_716_33,
        1.0e-9,
        "lon speed",
    );
    assert_close(
        motion.latitude_deg_per_day.unwrap(),
        0.768_410_660_313_179_5,
        1.0e-9,
        "lat speed",
    );
    assert_close(
        motion.distance_au_per_day.unwrap(),
        -0.024_077_383_673_174_024,
        1.0e-10,
        "distance speed",
    );
}

#[test]
fn a_heliocentric_body_moves_against_a_fixed_earth() {
    // `helio_kepler_third_law` at J2000 + 0.5 century, Earth fixed at
    // (sqrt3/2, -3/2, -1): geocentric place lon 334.48920, lat 1.13771,
    // r 5.06621; central-difference speeds from the textbook solution.
    let backend = FictitiousBackend::new(sun_at_120_30_distance_2());
    let el = helio_kepler_third_law();
    let at = instant(crate::J2000_JD + HALF_CENTURY_DAYS);
    let place = backend.geocentric_ecliptic(&el, at).unwrap();
    assert_close(
        place.longitude.degrees(),
        334.489_202_789_355_2,
        1.0e-9,
        "lon",
    );
    assert_close(
        place.latitude.degrees(),
        1.137_712_710_490_465_9,
        1.0e-9,
        "lat",
    );
    assert_close(
        place.distance_au.unwrap(),
        5.066_208_794_718_233,
        1.0e-10,
        "r",
    );
    let motion = backend.motion(&el, at).unwrap();
    assert_close(
        motion.longitude_deg_per_day.unwrap(),
        0.060_690_591_194_880_43,
        1.0e-9,
        "lon speed",
    );
    assert_close(
        motion.latitude_deg_per_day.unwrap(),
        -0.008_932_952_400_931_171,
        1.0e-9,
        "lat speed",
    );
    assert_close(
        motion.distance_au_per_day.unwrap(),
        0.000_819_880_624_938_207_3,
        1.0e-10,
        "distance speed",
    );
}

#[test]
fn a_one_sided_probe_divides_by_half_a_day() {
    // `helio_kepler_third_law` at J2000 + 0.5 century against the stub Sun
    // (Earth fixed at (-1, 0, 0)). With the Sun source covering only one side
    // of the instant, the speed is (place difference) / 0.5 d. Reference: the
    // textbook solution's geocentric place at jd and jd +- 0.5.
    let jd = crate::J2000_JD + HALF_CENTURY_DAYS;
    let el = helio_kepler_third_law();
    let cases = [
        // Sun covers [jd, jd + 1]: the earlier probe fails, speed = (after - at) / 0.5.
        (
            jd,
            jd + 1.0,
            [
                0.041_817_975_409_799_146,
                -0.005_486_007_778_133_128_5,
                0.000_524_422_213_272_757_2,
            ],
        ),
        // Sun covers [jd - 1, jd]: the later probe fails, speed = (at - before) / 0.5.
        (
            jd - 1.0,
            jd,
            [
                0.041_818_579_266_646_64,
                -0.005_488_573_939_935_293,
                0.000_525_703_312_229_453,
            ],
        ),
    ];
    for (lo, hi, want) in cases {
        let motion = FictitiousBackend::new(WindowedSun { lo, hi })
            .motion(&el, instant(jd))
            .unwrap();
        assert_close(
            motion.longitude_deg_per_day.unwrap(),
            want[0],
            1.0e-9,
            "lon speed",
        );
        assert_close(
            motion.latitude_deg_per_day.unwrap(),
            want[1],
            1.0e-9,
            "lat speed",
        );
        assert_close(
            motion.distance_au_per_day.unwrap(),
            want[2],
            1.0e-10,
            "distance speed",
        );
    }
}

/// Corpus rows: `crates/pleiades-validate/data/fictitious-corpus/fictitious.csv`
/// (Swiss Ephemeris 2.10.03, geometric geocentric J2000 ecliptic, TT). The
/// geocentric-orbit bodies need no Sun, so the rows pin the real element table
/// end to end. Tolerances are the `validate-fictitious` gate ceilings
/// (`fictitious_thresholds.rs`): 0.65 arcsec lon, 0.09 arcsec lat, 3.5e-6 AU.
#[test]
fn geocentric_orbit_bodies_reproduce_corpus_rows() {
    const LON_TOL: f64 = 0.65 / 3600.0;
    const LAT_TOL: f64 = 0.09 / 3600.0;
    const DIST_TOL: f64 = 3.5e-6;
    // (body, jd_tt, lon_deg, lat_deg, dist_au) from the CSV lines
    // `WhiteMoon,56,2415020.500000000,140.145713409,0.007487617,0.052800989`,
    // `WhiteMoon,56,2450285.534482758,64.907640105,0.000423102,0.052800989`,
    // `Waldemath,58,2415020.500000000,276.980656303,0.763593268,0.006971821`,
    // `Waldemath,58,2450285.534482758,167.243815055,2.265122216,0.007620587`.
    let rows = [
        (
            CelestialBody::WhiteMoon,
            2_415_020.5,
            140.145_713_409,
            0.007_487_617,
            0.052_800_989,
        ),
        (
            CelestialBody::WhiteMoon,
            2_450_285.534_482_758,
            64.907_640_105,
            0.000_423_102,
            0.052_800_989,
        ),
        (
            CelestialBody::Waldemath,
            2_415_020.5,
            276.980_656_303,
            0.763_593_268,
            0.006_971_821,
        ),
        (
            CelestialBody::Waldemath,
            2_450_285.534_482_758,
            167.243_815_055,
            2.265_122_216,
            0.007_620_587,
        ),
    ];
    let backend = FictitiousBackend::new(StubSun);
    for (body, jd, lon, lat, dist) in rows {
        let result = backend
            .position(&EphemerisRequest::new(body.clone(), instant(jd)))
            .unwrap();
        let ecl = result.ecliptic.unwrap();
        // Shortest signed arc, computed here so the reference path does not
        // depend on production code.
        let dlon = (ecl.longitude.degrees() - lon + 540.0) % 360.0 - 180.0;
        assert!(dlon.abs() < LON_TOL, "{body:?} {jd} lon off by {dlon}");
        assert_close(
            ecl.latitude.degrees(),
            lat,
            LAT_TOL,
            &format!("{body:?} {jd} lat"),
        );
        assert_close(
            ecl.distance_au.unwrap(),
            dist,
            DIST_TOL,
            &format!("{body:?} {jd} r"),
        );
    }
}
