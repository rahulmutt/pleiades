//! White-box checks of the station finder's pure pieces.

use super::{checked_speed, kind_of, step_days, StationKind};
use crate::error::EventError;
use pleiades_types::CelestialBody;

#[test]
fn step_is_scaled_to_the_body() {
    for body in [
        CelestialBody::Moon,
        CelestialBody::MeanNode,
        CelestialBody::TrueNode,
        CelestialBody::MeanApogee,
        CelestialBody::TrueApogee,
        CelestialBody::MeanPerigee,
        CelestialBody::TruePerigee,
    ] {
        assert_eq!(step_days(&body), 0.25, "{body:?}");
    }
    for body in [
        CelestialBody::Sun,
        CelestialBody::Mercury,
        CelestialBody::Venus,
    ] {
        assert_eq!(step_days(&body), 1.0, "{body:?}");
    }
    for body in [
        CelestialBody::Mars,
        CelestialBody::Pluto,
        CelestialBody::Ceres,
    ] {
        assert_eq!(step_days(&body), 2.0, "{body:?}");
    }
}

// `root::bisect` counts zero as the negative side, so the settled instant of
// a turn to retrograde can carry a speed of exactly zero.
#[test]
fn kind_follows_the_sign_of_the_settled_speed() {
    assert_eq!(kind_of(1e-12), StationKind::TurnsDirect);
    assert_eq!(kind_of(0.0), StationKind::TurnsRetrograde);
    assert_eq!(kind_of(-1e-12), StationKind::TurnsRetrograde);
}

#[test]
fn non_finite_speed_is_missing() {
    let body = CelestialBody::Mars;
    assert_eq!(checked_speed(Some(0.25), &body, 2_451_545.0), Ok(0.25));
    for speed in [
        None,
        Some(f64::NAN),
        Some(f64::INFINITY),
        Some(f64::NEG_INFINITY),
    ] {
        assert_eq!(
            checked_speed(speed, &body, 2_451_545.0),
            Err(EventError::MissingSpeed {
                body_label: "Mars",
                julian_day: 2_451_545.0,
            }),
            "{speed:?}"
        );
    }
}
