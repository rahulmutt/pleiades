# Deterministic math: route transcendentals through the pure-Rust `libm` crate (#288)

## Intent and why

`f64::sin`, `cos`, `atan2` and the other transcendental methods in `std` call the
host C library (glibc on Linux). Different glibc versions round some results
differently in the last bit. When GitHub's `ubuntu-latest` moved from 24.04 to
26.04, `pleiades-core`'s `apparent_chart_outputs_are_pinned` failed on every PR:
VSOP87-body coordinates and speeds moved by a few ULP, and the pinned checksum
moved (#285, runs 39928895887 / 39928902375). #286 pinned every workflow to
`ubuntu-24.04` as a stopgap.

The fix is to stop depending on the host libm: every transcendental call in the
workspace goes through the pure-Rust [`libm`](https://crates.io/crates/libm)
crate (the rust-lang port of musl's libm), whose results do not depend on the
platform's C library.

## Goals

- No workspace Rust code calls the `std` transcendental float methods, in
  library code **and** in tests, benches and test helpers. One simple rule:
  no system math anywhere in the workspace.
- A clippy guard keeps it that way: `disallowed_methods` in a workspace
  `clippy.toml`, enforced by the existing `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`.
- Every pinned checksum and golden file that moves is re-pinned in the same
  change, with a note recording that #288 moved only last bits.
- **Regenerate the committed packaged artifact in this PR (user's decision).**
  `pleiades-data/tests/artifact_regen.rs::regenerated_artifact_matches_committed`
  rebuilds the artifact from de440 + sb441-n373s and asserts byte identity
  with the committed fixture. Its fit path uses `atan2`/`asin`
  (`pleiades-data/src/regenerate.rs`, `pleiades-compression` frame
  recombination), so converting it moves bytes. Regenerate with the local
  kernels in `/workspace/.kernels` (see `docs/spk-kernel-sourcing.md`), make
  that test pass, re-check the accuracy baselines and thresholds the artifact
  feeds (`accuracy_baseline.rs`, `thresholds.rs`, coverage) against their
  existing limits, and record what moved in the PR. The JPL corpus regen test
  (`pleiades-jpl/tests/corpus_regen.rs`) is tolerance-based and should
  survive unchanged.
- CI returns to `runs-on: ubuntu-latest` (reverting #286's pin) and is green
  there. That is the proof the fix works.
- No measurable performance regression.

## Non-goals

- **No spec-level promise** of bit-identical results across platforms, and no
  CI on a second platform (macOS, ARM, Windows). The user's decision: fixing
  the Linux-runner problem is enough, and this is not a hard rule for users.
  The lint is a contributor guardrail, not a user-facing guarantee.
- No change to numerical algorithms, series, tolerances or validation
  thresholds.
- The C-backed Swiss Ephemeris reference generators under `tools/se-*` are
  excluded from the workspace and out of scope; their corpora are reference
  data, not outputs of our math.

## Anti-goals (would make this a failure even if it technically works)

- Accuracy changes beyond last-bit rounding. Every validation gate (SE/JPL
  corpora, crossings, aspects, stations, eclipses) must pass with its
  **existing** tolerance. Loosening a tolerance to absorb this change is not
  allowed. If one fails, that's a finding to investigate, not something to
  absorb.
- A performance regression in chart/batch workloads.
- Re-pinning a checksum without stating what moved. Each re-pin note should say
  what moved and roughly by how much, the way the existing notes in
  `bit_identity_tests.rs` do.

## Constraints

- Pure Rust: `libm` is pure Rust, MIT OR Apache-2.0 (already in `deny.toml`'s
  allow-list); it adds one small, rust-lang-maintained dependency.
- MSRV 1.99.1 (`workspace.package.rust-version`); pick a `libm` 0.2.x release
  compatible with it.
- Declare `libm` once under `[workspace.dependencies]` and use
  `libm.workspace = true` in each crate that needs it.
- Layering (`spec/architecture.md`): no new cross-crate coupling just for math.
  Each crate depends on `libm` directly; no shared math wrapper crate.
- Re-pinned goldens in the blocking tier (`release-smoke`'s crossings battery,
  crossings-golden + manifest) must be regenerated in this same PR, as with
  any longitude-moving change.
- `mise run ci` (including `mise run docs`) must pass, and `mise run test-full`
  before opening the PR, since `pleiades-validate`'s slow tier is excluded from
  blocking.

## Decided how (user-approved)

- Use the `libm` crate, not glibc, for: `sin`, `cos`, `tan`, `asin`, `acos`,
  `atan`, `atan2`, `sin_cos` (`libm::sincos`), `exp`, `ln`, `log10`, `log2`,
  `powf` (`libm::pow`), `sinh`, `cosh`, `tanh`, `cbrt`, `hypot`, `exp_m1`,
  `ln_1p`, and any other `std` float method that lowers to a libm call.
- Tests and test helpers are converted too (about 193 call sites), alongside
  library code (about 614), so the lint covers `--all-targets`.
- Leave alone what is already platform-independent: `sqrt` (IEEE-exact
  hardware instruction), `powi` (compiler-lowered multiplications), `abs`,
  `floor`, `ceil`, `round`, `trunc`, `rem_euclid`, `mul_add`, `min`/`max`,
  `to_radians`/`to_degrees`.
- Revert #286's `ubuntu-24.04` pin to `ubuntu-latest` in all workflows,
  including the comment added to `ci.yml`, and restore the original
  `mutants.yml` comment wording if it still reads naturally.

## Measured performance (microbenchmark, 2026-10-23)

Interleaved best-of-7 over 4M identical inputs, release build, loaded machine
(loadavg ~16, so absolute numbers are inflated; ratios are fair):

| Function | std (glibc) | `libm` | ratio |
|---|---|---|---|
| sin / cos | 17.6 / 17.5 ns | 17.1 / 17.2 ns | ~0.97× |
| cos, large args | 19.1 ns | 20.1 ns | 1.05× |
| tan | 19.4 ns | 22.4 ns | 1.16× |
| atan2 | 19.2 ns | 14.0 ns | 0.73× |
| asin / acos | 16.4 / 16.1 ns | 12.7 / 11.7 ns | ~0.75× |
| sin_cos | 23.3 ns | 18.9 ns | 0.81× |

3–8% of results differ from glibc in the last bit, so pinned values move.

**Performance bar (Claude's call).** Run `mise run benchmark` before and after,
alternating runs with `/proc/loadavg` reported, since the machine is shared.
The bar is the aggregate chart/batch figure: no slowdown beyond run-to-run
noise. Any single workload that slows by more than ~5% beyond noise (a
tan-heavy house system, say) is reported as a finding in the PR with numbers.
It is not an automatic blocker, and the fix for it is not to keep `std::tan`.

## Known pins to expect to move (starting list, not exhaustive)

- `pleiades-core` `CHART_CHECKSUM` (`chart/bit_identity_tests.rs`)
- `pleiades-events` `EVENTS_CHECKSUM` (`tests/fu25_bit_identity.rs`)
- `pleiades-core` `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`, if its
  content includes computed values
- `pleiades-validate` goldens checksums (`apparent_validation.rs`,
  `equatorial_validation.rs`, `topocentric_validation.rs`) and their CSVs, if
  they're generated by our code
- `pleiades-validate/data/crossings-corpus` golden + `manifest.txt`
- `pleiades-cli` `fixture_golden` outputs

Checksums over checked-in input data (nutation, leap-second and ΔT CSVs) hash
file bytes and should not move.

## Hand-picked boundary inputs (Claude's call)

Some mutation-killing tests use exact bit patterns chosen to land one ULP
from a boundary (e.g. `pleiades-apsides/src/tests.rs`). If the switch moves a
transcendental on that path, the builder may re-craft the input so the test
again exercises its boundary, with a short note per test, the same as for a
re-pin. Re-crafting an input is not loosening a tolerance. A test whose
*expected outcome* would change is a finding, not something to re-craft.

## Left to the builder

- The order of work (e.g. per crate, guarded by the lint from the start or
  turned on last), and how to split commits.
- The exact `clippy.toml` `disallowed_methods` list and its `reason` text,
  plus any narrowly scoped, documented `#[allow]` (there should be none
  expected).
- Whether the separate `fuzz/` workspace has any transcendental calls; convert
  them if so (cheap) for consistency.
- Whether to add a one-line note to `AGENTS.md` ("Numerical and domain
  correctness") telling contributors to use `libm::` for transcendentals.
- The wording of re-pin notes and the PR description.
- Release notes: release-plz handles versioning; the change is a fix (last-bit
  output changes, new `libm` dependency), not a breaking API change.
