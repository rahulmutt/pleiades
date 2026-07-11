# Report-surface relocation — Slice C design

Sub-slice of the [workspace report-surface relocation program](2026-07-10-report-surface-relocation-design.md).
Slices A and B delivered (2026-07-10 / 2026-07-11, merged in PRs #19 and #20);
this slice relocates the report-prose layer of **`pleiades-data`** into
`pleiades-validate`, and resolves the one calculation-path coupling this crate
carries (the packaged-data backend metadata).

Branch: `feat/report-surface-relocation-slice-c`. One implementation plan.

## Goal

Move `pleiades-data`'s free report-prose renderers (~5.7k LOC; 106 exported
`*_for_report` / `summary_line` / `format_*_summary` functions) into
`pleiades-validate/src/posture/data/`, **verbatim** (byte-identical rendered
text), together with their tests; rebuild the calculation-path strings that
still consume report helpers (the coupling); repoint every consumer. This is a
**pure relocation** slice:

- **No output changes.** Every moved renderer produces exactly the prose it
  produced before; fnv1a64 release-bundle checksum *values* do not change.
- **No behavior, gate, threshold, corpus, or version changes.** Compatibility
  profile stays `0.7.13`; API-stability profile stays `0.3.0`. The workspace
  `0.4.0` bump lands once, after Slice D.
- **Deletions/demotions** are limited to the four over-exposed, no-non-test-consumer
  items the program spec already assigned to this crate (see below).

## The stay / move partition (empirically confirmed)

Validate and the CLI consume the structured `Packaged*Summary` structs (which
stay) and render them in validate's own `render/` modules, or call **free**
`*_for_report` / `summary_line` functions. The boundary is crisp:

**MOVE** — free functions that render prose:
- `…_summary_for_report()` and `…_for_report()` free functions,
- `format_validated_…_for_report()` free functions,
- `format_*_summary(&Struct) -> String` prose formatters,
- `packaged_body_coverage_summary() -> String` (a `String`-returning prose
  renderer despite the un-suffixed name; its `_details()` struct counterpart
  stays).

**STAY** — structured data, release-gate core, and self-description:
- the 29 `Packaged*Summary` structs (public fields) and their
  `*_summary_details()` **constructors** (they read crate-private internals:
  artifact bytes, fit samples, corpus statics),
- the `&'static str` self-description accessors —
  `packaged_request_policy_summary()`, `packaged_frame_treatment_summary()`,
  `packaged_artifact_storage_summary()`, `packaged_artifact_access_summary()`
  (structured source for the coupling rebuild; see below),
- **release-gate data** in `thresholds.rs`: `accuracy_ceiling`,
  `PACKAGED_BUDGETS` (enforced by gates, not rendered),
- the **accuracy-baseline measurement core** in `accuracy_baseline.rs`:
  `accuracy_baseline_against`, `packaged_artifact_accuracy_baseline`,
- the **regeneration pipeline**: `regenerate_packaged_artifact*` calc functions,
- corpus checksum / manifest primitives and all catalog/descriptor data structs
  and their public fields.

## Module map (subdir per crate)

Following the layout chosen in Slice B — one posture subdirectory per source
crate, mirroring the source crate's module structure. Slice A's flat
`posture/backend_policy.rs` and Slice B's `houses/ ayanamsa/ vsop87/ elp/
compression/` subdirs are left untouched.

```
crates/pleiades-validate/src/posture/
  data/
    mod.rs
    coverage/{body,fit,generation,profile,regen,target,threshold}.rs
    lookup.rs
    regenerate.rs
    thresholds.rs
    accuracy_baseline.rs
```

| posture subdir | Moved from | Content and notes |
| --- | --- | --- |
| `data/coverage/*.rs` | `pleiades-data/src/coverage/{body,fit,generation,profile,regen,target,threshold}.rs` | The bulk of the slice: `profile.rs` (24 renderers), `fit.rs` (15), `target.rs` (13), `threshold.rs` (11), `regen.rs` (6), `generation.rs` (6), `body.rs` (2). The `*_summary_details()` constructors and `Packaged*Summary` structs stay; only the prose renderers move. |
| `data/lookup.rs` | `pleiades-data/src/lookup.rs` | 17 renderers, including the four coupling `_for_report` functions (see below). The `&'static str` self-description accessors and `packaged_body_claims`/lookup calc paths stay. |
| `data/regenerate.rs` | `pleiades-data/src/regenerate.rs` | 9 renderers. The `regenerate_packaged_artifact*` calc functions stay. |
| `data/thresholds.rs` | `pleiades-data/src/thresholds.rs` | 1 renderer wrapper (`packaged_artifact_thresholds_summary_for_report`). `accuracy_ceiling` and `PACKAGED_BUDGETS` stay (release-gate data). |
| `data/accuracy_baseline.rs` | `pleiades-data/src/accuracy_baseline.rs` | 2 renderer wrappers. The measurement core (`accuracy_baseline_against`, `packaged_artifact_accuracy_baseline`) stays. |

## Coupling — packaged-data backend metadata

`pleiades-data/src/backend.rs:221-227` — `PackagedDataBackend::metadata()`
builds `BackendProvenance.data_sources` and embeds **five rendered strings**:

```rust
data_sources: vec![
    packaged_body_coverage_summary(),                 // body.rs — String renderer
    packaged_request_policy_summary_for_report(),     // lookup.rs
    packaged_frame_treatment_summary_for_report(),    // lookup.rs
    packaged_artifact_storage_summary_for_report(),   // lookup.rs
    packaged_artifact_access_summary_for_report(),    // lookup.rs
],
```

This is a **calculation-path** consumer of report helpers: it would block those
helpers' move and violate the program's dependency-direction invariant.

Resolution (mirrors Slice B coupling 3): `backend.rs` rebuilds those five
strings inline from the structured data that stays — the four `&'static str`
self-description accessors (`packaged_request_policy_summary()`,
`packaged_frame_treatment_summary()`, `packaged_artifact_storage_summary()`,
`packaged_artifact_access_summary()`) and the `packaged_body_coverage_summary_details()`
struct fields — byte-identical to the current rendering. The five report helpers
then move with the rest of their families.

Verification: a **pre-move golden fixture** captures the current
`metadata().provenance.data_sources` before the change; a post-move equality
test pins the rebuilt vector to that fixture, so any drift fails closed. This
mirrors the coupling-2/coupling-3 rebuild-test pattern from Slices A and B.

**Ordering:** the coupling rebuild + fixture must land *before* the renderer
moves that remove those five symbols from `pleiades-data`, so the crate's own
`backend.rs` compiles once the helpers are gone.

## Deletions / demotions (program-spec trivia)

All four items below were confirmed to have **0 non-test consumers** in
`pleiades-validate` / `pleiades-cli`:

- **Demote to `pub(crate)`** (over-exposed, no non-test consumer):
  - `packaged_mixed_frame_batch_parity_summary_for_report` (`lookup.rs`),
  - `packaged_body_coverage_summary_details` (`coverage/body.rs`),
  - `packaged_artifact_fit_channel_outlier_summary_details` (`coverage/fit.rs`),
  - `eros_self_consistency_max_longitude_arcsec` (`accuracy_baseline.rs`).

These are demotions, not relocations — the `_details` constructors and the eros
accessor stay in `pleiades-data` (they read crate-private internals) but lose
their unnecessary `pub`.

## Visibility

`pub(crate)` by default, with `#![allow(dead_code)]` on the posture modules as in
Slices A and B (verbatim relocations retain surface without an in-crate caller).
`pub` only where a genuine cross-crate runtime consumer exists — audited during
the repoint and expected to be the set of renderers validate re-exports through
its existing public render surface, and any renderer the CLI calls at runtime.

## Consumer migrations

- **pleiades-validate**: repoint every moved renderer call
  (~50 `pleiades_data::*_for_report` / `summary_line` call sites) to
  `crate::posture::data::…`. Release-bundle checksum pins in `release/bundle.rs`
  and `release/bundle_verify_helpers.rs` repoint to the local (moved) functions —
  **checksum values unchanged**, because the rendered text is byte-identical (any
  drift is a defect in the move, not a checksum to regenerate). Moved report
  tests land beside their renderers.
- **pleiades-cli**: repoint any `pleiades_data` report-symbol imports to
  `pleiades_validate` (CLI already depends on validate — no Cargo change); CLI
  summary-command tests repoint to the surviving `pub` validate renderers.
- **pleiades-data's own tests**: report tests move with the code; contract,
  codec, coverage-calc, and lookup-calc tests stay in place.

## Invariants (from the program spec, enforced here)

1. **Byte-identical release-bundle text.** fnv1a64 checksum values in the bundle
   do not change. `release-smoke` proves it.
2. **Compatibility profile stable.** Profile id stays `0.7.13`;
   `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged.
3. **No behavior changes.** No calculation path consults these summaries once the
   backend-metadata coupling is decoupled to structured data. Gates
   (`validate-corpus`, claims-audit, numeric gates), corpora, and thresholds
   are unchanged.
4. **Dependency direction.** Nothing below `pleiades-validate` gains a
   dependency on it. The `&'static str` self-description accessors and the
   `Packaged*Summary` structs stay in `pleiades-data` for exactly this reason.

## Task breakdown

One branch, one plan. Proposed tasks:

1. **coupling** — capture the `metadata().provenance.data_sources` pre-move
   golden fixture; rebuild the five embedded strings inline in `backend.rs` from
   the structured `&'static str` accessors and `packaged_body_coverage_summary_details()`
   fields; add the equality test. **Must land before the renderer moves** that
   remove those five symbols from `pleiades-data`.
2. **coverage renderers** — move the prose renderers from
   `coverage/{body,fit,generation,profile,regen,target,threshold}.rs` →
   `posture/data/coverage/`, move their tests. (Largest task: profile 24 +
   fit 15 + target 13 + threshold 11 + regen 6 + generation 6 + body 2.)
3. **lookup + regenerate renderers** — move `lookup.rs` (17, incl. the four
   coupling `_for_report`) and `regenerate.rs` (9) → `posture/data/`, move tests.
4. **thresholds + accuracy_baseline renderer wrappers** — move the 1 + 2 renderer
   wrappers → `posture/data/`; the release-gate data and measurement core stay.
5. **demotions** — downgrade the three `_details`/parity renderers and the eros
   accessor to `pub(crate)`.
6. **consumer repoint + bundle-pin sweep** — finish validate/CLI repoints;
   confirm every checksum pin value unchanged.
7. **close-out** — `mise run ci` green; grep assertion
   (`grep -rn "pub fn .*_for_report" crates/pleiades-data/src` returns nothing);
   CHANGELOG entry; PLAN.md status refresh.

Task 1 (coupling) precedes tasks 2–4 (renderer moves). Tasks 2–5 are otherwise
independent of one another; 6 depends on 1–5; 7 is last.

## Verification

- `mise run ci` — fmt, clippy `-D warnings`, `cargo test --workspace
  --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check,
  release-smoke, claims-audit.
- `release-smoke` proves bundle checksum values unchanged (invariant 1).
- Coupling pre/post equality fixture (invariant 3).
- Grep assertion:
  `grep -rn "pub fn .*_for_report" crates/pleiades-data/src` returns nothing
  after the slice.

## Non-goals

- No renaming or redesign of the moved renderers' output (pure relocation).
- No version bump (compatibility `0.7.13`, API-stability `0.3.0` unchanged; the
  workspace `0.4.0` bump is deferred to post-Slice-D).
- No touching Slice D (`pleiades-jpl`) — independent, separately branched.
- No decomposition of `pleiades-validate` itself.

## Risks

- **Checksum-pin churn** in validate's bundle-verify tests — expected and
  mechanical; values must not change (invariant 1). Any change is a move defect.
- **Missed `pub` runtime consumer** downgraded to `pub(crate)` — caught by
  `cargo build` / CI across the workspace before merge.
- **Coupling rebuild drift** (five strings) — mitigated by the pre/post equality
  fixture; the four `&'static str` accessors make most of the rebuild a direct
  reuse rather than a re-derivation.
- **coverage breadth** (`profile.rs` alone carries 24 renderers) — the largest
  renderer count in the slice, but each is a verbatim move with tests that move
  with it; byte-identity is mechanically checkable.
