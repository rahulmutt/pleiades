# pleiades

Fast, pure Rust ephemeride utilities for astrological software.

## Current status

The repository is in the stage-4 validation phase, with the chart MVP already in place:

- the Rust workspace exists and is organized around `pleiades-*` crates,
- the local developer toolchain is managed through `mise.toml`,
- the CI workflow runs formatting, linting, and tests in pure Rust mode,
- the shared type system, backend contract, and thin façade are implemented,
- the baseline house and ayanamsa catalogs plus the compatibility profile scaffold are published,
- the tropical chart workflow works end to end with approximate Sun/Moon/planet backends and a CLI chart report,
- sidereal conversion is available in the chart layer,
- house placement works for the full baseline catalog (Equal, Whole Sign, Porphyry, Placidus, Koch, Regiomontanus, Campanus, Alcabitius, Topocentric, Morinus, Meridian, and Axial variants),
- a narrow JPL Horizons snapshot backend is now available for validation, and `pleiades-validate` can compare, benchmark, and report on that snapshot corpus.

## Local development

Install the pinned Rust toolchain and run the standard checks with `mise`:

```bash
mise install
mise run fmt
mise run lint
mise run test
```

The equivalent direct Cargo commands are:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Workspace layout

The first-party crates live under `crates/` and follow the `pleiades-*` naming rule required by the specification.
