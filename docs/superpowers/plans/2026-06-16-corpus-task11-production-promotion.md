# Corpus Task 11 — Production Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 3-row scaffold reference corpus with a real, broad, de440-generated corpus that the kernel-free `validate-corpus` gate passes and that a kernel-equipped checkout can fully reproduce.

**Architecture:** The corpus spec, generator, gate, and verify-from-kernel test already exist. This plan (1) pins the real `de440.bsp` SHA-256, (2) fixes the interior backbone to per-body cadence + anchor epochs, (3) adds a kernel-free command that derives the independent `fixture_golden` from the checked-in Horizons reference snapshot, (4) regenerates and commits the real slices, (5) extends verify-from-kernel to every slice, and (6) flips the live gate on.

**Tech Stack:** Rust (workspace crates `pleiades-jpl`, `pleiades-cli`, `pleiades-validate`); `cargo test`; `sha256sum`; `curl`.

---

## Background facts (verified against the codebase)

- Single source of truth: `crates/pleiades-jpl/src/spk/corpus_spec.rs`
  (`KERNEL_SHA256 = "<pinned-after-download>"`, per-body `max_gap_days`,
  `interior_backbone_epochs`, `boundary_epochs`, `fast_cluster_epochs`,
  `holdout_epochs`, `release_bodies`, `constrained_bodies`).
- Generator: `crates/pleiades-jpl/src/spk/generate.rs`
  (`generate_corpus_csv`, `generate_slice`, `generate_slice_with_bodies`,
  `build_manifest`). Interior currently samples **every body at the Moon grid**.
- CLI: `crates/pleiades-cli/src/commands/spk_corpus.rs` (`render_spk_corpus`),
  dispatched at `crates/pleiades-cli/src/cli.rs:831`.
- Gate: `crates/pleiades-validate/src/corpus/production.rs`
  (`run_corpus_gate`, embedded slices via `include_str!`,
  `embedded_corpus_gate_passes` is `#[ignore]`d).
- Verify-from-kernel: `crates/pleiades-jpl/tests/corpus_regen.rs` (boundary only).
- Committed corpus dir: `crates/pleiades-jpl/data/corpus/`
  (`boundary.csv`, `interior.csv`, `fast_clusters.csv`, `holdout.csv`,
  `fixture_golden.csv`, `manifest.txt`) — all scaffolds.
- `pleiades_jpl::reference_snapshot()` (root re-export) holds trusted
  **DE441/Horizons** fixtures whose x/y/z are **geocentric ecliptic Cartesian km**
  (confirmed: `SnapshotEntry::ecliptic()` reads `atan2(y, x)` with no rotation),
  including exact-J2000 (`2451545.0`) entries. This is the de440-independent
  source for `fixture_golden`.
- CSV schema (unchanged): `epoch_jd,body,x_km,y_km,z_km`, geocentric ecliptic,
  TDB. Cross-checks are numeric (float formatting is free).

---

## Task 1: Add anchor epochs and per-body interior epochs to the corpus spec

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs`

- [ ] **Step 1: Write the failing tests**

Add to the existing `mod backbone_tests` (or a new `mod anchor_tests`) in
`corpus_spec.rs`:

```rust
#[cfg(test)]
mod anchor_tests {
    use super::*;

    #[test]
    fn anchor_epochs_include_j2000() {
        assert!(anchor_epochs().contains(&2_451_545.0));
    }

    #[test]
    fn interior_epochs_for_include_anchors_sorted_unique() {
        let epochs = interior_epochs_for(&CelestialBody::Neptune);
        // anchor present
        assert!(epochs.contains(&2_451_545.0));
        // strictly increasing (sorted + deduped)
        for pair in epochs.windows(2) {
            assert!(pair[1] > pair[0], "epochs must strictly increase");
        }
    }

    #[test]
    fn interior_epochs_for_neptune_far_fewer_than_moon() {
        let moon = interior_epochs_for(&CelestialBody::Moon).len();
        let neptune = interior_epochs_for(&CelestialBody::Neptune).len();
        assert!(neptune < moon / 10, "slow body must be far sparser");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-jpl corpus_spec::anchor_tests`
Expected: FAIL — `anchor_epochs` / `interior_epochs_for` not found.

- [ ] **Step 3: Implement the helpers**

Add to `corpus_spec.rs` (after `interior_backbone_epochs`):

```rust
/// Anchor epochs always included in the interior backbone for every body, so
/// backend-generated slices overlap the independent `fixture_golden` slice at
/// known (body, epoch) pairs. J2000 has trusted Horizons evidence in
/// `reference_snapshot()`.
pub fn anchor_epochs() -> Vec<f64> {
    vec![2_451_545.0]
}

/// Interior epochs for one body: its per-body cadence backbone unioned with the
/// shared anchor epochs, sorted and deduplicated. Deterministic and stable so
/// checksums and verify-from-kernel reproduce.
pub fn interior_epochs_for(body: &CelestialBody) -> Vec<f64> {
    let mut epochs = interior_backbone_epochs(body);
    epochs.extend(anchor_epochs());
    epochs.sort_by(|a, b| a.partial_cmp(b).expect("epochs are finite"));
    epochs.dedup_by(|a, b| (*a - *b).abs() < 1e-6);
    epochs
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-jpl corpus_spec`
Expected: PASS (all `corpus_spec` tests, including the new module).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): add anchor epochs + per-body interior epochs to corpus spec

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Refactor interior generation to per-body cadence

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/generate.rs`

- [ ] **Step 1: Write the failing test**

Add to `mod slice_tests` in `generate.rs`:

```rust
#[test]
fn interior_uses_per_body_cadence_and_includes_anchor() {
    // Synthetic backend resolves Sun via the const segment chain.
    let slice = generate_slice_with_bodies(
        &backend(),
        SliceRole::InteriorBackbone,
        vec![CelestialBody::Sun],
    )
    .unwrap();
    assert!(slice.csv.contains("#Slice-Role: interior"));
    // Anchor epoch J2000 must appear.
    assert!(
        slice.csv.lines().any(|l| l.starts_with("2451545") && l.contains("Sun")),
        "interior must include the J2000 anchor"
    );
    // Body-outer ordering: all Sun rows are contiguous (only Sun here, so just
    // assert rows exist and are sorted by epoch).
    let epochs: Vec<f64> = slice
        .csv
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(|l| l.split(',').next().unwrap().parse().unwrap())
        .collect();
    assert!(epochs.windows(2).all(|w| w[1] >= w[0]), "epochs ascending per body");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p pleiades-jpl spk::generate::slice_tests::interior_uses_per_body_cadence_and_includes_anchor`
Expected: FAIL — current interior uses the Moon grid for Sun and may not include J2000.

- [ ] **Step 3: Implement per-body interior generation**

First, factor the shared header + row math out of `generate_corpus_csv` so both
emitters reuse it. Add near the top of `generate.rs` (after `AU_IN_KM` usage):

```rust
const AU_IN_KM: f64 = 149_597_870.7;

/// Standard provenance header for every corpus slice CSV.
fn corpus_header(source_label: &str, kernel_sha256: &str) -> String {
    let mut out = String::new();
    out.push_str("#Pleiades SPK Reference Corpus\n");
    out.push_str(&format!("#Source: {source_label}\n"));
    out.push_str(&format!("#Kernel-SHA256: {kernel_sha256}\n"));
    out.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    out.push_str(
        "#Redistribution: derived from public-domain JPL DE kernel; corpus is redistributable\n",
    );
    out.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");
    out
}

/// Samples one body at one epoch and appends its `epoch_jd,body,x,y,z` row.
fn push_corpus_row(
    out: &mut String,
    backend: &SpkBackend,
    body: &CelestialBody,
    jd: f64,
) -> Result<(), String> {
    let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
    let res = backend
        .position(&EphemerisRequest::new(body.clone(), inst))
        .map_err(|e| format!("body {body:?} at jd {jd}: {}", e.message))?;
    let ec = res
        .ecliptic
        .ok_or_else(|| format!("no ecliptic for {body:?}"))?;
    let r_km = ec.distance_au.unwrap_or(0.0) * AU_IN_KM;
    let lon = ec.longitude.degrees().to_radians();
    let lat = ec.latitude.degrees().to_radians();
    let x = r_km * lat.cos() * lon.cos();
    let y = r_km * lat.cos() * lon.sin();
    let z = r_km * lat.sin();
    out.push_str(&format!("{jd},{body},{x:.6},{y:.6},{z:.6}\n"));
    Ok(())
}
```

Rewrite `generate_corpus_csv` to use them (epoch-outer, body-inner — unchanged
order for boundary/fast/holdout):

```rust
pub fn generate_corpus_csv(backend: &SpkBackend, req: &CorpusRequest) -> Result<String, String> {
    let mut out = corpus_header(&req.source_label, &req.kernel_sha256);
    for &jd in &req.epoch_jds {
        for body in &req.bodies {
            push_corpus_row(&mut out, backend, body, jd)?;
        }
    }
    Ok(out)
}
```

Add the per-body emitter (body-outer, epoch-inner):

```rust
/// Emits a corpus CSV where each body is sampled at its own epoch list.
/// Bodies are emitted in the given order; epochs in the given (already sorted)
/// order. Used by the interior backbone so slow bodies are not over-sampled.
pub fn generate_corpus_csv_per_body(
    backend: &SpkBackend,
    per_body: &[(CelestialBody, Vec<f64>)],
    source_label: &str,
    kernel_sha256: &str,
) -> Result<String, String> {
    let mut out = corpus_header(source_label, kernel_sha256);
    for (body, epochs) in per_body {
        for &jd in epochs {
            push_corpus_row(&mut out, backend, body, jd)?;
        }
    }
    Ok(out)
}
```

Now route the interior role through it inside `generate_slice_with_bodies`.
Replace the `match role` block so interior is handled separately:

```rust
pub(crate) fn generate_slice_with_bodies(
    backend: &SpkBackend,
    role: SliceRole,
    bodies: Vec<CelestialBody>,
) -> Result<GeneratedSlice, String> {
    if role == SliceRole::InteriorBackbone {
        let per_body: Vec<(CelestialBody, Vec<f64>)> = bodies
            .iter()
            .map(|b| (b.clone(), corpus_spec::interior_epochs_for(b)))
            .collect();
        let mut csv = generate_corpus_csv_per_body(
            backend,
            &per_body,
            corpus_spec::KERNEL_LABEL,
            corpus_spec::KERNEL_SHA256,
        )?;
        csv = csv.replace(
            "#Columns:",
            &format!("#Slice-Role: {}\n#Columns:", role.token()),
        );
        return Ok(GeneratedSlice {
            role,
            file: "interior.csv".to_string(),
            csv,
        });
    }

    let (file, epochs) = match role {
        SliceRole::Boundary => ("boundary.csv", corpus_spec::boundary_epochs()),
        SliceRole::FastCluster => ("fast_clusters.csv", corpus_spec::fast_cluster_epochs()),
        SliceRole::Holdout => ("holdout.csv", corpus_spec::holdout_epochs(50)),
        SliceRole::InteriorBackbone => unreachable!("interior handled above"),
        SliceRole::FixtureGolden => {
            return Err("fixture_golden is sourced from existing fixtures, not generated".into())
        }
    };
    let req = CorpusRequest {
        bodies,
        epoch_jds: epochs,
        source_label: corpus_spec::KERNEL_LABEL.to_string(),
        kernel_sha256: corpus_spec::KERNEL_SHA256.to_string(),
    };
    let mut csv = generate_corpus_csv(backend, &req)?;
    csv = csv.replace(
        "#Columns:",
        &format!("#Slice-Role: {}\n#Columns:", role.token()),
    );
    Ok(GeneratedSlice {
        role,
        file: file.to_string(),
        csv,
    })
}
```

Delete the now-duplicated `const AU_IN_KM` inside the old `generate_corpus_csv`
body (it now lives at module scope). Ensure `SliceRole` derives `PartialEq`
(it already does in `corpus_spec.rs`).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-jpl spk::generate`
Expected: PASS (new interior test + existing `slice_tests` and `tests`,
including the round-trip `generates_csv_with_provenance_and_rows`).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/generate.rs
git commit -m "feat(jpl): sample interior backbone per body at spec cadence + anchors

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Add the kernel-free `generate-fixture-golden` command

**Files:**
- Create: `crates/pleiades-cli/src/commands/fixture_golden.rs`
- Modify: `crates/pleiades-cli/src/commands/mod.rs`
- Modify: `crates/pleiades-cli/src/cli.rs` (dispatch near line 831)
- Modify: `crates/pleiades-cli/src/help.rs` (help banner)

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-cli/src/commands/fixture_golden.rs` with the test first:

```rust
//! `generate-fixture-golden` command: derive the de440-independent
//! `fixture_golden.csv` from the checked-in Horizons reference snapshot.

use pleiades_jpl::spk::corpus_spec;

/// Writes `fixture_golden.csv` into `out_dir`, containing the reference-snapshot
/// (Horizons/DE441) entries at the corpus anchor epochs, in the geocentric
/// ecliptic corpus schema. This is the independent cross-check source; it is NOT
/// generated from the de440 kernel.
pub fn render_fixture_golden(args: &[&str]) -> Result<String, String> {
    let out_dir = args
        .first()
        .ok_or("generate-fixture-golden requires an output directory")?;

    let anchors = corpus_spec::anchor_epochs();
    let mut csv = String::new();
    csv.push_str("#Pleiades SPK Reference Corpus\n");
    csv.push_str("#Source: NASA/JPL Horizons reference snapshot (independent of the de440 corpus)\n");
    csv.push_str(&format!("#Kernel-SHA256: {}\n", corpus_spec::KERNEL_SHA256));
    csv.push_str("#Coverage: geocentric ecliptic (mean geometric), TDB epochs\n");
    csv.push_str("#Redistribution: derived from public-domain JPL Horizons fixture; corpus is redistributable\n");
    csv.push_str("#Slice-Role: fixture_golden\n");
    csv.push_str("#Columns:epoch_jd,body,x_km,y_km,z_km\n");

    let mut rows = 0usize;
    for entry in pleiades_jpl::reference_snapshot() {
        let jd = entry.epoch.julian_day.days();
        if anchors.iter().any(|a| (jd - a).abs() < 1e-6) {
            csv.push_str(&format!(
                "{},{},{:.6},{:.6},{:.6}\n",
                jd, entry.body, entry.x_km, entry.y_km, entry.z_km
            ));
            rows += 1;
        }
    }
    if rows == 0 {
        return Err("no reference-snapshot entries at the corpus anchor epochs".to_string());
    }

    std::fs::write(format!("{out_dir}/fixture_golden.csv"), &csv)
        .map_err(|e| format!("write fixture_golden.csv: {e}"))?;
    Ok(format!("wrote fixture_golden.csv ({rows} rows) to {out_dir}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_out_dir_errors() {
        assert!(render_fixture_golden(&[]).is_err());
    }

    #[test]
    fn writes_anchor_rows_to_dir() {
        let dir = std::env::temp_dir().join(format!(
            "pleiades_fg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let dir_str = dir.to_string_lossy();
        let msg = render_fixture_golden(&[&dir_str]).unwrap();
        assert!(msg.contains("fixture_golden.csv"));
        let written = std::fs::read_to_string(dir.join("fixture_golden.csv")).unwrap();
        assert!(written.contains("#Slice-Role: fixture_golden"));
        assert!(written.contains("#Columns:epoch_jd,body,x_km,y_km,z_km"));
        // At least one J2000 data row.
        assert!(written.lines().any(|l| l.starts_with("2451545")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 2: Wire the module and run the test to verify it fails first**

Add to `crates/pleiades-cli/src/commands/mod.rs`:

```rust
pub(crate) mod fixture_golden;
```

Run: `cargo test -p pleiades-cli commands::fixture_golden`
Expected: PASS for both tests (the command is fully implemented in Step 1).
If `reference_snapshot` has no J2000 entries the second test fails — that would
be a real coverage gap to fix in `pleiades-jpl` before proceeding.

> Note: this command is self-contained, so its implementation and tests land
> together. The "failing first" discipline is preserved by Tasks 1–2 and 5–7.

- [ ] **Step 3: Dispatch the command**

In `crates/pleiades-cli/src/cli.rs`, add the `use` near the other command import
(`use crate::commands::spk_corpus::render_spk_corpus;`):

```rust
use crate::commands::fixture_golden::render_fixture_golden;
```

And add a match arm next to the `generate-spk-corpus` arm (around line 831):

```rust
        Some("generate-fixture-golden") => render_fixture_golden(&args[1..]),
```

- [ ] **Step 4: Add help text and run the build**

In `crates/pleiades-cli/src/help.rs`, after the two `generate-spk-corpus` lines:

```rust
  generate-fixture-golden <out-dir>  Derive the de440-independent fixture_golden.csv from the checked-in Horizons reference snapshot
```

Run: `cargo test -p pleiades-cli`
Expected: PASS (including any CLI command-inventory tests; if a test enumerates
known commands, add `generate-fixture-golden` to its expected set).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/commands/fixture_golden.rs \
        crates/pleiades-cli/src/commands/mod.rs \
        crates/pleiades-cli/src/cli.rs \
        crates/pleiades-cli/src/help.rs
git commit -m "feat(cli): add generate-fixture-golden (independent Horizons source)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Download de440.bsp and pin its SHA-256

> This task requires network access and ~114 MB of disk. The kernel is NOT
> committed. If running in an environment without the kernel, perform this task
> on a machine that has it and copy the resulting hash into the repo.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (`KERNEL_SHA256`)
- Modify: `docs/spk-kernel-sourcing.md` (SHA-256 lines)

- [ ] **Step 1: Download the kernel to a gitignored path**

```bash
mkdir -p .cache/kernels
curl -L -o .cache/kernels/de440.bsp \
  https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp
ls -l .cache/kernels/de440.bsp
```
Expected: a ~119,799,808-byte file.

- [ ] **Step 2: Confirm `.cache/` is gitignored**

Add `.cache/` to `.gitignore` if absent:

```bash
grep -qxF '.cache/' .gitignore || printf '\n# Local ephemeris kernels (not committed)\n.cache/\n' >> .gitignore
git add .gitignore && git commit -m "chore: gitignore local kernel cache

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 3: Compute the SHA-256**

```bash
sha256sum .cache/kernels/de440.bsp
```
Expected: a 64-hex-char hash. Record it as `<DE440_SHA>` below.

- [ ] **Step 4: Sanity-check the kernel resolves**

```bash
PLEIADES_DE_KERNEL=.cache/kernels/de440.bsp \
  cargo test -p pleiades-jpl --test spk_full_kernel -- --nocapture
```
Expected: PASS (`de440_reports_coverage_and_resolves_sun`).

- [ ] **Step 5: Pin the hash and commit**

In `crates/pleiades-jpl/src/spk/corpus_spec.rs`, replace:

```rust
pub const KERNEL_SHA256: &str = "<pinned-after-download>";
```
with (using the value from Step 3):

```rust
pub const KERNEL_SHA256: &str = "<DE440_SHA>";
```

In `docs/spk-kernel-sourcing.md`, replace both
`<fill in after downloading: ...>` / `<pinned-after-download>` SHA placeholders
with `<DE440_SHA>`.

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs docs/spk-kernel-sourcing.md
git commit -m "feat(jpl): pin de440.bsp SHA-256 in corpus spec + sourcing doc

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Regenerate and commit the real corpus

> Requires the kernel from Task 4 and the pinned SHA (so generated headers carry
> the real hash). Order matters: fixture_golden first (the `--emit-slices` step
> reads it to build the manifest).

**Files:**
- Modify: `crates/pleiades-jpl/data/corpus/fixture_golden.csv`
- Modify: `crates/pleiades-jpl/data/corpus/boundary.csv`
- Modify: `crates/pleiades-jpl/data/corpus/interior.csv`
- Modify: `crates/pleiades-jpl/data/corpus/fast_clusters.csv`
- Modify: `crates/pleiades-jpl/data/corpus/holdout.csv`
- Modify: `crates/pleiades-jpl/data/corpus/manifest.txt`

- [ ] **Step 1: Derive the independent fixture_golden (kernel-free)**

```bash
cargo run -p pleiades-cli -- generate-fixture-golden crates/pleiades-jpl/data/corpus
```
Expected: `wrote fixture_golden.csv (N rows) to crates/pleiades-jpl/data/corpus`.

- [ ] **Step 2: Generate the four backend slices + manifest from the kernel**

```bash
cargo run -p pleiades-cli -- generate-spk-corpus \
  .cache/kernels/de440.bsp --emit-slices crates/pleiades-jpl/data/corpus
```
Expected: `wrote 5 slices + manifest (incl. fixture_golden) to ...`.

- [ ] **Step 3: Inspect the result**

```bash
for f in crates/pleiades-jpl/data/corpus/*.csv; do
  printf "%-40s %s data rows\n" "$f" "$(grep -vc '^#' "$f")"
done
cat crates/pleiades-jpl/data/corpus/manifest.txt
```
Expected: interior in the tens of thousands of rows (Moon-dominated, slow bodies
sparse), all checksums non-zero, no `<pinned-after-download>` anywhere, total
corpus on the order of ~1.5 MB.

- [ ] **Step 4: Run the gate manually (kernel-free path)**

```bash
cargo run -p pleiades-validate -- validate-corpus
```
Expected: `corpus gate ok: 5 slices, <N> data rows, kernel de440.bsp`.
If the fixture-golden cross-check errors, the SPK reader / frame disagrees with
Horizons beyond tolerance — stop and investigate (this is the gate doing its
job), do not loosen the tolerance to force a pass.

- [ ] **Step 5: Commit the real corpus**

```bash
git add crates/pleiades-jpl/data/corpus/
git commit -m "feat(corpus): regenerate real de440 corpus at full breadth (Task 11)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Extend verify-from-kernel to all backend slices

**Files:**
- Modify: `crates/pleiades-jpl/tests/corpus_regen.rs`

- [ ] **Step 1: Replace the boundary-only body with an all-slice loop**

Rewrite `crates/pleiades-jpl/tests/corpus_regen.rs`:

```rust
//! Gated: regenerates each backend slice from the real kernel and compares to
//! the checked-in CSV within a tight reproducibility tolerance. Skipped unless
//! PLEIADES_DE_KERNEL points at de440.bsp.

#[test]
fn regenerated_corpus_matches_checked_in() {
    let Ok(kernel) = std::env::var("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL to run");
        return;
    };
    use pleiades_jpl::spk::corpus_spec::SliceRole;
    use pleiades_jpl::{generate_slice, SpkBackend};

    let backend = SpkBackend::builder().add_kernel(&kernel).unwrap().build();

    // (role, checked-in CSV) for every backend-generated slice.
    let cases: [(SliceRole, &str); 4] = [
        (SliceRole::Boundary, include_str!("../data/corpus/boundary.csv")),
        (SliceRole::InteriorBackbone, include_str!("../data/corpus/interior.csv")),
        (SliceRole::FastCluster, include_str!("../data/corpus/fast_clusters.csv")),
        (SliceRole::Holdout, include_str!("../data/corpus/holdout.csv")),
    ];

    let parse = |csv: &str| -> Vec<(String, [f64; 3])> {
        csv.lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                (
                    format!("{},{}", f[0], f[1]),
                    [f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap()],
                )
            })
            .collect()
    };

    for (role, checked_in) in cases {
        let regenerated = generate_slice(&backend, role).unwrap();
        let a = parse(&regenerated.csv);
        let b = parse(checked_in);
        assert_eq!(a.len(), b.len(), "row count drift vs checked-in {role:?}");
        for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
            assert_eq!(ka, kb, "epoch/body ordering drift in {role:?}");
            for i in 0..3 {
                assert!((va[i] - vb[i]).abs() < 1.0, "value drift > 1 km at {ka} in {role:?}");
            }
        }
    }
}
```

- [ ] **Step 2: Run it with the kernel to verify reproduction**

Run: `PLEIADES_DE_KERNEL=.cache/kernels/de440.bsp cargo test -p pleiades-jpl --test corpus_regen -- --nocapture`
Expected: PASS — every backend slice reproduces the committed CSV within 1 km.

- [ ] **Step 3: Run it without the kernel to verify the skip path**

Run: `cargo test -p pleiades-jpl --test corpus_regen`
Expected: PASS via early-return skip.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-jpl/tests/corpus_regen.rs
git commit -m "test(jpl): verify-from-kernel reproduces all backend slices

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Flip the live gate on

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs`

- [ ] **Step 1: Remove the `#[ignore]` from the embedded gate test**

In `crates/pleiades-validate/src/corpus/production.rs`, change:

```rust
    #[test]
    #[ignore = "enabled after Task 11 regenerates real corpus data + checksums"]
    fn embedded_corpus_gate_passes() {
        run_corpus_gate().unwrap();
    }
```
to:

```rust
    #[test]
    fn embedded_corpus_gate_passes() {
        run_corpus_gate().unwrap();
    }
```

- [ ] **Step 2: Run the gate test (no kernel needed)**

Run: `cargo test -p pleiades-validate corpus::production`
Expected: PASS — `embedded_corpus_gate_passes` now runs against the real
committed corpus and succeeds (completeness, schema/provenance, checksum,
cross-check all green).

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs
git commit -m "test(validate): enable live corpus gate over real committed corpus

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Sync docs, plan, and status

**Files:**
- Modify: `PLAN.md`
- Modify: `plan/stages/01-production-reference-corpus.md`
- Modify: `plan/status/01-current-execution-frontier.md`
- Modify: `plan/status/02-next-slice-candidates.md`
- Modify: `README.md` (only if it describes the corpus as draft/scaffold)

- [ ] **Step 1: Update the Phase 1 stage exit criteria status**

In `plan/stages/01-production-reference-corpus.md`, under "Remaining
implementation work" and "Exit criteria": mark the generation-pipeline,
verify, and reproduce items as met (a clean checkout verifies kernel-free via
`validate-corpus`; with `PLEIADES_DE_KERNEL` it reproduces all slices). Keep the
broad **public-data reader** for arbitrary external products listed as the
remaining open item.

- [ ] **Step 2: Update the execution-frontier and next-slice status**

In `plan/status/01-current-execution-frontier.md` and
`plan/status/02-next-slice-candidates.md`: note the corpus is now real, broad,
checksum-pinned, and gated (no scaffold); the per-body interior cadence and the
independent fixture-golden cross-check are in place. Re-point "recommended next
slice" at the remaining Phase 1 public-data-reader work (and Phase 2 readiness).

- [ ] **Step 3: Update PLAN.md current limits**

In `PLAN.md`, revise the `pleiades-jpl` "Important current limits" bullet: the
checked-in corpus is no longer a sparse scaffold; the reproducible
generation pipeline produces a broad de440-sourced corpus with a fail-closed
gate. Leave the broad-public-data-reader gap and the draft-artifact (Phase 2)
limit intact. Update the `Status:` line date to 2026-06-16 with a one-line note.

- [ ] **Step 4: Verify no scaffold language remains**

Run: `grep -rn "scaffold\|Task 11\|pinned-after-download" PLAN.md plan/ docs/spk-kernel-sourcing.md crates/pleiades-jpl/data/corpus/ README.md`
Expected: no matches (the regenerated CSVs overwrote their scaffold headers; docs
and plan no longer reference a scaffold or an unpinned hash).

- [ ] **Step 5: Commit**

```bash
git add PLAN.md plan/ README.md
git commit -m "docs: sync plan/status/README to real gated reference corpus

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Final workspace verification

**Files:** none (verification only)

- [ ] **Step 1: Full test suite (kernel-free, as CI runs)**

Run: `cargo test --workspace`
Expected: PASS, including `embedded_corpus_gate_passes` and the skipped
(early-return) `corpus_regen` / `spk_full_kernel`.

- [ ] **Step 2: Full reproduction with the kernel**

Run: `PLEIADES_DE_KERNEL=.cache/kernels/de440.bsp cargo test --workspace`
Expected: PASS, with `corpus_regen` and `spk_full_kernel` now exercising the
real kernel.

- [ ] **Step 3: Lint and format gates**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 4: Confirm the gate one-liner**

Run: `cargo run -p pleiades-validate -- validate-corpus`
Expected: `corpus gate ok: 5 slices, <N> data rows, kernel de440.bsp`.

- [ ] **Step 5: Final commit (only if fmt/clippy changed anything)**

```bash
git add -A
git commit -m "chore: fmt/clippy clean-up after corpus Task 11

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Self-review notes

- **Spec coverage:** SHA pinning → Task 4; per-body interior → Tasks 1–2;
  real fixture-golden / Approach A anchors → Tasks 1, 3, 5; regenerate+commit →
  Task 5; all-slice verify-from-kernel → Task 6; flip gate → Task 7; docs/plan
  sync → Task 8; exit-criteria re-check → Tasks 5, 7, 8, 9. Non-goals
  (public-data reader, asteroid kernel, claim widening, schema change) are
  explicitly preserved in Tasks 5 and 8.
- **Type consistency:** `anchor_epochs()`, `interior_epochs_for()`,
  `generate_corpus_csv_per_body()`, `corpus_header()`, `push_corpus_row()`,
  `render_fixture_golden()` are defined where first used and referenced
  consistently. `SliceRole::InteriorBackbone` matches `corpus_spec.rs`.
- **Kernel-dependent tasks (4, 5, and the kernel arms of 6/9)** are clearly
  marked; everything CI runs (Tasks 1–3, 7, gate test, skips) is kernel-free.
