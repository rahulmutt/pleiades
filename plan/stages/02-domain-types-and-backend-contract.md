# Stage 2 — Domain Types and Backend Contract

## Goal
Define the stable semantic foundation: time, angles, bodies, coordinates, errors, requests/results, and backend metadata.

## Why this stage comes second
Before implementing algorithms or parsing datasets, the project needs a type system and trait boundary that can support multiple backends and astrology-domain layers without redesign.

## Primary deliverables

### `pleiades-types`
- angle and normalization types
- body identifiers and extensible body taxonomy
- time representations and Julian date abstractions
- observer location types
- coordinate frame enums and result structs
- shared error primitives or low-level error categories

### `pleiades-backend`
- `EphemerisBackend` trait or equivalent contract
- request/result types
- backend metadata and capability model
- batch query behavior definition
- composite-routing-friendly abstractions

### `pleiades-core`
- minimal façade traits/types that orchestrate, but do not yet hide too much

## Workable state at end of stage
A backend author can implement the contract for a toy backend, and an application author can compile code against the public types even before full astronomical functionality exists.

## Suggested tasks

1. Define angle/unit conventions, including normalization ranges.
2. Define time-scale model and document Delta T policy boundaries.
3. Define strongly typed request/result structs.
4. Define capability metadata for body coverage, date range, topocentric support, and uncertainty.
5. Add a mock backend for tests and examples.
6. Add rustdoc examples showing a minimal position query flow.

## Recommended validation

- unit tests for normalization and time conversions
- compile-time or integration tests for a mock backend
- API review against `spec/api-and-ergonomics.md` and `spec/backend-trait.md`

## Exit criteria

- no backend-specific assumptions leak into shared types
- unsupported feature and out-of-range failures are modeled explicitly
- batch APIs are specified clearly enough for chart workloads
- rustdoc examples compile

## Risks to avoid

- stringly typed enums or identifiers where stable enums/newtypes are practical
- hiding important astronomical assumptions in convenience helpers
- baking sidereal logic into backend-specific APIs unnecessarily
