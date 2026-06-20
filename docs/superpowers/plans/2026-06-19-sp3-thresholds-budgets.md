# SP3 — Published Accuracy Thresholds & Size/Latency Budgets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close Phase 2 by publishing and enforcing per-body-class × per-channel accuracy thresholds (longitude, latitude, distance, speed) and size/latency budgets for the packaged ephemeris artifact over the 1900–2100 window.

**Architecture:** Three parts. **Part 1** implements motion/speed output (analytic derivative of the fitted polynomials + Cartesian velocity recombination for heliocentric-stored planets), flipping the artifact's published capability from `Motion = Unsupported` to `Derived`. **Part 2** adds a published-ceiling source of truth (`thresholds.rs`), validates speed against new de440-sourced velocity truth in the hold-out corpus, and adds hard accuracy/size gates plus a tracked latency summary — integrating with (not duplicating) the existing `coverage/target.rs` target-threshold subsystem. **Part 3** reconciles spec/plan/README docs to the 1900–2100 window and the new motion capability.

**Tech Stack:** Rust workspace (`pleiades-*` crates), pure-Rust, no new dependencies. Tests via `cargo test`. de440 kernel-gated regen behind `PLEIADES_DE_KERNEL`.

## Global Constraints

- Pure-Rust, layered crate boundaries from `spec/architecture.md`; no new external deps.
- Coverage window for all thresholds/budgets: **1900–2100** (the default artifact + hold-out corpus). 1600–2600 is documented future expansion, not gated.
- Published speed values use `pleiades_types::Motion`: `longitude_deg_per_day`, `latitude_deg_per_day`, `distance_au_per_day` (°/day, °/day, AU/day).
- Speed error thresholds: arcsec/day (lon/lat), AU/day (radial).
- Hard gates: accuracy ceilings (10 major bodies, all 4 channels) and encoded size. Latency: tracked summary + `#[ignore]`/env-opt-in gate only (`PLEIADES_ENFORCE_LATENCY`) — known concurrent-load flakiness.
- Eros: documented/constrained target + self-consistency check, NOT an independent-truth gate (no independent hold-out exists for it).
- Keep unsupported outputs (apparent, topocentric, native sidereal, civil-time) explicitly rejected.
- Determinism: artifact regen stays byte-identical unless `speed_policy` is serialized into the artifact bytes — if it is, bump `ARTIFACT_VERSION 6→7` and regenerate; if it is declared only in code (coverage profile), no byte change. Verify in Task 7.
- Do NOT run `cargo` concurrently with subagents during accuracy/corpus regen (timing-sensitive benchmark tests fail under concurrent load; per the SP2 SDD log). Kernel available at `/workspace/.cache/kernels/de440.bsp`.
- Stored channel basis: monomial, ascending power order; segment normalized time `x = (t − start)/span_days`, so `d(value)/dt = evaluate_derivative(x) / span_days`.

---

# PART 1 — Motion/Speed capability

### Task 1: Polynomial & segment analytic derivative

**Files:**
- Modify: `crates/pleiades-compression/src/channels.rs` (add methods to `PolynomialChannel` ~after line 119, and to `Segment` ~after line 234)
- Test: same file, `#[cfg(test)]` module (channels.rs already has tests; add there)

**Interfaces:**
- Consumes: existing `PolynomialChannel { coefficients: Vec<f64> }` (monomial ascending), `Segment::span_days()`, `Segment::channel()/residual_channel()`, `ChannelKind`.
- Produces:
  - `PolynomialChannel::evaluate_derivative(&self, x: f64) -> f64` — value of dP/dx at normalized x.
  - `Segment::evaluate_channel_derivative(&self, kind: ChannelKind, x: f64) -> Result<f64, CompressionError>` — (base+residual) dP/dx; same missing-channel error as `evaluate_channel`.

- [ ] **Step 1: Write the failing test**

```rust
// in channels.rs tests module
#[test]
fn polynomial_derivative_matches_power_rule() {
    // P(x) = 2 + 3x + 4x^2  ->  P'(x) = 3 + 8x
    let ch = PolynomialChannel::new(ChannelKind::Longitude, 9, vec![2.0, 3.0, 4.0]);
    assert!((ch.evaluate_derivative(0.0) - 3.0).abs() < 1e-12);
    assert!((ch.evaluate_derivative(1.0) - 11.0).abs() < 1e-12);
    assert!((ch.evaluate_derivative(0.5) - 7.0).abs() < 1e-12);
}

#[test]
fn segment_channel_derivative_includes_residual() {
    let start = Instant::new(JulianDay::from_days(0.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(1.0), TimeScale::Tt);
    let seg = Segment::new(
        start,
        end,
        vec![PolynomialChannel::new(ChannelKind::Longitude, 9, vec![0.0, 2.0])], // base' = 2
    )
    .with_residual_channels(vec![PolynomialChannel::new(
        ChannelKind::Longitude,
        9,
        vec![0.0, 0.0, 5.0], // residual' = 10x
    )]);
    // total derivative at x=1: 2 + 10 = 12
    let d = seg.evaluate_channel_derivative(ChannelKind::Longitude, 1.0).unwrap();
    assert!((d - 12.0).abs() < 1e-12);
}
```

(If `Segment` has no `with_residual_channels` builder, construct the `Segment` struct literally with `residual_channels` populated — check the existing constructor at channels.rs:~140. Use whichever the file already exposes; do not invent a builder.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression polynomial_derivative_matches_power_rule segment_channel_derivative_includes_residual`
Expected: FAIL — `no method named evaluate_derivative` / `evaluate_channel_derivative`.

- [ ] **Step 3: Write minimal implementation**

```rust
impl PolynomialChannel {
    /// Derivative dP/dx of the monomial polynomial at normalized time `x`.
    /// Coefficients are ascending power order, so d/dx(Σ c_i x^i) = Σ i·c_i·x^(i-1).
    pub(crate) fn evaluate_derivative(&self, x: f64) -> f64 {
        let mut result = 0.0;
        let mut power = 1.0; // x^(i-1), starting at i=1
        for (i, coefficient) in self.coefficients.iter().enumerate().skip(1) {
            result += (i as f64) * coefficient * power;
            power *= x;
        }
        result
    }
}

impl Segment {
    /// (base + residual) derivative dP/dx at normalized time `x` for `kind`.
    pub(crate) fn evaluate_channel_derivative(
        &self,
        kind: ChannelKind,
        x: f64,
    ) -> Result<f64, CompressionError> {
        let base = self
            .channel(kind)
            .map(|channel| channel.evaluate_derivative(x))
            .ok_or_else(|| {
                CompressionError::new(
                    CompressionErrorKind::MissingChannel,
                    format!("missing {kind:?} channel"),
                )
            })?;
        let residual = self
            .residual_channel(kind)
            .map(|channel| channel.evaluate_derivative(x))
            .unwrap_or(0.0);
        Ok(base + residual)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-compression polynomial_derivative segment_channel_derivative`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/channels.rs
git commit -m "feat(compression): analytic derivative for polynomial channels and segments"
```

---

### Task 2: Spherical↔Cartesian velocity recombination (frame math)

**Files:**
- Modify: `crates/pleiades-compression/src/frame_recombine.rs` (the SP2 module with position recombination helpers)
- Test: same file's test module

**Interfaces:**
- Consumes: existing SP2 spherical↔Cartesian *position* helpers in this module (e.g. `ecliptic_to_cartesian_au`). Read the file first to reuse its vector type / conventions.
- Produces (all in AU and radians-per-day internally; callers convert units):
  - `pub(crate) struct SphericalState { pub lon_rad: f64, pub lat_rad: f64, pub dist_au: f64, pub lon_rate_rad_per_day: f64, pub lat_rate_rad_per_day: f64, pub dist_rate_au_per_day: f64 }`
  - `pub(crate) struct CartesianState { pub pos_au: [f64; 3], pub vel_au_per_day: [f64; 3] }`
  - `pub(crate) fn spherical_state_to_cartesian(s: SphericalState) -> CartesianState`
  - `pub(crate) fn cartesian_state_to_spherical(c: CartesianState) -> SphericalState`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn velocity_round_trips_through_cartesian() {
    let s = SphericalState {
        lon_rad: 0.7,
        lat_rad: 0.2,
        dist_au: 1.5,
        lon_rate_rad_per_day: 0.01,
        lat_rate_rad_per_day: -0.003,
        dist_rate_au_per_day: 0.002,
    };
    let c = spherical_state_to_cartesian(s);
    let back = cartesian_state_to_spherical(c);
    assert!((back.lon_rad - s.lon_rad).abs() < 1e-10);
    assert!((back.lat_rad - s.lat_rad).abs() < 1e-10);
    assert!((back.dist_au - s.dist_au).abs() < 1e-10);
    assert!((back.lon_rate_rad_per_day - s.lon_rate_rad_per_day).abs() < 1e-10);
    assert!((back.lat_rate_rad_per_day - s.lat_rate_rad_per_day).abs() < 1e-10);
    assert!((back.dist_rate_au_per_day - s.dist_rate_au_per_day).abs() < 1e-10);
}

#[test]
fn geocentric_velocity_is_helio_plus_sun() {
    // Cartesian velocity composes additively: V_geo = V_helio + V_sun.
    let helio = CartesianState { pos_au: [1.0, 0.0, 0.0], vel_au_per_day: [0.0, 0.017, 0.0] };
    let sun = CartesianState { pos_au: [0.0, 1.0, 0.0], vel_au_per_day: [-0.017, 0.0, 0.0] };
    let geo_vel = [
        helio.vel_au_per_day[0] + sun.vel_au_per_day[0],
        helio.vel_au_per_day[1] + sun.vel_au_per_day[1],
        helio.vel_au_per_day[2] + sun.vel_au_per_day[2],
    ];
    assert_eq!(geo_vel, [-0.017, 0.017, 0.0]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression velocity_round_trips geocentric_velocity_is_helio_plus_sun`
Expected: FAIL — types/functions not found.

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Clone, Copy, Debug)]
pub(crate) struct SphericalState {
    pub lon_rad: f64,
    pub lat_rad: f64,
    pub dist_au: f64,
    pub lon_rate_rad_per_day: f64,
    pub lat_rate_rad_per_day: f64,
    pub dist_rate_au_per_day: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CartesianState {
    pub pos_au: [f64; 3],
    pub vel_au_per_day: [f64; 3],
}

pub(crate) fn spherical_state_to_cartesian(s: SphericalState) -> CartesianState {
    let (sl, cl) = s.lon_rad.sin_cos();
    let (sb, cb) = s.lat_rad.sin_cos();
    let r = s.dist_au;
    let pos = [r * cb * cl, r * cb * sl, r * sb];
    // Chain rule with dλ, dβ, dr.
    let dr = s.dist_rate_au_per_day;
    let dl = s.lon_rate_rad_per_day;
    let db = s.lat_rate_rad_per_day;
    let vel = [
        dr * cb * cl - r * sb * cl * db - r * cb * sl * dl,
        dr * cb * sl - r * sb * sl * db + r * cb * cl * dl,
        dr * sb + r * cb * db,
    ];
    CartesianState { pos_au: pos, vel_au_per_day: vel }
}

pub(crate) fn cartesian_state_to_spherical(c: CartesianState) -> SphericalState {
    let [x, y, z] = c.pos_au;
    let [vx, vy, vz] = c.vel_au_per_day;
    let rho2 = x * x + y * y;
    let rho = rho2.sqrt();
    let r = (rho2 + z * z).sqrt();
    let dr = if r == 0.0 { 0.0 } else { (x * vx + y * vy + z * vz) / r };
    let dl = if rho2 == 0.0 { 0.0 } else { (x * vy - y * vx) / rho2 };
    // β = atan2(z, ρ); dβ/dt = (ρ·ż − z·ρ̇)/r²,  ρ̇ = (x·vx + y·vy)/ρ
    let drho = if rho == 0.0 { 0.0 } else { (x * vx + y * vy) / rho };
    let db = if r == 0.0 { 0.0 } else { (rho * vz - z * drho) / (r * r) };
    SphericalState {
        lon_rad: y.atan2(x),
        lat_rad: z.atan2(rho),
        dist_au: r,
        lon_rate_rad_per_day: dl,
        lat_rate_rad_per_day: db,
        dist_rate_au_per_day: dr,
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-compression velocity_round_trips geocentric_velocity_is_helio_plus_sun`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/frame_recombine.rs
git commit -m "feat(compression): spherical<->cartesian velocity recombination for motion"
```

---

### Task 3: `CompressedArtifact::lookup_motion`

**Files:**
- Modify: `crates/pleiades-compression/src/artifact.rs` (add `lookup_motion` near `lookup_ecliptic` at line 258; reuse the segment-selection + x-normalization at lines 274–279 and the heliocentric reframe path)
- Test: same file's test module

**Interfaces:**
- Consumes: Task 1 (`Segment::evaluate_channel_derivative`), Task 2 (`SphericalState`/`CartesianState` recombination), existing `lookup_ecliptic`, `StoredFrame` per-body frame (SP2), the Sun-presence invariant, `pleiades_types::Motion`.
- Produces: `pub fn lookup_motion(&self, body: &CelestialBody, instant: Instant) -> Result<Motion, CompressionError>` returning `Motion { longitude_deg_per_day: Some(_), latitude_deg_per_day: Some(_), distance_au_per_day: Some(_) }`.

**Implementation notes (read before coding):**
- For a **geocentric-stored** body (Sun, Moon, Eros): the stored channels already *are* geocentric ecliptic, so motion = direct per-day derivative of the stored lon/lat/dist channels: `dλ/dt = evaluate_channel_derivative(Longitude, x)/span_days` (deg/day), same for lat; `dr/dt` (AU/day).
- For a **heliocentric-stored** planet: compute the body's *heliocentric* spherical position+rates → `spherical_state_to_cartesian` → helio Cartesian velocity; compute the Sun's *geocentric* Cartesian position+velocity the same way; `V_geo = V_helio + V_sun`, `P_geo = P_helio + S_geo`; then `cartesian_state_to_spherical` → geocentric spherical rates → convert to `Motion`. This mirrors SP2's `P_geo = P_helio + S_geo` position reframe, extended to velocity. Convert deg↔rad at the boundary (stored channels are degrees; recombination is radians).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn lookup_motion_for_geocentric_body_is_direct_derivative() {
    // A Sun segment with longitude linear in time: lon = 100 + 2*x over a 10-day span
    // => dλ/dt = 2 deg / 10 days = 0.2 deg/day. lat const, dist const.
    let start = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let end = Instant::new(JulianDay::from_days(2_451_555.0), TimeScale::Tt);
    let segment = Segment::new(
        start,
        end,
        vec![
            PolynomialChannel::linear(ChannelKind::Longitude, 9, 100.0, 102.0),
            PolynomialChannel::linear(ChannelKind::Latitude, 9, 5.0, 5.0),
            PolynomialChannel::linear(ChannelKind::DistanceAu, 10, 1.0, 1.0),
        ],
    );
    let artifact = CompressedArtifact::new(
        ArtifactHeader::new("motion-test", "motion test"),
        vec![BodyArtifact::new(CelestialBody::Sun, vec![segment])],
    );
    let at = Instant::new(JulianDay::from_days(2_451_550.0), TimeScale::Tt);
    let m = artifact.lookup_motion(&CelestialBody::Sun, at).unwrap();
    assert!((m.longitude_deg_per_day.unwrap() - 0.2).abs() < 1e-9);
    assert!(m.latitude_deg_per_day.unwrap().abs() < 1e-9);
    assert!(m.distance_au_per_day.unwrap().abs() < 1e-9);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression lookup_motion_for_geocentric_body_is_direct_derivative`
Expected: FAIL — `no method named lookup_motion`.

- [ ] **Step 3: Write minimal implementation**

Add to `impl CompressedArtifact`. Reuse the exact segment-lookup + `x` computation pattern from `lookup_ecliptic` (artifact.rs:258–311). Sketch (adapt names to the real ones in the file):

```rust
pub fn lookup_motion(
    &self,
    body: &CelestialBody,
    instant: Instant,
) -> Result<Motion, CompressionError> {
    self.require_output_support(ArtifactOutput::Motion)?;
    // ... same TimeScale guard as lookup_ecliptic (TT/TDB only) ...
    let (segment, x, span_days) = self.locate_segment(body, instant)?; // mirror lookup_ecliptic's selection
    let dlon_dt = segment.evaluate_channel_derivative(ChannelKind::Longitude, x)? / span_days;
    let dlat_dt = segment.evaluate_channel_derivative(ChannelKind::Latitude, x)? / span_days;
    let ddist_dt = segment.evaluate_channel_derivative(ChannelKind::DistanceAu, x)? / span_days;

    match self.body_frame(body) {
        StoredFrame::Geocentric => Ok(Motion {
            longitude_deg_per_day: Some(dlon_dt),
            latitude_deg_per_day: Some(dlat_dt),
            distance_au_per_day: Some(ddist_dt),
        }),
        StoredFrame::Heliocentric => {
            let lon = segment.evaluate_channel(ChannelKind::Longitude, x)?;
            let lat = segment.evaluate_channel(ChannelKind::Latitude, x)?;
            let dist = segment.evaluate_channel(ChannelKind::DistanceAu, x)?;
            let helio = spherical_state_to_cartesian(SphericalState {
                lon_rad: lon.to_radians(),
                lat_rad: lat.to_radians(),
                dist_au: dist,
                lon_rate_rad_per_day: dlon_dt.to_radians(),
                lat_rate_rad_per_day: dlat_dt.to_radians(),
                dist_rate_au_per_day: ddist_dt,
            });
            let sun = self.sun_cartesian_state(instant)?; // Sun's geocentric pos+vel, AU & AU/day
            let geo = CartesianState {
                pos_au: [
                    helio.pos_au[0] + sun.pos_au[0],
                    helio.pos_au[1] + sun.pos_au[1],
                    helio.pos_au[2] + sun.pos_au[2],
                ],
                vel_au_per_day: [
                    helio.vel_au_per_day[0] + sun.vel_au_per_day[0],
                    helio.vel_au_per_day[1] + sun.vel_au_per_day[1],
                    helio.vel_au_per_day[2] + sun.vel_au_per_day[2],
                ],
            };
            let s = cartesian_state_to_spherical(geo);
            Ok(Motion {
                longitude_deg_per_day: Some(s.lon_rate_rad_per_day.to_degrees()),
                latitude_deg_per_day: Some(s.lat_rate_rad_per_day.to_degrees()),
                distance_au_per_day: Some(s.dist_rate_au_per_day),
            })
        }
    }
}
```

Add a private helper `sun_cartesian_state(&self, instant) -> Result<CartesianState, CompressionError>` that derivative-evaluates the Sun's geocentric segment (Sun is always geocentric per the SP2 Sun-presence invariant) into `CartesianState`. Factor the segment-locate code from `lookup_ecliptic` into a shared private `locate_segment` if not already present; otherwise inline the same logic. **Match the real method/field names in artifact.rs — do not invent `body_frame`/`locate_segment` if the file names them differently; read lines 258–341 first.**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-compression lookup_motion`
Expected: PASS.

- [ ] **Step 5: Add a heliocentric-body motion smoke test, run, commit**

Add a test building a 2-body artifact (Sun geocentric + a planet heliocentric) and assert `lookup_motion(planet)` returns finite, non-`None` components. Then:

```bash
cargo test -p pleiades-compression lookup_motion
git add crates/pleiades-compression/src/artifact.rs
git commit -m "feat(compression): CompressedArtifact::lookup_motion (geocentric direct + heliocentric recombination)"
```

---

### Task 4: Flip published capability to motion-derived

**Files:**
- Modify: `crates/pleiades-compression/src/format.rs:367-386` (profile builder `packaged_ecliptic_longitude_latitude_distance_with_derived_equatorial`)
- Modify: `crates/pleiades-data/src/coverage/profile.rs:869` (expected_states) and `:1006` (`PACKAGED_ARTIFACT_SPEED_POLICY_SUMMARY`)
- Modify: `crates/pleiades-data/src/lookup.rs:506-511` (EXPECTED_UNSUPPORTED_OUTPUTS / EXPECTED_DERIVED_OUTPUTS arrays) and `:542-543` (storage summary text)
- Test (update existing): `crates/pleiades-data/src/tests/coverage.rs:330`, `crates/pleiades-data/src/tests/lookup.rs:971` and `:1098`

**Interfaces:**
- Consumes: `SpeedPolicy::FittedDerivative` (maps to `ArtifactOutputSupport::Derived` via `format.rs:162`), `ArtifactOutput::Motion`.
- Produces: a packaged profile where Motion is in `derived_outputs`, `speed_policy == FittedDerivative`.

- [ ] **Step 1: Update the profile builder (format.rs:367-386)** — move `ArtifactOutput::Motion` from the unsupported vec into the derived vec; change `SpeedPolicy::Unsupported` → `SpeedPolicy::FittedDerivative`.

```rust
vec![
    ArtifactOutput::EclipticCoordinates,
    ArtifactOutput::EquatorialCoordinates,
    ArtifactOutput::Motion,
],
vec![
    ArtifactOutput::ApparentCorrections,
    ArtifactOutput::TopocentricCoordinates,
    ArtifactOutput::SiderealCoordinates,
],
SpeedPolicy::FittedDerivative,
```

- [ ] **Step 2: Update the three validation sites**

- `coverage/profile.rs:869`: `(ArtifactOutput::Motion, ArtifactOutputSupport::Derived),`
- `coverage/profile.rs:1006`: `policy: SpeedPolicy::FittedDerivative,`
- `lookup.rs`: in `EXPECTED_DERIVED_OUTPUTS` add `ArtifactOutput::Motion` (array size 2→3); in `EXPECTED_UNSUPPORTED_OUTPUTS` remove it (size 4→3).
- `lookup.rs:543`: change the summary string tail from `"...sidereal, and motion outputs remain unsupported"` to `"...apparent, topocentric, and sidereal outputs remain unsupported; motion/speed is derived from fitted segment derivatives"`.

- [ ] **Step 3: Update existing tests that assert Motion-unsupported**

- `tests/coverage.rs:330` block: the drift test removes Motion from `unsupported_outputs` and expects a `ProfileOutOfSync` error. Re-point it to instead remove Motion from `derived_outputs` (so it still exercises a drift). 
- `tests/lookup.rs:971` `backend_metadata_exposes_packaged_scope`: change the assertion from `unsupported_outputs.contains(Motion)` to `derived_outputs.contains(&ArtifactOutput::Motion)`.
- `tests/lookup.rs:1098`: update the expected substring to match the new summary text from Step 2.

- [ ] **Step 4: Run the coverage/lookup tests**

Run: `cargo test -p pleiades-data coverage:: lookup::`
Expected: PASS (updated assertions green; no remaining "motion unsupported" assertions).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/format.rs crates/pleiades-data/src/coverage/profile.rs crates/pleiades-data/src/lookup.rs crates/pleiades-data/src/tests/coverage.rs crates/pleiades-data/src/tests/lookup.rs
git commit -m "feat(data): declare packaged Motion=Derived, speed policy FittedDerivative"
```

---

### Task 5: Backend returns motion in `EphemerisResult`

**Files:**
- Modify: `crates/pleiades-data/src/backend.rs:127-167` (`position`)
- Test: `crates/pleiades-data/src/tests/lookup.rs`

**Interfaces:**
- Consumes: `CompressedArtifact::lookup_motion` (Task 3), `EphemerisResult.motion` field (Option<Motion>), `EphemerisRequest`.
- Produces: `position()` populating `result.motion = Some(self.artifact.lookup_motion(&req.body, instant)?)`.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn packaged_backend_returns_motion_for_a_major_body() {
    let backend = packaged_backend();
    let inst = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let res = backend
        .position(&EphemerisRequest::new(CelestialBody::Mars, inst))
        .expect("mars position");
    let motion = res.motion.expect("motion should be populated");
    assert!(motion.longitude_deg_per_day.unwrap().is_finite());
    // Mars mean motion is well under 1 deg/day in magnitude.
    assert!(motion.longitude_deg_per_day.unwrap().abs() < 1.0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data packaged_backend_returns_motion_for_a_major_body`
Expected: FAIL — `res.motion` is `None`.

- [ ] **Step 3: Implement** — in `position()`, after computing ecliptic/equatorial, add:

```rust
let motion = self
    .artifact
    .lookup_motion(&req.body, instant)
    .map_err(/* same CompressionError -> EphemerisError mapping used for lookup_ecliptic */)?;
// ... in the EphemerisResult construction:
motion: Some(motion),
```

Use the identical error-mapping closure the function already uses for `lookup_ecliptic`. Read backend.rs:148–164 and reuse it.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-data packaged_backend_returns_motion_for_a_major_body`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/backend.rs crates/pleiades-data/src/tests/lookup.rs
git commit -m "feat(data): packaged backend returns derived motion in EphemerisResult"
```

---

# PART 2 — Velocity truth, thresholds, budgets, gates

### Task 6: Add optional velocity columns to the corpus row

**Files:**
- Modify: `crates/pleiades-jpl/src/backend.rs:1214-1225` (`SnapshotEntry`) and `:2215-2248` (`parse_snapshot_line`)
- Test: `crates/pleiades-jpl/src/backend.rs` test module (parser tests already exist there)

**Interfaces:**
- Consumes: existing CSV row `epoch_jd,body,x_km,y_km,z_km`.
- Produces: `SnapshotEntry` with `pub vx_km_s: Option<f64>, pub vy_km_s: Option<f64>, pub vz_km_s: Option<f64>`; parser accepts BOTH 5-field rows (velocity `None`) and 8-field rows (velocity `Some`).

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn parses_eight_field_row_with_velocity() {
    let csv = "#Columns:epoch_jd,body,x_km,y_km,z_km,vx_km_s,vy_km_s,vz_km_s\n\
               2451545.0,Mars,1.0,2.0,3.0,0.1,0.2,0.3\n";
    let rows = parse_snapshot_entries(csv).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].vx_km_s, Some(0.1));
    assert_eq!(rows[0].vz_km_s, Some(0.3));
}

#[test]
fn parses_five_field_row_without_velocity() {
    let csv = "#Columns:epoch_jd,body,x_km,y_km,z_km\n2451545.0,Mars,1.0,2.0,3.0\n";
    let rows = parse_snapshot_entries(csv).unwrap();
    assert_eq!(rows[0].vx_km_s, None);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-jpl parses_eight_field_row_with_velocity parses_five_field_row_without_velocity`
Expected: FAIL — field doesn't exist / parser rejects 8 fields.

- [ ] **Step 3: Implement** — add the three `Option<f64>` fields to `SnapshotEntry`; in `parse_snapshot_line`, after splitting on `,`, accept length 5 (velocity `None`) or 8 (parse the three velocity floats `Some`); any other length is an error. Update all `SnapshotEntry { .. }` literals in the crate to set the three new fields to `None` (search for struct literals; the synthetic test builders in `accuracy_baseline.rs` also construct `SnapshotEntry` — add `vx_km_s: None, vy_km_s: None, vz_km_s: None`).

- [ ] **Step 4: Run to verify pass + build whole workspace**

Run: `cargo build --workspace && cargo test -p pleiades-jpl parses_`
Expected: PASS; workspace compiles (all struct literals updated).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-jpl/src crates/pleiades-data/src/accuracy_baseline.rs
git commit -m "feat(jpl): optional velocity columns on SnapshotEntry; parser accepts 5- or 8-field rows"
```

---

### Task 7: Regenerate the hold-out slice with de440 velocity truth (kernel-gated)

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/generate.rs:57-79` (`push_corpus_row`) and the holdout-slice header emission
- Add: a helper in `crates/pleiades-jpl/src/spk` that returns **ecliptic-frame Cartesian velocity** (km/s) for a body at an epoch, from the SPK `StateVector` (`segment::evaluate` → equatorial pos+vel) rotated into the ecliptic frame by the same obliquity rotation the position path uses.
- Modify: `crates/pleiades-jpl/data/corpus/holdout.csv` (regenerated) and `crates/pleiades-jpl/data/corpus/manifest.txt` (holdout checksum + header note)
- Test: `crates/pleiades-jpl/tests` kernel-gated regen check (mirror the existing corpus-regen gating)

**Interfaces:**
- Consumes: SPK `StateVector { position_km, velocity_km_s }` (segment::evaluate), existing obliquity rotation used by the position path.
- Produces: `holdout.csv` with 8-column rows (velocity populated) for the 10 major bodies; other slices unchanged (still 5-column).

**Note:** Only the hold-out slice gains velocity, so ONLY `holdout.csv` content and ONLY its manifest checksum change. The `validate_drift` gate (FNV-1a) will require the new checksum.

- [ ] **Step 1: Write the (kernel-gated) regen test**

```rust
// tests/holdout_velocity_regen.rs
#[test]
fn holdout_slice_regenerates_with_velocity_when_kernel_present() {
    let Some(kernel) = std::env::var_os("PLEIADES_DE_KERNEL") else {
        eprintln!("skipping: PLEIADES_DE_KERNEL not set");
        return;
    };
    let regenerated = pleiades_jpl::regenerate_holdout_slice_csv(&kernel).unwrap();
    // committed file must be byte-identical to a fresh regen
    let committed = include_str!("../data/corpus/holdout.csv");
    assert_eq!(regenerated, committed, "committed holdout.csv drifted from de440 regen");
    // velocity columns present
    assert!(regenerated.contains("vx_km_s"));
}
```

(Expose whatever the real regen entry point is — `generate_slice_with_bodies(Holdout, ...)`. If the existing entry point returns a `GeneratedSlice`, assert on its `.csv`. Name the wrapper to match existing conventions; do not invent a new public API if one exists.)

- [ ] **Step 2: Run to verify it fails**

Run: `PLEIADES_DE_KERNEL=/workspace/.cache/kernels/de440.bsp cargo test -p pleiades-jpl holdout_slice_regenerates_with_velocity`
Expected: FAIL — regen still emits 5-column rows ≠ committed (which will be 8-col after Step 4).

- [ ] **Step 3: Implement velocity emission** — in `push_corpus_row`, for the hold-out slice, compute ecliptic Cartesian velocity (km/s) via the new SPK-state helper and append `,{vx:.6},{vy:.6},{vz:.6}`; emit the 8-column header. Keep all other slices on the 5-column path.

- [ ] **Step 4: Regenerate the committed hold-out + checksum**

```bash
PLEIADES_DE_KERNEL=/workspace/.cache/kernels/de440.bsp \
  cargo run -p pleiades-cli -- regenerate-corpus --slice holdout   # or the existing regen subcommand
# update manifest.txt holdout checksum to the new FNV-1a (printed by validate-corpus failure or a maintainer helper)
```

If no single-slice regen subcommand exists, use the existing full corpus-regen path and keep only the holdout.csv diff (other slices must remain byte-identical because their generation is unchanged). Recompute the holdout checksum with `corpus_checksum64` (a maintainer `#[ignore]` helper that prints it is acceptable — mirror the accuracy-baseline golden helper).

- [ ] **Step 5: Confirm ARTIFACT_VERSION decision** — grep whether `speed_policy` is written by the binary codec (`crates/pleiades-compression/src/codec.rs`). If it is serialized, bump `ARTIFACT_VERSION 6→7` and regenerate the artifact `.bin` (kernel-gated, byte-identity verified by `artifact_regen.rs`); if not, leave bytes unchanged. Document the finding in the commit message.

Run: `PLEIADES_DE_KERNEL=/workspace/.cache/kernels/de440.bsp cargo test -p pleiades-jpl holdout_slice_regenerates_with_velocity`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/pleiades-jpl/src/spk crates/pleiades-jpl/data/corpus/holdout.csv crates/pleiades-jpl/data/corpus/manifest.txt crates/pleiades-jpl/tests/holdout_velocity_regen.rs
git commit -m "feat(jpl): hold-out slice carries de440 ecliptic velocity truth (kernel-gated regen)"
```

---

### Task 8: `validate-corpus` accepts the velocity column

**Files:**
- Modify: `crates/pleiades-validate/src/corpus/production.rs:54-88` (`validate_schema_and_provenance`)
- Test: `crates/pleiades-validate` corpus tests

**Interfaces:**
- Consumes: corpus CSV rows (5- or 8-field).
- Produces: schema gate that accepts a 5-field header/rows OR an 8-field header/rows, requiring the trailing 3 velocity fields to be finite numbers when present; still fail-closed on malformed/non-finite/wrong-arity.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn corpus_gate_accepts_eight_field_velocity_rows() {
    // The committed corpus now has an 8-field hold-out slice; the full gate must pass.
    let summary = pleiades_validate::run_corpus_gate().expect("corpus gate should pass");
    assert!(summary.contains("corpus gate ok"));
}

#[test]
fn corpus_gate_rejects_nonfinite_velocity() {
    // a hand-built 8-field row with NaN velocity must be rejected by the schema gate
    // (call the row/schema validator directly with a crafted slice)
    // ... assert Err ...
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-validate corpus_gate_accepts_eight_field_velocity_rows`
Expected: FAIL — current gate hard-codes the 5-field `#Columns` header and 5-field arity.

- [ ] **Step 3: Implement** — accept either the 5-field or 8-field `#Columns:` header; for data rows accept arity 5 or 8; when arity 8, parse+finite-check the last 3 fields. Reject all other arities and any non-finite value (preserve existing fail-closed behavior).

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p pleiades-validate corpus_gate_`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/src/corpus/production.rs
git commit -m "feat(validate): corpus gate accepts optional 8-field velocity rows, fail-closed on non-finite"
```

---

### Task 9: `thresholds.rs` — published ceilings + budgets SSOT

**Files:**
- Create: `crates/pleiades-data/src/thresholds.rs`
- Modify: `crates/pleiades-data/src/lib.rs` (add `pub mod thresholds;` / re-export)
- Test: in `thresholds.rs`

**Interfaces:**
- Produces:
  - `pub enum BodyClass { Luminary, InnerPlanet, OuterPlanet, Asteroid }`
  - `pub fn body_class(body: &CelestialBody) -> BodyClass`
  - `pub struct AccuracyCeiling { pub lon_arcsec: f64, pub lat_arcsec: f64, pub dist_km: f64, pub lon_speed_arcsec_per_day: f64, pub lat_speed_arcsec_per_day: f64, pub radial_speed_au_per_day: f64 }`
  - `pub fn accuracy_ceiling(body: &CelestialBody) -> AccuracyCeiling`
  - `pub struct ArtifactBudgets { pub max_encoded_bytes: usize, pub decode_latency_target_ms: f64, pub single_lookup_target_ms: f64, pub batch_throughput_target_per_s: f64, pub chart_workload_target_ms: f64 }`
  - `pub const PACKAGED_BUDGETS: ArtifactBudgets`

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_backend::CelestialBody;

    #[test]
    fn classes_map_bodies_correctly() {
        assert_eq!(body_class(&CelestialBody::Sun), BodyClass::Luminary);
        assert_eq!(body_class(&CelestialBody::Moon), BodyClass::Luminary);
        assert_eq!(body_class(&CelestialBody::Mercury), BodyClass::InnerPlanet);
        assert_eq!(body_class(&CelestialBody::Pluto), BodyClass::OuterPlanet);
    }

    #[test]
    fn outer_planets_have_looser_longitude_ceiling_than_inner() {
        assert!(
            accuracy_ceiling(&CelestialBody::Uranus).lon_arcsec
                > accuracy_ceiling(&CelestialBody::Mercury).lon_arcsec
        );
    }

    #[test]
    fn size_budget_exceeds_current_artifact() {
        // current ~10 MB; budget has headroom but is finite.
        assert!(PACKAGED_BUDGETS.max_encoded_bytes >= 10_000_000);
        assert!(PACKAGED_BUDGETS.max_encoded_bytes <= 16_000_000);
    }
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-data thresholds::`
Expected: FAIL — module/types don't exist.

- [ ] **Step 3: Implement** — published ceilings (position channels carry ≥10× headroom over the measured sub-arcsec maxima; speed/distance ceilings finalized in Task 11 after first measurement, set generously here and tightened then):

```rust
//! Published per-body-class accuracy ceilings and size/latency budgets (the
//! public contract). The hold-out gate (accuracy_baseline.rs) asserts measured
//! <= ceiling; the tight golden drift test stays as the regression catcher.

use pleiades_backend::CelestialBody;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyClass { Luminary, InnerPlanet, OuterPlanet, Asteroid }

pub fn body_class(body: &CelestialBody) -> BodyClass {
    match body {
        CelestialBody::Sun | CelestialBody::Moon => BodyClass::Luminary,
        CelestialBody::Mercury | CelestialBody::Venus | CelestialBody::Mars => BodyClass::InnerPlanet,
        CelestialBody::Jupiter | CelestialBody::Saturn | CelestialBody::Uranus
        | CelestialBody::Neptune | CelestialBody::Pluto => BodyClass::OuterPlanet,
        _ => BodyClass::Asteroid,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AccuracyCeiling {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
    pub dist_km: f64,
    pub lon_speed_arcsec_per_day: f64,
    pub lat_speed_arcsec_per_day: f64,
    pub radial_speed_au_per_day: f64,
}

pub fn accuracy_ceiling(body: &CelestialBody) -> AccuracyCeiling {
    match body_class(body) {
        BodyClass::Luminary | BodyClass::InnerPlanet => AccuracyCeiling {
            lon_arcsec: 1.0, lat_arcsec: 1.0, dist_km: 50_000.0,
            lon_speed_arcsec_per_day: 60.0, lat_speed_arcsec_per_day: 60.0,
            radial_speed_au_per_day: 1.0e-3,
        },
        BodyClass::OuterPlanet => AccuracyCeiling {
            lon_arcsec: 5.0, lat_arcsec: 5.0, dist_km: 5_000_000.0,
            lon_speed_arcsec_per_day: 60.0, lat_speed_arcsec_per_day: 60.0,
            radial_speed_au_per_day: 1.0e-3,
        },
        BodyClass::Asteroid => AccuracyCeiling {
            lon_arcsec: 30.0, lat_arcsec: 30.0, dist_km: 5_000_000.0,
            lon_speed_arcsec_per_day: 120.0, lat_speed_arcsec_per_day: 120.0,
            radial_speed_au_per_day: 1.0e-2,
        },
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArtifactBudgets {
    pub max_encoded_bytes: usize,
    pub decode_latency_target_ms: f64,
    pub single_lookup_target_ms: f64,
    pub batch_throughput_target_per_s: f64,
    pub chart_workload_target_ms: f64,
}

pub const PACKAGED_BUDGETS: ArtifactBudgets = ArtifactBudgets {
    max_encoded_bytes: 12_000_000,   // ~10.0 MB measured + headroom
    decode_latency_target_ms: 400.0, // ~260 ms measured
    single_lookup_target_ms: 6.0,    // ~3.3 ms measured
    batch_throughput_target_per_s: 1_000.0,
    chart_workload_target_ms: 50.0,
};
```

Then **move** the inline ceilings out of `accuracy_baseline.rs::outer_planet_longitude_meets_astrology_grade_envelope` to call `accuracy_ceiling(body).lon_arcsec` (delete the magic-number closure in that test; it now reads from this module). Adjust the speed/distance ceiling constants in Task 11 once measured.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p pleiades-data thresholds:: outer_planet_longitude_meets_astrology_grade_envelope`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/thresholds.rs crates/pleiades-data/src/lib.rs crates/pleiades-data/src/accuracy_baseline.rs
git commit -m "feat(data): thresholds.rs published accuracy ceilings + size/latency budgets (SSOT)"
```

---

### Task 10: Measure speed error against velocity truth

**Files:**
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs` (extend `BodyChannelError` + `BodyAccumulator` + `accuracy_baseline_against`)
- Test: same file

**Interfaces:**
- Consumes: `artifact.lookup_motion` (Task 3), hold-out `SnapshotEntry` velocity (`vx_km_s`/`vy_km_s`/`vz_km_s`, Task 6/7), `frame_recombine::cartesian_state_to_spherical` (Task 2 — expose a small pub(crate) conversion or replicate the inverse-Jacobian formula here), `AU_IN_KM`.
- Produces: `BodyChannelError` gains `max_lon_speed_arcsec_per_day`, `max_lat_speed_arcsec_per_day`, `max_radial_speed_au_per_day` (RMS optional); accumulation compares artifact motion vs truth motion derived from the hold-out velocity.

**Truth conversion:** hold-out stores ecliptic Cartesian position (km) + velocity (km/s). Convert to AU and per-day (`pos_au = km/AU_IN_KM`, `vel_au_day = km_s * 86400.0 / AU_IN_KM`), feed `CartesianState` into `cartesian_state_to_spherical` → truth `(dλ/dt, dβ/dt, dr/dt)` in rad/day & AU/day → deg/day. Compare to `artifact.lookup_motion` per channel; lon/lat error → arcsec/day (`deg_diff * 3600`), radial in AU/day. Rows whose `vx_km_s` is `None` contribute to position channels only (skip speed), so the existing 5-field slices remain usable.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn baseline_reports_speed_error_fields() {
    // synthetic_holdout currently has no velocity; add an entry WITH velocity and a
    // matching synthetic artifact segment whose derivative reproduces it, then assert
    // the speed error is ~0 and the new fields exist.
    let errors = accuracy_baseline_against(&synthetic_holdout_with_velocity(), &synthetic_artifact_linear());
    let sun = &errors[0];
    assert!(sun.max_lon_speed_arcsec_per_day < 1e-3);
}
```

Add `synthetic_holdout_with_velocity()` (a Sun entry at J2000 with a known ecliptic velocity) and `synthetic_artifact_linear()` (a Sun segment whose longitude is linear so the analytic speed matches) in the test module.

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p pleiades-data baseline_reports_speed_error_fields`
Expected: FAIL — field/method missing.

- [ ] **Step 3: Implement** — add the three speed fields to `BodyChannelError` and `BodyAccumulator`; in `accuracy_baseline_against`, when `entry.vx_km_s.is_some()`, compute truth motion + `artifact.lookup_motion`, accumulate per-channel speed error; extend `summary_line()` to append the speed maxima. Keep position-channel logic unchanged.

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p pleiades-data baseline_reports_speed_error_fields baseline_reports_zero_error_for_an_artifact_that_matches_holdout`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/accuracy_baseline.rs
git commit -m "feat(data): measure per-body speed error vs hold-out velocity truth"
```

---

### Task 11: Hard accuracy-ceiling gate (4 channels) + size-budget gate; refresh golden

**Files:**
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs` (new gate tests; update committed golden incl. speed)
- Test: same file

**Interfaces:**
- Consumes: `thresholds::accuracy_ceiling`, `thresholds::PACKAGED_BUDGETS`, `packaged_artifact_accuracy_baseline()`, the encoded artifact bytes (existing accessor for the committed `.bin` length).

- [ ] **Step 1: Finalize speed/distance ceilings from measurement** — run the maintainer print helper to read the live speed + distance maxima for all 10 majors:

Run: `cargo test -p pleiades-data print_packaged_artifact_baseline_summary -- --ignored --nocapture`
Then set `thresholds.rs` speed/distance ceilings to round numbers ≥~10× the measured maxima (keep within the structure already there).

- [ ] **Step 2: Write the failing gates**

```rust
#[test]
fn all_channels_within_published_ceilings_for_major_bodies() {
    let baseline = packaged_artifact_accuracy_baseline();
    for e in &baseline {
        let c = crate::thresholds::accuracy_ceiling(&e.body);
        assert!(e.max_longitude_arcsec <= c.lon_arcsec, "{:?} lon {} > {}", e.body, e.max_longitude_arcsec, c.lon_arcsec);
        assert!(e.max_latitude_arcsec <= c.lat_arcsec, "{:?} lat", e.body);
        assert!(e.max_distance_km <= c.dist_km, "{:?} dist", e.body);
        assert!(e.max_lon_speed_arcsec_per_day <= c.lon_speed_arcsec_per_day, "{:?} lon speed", e.body);
        assert!(e.max_lat_speed_arcsec_per_day <= c.lat_speed_arcsec_per_day, "{:?} lat speed", e.body);
        assert!(e.max_radial_speed_au_per_day <= c.radial_speed_au_per_day, "{:?} radial speed", e.body);
    }
}

#[test]
fn encoded_artifact_within_size_budget() {
    let bytes_len = crate::packaged_artifact_encoded_len(); // existing accessor for committed .bin length
    assert!(
        bytes_len <= crate::thresholds::PACKAGED_BUDGETS.max_encoded_bytes,
        "encoded artifact {} bytes exceeds budget {}",
        bytes_len, crate::thresholds::PACKAGED_BUDGETS.max_encoded_bytes
    );
}
```

(If no `packaged_artifact_encoded_len()` exists, use the committed `.bin` `include_bytes!`/`len()` the crate already references for decode; read lib.rs around the embedded artifact bytes.)

- [ ] **Step 3: Run to verify** — gates should PASS immediately (measured is far under ceilings). If a speed ceiling is too tight, widen it in `thresholds.rs` per Step 1.

Run: `cargo test -p pleiades-data all_channels_within_published_ceilings_for_major_bodies encoded_artifact_within_size_budget`
Expected: PASS.

- [ ] **Step 4: Update the committed golden** — regenerate and paste the new summary (now including speed maxima) into `packaged_artifact_baseline_summary_matches_committed_golden`:

Run: `cargo test -p pleiades-data print_packaged_artifact_baseline_summary -- --ignored --nocapture`
Update the per-body `assert!(report.contains(...))` lines (anchor speed buckets to first 3 significant digits, same convention as the position channels). Run the golden test to confirm.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/accuracy_baseline.rs crates/pleiades-data/src/thresholds.rs
git commit -m "test(data): hard accuracy-ceiling (4 channels) + size-budget gates; golden incl. speed"
```

---

### Task 12: Eros documented target + self-consistency check

**Files:**
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs` (Eros self-consistency test)
- Test: same file

**Interfaces:**
- Consumes: `thresholds::accuracy_ceiling(&Eros)` (Asteroid class), the committed reference snapshot Eros was fit from, `artifact.lookup_ecliptic`.

- [ ] **Step 1: Write the test** — Eros has no independent hold-out, so assert *self-consistency*: the artifact reproduces the reference snapshot it was fit from within the Asteroid-class ceiling, and document this is not an independent-truth gate.

```rust
#[test]
fn eros_round_trips_against_its_reference_snapshot_within_documented_target() {
    // Eros is re-derived from the committed reference snapshot (no independent truth).
    // This is a self-consistency check, NOT an independent-truth gate.
    let ceiling = crate::thresholds::accuracy_ceiling(&CelestialBody::Eros);
    let max_lon_arcsec = eros_self_consistency_max_longitude_arcsec(); // helper: snapshot vs artifact
    assert!(max_lon_arcsec <= ceiling.lon_arcsec, "Eros self-consistency {max_lon_arcsec}\" > {}\"", ceiling.lon_arcsec);
}
```

Implement `eros_self_consistency_max_longitude_arcsec()` by comparing `artifact.lookup_ecliptic(&Eros, epoch)` to the committed Eros reference-snapshot rows (reuse the snapshot loader the SP1/SP2 Eros fit used).

- [ ] **Step 2: Run to verify fail then pass** — write helper, run:

Run: `cargo test -p pleiades-data eros_round_trips_against_its_reference_snapshot_within_documented_target`
Expected: PASS (self-consistency is tight by construction).

- [ ] **Step 3: Commit**

```bash
git add crates/pleiades-data/src/accuracy_baseline.rs
git commit -m "test(data): Eros documented target + self-consistency check (not an independent gate)"
```

---

### Task 13: CLI thresholds summary + help-sync + golden

**Files:**
- Modify: `crates/pleiades-data/src/thresholds.rs` (add `pub fn packaged_artifact_thresholds_summary_for_report() -> String`)
- Modify: `crates/pleiades-validate/src/render/cli.rs` (dispatch arm ~1863 + `help_text()` ~1913)
- Modify: `crates/pleiades-cli/src/cli/tests/help.rs` (help-sync assertions)
- Test: `crates/pleiades-data/src/thresholds.rs` (golden) and help test

**Interfaces:**
- Consumes: `accuracy_ceiling`, `PACKAGED_BUDGETS`, `packaged_artifact_accuracy_baseline()`.
- Produces: command `packaged-artifact-thresholds-summary` (+ alias `artifact-thresholds`) rendering published ceiling + live measured + pass/margin per body class × channel, plus the size budget line.

- [ ] **Step 1: Implement the report function** (follow the `OnceLock` + format pattern from `coverage/threshold.rs`):

```rust
pub fn packaged_artifact_thresholds_summary_for_report() -> String {
    let baseline = crate::accuracy_baseline::packaged_artifact_accuracy_baseline();
    let mut lines = Vec::new();
    for e in &baseline {
        let c = accuracy_ceiling(&e.body);
        lines.push(format!(
            "{:?}: lon {:.4}\"/{:.1}\"  lat {:.4}\"/{:.1}\"  dist {:.1}/{:.0} km  lon_spd {:.4}/{:.1} \"/d  PASS",
            e.body, e.max_longitude_arcsec, c.lon_arcsec, e.max_latitude_arcsec, c.lat_arcsec,
            e.max_distance_km, c.dist_km, e.max_lon_speed_arcsec_per_day, c.lon_speed_arcsec_per_day,
        ));
    }
    format!(
        "Packaged-artifact thresholds ({} bodies); size budget {} bytes:\n{}",
        baseline.len(), PACKAGED_BUDGETS.max_encoded_bytes, lines.join("\n")
    )
}
```

- [ ] **Step 2: Add CLI dispatch + help text** — in `render/cli.rs`, add (mirroring the `packaged-artifact-accuracy-baseline-summary` arm at 1863):

```rust
Some("packaged-artifact-thresholds-summary") | Some("artifact-thresholds") => {
    ensure_no_extra_args(&args[1..], "packaged-artifact-thresholds-summary")?;
    Ok(pleiades_data::thresholds::packaged_artifact_thresholds_summary_for_report())
}
```

Add two lines to `help_text()`:
```
  packaged-artifact-thresholds-summary  Print published accuracy ceilings + size budget vs measured
  artifact-thresholds  Alias for packaged-artifact-thresholds-summary
```

- [ ] **Step 3: Add help-sync assertions** — in `crates/pleiades-cli/src/cli/tests/help.rs`:

```rust
assert!(help.contains("packaged-artifact-thresholds-summary  Print published accuracy ceilings"));
assert!(help.contains("artifact-thresholds  Alias for packaged-artifact-thresholds-summary"));
```

- [ ] **Step 4: Add a golden test + maintainer helper** for the new summary (mirror accuracy_baseline.rs:441-504): one `#[ignore]` printer, one `..._matches_committed_golden` asserting the header + one body line + `PASS`.

- [ ] **Step 5: Run + commit**

Run: `cargo test -p pleiades-cli help && cargo test -p pleiades-data thresholds`
Expected: PASS.
```bash
git add crates/pleiades-data/src/thresholds.rs crates/pleiades-validate/src/render/cli.rs crates/pleiades-cli/src/cli/tests/help.rs
git commit -m "feat(cli): packaged-artifact-thresholds-summary command + help-sync + golden"
```

---

### Task 14: Tracked latency-budget summary + opt-in gate

**Files:**
- Modify: `crates/pleiades-data/src/thresholds.rs` or a small `crates/pleiades-validate/src/render/text/benchmark.rs` helper (latency vs budget summary)
- Test: `crates/pleiades-validate` (opt-in gate `#[ignore]`)

**Interfaces:**
- Consumes: `benchmark_packaged_artifact_decode/lookup/batch_lookup` (`pleiades-validate/src/artifact/inspection.rs:339-419`, return `Duration`), `PACKAGED_BUDGETS`.
- Produces: a `packaged-artifact-latency-budget-summary` CLI command rendering measured-vs-target with margin (non-gating), and an `#[ignore]`/env-gated hard check.

- [ ] **Step 1: Implement the summary** — call the existing benchmark fns (small `rounds`, e.g. 16), format `decode {measured}ms / target {target}ms (margin)`, same for single-lookup and batch. Non-gating: always returns the string.

- [ ] **Step 2: Add CLI dispatch + help + help-sync** — same pattern as Task 13 for `packaged-artifact-latency-budget-summary` (+ alias). Add the two help lines and the two help-sync assertions.

- [ ] **Step 3: Add the opt-in gate**

```rust
#[test]
#[ignore = "latency is environment-sensitive; opt-in via PLEIADES_ENFORCE_LATENCY=1 (see SP2 SDD log: benchmark tests flake under concurrent load)"]
fn latency_within_budget_when_enforced() {
    if std::env::var("PLEIADES_ENFORCE_LATENCY").is_err() { return; }
    let decode_ms = /* benchmark_packaged_artifact_decode(16) -> ms */;
    assert!(decode_ms <= crate::PACKAGED_BUDGETS_OR_REEXPORT.decode_latency_target_ms);
    // ... single lookup, batch ...
}
```

- [ ] **Step 4: Run (summary + help only; gate stays ignored) + commit**

Run: `cargo test -p pleiades-cli help && cargo test -p pleiades-validate latency_within_budget_when_enforced -- --ignored` (the `--ignored` run should pass or no-op without the env var)
```bash
git add -A
git commit -m "feat(validate): tracked latency-budget summary + opt-in enforcement gate"
```

---

# PART 3 — Documentation reconciliation

### Task 15: Reconcile spec/plan/README to 1900–2100 + motion capability

**Files:**
- Modify: `spec/data-compression.md` (§Accuracy Targets), `plan/stages/02-production-compressed-ephemeris.md`, `plan/status/01-current-execution-frontier.md`, `plan/status/02-next-slice-candidates.md`, `plan/stages/04-advanced-request-modes.md`, `README.md`, `PLAN.md`
- Test: `cargo test --workspace` (catch any doc-sync/help-sync tests)

**Interfaces:** none (docs).

- [ ] **Step 1: `spec/data-compression.md` §Accuracy Targets** — replace the "for example" envelopes with the actual published per-class ceilings from `thresholds.rs`; state the **1900–2100** first-release window; note 1600–2600 as documented future expansion. Name `StoredFrame::Geocentric` in the geocentric section (the SP2 minor finding).

- [ ] **Step 2: `plan/stages/02`** — change the exit-criteria window 1600–2600 → 1900–2100; mark **SP3 done** (thresholds + budgets + motion-derived landed); update the "measured, not budgeted" line to "budgeted (size hard-gated, latency tracked)".

- [ ] **Step 3: `plan/status/01` & `02`** — remove the stale "SP2 is next"; record SP2 done + SP3 done; set the new frontier to Phase 3 (body/backend claim closure).

- [ ] **Step 4: `plan/stages/04-advanced-request-modes.md`** — record that motion/speed output (`FittedDerivative`, `Motion = Derived`) was implemented in SP3; narrow the remaining Phase 4 motion scope to apparent/topocentric/sidereal/civil-time only.

- [ ] **Step 5: `README.md` + `PLAN.md`** — drop "SP3 ... not yet completed"; note packaged motion/speed is derived and that accuracy thresholds + size budget are enforced (1900–2100), latency tracked.

- [ ] **Step 6: Full workspace test + commit**

Run: `cargo test --workspace` (do NOT run concurrently with other cargo invocations)
Expected: PASS across the workspace.
```bash
git add spec/ plan/ README.md PLAN.md
git commit -m "docs: record SP3 thresholds/budgets + motion-derived; reconcile to 1900-2100 window"
```

---

## Self-Review

**Spec coverage:**
- Coverage window 1900–2100 → Tasks 9, 11, 15. ✓
- Two-tier (published ceiling + retained golden) → Tasks 9 (ceilings), 11 (gate + golden refresh, golden kept). ✓
- All four channels incl. speed → Tasks 1–3 (derive), 10 (measure), 11 (gate). ✓
- Speed = `FittedDerivative` / `Motion = Derived` → Tasks 3, 4, 5. ✓
- Velocity truth → Tasks 6, 7, 8. ✓
- Hard size gate, tracked latency → Tasks 11 (size), 14 (latency). ✓
- Eros documented-not-gated → Task 12. ✓
- Integrate with existing target-threshold subsystem (no duplication) → Task 9 reuses scope vocabulary/conventions; speed not pushed into `target.rs` fit envelopes (explicitly out of scope per spec). ✓
- CLI thresholds summary + help-sync + golden → Task 13. ✓
- Docs reconciliation incl. Phase 4 note → Task 15. ✓
- ARTIFACT_VERSION decision → Task 7 Step 5. ✓

**Placeholder scan:** No "TBD"/"add error handling"/"similar to". Speed/distance ceiling *values* are explicitly finalized-from-measurement in Task 11 Step 1 with a concrete procedure (not a placeholder). Spots that say "match the real names in the file / read X first" are deliberate guards against the few signatures that must be confirmed at the edit site (artifact.rs segment-locate internals; the encoded-len accessor; the Segment residual builder) — each names the exact lines to read.

**Type consistency:** `Motion` fields (`longitude_deg_per_day`/`latitude_deg_per_day`/`distance_au_per_day`) consistent across Tasks 3/5/10. `AccuracyCeiling` field names consistent across Tasks 9/11/13. `SphericalState`/`CartesianState` consistent across Tasks 2/3/10. `BodyClass` variants consistent across Tasks 9/11/12. ✓
