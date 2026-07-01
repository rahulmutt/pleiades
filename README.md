# pleiades

[![crates.io](https://img.shields.io/crates/v/pleiades-core.svg)](https://crates.io/crates/pleiades-core)
[![docs.rs](https://img.shields.io/docsrs/pleiades-core)](https://docs.rs/pleiades-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#licensing)

`pleiades` is a pure-Rust workspace for ephemeris, house, ayanamsa, and chart-building utilities aimed at astrology software.

The repository is currently a release-hardening foundation, not a finished end-user ephemeris. The main architectural pieces are in place and are intentionally split into small `pleiades-*` crates so backend implementations, domain calculations, validation tooling, and release artifacts can evolve independently.

## Current state

As of the current workspace state, `pleiades` includes:

- a backend-agnostic request/result contract with capability metadata,
- a high-level chart façade with typed tropical/sidereal chart requests,
- baseline chart body placement, zodiac-sign summaries, aspect summaries, and optional house summaries,
- a release compatibility profile (`pleiades-compatibility-profile/0.7.2`),
- an API stability profile (`pleiades-api-stability/0.2.1`),
- public sidereal-time helpers (GMST, GAST, local sidereal time via `pleiades_apparent::sidereal_time` and `SiderealTime`) and `AscMc` chart-point extras (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant via `pleiades_houses::{AscMc, chart_points, chart_points_from_armc}`); `HouseSnapshot::asc_mc` carries `AscMc` on every house snapshot; `HouseSnapshot` is now `#[non_exhaustive]`; `ChartSnapshot::asc_mc()` re-exposes `AscMc` at the façade layer; the `validate-angles` gate is wired into `run_all_numeric_gates`,
- 25 catalogued house systems and 59 catalogued ayanamsas; of the 25 catalogued house systems, 24 house systems pass the SE numeric gate; of the 59 catalogued ayanamsas, 48 release-claimed ayanamsa modes pass theirs (the remaining 11 are catalogued with metadata only),
- pure-Rust algorithmic backends for VSOP87-style planetary positions and a compact Meeus-style lunar baseline,
- a reproducible, de440-sourced JPL reference corpus (checked-in, checksum-pinned, with the kernel SHA pinned but the kernel itself not committed) behind a live fail-closed corpus gate, used for comparison and validation,
- an ARTIFACT_VERSION 7 packaged-data artifact for the 1900–2100 CE window (planets Mercury–Pluto stored heliocentrically, geocentric ecliptic reconstructed at lookup via `P_geo = P_helio + S_geo`; all bodies sub-arcsec after the SP2 heliocentric-planet reframe; motion/speed output is `Motion = Derived` via `SpeedPolicy::FittedDerivative`; published per-body-class accuracy ceilings and hard size budget ≤ 12 MB enforced; wider coverage opt-in via `generate-artifact <kernel> --out <path> [--start --end]`),
- contributor CLI tools for chart inspection, validation reports, audits, artifact checks, and release-bundle rehearsal,
- global/geocentric solar and lunar eclipse data (type, greatest-eclipse time, magnitude, gamma, Saros series, eclipsed longitude, and solar greatest-eclipse location) for 1900-01-01 … 2100-01-01 via `pleiades-eclipse`, validated exhaustively against NASA's Five Millennium Canon by the fail-closed `validate-eclipses` gate; local (per-observer) circumstances are not provided.

Important current limits:

- body/backend claims are now **per-backend**: Pluto, the Moon, and Eros are release-grade via the packaged-data artifact, while VSOP87's Pluto stays approximate and the compact ELP Moon stays constrained; the thirty-six Tier-A asteroids/TNOs (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris, Eunomia, Cybele, Astraea, Hebe, Flora, Metis, Fortuna, Sappho, Eros, plus TNOs Eris, Sedna, Haumea, Makemake, Quaoar, Orcus, Gonggong, Varuna, Ixion, plus centaurs Chiron, Pholus, Nessus, Chariklo, Asbolus, and personal/NEA asteroids Amor, Lilith, Hidalgo, Icarus, Toro, Apollo) are release-grade via the corpus-dependent JPL/SPK backend (source: sb441-n373s perturber kernel + per-object JPL SPKs for centaurs/NEA); the Tier-B constrained asteroid slice is now empty (all former Tier-B bodies promoted via per-object SPK); the osculating true apogee/perigee (True Lilith) are now **release-grade** via the packaged-Moon-derived backend (`crates/pleiades-data` osculating path + `crates/pleiades-apsides`), gated against Swiss Ephemeris `SE_OSCU_APOG` by `validate-lilith` (3177 samples, 1900–2100, max longitude residual ~306″).
- apparent place of date is the **default chart-layer output** for release-grade bodies (light-time + precession-to-date + annual aberration + nutation-in-longitude; gravitational light-deflection omitted); chart-layer body positions now also carry **apparent equatorial of date** (RA/Dec, true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies, gated by `validate-equatorial` (JPL Horizons) and `validate-equatorial-se` (Swiss Ephemeris parity); first-party backends remain mean and J2000 ecliptic at the backend boundary (both longitude and latitude components, consistently J2000 across all first-party backends),
- `pleiades-time` now provides built-in civil UTC/UT1 → TT/TDB conversion (leap-second-exact UTC from 1972, observed/extrapolated Delta-T for UT1, TT↔TDB periodic term, 1900–2100 window, tiered `exact`/`observed`/`predicted` quality marker); direct backends still consume TT/TDB, and the caller-supplied `--tt-*`/`--tdb-*` offset flags remain the lower-level alternative,
- chart-layer topocentric body positions are supported as an opt-in correction (diurnal parallax + diurnal aberration); native-backend topocentric remains unsupported,
- native sidereal backend output is not assumed; chart-level sidereal longitude is handled by the façade/catalog layer,
- the packaged-data artifact has sub-arcsec accuracy across all major bodies (SP2 heliocentric-planet reframe); published accuracy thresholds and hard size budget are enforced (SP3 complete, 1900–2100 CE window); motion/speed output is derived (`SpeedPolicy::FittedDerivative`); latency is tracked but not hard-gated by default.

## Published crates

The eleven library crates (`pleiades-types`, `pleiades-backend`, `pleiades-core`,
`pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`, `pleiades-elp`,
`pleiades-jpl`, `pleiades-compression`, `pleiades-time`, `pleiades-apparent`) are published to crates.io as
experimental `0.2.x` releases under `MIT OR Apache-2.0`. The limits above apply
to the published crates as well; production-accuracy claims wait on the phases
in [PLAN.md](PLAN.md). `pleiades-cli`, `pleiades-data`, and `pleiades-validate`
are contributor tooling and stay unpublished. The release procedure is
documented in [docs/release-process.md](docs/release-process.md).

For the source-of-truth design and compatibility targets, read [SPEC.md](SPEC.md) and the documents in [`spec/`](spec/).

## Workspace layout

| Crate | Role |
| --- | --- |
| `pleiades-types` | Shared typed vocabulary: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas. |
| `pleiades-backend` | Backend traits, request/result types, capability metadata, policy summaries, and routing/composite helpers. |
| `pleiades-core` | High-level chart façade, chart request validation, compatibility profile, API stability profile, and re-exports for common consumers. |
| `pleiades-houses` | House-system catalog, aliases, formula-family metadata, and baseline house calculations. |
| `pleiades-ayanamsa` | Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers. |
| `pleiades-time` | Civil-time conversion: civil UTC/UT1 calendar datetimes → TT/TDB `Instant`s (1900–2100, leap-second-exact UTC, observed/extrapolated Delta-T, TT↔TDB periodic term, typed `ConversionProvenance` with `exact`/`observed`/`predicted` quality marker). |
| `pleiades-apparent` | Apparent-place chart layer: applies light-time, precession-to-date, annual aberration, and nutation-in-longitude to mean J2000 backend positions to produce true equinox-of-date coordinates for release-grade bodies (gravitational light-deflection omitted). |
| `pleiades-vsop87` | Pure-Rust VSOP87B-backed planetary backend with generated binary coefficient tables and a Pluto approximate path. |
| `pleiades-elp` | Compact Meeus-style lunar/lunar-point backend for Moon, mean/true node, and mean apogee/perigee channels. |
| `pleiades-jpl` | Reproducible de440-sourced JPL reference corpus (checksum-pinned, kernel SHA pinned, kernel not committed) and corpus-backed validation helpers behind a fail-closed gate. Also ingests external JPL-style products (Horizons vector-table / API JSON / generic CSV) into the corpus types via `pleiades-jpl::ingest`, with optional live fetch behind the default-off `horizons-fetch` feature. |
| `pleiades-compression` | Compressed artifact data structures and codec helpers. |
| `pleiades-data` | Packaged compressed-data backend and checked-in draft artifact fixture. |
| `pleiades-cli` | Contributor-facing inspection and chart CLI. |
| `pleiades-validate` | Validation reports, audits, benchmarks, artifact inspection, and release-bundle tooling. |

All first-party crates follow the `pleiades-*` naming rule required by the specification.

## CLI quick start

Run contributor commands through Cargo:

```bash
cargo run -q -p pleiades-cli -- help
cargo run -q -p pleiades-validate -- help
```

Useful inspection commands:

```bash
# One-screen release posture
cargo run -q -p pleiades-cli -- release-summary

# Current compatibility catalog/profile
cargo run -q -p pleiades-cli -- profile-summary

# Backend capability matrix
cargo run -q -p pleiades-cli -- backend-matrix-summary

# Request semantics and time/observer policy
cargo run -q -p pleiades-cli -- request-surface-summary
cargo run -q -p pleiades-cli -- utc-convenience-policy-summary

# Packaged artifact posture
cargo run -q -p pleiades-cli -- artifact-summary
```

Render a basic chart report:

```bash
cargo run -q -p pleiades-cli -- chart \
  --jd 2451545.0 \
  --body Sun \
  --body Moon
```

Render a sidereal chart with houses for an observer:

```bash
cargo run -q -p pleiades-cli -- chart \
  --jd 2451545.0 \
  --lat 51.5074 \
  --lon 0.0 \
  --ayanamsa Lahiri \
  --house-system "Whole Sign" \
  --body Sun \
  --body Moon \
  --mean
```

Notes:

- `chart` defaults to `JD 2451545.0` if `--jd` is omitted.
- If no `--body` flags are given, the CLI uses the default chart body set from `pleiades-core`.
- `--body` accepts built-in labels such as `Sun`, `Moon`, and `Ceres`, plus custom identifiers such as `asteroid:433-Eros` when supported by the selected path.
- `--ayanamsa` accepts built-in names such as `Lahiri` and custom definitions such as `custom:True Balarama|2451545.0|12.5`.
- Built-in civil-time conversion: use `--civil <YYYY-MM-DDTHH:MM:SS> [--civil-scale utc|ut1] [--civil-target tt|tdb]` to convert a calendar datetime to TT/TDB automatically (1900–2100, tiered quality). Alternatively, supply caller-chosen offsets via the `--tt-*` or `--tdb-*` flags. See [docs/time-observer-policy.md](docs/time-observer-policy.md).

## Validation and release tooling

`pleiades-validate` is the maintainer tool for audits, reports, artifact inspection, and release rehearsal:

```bash
# Native dependency / build-hook audit
cargo run -q -p pleiades-validate -- workspace-audit

# Compatibility profile verification
cargo run -q -p pleiades-validate -- verify-compatibility-profile

# Full packaged-artifact inspection
cargo run -q -p pleiades-validate -- validate-artifact

# Compact validation report
cargo run -q -p pleiades-validate -- report-summary --rounds 100

# Stage and verify a release bundle
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release
```

For release reproducibility details, see [docs/release-reproducibility.md](docs/release-reproducibility.md).

## Local development

Tooling is pinned with [`mise.toml`](mise.toml):

```bash
mise install
mise run fmt
mise run lint
mise run test
```

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

`release-smoke` runs the native dependency audit, validates the bundled compressed artifact, stages a release bundle, and verifies the bundle. `release-gate` also runs formatting, clippy, tests, and benchmark generation before invoking the smoke path.

## Documentation map

- [SPEC.md](SPEC.md) — top-level specification and crate family.
- [spec/architecture.md](spec/architecture.md) — workspace layering and dependency boundaries.
- [spec/requirements.md](spec/requirements.md) — functional and non-functional requirements.
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md) — public API shape and error posture.
- [spec/validation-and-testing.md](spec/validation-and-testing.md) — validation, benchmarking, and release gates.
- [spec/roadmap.md](spec/roadmap.md) — implementation roadmap.
- [docs/time-observer-policy.md](docs/time-observer-policy.md) — current time-scale, observer, apparentness, and frame policy.
- [docs/lunar-theory-policy.md](docs/lunar-theory-policy.md) — current lunar theory selection and limitations.
- [docs/release-reproducibility.md](docs/release-reproducibility.md) — release bundle and artifact reproducibility workflow.

## Licensing

Workspace manifests declare `MIT OR Apache-2.0`. The checked-in [`LICENSE-APACHE`](LICENSE-APACHE) and [`LICENSE-MIT`](LICENSE-MIT) files carry the full license texts.
