# Devkit Phase 2 — Two-Tier CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the single slow CI gate into a fast blocking tier (push/PR, budgeted) and a fail-loud nightly tier, pulling the nextest runner swap forward so the blocking tier can hit its wall-clock budget.

**Architecture:** Two `mise` aggregator tasks — `ci` (blocking) and `ci-nightly` — plus a new `.github/workflows/nightly.yml`. The existing `ci.yml` gains a job timeout with headroom. `cargo nextest` replaces `cargo test` as the runner; doctests are preserved as an explicit task so nothing validated today (including at `release-gate`) becomes unvalidated. `release-gate`'s own `depends` list is left byte-for-byte unchanged.

**Tech Stack:** mise task runner, GitHub Actions, `cargo-nextest`, `JasonEtco/create-an-issue` marketplace action, `gh` CLI.

## Global Constraints

- Toolchain is mise-pinned to exact versions; **never** use a fuzzy version. New pin: `cargo:cargo-nextest = "0.9.140"`.
- `release-gate`, `release.toml`, `release-plz.toml`, `cliff.toml`, and all crate code under `crates/` MUST remain unchanged by this phase.
- **Safety rule:** every task assigned to the nightly tier MUST remain reachable from `release-gate` (directly or via `depends`). Verified explicitly in Task 6.
- Blocking `ci` target wall-clock ≤ 10 min; workflow hard timeout set with headroom (13 min).
- Nightly failures are **fail-loud, never fail-blocking** — they open/update a pinned issue and never gate merges.
- A commit closes each task; the repo's `.githooks/pre-commit` runs `gitleaks protect --staged` (already active).

## File Structure

- `mise.toml` — modify `[tools]` (add nextest pin), `[tasks.test]`, `[tasks.test-full]`; add `[tasks.doctest]`, `[tasks.ci-nightly]`; re-scope `[tasks.ci].depends`. `[tasks.release-gate]` untouched.
- `.github/workflows/ci.yml` — add `timeout-minutes` to the blocking job.
- `.github/workflows/nightly.yml` — **create**: scheduled nightly workflow.
- `.github/nightly-failure-issue.md` — **create**: issue template for the pinned tracking issue.

---

### Task 1: nextest runner swap (pin + tasks), doctests preserved

**Files:**
- Modify: `mise.toml` — `[tools]`, `[tasks.test]`, `[tasks.test-full]`; add `[tasks.doctest]`.

**Interfaces:**
- Produces: `mise run test` → fast parallel suite (nextest, non-ignored); `mise run doctest` → doctests; `mise run test-full` → all tests incl. ignored **and** doctests. Later tasks (`ci`, `ci-nightly`) consume these task names.

- [ ] **Step 1: Confirm the current (pre-swap) behavior**

Run: `mise run test 2>&1 | tail -5`
Expected: output from `cargo test` (the serial runner) — this is the baseline being replaced.

- [ ] **Step 2: Pin cargo-nextest in `mise.toml` `[tools]`**

Add this line to the `[tools]` table (after the `gitleaks` line):

```toml
"cargo:cargo-nextest" = "0.9.140"
```

- [ ] **Step 3: Install and verify the pinned tool**

Run: `mise install && cargo nextest --version`
Expected: `cargo-nextest 0.9.140` printed. If the version differs, the pin did not take — stop and fix.

- [ ] **Step 4: Swap the `test` and `test-full` runners and add `doctest`**

Replace the existing `[tasks.test]` and `[tasks.test-full]` blocks:

```toml
[tasks.test]
run = "cargo nextest run --workspace"

[tasks.doctest]
run = "cargo test --doc --workspace"

[tasks.test-full]
depends = ["doctest"]
run = "cargo nextest run --workspace --run-ignored all"
```

(Order: keep `[tasks.doctest]` between `test` and `test-full` for readability. `test-full`'s `depends = ["doctest"]` is what preserves doctest coverage for `release-gate`, whose own `depends` on `test-full` is unchanged.)

- [ ] **Step 5: Verify the fast suite runs green under nextest**

Run: `mise run test`
Expected: nextest summary (`Summary [ ... ] N tests run: N passed`), exit 0. nextest runs each test in its own process — if any test that passed under `cargo test` now fails, it depended on shared in-process state; fix the test, do not revert the runner.

- [ ] **Step 6: Verify doctests run green**

Run: `mise run doctest`
Expected: `cargo test --doc` output, `test result: ok`, exit 0.

- [ ] **Step 7: Verify the full suite runs ignored tests + doctests**

Run: `mise run test-full`
Expected: the `doctest` dependency runs first (`Doc-tests ...`), then nextest runs with ignored tests included (the run count is ≥ the `mise run test` count). Exit 0.

- [ ] **Step 8: Commit**

```bash
git add mise.toml
git commit -m "test(ci): swap cargo test for nextest, preserve doctests via test-full"
```

---

### Task 2: Re-scope the blocking `ci` aggregator

**Files:**
- Modify: `mise.toml` — `[tasks.ci].depends`.

**Interfaces:**
- Consumes: `test`, `doctest`, `test-full` (Task 1).
- Produces: a blocking `ci` task that runs only fast checks + numeric validation, dropping the heavy `test-full` / `package-check` from the blocking path.

- [ ] **Step 1: Show the current blocking set**

Run: `grep -A1 '^\[tasks.ci\]' mise.toml`
Expected: `depends = ["fmt", "lint", "test-full", "docs", "audit", "package-check", "release-smoke", "claims-audit", "deny", "secrets"]`

- [ ] **Step 2: Replace `[tasks.ci].depends` with the re-scoped blocking set**

Replace the `depends` line of the `[tasks.ci]` block with:

```toml
depends = ["fmt", "lint", "docs", "audit", "deny", "secrets", "claims-audit", "test", "doctest", "release-smoke"]
```

Changes vs. before: `test-full` → `test` + `doctest` (fast suite + doctests, not the ignored suite); `package-check` removed (moves to nightly, Task 3). `release-smoke` stays (numeric validation kept blocking per the design).

- [ ] **Step 3: Verify the blocking gate passes end-to-end**

Run: `mise run ci`
Expected: every listed task runs and the aggregate exits 0. Confirm the task list shown includes `test` (nextest), `doctest`, `release-smoke`, `deny`, and `secrets`, and does **not** include `test-full` or `package-check`.

- [ ] **Step 4: Commit**

```bash
git add mise.toml
git commit -m "ci(tiering): re-scope blocking ci to fast checks + numeric validation"
```

---

### Task 3: Add the `ci-nightly` aggregator

**Files:**
- Modify: `mise.toml` — add `[tasks.ci-nightly]`.

**Interfaces:**
- Consumes: `test-full`, `package-check`, `benchmark` (existing tasks).
- Produces: `mise run ci-nightly` — the slow/broad tier the nightly workflow (Task 5) invokes.

- [ ] **Step 1: Add the `[tasks.ci-nightly]` block**

Add after the `[tasks.ci]` block:

```toml
[tasks.ci-nightly]
description = "Nightly tier: slow/broad checks (fail-loud, not fail-blocking)."
depends = ["test-full", "package-check", "benchmark"]
```

- [ ] **Step 2: Verify the nightly aggregate runs green**

Run: `mise run ci-nightly`
Expected: `test-full` (with its `doctest` dependency), `package-check`, and `benchmark` all run; aggregate exits 0.

- [ ] **Step 3: Commit**

```bash
git add mise.toml
git commit -m "ci(tiering): add ci-nightly aggregator (test-full, package-check, benchmark)"
```

---

### Task 4: Add the blocking-CI job timeout (budget with headroom)

**Files:**
- Modify: `.github/workflows/ci.yml` — add `timeout-minutes` to the `test` job.

**Interfaces:**
- Produces: the blocking workflow now self-polices its wall-clock budget.

- [ ] **Step 1: Add `timeout-minutes` to the job**

In `.github/workflows/ci.yml`, under `jobs.test:` (same indent level as `runs-on:`), add:

```yaml
    timeout-minutes: 13 # blocking-CI budget: target <=10 min, headroom for cold caches / runner variance
```

Resulting job header:

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 13 # blocking-CI budget: target <=10 min, headroom for cold caches / runner variance
    steps:
```

- [ ] **Step 2: Verify the YAML is valid**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml')); print('ci.yml OK')"`
Expected: `ci.yml OK`. If `python3`/`pyyaml` is unavailable, run `git diff .github/workflows/ci.yml` and eyeball the indentation instead.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci(tiering): cap blocking CI at 13-min timeout (10-min target + headroom)"
```

---

### Task 5: Nightly workflow with fail-loud pinned issue

**Files:**
- Create: `.github/workflows/nightly.yml`
- Create: `.github/nightly-failure-issue.md`

**Interfaces:**
- Consumes: `mise run ci-nightly` (Task 3).
- Produces: a scheduled workflow that runs the nightly tier and, on failure, opens/updates one pinned tracking issue (auto-closed on the next green run).

- [ ] **Step 1: Create the issue template**

Create `.github/nightly-failure-issue.md`:

```markdown
---
title: "Nightly CI failing"
labels: nightly-failure
---
The scheduled **nightly CI** run failed.

- Failing run: {{ env.RUN_URL }}
- Commit: {{ sha }}

This issue is auto-managed: it is updated on each consecutive nightly failure and
closed automatically the next time nightly CI passes. Do not close it by hand —
a green nightly will close it.
```

- [ ] **Step 2: Create the nightly workflow**

Create `.github/workflows/nightly.yml`:

```yaml
name: Nightly CI

on:
  schedule:
    - cron: "0 6 * * *" # 06:00 UTC daily
  workflow_dispatch:

permissions:
  contents: read
  issues: write

jobs:
  nightly:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 0 # gitleaks (mise run secrets) scans full history

      - name: Set up mise
        uses: jdx/mise-action@v2
        with:
          install: false

      - name: Cache Rust toolchain and Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.rustup
            ~/.cargo
          key: ${{ runner.os }}-rust-${{ hashFiles('mise.toml', 'Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-rust-

      - name: Install mise-managed tools
        run: mise install

      - name: Run nightly tier
        run: mise run ci-nightly

      - name: Open/update tracking issue on failure
        if: failure()
        uses: JasonEtco/create-an-issue@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RUN_URL: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}
        with:
          filename: .github/nightly-failure-issue.md
          update_existing: true
          search_existing: open

      - name: Close tracking issue on success
        if: success()
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          gh issue list --label nightly-failure --state open --json number --jq '.[].number' \
            | while read -r n; do
                [ -n "$n" ] && gh issue close "$n" --comment "Nightly CI green again on ${{ github.sha }}."
              done
```

- [ ] **Step 3: Verify both files parse**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/nightly.yml')); print('nightly.yml OK')"`
Expected: `nightly.yml OK`. If `python3`/`pyyaml` is unavailable, eyeball `git diff --staged` indentation instead.

Run: `head -4 .github/nightly-failure-issue.md`
Expected: the YAML frontmatter with `title:` and `labels: nightly-failure`.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/nightly.yml .github/nightly-failure-issue.md
git commit -m "ci(tiering): add fail-loud nightly workflow with pinned tracking issue"
```

- [ ] **Step 5: Record the one manual step**

The nightly issue automation needs the `nightly-failure` label to exist. `JasonEtco/create-an-issue` creates a missing label automatically on first failure, so no action is required up front; if you prefer to pre-create it, a maintainer runs once: `gh label create nightly-failure --description "Auto-managed nightly CI failure tracker" --color b60205`. Note this in the PR description alongside the existing Renovate-app manual step from Phase 1.

---

### Task 6: Phase acceptance check (safety rule + budget + no release drift)

**Files:**
- None created or modified — verification only.

**Interfaces:**
- Consumes: everything above.
- Produces: evidence that the design's acceptance criteria and safety rule hold.

- [ ] **Step 1: Assert the safety rule — every nightly task is reachable from `release-gate`**

The nightly tier is `test-full`, `package-check`, `benchmark`. Confirm each appears in `release-gate`'s `depends`:

Run: `grep -A2 '^\[tasks.release-gate\]' mise.toml | grep depends`
Expected: a `depends` line containing `test-full`, `package-check`, **and** `benchmark`. If any nightly task is missing here, the safety rule is violated — stop.

- [ ] **Step 2: Confirm `release-gate`'s `depends` is unchanged from before the phase**

Run: `git diff 282a3310..HEAD -- mise.toml | grep -E '^\+' | grep -i 'release-gate' || echo "release-gate depends unchanged"`
Expected: `release-gate depends unchanged` (no added line touches the `[tasks.release-gate]` block). `282a3310` is the pre-implementation commit (the design-clarification commit); substitute the actual commit this phase started from if later commits landed first.

- [ ] **Step 3: Confirm no release-facing config or crate code changed**

Run: `git diff 282a3310..HEAD -- release.toml release-plz.toml cliff.toml crates/`
Expected: empty output.

- [ ] **Step 4: Confirm doctest coverage is preserved at release time**

Run: `mise run test-full 2>&1 | grep -i 'doc-tests' | head -1`
Expected: a `Doc-tests <crate>` line — proof that `test-full` (and therefore `release-gate`) still exercises doctests after the nextest swap.

- [ ] **Step 5: Run the full blocking gate and confirm green**

Run: `mise run ci`
Expected: PASS. This is the re-scoped blocking tier; note it no longer runs `test-full` or `package-check` but still runs `test` (nextest), `doctest`, `release-smoke`, `deny`, `secrets`.

- [ ] **Step 6: Record real CI wall-clock after the first push (post-merge follow-up)**

After the branch's first CI run on GitHub, note the blocking job's actual duration. If it exceeds the 13-min timeout, apply the design's documented fallback: move `release-smoke` from `[tasks.ci].depends` to `[tasks.ci-nightly].depends` (it stays reachable from `release-gate`, so the safety rule holds). This is a follow-up observation, not a blocker for merging the plan.
```
