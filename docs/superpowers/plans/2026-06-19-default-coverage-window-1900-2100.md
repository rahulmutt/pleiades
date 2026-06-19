# Default Coverage Window 1900–2100 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a 1900–2100 CE default artifact (smaller, faster tests) while letting power users generate wider ranges via a public API + thin CLI.

**Architecture:** The shipped artifact's major bodies are fit densely from the de440 kernel over a window; that window is two constants in `corpus_spec.rs`. Flip them to 1900–2100, parameterize the kernel generation entry point so callers can pass any `CoverageWindow`, retire the dead snapshot-reconstruction major-body path, then regenerate all derived committed data (corpus slices, artifact bytes, golden baseline) from the kernel.

**Tech Stack:** Rust (workspace edition/MSRV as configured), JPL DE440 SPK kernel, existing `pleiades-cli` subcommand dispatch, existing `generate_slice` corpus toolchain.

## Global Constraints

- The de440 kernel path is supplied via env var `PLEIADES_DE_KERNEL`; the asteroid Tier-A regen also needs `PLEIADES_AST_KERNEL`. Gated tests skip without them.
- New default major-body window: `RANGE_START_JD = 2_415_020.5` (1900-01-01), `RANGE_END_JD = 2_488_069.5` (2100-01-01) — exactly the existing `AST_RANGE_*` values.
- Byte-identity (`regenerated_artifact_matches_committed`) is the correctness anchor — the committed artifact bytes MUST equal what the kernel regenerates.
- Do NOT thread the window through `corpus_spec` epoch functions — they read the constants and auto-narrow on the flip. The window parameter flows ONLY through the generation API (`build_packaged_artifact_from_reference_over` already accepts it).
- Eros / asteroid generation is independent of the major-body window (it reads `reference_snapshot.csv` Eros rows over the fixed `AST_RANGE_*`). A wider user window widens only major bodies.
- Add no new third-party dependencies.
- Commit after every task.

---

### Task 1: `CoverageWindow` type + capture pre-change baseline

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (after line 23)
- Re-export: `crates/pleiades-data/src/regenerate.rs` (use site) — none yet, just ensure path is reachable

**Interfaces:**
- Produces: `pleiades_jpl::spk::corpus_spec::CoverageWindow { start_jd: f64, end_jd: f64 }`, with `CoverageWindow::default() -> Self`, `CoverageWindow::new(start_jd: f64, end_jd: f64) -> Self`, `CoverageWindow::from_years(start_year: i32, end_year: i32) -> Self`, and `fn as_tuple(self) -> (f64, f64)`.

- [ ] **Step 1: Capture the pre-change size/perf baseline**

Run: `cargo test -p pleiades-data --test artifact_regen sp1_draft_size_perf_baseline -- --nocapture`
Record the printed `artifact_size`, `decode`, and `lookup` numbers in a scratch note — they become the "Before" column in the spec/PLAN baseline table (Task 8).

- [ ] **Step 2: Write the failing test for `CoverageWindow`**

Add to the bottom of `crates/pleiades-jpl/src/spk/corpus_spec.rs`:

```rust
#[cfg(test)]
mod coverage_window_tests {
    use super::*;

    #[test]
    fn default_window_is_the_range_constants() {
        let w = CoverageWindow::default();
        assert_eq!(w.start_jd, RANGE_START_JD);
        assert_eq!(w.end_jd, RANGE_END_JD);
        assert_eq!(w.as_tuple(), (RANGE_START_JD, RANGE_END_JD));
    }

    #[test]
    fn from_years_maps_to_jan_1_jd() {
        // 2000-01-01 = JD 2_451_544.5 (midnight); J2000 epoch (noon) is 2_451_545.0.
        let w = CoverageWindow::from_years(2000, 2001);
        assert!((w.start_jd - 2_451_544.5).abs() < 1e-6);
        assert!((w.end_jd - 2_451_909.5).abs() < 1e-6);
    }
}
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cargo test -p pleiades-jpl coverage_window_tests`
Expected: FAIL — `cannot find type CoverageWindow`.

- [ ] **Step 4: Implement `CoverageWindow`**

Insert after line 23 of `crates/pleiades-jpl/src/spk/corpus_spec.rs`:

```rust
/// A major-body coverage window in TDB Julian Days. The packaged artifact ships
/// `CoverageWindow::default()` (1900–2100); the kernel generation API accepts any
/// window so callers can build wider artifacts for themselves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoverageWindow {
    pub start_jd: f64,
    pub end_jd: f64,
}

impl CoverageWindow {
    pub fn new(start_jd: f64, end_jd: f64) -> Self {
        Self { start_jd, end_jd }
    }

    /// Build from calendar years, each at Jan 1 00:00 TDB. Uses the standard
    /// Gregorian JD-at-midnight formula; year is the proleptic Gregorian year.
    pub fn from_years(start_year: i32, end_year: i32) -> Self {
        Self {
            start_jd: jan1_midnight_jd(start_year),
            end_jd: jan1_midnight_jd(end_year),
        }
    }

    pub fn as_tuple(self) -> (f64, f64) {
        (self.start_jd, self.end_jd)
    }
}

impl Default for CoverageWindow {
    fn default() -> Self {
        Self {
            start_jd: RANGE_START_JD,
            end_jd: RANGE_END_JD,
        }
    }
}

/// Julian Day at Jan 1 00:00 of `year` (proleptic Gregorian). Standard algorithm
/// (Fliegel–Van Flandern) specialised to month=1, day=1.
fn jan1_midnight_jd(year: i32) -> f64 {
    let a = (14 - 1) / 12; // = 0 for January
    let y = year + 4800 - a;
    let m = 1 + 12 * a - 3;
    let jdn = 1 // day
        + (153 * m + 2) / 5
        + 365 * y
        + y / 4
        - y / 100
        + y / 400
        - 32045;
    jdn as f64 - 0.5
}
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test -p pleiades-jpl coverage_window_tests`
Expected: PASS (both tests).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): add CoverageWindow type for parameterized generation"
```

---

### Task 2: Parameterized kernel generation API

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs:2499-2534` (the kernel entry point + `build_packaged_artifact_from_reference`)
- Test: `crates/pleiades-data/src/tests/coverage.rs` (mirror the existing synthetic-window test near `fit_segment_within_span_reproduces_a_smooth_synthetic_body`, line 2679)

**Interfaces:**
- Consumes: `pleiades_jpl::spk::corpus_spec::CoverageWindow` (Task 1); existing `build_packaged_artifact_from_reference_over(reference: &dyn EphemerisBackend, base_window: (f64, f64)) -> CompressedArtifact`.
- Produces: `pub fn regenerate_packaged_artifact_from_kernel_over(kernel_path: &str, window: CoverageWindow) -> Result<CompressedArtifact, String>`; `regenerate_packaged_artifact_from_kernel(kernel_path: &str) -> Result<CompressedArtifact, String>` becomes a wrapper passing `CoverageWindow::default()`.

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-data/src/tests/coverage.rs` (use the same synthetic backend the existing test at line 2679 uses):

```rust
#[test]
fn default_window_artifact_matches_explicit_default_over() {
    use pleiades_jpl::spk::corpus_spec::CoverageWindow;
    // A tiny synthetic window keeps this in milliseconds; assert the public
    // window-parameterized builder and the default builder agree for the same window.
    let reference = smooth_synthetic_backend(); // existing test helper in this module
    let window = CoverageWindow::new(2_451_545.0, 2_451_545.0 + 40.0);
    let a = crate::regenerate::build_packaged_artifact_from_reference_over(
        &reference,
        window.as_tuple(),
    );
    let b = crate::regenerate::build_packaged_artifact_from_reference_over(
        &reference,
        (2_451_545.0, 2_451_545.0 + 40.0),
    );
    assert_eq!(a.encode().unwrap(), b.encode().unwrap());
}
```

If `smooth_synthetic_backend()` is not the exact helper name in this module, use the same construction the existing `fit_segment_within_span_reproduces_a_smooth_synthetic_body` test (line 2679) uses to obtain a `&dyn EphemerisBackend`.

- [ ] **Step 2: Run the test to verify it fails or passes trivially**

Run: `cargo test -p pleiades-data default_window_artifact_matches_explicit_default_over`
Expected: compiles and PASSES (this guards that the `_over` plumbing is the single path). If it fails to compile because the helper name differs, fix the helper reference and re-run.

- [ ] **Step 3: Add the public window-parameterized entry point**

Replace `regenerate_packaged_artifact_from_kernel` (lines 2506-2514) and `build_packaged_artifact_from_reference` (2528-2534) with:

```rust
/// Regenerates the packaged artifact from a de440 SPK kernel over an explicit
/// coverage window. Major bodies are fit densely from the kernel across `window`;
/// the constrained asteroid (Eros) is always sourced from its fixed 1900–2100
/// corpus data and is unaffected by `window`.
pub fn regenerate_packaged_artifact_from_kernel_over(
    kernel_path: &str,
    window: pleiades_jpl::spk::corpus_spec::CoverageWindow,
) -> Result<CompressedArtifact, String> {
    let backend = pleiades_jpl::SpkBackend::builder()
        .add_kernel(kernel_path)
        .map_err(|error| error.message)?
        .build();
    Ok(build_packaged_artifact_from_reference_over(&backend, window.as_tuple()))
}

/// Regenerates the packaged artifact from a de440 SPK kernel over the shipped
/// default window (1900–2100).
pub fn regenerate_packaged_artifact_from_kernel(
    kernel_path: &str,
) -> Result<CompressedArtifact, String> {
    regenerate_packaged_artifact_from_kernel_over(
        kernel_path,
        pleiades_jpl::spk::corpus_spec::CoverageWindow::default(),
    )
}
```

Delete the now-unused `build_packaged_artifact_from_reference` wrapper (2528-2534) and update its single caller (if any) to call `..._over(reference, CoverageWindow::default().as_tuple())`. Confirm callers with:

Run: `grep -rn "build_packaged_artifact_from_reference\b" crates/`

- [ ] **Step 4: Run the test + build to verify**

Run: `cargo test -p pleiades-data default_window_artifact_matches_explicit_default_over && cargo build -p pleiades-data`
Expected: PASS, clean build.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/coverage.rs
git commit -m "feat(data): public window-parameterized kernel generation API"
```

---

### Task 3: `generate-artifact` CLI subcommand

**Files:**
- Create: `crates/pleiades-cli/src/commands/generate_artifact.rs`
- Modify: `crates/pleiades-cli/src/commands/mod.rs` (register module)
- Modify: `crates/pleiades-cli/src/cli.rs:21` (use) and `:832`-area dispatch (`Some("generate-artifact") => ...`)
- Modify: `crates/pleiades-cli/src/help.rs` (add help lines)

**Interfaces:**
- Consumes: `CoverageWindow::new` / `from_years` (Task 1); `regenerate_packaged_artifact_from_kernel_over` (Task 2).
- Produces: `pub fn render_generate_artifact(args: &[&str]) -> Result<String, String>`.

- [ ] **Step 1: Write failing arg-wiring tests**

Create `crates/pleiades-cli/src/commands/generate_artifact.rs`:

```rust
//! `generate-artifact` command: regenerate the packaged artifact from a de440
//! kernel over a chosen coverage window and write the encoded bytes to a file.
//!
//! Usage:
//!   generate-artifact <kernel.bsp> --out <path> [--start <year|JD>] [--end <year|JD>]
//!
//! `--start`/`--end` accept a calendar year (e.g. 1850) or a Julian Day (a value
//! with a decimal point, e.g. 2451545.0). Omitted bounds default to the shipped
//! 1900–2100 window. Major-body generation requires the kernel (dense de440 fit).

use pleiades_data::regenerate_packaged_artifact_from_kernel_over;
use pleiades_jpl::spk::corpus_spec::CoverageWindow;

/// Parse a `--start`/`--end` token: a value containing '.' is a JD; otherwise a
/// calendar year converted to Jan 1 00:00 TDB JD.
fn parse_bound(token: &str) -> Result<Option<i32>, String> {
    // Returns Ok(Some(year)) for a year token, Ok(None) handled by caller for JD.
    token
        .parse::<i32>()
        .map(Some)
        .map_err(|_| format!("bad year/JD bound: {token}"))
}

pub fn render_generate_artifact(args: &[&str]) -> Result<String, String> {
    let kernel = args
        .first()
        .ok_or("generate-artifact requires a kernel path")?;

    let mut out: Option<&str> = None;
    let mut start: Option<f64> = None;
    let mut end: Option<f64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i] {
            "--out" => {
                out = Some(args.get(i + 1).ok_or("--out requires a path")?);
                i += 2;
            }
            "--start" => {
                start = Some(parse_bound_jd(args.get(i + 1).ok_or("--start requires a value")?)?);
                i += 2;
            }
            "--end" => {
                end = Some(parse_bound_jd(args.get(i + 1).ok_or("--end requires a value")?)?);
                i += 2;
            }
            other => return Err(format!("unknown generate-artifact arg: {other}")),
        }
    }

    let out = out.ok_or("generate-artifact requires --out <path>")?;
    let default = CoverageWindow::default();
    let window = CoverageWindow::new(
        start.unwrap_or(default.start_jd),
        end.unwrap_or(default.end_jd),
    );
    if window.end_jd <= window.start_jd {
        return Err("coverage window end must be after start".to_string());
    }

    let artifact = regenerate_packaged_artifact_from_kernel_over(kernel, window)?;
    let bytes = artifact.encode().map_err(|e| format!("encode: {e}"))?;
    let len = bytes.len();
    std::fs::write(out, &bytes).map_err(|e| format!("write {out}: {e}"))?;
    Ok(format!(
        "wrote {len} bytes to {out} (window {}..{} JD)",
        window.start_jd, window.end_jd
    ))
}

/// A token with a '.' is treated as a JD; otherwise as a calendar year.
fn parse_bound_jd(token: &str) -> Result<f64, String> {
    if token.contains('.') {
        token.parse::<f64>().map_err(|_| format!("bad JD: {token}"))
    } else {
        let year = parse_bound(token)?.expect("year token");
        Ok(CoverageWindow::from_years(year, year + 1).start_jd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_kernel_errors() {
        assert!(render_generate_artifact(&[]).is_err());
    }

    #[test]
    fn missing_out_errors() {
        let err = render_generate_artifact(&["/no/such/kernel.bsp"]).unwrap_err();
        assert!(err.contains("--out"), "unexpected: {err}");
    }

    #[test]
    fn year_token_parses_to_jan1_jd() {
        let jd = parse_bound_jd("2000").unwrap();
        assert!((jd - 2_451_544.5).abs() < 1e-6);
    }

    #[test]
    fn jd_token_parses_as_jd() {
        let jd = parse_bound_jd("2451545.0").unwrap();
        assert!((jd - 2_451_545.0).abs() < 1e-9);
    }

    #[test]
    fn inverted_window_errors() {
        let err = render_generate_artifact(&[
            "/no/such/kernel.bsp", "--out", "/tmp/x.bin", "--start", "2100", "--end", "1900",
        ])
        .unwrap_err();
        // Either the inverted-window guard or the kernel-load failure is acceptable,
        // but a real kernel would hit the guard; with a fake path the kernel load
        // fails first. Assert non-empty to keep the test kernel-free.
        assert!(!err.is_empty());
    }
}
```

- [ ] **Step 2: Register the module and dispatch**

In `crates/pleiades-cli/src/commands/mod.rs` add: `pub mod generate_artifact;`
In `crates/pleiades-cli/src/cli.rs` near line 21 add: `use crate::commands::generate_artifact::render_generate_artifact;`
In the dispatch match near line 832 add: `Some("generate-artifact") => render_generate_artifact(&args[1..]),`
In `crates/pleiades-cli/src/help.rs` after the `generate-spk-corpus` lines add:

```text
  generate-artifact <kernel.bsp> --out <path> [--start <year|JD>] [--end <year|JD>]  Regenerate the packaged artifact over a coverage window (default 1900-2100)
```

- [ ] **Step 3: Run the tests to verify they pass**

Run: `cargo test -p pleiades-cli generate_artifact`
Expected: PASS (5 tests).

- [ ] **Step 4: Build the whole workspace**

Run: `cargo build`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/commands/generate_artifact.rs crates/pleiades-cli/src/commands/mod.rs crates/pleiades-cli/src/cli.rs crates/pleiades-cli/src/help.rs
git commit -m "feat(cli): generate-artifact subcommand for custom coverage windows"
```

---

### Task 4: Retire the legacy snapshot major-body path

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs:146-195` (`packaged_body_artifacts_from_snapshot`), `:96-126` (snapshot regen entry points)
- Delete test: `crates/pleiades-data/src/tests/codec.rs` (`packaged_artifact_generation_from_supplied_snapshot_matches_the_default_fixture`, ~line 134)

**Interfaces:**
- Produces: `packaged_body_artifacts_from_snapshot` now emits artifacts ONLY for constrained-asteroid bodies; major bodies are not reconstructed from the snapshot anywhere.

- [ ] **Step 1: Delete the stale ignored byte-identity test**

Remove the `#[ignore]`d `packaged_artifact_generation_from_supplied_snapshot_matches_the_default_fixture` test in `crates/pleiades-data/src/tests/codec.rs` (it predates dense de440 generation and cannot pass).

- [ ] **Step 2: Write a test pinning the new restricted behavior**

Add to `crates/pleiades-data/src/tests/codec.rs`:

```rust
#[test]
fn snapshot_reconstruction_covers_only_constrained_asteroids() {
    use pleiades_jpl::reference_snapshot;
    let snapshot = reference_snapshot();
    let artifact =
        crate::regenerate::try_regenerate_packaged_artifact_from_snapshot(snapshot).unwrap();
    // Major bodies are no longer reconstructed from the snapshot; only the
    // constrained asteroid (Eros) is present.
    let bodies: Vec<_> = artifact.bodies.iter().map(|b| b.body.clone()).collect();
    assert!(bodies.iter().any(|b| matches!(b, pleiades_backend::CelestialBody::Custom(_))));
    assert!(
        !bodies.contains(&pleiades_backend::CelestialBody::Sun),
        "major bodies must not come from the snapshot path"
    );
}
```

(If `CompressedArtifact` exposes bodies under a different accessor than `.bodies`, use that accessor — confirm with `grep -n "pub fn bodies\|pub bodies" crates/pleiades-compression/src/*.rs`.)

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p pleiades-data snapshot_reconstruction_covers_only_constrained_asteroids`
Expected: FAIL (major bodies currently present).

- [ ] **Step 4: Restrict the snapshot path to asteroids**

In `packaged_body_artifacts_from_snapshot` (regenerate.rs:160), skip non-asteroid bodies. Replace the body loop guard so only `SelectedAsteroids` / `CustomBodies` cadence bodies are reconstructed:

```rust
for (body_index, body) in packaged_bodies().iter().cloned().enumerate() {
    use crate::coverage::{packaged_artifact_body_cadence, PackagedArtifactBodyCadence};
    if !matches!(
        packaged_artifact_body_cadence(&body),
        PackagedArtifactBodyCadence::SelectedAsteroids | PackagedArtifactBodyCadence::CustomBodies
    ) {
        continue; // major bodies are fit from the kernel, never from the snapshot
    }
    let Some(mut entries) = entries_by_body.remove(&body) else {
        continue;
    };
    // ... unchanged body-reconstruction below ...
```

- [ ] **Step 5: Run the test + suite to verify**

Run: `cargo test -p pleiades-data`
Expected: PASS (new test green; nothing else regresses — gated tests skip without the kernel).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/codec.rs
git commit -m "refactor(data): retire dead snapshot major-body reconstruction path"
```

---

### Task 5: Flip the window to 1900–2100 and regenerate all derived data (GATED — needs `PLEIADES_DE_KERNEL`)

This is the atomic narrowing cut: flipping the constants invalidates every piece of derived committed data at once, so the regenerations live in one task that ends fully green.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs:9-10` (the two constants)
- Regenerate (data): `crates/pleiades-jpl/data/corpus/{boundary,interior,fast_clusters,holdout}.csv`, `manifest.txt`; the packaged artifact `.bin` bytes
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs:380-432` (golden buckets)
- Possibly modify: `crates/pleiades-jpl/data/corpus/fixture_golden.csv` (if its epochs leave the window)
- Possibly fix in-code unit-test expectations that assumed 1600/2600 (search below)

- [ ] **Step 1: Flip the constants**

In `crates/pleiades-jpl/src/spk/corpus_spec.rs:9-10`:

```rust
/// Target packaged range, as TDB Julian Days (1900-01-01 .. 2100-01-01). Matches
/// the asteroid window; wider artifacts are user-generated via `generate-artifact`.
pub const RANGE_START_JD: f64 = 2_415_020.5;
pub const RANGE_END_JD: f64 = 2_488_069.5;
```

Update the doc comment on line 8 accordingly.

- [ ] **Step 2: Fix in-code unit-test expectations that hardcode 1600/2600**

Run: `grep -rn "1600\|2600\|2305447\|2670690" crates --include=*.rs`
For each hit in a non-gated unit test that asserts the old span (e.g. comments/fixtures in `crates/pleiades-jpl/tests/ingest_end_to_end.rs`), update the expectation to the new window. Do NOT touch prose docs yet (Task 7). Then:

Run: `cargo test -p pleiades-jpl corpus_spec`
Expected: the in-range/backbone unit tests pass against the new constants (they assert relative to `RANGE_*`, so most pass unchanged).

- [ ] **Step 3: Regenerate the corpus slices from the kernel**

Run:
```bash
cargo run -p pleiades-cli -- generate-spk-corpus "$PLEIADES_DE_KERNEL" --emit-slices crates/pleiades-jpl/data/corpus
```
Expected: rewrites `boundary.csv`, `interior.csv`, `fast_clusters.csv`, `holdout.csv`, `manifest.txt`. `interior.csv` shrinks substantially (it was ~1.5 MB over 1600–2600).

If the command errors that `fixture_golden.csv` epochs are stale/out of window, regenerate it first:
```bash
cargo run -p pleiades-cli -- generate-fixture-golden crates/pleiades-jpl/data/corpus
```
then re-run the `--emit-slices` command.

- [ ] **Step 4: Verify corpus regeneration matches (gated)**

Run: `PLEIADES_DE_KERNEL="$PLEIADES_DE_KERNEL" cargo test -p pleiades-jpl --test corpus_regen regenerated_corpus_matches_checked_in -- --nocapture`
Expected: PASS — the just-written CSVs match `generate_slice` output within 1 km.

- [ ] **Step 5: Regenerate the packaged artifact bytes from the kernel**

Use the new CLI subcommand to write the committed artifact at the default window. Confirm the committed artifact's path first:

Run: `grep -rn "packaged_artifact_bytes\|include_bytes!" crates/pleiades-data/src/data.rs`
Then regenerate to that exact path:
```bash
cargo run -p pleiades-cli -- generate-artifact "$PLEIADES_DE_KERNEL" --out <committed-artifact-path-from-grep>
```

- [ ] **Step 6: Verify byte-identity (gated)**

Run: `PLEIADES_DE_KERNEL="$PLEIADES_DE_KERNEL" cargo test -p pleiades-data --test artifact_regen regenerated_artifact_matches_committed -- --nocapture`
Expected: PASS — "byte-identical (N bytes)", N markedly smaller than before.

- [ ] **Step 7: Add a golden-baseline print helper**

Add to `crates/pleiades-data/src/accuracy_baseline.rs` test module:

```rust
#[test]
#[ignore = "maintainer helper: prints the accuracy baseline summary to regenerate the golden"]
fn print_packaged_artifact_baseline_summary() {
    eprintln!("{}", packaged_artifact_accuracy_baseline_summary_for_report());
}
```

- [ ] **Step 8: Recompute and update the golden buckets**

Run: `cargo test -p pleiades-data print_packaged_artifact_baseline_summary -- --ignored --nocapture`
Copy the printed per-body buckets into the assertions in `packaged_artifact_baseline_summary_matches_committed_golden` (lines ~391-431), replacing the old `max_lon=...` expected strings. Keep the non-vacuity anchor semantics (inner bodies sub-arcsecond; one outer body deliberately large).

- [ ] **Step 9: Run the accuracy baseline tests**

Run: `cargo test -p pleiades-data accuracy_baseline`
Expected: `packaged_artifact_baseline_is_non_vacuous` and `..._matches_committed_golden` PASS.

- [ ] **Step 10: Full workspace test (gated on)**

Run: `PLEIADES_DE_KERNEL="$PLEIADES_DE_KERNEL" cargo test`
Expected: green. Investigate any remaining failure (likely a prose/summary assertion handled in Task 6/7).

- [ ] **Step 11: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs crates/pleiades-jpl/data/corpus/ crates/pleiades-data/src/accuracy_baseline.rs <committed-artifact-path>
git commit -m "feat(data): narrow default coverage window to 1900-2100; regenerate artifact, corpus, golden"
```

---

### Task 6: Prune `reference_snapshot.csv` major-body rows + summary constants (GATED)

The major-body snapshot rows at out-of-window epochs (1500/1600/1749/1800/2500/2600/2634) are now used only by provenance summaries, not generation. Prune them so the summary stays consistent. Eros / asteroid rows are untouched.

**Files:**
- Modify: `crates/pleiades-jpl/data/reference_snapshot.csv`
- Modify: `crates/pleiades-jpl/src/backend.rs:1549-1574` (the out-of-window `REFERENCE_SNAPSHOT_*_EPOCH_JD` constants)
- Modify: `crates/pleiades-jpl/src/reference_summary/reference_snapshot/core/general_a.rs` (and `general_b.rs`) consumers of those constants
- Modify: `crates/pleiades-data/src/regenerate.rs:43-56` if `reference_snapshot_summary().validate()` enforces removed epochs

- [ ] **Step 1: Run the summary tests to see what enforces the old epochs**

Run: `cargo test -p pleiades-jpl reference_snapshot`
Record which assertions reference 1500/1600/1749/1800/2500/2600/2634 epochs.

- [ ] **Step 2: Prune out-of-window major-body rows**

Remove rows from `crates/pleiades-jpl/data/reference_snapshot.csv` whose epoch is a major body AND outside `[2_415_020.5, 2_488_069.5]`. Keep: all J2000-cluster rows (2451545–2453000.5), the 1900 (2415020.5) anchor, and ALL asteroid rows (Eros/Apophis/Ceres/Juno/Pallas/Vesta) regardless of their epochs (asteroid coverage is its own window).

- [ ] **Step 3: Remove the now-orphaned constants and their uses**

Delete `REFERENCE_SNAPSHOT_1500_*`, `_1600_*`, `_1749_*`, `_1800_*`, `_2500_*` (2_634_167.0), `_2600000_*`, and `_2400000_*` if they fall outside the new window, in `backend.rs:1549-1574`, and remove their references in `general_a.rs` / `general_b.rs` summary filters. Keep `_1900_*` (2_415_020.5), `_2200_*` only if ≤ 2_488_069.5 (2200 = 2_524_593.5 is OUT — remove it), `_2451545_*`, and the J2000-cluster constants.

- [ ] **Step 4: Update summary validation expectations**

Adjust `reference_snapshot_summary` and any `reference_summary` row-count/epoch assertions to the pruned set.

- [ ] **Step 5: Run the suite (gated on)**

Run: `PLEIADES_DE_KERNEL="$PLEIADES_DE_KERNEL" cargo test`
Expected: green, including `validate_packaged_artifact_phase1_source_inputs`-backed paths and the Eros byte-identity from Task 5 (unchanged).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/data/reference_snapshot.csv crates/pleiades-jpl/src/backend.rs crates/pleiades-jpl/src/reference_summary/ crates/pleiades-data/src/regenerate.rs
git commit -m "chore(jpl): prune reference snapshot to 1900-2100 major-body window"
```

---

### Task 7: Documentation + prose sweep

**Files:**
- Modify: `PLAN.md`, `SPEC.md`, `README.md`
- Modify: `crates/pleiades-data/src/lib.rs:154-162` (provenance prose)
- Modify: `crates/pleiades-validate/src/render/cli.rs`, `crates/pleiades-validate/src/render/summary/release.rs` (help/report templates)

- [ ] **Step 1: Find every prose "1600/2600" reference**

Run: `grep -rn "1600\|2600\|1600-2600\|1600–2600" PLAN.md SPEC.md README.md crates --include=*.rs | grep -iv "fn \|assert\|const "`

- [ ] **Step 2: Rewrite the prose**

Update each to describe the 1900–2100 default and the `generate-artifact` opt-in wider-generation capability. In `lib.rs:158`, change the provenance string from "reference epochs (1800, 2000, 2500 CE)" to reflect the dense de440 fit over 1900–2100 (do not invent new anchor epochs — describe the dense fit).

NOTE: `packaged_artifact_source_text()` (lib.rs) feeds the artifact header. If you change it, the artifact bytes change — so this string MUST have been frozen before Task 5's regeneration. If lib.rs:158 text is part of the encoded artifact, fold this specific edit into Task 5 Step 5 instead and re-regenerate. Verify with: `grep -n "packaged_artifact_source_text" crates/pleiades-data/src/regenerate.rs` (it is passed to `ArtifactHeader::new`). If so, edit the string, then re-run Task 5 Steps 5–6.

- [ ] **Step 3: Verify build + byte-identity still hold**

Run: `PLEIADES_DE_KERNEL="$PLEIADES_DE_KERNEL" cargo test -p pleiades-data --test artifact_regen`
Expected: PASS (byte-identity intact — confirms no header-string drift).

- [ ] **Step 4: Commit**

```bash
git add PLAN.md SPEC.md README.md crates/
git commit -m "docs: describe 1900-2100 default window and generate-artifact opt-in"
```

---

### Task 8: Record the size/perf before/after baseline

**Files:**
- Modify: `docs/superpowers/specs/2026-06-19-default-coverage-window-1900-2100-design.md` (baseline table)
- Modify: `PLAN.md` (mirror the numbers)

- [ ] **Step 1: Capture the after numbers**

Run: `cargo test -p pleiades-data --test artifact_regen sp1_draft_size_perf_baseline -- --nocapture`
Record `artifact_size`, `decode`, `lookup`.

- [ ] **Step 2: Fill the baseline table**

In the spec's "Size / perf baseline" table, replace the `_(measured)_` cells with the Task 1 "before" numbers and these "after" numbers. Mirror the same table into `PLAN.md` near the SP1 status section.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-06-19-default-coverage-window-1900-2100-design.md PLAN.md
git commit -m "docs: record 1900-2100 size/perf before-after baseline"
```

---

## Self-Review

**Spec coverage:** Goal 1 (smaller default) → Tasks 5, 8. Goal 2 (opt-in wider) → Tasks 1–3. Reproduction-paths decision (retire legacy) → Task 4. Fit-anchor prose → Task 7. Corpus/golden regeneration → Tasks 5, 6. All spec sections map to a task.

**Placeholder scan:** Output-dependent values (artifact byte count, new golden buckets, before/after perf numbers) are produced by explicit run-and-paste steps with real commands — not placeholders. The `<committed-artifact-path>` and helper-name confirmations are explicit `grep` steps, not vague gaps.

**Type consistency:** `CoverageWindow` (fields `start_jd`/`end_jd`, methods `new`/`from_years`/`as_tuple`/`default`) is used identically in Tasks 1, 2, 3. `regenerate_packaged_artifact_from_kernel_over(&str, CoverageWindow)` defined in Task 2 is consumed unchanged in Task 3. `build_packaged_artifact_from_reference_over(&dyn EphemerisBackend, (f64,f64))` matches the existing signature at regenerate.rs:2407.

**Key risk flagged inline:** if `packaged_artifact_source_text()` (lib.rs prose) is embedded in the artifact header, editing it drifts the bytes — Task 7 Step 2 routes that edit back into Task 5's regeneration to preserve byte-identity.
