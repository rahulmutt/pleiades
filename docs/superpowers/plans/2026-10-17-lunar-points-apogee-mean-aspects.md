# Osculating-Apogee and Mean-Point Aspects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gate the aspects of TrueApogee (4 pairs), MeanNode (7) and MeanApogee (4) event for event against Swiss Ephemeris SWIEPH, in `lunar-points.csv`, next to the true node's 7 pairs. Near-tangent turns are handled by an explicit, checksummed graze-window rule instead of a refusal.

**Architecture:**
- `tools/se-aspects-reference` gets a `Graze` policy for `scan`. `Refuse(margin)` keeps today's assert for the planet and asteroid modes. `Window { margin, out }` records each turn inside the margin as a window in a new `lunar-points-grazes.csv`.
- `pleiades-validate` gets a `aspects_validation/grazes.rs` submodule, which parses and checks the windows and splits each series into a strict outside part and per-window parity checks. `validate_scoped` gains an optional grazes input, used only by `Scope::LunarPoints`.

**Tech Stack:** Rust; Swiss Ephemeris via `libswisseph-sys` 0.1.2 (the tool only); the pleiades packaged backend.

**Spec:** `docs/superpowers/specs/2026-10-17-lunar-points-apogee-mean-aspects-design.md`

## Global Constraints

- **Pairs, in this order (22):**
  - TrueNode with Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto (unchanged, first);
  - TrueApogee with Sun, Jupiter, Saturn and Pluto;
  - MeanNode with Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto;
  - MeanApogee with Sun, Jupiter, Saturn and Pluto.
  All are group `geo`, span `FULL_SPAN` (JD 2415025.5–2488064.5), angles 0/60/90/120/180, tool grid 0.005 d, flags `GEO_SWIEPH`.
- **SE body ids:**
  - 11 `TrueNode`;
  - 13 `TrueApogee` (`SE_OSCU_APOG`);
  - 10 `MeanNode` (`SE_MEAN_NODE`);
  - 12 `MeanApogee` (`SE_MEAN_APOG`).
  The engine's `CelestialBody` variants have the same names.
- **Graze margins:**
  - planet and asteroid modes: 30″, refuse, with output byte-identical to the committed `aspects.csv` (checksum `4506302360407709888`) and `asteroids.csv` (`8132397916751651688`);
  - lunar mode: TrueApogee 12″; TrueNode, MeanNode and MeanApogee 3″. A turn inside the margin becomes a window.
- **Window rule (tool):** walk outward from the turn on the scan's grid instants, `span.0 + k as f64 * grid`, to the first instant on each side where `|wrap180(separation − level)| ≥ 2 × margin`.
  - Merge overlapping windows of one pair and angle.
  - Count SE's events of that pair and angle inside `[lo, hi]`, and assert the count's parity matches the window's edges: even when both edges lie on one side of the level, odd when they straddle it. (Ruling 2026-10-17: a speed-oscillating point can wiggle near a level while its trend crosses it; the Task 1 smoke found a 1-event window.)
  - A window that reaches the span's edge aborts the run.
- **Grazes CSV schema:** `group,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events`. Format: `{turn:.7},{miss_arcsec:.3},{lo:.7},{hi:.7},{n}`.
- **Grazes manifest:** `lunar-points-grazes-manifest.txt`, containing `slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows=N checksum=<fnv1a64>`.
- **Expected TrueApogee counts** (spike, 2026-10-17): Sun 3555, Jupiter 3438, Saturn 3485, Pluto 3421. The TrueNode rows must regenerate byte-identically to the committed ones. Any difference is investigated, never accepted silently.
- **Gate window rule:** outside windows, strict `compare_exact` as today. Inside, each side's count must have the window's parity (`se_events % 2`), any split of that parity is accepted, and equal non-zero counts are paired and their residuals reported, not ceiling-checked.
- **Floor:** `MIN_ROWS_VALIDATED_LUNAR_POINTS` = the corpus rows outside windows.
- **Ceilings:** ×1.4 of the measured outside-window maximum, rounded up to two significant figures, floored at 2.3″. A pair whose outside-window longitude maximum exceeds 10″ is a finding to stop on, not a ceiling to set.
- **No changes to:** `devenv.nix`, the compatibility profile, or any engine crate.
- **Pinned SWIEPH files:** already SHA-verified at `/tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe` (pass `--ephe <dir>`).
- **Building the SE tool:** `env CFLAGS=-std=gnu17 LIBCLANG_PATH=/nix/store/jdgw7h0g0l8clmcasaspxnx6v62jz1il-clang-21.1.8-lib/lib cargo build --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`. The tool has its own `[workspace]`.
- **Tests:** validate with libtest, `cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored` for the slow tier, or `cargo test -p pleiades-validate --lib aspects_validation` for this module. Never use nextest here.
- **Commits:** run `cargo fmt --all` before each commit, and clippy with `-D warnings`.
- **Subagent time limit:** any single command over 10 minutes (corpus generation, the full gate) is run by the controller in the background, not by a subagent.

## Review Focus

1. **The planet and asteroid corpora must not change.** The `Graze` refactor must leave `Refuse` behaving exactly as the old assert. Pinned in Task 1, Step 5, by regenerating both and comparing them with `cmp`.
2. **A tampered `lunar-points-grazes.csv`, a stale grazes manifest, or a window whose `se_events` disagrees with the corpus** must fail the blocking tier (`validate_aspects_corpus_subset`). Pinned in Task 3, Step 6.
3. **An event just outside a window must stay strict.** A `<` versus `<=` slip at the window edge would silently excuse a real count mismatch. Pinned in Task 3, Step 1 (`an_event_just_outside_a_window_stays_strict`).
4. **A window must not leak across angles or pairs.** A 60° window must not excuse a 90° event at the same instant. Pinned in Task 3, Step 1 (`a_window_applies_only_to_its_own_series`).
5. **SE may reject SWIEPH for the analytic mean points.** `state()` panics if the returned flags lack `SEFLG_SWIEPH`. Swiss Ephemeris returns the caller's flags for `SE_MEAN_NODE` and `SE_MEAN_APOG`, but if it doesn't, the run aborts. Covered in Task 1, Step 6, with an exemption limited to body ids 10 and 12.

---

### Task 1: The reference tool's graze windows and new lunar points

**Files:**
- Modify: `tools/se-aspects-reference/src/main.rs`

**Interfaces:**
- Produces: `--lunar-points --out <dir> --ephe <dir>` writes `lunar-points.csv`, `lunar-points-manifest.txt`, `lunar-points-grazes.csv` and `lunar-points-grazes-manifest.txt`. Task 2 runs it.

- [ ] **Step 1: Add the bodies, margins and pair table.** Next to `TRUE_NODE`:

```rust
/// SE_OSCU_APOG: the osculating lunar apogee ("true Lilith").
const TRUE_APOGEE: Body = (13, "TrueApogee");
/// SE_MEAN_NODE: the mean lunar ascending node (analytic).
const MEAN_NODE: Body = (10, "MeanNode");
/// SE_MEAN_APOG: the mean lunar apogee (analytic).
const MEAN_APOGEE: Body = (12, "MeanApogee");
```

Replace the `LUNAR_POINT_GRAZE_MARGIN_DEG` doc comment and constant, and `LUNAR_POINT_PAIRS`, with:

```rust
/// 3 arcseconds, for the true node and the mean points: 3.6 times the largest
/// engine-minus-SE SWIEPH separation at a turning point measured for the true
/// node (0.82"). A turn this close to a level becomes a graze window, not an
/// error (issue #168 (c)).
const LUNAR_POINT_GRAZE_MARGIN_DEG: f64 = 3.0 / 3600.0;
/// 12 arcseconds, for the osculating apogee: 3.3 times the largest
/// engine-minus-SE SWIEPH separation at its turning points (3.6", spike
/// 2026-10-17, after the apsides mu fix of #271).
const TRUE_APOGEE_GRAZE_MARGIN_DEG: f64 = 12.0 / 3600.0;

/// The graze margin of a lunar-point pair, chosen by its first body.
fn lunar_point_margin_deg(first: Body) -> f64 {
    if first.0 == TRUE_APOGEE.0 {
        TRUE_APOGEE_GRAZE_MARGIN_DEG
    } else {
        LUNAR_POINT_GRAZE_MARGIN_DEG
    }
}

/// The lunar-point mode's pairs, all `geo` with `GEO_SWIEPH` over the full span.
const LUNAR_POINT_PAIRS: [(Body, Body); 22] = [
    (TRUE_NODE, SUN),
    (TRUE_NODE, MARS),
    (TRUE_NODE, JUPITER),
    (TRUE_NODE, SATURN),
    (TRUE_NODE, URANUS),
    (TRUE_NODE, NEPTUNE),
    (TRUE_NODE, PLUTO),
    (TRUE_APOGEE, SUN),
    (TRUE_APOGEE, JUPITER),
    (TRUE_APOGEE, SATURN),
    (TRUE_APOGEE, PLUTO),
    (MEAN_NODE, SUN),
    (MEAN_NODE, MARS),
    (MEAN_NODE, JUPITER),
    (MEAN_NODE, SATURN),
    (MEAN_NODE, URANUS),
    (MEAN_NODE, NEPTUNE),
    (MEAN_NODE, PLUTO),
    (MEAN_APOGEE, SUN),
    (MEAN_APOGEE, JUPITER),
    (MEAN_APOGEE, SATURN),
    (MEAN_APOGEE, PLUTO),
];
```

- [ ] **Step 2: Add the `Graze` policy and use it in `scan`.** Above `fn scan`:

```rust
/// What `scan` does with a turning point inside the margin of a level.
enum Graze<'a> {
    /// Abort the run: an event there would depend on the ephemeris
    /// (planet and asteroid modes).
    Refuse(f64),
    /// Record a graze window in `out` (lunar-point mode, issue #168 (c)).
    Window {
        margin_deg: f64,
        out: &'a mut dyn FnMut(String),
    },
}

impl Graze<'_> {
    fn margin_deg(&self) -> f64 {
        match self {
            Self::Refuse(margin) => *margin,
            Self::Window { margin_deg, .. } => *margin_deg,
        }
    }
}
```

Change `scan`'s last parameter from `graze_margin_deg: f64` to `graze: Graze`. At the top of `scan`, add `let margin_deg = graze.margin_deg();` and `let mut close_turns: Vec<(usize, f64, f64, f64)> = Vec::new();`. That vector holds `(angle index, level, turn jd, miss deg)`.

Replace the per-level loop at the turn (`for angle in ANGLES { for level in levels(angle) { … assert!(…) } }`) with:

```rust
            for (index, angle) in ANGLES.iter().enumerate() {
                for level in levels(*angle) {
                    let miss = wrap180(turn.separation - level).abs();
                    closest_miss = closest_miss.min(miss);
                    if let Graze::Refuse(_) = graze {
                        assert!(
                            miss >= margin_deg,
                            "graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level \
                             at jd_tt={:.5}, inside the {:.0} arcsec margin; an event here would \
                             depend on the ephemeris",
                            first.1,
                            second.1,
                            miss * 3600.0,
                            turn.jd,
                            margin_deg * 3600.0
                        );
                    } else if miss < margin_deg {
                        close_turns.push((index, level, turn.jd, miss));
                    }
                }
            }
```

The panic text must be exactly the existing one. `{angle}` formats an `f64` either way, so it prints the same.

At the end of `scan`, after the loop that sorts and writes each angle's events, add:

```rust
    if let Graze::Window { out: windows_out, .. } = graze {
        write_graze_windows(
            windows_out,
            (group, first, second),
            &at,
            &events,
            &close_turns,
            (lo, hi),
            grid,
            margin_deg,
        );
    }
```

Then add the function below `scan`:

```rust
/// Turns each close turn into a window `[lo, hi]` on the scan's grid: the first
/// grid instant on each side at which the separation is at least twice the
/// margin from the level. Merges overlapping windows of one angle, counts the
/// angle's events inside, and writes one grazes row per merged window. The
/// count's parity is set by the edges: even when both lie on one side of the
/// level, odd when they straddle it (a wiggle on a crossing trend). A count of
/// the other parity (a missed root), or a window reaching the span's edge, aborts.
#[allow(clippy::too_many_arguments)]
fn write_graze_windows(
    out: &mut dyn FnMut(String),
    (group, first, second): (&str, Body, Body),
    at: &dyn Fn(f64) -> Sample,
    events: &[Vec<Sample>],
    close_turns: &[(usize, f64, f64, f64)],
    (span_lo, span_hi): (f64, f64),
    grid: f64,
    margin_deg: f64,
) {
    let steps = ((span_hi - span_lo) / grid).floor() as i64;
    let instant = |k: i64| span_lo + k as f64 * grid;
    // (lo, hi, turn_jd, miss_deg, level) per angle. The two levels of one angle
    // are at least 120 degrees apart, so overlapping windows share their level.
    let mut windows: Vec<Vec<(f64, f64, f64, f64, f64)>> = vec![Vec::new(); ANGLES.len()];
    for &(index, level, turn_jd, miss) in close_turns {
        let far = |k: i64| wrap180(at(instant(k)).separation - level).abs() >= 2.0 * margin_deg;
        let mut k_lo = ((turn_jd - span_lo) / grid).floor() as i64;
        while k_lo > 0 && !far(k_lo) {
            k_lo -= 1;
        }
        let mut k_hi = k_lo + 1;
        while k_hi < steps && !far(k_hi) {
            k_hi += 1;
        }
        assert!(
            far(k_lo) && far(k_hi),
            "graze window of {group},{},{} at jd_tt={turn_jd:.5} reaches the span's edge",
            first.1,
            second.1
        );
        windows[index].push((instant(k_lo), instant(k_hi), turn_jd, miss, level));
    }
    for (index, angle) in ANGLES.iter().enumerate() {
        let mut list = std::mem::take(&mut windows[index]);
        list.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut merged: Vec<(f64, f64, f64, f64, f64)> = Vec::new();
        for w in list {
            match merged.last_mut() {
                Some(last) if w.0 <= last.1 => {
                    last.1 = last.1.max(w.1);
                    if w.3 < last.3 {
                        last.2 = w.2;
                        last.3 = w.3;
                    }
                }
                _ => merged.push(w),
            }
        }
        for (lo, hi, turn_jd, miss, level) in merged {
            let inside = events[index]
                .iter()
                .filter(|e| lo <= e.jd && e.jd <= hi)
                .count();
            let above = |jd: f64| wrap180(at(jd).separation - level) > 0.0;
            let straddles = above(lo) != above(hi);
            assert!(
                inside % 2 == usize::from(straddles),
                "graze window of {group},{},{},{angle:.0} [{lo:.7}, {hi:.7}] holds {inside} SE events, but its edges imply an {} count",
                first.1,
                second.1,
                if straddles { "odd" } else { "even" }
            );
            eprintln!(
                "{group},{},{},{angle:.0}: graze window [{lo:.5}, {hi:.5}], closest turn {:.2} arcsec at {turn_jd:.5}, {inside} SE events",
                first.1,
                second.1,
                miss * 3600.0
            );
            out(format!(
                "{group},{},{},{angle:.0},{turn_jd:.7},{:.3},{lo:.7},{hi:.7},{inside}",
                first.1,
                second.1,
                miss * 3600.0
            ));
        }
    }
}
```

`at` in `scan` is a closure `|jd| sample(…)`. Pass it as `&at`. If the borrow checker objects to `at` being `Fn` while `graze` is matched by value, move the `if let Graze::Window` block after every other use of `at`. It already is the last statement.

- [ ] **Step 3: Update the callers.**
  - Every `scan(…, PLANET_GRAZE_MARGIN_DEG)` call in `main` and `asteroids_main` becomes `scan(…, Graze::Refuse(PLANET_GRAZE_MARGIN_DEG))`.
  - In `lunar_points_main`, build a second string `grazes` with its own header and pass `Graze::Window { margin_deg: lunar_point_margin_deg(first), out: &mut push_graze }`, where `push_graze` appends to `grazes` and counts `graze_rows`.

The lunar corpus header lines change to:

```rust
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags) with SEFLG_SWIEPH; TrueNode (SE_TRUE_NODE) with Sun, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto; TrueApogee (SE_OSCU_APOG) with Sun, Jupiter, Saturn, Pluto; MeanNode (SE_MEAN_NODE) with the TrueNode planets; MeanApogee (SE_MEAN_APOG) with the TrueApogee planets; JD 2415025.5-2488064.5.\n");
```

and the last comment line becomes:

```rust
    csv.push_str("# A turning point within the pair's graze margin of a level (TrueApogee 12 arcsec, the other points 3 arcsec) is a graze window in lunar-points-grazes.csv; events inside a window may exist in one ephemeris only.\n");
```

The grazes file starts with:

```rust
    grazes.push_str("# Source: as lunar-points.csv (Swiss Ephemeris 2.10.03, SEFLG_SWIEPH, sepl_18/semo_18 SHA-256 pinned).\n");
    grazes.push_str("# One row per graze window of a lunar-point pair and angle: a turning point of the separation within the pair's\n");
    grazes.push_str("# margin of a level (TrueApogee 12 arcsec, TrueNode/MeanNode/MeanApogee 3 arcsec). lo_jd_tt and hi_jd_tt are the first\n");
    grazes.push_str("# 0.005-day grid instants on each side at which |separation - level| >= 2 x margin; overlapping windows are merged and\n");
    grazes.push_str("# turn_jd_tt/miss_arcsec are the closest turn. se_events counts lunar-points.csv rows of the pair and angle inside [lo, hi]\n");
    grazes.push_str("# (even when both edges lie on one side of the level, odd when they straddle it). Events inside a window may exist\n");
    grazes.push_str("# in one ephemeris only; the gate requires each side's count to have se_events' parity (issue #168 (c)).\n");
    grazes.push_str("group,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events\n");
```

Write both files and both manifests. The grazes manifest line is `slice aspects-lunar-points-grazes file=lunar-points-grazes.csv role=grazes rows={graze_rows} checksum={}`, using `pins::fnv1a64(&grazes)`. Two `FnMut` closures that both borrow distinct strings are fine. If both would need `rows`, keep separate counters.

Update the module doc block (the `//!` lines near 49–53) to say what the lunar mode scans and that it writes graze windows instead of refusing.

- [ ] **Step 4: Build.** Run the build command from Global Constraints. Expected: it compiles without warnings. Then run `cargo clippy --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml -- -D warnings` with the same `env` prefix.

- [ ] **Step 5: Prove the planet and asteroid modes are unchanged.** Run, into a scratch directory:
  - `tools/se-aspects-reference/target/release/se-aspects-reference > /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/lp-regen/aspects.csv` (Moshier, stdout);
  - `tools/se-aspects-reference/target/release/se-aspects-reference --asteroids --out /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/lp-regen --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe`.

  Then `cmp` each against `crates/pleiades-validate/data/aspects-corpus/aspects.csv` and `asteroids.csv`. Expected: no output. If either run exceeds 10 minutes, stop and report. The controller runs it.

- [ ] **Step 6: Smoke the mean points and the apogee on a short span.** Temporarily, and do not commit this, run the lunar mode with `FULL_SPAN` replaced by `(2_451_545.0, 2_455_197.5)` for a quick check, or add a throwaway `--span lo,hi` override you remove before committing. Expected: no panic, and the stderr turn-count lines appear for all 22 pairs.

  If `state()` panics "fell back from SWIEPH" for ipl 10 or 12, change the fallback check to:

```rust
    // SE_MEAN_NODE (10) and SE_MEAN_APOG (12) are analytic: Swiss Ephemeris
    // computes them without an ephemeris file.
    let analytic = ipl == 10 || ipl == 12;
    if iflag & SEFLG_SWIEPH != 0 && ret & SEFLG_SWIEPH == 0 && !analytic {
```

  Note in the report which branch you took.

- [ ] **Step 7: Commit.**

```bash
cargo fmt --all
git add tools/se-aspects-reference/src/main.rs
git commit -m "feat(tools): graze windows and the osculating apogee and mean points in se-aspects-reference --lunar-points (#168)"
```

---

### Task 2 (controller): Generate the lunar-point corpus

**Files:**
- Modify (generated): `crates/pleiades-validate/data/aspects-corpus/lunar-points.csv`, `lunar-points-manifest.txt`
- Create (generated): `crates/pleiades-validate/data/aspects-corpus/lunar-points-grazes.csv`, `lunar-points-grazes-manifest.txt`

- [ ] **Step 1: Run in the background** (it takes more than 10 minutes):
  `tools/se-aspects-reference/target/release/se-aspects-reference --lunar-points --out crates/pleiades-validate/data/aspects-corpus --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe 2> /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/lp-gen.log`

- [ ] **Step 2: Check the output against the spike and #269.**
  - The TrueNode rows are byte-identical: compare `grep '^geo,TrueNode,'` of the new file with the same rows of `git show HEAD:crates/pleiades-validate/data/aspects-corpus/lunar-points.csv`.
  - TrueApogee per-pair counts (`grep -c '^geo,TrueApogee,Sun,'` etc.) are 3555, 3438, 3485 and 3421.
  - `lp-gen.log` lists a window for TrueApogee–Jupiter near JD 2420750.8 (the 1.84″ turn), and none for TrueNode.
  - Record the MeanNode and MeanApogee counts, the windows per pair, and the wall-clock time.

- [ ] **Step 3: Commit.**

```bash
git add crates/pleiades-validate/data/aspects-corpus/lunar-points.csv crates/pleiades-validate/data/aspects-corpus/lunar-points-manifest.txt crates/pleiades-validate/data/aspects-corpus/lunar-points-grazes.csv crates/pleiades-validate/data/aspects-corpus/lunar-points-grazes-manifest.txt
git commit -m "data(validate): lunar-point aspect corpus with TrueApogee, MeanNode and MeanApogee, and graze windows (#168)"
```

The gate's own tests fail until Task 3. Committing the data first gives Task 3 real inputs. The intermediate commit is not pushed alone.

---

### Task 3: The gate's graze windows

**Files:**
- Create: `crates/pleiades-validate/src/aspects_validation/grazes.rs`
- Modify: `crates/pleiades-validate/src/aspects_validation.rs`, `crates/pleiades-validate/src/aspects_validation/tests.rs`, `crates/pleiades-validate/src/aspects_thresholds.rs`

**Interfaces:**
- Consumes: Task 2's four files.
- Produces:
  - `grazes::GrazeWindow { pair: usize, angle: usize, lo: f64, hi: f64, se_events: usize }`;
  - `grazes::check_grazes(csv: &str, manifest: &str, pairs: &[Pair], rows: &[Row]) -> Result<Vec<GrazeWindow>, AspectsError>`;
  - `grazes::split_windows(label: &str, windows: &[GrazeWindow], engine: &[Found], corpus: &[Expected]) -> Result<(Vec<Found>, Vec<Expected>, WindowStats), AspectsError>`;
  - `WindowStats { windows, differing, compared: usize, max_sep_arcsec, max_time_s, max_lon_arcsec: f64 }` with `absorb`;
  - `validate_scoped(csv, manifest, scope, grazes: Option<(&str, &str)>)`;
  - `validate_subset(asteroid_csv, asteroid_manifest, lunar_csv, lunar_manifest, grazes_csv, grazes_manifest)`.

- [ ] **Step 1: Write the failing unit tests.** Append to `aspects_validation/tests.rs`. It already has `found(jd, first, second)` and `expected(jd, first, second, rel_speed)`. Add `use super::grazes::{split_windows, GrazeWindow};` to the imports.

```rust
fn window(angle: usize, lo: f64, hi: f64) -> GrazeWindow {
    GrazeWindow { pair: 0, angle, lo, hi, se_events: 0 }
}

/// `n` events at 60 degrees, spaced 0.1 day from `start`, first body ahead.
fn sextiles_from(start: f64, n: usize) -> Vec<Expected> {
    (0..n)
        .map(|i| expected(start + 0.1 * i as f64, 60.0, 0.0, 0.5))
        .collect()
}

#[test]
fn a_graze_window_accepts_any_even_split() {
    let w = [GrazeWindow { se_events: 2, ..window(1, 10.0, 12.0) }];
    for (got, want) in [(0, 2), (2, 0), (2, 2), (4, 2), (0, 0)] {
        let mut corpus = sextiles_from(10.5, want);
        corpus.push(expected(20.0, 60.0, 0.0, 0.5));
        let mut engine = engine_of(&sextiles_from(10.5, got));
        engine.push(found(20.0, 60.0, 0.0));
        let (eng_out, cor_out, stats) =
            split_windows("t", &w, &engine, &corpus).expect("even split is accepted");
        assert_eq!(eng_out, vec![found(20.0, 60.0, 0.0)]);
        assert_eq!(cor_out.len(), 1);
        assert_eq!(stats.windows, 1);
        assert_eq!(stats.differing, usize::from(got != want));
        let compared = if got == want { got } else { 0 };
        assert_eq!(stats.compared, compared);
    }
}

#[test]
fn a_count_of_the_wrong_parity_in_a_graze_window_fails() {
    // An even window (se_events 2): an odd count on either side fails.
    let even = [GrazeWindow { se_events: 2, ..window(1, 10.0, 12.0) }];
    for (got, want) in [(1, 2), (2, 1), (3, 0)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        assert!(matches!(
            split_windows("t", &even, &engine, &corpus),
            Err(AspectsError::GrazeWindowParity { .. })
        ));
    }
    // An odd window (se_events 1, edges straddling the level): an even count fails.
    let odd = [GrazeWindow { se_events: 1, ..window(1, 10.0, 12.0) }];
    for (got, want) in [(2, 1), (0, 1), (1, 2)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        assert!(matches!(
            split_windows("t", &odd, &engine, &corpus),
            Err(AspectsError::GrazeWindowParity { .. })
        ));
    }
}

#[test]
fn an_odd_window_accepts_any_odd_split() {
    let w = [GrazeWindow { se_events: 1, ..window(1, 10.0, 12.0) }];
    for (got, want) in [(1, 1), (3, 1), (1, 3)] {
        let corpus = sextiles_from(10.5, want);
        let engine = engine_of(&sextiles_from(10.5, got));
        let (_, _, stats) =
            split_windows("t", &w, &engine, &corpus).expect("odd split in an odd window");
        assert_eq!(stats.differing, usize::from(got != want));
    }
}

#[test]
fn an_event_just_outside_a_window_stays_strict() {
    let w = [window(1, 10.0, 12.0)];
    let corpus = vec![expected(12.000_001, 60.0, 0.0, 0.5)];
    let (eng_out, cor_out, stats) = split_windows("t", &w, &[], &corpus).unwrap();
    assert!(eng_out.is_empty());
    assert_eq!(cor_out.len(), 1, "an event past hi is not excused");
    assert_eq!(stats.windows, 1);
    let ceilings = Ceilings { sep_arcsec: 1.0, lon_arcsec: 1.0 };
    assert!(matches!(
        compare_exact("t", 60.0, &eng_out, &cor_out, ceilings),
        Err(AspectsError::CountMismatch { .. })
    ));
    // Both window edges are inside.
    let edges = vec![expected(10.0, 60.0, 0.0, 0.5), expected(12.0, 60.0, 0.0, 0.5)];
    let (_, cor_out, _) = split_windows("t", &w, &[], &edges).unwrap();
    assert!(cor_out.is_empty());
}

#[test]
fn equal_window_counts_report_residuals_without_ceilings() {
    let w = [window(1, 10.0, 12.0)];
    let corpus = sextiles_from(10.5, 2);
    // 600 s late and 40" off in longitude: far past any ceiling, still accepted.
    let engine: Vec<Found> = corpus
        .iter()
        .map(|e| found(e.found.jd + 600.0 / 86_400.0, 60.0 + 40.0 / 3600.0, 0.0))
        .collect();
    let (_, _, stats) = split_windows("t", &w, &engine, &corpus).unwrap();
    assert_eq!(stats.compared, 2);
    assert!((stats.max_time_s - 600.0).abs() < 1e-3);
    assert!((stats.max_lon_arcsec - 40.0).abs() < 1e-6);
    assert!(stats.max_sep_arcsec > 0.0);
}

#[test]
fn two_windows_of_one_series_are_checked_separately() {
    let w = [
        GrazeWindow { se_events: 2, ..window(1, 10.0, 12.0) },
        GrazeWindow { se_events: 2, ..window(1, 30.0, 32.0) },
    ];
    let mut corpus = sextiles_from(10.5, 2);
    corpus.extend(sextiles_from(30.5, 2));
    // 0 in the first window, 3 in the second: the second fails on its own.
    let engine = engine_of(&sextiles_from(30.5, 3));
    assert!(matches!(
        split_windows("t", &w, &engine, &corpus),
        Err(AspectsError::GrazeWindowParity { .. })
    ));
    let engine = engine_of(&sextiles_from(30.5, 2));
    let (_, _, stats) = split_windows("t", &w, &engine, &corpus).unwrap();
    assert_eq!((stats.windows, stats.differing, stats.compared), (2, 1, 2));
}

#[test]
fn a_window_applies_only_to_its_own_series() {
    // A 60-degree window of pair 0 must not excuse a 90-degree row, or a row of
    // pair 1, at the same instant.
    let rows = [
        Row { pair: 0, angle: 1, expected: expected(10.5, 60.0, 0.0, 0.5) },
        Row { pair: 0, angle: 1, expected: expected(10.6, 60.0, 0.0, 0.5) },
        Row { pair: 0, angle: 2, expected: expected(10.55, 90.0, 0.0, 0.5) },
        Row { pair: 1, angle: 1, expected: expected(10.55, 60.0, 0.0, 0.5) },
    ];
    let windows = [GrazeWindow { pair: 0, angle: 1, lo: 10.0, hi: 12.0, se_events: 2 }];
    let of_series = super::grazes::windows_of(&windows, 0, 2);
    assert!(of_series.is_empty());
    let of_series = super::grazes::windows_of(&windows, 1, 1);
    assert!(of_series.is_empty());
    assert_eq!(super::grazes::windows_of(&windows, 0, 1).len(), 1);
    assert!(super::grazes::check_window_counts(&windows, &rows).is_ok());
}
```

Then parsing and consistency tests:

```rust
fn grazes_csv(rows: &[&str]) -> String {
    let mut csv = String::from(
        "# test\ngroup,first,second,angle_deg,turn_jd_tt,miss_arcsec,lo_jd_tt,hi_jd_tt,se_events\n",
    );
    for row in rows {
        csv.push_str(row);
        csv.push('\n');
    }
    csv
}

#[test]
fn graze_rows_parse_and_reject_bad_windows() {
    use super::grazes::parse_grazes;
    let ok = grazes_csv(&["geo,TrueApogee,Jupiter,60,2420750.8200000,1.840,2420750.5000000,2420751.1000000,2"]);
    let windows = parse_grazes(&ok, &LUNAR_POINT_PAIRS).expect("parses");
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].angle, 1);
    assert_eq!(windows[0].se_events, 2);
    for bad in [
        // lo >= hi
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420751.1,2420750.5,2",
        // turn outside the window
        "geo,TrueApogee,Jupiter,60,2420752.0,1.8,2420750.5,2420751.1,2",
        // unknown pair
        "geo,TrueApogee,Mars,60,2420750.8,1.8,2420750.5,2420751.1,2",
        // angle not in the plan
        "geo,TrueApogee,Jupiter,45,2420750.8,1.8,2420750.5,2420751.1,2",
        // wrong field count
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420750.5,2420751.1",
    ] {
        assert!(
            matches!(parse_grazes(&grazes_csv(&[bad]), &LUNAR_POINT_PAIRS), Err(AspectsError::MalformedGrazes(_))),
            "{bad}"
        );
    }
    let overlapping = grazes_csv(&[
        "geo,TrueApogee,Jupiter,60,2420750.8,1.8,2420750.5,2420751.1,2",
        "geo,TrueApogee,Jupiter,60,2420751.0,2.0,2420750.9,2420751.4,0",
    ]);
    assert!(matches!(
        parse_grazes(&overlapping, &LUNAR_POINT_PAIRS),
        Err(AspectsError::MalformedGrazes(_))
    ));
}

#[test]
fn a_window_whose_se_events_disagree_with_the_corpus_fails() {
    let rows = [Row { pair: 0, angle: 1, expected: expected(10.5, 60.0, 0.0, 0.5) },
                Row { pair: 0, angle: 1, expected: expected(10.6, 60.0, 0.0, 0.5) }];
    let windows = [GrazeWindow { pair: 0, angle: 1, lo: 10.0, hi: 12.0, se_events: 0 }];
    assert!(matches!(
        super::grazes::check_window_counts(&windows, &rows),
        Err(AspectsError::GrazeEventCount { recorded: 0, corpus: 2, .. })
    ));
}
```

- [ ] **Step 2: Run them to see them fail.** `cargo test -p pleiades-validate --lib aspects_validation 2>&1 | tail -20`. Expected: compile errors, because `grazes` does not exist yet.

- [ ] **Step 3: Write `grazes.rs`.**

```rust
//! Graze windows of the lunar-point aspect corpus (issue #168 (c)).
//!
//! `tools/se-aspects-reference --lunar-points` writes a window wherever a
//! pair's separation turns within its graze margin of a level. There an event
//! pair may exist in one ephemeris only, so the gate compares events inside a
//! window by count parity, and everything outside strictly.

use super::{
    first_is_ahead, lon_residual_arcsec, AspectsError, Expected, Found, Group, Pair, Row,
    ANGLES_DEG, ARCSEC_PER_DEG, SECONDS_PER_DAY,
};

/// One graze window: indices into the corpus plan and [`ANGLES_DEG`], and the
/// window's bounds (Julian days, TT, both inclusive).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GrazeWindow {
    pub(super) pair: usize,
    pub(super) angle: usize,
    pub(super) lo: f64,
    pub(super) hi: f64,
    /// The corpus rows of the pair and angle inside the window.
    pub(super) se_events: usize,
}

impl GrazeWindow {
    fn contains(&self, jd: f64) -> bool {
        self.lo <= jd && jd <= self.hi
    }
}

/// What the window comparison saw for one series or more.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct WindowStats {
    pub(super) windows: usize,
    /// Windows whose engine and corpus counts differ (allowed when both have
    /// the window's parity).
    pub(super) differing: usize,
    /// Events paired inside windows whose counts agree (not ceiling-checked).
    pub(super) compared: usize,
    pub(super) max_sep_arcsec: f64,
    pub(super) max_time_s: f64,
    pub(super) max_lon_arcsec: f64,
}

impl WindowStats {
    pub(super) fn absorb(&mut self, other: WindowStats) {
        self.windows += other.windows;
        self.differing += other.differing;
        self.compared += other.compared;
        self.max_sep_arcsec = self.max_sep_arcsec.max(other.max_sep_arcsec);
        self.max_time_s = self.max_time_s.max(other.max_time_s);
        self.max_lon_arcsec = self.max_lon_arcsec.max(other.max_lon_arcsec);
    }
}

/// Parses the grazes CSV against the corpus plan `pairs`. Every window has
/// `lo < hi`, holds its turn, and does not overlap
/// the previous window of its pair and angle (rows ascend by `lo`).
pub(super) fn parse_grazes(csv: &str, pairs: &[Pair]) -> Result<Vec<GrazeWindow>, AspectsError> {
    let malformed = |what: String| AspectsError::MalformedGrazes(what);
    let mut windows: Vec<GrazeWindow> = Vec::new();
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("group,") {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 9 {
            return Err(malformed(format!("expected 9 fields, got {} in {line}", fields.len())));
        }
        let group = Group::from_name(fields[0])
            .ok_or_else(|| malformed(format!("unknown group {} in {line}", fields[0])))?;
        let pair = pairs
            .iter()
            .position(|p| p.group == group && p.first == fields[1] && p.second == fields[2])
            .ok_or_else(|| malformed(format!("pair {}-{} is not in the plan in {line}", fields[1], fields[2])))?;
        let num = |i: usize| -> Result<f64, AspectsError> {
            fields[i]
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| malformed(format!("field {i} is not a finite number in {line}")))
        };
        let angle_deg = num(3)?;
        let angle = ANGLES_DEG
            .iter()
            .position(|known| *known == angle_deg)
            .ok_or_else(|| malformed(format!("angle {angle_deg} is not in the plan in {line}")))?;
        let (turn, lo, hi) = (num(4)?, num(6)?, num(7)?);
        num(5)?;
        let se_events: usize = fields[8]
            .parse()
            .map_err(|_| malformed(format!("se_events is not a count in {line}")))?;
        if !(lo < hi && lo <= turn && turn <= hi) {
            return Err(malformed(format!("window does not hold its turn, or lo >= hi, in {line}")));
        }
        if let Some(prev) = windows.iter().rev().find(|w| w.pair == pair && w.angle == angle) {
            if lo <= prev.hi {
                return Err(malformed(format!("window overlaps or precedes the previous one in {line}")));
            }
        }
        windows.push(GrazeWindow { pair, angle, lo, hi, se_events });
    }
    Ok(windows)
}

/// Each window's `se_events` must equal the corpus rows of its pair and angle
/// inside it.
pub(super) fn check_window_counts(windows: &[GrazeWindow], rows: &[Row]) -> Result<(), AspectsError> {
    for w in windows {
        let corpus = rows
            .iter()
            .filter(|r| r.pair == w.pair && r.angle == w.angle && w.contains(r.expected.found.jd))
            .count();
        if corpus != w.se_events {
            return Err(AspectsError::GrazeEventCount {
                pair: w.pair,
                angle_deg: ANGLES_DEG[w.angle],
                lo: w.lo,
                hi: w.hi,
                recorded: w.se_events,
                corpus,
            });
        }
    }
    Ok(())
}

/// Ties the grazes file to its manifest and to the corpus rows: checksum, row
/// count, parse, and window counts.
pub(super) fn check_grazes(
    csv: &str,
    manifest: &str,
    pairs: &[Pair],
    rows: &[Row],
) -> Result<Vec<GrazeWindow>, AspectsError> {
    let (manifest_rows, manifest_checksum) = super::parse_manifest(manifest)?;
    let got = pleiades_apparent::fnv1a64(csv);
    if got != manifest_checksum {
        return Err(AspectsError::ChecksumMismatch { got, want: manifest_checksum });
    }
    let windows = parse_grazes(csv, pairs)?;
    if windows.len() != manifest_rows {
        return Err(AspectsError::ManifestDrift {
            rows_csv: windows.len(),
            rows_manifest: manifest_rows,
        });
    }
    check_window_counts(&windows, rows)?;
    Ok(windows)
}

/// The windows of one pair and angle, in order.
pub(super) fn windows_of(windows: &[GrazeWindow], pair: usize, angle: usize) -> Vec<GrazeWindow> {
    windows
        .iter()
        .filter(|w| w.pair == pair && w.angle == angle)
        .copied()
        .collect()
}

/// Splits one series at its `windows`: returns the engine events and corpus
/// rows outside every window, for the strict comparison, and what the windows
/// held. Inside each window both counts must have the window's parity
/// (`se_events % 2`, set by whether its edges straddle the level); equal non-zero counts
/// are paired in order and their residuals recorded, without ceilings.
pub(super) fn split_windows(
    label: &str,
    windows: &[GrazeWindow],
    engine: &[Found],
    corpus: &[Expected],
) -> Result<(Vec<Found>, Vec<Expected>, WindowStats), AspectsError> {
    let mut stats = WindowStats::default();
    for w in windows {
        let got: Vec<&Found> = engine.iter().filter(|f| w.contains(f.jd)).collect();
        let want: Vec<&Expected> = corpus.iter().filter(|e| w.contains(e.found.jd)).collect();
        let parity = w.se_events % 2;
        for (side, count) in [("engine", got.len()), ("corpus", want.len())] {
            if count % 2 != parity {
                return Err(AspectsError::GrazeWindowParity {
                    series: label.to_string(),
                    lo: w.lo,
                    hi: w.hi,
                    side,
                    count,
                    se_events: w.se_events,
                });
            }
        }
        stats.windows += 1;
        if got.len() != want.len() {
            stats.differing += 1;
            continue;
        }
        for (g, e) in got.iter().zip(&want) {
            let (sep, time_s, lon) = super::residuals_of(g, e);
            stats.compared += 1;
            stats.max_sep_arcsec = stats.max_sep_arcsec.max(sep);
            stats.max_time_s = stats.max_time_s.max(time_s.abs());
            stats.max_lon_arcsec = stats.max_lon_arcsec.max(lon);
        }
    }
    let outside = |jd: f64| !windows.iter().any(|w| w.contains(jd));
    let engine_out = engine.iter().filter(|f| outside(f.jd)).copied().collect();
    let corpus_out = corpus.iter().filter(|e| outside(e.found.jd)).copied().collect();
    Ok((engine_out, corpus_out, stats))
}
```

If `first_is_ahead`, `ARCSEC_PER_DEG` or `SECONDS_PER_DAY` end up unused in `grazes.rs`, drop them from the `use` list.

- [ ] **Step 4: Wire it into `aspects_validation.rs`.**
  1. Add `mod grazes;` (after the `use` block) and `use grazes::{check_grazes, split_windows, windows_of, WindowStats};`.
  2. **Constants:** add `LUNAR_POINT_GRAZES_CSV` and `LUNAR_POINT_GRAZES_MANIFEST` with `include_str!` of `lunar-points-grazes.csv` and `lunar-points-grazes-manifest.txt`, like the others.
  3. **Error variants**, with their `Display` arms:

```rust
    /// The grazes file is malformed (issue #168 (c)).
    MalformedGrazes(String),
    /// A graze window's recorded SE count disagrees with the corpus.
    GrazeEventCount {
        pair: usize,
        angle_deg: f64,
        lo: f64,
        hi: f64,
        recorded: usize,
        corpus: usize,
    },
    /// A graze window holds a count of the wrong parity on one side.
    GrazeWindowParity {
        series: String,
        lo: f64,
        hi: f64,
        side: &'static str,
        count: usize,
        se_events: usize,
    },
```

```rust
            Self::MalformedGrazes(s) => write!(f, "malformed grazes row: {s}"),
            Self::GrazeEventCount { pair, angle_deg, lo, hi, recorded, corpus } => write!(
                f,
                "graze window [{lo}, {hi}] of pair {pair} at {angle_deg} degrees records {recorded} SE events, the corpus has {corpus}"
            ),
            Self::GrazeWindowParity { series, lo, hi, side, count, se_events } => write!(
                f,
                "{series}: graze window [{lo}, {hi}] holds {count} {side} events; its parity must match the {se_events} SE events'"
            ),
```

  4. **Extract the residual formula** from `compare_exact` into `fn residuals_of(got: &Found, want: &Expected) -> (f64, f64, f64)`, returning `(sep_arcsec, signed_time_s, max(first_lon, second_lon))`. Make `compare_exact` call it, so the formula exists once.
  5. **`validate_scoped`** gains `grazes: Option<(&str, &str)>`. After `let rows = check_manifest(…)?;`:

```rust
    let windows = match grazes {
        Some((csv, manifest)) => check_grazes(csv, manifest, scope.pairs(), &rows)?,
        None => Vec::new(),
    };
```

  Keep a `let mut window_stats = WindowStats::default();` per pair. In the per-angle loop, replace the `compare_exact` call with:

```rust
            let (found, corpus, stats) = split_windows(
                &label,
                &windows_of(&windows, pair_index, angle_index),
                &found,
                &corpus,
            )?;
            window_stats.absorb(stats);
            residuals.absorb(compare_exact(&label, *angle_deg, &found, &corpus, ceilings)?);
```

  The counts per angle still print `found.len()` before the split. Keep that `counts.push` above the split, so it reports the engine total. After the existing `pair_lines.push(format!(…))`, when `scope == Scope::LunarPoints`, append to that last line:

```rust
            line.push_str(&format!(
                ", graze windows {} ({} differing, {} events compared inside, max sep {:.3}\", max time {:.1} s, max lon {:.3}\")",
                window_stats.windows,
                window_stats.differing,
                window_stats.compared,
                window_stats.max_sep_arcsec,
                window_stats.max_time_s,
                window_stats.max_lon_arcsec,
            ));
```

  Build the line in a `let mut line = format!(…)` first, then push it.
  6. **Callers:** `validate` and the asteroid call pass `None`. The lunar call in `validate_aspects_corpus` passes `Some((LUNAR_POINT_GRAZES_CSV, LUNAR_POINT_GRAZES_MANIFEST))`. `validate_subset` gains `grazes_csv, grazes_manifest` and, after `check_manifest(lunar_csv, …)`, calls:

```rust
    let lunar_rows = check_manifest(lunar_csv, lunar_manifest, &LUNAR_POINT_PAIRS)?;
    check_grazes(grazes_csv, grazes_manifest, &LUNAR_POINT_PAIRS, &lunar_rows)?;
```

  `validate_aspects_corpus_subset` passes the two new constants. Update every test call site of `validate_scoped`/`validate_subset` the same way (`grep -n "validate_scoped(\|validate_subset(" crates/pleiades-validate/src`).
  7. **Plan and names:**
     - `LUNAR_POINT_PAIRS: [Pair; 22]`, in the Global Constraints order.
     - `body_from_name` gains `"TrueApogee" => CelestialBody::TrueApogee`, `"MeanNode" => CelestialBody::MeanNode` and `"MeanApogee" => CelestialBody::MeanApogee`.
     - `Scope::LunarPoints.title()` becomes `"Lunar-point aspects"`. Grep the tests for `"True-node aspects"` and update them.
     - Update the `Scope::LunarPoints` doc and the module header paragraph "True node (issue #168)" to name all four points and the window rule.

- [ ] **Step 5: Provisional ceilings and floor**, replaced in Task 4. In `aspects_thresholds.rs` add arms for the 15 new pair names with **provisional** ceilings, marked as such:
  - `TrueApogee-<planet>`: sep 10.0, lon 60.0;
  - `MeanNode-<planet>` and `MeanApogee-<planet>`: sep 5.0, lon 5.0.

  Each gets the comment `// PROVISIONAL (Task 3 of the 2026-10-17 plan); measured in Task 4`.

  Set `MIN_ROWS_VALIDATED_LUNAR_POINTS` to the corpus rows outside windows. Compute it with a scratch test, or by counting `lunar-points.csv` rows minus the `se_events` total of `lunar-points-grazes.csv`. Update its doc comment.

- [ ] **Step 6: Update the existing lunar tests.**
  - `lunar_point_corpus_parses_into_the_seven_true_node_pairs` → `lunar_point_corpus_parses_into_the_twenty_two_lunar_pairs`, with the 22 names in order.
  - `lunar_point_scope_floor_is_its_own`: the manifest rows minus the window `se_events` sum equals the floor.
  - Add:
    - `committed_graze_windows_match_their_manifest_and_corpus`: `check_grazes` on the committed files is `Ok`, has a TrueApogee–Jupiter window holding JD 2420750.82 (the spike's 1.84″ turn; the spike did not record its angle, so match any angle), and every window's pair is a TrueApogee, MeanNode or MeanApogee pair;
    - `mean_subset_fails_tampered_graze_windows`: change one `se_events` value with `replacen` to a different even number, then expect `ChecksumMismatch` from `validate_subset`;
    - `mean_subset_fails_a_graze_manifest_row_count_drift`;
    - `the_full_report_lists_graze_windows_on_lunar_lines`, `#[ignore = "slow: full lunar-point scan"]`: `validate_scoped(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, Scope::LunarPoints, Some((LUNAR_POINT_GRAZES_CSV, LUNAR_POINT_GRAZES_MANIFEST)))` passes, and every pair line contains `"graze windows "`. It is slow, so the controller runs it in Task 4.

- [ ] **Step 7: Run the fast tests.** Run `cargo test -p pleiades-validate --lib aspects_validation`, expecting every non-ignored test to pass, then `cargo clippy -p pleiades-validate --all-targets -- -D warnings`.

- [ ] **Step 8: Commit.**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/aspects_validation.rs crates/pleiades-validate/src/aspects_validation crates/pleiades-validate/src/aspects_thresholds.rs
git commit -m "feat(validate): graze windows in the lunar-point aspect gate; TrueApogee, MeanNode and MeanApogee pairs (#168)"
```

---

### Task 4 (controller-run measurement, then implementer): Ceilings from the measured run

**Files:**
- Modify: `crates/pleiades-validate/src/aspects_thresholds.rs`

- [ ] **Step 1 (controller, background):** first run `cargo run -q --release -p pleiades-validate -- validate-aspects > /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/lp-gate-1.log 2>&1`, timed with `/usr/bin/time -v` or `date` stamps, recording `/proc/loadavg` before and after. Then run the slow test from Task 3, Step 6. Both must pass with the provisional ceilings.

- [ ] **Step 2: Check the mean-point margin.** From the pair lines, the largest `max sep` over the 11 MeanNode and MeanApogee pairs must be at most 1″.
  - If it is above 1″, stop. The margin for that point becomes 3.3 × the measured value: return to Task 1, Step 1 (the margin) and Task 2.
  - Also stop on a TrueApogee pair whose `max lon` outside windows exceeds 10″.

- [ ] **Step 3: Set the ceilings.** For each of the 15 new pairs, apply ceil(×1.4 of the measured outside-window max sep and max lon), rounded up to two significant figures, floored at 2.3″.
  - Replace each `PROVISIONAL` arm with the final values and a `// measured max <sep>", <lon>" (geo; SWIEPH sepl/semo, 2026-10-17)` comment, in the style of the TrueNode arms.
  - Extend the module's top comment paragraph on the lunar points with the osculating apogee and the mean points, and the window rule: inside-window events are reported, not ceiling-checked.

- [ ] **Step 4: Re-run the full gate (controller).** Expected: pass. Record the summary line and every lunar pair line in the task report.

- [ ] **Step 5: Commit.**

```bash
cargo fmt --all
git add crates/pleiades-validate/src/aspects_thresholds.rs
git commit -m "test(validate): measured ceilings for the TrueApogee, MeanNode and MeanApogee aspect pairs (#168)"
```

---

### Task 5: Docs, runtime, and the conditional split

**Files:**
- Modify:
  - `mise.toml` (the `gate-aspects` comment);
  - `AGENTS.md` (the CI-tier paragraph);
  - `docs/development.md`, `docs/status.md`;
  - `crates/pleiades-events/README.md`;
  - `crates/pleiades-validate/src/render/cli.rs` (the `validate-aspects` help text);
  - `.github/aspects-gate-failure-issue.md`;
  - `crates/pleiades-validate/src/aspects_validation.rs` (the `validate_aspects_corpus` doc);
  - `tools/se-aspects-reference/README.md`, if it lists modes.

- [ ] **Step 1: The runtime decision.**
  - Ratio: the true-node pass took local 18m56s (at load 13–15.6) against nightly 5m58s, so the ratio is r ≈ 3.2.
  - If (Task 4's local full-gate time) / r > 15 min, add a `validate-aspects-lunar` subcommand that runs only `Scope::LunarPoints`, and remove the lunar pass from `validate_aspects_corpus`.
  - In that case also add an `aspects-lunar-gate` job to `.github/workflows/nightly.yml`, copied from `aspects-gate` with `mise run gate-aspects-lunar`, add a `gate-aspects-lunar` task in `mise.toml`, and add it to `release-gate`'s `depends`.
  - Otherwise make no structural change.
  - Write the decision and the numbers in the task report.

- [ ] **Step 2: Update the docs.** Wherever the gated lunar pairs are listed, name TrueNode with seven planets, TrueApogee with four, MeanNode with seven and MeanApogee with four. State the graze-window rule in one sentence: within a window, events may exist in one ephemeris only, and are compared by count parity.
  - Replace "not yet measured on it since" and similar text with the nightly figure: "5m58s on the nightly runner, 2026-10-16, run 38971941852, before the apogee and mean points".
  - Add the new local figure from Task 4.
  - `crates/pleiades-events/README.md`: the fictitious bodies and the mean and osculating perigees are the remaining ungated aspect bodies.
  - Update the `validate_aspects_corpus` rustdoc floor list and timings.
  - Run `grep -rn "true node with seven\|true-node pairs\|TrueNode with seven" AGENTS.md docs mise.toml crates .github` and fix each hit.

- [ ] **Step 3: Verify.** Run `mise run docs`, `mise run audit` and `mise run release-smoke`. The controller runs `mise run test-full` in the background.

- [ ] **Step 4: Commit.**

```bash
git add -A
git commit -m "docs: lunar-point aspect gate covers the osculating apogee and mean points, graze windows, nightly runtime (#168)"
```
