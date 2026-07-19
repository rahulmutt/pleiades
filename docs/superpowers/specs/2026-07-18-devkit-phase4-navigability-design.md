# Devkit Phase 4 — Navigability Design

**Date:** 2026-07-18
**Status:** Approved design, pending implementation plan
**Parent:** `docs/superpowers/specs/2026-07-17-devkit-adoption-design.md` (Phase 4)

## Goal

Land the final phase of devkit adoption: make the repository navigable for the
next contributor — human or an agent with a bounded context window. A
discoverable front door (`CLAUDE.md` → `AGENTS.md`), the sprawling README
"Current state" prose replaced by an at-a-glance capability table, an explicit
codebase-map link, and an onboarding path verified by actually running it.
Phases 1–3 (security/deps, two-tier CI, testing upgrades) are shipped; this is
the last phase.

## Non-goals

- No new `ARCHITECTURE.md` — `spec/architecture.md` already serves as the map;
  a parallel doc would violate single-sourcing.
- No restatement of measured accuracy numbers in the README — residuals and
  carve-outs stay in per-crate rustdoc and the `pleiades-core` compatibility
  registry, which the table links to.
- No change to `release-gate` semantics or the release process.
- No change to what `claims-audit` *guarantees* — only where in the README it
  reads the counts from.

## Packaging

**One slice, one spec, one plan, one PR.** The five parts are individually
small and tightly coupled to the same navigability surface; the clone-to-green
onboarding run is the natural whole-phase acceptance test. This follows
AGENTS.md's small-reviewable-unit rule at the phase granularity — the phase
*is* the unit here, unlike Phase 3 which split into independently-valuable test
forms.

## Key findings from the gap audit (scope-shaping)

1. **The README is not the sole home of any accuracy claim.** A repo-wide
   search confirmed every load-bearing caveat has a canonical home outside the
   README, so the full restructure orphans nothing:
   - UT1-scale rise/set caveat → `crates/pleiades-apparent/src/sidereal.rs`
     rustdoc + `crates/pleiades-core/src/compatibility/mod.rs` +
     `crates/pleiades-validate/src/rise_trans_validation.rs`.
   - Nibiru / fictitious carve-outs → `crates/pleiades-fict/src/elements.rs`,
     `crates/pleiades-types/src/bodies.rs`, `fictitious_thresholds.rs`.
   - Occultation non-gated bounds (planet-total obscuration, `central` flag,
     miss-classification) → `crates/pleiades-core/src/compatibility/mod.rs` +
     `crates/pleiades-validate/src/occult_validation.rs`.
   - Per-surface residuals → per-module rustdoc in
     `crates/pleiades-events/src/{pheno,rise_trans,occult,crossings,nod_aps}.rs`.
   - **Correction to the parent design's assumption:** `CHANGELOG.md` is *not*
     the accuracy-detail home (it holds Conventional-Commit change entries, not
     residual tables). The homes are per-crate rustdoc + the `compatibility`
     registry. The README table links there, not to CHANGELOG.

2. **AGENTS.md is already substantially touched-up.** It already carries the
   `docs/threat-model.md` pointer (line ~169), the four-tier CI policy
   (blocking / nightly / fuzz / mutants, line ~300), and cargo-fuzz +
   cargo-mutants in its mise tool list (lines ~46–47) — these landed
   incrementally across Phases 1–3. So the AGENTS.md part of Phase 4 is a
   **gap-verification**, not a rewrite.

3. **The claims-audit anchor is a whole-file substring check, not
   section-scoped.** `check_surfaces` in
   `crates/pleiades-validate/src/claims/compat.rs` does
   `README.contains(format!(" {house_count} house systems pass"))` and
   `README.contains(format!(" {aya_count} release-claimed"))` over the entire
   `include_str!`'d README. Removing the prose blob breaks both unless the
   check is retargeted (the chosen approach — see Part 3).

## The five parts

### Part 1 — `CLAUDE.md` front door

New minimal `CLAUDE.md` at repo root:

- Contains `@AGENTS.md` so Claude Code auto-loads the existing agent
  instructions.
- One short comment line noting AGENTS.md is the single canonical source and
  should be edited there, not here.
- No content duplicated from AGENTS.md.

### Part 2 — README "Current state" restructure

Replace the ~2,500-word prose blob (README.md lines ~11–42, the `## Current
state` section through the "Important current limits" list) with a **capability
table**, one row per surface.

**Columns:** `Surface | Crate | Gate | Accuracy class`

**Rows (one per surface):** bodies / packaged artifact, houses, ayanamsas,
sidereal time & angles, apparent place (of-date ecliptic), equatorial (RA/Dec),
civil time conversion, topocentric, eclipses (global + local), longitude
crossings, rise/set/transit & horizontal, fictitious bodies, nodes & apsides,
phase / magnitude (pheno), lunar occultations, true (osculating) Lilith.

**Accuracy class** is a controlled vocabulary, **not** raw residual numbers:

- `sub-arcsecond` — sub-1″ parity vs the surface's reference authority.
- `arcsecond-class` — a few arcsec (e.g. eclipse on-sky positions,
  well-conditioned rise/set).
- `arcminute-class` — arcminute-scale (e.g. osculating planet apsides).
- `seconds-of-time` — timing surfaces whose ceiling is a time span
  (eclipse/occultation contact instants); widening noted as "…, wider near
  grazing geometry" only where it changes how a consumer reads the class.
- `definitional` — parity-by-definition against the SE reference set
  (fictitious bodies).
- `catalogued-only` — metadata present, no numeric gate (the catalogued-but-
  not-release-claimed house systems / ayanamsas).

The **Gate** cell names the fail-closed gate (e.g. `validate-rise-trans`) and
links to the crate/module rustdoc where the measured residuals and carve-outs
live. The **Crate** cell links to the crate's docs.rs page.

The "Important current limits" prose collapses to a short **"Known limits"**
bullet list (≤ ~8 bullets) that *points* rather than restates: per-backend body
grades, the apparent-place omissions (no gravitational light-deflection), the
time-scale caveats, and links to `docs/threat-model.md`,
`docs/time-observer-policy.md`, and `docs/lunar-theory-policy.md`. No residual
numbers, no per-body carve-out prose — those stay in their canonical homes.

### Part 3 — claims-audit retarget (ships with Part 2)

Update `check_surfaces` in `crates/pleiades-validate/src/claims/compat.rs`:

- Retarget the two `format!` tokens from the prose phrasings
  (`" {house_count} house systems pass"`, `" {aya_count} release-claimed"`) to
  short, stable count-adjacent phrases that the new table (or its one-line
  caption) carries verbatim. Exact strings are finalized in the plan once the
  table wording is fixed; they must be substrings that appear **only** for the
  release-grade count, and must not embed the *catalogued* total (25 / 59) so a
  catalogued-only change doesn't false-positive.
- Update the fixtures in the module's `#[cfg(test)] mod tests` to match.
- **Invariant preserved:** the forward drift-guard still fires when a
  descriptor's release-grade count is incremented without updating the README.
  Checks A and B (tier-evidence, profile-count) remain the same-commit
  backstop, unchanged.

This is the only release-critical code touched in the phase; it is why Part 2
and Part 3 ship together in the same commit.

### Part 4 — Codebase-map link

No new document. README's "Documentation map" already links
`spec/architecture.md` (line ~195). Verify that entry reads as *the* codebase
map; if it is buried, promote it to a short "Start here for the codebase
layout" callout near the top of the doc map. Single-sourced — no ARCHITECTURE.md.

### Part 5 — AGENTS.md gap-check

Verify, and patch only genuine gaps in:

- the `docs/threat-model.md` pointer,
- the blocking / nightly / fuzz / mutants tiering description,
- the "install via mise" tool list vs. the current `mise.toml` contents.

Expectation from the audit: little to no change needed. Any drift found is
corrected; no wholesale rewrite.

### Part 6 — Onboarding verified by running

- Add the missing `git config core.hooksPath .githooks` activation step and a
  one-line Renovate note to README's "Local development" section (lines
  ~162–190).
- **Acceptance test:** execute `mise install` → `mise run ci` in a clean
  checkout and confirm it passes green within the blocking-tier budget. This
  run is the whole-phase gate; a documented step that does not actually run
  clean is a failed acceptance.

## Risks & constraints

- **claims-audit regression is the top risk.** Mitigation: Part 3 updates the
  check and its tests in the same commit as the README change, and the
  clone-to-green run exercises `claims-audit` (it is in the blocking tier), so a
  broken anchor fails the acceptance test, not just review.
- **Silent claim loss.** Mitigation: spec self-review + the plan's first task
  cross-check every residual/caveat removed from the README against its
  canonical home (finding 1 above); anything without a home is preserved in the
  README, not dropped.
- **docs.rs link rot.** The table links to docs.rs crate pages; these resolve
  only after publish. Acceptable — the crates are published `0.2.x`; use the
  versioned-latest docs.rs URL form.

## Acceptance criteria

1. `CLAUDE.md` exists and loads `AGENTS.md` (`@AGENTS.md`).
2. README `## Current state` is a capability table (surface × crate × gate ×
   accuracy class) plus a short "Known limits" pointer list; the prose blob and
   all raw residual numbers are gone from the README.
3. `claims-audit` (retargeted `check_surfaces` + updated tests) passes; the
   forward drift-guard invariant is preserved and asserted by a test.
4. README "Documentation map" surfaces `spec/architecture.md` as the codebase
   map.
5. AGENTS.md's threat-model pointer, tiering policy, and mise tool list are
   verified consistent with `mise.toml`.
6. README "Local development" documents the `core.hooksPath` activation and the
   Renovate cadence.
7. `mise install` → `mise run ci` runs green in a clean checkout (executed, not
   assumed).
8. No accuracy claim orphaned: every residual/caveat removed from the README is
   confirmed present in per-crate rustdoc or the `pleiades-core` compatibility
   registry.
