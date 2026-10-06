//! Shared fixtures for the rise/set tests.

use std::sync::atomic::{AtomicUsize, Ordering};

use pleiades_backend::{
    BackendMetadata, CompositeBackend, EphemerisBackend, EphemerisError, EphemerisRequest,
    EphemerisResult,
};
use pleiades_elp::ElpBackend;
use pleiades_types::CelestialBody;
use pleiades_vsop87::Vsop87Backend;

/// The artifact-free composite the rise/set issues were reported on.
pub(crate) type Composite = CompositeBackend<ElpBackend, Vsop87Backend>;

pub(crate) fn composite() -> Composite {
    CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new())
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
