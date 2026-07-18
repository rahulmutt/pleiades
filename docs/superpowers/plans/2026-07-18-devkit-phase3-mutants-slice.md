# Devkit Phase 3 — cargo-mutants Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land `cargo-mutants` as report-only mutation testing over `pleiades-types`, `pleiades-time`, and `pleiades-apparent`, producing a committed baseline mutation score and a tracked triage backlog.

**Architecture:** Pin the tool via mise; expose two mise tasks; run the baseline locally to measure score and wall-clock; wire a standalone weekly GitHub Actions workflow whose pass/fail semantics distinguish "found gaps" (exit 2, pass) from "could not measure" (exit 1/3/4, fail loud). No production code changes.

**Tech Stack:** Rust 1.97.1 (stable), cargo-mutants 27.1.0, cargo-nextest 0.9.140, mise, GitHub Actions.

**Spec:** [`docs/superpowers/specs/2026-07-18-devkit-phase3-mutants-slice-design.md`](../specs/2026-07-18-devkit-phase3-mutants-slice-design.md)

**Branch:** `devkit-phase3-mutants` (already created; design committed as `ab2fc65d8`).

## Global Constraints

- **Report-only.** Mutation score never gates a merge. Surviving mutants are tracked debt, not failures.
- **No production code changes.** This slice touches only `mise.toml`, `.github/`, `AGENTS.md`, and docs. If a task appears to require editing a `crates/*/src/*.rs` file, stop and escalate.
- **No new tests written.** Survivors go to FU-9. Writing tests to kill mutants is explicitly out of scope.
- **Scope is exactly three crates:** `pleiades-types`, `pleiades-time`, `pleiades-apparent`. Do not add `pleiades-compression` or `pleiades-houses`.
- **`--test-workspace=false` is mandatory** on every cargo-mutants invocation. Without it the whole workspace suite runs per mutant, and `pleiades-validate`/`pleiades-cli` tests take 300+ seconds each.
- **`--test-tool nextest`** on every invocation, matching `mise run test`.
- **Pinned versions only** (devkit `developer-environment` rule): `cargo-mutants` is pinned to `27.1.0`, never floating.
- **`mise run ci` wall-clock must not change.** Nothing here may be reachable from `ci`, `ci-nightly`, or `release-gate`.
- Measured mutant counts (cargo-mutants 27.1.0): `pleiades-types` 311, `pleiades-time` 323, `pleiades-apparent` 817, total **1,451**.

---

### Task 1: Pin cargo-mutants and expose mise tasks

**Files:**
- Modify: `mise.toml` (`[tools]` block ~line 22-38; new `[tasks.mutants]` / `[tasks.mutants-crate]` after `[tasks.fuzz-target]` ~line 118)

**Interfaces:**
- Consumes: nothing (first task).
- Produces: `mise run mutants` (full three-crate run), `mise run mutants-crate <name>` (single crate). Task 2 runs the baseline through `mise run mutants`; Task 4's workflow calls `mise run mutants`.

**Context you need:** `mise.toml` pins tools in `[tools]` and defines tasks as `[tasks.<name>]` with `run = "..."`. Multi-line shell uses `run = """..."""`. Argument templating uses `{{arg(name="x")}}` — see the existing `[tasks.fuzz-target]` for the exact form. `workspace-audit` parses `[tools]` but only validates the `rust =` entry (must stay an inline table with a version matching `[workspace.package] rust-version` and rustfmt+clippy components) — a `cargo:` entry is inert to it, but Step 5 verifies that rather than trusting it.

- [ ] **Step 1: Add the tool pin**

In `mise.toml`, in the `[tools]` block, add after the `"cargo:cargo-fuzz" = "0.12.0"` line:

```toml
# Mutation testing (report-only, weekly tier — see .github/workflows/mutants.yml).
# Source build: no aqua/prebuilt backend for cargo-mutants, compiled once then
# served from the mise cache. Runs on the pinned STABLE toolchain — unlike
# cargo-fuzz it needs no nightly, so it adds no toolchain surface.
"cargo:cargo-mutants" = "27.1.0"
```

- [ ] **Step 2: Add the mutants tasks**

In `mise.toml`, immediately after the `[tasks.fuzz-target]` block, add:

```toml
[tasks.mutants]
description = "Mutation testing baseline over the pure-logic crates (report-only; NOT part of ci or ci-nightly)."
# --test-workspace=false is load-bearing, not an optimization: cargo-mutants runs
# the ENTIRE workspace suite per mutant by default, and pleiades-validate /
# pleiades-cli carry individual tests of 300+ seconds (see
# docs/superpowers/plans/test-timings.md). Without this flag one mutant costs
# minutes and 1451 of them are unschedulable at any cadence. Restricting to the
# mutated package's own tests is also the semantically correct scope.
# Exit code 2 (surviving mutants found) is an EXPECTED result here, not an error
# — this tier is report-only. See .github/workflows/mutants.yml for the mapping.
run = """
cargo mutants \
  --test-tool nextest \
  --test-workspace=false \
  --baseline auto \
  -p pleiades-types \
  -p pleiades-time \
  -p pleiades-apparent
"""

[tasks.mutants-crate]
description = "Mutation testing for one crate: mise run mutants-crate <name>."
run = "cargo mutants --test-tool nextest --test-workspace=false --baseline auto -p {{arg(name=\"crate\")}}"
```

- [ ] **Step 3: Install the pinned tool**

Run: `mise install`
Expected: cargo-mutants 27.1.0 compiles and installs (a source build, ~1 minute). Subsequent runs serve from cache.

- [ ] **Step 4: Verify the tool is wired and mutant counts match the spec**

Run:
```bash
for c in pleiades-types pleiades-time pleiades-apparent; do
  echo -n "$c: "; mise exec -- cargo mutants --list -p $c | wc -l
done
```
Expected exactly:
```
pleiades-types: 311
pleiades-time: 323
pleiades-apparent: 817
```

If a count differs, the crate changed since the design was measured. That is not a failure — record the new count and carry it forward into Task 3's baseline note, noting the drift.

- [ ] **Step 5: Verify workspace-audit still passes**

Run: `mise run audit`
Expected: PASS, no `tool-manifest.*` violations.

This is the check the fuzz slice tripped (`tool-manifest.rust-entry-invalid`) by restructuring the `rust =` entry. This task does not touch `rust =`, so it should pass — but verify, do not assume.

- [ ] **Step 6: Verify the blocking tier is unchanged**

Run: `mise run ci`
Expected: PASS. Confirm no `mutants` task ran as part of it (it is not in `[tasks.ci].depends`).

- [ ] **Step 7: Verify a single crate runs end-to-end**

Run: `mise run mutants-crate pleiades-time`
Expected: cargo-mutants builds a baseline, then tests 323 mutants. Runtime on the order of 30-90 minutes. It prints a summary like `323 mutants tested: N missed, M caught, ...` and exits 0 or 2.

**Both 0 and 2 are success here.** Exit 2 means surviving mutants were found, which is the expected steady state for a first baseline. Exit 1 (usage error), 3 (timeout), or 4 (baseline already failing) are real failures — if you see 3 or 4, stop and escalate rather than adjusting timeouts to force a pass.

- [ ] **Step 8: Verify mutants.out is gitignored**

Run: `git status --short`
Expected: `mutants.out/` does NOT appear as untracked.

`.gitignore:14` already carries `**/mutants.out*/` from the upstream Rust template, so no `.gitignore` edit is needed. Confirm directly:

Run: `git check-ignore -v mutants.out/`
Expected: `.gitignore:14:**/mutants.out*/	mutants.out/`

Note: `git check-ignore mutants.out` (no trailing slash, path absent) reports *not ignored* — a false negative, because the pattern's trailing slash restricts it to directories. Always test with the trailing slash against a directory that exists.

- [ ] **Step 9: Commit**

```bash
git add mise.toml
git commit -m "build(mutants): pin cargo-mutants and expose mise tasks"
```

---

### Task 2: Measure the full baseline

**Files:**
- Create: `/tmp/claude-1000/-workspace/mutants-baseline.log` (scratch, not committed)

**Interfaces:**
- Consumes: `mise run mutants` from Task 1.
- Produces: the measured overall score, per-crate breakdown, wall-clock, and survivor list that Task 3 writes up and Task 4 uses to set `timeout-minutes`.

**Context you need:** This is the long pole of the slice — expect roughly 2 hours at `-j4`, but the whole point of this task is to *measure* rather than assume. Do not skip it and do not estimate the number for Task 4.

- [ ] **Step 1: Run the full baseline with timing**

Run:
```bash
cd /workspace
time mise run mutants 2>&1 | tee /tmp/claude-1000/-workspace/mutants-baseline.log
echo "exit=${PIPESTATUS[0]}" | tee -a /tmp/claude-1000/-workspace/mutants-baseline.log
```

Expected: a run of ~1,451 mutants ending in a summary line of the form
`1451 mutants tested in Xm: N missed, M caught, K unviable`, followed by
`exit=0` or `exit=2`.

`${PIPESTATUS[0]}` captures cargo-mutants' own exit code rather than `tee`'s —
without it the pipeline always reports 0 and the exit code is lost. This is a
multi-hour run; do not plan to re-run it just to recover the code.

Record the `real` time from `time`. This number sets Task 4's `timeout-minutes`.

- [ ] **Step 2: Confirm the exit code is one you expect**

Read the `exit=` line from Step 1.

Expected: `0` (all caught) or `2` (survivors found — the expected first-baseline
result). Record which in Task 3's note.

If you see `1`, `3`, or `4`, the run did not measure anything: the invocation is
misconfigured, mutants timed out, or the workspace suite is already failing.
Stop and fix the cause — do not proceed to Task 3 with a partial report, and do
not raise timeouts to force a pass.

- [ ] **Step 3: Extract the per-crate and per-file breakdown**

Run:
```bash
cat mutants.out/outcomes.json | head -5
wc -l mutants.out/missed.txt mutants.out/caught.txt mutants.out/unviable.txt 2>/dev/null
```
Expected: `mutants.out/` contains `missed.txt`, `caught.txt`, `timeout.txt`, `unviable.txt`, and `outcomes.json`. The `missed.txt` entries are the survivors — these become the triage list.

- [ ] **Step 4: Group survivors for triage**

Run:
```bash
awk -F: '{print $1}' mutants.out/missed.txt | sort | uniq -c | sort -rn
```
Expected: a count of survivors per source file, highest first. This grouping is what Task 3's note and FU-9 are built from.

- [ ] **Step 5: Assess whether survivors are dominated by trivia**

Read the top ~20 lines of `mutants.out/missed.txt`. Judge whether survivors are mostly:
- **Meaningful:** numeric/logic functions where an unnoticed change is a real gap → straightforward triage list.
- **Trivia:** `Display`/`Debug` impls, trivial accessors, error-formatting paths → note this in Task 3 and recommend `--skip-calls` / `#[mutants::skip]` as follow-up. Per the spec, do NOT apply exclusions pre-emptively in this slice; record the recommendation.

- [ ] **Step 6: Check for proptest score jitter**

`pleiades-types` proptests use random seeds, so a mutant can be caught in one run and survive the next. Re-run just that crate and compare:

```bash
mise run mutants-crate pleiades-types 2>&1 | tail -5
```
Expected: a `missed` count within a mutant or two of the first run's `pleiades-types` portion.

If the count moves materially, record it in Task 3 and recommend pinning `PROPTEST_CASES` (and a fixed seed) for the mutants run so week-over-week baselines are comparable. Per the spec this is resolved on evidence — measure before prescribing.

- [ ] **Step 7: No commit**

This task produces measurements only; `mutants.out/` is gitignored and the log is scratch. Nothing to commit. Carry the numbers to Task 3.

---

### Task 3: Write the baseline note and FU-9 triage entry

**Files:**
- Create: `docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md`
- Modify: `docs/follow-ups.md` (append a new `## FU-9` section at end of file)

**Interfaces:**
- Consumes: all measurements from Task 2.
- Produces: the committed baseline artifact referenced by Task 5's AGENTS.md update and by the workflow's failure-issue template in Task 4.

**Context you need:** The sibling precedent is `docs/superpowers/specs/notes/2026-07-18-fuzz-campaign-results.md` — read it first and match its structure and tone. `docs/follow-ups.md` entries follow a fixed format: **What / Where / Evidence / Impact / Suggested fix / Origin**, plus a `**Status:**` and `**Severity:** ... · **Opened:**` footer. Read the existing FU-6/FU-7/FU-8 entries for the exact shape before writing FU-9.

- [ ] **Step 1: Read the precedents**

Run:
```bash
cat docs/superpowers/specs/notes/2026-07-18-fuzz-campaign-results.md
grep -n -A25 "^## FU-8" docs/follow-ups.md
```
Expected: you now know the note structure and the follow-up entry format.

- [ ] **Step 2: Write the baseline note**

Create `docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md` containing, with real measured values substituted (no placeholders left in the committed file):

```markdown
# cargo-mutants Baseline — Devkit Phase 3

**Date:** 2026-07-18
**Tool:** cargo-mutants 27.1.0 (pinned in `mise.toml`)
**Toolchain:** stable 1.97.1
**Commit:** <SHA of the tree the baseline was measured against>
**Design:** [`../2026-07-18-devkit-phase3-mutants-slice-design.md`](../2026-07-18-devkit-phase3-mutants-slice-design.md)

**Invocation:** `mise run mutants`
(`--test-tool nextest --test-workspace=false --baseline auto`, three crates)

## Result

| Crate | Mutants | Caught | Missed | Unviable | Score |
| --- | ---: | ---: | ---: | ---: | ---: |
| `pleiades-types` | 311 | | | | |
| `pleiades-time` | 323 | | | | |
| `pleiades-apparent` | 817 | | | | |
| **Total** | **1451** | | | | |

**Wall-clock:** <measured> (at `-j<N>` on <machine class>)
**Exit code:** <0 or 2>

Mutation score = caught / (caught + missed), excluding unviable mutants.

## Survivors by file

<output of the Task 2 Step 4 grouping, highest first>

## Assessment

<Are survivors meaningful gaps or trivia? Which modules concentrate them?
Anything that looks like a genuine correctness risk vs. cosmetic paths.>

## Reproducibility notes

<proptest jitter finding from Task 2 Step 6: measured delta between two runs
of pleiades-types, and whether PROPTEST_CASES pinning is recommended.>

## Posture

Report-only. This score gates nothing. Survivors are tracked as FU-9 in
`docs/follow-ups.md`. Re-measured weekly by `.github/workflows/mutants.yml`.
```

- [ ] **Step 3: Verify no placeholders survive**

Run: `grep -n "<.*>" docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md`
Expected: no output. Every `<...>` above is a slot you must fill with a measured value before committing.

- [ ] **Step 4: Append FU-9 to follow-ups.md**

Append to the end of `docs/follow-ups.md`:

```markdown
---

## FU-9: cargo-mutants surviving-mutant triage backlog

**Status:** open · Opened 2026-07-18 by the devkit Phase 3 cargo-mutants slice.

**What:** The first mutation-testing baseline over `pleiades-types`,
`pleiades-time`, and `pleiades-apparent` found <N> surviving mutants out of
1,451 — production logic that can be changed without any test noticing.

**Where:** Full breakdown in
`docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md`; survivors
concentrate in <top files from the grouping>.

**Evidence:** `mise run mutants` at <commit>, cargo-mutants 27.1.0,
score <X>%. Reproduce with `mise run mutants`; per-crate with
`mise run mutants-crate <name>`.

**Impact:** No known defect. A surviving mutant is a *coverage* signal, not a
bug — it means the test suite does not constrain that line, so a future
regression there would land silently. Highest concern is any survivor in
release-grade numeric paths, where the repo's parity gates are the intended
safety net.

**Suggested fix:** Work the backlog by writing tests that express intent, NOT
assertions that pin whatever the code currently returns — the latter locks in
behavior without validating it and is the failure mode the report-only posture
exists to avoid. Triage in priority order: numeric/logic survivors first;
`Display`/`Debug`/accessor survivors are low value and may instead be excluded
via `#[mutants::skip]` or `--skip-calls` to raise the signal of future runs.

**Severity:** test-coverage hardening (report-only, non-blocking) ·
**Opened:** 2026-07-18
```

- [ ] **Step 5: Verify no placeholders survive in FU-9**

Run: `grep -n "<.*>" docs/follow-ups.md`
Expected: no output.

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md docs/follow-ups.md
git commit -m "docs(mutants): record baseline mutation score and FU-9 triage backlog"
```

---

### Task 4: Wire the weekly workflow

**Files:**
- Create: `.github/workflows/mutants.yml`
- Create: `.github/mutants-failure-issue.md`

**Interfaces:**
- Consumes: `mise run mutants` (Task 1); the measured wall-clock (Task 2) sets `timeout-minutes`.
- Produces: a weekly scheduled run with a pinned tracking issue; the last CI-facing deliverable.

**Context you need:** Model this on `.github/workflows/fuzz.yml` — read it in full first. The established fail-loud pattern is `JasonEtco/create-an-issue@v2` with `update_existing: true` + `search_existing: open`, paired with a `gh issue close` step on success keyed to a dedicated label. Cron hours in use: nightly 06:00 UTC, fuzz 03:00 UTC — pick a third, offset hour.

**The critical difference from fuzz.yml:** cargo-mutants exit code 2 means "surviving mutants found", which is the *expected* steady state and must NOT fail the job. Exit 1/3/4 must. A bare `run: mise run mutants` would fail the job on exit 2 and page weekly for a non-bug, so the exit code must be handled explicitly.

- [ ] **Step 1: Read the precedent**

Run: `cat .github/workflows/fuzz.yml .github/fuzz-failure-issue.md`
Expected: you now know the caching, mise setup, issue-open, and issue-close step shapes to mirror.

- [ ] **Step 2: Create the workflow**

Create `.github/workflows/mutants.yml`:

```yaml
name: Mutants

# Separate from nightly.yml on purpose: nightly has a 60-minute budget already
# consumed by test-full, package-check, benchmark, deny and secrets, and a full
# mutation run is measured in hours. Weekly rather than nightly because the
# signal changes slowly — mutation score moves when test STRATEGY changes, not
# on every commit.
on:
  schedule:
    - cron: "0 9 * * 1" # 09:00 UTC Mondays — offset from nightly (06:00) and fuzz (03:00)
  workflow_dispatch:

permissions:
  contents: read
  issues: write

env:
  # ubuntu-latest is a 4-vCPU runner; cargo-mutants reads this natively.
  # Pinned rather than left to default so the weekly wall-clock stays
  # comparable run to run.
  CARGO_MUTANTS_JOBS: 4

jobs:
  mutants:
    runs-on: ubuntu-latest
    timeout-minutes: 240 # set from the measured baseline wall-clock, with headroom
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Set up mise
        uses: jdx/mise-action@v2
        with:
          install: false

      - name: Cache Rust toolchain, Cargo, and build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.rustup
            ~/.cargo
            ~/.local/share/mise
            target
          key: ${{ runner.os }}-mutants-${{ hashFiles('mise.toml', 'Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-mutants-

      - name: Install mise-managed tools
        run: mise install

      # Report-only tier: exit 2 means "surviving mutants found", which is the
      # EXPECTED steady state and must not fail the job. Exit 1 (usage error),
      # 3 (tests timed out) and 4 (baseline already failing) mean the run could
      # not measure anything and must fail loud — a misconfigured run and a
      # clean run must not look alike.
      - name: Run mutation baseline
        run: |
          set +e
          mise run mutants
          code=$?
          set -e
          echo "cargo-mutants exit code: $code"
          case "$code" in
            0) echo "All viable mutants caught." ;;
            2) echo "Surviving mutants found — expected for a report-only tier; see the uploaded report." ;;
            1) echo "::error::cargo-mutants usage error (exit 1) — the invocation is misconfigured."; exit 1 ;;
            3) echo "::error::cargo-mutants tests timed out (exit 3) — a mutant may have caused an infinite loop, or the timeout is too low."; exit 1 ;;
            4) echo "::error::cargo-mutants baseline tests are already failing (exit 4) — no mutants were tested."; exit 1 ;;
            *) echo "::error::cargo-mutants returned unexpected exit code $code."; exit 1 ;;
          esac

      - name: Upload mutation report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: mutants-report
          path: mutants.out/
          if-no-files-found: warn

      - name: Open/update tracking issue on failure
        if: failure()
        uses: JasonEtco/create-an-issue@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RUN_URL: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
        with:
          filename: .github/mutants-failure-issue.md
          update_existing: true
          search_existing: open

      - name: Close tracking issue on success
        if: success()
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh issue list --label mutants-failure --state open --json number --jq '.[].number' \
            | while read -r n; do
                [ -n "$n" ] && gh issue close "$n" --comment "Mutation run green again on ${{ github.sha }}."
              done
```

Set `timeout-minutes` from Task 2's measured wall-clock plus roughly 50% headroom (GitHub runners are slower than a dev machine). If the measured run was ~2 hours, 240 is right; adjust to the real number.

Note `if: always()` on the upload step: the report is the deliverable even when the job fails, and on exit 3 (timeouts) the partial report is exactly what diagnoses the problem.

- [ ] **Step 3: Create the failure-issue template**

Create `.github/mutants-failure-issue.md`:

```markdown
---
title: "Mutation testing run failure"
labels: bug, testing, mutants-failure
---

The scheduled cargo-mutants run could not complete a measurement.

Run: {{ env.RUN_URL }}

**This issue does not mean surviving mutants were found.** Surviving mutants
(cargo-mutants exit code 2) are the expected steady state for this report-only
tier and pass the job. This issue fires only on exit 1 (usage error), 3 (tests
timed out), or 4 (baseline tests already failing) — i.e. the run could not
measure anything. Check the job log for the specific exit code.

Reproduce locally:

```bash
mise run mutants
```

Or for a single crate:

```bash
mise run mutants-crate pleiades-types
```

The partial report is attached to the run as the `mutants-report` artifact.

Most likely causes, by exit code:

- **4 (baseline failing):** the workspace test suite is broken on `main`
  independently of mutation testing — fix that first; this run is a symptom.
- **3 (timeout):** a mutant caused an infinite loop, or `--minimum-test-timeout`
  is too low for a slow runner.
- **1 (usage error):** the `mise run mutants` invocation or a cargo-mutants
  version bump changed an interface.

Per `docs/superpowers/specs/2026-07-18-devkit-phase3-mutants-slice-design.md`,
mutation score is report-only and gates nothing; the surviving-mutant backlog
lives in `docs/follow-ups.md` as FU-9.

This issue is updated rather than duplicated on repeat failures, and closed
automatically when a run next completes.
```

- [ ] **Step 4: Validate the workflow YAML parses**

Run: `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/mutants.yml')); print('YAML OK')"`
Expected: `YAML OK`

- [ ] **Step 5: Verify the exit-code logic in isolation**

The `case` statement is the one piece of novel logic in this task, so test it directly rather than waiting a week for a cron run:

```bash
for code in 0 2 1 3 4 7; do
  result=$(bash -c '
    code='"$code"'
    case "$code" in
      0|2) echo pass ;;
      *) echo fail ;;
    esac')
  echo "exit $code -> $result"
done
```
Expected exactly:
```
exit 0 -> pass
exit 2 -> pass
exit 1 -> fail
exit 3 -> fail
exit 4 -> fail
exit 7 -> fail
```

- [ ] **Step 6: Trigger the workflow manually**

Push the branch, then run: `gh workflow run mutants.yml --ref devkit-phase3-mutants`

Then watch: `gh run list --workflow=mutants.yml --limit 1`

Expected: the run completes (hours) and is **green** despite surviving mutants, with a `mutants-report` artifact attached. A red run here means the exit-code mapping is wrong — fix it before merging rather than after.

If `workflow_dispatch` is not available until the workflow is on the default branch, note that and verify on the first post-merge scheduled run instead; record which path was taken.

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/mutants.yml .github/mutants-failure-issue.md
git commit -m "ci(mutants): add weekly report-only mutation workflow with fail-loud issue"
```

---

### Task 5: Update AGENTS.md and close out

**Files:**
- Modify: `AGENTS.md` (tool list; CI tiering policy section)

**Interfaces:**
- Consumes: everything above.
- Produces: the final documentation state; nothing depends on this task.

**Context you need:** `AGENTS.md` carries a tool list and a tiering policy that the fuzz slice already extended. Nothing machine-validates AGENTS.md against `mise.toml` (confirmed: `workspace-audit` parses only the `rust =` entry), so this is prose accuracy, not a gate — which means it is easy to get silently wrong. Read the surrounding sections before editing and match their existing phrasing.

- [ ] **Step 1: Locate the sections to update**

Run:
```bash
grep -n -i "cargo-fuzz\|nightly tier\|blocking tier\|mise install\|tool list" AGENTS.md
```
Expected: line numbers for the tool list and the tiering policy section that the fuzz slice touched.

- [ ] **Step 2: Add cargo-mutants to the tool list**

In `AGENTS.md`, the tool list runs from ~line 40. Insert immediately after the `cargo-fuzz` entry at line 46:

```markdown
- `cargo-mutants`, for report-only mutation testing on the weekly tier (`mise run mutants`); runs on the default stable toolchain, no nightly required
```

The trailing clause is the useful part — it pre-empts the reasonable assumption that mutation testing needs nightly the way `cargo-fuzz` does.

- [ ] **Step 3: Add the weekly tier to the tiering policy**

Replace the paragraph at `AGENTS.md:299` in full:

```markdown
CI has four tiers, each with its own budget: blocking (`mise run ci`, must pass before merge), nightly (`mise run ci-nightly`, slow/broad checks, fail-loud not fail-blocking), fuzz (`.github/workflows/fuzz.yml`, its own daily cron and timeout), and mutants (`.github/workflows/mutants.yml`, weekly cron). Fuzzing and mutation testing are separate tiers, not part of `ci-nightly` — their budgets would overrun nightly's. The mutants tier is **report-only**: surviving mutants (cargo-mutants exit code 2) pass the job, and mutation score gates nothing. It fails loud only when a run could not measure at all (exit 1/3/4). The surviving-mutant backlog is tracked as FU-9 in `docs/follow-ups.md`.
```

Note this changes "three tiers" to "four tiers" — the sentence counts them explicitly, so leaving the number stale would be a silent doc bug.

- [ ] **Step 4: Verify the claim you just wrote is true**

Run: `grep -n "mutants" mise.toml .github/workflows/mutants.yml | head`
Expected: the pinned version and cron hour in AGENTS.md match what is actually in those files. Do not let the doc drift from the config in the same commit that introduces it.

- [ ] **Step 5: Run the full blocking tier one final time**

Run: `mise run ci`
Expected: PASS, with wall-clock comparable to before this branch (acceptance criterion 4).

Run: `mise run audit`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add AGENTS.md
git commit -m "docs(mutants): document cargo-mutants and the weekly report-only tier"
```

- [ ] **Step 7: Verify every acceptance criterion**

Check each against evidence you actually ran, not recollection:

1. `mise run mutants` completes locally across all three crates on the pinned tool → Task 1 Step 7 / Task 2 Step 1.
2. Baseline run recorded with score, per-crate breakdown, measured wall-clock → Task 3 note.
3. `mutants.yml` runs on cron, passes on 0/2, fails loud on 1/3/4, uploads the report → Task 4 Steps 5-6.
4. `mise run ci` wall-clock unchanged → Task 5 Step 5.
5. All existing gates and the full suite stay green → `mise run ci` + `mise run audit`.
6. Baseline note and FU-9 committed; AGENTS.md updated → Tasks 3 and 5.

Any criterion you cannot point to evidence for is not met. Say so plainly rather than asserting completion.

---

## Completion

When all tasks are done, use `superpowers:finishing-a-development-branch` to open the PR. Per the maintainer's standing preference, auto-merge once CI is green and delete the feature branch.
