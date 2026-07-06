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
- a release compatibility profile (`pleiades-compatibility-profile/0.7.9`),
- an API stability profile (`pleiades-api-stability/0.2.2`),
- public sidereal-time helpers (GMST, GAST, local sidereal time via `pleiades_apparent::sidereal_time` and `SiderealTime`) and `AscMc` chart-point extras (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant via `pleiades_houses::{AscMc, chart_points, chart_points_from_armc}`); `HouseSnapshot::asc_mc` carries `AscMc` on every house snapshot; `HouseSnapshot` is now `#[non_exhaustive]`; `ChartSnapshot::asc_mc()` re-exposes `AscMc` at the façade layer; the `validate-angles` gate is wired into `run_all_numeric_gates`,
- 25 catalogued house systems and 59 catalogued ayanamsas; of the 25 catalogued house systems, 24 house systems pass the SE numeric gate; of the 59 catalogued ayanamsas, 48 release-claimed ayanamsa modes pass theirs (the remaining 11 are catalogued with metadata only),
- pure-Rust algorithmic backends for VSOP87-style planetary positions and a compact Meeus-style lunar baseline,
- a reproducible, de440-sourced JPL reference corpus (checked-in, checksum-pinned, with the kernel SHA pinned but the kernel itself not committed) behind a live fail-closed corpus gate, used for comparison and validation,
- an ARTIFACT_VERSION 7 packaged-data artifact for the 1900–2100 CE window (planets Mercury–Pluto stored heliocentrically, geocentric ecliptic reconstructed at lookup via `P_geo = P_helio + S_geo`; all bodies sub-arcsec after the SP2 heliocentric-planet reframe; motion/speed output is `Motion = Derived` via `SpeedPolicy::FittedDerivative`; published per-body-class accuracy ceilings and hard size budget ≤ 12 MB enforced; wider coverage opt-in via `generate-artifact <kernel> --out <path> [--start --end]`),
- contributor CLI tools for chart inspection, validation reports, audits, artifact checks, and release-bundle rehearsal,
- global/geocentric solar and lunar eclipse data (type, greatest-eclipse time, magnitude, gamma, Saros series, eclipsed longitude, and solar greatest-eclipse location) for 1900-01-01 … 2100-01-01 via `pleiades-eclipse`, validated exhaustively against NASA's Five Millennium Canon by the fail-closed `validate-eclipses` gate; plus, per-observer local circumstances (SP-2c) via `EclipseEngine::local_circumstances` and `next_local_eclipse`/`previous_local_eclipse` for both solar and lunar eclipses — contact times, magnitude/obscuration, on-sky azimuth/altitude, and local visibility for a given geographic observer and atmosphere, reusing the existing global eclipse walk with no new external dependency — gated by the fail-closed `validate-eclipses-local` gate (CLI aliases `eclipses-local-gate` / `eclipse-local`) over a committed 29-solar/20-lunar-row Swiss-Ephemeris corpus. Measured accuracy: solar contact/greatest-eclipse instants hold to 23.0 s for well-conditioned rows (measured max 16.1 s), widening to 95.0 s near grazing/central-limit geometry (measured max 65.0 s); lunar contacts hold to 7.0 s (measured max 5.0 s); solar magnitude/obscuration hold to 0.002 and lunar magnitude to 0.001; on-sky azimuth/altitude hold to 130.0″/120.0″ — **arcsecond-class**, not sub-arcsecond, parity.
- a longitude-crossing engine (SP-2a, distinct from the SP2 heliocentric-planet artifact reframe) via `pleiades-events`: `CrossingEngine` with `next_sun_crossing`/`next_moon_crossing` (Swiss-Ephemeris `solcross`/`mooncross` analogues), general geocentric-apparent-of-date body crossings, heliocentric `helio_cross` crossings, and a `CrossingEngine::longitude_at` evaluator over the 1900–2100 TDB window, exposed through the `validate-crossings` CLI (aliases `crossings` / `crossings-gate`) and not re-exported from `pleiades-core`; gated by a **two-tier** fail-closed `validate-crossings` gate over an 86-row committed corpus covering geocentric and heliocentric bodies Mercury–Pluto (plus Sun/Moon geocentric): Tier 1 recomputes each crossing against a committed engine golden column and holds it to a sub-second self-consistency ceiling; Tier 2 evaluates the engine's longitude at the Swiss-Ephemeris crossing time and holds it to per-body arcsecond ceilings, measured cross-theory floors (SE Moshier vs the engine's VSOP87/ELP theory) rather than a claim of tight arcsecond Swiss-Ephemeris parity — no body, including Pluto, is excluded. The corpus is checksum-guarded (fnv1a64) and pinned by row count.
- rise/set/transit and horizontal coordinates (SP-2b) via `pleiades-events`: `EventEngine::rise_trans` (a `swe_rise_trans` full-flag analogue — rise, set, upper transit, lower transit) for Sun/Moon/planets and a curated ~30-star fixed-star apparent-place catalog, plus `EventEngine::horizontal`/`EventEngine::horizontal_reverse` (`swe_azalt`/`swe_azalt_rev` analogues); `CrossingEngine` is renamed to `EventEngine` (`CrossingEngine` kept as a `#[deprecated]` alias for one release cycle). Atmospheric refraction is now implemented in `pleiades-apparent`'s `refraction` module, but it is applied **only** on this horizontal/rise-set surface — the `apparent_position` (of-date ecliptic longitude) pipeline still omits it. Gated by the fail-closed `validate-rise-trans` gate (CLI aliases `rise-trans` / `azalt` / `validate-rise-trans` / `rise-trans-gate`), wired into `run_all_numeric_gates`, over a committed Swiss-Ephemeris Moshier (`SEFLG_MOSEPH`) corpus of 50 rise-trans rows plus 20 azalt rows. Measured accuracy: horizontal coordinates (azimuth, true unrefracted altitude) agree with SE to **sub-arcsecond** (~0.1″); rise/set/transit instants agree to within a few seconds for well-conditioned rows, widening to roughly tens of seconds near grazing high-latitude/oblique-path geometry and at a below-horizon-refraction floor, with gate ceilings set from measured per-category maxima (~1.4×). **Time-scale caveat:** rise/set/transit instants are computed with sidereal time taken from the Julian Day as UT1 and no ΔT model, so the returned instants are UT1-scale (the `TimeScale::Tdb` label notwithstanding) — accurate to within ΔT (~64 s) of true TDB, not a claim of tight-TDB rise/set timing.
- fictitious/hypothetical bodies (SP-3) via `pleiades-fict`: `FictitiousBackend` computes the Swiss-Ephemeris default `seorbel.txt` fictitious body set (SE numbers 40–58 — the Uranian/Hamburg planets, Transpluto, Vulcan, White Moon, Waldemath, and the historical pre-discovery Neptune/Pluto predictions) from committed osculating orbital elements as unperturbed Kepler orbits rotated to the J2000 mean ecliptic (heliocentric-source bodies geocentricized via the packaged Sun source; White Moon and Waldemath served directly), routed into the chart backend chain. These bodies are **definitional** — correctness means parity with SE's `seorbel.txt`-driven output — gated by the fail-closed two-tier `validate-fictitious` gate (aliases `validate-fictitious`/`fictitious-gate`) over a committed 570-row Swiss-Ephemeris corpus (checksum-guarded, pinned by row count). Measured accuracy: all 18 non-Nibiru bodies hold sub-arcsecond SE parity (max longitude 0.459″); Nibiru carries a documented per-body carve-out for its ~370 AD reference equinox, which lies outside the accurate range of the IAU-1976 precession extrapolation (max longitude 1.262″).

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
