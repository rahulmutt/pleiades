# Backend Trait Specification

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

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

    // Provided. The place is bit-identical to `position`'s and `motion` is
    // `None`; it may succeed where `position` fails only because the motion
    // could not be computed. Backends whose motion costs extra evaluations
    // override it; mean-place-only callers (apparent-place light-time
    // re-queries, event and eclipse searches) use it.
    fn position_without_motion(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        let mut result = self.position(req)?;
        result.motion = None;
        Ok(result)
    }

    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        reqs.iter().map(|r| self.position(r)).collect()
    }
}
```

The exact API may evolve, but the semantics above are normative: one-request/one-result querying is mandatory, and batch querying must be supported either as an all-or-error vector API like the sketch above or as a documented per-item-result variant with equivalent capability.

## Request Model

`EphemerisRequest` must be able to express:

- body identifier
- instant/time scale
- observer location for topocentric calculations
- desired coordinate frame
- The `equatorial` channel's frame is part of each backend's documented
  contract. First-party backends give the J2000 mean equator and equinox
  (the J2000 ecliptic rotated by the J2000 mean obliquity), except the ELP
  lunar backend, which gives right ascension and declination of date. Chart
  assembly does not pass the channel through for mean placements (#210).
- flags for apparent vs mean values where meaningful

The canonical backend contract is for raw astronomical outputs. High-level APIs may accept tropical/sidereal preferences and ayanamsa choices, but sidereal conversion should normally happen in the domain layer from tropical coordinates plus the selected ayanamsa. If a backend offers native sidereal output anyway, capability metadata must make that distinction explicit.

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
- enough metadata to let higher layers apply deterministic zodiac and house logic without backend-specific special cases

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
- normalize backend-specific details into the common result model used by higher layers

## Threading and Caching

Backends should be safe for concurrent reads. Internal caches are allowed, but behavior must remain deterministic and side-effect free from the caller perspective.
