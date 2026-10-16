# True-Node Aspect Corpus Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gate exact aspects of the true node against seven planets, event for event, against a Swiss Ephemeris SWIEPH reference corpus (`lunar-points.csv`), with 2.3″ ceilings, a 3″ graze margin, and a blocking-tier manifest check.

**Architecture:** `tools/se-aspects-reference` gains a `--lunar-points` mode and a per-call graze margin. `pleiades-validate`'s `aspects_validation.rs` gains a third corpus (`Scope::LunarPoints`, `LUNAR_POINT_PAIRS`), merged into the full gate and manifest-checked in the subset, exactly as the asteroid corpus is.

**Tech Stack:** Rust; Swiss Ephemeris through `libswisseph-sys` 0.1.2 (the tool only); the pleiades packaged backend.

**Spec:** `docs/superpowers/specs/2026-10-16-true-node-aspects-design.md`

## Global Constraints

- **Pairs, in this order:** TrueNode with Sun, Mars, Jupiter, Saturn, Uranus, Neptune and Pluto; group `geo`; span `FULL_SPAN` (JD 2415025.5–2488064.5); angles 0/60/90/120/180; tool grid 0.005 d.
- **SE:** body id 11 (`SE_TRUE_NODE`), written as `TrueNode`. Flags `GEO_SWIEPH`. Files verified with the existing `pins::verify_swieph_files`.
- **Graze margin:** planet and asteroid modes 30″, with output byte-identical to the committed corpora. Lunar-point mode 3″.
- **Files:** `crates/pleiades-validate/data/aspects-corpus/lunar-points.csv`, and `lunar-points-manifest.txt` containing `slice aspects-lunar-points file=lunar-points.csv role=aspects rows=N checksum=<fnv1a64>`.
- **Expected per-pair event counts** (spike, 2026-10-16): Sun 1686, Mars 990, Jupiter 247, Saturn 166, Uranus 119, Neptune 109, Pluto 107; total **3424**. A different regenerated count is investigated, never accepted silently.
- **Ceilings:** every `TrueNode-<planet>` pair gets sep 2.3″ and lon 2.3″.
- **`MIN_ROWS_VALIDATED_LUNAR_POINTS`:** 3424.
- **No changes to:** `devenv.nix`, the compatibility profile, or any engine crate.
- **Pinned SWIEPH files:** already downloaded and SHA-verified at `/tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe` (pass `--ephe <dir>` or `SE_EPHE_PATH`).
- **Building the SE tool:** `devenv shell` is broken in this sandbox (nix is not on PATH). Build with `env CFLAGS=-std=gnu17 LIBCLANG_PATH=/nix/store/jdgw7h0g0l8clmcasaspxnx6v62jz1il-clang-21.1.8-lib/lib cargo build --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`. The tool's `Cargo.toml` has its own `[workspace]`. If the build still fails, look at the spike's working setup in `/tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/lunar-spike/se-probe/`.
- **Tests:** validate with libtest, `cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored`, never nextest.
- **Commits:** `cargo fmt --all` before each commit; clippy `-D warnings`.

## Review Focus

1. **The graze-margin refactor must not change existing corpora.** Regenerating the planet and asteroid corpora must reproduce the committed checksums (`4506302360407709888`, `8132397916751651688`). Pinned in Task 1, Step 5.
2. **A hand-edited `lunar-points.csv` or a stale manifest** must fail the blocking tier (`validate_aspects_corpus_subset`), not only nightly. Pinned in Task 2's subset tests.
3. **`Uranus` and `Neptune` are not in `body_from_name` today** (no existing corpus uses them). Without them, the gate errors "unknown body". Pinned in Task 2 (`lunar_point_corpus_parses_into_the_seven_true_node_pairs`).
4. **The near-graze events** (TrueNode–Jupiter turn 3.6″ from a level at JD ≈ 2431893.25; TrueNode–Pluto 4.1″ at JD ≈ 2486274.68) must be present in the corpus, so the gate actually compares them. Pinned in Task 1, Step 6, by checking the tool's stderr closest-turn lines.
5. **The full gate's runtime** grows by the lunar pass. Measure it, and record it in the `mise.toml` comment (Task 3), so a nightly-timeout surprise is visible.

---

### Task 1: The reference tool's lunar-point mode, and the corpus

**Files:**
- Modify: `tools/se-aspects-reference/src/main.rs`
- Create: `crates/pleiades-validate/data/aspects-corpus/lunar-points.csv`, `lunar-points-manifest.txt` (generated)

- [ ] **Step 1: Make the graze margin a parameter of `scan`.** Add `graze_margin_deg: f64` as the last parameter of `fn scan(...)`, and use it in place of `GRAZE_MARGIN_DEG` in the `assert!`. Keep the panic message, but print the margin: `"graze: {group},{},{} turns {:.2} arcsec from the {angle} degree level at jd_tt={:.5}, inside the {:.0} arcsec margin; an event here would depend on the ephemeris"`. Every existing call passes `GRAZE_MARGIN_DEG`. Rename the constant to `PLANET_GRAZE_MARGIN_DEG` and update its doc comment ("30 arcseconds: several times the largest difference between the Swiss Ephemeris (Moshier or SWIEPH seas_18) and pleiades separations of the planet and asteroid pairs").

- [ ] **Step 2: Add the constants and pair table** next to `ASTEROID_PAIRS`:

```rust
const URANUS: Body = (7, "Uranus");
const NEPTUNE: Body = (8, "Neptune");
/// SE_TRUE_NODE: the osculating (true) lunar ascending node.
const TRUE_NODE: Body = (11, "TrueNode");
/// The lunar-point grid: the true node's speed oscillates with a period of
/// about two weeks and touches zero, so its separation from a slow planet
/// turns thousands of times over the span (2392 for Jupiter, 6600 for Pluto).
const LUNAR_POINT_GRID_DAYS: f64 = 0.005;
/// 3 arcseconds: 3.6 times the largest engine-minus-SE SWIEPH separation at a
/// turning point measured for these pairs (0.82"), and below the closest turn
/// to a level they have (TrueNode-Jupiter, 3.6"), so a measured near-graze is
/// compared rather than refused (issue #168 (c)).
const LUNAR_POINT_GRAZE_MARGIN_DEG: f64 = 3.0 / 3600.0;
/// The lunar-point mode's pairs, all `geo` with `GEO_SWIEPH` over the full span.
const LUNAR_POINT_PAIRS: [(Body, Body); 7] = [
    (TRUE_NODE, SUN),
    (TRUE_NODE, MARS),
    (TRUE_NODE, JUPITER),
    (TRUE_NODE, SATURN),
    (TRUE_NODE, URANUS),
    (TRUE_NODE, NEPTUNE),
    (TRUE_NODE, PLUTO),
];
```

- [ ] **Step 3: Share the SWIEPH setup, and add `lunar_points_main`.** Extract the `--out`/`--ephe`/verify/`swe_set_ephe_path` block of `asteroids_main` into `fn swieph_setup(args: &[String], mode: &str) -> String` (it returns `out_dir`). Then add:

```rust
fn lunar_points_main(args: &[String]) {
    let out_dir = swieph_setup(args, "--lunar-points");
    let mut csv = String::new();
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with sepl_18/semo_18 (SHA-256 pinned in tools/se-aspects-reference/src/pins.rs).\n");
    csv.push_str("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)\n");
    csv.push_str("# equals +angle or -angle, scanned on a 0.005-day grid, each step split at the zero\n");
    csv.push_str("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# geo: apparent, tropical, true equinox of date (default flags) with SEFLG_SWIEPH; TrueNode (SE_TRUE_NODE) with Sun, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, JD 2415025.5-2488064.5.\n");
    csv.push_str("# Angles 0, 60, 90, 120, 180.\n");
    csv.push_str("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.\n");
    csv.push_str("# No turning point of any pair's separation is within 3 arcsec of one of its levels (the tool fails otherwise).\n");
    csv.push_str("group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n");
    let mut rows = 0usize;
    {
        let mut push = |row: String| {
            rows += 1;
            writeln!(csv, "{row}").unwrap();
        };
        for (first, second) in LUNAR_POINT_PAIRS {
            scan(
                &mut push,
                "geo",
                first,
                second,
                GEO_SWIEPH,
                FULL_SPAN,
                LUNAR_POINT_GRID_DAYS,
                LUNAR_POINT_GRAZE_MARGIN_DEG,
            );
        }
    }
    let manifest = format!(
        "slice aspects-lunar-points file=lunar-points.csv role=aspects rows={rows} checksum={}\n",
        pins::fnv1a64(&csv)
    );
    // Write both files exactly as asteroids_main does, with the names
    // lunar-points.csv and lunar-points-manifest.txt.
}
```

Finish the write block by mirroring `asteroids_main`'s. In `main()`, dispatch `--lunar-points` beside `--asteroids`. Update the module doc comment with a "Lunar-point mode" paragraph, modelled on the asteroid one: the pairs, the 0.005 d grid, the 3″ margin and why, the build command (both the devenv form and the `LIBCLANG_PATH` form), and the run command.

- [ ] **Step 4: Build.**

Run: `env CFLAGS=-std=gnu17 LIBCLANG_PATH=/nix/store/jdgw7h0g0l8clmcasaspxnx6v62jz1il-clang-21.1.8-lib/lib cargo build --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`
Expected: builds. Run `cargo fmt --manifest-path tools/se-aspects-reference/Cargo.toml` too.

- [ ] **Step 5: Prove the refactor changed nothing for the existing corpora.** Regenerate both into the scratchpad, never into the repo:
  - `tools/se-aspects-reference/target/release/se-aspects-reference > <scratch>/aspects.csv` (planet mode, Moshier; the stdout has no devenv banner because devenv is not used);
  - `tools/se-aspects-reference/target/release/se-aspects-reference --asteroids --out <scratch>/ast --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe`.

  Then `cmp <scratch>/aspects.csv crates/pleiades-validate/data/aspects-corpus/aspects.csv` and `cmp <scratch>/ast/asteroids-manifest.txt crates/pleiades-validate/data/aspects-corpus/asteroids-manifest.txt`.
  Expected: identical. If the planet CSV differs only by a leading banner line, say so. Any other difference is a STOP: report it.

- [ ] **Step 6: Generate the lunar-point corpus into the repo.**

Run: `tools/se-aspects-reference/target/release/se-aspects-reference --lunar-points --out crates/pleiades-validate/data/aspects-corpus --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe 2> <scratch>/lunar-stderr.txt`
Expected: it writes 3424 rows. Check the per-pair event counts on stderr against the Global Constraints counts. Check that the stderr closest-turn lines show TrueNode–Jupiter about 3.6″ and TrueNode–Pluto about 4.1″. Paste the stderr summary into the report. On any count difference, STOP and report.

- [ ] **Step 7: Commit.**

```bash
git add tools/se-aspects-reference/src/main.rs crates/pleiades-validate/data/aspects-corpus/lunar-points.csv crates/pleiades-validate/data/aspects-corpus/lunar-points-manifest.txt
git commit -m "feat(tools): se-aspects-reference --lunar-points: a SWIEPH true-node aspect corpus (#168)"
```

(Do not commit `tools/se-aspects-reference/target/` or any ephemeris file.)

---

### Task 2: Gate the lunar-point corpus

**Files:**
- Modify: `crates/pleiades-validate/src/aspects_validation.rs`
- Modify: `crates/pleiades-validate/src/aspects_thresholds.rs`
- Test: `crates/pleiades-validate/src/aspects_validation/tests.rs`

**Interfaces:**
- Consumes: the files from Task 1.
- Produces: `Scope::LunarPoints`, `LUNAR_POINT_PAIRS: [Pair; 7]`, `LUNAR_POINT_CSV`, `LUNAR_POINT_MANIFEST`, `MIN_ROWS_VALIDATED_LUNAR_POINTS`.

- [ ] **Step 1: Write the failing tests**, mirroring the asteroid ones (`tests.rs` 479–625). The `#[ignore]`/slow placement of each must match its asteroid twin: a test that scans with the engine is slow if the asteroid twin is.
  - `lunar_point_corpus_parses_into_the_seven_true_node_pairs`: `parse_corpus(LUNAR_POINT_CSV, &LUNAR_POINT_PAIRS)` succeeds; the pair names are exactly `TrueNode-Sun`, `TrueNode-Mars`, `TrueNode-Jupiter`, `TrueNode-Saturn`, `TrueNode-Uranus`, `TrueNode-Neptune`, `TrueNode-Pluto`; the per-pair row counts are 1686, 990, 247, 166, 119, 109, 107; every pair has a `ceilings_for` entry; `body_from_name` resolves every name.
  - `committed_lunar_point_corpus_matches_its_manifest`: `check_manifest(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, &LUNAR_POINT_PAIRS)` gives `MIN_ROWS_VALIDATED_LUNAR_POINTS` rows.
  - `lunar_point_corpus_tampering_fails_closed`: as `asteroid_corpus_tampering_fails_closed`, with `geo,TrueNode,Sun,` and the `aspects-lunar-points` manifest line, through `validate_scoped(.., Scope::LunarPoints)`.
  - `mean_subset_fails_a_tampered_lunar_point_corpus` and `mean_subset_fails_a_lunar_point_manifest_row_count_drift`: the subset must fail with `ChecksumMismatch` / `ManifestDrift` when only the lunar corpus is wrong.
  - `a_missing_lunar_point_aspect_fails_the_count`: as its asteroid twin, removing the first `geo,TrueNode,Jupiter,0,` row.
  - `lunar_point_scope_floor_is_its_own`: as the asteroid twin.
  - The existing total-count test (around line 384–388) becomes `PAIRS.len() + ASTEROID_PAIRS.len() + LUNAR_POINT_PAIRS.len()`, with the rows floor summed with `MIN_ROWS_VALIDATED_LUNAR_POINTS`.

- [ ] **Step 2: Run them to verify they fail to compile.**

Run: `cargo test -p pleiades-validate --lib aspects_validation`
Expected: compile errors (`LUNAR_POINT_CSV` not found).

- [ ] **Step 3: Implement.**
  - In `aspects_validation.rs`:
    - `LUNAR_POINT_CSV` and `LUNAR_POINT_MANIFEST` `include_str!` consts beside the asteroid ones.
    - `LUNAR_POINT_PAIRS: [Pair; 7]` (all `pair(Group::Geo, "TrueNode", "<planet>", FULL_SPAN)`, in the Global Constraints order), with a doc comment mirroring `ASTEROID_PAIRS`'s.
    - `"Uranus" => CelestialBody::Uranus`, `"Neptune" => CelestialBody::Neptune` and `"TrueNode" => CelestialBody::TrueNode` in `body_from_name`.
    - `Scope::LunarPoints` (doc: "The true-node corpus (`lunar-points.csv`, Swiss Ephemeris SWIEPH), the [`LUNAR_POINT_PAIRS`]. Part of the full gate only (issue #168)."), with `includes` → `true`, `floor` → `MIN_ROWS_VALIDATED_LUNAR_POINTS`, `title` → `"True-node aspects"`, `pairs` → `&LUNAR_POINT_PAIRS`.
    - `validate_aspects_corpus()` merges `validate_scoped(LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST, Scope::LunarPoints)?` after the asteroids.
    - `validate_subset` gains `lunar_csv: &str, lunar_manifest: &str` parameters and a `check_manifest(lunar_csv, lunar_manifest, &LUNAR_POINT_PAIRS)?` after the asteroid check. `validate_aspects_corpus_subset` passes the committed consts. Update the existing asteroid subset tests' calls (`validate_subset(&tampered, ASTEROID_MANIFEST)` → `validate_subset(&tampered, ASTEROID_MANIFEST, LUNAR_POINT_CSV, LUNAR_POINT_MANIFEST)`).
    - Update the docs of `validate_aspects_corpus`, `validate_aspects_corpus_subset` and the module header to name the third corpus: true node with seven planets, SWIEPH, 3424 events, the 3″ graze margin, and the measured near-grazes of 3.6″ and 4.1″.
  - In `aspects_thresholds.rs`:
    - `pub(crate) const MIN_ROWS_VALIDATED_LUNAR_POINTS: usize = 3_424;`, with a doc comment ("every row of `lunar-points.csv` (3424, generated 2026-10-16)").
    - Seven `ceilings_for` arms, all `sep_arcsec: 2.3, lon_arcsec: 2.3`, under a comment block. Each arm has a line with its measured maxima from the spike, separation then longitude at the event, then "(geo; SWIEPH sepl/semo corpus, 2026-10-16)":

| Pair | Separation | Longitude |
|---|---|---|
| Sun | 0.530″ | 0.783″ |
| Mars | 0.861″ | 0.761″ |
| Jupiter | 0.602″ | 0.767″ |
| Saturn | 0.641″ | 0.775″ |
| Uranus | 0.513″ | 1.073″ |
| Neptune | 0.719″ | 0.763″ |
| Pluto | 0.528″ | 0.734″ |

    - The block comment: "True node (Swiss Ephemeris SWIEPH sepl_18/semo_18 corpus): measured × 1.4, floored at 2.3" as the asteroids; the point itself agrees within 0.94" over 1900-2100."
    - Task 3 replaces these spike figures with the full gate's own measured maxima if they differ.

- [ ] **Step 4: Run the fast tests and the full gate.**

Run: `cargo fmt --all && cargo test -p pleiades-validate --lib aspects_validation` (the fast ones pass).
Then check `cat /proc/loadavg`, and run: `time mise run gate-aspects 2>&1 | tail -5`
Expected: PASS. A "True-node aspects: 3424 exact aspects validated across 7 pairs …" segment appears, with max separation and longitude under 2.3″. Record the wall-clock time and load.

- [ ] **Step 5: Run the full libtest and clippy.**

Run: `cargo test -p pleiades-validate --lib --bins --tests -- --include-ignored` and `cargo clippy -p pleiades-validate --all-targets --all-features -- -D warnings`
Expected: pass and clean.

- [ ] **Step 6: Commit.**

```bash
git add crates/pleiades-validate/src/aspects_validation.rs crates/pleiades-validate/src/aspects_thresholds.rs crates/pleiades-validate/src/aspects_validation/tests.rs
git commit -m "feat(validate): gate true-node aspects against a Swiss Ephemeris SWIEPH corpus (#168)"
```

---

### Task 3: Ceilings from the gate's own measurement, docs and timing

**Files:**
- Modify: `crates/pleiades-validate/src/aspects_thresholds.rs` (only if the measured maxima differ from the spike)
- Modify: `mise.toml` (the `[tasks.gate-aspects]` comment), `crates/pleiades-events/README.md`, `docs/status.md`, `docs/development.md`, `AGENTS.md` (wherever they list the gate's pairs)

- [ ] **Step 1: Per-pair maxima.** Read the gate's per-pair lines for the seven TrueNode pairs. The full gate prints `pair_lines` through the `validate-aspects` command, or use the gate's measuring test if one exists for asteroids (look for `measure` in `aspects_validation/tests.rs`). If any pair's measured separation or longitude maximum × 1.4 exceeds 2.3″, set that ceiling to the measured value × 1.4, rounded up to two significant figures, and say so. Otherwise keep 2.3″, and replace the spike figures in the comments with the gate's figures.

- [ ] **Step 2: Update the docs.**
  - **`crates/pleiades-events/README.md`:** replace "Aspects of the lunar points and fictitious bodies are found but not gated." with "Aspects of the true node are gated against Swiss Ephemeris SWIEPH for seven planets (Sun, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto, 1900–2100), including turns 3.6″ and 4.1″ from an angle; the other lunar points and the fictitious bodies are found but not gated."
  - **The `mise.toml` `[tasks.gate-aspects]` comment:** add "and TrueNode with seven planets (SWIEPH, issue #168)" to its pair list, and one wrapped line with the measured before/after wall-clock and load from Task 2, Step 4, labelled local.
  - **`docs/status.md`, `docs/development.md` and the CI-tier paragraph of `AGENTS.md`:** wherever these list the full gate's pairs ("planets plus Sun–Ceres..Vesta, Moon–Ceres, Mars–Vesta and Jupiter–Ceres"), append "and the true node with seven planets". Check each with `grep -n "Jupiter–Ceres\|Jupiter-Ceres"`.

- [ ] **Step 3: Check.**

Run: `cargo fmt --all --check && mise run docs && mise run audit`
Expected: clean.

- [ ] **Step 4: Commit.**

```bash
git add mise.toml crates/pleiades-events/README.md docs/status.md docs/development.md AGENTS.md crates/pleiades-validate/src/aspects_thresholds.rs
git commit -m "docs: the aspect gate covers the true node against seven planets (#168)"
```

(Drop from `git add` any path that wasn't changed.)

---

### Task 4: Whole-branch verification

- [ ] **Step 1:** `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && mise run docs`
- [ ] **Step 2:** `mise run test-full`. Make no edits or commits while it runs.
- [ ] **Step 3:** `mise run audit && mise run release-smoke && mise run package-check`. `release-smoke` runs the subset, including the new manifest check.
- [ ] **Step 4:** If anything fails, fix it in the owning task's files, commit, and repeat from Step 1.
