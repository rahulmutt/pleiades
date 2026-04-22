# Validation and Testing

## Validation Goals

Validation must demonstrate that each backend and packaged artifact is suitable for astrology-oriented production use.

## Reference Comparison

Validation should compare outputs against:

- authoritative public source datasets used by the relevant backend
- cross-backend internal consistency checks
- well-documented external reference tools where legally and practically appropriate

## Test Categories

### Unit Tests

- angle normalization
- time conversions
- house computations
- ayanamsa conversions
- backend capability reporting

### Property Tests

- normalization invariants
- monotonicity or continuity expectations within bounded windows
- compression encode/decode roundtrip behavior

### Golden Tests

- published sample charts
- known body positions at canonical epochs
- cusp outputs for selected locations and dates

### Regression Tests

- previously fixed numerical edge cases
- polar/high-latitude house failures and expected behavior
- boundary dates around segment edges in compressed data

## Benchmarking

Benchmarks should measure:

- single-position latency
- full-chart latency
- batch throughput
- compressed-data decode cost
- memory footprint of common workloads

## Release Gates

A release should not ship unless:

- all supported crates build in pure Rust mode on CI targets
- backend capability docs are current
- validation reports are generated and archived
- compressed artifacts pass checksum and error-threshold checks

## Validation Tooling

`pleiades-validate` should provide commands for:

- compare-backends
- validate-artifact
- benchmark
- generate-report
