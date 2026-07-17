# Devkit Phase 2 — Two-Tier CI Design

**Date:** 2026-07-17
**Status:** Approved design, pending implementation plan
**Parent:** [2026-07-17-devkit-adoption-design.md](2026-07-17-devkit-adoption-design.md) (Phase 2)

## Goal

Split the single, slow, everything-blocking CI gate into two tiers:

- a **blocking** tier that runs on every push/PR and is held to a wall-clock
  budget, and
- a **nightly** tier that runs the slow, broad, or long-running work on a
  schedule and fails *loud* (a pinned issue), not *blocking*.

`release-gate` and the release process (release-plz) are untouched. Nothing
validated today may become unvalidated at release time.

## Non-goals

- No change to `release-gate` semantics or the release-plz flow.
- No Bazel or meta-build adoption. Evaluated and rejected during brainstorming
  (see "Build-system evaluation" below).
- No sccache/remote compilation cache in this phase (a proportionate future
  option if *compile* time, distinct from test execution, becomes the pain).
- Phase 3 test *forms* (proptest, fuzz, mutants) are not implemented here — only
  the nextest **runner swap** is pulled forward (rationale below).

## Measurement baseline

Local warm-cache timings of each existing `mise` task (the workspace was already
compiled, so `lint`/`docs`/`package-check` read ~0s but carry real cold-runner
compile cost). Captured 2026-07-17:

| Task | Warm time | Nature |
| --- | --- | --- |
| fmt | 1s | compile-light |
| lint (clippy `--all-targets --all-features`) | compile-bound | shared workspace compile |
| docs | compile-bound | shared workspace compile |
| audit (workspace-audit) | ~0s | fast binary run |
| deny | ~0s | fast |
| secrets (gitleaks full history) | 13s | fast |
| claims-audit (drift/structural) | 23s | fast |
| package-check | 6s | release-facing |
| release-smoke (numeric gates + overclaim audit) | 153s (~2.5m) | numeric validation |
| **test** (`cargo test --workspace`, non-ignored) | **468s (~7.8m)** | **binding constraint** |

**Key finding:** the *fast* (non-ignored) test suite runs ~7.8 min serially —
`cargo test` runs test binaries one at a time. Adding cold-runner compilation,
the blocking tier cannot credibly reach the ≤ 10-min budget without parallel
test execution. This is what forces the nextest decision below.

## Build-system evaluation (Bazel — rejected)

Weighed against devkit `developer-environment`'s four Bazel-adoption triggers:

| Trigger | This repo |
| --- | --- |
| Polyglot monorepo | ✗ single-language pure-Rust cargo workspace (16 crates) |
| Build scale / incrementality | ~ weak; cargo already does incremental compile |
| Remote cache / execution | ✗ needs a remote-cache backend the project doesn't run |
| Hermeticity / reproducibility | ✗ already covered by mise-pinned toolchain + committed `Cargo.lock` + crates.io-only `deny.toml` |

No strong trigger holds — consistent with the adoption spec listing Bazel as an
explicit non-goal. Decisively, Bazel does **not** address the measured
bottleneck: the 468s is *warm test execution* (SE/JPL parity-corpus math), which
Bazel never makes faster; its wins are caching unchanged actions and
parallelism/remote execution, and its incremental benefit evaporates on any PR
touching a foundational crate (e.g. `pleiades-types`) since the whole graph
invalidates and every test re-runs. nextest attacks the real cost — parallel
execution of the test binaries — for a fraction of the ongoing cost of
`BUILD`-file + `rules_rust` upkeep.

## Design

### Structure: two aggregator tasks, two workflows

- **`ci` (blocking)** — the existing `mise` aggregator, re-scoped to fast checks.
  Runs on `push` and `pull_request` (unchanged trigger). The workflow job gains
  `timeout-minutes` set **with headroom** — target ≤ 10 min, hard timeout ~13 min
  — so a genuine regression fails loud while runner variance / cold caches do not.
- **`ci-nightly` (new)** — a new `mise` aggregator run by a new
  `.github/workflows/nightly.yml` on `schedule:` cron (~06:00 UTC daily) +
  `workflow_dispatch`. Holds the slow/broad/long-running work.

### Tier assignment

**Blocking `ci`:**

| Task | Rationale |
| --- | --- |
| fmt, lint, docs | fast / compile-bound; shared workspace compile |
| audit | measured fast (~0s); resolves the spec's "blocking if fast" toward blocking |
| deny, secrets | fast supply-chain + secret gates |
| claims-audit | 23s fast drift/structural |
| `test` → `cargo nextest run --workspace` | ~7.8m serial → ~2–3m parallel |
| doctests → `cargo test --doc --workspace` | nextest does not run doctests; kept as a separate blocking step |
| release-smoke | numeric gates + overclaim audit; kept blocking (see below) |

**Nightly `ci-nightly`:**

| Task | Rationale |
| --- | --- |
| test-full → `cargo nextest run --workspace --run-ignored all` | the ignored / slow suite |
| package-check | release-facing packaging + size budget |
| benchmark | perf tracking |
| *(Phase 3)* fuzz, mutants | land here later |

### nextest runner swap (pulled forward from Phase 3)

- Pin `cargo:cargo-nextest` in `mise.toml`.
- `test` → `cargo nextest run --workspace`;
  `test-full` → `cargo nextest run --workspace --run-ignored all`.
- Because nextest does not execute doctests, doctest coverage is preserved in
  two places so nothing that runs today stops running:
  - the **blocking tier** keeps an explicit `doctest` task
    (`cargo test --doc --workspace`);
  - **`test-full`** (which `release-gate` depends on) gains a `depends` on that
    same `doctest` task, so `release-gate` keeps validating doctests **without
    any edit to `release-gate`'s own `depends`**.
- Only the runner swap comes forward; proptest / fuzz / mutants remain Phase 3.

### Numeric validation stays blocking on PRs

`release-smoke` (~2.5 min) runs the SE-parity numeric gates (houses, ayanamsa,
apparent, topocentric, corpus) plus the overclaim audit, and today runs on every
PR. It stays in the **blocking** tier: this repo's culture is fail-closed, and
~2.5 min is affordable once nextest frees ~5 min from the test step. This is a
deliberate, documented deviation from the adoption spec's provisional table
(which placed numeric gates in nightly); the spec explicitly called that table
provisional and "finalized by measurement".

**Fallback:** if the first real cold-runner CI run shows the blocking tier
exceeds the ~13-min timeout with release-smoke included, move release-smoke to
nightly (the safety rule below already guarantees it stays reachable from
`release-gate`). The implementation plan records this contingency.

### Nightly fail-loud mechanism

On a nightly failure, a pinned marketplace action (`JasonEtco/create-an-issue`
or equivalent) opens/updates **one** tracking issue (e.g. titled "Nightly CI
failing") and a subsequent green run closes it. Fail-loud, never fail-blocking —
nightly failures never gate merges.

### Safety rule (hard invariant)

Every task assigned to nightly must remain reachable from `release-gate`, so
nothing validated today becomes unvalidated at release time:

- `test-full`, `benchmark`, `package-check` are all already in `release-gate`'s
  `depends` list. ✓
- Numeric validation stays blocking *and* is in `release-gate`. ✓

The implementation plan asserts this explicitly (compare the nightly task set
against `release-gate`'s transitive `depends`) so a future edit cannot silently
strand a check.

## Documented deviations from the adoption spec

The adoption spec's Phase 2 task table was provisional. Measurement produced
three deliberate refinements:

1. **nextest pulled forward** from Phase 3 — the ≤ 10-min budget is unreachable
   without parallel test execution.
2. **`audit` stays blocking** — measured fast, not deferred to nightly.
3. **release-smoke / numeric gates stay blocking** — fail-closed culture;
   affordable post-nextest; documented nightly fallback if the budget is blown.

## Acceptance criteria

1. Blocking `ci` passes under the workflow `timeout-minutes` (target ≤ 10 min,
   timeout ~13 min) and includes fmt, lint, docs, audit, deny, secrets,
   claims-audit, `nextest` test, doctests, and release-smoke.
2. `cargo nextest` is pinned in `mise.toml` and drives `test` / `test-full`;
   a `doctest` (`cargo test --doc --workspace`) task runs in the blocking tier
   and is a `depends` of `test-full`, so `release-gate` still validates doctests
   with its own `depends` unchanged.
3. `.github/workflows/nightly.yml` runs `ci-nightly` (test-full, package-check,
   benchmark) on `schedule:` + `workflow_dispatch` and fail-louds via a single
   pinned tracking issue that auto-closes on green.
4. Safety-rule assertion passes: every nightly task is reachable from
   `release-gate`.
5. `release-gate`, `release.toml`, `release-plz.toml`, `cliff.toml`, and all
   crate code are unchanged by this phase.
6. The blocking/nightly tiering policy is defined by this design and the two
   `mise` aggregator tasks. Wiring the AGENTS.md pointer to this policy is
   deferred to Phase 4 (navigability) per the adoption spec — not a Phase 2
   acceptance item.
