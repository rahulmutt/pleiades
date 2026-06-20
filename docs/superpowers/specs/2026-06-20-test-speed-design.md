# Faster test suite with gated heavy checks

Status: approved design (2026-06-20). Implementation pending plan.

## Problem

The default test run is slow. Measured on the development machine with plain
`cargo test`:

- Compilation is not the bottleneck: ~14s when cached.
- Execution dominates. The `pleiades-data` library tests alone run **183 tests
  in 214s** in the default (debug, `opt-level = 0`) profile.
- The `pleiades-validate` suite additionally performs **45 full `bundle-release`
  builds** (each hashes the ~10 MB packaged-artifact fixture), kernel-free
  artifact regeneration (`build_packaged_artifact_from_reference_over`), and
  dense fit / residual / lookup sweeps.

The workspace has 2006 tests total, no `#[ignore]`-based gating beyond the single
existing `PLEIADES_ENFORCE_LATENCY` latency test, and runs through `cargo test`
(which runs crate test binaries serially) rather than `cargo-nextest` (already
listed in `AGENTS.md` tooling and installed via mise).

### Root cause

The heavy tests are numerical: Chebyshev fits, kernel-free artifact
regeneration, and dense sample sweeps, all compiled at `opt-level = 0`. The same
`pleiades-data` library tests run in **76s with `opt-level = 2`** — a **2.8x**
speedup from the build profile alone, for a one-time ~30s extra cold compile and
no behavior change. 76s for one crate is still slow, so the heavy tests also do
genuinely large amounts of work that profile optimization alone does not remove.

## Goals

- Make the default developer test run substantially faster with **no loss of
  coverage** (Phase 1).
- Then move the genuinely heavy checks behind an opt-in gate so the default run
  is a fast sanity pass, while the full set stays guaranteed in CI and
  `release-gate` (Phase 2).
- Keep doctests (~108 rust doctests across 20 files) running.

## Non-goals

- No change to what any assertion checks in Phase 1.
- No new test framework beyond cargo-nextest.
- No production-code restructuring for its own sake.

## Contract

**The Phase 2 gate must never reduce what CI / `release-gate` runs — it only
changes what a bare default `mise test` run executes.** Every test that exists
today still runs under the `full` profile that CI and `release-gate` use.

## Phase 1 — Optimize, zero coverage change

### 1a. Build profile

Add to the root `Cargo.toml`:

```toml
[profile.test]
opt-level = 2

[profile.dev]
opt-level = 2
```

`profile.dev` is optimized too so the non-test dev build of the heavy math
crates is usable. Measured 2.8x on the compute-heavy crate. Cost: one-time ~30s
extra cold compile; incremental rebuilds slightly slower. Fallback if
incremental-edit speed regresses noticeably: optimize dependencies only via
`[profile.dev.package."*"]` and keep workspace crates at `opt-level = 0`. We
start with the full test-profile optimization.

### 1b. Runner: cargo-nextest, doctests separate

cargo-nextest runs all tests across all crate binaries in one global parallel
pool (today `cargo test` runs each crate's binary serially), compounding with
1a. nextest cannot run doctests, so doctests get a dedicated step.

`mise.toml` changes:

- Add `cargo-nextest` to `[tools]` for reproducibility (already installed via
  the mise shim).
- `test` task → `cargo nextest run --workspace` **and** `cargo test --workspace
  --doc`.
- `ci` and `release-gate` depend on the same two steps so doctests stay gated.

### 1c. Test-level performance (behavior-preserving + safe input-shrinking)

- **Memoize expensive fixtures.** The kernel-free synthetic artifact rebuilt
  per-test via `build_packaged_artifact_from_reference_over` is the prime
  suspect: cache it in a `OnceLock` in `test_support` so it is built once per
  test binary instead of once per test. Apply the same to any repeated
  decode / derive of shared inputs.
- **Dedup repeated bundle builds.** Factor the 45 `bundle-release` builds so
  tests whose assertions are independent of the build inputs share a single
  built bundle.
- **Shrink oversized sweeps where provably safe.** Dense fit / residual / lookup
  sweeps that iterate many sample fractions to prove a single property get
  reduced, **keeping at least one full-scale check per property** so the
  property's coverage is retained. Each reduction is an individual judgment
  call, called out explicitly in the diff.

Every 1c change is verified by running the affected tests before and after and
confirming an identical pass/fail set.

## Phase 2 — Gate the heavy checks (opt-in)

Mechanism: **cargo-nextest test groups / profiles**, reusing the repo's existing
`#[ignore = "...reason; opt-in via ENV"]` precedent (already used for
`PLEIADES_ENFORCE_LATENCY`).

- Define a `slow` set in a new `.config/nextest.toml`, identified by a nextest
  filterset over the regen / bundle / dense-sweep test families (and/or a
  `slow_` naming convention applied to those tests).
- **Default nextest profile** excludes `slow` → fast sanity run for everyday
  `mise test`. The default still touches every code path with small inputs: at
  least one cheap representative per property remains in the default set.
- **`full` nextest profile** runs everything including `slow`; used by `ci` and
  `release-gate`.
- A developer can run the full set anytime: `mise test-full` (i.e.
  `cargo nextest run --profile full`). The split is filter-based, never
  deletion.

### What counts as "slow"

The regeneration tests, the full `bundle-release` / `verify-release-bundle`
tests, and the dense numerical sweeps — the families that do full-scale data
work. Everything else stays in the default set.

## Verification strategy

- **No coverage lost (Phase 1):** `cargo nextest run --profile full` plus
  `cargo test --doc` are green before and after, with the same test count and
  the same pass set.
- **Speedup proven:** record default-run wall-clock before (current
  `cargo test`) vs after Phase 1, and default-vs-full after Phase 2.
- **Gate correctness (Phase 2):** the `full` profile's test count is >= the
  default profile's, and every `slow`-tagged test runs under `full`.

## Files touched

- `Cargo.toml` — `[profile.test]` / `[profile.dev]` opt-level.
- `mise.toml` — add `cargo-nextest` tool; rewrite `test`, `ci`, `release-gate`
  to use nextest + a doctest step; add `test-full`.
- `.config/nextest.toml` — new; default and `full` profiles, `slow` filterset.
- `crates/*/src/**/test_support.rs` and heavy test files — fixture caching,
  bundle dedup, safe input shrinking, `slow` tagging.
- `spec/validation-and-testing.md` — document the fast-vs-full split.

## Rollout

Two implementation phases, one spec:

1. Phase 1 lands the profile change, the nextest + doctest task wiring, and the
   behavior-preserving / safe-shrink test-level work. Coverage unchanged.
2. Phase 2 adds the nextest `slow` filterset and the default/`full` profile
   split, with CI and `release-gate` pinned to `full`.
