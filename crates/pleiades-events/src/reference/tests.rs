//! White-box checks that the reference module reproduces the pre-existing
//! frame functions bit for bit, and that its two entry points agree.

use super::{ecliptic_in, sampled_place, CrossingReference};
use crate::crossings::CrossingFrame;
use crate::ephemeris::{geocentric_apparent_longitude_deg, heliocentric_longitude_deg};
use pleiades_backend::{
    BackendMetadata, CompositeBackend, EphemerisBackend, EphemerisError, EphemerisRequest,
    EphemerisResult,
};
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
        CrossingReference::sidereal(CrossingFrame::Heliocentric, Ayanamsa::Lahiri),
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

/// Which backend entry point a query used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    Full,
    MotionFree,
}

type Query = (CelestialBody, f64, Entry);

/// Records every `(body, julian_day, entry point)` the wrapped backend is asked for.
struct Recording<B> {
    inner: B,
    queries: std::sync::Mutex<Vec<Query>>,
}

impl<B> Recording<B> {
    fn new(inner: B) -> Self {
        Self {
            inner,
            queries: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn record(&self, req: &EphemerisRequest, entry: Entry) {
        self.queries
            .lock()
            .unwrap()
            .push((req.body.clone(), req.instant.julian_day.days(), entry));
    }

    fn take(&self) -> Vec<Query> {
        std::mem::take(&mut *self.queries.lock().unwrap())
    }
}

impl<B: EphemerisBackend> EphemerisBackend for Recording<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.record(req, Entry::Full);
        self.inner.position(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.record(req, Entry::MotionFree);
        self.inner.position_without_motion(req)
    }
}

/// A backend that drops every distance.
struct NoDistance<B>(B);

impl<B: EphemerisBackend> EphemerisBackend for NoDistance<B> {
    fn metadata(&self) -> BackendMetadata {
        self.0.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.0.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = self.0.position(req)?;
        if let Some(ecliptic) = result.ecliptic.as_mut() {
            ecliptic.distance_au = None;
        }
        Ok(result)
    }
}

/// Queries of `body`, at `jd` when given.
fn count(queries: &[Query], body: &CelestialBody, jd: Option<f64>) -> usize {
    queries
        .iter()
        .filter(|(b, j, _)| b == body && jd.is_none_or(|jd| *j == jd))
        .count()
}

/// 2025-03-29 00:00 TDB, the issue #128 measurement start.
const ISSUE_128_JD: f64 = 2_460_763.5;

#[test]
fn an_apparent_sample_of_a_body_does_not_query_the_sun() {
    // Regression for issue #128: every apparent sample of every body queried
    // the backend for the Sun, only to feed a provenance estimate this engine
    // discards. For the Moon that query cost ten times the Moon's own.
    let backend = Recording::new(packaged_backend());
    let reference = CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate);
    for body in [
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Pluto,
    ] {
        ecliptic_in(&backend, &body, &reference, ISSUE_128_JD).unwrap();
        let queries = backend.take();
        assert_eq!(count(&queries, &CelestialBody::Sun, None), 0, "{body:?}");
        // The light-time loop reads the body once at the instant itself.
        assert_eq!(
            count(&queries, &body, Some(ISSUE_128_JD)),
            1,
            "{body:?}: {queries:?}"
        );
    }
}

#[test]
fn sampled_place_reads_the_body_once_at_its_instant() {
    // Regression for issue #128: the speed sample read the mean place and then
    // read it again inside the reduction.
    let backend = Recording::new(packaged_backend());
    for frame in [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
    ] {
        for body in [
            CelestialBody::Sun,
            CelestialBody::Moon,
            CelestialBody::Mars,
            CelestialBody::TrueNode,
        ] {
            let reference = CrossingReference::tropical(frame);
            sampled_place(&backend, &body, &reference, ISSUE_128_JD).unwrap();
            let queries = backend.take();
            assert_eq!(
                count(&queries, &body, Some(ISSUE_128_JD)),
                1,
                "{frame:?} {body:?}: {queries:?}"
            );
            if body != CelestialBody::Sun {
                assert_eq!(
                    count(&queries, &CelestialBody::Sun, None),
                    0,
                    "{frame:?} {body:?}"
                );
            }
        }
    }
}

#[test]
fn sampled_place_agrees_with_ecliptic_in_for_the_luminaries_and_lunar_points() {
    // The reused read must reproduce the unseeded reduction bit for bit on the
    // Sun's own path, the Moon, and the lunar-point paths, including the ELP
    // mean node, which the backend serves without a distance.
    let composite = CompositeBackend::new(
        pleiades_elp::ElpBackend::new(),
        pleiades_vsop87::Vsop87Backend::new(),
    );
    let packaged = packaged_backend();
    let references = [
        CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate),
        CrossingReference::tropical(CrossingFrame::GeocentricMeanOfDate),
        CrossingReference::sidereal(CrossingFrame::GeocentricApparentOfDate, Ayanamsa::Lahiri),
    ];
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::TrueNode,
        CelestialBody::MeanNode,
    ];
    for reference in &references {
        for body in &bodies {
            for jd in [2_415_100.25, ISSUE_128_JD] {
                let pairs = [
                    (
                        sampled_place(&packaged, body, reference, jd)
                            .unwrap()
                            .corrected,
                        ecliptic_in(&packaged, body, reference, jd).unwrap(),
                    ),
                    (
                        sampled_place(&composite, body, reference, jd)
                            .unwrap()
                            .corrected,
                        ecliptic_in(&composite, body, reference, jd).unwrap(),
                    ),
                ];
                for (place, direct) in pairs {
                    let context = format!("{reference:?} {body:?} {jd}");
                    assert_eq!(place.0.to_bits(), direct.0.to_bits(), "{context} lon");
                    assert_eq!(place.1.to_bits(), direct.1.to_bits(), "{context} lat");
                    assert_eq!(
                        place.2.map(f64::to_bits),
                        direct.2.map(f64::to_bits),
                        "{context} dist"
                    );
                }
            }
        }
    }
}

#[test]
fn sampled_place_still_requires_a_distance_for_a_body() {
    // Without a distance a body cannot be reduced; reusing the read must keep
    // the pre-existing error in both geocentric frames, whatever its wording.
    let backend = NoDistance(packaged_backend());
    for frame in [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
    ] {
        let reference = CrossingReference::tropical(frame);
        let seeded = sampled_place(&backend, &CelestialBody::Mars, &reference, ISSUE_128_JD);
        let direct = ecliptic_in(&backend, &CelestialBody::Mars, &reference, ISSUE_128_JD);
        let seeded = format!("{:?}", seeded.err().expect("no distance must fail"));
        let direct = format!("{:?}", direct.expect_err("no distance must fail"));
        assert_eq!(seeded, direct, "{frame:?}");
        assert!(
            seeded.to_lowercase().contains("distance"),
            "{frame:?}: {seeded}"
        );
    }
}

#[test]
fn mean_only_reads_never_ask_for_motion() {
    // Issue #128: apparent re-queries and the mean frames discard motion.
    let backend = Recording::new(packaged_backend());
    for frame in [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
    ] {
        for body in [
            CelestialBody::Moon,
            CelestialBody::Mars,
            CelestialBody::TrueNode,
        ] {
            let reference = CrossingReference::tropical(frame);
            ecliptic_in(&backend, &body, &reference, ISSUE_128_JD).unwrap();
            let queries = backend.take();
            assert!(!queries.is_empty());
            assert!(
                queries.iter().all(|(_, _, e)| *e == Entry::MotionFree),
                "{frame:?} {body:?}: {queries:?}"
            );
        }
    }
}

#[test]
fn sampled_place_asks_for_motion_once() {
    // The speed sample does need the backend's motion, at its instant only.
    let backend = Recording::new(packaged_backend());
    let reference = CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate);
    sampled_place(&backend, &CelestialBody::Mars, &reference, ISSUE_128_JD).unwrap();
    let queries = backend.take();
    let full: Vec<_> = queries
        .iter()
        .filter(|(_, _, e)| *e == Entry::Full)
        .collect();
    assert_eq!(full.len(), 1, "{queries:?}");
    assert_eq!(full[0].1, ISSUE_128_JD);
}
