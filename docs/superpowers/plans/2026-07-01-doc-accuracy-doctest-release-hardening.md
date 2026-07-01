# Doc-accuracy + doctest release hardening — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. **Note:** the source spec calls for executing this via the **Workflow tool** (multi-agent orchestration); the workflow script in Task 0 is the primary execution vehicle. The task breakdown below is the authoritative work definition the workflow implements, and is also directly executable task-by-task if the workflow is not used.

**Goal:** Make every public doc comment across the 13 public `pleiades` crates accurate (no overclaims), document every reachable public item under `#![deny(missing_docs)]`, and add runnable doctests for four common astrology usecases — all verified by `cargo test --doc` and `cargo doc -D warnings`.

**Architecture:** A single multi-agent workflow with four phases — (1) read-only per-crate audit/map, (2) dependency-tiered document-and-enforce, (3) four usecase doctests, (4) integration verify + synthesis. Crates are processed leaf-first so a crate is documented only after its dependencies are stable. The `#![deny(missing_docs)]` lint turns the compiler into the authoritative work-list generator.

**Tech Stack:** Rust workspace (cargo), `mise` task runner, rustdoc/doctests, `pleiades-data`'s embedded packaged artifact (`packaged_backend()`) as the real backend for runnable chart doctests.

## Global Constraints

- **In-scope crates (13):** `pleiades-types`, `pleiades-backend`, `pleiades-core`, `pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`, `pleiades-elp`, `pleiades-jpl`, `pleiades-compression`, `pleiades-time`, `pleiades-apparent`, `pleiades-apsides`, `pleiades-eclipse`. Out of scope: `pleiades-cli`, `pleiades-data`, `pleiades-validate`.
- **No API or behavior changes.** Docs, doctests, and the one `#![deny(missing_docs)]` inner attribute per crate only. No changes to public signatures, no runtime logic changes.
- **Accuracy over polish.** Every accuracy/capability claim in a doc comment must be cross-checked against three sources: the crate's actual code, the README's per-backend limits section, and the relevant `validate-*` gates. Correct overclaims to match reality; name the specific backend/path a claim applies to.
- **Doctests must be runnable** (no bare `ignore`/`no_run` unless a real external kernel file is genuinely required). Use `pleiades_data::packaged_backend()` for real chart computations; fall back to the in-tree `DemoBackend` pattern (see `pleiades-core/src/lib.rs`) only when a real backend cannot run offline.
- **Verification commands** (exact): per-crate `cargo test --doc -p <crate>` and `cargo doc -p <crate> --no-deps --all-features`; workspace-wide `cargo test --workspace` and `mise run docs` (which is `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`).
- **Dependency tiers** (process in this order, barrier between tiers):
  - Tier 0: `pleiades-types`, `pleiades-apsides`
  - Tier 1: `pleiades-backend`, `pleiades-compression`, `pleiades-time`, `pleiades-ayanamsa`
  - Tier 2: `pleiades-apparent`, `pleiades-vsop87`, `pleiades-jpl`
  - Tier 3: `pleiades-houses`, `pleiades-elp`, `pleiades-eclipse`
  - Tier 4: `pleiades-core`
- **Commit cadence:** one commit per crate task (Tasks 2–14) and one per usecase-doctest task (Tasks 15–18). Message form: `docs(<crate>): document public API + deny(missing_docs)` / `docs(core): add <usecase> usecase doctest`.

---

## Task 0: Baseline + workflow scaffold

**Files:**
- Create: `/tmp/claude-1000/-workspace/4c2fc872-7a82-45f0-92f5-46184e302491/scratchpad/doc-baseline.txt` (baseline capture, not committed)
- Create: workflow script (authored inline via the Workflow tool; persisted under the session dir)

**Interfaces:**
- Produces: a recorded green baseline (current `cargo test --workspace` + `mise run docs` status) so any regression introduced later is attributable; the workflow script that implements Tasks 1–19.

- [ ] **Step 1: Capture the current green baseline**

Run: `cargo test --workspace 2>&1 | tail -5 && mise run docs 2>&1 | tail -5`
Expected: both succeed (record pass/fail + doctest counts to `doc-baseline.txt`). If `mise run docs` already fails, note the pre-existing failures — they are not introduced by this work.

- [ ] **Step 2: Confirm the deterministic missing-docs enumeration works on one crate**

Temporarily prepend `#![deny(missing_docs)]` to `crates/pleiades-apsides/src/lib.rs` (smallest crate, ~5 items) and run:
Run: `cargo build -p pleiades-apsides 2>&1 | grep -c "missing documentation"`
Expected: a concrete count of undocumented reachable items (may be 0). Revert the edit.

- [ ] **Step 3: Author the workflow script**

Write the four-phase workflow (see the "Workflow script" appendix at the end of this plan) via the Workflow tool. Do not run it yet if executing task-by-task.

- [ ] **Step 4: Commit the baseline note only**

The baseline lives in scratchpad (not committed). No commit for this task unless a plan/spec tweak is needed.

---

## Per-crate procedure (applies to Tasks 2–14)

Each crate task (Tier 0 → Tier 4) follows the identical five-step cycle below. Only the crate name and its known claim hotspots differ; those are listed per task.

1. **Audit (read).** Read the crate's `lib.rs` module docs and every item doc comment. For each accuracy/capability claim, cross-check against (a) the code, (b) README per-backend limits, (c) `validate-*` gates. List overclaims to correct.
2. **Fix overclaims.** Edit prose so claims match reality and name the specific backend/path. Example correction shape: `"release-grade positions"` → `"release-grade only via the packaged-data artifact; the VSOP87 path here stays approximate for Pluto"`.
3. **Enforce.** Add `#![deny(missing_docs)]` as an inner attribute at the top of the crate's `lib.rs` (with the other `#![...]` attributes).
4. **Document flagged items.** Run `cargo build -p <crate>` to get the compiler's authoritative list of `missing documentation` items. Write an accurate, scoped rustdoc line for each — describing what it is, its units/frame where relevant, and any backend-specific caveat. Do not paraphrase the name; state what it does.
5. **Verify + commit.** `cargo test --doc -p <crate>` and `cargo doc -p <crate> --no-deps --all-features` must both pass clean; then commit.

**Per-crate cycle, concrete commands** (substitute `<crate>`):

- [ ] Read docs + list overclaims (Audit)
- [ ] Apply prose corrections (Fix)
- [ ] Add `#![deny(missing_docs)]` to `crates/<crate>/src/lib.rs`
- [ ] `cargo build -p <crate> 2>&1 | grep "missing documentation"` → document each reported item
- [ ] `cargo test --doc -p <crate>` → Expected: PASS (or "0 tests" if the crate has no doctests yet)
- [ ] `cargo doc -p <crate> --no-deps --all-features` → Expected: builds with no warnings
- [ ] `git add crates/<crate>/src && git commit -m "docs(<crate>): document public API + deny(missing_docs)"`

---

## Tier 0

### Task 2: `pleiades-types`
**Files:** Modify `crates/pleiades-types/src/*.rs` (lib.rs + item docs).
**Claim hotspots:** unit/frame semantics on `Angle`, `ObserverLocation` (lat/lon sign conventions, elevation units), `CoordinateFrame`, `ZodiacMode`, `HouseSystem` variants. These are pure types — the risk is *incorrect* semantic docs, not overclaims. Follow the per-crate procedure.

### Task 3: `pleiades-apsides`
**Files:** Modify `crates/pleiades-apsides/src/*.rs`.
**Claim hotspots:** True-Lilith (osculating true apogee/perigee) is release-grade via the packaged-Moon-derived backend gated by `validate-lilith` (max longitude residual ~306″). Ensure any accuracy wording matches that residual. Follow the per-crate procedure.

## Tier 1

### Task 4: `pleiades-backend`
**Files:** Modify `crates/pleiades-backend/src/*.rs`.
**Claim hotspots:** `BackendCapabilities`, `AccuracyClass`, `BodyClaim`, `BackendProvenance` — these are the vocabulary the whole per-backend-claim system uses; docs must describe them without asserting any specific backend's accuracy. Follow the per-crate procedure.

### Task 5: `pleiades-compression`
**Files:** Modify `crates/pleiades-compression/src/*.rs`.
**Claim hotspots:** codec/artifact format (ARTIFACT_VERSION, stored-vs-derived channels, size budget ≤ 12 MB). Keep numbers consistent with `spec/data-compression.md`. Follow the per-crate procedure.

### Task 6: `pleiades-time`
**Files:** Modify `crates/pleiades-time/src/*.rs` (calendar, convert, deltat, leap, sidereal, tdb, policy).
**Claim hotspots:** leap-second-exact UTC from 1972; observed/extrapolated Delta-T; TT↔TDB periodic term; 1900–2100 window; tiered `exact`/`observed`/`predicted` quality marker. Follow the per-crate procedure. This crate also anchors Task 18 doctests.

### Task 7: `pleiades-ayanamsa`
**Files:** Modify `crates/pleiades-ayanamsa/src/*.rs`.
**Claim hotspots:** 59 catalogued ayanamsas but only 48 release-claimed pass the gate (11 catalogued metadata-only). Any "supported ayanamsa" wording must distinguish release-claimed vs catalogued-only. Follow the per-crate procedure.

## Tier 2

### Task 8: `pleiades-apparent`
**Files:** Modify `crates/pleiades-apparent/src/*.rs` (aberration, apparent, equatorial, lighttime, nutation, parallax, precession, sidereal, policy).
**Claim hotspots:** apparent place = light-time + precession-to-date + annual aberration + nutation-in-longitude; **gravitational light-deflection omitted**. Equatorial-of-date (RA/Dec, true obliquity) gated by `validate-equatorial` / `-se`. Ensure the omission is stated. Follow the per-crate procedure. Currently has zero doctests — highest example-debt crate; anchors Task 18.

### Task 9: `pleiades-vsop87`
**Files:** Modify `crates/pleiades-vsop87/src/*.rs`.
**Claim hotspots:** VSOP87 Pluto stays **approximate**; first-party backend output is **mean, J2000 ecliptic** at the backend boundary. Do not let any doc imply apparent/of-date output at this layer. Follow the per-crate procedure.

### Task 10: `pleiades-jpl` (largest — may split by module)
**Files:** Modify `crates/pleiades-jpl/src/**`.
**Claim hotspots:** corpus-dependent JPL/SPK backend is release-grade for the Tier-A asteroid/TNO/centaur set; the fail-closed corpus gate governs availability. Distinguish corpus-backed release-grade bodies from approximate paths.
**Note:** this crate has by far the largest public surface. After Step 3 (add lint) + `cargo build`, if the `missing documentation` list is large, split the documentation work by module across multiple agents/passes; keep the single commit at the end. Follow the per-crate procedure otherwise.

## Tier 3

### Task 11: `pleiades-houses`
**Files:** Modify `crates/pleiades-houses/src/*.rs`.
**Claim hotspots:** 25 catalogued house systems, 24 pass the SE numeric gate; Porphyry high-latitude fallback behavior; `AscMc` chart points (ARMC, Vertex, antivertex, equatorial ascendant, co-ascendants, polar ascendant). Follow the per-crate procedure. Anchors Task 17.

### Task 12: `pleiades-elp`
**Files:** Modify `crates/pleiades-elp/src/*.rs`.
**Claim hotspots:** compact ELP Moon is **constrained** (not release-grade); release-grade Moon is via the packaged artifact. Any Moon-accuracy wording must say "compact/constrained." Follow the per-crate procedure.

### Task 13: `pleiades-eclipse`
**Files:** Modify `crates/pleiades-eclipse/src/*.rs`.
**Claim hotspots:** global/geocentric solar & lunar eclipse data for 1900-01-01 … 2100-01-01, validated against NASA's Five Millennium Canon by `validate-eclipses`; **local (per-observer) circumstances are not provided.** State that limit. Follow the per-crate procedure.

## Tier 4

### Task 14: `pleiades-core`
**Files:** Modify `crates/pleiades-core/src/**` (lib.rs, chart/*, api_stability, release_profiles, compatibility/*).
**Claim hotspots:** apparent-place-of-date is the default chart-layer output for release-grade bodies; per-backend claims flow through here; API stability + compatibility profile version strings must match the current profiles. The existing module-level `DemoBackend` doctest already passes — preserve it. Follow the per-crate procedure.

---

## Phase 3 — Usecase doctests

Each usecase doctest is a runnable rustdoc example added to the named home crate. Author it, then verify it compiles and runs. Where exact constructor signatures differ from the draft below, the verify step (`cargo test --doc`) is the source of truth — adjust the example until it passes; do **not** downgrade to `no_run` to dodge a mismatch.

### Task 15: Tropical natal chart doctest (home: `pleiades-core`)

**Files:** Modify `crates/pleiades-core/src/chart/mod.rs` (doc comment on `ChartEngine::chart` or a module-level `# Examples`).

- [ ] **Step 1: Write the doctest** (real backend via `pleiades-data` dev-dep)

```rust
/// # Examples
///
/// Build a tropical natal chart from a birth instant and location, then read
/// placements, signs, houses, and the Ascendant/Midheaven.
///
/// ```
/// use pleiades_core::{ChartEngine, ChartRequest};
/// use pleiades_types::{CelestialBody, HouseSystem, Instant, JulianDay, ObserverLocation, TimeScale};
/// use pleiades_data::packaged_backend;
///
/// // 2000-01-01 12:00 TT, observer at ~51.5°N, 0.0°E (London), sea level.
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let observer = ObserverLocation::new(51.5, 0.0, 0.0); // lat°, lon° (E+), elevation m
///
/// let request = ChartRequest::new(instant)
///     .with_observer(observer)
///     .with_house_system(HouseSystem::Placidus)
///     .with_bodies(vec![CelestialBody::Sun, CelestialBody::Moon]);
///
/// let engine = ChartEngine::new(packaged_backend());
/// let chart = engine.chart(&request).expect("packaged backend covers Sun & Moon");
///
/// // The Sun sits in some zodiac sign, and the chart exposes the Ascendant/MC.
/// assert!(chart.sign_for_body(&CelestialBody::Sun).is_some());
/// assert!(chart.asc_mc().is_some());
/// ```
```

- [ ] **Step 2: Verify it fails first if API is wrong, then passes**

Run: `cargo test --doc -p pleiades-core`
Expected: PASS. If it fails, fix constructor/method names against the crate (e.g. confirm `ObserverLocation::new` arity, `with_observer`, `sign_for_body`, `asc_mc`) until green. **Never** switch to `no_run` to hide a failure.

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-core/src/chart/mod.rs
git commit -m "docs(core): add tropical natal chart usecase doctest"
```

### Task 16: Sidereal chart + ayanamsa doctest (home: `pleiades-core`)

**Files:** Modify `crates/pleiades-core/src/chart/sidereal.rs` or `mod.rs` `# Examples`.

- [ ] **Step 1: Write the doctest** — same chart flow as Task 15 but selecting a sidereal zodiac mode with an explicit Lahiri ayanamsa, and asserting the sidereal longitude differs from the tropical one.

```rust
/// ```
/// use pleiades_core::{ChartEngine, ChartRequest};
/// use pleiades_types::{CelestialBody, Instant, JulianDay, ObserverLocation, TimeScale, ZodiacMode};
/// use pleiades_data::packaged_backend;
///
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let request = ChartRequest::new(instant)
///     .with_observer(ObserverLocation::new(51.5, 0.0, 0.0))
///     .with_zodiac_mode(ZodiacMode::sidereal_lahiri()) // confirm exact Lahiri selector against pleiades-ayanamsa
///     .with_bodies(vec![CelestialBody::Sun]);
///
/// let chart = ChartEngine::new(packaged_backend())
///     .chart(&request)
///     .expect("packaged backend covers the Sun");
/// assert!(chart.sign_for_body(&CelestialBody::Sun).is_some());
/// ```
```

- [ ] **Step 2:** `cargo test --doc -p pleiades-core` → PASS (confirm the exact `ZodiacMode` sidereal/Lahiri constructor against `pleiades-ayanamsa`; adjust until green).
- [ ] **Step 3:** commit `docs(core): add sidereal + Lahiri ayanamsa usecase doctest`.

### Task 17: House-system selection doctest (home: `pleiades-houses`)

**Files:** Modify `crates/pleiades-houses/src/lib.rs` `# Examples`.

- [ ] **Step 1: Write the doctest** — compute house cusps directly via the houses crate under a chosen system, showing `AscMc` and 12 cusps. Use the crate's own `HouseRequest`/entry point (confirm the exact public function during verify).

```rust
/// ```
/// use pleiades_houses::{compute_houses, HouseRequest}; // confirm exact public entry point
/// use pleiades_types::{HouseSystem, Instant, JulianDay, ObserverLocation, TimeScale};
///
/// let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
/// let request = HouseRequest::new(instant, ObserverLocation::new(51.5, 0.0, 0.0), HouseSystem::WholeSign);
/// let snapshot = compute_houses(&request).expect("Whole Sign is defined at mid-latitude");
///
/// assert_eq!(snapshot.cusps().len(), 12);
/// let _asc = snapshot.asc_mc(); // Ascendant / Midheaven and extended chart points
/// ```
```

- [ ] **Step 2:** `cargo test --doc -p pleiades-houses` → PASS (bind to the crate's real API — `HouseRequest`, the compute fn, cusp accessor, `asc_mc`).
- [ ] **Step 3:** commit `docs(houses): add house-system selection usecase doctest`.

### Task 18: Time/coords + corrections doctest (home: `pleiades-time`, cross-link `pleiades-apparent`)

**Files:** Modify `crates/pleiades-time/src/lib.rs` `# Examples` (civil→TT/TDB + sidereal time); add a companion example in `crates/pleiades-apparent/src/lib.rs` for apparent-vs-mean/topocentric.

- [ ] **Step 1: Write the time doctest** — civil UTC → TT conversion with the tiered quality marker, plus local sidereal time.

```rust
/// ```
/// use pleiades_time::{civil_utc_to_tt, greenwich_apparent_sidereal_time}; // confirm exact public names
/// // 2000-01-01 12:00:00 UTC → TT (leap-second-exact UTC, observed Delta-T).
/// let tt = civil_utc_to_tt(2000, 1, 1, 12, 0, 0.0).expect("inside the 1900–2100 window");
/// assert!(tt.quality().is_observed_or_better()); // tiered exact/observed/predicted marker
/// let _gast = greenwich_apparent_sidereal_time(tt.julian_day());
/// ```
```

- [ ] **Step 2:** `cargo test --doc -p pleiades-time` and `cargo test --doc -p pleiades-apparent` → PASS (bind to the crate's real conversion + sidereal-time + apparent-place entry points; adjust names until green).
- [ ] **Step 3:** commit `docs(time,apparent): add time/coords + corrections usecase doctest`.

---

## Task 19: Integration verify + synthesis

**Files:** none (verification); optionally update `docs/follow-ups.md` with any deferred items.

**Interfaces:**
- Consumes: all crate edits from Tasks 2–18.
- Produces: a green full-workspace build under `deny(missing_docs)` + all doctests, and a synthesis report.

- [ ] **Step 1: Full workspace doctest + test run**

Run: `cargo test --workspace`
Expected: PASS, doctest count strictly greater than the Task 0 baseline.

- [ ] **Step 2: Full docs build with -D warnings**

Run: `mise run docs`
Expected: PASS with zero warnings (proves `deny(missing_docs)` is satisfied workspace-wide and no rustdoc link/intra-doc warnings remain).

- [ ] **Step 3: Fix stragglers**

Address any cross-crate `missing_docs` or re-export gaps the per-crate passes missed (items re-exported from another crate can surface only in the workspace build). Re-run Steps 1–2 until green.

- [ ] **Step 4: Synthesis report**

Produce a short report: per-crate count of items documented, list of accuracy corrections (overclaims fixed) with before/after, and the four usecase doctests added. Save to scratchpad and surface in the final message.

- [ ] **Step 5: Final commit (if any straggler edits were made)**

```bash
git add -A && git commit -m "docs: workspace-wide missing_docs + doctest close-out"
```

---

## Self-review

- **Spec coverage:** (1) accuracy audit → per-crate Step "Audit/Fix" in Tasks 2–14 + claim hotspots; (2) `deny(missing_docs)` on all 13 → Tasks 2–14 Step 3 + Task 19 workspace verify; (3) four usecase doctests → Tasks 15–18; verification via `cargo test --doc` + `mise run docs` → Task 19. All spec success criteria mapped.
- **Placeholders:** worked doctests carry explicit "confirm exact name against the crate" notes because the plan cannot compile them in advance; the verify step (`cargo test --doc`) is the concrete pass/fail gate, so these are executable, not TBDs.
- **Type consistency:** method names used in doctests (`sign_for_body`, `asc_mc`, `with_observer`, `with_house_system`, `with_zodiac_mode`, `with_bodies`, `ChartEngine::new`, `ChartRequest::new`) match the core façade surface confirmed during planning; `packaged_backend()` matches `pleiades-data`'s public API. Any residual mismatch is caught by the per-task verify step.

---

## Appendix — Workflow script (primary execution vehicle)

The four phases above map to a single Workflow-tool script:

- **Phase 1 (Map):** `parallel` of 13 read-only agents, one per crate, each returning a structured map { crate, overclaims[], usecase-anchored }.
- **Phase 2 (Document + enforce):** process the 5 dependency tiers in order; within each tier, `parallel` over that tier's crates, each agent running the per-crate procedure and self-verifying `cargo test --doc -p <crate>` + `cargo doc -p <crate>`. Barrier between tiers.
- **Phase 3 (Usecase doctests):** `parallel` of 4 agents (Tasks 15–18), each self-verifying its doctest.
- **Phase 4 (Integration):** single serial agent running `cargo test --workspace` + `mise run docs`, fixing stragglers, emitting the synthesis report.

Cargo's `target/` lock serializes concurrent builds (safe); dependency-tiering prevents documenting a crate against churning deps; Phase 4 is the single source of truth.
