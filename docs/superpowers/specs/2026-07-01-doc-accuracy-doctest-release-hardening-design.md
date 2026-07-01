# Doc-accuracy + doctest release hardening

**Date:** 2026-07-01
**Status:** Approved (design)
**Author:** brainstormed with Rahul Muttineni

## Problem

The next `pleiades` release ships a public API across many crates, but the
documentation surface is uneven and unverified against reality:

- Doctest coverage is heavily skewed. `pleiades-core` has rich module-level
  doctests, while `pleiades-apparent`, `pleiades-jpl`, `pleiades-elp`,
  `pleiades-time`, and `pleiades-vsop87` have essentially **zero** runnable
  examples.
- There is no `missing_docs` lint, so undocumented public items can ship and
  doc coverage can silently regress after release.
- The repo makes carefully-scoped **per-backend** accuracy claims (e.g. VSOP87
  Pluto stays approximate; the compact ELP Moon is constrained; release-grade
  positions come only via the packaged-data artifact). Prose doc comments are
  the most likely place for those claims to drift into **overclaims** that the
  validation gates do not support.

CI already runs `cargo test --workspace` (which includes doctests) and
`mise run docs` (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
--all-features`), so any docs and doctests added here are enforced going
forward.

## Goal

For the public crates, guarantee that:

1. Every doc comment **accurately** describes the public API — with special
   attention to catching accuracy/capability **overclaims**.
2. Every reachable public item is documented under `#![deny(missing_docs)]`.
3. Doctests demonstrate the common astrology usecases end-to-end.

All changes must be verified by `cargo test --doc` and
`cargo doc` with `-D warnings`.

## Scope

**In scope — 13 crates** (the public API surface):

- The 11 published library crates: `pleiades-types`, `pleiades-backend`,
  `pleiades-core`, `pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`,
  `pleiades-elp`, `pleiades-jpl`, `pleiades-compression`, `pleiades-time`,
  `pleiades-apparent`.
- Plus `pleiades-apsides` and `pleiades-eclipse` (release-grade subsystems that
  are effectively public-facing).

**Must-cover usecase doctests (all four):**

1. **Tropical natal chart** — build a full tropical natal chart from
   date/time/location: body placements, signs, aspects, houses, angles
   (Asc/MC). Home crate: `pleiades-core` façade.
2. **Sidereal chart + ayanamsa** — sidereal chart with an explicit ayanamsa
   mode selected (e.g. Lahiri), showing how to choose from the ayanamsa
   catalog. Home crate: `pleiades-core` façade (with `pleiades-ayanamsa`
   selection).
3. **House system selection** — compute house cusps under a chosen system
   (Placidus, Whole Sign, Koch, Porphyry, …) from the house catalog, including
   `AscMc` chart points. Home crate: `pleiades-houses`.
4. **Time/coords + corrections** — civil UTC/UT1 → TT/TDB conversion, sidereal
   time, apparent-vs-mean place, topocentric correction (the lower-level
   building blocks). Home crates: `pleiades-time` / `pleiades-apparent`.

**Delivery model:** apply + verify in one pass. The workflow edits doc comments,
adds doctests, and adds the lint directly, verifying each crate compiles and its
doctests pass before finishing. Review is via the final diff and report.

**Enforcement:** add `#![deny(missing_docs)]` to each of the 13 crates.

**Out of scope:**

- `pleiades-cli`, `pleiades-data`, `pleiades-validate` (contributor tooling).
- Any API or behavior changes — this is docs + doctests + one lint only.
- New validation gates beyond the `missing_docs` lint.

## Design

### Guiding principle: the accuracy audit catches overclaims

The sharp part of this work is not prose polish; it is ensuring doc comments do
not claim more than the code and gates deliver. For every accuracy/capability
claim found in a doc comment, the audit cross-checks it against three sources:

- the crate's actual code / behavior,
- the README's per-backend limits section, and
- the relevant validation gates (`validate-*`) and their documented residuals.

Overclaims are corrected to match reality. Under-documented items get accurate,
scoped descriptions. Where a claim is genuinely backend-specific, the doc must
say which backend/path it applies to.

### Deterministic work-list from the compiler

Once `#![deny(missing_docs)]` is added to a crate, the compiler emits the
authoritative list of undocumented **reachable** public items. This makes the
`missing_docs` work-list deterministic rather than guesswork — many of the raw
`pub` items in a crate are unreachable internals the lint will not flag. Agents
document exactly the items the compiler reports.

### Workflow phases

The work is executed as a single multi-agent workflow with four phases.

**Phase 1 — Map (parallel, read-only, one agent per crate).**
Each agent returns a structured map for its crate:

- the public API surface,
- existing doc comments, with any accuracy issues (especially overclaims)
  flagged and cross-checked against README/gates,
- which of the four usecases (if any) the crate anchors.

No edits in this phase.

**Phase 2 — Document + enforce (dependency-tiered; barrier between tiers).**
Crates are grouped into dependency tiers, leaf crates first (`pleiades-types`)
through to the `pleiades-core` façade last. Within a tier, agents run in
parallel on disjoint crates; the barrier between tiers ensures a crate is only
documented after its dependencies are stable. Each agent:

- applies the accuracy fixes identified in Phase 1,
- adds `#![deny(missing_docs)]`,
- documents every compiler-flagged item,
- self-verifies with `cargo test --doc -p <crate>` and `cargo doc -p <crate>`.

**Phase 3 — Usecase doctests (parallel, one agent per usecase).**
The four must-cover usecases are authored as runnable doctests at their natural
home crates (natal & sidereal at the `pleiades-core` façade; houses at
`pleiades-houses`; time/coords at `pleiades-time` / `pleiades-apparent`). Each
agent verifies its own doctest compiles and runs.

**Phase 4 — Integration verify + synthesis (serial).**
Run the full `cargo test --workspace` and `mise run docs`
(`RUSTDOCFLAGS="-D warnings"`). Fix any stragglers — cross-crate `missing_docs`
the per-crate pass missed, re-exports, doctests that pass individually but
interact. Produce a final report: per-crate count of items documented, accuracy
corrections made, and usecase doctests added.

### Concurrency and correctness

- Cargo's `target/` lock serializes concurrent builds — this is safe (agents
  queue) but not parallel; correctness is preserved.
- Dependency-tiering prevents a crate being documented against churning
  dependencies.
- Phase 4's full-workspace verify is the single source of truth for
  "everything compiles, all doctests pass, `-D warnings` is clean."

### Known risk: `pleiades-jpl` size

`pleiades-jpl` has by far the largest raw public surface. If its **reachable**
public surface (what `deny(missing_docs)` actually flags) is genuinely large,
the tier handling it splits the crate across multiple agents by module so no
single agent context is overloaded.

## Success criteria

- `#![deny(missing_docs)]` present in all 13 crates, and the workspace builds
  clean under it.
- `cargo test --workspace` passes, including all doctests.
- `mise run docs` passes with `-D warnings`.
- The four usecase doctests exist, run, and demonstrate their flows
  end-to-end.
- No doc comment overclaims relative to the README limits and validation gates
  (verified in the Phase 1 audit and Phase 4 synthesis).
- No changes to public API signatures or runtime behavior.

## Alternatives considered

- **Two-phase global (audit report → apply).** Rejected: the chosen delivery is
  apply-and-verify in one pass, and a global apply loses per-crate verification
  isolation.
- **Usecase-first organization.** Rejected: it produces good doctest
  narratives but gives a weak coverage guarantee for the accuracy audit and the
  `missing_docs` sweep, which are the bulk of the work.
