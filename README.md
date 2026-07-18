# pleiades

[![crates.io](https://img.shields.io/crates/v/pleiades-core.svg)](https://crates.io/crates/pleiades-core)
[![docs.rs](https://img.shields.io/docsrs/pleiades-core)](https://docs.rs/pleiades-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#licensing)

`pleiades` is a pure-Rust workspace for ephemeris, house, ayanamsa, and chart-building utilities aimed at astrology software.

The repository is currently a release-hardening foundation, not a finished end-user ephemeris. The main architectural pieces are in place and are intentionally split into small `pleiades-*` crates so backend implementations, domain calculations, validation tooling, and release artifacts can evolve independently.

## Current state

`pleiades` is a release-hardening foundation, not a finished end-user
ephemeris. Each surface below is guarded by a fail-closed numeric gate; measured
residuals, carve-outs, and caveats live in the linked crate docs and in the
`pleiades-core` compatibility registry, not restated here.

Release-grade numeric compatibility today: 24 house systems pass the SE numeric
gate, and 48 ayanamsas pass theirs — of 25 and 59 catalogued respectively.

| Surface | Crate | Gate | Accuracy class |
| --- | --- | --- | --- |
| Body positions / packaged artifact | `pleiades-data` | `validate-corpus` | sub-arcsecond (majors) |
| House systems | `pleiades-houses` | `validate-houses` | sub-arcsecond |
| Ayanamsas | `pleiades-ayanamsa` | `validate-ayanamsa` | sub-arcsecond |
| Sidereal time & chart angles | `pleiades-houses` | `validate-angles` | sub-arcsecond |
| Apparent place (of-date ecliptic) | `pleiades-core` | `validate-apparent` | arcsecond-class |
| Apparent equatorial (RA/Dec) | `pleiades-core` | `validate-equatorial` | sub-arcsecond |
| Civil time conversion | `pleiades-time` | (unit/property) | leap-second-exact |
| Topocentric correction | `pleiades-core` | `validate-topocentric` | opt-in correction |
| Backend frame consistency (J2000) | `pleiades-core` | `release-gate` | invariant gate |
| Eclipses (global) | `pleiades-eclipse` | `validate-eclipses` | arcsecond-class; timing seconds-of-time |
| Eclipses (local circumstances) | `pleiades-eclipse` | `validate-eclipses-local` | arcsecond-class; timing seconds-of-time |
| Longitude crossings | `pleiades-events` | `validate-crossings` | arcsecond-class |
| Rise/set/transit & horizontal | `pleiades-events` | `validate-rise-trans` | sub-arcsecond (horizontal); timing seconds-of-time |
| Fictitious bodies | `pleiades-fict` | `validate-fictitious` | definitional (sub-arcsecond) |
| Nodes & apsides | `pleiades-events` | `validate-nod-aps` | sub-arcsecond (mean) / arcminute-class (osculating) |
| Phase & magnitude | `pleiades-events` | `validate-pheno` | arcsecond-class |
| Lunar occultations | `pleiades-events` | `validate-occultations` | timing seconds-of-time; position arcminute-class |
| True (osculating) Lilith | `pleiades-apsides` | `validate-lilith` | arcminute-class |

Crate names link to their docs.rs pages; gate names to the module rustdoc that
records the measured residuals for that surface.

### Known limits

- Body/backend grades are **per-backend**: Pluto/Moon/Eros are release-grade via
  the packaged artifact; VSOP87 Pluto and the compact ELP Moon stay constrained.
  See `crates/pleiades-core/src/compatibility/mod.rs`.
- Apparent place omits gravitational light-deflection; rise/set/transit instants
  are **UT1-scale** (no ΔT model) — see `crates/pleiades-apparent` rustdoc and
  [docs/time-observer-policy.md](docs/time-observer-policy.md).
- Several surfaces carry documented, non-gated bounds (occultation planet-total
  obscuration and `central` flag; fictitious Nibiru; osculating small-body
  nodes/apsides). Each is recorded in its crate's rustdoc and in
  `crates/pleiades-core/src/compatibility/mod.rs`.
- Ingestion and kernel/corpus parsing are treated as untrusted input — see
  [docs/threat-model.md](docs/threat-model.md).
- Lunar theory selection and its limits: [docs/lunar-theory-policy.md](docs/lunar-theory-policy.md).

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
| `pleiades-fict` | Fictitious/hypothetical body backend (SP-3): SE `seorbel.txt` bodies 40–58 as unperturbed Kepler orbits, definitional parity with Swiss Ephemeris via `validate-fictitious`. |
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

## Releasing

Releases are automated with [release-plz](https://release-plz.dev). On every
push to `main`, release-plz maintains a **Release** pull request that bumps all
crates to the next unified version and updates `CHANGELOG.md` from Conventional
Commits (`feat`/`fix`/`perf`/breaking). Merge that PR to tag the version,
publish all publishable crates to crates.io, and create the GitHub Release.

### Required repository secrets

- `CARGO_REGISTRY_TOKEN` — a crates.io API token (Settings → API Tokens) with
  publish scope for the `pleiades-*` crates.
- `RELEASE_PLZ_TOKEN` — a GitHub token used by the workflow so its PRs and tags
  trigger CI. Prefer a **GitHub App** installation token (scoped, rotating);
  a fine-grained PAT with `contents: write` + `pull-requests: write` also works.
  The default `GITHUB_TOKEN` cannot trigger downstream workflows, so it is not
  sufficient here.

### Manual fallback

To cut a release by hand (e.g. if crates.io automation is unavailable), use the
retained `release.toml` config: `cargo release <version> --execute`.

## Licensing

Workspace manifests declare `MIT OR Apache-2.0`. The checked-in [`LICENSE-APACHE`](LICENSE-APACHE) and [`LICENSE-MIT`](LICENSE-MIT) files carry the full license texts.
