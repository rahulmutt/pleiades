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

## Type Safety

The API should prefer:

- strongly typed enums for house systems, ayanamsas, bodies, and frames
- explicit newtypes or documented aliases for angles, degrees, Julian day values, and coordinates
- structured errors over stringly typed failures

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

## Determinism

For the same backend, dataset, and inputs, outputs must be deterministic across platforms within documented floating-point tolerance.

## Documentation Expectation

Every public item in the main façade crates should include rustdoc with:

- mathematical or domain meaning
- units and normalization rules
- failure conditions
- examples for common chart-generation tasks
