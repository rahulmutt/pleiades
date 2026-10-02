//! Serialized `Crossing` and `EclipticPosition` values written before the
//! `zodiac` field existed (issue #88) still deserialize, as tropical.

#![cfg(feature = "serde")]

use pleiades_data::packaged_backend;
use pleiades_events::{Crossing, CrossingFrame, CrossingReference, EclipticPosition, EventEngine};
use pleiades_types::{
    Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale, ZodiacMode,
};
use serde_json::Value;

const APPARENT: CrossingFrame = CrossingFrame::GeocentricApparentOfDate;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn sidereal() -> CrossingReference {
    CrossingReference::sidereal(APPARENT, Ayanamsa::Lahiri)
}

fn crossing(reference: CrossingReference) -> Crossing {
    EventEngine::new(packaged_backend())
        .next_longitude_crossing(
            CelestialBody::Sun,
            Longitude::from_degrees(10.0),
            reference,
            tdb(2_451_545.0),
        )
        .unwrap()
        .expect("crossing")
}

fn position(reference: CrossingReference) -> EclipticPosition {
    EventEngine::new(packaged_backend())
        .position_at(CelestialBody::Sun, reference, tdb(2_451_545.0))
        .unwrap()
}

fn without_zodiac(value: &impl serde::Serialize) -> Value {
    let mut json = serde_json::to_value(value).unwrap();
    json.as_object_mut()
        .unwrap()
        .remove("zodiac")
        .expect("zodiac key present when serialized");
    json
}

#[test]
fn a_crossing_without_a_zodiac_key_is_tropical() {
    let old = without_zodiac(&crossing(APPARENT.into()));
    let back: Crossing = serde_json::from_value(old).unwrap();
    assert_eq!(back.zodiac, ZodiacMode::Tropical);
}

#[test]
fn a_position_without_a_zodiac_key_is_tropical() {
    let old = without_zodiac(&position(APPARENT.into()));
    let back: EclipticPosition = serde_json::from_value(old).unwrap();
    assert_eq!(back.zodiac, ZodiacMode::Tropical);
}

#[test]
fn sidereal_values_round_trip() {
    let expected = ZodiacMode::Sidereal {
        ayanamsa: Ayanamsa::Lahiri,
    };

    let json = serde_json::to_value(crossing(sidereal())).unwrap();
    let back: Crossing = serde_json::from_value(json).unwrap();
    assert_eq!(back.zodiac, expected);

    let json = serde_json::to_value(position(sidereal())).unwrap();
    let back: EclipticPosition = serde_json::from_value(json).unwrap();
    assert_eq!(back.zodiac, expected);
}
