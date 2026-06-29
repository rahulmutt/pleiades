# Asteroid Per-Object Pinned-SPK Release-Grade Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote the astrologically-used, kernel-absent Tier-B asteroids (5 centaurs + 6 personal/minor/NEA bodies) from `Constrained` to `ReleaseGrade` by sourcing each from its own JPL per-object SPK (pinned by SHA), regenerating its corpus rows byte-reproducibly, and passing the existing accuracy + astrological-usage gates.

**Architecture:** The release-grade/constrained line is drawn by reproducibility from a pinned SPK. `AsteroidTier` is the grade discriminator; a new per-entry `source: &'static str` field carries the evidence source string (so `spk_body_claims` stops hard-coding `"sb441-n373s"`). A committed object-SPK manifest pins each per-object `.bsp` by SHA; the regen path loads those SPKs (uncommitted, env-gated) alongside de440 + n373s. The change is: add the `source` field, add the manifest, teach the regen path to load a per-object SPK directory, flip confirmed roster tiers + regenerate the reference CSV (atomic), filter the promoted bodies out of the constrained CSV, and align every count/label/citation surface.

**Tech Stack:** Rust (workspace crates `pleiades-jpl`, `pleiades-backend`, `pleiades-validate`), JPL SPK/DAF kernels, JPL Horizons/SBDB per-object SPK API, `cargo test`, env-gated maintainer binaries.

## Global Constraints

- **Pure Rust, no new deps** (`requirements.md` NFR-1).
- **Clean checkout stays kernel-free**: per-object `.bsp` files are never committed; all kernel-reading steps are env-gated and skip without the env var.
- **No `CelestialBody` / `Ayanamsa` enum change**: promoted bodies keep their existing `asteroid:NN-Name` `Custom` ids.
- **Stable roster order**: roster order feeds checksums/reports — flip tiers in place, never reorder existing entries.
- **Coverage window unchanged**: 1900–2100 CE (JD 2415020.5 .. 2488069.5), fail-closed outside it.
- **No new accuracy ceiling / backend boundary change**: existing `BodyClass::Asteroid` ceiling (30″ lon/lat, 5e6 km); backend stays mean/geometric/geocentric/tropical.
- **`sb441-n373s` untouched**: its pin, its 25 Tier-A bodies, and their committed rows stay byte-identical.
- **No Horizons re-fetch** of the remaining constrained slice — promoted rows are filtered out; the rest stay byte-identical.
- **SHA pins are lowercase 64-hex** (mirroring `asteroid_kernel_sha_is_pinned_64_hex`).
- **Reproducibility tolerance**: regenerated rows must match the committed CSV within < 1.0 km per coordinate (`corpus_regen.rs`).
- **Honest split outcome**: a candidate that fails any of the three gates (obtainable SPK, cited astrological usage, accuracy ceiling) stays `Constrained`; never force-promoted.

## File Structure

- `crates/pleiades-jpl/src/spk/asteroid_roster.rs` — add `source` field to `AsteroidEntry`; flip confirmed entries to `PinnedKernel`; update `spk_body_claims` to read `entry.source`; update tests. (Modify)
- `crates/pleiades-jpl/src/spk/object_spk.rs` — **new** module: the per-object SPK manifest (NAIF id, SHA, source label, request params, window) + format tests. (Create)
- `crates/pleiades-jpl/src/spk/mod.rs` — register the new `object_spk` module. (Modify)
- `crates/pleiades-jpl/src/bin/regenerate-asteroid-corpus.rs` — load the per-object SPK directory alongside de440 + n373s. (Modify)
- `crates/pleiades-jpl/tests/corpus_regen.rs` — load the per-object SPK directory in the asteroid-reference regen test. (Modify)
- `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` — regenerated with promoted rows. (Modify, generated)
- `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv` — promoted rows filtered out. (Modify, generated)
- `crates/pleiades-jpl/data/corpus/manifest.txt` — new checksums/rows for both slices. (Modify, generated)
- `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` — prose label of the posture report. (Modify)
- `crates/pleiades-validate/src/corpus/mod.rs` — Tier-A asteroid corpus name/description label. (Modify)
- `docs/spk-kernel-sourcing.md` — new per-object-SPK Tier-A section; Tier-B section decremented. (Modify)
- `README.md` — Tier-A count/body list; Tier-B sentence. (Modify)
- `PLAN.md` — record the slice. (Modify)

---

## Task 1: Source per-object SPKs, SHAs, window, confirmed set (maintainer prerequisite)

This task pins the facts every later task depends on. It is env-gated/manual and produces no committed code — its output (confirmed body set, per-body SHAs, source labels, window OK) is recorded and used by Tasks 3 and 5.

**Files:**
- None committed. Downloads land in `.kernels/objects/` (gitignored). Working notes only.

**Interfaces:**
- Produces:
  - `CANDIDATES` — the 11 current Tier-B bodies, each with NAIF id `2_000_000 + minor_planet_number`:
    `asteroid:2060-Chiron`→2002060, `asteroid:5145-Pholus`→2005145, `asteroid:7066-Nessus`→2007066, `asteroid:10199-Chariklo`→2010199, `asteroid:8405-Asbolus`→2008405, `asteroid:1221-Amor`→2001221, `asteroid:1181-Lilith`→2001181, `asteroid:944-Hidalgo`→2000944, `asteroid:1566-Icarus`→2001566, `asteroid:1685-Toro`→2001685, `asteroid:1862-Apollo`→2001862.
  - `CONFIRMED` — subset of `CANDIDATES` for which (a) a per-object SPK covering JD 2415020.5..2488069.5 was obtained and (b) an astrological-usage citation exists (see Step 4). Bodies failing either stay Tier-B.
  - `SHA[body]` — lowercase 64-hex SHA-256 of each confirmed body's `.bsp`.
  - `REQUEST[body]` — the exact Horizons/SBDB request params used (recorded for the manifest).

- [ ] **Step 1: Create the download dir (gitignored)**

```bash
mkdir -p .kernels/objects
grep -q '^.kernels/' .gitignore || echo '.kernels/' >> .gitignore
```

- [ ] **Step 2: Download a per-object SPK for each candidate over 1900–2100**

The JPL Horizons API returns a binary SPK (base64 in JSON) for a small body when `EPHEM_TYPE=SPK`. Run for each candidate (example shows Chiron = NAIF 2002060; repeat for all 11):

```bash
for nid in 2002060 2005145 2007066 2010199 2008405 2001221 2001181 2000944 2001566 2001685 2001862; do
  curl -s "https://ssd.jpl.nasa.gov/api/horizons.api" \
    --data-urlencode "format=json" \
    --data-urlencode "COMMAND='DES=${nid};'" \
    --data-urlencode "OBJ_DATA=NO" \
    --data-urlencode "MAKE_EPHEM=YES" \
    --data-urlencode "EPHEM_TYPE=SPK" \
    --data-urlencode "START_TIME=1899-12-01" \
    --data-urlencode "STOP_TIME=2100-02-01" \
    -o ".kernels/objects/${nid}.json"
  python3 - "$nid" <<'PY'
import sys, json, base64, pathlib
nid = sys.argv[1]
d = json.load(open(f'.kernels/objects/{nid}.json'))
spk = d.get('spk')
if not spk:
    print(f'{nid}: NO SPK in response — {d.get("error") or list(d)} (stays Tier-B)'); sys.exit(0)
pathlib.Path(f'.kernels/objects/{nid}.bsp').write_bytes(base64.b64decode(spk))
print(f'{nid}: wrote .bsp')
PY
done
```

Expected: a `.bsp` per body that JPL can solve. Any body with no SPK is dropped from `CONFIRMED` (stays Tier-B). Record which bodies produced a `.bsp`.

*(The exact request parameter spelling is verified here against the live API — do not hard-code it from memory elsewhere. If `COMMAND='DES=<nid>;'` is rejected for a numbered body, use the bare-number form `COMMAND='<minor_planet_number>;'` as `regenerate-asteroid-constrained.rs` already does.)*

- [ ] **Step 3: Record each SHA and verify the coverage window**

```bash
for f in .kernels/objects/*.bsp; do shasum -a 256 "$f"; done
```

Then DAF-inspect each `.bsp` to confirm it covers JD 2415020.5..2488069.5 (reuse the DAF-summary parse from the slice-2 plan Task 0 Step 3, pointed at each per-object file). A body whose SPK does not cover the full window stays Tier-B.

Record `SHA[body]` (64-hex) and `WINDOW_OK[body]` for each.

- [ ] **Step 4: Confirm an astrological-usage citation per body (gate 2)**

For each body that passed Steps 2–3, confirm a citable tradition (Swiss Ephemeris `seasnam.txt` catalog membership + an interpretive source). Reference set to cite in `docs/spk-kernel-sourcing.md` (Task 6):
- Centaurs — centaur astrology (e.g. Melanie Reinhart, *Chiron and the Healing Journey*; Robert von Heeren / Dieter Koch centaur work): 2060 Chiron (the wounded healer), 5145 Pholus, 7066 Nessus, 10199 Chariklo, 8405 Asbolus.
- Personal/minor/NEA — asteroid-astrology tradition (Martha Lang-Wescott, *Mechanics of the Future: Asteroids*; Demetra George, *Asteroid Goddesses*): 1221 Amor (love/compassion), 944 Hidalgo (advocacy/authority), 1566 Icarus (recklessness/risk), 1685 Toro (force/power), 1862 Apollo (NEA, ambition). 1181 Lilith — the numbered asteroid is in `seasnam.txt`; cite it explicitly as the asteroid (distinct from Black Moon Lilith) and only include if the citation is judged sufficient, else leave Tier-B.

`CONFIRMED` = bodies passing Steps 2, 3, and 4. Record the final set.

- [ ] **Step 5: Record outputs**

Write `CONFIRMED`, `SHA[body]`, `REQUEST[body]` (the param set from Step 2), and `WINDOW_OK[body]` into working notes. No commit.

---

## Task 2: Add per-entry `source` to `AsteroidEntry` and thread it through claims

Behavior-preserving refactor: every existing `PinnedKernel` body keeps source `"sb441-n373s"`, every `Constrained` body keeps `"horizons"`, so claims are byte-identical before any promotion. This removes the hard-coded source string so promoted bodies can later declare their own.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs`

**Interfaces:**
- Produces: `AsteroidEntry { body, tier, class, source: &'static str }`; `spk_body_claims` emits `ClaimEvidence::CorpusValidated { source: entry.source.to_string() }` for Tier-A and Tier-B bodies.
- Consumes: nothing new.

- [ ] **Step 1: Update the claims test to assert per-entry source**

Replace the body of `tier_a_claims_cite_n373s` (it currently asserts every Tier-A claim cites `"sb441-n373s"`) with one that asserts each Tier-A claim cites the source declared on its roster entry:

```rust
    #[test]
    fn tier_a_claims_cite_their_declared_source() {
        use pleiades_backend::ClaimEvidence;
        let claims = spk_body_claims(&tier_a_bodies());
        assert!(!claims.is_empty());
        for c in &claims {
            let entry = asteroid_core_roster()
                .iter()
                .find(|e| e.body == c.body)
                .expect("claim body is in roster");
            match &c.evidence {
                ClaimEvidence::CorpusValidated { source } => {
                    assert_eq!(source, entry.source, "{:?} cites wrong source", c.body)
                }
                other => panic!("unexpected evidence {other:?}"),
            }
        }
        // Until promotion, every Tier-A body is still kernel-sourced.
        assert!(claims.iter().all(|c| matches!(
            &c.evidence,
            ClaimEvidence::CorpusValidated { source } if source == "sb441-n373s"
        )));
    }
```

- [ ] **Step 2: Run it to verify it fails to compile**

Run: `cargo test -p pleiades-jpl asteroid_roster 2>&1 | head -30`
Expected: compile error — `AsteroidEntry` has no field `source`.

- [ ] **Step 3: Add the `source` field and set it on every entry**

In `crates/pleiades-jpl/src/spk/asteroid_roster.rs`:

Add the field to the struct:

```rust
pub struct AsteroidEntry {
    pub body: CelestialBody,
    pub tier: AsteroidTier,
    pub class: AsteroidClass,
    /// Evidence source string used in `spk_body_claims` (e.g. `"sb441-n373s"`,
    /// `"jpl-sbdb-spk:2060"`, `"horizons"`).
    pub source: &'static str,
}
```

Change the `e` closure to take the source and apply the rule to all entries — **`PinnedKernel` → `"sb441-n373s"`, `Constrained` → `"horizons"`** (no behavior change yet):

```rust
            let e = |body, tier, class, source| AsteroidEntry {
                body,
                tier,
                class,
                source,
            };
            vec![
                e(CelestialBody::Ceres, PinnedKernel, MainBelt, "sb441-n373s"),
                e(CelestialBody::Pallas, PinnedKernel, MainBelt, "sb441-n373s"),
                // … every PinnedKernel entry gets "sb441-n373s" …
                e(ast("2060-Chiron"), Constrained, Centaur, "horizons"),
                e(ast("5145-Pholus"), Constrained, Centaur, "horizons"),
                // … every Constrained entry gets "horizons" …
            ]
```

Apply to all ~36 entries: PinnedKernel rows → `"sb441-n373s"`, Constrained rows → `"horizons"`.

- [ ] **Step 4: Update `spk_body_claims` to read `entry.source`**

Replace the hard-coded source strings. Look up the entry to get its source:

```rust
pub fn spk_body_claims(covered: &[CelestialBody]) -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let roster = asteroid_core_roster();
    covered
        .iter()
        .cloned()
        .map(|body| {
            match roster.iter().find(|e| e.body == body) {
                Some(e) if e.tier == AsteroidTier::PinnedKernel => BodyClaim::release_grade(
                    body,
                    AccuracyClass::High,
                    ClaimEvidence::CorpusValidated {
                        source: e.source.to_string(),
                    },
                ),
                Some(e) => BodyClaim::constrained(
                    body,
                    AccuracyClass::Moderate,
                    ClaimEvidence::CorpusValidated {
                        source: e.source.to_string(),
                    },
                ),
                None => BodyClaim::constrained(
                    body,
                    AccuracyClass::High,
                    ClaimEvidence::CorpusValidated {
                        source: "de440".to_string(),
                    },
                ),
            }
        })
        .collect()
}
```

- [ ] **Step 5: Run the roster tests**

Run: `cargo test -p pleiades-jpl asteroid_roster`
Expected: PASS (all existing tests, including the rewritten claims test).

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/spk/asteroid_roster.rs
git commit -m "refactor(jpl): per-entry source field on AsteroidEntry, claims read it (slice 3)"
```

---

## Task 3: Add the per-object SPK manifest module

A committed, CI-visible record of each per-object `.bsp`: NAIF id, pinned SHA, source label, request params, window. The `.bsp` files stay uncommitted. Fill the real SHAs/params from Task 1's `CONFIRMED`/`SHA`/`REQUEST`.

**Files:**
- Create: `crates/pleiades-jpl/src/spk/object_spk.rs`
- Modify: `crates/pleiades-jpl/src/spk/mod.rs`

**Interfaces:**
- Produces:
  - `pub struct ObjectSpk { pub body_designation: &'static str, pub naif_id: i32, pub sha256: &'static str, pub source_label: &'static str, pub request: &'static str }`
  - `pub fn object_spk_manifest() -> &'static [ObjectSpk]`
  - `pub fn object_spk_for(designation: &str) -> Option<&'static ObjectSpk>`
- Consumes: nothing.

- [ ] **Step 1: Write failing tests for the manifest**

Create `crates/pleiades-jpl/src/spk/object_spk.rs` with the test module only first (so it fails to compile), then add the impl in Step 3. Tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sha_is_lowercase_64_hex() {
        for o in object_spk_manifest() {
            assert_eq!(o.sha256.len(), 64, "{} sha length", o.body_designation);
            assert!(
                o.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
                "{} sha not lowercase hex",
                o.body_designation
            );
        }
    }

    #[test]
    fn naif_ids_match_designation_number() {
        for o in object_spk_manifest() {
            let n: i32 = o
                .body_designation
                .split([':', '-'])
                .find_map(|s| s.parse().ok())
                .expect("designation has a number");
            assert_eq!(o.naif_id, 2_000_000 + n, "{} naif id", o.body_designation);
        }
    }

    #[test]
    fn source_labels_and_requests_are_present() {
        for o in object_spk_manifest() {
            assert!(o.source_label.starts_with("jpl-sbdb-spk:"), "{}", o.body_designation);
            assert!(!o.request.is_empty(), "{}", o.body_designation);
        }
    }

    #[test]
    fn lookup_round_trips() {
        for o in object_spk_manifest() {
            assert_eq!(
                object_spk_for(o.body_designation).map(|x| x.naif_id),
                Some(o.naif_id)
            );
        }
    }
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p pleiades-jpl object_spk 2>&1 | head -20`
Expected: compile error (`object_spk_manifest` not found).

- [ ] **Step 3: Implement the manifest with the real Task-1 values**

Prepend to `crates/pleiades-jpl/src/spk/object_spk.rs` (above the test module). Include **one entry per body in `CONFIRMED`** from Task 1, using its real `SHA[body]` and `REQUEST[body]`. Example shows Chiron; add every confirmed body:

```rust
//! Per-object pinned SPK manifest for Tier-A asteroids absent from the bundled
//! `sb441-n373s` perturber kernel (centaurs, personal/minor/NEA bodies). Each
//! `.bsp` is sourced once from JPL Horizons over 1900–2100 and pinned by SHA;
//! the files are uncommitted (like de440/sb441-n373s) — this manifest is the
//! committed provenance. The regen path loads them from `PLEIADES_OBJECT_SPK_DIR`.

/// One pinned per-object SPK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectSpk {
    /// Roster `Custom` designation, e.g. `"asteroid:2060-Chiron"`.
    pub body_designation: &'static str,
    /// NAIF id (`2_000_000 + minor-planet number`).
    pub naif_id: i32,
    /// Lowercase 64-hex SHA-256 of the pinned `.bsp`.
    pub sha256: &'static str,
    /// Claim evidence source label (`"jpl-sbdb-spk:<number>"`).
    pub source_label: &'static str,
    /// Exact Horizons/SBDB request used to generate the SPK (provenance).
    pub request: &'static str,
}

/// The committed per-object SPK manifest, in roster order.
pub fn object_spk_manifest() -> &'static [ObjectSpk] {
    &[
        ObjectSpk {
            body_designation: "asteroid:2060-Chiron",
            naif_id: 2_002_060,
            sha256: "<SHA from Task 1 Step 3>",
            source_label: "jpl-sbdb-spk:2060",
            request: "Horizons EPHEM_TYPE=SPK COMMAND='DES=2002060;' START=1899-12-01 STOP=2100-02-01",
        },
        // … one ObjectSpk per body in CONFIRMED …
    ]
}

/// Looks up the pinned SPK for a roster designation.
pub fn object_spk_for(designation: &str) -> Option<&'static ObjectSpk> {
    object_spk_manifest()
        .iter()
        .find(|o| o.body_designation == designation)
}
```

*(The `<SHA from Task 1 Step 3>` markers are filled with the real digests recorded in Task 1 — they are inputs, not placeholders. The manifest contains exactly the `CONFIRMED` bodies.)*

- [ ] **Step 4: Register the module**

In `crates/pleiades-jpl/src/spk/mod.rs`, add alongside the other `pub mod` lines:

```rust
pub mod object_spk;
```

- [ ] **Step 5: Run the manifest tests**

Run: `cargo test -p pleiades-jpl object_spk`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/spk/object_spk.rs crates/pleiades-jpl/src/spk/mod.rs
git commit -m "feat(jpl): per-object SPK manifest for kernel-absent Tier-A asteroids (slice 3)"
```

---

## Task 4: Load the per-object SPK directory in the regen binary and the regen test

Teach the Tier-A regen path to load each manifest `.bsp` from `PLEIADES_OBJECT_SPK_DIR` alongside de440 + n373s, so flipped bodies become `supports_body == true` and their rows are generated. Env-gated: absent dir → no per-object kernels loaded (existing behavior unchanged, clean checkout still skips).

**Files:**
- Modify: `crates/pleiades-jpl/src/bin/regenerate-asteroid-corpus.rs`
- Modify: `crates/pleiades-jpl/tests/corpus_regen.rs`

**Interfaces:**
- Consumes: `object_spk_manifest()` (Task 3); `SpkBackend::builder().add_kernel(...)`.
- Produces: both the binary and the `regenerated_asteroid_reference_matches_checked_in` test load `<PLEIADES_OBJECT_SPK_DIR>/<naif_id>.bsp` for each manifest entry that exists on disk.

- [ ] **Step 1: Add a shared helper to load per-object SPKs onto a builder**

In `regenerate-asteroid-corpus.rs`, after reading the kernels, add the per-object dir load. Replace the builder block:

```rust
    let mut builder = SpkBackend::builder()
        .add_kernel(&de)
        .map_err(|e| e.message)?
        .add_kernel(&ast)
        .map_err(|e| e.message)?;

    // Per-object SPKs for kernel-absent Tier-A bodies (centaurs/NEA). Env-gated:
    // without the dir, none load and only n373s/de440 bodies are covered.
    if let Ok(dir) = std::env::var("PLEIADES_OBJECT_SPK_DIR") {
        use pleiades_jpl::spk::object_spk::object_spk_manifest;
        for o in object_spk_manifest() {
            let path = format!("{dir}/{}.bsp", o.naif_id);
            if std::path::Path::new(&path).exists() {
                builder = builder.add_kernel(&path).map_err(|e| e.message)?;
            } else {
                eprintln!("per-object SPK missing (body stays uncovered): {path}");
            }
        }
    }

    let backend = builder.build();
```

(Keep the existing `import` line `use pleiades_jpl::spk::...` intact; the `object_spk_manifest` is imported locally above.)

- [ ] **Step 2: Mirror the load in the regen test**

In `crates/pleiades-jpl/tests/corpus_regen.rs`, `regenerated_asteroid_reference_matches_checked_in`, replace the builder construction with the same dir-gated load:

```rust
    let mut builder = SpkBackend::builder()
        .add_kernel(&de)
        .unwrap()
        .add_kernel(&ast)
        .unwrap();
    if let Ok(dir) = std::env::var("PLEIADES_OBJECT_SPK_DIR") {
        use pleiades_jpl::spk::object_spk::object_spk_manifest;
        for o in object_spk_manifest() {
            let path = format!("{dir}/{}.bsp", o.naif_id);
            if std::path::Path::new(&path).exists() {
                builder = builder.add_kernel(&path).unwrap();
            }
        }
    }
    let backend = builder.build();
```

- [ ] **Step 3: Verify the workspace still builds and the gated test still skips**

Run: `cargo test -p pleiades-jpl --test corpus_regen`
Expected: PASS (skips without `PLEIADES_DE_KERNEL`; the new code compiles).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-jpl/src/bin/regenerate-asteroid-corpus.rs crates/pleiades-jpl/tests/corpus_regen.rs
git commit -m "feat(jpl): regen path loads per-object SPK dir alongside de440+n373s (slice 3)"
```

---

## Task 5: Promote confirmed bodies and regenerate the corpus (atomic)

Flip each `CONFIRMED` entry to `PinnedKernel` with its per-object source label, then regenerate `asteroid_reference.csv` (now including the promoted rows), filter the promoted bodies out of `asteroid_constrained.csv`, and update `manifest.txt`. These land in one commit because claims-audit holds release-grade bodies to the ceiling against the committed rows — the roster and the CSVs must agree.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs`
- Modify (generated): `crates/pleiades-jpl/data/corpus/asteroid_reference.csv`, `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv`, `crates/pleiades-jpl/data/corpus/manifest.txt`

**Interfaces:**
- Consumes: `CONFIRMED` + `SHA`/`source_label` (Tasks 1, 3); the regen binary (Task 4).
- Produces: each `CONFIRMED` body is `PinnedKernel` in the roster with `source = "jpl-sbdb-spk:<n>"`; its rows live once in `asteroid_reference.csv`.

- [ ] **Step 1: Update roster guard tests for the promoted set**

In `asteroid_roster.rs` tests: for each `CONFIRMED` body add it to the appropriate promoted-assertion list, and flip the per-body assertions. Concretely, add a test asserting every confirmed body is `PinnedKernel` with a per-object source:

```rust
    #[test]
    fn slice3_confirmed_bodies_are_tier_a_object_spk() {
        // CONFIRMED designations from Task 1 (fill with the real set).
        let confirmed = [
            "2060-Chiron", "5145-Pholus", "7066-Nessus", "10199-Chariklo", "8405-Asbolus",
            "1221-Amor", "944-Hidalgo", "1566-Icarus", "1685-Toro", "1862-Apollo",
            // include "1181-Lilith" only if CONFIRMED includes it
        ];
        for designation in confirmed {
            let e = asteroid_core_roster()
                .iter()
                .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == designation))
                .unwrap_or_else(|| panic!("{designation} missing"));
            assert_eq!(e.tier, AsteroidTier::PinnedKernel, "{designation} tier");
            assert!(e.source.starts_with("jpl-sbdb-spk:"), "{designation} source");
        }
    }
```

Update `chiron_is_constrained_centaur`: if Chiron is in `CONFIRMED`, replace it with a release-grade assertion (tier `PinnedKernel`, class `Centaur`, source starts with `jpl-sbdb-spk:`); if Chiron is *not* confirmed, leave it unchanged. Remove any now-promoted body from the implicit "Tier-B" expectations of `tiers_are_disjoint_and_cover_roster` — that test stays valid automatically (it counts, not names).

- [ ] **Step 2: Run to verify the new test fails**

Run: `cargo test -p pleiades-jpl asteroid_roster 2>&1 | head -30`
Expected: FAIL — confirmed bodies are still `Constrained`.

- [ ] **Step 3: Flip the confirmed entries in the roster**

For each `CONFIRMED` body, change its entry in place (preserve order): `Constrained → PinnedKernel`, class unchanged, source `"horizons" → "jpl-sbdb-spk:<number>"`. Example:

```rust
                e(ast("2060-Chiron"), PinnedKernel, Centaur, "jpl-sbdb-spk:2060"),
                e(ast("5145-Pholus"), PinnedKernel, Centaur, "jpl-sbdb-spk:5145"),
                e(ast("7066-Nessus"), PinnedKernel, Centaur, "jpl-sbdb-spk:7066"),
                e(ast("10199-Chariklo"), PinnedKernel, Centaur, "jpl-sbdb-spk:10199"),
                e(ast("8405-Asbolus"), PinnedKernel, Centaur, "jpl-sbdb-spk:8405"),
                e(ast("1221-Amor"), PinnedKernel, MainBelt, "jpl-sbdb-spk:1221"),
                e(ast("944-Hidalgo"), PinnedKernel, MainBelt, "jpl-sbdb-spk:944"),
                e(ast("1566-Icarus"), PinnedKernel, MainBelt, "jpl-sbdb-spk:1566"),
                e(ast("1685-Toro"), PinnedKernel, MainBelt, "jpl-sbdb-spk:1685"),
                e(ast("1862-Apollo"), PinnedKernel, MainBelt, "jpl-sbdb-spk:1862"),
```

Leave any **non-confirmed** body as `Constrained, …, "horizons"` (honest split outcome). Update the inline comments that say "absent from sb441-n373s, stay Tier B" to reflect the per-object-SPK promotion.

- [ ] **Step 4: Run roster tests (code-only, no kernels)**

Run: `cargo test -p pleiades-jpl asteroid_roster`
Expected: PASS (tier/source assertions). The regen/accuracy gates run in later steps.

- [ ] **Step 5: Regenerate the reference CSV and filter the constrained CSV (env-gated)**

With de440, n373s, and the per-object SPK dir present:

```bash
PLEIADES_DE_KERNEL=.kernels/de440.bsp \
PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp \
PLEIADES_OBJECT_SPK_DIR=.kernels/objects \
  cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
```

Expected stderr: each confirmed body printed `supported_by_kernel=true`; a `wrote …asteroid_reference.csv: <rows> rows` line; a `filtered …asteroid_constrained.csv: <rows> rows` line. Note the two printed `slice …` manifest lines.

- [ ] **Step 6: Update `manifest.txt` with the two new lines**

Replace the `slice asteroid_reference …` and `slice asteroid_constrained …` lines in `crates/pleiades-jpl/data/corpus/manifest.txt` with the exact lines printed in Step 5 (new rows + checksums).

- [ ] **Step 7: Verify reproducibility within tolerance**

```bash
PLEIADES_DE_KERNEL=.kernels/de440.bsp \
PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp \
PLEIADES_OBJECT_SPK_DIR=.kernels/objects \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

Expected: PASS — `regenerated_asteroid_reference_matches_checked_in` reproduces every row (including promoted bodies) within < 1.0 km; the existing 25 Tier-A rows are unchanged.

- [ ] **Step 8: Verify the accuracy ceiling for promoted bodies**

Run the release-grade accuracy audit (the claims-audit small-body check that compares `asteroid_reference` rows to the `BodyClass::Asteroid` ceiling):

```bash
cargo run -q -p pleiades-validate -- claims-audit
```

Expected: exits 0, no overclaim/accuracy failure for the promoted bodies. If any promoted body exceeds the 30″ / 5e6 km ceiling, revert that body to `Constrained, …, "horizons"` (Step 3), drop it from `CONFIRMED`/the manifest (Task 3), drop its rows (re-run Step 5), and record it as staying Tier-B.

- [ ] **Step 9: Commit the atomic promotion**

```bash
git add crates/pleiades-jpl/src/spk/asteroid_roster.rs \
        crates/pleiades-jpl/data/corpus/asteroid_reference.csv \
        crates/pleiades-jpl/data/corpus/asteroid_constrained.csv \
        crates/pleiades-jpl/data/corpus/manifest.txt
git commit -m "feat(jpl): promote per-object-SPK asteroids to Tier-A, regen corpus (slice 3)"
```

---

## Task 6: Align claim surfaces (reports, validate label, docs, README, PLAN)

Make every count/label/citation surface consistent with the new Tier-A/Tier-B split.

**Files:**
- Modify: `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs`
- Modify: `crates/pleiades-validate/src/corpus/mod.rs`
- Modify: `docs/spk-kernel-sourcing.md`
- Modify: `README.md`
- Modify: `PLAN.md`

**Interfaces:**
- Consumes: the final Tier-A/Tier-B counts and `CONFIRMED` set.

- [ ] **Step 1: Update the validate-crate asteroid corpus label**

In `crates/pleiades-validate/src/corpus/mod.rs` (~line 388), the Tier-A asteroid window is named/described as `sb441-n373s`-only. Generalize it to reflect the mixed source:

```rust
                name: "Tier-A asteroid reference window (sb441-n373s + per-object SPK)".to_string(),
                description: "Asteroid accuracy-ceiling corpus built from the committed Tier-A reference rows (sb441-n373s perturber kernel plus per-object JPL SPKs for centaurs/NEA), used by the slow release-grade accuracy audit for small bodies.",
```

Run any test asserting this string (e.g. in `crates/pleiades-validate/src/tests/`) and update the expected text to match.

- [ ] **Step 2: Update the selected-asteroid posture prose**

In `selected_asteroid.rs::selected_asteroid_constrained_class_report`, the counts are already dynamic (`tier_a_bodies().len()` / `tier_b_bodies().len()`), so only the prose wording needs to stay truthful. Change "pinned-kernel (Tier A) reproducible core and a Horizons-sourced (Tier B) constrained set" framing so Tier A reads as "pinned-kernel + per-object SPK (reproducible)". Update the matching expected string in `selected_asteroid.rs`'s `tests` module.

Run: `cargo test -p pleiades-jpl selected_asteroid`
Expected: PASS.

- [ ] **Step 3: Update `docs/spk-kernel-sourcing.md`**

Add a new subsection "Asteroid per-object SPKs (Tier A — pinned, kernel-absent bodies)" listing, per `CONFIRMED` body: NAIF id, SHA-256, Horizons request params, verified window, and the astrological-usage citation from Task 1 Step 4. Decrement the existing "Tier B — Horizons-sourced, constrained" section to the bodies that stayed Constrained (and remove the promoted ones from its body list / count). Record the regen recipe including `PLEIADES_OBJECT_SPK_DIR`.

- [ ] **Step 4: Update `README.md`**

Update the Tier-A asteroid sentence (count + body list now includes the promoted centaurs/NEA) and the Tier-B sentence (decremented count + remaining body list, or removal if the constrained slice is now empty).

- [ ] **Step 5: Update `PLAN.md`**

Per the plan-maintenance rule, update the "Important current limits" asteroid bullet, the Phase-6 progress narrative, and the status line: record slice 3 (mechanism = per-object pinned SPK; promoted set; new Tier-A count = 25 + |CONFIRMED|; new Tier-B count = 11 − |CONFIRMED|). Refresh the `Status:` date stamp.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs \
        crates/pleiades-validate/src/corpus/mod.rs \
        docs/spk-kernel-sourcing.md README.md PLAN.md
git commit -m "docs+report: align Tier-A/Tier-B asteroid surfaces with slice-3 promotion"
```

---

## Task 7: Full gate run and branch finish

**Files:** none (verification + integration).

- [ ] **Step 1: Workspace test suite**

Run: `cargo test --workspace`
Expected: PASS, no failures. (Kernel-gated tests skip without env vars.)

- [ ] **Step 2: Release gates**

Run the project gates the same way CI does (mise tasks wrapping `cargo run -q -p pleiades-validate -- <gate>`):

```bash
cargo run -q -p pleiades-validate -- validate-corpus
cargo run -q -p pleiades-validate -- claims-audit
cargo run -q -p pleiades-validate -- compat-claims-audit
cargo run -q -p pleiades-validate -- release-smoke
mise run release-gate
```

Expected: each exits 0. Confirm no surface still reports a stale Tier-A/Tier-B count or a body in both tiers.

- [ ] **Step 3: Confirm the empty-Tier-B path (only if all 11 promoted)**

If `CONFIRMED` is all 11, confirm `tier_b_bodies()` is empty and `asteroid_constrained.csv` is header-only without panics in the report/claims/validation paths. Run: `cargo test -p pleiades-jpl selected_asteroid && cargo run -q -p pleiades-validate -- validate-corpus`. Expected: PASS / exit 0.

- [ ] **Step 4: Compatibility profile bump (if applicable)**

If the rendered compatibility profile embeds asteroid claims or a kernel label (confirm by grepping the rendered profile for `sb441` / asteroid body names), bump its patch version and update the content-checksum guard, mirroring slice-4 hygiene. Otherwise, note "no profile bump (house/ayanamsa-scoped)".

- [ ] **Step 5: Finish the branch**

Use `superpowers:finishing-a-development-branch` to merge `phase6-asteroid-slice3-per-object-spk-promotion` (or open a PR), per the project's slice workflow.

---

## Self-review notes

- **Spec coverage:** §1 selection policy → Task 1 (gates) + Task 5 Step 8 (accuracy). §2 sourcing/manifest → Tasks 1, 3. §3 data model → Task 2. §4 regen path → Task 4. §5 corpus changes → Task 5. §6 validation gates → Task 5 Steps 7–8, Task 7. §7 cadence note → handled by existing `AsteroidClass` (centaurs 365 d; NEA stays MainBelt 180 d unless Task 1 measurement shows a need — flagged in Task 5 Step 8). §8 claim surfaces → Task 6. §9 behavior → no runtime change (Task 2 keeps claims byte-identical pre-promotion). Risks (empty Tier-B, accuracy failure, SPK availability) → Task 5 Step 8, Task 7 Step 3, Task 1 Steps 2–4.
- **NEA cadence (§7):** kept at the existing MainBelt 180 d validation cadence; the runtime interpolates the SPK directly, so this is validation density only. If Task 5 Step 8 shows an NEA failing the ceiling purely from coarse sampling, a `NearEarth` class / finer cadence is the documented fallback — not pre-built (YAGNI).
- **Type consistency:** `source: &'static str` is defined in Task 2 and consumed in Tasks 2/5; `ObjectSpk`/`object_spk_manifest`/`object_spk_for` defined in Task 3 and consumed in Task 4; `PLEIADES_OBJECT_SPK_DIR` used identically in the binary and the test (Task 4).
- **Parameterized inputs:** `CONFIRMED`, `SHA[body]`, `REQUEST[body]` are verified-at-implementation outputs of Task 1 (mirroring slice 2's Task 0), not placeholders.
