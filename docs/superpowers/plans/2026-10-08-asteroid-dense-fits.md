# Dense Asteroid Fits Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `PackagedDataBackend` serves Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros` on every date in 1900–2100 from dense heliocentric fits to the JPL `sb441-n373s` kernel, gated against `asteroid_reference.csv`.

**Architecture:** Regeneration builds one `SpkBackend` from de440 + sb441-n373s and fits the five asteroids through the same dense path as the planets (`fit_segment_within_span`), stored `StoredFrame::Heliocentric`; lookup already recombines heliocentric segments with the Sun. The 17-row Eros re-fit and the `is_carried_but_unserved` predicate are deleted, and the bodies become release-grade with `CorpusValidated { source: "sb441-n373s" }` evidence.

**Tech Stack:** Rust workspace (`pleiades-data`, `pleiades-jpl`, `pleiades-cli`, `pleiades-validate`, `pleiades-core`, `pleiades-events`), cargo-nextest, mise tasks.

**Spec:** `docs/superpowers/specs/2026-10-08-asteroid-dense-fits-design.md`

## Global Constraints

- Window: 1900–2100 (`CoverageWindow::default()`); degree 8 and `FITTING_OVERSAMPLE` 3 for every body, unchanged.
- Body order after the ten base bodies, fixed: Ceres, Pallas, Juno, Vesta, `asteroid:433-Eros`.
- Span acceptance rule per asteroid: max geocentric error vs the two-kernel `SpkBackend`, sampled every 0.5 day over the window, ≤ 1″ in longitude (×cos lat) and ≤ 1″ in latitude. Chosen span = longest power-of-two span meeting the rule.
- Size: `PACKAGED_BUDGETS.max_encoded_bytes = 12_000_000` and `package-check`'s 9 MiB gzipped `.crate` cap are NOT raised in this change. If the rule-meeting spans do not fit, STOP and report to the maintainer (spec §2).
- The ten base bodies' segments must come out bit-identical to the committed artifact.
- Claims: asteroids `BodyClaim::release_grade(body, AccuracyClass::High, ClaimEvidence::CorpusValidated { source: "sb441-n373s".into() })`; base bodies keep `ArtifactValidated`.
- Gate ceiling per channel = measured maximum × 1.4, rounded up to 2 significant figures.
- Compatibility profile `0.7.33` → `0.7.34` with a release note.
- Kernels (not committed, gitignored): `PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp`, `PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp`. Both SHA-256-verified 2026-10-07 against `docs/spk-kernel-sourcing.md`.
- Out of scope: Apophis/Juno Horizons rows, the 1.0″/1.9″ Horizons-vs-sb441 difference, `JplSnapshotBackend` metadata, asteroid stations/aspects gating (#167 (d), #168 (f)), #160.
- No source edits or commits while a background test run is in progress (release-bundle tests compare git provenance).
- Run `cargo fmt --all` before every commit.

## Review Focus

1. **Eros near Earth** (close approaches ≈0.15 AU, e.g. 1931, 1975, 2012, 2056): geocentric error is magnified ~7×; the 0.5-day measurement must report its worst JD and it must sit inside 1″ — Task 2.
2. **A window-edge instant** (JD exactly 2415020.5 and the last instant of the window) for an asteroid: served, not `OutOfRangeInstant`; one step past the window: still refused — Task 4.
3. **Apparent chart of an asteroid** (light-time step reads the asteroid a few minutes earlier): now reduced (no "0 of 1 placements reduced"), since the fit is dense — Task 4.
4. **A caller-supplied artifact without asteroids** (`PackagedDataBackend::from_artifact` with the ten base bodies only): asking for Ceres is a clean `UnsupportedBody`, not a panic — Task 4.
5. **Apophis through the default chain**: still answered by `JplSnapshotBackend` at its rows and refused elsewhere; the packaged backend must not claim it — Task 4.

---

## File Map

- `crates/pleiades-data/src/regenerate.rs` — extract `fit_dense_body_artifact`; two-kernel entry points; delete the Eros snapshot branch.
- `crates/pleiades-data/src/coverage/generation_spec.rs` — per-asteroid `fitting_segment_span_days`.
- `crates/pleiades-data/src/lib.rs` — `PACKAGED_ASTEROIDS`, `packaged_bodies()`, source text, claims; delete `is_carried_but_unserved`.
- `crates/pleiades-data/src/backend.rs`, `crates/pleiades-data/src/lookup.rs` — remove predicate call sites.
- `crates/pleiades-data/src/tests/asteroid_fit.rs` (new) — kernel-gated measurement + base-body identity check.
- `crates/pleiades-data/src/tests/asteroid_gate.rs` (new) — blocking gate vs `asteroid_reference.csv`.
- `crates/pleiades-data/src/thresholds.rs` — `ASTEROID_CORPUS_CEILING`.
- `crates/pleiades-data/src/accuracy_baseline.rs` — remove Eros self-consistency.
- `crates/pleiades-data/tests/artifact_regen.rs` — two kernels.
- `crates/pleiades-data/tests/fixtures/packaged-artifact.bin` — regenerated.
- `crates/pleiades-cli/src/commands/{packaged_artifact.rs,generate_artifact.rs}`, `crates/pleiades-validate/src/render/cli.rs` — require `PLEIADES_AST_KERNEL`.
- Tests flipping from refusal to served: `crates/pleiades-cli/src/cli/tests/{asteroid_gate.rs,chart.rs}`, `crates/pleiades-data/src/tests/lookup.rs`, `crates/pleiades-events/tests/nod_aps.rs`, plus the "11 bundled bodies" pins.
- `crates/pleiades-validate/src/claims/audit.rs` — packaged asteroid block.
- `crates/pleiades-core/src/compatibility/mod.rs` — profile 0.7.34 + note.
- Docs: `docs/status.md`, `spec/data-compression.md`, `docs/cli.md`, `crates/pleiades-data/README.md`, `docs/spk-kernel-sourcing.md`, `crates/pleiades-validate/data/apparent-goldens.csv` (comment + re-pinned checksum).
- `docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md` (new) — measured spans, errors, sizes.

---

### Task 1: Extract the dense per-body fit (pure refactor)

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs` (major-body branch of `build_packaged_artifact_from_reference_over`, ~lines 2470–2495)

**Interfaces:**
- Produces: `pub(crate) fn fit_dense_body_artifact(body: &CelestialBody, window: (f64, f64), reference: &dyn EphemerisBackend) -> BodyArtifact`

- [ ] **Step 1: Confirm the committed artifact regenerates byte-identically today (baseline)**

Run (release; takes minutes):
```bash
PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp cargo test --release -p pleiades-data --test artifact_regen regenerated_artifact_matches_committed -- --nocapture
```
Expected: `artifact_regen: PASS — byte-identical (10491298 bytes)`. If it fails, STOP: the base-body identity guarantee cannot be checked and the maintainer must know.

- [ ] **Step 2: Extract the function**

Add above `build_packaged_artifact_from_reference_over`:
```rust
/// Fits `body` densely from `reference` over `window`, one segment per
/// [`fitting_segment_boundaries`] span, in the body's stored frame.
///
/// # Panics
///
/// When `reference` cannot serve `body` somewhere in `window`: regeneration
/// is a maintainer step whose kernels must cover the whole window.
pub(crate) fn fit_dense_body_artifact(
    body: &CelestialBody,
    window: (f64, f64),
    reference: &dyn EphemerisBackend,
) -> BodyArtifact {
    use crate::coverage::fitting_segment_boundaries;
    let (start_jd, end_jd) = window;
    let segments: Vec<Segment> = fitting_segment_boundaries(body, start_jd, end_jd)
        .into_iter()
        .map(|(t0, t1)| {
            fit_segment_within_span(body, t0, t1, reference).unwrap_or_else(|| {
                panic!("fit_segment_within_span failed for body {body} over [{t0}, {t1}]")
            })
        })
        .collect();
    let frame = if body_uses_heliocentric_frame(body) {
        pleiades_compression::StoredFrame::Heliocentric
    } else {
        pleiades_compression::StoredFrame::Geocentric
    };
    BodyArtifact::with_frame(body.clone(), segments, frame)
}
```
Replace the major-body branch body with:
```rust
_ => {
    // Major body: fit densely from the reference (de440) backend.
    handles.push(scope.spawn(move || {
        (body_index, fit_dense_body_artifact(&body, base_window, reference))
    }));
}
```
Remove the now-unused `use crate::coverage::fitting_segment_boundaries;` at the top of `build_packaged_artifact_from_reference_over` if the compiler flags it.

- [ ] **Step 3: Re-run the byte-identity test**

Same command as Step 1. Expected: PASS, byte-identical.

- [ ] **Step 4: Fast checks**

Run: `cargo clippy -q -p pleiades-data --all-targets --all-features -- -D warnings && cargo test -q -p pleiades-data --lib`
Expected: clean, all pass.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add crates/pleiades-data/src/regenerate.rs
git commit -m "refactor(data): extract the dense per-body fit used by regeneration (#201)"
```

---

### Task 2: Measure spans and size (the stop-rule gate)

**Files:**
- Modify: `crates/pleiades-data/src/lib.rs` (add `PACKAGED_ASTEROIDS`)
- Modify: `crates/pleiades-data/src/coverage/generation_spec.rs:13-26`
- Modify: `crates/pleiades-data/src/regenerate.rs` (`body_uses_heliocentric_frame`)
- Create: `crates/pleiades-data/src/tests/asteroid_fit.rs`; Modify: `crates/pleiades-data/src/tests/mod.rs`
- Create: `docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md`

**Interfaces:**
- Consumes: `fit_dense_body_artifact` (Task 1).
- Produces: `pub(crate) fn packaged_asteroids() -> &'static [CelestialBody]` returning Ceres, Pallas, Juno, Vesta, `asteroid:433-Eros` in that order (a `OnceLock` like `packaged_bodies`, because Eros is a `Custom` body and cannot sit in a `const` array).

None of these edits changes the artifact yet: `packaged_bodies()` still lists only Eros among asteroids and Eros still takes the snapshot branch.

- [ ] **Step 1: Add `packaged_asteroids()` in `lib.rs`** (next to `packaged_bodies`)

```rust
/// The asteroids the artifact fits densely from the JPL `sb441-n373s`
/// kernel, in artifact order (issue #201).
pub(crate) fn packaged_asteroids() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        vec![
            CelestialBody::Ceres,
            CelestialBody::Pallas,
            CelestialBody::Juno,
            CelestialBody::Vesta,
            CelestialBody::Custom(CustomBodyId::new("asteroid", "433-Eros")),
        ]
    })
}
```

- [ ] **Step 2: Initial spans and heliocentric frame**

In `fitting_segment_span_days`, before the `_ =>` arm:
```rust
        // Dense sb441-n373s fits (issue #201), stored heliocentric. Spans
        // measured against the kernel: see
        // docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md.
        CelestialBody::Ceres | CelestialBody::Pallas | CelestialBody::Juno | CelestialBody::Vesta => 64.0,
        CelestialBody::Custom(id) if id.catalog == "asteroid" && id.designation == "433-Eros" => 32.0,
```
In `body_uses_heliocentric_frame`, extend the doc comment ("the eight true planets and the dense asteroids") and the match:
```rust
            | CelestialBody::Ceres
            | CelestialBody::Pallas
            | CelestialBody::Juno
            | CelestialBody::Vesta
    ) || matches!(
        body,
        CelestialBody::Custom(id) if id.catalog == "asteroid" && id.designation == "433-Eros"
    )
```
Caution: Eros still goes through the snapshot branch in `build_packaged_artifact_from_reference_over`, which builds `BodyArtifact::new` (geocentric) without consulting this function, so the committed bytes are unaffected. Verify by grepping every caller of `body_uses_heliocentric_frame` (`grep -rn body_uses_heliocentric_frame crates/`) and confirming none is on the snapshot/Eros path; if one is, STOP and report.

- [ ] **Step 3: Write the measurement test** — `crates/pleiades-data/src/tests/asteroid_fit.rs`

```rust
//! Kernel-gated maintainer measurements for the dense asteroid fits
//! (issue #201). Run in release with both kernels:
//!
//! PLEIADES_DE_KERNEL=… PLEIADES_AST_KERNEL=… cargo test --release -p pleiades-data \
//!     --lib asteroid_fit -- --ignored --nocapture

use crate::packaged_asteroids;
use crate::regenerate::fit_dense_body_artifact;
use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_compression::{ArtifactHeader, CompressedArtifact};
use pleiades_jpl::spk::corpus_spec::CoverageWindow;
use pleiades_jpl::SpkBackend;

const SAMPLE_STEP_DAYS: f64 = 0.5;

fn two_kernel_reference() -> Option<SpkBackend> {
    let (Ok(de), Ok(ast)) = (
        std::env::var("PLEIADES_DE_KERNEL"),
        std::env::var("PLEIADES_AST_KERNEL"),
    ) else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL to run");
        return None;
    };
    Some(
        SpkBackend::builder()
            .add_kernel(&de)
            .expect("de440 kernel loads")
            .add_kernel(&ast)
            .expect("sb441-n373s kernel loads")
            .build(),
    )
}

fn artifact_of(bodies: Vec<pleiades_compression::BodyArtifact>) -> CompressedArtifact {
    let mut artifact =
        CompressedArtifact::new(ArtifactHeader::new("measurement", "measurement"), bodies);
    artifact.checksum = artifact.checksum().expect("checksum");
    artifact
}

/// Wrapped longitude difference × cos(lat) and latitude difference, arcsec.
fn error_arcsec(
    got: &pleiades_backend::EclipticCoordinates,
    want: &pleiades_backend::EclipticCoordinates,
) -> (f64, f64) {
    let dlon = (got.longitude.degrees() - want.longitude.degrees() + 180.0).rem_euclid(360.0)
        - 180.0;
    let lon = dlon.abs() * want.latitude.degrees().to_radians().cos() * 3600.0;
    let lat = (got.latitude.degrees() - want.latitude.degrees()).abs() * 3600.0;
    (lon, lat)
}

#[test]
#[ignore = "maintainer measurement: needs PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL; run in release"]
fn asteroid_fit_error_and_size_against_the_kernel() {
    use pleiades_backend::{EphemerisBackend, EphemerisRequest};
    let Some(reference) = two_kernel_reference() else {
        return;
    };
    let window = CoverageWindow::default().as_tuple();
    let sun = fit_dense_body_artifact(&CelestialBody::Sun, window, &reference);
    let sun_only_bytes = artifact_of(vec![sun.clone()]).encode().expect("encode").len();
    let mut total_added = 0_usize;
    for body in packaged_asteroids() {
        let fitted = fit_dense_body_artifact(body, window, &reference);
        let segments = fitted.segments.len();
        let artifact = artifact_of(vec![sun.clone(), fitted]);
        let added = artifact.encode().expect("encode").len() - sun_only_bytes;
        total_added += added;
        let (mut max_lon, mut lon_jd, mut max_lat, mut lat_jd) = (0.0_f64, 0.0, 0.0_f64, 0.0);
        let mut jd = window.0;
        while jd < window.1 {
            let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
            let got = artifact.lookup_ecliptic(body, instant).expect("artifact lookup");
            let want = reference
                .position(&EphemerisRequest::new(body.clone(), instant))
                .expect("kernel position")
                .ecliptic
                .expect("kernel ecliptic");
            let (lon, lat) = error_arcsec(&got, &want);
            if lon > max_lon {
                (max_lon, lon_jd) = (lon, jd);
            }
            if lat > max_lat {
                (max_lat, lat_jd) = (lat, jd);
            }
            jd += SAMPLE_STEP_DAYS;
        }
        eprintln!(
            "{body}: span {} d, {segments} segments, +{added} bytes, \
             max lon {max_lon:.4}\" at JD {lon_jd}, max lat {max_lat:.4}\" at JD {lat_jd}",
            crate::coverage::fitting_segment_span_days(body)
        );
    }
    eprintln!("total added by the five asteroids: {total_added} bytes");
}
```
Register it in `crates/pleiades-data/src/tests/mod.rs`: `mod asteroid_fit;`. If `regenerate` or `coverage::fitting_segment_span_days` is not visible from `tests`, widen to `pub(crate)` only.

- [ ] **Step 4: Run it**

```bash
PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp \
  cargo test --release -p pleiades-data --lib asteroid_fit -- --ignored --nocapture
```
Expected: five lines plus the total. Record them.

- [ ] **Step 5: Choose spans by the rule**

For each body: if max lon or max lat > 1″, halve its span and re-run; if both ≤ 1″, try doubling and re-run, keeping the longest power-of-two span that meets the rule. Record every run (span, segments, bytes, both maxima with JD) — one row per body per span, not aggregated.

- [ ] **Step 6: Apply the stop rule**

Add to the end of `asteroid_fit_error_and_size_against_the_kernel` (before the final `eprintln!`) the bytes the committed artifact spends on its old Eros segments:
```rust
    let committed = crate::packaged_artifact_bytes();
    let mut without_eros = CompressedArtifact::decode(committed).expect("decode committed");
    without_eros.bodies.retain(|b| !matches!(&b.body, CelestialBody::Custom(_)));
    without_eros.checksum = without_eros.checksum().expect("checksum");
    let old_eros = committed.len() - without_eros.encode().expect("encode").len();
    eprintln!(
        "projected artifact: {} bytes (committed {} - old Eros {old_eros} + added {total_added})",
        committed.len() - old_eros + total_added,
        committed.len()
    );
```
Re-run Step 4's command. If the projected size > 12,000,000, STOP: report the table and the two options (longer spans with a looser rule, or a larger budget) to the maintainer; do not continue to Task 3.

- [ ] **Step 7: Write the results note** — `docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md`

Contents: date, kernels (file names + SHA-256), command, the full per-run table from Step 5, the chosen span per body, the per-body maxima at the chosen spans with their JDs, the projected artifact size and headroom against 12,000,000, and the machine load (`cat /proc/loadavg`) — timing is not a claim here.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/pleiades-data/src docs/superpowers/specs/notes/2026-10-08-asteroid-span-measurement.md
git commit -m "test(data): measure dense asteroid fit spans against sb441-n373s (#201)"
```

---

### Task 3: Regenerate the artifact with the five asteroids

**Files:**
- Modify: `crates/pleiades-data/src/regenerate.rs` (entry points, delete Eros snapshot branch and helpers only it uses)
- Modify: `crates/pleiades-data/src/lib.rs` (`packaged_bodies`, `packaged_artifact_source_text`)
- Modify: `crates/pleiades-data/tests/artifact_regen.rs`
- Modify: `crates/pleiades-cli/src/commands/packaged_artifact.rs:150-165`, `crates/pleiades-cli/src/commands/generate_artifact.rs`, `crates/pleiades-validate/src/render/cli.rs:2785-2795`, their tests (`crates/pleiades-cli/src/cli/tests/artifact_and_workspace.rs:543-690`, `crates/pleiades-validate/src/tests/render_packaged_artifact.rs:259`)
- Modify: `crates/pleiades-data/src/tests/asteroid_fit.rs` (base-body identity check)
- Modify: `crates/pleiades-data/tests/fixtures/packaged-artifact.bin`
- Modify: `docs/spk-kernel-sourcing.md`

**Interfaces:**
- Consumes: `packaged_asteroids()`, spans from Task 2, `fit_dense_body_artifact`.
- Produces:
  - `pub fn regenerate_packaged_artifact_from_kernels_over(de_kernel: &str, asteroid_kernel: &str, window: CoverageWindow) -> Result<CompressedArtifact, String>`
  - `pub fn regenerate_packaged_artifact_from_kernels(de_kernel: &str, asteroid_kernel: &str) -> Result<CompressedArtifact, String>`
  - The single-kernel functions are removed (pre-1.0 crate; callers are first-party). `generate-packaged-artifact` error text: `"generate-packaged-artifact requires PLEIADES_DE_KERNEL (path to de440.bsp) and PLEIADES_AST_KERNEL (path to sb441-n373s.bsp); kernel-free callers use the committed artifact via packaged-artifact decode"`.

- [ ] **Step 1: Save the current artifact as the baseline**

```bash
cp crates/pleiades-data/tests/fixtures/packaged-artifact.bin /tmp/claude-1000/-workspace/f2f847ed-5fa7-4984-b2f1-ab8b2d8085b8/scratchpad/packaged-artifact.before.bin
```
(Any scratch path outside the repo works; record it.)

- [ ] **Step 2: Write the base-body identity check** (append to `tests/asteroid_fit.rs`)

```rust
/// The ten base bodies keep their exact segments across the asteroid
/// regeneration. Point `PLEIADES_BASELINE_ARTIFACT` at the artifact from
/// before the regeneration.
#[test]
#[ignore = "maintainer check: needs PLEIADES_BASELINE_ARTIFACT"]
fn base_bodies_match_the_baseline_artifact() {
    let Ok(path) = std::env::var("PLEIADES_BASELINE_ARTIFACT") else {
        eprintln!("skipping: set PLEIADES_BASELINE_ARTIFACT to run");
        return;
    };
    let before = CompressedArtifact::decode(&std::fs::read(path).expect("read baseline"))
        .expect("decode baseline");
    let after = CompressedArtifact::decode(crate::packaged_artifact_bytes()).expect("decode");
    for body in crate::PACKAGED_BASE_BODIES.iter() {
        let old = before.bodies.iter().find(|b| &b.body == body).expect("baseline body");
        let new = after.bodies.iter().find(|b| &b.body == body).expect("new body");
        assert_eq!(old, new, "{body}: segments or frame changed");
    }
}
```
If `BodyArtifact` lacks `PartialEq`, compare `old.frame == new.frame` and `old.segments == new.segments`; if `Segment` lacks `PartialEq` too, compare each segment's encoded bytes via the codec helper used by `CompressedArtifact::encode` (find it in `crates/pleiades-compression/src/codec.rs`). Make `PACKAGED_BASE_BODIES` `pub(crate)` if needed.

- [ ] **Step 3: Update the byte-identity test for two kernels** (`tests/artifact_regen.rs`)

```rust
#[test]
fn regenerated_artifact_matches_committed() {
    let (Ok(de), Ok(ast)) = (
        std::env::var("PLEIADES_DE_KERNEL"),
        std::env::var("PLEIADES_AST_KERNEL"),
    ) else {
        eprintln!("skipping: set PLEIADES_DE_KERNEL and PLEIADES_AST_KERNEL to run");
        return;
    };
    let regenerated = pleiades_data::regenerate_packaged_artifact_from_kernels(&de, &ast)
        .expect("artifact regeneration from de440 and sb441-n373s should succeed");
```
(rest unchanged). Update the module doc comment to name both variables.

- [ ] **Step 4: Run it to see it fail**

Run: `cargo test -p pleiades-data --test artifact_regen --no-run`
Expected: compile error, `regenerate_packaged_artifact_from_kernels` not found.

- [ ] **Step 5: Implement regeneration**

In `regenerate.rs`:
```rust
/// Regenerates the packaged artifact from the de440 planetary kernel and the
/// JPL `sb441-n373s` small-body kernel over an explicit coverage window.
/// Every body, asteroids included, is fit densely across `window`.
pub fn regenerate_packaged_artifact_from_kernels_over(
    de_kernel: &str,
    asteroid_kernel: &str,
    window: pleiades_jpl::spk::corpus_spec::CoverageWindow,
) -> Result<CompressedArtifact, String> {
    let backend = pleiades_jpl::SpkBackend::builder()
        .add_kernel(de_kernel)
        .map_err(|error| error.message)?
        .add_kernel(asteroid_kernel)
        .map_err(|error| error.message)?
        .build();
    Ok(build_packaged_artifact_from_reference_over(&backend, window.as_tuple()))
}

/// [`regenerate_packaged_artifact_from_kernels_over`] over the shipped
/// default window (1900–2100).
pub fn regenerate_packaged_artifact_from_kernels(
    de_kernel: &str,
    asteroid_kernel: &str,
) -> Result<CompressedArtifact, String> {
    regenerate_packaged_artifact_from_kernels_over(
        de_kernel,
        asteroid_kernel,
        pleiades_jpl::spk::corpus_spec::CoverageWindow::default(),
    )
}
```
Delete the two single-kernel functions. In `build_packaged_artifact_from_reference_over`, delete the `SelectedAsteroids | CustomBodies` match arm and the `match cadence` (every body now takes `fit_dense_body_artifact`), and update its doc comment (no snapshot re-fit). Delete any function, import or constant that becomes unused (the compiler and `cargo clippy` list them: e.g. `body_segments_from_entries`, `snapshot_fit_source`, `body_segment_span_limit` and the split constants, if nothing else uses them — check each with `grep -rn <name> crates/`; keep anything with another caller). Update the public re-exports in `crates/pleiades-data/src/lib.rs` (`grep -n regenerate_packaged_artifact crates/pleiades-data/src/lib.rs`).

In `lib.rs`, `packaged_bodies()` becomes:
```rust
pub(crate) fn packaged_bodies() -> &'static [CelestialBody] {
    static BODIES: OnceLock<Vec<CelestialBody>> = OnceLock::new();
    BODIES.get_or_init(|| {
        let mut bodies = PACKAGED_BASE_BODIES.to_vec();
        bodies.extend(packaged_asteroids().iter().cloned());
        bodies
    })
}
```
In `packaged_artifact_source_text`, replace `"… and the constrained asteroid:433-Eros sourced from its committed reference corpus, …"` with `"… and Ceres, Pallas, Juno, Vesta and asteroid:433-Eros densely fit from the JPL sb441-n373s small-body kernel, …"`. Leave the rest of the sentence alone.

- [ ] **Step 6: Update the three write paths**

`packaged_artifact.rs` and `render/cli.rs` (identical change):
```rust
    let missing = || {
        "generate-packaged-artifact requires PLEIADES_DE_KERNEL (path to de440.bsp) and \
         PLEIADES_AST_KERNEL (path to sb441-n373s.bsp); kernel-free callers use the \
         committed artifact via packaged-artifact decode"
            .to_string()
    };
    let de_kernel = std::env::var("PLEIADES_DE_KERNEL").map_err(|_| missing())?;
    let asteroid_kernel = std::env::var("PLEIADES_AST_KERNEL").map_err(|_| missing())?;
    let artifact =
        pleiades_data::regenerate_packaged_artifact_from_kernels(&de_kernel, &asteroid_kernel)?;
```
Their tests assert `error.contains("generate-packaged-artifact requires PLEIADES_DE_KERNEL")` — still true; add one assertion each that the message also contains `"PLEIADES_AST_KERNEL"`.

`generate_artifact.rs`: add a required `--asteroid-kernel <path>` flag (usage line, parse arm, `ok_or("generate-artifact requires --asteroid-kernel <sb441-n373s.bsp>")`), and call `regenerate_packaged_artifact_from_kernels_over(kernel, asteroid_kernel, window)`. Update the module doc and any CLI help text that lists the command (`grep -rn "generate-artifact" crates/pleiades-cli/src docs/cli.md`). Add a test next to the existing generate-artifact tests (find with `grep -rn render_generate_artifact crates/pleiades-cli/src`) that `render_generate_artifact(&["k.bsp", "--out", "x.bin"])` errors with the `--asteroid-kernel` message.

- [ ] **Step 7: Regenerate the committed artifact**

```bash
PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp \
  cargo run --release -p pleiades-cli -- validate generate-packaged-artifact --out crates/pleiades-data/tests/fixtures/packaged-artifact.bin
ls -l crates/pleiades-data/tests/fixtures/packaged-artifact.bin
```
Expected: size ≈ the Task 2 projection, ≤ 12,000,000. If the command spelling differs, find it with `grep -rn '"generate-packaged-artifact"' crates/pleiades-cli/src`.

- [ ] **Step 8: Verify determinism, base-body identity and size**

```bash
PLEIADES_DE_KERNEL=/workspace/.kernels/de440.bsp PLEIADES_AST_KERNEL=/workspace/.kernels/sb441-n373s.bsp \
  cargo test --release -p pleiades-data --test artifact_regen -- --nocapture
PLEIADES_BASELINE_ARTIFACT=/tmp/claude-1000/-workspace/f2f847ed-5fa7-4984-b2f1-ab8b2d8085b8/scratchpad/packaged-artifact.before.bin \
  cargo test --release -p pleiades-data --lib base_bodies_match_the_baseline_artifact -- --ignored --nocapture
cargo test -q -p pleiades-data --lib encoded_artifact_within_size_budget
```
Expected: all three PASS. A base-body failure names the body: STOP and investigate (likely the Task 2 frame edit reached a base-body path).

- [ ] **Step 9: Document the recipe** in `docs/spk-kernel-sourcing.md` — a "Regenerating the packaged artifact" subsection with the Step 7 command, both kernels, and the Step 8 checks.

- [ ] **Step 10: Commit** (other tests in `pleiades-data` may still fail on body counts; Task 4 fixes them — note this in the message)

```bash
cargo fmt --all
git add -A crates/pleiades-data crates/pleiades-cli crates/pleiades-validate docs/spk-kernel-sourcing.md
git commit -m "feat(data): fit Ceres, Pallas, Juno, Vesta and Eros densely from sb441-n373s (#201)

Regeneration now reads the de440 and sb441-n373s kernels and fits the five
asteroids heliocentric over 1900-2100, like the planets. The 17-row Eros
re-fit is gone. Base-body segments are bit-identical (checked against the
previous artifact). Serving follows in the next commit."
```

---

### Task 4: Serve the asteroids, gate them, flip the refusal tests

**Files:**
- Modify: `crates/pleiades-data/src/lib.rs` (delete `is_carried_but_unserved`; claims)
- Modify: `crates/pleiades-data/src/backend.rs` (~lines 375, 443, 488-500), `crates/pleiades-data/src/lookup.rs` (~790, 989, 1206)
- Modify: `crates/pleiades-data/src/thresholds.rs` (`ASTEROID_CORPUS_CEILING`)
- Create: `crates/pleiades-data/src/tests/asteroid_gate.rs`; Modify: `crates/pleiades-data/src/tests/mod.rs`
- Modify: `crates/pleiades-data/src/accuracy_baseline.rs` (remove Eros self-consistency, ~275-310 and ~755-800)
- Modify: `crates/pleiades-data/src/tests/lookup.rs:89,1650`, `crates/pleiades-cli/src/cli/tests/asteroid_gate.rs`, `crates/pleiades-cli/src/cli/tests/chart.rs:600`, `crates/pleiades-events/tests/nod_aps.rs:1-12,235-300`
- Modify: every test pinning "11 bundled bodies" (list in Step 8)

**Interfaces:**
- Consumes: regenerated artifact (Task 3), `packaged_asteroids()`.
- Produces: `pub(crate) const ASTEROID_CORPUS_CEILING: AsteroidCorpusCeiling` with `lon_arcsec: f64, lat_arcsec: f64` in `thresholds.rs`.

- [ ] **Step 1: Write the failing gate** — `crates/pleiades-data/src/tests/asteroid_gate.rs`

```rust
//! The packaged asteroids against every JPL `sb441-n373s` row of
//! `asteroid_reference.csv` (issue #201): geocentric ecliptic J2000,
//! geometric. The rows come from the same kernel the fits sample, at
//! instants the fits do not, so this measures fit error.

use crate::packaged_asteroids;
use crate::thresholds::ASTEROID_CORPUS_CEILING;
use crate::PackagedDataBackend;
use pleiades_backend::{EphemerisBackend, EphemerisRequest};
use pleiades_jpl::SnapshotEntry;

const ROWS_PER_BODY: usize = 407;

fn truth_longitude_latitude(row: &SnapshotEntry) -> (f64, f64) {
    let radius = (row.x_km * row.x_km + row.y_km * row.y_km + row.z_km * row.z_km).sqrt();
    (
        row.y_km.atan2(row.x_km).to_degrees(),
        (row.z_km / radius).clamp(-1.0, 1.0).asin().to_degrees(),
    )
}

#[test]
fn packaged_asteroids_match_the_sb441_rows() {
    let backend = PackagedDataBackend::new();
    for body in packaged_asteroids() {
        let rows: Vec<&SnapshotEntry> = pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| &row.body == body)
            .collect();
        assert_eq!(rows.len(), ROWS_PER_BODY, "{body}: reference rows");
        let (mut max_lon, mut max_lat) = (0.0_f64, 0.0_f64);
        for row in rows {
            let got = backend
                .position(&EphemerisRequest::new(body.clone(), row.epoch))
                .unwrap_or_else(|e| panic!("{body} at {}: {e}", row.epoch.julian_day.days()))
                .ecliptic
                .expect("ecliptic");
            let (lon, lat) = truth_longitude_latitude(row);
            let dlon = (got.longitude.degrees() - lon + 180.0).rem_euclid(360.0) - 180.0;
            max_lon = max_lon.max(dlon.abs() * lat.to_radians().cos() * 3600.0);
            max_lat = max_lat.max((got.latitude.degrees() - lat).abs() * 3600.0);
        }
        eprintln!("{body}: max lon {max_lon:.4}\", max lat {max_lat:.4}\"");
        assert!(
            max_lon <= ASTEROID_CORPUS_CEILING.lon_arcsec
                && max_lat <= ASTEROID_CORPUS_CEILING.lat_arcsec,
            "{body}: {max_lon:.4}\" / {max_lat:.4}\" over the ceiling"
        );
    }
}
```
Check that `EphemerisRequest::new` defaults to ecliptic, tropical, mean (geometric) — compare with the explicit request in `crates/pleiades-cli/src/cli/tests/asteroid_gate.rs`; if not, build the request the same explicit way. Register `mod asteroid_gate;`.

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test -q -p pleiades-data --lib packaged_asteroids_match_the_sb441_rows`
Expected: FAIL — compile error (`ASTEROID_CORPUS_CEILING` missing); after adding a placeholder-free ceiling in Step 4 it would still fail for Eros with `UnsupportedBody` until Step 3.

- [ ] **Step 3: Serve the bodies**

Delete `is_carried_but_unserved` from `lib.rs`. In each call site remove the filter/condition:
- `backend.rs` metadata: drop `.filter(|body| !crate::is_carried_but_unserved(body))`.
- `backend.rs` `supports_body`: `|| self.artifact.bodies.iter().any(|series| series.body == body)`.
- `backend.rs` position: delete the whole `if crate::is_carried_but_unserved(&req.body) { return Err(…) }` block.
- `lookup.rs` (two batch builders): `for (index, body) in packaged_bodies().iter().cloned().enumerate()`; update the comment above.
- `lookup.rs` `packaged_lookup`: delete the early-return block.

Claims in `lib.rs`:
```rust
/// Returns the per-body release claims for the packaged artifact. The planets,
/// Sun and Moon are validated inside the artifact build against the hold-out
/// corpus; the asteroids against the JPL `sb441-n373s` rows of
/// `asteroid_reference.csv` (issue #201).
pub fn packaged_body_claims() -> Vec<pleiades_backend::BodyClaim> {
    use pleiades_backend::{AccuracyClass, BodyClaim, ClaimEvidence};
    let asteroids = packaged_asteroids();
    packaged_bodies()
        .iter()
        .cloned()
        .map(|body| {
            let evidence = if asteroids.contains(&body) {
                ClaimEvidence::CorpusValidated {
                    source: "sb441-n373s".to_string(),
                }
            } else {
                ClaimEvidence::ArtifactValidated
            };
            BodyClaim::release_grade(body, AccuracyClass::High, evidence)
        })
        .collect()
}
```

- [ ] **Step 4: Set the ceiling from the measurement**

Run the gate with a temporary print-only run (`--nocapture`) by first adding the constant at the Asteroid-class value 30.0/30.0, then replace with measured × 1.4 rounded up to 2 significant figures:
```rust
/// Ceilings for the packaged asteroids against the `sb441-n373s` rows of
/// `asteroid_reference.csv` (issue #201): measured maximum × 1.4, rounded up.
///
/// | body | measured lon | measured lat |
/// |---|---|---|
/// | (one row per body, from the gate's output, 2026-10-08) | | |
pub(crate) const ASTEROID_CORPUS_CEILING: AsteroidCorpusCeiling = AsteroidCorpusCeiling {
    lon_arcsec: /* measured worst × 1.4, rounded up */,
    lat_arcsec: /* measured worst × 1.4, rounded up */,
};

/// Longitude (× cos latitude) and latitude ceilings, arcseconds.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AsteroidCorpusCeiling {
    pub lon_arcsec: f64,
    pub lat_arcsec: f64,
}
```
The table rows and both values are filled with the real numbers from the run — the comment placeholders above must not be committed. If the worst body exceeds 1″ on these rows (it should not, given Task 2), STOP and report.

- [ ] **Step 5: Run the gate**

Run: `cargo test -q -p pleiades-data --lib packaged_asteroids_match_the_sb441_rows -- --nocapture`
Expected: PASS, five measured lines.

- [ ] **Step 6: Remove the Eros self-consistency check** from `accuracy_baseline.rs`: the helper `eros_self_consistency_max_longitude_arcsec`, the ignored `print_eros_self_consistency_max_longitude_arcsec`, and `eros_round_trips_against_its_reference_snapshot_within_documented_target`. Remove imports this leaves unused.

- [ ] **Step 7: Review-focus tests** (append to `tests/asteroid_gate.rs`)

```rust
#[test]
fn packaged_asteroids_are_served_at_both_window_ends_and_refused_past_them() {
    use pleiades_backend::{Instant, JulianDay, TimeScale};
    let backend = PackagedDataBackend::new();
    let range = backend.metadata().nominal_range;
    let (start, end) = (range.start.julian_day.days(), range.end.julian_day.days());
    for body in packaged_asteroids() {
        for jd in [start, end] {
            let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tdb);
            assert!(
                backend.position(&EphemerisRequest::new(body.clone(), at)).is_ok(),
                "{body} at JD {jd}"
            );
        }
        let past = Instant::new(JulianDay::from_days(end + 1.0), TimeScale::Tdb);
        assert!(backend.position(&EphemerisRequest::new(body.clone(), past)).is_err());
    }
}

#[test]
fn an_artifact_without_asteroids_refuses_them_cleanly() {
    use pleiades_backend::{EphemerisErrorKind, Instant, JulianDay, TimeScale};
    let mut artifact = pleiades_compression::CompressedArtifact::decode(
        crate::packaged_artifact_bytes(),
    )
    .expect("decode");
    artifact.bodies.retain(|b| !packaged_asteroids().contains(&b.body));
    artifact.checksum = artifact.checksum().expect("checksum");
    let backend = PackagedDataBackend::from_artifact(artifact);
    let at = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
    let error = backend
        .position(&EphemerisRequest::new(pleiades_backend::CelestialBody::Ceres, at))
        .expect_err("no Ceres segments");
    assert_eq!(error.kind, EphemerisErrorKind::UnsupportedBody, "{error}");
}

#[test]
fn the_packaged_backend_does_not_claim_apophis() {
    let apophis = pleiades_backend::CelestialBody::Custom(pleiades_backend::CustomBodyId::new(
        "asteroid",
        "99942-Apophis",
    ));
    assert!(!PackagedDataBackend::new().supports_body(apophis));
}
```
Adjust field/method names to the real ones (`nominal_range`, `error.kind`, `supports_body` signature) — check `crates/pleiades-backend/src/` before running. If the window-end `start` instant is the reason an apparent read fails (light-time), that is fine here: these are geometric (mean) requests.

Run: `cargo test -q -p pleiades-data --lib asteroid_gate`
Expected: PASS.

- [ ] **Step 8: Flip the refusal pins and update the body-count text**

Rule for every failing test: a test that pinned *refusal* of one of the five bodies inside 1900–2100 by the packaged backend or the default chain now pins that it is *served* (and, where it checks a value, within 5″ of the `asteroid_reference.csv` row or within the corpus ceiling). A test that pins refusal *outside* the window, or anything about `JplSnapshotBackend` alone, or Apophis, stays as is. Known sites:
- `crates/pleiades-cli/src/cli/tests/asteroid_gate.rs`: rename to `the_default_chain_serves_every_truth_epoch_within_tolerance`; every row must be `Ok`, within `TRUTH_TOLERANCE_ARCSEC`, with `result.backend_id.as_str() == "packaged-data"` (check the real packaged backend id string in `backend.rs` metadata); assert `served.len() == 407`; drop `SHARED_EPOCH_JD` and `refused`. Update the module doc. `stations_of_an_asteroid_report_the_refusal` and `aspects_of_an_asteroid_report_the_refusal` become `stations_of_an_asteroid_are_searched` / `aspects_of_an_asteroid_are_searched`: `render_stations`/`render_aspects` with the same args return `Ok` (do not assert event counts — asteroid stations/aspects are ungated, #167 (d), #168 (f)).
- `crates/pleiades-cli/src/cli/tests/chart.rs:600` `chart_command_routes_selected_asteroids_via_jpl_fallback` → `chart_command_serves_selected_asteroids_from_the_packaged_backend` (Review Focus 3). Keep the J2000 longitude check (`Ceres 184.4`; adjust only the last digit if the dense fit moves it, and confirm against the `asteroid_reference.csv` J2000-nearest row within 5″). Replace the `0 of 1 placements reduced` assertion with `Apparentness: Apparent (1 of 1 placements reduced)` — the apparent reduction's light-time read now has data. Replace the JD 2451700 refusal with a success: `render_chart(&["--jd", "2451700.0", "--body", "Ceres"])` is `Ok` and also shows `1 of 1 placements reduced`. Update the doc comment above the test.
- `crates/pleiades-data/src/tests/lookup.rs:1650` `the_backend_does_not_serve_the_constrained_asteroid` → `the_backend_serves_eros`; `:89` `lookup_uses_packaged_custom_asteroid_segments` — update to the dense segments if it pins counts.
- `crates/pleiades-events/tests/nod_aps.rs`: `snapshot_only_asteroids_fail_closed` and `eros_fails_closed_past_the_packaged_backend` → the osculating nod_aps for Ceres and Eros at J2000 return `Ok` with finite fields; keep `a_snapshot_asteroid_is_served_inside_the_fixture_cluster`. Rewrite the header comment: the packaged backend serves the five asteroids densely over 1900–2100 (issue #201); Apophis is still snapshot-only.
- "11 bundled bodies" → "15 bundled bodies" with the body list as the code now renders it: `crates/pleiades-data/src/backend.rs:546`, `crates/pleiades-data/src/tests/lookup.rs:1011`, `crates/pleiades-data/src/tests/coverage.rs:505,574,2539`, `crates/pleiades-cli/src/cli/tests/artifact_and_workspace.rs:586`, `crates/pleiades-validate/src/tests/release_bundle_verify_a.rs:608,862,875,915`, `crates/pleiades-compression/src/tests.rs:1279,1283,1365` (only if the compression tests use the packaged artifact; synthetic fixtures stay).

Then sweep:
```bash
cargo test -q -p pleiades-data -p pleiades-cli -p pleiades-events -p pleiades-core -p pleiades-compression 2>&1 | grep -E "FAILED|panicked|test result" 
cargo test -q -p pleiades-validate -- --include-ignored 2>&1 | grep -E "FAILED|panicked|test result"
```
Apply the rule above to each remaining failure. Any failure the rule does not cover (a moved base-body value, a size pin, a checksum pin you cannot trace to the new bodies) — STOP and report rather than re-pinning.

- [ ] **Step 9: Commit**

```bash
cargo fmt --all
git add -A crates
git commit -m "feat(data): serve the dense asteroids from the packaged backend and gate them (#201)"
```

---

### Task 5: Claims audit, compatibility profile, docs

**Files:**
- Modify: `crates/pleiades-validate/src/claims/audit.rs:185-265`
- Modify: `crates/pleiades-core/src/compatibility/mod.rs` (profile id line 26; summary text)
- Modify: `docs/status.md:47-52`, `spec/data-compression.md:93-130,156-170`, `docs/cli.md`, `crates/pleiades-data/README.md`, `crates/pleiades-validate/data/apparent-goldens.csv` (comment lines 21-22 + pinned checksum)

- [ ] **Step 1: Audit block** — in `audit_release_grade_accuracy`, after the packaged/hold-out block, add:

```rust
    // packaged-data asteroids vs the sb441-n373s rows (issue #201). The
    // hold-out corpus has no asteroid rows, so the block above skips them.
    {
        let asteroids: Vec<CelestialBody> = pleiades_data::packaged_body_claims()
            .into_iter()
            .filter(|claim| {
                matches!(claim.evidence, pleiades_backend::ClaimEvidence::CorpusValidated { ref source } if source == "sb441-n373s")
            })
            .map(|claim| claim.body)
            .collect();
        let rows: Vec<pleiades_jpl::SnapshotEntry> = pleiades_jpl::asteroid_reference_corpus()
            .iter()
            .filter(|row| asteroids.contains(&row.body))
            .cloned()
            .collect();
        let reference = pleiades_jpl::SnapshotCorpusBackend::from_entries(rows.clone());
        let candidate = pleiades_data::PackagedDataBackend::default();
        let corpus = crate::corpus::corpus_from_entries(&rows);
        match crate::comparison::compare_backends(&reference, &candidate, &corpus) {
            Ok(report) => check_report(&report, "pleiades-data", &asteroids, &mut errors),
            Err(_) => errors.push(ClaimAuditError::DeclaredBodyNotComputable {
                backend: "pleiades-data".into(),
                body: "<sb441-n373s asteroids>".into(),
            }),
        }
    }
```
`crate::corpus::corpus_from_entries` may not exist: look at how `crate::corpus::asteroid_corpus()` builds its corpus and add a filtered variant beside it (same construction, entries filtered to `asteroids`). Use the real field names of `BodyClaim` (check `crates/pleiades-backend/src/claims.rs`). Update the stale comment in the hold-out block ("The packaged backend has no claim for asteroid:433-Eros…").

Run: `cargo test -q -p pleiades-validate --lib claims -- --include-ignored`
Expected: PASS (the audit is `#[ignore]`d; `--include-ignored` runs it).

- [ ] **Step 2: Compatibility profile** — `CURRENT_COMPATIBILITY_PROFILE_ID` → `"pleiades-compatibility-profile/0.7.34"`. In `CURRENT_COMPATIBILITY_PROFILE_SUMMARY`, find the `nod_aps` sentence "offline, nod_aps serves an asteroid only inside the January 2001 fixture cluster, …" and replace with "offline, nod_aps serves Ceres, Pallas, Juno, Vesta and asteroid:433-Eros across 1900-2100 from the packaged artifact's dense sb441-n373s fits (issue #201), and asteroid:99942-Apophis only inside the January 2001 fixture cluster". Append a release note sentence at the end of the summary: "Dense packaged asteroids (issue #201): PackagedDataBackend serves Ceres, Pallas, Juno, Vesta and asteroid:433-Eros on every date in 1900-2100 from heliocentric fits to the JPL sb441-n373s small-body kernel, release-grade with corpus evidence and gated against the 407 sb441-n373s rows per body of asteroid_reference.csv (measured max <lon>″ longitude, <lat>″ latitude); a chart asking for one of them away from the snapshot fixture rows now returns a position instead of the backend's out-of-range refusal, and inside the January 2001 cluster moves from the snapshot interpolation to the packaged fit by up to about 2″. Compatibility profile bumped to 0.7.34." — with the real measured maxima from Task 4. Run the compatibility tests and re-pin whatever content checksum they report (`cargo test -q -p pleiades-core compatibility`); a checksum test failure here is expected and is re-pinned to the value the test prints.

- [ ] **Step 3: Docs**
- `docs/status.md`: replace the "Asteroids offline are served only at their sample rows" limit with: the packaged backend serves the five bodies across 1900–2100 (measured maxima, link the gate); Apophis remains snapshot-only; asteroid stations and aspects are searched but ungated (#167 (d), #168 (f)). Delete the carried-but-unserved Eros note.
- `spec/data-compression.md`: move Eros from "Geocentric storage (Sun, Moon, Eros)" to the heliocentric section, renamed "Heliocentric storage (planets Mercury–Pluto and the dense asteroids)"; replace the Eros ceiling row and its "self-consistency only / not served" note with a row for the five asteroids citing `ASTEROID_CORPUS_CEILING` and its source rows.
- `docs/cli.md`: remove the note that a default chart at an isolated sample row renders the asteroid on its J2000 mean place with "0 of 1 placements reduced" (find with `grep -n "placements reduced" docs/cli.md`); document `generate-artifact --asteroid-kernel`.
- `crates/pleiades-data/README.md`: body list and source description.
- `crates/pleiades-validate/data/apparent-goldens.csv` lines 21-22: replace the Eros exclusion reason with "433-Eros: EXCLUDED from these goldens, which predate its dense packaged fit (issue #201); its accuracy is gated against asteroid_reference.csv." Then re-pin the file's checksum where the test reports it (`cargo test -q -p pleiades-validate apparent_goldens -- --include-ignored`).

- [ ] **Step 4: Docs build** — `mise run docs`. Expected: clean (no broken intra-doc links; `is_carried_but_unserved` links are gone).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
git add -A crates docs spec
git commit -m "docs: dense packaged asteroids; claims audit and compatibility profile 0.7.34 (#201)"
```

---

### Task 6: Full verification and PR

- [ ] **Step 1: Static checks**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
Expected: clean.

- [ ] **Step 2: Blocking tier** — `mise run ci`. Expected: pass. (Includes `mise run docs`.)

- [ ] **Step 3: Release smoke** — `mise run release-smoke`. Expected: pass; the crossings golden must not move (base bodies are bit-identical). If it moves, STOP: that means a base body changed.

- [ ] **Step 4: Full tier** — `mise run test-full` (foreground; no edits or commits while it runs). Expected: pass.

- [ ] **Step 5: Package size** — `mise run package-check`. Expected: every `.crate` under 9 MiB; record `pleiades-data`'s size.

- [ ] **Step 6: Push and open the PR** (credential workaround: `git -c credential.helper= -c credential.helper='!gh auth git-credential' push -u origin feat/201-asteroid-dense-fits`). PR body: problem, the span table and size from the Task 2 note, the gate's measured maxima and ceilings, base-body bit-identity evidence, the behaviour change (asteroids served offline; snapshot cluster moves ≤ 2″), what stays open on #201/#167/#168/#160, and the verification list with real results. Dispatch nightly on the branch (`gh workflow run nightly.yml --ref feat/201-asteroid-dense-fits`) and report its jobs in a PR comment.
