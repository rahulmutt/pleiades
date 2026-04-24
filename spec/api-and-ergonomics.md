# API and Ergonomics

## API Goals

The public API should be easy for astrology applications to use while remaining explicit about astronomical assumptions.

## Core Usage Model

A typical caller should be able to:

1. construct or select a backend
2. query one or more body positions for a timestamp
3. convert to tropical or sidereal mode
4. compute houses for an observer location
5. assemble a chart object with derived placements

## High-Level Facade

`pleiades-core` should expose a façade that hides multi-crate orchestration while preserving access to lower-level crates for advanced users.

The façade should make a clear distinction between:

- raw backend-oriented coordinate queries
- domain-layer transforms such as sidereal conversion, house placement, and chart assembly

This keeps backend contracts simpler, avoids duplicating astrology logic across backends, and still gives end users a convenient astrology-focused API.

Chart-level observer locations are used for house calculations unless a chart API explicitly offers a topocentric body-position mode. Passing a house observer to geocentric-only backends must not silently imply topocentric positions; direct backend requests that include an observer against a geocentric-only backend should fail with a structured unsupported-observer error.

Likewise, apparent-place requests must not be silently satisfied with mean geometric coordinates. A backend that does not implement light-time, aberration, nutation, or other apparent-place corrections must either expose only mean requests through a higher-level configuration or reject direct apparent requests with a structured unsupported/invalid-request error while its capability metadata reports `apparent = false`.

## Type Safety and Extensibility

The API should prefer:

- strongly typed identifiers for house systems, ayanamsas, bodies, and frames
- explicit newtypes or documented aliases for angles, degrees, Julian day values, and coordinates
- structured errors over stringly typed failures
- identifier models that can grow as the target compatibility catalog grows without forcing breaking enum churn

If enums are used for built-in catalogs, they should be `#[non_exhaustive]` or wrapped by stable identifier types so future built-ins and aliases can be added without redesigning the public API.

## Configuration

Configuration objects should be immutable-friendly and serializable when the optional serde feature is enabled.

## Error Handling

The API must make it easy to distinguish:

- invalid input
- unsupported feature
- out-of-range date
- backend data/configuration issue
- numerical failure

## Batch Queries

The API should provide efficient batch methods for chart-style use cases, since astrology applications usually need many bodies and several derived values at once.

Batch-oriented façade APIs should let callers obtain body positions, house cusps, and the release compatibility profile relevant to the selected backend and configuration without forcing low-level crate orchestration.

## Determinism

For the same backend, dataset, and inputs, outputs must be deterministic across platforms within documented floating-point tolerance.

## Documentation Expectation

Every public item in the main façade crates should include rustdoc with:

- mathematical or domain meaning
- units and normalization rules
- failure conditions
- examples for common chart-generation tasks
