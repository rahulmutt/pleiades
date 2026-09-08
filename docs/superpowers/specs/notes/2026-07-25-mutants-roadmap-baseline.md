# FU-9 next-campaign roadmap baseline (2026-07-25)

Measured at the close of the `pleiades-houses` campaign (FU-9 PR 6), so the
next slice's ordering rests on data rather than crate size.

Command per crate:

```bash
cargo mutants -p <crate> --test-tool nextest --test-workspace=false --baseline run
```

## Measured survivor baselines

| Crate | Mutants | Missed | Caught | Unviable | Score |
|-------|---------|--------|--------|----------|-------|
| ~~`pleiades-apsides`~~ (triaged 2026-09-08) | 223 | ~~33~~ → 1 | ~~186~~ → 218 | 4 | ~~84.9%~~ → 99.5% |
| `pleiades-backend` | 263 | 70 | 145 | 48 | 67.4% |
| `pleiades-ayanamsa` | 305 | 85 | 196 | 24 | 69.8% |
| `pleiades-fict`¹ | 308 | 148 | 147 | 11 | 49.5%¹ |
| **Total** | **1099** | **336** | **674** | **87** | **66.6%**¹ |

Score = `caught / (tested - unviable)`, one decimal place on the percentage,
per the task brief's formula. This is only equivalent to `caught / (caught +
missed)` — how the FU-9 baseline note phrases it
(`docs/superpowers/specs/notes/2026-07-18-mutants-baseline.md:35`) — when
there are zero timeouts; the two diverge once timeouts appear.
`pleiades-fict` is the one row where they diverge: `caught / (tested -
unviable)` = 147/297 = **49.5%** (timeouts counted in the denominator, the
figure used above) versus `caught / (caught + missed)` = 147/295 = 49.8%
(timeouts excluded from both). This table applies the brief's formula
uniformly, including to `pleiades-fict`, rather than asserting the two
formulas match when they do not for that row.

Verbatim `cargo-mutants` summary lines (tails of Step 1's four runs):

```
223 mutants tested in 2m: 33 missed, 186 caught, 4 unviable
263 mutants tested in 5m: 70 missed, 145 caught, 48 unviable
305 mutants tested in 6m: 85 missed, 196 caught, 24 unviable
308 mutants tested in 4m: 148 missed, 147 caught, 11 unviable, 2 timeouts
```

Exit codes: `apsides`/`backend`/`ayanamsa` exited 2 (survivors found — the
report-only tier's expected outcome, per `mise.toml`'s `[tasks.mutants]`
comment). `pleiades-fict` exited 3.

¹ **`pleiades-fict` is not a clean measurement — treat the row as
provisional, not a peer of the three above.** The run exited 3
(`cargo-mutants`'s `Timeout` code — 2 of the 308 mutants exceeded their test
timeout; see `exit_code.rs` in the `cargo-mutants` 27.1.0 source, which
defines exit 3 as "one or more tests timed out," distinct from exit 4
(`BaselineFailed`, "tests are already failing in an unmutated tree")). The
data is kept, not discarded, and no re-run was attempted: the baseline
itself passed and the run produced a complete, internally consistent
breakdown (148 missed + 147 caught + 11 unviable + 2 timeouts = 308,
matching "308 mutants tested" exactly). But the `Score` (49.5%) is computed
over only 306 of the 308 verdicts — the 2 timeouts are neither caught nor
missed nor unviable — so it is **not directly comparable to the
`pleiades-apsides` / `pleiades-backend` / `pleiades-ayanamsa` rows**, whose
scores rest on a complete verdict for every tested mutant.

Note on precedent: `docs/follow-ups.md:1083-1084` reports a run in the same
`N missed / M caught / K unviable / T timeout` shape, which establishes that
this campaign already uses that four-way vocabulary — but that cited run
reported `0 timeout` and exited 2, so it does **not** establish that a
genuine timeout-bearing exit-3 run has previously been accepted here as a
clean baseline. No such precedent is claimed.

`Missed + Caught + Unviable` (148 + 147 + 11 = 306) is 2 short of `Mutants`
(308) by design, not an arithmetic error — the difference is the 2
timeouts. The `Total` row's `Score` (66.6%) inherits this same
non-comparability because it aggregates `pleiades-fict`'s 306-verdict row
alongside three complete rows. The remedy, out of scope for this task, is a
re-run of `pleiades-fict` with a raised test timeout, which would resolve
the 2 timeouts to caught/missed and produce a clean, comparable row.

## Not measured — mutant counts only

Enumerated with `cargo mutants -p <crate> --list | wc -l` on 2026-07-25 and
verified against the plan's figures (all eight matched exactly; no
corrections were needed). These are **sizing figures, not survivor counts**;
no ordering may be inferred from them.

| Crate | Mutants | Crate | Mutants |
|-------|---------|-------|---------|
| `pleiades-compression` | 607 | `pleiades-elp` | 1,521 |
| `pleiades-eclipse` | 913 | `pleiades-data` | 1,752 |
| `pleiades-core` | 962 | `pleiades-events` | 1,901 |
| `pleiades-vsop87` | 1,493 | `pleiades-jpl` | 3,662 |

Eight-crate sizing subtotal: 12,811. Combined with the four measured crates'
1,099 tested mutants, the twelve-crate total is **13,910** (the plan's
"~13,900" estimate, recomputed here from verified figures rather than
carried forward).

## Posture

Report-only; this note gates nothing. No parity gate
(`validate-houses`/`validate-angles` corpora, tolerances, gate code) or
production code was touched producing these numbers.

**Queue status (updated 2026-09-08).** `pleiades-apsides` is **triaged and
closed** by the 2026-09-08 expansion slice — 33 missed → 1, the single
residual survivor (`120:43`) carrying a written reachability argument at its
site in `crates/pleiades-apsides/src/lib.rs` (no `#[mutants::skip]`). Its row above is
struck through and its post-triage figures shown; the pre-triage figures are
retained only as the measurement this note originally recorded, not as an open
survivor count. The **Total** row is deliberately left at its as-measured
2026-07-25 values (1099 / 336) — it is the historical baseline aggregate, not a
live count; the live open-survivor total is 336 - 33 = **303** across the three
crates below. (`99.5%` = 218 caught / 219 viable.) See the FU-9 "apsides" entry in `docs/follow-ups.md`.

The **remaining queue is three crates**: `pleiades-backend` (70 missed),
`pleiades-ayanamsa` (85), and `pleiades-fict` (148, provisional — see the
footnote above). Their survivor counts are still open and still accurate.
`pleiades-apsides` has also joined the weekly report-only mutants tier in
`mise.toml`'s `[tasks.mutants]`; the other three have not.
