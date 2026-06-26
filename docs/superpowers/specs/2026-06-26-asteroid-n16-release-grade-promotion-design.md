# Asteroid sb441-n16 Release-Grade Promotion (Phase 6, asteroid slice 1)

- **Status:** design approved, pending implementation plan
- **Date:** 2026-06-26
- **Phase:** PLAN.md Phase 6 — target catalog completion and expansion
- **Crates:** `pleiades-jpl` (roster, corpus, regen), `pleiades-validate`
  (corpus/claims gates), plus claim-surface docs/reports

## Goal

Promote a small, astrologically-relevant set of `sb441-n16` perturber-kernel
asteroids from **constrained** to **release-grade**, reusing the *existing*
Tier-A machinery (pinned kernel + `corpus_regen` gate + Asteroid accuracy
ceiling). This advances the Phase 6 bullet:

> Expand selected-asteroid coverage beyond Ceres/Pallas/Juno/Vesta where source
> evidence and backend metadata support release-grade claims.
> — `plan/stages/06-catalog-completion-and-expansion.md`

Today 7 of the 16 `sb441-n16` perturbers are release-grade Tier-A
(Ceres, Pallas, Juno, Vesta, Hygiea, Psyche, Iris); the rest of the kernel's
members are either committed as Tier-B/Horizons constrained data (Hebe) or not
in the roster at all. Because the release-grade/constrained line is drawn purely
by *reproducibility from the pinned kernel*, any kernel member with documented
astrological usage can earn release-grade through the same gate that already
justifies the existing seven — no new data source, no new accuracy ceiling, no
API change.

This is the asteroid analogue of the ayanamsa promotion slices: promote the
bodies whose airtight reproducible evidence already exists, leave the rest
honestly constrained.

## Scope decisions (from brainstorming)

| Decision | Choice |
| --- | --- |
| Promotion basis | **Pinned-kernel members first.** This slice promotes only `sb441-n16` members; per-body reproducible sources for non-kernel bodies are a separate follow-up slice. |
| Roster scope | **Astrologically-relevant subset only**, not the full perturber set. Massive-but-obscure members (Davida, Interamnia, Sylvia, Thisbe, Europa…) are excluded even though in-kernel. |
| Selection rule | **Reclassify existing Tier-B roster members that are n16 members, AND add a few well-attested asteroid "goddesses" that are n16 members but not yet in the roster.** |
| Roster pinning | **Policy + candidate list now; final body set pinned at implementation** against the actual downloaded kernel. No designation is hard-coded from memory. |

## Non-goals

- **Non-kernel Tier-B bodies stay constrained.** Chiron, Eros, Pholus, the
  remaining personal/"goddess" asteroids, and all TNOs are *not* in
  `sb441-n16`; promoting them needs per-body reproducible sources and is
  deferred to the next asteroid slice. This slice must not broaden their claims.
- **No full-perturber-set promotion.** Obscure massive n16 members with no
  astrological tradition are deliberately left out (see Selection policy §2).
- **No `CelestialBody` enum change.** Every promoted non-classical body uses the
  existing `asteroid:NN-Name` `Custom` catalog id, exactly like Hygiea/Psyche/Iris.
- **No new accuracy ceiling, kernel, or backend boundary change.** The existing
  `BodyClass::Asteroid` ceiling and the mean/geometric/geocentric boundary are
  unchanged.
- **No calculated points / fictional bodies** (unchanged from the 2026-06-17
  asteroid-coverage design's non-goals).

## Background: why this is a focused promotion, not new infrastructure

- **Tier is the only lever.** `AsteroidTier::{PinnedKernel, Constrained}`
  (`crates/pleiades-jpl/src/spk/asteroid_roster.rs:13`) drives everything:
  `spk_body_claims` (`asteroid_roster.rs:134`) emits
  `ReleaseGrade/High/CorpusValidated{"sb441-n16"}` for `PinnedKernel` bodies and
  `Constrained/Moderate/CorpusValidated{"horizons"}` for `Constrained` bodies.
  Flipping a roster entry's tier flips its claim automatically.
- **The kernel already contains them.** `sb441-n16.bsp` holds the 16 most-massive
  perturbers (`docs/spk-kernel-sourcing.md:35`); the regen path
  (`PLEIADES_AST_KERNEL` → `corpus_regen`) already reproduces Tier-A asteroid
  rows within ~1 km. Additional members reuse this path with no plumbing change.
- **The gates already fail closed.** `validate-corpus` /
  `crates/pleiades-validate/src/corpus/asteroid.rs` enforce the 1900–2100 window,
  5-column schema, and finiteness; the `claims-audit`
  release-grade-accuracy check (`crates/pleiades-validate/src/claims/audit.rs`)
  holds release-grade bodies to the `BodyClass::Asteroid` ceiling
  (`crates/pleiades-data/src/thresholds.rs:75` — 30″ lon/lat, 5e6 km).
- **Hebe is already committed**, as Tier-B Horizons data
  (`asteroid_roster.rs:88`, `data/corpus/asteroid_constrained.csv`), so its
  reclassification is a re-source + re-tag, not a new body.

## Design

### 1. Selection policy

A body is promoted to Tier-A release-grade **only if both** conditions hold,
both verified at implementation time:

1. **Kernel membership** — confirmed present in `sb441-n16.bsp` by reading the
   downloaded kernel's segment NAIF ids (env-gated maintainer step). Never
   asserted from memory.
2. **Documented astrological usage** — a citable source recorded in
   `docs/spk-kernel-sourcing.md`. Existing Tier-B roster inclusion already
   establishes relevance (those bodies were curated for it); each *new* body
   added by this slice needs an explicit cited astrological tradition.

A kernel member that fails either gate is **not** promoted: obscure massive
members (Davida, Interamnia, Sylvia, Thisbe, Europa, …) fail gate 2 and stay
out; astrologically-used non-members (Chiron, Eros, …) fail gate 1 and stay
constrained. This is the "astrologically-relevant only" decision made concrete.

### 2. Candidate roster (subject to both gates)

Final membership is pinned during implementation against the actual kernel; the
candidate set is:

- **Reclassify (already committed, Tier-B → Tier-A):**
  - `asteroid:6-Hebe` — main-belt; currently Tier-B Horizons. If confirmed in
    `sb441-n16`, re-sourced from the kernel and promoted.
  - (Any other current Tier-B main-belt body that turns out to be an n16 member
    is reclassified the same way; verified at implementation. None besides Hebe
    is expected.)
- **Add (new Tier-A entries, n16 members with cited astrological usage):**
  - `asteroid:65-Cybele` — Great Mother goddess; established asteroid-astrology
    usage.
  - `asteroid:15-Eunomia` and/or `asteroid:29-Amphitrite` — included only if
    both (a) confirmed in-kernel and (b) backed by a cited astrological tradition
    in the provenance doc; otherwise dropped.

Any candidate failing kernel-membership or documented-usage verification is
silently dropped from the slice (not committed as a known gap — it simply
remains outside the curated core, reachable on demand like any other body).

### 3. Corpus & data changes

- **Reclassified bodies (Hebe):**
  - Remove their rows from `data/corpus/asteroid_constrained.csv` (Tier-B).
  - Regenerate their rows into `data/corpus/asteroid_reference.csv` (Tier-A)
    from `sb441-n16` at the `MainBelt` 180-day cadence
    (`AsteroidClass::max_gap_days`, `asteroid_roster.rs:32`).
  - Flip the roster entry `Constrained → PinnedKernel`.
  - The Tier-A and Tier-B values for the body differ slightly (kernel vs
    Horizons solution); the Tier-B rows are deleted, not kept in parallel.
- **New bodies (Cybele, …):**
  - Add `PinnedKernel/MainBelt` roster entries in stable order
    (roster order feeds checksums/reports — append within the main-belt group).
  - Generate their reference rows from the kernel at the 180-day cadence.
- **Claims:** no code change in `spk_body_claims` — it is already tier-driven, so
  the roster edits alone move the claims to `ReleaseGrade`.
- **Manifests/checksums:** both asteroid slice checksums update; the Tier-B row
  count drops by the reclassified set, the Tier-A row count grows by the
  reclassified + added set.

### 4. Validation gates (reuse, do not invent)

- **Tier-A regen:** promoted bodies join the env-gated `PLEIADES_AST_KERNEL`
  `corpus_regen` path and must reproduce each committed row within the existing
  ~1 km tolerance. Clean checkout still skips via early return (kernel-free).
- **Accuracy ceiling:** the `claims-audit` release-grade-accuracy check holds
  each promoted body to the existing `BodyClass::Asteroid` ceiling (30″ lon/lat,
  5e6 km). Massive main-belt bodies clear it comfortably; if any candidate does
  not, it is dropped from the slice rather than shipped over-claimed.
- **Corpus validation:** `validate-corpus` / `corpus/asteroid.rs` window + schema
  + finiteness checks cover the moved and added rows with no change.

### 5. Claim-surface alignment

Every surface that enumerates or counts the Tier-A asteroid set is updated in the
same change so claims stay consistent:

- `README.md:28` — the "seven `sb441-n16` Tier-A asteroids …" sentence (count +
  body list).
- `crates/pleiades-jpl/src/reference_summary/production_generation.rs` and
  `crates/pleiades-jpl/src/reference_summary/selected_asteroid.rs` — per-body and
  per-class row/body counts.
- Any test fixture asserting the exact Tier-A / Tier-B membership or counts
  (e.g. `asteroid_roster.rs` tests, validate/claims tests).
- `docs/spk-kernel-sourcing.md` — expand the n16 member list to name the promoted
  bodies, add per-new-body astrological-usage citations, and confirm the regen
  recipe covers them.
- `PLAN.md` — record the slice (promoted bodies, new release-grade asteroid
  count, Tier-B count decrement) per the plan-maintenance rule.

**Compatibility profile:** the rendered compatibility profile is house/ayanamsa
scoped; whether it embeds asteroid body claims is confirmed during
implementation. If it does, a patch version bump follows (mirroring slice-4
hygiene); if not, no bump.

### 6. Error handling / behavior

No runtime behavior change for consumers beyond the claim tier the backend
reports for the promoted bodies (Constrained → ReleaseGrade) and the source of
their reference rows (Horizons → sb441-n16 for reclassified bodies). The backend
boundary stays mean/geometric/geocentric/tropical; coverage stays 1900–2100 CE
with fail-closed behavior outside it.

## Acceptance criteria

- Each promoted body is `ReleaseGrade` in `spk_body_claims`, sourced from
  `sb441-n16`, and reproduces within ~1 km under the env-gated `corpus_regen`.
- Each promoted body passes the `BodyClass::Asteroid` accuracy ceiling via
  `claims-audit`.
- Reclassified bodies (Hebe, …) no longer appear in `asteroid_constrained.csv`;
  they appear once, in `asteroid_reference.csv`.
- Every newly added body carries a cited astrological-usage source in
  `docs/spk-kernel-sourcing.md`; no body is promoted without both gates passing.
- The Tier-A asteroid count and body list are consistent across `README.md`,
  the production/selected-asteroid reports, roster tests, and `PLAN.md`.
- `cargo test` is green workspace-wide; `validate-corpus`, `claims-audit`,
  `release-smoke`, and `release-gate` re-run green.
- `PLAN.md` records the slice; the public `Ayanamsa`/`CelestialBody` enums are
  unchanged; non-kernel Tier-B claims are unchanged.

## Risks & mitigations

- **Kernel-membership uncertainty** — the exact n16 list is not in-repo. *Mitigation:*
  membership is read from the downloaded kernel's segment table at implementation;
  candidates failing verification are dropped, none are committed from memory.
- **Kernel unavailability in CI/clean checkout** — the 645 MB kernel is
  uncommitted. *Mitigation:* the regen gate is env-gated and skips kernel-free,
  exactly as the existing Tier-A path does; committed rows + checksums are the
  CI-visible evidence.
- **Reclassification drift (Hebe)** — Tier-A kernel values differ from the
  deleted Tier-B Horizons values. *Mitigation:* the Tier-B rows are removed, not
  kept; the body appears in exactly one slice; checksums updated atomically.
- **Astrological-usage subjectivity for new bodies** — "well-attested" is a
  judgment. *Mitigation:* gate 2 requires a *cited* source in the provenance doc;
  bodies without one are dropped rather than promoted on assertion.

## Open questions

None blocking. The final body set is pinned at implementation by the two-gate
selection policy (§1); candidates failing either gate are dropped.
