# Rebase Artifact Generation on the Phase 1 Corpus — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the `pleiades-data` packaged-artifact generator fit from the broad Phase 1 corpus (committed under `crates/pleiades-jpl/data/corpus/`) instead of the narrow `reference_snapshot()` fixture, regenerate the committed draft artifact deterministically from it, and keep it draft-labeled.

**Architecture:** Expose the committed corpus slices as typed accessors from `pleiades-jpl` (the crate that owns the CSV files), add a corpus-backed reference backend so fitting can interpolate arbitrary corpus rows, re-point the generator and its input guard at the corpus (base 10 bodies from de440 slices; `asteroid:433-Eros` from the Tier B `asteroid_constrained` slice), regenerate the committed `.bin`, dedup the duplicate corpus embedding in `pleiades-validate`, and realign posture/summary strings and their tests.

**Tech Stack:** Rust (workspace of `pleiades-*` crates), pure-Rust deterministic generation, `cargo test`, `cargo run -p pleiades-cli` / `pleiades-validate` CLI.

## Global Constraints

- Pure-Rust, no new native dependencies; preserve layered crate boundaries (`pleiades-jpl` → `pleiades-data` → `pleiades-validate`). (`spec/architecture.md`)
- Generation must be deterministic and produce byte-identical artifact bytes across runs.
- The packaged artifact stays **draft-grade**; do NOT introduce or enforce production accuracy thresholds in this slice.
- Asteroid coverage stays **constrained**; all release-facing summaries remain truthful that Eros is a constrained 1900–2100 body and asteroids are not release-grade.
- Corpus CSV schema is exactly `epoch_jd,body,x_km,y_km,z_km`; TDB time scale; geocentric ecliptic mean-geometric frame.
- Keep fitting/reference, hold-out, boundary, fixture-exactness, and provenance evidence classes separate in data and reports.
- Run `cargo fmt` and `cargo clippy` clean before each commit (workspace requires `rustfmt` + `clippy`).

## File Structure

- `crates/pleiades-jpl/src/corpus.rs` (**create**) — typed accessors over the embedded `data/corpus/*.csv` slices, returning `Vec<SnapshotEntry>` grouped by role, manifest-checked. One responsibility: expose the committed corpus as parsed entries.
- `crates/pleiades-jpl/src/backend.rs` (**modify**) — add `SnapshotCorpusBackend` (holds arbitrary entries, implements `EphemerisBackend` via existing `resolve_fixture_state_from_entries`).
- `crates/pleiades-jpl/src/lib.rs` (**modify**) — register/export the new `corpus` module and `SnapshotCorpusBackend`.
- `crates/pleiades-data/src/regenerate.rs` (**modify**) — source generation input from the corpus accessors + Eros; generalize the reference-backend parameter; re-point the input guard.
- `crates/pleiades-data/tests/fixtures/packaged-artifact.bin` (**modify**) — regenerated draft artifact bytes.
- `crates/pleiades-validate/src/corpus/production.rs` (**modify**) — replace the `../../../pleiades-jpl/...` `include_str!` block with the new `pleiades-jpl` accessors.
- Posture/summary string sites + tests (**modify**) — listed in Task 6.
- `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md` (**modify**) — Task 7 doc cleanup.

---

### Task 1: Typed corpus accessors in `pleiades-jpl`

**Files:**
- Create: `crates/pleiades-jpl/src/corpus.rs`
- Modify: `crates/pleiades-jpl/src/lib.rs` (register `mod corpus;` + `pub use corpus::*;`)
- Test: inline `#[cfg(test)]` module in `crates/pleiades-jpl/src/corpus.rs`

**Interfaces:**
- Consumes: `parse_snapshot_entries(&str) -> Result<Vec<SnapshotEntry>, SnapshotLoadError>` and `SnapshotEntry` (from `crate::backend`); `crate::spk::corpus_manifest` for manifest row-count parsing; `pleiades_backend::{CelestialBody, CustomBodyId}`.
- Produces:
  - `pub fn production_reference_corpus() -> &'static [SnapshotEntry]` — `interior ∪ boundary ∪ fast_clusters`, base-body fitting rows.
  - `pub fn production_holdout_corpus() -> &'static [SnapshotEntry]` — holdout rows.
  - `pub fn fixture_golden_corpus() -> &'static [SnapshotEntry]` — fixture-exactness rows.
  - `pub fn asteroid_reference_corpus() -> &'static [SnapshotEntry]` — Tier A rows.
  - `pub fn asteroid_constrained_corpus() -> &'static [SnapshotEntry]` — Tier B rows.
  - `pub fn asteroid_constrained_entries_for(body: &CelestialBody) -> Vec<SnapshotEntry>` — rows for one constrained body (used to pull Eros).

- [ ] **Step 1: Write the failing test**

Add to the bottom of the new `crates/pleiades-jpl/src/corpus.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::{CelestialBody, CustomBodyId};

    fn base_bodies() -> Vec<CelestialBody> {
        vec![
            CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mercury,
            CelestialBody::Venus, CelestialBody::Mars, CelestialBody::Jupiter,
            CelestialBody::Saturn, CelestialBody::Uranus, CelestialBody::Neptune,
            CelestialBody::Pluto,
        ]
    }

    #[test]
    fn production_reference_corpus_covers_base_bodies_only() {
        let entries = production_reference_corpus();
        assert!(!entries.is_empty(), "reference corpus should parse non-empty");
        let mut bodies: Vec<_> = entries.iter().map(|e| e.body.clone()).collect();
        bodies.sort();
        bodies.dedup();
        for body in base_bodies() {
            assert!(bodies.contains(&body), "missing base body {body}");
        }
        assert!(
            bodies.iter().all(|b| !matches!(b, CelestialBody::Custom(_))),
            "reference corpus must not contain custom/asteroid bodies"
        );
    }

    #[test]
    fn reference_corpus_row_count_matches_manifest_sum() {
        // interior(24813) + boundary(60) + fast_cluster(270) = 25143
        assert_eq!(production_reference_corpus().len(), 25_143);
    }

    #[test]
    fn holdout_corpus_is_separate_and_nonempty() {
        assert_eq!(production_holdout_corpus().len(), 500);
    }

    #[test]
    fn asteroid_constrained_includes_eros() {
        let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
        let rows = asteroid_constrained_entries_for(&eros);
        assert!(!rows.is_empty(), "asteroid_constrained should contain Eros rows");
        assert!(rows.iter().all(|e| e.body == eros));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl --lib corpus::tests`
Expected: FAIL to compile — `production_reference_corpus` and siblings are not defined.

- [ ] **Step 3: Write minimal implementation**

Create `crates/pleiades-jpl/src/corpus.rs` (above the test module):

```rust
//! Typed accessors over the committed production reference corpus.
//!
//! The CSV slices live under `crates/pleiades-jpl/data/corpus/` and share the
//! `epoch_jd,body,x_km,y_km,z_km` schema. These accessors parse them once into
//! `SnapshotEntry` values so both the artifact generator (`pleiades-data`) and
//! the `validate-corpus` gate (`pleiades-validate`) consume one source.

use std::sync::OnceLock;

use pleiades_backend::CelestialBody;

use crate::backend::{parse_snapshot_entries, SnapshotEntry};

const INTERIOR_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/interior.csv"));
const BOUNDARY_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/boundary.csv"));
const FAST_CLUSTERS_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/fast_clusters.csv"));
const HOLDOUT_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/holdout.csv"));
const FIXTURE_GOLDEN_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/fixture_golden.csv"));
const ASTEROID_REFERENCE_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/asteroid_reference.csv"));
const ASTEROID_CONSTRAINED_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/asteroid_constrained.csv"));

fn parse_or_panic(label: &str, source: &str) -> Vec<SnapshotEntry> {
    parse_snapshot_entries(source)
        .unwrap_or_else(|error| panic!("committed corpus slice `{label}` failed to parse: {error}"))
}

/// Base-body fitting rows: interior ∪ boundary ∪ fast_clusters.
pub fn production_reference_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries = parse_or_panic("interior", INTERIOR_CSV);
        entries.extend(parse_or_panic("boundary", BOUNDARY_CSV));
        entries.extend(parse_or_panic("fast_clusters", FAST_CLUSTERS_CSV));
        entries
    })
}

/// Independent hold-out rows (excluded from fitting).
pub fn production_holdout_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| parse_or_panic("holdout", HOLDOUT_CSV)).as_slice()
}

/// Fixture-exactness cross-check rows.
pub fn fixture_golden_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| parse_or_panic("fixture_golden", FIXTURE_GOLDEN_CSV)).as_slice()
}

/// Tier A asteroid reference rows (sb441-n16).
pub fn asteroid_reference_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| parse_or_panic("asteroid_reference", ASTEROID_REFERENCE_CSV)).as_slice()
}

/// Tier B constrained asteroid rows (Horizons, 1900–2100).
pub fn asteroid_constrained_corpus() -> &'static [SnapshotEntry] {
    static ENTRIES: OnceLock<Vec<SnapshotEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| parse_or_panic("asteroid_constrained", ASTEROID_CONSTRAINED_CSV)).as_slice()
}

/// Returns the constrained-corpus rows for a single body (e.g. Eros).
pub fn asteroid_constrained_entries_for(body: &CelestialBody) -> Vec<SnapshotEntry> {
    asteroid_constrained_corpus()
        .iter()
        .filter(|entry| &entry.body == body)
        .cloned()
        .collect()
}
```

Register the module in `crates/pleiades-jpl/src/lib.rs` next to `mod snapshot;` (around line 51) and add the re-export near the other `pub use` lines (around line 55):

```rust
mod corpus;
pub use corpus::*;
```

Note: `parse_snapshot_entries` and `SnapshotEntry` are already `pub` in `backend.rs`; if `crate::backend::parse_snapshot_entries` is not directly reachable, use the crate-root re-export path `crate::{parse_snapshot_entries, SnapshotEntry}` instead (backend is re-exported via `pub use backend::*;` at lib.rs:68).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl --lib corpus::tests`
Expected: PASS (4 tests). If `reference_corpus_row_count_matches_manifest_sum` fails, read the actual count from the failure and reconcile against `crates/pleiades-jpl/data/corpus/manifest.txt` (interior+boundary+fast_cluster), then correct the expected constant.

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-jpl && cargo clippy -p pleiades-jpl --all-targets -- -D warnings
git add crates/pleiades-jpl/src/corpus.rs crates/pleiades-jpl/src/lib.rs
git commit -m "feat(jpl): expose typed production corpus accessors"
```

---

### Task 2: Corpus-backed reference backend in `pleiades-jpl`

**Files:**
- Modify: `crates/pleiades-jpl/src/backend.rs` (add `SnapshotCorpusBackend` near `JplSnapshotBackend`, ~line 175)
- Test: inline `#[cfg(test)]` in `crates/pleiades-jpl/src/backend.rs` (or `backend/tests.rs` if that is where backend tests live)

**Interfaces:**
- Consumes: `resolve_fixture_state_from_entries(&[SnapshotEntry], CelestialBody, f64) -> Result<ResolvedFixtureState, EphemerisError>` (private fn in `backend.rs`, same module — reachable); `validate_request_policy`, `validate_zodiac_policy`, `validate_observer_policy` (already used by `JplSnapshotBackend::position`); `pleiades_backend::{EphemerisBackend, EphemerisRequest, EphemerisResult, BackendId, Motion}`.
- Produces: `pub struct SnapshotCorpusBackend { entries: Vec<SnapshotEntry> }` with `pub fn from_entries(entries: Vec<SnapshotEntry>) -> Self`, implementing `EphemerisBackend` so callers can `.position(&req)`. Re-exported from the crate root via the existing `pub use backend::*;`.

- [ ] **Step 1: Write the failing test**

Add to the backend test module:

```rust
#[test]
fn snapshot_corpus_backend_resolves_exact_corpus_epoch() {
    use pleiades_backend::{CoordinateFrame, EphemerisBackend, EphemerisRequest, Instant, JulianDay, TimeScale, ZodiacMode, Apparentness};

    // Two adjacent Sun samples so interpolation/exact lookup has a window.
    let entries = vec![
        SnapshotEntry { body: CelestialBody::Sun, epoch: Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb), x_km: 1.0, y_km: 2.0, z_km: 3.0 },
        SnapshotEntry { body: CelestialBody::Sun, epoch: Instant::new(JulianDay::from_days(2_451_546.0), TimeScale::Tdb), x_km: 4.0, y_km: 5.0, z_km: 6.0 },
    ];
    let backend = SnapshotCorpusBackend::from_entries(entries);
    let req = EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb),
        CoordinateFrame::Ecliptic,
        ZodiacMode::Tropical,
        Apparentness::Mean,
    );
    let result = backend.position(&req).expect("exact corpus epoch should resolve");
    let ecliptic = result.ecliptic.expect("ecliptic output");
    // Exact lookup returns the stored sample's ecliptic; assert it is finite and present.
    assert!(ecliptic.longitude.degrees().is_finite());
}
```

If `EphemerisRequest::new` / `SnapshotEntry` literal fields differ from the above, match the exact shapes already used in `backend.rs` (see `resolve_fixture_state` callers and the `SnapshotEntry` struct definition) — do not invent fields.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl --lib snapshot_corpus_backend_resolves_exact_corpus_epoch`
Expected: FAIL to compile — `SnapshotCorpusBackend` not defined.

- [ ] **Step 3: Write minimal implementation**

In `crates/pleiades-jpl/src/backend.rs`, after the `JplSnapshotBackend` `impl EphemerisBackend` block (~line 268), add:

```rust
/// A reference backend backed by an explicit set of snapshot entries.
///
/// Unlike [`JplSnapshotBackend`], which reads the global narrow reference
/// snapshot, this backend interpolates whatever corpus rows it is constructed
/// over. It is used by the packaged-artifact generator to fit against the broad
/// production reference corpus.
#[derive(Clone, Debug)]
pub struct SnapshotCorpusBackend {
    entries: Vec<SnapshotEntry>,
}

impl SnapshotCorpusBackend {
    /// Builds a corpus-backed backend over the given entries.
    pub fn from_entries(entries: Vec<SnapshotEntry>) -> Self {
        Self { entries }
    }
}

impl EphemerisBackend for SnapshotCorpusBackend {
    fn metadata(&self) -> BackendMetadata {
        // Mirror JplSnapshotBackend's posture; coverage derives from the held entries.
        JplSnapshotBackend.metadata()
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        validate_request_policy(
            req,
            "the JPL snapshot corpus backend",
            &[TimeScale::Tt, TimeScale::Tdb],
            &[CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial],
            true,
            false,
        )?;
        validate_zodiac_policy(req, "the JPL snapshot corpus backend", &[ZodiacMode::Tropical])?;
        validate_observer_policy(req, "the JPL snapshot corpus backend", false)?;

        let resolved = resolve_fixture_state_from_entries(
            &self.entries,
            req.body.clone(),
            req.instant.julian_day.days(),
        )?;

        let mut result = EphemerisResult::new(
            BackendId::new("jpl-snapshot"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        let ecliptic = resolved.entry.ecliptic();
        result.ecliptic = Some(ecliptic);
        result.equatorial = Some(ecliptic.to_equatorial(req.instant.mean_obliquity()));
        result.motion = None::<Motion>;
        result.quality = resolved.quality;
        Ok(result)
    }
}
```

If `resolve_fixture_state_from_entries` is `fn` (private) and the new struct is in the same `backend.rs` module, it is already reachable — no visibility change needed. Ensure `BackendMetadata` and the `validate_*` helpers are in scope (they are, since `JplSnapshotBackend` uses them in the same file).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl --lib snapshot_corpus_backend_resolves_exact_corpus_epoch`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-jpl && cargo clippy -p pleiades-jpl --all-targets -- -D warnings
git add crates/pleiades-jpl/src/backend.rs
git commit -m "feat(jpl): add SnapshotCorpusBackend over explicit entries"
```

---

### Task 3: Rebase the generator onto the corpus

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs`
- Test: `crates/pleiades-data/src/tests/coverage.rs` (or the regenerate test module — match where existing `regenerate_packaged_artifact` tests live)

**Interfaces:**
- Consumes (from Tasks 1–2): `pleiades_jpl::{production_reference_corpus, asteroid_constrained_entries_for, SnapshotCorpusBackend}`.
- Produces: `regenerate_packaged_artifact() -> CompressedArtifact` now sourced from the corpus; the per-body fitting functions accept `&dyn EphemerisBackend` instead of `&JplSnapshotBackend`.

- [ ] **Step 1: Write the failing test**

Add to the data-crate test module that already exercises regeneration:

```rust
#[test]
fn regenerated_artifact_sources_base_bodies_and_eros_from_corpus() {
    use pleiades_backend::{CelestialBody, CustomBodyId};
    let artifact = crate::regenerate::regenerate_packaged_artifact();
    let bodies: Vec<_> = artifact.bodies.iter().map(|b| b.body.clone()).collect();

    // All 10 base bodies present.
    for body in [
        CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mercury,
        CelestialBody::Venus, CelestialBody::Mars, CelestialBody::Jupiter,
        CelestialBody::Saturn, CelestialBody::Uranus, CelestialBody::Neptune,
        CelestialBody::Pluto,
    ] {
        assert!(bodies.contains(&body), "missing base body {body}");
    }
    // Eros present and span-bounded to its constrained corpus rows (1900-2100).
    let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
    let eros_artifact = artifact.bodies.iter().find(|b| b.body == eros)
        .expect("Eros should be present in the packaged artifact");
    assert!(!eros_artifact.segments.is_empty(), "Eros should have fit segments");
}

#[test]
fn holdout_rows_are_not_used_as_generation_input() {
    // The generation input is the reference corpus + Eros; holdout must be excluded.
    let input = crate::regenerate::packaged_artifact_generation_input();
    let holdout = pleiades_jpl::production_holdout_corpus();
    let input_keys: std::collections::HashSet<(String, u64)> = input.iter()
        .map(|e| (e.body.to_string(), e.epoch.julian_day.days().to_bits()))
        .collect();
    let overlap = holdout.iter().filter(|h| {
        input_keys.contains(&(h.body.to_string(), h.epoch.julian_day.days().to_bits()))
    }).count();
    assert_eq!(overlap, 0, "hold-out rows must not appear in the fitting input");
}
```

Adjust field access (`artifact.bodies`, `b.body`, `b.segments`) to the real `CompressedArtifact` / `BodyArtifact` field names used elsewhere in `regenerate.rs` (see `BodyArtifact::new(body, segments)` at the existing call site).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data --lib regenerated_artifact_sources_base_bodies_and_eros_from_corpus holdout_rows_are_not_used_as_generation_input`
Expected: FAIL — `packaged_artifact_generation_input` not defined and Eros sourcing not yet wired.

- [ ] **Step 3: Write minimal implementation**

In `crates/pleiades-data/src/regenerate.rs`:

1. Update the import (line 14–17) to add the new symbols:

```rust
use pleiades_jpl::{
    production_generation_source_summary, production_reference_corpus,
    asteroid_constrained_entries_for, reference_snapshot_summary, SnapshotCorpusBackend,
    SnapshotEntry,
};
```

(Drop `reference_snapshot` and `JplSnapshotBackend` from this import if they become unused; keep `EphemerisBackend` from the `pleiades_backend` import at the top.)

2. Add the generation-input assembler and re-point `regenerate_packaged_artifact()`:

```rust
/// The exact entries fed to packaged-artifact fitting: the base-body reference
/// corpus plus the constrained Eros rows. Hold-out rows are intentionally excluded.
pub(crate) fn packaged_artifact_generation_input() -> Vec<SnapshotEntry> {
    let mut entries = production_reference_corpus().to_vec();
    let eros = pleiades_backend::CelestialBody::Custom(
        pleiades_backend::CustomBodyId::new("asteroid", "433-Eros"),
    );
    entries.extend(asteroid_constrained_entries_for(&eros));
    entries
}
```

Change `regenerate_packaged_artifact()` (line 134–140) to source from the corpus input:

```rust
pub fn regenerate_packaged_artifact() -> CompressedArtifact {
    static ARTIFACT: OnceLock<CompressedArtifact> = OnceLock::new();
    ARTIFACT
        .get_or_init(|| regenerate_packaged_artifact_from_snapshot(&packaged_artifact_generation_input()))
        .clone()
}
```

3. Re-point the input guard. Replace the body of `validate_packaged_artifact_reference_snapshot_inputs` (line 62–89) so it validates against `packaged_artifact_generation_input()` instead of `reference_snapshot()`:

```rust
fn validate_packaged_artifact_reference_snapshot_inputs(
    snapshot: &[SnapshotEntry],
) -> Result<(), pleiades_compression::CompressionError> {
    let expected = packaged_artifact_generation_input();
    if snapshot.len() != expected.len() {
        return Err(pleiades_compression::CompressionError::new(
            pleiades_compression::CompressionErrorKind::InvalidFormat,
            format!(
                "packaged artifact regeneration input length {} does not match the production corpus input length {}",
                snapshot.len(),
                expected.len()
            ),
        ));
    }
    for (index, (actual, expected)) in snapshot.iter().zip(&expected).enumerate() {
        if actual != expected {
            return Err(pleiades_compression::CompressionError::new(
                pleiades_compression::CompressionErrorKind::InvalidFormat,
                format!(
                    "packaged artifact regeneration input at index {index} does not match the production corpus: expected {expected:?}; found {actual:?}",
                ),
            ));
        }
    }
    Ok(())
}
```

4. Replace the reference backend used during fitting. In `packaged_body_artifacts_from_snapshot` (line 186) change:

```rust
                let reference_backend = JplSnapshotBackend;
```

to construct a corpus-backed backend over the full input once (hoist it above the `thread::scope` and pass a shared `&dyn EphemerisBackend` into each spawned closure):

```rust
        let reference_backend = SnapshotCorpusBackend::from_entries(snapshot.to_vec());
        let reference_backend: &dyn EphemerisBackend = &reference_backend;
```

5. Change every `reference_backend: &JplSnapshotBackend` parameter to `reference_backend: &dyn EphemerisBackend` in: `body_segments_from_entries` (211), `segment_from_pair` (1252), `segment_from_pair_fit_attempt` (1346), `body_segment_windows_for_interval` (928), `segment_with_optional_residual_channels` (1887), and the residual helper at line 2002, plus the dense-fit helper at 619. The functions only call `.position(&request)`, which is available on the trait object.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-data --lib regenerated_artifact_sources_base_bodies_and_eros_from_corpus holdout_rows_are_not_used_as_generation_input`
Expected: PASS. Then run the whole data-crate lib tests to see which existing regeneration/posture tests now fail (expected — addressed in Tasks 4 and 6): `cargo test -p pleiades-data --lib` (note failures; do not fix posture strings yet).

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-data && cargo clippy -p pleiades-data --all-targets -- -D warnings
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/coverage.rs
git commit -m "feat(data): fit packaged artifact from the Phase 1 production corpus"
```

---

### Task 4: Regenerate the committed draft artifact bytes

**Files:**
- Modify: `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`

**Interfaces:**
- Consumes: the corpus-sourced `regenerate_packaged_artifact_bytes()` from Task 3.
- Produces: committed `.bin` whose bytes equal the corpus-derived regeneration, so `generate-packaged-artifact --check` and the in-crate `regenerate == committed` test both pass.

- [ ] **Step 1: Confirm the failing equality test**

The data crate has a test asserting `regenerate_packaged_artifact_bytes() == packaged_artifact_bytes()` (the committed `.bin`). Find it:

Run: `cargo test -p pleiades-data --lib -- --list 2>/dev/null | grep -i "regenerat\|byte\|fixture"`
Then run that test.
Expected: FAIL — committed bytes are the old narrow-fixture artifact; regeneration now differs.

- [ ] **Step 2: Write the new bytes from the corpus-sourced generator**

Run (writes the regenerated artifact over the committed fixture):

```bash
cargo run -p pleiades-cli -- validate generate-packaged-artifact --out crates/pleiades-data/tests/fixtures/packaged-artifact.bin
```

If `pleiades-cli` is not the validate entrypoint, use the binary that exposes the `validate` subcommands (check `crates/pleiades-cli/Cargo.toml` / `crates/pleiades-validate` bins). The handler is at `crates/pleiades-validate/src/render/cli.rs:166`.

- [ ] **Step 3: Verify determinism and the check path**

Run:

```bash
cargo run -p pleiades-cli -- validate generate-packaged-artifact --check
```

Expected: success / "matches" (exit 0). Run it twice to confirm byte-stability.

- [ ] **Step 4: Run the equality test to verify it passes**

Run: the test from Step 1.
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/tests/fixtures/packaged-artifact.bin
git commit -m "chore(data): regenerate draft artifact bytes from production corpus"
```

---

### Task 5: Deduplicate the corpus embedding in `pleiades-validate`

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs:216-256`

**Interfaces:**
- Consumes (from Task 1): the `pleiades-jpl` corpus accessors. The validate gate currently works on `LoadedSlice { role, csv }` over embedded CSV text. The cleanest dedup keeps `EMBEDDED_MANIFEST` from the same path but removes the per-slice `include_str!` of CSVs that now have a typed source.
- Produces: `embedded_slices()` returning the same `Vec<LoadedSlice>` content, but the CSV text comes from a single embedding owned by `pleiades-jpl`.

- [ ] **Step 1: Confirm the existing drift test still passes (baseline)**

Run: `cargo test -p pleiades-validate --lib corpus::production`
Expected: PASS (this is the current state; we must keep it passing after the change).

- [ ] **Step 2: Add a raw-slice accessor in `pleiades-jpl` corpus module**

The validate gate needs raw CSV text per role (it re-parses for checksum/row checks). Add to `crates/pleiades-jpl/src/corpus.rs`:

```rust
/// Raw `(role, csv_text)` pairs for the committed corpus slices, in gate order.
pub fn corpus_slice_sources() -> &'static [(&'static str, &'static str)] {
    &[
        ("boundary", BOUNDARY_CSV),
        ("interior", INTERIOR_CSV),
        ("fast_cluster", FAST_CLUSTERS_CSV),
        ("holdout", HOLDOUT_CSV),
        ("fixture_golden", FIXTURE_GOLDEN_CSV),
        ("asteroid_reference", ASTEROID_REFERENCE_CSV),
        ("asteroid_constrained", ASTEROID_CONSTRAINED_CSV),
    ]
}

/// The committed corpus manifest text.
pub fn corpus_manifest_source() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/corpus/manifest.txt"))
}
```

- [ ] **Step 3: Re-point `pleiades-validate` at the accessors**

In `crates/pleiades-validate/src/corpus/production.rs`, replace `EMBEDDED_MANIFEST` (line 216) and `embedded_slices()` (218–256) with:

```rust
fn embedded_manifest() -> &'static str {
    pleiades_jpl::corpus_manifest_source()
}

fn embedded_slices() -> Vec<LoadedSlice> {
    pleiades_jpl::corpus_slice_sources()
        .iter()
        .map(|(role, csv)| LoadedSlice {
            role: role.to_string(),
            csv: csv.to_string(),
        })
        .collect()
}
```

Update the single use of `EMBEDDED_MANIFEST` elsewhere in the file to call `embedded_manifest()` instead (grep within the file for `EMBEDDED_MANIFEST`).

- [ ] **Step 4: Run the gate tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib corpus::production`
Then exercise the live gate: `cargo run -p pleiades-cli -- validate validate-corpus`
Expected: both PASS / "corpus gate ok".

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy -p pleiades-jpl -p pleiades-validate --all-targets -- -D warnings
git add crates/pleiades-jpl/src/corpus.rs crates/pleiades-validate/src/corpus/production.rs
git commit -m "refactor(validate): consume committed corpus via pleiades-jpl accessors"
```

---

### Task 6: Realign posture/summary strings and tests

**Files (known sites — confirm via failing tests):**
- Modify: `crates/pleiades-jpl/src/reference_summary/production_generation.rs` (lines ~575, ~650 — the "input path=checked-in CSV fixtures via include_str! reference_snapshot.csv" posture strings)
- Modify: `crates/pleiades-jpl/src/reference_summary/production_generation/tests.rs:612` (asserts the old posture)
- Modify: `crates/pleiades-data/src/coverage/target.rs` (the `phase2_corpus_alignment` summary, if its description text references narrow-fixture sourcing)
- Modify: `crates/pleiades-data/src/tests/coverage.rs` (the `phase2_corpus_alignment` / source-fit-holdout-sync assertions, ~lines 1900–2129)
- Modify: any other site flagged by the test run below.

**Interfaces:**
- Consumes: nothing new.
- Produces: posture/summary strings that truthfully describe corpus sourcing (base bodies from the de440 reference corpus; Eros from the Tier B constrained slice; hold-out separate; artifact still draft-grade; asteroids constrained).

- [ ] **Step 1: Enumerate the failing assertions (this is the "failing test")**

Run the full workspace test suite and capture failures:

```bash
cargo test --workspace 2>&1 | tee /tmp/posture-failures.txt | grep -E "test .* FAILED|panicked|assertion" | head -80
```

Expected: a set of posture/summary equality failures whose messages quote the old narrow-fixture text (e.g. `reference_snapshot.csv`). These enumerate exactly which strings to update.

- [ ] **Step 2: Update each posture/summary string to describe corpus sourcing**

For each failing assertion, update BOTH the source string and the test's expected string in lockstep. Replace narrow-fixture descriptions such as:

```
input path=checked-in CSV fixtures via include_str! reference_snapshot.csv and independent_holdout_snapshot.csv
```

with corpus-sourced wording, for example:

```
input path=committed production corpus slices via pleiades-jpl corpus accessors (interior, boundary, fast_clusters; Eros from asteroid_constrained); hold-out held separate
```

Keep every release-facing claim truthful: artifact remains draft-grade; Eros/asteroids remain constrained (1900–2100); evidence classes stay separate. Do not change numeric thresholds or introduce production-grade claims.

- [ ] **Step 3: Re-run the targeted modules to verify they pass**

Run:

```bash
cargo test -p pleiades-jpl --lib reference_summary::production_generation
cargo test -p pleiades-data --lib coverage
```

Expected: PASS.

- [ ] **Step 4: Run the full suite to confirm green**

Run: `cargo test --workspace`
Expected: PASS (0 failures). Fix any remaining quoted-posture assertions the same way until green.

- [ ] **Step 5: Commit**

```bash
cargo fmt && cargo clippy --workspace --all-targets -- -D warnings
git add -A
git commit -m "docs(jpl,data): realign generation posture summaries to corpus sourcing"
```

---

### Task 7: Plan/status documentation cleanup

**Files:**
- Modify: `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`

**Interfaces:** documentation only.

- [ ] **Step 1: Prune stale Phase 1 entries**

In `plan/status/02-next-slice-candidates.md`, remove the two Phase 1 bullets (broad public-data reader; asteroid SPK kernel adoption) — both are Met. In `plan/status/01-current-execution-frontier.md`, update the "Frontier"/"Recommended next slice" prose to state that generation is now rebased on the Phase 1 corpus and the next slice is Phase 2 threshold definition.

- [ ] **Step 2: Update the Phase 2 stage and PLAN status footer**

In `plan/stages/02-production-compressed-ephemeris.md`, mark "Rebase artifact generation on the Phase 1 production reference and hold-out inputs" as done (remove it from "Remaining implementation work", per the plan-maintenance rule "remove tasks when implemented"). Update the `Status:` footer in `PLAN.md` to record this slice.

- [ ] **Step 3: Verify no contradictions**

Re-read the three files; confirm they consistently state generation is corpus-sourced, the artifact is still draft-grade, and thresholds are the next slice.

- [ ] **Step 4: Commit**

```bash
git add PLAN.md plan/status/01-current-execution-frontier.md plan/status/02-next-slice-candidates.md plan/stages/02-production-compressed-ephemeris.md
git commit -m "docs(plan): record corpus-rebased generation; prune Met Phase 1 entries"
```

---

## Self-Review

**Spec coverage:**
- Corpus accessors in `pleiades-jpl` → Task 1. ✓
- Corpus-backed reference backend → Task 2. ✓
- Generator rebase + Eros from `asteroid_constrained` + hold-out excluded + input guard re-point → Task 3. ✓
- Regenerate committed bytes, draft-labeled, deterministic `--check` → Task 4. ✓
- Dedup the validate embedding → Task 5. ✓
- Posture/summary string + test churn → Task 6. ✓
- Out-of-band plan/status cleanup → Task 7. ✓
- Draft-grade / constrained-asteroid / no-threshold constraints → Global Constraints + reinforced in Tasks 4 and 6. ✓

**Placeholder scan:** No "TBD"/"handle edge cases"/"similar to Task N"; each code step shows code; commands have expected output. Task 6 is intentionally test-driven (the failing assertions enumerate the exact strings) with known file sites listed — not a placeholder. ✓

**Type consistency:** `production_reference_corpus`, `production_holdout_corpus`, `asteroid_constrained_entries_for`, `corpus_slice_sources`, `corpus_manifest_source`, `SnapshotCorpusBackend::from_entries`, `packaged_artifact_generation_input` are used with consistent names/signatures across tasks. Tasks note where field/constructor shapes must be matched to the real code (`SnapshotEntry`, `EphemerisRequest`, `CompressedArtifact`/`BodyArtifact`) rather than invented. ✓
