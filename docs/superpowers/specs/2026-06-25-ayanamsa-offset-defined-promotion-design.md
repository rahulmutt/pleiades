# Ayanamsa Offset-Defined Family Promotion (Phase 6, slice 1)

Date: 2026-06-25
Status: design approved, pending implementation plan

## Goal

Promote every **anchored, non-star-pinned** built-in ayanamsa to
`ReleaseGradeNumeric` via the existing Swiss-Ephemeris numeric gate, by giving
each its authoritative SE anchor and classifying it `OffsetDefined`. The gate
promotes exactly those modes whose `anchor + IAU-2006 precession` model
reproduces SE within the offset ceiling over 1900–2100; modes that fail are
**deferred with a recorded reason** to the fitted-family spec (slice 2).

This closes the first slice of the ayanamsa half of the Phase 6 target
compatibility catalog (`spec/compatibility-catalog.md`). It adds **evidence**
(corpus rows + numeric gate) and **promotes claim tiers**; it adds no new
ayanamsa computation path beyond the SE-sourced anchors.

## Background / current baseline

- `pleiades_ayanamsa::sidereal_offset` has three paths:
  - **OffsetDefined** (gated): documented epoch anchor + IAU-2006 precession
    drift (`precession::precession_delta_degrees`) — first-principles, *checked*
    against SE.
  - **TrueStar** (gated): committed cubic fit *to* SE
    (`truestar::true_star_offset_degrees`).
  - **None** (ungated): legacy linear-rate path (`AyanamsaDescriptor::offset_at`
    → `offset_from_components`), not validated.
- The gate (`pleiades-validate::validate_ayanamsa_corpus`) validates a 60-row SE
  corpus (`crates/pleiades-validate/data/ayanamsa-corpus/ayanamsa.csv`): 6 modes
  × 10 holdout JDs. Each residual is checked against a per-mode-class ceiling
  (`pleiades_ayanamsa::thresholds::ayanamsa_mode_ceiling`): OffsetDefined ≤ 2.0″,
  TrueStar ≤ 1.0″.
- The corpus CSV is checksum-pinned in `manifest.txt` (FNV-1a-64 via
  `pleiades_apparent::fnv1a64`; **note the project uses a non-canonical FNV
  prime** — recompute checksums with the in-repo function, not a stock FNV).
- The SE reference tool `tools/se-ayanamsa-reference` currently hardcodes only
  the 6 gated modes (2 distinct `SE_SIDM` codes) and emits SE mean ayanamsa
  (`SEFLG_NONUT | SEFLG_NOABERR`) at 10 holdout JDs.
- Currently **6** modes are numerically gated; ~48 built-in variants are
  descriptor-only.

This work mirrors the just-completed house-catalog promotion
(`2026-06-24-house-catalog-release-grade-promotion-design.md`): same
parse → checksum → manifest-drift → per-row-residual-vs-ceiling structure, same
"evidence promotes the claim" model.

## Scope

**In scope** (anchored, non-star-pinned built-ins — attempted as `OffsetDefined`),
by `Ayanamsa` enum variant: `J2000`, `J1900`, `B1950`, `UshaShashi`, `DeLuce`,
`Yukteshwar`, `JnBhasin`, `DjwhalKhul`, `Udayagiri`, `LahiriIcrc`, `Lahiri1940`,
`LahiriVP285`, `KrishnamurtiVP291`, `BabylonianKugler1`, `BabylonianKugler2`,
`BabylonianKugler3`, `BabylonianHuber`, `BabylonianEtaPiscium`,
`BabylonianAldebaran`, `BabylonianBritton`, `Hipparchus`, `Sassanian`,
`Suryasiddhanta499`, `Aryabhata499`, `Aryabhata522`, `Suryasiddhanta499MeanSun`,
`Aryabhata499MeanSun`, `SuryasiddhantaRevati`, `SuryasiddhantaCitra`,
`ValensMoon`, `PvrPushyaPaksha`, `Sheoran`.

The **mean-Sun** variants (`Suryasiddhanta499MeanSun`, `Aryabhata499MeanSun`)
are *attempted* here but are the likeliest to fail the linear model; if they do,
they defer cleanly. Final membership is confirmed during anchor sourcing (§1): a
mode for which SE has no fixed offset anchor — e.g. `BabylonianHouse`, which
currently carries no epoch anchor — is moved to the deferred set rather than
forced into `OffsetDefined`.

**Explicitly deferred to slice 2 (fitted / star-pinned family):** all `True*`
modes (True Revati/Mula/Pushya/Sheoran), all `GalacticCenter*` and
`GalacticEquator*` modes, and the observational Babylonians
(`BabylonianTrueGeoc`, `BabylonianTrueTopc`, `BabylonianTrueObs`,
`BabylonianHouse`, `BabylonianHouseObs`, `BabylonianSissy`). These lack a fixed
offset anchor and/or are pinned to a star/galactic reference; they need a
cubic-fit, not an offset.

**Out of scope:** any fitted/star-pinned mode (slice 2), native sidereal
backend output, changing the holdout grid or the precession model, and the
compatibility-profile *version* bump (deferred until the full ayanamsa family
lands — see §7).

## Design

### 1. Anchor sourcing & provenance

Each in-scope mode gets its official `(t0, ayan_t0)` transcribed from Swiss
Ephemeris's `ayamsa[]` reference table (`swephlib.c`, SE **2.10.03** — the
version already pinned in the corpus manifest). These replace any guessed
catalog anchors; where an existing anchor already matches SE, it is a no-op.

- Anchors live in a new sibling module `crates/pleiades-ayanamsa/src/se_anchors.rs`,
  keeping SE-sourced reference data separate from the catalog tables.
- A committed comment block records, per mode, the SE table row and SE version,
  so the provenance is explicit and re-derivable.
- The descriptor's `epoch` / `offset_degrees` are populated from these anchors.
- Cross-check by construction: the `se-ayanamsa-reference` tool generates the
  corpus reference rows from the **same SE version**, so anchors and reference
  values share their origin.

### 2. Empirical classifier + ceiling policy

Promotion is **measured, not asserted**:

1. Add each in-scope mode's `(name, SE_SIDM)` pair to the tool's `MODES` table.
   No algorithm change — same `MEAN_IFLAG`, same 10-JD holdout grid.
2. For each mode, `sidereal_offset` runs the `OffsetDefined` path; the residual
   vs the SE reference is measured per row (`wrap_arcsec`).
3. **Pass** (every row ≤ ceiling) → promote (`claim_tier = ReleaseGradeNumeric`,
   class mapped in `thresholds.rs`). **Fail** (any row > ceiling) → defer,
   recorded in a deferral table with the measured worst residual.

**Ceiling policy:** keep the established convention — `ceil(measured_max × 2)`
with a 1.0″ floor — recomputed over the full promoted `OffsetDefined` set. Today
that is 2.0″ (measured max 0.828″). If the broader set raises the measured max,
the ceiling rises with it, transparently and on the record.

**Sub-family contingency:** if one group (most likely the mean-Sun siddhanta
modes) clusters at a distinctly higher-but-bounded residual, it gets its own
mode-class + ceiling rather than inflating the shared class — the same
per-formula-family pattern the house gate uses. If a group cannot be bounded at
all, it defers to slice 2.

### 3. Corpus & tool changes

- **Tool** (`tools/se-ayanamsa-reference/src/main.rs`): extend the `MODES` table
  with the in-scope `(name, SE_SIDM)` pairs. No other change.
- **Corpus** (`ayanamsa.csv`): regenerate with the promoted modes' rows appended
  (10 rows each at the existing holdout JDs); bump `manifest.txt` `rows=` and
  `checksum=` (FNV-1a-64 via `pleiades_apparent::fnv1a64`).
- **Gate** (`ayanamsa_validation.rs`): the completeness list, `mode_for_code`,
  and `ayanamsa_mode_class` extend mechanically to the promoted set.

### 4. Claims-surface alignment

Every promotion stays consistent across the four surfaces `compat-claims-audit`
checks bidirectionally:

- catalog descriptor `claim_tier` → `ReleaseGradeNumeric`;
- SE numeric-gate evidence (corpus row present + passing);
- the compatibility profile (`pleiades-compatibility-profile`) — content updated,
  version bump deferred (§7);
- README prose ("N release-claimed ayanamsa modes pass…").

PLAN.md's Phase 6 ayanamsa note is updated to the new gated count and the
remaining deferred set.

### 5. Error handling

Fail-closed, exactly as today — each aborts the gate:

- checksum mismatch, manifest drift (rows / completeness),
- malformed row, unknown mode code,
- calculation returned `None`,
- ceiling exceeded.

Because promoted modes join the completeness list, a *missing* row for a
promoted mode also fails closed. Deferred modes keep their `None` class, stay on
the legacy path, and stay out of release claims (no overclaim possible).

### 6. Testing

- `gate_passes_over_committed_corpus`: updated `modes_checked` count; gate green
  over the expanded corpus.
- Per-mode residual assertions for the promoted set.
- Non-vacuous ceiling-exceeded test (already present) retained.
- A test asserting the **deferred set is still `DescriptorOnly`** — guards
  against accidental claim broadening.
- `compat-claims-audit`, `release-smoke`, and `release-gate` re-run green.
- `checksum_drift_fails_closed` continues to assert the committed checksum equals
  `fnv1a64(CORPUS_CSV)`.

### 7. Compatibility profile versioning

Profile **content** (the release-claimed ayanamsa set) updates each slice,
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
- All deferred (slice-2) modes remain `DescriptorOnly`.
- `release-gate` is green; `compat-claims-audit` passes bidirectionally.
- PLAN.md reflects the new gated count and the remaining deferred set.

## Open questions

None blocking. The only contingent decision (a sub-family ceiling for the
mean-Sun siddhanta group) is resolved empirically during implementation from the
measured residuals.
