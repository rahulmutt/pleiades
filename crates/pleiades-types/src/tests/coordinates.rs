use crate::*;

#[test]
fn ecliptic_to_equatorial_preserves_zero_obliquity_identity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(123.45),
        Latitude::from_degrees(-6.75),
        Some(0.123),
    );

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(Angle::from_degrees(0.0));

    assert_eq!(equatorial.validate(), Ok(()));
    assert_eq!(equatorial.right_ascension.degrees(), 123.45);
    assert!((equatorial.declination.degrees() + 6.75).abs() < 1e-12);
    assert_eq!(equatorial.distance_au, Some(0.123));
}

#[test]
fn ecliptic_to_equatorial_rotates_by_mean_obliquity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(Angle::from_degrees(23.439_291_11));

    assert_eq!(equatorial.validate(), Ok(()));
    assert_eq!(equatorial.right_ascension.degrees(), 90.0);
    assert!((equatorial.declination.degrees() - 23.439_291_11).abs() < 1e-10);
    assert_eq!(equatorial.distance_au, Some(1.0));
}

#[test]
fn equatorial_to_ecliptic_round_trip_uses_the_same_obliquity() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(123.45),
        Latitude::from_degrees(-6.75),
        Some(0.123),
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(ecliptic.validate(), Ok(()));

    let equatorial = ecliptic.to_equatorial(obliquity);
    assert_eq!(equatorial.validate(), Ok(()));
    let round_trip = equatorial.to_ecliptic(obliquity);

    assert_eq!(round_trip.validate(), Ok(()));
    assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
    assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
    assert_eq!(round_trip.distance_au, Some(0.123));
}

#[test]
fn mean_obliquity_round_trip_stays_stable_across_quadrants() {
    let cases = [
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(359.2),
                Latitude::from_degrees(11.75),
                None,
            ),
            Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
        ),
        (
            EclipticCoordinates::new(
                Longitude::from_degrees(27.5),
                Latitude::from_degrees(-33.25),
                Some(2.5),
            ),
            Instant::new(JulianDay::from_days(2_459_000.5), TimeScale::Tt),
        ),
    ];

    for (ecliptic, instant) in cases {
        assert_eq!(ecliptic.validate(), Ok(()));
        let obliquity = instant.mean_obliquity();
        let equatorial = ecliptic.to_equatorial(obliquity);
        assert_eq!(equatorial.validate(), Ok(()));
        let round_trip = equatorial.to_ecliptic(obliquity);

        assert_eq!(round_trip.validate(), Ok(()));
        assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
        assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
        assert_eq!(round_trip.distance_au, ecliptic.distance_au);
    }
}

#[test]
fn mean_obliquity_round_trip_stays_stable_near_the_poles() {
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let obliquity = instant.mean_obliquity();

    for ecliptic in [
        EclipticCoordinates::new(
            Longitude::from_degrees(0.025),
            Latitude::from_degrees(89.75),
            Some(0.42),
        ),
        EclipticCoordinates::new(
            Longitude::from_degrees(179.975),
            Latitude::from_degrees(-89.75),
            Some(0.42),
        ),
    ] {
        assert_eq!(ecliptic.validate(), Ok(()));
        let equatorial = ecliptic.to_equatorial(obliquity);
        assert_eq!(equatorial.validate(), Ok(()));
        let round_trip = equatorial.to_ecliptic(obliquity);

        assert_eq!(round_trip.validate(), Ok(()));
        assert!((round_trip.longitude.degrees() - ecliptic.longitude.degrees()).abs() < 1e-10);
        assert!((round_trip.latitude.degrees() - ecliptic.latitude.degrees()).abs() < 1e-10);
        assert_eq!(round_trip.distance_au, Some(0.42));
    }
}

#[test]
fn ecliptic_to_equatorial_normalizes_negative_right_ascension() {
    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(180.0),
        Latitude::from_degrees(30.0),
        None,
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(ecliptic.validate(), Ok(()));
    let equatorial = ecliptic.to_equatorial(obliquity);

    assert_eq!(equatorial.validate(), Ok(()));
    assert!(equatorial.right_ascension.degrees() >= 0.0);
    assert!(equatorial.right_ascension.degrees() < 360.0);
    assert!((equatorial.right_ascension.degrees() - 192.934_084_332_518_07).abs() < 1e-10);
    assert!((equatorial.declination.degrees() - 27.305_898_332_307_97).abs() < 1e-10);
}

#[test]
fn equatorial_to_ecliptic_round_trip_preserves_negative_declination_near_wraparound() {
    let equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(359.5),
        Latitude::from_degrees(-27.5),
        Some(3.25),
    );
    let obliquity = Angle::from_degrees(23.439_291_11);

    assert_eq!(equatorial.validate(), Ok(()));

    let ecliptic = equatorial.to_ecliptic(obliquity);
    assert_eq!(ecliptic.validate(), Ok(()));
    let round_trip = ecliptic.to_equatorial(obliquity);

    assert_eq!(round_trip.validate(), Ok(()));
    assert!(
        (round_trip.right_ascension.degrees() - equatorial.right_ascension.degrees()).abs() < 1e-10
    );
    assert!((round_trip.declination.degrees() - equatorial.declination.degrees()).abs() < 1e-10);
    assert_eq!(round_trip.distance_au, equatorial.distance_au);
}

#[test]
fn equatorial_to_ecliptic_treats_negative_right_ascension_as_normalized_angle() {
    let obliquity = Angle::from_degrees(23.439_291_11);
    let normalized = EquatorialCoordinates::new(
        Angle::from_degrees(359.75),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );
    let wrapped = EquatorialCoordinates::new(
        Angle::from_degrees(-0.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );

    let normalized_ecliptic = normalized.to_ecliptic(obliquity);
    let wrapped_ecliptic = wrapped.to_ecliptic(obliquity);

    assert_eq!(normalized_ecliptic.validate(), Ok(()));
    assert_eq!(wrapped_ecliptic.validate(), Ok(()));
    assert!(
        (normalized_ecliptic.longitude.degrees() - wrapped_ecliptic.longitude.degrees()).abs()
            < 1e-10
    );
    assert!(
        (normalized_ecliptic.latitude.degrees() - wrapped_ecliptic.latitude.degrees()).abs()
            < 1e-10
    );
    assert_eq!(
        normalized_ecliptic.distance_au,
        wrapped_ecliptic.distance_au
    );

    let round_trip = wrapped_ecliptic.to_equatorial(obliquity);
    assert_eq!(round_trip.validate(), Ok(()));
    assert!(round_trip.right_ascension.degrees() >= 0.0);
    assert!(round_trip.right_ascension.degrees() < 360.0);
    assert!((round_trip.right_ascension.degrees() - 359.75).abs() < 1e-10);
}

#[test]
fn equatorial_to_ecliptic_treats_full_turn_right_ascension_as_normalized_angle() {
    let obliquity = Angle::from_degrees(23.439_291_11);
    let normalized = EquatorialCoordinates::new(
        Angle::from_degrees(0.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );
    let wrapped = EquatorialCoordinates::new(
        Angle::from_degrees(360.25),
        Latitude::from_degrees(12.5),
        Some(1.23),
    );

    let normalized_ecliptic = normalized.to_ecliptic(obliquity);
    let wrapped_ecliptic = wrapped.to_ecliptic(obliquity);

    assert_eq!(normalized_ecliptic.validate(), Ok(()));
    assert_eq!(wrapped_ecliptic.validate(), Ok(()));
    assert!(
        (normalized_ecliptic.longitude.degrees() - wrapped_ecliptic.longitude.degrees()).abs()
            < 1e-10
    );
    assert!(
        (normalized_ecliptic.latitude.degrees() - wrapped_ecliptic.latitude.degrees()).abs()
            < 1e-10
    );
    assert_eq!(
        normalized_ecliptic.distance_au,
        wrapped_ecliptic.distance_au
    );

    let round_trip = wrapped_ecliptic.to_equatorial(obliquity);
    assert_eq!(round_trip.validate(), Ok(()));
    assert!(round_trip.right_ascension.degrees() >= 0.0);
    assert!(round_trip.right_ascension.degrees() < 360.0);
    assert!((round_trip.right_ascension.degrees() - 0.25).abs() < 1e-10);
}

#[test]
fn coordinate_validation_rejects_non_finite_and_out_of_range_values() {
    let bad_ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(91.0),
        Some(-1.0),
    );
    assert_eq!(
        bad_ecliptic.validate(),
        Err(CoordinateValidationError::LatitudeOutOfRange {
            coordinate: "ecliptic",
            field: "latitude",
            value: 91.0,
        })
    );
    assert_eq!(
        CoordinateValidationError::LatitudeOutOfRange {
            coordinate: "ecliptic",
            field: "latitude",
            value: 91.0,
        }
        .summary_line(),
        "ecliptic coordinate field `latitude` must stay within [-90, 90], got 91"
    );

    let bad_distance = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(12.0),
        Some(-1.0),
    );
    assert_eq!(
        bad_distance.validate(),
        Err(CoordinateValidationError::NegativeDistance {
            coordinate: "ecliptic",
            value: -1.0,
        })
    );

    let bad_non_finite_distance = EclipticCoordinates::new(
        Longitude::from_degrees(12.0),
        Latitude::from_degrees(12.0),
        Some(f64::NAN),
    );
    assert!(matches!(
        bad_non_finite_distance.validate(),
        Err(CoordinateValidationError::NonFiniteValue {
            coordinate: "ecliptic",
            field: "distance_au",
            value,
        }) if value.is_nan()
    ));

    let bad_equatorial = EquatorialCoordinates::new(
        Angle::from_degrees(360.0),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    assert_eq!(
        bad_equatorial.validate(),
        Err(CoordinateValidationError::RightAscensionOutOfRange {
            coordinate: "equatorial",
            field: "right_ascension",
            value: 360.0,
        })
    );
    assert_eq!(
        CoordinateValidationError::NegativeDistance {
            coordinate: "ecliptic",
            value: -0.5,
        }
        .summary_line(),
        "ecliptic coordinate field `distance_au` must be non-negative, got -0.5"
    );
}
