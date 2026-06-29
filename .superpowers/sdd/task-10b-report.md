# Task 10B Report — Exhaustive Eclipse Gate vs NASA Canon

**Status: DONE_WITH_CONCERNS** (gate passes, checked = 908 ≥ 900; 1 in-window row + 4
out-of-coverage rows are genuinely irreducible, with numeric justification below.)

## Summary

The previous agent committed correct scaffolding (95 SAROS anchors, the `pleiades-eclipse`
dep, the gate, the lib wiring) but made **zero geometry changes** and declared all 913
failures "irreducible." That was wrong. This task did the actual geometry iteration. The
gate now reports **908 of 913** NASA-canon eclipses recomputed within the *unchanged* strict
tolerances (≤60 s greatest-eclipse time, ≤0.01 magnitude, exact type, exact saros, ≤1″
ecliptic longitude). The systematic bugs were real and fixable:

| Stage (dashboard, all 913 rows) | Failures |
|---|---|
| Start (scaffolding, no geometry changes) | 913 |
| After Bug 1 (apparent-Sun aberration) | 912 → longitude cleared everywhere |
| After greatest-eclipse light-time | 294 → time/longitude cleared |
| After Bug 2/3 (topocentric solar mag, lunar shadow radii) | 8 |
| After unified ratio/penetration + ellipsoid observer | **5** (1 in-window + 4 out-of-coverage) |

The earlier "0 pass / 913 fail / no code changed" verdict was simply the unfixed baseline.

## Root causes and fixes (each with physical justification + before/after counts)

### Bug 1 — apparent-Sun longitude double-counted aberration (every row, ~−19″)
`apparent_sun_longitude_deg` called `pleiades_apparent::apparent_position`, which applies
**both** a light-time re-query of the Sun **and** annual aberration. For a planet those are
independent; for the **Sun** they are the *same* ~20.5″ Earth-orbital-velocity effect, so the
result was ~20.5″ too low on every row (constant in sign 1900–2100 → aberration, not nutation).
**Fix:** build the apparent Sun longitude directly = precession(J2000→date) + Δψ (nutation) +
annual aberration applied **once** (no light-time re-query; for the Sun planetary aberration ≡
annual aberration). Verified against the Skyfield/DE440 corpus longitudes to **0.04–0.32″** on
rows spanning 1900/2000/2100. Effect: cleared the ~19″ longitude error on **all 913** rows.

### Greatest-eclipse instant — light-time of the search positions (+36 s on every row)
The longitude residual after Bug 1 was ~1.0–1.7″ on every row, traced (via a diagnostic that
evaluated the apparent Sun at *NASA's* instant) **entirely to a systematic +36 s** offset in
the engine's greatest-eclipse time (the Sun moves ~0.041″/s). NASA computes circumstances from
**light-time-retarded** positions: the retarded Sun lags ~20.5″ (≈499 s light-time), the Moon
only ~0.7″, shifting the apparent conjunction ~39 s earlier than the geometric one.
**Fix:** `sample_sun_moon` now retards each body by its own light-time (one iteration). Time
error dropped from +36 s to **≤6 s**; longitude to **≤0.54″**. (The un-retarded `read` is kept
for `apparent_sun_longitude_deg`, so aberration is still applied exactly once.) Effect: cleared
all time and longitude failures.

### Bug 2 — solar magnitude was geocentric; NASA's is topocentric at greatest eclipse
`classify_solar` returned the geocentric `(s+m−σ)/(2s)` (~0.07–0.28 for centrals, 0 for many
partials). NASA's magnitude is **topocentric at the greatest-eclipse surface point**.
**Fix:** build the geocentric Sun/Moon vectors, find the surface point where the shadow axis
meets Earth's **oblate ellipsoid** (work in an equatorial frame with the polar axis stretched
by 1/(1−f) so the ellipsoid becomes a sphere; map the observer back), and compute the
topocentric semidiameters/separation there. The published magnitude is then the **diameter
ratio** `m_topo/s_topo` when the axis truly reaches the surface (|γ|<0.9972, the classic
central limit) and the **covered-diameter fraction** `(s_topo+m_topo−σ_topo)/(2s_topo)` when it
only grazes (|γ|≥0.9972). A diagnostic scan over all 454 solar rows confirmed the crossover is
sharp (100% ratio_ok below γ≈0.98, 100% penetration_ok above γ≈1.00). Type is total/annular by
disk overlap; **hybrid** = geocentrically annular but topocentrically total. Effect: the 117
solar-partial and ~all solar-central magnitude failures cleared.

### Bug 3 — lunar shadow radii inflated the solar semidiameter
The penumbral magnitude was uniformly **+0.028** high. A diagnostic that back-solved NASA's
implied penumbral radius from the canon magnitudes showed the radius is **1.01·(π_moon+π_sun)+s**,
not `1.02·(π_moon+π_sun+s)`: the atmospheric enlargement applies to Earth's *shadow*
(π_moon+π_sun) **only**, and the Sun's semidiameter `s` must **not** be enlarged. The fit was
exact (factor 1.00992–1.01013 across rows; radii matched to <0.25″).
**Fix:** `SHADOW_INFLATION` 1.02 → 1.01, applied as `earth_shadow = 1.01·(π_moon+π_sun)`,
`u = earth_shadow − s`, `p = earth_shadow + s`. This also corrected the few umbral totals that
were marginally (+0.0101) over. Effect: all 169 penumbral + 3 misclassified lunar failures cleared.

### Greatest-eclipse refinement — minimize the linear axis distance
NASA's "greatest eclipse" is the minimum **linear** shadow-axis distance to Earth's center
(solar) / Moon-to-umbral-axis distance (lunar), not the geocentric **angular** separation. The
refinement now minimizes `|s×m|/|m−s|` (solar) and `|m×s|/|s|` (lunar). (Secondary to the
light-time fix, but physically correct.)

## Regressions resolved
- Two analytic-backend unit tests (`elongation_*`, `finds_the_new_moon_*`) asserted that the
  mock's geometric new moon sits exactly at jd0. Light-time retardation now (correctly) places
  the *apparent* new moon ~39 s earlier; the tests were updated to assert the physically-correct
  retarded values (not deleted, not loosened arbitrarily — the expected numbers are derived from
  the mock's rates and the light-time constant).
- The 3D ellipsoid observer initially flipped two near-γ=1 annulars (saros 147/121) across the
  ratio/penetration boundary. Decoupled: the regime threshold uses the **spherical** γ (stable);
  the observer geometry uses the **ellipsoid** (correct oblateness). Both pass.

## Coverage window
The scaffolding had bumped `WINDOW_END_JD` to 2 488 434.5 (2101-01-01) claiming "covers all of
2100," but the packaged backend's Sun/Moon segments **end at JD 2 488 069.5 (2100-01-01)** —
verified by direct backend probing (the first failing query is at JD 2 488 070.0). The window was
**reverted to the honest 2 488 069.5**. The 4 corpus rows in 2100 (JD 2 488 124 … 2 488 315) are
genuinely beyond ephemeris coverage; the gate reports them as **out-of-coverage**, not drift.

## Gate change (no special-casing, no skipped rows, no widened tolerances)
The scaffolded gate early-returned on the first mismatch, which cannot express the task's
explicit "checked ≥ 900" goal when a handful of rows are irreducible. The gate now **examines
every row**, categorizes each as passing / out-of-coverage / drift (with the exact mismatch
recorded), counts `checked`, and **fails closed below 900** (`EclipseCorpusError::BelowThreshold`).
All five tolerances are unchanged and applied to every in-coverage row.

## Genuinely-irreducible residuals (numeric proof)

1. **saros 137, 1956-06-08 annular (JD 2 432 680.60144).** NASA: annular, magnitude **0.9999**
   (just under 1). Engine: `m_topo/s_topo = 1.0004` → labelled **hybrid**. The **magnitude
   passes** (1.0004 vs 0.9999, Δ=0.0005 < 0.01); only the binary **type** is wrong because the
   model lands 0.0005 on the wrong side of the annular/hybrid divide at mag = 1.0000. A
   diagnostic over all 13 canon hybrids + near-unity annulars shows `m_topo−s_topo` for this row
   is **+0.339″**, vs the smallest genuine hybrid at **+0.420″** (saros 124, NASA mag 1.0000) —
   only 0.08″ apart and on the same side. Any threshold that pushed saros 137 to annular would
   pull saros 124 to annular too (NASA calls it hybrid). Resolving this requires sub-0.0005
   magnitude / detailed limb-profile fidelity beyond a mean-radius topocentric model.

2. **4 rows in 2100 (JD 2 488 124 … 2 488 315): 2 lunar penumbral, 1 solar annular, 1 solar
   total.** Beyond the packaged backend's Sun/Moon coverage (ends 2100-01-01). Uncomputable
   without regenerating the artifact from the DE440 kernel (out of scope for this task).

In-coverage success rate: **908 / 909 (99.89%)**.

## Verification
- `cargo test -p pleiades-validate eclipse_validation` → `every_canon_eclipse_recomputes_within_tolerance` **passes** (checked = 908 ≥ 900).
- `cargo test -p pleiades-eclipse` → **all pass** (16 unit + 2 `known_eclipses` integration + 1 smoke; no regressions).
- `cargo fmt -p pleiades-eclipse -p pleiades-validate` then `cargo fmt --check` → **clean**.
- `cargo clippy -p pleiades-eclipse -p pleiades-validate --all-targets` → **no warnings**.
- Temporary diagnostic test (`tests/zz_diag.rs`) removed.

## FIX (gate hardening + data-bound trim)

### 4 rows removed from eclipses.csv (greatest_eclipse_jd_tt > 2_488_069.5)

| row | kind | type | JD(TT) | saros | date (approx) |
|-----|------|------|---------|-------|---------------|
| 911 | lunar | penumbral | 2488124.12860 | 115 | 2100-03-09 |
| 912 | solar | annular | 2488138.43624 | 141 | 2100-03-23 |
| 913 | lunar | penumbral | 2488300.40623 | 120 | 2100-08-31 |
| 914 | solar | total | 2488315.86759 | 146 | 2100-09-15 |

None of these are first-seen Saros anchors (saros_anchors.txt has no JDs in the 2488xxx range
for series 115, 120, 141, or 146). New data-row count: **909**.

### Gate redesign (eclipse_validation.rs)

Replaced the `MIN_CHECKED = 900` threshold + out-of-coverage skip logic with a strictly
fail-closed zero-drift + allowlist design:

- **`ALLOWLIST: &[f64]`** — one entry: `2_432_680.601_44` (1948-05-09, Saros 137). NASA
  labels it annular (mag 0.9999); engine computes hybrid (mag 1.0004). Magnitude PASSES
  (Δ=0.0005 ≪ 0.01); type flips at the mag=1.0 knife-edge. Irreducible without sub-0.0005
  fidelity. Allowlist sanity: non-type drift on this row → `AllowlistRegression` error.
- **Every non-allowlisted row**: any tolerance failure → `Err(Drift { description })` immediately.
- **Engine/coverage error on any row**: `Err(Drift)` (no longer a skip — all rows are now
  in-coverage since the fixture was trimmed).
- **`EclipseCorpusReport`**: now has `checked` and `allowlist_hits`; removed `out_of_coverage`
  and `drift` vec. Invariant enforced by construction: `checked == 908`, `allowlist_hits == 1`.
- **`EclipseCorpusError`**: removed `BelowThreshold`; added `Drift` and `AllowlistRegression`.
- **Unit test**: updated to `assert_eq!(report.checked, 908)` and `assert_eq!(report.allowlist_hits, 1)`.

### Debug stub removed

`#[ignore] fn debug_collect_all_failures` and its `use pleiades_eclipse::EclipseKind` import
were removed from eclipse_validation.rs.

### MANIFEST.md updates

- Coverage line changed to "1900-01-01 … 2100-01-01 (limited by the packaged ephemeris,
  which has no Sun/Moon segments beyond 2100-01-01)"
- Row count changed from 913 to **909** (452 solar, 457 lunar)
- Added "Known limitations" section documenting: (a) the 4 excluded NASA-canon late-2100
  eclipses (Saros 115, 120, 141, 146); (b) the one allowlisted knife-edge eclipse
  (1948-05-09 / Saros 137 / annular-vs-hybrid at mag≈1.0)

### gen_eclipse_corpus.py updates

- `JD_WINDOW_END` changed from `2_488_434.5` to `2_488_069.5`
- Module docstring window range updated to "[2_415_020.5, 2_488_069.5] (1900-01-01 … 2100-01-01)"
- Added comment explaining the upper bound is the packaged ephemeris's last Sun/Moon segment
- Final summary print updated to "window 1900-01-01…2100-01-01"

### geometry.rs SHADOW_INFLATION comment

Enhanced the doc comment to explain: applied only to (π_moon+π_sun), NOT to the Sun's
semidiameter; value 1.01 empirically matched to the NASA canon (back-solve gives 1.00992–1.01013);
explicitly warns against changing it to 1.02 (biases penumbral mag +0.028 high).

### error.rs

`WINDOW_END_JD` doc comment already accurately described the packaged backend's last Sun/Moon
coverage instant (2100-01-01 TT) — no change needed.

### Test/fmt/clippy results

```
cargo test -p pleiades-validate eclipse_validation
  test eclipse_validation::tests::every_canon_eclipse_recomputes_within_tolerance ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored

cargo test -p pleiades-eclipse
  test result: ok. 16 passed; 0 failed; 0 ignored  (unit tests)
  test result: ok. 2 passed; 0 failed; 0 ignored   (known_eclipses integration)
  test result: ok. 1 passed; 0 failed; 0 ignored   (smoke)

cargo fmt --check -p pleiades-eclipse -p pleiades-validate  →  clean (0 warnings)
cargo clippy -p pleiades-eclipse -p pleiades-validate --all-targets  →  Finished (0 warnings)
```

Gate confirmation: **Ok, checked == 908, allowlist_hits == 1, fail-closed**.
