use crate::capabilities::BackendCapabilities;
use crate::errors::{EphemerisError, EphemerisErrorKind};
use crate::identity::{AccuracyClass, BackendFamily, BackendId};
use crate::metadata::{BackendMetadata, BackendProvenance};
use crate::request::EphemerisRequest;
use crate::result::EphemerisResult;
use pleiades_types::{CelestialBody, Instant, TimeRange, TimeScale};

/// The shared backend contract.
///
/// Implementations must support one-request/one-result queries. Batch querying
/// is provided as a default all-or-error adapter that fail-fast stops on the
/// first structured error so callers can build chart-style workflows without
/// hand-rolling request loops.
pub trait EphemerisBackend: Send + Sync {
    /// Returns backend metadata.
    fn metadata(&self) -> BackendMetadata;

    /// Returns whether the backend supports the requested body.
    fn supports_body(&self, body: CelestialBody) -> bool;

    /// Computes a single ephemeris result.
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError>;

    /// Computes multiple ephemeris results.
    ///
    /// The default adapter calls [`Self::position`] for each request in order and
    /// preserves each request's own instant and time-scale label exactly as
    /// supplied, so mixed TT/TDB batches remain mixed in the returned results
    /// instead of being normalized to a batch-wide scale.
    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        reqs.iter().map(|req| self.position(req)).collect()
    }
}

/// A simple composite backend that routes requests to one of two providers.
///
/// The primary backend is consulted first. If it does not advertise support for
/// the requested body, the secondary backend is tried instead.
#[derive(Debug)]
pub struct CompositeBackend<A, B> {
    primary: A,
    secondary: B,
}

impl<A, B> CompositeBackend<A, B> {
    /// Creates a new routing backend.
    pub const fn new(primary: A, secondary: B) -> Self {
        Self { primary, secondary }
    }

    /// Returns the primary backend.
    pub const fn primary(&self) -> &A {
        &self.primary
    }

    /// Returns the secondary backend.
    pub const fn secondary(&self) -> &B {
        &self.secondary
    }
}

impl<A: EphemerisBackend, B: EphemerisBackend> EphemerisBackend for CompositeBackend<A, B> {
    fn metadata(&self) -> BackendMetadata {
        let primary = self.primary.metadata();
        let secondary = self.secondary.metadata();
        BackendMetadata {
            id: BackendId::new(format!(
                "composite:{}+{}",
                primary.id.as_str(),
                secondary.id.as_str()
            )),
            version: primary.version.clone(),
            family: BackendFamily::Composite,
            provenance: BackendProvenance {
                summary: format!(
                    "Composite routing backend combining {} and {}.",
                    primary.provenance.summary, secondary.provenance.summary
                ),
                data_sources: combine_sources(
                    &primary.provenance.data_sources,
                    &secondary.provenance.data_sources,
                ),
            },
            nominal_range: intersect_ranges(primary.nominal_range, secondary.nominal_range),
            supported_time_scales: intersect_strings(
                &primary.supported_time_scales,
                &secondary.supported_time_scales,
            ),
            body_coverage: combine_bodies(&primary.body_coverage, &secondary.body_coverage),
            supported_frames: intersect_strings(
                &primary.supported_frames,
                &secondary.supported_frames,
            ),
            capabilities: BackendCapabilities {
                geocentric: primary.capabilities.geocentric && secondary.capabilities.geocentric,
                topocentric: primary.capabilities.topocentric && secondary.capabilities.topocentric,
                apparent: primary.capabilities.apparent && secondary.capabilities.apparent,
                mean: primary.capabilities.mean && secondary.capabilities.mean,
                batch: primary.capabilities.batch && secondary.capabilities.batch,
                native_sidereal: primary.capabilities.native_sidereal
                    && secondary.capabilities.native_sidereal,
            },
            accuracy: min_accuracy(primary.accuracy, secondary.accuracy),
            deterministic: primary.deterministic && secondary.deterministic,
            offline: primary.offline && secondary.offline,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.primary.supports_body(body.clone()) || self.secondary.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let primary_supports = self.primary.supports_body(req.body.clone());
        let secondary_supports = self.secondary.supports_body(req.body.clone());

        if primary_supports {
            match self.primary.position(req) {
                Ok(result) => Ok(result),
                Err(error) if secondary_supports && should_fallback_to_secondary(&error.kind) => {
                    self.secondary.position(req)
                }
                Err(error) => Err(error),
            }
        } else if secondary_supports {
            self.secondary.position(req)
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the composite router supports the requested body",
            ))
        }
    }
}

/// A routing backend that can chain any number of providers.
///
/// The router queries providers in priority order and falls back to later
/// backends when the earlier ones report a retryable routing error. This makes
/// it convenient to compose packaged, algorithmic, and reference-data
/// backends without nesting multiple binary composites.
#[derive(Default)]
pub struct RoutingBackend {
    backends: Vec<Box<dyn EphemerisBackend>>,
}

impl RoutingBackend {
    /// Creates a new routing backend from a prioritized list of providers.
    pub fn new(backends: Vec<Box<dyn EphemerisBackend>>) -> Self {
        Self { backends }
    }

    /// Returns the configured provider chain.
    pub fn backends(&self) -> &[Box<dyn EphemerisBackend>] {
        &self.backends
    }

    /// Returns `true` if no providers are configured.
    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }
}

impl EphemerisBackend for RoutingBackend {
    fn metadata(&self) -> BackendMetadata {
        let backends: Vec<&dyn EphemerisBackend> = self
            .backends
            .iter()
            .map(|backend| backend.as_ref())
            .collect();
        let metadatas: Vec<BackendMetadata> =
            backends.iter().map(|backend| backend.metadata()).collect();

        if metadatas.is_empty() {
            return BackendMetadata {
                id: BackendId::new("routing:empty"),
                version: "routing[none]".to_string(),
                family: BackendFamily::Composite,
                provenance: BackendProvenance::new("Routing backend with no configured providers."),
                nominal_range: TimeRange::new(None, None),
                supported_time_scales: Vec::new(),
                body_coverage: Vec::new(),
                supported_frames: Vec::new(),
                capabilities: BackendCapabilities {
                    geocentric: false,
                    topocentric: false,
                    apparent: false,
                    mean: false,
                    batch: false,
                    native_sidereal: false,
                },
                accuracy: AccuracyClass::Unknown,
                deterministic: true,
                offline: true,
            };
        }

        let mut id_parts = Vec::with_capacity(metadatas.len());
        let mut version_parts = Vec::with_capacity(metadatas.len());
        let mut provenance_parts = Vec::with_capacity(metadatas.len());
        let mut data_sources = Vec::new();
        let mut nominal_range = metadatas[0].nominal_range;
        let mut supported_time_scales = metadatas[0].supported_time_scales.clone();
        let mut body_coverage = metadatas[0].body_coverage.clone();
        let mut supported_frames = metadatas[0].supported_frames.clone();
        let mut capabilities = metadatas[0].capabilities.clone();
        let mut accuracy = metadatas[0].accuracy;
        let mut deterministic = metadatas[0].deterministic;
        let mut offline = metadatas[0].offline;

        for metadata in &metadatas {
            id_parts.push(metadata.id.as_str().to_string());
            version_parts.push(metadata.version.clone());
            provenance_parts.push(metadata.provenance.summary.clone());
            data_sources = combine_sources(&data_sources, &metadata.provenance.data_sources);
            nominal_range = intersect_ranges(nominal_range, metadata.nominal_range);
            supported_time_scales =
                intersect_strings(&supported_time_scales, &metadata.supported_time_scales);
            body_coverage = combine_bodies(&body_coverage, &metadata.body_coverage);
            supported_frames = intersect_strings(&supported_frames, &metadata.supported_frames);
            capabilities.geocentric &= metadata.capabilities.geocentric;
            capabilities.topocentric &= metadata.capabilities.topocentric;
            capabilities.apparent &= metadata.capabilities.apparent;
            capabilities.mean &= metadata.capabilities.mean;
            capabilities.batch &= metadata.capabilities.batch;
            capabilities.native_sidereal &= metadata.capabilities.native_sidereal;
            accuracy = min_accuracy(accuracy, metadata.accuracy);
            deterministic &= metadata.deterministic;
            offline &= metadata.offline;
        }

        BackendMetadata {
            id: BackendId::new(format!("routing:{}", id_parts.join("+"))),
            version: format!("routing[{}]", version_parts.join("+")),
            family: BackendFamily::Composite,
            provenance: BackendProvenance {
                summary: format!(
                    "Routing backend combining {} provider(s): {}.",
                    metadatas.len(),
                    provenance_parts.join("; ")
                ),
                data_sources,
            },
            nominal_range,
            supported_time_scales,
            body_coverage,
            supported_frames,
            capabilities,
            accuracy,
            deterministic,
            offline,
        }
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.backends
            .iter()
            .any(|backend| backend.supports_body(body.clone()))
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut saw_support = false;
        let mut last_retryable_error = None;

        for backend in &self.backends {
            if !backend.supports_body(req.body.clone()) {
                continue;
            }

            saw_support = true;
            match backend.position(req) {
                Ok(result) => return Ok(result),
                Err(error) if should_fallback_to_secondary(&error.kind) => {
                    last_retryable_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        if let Some(error) = last_retryable_error {
            Err(error)
        } else if saw_support {
            Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "configured providers could not satisfy the requested body and request shape",
            ))
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the routing chain supports the requested body",
            ))
        }
    }
}

fn combine_sources(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut combined = primary.to_vec();
    for source in secondary {
        if !combined.iter().any(|existing| existing == source) {
            combined.push(source.clone());
        }
    }
    combined
}

fn combine_bodies(primary: &[CelestialBody], secondary: &[CelestialBody]) -> Vec<CelestialBody> {
    let mut combined = primary.to_vec();
    for body in secondary {
        if !combined.contains(body) {
            combined.push(body.clone());
        }
    }
    combined
}

fn intersect_strings<T: Clone + PartialEq>(primary: &[T], secondary: &[T]) -> Vec<T> {
    primary
        .iter()
        .filter(|value| secondary.contains(value))
        .cloned()
        .collect()
}

fn intersect_ranges(primary: TimeRange, secondary: TimeRange) -> TimeRange {
    let start = match (primary.start, secondary.start) {
        (Some(a), Some(b)) => Some(if a.julian_day.days() >= b.julian_day.days() {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    let end = match (primary.end, secondary.end) {
        (Some(a), Some(b)) => Some(if a.julian_day.days() <= b.julian_day.days() {
            a
        } else {
            b
        }),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };

    let canonical_scale = primary
        .start
        .or(primary.end)
        .or(secondary.start)
        .or(secondary.end)
        .map(|instant| instant.scale);

    TimeRange::new(
        start.map(|instant| retag_instant(instant, canonical_scale)),
        end.map(|instant| retag_instant(instant, canonical_scale)),
    )
}

fn retag_instant(instant: Instant, scale: Option<TimeScale>) -> Instant {
    match scale {
        Some(scale) if instant.scale != scale => Instant::new(instant.julian_day, scale),
        _ => instant,
    }
}

fn min_accuracy(primary: AccuracyClass, secondary: AccuracyClass) -> AccuracyClass {
    use AccuracyClass::*;

    match (primary, secondary) {
        (Unknown, _) | (_, Unknown) => Unknown,
        (Approximate, _) | (_, Approximate) => Approximate,
        (Moderate, _) | (_, Moderate) => Moderate,
        (High, _) | (_, High) => High,
        (Exact, Exact) => Exact,
    }
}

fn should_fallback_to_secondary(kind: &EphemerisErrorKind) -> bool {
    matches!(
        kind,
        EphemerisErrorKind::UnsupportedBody
            | EphemerisErrorKind::UnsupportedCoordinateFrame
            | EphemerisErrorKind::UnsupportedTimeScale
            | EphemerisErrorKind::InvalidObserver
            | EphemerisErrorKind::UnsupportedObserver
            | EphemerisErrorKind::MissingDataset
            | EphemerisErrorKind::UnsupportedApparentness
            | EphemerisErrorKind::UnsupportedZodiacMode
            | EphemerisErrorKind::InvalidRequest
    )
}

#[cfg(test)]
#[path = "traits_tests.rs"]
mod tests;
