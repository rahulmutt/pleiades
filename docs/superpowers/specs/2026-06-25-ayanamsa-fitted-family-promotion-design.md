# Ayanamsa Fitted-Family Promotion (Phase 6, slice 2)

Date: 2026-06-25
Status: design approved, pending implementation plan

## Goal

Promote the **smooth fitted family** of built-in ayanamsas to
`ReleaseGradeNumeric` via committed cubic polynomials fit to Swiss Ephemeris
**mean** ayanamsa over 1900–2100 — the same mechanism already proven for
TrueChitra/TrueCitra in slice 1. The mean ayanamsa
(`SEFLG_NONUT | SEFLG_NOABERR`) drops apparent-place periodicity (nutation,
annual aberration), so it is smooth and cubic-fittable to sub-arcsecond
accuracy.

This continues the ayanamsa half of the Phase 6 target compatibility catalog
(`spec/compatibility-catalog.md`). Like slice 1, it adds **evidence** (corpus
rows + numeric gate) and **promotes claim tiers**; it adds no new ayanamsa
computation path beyond the cubic-fit path that already exists.

## Background / current baseline

Slice 1 (offset-defined family promotion, merged 2026-06-25) brought the gated
set to **23** modes:

- 21 `OffsetDefined` (4 original + 17 promoted) — documented epoch anchor +
  IAU-2006 precession drift, checked against SE; ceiling raised to 3.0″.
- 2 `TrueStar` (TrueChitra, TrueCitra) — committed cubic fit *to* SE mean
  ayanamsa (`truestar.rs`), 1.0″ floor ceiling (measured max 0.011″).

The remaining 36 modes are `DescriptorOnly`. They split into:

- **Anchored offset modes that failed the 3.0″ linear ceiling** (LahiriVP285,
  KrishnamurtiVP291, Babylonian Kugler1/2/3/Huber/Aldebaran/Britton/EtaPiscium,
  Hipparchus, ValensMoon, DeLuce) — have SE anchors but the
  `anchor + IAU-2006 precession` model misses.
- **Anchorless offset modes** (Udayagiri, PvrPushyaPaksha, Sheoran) — no fixed
  SE offset anchor.
- **True-star fitted** (TrueRevati, TrueMula, TruePushya, TrueSheoran) — same
  shape as the already-gated TrueChitra/TrueCitra.
- **Galactic** (10 GalacticCenter*/GalacticEquator* modes + the legacy
  `GalacticEquator` alias mode).
- **Observational Babylonians** (BabylonianTrueGeoc/TrueTopc/TrueObs/House/
  HouseObs/Sissy) — depend on apparent place / observer / house position rather
  than a smooth function of time.

The cubic-fit machinery (`crates/pleiades-ayanamsa/src/truestar.rs`,
`tools/se-ayanamsa-reference fit`) already exists and is the obvious tool for
the **smooth** subset of this set.

This work mirrors slice 1
(`2026-06-25-ayanamsa-offset-defined-promotion-design.md`) and the house-catalog
promotion (`2026-06-24-house-catalog-release-grade-promotion-design.md`): same
parse → checksum → manifest-drift → per-row-residual-vs-ceiling structure, same
"evidence promotes the claim" model.

## Scope

**In scope (~15 modes), promoted via cubic fit:**

- True-star → existing `TrueStar` class: `TrueRevati`, `TrueMula`, `TruePushya`,
  `TrueSheoran`.
- Galactic → new `Galactic` class: `GalacticCenter`, `GalacticCenterRgilbrand`,
  `GalacticCenterMardyks`, `GalacticCenterCochrane`, `GalacticCenterMulaWilhelm`,
  `DhruvaGalacticCenterMula`, `GalacticEquatorIau1958`, `GalacticEquatorFiorenza`,
  `GalacticEquatorTrue`, `GalacticEquatorMula`, plus the legacy `GalacticEquator`
  alias mode.

Final membership is confirmed empirically during sourcing (§1): a mode for which
SE has no `SE_SIDM` code (plausible for `TrueSheoran` or the legacy
`GalacticEquator` alias), or that a cubic cannot bound, is moved to the deferred
set rather than forced into a gated class.

**Out of scope — stay deferred:**

- failed-offset modes (LahiriVP285, KrishnamurtiVP291, the Babylonian
  Kugler/Huber/Aldebaran/Britton/EtaPiscium set, Hipparchus, ValensMoon,
  DeLuce);
- anchorless modes (Udayagiri, PvrPushyaPaksha, Sheoran);
- observational/topocentric/house Babylonians (BabylonianTrueGeoc, TrueTopc,
  TrueObs, House, HouseObs, Sissy).

**Also out of scope:** native sidereal backend output, changing the holdout grid
or the precession model, and the compatibility-profile *version* bump (content
updates each slice; the version bump is deferred until the full ayanamsa family
lands — see §7).

## Design

### 1. Fit sourcing & provenance

For each in-scope mode, fit `ayan_deg(T) = c0 + c1·T + c2·T² + c3·T³`,
`T = (jd − 2451545)/36525`, to SE mean ayanamsa over 1900–2100 using the
existing `se-ayanamsa-reference fit` subcommand (same `MEAN_IFLAG`, same SE
version 2.10.03 already pinned in the corpus manifest).

- True-star coefficients are committed to `truestar.rs` (extending the existing
  module); galactic coefficients to a new sibling `galactic.rs` (symmetric with
  the per-family-class split and the existing `se_anchors.rs`/`truestar.rs`
  separation).
- A committed comment block records, per mode, the `SE_SIDM` code and SE
  version, so provenance is explicit and re-derivable.
- Cross-check by construction: the `se-ayanamsa-reference` tool generates the
  corpus reference rows from the **same SE version**, so committed coefficients
  and reference values share their origin.

### 2. Empirical classifier + ceiling policy

Promotion is **measured, not asserted**:

1. Add each in-scope mode's `(name, SE_SIDM)` pair to the tool's `MODES` table.
2. For each mode, `sidereal_offset` runs the cubic-fit path; the residual vs the
   SE reference is measured per row (`wrap_arcsec`) over the 10-JD holdout grid.
3. **Pass** (every row ≤ ceiling) → promote (`claim_tier = ReleaseGradeNumeric`,
   class mapped in `thresholds.rs`). **Fail** (any row > ceiling) → defer,
   recorded in a deferral table with the measured worst residual.

**Ceiling policy:** keep the established convention — `ceil(measured_max × 2)`
with a 1.0″ floor — recomputed per family:

- `TrueStar` ceiling recomputed over all 6 True-star modes (likely stays 1.0″;
  rises transparently if the new modes measure higher).
- `Galactic` ceiling measured over the 11 galactic modes.

**Sub-family contingency:** if a subset (most likely the "true" galactic-equator
projection modes) clusters at a distinctly higher-but-bounded residual, it gets
its own mode-class + ceiling rather than inflating the family's — the same
per-formula-family pattern the house gate uses. If a group cannot be bounded at
all, it defers.

### 3. Code, tool & corpus changes

- **Module layout** (`crates/pleiades-ayanamsa/src/`): extend `truestar.rs` with
  the 4 new True-star polynomials; add a sibling `galactic.rs` for the galactic
  polynomials. `lookup.rs::sidereal_offset` routes galactic modes to
  `galactic.rs` the way it already routes star modes to `truestar.rs`.
- **`thresholds.rs`**: add the `Galactic` arm to `AyanamsaModeClass`, its ceiling
  in `ayanamsa_mode_ceiling`, and the in-scope modes to `ayanamsa_mode_class`.
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

### 4. Claims-surface alignment

Every promotion stays consistent across the four surfaces `compat-claims-audit`
checks bidirectionally:

- catalog descriptor `claim_tier` → `ReleaseGradeNumeric`;
- SE numeric-gate evidence (corpus row present + passing);
- compatibility profile (`pleiades-compatibility-profile`) — content updated,
  version bump deferred (§7);
- README prose ("N release-claimed ayanamsa modes pass…") and PLAN.md Phase 6
  note → new gated count and shrunken deferred set.

### 5. Error handling

Fail-closed, exactly as today — each aborts the gate:

- checksum mismatch, manifest drift (rows / completeness),
- malformed row, unknown mode code,
- calculation returned `None` (includes out-of-fit-window: identical 1900–2100
  fail-closed boundary as `truestar.rs`, no new extrapolation surface),
- ceiling exceeded.

Deferred modes keep their `None` class, stay on the legacy path, and stay out of
release claims (no overclaim possible).

### 6. Testing

- `gate_passes_over_committed_corpus`: updated `modes_checked` count; gate green
  over the expanded corpus.
- Per-mode residual assertions for the promoted set.
- Non-vacuous ceiling-exceeded test (already present) retained.
- A test asserting the **still-deferred set stays `DescriptorOnly`** — guards
  against accidental claim broadening.
- `checksum_drift_fails_closed` continues to assert the committed checksum equals
  `fnv1a64(CORPUS_CSV)`.
- `compat-claims-audit`, `release-smoke`, and `release-gate` re-run green.

### 7. Compatibility profile versioning

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
- All out-of-scope modes remain `DescriptorOnly`.
- `release-gate` is green; `compat-claims-audit` passes bidirectionally.
- PLAN.md reflects the new gated count and the remaining deferred set.

## Open questions

None blocking. The only contingent decisions — a sub-family ceiling for a
higher-but-bounded galactic subset, and final membership of `TrueSheoran` / the
legacy `GalacticEquator` alias — are resolved empirically during implementation
from the measured residuals and SE SIDM-code availability.
