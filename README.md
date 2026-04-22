# pleiades

Fast, pure Rust ephemeride utilities for astrological software.

## Current status

The repository is in the stage-6 release-hardening phase, with the chart MVP and packaged-data backend already in place:

- the Rust workspace exists and is organized around `pleiades-*` crates,
- the local developer toolchain is managed through `mise.toml`,
- the CI workflow runs formatting, linting, and tests in pure Rust mode,
- the shared type system, backend contract, and thin façade are implemented,
- the baseline house and ayanamsa catalogs plus the compatibility profile scaffold are published,
- the tropical chart workflow works end to end with approximate Sun/Moon/planet backends and a CLI chart report,
- sidereal conversion is available in the chart layer,
- house placement works for the full baseline catalog (Equal, Whole Sign, Porphyry, Placidus, Koch, Regiomontanus, Campanus, Alcabitius, Topocentric, Morinus, Meridian, and Axial variants), and the release-specific catalog now also includes Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD/SR, Sunshine, and Gauquelin sectors,
- the release compatibility profile and API stability posture are both published through `pleiades-core`, surfaced in the CLI and validation reports, and kept in sync with the current release notes,
- the compatibility catalog now includes the release-specific ayanamsa breadth for J2000, J1900, B1950, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, True Revati, True Mula, Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, Hipparchus, Babylonian (Kugler 1/2/3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Galactic Center, and Galactic Equator,
- a checked-in JPL Horizons snapshot corpus now spans multiple comparison epochs for validation, includes selected asteroid entries, and `pleiades-validate` can compare, benchmark, report on the planetary comparison subset, inspect the bundled compressed artifact, and write a reproducible release bundle with the compatibility profile, API stability posture, and validation report; the CLI chart workflow also routes selected asteroid queries through the JPL snapshot fallback at supported epochs.

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

For a release-style smoke check of the validation bundle, run:

```bash
mise run release-smoke
```

## Workspace layout

The first-party crates live under `crates/` and follow the `pleiades-*` naming rule required by the specification.
