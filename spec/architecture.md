# Architecture

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

## Workspace Structure

The project is a Rust workspace composed of small, focused crates with explicit layering and clean dependency direction.

## Mandatory Crates

### Core and Types

- `pleiades-types`: shared enums, structs, identifiers, coordinate types, time types, errors, and compatibility-profile types
- `pleiades-backend`: backend trait definitions, capability metadata, and adapter helpers
- `pleiades-core`: high-level façade API that composes backends with domain crates

### Domain Calculation Crates

- `pleiades-houses`: house-system implementations and related domain helpers
- `pleiades-ayanamsa`: ayanamsa catalog and sidereal conversion logic

### Artifact and Data Support Crates

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
7. `pleiades-core` may depend on domain crates, `pleiades-backend`, and backend-neutral support crates, but must not require one specific backend by default.
8. `pleiades-data` may depend on `pleiades-types`, `pleiades-compression`, and `pleiades-backend`.
9. Tooling crates may depend on all first-party crates.

## Architectural Layers

These layers define dependency direction, not a strict execution pipeline.

### Layer 1: Foundation

- `pleiades-types`
- primitive units, identifiers, shared errors, and compatibility-profile data

### Layer 2: Contracts and Formats

- `pleiades-backend`
- `pleiades-compression`

Both crates depend only on `pleiades-types`. They define contracts and file formats, not astrology-domain behavior.

### Layer 3: Implementations

This layer has two parallel families that must not depend on each other directly:

- **Domain implementations**: `pleiades-houses`, `pleiades-ayanamsa`
- **Backend implementations**: `pleiades-jpl`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-data`, and future backend crates

`pleiades-data` belongs in the backend family even though it uses compressed artifacts. It is still a backend implementation, not a higher-level domain layer.

### Layer 4: Composition API

- `pleiades-core`

This façade composes backend results with domain logic and presents the main user-facing Rust API.

### Layer 5: Tooling

- `pleiades-cli`
- `pleiades-validate`
- future artifact-build or report-generation crates

## Global Layering Rules

- No crate may depend on a higher layer.
- Domain crates must not import or special-case concrete backend crates.
- Backend crates must not embed house, ayanamsa, or chart assembly logic as required behavior.
- Shared behavior needed by multiple layers must move downward into an appropriate lower crate rather than sideways across peer crates.

## Backend Composition Strategy

The architecture must allow hybrid composition. For example:

- planets via `pleiades-vsop87`
- Moon via `pleiades-elp`
- asteroids via `pleiades-jpl`
- common-range lookups via `pleiades-data`

A composite adapter may route body queries to different providers while presenting one unified backend implementation.

Astrology-specific transforms such as sidereal conversion, house placement, and chart assembly must remain above the source-specific backend layer unless a backend explicitly exposes an equivalent capability through the common contract.

If a domain algorithm needs shared astronomical input, that input must be expressed through backend-neutral types defined in `pleiades-types` and supplied by `pleiades-core` or a caller.

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
