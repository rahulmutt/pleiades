# Ayanamsa Offset-Defined Family Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote every anchored, non-star-pinned built-in ayanamsa that empirically reproduces Swiss Ephemeris within the offset ceiling to `ReleaseGradeNumeric`, backed by SE numeric-gate evidence.

**Architecture:** Transcribe each in-scope mode's authoritative `(t0, ayan_t0)` anchor from SE's `sweph.h` `ayanamsa[]` table into a new `se_anchors.rs`; extend the SE reference tool to emit those modes; **measure** each mode's residual on the existing `anchor + IAU-2006 precession` path; promote exactly the modes that pass and defer the rest with a recorded residual. Promotion = flip `claim_tier` in both catalog tables + add corpus rows + extend the gate's class/completeness maps + align profile/README/PLAN, all guarded by the existing fail-closed gate and overclaim audit.

**Tech Stack:** Rust workspace (cargo), `pleiades-ayanamsa` (catalog + thresholds + precession), `pleiades-validate` (numeric gate + overclaim audit), `pleiades-core` (compatibility profile), `tools/se-ayanamsa-reference` (SE FFI reference generator, links vendored `libswisseph-sys`).

## Global Constraints

- Reference engine: **Swiss Ephemeris 2.10.03**, mean ayanamsa flags `SEFLG_NONUT | SEFLG_NOABERR` (= 1088). Copy verbatim into any new corpus manifest line.
- Corpus checksums use `pleiades_apparent::fnv1a64` — a **non-canonical FNV prime**. Never recompute checksums with a stock FNV implementation; always use the in-repo function.
- Holdout JD grid is fixed (10 instants, already in `tools/se-ayanamsa-reference/src/main.rs`); do not change it.
- Ceiling rule (unchanged convention): per mode-class `ceil(measured_max_arcsec × 2)` with a **1.0″ floor**.
- Coverage window: 1900–2100 CE.
- A mode is `ReleaseGradeNumeric` **only** with passing corpus evidence; deferred modes stay `DescriptorOnly` (no overclaim). The overclaim audit enforces catalog ⇔ corpus-evidence ⇔ profile ⇔ README agreement bidirectionally.
- Building `tools/se-ayanamsa-reference` requires a working C compiler (`CC`, e.g. gcc/clang) to compile vendored Swiss Ephemeris. Task 2 cannot run without it. All other tasks run against the committed corpus and need no C compiler.

## File Structure

- `crates/pleiades-ayanamsa/src/se_anchors.rs` **(new)** — SE-sourced `(SE_SIDM, t0, ayan_t0, t0_is_ut)` table for in-scope modes, with provenance comments; one lookup fn `se_anchor(&Ayanamsa) -> Option<SeAnchor>`.
- `crates/pleiades-ayanamsa/src/lib.rs` — register `mod se_anchors;`.
- `tools/se-ayanamsa-reference/src/main.rs` — extend `MODES` with in-scope `(name, SE_SIDM)` pairs; extend `holdout` subcommand to measure all modes.
- `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv` — append promoted-mode rows.
- `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt` — bump `rows=` and `checksum=`.
- `crates/pleiades-ayanamsa/src/thresholds.rs` — extend `ayanamsa_mode_class` to map the promoted set to `OffsetDefined` (or a sub-class); recompute ceiling if measured max rose; update tests.
- `crates/pleiades-ayanamsa/src/catalog.rs` — set anchors + flip `new` → `new_release_grade` for promoted modes, in **both** `BUILT_IN_AYANAMSAS` and the matching `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` entry.
- `crates/pleiades-validate/src/ayanamsa_validation.rs` — extend `mode_for_code` and the completeness list; update gate tests (`modes_checked`, `validated_modes` count).
- `crates/pleiades-ayanamsa/src/catalog/tests.rs` — update `release_grade_numeric_ayanamsa_set_is_exactly_the_six_gated_modes` to the new set + add a deferred-stay-`DescriptorOnly` guard.
- `README.md` — update the "N release-claimed ayanamsa modes pass" prose token.
- `PLAN.md` — update the Phase 6 ayanamsa note (new gated count + remaining deferred set).

> **Empirical parameterization:** Task 2 produces the **promoted set P** and **deferred set D** (a measured table committed into this plan file's Task 2 results block). Tasks 3–8 operate over P and D. Worked examples below use `J2000` as the representative mode; repeat the shown edits for **every** mode in P.

---

### Task 1: SE anchor table (`se_anchors.rs`)

Transcribe each in-scope mode's authoritative `(t0, ayan_t0, t0_is_ut)` from SE's anchor table. No C compiler needed — this is source transcription + a pure lookup.

**Files:**
- Create: `crates/pleiades-ayanamsa/src/se_anchors.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (add `mod se_anchors;`)
- Test: inline `#[cfg(test)]` in `se_anchors.rs`

**Interfaces:**
- Produces: `pub(crate) struct SeAnchor { pub se_sidm: i32, pub t0: f64, pub ayan_t0: f64, pub t0_is_ut: bool }` and `pub(crate) fn se_anchor(a: &Ayanamsa) -> Option<SeAnchor>`.

**Source of truth (read-only, already on disk):**
- SE_SIDM integer constants: `~/.cargo/registry/src/*/libswisseph-sys-0.1.2/libswisseph/swephexp.h` lines ~238–286.
- Anchor rows `{t0, ayan_t0, t0_is_UT, prec_offset}`: `.../libswisseph/sweph.h`, `static const struct aya_init ayanamsa[SE_NSIDM_PREDEF]` (starts ~line 351), one row per SE_SIDM index with a naming comment.

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::Ayanamsa;

    #[test]
    fn j2000_anchor_matches_se_table() {
        // sweph.h ayanamsa[18] = J2000: {J2000, 0, FALSE, ...}
        let a = se_anchor(&Ayanamsa::J2000).expect("J2000 has an SE anchor");
        assert_eq!(a.se_sidm, 18);
        assert!((a.t0 - 2_451_545.0).abs() < 1e-6);
        assert!((a.ayan_t0 - 0.0).abs() < 1e-9);
        assert!(!a.t0_is_ut);
    }

    #[test]
    fn every_anchor_is_finite_and_unique_sidm() {
        use std::collections::BTreeSet;
        let mut seen = BTreeSet::new();
        for a in IN_SCOPE_ANCHORS {
            assert!(a.t0.is_finite() && a.ayan_t0.is_finite());
            assert!(seen.insert(a.se_sidm), "duplicate SE_SIDM {}", a.se_sidm);
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-ayanamsa se_anchors 2>&1 | tail -20`
Expected: FAIL — `se_anchors` module / `se_anchor` not found.

- [ ] **Step 3: Write the module**

Create `crates/pleiades-ayanamsa/src/se_anchors.rs`. For each in-scope mode, add a row transcribed from `sweph.h`. Example head (fill the full in-scope set from the spec §Scope — `J2000, J1900, B1950, UshaShashi, DeLuce, Yukteshwar, JnBhasin, DjwhalKhul, Udayagiri, LahiriIcrc, Lahiri1940, LahiriVP285, KrishnamurtiVP291, BabylonianKugler1/2/3, BabylonianHuber, BabylonianEtaPiscium, BabylonianAldebaran, BabylonianBritton, Hipparchus, Sassanian, Suryasiddhanta499, Aryabhata499, Aryabhata522, Suryasiddhanta499MeanSun, Aryabhata499MeanSun, SuryasiddhantaRevati, SuryasiddhantaCitra, ValensMoon, PvrPushyaPaksha, Sheoran`):

```rust
//! SE-sourced ayanamsa anchors, transcribed from Swiss Ephemeris 2.10.03
//! `sweph.h` `static const struct aya_init ayanamsa[SE_NSIDM_PREDEF]`.
//! Each row: { SE_SIDM index, t0 (JD), ayan_t0 (deg at t0), t0_is_UT }.
//! Provenance: libswisseph-sys 0.1.2 vendored SE 2.10.03. SE_SIDM indices
//! per `swephexp.h`. Do not edit values without re-checking the SE source row
//! named in the trailing comment.
use pleiades_types::Ayanamsa;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SeAnchor {
    pub se_sidm: i32,
    pub t0: f64,
    pub ayan_t0: f64,
    pub t0_is_ut: bool,
}

pub(crate) const IN_SCOPE_ANCHORS: &[(Ayanamsa, SeAnchor)] = &[
    // sweph.h ayanamsa[18]: {J2000, 0, FALSE} — J2000
    (Ayanamsa::J2000, SeAnchor { se_sidm: 18, t0: 2_451_545.0, ayan_t0: 0.0, t0_is_ut: false }),
    // ... transcribe every in-scope mode here, each with its sweph.h row comment ...
];

pub(crate) fn se_anchor(a: &Ayanamsa) -> Option<SeAnchor> {
    IN_SCOPE_ANCHORS.iter().find(|(m, _)| m == a).map(|(_, s)| *s)
}
```

Add `mod se_anchors;` to `crates/pleiades-ayanamsa/src/lib.rs` (near the other `mod` lines, e.g. beside `mod precession;`).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-ayanamsa se_anchors 2>&1 | tail -20`
Expected: PASS (both tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/se_anchors.rs crates/pleiades-ayanamsa/src/lib.rs
git commit -m "feat(ayanamsa): add SE-sourced anchor table for offset-defined family"
```

---

### Task 2: Extend SE tool + measure residuals (produces P and D)

**Requires a C compiler.** Extend the reference tool to emit and measure all in-scope modes, then record which pass the offset ceiling.

**Files:**
- Modify: `tools/se-ayanamsa-reference/src/main.rs` (`MODES` table; `holdout` loop)

**Interfaces:**
- Consumes: SE_SIDM integers from Task 1's `se_anchors.rs` provenance (same values).
- Produces: a committed **Task 2 results block** (below) listing per-mode worst residual and pass/defer.

- [ ] **Step 1: Extend the `MODES` table**

In `tools/se-ayanamsa-reference/src/main.rs`, add every in-scope `(name, SE_SIDM)` pair to `MODES` (names must match the `mode_code` strings used in Task 6's `mode_for_code`). Example additions:

```rust
const MODES: &[(&str, i32)] = &[
    ("FaganBradley",  0), ("Lahiri", 1), ("Raman", 3), ("Krishnamurti", 5),
    ("TrueChitra", 27), ("TrueCitra", 27),
    // offset-defined family (Task 1 set):
    ("J2000", 18), ("J1900", 19), ("B1950", 20), ("UshaShashi", 4),
    ("DeLuce", 2), ("Yukteshwar", 7), ("JnBhasin", 8), ("DjwhalKhul", 6),
    ("BabylonianKugler1", 9), ("BabylonianKugler2", 10), ("BabylonianKugler3", 11),
    ("BabylonianHuber", 12), ("BabylonianEtaPiscium", 13), ("BabylonianAldebaran", 14),
    ("Hipparchus", 15), ("Sassanian", 16), ("Suryasiddhanta499", 21),
    ("Suryasiddhanta499MeanSun", 22), ("Aryabhata499", 23), ("Aryabhata499MeanSun", 24),
    ("SuryasiddhantaRevati", 25), ("SuryasiddhantaCitra", 26), ("Aryabhata522", 37),
    ("BabylonianBritton", 38), ("ValensMoon", 42), ("Lahiri1940", 43),
    ("LahiriVP285", 44), ("KrishnamurtiVP291", 45), ("LahiriIcrc", 46),
    // Note: PvrPushyaPaksha, Udayagiri, Sheoran have no upstream SE_SIDM — see Step 4.
];
```

- [ ] **Step 2: Add an offset-residual measurement subcommand**

Add a `measure-offset` mode that, for each `MODES` entry, computes the worst residual between SE and the pleiades offset model (`ayan_t0 + IAU-2006 precession from t0`). Because the tool cannot depend on `pleiades-ayanamsa`'s private precession fn, reproduce the same IAU-2006 longitude precession formula the crate uses (copy from `crates/pleiades-ayanamsa/src/precession.rs::general_precession_longitude_arcsec` + `precession_delta_degrees`) so the measurement matches the gate's path exactly:

```rust
fn precession_delta_degrees(jd_tt: f64, epoch_jd_tt: f64) -> f64 {
    // MUST stay identical to crates/pleiades-ayanamsa/src/precession.rs.
    (general_precession_longitude_arcsec((jd_tt - 2_451_545.0) / 36_525.0)
        - general_precession_longitude_arcsec((epoch_jd_tt - 2_451_545.0) / 36_525.0)) / 3600.0
}

fn emit_offset_measure() {
    // For each (name, code): anchor (t0, ayan_t0) from the same SE row,
    // worst |SE(jd) - (ayan_t0 + precession_delta_degrees(jd, t0))| * 3600 over HOLDOUT_JD_TT.
    // Print: "name worst=<arcsec> verdict=<PASS|DEFER@2.0>"
}
```

- [ ] **Step 3: Build and run (needs `CC`)**

Run:
```bash
cd tools/se-ayanamsa-reference && cargo run -q -- measure-offset
```
Expected: one line per mode with `worst=<arcsec>`.

If `cargo build` fails with `libswisseph-sys` build-script error and `CC = None`, install a C compiler (e.g. `apt-get install -y gcc` or set `CC=clang`) and retry. The pre-built `target/debug/se-ayanamsa-reference` only has the original 6 modes and cannot measure the new ones.

- [ ] **Step 4: Record P and D in this plan file**

Paste the measured table into the block below. **Promoted set P** = modes with `worst ≤ ceiling`; **Deferred set D** = the rest (with reason). Modes with no upstream SE_SIDM (`PvrPushyaPaksha`, `Udayagiri`, `Sheoran` — confirm against `swephexp.h`) go to D with reason "no SE reference mode". Set the **ceiling** = `ceil(max(worst over P) × 2)`, 1.0″ floor; if a sub-group (e.g. mean-Sun) is bounded but distinctly higher, give it its own class/ceiling, else defer it.

```
<!-- TASK 2 RESULTS — measured via `cargo run -q -- measure-offset`
(SE 2.10.03, MEAN_IFLAG=1088; model = ayan_t0 + IAU-2006 precession_delta from t0;
worst = max |wrap_to_pm180(SE - model)| * 3600 over the 10 HOLDOUT_JD_TT)

PROMOTED (P): mode  worst_arcsec        (17 modes — the tight cluster, worst <= 1.370402")
  J2000                       0.000280
  B1950                       0.000642
  UshaShashi                  0.001493
  DjwhalKhul                  0.001493
  Yukteshwar                  0.001493
  JnBhasin                    0.001493
  J1900                       0.001493
  LahiriIcrc                  0.361195
  Lahiri1940                  0.827955
  Sassanian                   1.162552
  Aryabhata522                1.293850
  Suryasiddhanta499           1.370402
  Suryasiddhanta499MeanSun    1.370402
  Aryabhata499                1.370402
  Aryabhata499MeanSun         1.370402
  SuryasiddhantaRevati        1.370402
  SuryasiddhantaCitra         1.370402

OFFSET CEILING (recomputed): 3.0"  (ceil(max(worst over P) * 2) = ceil(1.370402 * 2) = ceil(2.740804), floor 1.0)

SUB-CLASS (if any): NONE.
  The two mean-Sun modes (Suryasiddhanta499MeanSun, Aryabhata499MeanSun) do NOT cluster
  higher — their worst (1.370402") is identical to their non-mean-Sun siblings, so they
  fold into the shared OffsetDefined class with no separate ceiling.

DEFERRED (D): mode  reason  worst_arcsec        (15 modes)
  LahiriVP285        residual exceeds ceiling  2.272315   (large-residual)
  KrishnamurtiVP291  residual exceeds ceiling  2.242511   (large-residual)
  ValensMoon         residual exceeds ceiling  3.014063   (large-residual)
  DeLuce             residual exceeds ceiling  4.058897   (large-residual)
  BabylonianBritton  residual exceeds ceiling  4.058897   (large-residual)
  BabylonianKugler1  residual exceeds ceiling  4.894246   (large-residual)
  BabylonianKugler2  residual exceeds ceiling  4.894246   (large-residual)
  BabylonianKugler3  residual exceeds ceiling  4.894246   (large-residual)
  BabylonianHuber    residual exceeds ceiling  4.894246   (large-residual)
  BabylonianAldebaran residual exceeds ceiling 4.894246   (large-residual)
  Hipparchus         residual exceeds ceiling  5.145347   (large-residual)
  BabylonianEtaPiscium residual exceeds ceiling 5.159189  (large-residual)
  PvrPushyaPaksha    no SE reference mode      n/a        (no-SE_SIDM; absent from IN_SCOPE_ANCHORS)
  Udayagiri          no SE reference mode      n/a        (no-SE_SIDM; absent from IN_SCOPE_ANCHORS)
  Sheoran            no SE reference mode      n/a        (no-SE_SIDM; absent from IN_SCOPE_ANCHORS)

FAIL-SAFE NOTE: LahiriVP285 (2.272") and KrishnamurtiVP291 (2.243") fall numerically below
the 3.0" formula ceiling but sit in the doubling-headroom GAP (0.87" above the cluster top of
1.370402", and below the next residuals at 3.01"+), NOT in the cluster itself. Promoting them
would re-seed P with max=2.272" -> ceiling=ceil(4.545)=5.0", which then admits ValensMoon/DeLuce/
Britton/Kugler/Aldebaran (3.0-4.9") -> runaway ceiling that force-promotes the whole table. Per
resolution E ("NEVER loosen/inflate the ceiling to force-promote"), P is fixed to the defensible
empirical cluster and these two defer. The tool's printed PASS/DEFER uses a cutoff in the
unambiguous gap (1.5") so its verdict matches this record.
-->
```

- [ ] **Step 5: Commit**

```bash
git add tools/se-ayanamsa-reference/src/main.rs docs/superpowers/plans/2026-06-25-ayanamsa-offset-defined-promotion.md
git commit -m "feat(tool): emit + measure offset-defined ayanamsa modes; record P/D"
```

---

### Task 3: Thresholds — class map + ceiling

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/thresholds.rs`
- Test: inline tests in same file

**Interfaces:**
- Consumes: P, ceiling from Task 2.
- Produces: `ayanamsa_mode_class` returns `Some(OffsetDefined)` (or the new sub-class) for every mode in P.

- [ ] **Step 1: Write the failing test** (replace `only_the_six_release_modes_are_gated`)

```rust
#[test]
fn promoted_offset_modes_are_gated_as_offset_defined() {
    for m in [Ayanamsa::J2000 /*, …every mode in P… */] {
        assert_eq!(ayanamsa_mode_class(&m), Some(AyanamsaModeClass::OffsetDefined));
    }
    // A still-deferred mode stays ungated:
    assert_eq!(ayanamsa_mode_class(&Ayanamsa::GalacticCenter), None);
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-ayanamsa thresholds 2>&1 | tail -20`
Expected: FAIL — `J2000` currently returns `None`.

- [ ] **Step 3: Extend `ayanamsa_mode_class`**

Add every mode in P to the `OffsetDefined` arm (and, if Task 2 created a sub-class, add that variant + a `ayanamsa_mode_ceiling` arm with its ceiling). If Task 2's recomputed offset ceiling differs from 2.0″, update the `OffsetDefined` ceiling and the doc comment's "Measured maxima" line to the new value.

```rust
pub fn ayanamsa_mode_class(ayanamsa: &Ayanamsa) -> Option<AyanamsaModeClass> {
    match ayanamsa {
        Ayanamsa::Lahiri | Ayanamsa::Raman | Ayanamsa::Krishnamurti | Ayanamsa::FaganBradley
        | Ayanamsa::J2000 /* | …every mode in P… */ => Some(AyanamsaModeClass::OffsetDefined),
        Ayanamsa::TrueChitra | Ayanamsa::TrueCitra => Some(AyanamsaModeClass::TrueStar),
        _ => None,
    }
}
```

Also update `every_class_has_finite_positive_ceiling` if a sub-class enum variant was added.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-ayanamsa thresholds 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/thresholds.rs
git commit -m "feat(ayanamsa): gate offset-defined family in mode-class map"
```

---

### Task 4: Catalog — anchors + claim-tier flip

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/catalog.rs`
- Test: `crates/pleiades-ayanamsa/src/catalog/tests.rs`

**Interfaces:**
- Consumes: P (Task 2), `se_anchor` (Task 1).
- Produces: each P mode is `new_release_grade` with SE `(epoch=t0, offset=ayan_t0)` in **both** `BUILT_IN_AYANAMSAS` and its `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` entry.

- [ ] **Step 1: Write the failing test** (update `release_grade_numeric_ayanamsa_set_is_exactly_the_six_gated_modes`)

Rename to `release_grade_numeric_ayanamsa_set_is_exactly_the_gated_modes` and assert the expected set = the original 6 **plus every mode in P**:

```rust
let expected = [
    Ayanamsa::Lahiri, Ayanamsa::Raman, Ayanamsa::Krishnamurti, Ayanamsa::FaganBradley,
    Ayanamsa::TrueChitra, Ayanamsa::TrueCitra,
    Ayanamsa::J2000, /* …every mode in P… */
];
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-ayanamsa release_grade_numeric 2>&1 | tail -20`
Expected: FAIL — count/contains mismatch (P modes still `DescriptorOnly`).

- [ ] **Step 3: Edit the catalog**

For each mode in P, in **both** tables (`BUILT_IN_AYANAMSAS` and the matching `BASELINE_AYANAMSAS`/`RELEASE_AYANAMSAS` entry): change `AyanamsaDescriptor::new(` → `AyanamsaDescriptor::new_release_grade(` and set `epoch`/`offset_degrees` from `se_anchor`. Example for `J2000`:

```rust
AyanamsaDescriptor::new_release_grade(
    Ayanamsa::J2000,
    "J2000",
    &[/* unchanged aliases */],
    "J2000 sidereal zero point (SE SE_SIDM_J2000).",
    Some(JulianDay::from_days(2_451_545.0)),   // SE t0
    Some(Angle::from_degrees(0.0)),            // SE ayan_t0
),
```

Keep aliases/notes otherwise unchanged. (`offset_at`'s legacy linear path is now bypassed for these modes by Task 3's class map, so the offset value is consumed only by the `OffsetDefined` path.)

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-ayanamsa 2>&1 | tail -20`
Expected: PASS (catalog validation + the updated set test). If `PartialSiderealMetadata` fires, an entry has epoch xor offset — set both.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-ayanamsa/src/catalog.rs crates/pleiades-ayanamsa/src/catalog/tests.rs
git commit -m "feat(ayanamsa): set SE anchors + promote offset-defined family to release-grade"
```

---

### Task 5: Corpus rows + manifest

**Files:**
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`
- Modify: `crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt`

**Interfaces:**
- Consumes: P (Task 2), the tool's default corpus output (Task 2 build).
- Produces: corpus containing 10 rows per mode in P, checksum-pinned.

- [ ] **Step 1: Regenerate the corpus CSV (needs `CC`)**

Run:
```bash
cd tools/se-ayanamsa-reference && cargo run -q > /workspace/crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv
```
This emits the header + 10 rows for every `MODES` entry. Confirm the original 6 modes' rows are byte-identical to before (same SE version) so only additions changed:
```bash
cd /workspace && git diff --stat crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv
```
Expected: only added lines for P modes.

- [ ] **Step 2: Compute the new row count and checksum**

Row count = data rows (exclude header/comments). Checksum = `pleiades_apparent::fnv1a64(<csv contents>)`. Use the existing gate test to surface the correct value rather than guessing: temporarily run

Run: `cargo test -p pleiades-validate checksum_drift_fails_closed 2>&1 | tail -20`
Expected: FAIL — assertion prints `manifest.checksum` (old) vs `fnv1a64(CORPUS_CSV)` (new). Copy the new value.

- [ ] **Step 3: Update the manifest**

Edit `manifest.txt`: set `rows=<new count>` and `checksum=<new value>` on the `slice ayanamsa` line. Leave the `#Reference-*` provenance lines unchanged (same SE version/flags).

- [ ] **Step 4: Verify the gate passes over the committed corpus**

Run: `cargo test -p pleiades-validate ayanamsa 2>&1 | tail -30`
Expected: `gate_passes_over_committed_corpus` and `checksum_drift_fails_closed` PASS. (`modes_checked` assertion still expects 6 — fixed in Task 6; it may fail here, which is acceptable mid-task.)

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv crates/pleiades-validate/data/ayanamsa-corpus/manifest.txt
git commit -m "test(ayanamsa): commit SE corpus rows for promoted offset-defined modes"
```

---

### Task 6: Gate wiring — `mode_for_code`, completeness, counts

**Files:**
- Modify: `crates/pleiades-validate/src/ayanamsa_validation.rs`

**Interfaces:**
- Consumes: P (Task 2); `Ayanamsa` variants; corpus `mode_code` strings (Task 2 `MODES` names).
- Produces: gate validates `6 + |P|` modes; `validated_modes()` includes all of P.

- [ ] **Step 1: Update the gate tests to the new count**

In the `#[cfg(test)] mod tests`, change the expected counts from 6 to `6 + |P|` in `gate_passes_over_committed_corpus` (`report.modes_checked`) and `corpus_report_exposes_six_validated_modes` (rename to `…_exposes_all_validated_modes`, assert `validated_modes().len() == 6 + |P|`).

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-validate ayanamsa 2>&1 | tail -20`
Expected: FAIL — `mode_for_code` returns `None` for P modes → `UnknownModeCode`, and counts mismatch.

- [ ] **Step 3: Extend `mode_for_code` and the completeness list**

Add every P mode to `mode_for_code` (string ⇒ `Ayanamsa` variant, strings matching Task 2 `MODES` names) and to the completeness `for code in [...]` array:

```rust
fn mode_for_code(code: &str) -> Option<Ayanamsa> {
    match code {
        "Lahiri" => Some(Ayanamsa::Lahiri),
        // …existing 6…
        "J2000" => Some(Ayanamsa::J2000),
        // …every mode in P…
        _ => None,
    }
}
```

Add the same P codes to the completeness array so a missing row fails closed.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p pleiades-validate ayanamsa 2>&1 | tail -20`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/ayanamsa_validation.rs
git commit -m "feat(validate): gate promoted offset-defined ayanamsa modes"
```

---

### Task 7: Guard test — deferred set stays descriptor-only

**Files:**
- Modify: `crates/pleiades-ayanamsa/src/catalog/tests.rs`

**Interfaces:**
- Consumes: D (Task 2).
- Produces: a test asserting no deferred/slice-2 mode is `ReleaseGradeNumeric`.

- [ ] **Step 1: Write the failing-by-construction guard test**

```rust
#[test]
fn deferred_modes_stay_descriptor_only() {
    use pleiades_types::{Ayanamsa, CompatibilityClaimTier};
    let deferred = [
        Ayanamsa::GalacticCenter, Ayanamsa::TrueRevati, Ayanamsa::TrueMula,
        Ayanamsa::TruePushya, Ayanamsa::TrueSheoran, Ayanamsa::BabylonianTrueGeoc,
        // …every mode in D, including any measurement-deferred offset modes…
    ];
    for m in deferred {
        let d = crate::built_in_ayanamsas().iter().find(|d| d.ayanamsa == m).unwrap();
        assert_eq!(d.claim_tier, CompatibilityClaimTier::DescriptorOnly, "{m:?} must stay descriptor-only");
    }
}
```

- [ ] **Step 2: Run to verify it passes (guard is green by construction)**

Run: `cargo test -p pleiades-ayanamsa deferred_modes_stay 2>&1 | tail -20`
Expected: PASS. (If it fails, a deferred mode was wrongly promoted in Task 4 — fix Task 4.)

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-ayanamsa/src/catalog/tests.rs
git commit -m "test(ayanamsa): guard that deferred ayanamsa modes stay descriptor-only"
```

---

### Task 8: Align profile, README, PLAN; run full audit

**Files:**
- Modify: `README.md`
- Modify: `PLAN.md`
- (Profile content already auto-derives from catalog tables — verify, do not duplicate.)

**Interfaces:**
- Consumes: new RGN count `N = 6 + |P|`.
- Produces: green `compat-claims-audit` and `release-gate`.

- [ ] **Step 1: Update the README token**

The overclaim audit's Check C requires the README to contain `" {N} release-claimed"`. Edit `README.md` line ~20: change `6 release-claimed ayanamsa modes pass theirs` to `N release-claimed ayanamsa modes pass theirs`, and update the surrounding "(the rest are catalogued with metadata only)" count prose to match.

- [ ] **Step 2: Run the overclaim audit**

Run: `cargo test -p pleiades-validate compat 2>&1 | tail -30`
Expected: PASS — Check A (tier⇔evidence), Check B (profile count == descriptor count; both derive from the catalog tables edited in Task 4, so they move together), Check C (README token).

If Check B fails with `ProfileCountMismatch`, a mode was flipped in `BUILT_IN_AYANAMSAS` but not in its `BASELINE`/`RELEASE` twin (or vice-versa) — reconcile in `catalog.rs`.

- [ ] **Step 3: Update PLAN.md**

In PLAN.md's Phase 6 ayanamsa note, replace "6 release-claimed modes pass… remaining ~48 built-in ayanamsa variants are not-yet-gated" with the new gated count `N`, the offset-defined family promotion, and the remaining deferred set D (fitted/star-pinned family → slice 2). Update the `Status:` line's ayanamsa clause.

- [ ] **Step 4: Run the full release gate**

Run: `cargo test -p pleiades-validate 2>&1 | tail -40`
Expected: PASS — full numeric-gate set (house, ayanamsa, apparent, topocentric, corpus) + overclaim audit green.

Then the workspace build/clippy:
Run: `cargo test --workspace 2>&1 | tail -20 && cargo clippy --workspace --all-targets 2>&1 | tail -10`
Expected: all green, no new warnings.

- [ ] **Step 5: Commit**

```bash
git add README.md PLAN.md
git commit -m "docs: align README/PLAN to promoted offset-defined ayanamsa count"
```

---

## Self-Review

**Spec coverage:**
- Goal / empirical classifier → Tasks 2–4, 6 (measure → class map → tier flip → gate).
- Anchor sourcing & provenance (official SE t0/ayan_t0, `se_anchors.rs`) → Task 1.
- Ceiling policy (`ceil(max×2)`, 1.0″ floor, sub-family contingency) → Task 2 Step 4 + Task 3.
- Corpus & tool changes → Tasks 2, 5.
- Claims-surface alignment (catalog ×2 tables, profile, README, audit) → Tasks 4, 8.
- Error handling / fail-closed → preserved; exercised by Tasks 5–6 tests.
- Testing (gate count, per-mode residual via corpus, deferred-stay-descriptor-only) → Tasks 3, 4, 6, 7.
- Profile version bump deferred → noted in Task 8 (content only; no version bump).
- PLAN.md update → Task 8.

**Placeholder scan:** Code steps carry real code. The only deliberately deferred values are the empirical P/D set and the recomputed ceiling — these are *outputs of Task 2's measurement*, not authoring placeholders; every later task references them explicitly and shows the exact edit shape with `J2000` worked through end-to-end.

**Type consistency:** `SeAnchor`/`se_anchor` (Task 1) used in Tasks 2, 4. `AyanamsaModeClass::OffsetDefined` (existing) extended in Task 3, consumed by the gate in Task 6. `mode_for_code` strings (Task 6) match `MODES` names (Task 2). `claim_tier`/`new_release_grade` (Task 4) drive audit counts (Task 8). Corpus `mode_code` ⇔ `mode_for_code` ⇔ `MODES` name kept identical across Tasks 2/5/6.
