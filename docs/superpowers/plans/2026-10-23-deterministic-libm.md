# Deterministic libm Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** No workspace Rust code calls the host C library's transcendental math. Every such call goes through the pure-Rust `libm` crate, so pinned outputs stop depending on the runner's glibc (#288).

**Architecture:** Each crate that needs transcendentals depends on `libm` directly through `[workspace.dependencies]`, with no wrapper crate. A workspace `clippy.toml` `disallowed_methods` list is both the inventory and the guard. After conversion, the packaged artifact is regenerated from the local kernels, every moved checksum/golden is re-pinned with a note, and #286's `ubuntu-24.04` runner pin is reverted.

**Tech Stack:** Rust 1.99.1 (MSRV), `libm = "0.2.16"`, clippy `disallowed-methods`, cargo-nextest, mise tasks, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-23-deterministic-libm-design.md`

## Global Constraints

- Pure Rust; `libm` is MIT OR Apache-2.0 (already allowed by `deny.toml`). No other new dependency.
- MSRV 1.99.1 (`workspace.package.rust-version`).
- `libm` declared once in root `Cargo.toml` `[workspace.dependencies]`; crates use `libm.workspace = true` (as a `[dev-dependencies]` entry where only tests need it).
- No system math anywhere in the workspace, tests/benches/test helpers included, plus the separate `fuzz/` workspace if it has any calls.
- Leave as is: `sqrt`, `powi`, `abs`, `floor`, `ceil`, `round`, `trunc`, `rem_euclid`, `mul_add`, `min`/`max`, `to_radians`/`to_degrees`.
- No `#[allow(clippy::disallowed_methods)]` expected. If one is truly needed, scope it narrowly and document it.
- No validation tolerance, threshold or series is changed. A gate failing at its existing tolerance is a finding to report, not something to absorb.
- Every re-pin carries a note stating what moved and by roughly how much, in the style of the notes above `CHART_CHECKSUM` in `crates/pleiades-core/src/chart/bit_identity_tests.rs`, beginning `Issue #288 re-pinned it from \`<old>\`:`.
- `tools/se-*` (excluded from the workspace) are out of scope.
- `rustfmt`-clean before every commit (`cargo fmt --all`).
- Work in an isolated worktree branched from current `origin/main` (`git fetch && git reset --hard origin/main` if the new branch is stale). Commit the spec and this plan on the branch.
- Unset `PLEIADES_*_KERNEL` / `PLEIADES_OBJECT_SPK_DIR` for ordinary test tiers. Set them only for the artifact regeneration in Task 5.

## Review Focus

1. **Lint completeness:** every `std` `f64` method that lowers to a libm call is on the list, including the rare ones (`exp2`, `log`, `asinh`, `acosh`, `atanh`, `ln_1p`, `exp_m1`). Both `x.sin()` and `f64::sin(x)` forms are caught. Pinned by the probe in Task 1 Step 3.
2. **Validation accuracy at existing tolerances:** every SE/JPL corpus gate, crossings, aspects (both gates), stations, eclipses, houses and ayanamsa still passes untouched. Pinned by the nightly dispatch in Task 7.
3. **Special inputs:** `atan2(±0, ±0)`, `asin`/`acos` at exactly ±1, and NaN/inf propagation behave as before, because the existing fail-closed/non-finite tests must pass with no edits. Any edit to such a test is a finding (Task 6 Step 4).
4. **Tan-heavy house systems:** `tan` measured 16% slower in isolation. The per-workload numbers from `mise run benchmark` must show whether house computation regressed (Task 7 Step 2).
5. **Published-crate packaging:** every crate that now depends on `libm` still packages (`mise run package-check`, Task 7 Step 1).

---

### Task 1: Dependency, lint guard and performance baseline

**Files:**
- Modify: `Cargo.toml` (`[workspace.dependencies]`)
- Create: `clippy.toml`
- Modify: `AGENTS.md` ("Numerical and domain correctness" section)
- Commit: `docs/superpowers/specs/2026-10-23-deterministic-libm-design.md`, `docs/superpowers/plans/2026-10-23-deterministic-libm.md`

**Interfaces:**
- Produces: workspace dependency `libm = "0.2.16"`; the lint `clippy::disallowed_methods` firing on every listed `f64::*` method. Tasks 2–4 make it pass crate by crate. The baseline file `/tmp/…/scratchpad/bench-before.txt` (any scratch path; record it in the task report) is consumed by Task 7.

- [ ] **Step 1: Record the performance baseline before any conversion.** In the worktree at its unmodified base commit, wait until `/proc/loadavg` 1-min < 3. Run `mise run benchmark` three times, saving each output with its loadavg. These are the "before" numbers.

- [ ] **Step 2: Add `libm = "0.2.16"` to `[workspace.dependencies]`** in root `Cargo.toml`, near the existing entries, with a one-line comment: `# Pure-Rust libm: transcendentals independent of the host C library (#288).`

- [ ] **Step 3: Create `clippy.toml`** with a `disallowed-methods` list covering these `f64` paths, each with `reason = "use libm::<fn> (host-libm-independent results, #288)"`:
  `sin cos tan asin acos atan atan2 sin_cos(→libm::sincos) sinh cosh tanh asinh acosh atanh exp exp2 exp_m1(→libm::expm1) ln(→libm::log) log(→ln(x)/ln(base) via libm::log) log2 log10 ln_1p(→libm::log1p) powf(→libm::pow) cbrt hypot`.
  Verify the list against `std`'s `f64` docs for 1.99.1 and add any other method documented as calling the platform libm. Confirm both call forms are caught by running clippy on a throwaway scratch crate (outside the repo) containing `x.sin()` and `f64::sin(x)`.

- [ ] **Step 4: Run the lint to capture the inventory (the "failing test").**
  Run: `cargo clippy --workspace --all-targets --all-features 2>&1 | grep -c "disallowed method"`
  Expected: several hundred hits (the grep estimate is ~807). Record the number and the per-crate split in the task report.

- [ ] **Step 5: Add one bullet to `AGENTS.md` → "Numerical and domain correctness":** `- Call transcendentals through \`libm::\` (\`libm::sin(x)\`, not \`x.sin()\`); \`clippy.toml\` enforces it so results do not depend on the host C library (#288).`

- [ ] **Step 6: Commit** spec, plan, `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `AGENTS.md`:
  `git commit -m "build: add libm workspace dependency and disallowed-methods guard (#288)"`

### Task 2: Convert the domain and foundation crates

**Files:** every `.rs` under `crates/pleiades-types`, `pleiades-time`, `pleiades-apparent`, `pleiades-houses`, `pleiades-apsides`, `pleiades-fict` (and `pleiades-ayanamsa` if clippy flags it), plus their `Cargo.toml`s.

**Interfaces:**
- Consumes: Task 1's workspace dep and lint.
- Produces: these crates are clippy-clean under the lint. No public API changes: signatures stay the same, only bodies change.

- [ ] **Step 1: Confirm the lint fails for these crates.**
  Run: `cargo clippy -p pleiades-types -p pleiades-time -p pleiades-apparent -p pleiades-houses -p pleiades-apsides -p pleiades-fict --all-targets --all-features -- -D warnings`
  Expected: FAIL with `disallowed method` errors.

- [ ] **Step 2: Add `libm.workspace = true`** to each crate's `[dependencies]` (or `[dev-dependencies]` if only test code needs it). Rewrite each flagged call mechanically: `x.sin()` → `libm::sin(x)`, `y.atan2(x)` → `libm::atan2(y, x)` (keep receiver = first argument), `x.sin_cos()` → `libm::sincos(x)` (same `(sin, cos)` order), `x.powf(y)` → `libm::pow(x, y)`, `x.ln()` → `libm::log(x)`, `x.hypot(y)` → `libm::hypot(x, y)`. Don't otherwise restructure expressions; keep evaluation order identical.

- [ ] **Step 3: Run the lint again.** Same command as Step 1. Expected: PASS.

- [ ] **Step 4: Run these crates' tests.**
  Run: `cargo nextest run -p pleiades-types -p pleiades-time -p pleiades-apparent -p pleiades-houses -p pleiades-apsides -p pleiades-fict`
  Expected: PASS, except tests that pin exact bits (checksums, bit-pattern boundary inputs). List each failure with its name and whether it's a pin or an oracle change. Pins are re-pinned in Task 6, not here. A failure that isn't a pin (a tolerance or behavior assertion) is a finding: stop and report it.

- [ ] **Step 5: `cargo fmt --all` and commit**: `git commit -m "refactor: route domain-crate transcendentals through libm (#288)"`

### Task 3: Convert the backend and data crates

**Files:** `crates/pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-compression`, `pleiades-data` (including `src/regenerate.rs` and the compression frame recombination), plus their `Cargo.toml`s.

**Interfaces:** Consumes Task 1. Produces the same guarantee as Task 2 for these crates.

- [ ] **Step 1:** Same as Task 2 Step 1 with `-p pleiades-vsop87 -p pleiades-elp -p pleiades-jpl -p pleiades-compression -p pleiades-data`. Expected: FAIL.
- [ ] **Step 2:** Same mechanical rewrite and dependency entries as Task 2 Step 2.
- [ ] **Step 3:** Re-run the lint. Expected: PASS.
- [ ] **Step 4:** `cargo nextest run` for these crates. Expected: PASS except pins (listed as in Task 2 Step 4). `pleiades-jpl/tests/corpus_regen.rs` is tolerance-based and must not need edits.
- [ ] **Step 5:** `cargo fmt --all`; commit `refactor: route backend and data transcendentals through libm (#288)`.

### Task 4: Convert the event, eclipse, façade and tooling crates (and fuzz)

**Files:** `crates/pleiades-events`, `pleiades-eclipse`, `pleiades-core`, `pleiades-cli`, `pleiades-validate`, any remaining crate clippy flags, and `fuzz/` if `grep -rnE "\.(sin|cos|tan|atan2|asin|acos)\(" fuzz/fuzz_targets` finds calls.

**Interfaces:** Consumes Task 1. Produces a workspace clippy-clean under the lint.

- [ ] **Step 1:** Lint these crates. Expected: FAIL.
- [ ] **Step 2:** Same mechanical rewrite as Task 2 Step 2. For `fuzz/`, add `libm = "0.2.16"` directly to `fuzz/Cargo.toml`, since it's a separate workspace.
- [ ] **Step 3: Whole-workspace lint gate.**
  Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  Expected: PASS with zero `disallowed method` hits.
- [ ] **Step 4:** `cargo nextest run --workspace --exclude pleiades-validate` and `cargo test -p pleiades-validate -- --include-ignored`. Expected: PASS except pins, listed as in Task 2 Step 4.
- [ ] **Step 5:** `cargo fmt --all`; commit `refactor: route event, façade and tooling transcendentals through libm (#288)`.

### Task 5: Regenerate the packaged artifact

**Files:**
- Modify: `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`
- Modify (only if their tests fail with the new artifact): pinned values in `crates/pleiades-data/src/accuracy_baseline.rs`, `thresholds.rs`, `coverage/` (pinned measurements only, never limits)

**Interfaces:** Consumes Tasks 2–4 (the converted fit path). Produces the committed artifact that Task 6's chart pins are computed from.

- [ ] **Step 1: Run the byte-identity test (expected to fail).** Save a copy of the current artifact to scratch first.
  Run: `PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp cargo test --release -p pleiades-data --test artifact_regen -- --nocapture`
  Expected: FAIL in `regenerated_artifact_matches_committed`. If it unexpectedly passes, note that the artifact didn't move and skip to Step 4.
- [ ] **Step 2: Regenerate.**
  Run: `PLEIADES_DE_KERNEL=… PLEIADES_AST_KERNEL=… cargo run --release -p pleiades-cli -- generate-packaged-artifact --out crates/pleiades-data/tests/fixtures/packaged-artifact.bin` (paths as in Step 1)
- [ ] **Step 3: Re-run the Step 1 command.** Expected: PASS. Also run `cargo test -q -p pleiades-data --lib encoded_artifact_within_size_budget`. Expected: PASS. Do not run `base_bodies_match_the_baseline_artifact`: it guards a different change (body additions) and is expected to differ here.
- [ ] **Step 4: Run `cargo nextest run -p pleiades-data`** with kernel variables unset. Re-pin only *measured* values whose tests fail (accuracy baselines, coverage figures), each with a #288 note giving old → new. If any measured error now exceeds its unchanged limit, stop and report a finding.
- [ ] **Step 5: Record in the task report** how many bytes or segments of the artifact changed (`cmp -l old new | wc -l`) and the largest change in any accuracy figure. Commit: `data: regenerate the packaged artifact with libm transcendentals (#288)`.

### Task 6: Re-pin moved checksums and goldens

**Files** (the starting list from the spec; the failing tests are the authoritative list):
- `crates/pleiades-core/src/chart/bit_identity_tests.rs` (`CHART_CHECKSUM` and its doc note)
- `crates/pleiades-events/tests/fu25_bit_identity.rs` (`EVENTS_CHECKSUM`)
- `crates/pleiades-core/src/compatibility/mod.rs` (`CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` and, per its doc, the profile ID) — only if it fails
- `crates/pleiades-validate/data/crossings-corpus/` golden column and `manifest.txt`, through `cargo run -q -p pleiades-validate -- crossings-golden --regenerate`
- `pleiades-cli` fixture-golden outputs, through `generate-fixture-golden`, only if a test reports them stale
- the bit-pattern boundary inputs (e.g. `crates/pleiades-apsides/src/tests.rs`), only if their tests fail

**Interfaces:** Consumes Tasks 2–5. Produces a fully green test suite.

- [ ] **Step 1: List every failing test.** Run `cargo nextest run --workspace --exclude pleiades-validate --no-fail-fast`, `cargo test -p pleiades-validate -- --include-ignored`, and `mise run release-smoke`. Expected: only the pins listed in Tasks 2–5's reports fail.
- [ ] **Step 2: For each checksum, measure what moved before re-pinning.** Dump the rendered string at the base commit and at HEAD, temporarily asserting against `0` so the message prints it, then revert. Decode the hex bit fields and report the largest |Δ| per quantity (arcseconds for angles, relative for distance, per-day for speeds). Write the #288 note from those numbers, then update the constant.
- [ ] **Step 3: Regenerate goldens** with the commands in the Files list, then run `git diff --stat` and confirm only expected files changed.
- [ ] **Step 4: Fix boundary inputs only where needed.** For a bit-pattern boundary test that now fails, re-craft the input so the test again exercises its boundary, and add a short #288 note. If a test's *expected outcome* would change, or any fail-closed/non-finite test needs an edit, stop and report a finding.
- [ ] **Step 5: Full verification.** Run `mise run ci` and `mise run test-full`. Expected: PASS.
- [ ] **Step 6:** `cargo fmt --all`; commit `test: re-pin checksums and goldens moved by libm (#288)`.

### Task 7: Revert the runner pin, measure, validate, PR

**Files:**
- Modify: `.github/workflows/{ci,nightly,fuzz,mutants,release-plz}.yml` (`runs-on: ubuntu-24.04` → `ubuntu-latest`; delete the two-line pin comment in `ci.yml`; restore the `mutants.yml` env comment to `# ubuntu-latest is a 4-vCPU runner; cargo-mutants reads this natively.`)

**Interfaces:** Consumes everything above.

- [ ] **Step 1: Revert the pin.** Then run `grep -rn "ubuntu-24.04" .github/`. Expected: no output. Run `mise run package-check`. Expected: PASS. Commit: `ci: return to ubuntu-latest now that results don't depend on host libm (#288)`.
- [ ] **Step 2: Performance after.** With loadavg < 3, run `mise run benchmark` three times, alternating with the Task 1 baseline build if it's still available. Compare per workload against Task 1 Step 1. Bar: the aggregate chart/batch figure shows no slowdown beyond run-to-run noise. Any workload more than ~5% slower beyond noise goes in the PR as a finding with numbers. It's not a blocker, and keeping `std::tan` is not an acceptable fix.
- [ ] **Step 3: Push and dispatch nightly on the branch** (`gh workflow run nightly.yml --ref <branch>`) so the full gates run on `ubuntu-latest`: aspects, aspects-lunar, stations, corpus, houses, ayanamsa, apparent, topocentric. Expected: every job green with no tolerance edits. A red gate is a finding.
- [ ] **Step 4: Open the PR** with: what changed, the measured runner-independence (CI green on `ubuntu-latest`, which now resolves to 26.04), each re-pin with what moved, the artifact bytes changed, benchmark before/after with loadavg, and the nightly run ID. Close #288 from the PR body. Watch CI, and merge only once all checks are green (squash, never `--auto`), then delete the branch.
