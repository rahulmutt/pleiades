# A public ecliptic position with latitude and speed (issue #89) — design

**Status:** approved design, not yet implemented ·
**Opened:** 2026-10-01 · **Issue:** #89 ·
**Crates:** `pleiades-events`, `pleiades-apparent`, `pleiades-core`,
`pleiades-validate` ·
**Tool:** `tools/se-helio-reference` (new) ·
**Also in this change:** Rust toolchain and MSRV 1.98.1 → 1.99.0

## Context

The only public heliocentric read is
`EventEngine::longitude_at(body, CrossingFrame::Heliocentric, instant)`
(`pleiades-events` `src/crossings.rs`). It returns a `Longitude` and nothing
else. The reconstruction behind it, `heliocentric_longitude_deg`
(`src/ephemeris.rs`), computes the heliocentric latitude and has the distance
to hand, then discards both. No speed is produced in either frame.

A consumer that wants a heliocentric longitude, latitude, distance and speed
must repeat the reconstruction outside pleiades (planet vector minus Sun
vector, precession and nutation to the true equinox of date), and its result
can then disagree with the heliocentric crossings the engine finds.

Decisions taken on 2026-10-01:

1. The new method covers **both** `CrossingFrame` variants, mirroring
   `longitude_at`, not the heliocentric frame alone.
2. Speed is the **backend's speed plus a differenced correction**, the
   technique #91/#92 introduced for chart placements. The position itself is
   not differenced.
3. The heliocentric values are gated against a **Swiss Ephemeris corpus**
   (`SEFLG_HELCTR | SEFLG_SPEED`). The geocentric values are gated by
   agreement with the chart layer, whose apparent place is already gated
   against JPL Horizons by `validate-apparent`.
4. The pending `mise.toml` toolchain bump ships in the same pull request.

### What already exists

- **Geocentric position.** `geocentric_apparent_ecliptic` (`src/ephemeris.rs`)
  returns apparent-of-date `(longitude, latitude, distance)`.
  `geocentric_apparent_longitude_deg` is a thin wrapper kept byte-identical for
  `validate-crossings`.
- **Heliocentric longitude.** `heliocentric_longitude_deg` reads the mean
  J2000 geocentric planet and Sun, subtracts the vectors, precesses to the mean
  ecliptic of date and adds nutation in longitude. It is geometric: no
  light-time and no aberration, matching `SEFLG_HELCTR`.
- **Apparent speed.** `pleiades-core` `src/chart/apparent_motion.rs` holds
  `Correction`, `CorrectionSample` and `apparent_motion`: the backend's mean
  `Motion` plus the rate of (apparent − mean), differenced centrally over
  ±0.5 day and one-sided at the edge of a backend's range. All three are
  private to the chart module.
- **Layering.** `pleiades-events` and `pleiades-core` both depend on
  `pleiades-apparent`; neither depends on the other.

## Public API

In `pleiades-events`, next to `longitude_at`:

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    pub fn position_at(
        &self,
        body: CelestialBody,
        frame: CrossingFrame,
        instant: Instant,
    ) -> Result<EclipticPosition, EventError>;
}

#[non_exhaustive]
pub struct EclipticPosition {
    pub body: CelestialBody,
    pub frame: CrossingFrame,
    pub instant: Instant,
    /// Longitude, latitude and distance (AU) in `frame`.
    pub ecliptic: EclipticCoordinates,
    /// Longitude, latitude and distance speed in `frame`, per day.
    pub motion: Motion,
}
```

`EclipticPosition` derives `Clone`, `Debug` and `PartialEq`, and serde under
the existing `serde` feature, as `Crossing` does. It reuses
`EclipticCoordinates` and `Motion` from `pleiades-types`.

Behaviour:

- **Guards** are those of `longitude_at`: `check_window` (1900–2100 TDB) and
  `EventError::UnsupportedFrame` for a heliocentric Sun or Moon. The instant is
  read as TDB, as in `longitude_at`.
- **Longitude identity.** `position_at(b, f, t).ecliptic.longitude` equals
  `longitude_at(b, f, t)` bit for bit. `longitude_at` keeps its current
  evaluation path, so the crossings golden and `validate-crossings` do not
  move.
- **Distance** is always `Some`; a backend read without a distance fails
  closed with `MissingCoordinates`, as today.
- **Speed channels** the backend does not supply are `None`. A speed is never
  substituted from another place or frame. When no neighbouring instant is
  available for the correction difference, `motion` is all `None`.

`CrossingFrame` is unchanged. Mean and sidereal frames are issue #88.

## Shared speed helper

Move the contents of `pleiades-core` `src/chart/apparent_motion.rs`
(`Correction`, `CorrectionSample`, `apparent_motion`, `HALF_SPAN_DAYS`, the
signed wrap) into a new public module `pleiades_apparent::motion`, with new
unit tests of the pure helper as `motion/tests.rs`. `pleiades-core` imports the
module; its chart behaviour is unchanged, and the existing chart-level tests
(`chart/apparent_motion/tests.rs`, which drive `ChartEngine`) stay in core.
Because the helper becomes public, a zero or non-finite span between the two
samples returns a `Motion` with every channel `None` instead of a non-finite
speed.

The item names do not change. The helper adds the rate of *any* small smooth
correction between two places to a base `Motion`, so its rustdoc is reworded in
those terms ("base" and "corrected" place); the heliocentric path uses it with
the J2000 place as base and the of-date place as corrected.

This is the only refactor in the change. It is required so that the chart and
the event engine share one implementation and cannot drift.

## Position and speed per frame

Both frames follow one pattern: a base place with a base `Motion`, a corrected
place, and `motion = base motion + d(corrected − base)/dt` by the shared
helper, sampled at `t ± 0.5 day` (one-sided where a sample would leave the
window or the backend's range).

### Geocentric apparent of date

- Base: the backend's mean J2000 geocentric ecliptic place and its `Motion`,
  read at `t`. `read_mean_ecliptic` gains a sibling that also returns the
  backend `Motion`.
- Corrected: `geocentric_apparent_ecliptic` at the same instant.
- Result: the same quantity the chart layer reports for an apparent tropical
  placement of a release-grade body.

### Heliocentric of date

1. Refactor `heliocentric_longitude_deg` into `heliocentric_ecliptic`, which
   returns `(longitude, latitude, distance)` of date. The longitude wrapper
   remains and returns the same bits as today. Latitude is the precessed
   latitude (nutation in longitude does not change ecliptic latitude);
   distance is the norm of the heliocentric vector.
2. Base: the heliocentric **J2000** spherical place and its rates. The rates
   come from Cartesian velocities: convert the planet's and the Sun's
   geocentric `(λ, β, r)` and `(λ̇, β̇, ṙ)` to Cartesian position and velocity,
   subtract Sun from planet, and convert the heliocentric state back to
   spherical rates. If any needed backend rate is `None`, the base `Motion` is
   all `None`.
3. Corrected: the of-date place from step 1. The correction (of-date − J2000)
   is the precession and nutation rotation, small and smooth, and is
   differenced by the shared helper.

The spherical ↔ Cartesian state conversions are pure functions in a new
`pleiades-events` `src/state_vector.rs`, unit-tested on their own. At a
heliocentric pole (`x² + y² = 0`) the longitude rate is undefined; the
conversion returns `None` for that channel rather than a non-finite value.

`src/ephemeris.rs` keeps evaluating places and grows only by the heliocentric
refactor. `EclipticPosition`, `position_at` and the speed sampling live in a
new `src/position.rs`, so no file needs splitting.

## Errors

No new `EventError` variants. Out-of-window instants, heliocentric Sun/Moon,
missing coordinates and backend failures map to the existing variants, exactly
as in `longitude_at`. Missing speeds are not errors (see above).

## Validation

### Heliocentric: Swiss Ephemeris corpus gate

- **Tool.** `tools/se-helio-reference`, modelled on
  `tools/se-crossings-reference` (vendored Swiss Ephemeris via
  `libswisseph-sys`, built only under `devenv shell`, never by the workspace
  build). It writes a CSV of `swe_calc(tjd_et, body, SEFLG_HELCTR |
  SEFLG_SPEED)` in the tool's default ephemeris mode, recorded in the CSV
  header: longitude, latitude, distance and the three speeds.
- **Corpus.** Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune and
  Pluto, sampled across 1900–2100 TDB on a step that is not commensurate with
  any body's period; committed and read with `include_str!`.
- **Gate.** `validate-helio-position` in `pleiades-validate`
  (`helio_position_validation.rs` plus a thresholds data module), evaluating
  `EventEngine::new(packaged_backend()).position_at(..)` at every row. Ceilings
  are set per channel from the measured maxima with headroom, recorded with
  the measured values, and wired into the release posture alongside the other
  SE gates. The gate enforces a validated-row floor on the release path.
- **Expected residual class.** `validate-crossings` already measures the
  heliocentric longitude against SE: 35.09″ maximum for Mercury–Neptune and
  3.53″ for Pluto (ceilings 50″ and 5″). The larger figure is the known
  light-time signature of reconstructing the heliocentric vector from the
  backend's geocentric vectors (see `tests/heliocentric.rs`); removing it is
  not part of this change. The new gate's longitude maximum is expected in the
  same class. The latitude, distance and speed residuals are measured, not
  assumed; a longitude residual above 50″, or any residual that neither the
  light-time signature nor the Moshier-versus-DE440 difference explains, is
  investigated before a ceiling is set over it.

### Geocentric: agreement with the chart layer

`pleiades-core` becomes a dev-dependency of `pleiades-events` (no cycle: core
does not depend on events). A test builds an apparent tropical geocentric
chart on the packaged backend and asserts that each release-grade body's
ecliptic place and motion equal `position_at(.., GeocentricApparentOfDate, ..)`
at the same instant, across several epochs including one near each window
edge. The tolerance is exact equality where both paths call the same
functions, and otherwise a documented bound measured in the test.

### Unit and regression tests

- State-vector algebra: a circular coplanar orbit with an analytic velocity; an
  inclined orbit checked against a central difference of the position at a
  small step; the pole case returns `None`.
- Longitude identity between `position_at` and `longitude_at`, both frames,
  several bodies and epochs.
- Guards: out-of-window instant, heliocentric Sun and Moon.
- Window edges: one-sided difference at 1900 and 2100; a backend without
  motion yields `Motion` of all `None` with a valid position.
- Heliocentric speed sanity: the of-date longitude speed exceeds the J2000
  speed by the precession rate (about 3.8e-5 °/day) within the nutation-rate
  bound.
- The moved `pleiades_apparent::motion` tests keep passing unmodified apart
  from their paths.

`validate-crossings` and the `crossings-golden` manifest must be unchanged by
this work; a diff in either is a defect, not something to regenerate.

## Toolchain and MSRV

`mise.toml` moves Rust from 1.98.1 to 1.99.0. Per `README.md`, the MSRV must
match the mise toolchain, so the same pull request sets
`[workspace.package] rust-version = "1.99.0"` in `Cargo.toml` and updates the
MSRV line in `README.md`. This is a separate commit from the feature, with any
lint fixes 1.99.0 requires. The pinned fuzz nightly (`FUZZ_NIGHTLY`) is not
touched.

Consumer-visible effect: crates released after this change require Rust
1.99.0.

## Documentation

- Rustdoc on `position_at` and `EclipticPosition`, with a doctest on the
  packaged backend for each frame, stating units, frame, the geometric nature
  of the heliocentric place and the failure modes.
- `crates/pleiades-events/README.md` and the workspace `README.md` capability
  table gain the new surface and its gate.
- `docs/follow-ups.md`: a resolved entry (FU-16) with the measured gate maxima.
- The compatibility profile (`pleiades-core` `src/compatibility/mod.rs`) gains
  an entry for the new surface and moves to 0.7.17, with its pinned checksum
  and the two tests that pin the identifier.
- `spec/*.md`: updated only where a document enumerates the events surface or
  the validation gates.

## Out of scope

- A heliocentric chart (`ChartRequest` has no centre).
- Mean-place or sidereal frames for crossings or positions (#88).
- Equatorial output for `position_at`.
- Heliocentric positions for the Sun, the Moon, lunar points or fictitious
  bodies.
- Station finding on the new speed (#85).

## Risks

- **SE speed convention.** `SEFLG_SPEED` under `SEFLG_HELCTR` is a geometric
  heliocentric rate of date. If the measured speed residual shows a systematic
  offset of the size of the precession rate, the convention differs from the
  one above and the design is revisited before any ceiling is set.
- **Backend rates near the window edge.** One-sided differences are less
  accurate than central ones; the corpus includes rows within a day of each
  edge so the ceiling covers them.
- **Refactor drift.** Moving the helper into `pleiades-apparent` must not
  change chart output; the existing #92 tests and the chart goldens are the
  guard.
