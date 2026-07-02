# Release-bundle tests: generate once, verify per test

Status: proposed design (2026-07-02). Awaiting review; implementation pending plan.

## Problem

The release-bundle write/verify family regenerates the full release bundle for
every test:

- `pleiades-validate` `tests::release_bundle_verify_a` (50 tests) and
  `tests::release_bundle_verify_b` (122 tests) each run
  `bundle-release --rounds 1` into a fresh temp dir, tamper with one file (or
  one manifest line), and assert `verify-release-bundle` rejects it.
- `pleiades-cli` `cli::tests::release` (4 tests) does the same with 9
  generations at default rounds.

Measured on the development machine (2026-07-02, `opt-level = 2` test profile):

- One test in isolation: **74 s** — almost entirely the one-time per-process
  warm-up (`build_validation_report`, benchmark report, fit analysis), which is
  `OnceLock`/`Mutex`-memoized per process and keyed by `rounds`.
- Marginal cost per additional test in the same process: **~2 s** in isolation
  (bundle file writing + checksumming; a bundle is 13 MB across 143 files).
  `verify-release-bundle` itself is nearly free once the process is warm
  (a generate-plus-two-verifies test times the same as generate alone).
- Full family under default parallelism: **~15 min wall, ~79 CPU-minutes**
  (`cargo test -p pleiades-validate --lib -- --include-ignored release_bundle`,
  172 passed, 906 s). Contention inflates the marginal cost well past the
  isolated ~2 s.

CI (`mise run ci` → `test-full`) runs on a standard `ubuntu-latest` runner, so
those ~79 CPU-minutes dominate CI wall clock. That is the pain point this
design targets (per user: CI wall clock, not the local loop).

One observed full-family run also produced 47 spurious failures that vanished
on a clean rerun — consistent with the family's documented sensitivity to
parallel load (see `2026-06-20-test-speed-findings-addendum.md`). Fewer
generations means less exposure to that class of flake.

## Goal

Generate the pristine bundle once per test process and have every verify test
work from a cheap copy, with **zero change to what any assertion checks** and
no loss of per-test failure granularity.

Non-goals: making the ~74 s warm-up itself cheaper (it is validation work paid
once per process regardless); changing the test runner; caching anything across
processes or CI runs (release validation must genuinely run in CI).

## Alternatives considered

- **Table-driven consolidation** ("all validations in one go" literally): one
  generation, one or a few tests looping a scenario table. Same speed as the
  chosen design (verify cost is identical), but a much larger refactor of
  ~5,300 test lines, loses per-scenario test names in CI output, and serializes
  currently-parallel scenarios. Rejected.
- **Cheapen generation itself** (attack the warm-up): touches production
  validation code and risks weakening what release validation measures, for a
  cost that is paid once per process anyway. Rejected.

## Design

### Fixture (`pleiades-validate/src/tests/test_support.rs`)

```rust
pub(crate) struct PristineBundle {
    pub dir: PathBuf,        // pristine bundle directory; never mutated
    pub rendered: String,    // CLI output of the one bundle-release run
    pub bundle: ReleaseBundle, // returned struct (ReleaseBundle is Clone)
}

pub(crate) fn pristine_release_bundle() -> &'static PristineBundle
```

Backed by a `OnceLock`. Initialization runs
`render_cli(["bundle-release", "--out", <pristine dir>, "--rounds", "1"])`
exactly once per test process; concurrent tests block on the `OnceLock` until
it is ready (they already block on the memoization mutex today). If generation
fails, the fixture panics with a clear message and every dependent test fails
loudly — same failure surface as today. The pristine dir lives in the OS temp
dir and is intentionally not deleted (process-scoped; OS temp cleanup applies).

```rust
pub(crate) fn stage_bundle_copy(prefix: &str) -> PathBuf
```

Creates `unique_temp_dir(prefix)` and recursively copies the pristine dir into
it (13 MB / 143 files — tens of milliseconds). Tests tamper with the copy and
keep their existing per-test `remove_dir_all` cleanup.

### Migration

1. The seven shared helpers in `test_support.rs`
   (`assert_release_bundle_rejects_tampered_text_file`,
   `..._semantically_tampered_text_file_with_updated_checksum`,
   `..._symlinked_text_file`, `..._missing_manifest_entry`,
   `..._blank_manifest_value`, `..._duplicate_manifest_entry`,
   `..._whitespace_manifest_entry`) replace their inline `bundle-release`
   call with `stage_bundle_copy`. This alone converts ~120 tests.
2. The ~45 tests that inline-generate replace their
   `render_cli(["bundle-release", ...])` arrange block with
   `stage_bundle_copy`. Assertions unchanged.
3. Tests that assert generation itself consume the fixture instead of
   regenerating:
   - `release_bundle_writes_expected_artifacts` asserts against the fixture's
     `rendered` string and reads the pristine dir (read-only — it tampers with
     nothing).
   - The `release_bundle_validate_*` family clones the fixture's
     `ReleaseBundle` struct, mutates the clone in memory, and calls
     `.validate()` as today.
4. Exactly one test keeps a real second generation:
   `release_bundle_commands_accept_output_aliases_in_the_validation_front_end`
   exercises `bundle-release` argument parsing, so it must invoke the command
   itself (~2 s marginal once warm). Its mixed-alias rejection cases fail in
   the front end before generation and stay as-is.
5. Same pattern in `pleiades-cli/src/cli/tests/release.rs` (its own process):
   one fixture generation replaces the 9 current generations, except that its
   `--output`-alias front-end test keeps a real generation for the same reason
   as in step 4. Those tests use default `--rounds`; the CLI fixture keeps
   default rounds so existing assertions are untouched.

### Invariants

- The pristine dir is never handed to a test that mutates files; only
  `stage_bundle_copy` copies are tampered with. The only direct pristine-dir
  consumer is the read-only `writes_expected_artifacts` path.
- No assertion text, expected-error fragment, or test name changes.
- `verify-release-bundle` semantic checks compare against in-process memoized
  posture, so verifying a copied bundle in the same process is exactly as
  strict as verifying a freshly generated one.

## Expected outcome

- `pleiades-validate` release-bundle family: ~15 min wall / ~79 CPU-min →
  roughly 2–3 min wall (dominated by the one warm-up, which the rest of the
  suite pays anyway) and single-digit CPU-minutes.
- `pleiades-cli` release tests: 9 generations → 1–2.
- CI `test-full` wall clock drops by roughly the same amount.

## Verification

- `cargo test -p pleiades-validate --lib -- --include-ignored release_bundle`
  passes 172/172; re-time and record before/after in the implementation PR.
- `cargo test -p pleiades-cli -- --include-ignored release` passes.
- `mise run test-full` passes.

## Accepted risk

Today 172 independent generations would (accidentally) surface generation
nondeterminism as flakes. With one generation, that signal narrows to the
determinism/checksum assertions that already exist. Given the family's history
of load-induced false positives, trading that accidental coverage for
reliability and ~13 min of CI wall clock is the right call.
