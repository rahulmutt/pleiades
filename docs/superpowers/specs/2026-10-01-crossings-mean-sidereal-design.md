# Longitude crossings in a mean place or a sidereal zodiac (issue #88) — design

**Status:** implemented (2026-10-01) ·
**Opened:** 2026-10-01 · **Issue:** #88 ·
**Crates:** `pleiades-events`, `pleiades-validate`, `pleiades-core` (compatibility
profile text only) ·
**Tool:** `tools/se-crossings-reference` (extended)

## Context

`EventEngine` finds longitude crossings in two frames, chosen by
`CrossingFrame` (`pleiades-events` `src/crossings.rs`): geocentric apparent
tropical of date, and heliocentric. Both are tropical, and the geocentric one
is apparent. A caller cannot ask when a body reaches a longitude in a mean
place or in a sidereal zodiac, although the chart layer offers both choices for
a single instant (`ChartRequest::apparentness`, `ChartRequest::zodiac_mode`).
Such a caller has to root-find over position queries and repeat the bracketing,
refinement and window handling the engine already has.

Decisions taken on 2026-10-01:

1. **Scope: the full geocentric matrix.** Place (apparent or mean) and zodiac
   (tropical or any sidereal ayanamsa) are independent choices on geocentric
   crossings. Heliocentric crossings stay tropical.
2. **Mean place is geometric, mean equinox of date.** The backend's geometric
   J2000 place precessed to date: no light-time, no aberration, no nutation.
   This is deliberately *not* what a `pleiades-core` mean chart reports; see
   "What already exists".
3. **Sidereal longitudes are nutation-free**, the Swiss Ephemeris convention:
   the longitude on the mean equinox of date minus the mean ayanamsa. A
   sidereal zodiac is fixed to the stars, so the wobble of the equinox does not
   move a body through it.
4. **Interface: a frame variant plus a zodiac carried beside it**
   (`CrossingReference`), so the change is additive and `CrossingFrame` keeps
   `Copy`, `Eq` and `Hash`.

### What already exists

- `pleiades-events` `src/ephemeris.rs` reads the backend's mean geocentric
  place (`read_mean_ecliptic`, `read_mean_ecliptic_with_motion`) and builds the
  apparent place of date (`geocentric_apparent_ecliptic`) and the heliocentric
  place (`heliocentric_j2000`, `heliocentric_of_date`).
- All first-party backends emit the **J2000** ecliptic at the backend boundary
  (FU-3, pinned by `validate-frame-consistency`).
- `pleiades_apparent::precess_ecliptic_j2000_to_date` precesses a J2000
  ecliptic place to the mean equinox and ecliptic of date;
  `pleiades_apparent::nutation::nutation` gives Δψ.
- `pleiades_ayanamsa::sidereal_offset(&Ayanamsa, Instant) -> Option<Angle>` is
  the **mean** ayanamsa: its fits and its validation corpus are Swiss Ephemeris
  with `SEFLG_NONUT | SEFLG_NOABERR`. `pleiades-events` does not depend on
  `pleiades-ayanamsa` today.
- `src/root.rs` scanners take a closure of the Julian day, so any quantity that
  varies with time can be evaluated at each trial instant.
- `src/position.rs` (`position_at`, issue #89) derives a speed as the base
  place's speed plus the differenced rate of a correction.
- `CrossingFrame` derives `Copy`, `Eq`, `Hash`; `Ayanamsa` and `ZodiacMode` are
  only `Clone + PartialEq`. `Crossing` and `EclipticPosition` are
  `#[non_exhaustive]`; `EventError` is not.

Two chart-layer behaviours were read in the code while designing this and are
**not** changed here (see "Follow-ups"):

- A mean chart stores the backend's J2000 place unchanged
  (`pleiades-core` `src/chart/mod.rs`), so `Apparentness::Mean` in a chart is a
  J2000 longitude, not a longitude of date.
- A sidereal chart subtracts the mean ayanamsa from whatever longitude it holds
  (`src/chart/sidereal.rs`): the true-equinox longitude for an apparent chart,
  which keeps nutation in the result, and the J2000 longitude for a mean chart.

## Public interface

All in `pleiades-events`.

### Types

```rust
#[non_exhaustive]
pub enum CrossingFrame {
    GeocentricApparentOfDate,
    Heliocentric,
    /// Geocentric geometric place in the mean ecliptic and equinox of date:
    /// no light-time, no aberration, no nutation.
    GeocentricMeanOfDate, // new
}

/// The frame and zodiac a longitude is measured in.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct CrossingReference {
    pub frame: CrossingFrame,
    pub zodiac: ZodiacMode,
}

impl From<CrossingFrame> for CrossingReference { /* zodiac: Tropical */ }

impl CrossingReference {
    pub fn tropical(frame: CrossingFrame) -> Self;
    pub fn sidereal(frame: CrossingFrame, ayanamsa: Ayanamsa) -> Self;
}
```

`CrossingReference` carries optional serde like its neighbours and is
re-exported from the crate root.

`Crossing` and `EclipticPosition` each gain `pub zodiac: ZodiacMode`.
`Crossing::target_longitude` is a longitude in that zodiac.

### Methods

`longitude_crossings_in_range`, `next_longitude_crossing`,
`previous_longitude_crossing`, `longitude_at` and `position_at` take
`reference: impl Into<CrossingReference>` where they take
`frame: CrossingFrame` today. `next_sun_crossing` and `next_moon_crossing` are
unchanged (apparent, tropical).

### Meaning of each geocentric combination

| Frame | Zodiac | Longitude | Swiss Ephemeris flags |
|---|---|---|---|
| `GeocentricApparentOfDate` | tropical | unchanged: apparent place, true equinox of date | default |
| `GeocentricMeanOfDate` | tropical | backend J2000 place precessed to the mean equinox of date | `TRUEPOS \| NOABERR \| NOGDEFL \| NONUT` |
| `GeocentricApparentOfDate` | sidereal | apparent longitude − Δψ − mean ayanamsa | `SIDEREAL` |
| `GeocentricMeanOfDate` | sidereal | mean-of-date longitude − mean ayanamsa | `SIDEREAL \| TRUEPOS \| NOABERR \| NOGDEFL` |

Latitude and distance are those of the frame; the zodiac changes only the
longitude. Nutation moves the equinox along the ecliptic and leaves the
ecliptic plane alone, so subtracting Δψ from a true-equinox longitude gives the
mean-equinox longitude exactly.

### Errors

`EventError` is not `#[non_exhaustive]`, so no variant is added. Both new
failures are `EventError::UnsupportedFrame { detail }`:

- `Heliocentric` with a sidereal zodiac;
- an ayanamsa for which `sidereal_offset` returns `None` (for example a custom
  ayanamsa without an epoch or offset).

Neither falls back to a tropical result. Existing errors are unchanged.

### Compatibility

- Every existing call compiles unchanged (`CrossingFrame: Into<CrossingReference>`)
  and returns the same bits. The 86 committed golden crossings do not move.
- Additive: a minor version of `pleiades-events`.
- With the `serde` feature, `Crossing` and `EclipticPosition` serialize one
  more field.

## Internals

### Module layout

- **`src/reference.rs`** (new; tests in `src/reference/tests.rs`). Holds
  `CrossingReference` and one crate-private function:

  ```rust
  pub(crate) fn ecliptic_in<B: EphemerisBackend>(
      backend: &B,
      body: &CelestialBody,
      reference: &CrossingReference,
      julian_day: f64,
  ) -> Result<(f64, f64, f64), EventError>; // lon°, lat°, dist AU
  ```

  It is the single place a frame and zodiac become a position, plus
  `pub(crate) fn check_supported(body, reference, julian_day)` for the up-front
  checks (heliocentric Sun/Moon, heliocentric sidereal, ayanamsa without an
  offset). The `match frame` blocks in `crossings.rs` (`longitude_deg`) and
  `position.rs` (`sample`) are replaced by calls into this module.
- **`src/ephemeris.rs`** gains `geocentric_mean_of_date_ecliptic`:
  `read_mean_ecliptic` followed by `precess_ecliptic_j2000_to_date`. No
  light-time re-query.
- **`Cargo.toml`** gains `pleiades-ayanamsa = { workspace = true }`. Both are
  domain crates below `pleiades-core`; no cycle is introduced.

### Forming a longitude

1. Compute the tropical `(lon, lat, dist)` for the frame with the existing
   functions or the new mean-of-date one.
2. Tropical zodiac: return it untouched. No arithmetic is applied, which is
   what keeps the two existing frames bit-identical.
3. Sidereal zodiac:
   - `GeocentricApparentOfDate`: subtract `nutation(jd).delta_psi_arcsec / 3600`;
   - subtract `sidereal_offset(ayanamsa, instant)` in degrees;
   - `rem_euclid(360.0)`.

`sidereal_offset` reads its Julian day as TT while the engine works in TDB. The
two differ by under 2 ms, which moves the ayanamsa by less than 10⁻⁹″; the TDB
day is passed through and a comment records why.

### Root-finding

The closure stays `wrap180(longitude(jd) − target)` and calls `ecliptic_in`, so
Δψ and the ayanamsa are evaluated at every trial instant; a tropical crossing
of a shifted target is not used. Step sizes, window clamps, the wrap-seam guard
and the settled-instant rule (#80/#81) are untouched. `check_supported` runs
once before scanning.

### `position_at`

The base-plus-correction pattern covers the new cases:

| Reference | Base place and speed | Corrected place |
|---|---|---|
| mean of date, tropical | backend J2000 place and its motion | precessed to date |
| apparent, sidereal | backend J2000 place and its motion | apparent − Δψ − ayanamsa |
| mean of date, sidereal | backend J2000 place and its motion | precessed − ayanamsa |

The ±0.5 day differencing of the correction picks up the precession, Δψ and
ayanamsa rates. `longitude_at` and `position_at` both read `ecliptic_in`, so
their longitudes stay bit-identical in every combination.

### Bodies and range

The mean frame accepts the same bodies as the apparent frame, including the
lunar points. It has no light-time re-query, so the range-start failure of
FU-17(c) does not affect it.

## Validation

### Swiss Ephemeris reference (`validate-crossings` Tier 2)

`tools/se-crossings-reference` and
`crates/pleiades-validate/data/crossings-corpus/` are extended; there is no new
gate.

- The CSV gains a `zodiac` column (`tropical` or an ayanamsa name) and the
  frame code `geo-mean`. The 86 existing rows become `tropical` with their
  reference and golden times unchanged to the bit. `manifest.txt` (row count,
  checksum, frame legend) is regenerated.
- New rows are located by bisection on `swe_calc` longitudes, as geocentric
  planets already are, with the flags in "Meaning of each geocentric
  combination" and `swe_set_sid_mode` for sidereal rows.
- Coverage, about 70 rows:
  - bodies: Sun, Moon, Mars (with the 2003 retrograde triple crossing), one
    slow planet;
  - epochs: three, spread over 1900–2100;
  - places: mean tropical, apparent sidereal, mean sidereal;
  - ayanamsas: one per computation class in `pleiades-ayanamsa`
    (`OffsetDefined` — Lahiri; `TrueStar`; `Galactic`; `FittedOffset`).
- Ceilings follow the corpus rule: measured per group and set to
  `ceil(1.4 × group max)`. Expected near the tropical groups (about 1″ Sun and
  planets, 4″ Moon) plus the ayanamsa gate's residual for that class.
- **Stop condition.** If a sidereal group shows a systematic offset larger than
  the ayanamsa gate's ceiling for its class, the cause is found before any
  ceiling is set. This is where a mismatch between the nutation-free reading
  here and what Swiss Ephemeris does under `SEFLG_SIDEREAL` would appear.

### Engine golden (Tier 1)

The new rows get `pleiades_jd_tdb` through `crossings-golden --regenerate` in
the same pull request. Tier 1 runs in the blocking tier via `pleiades-cli`.

## Tests in `pleiades-events`

Written before the implementation.

- **Tropical unchanged.** Existing pinned-bit tests pass; a new test pins that a
  `CrossingFrame` converted to a `CrossingReference` gives the same longitude
  bits as before for both existing frames.
- **Mean of date.** The Sun's mean-of-date longitude differs from its apparent
  longitude by aberration plus nutation (about −20.5″ aberration, plus Δψ), a
  check that does not pass through the engine's own formula.
- **Crossing residual.** In each new combination the longitude at the returned
  instant is within tolerance of the target, on the settled (later) side only.
- **Retrograde loop.** Mars in 2003 yields three sidereal crossings of one
  target.
- **Chaining.** `next_longitude_crossing` handed a returned sidereal instant
  gives the following crossing.
- **`position_at`.** Longitude bit-identical to `longitude_at` in every
  combination; sidereal speed equals tropical speed minus the ayanamsa rate
  (and the Δψ rate, apparent frame); mean-frame speed matches a central
  difference of `longitude_at`.
- **Errors.** Heliocentric with sidereal, and an ayanamsa with no offset, return
  `UnsupportedFrame`.
- **Range start.** A mean-frame read at JD 2415020.5 succeeds.

## Documentation

- Rustdoc on the five methods, `CrossingFrame` and `CrossingReference`,
  including one sidereal-ingress example and a note that
  `GeocentricMeanOfDate` is not the J2000 place a mean chart reports.
- `crates/pleiades-events/README.md`, the workspace `README.md` state table and
  the compatibility profile entry in `pleiades-core`
  (`src/compatibility/mod.rs`).
- `spec/*.md` and `SPEC.md` do not enumerate crossing frames and need no
  change.

## Follow-ups

Recorded in `docs/follow-ups.md` by this change, each confirmed by a
measurement before it is written:

- **Sidereal apparent chart keeps nutation.** `pleiades-core` subtracts the
  mean ayanamsa from the true-equinox longitude, so a sidereal apparent chart
  and a sidereal crossing differ by Δψ (up to about 17″).
- **Sidereal mean chart mixes frames.** A mean chart's longitude is J2000, and
  the of-date ayanamsa is subtracted from it.

## Out of scope

- Heliocentric sidereal crossings.
- Sidereal or mean variants of `next_sun_crossing` and `next_moon_crossing`.
- New CLI query flags.
- Any change to the chart layer.
- Aspect and station finders (#84, #85); they are expected to accept
  `CrossingReference`.

## Amendments

- **Final corpus.** The corpus has 169 rows (the 86 tropical rows plus 83 new
  mean-of-date and sidereal rows, not "about 70") and four ayanamsas: Lahiri,
  TrueCitra, GalacticCenter and DeLuce.
- **Stop condition tripped for apparent-frame TrueCitra and GalacticCenter.**
  The corpus rows disagreed with the engine by 20.615″ (TrueCitra) and 20.101″
  (GalacticCenter). Cause, confirmed in the reference tool: for star-anchored
  ayanamsas Swiss Ephemeris under plain `SEFLG_SIDEREAL` computes the ayanamsa
  from the anchoring star's place under the same flags as the body, so with
  apparent flags the ayanamsa carries the star's annual aberration (up to about
  20″). Its apparent-minus-`TRUEPOS|NOABERR|NOGDEFL` ayanamsa was 20.451″
  (TrueCitra, JD 2470276.058) and −19.737″ (GalacticCenter, JD 2480002.727).
  Resolution: the engine is unchanged (decision 3: the mean ayanamsa in every
  frame). The `geo`-frame TrueCitra and GalacticCenter reference rows are
  Swiss Ephemeris's apparent mean-equinox longitude (`SEFLG_NONUT`) minus its
  mean ayanamsa (`swe_get_ayanamsa_ex` with `TRUEPOS|NOABERR|NOGDEFL`);
  Lahiri, DeLuce and all mean-of-date rows use `SEFLG_SIDEREAL` directly.
  After the reference change every sidereal group (both geocentric places,
  all four ayanamsas) measures at most 0.456″. The tropical mean-of-date
  groups measure Sun 0.309″, Moon 2.638″ and planets 0.342″; the Moon is in
  line with the existing tropical apparent Moon group (2.606″). Recorded as
  FU-18(c).
  The affected set is every ayanamsa in the `TrueStar` and `Galactic`
  computation classes of `pleiades-ayanamsa`; True Citra and Galactic Center
  are the two that were measured, the rest follow from the same mechanism and
  are not measured. About 20″ is roughly 8 minutes of crossing time for the
  Sun and hours for a slow planet such as Saturn, more near a station.
- **Corpus coverage.** The sidereal groups use two epochs for Sun and Moon and
  one for Jupiter (the spec asked for three). The sidereal Moon ceiling (1″)
  rests on 16 rows, while the tropical Moon groups measure 2.6″; a
  regeneration with more Moon rows should expect a ceiling near 4″. Recorded
  as FU-18(d).
