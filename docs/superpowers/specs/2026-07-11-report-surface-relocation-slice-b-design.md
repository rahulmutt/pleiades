# Report-surface relocation — Slice B design

Sub-slice of the [workspace report-surface relocation program](2026-07-10-report-surface-relocation-design.md).
Slice A delivered (2026-07-10, merged in PR #19); this slice relocates the
report-prose layer of five functional crates — **houses, ayanamsa, vsop87,
elp, compression** — into `pleiades-validate`, and resolves the program's third
coupling edge (elp backend metadata).

Branch: `feat/report-surface-relocation-slice-b`. One implementation plan.

## Goal

Move the free report-prose functions from these five crates into
`pleiades-validate/src/posture/<crate>/`, **verbatim** (byte-identical rendered
text), together with their tests; rebuild the one calculation-path string that
still consumes a report helper (coupling 3); repoint every consumer. This is a
**pure relocation** slice:

- **No deletions.** The two "deleted outright" targets the program spec
  assigned near these crates —
  `pleiades-ayanamsa::ayanamsa_thresholds_summary_for_report` and
  `pleiades-houses::house_thresholds_summary_for_report` — were already removed
  in Slice A (commit `1eb895b6`). Slice B removes nothing further.
- **No output changes.** Every moved renderer produces exactly the prose it
  produced before; fnv1a64 release-bundle checksum *values* do not change.
- **No behavior, gate, threshold, corpus, or version changes.** Compatibility
  profile stays `0.7.13`; API-stability profile stays `0.3.0`. The workspace
  `0.4.0` bump lands once, after Slice D.

## The stay / move partition (empirically confirmed)

Validate and the CLI never call an inherent `.summary_line()` **method** on any
of these five crates' structs at runtime — they consume the structured
`*Summary` structs (which stay) and render them in validate's own `render/`
modules, or call **free** `*_for_report` / `summary_line` functions. So the
boundary is crisp:

**MOVE** — free functions that render prose:
- `…_summary_for_report()` and `…_for_report()` free functions,
- `format_validated_…_for_report()` free functions,
- `format_*_summary(&Struct) -> String` prose formatters.

**STAY** — structured data and its self-description:
- the `*_summary()` / `*_details()` **constructors** that build `*Summary`
  structs (they read crate-private internals),
- the descriptor/summary structs' own inherent `summary_line()` /
  `validated_summary_line()` **methods** (matching Slice A leaving
  `CompatibilityProfile`'s `*_summary_line()` methods in `pleiades-core`),
- `pleiades-elp::lunar_theory_frame_treatment_summary_details()` — returns
  `pleiades_backend::FrameTreatmentSummary`, which stays in `pleiades-backend`
  per the program's dependency-direction invariant,
- all catalog / descriptor / source-docs data structs and their public fields.

## Module map (subdir per crate)

Following the layout chosen for this slice — one posture subdirectory per source
crate, mirroring the source crate's own module structure. Slice A's flat
`posture/backend_policy.rs` is left untouched.

```
crates/pleiades-validate/src/posture/
  mod.rs                 (add: houses, ayanamsa, vsop87, elp, compression)
  backend_policy.rs      (from Slice A — unchanged)
  houses/mod.rs
  ayanamsa/mod.rs
  vsop87/{spec,audit,documentation,request_corpus,evidence,batch_parity}.rs
  elp/{lib,catalog,evidence,source}.rs
  compression/mod.rs
```

| posture subdir | Moved from | Content and notes |
| --- | --- | --- |
| `houses/mod.rs` | `pleiades-houses/src/catalog/mod.rs` | Four free renderers: `house_system_code_aliases_summary_line` (+ `validated_house_system_code_aliases_summary_line`), `house_formula_families_summary_line`, `latitude_sensitive_house_failure_modes_summary_line`. `validated_house_system_code_aliases_summary_line` is validate's one runtime consumer (`render/text/catalog.rs:387`) → stays `pub`, repoint the call. `HouseCatalogValidationSummary` and its constructor stay. |
| `ayanamsa/mod.rs` | `pleiades-ayanamsa/src/lookup.rs` | `validated_provenance_summary_for_report` and its `format_validated_*` helper. The `provenance_summary()` constructor and `provenance_sample_ayanamsas()` stay — validate already rebuilds the provenance line itself from structured data (coupling 2, resolved in Slice A). `AyanamsaCatalogValidationSummary` constructor stays. |
| `vsop87/*.rs` | `pleiades-vsop87/src/source_docs/{spec,audit,documentation,request_corpus,evidence,batch_parity}.rs` | Every `…_for_report`, `format_validated_…_for_report`, and `format_*_summary(&Struct)` renderer across the six source-docs modules (the largest count; ~batch_parity alone has 8 parity families). The `*_summary()` struct constructors and the source-docs catalog stay public as structured data — validate's `render/text/evidence.rs` renders `Vsop87SourceDocumentation[Health]Summary` itself and only needs the structs. |
| `elp/*.rs` | `pleiades-elp/src/{lib,catalog,evidence,source}.rs` | The crux — ~38 renderers across the lunar-theory catalog / capability / limitations / source-selection / evidence / reference-batch-parity families. Genuine runtime consumers to repoint: validate `release/bundle.rs` (bundle assembly), `render/summary/writers.rs`; CLI `cli.rs:529` calls `lunar_theory_source_selection_summary_for_report()` at runtime → that symbol stays `pub`. Struct constructors (`lunar_theory_source_summary`, `lunar_theory_source_family_summary`, `lunar_theory_catalog_summary`, `…_capability_summary`, `…_limitations_summary`, evidence `*_summary()` constructors) and `lunar_theory_frame_treatment_summary_details` stay. **Includes coupling 3 — see below.** |
| `compression/mod.rs` | `pleiades-compression/src/artifact.rs` | The `ArtifactProfileCoverageSummary` / `ArtifactResidualBodyCoverageSummary` **rendering**, recomputed validate-side from public `Artifact` / `CompressedArtifact` accessors. This is the one place the moved surface is a struct **method** (`.summary_line()`) rather than a free function: it becomes a validate free function taking the coverage-summary struct (or the artifact). Validate's `render/summary/artifact.rs` already takes `ArtifactResidualBodyCoverageSummary` + `CompressedArtifact`, so the rendering partly lives there already. The two coverage-summary structs and their `Artifact` accessor constructors stay. |

## Coupling 3 — elp backend metadata

`pleiades-elp/src/backend.rs:229` — `EphemerisBackend::metadata()` builds
`BackendProvenance.data_sources` and embeds
`lunar_theory_source_family_summary_for_report()` as one vector element. This is
a **calculation-path** consumer of a report helper: it would block the helper's
move and violate the program's dependency-direction invariant.

Resolution: elp rebuilds that exact string inline in `backend.rs` from the
structured `lunar_theory_source_family_summary()` fields (source identifier,
family label, supported/unsupported body counts — the same values the report
helper reads), byte-identical to the current rendering. The report helper then
moves with the rest of the family.

Verification: a **pre-move golden fixture** captures the current
`metadata().provenance.data_sources` (or the specific embedded element) before
the change; a post-move equality test pins the rebuilt string to that fixture,
so any drift fails closed. This mirrors the coupling-2 rebuild test pattern
established in Slice A.

## Consumer migrations

- **pleiades-validate**: repoint every moved renderer call to
  `crate::posture::<crate>::…`. Release-bundle checksum pins in
  `release/bundle.rs` and `release/bundle_verify_helpers.rs` repoint to the
  local (moved) functions — **checksum values unchanged**, because the rendered
  text is byte-identical (any drift is a defect in the move, not a checksum to
  regenerate). Moved report tests (`tests/render_catalog.rs` fragments and the
  crates' own report tests) land beside their renderers.
- **pleiades-cli**: `cli.rs` repoints `lunar_theory_source_selection_summary_for_report`
  to `pleiades_validate` (CLI already depends on validate — no Cargo change);
  `cli/tests/summary_commands.rs` repoints its elp/vsop87/houses report
  assertions to the surviving `pub` validate renderers.
- **Functional crates' own tests**: report tests move with the code; contract,
  catalog, and calculation tests stay in place.

## Visibility

`pub(crate)` by default. `pub` only where a genuine cross-crate runtime consumer
exists:
- `elp::lunar_theory_source_selection_summary_for_report` (CLI `cli.rs:529`),
- any moved renderer re-exported through validate's existing public render
  surface (audited during the repoint; expected to be a small set).

Everything else — the bulk, which is now exercised only by tests that move with
it — is `pub(crate)`, with `#![allow(dead_code)]` on the posture modules as in
Slice A's `backend_policy.rs` (verbatim relocations retain surface without an
in-crate caller).

## Invariants (from the program spec, enforced here)

1. **Byte-identical release-bundle text.** fnv1a64 checksum values in the bundle
   do not change. `release-smoke` proves it.
2. **Compatibility profile stable.** Profile id stays `0.7.13`;
   `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged.
3. **No behavior changes.** No calculation path consults these summaries (the
   sole calc-path consumer, coupling 3, is decoupled to structured data). Gates,
   corpora, thresholds unchanged.
4. **Dependency direction.** Nothing below `pleiades-validate` gains a
   dependency on it. `FrameTreatmentSummary` (backend) and
   `frame_treatment_summary_details` (elp) stay put for exactly this reason.

## Task breakdown

One branch, one plan. Proposed tasks:

1. **houses** — move 4 renderers → `posture/houses/`, move tests, repoint
   validate's one consumer.
2. **ayanamsa** — move provenance report renderer → `posture/ayanamsa/`, move
   tests.
3. **vsop87** — move source-docs renderers → `posture/vsop87/{...}.rs`, move
   tests, repoint validate's evidence renderers (which keep taking the structs).
4. **elp renderers** — move the ~38-symbol family → `posture/elp/{...}.rs`, move
   tests, repoint validate bundle/writers + CLI consumers.
5. **coupling 3** — capture pre-move metadata fixture; rebuild the elp
   `backend.rs` metadata string from structured fields; add the equality test.
   **Must land before task 4 removes `lunar_theory_source_family_summary_for_report`
   from elp** — the rebuild is what drops the last calculation-path use, so
   elp's own `backend.rs` compiles once the helper is gone. In practice: do the
   rebuild + fixture, then move the elp renderer family.
6. **compression** — move the two coverage-summary renderings validate-side;
   verify against public `Artifact` accessors; move tests.
7. **consumer repoint + bundle-pin sweep** — finish validate/CLI repoints;
   confirm every checksum pin value unchanged.
8. **close-out** — `mise run ci` green; grep assertion (no `_for_report` symbol
   exported from any of the five crates); CHANGELOG entry; PLAN.md status
   refresh.

Tasks 1–3 and 6 are independent of one another; task 5 (coupling-3 rebuild)
precedes task 4 (elp renderer move); 7 depends on 1–6; 8 is last.

## Verification

- `mise run ci` — fmt, clippy `-D warnings`, `cargo test --workspace
  --include-ignored`, `cargo doc -D warnings`, workspace-audit, package-check,
  release-smoke, claims-audit.
- `release-smoke` proves bundle checksum values unchanged (invariant 1).
- Coupling-3 pre/post equality fixture (invariant 3).
- Grep assertion: `grep -rn "pub fn .*_for_report" crates/pleiades-{houses,ayanamsa,vsop87,elp,compression}/src`
  returns nothing after the slice.

## Non-goals

- No renaming or redesign of the moved renderers' output (pure relocation).
- No version bump (compatibility `0.7.13`, API-stability `0.3.0` unchanged; the
  workspace `0.4.0` bump is deferred to post-Slice-D).
- No touching Slices C (`pleiades-data`) or D (`pleiades-jpl`) — independent,
  separately branched.
- No decomposition of `pleiades-validate` itself.

## Risks

- **Checksum-pin churn** in validate's bundle-verify tests — expected and
  mechanical; values must not change (invariant 1). Any change is a move defect.
- **Missed `pub` runtime consumer** downgraded to `pub(crate)` — caught by
  `cargo build`/CI across the workspace before merge.
- **Coupling-3 rebuild drift** — mitigated by the pre/post equality fixture.
- **vsop87 breadth** (`batch_parity.rs` alone carries 8 parity families) — the
  largest renderer count in the slice, but each is a verbatim move with tests
  that move with it; byte-identity is mechanically checkable.
