# Civil-Time Conversion (Phase 4 sub-project) — Design

Status: approved design, ready for implementation planning.
Date: 2026-06-21
Phase: 4 — Request-Mode Semantics (first of the independent sub-projects)

## Summary

Add built-in civil-time conversion to `pleiades`: callers supply a civil
calendar datetime tagged UTC or UT1 and receive a Terrestrial Time (TT) or
Barycentric Dynamical Time (TDB) `Instant` — the time scales the first-party
backends accept — together with typed provenance describing how the conversion
was made and how trustworthy it is.

This **reverses the current deliberate non-goal**. Today the workspace models no
Delta-T, leap seconds, DUT1, or TDB periodic terms: `Instant` carries a tagged
`TimeScale` and only offers *caller-supplied* mechanical retagging helpers, and
the `utc-convenience-policy-summary` / `delta-t-policy-summary` surfaces document
that built-in conversion is out of scope. This sub-project implements the modeled
path and updates those policy surfaces to the now-supported posture.

The work is additive: the existing caller-supplied retagging contract
(`Instant::tt_from_utc`, `TimeScaleConversion`, the CLI `--tt-from-utc-offset-seconds`
family, etc.) stays untouched for advanced callers. The modeled path is a parallel
entry point distinguished by the presence of conversion provenance.

## Goals

- Convert civil datetimes (Gregorian, UTC or UT1) to TT/TDB across the full
  packaged window (1900–2100 CE).
- Use leap-second-exact conversion where UTC is well-defined, a Delta-T model
  elsewhere, and a deterministic TT↔TDB periodic term throughout.
- Never silently present a predicted conversion as observed-grade: every result
  carries a typed quality marker (`Exact` / `Observed` / `Predicted`).
- Keep all data tables checksum-pinned and behind a fail-closed freshness gate,
  consistent with the `validate-corpus` discipline.
- Satisfy the Phase 4 exit criteria for an implemented request mode: validation
  fixtures, rustdoc/API examples, CLI coverage, backend/metadata surfacing, and
  release-profile/policy entries.

## Non-goals (YAGNI)

- DUT1 modeling / UTC→UT1 prediction. `|UTC − UT1| < 0.9 s`; the difference is
  folded into the Delta-T fallback rather than modeled separately.
- Timezone / local-civil handling. Callers convert local time to UTC or UT1
  before calling.
- Leap-second *smearing* variants.
- Apparent-place corrections, topocentric body positions, and native sidereal
  backend output — these remain separate Phase 4 sub-projects with their own
  spec → plan → implementation cycles.

## Decisions (locked during brainstorming)

1. **Full civil bridge** — leap-second-exact UTC, Delta-T UT1/civil, and TT↔TDB.
2. **Pinned Delta-T table + documented extrapolation** (not closed-form-only).
3. **Calendar + scale as two independently testable layers.**
4. **New `pleiades-time` crate** (published `0.2.x`, depends on `pleiades-types`).
5. **Tiered conversion with a typed quality marker and a hard outer horizon.**
6. **Additive parallel path** integrating with — not replacing — the existing
   caller-supplied retagging contract.

## Architecture

New crate `pleiades-time`, depending only on `pleiades-types` for `Instant`,
`JulianDay`, and `TimeScale`. Consumed by `pleiades-core` and the CLI; direct
`pleiades-backend` callers can use it too.

| Module | Responsibility |
| --- | --- |
| `calendar` | Proleptic-Gregorian `CivilDateTime` ↔ `JulianDay` (Meeus algorithm), with field-range validation and a round-trip inverse. |
| `leap` | Checksum-pinned leap-second table (`TAI − UTC`, 1972→present), a `valid_through` stamp, and a `tai_minus_utc(jd_utc)` lookup. |
| `deltat` | Checksum-pinned observed `ΔT` table plus a documented extrapolation formula; `delta_t(jd) -> (f64, DeltaTQuality)`. |
| `tdb` | Deterministic `TT ↔ TDB` periodic term (USNO/Fairhead approximation, sub-millisecond). |
| `convert` | Orchestrator: civil input + target scale → tagged `Instant` + `ConversionProvenance`. |
| `error` | `CivilTimeError` enum with `summary_line()` / `Display`, matching repo convention. |
| `policy` | Typed `CivilTimePolicySummary`, plus the updated `utc-convenience` / `delta-t` posture wording. |

## Conversion algorithm

All paths output TT or TDB. The orchestrator chooses the path from the source
scale and the epoch, and tags the result with a quality marker:

- **UTC, inside the leap-second window** →
  `TT = UTC + (TAI − UTC) + 32.184 s`, then TDB via the periodic term.
  Quality **Exact** (no Delta-T model error; only the sub-ms TDB term).
- **UT1, inside the observed ΔT range** → `TT = UT1 + ΔT`, then TDB.
  Quality **Observed**.
- **UT1, or future UTC past the leap-second table** →
  `TT = civil + ΔT_extrapolated`, then TDB. Future leap seconds are unknowable
  and `|UTC − UT1| < 0.9 s`, so the extrapolated Delta-T is the principled
  estimate at the seam. Quality **Predicted** (flagged).
- **Outside the documented support window (default 1900–2100, aligned to the
  artifact)** → hard `CivilTimeError::BeyondHorizon`.

### Accuracy rationale

The Moon moves ~0.5″ per second of time, so a 1-second ΔT error is roughly a
0.5″ lunar-longitude error — enough to dominate the artifact's sub-arcsec
accuracy. For 1972→present the leap-second path is exact and sidesteps ΔT
entirely; ΔT error only enters for pre-1972 and future dates, where observed
values (pre-1972) are well-tabulated and future values are explicitly flagged
`Predicted`. The TDB−TT term is bounded at ~1.7 ms throughout.

### Data provenance & fail-closed gate

The leap-second and ΔT tables are committed with checksums, an `as-of` /
`valid_through` stamp, and source citations (IERS / USNO). A fail-closed audit
in the spirit of `validate-corpus` rejects stale or checksum-drifted tables
rather than converting with outdated data (`CivilTimeError::StaleTimeData`). A
`ConversionProvenance` value — path, quality, ΔT seconds, `TAI − UTC`, and source
citation — rides with every successful result, mirroring the per-backend
"validated / constrained / approximate / unsupported" claims discipline so a
`Predicted` conversion is never mistaken for an `Exact` / `Observed` one.

## Public API surface

`pleiades-time` core API (illustrative shapes; final signatures settled in the
implementation plan):

```rust
pub struct CivilDateTime { /* year, month, day, hour, minute, second: f64 */ }
impl CivilDateTime {
    pub fn to_julian_day(&self) -> Result<JulianDay, CivilTimeError>;   // calendar layer
    pub fn from_julian_day(jd: JulianDay) -> CivilDateTime;             // inverse, round-trip
}

pub enum DeltaTQuality { Exact, Observed, Predicted } // Exact = leap-second path, no ΔT model error

pub enum ConversionPath { UtcLeapSecond, Ut1DeltaT, FutureExtrapolated }

pub struct ConversionProvenance {
    pub path: ConversionPath,
    pub quality: DeltaTQuality,
    pub delta_t_seconds: Option<f64>,
    pub tai_minus_utc: Option<i32>,
    pub sources: &'static str,        // leap/ΔT table citation + as-of
}
impl ConversionProvenance { pub fn summary_line(&self) -> String; }

pub struct CivilInstant { pub instant: Instant, pub provenance: ConversionProvenance }

// Orchestrator — the primary entry point:
pub fn to_terrestrial(
    civil: CivilDateTime,
    source: TimeScale,   // Utc | Ut1
    target: TimeScale,   // Tt | Tdb
) -> Result<CivilInstant, CivilTimeError>;

// Thin wrappers:
//   tt_from_utc_civil(), tdb_from_utc_civil(), tt_from_ut1_civil(), tdb_from_ut1_civil()
```

The result is a `CivilInstant` (tagged `Instant` + provenance), not a bare
`Instant`, so the modeled path is self-describing and distinguishable from a
caller-supplied retag.

### Integration (additive)

- **`pleiades-core`**: `ChartRequest::from_civil(civil, source, target, …)` builds
  a request through `pleiades-time`, stashing provenance so `observer_summary()`
  and snapshot summaries can render the conversion line. Mirrors the existing
  `with_instant_*` builders rather than replacing them.
- **`pleiades-backend`**: direct callers reach `pleiades-time` to obtain an
  `EphemerisRequest` instant — symmetric with how every request-policy surface is
  mirrored at both the façade and direct-backend layers.
- **`pleiades-cli chart`**: new `--civil "1990-05-15T08:30:00"` plus
  `--civil-scale utc|ut1` flags (target defaults to TT; `--civil-target tdb`
  opt-in). Output reports the conversion path, ΔT / leap values, and quality.
  Combining `--civil` with the manual `--tt` / `--utc` offset flags is rejected so
  the input is unambiguous, consistent with the existing repeated-flag rejection.

### Error type

`CivilTimeError` with `summary_line()` / `Display`, fail-closed:

| Variant | Cause |
| --- | --- |
| `InvalidCivilDate { field }` | month/day/hour/etc. out of range, or non-finite second |
| `UtcBeforeLeapEpoch` | UTC-tagged date before 1972 (guidance: tag as UT1) |
| `BeyondHorizon { jd, window }` | outside the documented support window → hard fail |
| `UnsupportedScale { source, target }` | source not UTC/UT1, or target not TT/TDB |
| `StaleTimeData { kind, as_of }` | leap / ΔT table failed the freshness / checksum gate |
| `NonFiniteOffset` | computed offset not finite (defensive) |

## Testing & validation

Committed fixtures, in the repo's pinned-reference style:

- **Anchor epochs**: J2000 (`2000-01-01 12:00:00 TT` ↔ JD 2451545.0); known
  `TT − UTC` checkpoints at each leap-second boundary (e.g. 2017-01-01 →
  37 s + 32.184 s).
- **ΔT spot values** from IERS/USNO (e.g. ΔT(1900) ≈ −2.8 s, ΔT(1950) ≈ 29.1 s,
  ΔT(2000) ≈ 63.8 s), asserted within a documented tolerance, with the quality
  flag checked (`Observed` vs `Predicted`).
- **TDB − TT** bounded ≤ ~1.7 ms across the window.
- **Calendar round-trip**: `CivilDateTime → JulianDay → CivilDateTime` identity
  across the window, including proleptic edge cases and leap-day boundaries.
- **Fail-closed tests**: pre-1972 UTC → `UtcBeforeLeapEpoch`; out-of-window →
  `BeyondHorizon`; stale / checksum-drifted table → `StaleTimeData`; bad calendar
  fields → `InvalidCivilDate`.
- **End-to-end cross-check**: convert a civil instant → TT → query the packaged
  backend → compare body longitude against the de440 corpus at that epoch,
  confirming the conversion error stays well under the artifact's sub-arcsec
  budget. This ties the new path to the existing corpus gate.

## Docs, policy & release integration

Per the plan-maintenance rule (keep README, release profiles, generated reports,
and PLAN aligned when public claims change):

- Rewrite `docs/time-observer-policy.md`: the "UTC convenience / Delta-T deferred
  non-goal" sections become the *supported* posture, describing the tiered model,
  the data windows, and the quality marker.
- Update the typed `utc-convenience-policy-summary` and `delta-t-policy-summary`
  CLI/report wording and their fail-closed drift checks to the new posture; add a
  `civil-time-policy-summary` and a table-freshness `time-data` audit alongside
  `validate-corpus`.
- Update `README.md` "current limits" and `PLAN.md` Phase 4: mark civil-time
  conversion done; the remaining Phase 4 items (apparent, topocentric, native
  sidereal) stay open.
- Add rustdoc examples on the new entry points and CLI `--help` coverage.

## Exit criteria (this sub-project)

- Civil UTC/UT1 datetimes convert to TT/TDB across 1900–2100 with the tiered
  Exact/Observed/Predicted quality marker and typed provenance.
- Out-of-window and stale-data cases fail closed with structured errors.
- Validation fixtures, rustdoc/API examples, CLI coverage, façade/backend
  surfacing, and updated policy/release wording are all in place.
- README, PLAN Phase 4, `time-observer-policy.md`, and the policy summaries are
  aligned with the implemented posture.
