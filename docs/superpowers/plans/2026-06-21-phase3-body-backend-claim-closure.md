# Phase 3 — Body & Backend Claim Closure Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the global, string-matched body/backend claim layer with a per-backend `BodyClaim` model derived from each backend's metadata, promote the bodies their backends already validate (per spec), and enforce honesty + cross-surface agreement with an audit + drift gate.

**Architecture:** Each `BackendMetadata` carries `body_claims: Vec<BodyClaim>` (tier + per-body accuracy + evidence) as the single source of truth, replacing `body_coverage: Vec<CelestialBody>`. Each backend crate exposes a static `*_body_claims()` declaration; `pleiades-validate` aggregates the four release-facing backends into a derived `ReleasePosture` that all summaries/matrices/CLI render from. A new `claims-audit` gate proves declared tiers against actual support + SP3 accuracy ceilings (slow, `#[ignore]`'d) and that every rendered surface matches the derived posture (drift, fast).

**Tech Stack:** Rust workspace (`pleiades-*` crates), `cargo test`, `mise` task runner, `serde` (feature-gated), existing SP3 thresholds (`pleiades-data`) and comparison harness (`pleiades-validate`).

Spec: `docs/superpowers/specs/2026-06-20-phase3-body-backend-claim-closure-design.md`
Branch: `phase3-body-backend-claim-closure` (already created)

## Global Constraints

- **Pure Rust, no native dependencies** — first-party crates must build offline; the workspace audit forbids native-dep drift.
- **Experimental `0.2.x` crates** — breaking API changes are acceptable; `BackendMetadata.body_coverage` is replaced outright with no deprecated shim.
- **Fail-closed gates** — all new validation surfaces return structured errors and fail closed, matching `validate-corpus` / size-gate culture.
- **Test-speed discipline** — slow tests (corpus comparison) are `#[ignore]`'d and run only via `mise test-full` / CI; the default `mise test` stays fast. `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` must pass.
- **`ReleaseGrade` bar** — a body is `ReleaseGrade` for a backend only if its computed positions pass the SP3 per-body-class accuracy ceiling (`pleiades_data::accuracy_ceiling`) against the de440/sb441 corpus over 1900–2100; Tier-A asteroids additionally require `sb441-n16` evidence.
- **Canonical release-backend set** — the four release-facing backends: `pleiades-data` (packaged), `jpl-spk`, `pleiades-elp`, `pleiades-vsop87`. `jpl-snapshot` is a validation fixture, excluded from the aggregated posture.
- **Claim tiers** — `ReleaseGrade > Constrained > Approximate > Unsupported` (rank 3→0); on backend body collision the stronger tier wins in merges.

---

### Task 1: Claim types (`BodyClaimTier`, `ClaimEvidence`, `BodyClaim`)

**Files:**
- Create: `crates/pleiades-backend/src/claims.rs`
- Create: `crates/pleiades-backend/src/claims_tests.rs`
- Modify: `crates/pleiades-backend/src/lib.rs` (add `mod claims;` near the other `mod` lines ~66–77; add re-export block near the `identity` re-export ~line 90)

**Interfaces:**
- Produces: `BodyClaimTier::{ReleaseGrade, Constrained, Approximate, Unsupported}` with `label(self) -> &'static str` and `rank(self) -> u8`; `ClaimEvidence::{ArtifactValidated, CorpusValidated{source:String}, AlgorithmicModel, None}` with `label(&self) -> String`; `BodyClaim { pub body: CelestialBody, pub tier: BodyClaimTier, pub accuracy: AccuracyClass, pub evidence: ClaimEvidence }` with constructors `new`, `release_grade`, `constrained`, `approximate`, `unsupported`, `summary_line(&self) -> String`, and `impl From<CelestialBody> for BodyClaim` (→ `constrained`, `AccuracyClass::Unknown`, `ClaimEvidence::None`).

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-backend/src/claims_tests.rs`:

```rust
use super::claims::{BodyClaim, BodyClaimTier, ClaimEvidence};
use crate::{AccuracyClass, CelestialBody};

#[test]
fn tier_rank_orders_release_grade_strongest() {
    assert!(BodyClaimTier::ReleaseGrade.rank() > BodyClaimTier::Constrained.rank());
    assert!(BodyClaimTier::Constrained.rank() > BodyClaimTier::Approximate.rank());
    assert!(BodyClaimTier::Approximate.rank() > BodyClaimTier::Unsupported.rank());
}

#[test]
fn tier_labels_are_stable() {
    assert_eq!(BodyClaimTier::ReleaseGrade.label(), "ReleaseGrade");
    assert_eq!(BodyClaimTier::Unsupported.label(), "Unsupported");
}

#[test]
fn release_grade_constructor_sets_fields() {
    let claim = BodyClaim::release_grade(
        CelestialBody::Pluto,
        AccuracyClass::High,
        ClaimEvidence::ArtifactValidated,
    );
    assert_eq!(claim.body, CelestialBody::Pluto);
    assert_eq!(claim.tier, BodyClaimTier::ReleaseGrade);
    assert_eq!(claim.accuracy, AccuracyClass::High);
    assert_eq!(claim.evidence, ClaimEvidence::ArtifactValidated);
}

#[test]
fn from_celestial_body_defaults_to_constrained() {
    let claim: BodyClaim = CelestialBody::Sun.into();
    assert_eq!(claim.tier, BodyClaimTier::Constrained);
    assert_eq!(claim.accuracy, AccuracyClass::Unknown);
    assert_eq!(claim.evidence, ClaimEvidence::None);
}

#[test]
fn summary_line_mentions_body_tier_and_evidence() {
    let claim = BodyClaim::release_grade(
        CelestialBody::Ceres,
        AccuracyClass::High,
        ClaimEvidence::CorpusValidated { source: "sb441-n16".to_string() },
    );
    let line = claim.summary_line();
    assert!(line.contains("Ceres"));
    assert!(line.contains("ReleaseGrade"));
    assert!(line.contains("sb441-n16"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-backend claims_tests 2>&1 | head -30`
Expected: FAIL — `module 'claims' not found` / unresolved imports.

- [ ] **Step 3: Write the implementation**

Create `crates/pleiades-backend/src/claims.rs`:

```rust
use crate::identity::AccuracyClass;
use core::fmt;
use pleiades_types::CelestialBody;

/// The release-claim status of a single body for a single backend.
///
/// This is orthogonal to [`AccuracyClass`]: accuracy describes the numeric
/// band, the tier describes what the project promises at release.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BodyClaimTier {
    /// Production claim: validated against the reference corpus to the SP3 ceiling.
    ReleaseGrade,
    /// Source-backed but below the release ceiling, or corpus/kernel dependent.
    Constrained,
    /// Algorithmic approximation; no release claim.
    Approximate,
    /// Explicitly not supported; preflight rejects requests for it.
    Unsupported,
}

impl BodyClaimTier {
    /// Returns a stable human-readable label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReleaseGrade => "ReleaseGrade",
            Self::Constrained => "Constrained",
            Self::Approximate => "Approximate",
            Self::Unsupported => "Unsupported",
        }
    }

    /// Returns the merge rank: stronger tiers win on backend body collisions.
    pub const fn rank(self) -> u8 {
        match self {
            Self::ReleaseGrade => 3,
            Self::Constrained => 2,
            Self::Approximate => 1,
            Self::Unsupported => 0,
        }
    }
}

impl fmt::Display for BodyClaimTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// The evidence backing a body claim.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ClaimEvidence {
    /// Validated inside the packaged artifact build against the reference corpus.
    ArtifactValidated,
    /// Validated against a named reference corpus (e.g. `de440`, `sb441-n16`).
    CorpusValidated {
        /// The reference source identifier.
        source: String,
    },
    /// Backed only by an algorithmic model (VSOP87, compact ELP).
    AlgorithmicModel,
    /// No release-relevant evidence.
    None,
}

impl ClaimEvidence {
    /// Returns a compact human-readable label.
    pub fn label(&self) -> String {
        match self {
            Self::ArtifactValidated => "artifact-validated".to_string(),
            Self::CorpusValidated { source } => format!("corpus-validated:{source}"),
            Self::AlgorithmicModel => "algorithmic-model".to_string(),
            Self::None => "none".to_string(),
        }
    }
}

impl fmt::Display for ClaimEvidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

/// A single backend's claim about a single body.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BodyClaim {
    /// The body this claim describes.
    pub body: CelestialBody,
    /// The release-claim tier.
    pub tier: BodyClaimTier,
    /// The per-body accuracy class.
    pub accuracy: AccuracyClass,
    /// The evidence backing the claim.
    pub evidence: ClaimEvidence,
}

impl BodyClaim {
    /// Creates a claim from explicit parts.
    pub fn new(
        body: CelestialBody,
        tier: BodyClaimTier,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self { body, tier, accuracy, evidence }
    }

    /// Creates a `ReleaseGrade` claim.
    pub fn release_grade(
        body: CelestialBody,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self::new(body, BodyClaimTier::ReleaseGrade, accuracy, evidence)
    }

    /// Creates a `Constrained` claim.
    pub fn constrained(
        body: CelestialBody,
        accuracy: AccuracyClass,
        evidence: ClaimEvidence,
    ) -> Self {
        Self::new(body, BodyClaimTier::Constrained, accuracy, evidence)
    }

    /// Creates an `Approximate` claim.
    pub fn approximate(body: CelestialBody) -> Self {
        Self::new(
            body,
            BodyClaimTier::Approximate,
            AccuracyClass::Approximate,
            ClaimEvidence::AlgorithmicModel,
        )
    }

    /// Creates an `Unsupported` claim (listed for explicitness; preflight rejects it).
    pub fn unsupported(body: CelestialBody) -> Self {
        Self::new(
            body,
            BodyClaimTier::Unsupported,
            AccuracyClass::Unknown,
            ClaimEvidence::None,
        )
    }

    /// Returns a compact one-line rendering.
    pub fn summary_line(&self) -> String {
        format!(
            "{} [{}; accuracy={}; evidence={}]",
            self.body, self.tier, self.accuracy, self.evidence
        )
    }
}

impl From<CelestialBody> for BodyClaim {
    fn from(body: CelestialBody) -> Self {
        Self::constrained(body, AccuracyClass::Unknown, ClaimEvidence::None)
    }
}

#[cfg(test)]
#[path = "claims_tests.rs"]
mod tests;
```

Note: the test calls `BodyClaim::release_grade(..)` and `BodyClaim::constrained` via `From`; the explicit-arg `constrained` keeps fixtures terse in Task 2.

- [ ] **Step 4: Wire module + re-exports**

In `crates/pleiades-backend/src/lib.rs`, add `mod claims;` alphabetically among the `mod` lines (after `mod capabilities;`). Add after the `pub use identity::{...}` line:

```rust
pub use claims::{BodyClaim, BodyClaimTier, ClaimEvidence};
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p pleiades-backend claims_tests 2>&1 | tail -15`
Expected: PASS (5 tests).

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-backend --all-targets --all-features -- -D warnings
git add crates/pleiades-backend/src/claims.rs crates/pleiades-backend/src/claims_tests.rs crates/pleiades-backend/src/lib.rs
git commit -m "feat(backend): add BodyClaim/BodyClaimTier/ClaimEvidence types"
```

---

### Task 2: Replace `body_coverage` with `body_claims` + derived accessors

This is the compile-atomic migration: the field change forces every producer and consumer to move in one commit. `From<CelestialBody>` (Task 1) keeps fixtures terse and defaults everything to `Constrained` — **never over-claims** during the window before per-backend tiers land (Tasks 3–6).

**Files:**
- Modify: `crates/pleiades-backend/src/metadata.rs` (field at line 136; `summary_line` ~152; `validate_request` ~213; `validate` ~322)
- Modify: `crates/pleiades-backend/src/traits.rs` (lines 90, 205, 220 — `combine_bodies`)
- Modify: `crates/pleiades-backend/src/lib.rs:36` (doc example)
- Modify (producers, body_coverage → body_claims): `crates/pleiades-data/src/backend.rs:104`, `crates/pleiades-vsop87/src/backend.rs:384`, `crates/pleiades-elp/src/backend.rs:199`, `crates/pleiades-jpl/src/backend.rs:208`, `crates/pleiades-jpl/src/spk/backend.rs:162`
- Modify (consumers): `crates/pleiades-validate/src/render/summary/backend.rs:76,82`, `crates/pleiades-validate/src/render/summary/writers.rs:80,81`, `crates/pleiades-data/src/tests/codec.rs:29,54`
- Modify (fixtures): `crates/pleiades-backend/src/validation_tests.rs` (16 sites), `crates/pleiades-backend/src/metadata_tests.rs` (3), `crates/pleiades-backend/src/request_tests.rs` (4), `crates/pleiades-backend/src/test_support.rs:17`, `crates/pleiades-elp/src/tests.rs:1931`

**Interfaces:**
- Produces: `BackendMetadata.body_claims: Vec<BodyClaim>`; methods `supported_bodies(&self) -> Vec<CelestialBody>` (tier ≠ `Unsupported`), `claim_for(&self, &CelestialBody) -> Option<&BodyClaim>`, `release_grade_bodies(&self) -> Vec<CelestialBody>`, `claims_by_tier(&self, BodyClaimTier) -> Vec<&BodyClaim>`; free fn `merge_body_claims(a: &[BodyClaim], b: &[BodyClaim]) -> Vec<BodyClaim>` (keeps higher `rank`).
- Consumes: `BodyClaim`, `BodyClaimTier` from Task 1.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-backend/src/metadata_tests.rs`:

```rust
#[test]
fn supported_bodies_excludes_unsupported_tier() {
    use crate::claims::{BodyClaim, BodyClaimTier};
    let mut meta = sample_metadata(); // existing helper building a valid BackendMetadata
    meta.body_claims = vec![
        BodyClaim::from(CelestialBody::Moon),
        BodyClaim::unsupported(CelestialBody::TrueApogee),
    ];
    let bodies = meta.supported_bodies();
    assert!(bodies.contains(&CelestialBody::Moon));
    assert!(!bodies.contains(&CelestialBody::TrueApogee));
    assert_eq!(meta.claim_for(&CelestialBody::TrueApogee).map(|c| c.tier), Some(BodyClaimTier::Unsupported));
}

#[test]
fn validate_rejects_duplicate_body_claims() {
    let mut meta = sample_metadata();
    meta.body_claims = vec![
        CelestialBody::Sun.into(),
        CelestialBody::Sun.into(),
    ];
    assert!(meta.validate().is_err());
}

#[test]
fn merge_body_claims_keeps_stronger_tier() {
    use crate::claims::{BodyClaim, BodyClaimTier, ClaimEvidence};
    use crate::metadata::merge_body_claims;
    use crate::AccuracyClass;
    let a = vec![BodyClaim::approximate(CelestialBody::Pluto)];
    let b = vec![BodyClaim::release_grade(CelestialBody::Pluto, AccuracyClass::High, ClaimEvidence::ArtifactValidated)];
    let merged = merge_body_claims(&a, &b);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].tier, BodyClaimTier::ReleaseGrade);
}
```

If `sample_metadata()` does not exist in `metadata_tests.rs`, add a small helper that constructs a valid `BackendMetadata` with `body_claims: vec![CelestialBody::Sun.into()]` and otherwise mirrors the existing fixtures in that file.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-backend 2>&1 | head -30`
Expected: FAIL to **compile** — `no field body_claims`, `no method supported_bodies`, `merge_body_claims` unresolved. (Compile failure is the expected "red" here.)

- [ ] **Step 3: Change the field and add accessors in `metadata.rs`**

Replace the field (line 136):

```rust
    /// Supported body coverage and per-body release claims.
    pub body_claims: Vec<BodyClaim>,
```

Add `use crate::claims::{BodyClaim, BodyClaimTier};` to the imports. Add accessors inside the first `impl BackendMetadata` block:

```rust
    /// Returns the bodies the backend serves (every tier except `Unsupported`).
    pub fn supported_bodies(&self) -> Vec<CelestialBody> {
        self.body_claims
            .iter()
            .filter(|c| c.tier != BodyClaimTier::Unsupported)
            .map(|c| c.body.clone())
            .collect()
    }

    /// Returns the claim for a body, if declared.
    pub fn claim_for(&self, body: &CelestialBody) -> Option<&BodyClaim> {
        self.body_claims.iter().find(|c| &c.body == body)
    }

    /// Returns the bodies claimed `ReleaseGrade`.
    pub fn release_grade_bodies(&self) -> Vec<CelestialBody> {
        self.body_claims
            .iter()
            .filter(|c| c.tier == BodyClaimTier::ReleaseGrade)
            .map(|c| c.body.clone())
            .collect()
    }

    /// Returns claims at a given tier.
    pub fn claims_by_tier(&self, tier: BodyClaimTier) -> Vec<&BodyClaim> {
        self.body_claims.iter().filter(|c| c.tier == tier).collect()
    }
```

Add the free merge function (module level in `metadata.rs`):

```rust
/// Merges two claim lists, keeping the stronger-ranked tier on body collisions.
pub fn merge_body_claims(a: &[BodyClaim], b: &[BodyClaim]) -> Vec<BodyClaim> {
    let mut out: Vec<BodyClaim> = a.to_vec();
    for claim in b {
        match out.iter_mut().find(|c| c.body == claim.body) {
            Some(existing) => {
                if claim.tier.rank() > existing.tier.rank() {
                    *existing = claim.clone();
                }
            }
            None => out.push(claim.clone()),
        }
    }
    out
}
```

- [ ] **Step 4: Update `summary_line`, `validate_request`, `validate`**

In `summary_line` (~line 163) replace `format_display_list(&self.body_coverage)` with a claims rendering:

```rust
            self.body_claims
                .iter()
                .map(BodyClaim::summary_line)
                .collect::<Vec<_>>()
                .join(", "),
```

In `validate_request` (~line 213) replace the coverage check:

```rust
        if !self.supported_bodies().contains(&req.body) {
```

In `validate` (~line 322) replace `validate_non_empty_unique("body coverage", &self.body_coverage)?;` with an explicit non-empty + unique-by-body check:

```rust
        if self.body_claims.is_empty() {
            return Err(BackendMetadataValidationError::EmptyField { field: "body claims" });
        }
        let mut seen: Vec<CelestialBody> = Vec::new();
        for claim in &self.body_claims {
            if seen.contains(&claim.body) {
                return Err(BackendMetadataValidationError::DuplicateEntry {
                    field: "body claims",
                    value: claim.body.to_string(),
                });
            }
            seen.push(claim.body.clone());
        }
```

- [ ] **Step 5: Update `traits.rs` composite/routing merge**

At line 90 (`CompositeBackend`) replace:

```rust
            body_claims: crate::metadata::merge_body_claims(&primary.body_claims, &secondary.body_claims),
```

At line 205 replace `let mut body_coverage = metadatas[0].body_coverage.clone();` with `let mut body_claims = metadatas[0].body_claims.clone();`, and at line 220 replace the loop body with `body_claims = crate::metadata::merge_body_claims(&body_claims, &metadata.body_claims);`. Update the struct construction in that function to use `body_claims`.

- [ ] **Step 6: Update producers (all five backends)**

Each currently sets `body_coverage: <list>`. For now, convert the list to claims with `.into()` defaults (correct tiers land in Tasks 3–6). Pattern, applied at each producer line:

```rust
        body_claims: <existing list expression>.iter().cloned().map(BodyClaim::from).collect(),
```

Concretely:
- `pleiades-data/src/backend.rs:104` → `body_claims: bodies.iter().cloned().map(BodyClaim::from).collect(),`
- `pleiades-vsop87/src/backend.rs:384` → `body_claims: Self::supported_bodies().iter().cloned().map(BodyClaim::from).collect(),`
- `pleiades-elp/src/backend.rs:199` → `body_claims: lunar_theory_supported_bodies().iter().cloned().map(BodyClaim::from).collect(),`
- `pleiades-jpl/src/backend.rs:208` → `body_claims: bodies.iter().cloned().map(BodyClaim::from).collect(),`
- `pleiades-jpl/src/spk/backend.rs:162` → `body_claims: self.covered_bodies().iter().cloned().map(BodyClaim::from).collect(),`

Add `use pleiades_backend::BodyClaim;` (or crate-local path) to each file's imports.

- [ ] **Step 7: Update consumers**

- `pleiades-validate/src/render/summary/backend.rs:76,82` and `writers.rs:80,81`: replace `&entry.metadata.body_coverage` / `&backend.body_coverage` with `&backend.supported_bodies()` (bind to a local `let bodies = backend.supported_bodies();` then pass `&bodies` to `format_bodies` / `selected_asteroid_coverage`).
- `pleiades-data/src/tests/codec.rs:29,54`: replace `metadata.body_coverage.contains(&CelestialBody::Sun)` with `metadata.supported_bodies().contains(&CelestialBody::Sun)`.

- [ ] **Step 8: Update fixtures**

In every fixture site listed in **Files**, replace `body_coverage: vec![CelestialBody::X, ...]` with `body_claims: vec![CelestialBody::X.into(), ...]`. For the field-assignment sites (`metadata_tests.rs:99`, `elp/tests.rs:1931` assertion) adjust to `body_claims` / `supported_bodies()` respectively:
- `elp/tests.rs:1931`: `assert_eq!(backend.metadata().supported_bodies(), lunar_theory_supported_bodies());`

Use a search to confirm none remain: `grep -rn "body_coverage" crates/ | grep -v residual_body_coverage | grep -v packaged_artifact_profile_summary_with_body_coverage | grep -v packaged_body_coverage_summary`. (The compression/artifact `residual_body_coverage_summary` and the packaged `*_body_coverage*` report fns are unrelated names — leave them.)

- [ ] **Step 9: Build, test, lint**

Run: `cargo test --workspace 2>&1 | tail -20`
Expected: PASS (workspace compiles; new metadata_tests pass; existing tests still pass — claims default to `Constrained`, posture layer untouched).
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 10: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "refactor(backend): replace body_coverage with body_claims + derived accessors"
```

---

### Task 3: packaged-data declares release-grade claims

**Files:**
- Modify: `crates/pleiades-data/src/lib.rs` (near `packaged_bodies` ~line 187)
- Modify: `crates/pleiades-data/src/backend.rs:104`
- Test: `crates/pleiades-data/src/tests/codec.rs` (or a new `claims` test module)

**Interfaces:**
- Produces: `pub fn packaged_body_claims() -> Vec<BodyClaim>` — all 11 packaged bodies (`PACKAGED_BASE_BODIES` + `asteroid:433-Eros`) as `ReleaseGrade`, `AccuracyClass::High`, `ClaimEvidence::ArtifactValidated`.

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-data/src/tests/codec.rs`:

```rust
#[test]
fn packaged_metadata_claims_eleven_bodies_release_grade() {
    use pleiades_backend::{BodyClaimTier, CelestialBody, CustomBodyId, EphemerisBackend};
    let backend = crate::PackagedDataBackend::default();
    let meta = backend.metadata();
    assert_eq!(meta.release_grade_bodies().len(), 11);
    for body in [CelestialBody::Pluto, CelestialBody::Moon,
                 CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))] {
        assert_eq!(meta.claim_for(&body).map(|c| c.tier), Some(BodyClaimTier::ReleaseGrade));
    }
}
```

(If `PackagedDataBackend::default()` is not the constructor, use the same constructor the existing codec tests use.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-data packaged_metadata_claims 2>&1 | tail -15`
Expected: FAIL — `release_grade_bodies().len()` is 0 (claims default to `Constrained`).

- [ ] **Step 3: Add `packaged_body_claims()` in `lib.rs`**

```rust
/// Returns the per-body release claims for the packaged artifact: every shipped
/// body is release-grade, validated inside the artifact build against the corpus.
pub fn packaged_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    packaged_bodies()
        .iter()
        .cloned()
        .map(|body| {
            BodyClaim::release_grade(body, AccuracyClass::High, ClaimEvidence::ArtifactValidated)
        })
        .collect()
}
```

- [ ] **Step 4: Use it in `metadata()`**

In `backend.rs`, the metadata builds `bodies` from `artifact.bodies`. Replace the `body_claims` line so claims are restricted to the bodies actually present in the artifact but tiered from the declaration:

```rust
        body_claims: {
            let declared = crate::packaged_body_claims();
            bodies
                .iter()
                .map(|body| {
                    declared
                        .iter()
                        .find(|c| &c.body == body)
                        .cloned()
                        .unwrap_or_else(|| pleiades_backend::BodyClaim::from(body.clone()))
                })
                .collect()
        },
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p pleiades-data 2>&1 | tail -15`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(data): packaged backend declares 11 release-grade body claims"
```

---

### Task 4: VSOP87 declares constrained majors + approximate Pluto

**Files:**
- Modify: `crates/pleiades-vsop87/src/backend.rs` (near `supported_bodies` ~line 38 and `metadata` ~384)
- Test: `crates/pleiades-vsop87/src/backend.rs` test module (or existing tests file)

**Interfaces:**
- Produces: `pub(crate) fn vsop87_body_claims() -> Vec<BodyClaim>` — Sun + Mercury–Neptune `Constrained`/`AccuracyClass::Moderate`/`AlgorithmicModel`; Pluto `Approximate`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn vsop87_claims_majors_constrained_pluto_approximate() {
    use pleiades_backend::{BodyClaimTier, CelestialBody, EphemerisBackend};
    let meta = Vsop87Backend::default().metadata();
    assert_eq!(meta.claim_for(&CelestialBody::Mars).map(|c| c.tier), Some(BodyClaimTier::Constrained));
    assert_eq!(meta.claim_for(&CelestialBody::Pluto).map(|c| c.tier), Some(BodyClaimTier::Approximate));
    assert!(meta.release_grade_bodies().is_empty());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-vsop87 vsop87_claims 2>&1 | tail -15`
Expected: FAIL — Mars defaults `Constrained` (ok) but Pluto is `Constrained`, not `Approximate`.

- [ ] **Step 3: Add `vsop87_body_claims()`**

```rust
pub(crate) fn vsop87_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, CelestialBody, ClaimEvidence};
    supported_bodies()
        .iter()
        .cloned()
        .map(|body| match body {
            CelestialBody::Pluto => BodyClaim::approximate(body),
            other => BodyClaim::constrained(other, AccuracyClass::Moderate, ClaimEvidence::AlgorithmicModel),
        })
        .collect()
}
```

- [ ] **Step 4: Use in `metadata()`** — replace the `body_claims` line (from Task 2) with `body_claims: vsop87_body_claims(),`.

- [ ] **Step 5: Run tests**

Run: `cargo test -p pleiades-vsop87 2>&1 | tail -15`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(vsop87): declare constrained majors and approximate Pluto claims"
```

---

### Task 5: ELP declares constrained lunar + unsupported true apogee/perigee

**Files:**
- Modify: `crates/pleiades-elp/src/backend.rs` (~line 199) and `crates/pleiades-elp/src/catalog.rs` (near `lunar_theory_supported_bodies` ~602)
- Test: `crates/pleiades-elp/src/tests.rs`

**Interfaces:**
- Produces: `pub fn elp_body_claims() -> Vec<BodyClaim>` — Moon + Mean/True Node + Mean Apogee/Perigee `Constrained`/`Moderate`/`AlgorithmicModel`; `TrueApogee`, `TruePerigee` listed `Unsupported`.

- [ ] **Step 1: Write the failing test** in `tests.rs`:

```rust
#[test]
fn elp_claims_lunar_constrained_true_apsides_unsupported() {
    use pleiades_backend::{BodyClaimTier, CelestialBody, EphemerisBackend};
    let meta = ElpBackend::default().metadata();
    assert_eq!(meta.claim_for(&CelestialBody::Moon).map(|c| c.tier), Some(BodyClaimTier::Constrained));
    assert_eq!(meta.claim_for(&CelestialBody::TrueApogee).map(|c| c.tier), Some(BodyClaimTier::Unsupported));
    assert!(!meta.supported_bodies().contains(&CelestialBody::TrueApogee));
    assert!(meta.release_grade_bodies().is_empty());
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-elp elp_claims 2>&1 | tail -15`
Expected: FAIL — `TrueApogee` not present at all (so `claim_for` is `None`).

- [ ] **Step 3: Add `elp_body_claims()`** in `backend.rs`:

```rust
pub fn elp_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, CelestialBody, ClaimEvidence};
    let mut claims: Vec<BodyClaim> = lunar_theory_supported_bodies()
        .iter()
        .cloned()
        .map(|body| BodyClaim::constrained(body, AccuracyClass::Moderate, ClaimEvidence::AlgorithmicModel))
        .collect();
    claims.push(BodyClaim::unsupported(CelestialBody::TrueApogee));
    claims.push(BodyClaim::unsupported(CelestialBody::TruePerigee));
    claims
}
```

- [ ] **Step 4: Use in `metadata()`** — replace the `body_claims` line with `body_claims: elp_body_claims(),`.

- [ ] **Step 5: Run tests**

Run: `cargo test -p pleiades-elp 2>&1 | tail -15`
Expected: PASS. (The Task 2 assertion `supported_bodies() == lunar_theory_supported_bodies()` still holds because `Unsupported` entries are excluded from `supported_bodies()`.)

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(elp): declare constrained lunar claims and unsupported true apsides"
```

---

### Task 6: JPL backends declare claims (sb441-n16 Tier-A release-grade)

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (use `tier_a_bodies()` / `tier_b_bodies()` ~112)
- Modify: `crates/pleiades-jpl/src/spk/backend.rs:162`
- Modify: `crates/pleiades-jpl/src/backend.rs:208` (jpl-snapshot — all `Constrained`)
- Test: `crates/pleiades-jpl/src/spk/backend.rs` test module

**Interfaces:**
- Produces: `pub fn spk_body_claims(covered: &[CelestialBody]) -> Vec<BodyClaim>` in `asteroid_roster.rs` (or `spk/backend.rs`) — for each covered body: if in `tier_a_bodies()` → `ReleaseGrade`/`High`/`CorpusValidated{"sb441-n16"}`; if in `tier_b_bodies()` → `Constrained`/`Moderate`/`CorpusValidated{"horizons"}`; planets it serves → `Constrained`/`High`/`CorpusValidated{"de440"}`.

- [ ] **Step 1: Write the failing test** in `spk/backend.rs`:

```rust
#[test]
fn spk_claims_tier_a_release_grade_tier_b_constrained() {
    use pleiades_backend::{BodyClaimTier, CelestialBody};
    use crate::spk::asteroid_roster::spk_body_claims;
    let covered = vec![
        CelestialBody::Ceres,                         // Tier A
        CelestialBody::Vesta,                         // Tier A
        ast_body("2060-Chiron"),                      // Tier B (test helper for Custom asteroid id)
        CelestialBody::Mars,                          // planet
    ];
    let claims = spk_body_claims(&covered);
    let tier = |b: &CelestialBody| claims.iter().find(|c| &c.body == b).map(|c| c.tier);
    assert_eq!(tier(&CelestialBody::Ceres), Some(BodyClaimTier::ReleaseGrade));
    assert_eq!(tier(&CelestialBody::Vesta), Some(BodyClaimTier::ReleaseGrade));
    assert_eq!(tier(&ast_body("2060-Chiron")), Some(BodyClaimTier::Constrained));
    assert_eq!(tier(&CelestialBody::Mars), Some(BodyClaimTier::Constrained));
}
```

Add a `fn ast_body(d: &str) -> CelestialBody { CelestialBody::Custom(CustomBodyId::new("asteroid", d)) }` test helper if none exists.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-jpl spk_claims 2>&1 | tail -15`
Expected: FAIL — `spk_body_claims` unresolved.

- [ ] **Step 3: Add `spk_body_claims()`** in `asteroid_roster.rs`:

```rust
/// Builds per-body claims for the SPK backend over the bodies it actually covers.
pub fn spk_body_claims(covered: &[CelestialBody]) -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let tier_a = tier_a_bodies();
    let tier_b = tier_b_bodies();
    covered
        .iter()
        .cloned()
        .map(|body| {
            if tier_a.contains(&body) {
                BodyClaim::release_grade(
                    body,
                    AccuracyClass::High,
                    ClaimEvidence::CorpusValidated { source: "sb441-n16".to_string() },
                )
            } else if tier_b.contains(&body) {
                BodyClaim::constrained(
                    body,
                    AccuracyClass::Moderate,
                    ClaimEvidence::CorpusValidated { source: "horizons".to_string() },
                )
            } else {
                BodyClaim::constrained(
                    body,
                    AccuracyClass::High,
                    ClaimEvidence::CorpusValidated { source: "de440".to_string() },
                )
            }
        })
        .collect()
}
```

- [ ] **Step 4: Use in SPK `metadata()`** (`spk/backend.rs:162`) — replace with:

```rust
        body_claims: crate::spk::asteroid_roster::spk_body_claims(&self.covered_bodies()),
```

For jpl-snapshot (`backend.rs:208`) keep the `.into()` default (all `Constrained`) — it is a validation fixture, excluded from the release posture, so leave Task 2's mapping; no change needed beyond what Task 2 did.

- [ ] **Step 5: Run tests**

Run: `cargo test -p pleiades-jpl 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(jpl): SPK declares sb441-n16 Tier-A release-grade asteroid claims"
```

---

### Task 7: `ReleasePosture` type + pure derivation (in `pleiades-backend`)

**Files:**
- Create: `crates/pleiades-backend/src/release_posture.rs`
- Create: `crates/pleiades-backend/src/release_posture_tests.rs`
- Modify: `crates/pleiades-backend/src/lib.rs` (`mod release_posture;` + re-export)

**Interfaces:**
- Produces: `ReleasePosture { entries: Vec<(BackendId, BodyClaim)> }` with `from_backends(metas: &[&BackendMetadata]) -> ReleasePosture`, `release_grade(&self) -> Vec<(BackendId, CelestialBody)>`, `claims_for_tier(&self, BodyClaimTier) -> Vec<(BackendId, BodyClaim)>`, and `summary_line(&self) -> String` (derived prose grouping by tier; deterministic ordering by backend id then body).

- [ ] **Step 1: Write the failing test** (`release_posture_tests.rs`):

```rust
use super::release_posture::ReleasePosture;
use crate::claims::BodyClaimTier;
use crate::{AccuracyClass, BackendId, BackendMetadata, BodyClaim, CelestialBody, ClaimEvidence};

fn meta_with(id: &str, claims: Vec<BodyClaim>) -> BackendMetadata { /* build minimal valid metadata; reuse a shared test helper or inline a constructor mirroring metadata_tests fixtures, setting body_claims = claims */ }

#[test]
fn posture_collects_release_grade_per_backend() {
    let a = meta_with("packaged-data", vec![BodyClaim::release_grade(CelestialBody::Pluto, AccuracyClass::High, ClaimEvidence::ArtifactValidated)]);
    let b = meta_with("pleiades-vsop87", vec![BodyClaim::approximate(CelestialBody::Pluto)]);
    let posture = ReleasePosture::from_backends(&[&a, &b]);
    let rg = posture.release_grade();
    assert!(rg.iter().any(|(id, body)| id.as_str() == "packaged-data" && body == &CelestialBody::Pluto));
    assert!(!rg.iter().any(|(id, _)| id.as_str() == "pleiades-vsop87"));
}

#[test]
fn summary_line_is_deterministic() {
    let a = meta_with("packaged-data", vec![BodyClaim::release_grade(CelestialBody::Moon, AccuracyClass::High, ClaimEvidence::ArtifactValidated)]);
    let p1 = ReleasePosture::from_backends(&[&a]);
    let p2 = ReleasePosture::from_backends(&[&a]);
    assert_eq!(p1.summary_line(), p2.summary_line());
    assert!(p1.summary_line().contains("Moon"));
    assert!(p1.summary_line().contains("ReleaseGrade"));
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-backend release_posture 2>&1 | head -20` → FAIL (module missing).

- [ ] **Step 3: Implement `release_posture.rs`**

```rust
use crate::claims::{BodyClaim, BodyClaimTier};
use crate::identity::BackendId;
use crate::metadata::BackendMetadata;
use pleiades_types::CelestialBody;

/// A derived, cross-backend view of body claims for release reporting.
#[derive(Clone, Debug, PartialEq)]
pub struct ReleasePosture {
    /// Flat (backend, claim) entries, ordered deterministically.
    pub entries: Vec<(BackendId, BodyClaim)>,
}

impl ReleasePosture {
    /// Aggregates claims across the given backend metadata (the caller chooses the set).
    pub fn from_backends(metas: &[&BackendMetadata]) -> Self {
        let mut entries: Vec<(BackendId, BodyClaim)> = Vec::new();
        for meta in metas {
            for claim in &meta.body_claims {
                entries.push((meta.id.clone(), claim.clone()));
            }
        }
        entries.sort_by(|a, b| {
            a.0.as_str()
                .cmp(b.0.as_str())
                .then_with(|| a.1.body.to_string().cmp(&b.1.body.to_string()))
        });
        Self { entries }
    }

    /// Returns the `(backend, body)` pairs claimed `ReleaseGrade`.
    pub fn release_grade(&self) -> Vec<(BackendId, CelestialBody)> {
        self.entries
            .iter()
            .filter(|(_, c)| c.tier == BodyClaimTier::ReleaseGrade)
            .map(|(id, c)| (id.clone(), c.body.clone()))
            .collect()
    }

    /// Returns the entries at a given tier.
    pub fn claims_for_tier(&self, tier: BodyClaimTier) -> Vec<(BackendId, BodyClaim)> {
        self.entries
            .iter()
            .filter(|(_, c)| c.tier == tier)
            .cloned()
            .collect()
    }

    /// Renders a deterministic one-line summary grouped by tier.
    pub fn summary_line(&self) -> String {
        let render = |tier: BodyClaimTier| -> String {
            self.entries
                .iter()
                .filter(|(_, c)| c.tier == tier)
                .map(|(id, c)| format!("{}@{}", c.body, id))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "ReleaseGrade: [{}]; Constrained: [{}]; Approximate: [{}]; Unsupported: [{}]",
            render(BodyClaimTier::ReleaseGrade),
            render(BodyClaimTier::Constrained),
            render(BodyClaimTier::Approximate),
            render(BodyClaimTier::Unsupported),
        )
    }
}

#[cfg(test)]
#[path = "release_posture_tests.rs"]
mod tests;
```

For the `meta_with` test helper, factor a `pub(crate) fn` in `test_support.rs` that builds a valid `BackendMetadata` given an id + `Vec<BodyClaim>` (mirror existing fixture fields), and reuse it here and in Task 8.

- [ ] **Step 4: Wire module + re-export** in `lib.rs`: `mod release_posture;` and `pub use release_posture::ReleasePosture;`.

- [ ] **Step 5: Run tests** — `cargo test -p pleiades-backend release_posture 2>&1 | tail -15` → PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(backend): add ReleasePosture derived from backend metadata"
```

---

### Task 8: Derive the release posture from the canonical backends; retire the global string layer

This is the cutover. `pleiades-validate` instantiates the four release backends, derives `ReleasePosture`, and feeds the renderers. The hardcoded `*_bodies()` lists, the hand-written summary, and the `.contains(PHRASE)` validation in `release_body_claims.rs` are deleted; `validate_release_body_claims_posture` is replaced by a structural check. Summary text changes (intended) → snapshot fixtures update here.

**Files:**
- Create: `crates/pleiades-validate/src/claims/mod.rs`, `crates/pleiades-validate/src/claims/posture.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (`mod claims;` + exports)
- Modify: `crates/pleiades-validate/src/render/summary/release.rs:189-251` (renderers consume the derived posture)
- Modify: `crates/pleiades-backend/src/release_body_claims.rs` (delete hardcoded lists + phrase consts; reimplement `validate_release_body_claims_posture` structurally, or move it to `claims/posture.rs`)
- Modify: `crates/pleiades-backend/src/policy/mod.rs` (remove `CURRENT_RELEASE_BODY_CLAIMS_SUMMARY_TEXT`, `CURRENT_PLUTO_FALLBACK_POLICY_SUMMARY_TEXT` if now derived) and `policy/current.rs` (`current_release_body_claims_summary` etc.)
- Modify: `crates/pleiades-backend/src/lib.rs` (drop deleted re-exports)
- Modify: any snapshot/golden test asserting the old summary text (search: `grep -rn "release-grade major-body claims" crates/` and `grep -rn "source-backed validation bodies" crates/`)

**Interfaces:**
- Produces (in `pleiades-validate`): `pub fn canonical_release_metadata() -> Vec<BackendMetadata>` (packaged-data, jpl-spk, elp, vsop87); `pub fn derived_release_posture() -> ReleasePosture`; `pub fn render_release_body_claims_summary_text() -> String` now built from `derived_release_posture().summary_line()`.

- [ ] **Step 1: Write the failing test** (`crates/pleiades-validate/src/claims/posture.rs` test module):

```rust
#[test]
fn derived_posture_promotes_pluto_via_packaged_data() {
    use pleiades_backend::CelestialBody;
    let posture = super::derived_release_posture();
    let rg = posture.release_grade();
    assert!(rg.iter().any(|(id, b)| id.as_str() == "packaged-data" && b == &CelestialBody::Pluto));
    // Pluto via vsop87 must NOT be release-grade.
    assert!(!rg.iter().any(|(id, b)| id.as_str() == "pleiades-vsop87" && b == &CelestialBody::Pluto));
    // The 7-body sb441-n16 Tier-A set is release-grade via jpl-spk (when kernel present).
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-validate derived_posture 2>&1 | head -20` → FAIL (unresolved).

- [ ] **Step 3: Implement `canonical_release_metadata` + `derived_release_posture`** in `claims/posture.rs`:

```rust
use pleiades_backend::{BackendMetadata, EphemerisBackend, ReleasePosture};

/// Metadata for the four release-facing backends. `jpl-snapshot` is intentionally
/// excluded — it is a validation fixture, not a release-claimed backend.
pub fn canonical_release_metadata() -> Vec<BackendMetadata> {
    vec![
        pleiades_data::PackagedDataBackend::default().metadata(),
        // SPK backend constructed from the checked-in kernel/corpus loader used elsewhere
        // in pleiades-validate; see corpus/comparison setup for the exact constructor.
        crate::claims::spk_release_backend().metadata(),
        pleiades_elp::ElpBackend::default().metadata(),
        pleiades_vsop87::Vsop87Backend::default().metadata(),
    ]
}

/// The derived, cross-backend release posture.
pub fn derived_release_posture() -> ReleasePosture {
    let metas = canonical_release_metadata();
    let refs: Vec<&BackendMetadata> = metas.iter().collect();
    ReleasePosture::from_backends(&refs)
}
```

Add a small `spk_release_backend()` helper in `claims/mod.rs` that builds the `SpkBackend` the same way the comparison/corpus code does (reuse the existing constructor pattern from `pleiades-validate/src/corpus` / `comparison`). If the SPK kernel is absent in a given environment, its `covered_bodies()` is empty and its claims simply do not appear — the posture stays well-defined; the audit (Task 11) is where kernel-absence for a `ReleaseGrade`-intended body is reported.

- [ ] **Step 4: Point renderers at the derived posture**

In `render/summary/release.rs`, change `render_release_body_claims_summary_text()` to:

```rust
pub(crate) fn render_release_body_claims_summary_text() -> String {
    format!(
        "Release-grade body claims summary\nRelease-grade body claims: {}\n",
        crate::claims::derived_release_posture().summary_line()
    )
}
```

Update `render_pluto_fallback_summary_text_from_report` to obtain the release-claims line and pluto-fallback line from the derived posture (Pluto appears `ReleaseGrade@packaged-data` and `Approximate@pleiades-vsop87`), and replace its call to `validate_release_body_claims_posture(release_line, policy_line)` with the new structural validator (Step 5).

- [ ] **Step 5: Replace the brittle validator**

Delete from `crates/pleiades-backend/src/release_body_claims.rs`: `release_body_claims_lunar_validation_bodies`, `release_body_claims_major_bodies`, `release_body_claims_selected_asteroids`, `release_body_claims_summary_text`, and the four `const *_PHRASE` strings inside `validate_release_body_claims_posture`. Reimplement the posture validation as a structural check over `ReleasePosture` (move it to `pleiades-validate/src/claims/posture.rs` as `validate_release_posture(posture: &ReleasePosture) -> Result<(), ClaimPostureError>`), asserting invariants that are true by construction of the new model, e.g.:

```rust
pub fn validate_release_posture(posture: &ReleasePosture) -> Result<(), ClaimPostureError> {
    // Every entry has exactly one tier (guaranteed by type); assert no body is
    // simultaneously ReleaseGrade and Unsupported across the SAME backend.
    for (id, claim) in &posture.entries {
        for (id2, claim2) in &posture.entries {
            if id == id2 && claim.body == claim2.body && claim.tier != claim2.tier {
                return Err(ClaimPostureError::ConflictingTier { backend: id.as_str().to_string(), body: claim.body.to_string() });
            }
        }
    }
    Ok(())
}
```

Define `ClaimPostureError` (structured enum, `Display` + `Error`) in `claims/posture.rs`. Remove the now-unused `policy/mod.rs` consts and `policy/current.rs` accessors, and drop their `pub use` lines from `pleiades-backend/src/lib.rs`. Update `bundle_verify.rs` to call the new renderers (it re-renders in-process; no committed file changes).

- [ ] **Step 6: Update snapshot/golden tests**

Run the searches in **Files**. For each test asserting old summary substrings (e.g. "Sun through Neptune are release-grade", "source-backed validation bodies"), update the expectation to the new derived `summary_line()` shape (`Body@backend` grouped by tier). Where a test pinned exact prose, switch it to assert the structural facts (e.g. `posture.release_grade()` contains `(packaged-data, Pluto)`), which is more robust.

- [ ] **Step 7: Build, test, lint, docs**

Run: `cargo test --workspace 2>&1 | tail -25` → PASS.
Run: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features 2>&1 | tail -5` → clean (the deleted re-exports must not be referenced in rustdoc).
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -5` → clean.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(validate): derive release posture from canonical backends; retire global string claim layer"
```

---

### Task 9: Drift gate — every surface matches the derived posture

**Files:**
- Create: `crates/pleiades-validate/src/claims/drift.rs`
- Modify: `crates/pleiades-validate/src/claims/mod.rs` (export)

**Interfaces:**
- Produces: `ClaimDriftError::{ SurfaceDisagreesWithPosture { surface: String } }` (+ `Display`/`Error`); `pub fn check_claim_drift() -> Result<(), Vec<ClaimDriftError>>` comparing each rendered surface (release-body-claims summary, pluto-fallback summary, backend matrix, compatibility profile) against `derived_release_posture()`.

- [ ] **Step 1: Write the failing test** (`drift.rs` test module):

```rust
#[test]
fn drift_passes_for_freshly_rendered_surfaces() {
    assert!(super::check_claim_drift().is_ok());
}

#[test]
fn drift_detects_tampered_posture() {
    // Build a posture that disagrees with the rendered summary and assert the
    // comparator flags it. Uses an injectable comparison helper:
    use pleiades_backend::{ReleasePosture};
    let empty = ReleasePosture { entries: vec![] };
    let rendered = super::render_release_body_claims_summary_text();
    assert!(super::summary_matches_posture(&rendered, &empty).is_err());
}
```

- [ ] **Step 2: Run to verify failure** — FAIL (unresolved).

- [ ] **Step 3: Implement `drift.rs`**

```rust
use crate::claims::derived_release_posture;
use pleiades_backend::ReleasePosture;

/// A surface whose rendered claims disagree with the derived posture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimDriftError {
    /// The named surface does not contain the derived posture's claims.
    SurfaceDisagreesWithPosture { surface: String },
}

impl core::fmt::Display for ClaimDriftError { /* "surface {} disagrees with derived posture" */ }
impl std::error::Error for ClaimDriftError {}

/// Returns Ok if `rendered` reflects every release-grade entry in `posture`.
pub fn summary_matches_posture(rendered: &str, posture: &ReleasePosture) -> Result<(), ClaimDriftError> {
    for (id, body) in posture.release_grade() {
        let token = format!("{body}@{id}");
        if !rendered.contains(&token) {
            return Err(ClaimDriftError::SurfaceDisagreesWithPosture { surface: format!("release-body-claims-summary (missing {token})") });
        }
    }
    Ok(())
}

/// Checks every release-facing surface against the derived posture.
pub fn check_claim_drift() -> Result<(), Vec<ClaimDriftError>> {
    let posture = derived_release_posture();
    let mut errors = Vec::new();
    let surfaces = [
        ("release-body-claims-summary", crate::render::summary::release::render_release_body_claims_summary_text()),
        ("backend-matrix", crate::render::summary::backend::render_backend_matrix_text()),
        // add compatibility-profile + pluto-fallback renders
    ];
    for (name, rendered) in surfaces {
        if let Err(mut e) = summary_matches_posture(&rendered, &posture) {
            if let ClaimDriftError::SurfaceDisagreesWithPosture { surface } = &mut e {
                *surface = format!("{name}: {surface}");
            }
            errors.push(e);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

Expose `render_release_body_claims_summary_text` and the backend-matrix renderer via `pub(crate)` paths as needed; pick the exact renderer names that exist after Task 8. (If a surface uses a different `Body@backend` shape, normalize the comparison token accordingly.)

- [ ] **Step 4: Run tests** — `cargo test -p pleiades-validate claims::drift 2>&1 | tail -15` → PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(validate): add claim drift gate comparing surfaces to derived posture"
```

---

### Task 10: Capability audit — fast structural checks

**Files:**
- Create: `crates/pleiades-validate/src/claims/audit.rs`
- Modify: `crates/pleiades-validate/src/claims/mod.rs` (export)

**Interfaces:**
- Produces: `ClaimAuditError::{ DeclaredBodyNotComputable{backend,body}, UnsupportedBodyAccepted{backend,body}, TierEvidenceMismatch{backend,body} }` (+ `Display`/`Error`); `pub fn audit_structural() -> Result<(), Vec<ClaimAuditError>>` over `canonical_release_metadata()` — does NOT touch the corpus.

- [ ] **Step 1: Write the failing test**:

```rust
#[test]
fn structural_audit_passes_for_canonical_backends() {
    assert!(super::audit_structural().is_ok());
}

#[test]
fn structural_audit_flags_release_grade_without_corpus_evidence() {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence, CelestialBody};
    // A ReleaseGrade claim with ClaimEvidence::None|AlgorithmicModel must be rejected.
    let bad = BodyClaim::release_grade(CelestialBody::Mars, AccuracyClass::High, ClaimEvidence::None);
    assert!(super::tier_evidence_consistent(&bad).is_err());
    let bad2 = BodyClaim::release_grade(CelestialBody::Mars, AccuracyClass::High, ClaimEvidence::AlgorithmicModel);
    assert!(super::tier_evidence_consistent(&bad2).is_err());
}
```

- [ ] **Step 2: Run to verify failure** — FAIL (unresolved).

- [ ] **Step 3: Implement `audit.rs`** (structural only):

```rust
use crate::claims::canonical_release_metadata;
use pleiades_backend::{BodyClaim, BodyClaimTier, ClaimEvidence};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimAuditError {
    DeclaredBodyNotComputable { backend: String, body: String },
    UnsupportedBodyAccepted { backend: String, body: String },
    TierEvidenceMismatch { backend: String, body: String },
}
// impl Display + Error ...

/// A ReleaseGrade claim must be backed by artifact- or corpus-validated evidence.
pub fn tier_evidence_consistent(claim: &BodyClaim) -> Result<(), ()> {
    match (claim.tier, &claim.evidence) {
        (BodyClaimTier::ReleaseGrade, ClaimEvidence::ArtifactValidated)
        | (BodyClaimTier::ReleaseGrade, ClaimEvidence::CorpusValidated { .. }) => Ok(()),
        (BodyClaimTier::ReleaseGrade, _) => Err(()),
        _ => Ok(()),
    }
}

/// Structural audit: tier/evidence consistency + Unsupported bodies are rejected by preflight.
pub fn audit_structural() -> Result<(), Vec<ClaimAuditError>> {
    let mut errors = Vec::new();
    for meta in canonical_release_metadata() {
        for claim in &meta.body_claims {
            if tier_evidence_consistent(claim).is_err() {
                errors.push(ClaimAuditError::TierEvidenceMismatch {
                    backend: meta.id.as_str().to_string(),
                    body: claim.body.to_string(),
                });
            }
            // Unsupported-tier bodies must be rejected by preflight (validate_request).
            if claim.tier == BodyClaimTier::Unsupported {
                let req = crate::claims::sample_request_for(&claim.body); // helper building a minimal valid request
                if meta.validate_request(&req).is_ok() {
                    errors.push(ClaimAuditError::UnsupportedBodyAccepted {
                        backend: meta.id.as_str().to_string(),
                        body: claim.body.to_string(),
                    });
                }
            }
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

Add `sample_request_for(&CelestialBody) -> EphemerisRequest` in `claims/mod.rs` building a minimal TT/ecliptic/geocentric request (mirror existing request fixtures).

- [ ] **Step 4: Run tests** — PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(validate): add fast structural claim audit"
```

---

### Task 11: Capability audit — slow corpus accuracy-ceiling check (`#[ignore]`)

**Files:**
- Modify: `crates/pleiades-validate/src/claims/audit.rs`

**Interfaces:**
- Produces: `pub fn audit_release_grade_accuracy() -> Result<(), Vec<ClaimAuditError>>` adding `ReleaseGradeAboveCeiling { backend, body, channel }` to `ClaimAuditError`; for each `ReleaseGrade` body, compares the owning backend against the reference corpus and asserts max deltas ≤ `accuracy_ceiling`.

- [ ] **Step 1: Write the failing test** (`#[ignore]`'d, run in CI/test-full):

```rust
#[test]
#[ignore = "slow: runs corpus comparison"]
fn release_grade_bodies_meet_accuracy_ceiling() {
    assert!(super::audit_release_grade_accuracy().is_ok());
}
```

- [ ] **Step 2: Run to verify failure** — `cargo test -p pleiades-validate --  --ignored release_grade_bodies_meet 2>&1 | tail -15` → FAIL (unresolved fn).

- [ ] **Step 3: Implement `audit_release_grade_accuracy`**

Reuse the comparison harness (signatures confirmed): `compare_backends(reference, candidate, corpus) -> Result<ComparisonReport, EphemerisError>`, `report.body_summaries() -> Vec<BodyComparisonSummary>` (fields `max_longitude_delta_deg`, `max_latitude_delta_deg`, `max_distance_delta_au`), `pleiades_data::accuracy_ceiling(&body) -> AccuracyCeiling` (fields `lon_arcsec`, `lat_arcsec`, `dist_km`), and corpus loaders `pleiades_validate::corpus::default_corpus()` / `release_grade_corpus()` and `pleiades_jpl::corpus::{production_holdout_corpus, asteroid_reference_corpus}`.

```rust
use pleiades_data::accuracy_ceiling;

const DEG_TO_ARCSEC: f64 = 3600.0;
const AU_TO_KM: f64 = 149_597_870.7;

pub fn audit_release_grade_accuracy() -> Result<(), Vec<ClaimAuditError>> {
    let mut errors = Vec::new();
    // packaged-data ReleaseGrade bodies vs de440 holdout corpus.
    {
        let candidate = pleiades_data::PackagedDataBackend::default();
        let reference = crate::comparison::default_reference_backend(); // de440 snapshot
        let corpus = crate::corpus::default_corpus();
        if let Ok(report) = crate::comparison::compare_backends(&reference, &candidate, &corpus) {
            check_report(&report, "packaged-data", &candidate.metadata().release_grade_bodies(), &mut errors);
        }
    }
    // jpl-spk Tier-A asteroids vs sb441-n16 asteroid corpus.
    {
        let candidate = crate::claims::spk_release_backend();
        let reference = crate::comparison::default_reference_backend();
        let corpus = crate::corpus::asteroid_corpus(); // wrap pleiades_jpl::corpus::asteroid_reference_corpus()
        if let Ok(report) = crate::comparison::compare_backends(&reference, &candidate, &corpus) {
            check_report(&report, "jpl-spk", &candidate.metadata().release_grade_bodies(), &mut errors);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn check_report(report: &crate::comparison::ComparisonReport, backend: &str, release_bodies: &[pleiades_backend::CelestialBody], errors: &mut Vec<ClaimAuditError>) {
    for summary in report.body_summaries() {
        if !release_bodies.contains(&summary.body) { continue; }
        let ceiling = accuracy_ceiling(&summary.body);
        if summary.max_longitude_delta_deg * DEG_TO_ARCSEC > ceiling.lon_arcsec {
            errors.push(ClaimAuditError::ReleaseGradeAboveCeiling { backend: backend.to_string(), body: summary.body.to_string(), channel: "longitude".to_string() });
        }
        if summary.max_latitude_delta_deg * DEG_TO_ARCSEC > ceiling.lat_arcsec {
            errors.push(ClaimAuditError::ReleaseGradeAboveCeiling { backend: backend.to_string(), body: summary.body.to_string(), channel: "latitude".to_string() });
        }
        if let Some(d) = summary.max_distance_delta_au {
            if d * AU_TO_KM > ceiling.dist_km {
                errors.push(ClaimAuditError::ReleaseGradeAboveCeiling { backend: backend.to_string(), body: summary.body.to_string(), channel: "distance".to_string() });
            }
        }
    }
}
```

Add the `ReleaseGradeAboveCeiling { backend: String, body: String, channel: String }` variant to `ClaimAuditError` (+ `Display`). Add a thin `crate::corpus::asteroid_corpus()` wrapper around `pleiades_jpl::corpus::asteroid_reference_corpus()` returning a `ValidationCorpus` (mirror how `ValidationCorpus::jpl_snapshot()` is built from snapshot entries). If the exact reference/candidate pairing for asteroids differs from the planet path, follow the existing selected-asteroid evidence test in `pleiades-validate` as the template.

- [ ] **Step 4: Run the ignored test** — `cargo test -p pleiades-validate -- --ignored release_grade_bodies_meet 2>&1 | tail -20`
Expected: PASS (the promoted bodies are already sub-arcsec per SP3; the audit confirms it). If any body fails, that is a real finding — stop and report rather than loosening the ceiling.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(validate): add slow corpus accuracy-ceiling audit for release-grade bodies"
```

---

### Task 12: `claims-audit` CLI command + gate wiring

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (add `Some("claims-audit")` arm near the existing claim commands ~1825)
- Modify: `crates/pleiades-cli/src/cli.rs` (add `Some("claims-audit") => validate_render_cli(args),` near line 711–721)
- Modify: `mise.toml` (add `[tasks.claims-audit]`; add to `release-gate` and `ci` `depends`)
- Test: `crates/pleiades-validate/src/render/cli.rs` test module

**Interfaces:**
- Consumes: `audit_structural`, `audit_release_grade_accuracy`, `check_claim_drift` (Tasks 9–11).
- Produces: CLI command `claims-audit` running drift + structural audit (fast) and printing a report; exits non-zero on any error.

- [ ] **Step 1: Write the failing test**:

```rust
#[test]
fn claims_audit_command_reports_ok() {
    let out = render_cli(&["claims-audit".to_string()]).expect("claims-audit renders");
    assert!(out.contains("claim audit"));
    assert!(out.contains("OK") || out.contains("ok"));
}
```

- [ ] **Step 2: Run to verify failure** — FAIL (unknown command).

- [ ] **Step 3: Implement the command** in `render/cli.rs`:

```rust
        Some("claims-audit") => {
            ensure_no_extra_args(&args[1..], "claims-audit")?;
            let mut lines = vec!["claim audit".to_string()];
            match crate::claims::check_claim_drift() {
                Ok(()) => lines.push("drift: OK".to_string()),
                Err(errs) => { for e in errs { lines.push(format!("drift: {e}")); } }
            }
            match crate::claims::audit_structural() {
                Ok(()) => lines.push("structural: OK".to_string()),
                Err(errs) => { for e in errs { lines.push(format!("structural: {e}")); } }
            }
            // Fail closed: if any line is not OK, return an error so CLI exits non-zero.
            if lines.iter().any(|l| l.contains("drift: ") && !l.ends_with("OK") || l.contains("structural: ") && !l.ends_with("OK")) {
                return Err(EphemerisError::new(EphemerisErrorKind::InvalidRequest, lines.join("\n")));
            }
            Ok(format!("{}\n", lines.join("\n")))
        }
```

(The slow `audit_release_grade_accuracy` runs in the `#[ignore]`'d test + CI, not in the default CLI invocation, to keep `claims-audit` fast; add an opt-in `claims-audit --full` arg that also runs it, mirroring the env-opt-in pattern used by the latency ceiling.)

- [ ] **Step 4: Wire `pleiades-cli`** — add `Some("claims-audit") => validate_render_cli(args),` to the dispatch in `cli.rs`.

- [ ] **Step 5: Wire `mise.toml`**

```toml
[tasks.claims-audit]
run = "cargo run -q -p pleiades-validate -- claims-audit"
```

Add `"claims-audit"` to the `depends` arrays of `[tasks.release-gate]` and `[tasks.ci]`.

- [ ] **Step 6: Run tests + the command**

Run: `cargo test -p pleiades-validate claims_audit_command 2>&1 | tail -15` → PASS.
Run: `cargo run -q -p pleiades-validate -- claims-audit` → prints `drift: OK` / `structural: OK`, exit 0.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add -A
git commit -m "feat(cli): add claims-audit command and wire into release-gate/ci"
```

---

### Task 13: Align README and PLAN.md

**Files:**
- Modify: `README.md` (the "Important current limits" bullets about Pluto/lunar/asteroids)
- Modify: `PLAN.md` (Phase 3 status + "Important current limits")

**Interfaces:** none (docs).

- [ ] **Step 1: Update README**

Replace the Pluto limit bullet ("Pluto remains approximate/fallback-backed in first-party algorithmic paths…") with the per-backend reality:

> - body/backend claims are now **per-backend**: Pluto, the Moon, and Eros are release-grade via the packaged-data artifact, while VSOP87's Pluto stays approximate and the compact ELP Moon stays constrained; the seven `sb441-n16` Tier-A asteroids (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris) are release-grade via the corpus-dependent JPL/SPK backend; True Apogee/Perigee remain unsupported.

- [ ] **Step 2: Update PLAN.md**

In "Current priority" / Status, mark Phase 3 complete: claim model is per-backend and enforced by the `claims-audit` gate; update the "Important current limits" Pluto/lunar/asteroid bullets to the per-backend posture. Remove the now-resolved Phase 3 frontier note; set the active frontier to **Phase 4**.

- [ ] **Step 3: Verify gates still green**

Run: `mise test 2>&1 | tail -10` → PASS.
Run: `cargo run -q -p pleiades-validate -- claims-audit` → OK.

- [ ] **Step 4: Commit**

```bash
git add README.md PLAN.md
git commit -m "docs: align README/PLAN with per-backend claim posture (Phase 3 complete)"
```

---

## Self-Review

**Spec coverage:**
- Per-backend claim model + `BodyClaim` replacing `body_coverage` → Tasks 1–2. ✓
- Promotions (packaged-data Pluto/Moon/Eros; vsop87 constrained/approximate; elp constrained/unsupported; jpl-spk 7-body Tier-A) → Tasks 3–6. ✓
- Derived `ReleasePosture` + canonical backend set + retire global string layer → Tasks 7–8. ✓
- Audit + drift gate (structural fast + corpus slow `#[ignore]`) + CLI/gate wiring → Tasks 9–12. ✓
- README/PLAN alignment → Task 13. ✓
- `ReleaseGrade` bar = SP3 ceilings via comparison harness → Task 11. ✓
- jpl-snapshot excluded from posture → Task 8 (`canonical_release_metadata`). ✓

**Placeholder scan:** No "TBD"/"implement later". A few steps reference existing constructors/renderers whose exact names depend on the post-Task-8 state (`spk_release_backend`, the backend-matrix renderer, `asteroid_corpus` wrapper); each names the concrete existing template to copy (corpus/comparison setup, selected-asteroid evidence test). These are integration points, not unspecified logic.

**Type consistency:** `BodyClaim`/`BodyClaimTier`/`ClaimEvidence` (Task 1) used identically in Tasks 2–11; `supported_bodies()`/`claim_for()`/`release_grade_bodies()`/`merge_body_claims` defined in Task 2 and consumed consistently; `ReleasePosture`/`from_backends`/`release_grade()`/`summary_line()` defined Task 7, consumed Tasks 8–11; `ClaimAuditError`/`ClaimDriftError`/`ClaimPostureError` introduced where first used and extended (Task 11 adds a variant). `accuracy_ceiling`/`compare_backends`/`body_summaries` signatures match the captured API.

**Open integration points to resolve during implementation (named, not placeholders):** the exact `SpkBackend` constructor (`spk_release_backend`) and the asteroid `ValidationCorpus` wrapper follow the existing `pleiades-validate` corpus/comparison code; the backend-matrix renderer name is whatever exists post-Task-8. Resolve by reading the cited templates, not by inventing APIs.
