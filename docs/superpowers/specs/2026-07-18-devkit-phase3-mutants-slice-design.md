# Devkit Phase 3 Slice — cargo-mutants baseline

**Date:** 2026-07-18
**Status:** Approved design, pending implementation plan
**Parent:** [`2026-07-17-devkit-adoption-design.md`](./2026-07-17-devkit-adoption-design.md) — Phase 3, third and final slice
**Siblings:** [`2026-07-17-devkit-phase3-proptest-slice-design.md`](./2026-07-17-devkit-phase3-proptest-slice-design.md) (merged), [`2026-07-18-devkit-phase3-fuzz-slice-design.md`](./2026-07-18-devkit-phase3-fuzz-slice-design.md) (merged)

## Goal

Land `cargo-mutants` as **report-only** mutation testing over the three
pure-logic crates named by the parent design, producing a committed baseline
mutation score and a triage list of surviving mutants. This closes Phase 3.

Where the proptest slice added invariant oracles and the fuzz slice added a
no-panic oracle, this slice measures something different in kind: not whether
more tests exist, but whether the tests that exist actually *constrain*
behavior. A surviving mutant is a line of production logic that can be changed
without any test noticing.

Aligns with `devkit:testing-practices` (mutation testing as the check on test
suite strength, distinct from coverage) — and, per that skill and the parent
design, as a diagnostic rather than a gate.

## Posture: report-only, and why that is load-bearing

The parent design specifies mutation testing lands "report-only" with
"surviving mutants [as] a triage list, not a gate". This slice treats that as a
design constraint rather than a starting concession.

Mutation score is a diagnostic, not a contract. Gating on it pressures
contributors toward assertions that kill mutants rather than tests that express
intent — writing `assert_eq!(x, 3.7)` against whatever the code currently
returns, which locks in behavior without validating it. In a repository whose
tests are largely *numeric parity gates against external authorities* (Swiss
Ephemeris, JPL Horizons), that pressure is actively harmful: it would reward
pinning computed values over checking them against a reference.

So the score informs where to invest test effort. It never blocks a merge.

## Non-goals

- **No score gate**, now or as a follow-up in this slice. See above.
- **No new tests written in this branch.** Survivors become tracked debt
  (FU-9), not branch scope. This keeps the slice a small reviewable unit per
  AGENTS.md change-management rules, and keeps test-writing decisions from
  being made under the time pressure of an open branch.
- No change to `release-gate`, the numeric gates, or the blocking CI tier.
- No Phase 4 navigability work.
- No widening of visibility and no test-only hooks in production code.

## Scope

The parent design's three pure-logic crates, unchanged. Mutant counts are
measured, not estimated (`cargo mutants --list`, cargo-mutants 27.1.0):

| Crate | Mutants |
| --- | ---: |
| `pleiades-types` | 311 |
| `pleiades-time` | 323 |
| `pleiades-apparent` | 817 |
| **Total** | **1,451** |

These are the crates where a mutant is unambiguously meaningful: pure logic,
no embedded data tables, fast test suites (< 3 s each per
`docs/superpowers/plans/test-timings.md`).

### Deliberately deferred

`pleiades-compression` (607 mutants) and `pleiades-houses` (1,231) are **not**
in scope. Both were measured and both are now genuinely viable candidates —
the proptest slice gave compression codec round-trip properties and houses
cusp-ordering invariants, and the fuzz slice hardened the compression decode
path, so mutation score against them would be newly informative. They are held
back to keep the first triage list a size a human can actually read and act on.
Recorded here as a decision, not an oversight; a follow-up slice can add them
once FU-9's backlog is worked down.

## Architecture

### Tool and tasks

Pin `cargo:cargo-mutants = "27.1.0"` in `mise.toml`, per
`developer-environment`'s pinning rule. Expose two tasks, mirroring the
`fuzz` / `fuzz-target` pair the fuzz slice established:

- `mise run mutants` — full three-crate scope.
- `mise run mutants-crate <name>` — a single crate, for local iteration.

Neither is reachable from `ci`, `ci-nightly`, or `release-gate`.

Unlike `cargo-fuzz`, cargo-mutants needs no nightly toolchain — it runs on the
pinned stable `1.97.1`, so this slice adds no toolchain surface.

### Two load-bearing flags

Both are recorded here because discovering either during implementation is
expensive, and one of them decides whether the workflow is schedulable at all.

**`--test-workspace=false`.** cargo-mutants runs the *entire workspace* test
suite for every mutant by default. This workspace has `pleiades-validate` and
`pleiades-cli` tests that individually take 300+ seconds — `mise run test`
already excludes `pleiades-validate` (`-E 'not package(pleiades-validate)'`)
for exactly this reason. Without this flag, a single mutant costs minutes and
1,451 of them are unschedulable at any cadence. With it, only the mutated
package's own tests run, which is also the semantically correct scope: a mutant
in `pleiades-types` should be caught by `pleiades-types`' tests.

**`--test-tool nextest`.** Matches the repo's runner, so mutants inherit the
same execution semantics, filtering, and per-test isolation as
`mise run test`. (Note that nextest does not run doctests; doctests are not
part of the mutation oracle. Acceptable — doctests here are documentation
examples, not the primary behavioral suite.)

Supporting configuration: `--baseline auto` (proves the unmutated tree is green
before spending hours testing 1,451 mutants against a broken baseline) and `-j`
matched to the runner's 4 vCPUs.

### Report artifacts

`mutants.out/` is uploaded as a CI artifact and **not** committed: it is large
and fully regenerable from a pinned tool plus a pinned commit, so committing it
would add churn without adding recoverable information. This differs
deliberately from the fuzz slice's committed-corpus policy — a fuzz corpus
represents accumulated search effort that cannot be regenerated, whereas a
mutants report is a deterministic function of the tree.

`.gitignore` gains a `mutants.out/` entry; the implementation plan verifies the
result with `git check-ignore` rather than assuming it.

## CI wiring

A **separate workflow**, `.github/workflows/mutants.yml`, on a **weekly** cron
with `workflow_dispatch`, its own timeout, and its own pinned tracking issue
via the established fail-loud pattern (`JasonEtco/create-an-issue@v2` +
`.github/mutants-failure-issue.md`, mirroring `.github/fuzz-failure-issue.md`
and `.github/nightly-failure-issue.md`). Its cron hour is offset from both
nightly (06:00 UTC) and fuzz (03:00 UTC).

Not added to `ci-nightly`. The existing nightly workflow has
`timeout-minutes: 60` already consumed by `test-full`, `package-check`,
`benchmark`, `deny`, and `secrets`; a multi-hour mutation run would overrun
that envelope and fail nightly on duration, tripping the pinned-issue alarm for
a non-bug. This is the same reasoning that gave fuzzing its own workflow.

Weekly rather than nightly because the signal changes slowly: mutation score
moves when test *strategy* changes, not on every commit. A weekly cadence
matches the rate of real change and keeps CI cost proportionate to it.

### Exit-code handling — where report-only becomes executable

cargo-mutants distinguishes "found gaps" from "could not measure", and the
workflow must too:

| Exit | Meaning | Workflow behavior |
| ---: | --- | --- |
| 0 | Every viable mutant was caught | Pass |
| **2** | **Surviving mutants found** | **Pass** — the expected steady state |
| 1 | Usage error (bad arguments) | Fail loud |
| 3 | Tests timed out | Fail loud |
| 4 | Baseline tests already failing/hanging | Fail loud |

Treating exit 2 as success is precisely what makes this report-only rather than
a gate. Treating 1, 3, and 4 as failures is what stops the job from silently
rotting into a no-op that reports a stale score — or no score — indefinitely.
A misconfigured run and a clean run must not look alike.

### Relationship to the parent design's safety rule

The parent design requires anything moved to nightly to stay reachable from
`release-gate`. That rule governs **existing validation being relocated**;
mutation testing is net-new, gates nothing, and validates nothing that was
previously validated elsewhere. `release-gate` is untouched.

## Deliverables

1. **Baseline note** — `docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md`,
   following the fuzz slice's campaign-results precedent: overall and per-crate
   mutation score, wall-clock, tool version, commit SHA, and a grouped analysis
   of survivors (by crate, module, and mutation kind).
2. **FU-9 in `docs/follow-ups.md`** — the triage backlog, in the established
   follow-up format (what, where, evidence, impact, suggested fix, origin), so
   survivors are tracked engineering debt with an owner rather than a notes
   file nobody revisits.
3. **`AGENTS.md`** — tool list gains cargo-mutants; the tiering policy section
   names the weekly mutation tier alongside blocking / nightly / fuzz.

## Acceptance criteria

1. `mise run mutants` completes locally across all three crates on the pinned
   tool version.
2. A full baseline run completes with a recorded overall score, per-crate
   breakdown, and measured wall-clock.
3. `mutants.yml` runs on cron, passes on exit 0/2, fails loud via a pinned
   issue on exit 1/3/4, and uploads `mutants.out` as an artifact.
4. `mise run ci` wall-clock is unchanged — mutation testing is not in the
   blocking tier.
5. All existing gates and the full test suite stay green (this slice changes no
   production code, so any failure is a configuration defect).
6. Baseline note and FU-9 committed; AGENTS.md updated.

## Risks

- **Runtime is estimated, not measured.** ~1,451 mutants at roughly 10–20 s
  each is on the order of 2 hours at `-j4`. The workflow's `timeout-minutes` is
  set from the first real measured run, not from this estimate; the
  implementation plan measures before fixing the number. If the measured
  runtime substantially exceeds the estimate, `--shard` across consecutive
  weekly runs is the fallback — decided on evidence.

- **Proptest nondeterminism jitters the score.** `pleiades-types` proptests use
  random seeds, so a given mutant can be caught in one run and survive the next,
  making week-over-week scores not exactly comparable. Mitigation: pin
  `PROPTEST_CASES` (and consider a fixed seed) for the mutants run so the
  baseline is reproducible. Resolved during implementation on evidence.

- **Survivors may be dominated by trivia.** If the baseline is mostly mutants
  in `Display` impls, trivial accessors, or error-formatting paths, the score is
  technically accurate but unactionable. Response is `--skip-calls` and
  `#[mutants::skip]` attributes to raise signal — applied against the actual
  first output rather than pre-emptively, since guessing at exclusions before
  seeing the report risks hiding real gaps.

- **A report nobody reads.** The genuine failure mode for report-only tooling:
  it runs weekly, costs CI minutes, and changes no behavior. FU-9 exists
  specifically to give the output a destination in the repo's existing
  engineering-debt process. If the backlog is never worked, that is the signal
  to reconsider the tooling rather than to add a gate.
