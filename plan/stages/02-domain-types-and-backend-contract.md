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

## Progress update

Stage 2 foundation is in place as of 2026-04-22.

- [x] `pleiades-types` now defines the angle, time, body, observer, frame, house-system, and ayanamsa primitives needed by later stages.
- [x] `pleiades-backend` now defines request/result types, backend metadata, capability flags, structured errors, and batch-query semantics.
- [x] `pleiades-core` now provides a thin façade that delegates to a backend without hiding the lower-level contract.
- [x] Toy backend coverage and rustdoc examples compile successfully under workspace tests.
- [x] The shared body taxonomy now includes explicit mean/true lunar node, apogee, and perigee identifiers, and the CLI/compression/validation layers accept the expanded lunar-point catalog.
- [x] Stage 3 algorithmic backends and chart assembly are in place, and the baseline house-system catalog is now implemented in `pleiades-houses`.

## Suggested implementation slices

1. Define angle, coordinate, and time primitives first, including normalization and unit semantics.
2. Add body identifiers, observer types, and frame enums once the low-level numerical vocabulary is stable.
3. Define backend request/result structs and structured error categories.
4. Define backend metadata and capability reporting with enough room for composite routing and future backends.
5. Add a mock or toy backend used by tests, examples, and compile-time API validation.
6. Add rustdoc examples showing a minimal position query flow through the shared contract.

Keep `pleiades-core` intentionally thin in this stage; it should prove orchestration, not hide unresolved semantics.

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
