# Sun Aberration Double-Count Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Apply the geocentric Sun's apparent-place aberration/light-time correction exactly once (not twice) in the chart path, by adding a shared `apparent_sun_position` routine in `pleiades-apparent` that both `pleiades-core` (chart) and `pleiades-eclipse` use.

**Architecture:** A new pure function `apparent_sun_position(instant, sun_geocentric_j2000) -> ApparentPosition` in `pleiades-apparent` precesses J2000→of-date, adds nutation, and adds annual aberration *once* with `⊙ = λ` — no light-time re-query. The chart special-cases `body == Sun` to call it; planets/Moon keep the existing `apparent_position` closure path. The eclipse crate's `apparent_sun_longitude_deg` is refactored to delegate to it. The Sun golden tolerance is then tightened from 26″ to the post-fix residual.

**Tech Stack:** Rust, Cargo workspace. Crates: `pleiades-apparent`, `pleiades-core`, `pleiades-eclipse`, `pleiades-validate`.

## Global Constraints

- Sun apparent place applies annual aberration **exactly once**; **no** light-time re-query for the Sun (light-time and annual aberration are the same ~20.5″ effect for the geocentric Sun).
- Planet and Moon apparent paths are **unchanged**.
- Chart `Err → Mean` graceful fallback for release-grade bodies is **preserved**.
- `apparent_sun_position` is a pure function: no backend, no `query` closure, no `max_iterations`, no external `sun_lon` argument.
- Provenance for the Sun: `corrections.light_time = false`, `light_time_days = 0.0`, `iterations = 0`, `annual_aberration = true`, `precession = true`, `nutation_longitude = true`, diurnal flags `false`.
- Golden Sun longitudes in `apparent-goldens.csv` are independent JPL Horizons Q31 values and **must not change** — only the `tolerance_arcsec` column for Sun rows changes.
- TDD: write the failing test first, watch it fail, implement minimally, watch it pass, commit.

---

### Task 1: Add `apparent_sun_position` to `pleiades-apparent`

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent.rs` (add function + unit tests)
- Modify: `crates/pleiades-apparent/src/lib.rs:38` (export the new symbol)

**Interfaces:**
- Consumes (already public): `precess_ecliptic_j2000_to_date`, `annual_aberration`, `nutation`, `ApparentPosition`, `ApparentProvenance`, `CorrectionSet`, `MODEL_SOURCES`, `EclipticCoordinates`, `Instant`, `Longitude`, `Latitude`.
- Produces (later tasks rely on this exact signature):
  ```rust
  pub fn apparent_sun_position(
      instant: Instant,
      sun_geocentric_j2000: EclipticCoordinates,
  ) -> Result<ApparentPosition, ApparentPlaceError>
  ```
  Returns `ApparentPosition { ecliptic, provenance }`. The returned `ecliptic.distance_au` equals the input's `distance_au` (passed through unchanged).

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/pleiades-apparent/src/apparent.rs` (it already imports `super::*`, `JulianDay`, `TimeScale`, and has the `fixed` helper):

```rust
#[test]
fn sun_applies_aberration_once_no_light_time_requery() {
    // At J2000, precession ≈ identity. The Sun routine must apply aberration
    // exactly once and NOT re-query light-time. Compare against a hand-built
    // single-aberration reference: precess (≈identity here) + Δψ + one annual
    // aberration term with ⊙ = λ.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let sun_j2000 = fixed(280.0, 0.0, 0.983);
    let out = apparent_sun_position(instant, sun_j2000).unwrap();

    // Reference: same math, applied once.
    let jd = 2_451_545.0_f64;
    let p = crate::precession::precess_ecliptic_j2000_to_date(280.0, 0.0, jd).unwrap();
    let lambda = p.longitude_deg;
    let beta = p.latitude_deg;
    let ab = crate::aberration::annual_aberration(lambda, beta, lambda, jd);
    let nut = crate::nutation::nutation(jd).unwrap();
    let expected_lon =
        (lambda + (ab.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);

    let diff_arcsec = (out.ecliptic.longitude.degrees() - expected_lon) * 3600.0;
    assert!(diff_arcsec.abs() < 1e-6, "Sun apparent lon off by {diff_arcsec}\"");

    // Distance is passed through unchanged (no re-query).
    assert_eq!(out.ecliptic.distance_au, Some(0.983));

    // Provenance: aberration once, no light-time iteration.
    assert!(!out.provenance.corrections.light_time, "light_time must be false for Sun");
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.precession);
    assert!(out.provenance.corrections.nutation_longitude);
    assert_eq!(out.provenance.light_time_days, 0.0);
    assert_eq!(out.provenance.iterations, 0);
    // The single applied aberration term, recorded (≈ -20" for the Sun).
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-9);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-apparent sun_applies_aberration_once -- --nocapture`
Expected: FAIL to compile — `cannot find function apparent_sun_position in this scope`.

- [ ] **Step 3: Write minimal implementation**

Add this function in `crates/pleiades-apparent/src/apparent.rs`, immediately after `apparent_position` (before the `#[cfg(test)]` module):

```rust
/// Computes the apparent ecliptic-of-date position of the **geocentric Sun**,
/// applying annual aberration **exactly once** with no light-time re-query.
///
/// For a planet, light-time retardation and annual aberration are physically
/// distinct effects ("planetary aberration" = both). For the Sun they are the
/// *same* ~20.5″ Earth-orbital reflex effect (Meeus, *Astronomical Algorithms*
/// §25): re-querying the geocentric Sun at `t − τ` already displaces it by the
/// aberration amount, so applying a separate annual-aberration term on top
/// double-counts it. This routine therefore takes the Sun's instantaneous
/// (un-retarded) Mean/J2000 geocentric ecliptic position and applies precession,
/// nutation, and aberration once — never a light-time re-query.
///
/// The Sun's own true longitude of date supplies the `⊙` argument of the
/// aberration formula (`⊙ = λ`). Distance passes through unchanged (it is
/// essentially constant over the Sun's own light-time).
///
/// `corrections.light_time` is reported `false` even though this is an apparent
/// place: no light-time iteration is performed; the aberration term *is* the
/// light-time displacement for the Sun.
pub fn apparent_sun_position(
    instant: Instant,
    sun_geocentric_j2000: EclipticCoordinates,
) -> Result<ApparentPosition, ApparentPlaceError> {
    let jd_tt = instant.julian_day.days();
    let lambda_j2000 = sun_geocentric_j2000.longitude.degrees();
    let beta_j2000 = sun_geocentric_j2000.latitude.degrees();

    let precessed = precess_ecliptic_j2000_to_date(lambda_j2000, beta_j2000, jd_tt)?;
    let lambda = precessed.longitude_deg;
    let beta = precessed.latitude_deg;

    // Sun is its own aberration argument: ⊙ = λ. Applied ONCE.
    let aberration = annual_aberration(lambda, beta, lambda, jd_tt);
    let nut = nutation(jd_tt)?;

    let apparent_lon =
        (lambda + (aberration.d_lambda_arcsec + nut.delta_psi_arcsec) / 3600.0).rem_euclid(360.0);
    let apparent_lat = beta + aberration.d_beta_arcsec / 3600.0;
    if !apparent_lon.is_finite() || !apparent_lat.is_finite() {
        return Err(ApparentPlaceError::NonFiniteCorrection {
            stage: "apparent-sun-combine",
        });
    }

    let mut precession_shift = lambda - lambda_j2000;
    if precession_shift > 180.0 {
        precession_shift -= 360.0;
    } else if precession_shift < -180.0 {
        precession_shift += 360.0;
    }

    let ecliptic = EclipticCoordinates::new(
        Longitude::from_degrees(apparent_lon),
        Latitude::from_degrees(apparent_lat),
        sun_geocentric_j2000.distance_au,
    );
    let provenance = ApparentProvenance {
        light_time_days: 0.0,
        iterations: 0,
        precession_longitude_arcsec: precession_shift * 3600.0,
        nutation_longitude_arcsec: nut.delta_psi_arcsec,
        aberration_longitude_arcsec: aberration.d_lambda_arcsec,
        corrections: CorrectionSet {
            light_time: false,
            precession: true,
            annual_aberration: true,
            nutation_longitude: true,
            diurnal_parallax: false,
            diurnal_aberration: false,
        },
        model_sources: MODEL_SOURCES,
    };
    Ok(ApparentPosition {
        ecliptic,
        provenance,
    })
}
```

This requires the `nutation` function in scope. The module already imports `use crate::nutation::nutation;` (used by `apparent_position`) — confirm that import line exists at the top of `apparent.rs`; it does (`apparent.rs:11`). No new `use` needed.

- [ ] **Step 4: Export the symbol**

In `crates/pleiades-apparent/src/lib.rs`, change line 38 from:

```rust
pub use apparent::{apparent_position, ApparentPosition, DEFAULT_MAX_ITERATIONS};
```
to:
```rust
pub use apparent::{apparent_position, apparent_sun_position, ApparentPosition, DEFAULT_MAX_ITERATIONS};
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p pleiades-apparent sun_applies_aberration_once -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Run the full apparent crate tests**

Run: `cargo test -p pleiades-apparent`
Expected: PASS (existing `at_j2000_only_aberration_and_nutation_shift_longitude`, etc., still pass — `apparent_position` is untouched).

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-apparent/src/apparent.rs crates/pleiades-apparent/src/lib.rs
git commit -m "feat(apparent): add apparent_sun_position (aberration applied once for the Sun)"
```

---

### Task 2: Refactor `pleiades-eclipse` to delegate to the shared routine

**Files:**
- Modify: `crates/pleiades-eclipse/src/ephemeris.rs:161-185` (`apparent_sun_longitude_deg` body)

**Interfaces:**
- Consumes: `apparent_sun_position` (Task 1).
- Produces: `apparent_sun_longitude_deg` unchanged signature `fn(&B, f64) -> Result<f64, EclipseError>` and unchanged numeric output (the existing `validate-eclipses` gate proves this).

- [ ] **Step 1: Run the existing eclipse gate to capture the baseline**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS — record that it currently passes (this is the behaviour-preservation baseline). Also run the validation gate if it is a separate binary:
Run: `cargo test -p pleiades-validate eclipse 2>/dev/null || true`

- [ ] **Step 2: Replace the function body to delegate**

In `crates/pleiades-eclipse/src/ephemeris.rs`, replace the body of `apparent_sun_longitude_deg` (the steps 1–4 block, lines ~165–184) with:

```rust
    // Step 1: Sun's J2000 mean geocentric ecliptic at the true (un-retarded) epoch.
    let (sun_lon_j2000, sun_lat_j2000, sun_dist_au) =
        read(backend, CelestialBody::Sun, "Sun", julian_day)?;

    // Steps 2–4: precess → nutation → aberration ONCE, via the shared routine.
    // For the geocentric Sun, light-time retardation and annual aberration are
    // the same effect, so `apparent_sun_position` performs no light-time re-query.
    let sun_j2000 = EclipticCoordinates::new(
        Longitude::from_degrees(sun_lon_j2000),
        Latitude::from_degrees(sun_lat_j2000),
        Some(sun_dist_au),
    );
    let instant = Instant::new(JulianDay::from_days(julian_day), TimeScale::Tdb);
    let apparent = apparent_sun_position(instant, sun_j2000)
        .map_err(|e| EclipseError::Backend(format!("Sun apparent place failed: {e}")))?;
    Ok(apparent.ecliptic.longitude.degrees())
```

- [ ] **Step 3: Update imports in `ephemeris.rs`**

At the top of `crates/pleiades-eclipse/src/ephemeris.rs`, update the `pleiades_apparent` use line. Current:
```rust
use pleiades_apparent::aberration::annual_aberration;
use pleiades_apparent::nutation::nutation;
use pleiades_apparent::{precess_ecliptic_j2000_to_date, LIGHT_TIME_DAYS_PER_AU};
```
Replace with (drop the now-unused `annual_aberration`, `nutation`, `precess_ecliptic_j2000_to_date`; keep `LIGHT_TIME_DAYS_PER_AU`, which is still used by `read_retarded`):
```rust
use pleiades_apparent::{apparent_sun_position, LIGHT_TIME_DAYS_PER_AU};
```
And ensure `Longitude`, `Latitude`, `EclipticCoordinates` are imported from `pleiades_types`. The existing `use pleiades_types::{ ... }` line (`ephemeris.rs:12-14`) imports `Apparentness, CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale, ZodiacMode`. Add `EclipticCoordinates, Latitude, Longitude` to that list.

- [ ] **Step 4: Update the now-stale doc-comment**

The doc-comment above `apparent_sun_longitude_deg` (`ephemeris.rs:128-160`) describes the inlined steps. Trim it so it still explains *why aberration is applied once for the Sun* but points to the shared routine instead of re-listing the deleted math. Keep the "Assumption: backend returns Mean/J2000 geocentric coordinates" note. Example replacement for the numbered-steps paragraph:

```rust
/// Computes the apparent geocentric solar ecliptic longitude of date (degrees)
/// by delegating to [`pleiades_apparent::apparent_sun_position`], which applies
/// annual aberration exactly once (no light-time re-query) — see that function
/// for why light-time and aberration are the same effect for the geocentric Sun.
```

Also update the cross-reference note in `read_retarded`'s doc-comment (`ephemeris.rs:88-91`) if it names the deleted inline steps; it currently says aberration "applied in `apparent_sun_longitude_deg`" — that is still accurate (the function still owns the once-only aberration), so leave it unless it references deleted internals.

- [ ] **Step 5: Build and run the eclipse gate (behaviour preservation)**

Run: `cargo test -p pleiades-eclipse`
Expected: PASS — identical to the Step 1 baseline. The `elongation_reflects_light_time_at_geometric_new_moon` and `missing_coordinates_fail_closed` unit tests still pass.

Run the full eclipse validation gate (≤1.0″ on 908 rows):
Run: `cargo test -p pleiades-validate -- eclipse` (or the workspace command the repo uses for `validate-eclipses`; check `crates/pleiades-validate` for the exact harness)
Expected: PASS — all in-coverage rows still within the ≤1.0″ longitude tolerance.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-eclipse/src/ephemeris.rs
git commit -m "refactor(eclipse): delegate apparent_sun_longitude_deg to shared apparent_sun_position"
```

---

### Task 3: Special-case the Sun in the chart apparent path

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:39-42` (import), `:51-59` (add error mapper), `:304-337` (Sun branch)

**Interfaces:**
- Consumes: `apparent_sun_position` (Task 1), existing `self.query_mean_ecliptic`.
- Produces: chart Sun placements with apparent longitude corrected once (no behaviour change to other bodies).

- [ ] **Step 1: Write the failing test**

The most direct regression guard is the apparent golden gate (Task 4). For an isolated unit assertion that the Sun no longer double-counts, add a test to `crates/pleiades-core` that builds a chart with the packaged backend and checks the Sun's apparent longitude against the JPL Horizons Q31 value at J2000 (2451545.0 = 280.3689092°, from `apparent-goldens.csv`) within a tight bound that the *buggy* code would fail.

Locate the chart integration test module (search: `grep -rn "Apparentness::Apparent" crates/pleiades-core/tests crates/pleiades-core/src/chart`). Add, in the appropriate existing test file that already constructs a `ChartEngine` over the packaged backend:

```rust
#[test]
fn sun_apparent_longitude_is_not_double_counted_at_j2000() {
    // JPL Horizons Q31 apparent ecliptic longitude of the Sun at JD_TT 2451545.0
    // is 280.3689092°. The pre-fix code double-counted aberration (~+20"), landing
    // ~20" high. Post-fix must be within a few arcsec.
    let engine = /* construct ChartEngine over packaged_backend(), as neighbouring tests do */;
    let request = /* ChartRequest at Instant TT 2451545.0, bodies = [Sun],
                     zodiac Tropical, Apparentness::Apparent, no observer */;
    let snapshot = engine.assemble(&request).unwrap();
    let sun = snapshot /* .placement / sign_for_body lookup for CelestialBody::Sun */;
    let lon = sun.ecliptic.longitude.degrees();
    let diff_arcsec = ((lon - 280.3689092).rem_euclid(360.0)) * 3600.0;
    let diff_arcsec = if diff_arcsec > 180.0 * 3600.0 { diff_arcsec - 360.0 * 3600.0 } else { diff_arcsec };
    assert!(diff_arcsec.abs() < 8.0, "Sun apparent lon residual {diff_arcsec}\" (double-count not fixed?)");
}
```

> Implementer note: fill the `/* ... */` placeholders by copying the exact construction pattern from the nearest existing apparent-mode chart test in `pleiades-core` (the same crate already has tests asserting `Apparentness::Apparent` placements, e.g. `release_grade_body_falls_back_to_mean_when_apparent_unavailable`). Do not invent new constructors. If the packaged backend is not already a test dependency of `pleiades-core`, prefer adding this assertion to the apparent golden gate in Task 4 instead and skip this unit test.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-core sun_apparent_longitude_is_not_double_counted -- --nocapture`
Expected: FAIL — residual ~20″ (the double-count) exceeds the 8″ bound.

- [ ] **Step 3: Add the `ApparentPlaceError → EphemerisError` mapper**

In `crates/pleiades-core/src/chart/mod.rs`, immediately after `map_apparent_error` (ends line 59), add:

```rust
fn map_apparent_place_error(error: ApparentPlaceError) -> EphemerisError {
    EphemerisError::new(
        EphemerisErrorKind::InvalidRequest,
        format!("apparent-place computation failed: {error}"),
    )
}
```

- [ ] **Step 4: Update imports**

Change `crates/pleiades-core/src/chart/mod.rs:39-42` from:
```rust
use pleiades_apparent::{
    apparent_position, precess_ecliptic_j2000_to_date, ApparentLightTimeError,
    DEFAULT_MAX_ITERATIONS,
};
```
to:
```rust
use pleiades_apparent::{
    apparent_position, apparent_sun_position, precess_ecliptic_j2000_to_date,
    ApparentLightTimeError, ApparentPlaceError, DEFAULT_MAX_ITERATIONS,
};
```

- [ ] **Step 5: Add the Sun branch at the apparent call site**

In `crates/pleiades-core/src/chart/mod.rs`, replace the `outcome` computation inside the `if release_grade.contains(&body)` block (currently lines 306–314) with a branch on the Sun. Current:
```rust
                        let body_for_query = body.clone();
                        let body_observer_for_query = request.body_observer.clone();
                        let outcome = apparent_position::<_, EphemerisError>(
                            request.instant,
                            sun_lon,
                            DEFAULT_MAX_ITERATIONS,
                            |instant| self.query_mean_ecliptic(&body_for_query, instant, &backend_zodiac_mode, body_observer_for_query.clone()),
                        )
                        .map_err(map_apparent_error);
```
Replace with:
```rust
                        let outcome = if matches!(body, pleiades_types::CelestialBody::Sun) {
                            // Sun: aberration and light-time are the same effect, so
                            // apply aberration ONCE via apparent_sun_position with the
                            // instantaneous (un-retarded) geocentric Sun. observer = None
                            // keeps the aberration argument geocentric.
                            self.query_mean_ecliptic(
                                &body,
                                request.instant,
                                &backend_zodiac_mode,
                                None,
                            )
                            .and_then(|sun_j2000| {
                                apparent_sun_position(request.instant, sun_j2000)
                                    .map_err(map_apparent_place_error)
                            })
                        } else {
                            let body_for_query = body.clone();
                            let body_observer_for_query = request.body_observer.clone();
                            apparent_position::<_, EphemerisError>(
                                request.instant,
                                sun_lon,
                                DEFAULT_MAX_ITERATIONS,
                                |instant| self.query_mean_ecliptic(&body_for_query, instant, &backend_zodiac_mode, body_observer_for_query.clone()),
                            )
                            .map_err(map_apparent_error)
                        };
```

Both arms yield `Result<ApparentPosition, EphemerisError>`, so the existing `match outcome { Ok(outcome) => {...} Err(_) => { fall back to mean } }` block (lines 315–337) is unchanged and the graceful fallback still applies to the Sun. Note `sun_lon` is now unused in the Sun arm but still consumed by the planet/Moon arm, so the outer `if let Some(sun_lon) = ...` stays.

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p pleiades-core sun_apparent_longitude_is_not_double_counted -- --nocapture`
Expected: PASS — residual now a few arcsec, under 8″.

- [ ] **Step 7: Run the full core test suite (fallback regression intact)**

Run: `cargo test -p pleiades-core`
Expected: PASS — including `release_grade_body_falls_back_to_mean_when_apparent_unavailable` (the Sun arm uses the same `Err → Mean` match arm).

- [ ] **Step 8: Commit**

```bash
git add crates/pleiades-core/src/chart/mod.rs
git commit -m "fix(chart): apply Sun apparent aberration once (use apparent_sun_position)"
```

---

### Task 4: Tighten the Sun golden tolerance and update rationale

**Files:**
- Modify: `crates/pleiades-validate/data/apparent-goldens.csv` (5 Sun rows + header)
- Modify: `crates/pleiades-validate/scripts/regen-apparent-goldens.sh` (tolerance rationale comments)

**Interfaces:**
- Consumes: the corrected chart Sun path (Task 3).
- Produces: an apparent gate that fails if the Sun double-count regresses.

- [ ] **Step 1: Measure the post-fix Sun residual**

Run the apparent validation gate and capture the per-row Sun residuals (the gate compares the engine's apparent Sun longitude to the Horizons golden). Identify the harness first:
Run: `grep -rn "apparent-goldens\|apparent_validation\|fn .*apparent" crates/pleiades-validate/src | head`
Then run the gate (exact command per the repo's harness, e.g.):
Run: `cargo test -p pleiades-validate apparent -- --nocapture`
Expected: PASS at the current 26″ tolerance. Read the printed Sun residuals (or temporarily add a `println!` of `(engine_lon - golden_lon)*3600` in the Sun comparison if the harness doesn't already print it). Record the **max absolute Sun residual** across the 5 epochs (expected single-digit arcsec, ~1–6″).

- [ ] **Step 2: Set the new tolerance**

Choose `new_tol = ceil(max_observed_sun_residual) + 2.0` arcsec (mirrors the existing "max residual + 2″ margin" rule in the header). In `crates/pleiades-validate/data/apparent-goldens.csv`, change the `tolerance_arcsec` column on the 5 Sun rows (lines 29–33) from `26.0` to `new_tol`. Leave the `apparent_longitude_deg` values (Horizons Q31) **unchanged**. Example if max residual is 4″ → tolerance `6.0`:

```
Sun,2415025.5,285.2520208,6.0
Sun,2433282.5,280.0048301,6.0
Sun,2451545.0,280.3689092,6.0
Sun,2469807.5,280.7483801,6.0
Sun,2488065.5,276.5293785,6.0
```

- [ ] **Step 3: Update the CSV header rationale**

In `crates/pleiades-validate/data/apparent-goldens.csv`, the TOLERANCE RATIONALE block (lines 6–13) currently lumps Sun with planets and attributes 15–25″ residuals to polynomial-fit error. Edit the Sun-specific claim to reflect the fix. Change the `Sun/planets` bullet so it no longer claims 15–25″ for the Sun, and add a Sun line, e.g.:

```
#   - Sun: apparent corrections (precession, nutation, annual aberration) applied ONCE
#     via apparent_sun_position; for the geocentric Sun light-time and annual aberration
#     are the same effect, so no separate light-time re-query is performed. Residual vs
#     Horizons Q31 is now a few arcsec (was masked by a ~20" aberration double-count under
#     the old 26" tolerance). Tolerance = max observed residual + 2".
#   - Planets: polynomial-fit ephemeris matches JPL DE441 to ~0.01 deg (36 arcsec);
#     apparent-mode residuals 15-25 arcsec, set to max observed residual + 2 arcsec margin.
```

- [ ] **Step 4: Mirror the rationale in the regen script**

In `crates/pleiades-validate/scripts/regen-apparent-goldens.sh`, the header comment block (the `Tolerances:` paragraph, ~lines 7–11) hard-codes "planets/Sun 26 arcsec". Update it to note the Sun is now a few arcsec (aberration applied once) while planets remain ~26″. Keep the Moon (45″) and Eros-excluded notes unchanged. This is a comment-only edit; the script does not need to re-run.

- [ ] **Step 5: Run the apparent gate at the new tolerance**

Run: `cargo test -p pleiades-validate apparent -- --nocapture`
Expected: PASS — all 5 Sun rows within `new_tol`; Moon and planet rows unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-validate/data/apparent-goldens.csv crates/pleiades-validate/scripts/regen-apparent-goldens.sh
git commit -m "test(apparent): tighten Sun golden tolerance after aberration double-count fix"
```

---

### Task 5: Workspace verification and close out FU-1

**Files:**
- Modify: `docs/follow-ups.md` (mark FU-1 resolved)

- [ ] **Step 1: Full workspace build + test**

Run: `cargo build --workspace`
Expected: clean build, no unused-import warnings (Task 2 dropped `annual_aberration`/`nutation`/`precess_ecliptic_j2000_to_date` from eclipse; Task 3 still uses its imports).

Run: `cargo test --workspace`
Expected: PASS across all crates.

- [ ] **Step 2: Lint**

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: no warnings (catches any now-unused import or variable, e.g. a stale `sun_lon` binding).

- [ ] **Step 3: Mark FU-1 resolved**

In `docs/follow-ups.md`, update FU-1's `**Status:**` line from `open` to `resolved` and append a one-line resolution note pointing at the commits, e.g.:

```
**Status:** resolved (2026-06-29) · Fixed by `apparent_sun_position` in pleiades-apparent;
chart Sun path applies aberration once; eclipse delegates to the shared routine; Sun golden
tolerance tightened from 26″ to a few arcsec. · Severity: important (accuracy) · Opened: 2026-06-29
```

- [ ] **Step 4: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs: mark FU-1 (Sun aberration double-count) resolved"
```

---

## Self-Review

**Spec coverage:**
- Spec §1 (shared routine) → Task 1. ✅
- Spec §2 (chart call site, fallback preserved) → Task 3 (Steps 5, 7). ✅
- Spec §3 (unify eclipse) → Task 2. ✅
- Spec §4 (provenance) → Task 1 Step 3 (provenance block) + test Step 1. ✅
- Spec §5 (goldens, Moon untouched) → Task 4. ✅
- Spec Testing (unit, eclipse gate, apparent gate, fallback regression) → Tasks 1/2/3/4. ✅
- Spec Files touched (incl. `docs/follow-ups.md`) → Task 5. ✅

**Placeholder scan:** The only `/* ... */` placeholders are in Task 3 Step 1's optional unit test, with an explicit implementer note and a documented fallback (assert in Task 4 instead) — acceptable because the binding regression is the golden gate, and the construction pattern is named precisely (copy from `release_grade_body_falls_back_to_mean_when_apparent_unavailable`). No "TBD/handle errors/add validation" placeholders elsewhere.

**Type consistency:** `apparent_sun_position(instant: Instant, sun_geocentric_j2000: EclipticCoordinates) -> Result<ApparentPosition, ApparentPlaceError>` is defined identically in Task 1 and consumed with that exact shape in Tasks 2 and 3. `map_apparent_place_error(ApparentPlaceError) -> EphemerisError` defined and used in Task 3. `ApparentPlaceError::NonFiniteCorrection { stage }` matches the variant used by `apparent_position` (apparent.rs:57). Provenance field names match `ApparentProvenance` / `CorrectionSet` exactly.
