use crate::*;

#[test]
fn observer_location_has_a_compact_display() {
    let observer = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(35.75),
    );

    assert_eq!(
        observer.summary_line(),
        "latitude=51.5°, longitude=359.9°, elevation=35.750 m"
    );
    assert_eq!(observer.to_string(), observer.summary_line());
}

#[test]
fn observer_location_validation_rejects_invalid_values() {
    let valid = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(45.0),
    );
    assert_eq!(valid.validate(), Ok(()));
    assert_eq!(
        valid.summary_line(),
        "latitude=51.5°, longitude=359.9°, elevation=45.000 m"
    );
    assert_eq!(valid.validated_summary_line(), Ok(valid.summary_line()));

    let bad_latitude = ObserverLocation::new(
        Latitude::from_degrees(91.0),
        Longitude::from_degrees(-0.1),
        None,
    );
    assert_eq!(
        bad_latitude.validate(),
        Err(ObserverLocationValidationError::LatitudeOutOfRange { value: 91.0 })
    );
    assert_eq!(
        ObserverLocationValidationError::LatitudeOutOfRange { value: 91.0 }.summary_line(),
        "observer latitude must stay within [-90, 90], got 91"
    );

    let bad_longitude = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(f64::NAN),
        None,
    );
    assert!(matches!(
        bad_longitude.validate(),
        Err(ObserverLocationValidationError::NonFiniteLongitude { value }) if value.is_nan()
    ));

    let bad_elevation = ObserverLocation::new(
        Latitude::from_degrees(51.5),
        Longitude::from_degrees(-0.1),
        Some(f64::INFINITY),
    );
    assert_eq!(
        bad_elevation.validate(),
        Err(ObserverLocationValidationError::NonFiniteElevation {
            value: f64::INFINITY,
        })
    );
    assert_eq!(
        ObserverLocationValidationError::NonFiniteElevation {
            value: f64::INFINITY,
        }
        .summary_line(),
        "observer elevation must be finite, got inf"
    );
    assert!(bad_elevation.validated_summary_line().is_err());
}
