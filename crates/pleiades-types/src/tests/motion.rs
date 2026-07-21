use crate::*;

#[test]
fn motion_accessors_return_the_original_speed_components() {
    let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

    assert_eq!(motion.longitude_speed(), Some(0.12));
    assert_eq!(motion.latitude_speed(), Some(-0.03));
    assert_eq!(motion.distance_speed(), Some(0.000_4));
}

#[test]
fn motion_summary_line_matches_display() {
    let motion = Motion::new(Some(0.12), Some(-0.03), Some(0.000_4));

    assert_eq!(
        motion.summary_line(),
        "longitude=0.12 deg/day; latitude=-0.03 deg/day; distance=0.0004 au/day"
    );
    assert_eq!(motion.to_string(), motion.summary_line());
}

#[test]
fn motion_direction_tracks_the_sign_of_longitude_speed() {
    assert_eq!(
        Motion::new(Some(0.12), None, None).longitude_direction(),
        Some(MotionDirection::Direct)
    );
    assert_eq!(
        Motion::new(Some(-0.04), None, None).longitude_direction(),
        Some(MotionDirection::Retrograde)
    );
    assert_eq!(
        Motion::new(Some(0.0), None, None).longitude_direction(),
        Some(MotionDirection::Stationary)
    );
    assert_eq!(Motion::new(None, None, None).longitude_direction(), None);
    assert_eq!(
        Motion::new(Some(f64::NAN), None, None).longitude_direction(),
        None
    );
}

#[test]
fn motion_validation_rejects_non_finite_components() {
    let longitude = Motion::new(Some(f64::INFINITY), None, None);
    assert_eq!(
        longitude.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "longitude_deg_per_day",
            value: f64::INFINITY,
        })
    );

    let latitude = Motion::new(None, Some(f64::NAN), None);
    assert!(matches!(
        latitude.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "latitude_deg_per_day",
            value,
        }) if value.is_nan()
    ));

    let distance = Motion::new(None, None, Some(f64::NEG_INFINITY));
    assert_eq!(
        distance.validate(),
        Err(MotionValidationError::NonFiniteSpeed {
            field: "distance_au_per_day",
            value: f64::NEG_INFINITY,
        })
    );
}
