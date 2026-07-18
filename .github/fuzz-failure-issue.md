---
title: "Fuzz campaign failure"
labels: bug, fuzzing, security
---

The scheduled fuzz campaign found a crash, hang, or OOM.

Run: {{ env.RUN_URL }}

The crashing input is attached to that run as the `fuzz-artifacts` artifact.
To reproduce locally:

```bash
cargo "+$FUZZ_NIGHTLY" fuzz run <target> fuzz/artifacts/<target>/<crash-file>
```

(`FUZZ_NIGHTLY` is set by `mise.toml` — run this via `mise exec` or any shell
where `mise activate`/`mise en` has exported it. It is the single source of
truth for the pinned fuzzing nightly date; do not hardcode a toolchain date
here.)

Per `docs/superpowers/specs/2026-07-18-devkit-phase3-fuzz-slice-design.md`,
a finding is fixed with a committed reproducer that runs as an ordinary unit
test in the blocking tier — not only as a corpus entry.

This issue is updated rather than duplicated on repeat failures, and closed
automatically when a campaign next passes.
