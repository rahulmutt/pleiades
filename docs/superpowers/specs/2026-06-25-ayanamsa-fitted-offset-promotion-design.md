# Ayanamsa Fitted-Offset Promotion (Phase 6, slice 3)

Date: 2026-06-25
Status: design approved, pending implementation plan

## Goal

Promote **bucket 1** — the smooth, SE-coded ayanamsa modes that only failed the
*linear* anchor + IAU-2006 precession model — to `ReleaseGradeNumeric` via
committed cubic polynomials fit to Swiss Ephemeris **mean** ayanamsa
(`SEFLG_NONUT | SEFLG_NOABERR`) over 1900–2100. This is the exact mechanism
already proven in slice 1 (TrueChitra/TrueCitra) and slice 2 (true-star +
galactic families): the mean ayanamsa drops apparent-place periodicity, so it is
smooth and cubic-fittable to sub-arcsecond accuracy.

These modes failed slice 1's promotion not because they are unfittable, but
because the linear `anchor + precession` model is the wrong *shape* for them. A
cubic fit to SE's own output bounds them regardless of why the linear model
missed.

This continues the ayanamsa half of the Phase 6 target compatibility catalog
(`spec/compatibility-catalog.md`). Like the prior slices, it adds **evidence**
(corpus rows + numeric gate) and **promotes claim tiers**; it adds no new
ayanamsa computation path beyond the cubic-fit path that already exists.

## Background / current baseline

After slice 2 (fitted-family promotion, merged 2026-06-25) the gated set is **36**
modes:

- 21 `OffsetDefined` (4 original + 17 promoted) — documented epoch anchor +
  IAU-2006 precession drift, checked against SE; ceiling 3.0″.
- 6 `TrueStar` (TrueChitra, TrueCitra, TrueRevati, TruePushya, TrueMula,
  TrueSheoran) — committed cubic fit to SE mean ayanamsa (`truestar.rs`), 1.0″
  floor ceiling.
- 9 `Galactic` — committed cubic fit (`galactic.rs`), 1.0″ floor ceiling.

The remaining **23** modes are `DescriptorOnly`. They split into three buckets:

- **Bucket 1 — smooth SE modes that only failed the 3.0″ *linear* ceiling
  (this slice's target, ≤ 15):**
  - *Failed-offset (12):* KrishnamurtiVP291, LahiriVP285, ValensMoon, DeLuce,
    Hipparchus, BabylonianKugler1/2/3, BabylonianHuber, BabylonianAldebaran,
    BabylonianBritton, BabylonianEtaPiscium — have SE anchors but the
    `anchor + IAU-2006 precession` model misses.
  - *Anchorless (3):* Udayagiri, PvrPushyaPaksha, Sheoran — no fixed SE *offset
    anchor* for the linear model, but SE still computes them as a smooth
    function of time.
- **Bucket 2 — observational/topocentric/house Babylonians (stay deferred):**
  BabylonianTrueGeoc, TrueTopc, TrueObs, House, HouseObs, Sissy — depend on
  apparent place / observer / house position rather than a smooth function of
  time, so they are not cubic-fittable.
- **Bucket 3 — no distinct SE code (stay deferred):** DhruvaGalacticCenterMula,
  legacy GalacticEquator alias — nothing to fit against.

The cubic-fit machinery (`crates/pleiades-ayanamsa/src/{truestar,galactic}.rs`,
`tools/se-ayanamsa-reference fit`) already exists and is the obvious tool for
bucket 1. This work mirrors slices 1–2 and the house-catalog promotion: same
parse → checksum → manifest-drift → per-row-residual-vs-ceiling structure, same
"evidence promotes the claim" model.

## Scope

**In scope (≤ 15 candidate modes), promoted via cubic fit:**

- Failed-offset: KrishnamurtiVP291, LahiriVP285, ValensMoon, DeLuce, Hipparchus,
  BabylonianKugler1, BabylonianKugler2, BabylonianKugler3, BabylonianHuber,
  BabylonianAldebaran, BabylonianBritton, BabylonianEtaPiscium.
- Anchorless: Udayagiri, PvrPushyaPaksha, Sheoran.

Final membership is confirmed **empirically during sourcing** (§1), exactly as in
slice 2: a mode for which SE has no usable `SE_SIDM` code, or that a cubic cannot
bound below the measured ceiling, is moved to the deferred set with its measured
worst residual rather than forced into a gated class. The anchorless three are
likely fittable (SE computes them), but the gate decides, not assertion.

**Out of scope — stay deferred (genuine known gaps):**

- Observational/topocentric/house Babylonians (bucket 2): BabylonianTrueGeoc,
  TrueTopc, TrueObs, House, HouseObs, Sissy.
- No-SE-code modes (bucket 3): DhruvaGalacticCenterMula, legacy GalacticEquator.

**Also out of scope:** native sidereal backend output, changing the holdout grid
or the precession model, the already-gated `OffsetDefined`/`TrueStar`/`Galactic`
modes (not re-touched), and the compatibility-profile *version* bump (content
updates each slice; the version bump is deferred until the full ayanamsa family
lands — see §7).

## Design

### 1. Fit sourcing & provenance

For each in-scope candidate, fit `ayan_deg(T) = c0 + c1·T + c2·T² + c3·T³`,
`T = (jd − 2451545)/36525`, to SE mean ayanamsa over 1900–2100 using the existing
`se-ayanamsa-reference fit` subcommand (same `MEAN_IFLAG`, same SE version
2.10.03 already pinned in the corpus manifest).

- Coefficients are committed to a new sibling module `fitted_offset.rs`,
  symmetric with the existing `truestar.rs`/`galactic.rs` per-family split.
- A committed comment block records, per mode, the `SE_SIDM` code and SE version,
  so provenance is explicit and re-derivable.
- Cross-check by construction: the `se-ayanamsa-reference` tool generates the
  corpus reference rows from the **same SE version**, so committed coefficients
  and reference values share their origin.

### 2. Empirical classifier + ceiling policy

Promotion is **measured, not asserted**:

1. Add each candidate's `(name, SE_SIDM)` pair to the tool's `MODES` table.
2. For each mode, `sidereal_offset` runs the cubic-fit path; the residual vs the
   SE reference is measured per row (`wrap_arcsec`) over the 10-JD holdout grid.
3. **Pass** (every row ≤ ceiling) → promote (`claim_tier = ReleaseGradeNumeric`,
   class mapped in `thresholds.rs`). **Fail** (any row > ceiling) → defer,
   recorded in a deferral table with the measured worst residual.

**Ceiling policy:** keep the established convention — `ceil(measured_max × 2)`
with a 1.0″ floor — measured over the modes that actually pass into the new
class. Cubics over 200 yr of precession-smooth ayanamsa land at ~0.01–0.2″, so
this almost certainly rests on the 1.0″ floor; it rises transparently if measured
higher.

**Sub-family contingency:** if a subset (most likely the Babylonian group)
clusters at a distinctly higher-but-bounded residual, it gets its own mode-class
+ ceiling rather than inflating `FittedOffset` — the same per-formula-family
pattern the house gate and slice 2 use. Default is a single class. If a group
cannot be bounded at all, it defers.

### 3. Classification (Approach A)

- Add one arm `FittedOffset` to `AyanamsaModeClass` in `thresholds.rs`, with its
  measured ceiling in `ayanamsa_mode_ceiling` and the passing modes in
  `ayanamsa_mode_class`.
- The existing `OffsetDefined` class (3.0″, *linear* anchor + precession) and its
  invariant — all 17 promoted offset modes pass via the linear model — are left
  untouched. The newly cubic-fitted modes do **not** join `OffsetDefined`:
  folding them in would conflate two mechanisms under one ceiling and overstate
  the cubic modes' real (~0.01″) accuracy under the linear 3.0″ budget.

### 4. Code, tool & corpus changes

- **Module layout** (`crates/pleiades-ayanamsa/src/`): add `fitted_offset.rs`
  holding the committed cubic coefficients + provenance. `lookup.rs::sidereal_offset`
  routes these modes to it the way it already routes star modes to `truestar.rs`
  and galactic modes to `galactic.rs`.
- **`thresholds.rs`**: add the `FittedOffset` arm, its ceiling, and the promoted
  modes to `ayanamsa_mode_class`.
- **Tool** (`tools/se-ayanamsa-reference/src/main.rs`): extend the `MODES` table
  with the in-scope `(name, SE_SIDM)` pairs; `fit` emits the coefficients to
  paste in. No algorithm change.
- **Corpus** (`ayanamsa.csv`): regenerate with 10 holdout rows per promoted mode
  appended at the existing holdout JDs; bump `manifest.txt` `rows=` and
  `checksum=` (FNV-1a-64 via `pleiades_apparent::fnv1a64` — the project uses a
  **non-canonical FNV prime**; recompute with the in-repo function, not stock
  FNV).
- **Gate** (`ayanamsa_validation.rs`): the completeness list, `mode_for_code`,
  and `ayanamsa_mode_class` extend mechanically to the promoted set. Because
  promoted modes join the completeness list, a *missing* row for a promoted mode
  also fails closed.

### 5. Claims-surface alignment

Every promotion stays consistent across the four surfaces `compat-claims-audit`
checks bidirectionally:

- catalog descriptor `claim_tier` → `ReleaseGradeNumeric`;
- SE numeric-gate evidence (corpus row present + passing);
- compatibility profile (`pleiades-compatibility-profile`) — content updated,
  version bump deferred (§7);
- README prose ("N release-claimed ayanamsa modes pass…") and PLAN.md Phase 6
  note → new gated count and shrunken deferred set.

### 6. Error handling

Fail-closed, exactly as today — each aborts the gate:

- checksum mismatch, manifest drift (rows / completeness),
- malformed row, unknown mode code,
- calculation returned `None` (includes out-of-fit-window: identical 1900–2100
  fail-closed boundary as `truestar.rs`/`galactic.rs`, no new extrapolation
  surface),
- ceiling exceeded.

Deferred modes keep their `None` class, stay on the legacy path, and stay out of
release claims (no overclaim possible).

### 7. Testing

- `gate_passes_over_committed_corpus`: updated `modes_checked` count; gate green
  over the expanded corpus.
- Per-mode residual assertions for the promoted set.
- Non-vacuous ceiling-exceeded test (already present) retained.
- A test asserting the **still-deferred set (buckets 2 & 3) stays
  `DescriptorOnly`** — guards against accidental claim broadening.
- `checksum_drift_fails_closed` continues to assert the committed checksum equals
  `fnv1a64(CORPUS_CSV)`.
- `compat-claims-audit`, `release-smoke`, and `release-gate` re-run green.

### 8. Compatibility profile versioning

Profile **content** (the release-claimed ayanamsa set) updates this slice,
because the overclaim audit enforces profile↔evidence agreement. The profile
**version** bump is deferred until the full ayanamsa family (all slices) lands,
to avoid per-slice version churn. This is recorded so the deferral is
intentional, not forgotten.

## Acceptance criteria

- Every in-scope mode that passes the gate is `ReleaseGradeNumeric`, present and
  passing in the committed corpus, and consistent across catalog / profile /
  README / audit.
- Every in-scope mode that fails is recorded in the deferral table with its
  measured worst residual and remains `DescriptorOnly`.
- All out-of-scope modes (buckets 2 & 3) remain `DescriptorOnly`.
- `release-gate` is green; `compat-claims-audit` passes bidirectionally.
- PLAN.md reflects the new gated count and the remaining deferred set.

## Open questions

None blocking. The only contingent decisions — a sub-family ceiling for a
higher-but-bounded Babylonian subset, and final membership of any anchorless mode
whose SE code proves unusable — are resolved empirically during implementation
from the measured residuals and SE SIDM-code availability.
