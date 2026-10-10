# Development

How to build, test and find your way around the workspace. Agent-specific
rules live in [AGENTS.md](../AGENTS.md).

## Setup and checks

Tooling is pinned with [`mise.toml`](../mise.toml):

```bash
mise install
mise run fmt
mise run lint
mise run test
```

Activate the committed pre-commit hooks (opt-in — git cannot force hooks):

```bash
git config core.hooksPath .githooks
```

Dependency and toolchain updates arrive as grouped [Renovate](https://docs.renovatebot.com)
pull requests, gated by the blocking CI tier.

Equivalent direct Cargo checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Additional useful tasks:

```bash
mise run docs
mise run audit
mise run release-smoke
mise run release-gate
```

`release-smoke` runs the native dependency audit, validates the bundled compressed artifact, stages a release bundle, and verifies the bundle. `release-gate` runs formatting, clippy, tests, benchmark generation, the full exact-aspect gate (about 15 minutes, `mise run gate-aspects`) and the full planetary-stations gate (planets, true node, and Ceres–Vesta; a few minutes, `mise run gate-stations`), then performs the same smoke checks itself; it does not run `release-smoke` as a separate step.

The command-line tools behind these tasks are described in [cli.md](cli.md).

## Workspace layout

| Crate | Role |
| --- | --- |
| `pleiades-types` | Shared typed vocabulary: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas. |
| `pleiades-backend` | Backend traits, request/result types, capability metadata, policy summaries, and routing/composite helpers. |
| `pleiades-core` | High-level chart façade, chart request validation, compatibility profile, API stability profile, and re-exports for common consumers. |
| `pleiades-houses` | House-system catalog, aliases, formula-family metadata, and baseline house calculations. |
| `pleiades-ayanamsa` | Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers. |
| `pleiades-time` | Civil-time conversion: civil UTC/UT1 calendar datetimes → TT/TDB `Instant`s and back (`from_terrestrial`, millisecond precision, leap seconds as `23:59:60`) (1900–2100, leap-second-exact UTC, observed/extrapolated Delta-T, TT↔TDB periodic term, typed `ConversionProvenance` with `exact`/`observed`/`predicted` quality marker). |
| `pleiades-apparent` | Apparent-place chart layer: applies light-time, precession-to-date, annual aberration, and nutation-in-longitude to mean J2000 backend positions to produce true equinox-of-date coordinates for every body a backend serves, whatever its claim tier (gravitational light-deflection omitted). |
| `pleiades-vsop87` | Pure-Rust VSOP87B-backed planetary backend with generated binary coefficient tables and a Meeus Table 37.A Pluto path (1885–2099). |
| `pleiades-elp` | Compact Meeus-style lunar/lunar-point backend for Moon, mean/true node, and mean apogee/perigee channels. |
| `pleiades-fict` | Fictitious/hypothetical body backend (SP-3): SE `seorbel.txt` bodies 40–58 as unperturbed Kepler orbits, definitional parity with Swiss Ephemeris via `validate-fictitious`. |
| `pleiades-apsides` | Lunar orbit points: osculating and mean nodes and apsides from the Moon's state vector and mean elements, plus the shared Kepler-elements helpers. |
| `pleiades-eclipse` | Global and per-observer local solar and lunar eclipse computation over the packaged 1900–2100 window. |
| `pleiades-events` | Event engine: longitude crossings, ecliptic position and speed, rise/set/transit and horizontal coordinates, nodes and apsides, phase and magnitude, lunar occultations. |
| `pleiades-jpl` | Reproducible de440-sourced JPL reference corpus (checksum-pinned, kernel SHA pinned, kernel not committed) and corpus-backed validation helpers behind a fail-closed gate. Also ingests external JPL-style products (Horizons vector-table / API JSON / generic CSV) into the corpus types via `pleiades-jpl::ingest`, with optional live fetch behind the default-off `horizons-fetch` feature. |
| `pleiades-compression` | Compressed artifact data structures and codec helpers. |
| `pleiades-data` | Packaged compressed-data backend and checked-in draft artifact fixture. |
| `pleiades-cli` | Contributor-facing inspection and chart CLI. |
| `pleiades-validate` | Validation reports, audits, benchmarks, artifact inspection, and release-bundle tooling. |

All first-party crates follow the `pleiades-*` naming rule required by the specification.

The layering and dependency rules are in
[spec/architecture.md](../spec/architecture.md) — start there if you are new
to the codebase.

## Specification

- [SPEC.md](../SPEC.md) — top-level specification and crate family.
- [spec/architecture.md](../spec/architecture.md) — workspace layering and dependency boundaries.
- [spec/requirements.md](../spec/requirements.md) — functional and non-functional requirements.
- [spec/api-and-ergonomics.md](../spec/api-and-ergonomics.md) — public API shape and error posture.
- [spec/validation-and-testing.md](../spec/validation-and-testing.md) — validation, benchmarking, and release gates.
- [spec/roadmap.md](../spec/roadmap.md) — implementation roadmap.

## Releasing

Releases are automated with [release-plz](https://release-plz.dev); see
[release-process.md](release-process.md) for the procedure, the required
repository secrets and the manual fallback, and
[release-reproducibility.md](release-reproducibility.md) for the release
bundle.
