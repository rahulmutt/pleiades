# SP-6-FU KNOWN GAP 3 — graze-boundary differential diagnosis

**Date:** 2026-07-09
**Branch:** `feat/sp6-fu-occult-central-graze` (HEAD deef4641)
**Task:** SP-6-FU Task 7 — quantified probe decision tree (ΔT → ephemeris source → topocentric transform → classification semantics).

## Command

```bash
cargo test -p pleiades-validate --test occult_graze_diagnosis -- --ignored --nocapture
```

Supporting checks:
```bash
cargo test -p pleiades-data --lib print_packaged_artifact_baseline_summary -- --ignored --nocapture   # Moon ≡ de440
```

## Differential table (build noise trimmed)

Columns: `dt_diff_s` = our ΔT − SE ΔT (s); `geoMoon"`/`topoMoon"` = our-vs-SE geocentric/topocentric Moon
offset (arcsec); `topoTgt"` = our-vs-SE topocentric target offset (arcsec); `sdMoon"` = semidiameter diff
(arcsec); `ourMargin'` = our refined `sep − (s_moon+s_tgt)` at OUR closest approach (arcmin);
`seMargin'` = SE's `sep − (s_moon+s_tgt)` from SE's topo columns AT THE ANCHOR (arcmin). Negative margin = disks overlap.

```
row                    dt_diff_s  geoMoon" topoMoon"  topoTgt"  sdMoon" ourMargin' seMargin'    type flag
Aldebaran@miss           -0.032    14.486    14.438     0.151    0.021    -5.567    -5.486 Total <-- DISAGREE (SE: Miss)
Aldebaran@miss            0.099     9.903     9.977     0.161    0.026    -0.889    -0.875 Total <-- DISAGREE (SE: Miss)
Aldebaran@miss            9.911     5.856     6.519     0.130   -0.147    -4.619    -4.639 Total <-- DISAGREE (SE: Miss) [dT predicted]
Regulus@miss              0.262    14.946    15.057     0.069   -0.120    -3.653    -3.092 Total <-- DISAGREE (SE: Miss)
Regulus@miss             -0.179    10.082    10.243     0.146   -0.139     0.175    17.143 Miss
Regulus@miss              5.866    18.555    18.735     0.161   -0.013    -0.363    -0.236 Total <-- DISAGREE (SE: Miss) [dT predicted]
Spica@miss                0.324    16.339    16.222     0.062    0.002    -0.005     6.729 Total <-- DISAGREE (SE: Miss)
Spica@miss                0.178     4.195     4.099     0.140    0.037    -4.795    -4.467 Total <-- DISAGREE (SE: Miss)
Spica@miss                5.087     9.051     9.590     0.047    0.033     0.072    11.695 Miss [dT predicted]
Antares@miss              0.265    16.779    16.715     0.153   -0.115     0.035     0.653 Miss
Antares@miss              0.032    15.255    15.408     0.212   -0.116     0.073     0.544 Miss
Antares@miss              4.534     3.579     4.611     0.133    0.043     0.129     4.900 Miss [dT predicted]
Venus@miss               -0.040    18.871    19.237    18.807   -0.090     0.093     0.098 Miss
Venus@miss                2.480    18.164    18.128    18.428   -0.087     0.107     0.160 Miss [dT predicted]
Jupiter@miss             -0.056    18.681    18.999    20.106   -0.057     0.205     0.217 Miss
Jupiter@miss              2.267    19.688    19.583    19.416   -0.090     0.030     0.026 Miss [dT predicted]
Saturn@miss              -0.057    20.034    20.106    19.992   -0.048     0.060     0.080 Miss
Saturn@miss               4.952    16.237    16.307    17.446   -0.105   -11.759    -7.209 Total <-- DISAGREE (SE: Miss) [dT predicted]
disagreements: 8/18 (SP-6 measured 8: 3 knife-edge + 5 genuine)
```

Baseline confirmed: **8/18** disagreements.

### Disagreeing rows, annotated with epoch and target altitude at anchor

| row | epoch | dt_diff_s | geoMoon″ | ourMargin′ | seMargin′ | tgt alt° @anchor |
|-----|-------|-----------|----------|------------|-----------|------------------|
| Aldebaran | 2000.0 | −0.032 | 14.49 | −5.567 | −5.486 | **−1.5 (below)** |
| Aldebaran | 2015.1 | 0.099 | 9.90 | −0.889 | −0.875 | **−1.0 (below)** |
| Aldebaran | 2033.6 (pred) | 9.911 | 5.86 | −4.619 | −4.639 | **−1.0 (below)** |
| Regulus | 2007.0 | 0.262 | 14.95 | −3.653 | −3.092 | **−1.9 (below)** |
| Regulus | 2025.6 (pred) | 5.866 | 18.56 | −0.363 | −0.236 | **−0.9 (below)** |
| Spica | 2005.7 | 0.324 | 16.34 | −0.005 | +6.729 | +7.3 (above) |
| Spica | 2012.6 | 0.178 | 4.20 | −4.795 | −4.467 | **−1.8 (below)** |
| Saturn | 2024.3 (pred) | 4.952 | 16.24 | −11.759 | −7.209 | **−2.3 (below)** |

Altitude computed from the fixture's own SE topocentric target RA/Dec, observer lat/lon, standard GMST
(ΔT≈64 s, no refraction; ±~0.5° accuracy). **7 of 8 disagreeing rows have the target 0.9–2.3° below the
observer's horizon at the event.** The one exception (Spica 2005.7) has `ourMargin = −0.005′` — dead on the
graze limb, i.e. a genuine knife-edge.

## Probe 1 — ΔT / UT1 (timing)

Rule: timing explains a row iff `0.25 × |dt_diff_s| ≳ seMargin′`.

- **Non-predicted disagreeing rows** (epochs ≤ 2020): `|dt_diff|` = 0.032–0.324 s → `0.25×|dt_diff|` =
  0.008–0.081″ ≈ ≤0.08′ of track shift. That is <2% of the multi-arcmin `seMargin`s. **Timing does not
  explain these rows.** Our ΔT matches SE's to sub-second at these epochs.
- **Predicted rows** (2024–2033, past our observed ΔT node at 2020-01-01, correctly flagged `[dT predicted]`):
  `dt_diff` = 4.9–9.9 s → 1.2–2.5″ ≈ 1.2–2.5′ of track shift. This is a *partial* contributor on those rows
  only, and it is **expected and unfixable**: we extrapolate ΔT past our observed table while SE uses its own
  polynomial — the divergence is inherent to being past observed data, not a bug. Even there it accounts for
  at most half the margin.
- **`unwrap_or(jd)` fallback** (`crates/pleiades-events/src/occult.rs:443`): this is on
  `ut1_jd_from_tt(jd).unwrap_or(jd)` (the UT1 sidereal-time rotation), which internally calls
  `pleiades_time::deltat::delta_t`. `delta_t` only returns `Err(CivilTimeError::StaleTimeData)` on a corrupted
  committed ΔT table (`crates/pleiades-time/src/deltat.rs:38,50,54`); it never errors by epoch (past 2050 it
  returns `Ok(_, Predicted)`, deltat.rs:93). Therefore the fallback is **structurally unreachable in practice**
  — the "silent fallback" suspicion resolves to dead code. All disagreeing rows resolve ΔT cleanly
  (`dt_diff` finite and small; predicted correctly flagged only after 2020).

**Verdict: ΔT is not the failing stage. Branch (a) rejected.**

## Probe 2 — ephemeris source (geocentric stage)

Moon baseline (re-verified):
```
Moon: n=50 max_lon=0.0001 arcsec  rms_lon=0.0000 arcsec  max_lat=0.0001 arcsec  ...
```
Our packaged Moon ≡ de440 to ~0.0001″. SE's corpus truth is Moshier (SEFLG_MOSEPH).

The `geoMoon"` residuals are 4–20″ (Moshier vs de440). The brief's branch-(c) test is `geoMoon″ ≈ seMargin′`
on the genuine rows. It **fails**: `geoMoon″` (14.5, 15.0, 16.3, 20.0…) is 2.4–4.8× the `seMargin′`
(3.1, 5.5, 6.7…) — it over-explains. More decisively, **`ourMargin′ ≈ seMargin′` on 6 of the 8 disagreeing
rows** (e.g. Aldebaran −5.567/−5.486, −4.619/−4.639; Regulus −3.653/−3.092; Spica −4.795/−4.467), matching to
~0.1–0.6′. Our refined min-separation geometry and SE's swe_calc geometry **agree**. So the 14–20″ Moon offset
manifests almost entirely as a small conjunction-**time** shift (along-track ~14″/0.5″·s⁻¹ ≈ 28 s), not an
impact-parameter (min-separation) shift — its cross-track component is <0.6′, consistent with the margin
agreement. It therefore does **not** flip the graze classification.

**Verdict: the ephemeris source difference is real but classification-neutral. Branch (c) rejected as the
cause of the disagreements.**

## Probe 3 — topocentric transform (geodesy / parallax / sidereal)

- `topoMoon″ ≈ geoMoon″` on **every** row (Δ < 1″). The topocentric transform adds essentially nothing to the
  our-vs-SE divergence — so the divergence does not enter in parallax/sidereal/geodesy.
- `sdMoon″` = 0.02–0.15″ (the `R_MOON_KM` 1737.4 vs SE `RMOON` 1738.15 difference); cannot explain multi-arcmin
  margins. Noted, negligible.
- Observer geodesy (`crates/pleiades-apparent/src/parallax.rs:25-36`) is textbook Meeus Ch. 11:
  `u = atan((b/a)·tanφ)`, `ρ·sinφ′ = (b/a)·sin u + (h/a)·sinφ`, `ρ·cosφ′ = cos u + (h/a)·cosφ`, using
  **geodetic** latitude. It passes the Meeus Palomar worked example (φ=+33.356°, H=1706 m → ρ·sinφ′=0.546861,
  ρ·cosφ′=0.836339). No geodetic/geocentric latitude mix-up.
- Sidereal-time convention (UT1-rotated GAST) is consistent, and any RA-only residual would show up as
  `topoMoon ≫ geoMoon` — which it does not.

**Verdict: the topocentric transform is sound. Branch (b: transform/apparent) rejected.**

## Root cause (Step-5 outcome) — classification / visibility semantics

No numerical stage explains the disagreements, because **our topocentric geometry matches SE's** (ourMargin ≈
seMargin, both deep-negative). The disagreement is between our **verdict** and SE's **verdict**:

1. All 18 corpus rows are SE `@miss` (`swe_lun_occult_when_loc` returned retflag 0 = "beyond the graze limit";
   their own `max_jd = −1`). The differential fixture therefore **borrows the @center/@graze sibling's
   conjunction time as the anchor** (`graze-diagnosis.csv` header; tool
   `tools/se-occultations-reference/src/main.rs`, `build_diagnosis_csv`).
2. At that borrowed anchor, **SE's own swe_calc** puts the star 0.9–5.5′ *inside* the Moon's disk for the miss
   observer (`seMargin′` negative on 6 rows). So SE's `swe_calc` geometry and SE's `when_loc` verdict are
   *internally inconsistent* at the anchor — proving the anchor is **not** the instant on which `when_loc`
   decided.
3. **7 of 8 disagreeing rows have the target 0.9–2.3° below the observer's horizon at the event.** SE's
   `swe_lun_occult_when_loc` reports no visible occultation (retflag 0) for a below-horizon graze; our
   `classify` (`crates/pleiades-events/src/occult.rs:236`, `sep < s_moon + s_tgt` at the observer's refined
   closest approach) has **no horizon/visibility gate**, so it reports the instantaneous disk overlap as Total.
4. The 8th row (Spica 2005.7) is above the horizon but has `ourMargin = −0.005′` — a genuine knife-edge on the
   graze limb; classification there is on the razor's edge and is not a "genuine" deep disagreement.

Corroborating: on the AGREEING rows, `seMargin` (anchor) and `ourMargin` (refined min) diverge wildly
(Regulus 17.143′ vs 0.175′; Spica 11.695′ vs 0.072′) — direct proof the anchor is not the closest-approach
instant and that `seMargin` is not SE's decision margin.

### Per-row quantitative account (the "genuine" set)

For each genuine disagreeing row, the margin is **not** explained by any numerical stage's residual — instead
our geometry reproduces SE's swe_calc margin, and SE's Miss is a visibility/contact-search verdict:

| row | seMargin′ | our vs SE geometry | ΔT contribution | ephemeris contribution | actual cause |
|-----|-----------|--------------------|-----------------|------------------------|--------------|
| Aldebaran 2000.0 | −5.49 | ourMargin −5.57 ≈ seMargin (match) | 0.008″ (nil) | time-shift only | tgt −1.5° below horizon |
| Aldebaran 2033.6 | −4.64 | −4.62 ≈ −4.64 (match) | 2.5′ (partial, predicted) | time-shift only | tgt −1.0° below horizon |
| Regulus 2007.0 | −3.09 | −3.65 ≈ −3.09 (match within 0.6′) | 0.07″ (nil) | time-shift only | tgt −1.9° below horizon |
| Spica 2005.7 | +6.73 | ourMargin −0.005′ (knife-edge; anchor≠min) | 0.08″ (nil) | time-shift only | knife-edge on graze limb |
| Spica 2012.6 | −4.47 | −4.80 ≈ −4.47 (match) | 0.04″ (nil) | time-shift only | tgt −1.8° below horizon |
| Saturn 2024.3 | −7.21 | −11.76 (deeper; predicted ΔT + anchor) | 1.2′ (partial, predicted) | time-shift + predicted ΔT | tgt −2.3° below horizon |

Every genuine row is accounted for by (i) our geometry matching SE's swe_calc, plus (ii) the target being
below the observer's horizon at the event — i.e. an SE `when_loc` visibility verdict, not a numerical residual.

## Verdict

**Failing stage: classification / visibility semantics — NOT (a) timing, NOT (b) transform/apparent, NOT (c)
ephemeris source.** This is the Step-5 fourth outcome: "the topocentric comparison itself is sound but the
classification threshold differs." Our purely geometric occultation classifier lacks the above-horizon
visibility gate that SE's `swe_lun_occult_when_loc` applies; at high geographic latitude the graze-boundary
events fall just below the observer's horizon (−0.9° to −2.3°), so SE reports no visible occultation while we
report Total.

**Fix branch for Task 8:** Disposition matches **(c)** — the ephemeris, transform, and ΔT are all correct and
agree with SE, so *do not* change any engine numerics. But the concrete action is a **classifier-level fix**
(closest to a "(b)-style" our-side logic change, in the classification layer, **not** the apparent-place
transform): gate the occultation classification / the SE-differential comparison on target altitude ≥ 0 (with
whatever refraction margin `when_loc` uses) to match `swe_lun_occult_when_loc`'s visibility semantics; and/or
formally document KNOWN GAP 3 as an inherent geometric-classify-vs-visible-`when_loc` methodology difference,
and correct the differential fixture's borrowed-anchor artifact (compare at the observer's own closest
approach, not the sibling conjunction). The knife-edge rows plus Spica 2005.7 remain out of scope.

## Anomalies / residual doubt

- **Knife-edge/genuine split shifted since SP-6.** SP-6 recorded 3 knife-edge (|seMargin| ≤ 1′) + 5 genuine
  (3.7–11.6′). Current numbers give **2** rows with |seMargin| ≤ 1′ (Aldebaran −0.875, Regulus −0.236) and
  **6** with |seMargin| > 1′ (range 3.1–7.2′). Total disagreements unchanged at **8/18**. The margins drifted
  slightly because this branch added the central axis-pierce refinement; the verdict is unaffected.
- **Visibility mechanism is strong circumstantial, not a direct SE-source read.** I did not instrument SE's
  `when_loc` internals; the conclusion rests on (i) the 7/8 below-horizon correlation, (ii) SE's internal
  swe_calc-vs-when_loc inconsistency at the anchor. My altitude calc uses approximate GMST/ΔT (±~0.5°); the two
  marginal rows (~−1.0°) could sit within uncertainty of the horizon, but 5 rows are clearly below (−1.5 to
  −2.3°) and the overall pattern is unambiguous.
- **Spica 2005.7** is above the horizon yet disagrees — but its `ourMargin` is −0.005′ (a true knife-edge), so
  it is not a genuine deep disagreement.
