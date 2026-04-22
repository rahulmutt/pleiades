# Backend Implementations

## Backend Families

Pleiades must support multiple backend families because no single approach optimizes accuracy, size, speed, and licensing equally.

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
A packaged compressed backend using artifacts generated for 1500-2500.

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
