# cargo-mutants Baseline — Devkit Phase 3

**Date:** 2026-07-18
**Tool:** cargo-mutants 27.1.0 (pinned in `mise.toml`)
**Toolchain:** stable 1.97.1
**Commit:** `5eaeaaadd17d4271f65df9232e2c5ca035499f48`
**Design:** [`../2026-07-18-devkit-phase3-mutants-slice-design.md`](../2026-07-18-devkit-phase3-mutants-slice-design.md)

**Invocation:** `mise run mutants`
(`--test-tool nextest --test-workspace=false --baseline run`, three crates)

## Result

| Crate | Mutants | Caught | Missed | Unviable | Score |
| --- | ---: | ---: | ---: | ---: | ---: |
| `pleiades-types` | 311 | 232 | 41 | 38 | 85.0% |
| `pleiades-time` | 323 | 262 | 49 | 12 | 84.2% |
| `pleiades-apparent` | 817 | 576 | 228 | 13 | 71.6% |
| **Total** | **1451** | **1070** | **318** | **63** | **77.1%** |

**Wall-clock:** 6m52.478s (`real`) at `-j4` (`CARGO_MUTANTS_JOBS=4`, deliberately
restricted on a 24-core dev box to stay comparable to the 4-vCPU GitHub
`ubuntu-latest` runner)
**Exit code:** 2 (surviving mutants found — expected for a first baseline, and a
passing outcome for a report-only tier)

Mutation score = caught / (caught + missed), excluding unviable mutants.
Overall: 1070 / (1070 + 318) = 1070 / 1388 = 77.1%.

## Survivors by file

```
     49 crates/pleiades-apparent/src/apparent.rs
     45 crates/pleiades-apparent/src/nutation.rs
     37 crates/pleiades-apparent/src/refraction.rs
     28 crates/pleiades-apparent/src/aberration.rs
     27 crates/pleiades-apparent/src/topocentric.rs
     17 crates/pleiades-apparent/src/sidereal.rs
     17 crates/pleiades-apparent/src/precession.rs
     16 crates/pleiades-time/src/convert.rs
     12 crates/pleiades-types/src/zodiac.rs
     10 crates/pleiades-types/src/time.rs
     10 crates/pleiades-time/src/deltat.rs
      9 crates/pleiades-time/src/tdb.rs
      9 crates/pleiades-time/src/calendar.rs
      5 crates/pleiades-time/src/sidereal.rs
      5 crates/pleiades-apparent/src/lighttime.rs
      4 crates/pleiades-types/src/time_range.rs
      3 crates/pleiades-types/src/coordinates.rs
      3 crates/pleiades-types/src/ayanamsa.rs
      3 crates/pleiades-types/src/angles.rs
      3 crates/pleiades-apparent/src/provenance.rs
      2 crates/pleiades-types/src/motion.rs
      2 crates/pleiades-types/src/house_systems.rs
      1 crates/pleiades-types/src/observer.rs
      1 crates/pleiades-types/src/frames.rs
```

## Assessment

Survivors are meaningful, not trivia. The plan anticipated a `Display`/`Debug`/
accessor-dominated tail; that did not materialise. A representative sample of
the top of `missed.txt`:

```
crates/pleiades-apparent/src/nutation.rs:69:67: replace + with - in fundamental_arguments
crates/pleiades-apparent/src/nutation.rs:69:45: replace - with + in fundamental_arguments
crates/pleiades-apparent/src/nutation.rs:69:63: replace * with + in fundamental_arguments
crates/pleiades-apparent/src/nutation.rs:69:79: replace / with % in fundamental_arguments
crates/pleiades-apparent/src/nutation.rs:70:44: replace - with / in fundamental_arguments
```

These are arithmetic-operator swaps inside polynomial series evaluation
(`fundamental_arguments`), not cosmetic paths. Survivors concentrate in
`apparent.rs`, `nutation.rs`, `refraction.rs`, `aberration.rs`, and
`topocentric.rs` — all release-grade numeric code in `pleiades-apparent`,
which is why that crate scores lowest at 71.6%.

A hypothesis, to be confirmed (not assumed) during FU-9 triage: the repo's
numeric gates are tolerance-based, so an operator swap in a low-order
polynomial term can perturb the result by less than the assertion tolerance,
letting the test pass and the mutant survive. If confirmed, these survivors
are a signal about gate *tightness*, not about missing test coverage — but
that has not been established yet.

No blanket exclusion is recommended here. The small tail (e.g. the 3
`provenance.rs` survivors) may turn out to be `#[mutants::skip]` /
`--skip-calls` candidates once triaged, but the numeric bulk must be worked
through, not suppressed — suppressing it would hide exactly the signal this
tier exists to surface.

## Reproducibility notes

The plan anticipated proptest-seed score jitter in `pleiades-types` and asked
for a second run to measure it. A repeat run of `mise run mutants-crate
pleiades-types` produced:

```
311 mutants tested in 3m: 41 missed, 232 caught, 38 unviable
```

identical to the baseline's `pleiades-types` portion in all three counts
(41 / 232 / 38) — a delta of zero mutants across two runs.

Based on that two-run comparison, we say "no jitter observed across two
runs", not "there is no jitter" — a real but limited sample. Pinning
`PROPTEST_CASES` or a fixed seed is **not** recommended: week-over-week
baselines are already comparable without it, and pinning would add
unjustified complexity.

## Posture

Report-only. This score gates nothing. Survivors are tracked as FU-9 in
`docs/follow-ups.md`. Re-measured weekly by `.github/workflows/mutants.yml`.
