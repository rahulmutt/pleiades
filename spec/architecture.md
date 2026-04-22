# Architecture

## Workspace Structure

The project is a Rust workspace composed of small focused crates.

## Mandatory Crates

### Core and Types

- `pleiades-types`: shared enums, structs, identifiers, coordinate types, time types, errors
- `pleiades-backend`: backend trait definitions, capability metadata, adapter helpers
- `pleiades-core`: high-level façade API combining backend queries with domain calculations

### Domain Calculation Crates

- `pleiades-houses`: house system implementations
- `pleiades-ayanamsa`: ayanamsa catalog and conversion logic
- `pleiades-compression`: compression codecs and data packing/unpacking logic

### Backend Crates

- `pleiades-jpl`: backend reading public JPL ephemeris sources or derivative public data products
- `pleiades-vsop87`: planetary algorithm backend based on pure-Rust VSOP87 implementation/data
- `pleiades-elp`: Moon backend based on a documented pure-Rust lunar theory implementation
- `pleiades-data`: packaged compressed ephemeris backend for 1500-2500

### Tooling Crates

- `pleiades-cli`: inspection, chart query, conversion, and data build commands
- `pleiades-validate`: cross-backend comparison, benchmark, and regression tools

## Dependency Rules

1. `pleiades-types` must not depend on backend crates.
2. `pleiades-backend` may depend on `pleiades-types` only.
3. Backend crates may depend on `pleiades-types` and `pleiades-backend`.
4. `pleiades-core` may depend on all domain crates and on backend traits, but should not require a specific backend by default.
5. `pleiades-data` may depend on `pleiades-compression` and the backend trait crate.
6. Tooling crates may depend on all first-party crates.

## Layering

- **Layer 1**: primitive types and errors
- **Layer 2**: backend contracts
- **Layer 3**: source-specific backend crates
- **Layer 4**: astrology-domain computations and façade API
- **Layer 5**: CLI, validators, and data builders

## Backend Composition Strategy

The architecture must allow hybrid composition. For example:

- planets via `pleiades-vsop87`
- Moon via `pleiades-elp`
- asteroids via `pleiades-jpl`
- prepacked common range via `pleiades-data`

A composite backend adapter may route body queries to different underlying providers while presenting one unified backend implementation.

## Feature Flags

Crates should use Cargo features for optional heavy capabilities, such as:

- large public datasets
- serde support
- no_std-compatible subsets where feasible
- benchmark/validation instrumentation

## Data Separation

The system must separate:

- algorithm source code
- raw imported source data
- normalized intermediate products
- compressed distributable artifacts

This separation ensures regeneration and licensing clarity.
