# Asteroid Positions Fail Closed Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** No published backend returns an asteroid position away from real sample rows; the caller gets a typed error that names `SpkBackend`.

**Architecture:** `JplSnapshotBackend::position` gains a stencil guard (rows on both sides, at most 5 days wide). `PackagedDataBackend` stops serving `asteroid:433-Eros`, so Eros routes to the guarded snapshot backend. The artifact generator and fit check move to the unguarded `SnapshotCorpusBackend` over the same rows, so the committed artifact does not change. Four placeholder Apophis rows are deleted.

**Tech Stack:** Rust (stable, toolchain from `mise.toml`), `cargo nextest`, `mise run ci`, `gh`.

**Spec:** `docs/superpowers/specs/2026-10-06-asteroid-fail-closed-design.md`

## Global Constraints

- Work only in the worktree `/workspace/.claude/worktrees/asteroid-fail-closed` (branch `worktree-asteroid-fail-closed`). Another agent shares `/workspace`.
- The committed artifact `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`, its checksum and `packaged_bodies()` must not change.
- `MAX_STENCIL_SPAN_DAYS` is `5.0`. The error kind is `EphemerisErrorKind::OutOfRangeInstant`; no error variant is added.
- The served-or-refused tolerance is 5″ in longitude (scaled by cos latitude) and in latitude.
- A pinned string or count that changes is updated to the newly measured value. It is never loosened, deleted or turned into a substring match.
- No new dependencies. No `unwrap`/`expect` in library paths.
- Run `cargo fmt --all` before every commit.
- Run cargo commands in the foreground. Make no edits and no commits while a test run is in progress.
- Do not bump versions or edit changelogs; release-plz does that.

## Review Focus

1. A request tagged TT at an exact row's Julian day (the backend compares the raw JD): it must still be served `Exact`. Test in Task 3.
2. A batch `positions` call where one request is refused: the batch must return `OutOfRangeInstant`, not a partial or a panic. Test in Task 3.
3. An equatorial-frame request away from the rows: the frame must not bypass the guard. Test in Task 3.
4. An instant a hair outside the cluster's last row (JD 2451919.5 + 1e-6): refused, because no row follows within the stencil. Test in Task 3.
5. `pleiades-cli stations --body Ceres` over a range: a readable error that names `SpkBackend`, not a panic or an empty list. Test in Task 5.

---

### Task 1: Generator and fit check read the rows through `SnapshotCorpusBackend`

A pure refactor, done first so the guard in Task 3 cannot change what the generator fits.

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs` (signatures at lines 208, 633, 910, 1237, 1331, 1872, 1987; call sites at 183 and 2451; import at 15)
- Modify: `crates/pleiades-data/src/coverage/threshold.rs:458-513`
- Modify: `crates/pleiades-data/src/tests/lookup.rs:693-721`, `crates/pleiades-data/src/tests/coverage.rs:2761-2773`
- Test: `crates/pleiades-data/src/tests/codec.rs`

**Interfaces:**
- Produces: `pub(crate) fn snapshot_fit_source() -> &'static SnapshotCorpusBackend` in `crate::regenerate`.

- [ ] **Step 1: Write the characterization test**

Add to `crates/pleiades-data/src/tests/codec.rs`, after `snapshot_reconstruction_covers_only_constrained_asteroids`:

```rust
/// The constrained asteroid's segments are fitted to the reference snapshot's
/// interpolation. This pins the generator's output for it to the committed
/// artifact, so a change in what the generator samples cannot pass unnoticed
/// (the kernel-free regeneration path returns the committed bytes and proves
/// nothing about the generator).
#[test]
fn snapshot_fit_of_the_constrained_asteroid_matches_the_committed_artifact() {
    use pleiades_backend::{CelestialBody, CustomBodyId};
    let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
    let regenerated = try_regenerate_packaged_artifact_from_snapshot(reference_snapshot())
        .expect("the reference snapshot should regenerate");
    let fitted = regenerated
        .bodies
        .iter()
        .find(|series| series.body == eros)
        .expect("the snapshot fit should hold the constrained asteroid");
    let committed = crate::data::packaged_artifact()
        .bodies
        .iter()
        .find(|series| series.body == eros)
        .expect("the committed artifact should hold the constrained asteroid");
    assert_eq!(fitted, committed);
}
```

- [ ] **Step 2: Run it on the unchanged code**

Run: `cargo nextest run -p pleiades-data snapshot_fit_of_the_constrained_asteroid`
Expected: PASS.

If it FAILS, the committed Eros segments were produced by another path. Then replace the final `assert_eq!` with a pinned digest of the generator's own output: add `eprintln!("{}", regenerated.checksum);` temporarily, run with `--no-capture`, and assert `assert_eq!(regenerated.checksum, <printed value>);` with a comment giving the date measured. Remove the `eprintln!`. Either way the test must pass before Step 3 and must not be edited again in this plan.

- [ ] **Step 3: Add the unguarded source and switch the generator to it**

In `crates/pleiades-data/src/regenerate.rs`, change the `pleiades_jpl` import to bring in `SnapshotCorpusBackend` and drop `JplSnapshotBackend`, then add:

```rust
/// The reference snapshot rows behind a backend that interpolates them
/// without refusing.
///
/// The constrained asteroid's segments are fitted to the snapshot's cubic
/// between rows that can lie decades apart. `JplSnapshotBackend::position`
/// refuses such a request (issue #158), so the generator and the fit-envelope
/// check read the same rows through [`SnapshotCorpusBackend`], which runs the
/// same interpolation and never refuses. Nothing fitted here is served: the
/// packaged backend declines the bodies fitted this way.
pub(crate) fn snapshot_fit_source() -> &'static SnapshotCorpusBackend {
    static SOURCE: OnceLock<SnapshotCorpusBackend> = OnceLock::new();
    SOURCE.get_or_init(|| SnapshotCorpusBackend::from_entries(reference_snapshot().to_vec()))
}
```

Add `use std::sync::OnceLock;` if the file does not already import it.

In the seven functions `body_segments_from_entries`, `packaged_artifact_segment_fit_error`, `body_segment_windows_for_interval`, `segment_from_pair`, `segment_from_pair_fit_attempt`, `segment_with_optional_residual_channels` and `residual_segment`, change the parameter type:

```rust
    reference_backend: &SnapshotCorpusBackend,
```

At line 183 replace `let reference_backend = JplSnapshotBackend;` and its use:

```rust
                let segments = body_segments_from_entries(&entries, snapshot_fit_source());
```

At line 2451:

```rust
                    let segments = body_segments_from_entries(&entries, snapshot_fit_source());
```

- [ ] **Step 4: Switch the fit check**

In `crates/pleiades-data/src/coverage/threshold.rs`, change the struct field and its construction:

```rust
struct FitTruthBackend {
    corpus: SnapshotCorpusBackend,
    snapshot: &'static SnapshotCorpusBackend,
}
```

```rust
        FitTruthBackend {
            corpus: SnapshotCorpusBackend::from_entries(entries),
            snapshot: crate::regenerate::snapshot_fit_source(),
        }
```

In the doc comment above the struct, replace "`JplSnapshotBackend`, the exact source they were fit against" with "the reference snapshot rows through `snapshot_fit_source`, the exact source they were fit against". Remove `JplSnapshotBackend` from the imports in `coverage/mod.rs` and `lib.rs` if the compiler reports them unused.

- [ ] **Step 5: Switch the two test call sites**

`crates/pleiades-data/src/tests/lookup.rs:693`:

```rust
    let reference_backend = crate::regenerate::snapshot_fit_source();
```

and at line 721 pass `reference_backend` (not `&reference_backend`).

`crates/pleiades-data/src/tests/coverage.rs:2761-2773`: drop `JplSnapshotBackend` from the `use`, and call

```rust
                    crate::regenerate::body_segments_from_entries(
                        &e,
                        crate::regenerate::snapshot_fit_source(),
                    )
                    .len();
```

- [ ] **Step 6: Run the crate's tests**

Run: `cargo nextest run -p pleiades-data`
Expected: PASS, including the Step 1 test and every pinned fit constant in `coverage/fit.rs` unchanged.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add crates/pleiades-data
git commit -m "refactor(data): fit the constrained asteroid from the snapshot rows through SnapshotCorpusBackend (#158)"
```

---

### Task 2: Remove the placeholder Apophis rows

**Files:**
- Modify: `crates/pleiades-jpl/data/reference_snapshot.csv` (lines 121, 240, 242, 282 and the header)
- Modify: `crates/pleiades-jpl/data/independent_holdout_snapshot.csv` (lines 57, 71 and the header)
- Test: `crates/pleiades-jpl/src/backend/tests.rs`
- Modify: pinned literals in `crates/pleiades-jpl`, `crates/pleiades-validate`, `crates/pleiades-cli`, `crates/pleiades-data` tests that follow from the row counts

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-jpl/src/backend/tests.rs`:

```rust
/// A geocentric vector whose x and z are both within 1000 km of zero is a
/// placeholder, not a position. Four Apophis rows once were (each read as
/// longitude 89.9998°, latitude 0.000°) and were served as `Exact`.
#[test]
fn snapshot_fixtures_hold_no_placeholder_rows() {
    let fixtures = [
        ("reference_snapshot", reference_snapshot()),
        (
            "independent_holdout_snapshot",
            independent_holdout_snapshot_entries().expect("the hold-out snapshot should load"),
        ),
    ];
    for (label, entries) in fixtures {
        assert!(!entries.is_empty(), "{label} should hold rows");
        for entry in entries {
            assert!(
                entry.x_km.abs() >= 1000.0 || entry.z_km.abs() >= 1000.0,
                "{label}: {} at JD {} is a placeholder row",
                entry.body,
                entry.epoch.julian_day.days()
            );
        }
    }
}
```

- [ ] **Step 2: Run it**

Run: `cargo nextest run -p pleiades-jpl snapshot_fixtures_hold_no_placeholder_rows`
Expected: FAIL naming `asteroid:99942-Apophis at JD 2451915`.

- [ ] **Step 3: Delete the rows**

In `reference_snapshot.csv` delete the four lines beginning:

```
2451915.000000000,asteroid:99942-Apophis,4.591746675914950E+02,
2500000.000000000,asteroid:99942-Apophis,4.661193440371725E+02,
2634167.000000000,asteroid:99942-Apophis,4.573135904529489E+02,
2453000.500000000,asteroid:99942-Apophis,4.185864453791026E+02,
```

In `independent_holdout_snapshot.csv` delete the two lines beginning:

```
2500000.000000000,asteroid:99942-Apophis,4.661193440371725E+02,
2634167.000000000,asteroid:99942-Apophis,457.3135904529489,
```

Read each file's `# Coverage:` header. Where it lists Apophis at a removed epoch, or gives a total row count, correct it to match the file (the hold-out header says "66 rows across 16 bodies and 12 epochs"; it becomes 64 rows, and the epoch count stays 12 only if another body still has rows at 2500000 and 2634167 — count, do not assume).

- [ ] **Step 4: Run the new test**

Run: `cargo nextest run -p pleiades-jpl snapshot_fixtures_hold_no_placeholder_rows`
Expected: PASS.

- [ ] **Step 5: Update the pinned values that follow**

Run: `cargo nextest run -p pleiades-jpl --no-fail-fast`

For each failure, read the assertion diff. Accept a change only if it is one of:
- a sample or row count lower by the number of Apophis rows removed from that file (4 or 2);
- an Apophis line in a per-body report (epochs, windows, leave-one-out samples);
- a checksum of one of the two CSV files;
- the epoch count or range of the Apophis body.

Update the pinned literal to the new value. Any other difference is a bug in this task: stop and investigate.

Repeat for the crates that copy these reports:

Run: `cargo nextest run -p pleiades-data -p pleiades-cli --no-fail-fast`
Run: `cargo nextest run -p pleiades-validate --no-fail-fast`

Apply the same acceptance rule. `pleiades-validate` is slow; run it once, fix every reported literal, then run it once more to confirm.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates
git commit -m "fix(jpl): remove four placeholder Apophis rows from the snapshot fixtures (#158)"
```

---

### Task 3: Stencil guard in `JplSnapshotBackend::position`

**Files:**
- Modify: `crates/pleiades-jpl/src/backend.rs` (`position` at 210-247, `interpolate_fixture_state` at 1412-1494, module list at 2384)
- Create: `crates/pleiades-jpl/src/backend/stencil_guard_tests.rs`
- Modify: `crates/pleiades-jpl/src/test_support.rs`
- Modify: `crates/pleiades-events/tests/nod_aps.rs:226-249`
- Modify: `crates/pleiades-jpl/README.md`

**Interfaces:**
- Produces: `pub const MAX_STENCIL_SPAN_DAYS: f64 = 5.0;` (re-exported from `pleiades_jpl`), `fn interpolation_stencil<'a>(entries: &'a [SnapshotEntry], body: &CelestialBody, epoch_jd: f64) -> Vec<&'a SnapshotEntry>`, `fn stencil_supports(entries: &[SnapshotEntry], body: &CelestialBody, epoch_jd: f64) -> bool`.
- Produces for tests: `pub(crate) fn mean_request(body: CelestialBody, julian_day: f64) -> EphemerisRequest` in `crate::test_support`.

- [ ] **Step 1: Add the request helper**

Append to `crates/pleiades-jpl/src/test_support.rs`:

```rust
/// A mean, tropical, geocentric ecliptic request at a TDB Julian day.
pub(crate) fn mean_request(body: CelestialBody, julian_day: f64) -> EphemerisRequest {
    EphemerisRequest {
        body,
        instant: Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: pleiades_types::ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    }
}
```

- [ ] **Step 2: Write the failing tests**

Create `crates/pleiades-jpl/src/backend/stencil_guard_tests.rs`:

```rust
//! The snapshot backend answers only where its rows support the answer
//! (issue #158).

use super::*;
use crate::test_support::mean_request;
use crate::{asteroid_reference_corpus, reference_snapshot};
use pleiades_backend::{
    CelestialBody, CustomBodyId, EphemerisBackend, EphemerisErrorKind, QualityAnnotation,
};
use pleiades_types::{CoordinateFrame, TimeScale};

/// Largest disagreement allowed between a served position and the
/// `sb441-n373s` row. At JD 2453000.5 the Horizons row and the kernel row
/// differ by up to 1.0″ in longitude and 1.9″ in latitude (measured
/// 2026-10-06).
const TRUTH_TOLERANCE_ARCSEC: f64 = 5.0;
/// The one epoch a snapshot row and a truth row share.
const SHARED_EPOCH_JD: f64 = 2_453_000.5;
/// Between J2000 and the January 2001 cluster.
const MID_GAP_JD: f64 = 2_451_700.0;
const CLUSTER_FIRST_JD: f64 = 2_451_910.5;
const CLUSTER_LAST_JD: f64 = 2_451_919.5;

fn eros() -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"))
}

fn apophis() -> CelestialBody {
    CelestialBody::Custom(CustomBodyId::new("asteroid", "99942-Apophis"))
}

fn truth_bodies() -> [CelestialBody; 5] {
    [
        CelestialBody::Ceres,
        CelestialBody::Pallas,
        CelestialBody::Juno,
        CelestialBody::Vesta,
        eros(),
    ]
}

/// Longitude (scaled by cos latitude) and latitude differences, arcseconds.
fn separation_arcsec(got: &SnapshotEntry, truth: &SnapshotEntry) -> (f64, f64) {
    let (got, truth) = (got.ecliptic(), truth.ecliptic());
    let longitude = angular_degrees_delta(got.longitude.degrees(), truth.longitude.degrees())
        * truth.latitude.degrees().to_radians().cos()
        * 3600.0;
    let latitude = (got.latitude.degrees() - truth.latitude.degrees()).abs() * 3600.0;
    (longitude, latitude)
}

#[test]
fn every_truth_epoch_is_served_within_tolerance_or_refused() {
    let backend = JplSnapshotBackend;
    for body in truth_bodies() {
        let mut served = Vec::new();
        let mut refused = 0_usize;
        for row in asteroid_reference_corpus().iter().filter(|row| row.body == body) {
            let jd = row.epoch.julian_day.days();
            match backend.position(&mean_request(body.clone(), jd)) {
                Ok(result) => {
                    let got = result.ecliptic.expect("a served row has ecliptic coordinates");
                    let truth = row.ecliptic();
                    let longitude =
                        angular_degrees_delta(got.longitude.degrees(), truth.longitude.degrees())
                            * truth.latitude.degrees().to_radians().cos()
                            * 3600.0;
                    let latitude =
                        (got.latitude.degrees() - truth.latitude.degrees()).abs() * 3600.0;
                    assert!(
                        longitude <= TRUTH_TOLERANCE_ARCSEC && latitude <= TRUTH_TOLERANCE_ARCSEC,
                        "{body} at JD {jd}: served {longitude:.3}″ / {latitude:.3}″ from truth"
                    );
                    served.push(jd);
                }
                Err(error) => {
                    assert_eq!(
                        error.kind,
                        EphemerisErrorKind::OutOfRangeInstant,
                        "{body} at JD {jd}: {error}"
                    );
                    refused += 1;
                }
            }
        }
        assert_eq!(served, vec![SHARED_EPOCH_JD], "{body}: served epochs");
        assert_eq!(refused, 406, "{body}: refused epochs");
    }
}

#[test]
fn a_request_between_j2000_and_the_cluster_is_refused_for_every_asteroid() {
    let backend = JplSnapshotBackend;
    for body in truth_bodies().into_iter().chain([apophis()]) {
        let error = backend
            .position(&mean_request(body.clone(), MID_GAP_JD))
            .expect_err("a mid-gap request must be refused");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant, "{body}");
        let message = error.to_string();
        assert!(message.contains("SpkBackend"), "{body}: {message}");
        assert!(message.contains("2451545"), "{body}: {message}");
    }
}

#[test]
fn a_request_inside_the_cluster_interpolates_between_its_neighbours() {
    let backend = JplSnapshotBackend;
    let longitude_at = |jd: f64| {
        let result = backend
            .position(&mean_request(CelestialBody::Ceres, jd))
            .expect("the cluster supports this instant");
        (
            result.ecliptic.expect("ecliptic").longitude.degrees(),
            result.quality,
        )
    };
    let (before, before_quality) = longitude_at(2_451_914.0);
    let (middle, middle_quality) = longitude_at(2_451_914.25);
    let (after, after_quality) = longitude_at(2_451_914.5);
    assert_eq!(before_quality, QualityAnnotation::Exact);
    assert_eq!(after_quality, QualityAnnotation::Exact);
    assert_eq!(middle_quality, QualityAnnotation::Interpolated);
    let chord = (before + after) / 2.0;
    assert!(
        (middle - chord).abs() * 3600.0 < 1.0,
        "Ceres at the half-day point: {middle} against the chord {chord}"
    );
}

#[test]
fn a_request_just_outside_the_cluster_is_refused() {
    let backend = JplSnapshotBackend;
    for jd in [
        CLUSTER_FIRST_JD - 0.1,
        CLUSTER_LAST_JD + 0.1,
        CLUSTER_LAST_JD + 1e-6,
    ] {
        let error = backend
            .position(&mean_request(CelestialBody::Ceres, jd))
            .expect_err("no row brackets this instant within the stencil");
        assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant, "JD {jd}");
    }
}

#[test]
fn a_light_time_step_off_an_isolated_row_is_refused() {
    let error = JplSnapshotBackend
        .position(&mean_request(CelestialBody::Ceres, 2_451_545.0 - 0.013))
        .expect_err("the J2000 row stands alone");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn an_exact_row_is_served_whatever_its_time_scale_tag() {
    let mut request = mean_request(CelestialBody::Ceres, 2_451_545.0);
    request.instant.scale = TimeScale::Tt;
    let result = JplSnapshotBackend
        .position(&request)
        .expect("an exact row is served");
    assert_eq!(result.quality, QualityAnnotation::Exact);
}

#[test]
fn the_frame_does_not_bypass_the_guard() {
    let mut request = mean_request(CelestialBody::Ceres, MID_GAP_JD);
    request.frame = CoordinateFrame::Equatorial;
    let error = JplSnapshotBackend
        .position(&request)
        .expect_err("an equatorial request is refused like an ecliptic one");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn a_batch_holding_a_refused_request_returns_the_refusal() {
    let served = mean_request(CelestialBody::Ceres, 2_451_545.0);
    let refused = mean_request(CelestialBody::Ceres, MID_GAP_JD);
    let error = JplSnapshotBackend
        .positions(&[served, refused])
        .expect_err("the batch must not return a partial result");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
}

#[test]
fn a_request_past_the_last_row_keeps_its_message() {
    let error = JplSnapshotBackend
        .position(&mean_request(CelestialBody::Ceres, 2_634_168.0))
        .expect_err("past the last row");
    assert_eq!(error.kind, EphemerisErrorKind::OutOfRangeInstant);
    assert!(error
        .to_string()
        .contains("outside adjacent JPL fixture samples"));
}

/// Holds every row out in turn. Where the remaining rows pass the guard, the
/// interpolation must reproduce the held-out row. This is the measurement
/// behind `MAX_STENCIL_SPAN_DAYS`.
#[test]
fn stencils_the_guard_admits_reproduce_held_out_rows() {
    let entries = reference_snapshot();
    let mut asteroid_cases = 0_usize;
    let mut worst_asteroid = 0.0_f64;
    let mut worst_other = 0.0_f64;
    for held_out in entries {
        let jd = held_out.epoch.julian_day.days();
        let rest = entries
            .iter()
            .filter(|entry| entry.body != held_out.body || entry.epoch.julian_day.days() != jd)
            .cloned()
            .collect::<Vec<_>>();
        if !stencil_supports(&rest, &held_out.body, jd) {
            continue;
        }
        let interpolated = interpolate_fixture_state(&rest, held_out.body.clone(), jd)
            .expect("an admitted stencil interpolates");
        let (longitude, latitude) = separation_arcsec(&interpolated, held_out);
        let worst = longitude.max(latitude);
        let is_asteroid = truth_bodies().contains(&held_out.body) || held_out.body == apophis();
        if is_asteroid {
            asteroid_cases += 1;
            worst_asteroid = worst_asteroid.max(worst);
        } else {
            worst_other = worst_other.max(worst);
        }
    }
    assert!(asteroid_cases >= 40, "only {asteroid_cases} asteroid cases");
    assert!(
        worst_asteroid <= ADMITTED_ASTEROID_CEILING_ARCSEC,
        "asteroids: {worst_asteroid:.4}″"
    );
    assert!(
        worst_other <= ADMITTED_MAJOR_BODY_CEILING_ARCSEC,
        "major bodies: {worst_other:.4}″"
    );
}

/// Measured 2026-10-06: see the note in Step 6 of the plan for the value.
const ADMITTED_ASTEROID_CEILING_ARCSEC: f64 = 0.05;
/// Set from the measurement in Step 6.
const ADMITTED_MAJOR_BODY_CEILING_ARCSEC: f64 = 0.05;
```

Declare the module in `crates/pleiades-jpl/src/backend.rs`, below `mod tests;`:

```rust
#[cfg(test)]
mod stencil_guard_tests;
```

- [ ] **Step 3: Run them**

Run: `cargo nextest run -p pleiades-jpl stencil_guard_tests`
Expected: does not compile (`stencil_supports` undefined). That is the red state. After Step 4 compiles, `every_truth_epoch_is_served_within_tolerance_or_refused` run against the unguarded `position` is the re-measurement the spec asks for; record its failure message in the commit body if you run it before wiring the guard.

- [ ] **Step 4: Implement the stencil and the guard**

In `crates/pleiades-jpl/src/backend.rs`, replace `interpolate_fixture_state` (lines 1412-1494) with a stencil selector and an interpolation built on it:

```rust
/// Widest stencil, in days, that [`JplSnapshotBackend`] interpolates across.
///
/// The backend answers between rows only where the rows it would use lie on
/// both sides of the instant and span no more than this. Holding each row out
/// in turn, every stencil this admits reproduces the held-out asteroid row
/// within 0.05″ (`stencils_the_guard_admits_reproduce_held_out_rows`). The
/// next wider bracket in the snapshot is a year, where the same cubic is
/// wrong by tens of degrees (issue #158).
pub const MAX_STENCIL_SPAN_DAYS: f64 = 5.0;

/// The rows an interpolation at `epoch_jd` uses for `body`, ascending: the
/// four nearest in time, or all three when the body has exactly three. Empty
/// when the body has fewer than three rows.
fn interpolation_stencil<'a>(
    entries: &'a [SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Vec<&'a SnapshotEntry> {
    let epoch_of = |entry: &SnapshotEntry| entry.epoch.julian_day.days();
    let mut ranked = entries
        .iter()
        .filter(|entry| &entry.body == body)
        .map(|entry| ((epoch_of(entry) - epoch_jd).abs(), entry))
        .collect::<Vec<_>>();
    if ranked.len() < 3 {
        return Vec::new();
    }
    ranked.sort_by(|left, right| {
        left.0
            .partial_cmp(&right.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                epoch_of(left.1)
                    .partial_cmp(&epoch_of(right.1))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let window_size = if ranked.len() >= 4 { 4 } else { 3 };
    let mut stencil = ranked
        .into_iter()
        .take(window_size)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    stencil.sort_by(|left, right| {
        epoch_of(left)
            .partial_cmp(&epoch_of(right))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    stencil
}

pub(crate) fn interpolate_fixture_state(
    entries: &[SnapshotEntry],
    body: pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<SnapshotEntry> {
    match interpolation_stencil(entries, &body, epoch_jd).as_slice() {
        &[a, b, c, d] => Some(SnapshotEntry::interpolate_cubic(a, b, c, d, epoch_jd)),
        &[a, b, c] => Some(SnapshotEntry::interpolate_quadratic(a, b, c, epoch_jd)),
        _ => None,
    }
}

/// First and last epoch of the rows an interpolation at `epoch_jd` rests on:
/// the stencil, or the two adjacent rows when the body has fewer than three.
fn stencil_bounds(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Option<(f64, f64)> {
    let stencil = interpolation_stencil(entries, body, epoch_jd);
    if let (Some(first), Some(last)) = (stencil.first(), stencil.last()) {
        return Some((first.epoch.julian_day.days(), last.epoch.julian_day.days()));
    }
    let (before, after) = adjacent_epochs(entries, body, epoch_jd);
    Some((before?, after?))
}

/// The body's nearest row epoch before `epoch_jd` and nearest after it.
fn adjacent_epochs(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> (Option<f64>, Option<f64>) {
    let epochs = || {
        entries
            .iter()
            .filter(|entry| &entry.body == body)
            .map(|entry| entry.epoch.julian_day.days())
    };
    (
        epochs().filter(|jd| *jd < epoch_jd).reduce(f64::max),
        epochs().filter(|jd| *jd > epoch_jd).reduce(f64::min),
    )
}

/// Whether the rows nearest `epoch_jd` support an interpolation there: they
/// lie on both sides of it and span at most [`MAX_STENCIL_SPAN_DAYS`].
fn stencil_supports(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> bool {
    stencil_bounds(entries, body, epoch_jd).is_some_and(|(first, last)| {
        first < epoch_jd && epoch_jd < last && last - first <= MAX_STENCIL_SPAN_DAYS
    })
}

/// Refuses an interpolation the rows cannot support (issue #158).
fn require_supported_stencil(
    entries: &[SnapshotEntry],
    body: &pleiades_backend::CelestialBody,
    epoch_jd: f64,
) -> Result<(), EphemerisError> {
    if stencil_supports(entries, body, epoch_jd) {
        return Ok(());
    }
    let describe = |epoch: Option<f64>| match epoch {
        Some(jd) => format!("JD {jd}"),
        None => "none".to_string(),
    };
    let (before, after) = adjacent_epochs(entries, body, epoch_jd);
    Err(EphemerisError::new(
        EphemerisErrorKind::OutOfRangeInstant,
        format!(
            "the JPL snapshot has no rows close enough to JD {epoch_jd} to interpolate {body} \
             (nearest row before: {}; nearest after: {}); it interpolates only between rows on \
             both sides of an instant that span at most {MAX_STENCIL_SPAN_DAYS} days. Serve \
             {body} at this instant from pleiades_jpl::SpkBackend with a JPL kernel",
            describe(before),
            describe(after)
        ),
    ))
}
```

In `JplSnapshotBackend::position`, replace the `let resolved = …` line with:

```rust
        let epoch_jd = req.instant.julian_day.days();
        let resolved = resolve_fixture_state(req.body.clone(), epoch_jd)?;
        if resolved.quality != QualityAnnotation::Exact {
            if let Some(entries) = snapshot_entries() {
                require_supported_stencil(entries, &req.body, epoch_jd)?;
            }
        }
```

The resolver runs first so an unsupported body and an instant past the last row keep their existing errors. `SnapshotCorpusBackend::position` and `resolve_fixture_state_from_entries` are not changed.

Add to the `JplSnapshotBackend` struct's doc comment:

```rust
///
/// A request is answered at an exact row, or between rows that lie on both
/// sides of it and span at most [`MAX_STENCIL_SPAN_DAYS`]. Any other instant
/// returns `EphemerisErrorKind::OutOfRangeInstant`: the rows are too sparse
/// to interpolate across (issue #158). [`crate::SpkBackend`] serves these
/// bodies from a JPL kernel.
```

- [ ] **Step 5: Run the guard tests**

Run: `cargo nextest run -p pleiades-jpl stencil_guard_tests`
Expected: all PASS except possibly `stencils_the_guard_admits_reproduce_held_out_rows` on the major-body ceiling.

- [ ] **Step 6: Set the two ceilings from the measurement**

Temporarily add `eprintln!("asteroid {worst_asteroid} other {worst_other} cases {asteroid_cases}");` before the assertions and run:

Run: `cargo nextest run -p pleiades-jpl stencils_the_guard_admits --no-capture`

- Asteroids: the spec expects about 0.005″. If the measured value is at or under 0.05″, keep `ADMITTED_ASTEROID_CEILING_ARCSEC = 0.05` and write the measured value into its doc comment. If it is over 0.05″, stop: the cap of 5 days is not supported and the spec must be revisited with the user.
- Major bodies: set `ADMITTED_MAJOR_BODY_CEILING_ARCSEC` to the measured value times 1.5, rounded up to two significant figures, and write the measured value and the body it comes from into its doc comment. Add one sentence with that figure to the `MAX_STENCIL_SPAN_DAYS` doc comment.

Remove the `eprintln!`. Replace both "see the note in Step 6 of the plan" comments with the measured figures.

- [ ] **Step 7: Update the Ceres nod_aps test**

In `crates/pleiades-events/tests/nod_aps.rs`, replace the comment and body of `snapshot_only_asteroids_fail_closed`:

```rust
/// Ceres (and the other JPL-snapshot-only selected asteroids) is served by no
/// continuous backend in the production chain — only `JplSnapshotBackend`,
/// which answers at its sample rows and refuses an instant its rows cannot
/// support (issue #158). nod_aps samples a fraction of a day either side of
/// the query, off the J2000 row, so the engine fails closed with the
/// backend's refusal. SE small-body parity here is a documented coverage
/// bound (issues #158 and #160).
#[test]
fn snapshot_only_asteroids_fail_closed() {
    let engine = engine();
    let err = engine
        .nod_aps(
            CelestialBody::Ceres,
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap_err();
    assert!(
        matches!(err, EventError::Backend { .. }),
        "expected the backend's refusal, got: {err:?}"
    );
    assert!(err.to_string().contains("SpkBackend"), "{err}");
}
```

If `EventError::Backend`'s `Display` does not include the backend message, assert on the inner error's kind instead (`EphemerisErrorKind::OutOfRangeInstant`), reading the variant's definition in `crates/pleiades-events/src/error.rs`.

- [ ] **Step 8: Find every other consumer the guard reaches**

Run: `cargo nextest run -p pleiades-jpl -p pleiades-data -p pleiades-events -p pleiades-core -p pleiades-cli --no-fail-fast`
Run: `cargo nextest run -p pleiades-validate --no-fail-fast`

For each failure decide which case it is:
- **A test or report that asked the snapshot backend for an instant its rows cannot support.** If it is internal fitting or transparency code, route it through `SnapshotCorpusBackend::from_entries(reference_snapshot().to_vec())`. If it is a user-facing path, the refusal is the intended behaviour: update the expectation and say so in the test comment.
- **The packaged Eros test `packaged_fit_asteroids_fail_closed`** still passes here (the packaged backend still serves Eros); it changes in Task 5.
- **Anything else** is a bug in this task: stop and investigate.

- [ ] **Step 9: README**

In `crates/pleiades-jpl/README.md`, after the sentence that calls the checked-in corpus sparse regression evidence, add:

```markdown
`JplSnapshotBackend` answers at an exact fixture row, or between rows that lie
on both sides of the instant and span at most five days. Any other instant
returns `OutOfRangeInstant`. Use `SpkBackend` with a JPL kernel for positions
at arbitrary dates.
```

- [ ] **Step 10: Commit**

```bash
cargo fmt --all
git add crates
git commit -m "fix(jpl): refuse snapshot positions the fixture rows cannot support (#158)"
```

---

### Task 4: `PackagedDataBackend` stops serving Eros

**Files:**
- Modify: `crates/pleiades-data/src/lib.rs:181-204`
- Modify: `crates/pleiades-data/src/backend.rs` (`metadata` 369-431, `supports_body` 433-447, `position` 449-512)
- Modify: `crates/pleiades-data/src/coverage/threshold.rs:515-575`
- Modify: `crates/pleiades-data/src/tests/codec.rs:101-125`
- Modify: `crates/pleiades-validate/src/artifact/inspection.rs:421-423`
- Test: `crates/pleiades-data/src/tests/lookup.rs`

**Interfaces:**
- Produces: `pub(crate) fn is_carried_but_unserved(body: &CelestialBody) -> bool` in `pleiades_data` (crate root).

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-data/src/tests/lookup.rs` (reuse the file's existing imports; add any that are missing):

```rust
/// The artifact carries Eros segments fitted to rows decades apart. The
/// backend does not serve them (issue #158): it reports the body unsupported
/// so a routing chain moves on to a backend that can answer honestly.
#[test]
fn the_backend_does_not_serve_the_constrained_asteroid() {
    use pleiades_backend::{BodyClaimTier, EphemerisBackend, EphemerisErrorKind};
    let eros = CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros"));
    let backend = crate::PackagedDataBackend::new();

    assert!(crate::packaged_bodies().contains(&eros), "the artifact still carries it");
    assert!(!backend.supports_body(eros.clone()));

    let request = EphemerisRequest {
        body: eros.clone(),
        instant: instant_tt(2_451_545.0),
        observer: None,
        frame: CoordinateFrame::Ecliptic,
        zodiac_mode: ZodiacMode::Tropical,
        apparent: Apparentness::Mean,
    };
    let error = backend
        .position(&request)
        .expect_err("the constrained asteroid is not served");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody);
    assert!(error.to_string().contains("SpkBackend"), "{error}");

    let metadata = backend.metadata();
    assert!(metadata.claim_for(&eros).is_none());
    assert!(metadata
        .release_grade_bodies()
        .iter()
        .all(|body| body != &eros));
    assert_eq!(
        metadata.claim_for(&CelestialBody::Pluto).map(|claim| claim.tier),
        Some(BodyClaimTier::ReleaseGrade)
    );
}
```

If `release_grade_bodies()` returns claims and not bodies, compare on `.body`.

In `crates/pleiades-data/src/tests/codec.rs`, rename `packaged_metadata_claims_seventeen_bodies_release_grade` to `packaged_metadata_claims_sixteen_bodies_release_grade`, change the comment's first clause to "10 served artifact bodies", change `17` to `16`, and remove the Eros line from the loop.

- [ ] **Step 2: Run them**

Run: `cargo nextest run -p pleiades-data the_backend_does_not_serve packaged_metadata_claims_sixteen`
Expected: both FAIL.

- [ ] **Step 3: Implement the predicate and the claims**

In `crates/pleiades-data/src/lib.rs`, after `packaged_bodies()`:

```rust
/// Whether the artifact carries `body` without the backend serving it.
///
/// `asteroid:433-Eros` is fitted to 17 reference rows that lie decades apart
/// outside one nine-day cluster, so its segments are wrong by tens of degrees
/// on almost every date (issue #158). The artifact keeps them, so its bytes
/// and checksum are unchanged, and [`PackagedDataBackend`] declines the body.
/// The dense-data follow-up removes or replaces the segments.
pub(crate) fn is_carried_but_unserved(body: &CelestialBody) -> bool {
    matches!(
        body,
        CelestialBody::Custom(id) if id.catalog == "asteroid" && id.designation == "433-Eros"
    )
}
```

Change `packaged_body_claims()`:

```rust
/// Returns the per-body release claims for the packaged artifact: every body
/// the backend serves is release-grade, validated inside the artifact build
/// against the corpus. A body the artifact carries but the backend does not
/// serve ([`is_carried_but_unserved`]) has no claim.
pub fn packaged_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    packaged_bodies()
        .iter()
        .filter(|body| !is_carried_but_unserved(body))
        .cloned()
        .map(|body| {
            BodyClaim::release_grade(body, AccuracyClass::High, ClaimEvidence::ArtifactValidated)
        })
        .collect()
}
```

- [ ] **Step 4: Implement the backend change**

In `crates/pleiades-data/src/backend.rs`:

`metadata()` — filter the body list:

```rust
        let bodies = artifact
            .bodies
            .iter()
            .map(|series| series.body.clone())
            .filter(|body| !crate::is_carried_but_unserved(body))
            .collect::<Vec<_>>();
```

`supports_body()` — replace the final clause:

```rust
        ) || (!crate::is_carried_but_unserved(&body)
            && self
                .artifact
                .bodies
                .iter()
                .any(|series| series.body == body))
```

`position()` — immediately before `let lookup_instant = normalize_lookup_instant(req.instant);`:

```rust
        if crate::is_carried_but_unserved(&req.body) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                format!(
                    "packaged data carries {} but does not serve it: its segments are fitted \
                     to rows too sparse to interpolate. Serve it from pleiades_jpl::SpkBackend \
                     with a JPL kernel",
                    req.body
                ),
            ));
        }
```

- [ ] **Step 5: Keep the fit check reading the artifact**

`packaged_artifact_fit_samples_with_filter` in `coverage/threshold.rs` compares the reference with `packaged_backend.position`, which now declines Eros. Read the artifact the backend holds directly, so every carried body is still measured. Replace the `let actual = match packaged_backend.position(&request) { … };` block and the `(expected.ecliptic, actual.ecliptic)` destructuring with:

```rust
                // The artifact is read directly, not through the backend, so a body the
                // artifact carries but the backend declines is still measured.
                let actual_ecliptic = match crate::data::packaged_artifact().lookup_ecliptic(
                    &body_artifact.body,
                    crate::regenerate::normalize_lookup_instant(request.instant),
                ) {
                    Ok(ecliptic) => ecliptic,
                    Err(_) => continue,
                };
                let Some(expected_ecliptic) = expected.ecliptic else {
                    continue;
                };
```

Remove the now-unused `let packaged_backend = packaged_backend();` and its import if the compiler reports them unused.

- [ ] **Step 6: Run the crate's tests**

Run: `cargo nextest run -p pleiades-data --no-fail-fast`
Expected: the two Step 1 tests PASS; the Task 1 characterization test PASS; the fit constants in `coverage/fit.rs` unchanged. A failing test that pins a served-body list or count is updated to the new value under the Global Constraints rule; a failing test that pins what the artifact carries ("11 bundled bodies") must still pass unchanged.

- [ ] **Step 7: Validation report corpus**

In `crates/pleiades-validate/src/artifact/inspection.rs`, change `packaged_artifact_corpus`:

```rust
/// Endpoints and midpoints of every body the packaged backend serves. A body
/// the artifact carries but the backend declines is left out: the corpus is
/// run against the backend.
pub(crate) fn packaged_artifact_corpus() -> ValidationCorpus {
    let backend = pleiades_data::PackagedDataBackend::new();
    artifact_comparison_corpus_filtered(packaged_artifact(), |body| {
        backend.supports_body(body.clone())
    })
}
```

Add `use pleiades_backend::EphemerisBackend;` if `supports_body` is not in scope.

Run: `cargo nextest run -p pleiades-validate -p pleiades-cli --no-fail-fast`

Update pinned values that follow from Eros leaving the served set: release-grade body lists and counts (17 to 16, `asteroid:433-Eros@pleiades-data` gone from the ReleaseGrade summary), and the packaged-artifact corpus request count (three fewer). Apply the Global Constraints rule; anything else is a bug.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates
git commit -m "fix(data): stop serving the packaged Eros fit and drop its release-grade claim (#158)"
```

---

### Task 5: Default-chain gate and consumers

**Files:**
- Modify: `crates/pleiades-cli/src/cli/tests/chart.rs:522-528`
- Create: `crates/pleiades-cli/src/cli/tests/asteroid_gate.rs` (declare it beside the other test modules in `crates/pleiades-cli/src/cli/tests/mod.rs` or wherever `chart` is declared)
- Modify: `crates/pleiades-events/tests/nod_aps.rs:1-10, 252-279`

**Interfaces:**
- Consumes: `crate::commands::chart::default_chart_backend() -> RoutingBackend`, `pleiades_jpl::asteroid_reference_corpus() -> &'static [SnapshotEntry]`.

- [ ] **Step 1: Write the default-chain gate**

Create `crates/pleiades-cli/src/cli/tests/asteroid_gate.rs`:

```rust
//! The default chain never serves an asteroid position that disagrees with
//! the JPL `sb441-n373s` rows (issue #158).

use crate::commands::chart::default_chart_backend;
use pleiades_backend::{
    Apparentness, CelestialBody, CustomBodyId, EphemerisBackend, EphemerisRequest,
};
use pleiades_types::{CoordinateFrame, ZodiacMode};

const TRUTH_TOLERANCE_ARCSEC: f64 = 5.0;
const SHARED_EPOCH_JD: f64 = 2_453_000.5;

#[test]
fn the_default_chain_serves_each_truth_epoch_within_tolerance_or_refuses_it() {
    let backend = default_chart_backend();
    let bodies = [
        CelestialBody::Ceres,
        CelestialBody::Pallas,
        CelestialBody::Juno,
        CelestialBody::Vesta,
        CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
    ];
    for body in bodies {
        let mut served = Vec::new();
        let mut refused = 0_usize;
        for row in pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| row.body == body)
        {
            let jd = row.epoch.julian_day.days();
            let request = EphemerisRequest {
                body: body.clone(),
                instant: row.epoch,
                observer: None,
                frame: CoordinateFrame::Ecliptic,
                zodiac_mode: ZodiacMode::Tropical,
                apparent: Apparentness::Mean,
            };
            match backend.position(&request) {
                Ok(result) => {
                    let got = result.ecliptic.expect("a served row has ecliptic coordinates");
                    let truth = row.ecliptic();
                    let longitude = ((got.longitude.degrees() - truth.longitude.degrees() + 180.0)
                        .rem_euclid(360.0)
                        - 180.0)
                        .abs()
                        * truth.latitude.degrees().to_radians().cos()
                        * 3600.0;
                    let latitude =
                        (got.latitude.degrees() - truth.latitude.degrees()).abs() * 3600.0;
                    assert!(
                        longitude <= TRUTH_TOLERANCE_ARCSEC && latitude <= TRUTH_TOLERANCE_ARCSEC,
                        "{body} at JD {jd}: served {longitude:.3}″ / {latitude:.3}″ from truth"
                    );
                    assert_eq!(result.backend_id.as_str(), "jpl-snapshot", "{body} at JD {jd}");
                    served.push(jd);
                }
                Err(_) => refused += 1,
            }
        }
        assert_eq!(served, vec![SHARED_EPOCH_JD], "{body}: served epochs");
        assert_eq!(refused, 406, "{body}: refused epochs");
    }
}

#[test]
fn stations_of_an_asteroid_report_the_refusal() {
    let error = crate::commands::events::render_stations(&[
        "--body",
        "Ceres",
        "--from",
        "2451545.0",
        "--to",
        "2451645.0",
    ])
    .expect_err("Ceres has no rows across this range");
    assert!(error.contains("SpkBackend"), "{error}");
}
```

Check two names against the code before running and adjust the test to match: the field or accessor that holds an `EphemerisResult`'s backend id (the constructor takes `BackendId::new("jpl-snapshot")` first), and the function in `crates/pleiades-cli/src/commands/events.rs` that renders the `stations` command from arguments.

- [ ] **Step 2: Run it**

Run: `cargo nextest run -p pleiades-cli asteroid_gate`
Expected: PASS. (Tasks 3 and 4 already made it true. To confirm the test has teeth, run it once with the Task 4 `supports_body` clause temporarily reverted: it must fail on Eros. Restore the clause and re-run green before continuing.)

- [ ] **Step 3: Tighten the Ceres chart test**

In `crates/pleiades-cli/src/cli/tests/chart.rs`:

```rust
/// Ceres at J2000 sits on a snapshot row. The apparent reduction's light-time
/// step lands off the row and is refused (issue #158), so the placement
/// renders on its mean place.
#[test]
fn chart_command_routes_selected_asteroids_via_jpl_fallback() {
    let rendered = render_chart(&["--jd", "2451545.0", "--body", "Ceres"])
        .expect("asteroid chart should render");
    assert!(rendered.contains("Ceres 184.4"), "{rendered}");
    assert!(rendered.contains("Backend:"));

    let refused = render_chart(&["--jd", "2451700.0", "--body", "Ceres"])
        .expect_err("no snapshot row supports this date");
    assert!(refused.contains("SpkBackend"), "{refused}");
}
```

The J2000 row gives longitude 184.4596°. If the first assertion fails, print `rendered` and match the line as the chart prints it; if the chart reports the refusal in its body instead of returning `Err`, assert on that text instead.

- [ ] **Step 4: Update the Eros nod_aps test**

In `crates/pleiades-events/tests/nod_aps.rs`, replace the file header's second paragraph:

```rust
//! Asteroid coverage bound: no asteroid in the offline chain supports
//! `nod_aps`'s osculating sampling. Ceres, Pallas, Juno, Vesta,
//! asteroid:99942-Apophis and asteroid:433-Eros are served only by
//! `JplSnapshotBackend`, at its sample rows; the packaged backend carries an
//! Eros fit it does not serve. nod_aps samples off the row, so every one
//! fails closed with the backend's refusal (issue #158) — pinned below as the
//! correct production behavior.
```

and `packaged_fit_asteroids_fail_closed`:

```rust
/// asteroid:433-Eros routes past the packaged backend, which carries a fit it
/// does not serve, to `JplSnapshotBackend`. The J2000 row is exact, but
/// nod_aps samples a fraction of a day either side of it, where the snapshot
/// refuses (issue #158).
#[test]
fn packaged_fit_asteroids_fail_closed() {
    let engine = engine();
    let err = engine
        .nod_aps(
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
            tdb(JD),
            NodApsMethod::Osculating,
            ApsisConvention::Aphelion,
        )
        .unwrap_err();
    assert!(
        matches!(err, EventError::Backend { .. }),
        "expected the backend's refusal, got: {err:?}"
    );
}
```

- [ ] **Step 5: Run the affected crates**

Run: `cargo nextest run -p pleiades-cli -p pleiades-events -p pleiades-core --no-fail-fast`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
git add crates
git commit -m "test(cli,events): hold the default chain to served-or-refused for asteroids (#158)"
```

---

### Task 6: Documentation and claims

**Files:**
- Modify: `docs/status.md:42-44`
- Modify: `crates/pleiades-data/README.md:7-16`, `crates/pleiades-data/src/lib.rs:7-11`
- Modify: `spec/data-compression.md:163-167`
- Modify: `docs/cli.md:64`
- Modify: `crates/pleiades-events/README.md:124,178`
- Modify: `crates/pleiades-core/src/compatibility/mod.rs:89` and the pinned checksum at `mod.rs:42`
- Modify: `crates/pleiades-validate/src/claims/mod.rs:51-53`, `crates/pleiades-validate/src/claims/audit.rs:171-174,199-204`

- [ ] **Step 1: `docs/status.md`**

Replace the first known-limits bullet with:

```markdown
- Body/backend grades are **per-backend**: Pluto and the Moon are release-grade
  via the packaged artifact; VSOP87 Pluto and the compact ELP Moon stay
  constrained.
- **Asteroids offline are served only at their sample rows.** Ceres, Pallas,
  Juno, Vesta, `asteroid:433-Eros` and `asteroid:99942-Apophis` come from a
  sparse JPL Horizons fixture: a handful of epochs and a nine-day cluster in
  January 2001. Any other date returns an out-of-range error. The packaged
  artifact still carries an Eros fit that the backend does not serve. For
  asteroid positions across 1900–2100, use `pleiades_jpl::SpkBackend` with a
  JPL kernel (`docs/spk-kernel-sourcing.md`).
```

Keep the existing pointer sentence to the compatibility registry on the first bullet if it is accurate; the registry holds no per-body grade table, so if the sentence claims one, point it at `crates/pleiades-data/src/lib.rs` (`packaged_body_claims`) instead.

- [ ] **Step 2: `pleiades-data` README and crate docs**

In both, where the text says the packaged data covers "Mercury through Pluto and the asteroid 433 Eros" or names "the source-backed custom asteroid `asteroid:433-Eros`", rewrite to say the backend serves the Sun, the Moon and Mercury through Pluto, and add:

```markdown
The artifact also carries segments for `asteroid:433-Eros`, fitted to 17
reference rows. They are not served: outside those rows the fit is wrong by
tens of degrees. `PackagedDataBackend` reports the body unsupported.
```

- [ ] **Step 3: `spec/data-compression.md`**

Keep the Eros ceiling row. Replace the note at lines 165-167 with:

```markdown
Eros ceilings are a self-consistency target only: they compare the artifact
with the 17 reference rows it was fitted to. No independent-truth gate is
applied, and against the JPL `sb441-n373s` rows the fit is wrong by tens of
degrees on almost every date. The packaged backend therefore does not serve
Eros; the segments remain in the artifact until they are regenerated from
dense data.
```

- [ ] **Step 4: `docs/cli.md`**

Replace the `--body` note with:

```markdown
- `--body` accepts built-in labels such as `Sun`, `Moon`, and `Ceres`, plus custom identifiers such as `asteroid:433-Eros`. Asteroids are served offline only at the sample epochs of a sparse JPL fixture (J2000, and 2001-01-01 to 2001-01-10); any other date returns an out-of-range error that names `SpkBackend`, the kernel-backed way to compute them. `stations` and `aspects` return the same error for an asteroid.
```

Confirm the cluster's calendar dates from the rows (JD 2451910.5 to 2451919.5) before writing them.

- [ ] **Step 5: `pleiades-events` README**

At line 124 replace "Stations of asteroids, fictitious bodies and the osculating apogee are found but not gated." with:

```markdown
Stations of fictitious bodies and the osculating apogee are found but not
gated. An asteroid's stations cannot be searched offline: the backend chain
serves asteroids only at sparse sample rows and the search returns its
out-of-range error.
```

At line 178 make the matching change for aspects.

- [ ] **Step 6: Release note and validate comments**

In `crates/pleiades-core/src/compatibility/mod.rs:89`, replace the release note with:

```rust
            "The JPL snapshot backend serves the selected asteroids, including the source-backed custom body asteroid:433-Eros, at its fixture rows only and refuses other instants; the validation report surfaces that subset separately from the planetary comparison corpus.",
```

Run: `cargo nextest run -p pleiades-core rendered_profile_matches_pinned_content_checksum release_notes_cover_release_catalog_entries`

The checksum test fails and prints the new value: update the constant at `mod.rs:42`. `release_notes_cover_release_catalog_entries` asserts on "selected asteroid coverage" and "asteroid:433-Eros"; if the first phrase no longer matches, update the assertion to "serves the selected asteroids".

In `crates/pleiades-validate/src/claims/mod.rs:51-53` and `claims/audit.rs:171-174,199-204`, rewrite the comments that say the packaged backend carries a release-grade Eros claim: it now claims Pluto and the Moon, and Eros has no packaged claim.

- [ ] **Step 7: Docs build and commit**

Run: `mise run docs`
Expected: no warnings.

```bash
cargo fmt --all
git add docs spec crates
git commit -m "docs: state that asteroids are served offline only at their sample rows (#158)"
```

---

### Task 7: Verify, track, and open the PR

- [ ] **Step 1: Full verification**

Wait for `/proc/loadavg` to settle if another session is building. Then, in the foreground and with no edits in between:

Run: `cargo fmt --all --check`
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
Run: `mise run ci`
Expected: all pass. Fix what fails and re-run from the top.

`crates/pleiades-data/tests/artifact_regen.rs` needs `PLEIADES_DE_KERNEL`. If no kernel is available, it does not run; say so in the PR body.

- [ ] **Step 2: File the follow-up issue**

```bash
gh issue create --label enhancement --title "Serve asteroid positions offline across 1900-2100" --body-file <path to a body file in the scratchpad>
```

The body states: what #158's fix left (asteroids served offline only at fixture rows); the measurements (Eros mean error 76.2° over 407 truth rows before the fix); and the scope:
- dense fits of Ceres, Pallas, Juno, Vesta and Eros from `sb441-n373s` in the packaged artifact, gated against `asteroid_reference.csv`;
- real Horizons rows for Apophis at the four removed epochs;
- removal or replacement of the unserved Eros segments, and `is_carried_but_unserved` with them;
- the unexplained 1.0″ longitude / 1.9″ latitude difference between the Horizons rows and the `sb441-n373s` rows at JD 2453000.5, which has the pattern of a rotation about the equinox axis;
- `JplSnapshotBackend`'s backend-level `AccuracyClass::Exact` and `nominal_range`, which describe the rows and not what is served.

- [ ] **Step 3: Open the PR**

Push with the credential-helper override this repository needs:

```bash
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin worktree-asteroid-fail-closed:fix/asteroid-fail-closed
gh pr create --head fix/asteroid-fail-closed --title "fix(jpl,data): refuse asteroid positions away from their sample rows (#158)" --body-file <path to a body file in the scratchpad>
```

The PR body gives: the measured problem, the rule, what users now see, the four removed Apophis rows, the tests added, the verification run (and whether `artifact_regen.rs` ran), and a link to the follow-up issue.

- [ ] **Step 4: Comment on the tracker**

- #158: the position measurements; this PR is the "decline" option; the follow-up issue is the "densify" option.
- #167 (d) and #168 (f): asteroid stations and aspects now return the backend's refusal offline; gating them waits on the follow-up issue.

- [ ] **Step 5: Merge**

Watch the PR's checks to green (`gh pr checks --watch`). A check that fails at 15 minutes with no steps run is a runner-queue cancellation: confirm with `gh run view`, then `gh run rerun <id> --failed`. When every check is green, merge with `gh pr merge --squash --delete-branch` (never `--auto`). Confirm the squash commit on `main` carries the branch's diff before removing the worktree.
