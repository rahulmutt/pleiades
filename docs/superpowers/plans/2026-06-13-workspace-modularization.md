# Workspace Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose all 12 `pleiades` crates from a few oversized source files into small, single-responsibility modules — relocating large inline test suites into co-located test files, factoring shared test setup into helpers, isolating generated data, and encoding anti-drift conventions in `AGENTS.md`.

**Architecture:** Behavior-preserving structural refactor only (move code + adjust `mod`/`use`/visibility + extract test helpers; no logic changes). Approach B from the spec: domain module trees (directory modules with `mod.rs` + submodules), not flat siblings. Public module-path reorganization is permitted; prefer stable re-exports where they keep call-sites clean. Worst-first, one crate per phase, each phase lands green and self-contained.

**Tech Stack:** Rust 2021 workspace, `mise` task runner (`cargo fmt`/`clippy`/`test`/`doc`), `pleiades-validate` self-audit gate.

**Spec:** `docs/superpowers/specs/2026-06-13-workspace-modularization-design.md`

---

## How to use this plan

This is a refactor, not new-feature work, so the per-task loop is not red-green-refactor. The "test" for every task is: **the crate already builds and its existing tests already pass, and they still do after the move.** Each task follows the **Standard Refactor Recipe** below, applied to that task's concrete module tree and item→file mapping.

### Standard Refactor Recipe (every extraction task applies these steps)

1. **Confirm green baseline for the crate.** Run `cargo build -p <crate>` and `cargo test -p <crate> 2>&1 | tail -n 5`. Record the `test result:` line(s) — the passing test count is the baseline for this crate. If not green, STOP and report (the refactor must start from green).
2. **Create the target module files** listed in the task (empty or with only `//!` module docs and `use` lines to start).
3. **Move one cohesive item group at a time** from the source file into its target file — cut the exact lines (the `struct`/`enum`/`impl`/`fn`/`const` block and its doc comments), paste into the target file. Do not rewrite bodies. Adjust `use` paths and item visibility (`pub`/`pub(crate)`) so the moved code compiles in its new location.
4. **Wire the module** in the parent (`mod.rs` or `lib.rs`): add `mod <name>;` / `pub mod <name>;` and any `pub use <name>::*;` (or specific re-exports) needed to preserve the crate's public surface and internal call-sites.
5. **Build after each moved group:** `cargo build -p <crate>`. Fix path/visibility errors before moving the next group. Small moves keep errors local and legible.
6. **Move the matching tests.** For each module, relocate its inline `#[cfg(test)] mod tests { ... }` into a sibling test file (see "Test relocation convention"). Wire with `#[cfg(test)] mod tests;`.
7. **Extract shared test setup.** Within the crate's tests, group test functions by setup similarity and lift the common arrange code into helpers in a `#[cfg(test)] mod test_support` (or `tests/support.rs` for integration tests) — builder fns + shared constants. Replace duplicated arrange blocks with helper calls. No assertion or case changes.
8. **Verify the phase gate** (see "Per-phase verification gate"). Test count must be ≥ baseline.
9. **Commit** with a message scoped to the crate.

### Test relocation convention

- A module `foo.rs`'s unit tests go in `foo/tests.rs` with `foo.rs` declaring `#[cfg(test)] mod tests;` — OR, when `foo` is already a directory module (`foo/mod.rs`), in `foo/tests.rs`. Keep white-box unit tests as unit tests (same crate, access to private items). **Do not** convert them to black-box `tests/` integration tests.
- Cross-cutting suites that only exercise the public API and read as integration tests may move to the crate's top-level `tests/` directory.
- Shared test setup → `test_support` module (unit, `#[cfg(test)]`) or `tests/support.rs` (integration).

### Per-phase verification gate (run at the end of every crate phase)

```bash
cargo build -p <crate>
cargo test  -p <crate> 2>&1 | tail -n 8     # test count >= baseline from step 1
cargo clippy -p <crate> --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc -p <crate> --no-deps --all-features
cargo fmt --all
```

A phase is **not done** until all five are green and the test count has not dropped. The final phase additionally runs `mise run release-gate`.

---

## Task 0: Baseline + AGENTS.md conventions

**Files:**
- Modify: `AGENTS.md`

- [ ] **Step 1: Capture and record the whole-workspace baseline**

```bash
cargo build --workspace 2>&1 | tail -n 3
cargo test --workspace 2>&1 | grep 'test result:' | tee /tmp/pleiades-test-baseline.txt
```

Expected: build succeeds; a list of `test result: ok. N passed; 0 failed; ...` lines. Keep `/tmp/pleiades-test-baseline.txt` — per-crate counts here are the floor each phase must not drop below.

- [ ] **Step 2: Add the "Module structure and file size" subsection to AGENTS.md**

Insert immediately after the `### Readability and maintainability` subsection (before `### Testing`) in `AGENTS.md`:

```markdown
### Module structure and file size

- Give each file/module one clear responsibility. Group related concerns into
  module trees (a directory module with `mod.rs` plus focused submodules) rather
  than accumulating unrelated items in one file.
- Treat a file growing large as a signal that it is doing too much. Split it
  before adding more, not after.
- Keep large inline test suites out of the file under test: relocate a module's
  `#[cfg(test)] mod tests` into a co-located test file (`<module>/tests.rs`).
  Keep white-box unit tests as unit tests; do not convert them to black-box
  integration tests just to move them.
- Factor shared test setup (backend/fixture/corpus construction, expected-value
  scaffolding) into helpers (`#[cfg(test)] mod test_support` or
  `tests/support.rs`) rather than copy-pasting arrange blocks across tests.
- Isolate generated or embedded data (series tables, fixtures, packaged
  artifacts) into dedicated `data` modules, kept whole — never reformatted or
  sub-divided, so regeneration tooling continues to round-trip.
```

- [ ] **Step 3: Add the test-helper rule cross-reference under `### Testing`**

In `AGENTS.md` under `### Testing`, append one bullet:

```markdown
- Co-locate a module's tests in its own test file and share setup through test
  helpers; see "Module structure and file size".
```

- [ ] **Step 4: Verify docs/markdown sanity and commit**

```bash
git add AGENTS.md
git commit -m "docs(agents): add modular file-structure and test conventions"
```

---

## Task 1: pleiades-core — split `chart.rs` and `compatibility.rs`

`chart.rs` (6,346 lines) and `compatibility.rs` (4,058 lines) are the entry point and lowest-risk large hand-written modules. `core` already uses the `mod x; pub use x::*;` pattern in `lib.rs`.

**Files:**
- Create: `crates/pleiades-core/src/chart/mod.rs`, `chart/observer.rs`, `chart/request.rs`, `chart/placement.rs`, `chart/snapshot.rs`, `chart/signs.rs`, `chart/houses.rs`, `chart/motion.rs`, `chart/aspects.rs`, `chart/sidereal.rs`, `chart/errors.rs`, `chart/tests.rs`
- Delete: `crates/pleiades-core/src/chart.rs` (content moves into `chart/`)
- Modify: `crates/pleiades-core/src/lib.rs:86` (`mod chart;` → unchanged decl, now resolves to dir)
- Test: tests move from `chart.rs` (`#[cfg(test)]` at line 3192) into `chart/tests.rs` (+ `chart/test_support.rs`)

**Item → file mapping for `chart/` (from current `chart.rs` layout):**

| Target file | Items |
| --- | --- |
| `observer.rs` | `default_chart_bodies`, `render_house_system_label`, `render_observer_location_label`, `ObserverPolicy`, `ObserverSummary` (+`ObserverSummaryValidationError`) |
| `request.rs` | `ChartRequest` and its impls |
| `placement.rs` | `BodyPlacement` (+`BodyPlacementValidationError`) |
| `snapshot.rs` | `ChartSnapshot` (+`ChartSnapshotValidationError`) |
| `signs.rs` | `SignSummary` (+`SignSummaryValidationError`), `dominant_sign_summary` |
| `houses.rs` | `HouseSummary` (+`HouseSummaryValidationError`), `dominant_house_summary`, `house_ordinal` |
| `motion.rs` | `MotionSummary` (+`MotionSummaryValidationError`) |
| `aspects.rs` | `AspectSummary`, `AspectKind`, `AspectDefinition` (+errors), `AspectMatch`, `DEFAULT_ASPECT_DEFINITIONS`, `default_aspect_definitions`, `validate_aspect_definitions`, `angle_separation`, `best_aspect_definition` |
| `sidereal.rs` | `sidereal_longitude` |
| `errors.rs` | `map_observer_location_error`, `map_house_error`, `map_custom_definition_error` |
| `mod.rs` | `//!` docs, `mod`/`pub use` re-exports recreating the prior public surface of `chart` |

- [ ] **Step 1: Confirm green baseline** — Standard Refactor Recipe step 1 with `<crate>=pleiades-core`. Record count.
- [ ] **Step 2: Create `chart/mod.rs`** with the module's existing `//!` doc comment and top-of-file `use` imports; add `mod observer; mod request; ...` lines and `pub use` re-exports mirroring what `lib.rs` previously re-exported from `chart`.
- [ ] **Step 3: Move `observer.rs` items**, then `cargo build -p pleiades-core`. Fix `use`/visibility. (Repeat move+build per file in the table, smallest dependencies first: `errors.rs`, `observer.rs`, `motion.rs`, `signs.rs`, `houses.rs`, `placement.rs`, `aspects.rs`, `sidereal.rs`, `request.rs`, `snapshot.rs`.)
- [ ] **Step 4: Delete the now-empty `chart.rs`** and confirm `mod chart;` in `lib.rs` resolves to `chart/mod.rs`. `cargo build -p pleiades-core`.
- [ ] **Step 5: Relocate tests** — move the `#[cfg(test)] mod tests` block from old `chart.rs` into `chart/tests.rs`; add `#[cfg(test)] mod tests;` to `chart/mod.rs`. `cargo test -p pleiades-core 2>&1 | tail -n 5`.
- [ ] **Step 6: Extract shared chart test setup** into `chart/test_support.rs` (`#[cfg(test)] mod test_support;`): builders for `ChartRequest`/instants/observer locations reused across tests; replace duplicated arrange blocks.
- [ ] **Step 7: Apply the same split to `compatibility.rs`** into `compatibility/mod.rs` + submodules grouped by the profile/summary/verification clusters it contains, with tests in `compatibility/tests.rs`. Build after each moved group.
- [ ] **Step 8: Phase gate** — run the full Per-phase verification gate for `pleiades-core`; confirm test count ≥ baseline.
- [ ] **Step 9: Commit**

```bash
git add crates/pleiades-core
git commit -m "refactor(core): split chart and compatibility into module trees"
```

---

## Task 2: pleiades-types — split type clusters

`lib.rs` (3,976 lines; tests at line 2,567) is a flat catalog of domain newtypes/enums.

**Files:**
- Create: `crates/pleiades-types/src/{angles,time,observer,frames,zodiac,bodies,custom_bodies,house_systems,ayanamsa,coordinates,motion,time_range}.rs`, `tests.rs`, `test_support.rs`
- Modify: `crates/pleiades-types/src/lib.rs` (becomes `//!` docs + `mod`/`pub use` re-exports)

**Item → file mapping:**

| Target file | Items |
| --- | --- |
| `angles.rs` | `Angle`, `Longitude`, `Latitude` |
| `time.rs` | `JulianDay`, `TimeScale`, `SECONDS_PER_DAY`, `TimeScaleConversionError`, `TimeScaleConversion`, `checked_time_scale_offset`, `Instant` |
| `observer.rs` | `ObserverLocation` (+`ObserverLocationValidationError`) |
| `frames.rs` | `CoordinateFrame`, `Apparentness` |
| `zodiac.rs` | `ZodiacMode`, `ZodiacSign` |
| `bodies.rs` | `CelestialBodyClass`, `CelestialBody` |
| `custom_bodies.rs` | `CustomBodyId`, `CustomDefinitionValidationError`, `validate_canonical_text` |
| `house_systems.rs` | `HouseSystem`, `CustomHouseSystem` |
| `ayanamsa.rs` | `Ayanamsa`, `CustomAyanamsa` |
| `coordinates.rs` | `EclipticCoordinates`, `EquatorialCoordinates`, `CoordinateValidationError`, `validate_finite_coordinate_value`, `validate_latitude_range`, `validate_right_ascension_range`, `validate_distance` |
| `motion.rs` | `MotionDirection`, `Motion` (+`MotionValidationError`) |
| `time_range.rs` | `TimeRange` (+`TimeRangeValidationError`), `format_time_range_instant`, `same_scale_and_jd` |

- [ ] **Step 1: Confirm green baseline** (`pleiades-types`). Record count.
- [ ] **Step 2: Convert `lib.rs`** to keep its `//!` crate docs + add `mod`/`pub use` re-exports for the public types (preserve the exact public names; `validate_canonical_text` etc. stay `pub(crate)`).
- [ ] **Step 3: Move each cluster** per the table, building after each (`angles` → `frames`/`zodiac` → `time` → `observer`/`bodies`/`custom_bodies` → `coordinates`/`motion`/`house_systems`/`ayanamsa` → `time_range`). `cargo build -p pleiades-types` between moves.
- [ ] **Step 4: Relocate tests** to `tests.rs` (or split per-module `<file>` if a module owns a large block); add `#[cfg(test)] mod tests;` to `lib.rs`.
- [ ] **Step 5: Extract shared setup** into `test_support.rs` (constructors for valid `Instant`/coordinates/etc. reused across cases).
- [ ] **Step 6: Phase gate** for `pleiades-types`; count ≥ baseline.
- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-types
git commit -m "refactor(types): split domain types into focused modules"
```

---

## Task 3: pleiades-backend — split contract, policy, errors

`lib.rs` (7,291 lines; tests at line 3,522) holds backend identity/capabilities/metadata, request/result types, errors, the large family of `*PolicySummary` types, and release-body-claims.

**Files:**
- Create: `crates/pleiades-backend/src/{identity,capabilities,metadata,request,result,errors,validation,policy/mod.rs,policy/time_scale.rs,policy/delta_t.rs,policy/utc.rs,policy/observer.rs,policy/apparentness.rs,policy/request.rs,policy/frame.rs,policy/native_sidereal.rs,policy/zodiac.rs,policy/pluto_fallback.rs,policy/current.rs,release_body_claims}.rs`, plus `tests.rs`/`test_support.rs`
- Modify: `crates/pleiades-backend/src/lib.rs` (`//!` docs + `mod`/`pub use`)

**Item → file mapping:**

| Target file | Items |
| --- | --- |
| `identity.rs` | `BackendId`, `BackendFamily`, `BackendFamilyPosture`, `AccuracyClass` |
| `capabilities.rs` | `BackendCapabilities` (+`BackendCapabilitiesValidationError`) |
| `metadata.rs` | `BackendProvenance` (+error), `BackendMetadata` (+error) |
| `validation.rs` | `validate_non_blank`, `validate_unique_entries`, `validate_non_empty_unique` |
| `request.rs` | `EphemerisRequest` |
| `result.rs` | `QualityAnnotation`, `EphemerisResult` (+error), `format_optional_*` helpers |
| `errors.rs` | `EphemerisErrorKind`, `EphemerisError`, `format_display_list`, `map_custom_definition_error` |
| `policy/<name>.rs` | one file per `*PolicySummary` type (+ its `*ValidationError`): `time_scale`, `delta_t`, `utc`, `observer`, `apparentness`, `request`, `frame`, `native_sidereal`, `zodiac`, `pluto_fallback` |
| `policy/current.rs` | the `CURRENT_*_SUMMARY_TEXT` consts + `current_*_policy_summary()` const fns + `*_for_report` helpers |
| `release_body_claims.rs` | `release_body_claims_*` fns, `ReleaseBodyClaimsSummary` (+errors), `validate_release_body_claims_posture` |
| `policy/mod.rs` | declares the policy submodules + re-exports |

> Note: `EphemerisRequest` and `EphemerisResult` impl the `EphemerisBackend` trait usage; confirm the trait definition (search `trait EphemerisBackend`) and keep it in `lib.rs` or a `traits.rs` — place the trait wherever the fewest cross-imports result; default to a new `traits.rs`.

- [ ] **Step 1: Confirm green baseline** (`pleiades-backend`). Record count.
- [ ] **Step 2: Locate the `EphemerisBackend` trait** (`grep -n 'trait EphemerisBackend' crates/pleiades-backend/src/lib.rs`) and decide its home (`traits.rs`); note it in `lib.rs` re-exports.
- [ ] **Step 3: Build `lib.rs` skeleton** — `//!` docs + `mod`/`pub use` for the modules above.
- [ ] **Step 4: Move clusters** in dependency order (`identity` → `validation` → `capabilities`/`metadata` → `errors` → `request`/`result` → `policy/*` → `policy/current` → `release_body_claims`), building after each.
- [ ] **Step 5: Relocate tests** into co-located test files per module (large policy/result suites get `<module>/tests.rs` or a `tests` submodule), wired with `#[cfg(test)] mod tests;`.
- [ ] **Step 6: Extract shared setup** into `test_support.rs` (builders for valid `BackendMetadata`/`EphemerisRequest`/`EphemerisResult` reused across the suite).
- [ ] **Step 7: Phase gate** for `pleiades-backend`; count ≥ baseline.
- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-backend
git commit -m "refactor(backend): split contract, policy summaries, and errors into modules"
```

---

## Task 4: pleiades-cli — split into commands

`main.rs` (8,817 lines; tests at line 1,873) mixes arg dispatch, per-command rendering, and parsing helpers.

**Files:**
- Create: `crates/pleiades-cli/src/{cli.rs,help.rs,parse.rs,commands/mod.rs,commands/chart.rs,commands/packaged_artifact.rs,render.rs}`, `tests/` integration files + `tests/support.rs`
- Modify: `crates/pleiades-cli/src/main.rs` (thin entry: `fn main()` + `mod` decls only)

**Item → file mapping:**

| Target file | Items |
| --- | --- |
| `main.rs` | `fn main()` only (+ `mod` declarations) |
| `cli.rs` | `render_cli` (top-level dispatch), `banner`, `ensure_no_extra_args` |
| `help.rs` | `help_text`, `shared_request_policy_help_block` |
| `commands/chart.rs` | `render_chart`, `ChartInstantConversionFlags`, `build_chart_instant` |
| `commands/packaged_artifact.rs` | `PackagedArtifactCommand`, `parse_packaged_artifact_command`, `render_packaged_artifact_regeneration`, `render_packaged_artifact_regeneration_check`, `write_text_file`, `checksum64` |
| `parse.rs` | `parse_rounds`, `parse_release_bundle_output_dir`, `parse_f64`, `parse_seconds`, `parse_signed_seconds`, `parse_body`, `parse_builtin_body`, `parse_custom_body`, `parse_ayanamsa`, `parse_custom_ayanamsa`, `strip_custom_ayanamsa_prefix`, `strip_case_insensitive_prefix`, `parse_house_system` |
| `render.rs` | `render_error` |

- [ ] **Step 1: Confirm green baseline** (`pleiades-cli`). Record count.
- [ ] **Step 2: Create modules** and reduce `main.rs` to `mod` decls + `fn main()`. Items default to `pub(crate)` (binary crate; no external API). `cargo build -p pleiades-cli` after each moved group (`parse` → `help` → `render` → `commands/*` → `cli`).
- [ ] **Step 3: Relocate tests.** CLI tests exercise `render_cli`/command output as text — these read as integration tests. Move them to `crates/pleiades-cli/tests/<command>.rs`. Where a test needs a `pub(crate)` helper, either make it `pub` for test use or keep that test inline in the owning module's `#[cfg(test)] mod tests`. Prefer integration tests for the command-output suites.
- [ ] **Step 4: Extract shared setup** into `crates/pleiades-cli/tests/support.rs` (helpers to invoke `render_cli(&[...])` and assert on output; shared arg fixtures).
- [ ] **Step 5: Phase gate** for `pleiades-cli`; count ≥ baseline.
- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-cli
git commit -m "refactor(cli): split main into command, parse, help, and render modules"
```

---

## Task 5: pleiades-jpl — separate backend logic, fixture parsing, and embedded data

`lib.rs` (37,204 lines; logic to ~7,638, then ~30k tests) + existing `selected_asteroid_2001.rs`, `selected_asteroid_2378498.rs` (data).

**Files:**
- Create: `crates/pleiades-jpl/src/{backend.rs,snapshot.rs,requests.rs,production_generation.rs,reference_summary.rs,fixture.rs,data/mod.rs}`, co-located test files, `test_support.rs`
- Move: `selected_asteroid_2001.rs`, `selected_asteroid_2378498.rs` → `data/` (kept whole)
- Modify: `crates/pleiades-jpl/src/lib.rs` (`//!` docs + `mod`/`pub use`)

**Item → file mapping (logic portion):**

| Target file | Items |
| --- | --- |
| `data/mod.rs` | `mod selected_asteroid_2001; mod selected_asteroid_2378498;` (files moved verbatim) + re-exports; `REFERENCE_EPOCH_JD`, `AU_IN_KM` constants if data-adjacent |
| `snapshot.rs` | `SnapshotEntry` type, `reference_instant`, `reference_bodies`, `reference_epochs`, `reference_snapshot` |
| `requests.rs` | all `reference_snapshot_*_requests` / `*_request_corpus` functions |
| `production_generation.rs` | `production_generation_*` fns/types, `ProductionGenerationBoundary*` summaries (+ their `*ValidationError`s), `independent_holdout_snapshot_checksum`, `fnv1a64`, `extend_unique_snapshot_entries` |
| `reference_summary.rs` | `ReferenceSnapshotSummary` (+error) and related summary types |
| `backend.rs` | the backend struct + reader/interpolator impl (`EphemerisBackend` for the snapshot backend) and frame-rotation logic |
| `fixture.rs` | manifest/row corpus parsing (in-memory text + path-backed file inputs) |

> Confirm exact backend/struct names with `grep -nE '^(pub )?(struct|impl) ' crates/pleiades-jpl/src/lib.rs` before moving; the names above are functional groupings.

- [ ] **Step 1: Confirm green baseline** (`pleiades-jpl`). Record count.
- [ ] **Step 2: Move embedded asteroid data** into `data/` (git mv the two files, add `data/mod.rs` declaring them) and update `mod` paths in `lib.rs`. `cargo build -p pleiades-jpl`.
- [ ] **Step 3: Build `lib.rs` skeleton** (`//!` docs + `mod`/`pub use`).
- [ ] **Step 4: Move logic clusters** per table, building after each (`snapshot` → `data` consts → `requests` → `production_generation` → `reference_summary` → `fixture` → `backend`).
- [ ] **Step 5: Relocate the ~30k lines of tests** into co-located `<module>/tests.rs` files matching the module each suite exercises. This is the bulk of the task — move incrementally, running `cargo test -p pleiades-jpl 2>&1 | tail -n 5` after each block to keep the count steady.
- [ ] **Step 6: Extract shared setup** into `test_support.rs`: builders for snapshot requests, fixture text, and expected `EphemerisResult`s reused across the suite (high duplication expected here).
- [ ] **Step 7: Phase gate** for `pleiades-jpl`; count ≥ baseline.
- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-jpl
git commit -m "refactor(jpl): split backend, requests, fixtures, and data; relocate tests"
```

---

## Task 6: pleiades-data — separate packaged-backend logic from artifact data

`lib.rs` (14,054 lines): packaged-backend logic + embedded artifact bytes/constants.

**Files:**
- Create: `crates/pleiades-data/src/{backend.rs,lookup.rs,coverage.rs,regenerate.rs,data/mod.rs}`, test files, `test_support.rs`
- Modify: `crates/pleiades-data/src/lib.rs` (`//!` docs + `mod`/`pub use`)

- [ ] **Step 1: Confirm green baseline** (`pleiades-data`). Record count.
- [ ] **Step 2: Identify the embedded artifact data** — `grep -nE '^(pub )?(const|static) ' crates/pleiades-data/src/lib.rs` to find the large `const`/`static` byte arrays / fixture tables. Move them verbatim into `data/mod.rs` (kept whole, not reformatted). `cargo build -p pleiades-data`.
- [ ] **Step 3: Split the remaining logic** by responsibility: `backend.rs` (the `packaged_backend()` + `EphemerisBackend` impl, fallback logic, equatorial reconstruction), `lookup.rs` (`packaged_lookup` + epoch policy), `coverage.rs` (`packaged_body_coverage_summary` + coverage types), `regenerate.rs` (maintainer regeneration helper + `packaged-artifact-path` feature-gated loader). Build after each.
- [ ] **Step 4: Relocate tests** into co-located test files; the deterministic binary-fixture tests go beside `backend.rs`/`lookup.rs`.
- [ ] **Step 5: Extract shared setup** into `test_support.rs` (instant/body builders, expected-coordinate scaffolding).
- [ ] **Step 6: Phase gate** for `pleiades-data`; count ≥ baseline. Confirm the `packaged-artifact-path` feature still builds: `cargo build -p pleiades-data --all-features`.
- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-data
git commit -m "refactor(data): separate packaged-backend logic from embedded artifact data"
```

---

## Task 7: pleiades-vsop87 — separate logic from generated tables

`lib.rs` (13,249 lines) + per-planet table files already split (`vsop87b_*.rs`). The generated tables stay whole; only the `lib.rs` logic is split.

**Files:**
- Create: `crates/pleiades-vsop87/src/{backend.rs,series.rs,transforms.rs,elements.rs,source_docs.rs,profiles.rs,tables/mod.rs}`, test files, `test_support.rs`
- Move: existing `vsop87b_*.rs` table files → `tables/` (kept whole)
- Modify: `crates/pleiades-vsop87/src/lib.rs` (`//!` docs + `mod`/`pub use`)

- [ ] **Step 1: Confirm green baseline** (`pleiades-vsop87`). Record count.
- [ ] **Step 2: Group the generated tables** — `git mv crates/pleiades-vsop87/src/vsop87b_*.rs crates/pleiades-vsop87/src/tables/` and add `tables/mod.rs` declaring + re-exporting them. Update `mod` paths in `lib.rs`. Confirm the `regenerate-vsop87b-tables` binary's output path expectations still match (`grep -rn 'vsop87b_' crates/pleiades-vsop87/src/bin/`); update the binary's target paths if it writes table files. `cargo build -p pleiades-vsop87`.
- [ ] **Step 3: Map and split the logic** — `grep -nE '^(pub )?(fn|struct|enum|trait|const) ' crates/pleiades-vsop87/src/lib.rs | head -80`, then split by responsibility: `series.rs` (VSOP87B series evaluation), `elements.rs` (low-precision orbital elements / fallback profiles), `transforms.rs` (heliocentric→geocentric, ecliptic↔equatorial), `backend.rs` (`Vsop87Backend` + `EphemerisBackend` impl + TT/TDB acceptance, UT rejection), `source_docs.rs` (`source_documentation_summary`, `source_specifications`, typed source records), `profiles.rs` (source-backed + fallback body-profile helpers). Build after each.
- [ ] **Step 4: Regenerate the tables to prove round-trip** — `cargo run -p pleiades-vsop87 --bin regenerate-vsop87b-tables` (if it regenerates in place) and confirm `git diff --stat` shows no table-content changes. Report if it does.
- [ ] **Step 5: Relocate tests** into co-located files; the source-documentation doctest in `lib.rs` stays (update its `use` paths if the public path changed).
- [ ] **Step 6: Extract shared setup** into `test_support.rs`.
- [ ] **Step 7: Phase gate** for `pleiades-vsop87`; count ≥ baseline.
- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-vsop87
git commit -m "refactor(vsop87): separate backend logic from generated tables"
```

---

## Task 8: pleiades-elp — separate logic from lunar data

`lib.rs` (8,721 lines) + existing `moonposition.rs`.

**Files:**
- Create: `crates/pleiades-elp/src/{backend.rs,series.rs,data/mod.rs}`, test files, `test_support.rs`
- Modify: `crates/pleiades-elp/src/lib.rs` (`//!` docs + `mod`/`pub use`); consider moving `moonposition.rs` into the new structure if it is data.

- [ ] **Step 1: Confirm green baseline** (`pleiades-elp`). Record count.
- [ ] **Step 2: Inspect** — `grep -nE '^(pub )?(fn|struct|enum|const|static) ' crates/pleiades-elp/src/lib.rs | head -60` and check whether the bulk is series-coefficient data (→ `data/mod.rs`, kept whole) or logic. Classify `moonposition.rs` (logic vs data) and place accordingly.
- [ ] **Step 3: Move the lunar series data** into `data/mod.rs` verbatim; `cargo build -p pleiades-elp`.
- [ ] **Step 4: Split logic** into `series.rs` (Meeus-style series evaluation) and `backend.rs` (`ElpBackend` + `EphemerisBackend` impl). Build after each.
- [ ] **Step 5: Relocate tests** into co-located files.
- [ ] **Step 6: Extract shared setup** into `test_support.rs`.
- [ ] **Step 7: Phase gate** for `pleiades-elp`; count ≥ baseline.
- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-elp
git commit -m "refactor(elp): separate lunar backend logic from series data"
```

---

## Task 9: pleiades-compression — split codec concerns

`lib.rs` (4,146 lines).

**Files:**
- Create: `crates/pleiades-compression/src/{encode.rs,decode.rs,channels.rs,residual.rs,format.rs}` (names refined after inspection), test files, `test_support.rs`
- Modify: `crates/pleiades-compression/src/lib.rs` (`//!` docs + `mod`/`pub use`)

- [ ] **Step 1: Confirm green baseline** (`pleiades-compression`). Record count.
- [ ] **Step 2: Inspect** — `grep -nE '^(pub )?(fn|struct|enum|const) ' crates/pleiades-compression/src/lib.rs | head -60`. Group items by codec responsibility: container/format definitions, encoder, decoder, channel handling, residual-correction. Name files to match the actual clusters found.
- [ ] **Step 3: Split** per the discovered clusters, building after each group.
- [ ] **Step 4: Relocate tests**; round-trip/property tests go beside `encode.rs`/`decode.rs`.
- [ ] **Step 5: Extract shared setup** into `test_support.rs` (sample-channel builders, round-trip helpers).
- [ ] **Step 6: Phase gate** for `pleiades-compression`; count ≥ baseline.
- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-compression
git commit -m "refactor(compression): split codec into focused modules"
```

---

## Task 10: pleiades-ayanamsa — split catalog and logic

`lib.rs` (3,506 lines; the 59-ayanamsa catalog + lookup/logic).

**Files:**
- Create: `crates/pleiades-ayanamsa/src/{catalog.rs,model.rs,lookup.rs}` (refined after inspection), test files, `test_support.rs`
- Modify: `crates/pleiades-ayanamsa/src/lib.rs` (`//!` docs + `mod`/`pub use`)

- [ ] **Step 1: Confirm green baseline** (`pleiades-ayanamsa`). Record count.
- [ ] **Step 2: Inspect** — `grep -nE '^(pub )?(fn|struct|enum|const|static) ' crates/pleiades-ayanamsa/src/lib.rs | head -60`. Separate the catalog data (the 59 catalogued ayanamsas — `data`/`catalog.rs`, kept whole if it is a table) from the lookup/computation logic and types.
- [ ] **Step 3: Split** accordingly; build after each group.
- [ ] **Step 4: Relocate tests**; catalog-coverage tests go beside `catalog.rs`.
- [ ] **Step 5: Extract shared setup** into `test_support.rs`.
- [ ] **Step 6: Phase gate** for `pleiades-ayanamsa`; count ≥ baseline.
- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-ayanamsa
git commit -m "refactor(ayanamsa): split catalog data from lookup logic"
```

---

## Task 11: pleiades-houses — split `lib.rs` and `houses.rs`

`lib.rs` (2,449) + `houses.rs` (2,198): 25-system house catalog + computation.

**Files:**
- Create: `crates/pleiades-houses/src/{catalog.rs,systems/mod.rs,error.rs}` and split `houses.rs` into per-algorithm modules (e.g. `systems/placidus.rs`, `systems/koch.rs`, `systems/equal.rs`, ... grouped by computation family), test files, `test_support.rs`
- Modify: `crates/pleiades-houses/src/lib.rs` (`//!` docs + `mod`/`pub use`)

- [ ] **Step 1: Confirm green baseline** (`pleiades-houses`). Record count.
- [ ] **Step 2: Inspect both files** — `grep -nE '^(pub )?(fn|struct|enum|const) ' crates/pleiades-houses/src/lib.rs crates/pleiades-houses/src/houses.rs | head -80`. Identify the `HouseError` type, the catalog of 25 systems, and the per-system computation functions.
- [ ] **Step 3: Split** — `error.rs` (`HouseError`), `catalog.rs` (system catalog/metadata), `systems/` (computation grouped by family). Build after each group.
- [ ] **Step 4: Relocate tests** into co-located files per module.
- [ ] **Step 5: Extract shared setup** into `test_support.rs` (latitude/obliquity/ascendant fixtures reused across system tests).
- [ ] **Step 6: Phase gate** for `pleiades-houses`; count ≥ baseline.
- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-houses
git commit -m "refactor(houses): split catalog, errors, and per-system computation"
```

---

## Task 12: pleiades-validate — split the validation crate (largest)

`lib.rs` (45,768 lines; logic ~lines 17–8,297 then large interleaved test blocks). Also existing `artifact.rs` (3,272), `chart_benchmark.rs` (322), `house_validation.rs` (797). This is the biggest phase; do it last, after the conventions and patterns are established by the earlier crates.

**Files:**
- Create module trees:
  - `crates/pleiades-validate/src/corpus/` — `mod.rs`, corpus + summary
  - `comparison/` — `mod.rs`, `sample.rs`, `tolerance.rs`, `audit.rs`, `report.rs`, `body_class.rs`
  - `release/` — `mod.rs`, `bundle.rs`, `checklist.rs`, `workspace_audit.rs`, `notes.rs`
  - `compatibility/` — `mod.rs` (profile verification)
  - `render/` — `mod.rs` (CLI dispatch `render_cli` + the `render_*` summary family)
  - `provenance.rs`
  - co-located test files per module + `test_support.rs`
- Fold existing `artifact.rs`, `chart_benchmark.rs`, `house_validation.rs` into the tree (e.g. under `release/` or a top-level `benchmark.rs`) where they belong.
- Modify: `crates/pleiades-validate/src/lib.rs` (`//!` docs + `mod`/`pub use`)

**Item → file mapping (from current `lib.rs` layout):**

| Target file | Items |
| --- | --- |
| `corpus/mod.rs` | `ValidationCorpus`, `CorpusSummary` (+impls/Display), `default_corpus`, `release_grade_corpus`, `benchmark_corpus` |
| `comparison/sample.rs` | `ComparisonSample`, `ComparisonSummary`, `ComparisonAuditSummary`, `BodyComparisonSummary` (+impls) |
| `comparison/tolerance.rs` | `ComparisonTolerance`, `ComparisonToleranceScope`, `ComparisonToleranceEntry`, `ComparisonTolerancePolicySummary`, `ComparisonToleranceScopeCoverageSummary`, `BodyToleranceSummary`, the threshold `const`s (lines 312–332), `validate_comparison_tolerance*`, `comparison_tolerance_policy_coverage`, `write_tolerance_policy*` |
| `comparison/body_class.rs` | `BodyClass`, `BodyClassSummary`, `BodyClassToleranceSummary`, `BodyClassSummaryValidationError`, `body_class` |
| `comparison/report.rs` | `ComparisonReport` (+ its big `Display`), `compare_backends`, `default_reference_backend`, `default_candidate_backend` |
| `comparison/audit.rs` | `RegressionFinding`, `RegressionArchive`, audit-summary text helpers (`comparison_snapshot_batch_parity_summary_text`, etc., lines 283–311) |
| `release/bundle.rs` | `ReleaseBundle` (+ `ReleaseBundleError` + `From` impls + its `Display`) |
| `release/workspace_audit.rs` | `WorkspaceAuditReport`, `WorkspaceAuditViolation`, `WorkspaceAuditSummary`, `workspace_audit_summary`, `workspace_audit_rule_counts`, `render_workspace_audit_summary`, `render_native_dependency_audit_summary`, `render_workspace_audit_summary_text` |
| `release/checklist.rs` | `ReleaseChecklistSummary`, the `RELEASE_CHECKLIST_*` consts + their accessor fns, `release_checklist_summary` |
| `report.rs` (validation) | `ValidationReport`, `BenchmarkReport` (+impls + Display) |
| `compatibility/mod.rs` | `CompatibilityProfileVerificationSummary`, `compatibility_profile_verification_summary`, `verify_compatibility_profile` |
| `render/mod.rs` | `render_cli`, `banner`, and the ~30 `render_*` summary functions (delegating to the modules above) |
| `provenance.rs` | `WorkspaceProvenance` and related |

- [ ] **Step 1: Confirm green baseline** (`pleiades-validate`). Record count — this is the largest test suite, so the floor matters most here.
- [ ] **Step 2: Build the `lib.rs` skeleton** — `//!` docs + `mod`/`pub use` re-exports preserving the crate's public API (the CLI binary `src/main.rs` and `tests/comparison_audit_summary.rs` depend on it; keep their referenced paths working via re-exports).
- [ ] **Step 3: Move clusters in dependency order**, building after each group: `corpus` → `comparison/sample` → `comparison/body_class` → `comparison/tolerance` → `comparison/audit` → `comparison/report` → `report` (validation/benchmark) → `release/*` → `compatibility` → `provenance` → `render` (last; it references everything). Run `cargo build -p pleiades-validate` after each.
- [ ] **Step 4: Fold the existing sibling modules** (`artifact.rs`, `chart_benchmark.rs`, `house_validation.rs`) into the new tree where they belong, updating `mod`/`use` paths. Build.
- [ ] **Step 5: Relocate the interleaved test blocks** (the multiple `#[cfg(test)]` blocks at lines 126, 8298, 8308, 14911, 23733, 28239, …) into co-located `<module>/tests.rs` files matching the module each block exercises. Move incrementally; run `cargo test -p pleiades-validate 2>&1 | tail -n 5` after each to keep the count steady. This is the largest single chunk of work in the whole plan.
- [ ] **Step 6: Extract shared setup** into `test_support.rs`: builders for corpora, comparison samples, tolerance entries, reference/candidate backends, and expected report fragments — the validate suite has the heaviest setup duplication, so this yields the biggest line reduction.
- [ ] **Step 7: Phase gate** for `pleiades-validate`; count ≥ baseline. Also confirm the validate **binary** and self-audit still work: `cargo run -q -p pleiades-validate -- workspace-audit` and `cargo run -q -p pleiades-validate -- release-checklist`.
- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-validate
git commit -m "refactor(validate): split into corpus/comparison/release/render module trees"
```

---

## Task 13: Final workspace verification

**Files:** none (verification only)

- [ ] **Step 1: Full workspace build, lint, test, docs**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace 2>&1 | grep 'test result:'
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
```

Expected: all green; the summed passing test count is ≥ the baseline in `/tmp/pleiades-test-baseline.txt`.

- [ ] **Step 2: Run the release gate**

```bash
mise run release-gate
```

Expected: fmt, lint, test, benchmark, audit, package-check, release-smoke, release-gate all pass. `package-check` confirms each publishable crate still fits the 9 MiB budget after restructuring.

- [ ] **Step 3: Confirm no file regressed into a new mega-file and data is unchanged**

```bash
find crates -name '*.rs' -not -path '*/target/*' | xargs wc -l | sort -rn | head -20
git diff --stat main -- crates/pleiades-vsop87/src/tables crates/pleiades-jpl/src/data
```

Expected: the previous 45k/37k/14k-line files are gone; remaining large files are only the kept-whole generated-data modules; data-table diffs show pure moves (renames), not content changes.

- [ ] **Step 4: Finish the branch** — invoke `superpowers:finishing-a-development-branch` to choose merge/PR.

---

## Self-review notes

- **Spec coverage:** Goals 1–5 map to Tasks 1–12 (decompose), the test-relocation steps in every task (goal 2), the `test_support` extraction steps (goal 3), the `data/` isolation steps in Tasks 5–10 (goal 4), and Task 0 (goal 5, AGENTS.md). Non-goals respected: no logic edits (recipe step 3), no coverage loss (count-≥-baseline gate), data kept whole (Tasks 5/7/8 + Task 13 diff check), no numeric limits/CI in AGENTS.md (Task 0 wording).
- **Ordering:** worst-first per spec is adjusted to *establish patterns on smaller crates first, do the 45k-line `validate` last* — this is intentional: the mechanical recipe and `test_support` conventions are proven on `core`/`types`/`backend` before the riskiest, largest move. `validate` remains a single self-contained phase.
- **Concrete mappings** are provided from the actual current file layouts for `core/chart`, `types`, `backend`, `cli`, `jpl`, and `validate`. For `data`, `vsop87`, `elp`, `compression`, `ayanamsa`, `houses`, the task's first step is a targeted `grep` to confirm the exact item names before moving, because their internal clustering wasn't fully enumerated during planning — the responsibility-based target files are fixed; only the precise item-to-file assignment is confirmed at execution.
