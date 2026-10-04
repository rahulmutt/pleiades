//! White-box checks that the reference module reproduces the pre-existing
//! frame functions bit for bit, and that its two entry points agree.

use super::{ecliptic_in, sampled_place, CrossingReference};
use crate::crossings::CrossingFrame;
use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use pleiades_data::packaged_backend;
use pleiades_types::{Ayanamsa, CelestialBody};

const CASES: [(CelestialBody, &str, f64); 4] = [
    (CelestialBody::Mercury, "Mercury", 2_415_100.25),
    (CelestialBody::Mars, "Mars", 2_451_545.0),
    (CelestialBody::Saturn, "Saturn", 2_439_500.066527),
    (CelestialBody::Pluto, "Pluto", 2_487_900.5),
];

#[test]
fn tropical_references_match_the_frame_wrappers_bitwise() {
    // `validate-crossings` root-finds on these values; a tropical reference
    // must not move them by a single bit.
    let backend = packaged_backend();
    for (body, label, jd) in CASES {
        let apparent = ecliptic_in(
            &backend,
            &body,
            &CrossingFrame::GeocentricApparentOfDate.into(),
            jd,
        )
        .unwrap()
        .0;
        let wrapper = geocentric_apparent_longitude_deg(&backend, body.clone(), label, jd).unwrap();
        assert_eq!(apparent.to_bits(), wrapper.to_bits(), "{label} apparent");

        let helio = ecliptic_in(&backend, &body, &CrossingFrame::Heliocentric.into(), jd)
            .unwrap()
            .0;
        let wrapper = heliocentric_longitude_deg(&backend, body.clone(), label, jd).unwrap();
        assert_eq!(helio.to_bits(), wrapper.to_bits(), "{label} helio");
    }
}

#[test]
fn sampled_place_agrees_with_ecliptic_in() {
    let backend = packaged_backend();
    let references = [
        CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate),
        CrossingReference::tropical(CrossingFrame::Heliocentric),
        CrossingReference::tropical(CrossingFrame::GeocentricMeanOfDate),
        CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri),
        CrossingReference::sidereal(CrossingFrame::GeocentricMeanOfDate, Ayanamsa::Lahiri),
    ];
    for reference in &references {
        for (body, label, jd) in CASES {
            let place = sampled_place(&backend, &body, reference, jd).unwrap();
            let direct = ecliptic_in(&backend, &body, reference, jd).unwrap();
            assert_eq!(
                place.corrected.0.to_bits(),
                direct.0.to_bits(),
                "{reference:?} {label} lon"
            );
            assert_eq!(
                place.corrected.1.to_bits(),
                direct.1.to_bits(),
                "{reference:?} {label} lat"
            );
            assert_eq!(
                place.corrected.2.map(f64::to_bits),
                direct.2.map(f64::to_bits),
                "{reference:?} {label} dist"
            );
        }
    }
}
