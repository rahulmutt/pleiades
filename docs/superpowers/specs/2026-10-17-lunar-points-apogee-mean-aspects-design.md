# Osculating-apogee and mean-point aspects, with a graze-window rule (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-17 ·
**Issue:** #168 (f), the rest of the lunar points, and (c), grazes ·
**Crates/tools:** `tools/se-aspects-reference`, `pleiades-validate`, docs

## Context

`validate-aspects` gates the true node's aspects with seven planets against
Swiss Ephemeris SWIEPH (`lunar-points.csv`, #269). Its reference tool refuses
any pair whose separation turns within 3″ of a level.

The osculating apogee (`TrueApogee`, SE body 13) was left out because it
disagreed with SWIEPH by up to 86″. #271 found the cause: our Earth–Moon μ
was 1.43e-5 too large. With the μ fixed, the apogee matches SWIEPH
`SE_OSCU_APOG` to 1.78″ max (0.27″ rms), daily over 1900–2100.

The mean node (`MeanNode`, SE 10) and mean apogee (`MeanApogee`, SE 12) are
analytic. `validate-mean-lunar-points` already holds them within 0.57″ of
Swiss Ephemeris, but their aspects are not gated.

## Measurements (spike, 2026-10-17, on #271's fixed μ)

The spike was throwaway. It used the same SE scan as #269:
- SWIEPH with the pinned `sepl_18`/`semo_18`;
- a 0.005 d grid, splitting each step at the zero of the relative speed;
- JD 2415025.5–2488064.5 and angles 0/60/90/120/180.

The engine ran on the packaged backend at its default step.

| Pair | SE events | SE turns | Closest turn to a level | Unmatched | Max sep residual | p99 | Max \|Δt\| | Max \|engine−SE sep\| at SE turns |
|---|---|---|---|---|---|---|---|---|
| TrueApogee–Sun | 3555 | 4618 | **8.77″** | 0 / 0 | 2.32″ | 0.88″ | 771 s | 3.56″ |
| TrueApogee–Jupiter | 3438 | 4802 | **1.84″** | 0 / 0 | 3.00″ | 0.85″ | 162 s | 3.57″ |
| TrueApogee–Saturn | 3485 | 4798 | 14.60″ | 0 / 0 | 3.11″ | 0.89″ | 37 s | 3.57″ |
| TrueApogee–Pluto | 3421 | 4816 | **7.47″** | 0 / 0 | 2.48″ | 0.88″ | 119 s | 3.60″ |

Before the μ fix the same pairs had 16 events present in only one
ephemeris. Now every event pairs one for one.

Two things block the 3″ refusal that #269 used:

1. **A 3″ margin is not safe for the apogee.** At SE's turning points the
   engine and SE separations differ by up to 3.6″, against 0.82″ for the true
   node. A margin of 3.3 × 3.6″ ≈ **12″** is needed before "SE has, or lacks,
   this pair of events" means the same for the engine.
2. **Some turns fall inside 12″** (8.8″ Sun, 1.8″ Jupiter, 7.5″ Pluto), so a
   refusal would drop three of the four pairs. Events next to such turns
   also have unstable timing: the Sun pair's 771 s time residual turns into a
   43.5″ longitude residual at the event instant.

So this design replaces refusal with a **graze window**: a stretch of time
where events may legitimately exist in one ephemeris and not the other.

## Goal

Gate, event for event against SWIEPH, in `lunar-points.csv`:
- TrueApogee with the Sun, Jupiter, Saturn and Pluto;
- MeanNode with the Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto;
- MeanApogee with the Sun, Jupiter, Saturn and Pluto.

Grazes are handled by an explicit, checksummed window rule instead of a
refusal.

Non-goals:

- **Graze windows for the planet and asteroid modes.** They keep their 30″
  refusal, and their output stays byte-identical.
- **Fictitious bodies, mean or osculating perigee, and other frames.**
- **Any engine change.**
- **#168 (a)** (two events of one angle within one engine step).

## Reference tool (`tools/se-aspects-reference --lunar-points`)

**Bodies and pairs:**
- Add `(13, "TrueApogee")`, `(10, "MeanNode")` and `(12, "MeanApogee")`.
- `LUNAR_POINT_PAIRS` becomes the seven TrueNode pairs, followed by the four
  TrueApogee pairs, the seven MeanNode pairs and the four MeanApogee pairs:
  22 pairs, all over `FULL_SPAN`.
- Flags, grid (0.005 d), pins and the CSV schema are unchanged.

**Margin per point** (a function of the pair's first body):

| Point | Margin | Measured engine−SE sep at turns |
|---|---|---|
| TrueNode, MeanNode, MeanApogee | 3″ | 0.82″ (TrueNode); MeanNode and MeanApogee are measured by the plan as the gate's maximum separation residual over their pairs, which must be ≤ 1″ |
| TrueApogee | 12″ | 3.6″ |

If the plan measures a mean point above 1″ at turns, its margin becomes
3.3 × the measured value. The plan records the figure either way.

**Graze windows, not refusal.** In lunar-points mode, a turn whose separation
lies within the pair's margin of a level no longer asserts. The tool:

1. Walks outward from the turn, on the scan's SE samples, in both
   directions. It stops at the first grid sample where |separation − level|
   ≥ 2 × margin. The two samples are the window `[lo_jd, hi_jd]`.
2. Merges the window with any overlapping window of the same pair and angle.
3. Counts SE's events of that pair and angle inside the merged window.

The window's **parity** is set by its edges. Both edges are at least
2 × margin from the level, so they lie on one side of it or on opposite
sides:
- For an isolated turn, both edges are on the turn's far side, and the count
  is even.
- A point whose speed oscillates can wiggle near the level while its overall
  trend carries it across. Then the edges are on opposite sides, and the
  count is odd: 1 or 3.

The tool asserts that SE's count has the parity its edges imply. By the
intermediate value theorem it always does, unless a root was missed, so the
check is a sanity assert. (Ruling, 2026-10-17: the first design said "always
even"; a smoke run at a widened margin found a TrueNode–Jupiter window
holding 1 event.)

It writes `lunar-points-grazes.csv`, one row per merged window:

```
group,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events
```

- `turn_jd_tt` and `miss_arcsec` are the closest turn in the window.
- The file has a header comment like the other corpora: source, pins, the
  margin table, and the window rule.

It has its own one-line manifest, `lunar-points-grazes-manifest.txt`:
`slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows=N checksum=<fnv1a64>`.
(The shared `corpus_manifest::slice_entry` reads one `slice` line per file,
so a separate file keeps that parser unchanged.)

**Stderr.** Each turn's miss is still logged per pair, and each window is
logged as it is written.

**Unchanged output.**
- Planet and asteroid modes keep `assert!(miss >= margin)` with 30″. The plan
  regenerates `aspects.csv` and `asteroids.csv` and `cmp`s them against the
  committed files.
- The seven TrueNode pairs' rows in `lunar-points.csv` must regenerate
  byte-identically: none of them has a turn within 3″, so none gets a window.

**Building:** as in #269's PR, with `CFLAGS=-std=gnu17` and libclang from
`devenv.nix`. `devenv.nix` is unchanged.

## Gate (`crates/pleiades-validate`, `Scope::LunarPoints`)

**Data:**
- the committed `lunar-points-grazes.csv` and
  `lunar-points-grazes-manifest.txt`, read with `include_str!` and checked
  like the corpus (rows, checksum).

**Bodies:** `body_from_name` gains `TrueApogee`, `MeanNode` and `MeanApogee`.
`LUNAR_POINT_PAIRS` grows to the same 22 pairs, in the tool's order.

**Window matching**, per pair and angle. A window applies to both levels of
its angle, e.g. ±60°:

1. **Consistency.** Every grazes row names a known pair and angle, has
   `lo < hi`, and does not overlap another window of the same pair and angle.
   Its `se_events` equals the number of corpus rows of that pair and angle
   inside `[lo, hi]`. A failure is a typed `AspectsError`, like the existing
   manifest and row-drift errors.
2. **Split.** Engine events and corpus rows fall either inside a window
   (`lo ≤ jd ≤ hi`) or outside every window.
3. **Outside: strict, unchanged.** `compare_exact` on the outside events of
   both sides: equal counts, side checks and ceilings, as today.
4. **Inside: one check per window.**
   - Each side's count must have the window's parity, `se_events % 2`. The
     engine shares the edge signs, because the edges are at least
     2 × margin from the level and that exceeds the engine–SE difference.
     A count of the wrong parity fails (`AspectsError::GrazeWindowParity`).
   - Any split of the right parity is accepted: 0/2, 2/0, 2/2 or 4/2 in an
     even window, and 1/1, 1/3 or 3/1 in an odd one.
   - When both counts are equal and non-zero, the events pair in order and
     their separation, time and longitude residuals are **reported, not
     checked** against ceilings.
5. **Report.** Each lunar pair's line adds the windows, the windows whose
   counts differ, and the window events compared, with their maxima. So a
   window that hides a mismatch is visible in the nightly log.

**Ceilings** (`aspects_thresholds.rs`):
- The rule stays ×1.4 of the measured maximum outside windows, floored at
  2.3″.
- The spike's figures suggest about 3.3–4.4″ separation for TrueApogee and
  2.3″ for the mean points. The longitude ceiling is set from the measured
  outside-window maximum. The plan measures and records each pair.
- A pair whose outside-window longitude residual exceeds 10″ is a finding to
  investigate, not a ceiling to set.

**Floors:** `MIN_ROWS_VALIDATED_LUNAR_POINTS` becomes the regenerated count
of corpus rows **outside** graze windows. Those rows are always compared;
window rows may legitimately go uncompared (a 0/2 window). TrueApogee's per-pair counts must equal the spike's table above; a
difference is investigated, not accepted.

**Blocking tier:** `validate_aspects_corpus_subset()` already checks the
lunar manifest without scanning. It gains the grazes file's checksum and
rows, and the window consistency check (item 1, no engine call).

**Tests** (`aspects_validation/tests.rs`). The window matcher is a pure
function over `Found`/`Expected` slices and window bounds, unit-tested with
synthetic rows:
- in an even window, 0/2, 2/0 and 2/2 accepted, and 1/2 and 2/1 rejected;
- in an odd window, 1/1, 1/3 and 3/1 accepted, and 2/1 and 0/1 rejected;
- an event just outside a window held to the strict count;
- two windows of one pair and angle;
- a window of one angle that does not affect another angle;
- consistency errors: overlap, `lo ≥ hi`, an `se_events` mismatch, an unknown
  pair;
- the grazes-file tamper and row-drift tests, like the existing manifest
  tests;
- updated pair-list and floor pins.

## Runtime and tiers

The full gate (`mise run gate-aspects`) measured **5m58s** on the nightly
runner on 2026-10-16, run 38971941852, including the true-node pass. The new
15 pairs add scan time; the apogee's frequent turns make it the expensive
one. The plan measures the full gate locally before and after, at recorded
load.

If the local after-figure, scaled by the measured local/nightly ratio of the
true-node pass, predicts more than 15 minutes on the nightly runner, the
lunar-points pass moves to its own subcommand (`validate-aspects-lunar`) and
nightly job (`aspects-lunar-gate`). It then also becomes a `release-gate`
dependency. That is a conditional plan step. Otherwise nothing moves, and
the first nightly run after merge is reported on #168.

**Outcome (2026-10-17):** the prediction was about 5 minutes (927 s local ÷ 3.2),
but a nightly run dispatched on the branch measured the combined gate step at
**15m40s** (run 39039081162). The local-to-nightly ratio of the true-node pass
did not carry over to the apogee pairs. So the split was made: the subcommand
`validate-aspects-lunar`, the mise task `gate-aspects-lunar`, and the nightly
job `aspects-lunar-gate`, which is a `release-gate` dependency.

## Documentation

- `crates/pleiades-events/README.md`, which says which aspects are gated:
  the true node, osculating apogee, mean node and mean apogee are gated;
  fictitious bodies and the perigees are not.
- The `aspects_validation.rs` header and the `validate_aspects_corpus` doc.
- The `mise.toml` `gate-aspects` comment, `AGENTS.md`'s CI-tier paragraph,
  `docs/development.md` and `docs/status.md`: the pair lists, and the
  nightly runtime (5m58s measured, replacing "not yet measured").
- The CLI help and the gate-failure issue template, wherever they list
  gated pairs.
- No compatibility-profile change, as in #251 and #269.
- **#168:**
  - (f): every lunar point except the perigees is gated.
  - (c): graze windows are in place for lunar points, with the closest gated
    graze measured.

## Testing summary

- **Tool:**
  - `aspects.csv` and `asteroids.csv` regenerate byte-identically;
  - the TrueNode rows of `lunar-points.csv` regenerate byte-identically;
  - TrueApogee's per-pair counts equal the spike's.
- **Gate:**
  - the full `mise run gate-aspects` passes;
  - each lunar pair line reports its windows;
  - `release-smoke`'s manifest and window-consistency check passes;
  - the matcher unit tests pass.
- `mise run test-full`, `mise run audit` and `mise run docs`.
