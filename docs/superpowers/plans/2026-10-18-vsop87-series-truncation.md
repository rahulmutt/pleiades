# Truncated VSOP87B Series Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the default `Vsop87Backend` evaluate VSOP87B series truncated at
ε = 1e-8 (cutoff for t_max = 0.6 millennia) for instants inside 1600–2600 CE.
Outside that window it keeps the full series. A new `Vsop87FullSeriesBackend`
reproduces today's output everywhere.

**Architecture:** Each planet's table module keeps its full tables and adds a
lazily built truncated copy. The copy is filtered from the full tables at load
time, so no new data artifacts are needed. The backend picks a
`SeriesFidelity` once per request from the request instant, the same way it
already pins the Pluto path. That fidelity is threaded through the geocentric
reduction and the finite-difference motion, so a request never mixes the two
series. Both public backend types share one implementation, parameterised by
an internal `SeriesPolicy`.

**Tech Stack:** Rust (workspace toolchain from `mise.toml`), no new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-18-vsop87-series-truncation-design.md`

## Global Constraints

- Truncation rule: keep a term when `|A| · 0.6^α ≥ 1e-8`. α is the term's
  power of t (Julian millennia from J2000). The rule applies to the L, B and R
  series of every planet and of the Earth.
- The truncated window is `2_305_447.5 ≤ jd_tt < 2_670_690.5`, that is from
  1600-01-01 00:00 TT up to 2600-01-01 00:00 TT. Outside it, output is
  bit-identical to today's.
- Drift from the full series is at most 0.30″ in longitude and in latitude for
  every body anywhere in the window. If any measured max exceeds 0.27″, stop
  and report: that is the design's anti-goal.
- `Vsop87Backend` stays a unit struct with `Debug, Default, Clone, Copy` and
  `pub const fn new()`. No source-breaking change.
- Pure Rust, no new dependencies.
- Do not edit `crates/pleiades-vsop87/data/*`, and do not reformat or split the
  generated data.
- Do not bump versions or edit CHANGELOG files by hand: release-plz generates
  them from commit and PR titles (`docs/release-process.md`). The squash-merge
  title is `perf(vsop87): truncate the VSOP87B series inside 1600–2600 (#281)`.
- Run `cargo fmt --all` before every commit.

## Review Focus

1. **A request on either side of a window edge.** Just inside, it is the
   truncated value; just outside, it is the full series bit for bit. The step
   between them is within the drift ceiling. Pinned in Task 2 (edge tests).
2. **Speed for a request just inside an edge.** Both stencil points (±0.005
   day) must use the request's fidelity. A mixed stencil would turn the edge
   step into a speed spike of up to about 50″/day. Pinned in Task 2 (motion
   test).
3. **Far dates such as −3000 or 5000 CE.** These must be exactly today's
   output. Pinned in Task 2 (far-date identity).
4. **Pluto.** Its own series is untouched, but it subtracts the Earth, so
   Pluto drifts too. Pluto belongs in the drift gate. Pinned in Task 2.
5. **`CompositeBackend` and chart callers holding `Vsop87FullSeriesBackend`.**
   It must be a drop-in `EphemerisBackend` with the same trait derives. Pinned
   in Task 2 (composite smoke test).

---

### Task 1: Truncated series tables

**Files:**
- Create: `crates/pleiades-vsop87/src/tables/series.rs`
- Modify: `crates/pleiades-vsop87/src/tables/mod.rs` (add `pub(crate) mod series;`)
- Modify: `crates/pleiades-vsop87/src/tables/vsop87b_{earth,mercury,venus,mars,jupiter,saturn,uranus,neptune}.rs`
  (each `*_lbr` and its `OnceLock`, plus the inline J2000 tests that call `*_lbr`)
- Test: `crates/pleiades-vsop87/src/tests/series_truncation.rs` (new; add `mod series_truncation;` to `src/tests/mod.rs`)

**Interfaces:**
- Produces, in `tables::series`:
  - `pub(crate) enum SeriesFidelity { Full, Truncated }`, deriving `Clone, Copy, Debug, PartialEq, Eq`
  - `pub(crate) const TRUNCATION_THRESHOLD: f64 = 1e-8;`
  - `pub(crate) const TRUNCATION_T_MAX_MILLENNIA: f64 = 0.6;`
  - `pub(crate) fn truncate_tables(full: &Vsop87SeriesTables, threshold: f64, t_max_millennia: f64) -> Vsop87SeriesTables`
  - `pub(crate) struct Vsop87SeriesSource`, with:
    - `pub(crate) const fn new(bytes: &'static [u8]) -> Self`
    - `pub(crate) fn tables(&self, fidelity: SeriesFidelity) -> &Vsop87SeriesTables`
    - `pub(crate) fn lbr(&self, julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr`
  - `Vsop87SeriesSource` holds two `OnceLock<Vsop87SeriesTables>`. The full
    tables come from `parse_generated_vsop87b_tables(bytes)`. The truncated
    tables come from
    `truncate_tables(full, TRUNCATION_THRESHOLD, TRUNCATION_T_MAX_MILLENNIA)`.
- Produces, in each planet module:
  `pub(crate) fn <planet>_lbr(julian_day_tt: f64, fidelity: SeriesFidelity) -> SphericalLbr`.
  It is backed by `static <PLANET>: Vsop87SeriesSource = Vsop87SeriesSource::new(include_bytes!(…));`,
  which replaces the module's `OnceLock` and `<planet>_tables()`. The
  longitude stays `rem_euclid(TAU)` as today.

- [ ] **Step 1: Write the failing tests** in `src/tests/series_truncation.rs`.
  - `truncated_tables_keep_the_expected_term_counts`: for each body, the total
    count of kept terms (L+B+R, all powers) at the default ε and t_max equals:
    - Earth 578
    - Mercury 672
    - Venus 419
    - Mars 1390
    - Jupiter 1654
    - Saturn 3115
    - Uranus 3159
    - Neptune 1278

    These come from the throwaway probe as planet+Earth minus Earth. If the
    implementation disagrees, check the rule (`>=`, `abs()`, power index from
    0) before changing a number, and report any change.
  - `truncate_tables_keeps_power_alignment`: the truncated tables have the
    same number of power vectors per variable as the full ones, even when one
    is emptied. For example, with `threshold = f64::INFINITY` every power
    vector is empty and their count is unchanged.
  - `full_fidelity_is_bit_identical_to_the_generated_tables`: at JD 2451545.0,
    2305447.5 and 2670690.5, `<planet>_lbr(jd, Full)` equals the value from
    `evaluate` over `parse_generated_vsop87b_tables(include_bytes!(…))` for
    each planet. The comparison is `==` on all three components.
  - `truncated_fidelity_differs_but_stays_close_at_j2000`: for Mars,
    `Truncated` differs from `Full` (`!=`), and the longitude difference is
    under `0.30 / 3600` degrees once converted from radians.

- [ ] **Step 2: Run the tests to verify they fail.**
  Run: `cargo test -p pleiades-vsop87 series_truncation`
  Expected: compile errors (`series` module and `SeriesFidelity` missing).

- [ ] **Step 3: Implement `tables/series.rs` and switch the eight planet
  modules to `Vsop87SeriesSource`.** Inside the planet modules the existing
  inline tests call `<planet>_lbr(2_451_545.0)`; change them to pass
  `SeriesFidelity::Full`, with their expected values unchanged. In
  `backend.rs`, pass `SeriesFidelity::Full` at every call site for now (Task 2
  threads the real choice through). Output therefore stays bit-identical
  after this task.

- [ ] **Step 4: Run the crate's tests.**
  Run: `cargo test -p pleiades-vsop87`
  Expected: PASS, including every pre-existing test.

- [ ] **Step 5: Commit.**
  ```bash
  cargo fmt --all
  git add crates/pleiades-vsop87/src
  git commit -m "refactor(vsop87): load a truncated copy of each VSOP87B series alongside the full one (#281)"
  ```

### Task 2: Windowed truncation in the default backend, plus the full-series backend

**Files:**
- Modify: `crates/pleiades-vsop87/src/tables/series.rs` (window constants and `default_fidelity_at`)
- Modify: `crates/pleiades-vsop87/src/backend.rs`
- Modify: `crates/pleiades-vsop87/src/lib.rs` (`pub use backend::{Vsop87Backend, Vsop87FullSeriesBackend};`)
- Test: `crates/pleiades-vsop87/src/tests/series_truncation.rs`

**Interfaces:**
- Consumes: `SeriesFidelity` and `<planet>_lbr(jd, fidelity)` from Task 1.
- Produces, in `tables::series`:
  - `pub(crate) const TRUNCATED_WINDOW_START_JD: f64 = 2_305_447.5;`, which is 1600-01-01 00:00 TT, inclusive
  - `pub(crate) const TRUNCATED_WINDOW_END_JD: f64 = 2_670_690.5;`, which is 2600-01-01 00:00 TT, exclusive
  - `pub(crate) fn default_fidelity_at(julian_day_tt: f64) -> SeriesFidelity`
- Produces, in `backend`:
  - `pub struct Vsop87FullSeriesBackend;`, deriving `Debug, Default, Clone, Copy`, with `pub const fn new() -> Self`
  - `impl EphemerisBackend for Vsop87FullSeriesBackend`
  - internal `enum SeriesPolicy { WindowedTruncation, Full }` with `fn fidelity_at(self, julian_day_tt: f64) -> SeriesFidelity`
  - `Vsop87Backend::geocentric_coordinates_on(body, days, pluto_path, fidelity: SeriesFidelity)`
  - `Vsop87Backend::motion(body, days, pluto_path, fidelity: SeriesFidelity)`
  - The shared request path is `fn compute(policy: SeriesPolicy, req, with_motion)`.
    It picks the fidelity once from the request instant and uses it for the
    position, both motion stencil points and the quality annotation.
    `Vsop87Backend` uses `WindowedTruncation`, `Vsop87FullSeriesBackend` uses
    `Full`.
- Quality, my call:
  - A `GeneratedBinaryVsop87b` or `VendoredVsop87b` body evaluated `Truncated`
    reports `QualityAnnotation::Approximate`. This follows the mapping the code
    already has for `TruncatedVsop87b`.
  - The same bodies evaluated `Full` report `Exact`, as today.
  - Pluto keeps its current mapping on both backends.
- Metadata, my call:
  - `Vsop87FullSeriesBackend::metadata()` returns exactly today's
    `BackendMetadata`.
  - `Vsop87Backend::metadata()` is the same, but its provenance summary
    sentence gains this text:
    `" Inside 1600-01-01..2600-01-01 TT the VSOP87B series are truncated (terms with |A|·0.6^α < 1e-8 dropped, at most 0.3″ from the full series); outside it the full series is evaluated."`
  - Build both from one helper so they cannot drift apart.
  - Rustdoc on both types states the window, the edge step of up to about 0.3″,
    and when to choose the full type.

- [ ] **Step 1: Write the failing tests** in `src/tests/series_truncation.rs`.
  Use a shared helper,
  `fn mean_place(backend: &impl EphemerisBackend, body, jd) -> EclipticCoordinates`,
  built on `position_without_motion` and TT.
  - `truncation_drift_stays_within_ceilings_over_1600_2600`:
    - Instants: 2,000 of them,
      `jd_i = START + (i + frac(i · 0.618034)) · (END − START) / 2000` for
      i in 0..2000. The jitter avoids aliasing with the year.
    - Bodies: Sun through Neptune, and Pluto.
    - Compare the default backend with `Vsop87FullSeriesBackend`. Track max
      |Δλ| (via `signed_longitude_delta_degrees`, in ″) and max |Δβ| per body.
    - Assert each max is at most its entry in
      `const TRUNCATION_DRIFT_CEILINGS_ARCSEC: [(CelestialBody, f64, f64); 9]`
      (body, longitude, latitude).
    - Fill the table from one measuring run, printed with `--nocapture`:
      `ceiling = min(0.30, round_up_to_0.01(1.25 × measured max))`.
    - Expected longitude maxima, roughly: Sun 0.05, Mercury 0.07, Venus 0.17,
      Mars 0.25, Jupiter 0.06, Saturn 0.09, Uranus 0.07, Neptune 0.04.
  - `default_backend_is_truncated_from_the_window_start`: at
    `TRUNCATED_WINDOW_START_JD` and at `TRUNCATED_WINDOW_END_JD − 1e-3`, the
    Mars default result `!=` the full one.
  - `default_backend_is_full_series_outside_the_window`:
    - At `START − 1e-3`, at `END`, at JD 625_673.5 (about −3000) and at
      JD 3_547_272.5 (about 5000 CE), every body's default result equals the
      full one.
    - Use `==` on longitude, latitude, distance and the equatorial channel,
      with motion included (via `position`).
  - `edge_step_is_within_the_drift_ceiling`: for every body, the longitude
    difference between the default at `START` and the default at `START − 1e-6`
    days is within that body's ceiling plus 1e-4″. The same check applies at
    `END − 1e-6` against `END`. Use 1e-6 rather than anything smaller: a JD near
    2.3e6 has an ulp of about 5e-10 days.
  - `motion_uses_the_request_fidelity_on_both_stencil_points`: at
    `START + 0.001` (so the −0.005 day stencil point falls outside the window),
    Mars' default longitude speed is within 1e-9 °/day of the central
    difference of `geocentric_coordinates_on(…, SeriesFidelity::Truncated)` at
    ±0.005 day, converted with the backend's own ecliptic conversion. Make
    `to_ecliptic` `pub(crate)` if needed. A mixed stencil would be off by the
    edge step divided by 0.01 day. This is white-box, through
    `Vsop87Backend::motion`.
  - `full_series_backend_reports_exact_and_default_reports_approximate_inside_the_window`:
    - At J2000, Mars reports `Approximate` on the default and `Exact` on the
      full type.
    - At JD 625_673.5, Mars reports `Exact` on both.
  - `full_series_backend_metadata_matches_the_default_except_the_truncation_note`:
    - The provenance summaries differ only by the sentence above.
    - Every other field is equal.
  - `full_series_backend_composes`: `CompositeBackend::new(ElpBackend::new(), Vsop87FullSeriesBackend::new())`
    returns Mars at J2000 equal to the full type's own result. If ELP is not a
    dev-dependency of pleiades-vsop87, put this test in
    `crates/pleiades-core/src/chart/` next to the other composite tests
    instead of adding the dependency.

- [ ] **Step 2: Run the tests to verify they fail.**
  Run: `cargo test -p pleiades-vsop87 series_truncation`
  Expected: compile errors (`Vsop87FullSeriesBackend`, window constants missing).

- [ ] **Step 3: Implement** the window constants, `SeriesPolicy`, the fidelity
  threading, `Vsop87FullSeriesBackend`, quality and metadata, as the
  Interfaces block specifies.

- [ ] **Step 4: Run the drift test with output, and fill in the ceilings.**
  Run: `cargo test -p pleiades-vsop87 truncation_drift -- --nocapture`
  Expected: the per-body maxima print. If any is over 0.27″, stop and report.
  Otherwise fill `TRUNCATION_DRIFT_CEILINGS_ARCSEC` by the rule and re-run.
  Expected: PASS.

- [ ] **Step 5: Run the crate's tests.**
  Run: `cargo test -p pleiades-vsop87`
  Expected: the new tests pass. Pre-existing tests that pin
  `Vsop87Backend::new()` numbers or `Exact` quality inside 1600–2600 may now
  fail. Handle them by these rules:
  - A test that asserts reproduction of the VSOP87B source (canonical epoch
    evidence, batch parity, generated-table audits in `src/source_docs/`):
    switch it to `Vsop87FullSeriesBackend`, values unchanged.
  - A test that pins the default backend's chart-facing output or quality:
    update the expected value to the new output, and confirm the change is
    within the drift ceiling.
  - List every test you changed, and which rule applied, in the commit body.

- [ ] **Step 6: Commit.**
  ```bash
  cargo fmt --all
  git add crates/pleiades-vsop87/src
  git commit -m "perf(vsop87): truncate the VSOP87B series inside 1600–2600 (#281)"
  ```
  The body records the per-body drift maxima from Step 4 and the changed-test
  list.

### Task 3: Crate docs, README and generated capability blocks

**Files:**
- Modify: `crates/pleiades-vsop87/src/lib.rs` (crate rustdoc), `crates/pleiades-vsop87/README.md`
- Modify: whatever `mise run readme-capabilities` regenerates
- Modify: any `spec/*.md` or `docs/*.md` sentence that says the default
  backend evaluates the full, untruncated series. Find them with:
  `grep -rn -i "untruncated\|full.*VSOP87\|VSOP87.*full" spec docs crates/*/README.md crates/pleiades-vsop87/src --include=*.md --include=*.rs`.
  Skip `docs/superpowers/` history.

- [ ] **Step 1: Update the crate rustdoc and the README.** Describe the window,
  the rule, the drift ceiling, the edge step and `Vsop87FullSeriesBackend`.
  Add a rustdoc example that builds both backends and shows they agree outside
  the window (`==` at JD 625_673.5) and differ inside it.

- [ ] **Step 2: Regenerate the capability blocks.**
  Run: `mise run readme-capabilities`, then `mise run readme-capabilities -- --check`
  Expected: the check passes. The diff touches only the VSOP87 capability text.

- [ ] **Step 3: Run the docs and the doctests.**
  Run: `mise run docs && cargo test -p pleiades-vsop87 --doc`
  Expected: PASS with no rustdoc warnings. Check that no intra-doc link points
  from public to private items.

- [ ] **Step 4: Commit.**
  ```bash
  cargo fmt --all
  git add -A crates spec docs ':!docs/superpowers'
  git commit -m "docs(vsop87): document the truncated series window and the full-series backend (#281)"
  ```

### Task 4: Workspace fallout

**Files:** whatever the tiers below show to be affected. The likely ones are
`crates/pleiades-validate/src/{pheno,nod_aps,occult,frame_consistency}_validation.rs`,
`crates/pleiades-validate/src/comparison/report.rs` and
`crates/pleiades-core/src/chart/*_tests.rs`.

- [ ] **Step 1: Run the blocking tier.**
  Run: `mise run ci`
  Expected: PASS, or a list of failures to triage in Step 3.

- [ ] **Step 2: Run the slow tier and the SE-comparison gates that use the
  composite.**
  Run, with the kernel variables unset:
  - `mise run test-full`
  - `cargo run -q -p pleiades-validate -- validate-pheno`
  - `cargo run -q -p pleiades-validate -- validate-nod-aps`
  - `cargo run -q -p pleiades-validate -- validate-occultations`
  - `cargo run -q -p pleiades-validate -- compare-backends-audit`

  Expected: PASS, or failures to triage.

- [ ] **Step 3: Triage each failure by the design's rules.** Don't loosen
  anything beyond these:
  - **A check of the default backend against Swiss Ephemeris, DE440 or JPL**
    stays on the default backend. It may be widened by no more than the
    measured drift for that body (Task 2, Step 4). Put a comment at the
    tolerance naming #281 and the drift figure, and list it in the PR. Never
    switch such a check to `Vsop87FullSeriesBackend`.
  - **A check that asserts reproduction of the VSOP87B source files** moves to
    `Vsop87FullSeriesBackend`.
  - **A golden or corpus that pins pleiades' own output:** regenerate it with
    its documented command, in this same PR.
  - **A failure that the drift cannot explain** (larger than the drift ceiling,
    or a body or date outside 1600–2600): stop and report it. It is a bug, not
    a tolerance.

- [ ] **Step 4: Re-run Steps 1–2 until everything is green.** Commit each
  coherent fix separately, for example
  `test(validate): allow #281 truncation drift in the occultation contact gate`.

### Task 5: Measure and report

**Files:** none committed. The probe lives in the session scratchpad (an
empty-`[workspace]` scratch project with path dependencies, run with
`--offline --release`).

- [ ] **Step 1: Measure the per-sample cost** on `main` (before) and on this
  branch (after). Use starfold's shape:
  - `EventEngine::new(CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new()))`
  - `longitude_at` for Sun through Pluto
  - 4,000 instants, `scattered` across 1900–2100 and `daily` from J2000
  - single thread, best of 3

  Wait for `/proc/loadavg` below 3, alternate the before and after runs, and
  record the load. Expected: roughly 40–100 µs per planet after, against
  130–330 µs before. Report what you measure, not this expectation.

- [ ] **Step 2: Measure the change in returned apparent longitudes** against
  current output. Compare `longitude_at` on `main` and on this branch over the
  same instants, per body: max and RMS in arcseconds over 1900–2100 and over
  1600–2600.

- [ ] **Step 3: Write the PR description.**
  - Title: `perf(vsop87): truncate the VSOP87B series inside 1600–2600 (#281)`.
  - Include the cost table and the longitude-change table.
  - Include the drift ceilings.
  - List every tolerance widened in Task 4, with its justification.
  - Point to `Vsop87FullSeriesBackend` for anyone who needs today's output.
  - Note the `Exact` → `Approximate` quality change inside the window.
  - Add a closing line, `Closes #281`.

  Don't open the PR or comment on the issue without the maintainer's go-ahead.
