# Report-surface relocation — Slice D design

Sub-slice of the [workspace report-surface relocation program](2026-07-10-report-surface-relocation-design.md).
Slices A, B, and C delivered (2026-07-10 / 2026-07-11, merged in PRs #19, #20,
#21); this slice relocates the report-prose layer of **`pleiades-jpl`** — the
largest and last slice — into `pleiades-validate`, and resolves the couplings
this crate carries (two in `backend.rs`, the `production_generation.rs` report
half, and one dependent renderer in `pleiades-data`).

Branch: `feat/report-surface-relocation-slice-d`. One implementation plan.

## Goal

Relocate **all report prose** out of `pleiades-jpl` (and the one dependent
`pleiades-data` renderer) into `pleiades-validate/src/posture/jpl/`,
**verbatim** (byte-identical rendered text), together with the tests that
exercise it; promote the crate-private corpus/evidence accessors the moved
renderers read to public structured API; repoint every consumer. This is a
**pure relocation** slice:

- **No output changes.** Every moved renderer produces exactly the prose it
  produced before; fnv1a64 release-bundle checksum *values* do not change.
- **No behavior, gate, threshold, corpus, or version changes.** Compatibility
  profile stays `0.7.13`; API-stability profile stays `0.3.0`;
  `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged. The workspace
  `0.3.0 → 0.4.0` bump + release is a **separate follow-up step** after this
  slice merges (interim main stays green but unreleased).
- **Pure end-state.** Unlike Slices B/C — which moved only *free* prose
  renderers and left inherent `summary_line()`/`Display` methods on the data
  structs — Slice D additionally **strips inherent rendering** off every jpl
  evidence struct into validate free functions, so no functional crate renders
  prose. This is the deliberate, program-owner-approved scope for the final
  slice.

## The novelty vs. Slices B/C — inherent-method rendering

Slices B/C relied on a crisp "free functions render prose (MOVE); structs are
structured data (STAY)" partition. `pleiades-jpl` breaks that cleanly: its
report prose is entangled as **inherent methods** — `summary_line()`,
`validated_summary_line()`, and `impl fmt::Display` — on the same structs whose
`validate()` / `label()` / `*_details()` calc-and-gate half must stay. Measured
in `reference_summary/` alone: **129** free `*_for_report` functions, **~175**
inherent `summary_line`/`validated_summary_line` methods, and **~168**
`Display` impls.

An inherent method cannot be relocated to another crate, and the orphan rule
forbids validate from re-`impl`ing `Display` for a foreign (`pleiades_jpl`)
struct. Resolution: **strip the rendering methods off the staying structs and
re-home each as a free function** `…_for_report(&Struct) -> String` in
`posture/jpl/`, reading the struct's public fields. Byte-identical output;
consumers that used `Display`/`.to_string()`/`.summary_line()` switch to the
free function.

## Stay / move partition

**STAY in `pleiades-jpl`** — structured data, gate/calc logic, self-description:

- Every evidence/summary struct with its public fields:
  `ReferenceSnapshot*Summary`, `ComparisonSnapshot*Summary`,
  `IndependentHoldout*Summary`, `SelectedAsteroidSource*Summary`,
  `ProductionGenerationBoundary*Summary`, `InterpolationQualitySample`,
  `SnapshotManifestSummary`, and peers — together with their
  `*_summary()` / `*_details()` **constructors** (they read crate-private
  corpus internals), their `validate()` / `label()` methods, and their
  `…ValidationError` enums (with the `Display` impls on the *errors*, which are
  contract surface, not report prose).
- The calculation surfaces: `corpus.rs`, `snapshot.rs`, `requests.rs`,
  `spk/*`, `ingest/*`, `backend.rs`'s `EphemerisBackend` impls and
  fixture/interpolation machinery, `production_generation.rs`'s request-corpus
  builders (`production_generation_snapshot_entries`,
  `production_generation_snapshot_requests`,
  `production_generation_boundary_requests`,
  `production_generation_boundary_request_corpus`, …), and the
  `data/selected_asteroid_*` data statics.

**MOVE to `pleiades-validate/src/posture/jpl/`** — all report prose:

- The **129** free `*_for_report` renderers in `reference_summary/*` and their
  tests (verbatim).
- The **9** non-`reference_summary` free renderers: `production_generation.rs`
  (7: `…_boundary_summary_for_report`, `…_boundary_source_summary_for_report`,
  `…_boundary_window_summary_for_report`,
  `…_boundary_body_class_coverage_summary_for_report`,
  `…_boundary_request_corpus_summary_for_report`,
  `…_boundary_request_corpus_equatorial_summary_for_report`,
  `validated_…_boundary_request_corpus_equatorial_summary_for_report`) and
  `data/selected_asteroid_*.rs` (2:
  `selected_asteroid_source_2451917_summary_for_report`,
  `selected_asteroid_source_2378498_summary_for_report`).
- **Stripped inherent rendering** — every `summary_line()`,
  `validated_summary_line()`, and `impl Display` on the staying evidence
  structs, re-homed as a free `…_for_report(&Struct) -> String` in
  `posture/jpl/`, byte-identical.

## Accessor-promotion prerequisite

The moved renderers, once in validate, must read the raw corpus/evidence data
they previously reached as crate-private jpl internals. Promote to `pub`:

- The 8 `pub(crate)` accessors in `backend.rs`: `snapshot_entries`,
  `comparison_snapshot_entries`, `snapshot_bodies`, `snapshot_instants`,
  `comparison_body_list`, `reference_asteroid_evidence_list`,
  `interpolation_quality_sample_list`, `is_comparison_body`.
- The `REFERENCE_SNAPSHOT_*_EPOCH_JD` boundary-epoch constants in `backend.rs`
  that boundary renderers consume.
- Any struct field a moved renderer reads that is not already `pub` (audited
  during the strip; most are already public).

This is the structural difference from Slice C, where the `*_details()`
constructors stayed in `pleiades-data` and thus no accessor promotion was
needed. Here the rendering leaves the crate while the constructors stay, so the
low-level corpus data the rendering reads must become public.

## Couplings and resolutions

1. **`backend.rs` — `InterpolationQualitySample` / `SnapshotManifestSummary`.**
   `InterpolationQualitySample` is the element type of the promoted
   `interpolation_quality_sample_list()` accessor and is produced on jpl's
   internal evidence-generation path; `SnapshotManifestSummary` is used on the
   corpus manifest-provenance validation path. Both **stay**. Strip their
   `Display` / `summary_line()` / `format_*_for_report` rendering into
   `posture/jpl/backend.rs`, reading promoted public fields. A **pre-move
   golden fixture** pins the rendered text; a post-move equality test fails
   closed on drift.

2. **`production_generation.rs` — report half.** The request-corpus builders
   and the boundary summary structs (`ProductionGenerationBoundary*Summary`)
   with their `validate()`/`label()` **stay**; the 7 free `*_for_report`
   functions and the structs' inherent `summary_line()`/`validated_summary_line()`
   **move**. Golden fixture for the request-corpus-adjacent boundary rendering.

3. **`data/selected_asteroid_*.rs` — renderer halves.** The
   `SelectedAsteroidSource*Summary` data structs and their `*_summary()`
   constructors **stay**; the 2 `*_summary_for_report` renderers move.

4. **`pleiades-data` re-open (bounded to one struct).**
   `crates/pleiades-data/src/coverage/target.rs`'s
   `PackagedArtifactPhase2CorpusAlignmentSummary` holds ~13 jpl
   `reference_summary` evidence structs as public fields and renders them: its
   `summary_line()` (line ~181) and `impl Display` (line ~341) call each
   field's inherent `.summary_line()` plus the free jpl renderer
   `format_production_generation_boundary_source_summary`. Once the jpl inherent
   methods are stripped and that free renderer moves, this rendering can only
   live in validate. Resolution: **relocate the struct's rendering** into
   `posture/jpl/` as
   `packaged_artifact_phase2_corpus_alignment_summary_for_report(&s) -> String`,
   which renders each jpl field via validate's new posture/jpl free functions.
   The struct itself, its `validate()` (gate logic), its `_details()`
   constructor, and its `ValidationError` **stay in `pleiades-data`**. Pre/post
   golden fixture pins the rebuilt line.

   This is the only `pleiades-data` file touched; no other functional crate
   consumes jpl report prose (`pleiades-time` / `pleiades-events` use only the
   `spk::corpus_manifest::corpus_checksum64` calc primitive).

5. **Dependency direction (invariant).** Nothing below `pleiades-validate`
   gains a dependency on it. jpl and data retain only data structs +
   constructors + `validate()`; all prose lives in validate.

## Module map (subdir per crate)

Following the layout of Slices B/C — one posture subdirectory per source crate,
mirroring the source crate's module structure. Slice A/B/C subdirs are left
untouched.

```
crates/pleiades-validate/src/posture/
  jpl/
    mod.rs
    reference_snapshot/{core,boundaries}.rs   # mirrors src/reference_summary/reference_snapshot/*
    comparison.rs
    holdout.rs
    jpl_posture.rs
    reference_asteroid.rs
    selected_asteroid.rs
    production_generation.rs                   # the 7 report-half renderers + stripped methods
    backend.rs                                 # InterpolationQualitySample / SnapshotManifestSummary renderers
    data_phase2_alignment.rs                   # the relocated pleiades-data renderer
```

The exact intra-`jpl/` file granularity may mirror the source submodules more
finely (e.g. splitting `reference_snapshot/` into `core/{coverage,evidence,
general_a,general_b,parity}` and `boundaries/{era_a..d}`) — the plan settles it;
the constraint is a verbatim move with tests co-located beside their renderers.

## Visibility

`pub(crate)` by default, with `#![allow(dead_code)]` on the posture modules as
in Slices A–C (verbatim relocations retain surface without an in-crate caller).
`pub` only where a genuine cross-crate runtime consumer exists — the set of
renderers validate re-exports through its existing public render surface, and
any renderer the CLI calls at runtime — audited during the repoint.

## Consumer migrations

- **pleiades-validate**: repoint every moved renderer call site (the
  `pleiades_jpl::*_for_report` / `*summary_line` imports currently aliased in
  `lib.rs` and called in `report.rs`) to `crate::posture::jpl::…`.
  Release-bundle checksum pins in `release/bundle.rs` /
  `release/bundle_verify_helpers.rs` repoint to the local (moved) functions —
  **checksum values unchanged**, because the rendered text is byte-identical
  (any drift is a defect in the move, not a checksum to regenerate). Moved
  report tests land beside their renderers.
- **pleiades-cli**: repoint any `pleiades_jpl` report-symbol imports to
  `pleiades_validate` (CLI already depends on validate — no Cargo change); CLI
  summary-command tests repoint to the surviving `pub` validate renderers.
- **pleiades-data**: `coverage/target.rs` drops the
  `PackagedArtifactPhase2CorpusAlignmentSummary` rendering (`summary_line`,
  `Display`) and the `pleiades_jpl::format_…` call; validate renders it
  instead. The struct, `validate()`, `_details()` constructor, and error enum
  stay. No Cargo dependency changes.
- **pleiades-jpl's own tests**: report tests move with the code; contract,
  codec, corpus-calc, snapshot, spk, and ingest tests stay in place.

## Invariants (from the program spec, enforced here)

1. **Byte-identical release-bundle text.** fnv1a64 checksum values in the bundle
   do not change. `release-smoke` proves it.
2. **Compatibility profile stable.** Profile id stays `0.7.13`;
   `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged.
3. **No behavior changes.** No calculation path consults these summaries; the
   backend metadata, corpus manifest validation, and gate data are unchanged.
   Gates (`validate-corpus`, claims-audit, numeric gates), corpora, and
   thresholds are untouched.
4. **Dependency direction.** Nothing below `pleiades-validate` gains a
   dependency on it. jpl evidence structs + constructors stay in jpl; the
   `pleiades-data` aggregator struct + its `validate()` stay in data.

## Task breakdown

One branch, one plan. Proposed tasks:

1. **accessor + field promotion** — promote the 8 `pub(crate)` accessors and
   the `REFERENCE_SNAPSHOT_*_EPOCH_JD` boundary constants (and any renderer-read
   struct fields) to `pub`. Lands first so the moved renderers compile once in
   validate.
2. **backend.rs coupling** — capture the pre-move golden fixture for
   `InterpolationQualitySample` / `SnapshotManifestSummary` rendering; strip the
   inherent `Display`/`summary_line` and free `*_for_report` into
   `posture/jpl/backend.rs`; add the equality test.
3. **reference_summary bulk move** — move the 129 free renderers + the stripped
   inherent `summary_line`/`validated_summary_line`/`Display` + their tests into
   `posture/jpl/`. Largest task; split per family
   (comparison / holdout / jpl_posture / reference_asteroid / reference_snapshot
   {core, boundaries} / selected_asteroid) so each sub-move stays reviewable.
4. **production_generation report half** — move the 7 renderers + inherent
   methods into `posture/jpl/production_generation.rs`; golden fixture for the
   boundary/request-corpus-adjacent rendering; the request-corpus builders stay.
5. **data/selected_asteroid renderers** — move the 2 `*_summary_for_report`
   renderers into `posture/jpl/selected_asteroid.rs`; the data statics +
   constructors stay.
6. **pleiades-data re-open** — capture the pre-move golden fixture for
   `PackagedArtifactPhase2CorpusAlignmentSummary::summary_line()`/`Display`;
   relocate that rendering into `posture/jpl/data_phase2_alignment.rs`; drop it
   (and the `pleiades_jpl::format_…` call) from `coverage/target.rs`; add the
   equality test. The struct + `validate()` + `_details()` stay.
7. **consumer repoint + bundle-pin sweep** — finish validate/CLI repoints;
   confirm every checksum pin value unchanged.
8. **close-out** — `mise run ci` green; grep assertions (below); CHANGELOG
   entry; PLAN.md status refresh noting only the `0.4.0` release step remains
   after Slice D.

Task 1 precedes the renderer moves. Tasks 2–6 are otherwise independent of one
another; 7 depends on 1–6; 8 is last.

## Verification

- `mise run ci` — fmt, clippy `-D warnings`, `cargo test --workspace
  --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check,
  release-smoke, claims-audit.
- `release-smoke` proves bundle checksum values unchanged (invariant 1).
- Coupling pre/post equality fixtures (invariant 3): backend pair,
  production_generation boundary rendering, data phase-2 alignment line.
- Grep assertions after the slice:
  - `grep -rn "pub fn .*_for_report" crates/pleiades-jpl/src` returns nothing;
  - no `summary_line` / `validated_summary_line` / `impl fmt::Display` remains
    on jpl evidence structs (contract `Display`s on `…ValidationError` enums and
    on non-report types are excluded from the assertion);
  - `crates/pleiades-data/src/coverage/target.rs` no longer renders
    `PackagedArtifactPhase2CorpusAlignmentSummary` nor calls any
    `pleiades_jpl::*_for_report` / `format_*_summary`.

## Non-goals

- No renaming or redesign of the moved renderers' output (pure relocation).
- **No version bump** in this slice (compat `0.7.13`, api-stability `0.3.0`
  unchanged). The workspace `0.3.0 → 0.4.0` bump + release-plz cut is a separate
  follow-up task after Slice D merges — it finalizes the api-stability prose,
  the `0.4.0` CHANGELOG migration entry, and posture identifiers once, for the
  whole program.
- No wholesale re-do of Slice C: only the single
  `PackagedArtifactPhase2CorpusAlignmentSummary` renderer in `pleiades-data`
  is re-opened (it is the sole below-validate consumer of jpl report prose).
- No decomposition of `pleiades-validate` itself (it absorbs the largest single
  chunk of the program; a future split into render/gate crates is out of scope).

## Risks

- **Slice size (~23.8k LOC moved)** — the largest slice. Mitigated by verbatim
  moves, the byte-identity invariant, tests that move with their renderers, and
  the per-family task split in task 3.
- **Inherent-method stripping breadth (~175 methods + ~168 Display impls)** —
  more invasive than B/C's free-function moves. Mitigated by byte-identity being
  mechanically checkable (`release-smoke`) and by the golden fixtures on the
  coupled rebuilds; each strip is a field-reading free function whose output is
  pinned.
- **Accessor over-exposure** — promoting corpus accessors to `pub` widens jpl's
  public API. Accepted for the duration (the `0.4.0` breaking bump is the
  program's purpose); the accessors expose already-committed corpus statics, no
  new data.
- **Checksum-pin churn** in validate's bundle-verify tests — expected and
  mechanical; values must not change (invariant 1). Any change is a move defect.
- **Coupling rebuild drift** (backend pair, production_generation, data
  phase-2) — mitigated by the pre/post equality fixtures; the promoted public
  fields make each rebuild a direct field read, not a re-derivation.
- **Missed `pub` runtime consumer** downgraded to `pub(crate)` — caught by
  `cargo build` / CI across the workspace before merge.
