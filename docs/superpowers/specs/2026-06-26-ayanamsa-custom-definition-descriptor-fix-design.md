# Ayanamsa Custom-Definition Descriptor Accuracy Fix (Phase 6, slice 4)

Date: 2026-06-26 (revised same day after deeper investigation)
Status: design approved, pending implementation plan

> **Revision note.** An earlier draft of this spec proposed *removing* the six
> observational-Babylonian ayanamsa variants as "fabricated dead surface." That
> premise was wrong. Deeper investigation found the six are a **deliberate,
> validated "custom-definition-only" provenance category** — not dead surface —
> already modeled correctly and sanctioned by the spec. The only genuine defect
> is inaccurate descriptor *text*. This spec is rewritten to that reality: a
> surgical text-accuracy fix, no removal, no API change.

## Goal

Correct the inaccurate provenance text shipped in six ayanamsa catalog
descriptors. `BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`,
`BabylonianHouse`, `BabylonianHouseObs`, and `BabylonianSissy` each carry a
description asserting the mode is *"labeled BABYL_…  in Swiss Ephemeris."* Swiss
Ephemeris defines no such sidereal mode (its `SE_SIDM_*` codes stop at 46;
none of these names exist, and no `SE_SIDBIT_*` flag produces them). The text is
simply false. Replace it with accurate text that states what these modes
actually are: project custom-definition-only Babylonian labels with no Swiss
Ephemeris `SE_SIDM` code.

This is a correctness fix to shipped catalog metadata. It does **not** remove
modes, change the public `Ayanamsa` enum, change any catalogued count, or alter
the custom-definition-only classification — all of which are already correct.

## Background: why these six are *not* dead surface

The six modes are an intentional, validated provenance category, introduced by a
prior slice to give honest, non-overclaiming provenance to Babylonian labels
that have no Swiss Ephemeris code:

- **Runtime classification.** `is_custom_definition_only_ayanamsa()`
  (`crates/pleiades-ayanamsa/src/lookup.rs:145`) tests membership in
  `CUSTOM_DEFINITION_ONLY_AYANAMSAS` (`lookup.rs:115`), which is exactly these
  six canonical names.
- **Validated three-way partition.** `AyanamsaProvenanceSummary`
  (`crates/pleiades-ayanamsa/src/model.rs`) partitions every built-in into
  `with_sidereal_metadata` / `custom_definition_only` / `without_sidereal_metadata`,
  asserts the three counts sum to the total (`model.rs:367`, `:409`), and
  **fails closed** unless the custom-definition-only set equals
  `CUSTOM_DEFINITION_ONLY_AYANAMSAS` (`model.rs:448`). These six populate that
  bucket because they carry `epoch: None, offset: None`.
- **Published in the release compatibility profile.**
  `custom_definition_ayanamsa_labels()` (`lookup.rs:131`) publishes the six —
  alongside ad-hoc example labels (True Balarama, Aphoric, Takra) — into
  `crates/pleiades-core/src/compatibility/` as explicit custom-definition
  territory "rather than unresolved release gaps."
- **Spec-sanctioned.** `spec/compatibility-catalog.md` requires the catalog
  model to "accept user-defined ayanamsas … that remain clearly distinguishable
  from project-defined built-ins." The custom-definition-only category is that
  mechanism.

So the *classification* is correct and honest: these modes are **not** claimed
as Swiss-Ephemeris-compatible anywhere that matters — they are surfaced as
custom-definition territory. There is no overclaim in the claim tiers, the
compatibility profile framing, or the numeric gate. The single inaccuracy is the
descriptor sentence that says the label exists "in Swiss Ephemeris."

## Scope

**In scope — correct the false provenance text only:**

- The six descriptor `description` fields in
  `crates/pleiades-ayanamsa/src/catalog.rs`. Each appears **twice** (once in the
  `RELEASE_AYANAMSAS` const, once in the `BUILT_IN_AYANAMSAS` static), so there
  are **12 string literals** to change:
  - `RELEASE_AYANAMSAS`: lines 288, 296, 304, 312, 320, 328.
  - `BUILT_IN_AYANAMSAS`: lines 817, 825, 833, 841, 849, 857.
- The two profile-render assertions that pin the old text:
  `crates/pleiades-core/src/compatibility/tests.rs:1111` (`BABYL_HOUSE`) and
  `:1113` (`BABYL_HOUSE_OBS`).
- A compatibility-profile **patch version bump** `0.7.0` → `0.7.1`, because the
  rendered profile embeds these descriptions and its rendered bytes change.
  Version-string sites: `crates/pleiades-core/src/compatibility/mod.rs:26`,
  `README.md:18`, `crates/pleiades-cli/src/cli/tests/summary_commands.rs:433`,
  `crates/pleiades-validate/src/tests/render_request.rs:333`.

**Out of scope — explicitly unchanged:**

- No `Ayanamsa` enum change; no mode removed; the public API is untouched.
- No catalogued-count change: still 59 catalogued / 48 release-grade / 11
  deferred. `README.md:20` does not change.
- The custom-definition-only classification, its runtime predicate, the
  provenance-summary partition + validation, and `CUSTOM_DEFINITION_ONLY_AYANAMSAS`
  membership are all correct and stay as-is.
- The other deferred modes' descriptors are *not* touched: an audit found the
  false `"labeled … in Swiss Ephemeris"` pattern only on these six. Udayagiri,
  PvrPushyaPaksha, Dhruva, and legacy GalacticEquator use different,
  non-false wording.
- No SE numeric gate, corpus, holdout grid, or precession-model change.

## Design

### 1. Corrected descriptor text

Replace each false sentence with accurate, parallel wording that (a) drops the
Swiss-Ephemeris-label claim, (b) names the project alias as an alias (not an SE
constant), and (c) states the custom-definition-only nature. Template, per mode:

> `Custom-definition-only Babylonian sidereal label (alias <ALIAS>); not a Swiss
> Ephemeris sidereal mode — Swiss Ephemeris defines no SE_SIDM code for it.`

with `<ALIAS>` ∈ {`BABYL_HOUSE`, `BABYL_SISSY`, `BABYL_TRUE_GEOC`,
`BABYL_TRUE_TOPC`, `BABYL_TRUE_OBS`, `BABYL_HOUSE_OBS`}. The same corrected
string is written to both the `RELEASE_AYANAMSAS` and `BUILT_IN_AYANAMSAS`
occurrence of each mode, so the two arrays stay byte-identical per mode (the
completeness test relies on consistent descriptors).

### 2. Test alignment

`compatibility/tests.rs:1111` and `:1113` assert the rendered profile contains
the old `BABYL_HOUSE` / `BABYL_HOUSE_OBS` descriptions. Update both to the
corrected strings. These two are the only assertions in the workspace that pin
this description text (verified by grep: the pattern `"labeled … in Swiss
Ephemeris"` appears only in `catalog.rs` and these two test lines). All other
profile/CLI tests assert canonical names and aliases, which are unchanged.

### 3. Compatibility-profile version bump

Bump `CURRENT_COMPATIBILITY_PROFILE_ID` from
`pleiades-compatibility-profile/0.7.0` to `…/0.7.1` and update the three other
hardcoded sites (README prose + the two CLI/validate render tests). This signals
honestly that the rendered profile differs from the archived `0.7.0`. (There is
no automated version↔content-hash coupling test; the bump is project hygiene for
a versioned artifact, and it folds in the version-bump intent deferred by slice
3 §7.)

### 4. Error handling / behavior

No runtime behavior changes. Descriptions are display metadata only; the
custom-definition partition and its fail-closed validation key on canonical
names, not descriptions. The provenance-summary counts, claim tiers, numeric
gate, and overclaim audit are all untouched.

## Acceptance criteria

- The six descriptors no longer claim a Swiss Ephemeris label; each reads as
  accurate custom-definition-only text, identical across both array occurrences.
- No descriptor anywhere in the catalog asserts `"labeled … in Swiss Ephemeris"`
  (grep returns nothing).
- The compatibility profile renders the corrected text; `compatibility/tests.rs`
  assertions pass against the new strings.
- The compatibility-profile id is `…/0.7.1` consistently across `mod.rs`,
  `README.md`, and the two render tests.
- Catalogued counts are unchanged (59 / 48 / 11); the public `Ayanamsa` enum is
  unchanged; `cargo test` is green across the workspace; `compat-claims-audit`,
  `release-smoke`, and `release-gate` re-run green.
- `PLAN.md` records the descriptor-accuracy fix and the profile bump for slice 4.

## Open questions

None blocking. The corrected wording is fixed by the template in §1; the version
bump is a patch increment.
