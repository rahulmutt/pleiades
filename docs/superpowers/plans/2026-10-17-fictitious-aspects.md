# Fictitious-Body Aspects Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Gate exact aspects of the 19 fictitious bodies against Swiss Ephemeris SWIEPH in the mean-of-date frame: each body with the Sun, and the Uranian bodies plus Transpluto with Saturn, 28 pairs. Near-tangent turns use #272's graze windows.

**Architecture:**
- **Tool:** `tools/se-aspects-reference` gets a `--fictitious` mode: `mean` group, `MEAN_SWIEPH` flags, a pinned `seorbel.txt`, and `Graze::Window` with a 3″ margin.
- **Gate:** `pleiades-validate` gets `Scope::Fictitious`. Its engine is `RoutingBackend[packaged, FictitiousBackend(packaged)]`; the comparison loop moves into a function generic over the backend. The pass is merged into `validate_aspects_corpus` (the `aspects-gate` job) and also has a standalone subcommand.
- **Where it runs:** a nightly run dispatched on the branch decides whether it stays in that job or gets its own.

**Tech Stack:** Rust; Swiss Ephemeris via `libswisseph-sys` 0.1.2 (the tool only); `pleiades-data`, `pleiades-fict`, `pleiades-events`.

**Spec:** `docs/superpowers/specs/2026-10-17-fictitious-aspects-design.md`

## Global Constraints

- **Pairs, in this order (28), all group `mean`, span `FULL_SPAN` (JD 2415025.5–2488064.5), angles 0/60/90/120/180:**
  - Sun with each of Cupido, Hades, Zeus, Kronos, Apollon, Admetos, Vulkanus, Poseidon, Transpluto, Nibiru, Harrington, NeptuneLeverrier, NeptuneAdams, PlutoLowell, PlutoPickering, Vulcan, WhiteMoon, Proserpina, Waldemath, with the fictitious body first: `(Cupido, Sun)` … `(Waldemath, Sun)`;
  - then `(Cupido, Saturn)`, `(Hades, Saturn)`, `(Zeus, Saturn)`, `(Kronos, Saturn)`, `(Apollon, Saturn)`, `(Admetos, Saturn)`, `(Vulkanus, Saturn)`, `(Poseidon, Saturn)`, `(Transpluto, Saturn)`.
- **SE body numbers:** Cupido 40, Hades 41, Zeus 42, Kronos 43, Apollon 44, Admetos 45, Vulkanus 46, Poseidon 47, Transpluto 48, Nibiru 49, Harrington 50, NeptuneLeverrier 51, NeptuneAdams 52, PlutoLowell 53, PlutoPickering 54, Vulcan 55, WhiteMoon 56, Proserpina 57, Waldemath 58. The CSV names are exactly the engine's `CelestialBody` variant names.
- **Flags:** `MEAN_SWIEPH = (MEAN & !SEFLG_MOSEPH) | SEFLG_SWIEPH`, i.e. `SWIEPH|SPEED|TRUEPOS|NOABERR|NOGDEFL|NONUT` = 1874.
- **Grid:** 0.05 d for every pair except `(Vulcan, Sun)` at 0.01 d.
- **Graze margin:** 3″ for every fictitious pair, with `Graze::Window`.
- **Files:**
  - `crates/pleiades-validate/data/aspects-corpus/fictitious.csv` with `fictitious-manifest.txt`: `slice aspects-fictitious file=fictitious.csv role=aspects rows=N checksum=<fnv1a64>`;
  - `fictitious-grazes.csv` with `fictitious-grazes-manifest.txt`: `slice aspects-fictitious-grazes file=fictitious-grazes.csv role=grazes rows=N checksum=<fnv1a64>`.
  The grazes schema is #272's.
- **`seorbel.txt` pin:** SHA-256 `97b454ff78f4f4716b5cc987a93ca8f33e44ef4b524a165a155a8a4885fd2e18`, of `tools/se-fictitious-reference/data/seorbel.txt`.
- **Unchanged outputs:** `aspects.csv` (checksum 4506302360407709888), `asteroids.csv` (8132397916751651688) and `lunar-points.csv` (17431195134162698173) must regenerate byte-identically.
- **Gate engine for `Scope::Fictitious`:** `RoutingBackend::new(vec![Box::new(PackagedDataBackend::new()), Box::new(FictitiousBackend::new(PackagedDataBackend::new()))])`. Every other scope keeps `EventEngine::new(packaged_backend())`, with identical results.
- **Ceilings:** ×1.4 of the measured outside-window maximum, rounded up to two significant figures, floored at 2.3″. Stop on any pair above 10″.
- **Floor:** `MIN_ROWS_VALIDATED_FICTITIOUS` = the corpus rows outside windows.
- **Runtime rule:** a nightly run dispatched on the branch decides. If its `aspects-gate` step (planets, asteroids and fictitious) is under 15 minutes, the pass stays there. Otherwise it gets its own job `aspects-fictitious-gate` and a `release-gate` dependency, as the lunar job did in #272.
- **No changes to:** `devenv.nix`, the compatibility profile, or any engine crate.
- **SE build:** `env CFLAGS=-std=gnu17 LIBCLANG_PATH=/nix/store/jdgw7h0g0l8clmcasaspxnx6v62jz1il-clang-21.1.8-lib/lib cargo build --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`.
- **Ephemeris directory for runs:** `/tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/fict-spike/ephe`, which holds `sepl_18`, `semo_18` and `seorbel.txt`. The asteroid regeneration needs `/tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe`, which holds `seas_18`.
- **Tests:** validate with libtest, never nextest. Every subagent command runs in the foreground for under 10 minutes; longer runs are the controller's.
- **Commits:** run `cargo fmt --all` before each commit, and clippy with `-D warnings`.

## Review Focus

1. **The other scopes' engine must not change.** Making the backend scope-dependent must leave planets, asteroids and lunar points on the same `EventEngine<PackagedDataBackend>`. Pinned by Task 3's unchanged existing tests, and by the full gates in Task 4.
2. **A wrong or missing `seorbel.txt`** must abort the tool before any row is written. Otherwise SE silently falls back to its built-in elements. Pinned in Task 1, Step 1 (`verify_fictitious_files`) and Step 5.
3. **Vulcan–Sun's thousands of turns** must keep the 0.01 d grid, and halving each grid must leave every count unchanged. Pinned in Task 2, Step 2.
4. **A tampered fictitious corpus or grazes file** must fail the blocking tier. Pinned in Task 3, Step 5.
5. **Nibiru's steady +1.26″ offset** must not be absorbed by a loose ceiling on the other pairs. Ceilings are per pair. Pinned in Task 4, Step 2.

---

### Task 1: The tool's `--fictitious` mode

**Files:**
- Modify: `tools/se-aspects-reference/src/main.rs`, `tools/se-aspects-reference/src/pins.rs`

**Interfaces:**
- Produces: `--fictitious --out <dir> [--ephe <dir>] [--grid-divisor <n>]`, which writes the four files named in Global Constraints. `--grid-divisor` (default 1) divides every fictitious grid, and is used for Task 2's convergence check.

- [ ] **Step 1: Pin `seorbel.txt`.** In `pins.rs`, next to `PINNED_FILES`:

```rust
/// The fictitious-body element file Swiss Ephemeris reads from the ephemeris
/// path, pinned to the committed copy in tools/se-fictitious-reference/data/.
/// Without it SE falls back to built-in elements silently, so the
/// --fictitious mode refuses to run.
pub const PINNED_SEORBEL: (&str, &str) = (
    "seorbel.txt",
    "97b454ff78f4f4716b5cc987a93ca8f33e44ef4b524a165a155a8a4885fd2e18",
);

/// [`verify_swieph_files`]'s planet and Moon files plus `seorbel.txt`.
/// `seas_18` is not needed for fictitious bodies, so it is not checked here.
pub fn verify_fictitious_files(ephe_dir: &str) -> Result<(), String> {
    for (name, want) in PINNED_FILES.iter().filter(|(n, _)| *n != "seas_18.se1").chain([&PINNED_SEORBEL]) {
        let path = format!("{ephe_dir}/{name}");
        let bytes = std::fs::read(&path).map_err(|e| {
            if *name == "seorbel.txt" {
                format!("cannot read {path}: {e}\nCopy it: cp tools/se-fictitious-reference/data/seorbel.txt {ephe_dir}/")
            } else {
                format!(
                    "cannot read {path}: {e}\nDownload: curl -fLo {path} \
                     https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/{name}"
                )
            }
        })?;
        let got = sha256_hex(&bytes);
        if got != *want {
            return Err(format!("SHA-256 mismatch for {path}: got {got}, pinned {want}"));
        }
    }
    Ok(())
}
```

Add a unit test beside the existing pins tests. Use a temp dir under `std::env::temp_dir()`, holding a `seorbel.txt` with the wrong content and no `.se1` files. `verify_fictitious_files` must return an `Err` that names a file. Read how the existing pins tests make files, and copy that pattern.

- [ ] **Step 2: Constants, bodies and pairs** in `main.rs`, beside the lunar ones:

```rust
/// The planet `mean` group's flags on the SWIEPH files: geometric (no
/// light-time), no aberration, no deflection, mean equinox of date.
/// Fictitious bodies are gated in this frame because Swiss Ephemeris's
/// light-time correction for them jumps by up to about 5" at isolated
/// instants in the apparent frame (spike 2026-10-17, issue #168).
const MEAN_SWIEPH: c_int = (MEAN & !SEFLG_MOSEPH) | SEFLG_SWIEPH;

const CUPIDO: Body = (40, "Cupido");
const HADES: Body = (41, "Hades");
const ZEUS: Body = (42, "Zeus");
const KRONOS: Body = (43, "Kronos");
const APOLLON: Body = (44, "Apollon");
const ADMETOS: Body = (45, "Admetos");
const VULKANUS: Body = (46, "Vulkanus");
const POSEIDON: Body = (47, "Poseidon");
const TRANSPLUTO: Body = (48, "Transpluto");
const NIBIRU: Body = (49, "Nibiru");
const HARRINGTON: Body = (50, "Harrington");
const NEPTUNE_LEVERRIER: Body = (51, "NeptuneLeverrier");
const NEPTUNE_ADAMS: Body = (52, "NeptuneAdams");
const PLUTO_LOWELL: Body = (53, "PlutoLowell");
const PLUTO_PICKERING: Body = (54, "PlutoPickering");
const VULCAN: Body = (55, "Vulcan");
const WHITE_MOON: Body = (56, "WhiteMoon");
const PROSERPINA: Body = (57, "Proserpina");
const WALDEMATH: Body = (58, "Waldemath");

/// 3 arcseconds: at least 1.9 times the largest engine-minus-SE difference of
/// any fictitious body in the mean frame (Nibiru, 1.57", a steady +1.26"
/// offset) and 3.9 times every other body's (0.76").
const FICTITIOUS_GRAZE_MARGIN_DEG: f64 = 3.0 / 3600.0;
/// Vulcan circles inside Mercury's orbit, so its separation from the Sun
/// turns about 37 times a year (7460 turns over the span); the other pairs
/// use the planet grid.
const VULCAN_SUN_GRID_DAYS: f64 = 0.01;

/// The fictitious mode's pairs, all `mean` with `MEAN_SWIEPH` over the full
/// span: every body with the Sun, then the Uranian bodies and Transpluto with
/// Saturn, whose separations turn 386 times.
const FICTITIOUS_PAIRS: [(Body, Body, f64); 28] = [
    (CUPIDO, SUN, GRID_DAYS),
    (HADES, SUN, GRID_DAYS),
    (ZEUS, SUN, GRID_DAYS),
    (KRONOS, SUN, GRID_DAYS),
    (APOLLON, SUN, GRID_DAYS),
    (ADMETOS, SUN, GRID_DAYS),
    (VULKANUS, SUN, GRID_DAYS),
    (POSEIDON, SUN, GRID_DAYS),
    (TRANSPLUTO, SUN, GRID_DAYS),
    (NIBIRU, SUN, GRID_DAYS),
    (HARRINGTON, SUN, GRID_DAYS),
    (NEPTUNE_LEVERRIER, SUN, GRID_DAYS),
    (NEPTUNE_ADAMS, SUN, GRID_DAYS),
    (PLUTO_LOWELL, SUN, GRID_DAYS),
    (PLUTO_PICKERING, SUN, GRID_DAYS),
    (VULCAN, SUN, VULCAN_SUN_GRID_DAYS),
    (WHITE_MOON, SUN, GRID_DAYS),
    (PROSERPINA, SUN, GRID_DAYS),
    (WALDEMATH, SUN, GRID_DAYS),
    (CUPIDO, SATURN, GRID_DAYS),
    (HADES, SATURN, GRID_DAYS),
    (ZEUS, SATURN, GRID_DAYS),
    (KRONOS, SATURN, GRID_DAYS),
    (APOLLON, SATURN, GRID_DAYS),
    (ADMETOS, SATURN, GRID_DAYS),
    (VULKANUS, SATURN, GRID_DAYS),
    (POSEIDON, SATURN, GRID_DAYS),
    (TRANSPLUTO, SATURN, GRID_DAYS),
];
```

If `SUN`, `SATURN` or `GRID_DAYS` are named differently in the file, use the existing names. Read the constants block first.

- [ ] **Step 3: `fictitious_main`.** Add a `--fictitious` branch in `main`, after the `--lunar-points` branch, and a `fictitious_main(args)` function modelled line for line on `lunar_points_main`, with these differences:
  - **Setup:** a `fictitious_setup` that parses `--out`/`--ephe` exactly like `swieph_setup` but calls `pins::verify_fictitious_files` instead of `verify_swieph_files`. Factor the shared `--out`/`--ephe` parsing and `swe_set_ephe_path` into one helper used by both, rather than duplicating it.
  - **Grid divisor:** `--grid-divisor <n>`, a positive integer (default 1; any other value panics with a message), divides each pair's grid.
  - **Scan:** loop over `FICTITIOUS_PAIRS`, calling `scan(&mut push, "mean", first, second, MEAN_SWIEPH, FULL_SPAN, grid / divisor as f64, Graze::Window { margin_deg: FICTITIOUS_GRAZE_MARGIN_DEG, out: &mut push_graze })`.
  - **Header** of `fictitious.csv`:

```rust
    csv.push_str("# Source: Swiss Ephemeris 2.10.03 (libswisseph-sys 0.1.2), SEFLG_SWIEPH with sepl_18/semo_18 and seorbel.txt (SHA-256 pinned in tools/se-aspects-reference/src/pins.rs).\n");
    csv.push_str("# A row is an instant at which wrap180(lon(first) - lon(second)) of swe_calc(jd_tt, body, iflag|SEFLG_SPEED)\n");
    csv.push_str("# equals +angle or -angle, scanned on a 0.05-day grid (0.01 day for Vulcan-Sun), each step split at the zero\n");
    csv.push_str("# of the relative longitude speed, and bisected to 1e-7 day. jd_tt is TT.\n");
    csv.push_str("# mean: SEFLG_SWIEPH|SEFLG_TRUEPOS|SEFLG_NOABERR|SEFLG_NOGDEFL|SEFLG_NONUT (geometric, mean equinox of date); the 19 fictitious bodies (SE 40-58) with the Sun, and Cupido..Poseidon and Transpluto with Saturn, JD 2415025.5-2488064.5.\n");
    csv.push_str("# The apparent frame is not used: SE's light-time correction for fictitious bodies jumps by up to about 5 arcsec at isolated instants.\n");
    csv.push_str("# Angles 0, 60, 90, 120, 180.\n");
    csv.push_str("# rel_speed_deg_per_day is the longitude speed of first less that of second at the row's instant.\n");
    csv.push_str("# A turning point within 3 arcsec of a level is a graze window in fictitious-grazes.csv; events inside a window may exist in one ephemeris only.\n");
    csv.push_str("group,first,second,angle_deg,jd_tt,first_lon_deg,second_lon_deg,rel_speed_deg_per_day\n");
```

  - **Grazes header:** the same seven lines as `lunar-points-grazes.csv`'s header, with "lunar-point" replaced by "fictitious", "lunar-points.csv" by "fictitious.csv", the margin text replaced by "3 arcsec for every pair", and "0.005-day grid" replaced by "scan grid".
  - **Manifests:** as in Global Constraints. Log `wrote … (N rows), … (M windows) and their manifests` to stderr.

  If `lunar_points_main` and `fictitious_main` would share more than the setup and the file writing, factor the four-file writer into one helper used by both, and keep the lunar output byte-identical (Task 2 verifies this).

- [ ] **Step 4: Module doc.** Add a "Fictitious mode" paragraph to the `//!` block: flags, pairs, grids, margin, files, the reason for the mean frame, and the run command `… --fictitious --out crates/pleiades-validate/data/aspects-corpus --ephe <dir holding sepl_18, semo_18, seorbel.txt>`.

- [ ] **Step 5: Build, clippy and quick checks**, each in the foreground, under 10 minutes:
  - Build and clippy `-D warnings` with the commands in Global Constraints.
  - `cargo test --release --offline --manifest-path tools/se-aspects-reference/Cargo.toml`, with the same `env` prefix, so the pins tests pass.
  - **Moshier planets:** `target/release/se-aspects-reference > /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/fa-regen/aspects.csv`, then `cmp` it with the committed file.
  - **Asteroids:** `--asteroids --out …/fa-regen --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe`, then `cmp`. If it would exceed 10 minutes, skip it and report.
  - **Missing elements file:** `--fictitious` with `--ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/ephe`, which has no `seorbel.txt`, must exit non-zero with the copy hint and write nothing.
  - **Smoke:** add a throwaway short-span override for the fictitious mode, for example JD 2451545–2455197.5, and remove it before committing. Running `--fictitious` with the fict-spike ephemeris directory must not panic, and must print turn lines for all 28 pairs.

- [ ] **Step 6: Commit.**

```bash
cargo fmt --all
git add tools/se-aspects-reference/src/main.rs tools/se-aspects-reference/src/pins.rs
git commit -m "feat(tools): se-aspects-reference --fictitious, mean-of-date frame with a pinned seorbel.txt and graze windows (#168)"
```

---

### Task 2 (controller): Generate the corpus; check the grid and unchanged outputs

**Files:**
- Create (generated): the four `aspects-corpus/fictitious*` files.

- [ ] **Step 1 (background):** run `--fictitious --out crates/pleiades-validate/data/aspects-corpus --ephe /tmp/claude-1000/-workspace/65962505-e58e-4f50-92f2-32207b566b7e/scratchpad/fict-spike/ephe`, recording the stderr log and wall-clock time.
- [ ] **Step 2: Grid convergence (background).** Re-run with `--grid-divisor 2` into a scratch directory, and compare the per-pair, per-angle event counts with Step 1's (`cut -d, -f1-4 | sort | uniq -c`). Any difference stops the work: the grid for that pair is halved and Step 1 re-run.
- [ ] **Step 3: Unchanged lunar output (background, about 65 minutes).** Regenerate `--lunar-points` into scratch with the lunar ephemeris directory, then `cmp` all four lunar files against the committed ones.
- [ ] **Step 4: Check against the spike.** The Sun-pair counts must be close to the spike's coarse estimates: about 1,593–1,598 for the slow bodies and about 7,460 for Vulcan. The windows logged should include the Poseidon–Saturn near-graze. Record the counts per pair, the windows, and the run time.
- [ ] **Step 5: Commit** the four files: `data(validate): fictitious-body aspect corpus in the mean-of-date frame, with graze windows (#168)`.

---

### Task 3: The gate's `Scope::Fictitious`

**Files:**
- Modify:
  - `crates/pleiades-validate/src/aspects_validation.rs`;
  - `aspects_validation/tests.rs`;
  - `aspects_thresholds.rs`;
  - `crates/pleiades-validate/src/lib.rs` (the re-export);
  - `crates/pleiades-validate/src/render/cli.rs`;
  - `crates/pleiades-cli/src/cli.rs`.

**Interfaces:**
- Consumes: Task 2's files.
- Produces:
  - `pub fn validate_fictitious_aspects_corpus() -> Result<AspectsReport, AspectsError>`;
  - `validate_aspects_corpus()` returns planets, asteroids and fictitious bodies;
  - the CLI `validate-aspects-fictitious` (alias `aspects-fictitious-gate`).

- [ ] **Step 1: Data and plan.**
  - Add the constants `FICTITIOUS_CSV`, `FICTITIOUS_MANIFEST`, `FICTITIOUS_GRAZES_CSV` and `FICTITIOUS_GRAZES_MANIFEST` (`include_str!`, like the lunar ones).
  - Add `FICTITIOUS_PAIRS: [Pair; 28]` in the Global Constraints order, each `pair(Group::Mean, "<Body>", "Sun"|"Saturn", FULL_SPAN)`.
  - Add the 19 arms to `body_from_name`, each mapping the CSV name to the `CelestialBody` variant of the same name.

- [ ] **Step 2: `Scope::Fictitious`.**
  - Add the variant, with its doc.
  - `includes`: `true`.
  - `floor`: `MIN_ROWS_VALIDATED_FICTITIOUS`.
  - `title`: `"Fictitious-body aspects"`.
  - `pairs`: `&FICTITIOUS_PAIRS`.
  - The `source` match in `validate_scoped`: `"Swiss Ephemeris SWIEPH corpus (mean of date)"`.
  - The `" graze windows …"` suffix on pair lines applies to `Scope::LunarPoints | Scope::Fictitious`.

- [ ] **Step 3: Engine per scope.** Split `validate_scoped` in two:
  - `validate_scoped` keeps the manifest and grazes checks.
  - It then calls `fn compare_scope<B: EphemerisBackend>(engine: &EventEngine<B>, scope: Scope, rows: &[Row], windows: &[GrazeWindow]) -> Result<AspectsReport, AspectsError>`, which holds today's loop, floor check and summary verbatim.
  - The engine is chosen by scope:

```rust
    match scope {
        Scope::Fictitious => {
            let engine = EventEngine::new(RoutingBackend::new(vec![
                Box::new(PackagedDataBackend::new()),
                Box::new(FictitiousBackend::new(PackagedDataBackend::new())),
            ]));
            compare_scope(&engine, scope, &rows, &windows)
        }
        _ => compare_scope(&EventEngine::new(packaged_backend()), scope, &rows, &windows),
    }
```

  Use whatever trait bounds `EventEngine`'s aspect methods require (read `crates/pleiades-events/src/aspects.rs`'s `impl` header). `RoutingBackend` comes from `pleiades_backend`, `FictitiousBackend` from `pleiades_fict`, and `PackagedDataBackend` from `pleiades_data`.

- [ ] **Step 4: Entry points.**
  - `pub fn validate_fictitious_aspects_corpus()` runs `validate_scoped(FICTITIOUS_CSV, FICTITIOUS_MANIFEST, Scope::Fictitious, Some((FICTITIOUS_GRAZES_CSV, FICTITIOUS_GRAZES_MANIFEST)))`.
  - `validate_aspects_corpus()` merges it after the asteroids.
  - Re-export it in `lib.rs` like `validate_lunar_point_aspects_corpus`.
  - CLI arms, in both crates, exactly like the `validate-aspects-lunar` ones.
  - Help-text lines for the new command and its alias. Change the `validate-aspects` help to mention the fictitious bodies (mean of date).
  - `validate_subset` gains `fictitious_csv, fictitious_manifest, fictitious_grazes_csv, fictitious_grazes_manifest` parameters. It runs `check_manifest` and then `check_grazes` for them, after the lunar ones. `validate_aspects_corpus_subset` passes the constants.
  - Update every `validate_subset` call in the tests.

- [ ] **Step 5: Ceilings, floor and tests.**
  - `aspects_thresholds.rs`: add 28 arms named `<Body>-Sun` and `<Body>-Saturn`, PROVISIONAL at sep 5.0 and lon 5.0, each commented `// PROVISIONAL (Task 3 of the 2026-10-17 fictitious plan); measured in Task 4`.
  - `MIN_ROWS_VALIDATED_FICTITIOUS` = the rows of `fictitious.csv` minus the sum of `se_events` in `fictitious-grazes.csv`, computed from the committed files, with a doc comment.
  - Tests, mirroring the lunar ones:
    - `fictitious_corpus_parses_into_the_twenty_eight_pairs`, with the 28 names in order;
    - `committed_fictitious_corpus_matches_its_manifest`;
    - `committed_fictitious_graze_windows_match_their_manifest_and_corpus`;
    - `fictitious_scope_floor_is_its_own` (rows minus window `se_events` equals the floor);
    - `mean_subset_fails_a_tampered_fictitious_corpus`;
    - `mean_subset_fails_tampered_fictitious_graze_windows`. If the committed grazes file has no rows, assert the file holds only its header and comments, and tamper with the header comment instead.
    - `mean_subset_fails_a_fictitious_manifest_row_count_drift`;
    - `fictitious_aspects_gate_passes_within_ceilings`, opt-in through `PLEIADES_FULL_ASPECTS_GATE=1`.
  - Update `aspects_gate_passes_within_ceilings`'s counts: the full gate is now planets + asteroids + fictitious.

- [ ] **Step 6: Verify:**
  - `cargo test -p pleiades-validate --lib aspects_validation`, without the env var or `--include-ignored`;
  - the `pleiades-cli` dispatch tests;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  - `cargo fmt --all --check`;
  - `mise run docs`;
  - `cargo run -q -p pleiades-validate -- help`, which shows the command;
  - the new command with an extra argument, which must error.

  Do not run any gate.

- [ ] **Step 7: Commit:** `feat(validate): gate fictitious-body aspects in the mean-of-date frame (#168)`.

---

### Task 4 (controller-run measurement, then implementer): Ceilings

**Files:**
- Modify: `crates/pleiades-validate/src/aspects_thresholds.rs`

- [ ] **Step 1 (controller, background):** run `PLEIADES_FULL_ASPECTS_GATE=1 cargo test --release -p pleiades-validate --lib -- --nocapture fictitious_aspects_gate_passes_within_ceilings aspects_gate_passes_within_ceilings`. Both must pass. Save the fictitious pair lines and the time.
- [ ] **Step 2: Stop checks.**
  - No pair may exceed 10″ outside its windows.
  - Nibiru's pairs may differ from the rest. Their ceilings are per pair, so a loose Nibiru value must not be copied to the others.
- [ ] **Step 3: Set the 28 ceilings** by the rule in Global Constraints, each with a `// measured max <sep>", <lon>" (mean; SWIEPH sepl/semo, seorbel.txt, 2026-10-17)` comment. Add a group comment, and a module `//!` paragraph on the fictitious pairs: the mean frame and why, the rule, and the window rule.
- [ ] **Step 4: Commit:** `test(validate): measured ceilings for the fictitious-body aspect pairs (#168)`.

---

### Task 5: Runtime decision, docs

**Files:**
- Modify:
  - `mise.toml`, `AGENTS.md`, `docs/development.md`, `docs/status.md`;
  - `crates/pleiades-events/README.md`;
  - `crates/pleiades-validate/src/aspects_validation.rs` (rustdoc);
  - `.github/aspects-gate-failure-issue.md`;
  - conditionally, `.github/workflows/nightly.yml` and a new `.github/aspects-fictitious-gate-failure-issue.md`.

- [ ] **Step 1 (controller):** push the branch, dispatch `gh workflow run nightly.yml --ref <branch>`, and read the `aspects-gate` job's "Run full aspects gate" step timestamps.
  - Under 15 minutes: no structural change.
  - Otherwise, remove the fictitious pass from `validate_aspects_corpus` and add:
    - the mise task `gate-aspects-fictitious` (run `validate-aspects-fictitious`);
    - a `release-gate` dependency;
    - an `aspects-fictitious-gate` nightly job copied from `aspects-lunar-gate`;
    - its failure template and label `aspects-fictitious-gate-failure`.
  - Record the figure either way.
- [ ] **Step 2: Docs.** Wherever the gated aspect bodies are listed:
  - The fictitious bodies are gated in the mean-of-date frame: the 19 bodies with the Sun, and Cupido–Poseidon and Transpluto with Saturn.
  - The apparent frame is not gated, because of SE's fictitious light-time jumps (up to about 5″).
  - The `mise.toml` `gate-aspects` comment and `validate_aspects_corpus`'s rustdoc gain the measured nightly figure.
  - The events README's list of ungated bodies shrinks to the mean and osculating perigees.
  - Grep `fictitious` across `AGENTS.md`, `docs`, `mise.toml`, `crates/*/README.md` and `.github` for stale "not gated" text.
- [ ] **Step 3: Verify:** `mise run docs`, `mise run audit` and `mise run release-smoke`. The controller runs `mise run test-full`.
- [ ] **Step 4: Commit:** `docs: fictitious-body aspects gated in the mean-of-date frame (#168)`.
