# Heliocentric Re-Frame for Outer-Planet Accuracy — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Store the eight planets (Mercury–Pluto) heliocentrically and reconstruct geocentric ecliptic at lookup via `P_geo = P_helio + S_geo`, dropping outer-planet longitude error from ~100–190″ to a few arcsec without regressing inner bodies, Sun, or Moon.

**Architecture:** A planet's geocentric longitude inherits Earth's ~1-year retrograde signal, which a low-order polynomial over a long span cannot fit. We move that signal out of the planet fit: planets are fit in the smooth heliocentric frame, and the already-stored geocentric Sun supplies the Earth-position term at lookup. Sun, Moon, and Eros stay geocentric. The codec, channel structure, quantization, residual mechanism, and span limits are unchanged; only the per-body frame and a small recombination step are added.

**Tech Stack:** Rust workspace (`pleiades-compression`, `pleiades-data`, `pleiades-jpl`), `cargo`/`mise`, pure-Rust SPK reader, de440 kernel for kernel-gated generation.

**Spec:** `docs/superpowers/specs/2026-06-19-heliocentric-reframe-accuracy-design.md`

## Global Constraints

- License headers / dual license unchanged: `MIT OR Apache-2.0`.
- No new external dependencies. Pure Rust only; the math uses `std`/`core` trig.
- Determinism: generation stays byte-deterministic and kernel-gated behind `PLEIADES_DE_KERNEL`; a clean checkout must verify the committed artifact kernel-free and reproduce it from de440.
- Fail-closed: every new validation path returns `CompressionError`, never a partial/❮default❯ result.
- Channel ordering is fixed: `Longitude=0, Latitude=1, DistanceAu=2`. Scale exponents stay `Longitude=9, Latitude=9, DistanceAu=10`.
- Frame is per-**body**, not per-segment. Only `Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto` are `Heliocentric`; `Sun, Moon, Eros (asteroid:433-Eros), lunar points` stay `Geocentric`.
- The recombination must stay in the ecliptic-of-date frame (no obliquity rotation at lookup): both planet-heliocentric and Sun-geocentric channels are stored as ecliptic-of-date, so their Cartesian sum is valid in-frame.
- Combine vectors in AU.
- Public API signatures (`lookup_ecliptic`, `lookup_equatorial`, batch paths) must not change.
- Toolchain pinned by `mise.toml`; run `cargo` through the pinned toolchain.

---

### Task 1: Frame recombination math (pure functions)

Adds the only new math: ecliptic spherical ↔ Cartesian (AU) and the two recombination directions. Lives in `pleiades-compression` so both lookup (add) and generation (subtract) share one tested implementation.

**Files:**
- Create: `crates/pleiades-compression/src/frame_recombine.rs`
- Modify: `crates/pleiades-compression/src/lib.rs` (add `mod frame_recombine;` and re-export)
- Test: inline `#[cfg(test)]` module in `frame_recombine.rs`

**Interfaces:**
- Produces:
  - `pub fn ecliptic_to_cartesian_au(coords: &EclipticCoordinates) -> Option<[f64; 3]>` — `None` if distance is absent.
  - `pub fn cartesian_au_to_ecliptic(v: [f64; 3]) -> EclipticCoordinates`
  - `pub fn geocentric_from_heliocentric(planet_helio: &EclipticCoordinates, sun_geo: &EclipticCoordinates) -> Option<EclipticCoordinates>`
  - `pub fn heliocentric_from_geocentric(planet_geo: &EclipticCoordinates, sun_geo: &EclipticCoordinates) -> Option<EclipticCoordinates>`

- [ ] **Step 1: Write the failing test**

Create `crates/pleiades-compression/src/frame_recombine.rs` with only the test module:

```rust
//! Ecliptic spherical ↔ Cartesian (AU) and geocentric/heliocentric recombination.

#[cfg(test)]
mod tests {
    use super::*;
    use pleiades_types::{EclipticCoordinates, Latitude, Longitude};

    fn ec(lon: f64, lat: f64, r: f64) -> EclipticCoordinates {
        EclipticCoordinates::new(
            Longitude::from_degrees(lon),
            Latitude::from_degrees(lat),
            Some(r),
        )
    }

    #[test]
    fn cartesian_round_trips_within_tolerance() {
        let original = ec(123.456, -4.321, 9.87);
        let v = ecliptic_to_cartesian_au(&original).unwrap();
        let back = cartesian_au_to_ecliptic(v);
        assert!((back.longitude.degrees() - 123.456).abs() < 1e-9);
        assert!((back.latitude.degrees() - (-4.321)).abs() < 1e-9);
        assert!((back.distance_au.unwrap() - 9.87).abs() < 1e-9);
    }

    #[test]
    fn helio_and_geo_are_inverse_via_sun() {
        // Known truth: planet geocentric, Sun geocentric. Heliocentric = geo - sun;
        // reconstructing geo = helio + sun must return the original geocentric value.
        let planet_geo = ec(200.0, 1.5, 19.2);
        let sun_geo = ec(95.0, 0.0, 1.0);
        let helio = heliocentric_from_geocentric(&planet_geo, &sun_geo).unwrap();
        let geo_back = geocentric_from_heliocentric(&helio, &sun_geo).unwrap();
        assert!((geo_back.longitude.degrees() - 200.0).abs() < 1e-9);
        assert!((geo_back.latitude.degrees() - 1.5).abs() < 1e-9);
        assert!((geo_back.distance_au.unwrap() - 19.2).abs() < 1e-9);
    }

    #[test]
    fn missing_distance_yields_none() {
        let no_dist = EclipticCoordinates::new(
            Longitude::from_degrees(10.0),
            Latitude::from_degrees(0.0),
            None,
        );
        assert!(ecliptic_to_cartesian_au(&no_dist).is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression frame_recombine`
Expected: FAIL to compile — `ecliptic_to_cartesian_au` not found.

- [ ] **Step 3: Write minimal implementation**

Prepend the implementation above the test module in `frame_recombine.rs`:

```rust
use pleiades_types::{EclipticCoordinates, Latitude, Longitude};

/// Converts ecliptic spherical (deg, deg, AU) to ecliptic Cartesian (AU).
/// Returns `None` when distance is absent — recombination requires a radius.
pub fn ecliptic_to_cartesian_au(coords: &EclipticCoordinates) -> Option<[f64; 3]> {
    let r = coords.distance_au?;
    let lon = coords.longitude.degrees().to_radians();
    let lat = coords.latitude.degrees().to_radians();
    Some([
        r * lat.cos() * lon.cos(),
        r * lat.cos() * lon.sin(),
        r * lat.sin(),
    ])
}

/// Converts ecliptic Cartesian (AU) back to ecliptic spherical. Longitude is
/// normalized to [0, 360) by `Longitude::from_degrees`.
pub fn cartesian_au_to_ecliptic(v: [f64; 3]) -> EclipticCoordinates {
    let [x, y, z] = v;
    let radius = (x * x + y * y + z * z).sqrt();
    let longitude = Longitude::from_degrees(y.atan2(x).to_degrees());
    let latitude = if radius == 0.0 {
        Latitude::from_degrees(0.0)
    } else {
        Latitude::from_degrees((z / radius).clamp(-1.0, 1.0).asin().to_degrees())
    };
    EclipticCoordinates::new(longitude, latitude, Some(radius))
}

/// Reconstructs geocentric ecliptic from a planet's heliocentric ecliptic and
/// the geocentric Sun: `P_geo = P_helio + S_geo` (vector add in ecliptic-of-date).
pub fn geocentric_from_heliocentric(
    planet_helio: &EclipticCoordinates,
    sun_geo: &EclipticCoordinates,
) -> Option<EclipticCoordinates> {
    let p = ecliptic_to_cartesian_au(planet_helio)?;
    let s = ecliptic_to_cartesian_au(sun_geo)?;
    Some(cartesian_au_to_ecliptic([p[0] + s[0], p[1] + s[1], p[2] + s[2]]))
}

/// Derives a planet's heliocentric ecliptic from its geocentric ecliptic and
/// the geocentric Sun: `P_helio = P_geo − S_geo` (vector subtract in ecliptic-of-date).
pub fn heliocentric_from_geocentric(
    planet_geo: &EclipticCoordinates,
    sun_geo: &EclipticCoordinates,
) -> Option<EclipticCoordinates> {
    let p = ecliptic_to_cartesian_au(planet_geo)?;
    let s = ecliptic_to_cartesian_au(sun_geo)?;
    Some(cartesian_au_to_ecliptic([p[0] - s[0], p[1] - s[1], p[2] - s[2]]))
}
```

Add to `crates/pleiades-compression/src/lib.rs` near the other `mod`/`pub use` lines:

```rust
mod frame_recombine;
pub use frame_recombine::{
    cartesian_au_to_ecliptic, ecliptic_to_cartesian_au, geocentric_from_heliocentric,
    heliocentric_from_geocentric,
};
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p pleiades-compression frame_recombine`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/frame_recombine.rs crates/pleiades-compression/src/lib.rs
git commit -m "feat(compression): ecliptic Cartesian recombination helpers for frame reframe"
```

---

### Task 2: `StoredFrame` enum + per-body frame field (no serialization yet)

Adds the frame to `BodyArtifact` with a `Geocentric` default so all existing in-memory and committed-byte paths keep working. Serialization and the version bump come in Task 5.

**Files:**
- Modify: `crates/pleiades-compression/src/channels.rs` (enum + field + constructors)
- Modify: `crates/pleiades-compression/src/lib.rs` (re-export `StoredFrame`)
- Test: inline `#[cfg(test)]` in `channels.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces:
  - `pub enum StoredFrame { Geocentric, Heliocentric }` (derives `Clone, Copy, Debug, Eq, PartialEq, Hash`; `serde` behind the existing feature gate).
  - `BodyArtifact { pub body, pub segments, pub frame: StoredFrame }`.
  - `BodyArtifact::new(body, segments)` unchanged signature, sets `frame: Geocentric`.
  - `BodyArtifact::with_frame(body: CelestialBody, segments: Vec<Segment>, frame: StoredFrame) -> Self`.

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/pleiades-compression/src/channels.rs`:

```rust
#[cfg(test)]
mod frame_field_tests {
    use super::*;
    use pleiades_types::CelestialBody;

    #[test]
    fn new_defaults_to_geocentric() {
        let b = BodyArtifact::new(CelestialBody::Sun, vec![]);
        assert_eq!(b.frame, StoredFrame::Geocentric);
    }

    #[test]
    fn with_frame_sets_heliocentric() {
        let b = BodyArtifact::with_frame(CelestialBody::Jupiter, vec![], StoredFrame::Heliocentric);
        assert_eq!(b.frame, StoredFrame::Heliocentric);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression frame_field`
Expected: FAIL to compile — `StoredFrame` / `frame` / `with_frame` not found.

- [ ] **Step 3: Write minimal implementation**

In `crates/pleiades-compression/src/channels.rs`, add the enum above `BodyArtifact`:

```rust
/// The coordinate frame a body's stored channels are expressed in.
///
/// `Geocentric` channels are returned directly at lookup. `Heliocentric` channels
/// are recombined with the geocentric Sun (`P_geo = P_helio + S_geo`) before being
/// returned, so the public lookup output is always geocentric ecliptic.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum StoredFrame {
    /// Stored channels are geocentric ecliptic; returned as-is.
    Geocentric,
    /// Stored channels are heliocentric ecliptic; recombined with the Sun at lookup.
    Heliocentric,
}
```

Replace the `BodyArtifact` struct and its `new` constructor:

```rust
pub struct BodyArtifact {
    /// Body identifier.
    pub body: CelestialBody,
    /// Time segments for the body.
    pub segments: Vec<Segment>,
    /// Frame the stored channels are expressed in.
    pub frame: StoredFrame,
}

impl BodyArtifact {
    /// Creates a new geocentric body artifact (the default frame).
    pub fn new(body: CelestialBody, segments: Vec<Segment>) -> Self {
        Self { body, segments, frame: StoredFrame::Geocentric }
    }

    /// Creates a body artifact with an explicit stored frame.
    pub fn with_frame(body: CelestialBody, segments: Vec<Segment>, frame: StoredFrame) -> Self {
        Self { body, segments, frame }
    }
```

(Leave the rest of the `impl` block — `validate`, `summary_line`, `segment_at` — unchanged.)

Update the literal construction in `decode_body` so it still compiles. In `crates/pleiades-compression/src/codec.rs`, change:

```rust
    Ok(crate::channels::BodyArtifact { body, segments })
```

to:

```rust
    Ok(crate::channels::BodyArtifact {
        body,
        segments,
        frame: crate::channels::StoredFrame::Geocentric,
    })
```

Add to `crates/pleiades-compression/src/lib.rs` exports (alongside `BodyArtifact`):

```rust
pub use channels::StoredFrame;
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-compression`
Expected: PASS — existing tests unaffected (default frame), new `frame_field` tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/channels.rs crates/pleiades-compression/src/codec.rs crates/pleiades-compression/src/lib.rs
git commit -m "feat(compression): add per-body StoredFrame (geocentric default), no serialization yet"
```

---

### Task 3: Heliocentric lookup reconstruction + Sun-presence invariant

Makes `lookup_ecliptic` recombine heliocentric bodies, and makes `validate()` reject a heliocentric body with no geocentric Sun. Tested entirely with in-memory synthetic artifacts (no committed bytes, no kernel).

**Files:**
- Modify: `crates/pleiades-compression/src/artifact.rs` (`lookup_ecliptic`, `validate`)
- Test: inline `#[cfg(test)]` in `artifact.rs` (or the existing tests module)

**Interfaces:**
- Consumes: `geocentric_from_heliocentric` (Task 1), `StoredFrame` and `BodyArtifact::frame` (Task 2).
- Produces: `lookup_ecliptic` returns geocentric ecliptic for both frames; `validate()` enforces the Sun-presence invariant.

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/pleiades-compression/src/artifact.rs`. (If a `#[cfg(test)] mod tests` already exists there, append; otherwise create it. Use the existing test helpers/patterns in that file for building a `CompressedArtifact` — the snippet below assumes constructor helpers; adapt to the file's existing artifact-building helper if one is present.)

```rust
#[cfg(test)]
mod reframe_lookup_tests {
    use super::*;
    use crate::channels::{BodyArtifact, ChannelKind, PolynomialChannel, Segment, StoredFrame};
    use crate::frame_recombine::heliocentric_from_geocentric;
    use pleiades_types::{
        CelestialBody, EclipticCoordinates, Instant, JulianDay, Latitude, Longitude, TimeScale,
    };

    // Builds a single-segment body whose three channels are constant (degree-0)
    // equal to the given ecliptic coordinates across the whole span.
    fn const_body(
        body: CelestialBody,
        frame: StoredFrame,
        start: f64,
        end: f64,
        coords: &EclipticCoordinates,
    ) -> BodyArtifact {
        let channels = vec![
            PolynomialChannel::new(ChannelKind::Longitude, 9, vec![coords.longitude.degrees()]),
            PolynomialChannel::new(ChannelKind::Latitude, 9, vec![coords.latitude.degrees()]),
            PolynomialChannel::new(ChannelKind::DistanceAu, 10, vec![coords.distance_au.unwrap()]),
        ];
        let seg = Segment::new(
            Instant::new(JulianDay::from_days(start), TimeScale::Tt),
            Instant::new(JulianDay::from_days(end), TimeScale::Tt),
            channels,
        );
        BodyArtifact::with_frame(body, vec![seg], frame)
    }

    #[test]
    fn heliocentric_body_reconstructs_geocentric() {
        let sun_geo = EclipticCoordinates::new(
            Longitude::from_degrees(95.0),
            Latitude::from_degrees(0.0),
            Some(1.0),
        );
        let jupiter_geo = EclipticCoordinates::new(
            Longitude::from_degrees(200.0),
            Latitude::from_degrees(1.2),
            Some(5.4),
        );
        let jupiter_helio = heliocentric_from_geocentric(&jupiter_geo, &sun_geo).unwrap();

        let artifact = build_test_artifact(vec![
            const_body(CelestialBody::Sun, StoredFrame::Geocentric, 0.0, 100.0, &sun_geo),
            const_body(CelestialBody::Jupiter, StoredFrame::Heliocentric, 0.0, 100.0, &jupiter_helio),
        ]);

        let at = Instant::new(JulianDay::from_days(50.0), TimeScale::Tt);
        let out = artifact.lookup_ecliptic(&CelestialBody::Jupiter, at).unwrap();
        assert!((out.longitude.degrees() - 200.0).abs() < 1e-6);
        assert!((out.latitude.degrees() - 1.2).abs() < 1e-6);
        assert!((out.distance_au.unwrap() - 5.4).abs() < 1e-6);
    }

    #[test]
    fn heliocentric_body_without_sun_fails_validation() {
        let jupiter_helio = EclipticCoordinates::new(
            Longitude::from_degrees(120.0),
            Latitude::from_degrees(0.5),
            Some(5.0),
        );
        let result = try_build_test_artifact(vec![const_body(
            CelestialBody::Jupiter,
            StoredFrame::Heliocentric,
            0.0,
            100.0,
            &jupiter_helio,
        )]);
        assert!(result.is_err(), "heliocentric body without a Sun must fail validation");
    }
}
```

`build_test_artifact` / `try_build_test_artifact` are thin local helpers: construct a `CompressedArtifact` whose profile advertises `EclipticCoordinates` as a derived output (reuse the existing test helper in `artifact.rs`/`tests.rs` that already does this for other lookup tests — search for an existing `lookup_ecliptic` test and copy its artifact-construction setup). `build_test_artifact` unwraps; `try_build_test_artifact` returns the `validate()` `Result`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression reframe_lookup`
Expected: FAIL — `heliocentric_body_reconstructs_geocentric` returns the raw heliocentric value (not 200.0), and `heliocentric_body_without_sun_fails_validation` does not error yet.

- [ ] **Step 3: Write minimal implementation**

In `lookup_ecliptic` (in `crates/pleiades-compression/src/artifact.rs`), after computing the raw channel values into `EclipticCoordinates`, branch on the body's frame. Replace the tail of `lookup_ecliptic`:

```rust
        use pleiades_types::{Latitude, Longitude};
        let longitude = segment.evaluate_channel(ChannelKind::Longitude, x)?;
        let latitude = segment.evaluate_channel(ChannelKind::Latitude, x)?;
        let distance_au = segment.evaluate_channel(ChannelKind::DistanceAu, x)?;

        let stored = EclipticCoordinates::new(
            Longitude::from_degrees(longitude),
            Latitude::from_degrees(latitude),
            Some(distance_au),
        );

        let frame = self
            .body_artifact(body)
            .map(|b| b.frame)
            .unwrap_or(crate::channels::StoredFrame::Geocentric);

        match frame {
            crate::channels::StoredFrame::Geocentric => Ok(stored),
            crate::channels::StoredFrame::Heliocentric => {
                let sun_geo = self.lookup_ecliptic(&CelestialBody::Sun, instant)?;
                crate::frame_recombine::geocentric_from_heliocentric(&stored, &sun_geo).ok_or_else(
                    || {
                        CompressionError::new(
                            CompressionErrorKind::InvalidFormat,
                            "heliocentric reconstruction requires finite distances on body and Sun",
                        )
                    },
                )
            }
        }
```

(`CompressionErrorKind` is already imported in this file; if not, add it to the `use` line.)

Add the Sun-presence invariant to `validate()`:

```rust
    pub fn validate(&self) -> Result<(), CompressionError> {
        self.header.validate()?;
        validate_body_artifacts(&self.bodies)?;
        self.profile_coverage_summary().validate()?;
        for body in &self.bodies {
            body.validate()?;
        }

        // A heliocentric body is reconstructed against the geocentric Sun at lookup,
        // so a Sun body must be present. Fail closed rather than mis-reconstruct.
        let has_heliocentric = self
            .bodies
            .iter()
            .any(|b| b.frame == crate::channels::StoredFrame::Heliocentric);
        if has_heliocentric {
            let sun = self
                .bodies
                .iter()
                .find(|b| b.body == CelestialBody::Sun);
            match sun {
                Some(s) if s.frame == crate::channels::StoredFrame::Geocentric => {}
                Some(_) => {
                    return Err(CompressionError::new(
                        CompressionErrorKind::InvalidFormat,
                        "artifact has heliocentric bodies but the Sun is not stored geocentric",
                    ));
                }
                None => {
                    return Err(CompressionError::new(
                        CompressionErrorKind::InvalidFormat,
                        "artifact has heliocentric bodies but contains no Sun reference",
                    ));
                }
            }
        }

        Ok(())
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-compression`
Expected: PASS — reframe tests pass; existing tests unaffected (all-geocentric artifacts skip the new branch and the invariant).

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-compression/src/artifact.rs
git commit -m "feat(compression): reconstruct geocentric for heliocentric bodies; require Sun reference"
```

---

### Task 4: Generation — fit planets heliocentrically via Sun-subtraction

Changes the dense fit so planets are sampled, Sun-subtracted, and fit in the heliocentric frame, and tags those bodies `Heliocentric`. Sun, Moon, Eros unchanged.

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs` (`fit_segment_within_span`, `build_packaged_artifact_from_reference_over`)
- Test: kernel-free synthetic-backend test in the existing `crates/pleiades-data/src/tests/fit.rs`

**Interfaces:**
- Consumes: `heliocentric_from_geocentric` (Task 1), `StoredFrame` + `BodyArtifact::with_frame` (Task 2).
- Produces:
  - `pub(crate) fn body_uses_heliocentric_frame(body: &CelestialBody) -> bool` — true for Mercury–Pluto only.
  - planet `BodyArtifact`s tagged `StoredFrame::Heliocentric`.

- [ ] **Step 1: Write the failing test**

Add to `crates/pleiades-data/src/tests/fit.rs`. Use the existing synthetic `EphemerisBackend` test double in that module if present; otherwise build a tiny one that returns a known geocentric ecliptic for the queried body (planet and Sun). The assertion: a planet segment fit through `fit_segment_within_span` stores heliocentric values, i.e. evaluating the stored longitude differs from the geocentric truth and equals `geo − sun` recombined.

```rust
#[test]
fn planet_segment_is_fit_in_heliocentric_frame() {
    use pleiades_compression::heliocentric_from_geocentric;
    use pleiades_types::CelestialBody;

    // Synthetic backend: Jupiter at fixed geocentric ecliptic, Sun at fixed geocentric ecliptic.
    let backend = FixedEclipticBackend::new()
        .with(CelestialBody::Jupiter, /*lon*/ 200.0, /*lat*/ 1.2, /*au*/ 5.4)
        .with(CelestialBody::Sun, 95.0, 0.0, 1.0);

    let seg = crate::regenerate::fit_segment_within_span(
        &CelestialBody::Jupiter,
        2_451_545.0,
        2_451_545.0 + 30.0,
        &backend,
    )
    .expect("segment should fit");

    // Stored longitude (degree-0/constant for a constant source) must equal the
    // HELIOCENTRIC longitude, not the geocentric 200.0.
    let stored_lon = seg
        .channels
        .iter()
        .find(|c| c.kind == pleiades_compression::ChannelKind::Longitude)
        .unwrap()
        .coefficients[0];

    let expected = heliocentric_from_geocentric(
        &ecliptic(200.0, 1.2, 5.4),
        &ecliptic(95.0, 0.0, 1.0),
    )
    .unwrap();
    assert!((stored_lon - expected.longitude.degrees()).abs() < 1e-6);
    assert!((stored_lon - 200.0).abs() > 1.0, "must not store geocentric longitude");
}
```

`FixedEclipticBackend` and `ecliptic(..)` are local test helpers: a backend implementing `EphemerisBackend::position` that returns the registered geocentric ecliptic for the requested body regardless of instant. Reuse any existing synthetic backend in the test suite if one already provides per-body fixed coordinates.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-data planet_segment_is_fit_in_heliocentric_frame`
Expected: FAIL — `fit_segment_within_span` currently stores geocentric `200.0`.

- [ ] **Step 3: Write minimal implementation**

Add the frame predicate near `packaged_artifact_body_cadence` usage in `crates/pleiades-data/src/regenerate.rs`:

```rust
/// Bodies fit in the heliocentric frame and recombined with the geocentric Sun
/// at lookup. Only the eight true planets; Sun, Moon, Eros, and lunar points
/// stay geocentric.
pub(crate) fn body_uses_heliocentric_frame(body: &CelestialBody) -> bool {
    matches!(
        body,
        CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars
            | CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto
    )
}
```

In `fit_segment_within_span`, after `let ec = res.ecliptic?;`, convert to heliocentric for reframed bodies before pushing samples:

```rust
        let ec = res.ecliptic?;
        let ec = if body_uses_heliocentric_frame(body) {
            let sun = reference
                .position(&EphemerisRequest::new(CelestialBody::Sun, inst))
                .ok()?
                .ecliptic?;
            pleiades_compression::heliocentric_from_geocentric(&ec, &sun)?
        } else {
            ec
        };
        xs.push(frac);
        lon_deg.push(ec.longitude.degrees());
        lat.push(ec.latitude.degrees());
        dist.push(ec.distance_au?);
```

In `build_packaged_artifact_from_reference_over`, tag the planet bodies. In the major-body arm, change the `BodyArtifact::new(body, segments)` construction to:

```rust
                        let frame = if body_uses_heliocentric_frame(&body) {
                            pleiades_compression::StoredFrame::Heliocentric
                        } else {
                            pleiades_compression::StoredFrame::Geocentric
                        };
                        (body_index, BodyArtifact::with_frame(body, segments, frame))
```

(Leave the asteroid arm using `BodyArtifact::new(body, segments)` — Eros stays geocentric.)

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p pleiades-data planet_segment_is_fit_in_heliocentric_frame`
Then: `cargo test -p pleiades-data` (kernel-free suite)
Expected: PASS. Kernel-gated tests that decode the committed (still-v5) artifact may now mismatch in-memory frames; those are addressed in Task 5. If any kernel-free test asserts the in-memory regenerated artifact validates, it should still pass because the Sun is geocentric and present.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/regenerate.rs crates/pleiades-data/src/tests/fit.rs
git commit -m "feat(data): fit planets heliocentrically via Sun-subtraction; tag StoredFrame"
```

---

### Task 5: Serialize frame, bump ARTIFACT_VERSION 5→6, regenerate committed artifact

Atomic format change: write/read the frame byte, bump the version, and regenerate + commit the v6 artifact bytes from de440. **Requires `PLEIADES_DE_KERNEL`.** Until the bytes are regenerated the committed-artifact decode fails, so these steps land together.

**Files:**
- Modify: `crates/pleiades-compression/src/codec.rs` (`encode_body`, `decode_body`)
- Modify: `crates/pleiades-compression/src/lib.rs` (`ARTIFACT_VERSION = 6`)
- Modify: committed artifact bytes under `crates/pleiades-data/` (the checked-in `.bin`; path per the existing `regenerate_packaged_artifact_bytes` include)
- Modify: any test asserting `ARTIFACT_VERSION == 5` and the reproduce test `crates/pleiades-data/tests/artifact_regen.rs`
- Test: codec round-trip for frame in `crates/pleiades-compression/src/tests.rs`; existing kernel-gated reproduce test

**Interfaces:**
- Consumes: `StoredFrame` (Task 2), heliocentric generation (Task 4).
- Produces: v6 artifact format carrying a per-body frame byte; committed v6 bytes.

- [ ] **Step 1: Write the failing test**

Add a codec round-trip test to `crates/pleiades-compression/src/tests.rs`:

```rust
#[test]
fn body_frame_round_trips_through_codec() {
    use crate::channels::{BodyArtifact, StoredFrame};
    use pleiades_types::CelestialBody;

    let body = BodyArtifact::with_frame(CelestialBody::Jupiter, vec![], StoredFrame::Heliocentric);
    let mut bytes = Vec::new();
    crate::codec::encode_body(&mut bytes, &body).unwrap();
    let mut cursor = crate::codec::Cursor::new(&bytes);
    let decoded = crate::codec::decode_body(&mut cursor).unwrap();
    assert_eq!(decoded.frame, StoredFrame::Heliocentric);
    assert_eq!(decoded.body, CelestialBody::Jupiter);
}
```

(If `Cursor` is not `pub(crate)`-reachable from the tests module, place this test inside `codec.rs`'s own `#[cfg(test)]` module instead, where `Cursor` is in scope.)

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p pleiades-compression body_frame_round_trips`
Expected: FAIL — frame is not serialized, decode yields `Geocentric`.

- [ ] **Step 3: Write minimal implementation**

In `crates/pleiades-compression/src/codec.rs`, write the frame byte in `encode_body` (after the body, before the segment count) and read it in `decode_body`:

```rust
pub(crate) fn encode_body(
    bytes: &mut Vec<u8>,
    body: &crate::channels::BodyArtifact,
) -> Result<(), CompressionError> {
    encode_celestial_body(bytes, &body.body)?;
    write_u8(bytes, encode_stored_frame(body.frame));
    write_u32(bytes, body.segments.len() as u32);
    for segment in &body.segments {
        encode_segment(bytes, segment)?;
    }
    Ok(())
}

fn encode_stored_frame(frame: crate::channels::StoredFrame) -> u8 {
    match frame {
        crate::channels::StoredFrame::Geocentric => 0,
        crate::channels::StoredFrame::Heliocentric => 1,
    }
}

fn decode_stored_frame(value: u8) -> Result<crate::channels::StoredFrame, CompressionError> {
    match value {
        0 => Ok(crate::channels::StoredFrame::Geocentric),
        1 => Ok(crate::channels::StoredFrame::Heliocentric),
        other => Err(CompressionError::new(
            CompressionErrorKind::InvalidFormat,
            format!("unknown stored frame tag {other}"),
        )),
    }
}
```

Update `decode_body`:

```rust
pub(crate) fn decode_body(
    cursor: &mut Cursor<'_>,
) -> Result<crate::channels::BodyArtifact, CompressionError> {
    let body = decode_celestial_body(cursor)?;
    let frame = decode_stored_frame(cursor.read_u8()?)?;
    let segment_count = cursor.read_u32()? as usize;
    let mut segments = Vec::with_capacity(segment_count);
    for _ in 0..segment_count {
        segments.push(decode_segment(cursor)?);
    }
    Ok(crate::channels::BodyArtifact { body, segments, frame })
}
```

Bump the version in `crates/pleiades-compression/src/lib.rs`:

```rust
pub const ARTIFACT_VERSION: u16 = 6;
```

- [ ] **Step 4: Run the compression suite**

Run: `cargo test -p pleiades-compression`
Expected: codec round-trip passes. Update any test that hard-codes `ARTIFACT_VERSION == 5` to `6`.

- [ ] **Step 5: Regenerate and commit the v6 artifact bytes (kernel required)**

Identify the regeneration command from the existing CLI/tooling (the one used historically, e.g. the `pleiades-cli` regenerate subcommand or the kernel-gated regenerate binary that writes the committed `.bin`). Run it with the kernel:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp <existing regenerate command that writes the committed artifact bytes>
```

Then verify the kernel-gated reproduce test:

```bash
PLEIADES_DE_KERNEL=/path/to/de440.bsp cargo test -p pleiades-data --test artifact_regen
```

Expected: the freshly committed bytes decode as v6, planets carry `Heliocentric`, and regeneration is byte-identical. If `artifact_regen.rs` pins a checksum or byte length, update it to the new value.

- [ ] **Step 6: Run the kernel-free decode path**

Run: `cargo test -p pleiades-data` (without the kernel)
Expected: PASS — `regenerate_packaged_artifact()` decodes the committed v6 bytes; lookups for planets reconstruct geocentric via the Sun.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-compression/src/codec.rs crates/pleiades-compression/src/lib.rs crates/pleiades-compression/src/tests.rs crates/pleiades-data
git commit -m "feat(data): ARTIFACT_VERSION 6 with per-body frame byte; regenerate heliocentric-planet artifact"
```

---

### Task 6: Re-measure accuracy baseline + regression gate

Re-measures the committed v6 artifact against the hold-out, commits the new per-body numbers, and adds a regression test enforcing the astrology-grade envelope.

**Files:**
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs` (committed baseline numbers + per-body envelope constants)
- Modify: the `packaged-artifact-accuracy-baseline-summary` surface if it hard-codes numbers
- Test: regression test in `accuracy_baseline.rs` test module

**Interfaces:**
- Consumes: committed v6 artifact (Task 5), existing `accuracy_baseline_against` / `production_holdout_corpus`.
- Produces: a regression test asserting per-body longitude ceilings.

- [ ] **Step 1: Write the failing test**

Add to the test module in `crates/pleiades-data/src/accuracy_baseline.rs`:

```rust
#[test]
fn outer_planet_longitude_meets_astrology_grade_envelope() {
    let baseline = crate::accuracy_baseline::packaged_artifact_accuracy_baseline();
    // Astrology-grade longitude ceilings (max abs error, arcsec).
    let ceiling = |body: &CelestialBody| -> f64 {
        match body {
            CelestialBody::Sun
            | CelestialBody::Moon
            | CelestialBody::Mercury
            | CelestialBody::Venus
            | CelestialBody::Mars => 1.0,
            CelestialBody::Jupiter
            | CelestialBody::Saturn
            | CelestialBody::Uranus
            | CelestialBody::Neptune
            | CelestialBody::Pluto => 5.0,
            _ => f64::INFINITY,
        }
    };
    for body_error in &baseline {
        let c = ceiling(&body_error.body);
        assert!(
            body_error.max_longitude_arcsec <= c,
            "{:?} longitude {:.3}\" exceeds ceiling {:.1}\"",
            body_error.body,
            body_error.max_longitude_arcsec,
            c
        );
    }
}
```

(`packaged_artifact_accuracy_baseline() -> Vec<BodyChannelError>` is the existing accessor in `accuracy_baseline.rs` that measures the committed artifact against `production_holdout_corpus()`.)

- [ ] **Step 2: Run test to verify it fails (pre-regen) or passes (post-regen)**

Run: `cargo test -p pleiades-data outer_planet_longitude_meets_astrology_grade_envelope`
Expected: with the v6 heliocentric artifact, outer-planet errors should now be within 5″. If it FAILS, the reframe did not land correctly — stop and diagnose (do not loosen the ceiling).

- [ ] **Step 3: Update the committed baseline numbers**

Re-run the baseline measurement and update the committed per-body numbers in `accuracy_baseline.rs` (and any summary surface) to the measured v6 values.

- [ ] **Step 4: Run the accuracy suite**

Run: `cargo test -p pleiades-data accuracy`
Expected: PASS — committed numbers match measurement; envelope regression test passes; inner/Sun/Moon unchanged sub-arcsec.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-data/src/accuracy_baseline.rs
git commit -m "test(data): commit v6 accuracy baseline; gate outer planets at astrology-grade envelope"
```

---

### Task 7: Documentation alignment

Aligns README, PLAN, the Phase 2 stage doc, and the data-compression spec with the heliocentric model and the new numbers (PLAN.md maintenance rule: docs track behavior changes).

**Files:**
- Modify: `README.md`, `PLAN.md` (size/perf/accuracy baseline table; current-state wording)
- Modify: `plan/stages/02-production-compressed-ephemeris.md` (SP2 records the reframe as the accuracy mechanism)
- Modify: `spec/data-compression.md` (heliocentric storage + geocentric reconstruction; co-frame invariant)

- [ ] **Step 1: Update the docs**

- In `spec/data-compression.md`, document: planets stored heliocentric, recombined with the geocentric Sun at lookup (`P_geo = P_helio + S_geo`); the co-frame (ecliptic-of-date) invariant; Sun/Moon/Eros remain geocentric.
- In `plan/stages/02-production-compressed-ephemeris.md`, mark SP2 done via the reframe and record the new outer-planet numbers.
- In `README.md` and `PLAN.md`, update the accuracy baseline figures and note the heliocentric-planet posture and `ARTIFACT_VERSION 6`.

- [ ] **Step 2: Verify doc/help sync checks pass**

Run: `cargo test -p pleiades-cli` (and any help-sync / doc-sync test the repo runs)
Expected: PASS — any summary/help text that mentions the artifact posture matches the new strings.

- [ ] **Step 3: Commit**

```bash
git add README.md PLAN.md plan/stages/02-production-compressed-ephemeris.md spec/data-compression.md
git commit -m "docs: record heliocentric-planet reframe, ARTIFACT_VERSION 6, new accuracy baseline"
```

---

## Final verification

- [ ] `cargo test --workspace` (kernel-free) passes.
- [ ] `PLEIADES_DE_KERNEL=… cargo test --workspace` passes, including the kernel-gated reproduce test.
- [ ] `cargo fmt --all --check` and `cargo clippy --workspace --all-targets` are clean (release gate).
- [ ] Outer-planet longitude ≤ 5″, inner/Sun/Moon sub-arcsec, no regression.
- [ ] Committed artifact is v6, decodes kernel-free, and reproduces byte-identically from de440.

## Self-review notes (spec coverage)

- Per-body `StoredFrame` + explicit tag → Tasks 2, 5.
- `P_geo = P_helio + S_geo` reconstruction → Tasks 1, 3.
- Sun-presence structural invariant (fail-closed) → Task 3.
- Heliocentric-via-Sun-subtraction generation, Eros geocentric → Task 4.
- ARTIFACT_VERSION 5→6, deterministic regen → Task 5.
- Accuracy re-measure + regression gate → Task 6.
- README/PLAN/stage/spec alignment → Task 7.
- Co-frame (no obliquity at lookup) invariant → enforced implicitly by Task 1 math; documented in Task 7 and asserted by Task 3's round-trip.
- Out of scope (SP3 thresholds/budgets, span re-tuning, Chebyshev, Pluto promotion, Phase 4 modes) → not implemented, consistent with spec.
