# Apparent-place annual aberration double count (#93) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop `apparent_position` adding a second annual-aberration term on top of the light-time re-query that already carries it, then re-pin every validation ceiling that had absorbed the ~20″ error.

**Architecture:** One function in `pleiades-apparent` changes (the generic light-time path drops the applied Meeus 23.2 term but keeps reporting its estimate in provenance). Nothing else in the code paths changes: charts, events and the eclipse crate already consume that function or have their own correct path. The rest of the work is measurement: diagnostic per-body residual tests, re-pinned goldens tolerances and gate ceilings in `pleiades-validate`, a regenerated engine-golden column in the crossings corpus, and a follow-up entry for the Moon.

**Tech Stack:** Rust workspace (stable toolchain from `mise.toml`), `cargo nextest`, `pleiades-validate` gates run via `cargo run -q -p pleiades-validate -- <gate>`. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-30-apparent-aberration-double-count-design.md`

**Branch:** `fix/apparent-aberration-double-count` (already created; the spec is its first commit).

## Global Constraints

- Public API of `pleiades-apparent` does not change: `apparent_position` keeps its signature including `sun_true_longitude_of_date_deg`.
- The Sun path (`apparent_sun_position`) and apsis path (`apparent_apsis_position`) are not modified.
- Provenance on the light-time path: `corrections.annual_aberration` stays `true`, `corrections.light_time` stays `true`, `aberration_longitude_arcsec` carries the Meeus 23.2 estimate of the included component.
- A validation ceiling is only ever tightened in this change, never loosened. A gate whose measured maximum gets worse is a finding to diagnose, not a number to raise.
- Goldens values (Horizons columns) are never regenerated here; only tolerance columns and header comments change. The crossings corpus `pleiades_jd_tdb` engine-golden column IS regenerated (it is engine output, not a reference).
- Ceiling conventions, per gate: apparent/equatorial/topocentric goldens = max observed residual + 2″ per body, rounded up to 0.1″; crossings Tier-2 = ceil(1.4 × group max) in whole arcseconds; pheno, occultation and rise/set constants = ~1.4 × measured, rounded up to a clean value (as their threshold-file comments already do).
- Every re-pinned number carries the measured maximum, the row it came from, and the date `2026-09-30` in the adjacent comment or header.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the blocking suite `mise run test` must pass at every commit. No source edits while a background test run is in progress.
- Commit messages follow conventional commits (`fix(apparent): …`, `test(validate): …`, `docs: …`); release-plz owns versions and changelogs.

## Review Focus

Inputs the spec implies but no gate exercises, each pinned by a test in the owning task:

1. **A non-finite `sun_true_longitude_of_date_deg`** (NaN from a failed Sun lookup upstream). Before the fix this failed closed through `combine_apparent`; after the fix the term is no longer applied, so without a guard the position would come back finite with NaN provenance. Expected: still fail closed. Pinned in Task 2 (`non_finite_sun_longitude_fails_closed`).
2. **A body at high ecliptic latitude** (Pluto, β up to 17°). The retarded-Earth aberration must move latitude as well as longitude. Expected: agreement with a retarded-Earth reference in both coordinates. Pinned in Task 2 (the regression test uses an inclined orbit and asserts latitude too).
3. **A synthetic backend whose body does not move** (the unit-test fixtures). The retarded query then contains no aberration at all, and the output must be precession plus nutation only, not a 20″ offset. Pinned in Task 2 (`light_time_path_applies_no_separate_aberration_term`).
4. **The apparent-place speed from PR #92** differences the apparent-minus-mean correction over ±0.5 day. The correction no longer contains the aberration term, so its rate changes; the speed must still equal the rate of the chart's own apparent longitude. Expected: the existing `chart/apparent_motion/tests.rs` suite passes unchanged. Checked in Task 4.
5. **Chained longitude-crossing searches** rely on the committed crossings corpus engine-golden column; after the fix every geocentric crossing instant moves (about 40 s for the Moon, up to ~15 min for slow planets). Expected: Tier-1 self-consistency passes only after the golden column is regenerated from the fixed engine, and Tier-2 SE parity tightens. Pinned in Task 6.

---

## Measurement log

Fill these tables in as the tasks run. They are the record the spec asks for.

### Before the fix (Task 1, on the spec commit)

| Gate | Reported maximum | Row / body |
|---|---|---|
| validate-apparent | 43.46" | Moon, jd 2451545 |
| validate-equatorial (RA / Dec) | 41.69" / 11.70" | Moon, jd 2451545 (both) |
| validate-topocentric (lon) | 35.82" | Moon, jd 2488065.5 |
| validate-crossings Tier-2 (overall) | 35.1" | helio/Mercury (35.090"), row helio,Mercury,0.0,2426000.5 |
| validate-pheno elongation / phase angle | 20.966" / 57.791" | row not printed by the gate |
| validate-occultations contact / contact_grazing | 45.782 s / 710.029 s | row not printed by the gate |
| validate-rise-trans rise/set tight / transit | 3.259 s / 0.781 s | row not printed by the gate |

Per-body (from the diagnostic tests in Task 1):

| Body | apparent goldens max \|res\| | equatorial max RA / Dec | topocentric max lon | crossings Tier-2 group max |
|---|---|---|---|---|
| Sun | 2.83" | 2.825" / 0.244" | 0.080" | geo 0.322" |
| Moon | 43.46" | 41.69" / 11.70" | 35.82" | geo 21.701" |
| Mercury | 25.31" | 25.21" / 4.76" | n/a (no rows) | geo 20.868" |
| Venus | 24.51" | 24.44" / 6.31" | n/a (no rows) | geo 20.911" |
| Mars | 20.93" | 20.74" / 5.87" | 20.99" | geo 19.998" |
| Jupiter | 19.93" | 19.42" / 4.84" | n/a (no rows) | geo 19.989" |
| Saturn | 20.21" | 19.95" / 4.00" | n/a (no rows) | geo 20.960" |
| Uranus | 20.87" | 20.87" / 5.19" | n/a (no rows) | geo 20.209" |
| Neptune | 19.52" | 19.50" / 4.41" | n/a (no rows) | geo 19.831" |
| Pluto | 18.61" | 18.38" / 7.98" | n/a (no rows) | geo 11.882" |
| helio (non-Pluto) | n/a | n/a | n/a | 35.090" (Mercury; Venus 23.912", Mars 17.951", Jupiter 9.322", Saturn 6.540", Uranus 5.095", Neptune 4.137") |
| helio Pluto | n/a | n/a | n/a | 3.530" |

### After the fix (Task 4)

Same two tables, measured on the Task 2 commit. The crossings engine-golden column was regenerated in Task 2, so `validate-crossings` passes Tier-1 (max 0.000 s).

| Gate | Reported maximum | Row / body |
|---|---|---|
| validate-apparent | 38.78" | Moon, jd 2469807.5 |
| validate-equatorial (RA / Dec) | 38.28" / 12.37" | Moon, RA jd 2488065.5 / Dec jd 2469807.5 |
| validate-topocentric (lon) | 14.68" | Moon, jd 2488065.5 |
| validate-crossings Tier-2 (overall) | 35.1" | helio/Mercury (35.090"), row helio,Mercury,0.0,2426000.5 |
| validate-pheno elongation / phase angle | 1.803" / 37.037" | row not printed by the gate |
| validate-occultations contact / contact_grazing | 6.747 s / 709.779 s | row not printed by the gate |
| validate-rise-trans rise/set tight / transit | 3.259 s / 0.383 s | row not printed by the gate |

| Body | apparent goldens max \|res\| | equatorial max RA / Dec | topocentric max lon | crossings Tier-2 group max |
|---|---|---|---|---|
| Sun | 2.83" | 2.825" / 0.244" | 0.080" | geo 0.322" |
| Moon | 38.78" | 38.28" / 12.37" | 14.68" | geo 2.606" |
| Mercury | 4.54" | 4.53" / 0.49" | n/a (no rows) | geo 0.328" |
| Venus | 3.67" | 3.68" / 1.05" | n/a (no rows) | geo 0.376" |
| Mars | 2.03" | 1.89" / 0.73" | 0.17" | geo 0.356" |
| Jupiter | 0.46" | 0.45" / 0.10" | n/a (no rows) | geo 0.315" |
| Saturn | 0.26" | 0.26" / 0.05" | n/a (no rows) | geo 0.483" |
| Uranus | 0.26" | 0.25" / 0.08" | n/a (no rows) | geo 0.261" |
| Neptune | 0.19" | 0.18" / 0.06" | n/a (no rows) | geo 0.435" |
| Pluto | 0.18" | 0.17" / 0.08" | n/a (no rows) | geo 0.697" |
| helio (non-Pluto) | n/a | n/a | n/a | 35.090" (Mercury; Venus 23.912", Mars 17.951", Jupiter 9.322", Saturn 6.540", Uranus 5.095", Neptune 4.137") |
| helio Pluto | n/a | n/a | n/a | 3.530" |

Moon signed apparent residuals per epoch (jd 2415025.5, 2433282.5, 2451545, 2469807.5, 2488065.5): +1.095", -14.781", -32.100", -38.781", -38.704". They grow steadily with epoch and do not agree with the SE crossings residual (geo/Moon max 2.606"), so the Moon's apparent-vs-Horizons gap is not a crossings-parity effect; the cause is the goldens' UT epoch tag (Horizons queried without `TIME_TYPE=TT`; see FU-14). The Moon's equatorial Dec maximum rose from 11.70" (Task 1) to 12.37" (jd 2469807.5) after the fix; every other cell is unchanged or lower.

In `validate-occultations` the metrics not gated by this change rose after the fix: `planet_mag_rel` 0.0489 to 0.0502 and `sublunar` 20.2' to 21.1' (both within their ceilings 0.07 / 30').

---

### Task 1: Per-body diagnostic tests and the pre-fix baseline

**Files:**
- Modify: `crates/pleiades-validate/src/apparent_validation.rs` (tests module, after `pinned_checksum`, ~line 396)
- Modify: `crates/pleiades-validate/src/equatorial_validation.rs` (goldens tests module, next to `pinned_checksum`, ~line 603)
- Modify: `crates/pleiades-validate/src/topocentric_validation.rs` (tests module, ~line 541)
- Modify: `crates/pleiades-validate/src/crossings_validation.rs` (tests module, ~line 298)
- Modify: this plan's Measurement log

**Interfaces:**
- Consumes: `parse_goldens()` / `parse()` row structs already in each module (`row.body`, `row.body_label`, `row.jd_tt`, plus `apparent_longitude_deg`, `ra_deg`/`dec_deg`, `topo_longitude_deg`), `madrid_observer()` in topocentric, `parse_body` and `EventEngine::longitude_at` in crossings.
- Produces: four `#[ignore]` diagnostic tests that print per-body residual maxima. They stay in the tree (precedent: `equatorial_validation::se_tests::measure_max_residuals`).

- [ ] **Step 1: Add the apparent-goldens per-body diagnostic**

In `crates/pleiades-validate/src/apparent_validation.rs`, inside `mod tests`, after `pinned_checksum`:

```rust
    /// Diagnostic: signed per-row residual against the Horizons goldens and the
    /// aberration component the pipeline reports, plus a per-body maximum. Run with
    /// `cargo test -p pleiades-validate apparent_validation::tests::measure_per_body_residuals -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_per_body_residuals() {
        let rows = parse_goldens().expect("goldens parse");
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let mut max_by_body: std::collections::BTreeMap<String, (f64, f64)> =
            std::collections::BTreeMap::new();
        eprintln!("body,jd_tt,signed_residual_arcsec,reported_aberration_arcsec");
        for row in &rows {
            let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
            let request = ChartRequest::new(instant)
                .with_bodies(vec![row.body.clone()])
                .with_apparentness(Apparentness::Apparent);
            let snapshot = engine.chart(&request).expect("chart");
            let placement = snapshot.placement_for(&row.body).expect("placement");
            let got = placement
                .position
                .ecliptic
                .as_ref()
                .expect("ecliptic")
                .longitude
                .degrees();
            let mut signed = got - row.apparent_longitude_deg;
            if signed > 180.0 {
                signed -= 360.0;
            } else if signed < -180.0 {
                signed += 360.0;
            }
            let residual_arcsec = signed * 3600.0;
            let aberration = placement
                .apparent
                .as_ref()
                .map(|a| a.aberration_longitude_arcsec)
                .unwrap_or(f64::NAN);
            eprintln!(
                "{},{},{residual_arcsec:.3},{aberration:.3}",
                row.body_label, row.jd_tt
            );
            let entry = max_by_body
                .entry(row.body_label.clone())
                .or_insert((0.0, row.jd_tt));
            if residual_arcsec.abs() > entry.0 {
                *entry = (residual_arcsec.abs(), row.jd_tt);
            }
        }
        for (body, (max, jd)) in &max_by_body {
            eprintln!("max {body} {max:.3}\" at jd {jd}");
        }
    }
```

- [ ] **Step 2: Add the equatorial-goldens per-body diagnostic**

In `crates/pleiades-validate/src/equatorial_validation.rs`, in the same `mod tests` block that holds the goldens `pinned_checksum` (not `se_tests`):

```rust
    /// Diagnostic: per-body maximum cos(Dec)-weighted RA and Dec residual against
    /// the Horizons goldens. Run with
    /// `cargo test -p pleiades-validate equatorial_validation::tests::measure_goldens_per_body -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_goldens_per_body() {
        let rows = parse().expect("goldens parse");
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let mut max_by_body: std::collections::BTreeMap<String, (f64, f64, f64, f64)> =
            std::collections::BTreeMap::new();
        for row in &rows {
            let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
            let req = ChartRequest::new(instant)
                .with_bodies(vec![row.body.clone()])
                .with_apparentness(Apparentness::Apparent);
            let snap = engine.chart(&req).expect("chart");
            let p = snap.placement_for(&row.body).expect("placement");
            let eq = p.position.equatorial.expect("equatorial");
            let cos_dec = row.dec_deg.to_radians().cos();
            let ra_resid =
                (wrap_deg(eq.right_ascension.degrees() - row.ra_deg).abs() * cos_dec) * 3600.0;
            let dec_resid = (eq.declination.degrees() - row.dec_deg).abs() * 3600.0;
            eprintln!(
                "{},{},{ra_resid:.3},{dec_resid:.3}",
                row.body_label, row.jd_tt
            );
            let entry = max_by_body
                .entry(row.body_label.clone())
                .or_insert((0.0, row.jd_tt, 0.0, row.jd_tt));
            if ra_resid > entry.0 {
                entry.0 = ra_resid;
                entry.1 = row.jd_tt;
            }
            if dec_resid > entry.2 {
                entry.2 = dec_resid;
                entry.3 = row.jd_tt;
            }
        }
        for (body, (ra, ra_jd, dec, dec_jd)) in &max_by_body {
            eprintln!("max {body} RA {ra:.3}\" at jd {ra_jd}; Dec {dec:.3}\" at jd {dec_jd}");
        }
    }
```

- [ ] **Step 3: Add the topocentric-goldens per-body diagnostic**

In `crates/pleiades-validate/src/topocentric_validation.rs`, inside `mod tests` after `pinned_checksum`:

```rust
    /// Diagnostic: per-body maximum longitude residual against the Horizons
    /// topocentric goldens (Madrid observer). Run with
    /// `cargo test -p pleiades-validate topocentric_validation::tests::measure_per_body_residuals -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_per_body_residuals() {
        let rows = parse_goldens().expect("goldens parse");
        let engine = ChartEngine::new(PackagedDataBackend::new());
        let mut max_by_body: std::collections::BTreeMap<String, (f64, f64)> =
            std::collections::BTreeMap::new();
        for row in &rows {
            let instant = Instant::new(JulianDay::from_days(row.jd_tt), TimeScale::Tt);
            let request = ChartRequest::new(instant)
                .with_bodies(vec![row.body.clone()])
                .with_apparentness(Apparentness::Apparent)
                .with_observer(madrid_observer())
                .with_topocentric(true);
            let snapshot = engine.chart(&request).expect("chart");
            let placement = snapshot.placement_for(&row.body).expect("placement");
            let ecl = placement.position.ecliptic.as_ref().expect("ecliptic");
            let mut lon_diff = (ecl.longitude.degrees() - row.topo_longitude_deg).abs();
            if lon_diff > 180.0 {
                lon_diff = 360.0 - lon_diff;
            }
            let lon_residual_arcsec = lon_diff * 3600.0;
            let lat_residual_arcsec =
                (ecl.latitude.degrees() - row.topo_latitude_deg).abs() * 3600.0;
            eprintln!(
                "{},{},{lon_residual_arcsec:.3},{lat_residual_arcsec:.3}",
                row.body_label, row.jd_tt
            );
            let entry = max_by_body
                .entry(row.body_label.clone())
                .or_insert((0.0, row.jd_tt));
            if lon_residual_arcsec > entry.0 {
                *entry = (lon_residual_arcsec, row.jd_tt);
            }
        }
        for (body, (max, jd)) in &max_by_body {
            eprintln!("max {body} lon {max:.3}\" at jd {jd}");
        }
    }
```

- [ ] **Step 4: Add the crossings per-group Tier-2 diagnostic**

In `crates/pleiades-validate/src/crossings_validation.rs`, inside `mod tests`:

```rust
    /// Diagnostic: Tier-2 SE-parity maximum per (frame, body) group, so the
    /// per-group ceilings can be re-measured. Run with
    /// `cargo test -p pleiades-validate crossings_validation::tests::measure_per_group_parity -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn measure_per_group_parity() {
        let engine = EventEngine::new(packaged_backend());
        let mut max_by_group: std::collections::BTreeMap<String, (f64, String)> =
            std::collections::BTreeMap::new();
        for line in CORPUS_CSV.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("frame,") {
                continue;
            }
            let f: Vec<&str> = line.split(',').collect();
            let frame = match f[0] {
                "geo" => CrossingFrame::GeocentricApparentOfDate,
                "helio" => CrossingFrame::Heliocentric,
                other => panic!("unknown frame {other}"),
            };
            let body = parse_body(f[1]).expect("known body");
            let target: f64 = f[2].parse().expect("target");
            let se_jd: f64 = f[5].parse().expect("se jd");
            let se_instant = Instant::new(JulianDay::from_days(se_jd), TimeScale::Tdb);
            let lambda = engine
                .longitude_at(body.clone(), frame, se_instant)
                .expect("longitude_at");
            let residual_arcsec = wrap180_deg(lambda.degrees() - target).abs() * 3600.0;
            let group = format!("{}/{}", f[0], f[1]);
            let entry = max_by_group
                .entry(group)
                .or_insert((0.0, String::new()));
            if residual_arcsec > entry.0 {
                *entry = (residual_arcsec, line.to_string());
            }
        }
        for (group, (max, row)) in &max_by_group {
            eprintln!("max {group} {max:.3}\" on row: {row}");
        }
    }
```

- [ ] **Step 5: Compile the tests and confirm they are skipped by default**

Run: `cargo nextest run -p pleiades-validate -E 'test(measure_per_body_residuals) | test(measure_goldens_per_body) | test(measure_per_group_parity)'`
Expected: builds; the four tests are listed as skipped (ignored). Then `cargo clippy -p pleiades-validate --all-targets -- -D warnings` and `cargo fmt --all --check` both clean.

- [ ] **Step 6: Run the diagnostics and the gates on the unfixed code**

```bash
cargo test -p pleiades-validate apparent_validation::tests::measure_per_body_residuals -- --nocapture --ignored
cargo test -p pleiades-validate equatorial_validation::tests::measure_goldens_per_body -- --nocapture --ignored
cargo test -p pleiades-validate topocentric_validation::tests::measure_per_body_residuals -- --nocapture --ignored
cargo test -p pleiades-validate crossings_validation::tests::measure_per_group_parity -- --nocapture --ignored
cargo run -q -p pleiades-validate -- validate-apparent
cargo run -q -p pleiades-validate -- validate-equatorial
cargo run -q -p pleiades-validate -- validate-topocentric
cargo run -q -p pleiades-validate -- validate-crossings
cargo run -q -p pleiades-validate -- validate-pheno
cargo run -q -p pleiades-validate -- validate-occultations
cargo run -q -p pleiades-validate -- validate-rise-trans
```

Expected: every gate passes with its current ceilings. Copy the printed maxima into the "Before the fix" tables of the Measurement log. Sanity check against the issue: planet rows in the apparent diagnostic should show |residual| tracking the reported aberration column (about 15–25″), Sun rows under 3″.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/src/apparent_validation.rs crates/pleiades-validate/src/equatorial_validation.rs crates/pleiades-validate/src/topocentric_validation.rs crates/pleiades-validate/src/crossings_validation.rs docs/superpowers/plans/2026-09-30-apparent-aberration-double-count.md
git commit -m "test(validate): per-body residual diagnostics for the apparent, equatorial, topocentric and crossings gates (#93)"
```

---

### Task 2: Drop the separate aberration term from the light-time path

**Files:**
- Modify: `crates/pleiades-apparent/src/apparent.rs` (module doc lines 1–5; `apparent_position` lines 62–130; `apparent_sun_position` doc lines 132–150)
- Modify: `crates/pleiades-apparent/src/apparent/tests.rs`
- Modify: `crates/pleiades-apparent/src/provenance.rs` (field docs, lines 14–17 and 54–58)
- Modify: `crates/pleiades-apparent/src/lib.rs` (crate doc lines 1–2 and 11–15, example comment line 29)
- Modify: `crates/pleiades-apparent/README.md` (lines 3–4)

**Interfaces:**
- Consumes: `annual_aberration(lambda_deg, beta_deg, sun_true_longitude_deg, jd_tt) -> AberrationOffset { d_lambda_arcsec, d_beta_arcsec }`, `combine_apparent(lambda, beta, d_lambda_arcsec, d_beta_arcsec, delta_psi_arcsec, stage)`, `LIGHT_TIME_DAYS_PER_AU` from `crate::lighttime`.
- Produces: unchanged public signature `apparent_position<F, E>(instant, sun_true_longitude_of_date_deg, max_iterations, query) -> Result<ApparentPosition, ApparentLightTimeError<E>>`; on this path the returned `ecliptic` is retarded-geocentric + precession + Δψ only, `provenance.aberration_longitude_arcsec` is the Meeus estimate, and a non-finite estimate returns `ApparentPlaceError::NonFiniteCorrection { stage: "aberration-estimate" }`.

- [ ] **Step 1: Write the failing "no separate term" test**

Append to `crates/pleiades-apparent/src/apparent/tests.rs`:

```rust
#[test]
fn light_time_path_applies_no_separate_aberration_term() {
    // A body that does not move between queries carries no aberration in its
    // retarded geocentric position, so the output must be precession + Δψ only.
    // With the Sun 180° from the body the Meeus estimate is ≈ +κ ≈ +20.5″, a
    // regression that re-adds the term fails here by that amount.
    let jd = 2_451_545.0_f64;
    let instant = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 280.0, 8, |_| {
        Ok(fixed(100.0, 5.0, 1.0))
    })
    .unwrap();

    let p = crate::precession::precess_ecliptic_j2000_to_date(100.0, 5.0, jd).unwrap();
    let nut = crate::nutation::nutation(jd).unwrap();
    let expected_lon = (p.longitude_deg + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0);
    let dlon_arcsec = (out.ecliptic.longitude.degrees() - expected_lon) * 3600.0;
    let dlat_arcsec = (out.ecliptic.latitude.degrees() - p.latitude_deg) * 3600.0;
    assert!(dlon_arcsec.abs() < 1e-6, "separate aberration term applied: {dlon_arcsec}\"");
    assert!(dlat_arcsec.abs() < 1e-6, "latitude moved: {dlat_arcsec}\"");

    // The estimate of the included component is still reported, and it is not small.
    let ab = crate::aberration::annual_aberration(p.longitude_deg, p.latitude_deg, 280.0, jd);
    assert!(ab.d_lambda_arcsec.abs() > 5.0, "fixture should give a ~20\" estimate");
    assert!((out.provenance.aberration_longitude_arcsec - ab.d_lambda_arcsec).abs() < 1e-12);
    assert!(out.provenance.corrections.annual_aberration);
    assert!(out.provenance.corrections.light_time);
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo nextest run -p pleiades-apparent -E 'test(light_time_path_applies_no_separate_aberration_term)'`
Expected: FAIL with `separate aberration term applied: 20.4…"`.

- [ ] **Step 3: Write the failing retarded-Earth regression test**

Append to the same file. Heliocentric circular orbits, Earth at 1 AU, a Jupiter-like body at 5.2 AU on an orbit inclined 1.3°; the geocentric query returns `body(t′) − Earth(t′)`; the reference retards only the body, keeps the Earth at `t`, and applies first-order aberration once.

```rust
mod retarded_earth {
    use super::*;
    use crate::lighttime::LIGHT_TIME_DAYS_PER_AU;

    type Vec3 = [f64; 3];

    pub fn earth_helio(t_days: f64) -> Vec3 {
        let a = core::f64::consts::TAU / 365.25 * t_days;
        [a.cos(), a.sin(), 0.0]
    }

    pub fn body_helio(t_days: f64) -> Vec3 {
        // Phase offset 1 rad so the body is neither at opposition nor conjunction.
        let a = core::f64::consts::TAU / 4332.6 * t_days + 1.0;
        let (r, inc) = (5.2, 1.3_f64.to_radians());
        [r * a.cos(), r * a.sin() * inc.cos(), r * a.sin() * inc.sin()]
    }

    pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
        [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
    }

    pub fn norm(v: Vec3) -> f64 {
        (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
    }

    pub fn to_ecliptic(v: Vec3) -> EclipticCoordinates {
        let d = norm(v);
        let lon = v[1].atan2(v[0]).to_degrees().rem_euclid(360.0);
        let lat = (v[2] / d).asin().to_degrees();
        fixed(lon, lat, d)
    }

    /// `body(t − τ) − Earth(t)`, τ converged, then first-order aberration
    /// (û + v/c) applied once. J2000 frame, degrees.
    pub fn reference_j2000_at_epoch() -> EclipticCoordinates {
        let earth = earth_helio(0.0);
        let mut tau = 0.0_f64;
        for _ in 0..10 {
            tau = norm(sub(body_helio(-tau), earth)) * LIGHT_TIME_DAYS_PER_AU;
        }
        let u = sub(body_helio(-tau), earth);
        let d = norm(u);
        let h = 1e-3;
        let (e1, e0) = (earth_helio(h), earth_helio(-h));
        let v = [
            (e1[0] - e0[0]) / (2.0 * h),
            (e1[1] - e0[1]) / (2.0 * h),
            (e1[2] - e0[2]) / (2.0 * h),
        ];
        // v/c with c in AU/day = 1 / LIGHT_TIME_DAYS_PER_AU.
        let aberrated = [
            u[0] / d + v[0] * LIGHT_TIME_DAYS_PER_AU,
            u[1] / d + v[1] * LIGHT_TIME_DAYS_PER_AU,
            u[2] / d + v[2] * LIGHT_TIME_DAYS_PER_AU,
        ];
        to_ecliptic(aberrated)
    }
}

#[test]
fn light_time_requery_of_geocentric_position_matches_retarded_earth_reference() {
    // The retarded geocentric query is body(t−τ) − Earth(t−τ). Retarding the
    // Earth by τ displaces the direction by v·τ = Δ·(v/c), the first-order
    // annual aberration, so the pipeline must agree with an explicit
    // "retard the body only, then aberrate once" reference to well under 0.1″
    // in both longitude and latitude. Second-order terms are ~0.01″.
    use retarded_earth::*;
    let jd0 = 2_451_545.0 + 1000.0;
    let instant = Instant::new(JulianDay::from_days(jd0), TimeScale::Tt);
    let out = apparent_position::<_, ApparentPlaceError>(instant, 0.0, 8, |t| {
        let dt = t.julian_day.days() - jd0;
        Ok(to_ecliptic(sub(body_helio(dt), earth_helio(dt))))
    })
    .unwrap();

    let reference = reference_j2000_at_epoch();
    let p = crate::precession::precess_ecliptic_j2000_to_date(
        reference.longitude.degrees(),
        reference.latitude.degrees(),
        jd0,
    )
    .unwrap();
    let nut = crate::nutation::nutation(jd0).unwrap();
    let expected_lon = (p.longitude_deg + nut.delta_psi_arcsec / 3600.0).rem_euclid(360.0);
    let expected_lat = p.latitude_deg;

    let mut dlon = out.ecliptic.longitude.degrees() - expected_lon;
    if dlon > 180.0 {
        dlon -= 360.0;
    } else if dlon < -180.0 {
        dlon += 360.0;
    }
    let dlon_arcsec = dlon * 3600.0;
    let dlat_arcsec = (out.ecliptic.latitude.degrees() - expected_lat) * 3600.0;
    assert!(dlon_arcsec.abs() < 0.1, "longitude off retarded-Earth reference by {dlon_arcsec}\"");
    assert!(dlat_arcsec.abs() < 0.1, "latitude off retarded-Earth reference by {dlat_arcsec}\"");
    assert!(out.provenance.light_time_days > 0.02, "Jupiter-like light-time expected");
}
```

- [ ] **Step 4: Write the failing non-finite Sun-longitude test**

```rust
#[test]
fn non_finite_sun_longitude_fails_closed() {
    // The Sun longitude only feeds the provenance estimate now, but a NaN in
    // provenance must not ride out on a finite position.
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
    let err = apparent_position::<_, ApparentPlaceError>(instant, f64::NAN, 8, |_| {
        Ok(fixed(100.0, 0.0, 1.0))
    })
    .unwrap_err();
    assert!(
        matches!(
            err,
            ApparentLightTimeError::Apparent(ApparentPlaceError::NonFiniteCorrection {
                stage: "aberration-estimate"
            })
        ),
        "unexpected error: {err:?}"
    );
}
```

- [ ] **Step 5: Run the three new tests and confirm they fail**

Run: `cargo nextest run -p pleiades-apparent -E 'test(light_time_requery_of_geocentric_position_matches_retarded_earth_reference) | test(non_finite_sun_longitude_fails_closed)'`
Expected: the retarded-Earth test FAILS by roughly 20″ in longitude; the NaN test FAILS because the current error stage is `"apparent-combine"`.

- [ ] **Step 6: Change `apparent_position`**

In `crates/pleiades-apparent/src/apparent.rs`, replace the body of `apparent_position` from `let aberration = …` through `.map_err(ApparentLightTimeError::Apparent)?;` of the `combine_apparent` call with:

```rust
    // The light-time re-query retarded the Earth along with the body, so the
    // retarded geocentric direction already carries annual aberration to first
    // order. The Meeus 23.2 value is computed only to report that included
    // component in the provenance; it is NOT added to the position (#93).
    let aberration = annual_aberration(lambda, beta, sun_true_longitude_of_date_deg, jd_tt);
    if !aberration.d_lambda_arcsec.is_finite() || !aberration.d_beta_arcsec.is_finite() {
        return Err(ApparentLightTimeError::Apparent(
            ApparentPlaceError::NonFiniteCorrection {
                stage: "aberration-estimate",
            },
        ));
    }
    let nut = nutation(jd_tt).map_err(ApparentLightTimeError::Apparent)?;

    let (apparent_lon, apparent_lat) = combine_apparent(
        lambda,
        beta,
        0.0,
        0.0,
        nut.delta_psi_arcsec,
        "apparent-combine",
    )
    .map_err(ApparentLightTimeError::Apparent)?;
```

Leave the provenance construction as is (`aberration_longitude_arcsec: aberration.d_lambda_arcsec`, `annual_aberration: true`, `light_time: true`).

- [ ] **Step 7: Rewrite the module doc and the two function docs**

Replace lines 1–5 of `apparent.rs` with:

```rust
//! Orchestrator: light-time-corrected J2000 position + Sun's longitude of date +
//! instant -> apparent ecliptic-of-date position with provenance. Applies, in
//! order: light-time (a re-query of the *geocentric* position at `t − τ`, which
//! retards the Earth along with the body and so already carries annual
//! aberration to first order — Meeus, *Astronomical Algorithms* ch. 33),
//! precession (J2000 -> mean equinox of date), then nutation Δψ (-> true equinox
//! of date). No separate annual-aberration term is added on that path; the
//! Meeus 23.2 value is computed only to report the included component in the
//! provenance (#93). Gravitational light-deflection is not applied (sub-arcsec
//! except near the solar limb).
```

Replace the `apparent_position` doc comment with:

```rust
/// Computes the apparent ecliptic-of-date position for a body.
///
/// `query` returns the body's geocentric ecliptic position (J2000, with
/// `distance_au`) at a given instant in mean mode; it is re-queried at the
/// light-time-retarded instant `t − τ`. Because that position is
/// `body(t − τ) − Earth(t − τ)`, the Earth is retarded with the body and the
/// direction already includes annual aberration. Precession and nutation are
/// then applied; no further aberration term is added.
///
/// `sun_true_longitude_of_date_deg` is the Sun's true geometric longitude OF
/// DATE at `instant` (the caller is responsible for precessing it). It feeds
/// only the provenance's `aberration_longitude_arcsec`, the Meeus 23.2 estimate
/// of the aberration component the retarded query contains; a non-finite value
/// fails closed with `NonFiniteCorrection { stage: "aberration-estimate" }`.
```

In the `apparent_sun_position` doc, replace the paragraph beginning `/// For a planet, light-time retardation and annual aberration are physically` and ending `/// nutation, and aberration once — never a light-time re-query.` with:

```rust
/// Every body's light-time re-query of its *geocentric* position at `t − τ`
/// carries annual aberration, because the Earth is retarded together with the
/// body (see the module doc; this is why [`apparent_position`] adds no separate
/// term). The Sun path takes the equivalent un-retarded route instead: it
/// starts from the Sun's instantaneous Mean/J2000 geocentric position and
/// applies precession, nutation, and the Meeus 23.2 term once — never a
/// light-time re-query. For the Sun the two routes agree to first order
/// (~20.5″); the un-retarded form keeps the Sun's distance and provenance
/// simple and was the FU-1 fix (2026-06-30).
```

- [ ] **Step 8: Update the provenance field docs**

In `crates/pleiades-apparent/src/provenance.rs`:

Replace `/// Annual aberration was applied.` (the `CorrectionSet` field) with:

```rust
    /// The place includes annual aberration. On the light-time path it is
    /// carried by the retarded geocentric re-query (no separate term is added);
    /// on the Sun path it is the single Meeus 23.2 term added to the
    /// un-retarded position.
```

Replace the `aberration_longitude_arcsec` doc (`/// Annual-aberration shift applied to longitude, arcseconds (0.0 on the` / `/// aberration-free lunar-apsis path).`) with:

```rust
    /// Annual-aberration component of the longitude, arcseconds. On the
    /// light-time path this is the Meeus 23.2 *estimate* of the component the
    /// retarded geocentric query already contains (reported, not added); on
    /// the Sun path it is the term actually added; 0.0 on the aberration-free
    /// lunar-apsis path.
```

- [ ] **Step 9: Update the crate doc, doc example comment, and README**

In `crates/pleiades-apparent/src/lib.rs` replace lines 1–2 with:

```rust
//! Apparent-place corrections: light-time (whose geocentric re-query carries
//! annual aberration), precession-to-date, and nutation-in-longitude, with
//! typed provenance; the geocentric Sun applies aberration once instead.
```

In the same file replace the example paragraph sentence `geocentric position at a (light-time-retarded) instant; the routine applies` / `light-time, precession to the equinox of date, annual aberration, and` / `nutation-in-longitude.` with:

```rust
//! geocentric position at a (light-time-retarded) instant; the routine applies
//! light-time (which carries annual aberration), precession to the equinox of
//! date, and nutation-in-longitude.
```

and the example line `//!     280.0, // Sun's true longitude of date, of-date, for the aberration term` with:

```rust
//!     280.0, // Sun's true longitude of date, for the aberration provenance estimate
```

In `crates/pleiades-apparent/README.md` replace lines 3–4 (`Apparent-place corrections for the `pleiades` workspace: light-time (planetary` / `aberration), precession-to-date, annual aberration, and nutation-in-longitude,`) with:

```markdown
Apparent-place corrections for the `pleiades` workspace: light-time (whose
geocentric re-query carries annual aberration), precession-to-date, and
nutation-in-longitude,
```

- [ ] **Step 10: Adjust the two existing tests whose premise changed**

In `apparent/tests.rs`:

Rename `at_j2000_only_aberration_and_nutation_shift_longitude` to `at_j2000_only_nutation_shifts_longitude`, change its first comment line to `// At J2000 precession is the identity, so the shift from mean is only Δψ/3600`, and replace `assert!(shift_arcsec.abs() < 40.0, "shift {shift_arcsec}\"");` with:

```rust
    assert!(shift_arcsec.abs() < 20.0, "shift {shift_arcsec}\" must be Δψ only");
    assert!(shift_arcsec.abs() > 1.0, "nutation Δψ not applied: {shift_arcsec}\"");
```

Rename `latitude_moves_by_precession_and_aberration_only` to `latitude_unchanged_on_light_time_path_at_j2000`, change its comment to `// At J2000 precession is the identity and Δψ does not touch latitude; with no separate aberration term the latitude passes through.` and replace `assert!(dlat_arcsec.abs() < 1.0, "Δβ {dlat_arcsec}\"");` with `assert!(dlat_arcsec.abs() < 1e-3, "Δβ {dlat_arcsec}\"");`.

`apparent_position_provenance_is_fully_specified` needs no change (the estimate still matches `annual_aberration`).

- [ ] **Step 11: Run the crate's tests, docs, lint and format**

```bash
cargo nextest run -p pleiades-apparent
cargo test --doc -p pleiades-apparent
cargo clippy -p pleiades-apparent --all-targets --all-features -- -D warnings
cargo fmt --all --check
```

Expected: all pass, including the three new tests.

- [ ] **Step 12: Commit**

```bash
git add crates/pleiades-apparent
git commit -m "fix(apparent): stop adding annual aberration on top of the light-time re-query (#93)"
```

---

### Task 3: Correct the consumer documentation and the FU-1 record

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:604-607` (comment on the Sun arm of `apparent_place`)
- Modify: `crates/pleiades-eclipse/src/ephemeris.rs:87-90` and `:132-140`
- Modify: `docs/follow-ups.md` FU-1 entry (line 16 status line; paragraph at lines 30–33)
- Modify: `docs/time-observer-policy.md:70`

**Interfaces:** none (comments and docs only).

- [ ] **Step 1: Chart dispatch comment**

In `crates/pleiades-core/src/chart/mod.rs`, replace the four comment lines above `CelestialBody::Sun =>` with:

```rust
            // Sun: light-time and aberration are the same ~20.5″ effect, so the
            // Sun path applies aberration ONCE to the instantaneous (un-retarded)
            // geocentric Sun. Every other body goes through apparent_position,
            // whose geocentric light-time re-query already carries aberration
            // (no separate term, #93). observer = None keeps the aberration
            // argument geocentric.
```

- [ ] **Step 2: Eclipse ephemeris docs**

In `crates/pleiades-eclipse/src/ephemeris.rs`, replace the note on `read_retarded` (`/// Note: this is distinct from, and must not be combined with, the annual` through `/// uses the *un*-retarded [`read`].`) with:

```rust
/// Note: a retarded *geocentric* position already carries annual aberration
/// (the Earth is retarded with the body), so nothing here adds a separate
/// aberration term; `eclipsed_longitude` takes the equivalent un-retarded
/// route through [`apparent_sun_longitude_deg`] and uses the *un*-retarded
/// [`read`].
```

Replace the paragraph `/// For a planet, light-time retardation (re-querying the body at the epoch the` through `/// separate annual-aberration term double-counts ~20.5″.` with:

```rust
/// Re-querying any body's geocentric position at the epoch the light left it
/// retards the Earth as well, so the retarded direction already carries annual
/// aberration; `pleiades_apparent::apparent_position` therefore adds no
/// separate term (#93). For the **Sun** this crate takes the equivalent
/// un-retarded route: the ~20.5″ displacement caused by Earth's orbital
/// velocity is applied once to the instantaneous position. A routine that
/// applied a light-time re-query *and* a separate annual-aberration term would
/// double-count ~20.5″, which was FU-1 for the Sun and #93 for the planets.
```

- [ ] **Step 3: FU-1 record**

In `docs/follow-ups.md`, append to the FU-1 `**Status:**` line (line 16), before the `· **Severity:**` token:

```
· **Addendum (2026-09-30):** the "planets are unaffected" reasoning below was wrong: the geocentric light-time re-query retards the Earth too and so already carries aberration, and the generic path added it a second time for every non-Sun body. Fixed under issue #93 (`docs/superpowers/specs/2026-09-30-apparent-aberration-double-count-design.md`); the Moon's remaining residual is FU-14.
```

Replace the FU-1 parenthetical `(This is Sun-specific: for the planets, light-time and` … `checked but is likely unaffected for the same reason as planets.)` with:

```
(At the time this was thought Sun-specific; see the 2026-09-30 addendum in the status line — it was not.)
```

- [ ] **Step 4: Time/observer policy line**

In `docs/time-observer-policy.md` line 70, replace `light-time + precession-to-date + annual aberration + nutation-in-longitude, true equinox of date;` with `light-time (whose geocentric re-query carries annual aberration) + precession-to-date + nutation-in-longitude, true equinox of date; the geocentric Sun applies aberration once to its un-retarded position instead;`.

Leave `crates/pleiades-validate/src/posture/backend_policy.rs`'s policy summary string unchanged: it lists the corrections a place contains, which is still true, and it is pinned by tests.

- [ ] **Step 5: Build docs and commit**

Run: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` (this is `mise run docs`).
Expected: clean.

```bash
git add crates/pleiades-core/src/chart/mod.rs crates/pleiades-eclipse/src/ephemeris.rs docs/follow-ups.md docs/time-observer-policy.md
git commit -m "docs: the light-time re-query carries annual aberration for every body (#93)"
```

---

### Task 4: Blocking suite and post-fix measurement

**Files:**
- Modify: this plan's Measurement log ("After the fix" tables)

**Interfaces:** none.

- [ ] **Step 1: Run the blocking suite and doctests**

```bash
cargo nextest run --workspace -E 'not package(pleiades-validate)'
cargo test --doc --workspace
```

Expected: all pass. The `pleiades-core` `chart/apparent_motion/tests.rs` suite passes unchanged (Review Focus item 4). If any test outside `pleiades-validate` fails, apply this rule: a failure whose expected value was pinned from the engine's own apparent output and now differs by roughly the reported aberration (15–25″ in longitude, or that amount divided by the body's rate in time) has the bug baked into its expectation; re-derive that one expectation from the fixed engine and say so in the commit message. Any other failure is a finding: stop and report it rather than adjusting the test.

- [ ] **Step 2: Run the diagnostics and gates on the fixed code**

Run the same eleven commands as Task 1 Step 6. Expected outcomes:

- `validate-apparent`, `validate-equatorial`, `validate-topocentric`, `validate-pheno`, `validate-occultations`, `validate-rise-trans`: PASS with smaller maxima (the old ceilings are loose).
- `validate-crossings`: FAILS Tier-1 self-consistency (the engine-golden column is stale). That is expected; Task 6 regenerates it. The per-group diagnostic still runs because it only exercises Tier-2.
- Apparent diagnostic: Jupiter–Pluto within 0.5″, Mars within 2.1″, Mercury and Venus within 4.6″ (issue #93's numbers); Sun rows unchanged from Task 1; Moon rows between about −39″ and +1″.

- [ ] **Step 3: Fill the "After the fix" tables and commit the log**

```bash
git add docs/superpowers/plans/2026-09-30-apparent-aberration-double-count.md
git commit -m "docs(plan): record gate maxima before and after the #93 fix"
```

---

### Task 5: Re-pin the three goldens files

**Files:**
- Modify: `crates/pleiades-validate/data/apparent-goldens.csv` (header lines 6–17, tolerance column)
- Modify: `crates/pleiades-validate/data/equatorial-goldens.csv` (header planet/Moon lines, tolerance columns)
- Modify: `crates/pleiades-validate/data/topocentric-goldens.csv` (header lines 8–14, `lon_tolerance_arcsec` column)
- Modify: `crates/pleiades-validate/scripts/regen-apparent-goldens.sh:29-52`
- Modify: `crates/pleiades-validate/scripts/regen-equatorial-goldens.sh:30-32`
- Modify: `crates/pleiades-validate/src/apparent_validation.rs:17`, `equatorial_validation.rs:17`, `topocentric_validation.rs:20` (`GOLDENS_CHECKSUM`)

**Interfaces:**
- Consumes: the "After the fix" per-body table.
- Produces: tightened tolerances that the gates enforce.

Tolerance rule for every row of a body: `ceil((max |residual| for that body in that file) + 2.0)` to one decimal place; Sun rows keep `5.0` in the apparent file and `6.0` in the equatorial file (the Sun did not move, and those were pinned under FU-1). Latitude and Dec tolerances that already sit at their measured basis (`20.0` topocentric latitude; equatorial Dec) are re-pinned only if their measured maximum dropped below the current value minus 2″.

- [ ] **Step 1: Apparent goldens**

Rewrite the header's `TOLERANCE RATIONALE` block so the planet and Moon lines read (substituting the measured values):

```
#   - Planets: the light-time re-query of the geocentric position carries annual
#     aberration; no separate term is added (#93, 2026-09-30). The residual is the
#     polynomial-fit ephemeris vs DE441 floor. Tolerance = per-body max observed
#     residual + 2" (Mercury <max>", Venus <max>", Mars <max>", Jupiter <max>",
#     Saturn <max>", Uranus <max>", Neptune <max>", Pluto <max>"), measured 2026-09-30.
#     The former 26" ceiling was absorbing a ~20" double count.
#   - Moon: same path. Max observed residual <max>" (2026-09-30) after the #93 fix;
#     tolerance = max + 2". The remaining residual is tracked as FU-14 in
#     docs/follow-ups.md.
```

Change every planet row's fourth column from `26.0` to that body's computed tolerance, and the Moon rows from `45.0` to the Moon's, only if lower.

- [ ] **Step 2: Regen script for the apparent goldens**

In `regen-apparent-goldens.sh`, mirror the new header text in the `echo` lines 29–39, and replace

```bash
    if [ "$label" = "Moon" ]; then tol=45.0; else tol=26.0; fi
```

with a per-body table using the same values as the CSV:

```bash
    # Per-body tolerance = max observed residual + 2" (measured 2026-09-30, #93).
    case "$label" in
      Sun) tol=5.0 ;;
      Moon) tol=<Moon> ;;
      Mercury) tol=<Mercury> ;;
      Venus) tol=<Venus> ;;
      Mars) tol=<Mars> ;;
      Jupiter) tol=<Jupiter> ;;
      Saturn) tol=<Saturn> ;;
      Uranus) tol=<Uranus> ;;
      Neptune) tol=<Neptune> ;;
      Pluto) tol=<Pluto> ;;
    esac
```

- [ ] **Step 3: Equatorial goldens and its regen script**

Replace the two header lines

```
#   - Planets: polynomial-fit ephemeris accuracy vs JPL DE441, apparent-mode residuals
#     15-25 arcsec; tolerance 26" matches the ecliptic apparent gate.
#   - Moon: ELP2000 theory limit (annual aberration systematic), tolerance 45" unchanged.
```

with

```
#   - Planets: light-time re-query carries annual aberration, no separate term (#93,
#     2026-09-30); residual is the polynomial-fit floor. RA tolerance = per-body max
#     observed cos(Dec)-weighted residual + 2" (Mercury <max>", Venus <max>", Mars <max>",
#     Jupiter <max>", Saturn <max>", Uranus <max>", Neptune <max>", Pluto <max>"); Dec
#     tolerance = per-body max observed + 2" where that is below the former value.
#   - Moon: same path; RA max observed <max>" after #93 (2026-09-30), tolerance = max + 2".
#     Remaining residual tracked as FU-14.
```

Update the planet and Moon rows' fifth (and, where re-pinned, sixth) columns. Mirror the header text in the `echo "#` lines 30–32 of `regen-equatorial-goldens.sh`, update its line 11 comment (`planets 26", Moon 45"`) to `per-body RA/Dec tolerances = max observed + 2" (#93, 2026-09-30)`, and replace its single shared tolerance

```bash
    if [ "$label" = "Moon" ]; then tol=45.0; elif [ "$label" = "Sun" ]; then tol=6.0; else tol=26.0; fi
```

and the row emit `echo "$label,$jd,$radec,$tol,$tol"` with separate RA and Dec values matching the CSV:

```bash
    # Per-body RA / Dec tolerance = max observed residual + 2" (measured 2026-09-30, #93).
    case "$label" in
      Sun) ra_tol=6.0; dec_tol=6.0 ;;
      Moon) ra_tol=<Moon RA>; dec_tol=<Moon Dec> ;;
      Mercury) ra_tol=<Mercury RA>; dec_tol=<Mercury Dec> ;;
      Venus) ra_tol=<Venus RA>; dec_tol=<Venus Dec> ;;
      Mars) ra_tol=<Mars RA>; dec_tol=<Mars Dec> ;;
      Jupiter) ra_tol=<Jupiter RA>; dec_tol=<Jupiter Dec> ;;
      Saturn) ra_tol=<Saturn RA>; dec_tol=<Saturn Dec> ;;
      Uranus) ra_tol=<Uranus RA>; dec_tol=<Uranus Dec> ;;
      Neptune) ra_tol=<Neptune RA>; dec_tol=<Neptune Dec> ;;
      Pluto) ra_tol=<Pluto RA>; dec_tol=<Pluto Dec> ;;
    esac
```

```bash
      echo "$label,$jd,$radec,$ra_tol,$dec_tol"
```

A Dec value that is not re-pinned keeps its current CSV value in the table.

- [ ] **Step 4: Topocentric goldens**

Replace the header lines

```
#   - Sun/planets longitude: The packaged polynomial-fit ephemeris matches JPL DE441 to ~26
#     arcsec for Sun/planets (the observed residual floor in the apparent goldens). The
#     topocentric correction for the Sun is ~8.8 arcsec peak (sub-arcsec vs the data limit)
#     and ~0.7 arcsec for Mars, so the dominant error is the packaged ephemeris vs DE441,
#     not the topocentric formula. Longitude tolerances match the apparent goldens (26 arcsec).
#   - Moon longitude: ELP2000-based theory; same 45 arcsec data ceiling as the apparent
#     goldens. The lunar topocentric parallax is ~52 arcsec peak and is resolvable via the
#     non-geocentric assertion (Moon topo vs geo > 0.1°). The absolute longitude tolerance is
#     set to 45 arcsec for the same data-ceiling reason.
```

with

```
#   - Sun/Mars longitude: the topocentric correction is ~8.8 arcsec peak for the Sun and
#     ~0.7 arcsec for Mars; the residual is the packaged-ephemeris floor. Since #93
#     (2026-09-30) the light-time re-query carries annual aberration with no separate term.
#     Longitude tolerance = per-body max observed residual + 2" (Sun <max>", Mars <max>").
#   - Moon longitude: same path; max observed residual <max>" after #93 (2026-09-30),
#     tolerance = max + 2" (remaining residual tracked as FU-14). The lunar topocentric
#     parallax is ~52 arcsec peak and is resolvable via the non-geocentric assertion
#     (Moon topo vs geo > 0.1°).
```

Update the `lon_tolerance_arcsec` column of every row accordingly. There is no regen script for this file.

- [ ] **Step 5: Re-pin the three checksums**

Run: `cargo test -p pleiades-validate pinned_checksum -- --nocapture`
Expected: three failures, each printing the new value (`checksum = N` or `update GOLDENS_CHECKSUM to N`). Set each `GOLDENS_CHECKSUM` constant to its printed value (keep the existing underscore grouping style in `topocentric_validation.rs` and `equatorial_validation.rs`).

- [ ] **Step 6: Run the three gates and their tests**

```bash
cargo run -q -p pleiades-validate -- validate-apparent
cargo run -q -p pleiades-validate -- validate-equatorial
cargo run -q -p pleiades-validate -- validate-topocentric
cargo nextest run -p pleiades-validate -E 'test(apparent_validation) | test(equatorial_validation) | test(topocentric_validation)'
```

Expected: all PASS with the tightened tolerances.

- [ ] **Step 7: Commit**

```bash
git add crates/pleiades-validate/data/apparent-goldens.csv crates/pleiades-validate/data/equatorial-goldens.csv crates/pleiades-validate/data/topocentric-goldens.csv crates/pleiades-validate/scripts crates/pleiades-validate/src/apparent_validation.rs crates/pleiades-validate/src/equatorial_validation.rs crates/pleiades-validate/src/topocentric_validation.rs
git commit -m "test(validate): tighten apparent, equatorial and topocentric goldens to the post-#93 residuals"
```

---

### Task 6: Regenerate the crossings golden column and re-pin the event-gate ceilings

**Files:**
- Modify: `crates/pleiades-validate/data/crossings-corpus/crossings.csv` (column `pleiades_jd_tdb`, via the CLI)
- Modify: `crates/pleiades-validate/data/crossings-corpus/manifest.txt` (`checksum=` line)
- Modify: `crates/pleiades-validate/src/crossings_validation.rs:34-48` (ceiling constants and their comment)
- Modify: `crates/pleiades-validate/src/pheno_thresholds.rs` (`PHASE_ANGLE_ARCSEC`, `ELONGATION_ARCSEC` and comments)
- Modify: `crates/pleiades-validate/src/occult_thresholds.rs` (`CONTACT_SECONDS`, `CONTACT_SECONDS_GRAZING` and comments)
- Modify: `crates/pleiades-validate/src/rise_trans_thresholds.rs` (`RISE_SET_SECONDS_TIGHT`, `TRANSIT_SECONDS` and comments, only if moved)

**Interfaces:**
- Consumes: the "After the fix" tables; `cargo run -q -p pleiades-validate -- crossings-golden` (regenerate) and `-- crossings-golden --check`.
- Produces: a crossings corpus whose Tier-1 golden matches the fixed engine, and ceilings at ~1.4× the new maxima.

- [ ] **Step 1: Regenerate the crossings golden column**

```bash
cargo run -q -p pleiades-validate -- crossings-golden
```

Expected output: `crossings-golden: wrote …/crossings.csv; manifest checksum= N`. Put `N` on the `checksum=` line of `manifest.txt`. Then:

```bash
cargo run -q -p pleiades-validate -- crossings-golden --check
git diff --stat crates/pleiades-validate/data/crossings-corpus/
```

Expected: `crossings-golden: committed golden column is current`; the diff touches only the seventh column and the manifest, and the row count stays 86 (`EXPECTED_ROWS` unchanged). Spot-check one Moon row: the new golden should differ from the old by about 40 s (`0.00046` day), and one outer-planet row by minutes.

- [ ] **Step 2: Re-pin the crossings Tier-2 ceilings**

Using the per-group maxima from the Task 4 diagnostic, set each constant to `ceil(1.4 × group max)`, never higher than today's value, and rewrite the comment block (lines 34–38) to:

```rust
// Tier-2 per-body arcsecond ceilings — MEASURED from the committed corpus and set
// to ceil(1.4x each body-class group max). Cross-theory (SE Moshier vs engine)
// floors, not engine error. Measured group maxima (86-row corpus, 2026-09-30, after
// the #93 aberration fix): geo Sun <max>", geo Moon <max>", geo planets
// (Mercury-Neptune) <max>", helio (non-Pluto) <max>". Before #93 the geo Moon and
// planet groups measured 21.70" and 20.96": the double-counted ~20" term.
```

Update the Pluto comment's measured numbers the same way (`PLUTO_ARCSEC` covers both geo and helio Pluto; take the larger of the two groups).

- [ ] **Step 3: Re-pin pheno, occultation and rise/set ceilings**

For each constant below, if the "After the fix" maximum is lower than the "Before" one, set the constant to ~1.4× the new maximum rounded up to a clean value (whole arcseconds or seconds), and rewrite its doc comment to `Measured max <value> (<row>) on 2026-09-30 corpus, after the #93 aberration fix; ceiling ~1.4×.` If the maximum did not drop, leave both the constant and comment alone.

- `pheno_thresholds.rs`: `ELONGATION_ARCSEC` (was 30.0 from 20.966″ Uranus), `PHASE_ANGLE_ARCSEC` (was 85.0 from 57.79″ Mercury).
- `occult_thresholds.rs`: `CONTACT_SECONDS` (was 65.0 from 46.44 s), `CONTACT_SECONDS_GRAZING` (was 995.0 from 710.03 s).
- `rise_trans_thresholds.rs`: `RISE_SET_SECONDS_TIGHT` (was 5.0 from 3.4631 s), `TRANSIT_SECONDS` (was 4.0).

If any of these maxima got *worse*, do not touch the constant: record the number in the Measurement log, and stop to report it as a finding before continuing.

- [ ] **Step 4: Run the four gates and the validate package tests**

```bash
cargo run -q -p pleiades-validate -- validate-crossings
cargo run -q -p pleiades-validate -- validate-pheno
cargo run -q -p pleiades-validate -- validate-occultations
cargo run -q -p pleiades-validate -- validate-rise-trans
cargo nextest run -p pleiades-validate -E 'test(crossings_validation) | test(pheno) | test(occult) | test(rise_trans)'
```

Expected: all PASS; the crossings summary reports Tier-1 max well under 1 s and Tier-2 max under the new ceilings.

- [ ] **Step 5: Commit**

```bash
git add crates/pleiades-validate/data/crossings-corpus crates/pleiades-validate/src/crossings_validation.rs crates/pleiades-validate/src/pheno_thresholds.rs crates/pleiades-validate/src/occult_thresholds.rs crates/pleiades-validate/src/rise_trans_thresholds.rs
git commit -m "test(validate): regenerate the crossings golden column and tighten event-gate ceilings after #93"
```

---

### Task 7: Record the Moon measurement as FU-14

**Files:**
- Modify: `docs/follow-ups.md` (append after FU-13)

**Interfaces:**
- Consumes: the Moon rows of the "After the fix" per-body table (apparent goldens signed residuals per epoch, equatorial and topocentric Moon maxima, crossings `geo/Moon` group max).

- [ ] **Step 1: Write the entry**

Append to `docs/follow-ups.md`:

```markdown
---

## FU-14: Moon apparent-place residual against Horizons after the #93 aberration fix

**Status:** open · Opened 2026-09-30 while fixing #93.

**What:** #93 removed the separate annual-aberration term from the generic
light-time path; the planets' Horizons residuals collapsed to the ephemeris-fit
floor (Jupiter–Pluto under 0.5″). The Moon did not. Measured on the #93 branch
(`validate-apparent` diagnostic, geocentric, Horizons ObsEcLon Q31):

| JD (TT) | Moon residual vs Horizons |
|---|---|
| 2415025.5 | <signed>″ |
| 2433282.5 | <signed>″ |
| 2451545.0 | <signed>″ |
| 2469807.5 | <signed>″ |
| 2488065.5 | <signed>″ |

Against Swiss Ephemeris (`validate-crossings` Tier-2, `geo/Moon` group, engine
longitude at the SE crossing instant) the same code measures a maximum of
<max>″.

**Reading:** <one of the two sentences below, whichever the numbers support>
- The two references disagree by roughly the size of the removed term, so the
  suspect is the Horizons goldens themselves (what Horizons' geocentric
  "apparent" ObsEcLon includes for the Moon) rather than the packaged Moon.
- The two references agree, so the suspect is the packaged Moon's fit against
  its source over these epochs, and the crossings gate ceiling now bounds it.

**Impact:** Moon apparent longitude may carry a systematic error of up to the
Horizons residual above in charts and every `pleiades-events` surface built on
it (about 2 s of time per 1″). The `validate-apparent`, `validate-equatorial`
and `validate-topocentric` Moon tolerances were tightened to the measured
maxima + 2″ under #93 and bound it.

**Suggested next step:** one probe per hypothesis. (a) Query Horizons for the
Moon at one epoch with `QUANTITIES='31'` and again with the geometric
`QUANTITIES='18'`-style astrometric quantity and compare their difference to
the engine's provenance aberration estimate. (b) Compare the packaged Moon's
J2000 geometric longitude at the same epoch against the DE440 SPK sample the
artifact was fit from.

**Origin:** issue #93, `docs/superpowers/specs/2026-09-30-apparent-aberration-double-count-design.md` section 4.
```

Fill every `<…>` from the Measurement log before committing; the entry must contain no placeholders.

- [ ] **Step 2: Commit**

```bash
git add docs/follow-ups.md
git commit -m "docs(follow-ups): FU-14, the Moon's Horizons residual after the #93 fix"
```

---

### Task 8: Whole-branch verification

**Files:** none new.

- [ ] **Step 1: Blocking CI**

Run: `mise run ci`
Expected: PASS (fmt, lint, docs, audit, deny, secrets, claims-audit, test, doctest, release-smoke).

- [ ] **Step 2: Full suite including the validate package**

Run: `cargo nextest run --workspace --run-ignored all`
Expected: PASS. This runs the goldens `*_pass` tests, the crossings corpus test, the checksum pins and the new diagnostics. Do not edit sources or commit while it runs.

- [ ] **Step 3: Every gate the spec names, once more, from clean**

```bash
for g in validate-apparent validate-equatorial validate-topocentric validate-crossings validate-pheno validate-occultations validate-rise-trans validate-eclipses validate-eclipses-local; do
  cargo run -q -p pleiades-validate -- $g || { echo "FAILED: $g"; break; }
done
```

Expected: nine passes. The two eclipse gates are included to confirm the eclipse crate was unaffected (its numbers should match main).

- [ ] **Step 4: Check the plan's Measurement log has no empty cells and the spec's table matches**

Read both tables; every cell filled, every "After" value at or below its "Before" value except where a finding was recorded in Task 6 Step 3.

- [ ] **Step 5: Hand off**

Use `superpowers:finishing-a-development-branch`. PR title: `fix(apparent): stop double-counting annual aberration on the light-time path (#93)`. The PR body lists: the fix, the provenance semantics, the before/after table per gate, the FU-14 pointer, and the note that the crossings golden column was regenerated from the fixed engine.
