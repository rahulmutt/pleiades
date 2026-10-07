//! White-box checks of the aspect finder's pure pieces.

use super::{finite_longitude, levels_for, search_step, separation_bound};
use crate::crossings::CrossingFrame;
use crate::error::EventError;
use pleiades_types::{Angle, CelestialBody};

fn levels(degrees: f64) -> Result<Vec<f64>, EventError> {
    levels_for(Angle::from_degrees(degrees))
}

#[test]
fn an_angle_strictly_inside_the_range_has_a_level_on_each_side() {
    assert_eq!(levels(90.0), Ok(vec![90.0, -90.0]));
    assert_eq!(levels(1e-6), Ok(vec![1e-6, -1e-6]));
}

#[test]
fn conjunction_and_opposition_have_one_level() {
    assert_eq!(levels(0.0), Ok(vec![0.0]));
    assert_eq!(levels(-0.0), Ok(vec![-0.0]));
    assert_eq!(levels(180.0), Ok(vec![180.0]));
}

#[test]
fn angles_outside_the_range_are_invalid() {
    for degrees in [
        -1e-9,
        180.000_001,
        360.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let result = levels(degrees);
        assert!(
            matches!(result, Err(EventError::InvalidAspect { .. })),
            "{degrees}: {result:?}"
        );
    }
    let message = levels(200.0).unwrap_err().to_string();
    assert!(message.contains("200"), "{message}");
    assert!(message.contains("0 and 180"), "{message}");
}

#[test]
fn the_step_is_the_smaller_of_the_two_bodies() {
    use CelestialBody::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, TrueNode};
    assert_eq!(search_step(&Sun, &Moon), 0.25);
    assert_eq!(search_step(&Moon, &Saturn), 0.25);
    assert_eq!(search_step(&Mars, &TrueNode), 0.25);
    assert_eq!(search_step(&Mercury, &Saturn), 1.0);
    assert_eq!(search_step(&Saturn, &Sun), 1.0);
    assert_eq!(search_step(&Mars, &Jupiter), 2.0);
}

// A NaN longitude would make every sign test false and read as "no event".
#[test]
fn a_non_finite_longitude_is_missing_coordinates() {
    let body = CelestialBody::Mars;
    assert_eq!(finite_longitude(123.5, &body, 2_451_545.0), Ok(123.5));
    for degrees in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            finite_longitude(degrees, &body, 2_451_545.0),
            Err(EventError::MissingCoordinates {
                body_label: "Mars",
                julian_day: 2_451_545.0,
            }),
            "{degrees}"
        );
    }
}

#[test]
fn only_inner_planet_pairs_in_geocentric_frames_are_bounded() {
    use CelestialBody::{Ceres, Jupiter, Mars, Mercury, Moon, Sun, Venus};
    for frame in [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
    ] {
        for (first, second, bound) in [
            (Sun, Mercury, 28.5),
            (Sun, Venus, 48.5),
            (Mercury, Venus, 77.0),
        ] {
            assert_eq!(separation_bound(&first, &second, frame), Some(bound));
            assert_eq!(separation_bound(&second, &first, frame), Some(bound));
        }
        for (first, second) in [(Sun, Mars), (Sun, Moon), (Mercury, Jupiter), (Ceres, Sun)] {
            assert_eq!(separation_bound(&first, &second, frame), None);
        }
    }
    // Seen from the Sun, the inner planets reach every separation.
    for (first, second) in [(Mercury, Venus), (Venus, Mercury)] {
        assert_eq!(
            separation_bound(&first, &second, CrossingFrame::Heliocentric),
            None
        );
    }
}
