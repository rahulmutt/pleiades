# Devkit Adoption Design

**Date:** 2026-07-17
**Status:** Approved design, pending implementation plan

## Goal

Close the gaps between this repository and the devkit skill set
(`developer-environment`, `testing-practices`, `writing-clean-code`,
`security-practices`, `navigable-codebases`). A gap audit found the repo
already strongly aligned on environment management (mise-pinned tools,
minimal documented devenv.nix, committed lockfile, native cargo builds) and
clean-code policy (AGENTS.md), with gaps in security tooling, dependency
update cadence, test-form coverage, CI speed tiering, and two navigability
details.

## Non-goals

- No change to the release process (release-plz) or `release-gate` semantics.
- No Bazel or meta-build adoption — no trigger holds for a single-language
  cargo workspace.
- No new ARCHITECTURE.md — `spec/architecture.md` already serves as the map.
- No hard mutation-score gate — mutation testing lands report-only.

## Structure: four phases, each independently landable

Phases are sequenced by value and dependency: security first (live exposure),
CI tiering before the test forms that need a nightly tier, navigability last.
Each phase is a small reviewable unit per AGENTS.md change-management rules.

---

## Phase 1 — Security & dependency hygiene

### SCA: cargo-deny

- Pin `cargo:cargo-deny` in `mise.toml` (subsumes cargo-audit; adds
  license/bans/sources checks).
- Commit `deny.toml`:
  - `advisories` — RustSec CVEs, fail-closed;
  - `licenses` — allowlist compatible with `MIT OR Apache-2.0` distribution;
  - `bans` — warn on duplicate major versions, no hard bans initially;
  - `sources` — crates.io only (matches the pure-Rust, no-C-compiler stance).
- Expose as `mise run deny`; wire into blocking CI **and** the nightly tier
  (new CVEs land on old code).

### Secret scanning: gitleaks

- Pin gitleaks via mise; expose as `mise run secrets`.
- One-time full-history scan during implementation; findings handled before
  the CI gate lands.
- Blocking CI job on every push/PR.
- Local pre-commit hook: committed `.githooks/pre-commit` running
  `gitleaks protect --staged`; activated by
  `git config core.hooksPath .githooks` in the onboarding steps (opt-in —
  git cannot force hooks).

### SAST

Clippy with `-D warnings` (already blocking) fills the SAST role for Rust.
No additional SAST tool. Recorded as a deliberate choice in the threat model.

### Dependency updates: Renovate

- Renovate over Dependabot because Renovate natively understands `mise.toml`,
  so pinned toolchain versions get the same cadence-based updates as Cargo
  dependencies.
- Config: grouped minor/patch updates, separate PRs for majors, gated by
  blocking CI.
- **Manual step:** the Renovate GitHub App must be installed on the repo by a
  maintainer.

### Threat model: `docs/threat-model.md` (one-pager)

Names assets, assumed adversaries, explicit out-of-scope (e.g. DoS on a
local computation library), and four trust boundaries:

1. `pleiades-jpl::ingest` + kernel/corpus parsing — untrusted bytes;
   fail-closed checksums already guard this; document it.
2. `horizons-fetch` (default-off feature) — the only network path; rustls
   with pinned webpki trust anchors.
3. CI release pipeline — `RELEASE_PLZ_TOKEN` / `CARGO_REGISTRY_TOKEN` scope
   and blast radius.
4. Outbound supply chain — what a compromised `pleiades-*` release would mean
   for consumers; which control (deny / gitleaks / Renovate) guards what.

Pointed to from AGENTS.md's "Security and reliability" section — referenced,
not duplicated. Revisited when a boundary changes.

---

## Phase 2 — Two-tier CI

### Tiers

- **Blocking** (`mise run ci`, every push/PR): held to an explicit wall-clock
  budget of **≤ 10 minutes** on the GitHub runner.
- **Nightly** (`mise run ci-nightly`, new `.github/workflows/nightly.yml` on
  `schedule:` cron + `workflow_dispatch`): slow, broad, or long-running work.
  Failure opens/updates a pinned GitHub issue — fail-loud, not fail-blocking.

### Provisional assignment (finalized by measurement, not guesswork)

| Task | Tier |
| --- | --- |
| `fmt`, `lint`, `docs` | Blocking |
| `test` (fast suite, via nextest) + doctests | Blocking |
| `secrets`, `deny` | Blocking |
| `claims-audit` (fast drift/structural) | Blocking |
| `audit` (workspace-audit) | Blocking if measured fast, else nightly |
| `test-full --include-ignored` | Nightly |
| `package-check`, `release-smoke`, `benchmark` | Nightly |
| Numeric gates (`gate-houses`, `gate-ayanamsa`, …) | Nightly |
| Fuzz, mutation (Phase 3) | Nightly |

Each task's runtime is measured during implementation; anything fitting
comfortably in budget may stay blocking (the repo's culture leans
fail-closed, so blocking keeps whatever the budget allows).

### Safety rule

Any test or gate moved to nightly must remain reachable from `release-gate`,
so nothing validated today becomes unvalidated at release time. The
implementation plan verifies this property explicitly. `release-gate` itself
is untouched.

---

## Phase 3 — Testing upgrades

### cargo-nextest

- Pin via mise. `mise run test` → `cargo nextest run --workspace`;
  `test-full` → `cargo nextest run --workspace --run-ignored all`.
- nextest does not run doctests: the blocking tier keeps a
  `cargo test --doc --workspace` step alongside.

### Property tests (proptest)

External dev-dependency; bounded case count (~256) so it stays in the
blocking tier; co-located per AGENTS.md test-layout rules. Initial targets —
chosen where an *invariant* oracle beats the existing *derived*
(SE/JPL-parity) oracles:

- `pleiades-types`: angle normalization idempotence + range invariants,
  degree/radian round-trips;
- `pleiades-time`: civil → TT/TDB → civil round-trips within documented
  tolerance; monotonicity of conversions;
- `pleiades-compression`: codec encode/decode round-trip;
- `pleiades-houses`: cusp ordering/wraparound invariants across arbitrary
  geo/time inputs.

### Fuzzing (cargo-fuzz)

- Targets only the true untrusted-byte boundaries:
  `pleiades-jpl::ingest` (Horizons vector-table / API JSON / generic CSV)
  and the `pleiades-compression` artifact decode path.
- Oracle: no panics/UB — parsers must return `Err`, never crash (AGENTS.md's
  "treat ingestion as untrusted input" made executable).
- Requires nightly Rust: pin a **dated** nightly alongside stable in mise;
  the `fuzz` task selects it explicitly.
- Bounded campaign (~10 min/target) in nightly CI; corpora committed under
  `fuzz/corpus/` so coverage accumulates.

### Mutation testing (cargo-mutants)

- Nightly-tier, report-only. Initial scope: `pleiades-types`,
  `pleiades-time`, `pleiades-apparent` (pure-logic crates where mutants are
  meaningful and fast).
- First run establishes a baseline mutation score; surviving mutants become
  a triage list, not a gate.
- Excluded: data/corpus crates, where embedded tables would drown the run.

---

## Phase 4 — Navigability

### CLAUDE.md front door

Minimal `CLAUDE.md` containing `@AGENTS.md`, so Claude Code auto-loads the
existing agent instructions while AGENTS.md stays the single canonical
source.

### README "Current state" restructure

The ~2,500-word prose section becomes a short capability table — one row per
surface (bodies, houses, ayanamsas, eclipses, crossings, rise/set,
fictitious, nod-aps, pheno, occultations) with its gate name and accuracy
class — with rows linking to crate docs and CHANGELOG for detail. Measured
accuracy details live in per-crate rustdoc/CHANGELOG, not restated in README
prose.

**Constraint:** `claims-audit` parses release claims. Before touching README
structure, the implementation plan must map exactly which files/sections
`pleiades-validate`'s claims-audit reads, and either preserve those anchors
or update the audit in the same change.

### Codebase map

No new document. README links explicitly to `spec/architecture.md` as the
map. A parallel ARCHITECTURE.md would violate single-sourcing.

### Onboarding verified by running

README quickstart gains the hooks-path step and Renovate note; the full
clone-to-green sequence (`mise install` → `mise run ci`) is executed in a
clean checkout as the phase's acceptance test.

### AGENTS.md touch-ups

Add pointers to `docs/threat-model.md` and the blocking/nightly tiering
policy; align the "install via mise" tool list with what `mise.toml`
actually contains after Phases 1–3.

---

## Acceptance criteria

1. `mise run ci` (blocking) passes in ≤ 10 minutes on the GitHub runner and
   includes secrets + deny gates.
2. Nightly workflow runs the full suite, gates, fuzz, and mutants on a cron
   schedule and fail-louds via a pinned issue.
3. A gitleaks full-history scan (`gitleaks git`) is clean (or findings
   rotated and documented).
4. `docs/threat-model.md` exists and AGENTS.md points to it.
5. Renovate opens grouped update PRs that CI gates.
6. Proptest suites run green in the blocking tier; fuzz targets build and run
   a bounded campaign in nightly; cargo-mutants produces a baseline report.
7. `CLAUDE.md` loads AGENTS.md; README capability table replaces the prose
   blob with `claims-audit` still green.
8. Clone-to-green onboarding sequence verified by running it.
