//! Shared fixtures for the corpus-gate unit tests.

use pleiades_backend::{
    BackendMetadata, EphemerisBackend, EphemerisError, EphemerisErrorKind, EphemerisRequest,
    EphemerisResult,
};
use pleiades_data::PackagedDataBackend;
use pleiades_types::CelestialBody;

/// A backend that answers every query with `OutOfRangeInstant`, so a gate that
/// skips out-of-range rows would otherwise validate nothing and still pass.
pub(crate) struct AlwaysOutOfRange(PackagedDataBackend);

impl AlwaysOutOfRange {
    pub(crate) fn new() -> Self {
        Self(PackagedDataBackend::new())
    }
}

impl EphemerisBackend for AlwaysOutOfRange {
    fn metadata(&self) -> BackendMetadata {
        self.0.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.0.supports_body(body)
    }

    fn position(&self, _req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        Err(EphemerisError::new(
            EphemerisErrorKind::OutOfRangeInstant,
            "test stub: every instant is out of range",
        ))
    }
}
