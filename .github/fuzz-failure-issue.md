---
title: "Fuzz campaign failure"
labels: bug, fuzzing, security, fuzzing-failure
---

The scheduled fuzz campaign found a crash, hang, or OOM.

Run: {{ env.RUN_URL }}

The crashing input is attached to that run as the `fuzz-artifacts` artifact.
To reproduce locally:

```bash
mise exec -- bash -c 'cargo "+$FUZZ_NIGHTLY" fuzz run <target> fuzz/artifacts/<target>/<crash-file>'
```

(`FUZZ_NIGHTLY` is set by `mise.toml` and is the single source of truth for
the pinned fuzzing nightly date; do not hardcode a toolchain date here. The
`bash -c '...'` wrapper matters: it defers `$FUZZ_NIGHTLY` expansion into the
subprocess `mise exec` spawns. Without it — e.g. `mise exec -- echo
"+$FUZZ_NIGHTLY"` — your outer shell expands the double-quoted variable
*before* `mise exec` ever runs, since `mise exec` only injects the variable
into the child process it launches, not into how its own argv is parsed; you
would silently get `+` instead of the nightly date. If your shell already has
`FUZZ_NIGHTLY` exported — e.g. via `mise activate` or `mise en` — you can skip
`mise exec` and run the plain
`cargo "+$FUZZ_NIGHTLY" fuzz run <target> fuzz/artifacts/<target>/<crash-file>`
directly.)

Per `docs/superpowers/specs/2026-07-18-devkit-phase3-fuzz-slice-design.md`,
a finding is fixed with a committed reproducer that runs as an ordinary unit
test in the blocking tier — not only as a corpus entry.

This issue is updated rather than duplicated on repeat failures, and closed
automatically when a campaign next passes.
