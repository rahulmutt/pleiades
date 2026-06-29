# Eclipse Subsystem (Phase 6 sub-project) — Design

Status: approved design, ready for implementation planning.
Date: 2026-06-29
Phase: 6 — Target Catalog Completion and Expansion (independent sub-project)

## Summary

Add a new first-party `pleiades-eclipse` crate that provides **global /
geocentric** solar and lunar eclipse data over the packaged window (1900–2100
CE), derived entirely from pleiades' already-gated Sun + Moon positions. For any
time range it returns the eclipses that occur, each carrying its type, the
instant of greatest eclipse, magnitude, gamma, Saros series, the eclipsed
ecliptic longitude, the node it falls near, and (for solar eclipses) the
geographic location of greatest eclipse.

This closes a real reliability gap for any consuming astrology application:
eclipse circumstances are genuinely hard to compute and cannot be derived from
instantaneous body positions alone, so they are squarely the library's
responsibility. Today the workspace has **no eclipse computation of any kind** —
everything pleiades produces is single-moment, and the only "eclipse" references
in the tree are an EclipseWise datum used to validate Moon coordinates. This
sub-project introduces pleiades' first **time-domain search** capability.

The work is additive and sits above the existing backends: it owns no ephemeris
data of its own, takes a Sun + Moon position backend, and inherits the existing
sub-arcsec accuracy story rather than introducing a parallel model.

## Context: why this is the chosen slice

The library's responsibility, as scoped with the consumer, is **accurate body
positions plus data that cannot otherwise be computed easily**. Aspects and
dignities are explicitly the consuming application's job (trivial given
positions) and are out of scope. Against that definition the open gaps are
eclipses, equatorial/declination output, and the osculating ("true") lunar
apogee. Eclipses are the highest-value, most-requested of these and the only one
that is a wholly new capability, so they are brought forward as their own
sub-project. The other two remain separate, smaller follow-up specs.

## Goals

- Compute geocentric solar and lunar eclipse circumstances across 1900–2100 CE
  from pleiades' own Sun + Moon positions, with no new ephemeris data or kernel.
- Offer a time-ordered range query plus `next` / `previous` convenience
  wrappers, with an optional solar/lunar filter, returning a unified `Eclipse`
  type that carries its own kind.
- Classify eclipse type exactly, including the annular/total/**hybrid** solar
  boundary and the penumbral/partial/total lunar boundary; **penumbral lunar
  eclipses are included** (the NASA canon lists them).
- Prove correctness with a fail-closed `validate-eclipses` gate against an
  **exhaustive** committed fixture of every eclipse in 1900–2100 from NASA's
  Five Millennium Canon (Espenak/Meeus), wired into `release-smoke` /
  `release-gate` alongside the existing numeric gates.
- Satisfy the Phase 6 exit criteria for a newly shipped capability: formula and
  source provenance, rustdoc/API examples, CLI coverage, validation fixtures,
  and truthful release-profile entries.

## Non-goals (YAGNI)

- **Local / topocentric circumstances.** No per-observer "is it total here",
  local magnitude, or local contact times. Scope is global/geocentric only.
- **Geocentric contact times** (penumbral/umbral P1–P4). Omitted; for
  global-only astrology use they are noise. The result type can gain them later
  without an API redesign if a need appears.
- **Native sidereal eclipse longitude.** `eclipsed_longitude` is tropical
  ecliptic of date; sidereal conversion is the façade/ayanamsa layer's job,
  consistent with the rest of the library (native sidereal backend output
  remains a deliberate non-goal).
- **Coverage beyond the packaged window.** Eclipses are searched within
  1900–2100 CE; the consumer confirmed this window suffices.
- **Occultations, transits of Mercury/Venus, eclipse path polygons / maps.**

## Public API

A new crate `pleiades-eclipse`:

```rust
pub enum EclipseKind { Solar, Lunar }

pub enum SolarEclipseType { Total, Annular, Hybrid, Partial }
pub enum LunarEclipseType { Penumbral, Partial, Total }
pub enum EclipseType {
    Solar(SolarEclipseType),
    Lunar(LunarEclipseType),
}

pub enum EclipseFilter { All, SolarOnly, LunarOnly }
pub enum Node { North, South }

pub struct Eclipse {
    pub kind: EclipseKind,
    pub eclipse_type: EclipseType,
    /// Instant of maximum eclipse — the moment to cast a chart on.
    pub greatest_eclipse: Instant,
    /// Eclipse magnitude (fraction of the disk covered).
    pub magnitude: f64,
    /// Least distance of the shadow axis from Earth's center, in Earth radii.
    pub gamma: f64,
    /// Saros series number.
    pub saros_series: u32,
    /// Tropical ecliptic longitude of date at greatest eclipse.
    pub eclipsed_longitude: Longitude,
    /// Which node the eclipse falls near.
    pub near_node: Node,
    /// Geographic location of greatest eclipse (solar only).
    pub greatest_eclipse_location: Option<GeoLocation>,
}

// `GeoLocation` is a new lightweight `{ latitude, longitude }` value. It is
// deliberately *not* the existing `ObserverLocation`, which implies an observer;
// the greatest-eclipse point is a geocentric sub-shadow position, not an
// observing site.

/// Search engine over a Sun + Moon position backend, configured for
/// apparent-place computation.
impl<B: Backend> EclipseEngine<B> {
    pub fn eclipses_in_range(
        &self,
        start: Instant,
        end: Instant,
        filter: EclipseFilter,
    ) -> Result<Vec<Eclipse>, EclipseError>; // time-ordered ascending

    pub fn next_eclipse(
        &self,
        after: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError>;

    pub fn previous_eclipse(
        &self,
        before: Instant,
        filter: EclipseFilter,
    ) -> Result<Option<Eclipse>, EclipseError>;
}
```

`next_eclipse` / `previous_eclipse` are thin wrappers over a bounded range scan.
`EclipseError` covers out-of-window requests and backend failures, consistent
with the existing fail-closed error posture.

## Algorithm (Approach A — direct geometric from pleiades' ephemeris)

Every number flows from the existing, gated Sun + Moon apparent positions, so the
eclipse accuracy story inherits the workspace's sub-arcsec one.

1. **Syzygy search.** Bracket new moons (solar candidates) and full moons (lunar
   candidates) by detecting sign changes in the wrapped Sun−Moon ecliptic
   elongation, stepping by roughly one mean synodic month and refining each root
   with Newton / Brent iteration.
2. **Eclipse test & classification.** At each syzygy compute the geocentric
   angular separation of the centers, the apparent solar and lunar radii, the
   lunar horizontal parallax, and Earth's umbra/penumbra cone geometry. Derive
   `gamma` and classify:
   - **Solar:** partial vs central from `gamma` against the limiting value;
     within central, annular vs total from whether the umbral cone reaches
     Earth's surface, with **hybrid** at the crossover.
   - **Lunar:** penumbral / partial / total from the Moon's immersion in Earth's
     penumbral and umbral radii at its distance.
   Compute `magnitude` from the same geometry.
3. **Greatest eclipse.** Refine the instant by minimizing the geocentric
   separation (solar) or the shadow-axis distance (lunar).
4. **Saros series.** Assign from the lunation number via the standard
   series-numbering scheme, anchored to a known reference eclipse.
5. **Derived fields.** `near_node` = North when the Moon crosses ascending,
   South when descending. `greatest_eclipse_location` (solar) = the sub-shadow
   point from the geometry plus Greenwich Mean Sidereal Time via `pleiades-time`.
   `eclipsed_longitude` = apparent solar longitude of date at greatest eclipse.

## Validation — the trust story

A fail-closed **`validate-eclipses`** gate, following the established
`validate-corpus` / `validate-houses` / `validate-ayanamsa` discipline:

- A committed fixture table of **every** eclipse in 1900–2100 CE sourced from
  **NASA's Five Millennium Canon of Solar/Lunar Eclipses (Espenak/Meeus)**, with
  documented provenance (~470 solar + ~480 lunar).
- The gate recomputes each eclipse and fails closed on any drift:
  - `eclipse_type` — exact match
  - `saros_series` — exact match
  - `greatest_eclipse` — within **≤ 60 s**
  - `magnitude` — within **≤ 0.01**
  - `eclipsed_longitude` — within tolerance (effectively free given the gated
    sub-arcsec Sun position)
- The gate is added to `release-smoke` / `release-gate` alongside the existing
  numeric gates.

These tolerances are release-grade for chart casting: the Moon moves ~0.5°/hr,
so 60 s is ~0.008° of longitude — negligible for a chart.

## Architecture & crate layering

`pleiades-eclipse` depends on:

- `pleiades-types` — shared `Instant`, `Longitude`, body identifiers, the new
  eclipse types.
- `pleiades-backend` — the Sun + Moon position trait the engine drives.
- `pleiades-apparent` — apparent-place positions.
- `pleiades-time` — GMST for the greatest-eclipse location.

It owns no ephemeris data and pulls in no kernel. A `pleiades-cli` command lists
eclipses in a range; the `validate-eclipses` gate lives in `pleiades-validate`.
The crate boundaries follow `spec/architecture.md`: one clear purpose
(time-domain eclipse search), a small well-defined public surface, dependencies
pointing only downward.

## Testing

Test-driven throughout:

- Unit tests for syzygy root-finding, each classification boundary (with
  explicit cases at the hybrid and penumbral edges), Saros assignment, and the
  node / greatest-eclipse-location geometry.
- Rustdoc examples on the public API (`eclipses_in_range`, `next_eclipse`).
- The exhaustive `validate-eclipses` gate as the integration backstop and the
  primary correctness evidence.

## Exit criteria

- `pleiades-eclipse` ships the public API above with rustdoc/API examples and
  passes `validate-eclipses` exhaustively over 1900–2100.
- `validate-eclipses` is wired into `release-smoke` / `release-gate`.
- A CLI command lists eclipses in a range.
- The release compatibility profile and README state eclipse support truthfully
  (global/geocentric, 1900–2100, NASA-canon-validated), with no overclaim of
  local circumstances.

## As-built notes (maintainer-ratified deviations from the design)

The implementation was reviewed and merged with the following ratified deviations
from the original design wording. These notes are recorded here so the design
doc is not contradicted by the code.

**(a) Window data-bound to 2100-01-01, not "all of 2100 CE."**
The packaged Sun/Moon ephemeris has no segments beyond JD 2 488 069.5
(2100-01-01 TDB). Four NASA-canon eclipses falling in mid/late 2100 are
therefore uncomputable with the packaged data and were trimmed from the corpus
fixture. Any public documentation must state the window as
"1900-01-01 … 2100-01-01", not "through 2100 CE" or "all of 2100".

**(b) Lunar shadow enlargement ratified at factor 1.01 on (π_moon + π_sun),
not "1.02 on Earth's radius."**
The design prose loosely described a "1% enlargement on Earth's shadow radius"
following Danjon. The shipped code applies 1.01 × (lunar horizontal parallax +
solar horizontal parallax) to the penumbral/umbral cone half-angles — i.e. the
factor is applied to the angular sum, not to Earth's physical radius. This is
the standard Danjon convention and is what the validate-eclipses gate enforces.

**(c) Apparent eclipsed longitude via single light-time and aberration
application.**
`eclipsed_longitude` is the apparent solar longitude of date at greatest
eclipse, computed by a single light-time + annual aberration application in
`pleiades-apparent`. This matches the `≤ 1.0″` gate tolerance on the
independently-sourced (Skyfield/DE440) reference; no further iteration is
needed.

**(d) One allowlisted knife-edge eclipse.**
1948-05-09, Saros 137 (NASA: annular, magnitude 0.9999; engine: hybrid,
magnitude 1.0004). The geocentric shadow-cone model cannot resolve this at
better than ≈ 0.0005 magnitude without topocentric limb profiles. It is
allowlisted for the exact-type check only; time, magnitude, Saros, and
eclipsed-longitude tolerances are still enforced for this row.

---

## Follow-up sub-projects (out of scope here, recorded for sequencing)

- **Equatorial / declination output** — populate the existing `equatorial`
  result field for all release-grade backends from the library's true obliquity
  + nutation, gated vs Swiss Ephemeris. Enables declination parallels,
  out-of-bounds detection, and right-ascension work.
- **True (osculating) lunar apogee** — implement the currently-unsupported
  `TrueApogee` (osculating Lilith), reaching Swiss Ephemeris parity
  (`SE_OSCU_APOG`) with the already-present mean apogee (`SE_MEAN_APOG`).
</content>
</invoke>
