# Workspace-wide Modularization — Design

**Date:** 2026-06-13
**Status:** Approved (pending spec review)
**Scope:** All 12 crates in the `pleiades` workspace, plus `AGENTS.md`.

## Problem

The workspace has grown several very large source files that mix multiple
responsibilities, enormous inline `#[cfg(test)]` suites, and embedded generated
data in a single file. This hurts readability, reviewability, and the ability to
reason about (and safely edit) any one concern.

Representative line counts at design time:

| File | Lines | Notes |
| --- | --- | --- |
| `pleiades-validate/src/lib.rs` | 45,768 | ~99% inline tests (first `#[cfg(test)]` at line 126) |
| `pleiades-jpl/src/lib.rs` | 37,204 | ~7.6k logic + embedded asteroid data + ~30k tests |
| `pleiades-data/src/lib.rs` | 14,054 | packaged-backend logic + embedded artifact data |
| `pleiades-vsop87/src/lib.rs` | 13,249 | logic + generated tables (per-planet files already split) |
| `pleiades-cli/src/main.rs` | 8,817 | ~1.9k logic + ~7k tests |
| `pleiades-elp/src/lib.rs` | 8,721 | logic + lunar data |
| `pleiades-backend/src/lib.rs` | 7,291 | contract types + policy + errors + tests |
| `pleiades-core/src/chart.rs` | 6,346 | ~half logic, ~half tests |
| `pleiades-core/src/compatibility.rs` | 4,058 | |
| `pleiades-compression/src/lib.rs` | 4,146 | |
| `pleiades-types/src/lib.rs` | 3,976 | |
| `pleiades-ayanamsa/src/lib.rs` | 3,506 | |
| `pleiades-houses/src/{lib,houses}.rs` | 2,449 + 2,198 | |

## Goals

1. Decompose oversized files into small, single-responsibility modules.
2. Relocate large inline test suites into co-located test files.
3. Factor common test setup into shared helpers to cut duplicated setup lines.
4. Isolate generated/embedded data into dedicated modules, kept whole.
5. Encode anti-drift conventions into `AGENTS.md` so the repo stays modular.

## Non-goals

- No behavioral changes. This is a structural, behavior-preserving refactor.
- No reduction in test coverage; no conversion of white-box unit tests to
  black-box integration tests.
- No reformatting or sub-dividing of generated data tables.
- No numeric file-size limits or CI size-enforcement (principle-based only).

## Decisions (from brainstorming)

- **Tests:** split out of oversized files into co-located test files.
- **Generated data:** separated from logic into dedicated data modules, kept
  whole (not sub-divided).
- **AGENTS.md rule:** principle-based, no numeric thresholds, no CI check.
- **Public API:** reorganization of public module paths is permitted; prefer
  stable re-exports where they keep call-sites clean, use real namespaces where
  they read better.
- **Sequencing:** one design spec; implementation plan broken into per-crate
  phases, ordered worst-first, each verified before the next.
- **Strategy:** Approach B — domain module trees (not flat siblings).

## Approach (B): Domain module trees

### Module conventions (the standard every crate follows)

- **One responsibility per file.** A file holds one cohesive concept — a type
  cluster, an algorithm, a renderer, a parser. A file mixing several splits.
- **Module trees over flat siblings.** Related concerns group under a directory
  module (`comparison/{mod,sample,tolerance,report}.rs`). `mod.rs` declares
  submodules, holds shared glue, and re-exports.
- **Tests co-located, in their own file.** Each module's inline
  `#[cfg(test)] mod tests` moves to a sibling `tests.rs` included via
  `#[cfg(test)] mod tests;`. Cross-module suites that read as integration tests
  move to the crate's `tests/` directory. White-box unit tests remain unit
  tests; they are never converted to black-box.
- **Shared test setup factored into helpers.** Within each crate, group test
  functions by setup similarity (backend construction, fixture loading, corpus
  building, expected-value scaffolding) and extract the common parts into a
  `#[cfg(test)] mod test_support` (unit) or `tests/support.rs` (integration):
  builder functions and shared constants. Individual tests then read as
  arrange-via-helper / act / assert. Extraction must be behavior-preserving:
  same cases, same assertions, no drop in what is tested.
- **Generated/embedded data isolated and kept whole.** Data arrays/fixtures move
  to a dedicated `data.rs` or `data/` module — moved verbatim so regeneration
  tooling still round-trips. Not reformatted, not sub-divided.
- **Public API:** submodules may be `pub`. Prefer re-exports that keep call-sites
  stable; use real namespaces where clearer. Doc-tests and READMEs are updated
  to match any moved paths.

### Per-crate decomposition (worst-first)

Module names below are the planned target seams, derived from the current
structural clustering; exact file names may be refined during implementation as
long as they follow the conventions above.

1. **`pleiades-validate`** (largest; bulk of the effort)
   - `corpus/` — corpus + summary construction.
   - `comparison/` — `sample`, `tolerance` (policy/scope/coverage), `audit`,
     `report`.
   - `release/` — `bundle`, `checklist`, `workspace_audit`, `notes`.
   - `compatibility/` — profile verification.
   - `render/` — CLI dispatch + the ~30 `render_*` summary functions.
   - `provenance.rs` — workspace provenance.
   - Existing `artifact.rs`, `chart_benchmark.rs`, `house_validation.rs`
     folded into the new tree as appropriate.
   - Tests follow each module; shared setup → `test_support`.

2. **`pleiades-jpl`**
   - `backend.rs` — reader/interpolator/backend logic.
   - `fixture/` — manifest/row corpus parsing (in-memory + path-backed).
   - `data/` — embedded asteroid fixtures (fold in existing
     `selected_asteroid_*` files), kept whole.
   - Tests co-located; shared fixture setup → helpers.

3. **`pleiades-cli`**
   - `main.rs` — thin entry point / arg dispatch only.
   - `commands/` — one file per subcommand.
   - `render.rs` — output formatting.
   - Tests per command; shared CLI-invocation setup → helpers.

4. **`pleiades-elp`**
   - logic in `lib.rs` / `series.rs`; lunar series data → `data.rs`
     (alongside existing `moonposition.rs`).

5. **`pleiades-backend`**
   - split trait/contract types, capabilities, policy-validation, and errors
     into focused modules.

6. **`pleiades-core`**
   - `chart.rs` → `chart/{request,placement,aspects,houses,summary}.rs`.
   - `compatibility.rs` → focused submodules.

7. **`pleiades-data`**
   - separate packaged-backend logic from the embedded artifact data
     (`data` module, kept whole).

8. **`pleiades-vsop87`**
   - split remaining `lib.rs` logic from generated-table modules; the existing
     per-planet table files and generated data stay whole.

9. **`pleiades-types`, `pleiades-ayanamsa`, `pleiades-houses`,
   `pleiades-compression`**
   - split type/logic clusters into focused modules and extract inline test
     suites + shared setup.

### Per-phase verification gate

Each crate is its own phase. A phase is not done until its gate is green:

1. `cargo build -p <crate>`
2. `mise run test` (scoped to the crate where practical)
3. `mise run lint` (`clippy ... -D warnings`)
4. `mise run docs` (catches broken/moved doc-test paths)

Constraints enforced per phase:

- Behavior-preserving only: moving code + adjusting `mod`/`use`/visibility, plus
  test-helper extraction. No logic edits.
- Test count must not drop versus the pre-phase baseline.

The final phase runs the full `mise run release-gate`.

### AGENTS.md update (anti-drift)

Add a "Module structure and file size" subsection under *Software Development
Best Practices*, stating (principle-based, no numbers, no CI check):

- One clear responsibility per file/module; group related concerns into module
  trees.
- A file growing large is a signal it is doing too much — split before adding
  more.
- Extract large inline test suites into co-located test files; keep white-box
  unit tests as unit tests.
- Factor shared test setup into helpers rather than copy-pasting arrange blocks.
- Isolate generated/embedded data into dedicated modules, kept whole.

## Risks and mitigations

- **Risk:** path/visibility churn breaks downstream or doc-tests.
  **Mitigation:** `mise run docs` in every phase; prefer stable re-exports.
- **Risk:** test-helper extraction silently drops coverage.
  **Mitigation:** test-count baseline check; behavior-preserving rule; review.
- **Risk:** touching generated data breaks regeneration round-trips.
  **Mitigation:** move data verbatim; never reformat; re-run regeneration
  helpers / table-regeneration binaries where they exist.
- **Risk:** scope is large; partial work leaves the tree inconsistent.
  **Mitigation:** per-crate phases each land green and self-contained.

## Out of scope / future

- Numeric size thresholds or a CI size-linter (explicitly declined).
- Any algorithmic or accuracy improvements to the ephemeris backends.
