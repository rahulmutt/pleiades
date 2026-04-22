# Backend Trait Specification

## Purpose

The backend contract defines how ephemeris engines provide positions and metadata to the rest of the system.

## Trait Responsibilities

A backend implementation must provide:

- supported bodies
- supported time range
- supported coordinate outputs
- capability flags for geocentric/topocentric/apparent/mean values
- single-query and batch-query APIs
- error reporting for unsupported bodies, out-of-range times, and data availability
- optional uncertainty metadata

## Conceptual Rust API

```rust
pub trait EphemerisBackend: Send + Sync {
    fn metadata(&self) -> BackendMetadata;

    fn supports_body(&self, body: CelestialBody) -> bool;

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError>;

    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        reqs.iter().map(|r| self.position(r)).collect()
    }
}
```

The exact API may evolve, but the semantics above are normative.

## Request Model

`EphemerisRequest` must be able to express:

- body identifier
- instant/time scale
- observer location for topocentric calculations
- desired coordinate frame
- zodiac mode (tropical/sidereal)
- ayanamsa selection when sidereal mode is chosen
- flags for apparent vs mean values where meaningful

## Result Model

`EphemerisResult` should include, where supported:

- longitude
- latitude
- radius vector / distance
- right ascension
- declination
- longitudinal speed
- latitudinal speed
- radial speed
- source backend id
- quality/uncertainty annotation

## Metadata Model

`BackendMetadata` should declare:

- backend name and version
- algorithm family
- data source provenance
- nominal time range
- body coverage
- expected accuracy class
- whether results are deterministic and offline

## Error Model

Backends must distinguish at least:

- unsupported body
- unsupported coordinate mode
- invalid observer parameters
- out-of-range instant
- missing dataset
- internal numerical failure

## Composite Backends

The trait design must permit adapters that:

- route different bodies to different backends
- select a preferred backend based on date range
- fall back from compressed data to algorithmic calculation

## Threading and Caching

Backends should be safe for concurrent reads. Internal caches are allowed, but behavior must remain deterministic and side-effect free from the caller perspective.
