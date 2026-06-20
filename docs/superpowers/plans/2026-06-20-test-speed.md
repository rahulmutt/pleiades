# Faster Test Suite With Gated Heavy Checks — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the default test run substantially faster with no loss of coverage (Phase 1), then move the heavy checks behind an opt-in cargo-nextest profile so the default run is a fast sanity pass while CI/release-gate still run everything (Phase 2).

**Architecture:** Three independent levers. (1) Optimize the `test`/`dev` build profiles so the numerical tests stop running unoptimized. (2) Switch the runner to cargo-nextest (global parallel pool across crate binaries) with a separate doctest step. (3) Make the heavy tests cheaper (memoized fixtures, deduped bundle builds, safely shrunk sweeps), then gate the remaining slow families via a nextest `slow` filterset.

**Tech Stack:** Rust workspace (12 crates), cargo, cargo-nextest (installed via mise), mise task runner.

## Global Constraints

- Rust edition 2021; workspace `rust-version = 1.96.0` — copied verbatim from `Cargo.toml`. Do not raise the MSRV.
- Pure-Rust, no new native build deps. Only new tool allowed: `cargo-nextest` (already in the mise shim, endorsed by `AGENTS.md`).
- **Coverage contract:** the `full` nextest profile (used by `ci` and `release-gate`) must run every test that exists today. The gate only changes what the bare default `mise test` runs.
- Doctests (~108 rust doctests across 20 files) must keep running — nextest cannot run them, so they get a dedicated `cargo test --doc` step.
- Phase 1 changes must not alter what any assertion checks. Each Phase 1c / Phase 2 change is verified by identical before/after pass set.
- Branch: `perf/test-suite-speedup` (already created). Commit after every task.
- Follow `AGENTS.md`: smallest change that solves the problem, manage tools via `mise.toml`.

---

### Task 1: Optimize the test and dev build profiles

**Files:**
- Modify: `Cargo.toml` (root, append a profiles section after the `[workspace.dependencies]` block)

**Interfaces:**
- Consumes: nothing.
- Produces: optimized `test`/`dev` builds. No code symbols.

- [ ] **Step 1: Capture the debug-profile baseline timing**

Run (records the slow crate's current execution time):
```bash
cargo test -p pleiades-data --lib --no-run
t0=$(date +%s); cargo test -p pleiades-data --lib 2>&1 | grep "test result" | tail -1; t1=$(date +%s); echo "DEBUG_RUN=$((t1-t0))s"
```
Expected: `183 passed` and `DEBUG_RUN` around 200s+ (baseline ~214s on the reference machine). Write the number down in the commit message.

- [ ] **Step 2: Add the profiles to `Cargo.toml`**

Append at end of root `Cargo.toml`:
```toml
# Optimize test and dev builds. The packaged-artifact fit/regeneration and
# corpus comparison tests are numerically heavy; at opt-level = 0 they dominate
# the suite (pleiades-data lib measured 214s debug vs 76s at opt-level = 2).
# One-time cost: ~30s extra cold compile; incremental rebuilds slightly slower.
[profile.test]
opt-level = 2

[profile.dev]
opt-level = 2
```

- [ ] **Step 3: Recompile and re-time the slow crate**

Run:
```bash
cargo test -p pleiades-data --lib --no-run
t0=$(date +%s); cargo test -p pleiades-data --lib 2>&1 | grep "test result" | tail -1; t1=$(date +%s); echo "OPT_RUN=$((t1-t0))s"
```
Expected: same `183 passed; 0 failed`, and `OPT_RUN` roughly 1/3 of the debug baseline (reference: ~76s). Same pass count is the coverage check.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "perf(test): optimize test/dev build profiles (opt-level=2)

pleiades-data lib tests: ~214s -> ~76s. One-time ~30s extra cold compile."
```

---

### Task 2: Adopt cargo-nextest with a dedicated doctest step

**Files:**
- Modify: `mise.toml` (`[tools]`, `[tasks.test]`; add `[tasks.doctest]`)

**Interfaces:**
- Consumes: Task 1's optimized profile (nextest builds with the `test` profile).
- Produces: `mise test` = nextest run (all tests) + doctests. Later tasks add a `full`/default profile split on top.

- [ ] **Step 1: Pin cargo-nextest in `[tools]`**

In `mise.toml`, change:
```toml
[tools]
rust = { version = "1.96.0", components = "rustfmt,clippy" }
"cargo:cargo-release" = "1.1.2"
```
to add nextest:
```toml
[tools]
rust = { version = "1.96.0", components = "rustfmt,clippy" }
"cargo:cargo-release" = "1.1.2"
"cargo:cargo-nextest" = "0.9"
```

- [ ] **Step 2: Verify nextest runs the workspace and reports a test count**

Run:
```bash
cargo nextest run --workspace 2>&1 | tail -5
```
Expected: a `Summary ... tests run: N passed` line with N in the ~1900 range (nextest excludes the ~108 doctests, which run separately). All passing.

- [ ] **Step 3: Add a `doctest` task and point `test` at nextest + doctests**

In `mise.toml`, replace:
```toml
[tasks.test]
run = "cargo test --workspace"
```
with:
```toml
[tasks.test]
depends = ["nextest", "doctest"]

[tasks.nextest]
run = "cargo nextest run --workspace"

[tasks.doctest]
run = "cargo test --workspace --doc"
```

- [ ] **Step 4: Verify the new `test` task and doctests**

Run:
```bash
mise run test 2>&1 | tail -8
```
Expected: nextest summary (all passed) followed by the doctest run reporting `test result: ok` with ~100+ doctests measured, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add mise.toml
git commit -m "build(test): run tests via cargo-nextest with a dedicated doctest step"
```

---

### Task 3: Capture the per-test timing inventory (drives the slow-set)

**Files:**
- Create: `docs/superpowers/plans/test-timings.md` (working artifact — the measured slow-test list; committed so later tasks and reviewers share one source of truth)

**Interfaces:**
- Consumes: Task 2's nextest setup.
- Produces: `test-timings.md` — a ranked list of the slowest tests by name, used verbatim by Tasks 4–6 (which to optimize) and Task 7 (which to tag `slow`).

- [ ] **Step 1: Record per-test timings**

Run (nextest prints per-test durations; capture the slowest):
```bash
cargo nextest run --workspace --no-fail-fast 2>&1 | grep -E "PASS|SLOW" | sort -t'[' -k2 -rn | head -40
```
Expected: a list of `PASS [   Ns] crate test::name` lines, slowest first. nextest also auto-flags long tests as `SLOW`.

- [ ] **Step 2: Write the inventory**

Create `docs/superpowers/plans/test-timings.md` with:
- A table of the 30–40 slowest tests: `crate | test name | seconds`.
- A "Families" section grouping them, expected to include:
  - kernel-free regeneration: `build_from_reference_produces_all_bodies_with_spanning_segments`, `default_window_artifact_matches_explicit_default_over`, and other `pleiades-data` callers of `build_packaged_artifact_from_reference_over` / `try_regenerate_packaged_artifact_from_snapshot`.
  - release-bundle: tests in `pleiades-validate` `tests/release_bundle_verify_a.rs` / `release_bundle_verify_b.rs` and `test_support` helpers that call `render_cli(&["bundle-release", ...])` (45 call sites).
  - dense numerical sweeps: `pleiades-data` `tests/fit.rs` / `tests/coverage.rs` / `tests/lookup.rs` sweep tests, and `pleiades-vsop87` `tests/evidence.rs` / `tests/backend.rs`.
- A one-line note per family on the planned treatment (memoize / dedup / shrink / tag-only).

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/plans/test-timings.md
git commit -m "docs(test): record per-test timing inventory and slow-test families"
```

---

### Task 4: Memoize the kernel-free regeneration fixture

**Files:**
- Modify: `crates/pleiades-data/src/test_support.rs` (add a memoized builder)
- Modify: `crates/pleiades-data/src/tests/coverage.rs` and any other `pleiades-data` test calling `build_packaged_artifact_from_reference_over` with the **default** window
- Reference: `crates/pleiades-data/src/regenerate.rs:2443` — `pub(crate) fn build_packaged_artifact_from_reference_over(reference: &dyn EphemerisBackend, base_window: (f64, f64)) -> CompressedArtifact`

**Interfaces:**
- Consumes: `build_packaged_artifact_from_reference_over`, the `Synthetic` reference backend used in those tests.
- Produces: `pub(crate) fn synthetic_default_artifact() -> &'static CompressedArtifact` in `test_support` — a once-built artifact over the default window, reused by all identical-input call sites.

- [ ] **Step 1: Confirm which call sites use identical inputs**

Run:
```bash
grep -rn "build_packaged_artifact_from_reference_over" crates/pleiades-data/src/tests/*.rs
```
Expected: list of call sites. Only memoize those built from `&Synthetic` over the **same default window**. Leave comparison tests that intentionally build two different windows (e.g. `default_window_artifact_matches_explicit_default_over`, which compares `a` vs `b`) untouched — they need distinct builds.

- [ ] **Step 2: Add the memoized helper to `test_support.rs`**

Add (matching the existing `OnceLock` style used in `data/mod.rs`):
```rust
use std::sync::OnceLock;

/// The kernel-free synthetic artifact over the default window, built once per
/// test binary. The build is numerically heavy; identical-input call sites
/// share this instead of rebuilding per test.
pub(crate) fn synthetic_default_artifact() -> &'static crate::CompressedArtifact {
    static ARTIFACT: OnceLock<crate::CompressedArtifact> = OnceLock::new();
    ARTIFACT.get_or_init(|| {
        let base_window = (2_451_545.0, 2_451_545.0 + 200.0);
        crate::regenerate::build_packaged_artifact_from_reference_over(
            &crate::tests::coverage::Synthetic,
            base_window,
        )
    })
}
```
(If `Synthetic` is private to `coverage.rs`, hoist it into `test_support` first and re-import it in `coverage.rs`; do that as the first edit of this step.)

- [ ] **Step 3: Point identical-input call sites at the helper**

For each confirmed call site, replace the local `let artifact = build_packaged_artifact_from_reference_over(&Synthetic, base_window);` with `let artifact = synthetic_default_artifact();` and adjust to the borrowed `&CompressedArtifact` (clone only if a test mutates it).

- [ ] **Step 4: Verify identical pass set and a speedup**

Run:
```bash
cargo nextest run -p pleiades-data 2>&1 | tail -3
```
Expected: same passed count as before this task, 0 failed. Spot-check one rewritten test name still present in the run.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/test_support.rs crates/pleiades-data/src/tests/
git commit -m "perf(test): memoize kernel-free synthetic artifact across pleiades-data tests"
```

---

### Task 5: Deduplicate repeated release-bundle builds

**Files:**
- Modify: `crates/pleiades-validate/src/tests/test_support.rs` (add a shared built-bundle helper)
- Modify: `crates/pleiades-validate/src/tests/release_bundle_verify_a.rs`, `release_bundle_verify_b.rs` (use it where assertions are independent of build inputs)
- Reference: `render_cli(&["bundle-release", "--out", <dir>, "--rounds", "1"])` at `crates/pleiades-validate/src/render/cli.rs:72`

**Interfaces:**
- Consumes: `render_cli`, `unique_temp_dir`.
- Produces: `pub(crate) fn shared_release_bundle_dir() -> &'static std::path::Path` in validate `test_support` — a bundle built once per binary, for read-only verification tests.

- [ ] **Step 1: Identify read-only bundle tests**

Run:
```bash
grep -rn "bundle-release" crates/pleiades-validate/src/tests/*.rs | wc -l
grep -rln "bundle-release" crates/pleiades-validate/src/tests/*.rs
```
Expected: ~45 call sites. Classify each: tests that **only read/verify** a freshly built untampered bundle can share one build; tests that **tamper** the bundle, pass alias/error args, or assert on build-time behavior must keep building their own.

- [ ] **Step 2: Add the shared-bundle helper**

In validate `test_support.rs`:
```rust
use std::path::Path;
use std::sync::OnceLock;

/// A release bundle built once per test binary for read-only verification
/// tests. Tests that tamper with or rebuild the bundle must build their own.
pub(crate) fn shared_release_bundle_dir() -> &'static Path {
    static DIR: OnceLock<std::path::PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = unique_temp_dir("pleiades-shared-release-bundle");
        let dir_string = dir.to_string_lossy().to_string();
        render_cli(&["bundle-release", "--out", &dir_string, "--rounds", "1"])
            .expect("shared release bundle should build");
        dir
    })
}
```

- [ ] **Step 3: Rewrite read-only verification tests to use the shared bundle**

For each read-only test, replace its build-then-verify preamble with:
```rust
let bundle_dir = shared_release_bundle_dir();
let bundle_dir_string = bundle_dir.to_string_lossy().to_string();
let verified = render_cli(&["verify-release-bundle", "--out", &bundle_dir_string])
    .expect("verify-release-bundle should succeed");
```
Do not remove the per-test build from any tamper/alias/error test.

- [ ] **Step 4: Verify identical pass set**

Run:
```bash
cargo nextest run -p pleiades-validate 2>&1 | tail -3
```
Expected: same passed count as before, 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/tests/
git commit -m "perf(test): share one release bundle across read-only verify tests"
```

---

### Task 6: Shrink oversized numerical sweeps where provably safe

**Files:**
- Modify: dense-sweep tests named in `test-timings.md` under `pleiades-data` (`tests/fit.rs`, `tests/coverage.rs`, `tests/lookup.rs`) and `pleiades-vsop87` (`tests/evidence.rs`, `tests/backend.rs`)

**Interfaces:**
- Consumes: the per-test inventory from Task 3.
- Produces: no new symbols; smaller iteration counts on sweeps that prove a single property.

- [ ] **Step 1: For each sweep test in the inventory, decide reduce-or-keep**

A sweep is reducible only if it iterates many sample points/fractions to prove **one** property (e.g. "all residuals under threshold"). Keep at least one full-scale check of that property somewhere in the suite. Tests asserting per-point golden values are NOT reducible — skip them.

- [ ] **Step 2: Reduce the iteration count, leaving a comment**

For a reducible sweep, cut the sample set to a representative subset (endpoints + a few interior points) and add a comment:
```rust
// Reduced sweep: a representative subset proves the residual-bound property.
// Full-scale coverage retained by `<name of the full-scale test>`.
```
Make one such edit per test, each in its own logical change.

- [ ] **Step 3: Verify the property still fails when broken (sanity)**

For one reduced test, temporarily perturb the threshold/expected value, run it, confirm it FAILS, then revert. This proves the shrunk test still has teeth.
```bash
cargo nextest run -p pleiades-data -E 'test(<reduced_test_name>)' 2>&1 | tail -3
```

- [ ] **Step 4: Verify full pass set across both crates**

Run:
```bash
cargo nextest run -p pleiades-data -p pleiades-vsop87 2>&1 | tail -3
```
Expected: same passed count (no tests deleted — only iteration counts changed), 0 failed.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/tests/ crates/pleiades-vsop87/src/tests/
git commit -m "perf(test): shrink oversized numerical sweeps, retaining full-scale property checks"
```

---

### Task 7: Add the nextest `slow` filterset and default/full profiles (Phase 2)

**Files:**
- Create: `.config/nextest.toml` (workspace root)

**Interfaces:**
- Consumes: the slow-test families from `test-timings.md` (Task 3).
- Produces: nextest `default` profile (excludes `slow`) and `full` profile (runs everything). Task 8 wires mise tasks to these.

- [ ] **Step 1: Write `.config/nextest.toml`**

Create it with a `slow` filterset matching the measured families (adjust the patterns to the real test/binary names from `test-timings.md`):
```toml
# Default profile = fast sanity run: excludes the heavy data/regeneration,
# release-bundle, and dense-sweep families. The `full` profile runs everything
# and is what CI and release-gate use — the gate never reduces CI coverage.

[[profile.default.overrides]]
# Tests matched here are skipped by the default profile only.
filter = """
  test(/build_from_reference/)
  + test(/regenerate_packaged_artifact/)
  + binary(/release_bundle_verify/)
  + test(/_sweep$/) + test(/dense_/)
"""
default-filter = "not (
  test(/build_from_reference/)
  + test(/regenerate_packaged_artifact/)
  + binary(/release_bundle_verify/)
  + test(/_sweep$/) + test(/dense_/)
)"

[profile.full]
# Runs every test; no default-filter exclusions.
```
Note: nextest's exclusion is expressed via `default-filter` on the `default` profile; the `full` profile leaves `default-filter` at its built-in `all()`. Refine the filterset expression against `cargo nextest run --profile default --list` until only the intended tests are excluded.

- [ ] **Step 2: Verify the default profile excludes the slow set**

Run:
```bash
cargo nextest run --profile default --list 2>&1 | tail -3
cargo nextest run --profile full --list 2>&1 | tail -3
```
Expected: the `full` list count is strictly greater than the `default` list count, and the difference equals the number of slow-family tests.

- [ ] **Step 3: Verify the default profile is faster and the full profile still all-passes**

Run:
```bash
t0=$(date +%s); cargo nextest run --profile default 2>&1 | tail -2; t1=$(date +%s); echo "DEFAULT=$((t1-t0))s"
t0=$(date +%s); cargo nextest run --profile full 2>&1 | tail -2; t1=$(date +%s); echo "FULL=$((t1-t0))s"
```
Expected: `DEFAULT` noticeably less than `FULL`; both report 0 failed; `full` runs the larger count.

- [ ] **Step 4: Commit**

```bash
git add .config/nextest.toml
git commit -m "test: add nextest default (fast) and full profiles with a slow filterset"
```

---

### Task 8: Wire mise tasks to the profiles and document the split (Phase 2)

**Files:**
- Modify: `mise.toml` (`nextest` task → default profile; add `test-full`; ensure `ci`/`release-gate` use full)
- Modify: `spec/validation-and-testing.md` (document fast-vs-full)

**Interfaces:**
- Consumes: Task 7's profiles.
- Produces: `mise test` (fast default + doctests), `mise test-full` (full + doctests). `ci`/`release-gate` run full.

- [ ] **Step 1: Point the default `nextest` task at the default profile, add `test-full`**

In `mise.toml`, change the `nextest` task and add a full variant:
```toml
[tasks.nextest]
run = "cargo nextest run --workspace --profile default"

[tasks.nextest-full]
run = "cargo nextest run --workspace --profile full"

[tasks.test-full]
depends = ["nextest-full", "doctest"]
```
Leave `[tasks.test]` depending on `["nextest", "doctest"]` (now fast + doctests).

- [ ] **Step 2: Make `ci` and `release-gate` run the full set**

`ci` and `release-gate` currently depend on `test`. Change both to depend on `test-full` instead of `test` so CI/release run every test (the coverage contract). Edit the `depends` arrays:
```toml
[tasks.release-gate]
depends = ["fmt", "lint", "test-full", "benchmark", "audit", "package-check", "release-smoke"]

[tasks.ci]
depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke"]
```

- [ ] **Step 3: Verify both task paths**

Run:
```bash
mise run test 2>&1 | tail -4
mise run test-full 2>&1 | tail -4
```
Expected: `test` runs the smaller (default) count + doctests; `test-full` runs the full count + doctests; both 0 failed.

- [ ] **Step 4: Document the split in the spec**

In `spec/validation-and-testing.md`, under the testing/release-gates section, add a short subsection:
```markdown
### Fast default vs full test runs

`mise test` runs the fast default nextest profile (heavy data-regeneration,
release-bundle, and dense-sweep families excluded) plus all doctests — a quick
sanity pass for local iteration. `mise test-full` runs the `full` nextest
profile (every test) plus doctests. CI and `release-gate` always run
`test-full`, so the gate never reduces released coverage. The excluded
families are listed by filterset in `.config/nextest.toml`.
```

- [ ] **Step 5: Commit**

```bash
git add mise.toml spec/validation-and-testing.md
git commit -m "test: gate heavy checks behind nextest full profile; CI/release-gate run full"
```

---

## Self-Review

**1. Spec coverage**
- Build profile (1a) → Task 1. ✓
- nextest runner + doctest step (1b) → Task 2 (+ profile split in Task 8). ✓
- Test-level perf — memoize fixtures (1c) → Task 4; dedup bundle builds (1c) → Task 5; safe input shrinking (1c) → Task 6. ✓
- Phase 2 gating via nextest groups + `full` profile → Tasks 7–8. ✓
- Coverage contract (CI/release-gate run full) → Task 8 Step 2. ✓
- Doctests preserved → Task 2 + Task 8. ✓
- Spec doc of fast-vs-full split → Task 8 Step 4. ✓
- Verification strategy (count parity, before/after timing) → built into each task's verify step and Task 7 Step 2. ✓
- Files-touched list in spec → all covered by Tasks 1–8. ✓

**2. Placeholder scan:** No "TBD/TODO/handle edge cases". The one inherently discovery-driven element (which exact tests are slowest) is made concrete by Task 3 producing `test-timings.md`, which Tasks 4–7 consume by name. Filterset patterns in Task 7 are explicitly flagged to refine against `--list` output — this is a tuning step with a concrete acceptance gate (count difference equals slow-family size), not a placeholder.

**3. Type consistency:** `synthetic_default_artifact() -> &'static CompressedArtifact` (Task 4) and `shared_release_bundle_dir() -> &'static Path` (Task 5) are referenced consistently. `build_packaged_artifact_from_reference_over(&dyn EphemerisBackend, (f64,f64)) -> CompressedArtifact` matches `regenerate.rs:2443`. Profile/task names (`default`, `full`, `nextest`, `nextest-full`, `test-full`, `doctest`) are used consistently across Tasks 2, 7, 8.
