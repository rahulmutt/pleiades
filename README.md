# pleiades

`pleiades` is a pure-Rust workspace for ephemeris, house, ayanamsa, and chart-building utilities aimed at astrology software.

The repository is currently a release-hardening foundation, not a finished end-user ephemeris. The main architectural pieces are in place and are intentionally split into small `pleiades-*` crates so backend implementations, domain calculations, validation tooling, and release artifacts can evolve independently.

## Current state

As of the current workspace state, `pleiades` includes:

- a backend-agnostic request/result contract with capability metadata,
- a high-level chart façade with typed tropical/sidereal chart requests,
- baseline chart body placement, zodiac-sign summaries, aspect summaries, and optional house summaries,
- a release compatibility profile (`pleiades-compatibility-profile/0.6.123`),
- an API stability profile (`pleiades-api-stability/0.1.0`),
- 25 catalogued house systems and 59 catalogued ayanamsas,
- pure-Rust algorithmic backends for VSOP87-style planetary positions and a compact Meeus-style lunar baseline,
- checked-in JPL Horizons reference snapshots used for comparison and validation,
- a stage-5 draft packaged-data artifact for the common 1500-2500 CE range,
- contributor CLI tools for chart inspection, validation reports, audits, artifact checks, and release-bundle rehearsal.

Important current limits:

- first-party backends currently expose **mean geometric** coordinates; apparent-place corrections are rejected unless a backend advertises support,
- direct backend requests accept TT/TDB; UTC/UT1 require caller-supplied conversion offsets,
- body-position observer/topocentric requests remain unsupported by current first-party backends,
- native sidereal backend output is not assumed; chart-level sidereal longitude is handled by the façade/catalog layer,
- the packaged-data artifact is a draft reproducibility fixture and should not be treated as final production-accuracy compressed ephemeris data.

For the source-of-truth design and compatibility targets, read [SPEC.md](SPEC.md) and the documents in [`spec/`](spec/).

## Workspace layout

| Crate | Role |
| --- | --- |
| `pleiades-types` | Shared typed vocabulary: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas. |
| `pleiades-backend` | Backend traits, request/result types, capability metadata, policy summaries, and routing/composite helpers. |
| `pleiades-core` | High-level chart façade, chart request validation, compatibility profile, API stability profile, and re-exports for common consumers. |
| `pleiades-houses` | House-system catalog, aliases, formula-family metadata, and baseline house calculations. |
| `pleiades-ayanamsa` | Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers. |
| `pleiades-vsop87` | Pure-Rust VSOP87B-backed planetary backend with generated binary coefficient tables and a Pluto fallback. |
| `pleiades-elp` | Compact Meeus-style lunar/lunar-point backend for Moon, mean/true node, and mean apogee/perigee channels. |
| `pleiades-jpl` | Checked-in JPL Horizons reference snapshots and snapshot-backed validation helpers. |
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
- UTC/UT1 convenience is explicit: use the `--tt-*` or `--tdb-*` offset flags when converting from UTC/UT1 at the CLI boundary. See [docs/time-observer-policy.md](docs/time-observer-policy.md).

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

Workspace manifests declare `MIT OR Apache-2.0`. The checked-in [LICENSE](LICENSE) file contains the Apache-2.0 text.
