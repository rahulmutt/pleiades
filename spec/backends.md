# Backend Implementations

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

## Backend Families

Pleiades must support multiple backend families because no single approach optimizes accuracy, size, speed, and licensing equally.

Each backend implementation must live in its own first-party `pleiades-*` crate so data sources and algorithm families remain independently versioned, testable, and optional.

## Required Initial Backends

### `pleiades-jpl`
A backend crate using public JPL ephemeris sources or publicly redistributable derivative data.

Responsibilities:

- high-confidence reference calculations
- support for validation and artifact generation
- optional support for a broad body catalog including selected asteroids when source data permits

Notes:

- must remain pure Rust at runtime and build time
- may parse official/public data files in Rust

### `pleiades-vsop87`
A pure algorithmic planetary backend for Sun/planetary barycentric or heliocentric foundations transformed for geocentric astrology use.

Responsibilities:

- broad date support without large data files
- fast computation for major planets
- documented transformation pipeline to astrology-facing coordinates

### `pleiades-elp`
A lunar backend based on a pure-Rust implementation of a documented lunar theory.

Responsibilities:

- Moon position and velocity-related quantities
- optional node/apogee derived support where mathematically justified

### `pleiades-data`
A packaged compressed backend using artifacts generated for 1600-2600.

Responsibilities:

- very fast common-range lookups
- offline distribution for applications
- predictable artifact versioning

## Optional Backend Crates

Possible future crates include:

- `pleiades-moshier`
- `pleiades-kepler`
- `pleiades-asteroids`
- `pleiades-composite`

Each must still implement the common backend contract.

## Backend Selection Policy

Consumers may choose backends based on:

- required date range
- body coverage
- accuracy needs
- bundle size constraints
- offline availability
- licensing/provenance preference

## Capability Matrix

Each backend crate must ship a documented capability matrix that states:

- supported bodies
- supported date range
- topocentric support status
- apparent/mean handling
- expected error class
- required external data files, if any

## Backend Independence Rule

No astrology-domain crate may assume a specific backend implementation. Specialized behavior must be exposed through generic capabilities or optional extension traits.

## Data Provenance and Licensing

Because the bootstrap requires backends built on **public data sources**, provenance and
redistribution terms are part of the backend contract, not an afterthought.

- Each backend crate must document, in its capability matrix, the origin of any algorithm or
  data it relies on (e.g. JPL DE series, VSOP87, an ELP-family lunar theory) and the license or
  public-domain status under which that source is used.
- No source that requires a proprietary, non-redistributable, or C/C++-linked dependency may be
  a **required** dependency of a first-party crate. This is the backend-level expression of the
  pure-Rust and no-mandatory-FFI rules in [`requirements.md`](requirements.md) NFR-1.
- Any data file a backend ships or downloads must be redistributable under a documented license,
  and that license must be recorded alongside the crate.
- Bundled artifacts in `pleiades-data` must be regenerable from public inputs and must carry
  source identity, version, and license/provenance in the artifact header defined by
  [`data-compression.md`](data-compression.md). Reproducibility expectations are governed by
  [`docs/release-reproducibility.md`](../docs/release-reproducibility.md).

These rules keep redistribution auditable and ensure the "public data sources" requirement is
verifiable per release rather than assumed.
