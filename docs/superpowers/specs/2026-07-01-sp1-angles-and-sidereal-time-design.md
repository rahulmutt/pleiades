# SP-1 · Angles & Sidereal Time — Design

Status: **draft — 2026-07-01**. Design proposed; awaiting user review before
handing to writing-plans.

This is the first of a **three-sub-project arc** that closes the engine-layer
gaps for which Swiss Ephemeris *ships a function* (gaps where SE has no function
are deliberately out of scope per the readiness assessment):

- **SP-1 (this spec):** angles & sidereal time — `swe_sidtime` + the
  `swe_houses`/`ascmc[]` extras.
- **SP-2 (later spec):** ephemeris event-finding — longitude crossings
  (`swe_solcross`/`mooncross`/`helio_cross`), rise/set/meridian transit
  (`swe_rise_trans`, `swe_azalt`), local eclipse circumstances.
- **SP-3 (later spec):** new body sources — fixed stars (`swe_fixstar`),
  hypothetical/fictitious bodies from orbital elements, and (optionally)
  `swe_pheno` / planetary `swe_nod_aps`.

## Context

pleiades is being used as an **astronomy engine only**: it supplies
positions/houses/ayanamsas/eclipses, and the application (or other libraries)
builds the technique layer on top. This sub-project therefore adds only
engine-level quantities, not astrological techniques.

Current state (verified in code):

- `pleiades-houses::systems` already computes the Ascendant and Midheaven
  (`derive_angles`) using **true obliquity of date** (mean + Δε; overridable via
  `HouseRequest::obliquity`), and a private `local_sidereal_time(instant, lon)`
  that returns **local apparent sidereal time** (GMST via the IAU polynomial +
  equation of the equinoxes from the IAU-1980 nutation series, plus east
  longitude). Nutation comes from `pleiades-apparent::nutation`.
- `HouseAngles` exposes only Ascendant / Descendant / Midheaven / Imum Coeli.
  **ARMC, Vertex, equatorial ascendant, the co-ascendants, and the polar
  ascendant are absent**, and there is **no public sidereal-time API**.
- The isolated SE reference harness `tools/se-house-reference/` already calls
  `swe_houses` with a 10-element `ascmc` buffer and currently keeps only
  `ascmc[0]` (Asc) and `ascmc[1]` (MC), **discarding `ascmc[2..=7]`** — exactly
  the extra points this sub-project exposes.

So SP-1 is largely **exposure + a handful of derived points + a parity gate**,
not new astronomy. The of-date convention and ARMC are effectively already
present.

## Decisions captured

| Topic | Decision |
| --- | --- |
| SE functions targeted | `swe_sidtime` / `swe_sidtime0`; `swe_houses_ex` → `ascmc[2..=7]`; a `swe_houses_armc`-style "from ARMC" entry point |
| Points exposed | **Sidereal time**: GMST, GAST (Greenwich), local mean, local apparent — in degrees `[0,360)` and hours `[0,24)`. **Angle points**: ARMC, Vertex (+ Antivertex), equatorial ascendant (East Point), co-ascendant (Koch), co-ascendant (Munkasey), polar ascendant (Munkasey) |
| Foundation home | Canonical `sidereal_time` + obliquity-of-date primitives live in **`pleiades-apparent`** (it already owns nutation + of-date geometry; SP-2's rise/set + crossings need the same primitives). `pleiades-houses`'s private helper **delegates** to it so its SE-gated house numerics are preserved bit-for-bit. |
| `AscMc` home | The `AscMc` struct and `chart_points` computations live in **`pleiades-houses`** (angle points are house/angle concepts; SE computes `ascmc` inside its houses routine) |
| Formula provenance | Vertex / equatorial-ascendant / co-ascendant / polar-ascendant definitions are **ported directly from Swiss Ephemeris `swehouse.c`** (`Asc1` + vertex/co-ascendant/polar blocks). SE is the compatibility oracle; correctness is proven by the parity gate, not reconstructed from memory. |
| Obliquity convention | **True obliquity of date** by default (matches existing `derive_angles` and SE), overridable |
| Surfacing | New `asc_mc: AscMc` field on `HouseSnapshot`; re-exported through `pleiades-core`; surfaced on `ChartSnapshot` |
| Breaking change | Add the field and mark `HouseSnapshot` + `AscMc` `#[non_exhaustive]`; take the **one-time break at 0.2.x** (experimental line explicitly permits churn); bump the API-stability + compatibility profiles and note it in README. Rejected: a recompute-on-demand accessor (splits the source of truth, duplicates work). |
| Sidereal-time time base | Sidereal time is a function of **UT1** (Earth rotation), not TT/TDB. Preserve the existing GMST + equation-of-equinoxes formula exactly (to avoid perturbing SE-gated house numerics); **document** the UT1 input expectation, consistent with `docs/time-observer-policy.md`. Any change to the ΔT/UT1 posture is out of scope for SP-1. |
| Validation | Extend `tools/se-house-reference` to emit `ascmc[2..=7]` + `swe_sidtime`; extend the committed `houses-corpus`; add a fail-closed **`validate-angles`** gate (per-point arcsecond ceilings) wired into `release-smoke`/`release-gate`; keep claim-tier ↔ evidence alignment in the overclaim audit |

## Scope & boundaries

**In:**

- A public sidereal-time API (GMST, GAST, local mean, local apparent).
- The `ascmc` extra points: ARMC, Vertex (+ Antivertex), equatorial ascendant,
  co-ascendant (Koch), co-ascendant (Munkasey), polar ascendant (Munkasey).
- A `chart_points_from_armc`-style entry point (the `swe_houses_armc` case:
  compute the angle points from a supplied ARMC + geographic latitude +
  obliquity, without a clock time).
- Surfacing on `HouseSnapshot`, `pleiades-core` re-exports, and `ChartSnapshot`.
- An SE-parity numeric gate for all new quantities.

**Out (explicitly):**

- Anything SE has no function for (retrograde station finder, returns, general
  transit search, parans/out-of-bounds as functions) — excluded per the
  readiness rule.
- SP-2 / SP-3 material (rise/set, crossings, eclipses-local, fixed stars,
  hypothetical bodies).
- `swe_house_pos` (fractional house position of a body) — candidate for a later
  houses cycle; not required to expose the angle points.
- Any change to the ΔT / UT1 / time-scale policy beyond documenting the existing
  behavior.
- Sidereal-zodiac (ayanamsa) variants of the angle points — the façade already
  applies ayanamsa downstream; the new points are produced tropical/of-date like
  the existing Asc/MC.

## Architecture

### 1. Foundation: sidereal time + obliquity in `pleiades-apparent`

Add a public, self-contained module (e.g. `pleiades-apparent::sidereal`):

- `SiderealTime { gmst_deg, gast_deg, local_mean_deg, local_apparent_deg }` with
  `hours()` accessors (SE returns hours; astrology consumes degrees). All
  normalized to `[0,360)` / `[0,24)`.
- `fn sidereal_time(instant: Instant, observer_longitude: Longitude) ->
  Result<SiderealTime, ...>` — GMST from the IAU polynomial, GAST via the
  equation of the equinoxes (Δψ·cos ε_true from the existing IAU-1980 series),
  local variants by adding east longitude. **Formula preserved verbatim** from
  the current `pleiades-houses::local_sidereal_time`.
- Expose the obliquity-of-date helpers already implied here (mean obliquity +
  true obliquity = mean + Δε), de-duplicating the copy currently private in
  `pleiades-houses`.

`pleiades-houses` keeps its internal `local_sidereal_time`/`mean_obliquity` names
but has them **delegate** to the new `pleiades-apparent` functions, so no house
cusp value changes. (`pleiades-houses` already depends on `pleiades-apparent`.)

### 2. `AscMc` + chart points in `pleiades-houses`

```rust
#[non_exhaustive]
pub struct AscMc {
    pub ascendant: Longitude,
    pub midheaven: Longitude,
    pub descendant: Longitude,
    pub imum_coeli: Longitude,
    pub armc: Longitude,                 // ascmc[2]
    pub vertex: Longitude,               // ascmc[3]
    pub antivertex: Longitude,           // vertex + 180 (SE does not return it)
    pub equatorial_ascendant: Longitude, // ascmc[4] (East Point)
    pub coascendant_koch: Longitude,     // ascmc[5]
    pub coascendant_munkasey: Longitude, // ascmc[6]
    pub polar_ascendant: Longitude,      // ascmc[7]
}
```

- `fn chart_points(instant, observer, obliquity: Option<Angle>) ->
  Result<AscMc, HouseError>` — obliquity defaults to true-of-date (reusing
  `validated_obliquity`).
- `fn chart_points_from_armc(armc: Longitude, geolat: Latitude,
  obliquity: Angle) -> Result<AscMc, HouseError>` — the `swe_houses_armc` case.
- Both reuse the existing `ascendant_for` / midheaven math and add the
  SE-`swehouse.c`-ported Vertex / equatorial-ascendant / co-ascendant /
  polar-ascendant formulas.

### 3. Surfacing

- Add `pub asc_mc: AscMc` to `HouseSnapshot`; populate it in `calculate_houses`
  (it already derives `HouseAngles`; `AscMc` is a superset). Mark `HouseSnapshot`
  `#[non_exhaustive]`.
- Re-export `AscMc`, `SiderealTime`, `sidereal_time`, and the `chart_points`
  entry points from `pleiades-core`.
- Surface the points on `ChartSnapshot` (it already carries
  `houses: Option<HouseSnapshot>`, so `asc_mc` rides along; add a convenience
  accessor and include it in the CLI chart render).

### 4. SE reference extension (`tools/se-house-reference`)

Extend `main.rs` to keep `ascmc[2..=7]` (currently discarded) and to call
`swe_sidtime` (GAST at Greenwich) / derive GMST, emitting them to the corpus.
No new harness or new `-sys` dependency — reuses the existing isolated,
out-of-workspace crate (Constraint C1 already satisfied there).

### 5. Corpus extension (`crates/pleiades-validate/data/houses-corpus`)

Add the new quantities as **either** extra columns on `cusps.csv` **or** a new
`angles.csv` slice (open item — pick to keep the schema clean). Update
`manifest.txt` (row counts + checksums; SE version already recorded as
2.10.03). A clean checkout stays tool-free and only validates committed values,
per the de440/SE-corpus precedent.

### 6. The gate: `validate-angles`

A fail-closed command in `pleiades-validate` (either a new `angles_validation.rs`
or an extension of `house_validation.rs`), wired into `release-smoke` /
`release-gate` next to `validate-houses`. Checks:

- corpus checksum / schema / provenance drift;
- completeness (all fixtures × all new points present);
- numeric residuals vs **per-point arcsecond ceilings** (set from measured
  SE-vs-pleiades residuals — tight for ARMC/sidereal time and the well-conditioned
  points, documented looser bands near singularities), living in a `thresholds`
  module mirroring the house gate;
- claim-tier ↔ evidence alignment carried through the existing overclaim audit.

## Data flow

```
existing house fixtures (instant, lat, lon)
  └─(offline, SE present)─> se-house-reference: swe_houses → ascmc[0..=7]; swe_sidtime
        └─ write extended houses-corpus (cusps/angles + sidereal time) + manifest
              └─> committed corpus (source of truth)
                     └─(runtime gate: validate-angles)─>
                           pleiades: sidereal_time() + chart_points()
                           └─ compare each point vs SE reference within its ceiling
                                 └─> pass / fail (fail-closed)
```

## Error handling / fail-closed conditions

- Vertex, equatorial ascendant, and the co-/polar-ascendants have singularities
  at the poles / equator. Match the existing posture: return
  `HouseError { kind: InvalidLatitude | NumericalFailure }` rather than emitting
  non-finite values; document each point's defined domain.
- **Gate fails** on: missing slice, checksum/schema/provenance drift, any residual
  over its ceiling, or a missing point.
- **Generation fails** only on SE-side problems (mirrors `validate-houses`).

## Constraints

- **C1 — Pure-Rust workspace audit (hard).** The SE binding must never enter the
  published workspace lockfile (`workspace-audit` fails closed on `-sys`/`links`/
  `build.rs`). Satisfied by reusing the existing isolated `tools/se-house-reference`
  crate (its own `Cargo.lock`, outside the workspace). No new FFI surface.
- **C2 — SE license.** Already handled for this harness family
  (`tools/se-house-reference/LICENSE-NOTES.md`); verification-only, non-shipping.
  No new obligation from reusing it.

## Compatibility / versioning

- Adding a field to `HouseSnapshot` is breaking; take it once now at `0.2.x` and
  add `#[non_exhaustive]` to `HouseSnapshot` and `AscMc` to prevent future breaks.
- Bump the API-stability profile and the compatibility profile; note the new
  public surface (sidereal time + `ascmc` points) in README's "current state".

## Testing

- Unit tests per point against inline goldens (known ARMC / latitude cases,
  including ARMC = 0/90/180/270 and mid + high latitudes).
- Sidereal-time unit tests (GMST/GAST at reference epochs; local vs Greenwich).
- Doctest examples on every new public item (the crates doctest heavily).
- High-latitude / singularity edge tests (assert the documented error, not NaN).
- Integration: the `validate-angles` gate over the extended corpus; a manifest
  checksum-drift test.
- Regression guard: the existing `validate-houses` gate must still pass unchanged
  after the sidereal-time delegation (proves house numerics were preserved).

## Open items (confirm during planning/implementation)

1. **Per-point arcsecond ceilings** — set from measured SE-vs-pleiades residuals.
2. **SE Vertex / co-ascendant / polar-ascendant formulas** — port and confirm the
   exact `swehouse.c` definitions and hemisphere-sign handling before encoding.
3. **Corpus shape** — extra columns on `cusps.csv` vs. a new `angles.csv` slice.
4. **Sidereal-time surface** — expose GMST + GAST + local (mean/apparent) as a
   struct; confirm the UT1 input expectation wording against
   `docs/time-observer-policy.md`.
5. **Per-point defined domains** — document the latitude/obliquity ranges where
   Vertex / equatorial ascendant / co-/polar ascendants are well-defined, and the
   error emitted outside them.
