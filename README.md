# pleiades

Fast, pure Rust ephemeride utilities for astrological software.

## Current status

The repository is in the stage-6 release-hardening phase, with the chart MVP and packaged-data backend already in place:

- the Rust workspace exists and is organized around `pleiades-*` crates,
- the local developer toolchain is managed through `mise.toml`,
- the CI workflow runs formatting, linting, and tests in pure Rust mode,
- the shared type system, backend contract, and thin façade are implemented, including the built-in lunar node/apogee/perigee body identifiers,
- the baseline house and ayanamsa catalogs plus the compatibility profile scaffold are published,
- the tropical chart workflow works end to end with approximate Sun/Moon/planet backends and a CLI chart report,
- sidereal conversion is available in the chart layer,
- `ChartSnapshot` now offers lookup helpers for body, sign, house, sign-scoped placement, and house-scoped placement questions plus direct/stationary/unknown-motion/retrograde motion helpers, a motion-direction filter helper (`placements_with_motion_direction`), sign summaries, house summaries, motion summaries, and aspect summaries so downstream chart reports can ask higher-level questions without re-scanning placements manually,
- house placement works for the full baseline catalog (Equal, Whole Sign, Porphyry, Placidus, Koch, Regiomontanus, Campanus, Alcabitius, Topocentric, Morinus, Meridian, and Axial variants), and the release-specific catalog now also includes Equal (MC), Equal (1=Aries), Vehlow Equal, Sripati, Carter, Horizon/Azimuth, APC, Krusinski-Pisa-Goelzer, Albategnius, Pullen SD/SR, Sunshine, and Gauquelin sectors,
- the release compatibility profile and API stability posture are both published through `pleiades-core`, surfaced in the CLI and validation reports, and kept in sync with the current release notes bundle and expanded source-label appendix, including the latest house-system labels plus the Swiss Ephemeris `Equal (cusp 1 = Asc)`, `Equal (MC)`, and `Equal (1=Aries)` spellings and J2000/J1900/B1950, B. V. Raman, B.V. Raman, B V Raman, True Citra, True Revati, True Mula, Udayagiri, Lahiri (ICRC), Lahiri (1940), DeLuce, Yukteshwar, Aryabhata 499/522, Suryasiddhanta 499, Sassanian/Zij al-Shah, and Moon sign ayanamsa source spellings,
- the CLI now also exposes the implemented backend capability matrices so maintainers can inspect body coverage, time-range notes, accuracy classes, expected error classes, and required external data files without leaving the repository,
- the compatibility catalog now includes the release-specific ayanamsa breadth for J2000, J1900, B1950, DeLuce, Yukteshwar, PVR Pushya-paksha, Sheoran, True Revati, True Mula, Udayagiri, Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, Hipparchus, Babylonian (Kugler 1/2/3), Babylonian (Huber), Babylonian (Britton), Babylonian (Eta Piscium), Babylonian (Aldebaran), Dhruva Galactic Center (Middle Mula), Galactic Center, and Galactic Equator, with the latest Mardyks/Cochrane zero-point backfills keeping the release profile synchronized,
- a checked-in JPL Horizons snapshot corpus now spans multiple comparison epochs for validation, includes selected asteroid entries, and `pleiades-validate` can compare, benchmark, report on the planetary comparison subset, render a compact house-validation corpus for baseline systems, inspect the bundled compressed artifact, and write and verify a reproducible release bundle with the compatibility profile, release notes, release checklist, backend capability matrix, API stability posture, validation report, and manifest checksums; the CLI chart workflow also routes selected asteroid queries through the JPL snapshot fallback at supported epochs.

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

For a workspace-native dependency audit, run:

```bash
mise run audit
```

For a release-style smoke check of the validation bundle, run:

```bash
mise run release-smoke
```

That smoke check runs the workspace audit, generates the bundle, and verifies its manifest checksums through `pleiades-validate`.

For a step-by-step description of the release workflow, see [docs/release-reproducibility.md](docs/release-reproducibility.md).

## Workspace layout

The first-party crates live under `crates/` and follow the `pleiades-*` naming rule required by the specification.
