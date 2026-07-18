# Validation and Testing

Unless stated otherwise, the conformance terms defined in [`SPEC.md`](../SPEC.md) apply here.

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
- **Property-based tests (proptest):** invariant oracles for pure-logic
  crates — angle normalization/idempotence and degree/radian round-trips in
  `pleiades-types`; coordinate, state, and codec re-encode round-trips in
  `pleiades-compression`. Run in the blocking tier via `mise run test`;
  discovered counterexamples are committed under each crate's
  `proptest-regressions/`.

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

### Fast default vs full test runs

`cargo test` (and `mise test`) skip tests marked `#[ignore = "slow: ..."]` —
the heavy release-bundle, benchmark/validation-report, and fit-analysis
families — giving a fast local sanity run. `mise test-full`
(`cargo test --workspace -- --include-ignored`) runs every test. CI and
`release-gate` always run `test-full`, so the gate never reduces released
coverage. The slow families are catalogued in
`docs/superpowers/plans/test-timings.md`.

## Release Gates

A release should not ship unless:

- all supported crates build in pure Rust mode on CI targets
- dependency and build audits confirm there are no mandatory C/C++ toolchain, FFI, or native runtime requirements
- backend capability docs are current
- the release compatibility profile is updated and archived with the release artifacts
- validation reports are generated and archived
- compressed artifacts pass checksum and error-threshold checks
- generated artifacts meet the reproducibility expectations in [`docs/release-reproducibility.md`](../docs/release-reproducibility.md)

## Validation Tooling

`pleiades-validate` should provide commands for:

- compare-backends
- validate-artifact
- benchmark
- generate-report
- verify-compatibility-profile
