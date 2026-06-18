# SP1 — Dense de440-Backed Generation Source + Accuracy Baseline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the packaged-artifact generator fit each segment against de440 sampled densely *within* the segment span (kernel-gated), regenerate the draft artifact, and produce a committed per-body accuracy baseline measured against the hold-out corpus.

**Architecture:** A new configurable-degree least-squares polynomial fitter samples `SpkBackend` (de440) at many points inside each per-body segment span and fits Longitude/Latitude/DistanceAu channels, decoupling sample density from segment count. Generation becomes kernel-gated (mirroring the existing `corpus_regen` contract); runtime still decodes the committed `.bin`; a kernel-free accuracy-baseline summary compares the decoded artifact to the committed hold-out.

**Tech Stack:** Rust workspace (`pleiades-*` crates). `cargo test`, `cargo clippy`, the `pleiades-validate`/`pleiades-cli` `validate` subcommands, and the `PLEIADES_DE_KERNEL`-gated de440 path.

## Global Constraints

- Pure-Rust, no new dependencies; preserve crate layering (`pleiades-jpl → pleiades-data → pleiades-validate`).
- The artifact stays **draft-grade**. Do NOT define or enforce accuracy thresholds (SP3). Do NOT budget/optimize size or latency (SP3). Do NOT tune spans/degrees for accuracy beyond the documented defaults (SP2).
- Asteroid (Eros) coverage stays **constrained to its 1900–2100 corpus window**.
- Corpus/sample frame is geocentric **ecliptic** (mean geometric), TDB epochs; channels are ecliptic **Longitude**/**Latitude** (degrees) and **DistanceAu**.
- **de440 kernel:** all real artifact regeneration and de440 comparisons are gated behind `PLEIADES_DE_KERNEL` (path to `de440.bsp`), exactly like the existing `crates/pleiades-jpl/tests/corpus_regen.rs`. The kernel is NOT committed; its SHA-256 is pinned in `corpus_spec::KERNEL_SHA256`. Kernel-free verification = committed artifact bytes + checksum + hold-out spot-checks.
- **Generation cost:** dense within-span fitting over 1600–2600 CE is heavy and the artifact grows large; this is expected and *measured* (not budgeted) in SP1.
- Generation must be **deterministic** (byte-identical regeneration).
- Run `cargo fmt -p <crate>` (scoped) and `cargo clippy -p <crate> --all-targets -- -D warnings` clean before each commit. NOTE: some pre-existing `pleiades-jpl` files (asteroid bins, `spk/*`, `tests/corpus_regen.rs`) carry committed rustfmt drift; running unscoped `cargo fmt` reformats them — never stage those collateral changes.

## Kernel-free vs kernel-gated tasks

- **Kernel-free (fully verifiable by any worker):** Task 1 (spec constants), Task 2 (LSQ fitter + within-span fit core, tested against analytic backends), Task 3 (per-body segment generation, tested with a synthetic backend), Task 5 (baseline measurement code, tested against a synthetic artifact), Task 7 (docs).
- **Kernel-gated (require `PLEIADES_DE_KERNEL` + a long-running environment):** Task 4 (regenerate the committed `.bin` from de440) and Task 6 (gated determinism/reproduce test + real size/perf + real baseline numbers). These are slow (potentially hours) and must be run where the kernel is available.

## File Structure

- `crates/pleiades-data/src/coverage/generation_spec.rs` (**create**) — per-body segment-span/degree/oversample constants + segment-boundary generation across the 1600–2600 window. One responsibility: the fitting cadence model.
- `crates/pleiades-data/src/coverage/lsq.rs` (**create**) — configurable-degree least-squares polynomial fitter (power basis). One responsibility: numerical fit.
- `crates/pleiades-data/src/regenerate.rs` (**modify**) — within-span sample-and-fit segment builder; kernel-gated artifact build from `SpkBackend`; reuse existing `unwrap_longitude_samples`.
- `crates/pleiades-data/src/coverage/mod.rs` (**modify**) — register `generation_spec`, `lsq`.
- `crates/pleiades-data/src/accuracy_baseline.rs` (**create**) — decode-vs-hold-out per-body/channel error envelope + validated summary.
- `crates/pleiades-data/src/lib.rs` (**modify**) — register `accuracy_baseline`; re-export its summary.
- `crates/pleiades-validate/src/render/cli.rs` (**modify**) — add the `packaged-artifact-accuracy-baseline-summary` command and make artifact regeneration kernel-gated.
- `crates/pleiades-data/tests/fixtures/packaged-artifact.bin` (**modify**, Task 4, kernel) — regenerated draft artifact.
- `PLAN.md`, `plan/status/*` (**modify**, Task 7).

---

### Task 1: Generation spec — per-body span/degree/oversample model

**Files:**
- Create: `crates/pleiades-data/src/coverage/generation_spec.rs`
- Modify: `crates/pleiades-data/src/coverage/mod.rs` (add `pub mod generation_spec; pub use generation_spec::*;`)
- Test: inline `#[cfg(test)]` in `generation_spec.rs`

**Interfaces:**
- Consumes: `pleiades_backend::CelestialBody`.
- Produces:
  - `pub fn fitting_segment_span_days(body: &CelestialBody) -> f64`
  - `pub fn fitting_degree(body: &CelestialBody) -> usize`
  - `pub const FITTING_OVERSAMPLE: usize` (= 3)
  - `pub fn fitting_within_span_sample_count(body: &CelestialBody) -> usize` (= `(fitting_degree(body) + 1) * FITTING_OVERSAMPLE`)
  - `pub fn fitting_segment_boundaries(body: &CelestialBody, start_jd: f64, end_jd: f64) -> Vec<(f64, f64)>` (consecutive `[t0, t1]` spans tiling `[start_jd, end_jd]`, last span clamped to `end_jd`).

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::CelestialBody;

    #[test]
    fn spans_match_documented_defaults() {
        assert_eq!(fitting_segment_span_days(&CelestialBody::Moon), 4.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Mercury), 8.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Venus), 16.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Sun), 16.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Mars), 32.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Jupiter), 128.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Saturn), 256.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Uranus), 512.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Neptune), 512.0);
        assert_eq!(fitting_segment_span_days(&CelestialBody::Pluto), 512.0);
    }

    #[test]
    fn within_span_sample_count_oversamples_degree() {
        let n = fitting_within_span_sample_count(&CelestialBody::Moon);
        assert_eq!(n, (fitting_degree(&CelestialBody::Moon) + 1) * FITTING_OVERSAMPLE);
        assert!(n > fitting_degree(&CelestialBody::Moon) + 1, "must oversample");
    }

    #[test]
    fn boundaries_tile_the_window_without_gaps_or_overlap() {
        let spans = fitting_segment_boundaries(&CelestialBody::Jupiter, 1000.0, 1500.0);
        assert_eq!(spans.first().unwrap().0, 1000.0);
        assert_eq!(spans.last().unwrap().1, 1500.0);
        for pair in spans.windows(2) {
            assert_eq!(pair[0].1, pair[1].0, "spans must be contiguous");
        }
        for (t0, t1) in &spans {
            assert!(t1 > t0, "each span is non-empty");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data --lib generation_spec::tests`
Expected: FAIL to compile — module/functions not defined.

- [ ] **Step 3: Write minimal implementation**

```rust
//! Per-body fitting cadence model for dense de440-backed artifact generation.
//!
//! Spans are accuracy-safe initial defaults (SP1); SP2 tunes them against the
//! measured accuracy baseline. Within-span sampling oversamples the polynomial
//! degree so each segment's least-squares fit is over-determined.

use pleiades_backend::CelestialBody;

/// Oversample factor: within-span sample count = (degree + 1) * this.
pub const FITTING_OVERSAMPLE: usize = 3;

/// Per-body segment span in days (initial SP1 defaults; tuned in SP2).
pub fn fitting_segment_span_days(body: &CelestialBody) -> f64 {
    match body {
        CelestialBody::Moon => 4.0,
        CelestialBody::Mercury => 8.0,
        CelestialBody::Venus | CelestialBody::Sun => 16.0,
        CelestialBody::Mars => 32.0,
        CelestialBody::Jupiter => 128.0,
        CelestialBody::Saturn => 256.0,
        CelestialBody::Uranus | CelestialBody::Neptune | CelestialBody::Pluto => 512.0,
        // Constrained asteroids (e.g. Eros) use a Mars-like span; only generated
        // within their own corpus window by the caller.
        _ => 16.0,
    }
}

/// Per-body polynomial degree for the within-span fit (SP1 default).
pub fn fitting_degree(_body: &CelestialBody) -> usize {
    8
}

/// Number of de440 samples taken within each segment span.
pub fn fitting_within_span_sample_count(body: &CelestialBody) -> usize {
    (fitting_degree(body) + 1) * FITTING_OVERSAMPLE
}

/// Contiguous `[t0, t1]` spans tiling `[start_jd, end_jd]`, last clamped to `end_jd`.
pub fn fitting_segment_boundaries(
    body: &CelestialBody,
    start_jd: f64,
    end_jd: f64,
) -> Vec<(f64, f64)> {
    let span = fitting_segment_span_days(body);
    let mut spans = Vec::new();
    let mut t0 = start_jd;
    while t0 < end_jd {
        let t1 = (t0 + span).min(end_jd);
        if t1 > t0 {
            spans.push((t0, t1));
        }
        t0 = t1;
    }
    spans
}
```

Register in `crates/pleiades-data/src/coverage/mod.rs` alongside the other `pub mod`/`pub use` lines:

```rust
pub mod generation_spec;
pub use generation_spec::*;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-data --lib generation_spec::tests`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-data && cargo clippy -p pleiades-data --all-targets -- -D warnings
git add crates/pleiades-data/src/coverage/generation_spec.rs crates/pleiades-data/src/coverage/mod.rs
git commit -m "feat(data): add per-body fitting cadence model for dense generation"
```

---

### Task 2: Configurable-degree least-squares polynomial fitter + within-span fit core

**Files:**
- Create: `crates/pleiades-data/src/coverage/lsq.rs`
- Modify: `crates/pleiades-data/src/coverage/mod.rs` (add `pub mod lsq; pub use lsq::*;`)
- Modify: `crates/pleiades-data/src/regenerate.rs` (add `fit_segment_within_span`)
- Test: inline `#[cfg(test)]` in `lsq.rs`; new test in the regenerate test module for `fit_segment_within_span`

**Interfaces:**
- Consumes: Task 1 (`fitting_degree`, `fitting_within_span_sample_count`); `pleiades_backend::{EphemerisBackend, EphemerisRequest, CelestialBody, Instant, JulianDay, TimeScale}`; existing `unwrap_longitude_samples` in `regenerate.rs`; `pleiades_compression::{PolynomialChannel, ChannelKind, Segment}`; existing `channel_from_fit_samples_with_control_points` / `distance_channel_from_fit_samples`.
- Produces:
  - `pub fn fit_polynomial_lsq(samples: &[(f64, f64)], degree: usize) -> Option<Vec<f64>>` — power-basis coefficients (ascending), least-squares over normalized x; `None` if under-determined or singular.
  - `fn fit_segment_within_span(body: &CelestialBody, t0_jd: f64, t1_jd: f64, reference: &dyn EphemerisBackend) -> Option<Segment>` (in `regenerate.rs`) — samples `reference` within `[t0, t1]`, fits Longitude/Latitude/DistanceAu channels, returns a `Segment`.

- [ ] **Step 1: Write the failing test (LSQ fitter)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_known_cubic_exactly() {
        // y = 1 + 2x - 3x^2 + 0.5x^3 sampled at 12 points over [0,1].
        let f = |x: f64| 1.0 + 2.0 * x - 3.0 * x * x + 0.5 * x * x * x;
        let samples: Vec<(f64, f64)> =
            (0..12).map(|i| { let x = i as f64 / 11.0; (x, f(x)) }).collect();
        let coeffs = fit_polynomial_lsq(&samples, 3).expect("fit should succeed");
        let expected = [1.0, 2.0, -3.0, 0.5];
        assert_eq!(coeffs.len(), 4);
        for (got, want) in coeffs.iter().zip(expected) {
            assert!((got - want).abs() < 1e-6, "coeff {got} vs {want}");
        }
    }

    #[test]
    fn underdetermined_returns_none() {
        let samples = [(0.0, 1.0), (1.0, 2.0)];
        assert!(fit_polynomial_lsq(&samples, 5).is_none());
    }

    #[test]
    fn fits_higher_degree_smooth_function_within_tolerance() {
        // sin over [0,1] fit at degree 8 with oversampling -> tiny residual.
        let f = |x: f64| (x * std::f64::consts::PI).sin();
        let samples: Vec<(f64, f64)> =
            (0..27).map(|i| { let x = i as f64 / 26.0; (x, f(x)) }).collect();
        let coeffs = fit_polynomial_lsq(&samples, 8).expect("fit");
        let eval = |x: f64| coeffs.iter().rev().fold(0.0, |acc, c| acc * x + c);
        for i in 0..50 {
            let x = i as f64 / 49.0;
            assert!((eval(x) - f(x)).abs() < 1e-4, "residual too large at {x}");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data --lib lsq::tests`
Expected: FAIL to compile — `fit_polynomial_lsq` not defined.

- [ ] **Step 3: Write minimal implementation (LSQ fitter)**

Create `crates/pleiades-data/src/coverage/lsq.rs`:

```rust
//! Configurable-degree least-squares polynomial fitting (power basis).
//!
//! Solves the normal equations `(VᵀV) c = Vᵀy` for the Vandermonde matrix `V`
//! of the sample x-values, returning ascending power-basis coefficients. Used by
//! the dense within-span segment fitter.

/// Fits a degree-`degree` polynomial to `(x, y)` samples by least squares.
/// Returns ascending power-basis coefficients, or `None` if there are fewer
/// samples than coefficients or the normal-equations matrix is singular.
pub fn fit_polynomial_lsq(samples: &[(f64, f64)], degree: usize) -> Option<Vec<f64>> {
    let n_coeffs = degree + 1;
    if samples.len() < n_coeffs {
        return None;
    }
    // Build normal-equations matrix A (n_coeffs x n_coeffs) and rhs b.
    let mut a = vec![vec![0.0f64; n_coeffs]; n_coeffs];
    let mut b = vec![0.0f64; n_coeffs];
    for &(x, y) in samples {
        let mut powers = vec![1.0f64; n_coeffs];
        for k in 1..n_coeffs {
            powers[k] = powers[k - 1] * x;
        }
        for i in 0..n_coeffs {
            b[i] += powers[i] * y;
            for j in 0..n_coeffs {
                a[i][j] += powers[i] * powers[j];
            }
        }
    }
    solve_linear_system(a, b)
}

/// Gaussian elimination with partial pivoting. Returns `None` if singular.
fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        for row in (col + 1)..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in (col + 1)..n {
            let factor = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= factor * a[col][k];
            }
            b[row] -= factor * b[col];
        }
    }
    let mut x = vec![0.0f64; n];
    for row in (0..n).rev() {
        let mut sum = b[row];
        for k in (row + 1)..n {
            sum -= a[row][k] * x[k];
        }
        x[row] = sum / a[row][row];
    }
    if x.iter().all(|v| v.is_finite()) { Some(x) } else { None }
}
```

Register in `coverage/mod.rs`:

```rust
pub mod lsq;
pub use lsq::*;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-data --lib lsq::tests`
Expected: PASS (3 tests).

- [ ] **Step 5: Write the failing test (within-span fit core)**

Add to the regenerate test module (`crates/pleiades-data/src/tests/coverage.rs` or wherever regenerate tests live). It uses a synthetic backend implementing `EphemerisBackend` with an analytic, smooth ecliptic position so no kernel is needed:

```rust
#[test]
fn fit_segment_within_span_reproduces_a_smooth_synthetic_body() {
    use pleiades_backend::{CelestialBody, CoordinateFrame, EphemerisBackend, EphemerisError,
        EphemerisRequest, EphemerisResult, Instant, JulianDay, TimeScale, ZodiacMode, Apparentness};
    use pleiades_backend::BackendId;

    // Synthetic backend: longitude advances 1 deg/day, small latitude wobble,
    // distance ~1 AU — smooth over a 16-day span, so a degree-8 fit nails it.
    struct Synthetic;
    impl EphemerisBackend for Synthetic {
        fn metadata(&self) -> pleiades_backend::BackendMetadata { unimplemented!() }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let jd = req.instant.julian_day.days();
            let lon = (jd * 1.0).rem_euclid(360.0);
            let lat = 0.1 * (jd / 50.0).sin();
            let dist = 1.0 + 0.01 * (jd / 80.0).cos();
            let mut r = EphemerisResult::new(BackendId::new("synthetic"), req.body.clone(),
                req.instant, req.frame, req.zodiac_mode.clone(), req.apparent);
            r.ecliptic = Some(pleiades_backend::EclipticCoordinates::from_lon_lat_distance(lon, lat, dist));
            Ok(r)
        }
    }

    let body = CelestialBody::Sun;
    let (t0, t1) = (2_451_545.0, 2_451_545.0 + 16.0);
    let seg = crate::regenerate::fit_segment_within_span(&body, t0, t1, &Synthetic)
        .expect("fit should succeed");
    // The segment spans [t0, t1] and carries the three channels.
    assert_eq!(seg.start.julian_day.days(), t0);
    assert_eq!(seg.end.julian_day.days(), t1);
    assert!(seg.channels.iter().any(|c| matches!(c.kind, pleiades_compression::ChannelKind::Longitude)));
    assert!(seg.channels.iter().any(|c| matches!(c.kind, pleiades_compression::ChannelKind::Latitude)));
    assert!(seg.channels.iter().any(|c| matches!(c.kind, pleiades_compression::ChannelKind::DistanceAu)));
}
```

Match the REAL `EphemerisResult`/`EclipticCoordinates` constructors as they exist (inspect `regenerate.rs` and `pleiades-backend`); if `from_lon_lat_distance` is not the real name, build the ecliptic coordinate the same way existing code does. Do not invent APIs.

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo test -p pleiades-data --lib fit_segment_within_span_reproduces`
Expected: FAIL — `fit_segment_within_span` not defined.

- [ ] **Step 7: Implement `fit_segment_within_span` in `regenerate.rs`**

```rust
/// Fits one segment over `[t0_jd, t1_jd]` by sampling `reference` (de440 or a
/// test backend) at the body's within-span sample count and least-squares
/// fitting Longitude/Latitude/DistanceAu channels over the normalized interval.
pub(crate) fn fit_segment_within_span(
    body: &CelestialBody,
    t0_jd: f64,
    t1_jd: f64,
    reference: &dyn EphemerisBackend,
) -> Option<Segment> {
    use crate::coverage::{fit_polynomial_lsq, fitting_degree, fitting_within_span_sample_count};

    let n = fitting_within_span_sample_count(body).max(fitting_degree(body) + 1);
    let span = t1_jd - t0_jd;
    if span <= 0.0 {
        return None;
    }

    let mut xs = Vec::with_capacity(n);
    let mut lon_deg = Vec::with_capacity(n);
    let mut lat = Vec::with_capacity(n);
    let mut dist = Vec::with_capacity(n);
    for i in 0..n {
        let frac = i as f64 / (n as f64 - 1.0);
        let jd = t0_jd + frac * span;
        let inst = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
        let res = reference.position(&EphemerisRequest::new(body.clone(), inst)).ok()?;
        let ec = res.ecliptic?;
        xs.push(frac);
        lon_deg.push(ec.longitude.degrees());
        lat.push(ec.latitude.degrees());
        dist.push(ec.distance_au.unwrap_or_default());
    }

    // Unwrap longitude to a continuous series before fitting (reuse existing helper).
    let lon_unwrapped = unwrap_longitude_samples(&lon_deg);

    let degree = fitting_degree(body);
    let to_samples = |ys: &[f64]| -> Vec<(f64, f64)> {
        xs.iter().copied().zip(ys.iter().copied()).collect()
    };

    let lon_coeffs = fit_polynomial_lsq(&to_samples(&lon_unwrapped), degree)?;
    let lat_coeffs = fit_polynomial_lsq(&to_samples(&lat), degree)?;
    let dist_coeffs = fit_polynomial_lsq(&to_samples(&dist), degree)?;

    let channels = vec![
        PolynomialChannel::new(ChannelKind::Longitude, 6, lon_coeffs),
        PolynomialChannel::new(ChannelKind::Latitude, 6, lat_coeffs),
        PolynomialChannel::new(ChannelKind::DistanceAu, 10, dist_coeffs),
    ];
    let seg = Segment::new(
        Instant::new(JulianDay::from_days(t0_jd), TimeScale::Tdb),
        Instant::new(JulianDay::from_days(t1_jd), TimeScale::Tdb),
        channels,
    );
    seg.validate_channels().ok()?; // if no such method exists, validate each channel via PolynomialChannel::validate
    Some(seg)
}
```

Notes for the implementer:
- `unwrap_longitude_samples` already exists in `regenerate.rs` — check its exact signature (it may take `&[f64]` or `(reference, &[f64])`); adapt the call. If it operates pairwise, fold it over the series.
- `scale_exponent` values (6 for angles, 10 for distance) mirror the existing channel constructors in `regenerate.rs`/`threshold.rs`. Match what the current code uses so encoding/quantization is consistent.
- Validate channel coefficients are finite before returning (`PolynomialChannel::validate`), returning `None` on failure (fail-closed per the spec).

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test -p pleiades-data --lib lsq:: fit_segment_within_span_reproduces`
Expected: PASS.

- [ ] **Step 9: Commit**

```bash
cargo fmt -p pleiades-data && cargo clippy -p pleiades-data --all-targets -- -D warnings
git add crates/pleiades-data/src/coverage/lsq.rs crates/pleiades-data/src/coverage/mod.rs crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/coverage.rs
git commit -m "feat(data): least-squares within-span segment fitter"
```

---

### Task 3: Per-body dense artifact assembly from a reference backend

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs`
- Test: regenerate test module

**Interfaces:**
- Consumes: Task 2 (`fit_segment_within_span`), Task 1 (`fitting_segment_boundaries`); `packaged_bodies()`; `pleiades-jpl` body-window helpers for the 1600–2600 span (reuse the existing window constants used by the current generator — find them in `regenerate.rs`/`pleiades-jpl`); for Eros, the 1900–2100 window.
- Produces: `pub(crate) fn build_packaged_artifact_from_reference(reference: &dyn EphemerisBackend) -> CompressedArtifact` — builds every packaged body's segments by tiling its window with `fitting_segment_boundaries` and calling `fit_segment_within_span`, in deterministic body order.

- [ ] **Step 1: Write the failing test (synthetic backend, kernel-free)**

```rust
#[test]
fn build_from_reference_produces_all_bodies_with_spanning_segments() {
    // Reuse the Synthetic backend from Task 2's test (promote it to a shared
    // test helper in this module if needed).
    let artifact = crate::regenerate::build_packaged_artifact_from_reference(&Synthetic);
    // Every packaged body present.
    for body in crate::packaged_bodies() {
        let ba = artifact.bodies.iter().find(|b| &b.body == body)
            .unwrap_or_else(|| panic!("missing body {body}"));
        assert!(!ba.segments.is_empty(), "{body} has no segments");
        // Segments are contiguous and ascending.
        for pair in ba.segments.windows(2) {
            assert!(pair[1].start.julian_day.days() >= pair[0].end.julian_day.days());
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data --lib build_from_reference_produces_all_bodies`
Expected: FAIL — `build_packaged_artifact_from_reference` not defined.

- [ ] **Step 3: Implement `build_packaged_artifact_from_reference`**

Build per body using the existing `thread::scope` fan-out pattern already in `packaged_body_artifacts_from_snapshot` (reuse its structure). For each body: determine its `[start_jd, end_jd]` window (base bodies → the 1600–2600 packaged window already used by the current generator; Eros → its 1900–2100 constrained window), call `fitting_segment_boundaries`, then `fit_segment_within_span` per span, collect into `BodyArtifact::new(body, segments)`. Assemble via `CompressedArtifact::new(ArtifactHeader::new(ARTIFACT_LABEL, packaged_artifact_source_text()), bodies)`, set `checksum`, and `validate()` (same tail as the existing `try_regenerate_packaged_artifact_from_snapshot`). Show the full function in the implementation, mirroring the existing helpers' structure (do not invent new artifact APIs).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-data --lib build_from_reference_produces_all_bodies`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt -p pleiades-data && cargo clippy -p pleiades-data --all-targets -- -D warnings
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/coverage.rs
git commit -m "feat(data): assemble dense artifact from a reference backend"
```

---

### Task 4: Kernel-gated regeneration from de440 + regenerate the committed `.bin`

**Requires `PLEIADES_DE_KERNEL` and a long-running environment. Slow (potentially hours).**

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs` (kernel-gated entrypoint), `crates/pleiades-validate/src/render/cli.rs` (gate `generate-packaged-artifact` on the kernel)
- Modify: `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`

**Interfaces:**
- Consumes: Task 3 (`build_packaged_artifact_from_reference`); `pleiades_jpl::{SpkBackend}` (implements `EphemerisBackend`); `corpus_spec::KERNEL_SHA256`.
- Produces: a documented kernel-gated regeneration command and the regenerated committed artifact bytes.

- [ ] **Step 1: Wire `SpkBackend` into a gated regeneration entrypoint**

In `regenerate.rs`, add `pub fn regenerate_packaged_artifact_from_kernel(kernel_path: &str) -> Result<CompressedArtifact, String>` that builds `SpkBackend::builder().add_kernel(kernel_path)?.build()` and calls `build_packaged_artifact_from_reference(&backend)`. In `cli.rs`, make `generate-packaged-artifact` read `PLEIADES_DE_KERNEL`; if unset, fail closed with a clear message (mirror `corpus_regen`'s gating text). The in-process kernel-free `regenerate_packaged_artifact()` that previously fit from the committed snapshot is removed or redirected — runtime decode (`packaged_artifact_bytes()`) is the only kernel-free path; any kernel-free test that depended on regeneration is updated to decode the committed bytes instead.

- [ ] **Step 2: Regenerate the committed fixture (kernel present)**

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
  cargo run --release -p pleiades-cli -- validate generate-packaged-artifact \
  --out crates/pleiades-data/tests/fixtures/packaged-artifact.bin
```
Confirm the correct bin crate from `crates/pleiades-cli/Cargo.toml`. Expect a long run; record wall-clock.

- [ ] **Step 3: Verify determinism (kernel present)**

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp \
  cargo run --release -p pleiades-cli -- validate generate-packaged-artifact --check
```
Run TWICE; both must report a match (byte-stable). If non-deterministic, STOP and report BLOCKED.

- [ ] **Step 4: Confirm kernel-free data-crate tests pass against the new bytes**

Run: `cargo test -p pleiades-data`
Expected: green (the slow full-generation test stays `#[ignore]`d; decode/lookup tests pass against the regenerated bytes — update any lookup test that referenced old artifact characteristics, but do NOT widen tolerances to mask a real defect; if a body's lookup is wildly off, report it as a baseline finding for Task 5, not a test to silence).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-validate/src/render/cli.rs crates/pleiades-data/tests/fixtures/packaged-artifact.bin
git commit -m "feat(data): regenerate draft artifact from de440 (kernel-gated)"
```

---

### Task 5: Accuracy baseline — decoded artifact vs committed hold-out

**Files:**
- Create: `crates/pleiades-data/src/accuracy_baseline.rs`
- Modify: `crates/pleiades-data/src/lib.rs` (register + re-export), `crates/pleiades-validate/src/render/cli.rs` (add `packaged-artifact-accuracy-baseline-summary`)
- Test: inline tests + a synthetic-artifact test

**Interfaces:**
- Consumes: `pleiades_jpl::production_holdout_corpus()` (Task 1 of the prior slice — kept); the decoded artifact via `packaged_artifact_bytes()`/`build_packaged_artifact()`; `SnapshotEntry::ecliptic()`.
- Produces:
  - `pub struct BodyChannelError { pub body: CelestialBody, pub max_longitude_arcsec: f64, pub rms_longitude_arcsec: f64, pub max_latitude_arcsec: f64, pub rms_latitude_arcsec: f64, pub max_distance_km: f64, pub rms_distance_km: f64 }`
  - `pub fn packaged_artifact_accuracy_baseline() -> Vec<BodyChannelError>` — for each hold-out row, look up the decoded artifact at that epoch/body, diff against the hold-out's ecliptic, aggregate per body.
  - `pub fn packaged_artifact_accuracy_baseline_summary_for_report() -> String` — validated, drift-checkable summary string.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn baseline_reports_zero_error_for_an_artifact_that_matches_holdout() {
    // Construct a tiny synthetic hold-out + artifact that agree exactly, and
    // assert the per-body error is ~0. (Use a small in-test artifact/lookup or a
    // dependency-injected hold-out slice; do NOT require the real 25k corpus.)
    let errors = crate::accuracy_baseline::accuracy_baseline_against(
        &synthetic_holdout(), &synthetic_artifact());
    assert!(errors.iter().all(|e| e.max_longitude_arcsec < 1e-3));
}
```
(Implement `accuracy_baseline_against(holdout: &[SnapshotEntry], artifact: &CompressedArtifact)` as the testable core, with `packaged_artifact_accuracy_baseline()` wiring the real committed hold-out + decoded artifact on top.)

- [ ] **Step 2: Run test to verify it fails** — `cargo test -p pleiades-data --lib baseline_reports_zero_error`; Expected: FAIL (undefined).

- [ ] **Step 3: Implement the baseline core + summary.** Look up the artifact at each hold-out epoch (reuse the existing artifact lookup path in `lookup.rs`), convert both to ecliptic lon/lat/dist, accumulate max/RMS per body (longitude diff wrapped to ±180° then to arcsec). Render a deterministic summary string (one line per body), with a `validate()` that recomputes and compares (drift gate), following the existing validated-summary pattern in the crate.

- [ ] **Step 4: Run test to verify it passes** — Expected: PASS.

- [ ] **Step 5: Add the CLI command** in `cli.rs` (`packaged-artifact-accuracy-baseline-summary` + alias), printing `packaged_artifact_accuracy_baseline_summary_for_report()`, mirroring an existing summary command handler.

- [ ] **Step 6: Commit**

```bash
cargo fmt -p pleiades-data pleiades-validate && cargo clippy -p pleiades-data -p pleiades-validate --all-targets -- -D warnings
git add crates/pleiades-data/src/accuracy_baseline.rs crates/pleiades-data/src/lib.rs crates/pleiades-validate/src/render/cli.rs
git commit -m "feat(data): per-body accuracy baseline vs committed hold-out"
```

- [ ] **Step 7 (kernel-environment): record the real baseline.** With the Task-4 regenerated `.bin` committed, run `cargo run --release -p pleiades-cli -- validate packaged-artifact-accuracy-baseline-summary`, capture the per-body numbers into the report, and commit the validated summary expectation so the drift gate is anchored. These numbers are the SP1 deliverable that scopes SP2.

---

### Task 6: Gated determinism/reproduce test + size/perf baseline

**Requires `PLEIADES_DE_KERNEL`. Slow.**

**Files:**
- Create/modify: a gated test (e.g. `crates/pleiades-data/tests/artifact_regen.rs`) mirroring `crates/pleiades-jpl/tests/corpus_regen.rs`
- Modify: report/benchmark surfaces for size/decode/lookup baseline metrics

**Interfaces:** Consumes Task 4's gated regeneration; the committed `.bin`.

- [ ] **Step 1: Write the gated reproduce test** — skipped unless `PLEIADES_DE_KERNEL` is set; regenerates from de440 and asserts byte-identity with the committed `packaged_artifact_bytes()`. Mirror `corpus_regen.rs`'s gating boilerplate exactly.
- [ ] **Step 2 (kernel env): run it** — `PLEIADES_DE_KERNEL=… cargo test -p pleiades-data --test artifact_regen`; Expected: PASS (byte-identical).
- [ ] **Step 3: Record size/perf baseline** — capture artifact byte size + decode + single-lookup latency through the existing benchmark/summary surfaces; commit the baseline metrics (drift-gated where the pattern exists). Mark them explicitly as draft baselines.
- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-data/tests/artifact_regen.rs <benchmark/summary files>
git commit -m "test(data): gated de440 artifact reproduce + size/perf baseline"
```

---

### Task 7: Plan/status documentation alignment

**Files:** `PLAN.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`, `plan/stages/02-production-compressed-ephemeris.md`

- [ ] **Step 1: Prune stale Phase-1 entries** (public-data reader + asteroid-kernel adoption are Met) from `plan/status/02`; update `plan/status/01` to state the dense de440-backed generation source + accuracy baseline (SP1) landed and SP2 (accuracy tuning) is next.
- [ ] **Step 2: Update Phase-2 stage + PLAN footer** — record that generation now fits against de440 within-span (SP1), artifact remains draft, and SP2/SP3 (tuning, thresholds) are the remaining Phase-2 work. Remove the superseded "rebase artifact generation on the Phase 1 corpus" wording.
- [ ] **Step 3: Verify consistency** — re-read; confirm the docs consistently state generation is de440-backed/kernel-gated, artifact is draft, asteroids constrained.
- [ ] **Step 4: Commit**

```bash
git add PLAN.md plan/status/01-current-execution-frontier.md plan/status/02-next-slice-candidates.md plan/stages/02-production-compressed-ephemeris.md
git commit -m "docs(plan): record SP1 dense de440-backed generation; prune Met Phase-1 entries"
```

---

## Self-Review

**Spec coverage:**
- Sourcing model B (fit against de440, gated; commit only bytes+checksum) → Tasks 4, 6. ✓
- Fit model (per-body span + within-span dense sampling + degree, decouple density from count) → Tasks 1, 2, 3. ✓
- Per-body span defaults → Task 1. ✓
- Reuse existing channel primitives + longitude unwrap → Task 2. ✓
- Regenerate draft artifact, kernel-gated, deterministic → Tasks 4, 6. ✓
- Per-body accuracy baseline vs committed hold-out, validated summary → Task 5. ✓
- Size/perf baseline metrics → Task 6. ✓
- Determinism + kernel-free vs gated verification → Tasks 4, 6 + Global Constraints. ✓
- Draft-grade, asteroid-constrained, no thresholds/tuning → Global Constraints, reinforced per task. ✓
- Docs/status alignment + prune Phase-1 → Task 7. ✓

**Placeholder scan:** Tasks 1–2 carry complete code. Tasks 3–6 specify exact functions, signatures, commands, and gating, with implementation described concretely against named existing helpers (the per-body assembly reuses the existing `thread::scope` fan-out; the baseline reuses the existing lookup path). No "TBD"/"handle edge cases". The kernel-bound steps are explicitly marked as requiring `PLEIADES_DE_KERNEL` rather than hand-waved. ✓

**Type consistency:** `fit_polynomial_lsq`, `fit_segment_within_span`, `build_packaged_artifact_from_reference`, `regenerate_packaged_artifact_from_kernel`, `fitting_segment_span_days`/`fitting_within_span_sample_count`/`fitting_segment_boundaries`, `packaged_artifact_accuracy_baseline`/`accuracy_baseline_against`/`BodyChannelError` are used consistently across tasks. Tasks flag where the implementer must match real APIs (`EphemerisResult`/`EclipticCoordinates` constructors, `unwrap_longitude_samples` signature, `scale_exponent` values) rather than trust the illustrative code. ✓
