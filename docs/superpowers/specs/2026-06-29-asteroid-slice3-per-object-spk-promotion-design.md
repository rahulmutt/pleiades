# Asteroid Per-Object Pinned-SPK Release-Grade Promotion (Phase 6, asteroid slice 3)

- **Status:** design approved, pending implementation plan
- **Date:** 2026-06-29
- **Phase:** PLAN.md Phase 6 — target catalog completion and expansion
- **Crates:** `pleiades-jpl` (roster, corpus_spec/manifest, regen, reference
  summaries), `pleiades-validate` (corpus/claims gates), plus claim-surface
  docs/reports

## Goal

Raise the astrologically-used, **kernel-absent** Tier-B asteroids from
`Constrained` to `ReleaseGrade` by sourcing each from its **own JPL SBDB
per-object SPK** (a `.bsp` pinned by SHA), regenerating its committed corpus rows
byte-reproducibly, and promoting it through the *same* accuracy and
astrological-usage gates as slices 1–2 — with no gate weakening and no API
change.

This advances the Phase 6 bullet:

> Expand selected-asteroid coverage beyond Ceres/Pallas/Juno/Vesta where source
> evidence and backend metadata support release-grade claims.
> — `plan/stages/06-catalog-completion-and-expansion.md`

## Why this reframing (from brainstorming)

The slice-1/2 mechanism — flip a roster entry `Constrained → PinnedKernel` and
regenerate it from the single pinned `sb441-n373s` kernel — **cannot reach the
remaining 11 Tier-B bodies.** They are Tier-B precisely because they are *not in
any bundled perturber kernel*: `sb441-n373s` is 343 main-belt perturbers + 30
KBOs, and the 5 centaurs and 6 personal/minor main-belt/NEA bodies are absent
from it (`docs/spk-kernel-sourcing.md`, "Tier B — Horizons-sourced" notes this
explicitly). Their Horizons-sourced data is window/schema/provenance-validated
but **not byte-reproducible**, which is exactly what keeps them below
release-grade.

The honest way to raise their quality is the asteroid analogue of "source the
evidence, then claim it": give each body a reproducible, pinned source it does
have — a **per-object JPL SBDB SPK** — then promote it through the gates that
already justify the existing 25 Tier-A asteroids. The user's intent is to cover
**every astrologically-relevant body**, so the eligible set is the whole Tier-B
roster (already the curated astrologically-relevant core), gated per-body rather
than a hand-picked subset.

## Scope decisions (from brainstorming)

| Decision | Choice |
| --- | --- |
| Reproducible source | **Per-object pinned SPK** — one JPL SBDB `.bsp` per body, SHA-pinned. (Rejected: hardening Horizons provenance — never byte-reproducible, stays below release-grade; documenting as permanent known gaps — the user wants promotion.) |
| Body scope | **All 11 Tier-B bodies are candidates**, gated per-body. The gate *is* astrological relevance (plus an obtainable SPK and the accuracy ceiling); roster membership already encodes relevance. Honest split outcome: any body failing a gate stays Tier-B. |
| Packaging | **Approach B — per-object `.bsp` files, uncommitted, pinned by SHA in a manifest.** (Rejected: a self-merged bundle (weaker self-assembled provenance); committing the files (departs from the repo's "kernels never committed" policy — kept as a noted runner-up for CI-enforced reproduction).) |
| Tier model | **`AsteroidTier` stays the grade discriminator**; a per-body `source` field carries the evidence source string. (Rejected: a new `PinnedObjectSpk` tier variant — splits the grade concept and still needs a per-body label.) |
| Roster pinning | **Final promoted set pinned at implementation** by the three-gate policy; no body promoted from memory. |

## Non-goals

- **No gate-2 weakening / no force-promotion.** A body with no citable
  astrological tradition, no obtainable SPK, or a failing accuracy check is **not**
  promoted; it stays Tier-B. Honest split outcome.
- **No `CelestialBody` enum change.** Every promoted body keeps its existing
  `asteroid:NN-Name` / `tno:NNNNN-Name` `Custom` catalog id.
- **No new accuracy ceiling, backend, or boundary change.** The existing
  `BodyClass::Asteroid` ceiling (30″ lon/lat, 5e6 km) and the
  mean/geometric/geocentric/tropical backend boundary are unchanged. Coverage
  stays 1900–2100 CE, fail-closed outside it.
- **No change to `sb441-n373s`** or the existing 25 Tier-A bodies — their pin,
  rows, and claims are untouched.
- **No Horizons re-fetch** of the remaining constrained slice — promoted rows are
  filtered out of the committed CSV; the rest stay byte-identical.
- **No calculated points / fictional bodies.** Black Moon Lilith (the calculated
  lunar-apogee point) remains out of scope and is distinct from the numbered
  asteroid 1181 Lilith considered here.

## Background: why this stays a focused change

- **Multi-kernel is already supported.** `regenerate-asteroid-corpus` builds one
  `SpkBackend` via `.add_kernel(&de).add_kernel(&ast)`; per-object SPKs are just
  additional `.add_kernel(...)` calls — no chain-resolution change.
- **NAIF-id resolution already covers all 11.** `chain::naif_ids` parses the
  leading IAU number of a `Custom` id into both `2_000_000+n` and `20_000_000+n`
  candidates; the `every_roster_body_resolves_to_a_naif_id` test already passes
  for every Tier-B body.
- **Tier is the grade lever.** `AsteroidTier::{PinnedKernel, Constrained}`
  (`asteroid_roster.rs`) drives `spk_body_claims`: `PinnedKernel` →
  `ReleaseGrade/High/CorpusValidated`, `Constrained` →
  `Constrained/Moderate/CorpusValidated{"horizons"}`. Flipping the tier flips the
  grade automatically.
- **Counts are mostly dynamic.** `selected_asteroid.rs` reports via
  `tier_a_bodies().len()` / `tier_b_bodies().len()`; the roster size test tolerates
  the unchanged total (36). The churn is prose lists (README/PLAN/docs) + a few
  membership tests, not pervasive hard-coded counts.
- **The gates already fail closed.** The env/dir-gated `corpus_regen` test skips
  kernel-free and otherwise reproduces each row within ~1 km; `claims-audit` holds
  release-grade bodies to the `BodyClass::Asteroid` ceiling; `validate-corpus`
  enforces window/schema/finiteness.

## Design

### 1. Selection policy

A Tier-B body is promoted to Tier-A release-grade **only if all three hold**, each
verified at implementation:

1. **Obtainable pinned SPK** — a JPL SBDB SPK (`.bsp`) covering 1900–2100 exists
   for the body's NAIF id and is pinned by SHA-256 + recorded request params.
2. **Documented astrological usage** — a cited tradition (Swiss Ephemeris
   `seasnam.txt` catalog membership plus an interpretive source — e.g. the
   centaur-astrology literature for the centaurs, Martha Lang-Wescott's
   *Mechanics of the Future: Asteroids* for the personal/minor bodies), the same
   gate-2 standard as slices 1–2. Each promoted body gets an explicit citation in
   `docs/spk-kernel-sourcing.md`.
3. **Accuracy ceiling** — clears the existing `BodyClass::Asteroid` ceiling (30″
   lon/lat, 5e6 km) under the regen/claims gate.

Any body failing any gate **stays Tier-B**. The final promoted set is pinned at
implementation by this policy, mirroring how slice 2 pinned its set against the
kernel.

**Eligible set (all candidates, gated per-body):**

- **Centaurs (5):** `asteroid:2060-Chiron`, `asteroid:5145-Pholus`,
  `asteroid:7066-Nessus`, `asteroid:10199-Chariklo`, `asteroid:8405-Asbolus`.
- **Personal / minor main-belt / NEA (6):** `asteroid:1221-Amor`,
  `asteroid:1181-Lilith`, `asteroid:944-Hidalgo`, `asteroid:1566-Icarus`,
  `asteroid:1685-Toro`, `asteroid:1862-Apollo`.

### 2. Per-object SPK sourcing + manifest

- **Source:** for each promoted body, generate a JPL SBDB per-object SPK over
  1900–2100 (NAIF id from the existing `2_000_000+n` / `20_000_000+n` scheme).
  Exact endpoint and request parameters are verified and recorded at
  implementation (the same verification-at-implementation discipline slice 2 used
  for kernel membership). Public domain (U.S. Government work).
- **Manifest (committed):** a new committed manifest — per-object SPK constants in
  `corpus_spec.rs` and/or a `data/object_spk_manifest` file — records, per body:
  NAIF id, SHA-256, SBDB request params, solution epoch, file size, verified
  coverage window. This is the CI-visible provenance; the `.bsp` files themselves
  stay **uncommitted** (pinned-by-SHA, exactly like `sb441-n373s` and `de440`).
- **Docs:** `docs/spk-kernel-sourcing.md` gains a "Tier A — per-object pinned SPK"
  section (per-body source URL/request, SHA, size, window, astrological-usage
  citation, regen recipe). The Tier-B section is decremented to the
  still-constrained remainder.

### 3. Data-model change (the one real code change)

Today `spk_body_claims` hard-codes `source = "sb441-n373s"` for every
`PinnedKernel` body. Promoted bodies are also release-grade but sourced from a
different file, so the source must become **per-body**:

- Add a `source: &'static str` field to `AsteroidEntry`. Values: `"sb441-n373s"`
  for the existing kernel bodies; a per-object label (e.g. `"jpl-sbdb-spk:2060"`)
  for promoted bodies; `"horizons"` for the remaining constrained bodies.
- `AsteroidTier` stays the **grade** discriminator (`PinnedKernel` =
  release-grade; `Constrained` = constrained). `spk_body_claims` reads
  `entry.source` for the evidence string instead of branching tier → hard-coded
  string.
- The `tier_a_claims_cite_n373s` test relaxes to "each Tier-A body cites its
  declared source" (n373s bodies still cite `sb441-n373s`; promoted bodies cite
  their per-object label).

*Alternative considered:* a new `AsteroidTier::PinnedObjectSpk` variant —
rejected; it splits the single grade concept and still requires a per-body source
label. The `source` field is the smaller, more future-proof change.

### 4. Regen path

- `regenerate-asteroid-corpus` currently reads `PLEIADES_DE_KERNEL` +
  `PLEIADES_AST_KERNEL` and `.add_kernel`s each. Extend it to also read a
  per-object kernels directory (`PLEIADES_OBJECT_SPK_DIR`), iterate the manifest,
  and `.add_kernel` each present per-object `.bsp` alongside de440 + n373s.
- The `corpus_regen` integration test gains the same dir-gated path: directory
  present → regenerate each promoted body's rows and compare within the existing
  ~1 km tolerance; absent → early-return skip (kernel-free), exactly as the
  existing Tier-A path. No change to the single-backend, multi-kernel chain
  resolution.

### 5. Corpus & data changes

- **`asteroid_reference.csv` (Tier-A) — regenerate with promoted rows.** Re-run
  the regen with de440 + n373s + the per-object SPKs. The existing 25 Tier-A
  bodies stay **byte-identical** (same kernels, same recipe); promoted bodies join
  at their class cadence (`AsteroidClass::max_gap_days`: main-belt 180 d, centaur
  365 d). New checksum + manifest line.
- **`asteroid_constrained.csv` (Tier-B) — filter, do not re-fetch.** Drop the
  promoted bodies' rows by body id. Remaining Tier-B rows stay **byte-identical**
  (no Horizons re-fetch, so the non-reproducible slice is not perturbed). If all
  11 promote, this file becomes header-only and `tier_b_bodies()` becomes empty —
  handled gracefully (the `Constrained` claim branch and the reports stay valid,
  just empty).
- **Roster (`asteroid_roster.rs`):** flip each promoted entry
  `Constrained → PinnedKernel` **in place** (preserve stable order —
  checksums/reports depend on it) and set its `source`.

### 6. Validation gates (reuse, do not invent)

- **Regen:** promoted bodies join the dir-gated `corpus_regen` path and reproduce
  each committed row within the existing ~1 km tolerance against their pinned SPK;
  a clean checkout still skips kernel-free.
- **Accuracy ceiling:** `claims-audit`'s release-grade-accuracy check holds each
  promoted body to the existing `BodyClass::Asteroid` ceiling (30″ lon/lat,
  5e6 km). Any candidate that misses is dropped from the slice rather than shipped
  over-claimed.
- **Corpus validation:** `validate-corpus` / `corpus/asteroid.rs` window + schema
  + finiteness checks cover the moved/regenerated rows with no change.

### 7. Cadence note (decided at implementation)

Centaurs already use the 365 d centaur cadence. The NEAs (Icarus, Toro, Apollo,
Amor) move fast near perihelion, so the 180 d MainBelt **validation** cadence may
sample too coarsely. The committed corpus is reference *sample points* — the
backend interpolates the SPK directly at runtime, not the CSV — so this affects
validation density, not runtime accuracy. Whether to give the eccentric NEAs a
finer cadence (or a `NearEarth` class) is decided at implementation; it does not
change the backend boundary.

### 8. Claim-surface alignment

Every surface that enumerates or counts the Tier-A / Tier-B asteroid sets is
updated in the same change:

- `README.md` — Tier-A count/body list (now includes promoted centaurs/NEAs) and
  the Tier-B sentence (decremented, or removed if the constrained slice empties).
- `docs/spk-kernel-sourcing.md` — new per-object-SPK Tier-A section (manifest,
  per-body SHA/request, citations); Tier-B section decremented.
- `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` and
  `production_generation.rs` — counts are dynamic
  (`tier_a_bodies().len()`/`tier_b_bodies().len()`), so they follow
  automatically; verify the prose label text.
- `crates/pleiades-validate/src/corpus/mod.rs` — any Tier-A/Tier-B window label or
  description text.
- Tests: roster guard tests (the `promoted_goddesses_*`/`promoted_tnos_*`-style
  lists gain the promoted bodies; `chiron_is_constrained_centaur` flips to a
  release-grade assertion if Chiron promotes), `tier_a_claims_cite_n373s` relaxes
  to per-body source, `corpus_spec` manifest/pin tests (new per-object SHA pins
  format-checked as 64-hex, mirroring the n373s pin test).
- `PLAN.md` — record the slice (mechanism, promoted set, new Tier-A / Tier-B
  counts) per the plan-maintenance rule.

**Compatibility profile:** the rendered compatibility profile is house/ayanamsa
scoped; whether it embeds asteroid body claims or a kernel label is confirmed
during implementation. If it does, a patch version bump follows (mirroring slice-4
hygiene); if not, no bump.

### 9. Error handling / behavior

No runtime behavior change for consumers beyond (a) the claim tier the backend
reports for promoted bodies (`Constrained → ReleaseGrade`) and (b) the source of
their reference rows (Horizons → per-object SPK). The backend boundary stays
mean/geometric/geocentric/tropical; coverage stays 1900–2100 CE with fail-closed
behavior outside it.

## Acceptance criteria

- Each promoted body is `ReleaseGrade` in `spk_body_claims`, cites its per-object
  SPK source, and reproduces each committed row within ~1 km under the dir-gated
  `corpus_regen` path.
- Each promoted body passes the `BodyClass::Asteroid` accuracy ceiling via
  `claims-audit`, and carries a cited astrological-usage source in
  `docs/spk-kernel-sourcing.md`.
- Each promoted body's per-object SPK is recorded in the committed manifest (NAIF
  id, SHA-256, request params, solution epoch, window); the SHA pins are
  format-validated (64-hex) by test.
- Promoted bodies no longer appear in `asteroid_constrained.csv`; they appear
  once, in `asteroid_reference.csv`. Remaining Tier-B rows are byte-identical to
  before (no Horizons re-fetch); the dropped set matches the promoted set exactly.
- The existing 25 Tier-A bodies and the `sb441-n373s` pin are unchanged; their
  committed rows stay byte-identical.
- Any Tier-B body that fails a gate stays `Constrained` with its claim unchanged
  (honest split outcome documented).
- The Tier-A / Tier-B asteroid counts, body lists, and source labels are
  consistent across `README.md`, the production/selected-asteroid reports,
  roster/spec/claims tests, `docs/spk-kernel-sourcing.md`, and `PLAN.md`.
- An empty Tier-B set (if all 11 promote) is handled without panics by claims,
  reports, and validation.
- `cargo test` is green workspace-wide; `validate-corpus`, `claims-audit`,
  `release-smoke`, and `release-gate` re-run green.
- `PLAN.md` records the slice; the public `Ayanamsa`/`CelestialBody` enums are
  unchanged.

## Risks & mitigations

- **SBDB SPK availability / endpoint.** The exact endpoint and parameters are not
  hard-coded from memory. *Mitigation:* verified and recorded at implementation; a
  body with no obtainable SPK stays Tier-B.
- **Upstream re-fit drift.** JPL re-fits per-object solutions as observations
  accumulate, so (unlike the static NAIF-FTP `sb441-n373s`) a future re-download
  may differ. *Mitigation:* pin each file by SHA, archive it, and record the SBDB
  request + solution epoch; reproduction is "from the pinned file," the same model
  as `sb441-n373s` and `de440`.
- **Centaur chaotic orbits** (Chiron and others have Saturn close approaches).
  *Mitigation:* over 1900–2100, anchored to modern astrometry, the solution easily
  clears the 30″ ceiling; any candidate that does not stays Tier-B.
- **NEA fast perihelion motion.** *Mitigation:* the runtime interpolates the SPK
  directly, so this is a validation-cadence question only; a finer cadence /
  `NearEarth` class is decided at implementation.
- **Empty Tier-B slice** if all 11 promote. *Mitigation:* ensure empty
  `tier_b_bodies()` / header-only constrained CSV is handled by claims, reports,
  and validation without panics; assert it explicitly.
- **Per-object SPK trust without SHA verification at regen.** Like `sb441-n373s`,
  the SHA is pinned but not cryptographically verified at regen (only
  format-checked). *Mitigation:* unchanged from the existing model; the committed
  rows + checksums are the CI-visible evidence, and the manifest records the SHA
  for manual verification.

## Open questions

None blocking. The final promoted set is pinned at implementation by the
three-gate selection policy (§1). The NEA validation cadence (§7) and the
manifest's exact on-disk form (§2) are settled during implementation.
