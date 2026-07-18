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
