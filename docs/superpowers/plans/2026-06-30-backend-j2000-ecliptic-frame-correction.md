# Backend J2000 Ecliptic Frame Correction — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every first-party backend emit a frame-consistent **J2000 ecliptic** geocentric position (longitude **and** latitude) at the boundary. Confirmed root cause: `icrf_to_ecliptic` (`crates/pleiades-jpl/src/spk/chain.rs:148`) and `icrf_velocity_to_ecliptic` (`chain.rs:113`) rotate by `instant.mean_obliquity()` (of-date) instead of a fixed J2000 ε₀, so kernel-fit bodies (Sun, Moon, planets, Pluto) carry J2000 longitude but of-date latitude. That of-date latitude is baked into the committed `packaged-artifact.bin` and the de440 corpus CSVs, so the fix is code + a kernel-gated data regeneration, plus a permanent independent-source frame-correctness gate, the band-aid revert, the ELP boundary correction, a non-J2000 SPK cross-check, and tolerance/claims recalibration.

**Architecture:** ε₀ becomes a single shared `pub const OBLIQUITY_J2000_DEG` in `pleiades-types`; `pleiades-jpl`'s SPK reduction and `pleiades-apparent`'s precession both consume it. The packaged artifact and de440 corpus regenerate from `/workspace/.kernels/de440.bsp` so the accuracy truth-set moves into the corrected frame with the artifact. A new `pleiades-validate` gate cross-checks the regenerated packaged backend's RAW J2000 latitude against an **independent** J2000 source (VSOP87 for Sun+planets+Pluto, the JPL snapshot fixture for the Moon) at 1900/2100 — the keystone that distinguishes "correct J2000" from "self-consistently wrong". The `pleiades-apparent` band-aid is removed; ELP precesses its of-date series back to J2000 at the boundary; tolerances and J2000-boundary posture claims are recalibrated.

**Tech Stack:** Rust workspace (`crates/pleiades-*`), `cargo test`, TDD with FNV-1a-64 checksum-pinned goldens, kernel-gated (`PLEIADES_DE_KERNEL`) deterministic data regeneration.

**Spec:** docs/superpowers/specs/2026-06-30-backend-j2000-ecliptic-frame-correction-design.md

## Global Constraints
- Branch: `feat/equatorial-declination-output` (sequenced before the equatorial RA/Dec feature's remaining Tasks 4–6, which live in a separate plan — this plan covers **B1–B7 only**).
- `ARTIFACT_VERSION` stays **7** (`crates/pleiades-compression/src/lib.rs:67`) — binary format unchanged; only latitude-channel **values** move on regeneration.
- ε₀ = `23.439_291_111_111_11` degrees is the **single source of truth**: one `pub const OBLIQUITY_J2000_DEG` in `pleiades-types`, consumed by `chain.rs`, `precession.rs`, and `Instant::mean_obliquity` (T=0 term).
- Data regeneration is **`PLEIADES_DE_KERNEL`-gated** against `/workspace/.kernels/de440.bsp` and **deterministic / byte-identity-gated** (`crates/pleiades-data/tests/artifact_regen.rs`, `crates/pleiades-jpl/tests/corpus_regen.rs`).
- The trusted "J2000-frame-correct" reference is always **INDEPENDENT** (VSOP87 / JPL snapshot fixture), **never** the regenerated corpus (which shares the fixed code and is self-consistent by construction).
- All longitude/RA residuals are cos(Dec)-weighted and wrap-aware; latitude/Dec residuals are signed.
- TDD: one test-first cycle per task; commit per task.

---

### Task B1: ε₀ fix in the SPK reduction + shared ε₀ constant

Change `icrf_to_ecliptic` and `icrf_velocity_to_ecliptic` to rotate by a fixed J2000 ε₀, sourced from a new `pub const` in `pleiades-types`, and refactor `precession.rs` to the same constant. This is the code half of the fix; it invalidates the committed bytes (B2 regenerates them). The new unit test is **kernel-free** so the non-gated suite stays green.

**Files:**
- Modify: `crates/pleiades-types/src/time.rs` (add `pub const OBLIQUITY_J2000_DEG`; use it in `mean_obliquity`, line ~340–348)
- Modify: `crates/pleiades-types/src/lib.rs` (re-export the const from the `time` module, line 49–51)
- Modify: `crates/pleiades-jpl/src/spk/chain.rs` (`icrf_to_ecliptic` line 148, `icrf_velocity_to_ecliptic` line 113; add the kernel-free unit test in the existing `#[cfg(test)] mod tests`, line 187)
- Modify: `crates/pleiades-apparent/src/precession.rs` (replace the private `const OBLIQUITY_J2000_DEG`, line 15, with the shared one)
- Test: `crates/pleiades-jpl/src/spk/chain.rs` (new `icrf_to_ecliptic_uses_fixed_j2000_obliquity`); existing `crates/pleiades-types/src/tests.rs:115` still asserts `23.439_291_111_111_11`.

**Interfaces:**
- Produces: `pleiades_types::OBLIQUITY_J2000_DEG: f64` (= `23.439_291_111_111_11`)
- Consumes: `pleiades_types::OBLIQUITY_J2000_DEG` in `chain.rs` and `precession.rs`
- Unchanged public signatures: `pub fn icrf_to_ecliptic(position_km: [f64; 3], instant: Instant) -> EclipticCoordinates`, `pub fn icrf_velocity_to_ecliptic(vel_km_s: [f64; 3], instant: Instant) -> [f64; 3]`

Steps:

1. - [ ] **Step 1: Write the failing kernel-free unit test in `chain.rs`.** Add to `mod tests` (after `obliquity_rotation_matches_existing_backend_constant`, ~line 233):
   ```rust
       #[test]
       fn icrf_to_ecliptic_uses_fixed_j2000_obliquity() {
           // At a far-from-J2000 epoch the of-date mean obliquity differs from
           // ε₀ by ~0.013° (~46″). Feed a pure +Z equatorial vector (the north
           // celestial pole direction): rotating about X by ε gives latitude
           // 90° − ε. With the J2000 fix this is 90 − 23.439291111 = 66.560708889°
           // for ANY epoch; the of-date bug would yield ~66.5479° at 1900.
           let inst_1900 = Instant::new(JulianDay::from_days(2_415_025.5), TimeScale::Tt);
           let ec = icrf_to_ecliptic([0.0, 0.0, AU_IN_KM], inst_1900);
           let expected = 90.0 - pleiades_types::OBLIQUITY_J2000_DEG;
           assert!(
               (ec.latitude.degrees() - expected).abs() < 1e-7,
               "lat {} should equal 90 − ε₀ = {} (epoch-independent), not the of-date value",
               ec.latitude.degrees(),
               expected
           );
           // Epoch independence: same vector at 2100 gives the same latitude.
           let inst_2100 = Instant::new(JulianDay::from_days(2_488_065.5), TimeScale::Tt);
           let ec2 = icrf_to_ecliptic([0.0, 0.0, AU_IN_KM], inst_2100);
           assert!((ec.latitude.degrees() - ec2.latitude.degrees()).abs() < 1e-12);
       }
   ```
2. - [ ] **Step 2: Run it — expect FAIL.** `cargo test -p pleiades-jpl icrf_to_ecliptic_uses_fixed_j2000_obliquity`. Expected: panic `lat 66.547... should equal 90 − ε₀ = 66.560708889` (the of-date code rotates by ~23.452° at 1900). Also `pleiades_types::OBLIQUITY_J2000_DEG` will not yet resolve — if that produces a compile error first, that is the expected red.
3. - [ ] **Step 3: Add the shared const in `pleiades-types/src/time.rs`.** Next to `pub const SECONDS_PER_DAY: f64 = 86_400.0;` (line 68):
   ```rust
   /// J2000.0 mean obliquity of the ecliptic, degrees (IAU 1976 constant term).
   /// Single source of truth shared by the SPK ICRF→ecliptic reduction
   /// (`pleiades-jpl`), the J2000→date precession (`pleiades-apparent`), and the
   /// constant term of [`Instant::mean_obliquity`].
   pub const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_111_11;
   ```
   Then in `mean_obliquity` (line 342–347) replace the literal constant term with the const:
   ```rust
       pub fn mean_obliquity(self) -> Angle {
           let t = (self.julian_day.days() - 2_451_545.0) / 36_525.0;
           Angle::from_degrees(
               OBLIQUITY_J2000_DEG
                   - 0.013_004_166_666_666_667 * t
                   - 0.000_000_163_888_888_888_888_88 * t * t
                   + 0.000_000_503_611_111_111_111_1 * t * t * t,
           )
       }
   ```
4. - [ ] **Step 4: Re-export the const.** In `crates/pleiades-types/src/lib.rs` line 49–51, add `OBLIQUITY_J2000_DEG` to the `pub use time::{…}` list:
   ```rust
   pub use time::{
       Instant, JulianDay, TimeScale, TimeScaleConversion, TimeScaleConversionError,
       OBLIQUITY_J2000_DEG, SECONDS_PER_DAY,
   };
   ```
5. - [ ] **Step 5: Fix `icrf_to_ecliptic` (chain.rs:148).** Replace the body's obliquity line and rename the now-unused parameter:
   ```rust
   /// Reduces an ICRF/J2000-equatorial geocentric position to **J2000** ecliptic
   /// coords, rotating about X by the fixed J2000 mean obliquity ε₀
   /// (`pleiades_types::OBLIQUITY_J2000_DEG`). The result is frame-consistent J2000
   /// in both longitude and latitude (independent of the epoch).
   pub fn icrf_to_ecliptic(position_km: [f64; 3], _instant: Instant) -> EclipticCoordinates {
       let eps = pleiades_types::OBLIQUITY_J2000_DEG.to_radians();
       let (x, y_eq, z_eq) = (position_km[0], position_km[1], position_km[2]);
       // Rotate about X by +ε₀: J2000-equatorial -> J2000-ecliptic.
       let y = y_eq * eps.cos() + z_eq * eps.sin();
       let z = -y_eq * eps.sin() + z_eq * eps.cos();
       let radius = (x * x + y * y + z * z).sqrt();
       let longitude = Longitude::from_degrees(y.atan2(x).to_degrees());
       let latitude = Latitude::from_degrees((z / radius).clamp(-1.0, 1.0).asin().to_degrees());
       EclipticCoordinates::new(longitude, latitude, Some(radius / AU_IN_KM))
   }
   ```
6. - [ ] **Step 6: Fix `icrf_velocity_to_ecliptic` (chain.rs:113).** Same ε₀ substitution and parameter rename:
   ```rust
   /// Rotates an ICRF/J2000-equatorial velocity vector (km/s) into the **J2000**
   /// ecliptic frame using the same fixed ε₀ rotation as `icrf_to_ecliptic`.
   /// Velocity is a free vector, so the rotation is identical (no translation term).
   pub fn icrf_velocity_to_ecliptic(vel_km_s: [f64; 3], _instant: Instant) -> [f64; 3] {
       let eps = pleiades_types::OBLIQUITY_J2000_DEG.to_radians();
       let (vx, vy_eq, vz_eq) = (vel_km_s[0], vel_km_s[1], vel_km_s[2]);
       // Rotate about X by +ε₀: J2000-equatorial -> J2000-ecliptic (same as position path).
       let vy = vy_eq * eps.cos() + vz_eq * eps.sin();
       let vz = -vy_eq * eps.sin() + vz_eq * eps.cos();
       [vx, vy, vz]
   }
   ```
   Note: `ecliptic_for_body` / `ecliptic_velocity_for_body` still pass `instant` positionally and still use it for `et_seconds_from_instant`; only the reduction helpers ignore it now.
7. - [ ] **Step 7: Point `precession.rs` at the shared const.** In `crates/pleiades-apparent/src/precession.rs`, delete the private `const OBLIQUITY_J2000_DEG: f64 = 23.439_291_111_111_11;` (line 15) and its doc comment, and add to the imports near line 9–10:
   ```rust
   use pleiades_types::OBLIQUITY_J2000_DEG;
   ```
   Leave both `eps0 = OBLIQUITY_J2000_DEG.to_radians()` uses (lines 45, 79) as-is — they now resolve to the imported const. (The line-81 band-aid stays for now; B4 removes it.)
8. - [ ] **Step 8: Run B1 tests — expect PASS.** `cargo test -p pleiades-types -p pleiades-jpl -p pleiades-apparent`. Expected: `icrf_to_ecliptic_uses_fixed_j2000_obliquity` passes; `instant_mean_obliquity_matches_the_shared_cubic_approximation` (types) still passes; `obliquity_rotation_matches_existing_backend_constant` (chain, J2000 epoch where ε_date≈ε₀) still passes; all `precession` tests still pass.
9. - [ ] **Step 9: Confirm the non-gated workspace suite is green except the data-identity gates.** `cargo build --workspace` then `cargo test -p pleiades-jpl -p pleiades-types -p pleiades-apparent`. The kernel-gated `artifact_regen` / `corpus_regen` tests are skipped without `PLEIADES_DE_KERNEL`, so they stay green here; the committed-bytes accuracy gate is addressed in B2.
10. - [ ] **Step 10: Commit.** `git add crates/pleiades-types/src/time.rs crates/pleiades-types/src/lib.rs crates/pleiades-jpl/src/spk/chain.rs crates/pleiades-apparent/src/precession.rs` then `git commit -m "fix(spk): reduce ICRF to J2000 ecliptic with fixed ε₀, not of-date obliquity (B1)"`.

---

### Task B2: Regenerate the packaged artifact + de440 corpus and re-pin goldens

Kernel-gated data build. With B1 in place, the committed `packaged-artifact.bin` and de440 corpus CSVs are stale (latitude channels still of-date). Regenerate both from de440 in one pass so the accuracy gate's truth-set moves with the artifact, overwrite the committed files, re-pin manifest checksums, and confirm the gated byte/value-identity tests pass. No writer-to-disk tooling exists for these (only the asteroid bin), so this task provides two throwaway example writers.

**Files:**
- Create (throwaway): `crates/pleiades-data/examples/regen-packaged-artifact.rs`
- Create (throwaway): `crates/pleiades-jpl/examples/regen-corpus.rs`
- Modify (regenerated bytes): `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`
- Modify (regenerated CSVs): `crates/pleiades-jpl/data/corpus/{boundary,interior,fast_clusters,holdout}.csv` (and `fixture_golden.csv` only if its bytes change — it is snapshot-derived and may not)
- Modify (re-pin checksums/rows): `crates/pleiades-jpl/data/corpus/manifest.txt` (lines 4–8)
- Verify (auto-recompute, no literal edit): `crates/pleiades-data/src/tests/codec.rs:145` (`assert_eq!(generated.checksum, fixture.checksum)`), `crates/pleiades-data/src/coverage/profile.rs:259` + `coverage/regen.rs:462` (`checksum: u64` populated from the regenerated artifact at runtime), `crates/pleiades-data/src/accuracy_baseline.rs:723` baseline summary golden
- Gated tests: `crates/pleiades-data/tests/artifact_regen.rs` (`regenerated_artifact_matches_committed`), `crates/pleiades-jpl/tests/corpus_regen.rs` (`regenerated_corpus_matches_checked_in`)

**Interfaces:**
- Consumes: `pleiades_data::regenerate_packaged_artifact_from_kernel(kernel_path: &str) -> Result<CompressedArtifact, String>` (`regenerate.rs:2561`); `CompressedArtifact::encode() -> Result<Vec<u8>, _>`
- Consumes: `pleiades_jpl::generate_slice(&SpkBackend, SliceRole) -> Result<GeneratedSlice, String>` with `GeneratedSlice { csv: String, .. }` (`generate.rs:158`); `SpkBackend::builder().add_kernel(path)?.build()`; `pleiades_jpl::spk::corpus_spec::SliceRole::{Boundary, InteriorBackbone, FastCluster, Holdout, FixtureGolden}`; `pleiades_jpl::spk::corpus_manifest::corpus_checksum64(&str) -> u64`
- Produces: overwritten committed data files + updated manifest.

Steps:

1. - [ ] **Step 1: Confirm the gated identity tests currently FAIL after B1.** With the kernel present: `PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo test -p pleiades-data --test artifact_regen regenerated_artifact_matches_committed`. Expected: FAIL — "re-encoded artifact is not byte-identical to the committed fixture" (B1 changed the latitude channels). This is the red proving regeneration is required.
2. - [ ] **Step 2: Add the throwaway artifact writer `crates/pleiades-data/examples/regen-packaged-artifact.rs`:**
   ```rust
   //! Throwaway maintainer writer: regenerate the packaged artifact from de440 and
   //! overwrite the committed fixture. Kernel-gated; delete after the data build.
   //!   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp \
   //!     cargo run -p pleiades-data --example regen-packaged-artifact
   fn main() -> Result<(), String> {
       let kernel = std::env::var("PLEIADES_DE_KERNEL")
           .map_err(|_| "set PLEIADES_DE_KERNEL to the de440.bsp path".to_string())?;
       let artifact = pleiades_data::regenerate_packaged_artifact_from_kernel(&kernel)?;
       let bytes = artifact.encode().map_err(|e| format!("encode: {e:?}"))?;
       let out = "crates/pleiades-data/tests/fixtures/packaged-artifact.bin";
       std::fs::write(out, &bytes).map_err(|e| format!("write {out}: {e}"))?;
       eprintln!("wrote {out}: {} bytes, checksum={}", bytes.len(), artifact.checksum);
       Ok(())
   }
   ```
3. - [ ] **Step 3: Add the throwaway corpus writer `crates/pleiades-jpl/examples/regen-corpus.rs`:**
   ```rust
   //! Throwaway maintainer writer: regenerate the de440 backbone corpus slices and
   //! overwrite the committed CSVs, printing the manifest line (rows + checksum) for
   //! each. Kernel-gated; delete after the data build.
   //!   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp \
   //!     cargo run -p pleiades-jpl --example regen-corpus
   use pleiades_jpl::spk::corpus_manifest::corpus_checksum64;
   use pleiades_jpl::spk::corpus_spec::SliceRole;
   use pleiades_jpl::{generate_slice, SpkBackend};

   fn main() -> Result<(), String> {
       let de = std::env::var("PLEIADES_DE_KERNEL")
           .map_err(|_| "set PLEIADES_DE_KERNEL to the de440.bsp path".to_string())?;
       let backend = SpkBackend::builder().add_kernel(&de).map_err(|e| e.message)?.build();
       // (role, file) for every de440 backbone slice that flows through icrf_to_ecliptic.
       let cases: [(SliceRole, &str, &str); 4] = [
           (SliceRole::Boundary, "boundary", "crates/pleiades-jpl/data/corpus/boundary.csv"),
           (SliceRole::InteriorBackbone, "interior", "crates/pleiades-jpl/data/corpus/interior.csv"),
           (SliceRole::FastCluster, "fast_cluster", "crates/pleiades-jpl/data/corpus/fast_clusters.csv"),
           (SliceRole::Holdout, "holdout", "crates/pleiades-jpl/data/corpus/holdout.csv"),
       ];
       for (role, name, path) in cases {
           let slice = generate_slice(&backend, role)?;
           let rows = slice.csv.lines().filter(|l| !l.starts_with('#') && !l.is_empty()).count();
           let checksum = corpus_checksum64(&slice.csv);
           std::fs::write(path, &slice.csv).map_err(|e| format!("write {path}: {e}"))?;
           println!("slice {name} file={name}.csv role={name} rows={rows} checksum={checksum}");
       }
       Ok(())
   }
   ```
   Note: `fast_clusters.csv` is the file name but the manifest role token is `fast_cluster` (matches `SliceRole::FastCluster.as_str()`); the printed line mirrors the existing manifest format. `fixture_golden` is **not** regenerated here — it is sourced from the independent Horizons snapshot (header: "independent of the de440 corpus") and does not pass through `icrf_to_ecliptic`; verify in Step 6 that its bytes are unchanged.
4. - [ ] **Step 4: Regenerate and overwrite the committed data.** Run both writers from the workspace root:
   ```
   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo run -p pleiades-data --example regen-packaged-artifact
   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo run -p pleiades-jpl --example regen-corpus
   ```
   Capture the four printed `slice … rows=… checksum=…` lines. Expected: `packaged-artifact.bin` and the four corpus CSVs are overwritten (`git status` shows them modified). Longitude/distance bytes are essentially unchanged; latitude channels move by ~45″ at the epoch extremes.
5. - [ ] **Step 5: Re-pin `manifest.txt`.** In `crates/pleiades-jpl/data/corpus/manifest.txt`, replace the `checksum=` (and `rows=` if changed — row counts should NOT change, only values) on lines 4–7 (`boundary`, `interior`, `fast_cluster`, `holdout`) with the printed values from Step 4. Leave line 8 (`fixture_golden`) untouched unless Step 6 shows its bytes changed.
6. - [ ] **Step 6: Confirm `fixture_golden.csv` did not move.** `git diff --stat crates/pleiades-jpl/data/corpus/fixture_golden.csv` — expect NO change (it is snapshot-derived J2000). If it did change, regenerate it too (`generate_slice(&backend, SliceRole::FixtureGolden)`), overwrite the file, and re-pin its line-8 checksum; otherwise leave it.
7. - [ ] **Step 7: Run the gated identity tests — expect PASS.**
   ```
   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo test -p pleiades-data --test artifact_regen regenerated_artifact_matches_committed
   PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo test -p pleiades-jpl --test corpus_regen regenerated_corpus_matches_checked_in
   ```
   Expected: artifact "PASS — byte-identical (N bytes)"; corpus "row count" and per-value `< 1.0 km` checks pass for all four roles (the committed CSVs now equal a fresh regen).
8. - [ ] **Step 8: Run the non-gated accuracy + codec gates against the regenerated data — expect PASS.** `cargo test -p pleiades-data`. Expected: `codec.rs` `generated.checksum == fixture.checksum` passes (both recomputed from the new bytes); `accuracy_baseline` `all_channels_within_published_ceilings_for_major_bodies` passes (artifact and holdout now share the corrected J2000 frame, so `max_latitude_arcsec` stays sub-1″, not ~45″); `packaged_artifact_baseline_summary_matches_committed_golden` passes (its anchors are on `max_lon`/speeds, which are unchanged — if any `max_lon` bucket drifts, re-pin the matching `report.contains(...)` string in `accuracy_baseline.rs:733–772` from the printed summary via the `#[ignore]` helper `print_packaged_artifact_baseline_summary`).
9. - [ ] **Step 9: Delete the throwaway writers and commit.** `git rm crates/pleiades-data/examples/regen-packaged-artifact.rs crates/pleiades-jpl/examples/regen-corpus.rs`, then `git add crates/pleiades-data/tests/fixtures/packaged-artifact.bin crates/pleiades-jpl/data/corpus/ ` (plus any re-pinned `accuracy_baseline.rs`) and `git commit -m "data(regen): rebuild packaged artifact + de440 corpus on J2000 ecliptic frame; re-pin manifest (B2)"`.

---

### Task B3: Keystone independent frame-correctness gate (NEW)

A permanent regression guard that cross-checks the regenerated packaged backend's **raw** J2000 ecliptic latitude (via `PackagedDataBackend::position` directly — no chart/apparent pipeline) against an **independent** J2000 source at far epochs JD 2415025.5 (1900) and 2488065.5 (2100): VSOP87 for Sun + planets + Pluto (VSOP87 covers the Sun natively), the JPL snapshot fixture for the Moon. The discriminator: the Sun's raw J2000 latitude at 1900 is ≈ −45″ (NOT ~0); a regression to the of-date frame would make it ~0 and trip the gate. Not kernel-gated — it reads the committed (regenerated) artifact.

**Files:**
- Create: `crates/pleiades-validate/src/frame_consistency_validation.rs`
- Modify: `crates/pleiades-validate/src/lib.rs` (declare `mod frame_consistency_validation;` near line 24 and `pub use frame_consistency_validation::{…}` near line 186)
- Test: inline `#[cfg(test)] mod tests` in the new file (modeled on `lilith_validation.rs:284`)

**Interfaces:**
- Produces: `pub fn validate_frame_consistency() -> Result<FrameConsistencyReport, FrameConsistencyError>`; `pub struct FrameConsistencyReport { pub rows_validated: usize, pub max_residual_lat_arcsec: f64, summary_line: String }` with `pub fn summary_line(&self) -> &str`; `pub enum FrameConsistencyError { ReferenceUnavailable {…}, PackagedUnavailable {…}, MissingEcliptic {…}, SentinelTooSmall {…}, ToleranceExceeded {…} }`
- Consumes: `pleiades_data::PackagedDataBackend`, `pleiades_vsop87::Vsop87Backend`, `pleiades_jpl::JplSnapshotBackend`, all via `pleiades_backend::{EphemerisBackend, EphemerisRequest}`; `pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale}`

Steps:

1. - [ ] **Step 1: Scaffold the gate file with a failing test.** Create `crates/pleiades-validate/src/frame_consistency_validation.rs` with the full module below (report/error structs modeled on `lilith_validation.rs`). The reference for each body is independent of the packaged corpus; latitude residuals are signed arcsec; a sentinel asserts the Sun's 1900 J2000 latitude is genuinely non-zero.
   ```rust
   //! Keystone fail-closed gate: the regenerated packaged backend's RAW J2000
   //! ecliptic latitude (no chart/apparent pipeline) must match an INDEPENDENT
   //! J2000 source at far epochs (1900/2100, where the of-date vs J2000 obliquity
   //! gap is ~46″ and so discriminates the frame): VSOP87 for Sun + planets +
   //! Pluto, the JPL snapshot fixture for the Moon. Distinguishes "correct J2000"
   //! from "self-consistently wrong"; it is non-negotiable and not kernel-gated.
   #![forbid(unsafe_code)]

   use core::fmt;

   use pleiades_backend::{EphemerisBackend, EphemerisRequest};
   use pleiades_data::PackagedDataBackend;
   use pleiades_jpl::JplSnapshotBackend;
   use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
   use pleiades_vsop87::Vsop87Backend;

   // Far epochs where ε_date − ε₀ ≈ ±46″ makes an of-date latitude error visible.
   const EPOCH_1900_JD_TT: f64 = 2_415_025.5;
   const EPOCH_2100_JD_TT: f64 = 2_488_065.5;

   // Tight ceiling on the packaged-vs-independent J2000 latitude residual. The
   // residual is the genuine DE440-vs-(VSOP87/snapshot) model difference once both
   // sides are J2000; the ~46″ of-date frame error is far above this. Generous
   // placeholder — Step 6 re-pins it from the printed `max_residual_lat_arcsec`.
   const FRAME_LAT_TOLERANCE_ARCSEC: f64 = 15.0;

   // Sentinel: the Sun's J2000 latitude at 1900 is ~ −46″·sin λ ≈ −45″. Require it
   // to exceed this floor in magnitude so a silent revert to the of-date frame
   // (Sun lat ≈ 0) fails the gate even if a future reference drift loosens the
   // tolerance above.
   const SUN_1900_LAT_SENTINEL_ARCSEC: f64 = 30.0;

   #[derive(Clone, Debug, PartialEq)]
   pub struct FrameConsistencyReport {
       pub rows_validated: usize,
       pub max_residual_lat_arcsec: f64,
       summary_line: String,
   }

   impl FrameConsistencyReport {
       pub fn summary_line(&self) -> &str {
           &self.summary_line
       }
   }

   impl fmt::Display for FrameConsistencyReport {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           f.write_str(&self.summary_line)
       }
   }

   #[derive(Clone, Debug, PartialEq)]
   pub enum FrameConsistencyError {
       PackagedUnavailable { body: String, jd_tt: f64, message: String },
       ReferenceUnavailable { body: String, jd_tt: f64, source: &'static str, message: String },
       MissingEcliptic { body: String, jd_tt: f64, which: &'static str },
       SentinelTooSmall { jd_tt: f64, got_arcsec: f64, floor_arcsec: f64 },
       ToleranceExceeded {
           body: String,
           jd_tt: f64,
           packaged_lat_deg: f64,
           reference_lat_deg: f64,
           residual_arcsec: f64,
           tolerance_arcsec: f64,
       },
   }

   impl fmt::Display for FrameConsistencyError {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           match self {
               Self::PackagedUnavailable { body, jd_tt, message } => write!(
                   f, "packaged backend unavailable for {body} @ JD {jd_tt}: {message}"
               ),
               Self::ReferenceUnavailable { body, jd_tt, source, message } => write!(
                   f, "independent reference ({source}) unavailable for {body} @ JD {jd_tt}: {message}"
               ),
               Self::MissingEcliptic { body, jd_tt, which } => write!(
                   f, "{which} ecliptic latitude absent for {body} @ JD {jd_tt}"
               ),
               Self::SentinelTooSmall { jd_tt, got_arcsec, floor_arcsec } => write!(
                   f, "frame sentinel failed: Sun J2000 latitude @ JD {jd_tt} is {got_arcsec:.2}\u{2033} \
                       (|·| must exceed {floor_arcsec:.1}\u{2033}); the backend looks of-date, not J2000"
               ),
               Self::ToleranceExceeded { body, jd_tt, packaged_lat_deg, reference_lat_deg, residual_arcsec, tolerance_arcsec } => write!(
                   f, "frame latitude mismatch for {body} @ JD {jd_tt}: packaged {packaged_lat_deg:.7}\u{00b0} \
                       vs independent J2000 {reference_lat_deg:.7}\u{00b0}, residual {residual_arcsec:.2}\u{2033} > tol {tolerance_arcsec:.1}\u{2033}"
               ),
           }
       }
   }

   impl std::error::Error for FrameConsistencyError {}

   #[derive(Clone, Copy)]
   enum Reference {
       Vsop87,
       Snapshot,
   }

   fn raw_lat_deg(
       backend: &dyn EphemerisBackend,
       body: &CelestialBody,
       instant: Instant,
   ) -> Result<f64, String> {
       let result = backend
           .position(&EphemerisRequest::new(body.clone(), instant))
           .map_err(|e| e.to_string())?;
       result
           .ecliptic
           .map(|ec| ec.latitude.degrees())
           .ok_or_else(|| "no ecliptic channel".to_string())
   }

   pub fn validate_frame_consistency() -> Result<FrameConsistencyReport, FrameConsistencyError> {
       let packaged = PackagedDataBackend::new();
       let vsop = Vsop87Backend::new();
       let snapshot = JplSnapshotBackend::new();

       // (body, reference source). VSOP87 covers the Sun natively and all 8
       // planets; the snapshot fixture covers the Moon in J2000.
       // RECONCILE BEFORE RUNNING: VSOP87 classically has NO Pluto, and the
       // snapshot fixture's body/epoch coverage is finite. The gate fails closed
       // (ReferenceUnavailable) for any body a reference can't supply — so verify
       // each (body, Reference) pair against the real backend coverage and, for
       // any uncovered body (likely Pluto), switch its reference to the snapshot
       // fixture if it covers it at 1900/2100, else drop that body with a
       // one-line note (Sun + planets + Moon already prove the frame).
       let bodies: [(CelestialBody, Reference); 10] = [
           (CelestialBody::Sun, Reference::Vsop87),
           (CelestialBody::Moon, Reference::Snapshot),
           (CelestialBody::Mercury, Reference::Vsop87),
           (CelestialBody::Venus, Reference::Vsop87),
           (CelestialBody::Mars, Reference::Vsop87),
           (CelestialBody::Jupiter, Reference::Vsop87),
           (CelestialBody::Saturn, Reference::Vsop87),
           (CelestialBody::Uranus, Reference::Vsop87),
           (CelestialBody::Neptune, Reference::Vsop87),
           (CelestialBody::Pluto, Reference::Vsop87),
       ];

       let mut rows_validated = 0usize;
       let mut max_residual_lat_arcsec = 0.0_f64;

       for jd_tt in [EPOCH_1900_JD_TT, EPOCH_2100_JD_TT] {
           let instant = Instant::new(JulianDay::from_days(jd_tt), TimeScale::Tt);
           for (body, reference) in &bodies {
               let body_label = format!("{body:?}");
               let packaged_lat = raw_lat_deg(&packaged, body, instant).map_err(|message| {
                   FrameConsistencyError::PackagedUnavailable { body: body_label.clone(), jd_tt, message }
               })?;

               // Sentinel: Sun's J2000 latitude at 1900 must be genuinely non-zero.
               if *body == CelestialBody::Sun && jd_tt == EPOCH_1900_JD_TT {
                   let got = (packaged_lat * 3600.0).abs();
                   if got < SUN_1900_LAT_SENTINEL_ARCSEC {
                       return Err(FrameConsistencyError::SentinelTooSmall {
                           jd_tt, got_arcsec: packaged_lat * 3600.0, floor_arcsec: SUN_1900_LAT_SENTINEL_ARCSEC,
                       });
                   }
               }

               let (source, reference_lat) = match reference {
                   Reference::Vsop87 => (
                       "vsop87",
                       raw_lat_deg(&vsop, body, instant).map_err(|message| {
                           FrameConsistencyError::ReferenceUnavailable {
                               body: body_label.clone(), jd_tt, source: "vsop87", message,
                           }
                       })?,
                   ),
                   Reference::Snapshot => (
                       "jpl-snapshot",
                       raw_lat_deg(&snapshot, body, instant).map_err(|message| {
                           FrameConsistencyError::ReferenceUnavailable {
                               body: body_label.clone(), jd_tt, source: "jpl-snapshot", message,
                           }
                       })?,
                   ),
                   _ => unreachable!(),
               };
               let _ = source;

               let residual = ((packaged_lat - reference_lat) * 3600.0).abs();
               if residual > FRAME_LAT_TOLERANCE_ARCSEC {
                   return Err(FrameConsistencyError::ToleranceExceeded {
                       body: body_label,
                       jd_tt,
                       packaged_lat_deg: packaged_lat,
                       reference_lat_deg: reference_lat,
                       residual_arcsec: residual,
                       tolerance_arcsec: FRAME_LAT_TOLERANCE_ARCSEC,
                   });
               }
               max_residual_lat_arcsec = max_residual_lat_arcsec.max(residual);
               rows_validated += 1;
           }
       }

       let summary_line = format!(
           "Frame-consistency gate: {rows_validated} rows validated (packaged raw J2000 latitude vs VSOP87/snapshot at 1900/2100), max lat residual {max_residual_lat_arcsec:.2}\u{2033}"
       );
       Ok(FrameConsistencyReport { rows_validated, max_residual_lat_arcsec, summary_line })
   }

   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn frame_consistency_gate_passes() {
           let report = validate_frame_consistency().expect("frame-consistency gate passes");
           assert!(report.rows_validated >= 18, "too few rows: {}", report.rows_validated);
           // Print the measured maximum so FRAME_LAT_TOLERANCE_ARCSEC can be tightened.
           eprintln!("{}", report.summary_line());
       }
   }
   ```
   The `enum Reference` has only two real arms; the trailing `_ => unreachable!()` is defensive against future additions. (If the linter flags it, drop the `_` arm and the `let _ = source;`.)
2. - [ ] **Step 2: Declare and export the module in `lib.rs`.** Add `mod frame_consistency_validation;` alphabetically near line 24 (after `mod equatorial_validation;`) and, near line 186 (after the `equatorial_validation` re-export):
   ```rust
   pub use frame_consistency_validation::{
       validate_frame_consistency, FrameConsistencyError, FrameConsistencyReport,
   };
   ```
3. - [ ] **Step 3: Run the gate against the regenerated artifact — observe the measured residual.** `cargo test -p pleiades-validate frame_consistency_gate_passes -- --nocapture`. Expected: PASS (B2 already moved the artifact to J2000), with the eprintln printing e.g. `max lat residual X.XX″`. If it FAILS on `SentinelTooSmall`, B2's regeneration did not land — stop and fix B2. If it fails `ToleranceExceeded` with a residual a few arcsec above 15″ for a planet, that is a genuine VSOP87-vs-DE440 model gap — record it; Step 6 re-pins the tolerance.
4. - [ ] **Step 4: Verify the gate is a true discriminator (manual red check).** Temporarily revert `icrf_to_ecliptic` to `instant.mean_obliquity()` in a stash (do NOT commit), rerun the test, and confirm it FAILS on the Sun 1900 `SentinelTooSmall` / `ToleranceExceeded` (~45″). Restore the J2000 code (`git checkout crates/pleiades-jpl/src/spk/chain.rs` or pop the stash). This proves the gate catches the bug.
5. - [ ] **Step 5: Confirm coverage of the printed max.** From Step 3's printed `max_residual_lat_arcsec`, set `FRAME_LAT_TOLERANCE_ARCSEC` to `ceil(measured_max) + small headroom` while keeping it well under the ~46″ frame-error signal (e.g. measured 3.2″ → pin to 6.0″). The sentinel stays at 30″ regardless.
6. - [ ] **Step 6: Re-pin the tolerance and re-run — expect PASS.** Edit `FRAME_LAT_TOLERANCE_ARCSEC` to the Step 5 value, then `cargo test -p pleiades-validate frame_consistency_gate_passes`. Expected: PASS with the tightened ceiling.
7. - [ ] **Step 7: Commit.** `git add crates/pleiades-validate/src/frame_consistency_validation.rs crates/pleiades-validate/src/lib.rs` then `git commit -m "test(validate): keystone J2000 frame-consistency gate vs VSOP87/snapshot at 1900/2100 (B3)"`.

---

### Task B4: Revert the precession band-aid

Remove the `lat_corrected = lat + sin(λ)·Δε` term that was canceling the backend's of-date tilt at the output. With a true-J2000 backend latitude (B1+B2) the rigorous 3-step precession is already correct; keeping the term would double-correct. End-to-end proof: the equatorial-goldens (JPL Horizons) gate, which passed *with* the band-aid, must still pass *without* it.

**Files:**
- Modify: `crates/pleiades-apparent/src/precession.rs` (remove lines 71–81 band-aid + misleading comment; restore `latitude_deg = lat.to_degrees();`)
- Test: `crates/pleiades-apparent/src/precession.rs` `#[cfg(test)] mod tests` (existing tests stay green); `crates/pleiades-validate/src/equatorial_validation.rs` `equatorial_goldens_pass`

**Interfaces:**
- Unchanged: `pub fn precess_ecliptic_j2000_to_date(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> Result<PrecessedEcliptic, ApparentPlaceError>`

Steps:

1. - [ ] **Step 1: Confirm the equatorial gate is green before the revert (baseline).** `cargo test -p pleiades-validate equatorial_goldens_pass -- --nocapture`. Expected: PASS against the regenerated artifact; note the printed `max Dec` for comparison.
2. - [ ] **Step 2: Remove the band-aid.** In `precess_ecliptic_j2000_to_date`, delete the block at lines 71–81 (the comment paragraph plus `let eps0 = …; let d_eps = …; let lat_corrected = lat + lon.sin() * d_eps;`) and change line 84 from `let latitude_deg = lat_corrected.to_degrees();` to:
   ```rust
       let longitude_deg = lon.to_degrees().rem_euclid(360.0);
       let latitude_deg = lat.to_degrees();
   ```
   The outbound `lat` (Meeus 13.2, using ε_date) is now the correct ecliptic-of-date latitude because the inbound `beta_deg` is a genuine J2000 latitude. Update the function's header doc (lines 1–7) only if it references the spurious-latitude correction — keep the rigorous 3-step description.
3. - [ ] **Step 3: Run the precession unit tests — expect PASS.** `cargo test -p pleiades-apparent precession`. Expected: `identity_at_j2000`, `general_precession_one_century` (β' < 2e-3°, the real ~4.4″ ecliptic-plane precession), and `longitude_shifts_by_precession_off_the_ecliptic` all pass unchanged (none depended on the band-aid term — they use β=0 or β=30 where the artificial correction was small but the rigorous result already satisfies the bounds).
4. - [ ] **Step 4: Run the equatorial Horizons gate without the band-aid — expect PASS.** `cargo test -p pleiades-validate equatorial_goldens_pass -- --nocapture`. Expected: PASS, including the Sun 6″ Dec rows. If `max Dec` shifts versus Step 1, that is expected and should be equal-or-better (the corrected latitude now flows straight through); only a tolerance breach is a failure (handled in B7 if a re-pin is warranted).
5. - [ ] **Step 5: Run the apparent + lilith gates that consume precession — expect PASS.** `cargo test -p pleiades-apparent -p pleiades-validate apparent_goldens equatorial_goldens lilith_gate`. Expected: all pass.
6. - [ ] **Step 6: Commit.** `git add crates/pleiades-apparent/src/precession.rs` then `git commit -m "fix(apparent): drop the of-date latitude band-aid; backend now emits true J2000 (B4)"`.

---

### Task B5: ELP emits J2000 at the boundary (date→J2000 precession)

ELP is a Meeus Ch.47 *of-date* lunar series (consistent of-date in both components — a separate J2000-boundary violation that would double-precess any ELP-sourced Moon through the apparent pipeline). Add a date→J2000 precession of ELP's of-date lon/lat at the backend output. Reuse `pleiades-apparent`'s precession by adding the **inverse** helper `precess_ecliptic_date_to_j2000` (the algebraic inverse of `precess_ecliptic_j2000_to_date`) and a `pleiades-apparent` dependency on `pleiades-elp`. **Dependency decision:** `pleiades-elp` currently depends only on `pleiades-backend` + `pleiades-types`; `pleiades-apparent` depends only on `pleiades-types` (no `pleiades-elp`), so adding `pleiades-apparent` to `pleiades-elp` introduces no cycle.

**Files:**
- Modify: `crates/pleiades-apparent/src/precession.rs` (add `pub fn precess_ecliptic_date_to_j2000`)
- Modify: `crates/pleiades-apparent/src/lib.rs:26` (export the new fn)
- Modify: `crates/pleiades-elp/Cargo.toml` (add `pleiades-apparent = { workspace = true }`)
- Modify: `crates/pleiades-elp/src/backend.rs` (`moon_ecliptic_coordinates` precesses date→J2000; the equatorial derivation still uses `instant.mean_obliquity()` on the of-date lon/lat — see Step 5)
- Test: `crates/pleiades-apparent/src/precession.rs` (round-trip inverse test); `crates/pleiades-elp/src/` tests (ELP J2000 longitude at 1900); an end-to-end Moon-of-date pipeline assertion

**Interfaces:**
- Produces: `pub fn precess_ecliptic_date_to_j2000(lambda_deg: f64, beta_deg: f64, jd_tt: f64) -> Result<PrecessedEcliptic, ApparentPlaceError>` (longitude/latitude referred to the J2000 mean equinox/ecliptic)
- Consumes (ELP): `pleiades_apparent::precess_ecliptic_date_to_j2000`

Steps:

1. - [ ] **Step 1: Write the failing inverse round-trip test in `precession.rs`.** Add to `mod tests`:
   ```rust
       #[test]
       fn date_to_j2000_is_the_inverse_of_j2000_to_date() {
           // Round-trip a non-trivial point at a far epoch: J2000 -> date -> J2000.
           let jd = 2_415_025.5; // 1900
           let to_date = precess_ecliptic_j2000_to_date(123.456, 4.5, jd).unwrap();
           let back = precess_ecliptic_date_to_j2000(to_date.longitude_deg, to_date.latitude_deg, jd)
               .unwrap();
           assert!((back.longitude_deg - 123.456).abs() < 1e-6, "λ round-trip {}", back.longitude_deg);
           assert!((back.latitude_deg - 4.5).abs() < 1e-6, "β round-trip {}", back.latitude_deg);
       }
   ```
2. - [ ] **Step 2: Run it — expect FAIL (unresolved name).** `cargo test -p pleiades-apparent date_to_j2000_is_the_inverse`. Expected: compile error `cannot find function precess_ecliptic_date_to_j2000`.
3. - [ ] **Step 3: Implement the inverse helper.** Add to `precession.rs` (after `precess_ecliptic_j2000_to_date`). It mirrors the forward path but (a) converts ecliptic-of-date → equatorial-of-date with ε_date, (b) applies the inverse precession rotation (ζ→−z, z→−ζ, θ→−θ), and (c) converts equatorial-J2000 → ecliptic-J2000 with ε₀:
   ```rust
   /// Precesses geocentric ecliptic coordinates from the mean equinox/ecliptic of
   /// date `jd_tt` back to the J2000 mean equinox/ecliptic. Algebraic inverse of
   /// [`precess_ecliptic_j2000_to_date`] (round-trips to < 1e-6° over ±1 century).
   pub fn precess_ecliptic_date_to_j2000(
       lambda_deg: f64,
       beta_deg: f64,
       jd_tt: f64,
   ) -> Result<PrecessedEcliptic, ApparentPlaceError> {
       let t = julian_centuries(jd_tt);
       let zeta = (2306.2181 * t + 0.30188 * t * t + 0.017998 * t * t * t) / 3600.0;
       let z = (2306.2181 * t + 1.09468 * t * t + 0.018203 * t * t * t) / 3600.0;
       let theta = (2004.3109 * t - 0.42665 * t * t - 0.041833 * t * t * t) / 3600.0;

       // ecliptic (of date) -> equatorial (of date), Meeus 13.3/13.4, ε_date.
       let eps = mean_obliquity_degrees(jd_tt).to_radians();
       let lambda = lambda_deg.to_radians();
       let beta = beta_deg.to_radians();
       let alpha_d = (lambda.sin() * eps.cos() - beta.tan() * eps.sin()).atan2(lambda.cos());
       let delta_d = (beta.sin() * eps.cos() + beta.cos() * eps.sin() * lambda.sin())
           .clamp(-1.0, 1.0)
           .asin();

       // precess equatorial (of date) -> equatorial (J2000): inverse rotation,
       // ζ→−z, z→−ζ, θ→−θ (Meeus 21.4 reduction-to-J2000 form).
       let zeta_r = zeta.to_radians();
       let z_r = z.to_radians();
       let theta_r = theta.to_radians();
       let a = delta_d.cos() * (alpha_d - z_r).sin();
       let b = theta_r.cos() * delta_d.cos() * (alpha_d - z_r).cos() + theta_r.sin() * delta_d.sin();
       let c = -theta_r.sin() * delta_d.cos() * (alpha_d - z_r).cos() + theta_r.cos() * delta_d.sin();
       let alpha0 = a.atan2(b) - zeta_r;
       let delta0 = c.clamp(-1.0, 1.0).asin();

       // equatorial (J2000) -> ecliptic (J2000), Meeus 13.1/13.2, ε₀.
       let eps0 = OBLIQUITY_J2000_DEG.to_radians();
       let lon = (alpha0.sin() * eps0.cos() + delta0.tan() * eps0.sin()).atan2(alpha0.cos());
       let lat = (delta0.sin() * eps0.cos() - delta0.cos() * eps0.sin() * alpha0.sin())
           .clamp(-1.0, 1.0)
           .asin();

       let longitude_deg = lon.to_degrees().rem_euclid(360.0);
       let latitude_deg = lat.to_degrees();
       if !longitude_deg.is_finite() || !latitude_deg.is_finite() {
           return Err(ApparentPlaceError::NonFiniteCorrection { stage: "precession" });
       }
       Ok(PrecessedEcliptic { longitude_deg, latitude_deg })
   }
   ```
4. - [ ] **Step 4: Export it.** In `crates/pleiades-apparent/src/lib.rs:26`:
   ```rust
   pub use precession::{
       precess_ecliptic_date_to_j2000, precess_ecliptic_j2000_to_date, PrecessedEcliptic,
   };
   ```
   Run `cargo test -p pleiades-apparent date_to_j2000_is_the_inverse` — expect PASS.
5. - [ ] **Step 5: Add the dependency and precess ELP's Moon to J2000.** In `crates/pleiades-elp/Cargo.toml` `[dependencies]` add `pleiades-apparent = { workspace = true }`. In `crates/pleiades-elp/src/backend.rs`, change `moon_ecliptic_coordinates` (line 58) to precess the of-date series output back to J2000 so the **ecliptic boundary output** is J2000:
   ```rust
       fn moon_ecliptic_coordinates(days: f64) -> EclipticCoordinates {
           let jd_tt = crate::J2000 + days;
           let (longitude, latitude, distance_au) = crate::data::moonposition::position(jd_tt);
           // The Meeus Ch.47 series is referred to the mean equinox/ecliptic OF DATE.
           // Precess it back to J2000 so the backend emits a J2000 boundary frame,
           // consistent with every other first-party backend; the apparent pipeline
           // re-applies the forward J2000->date precession (no double-precession).
           let precessed = pleiades_apparent::precess_ecliptic_date_to_j2000(
               longitude.degrees(),
               latitude.degrees(),
               jd_tt,
           )
           .expect("ELP lunar lon/lat precess cleanly to J2000");
           EclipticCoordinates::new(
               Longitude::from_degrees(precessed.longitude_deg),
               Latitude::from_degrees(precessed.latitude_deg),
               Some(distance_au),
           )
       }
   ```
   Note the equatorial branch: `ecliptic_point_to_equatorial` (line 179) applies `instant.mean_obliquity()` to whatever lon/lat it is given. Since `moon_ecliptic_coordinates` now returns **J2000** lon/lat, calling `to_equatorial(instant.mean_obliquity())` on it would mix frames. In `position` (line 277–284), keep the J2000 ecliptic for `result.ecliptic`, but derive `result.equatorial` from the **of-date** series directly so the equatorial output stays mean-of-date (its prior, validated behavior). Add a private helper that returns the raw of-date coords for the equatorial derivation:
   ```rust
       fn moon_ecliptic_of_date(days: f64) -> EclipticCoordinates {
           let (longitude, latitude, distance_au) =
               crate::data::moonposition::position(crate::J2000 + days);
           EclipticCoordinates::new(longitude, latitude, Some(distance_au))
       }
   ```
   and in the `CelestialBody::Moon` arm:
   ```rust
           CelestialBody::Moon => {
               let coords = Self::moon_ecliptic_coordinates(days); // J2000 boundary
               result.ecliptic = Some(coords);
               let of_date = Self::moon_ecliptic_of_date(days);    // for mean-obliquity equatorial
               result.equatorial = Some(Self::ecliptic_point_to_equatorial(
                   of_date.longitude,
                   of_date.latitude,
                   req.instant,
                   of_date.distance_au,
               ));
           }
   ```
   The node/apogee/perigee channels have latitude 0 and longitude that is already a mean-of-date angle defined by their own series; leave them unchanged (they are longitude-only points, not part of this J2000-ecliptic-latitude correction). `motion` derives from `ecliptic_for_body`, which for the Moon calls `moon_ecliptic_coordinates`; the finite-difference rate is unaffected to the precision used (precession is smooth and near-linear across the ±0.5-day probe).
6. - [ ] **Step 6: Write the ELP J2000-longitude test.** Add an ELP test (in `crates/pleiades-elp/src/tests/` or the existing test module) asserting the Moon's boundary longitude at 1900 is the J2000 value (~273.81°), not the of-date 272.41°:
   ```rust
       #[test]
       fn moon_boundary_longitude_is_j2000_not_of_date() {
           use pleiades_backend::{EphemerisBackend, EphemerisRequest};
           use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
           let backend = crate::ElpBackend::new();
           let inst = Instant::new(JulianDay::from_days(2_415_025.5), TimeScale::Tt);
           let res = backend
               .position(&EphemerisRequest::new(CelestialBody::Moon, inst))
               .unwrap();
           let lon = res.ecliptic.unwrap().longitude.degrees();
           // J2000 longitude differs from the raw of-date series by ~+1.4° (precession).
           let of_date = crate::data::moonposition::position(2_415_025.5).0.degrees();
           assert!((lon - of_date).abs() > 1.0, "ELP Moon not precessed to J2000: {lon} vs of-date {of_date}");
       }
   ```
   Run `cargo test -p pleiades-elp moon_boundary_longitude_is_j2000_not_of_date` — first expect FAIL if written before Step 5 is applied, then PASS after.
7. - [ ] **Step 7: End-to-end Moon-of-date round-trip.** Add (or extend) an apparent-pipeline test asserting that an ELP-sourced Moon, after the pipeline's forward J2000→date precession, lands back on the of-date series (no double-precession) to < 0.001″. Reuse `pleiades_apparent::precess_ecliptic_j2000_to_date` on the ELP J2000 boundary output and compare to `moonposition::position` of-date:
   ```rust
       #[test]
       fn elp_moon_round_trips_to_of_date_through_the_pipeline() {
           let jd = 2_415_025.5;
           let days = jd - crate::J2000;
           let j2000 = crate::ElpBackend::moon_ecliptic_coordinates(days); // pub(crate) or via position()
           let redate = pleiades_apparent::precess_ecliptic_j2000_to_date(
               j2000.longitude.degrees(), j2000.latitude.degrees(), jd,
           ).unwrap();
           let (od_lon, od_lat, _) = crate::data::moonposition::position(jd);
           assert!((redate.longitude_deg - od_lon.degrees()).abs() * 3600.0 < 1e-3);
           assert!((redate.latitude_deg - od_lat.degrees()).abs() * 3600.0 < 1e-3);
       }
   ```
   (Expose `moon_ecliptic_coordinates` as `pub(crate)` if the test is in a sibling module, or drive it through `backend.position`.)
8. - [ ] **Step 8: Run the ELP + apparent suites — expect PASS.** `cargo test -p pleiades-elp -p pleiades-apparent`. Expected: new tests pass; existing ELP equatorial/validation tests (which use the of-date equatorial path preserved in Step 5) stay green. If any committed ELP frame-claim string asserts "produced directly from the truncated lunar series", revisit it in B7 (claims), not here.
9. - [ ] **Step 9: Commit.** `git add crates/pleiades-apparent/src/precession.rs crates/pleiades-apparent/src/lib.rs crates/pleiades-elp/Cargo.toml crates/pleiades-elp/src/` then `git commit -m "feat(elp): emit J2000 ecliptic at the boundary via date->J2000 precession (B5)"`.

---

### Task B6: Non-J2000 SPK ↔ snapshot ecliptic cross-check

The existing SPK cross-check (`spk_reduction_matches_snapshot_entry_ecliptic`, `cross_check_tests.rs`) only runs at J2000, where ε_date ≈ ε₀, so it is frame-blind. Add a non-J2000-epoch cross-check that locks the live `SpkBackend` reduction to the fixed-ε₀ (J2000) frame — i.e. it must match the ε₀ rotation and would NOT match the of-date rotation at a far epoch. The existing test is kernel-free (synthetic DAF), so this one is too; an optional kernel-gated live cross-check mirrors the `PLEIADES_DE_KERNEL` skip pattern.

**Files:**
- Modify: `crates/pleiades-jpl/src/spk/cross_check_tests.rs` (add a non-J2000-epoch test)

**Interfaces:**
- Consumes: `SpkBackend::builder().add_kernel_bytes(blob, "x")?.build()`, `EphemerisBackend::position`, `pleiades_types::OBLIQUITY_J2000_DEG`

Steps:

1. - [ ] **Step 1: Write the failing-then-passing non-J2000 cross-check.** Add to `cross_check_tests.rs`:
   ```rust
   #[test]
   fn spk_reduction_is_j2000_frame_at_non_j2000_epoch() {
       // Same synthetic ICRF geocentric vector as the J2000 test, but evaluated at
       // 1900 where ε_date − ε₀ ≈ 46″. The reduction must use the FIXED ε₀ (J2000),
       // so the ecliptic latitude equals the ε₀ rotation and is epoch-independent.
       let body_icrf = [1.2e8, -3.4e7, 5.6e6];
       let blob = build_daf(&[
           const_seg(10, 0, body_icrf),
           const_seg(399, 3, [0.0, 0.0, 0.0]),
           const_seg(3, 0, [0.0, 0.0, 0.0]),
       ]);
       let backend = SpkBackend::builder().add_kernel_bytes(blob, "x").unwrap().build();
       let inst_1900 = Instant::new(JulianDay::from_days(2_415_025.5), TimeScale::Tt);
       let ec = backend
           .position(&pleiades_backend::EphemerisRequest::new(CelestialBody::Sun, inst_1900))
           .unwrap()
           .ecliptic
           .unwrap();

       // Independent J2000 reference: rotate by ε₀, NOT by the of-date obliquity.
       let eps0 = pleiades_types::OBLIQUITY_J2000_DEG.to_radians();
       let (x, ye, ze) = (body_icrf[0], body_icrf[1], body_icrf[2]);
       let y = ye * eps0.cos() + ze * eps0.sin();
       let z = -ye * eps0.sin() + ze * eps0.cos();
       let r = (x * x + y * y + z * z).sqrt();
       let expect_lat = (z / r).asin().to_degrees();
       assert!(
           (ec.latitude.degrees() - expect_lat).abs() < 1e-9,
           "lat {} should equal the ε₀ rotation {} at 1900 (J2000 frame)",
           ec.latitude.degrees(),
           expect_lat
       );

       // And it must DIFFER from the of-date rotation by ~tens of arcsec — proving
       // the test is frame-discriminating, not vacuous.
       let eps_d = inst_1900.mean_obliquity().radians();
       let z_d = -ye * eps_d.sin() + ze * eps_d.cos();
       let of_date_lat = (z_d / (x * x + (ye * eps_d.cos() + ze * eps_d.sin()).powi(2) + z_d * z_d).sqrt())
           .asin()
           .to_degrees();
       assert!(
           (ec.latitude.degrees() - of_date_lat).abs() * 3600.0 > 1.0,
           "frame-blind: J2000 and of-date latitudes are too close to discriminate"
       );
   }
   ```
2. - [ ] **Step 2: Run it — expect PASS (post-B1).** `cargo test -p pleiades-jpl spk_reduction_is_j2000_frame_at_non_j2000_epoch`. Expected: PASS (B1 made the reduction ε₀-based). The second assertion confirms the test would catch an of-date regression.
3. - [ ] **Step 3 (optional, kernel-gated live cross-check):** If a live-SPK = snapshot lock is wanted beyond the synthetic vector, add a `PLEIADES_DE_KERNEL`-gated test that builds `SpkBackend` from de440, evaluates the Sun's ecliptic latitude at 1900, and compares it to `JplSnapshotBackend`'s Sun latitude at the same epoch within a tight tolerance — mirroring the skip pattern `let Ok(kernel) = std::env::var("PLEIADES_DE_KERNEL") else { eprintln!("skipping…"); return; };`. Keep the assertion latitude-only and tight (both are J2000 post-B1).
4. - [ ] **Step 4: Commit.** `git add crates/pleiades-jpl/src/spk/cross_check_tests.rs` then `git commit -m "test(spk): lock live SPK reduction to J2000 frame at a non-J2000 epoch (B6)"`.

---

### Task B7: Tolerance recalibration + J2000-boundary claims

Tighten the topocentric latitude tolerance (the stale 90″ "data ceiling" was absorbing this very bug) toward the true post-fix residual with documented headroom, correct its rationale, and re-pin its `GOLDENS_CHECKSUM`. Re-run the equatorial Sun-Dec 6″ gate and the lilith 80″ ceiling. Update the J2000-at-the-backend-boundary posture claims so they are now true and asserted — but leave the backend-boundary mean-obliquity **equatorial** strings unchanged (they remain true). Finish with the full workspace + release gate.

**Files:**
- Modify: `crates/pleiades-validate/data/topocentric-goldens.csv` (per-row `lat_tolerance_arcsec` 90 → measured + headroom; rationale comment lines ~8–21)
- Modify: `crates/pleiades-validate/src/topocentric_validation.rs:20` (`GOLDENS_CHECKSUM` re-pin)
- Re-run (re-pin only if a value drifts): `crates/pleiades-validate/src/equatorial_validation.rs` (Sun-Dec 6″), `crates/pleiades-validate/src/lilith_validation.rs:24` (80″ ceiling)
- Modify (claims): the J2000-boundary posture strings flagged in the spec/investigation §5 — `crates/pleiades-data/src/lookup.rs:405` (frame/storage posture) and any `packaged_frame_treatment`/J2000-boundary claim that is now provably true. Do **not** edit `pleiades-vsop87` `profiles.rs:279` (already correct) nor any mean-obliquity *equatorial* boundary string.
- Test: topocentric/equatorial/lilith gates; `cargo test --workspace`; release gate `run_all_numeric_gates`

**Interfaces:**
- Consumes: `pleiades_apparent::fnv1a64` (checksum re-pin), the three gate entry points (`validate_topocentric_goldens`, `validate_equatorial_goldens`, `validate_lilith_corpus`)

Steps:

1. - [ ] **Step 1: Measure the post-fix topocentric latitude residual.** Temporarily widen the rationale aside and run `cargo test -p pleiades-validate topocentric_goldens_pass -- --nocapture` (it passes at 90″). To read the actual max latitude residual, add a throwaway `eprintln!` of `lat_residual_arcsec` per row (or compute via a scratch test) — the spec expects ~10″. Record the measured maximum `M`.
2. - [ ] **Step 2: Set the new latitude tolerance.** In `topocentric-goldens.csv`, change every data row's `lat_tolerance_arcsec` from `90.0` to `ceil(M) + headroom` (spec target ~10 → e.g. `20.0`; keep round and comfortably above `M` but far below the old 90″). Update the rationale block (CSV lines ~8–21) to delete the "~44 arcsec accuracy limit / 2× the data ceiling" text (that ceiling was the frame bug) and state the new basis: "post-J2000-frame-fix latitude residual ≈ M″ (DE440-vs-Horizons model + interpolation); tolerance set to N″ with ~X× headroom."
3. - [ ] **Step 3: Run the topocentric gate — expect checksum FAIL first.** `cargo test -p pleiades-validate topocentric -- --nocapture`. Expected: `pinned_checksum` FAILS ("checksum = <new>") because the CSV changed; the body of `topocentric_goldens_pass` passes against the tighter tolerance.
4. - [ ] **Step 4: Re-pin `GOLDENS_CHECKSUM`.** Copy the printed value into `topocentric_validation.rs:20` (`const GOLDENS_CHECKSUM: u64 = <new>;`). Re-run `cargo test -p pleiades-validate topocentric` — expect PASS (both `pinned_checksum` and `topocentric_goldens_pass`).
5. - [ ] **Step 5: Re-run the equatorial Sun-Dec 6″ gate and lilith 80″ ceiling.** `cargo test -p pleiades-validate equatorial_goldens_pass lilith_gate_passes -- --nocapture`. Expected: both PASS against the regenerated artifact + reverted band-aid. The equatorial CSV is unchanged (Dec goldens are external Horizons values), so `equatorial_validation.rs:17` checksum stays; lilith's `LAT_CEILING_ARCSEC = 80.0` is a generous ceiling (measured 53″) and should still pass. Only re-pin a constant if a printed residual now exceeds it (it should not — corrected latitude is equal-or-better).
6. - [ ] **Step 6: Update the J2000-boundary posture claims.** Edit `crates/pleiades-data/src/lookup.rs:405` (and any flagged sibling posture string) so the now-true "J2000 at the backend boundary" contract is stated/asserted rather than silently violated. Keep "stores ecliptic coordinates directly" (still true) and do not touch mean-obliquity *equatorial* boundary strings. If a claims drift test (e.g. in `pleiades-validate` `claims` / compatibility-summary goldens) pins these strings, update the matching golden/checksum from the failing assertion's printed value.
7. - [ ] **Step 7: Full workspace test — expect PASS.** `cargo test --workspace`. Expected: green. Investigate any failure (per spec risk note: tightening surfaces latent issues — fix, do not re-loosen). The kernel-gated `artifact_regen`/`corpus_regen` tests are skipped here (no env var) and already confirmed in B2.
8. - [ ] **Step 8: Release numeric gate — expect PASS.** Run the workspace's release gate `run_all_numeric_gates` (e.g. `cargo test -p pleiades-validate run_all_numeric_gates` or the documented release-gate invocation) and confirm the new `validate_frame_consistency` keystone is included or invoked alongside it; if the release gate enumerates gates explicitly, add `validate_frame_consistency` to that list so the keystone runs in the release posture.
9. - [ ] **Step 9: Commit.** `git add crates/pleiades-validate/data/topocentric-goldens.csv crates/pleiades-validate/src/topocentric_validation.rs crates/pleiades-data/src/lookup.rs` (plus any re-pinned claims goldens) then `git commit -m "test+docs: tighten topocentric lat tolerance, correct rationale, assert J2000-boundary posture (B7)"`.

---

## Self-review against the spec

Acceptance criteria → task mapping (all covered, no placeholders):
1. ε₀ fix + kernel-free J2000-latitude unit test → **B1**.
2. Regenerated artifact + corpus committed; gated byte-identity tests pass; accuracy gate green → **B2**.
3. Keystone independent frame gate green (VSOP87 + snapshot at 1900/2100) → **B3**.
4. Band-aid removed; equatorial-goldens green end-to-end → **B4**.
5. ELP emits J2000 at the boundary; no double-precession → **B5**.
6. Live-SPK = snapshot / J2000 frame at a non-J2000 epoch → **B6**.
7. Tolerances recalibrated + claims re-pinned; `cargo test --workspace` + release gate green → **B7**.

Type/signature consistency verified across tasks: `OBLIQUITY_J2000_DEG` (B1 produces, B3/B5/B6 consume), `precess_ecliptic_date_to_j2000` (B5 produces/consumes), `validate_frame_consistency`/`FrameConsistencyReport`/`FrameConsistencyError` (B3), `EphemerisBackend::position` + `EphemerisRequest::new` usage matches the live API (`PackagedDataBackend::new`, `Vsop87Backend::new`, `JplSnapshotBackend::new` are all `const fn new()`). Epochs JD 2415025.5 / 2488065.5 used consistently in B1, B3, B6.
