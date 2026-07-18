# Devkit Phase 4 — Navigability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the repo navigable — a `CLAUDE.md` front door, a README capability table replacing the ~2,500-word "Current state" prose, an explicit codebase-map link, a verified clone-to-green onboarding path — landed as one PR.

**Architecture:** Six small edits to docs + one release-critical code change (retarget the claims-audit README anchor). The README prose blob becomes an at-a-glance capability table (surface × crate × gate × accuracy class) whose detail links to per-crate rustdoc and the `pleiades-core` compatibility registry — the canonical homes for measured residuals and carve-outs. The whole phase is acceptance-tested by running `mise install` → `mise run ci` in a clean checkout.

**Tech Stack:** Rust workspace, `mise` task runner, `cargo-nextest`, markdown docs. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-07-18-devkit-phase4-navigability-design.md`

## Global Constraints

- **Branch:** all work lands on `devkit-phase4-navigability` (already created; the spec is committed there). One PR for the phase.
- **Single canonical agent-instruction source:** `AGENTS.md`. `CLAUDE.md` references it, never duplicates it.
- **No new residual numbers in the README.** Measured accuracy stays in per-crate rustdoc + `crates/pleiades-core/src/compatibility/mod.rs`; the README links there.
- **No new `ARCHITECTURE.md`.** `spec/architecture.md` is the codebase map; single-sourced.
- **claims-audit invariant preserved:** after the README restructure, `check_surfaces` must still fire when a release-grade descriptor count changes without a README update. The existing test `readme_counts_match_descriptors` (in `crates/pleiades-validate/src/claims/compat.rs`) must stay green, and it reads the real `include_str!`'d README (there is no injected-README test fixture to update).
- **Blocking-tier budget:** `mise run ci` must stay ≤ 10 minutes; this phase adds no CI work.
- **Commit style:** Conventional Commits (`docs:`, `fix:`), since release-plz parses them.
- **Do not push or open a PR** until all tasks are done and the user asks.

---

### Task 1: `CLAUDE.md` front door

**Files:**
- Create: `CLAUDE.md`

**Interfaces:**
- Consumes: nothing.
- Produces: nothing consumed by later tasks (independent).

- [ ] **Step 1: Create `CLAUDE.md`**

Create `CLAUDE.md` at the repo root with exactly:

```markdown
<!-- AGENTS.md is the single canonical source of agent instructions for this
     repo. Edit it there, not here. This file only makes Claude Code auto-load
     it. -->

@AGENTS.md
```

- [ ] **Step 2: Verify the reference resolves**

Run: `test -f AGENTS.md && grep -q '@AGENTS.md' CLAUDE.md && echo OK`
Expected: `OK`

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add CLAUDE.md front door referencing AGENTS.md"
```

---

### Task 2: README capability table + claims-audit anchor retarget

This is the substantive task. The README edit and the claims-audit code change **must land in the same commit** — removing the prose blob moves the audit's anchor, so the two are coupled.

**Files:**
- Modify: `README.md:11-42` (replace the `## Current state` section body: the "As of the current workspace state…" bullet list **and** the "Important current limits:" list)
- Modify: `crates/pleiades-validate/src/claims/compat.rs:192` (the `aya_token` `format!`)
- Test: `crates/pleiades-validate/src/claims/compat.rs` — existing `readme_counts_match_descriptors` (no new test file)

**Interfaces:**
- Consumes: nothing.
- Produces: the capability table other docs may link to; no code interface.

- [ ] **Step 1: Baseline — confirm claims-audit is green before changes**

Run: `cargo nextest run -p pleiades-validate readme_counts_match_descriptors`
Expected: PASS (1 test). This proves the current anchor works before we move it.

- [ ] **Step 2: Confirm the current README anchor substrings**

Run: `grep -n ' 24 house systems pass\| 48 release-claimed' README.md`
Expected: two matches inside the prose blob (lines ~40). These are the substrings `check_surfaces` greps for (`house_count`=24, `aya_count`=48 today).

- [ ] **Step 3: Enumerate the authoritative gate names for the table**

Do **not** hardcode gate names from memory. List the real validate/gate surfaces:

Run: `cargo run -q -p pleiades-cli -- --help`
and: `grep -oE '^\[tasks\.gate-[a-z-]+\]' mise.toml`

Use the `validate-*` CLI subcommand names (the form the existing README prose already uses, e.g. `validate-angles`, `validate-rise-trans`, `validate-eclipses`, `validate-crossings`, `validate-fictitious`, `validate-nod-aps`, `validate-pheno`, `validate-occultations`, `validate-lilith`, `validate-equatorial`, `validate-frame-consistency`) for the Gate column. For any row whose gate name you cannot confirm from that output, use the mise `gate-*` task name instead and note it. **No invented names.**

- [ ] **Step 4: Replace the `## Current state` body with the capability table**

In `README.md`, replace everything from the paragraph beginning "As of the current workspace state, `pleiades` includes:" through the end of the "Important current limits:" list (the blob ending just before `## Published crates`) with the block below. Populate each row's **Gate** from Step 3 and keep each **Accuracy class** transcribed faithfully from the prose you are removing (the prose is the current source of truth — do not upgrade a class):

````markdown
`pleiades` is a release-hardening foundation, not a finished end-user
ephemeris. Each surface below is guarded by a fail-closed numeric gate; measured
residuals, carve-outs, and caveats live in the linked crate docs and in the
`pleiades-core` compatibility registry, not restated here.

Release-grade numeric compatibility today: 24 house systems pass the SE numeric
gate, and 48 ayanamsas pass theirs — of 25 and 59 catalogued respectively.

| Surface | Crate | Gate | Accuracy class |
| --- | --- | --- | --- |
| Body positions / packaged artifact | `pleiades-data` | `gate-corpus` | sub-arcsecond (majors) |
| House systems | `pleiades-houses` | `gate-houses` | sub-arcsecond |
| Ayanamsas | `pleiades-ayanamsa` | `gate-ayanamsa` | sub-arcsecond |
| Sidereal time & chart angles | `pleiades-houses` | `validate-angles` | sub-arcsecond |
| Apparent place (of-date ecliptic) | `pleiades-core` | `gate-apparent` | arcsecond-class |
| Apparent equatorial (RA/Dec) | `pleiades-core` | `validate-equatorial` | sub-arcsecond |
| Civil time conversion | `pleiades-time` | (unit/property) | leap-second-exact |
| Topocentric correction | `pleiades-core` | `gate-topocentric` | opt-in correction |
| Backend frame consistency (J2000) | `pleiades-core` | `validate-frame-consistency` | invariant gate |
| Eclipses (global) | `pleiades-eclipse` | `validate-eclipses` | arcsecond-class; timing seconds-of-time |
| Eclipses (local circumstances) | `pleiades-eclipse` | `validate-eclipses-local` | arcsecond-class; timing seconds-of-time |
| Longitude crossings | `pleiades-events` | `validate-crossings` | arcsecond-class |
| Rise/set/transit & horizontal | `pleiades-events` | `validate-rise-trans` | sub-arcsecond (horizontal); timing seconds-of-time |
| Fictitious bodies | `pleiades-fict` | `validate-fictitious` | definitional (sub-arcsecond) |
| Nodes & apsides | `pleiades-events` | `validate-nod-aps` | sub-arcsecond (mean) / arcminute-class (osculating) |
| Phase & magnitude | `pleiades-events` | `validate-pheno` | arcsecond-class |
| Lunar occultations | `pleiades-events` | `validate-occultations` | timing seconds-of-time; position arcminute-class |
| True (osculating) Lilith | `pleiades-apsides` | `validate-lilith` | arcminute-class |

Crate names link to their docs.rs pages; gate names to the module rustdoc that
records the measured residuals for that surface.

### Known limits

- Body/backend grades are **per-backend**: Pluto/Moon/Eros are release-grade via
  the packaged artifact; VSOP87 Pluto and the compact ELP Moon stay constrained.
  See `crates/pleiades-core/src/compatibility/mod.rs`.
- Apparent place omits gravitational light-deflection; rise/set/transit instants
  are **UT1-scale** (no ΔT model) — see `crates/pleiades-apparent` rustdoc and
  [docs/time-observer-policy.md](docs/time-observer-policy.md).
- Several surfaces carry documented, non-gated bounds (occultation planet-total
  obscuration and `central` flag; fictitious Nibiru; osculating small-body
  nodes/apsides). Each is recorded in its crate's rustdoc and in
  `crates/pleiades-core/src/compatibility/mod.rs`.
- Ingestion and kernel/corpus parsing are treated as untrusted input — see
  [docs/threat-model.md](docs/threat-model.md).
- Lunar theory selection and its limits: [docs/lunar-theory-policy.md](docs/lunar-theory-policy.md).
````

Adjust the two Gate cells for houses/ayanamsa to the CLI `validate-*` name if Step 3 shows one; otherwise the `gate-*` task names above are correct.

- [ ] **Step 5: Run claims-audit — expect the ayanamsa anchor to now fail (red)**

Run: `cargo nextest run -p pleiades-validate readme_counts_match_descriptors`
Expected: **FAIL** — `surface drift: [SurfaceDisagrees { surface: "README:ayanamsa" }]`. The house token `" 24 house systems pass"` is still present (the caption carries it), but the old ayanamsa token `" 48 release-claimed"` is gone. This is the anchor move made visible.

- [ ] **Step 6: Retarget the ayanamsa anchor token**

In `crates/pleiades-validate/src/claims/compat.rs`, change line ~192 from:

```rust
    let aya_token = format!(" {aya_count} release-claimed");
```

to:

```rust
    let aya_token = format!(" {aya_count} ayanamsas pass");
```

Leave `house_token` (`format!(" {house_count} house systems pass")`) unchanged — the new caption still carries it verbatim, so the house anchor needs no code change. Update the doc comment above `check_surfaces` if it quotes the old wording.

- [ ] **Step 7: Run claims-audit — expect green**

Run: `cargo nextest run -p pleiades-validate readme_counts_match_descriptors`
Expected: PASS. The caption's " 48 ayanamsas pass" and " 24 house systems pass" both match.

- [ ] **Step 8: Prove the drift-guard still fires (temporary mutation)**

Temporarily edit the README caption to read "47 ayanamsas pass", then:

Run: `cargo nextest run -p pleiades-validate readme_counts_match_descriptors`
Expected: **FAIL** with `SurfaceDisagrees { surface: "README:ayanamsa" }`.
Then **revert** the caption back to "48 ayanamsas pass" and re-run:
Expected: PASS. This confirms the invariant is preserved, then leaves the tree clean.

- [ ] **Step 9: Confirm no accuracy claim was orphaned**

For each caveat removed from the README, confirm a home exists:

Run:
```bash
grep -rl "UT1-scale" crates/pleiades-apparent/src crates/pleiades-core/src
grep -rl "Nibiru" crates/pleiades-fict/src crates/pleiades-types/src
grep -rln "central\|obscuration\|not.*gated" crates/pleiades-core/src/compatibility/mod.rs
```
Expected: each returns at least one path. If any caveat has **no** home outside the removed prose, restore that one caveat as a "Known limits" bullet rather than dropping it.

- [ ] **Step 10: Run fmt, clippy, and the full validate suite**

Run:
```bash
cargo fmt --all --check
cargo clippy -p pleiades-validate --all-targets -- -D warnings
cargo nextest run -p pleiades-validate
```
Expected: all green.

- [ ] **Step 11: Commit**

```bash
git add README.md crates/pleiades-validate/src/claims/compat.rs
git commit -m "docs: replace README Current-state prose with capability table; retarget claims-audit anchor"
```

---

### Task 3: Codebase-map callout in the Documentation map

**Files:**
- Modify: `README.md` — the `## Documentation map` section (currently ~line 192)

**Interfaces:**
- Consumes: nothing. Produces: nothing.

- [ ] **Step 1: Confirm `spec/architecture.md` is the map**

Run: `head -20 spec/architecture.md`
Expected: content describing workspace layering / dependency boundaries (it is the codebase map).

- [ ] **Step 2: Promote it to a "start here" callout**

In `README.md`, at the top of the `## Documentation map` section (immediately after the heading, before the existing bullet list), add:

```markdown
**New to the codebase?** Start with
[spec/architecture.md](spec/architecture.md) — the workspace layering and
crate-dependency map.
```

Leave the existing bullet list (which already links `spec/architecture.md`) unchanged.

- [ ] **Step 3: Verify links resolve**

Run: `test -f spec/architecture.md && grep -q 'Start with' README.md && echo OK`
Expected: `OK`

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: surface spec/architecture.md as the codebase-map front door"
```

---

### Task 4: AGENTS.md gap-check

The audit found AGENTS.md already carries the threat-model pointer, the four-tier CI policy, and cargo-fuzz/cargo-mutants in its mise list. This task **verifies** and patches only genuine drift — expect little or no change.

**Files:**
- Modify (only if drift found): `AGENTS.md`

**Interfaces:**
- Consumes: nothing. Produces: nothing.

- [ ] **Step 1: Verify the three consistency points**

Run:
```bash
grep -n "docs/threat-model.md" AGENTS.md
grep -n "blocking\|nightly\|fuzz\|mutants" AGENTS.md | head
```
Expected: threat-model pointer present (~line 169); tiering policy present (~line 300).

- [ ] **Step 2: Diff the mise tool list against `mise.toml`**

Run:
```bash
grep -oE '\[tools\][^[]*' -z mise.toml 2>/dev/null | tr ',' '\n' | grep -oE 'cargo-[a-z]+|nextest' | sort -u
```
Compare against the "Use `mise.toml` for things such as:" bullet list in `AGENTS.md` (lines ~39–51). If a tool is pinned in `mise.toml` but absent from that list (or vice-versa), that is drift.

- [ ] **Step 3: Patch only genuine gaps**

If Step 1 or 2 found drift, edit `AGENTS.md` to close it (add a missing tool bullet; fix a stale pointer). If nothing drifted, make **no** edit and skip to the next task — do not manufacture a change.

- [ ] **Step 4: Commit (only if edited)**

```bash
git add AGENTS.md
git commit -m "docs: align AGENTS.md tooling list and pointers with current mise.toml"
```

If no edit was made, record in the task notes that AGENTS.md was already consistent (no commit).

---

### Task 5: Onboarding steps + clone-to-green verification

**Files:**
- Modify: `README.md` — the `## Local development` section (~lines 162–190)

**Interfaces:**
- Consumes: everything above (this is the whole-phase acceptance test).
- Produces: nothing.

- [ ] **Step 1: Add the hooks-path activation and Renovate note**

In `README.md`'s `## Local development` section, immediately after the `mise install` code fence, insert:

```markdown
Activate the committed pre-commit hooks (opt-in — git cannot force hooks):

```bash
git config core.hooksPath .githooks
```

Dependency and toolchain updates arrive as grouped [Renovate](https://docs.renovatebot.com)
pull requests, gated by the blocking CI tier.
```

Confirm `.githooks/pre-commit` exists (`ls .githooks/`) so the instruction is real.

- [ ] **Step 2: Run the clone-to-green acceptance sequence**

In a clean checkout of the branch (or `git stash`-clean working tree), run:

```bash
mise install
mise run ci
```
Expected: `mise install` provisions the pinned tools; `mise run ci` passes green within the ≤ 10-minute blocking budget, **including** the `claims-audit` step that Task 2 retargeted. If `mise run ci` fails, the phase is not done — fix the cause (a documented step that does not run clean is a failed acceptance).

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document hooks-path activation and Renovate cadence in onboarding"
```

---

## Self-Review

**Spec coverage** (each spec part → task):
- Part 1 CLAUDE.md front door → Task 1 ✓
- Part 2 README capability table → Task 2 Steps 3–4 ✓
- Part 3 claims-audit retarget → Task 2 Steps 5–8 ✓ (refined: only the ayanamsa token changes; the house token's wording is preserved by the caption, reducing release-critical churn — consistent with the spec's "preserve the invariant, clean wording" intent)
- Part 4 codebase-map link → Task 3 ✓
- Part 5 AGENTS.md gap-check → Task 4 ✓
- Part 6 onboarding verified by running → Task 5 ✓
- Acceptance criterion "no accuracy claim orphaned" → Task 2 Step 9 ✓
- Acceptance criterion "drift-guard preserved" → Task 2 Step 8 ✓

**Placeholder scan:** The Gate/Accuracy-class cells are populated concretely and cross-checked against authoritative sources (Task 2 Step 3 for gate names; the replaced prose for accuracy classes). Task 4 is intentionally a verify-then-patch-if-needed task, not a placeholder — its "only if drift" branches are explicit. No TBD/TODO remain.

**Type consistency:** The only code identifiers are `house_token`, `aya_token`, `house_count`, `aya_count`, `check_surfaces`, and the test `readme_counts_match_descriptors` — all match `crates/pleiades-validate/src/claims/compat.rs` as read. The retargeted `aya_token` string `" {aya_count} ayanamsas pass"` matches the README caption "48 ayanamsas pass" exactly.

**Note carried to execution:** if Task 2 Step 3 reveals the houses/ayanamsa CLI gate names differ from the `gate-houses`/`gate-ayanamsa` mise-task names used in the starter table, use whichever name a reader can actually run, and keep the choice consistent across the table.
