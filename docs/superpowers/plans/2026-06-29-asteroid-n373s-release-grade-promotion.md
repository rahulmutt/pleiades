# Asteroid sb441-n373s Kernel Swap + Release-Grade Promotion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retire the `sb441-n16` asteroid kernel, pin `sb441-n373s` as the single asteroid reference kernel, and promote every curated-roster asteroid/TNO confirmed present in `sb441-n373s` from Tier-B/Constrained to Tier-A/ReleaseGrade.

**Architecture:** The asteroid release-grade/constrained line is drawn purely by reproducibility from the pinned kernel. `AsteroidTier` drives `spk_body_claims`; kernel identity lives in two `corpus_spec.rs` constants; the regen path filters by tier. So the change is: swap the kernel pin, flip roster tiers for confirmed bodies, regenerate the Tier-A reference CSV against the new kernel, filter the promoted bodies out of the Tier-B constrained CSV (no Horizons re-fetch), and align every count/label surface.

**Tech Stack:** Rust (workspace crates `pleiades-jpl`, `pleiades-validate`), JPL SPK/DAF kernels, `cargo test`, env-gated maintainer binaries.

## Global Constraints

- **Pure Rust, no new deps** (`requirements.md` NFR-1).
- **Clean checkout stays kernel-free**: the 937 MB kernel is never committed; all kernel-reading steps are env-gated and skip without the env var.
- **No `CelestialBody` / `Ayanamsa` enum change**: promoted bodies keep their existing `asteroid:NN-Name` / `tno:NNNNN-Name` `Custom` ids.
- **Stable roster order**: roster order feeds checksums/reports — flip tiers in place, never reorder existing entries.
- **Coverage window unchanged**: 1900–2100 CE (JD 2415020.5 .. 2488069.5), fail-closed outside it.
- **No new accuracy ceiling / backend boundary change**: existing `BodyClass::Asteroid` ceiling (30″ lon/lat, 5e6 km); backend stays mean/geometric/geocentric/tropical.
- **Single asteroid kernel**: `sb441-n16` fully retired; `sb441-n373s` is the sole pinned asteroid kernel.
- **SHA pins are lowercase 64-hex** (enforced by `asteroid_kernel_sha_is_pinned_64_hex`).
- **Reproducibility tolerance**: regenerated Tier-A rows must match committed CSV within < 1.0 km per coordinate (`corpus_regen.rs`).

---

## Task 0: Confirm n373s membership, window, and SHA (maintainer prerequisite)

This task pins the facts every later task depends on. It is env-gated/manual and produces no committed code — its output (confirmed body set, SHA, window OK) is recorded and used as the source of truth for Tasks 1–5.

**Files:**
- None committed. Working notes only.

**Interfaces:**
- Produces: `CONFIRMED_BODIES` (the subset of the current Tier-B roster whose NAIF id is in `sb441-n373s`), `AST_SHA` (lowercase 64-hex), `WINDOW_OK` (bool: kernel covers JD 2415020.5..2488069.5).

- [ ] **Step 1: Download the kernel**

```bash
mkdir -p .kernels
curl -L -o .kernels/sb441-n373s.bsp \
  https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n373s.bsp
```

- [ ] **Step 2: Record the SHA-256**

```bash
shasum -a 256 .kernels/sb441-n373s.bsp
```

Expected: a 64-char lowercase hex digest. Record it as `AST_SHA`.

- [ ] **Step 3: Inspect DAF segments — NAIF ids present + coverage window**

Save and run this script (parses the DAF summary records the same way the in-repo reader does):

```bash
python3 - <<'PY'
import struct
PATH='.kernels/sb441-n373s.bsp'
# Roster Tier-B candidates -> NAIF id (2000000 + minor-planet number).
candidates = {
  'asteroid:5-Astraea':2000005,'asteroid:6-Hebe':2000006,'asteroid:8-Flora':2000008,
  'asteroid:9-Metis':2000009,'asteroid:19-Fortuna':2000019,'asteroid:80-Sappho':2000080,
  'asteroid:433-Eros':2000433,'asteroid:944-Hidalgo':2000944,'asteroid:1181-Lilith':2001181,
  'asteroid:1221-Amor':2001221,'asteroid:1566-Icarus':2001566,'asteroid:1685-Toro':2001685,
  'asteroid:1862-Apollo':2001862,'asteroid:2060-Chiron':2002060,'asteroid:5145-Pholus':2005145,
  'asteroid:7066-Nessus':2007066,'asteroid:8405-Asbolus':2008405,'asteroid:10199-Chariklo':2010199,
  'tno:20000-Varuna':2020000,'tno:28978-Ixion':2028978,'tno:50000-Quaoar':2050000,
  'tno:90377-Sedna':2090377,'tno:90482-Orcus':2090482,'tno:136108-Haumea':2136108,
  'tno:136199-Eris':2136199,'tno:136472-Makemake':2136472,'tno:225088-Gonggong':2225088,
}
f=open(PATH,'rb'); rec=f.read(1024)
assert rec[:7]==b'DAF/SPK', rec[:8]
nd,ni=struct.unpack('<ii',rec[8:16])
fward,bward,free=struct.unpack('<iii',rec[76:88])
ss=nd+(ni+1)//2
present=set(); windows={}
recno=fward
while recno!=0:
    f.seek((recno-1)*1024); r=f.read(1024)
    nxt,prv,nsum=struct.unpack('<ddd',r[:24]); nsum=int(nsum); off=24
    for _ in range(nsum):
        chunk=r[off:off+ss*8]; off+=ss*8
        dbls=struct.unpack('<%dd'%nd,chunk[:nd*8])
        ints=struct.unpack('<%di'%ni,chunk[nd*8:nd*8+ni*4])
        target=ints[0]; present.add(target)
        windows.setdefault(target,(dbls[0],dbls[1]))
    recno=int(nxt)
# Window check: ET seconds past J2000 -> JD via 2451545.0 + et/86400.
def et_to_jd(et): return 2451545.0 + et/86400.0
need_start, need_end = 2415020.5, 2488069.5
print("CONFIRMED (in kernel):")
for name,nid in sorted(candidates.items()):
    if nid in present:
        s,e=windows[nid]; js,je=et_to_jd(s),et_to_jd(e)
        ok = js<=need_start and je>=need_end
        print(f"  {name:28} JD {js:.1f}..{je:.1f} window_ok={ok}")
print("ABSENT (stay Tier-B):")
for name,nid in sorted(candidates.items()):
    if nid not in present: print(f"  {name}")
PY
```

- [ ] **Step 4: Record results**

From the output record:
- `CONFIRMED_BODIES` = the list under "CONFIRMED" — these get promoted.
- `WINDOW_OK` = every confirmed body shows `window_ok=True`.
- If any confirmed body shows `window_ok=False`, the abbreviated kernel is too narrow: re-run Steps 1–3 against the full `sb441-n373.bsp` (14 GB, same URL minus the `s`) and use it instead. The SHA and label in later tasks then refer to `sb441-n373.bsp`.

Expected result (to be confirmed, not assumed): main-belt `5-Astraea, 6-Hebe, 8-Flora, 9-Metis, 19-Fortuna` confirmed; TNOs `136199-Eris, 90377-Sedna, 136108-Haumea, 136472-Makemake, 50000-Quaoar, 90482-Orcus, 225088-Gonggong` likely confirmed (`28978-Ixion`, `20000-Varuna` uncertain); centaurs and small NEAs/personal asteroids absent.

> Tasks 1–5 below are written against this **expected** set as the worked example. Substitute `CONFIRMED_BODIES` exactly: include every confirmed body, omit any expected body that came back absent, and add any unexpected confirmed body the same way.

---

## Task 1: Pin the n373s kernel identity

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/corpus_spec.rs:79-84` (constants + doc), `crates/pleiades-jpl/src/spk/corpus_spec.rs:475-486` (pin test comment)
- Test: `crates/pleiades-jpl/src/spk/corpus_spec.rs` (existing `asteroid_kernel_sha_is_pinned_64_hex`)

**Interfaces:**
- Consumes: `AST_SHA` from Task 0.
- Produces: `AST_KERNEL_LABEL` (now names `sb441-n373s.bsp`), `AST_KERNEL_SHA256` (= `AST_SHA`).

- [ ] **Step 1: Update the constants and doc comment**

Replace lines 79-84:

```rust
/// Pinned identity of the Tier A small-body perturber kernel. SHA-256 is
/// computed via `shasum -a 256 sb441-n373s.bsp` and recorded here +
/// docs/spk-kernel-sourcing.md. `sb441-n373s` (343 main-belt perturbers + 30
/// KBOs, DE441-consistent) supersedes the retired 16-body `sb441-n16`.
pub const AST_KERNEL_LABEL: &str = "JPL DE small-body perturber kernel: sb441-n373s.bsp";
pub const AST_KERNEL_SHA256: &str =
    "<AST_SHA from Task 0 — lowercase 64-hex>";
```

- [ ] **Step 2: Update the pin-test comment**

In `asteroid_kernel_sha_is_pinned_64_hex` replace the comment line:

```rust
        // Pinned from `shasum -a 256 sb441-n373s.bsp`.
```

- [ ] **Step 3: Add a label assertion to the pin test**

Inside `asteroid_kernel_sha_is_pinned_64_hex`, after the existing assertions, add:

```rust
        assert!(
            AST_KERNEL_LABEL.contains("sb441-n373s"),
            "asteroid kernel label must name the n373s kernel"
        );
        assert!(
            !AST_KERNEL_LABEL.contains("sb441-n16"),
            "sb441-n16 must be fully retired"
        );
```

- [ ] **Step 4: Run the test**

Run: `cargo test -p pleiades-jpl --lib asteroid_kernel_sha_is_pinned_64_hex`
Expected: PASS (with the real `AST_SHA`).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/spk/corpus_spec.rs
git commit -m "feat(jpl): pin sb441-n373s asteroid kernel, retire sb441-n16 (slice 2)"
```

---

## Task 2: Flip roster tiers for confirmed bodies + update claim source

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/asteroid_roster.rs:78-108` (roster entries), `:149` (claim source string), `:216-227` (`promoted_goddesses_are_tier_a_main_belt`)
- Test: `crates/pleiades-jpl/src/spk/asteroid_roster.rs` (roster guard tests)

**Interfaces:**
- Consumes: `CONFIRMED_BODIES` from Task 0.
- Produces: each confirmed body's roster entry now `PinnedKernel`; `spk_body_claims` emits `CorpusValidated{ source: "sb441-n373s" }` for Tier-A.

- [ ] **Step 1: Flip the confirmed main-belt entries to `PinnedKernel`**

In `asteroid_core_roster()`, for each confirmed main-belt body change its tier `Constrained → PinnedKernel` **in place** (do not move the line). Worked example:

```rust
                e(ast("5-Astraea"), PinnedKernel, MainBelt),
                e(ast("6-Hebe"), PinnedKernel, MainBelt),
                e(ast("8-Flora"), PinnedKernel, MainBelt),
                e(ast("9-Metis"), PinnedKernel, MainBelt),
                e(ast("19-Fortuna"), PinnedKernel, MainBelt),
```

- [ ] **Step 2: Flip the confirmed TNO entries to `PinnedKernel`**

For each confirmed TNO, change `Constrained → PinnedKernel` in place (class stays `Tno`). Worked example:

```rust
                e(tno("136199-Eris"), PinnedKernel, Tno),
                e(tno("90377-Sedna"), PinnedKernel, Tno),
                e(tno("136108-Haumea"), PinnedKernel, Tno),
                e(tno("136472-Makemake"), PinnedKernel, Tno),
                e(tno("50000-Quaoar"), PinnedKernel, Tno),
                e(tno("90482-Orcus"), PinnedKernel, Tno),
                e(tno("225088-Gonggong"), PinnedKernel, Tno),
```

Leave bodies absent from the kernel (centaurs, NEAs, `80-Sappho`, `433-Eros`, `944-Hidalgo`, `1181-Lilith`, and any TNO not confirmed) as `Constrained`.

- [ ] **Step 3: Update the claim source string**

At line 149, change the Tier-A claim source:

```rust
                    ClaimEvidence::CorpusValidated {
                        source: "sb441-n373s".to_string(),
                    },
```

- [ ] **Step 4: Extend the main-belt promotion guard test**

Replace the `confirmed` array in `promoted_goddesses_are_tier_a_main_belt` to include the new main-belt bodies:

```rust
        let confirmed = ["15-Eunomia", "65-Cybele", "5-Astraea", "6-Hebe", "8-Flora", "9-Metis", "19-Fortuna"];
```

- [ ] **Step 5: Add a TNO promotion guard test**

After `promoted_goddesses_are_tier_a_main_belt`, add:

```rust
    #[test]
    fn promoted_tnos_are_tier_a_tno() {
        let confirmed = [
            "136199-Eris", "90377-Sedna", "136108-Haumea", "136472-Makemake",
            "50000-Quaoar", "90482-Orcus", "225088-Gonggong",
        ];
        for designation in confirmed {
            let e = asteroid_core_roster()
                .iter()
                .find(|e| matches!(&e.body, CelestialBody::Custom(c) if c.designation == designation))
                .unwrap_or_else(|| panic!("{designation} missing from roster"));
            assert_eq!(e.tier, AsteroidTier::PinnedKernel, "{designation} tier");
            assert_eq!(e.class, AsteroidClass::Tno, "{designation} class");
        }
    }
```

- [ ] **Step 6: Add a Tier-A claim-source guard test**

After the TNO test, add (confirms the source string flipped):

```rust
    #[test]
    fn tier_a_claims_cite_n373s() {
        use pleiades_backend::ClaimEvidence;
        let claims = spk_body_claims(&tier_a_bodies());
        assert!(!claims.is_empty());
        for c in &claims {
            match &c.evidence {
                ClaimEvidence::CorpusValidated { source } => {
                    assert_eq!(source, "sb441-n373s", "{:?} cites wrong source", c)
                }
                other => panic!("unexpected evidence {other:?}"),
            }
        }
    }
```

- [ ] **Step 7: Run the roster tests**

Run: `cargo test -p pleiades-jpl --lib asteroid_roster`
Expected: PASS — `promoted_goddesses_are_tier_a_main_belt`, `promoted_tnos_are_tier_a_tno`, `tier_a_claims_cite_n373s`, `tiers_are_disjoint_and_cover_roster`, `chiron_is_constrained_centaur` all green.

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-jpl/src/spk/asteroid_roster.rs
git commit -m "feat(jpl): promote n373s-confirmed asteroids+TNOs to Tier-A roster (slice 2)"
```

---

## Task 3: Teach the regen tool to filter the constrained slice

The promoted bodies must leave `asteroid_constrained.csv`. We filter the committed rows by current Tier-B membership (no Horizons re-fetch, so remaining Tier-B rows stay byte-identical) and emit the new manifest line. This logic goes into the existing maintainer binary so it stays a single reproducible step.

**Files:**
- Modify: `crates/pleiades-jpl/src/bin/regenerate-asteroid-corpus.rs`

**Interfaces:**
- Consumes: `tier_b_bodies()`, `corpus_checksum64`, the committed `asteroid_constrained.csv`.
- Produces: rewritten `asteroid_constrained.csv` (Tier-A bodies removed) + printed `slice asteroid_constrained …` manifest line.

- [ ] **Step 1: Add the constrained-filter constant and import**

Below `const OUT_PATH` (line 24) add:

```rust
const CONSTRAINED_PATH: &str = "crates/pleiades-jpl/data/corpus/asteroid_constrained.csv";
```

Extend the roster import on line 19 to also bring in `tier_b_bodies`:

```rust
use pleiades_jpl::spk::asteroid_roster::{asteroid_core_roster, tier_b_bodies, AsteroidTier};
```

- [ ] **Step 2: After writing the reference slice, filter the constrained slice**

Immediately before the final `Ok(())` (after the `wrote {OUT_PATH}` eprintln), insert:

```rust
    // Drop now-Tier-A bodies from the committed constrained slice without
    // re-fetching Horizons: keep header lines and rows whose body id is still
    // Tier-B, so remaining rows stay byte-identical.
    let tier_b: std::collections::HashSet<String> =
        tier_b_bodies().iter().map(|b| format!("{b}")).collect();
    let existing = std::fs::read_to_string(CONSTRAINED_PATH)
        .map_err(|e| format!("read {CONSTRAINED_PATH}: {e}"))?;
    let mut out = String::new();
    let mut kept_rows = 0usize;
    for line in existing.lines() {
        if line.starts_with('#') || line.is_empty() {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let body = line.split(',').nth(1).unwrap_or_default();
        if tier_b.contains(body) {
            out.push_str(line);
            out.push('\n');
            kept_rows += 1;
        }
    }
    std::fs::write(CONSTRAINED_PATH, &out)
        .map_err(|e| format!("write {CONSTRAINED_PATH}: {e}"))?;
    let c_checksum = corpus_checksum64(&out);
    eprintln!("\nfiltered {CONSTRAINED_PATH}: {kept_rows} rows, checksum={c_checksum}");
    println!(
        "slice asteroid_constrained file=asteroid_constrained.csv role=asteroid_constrained rows={kept_rows} checksum={c_checksum}"
    );
```

- [ ] **Step 3: Update the binary doc comment**

Replace the kernel name in the header doc comment (lines 1-16): `sb441-n16` → `sb441-n373s`, and note it now also filters the constrained slice. Change the usage example `PLEIADES_AST_KERNEL=/path/sb441-n16.bsp` → `.../sb441-n373s.bsp`.

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p pleiades-jpl --bin regenerate-asteroid-corpus`
Expected: builds clean (no warnings).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src/bin/regenerate-asteroid-corpus.rs
git commit -m "feat(jpl): regen tool filters promoted bodies out of constrained slice (slice 2)"
```

---

## Task 4: Regenerate corpora + update manifest + relabel validation corpus

**Files:**
- Modify: `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (regenerated), `crates/pleiades-jpl/data/corpus/asteroid_constrained.csv` (filtered), `crates/pleiades-jpl/data/corpus/manifest.txt` (two slice lines), `crates/pleiades-validate/src/corpus/mod.rs:388-389` (label)
- Test: `crates/pleiades-jpl/tests/corpus_regen.rs` (env-gated)

**Interfaces:**
- Consumes: Task 0 kernel, Task 1 pins, Task 2 roster, Task 3 tool.
- Produces: regenerated/filtered CSVs and matching manifest lines.

- [ ] **Step 1: Run the regen + filter (env-gated)**

```bash
PLEIADES_DE_KERNEL=.kernels/de440.bsp \
PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp \
  cargo run -p pleiades-jpl --bin regenerate-asteroid-corpus
```

Expected stdout: two manifest lines —
`slice asteroid_reference file=asteroid_reference.csv role=asteroid_reference rows=<N_ref> checksum=<C_ref>`
`slice asteroid_constrained file=asteroid_constrained.csv role=asteroid_constrained rows=<N_con> checksum=<C_con>`
Expected: the `Tier A … supported_by_kernel=true` line prints for every Tier-A body (including the newly promoted ones). If any shows `false`, the kernel/NAIF id is wrong — stop and recheck Task 0/Task 2.

- [ ] **Step 2: Confirm the reference CSV header now names n373s**

Run: `head -3 crates/pleiades-jpl/data/corpus/asteroid_reference.csv`
Expected: `#Source: JPL DE small-body perturber kernel: sb441-n373s.bsp` and the new `#Kernel-SHA256:` matching `AST_SHA`.

- [ ] **Step 3: Confirm promoted bodies moved slices**

```bash
grep -v '^#' crates/pleiades-jpl/data/corpus/asteroid_constrained.csv | cut -d, -f2 | sort -u
grep -v '^#' crates/pleiades-jpl/data/corpus/asteroid_reference.csv | cut -d, -f2 | sort -u
```

Expected: each confirmed body appears in `asteroid_reference.csv` and **not** in `asteroid_constrained.csv`; every remaining Tier-B body still appears in `asteroid_constrained.csv`.

- [ ] **Step 4: Update the manifest**

In `crates/pleiades-jpl/data/corpus/manifest.txt`, replace the `slice asteroid_reference …` and `slice asteroid_constrained …` lines with the two lines printed in Step 1 (exact rows + checksum).

- [ ] **Step 5: Relabel the validation corpus**

In `crates/pleiades-validate/src/corpus/mod.rs:388-389` replace `sb441-n16` with `sb441-n373s` in both the `name` and `description`:

```rust
                name: "Tier-A asteroid reference window (sb441-n373s)".to_string(),
                description: "Asteroid accuracy-ceiling corpus built from the committed sb441-n373s Tier-A reference rows, used by the slow release-grade accuracy audit for small bodies.",
```

- [ ] **Step 6: Run the env-gated regen reproducibility test**

```bash
PLEIADES_DE_KERNEL=.kernels/de440.bsp \
PLEIADES_AST_KERNEL=.kernels/sb441-n373s.bsp \
  cargo test -p pleiades-jpl --test corpus_regen -- --nocapture
```

Expected: both `regenerated_corpus_matches_checked_in` and `regenerated_asteroid_reference_matches_checked_in` PASS (rows match, all coords < 1.0 km drift).

- [ ] **Step 7: Run the corpus validation gate**

Run: `cargo test -p pleiades-validate corpus`
Expected: PASS — window/schema/finiteness checks green over moved/regenerated rows.

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-jpl/data/corpus/asteroid_reference.csv \
        crates/pleiades-jpl/data/corpus/asteroid_constrained.csv \
        crates/pleiades-jpl/data/corpus/manifest.txt \
        crates/pleiades-validate/src/corpus/mod.rs
git commit -m "feat(jpl): regenerate asteroid corpus from sb441-n373s, filter promoted bodies (slice 2)"
```

---

## Task 5: Align claim-surface docs (sourcing, README, PLAN)

**Files:**
- Modify: `docs/spk-kernel-sourcing.md` (kernel section + citations), `README.md:28`, `PLAN.md` (limits + status line)
- Test: `cargo test -p pleiades-jpl --lib reference_summary::selected_asteroid` (counts are dynamic — verify, don't hardcode)

**Interfaces:**
- Consumes: `CONFIRMED_BODIES`, final Tier-A count = 9 + |CONFIRMED_BODIES|, Tier-B count = 27 − |CONFIRMED_BODIES|.

- [ ] **Step 1: Rewrite the asteroid-kernel section of `docs/spk-kernel-sourcing.md`**

Replace the "Asteroid kernel (Tier A — pinned)" section (lines 23-56) so it: names `sb441-n373s.bsp`, its URL (`https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n373s.bsp`), the recorded SHA-256, the size (~937 MB), the body-set description (343 main-belt perturbers + 30 KBOs, DE441-consistent; supersedes the retired 16-body `sb441-n16`), the confirmed verified window, and the updated regen recipe (`PLEIADES_AST_KERNEL=…/sb441-n373s.bsp`). Add a "Astrological usage (gate 2 — promoted bodies)" subsection with one cited line per promoted body, e.g.:

```markdown
  - 5 Astraea: Greek goddess of justice (Astraea/Dike); listed in the Swiss
    Ephemeris asteroid name catalog (`seasnam.txt`, Astrodienst/astro.com) and
    used in the asteroid-astrology interpretive tradition (cf. Martha
    Lang-Wescott, *Mechanics of the Future: Asteroids*).
  - 6 Hebe: goddess of youth, cupbearer to the gods; same catalog + tradition.
  - 8 Flora: goddess of flowers and spring; same catalog + tradition.
  - 9 Metis: Titaness of wisdom/counsel, first wife of Zeus; same catalog + tradition.
  - 19 Fortuna: Roman goddess of fortune/luck; same catalog + tradition.
  - 136199 Eris: goddess of strife/discord; widely used in modern outer-body
    astrology; same catalog + tradition.
  - 90377 Sedna: Inuit sea goddess; widely used in modern TNO astrology.
  - 136108 Haumea / 136472 Makemake / 50000 Quaoar / 90482 Orcus / 225088 Gonggong:
    creation/underworld deities used in modern TNO astrology; same catalog.
```

(Write a line only for each body actually in `CONFIRMED_BODIES`.) Move the now-constrained-only note: centaurs/NEAs/personal main-belt/unconfirmed TNOs remain Tier-B (Horizons).

- [ ] **Step 2: Update `README.md:28`**

Replace "the nine `sb441-n16` Tier-A asteroids (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris, Eunomia, Cybele)" with the new count, kernel name, and full body list — e.g. for the expected set (16 bodies): "the sixteen `sb441-n373s` Tier-A asteroids/TNOs (Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris, Eunomia, Cybele, Astraea, Hebe, Flora, Metis, Fortuna, plus TNOs Eris, Sedna, Haumea, Makemake, Quaoar, Orcus, Gonggong)". Use the actual confirmed set and count.

- [ ] **Step 3: Update `PLAN.md`**

In "Important current limits" (the per-backend claims bullet, ~line 62-65) and the bottom Status line (~line 191): change the Tier-A kernel from `sb441-n16` to `sb441-n373s`, update the release-grade asteroid count and body list, the Tier-B count (27 − promoted), and add a one-line slice-2 record (kernel swap + promoted bodies + that `sb441-n16` is retired). Keep within the plan-maintenance rules (no per-alias noise).

- [ ] **Step 4: Verify the dynamic report still passes**

Run: `cargo test -p pleiades-jpl --lib reference_summary::selected_asteroid`
Expected: PASS. `selected_asteroid_constrained_class_report()` derives counts from `tier_a_bodies().len()`/`tier_b_bodies().len()`, so no code edit is needed — this confirms the counts flow through.

- [ ] **Step 5: Commit**

```bash
git add docs/spk-kernel-sourcing.md README.md PLAN.md
git commit -m "docs: record sb441-n373s swap + slice-2 asteroid promotions"
```

---

## Task 6: Whole-workspace verification

**Files:** none (verification only).

- [ ] **Step 1: Full test suite**

Run: `cargo test --workspace`
Expected: PASS workspace-wide. (Env-gated kernel tests skip cleanly without the env vars; run Task 4 Step 6 separately with the kernel for the gated path.)

- [ ] **Step 2: Claims audit**

Run: `cargo test -p pleiades-validate claims`
Expected: PASS — every Tier-A body (including promoted) holds the `BodyClass::Asteroid` ceiling; sources cite `sb441-n373s`.

- [ ] **Step 3: Release smoke + gate**

Run the project's release-smoke / release-gate commands (per `plan/checklists/01-phase-gates.md`).
Expected: green — full numeric-gate set + overclaim audit pass with the new kernel label and counts.

- [ ] **Step 4: Grep for residual `sb441-n16`**

Run: `grep -rn "sb441-n16\|sb441_n16" crates/ docs/ README.md PLAN.md`
Expected: no matches outside historical spec/plan files under `docs/superpowers/specs/` and `docs/superpowers/plans/` (the slice-1 records). If any live code/doc surface still names n16, fix it.

- [ ] **Step 5: Final commit (if Step 4 required fixes)**

```bash
git add -A
git commit -m "chore(jpl): scrub residual sb441-n16 references (slice 2)"
```

---

## Self-Review

**Spec coverage:**
- Kernel swap (spec §2) → Task 1 + Task 4 (header/label) + Task 5 (docs).
- Selection policy / DAF membership (spec §1, §3) → Task 0.
- Roster tier flips + claim source (spec §4) → Task 2.
- Reference-CSV regen + constrained-CSV filter-don't-refetch (spec §4) → Task 3 + Task 4.
- Validation gates reuse (spec §5) → Task 4 (corpus_regen, validate-corpus) + Task 6 (claims-audit, release gates).
- Claim-surface alignment (spec §6) → Task 5 + Task 6 Step 4 grep.
- Acceptance criteria → covered across Tasks 4–6 (regen <1km, accuracy ceiling, slice membership, counts consistency, full green, PLAN record).
- Risks: n373s window → Task 0 Step 4 fallback; membership uncertainty → Task 0; kernel-free CI → env-gating throughout; value drift → Task 4 Step 6 tolerance; Tier-B filtering correctness → Task 3 (filter, not refetch) + Task 4 Step 3.

**Placeholder scan:** the only intentional fill-in is `<AST_SHA from Task 0>` (the real digest is unknowable until the kernel is downloaded) and `CONFIRMED_BODIES` (DAF-determined) — both are explicitly produced by Task 0 and flagged at each use, matching the spec's "pinned at implementation" pattern. No vague "add error handling"/"write tests"-style gaps.

**Type consistency:** `AsteroidTier::PinnedKernel`, `AsteroidClass::{MainBelt,Tno}`, `ClaimEvidence::CorpusValidated{source}`, `tier_a_bodies()`/`tier_b_bodies()`, `spk_body_claims()`, `corpus_checksum64()`, `generate_slice()`, `SliceRole::{AsteroidReference,AsteroidConstrained}` all match the signatures read from the source files.
