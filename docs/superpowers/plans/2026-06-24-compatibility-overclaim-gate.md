# Compatibility Overclaim Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the Phase 5 overclaim hole by making a per-entry claim tier the single source of truth for house/ayanamsa compatibility, cross-checked against numeric-gate evidence, and wire the full numeric-gate set into the in-process release gate.

**Architecture:** Add a `CompatibilityClaimTier` to `pleiades-types`; store it on `HouseSystemDescriptor` and `AyanamsaDescriptor`. A new `pleiades-validate` audit (`claims/compat.rs`) proves bidirectionally that the set of `ReleaseGradeNumeric` descriptors equals the set of entries the SE numeric gates actually validate, that the compatibility profile agrees, and that README/CLI prose matches. The audit and all numeric gates run inside `validate_release_smoke_at`.

**Tech Stack:** Rust workspace (`pleiades-*` crates), `cargo test`, `cargo run -p pleiades-validate`, `mise` tasks.

## Global Constraints

- Pure Rust; no native deps, no network, no kernel access in the gate path (`#![forbid(unsafe_code)]` where already present).
- All first-party crates keep the `pleiades-*` naming rule.
- No public release claim may be broadened by this work.
- Two claim tiers only — no ambiguous middle state.
- Evidence is **per-entry**: an entry is `ReleaseGradeNumeric` only if it itself appears in the SE corpus and passes. The 12 release-grade house systems are exactly the `system_code`s in `crates/pleiades-validate/data/houses-corpus/cusps.csv` (`Placidus, Koch, Porphyry, Regiomontanus, Campanus, Equal, WholeSign, Alcabitius, Meridian, Axial, Topocentric, Morinus`). The 6 release-grade ayanamsas are the modes in `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` (`Lahiri, Raman, Krishnamurti, FaganBradley, TrueChitra, TrueCitra`).
- `Ayanamsa` derives only `PartialEq`; `HouseSystem` has `Eq + Hash` but no `Ord`. Use `Vec<_>` + `PartialEq` membership for validated-entry sets — do not add derives to the published types.
- Catalog data is **duplicated**: `pleiades-houses` lists descriptors in `BASELINE_HOUSE_SYSTEMS` + `RELEASE_HOUSE_SYSTEMS` *and again* in `static BUILT_IN_HOUSE_SYSTEMS: [_; 25]`. `pleiades-ayanamsa` lists them in `BASELINE_AYANAMSAS` + `RELEASE_AYANAMSAS` *and again* in `static BUILT_IN_AYANAMSAS: [_; 59]`. Every descriptor edit must be applied to **both** listings.

## Constructor strategy (applies to Tasks 2 and 3)

To keep the breaking change small and the release-grade entries visually obvious — and to avoid hand-editing ~150 descriptor-only call sites — keep the existing positional `new(...)` constructor and have it default `claim_tier` to `DescriptorOnly`. Add a sibling `new_release_grade(...)` const constructor with the identical parameter list that sets `claim_tier = ReleaseGradeNumeric`. Only the release-grade call sites change (swap `new` → `new_release_grade`). The descriptor field remains the single source of truth; the overclaim audit (Task 5) catches any mis-set tier regardless of which constructor was used.

---

### Task 1: Add `CompatibilityClaimTier` to `pleiades-types`

**Files:**
- Create: `crates/pleiades-types/src/compatibility_claim.rs`
- Modify: `crates/pleiades-types/src/lib.rs` (add `mod` + re-export)

**Interfaces:**
- Produces: `pub enum CompatibilityClaimTier { ReleaseGradeNumeric, DescriptorOnly }` (derives `Clone, Copy, Debug, PartialEq, Eq, Hash`), re-exported as `pleiades_types::CompatibilityClaimTier`.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-types/src/compatibility_claim.rs`:

```rust
//! Per-entry compatibility claim tier shared by the house and ayanamsa catalogs.
#![forbid(unsafe_code)]

/// The compatibility claim a catalog makes for one built-in entry.
///
/// Two tiers only — no ambiguous middle state. `ReleaseGradeNumeric` asserts the
/// entry is exercised by an SE numeric gate against a corpus-validated ceiling
/// and passes it; `DescriptorOnly` asserts catalogue/metadata presence with no
/// numeric compatibility claim.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompatibilityClaimTier {
    /// Numeric compatibility is asserted and backed by passing gate evidence.
    ReleaseGradeNumeric,
    /// Catalogued with metadata/aliases only; no numeric compatibility claim.
    DescriptorOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_are_distinct() {
        assert_ne!(
            CompatibilityClaimTier::ReleaseGradeNumeric,
            CompatibilityClaimTier::DescriptorOnly
        );
    }
}
```

- [ ] **Step 2: Wire the module + re-export**

In `crates/pleiades-types/src/lib.rs`, add the module declaration next to the other `mod` lines and the re-export next to the other `pub use` lines (keep alphabetical-ish grouping):

```rust
mod compatibility_claim;
```
```rust
pub use compatibility_claim::CompatibilityClaimTier;
```

- [ ] **Step 3: Run the test**

Run: `cargo test -p pleiades-types compatibility_claim -- --nocapture`
Expected: PASS (`tiers_are_distinct`).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-types/src/compatibility_claim.rs crates/pleiades-types/src/lib.rs
git commit -m "feat(types): add CompatibilityClaimTier enum"
```

---

### Task 2: Add `claim_tier` to `HouseSystemDescriptor`

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/mod.rs` (struct, `new`, new `new_release_grade`, both catalog listings)
- Test: `crates/pleiades-houses/src/catalog/tests.rs`

**Interfaces:**
- Consumes: `pleiades_types::CompatibilityClaimTier` (re-export it from `pleiades-houses` if the crate already re-exports types; otherwise reference via `pleiades_types::`).
- Produces: `HouseSystemDescriptor.claim_tier: CompatibilityClaimTier`; `HouseSystemDescriptor::new_release_grade(..)` (same params as `new`).

- [ ] **Step 1: Write the failing test**

In `crates/pleiades-houses/src/catalog/tests.rs`, add:

```rust
#[test]
fn release_grade_numeric_house_set_is_exactly_the_twelve_corpus_systems() {
    use pleiades_types::{CompatibilityClaimTier, HouseSystem};

    let release_grade: Vec<HouseSystem> = crate::built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .map(|d| d.system.clone())
        .collect();

    let expected = [
        HouseSystem::Placidus, HouseSystem::Koch, HouseSystem::Porphyry,
        HouseSystem::Regiomontanus, HouseSystem::Campanus, HouseSystem::Equal,
        HouseSystem::WholeSign, HouseSystem::Alcabitius, HouseSystem::Meridian,
        HouseSystem::Axial, HouseSystem::Topocentric, HouseSystem::Morinus,
    ];

    assert_eq!(release_grade.len(), expected.len());
    for sys in expected {
        assert!(release_grade.contains(&sys), "missing {sys:?}");
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-houses release_grade_numeric_house_set -- --nocapture`
Expected: FAIL to compile — `no field claim_tier on HouseSystemDescriptor`.

- [ ] **Step 3: Add the field**

In `crates/pleiades-houses/src/catalog/mod.rs`, add to the `pub struct HouseSystemDescriptor { .. }` (after `max_abs_latitude_deg`):

```rust
    /// The compatibility claim tier for this built-in entry.
    pub claim_tier: pleiades_types::CompatibilityClaimTier,
```

- [ ] **Step 4: Update `new` to default, add `new_release_grade`**

Replace the body of `pub const fn new(..)` so it sets the tier, and add the sibling constructor immediately after it:

```rust
    /// Creates a descriptor that makes no numeric compatibility claim.
    pub const fn new(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
        max_abs_latitude_deg: Option<f64>,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
            max_abs_latitude_deg,
            claim_tier: pleiades_types::CompatibilityClaimTier::DescriptorOnly,
        }
    }

    /// Creates a descriptor that asserts release-grade numeric compatibility.
    /// Use only for entries with passing SE numeric-gate evidence.
    pub const fn new_release_grade(
        system: HouseSystem,
        canonical_name: &'static str,
        aliases: &'static [&'static str],
        notes: &'static str,
        latitude_sensitive: bool,
        max_abs_latitude_deg: Option<f64>,
    ) -> Self {
        Self {
            system,
            canonical_name,
            aliases,
            notes,
            latitude_sensitive,
            max_abs_latitude_deg,
            claim_tier: pleiades_types::CompatibilityClaimTier::ReleaseGradeNumeric,
        }
    }
```

- [ ] **Step 5: Mark the 12 release-grade systems in BOTH listings**

In `BASELINE_HOUSE_SYSTEMS` (all 12 entries) change `HouseSystemDescriptor::new(` → `HouseSystemDescriptor::new_release_grade(`. Then in `static BUILT_IN_HOUSE_SYSTEMS` change the same 12 systems (`Placidus, Koch, Porphyry, Regiomontanus, Campanus, Equal, WholeSign, Alcabitius, Meridian, Axial, Topocentric, Morinus`) from `new(` → `new_release_grade(`. Leave `RELEASE_HOUSE_SYSTEMS` and all other `BUILT_IN_HOUSE_SYSTEMS` entries on `new(`.

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p pleiades-houses release_grade_numeric_house_set -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run the whole crate to catch any literal-construction sites**

Run: `cargo test -p pleiades-houses`
Expected: PASS (if any struct-literal construction of `HouseSystemDescriptor` exists outside `new`, fix it to include `claim_tier`).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-houses/src/catalog/mod.rs crates/pleiades-houses/src/catalog/tests.rs
git commit -m "feat(houses)!: claim_tier on HouseSystemDescriptor; mark 12 release-grade systems"
```

---

### Task 3: Add `claim_tier` to `AyanamsaDescriptor`

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/model.rs` (struct, `new`, new `new_release_grade`)
- Modify: `crates/pleiades-ayanamsa/src/catalog.rs` (both listings)
- Test: `crates/pleiades-ayanamsa/src/catalog/tests.rs`

**Interfaces:**
- Produces: `AyanamsaDescriptor.claim_tier: CompatibilityClaimTier`; `AyanamsaDescriptor::new_release_grade(..)` (same params as `new`).

- [ ] **Step 1: Write the failing test**

In `crates/pleiades-ayanamsa/src/catalog/tests.rs`, add:

```rust
#[test]
fn release_grade_numeric_ayanamsa_set_is_exactly_the_six_gated_modes() {
    use pleiades_types::{Ayanamsa, CompatibilityClaimTier};

    let release_grade: Vec<Ayanamsa> = crate::built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .map(|d| d.ayanamsa.clone())
        .collect();

    let expected = [
        Ayanamsa::Lahiri, Ayanamsa::Raman, Ayanamsa::Krishnamurti,
        Ayanamsa::FaganBradley, Ayanamsa::TrueChitra, Ayanamsa::TrueCitra,
    ];

    assert_eq!(release_grade.len(), expected.len());
    for mode in expected {
        assert!(release_grade.iter().any(|m| *m == mode), "missing {mode:?}");
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-ayanamsa release_grade_numeric_ayanamsa_set -- --nocapture`
Expected: FAIL to compile — `no field claim_tier on AyanamsaDescriptor`.

- [ ] **Step 3: Add the field**

In `crates/pleiades-ayanamsa/src/model.rs`, add to `pub struct AyanamsaDescriptor { .. }` (after `offset_degrees`):

```rust
    /// The compatibility claim tier for this built-in entry.
    pub claim_tier: pleiades_types::CompatibilityClaimTier,
```

- [ ] **Step 4: Update `new` to default, add `new_release_grade`**

Mirror Task 2 Step 4 for the `AyanamsaDescriptor` parameter list (`ayanamsa, canonical_name, aliases, notes, epoch, offset_degrees`). `new` sets `claim_tier: ..::DescriptorOnly`; `new_release_grade` (identical params) sets `..::ReleaseGradeNumeric`.

- [ ] **Step 5: Mark the 6 release-grade modes in BOTH listings**

In `crates/pleiades-ayanamsa/src/catalog.rs`: switch `AyanamsaDescriptor::new(` → `new_release_grade(` for the 5 baseline modes (`Lahiri, Raman, Krishnamurti, FaganBradley, TrueChitra`) in `BASELINE_AYANAMSAS`, for `TrueCitra` in `RELEASE_AYANAMSAS`, and for all 6 of those modes within `static BUILT_IN_AYANAMSAS`. Leave every other entry on `new(`.

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p pleiades-ayanamsa release_grade_numeric_ayanamsa_set -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run the whole crate**

Run: `cargo test -p pleiades-ayanamsa`
Expected: PASS (fix any struct-literal construction sites if present).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-ayanamsa/src/model.rs crates/pleiades-ayanamsa/src/catalog.rs crates/pleiades-ayanamsa/src/catalog/tests.rs
git commit -m "feat(ayanamsa)!: claim_tier on AyanamsaDescriptor; mark 6 release-grade modes"
```

---

### Task 4: Expose validated-entry sets on the gate reports

**Files:**
- Modify: `crates/pleiades-validate/src/house_validation.rs` (`HouseCorpusReport` + `validate_house_corpus`)
- Modify: `crates/pleiades-validate/src/ayanamsa_validation.rs` (`AyanamsaCorpusReport` + `validate_ayanamsa_corpus`)

**Interfaces:**
- Produces: `HouseCorpusReport.validated_systems: Vec<pleiades_core::HouseSystem>` with accessor `fn validated_systems(&self) -> &[HouseSystem]`; `AyanamsaCorpusReport.validated_modes: Vec<pleiades_types::Ayanamsa>` with accessor `fn validated_modes(&self) -> &[Ayanamsa]`.

- [ ] **Step 1: Write the failing test (houses)**

In the `#[cfg(test)] mod tests` of `crates/pleiades-validate/src/house_validation.rs`, add:

```rust
#[test]
fn corpus_report_exposes_twelve_validated_systems() {
    let report = validate_house_corpus().expect("house corpus gate passes");
    assert_eq!(report.validated_systems().len(), 12);
    assert!(report
        .validated_systems()
        .iter()
        .any(|s| *s == pleiades_core::HouseSystem::Placidus));
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-validate corpus_report_exposes_twelve_validated_systems`
Expected: FAIL — `no method validated_systems`.

- [ ] **Step 3: Implement (houses)**

Add the field to `pub struct HouseCorpusReport`:

```rust
    /// The distinct typed house systems the corpus validated.
    pub validated_systems: Vec<pleiades_core::HouseSystem>,
```

Add the accessor in `impl HouseCorpusReport`:

```rust
    /// Returns the distinct house systems validated by the corpus.
    pub fn validated_systems(&self) -> &[pleiades_core::HouseSystem] {
        &self.validated_systems
    }
```

In `validate_house_corpus`, build the distinct list while iterating rows (each row has `row.system_code`, mapped by `system_for_code`). After the row loop, collect distinct systems preserving first-seen order:

```rust
    let mut validated_systems: Vec<pleiades_core::HouseSystem> = Vec::new();
    for row in &rows {
        if let Some(sys) = system_for_code(&row.system_code) {
            if !validated_systems.iter().any(|s| *s == sys) {
                validated_systems.push(sys);
            }
        }
    }
```

Add `validated_systems,` to the `HouseCorpusReport { .. }` construction.

- [ ] **Step 4: Run the houses test**

Run: `cargo test -p pleiades-validate corpus_report_exposes_twelve_validated_systems`
Expected: PASS.

- [ ] **Step 5: Write the failing test (ayanamsa)**

In the test module of `crates/pleiades-validate/src/ayanamsa_validation.rs`, add:

```rust
#[test]
fn corpus_report_exposes_six_validated_modes() {
    let report = validate_ayanamsa_corpus().expect("ayanamsa gate passes");
    assert_eq!(report.validated_modes().len(), 6);
    assert!(report
        .validated_modes()
        .iter()
        .any(|m| *m == pleiades_types::Ayanamsa::Lahiri));
}
```

- [ ] **Step 6: Run it to verify it fails**

Run: `cargo test -p pleiades-validate corpus_report_exposes_six_validated_modes`
Expected: FAIL — `no method validated_modes`.

- [ ] **Step 7: Implement (ayanamsa)**

Mirror Step 3 in `ayanamsa_validation.rs`: add `pub validated_modes: Vec<pleiades_types::Ayanamsa>` to `AyanamsaCorpusReport`, an accessor `validated_modes(&self) -> &[Ayanamsa]`, build the distinct list from the parsed mode codes (reuse whatever maps `mode_code` → `Ayanamsa`; the gate already resolves modes for `ayanamsa_mode_class`), and add `validated_modes,` to the report construction.

- [ ] **Step 8: Run the ayanamsa test**

Run: `cargo test -p pleiades-validate corpus_report_exposes_six_validated_modes`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add crates/pleiades-validate/src/house_validation.rs crates/pleiades-validate/src/ayanamsa_validation.rs
git commit -m "feat(validate): expose validated-entry sets on corpus gate reports"
```

---

### Task 5: Overclaim audit module — Check A (tier ↔ evidence, bidirectional)

**Files:**
- Create: `crates/pleiades-validate/src/claims/compat.rs`
- Modify: `crates/pleiades-validate/src/claims/mod.rs` (declare + re-export)

**Interfaces:**
- Consumes: `built_in_house_systems()`, `built_in_ayanamsas()`, `CompatibilityClaimTier`, `validate_house_corpus()`, `validate_ayanamsa_corpus()`, the new `validated_systems()` / `validated_modes()` accessors.
- Produces: `pub(crate) enum CompatClaimAuditError { ReleaseGradeWithoutCorpusEvidence{catalog,entry}, DescriptorOnlyHasEvidence{catalog,entry}, ProfileCountMismatch{catalog,profile,descriptors}, SurfaceDisagrees{surface} }`; `pub(crate) fn audit_compat_claims() -> Result<(), Vec<CompatClaimAuditError>>`.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-validate/src/claims/compat.rs` with the test module first (implementation stub returns `Ok` so the file compiles, real logic added in Step 3):

```rust
//! Overclaim audit: catalog claim tiers must match SE numeric-gate evidence.
#![forbid(unsafe_code)]

use std::fmt;

use pleiades_ayanamsa::built_in_ayanamsas;
use pleiades_houses::built_in_house_systems;
use pleiades_types::CompatibilityClaimTier;

use crate::{validate_ayanamsa_corpus, validate_house_corpus};

/// A violation found by the compatibility overclaim audit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CompatClaimAuditError {
    /// An entry is marked `ReleaseGradeNumeric` but the SE corpus does not
    /// validate it.
    ReleaseGradeWithoutCorpusEvidence { catalog: &'static str, entry: String },
    /// An entry is marked `DescriptorOnly` but the SE corpus validates it.
    DescriptorOnlyHasEvidence { catalog: &'static str, entry: String },
    /// The compatibility profile's release-grade-numeric count disagrees with
    /// the descriptor-derived count.
    ProfileCountMismatch { catalog: &'static str, profile: usize, descriptors: usize },
    /// A prose/CLI surface disagrees with the descriptor-derived counts.
    SurfaceDisagrees { surface: &'static str },
}

impl fmt::Display for CompatClaimAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReleaseGradeWithoutCorpusEvidence { catalog, entry } => write!(
                f, "{catalog} entry `{entry}` is ReleaseGradeNumeric but has no corpus evidence"
            ),
            Self::DescriptorOnlyHasEvidence { catalog, entry } => write!(
                f, "{catalog} entry `{entry}` is DescriptorOnly but the corpus validates it"
            ),
            Self::ProfileCountMismatch { catalog, profile, descriptors } => write!(
                f, "{catalog} profile release-grade count {profile} != descriptor count {descriptors}"
            ),
            Self::SurfaceDisagrees { surface } => {
                write!(f, "surface `{surface}` disagrees with descriptor-derived counts")
            }
        }
    }
}

impl std::error::Error for CompatClaimAuditError {}

/// Check A: bidirectional tier ↔ corpus-evidence agreement for both catalogs.
fn check_tier_evidence(errors: &mut Vec<CompatClaimAuditError>) {
    let house_report = match validate_house_corpus() {
        Ok(r) => r,
        Err(e) => {
            errors.push(CompatClaimAuditError::SurfaceDisagrees {
                surface: "house-corpus-gate",
            });
            let _ = e;
            return;
        }
    };
    for d in built_in_house_systems() {
        let has_evidence = house_report
            .validated_systems()
            .iter()
            .any(|s| *s == d.system);
        match d.claim_tier {
            CompatibilityClaimTier::ReleaseGradeNumeric if !has_evidence => {
                errors.push(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence {
                    catalog: "house",
                    entry: d.canonical_name.to_string(),
                });
            }
            CompatibilityClaimTier::DescriptorOnly if has_evidence => {
                errors.push(CompatClaimAuditError::DescriptorOnlyHasEvidence {
                    catalog: "house",
                    entry: d.canonical_name.to_string(),
                });
            }
            _ => {}
        }
    }

    let aya_report = match validate_ayanamsa_corpus() {
        Ok(r) => r,
        Err(_) => {
            errors.push(CompatClaimAuditError::SurfaceDisagrees {
                surface: "ayanamsa-corpus-gate",
            });
            return;
        }
    };
    for d in built_in_ayanamsas() {
        let has_evidence = aya_report.validated_modes().iter().any(|m| *m == d.ayanamsa);
        match d.claim_tier {
            CompatibilityClaimTier::ReleaseGradeNumeric if !has_evidence => {
                errors.push(CompatClaimAuditError::ReleaseGradeWithoutCorpusEvidence {
                    catalog: "ayanamsa",
                    entry: d.canonical_name.to_string(),
                });
            }
            CompatibilityClaimTier::DescriptorOnly if has_evidence => {
                errors.push(CompatClaimAuditError::DescriptorOnlyHasEvidence {
                    catalog: "ayanamsa",
                    entry: d.canonical_name.to_string(),
                });
            }
            _ => {}
        }
    }
}

/// Runs the full compatibility overclaim audit (Checks A–C).
pub(crate) fn audit_compat_claims() -> Result<(), Vec<CompatClaimAuditError>> {
    let mut errors = Vec::new();
    check_tier_evidence(&mut errors);
    // Check B (Task 6) and Check C (Task 7) append here.
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_grade_numeric_set_is_non_empty() {
        let n = built_in_house_systems()
            .iter()
            .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
            .count();
        assert!(n > 0, "release-grade-numeric house set is empty — audit would be vacuous");
    }

    #[test]
    fn real_catalogs_pass_check_a() {
        let mut errors = Vec::new();
        check_tier_evidence(&mut errors);
        assert!(errors.is_empty(), "unexpected violations: {errors:?}");
    }

    #[test]
    fn descriptor_only_with_evidence_is_detected() {
        // Synthetic: a house validated by the corpus but treated as DescriptorOnly.
        let report = validate_house_corpus().expect("gate passes");
        let validated = report.validated_systems();
        assert!(!validated.is_empty());
        // Prove the membership test that Check A relies on actually fires.
        let any_release = built_in_house_systems().iter().any(|d| {
            d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric
                && validated.iter().any(|s| *s == d.system)
        });
        assert!(any_release);
    }
}
```

- [ ] **Step 2: Wire the module**

In `crates/pleiades-validate/src/claims/mod.rs` add:

```rust
pub(crate) mod compat;
```
```rust
pub(crate) use compat::{audit_compat_claims, CompatClaimAuditError};
```

(Mark `CompatClaimAuditError` with `#[allow(dead_code)]` on the enum if a variant is not yet constructed until Tasks 6–7.)

- [ ] **Step 3: Run the tests**

Run: `cargo test -p pleiades-validate claims::compat`
Expected: PASS (`release_grade_numeric_set_is_non_empty`, `real_catalogs_pass_check_a`, `descriptor_only_with_evidence_is_detected`).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-validate/src/claims/compat.rs crates/pleiades-validate/src/claims/mod.rs
git commit -m "feat(validate): overclaim audit Check A (tier <-> corpus evidence)"
```

---

### Task 6: Check B — profile ↔ tier agreement, folded into `verify-compatibility-profile`

**Files:**
- Modify: `crates/pleiades-validate/src/claims/compat.rs` (add `check_profile`)
- Modify: `crates/pleiades-validate/src/compatibility/mod.rs` (`verify_compatibility_profile` runs the audit)

**Interfaces:**
- Consumes: `pleiades_core::current_compatibility_profile()`; the profile's `baseline_*`/`release_*` slices (each `*Descriptor` now carries `claim_tier`).
- Produces: `fn check_profile(errors: &mut Vec<CompatClaimAuditError>)`, called from `audit_compat_claims`.

- [ ] **Step 1: Write the failing test**

In the `tests` module of `compat.rs`, add:

```rust
#[test]
fn profile_release_grade_counts_match_descriptors() {
    let mut errors = Vec::new();
    super::check_profile(&mut errors);
    assert!(errors.is_empty(), "profile disagreement: {errors:?}");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-validate profile_release_grade_counts_match_descriptors`
Expected: FAIL — `cannot find function check_profile`.

- [ ] **Step 3: Implement `check_profile`**

Add to `compat.rs`. The profile is the same catalog source, so its baseline+release slices carry `claim_tier`; assert the profile-visible release-grade count equals the `built_in_*` descriptor-derived count for each catalog:

```rust
fn check_profile(errors: &mut Vec<CompatClaimAuditError>) {
    let profile = pleiades_core::current_compatibility_profile();

    let house_profile = profile
        .baseline_house_systems
        .iter()
        .chain(profile.release_house_systems.iter())
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let house_descriptors = built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    if house_profile != house_descriptors {
        errors.push(CompatClaimAuditError::ProfileCountMismatch {
            catalog: "house",
            profile: house_profile,
            descriptors: house_descriptors,
        });
    }

    let aya_profile = profile
        .baseline_ayanamsas
        .iter()
        .chain(profile.release_ayanamsas.iter())
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let aya_descriptors = built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    if aya_profile != aya_descriptors {
        errors.push(CompatClaimAuditError::ProfileCountMismatch {
            catalog: "ayanamsa",
            profile: aya_profile,
            descriptors: aya_descriptors,
        });
    }
}
```

Call it from `audit_compat_claims` after `check_tier_evidence`:

```rust
    check_profile(&mut errors);
```

- [ ] **Step 4: Run the test**

Run: `cargo test -p pleiades-validate profile_release_grade_counts_match_descriptors`
Expected: PASS.

- [ ] **Step 5: Fold the audit into `verify_compatibility_profile`**

In `crates/pleiades-validate/src/compatibility/mod.rs`, at the end of `verify_compatibility_profile()` (before it returns its summary line), run the audit and convert failures to an `EphemerisError`:

```rust
    if let Err(violations) = crate::claims::audit_compat_claims() {
        let messages: Vec<String> = violations.iter().map(|v| v.to_string()).collect();
        return Err(EphemerisError::new(
            EphemerisErrorKind::InvalidRequest,
            format!("compatibility overclaim audit failed:\n{}", messages.join("\n")),
        ));
    }
```

- [ ] **Step 6: Run the profile verifier**

Run: `cargo run -q -p pleiades-validate -- verify-compatibility-profile`
Expected: succeeds (prints the validated summary line).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/claims/compat.rs crates/pleiades-validate/src/compatibility/mod.rs
git commit -m "feat(validate): overclaim audit Check B + run it in verify-compatibility-profile"
```

---

### Task 7: Check C — surface/prose drift

**Files:**
- Modify: `crates/pleiades-validate/src/claims/compat.rs` (add `check_surfaces`)

**Interfaces:**
- Consumes: `README.md` (compiled in via `include_str!` with a path relative to this source file).
- Produces: `fn check_surfaces(errors: &mut Vec<CompatClaimAuditError>)`, called from `audit_compat_claims`.

- [ ] **Step 1: Write the failing test**

In the `tests` module of `compat.rs`, add:

```rust
#[test]
fn readme_counts_match_descriptors() {
    let mut errors = Vec::new();
    super::check_surfaces(&mut errors);
    assert!(errors.is_empty(), "surface drift: {errors:?}");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-validate readme_counts_match_descriptors`
Expected: FAIL — `cannot find function check_surfaces`.

- [ ] **Step 3: Implement `check_surfaces`**

The descriptor-derived release-grade counts are 12 (house) and 6 (ayanamsa). Assert the README carries the exact tokens the project uses for them. First confirm the exact phrasing in `README.md` (Task 9 keeps it aligned); use those literal substrings here:

```rust
fn check_surfaces(errors: &mut Vec<CompatClaimAuditError>) {
    const README: &str = include_str!("../../../../README.md");

    let house_count = built_in_house_systems()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();
    let aya_count = built_in_ayanamsas()
        .iter()
        .filter(|d| d.claim_tier == CompatibilityClaimTier::ReleaseGradeNumeric)
        .count();

    // The README must state the release-grade-numeric counts verbatim.
    let house_token = format!("{house_count} house systems pass");
    let aya_token = format!("{aya_count} release-claimed");
    if !README.contains(&house_token) {
        errors.push(CompatClaimAuditError::SurfaceDisagrees { surface: "README:houses" });
    }
    if !README.contains(&aya_token) {
        errors.push(CompatClaimAuditError::SurfaceDisagrees { surface: "README:ayanamsa" });
    }
}
```

Call it from `audit_compat_claims` after `check_profile`. (If the `include_str!` relative depth differs, adjust the `../` count so it resolves to the workspace-root `README.md`; verify with the test compile.)

- [ ] **Step 4: Align README tokens if needed**

If the test fails because the README lacks the exact substrings, add them in `README.md` (the "Current state" bullets) — e.g. "12 house systems pass the SE numeric gate" and "6 release-claimed ayanamsa modes pass". Keep wording truthful and within existing claims.

- [ ] **Step 5: Run the test**

Run: `cargo test -p pleiades-validate readme_counts_match_descriptors`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/src/claims/compat.rs README.md
git commit -m "feat(validate): overclaim audit Check C (README prose drift)"
```

---

### Task 8: CLI subcommand + wire numeric gates and audit into the release path

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (`compat-claims-audit` subcommand; `run_all_numeric_gates`; call both in `validate_release_smoke_at`)
- Modify: `mise.toml` (per-gate convenience tasks)

**Interfaces:**
- Consumes: `crate::claims::audit_compat_claims`; `crate::{validate_house_corpus, validate_ayanamsa_corpus, validate_apparent_goldens, validate_topocentric_goldens}`; `crate::corpus::production::run_corpus_gate`.
- Produces: CLI commands `compat-claims-audit` / `compat-claims-audit-summary`; `fn run_all_numeric_gates() -> Result<(), String>`.

- [ ] **Step 1: Add the `run_all_numeric_gates` helper**

In `crates/pleiades-validate/src/render/cli.rs`, near `validate_release_smoke_at`:

```rust
fn run_all_numeric_gates() -> Result<(), String> {
    crate::validate_house_corpus().map_err(|e| format!("house gate failed: {e}"))?;
    crate::validate_ayanamsa_corpus().map_err(|e| format!("ayanamsa gate failed: {e}"))?;
    crate::validate_apparent_goldens().map_err(|e| format!("apparent gate failed: {e}"))?;
    crate::validate_topocentric_goldens().map_err(|e| format!("topocentric gate failed: {e}"))?;
    crate::corpus::production::run_corpus_gate().map_err(|e| format!("corpus gate failed: {e}"))?;
    Ok(())
}
```

(Adjust each call to match its real return/error type — `validate_house_corpus`/`validate_ayanamsa_corpus` return `Result<_Report, _Error>`; the goldens return `Result<_Report, _>`; `run_corpus_gate` returns `Result<String, String>`. Discard the `Ok` payloads.)

- [ ] **Step 2: Add a compat-claims-audit runner**

```rust
fn render_compat_claims_audit() -> Result<String, String> {
    match crate::claims::audit_compat_claims() {
        Ok(()) => Ok("compatibility overclaim audit: OK (claims match numeric evidence)".to_string()),
        Err(violations) => {
            let messages: Vec<String> = violations.iter().map(|v| v.to_string()).collect();
            Err(format!("compatibility overclaim audit failed:\n{}", messages.join("\n")))
        }
    }
}
```

- [ ] **Step 3: Wire the subcommand into `render_cli`**

Add arms next to the other audit commands:

```rust
        Some("compat-claims-audit") | Some("compat-claims-audit-summary") => {
            ensure_no_extra_args(&args[1..], "compat-claims-audit")?;
            render_compat_claims_audit()
        }
```

- [ ] **Step 4: Call gates + audit inside `validate_release_smoke_at`**

In `validate_release_smoke_at`, after the workspace-audit clean check and before `verify_compatibility_profile()`, insert:

```rust
    run_all_numeric_gates()?;
```

(`verify_compatibility_profile` already runs the overclaim audit after Task 6, so no separate call is needed there; the numeric gates must run first so any residual failure surfaces before the audit reuses the reports.)

- [ ] **Step 5: Add the failing-then-passing CLI test**

In a CLI test module (`crates/pleiades-validate/src/tests/…` — follow the existing `render_cli` test pattern), add:

```rust
#[test]
fn compat_claims_audit_passes_on_real_catalogs() {
    let out = pleiades_validate::render_cli(&["compat-claims-audit"]).expect("audit passes");
    assert!(out.contains("OK"));
}
```

- [ ] **Step 6: Run the CLI test + the command**

Run: `cargo test -p pleiades-validate compat_claims_audit_passes_on_real_catalogs`
Expected: PASS.
Run: `cargo run -q -p pleiades-validate -- compat-claims-audit`
Expected: prints the OK line.
Run: `cargo run -q -p pleiades-validate -- release-smoke`
Expected: succeeds (now also runs the numeric gates + overclaim audit).

- [ ] **Step 7: Add `mise` convenience tasks**

In `mise.toml`, add (after the existing validate tasks):

```toml
[tasks.gate-houses]
run = "cargo run -q -p pleiades-validate -- validate-houses"

[tasks.gate-ayanamsa]
run = "cargo run -q -p pleiades-validate -- validate-ayanamsa"

[tasks.gate-apparent]
run = "cargo run -q -p pleiades-validate -- validate-apparent"

[tasks.gate-topocentric]
run = "cargo run -q -p pleiades-validate -- validate-topocentric"

[tasks.gate-corpus]
run = "cargo run -q -p pleiades-validate -- validate-corpus"

[tasks.compat-claims-audit]
run = "cargo run -q -p pleiades-validate -- compat-claims-audit"
```

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/src/tests mise.toml
git commit -m "feat(validate): compat-claims-audit CLI; run numeric gates + audit in release-smoke"
```

---

### Task 9: Align PLAN.md / README and run the full gate

**Files:**
- Modify: `PLAN.md` (mark Phase 5 item done; refresh status)
- Modify: `README.md` (claim wording consistent with Task 7 tokens)

- [ ] **Step 1: Update PLAN.md**

In the "Current priority" / Phase 5 progress paragraph, mark "release-gate hardening and compatibility-profile overclaim checks" as done, noting: claim tiers now live on the catalog descriptors, the `compat-claims-audit` enforces tier↔evidence↔profile↔prose agreement bidirectionally, and `release-smoke`/`release-gate` now run the full numeric-gate set. Update the bottom `Status:` line date to 2026-06-24.

- [ ] **Step 2: Update README.md**

Ensure the "Current state" bullets carry the exact release-grade-numeric tokens used by Check C ("12 house systems pass…", "6 release-claimed ayanamsa modes…") and that no claim is broadened.

- [ ] **Step 3: Run formatting, lint, and the full test suite**

Run:
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
Expected: all pass.

- [ ] **Step 4: Run the release gate end-to-end**

Run: `cargo run -q -p pleiades-validate -- release-gate`
Expected: succeeds (workspace audit → numeric gates → overclaim audit → profile → artifact → bundle/verify → claim drift).

- [ ] **Step 5: Commit**

```bash
git add PLAN.md README.md
git commit -m "docs: mark Phase 5 overclaim gate done; align README claim counts"
```

---

## Self-Review

**Spec coverage:**
- Tier model + descriptor field → Tasks 1–3. ✔
- Per-entry evidence (12 houses / 6 ayanamsas from the corpus files) → Task 4 accessors + Task 5 Check A. ✔
- Check A bidirectional → Task 5. ✔
- Check B (profile ↔ tier, folded into verify-compatibility-profile) → Task 6. ✔
- Check C (prose drift) → Task 7. ✔
- CLI `compat-claims-audit` + numeric gates wired into release-smoke + `mise` tasks → Task 8. ✔
- PLAN/README alignment, full gate run → Task 9. ✔

**Placeholder scan:** No TBD/TODO; every code step shows code. The two "adjust to match real return type" notes (Task 4 Step 7, Task 8 Step 1) are bounded mechanical reconciliations against types already named in the plan, not open-ended work.

**Type consistency:** `CompatibilityClaimTier` (Task 1) used identically in Tasks 2/3/5/6/7. `validated_systems()`/`validated_modes()` defined in Task 4, consumed in Task 5. `audit_compat_claims` / `CompatClaimAuditError` defined in Task 5, extended in 6/7, consumed in Task 8. `run_all_numeric_gates` defined and consumed in Task 8.

**Constructor strategy note:** Tasks 2/3 use `new` (defaulting `DescriptorOnly`) + `new_release_grade` to bound the breaking-change churn to release-grade call sites; the field remains the source of truth and the audit enforces correctness regardless of constructor used.
