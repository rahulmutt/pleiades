# Workspace report-surface relocation — public-API cleanup program

Phase: API-hygiene track (pre-0.4.0 release), parallel to the SP event-engine
series. One program spec covering four implementation slices (A–D), each with
its own plan and branch.

## Summary

Every functional (calculation) crate in the workspace has accreted a
report/validation/compatibility prose layer — `*_summary_for_report`
functions, `validated_*` wrappers, policy-text constants, and
documentation-string `*Summary` structs — whose only real consumers are
`pleiades-validate` and the CLI's report subcommands. Measured across the
workspace, this layer is ~30k+ LOC and dominates the public API of two crates
(~78% of `pleiades-jpl`'s and `pleiades-data`'s public items).

This program relocates the entire prose layer into `pleiades-validate` and
deletes the dead parts, establishing one architectural rule:

> **Functional crates expose structured data; `pleiades-validate` owns all
> report prose.**

Hard removal in a single breaking bump: workspace 0.3.0 → 0.4.0, released once
all four slices land. API stability profile 0.2.2 → 0.3.0 (bumped in Slice A).

## What stays in functional crates (the data boundary)

- **Summary/evidence data structs** with public fields, and the
  `*_summary_details()` / accessor constructors that build them (they read
  crate-private internals: artifact bytes, fit samples, corpus statics).
- **Release-gate data**: `pleiades-data`'s `thresholds.rs` (accuracy ceilings,
  `PACKAGED_BUDGETS`), the accuracy-baseline measurement core
  (`accuracy_baseline_against`, `packaged_artifact_accuracy_baseline`),
  corpus checksum/manifest primitives. These are *enforced* by gates, not
  rendering.
- **Descriptor catalogs** (`AyanamsaDescriptor`, `HouseSystemDescriptor`)
  including their `claim_tier: CompatibilityClaimTier` fields — calc-time
  structs whose claim metadata validate merely reads. The
  `CompatibilityClaimTier` enum stays in `pleiades-types` (shared descriptor
  vocabulary).
- **Provenance attached to calculation output**: `pleiades-apparent`'s
  `ApparentProvenance`/`TopocentricProvenance`/`CorrectionSet`,
  `pleiades-fict`'s backend provenance, `pleiades-compression`'s
  `ArtifactHeader.provenance` + its decode-path `validate()`.
- **Backend contract** (`pleiades-backend`): the trait, request/result types,
  metadata + metadata validation, routing/composite backends, claims,
  capabilities, and the five `validate_*` request/policy contract functions
  used by every backend implementation (`validate_request_against_metadata`,
  `validate_requests_against_metadata`, `validate_request_policy`,
  `validate_zodiac_policy`, `validate_observer_policy`) — split out of
  `policy/current.rs`, which today mixes them with report functions.
- **`FrameTreatmentSummary` (+ its ValidationError)** stays in
  `pleiades-backend`: `pleiades-jpl`/`-vsop87`/`-elp` construct it, and they
  sit below validate (moving it would create a dependency cycle).
- **`pleiades-core`'s spec-anchored posture surface**
  (`spec/api-and-ergonomics.md` requires the façade to expose the release
  compatibility profile): `current_compatibility_profile()` +
  `CompatibilityProfile` (its `*_summary_line()` methods and `Display`
  rendering stay), `current_api_stability_profile()` + `ApiStabilityProfile`,
  `current_release_profile_identifiers()` + `ReleaseProfileIdentifiers`, and
  `validate_custom_definition_labels` (live validation logic used by
  `CompatibilityProfile::validate` and by validate).

## What moves into pleiades-validate

Moved code lands in `pleiades-validate/src/posture/<source-crate>/` modules,
verbatim (byte-identical rendered text), together with its tests.
**Visibility: `pub(crate)` by default**; only symbols `pleiades-cli` genuinely
calls at runtime (the 9 policy helpers in `help.rs`, a few summary-command
renderers) stay `pub`.

- **From `pleiades-backend`**: 9 of the 10 `CURRENT_*_POLICY_SUMMARY_TEXT`
  constants (`CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT` relocates to
  `pleiades-core` instead — see below), the 10 policy-summary structs + their ValidationErrors
  (`RequestPolicySummary`, `DeltaTPolicySummary`, …, all pure
  documentation-string wrappers — verified unused by any calculation path),
  all ~36 report functions in `policy/current.rs`, `policy_tests.rs`, and the
  6 report assertions currently mixed into `request_tests.rs`.
- **From `pleiades-jpl`**: the entire `reference_summary/` subtree
  (~21k LOC + 7.6k test LOC; 106 `*_for_report` renderers — all live, all
  consumed by validate/cli), the `data/selected_asteroid_*` renderer halves,
  the report half of `production_generation.rs`, and the
  `SnapshotManifestSummary`/`InterpolationQualitySample` rendering in
  `backend.rs`. Prerequisite: promote jpl's `pub(crate)` corpus/evidence
  accessors to public structured API (`snapshot_entries`,
  `comparison_snapshot_entries`, `snapshot_instants`, `snapshot_bodies`,
  `comparison_body_list`, `reference_asteroid_evidence_list`,
  `interpolation_quality_sample_list`, `is_comparison_body`, boundary-epoch
  constants).
- **From `pleiades-data`**: all `summary_line`/`validated_summary_line`/
  `*_for_report` string rendering across `coverage/{body,fit,generation,
  profile,regen,target}.rs`, the summary halves of `lookup.rs` and
  `coverage/threshold.rs`, and the renderer wrappers in `thresholds.rs` /
  `accuracy_baseline.rs` (~5.7k LOC). The `Packaged*Summary` structs and
  `*_details()` constructors stay.
- **From `pleiades-elp`**: the validate-facing summary families (~38 symbols:
  lunar-theory catalog/capability/limitations/source-selection/evidence/
  reference-batch-parity).
- **From `pleiades-vsop87`**: the source-documentation summaries; the
  source-docs catalog stays public as structured data.
- **From `pleiades-houses`**: catalog-validation and code-alias summary
  renderers.
- **From `pleiades-ayanamsa`**: provenance/catalog-validation/metadata-coverage
  summary renderers.
- **From `pleiades-compression`**: `ArtifactProfileCoverageSummary` /
  `ArtifactResidualBodyCoverageSummary` rendering (computable from public
  `Artifact` accessors).

## What is deleted outright (dead surface)

- `pleiades-core`: the 13 unvalidated compatibility/release
  `*_summary_for_report` free functions with no external caller, plus the 14
  live `validated_*` wrapper functions (validate migrates to the
  `CompatibilityProfile`/`ReleaseProfileIdentifiers` methods it already uses
  elsewhere); the caveats-composer
  (`compatibility_caveats_summary_for_report`) moves into validate next to its
  sole caller. Internal-only pub items downgraded to `pub(crate)`:
  `CURRENT_COMPATIBILITY_PROFILE_ID`, `CURRENT_API_STABILITY_PROFILE_ID`,
  `current_*_profile_id()` accessors (ids stay reachable via
  `current_*_profile().profile_id`), `HouseCodeAliasInventorySummary` (unless
  a public method returns it), never-named ValidationError enums.
- `pleiades-time/src/policy.rs` and `pleiades-apparent/src/policy.rs`
  (documentation-prose wrappers, zero consumers).
- `pleiades-ayanamsa::ayanamsa_thresholds_summary_for_report`,
  `pleiades-houses::house_thresholds_summary_for_report` (own tests only).
- `pleiades-data`: over-exposed items with no non-test consumer —
  `eros_self_consistency_max_longitude_arcsec`,
  `packaged_mixed_frame_batch_parity_summary_for_report` (demote),
  `packaged_body_coverage_summary_details` (demote),
  `packaged_artifact_fit_channel_outlier_summary_details` (demote).

## Coupling resolutions (the three hard edges)

1. **`ChartSnapshot::Display` (pleiades-core) embeds three global policy
   lines** (time-scale, frame, apparentness prose from backend constants).
   Decision: **strip those three lines** from the chart rendering — a
   deliberate, user-visible output change recorded in the CHANGELOG and the
   api-stability notes. The per-snapshot observer-policy lines stay (they
   describe the snapshot, not global policy). Tests pinning chart Display
   text (validate snapshot-render, cli chart tests) update accordingly.
2. **Core's compatibility rendering consumes
   `pleiades_ayanamsa::validated_provenance_summary_for_report`**
   (`compatibility/report.rs:111`, `profile.rs:197,206`) — core sits below
   validate. Decision: core rebuilds the identical provenance line from
   structured data (`provenance_sample_ayanamsas()` + `descriptor().notes` —
   the same derivation validate already implements), keeping
   `CompatibilityProfile` rendered text byte-identical. The ayanamsa helper
   then moves.
3. **`pleiades-elp`'s `EphemerisBackend::metadata()` embeds
   `lunar_theory_source_family_summary_for_report()`** in
   `BackendMetadata.provenance.data_sources` (`backend.rs:229`) — a
   calculation-path consumer. Decision: elp builds that string from
   structured `source_selection()` fields (byte-identical), decoupling the
   report helper so it can move.

Similarly, the unsupported-modes prose
(`CURRENT_UNSUPPORTED_MODES_SUMMARY_TEXT`, rendered inside
`CompatibilityProfile`) relocates from `pleiades-backend` into
`pleiades-core`'s compatibility module — it is compatibility posture, and this
keeps profile rendering below validate.

## Consumer migrations

- **pleiades-validate**: swaps ~17 aliased core-wrapper imports to
  `CompatibilityProfile`/`ReleaseProfileIdentifiers` method calls; repoints
  every moved renderer to its new `posture/` path; release-bundle checksum
  pins repoint to local functions (values unchanged — text is byte-identical).
- **pleiades-cli**: `help.rs` imports its 9 policy helpers from
  `pleiades_validate`; summary-command tests repoint to the surviving `pub`
  validate renderers; chart-output tests updated for the Display change. No
  Cargo dependency changes (cli already depends on validate).
- **Functional crates' own tests**: report tests move with the code;
  contract tests stay.

## Slices

| Slice | Content | Depends on |
| --- | --- | --- |
| A | pleiades-core façade cleanup + pleiades-backend policy-layer move + `ChartSnapshot::Display` change + ayanamsa→core decoupling (coupling 2) + all "deleted outright" trivia (time/apparent/ayanamsa/houses dead helpers) + api-stability profile 0.2.2 → 0.3.0 | — |
| B | houses, ayanamsa, vsop87, elp (incl. coupling 3), compression renderer moves | A |
| C | pleiades-data rendering move (~5.7k LOC) | A |
| D | pleiades-jpl `reference_summary/` move (~23.8k LOC incl. tests) + accessor promotion | A |

B, C, D are independent of each other (parallelizable); each slice leaves
`mise run ci` green. Each slice gets its own implementation plan
(superpowers:writing-plans) and branch.

## Invariants (enforced across every slice)

1. **Byte-identical release-bundle text.** Every moved renderer produces
   exactly the prose it produced before; fnv1a64 checksum *values* in the
   bundle do not change. The only deliberate output change in the whole
   program is the `ChartSnapshot::Display` policy-line removal (Slice A).
2. **Compatibility profile stable.** Profile id stays 0.7.13 and
   `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM` unchanged — any rendered
   profile-text drift is a defect in the slice, not a checksum to regenerate.
3. **No behavior changes.** All policy summaries were verified to be pure
   documentation strings; no calculation path consults them. Gates
   (`validate-corpus`, claims-audit, numeric gates) keep enforcing the same
   data.
4. **Dependency direction.** Nothing below `pleiades-validate` gains a
   dependency on it; `FrameTreatmentSummary` and `CompatibilityClaimTier`
   stay put for exactly this reason.

## Versioning and posture updates

- Workspace crates 0.3.0 → **0.4.0**, released once after Slice D merges
  (interim main stays green but unreleased).
- API stability profile → **pleiades-api-stability/0.3.0** in Slice A:
  stable-surface prose reworded ("compatibility-profile helpers" → the posture
  *types and their methods*); records the report-surface relocation and the
  chart-Display change; validate's posture pins and bundle goldens
  (`api-stability-summary.txt`) updated.
- CHANGELOG entry per slice; the 0.4.0 entry documents the migration path
  ("report prose now lives in pleiades-validate; functional crates expose
  structured data").

## Verification (per slice)

- `mise run ci` (fmt, clippy `-D warnings`, `cargo test --workspace
  --include-ignored`, `cargo doc -D warnings`, workspace-audit,
  package-check, release-smoke, claims-audit).
- Release-bundle rehearsal (`release-smoke`) proves checksum values
  unchanged.
- Grep assertions per slice: no `_for_report` symbol exported from the
  cleaned crates (Slice A: core exports none; backend exports none but keeps
  `FrameTreatmentSummary` + contract `validate_*`; B/C/D per crate).

## Non-goals

- No renaming/redesign of the moved renderers' output (pure relocation).
- No change to numeric gates, corpora, or thresholds.
- No decomposition of `pleiades-validate` itself (it grows by ~30k LOC; a
  future split of validate into render/gate crates is out of scope).
- `pleiades-apsides`, `-fict`, `-eclipse`, `-events` are already clean —
  untouched.

## Risks

- **Unknown external consumers** of removed/moved 0.3.0 APIs — accepted
  (hard removal; api-stability prose already classed report tooling as
  evolving).
- **Checksum-pin churn** in validate's bundle-verify tests — expected and
  mechanical; values must not change (invariant 1) except where a pin
  encodes a symbol path.
- **Slice D size** (~23.8k LOC moved): mitigated by verbatim moves, the
  byte-identity invariant, and the fact that all 106 renderers are
  exercised by existing validate/cli tests that move with them.
- **Rebuilt strings drifting** (couplings 2 and 3): each rebuilt derivation
  gets an equality test against the pre-move rendering captured as a fixture.
