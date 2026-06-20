# Faster Tests via Gating (gate-first, cargo test) — Revised Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Supersedes** the runner/test-level tasks (2,4,5,6,7,8) of `2026-06-20-test-speed.md`. Background: the cargo-nextest approach was abandoned after measurement — see `docs/superpowers/specs/2026-06-20-test-speed-findings-addendum.md`. The build-profile optimization (Task 1, commit `ce74b8f7`) and the timing inventory (`docs/superpowers/plans/test-timings.md`) are KEPT.

**Goal:** Make the default `cargo test` run fast by gating the heavy test families behind `#[ignore]`, so they run only on opt-in / in CI — without losing any coverage in CI/release-gate.

**Architecture:** Keep `cargo test` (preserves the process-scoped cache amortization that makes the suite fast in one process). Mark slow tests `#[ignore = "slow: ..."]`. `mise test` runs the fast default (skips ignored); `mise test-full` runs `cargo test -- --include-ignored` (everything). CI and release-gate use `test-full`.

**Tech Stack:** Rust workspace, cargo test, mise.

## Global Constraints

- Rust edition 2021; workspace `rust-version = 1.96.0` — do NOT raise MSRV.
- **Coverage contract:** `cargo test --workspace -- --include-ignored` (what CI/release-gate run) must run and pass EVERY test that exists today. Gating only changes what the bare `cargo test` default runs. No test is deleted.
- Slow tests must stay COMPILED by default (use `#[ignore]`, NOT `#[cfg(feature=...)]`), so default builds still catch breakage.
- Ignore-reason string convention, used verbatim on every gated test:
  `#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]`
- Measured baseline under `cargo test` with opt-level=2 already applied: pleiades-data 45s, **pleiades-validate 522s**, pleiades-cli 101s. Other crates < 5s.
- Branch: `perf/test-suite-speedup`. Commit after every task.
- Use the existing `#[ignore]` precedent in the repo (`PLEIADES_ENFORCE_LATENCY` latency test) as the style reference.

---

### Task G1: Gate the slow pleiades-validate tests (biggest win)

**Files:**
- Modify: `crates/pleiades-validate/src/tests/release_bundle_verify_a.rs`, `release_bundle_verify_b.rs`, and any other `crates/pleiades-validate/src/tests/*.rs` containing tests measured > ~5s (notably `tests/report.rs`, and `artifact::tests::render_artifact_summary_includes_span_caps`).

**Interfaces:**
- Consumes: nothing. Produces: a faster default `cargo test -p pleiades-validate`.

- [ ] **Step 1: Identify the slow tests in this crate by measured time**

Run (per-test timing via libtest report-time on the stable harness):
```bash
cargo test -p pleiades-validate -- -Zunstable-options --report-time 2>/dev/null | grep -E "test .* \.\.\. ok" | awk '{print $NF, $2}' | sort -rn | head -60 || \
cargo test -p pleiades-validate -- --report-time 2>&1 | grep -E "<[0-9.]+s>" | sort -t'<' -k2 -rn | head -60
```
(If `--report-time` is unavailable, fall back to the inventory in `docs/superpowers/plans/test-timings.md` Section 1 + the family list in Section 2.) Record the set of test functions taking more than ~2s.

- [ ] **Step 2: Add `#[ignore]` to each slow test**

For every slow test function identified, insert immediately above its `#[test]` line:
```rust
#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
```
The two `release_bundle_verify_*.rs` files have all their `#[test]` functions in the slow set — gate every `#[test]` in them. In other files, gate only the measured-slow functions.

- [ ] **Step 3: Verify the default run is fast and skips the slow set**

Run:
```bash
t0=$(date +%s); cargo test -p pleiades-validate 2>&1 | grep -E "test result" | tail -3; t1=$(date +%s); echo "DEFAULT=$((t1-t0))s"
```
Expected: `DEFAULT` is a small fraction of the 522s baseline (target < ~30s), and the result line shows a non-zero `ignored` count equal to the number of tests you gated.

- [ ] **Step 4: Verify the full run still passes everything (coverage contract)**

Run:
```bash
cargo test -p pleiades-validate -- --include-ignored 2>&1 | grep -E "test result" | tail -3
```
Expected: 0 failed, 0 ignored (everything runs), and the total passed count equals the pre-task passed count.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/tests/
git commit -m "test(validate): gate slow release-bundle/benchmark tests behind #[ignore]"
```

---

### Task G2: Gate the slow pleiades-cli tests

**Files:**
- Modify: `crates/pleiades-cli/src/cli/tests/*.rs` (the slow summary/release/artifact/validation/misc CLI tests).

**Interfaces:**
- Consumes: nothing. Produces: faster default `cargo test -p pleiades-cli`.

- [ ] **Step 1: Identify slow cli tests by measured time**

Run:
```bash
cargo test -p pleiades-cli -- --report-time 2>&1 | grep -E "<[0-9.]+s>" | sort -t'<' -k2 -rn | head -40
```
(Fallback: `test-timings.md` lists the slow cli tests by name, e.g. `summary_commands_render_compact_reports`, `bundle_release_commands_accept_output_alias`, `verify_release_bundle_command_verifies_a_staged_bundle`, `artifact_and_workspace_commands_render_compact_reports`, `validation_report_commands_render_compact_reports`, `fallback_summary_commands_remain_reachable_from_the_cli`, the two `packaged_artifact_*` misc tests.) Record tests > ~2s.

- [ ] **Step 2: Add `#[ignore]` to each slow cli test**

Insert above each slow test's `#[test]` line:
```rust
#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
```

- [ ] **Step 3: Verify fast default**

Run:
```bash
t0=$(date +%s); cargo test -p pleiades-cli 2>&1 | grep -E "test result" | tail -1; t1=$(date +%s); echo "DEFAULT=$((t1-t0))s"
```
Expected: `DEFAULT` well under the 101s baseline (target < ~15s); `ignored` count equals the number gated.

- [ ] **Step 4: Verify full run passes**

Run:
```bash
cargo test -p pleiades-cli -- --include-ignored 2>&1 | grep -E "test result" | tail -1
```
Expected: 0 failed; total passed equals pre-task passed count.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/cli/tests/
git commit -m "test(cli): gate slow benchmark/bundle CLI tests behind #[ignore]"
```

---

### Task G3: Gate the slow pleiades-data fit-analysis tests

**Files:**
- Modify: `crates/pleiades-data/src/tests/coverage.rs` (the `packaged_artifact_fit_*`, `packaged_artifact_generation_*`, `packaged_artifact_generator_parameters_*`, `packaged_artifact_production_profile_*`, `packaged_artifact_regeneration_summary_*` tests), and the kernel-free regeneration tests (`build_from_reference_produces_all_bodies_with_spanning_segments`, `default_window_artifact_matches_explicit_default_over`).

**Interfaces:**
- Consumes: nothing. Produces: faster default `cargo test -p pleiades-data`.

- [ ] **Step 1: Identify slow data tests by measured time**

Run:
```bash
cargo test -p pleiades-data -- --report-time 2>&1 | grep -E "<[0-9.]+s>" | sort -t'<' -k2 -rn | head -20
```
Note: because these share a process-scoped `OnceLock` fit-analysis cache, only the FIRST few to run will show large times under `cargo test`; later ones are fast. Gate by NAME (the families listed above + in `test-timings.md`), not only by the single test that happened to pay the cache cost. Record the family members.

- [ ] **Step 2: Add `#[ignore]` to each fit-analysis / regeneration test**

Insert above each `#[test]` line:
```rust
#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
```

- [ ] **Step 3: Verify fast default**

Run:
```bash
t0=$(date +%s); cargo test -p pleiades-data 2>&1 | grep -E "test result" | tail -1; t1=$(date +%s); echo "DEFAULT=$((t1-t0))s"
```
Expected: `DEFAULT` well under 45s (target < ~10s); `ignored` count equals the number gated.

- [ ] **Step 4: Verify full run passes**

Run:
```bash
cargo test -p pleiades-data -- --include-ignored 2>&1 | grep -E "test result" | tail -1
```
Expected: 0 failed; total passed equals pre-task passed count.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/tests/
git commit -m "test(data): gate slow fit-analysis/regeneration tests behind #[ignore]"
```

---

### Task G4: Wire mise tasks + document the fast/full split

**Files:**
- Modify: `mise.toml` (add `test-full`; point `ci`/`release-gate` at it).
- Modify: `spec/validation-and-testing.md` (document the split).

**Interfaces:**
- Consumes: G1–G3 gating. Produces: `mise test` (fast), `mise test-full` (everything), CI/release-gate on full.

- [ ] **Step 1: Add `test-full` and keep `test` fast**

`mise.toml` currently has `[tasks.test] run = "cargo test --workspace"`. Keep it (it now skips ignored = fast). Add:
```toml
[tasks.test-full]
run = "cargo test --workspace -- --include-ignored"
```

- [ ] **Step 2: Point `ci` and `release-gate` at the full run**

In `mise.toml`, change both `depends` arrays to use `test-full` instead of `test`:
```toml
[tasks.release-gate]
depends = ["fmt", "lint", "test-full", "benchmark", "audit", "package-check", "release-smoke"]

[tasks.ci]
depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke"]
```

- [ ] **Step 3: Verify both task paths**

Run:
```bash
t0=$(date +%s); mise run test 2>&1 | grep -E "test result" | tail -3; t1=$(date +%s); echo "FAST=$((t1-t0))s"
mise run test-full 2>&1 | grep -E "test result" | tail -3
```
Expected: `FAST` is a small fraction of the full run; `test-full` reports 0 failed and 0 ignored across the workspace.

- [ ] **Step 4: Document the split**

In `spec/validation-and-testing.md`, add under the testing/release-gates section:
```markdown
### Fast default vs full test runs

`cargo test` (and `mise test`) skip tests marked `#[ignore = "slow: ..."]` —
the heavy release-bundle, benchmark/validation-report, and fit-analysis
families — giving a fast local sanity run. `mise test-full`
(`cargo test --workspace -- --include-ignored`) runs every test. CI and
`release-gate` always run `test-full`, so the gate never reduces released
coverage. The slow families are catalogued in
`docs/superpowers/plans/test-timings.md`.
```

- [ ] **Step 5: Commit**

```bash
git add mise.toml spec/validation-and-testing.md
git commit -m "test: add mise test-full; CI/release-gate run all tests incl. #[ignore]'d slow ones"
```

---

## Self-Review

**Spec coverage:** Default-run speedup → G1 (validate, the 522s dominator), G2 (cli), G3 (data). Coverage contract (CI runs everything) → G4 Step 2 + every task's Step 4 (`--include-ignored` parity). Fast/full split documented → G4 Step 4. Profile win + inventory → retained from prior work.

**Placeholder scan:** The only discovery element is "which exact tests are slow" — each task's Step 1 measures it (with the committed inventory as a named fallback). The ignore-reason string is fixed verbatim in Global Constraints.

**Type consistency:** No new symbols; only `#[ignore]` attributes and mise task names (`test`, `test-full`) used consistently across G1–G4.
