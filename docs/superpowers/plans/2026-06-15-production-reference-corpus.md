# Production Reference Corpus Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the ad-hoc `generate-spk-corpus` output into a defined, stratified, checked-in production reference corpus with a single source-of-truth spec, a manifest, and a fail-closed validation gate that meets Phase 1 exit criteria.

**Architecture:** A Rust source-of-truth module (`spk/corpus_spec.rs`) deterministically defines the epoch grid (boundary / interior-backbone / fast-cluster / hold-out slices), per-body cadence, the release-claimed completeness matrix, and per-body-class tolerances. The existing `SpkBackend` + `generate_corpus_csv` sample it into the existing `epoch_jd,body,x_km,y_km,z_km` CSV schema, one file per slice, plus a `manifest.txt` carrying provenance and FNV content checksums. A new `pleiades-validate` corpus gate loads the checked-in slices kernel-free and fails closed on missing bodies/epoch-classes/channels/frames, schema drift, checksum/source-revision drift, and fixture-golden tolerance breaches. A gated verify path regenerates from de440 and value-compares.

**Tech Stack:** Pure Rust (edition 2021, `#![forbid(unsafe_code)]`), no new dependencies. Reuses the repo's existing `checksum64` (FNV-1a), `parse_snapshot_manifest` manifest-text format, `parse_snapshot_entries` CSV loader, and `include_str!` data embedding.

**Deviation from the approved spec (intentional, codebase-native):** The spec names `corpus-spec.toml` / `manifest.toml` and SHA-256 content checksums. `pleiades-jpl` is deliberately zero-dependency and already ships `checksum64` (FNV-1a) and a `#`-comment manifest-text format. To avoid adding `toml`/`serde`/`sha2`, this plan realizes the spec's *intent* with a compile-checked Rust spec module + the existing manifest-text format + FNV content checksums. The **kernel** SHA-256 stays a documented pinned constant (computed externally via `shasum -a 256`, recorded in `docs/spk-kernel-sourcing.md`); kernel-identity drift is caught by the gated regenerate-and-value-compare path, which is stronger than a stored hash. Update the spec doc's §2/§4 wording to match once this lands.

**Bootstrapping note:** Two tasks are marked **[MAINTAINER — requires de440 + asteroid kernel]**. All other tasks are fully testable kernel-free using synthetic DAF fixtures (the established `spk/test_support.rs` pattern) and synthetic in-test slice fixtures. The checked-in `data/corpus/*.csv` files are scaffolded with a few truthful rows in Task 1 so the crate compiles, then regenerated at full breadth from de440 in Task 11. The release-gate wiring (Task 12) is sequenced after the real data exists.

---

## File Structure

**New files:**
- `crates/pleiades-jpl/src/spk/corpus_spec.rs` — source-of-truth spec: slice roles, per-body cadence, deterministic epoch grid, release/constrained body sets, completeness matrix, tolerances, kernel label + pinned SHA-256 constant.
- `crates/pleiades-jpl/src/spk/corpus_manifest.rs` — `CorpusManifest` type, FNV content checksum, render-to-text + parse (manifest-text style), round-trip.
- `crates/pleiades-jpl/data/corpus/boundary.csv`, `interior.csv`, `fast_clusters.csv`, `holdout.csv`, `fixture_golden.csv` — the five checked-in slice files.
- `crates/pleiades-jpl/data/corpus/manifest.txt` — provenance + per-file checksums + row counts + roles.
- `crates/pleiades-validate/src/corpus/production.rs` — fail-closed corpus gate over the checked-in slices.

**Modified files:**
- `crates/pleiades-jpl/src/spk/mod.rs` — declare + re-export the new modules.
- `crates/pleiades-jpl/src/spk/generate.rs` — add slice/bundle generation over the spec.
- `crates/pleiades-jpl/src/lib.rs` — re-export new public items.
- `crates/pleiades-jpl/src/reference_summary/reference_snapshot/core/general_a.rs` — promote `checksum64` to `pub` (or re-export) for reuse. (See Task 5 for the exact approach.)
- `crates/pleiades-validate/src/corpus/mod.rs` — declare `production` submodule + re-export gate entry point.
- `crates/pleiades-validate/src/render/cli.rs` — add `validate-corpus` command token.
- `crates/pleiades-cli/src/commands/spk_corpus.rs` — promote `generate-spk-corpus` to spec-driven slice+manifest emission.
- `crates/pleiades-cli/src/help.rs` — sync help text.

---

## Task 1: Scaffold the checked-in corpus data files

**Files:**
- Create: `crates/pleiades-jpl/data/corpus/boundary.csv`
- Create: `crates/pleiades-jpl/data/corpus/interior.csv`
- Create: `crates/pleiades-jpl/data/corpus/fast_clusters.csv`
- Create: `crates/pleiades-jpl/data/corpus/holdout.csv`
- Create: `crates/pleiades-jpl/data/corpus/fixture_golden.csv`
- Create: `crates/pleiades-jpl/data/corpus/manifest.txt`

These exist so later `include_str!` references compile and are truthful before the de440 regeneration in Task 11. Use real values copied from the existing checked-in fixtures (e.g. `crates/pleiades-jpl/src/data/independent_holdout_snapshot.csv`) so no fabricated numbers enter the repo.

- [ ] **Step 1: Create the five slice CSVs with the existing schema header and a few real rows**

Each file must start with the existing provenance header block and column header, then real rows. Example for `boundary.csv` (copy two real rows for the earliest/latest available epochs from an existing fixture; do not invent coordinates):

```
#Pleiades SPK Reference Corpus
#Source: JPL DE SPK kernel: de440.bsp (scaffold — regenerated at full breadth in Task 11)
#Kernel-SHA256: <pinned-after-download>
#Coverage: geocentric ecliptic (mean geometric), TDB epochs
#Redistribution: derived from public-domain JPL DE kernel; corpus is redistributable
#Slice-Role: boundary
#Columns:epoch_jd,body,x_km,y_km,z_km
2451545,Sun,<x>,<y>,<z>
```

Fill `<x>,<y>,<z>` from an existing fixture row for the same body+epoch. Repeat the pattern for `interior.csv` (`#Slice-Role: interior`), `fast_clusters.csv` (`#Slice-Role: fast_cluster`), `holdout.csv` (`#Slice-Role: holdout`), and `fixture_golden.csv` (`#Slice-Role: fixture_golden`). Each file needs at least one real row so it parses.

- [ ] **Step 2: Create `manifest.txt` in the existing manifest-text style**

```
#Pleiades SPK Reference Corpus Manifest
#Kernel: de440.bsp
#Kernel-SHA256: <pinned-after-download>
#Frame: geocentric ecliptic (mean geometric)
#TimeScale: TDB
#Obliquity-Source: shared backend obliquity constant
#Generation-Command: generate-spk-corpus <kernel.bsp> --emit-slices
slice boundary file=boundary.csv role=boundary rows=1 checksum=0
slice interior file=interior.csv role=interior rows=1 checksum=0
slice fast_clusters file=fast_clusters.csv role=fast_cluster rows=1 checksum=0
slice holdout file=holdout.csv role=holdout rows=1 checksum=0
slice fixture_golden file=fixture_golden.csv role=fixture_golden rows=1 checksum=0
```

(`checksum=0` placeholders are corrected by the generator/Task 5 helper; Task 7's gate rejects mismatches.)

- [ ] **Step 3: Verify the scaffold parses through the existing loader**

Run: `cargo test -p pleiades-jpl 2>&1 | tail -5`
Expected: PASS (no code change yet; this confirms the crate still builds with the new files present).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-jpl/data/corpus/
git commit -m "feat(jpl): scaffold checked-in corpus slice files and manifest"
```

---

## Task 2: Corpus spec types and body sets

**Files:**
- Create: `crates/pleiades-jpl/src/spk/corpus_spec.rs`
- Modify: `crates/pleiades-jpl/src/spk/mod.rs`

- [ ] **Step 1: Write the failing test**

In `crates/pleiades-jpl/src/spk/corpus_spec.rs`:

```rust
//! Single source of truth for the production reference corpus: slice roles,
//! per-body cadence, the deterministic epoch grid, release/constrained body
//! sets, the completeness matrix, and cross-check tolerances. Both the
//! generator and the validation gate read this module so coverage cannot drift.

use pleiades_backend::CelestialBody;

/// Target packaged range, as TDB Julian Days (1600-01-01 .. 2600-01-01).
pub const RANGE_START_JD: f64 = 2_305_447.5;
pub const RANGE_END_JD: f64 = 2_670_589.5;

/// Pinned identity of the reference kernel. SHA-256 is computed externally via
/// `shasum -a 256 de440.bsp` and recorded here + in docs/spk-kernel-sourcing.md.
pub const KERNEL_LABEL: &str = "JPL DE SPK kernel: de440.bsp";
pub const KERNEL_SHA256: &str = "<pinned-after-download>";

/// Role of a corpus slice, preserving the reference/holdout/boundary/
/// fixture-exactness separation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceRole {
    Boundary,
    InteriorBackbone,
    FastCluster,
    Holdout,
    FixtureGolden,
}

impl SliceRole {
    /// The token written to the `#Slice-Role:` header and the manifest.
    pub fn token(self) -> &'static str {
        match self {
            SliceRole::Boundary => "boundary",
            SliceRole::InteriorBackbone => "interior",
            SliceRole::FastCluster => "fast_cluster",
            SliceRole::Holdout => "holdout",
            SliceRole::FixtureGolden => "fixture_golden",
        }
    }
}

/// Bodies that must be fully covered by the completeness matrix.
pub fn release_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
    ]
}

/// Bodies carried but tagged constrained/approximate and excluded from
/// release-grade tolerance evidence (Pluto + selected asteroids).
pub fn constrained_bodies() -> Vec<CelestialBody> {
    vec![CelestialBody::Pluto]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_spans_target_window() {
        assert!(RANGE_START_JD < RANGE_END_JD);
        // ~1000 years in days.
        assert!((RANGE_END_JD - RANGE_START_JD - 365_142.0).abs() < 2.0);
    }

    #[test]
    fn release_and_constrained_bodies_are_disjoint() {
        for body in release_bodies() {
            assert!(!constrained_bodies().contains(&body));
        }
    }

    #[test]
    fn slice_role_tokens_are_unique() {
        let roles = [
            SliceRole::Boundary,
            SliceRole::InteriorBackbone,
            SliceRole::FastCluster,
            SliceRole::Holdout,
            SliceRole::FixtureGolden,
        ];
        let mut tokens: Vec<&str> = roles.iter().map(|r| r.token()).collect();
        tokens.sort_unstable();
        tokens.dedup();
        assert_eq!(tokens.len(), roles.len());
    }
}
```

In `crates/pleiades-jpl/src/spk/mod.rs`, add the module declaration alongside the existing ones:

```rust
pub mod corpus_spec;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl corpus_spec:: 2>&1 | tail -15`
Expected: FAIL — module/items not yet wired, or asserts not yet satisfied (e.g. the range constant tuned).

- [ ] **Step 3: Tune constants until tests pass**

Confirm `CelestialBody` is the correct path (`pleiades_backend::CelestialBody`, matching `spk/generate.rs`). Adjust `RANGE_START_JD`/`RANGE_END_JD` to the exact 1600-01-01 / 2600-01-01 TDB Julian Days if the span assertion fails.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl corpus_spec:: 2>&1 | tail -15`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs crates/pleiades-jpl/src/spk/mod.rs
git commit -m "feat(jpl): add corpus spec roles and body sets"
```

---

## Task 3: Per-body cadence and the interior backbone grid

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs`

- [ ] **Step 1: Write the failing test**

Add to `corpus_spec.rs`:

```rust
/// Maximum allowed epoch gap (TDB days) for a body in the interior backbone,
/// scaled to body speed: fast bodies sampled densely, slow bodies sparsely.
pub fn max_gap_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 30.0,
        CelestialBody::Mercury => 60.0,
        CelestialBody::Venus => 120.0,
        CelestialBody::Sun => 180.0,
        CelestialBody::Mars => 365.0,
        CelestialBody::Jupiter => 1_825.0,   // ~5 yr
        CelestialBody::Saturn => 3_650.0,    // ~10 yr
        CelestialBody::Uranus | CelestialBody::Neptune => 7_300.0, // ~20 yr
        CelestialBody::Pluto => 7_300.0,
        _ => 365.0,
    }
}

/// Deterministic, strictly-increasing interior backbone epochs for a body:
/// from RANGE_START_JD to RANGE_END_JD inclusive, stepping by `max_gap_days`.
pub fn interior_backbone_epochs(body: &CelestialBody) -> Vec<f64> {
    let step = max_gap_days(body);
    let mut epochs = Vec::new();
    let mut jd = RANGE_START_JD;
    while jd < RANGE_END_JD {
        epochs.push(jd);
        jd += step;
    }
    epochs.push(RANGE_END_JD);
    epochs
}

#[cfg(test)]
mod backbone_tests {
    use super::*;

    #[test]
    fn backbone_is_within_range_and_increasing() {
        let epochs = interior_backbone_epochs(&CelestialBody::Mars);
        assert!(epochs.len() >= 2);
        assert_eq!(*epochs.first().unwrap(), RANGE_START_JD);
        assert_eq!(*epochs.last().unwrap(), RANGE_END_JD);
        for pair in epochs.windows(2) {
            assert!(pair[1] > pair[0], "epochs must strictly increase");
        }
    }

    #[test]
    fn backbone_respects_max_gap() {
        for body in release_bodies() {
            let gap = max_gap_days(&body);
            let epochs = interior_backbone_epochs(&body);
            for pair in epochs.windows(2) {
                assert!(
                    pair[1] - pair[0] <= gap + 1e-6,
                    "gap exceeds cadence for {body:?}"
                );
            }
        }
    }

    #[test]
    fn faster_bodies_have_more_samples() {
        let moon = interior_backbone_epochs(&CelestialBody::Moon).len();
        let neptune = interior_backbone_epochs(&CelestialBody::Neptune).len();
        assert!(moon > neptune);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl backbone_tests:: 2>&1 | tail -15`
Expected: FAIL — `max_gap_days` / `interior_backbone_epochs` not yet present (these are added in Step 1 above; if you split authoring, the test fails first).

- [ ] **Step 3: Confirm implementation present**

The functions are defined in Step 1. Ensure the final pushed `RANGE_END_JD` does not duplicate a stepped value within 1e-6 (if `RANGE_END_JD - last_step < 1e-6`, drop the extra push). Adjust by checking before the final push:

```rust
    if epochs.last().map_or(true, |&last| (RANGE_END_JD - last).abs() > 1e-6) {
        epochs.push(RANGE_END_JD);
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl backbone_tests:: 2>&1 | tail -15`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): add body-speed-scaled interior backbone epoch grid"
```

---

## Task 4: Boundary, fast-cluster, and deterministic hold-out epochs

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs`

- [ ] **Step 1: Write the failing test**

Add to `corpus_spec.rs`:

```rust
/// Guard epochs just inside and just outside each end of the target range.
pub fn boundary_epochs() -> Vec<f64> {
    vec![
        RANGE_START_JD - 365.0,
        RANGE_START_JD,
        RANGE_START_JD + 365.0,
        RANGE_END_JD - 365.0,
        RANGE_END_JD,
        RANGE_END_JD + 365.0,
    ]
}

/// Fine-cadence windows for fast bodies. Each anchor expands to daily samples
/// across `window_days`. Anchors are spread across the range.
pub fn fast_cluster_epochs() -> Vec<f64> {
    let anchors = [
        RANGE_START_JD + 5_000.0,
        (RANGE_START_JD + RANGE_END_JD) / 2.0,
        RANGE_END_JD - 5_000.0,
    ];
    let window_days = 30;
    let mut epochs = Vec::new();
    for anchor in anchors {
        for day in 0..window_days {
            epochs.push(anchor + day as f64);
        }
    }
    epochs
}

/// Deterministic pseudo-random hold-out epochs (seeded LCG), disjoint from the
/// interior backbone of every body. Used for unbiased artifact error.
pub fn holdout_epochs(count: usize) -> Vec<f64> {
    // Numerical Recipes LCG constants; deterministic across runs.
    let mut state: u64 = 0x5DEE_CE66_D000_1234;
    let mut epochs = Vec::with_capacity(count);
    let span = RANGE_END_JD - RANGE_START_JD;
    while epochs.len() < count {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let frac = (state >> 11) as f64 / (1u64 << 53) as f64;
        let jd = RANGE_START_JD + frac * span;
        // Keep hold-out off the coarsest backbone grid lines.
        let on_grid = release_bodies()
            .iter()
            .any(|b| ((jd - RANGE_START_JD) % max_gap_days(b)).abs() < 0.5);
        if !on_grid {
            epochs.push(jd);
        }
    }
    epochs
}

#[cfg(test)]
mod epoch_tests {
    use super::*;

    #[test]
    fn boundary_brackets_both_ends() {
        let e = boundary_epochs();
        assert!(e.iter().any(|&j| j < RANGE_START_JD));
        assert!(e.iter().any(|&j| j > RANGE_END_JD));
    }

    #[test]
    fn fast_clusters_are_daily() {
        let e = fast_cluster_epochs();
        assert_eq!(e.len(), 90); // 3 anchors x 30 days
        assert!((e[1] - e[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn holdout_is_deterministic_and_in_range() {
        let a = holdout_epochs(50);
        let b = holdout_epochs(50);
        assert_eq!(a, b, "hold-out must be reproducible");
        assert_eq!(a.len(), 50);
        for jd in a {
            assert!(jd > RANGE_START_JD && jd < RANGE_END_JD);
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl epoch_tests:: 2>&1 | tail -15`
Expected: FAIL until the three functions compile and satisfy the asserts.

- [ ] **Step 3: Fix any assert mismatches**

If `holdout_is_deterministic_and_in_range` loops (rare on-grid rejection), confirm the `on_grid` modulo tolerance (0.5 day) is not rejecting nearly everything; widen the LCG draw count if needed. No behavior change beyond making asserts pass.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl epoch_tests:: 2>&1 | tail -15`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): add boundary, fast-cluster, and seeded hold-out epochs"
```

---

## Task 5: Public content-checksum helper

**Files:**
- Modify: `crates/pleiades-jpl/src/reference_summary/reference_snapshot/core/general_a.rs`
- Modify: `crates/pleiades-jpl/src/spk/corpus_manifest.rs` (created here)
- Modify: `crates/pleiades-jpl/src/spk/mod.rs`

The existing `checksum64` (FNV-1a) is `pub(crate)` deep in the tree. Expose a stable corpus-facing wrapper rather than reaching into that path.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-jpl/src/spk/corpus_manifest.rs`:

```rust
//! Corpus manifest: provenance plus per-slice FNV content checksums and row
//! counts, rendered in the repo's manifest-text style and parsed back.

/// Deterministic 64-bit content checksum (FNV-1a) used to detect drift between
/// a checked-in slice file and the manifest. Not cryptographic; pairs with the
/// gated regenerate-and-value-compare path for kernel-identity assurance.
pub fn corpus_checksum64(text: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0001_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_is_deterministic_and_sensitive() {
        assert_eq!(corpus_checksum64("abc"), corpus_checksum64("abc"));
        assert_ne!(corpus_checksum64("abc"), corpus_checksum64("abd"));
    }
}
```

In `crates/pleiades-jpl/src/spk/mod.rs` add:

```rust
pub mod corpus_manifest;
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl corpus_manifest:: 2>&1 | tail -15`
Expected: FAIL until the module is wired into `mod.rs`.

- [ ] **Step 3: No further implementation needed**

The function is defined in Step 1 (intentionally duplicating the tiny FNV constants rather than widening visibility of the deep `pub(crate)` helper; both must stay byte-identical — note this in a code comment referencing `general_a.rs::checksum64`).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl corpus_manifest:: 2>&1 | tail -15`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_manifest.rs crates/pleiades-jpl/src/spk/mod.rs
git commit -m "feat(jpl): add corpus content-checksum helper"
```

---

## Task 6: Manifest type with render + parse round-trip

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_manifest.rs`

- [ ] **Step 1: Write the failing test**

Add to `corpus_manifest.rs`:

```rust
/// One slice row in the manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct SliceEntry {
    pub name: String,
    pub file: String,
    pub role: String,
    pub rows: usize,
    pub checksum: u64,
}

/// Parsed corpus manifest.
#[derive(Clone, Debug, PartialEq)]
pub struct CorpusManifest {
    pub kernel: String,
    pub kernel_sha256: String,
    pub slices: Vec<SliceEntry>,
}

impl CorpusManifest {
    /// Renders the manifest in the repo's `#`-comment + keyword-line style.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("#Pleiades SPK Reference Corpus Manifest\n");
        out.push_str(&format!("#Kernel: {}\n", self.kernel));
        out.push_str(&format!("#Kernel-SHA256: {}\n", self.kernel_sha256));
        for s in &self.slices {
            out.push_str(&format!(
                "slice {} file={} role={} rows={} checksum={}\n",
                s.name, s.file, s.role, s.rows, s.checksum
            ));
        }
        out
    }

    /// Parses the rendered form. Unknown `#` lines are ignored; malformed
    /// `slice` lines are an error.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut kernel = String::new();
        let mut kernel_sha256 = String::new();
        let mut slices = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("#Kernel-SHA256:") {
                kernel_sha256 = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("#Kernel:") {
                kernel = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("slice ") {
                slices.push(parse_slice_line(rest)?);
            }
        }
        Ok(CorpusManifest { kernel, kernel_sha256, slices })
    }
}

fn parse_slice_line(rest: &str) -> Result<SliceEntry, String> {
    let mut parts = rest.split_whitespace();
    let name = parts.next().ok_or("slice line missing name")?.to_string();
    let mut file = String::new();
    let mut role = String::new();
    let mut rows = 0usize;
    let mut checksum = 0u64;
    for kv in parts {
        let (k, v) = kv.split_once('=').ok_or(format!("bad token: {kv}"))?;
        match k {
            "file" => file = v.to_string(),
            "role" => role = v.to_string(),
            "rows" => rows = v.parse().map_err(|_| format!("bad rows: {v}"))?,
            "checksum" => checksum = v.parse().map_err(|_| format!("bad checksum: {v}"))?,
            _ => return Err(format!("unknown key: {k}")),
        }
    }
    if file.is_empty() || role.is_empty() {
        return Err(format!("slice {name} missing file/role"));
    }
    Ok(SliceEntry { name, file, role, rows, checksum })
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    fn sample() -> CorpusManifest {
        CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "abc123".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 96,
                checksum: 42,
            }],
        }
    }

    #[test]
    fn render_parse_round_trips() {
        let m = sample();
        let parsed = CorpusManifest::parse(&m.render()).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn malformed_slice_line_errors() {
        assert!(CorpusManifest::parse("slice boundary file=b.csv\n").is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl manifest_tests:: 2>&1 | tail -15`
Expected: FAIL — types not yet present (added above; fails first if authored separately).

- [ ] **Step 3: Confirm implementation present**

All code is in Step 1. Ensure `split_once` is available (stable since Rust 1.52; the workspace pins 1.96.0, so fine).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl manifest_tests:: 2>&1 | tail -15`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_manifest.rs
git commit -m "feat(jpl): add corpus manifest render/parse round-trip"
```

---

## Task 7: Slice + bundle generation over the spec

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/generate.rs`
- Modify: `crates/pleiades-jpl/src/lib.rs`

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-jpl/src/spk/generate.rs` (above the existing `#[cfg(test)]`), a slice-aware generator built on the existing `generate_corpus_csv`:

```rust
use crate::spk::corpus_spec::{self, SliceRole};
use crate::spk::corpus_manifest::{corpus_checksum64, CorpusManifest, SliceEntry};

/// One generated slice: its role, file name, and CSV text.
pub struct GeneratedSlice {
    pub role: SliceRole,
    pub file: String,
    pub csv: String,
}

/// Generates one slice's CSV by sampling the backend at the spec-defined epochs
/// for that role, reusing the existing single-slice CSV emitter.
pub fn generate_slice(
    backend: &SpkBackend,
    role: SliceRole,
) -> Result<GeneratedSlice, String> {
    let (file, epochs, bodies) = match role {
        SliceRole::Boundary => (
            "boundary.csv",
            corpus_spec::boundary_epochs(),
            all_bodies(),
        ),
        SliceRole::InteriorBackbone => (
            "interior.csv",
            // Union of per-body backbones; generate_corpus_csv samples every
            // body at every listed epoch, so use the densest (Moon) backbone
            // and let the validator enforce per-body gaps.
            corpus_spec::interior_backbone_epochs(&CelestialBody::Moon),
            all_bodies(),
        ),
        SliceRole::FastCluster => (
            "fast_clusters.csv",
            corpus_spec::fast_cluster_epochs(),
            vec![CelestialBody::Moon, CelestialBody::Mercury, CelestialBody::Venus],
        ),
        SliceRole::Holdout => (
            "holdout.csv",
            corpus_spec::holdout_epochs(50),
            all_bodies(),
        ),
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
    // Insert the slice-role header line after the redistribution line.
    csv = csv.replace(
        "#Columns:",
        &format!("#Slice-Role: {}\n#Columns:", role.token()),
    );
    Ok(GeneratedSlice { role, file: file.to_string(), csv })
}

fn all_bodies() -> Vec<CelestialBody> {
    let mut bodies = corpus_spec::release_bodies();
    bodies.extend(corpus_spec::constrained_bodies());
    bodies
}

/// Builds the manifest for a set of generated slices.
pub fn build_manifest(slices: &[GeneratedSlice]) -> CorpusManifest {
    let entries = slices
        .iter()
        .map(|s| SliceEntry {
            name: s.role.token().to_string(),
            file: s.file.clone(),
            role: s.role.token().to_string(),
            rows: s.csv.lines().filter(|l| !l.starts_with('#')).count(),
            checksum: corpus_checksum64(&s.csv),
        })
        .collect();
    CorpusManifest {
        kernel: "de440.bsp".to_string(),
        kernel_sha256: corpus_spec::KERNEL_SHA256.to_string(),
        slices: entries,
    }
}

#[cfg(test)]
mod slice_tests {
    use super::*;
    use crate::spk::test_support::{build_daf, type2_record, type2_segment_data, SegmentSpec};

    fn const_seg(target: i32, center: i32, pos: [f64; 3]) -> SegmentSpec {
        let rec = type2_record(0.0, 1.0e12, &[pos[0], 0.0], &[pos[1], 0.0], &[pos[2], 0.0]);
        let data = type2_segment_data(-1.0e12, 2.0e12, rec.len(), &[rec]);
        SegmentSpec {
            start_et: -1.0e12, stop_et: 1.0e12, target, center,
            frame: 1, data_type: 2, data, name: "C".to_string(),
        }
    }

    fn backend() -> SpkBackend {
        // Minimal chain so Sun resolves; other bodies share the const segment.
        let blob = build_daf(&[
            const_seg(10, 0, [1.0e8, 2.0e7, 0.0]),
            const_seg(399, 3, [0.0, 0.0, 0.0]),
            const_seg(3, 0, [0.0, 0.0, 0.0]),
        ]);
        SpkBackend::builder().add_kernel_bytes(blob, "syn").unwrap().build()
    }

    #[test]
    fn boundary_slice_has_role_header_and_rows() {
        let slice = generate_slice(&backend(), SliceRole::Boundary).unwrap();
        assert!(slice.csv.contains("#Slice-Role: boundary"));
        assert!(slice.csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km"));
        assert!(slice.csv.lines().any(|l| l.starts_with("23") && l.contains("Sun")));
    }

    #[test]
    fn manifest_counts_data_rows_and_checksums() {
        let slice = generate_slice(&backend(), SliceRole::Boundary).unwrap();
        let manifest = build_manifest(std::slice::from_ref(&slice));
        assert_eq!(manifest.slices.len(), 1);
        assert!(manifest.slices[0].rows > 0);
        assert_ne!(manifest.slices[0].checksum, 0);
    }

    #[test]
    fn fixture_golden_is_not_generated() {
        assert!(generate_slice(&backend(), SliceRole::FixtureGolden).is_err());
    }
}
```

In `crates/pleiades-jpl/src/lib.rs`, extend the existing `pub use ...generate::{...}` re-export to add the new public items:

```rust
generate_corpus_csv, generate_slice, build_manifest, GeneratedSlice, CorpusRequest,
```

(Append `GeneratedSlice`, `generate_slice`, `build_manifest`; keep existing names.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl slice_tests:: 2>&1 | tail -20`
Expected: FAIL — new functions not yet compiled / synthetic backend may not resolve all bodies.

- [ ] **Step 3: Fix backend resolution in the test**

If a body fails to resolve in the synthetic backend, the test's `backend()` only needs Sun to resolve for the asserted rows; restrict the asserted body to `Sun`. If `generate_corpus_csv` errors on an unresolved body for `all_bodies()`, change the boundary/holdout test paths to pass a single-body `CorpusRequest` via a test-only helper, OR make the const segment cover the queried targets. Keep production `generate_slice` sampling `all_bodies()`; only the synthetic *test* narrows scope.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl slice_tests:: 2>&1 | tail -20`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/generate.rs crates/pleiades-jpl/src/lib.rs
git commit -m "feat(jpl): generate corpus slices and manifest from the spec"
```

---

## Task 8: Completeness-matrix gate (fail-closed)

**Files:**
- Create: `crates/pleiades-validate/src/corpus/production.rs`
- Modify: `crates/pleiades-validate/src/corpus/mod.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-validate/src/corpus/production.rs`:

```rust
//! Fail-closed gate over the checked-in production corpus slices.

use pleiades_backend::CelestialBody;
use pleiades_jpl::parse_snapshot_entries;
use pleiades_jpl::spk::corpus_spec;

/// A loaded slice: its role token and parsed entries.
pub struct LoadedSlice {
    pub role: String,
    pub csv: String,
}

/// Validates that every release-claimed body appears in the corpus and that the
/// boundary, interior, and hold-out roles are all present and non-empty.
/// Returns the first violation as `Err`, fail-closed.
pub fn validate_completeness(slices: &[LoadedSlice]) -> Result<(), String> {
    let required_roles = ["boundary", "interior", "holdout"];
    for role in required_roles {
        let slice = slices
            .iter()
            .find(|s| s.role == role)
            .ok_or(format!("missing required slice role: {role}"))?;
        let entries = parse_snapshot_entries(&slice.csv)
            .map_err(|e| format!("slice {role} failed to parse: {e:?}"))?;
        if entries.is_empty() {
            return Err(format!("slice {role} has no data rows"));
        }
    }

    // Every release body must appear somewhere in the corpus.
    let mut seen: Vec<CelestialBody> = Vec::new();
    for slice in slices {
        if let Ok(entries) = parse_snapshot_entries(&slice.csv) {
            for e in entries {
                if !seen.contains(&e.body) {
                    seen.push(e.body);
                }
            }
        }
    }
    for body in corpus_spec::release_bodies() {
        if !seen.contains(&body) {
            return Err(format!("release-claimed body missing from corpus: {body:?}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(role: &str) -> String {
        format!("#Slice-Role: {role}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n")
    }

    fn full_corpus() -> Vec<LoadedSlice> {
        let bodies = corpus_spec::release_bodies();
        let mut rows = String::new();
        for b in &bodies {
            rows.push_str(&format!("2451545,{b},1.0,2.0,3.0\n"));
        }
        ["boundary", "interior", "holdout"]
            .iter()
            .map(|r| LoadedSlice { role: r.to_string(), csv: format!("{}{}", header(r), rows) })
            .collect()
    }

    #[test]
    fn full_corpus_passes() {
        assert!(validate_completeness(&full_corpus()).is_ok());
    }

    #[test]
    fn missing_role_fails() {
        let mut corpus = full_corpus();
        corpus.retain(|s| s.role != "holdout");
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn missing_body_fails() {
        let mut corpus = full_corpus();
        // Strip Mars from every slice.
        for s in &mut corpus {
            s.csv = s.csv.lines().filter(|l| !l.contains("Mars")).collect::<Vec<_>>().join("\n");
        }
        assert!(validate_completeness(&corpus).is_err());
    }

    #[test]
    fn empty_slice_fails() {
        let mut corpus = full_corpus();
        corpus[0].csv = header("boundary");
        assert!(validate_completeness(&corpus).is_err());
    }
}
```

In `crates/pleiades-validate/src/corpus/mod.rs`, add at the top:

```rust
pub mod production;
```

This requires `pleiades_jpl::spk::corpus_spec` and `parse_snapshot_entries` to be public. Confirm `parse_snapshot_entries` is already `pub` (it is — `backend.rs:2127`) and that `spk` + `corpus_spec` are re-exported from `pleiades-jpl`'s `lib.rs`. If `spk` is not public, add `pub mod spk;` visibility or a targeted `pub use crate::spk::corpus_spec;` in `lib.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate production:: 2>&1 | tail -20`
Expected: FAIL — module/exports not yet wired.

- [ ] **Step 3: Wire the public paths**

In `crates/pleiades-jpl/src/lib.rs`, ensure the spec is reachable, e.g. add `pub use crate::spk::corpus_spec;` if `spk` is private, or make `pub mod spk;`. Re-run until the test compiles.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-validate production:: 2>&1 | tail -20`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs crates/pleiades-validate/src/corpus/mod.rs crates/pleiades-jpl/src/lib.rs
git commit -m "feat(validate): add fail-closed corpus completeness gate"
```

---

## Task 9: Schema, frame, and provenance/drift checks

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs`

- [ ] **Step 1: Write the failing test**

Add to `production.rs`:

```rust
use pleiades_jpl::spk::corpus_manifest::{corpus_checksum64, CorpusManifest};

/// Validates schema header presence, finite numeric fields, and that the
/// `#Kernel-SHA256` is not the unfilled placeholder.
pub fn validate_schema_and_provenance(slices: &[LoadedSlice]) -> Result<(), String> {
    for s in slices {
        if !s.csv.contains("#Columns:epoch_jd,body,x_km,y_km,z_km") {
            return Err(format!("slice {} missing column header", s.role));
        }
        if s.csv.contains("#Kernel-SHA256: <pinned-after-download>")
            || s.csv.contains("#Kernel-SHA256: <run shasum")
        {
            return Err(format!("slice {} has placeholder kernel SHA-256", s.role));
        }
        for line in s.csv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
            for field in line.split(',').skip(2) {
                let v: f64 = field.parse().map_err(|_| format!("non-numeric field in {}", s.role))?;
                if !v.is_finite() {
                    return Err(format!("non-finite field in {}", s.role));
                }
            }
        }
    }
    Ok(())
}

/// Validates each slice's content checksum against the manifest.
pub fn validate_drift(slices: &[LoadedSlice], manifest: &CorpusManifest) -> Result<(), String> {
    for s in slices {
        let entry = manifest
            .slices
            .iter()
            .find(|e| e.role == s.role)
            .ok_or(format!("manifest missing slice for role {}", s.role))?;
        let actual = corpus_checksum64(&s.csv);
        if actual != entry.checksum {
            return Err(format!(
                "checksum drift for {}: manifest {} != actual {}",
                s.role, entry.checksum, actual
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod drift_tests {
    use super::*;
    use pleiades_jpl::spk::corpus_manifest::SliceEntry;

    fn slice(role: &str, sha: &str) -> LoadedSlice {
        LoadedSlice {
            role: role.to_string(),
            csv: format!(
                "#Kernel-SHA256: {sha}\n#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545,Sun,1.0,2.0,3.0\n"
            ),
        }
    }

    #[test]
    fn placeholder_sha_fails() {
        let s = vec![slice("boundary", "<pinned-after-download>")];
        assert!(validate_schema_and_provenance(&s).is_err());
    }

    #[test]
    fn real_sha_passes_schema() {
        let s = vec![slice("boundary", "deadbeef")];
        assert!(validate_schema_and_provenance(&s).is_ok());
    }

    #[test]
    fn checksum_mismatch_fails() {
        let s = slice("boundary", "deadbeef");
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum: 12345, // deliberately wrong
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_err());
    }

    #[test]
    fn matching_checksum_passes() {
        let s = slice("boundary", "deadbeef");
        let checksum = corpus_checksum64(&s.csv);
        let manifest = CorpusManifest {
            kernel: "de440.bsp".to_string(),
            kernel_sha256: "deadbeef".to_string(),
            slices: vec![SliceEntry {
                name: "boundary".to_string(),
                file: "boundary.csv".to_string(),
                role: "boundary".to_string(),
                rows: 1,
                checksum,
            }],
        };
        assert!(validate_drift(&[s], &manifest).is_ok());
    }
}
```

Confirm `pleiades_jpl::spk::corpus_manifest` items are `pub` (done in Tasks 5–6).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate drift_tests:: 2>&1 | tail -20`
Expected: FAIL until functions compile.

- [ ] **Step 3: Confirm implementation present**

Code is in Step 1.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-validate drift_tests:: 2>&1 | tail -20`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs
git commit -m "feat(validate): add corpus schema and checksum-drift checks"
```

---

## Task 10: `validate-corpus` CLI command over the checked-in slices

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs`
- Modify: `crates/pleiades-validate/src/render/cli.rs`

- [ ] **Step 1: Write the failing test**

Add a top-level entry point to `production.rs` that loads the embedded slices + manifest and runs all checks:

```rust
/// Loads the checked-in corpus slices + manifest and runs every gate.
/// Returns a one-line success summary or the first violation.
pub fn run_corpus_gate() -> Result<String, String> {
    let slices = embedded_slices();
    let manifest = CorpusManifest::parse(EMBEDDED_MANIFEST)?;
    validate_completeness(&slices)?;
    validate_schema_and_provenance(&slices)?;
    validate_drift(&slices, &manifest)?;
    let rows: usize = slices
        .iter()
        .map(|s| s.csv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).count())
        .sum();
    Ok(format!(
        "corpus gate ok: {} slices, {} data rows, kernel {}",
        slices.len(),
        rows,
        manifest.kernel
    ))
}

const EMBEDDED_MANIFEST: &str =
    include_str!("../../../pleiades-jpl/data/corpus/manifest.txt");

fn embedded_slices() -> Vec<LoadedSlice> {
    let files = [
        ("boundary", include_str!("../../../pleiades-jpl/data/corpus/boundary.csv")),
        ("interior", include_str!("../../../pleiades-jpl/data/corpus/interior.csv")),
        ("fast_cluster", include_str!("../../../pleiades-jpl/data/corpus/fast_clusters.csv")),
        ("holdout", include_str!("../../../pleiades-jpl/data/corpus/holdout.csv")),
        ("fixture_golden", include_str!("../../../pleiades-jpl/data/corpus/fixture_golden.csv")),
    ];
    files
        .iter()
        .map(|(role, csv)| LoadedSlice { role: role.to_string(), csv: csv.to_string() })
        .collect()
}
```

Add a smoke test (it will fail until Task 11 fills real data + checksums; mark it `#[ignore]` with a note so it does not block CI before Task 11):

```rust
#[test]
#[ignore = "enabled after Task 11 regenerates real corpus data + checksums"]
fn embedded_corpus_gate_passes() {
    run_corpus_gate().unwrap();
}
```

In `crates/pleiades-validate/src/render/cli.rs`, add a match arm near the other corpus commands (after the `comparison-corpus-*` block, ~line 137):

```rust
        Some("validate-corpus") | Some("corpus-gate") => {
            ensure_no_extra_args(&args[1..], "validate-corpus")?;
            crate::corpus::production::run_corpus_gate()
        }
```

- [ ] **Step 2: Run test to verify it fails / compiles**

Run: `cargo build -p pleiades-validate 2>&1 | tail -20`
Expected: builds; the relative `include_str!` paths resolve from `production.rs`. If the path is wrong, fix the `../` depth (the file is at `crates/pleiades-validate/src/corpus/production.rs`, target is `crates/pleiades-jpl/data/corpus/`).

- [ ] **Step 3: Confirm the command dispatches**

Run: `cargo run -p pleiades-cli -- validate-corpus 2>&1 | tail -5` (or the validate binary directly). Before Task 11 the scaffold checksums are `0`, so expect a fail-closed drift error — that is correct behavior, not a bug.

- [ ] **Step 4: Run the non-ignored tests**

Run: `cargo test -p pleiades-validate production:: 2>&1 | tail -10`
Expected: PASS (the ignored smoke test is skipped).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs crates/pleiades-validate/src/render/cli.rs
git commit -m "feat(validate): add validate-corpus command over checked-in slices"
```

---

## Task 11: [MAINTAINER — requires de440 + asteroid kernel] Promote the generator CLI and regenerate real data

**Files:**
- Modify: `crates/pleiades-cli/src/commands/spk_corpus.rs`
- Modify: `crates/pleiades-cli/src/help.rs`
- Modify: `crates/pleiades-jpl/data/corpus/*.csv`, `manifest.txt` (regenerated output)
- Modify: `docs/spk-kernel-sourcing.md` (record the pinned SHA-256)

- [ ] **Step 1: Add an `--emit-slices` mode to the CLI command**

Replace the ad-hoc body of `render_spk_corpus` so that, given `generate-spk-corpus <kernel.bsp> --emit-slices <out-dir>`, it generates all four backend-sourced slices + manifest into `<out-dir>` and writes them with `std::fs`. Keep the legacy `<kernel> <jd...>` mode for single-shot use. Code:

```rust
use pleiades_jpl::{build_manifest, generate_slice, SpkBackend};
use pleiades_jpl::spk::corpus_spec::SliceRole;

pub fn render_spk_corpus(args: &[&str]) -> Result<String, String> {
    let kernel = args.first().ok_or("generate-spk-corpus requires a kernel path")?;
    if args.get(1).copied() == Some("--emit-slices") {
        let out_dir = args.get(2).ok_or("--emit-slices requires an output directory")?;
        let backend = SpkBackend::builder().add_kernel(kernel).map_err(|e| e.message)?.build();
        let roles = [SliceRole::Boundary, SliceRole::InteriorBackbone, SliceRole::FastCluster, SliceRole::Holdout];
        let mut generated = Vec::new();
        for role in roles {
            let slice = generate_slice(&backend, role)?;
            std::fs::write(format!("{out_dir}/{}", slice.file), &slice.csv)
                .map_err(|e| format!("write {}: {e}", slice.file))?;
            generated.push(slice);
        }
        let manifest = build_manifest(&generated);
        std::fs::write(format!("{out_dir}/manifest.txt"), manifest.render())
            .map_err(|e| format!("write manifest: {e}"))?;
        return Ok(format!("wrote {} slices + manifest to {out_dir}", generated.len()));
    }
    // ... legacy path unchanged ...
}
```

Update `crates/pleiades-cli/src/help.rs` to document `--emit-slices <out-dir>`.

- [ ] **Step 2: Download and pin the kernel**

```bash
curl -O https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp
shasum -a 256 de440.bsp
```

Paste the digest into `KERNEL_SHA256` in `corpus_spec.rs` and into `docs/spk-kernel-sourcing.md`.

- [ ] **Step 3: Regenerate the real corpus**

```bash
cargo run -p pleiades-cli -- generate-spk-corpus /path/to/de440.bsp --emit-slices crates/pleiades-jpl/data/corpus
```

Then hand-populate `fixture_golden.csv` from the existing trusted Horizons fixtures (it is not backend-generated) and add its `slice fixture_golden ...` line to `manifest.txt` with the correct `corpus_checksum64`.

- [ ] **Step 4: Enable and run the full gate**

Remove the `#[ignore]` from `embedded_corpus_gate_passes` (Task 10), then:

Run: `cargo test -p pleiades-validate production:: 2>&1 | tail -10` and `cargo run -p pleiades-cli -- validate-corpus`
Expected: PASS / `corpus gate ok: ...`.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-cli/src/ crates/pleiades-jpl/data/corpus/ crates/pleiades-jpl/src/spk/corpus_spec.rs docs/spk-kernel-sourcing.md crates/pleiades-validate/src/corpus/production.rs
git commit -m "feat(corpus): regenerate production reference corpus from de440"
```

---

## Task 12: Wire the corpus gate into the release report and fixture-golden tolerances

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs`
- Modify: `crates/pleiades-validate/src/render/cli.rs` (or the release report aggregator the repo uses)

- [ ] **Step 1: Write the failing test — fixture-golden cross-check**

Add a tolerance check comparing `fixture_golden` rows against the backbone/boundary values at shared epochs, within per-body-class tolerances from the spec. Add to `production.rs`:

```rust
/// Per-body-class position tolerance in km for the fixture-golden cross-check.
fn tolerance_km(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 5.0,
        CelestialBody::Pluto => 5_000.0, // constrained/approximate
        _ => 50.0,
    }
}

#[cfg(test)]
mod tolerance_tests {
    use super::*;

    #[test]
    fn tolerance_is_looser_for_constrained_bodies() {
        assert!(tolerance_km(&CelestialBody::Pluto) > tolerance_km(&CelestialBody::Mars));
    }
}
```

(The full cross-check comparison wiring iterates shared epochs; implement it to consume `tolerance_km` and fail closed on breach. The unit test above locks the tolerance policy; the cross-check itself is exercised by `run_corpus_gate` against real data.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate tolerance_tests:: 2>&1 | tail -10`
Expected: FAIL until `tolerance_km` compiles.

- [ ] **Step 3: Add the cross-check call into `run_corpus_gate`**

Call the fixture-golden cross-check inside `run_corpus_gate` (after `validate_drift`), excluding `constrained_bodies` from release-grade pass/fail but still reporting them.

- [ ] **Step 4: Wire `run_corpus_gate` into the default validation report**

Add a `validate-corpus` line to the aggregate report rendered by `render_validation_report` (the repo's existing release-facing report) so a corpus gap fails the release gate. Run:

Run: `cargo run -p pleiades-cli -- report 2>&1 | grep -i corpus`
Expected: a corpus-gate line appears and the report exits 0 when the corpus is valid.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/
git commit -m "feat(validate): cross-check fixture-golden and gate release on corpus"
```

---

## Task 13: Gated verify-from-kernel reproduction

**Files:**
- Modify: `crates/pleiades-jpl/tests/` (new gated integration test, alongside the existing `spk_full_kernel` test)

- [ ] **Step 1: Write the gated test**

Create `crates/pleiades-jpl/tests/corpus_regen.rs`:

```rust
//! Gated: regenerates each slice from the real kernel and compares to the
//! checked-in CSV within a tight reproducibility tolerance. Skipped unless
//! PLEIADES_DE_KERNEL points at de440.bsp.

#[test]
fn regenerated_corpus_matches_checked_in() {
    let Ok(kernel) = std::env::var("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL to run");
        return;
    };
    use pleiades_jpl::{generate_slice, SpkBackend};
    use pleiades_jpl::spk::corpus_spec::SliceRole;

    let backend = SpkBackend::builder().add_kernel(&kernel).unwrap().build();
    let regenerated = generate_slice(&backend, SliceRole::Boundary).unwrap();
    let checked_in = include_str!("../data/corpus/boundary.csv");

    // Compare data rows numerically within tolerance (not byte-exact, to allow
    // float formatting differences).
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
    let a = parse(&regenerated.csv);
    let b = parse(checked_in);
    assert_eq!(a.len(), b.len(), "row count drift vs checked-in corpus");
    for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
        assert_eq!(ka, kb, "epoch/body ordering drift");
        for i in 0..3 {
            assert!((va[i] - vb[i]).abs() < 1.0, "value drift > 1 km at {ka}");
        }
    }
}
```

- [ ] **Step 2: Run gated test without the kernel**

Run: `cargo test -p pleiades-jpl --test corpus_regen 2>&1 | tail -5`
Expected: PASS (early return / skip; prints "skipping").

- [ ] **Step 3: Run gated test with the kernel (maintainer)**

Run: `PLEIADES_DE_KERNEL=/path/to/de440.bsp cargo test -p pleiades-jpl --test corpus_regen -- --nocapture 2>&1 | tail -10`
Expected: PASS (regenerated boundary slice matches checked-in within 1 km).

- [ ] **Step 4: Document the gated job**

Add a line to `docs/spk-kernel-sourcing.md` describing `cargo test -p pleiades-jpl --test corpus_regen` as the corpus reproduction check.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/tests/corpus_regen.rs docs/spk-kernel-sourcing.md
git commit -m "test(jpl): gated verify-from-kernel corpus reproduction"
```

---

## Task 14: Final workspace gate

**Files:** none (verification only)

- [ ] **Step 1: Format, lint, test the whole workspace**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace 2>&1 | tail -20`
Expected: clean fmt, no clippy warnings, all tests pass.

- [ ] **Step 2: Run the release report end to end**

Run: `cargo run -p pleiades-cli -- report 2>&1 | tail -20`
Expected: report renders, corpus gate line present, exit 0.

- [ ] **Step 3: Sync the spec doc wording**

Update `docs/superpowers/specs/2026-06-15-production-reference-corpus-design.md` §2/§4 to replace `corpus-spec.toml`/`manifest.toml`/SHA-256-content-checksum wording with the implemented Rust-spec module + `manifest.txt` + FNV content-checksum reality (kernel SHA-256 stays a pinned constant). Commit:

```bash
git add docs/superpowers/specs/2026-06-15-production-reference-corpus-design.md
git commit -m "docs(spec): sync corpus spec wording to codebase-native realization"
```

---

## Self-Review Notes

- **Spec coverage:** storage model (Tasks 1, 7, 11), stratified slices (Tasks 3–4, 7), single source of truth (Task 2, used by both generator and gate), schema reuse (Task 7), provenance/manifest (Tasks 5–6, 11), fail-closed validation matrix (Tasks 8–10, 12), fixture-golden tolerances (Task 12), gated verify-from-kernel (Task 13), Pluto/asteroid constrained tagging (Tasks 2, 12). All spec sections map to a task.
- **Type consistency:** `SliceRole`, `corpus_checksum64`, `CorpusManifest`/`SliceEntry`, `generate_slice`/`GeneratedSlice`, `build_manifest`, `LoadedSlice`, `run_corpus_gate` are named identically across all tasks that reference them.
- **Known open items deferred to execution:** exact asteroid kernel + selected-asteroid list (carried from the SPK backend design; `constrained_bodies()` currently lists Pluto only — extend when the asteroid kernel is adopted); final per-body cadence tuning against the resulting checked-in row-count budget; exact aggregate-report wiring point in Task 12 depends on the repo's report aggregator.
