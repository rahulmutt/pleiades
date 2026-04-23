# Architecture

## Workspace Structure

The project is a Rust workspace composed of small, focused crates with explicit layering.

## Mandatory Crates

### Core and Types

- `pleiades-types`: shared enums, structs, identifiers, coordinate types, time types, errors, and compatibility-profile types
- `pleiades-backend`: backend trait definitions, capability metadata, and adapter helpers
- `pleiades-core`: high-level façade API that composes backends with domain crates

### Domain Calculation Crates

- `pleiades-houses`: house-system implementations and related domain helpers
- `pleiades-ayanamsa`: ayanamsa catalog and sidereal conversion logic

### Artifact and Data Crates

- `pleiades-compression`: compression codecs and artifact packing/unpacking logic

### Backend Crates

- `pleiades-jpl`: backend reading public JPL ephemeris sources or derivative public data products
- `pleiades-vsop87`: planetary algorithm backend based on a pure-Rust VSOP87 implementation or data source
- `pleiades-elp`: lunar backend based on a documented pure-Rust lunar theory implementation
- `pleiades-data`: packaged compressed ephemeris backend for 1500-2500 CE

### Tooling Crates

- `pleiades-cli`: inspection, chart query, conversion, and data-build commands
- `pleiades-validate`: cross-backend comparison, benchmarking, and regression tooling

Optional crates such as `pleiades-composite` may be added later if they respect the same dependency rules.

## Dependency Rules

1. Every first-party workspace crate must be named with the `pleiades-*` prefix.
2. `pleiades-types` must not depend on backend crates.
3. `pleiades-backend` may depend on `pleiades-types` only.
4. Source-specific backend crates may depend on `pleiades-types` and `pleiades-backend`.
5. Domain crates such as `pleiades-houses` and `pleiades-ayanamsa` must remain backend-agnostic and may depend on `pleiades-types` only.
6. `pleiades-compression` may depend on `pleiades-types` only.
7. `pleiades-core` may depend on all domain crates and on `pleiades-backend`, but must not require one specific backend by default.
8. `pleiades-data` may depend on `pleiades-types`, `pleiades-compression`, and `pleiades-backend`.
9. Tooling crates may depend on all first-party crates.

## Layering

- **Layer 1**: primitive types, units, identifiers, and shared errors
- **Layer 2**: backend contracts and capability metadata
- **Layer 3**: algorithmic and reference-data backend implementations
- **Layer 4**: astrology-domain computation crates
- **Layer 5**: artifact codecs and packaged-data backends
- **Layer 6**: high-level façade and composition APIs
- **Layer 7**: CLI, validation, artifact generation, and reporting tools

No layer may depend on a higher layer.

## Backend Composition Strategy

The architecture must allow hybrid composition. For example:

- planets via `pleiades-vsop87`
- Moon via `pleiades-elp`
- asteroids via `pleiades-jpl`
- common-range lookups via `pleiades-data`

A composite adapter may route body queries to different providers while presenting one unified backend implementation.

Astrology-specific transforms such as sidereal conversion, house placement, and chart assembly must remain above the source-specific backend layer unless a backend explicitly exposes an equivalent capability through the common contract.

Domain crates must not import or special-case individual backend implementations. If a domain algorithm needs shared astronomical input, that input must be expressed through backend-neutral types defined in `pleiades-types` and supplied by `pleiades-core` or a caller.

## Feature Flags

Crates should use Cargo features for optional heavy capabilities, such as:

- large public datasets
- optional serde support
- benchmark or validation instrumentation
- no_std-compatible subsets where feasible

## Data Separation

The system must keep the following concerns separate:

- algorithm source code
- raw imported source data
- normalized intermediate products
- compressed distributable artifacts

This separation preserves reproducibility, licensing clarity, and clean runtime boundaries.
