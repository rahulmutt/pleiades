# Asteroid sb441-n16 Release-Grade Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

> **Outcome deviation (recorded post-implementation):** Task 1's kernel-membership probe found that **6-Hebe is NOT in `sb441-n16.bsp`** (the kernel holds exactly the 16 most-massive perturbers; Hebe is not among them). Per the two-gate policy below, Hebe was therefore **dropped** (it stays Tier-B/constrained), and Task 4 (remove Hebe from the constrained slice) was a no-op. The bodies actually promoted to Tier-A were **15-Eunomia and 65-Cybele** (both kernel-confirmed and cited). The Hebe-centric phrasing in the Goal and Tasks 2–4 below is the pre-probe expectation; the shipped outcome and the design spec reflect the corrected set.

**Goal:** Promote a small, astrologically-relevant set of `sb441-n16` perturber-kernel asteroids (reclassify Hebe; add Cybele and any other confirmed n16 goddess) from constrained to release-grade, reusing the existing Tier-A pinned-kernel machinery.

**Architecture:** The per-backend public claim tier is driven entirely by `AsteroidTier` in the curated roster (`asteroid_roster.rs`); flipping a roster entry to `PinnedKernel` makes `spk_body_claims` emit `ReleaseGrade/High/CorpusValidated{"sb441-n16"}` automatically. Promotion therefore means: (1) confirm a candidate is in the pinned kernel, (2) retag it in the roster, (3) regenerate its committed reference rows from the kernel, (4) realign every count/claim surface. No new accuracy ceiling, no new gate, no public enum change.

**Tech Stack:** Rust (workspace `pleiades-*` crates), pure-Rust SPK reader, FNV-1a content checksums, env-gated kernel regeneration tests.

## Global Constraints

Every task implicitly includes these. Values copied verbatim from the spec and code.

- **Pure Rust only; no required C/C++ dependencies** (workspace rule, `SPEC.md`).
- **Kernel prerequisite (Tasks 1, 3, and the kernel-gated checks in Task 7):** the 645 MB `sb441-n16.bsp` (SHA-256 `AST_KERNEL_SHA256`, `crates/pleiades-jpl/src/spk/corpus_spec.rs:83`) and a `de440.bsp` are **uncommitted**. They are supplied via `PLEIADES_AST_KERNEL` / `PLEIADES_DE_KERNEL`. Kernel-free checkouts skip those steps via early return — never fabricate coordinate rows or checksums.
- **Two-gate promotion.** A body is promoted only if BOTH hold: (1) **confirmed present in `sb441-n16.bsp`** by reading the actual kernel (Task 1); (2) **documented astrological usage** with a *cited* source in `docs/spk-kernel-sourcing.md` (Task 6). A body failing either gate is dropped from the slice — not committed as a known gap.
- **No designation from memory.** Every IAU designation/NAIF id is verified against the actual kernel and MPC/Horizons at implementation time.
- **Two independent "release/constrained" axes — do NOT conflate them.**
  - *Corpus-tolerance class* (`corpus_spec.rs`): asteroids live in `constrained_asteroid_bodies()` and use the loose `BodyClass::Asteroid` ceiling (30″ lon/lat, 5e6 km, `crates/pleiades-data/src/thresholds.rs:75`). They must **never** be added to `release_bodies()` or `constrained_bodies()` (that set must stay Pluto-only — enforced by `corpus_spec.rs:396` `curated_asteroids_are_constrained_class_not_release_and_not_in_generation_set`).
  - *Per-backend public claim tier* (`asteroid_roster.rs::spk_body_claims`): Tier-A → `ReleaseGrade`. This is the ONLY place the release-grade claim is expressed for asteroids.
  - The 7 current Tier-A asteroids already sit in both states at once. Promotion changes only the second axis.
- **No `CelestialBody` enum change.** New bodies use `CelestialBody::Custom(CustomBodyId::new("asteroid", "NN-Name"))` via the `ast(...)` helper, exactly like `7-Iris`.
- **MainBelt cadence is 180 TDB days** (`AsteroidClass::max_gap_days`, `asteroid_roster.rs:32`), giving **407 rows per body** over the 1900–2100 window (matches each existing Tier-A body).
- **Baselines to update from (current committed values):**
  - `asteroid_reference.csv`: 7 bodies × 407 = **2849 rows**, manifest checksum **3764151475794306169** (`manifest.txt:9`).
  - `asteroid_constrained.csv`: **6652 rows**, manifest checksum **1743170913221086087** (`manifest.txt:10`); `asteroid:6-Hebe` has **406 rows** in it.

---

### Task 1: Confirm sb441-n16 membership of the candidate bodies (kernel-gated discovery)

This task decides the final promoted set. It is investigation that produces a recorded decision; no source file changes are committed here.

**Files:**
- Read-only: `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (the `ast()` helper, roster)
- Read-only: `crates/pleiades-jpl/src/spk/chain.rs` (`naif_ids`, `parse_custom_naif`)
- Output: a recorded confirmed-body list (pasted into the Task 6 provenance edit and used by Task 2)

**Interfaces:**
- Consumes: `pleiades_jpl::SpkBackend::builder().add_kernel(path)` → builder; `backend.supports_body(body: CelestialBody) -> bool`; `pleiades_jpl::spk::asteroid_roster` `ast(designation: &str) -> CelestialBody`.
- Produces: `CONFIRMED_NEW_BODIES` — the subset of `{asteroid:6-Hebe, asteroid:65-Cybele, asteroid:15-Eunomia, asteroid:29-Amphitrite}` for which `supports_body` returns `true`. Task 2 retags exactly this set.

- [ ] **Step 1: Add a temporary, ignored membership-probe test**

Create `crates/pleiades-jpl/tests/n16_membership_probe.rs`:

```rust
//! TEMPORARY probe (Task 1): with PLEIADES_AST_KERNEL + PLEIADES_DE_KERNEL set,
//! prints whether each promotion candidate resolves in the pinned kernel.
//! Delete this file at the end of Task 1.

use pleiades_backend::CelestialBody;
use pleiades_types::CustomBodyId;

fn ast(designation: &str) -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", designation))
}

#[test]
#[ignore = "kernel-gated probe; run explicitly with PLEIADES_AST_KERNEL set"]
fn probe_n16_membership() {
    let de = std::env::var("PLEIADES_DE_KERNEL").expect("set PLEIADES_DE_KERNEL");
    let ast_k = std::env::var("PLEIADES_AST_KERNEL").expect("set PLEIADES_AST_KERNEL");
    let backend = pleiades_jpl::SpkBackend::builder()
        .add_kernel(&de)
        .expect("load de440")
        .add_kernel(&ast_k)
        .expect("load sb441-n16")
        .build();

    for designation in ["6-Hebe", "65-Cybele", "15-Eunomia", "29-Amphitrite"] {
        let body = ast(designation);
        println!(
            "n16_candidate {designation:<14} supported_by_kernel={}",
            backend.supports_body(body)
        );
    }
}
```

- [ ] **Step 2: Run the probe with the kernels**

Run:
```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test n16_membership_probe -- --ignored --nocapture
```
Expected: one `n16_candidate … supported_by_kernel=true|false` line per candidate. Record which are `true`. `6-Hebe` is expected `true` (it is one of the most-massive perturbers); others are confirmed empirically — **do not assume**.

- [ ] **Step 3: Record the confirmed set**

Write down `CONFIRMED_NEW_BODIES` = every candidate that printed `true`. This is the authoritative list for Tasks 2/3/6. Any candidate that printed `false` is dropped now and never mentioned again.

- [ ] **Step 4: Delete the temporary probe**

Run:
```bash
rm crates/pleiades-jpl/tests/n16_membership_probe.rs
```

- [ ] **Step 5: Commit (removal only — probe leaves no trace)**

```bash
git add -A
git commit -m "chore(jpl): confirm sb441-n16 membership of asteroid promotion candidates (no-op probe)"
```
(If the probe file was never staged, this commit is empty — skip it. The recorded `CONFIRMED_NEW_BODIES` is the real deliverable.)

---

### Task 2: Retag the roster — reclassify Hebe and add confirmed bodies (kernel-free)

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (roster `vec![...]`, lines ~76–106; tests at ~176–234)

**Interfaces:**
- Consumes: `CONFIRMED_NEW_BODIES` from Task 1; `AsteroidTier::PinnedKernel`, `AsteroidClass::MainBelt`, the `e(body, tier, class)` closure, `ast(designation)`.
- Produces: a roster where `tier_a_bodies()` includes Hebe + each confirmed new body; `spk_body_claims` emits `ReleaseGrade` for them with no further code change.

- [ ] **Step 1: Write the failing roster tests**

Add to the `tests` module in `asteroid_roster.rs` (after `classical_four_are_tier_a_main_belt`). Include one assertion per confirmed body; the block below assumes Hebe + Cybele were confirmed in Task 1 — **edit the `confirmed` array to match `CONFIRMED_NEW_BODIES` exactly** (drop Cybele if it was `false`; add Eunomia/Amphitrite if they were `true`):

```rust
#[test]
fn hebe_is_tier_a_main_belt_after_promotion() {
    let e = asteroid_core_roster()
        .iter()
        .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == "6-Hebe"))
        .expect("Hebe present");
    assert_eq!(e.tier, AsteroidTier::PinnedKernel);
    assert_eq!(e.class, AsteroidClass::MainBelt);
}

#[test]
fn promoted_goddesses_are_tier_a_main_belt() {
    // EDIT THIS LIST to equal CONFIRMED_NEW_BODIES (minus Hebe), verbatim.
    let confirmed = ["65-Cybele"];
    for designation in confirmed {
        let e = asteroid_core_roster()
            .iter()
            .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == designation))
            .unwrap_or_else(|| panic!("{designation} present"));
        assert_eq!(e.tier, AsteroidTier::PinnedKernel, "{designation} tier");
        assert_eq!(e.class, AsteroidClass::MainBelt, "{designation} class");
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p pleiades-jpl --lib asteroid_roster -- hebe_is_tier_a promoted_goddesses`
Expected: FAIL — `hebe_is_tier_a_main_belt_after_promotion` fails on `assert_eq!(e.tier, PinnedKernel)` (Hebe is currently `Constrained`); `promoted_goddesses_are_tier_a_main_belt` panics with `65-Cybele present` (not in roster yet).

- [ ] **Step 3: Edit the roster**

In `asteroid_core_roster()`:

(a) Move Hebe out of the Tier-B block and into the Tier-A main-belt block. Delete this line from the personal/goddess group:
```rust
                e(ast("6-Hebe"), Constrained, MainBelt),
```
and add it next to the other promoted main-belt members (after `7-Iris`):
```rust
                // Other massive main-belt members of sb441-n16 used in astrology.
                e(ast("10-Hygiea"), PinnedKernel, MainBelt),
                e(ast("16-Psyche"), PinnedKernel, MainBelt),
                e(ast("7-Iris"), PinnedKernel, MainBelt),
                e(ast("6-Hebe"), PinnedKernel, MainBelt),
                e(ast("65-Cybele"), PinnedKernel, MainBelt),
```
The `65-Cybele` line (and any other confirmed goddess) is **new**. Add exactly the bodies in `CONFIRMED_NEW_BODIES`; add nothing that was `false` in Task 1. Roster order feeds checksums and report order — append promoted bodies at the end of the Tier-A main-belt group, in ascending IAU-number order, before the centaur block.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p pleiades-jpl --lib asteroid_roster`
Expected: PASS — including `roster_has_curated_core` (count is now 34 + N new goddesses; with Hebe reclassified and ≤3 goddesses added the count is 35–37, inside the `>= 33 && <= 38` bound) and `tiers_are_disjoint_and_cover_roster`. If you added enough bodies to exceed 38, widen the upper bound in `roster_has_curated_core` to the new count and note it.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/asteroid_roster.rs
git commit -m "feat(jpl): promote Hebe + confirmed n16 goddesses to Tier-A roster (slice 1)"
```

---

### Task 3: Regenerate the Tier-A reference corpus from the kernel + update manifest (kernel-gated)

**Files:**
- Modify (generated): `crates/pleiades-jpl/data/corpus/asteroid_reference.csv`
- Modify: `crates/pleiades-jpl/data/corpus/manifest.txt:9`

**Interfaces:**
- Consumes: the retagged roster (Task 2); `regenerate-asteroid-corpus` binary; both kernels.
- Produces: `asteroid_reference.csv` with `(7 + 1 Hebe + N goddesses) × 407` rows; the matching `slice asteroid_reference …` manifest line.

- [ ] **Step 1: Regenerate the slice from the kernels**

Run:
```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
```
Expected stderr: a `Tier A <body> supported_by_kernel=true` line for **every** Tier-A body including Hebe and each promoted goddess (any `false` means the body is not really in the kernel — stop, remove it from the roster in Task 2, and re-run Task 1's reasoning). Expected stdout: a single line like
`slice asteroid_reference file=asteroid_reference.csv role=asteroid_reference rows=<R> checksum=<C>` where `<R>` = `(8 + N) × 407` (e.g. 9 bodies → 3663 rows).

- [ ] **Step 2: Update the manifest reference line**

Replace `manifest.txt:9` with the exact line printed in Step 1 (new `rows=<R>` and `checksum=<C>`).

- [ ] **Step 3: Verify the corpus gate accepts the regenerated slice**

Run: `cargo test -p pleiades-validate corpus`
Expected: PASS — `validate_drift` recomputes `corpus_checksum64(asteroid_reference.csv)` and matches the manifest; `validate_asteroid_slices` confirms every new row is in-window, 5-column, finite.

- [ ] **Step 4: Verify regeneration is reproducible (regen gate)**

Run:
```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```
Expected: PASS — `regenerated_asteroid_reference_matches_checked_in` reproduces every committed row (including the promoted bodies) within 1 km, with no row-count or ordering drift.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/data/corpus/asteroid_reference.csv crates/pleiades-jpl/data/corpus/manifest.txt
git commit -m "feat(jpl): regenerate Tier-A asteroid reference corpus with promoted bodies (slice 1)"
```

---

### Task 4: Surgically remove Hebe from the Tier-B constrained slice (kernel-free, no Horizons re-fetch)

Re-running `regenerate-asteroid-constrained` would re-fetch every Tier-B body from Horizons (byte-unstable, churns the whole slice). Instead, delete only Hebe's rows and recompute the one checksum.

**Files:**
- Modify: `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv` (remove `asteroid:6-Hebe` rows)
- Modify: `crates/pleiades-jpl/data/corpus/manifest.txt:10`

**Interfaces:**
- Consumes: `validate_drift`'s mismatch message (reports the actual checksum); `corpus_checksum64`.
- Produces: `asteroid_constrained.csv` with `6652 − 406 = 6246` rows and no Hebe; matching manifest line.

- [ ] **Step 1: Delete Hebe's rows in place**

Run:
```bash
cd /workspace
grep -v '^[0-9.]*,asteroid:6-Hebe,' crates/pleiades-jpl/data/corpus/asteroid_constrained.csv \
  > /tmp/claude-1000/-workspace/a2b1ff4a-edfd-4a2b-bcad-a8d2e123b09b/scratchpad/ast_constrained_noHebe.csv \
  && mv /tmp/claude-1000/-workspace/a2b1ff4a-edfd-4a2b-bcad-a8d2e123b09b/scratchpad/ast_constrained_noHebe.csv \
     crates/pleiades-jpl/data/corpus/asteroid_constrained.csv
```
(The `^[0-9.]*,asteroid:6-Hebe,` anchor removes only Hebe data rows; the `#` header lines and other bodies are untouched.)

- [ ] **Step 2: Confirm the new row count**

Run: `grep -c 'asteroid:6-Hebe' crates/pleiades-jpl/data/corpus/asteroid_constrained.csv`
Expected: `0`.
Run: `grep -vc '^#' crates/pleiades-jpl/data/corpus/asteroid_constrained.csv`
Expected: `6246`.

- [ ] **Step 3: Obtain the new checksum via the failing drift gate**

Run: `cargo test -p pleiades-validate corpus -- --nocapture`
Expected: FAIL with `checksum drift for asteroid_constrained: manifest 1743170913221086087 != actual <NEW_CHECKSUM>`. Copy `<NEW_CHECKSUM>`.

- [ ] **Step 4: Update the manifest constrained line**

Replace `manifest.txt:10` with:
```
slice asteroid_constrained file=asteroid_constrained.csv role=asteroid_constrained rows=6246 checksum=<NEW_CHECKSUM>
```

- [ ] **Step 5: Verify the gate now passes**

Run: `cargo test -p pleiades-validate corpus`
Expected: PASS — checksum matches; Hebe no longer appears in the constrained slice; it appears once, in `asteroid_reference.csv` (from Task 3).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/data/corpus/asteroid_constrained.csv crates/pleiades-jpl/data/corpus/manifest.txt
git commit -m "refactor(jpl): drop Hebe from Tier-B constrained slice (promoted to Tier-A)"
```

---

### Task 5: Realign report counts and any drifted snapshots (kernel-free)

The reference-summary "source windows" evidence (`reference_asteroid`, `jpl_posture`) is driven by a **separate** sparse fixture set (Ceres, Pallas, Juno, Vesta, Eros, Apophis — `reference_asteroids()`), not the curated roster, so it is expected to be unaffected. This task verifies that and fixes anything that does drift.

**Files:**
- Possibly modify: `crates/pleiades-jpl/src/reference_summary/**` tests/snapshots — only if the full suite shows drift.

**Interfaces:**
- Consumes: the full `pleiades-jpl` test suite.
- Produces: a green `pleiades-jpl` suite with any drifted asteroid-count snapshot updated to the new derived value.

- [ ] **Step 1: Run the full jpl suite**

Run: `cargo test -p pleiades-jpl`
Expected: PASS. If any `reference_summary` test fails on an asteroid count/body-list string, it has genuinely drifted.

- [ ] **Step 2: Fix only genuine drift**

For each failing assertion, replace the expected literal with the new derived value the test prints (counts/body lists). Do **not** weaken an assertion to a range — keep exact equality. If nothing failed, this task makes no code change (record that and skip to Step 4).

- [ ] **Step 3: Re-run to confirm green**

Run: `cargo test -p pleiades-jpl`
Expected: PASS.

- [ ] **Step 4: Commit (only if files changed)**

```bash
git add crates/pleiades-jpl/src/reference_summary
git commit -m "test(jpl): realign asteroid report snapshots after Tier-A promotion (slice 1)"
```

---

### Task 6: Update claim surfaces — provenance doc, README, PLAN (kernel-free)

Gate 2 (documented astrological usage) is enforced here: every newly-added body must carry a cited source, or it must be removed from the roster (back to Task 2) and dropped from the slice.

**Files:**
- Modify: `docs/spk-kernel-sourcing.md` (Tier-A member list + per-new-body usage citations)
- Modify: `README.md:28` (the "seven `sb441-n16` Tier-A asteroids …" sentence)
- Modify: `PLAN.md` (record the slice)

**Interfaces:**
- Consumes: `CONFIRMED_NEW_BODIES` (Task 1); the final Tier-A body list and counts (Tasks 2–4).
- Produces: claim surfaces consistent with the roster.

- [ ] **Step 1: Cite astrological usage for each new body (gate 2)**

In `docs/spk-kernel-sourcing.md`, under "Asteroid kernel (Tier A — pinned)": (a) expand the member-list bullet to name the promoted bodies; (b) add a short "Astrological usage" sub-list with a **real, citable source** per *newly added* body (Hebe is a reclassification — its prior Tier-B inclusion already established relevance, but cite it too for symmetry). Acceptable sources: Martha Lang-Wescott, *Mechanics of the Future: Asteroids*; the Swiss Ephemeris asteroid name list; astro.com's asteroid catalog; an equivalent published asteroid-astrology reference. **If no citable source exists for a body, delete that body from the roster (Task 2), regenerate (Task 3), and drop it here** — do not promote on assertion. Also confirm the regen recipe lines now cover the promoted bodies (they do, automatically, via the roster filter).

- [ ] **Step 2: Update the README claim sentence**

Replace the `README.md:28` fragment
`the seven \`sb441-n16\` Tier-A asteroids (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris) are release-grade via the corpus-dependent JPL/SPK backend`
with the new count and body list, e.g. (adjust to the final set):
`the nine \`sb441-n16\` Tier-A asteroids (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris, Hebe, Cybele) are release-grade via the corpus-dependent JPL/SPK backend`.

- [ ] **Step 3: Record the slice in PLAN.md**

Update the asteroid bullet in "Important current limits" and the Phase 6 progress prose: state that the Tier-A release-grade asteroid set grew from 7 to `<new count>` by reclassifying Hebe (Tier-B→Tier-A) and adding `<confirmed goddesses>`; Tier-B count dropped accordingly; non-kernel Tier-B bodies (Chiron, Eros, …) remain constrained and are deferred to a follow-up slice. Follow the plan-maintenance rule (record state, do not narrate task-by-task).

- [ ] **Step 4: Verify counts are consistent everywhere**

Run:
```bash
cd /workspace
grep -rn "Tier-A\|sb441-n16\|release-grade" README.md PLAN.md docs/spk-kernel-sourcing.md | grep -i asteroid
```
Expected: the new Tier-A asteroid count and body list match across all three files and the roster.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md docs/spk-kernel-sourcing.md
git commit -m "docs: record asteroid sb441-n16 Tier-A promotion (slice 1)"
```

---

### Task 7: Full gate run — claims audit, release gates, kernel-gated accuracy (final verification)

**Files:** none (verification only).

**Interfaces:**
- Consumes: all prior tasks.
- Produces: green workspace + green release gates.

- [ ] **Step 1: Workspace test run (kernel-free)**

Run: `cargo test --workspace`
Expected: PASS. In particular `claims-audit` structural checks pass — promoted bodies are `ReleaseGrade` with `CorpusValidated{"sb441-n16"}` evidence (consistency rule at `crates/pleiades-validate/src/claims/audit.rs:101`), and `corpus_spec.rs:396` still passes (asteroids stayed out of `release_bodies()`/`constrained_bodies()`).

- [ ] **Step 2: Release gate (kernel-free)**

Run the release-smoke/release-gate entry point used by this repo:
```bash
cargo test -p pleiades-validate release
```
Expected: PASS — the full numeric-gate set + overclaim audit run green with the new claim surfaces.

- [ ] **Step 3: Kernel-gated release-grade accuracy audit**

Run:
```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
PLEIADES_AST_KERNEL=/path/to/sb441-n16.bsp \
  cargo test -p pleiades-validate -- --ignored audit_release_grade_accuracy
```
Expected: PASS — each promoted Tier-A asteroid is within the `BodyClass::Asteroid` ceiling (30″ lon/lat, 5e6 km). With the kernels absent this audit is dormant by design (`audit.rs:244-264`); with them present every promoted body must clear the ceiling. If any body exceeds it, remove that body (Tasks 2/3/6) rather than ship it over-claimed.

- [ ] **Step 4: Final confirmation commit (if any snapshot moved)**

If Steps 1–3 required no further edits, there is nothing to commit. Otherwise:
```bash
git add -A
git commit -m "test: finalize asteroid sb441-n16 Tier-A promotion gates (slice 1)"
```

---

## Self-Review

**Spec coverage:**
- Spec §1 Selection policy (two gates) → Task 1 (gate 1, kernel membership) + Task 6 Step 1 (gate 2, cited usage). ✓
- Spec §2 Candidate roster (Hebe reclassify + goddesses) → Task 1 (confirm) + Task 2 (retag). ✓
- Spec §3 Corpus & data changes (Tier-A regen, Tier-B removal, tier-driven claims) → Task 3 (reference regen) + Task 4 (constrained removal); claims flip automatically via `spk_body_claims` (no code change, noted in Task 2). ✓
- Spec §4 Validation gates (regen ≤1 km, Asteroid ceiling, validate-corpus) → Task 3 Steps 3–4 + Task 7 Step 3. ✓
- Spec §5 Claim-surface alignment (README, reports, provenance, PLAN, compat profile) → Task 5 (reports) + Task 6 (README/provenance/PLAN). Compatibility-profile bump: the rendered profile is house/ayanamsa-scoped and does not embed asteroid body claims (asteroid claims live in `spk_body_claims`, not the compat profile), so no version bump is required; Task 7 Step 2 re-runs the profile/overclaim gates to confirm. ✓
- Spec §6 Acceptance criteria → distributed across Tasks 3, 4, 6, 7. ✓

**Placeholder scan:** No "TBD"/"handle edge cases"/"similar to Task N". The one intentional parameterization — the exact promoted-body set — is gated on Task 1's empirical result and every task says explicitly how to adapt (edit the `confirmed` array; drop unconfirmed bodies). Checksums are intentionally not hard-coded (they depend on kernel data and are produced by the regen binary / drift gate), which is correct, not a placeholder.

**Type consistency:** `AsteroidTier::PinnedKernel`, `AsteroidClass::MainBelt`, `ast()`, `spk_body_claims`, `asteroid_reference_corpus()`, `corpus_checksum64`, `validate_drift`, `validate_asteroid_slices`, `audit_release_grade_accuracy` are all referenced with the signatures verified in the source during planning. Row math (407/body; 2849→`(8+N)×407`; 6652→6246) is consistent across Tasks 3–6.
