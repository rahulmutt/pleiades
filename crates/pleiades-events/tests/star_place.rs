//! `CrossingReference::with_star_place` (issue #164 (c)).

use pleiades_core::apparent_star_ayanamsa_correction;
use pleiades_data::packaged_backend;
use pleiades_events::{CrossingFrame, CrossingReference, EventEngine, SiderealStarPlace};
use pleiades_types::{Ayanamsa, CelestialBody, Instant, JulianDay, Longitude, TimeScale};

const JD: f64 = 2_460_000.5;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn reference(
    frame: CrossingFrame,
    ayanamsa: Ayanamsa,
    place: SiderealStarPlace,
) -> CrossingReference {
    CrossingReference::sidereal(frame, ayanamsa).with_star_place(place)
}

fn lon(body: CelestialBody, r: CrossingReference, jd: f64) -> f64 {
    EventEngine::new(packaged_backend())
        .longitude_at(body, r, tdb(jd))
        .expect("longitude")
        .degrees()
}

fn wrap(d: f64) -> f64 {
    (d + 540.0).rem_euclid(360.0) - 180.0
}

#[test]
fn a_reference_defaults_to_the_mean_star_place() {
    let r =
        CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::TrueCitra);
    assert_eq!(r.star_place, SiderealStarPlace::Mean);
    assert_eq!(
        CrossingReference::from(CrossingFrame::Heliocentric).star_place,
        SiderealStarPlace::Mean
    );
}

// The events twin of the correction equals core's (spec amendment 10), at
// ten days inside both ends of the 1900-2100 window and across True Revati's 0°/360° seam (issue #226).
#[test]
fn the_apparent_frame_moves_by_core_s_correction() {
    for (ayanamsa, jd) in [
        Ayanamsa::TrueCitra,
        Ayanamsa::TruePushya,
        Ayanamsa::TrueRevati,
        Ayanamsa::GalacticCenterMulaWilhelm,
    ]
    .into_iter()
    .flat_map(|ayanamsa| [JD, 2_415_030.5, 2_488_059.5].map(|jd| (ayanamsa.clone(), jd)))
    {
        let correction = apparent_star_ayanamsa_correction(
            &ayanamsa,
            Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
        )
        .unwrap()
        .degrees();
        for body in [
            CelestialBody::Sun,
            CelestialBody::Mars,
            CelestialBody::TrueNode,
        ] {
            let geo = CrossingFrame::GeocentricApparentOfDate;
            let mean = lon(
                body.clone(),
                reference(geo, ayanamsa.clone(), SiderealStarPlace::Mean),
                jd,
            );
            let app = lon(
                body.clone(),
                reference(geo, ayanamsa.clone(), SiderealStarPlace::Apparent),
                jd,
            );
            assert!(
                (wrap(app - mean) + correction).abs() < 1e-12,
                "{ayanamsa:?} {jd} {body:?}"
            );
        }
    }
}

#[test]
fn the_mean_and_heliocentric_frames_and_unanchored_ayanamsas_are_unchanged() {
    for (frame, ayanamsa, body) in [
        (
            CrossingFrame::GeocentricMeanOfDate,
            Ayanamsa::TrueCitra,
            CelestialBody::Mars,
        ),
        (
            CrossingFrame::Heliocentric,
            Ayanamsa::TrueCitra,
            CelestialBody::Mars,
        ),
        (
            CrossingFrame::GeocentricApparentOfDate,
            Ayanamsa::Lahiri,
            CelestialBody::Mars,
        ),
    ] {
        let mean = lon(
            body.clone(),
            reference(frame, ayanamsa.clone(), SiderealStarPlace::Mean),
            JD,
        );
        let app = lon(
            body.clone(),
            reference(frame, ayanamsa.clone(), SiderealStarPlace::Apparent),
            JD,
        );
        assert_eq!(mean, app, "{frame:?} {ayanamsa:?}");
    }
}

#[test]
fn a_crossing_under_the_apparent_star_place_is_at_the_target_longitude() {
    let engine = EventEngine::new(packaged_backend());
    let r = reference(
        CrossingFrame::GeocentricApparentOfDate,
        Ayanamsa::TrueCitra,
        SiderealStarPlace::Apparent,
    );
    let crossing = engine
        .next_longitude_crossing(
            CelestialBody::Sun,
            Longitude::from_degrees(0.0),
            r.clone(),
            tdb(JD),
        )
        .expect("search")
        .expect("the Sun crosses 0° every year");
    let at = lon(CelestialBody::Sun, r, crossing.instant.julian_day.days());
    assert!(wrap(at).abs() < 1e-6, "{at}");
}
