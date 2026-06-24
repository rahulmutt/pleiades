# Ayanamsa Audit + Numeric Gate (Phase 5) — Design

Status: **approved — 2026-06-24**. Core design approved by the user (depth =
evidence-first, correct what's needed; corpus scope = release-claimed modes;
true-star modes implemented for real). Feasibility verified against the live
environment: Swiss Ephemeris runs here (the existing `se-house-reference` binary
emits cusps), `libswisseph-sys` 0.1.2 exposes `swe_set_sid_mode`, and the
high-level `swisseph` 0.1.1 crate wraps `swe_get_ayanamsa_ut`. Ready to hand to
writing-plans.

## Context

The just-merged work (`phase5-house-system-numeric-gate`) corrected the
baseline-11 **house systems** to match Swiss Ephemeris and built a fail-closed
`validate-houses` numeric-residual gate over a committed SE reference corpus
(`crates/pleiades-validate/data/houses-corpus/`), with per-formula-family
arcsecond ceilings set from measured residuals and a dev-only SE generator at
`tools/se-house-reference/`.

This slice applies the **same pattern to ayanamsas** — the remaining half of the
Phase 5 compatibility audit (`plan/stages/05-compatibility-and-release-readiness.md`:
"Audit ayanamsa epochs, offsets, formulas, aliases, near-equivalent variants,
provenance"; checklist `01-phase-gates.md`: "Ayanamsa reference epochs, offsets,
formulas, aliases, and provenance are audited for release-claimed entries").

There is currently **no numeric ayanamsa gate**. The ayanamsa crate has rich
descriptor / round-trip / metadata-coverage tests
(`pleiades-ayanamsa/src/{model,lookup,catalog}.rs`) but nothing that checks
computed offsets against Swiss Ephemeris.

### The computation today, and why a gate matters

`pleiades_ayanamsa::sidereal_offset` resolves a built-in to its
`AyanamsaDescriptor { epoch, offset_degrees }` and computes:

```
offset(t) = offset_degrees + centuries_since(epoch) × 1.396_971_277…°/century
```

— a **single global linear precession rate** applied to every mode
(`offset_from_components` in `lookup.rs`). Consequences the audit must surface and
the gate must bound:

- **Offset-defined modes** (Lahiri, Raman, Krishnamurti, Fagan/Bradley) track SE
  closely near their epoch but drift over 1900–2100 because real general
  precession in longitude is non-linear (a quadratic/accelerating term SE
  includes and the constant rate omits).
- **True-star modes** (True Chitra, True Citra) are *structurally* wrong: SE
  defines these by pinning the star Spica (Chitra) at 180°00′ sidereal longitude
  at all times, so the ayanamsa tracks Spica's actual position — not a fixed
  linear offset. The catalog currently gives `True Chitra` the **same**
  `(epoch=2435553.5, offset=23.245524743°)` as Lahiri, so it cannot match SE
  except coincidentally near that epoch.

## Decisions captured

| Topic | Decision |
| --- | --- |
| Depth | **Evidence-first, correct what's needed.** Build corpus + gate, measure per-mode residuals, then correct any release-claimed mode that exceeds a Swiss-Ephemeris-class ceiling; finally set tight measured ceilings and make the gate hard. |
| Reference source | **Swiss Ephemeris is the sole canonical gate**, mirroring `validate-houses`. (Ayanamsa is location-independent, so no second-engine cross-check is planned; the manifest still carries a `#CrossCheck-Engine:` field for parity, defaulting to `not-run`.) |
| Corpus scope | The **release-claimed modes**: baseline-5 (Lahiri, Raman, Krishnamurti, Fagan/Bradley, True Chitra) **+ True Citra** (release entry, near-equivalent of True Chitra). The remaining ~48 metadata-carrying built-ins stay descriptor-tested and are explicitly recorded as *not yet numerically gated* (no claim broadening) — parallel to the house gate leaving non-corpus families on generous, non-corpus-validated ceilings. |
| True-star modes | **Implement their real SE definition** (Spica-pinning for True Chitra/Citra) rather than constraining the claim. |
| SE provisioning | Dev-only generator `tools/se-ayanamsa-reference/`, **outside the published workspace** with its own lockfile (Constraint C1). Mirrors `tools/se-house-reference/`. |

## Scope & boundaries

**In:**
- SE reference corpus + `validate-ayanamsa` fail-closed numeric gate for the
  **6 release-claimed modes** (Lahiri, Raman, Krishnamurti, Fagan/Bradley,
  True Chitra, True Citra).
- Correction of those modes' computation where measured residuals exceed an
  SE-class ceiling: an SE-faithful precession treatment for the offset-defined
  modes, and real Spica-pinning for the true-star modes.
- A short audit record (which modes are gated, measured residuals, ceilings,
  precession treatment, and which built-ins remain not-yet-gated).

**Out (explicitly):**
- Numeric gating of the other ~48 metadata-carrying built-ins (kept on existing
  descriptor tests; recorded as not-yet-gated).
- Growing the ayanamsa catalog toward the full `SE_SIDM_*` set — Phase 6.
- Any public-API or `Ayanamsa` enum redesign; any new public ayanamsa.

## Architecture

### 1. SE reference generator — `tools/se-ayanamsa-reference` (dev-only)

Mirrors `tools/se-house-reference`: a standalone binary **outside the published
workspace** (own `Cargo.lock`, depends on `swisseph` 0.1.1 + `libswisseph-sys`
0.1.2), never a dependency of any shipping crate (Constraint C1). For each
in-scope mode it calls raw `swe_set_sid_mode(SE_SIDM_*, 0.0, 0.0)` then
`swisseph::swe::get_ayanamsa_ut(jd)` at each sampled instant and prints CSV.

`SE_SIDM_*` integer codes (passed raw; the crate exposes no named constants):
Fagan/Bradley = 0, Lahiri = 1, Raman = 3, Krishnamurti = 5, True Citra = 27,
True Revati = 28 (True Chitra is SE's True Citra family — exact code confirmed
during planning against the SE headers).

### 2. Committed corpus — `crates/pleiades-validate/data/ayanamsa-corpus/`

`{ayanamsa.csv, manifest.txt}`, mirroring the house corpus.

- **CSV schema:** `mode_code,jd_tt,se_ayanamsa_deg`. `mode_code` is the pleiades
  `Ayanamsa` variant name (e.g. `Lahiri`, `TrueChitra`) — strings, not SE
  integers, so the gate maps directly back to typed modes.
- **Instants:** span and slightly exceed the window to expose drift — J1900.0,
  1925, J1950.0, J2000.0, 2025, 2050, J2100.0, plus each mode's own reference
  epoch. ~8 instants × 6 modes ≈ 48 data rows (comparable to the 60-row house
  corpus).
- **Manifest:** `#Reference-Engine: SwissEphemeris <version>`,
  `#CrossCheck-Engine: not-run`, and a
  `slice ayanamsa file=ayanamsa.csv role=ayanamsa rows=<n> checksum=<u64>` line.
  Checksum via `pleiades_apparent::fnv1a64` over the CSV text — the project's
  established (non-canonical-prime) FNV-1a-64, same as the house gate.
- Reproducible from SE via the generator; a clean checkout stays SE-free and only
  **validates** the committed values (de440 / house precedent).

### 3. Ceilings — `pleiades-ayanamsa/src/thresholds.rs` (new)

Mirrors `pleiades-houses/src/thresholds.rs`. An `AyanamsaCeiling { offset_arcsec }`
keyed by **mode class** — `OffsetDefined` (Lahiri/Raman/Krishnamurti/Fagan-Bradley)
vs `TrueStar` (True Chitra/Citra). Ceilings set to `ceil(measured_max × 2)` over
the *corrected* implementation, with a 1.0″ floor, plus a release-facing
`ayanamsa_thresholds_summary_for_report()` line.

### 4. The gate — `validate_ayanamsa_corpus()` in `pleiades-validate/src/ayanamsa_validation.rs` (new)

Fail-closed, structured exactly like `house_validation.rs`:

- parse the CSV (fail-closed `MalformedRow` on any bad row);
- parse + verify the manifest (`MalformedManifest`, `ChecksumMismatch`,
  `ManifestDrift` on row-count/field drift);
- map `mode_code` → typed `Ayanamsa` (`UnknownModeCode` fail-closed);
- for each row recompute `pleiades_ayanamsa::sidereal_offset(mode, Instant::tt(jd))`
  and compare to `se_ayanamsa_deg` with a **circular** arcsecond residual
  (`wrap_arcsec`, reused/mirrored from the house gate);
- fail on `CeilingExceeded` (residual > the mode-class ceiling), `CalculationFailed`
  (mode returned `None`), or any parse/checksum/drift error.

Returns an `AyanamsaCorpusReport { rows_validated, modes_checked,
max_residual_arcsec, summary_line }`.

### 5. CLI wiring — `validate-ayanamsa`

Register the subcommand in `pleiades-validate/src/render/cli.rs` exactly like
`validate-houses` (and add it to the release-gate aggregate if houses is wired
there). Update the CLI help / snapshot tests that enumerate the `validate-*`
commands.

## The evidence-first correction step

1. Build the generator + corpus + gate with the gate computing and **printing**
   per-mode residuals (a measurement harness/soft assertion first, not yet a hard
   ceiling).
2. Read the measured residuals:
   - **Offset-defined modes:** if the constant-rate model drifts past an SE-class
     ceiling over the window, replace the single global rate in
     `offset_from_components` with an SE-faithful precession treatment (at minimum
     a quadratic term; the implementation picks whatever holds the ceiling). This
     touches shared `lookup.rs`, so all existing ayanamsa tests must stay green.
   - **True-star modes:** add a real computation path keyed on the mode that pins
     Spica at 180° sidereal (per SE's True Citra definition), instead of the
     borrowed-from-Lahiri linear offset. The catalog `(epoch, offset)` is retained
     only as descriptor metadata for these modes.
3. Set ceilings to `ceil(measured_max × 2)` over the corrected implementation and
   flip the gate to hard fail-closed.

## Data flow

```
in-scope modes × sampled instants
  └─(offline, SE present)─> se-ayanamsa-reference: set_sid_mode + get_ayanamsa_ut
        └─ write corpus CSV + manifest (fnv1a64 checksum + SE version)
              └─> committed corpus (source of truth, SE-free checkout)
                    └─(runtime gate: validate-ayanamsa)─>
                          recompute pleiades sidereal_offset per row
                          └─ compare to SE within mode-class ceiling (circular arcsec)
                                └─> pass / fail (fail-closed)
```

## Constraints

- **C1 — Pure-Rust workspace audit (hard).** `workspace-audit` fails closed on
  `links` assignments, `-sys` dependencies, `build.rs` scripts, and lockfile
  packages ending in `-sys`. An SE binding is FFI over libswe and appears as a
  `-sys` package. Therefore `tools/se-ayanamsa-reference` **must not** be a
  member of the published workspace and **must not** enter the workspace
  `Cargo.lock` — exactly the isolation already used by `tools/se-house-reference`.
  Shipping crates stay pure-Rust.
- **C2 — SE binding license.** Already handled for `tools/se-house-reference`
  (verification-only, non-shipping); the ayanamsa generator reuses the same
  binding under the same recorded posture (`tools/se-house-reference/LICENSE-NOTES.md`).

## Error handling / fail-closed conditions

- **Gate fails** on: missing/short corpus, checksum or manifest drift, malformed
  row, unknown mode code, calculation returning `None`, or residual over the
  mode-class ceiling.
- **Generation fails** only on SE-side problems; it is a maintainer-only,
  regeneration-time step. The committed corpus is the source of truth and a clean
  checkout validates without SE.

## Testing

Mirror `pleiades-validate/src/tests/validate_gates.rs`:

- corpus parses; full in-scope mode set present at every sampled instant;
- per-mode residuals within ceilings (post-correction);
- checksum-drift fails closed; manifest row-count drift fails closed; malformed
  row fails closed; unknown mode code fails closed;
- `validate-ayanamsa` CLI emits the summary line and correct exit code;
- gate runs with **no SE / no network** dependency (pure committed CSV);
- per-mode unit tests vs. a few inline SE goldens (esp. the true-star modes at
  two well-separated epochs to prove Spica-pinning, not a fixed offset).

## Audit record (deliverable)

A short audit note (in this spec's follow-up and/or a release-facing summary)
recording, for each in-scope mode: numerically-gated yes/no, measured max
residual, the mode-class ceiling, the precession treatment / definition used, and
the SE `(t0, ayan_t0)` reconciliation. It also lists the built-ins that remain
descriptor-only / not-yet-gated, so the compatibility profile never advertises an
ungated mode as fully validated.

## Open items (confirm during planning/implementation)

1. **Exact `SE_SIDM_*` code for "True Chitra" vs "True Citra".** Confirm against
   the SE headers whether pleiades' two entries map to one SE mode (TRUE_CITRA=27)
   or distinct ones (e.g. TRUE_CITRA vs TRUE_REVATI); reconcile the near-equivalent
   variants in the audit.
2. **SE `(t0, ayan_t0)` per mode.** SE's internal reference epoch/offset for each
   mode may differ from the catalog's stored `(epoch, offset_degrees)`; the corpus
   is SE's truth and the audit reconciles any mismatch (it may motivate updating
   the descriptor metadata too).
3. **Time scale of `jd` into SE.** `swe_get_ayanamsa_ut` takes UT; the pleiades
   path uses TT instants. Decide whether to call `swe_get_ayanamsa` (ET/TT) or
   apply ΔT so the corpus instants and the gate's `Instant` agree; encode the
   choice in the CSV column name (`jd_tt` vs `jd_ut`).
4. **Concrete per-mode-class ceilings** — set from observed residuals after the
   correction step.

## Audit findings (2026-06-24)

Implementation complete. All six release-claimed modes are numerically gated.
Findings recorded here; the gate passes on a clean checkout with no SE dependency.

### Gate corpus provenance

- Reference engine: Swiss Ephemeris 2.10.03, `swe_get_ayanamsa_ex` with
  `iflag = SEFLG_NONUT | SEFLG_NOABERR` (TT/ET, mean ayanamsa), via
  `tools/se-ayanamsa-reference` (excluded from the published workspace; C1 satisfied).
- Time scale: TT throughout (`jd_tt` column, matching `pleiades_types::Instant::Tt`).
- True-star corpus: regenerated as SE **mean** ayanamsa (`iflag = SEFLG_NONUT | SEFLG_NOABERR`)
  so the true-star signal is smooth and cubic-fittable. Gate semantics are
  apples-to-apples: pleiades' `sidereal_offset` for true-star modes returns a mean
  quantity (the committed cubic fit excludes nutation and aberration); the corpus
  reference is SE mean. SE's apparent signal (±17″ nutation / 18.6-year period;
  ±20″ annual aberration) is deliberately excluded from the reference.
  The corpus manifest records `#CrossCheck-Engine: not-run`.
- Corpus: 60 rows, 6 modes. fnv1a64 checksum (non-canonical prime, per project convention):
  `8858641012577117031`.

### Mode class: OffsetDefined (Lahiri, Raman, Krishnamurti, Fagan/Bradley)

- **Numerically gated:** yes.
- **Computation used:** descriptor epoch anchor + IAU-2006 general precession in
  longitude (quadratic + cubic terms; non-linear drift over the 1900–2100 window).
  Replaces the prior single global linear rate (`offset_from_components`) which
  drifted unacceptably over the window.
- **SE reference:** each mode uses SE's `(t0, ayan_t0)` as its anchor; Lahiri =
  SE_SIDM_LAHIRI (code 1), Raman = SE_SIDM_RAMAN (code 3), Krishnamurti =
  SE_SIDM_KRISHNAMURTI (code 5), Fagan/Bradley = SE_SIDM_FAGAN_BRADLEY (code 0).
  The IAU-2006 precession in longitude matches SE's non-linear drift over the window.
- **Measured max residual vs SE mean:** 0.828″ (over 1900–2100, across all four modes).
- **Mode-class ceiling:** 2.0″ (= ceil(2 × 0.828″), 1.0″ floor applied, result ≥ floor).

### Mode class: TrueStar (True Chitra, True Citra)

- **Numerically gated:** yes.
- **Computation used:** committed cubic polynomial fit to SE mean ayanamsa
  (NONUT | NOABERR) over 1900–2100. Coefficients committed in
  `crates/pleiades-ayanamsa/src/truestar.rs`. Not a linear offset; not a
  Lahiri/Spica-pinning approximation — a direct SE mean fit.
- **SE reference:** both modes map to SE TRUE_CITRA (code 27). True Chitra
  (`TrueChitra`) and True Citra (`TrueCitra`) are near-equivalents: identical corpus
  values, identical polynomial coefficients. The catalog entries differ only in
  display name and alias metadata; the gate validates both against code 27.
- **Measured max residual vs SE mean:** 0.0105″ (over 1900–2100, across both modes).
- **Mode-class ceiling:** 1.0″ (= 1.0″ floor, since ceil(2 × 0.0105″) < 1.0″).
- **Known validity limitation:** the cubic is fit over 1900–2100; outside this
  window the polynomial extrapolates and `sidereal_offset` should not be relied
  upon for true-star modes. The gate validates corpus rows only within the fit window.

### Open items resolved

1. **SE_SIDM code for True Chitra / True Citra:** both map to `SE_SIDM_TRUE_CITRA`
   (code 27). Confirmed against SE headers; reconciled above.
2. **SE `(t0, ayan_t0)` per mode:** offset-defined modes use SE's internal anchor
   directly (IAU-2006 precession propagates from it); the audit found no descriptor
   mismatch requiring a catalog update for the gated set.
3. **Time scale:** resolved as TT via `swe_get_ayanamsa` (ET/TT path, not `_ut`);
   corpus column is `jd_tt`; gate uses `Instant::Tt`.
4. **Concrete ceilings:** set from measurement — 2.0″ (OffsetDefined), 1.0″ (TrueStar).

### Not-yet-gated modes

The remaining ~48 metadata-carrying built-in ayanamsa variants in the pleiades
catalog are **not numerically gated**. They carry descriptor / round-trip /
metadata-coverage tests (`pleiades-ayanamsa/src/{model,lookup,catalog}.rs`) but no
Swiss Ephemeris residual check. No claim has been broadened: only the six
release-claimed modes listed above are attested to SE-class accuracy. Expanding
numerical gating to additional modes is Phase 6 work.
