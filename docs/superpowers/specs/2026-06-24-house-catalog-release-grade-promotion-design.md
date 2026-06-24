# House-Catalog Release-Grade Promotion (Phase 6)

Date: 2026-06-24
Status: design approved, pending implementation plan

## Goal

Promote the twelve `target`-tier house systems to `ReleaseGradeNumeric`, each
backed by Swiss-Ephemeris numeric-gate evidence, closing the house half of the
Phase 6 target compatibility catalog (`spec/compatibility-catalog.md`).

The target systems (by `swe_houses` letter code):

| Code | System | Formula family | Notes |
| --- | --- | --- | --- |
| D | Equal (from MC) | Equal | |
| N | Whole Sign / Equal from 0° Aries | Equal | |
| V | Vehlow Equal | Equal | |
| U | Krusinski-Pisa-Goelzer | GreatCircle | latitude-sensitive |
| Y | APC houses | GreatCircle | latitude-sensitive |
| S | Sripati | Quadrant | |
| F | Carter poli-equatorial | EquatorialProjection | |
| H | Horizontal / Azimuthal | GreatCircle | latitude-sensitive |
| G | Gauquelin sectors | Sector | **36 sectors**, latitude-sensitive |
| L | Pullen SD (sinusoidal delta) | Sector | |
| Q | Pullen SR (sinusoidal ratio) | Sector | |
| I | Sunshine (Makransky) | SolarArc | latitude-sensitive |

All twelve are **already computed** in `crates/pleiades-houses/src/systems/mod.rs`.
This work adds **evidence** (corpus rows + numeric gate) and **promotes the claim
tier**; it does not add or change any house-cusp formula.

## Background / current baseline

- The house numeric gate (`pleiades-validate::validate_house_corpus`) validates a
  60-row SE corpus (`crates/pleiades-validate/data/houses-corpus/cusps.csv`):
  5 fixtures × 12 baseline systems. Each cusp/angle residual is checked against a
  per-formula-family ceiling (`pleiades-houses::thresholds::house_family_ceiling`).
- The corpus CSV is checksum-pinned in `manifest.txt` (FNV-1a-64 via
  `pleiades_apparent::fnv1a64`; note the project uses a non-canonical FNV prime,
  so a stock FNV implementation will not reproduce it).
- The committed 60 rows carry an Astrolog 7.70 cross-check (context-only,
  non-gating); SwissEphemeris 2.10.03 remains the authoritative reference.
- `claim_tier` lives on each `HouseSystemDescriptor`. The overclaim audit
  (`pleiades-validate::claims`) enforces, **bidirectionally**, that
  `ReleaseGradeNumeric` ⟺ the SE corpus validates the system
  (`HouseCorpusReport::validated_systems()`), and that the compatibility profile
  and README prose agree.

### Constraints verified during design

- The `swisseph` crate's `HouseSystemKind` exposes every target letter code, so
  SE reference values are reachable for all twelve systems.
- **Gauquelin buffer hazard:** `swisseph::swe2::houses2` and
  `swisseph::swe::houses` both allocate a 13-element cusp buffer. For Gauquelin
  (`G`) the C library writes 37 doubles, so `houses2` truncates to 12 and the raw
  wrapper path would overflow. The generator MUST call
  `libswisseph-sys::swe_houses` directly with a `[f64; 37]` buffer for `G` and
  read sectors `[1..=36]`. `libswisseph-sys` is already a transitive dependency.
- **Lockstep requirement:** because the audit is bidirectional, adding corpus
  rows for a system without promoting its `claim_tier` produces a
  `DescriptorOnlyHasEvidence` error, and vice-versa. Corpus rows, tier
  promotion, profile counts, and README prose must change together.

## Architecture

The change spans five layers.

### 1. Generator — `tools/se-house-reference`

- Add the eleven standard (12-cusp) target systems to the `HSYS` mapping table so
  they are emitted into `cusps.csv`. Result: 5 fixtures × 23 systems = **115
  data rows** (up from 60). Regenerating with the same SE version
  (SwissEphemeris 2.10.03 via `libswisseph-sys 0.1.2`) and the same fixtures
  reproduces the existing 12 systems' rows byte-identically, preserving their
  Astrolog cross-check; the eleven new systems are appended.
- Add a **Gauquelin path** that calls `libswisseph-sys::swe_houses` with a
  `[f64; 37]` buffer for code `G` and writes a new `sectors.csv`
  (5 fixtures × 36 sectors).

### 2. Corpus schema — separate `sectors.csv` (variable-length)

Chosen approach (Approach A of three considered): keep the proven 12-cusp
`cusps.csv` schema and checksum path **fully intact and isolated**, and add a
sibling `sectors.csv` for sector-type systems.

- `sectors.csv` row shape (wide, variable-length):
  `chart_id,jd_ut,lat_deg,lon_deg,elev_m,system_code,n_sectors,s1,…,sN`
  where the parser reads `n_sectors` and then exactly that many sector values.
- `manifest.txt` gains a second slice line:
  `slice sectors file=sectors.csv role=sectors rows=<M> checksum=<u64>`.

Rejected alternatives: a tidy/long one-row-per-sector file (more flexible but a
different shape from the existing convention, verbose); widening `cusps.csv` to
36 sparse columns (pollutes the stable schema, forces a full regen + re-checksum
+ rewrite of the proven path, highest regression risk).

### 3. Gate — `crates/pleiades-validate/src/house_validation.rs`

- Extend `system_for_code` / `code_for_system` with the eleven new 12-cusp
  systems so they flow through the existing residual loop and into
  `validated_systems()`.
- Add `parse_house_sectors` (variable-length parser, fails closed on malformed or
  count-mismatched rows) and a second manifest slice parse for the sectors
  checksum/row-count.
- Add a sector-residual gate path that checks each Gauquelin sector against the
  `Sector`-family ceiling and adds `Gauquelin` to `validated_systems()`. Wire it
  into `validate_house_corpus()` so a single call gates both files (checksum, row
  count, completeness, residuals for both `cusps.csv` and `sectors.csv`).
- Extend the completeness check to require ≥1 corpus row for every promoted
  system (those that pass — see §5 fail-safe).

### 4. Thresholds — `crates/pleiades-houses/src/thresholds.rs`

The promoted systems activate three currently-unvalidated families — **GreatCircle**
(Horizon/APC/Krusinski), **SolarArc** (Sunshine), **Sector** (Pullen SD/SR,
Gauquelin) — and add members to already-validated families (Equal:
Vehlow/Equal-MC/Equal-Aries; Quadrant: Sripati; EquatorialProjection: Carter).

- Measure the maximum SE-vs-pleiades residual per family across the regenerated
  corpus, and set each family ceiling to `ceil(measured_max × 2)` with a 1.0″
  floor — the exact rule Phase 5 used for the baseline families.
- Update the threshold doc comment's measured-maxima table and the
  `house_thresholds_summary_for_report` line if family coverage changes.

### 5. Claims + profile + README (bidirectional consistency)

- Promote `claim_tier` → `ReleaseGradeNumeric` in the house catalog for every
  system that passes its family ceiling.
- Update the compatibility profile counts so overclaim Check B (profile) agrees.
- Update README house-system claim counts/prose so Check C (prose) agrees.
- Update `release-smoke` / `release-gate` expectations (e.g.
  `validated_systems().len()` rises from 12 toward 24) and any descriptor/summary
  tests that assert exact counts.

## Testing

- Unit tests for `parse_house_sectors`: well-formed row, short row, declared
  `n_sectors` not matching the trailing field count.
- Gate tests: regenerated `cusps.csv` row count (115), `sectors.csv` row count,
  Gauquelin sector residuals within the `Sector` ceiling, and
  `validated_systems()` containing the promoted set.
- Re-run `compat-claims-audit` plus the full numeric-gate set (house, ayanamsa,
  apparent, topocentric, corpus) under `release-smoke`.
- `cargo fmt` + `cargo clippy` clean; existing house-gate tests updated for the
  new counts.

## Fail-safe / known limits

- **Residual fail-safe:** any target system whose measured residual exceeds what a
  release-grade family ceiling can justify stays `DescriptorOnly` and is reported
  as an explicit known gap in the compatibility profile — never promoted with a
  loosened ceiling. The catalog must not be narrowed and claims must not be
  broadened beyond evidence.
- **Latitude-sensitive systems** (APC, Horizon, Krusinski, Sunshine) are gated
  only at the in-band corpus latitudes (0/40/55/66°). Extending strict
  high-latitude-rejection assertions to them is a follow-up, not this batch.
- `Albategnius` is an extra built-in outside the target catalog; it stays
  `DescriptorOnly`.

## Exit criteria

- Every promoted house system carries SE numeric-gate evidence and passes the
  overclaim audit (tier ⟺ evidence ⟺ profile ⟺ prose), or is recorded as a known
  gap.
- `cusps.csv` (12-cusp) and `sectors.csv` (Gauquelin) are checksum-pinned and
  gated by a single `validate_house_corpus()` entry point.
- README, compatibility profile, release gates, and the house catalog claim tiers
  are mutually consistent.
- Phase 6 house-system work is either complete for the target catalog or its
  remaining gaps are explicitly reported.
