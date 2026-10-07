//! Shared fixtures for the rise/set tests.

use std::sync::atomic::{AtomicUsize, Ordering};

use pleiades_apparent::Atmosphere;
use pleiades_backend::{
    BackendMetadata, CompositeBackend, EphemerisBackend, EphemerisError, EphemerisErrorKind,
    EphemerisRequest, EphemerisResult,
};
use pleiades_elp::ElpBackend;
use pleiades_types::{
    CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

use crate::crossings::EventEngine;
use crate::rise_trans::{RiseSetEvent, RiseSetOptions, RiseSetTarget};

/// The artifact-free composite the rise/set issues were reported on.
pub(crate) type Composite = CompositeBackend<ElpBackend, Vsop87Backend>;

pub(crate) fn composite() -> Composite {
    CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new())
}

/// 2025-06-01 06:00 TT, the cost tests' query instant.
pub(crate) const BRACKET_QUERY_JD: f64 = 2_460_827.75;

/// Issue #204's observer.
pub(crate) fn chennai() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(13.08),
        Longitude::from_degrees(80.27),
        Some(0.0),
    )
}

pub(crate) fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

/// The daily bracket of issue #204, the sunrise at or before `at_jd`, the
/// sunset after it and the sunrise after that, as the bits of their
/// Julian days, so callers can compare brackets bit for bit.
pub(crate) fn sun_bracket<B: EphemerisBackend>(engine: &EventEngine<B>, at_jd: f64) -> [u64; 3] {
    let sun = || RiseSetTarget::Body(CelestialBody::Sun);
    let atmos = Atmosphere::default();
    let rise = engine
        .previous_rise_set(
            sun(),
            RiseSetEvent::Rise,
            chennai(),
            atmos,
            RiseSetOptions::default(),
            tt(at_jd),
        )
        .expect("engine ok")
        .expect("a sunrise")
        .instant;
    let set = engine
        .next_rise_set(
            sun(),
            RiseSetEvent::Set,
            chennai(),
            atmos,
            RiseSetOptions::default(),
            rise,
        )
        .expect("engine ok")
        .expect("a sunset")
        .instant;
    let next = engine
        .next_rise_set(
            sun(),
            RiseSetEvent::Rise,
            chennai(),
            atmos,
            RiseSetOptions::default(),
            set,
        )
        .expect("engine ok")
        .expect("a sunrise")
        .instant;
    [rise, set, next].map(|instant| instant.julian_day.days().to_bits())
}

/// A backend that counts the reads made of it.
pub(crate) struct CountingBackend<B> {
    inner: B,
    reads: AtomicUsize,
}

impl<B> CountingBackend<B> {
    pub(crate) fn new(inner: B) -> Self {
        Self {
            inner,
            reads: AtomicUsize::new(0),
        }
    }

    /// Reads made since the last call, or since construction.
    pub(crate) fn take_reads(&self) -> usize {
        self.reads.swap(0, Ordering::Relaxed)
    }
}

impl<B: EphemerisBackend> EphemerisBackend for CountingBackend<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.inner.position(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.inner.position_without_motion(req)
    }
}

/// A backend whose first `failures` reads fail with a numerical error, which
/// the engine reports as `EventError::Backend`, not as a window error.
pub(crate) struct FailingFirstReads<B> {
    inner: B,
    failures: AtomicUsize,
}

impl<B> FailingFirstReads<B> {
    pub(crate) fn new(inner: B, failures: usize) -> Self {
        Self {
            inner,
            failures: AtomicUsize::new(failures),
        }
    }

    fn fail_now(&self) -> bool {
        self.failures
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |left| {
                left.checked_sub(1)
            })
            .is_ok()
    }
}

impl<B: EphemerisBackend> EphemerisBackend for FailingFirstReads<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if self.fail_now() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "injected failure",
            ));
        }
        self.inner.position(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        if self.fail_now() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "injected failure",
            ));
        }
        self.inner.position_without_motion(req)
    }
}
