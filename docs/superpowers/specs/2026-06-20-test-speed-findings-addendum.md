# Test-speed investigation — findings addendum (2026-06-20)

Supplements `2026-06-20-test-speed-design.md` with what execution revealed.
The original design assumed cargo-nextest would be a near-drop-in faster runner.
Measurement during execution disproved that assumption. This document records
the measured facts so the runner decision can be made with full information.

## What landed cleanly

- **Build profile opt-level=2** (commit `ce74b8f7`): pleiades-data lib tests
  199s → 45s (debug→opt), 183 passed both sides. Runner-agnostic win. KEPT.
- **Per-test timing inventory** (`docs/superpowers/plans/test-timings.md`,
  commits `e17975da`/`c3b2463a`). Useful but see the caveat below.

## What broke, and why

A naive `cargo test` → `cargo nextest run` swap (committed `7d927412`, reverted
`8f538427`) made the suite take ~1400s+ and fail non-deterministically.

**Root cause — measured, not assumed:**

This workspace's test speed depends on *process-scoped memoization* of several
very expensive computations. Under `cargo test` (one process per test binary,
tests are threads) each expensive computation runs once and every test in that
binary reuses it. Under `cargo nextest` (one process **per test**) every test
re-pays the full cost, in parallel, contending for CPU.

Three confirmed expensive, memoized computations:

1. **Fit-analysis summaries** (`pleiades-data`):
   `packaged_artifact_fit_outlier_summary_details()` and siblings in
   `crates/pleiades-data/src/coverage/{threshold,profile,target}.rs`. The heavy
   call is `packaged_artifact_fit_outlier_samples_for_current_artifact()`,
   memoized in a `OnceLock`. Measured: a single such test in isolation =
   **39s**; all 183 pleiades-data lib tests in one process = **45s** — proving
   the 39s is a one-time per-process cost amortized under cargo test.
2. **Benchmark path** (`pleiades-validate`):
   `render_benchmark_report` (`render/text/benchmark.rs`, OnceLock-cached) and
   `render_packaged_artifact_latency_budget_summary` (renders live timings).
3. **Validation-report path** (`pleiades-validate`):
   `build_validation_report` (`render/text/compatibility.rs`, OnceLock-cached),
   pulled into every `bundle-release` test.

**Measured control facts** (under opt-level=2, nextest, isolated = no contention):
- Raw artifact decode+validate: **0.25s** (NOT the bottleneck).
- One fit-analysis coverage test, isolated: **39s**.
- The 100–374s/test figures in `test-timings.md` are those per-process costs
  *inflated by nextest CPU contention* (~16 parallel processes), not genuine
  single-test work.

**Determinism:** some rendered summaries embed live measured timings
(`render_packaged_artifact_latency_budget_summary`, benchmark report). Under
parallel load these shift between staging and verification renders, breaking the
exact-equality bundle checks → the non-deterministic failures. (The validation
report already strips timing lines via `normalize_validation_report_*`; the
benchmark report is checksum-compared without normalization.)

## Consequence for the runner decision

nextest does not "just" need a localized fast-path in one crate. To run under
nextest without per-process blow-up, EVERY one of the process-scoped expensive
computations above must get a cheap, deterministic test-mode path —
fit-analysis in `pleiades-data` AND benchmark + validation-report in
`pleiades-validate`. That is pervasive production-code change touching
fit-analysis and release-verification semantics, with assertion blast radius
across ~100+ tests. Existing knobs only partially help: `--rounds 1` is already
used by all bundle tests; there is no sample-density knob for the fit-analysis.

## The two honest paths

- **A. Gate-first under cargo test (low risk, no nextest).** Keep `cargo test`
  (preserves the shared-cache amortization). Gate the heavy families
  (fit-analysis coverage tests, benchmark/validation/bundle tests) behind
  `#[ignore = "...; opt-in via PLEIADES_RUN_SLOW=1"]`, mirroring the existing
  `PLEIADES_ENFORCE_LATENCY` precedent. Default `cargo test` skips the ~39s
  computations → fast sanity run; CI/full sets the env var. Achieves the stated
  goal ("slow by default → fast, gate the long checks") with the profile win
  already in place, and no risky source change.

- **B. Fast-path then nextest (high effort/risk).** Add test-mode
  cheap+deterministic paths for fit-analysis (sample-density knob) AND benchmark
  AND validation-report, keeping every assertion meaningful, THEN adopt nextest.
  Delivers nextest parallelism on top of gating, at the cost of pervasive,
  delicate changes to fit-analysis and release-verification code.

The user selected option 2 (path B) before the fit-analysis scope was known.
This addendum exists so that choice can be reaffirmed or revised with the true
scope visible.
