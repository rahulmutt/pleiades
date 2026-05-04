# pleiades

`pleiades` is a pure Rust workspace for ephemeris and chart-building utilities aimed at astrological software.

The project is being built with a strong focus on correctness, reproducibility, and long-term extensibility. If you are evaluating it today, think of it as an actively maturing foundation rather than a finished end-user product.

## What it is

Today the workspace provides:

- a modular set of `pleiades-*` crates,
- a high-level chart façade,
- support for tropical and sidereal chart workflows,
- a growing catalog of house systems and ayanamsas,
- CLI tooling for validation, inspection, and reporting,
- reproducible pure-Rust development and release workflows.

## Project status

`pleiades` is currently in a release-hardening phase.

That means the core workspace structure, chart façade, packaged-data backend, validation pipeline, and release tooling are already in place, while coverage, polish, and compatibility breadth continue to improve.

If you want the detailed implementation and planning view, see:

- roadmap: [spec/roadmap.md](spec/roadmap.md)
- architecture: [spec/architecture.md](spec/architecture.md)
- requirements: [spec/requirements.md](spec/requirements.md)

The roadmap is the best place to look for what is done, what is being stabilized, and what is still ahead.

## CLI tools

The workspace currently ships two repo-focused CLI binaries:

- `pleiades-cli`: quick inspection and chart-oriented commands for contributors
- `pleiades-validate`: validation, benchmarking, audit, and release-bundle tooling

You can run them with `cargo run -p ... -- <command>` during development.

### `pleiades-cli`

Use `pleiades-cli` for lightweight inspection and chart reporting:

```bash
cargo run -q -p pleiades-cli -- help
```

Rough command overview:

- `compare-backends` / `compare-backends-audit`: compare the checked-in JPL snapshot against the algorithmic composite backend and fail the tolerance audit on regressions
- `compatibility-profile` / `profile`: print the full release compatibility profile
- `compatibility-profile-summary` / `profile-summary`: compact compatibility summary
- `catalog-inventory-summary` / `catalog-inventory`: compact compatibility catalog inventory, including house formula-family coverage
- `house-validation-summary` / `house-formula-families-summary` / `house-code-aliases-summary` / `ayanamsa-catalog-validation-summary` / `ayanamsa-metadata-coverage-summary` / `ayanamsa-reference-offsets-summary`: compact house and ayanamsa catalog audits
- `api-stability` / `api-posture`: print the API stability posture
- `api-stability-summary` / `api-posture-summary`: compact API stability summary
- `backend-matrix` / `capability-matrix`: print implemented backend capability matrices
- `backend-matrix-summary` / `matrix-summary`: compact backend matrix summary
- `benchmark [--rounds N]`: benchmark the candidate backend on the representative corpus and full chart assembly
- `release-notes`, `release-notes-summary`, `release-checklist`, `release-checklist-summary` / `checklist-summary`, `release-summary`, `request-policy-summary` / `request-policy` / `request-semantics-summary` / `request-semantics`, `time-scale-policy-summary`, `delta-t-policy-summary`, `request-surface-summary` / `request-surface`, `catalog-inventory-summary` / `catalog-inventory`, `house-validation-summary`, `house-formula-families-summary`, `house-code-aliases-summary`, `ayanamsa-catalog-validation-summary`, `ayanamsa-metadata-coverage-summary`, `ayanamsa-reference-offsets-summary`, `lunar-theory-catalog-summary`, `lunar-theory-catalog-validation-summary`, `packaged-lookup-epoch-policy-summary`, `verify-compatibility-profile`, `verify-release-bundle`: maintainer-facing release metadata and catalog-alignment checks
- `comparison-corpus-summary` / `comparison-corpus-release-guard-summary` / `comparison-envelope-summary` / `reference-holdout-overlap-summary` / `independent-holdout-summary`: compact release-grade comparison, hold-out, and validation-overlap summaries
- `reference-snapshot-1749-major-body-boundary-summary` / `1749-major-body-boundary-summary`, `reference-snapshot-early-major-body-boundary-summary` / `early-major-body-boundary-summary`, `reference-snapshot-1800-major-body-boundary-summary` / `1800-major-body-boundary-summary`, `reference-snapshot-2500-major-body-boundary-summary` / `2500-major-body-boundary-summary`, `reference-snapshot-selected-asteroid-terminal-boundary-summary` / `selected-asteroid-terminal-boundary-summary`: compact reference boundary archaeology summaries
- `comparison-snapshot-manifest-summary`, `reference-snapshot-manifest-summary`: compact source-manifest provenance summaries for the checked-in JPL snapshots (also available through `pleiades-cli`)
- `artifact-summary` / `artifact-posture-summary`: compact summary of the packaged compressed artifact
- `validate-artifact`: full packaged-artifact inspection mirrored from `pleiades-validate`
- `workspace-audit` / `audit` / `native-dependency-audit`: workspace-native dependency audit mirrored from `pleiades-validate`
- `report` / `generate-report`: full validation report mirrored from `pleiades-validate`
- `validation-report-summary` / `validation-summary` / `report-summary`: compact validation report summary
- `frame-policy-summary`: compact frame-policy summary
- `observer-policy-summary`: compact observer policy summary
- `apparentness-policy-summary`: compact apparentness policy summary
- `source-documentation-summary` / `source-documentation-health-summary` / `source-documentation-health`: compact VSOP87 source-documentation provenance and health summaries
- `production-generation-boundary-summary`: compact production-generation boundary overlay summary
- `request-policy-summary` / `request-policy` / `request-semantics-summary` / `request-semantics`: compact request-policy summary
- `request-surface-summary`: compact request-surface inventory, including the explicit chart TT/TDB offset aliases
- `chart`: render a basic chart report from a Julian day and optional observer settings

Example usage:

```bash
# Print the compact compatibility profile
cargo run -q -p pleiades-cli -- profile-summary

# Render a simple tropical chart for J2000 with selected bodies
cargo run -q -p pleiades-cli -- chart \
  --jd 2451545.0 \
  --body Sun \
  --body Moon

# Render a sidereal chart with houses for a location
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
- `--ayanamsa` accepts built-in names like `Lahiri` and custom definitions such as `custom:True Balarama|2451545.0|12.5`.
- `--body` accepts built-in labels like `Sun`, `Moon`, `Ceres`, and custom identifiers like `asteroid:433-Eros`.
- Current first-party backends expose mean geometric coordinates only; `--apparent` is parsed for API compatibility but returns an unsupported-request error until apparent-place corrections are implemented. The chart CLI keeps UTC/UT1/TDB conversion explicit via `--tt-offset-seconds`, `--tt-from-utc-offset-seconds`, `--tt-from-ut1-offset-seconds`, `--tdb-offset-seconds`, `--tdb-from-utc-offset-seconds`, `--tdb-from-ut1-offset-seconds`, `--tdb-from-tt-offset-seconds`, and `--tt-from-tdb-offset-seconds`. See [docs/time-observer-policy.md](docs/time-observer-policy.md).

### `pleiades-validate`

Use `pleiades-validate` for comparison reports, benchmarks, artifact inspection, workspace audits, compatibility-profile verification, and release-bundle generation:

```bash
cargo run -q -p pleiades-validate -- help
```

Rough command overview:

- `compare-backends` / `compare-backends-audit`: compare the checked-in JPL snapshot against the algorithmic composite backend and fail the tolerance audit on regressions
- `backend-matrix` / `capability-matrix`: print detailed backend capability matrices
- `backend-matrix-summary` / `matrix-summary`: print compact backend capability matrices
- `benchmark [--rounds N]`: benchmark the candidate backend on the representative corpus
- `report` / `generate-report`: render the full validation report
- `validation-report-summary` / `validation-summary` / `report-summary`: compact validation report summary
- `validate-artifact`: inspect and validate the packaged compressed artifact in detail
- `artifact-summary` / `artifact-posture-summary`: compact packaged-artifact summary
- `workspace-audit` / `audit` / `native-dependency-audit`: check the workspace for mandatory native build hooks
- `compatibility-profile`: print the full release compatibility profile
- `compatibility-profile-summary`: compact compatibility profile summary
- `verify-compatibility-profile`: verify the release compatibility profile against the canonical catalogs
- `api-stability`, `api-stability-summary`, `release-notes`, `release-notes-summary`, `release-checklist`, `release-checklist-summary`, `release-summary`, `production-generation-boundary-summary`, `frame-policy-summary`, `observer-policy-summary`, `apparentness-policy-summary`, `time-scale-policy-summary`, `delta-t-policy-summary`, `request-surface-summary` / `request-surface`, `interpolation-posture-summary`, `packaged-lookup-epoch-policy-summary`, `lunar-reference-error-envelope-summary`, `lunar-equatorial-reference-error-envelope-summary`, `request-policy-summary` / `request-policy` / `request-semantics-summary` / `request-semantics`: release-facing report helpers
- `bundle-release --out DIR`: write a staged release bundle to a directory
- `verify-release-bundle --out DIR`: verify a previously staged release bundle
- `regenerate-packaged-artifact FILE` or `regenerate-packaged-artifact --out FILE`: rebuild the checked-in packaged artifact fixture from the reference snapshot

Example usage:

```bash
# Compare the reference snapshot with the current algorithmic backend
cargo run -q -p pleiades-validate -- compare-backends

# Generate a compact validation summary with fewer benchmark rounds
cargo run -q -p pleiades-validate -- report-summary --rounds 100

# Inspect the packaged artifact
cargo run -q -p pleiades-validate -- validate-artifact

# Run the workspace native-build audit
cargo run -q -p pleiades-validate -- audit

# Verify that the compatibility profile still matches the canonical catalogs
cargo run -q -p pleiades-validate -- verify-compatibility-profile

# Generate and then verify a release bundle
cargo run -q -p pleiades-validate -- bundle-release --out /tmp/pleiades-release
cargo run -q -p pleiades-validate -- verify-release-bundle --out /tmp/pleiades-release

# Regenerate the packaged artifact fixture
cargo run -q -p pleiades-cli -- regenerate-packaged-artifact /tmp/pleiades-packaged.bin
```

These tools are primarily contributor-facing today: they help exercise the chart stack, inspect release metadata, validate the packaged data backend, and rehearse the release workflow described in [docs/release-reproducibility.md](docs/release-reproducibility.md).

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

That audit checks the workspace manifests, lockfile, and crate-root `build.rs` files for mandatory native build hooks.

For a release-style smoke check of the validation bundle, run:

```bash
mise run release-smoke
```

That smoke check runs the workspace audit, validates the bundled compressed artifact, generates the bundle, and verifies the manifest checksums plus the manifest checksum sidecar through `pleiades-validate`.

For a step-by-step description of the release workflow, see [docs/release-reproducibility.md](docs/release-reproducibility.md).

## Workspace layout

The first-party crates live under `crates/` and follow the `pleiades-*` naming rule required by the specification.
