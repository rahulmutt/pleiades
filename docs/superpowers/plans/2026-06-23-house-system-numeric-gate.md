# House-System Numeric Gate (Phase 5, Sub-cycle A) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fail-closed `validate-houses` release gate that proves the 11 baseline house systems reproduce Swiss Ephemeris reference cusps within per-formula-family ceilings, plus strict-by-default high-latitude rejection with an opt-in SE-compat fallback.

**Architecture:** Three layers. (1) **House-engine behavior** (pure Rust, ships): a documented latitude bound on `HouseSystemDescriptor`, strict `InvalidLatitude` rejection in `calculate_houses`, and an opt-in `HighLatitudePolicy` on `HouseRequest`. (2) **The gate** (pure Rust, ships): a committed CSV corpus + checksum-pinned manifest under `pleiades-validate/data/`, validated offline against recomputed pleiades cusps — mirroring `validate-apparent`/`validate-topocentric`. (3) **Offline generation tooling** (maintainer-only, *excluded from the workspace*): an isolated Swiss Ephemeris harness that produces the reference corpus, and a `devenv.nix` patched-Astrolog best-effort cross-check. The shipping crates and their lockfile stay pure-Rust.

**Tech Stack:** Rust (edition 2021, rust-version 1.96.0), `cargo test`, FNV-1a checksums via `pleiades_apparent::fnv1a64`, Swiss Ephemeris via the `swisseph` crate (verification-only, out-of-workspace), Astrolog via Nix/devenv (best-effort).

**Spec:** `docs/superpowers/specs/2026-06-23-house-system-numeric-gate-design.md`

## Global Constraints

Every task's requirements implicitly include these:

- **C1 — Pure-Rust workspace audit (hard).** No shipping/workspace crate may introduce a `links` key, a `-sys` dependency, a `build.rs`, or a `-sys` package in the workspace `Cargo.lock`. The only exempt lockfile phantoms are `cc`, `ring`, `windows-sys`. The Swiss Ephemeris harness is FFI over libswe and **must not** be a workspace member and **must not** enter the workspace `Cargo.lock`. It lives under `tools/` and is added to the root `[workspace] exclude` list with its own isolated `Cargo.lock`.
- **C2 — SE binding license.** The chosen SE crate declares no license in index metadata; SE is dual-licensed AGPL-3.0 / commercial. Confirm and record the actual license (and SE data-file license) before committing the harness. Verification-only, non-shipping use.
- **Fail-closed.** Gates return `Err` on any drift, missing data, or out-of-tolerance residual. Never mask a regression.
- **Versions/policy.** `edition = "2021"`, `rust-version = "1.96.0"`, `license = "MIT OR Apache-2.0"`. Shipping modules carry `#![forbid(unsafe_code)]` where the surrounding file already does.
- **Source of truth.** A clean checkout stays tool-free: the committed corpus is authoritative; generation tooling is maintainer-only and never a runtime/shipping dependency (de440 precedent: `PLEIADES_DE_KERNEL`, `horizons-fetch`).
- **Baseline system set.** The gate iterates `pleiades_houses::baseline_house_systems()` — the code is the source of truth for "the baseline set", not a hardcoded list. Latitude-sensitive baseline systems are Placidus, Koch, Topocentric.
- **Resolved decisions (from spec finalization).** Corpus location = `crates/pleiades-validate/data/houses-corpus/`. Threshold module = `crates/pleiades-houses/src/thresholds.rs`. SE crate = `swisseph` 0.1.1 (high-level). Astrolog cross-check = best-effort/optional (`not-run` is a valid, non-failing state).

---

## File Structure

**Shipping (pure Rust, in workspace):**
- `crates/pleiades-houses/src/catalog/mod.rs` (modify) — add `max_abs_latitude_deg` to `HouseSystemDescriptor`; populate for latitude-sensitive systems.
- `crates/pleiades-houses/src/systems/mod.rs` (modify) — `HighLatitudePolicy` enum; field on `HouseRequest`; strict bound + SE-compat fallback in `calculate_houses`.
- `crates/pleiades-houses/src/thresholds.rs` (create) — per-formula-family arcsecond ceilings; `house_family_ceiling()`.
- `crates/pleiades-houses/src/lib.rs` (modify) — `pub mod thresholds;` re-export.
- `crates/pleiades-validate/src/house_validation.rs` (create) — corpus parser, checksum gate, residual/strict/fallback validation, `validate_house_corpus()`.
- `crates/pleiades-validate/src/lib.rs` (modify) — `mod house_validation;` + re-export.
- `crates/pleiades-validate/src/render/cli.rs` (modify) — `validate-houses` / `houses-gate` dispatch.
- `crates/pleiades-validate/data/houses-corpus/cusps.csv` (create, committed) — SE reference cusps.
- `crates/pleiades-validate/data/houses-corpus/manifest.txt` (create, committed) — checksums + engine provenance.

**Maintainer-only (excluded from workspace):**
- `Cargo.toml` (modify) — add `tools/se-house-reference` to `[workspace] exclude`.
- `tools/se-house-reference/{Cargo.toml,Cargo.lock,src/main.rs}` (create) — SE harness.
- `tools/se-house-reference/LICENSE-NOTES.md` (create) — C2 license record.
- `devenv.nix`, `devenv.yaml` (create) — patched Astrolog + verification shell.

---

## Task 1: House-engine — documented latitude bound on the descriptor

**Files:**
- Modify: `crates/pleiades-houses/src/catalog/mod.rs` (struct `HouseSystemDescriptor` ~lines 13-24; baseline array ~lines 775-910)
- Test: same file's `#[cfg(test)] mod tests` (or `catalog/tests.rs`)

**Interfaces:**
- Produces: `HouseSystemDescriptor.max_abs_latitude_deg: Option<f64>` — `Some(bound)` for latitude-sensitive systems (Placidus, Koch, Topocentric), `None` otherwise. Accessor not needed (field is `pub`).

- [ ] **Step 1: Write the failing test**

Add to the catalog tests module:

```rust
#[test]
fn latitude_sensitive_systems_carry_a_latitude_bound() {
    for descriptor in baseline_house_systems() {
        if descriptor.latitude_sensitive {
            assert!(
                descriptor.max_abs_latitude_deg.is_some(),
                "latitude-sensitive system {:?} must declare max_abs_latitude_deg",
                descriptor.system
            );
            let bound = descriptor.max_abs_latitude_deg.unwrap();
            assert!(
                (60.0..=89.0).contains(&bound),
                "{:?} bound {bound} out of expected polar range",
                descriptor.system
            );
        } else {
            assert!(
                descriptor.max_abs_latitude_deg.is_none(),
                "non-latitude-sensitive system {:?} must not declare a bound",
                descriptor.system
            );
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-houses latitude_sensitive_systems_carry_a_latitude_bound -- --nocapture`
Expected: FAIL — compile error `no field max_abs_latitude_deg on type HouseSystemDescriptor`.

- [ ] **Step 3: Add the field and populate it**

In `HouseSystemDescriptor`:

```rust
pub struct HouseSystemDescriptor {
    pub system: HouseSystem,
    pub canonical_name: &'static str,
    pub aliases: &'static [&'static str],
    pub notes: &'static str,
    pub latitude_sensitive: bool,
    /// Maximum |geographic latitude| (degrees) at which this system yields a
    /// well-defined cusp set. `Some(bound)` only for latitude-sensitive systems;
    /// beyond it `calculate_houses` returns `InvalidLatitude` under the strict
    /// default. `None` for systems that are defined at every latitude.
    pub max_abs_latitude_deg: Option<f64>,
}
```

In the baseline array entries, add `max_abs_latitude_deg:` to every descriptor literal. For Placidus, Koch, Topocentric use `Some(66.0)` (just inside the polar circle, matching the in-band/strict-rejection fixture split). For all others use `None`. Example (Placidus):

```rust
HouseSystemDescriptor {
    system: HouseSystem::Placidus,
    canonical_name: "Placidus",
    aliases: &["P", "placidus"],
    notes: "...existing notes...",
    latitude_sensitive: true,
    max_abs_latitude_deg: Some(66.0),
},
```

Apply the same `None`/`Some(66.0)` addition to **every** descriptor literal in both `BASELINE_HOUSE_SYSTEMS` and `RELEASE_HOUSE_SYSTEMS` (the struct field is non-optional at the literal level). Release-array latitude-sensitive systems (if any, e.g. `Sunshine`) also get a bound; non-sensitive get `None`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses`
Expected: PASS (the new test plus all existing catalog tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/catalog/mod.rs
git commit -m "feat(houses): add documented latitude bound to house-system descriptor"
```

---

## Task 2: House-engine — strict `InvalidLatitude` rejection beyond the bound

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs` (`calculate_houses` ~line 181)
- Test: same file's `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: `HouseSystemDescriptor.max_abs_latitude_deg` (Task 1); `pleiades_houses::catalog::descriptor(&HouseSystem)`; `ObserverLocation` latitude accessor (`observer.latitude().degrees()` — confirm exact accessor on `ObserverLocation`).
- Produces: `calculate_houses` returns `Err(HouseError { kind: HouseErrorKind::InvalidLatitude, .. })` when `|lat| > bound` for a bounded system, **before** computing cusps.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn placidus_beyond_bound_is_rejected_strictly() {
    use pleiades_types::{HouseSystem, ObserverLocation};
    // 80°N is above the polar circle; Placidus carries a 66° bound.
    let observer = ObserverLocation::from_degrees(80.0, 0.0, 0.0)
        .expect("valid observer");
    let request = HouseRequest::new(sample_instant(), observer, HouseSystem::Placidus);
    let err = calculate_houses(&request).expect_err("must reject beyond-bound latitude");
    assert_eq!(err.kind, HouseErrorKind::InvalidLatitude);
}

#[test]
fn placidus_within_bound_is_accepted() {
    use pleiades_types::{HouseSystem, ObserverLocation};
    let observer = ObserverLocation::from_degrees(55.0, 0.0, 0.0)
        .expect("valid observer");
    let request = HouseRequest::new(sample_instant(), observer, HouseSystem::Placidus);
    calculate_houses(&request).expect("in-band latitude must succeed");
}
```

If a `sample_instant()` / `ObserverLocation::from_degrees` helper does not already exist in the test module, define `sample_instant()` using the same construction other tests in this file use (grep the file for existing `Instant::` test setup and reuse it verbatim). Confirm `ObserverLocation`'s constructor name; reuse whatever the existing house tests call.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-houses placidus_beyond_bound_is_rejected_strictly`
Expected: FAIL — `calculate_houses` currently returns `Ok` (garbage cusps) at 80°N.

- [ ] **Step 3: Add the strict check at the top of `calculate_houses`**

Immediately after `let obliquity = validated_obliquity(request)?;`:

```rust
    // Strict-default high-latitude policy: reject beyond a latitude-sensitive
    // system's documented bound rather than emitting garbage cusps. The opt-in
    // SE-compat fallback (Task 3) intercepts before this point.
    if let Some(descriptor) = crate::catalog::descriptor(&request.system) {
        if let Some(bound) = descriptor.max_abs_latitude_deg {
            let lat = request.observer.latitude().degrees();
            if lat.abs() > bound {
                return Err(HouseError::new(
                    HouseErrorKind::InvalidLatitude,
                    format!(
                        "{} is undefined beyond |latitude| {bound}\u{00b0} (got {lat:.4}\u{00b0})",
                        request.system
                    ),
                ));
            }
        }
    }
```

Confirm `HouseError::new(kind, message)` is the existing constructor (grep `error.rs`); if the constructor differs, match it exactly.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs
git commit -m "feat(houses): strict InvalidLatitude rejection beyond documented bound"
```

---

## Task 3: House-engine — opt-in `HighLatitudePolicy` SE-compat fallback

**Files:**
- Modify: `crates/pleiades-houses/src/systems/mod.rs` (`HouseRequest` ~lines 22-48; `calculate_houses` ~line 181)
- Test: same file's `#[cfg(test)] mod tests`

**Interfaces:**
- Produces:
  - `pub enum HighLatitudePolicy { Strict, SwissEphemerisFallback }` (derives `Clone, Copy, Debug, PartialEq, Eq`), `Default` = `Strict`.
  - `HouseRequest.high_latitude_policy: HighLatitudePolicy` (defaulted in `new`), builder `with_high_latitude_policy(self, policy) -> Self`.
  - Under `SwissEphemerisFallback`, beyond-bound latitude-sensitive systems substitute **Porphyry** cusps (SE's documented high-latitude substitution — verify in Task 9 against the SE harness) and the snapshot records the substitution.
- Consumes: `porphyry_houses(angles)` (already in this file, used by `HouseSystem::Porphyry`).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn se_compat_fallback_substitutes_porphyry_beyond_bound() {
    use pleiades_types::{HouseSystem, ObserverLocation};
    let observer = ObserverLocation::from_degrees(80.0, 0.0, 0.0).unwrap();
    let request = HouseRequest::new(sample_instant(), observer, HouseSystem::Placidus)
        .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);
    let snapshot = calculate_houses(&request)
        .expect("SE-compat fallback must succeed beyond bound");

    // Same instant/observer under Porphyry directly:
    let porphyry = calculate_houses(&HouseRequest::new(
        sample_instant(),
        ObserverLocation::from_degrees(80.0, 0.0, 0.0).unwrap(),
        HouseSystem::Porphyry,
    ))
    .expect("porphyry is defined at all latitudes");

    assert_eq!(snapshot.cusps, porphyry.cusps,
        "fallback cusps must equal Porphyry cusps");
}

#[test]
fn strict_policy_is_the_default() {
    assert_eq!(HighLatitudePolicy::default(), HighLatitudePolicy::Strict);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-houses se_compat_fallback_substitutes_porphyry_beyond_bound`
Expected: FAIL — `HighLatitudePolicy` / `with_high_latitude_policy` undefined.

- [ ] **Step 3: Add the enum, the field, and the fallback branch**

Add the enum near `HouseRequest`:

```rust
/// Behaviour when a latitude-sensitive system is requested beyond its
/// documented latitude bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HighLatitudePolicy {
    /// Reject with `InvalidLatitude` (the safe default).
    #[default]
    Strict,
    /// Reproduce Swiss Ephemeris's documented substitution (Porphyry) instead
    /// of erroring, recording the substitution in the snapshot provenance.
    SwissEphemerisFallback,
}
```

Add the field and builder to `HouseRequest` (default in `new`):

```rust
    pub high_latitude_policy: HighLatitudePolicy,
```

```rust
    /// Selects the high-latitude behaviour (default: `Strict`).
    pub fn with_high_latitude_policy(mut self, policy: HighLatitudePolicy) -> Self {
        self.high_latitude_policy = policy;
        self
    }
```

In `calculate_houses`, replace the strict block from Task 2 so the fallback intercepts first:

```rust
    if let Some(descriptor) = crate::catalog::descriptor(&request.system) {
        if let Some(bound) = descriptor.max_abs_latitude_deg {
            let lat = request.observer.latitude().degrees();
            if lat.abs() > bound {
                match request.high_latitude_policy {
                    HighLatitudePolicy::Strict => {
                        return Err(HouseError::new(
                            HouseErrorKind::InvalidLatitude,
                            format!(
                                "{} is undefined beyond |latitude| {bound}\u{00b0} (got {lat:.4}\u{00b0})",
                                request.system
                            ),
                        ));
                    }
                    HighLatitudePolicy::SwissEphemerisFallback => {
                        let angles = derive_angles(request.instant, &request.observer, obliquity);
                        return Ok(HouseSnapshot {
                            system: request.system.clone(),
                            instant: request.instant,
                            observer: request.observer.clone(),
                            obliquity,
                            angles,
                            cusps: porphyry_houses(angles).into(),
                        });
                    }
                }
            }
        }
    }
```

Note: if a snapshot-level provenance field is desired for the substitution, defer it to Sub-cycle B (metadata audit); for sub-cycle A the cusp equality with Porphyry is the validated contract.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/systems/mod.rs
git commit -m "feat(houses): opt-in SE-compat high-latitude fallback policy"
```

---

## Task 4: House-engine — per-formula-family arcsecond ceilings module

**Files:**
- Create: `crates/pleiades-houses/src/thresholds.rs`
- Modify: `crates/pleiades-houses/src/lib.rs` (add `pub mod thresholds;`)
- Test: inline `#[cfg(test)] mod tests` in `thresholds.rs`

**Interfaces:**
- Produces:
  - `pub struct HouseFamilyCeiling { pub cusp_arcsec: f64, pub angle_arcsec: f64 }`
  - `pub fn house_family_ceiling(family: HouseFormulaFamily) -> HouseFamilyCeiling`
  - `pub fn house_thresholds_summary_for_report() -> String`
- Consumes: `crate::catalog::HouseFormulaFamily`.

Ceilings are **provisional** here; Task 10 tightens them from observed SE-vs-pleiades residuals. Start generous so the structure lands first.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-houses/src/thresholds.rs`:

```rust
//! Per-formula-family numeric ceilings for the house-system gate.
//!
//! Mirrors `pleiades-data/src/thresholds.rs`: a small struct of ceilings plus a
//! lookup keyed by the abstract family, and a release-facing summary line.
#![forbid(unsafe_code)]

use crate::catalog::HouseFormulaFamily;

/// Arcsecond ceilings for one formula family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HouseFamilyCeiling {
    /// Max allowed |residual| on any house cusp, arcseconds.
    pub cusp_arcsec: f64,
    /// Max allowed |residual| on Ascendant/Midheaven, arcseconds.
    pub angle_arcsec: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_division_is_tighter_than_quadrant() {
        let equal = house_family_ceiling(HouseFormulaFamily::Equal);
        let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
        assert!(equal.cusp_arcsec <= quad.cusp_arcsec);
        assert!(equal.cusp_arcsec > 0.0);
    }

    #[test]
    fn every_family_has_finite_positive_ceilings() {
        for family in [
            HouseFormulaFamily::Equal,
            HouseFormulaFamily::WholeSign,
            HouseFormulaFamily::Quadrant,
            HouseFormulaFamily::EquatorialProjection,
            HouseFormulaFamily::GreatCircle,
            HouseFormulaFamily::SolarArc,
            HouseFormulaFamily::Sector,
            HouseFormulaFamily::Custom,
            HouseFormulaFamily::Unknown,
        ] {
            let c = house_family_ceiling(family);
            assert!(c.cusp_arcsec.is_finite() && c.cusp_arcsec > 0.0);
            assert!(c.angle_arcsec.is_finite() && c.angle_arcsec > 0.0);
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-houses --lib thresholds`
Expected: FAIL — `house_family_ceiling` undefined (and module not yet wired).

- [ ] **Step 3: Implement the lookup and register the module**

Append to `thresholds.rs`:

```rust
/// Returns the provisional arcsecond ceilings for a formula family.
///
/// Space-division systems (Equal/WholeSign/Quadrant-Porphyry) are tight; the
/// iterative/projected families are looser. Values are tightened in the gate
/// rollout from measured SE-vs-pleiades residuals (see plan Task 10).
pub fn house_family_ceiling(family: HouseFormulaFamily) -> HouseFamilyCeiling {
    match family {
        HouseFormulaFamily::Equal
        | HouseFormulaFamily::WholeSign => HouseFamilyCeiling { cusp_arcsec: 1.0, angle_arcsec: 1.0 },
        // Porphyry is a space-division Quadrant system: tight.
        HouseFormulaFamily::Quadrant => HouseFamilyCeiling { cusp_arcsec: 30.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::EquatorialProjection => HouseFamilyCeiling { cusp_arcsec: 15.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::GreatCircle => HouseFamilyCeiling { cusp_arcsec: 15.0, angle_arcsec: 5.0 },
        HouseFormulaFamily::SolarArc
        | HouseFormulaFamily::Sector
        | HouseFormulaFamily::Custom
        | HouseFormulaFamily::Unknown => HouseFamilyCeiling { cusp_arcsec: 60.0, angle_arcsec: 10.0 },
    }
}

/// Compact release-facing summary of the family ceilings.
pub fn house_thresholds_summary_for_report() -> String {
    let equal = house_family_ceiling(HouseFormulaFamily::Equal);
    let quad = house_family_ceiling(HouseFormulaFamily::Quadrant);
    format!(
        "House ceilings: space-division {:.1}\u{2033} cusp, quadrant {:.1}\u{2033} cusp",
        equal.cusp_arcsec, quad.cusp_arcsec
    )
}
```

Confirm the exact `HouseFormulaFamily` variant set against `catalog/mod.rs` and adjust the `match` arms to be exhaustive over the real variants. Add to `crates/pleiades-houses/src/lib.rs`:

```rust
pub mod thresholds;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-houses --lib thresholds`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-houses/src/thresholds.rs crates/pleiades-houses/src/lib.rs
git commit -m "feat(houses): per-formula-family arcsecond ceiling module"
```

---

## Task 5: SE reference harness (excluded crate) + C2 license record

**Files:**
- Modify: `Cargo.toml` (add `exclude`)
- Create: `tools/se-house-reference/Cargo.toml`
- Create: `tools/se-house-reference/src/main.rs`
- Create: `tools/se-house-reference/LICENSE-NOTES.md`

**Interfaces:**
- Produces: a maintainer-only binary that, given fixture rows on stdin (or a built-in fixture list), prints SE cusps as CSV rows in the corpus schema (Task 6). Never built by `cargo test` at the workspace root.
- Consumes: `swisseph` 0.1.1 (high-level SE binding exposing `swe_houses`).

> This task is maintainer/offline only. It MUST NOT enter the workspace `Cargo.lock` (C1). Verify it stays excluded in Step 4.

- [ ] **Step 1: Exclude the tool from the workspace**

In root `Cargo.toml`, add an `exclude` key to `[workspace]`:

```toml
[workspace]
members = [
    # ...unchanged...
]
exclude = ["tools/se-house-reference"]
resolver = "2"
```

- [ ] **Step 2: Confirm the SE crate license (C2) and record it**

Run (offline-capable check of the crate's declared license + house support):

```bash
curl -s https://index.crates.io/sw/is/swisseph | tail -1 \
  | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); print('license:', d.get('license'))"
```

Create `tools/se-house-reference/LICENSE-NOTES.md` recording: the crate's resolved license (or, if `None`, the upstream Swiss Ephemeris license — AGPL-3.0 / commercial), the SE data-file license, and the statement: *"Verification-only. Not a dependency of any shipping crate; excluded from the workspace lockfile per C1. Output (numeric cusps) is committed; the binding is never distributed."* If the license is incompatible even with non-distributed verification use, STOP and escalate before proceeding.

- [ ] **Step 3: Enumerate the authoritative baseline system set**

The gate's completeness check (Task 8) iterates `baseline_house_systems()` and requires a corpus row for **every** system it returns. So the harness must emit a row for each — not a hardcoded list. Print the exact set and their single-letter codes first:

```bash
cargo test -p pleiades-houses --lib -- --nocapture print_baseline_systems 2>/dev/null \
  || grep -nE "HouseSystem::" crates/pleiades-houses/src/catalog/mod.rs | sed -n '/BASELINE/,/]/p'
```

If no such test exists, add a throwaway one that prints `descriptor.system` + its first alias (the letter code) for each `baseline_house_systems()` entry, run it, and use that exact list in the `HSYS` table below. Note: the baseline set may include **both** `Meridian` and `Axial`, plus `Morinus` — if so the table has 12+ entries, not 11, and the row-count math in Task 6 changes accordingly.

- [ ] **Step 4: Write the harness**

`tools/se-house-reference/Cargo.toml`:

```toml
[package]
name = "se-house-reference"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
swisseph = "0.1.1"
```

`tools/se-house-reference/src/main.rs` — compute SE cusps for the fixture grid and print corpus CSV rows. The Swiss Ephemeris house entry point is `swe_houses(tjd_ut, geolat, geolon, hsys, &mut cusps, &mut ascmc)` where `cusps[1..=12]` are the house cusps and `ascmc[0]`/`ascmc[1]` are Ascendant/MC. Confirm the exact wrapper symbol/signature in the `swisseph` crate docs and adapt; the structure is:

```rust
// Fixture grid mirrors crates/pleiades-validate/data/houses-corpus (Task 6).
// Columns: chart_id, jd_ut, lat_deg, lon_deg, elev_m, system_code, cusp01..cusp12, asc, mc
const HSYS: &[(char, &str)] = &[
    ('P', "Placidus"), ('K', "Koch"), ('O', "Porphyry"), ('R', "Regiomontanus"),
    ('C', "Campanus"), ('A', "Equal"), ('W', "WholeSign"), ('B', "Alcabitius"),
    ('X', "Meridian"), ('M', "Morinus"), ('T', "Topocentric"),
];

fn main() {
    // (chart_id, jd_ut, lat, lon, elev_m)
    let fixtures = [
        ("c0_lat00", 2_451_545.0, 0.0, 0.0, 0.0),
        ("c1_lat40", 2_451_545.0, 40.0, 0.0, 0.0),
        ("c2_lat55", 2_451_545.0, 55.0, 0.0, 0.0),
        ("c3_lat66", 2_451_545.0, 66.0, 0.0, 0.0),
        ("c4_lat40_e2", 2_433_283.0, 40.0, 30.0, 0.0),
    ];
    println!("chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc");
    for (id, jd, lat, lon, elev) in fixtures {
        for (code, _name) in HSYS {
            // Pseudocode against the SE wrapper; adapt to swisseph 0.1.1 API:
            let (cusps, ascmc) = swisseph::houses(jd, lat, lon, *code as i32)
                .expect("swe_houses");
            // cusps[1..=12], ascmc[0]=asc, ascmc[1]=mc
            let row: Vec<String> = (1..=12).map(|i| format!("{:.6}", cusps[i])).collect();
            println!("{id},{jd},{lat},{lon},{elev},{code},{},{:.6},{:.6}",
                row.join(","), ascmc[0], ascmc[1]);
        }
    }
}
```

- [ ] **Step 5: Verify the tool builds in isolation and never touches the workspace lockfile**

```bash
cargo build --manifest-path tools/se-house-reference/Cargo.toml   # produces tools/se-house-reference/Cargo.lock
git -C /workspace status --porcelain Cargo.lock                    # MUST be empty (root lockfile unchanged)
grep -E '"(libswe|libswisseph)' Cargo.lock || echo "OK: no -sys in workspace lockfile"
```

Expected: root `Cargo.lock` unchanged; no `libswe*`/`libswisseph*` entries in it.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml tools/se-house-reference/Cargo.toml tools/se-house-reference/src/main.rs tools/se-house-reference/LICENSE-NOTES.md tools/se-house-reference/Cargo.lock
git commit -m "chore(verify): isolated Swiss Ephemeris house-reference harness (excluded crate)"
```

---

## Task 6: Reference corpus schema, fixtures, and committed manifest

**Files:**
- Create: `crates/pleiades-validate/data/houses-corpus/cusps.csv`
- Create: `crates/pleiades-validate/data/houses-corpus/manifest.txt`

**Interfaces:**
- Produces: the committed corpus the gate validates. CSV schema (header is literal, used by the parser in Task 7):
  `chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc`
- Manifest format (mirrors the de440 manifest):
  ```
  #Pleiades House Reference Corpus Manifest
  #Reference-Engine: SwissEphemeris <version>
  #CrossCheck-Engine: Astrolog <version+sha> | not-run
  slice cusps file=cusps.csv role=cusps rows=<N> checksum=<u64>
  ```

- [ ] **Step 1: Generate the reference CSV from the harness**

```bash
cargo run --manifest-path tools/se-house-reference/Cargo.toml \
  > crates/pleiades-validate/data/houses-corpus/cusps.csv
wc -l crates/pleiades-validate/data/houses-corpus/cusps.csv   # 1 header + (5 fixtures × |baseline set| systems)
```

The in-band fixtures are latitudes `0,40,55,66`; the strict-rejection latitudes (`70,80`) are NOT in the CSV — they are asserted by the gate's rejection check (Task 8), not by reference cusps.

- [ ] **Step 2: Compute the checksum with the workspace's FNV-1a**

Add a throwaway test that prints the checksum (or reuse the `pinned_checksum` pattern). Quickest path — a one-off test in `house_validation.rs` stub:

```rust
#[test]
fn print_corpus_checksum() {
    let csv = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/houses-corpus/cusps.csv"));
    println!("CORPUS_CHECKSUM={}", pleiades_apparent::fnv1a64(csv));
}
```

Run: `cargo test -p pleiades-validate print_corpus_checksum -- --nocapture` and copy the printed `u64`.

- [ ] **Step 3: Write the manifest**

`crates/pleiades-validate/data/houses-corpus/manifest.txt` (fill `<version>`, `<N>`, `<u64>` with real values; cross-check `not-run` until Task 11):

```
#Pleiades House Reference Corpus Manifest
#Reference-Engine: SwissEphemeris 2.10.03
#CrossCheck-Engine: not-run
slice cusps file=cusps.csv role=cusps rows=<N> checksum=<u64>
```

`<N>` = the actual data-row count (`5 × |baseline set|`); it must equal the count the gate parses (Task 8 checks `rows == manifest.rows`).

- [ ] **Step 4: Sanity-check the committed data**

```bash
head -3 crates/pleiades-validate/data/houses-corpus/cusps.csv
cat crates/pleiades-validate/data/houses-corpus/manifest.txt
```

Expected: header matches the schema; every fixture×system row present; manifest row count equals data-row count.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/data/houses-corpus/
git commit -m "data(houses): commit Swiss Ephemeris reference cusp corpus + manifest"
```

---

## Task 7: Gate — corpus parser + checksum + manifest provenance

**Files:**
- Create: `crates/pleiades-validate/src/house_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (`mod house_validation;` + re-export)
- Test: inline `#[cfg(test)] mod tests`

**Interfaces:**
- Produces (used by Tasks 8-9):
  - `struct HouseCorpusRow { chart_id: String, jd_ut: f64, lat_deg: f64, lon_deg: f64, elev_m: f64, system_code: char, cusps: [f64; 12], asc: f64, mc: f64 }`
  - `fn parse_house_corpus(csv: &str) -> Result<Vec<HouseCorpusRow>, HouseValidationError>`
  - `struct HouseManifest { reference_engine: String, crosscheck: String, rows: usize, checksum: u64 }`
  - `fn parse_house_manifest(text: &str) -> Result<HouseManifest, HouseValidationError>`
  - `enum HouseValidationError { ChecksumMismatch{expected,actual}, ManifestDrift{field,expected,actual}, MalformedRow{row,line,reason}, UnknownSystemCode{row,code}, CeilingExceeded{...}, MissingStrictRejection{system,lat}, FallbackMismatch{...} }` (last three populated in Tasks 8-9) — derive `Clone, Debug, PartialEq`, impl `Display` + `std::error::Error`.

- [ ] **Step 1: Write the failing test (parser + checksum + manifest)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc\n\
c0,2451545,0,0,0,P,1,2,3,4,5,6,7,8,9,10,11,12,1.5,10.5\n";

    #[test]
    fn parses_a_well_formed_row() {
        let rows = parse_house_corpus(SAMPLE).expect("valid");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].system_code, 'P');
        assert_eq!(rows[0].cusps[0], 1.0);
        assert_eq!(rows[0].cusps[11], 12.0);
        assert_eq!(rows[0].asc, 1.5);
    }

    #[test]
    fn rejects_short_row() {
        let bad = "chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,c1,c2,c3,c4,c5,c6,c7,c8,c9,c10,c11,c12,asc,mc\nc0,1,2,3\n";
        assert!(matches!(parse_house_corpus(bad), Err(HouseValidationError::MalformedRow { .. })));
    }

    #[test]
    fn parses_manifest_fields() {
        let m = "#Pleiades House Reference Corpus Manifest\n#Reference-Engine: SwissEphemeris 2.10.03\n#CrossCheck-Engine: not-run\nslice cusps file=cusps.csv role=cusps rows=55 checksum=12345\n";
        let parsed = parse_house_manifest(m).expect("valid manifest");
        assert_eq!(parsed.rows, 55);
        assert_eq!(parsed.checksum, 12345);
        assert_eq!(parsed.crosscheck, "not-run");
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: FAIL — module/functions undefined.

- [ ] **Step 3: Implement the module skeleton + parsers**

Create `crates/pleiades-validate/src/house_validation.rs`. Follow `apparent_validation.rs` style exactly (comment lines `#`, skip header, `splitn`, fail-closed on malformed). Implement `HouseValidationError` (full variant set above), `HouseCorpusRow`, `parse_house_corpus`, `HouseManifest`, `parse_house_manifest`. Wire the data:

```rust
const CORPUS_CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/houses-corpus/cusps.csv"));
const CORPUS_MANIFEST: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/houses-corpus/manifest.txt"));
```

Parsing rules: split each data line on `,`; require exactly 20 fields; `system_code` is the single char in field 6; `cusps` are fields 7..=18 parsed as `f64`; `asc`/`mc` are fields 19/20. The manifest parser reads `#Reference-Engine:` / `#CrossCheck-Engine:` comment values and the `slice ... rows=<n> checksum=<u64>` line (split on whitespace, then on `=`).

Register in `crates/pleiades-validate/src/lib.rs`:

```rust
mod house_validation;
pub use house_validation::{validate_house_corpus, HouseValidationError, HouseValidationReport};
```

(`validate_house_corpus`/`HouseValidationReport` are defined in Task 8; add the `mod` line now and the `pub use` once Task 8 lands, or add a temporary `pub use` of just the error type.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/house_validation.rs crates/pleiades-validate/src/lib.rs
git commit -m "feat(validate): house corpus + manifest parsers (fail-closed)"
```

---

## Task 8: Gate — numeric residual, strict-rejection, and SE-fallback checks

**Files:**
- Modify: `crates/pleiades-validate/src/house_validation.rs`
- Test: inline `#[cfg(test)] mod tests`

**Interfaces:**
- Produces:
  - `struct HouseValidationReport { rows_validated: usize, systems_checked: usize, max_cusp_residual_arcsec: f64, crosscheck: String, summary_line: String }` with `summary_line(&self) -> &str` + `Display`.
  - `fn validate_house_corpus() -> Result<HouseValidationReport, HouseValidationError>` — the full gate.
- Consumes: `parse_house_corpus`, `parse_house_manifest` (Task 7); `pleiades_houses::{calculate_houses, HouseRequest, HighLatitudePolicy}`; `pleiades_houses::thresholds::house_family_ceiling`; `pleiades_houses::catalog::{descriptor, baseline_house_systems}`; `pleiades_apparent::fnv1a64`.

- [ ] **Step 1: Write the failing tests (residual ceiling + fail-closed + strict rejection)**

```rust
#[test]
fn gate_passes_over_committed_corpus() {
    let report = validate_house_corpus().expect("committed house corpus must validate");
    assert!(report.rows_validated > 0);
    assert!(report.max_cusp_residual_arcsec.is_finite());
}

#[test]
fn checksum_drift_fails_closed() {
    // Recompute and compare against the manifest's pinned checksum.
    let actual = pleiades_apparent::fnv1a64(CORPUS_CSV);
    let manifest = parse_house_manifest(CORPUS_MANIFEST).unwrap();
    assert_eq!(actual, manifest.checksum, "corpus checksum drifted from manifest");
}

#[test]
fn strict_rejection_fires_above_polar_circle() {
    use pleiades_houses::{calculate_houses, HouseRequest};
    use pleiades_types::{HouseSystem, ObserverLocation};
    for lat in [70.0_f64, 80.0] {
        let observer = ObserverLocation::from_degrees(lat, 0.0, 0.0).unwrap();
        let req = HouseRequest::new(sample_instant(), observer, HouseSystem::Placidus);
        assert!(
            calculate_houses(&req).is_err(),
            "Placidus at {lat}\u{00b0} must be rejected"
        );
    }
}
```

Reuse/define `sample_instant()` as in Task 2 (same construction). Confirm how `system_code` maps to `HouseSystem` (`'P'->Placidus`, `'A'->Equal`, `'W'->WholeSign`, `'X'->Meridian`, `'T'->Topocentric`, `'M'->Morinus`, etc.) — implement a `fn system_for_code(c: char) -> Option<HouseSystem>` matching the harness `HSYS` table in Task 5.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: FAIL — `validate_house_corpus` undefined.

- [ ] **Step 3: Implement `validate_house_corpus`**

```rust
pub fn validate_house_corpus() -> Result<HouseValidationReport, HouseValidationError> {
    // 1. Checksum gate.
    let actual = pleiades_apparent::fnv1a64(CORPUS_CSV);
    let manifest = parse_house_manifest(CORPUS_MANIFEST)?;
    if manifest.checksum != 0 && actual != manifest.checksum {
        return Err(HouseValidationError::ChecksumMismatch { expected: manifest.checksum, actual });
    }

    let rows = parse_house_corpus(CORPUS_CSV)?;
    if rows.len() != manifest.rows {
        return Err(HouseValidationError::ManifestDrift {
            field: "rows".into(), expected: manifest.rows.to_string(), actual: rows.len().to_string(),
        });
    }

    // 2. Completeness: every baseline system appears for at least one fixture.
    use pleiades_houses::catalog::baseline_house_systems;
    for descriptor in baseline_house_systems() {
        let code = code_for_system(&descriptor.system); // inverse of system_for_code
        if !rows.iter().any(|r| r.system_code == code) {
            return Err(HouseValidationError::ManifestDrift {
                field: "completeness".into(),
                expected: format!("rows for {:?}", descriptor.system),
                actual: "missing".into(),
            });
        }
    }

    // 3. Numeric residuals vs per-family ceilings.
    let mut max_cusp_residual_arcsec = 0.0_f64;
    for (idx, row) in rows.iter().enumerate() {
        let data_row = idx + 1;
        let system = system_for_code(row.system_code).ok_or(HouseValidationError::UnknownSystemCode {
            row: data_row, code: row.system_code,
        })?;
        let family = pleiades_houses::catalog::descriptor(&system)
            .map(|d| d.formula_family())
            .unwrap_or(pleiades_houses::catalog::HouseFormulaFamily::Unknown);
        let ceiling = pleiades_houses::thresholds::house_family_ceiling(family);

        let snapshot = recompute_pleiades(row, &system)?; // builds HouseRequest, calls calculate_houses
        for (i, want) in row.cusps.iter().enumerate() {
            let got = snapshot.cusps[i].degrees();
            let resid = wrap_arcsec(got, *want);
            if resid > max_cusp_residual_arcsec { max_cusp_residual_arcsec = resid; }
            if resid > ceiling.cusp_arcsec {
                return Err(HouseValidationError::CeilingExceeded {
                    row: data_row, system_code: row.system_code, cusp: i + 1,
                    got, want: *want, residual_arcsec: resid, ceiling_arcsec: ceiling.cusp_arcsec,
                });
            }
        }
        // Asc/MC vs ceiling.angle_arcsec — same pattern against snapshot.angles.
    }

    let summary_line = format!(
        "House gate: {} rows / {} systems, max cusp residual {:.2}\u{2033}, cross-check {}",
        rows.len(), baseline_house_systems().len(), max_cusp_residual_arcsec, manifest.crosscheck,
    );
    Ok(HouseValidationReport {
        rows_validated: rows.len(),
        systems_checked: baseline_house_systems().len(),
        max_cusp_residual_arcsec,
        crosscheck: manifest.crosscheck,
        summary_line,
    })
}

fn wrap_arcsec(got: f64, want: f64) -> f64 {
    let mut d = (got - want).abs();
    if d > 180.0 { d = 360.0 - d; }
    d * 3600.0
}
```

Implement the helpers used above: `recompute_pleiades(row, system)` constructs `ObserverLocation::from_degrees(row.lat_deg, row.lon_deg, row.elev_m)`, an `Instant` from `row.jd_ut` (confirm UT→Instant construction: SE `jd_ut` vs the engine's TT scale; if the corpus is UT and the engine expects TT, document the scale handling in the harness and gate consistently — record the chosen scale in the manifest), a `HouseRequest`, and calls `calculate_houses`. Add `system_for_code`/`code_for_system` matching the Task 5 `HSYS` table. Add the new `HouseValidationError` variants (`CeilingExceeded`, `MissingStrictRejection`, `FallbackMismatch`) with `Display` arms.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: PASS — including `gate_passes_over_committed_corpus`. If a real cusp residual exceeds the provisional ceiling, that is a genuine signal: record the offending system/residual; Task 10 sets the ceiling from measured residuals (do not silently widen to hide a true defect).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/house_validation.rs
git commit -m "feat(validate): house numeric-residual gate over SE reference corpus"
```

---

## Task 9: Gate — SE-compat fallback assertion + strict-rejection assertion in the gate

**Files:**
- Modify: `crates/pleiades-validate/src/house_validation.rs` (extend `validate_house_corpus`)
- Test: inline `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: `HighLatitudePolicy::SwissEphemerisFallback` (Task 3); `pleiades_houses::calculate_houses`.
- Produces: `validate_house_corpus` additionally asserts (a) strict rejection fires for each latitude-sensitive baseline system at the strict-rejection latitudes (70°, 80°), and (b) the SE-compat fallback path produces Porphyry-equal cusps. These are computed inline (no extra corpus rows needed).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn gate_asserts_strict_and_fallback_paths() {
    // The full gate must internally exercise strict rejection + fallback;
    // a regression in either must fail the gate.
    let report = validate_house_corpus().expect("gate passes");
    assert!(report.rows_validated > 0);
}

#[test]
fn fallback_equals_porphyry_in_gate_helper() {
    use pleiades_houses::{calculate_houses, HouseRequest, HighLatitudePolicy};
    use pleiades_types::{HouseSystem, ObserverLocation};
    let obs = || ObserverLocation::from_degrees(80.0, 0.0, 0.0).unwrap();
    let fb = calculate_houses(&HouseRequest::new(sample_instant(), obs(), HouseSystem::Koch)
        .with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback)).unwrap();
    let po = calculate_houses(&HouseRequest::new(sample_instant(), obs(), HouseSystem::Porphyry)).unwrap();
    assert_eq!(fb.cusps, po.cusps);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: FAIL until the gate runs the strict/fallback assertions (the second test may pass from Task 3; the first guards the gate wiring).

- [ ] **Step 3: Add the strict + fallback assertions to `validate_house_corpus`**

After the residual loop, before building the report:

```rust
    // 4. Strict-rejection assertions: every latitude-sensitive baseline system
    //    must reject beyond its bound under the default Strict policy.
    use pleiades_houses::{calculate_houses, HouseRequest, HighLatitudePolicy};
    use pleiades_types::ObserverLocation;
    for descriptor in baseline_house_systems() {
        if descriptor.max_abs_latitude_deg.is_none() { continue; }
        for lat in [70.0_f64, 80.0] {
            let observer = ObserverLocation::from_degrees(lat, 0.0, 0.0)
                .map_err(|e| HouseValidationError::ManifestDrift {
                    field: "observer".into(), expected: "valid".into(), actual: e.to_string() })?;
            let req = HouseRequest::new(gate_instant(), observer, descriptor.system.clone());
            if calculate_houses(&req).is_ok() {
                return Err(HouseValidationError::MissingStrictRejection {
                    system: format!("{:?}", descriptor.system), lat,
                });
            }
            // 5. SE-compat fallback must succeed and equal Porphyry.
            let fb_req = req.clone().with_high_latitude_policy(HighLatitudePolicy::SwissEphemerisFallback);
            let fb = calculate_houses(&fb_req).map_err(|e| HouseValidationError::FallbackMismatch {
                system: format!("{:?}", descriptor.system), lat, reason: e.to_string() })?;
            let po = calculate_houses(&HouseRequest::new(
                gate_instant(),
                ObserverLocation::from_degrees(lat, 0.0, 0.0).unwrap(),
                pleiades_types::HouseSystem::Porphyry,
            )).map_err(|e| HouseValidationError::FallbackMismatch {
                system: "Porphyry".into(), lat, reason: e.to_string() })?;
            if fb.cusps != po.cusps {
                return Err(HouseValidationError::FallbackMismatch {
                    system: format!("{:?}", descriptor.system), lat,
                    reason: "fallback cusps differ from Porphyry".into(),
                });
            }
        }
    }
```

Add `fn gate_instant() -> Instant` (a fixed in-corpus instant). Verify Task 3's documented SE fallback (Porphyry) actually matches what the SE harness emits above the polar circle for Placidus/Koch — if SE substitutes something other than Porphyry, update both Task 3's fallback and this assertion to match the harness output (resolves spec open item: SE high-latitude fallback semantics).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/house_validation.rs
git commit -m "feat(validate): gate asserts strict-rejection and SE-compat fallback paths"
```

---

## Task 10: Tighten per-family ceilings from measured residuals

**Files:**
- Modify: `crates/pleiades-houses/src/thresholds.rs`
- Test: existing gate test `gate_passes_over_committed_corpus`

**Interfaces:** none new — only narrows the `house_family_ceiling` constants.

- [ ] **Step 1: Measure per-family residuals**

Add a temporary diagnostic test printing the max residual per family, run it, record values:

```rust
#[test]
fn print_per_family_residuals() {
    // Re-run validate_house_corpus's residual loop but collect per-family maxima.
    // Print "family=<F> max_cusp=<x>\" max_angle=<y>\"" for each.
}
```

Run: `cargo test -p pleiades-validate print_per_family_residuals -- --nocapture` and copy the maxima.

- [ ] **Step 2: Set ceilings to measured-max × safety factor**

Update each arm of `house_family_ceiling` so `cusp_arcsec` = `ceil(measured_max × 2)` (a 2× margin over observed SE-vs-pleiades residuals), with a documented floor of `1.0"` for space-division systems. Update the doc comment to state the ceilings are measured-derived, not arbitrary.

- [ ] **Step 3: Run the gate to confirm it still passes with tightened ceilings**

Run: `cargo test -p pleiades-houses --lib thresholds && cargo test -p pleiades-validate --lib house_validation`
Expected: PASS — residuals comfortably under the tightened ceilings. Remove the temporary diagnostic test.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-houses/src/thresholds.rs
git commit -m "feat(houses): tighten per-family ceilings from measured SE residuals"
```

---

## Task 11: Wire `validate-houses` into the CLI and the release-gate aggregate

**Files:**
- Modify: `crates/pleiades-validate/src/render/cli.rs` (dispatch ~lines 142-157)
- Modify: the release-gate aggregate (locate it — see Step 3)
- Test: `crates/pleiades-validate/tests/` integration test (new file) or inline CLI test

**Interfaces:**
- Consumes: `crate::validate_house_corpus()` (Task 8) and its `HouseValidationReport::summary_line()`.
- Produces: CLI commands `validate-houses` / `houses-gate`; inclusion in the aggregate release gate.

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-validate/tests/houses_gate_cli.rs`:

```rust
#[test]
fn validate_houses_command_passes() {
    let out = pleiades_validate::render::cli::render_cli(&["validate-houses"])
        .expect("validate-houses must succeed");
    assert!(out.contains("House gate"), "unexpected summary: {out}");
}

#[test]
fn houses_gate_alias_passes() {
    let out = pleiades_validate::render::cli::render_cli(&["houses-gate"]).unwrap();
    assert!(out.contains("House gate"));
}
```

Confirm `render` / `render_cli` visibility (the module is `pub fn render_cli`; ensure the path is reachable from an integration test — if `render` is private, add a thin `pub fn` re-export or test via the existing public entry point used by `validate-apparent` tests).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate --test houses_gate_cli`
Expected: FAIL — unknown command `validate-houses`.

- [ ] **Step 3: Add the dispatch arm and aggregate wiring**

In `render/cli.rs`, alongside the `validate-topocentric` arm:

```rust
        Some("validate-houses") | Some("houses-gate") => {
            ensure_no_extra_args(&args[1..], "validate-houses")?;
            crate::validate_house_corpus()
                .map(|report| report.summary_line().to_string())
                .map_err(|e| e.to_string())
        }
```

Locate the release-gate aggregate that already chains `validate-apparent` + `validate-topocentric` + `validate-corpus`:

```bash
grep -rn "validate_topocentric_goldens\|validate-topocentric\|release-gate\|release_gate" crates/pleiades-validate/src
```

Add `validate_house_corpus()` to that aggregate the same way the others are chained (collect its summary line; propagate its `Err` so the aggregate fails closed). Match the aggregate's existing error-mapping style exactly.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-validate`
Expected: PASS — CLI tests plus the full crate suite, including the aggregate.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/render/cli.rs crates/pleiades-validate/tests/houses_gate_cli.rs
git commit -m "feat(validate): validate-houses CLI gate wired into release-gate aggregate"
```

---

## Task 12: Best-effort Astrolog cross-check via devenv.nix + manifest provenance

**Files:**
- Create: `devenv.nix`, `devenv.yaml`
- Modify: `tools/se-house-reference/` (add an Astrolog cross-check sub-step or a sibling script) — OR a standalone `tools/astrolog-crosscheck.sh`
- Modify: `crates/pleiades-validate/data/houses-corpus/manifest.txt` (set `CrossCheck-Engine` when a working astrolog produced agreement)

**Interfaces:**
- Produces: a reproducible patched Astrolog (the stock nixpkgs 7.70 build crashes under modern hardening — verified). The cross-check is best-effort: if no working astrolog is built, the manifest keeps `CrossCheck-Engine: not-run` and the gate still passes (Task 8 already treats `crosscheck` as an opaque provenance string, never a pass/fail input).

- [ ] **Step 1: Write `devenv.nix` with a patched Astrolog**

```nix
{ pkgs, ... }:
let
  # Stock pkgs.astrolog 7.70 aborts under fortify/stackprotector (verified:
  # buffer-overflow -> stack-smashing -> segfault). Disable hardening so the
  # CLI runs headless for cross-check cusp emission. Verification-only.
  astrolog-patched = pkgs.astrolog.overrideAttrs (old: {
    hardeningDisable = [ "all" ];
  });
in {
  packages = [ astrolog-patched ];
  enterShell = ''
    echo "astrolog (patched): $(command -v astrolog)"
  '';
}
```

Create a minimal `devenv.yaml`:

```yaml
inputs:
  nixpkgs:
    url: github:NixOS/nixpkgs/nixpkgs-unstable
```

- [ ] **Step 2: Verify the patched Astrolog runs and emits cusps**

```bash
devenv shell -- astrolog -v        # MUST NOT crash (no "buffer overflow"/"stack smashing"/segfault)
devenv shell -- astrolog -qb 1 1 2000 12:00 0 0 0W0 40N0 -C | head -20
```

Expected: version prints; `-C` emits a house-cusp block. If still crashing after `hardeningDisable = [ "all" ]`, the cross-check stays best-effort: record the failure, leave the manifest `CrossCheck-Engine: not-run`, and proceed (do not block the plan). Capture the working/non-working outcome in `tools/se-house-reference/LICENSE-NOTES.md` or a sibling `CROSSCHECK-NOTES.md`.

- [ ] **Step 3: If working — record agreement and update the manifest**

For each fixture×system, compare Astrolog cusps to the committed SE cusps within a documented cross-tolerance (e.g. 5"). On agreement, set the manifest line to `#CrossCheck-Engine: Astrolog 7.70 (patched, hardeningDisable=all)`. On a flagged disagreement, append a note line `#CrossCheck-Exception: <system> <fixture> delta=<x>"` — flag only, never gate. Recompute and re-pin the corpus checksum only if `cusps.csv` itself changed (the cross-check does NOT modify reference values; manifest comment changes do not affect the CSV checksum).

- [ ] **Step 4: Confirm the gate is unaffected by cross-check state**

Run: `cargo test -p pleiades-validate --lib house_validation`
Expected: PASS regardless of whether `CrossCheck-Engine` is `not-run` or an Astrolog version (the gate reads it as provenance text only).

- [ ] **Step 5: Commit**

```bash
git add devenv.nix devenv.yaml crates/pleiades-validate/data/houses-corpus/manifest.txt tools/
git commit -m "chore(verify): best-effort patched-Astrolog cross-check via devenv.nix"
```

---

## Task 13: Workspace-audit + full-suite verification

**Files:** none (verification only)

- [ ] **Step 1: Confirm the pure-Rust workspace audit still passes (C1)**

```bash
cargo run -p pleiades-validate -- workspace-audit 2>/dev/null \
  || grep -rn "workspace.audit\|workspace_audit" crates/pleiades-validate/src/release | head
```

Run the workspace-audit gate however the existing release flow invokes it (locate the command name with the grep). Expected: PASS — no `links`, `-sys`, or `build.rs` introduced into the workspace; `tools/se-house-reference` is excluded and its lockfile is separate.

- [ ] **Step 2: Confirm no `-sys` package entered the workspace lockfile**

```bash
grep -nE 'name = "(libswe|libswisseph)[^"]*-sys?"' Cargo.lock || echo "OK: clean workspace lockfile"
git status --porcelain Cargo.lock   # expect empty
```

- [ ] **Step 3: Run the full affected test suites**

```bash
cargo test -p pleiades-houses
cargo test -p pleiades-validate
```

Expected: all PASS.

- [ ] **Step 4: Run the aggregate release gate end-to-end**

Invoke the aggregate release-gate command (the one Task 11 wired into) and confirm the house summary line appears and the aggregate passes.

- [ ] **Step 5: Commit any final fixups**

```bash
git add -A
git commit -m "chore(houses): finalize Phase 5 sub-cycle A house-system numeric gate" || echo "nothing to finalize"
```

---

## Self-Review notes (coverage map)

- Spec §"Reference corpus" → Tasks 5, 6 (SE harness, committed CSV + manifest with checksums + provenance).
- Spec §"Fixtures" (latitudes 0/40/55/66 in-band; 70/80 strict-reject) → Task 6 (in-band CSV) + Tasks 2, 9 (strict-reject asserted, not stored).
- Spec §"Strict-default latitude behavior" → Tasks 1, 2.
- Spec §"SE-compat opt-in" → Tasks 3, 9.
- Spec §"The gate: validate-houses" (checksum/schema/provenance, completeness, per-family residuals, strict-rejection, SE-fallback) → Tasks 7, 8, 9, 11.
- Spec §"Per-formula-family ceilings" (open item #5) → Tasks 4, 10.
- Spec §"Reproduction tooling & provisioning" → Tasks 5 (SE), 12 (Astrolog).
- Spec §"Data flow" / fail-closed → Tasks 7-9, 11.
- Spec Constraint C1 → Task 5 (exclude), Task 13 (audit verification).
- Spec Constraint C2 (license) → Task 5 Step 2.
- Spec resolved decisions (corpus location, threshold module, SE crate, best-effort Astrolog) → folded into Global Constraints + Tasks 4, 6, 12.
- Spec open item "corpus location" → resolved to `pleiades-validate/data/houses-corpus/` (Task 6).
- Spec open item "SE high-latitude fallback semantics" → Task 9 Step 3 (verify against harness, adjust Porphyry assumption if needed).
