# Devkit Phase 3 proptest slice (types + compression) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add property-based tests (proptest) with invariant oracles to `pleiades-types` (angle normalization / conversion) and `pleiades-compression` (coordinate & codec round-trips), establishing a reusable proptest harness.

**Architecture:** proptest lands as a pinned workspace dev-dependency; each of the two crates opts in. Property tests are ordinary `#[test]` functions (proptest expands to them), co-located in each module's existing test file, so the blocking-tier `mise run test` (nextest) collects them with zero new CI wiring.

**Tech Stack:** Rust, proptest, cargo-nextest, mise.

## Global Constraints

- Pure Rust only — no mandatory C/C++ or native-toolchain dependency. proptest is added as `default-features = false, features = ["std"]`.
- Pin/constrain dependency versions; exact resolution lives in `Cargo.lock` (AGENTS.md).
- Co-locate a module's tests in its own test file; keep white-box unit tests as unit tests (AGENTS.md).
- Case budget: proptest default (256 cases/property). No per-test override — total added runtime must stay comfortably inside the blocking-tier ≤ 10-minute budget.
- Determinism: `proptest-regressions/` must remain committable (never gitignored) so discovered counterexamples replay deterministically.
- Scope guard: `pleiades-time` and `pleiades-houses` are OUT of scope (follow-up slice). No `cargo-fuzz` / `cargo-mutants`.

---

### Task 1: Wire the proptest harness into both crates

**Files:**
- Modify: `Cargo.toml` (root — add to `[workspace.dependencies]`)
- Modify: `crates/pleiades-types/Cargo.toml` (add `[dev-dependencies]`)
- Modify: `crates/pleiades-compression/Cargo.toml` (`[dev-dependencies]`)
- Modify: `crates/pleiades-types/src/tests.rs` (temporary smoke property)

**Interfaces:**
- Produces: `proptest` available as a dev-dependency in both crates; the `proptest!` macro and `proptest::prelude::*` usable from their test modules.

- [ ] **Step 1: Add proptest to the workspace dependency table**

In root `Cargo.toml`, under `[workspace.dependencies]`, add (keep the existing alphabetical-ish grouping; place near the other third-party deps like `serde`):

```toml
proptest = { version = "1", default-features = false, features = ["std"] }
```

- [ ] **Step 2: Opt both crates in**

`crates/pleiades-types/Cargo.toml` — add a dev-dependencies section (the crate currently has none):

```toml
[dev-dependencies]
proptest = { workspace = true }
```

`crates/pleiades-compression/Cargo.toml` — it already has `[dev-dependencies]` with `serde_json = "1"`; add the line:

```toml
proptest = { workspace = true }
```

- [ ] **Step 3: Write a temporary smoke property to prove the wiring**

In `crates/pleiades-types/src/tests.rs`, append at the end of the file:

```rust
mod proptest_smoke {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn addition_is_commutative(a in any::<i32>(), b in any::<i32>()) {
            prop_assert_eq!(a.wrapping_add(b), b.wrapping_add(a));
        }
    }
}
```

- [ ] **Step 4: Resolve dependencies and run the smoke test — verify it passes**

Run: `cargo nextest run -p pleiades-types proptest_smoke`
Expected: PASS (`1 test run: 1 passed`). This proves proptest resolves, builds pure-Rust, and the macro works.

- [ ] **Step 5: Remove the smoke property**

Delete the `mod proptest_smoke { … }` block added in Step 3 (its job — proving the wiring — is done; the real properties land in later tasks).

- [ ] **Step 6: Confirm the crate still builds without the smoke test**

Run: `cargo nextest run -p pleiades-types`
Expected: PASS (existing tests only).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock crates/pleiades-types/Cargo.toml crates/pleiades-compression/Cargo.toml
git commit -m "test(proptest): add pinned proptest dev-dependency to types + compression"
```

---

### Task 2: Property tests for `pleiades-types` angle invariants

**Files:**
- Modify: `crates/pleiades-types/src/tests.rs` (append a `proptest` module)

**Interfaces:**
- Consumes: `Angle::{from_degrees, from_radians, radians, degrees, normalized_0_360, normalized_signed}`, `Longitude::{from_degrees, degrees}` from `crate::` (re-exported at crate root; `tests.rs` already uses `Angle` directly).

- [ ] **Step 1: Write the failing test module**

Append to `crates/pleiades-types/src/tests.rs`:

```rust
mod angle_properties {
    use super::*;
    use proptest::prelude::*;

    // Bounded finite degrees: wide enough to exercise many 360° wraps, small
    // enough that absolute floating-point tolerances stay tight. Non-finite
    // inputs are deliberately excluded — normalization assumes finite values.
    fn finite_degrees() -> impl Strategy<Value = f64> {
        -1.0e4f64..1.0e4f64
    }

    proptest! {
        #[test]
        fn normalized_0_360_in_range(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_0_360().degrees();
            prop_assert!((0.0..360.0).contains(&n), "d={d} n={n}");
        }

        #[test]
        fn normalized_0_360_idempotent(d in finite_degrees()) {
            let once = Angle::from_degrees(d).normalized_0_360();
            let twice = once.normalized_0_360();
            prop_assert_eq!(once.degrees(), twice.degrees());
        }

        #[test]
        fn normalized_0_360_congruent_mod_360(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_0_360().degrees();
            let k = ((d - n) / 360.0).round();
            prop_assert!((d - n - k * 360.0).abs() < 1e-9, "d={d} n={n} k={k}");
        }

        #[test]
        fn normalized_signed_in_range(d in finite_degrees()) {
            let n = Angle::from_degrees(d).normalized_signed().degrees();
            prop_assert!((-180.0..180.0).contains(&n), "d={d} n={n}");
        }

        #[test]
        fn normalized_signed_idempotent(d in finite_degrees()) {
            let once = Angle::from_degrees(d).normalized_signed();
            let twice = once.normalized_signed();
            prop_assert_eq!(once.degrees(), twice.degrees());
        }

        #[test]
        fn degree_radian_roundtrip(d in finite_degrees()) {
            let back = Angle::from_radians(Angle::from_degrees(d).radians()).degrees();
            prop_assert!((back - d).abs() < 1e-9, "d={d} back={back}");
        }

        #[test]
        fn longitude_constructor_normalizes(d in finite_degrees()) {
            let l = Longitude::from_degrees(d).degrees();
            prop_assert!((0.0..360.0).contains(&l), "d={d} l={l}");
        }
    }
}
```

- [ ] **Step 2: Run the properties — verify they pass**

Run: `cargo nextest run -p pleiades-types angle_properties`
Expected: PASS (7 tests, each running 256 cases). If any FAILS, proptest prints a shrunk counterexample and writes `crates/pleiades-types/proptest-regressions/…`. A failure here means a genuine invariant bug — investigate the reported input against `angles.rs` before adjusting the property.

- [ ] **Step 3: Run fmt + lint**

Run: `mise run fmt && mise run lint`
Expected: both clean (no diff, no clippy warnings).

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-types/src/tests.rs
git commit -m "test(types): property tests for angle normalization + degree/radian round-trip"
```

---

### Task 3: Property tests for `pleiades-compression` coordinate & state round-trips

**Files:**
- Modify: `crates/pleiades-compression/src/frame_recombine.rs` (extend the existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes (all in scope inside `mod tests` via `use super::*`): `ecliptic_to_cartesian_au`, `cartesian_au_to_ecliptic`, `spherical_state_to_cartesian`, `cartesian_state_to_spherical`, `heliocentric_from_geocentric`, `geocentric_from_heliocentric`, `SphericalState`, and the local test helper `ec(lon, lat, r) -> EclipticCoordinates`.

- [ ] **Step 1: Write the failing properties**

Inside the existing `mod tests { … }` block in `crates/pleiades-compression/src/frame_recombine.rs` (after the existing `#[test]` functions, before the closing brace), add:

```rust
    mod properties {
        use super::*;
        use proptest::prelude::*;

        // Small positive helper: circular longitude difference in [0, 360).
        fn lon_gap(a: f64, b: f64) -> f64 {
            (a - b).rem_euclid(360.0)
        }

        proptest! {
            #[test]
            fn ecliptic_cartesian_roundtrips(
                lon in 0.0f64..360.0,
                lat in -85.0f64..85.0,   // away from the poles: longitude is ill-conditioned near ±90°
                dist in 0.1f64..100.0,
            ) {
                let v = ecliptic_to_cartesian_au(&ec(lon, lat, dist)).unwrap();
                let back = cartesian_au_to_ecliptic(v);
                let g = lon_gap(back.longitude.degrees(), lon);
                prop_assert!(g < 1e-7 || g > 360.0 - 1e-7, "lon {lon} -> {}", back.longitude.degrees());
                prop_assert!((back.latitude.degrees() - lat).abs() < 1e-7);
                prop_assert!((back.distance_au.unwrap() - dist).abs() < 1e-7 * dist);
            }

            #[test]
            fn spherical_cartesian_state_roundtrips(
                lon in 0.0f64..std::f64::consts::TAU,
                lat in -1.4f64..1.4,     // radians, away from ±π/2
                dist in 0.1f64..100.0,
                dlon in -0.1f64..0.1,
                dlat in -0.1f64..0.1,
                ddist in -0.1f64..0.1,
            ) {
                let s = SphericalState {
                    lon_rad: lon, lat_rad: lat, dist_au: dist,
                    lon_rate_rad_per_day: dlon, lat_rate_rad_per_day: dlat, dist_rate_au_per_day: ddist,
                };
                let back = cartesian_state_to_spherical(spherical_state_to_cartesian(s));
                prop_assert!((back.lon_rad - lon).abs() < 1e-9);
                prop_assert!((back.lat_rad - lat).abs() < 1e-9);
                prop_assert!((back.dist_au - dist).abs() < 1e-9 * dist);
                prop_assert!((back.lon_rate_rad_per_day - dlon).abs() < 1e-9);
                prop_assert!((back.lat_rate_rad_per_day - dlat).abs() < 1e-9);
                prop_assert!((back.dist_rate_au_per_day - ddist).abs() < 1e-9);
            }

            #[test]
            fn helio_geo_inverse_via_sun(
                plon in 0.0f64..360.0, plat in -85.0f64..85.0, pdist in 0.5f64..50.0,
                slon in 0.0f64..360.0, slat in -85.0f64..85.0, sdist in 0.5f64..2.0,
            ) {
                let planet_geo = ec(plon, plat, pdist);
                let sun_geo = ec(slon, slat, sdist);
                let helio = heliocentric_from_geocentric(&planet_geo, &sun_geo).unwrap();
                let back = geocentric_from_heliocentric(&helio, &sun_geo).unwrap();
                let g = lon_gap(back.longitude.degrees(), plon);
                prop_assert!(g < 1e-6 || g > 360.0 - 1e-6, "plon {plon} -> {}", back.longitude.degrees());
                prop_assert!((back.latitude.degrees() - plat).abs() < 1e-6);
                prop_assert!((back.distance_au.unwrap() - pdist).abs() < 1e-6 * pdist);
            }
        }
    }
```

- [ ] **Step 2: Run the properties — verify they pass**

Run: `cargo nextest run -p pleiades-compression frame_recombine`
Expected: PASS (existing point tests + 3 new property tests). A shrunk counterexample here indicates a real round-trip defect — check it against `frame_recombine.rs` before touching the property.

- [ ] **Step 3: Run fmt + lint**

Run: `mise run fmt && mise run lint`
Expected: both clean.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-compression/src/frame_recombine.rs
git commit -m "test(compression): property tests for coordinate/state/geo-helio round-trips"
```

---

### Task 4: Property test for `pleiades-compression` codec re-encode stability

**Files:**
- Modify: `crates/pleiades-compression/src/tests.rs` (append a `proptest` module)

**Interfaces:**
- Consumes (in scope in `tests.rs` via its existing `use super::*` and codec imports): `CompressedArtifact::{new, encode, decode}`, `ArtifactHeader::new`, `BodyArtifact::new`, `Segment::new`, `PolynomialChannel::linear`, `ChannelKind::{Longitude, Latitude, DistanceAu}`, `CelestialBody`, `Instant`, `JulianDay`, `TimeScale`.

**Rationale (oracle choice):** `encode` quantizes channel coefficients (fixed-point via `scale_exponent`), so an exact-`f64` `decode(encode(a)) == a` is fragile. The robust invariant is **re-encode byte stability**: a decoded artifact re-encodes to identical bytes, because after one round-trip its values already sit at codec fixpoints. This holds regardless of quantization and needs no tolerance.

- [ ] **Step 1: Write the failing property**

Append to `crates/pleiades-compression/src/tests.rs`:

```rust
mod codec_properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn encode_decode_reencode_is_stable(
            start_day in 0.0f64..1.0e5,
            span in 0.1f64..1.0e4,
            // Bounded to realistic magnitudes so fixed-point encoding never overflows.
            lon0 in -360.0f64..360.0, lon1 in -360.0f64..360.0,
            lat0 in -90.0f64..90.0,   lat1 in -90.0f64..90.0,
            dist0 in -100.0f64..100.0, dist1 in -100.0f64..100.0,
        ) {
            let artifact = CompressedArtifact::new(
                ArtifactHeader::new("proptest", "reencode stability fixture"),
                vec![BodyArtifact::new(
                    CelestialBody::Sun,
                    vec![Segment::new(
                        Instant::new(JulianDay::from_days(start_day), TimeScale::Tt),
                        Instant::new(JulianDay::from_days(start_day + span), TimeScale::Tt),
                        vec![
                            PolynomialChannel::linear(ChannelKind::Longitude, 9, lon0, lon1),
                            PolynomialChannel::linear(ChannelKind::Latitude, 9, lat0, lat1),
                            PolynomialChannel::linear(ChannelKind::DistanceAu, 12, dist0, dist1),
                        ],
                    )],
                )],
            );

            let bytes = artifact.encode().expect("encode");
            let decoded = CompressedArtifact::decode(&bytes).expect("decode");
            // Re-encoding a decoded artifact reproduces identical bytes: the codec is a
            // stable projection, robust to any quantization applied on the first encode.
            let reencoded = decoded.encode().expect("re-encode");
            prop_assert_eq!(&reencoded, &bytes);
            // Decoding the re-encoded bytes yields a structurally identical artifact.
            let redecoded = CompressedArtifact::decode(&reencoded).expect("re-decode");
            prop_assert_eq!(redecoded, decoded);
        }
    }
}
```

- [ ] **Step 2: Run the property — verify it passes**

Run: `cargo nextest run -p pleiades-compression codec_properties`
Expected: PASS (1 test, 256 cases). If `encode` panics on an input, the generated ranges are too wide — narrow the offending coefficient range and note it in a code comment; do not widen tolerances (the oracle is exact byte equality).

- [ ] **Step 3: Run fmt + lint**

Run: `mise run fmt && mise run lint`
Expected: both clean.

- [ ] **Step 4: Commit**

```bash
git add crates/pleiades-compression/src/tests.rs
git commit -m "test(compression): property test for codec re-encode byte stability"
```

---

### Task 5: Acceptance verification + documentation pointer

**Files:**
- Modify: `spec/validation-and-testing.md` (one-line pointer)
- Verify: `.gitignore`, blocking-tier runtime

**Interfaces:**
- Consumes: the property tests from Tasks 2–4.

- [ ] **Step 1: Verify `proptest-regressions/` is not gitignored**

Run: `git check-ignore -v crates/pleiades-types/proptest-regressions 2>&1; echo "exit: $?"`
Expected: exit `1` with no matching rule printed (path is NOT ignored). If a rule matches, add `!proptest-regressions/` to `.gitignore` and re-verify. (As of writing, `.gitignore` has no proptest rule, so this should already pass.)

- [ ] **Step 2: Add the single-source docs pointer**

In `spec/validation-and-testing.md`, add one bullet to the list of validation forms (place it under the section that enumerates test types; keep it to one line, do not restate the individual properties):

```markdown
- **Property-based tests (proptest):** invariant oracles for pure-logic
  crates — angle normalization/idempotence and degree/radian round-trips in
  `pleiades-types`; coordinate, state, and codec re-encode round-trips in
  `pleiades-compression`. Run in the blocking tier via `mise run test`;
  discovered counterexamples are committed under each crate's
  `proptest-regressions/`.
```

- [ ] **Step 3: Measure the blocking-tier runtime delta**

Run: `cargo nextest run -p pleiades-types -p pleiades-compression`
Record the reported wall-clock. Expected: the added property tests contribute on the order of seconds (a handful of properties × 256 cases), leaving the ≤ 10-minute blocking budget untouched. Note the measured time in the commit message.

- [ ] **Step 4: Full blocking-tier green check**

Run: `mise run fmt && mise run lint && mise run test`
Expected: all pass, including the new property tests, within budget.

- [ ] **Step 5: Commit**

```bash
git add spec/validation-and-testing.md
git commit -m "docs(testing): note proptest invariant-oracle layer for types + compression"
```

---

## Self-Review

**Spec coverage:**
- Harness & infra (workspace dep, per-crate opt-in, layout, case budget, no CI wiring, regressions-not-ignored) → Task 1 + Task 5 Step 1.
- `pleiades-types` properties 1–6 → Task 2 (7 proptest fns; property 5 split into the two round-trip assertions, `from_degrees(d).degrees()==d` is exact by construction and covered implicitly by the radian round-trip using `from_degrees`).
- `pleiades-compression` properties 1–3 → Task 3; property 4 (encode/decode) → Task 4, realized as the quantization-robust re-encode-stability form the spec's fallback anticipated.
- Edge conditions (non-finite, poles, zero radius) → bounded in every strategy (finite ranges, latitude away from poles, distance `> 0.1`).
- Acceptance criteria 1–6 → Task 5 (fmt/lint/test green, runtime recorded, regressions committable, docs pointer, time/houses explicitly deferred in Global Constraints).

**Placeholder scan:** No TBD/TODO; every code step shows complete code; every run step shows the command and expected result.

**Type consistency:** Names match the surveyed source — `Angle::normalized_0_360/normalized_signed/from_degrees/from_radians/radians/degrees`, `Longitude::from_degrees/degrees`, the six `frame_recombine` free functions + `SphericalState` fields (`lon_rad`, `lat_rad`, `dist_au`, `lon_rate_rad_per_day`, `lat_rate_rad_per_day`, `dist_rate_au_per_day`), and `CompressedArtifact::new/encode/decode` + `ArtifactHeader::new` + `BodyArtifact::new` + `Segment::new` + `PolynomialChannel::linear` + `ChannelKind` variants, all as used in the existing fixtures.
