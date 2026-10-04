# Test Timing Inventory

Measured on 2026-06-20 using `cargo nextest run --message-format libtest-json-plus` with isolated
per-crate runs (to avoid cross-binary interference). Build profile: `test` with `opt-level = 2`.
Total workspace: 1995 tests across 23 binaries.

**Coverage note:** `pleiades-cli` (98/98 tests), `pleiades-data` (180/183 non-ignored), and
`pleiades-validate` (full run, both `release_bundle_verify_a` and `release_bundle_verify_b` batches
captured) are all fully measured. All other crates are fast (< 3 s each) and fully measured.

**Validate update:** the `release_bundle_verify_b` batch (122 tests, 40–121 s each) was confirmed
after the initial draft. Including it, `pleiades-validate` has **166 tests over 60 s** — 120 of them
in `release_bundle_verify_b`. This makes the release-bundle family larger than first documented.

**Update 2026-10-03 — the inventory below is historical.** It describes the suite under nextest's
one-process-per-test model, where every slow test rebuilt its fixture. Since PR #111 the slow
families run under libtest (`mise run test-full`), one process per binary, so each fixture is built
once and the per-test figures below no longer apply. The current breakdown is in the next section;
the remaining ideas are FU-23 in `docs/follow-ups.md`.

---

## Section 0: `test-full` as of 2026-10-03

Measured from nightly run 37119694132 (GitHub `ubuntu-latest`, 4 cores, commit `64cbd132c`), from
step and per-test timestamps in the log. Whole `ci-nightly` tier: 963 s.

| Phase | Seconds | Notes |
|-------|--------:|-------|
| `test` dependency: build | 220 | every workspace crate recompiles (shared with `doctest`, 109 s) |
| `test` dependency: nextest run | 61 | 1999 tests |
| `test-full` step 1: rebuild `pleiades-validate` tests | 50 | second build of the same test code, see below |
| `test-full` step 1: `pleiades-validate` lib suite | 444 | 1155 tests on 4 threads |
| `test-full` step 2: `pleiades-cli` ignored tests | 154 | 9 tests; one of them runs 153 s |
| `test-full` step 2: `pleiades-data` ignored tests | 31 | 74 tests |
| `test-full` total (after its dependencies) | 680 | |

Long poles inside the 444 s `pleiades-validate` lib suite (thread-seconds, same run):

| Test(s) | Seconds | What it does |
|---------|--------:|--------------|
| `stations_validation::tests::stations_gate_passes_within_ceilings` | 253 | full stations gate, single thread |
| `render::cli::tests::run_all_numeric_gates_includes_*` (6 tests) | ~80 each, ~490 total | each ran the whole numeric battery |
| `tests::release_checklist::release_gate_command_aliases_the_release_checklist` | 154 | battery + bundle through the command |
| `tests::release_checklist::release_smoke_command_renders_the_smoke_report` | 96 | battery + bundle through the command |

A local run of the same suite with libtest's `--report-time` (8-CPU container shared with another
full test run, so absolute numbers are inflated about fivefold and only the shares are meaningful)
gave 13 771 test-seconds in total: the six battery tests 29 %, the release-bundle verify families
21 % (172 tests, includes time blocked on the shared fixture), the stations gate 13 %, the two
release-checklist command tests 11 %, the crossings and occultation gate/alias tests about 5 % each.

Two changes followed from this (same PR as this section):

- The six `run_all_numeric_gates_includes_*` tests now assert one shared, per-process battery run
  instead of running it six times. Back-to-back on the loaded 8-CPU container, those six tests alone:
  320 s wall and about 1680 thread-seconds before, 182 s wall and about 182 thread-seconds after.
- `mise run test` selects with `--workspace --exclude pleiades-validate` instead of a
  `-E 'not package(pleiades-validate)'` filter. The filter still compiled `pleiades-validate`'s test
  binaries (then skipped them), and `test-full` compiled them again under `-p` with a different
  feature unification (`pleiades-backend/test-backend` is on in a workspace build, off under `-p`).
  `cargo nextest list` is identical for both forms (1999 tests).

Nightly run 37124856870 on the branch with both changes (same runner class, one run each, so
run-to-run noise is not separated out):

| Phase | Before (37119694132) | After (37124856870) |
|-------|---------------------:|--------------------:|
| `test` dependency: build | 220 | 180 |
| `test` dependency: nextest run | 61 | 59 |
| `test-full`: build `pleiades-validate` tests | 50 | 54 |
| `test-full`: `pleiades-validate` lib suite | 444 | 358 |
| `test-full`: `pleiades-cli` ignored tests | 154 | 145 |
| `test-full`: `pleiades-data` ignored tests | 31 | 31 |
| `test-full` total (after its dependencies) | 680 | 589 |
| whole `ci-nightly` tier | 963 | 829 |

The full stations gate (242 s, started 80 s into the suite) is now the lib suite's long pole.

### FU-23 items (c)–(g), 2026-10-03

Four changes and two measurements, all local (24-core container, dev/test profile at
`opt-level = 2`, one run each unless stated); no nightly run yet, so the CI effect is projected.

- **(f)** `test-full` is two mise tasks, `test-full-validate` and `test-full-ignored`, run in
  parallel after `test` and `doctest`. Cargo serializes the two builds on the build-directory lock;
  the test runs overlap. Projected saving on CI: up to the shorter step (about 150 s before (c)).
- **(c)** `summary_commands_render_compact_reports` is 13 `#[ignore]`d tests by command family,
  moved verbatim (483 `render_cli` calls, 816 assertions before and after). The release family
  carries the only full `release-gate` run (one, not two: see (d)) and stays the longest; the
  other twelve finish well inside it, so the step's length is now one gate run rather than one gate
  run plus every other command.
- **(d)** Per-process shared outcomes in `pleiades-validate`'s `tests::test_support`. Gate runs per
  test process: crossings 6 → 3, occultations 7 → 3, release battery plus bundle 3 → 1 (one of each
  count is the numeric battery's own run). The affected tests, run together with
  `cargo test -p pleiades-validate --lib -- crossings occult release_gate_command release_smoke_command`:
  178 s wall on the shared machine, where the two release-checklist tests and the battery test now
  wait on one gate run instead of making three.
- **(e)** The stations gate scans its series on one thread each
  (`compare_series` under `std::thread::scope`, folded in corpus order). The full-gate test, same
  binary, same machine: 343 s sequential → 175 s parallel, report lines byte-identical (16 lines).
  The wall-clock is now the longest series, so a 4-core runner should land near 175 s too; the
  343 s of CPU would otherwise take about 90 s spread over four cores, so splitting the longest
  series into time chunks is the next step if the gate stays the long pole.
- **(g)** `debug = "line-tables-only"` on the `test` and `dev` profiles: clean
  `cargo test -p pleiades-validate --no-run` took 112 s and 112 s at the committed profile and 119 s
  with the setting (its first run, 227 s, was the cold dependency cache). Not applied.
  `[profile.test] opt-level = 3` (every crate): the parallel full stations gate test took 180 s
  against 173 s at `opt-level = 2`, and the profile switch rebuilt the test binary in 75 s against
  47 s. Not applied either; the numeric crates gain nothing past `opt-level = 2`.

Nightly run 37154489996 (same runner class, commit `baf3d392c`, dispatched by hand once #115 had
merged) measured the four changes together. The two `test-full` halves now overlap, so their
individual figures include contention for the four cores and only the totals compare:

| Phase | Before (37124856870) | After (37154489996) |
|-------|---------------------:|--------------------:|
| `test` dependency: build | 180 | 177 |
| `test` dependency: nextest run | 59 | 62 |
| `test-full-validate`: build `pleiades-validate` tests | 54 | 53 |
| `test-full-validate`: `pleiades-validate` lib suite | 358 | 347 |
| `test-full-ignored`: `pleiades-cli` ignored tests | 145 | 166 |
| `test-full-ignored`: `pleiades-data` ignored tests | 31 | 65 |
| `test-full` total (after its dependencies) | 589 | 401 |
| whole `ci-nightly` tier | 829 | 639 |

### FU-23 item (b), 2026-10-03

The restored `target` cache served only registry crates. Those are fingerprinted by checksum; a
path crate is fingerprinted by comparing each source file's mtime with the mtime of the unit's
dep-info file, and a fresh checkout gives every tracked file a new mtime. The blocking run
37153876584 on `main` spent about 200 s of its 331 s `mise run ci` step compiling first-party
crates it had in the cache, and the nightly `test` step paid the same 177 s.

Two changes, both in `.github/workflows` (PR #117):

- `.github/scripts/cargo-cache-mtimes.sh` runs after the cache restore. It reads the commit the
  cached build came from (a marker file inside `target`), floors every tracked file's mtime to a
  fixed instant older than any cached dep-info, and touches back to now every file that differs from
  that commit. A file is left old only when its content is what the cached build compiled, so a
  cache from another branch cannot yield a stale build; with no usable marker the script is a
  no-op. The common alternative, restoring last-commit mtimes (`git-restore-mtime`), was rejected
  because under the cross-branch `restore-keys` fallback a file whose last commit predates the
  cached build is judged fresh while its content differs.
- `target` moved out of the tools cache into its own entry keyed by run id with prefix
  `restore-keys` (the fuzz corpus pattern) and an explicit save step, because `actions/cache`
  never overwrites an existing key: inside the tools entry the marker could only have entered the
  cache when `Cargo.lock` or `mise.toml` changed. `pull_request` runs restore but do not save,
  since their checkout is an ephemeral merge commit that later runs cannot fetch; the `push` run
  of the same commit saves the equivalent entry with a branch commit. GitHub scopes cache access:
  a `push` run restores the newest entry from its branch or `main`, a `pull_request` run sees
  `main`'s entries only (verified on PR #117: its `pull_request` runs found no cache while the
  `push` runs of the same commits restored the branch's). So a PR's checks warm up through
  `main`'s entries, and the diff against `main`'s commit is what rebuilds.

Measured on the blocking job (`ubuntu-latest`, 4 cores). "Before" is the last run on `main` under
the old layout; "after" is the rerun of PR #117's push run on its own seeded cache, so the marker
equals HEAD and nothing differs. The seeding run itself (cold tools cache under the new prefix,
no `target`) took 10m14s, 6 min of it the one-time tools install:

| Phase | Before (37153876584) | After (37155697448, attempt 2) |
|-------|---------------------:|-------------------------------:|
| Job wall-clock | 405 | 196 |
| Cache restore (tools + `target`) | 33 | 17 |
| `mise run ci` step | 331 | 159 |
| `lint` | 88 | 10 |
| `docs` | 75 | 10 |
| `doctest` | 228 | 49 |
| `test` (build + nextest; nextest alone) | 282 (83) | 104 (93) |
| `release-smoke` | 325 | 155 |
| First-party crates compiled | 18 (all) | 0 |
| Cache save | 28 | 7 |

The remaining `mise run ci` time is the work itself: the nextest run, the release-smoke battery
with its bundle render and verify, gitleaks over the full history, and the doctests, which cargo
re-links per crate. A run whose commit touches a crate rebuilds that crate and its dependents, so
a typical PR lands between the two columns.

### FU-23 items (e) continued and (a), 2026-10-04

Nightly run 37154489996 (the run after #115, before #117), read from the per-test timestamps:
the `pleiades-validate` lib suite's 347 s is a chain, not an even load. The six numeric-battery
tests share one run but each holds a libtest thread while it waits, so three of the four threads
sit blocked from about 60 s to 100 s; the full stations gate starts at 102 s and ends at 280 s
(178 s, the longest series, as (e) predicted); the `validate_gates` and `release_checklist` tail
runs from 280 s to 347 s because libtest dispatches in name order.

- **(e) continued: the series scan in chunks.** `validate_scoped` now scans every series in
  ten-year windows (`CHUNK_DAYS`) on a pool of one thread per core, and folds the windows back in
  order. The window length is a multiple of 2 days, the common multiple of the engine's steps
  (0.25, 1 and 2 days), so each window brackets exactly as the single scan does; neighbouring
  windows share the one bracket that starts at their boundary, so a station exactly on a boundary
  is kept once (`join_chunks`). `chunked_scan_matches_the_single_scan_at_every_seam` compares
  100-day windows against the single scan on the mean Mars (2-day step) and Mercury (1-day step)
  series, bit for bit over 292 seams, in 7 s. Measured locally under a 4-core affinity mask
  (`taskset -c 0-3`, consecutive runs, the machine shared with another session's test run at a
  load average above 30, so absolute figures are inflated and only the ratio is meaningful):

  | Full stations gate test | Wall-clock (s) |
  |-------------------------|---------------:|
  | one thread per series (before) | 228 |
  | ten-year windows on a 4-thread pool (after, first run) | 130 |
  | same, second run (448 s of CPU at a load average above 40) | 179 |

  The 16 report lines are byte-identical between before and after.

  Measured on CI by dispatching the nightly on the branch (run 37183040722, `ubuntu-latest`,
  4 cores; the branch commit differs from the cached build's, so 9 first-party crates rebuilt).
  "Before" is the last nightly on `main` before #117:

  | Phase | Before (37154489996) | After (37183040722) |
  |-------|---------------------:|--------------------:|
  | `test` dependency: build | 177 | 89 |
  | `test` dependency: nextest run | 62 | 46 |
  | `test-full-validate`: build `pleiades-validate` tests | 53 | 42 |
  | `test-full-validate`: `pleiades-validate` lib suite | 347 | 282 |
  | full stations gate test inside it | 178 | 105 |
  | `test-full-ignored`: `pleiades-cli` ignored tests | 166 | 133 |
  | `test-full-ignored`: `pleiades-data` ignored tests | 65 | 67 |
  | `test-full` total (after its dependencies) | 401 | 323 |
  | whole `ci-nightly` tier | 639 | 460 |

  The build rows are #117's cache at work, not this change. The lib suite's new long pole is the
  tail after the stations gate (FU-23 (k)): the `release_bundle_verify_b` batch draining, then the
  release gate's own battery and bundle, seeded late by libtest's name order and serial on one
  thread for about 45 s while the other half of `test-full` has already finished.
- **(j) libtest threads, measured and not applied.** `test-full-validate` with
  `--test-threads=$(( 2 * $(getconf _NPROCESSORS_ONLN) ))`, dispatched as a second nightly on the
  branch (37183098143) against the run above. The second runner was slower on the unchanged
  `test` step as well (build 111 s against 89 s, nextest 61 s against 46 s), so the columns carry
  about 25 % of runner variance:

  | Phase | 4 threads (37183040722) | 8 threads (37183098143) |
  |-------|------------------------:|------------------------:|
  | `pleiades-validate` lib suite | 282 | 345 |
  | numeric battery tests done (seconds from suite start) | 76 | 170 |
  | full stations gate test inside it | 105 | 138 |
  | `test-full-ignored`: `pleiades-cli` ignored tests | 133 | 280 |
  | `test-full-ignored`: `pleiades-data` ignored tests | 67 | 49 |
  | `test-full` total (after its dependencies) | 323 | 395 |
  | whole `ci-nightly` tier | 460 | 568 |

  The two halves of `test-full` already share the four cores, and the gates are CPU-bound on their
  own pools, so the extra threads added contention rather than filling idle time; the battery, not
  its waiters, got slower. Reverted before merge.
- **(a)** `release-smoke` left `release-gate`'s dependency list; the gate command performs the
  smoke checks itself. No measurement: it removes one battery run from the release procedure only.

### FU-23 item (k), 2026-10-04

In run 37183040722 the `pleiades-validate` lib suite ended about 100 s after the stations gate.
libtest dispatches in name order, so `tests::release_checklist` came last, and its first test seeded
`release_gate_via_cli` (`render_cli(&["release-gate"])`): a second full numeric battery (the first is
`numeric_battery_outcome` in `render::cli::tests`) plus a second bundle render and verify (the first
is `pristine_release_bundle`), about 45 s on one thread while the other three sat idle and the other
half of `test-full` had already finished.

- **(k) the release-gate tests assert the shared outcomes.** `numeric_battery_outcome` now lives in
  `tests::test_support` (with `run_all_numeric_gates` made `pub(crate)`), and the release-gate and
  release-smoke tests read it, call `verify_release_bundle` on the pristine bundle, compare the pure
  checklist and smoke renderers, and check the three arms' extra-argument errors by text (the error
  names the command, so the arm was reached). `release_gate_via_cli` is deleted. The lib suite now
  runs the battery once and renders one bundle per process; the commands run end to end in
  `mise run release-smoke` (blocking tier) and `mise run release-gate`. Locally the 15 tests that
  read the battery or the release-gate run pass in one process with one battery (223 s at a load
  average above 65, so no local timing is quoted).

  Measured on CI by dispatching the nightly on the branch twice (runs 37186458265 and
  37187128430, `ubuntu-latest`, 4 cores). "Before" is run 37183040722 (the (e)/(a) branch, same
  suite otherwise). Both new runners were slower than the old one on every step this change does
  not touch (the unchanged rows below), so the suite's absolute figure is confounded and the rows
  that isolate the change are the release-gate pair and the tail after `validate_gates`. Seconds
  from the lib suite's start unless noted:

  | Phase | Before (37183040722) | After, run 1 (37186458265) | After, run 2 (37187128430) |
  |-------|---------------------:|---------------------------:|---------------------------:|
  | `test`: nextest run (unchanged) | 46 | 63 | 73 |
  | `test-full-ignored`: `pleiades-cli` ignored tests (unchanged) | 133 | 161 | 164 |
  | `test-full-ignored`: `pleiades-data` ignored tests (unchanged) | 67 | 75 | 76 |
  | numeric battery tests done | 76 | 97 | 98 |
  | full stations gate test done | 181 | 205 | 210 |
  | `release_checklist` non-gate tests done | 235 | 271 | 276 |
  | release-gate and release-smoke tests done | 282 | 276 | 280 |
  | release-gate pair after the other `release_checklist` tests | 47 | 5 | 4.5 |
  | `validate_gates` tests done | 270 | 300 | 305 |
  | `pleiades-validate` lib suite | 282 | 300 | 305 |
  | whole `ci-nightly` tier (wall-clock) | 460 | 496 | 430 |

  The release-gate pair is the change: 47 s of battery, bundle render and verify on one thread
  became 4.5 s (two `verify_release_bundle` calls on the pristine bundle plus renderer
  comparisons). Before, the suite ended 12 s after `validate_gates`, waiting on that thread; now it
  ends with `validate_gates`, whose shared crossings and occultations CLI runs (about 20 s, seeded
  by libtest's name order after `release_checklist`) are the new tail. Run 2's tier is shorter
  than run 1's only because run 1 had saved a `target` cache for the branch, so run 2 compiled
  nothing for `test`. Both new runs failed on
  `tests::comparison::regression_finding_has_a_displayable_summary_line`, which fails on `main`
  for an unrelated reason (the default-corpus comparison has had no notable regression since #122
  fixed VSOP87 Pluto, and no nightly has run `test-full` on `main` since); it fails in 0.07 s and
  does not affect the timings above. Fixed right after #130: the test builds its
  `RegressionFinding` directly, and a new test pins that the default-corpus comparison has no
  notable regression.

### FU-23 item (l), 2026-10-04

With #130 and #132 on `main`, two nightlies on `main` (runs 37192969731 and 37192975951) put the
two release-bundle verify families (`release_bundle_verify_a` and `_b`, 172 tests) across about
90 s of a 190 s `pleiades-validate` lib suite. Each test is cheap apart from one
`verify-release-bundle` call, and a throwaway timing patch around the checks in
`verify_release_bundle_internal` showed that call re-rendering six summaries on every invocation
(seconds at a load average of 80, so only the ratios matter): release-notes summary 4.2,
packaged-artifact source-fit/hold-out sync 2.7, production profile 1.2–1.8, target thresholds 1.1,
backend-matrix summary 0.8, scope envelopes 0.3–0.7. Every other check was already memoized or
under 0.1 s.

- **(l) memoize the six summaries.** All six depend only on compiled-in data (the embedded packaged
  artifact, catalogs and profiles). Three already had identical `OnceLock` twins under
  `posture::data::coverage`, so their `validated_*` wrappers in `render::summary::artifact` now
  delegate to them; the release-notes summary, the backend-matrix summary and the target-threshold
  summary (whose error text differs from its twin's) get their own per-process `OnceLock` around an
  `_uncached` body, and `tests::render_memoization` pins each memoized value equal to its uncached
  rendering. `workspace_provenance` (git state) and the validation report (already cached per
  `rounds`; its 0.3–0.5 s is normalizing the text) are unchanged. Locally a full successful verify
  went from about 11 s of checks to under 2.4 s at a load average of 65–95.

  Measured on three nightlies dispatched on the branch (runs 37195011776, 37195017266,
  37195539899; the first two ran concurrently with no branch `target` cache, so their `test` step
  recompiled; the third restored the cache). The runners differed by up to 1.8× on the unchanged
  nextest step, so the suite's absolute length is not comparable; the rows that isolate the change
  are the verify families' span and how long the suite runs on after the stations gate. Seconds
  from the lib suite's start unless noted:

  | Phase | Before 1 (37192969731) | Before 2 (37192975951) | After 1 (37195011776) | After 2 (37195017266) | After 3 (37195539899) |
  |-------|---:|---:|---:|---:|---:|
  | `test`: nextest run (unchanged, runner-speed reference) | 40 | 40 | 45 | 63 | 73 |
  | verify families (`_a` first to `_b` last) | 85–173 | 81–172 | 85–115 | 117–162 | 111–151 |
  | verify families' span | 88 | 91 | 30 | 45 | 40 |
  | full stations gate test done | 112 | 111 | 160 | 216 | 207 |
  | `validate_gates` tests done | 191 | 189 | 169 | 223 | 219 |
  | `pleiades-validate` lib suite | 191 | 189 | 169 | 224 | 219 |
  | suite end after the stations gate | 79 | 78 | 9 | 7 | 12 |
  | `test-full-ignored` ends | 140 | 140 | 168 | 207 | 217 |
  | `nightly` job (wall-clock) | 319 | 309 | 416 | 526 | 343 |

  The verify families went from about 90 s to 30–45 s on runners 1.1–1.8× slower, about a
  quarter of their former cost after correcting for runner speed. Scaling each suite by its
  runner's nextest ratio gives roughly 120–150 s against 190 s, a crude correction that only
  supports "about a quarter to a third shorter". Before, the suite ran on for about 80 s after the
  stations gate; now it ends within about 10 s of it, and the other half of `test-full` ends at
  the same time. The long poles are now the stations gate (about 55 s after the battery on a fast
  runner) and the `pleiades-cli` ignored tests in the other half, so a further cut needs both
  halves to shrink.

### FU-23 items (m), (n) and (o), 2026-10-04

On `main` after #136 (nightly run 37199097620), the tier ran in four phases: nextest (0–69 s), a
54 s compile of `pleiades-validate`'s test binary (69–123 s, both `test-full` halves waiting on
it), the two halves in parallel (123–256 s), and the `pleiades-data` ignored tests on their own
(256–294 s) after the `pleiades-cli` ones in the same `cargo test` call.

- **(m)** `test-full-build` compiles the `test-full` binaries during the nextest run. `test-build`
  records nextest's binaries (`cargo nextest list --list-type binaries-only`) and the cargo
  metadata, and `test` runs from that record without invoking cargo. Without the record it gained
  nothing. Started alongside `test`, `test-full-build` sometimes took the build lock first, and
  nextest waited for it (runs 37229732320, 37229734252). After a plain `nextest run --no-run`,
  `cargo nextest run`'s own no-op build still waited on the lock, 53–61 s (runs 37230857839,
  37230859135).
- **(n)** The ignored half is two concurrent tasks, `--bins` (the `pleiades-cli` tests) and
  `--lib --test '*'` (everything else, `pleiades-data` included).
- **(o)** The `pleiades-cli` release-summary test no longer runs `release-gate` (a battery plus a
  bundle render and verify), and the output-alias test verifies the shared pristine bundle instead
  of rendering its own. Per-test times locally (load ~27, ratios only) put those two at 75 s and
  31 s of the half's 137 s.

A warm cache hides (m), because a nightly whose cache already holds the `pleiades-validate` test
binary has nothing to compile (the second round, runs 37230425262 to 37230432371, was like that).
To measure it, two throwaway branches, `main` and this branch, each got the same comment change in
a `pleiades-validate` test file, and each was dispatched twice at the same time. A third round was
needed, because the second force-pushed the probe branch and left the cached commit unreachable,
so the mtime script fell back to a full rebuild. Doctest (CPU-bound, starting at t=0 in both
arms) is the runner-speed reference, because nextest now shares its cores with the compile.
Durations in seconds; "at" rows are seconds from the start of `mise run ci-nightly`:

| Phase | `main` 1 (37232063518) | `main` 2 (37232065621) | Branch 1 (37232067510) | Branch 2 (37232069475) |
|-------|---:|---:|---:|---:|
| doctest duration (runner-speed reference) | 19 | 29 | 30 | 29 |
| nextest run duration | 34 | 58 | 90 | 70 |
| at: `test-full` binaries ready | 69 | 115 | 101 | 90 |
| `pleiades-cli` ignored tests duration | 56 | 102 | 86 | 71 |
| at: `pleiades-data` and other ignored lib tests end | 153 | 261 | 172 | 150 |
| `pleiades-validate` suite duration | 76 | 132 | 137 | 121 |
| tier duration (`mise run ci-nightly`) | 153 | 262 | 239 | 211 |
| `nightly` job (wall-clock) | 206 | 325 | 280 | 267 |

`main` 1 ran on a runner about 1.5× faster than the other three (doctest 19 s against 29–30 s),
so the comparison is `main` 2 against the two branch runs, which ran at the same speed. The
compile now overlaps nextest: the binaries are ready 14–25 s sooner, although nextest itself
slows from about 58 s to 70–90 s with the two sharing four cores. The `test-full` phase went from
147 s to 121–138 s. The tier went from 262 s to 211–239 s, about 25–50 s less. The
`pleiades-validate` half, whose long pole is the stations gate, now ends about 50 s after the
other `test-full` tasks, so a further cut has to shorten that suite.

---

## Section 1: Timing Inventory

Slowest 40 tests ranked slowest-first. All times from clean isolated single-crate runs.

| Rank | Crate | Test Name | Seconds |
|-----:|-------|-----------|--------:|
| 1 | pleiades-cli | `cli::tests::summary_commands::summary_commands_render_compact_reports` | 374 |
| 2 | pleiades-cli | `cli::tests::release::bundle_release_commands_accept_output_alias` | 324 |
| 3 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_commands_accept_output_aliases_in_the_validation_front_end` | 312 |
| 4 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_writes_expected_artifacts` | 310 |
| 5 | pleiades-cli | `cli::tests::release::verify_release_bundle_command_verifies_a_staged_bundle` | 309 |
| 6 | pleiades-cli | `cli::tests::release::bundle_release_command_writes_a_staged_bundle` | 308 |
| 7 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_accepts_rendered_bundle` | 302 |
| 8 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_whitespace_padded_provenance` | 293 |
| 9 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_placeholder_provenance` | 288 |
| 10 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_multiline_provenance` | 277 |
| 11 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_manifest_path_drift` | 267 |
| 12 | pleiades-cli | `cli::tests::artifact_and_workspace::artifact_and_workspace_commands_render_compact_reports` | 256 |
| 13 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_api_stability_posture_id_entry` | 246 |
| 14 | pleiades-cli | `cli::tests::validation::validation_report_commands_render_compact_reports` | 240 |
| 15 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_cargo_version_entry` | 229 |
| 16 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_profile_id_entry` | 228 |
| 17 | pleiades-cli | `cli::tests::misc::fallback_summary_commands_remain_reachable_from_the_cli` | 225 |
| 18 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_duplicate_api_stability_posture_id_entry` | 224 |
| 19 | pleiades-validate | `artifact::tests::render_artifact_summary_includes_span_caps` | 223 |
| 20 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_workspace_status_entry` | 223 |
| 21 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_source_revision_entry` | 223 |
| 22 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_checksum_mismatches` | 213 |
| 23 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_rustc_version_entry` | 212 |
| 24 | pleiades-data | `tests::coverage::packaged_artifact_fit_outlier_summary_prioritizes_distance_channel_outliers` | 127 |
| 25 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_reflects_the_current_posture` | 127 |
| 26 | pleiades-cli | `cli::tests::misc::packaged_artifact_and_ayanamsa_audit_summary_commands_render_directly_from_the_cli` | 126 |
| 27 | pleiades-data | `tests::coverage::packaged_artifact_fit_threshold_violation_summary_validation_rejects_drift` | 126 |
| 28 | pleiades-data | `tests::coverage::packaged_artifact_fit_channel_outlier_summary_validation_rejects_drift` | 121 |
| 29 | pleiades-cli | `cli::tests::misc::packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary` | 121 |
| 30 | pleiades-data | `tests::coverage::packaged_artifact_fit_threshold_summary_reflects_the_current_posture` | 120 |
| 31 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift` | 120 |
| 32 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_profile_drift` | 114 |
| 33 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_parameter_drift` | 114 |
| 34 | pleiades-data | `tests::coverage::packaged_artifact_generation_artifacts_keep_lookup_epoch_and_segment_strategy_aligned` | 112 |
| 35 | pleiades-data | `tests::coverage::packaged_artifact_fit_margin_summary_validation_rejects_envelope_drift` | 109 |
| 36 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_checksum_drift` | 107 |
| 37 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_profile_id_drift` | 106 |
| 38 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_regeneration_drift` | 105 |
| 39 | pleiades-data | `tests::coverage::packaged_artifact_generator_parameters_validation_rejects_artifact_profile_drift` | 103 |
| 40 | pleiades-data | `tests::coverage::packaged_artifact_fit_margin_summary_reflects_the_current_posture` | 103 |

Additional slow tests beyond rank 40 (all in `pleiades-data tests::coverage::*`):
- `packaged_artifact_generation_manifest_validation_rejects_label_drift` — 103 s
- `packaged_artifact_generator_parameters_validation_rejects_body_coverage_drift` — 102 s
- `packaged_artifact_generation_manifest_validation_rejects_source_drift` — 101 s
- `packaged_artifact_generation_manifest_validation_rejects_request_policy_drift` — 101 s
- `packaged_artifact_generator_parameters_validation_rejects_label_drift` — 100 s
- `packaged_artifact_generator_parameters_validation_rejects_artifact_version_drift` — 99 s
- `packaged_artifact_generator_parameters_validation_rejects_checksum_drift` — 99 s
- `packaged_artifact_fit_margin_summary_validation_rejects_threshold_drift` — 97 s
- `packaged_artifact_generator_parameters_validation_rejects_profile_id_drift` — 93 s
- `packaged_artifact_production_profile_summary_validation_rejects_artifact_profile_drift` — 86 s
- `packaged_artifact_regeneration_summary_validation_rejects_residual_body_subset_drift` — 83 s
- `packaged_artifact_generator_parameters_validation_rejects_source_provenance_drift` — 83 s
- `packaged_artifact_generator_parameters_validation_rejects_speed_policy_drift` — 82 s
- `packaged_artifact_regeneration_summary_includes_reference_snapshot_coverage` — 84 s
- `packaged_artifact_generator_parameters_validation_rejects_request_policy_drift` — 80 s
- `packaged_artifact_generator_parameters_validation_rejects_target_threshold_drift` — 79 s
- `packaged_artifact_generator_parameters_validation_rejects_time_range_drift` — 77 s
- `packaged_artifact_production_profile_summary_reflects_the_current_posture` — 77 s
- `default_window_artifact_matches_explicit_default_over` — 34 s
- `build_from_reference_produces_all_bodies_with_spanning_segments` — 31 s
- `snapshot_reconstruction_covers_only_constrained_asteroids` — 17 s

---

## Section 2: Families

### Family 1: release-bundle

**Description:** Tests that build a full release bundle by calling `render_cli(&["bundle-release",
...])` or `render_cli(&["bundle-verify", ...])`. Each test independently spawns the CLI binary,
runs the complete bundle-generation pipeline (compiles artifact, assembles manifest, signs checksums),
and then verifies the output. This pipeline takes 200–375 s per test process.

**Crates and files:**
- `pleiades-cli` — `src/tests/release.rs`, `src/tests/summary_commands.rs`,
  `src/tests/artifact_and_workspace.rs`, `src/tests/validation.rs`, `src/tests/misc.rs`
- `pleiades-validate` — `src/tests/release_bundle_verify_a.rs` (26 tests, 144–312 s each),
  `src/tests/release_bundle_verify_b.rs` (122 tests, 40–121 s each — confirmed; these tamper a
  rendered bundle then re-verify, so each re-runs the bundle pipeline)

**Real test names from measurement (rank 1–23, plus further verify_b tests):**
- `cli::tests::summary_commands::summary_commands_render_compact_reports` — 374 s
- `cli::tests::release::bundle_release_commands_accept_output_alias` — 324 s
- `tests::release_bundle_verify_a::release_bundle_commands_accept_output_aliases_in_the_validation_front_end` — 312 s
- `tests::release_bundle_verify_a::release_bundle_writes_expected_artifacts` — 310 s
- `cli::tests::release::verify_release_bundle_command_verifies_a_staged_bundle` — 309 s
- `cli::tests::release::bundle_release_command_writes_a_staged_bundle` — 308 s
- `tests::release_bundle_verify_a::release_bundle_validate_accepts_rendered_bundle` — 302 s
- ... (rest of verify_a batch, 144–293 s each, 26 tests total)
- `tests::release_bundle_verify_b::verify_release_bundle_rejects_tampered_reference_snapshot_boundary_summary_files_even_with_updated_checksum` — 121 s (representative of 122 verify_b tests, 40–121 s each)
- `cli::tests::artifact_and_workspace::artifact_and_workspace_commands_render_compact_reports` — 256 s
- `cli::tests::validation::validation_report_commands_render_compact_reports` — 240 s
- `cli::tests::misc::fallback_summary_commands_remain_reachable_from_the_cli` — 225 s
- `validate::artifact::tests::render_artifact_summary_includes_span_caps` — 223 s
- `cli::tests::misc::packaged_artifact_and_ayanamsa_audit_summary_commands_render_directly_from_the_cli` — 126 s
- `cli::tests::misc::packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary` — 121 s

**Planned treatment:** **dedup** — run the bundle pipeline once (or a small fixed number of times),
share the rendered output directory across all tests that only need to verify a single property of
the bundle. This would collapse the ~30 tests in this family from ~30 × 250 s to ~1 × 250 s + 29 × ~1 s.

---

### Family 2: kernel-free regeneration

**Description:** Tests in `pleiades-data` that call `build_packaged_artifact_from_reference_over`
directly (or indirectly via `packaged_artifact()` which calls `build_packaged_artifact`). These
tests rebuild the full Chebyshev-segment artifact from the reference snapshot without an external
kernel. Each process rebuilds independently (no cross-process `OnceLock` sharing).

**Crates and files:**
- `pleiades-data` — `src/tests/coverage.rs` (functions `build_from_reference_produces_all_bodies_with_spanning_segments`,
  `default_window_artifact_matches_explicit_default_over`)
- `pleiades-data` — `tests/artifact_regen.rs`

**Real test names from measurement:**
- `tests::coverage::build_from_reference_produces_all_bodies_with_spanning_segments` — **31 s**
- `tests::coverage::default_window_artifact_matches_explicit_default_over` — **34 s**
- `tests::codec::snapshot_reconstruction_covers_only_constrained_asteroids` — **17 s**

**Observation vs. expectation:** These tests are slower than expected but are overshadowed by the
dense-sweep family (see Family 3 below). The expected test names `build_from_reference_*` and
`default_window_artifact_matches_explicit_default_over` are confirmed present and slow, but
the biggest contributors to `pleiades-data` wall time are the parametric-sweep tests in Family 3.

**Planned treatment:** **memoize** — share one built artifact via an `OnceLock` at the
integration-test process level, or move the build into a `#[test_support]` fixture loaded once
per binary.

---

### Family 3: dense numerical sweeps (coverage/fit)

**Description:** The largest slow family by count. Every test in `src/tests/coverage.rs` that
calls any `packaged_artifact_*_details()` helper ultimately calls `packaged_artifact()`, which
builds the full artifact via `OnceLock::get_or_init`. Because nextest runs each test in a separate
process, each of ~100 coverage tests pays the full ~100 s artifact-build cost independently.
These are NOT dense numerical sweeps in the traditional sense (they do not iterate over a large
parameter grid); rather, they each rebuild the artifact to exercise a particular summary function
or validation path.

**Crates and files:**
- `pleiades-data` — `src/tests/coverage.rs` (tests prefixed `packaged_artifact_fit_*`,
  `packaged_artifact_generation_manifest_*`, `packaged_artifact_generator_parameters_*`,
  `packaged_artifact_production_profile_*`, `packaged_artifact_regeneration_summary_*`)

**Real test names from measurement (sample, all ~80–127 s each):**
- `tests::coverage::packaged_artifact_fit_outlier_summary_prioritizes_distance_channel_outliers` — 127 s
- `tests::coverage::packaged_artifact_generation_manifest_reflects_the_current_posture` — 127 s
- `tests::coverage::packaged_artifact_fit_threshold_violation_summary_validation_rejects_drift` — 126 s
- `tests::coverage::packaged_artifact_fit_channel_outlier_summary_validation_rejects_drift` — 121 s
- `tests::coverage::packaged_artifact_fit_threshold_summary_reflects_the_current_posture` — 120 s
- `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift` — 120 s
- ... (58+ more, all 77–127 s)

**Observation vs. expectation:** The task brief listed `tests/fit.rs`, `tests/coverage.rs`, and
`tests/lookup.rs` as expected dense-sweep files (implying parameter-space iteration). What was
actually found is that `src/tests/coverage.rs` contains ~100 parametric-validation tests that
each happen to rebuild the artifact from scratch per nextest process. The `src/tests/fit.rs` and
`src/tests/lookup.rs` files are fast (< 0.1 s each). The analogous `pleiades-vsop87` tests
(`tests/evidence.rs`, `tests/backend.rs`) are also fast (< 0.3 s each).

**Planned treatment:** **memoize / shrink** — consolidate coverage tests that share the same
`packaged_artifact()` call into a single test binary fixture (so the OnceLock fires once per
binary launch), OR collect all per-field validation assertions into fewer test functions. The
goal is to pay the ~100 s build cost O(1) times per coverage test run rather than O(N) times.

---

## Summary: actual vs. expected family map

| Expected family | Expected crate | Confirmed? | Notes |
|-----------------|---------------|-----------|-------|
| kernel-free regeneration | pleiades-data | Yes | `build_from_reference_*`, `default_window_artifact_*` at 31–34 s |
| release-bundle | pleiades-validate + pleiades-cli | Yes | 40–374 s per test; 26 `verify_a` + 122 `verify_b` + `cli::tests::release::*` — 166 validate tests over 60 s |
| dense numerical sweeps | pleiades-data coverage | Partial match | Not true sweeps — artifact-rebuild cost paid O(N) across ~100 validation tests |
| pleiades-vsop87 sweeps | pleiades-vsop87 | No slow tests | All vsop87 tests < 0.3 s; evidence.rs and backend.rs are fast |
| pleiades-data fit/lookup | pleiades-data | No slow tests | `src/tests/fit.rs` and `src/tests/lookup.rs` are fast (< 0.1 s) |
