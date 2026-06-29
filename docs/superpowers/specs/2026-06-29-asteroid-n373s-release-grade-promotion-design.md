# Asteroid sb441-n373s Kernel Swap + Release-Grade Promotion (Phase 6, asteroid slice 2)

- **Status:** design approved, pending implementation plan
- **Date:** 2026-06-29
- **Phase:** PLAN.md Phase 6 — target catalog completion and expansion
- **Crates:** `pleiades-jpl` (roster, corpus_spec, regen, reference summaries),
  `pleiades-validate` (corpus/claims gates), plus claim-surface docs/reports

## Goal

Retire the 16-body `sb441-n16` perturber kernel and pin the 373-body
`sb441-n373s` kernel (343 main-belt perturbers + 30 KBOs, fitted consistently
with DE441) as the **single** asteroid reference kernel, then promote every
astrologically-used roster body it newly contains from **Tier-B/Constrained** to
**Tier-A/ReleaseGrade** via the *existing* kernel-pinned + `corpus_regen` numeric
gate — the same rigor as slice 1, with no weakening of the astrological-usage
gate and no API change.

This advances the Phase 6 bullet:

> Expand selected-asteroid coverage beyond Ceres/Pallas/Juno/Vesta where source
> evidence and backend metadata support release-grade claims.
> — `plan/stages/06-catalog-completion-and-expansion.md`

## Why this reframing (from brainstorming)

The obvious "slice 2" — promote the remaining `sb441-n16` members — was rejected.
`sb441-n16` holds exactly 16 minor planets (verified by DAF inspection:
`{1,2,3,4,7,10,15,16,31,52,65,87,88,107,511,704}`). Nine are already Tier-A; the
seven remaining (Euphrosyne, Europa, Sylvia, Thisbe, Camilla, Davida,
Interamnia) are **massive perturbers chosen for gravitational effect, not
astrological use** — Davida (named after astronomer David P. Todd) and Interamnia
(named after the city of Teramo) have no astrological tradition at all. Promoting
them would mean stretching the astrological-usage gate to dress up
dynamically-massive rocks as release-grade — exactly the overclaim Phase 6 must
avoid.

Meanwhile the asteroids astrologers actually use — **Astraea (5), Hebe (6),
Flora (8), Metis (9), Fortuna (19)** plus several massive TNOs — are **absent
from `sb441-n16`** and are currently committed only as Tier-B/Horizons
constrained data. The honest way to raise their quality is not to weaken a gate
but to **source the kernel that contains them.** `sb441-n373` is a strict
superset of `sb441-n16` (top-343 by mass + 30 KBOs), so it contains the wanted
main-belt bodies (easily top-343 even though they miss the top-16) and the
massive TNOs (the 30-KBO set). Its abbreviated build `sb441-n373s` is 937 MB —
comparable to the 616 MB `sb441-n16` already in use — making it practical as a
drop-in replacement.

This is the asteroid analogue of "source the evidence, then claim it": pin the
kernel that holds the wanted bodies, then promote them through the same gate that
already justifies the existing nine Tier-A asteroids.

## Scope decisions (from brainstorming)

| Decision | Choice |
| --- | --- |
| Direction | **Raise the quality of the astrologically-wanted, kernel-absent asteroids** — not a kernel-completeness promotion of `sb441-n16`'s remaining members. |
| Kernel approach | **Approach 1A: pin a larger kernel for true release-grade**, not Horizons-hardening (which can never reach byte-reproducible-from-pinned-kernel and would stay honestly below release-grade). |
| Kernel relationship | **Replace `sb441-n16` entirely with `sb441-n373s`** as the single pinned asteroid kernel (n373 ⊇ n16, so all current Tier-A bodies remain covered by one source). |
| Body scope | **Rule-driven:** promote every current roster body whose NAIF id is confirmed present in `sb441-n373s`. The roster is already the curated astrologically-relevant core, so gate 2 is satisfied by roster membership; the kernel-membership inspection determines the exact promoted set. |
| Roster pinning | **Policy + expected set now; final body set pinned at implementation** against the actual downloaded kernel's segment table. No designation hard-coded from memory. |

## Non-goals

- **No gate-2 weakening / no kernel-completeness promotion.** The remaining
  `sb441-n16`/`n373` members with no astrological tradition (Davida, Interamnia,
  Sylvia, Thisbe, Camilla, Europa, Euphrosyne, and the long tail of the 343
  perturbers not already in the curated roster) are **not** added or promoted.
  Only bodies *already in the curated roster* are eligible.
- **Bodies absent from `sb441-n373s` stay constrained.** Centaurs (Chiron,
  Pholus, Nessus, Chariklo, Asbolus), NEAs and small main-belt personal
  asteroids (Eros, Sappho, Lilith, Amor, Hidalgo, Icarus, Toro, Apollo), and any
  TNO not in the 30-KBO set remain Tier-B/Horizons. This slice must not broaden
  their claims.
- **No `CelestialBody` enum change.** Every promoted non-classical body keeps its
  existing `asteroid:NN-Name` / `tno:NNNNN-Name` `Custom` catalog id.
- **No new accuracy ceiling, backend, or boundary change.** The existing
  `BodyClass::Asteroid` ceiling and the mean/geometric/geocentric/tropical
  backend boundary are unchanged. Coverage stays 1900–2100 CE, fail-closed
  outside it.
- **No second asteroid kernel.** `sb441-n16` is fully retired; `sb441-n373s` is
  the sole pinned asteroid kernel. No parallel two-kernel maintenance.
- **No calculated points / fictional bodies** (unchanged from prior asteroid
  designs).

## Background: why this stays a focused change, not new infrastructure

- **Tier is the only lever.** `AsteroidTier::{PinnedKernel, Constrained}`
  (`crates/pleiades-jpl/src/spk/asteroid_roster.rs:13`) drives everything:
  `spk_body_claims` (`asteroid_roster.rs:136`) emits
  `ReleaseGrade/High/CorpusValidated{<source>}` for `PinnedKernel` bodies and
  `Constrained/Moderate/CorpusValidated{"horizons"}` for `Constrained` bodies.
  Flipping a roster entry's tier flips its claim automatically.
- **Kernel identity is centralized.** The asteroid kernel is pinned in exactly
  one place — `corpus_spec.rs:82-84` (`AST_KERNEL_LABEL`, `AST_KERNEL_SHA256`) —
  consumed by `generate.rs:314,324,407`. Swapping kernels = editing two
  constants + the provenance doc.
- **The regen path is roster-driven.** `regenerate-asteroid-corpus`
  (`src/bin/regenerate-asteroid-corpus.rs`) filters by `AsteroidTier::PinnedKernel`
  and reads the kernel from `PLEIADES_AST_KERNEL`; promoting bodies = flipping
  their tier, with no recipe logic change (only the doc-comment kernel name).
- **The gates already fail closed.** `validate-corpus` /
  `crates/pleiades-validate/src/corpus/asteroid.rs` enforce the 1900–2100 window,
  schema, and finiteness; the `claims-audit` release-grade-accuracy check holds
  release-grade bodies to the `BodyClass::Asteroid` ceiling
  (`crates/pleiades-data/src/thresholds.rs` — 30″ lon/lat, 5e6 km).
- **The promoted bodies are already committed**, as Tier-B Horizons data
  (`asteroid_roster.rs` Tier-B entries, `data/corpus/asteroid_constrained.csv`),
  so each promotion is a re-source (Horizons → n373s) + re-tag, not a new body.

## Design

### 1. Selection policy

A body is promoted to Tier-A release-grade **only if both** conditions hold, both
verified at implementation time:

1. **Kernel membership** — its NAIF id is confirmed present in `sb441-n373s.bsp`
   by parsing the downloaded kernel's DAF segment table (the same parse used to
   enumerate `sb441-n16`). Never asserted from memory.
2. **Documented astrological usage** — already established by membership in the
   curated roster (`asteroid_roster.rs` is "the curated core of
   astrologically-relevant minor planets"). Each promoted body gets an explicit
   cited tradition recorded in `docs/spk-kernel-sourcing.md`, mirroring slice 1.

Because only existing roster bodies are eligible, **gate 1 (kernel membership) is
the sole discriminator** for this slice. A roster body absent from `sb441-n373s`
stays Tier-B; a `n373s` member that is *not* in the curated roster is not added.

### 2. Kernel swap

- **Source:** `sb441-n373s.bsp` from
  `https://ssd.jpl.nasa.gov/ftp/eph/small_bodies/asteroids_de441/sb441-n373s.bsp`
  (public domain, U.S. Government work; 937 MB; 343 main-belt perturbers + 30
  KBOs; DE441-consistent, agrees with de440 over the overlap).
- **Pin:** update `corpus_spec.rs`:
  - `AST_KERNEL_LABEL` → `"JPL DE small-body perturber kernel: sb441-n373s.bsp"`.
  - `AST_KERNEL_SHA256` → the `shasum -a 256 sb441-n373s.bsp` digest (recorded at
    implementation; the existing 64-hex pin test stays).
- **Window verification (prerequisite):** confirm by DAF inspection that
  `sb441-n373s` covers the 1900–2100 asteroid window. The file size implies an
  ~1,100-year span, which should cover it, but this is verified, not assumed. If
  the abbreviated window is too narrow, fall back to the full 14 GB
  `sb441-n373.bsp` (same body set, same pins otherwise).
- **Retirement:** `sb441-n16` is removed from all sourcing docs, labels, and
  descriptions. It is no longer a project dependency.

### 3. Expected promoted set (final set pinned at implementation)

Confirmed-by-DAF, but the strongly expected promotions (all current Tier-B
roster bodies whose mass/orbit class places them in the n373 perturber set):

- **Main-belt (expected present):** `asteroid:5-Astraea`, `asteroid:6-Hebe`,
  `asteroid:8-Flora`, `asteroid:9-Metis`, `asteroid:19-Fortuna`.
- **TNOs (expected present — the massive members of the 30-KBO set):**
  `tno:136199-Eris`, `tno:90377-Sedna`, `tno:136108-Haumea`,
  `tno:136472-Makemake`, `tno:50000-Quaoar`, `tno:90482-Orcus`,
  `tno:225088-Gonggong`; possibly `tno:28978-Ixion`, `tno:20000-Varuna`
  (smaller — membership confirmed by DAF).
- **Expected to stay Tier-B (absent from n373s):** centaurs (Chiron, Pholus,
  Nessus, Chariklo, Asbolus); NEAs / small personal main-belt (Eros, Sappho,
  Lilith, Amor, Hidalgo, Icarus, Toro, Apollo).

Any expected body that DAF inspection shows is *absent* stays Tier-B; any
unexpected roster body shown *present* is promoted. The roster source of truth is
the kernel, not this list.

### 4. Corpus & data changes

- **`asteroid_reference.csv` (Tier-A) — full regenerate:** re-run
  `regenerate-asteroid-corpus` against `sb441-n373s`. This regenerates **all**
  Tier-A rows, including the existing 9 bodies, now sourced from n373s instead of
  n16. Values shift sub-km (both kernels fit DE441), but the committed bytes
  change → new checksum + manifest line. Promoted bodies join at their class
  cadence (`AsteroidClass::max_gap_days`: main-belt 180 d, TNO 1825 d).
- **`asteroid_constrained.csv` (Tier-B) — filter, do not re-fetch:** remove the
  promoted bodies' rows from the existing committed CSV by filtering on body id.
  The remaining Tier-B rows stay **byte-identical** (no Horizons re-fetch, so the
  non-reproducible slice is not perturbed). Row count drops by the promoted set;
  new checksum + manifest line.
- **Roster (`asteroid_roster.rs`):** flip each promoted entry
  `Constrained → PinnedKernel`. Preserve stable roster order (checksums/reports
  depend on it) — flip in place, do not reorder.
- **Claims (`spk_body_claims`):** update the claim `source` string
  `"sb441-n16" → "sb441-n373s"`; otherwise no code change — it is tier-driven, so
  the roster flips move the claims to `ReleaseGrade` automatically.

### 5. Validation gates (reuse, do not invent)

- **Tier-A regen:** promoted bodies join the env-gated `PLEIADES_AST_KERNEL`
  `corpus_regen` path and must reproduce each committed row within the existing
  ~1 km tolerance against `sb441-n373s`. Clean checkout still skips via early
  return (kernel-free).
- **Accuracy ceiling:** the `claims-audit` release-grade-accuracy check holds each
  promoted body to the existing `BodyClass::Asteroid` ceiling (30″ lon/lat,
  5e6 km). Massive main-belt and TNO bodies are expected to clear it; any
  candidate that does not is dropped from the slice rather than shipped
  over-claimed.
- **Corpus validation:** `validate-corpus` / `corpus/asteroid.rs` window + schema
  + finiteness checks cover the moved/regenerated rows with no change.
- **TNO cadence note:** TNOs already validate as Tier-B at the 1825-day cadence;
  the Tier-A path uses the same `AsteroidClass`-derived cadence, so no new epoch
  grid is introduced.

### 6. Claim-surface alignment

Every surface that enumerates or counts the Tier-A asteroid set is updated in the
same change so claims stay consistent:

- `README.md` — the "nine `sb441-n16` Tier-A asteroids …" sentence (count, body
  list, kernel name → `sb441-n373s`), and the JPL corpus description line.
- `crates/pleiades-jpl/src/reference_summary/production_generation.rs` and
  `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` — per-body and
  per-class row/body counts.
- `crates/pleiades-validate/src/corpus/mod.rs:388-389` — the Tier-A window
  label/description (`sb441-n16` → `sb441-n373s`).
- Test fixtures asserting Tier-A/Tier-B membership, counts, or the kernel name
  (`asteroid_roster.rs` tests including the slice-1 roster guard, `corpus_spec.rs`
  pin test, validate/claims tests).
- `docs/spk-kernel-sourcing.md` — replace the n16 section with the n373s kernel
  (source URL, SHA, size, body-set description, window), per-promoted-body
  astrological-usage citations, and the unchanged regen recipe (now pointing at
  n373s). Retire the n16 references.
- `PLAN.md` — record the slice (kernel swap, promoted bodies, new release-grade
  asteroid count, Tier-B count decrement) per the plan-maintenance rule.

**Compatibility profile:** the rendered compatibility profile is house/ayanamsa
scoped; whether it embeds asteroid body claims or a kernel label is confirmed
during implementation. If it does, a patch version bump follows (mirroring
slice-4 hygiene); if not, no bump.

### 7. Error handling / behavior

No runtime behavior change for consumers beyond (a) the claim tier the backend
reports for promoted bodies (Constrained → ReleaseGrade), (b) the source of their
reference rows (Horizons → sb441-n373s), and (c) the reference source of the
existing 9 Tier-A bodies (sb441-n16 → sb441-n373s, sub-km value shift). The
backend boundary stays mean/geometric/geocentric/tropical; coverage stays
1900–2100 CE with fail-closed behavior outside it.

## Acceptance criteria

- `sb441-n16` is fully retired; `sb441-n373s` is the sole pinned asteroid kernel,
  with `AST_KERNEL_LABEL`/`AST_KERNEL_SHA256` updated and the 64-hex pin test
  green.
- `sb441-n373s` is confirmed (by DAF inspection) to cover 1900–2100 and to
  contain every promoted body's NAIF id.
- Each promoted body is `ReleaseGrade` in `spk_body_claims`, sourced from
  `sb441-n373s`, and reproduces within ~1 km under the env-gated `corpus_regen`.
- Each promoted body passes the `BodyClass::Asteroid` accuracy ceiling via
  `claims-audit`.
- Promoted bodies no longer appear in `asteroid_constrained.csv`; they appear
  once, in `asteroid_reference.csv`. Remaining Tier-B rows are byte-identical to
  before (no Horizons re-fetch).
- The existing 9 Tier-A bodies are regenerated from `sb441-n373s` and still pass
  the regen + accuracy gates.
- Every promoted body carries a cited astrological-usage source in
  `docs/spk-kernel-sourcing.md`.
- The Tier-A / Tier-B asteroid counts, body lists, and kernel name are consistent
  across `README.md`, the production/selected-asteroid reports, roster/spec/claims
  tests, and `PLAN.md`.
- `cargo test` is green workspace-wide; `validate-corpus`, `claims-audit`,
  `release-smoke`, and `release-gate` re-run green.
- `PLAN.md` records the slice; the public `Ayanamsa`/`CelestialBody` enums are
  unchanged; non-kernel Tier-B claims are unchanged.

## Risks & mitigations

- **n373s window too narrow.** The abbreviated kernel's exact span is not
  documented online. *Mitigation:* DAF-inspect the window before committing; fall
  back to the full 14 GB `sb441-n373.bsp` (identical body set + pins) if 1900–2100
  is not fully covered.
- **Body-membership uncertainty.** The exact 373-body / 30-KBO list is not
  in-repo. *Mitigation:* membership read from the downloaded kernel's segment
  table at implementation; expected bodies failing verification stay Tier-B,
  none promoted from memory.
- **Kernel unavailability in CI/clean checkout.** The 937 MB kernel is
  uncommitted. *Mitigation:* the regen gate is env-gated and skips kernel-free,
  exactly as the existing Tier-A path does; committed rows + checksums are the
  CI-visible evidence.
- **Existing Tier-A value drift (n16 → n373s).** Re-sourcing the 9 current
  Tier-A bodies changes their committed bytes. *Mitigation:* both kernels fit
  DE441 so the shift is sub-km (well within the ~1 km regen tolerance and the
  accuracy ceiling); the regen gate re-validates them; checksums updated
  atomically.
- **Tier-B filtering correctness.** Dropping promoted bodies from
  `asteroid_constrained.csv` must not perturb remaining rows. *Mitigation:* filter
  existing committed rows by body id (no re-fetch); assert remaining Tier-B rows
  are byte-identical and the dropped set matches the promoted set exactly.
- **Larger committed corpus.** Promoting several TNOs + 5 main-belt bodies grows
  `asteroid_reference.csv`. *Mitigation:* TNO 1825-day / main-belt 180-day
  cadences keep row growth bounded; the corpus has no hard size gate (only the
  packaged artifact does), and the reference CSV is comparison data, not shipped
  in the artifact.

## Open questions

None blocking. The final body set is pinned at implementation by the two-gate
selection policy (§1), with kernel membership as the sole discriminator since only
curated-roster bodies are eligible.
