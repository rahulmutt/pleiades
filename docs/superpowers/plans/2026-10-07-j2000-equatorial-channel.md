# J2000 Equatorial Channel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the first-party backends' `equatorial` channel true J2000
right ascension and declination. Make a chart's mean placement report J2000
RA/Dec on every backend, ELP included (issue #210).

**Architecture:**
- One helper, `EclipticCoordinates::to_j2000_equatorial`, rotates a J2000
  ecliptic place by `OBLIQUITY_J2000_DEG`.
- Five backend call sites switch from the obliquity of date to this helper.
  So do the JPL evidence builder and the four JPL parity self-checks.
- The chart overwrites each tropical backend result's equatorial channel with
  the helper's value before any sidereal or apparent rewrite. A successful
  apparent reduction still replaces it with RA/Dec of date.
- `ElpBackend` keeps its of-date channel.

**Tech Stack:** Rust workspace (`pleiades-*` crates), cargo-nextest, `mise`
tasks (`mise run ci`, `mise run test-full`).

**Spec:** `docs/superpowers/specs/2026-10-07-j2000-equatorial-channel-design.md`

## Global Constraints

- J2000 obliquity is `pleiades_types::OBLIQUITY_J2000_DEG` (23.439 291 111 111 11°). Never hard-code the number.
- `ElpBackend`'s equatorial channel, its tests and its evidence are **not** changed.
- No change to `pleiades-validate/src/equatorial_validation.rs`, `equatorial-goldens.csv`, the SE equatorial corpus, their checksums or tolerances. They must pass unchanged.
- No change to the packaged artifact bytes. Nothing is regenerated.
- The mean-obliquity round-trip evidence (`pleiades-validate/src/render/text/evidence.rs`) and the CLI command names `mean-obliquity-frame-round-trip[-summary]` stay as they are.
- A backend that returns no `equatorial` keeps returning none through the chart. A backend that returns `equatorial` but no `ecliptic` keeps its own value.
- Code must be `rustfmt`-clean (`cargo fmt --all`) and clippy-clean (`-D warnings`). Run `cargo fmt --all` before every commit.
- No source edits and no commits while a background test run is in progress.
- Commit messages use conventional commits. Squash into one `fix(...)` PR at the end.

## Review Focus

1. **A backend that serves a sidereal zodiac natively.** Its `ecliptic` is sidereal, and rotating it by ε₀ gives nonsense. Expected: the chart keeps that backend's own equatorial channel. Pinned in Task 4, `native_sidereal_backend_keeps_its_equatorial_channel`.
2. **A backend that gives `equatorial` but no `ecliptic`.** Expected: the chart keeps the backend's value and does not drop it. Pinned in Task 4, `equatorial_without_ecliptic_is_kept`.
3. **The batch path.** The chart reads `positions()` (batch), not only `position()`. Expected: the rewrite applies to every placement of a multi-body chart. Task 4's wrapper backend overrides `position`, `position_without_motion` and `positions`, and its tests chart two bodies.
4. **A body that falls back to mean inside an apparent chart** (for example the true node). Expected: J2000 RA/Dec, not the backend's value. Pinned in Task 4, `mean_fallback_in_an_apparent_chart_is_j2000`.
5. **RA wrap at 0°/360°.** Expected: RA stays in `[0, 360)`. `to_equatorial` already normalizes. Task 1's helper test includes a place just west of the equinox (ecliptic longitude 359.9°).

---

## File Structure

| File | Change |
|---|---|
| `crates/pleiades-types/src/coordinates.rs` | add `EclipticCoordinates::to_j2000_equatorial` |
| `crates/pleiades-types/src/tests/…` or the existing coordinates test module | helper tests |
| `crates/pleiades-vsop87/src/backend.rs` | use helper |
| `crates/pleiades-data/src/backend.rs` | use helper (2 sites) and doc line |
| `crates/pleiades-jpl/src/backend.rs` | use helper (3 sites) |
| `crates/pleiades-jpl/src/spk/backend.rs` | use helper |
| `crates/pleiades-jpl/src/reference_summary/{comparison,selected_asteroid,reference_asteroid}.rs`, `reference_snapshot/core/parity.rs` | parity self-checks use helper |
| backend test files listed per task | expectations move to ε₀, plus new off-J2000 tests |
| `crates/pleiades-core/src/chart/mod.rs` | chart owns J2000 RA/Dec |
| `crates/pleiades-core/src/chart/test_support.rs` | `ForeignEquatorialBackend` wrapper |
| `crates/pleiades-core/src/chart/tests.rs` | chart tests |
| `crates/pleiades-core/src/chart/request.rs` | `with_apparentness` rustdoc |
| `crates/pleiades-backend/src/result.rs`, `spec/backend-trait.md` | channel frame contract |
| prose and pinning tests (Task 5 list) | "J2000 mean-obliquity" wording |
| `crates/pleiades-core/src/compatibility/mod.rs` | release note, profile 0.7.31 → 0.7.32, checksum |

---

### Task 1: The J2000 rotation helper

**Files:**
- Modify: `crates/pleiades-types/src/coordinates.rs` (next to `to_equatorial`, around line 37)
- Test: the existing test module for `coordinates.rs`. Find it with `grep -rn "fn .*to_equatorial" crates/pleiades-types/src`. Add the tests beside the existing `to_equatorial` tests.

**Interfaces:**
- Produces: `pub fn to_j2000_equatorial(self) -> EquatorialCoordinates` on `EclipticCoordinates`. Every later task calls it.

- [ ] **Step 1: Write the failing tests.** The expected values come from spherical trigonometry, not from the code:

```rust
#[test]
fn to_j2000_equatorial_rotates_by_the_j2000_obliquity() {
    // The summer solstice point (λ = 90°, β = 0) lies at RA 90° and
    // Dec = +ε₀ on the J2000 equator.
    let solstice = EclipticCoordinates::new(
        Longitude::from_degrees(90.0),
        Latitude::from_degrees(0.0),
        Some(1.0),
    );
    let eq = solstice.to_j2000_equatorial();
    assert!((eq.right_ascension.degrees() - 90.0).abs() < 1e-12);
    assert!((eq.declination.degrees() - crate::OBLIQUITY_J2000_DEG).abs() < 1e-12);
    assert_eq!(eq.distance_au, Some(1.0));
    // The ecliptic pole (β = 90°) lies at RA 270°, Dec 90° − ε₀.
    let pole = EclipticCoordinates::new(
        Longitude::from_degrees(0.0),
        Latitude::from_degrees(90.0),
        None,
    );
    let eq = pole.to_j2000_equatorial();
    assert!((eq.right_ascension.degrees() - 270.0).abs() < 1e-9);
    assert!((eq.declination.degrees() - (90.0 - crate::OBLIQUITY_J2000_DEG)).abs() < 1e-12);
}

#[test]
fn to_j2000_equatorial_keeps_right_ascension_in_range_west_of_the_equinox() {
    // λ = 359.9°, β = 0: just west of the equinox, so RA is just under 360°.
    let place = EclipticCoordinates::new(
        Longitude::from_degrees(359.9),
        Latitude::from_degrees(0.0),
        None,
    );
    let ra = place.to_j2000_equatorial().right_ascension.degrees();
    assert!((359.0..360.0).contains(&ra), "{ra}");
}
```

  Use the module's existing imports (`Longitude`, `Latitude`). Add them if absent.

- [ ] **Step 2: Run the tests and confirm they fail.**
  Run: `cargo nextest run -p pleiades-types to_j2000_equatorial`
  Expected: compile error, `no method named to_j2000_equatorial`.

- [ ] **Step 3: Implement.** Put this directly after `to_equatorial`:

```rust
    /// Rotates a place on the J2000 ecliptic and equinox to the J2000 mean
    /// equator and equinox, by the J2000 mean obliquity
    /// [`OBLIQUITY_J2000_DEG`](crate::OBLIQUITY_J2000_DEG).
    ///
    /// This is the frame of the first-party backends' equatorial channel
    /// (issue #210). Rotating a J2000 place by an obliquity of date instead
    /// mixes two frames.
    pub fn to_j2000_equatorial(self) -> EquatorialCoordinates {
        self.to_equatorial(Angle::from_degrees(crate::OBLIQUITY_J2000_DEG))
    }
```

  If `OBLIQUITY_J2000_DEG` lives in `time.rs` and is re-exported from `lib.rs`, `crate::OBLIQUITY_J2000_DEG` resolves. Check with `grep -n OBLIQUITY_J2000_DEG crates/pleiades-types/src/lib.rs`.

- [ ] **Step 4: Run the tests and confirm they pass.**
  Run: `cargo nextest run -p pleiades-types to_j2000_equatorial`
  Expected: 2 passed.

- [ ] **Step 5: Commit.**

```bash
cargo fmt --all
git add crates/pleiades-types
git commit -m "feat(types): add EclipticCoordinates::to_j2000_equatorial (#210)"
```

---

### Task 2: VSOP87 and packaged-data backends give J2000 RA/Dec

**Files:**
- Modify: `crates/pleiades-vsop87/src/backend.rs:291-293`
- Modify: `crates/pleiades-data/src/backend.rs:85` (doc), `:93`, `:507`
- Test: `crates/pleiades-vsop87/src/tests/backend.rs` (lines 530, 584, 638, 692, 746, 840, 899, plus a new test)
- Test: `crates/pleiades-data/src/tests/lookup.rs` (lines 50 and 1293-1298, plus a new test)

**Interfaces:**
- Consumes: `EclipticCoordinates::to_j2000_equatorial` (Task 1).

- [ ] **Step 1: Write the failing VSOP87 test.** Add it to `crates/pleiades-vsop87/src/tests/backend.rs`. It pins the size and sign of the change against the issue's measurement, which comes from outside the code:

```rust
// Issue #210: the equatorial channel is the J2000 ecliptic rotated by the
// J2000 obliquity. The old channel (rotated by the obliquity of date) was
// off by RA +5.4″, Dec −44.8″ for Mars at 1900-01-01 (measured 2026-10-06).
#[test]
fn equatorial_channel_is_j2000_away_from_j2000() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
    let instant = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let result = Vsop87Backend::new()
        .position(&EphemerisRequest::new(CelestialBody::Mars, instant))
        .unwrap();
    let ecliptic = result.ecliptic.expect("ecliptic");
    let equatorial = result.equatorial.expect("equatorial");
    let j2000 = ecliptic.to_j2000_equatorial();
    assert!((equatorial.right_ascension.degrees() - j2000.right_ascension.degrees()).abs() < 1e-12);
    assert!((equatorial.declination.degrees() - j2000.declination.degrees()).abs() < 1e-12);
    let old = ecliptic.to_equatorial(instant.mean_obliquity());
    let d_ra = (old.right_ascension.degrees() - j2000.right_ascension.degrees()) * 3600.0;
    let d_dec = (old.declination.degrees() - j2000.declination.degrees()) * 3600.0;
    assert!((d_ra - 5.4).abs() < 0.15, "RA {d_ra}″");
    assert!((d_dec - -44.8).abs() < 0.15, "Dec {d_dec}″");
}
```

  If `Vsop87Backend::new()` is not the constructor, use the one the neighbouring tests use (`grep -n "Vsop87Backend::" crates/pleiades-vsop87/src/tests/backend.rs | head -3`). **If `d_ra`/`d_dec` match only with the opposite sign, or miss by more than 0.15″, stop and report. Do not change the expected values.**

- [ ] **Step 2: Run the test and confirm it fails.**
  Run: `cargo nextest run -p pleiades-vsop87 equatorial_channel_is_j2000_away_from_j2000`
  Expected: FAIL on the first assertion. The channel is still the of-date rotation, so the `d_ra`/`d_dec` assertions are never reached.

- [ ] **Step 3: Implement in VSOP87.** In `crates/pleiades-vsop87/src/backend.rs`, the `instant` parameter becomes unused, so drop it:

```rust
    fn to_equatorial(coords: HeliocentricCoordinates) -> EquatorialCoordinates {
        Self::to_ecliptic(coords).to_j2000_equatorial()
    }
```

  and at line 391:

```rust
        result.equatorial = Some(Self::to_equatorial(geocentric));
```

  Check for other callers with `grep -n "Self::to_equatorial\|Vsop87Backend::to_equatorial" crates/pleiades-vsop87/src` and update them the same way.

- [ ] **Step 4: Move the seven VSOP87 test expectations.** In `crates/pleiades-vsop87/src/tests/backend.rs`, replace each of the 7 lines

```rust
        let expected = ecliptic.to_equatorial(result.instant.mean_obliquity());
```

  with

```rust
        let expected = ecliptic.to_j2000_equatorial();
```

- [ ] **Step 5: Run the VSOP87 tests.**
  Run: `cargo nextest run -p pleiades-vsop87`
  Expected: all pass, including the new test. If a `source_docs`/`evidence` test fails, read it before changing it. `source_docs/evidence.rs:1919` already uses J2000-epoch values and should not move.

- [ ] **Step 6: Write the failing packaged-data test.** Add it to `crates/pleiades-data/src/tests/lookup.rs`:

```rust
// Issue #210: off J2000 the packaged equatorial channel is the J2000
// rotation, not the obliquity-of-date one.
#[test]
fn packaged_equatorial_channel_is_j2000_away_from_j2000() {
    use pleiades_backend::{CelestialBody, EphemerisBackend, EphemerisRequest};
    use pleiades_types::{Instant, JulianDay, TimeScale};
    let instant = Instant::new(JulianDay::from_days(2_415_020.5), TimeScale::Tt);
    let backend = PackagedDataBackend::new();
    for body in [CelestialBody::Sun, CelestialBody::Mars, CelestialBody::MeanNode] {
        let result = backend
            .position(&EphemerisRequest::new(body.clone(), instant))
            .unwrap();
        let ecliptic = result.ecliptic.expect("ecliptic");
        let equatorial = result.equatorial.expect("equatorial");
        let j2000 = ecliptic.to_j2000_equatorial();
        assert!(
            (equatorial.declination.degrees() - j2000.declination.degrees()).abs() < 1e-12,
            "{body:?}"
        );
        assert!(
            (equatorial.right_ascension.degrees() - j2000.right_ascension.degrees()).abs() < 1e-12,
            "{body:?}"
        );
    }
}
```

  `MeanNode` goes through `derived_point_position` (`backend.rs:93`). Sun and Mars go through the main lookup (`:507`). If `MeanNode` is not served at that epoch, use `CelestialBody::TrueNode`. Match the imports the file already uses.

- [ ] **Step 7: Run the test and confirm it fails.**
  Run: `cargo nextest run -p pleiades-data packaged_equatorial_channel_is_j2000_away_from_j2000`
  Expected: FAIL on the declination assertion.

- [ ] **Step 8: Implement in packaged data.** In `crates/pleiades-data/src/backend.rs`, at both `:93` and `:507`, replace

```rust
        let equatorial = ecliptic.to_equatorial(req.instant.mean_obliquity());
```

  with

```rust
        let equatorial = ecliptic.to_j2000_equatorial();
```

  At `:85`, change "mean-obliquity equatorial" to "J2000 equatorial (`to_j2000_equatorial`)".

- [ ] **Step 9: Move the two data test expectations.**
  - `lookup.rs:50`: `coordinates(reference).to_equatorial(reference.epoch.mean_obliquity())` becomes `coordinates(reference).to_j2000_equatorial()`.
  - `lookup.rs:1293-1296`: the comment becomes "Equatorial is the J2000 rotation of the ecliptic channel, like every other packaged body.", and `ecl.to_equatorial(instant.mean_obliquity())` becomes `ecl.to_j2000_equatorial()`.

- [ ] **Step 10: Run the data tests.**
  Run: `cargo nextest run -p pleiades-data`
  Expected: all pass. `backend_metadata_data_sources_is_stable` (`backend.rs:548`) still passes, because its wording changes in Task 5.

- [ ] **Step 11: Commit.**

```bash
cargo fmt --all
git add crates/pleiades-vsop87 crates/pleiades-data
git commit -m "fix(vsop87,data): give the equatorial channel on the J2000 equator (#210)"
```

---

### Task 3: JPL snapshot, corpus and SPK backends give J2000 RA/Dec

**Files:**
- Modify: `crates/pleiades-jpl/src/backend.rs:270`, `:341`, `:1883`
- Modify: `crates/pleiades-jpl/src/spk/backend.rs:218`
- Modify (parity self-checks): `crates/pleiades-jpl/src/reference_summary/comparison.rs:776`, `reference_snapshot/core/parity.rs:155`, `selected_asteroid.rs:2087` and `:2134`, `reference_asteroid.rs:733`
- Test: `crates/pleiades-jpl/src/spk/cross_check_tests.rs` (new test)
- Test (expectations move): `crates/pleiades-jpl/src/backend/tests.rs:430,465,801`, `reference_summary/holdout/tests.rs:628`, `reference_summary/reference_asteroid/tests.rs:247,296`, `reference_summary/selected_asteroid/tests.rs:146,194`

**Interfaces:**
- Consumes: `EclipticCoordinates::to_j2000_equatorial` (Task 1).

- [ ] **Step 1: Write the failing SPK test.** Add it to `crates/pleiades-jpl/src/spk/cross_check_tests.rs`, reusing its `const_seg` helper. The reference is the raw ICRF vector the kernel holds, taken before any rotation in the crate:

```rust
// Issue #210: the SPK equatorial channel is the kernel's own ICRF direction.
// The ε₀ rotation to the ecliptic and back must cancel at every epoch.
#[test]
fn spk_equatorial_channel_is_the_kernel_icrf_direction() {
    let body_icrf = [1.2e8, -3.4e7, 5.6e6]; // km, ICRF
    let blob = build_daf(&[
        const_seg(10, 0, body_icrf),
        const_seg(399, 3, [0.0, 0.0, 0.0]),
        const_seg(3, 0, [0.0, 0.0, 0.0]),
    ]);
    let backend = SpkBackend::builder()
        .add_kernel_bytes(blob, "x")
        .unwrap()
        .build();
    let (x, y, z) = (body_icrf[0], body_icrf[1], body_icrf[2]);
    let r = (x * x + y * y + z * z).sqrt();
    let want_ra = y.atan2(x).to_degrees().rem_euclid(360.0);
    let want_dec = (z / r).asin().to_degrees();
    for jd in [2_415_020.5, 2_451_545.0, 2_488_069.5] {
        let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
        let eq = backend
            .position(&pleiades_backend::EphemerisRequest::new(CelestialBody::Sun, inst))
            .unwrap()
            .equatorial
            .expect("equatorial");
        // 1e-9″ in degrees, with headroom for two rotations' round-off.
        let tol = 1e-9 / 3600.0 * 1e3;
        assert!((eq.right_ascension.degrees() - want_ra).abs() < tol, "JD {jd} RA");
        assert!((eq.declination.degrees() - want_dec).abs() < tol, "JD {jd} Dec");
    }
}
```

  The tolerance is 1e-6″, looser than the spec's 1e-9″. One ulp of a degree value near 300° is about 2e-10″, and the round trip ecliptic → equatorial runs a few dozen trigonometric operations. The 1e-6″ bound still sits far below the 5-45″ effect being fixed. If the residual exceeds 1e-6″ even at J2000, stop and report the measured value. Do not loosen the tolerance.

- [ ] **Step 2: Run the test and confirm it fails.**
  Run: `cargo nextest run -p pleiades-jpl spk_equatorial_channel_is_the_kernel_icrf_direction`
  Expected: passes at JD 2451545.0 and fails at JD 2415020.5 (first failing assertion `JD 2415020.5 Dec` or `RA`).

- [ ] **Step 3: Implement.** At each of `backend.rs:270`, `backend.rs:341` and `spk/backend.rs:218`, replace

```rust
        result.equatorial = Some(ecliptic.to_equatorial(req.instant.mean_obliquity()));
```

  with

```rust
        result.equatorial = Some(ecliptic.to_j2000_equatorial());
```

  At `backend.rs:1883`, `ecliptic.to_equatorial(sample.epoch.mean_obliquity())` becomes `ecliptic.to_j2000_equatorial()`.

- [ ] **Step 4: Move the parity self-checks to J2000.**
  - `comparison.rs:776`: `ecliptic.to_equatorial(result.instant.mean_obliquity())` becomes `ecliptic.to_j2000_equatorial()`.
  - `reference_snapshot/core/parity.rs:155`: the same change.
  - `selected_asteroid.rs:2087` and `:2134`: the same change.
  - `reference_asteroid.rs:733`: `exact_ecliptic.to_equatorial(exact_sample.epoch.mean_obliquity())` becomes `exact_ecliptic.to_j2000_equatorial()`.

- [ ] **Step 5: Move the test expectations.** In each test line listed under **Files**, replace `.to_equatorial(<x>.instant.mean_obliquity())`, `.to_equatorial(request.instant.mean_obliquity())` and `.to_equatorial(batch_result.instant.mean_obliquity())` with `.to_j2000_equatorial()`. Afterwards this should print nothing:

```bash
grep -rn "mean_obliquity()" crates/pleiades-jpl/src | grep -v "spk/cross_check_tests.rs"
```

  `cross_check_tests.rs:49,103` test the ecliptic reduction and keep `inst.mean_obliquity()`, which is evaluated at J2000.

- [ ] **Step 6: Run the JPL tests.**
  Run: `cargo nextest run -p pleiades-jpl`
  Expected: all pass. The holdout test (JD 2378498.5–2634167.0) and the multi-epoch reference snapshot tests fail before Step 5 and pass after it.

- [ ] **Step 7: Commit.**

```bash
cargo fmt --all
git add crates/pleiades-jpl
git commit -m "fix(jpl): give the snapshot and SPK equatorial channels on the J2000 equator (#210)"
```

---

### Task 4: The chart owns a mean placement's J2000 RA/Dec

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs`:
  - Insert after `let batch_mean = position.ecliptic;` (around line 461).
  - Replace the comment at `:619-624`.
- Modify: `crates/pleiades-core/src/chart/request.rs:557-568` (`with_apparentness` rustdoc)
- Modify: `crates/pleiades-core/src/chart/test_support.rs` (new wrapper backend)
- Test: `crates/pleiades-core/src/chart/tests.rs`:
  - Update `mean_fallback_keeps_backend_equatorial` (around line 3750).
  - Add the new tests.

**Interfaces:**
- Consumes: `EclipticCoordinates::to_j2000_equatorial` (Task 1).
- Produces: `test_support::ForeignEquatorialBackend<B>`, test-only.

- [ ] **Step 1: Add the test backend.** It wraps any backend and replaces a returned `equatorial` with a sentinel (RA 1°, Dec 2°) that no rotation of a real place produces. This stands in for ELP's of-date channel, and for any backend whose channel frame differs. Add it to `crates/pleiades-core/src/chart/test_support.rs`:

```rust
/// Wraps a backend and replaces any equatorial channel it returns with a
/// sentinel (RA 1°, Dec 2°), as a backend whose channel is in another frame
/// would (ELP's is of date). Optionally drops the ecliptic channel, or
/// advertises native sidereal support. Overrides the batch and no-motion paths
/// too, so every chart path sees the sentinel.
pub(super) struct ForeignEquatorialBackend<B> {
    pub(super) inner: B,
    pub(super) drop_ecliptic: bool,
    pub(super) native_sidereal: bool,
}

pub(super) fn sentinel_equatorial() -> pleiades_types::EquatorialCoordinates {
    pleiades_types::EquatorialCoordinates::new(
        pleiades_types::Angle::from_degrees(1.0),
        Latitude::from_degrees(2.0),
        Some(1.0),
    )
}

impl<B: EphemerisBackend> ForeignEquatorialBackend<B> {
    fn rewrite(&self, mut result: EphemerisResult) -> EphemerisResult {
        if result.equatorial.is_some() {
            result.equatorial = Some(sentinel_equatorial());
        }
        if self.drop_ecliptic {
            result.ecliptic = None;
        }
        result
    }
}

impl<B: EphemerisBackend> EphemerisBackend for ForeignEquatorialBackend<B> {
    fn metadata(&self) -> BackendMetadata {
        let mut metadata = self.inner.metadata();
        metadata.capabilities.native_sidereal = self.native_sidereal;
        metadata
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.inner.position(req).map(|r| self.rewrite(r))
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.inner.position_without_motion(req).map(|r| self.rewrite(r))
    }

    fn positions(&self, reqs: &[EphemerisRequest]) -> Result<Vec<EphemerisResult>, EphemerisError> {
        self.inner
            .positions(reqs)
            .map(|rs| rs.into_iter().map(|r| self.rewrite(r)).collect())
    }
}
```

  Check the exact signatures of `position_without_motion` and `positions` against `crates/pleiades-backend/src/traits.rs:42-60`, and the `native_sidereal` field name against `BackendCapabilities`. Adjust to match. Do not change the trait.

- [ ] **Step 2: Write the failing chart tests.** Add them to `crates/pleiades-core/src/chart/tests.rs` after `mean_fallback_keeps_backend_equatorial`, and replace that test with the J2000 version below:

```rust
const JD_1900: f64 = 2_415_020.5;

fn foreign(drop_ecliptic: bool, native_sidereal: bool) -> ChartEngine<test_support::ForeignEquatorialBackend<pleiades_data::PackagedDataBackend>> {
    ChartEngine::new(test_support::ForeignEquatorialBackend {
        inner: pleiades_data::PackagedDataBackend::new(),
        drop_ecliptic,
        native_sidereal,
    })
}

fn tt(jd: f64) -> Instant {
    Instant::new(pleiades_types::JulianDay::from_days(jd), TimeScale::Tt)
}

fn assert_j2000_of_backend_ecliptic(engine_jd: f64, p: &BodyPlacement) {
    // The reference is the backend's own J2000 ecliptic, read directly.
    use pleiades_backend::EphemerisRequest;
    let direct = pleiades_data::PackagedDataBackend::new()
        .position(&EphemerisRequest::new(p.body.clone(), tt(engine_jd)))
        .unwrap();
    let want = direct.ecliptic.unwrap().to_j2000_equatorial();
    let got = p.position.equatorial.expect("equatorial");
    assert!((got.right_ascension.degrees() - want.right_ascension.degrees()).abs() < 1e-9, "{:?}", p.body);
    assert!((got.declination.degrees() - want.declination.degrees()).abs() < 1e-9, "{:?}", p.body);
}

// Issue #210: whatever frame the backend's own channel is in, a mean
// placement reports the J2000 rotation of the backend's J2000 ecliptic.
#[test]
fn mean_chart_equatorial_is_j2000_whatever_the_backend_channel() {
    let snap = foreign(false, false)
        .chart(
            &ChartRequest::new(tt(JD_1900))
                .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
                .with_apparentness(Apparentness::Mean),
        )
        .unwrap();
    for body in [CelestialBody::Sun, CelestialBody::Mars] {
        assert_j2000_of_backend_ecliptic(JD_1900, snap.placement_for(&body).unwrap());
    }
}

#[test]
fn sidereal_mean_chart_equatorial_equals_the_tropical_one() {
    let request = |zodiac| {
        ChartRequest::new(tt(JD_1900))
            .with_bodies(vec![CelestialBody::Sun, CelestialBody::Mars])
            .with_apparentness(Apparentness::Mean)
            .with_zodiac_mode(zodiac)
    };
    let engine = foreign(false, false);
    let tropical = engine.chart(&request(ZodiacMode::Tropical)).unwrap();
    let sidereal = engine
        .chart(&request(ZodiacMode::Sidereal { ayanamsa: crate::Ayanamsa::Lahiri }))
        .unwrap();
    for body in [CelestialBody::Sun, CelestialBody::Mars] {
        let a = tropical.placement_for(&body).unwrap().position.equatorial.unwrap();
        let b = sidereal.placement_for(&body).unwrap().position.equatorial.unwrap();
        assert!((a.right_ascension.degrees() - b.right_ascension.degrees()).abs() < 1e-12, "{body:?}");
        assert!((a.declination.degrees() - b.declination.degrees()).abs() < 1e-12, "{body:?}");
        assert_j2000_of_backend_ecliptic(JD_1900, sidereal.placement_for(&body).unwrap());
    }
}

#[test]
fn mean_fallback_in_an_apparent_chart_is_j2000() {
    // The true node has no apparent reduction, so an apparent chart falls
    // back to its mean place (issue #170).
    let snap = foreign(false, false)
        .chart(
            &ChartRequest::new(tt(JD_1900))
                .with_bodies(vec![CelestialBody::Sun, CelestialBody::TrueNode])
                .with_apparentness(Apparentness::Apparent),
        )
        .unwrap();
    let node = snap.placement_for(&CelestialBody::TrueNode).unwrap();
    assert!(node.apparent.is_none(), "the node must fall back to mean");
    assert_j2000_of_backend_ecliptic(JD_1900, node);
    // The apparent Sun is still RA/Dec of date, not the sentinel or J2000.
    let sun = snap.placement_for(&CelestialBody::Sun).unwrap();
    assert!(sun.apparent.is_some());
    let eq = sun.position.equatorial.unwrap();
    let want = pleiades_apparent::apparent_equatorial_of_date(
        *sun.position.ecliptic.as_ref().unwrap(),
        JD_1900,
    )
    .unwrap();
    assert!((eq.declination.degrees() - want.declination.degrees()).abs() < 1e-9);
}

#[test]
fn native_sidereal_backend_keeps_its_equatorial_channel() {
    // Its ecliptic is sidereal, so the chart must not rotate it.
    let snap = foreign(false, true)
        .chart(
            &ChartRequest::new(tt(JD_1900))
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(Apparentness::Mean)
                .with_zodiac_mode(ZodiacMode::Sidereal { ayanamsa: crate::Ayanamsa::Lahiri }),
        );
    // A packaged backend asked for a sidereal zodiac may refuse the request.
    // That is acceptable here, since no rotation can then happen. If it
    // serves the request, the channel must be the backend's own.
    if let Ok(snap) = snap {
        let eq = snap.placement_for(&CelestialBody::Sun).unwrap().position.equatorial.unwrap();
        assert_eq!(eq, test_support::sentinel_equatorial());
    }
}

#[test]
fn equatorial_without_ecliptic_is_kept() {
    let snap = foreign(true, false).chart(
        &ChartRequest::new(tt(JD_1900))
            .with_bodies(vec![CelestialBody::Sun])
            .with_apparentness(Apparentness::Mean),
    );
    // A mean tropical chart needs no ecliptic. If the chart refuses the
    // request instead, record that in the PR and drop this test.
    let snap = snap.expect("a mean tropical chart without ecliptic");
    let eq = snap.placement_for(&CelestialBody::Sun).unwrap().position.equatorial.unwrap();
    assert_eq!(eq, test_support::sentinel_equatorial());
}

#[test]
fn a_backend_without_equatorial_still_gives_none() {
    // ToyChartBackend never fills the equatorial channel.
    let snap = ChartEngine::new(test_support::ToyChartBackend)
        .chart(
            &ChartRequest::new(tt(2_451_545.0))
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(Apparentness::Mean),
        )
        .unwrap();
    assert!(snap.placement_for(&CelestialBody::Sun).unwrap().position.equatorial.is_none());
}
```

  Replace `mean_fallback_keeps_backend_equatorial` with:

```rust
#[test]
fn mean_chart_equatorial_is_the_j2000_rotation() {
    // A mean-mode chart does not run the apparent path; equatorial is the
    // J2000 rotation of the backend's J2000 ecliptic (issue #210).
    let snap = ChartEngine::new(pleiades_data::PackagedDataBackend::new())
        .chart(
            &ChartRequest::new(tt(JD_1900))
                .with_bodies(vec![CelestialBody::Sun])
                .with_apparentness(Apparentness::Mean),
        )
        .unwrap();
    let p = snap.placement_for(&CelestialBody::Sun).unwrap();
    assert!(p.apparent.is_none(), "mean mode → no apparent provenance");
    assert_j2000_of_backend_ecliptic(JD_1900, p);
}
```

  Adjust these to the file's real names:
  - The placement type (`BodyPlacement`) and its field names (`body`, `apparent`, `position`): check `grep -n "pub struct .*Placement" -A 12 crates/pleiades-core/src/chart/*.rs`.
  - The imports already at the top of `tests.rs` (`ChartEngine`, `ChartRequest`, `Apparentness`, `ZodiacMode`, `TimeScale`, `Instant`).

  If `ToyChartBackend` sets `equatorial`, use another test backend that does not.

  Spec decision 5's failure case is left untested. In that case an apparent placement whose `apparent_equatorial_of_date` returns `Err` keeps the J2000 value. The step needs nutation to be unavailable at an in-window instant, and no test backend can force that without a production hook. The Step 4 code covers it by construction, because the J2000 value is set before the apparent block, which only overwrites on `Ok`. State this in the PR body.

- [ ] **Step 3: Run the tests and confirm which fail.**
  Run: `cargo nextest run --no-fail-fast -p pleiades-core chart::tests::`
  Expected:
  - FAIL: `mean_chart_equatorial_is_j2000_whatever_the_backend_channel`, `sidereal_mean_chart_equatorial_equals_the_tropical_one`, `mean_fallback_in_an_apparent_chart_is_j2000`, `mean_chart_equatorial_is_the_j2000_rotation`. Today they get the sentinel or the of-date rotation.
  - PASS: `native_sidereal_backend_keeps_its_equatorial_channel`, `equatorial_without_ecliptic_is_kept`, `a_backend_without_equatorial_still_gives_none`. These are Review Focus guards; they must stay green through Step 4.

- [ ] **Step 4: Implement.** In `crates/pleiades-core/src/chart/mod.rs`, directly after `let batch_mean = position.ecliptic;`:

```rust
                // A mean placement's equatorial coordinates are the J2000
                // rotation of the backend's J2000 ecliptic, whatever frame the
                // backend's own channel is in (ELP's is of date; issue #210).
                // A successful apparent reduction below replaces them with
                // RA/Dec of date. A natively sidereal backend's ecliptic is
                // sidereal, so its channel is left as it is.
                if !native_sidereal && position.equatorial.is_some() {
                    if let Some(ecliptic) = position.ecliptic {
                        position.equatorial = Some(ecliptic.to_j2000_equatorial());
                    }
                }
```

  Replace the comment at `:619-624` with:

```rust
                // Derive the apparent equatorial of date from the final tropical
                // apparent ecliptic (geocentric or topocentric), BEFORE the
                // sidereal longitude shift so RA/Dec stay ayanamsa-independent.
                // Mean-fallback rows (apparent.is_none()), and rows whose
                // nutation is unavailable, keep the J2000 RA/Dec set above.
```

- [ ] **Step 5: Run the chart tests.**
  Run: `cargo nextest run --no-fail-fast -p pleiades-core`
  Expected: all pass, including `apparent_chart_populates_equatorial_of_date` and `equatorial_is_identical_tropical_vs_sidereal`, which are unchanged. If a chart golden or snapshot test fails on a mean chart's RA/Dec, that is expected movement. Update it only if its old value was the of-date rotation, and say so in the commit message.

- [ ] **Step 6: Update the `with_apparentness` rustdoc.** In `crates/pleiades-core/src/chart/request.rs`, replace the sentence starting "The equatorial coordinates of a mean placement are the backend's J2000 right ascension and declination in every zodiac…" with:

```rust
    /// one (see [`ChartRequest::with_zodiac_mode`]). The equatorial
    /// coordinates of a mean placement are J2000 right ascension and
    /// declination in every zodiac: the chart rotates the backend's J2000
    /// ecliptic by the J2000 obliquity itself, whatever frame the backend's
    /// own equatorial channel is in (issue #210), so a sidereal mean
    /// placement has an ecliptic place on the equinox of date beside J2000
    /// equatorial coordinates. House cusps and angles are on the equinox of
    /// date in every chart.
```

  Keep the lines before "one (see" as they are.

- [ ] **Step 7: Run docs and commit.**

```bash
cargo fmt --all
cargo clippy -p pleiades-core --all-targets --all-features -- -D warnings
git add crates/pleiades-core/src/chart
git commit -m "fix(core): give a mean placement J2000 right ascension and declination on every backend (#210)"
```

---

### Task 5: Say which obliquity, and state the channel contract

**Files:**
- Modify: `crates/pleiades-backend/src/result.rs:59-60` (the `equatorial` field rustdoc)
- Modify: `spec/backend-trait.md` (the result section, near line 57)
- Modify (prose): every production string listed in Step 3
- Test: the pinning tests that fail in Step 4

**Interfaces:** none.

- [ ] **Step 1: State the contract on the field.** In `crates/pleiades-backend/src/result.rs`, replace the `equatorial` doc ("Equatorial coordinates when available.") with:

```rust
    /// Equatorial coordinates when available, in the frame the backend
    /// documents. The first-party backends give the J2000 mean equator and
    /// equinox (their J2000 ecliptic rotated by
    /// [`OBLIQUITY_J2000_DEG`](pleiades_types::OBLIQUITY_J2000_DEG)), except
    /// `ElpBackend`, whose channel is right ascension and declination of
    /// date. A chart does not pass this channel through; it computes a mean
    /// placement's J2000 coordinates itself (issue #210).
```

  If `pleiades_types` is not a dependency path that rustdoc can resolve from this file, write the constant name in backticks without the link. A public-to-private intra-doc link fails `mise run docs`.

- [ ] **Step 2: State it in the spec.** In `spec/backend-trait.md`, add after the bullet that mentions "desired coordinate frame":

```markdown
- The `equatorial` channel's frame is part of each backend's documented
  contract. First-party backends give the J2000 mean equator and equinox
  (the J2000 ecliptic rotated by the J2000 mean obliquity), except the ELP
  lunar backend, which gives right ascension and declination of date. Chart
  assembly does not pass the channel through for mean placements (#210).
```

- [ ] **Step 3: Reword the production strings.** Use "J2000 mean-obliquity" in place of "mean-obliquity" in exactly these strings. Leave ELP files, `render/text/evidence.rs`, the CLI command names and `posture/elp/lib_summaries.rs` alone.

  | File:line | Old fragment | New fragment |
  |---|---|---|
  | `crates/pleiades-validate/src/posture/backend_policy.rs:38` | `equatorial output is derived via mean-obliquity transforms when supported` | `equatorial output is derived via J2000 mean-obliquity transforms when supported` |
  | `crates/pleiades-vsop87/src/profiles.rs:319` | `derived with a mean-obliquity transform` | `derived with a J2000 mean-obliquity transform` |
  | `crates/pleiades-vsop87/src/source_docs/spec.rs:287` | same | same |
  | `crates/pleiades-vsop87/src/source_docs/request_corpus.rs:388,486` (doc) | `keep the mean-obliquity equatorial frame` | `keep the J2000 mean-obliquity equatorial frame` |
  | `crates/pleiades-jpl/src/reference_summary/jpl_posture.rs:595` | `derived with a mean-obliquity transform` | `derived with a J2000 mean-obliquity transform` |
  | `crates/pleiades-jpl/src/backend.rs:181` | `mean-obliquity equatorial` | `J2000 mean-obliquity equatorial` |
  | `crates/pleiades-jpl/src/lib.rs:13` (doc) | `mean-obliquity equatorial frame` | `J2000 mean-obliquity equatorial frame` |
  | `crates/pleiades-jpl/src/reference_summary/reference_asteroid.rs:368,371,402` | `"mean-obliquity equatorial transform"` | `"J2000 mean-obliquity equatorial transform"` (all three: the check and the value must agree) |
  | `crates/pleiades-jpl/src/reference_summary/reference_asteroid.rs:507,582,586,590,673` | `derived mean-obliquity transform` / `mean-obliquity transform` | `derived J2000 mean-obliquity transform` / `J2000 mean-obliquity transform` |
  | `crates/pleiades-validate/src/posture/jpl/holdout.rs:219` | `; mean-obliquity transform against` | `; J2000 mean-obliquity transform against` |
  | `crates/pleiades-validate/src/posture/jpl/reference_snapshot/core/parity.rs:48` | same | same |
  | `crates/pleiades-validate/src/posture/jpl/reference_asteroid.rs:300` | `using a mean-obliquity eq…` | `using a J2000 mean-obliquity eq…` |
  | `crates/pleiades-data/src/lookup.rs:387` | `stored channels and mean-obliquity transform` | `stored channels and J2000 mean-obliquity transform` |
  | `crates/pleiades-data/src/lib.rs:21` (doc) | `mean-obliquity transform` | `J2000 mean-obliquity transform` |
  | `crates/pleiades-compression/src/format.rs:358` (doc) | `mean-obliquity policy` | `J2000 mean-obliquity policy` |
  | `crates/pleiades-compression/src/artifact.rs:314` (doc) | `used for the mean-obliquity frame rotation` | `used for the mean-obliquity frame rotation (the packaged backend passes the J2000 obliquity)` |
  | `crates/pleiades-types/src/time.rs:345` (doc) | `the workspace for mean-obliquity equatorial transforms` | `the workspace for precession-era obliquity values; the backends' J2000 equatorial channel uses OBLIQUITY_J2000_DEG instead` |
  | `docs/time-observer-policy.md:102` | `both use a mean-obliquity transform` | `the VSOP87 path rotates its J2000 ecliptic by the J2000 mean obliquity, and the compact ELP lunar baseline gives right ascension and declination of date` |

  Before editing, confirm each line still holds the old fragment (`sed -n '<line>p' <file>`). Line numbers may have moved after Tasks 2-4.

- [ ] **Step 4: Find and update the pinning tests.**
  Run: `cargo nextest run --no-fail-fast -p pleiades-vsop87 -p pleiades-jpl -p pleiades-data -p pleiades-cli` and `cargo test -p pleiades-validate -- --include-ignored`. The second takes a while. Run it in the foreground, and make no edits while it runs.

  In every failing assertion that pins one of the Step 3 strings, apply the same fragment change. Known pins:
  - `pleiades-validate/src/posture/backend_policy.rs:1454-1519,1768`
  - `validate/src/tests/snapshot_render.rs:808,1868`
  - `validate/src/tests/release_checklist.rs:479,702`
  - `validate/src/tests/release_bundle_verify_a.rs:467,746,1071`
  - `validate/src/posture/jpl/jpl_posture.rs:1312-1319`
  - `validate/src/posture/jpl/holdout.rs:776`
  - `validate/src/posture/jpl/reference_snapshot/core/parity.rs:214`
  - `validate/src/posture/jpl/reference_asteroid.rs:318`
  - `pleiades-vsop87/src/tests/{profiles.rs:55, evidence.rs:1487-1490, documentation.rs:48,256-276,796}`
  - `pleiades-cli/src/cli/tests/{validation.rs:109, summary_commands.rs:2472,2606}`
  - `pleiades-data/src/backend.rs:548`

  `backend_policy.rs:1454-1519` pins an input string that the test supplies itself ("geocentric ecliptic inputs; …"). Change those only if the test fails. Assertions of the form `.contains("mean-obliquity")` still pass and stay as they are.

- [ ] **Step 5: Re-run and confirm green.** Same two commands as Step 4. Expected: all pass.

- [ ] **Step 6: Commit.**

```bash
cargo fmt --all
git add -A
git commit -m "docs(backend,validate,jpl,vsop87,data): say the equatorial channel uses the J2000 obliquity (#210)"
```

---

### Task 6: Release bookkeeping, full validation and PR

**Files:**
- Modify: `crates/pleiades-core/src/compatibility/mod.rs`:
  - `:26`: profile id.
  - `:42`: `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`.
  - Release-notes list: append after the #208 entry at about `:126`.

- [ ] **Step 1: Add the release note.** Append this as the last element of the release-notes array, after the "Crossing, station, aspect and occultation searches at the window's ends (issue #208)…" entry:

```rust
            "J2000 equatorial channel (issue #210): Vsop87Backend, PackagedDataBackend, JplSnapshotBackend, SnapshotCorpusBackend and SpkBackend now fill EphemerisResult::equatorial by rotating their J2000 ecliptic with the J2000 mean obliquity (EclipticCoordinates::to_j2000_equatorial), where they used the mean obliquity of date, a mixed frame up to about 45 arcseconds a century from J2000. A chart's mean placement (a Mean chart, or the mean fallback inside an Apparent chart) now reports that J2000 right ascension and declination on every backend, ELP included, computed by the chart from the backend's J2000 ecliptic; ElpBackend's own channel stays right ascension and declination of date. Apparent placements, ecliptic coordinates and the equatorial gates are unchanged. Behaviour change for a caller reading a mean placement's or one of these backends' equatorial coordinates away from J2000. Compatibility profile bumped to 0.7.32; API stability profile unchanged.",
```

- [ ] **Step 2: Bump the profile id.** `"pleiades-compatibility-profile/0.7.31"` becomes `"pleiades-compatibility-profile/0.7.32"`.

- [ ] **Step 3: Re-measure the checksum.**
  Run: `cargo nextest run -p pleiades-core rendered_profile_matches_pinned_content_checksum`
  Expected: FAIL, with the newly rendered checksum in the message. Copy that value into `CURRENT_COMPATIBILITY_PROFILE_CONTENT_CHECKSUM`, keeping the `0x____` grouping, and re-run it until it passes. If the message does not print the value, read the test to see how it computes the checksum and print it with a temporary `dbg!`. Remove the `dbg!` afterwards.

  Then run `grep -rn "0.7.31" crates --include=*.rs` and update any test that pins the old profile id.

- [ ] **Step 4: Full blocking validation.** Run in the foreground, with no edits while it runs:
  Run: `mise run ci`
  Expected: exit 0. Release-smoke prints `ok` for every line, and the equatorial gates pass unchanged. If `mise run docs` fails on an intra-doc link added in Task 5, fix the link and re-run.

- [ ] **Step 5: Slow tier.**
  Run: `mise run test-full`
  Expected: exit 0.

- [ ] **Step 6: Commit and open the PR.**

```bash
cargo fmt --all
git add -A
git commit -m "chore(core): record the J2000 equatorial channel in the compatibility profile (#210)"
git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin HEAD:fix-210-j2000-equatorial
gh pr create --base main --head fix-210-j2000-equatorial \
  --title "fix(backend,core): put the equatorial channel and mean charts on J2000 (#210)" \
  --body-file /tmp/claude-1000/-workspace/ba8c2bab-ce9e-4509-a5f1-b0845edc52ad/scratchpad/pr-210.md
```

  Write `pr-210.md` first. It contains:
  - `Closes #210`, and the spec and plan paths.
  - One bullet per task, saying what changed.
  - The Mars `d_ra`/`d_dec` values Task 2's test measured.
  - The untested `Err` path note from Task 4.
  - The exit codes and test counts of `mise run ci` and `mise run test-full`.

  The PR is squash-merged when its checks are green, so the single `fix(...)` title is what release-plz reads.
