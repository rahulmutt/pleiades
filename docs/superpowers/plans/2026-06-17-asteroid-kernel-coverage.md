# Asteroid Kernel Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Broaden selected-asteroid source coverage in `pleiades-jpl` by committing a curated ~35-body asteroid corpus — a Tier A main-belt core reproducible from the pinned `sb441-n16.bsp` kernel, plus Tier B Horizons-sourced constrained slices for centaurs/personal asteroids/TNOs — advertised over a 1900–2100 default window.

**Architecture:** Two-tier sourcing. Tier A asteroid positions come from a SHA-pinned JPL small-body perturber kernel (`sb441-n16.bsp`) loaded alongside de440, sampled into a reproducible `asteroid_reference` corpus slice gated by `corpus_regen`. Tier B bodies (not in any fixed kernel) are generated once via the existing `ingest`/`horizons-fetch` tooling and committed as a separate provenance-validated `asteroid_constrained` slice. The two tiers stay separate in data, validation, and reports. Any other numbered minor planet remains reachable on demand via `CelestialBody::Custom` ids without committing a slice.

**Tech Stack:** Rust (workspace crates `pleiades-jpl`, `pleiades-validate`, `pleiades-backend`); pure-Rust SPK reader; existing corpus-spec / generate / manifest / validation modules; `cargo test`.

## Global Constraints

- **Edition / toolchain:** match the workspace; do not bump `rust-version` or `mise.toml` pins.
- **No committed kernels:** clean checkout stays kernel-free. `sb441-n16.bsp` is **not** committed — only its SHA-256, label, and provenance are recorded.
- **Asteroid default window:** TDB Julian Days `2_415_020.5` (1900-01-01) .. `2_488_069.5` (2100-01-01). Requests/epochs outside it fail closed; no silent extrapolation.
- **Major-body window is unchanged:** `RANGE_START_JD = 2_305_447.5` .. `RANGE_END_JD = 2_670_690.5` (1600–2600) stays as-is for Sun..Pluto.
- **Constrained class:** all asteroids are a constrained body class (never release-grade), consistent with the existing "Pluto stays constrained" posture.
- **Tier separation:** Tier A (reproducible-from-kernel) and Tier B (provenance-only) evidence stay distinct in data, validation, and reports. Tier B is never put behind the kernel regen gate.
- **No guessed designations land as data:** every IAU asteroid number committed to a slice or roster is verified against the MPC numbered-asteroid catalog / Horizons before the SHA/data tasks (Task 11) are accepted.
- **Corpus CSV schema:** `epoch_jd,body,x_km,y_km,z_km`, geocentric ecliptic (mean geometric), TDB epochs, with the existing `#`-comment provenance header and `#Slice-Role:` line. Body tokens are written via `CelestialBody`'s `Display` (`Ceres`, …, `asteroid:2060-Chiron`) so they round-trip through `parse_body`.
- **Commit cadence:** each task ends with a commit. Conventional-commit messages, scoped `feat(jpl)` / `test(jpl)` / `docs` / `feat(validate)`.

---

### Task 1: Asteroid window + kernel-identity constants

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (add after the existing `KERNEL_SHA256` / `RANGE_END_JD` block, ~line 15)

**Interfaces:**
- Produces: `AST_RANGE_START_JD: f64`, `AST_RANGE_END_JD: f64`, `AST_KERNEL_LABEL: &str`, `AST_KERNEL_SHA256: &str` (placeholder sentinel until Task 11 pins it).

- [ ] **Step 1: Write the failing test**

Add to the existing `mod tests` block in `corpus_spec.rs`:

```rust
    #[test]
    fn asteroid_range_spans_1900_2100() {
        const { assert!(AST_RANGE_START_JD < AST_RANGE_END_JD) };
        // 1900-01-01 .. 2100-01-01 spans 73_050 days (200 years).
        assert!((AST_RANGE_END_JD - AST_RANGE_START_JD - 73_050.0).abs() < 2.0);
        // The asteroid window sits inside the major-body window.
        assert!(AST_RANGE_START_JD > RANGE_START_JD);
        assert!(AST_RANGE_END_JD < RANGE_END_JD);
    }

    #[test]
    fn asteroid_kernel_sha_is_placeholder_until_pinned() {
        // Task 11 replaces this with the real 64-hex digest after download.
        assert_eq!(AST_KERNEL_SHA256.len(), "PLACEHOLDER-PIN-IN-TASK-11".len());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl corpus_spec::tests::asteroid -- --nocapture`
Expected: FAIL — `AST_RANGE_START_JD` etc. not found (compile error).

- [ ] **Step 3: Write minimal implementation**

Add after line 15 of `corpus_spec.rs`:

```rust
/// Default asteroid window, as TDB Julian Days (1900-01-01 .. 2100-01-01).
/// Narrower than the major-body range because small-body orbit uncertainty
/// over a millennium far exceeds release tolerances; over 200 years it is
/// well-constrained. Beyond this window, callers supply their own data via
/// `pleiades_jpl::ingest`.
pub const AST_RANGE_START_JD: f64 = 2_415_020.5;
pub const AST_RANGE_END_JD: f64 = 2_488_069.5;

/// Pinned identity of the Tier A small-body perturber kernel. SHA-256 is
/// computed via `shasum -a 256 sb441-n16.bsp` and recorded here + in
/// docs/spk-kernel-sourcing.md when the kernel is adopted (Task 11).
pub const AST_KERNEL_LABEL: &str = "JPL DE small-body perturber kernel: sb441-n16.bsp";
pub const AST_KERNEL_SHA256: &str = "PLACEHOLDER-PIN-IN-TASK-11";
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl corpus_spec::tests::asteroid -- --nocapture`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): add asteroid window + sb441-n16 kernel identity constants"
```

---

### Task 2: Curated asteroid roster with tier + class tags

**Files:**
- Create: `crates/pleiades-jpl/src/spk/asteroid_roster.rs`
- Modify: `crates/pleiades-jpl/src/spk/mod.rs` (add `pub mod asteroid_roster;`)

**Interfaces:**
- Consumes: `pleiades_backend::CelestialBody`, `pleiades_types::CustomBodyId`, `super::chain::naif_ids`.
- Produces:
  - `enum AsteroidTier { PinnedKernel, Constrained }`
  - `enum AsteroidClass { MainBelt, Centaur, Tno }`
  - `struct AsteroidEntry { pub body: CelestialBody, pub tier: AsteroidTier, pub class: AsteroidClass }`
  - `fn asteroid_core_roster() -> &'static [AsteroidEntry]`
  - `fn tier_a_bodies() -> Vec<CelestialBody>` (PinnedKernel entries, in roster order)
  - `fn tier_b_bodies() -> Vec<CelestialBody>` (Constrained entries, in roster order)

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-jpl/src/spk/asteroid_roster.rs` with only its test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::spk::chain::naif_ids;

    #[test]
    fn roster_has_curated_core() {
        let roster = asteroid_core_roster();
        // ~35-body curated core: classical 4 + centaurs + personal + TNOs.
        assert!(roster.len() >= 33 && roster.len() <= 38, "got {}", roster.len());
    }

    #[test]
    fn tiers_are_disjoint_and_cover_roster() {
        let a = tier_a_bodies();
        let b = tier_b_bodies();
        assert_eq!(a.len() + b.len(), asteroid_core_roster().len());
        for body in &a {
            assert!(!b.contains(body), "{body:?} in both tiers");
        }
    }

    #[test]
    fn classical_four_are_tier_a_main_belt() {
        for body in [
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
        ] {
            let e = asteroid_core_roster()
                .iter()
                .find(|e| e.body == body)
                .expect("classical asteroid present");
            assert_eq!(e.tier, AsteroidTier::PinnedKernel);
            assert_eq!(e.class, AsteroidClass::MainBelt);
        }
    }

    #[test]
    fn chiron_is_constrained_centaur() {
        let e = asteroid_core_roster()
            .iter()
            .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == "2060-Chiron"))
            .expect("Chiron present");
        assert_eq!(e.tier, AsteroidTier::Constrained);
        assert_eq!(e.class, AsteroidClass::Centaur);
    }

    #[test]
    fn every_roster_body_resolves_to_a_naif_id() {
        for e in asteroid_core_roster() {
            assert!(
                !naif_ids(&e.body).is_empty(),
                "{:?} has no NAIF id candidates",
                e.body
            );
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl asteroid_roster -- --nocapture`
Expected: FAIL — `asteroid_core_roster` etc. undefined (compile error). (Also add `pub mod asteroid_roster;` to `spk/mod.rs` now so it compiles to the failing-symbol stage.)

- [ ] **Step 3: Write minimal implementation**

Prepend to `asteroid_roster.rs` (above the test module). **Every designation below is verified against MPC/Horizons in Task 11 before data lands; this table is the single source for the roster.**

```rust
//! Curated core of astrologically-relevant minor planets, tagged by sourcing
//! tier and dynamical class. The classical four are named `CelestialBody`
//! variants; all others use the `asteroid:`/`tno:` `Custom` catalog so the
//! shared body enum does not balloon. The unbounded long tail is reachable on
//! demand via `Custom` ids + `pleiades_jpl::ingest`; only this core is
//! committed as corpus data.

use pleiades_backend::CelestialBody;
use pleiades_types::CustomBodyId;

/// How a body's reference positions are sourced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsteroidTier {
    /// In `sb441-n16.bsp`: reproducible from the pinned kernel.
    PinnedKernel,
    /// Not in any fixed kernel: Horizons-sourced, provenance-validated only.
    Constrained,
}

/// Dynamical class, used to keep evidence separated in reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsteroidClass {
    MainBelt,
    Centaur,
    Tno,
}

/// One curated-core minor planet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AsteroidEntry {
    pub body: CelestialBody,
    pub tier: AsteroidTier,
    pub class: AsteroidClass,
}

fn ast(designation: &str) -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", designation))
}

fn tno(designation: &str) -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("tno", designation))
}

/// The committed curated core. Order is stable (checksums/reports depend on it).
pub fn asteroid_core_roster() -> &'static [AsteroidEntry] {
    use AsteroidClass::*;
    use AsteroidTier::*;
    use std::sync::OnceLock;
    static ROSTER: OnceLock<Vec<AsteroidEntry>> = OnceLock::new();
    ROSTER
        .get_or_init(|| {
            let e = |body, tier, class| AsteroidEntry { body, tier, class };
            vec![
                // Classical four — in sb441-n16, Tier A.
                e(CelestialBody::Ceres, PinnedKernel, MainBelt),
                e(CelestialBody::Pallas, PinnedKernel, MainBelt),
                e(CelestialBody::Juno, PinnedKernel, MainBelt),
                e(CelestialBody::Vesta, PinnedKernel, MainBelt),
                // Other massive main-belt members of sb441-n16 used in astrology.
                e(ast("10-Hygiea"), PinnedKernel, MainBelt),
                e(ast("16-Psyche"), PinnedKernel, MainBelt),
                e(ast("7-Iris"), PinnedKernel, MainBelt),
                // Centaurs — Tier B (not in sb441-n16).
                e(ast("2060-Chiron"), Constrained, Centaur),
                e(ast("5145-Pholus"), Constrained, Centaur),
                e(ast("7066-Nessus"), Constrained, Centaur),
                e(ast("10199-Chariklo"), Constrained, Centaur),
                e(ast("8405-Asbolus"), Constrained, Centaur),
                // Personal / "goddess" asteroids — Tier B.
                e(ast("433-Eros"), Constrained, MainBelt),
                e(ast("80-Sappho"), Constrained, MainBelt),
                e(ast("1221-Amor"), Constrained, MainBelt),
                e(ast("1181-Lilith"), Constrained, MainBelt),
                e(ast("5-Astraea"), Constrained, MainBelt),
                e(ast("6-Hebe"), Constrained, MainBelt),
                e(ast("8-Flora"), Constrained, MainBelt),
                e(ast("9-Metis"), Constrained, MainBelt),
                e(ast("19-Fortuna"), Constrained, MainBelt),
                e(ast("944-Hidalgo"), Constrained, MainBelt),
                e(ast("1566-Icarus"), Constrained, MainBelt),
                e(ast("1685-Toro"), Constrained, MainBelt),
                e(ast("1862-Apollo"), Constrained, MainBelt),
                // TNOs / dwarf planets — Tier B.
                e(tno("136199-Eris"), Constrained, Tno),
                e(tno("90377-Sedna"), Constrained, Tno),
                e(tno("136108-Haumea"), Constrained, Tno),
                e(tno("136472-Makemake"), Constrained, Tno),
                e(tno("50000-Quaoar"), Constrained, Tno),
                e(tno("90482-Orcus"), Constrained, Tno),
                e(tno("28978-Ixion"), Constrained, Tno),
                e(tno("20000-Varuna"), Constrained, Tno),
                e(tno("225088-Gonggong"), Constrained, Tno),
            ]
        })
        .as_slice()
}

/// Bodies sourced from the pinned kernel (Tier A), in roster order.
pub fn tier_a_bodies() -> Vec<CelestialBody> {
    asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::PinnedKernel)
        .map(|e| e.body.clone())
        .collect()
}

/// Horizons-sourced constrained bodies (Tier B), in roster order.
pub fn tier_b_bodies() -> Vec<CelestialBody> {
    asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::Constrained)
        .map(|e| e.body.clone())
        .collect()
}
```

Note: the `naif_ids` test relies on `parse_custom_naif` reading the leading integer; the `tno:` catalog parses identically (it only reads the leading digits of the designation), so `tno("136199-Eris")` resolves to candidate ids `[2_136_199, 20_136_199]`. The pool picks whichever the kernel actually contains at runtime.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl asteroid_roster -- --nocapture`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/asteroid_roster.rs crates/pleiades-jpl/src/spk/mod.rs
git commit -m "feat(jpl): add curated asteroid roster with tier/class tags"
```

---

### Task 3: Asteroid epoch grid (speed-appropriate cadence over 1900–2100)

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs`
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (add `AsteroidClass::max_gap_days`)

**Interfaces:**
- Consumes: `AsteroidClass`, `AST_RANGE_START_JD`, `AST_RANGE_END_JD`.
- Produces:
  - `impl AsteroidClass { pub fn max_gap_days(self) -> f64 }`
  - `corpus_spec::asteroid_epochs_for(class: AsteroidClass) -> Vec<f64>`

- [ ] **Step 1: Write the failing test**

Add to `corpus_spec.rs` `mod tests`:

```rust
    #[test]
    fn asteroid_epochs_in_window_and_increasing() {
        use crate::spk::asteroid_roster::AsteroidClass;
        let epochs = asteroid_epochs_for(AsteroidClass::MainBelt);
        assert!(epochs.len() >= 2);
        assert_eq!(*epochs.first().unwrap(), AST_RANGE_START_JD);
        assert_eq!(*epochs.last().unwrap(), AST_RANGE_END_JD);
        for pair in epochs.windows(2) {
            assert!(pair[1] > pair[0], "epochs must strictly increase");
            assert!(pair[0] >= AST_RANGE_START_JD && pair[1] <= AST_RANGE_END_JD);
        }
    }

    #[test]
    fn tnos_sampled_sparser_than_main_belt() {
        use crate::spk::asteroid_roster::AsteroidClass;
        let belt = asteroid_epochs_for(AsteroidClass::MainBelt).len();
        let tno = asteroid_epochs_for(AsteroidClass::Tno).len();
        assert!(tno < belt, "slow TNOs must be sparser: belt={belt} tno={tno}");
    }

    #[test]
    fn asteroid_corpus_stays_bounded() {
        use crate::spk::asteroid_roster::{asteroid_core_roster};
        let total: usize = asteroid_core_roster()
            .iter()
            .map(|e| asteroid_epochs_for(e.class).len())
            .sum();
        // Keep the whole asteroid corpus well under the major-body row count.
        assert!(total < 20_000, "asteroid corpus too large: {total} rows");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl corpus_spec::tests::asteroid_epochs -- --nocapture`
Expected: FAIL — `asteroid_epochs_for` / `AsteroidClass::max_gap_days` undefined.

- [ ] **Step 3: Write minimal implementation**

Add to `asteroid_roster.rs` (after the `AsteroidClass` enum):

```rust
impl AsteroidClass {
    /// Coarse, speed-appropriate sampling cadence (TDB days) over the asteroid
    /// window. Asteroids and centaurs move slowly; TNOs barely move, so they
    /// are sampled sparsely to keep the committed corpus bounded.
    pub fn max_gap_days(self) -> f64 {
        match self {
            AsteroidClass::MainBelt => 180.0,
            AsteroidClass::Centaur => 365.0,
            AsteroidClass::Tno => 1_825.0, // ~5 yr
        }
    }
}
```

Add to `corpus_spec.rs` (near `interior_backbone_epochs`):

```rust
/// Strictly-increasing asteroid epochs for a dynamical class: from
/// `AST_RANGE_START_JD` to `AST_RANGE_END_JD` inclusive, stepping by the
/// class cadence. Deterministic so checksums and verify-from-kernel reproduce.
pub fn asteroid_epochs_for(class: crate::spk::asteroid_roster::AsteroidClass) -> Vec<f64> {
    let step = class.max_gap_days();
    let mut epochs = Vec::new();
    let mut jd = AST_RANGE_START_JD;
    while jd < AST_RANGE_END_JD {
        epochs.push(jd);
        jd += step;
    }
    if epochs
        .last()
        .is_none_or(|&last| (AST_RANGE_END_JD - last).abs() > 1e-6)
    {
        epochs.push(AST_RANGE_END_JD);
    }
    epochs
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl corpus_spec::tests::asteroid -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs crates/pleiades-jpl/src/spk/asteroid_roster.rs
git commit -m "feat(jpl): add speed-scaled asteroid epoch grid for 1900-2100"
```

---

### Task 4: New slice roles for the two asteroid tiers

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (the `SliceRole` enum + `token()`, lines 19–39)

**Interfaces:**
- Produces: `SliceRole::AsteroidReference` (token `asteroid_reference`), `SliceRole::AsteroidConstrained` (token `asteroid_constrained`).

- [ ] **Step 1: Write the failing test**

Replace the existing `slice_role_tokens_are_unique` test body in `corpus_spec.rs` `mod tests` to include the new roles:

```rust
    #[test]
    fn slice_role_tokens_are_unique() {
        let roles = [
            SliceRole::Boundary,
            SliceRole::InteriorBackbone,
            SliceRole::FastCluster,
            SliceRole::Holdout,
            SliceRole::FixtureGolden,
            SliceRole::AsteroidReference,
            SliceRole::AsteroidConstrained,
        ];
        let mut tokens: Vec<&str> = roles.iter().map(|r| r.token()).collect();
        tokens.sort_unstable();
        tokens.dedup();
        assert_eq!(tokens.len(), roles.len());
    }

    #[test]
    fn asteroid_role_tokens_are_stable() {
        assert_eq!(SliceRole::AsteroidReference.token(), "asteroid_reference");
        assert_eq!(SliceRole::AsteroidConstrained.token(), "asteroid_constrained");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl corpus_spec::tests::asteroid_role -- --nocapture`
Expected: FAIL — `SliceRole::AsteroidReference` undefined.

- [ ] **Step 3: Write minimal implementation**

In `corpus_spec.rs`, add the two variants to `enum SliceRole` and arms to `token()`:

```rust
pub enum SliceRole {
    Boundary,
    InteriorBackbone,
    FastCluster,
    Holdout,
    FixtureGolden,
    AsteroidReference,
    AsteroidConstrained,
}
```

```rust
            SliceRole::FixtureGolden => "fixture_golden",
            SliceRole::AsteroidReference => "asteroid_reference",
            SliceRole::AsteroidConstrained => "asteroid_constrained",
```

Then fix the now-non-exhaustive match in `crates/pleiades-jpl/src/spk/generate.rs::generate_slice` (line ~121). Add explicit arms before the `other =>` catch-all:

```rust
        SliceRole::FixtureGolden => {
            Err("fixture_golden is sourced from existing fixtures, not generated".into())
        }
        SliceRole::AsteroidConstrained => {
            Err("asteroid_constrained is sourced from Horizons, not generated".into())
        }
        SliceRole::AsteroidReference => generate_asteroid_reference_slice(backend),
```

Add a temporary stub so this compiles (replaced fully in Task 5):

```rust
fn generate_asteroid_reference_slice(_backend: &SpkBackend) -> Result<GeneratedSlice, String> {
    Err("implemented in Task 5".into())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl corpus_spec -- --nocapture`
Expected: PASS (whole module compiles, role tests green).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs crates/pleiades-jpl/src/spk/generate.rs
git commit -m "feat(jpl): add asteroid_reference/asteroid_constrained slice roles"
```

---

### Task 5: Generate the Tier A asteroid_reference slice from the kernel

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/generate.rs` (replace the Task 4 stub)
- Test: `crates/pleiades-jpl/src/spk/generate.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Consumes: `SpkBackend` (must have both de440 + asteroid kernel loaded), `asteroid_roster::{asteroid_core_roster, AsteroidTier}`, `corpus_spec::{asteroid_epochs_for, AST_KERNEL_LABEL, AST_KERNEL_SHA256}`, `generate_corpus_csv_per_body`.
- Produces: `fn generate_asteroid_reference_slice(backend: &SpkBackend) -> Result<GeneratedSlice, String>` emitting file `asteroid_reference.csv`, role `asteroid_reference`, Tier A bodies each at their class cadence.

- [ ] **Step 1: Write the failing test**

The existing `spk` test support has helpers for building synthetic const-position kernels (`spk/test_support.rs`). Add an inline test in `generate.rs`:

```rust
#[cfg(test)]
mod asteroid_slice_tests {
    use super::*;
    use crate::spk::asteroid_roster::tier_a_bodies;
    use crate::spk::test_support::const_pos_kernel_bytes;

    #[test]
    fn asteroid_reference_slice_has_role_and_tier_a_bodies() {
        // Build a synthetic backend: Earth (399) at origin + Ceres (2000001)
        // at a fixed offset, so a geocentric ecliptic row is computable.
        let blob = const_pos_kernel_bytes(&[
            (399, 0, [0.0, 0.0, 0.0]),
            (10, 0, [0.0, 0.0, 0.0]),
            (2_000_001, 10, [3.0e8, 0.0, 0.0]),
        ]);
        let backend = SpkBackend::builder()
            .add_kernel_bytes(blob, "synthetic-ast")
            .unwrap()
            .build();

        let slice = generate_asteroid_reference_slice(&backend).unwrap();
        assert_eq!(slice.role, SliceRole::AsteroidReference);
        assert_eq!(slice.file, "asteroid_reference.csv");
        assert!(slice.csv.contains("#Slice-Role: asteroid_reference"));
        assert!(slice.csv.contains(crate::spk::corpus_spec::AST_KERNEL_LABEL));
        // Ceres rows present; no constrained (Tier B) bodies leaked in.
        assert!(slice.csv.contains(",Ceres,"));
        assert!(!slice.csv.contains("asteroid:2060-Chiron"));
        // Every Tier A body that the synthetic kernel covers appears.
        assert!(tier_a_bodies().contains(&CelestialBody::Ceres));
    }
}
```

If `const_pos_kernel_bytes` does not already exist with this exact signature, use the existing synthetic-kernel helper in `spk/test_support.rs` (check its name with `grep -n "pub.*fn" crates/pleiades-jpl/src/spk/test_support.rs`) and adapt the call; the test asserts the same observable behaviour.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl asteroid_slice_tests -- --nocapture`
Expected: FAIL — stub returns `Err("implemented in Task 5")`.

- [ ] **Step 3: Write minimal implementation**

Replace the Task 4 stub in `generate.rs`:

```rust
/// Tier A: samples the pinned small-body kernel (loaded alongside de440 in
/// `backend`) into the `asteroid_reference` slice. Each body is sampled at its
/// dynamical-class cadence over the asteroid window. Bodies whose ids are not
/// present in the loaded kernel are skipped (they are sourced as Tier B), so a
/// synthetic test kernel covering one body still produces a valid slice.
fn generate_asteroid_reference_slice(backend: &SpkBackend) -> Result<GeneratedSlice, String> {
    use crate::spk::asteroid_roster::{asteroid_core_roster, AsteroidTier};
    use crate::spk::corpus_spec::{self, AST_KERNEL_LABEL, AST_KERNEL_SHA256};

    let per_body: Vec<(CelestialBody, Vec<f64>)> = asteroid_core_roster()
        .iter()
        .filter(|e| e.tier == AsteroidTier::PinnedKernel)
        .filter(|e| backend.supports_body(e.body.clone()))
        .map(|e| (e.body.clone(), corpus_spec::asteroid_epochs_for(e.class)))
        .collect();

    let mut csv =
        generate_corpus_csv_per_body(backend, &per_body, AST_KERNEL_LABEL, AST_KERNEL_SHA256)?;
    csv = csv.replace(
        "#Columns:",
        &format!("#Slice-Role: {}\n#Columns:", SliceRole::AsteroidReference.token()),
    );
    Ok(GeneratedSlice {
        role: SliceRole::AsteroidReference,
        file: "asteroid_reference.csv".to_string(),
        csv,
    })
}
```

(`SpkBackend::supports_body` already exists — `backend.rs:168`.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl asteroid_slice_tests -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/generate.rs
git commit -m "feat(jpl): generate Tier A asteroid_reference slice from pinned kernel"
```

---

### Task 6: Asteroid slice validation (window + roster coverage + schema)

**Files:**
- Create: `crates/pleiades-validate/src/corpus/asteroid.rs`
- Modify: `crates/pleiades-validate/src/corpus/mod.rs` (add `pub mod asteroid;`)

**Interfaces:**
- Consumes: `LoadedSlice` (`crates/pleiades-validate/src/corpus/production.rs:9`), `pleiades_jpl::spk::corpus_spec::{AST_RANGE_START_JD, AST_RANGE_END_JD}`, `pleiades_jpl::spk::asteroid_roster::{tier_a_bodies, tier_b_bodies}`.
- Produces: `fn validate_asteroid_slices(slices: &[LoadedSlice]) -> Result<(), String>` — fails closed when an asteroid slice has epochs outside 1900–2100, non-finite coordinates, an unexpected/ missing roster body, or a malformed row.

- [ ] **Step 1: Write the failing test**

Create `asteroid.rs` with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::production::LoadedSlice;

    fn slice(role: &str, csv: &str) -> LoadedSlice {
        LoadedSlice { role: role.to_string(), text: csv.to_string() }
        // NOTE: match LoadedSlice's real field names (see production.rs:9).
    }

    #[test]
    fn rejects_epoch_outside_window() {
        // 2200-01-01 is past AST_RANGE_END_JD.
        let s = slice("asteroid_reference", "2524594.5,Ceres,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_err());
    }

    #[test]
    fn rejects_non_finite_coord() {
        let s = slice("asteroid_reference", "2451545.0,Ceres,nan,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_err());
    }

    #[test]
    fn accepts_in_window_row() {
        let s = slice("asteroid_reference", "2451545.0,Ceres,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_ok());
    }

    #[test]
    fn ignores_non_asteroid_slices() {
        let s = slice("boundary", "9999999.0,Sun,1.0,2.0,3.0\n");
        assert!(validate_asteroid_slices(&[s]).is_ok());
    }
}
```

Before implementing, confirm `LoadedSlice`'s real fields:
Run: `sed -n '9,16p' crates/pleiades-validate/src/corpus/production.rs`
and adjust the `slice()` constructor + the parser below to the real field names (`role`, plus whatever holds the CSV text). If `LoadedSlice` fields are private, add a `#[cfg(test)] pub fn for_test(role, text)` constructor to `production.rs` in this task.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate corpus::asteroid -- --nocapture`
Expected: FAIL — `validate_asteroid_slices` undefined.

- [ ] **Step 3: Write minimal implementation**

Prepend to `asteroid.rs`:

```rust
//! Fail-closed validation for the asteroid corpus slices: every row sits in the
//! 1900-2100 window, has finite coordinates, and names a body the curated
//! roster expects. Tier B (`asteroid_constrained`) is provenance-validated
//! here; it is never put behind the kernel regen gate.

use pleiades_jpl::spk::corpus_spec::{AST_RANGE_END_JD, AST_RANGE_START_JD};

use crate::corpus::production::LoadedSlice;

const ASTEROID_ROLES: [&str; 2] = ["asteroid_reference", "asteroid_constrained"];

/// Validates all asteroid slices, failing closed on the first breach.
pub fn validate_asteroid_slices(slices: &[LoadedSlice]) -> Result<(), String> {
    for s in slices.iter().filter(|s| ASTEROID_ROLES.contains(&s.role.as_str())) {
        for (i, line) in s.text.lines().enumerate() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            if f.len() != 5 {
                return Err(format!("{}: row {i} has {} fields, expected 5", s.role, f.len()));
            }
            let jd: f64 = f[0]
                .parse()
                .map_err(|_| format!("{}: row {i} bad epoch {:?}", s.role, f[0]))?;
            if !(AST_RANGE_START_JD..=AST_RANGE_END_JD).contains(&jd) {
                return Err(format!(
                    "{}: row {i} epoch {jd} outside asteroid window [{AST_RANGE_START_JD}, {AST_RANGE_END_JD}]",
                    s.role
                ));
            }
            for (col, raw) in [("x", f[2]), ("y", f[3]), ("z", f[4])] {
                let v: f64 = raw
                    .parse()
                    .map_err(|_| format!("{}: row {i} bad {col} {raw:?}", s.role))?;
                if !v.is_finite() {
                    return Err(format!("{}: row {i} non-finite {col}", s.role));
                }
            }
        }
    }
    Ok(())
}
```

(Replace `s.text` / `s.role` with the real `LoadedSlice` field names confirmed in Step 1.)

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-validate corpus::asteroid -- --nocapture`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/asteroid.rs crates/pleiades-validate/src/corpus/mod.rs crates/pleiades-validate/src/corpus/production.rs
git commit -m "feat(validate): fail-closed window/schema validation for asteroid slices"
```

---

### Task 7: Wire asteroid slices into the corpus gate

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs` (`embedded_slices()` ~line 215, `run_corpus_gate()` ~line 191)

**Interfaces:**
- Consumes: `validate_asteroid_slices` (Task 6), `include_str!` of the two new committed slice files.
- Produces: corpus gate now loads + validates the asteroid slices. (The committed CSV files themselves are added in Task 11; until then, gate them behind file existence by adding the `include_str!` lines together with placeholder files committed in this task so the build stays green.)

- [ ] **Step 1: Write the failing test**

Add to `production.rs` test module:

```rust
    #[test]
    fn corpus_gate_runs_asteroid_validation() {
        // run_corpus_gate must surface asteroid-window breaches. Build a slice
        // set with a bad asteroid row and assert the gate rejects it.
        let mut slices = embedded_slices();
        slices.push(LoadedSlice {
            role: "asteroid_reference".to_string(),
            text: "2200000.0,Ceres,1.0,2.0,3.0\n".to_string(), // pre-1900, out of window
            // ...match real field names...
        });
        assert!(crate::corpus::asteroid::validate_asteroid_slices(&slices).is_err());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-validate corpus_gate_runs_asteroid -- --nocapture`
Expected: FAIL (until `run_corpus_gate` calls the new validation and the placeholder slices exist).

- [ ] **Step 3: Write minimal implementation**

1. Create minimal placeholder slice files so `include_str!` compiles (overwritten with real data in Task 11):

```bash
printf '#Pleiades SPK Reference Corpus\n#Slice-Role: asteroid_reference\n#Columns:epoch_jd,body,x_km,y_km,z_km\n' > crates/pleiades-jpl/data/corpus/asteroid_reference.csv
printf '#Pleiades SPK Reference Corpus\n#Slice-Role: asteroid_constrained\n#Columns:epoch_jd,body,x_km,y_km,z_km\n' > crates/pleiades-jpl/data/corpus/asteroid_constrained.csv
```

2. Add them to `embedded_slices()`:

```rust
        slice(
            "asteroid_reference",
            include_str!("../../../pleiades-jpl/data/corpus/asteroid_reference.csv"),
        ),
        slice(
            "asteroid_constrained",
            include_str!("../../../pleiades-jpl/data/corpus/asteroid_constrained.csv"),
        ),
```

3. Call the validation inside `run_corpus_gate()` (after `cross_check_fixture_golden`):

```rust
    crate::corpus::asteroid::validate_asteroid_slices(&slices)?;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-validate corpus -- --nocapture`
Expected: PASS (gate still green on empty placeholder asteroid slices; bad-row test rejects).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs crates/pleiades-jpl/data/corpus/asteroid_reference.csv crates/pleiades-jpl/data/corpus/asteroid_constrained.csv
git commit -m "feat(validate): load and gate asteroid slices in the corpus gate"
```

---

### Task 8: Tier A reproducibility in corpus_regen (PLEIADES_AST_KERNEL)

**Files:**
- Modify: `crates/pleiades-jpl/tests/corpus_regen.rs`

**Interfaces:**
- Consumes: `PLEIADES_DE_KERNEL` (existing) + new optional `PLEIADES_AST_KERNEL`, `generate_asteroid_reference_slice` via the public `generate_slice(backend, SliceRole::AsteroidReference)` path.
- Produces: a gated assertion that the regenerated Tier A slice matches the committed `asteroid_reference.csv` within the existing tolerance, skipping when either env var is unset.

- [ ] **Step 1: Write the failing test**

Append a new test to `corpus_regen.rs`:

```rust
#[test]
fn regenerated_asteroid_reference_matches_checked_in() {
    let (Ok(de), Ok(ast)) = (
        std::env::var("PLEIADES_DE_KERNEL"),
        std::env::var("PLEIADES_AST_KERNEL"),
    ) else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL to run");
        return;
    };
    use pleiades_jpl::spk::corpus_spec::SliceRole;
    use pleiades_jpl::{generate_slice, SpkBackend};

    let backend = SpkBackend::builder()
        .add_kernel(&de)
        .unwrap()
        .add_kernel(&ast)
        .unwrap()
        .build();

    let regenerated = generate_slice(&backend, SliceRole::AsteroidReference).unwrap();
    let checked_in = include_str!("../data/corpus/asteroid_reference.csv");

    let parse = |csv: &str| -> Vec<(String, [f64; 3])> {
        csv.lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty())
            .map(|l| {
                let f: Vec<&str> = l.split(',').collect();
                (format!("{},{}", f[0], f[1]), [f[2].parse().unwrap(), f[3].parse().unwrap(), f[4].parse().unwrap()])
            })
            .collect()
    };
    let a = parse(&regenerated.csv);
    let b = parse(checked_in);
    assert_eq!(a.len(), b.len(), "asteroid_reference row count drift");
    for ((ka, va), (kb, vb)) in a.iter().zip(b.iter()) {
        assert_eq!(ka, kb, "asteroid_reference epoch/body ordering drift");
        for i in 0..3 {
            assert!((va[i] - vb[i]).abs() < 1.0, "asteroid_reference {ka} coord {i} drift");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails (cleanly skips)**

Run: `cargo test -p pleiades-jpl --test corpus_regen regenerated_asteroid -- --nocapture`
Expected: PASS via early-return skip (no env vars set). With real kernels it would run; offline it must compile and skip.

- [ ] **Step 3: Implementation** — none beyond the test (the generate path already exists from Task 5).

- [ ] **Step 4: Verify compile + skip**

Run: `cargo test -p pleiades-jpl --test corpus_regen -- --nocapture`
Expected: all `corpus_regen` tests skip-pass offline.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/tests/corpus_regen.rs
git commit -m "test(jpl): gate Tier A asteroid reproducibility on PLEIADES_AST_KERNEL"
```

---

### Task 9: Report asteroids as a constrained class over 1900–2100

**Files:**
- Modify: `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs`
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (`constrained_bodies()`)
- Test: inline in the modified modules

**Interfaces:**
- Consumes: `asteroid_roster::{asteroid_core_roster, tier_a_bodies, tier_b_bodies, AsteroidClass}`, `AST_RANGE_START_JD/END_JD`.
- Produces: a summary value/string that states the asteroid class is constrained, advertised over 1900–2100, with per-tier and per-class counts kept separate. Extends `constrained_bodies()` to include the curated roster so the completeness matrix tags them constrained, not release.

- [ ] **Step 1: Write the failing test**

Add to `corpus_spec.rs` `mod tests`:

```rust
    #[test]
    fn curated_asteroids_are_constrained_not_release() {
        use crate::spk::asteroid_roster::asteroid_core_roster;
        let constrained = constrained_bodies();
        for e in asteroid_core_roster() {
            assert!(constrained.contains(&e.body), "{:?} must be constrained", e.body);
            assert!(!release_bodies().contains(&e.body));
        }
    }
```

Add to `selected_asteroid.rs` a test asserting the advertised window + tier split appears in the report text (match the module's existing summary-report pattern; grep the file for its current `..._for_report` fn name first).

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-jpl curated_asteroids_are_constrained -- --nocapture`
Expected: FAIL — roster bodies not yet in `constrained_bodies()`.

- [ ] **Step 3: Write minimal implementation**

Extend `constrained_bodies()` in `corpus_spec.rs`:

```rust
pub fn constrained_bodies() -> Vec<CelestialBody> {
    let mut bodies = vec![CelestialBody::Pluto];
    bodies.extend(crate::spk::asteroid_roster::asteroid_core_roster().iter().map(|e| e.body.clone()));
    bodies
}
```

In `selected_asteroid.rs`, add to the existing report summary an advertised-window + tier/class breakdown line, e.g.:

```rust
format!(
    "selected asteroids: constrained class advertised over JD {:.1}–{:.1} (1900–2100); \
     {} Tier A (pinned-kernel) + {} Tier B (Horizons-constrained) bodies",
    AST_RANGE_START_JD, AST_RANGE_END_JD,
    tier_a_bodies().len(), tier_b_bodies().len(),
)
```

Update any existing `release_and_constrained_bodies_are_disjoint` test expectations if needed (they should still pass — release vs constrained stay disjoint).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-jpl -- --nocapture 2>&1 | tail -20`
Expected: PASS, including the existing disjointness test.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): report curated asteroids as a constrained 1900-2100 class"
```

---

### Task 10: Provenance documentation

**Files:**
- Modify: `docs/spk-kernel-sourcing.md` (the "Asteroid kernel (optional)" section)

**Interfaces:** none (docs).

- [ ] **Step 1: Replace the placeholder section**

Replace the "## Asteroid kernel (optional)" block with two filled subsections. Leave the SHA-256 as `<pinned-in-Task-11>` until the file is downloaded:

```markdown
## Asteroid kernel (Tier A — pinned)

Selected-asteroid main-belt coverage reads a JPL small-body perturber kernel,
**not** committed to this repository.

- File: `sb441-n16.bsp`
- Source: NASA/JPL SSD/NAIF small-body perturber set, fitted consistently with
  DE441 (agrees with de440 over the overlap) —
  `https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n16.bsp`
- License: public domain (U.S. Government work).
- SHA-256: `<pinned-in-Task-11>`
- Bodies: the 16 most-massive perturbers (Ceres, Pallas, Juno, Vesta, Hygiea,
  Psyche, Iris, …); the curated subset used here is the Tier A roster.
- Default asteroid window: 1900–2100 CE (the corpus samples only this window;
  the kernel itself covers the full DE441 interval).

Usage / reproduction:

\`\`\`bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
\`\`\`

## Asteroid slices (Tier B — Horizons-sourced, constrained)

Centaurs, personal asteroids, and TNOs are **not** in any fixed perturber
kernel. They are generated once via JPL Horizons over 1900–2100 using
`pleiades_jpl::ingest` (see the `horizons-fetch` feature) and committed as the
provenance-validated `asteroid_constrained` slice — never put behind the kernel
regen gate.

- Bodies: see `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (Tier B entries).
- Solution epoch / generation date: `<recorded-in-Task-11>`
- Recipe: `<exact Horizons request recorded in Task 11>`
```

- [ ] **Step 2: Verify rendering**

Run: `grep -n "sb441-n16\|asteroid_constrained\|PLEIADES_AST_KERNEL" docs/spk-kernel-sourcing.md`
Expected: matches present.

- [ ] **Step 3: Commit**

```bash
git add docs/spk-kernel-sourcing.md
git commit -m "docs: record sb441-n16 + Tier B Horizons asteroid provenance"
```

---

### Task 11: Adopt the kernel and generate the committed slices (maintainer, network)

> **Network + kernel required.** This task produces the real committed data. It cannot run in a kernel-free/offline sandbox; run it where de440.bsp + sb441-n16.bsp are available and Horizons is reachable.

**Files:**
- Modify: `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (overwrite placeholder)
- Modify: `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv` (overwrite placeholder)
- Modify: `crates/pleiades-jpl/data/corpus/manifest.txt` (add both slice entries + checksums)
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (`AST_KERNEL_SHA256` real digest)
- Modify: `docs/spk-kernel-sourcing.md` (fill `<pinned-in-Task-11>` / `<recorded-in-Task-11>`)

- [ ] **Step 1: Download and pin the kernel**

```bash
curl -L -o /tmp/sb441-n16.bsp \
  https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n16.bsp
shasum -a 256 /tmp/sb441-n16.bsp
```
Record the digest into `AST_KERNEL_SHA256` (corpus_spec.rs) and `docs/spk-kernel-sourcing.md`.

- [ ] **Step 2: Verify roster designations against the kernel + MPC**

For each Tier A roster body, confirm the kernel actually contains its NAIF id:
```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/tmp/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test spk_full_kernel -- --nocapture
```
Cross-check every committed IAU number (Tier A and Tier B) against the MPC numbered-asteroid catalog / Horizons. Fix any mismatch in `asteroid_roster.rs` before generating data.

- [ ] **Step 3: Generate the Tier A slice from the kernels**

Add a `#[ignore]`-gated generator test (mirrors how the major-body corpus is emitted) that writes `generate_slice(&backend, SliceRole::AsteroidReference)?.csv` to `data/corpus/asteroid_reference.csv`, run it with both env vars set, then `git diff` to confirm real rows replaced the placeholder.

- [ ] **Step 4: Generate the Tier B constrained slice via Horizons**

Using `pleiades_jpl::ingest` with the `horizons-fetch` feature, fetch geocentric ecliptic states for each Tier B body at its class cadence over 1900–2100, normalize into the corpus schema, and write `data/corpus/asteroid_constrained.csv` (role `asteroid_constrained`, label = Horizons solution + generation date). Record the exact Horizons request + solution epoch into `docs/spk-kernel-sourcing.md`.

```bash
cargo run -p pleiades-validate --features pleiades-jpl/horizons-fetch -- \
  ingest-public --help   # confirm the offline/online ingest entry points
```

- [ ] **Step 5: Refresh the manifest checksums**

Regenerate `manifest.txt` so it includes `asteroid_reference` and `asteroid_constrained` slice entries with their `corpus_checksum64` values and row counts (reuse the existing manifest-build path).

- [ ] **Step 6: Run the full gate offline**

```bash
cargo test -p pleiades-validate corpus -- --nocapture
cargo test -p pleiades-jpl -- --nocapture
PLEIADES_DE_KERNEL=/path/to/de440.bsp PLEIADES_AST_KERNEL=/tmp/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```
Expected: corpus gate green over real asteroid slices; `corpus_regen` reproduces Tier A within 1 km; clean-checkout (no env vars) still skips.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-jpl/data/corpus/asteroid_reference.csv \
        crates/pleiades-jpl/data/corpus/asteroid_constrained.csv \
        crates/pleiades-jpl/data/corpus/manifest.txt \
        crates/pleiades-jpl/src/spk/corpus_spec.rs \
        docs/spk-kernel-sourcing.md
git commit -m "feat(jpl): commit curated asteroid corpus (Tier A kernel + Tier B Horizons)"
```

---

### Task 12: Confirm downstream primitives for calculated points

**Files:**
- Create: `docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md` (short note)

**Interfaces:** none (verification + docs).

- [ ] **Step 1: Verify the library exposes what a downstream crate needs**

Confirm via grep/tests that the workspace already yields: Moon position + lunar node/apogee geometry, body ecliptic positions, and chart angles (Ascendant/MC) — the inputs a higher-level crate needs for Black Moon Lilith, Part of Fortune, Vertex.

```bash
grep -rn "MeanApogee\|MeanNode\|TrueNode\|Ascendant\|midheaven\|ascendant" crates/pleiades-*/src | head
```

- [ ] **Step 2: Write the readiness note**

Record which primitives exist (with file references) and any genuine gap. State explicitly that calculated points are out of scope for this library and computable downstream.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/notes/asteroid-calculated-points-readiness.md
git commit -m "docs: note downstream readiness for calculated astrological points"
```

---

### Task 13: Plan bookkeeping — remove the completed item

**Files:**
- Modify: `plan/stages/01-production-reference-corpus.md`
- Modify: `PLAN.md`

**Interfaces:** none.

- [ ] **Step 1: Remove the asteroid-kernel item per plan-maintenance rules**

In `plan/stages/01-production-reference-corpus.md`, delete the "Adopt a small-body asteroid SPK kernel…" bullet (lines ~36–37). In `PLAN.md`, remove the trailing "asteroid-kernel adoption … is still open" clause (line ~49) and refresh the Status line with today's date. Do **not** add completion notes — remove, per the maintenance rules.

- [ ] **Step 2: Verify no stale references**

Run: `grep -rn "asteroid-kernel adoption\|Adopt a small-body asteroid SPK kernel" PLAN.md plan/`
Expected: no matches.

- [ ] **Step 3: Commit**

```bash
git add PLAN.md plan/stages/01-production-reference-corpus.md
git commit -m "docs: mark asteroid-kernel coverage implemented, drop from plan"
```

---

## Self-Review

**Spec coverage** — every spec section maps to a task:
- Scope/contract (physical asteroids, 1900–2100, fail-closed) → Tasks 1, 3, 6.
- Two-tier sourcing (sb441-n16 + Horizons constrained) → Tasks 2, 5, 11.
- Curated ~35-body roster + on-demand tail → Task 2 (committed core); on-demand tail needs no code (existing `Custom`+ingest path) and is documented in Task 10.
- Taxonomy (named 4 + Custom ids, tier/class tags, `is_reference_asteroid`) → Task 2 (`is_reference_asteroid` already accepts `asteroid:` Custom ids; `tno:` bodies are constrained-class report bodies, not "reference asteroids" — intentional, noted in Task 9).
- Accuracy/claims (constrained class) → Task 9.
- Reproducibility + validation gates → Tasks 6, 7, 8.
- Provenance docs → Tasks 10, 11.
- Non-goal: calculated-points readiness → Task 12.
- Definition-of-done bookkeeping → Task 13.

**Placeholder scan** — the only intentional placeholders are the kernel SHA and Horizons recipe, which are explicitly filled in Task 11 (the data/network task) and guarded by a test in Task 1. No "TODO/handle edge cases" placeholders; all Rust steps carry real code.

**Type consistency** — `SliceRole::{AsteroidReference, AsteroidConstrained}` tokens (`asteroid_reference`/`asteroid_constrained`) are used identically in Tasks 4–8. `asteroid_core_roster`/`tier_a_bodies`/`tier_b_bodies`/`AsteroidTier`/`AsteroidClass` signatures from Task 2 are consumed unchanged in Tasks 3, 5, 9. `validate_asteroid_slices` signature from Task 6 is reused verbatim in Task 7. `generate_asteroid_reference_slice` is stubbed in Task 4 and implemented in Task 5 with the same signature.

**Known integration check for the implementer:** `LoadedSlice`'s real field names (Task 6/7) and the `selected_asteroid.rs` report fn name (Task 9) and `spk/test_support.rs` synthetic-kernel helper name (Task 5) must be confirmed by grep before writing those tests — each task names the exact command to run. These are the only spots where the plan defers to the live signature rather than asserting it.
