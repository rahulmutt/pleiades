//! How many backend queries an apparent chart makes (issue #128).
//!
//! The apparent reduction re-queries a body at light-time-retarded instants,
//! but its first query is always at the sampled instant itself, which the
//! chart has already read: in the position batch at the chart instant, and in
//! the mean-place query at each speed-difference instant. These tests pin
//! that each such read happens once, and that the reuse leaves the placement
//! bit-identical to the reduction driven by fresh backend queries.

use std::sync::{Arc, Mutex};

use pleiades_apparent::{
    apparent_position, precess_ecliptic_j2000_to_date, DEFAULT_MAX_ITERATIONS,
};
use pleiades_backend::{
    Apparentness, BackendMetadata, CompositeBackend, EphemerisBackend, EphemerisError,
    EphemerisRequest, EphemerisResult,
};
use pleiades_elp::ElpBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest};

/// 2025-03-29 00:00 TT, the issue #128 measurement start.
const ISSUE_128_JD: f64 = 2_460_763.5;

fn composite() -> CompositeBackend<ElpBackend, Vsop87Backend> {
    CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new())
}

type QueryLog = Arc<Mutex<Vec<(CelestialBody, f64)>>>;

/// Records every `(body, julian_day)` the wrapped backend is asked for.
struct Recording<B> {
    inner: B,
    queries: QueryLog,
}

impl<B: EphemerisBackend> EphemerisBackend for Recording<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.queries
            .lock()
            .unwrap()
            .push((req.body.clone(), req.instant.julian_day.days()));
        self.inner.position(req)
    }
}

fn chart_instant() -> Instant {
    Instant::new(JulianDay::from_days(ISSUE_128_JD), TimeScale::Tt)
}

fn apparent_request(bodies: Vec<CelestialBody>) -> ChartRequest {
    ChartRequest::new(chart_instant())
        .with_bodies(bodies)
        .with_apparentness(Apparentness::Apparent)
}

#[test]
fn an_apparent_chart_reads_each_body_once_per_sampled_instant() {
    let bodies = vec![
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ];
    let queries = QueryLog::default();
    let backend = Recording {
        inner: composite(),
        queries: Arc::clone(&queries),
    };
    let snapshot = ChartEngine::new(backend)
        .chart(&apparent_request(bodies.clone()))
        .expect("apparent chart");
    let queries = queries.lock().unwrap().clone();
    for body in &bodies {
        let placement = snapshot.placement_for(body).expect("placement");
        assert_eq!(
            placement.position.apparent,
            Apparentness::Apparent,
            "{body:?}"
        );
        // The chart instant and the two speed-difference instants.
        for jd in [ISSUE_128_JD - 0.5, ISSUE_128_JD, ISSUE_128_JD + 0.5] {
            let reads = queries
                .iter()
                .filter(|(b, j)| b == body && *j == jd)
                .count();
            assert_eq!(reads, 1, "{body:?} at {jd}: {queries:?}");
        }
    }
}

#[test]
fn reusing_the_batch_read_leaves_the_apparent_place_bit_identical() {
    // The reduction driven by fresh backend queries, as the chart computed it
    // before it reused its batch read.
    let backend = composite();
    let instant = chart_instant();
    let mean = |body: &CelestialBody, at: Instant| {
        let mut request = EphemerisRequest::new(body.clone(), at);
        request.apparent = Apparentness::Mean;
        backend
            .position(&request)
            .map(|result| result.ecliptic.expect("ecliptic"))
    };
    let sun = mean(&CelestialBody::Sun, instant).expect("Sun");
    let sun_lon = precess_ecliptic_j2000_to_date(
        sun.longitude.degrees(),
        sun.latitude.degrees(),
        ISSUE_128_JD,
    )
    .expect("precession")
    .longitude_deg;

    let bodies = vec![
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
    ];
    let snapshot = ChartEngine::new(composite())
        .chart(&apparent_request(bodies.clone()))
        .expect("apparent chart");
    for body in &bodies {
        let expected = apparent_position::<_, EphemerisError>(
            instant,
            sun_lon,
            DEFAULT_MAX_ITERATIONS,
            |at| mean(body, at),
        )
        .expect("apparent place");
        let placement = snapshot.placement_for(body).expect("placement");
        let got = placement.position.ecliptic.expect("ecliptic");
        assert_eq!(
            got.longitude.degrees().to_bits(),
            expected.ecliptic.longitude.degrees().to_bits(),
            "{body:?} longitude"
        );
        assert_eq!(
            got.latitude.degrees().to_bits(),
            expected.ecliptic.latitude.degrees().to_bits(),
            "{body:?} latitude"
        );
        assert_eq!(
            placement.apparent.as_ref().expect("provenance"),
            &expected.provenance,
            "{body:?} provenance"
        );
    }
}
