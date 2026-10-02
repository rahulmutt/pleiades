# Event finder for planetary stations (issue #85) — design

**Status:** implemented (2026-10-02) ·
**Opened:** 2026-10-02 · **Issue:** #85 ·
**Crates:** `pleiades-events`, `pleiades-validate`, `pleiades-cli` (gate
command routing only), `pleiades-core` (compatibility profile entry only) ·
**Tool:** `tools/se-stations-reference` (new)

## Context

`Motion::longitude_direction` (`pleiades-types` `src/motion.rs`) classifies
one sample's longitude speed as direct, retrograde or (for an exactly zero
speed) stationary. Nothing finds the instant at which a body stations, that
is, when its longitude speed changes sign. A caller can only sample the
direction flag and learns a station to the resolution of the sampling.

Intended outcome: a caller asks `pleiades-events` for the stations of a body
in a range, or the next one after an instant, and gets instants that agree
with the direction the engine itself reports, validated once against an
independent reference.

Decisions taken on 2026-10-02:

1. **Reference authority:** a Swiss Ephemeris speed-zero corpus. Swiss
   Ephemeris has no station finder, so a reference tool bisects its own
   longitude speed to zero.
2. **Scope:** exactly the two finders the issue names, plus the gate. No
   `previous_station`, no user-facing CLI command, no asteroids in the gate.
3. **Definition:** a station is a sign change of the longitude speed
   `EventEngine::position_at` reports (approach A below).
4. **Errors:** a new `EventError::MissingSpeed` variant; the change ships as
   `feat(events)!`.

### What already exists

- **Speed.** `EventEngine::position_at` (`src/position.rs`, issue #89)
  returns `motion.longitude_deg_per_day` in every `CrossingFrame` and zodiac:
  the backend's speed plus the rate of the frame or apparent-place
  correction, differenced over ±0.5 day.
- **Root-finding.** `src/root.rs` brackets by stepping and refines by
  bisection (`crossings_in_range`, `first_crossing_after`). Since #80/#81
  `bisect` returns the later end of its final bracket, so a returned instant
  is "settled": the event has already happened there, less than 0.5 s
  earlier.
- **Guards.** `check_window` and `reference::check_supported` reject
  out-of-window instants, a heliocentric Sun or Moon, a sidereal zodiac in
  the heliocentric frame, and an ayanamsa with no finite offset.
- **Gate pattern.** `validate-helio-position`: a `tools/se-*-reference`
  generator built only under `devenv shell`, a committed CSV with a manifest
  checksum read by `include_str!`, a validation module plus a thresholds data
  module in `pleiades-validate`, and a validated-row floor.

### Approaches considered

- **A. Zero of the speed `position_at` reports (chosen).** A station is
  exactly where the engine's own direction flag flips, in every frame and
  zodiac. Reuses `root.rs` unchanged and keeps the backend's own speed, which
  is the accurate part.
- **B. Zero of a differenced longitude.** Works for a backend with no speed,
  but defines a second speed inside the engine, so a station could sit
  minutes away from where `position_at` flips direction.
- **C. Extremum search on longitude.** A flat extremum resolves time only to
  about the square root of the longitude noise, and yields no settled
  instant for chaining.

## Public API

In `pleiades-events`, re-exported from `lib.rs`:

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    /// All stations of `body` in `[start, end]` (TDB), ascending.
    pub fn stations_in_range(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<Station>, EventError>;

    /// The first station strictly after `after`, or `None`.
    pub fn next_station(
        &self,
        body: CelestialBody,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<Station>, EventError>;
}

#[non_exhaustive]
pub struct Station {
    pub body: CelestialBody,
    /// Instant of the station (TDB).
    pub instant: Instant,
    /// Longitude at `instant`, in `frame` and `zodiac`.
    pub longitude: Longitude,
    pub kind: StationKind,
    pub frame: CrossingFrame,
    pub zodiac: ZodiacMode,
}

#[non_exhaustive]
pub enum StationKind {
    /// Direct before, retrograde after.
    TurnsRetrograde,
    /// Retrograde before, direct after.
    TurnsDirect,
}
```

`Station` derives `Clone`, `Debug` and `PartialEq`; `StationKind` also
`Copy`, `Eq` and `Hash`. Both carry serde under the existing `serde`
feature, as `Crossing` and `EclipticPosition` do.

The signatures mirror `longitude_crossings_in_range` and
`next_longitude_crossing`: two instants rather than a range type, and
`impl Into<CrossingReference>` so a bare `CrossingFrame` (tropical) or a
sidereal reference both work. The zodiac is not cosmetic: the ayanamsa's
rate (about 3.8e-5 °/day) shifts the zero of the speed, so a sidereal station
falls at a slightly different instant from the tropical one.

### Semantics

- **Definition.** A station is a sign change of
  `position_at(body, reference, t).motion.longitude_deg_per_day`.
- **Settled instant.** The returned instant is the later end of the final
  bisection bracket. It trails the sign change by less than 0.5 s and never
  precedes it, so `next_station(after = s.instant)` returns the following
  station, not `s` again. `position_at` at the returned instant already
  reports the post-station direction.
- **Longitude and kind.** Both come from one `position_at` at the returned
  instant. A speed greater than zero there is `TurnsDirect`; otherwise
  `TurnsRetrograde`. This matches `root::bisect`, which counts zero as the
  negative side.
- **`next_station` and `stations_in_range` agree.** `next_station(b, r, t)`
  equals the first element of `stations_in_range(b, r, t, WINDOW_END)` that
  is strictly after `t`: same clamps, same step, same tolerance.
- **Bodies that never station.** The Sun, the Moon, the mean node and every
  body in the heliocentric frame return an empty `Vec` or `None` because no
  sign change is found. There is no special-casing by body.
- **Instants** are read as TDB, as everywhere else on the crossings surface.

### Accuracy of a station instant

The bisection tolerance (0.5 s) bounds only how well the engine locates the
zero of *its own* speed. How well that zero matches the true station is set
by the speed error divided by the body's longitude acceleration at the
station, and the acceleration is small for slow bodies. A station instant is
therefore soft: expect minutes for the inner planets and possibly an hour or
more for Pluto. The rustdoc states the per-body figures the gate measures;
it does not claim a blanket precision.

## Errors

One new variant:

```rust
/// The backend reported no longitude speed for a body, so its stations
/// cannot be found.
EventError::MissingSpeed { body_label: &'static str, julian_day: f64 }
```

`EventError` is not `#[non_exhaustive]`, so this breaks exhaustive matches
and the change is `feat(events)!`. Reusing `MissingCoordinates` was rejected:
its message ("no ecliptic coordinates") would misdescribe the failure.
Marking `EventError` `#[non_exhaustive]` in the same breaking release is
included, so the next variant is not breaking.

`MissingSpeed` is returned when `position_at` yields
`motion.longitude_deg_per_day == None` at any instant the scan or the
bisection evaluates. A missing speed is never read as "no station".

All other failures map to existing variants exactly as in `position_at`:
`OutOfWindow`, `UnsupportedFrame`, `MissingCoordinates`, `Backend`. Checks
run in the order `longitude_crossings_in_range` uses (window, then
`check_supported` with the label `"stations are"`).

## Algorithm and layout

- **Module.** New `crates/pleiades-events/src/stations.rs` holding `Station`,
  `StationKind`, the two finders and the step table, with tests in
  `src/stations/tests.rs`. `crossings.rs` is not grown. `root.rs` is reused
  unchanged; its wrap-seam guard (`|Δf| < 180`) is inert for a speed.
- **Root function.** `f(jd) = position_at(body, reference, jd)
  .motion.longitude_deg_per_day`, or `MissingSpeed`. Internally this calls
  the same `sample` and `motion` helpers `position_at` uses (made
  `pub(crate)`), not the public method, to avoid cloning the body and
  re-running the guards at every evaluation.
- **Bracketing step.** Chosen from the shortest interval between consecutive
  stations:

  | Body | Step (days) |
  |---|---|
  | Mercury, Venus, Sun | 1.0 |
  | Moon, `MeanNode`, `TrueNode`, `MeanApogee`, `TrueApogee`, `MeanPerigee`, `TruePerigee` | 0.25 |
  | everything else | 2.0 |

  The shortest planetary retrograde (Mercury) lasts about three weeks, so the
  planetary steps have a wide margin.
- **Window clamp.** The scan runs over
  `[max(start, WINDOW_START + step), min(end, WINDOW_END − step)]`, as
  crossings do, so neighbouring samples stay in the window.
- **Known limit** *(superseded for the true node — see the amendment at the
  end)*. Two stations closer together than the step are missed as a
  pair. This can matter only for the true node and the osculating apogee,
  whose speeds oscillate. The reference tool scans far finer than the engine
  (see below), so the gate's count comparison measures whether 0.25 day is
  enough for the true node; if it is not, the step is reduced until the
  counts match and the cost is recorded. The limit is documented on both
  finders.
- **Cost.** Each evaluation is three place samples. A full-window Mercury
  scan is about 73 000 evaluations; measured in the plan and recorded, not
  optimised unless it is a problem for the gate's runtime.

### To verify first *(resolved — see the amendment at the end)*

Whether `packaged_backend()` reports a longitude speed for `TrueNode` in the
geocentric frames has not been checked. It is the first task of the plan. If
it does not, the true node returns `MissingSpeed`, the issue's "lunar nodes"
case is not met, and the design returns to the user before further work.

## Validation

### Reference tool

`tools/se-stations-reference`, modelled on `tools/se-helio-reference`
(vendored Swiss Ephemeris through `libswisseph-sys`, Moshier mode, built only
under `devenv shell` with the same `CFLAGS=-std=gnu17` caveat, never by the
workspace build). For each body and group it scans the `swe_calc` longitude
speed on a 0.25-day grid (0.005 day for the true node), bisects each sign
change to 1e-7 day, and writes one CSV row per station: group, body, TT
Julian day, longitude, kind.

Groups and Swiss Ephemeris flags (all with `SEFLG_MOSEPH | SEFLG_SPEED`):

| Group | Bodies | Span | Extra flags |
|---|---|---|---|
| `geo` (geocentric apparent, tropical) | Mercury–Pluto | 1900–2100 | none |
| `geo` | true node | 1990–2030 | none |
| `mean` (geocentric mean of date) | Mercury, Mars, Saturn | 1990–2030 | `TRUEPOS \| NOABERR \| NOGDEFL \| NONUT` |
| `sid` (geocentric apparent, Lahiri) | Mercury, Mars, Saturn | 1990–2030 | `SIDEREAL`, `swe_set_sid_mode(SE_SIDM_LAHIRI)` |

Lahiri is not star-anchored, so the FU-18(c) aberration difference does not
apply to the `sid` group.

### Gate

`validate-stations` in `pleiades-validate`: `stations_validation.rs`,
`stations_validation/tests.rs`, `stations_thresholds.rs`, corpus under
`data/stations-corpus/` with a manifest checksum, wired into
`render/cli.rs`, the `pleiades-cli` validate routing and the release posture
alongside `validate-helio-position`.

For each group and body it calls
`EventEngine::new(packaged_backend()).stations_in_range(..)` over the
corpus span and requires:

1. **Same count and same kind sequence** as the corpus, for the planets. A
   missed or extra station fails the gate; nothing is matched by nearest
   neighbour. The true node uses the separated-station rule in the amendment
   at the end.
2. **Time residual** within a per-body ceiling.
3. **Longitude residual** within a per-body ceiling.
4. **Validated-row floor** enforced on the release path.

Ceilings are set from the measured maxima with headroom and recorded with
the measured values in the thresholds module and in `docs/follow-ups.md`.
A residual that neither the Moshier-versus-DE440 difference nor the
soft-instant argument above explains is investigated before a ceiling is set
over it. The tool and the gate use the same spans, which start and end five
days inside the window, so neither side meets the engine's edge clamp.

### Unit and regression tests (`src/stations/tests.rs`)

*(Backends amended — see the end: these run on the packaged backend, and the
missing-speed case on `LinearSunMoon`.)*

- one known station: instant within tolerance, never before the true zero,
  correct kind and longitude;
- a retrograde loop: `TurnsRetrograde` then `TurnsDirect`, in order;
- `next_station` equals the first in range;
- chaining: `next_station(after = s.instant)` is the following station;
- a monotonic body (Sun, Moon) returns empty and `None`;
- a backend with no speed returns `MissingSpeed`;
- out-of-window range fails closed; heliocentric Sun/Moon and
  sidereal-heliocentric are `UnsupportedFrame`.

Against `packaged_backend()` (integration test):

- every planet in the heliocentric frame has no stations over a multi-year
  span;
- `position_at` just before and at each returned station disagree in
  direction;
- sidereal and tropical stations of one body differ by a small, non-zero
  time with the expected sign.

`validate-crossings` and the `crossings-golden` manifest must be unchanged;
`longitude_at` is not touched. A diff in either is a defect.

## Documentation

- Rustdoc on both finders, `Station` and `StationKind`, with a doctest on the
  packaged backend (Mercury's next station after J2000), stating units,
  frame, the settled-instant contract, the step limit, the measured accuracy
  and the failure modes.
- `crates/pleiades-events/README.md` and the workspace `README.md`
  capability table. `CHANGELOG.md` is written by release-plz from the
  `feat(events)!` commit (breaking: `MissingSpeed`, `#[non_exhaustive]` on
  `EventError`); it is not edited by hand.
- `docs/follow-ups.md`: a resolved FU-21 entry with the measured gate maxima
  and the deferred items below.
- `spec/astrology-domain.md` ("retrograde and stationary classification") and
  any `spec/*.md` list of event surfaces or validation gates gain the station
  finder.
- The `pleiades-core` compatibility profile gains an entry for the new
  surface, with its pinned checksum and the tests that pin the identifier.

## Out of scope

- `previous_station` (it would inherit the FU-13 backward-search caveat).
- A user-facing `pleiades-cli` stations command.
- Asteroids, fictitious bodies and the osculating apogee in the gate. The
  finders accept them; their stations are ungated and documented as such
  (FU-7 records asteroid speed defects).
- Rejecting heliocentric lunar points (FU-17(b)).
- Aspect-exactness finding (#84).

## Risks

- **True-node speed.** The backend may report none (see "To verify first"),
  or the 0.25-day step may miss close station pairs. Both are measured before
  the gate is written.
- **Soft instants.** Pluto's and Neptune's time residuals may be large enough
  that a tight ceiling is not meaningful. The gate then relies on the
  longitude residual and the count for those bodies, and the documentation
  says so with the measured figures.
- **Reference speed convention.** Swiss Ephemeris's speed is analytic or
  differenced depending on body and flags. A systematic time offset of one
  sign across a body's stations indicates a convention difference and is
  investigated before any ceiling is set.
- **Gate runtime.** Full-window scans of nine bodies at three samples per
  evaluation run in the blocking tier's validate budget only if measured to
  fit; otherwise the full span moves to the nightly tier and the blocking
  tier keeps a subset, as other gates do.

## Amendment (2026-10-02, pre-plan probe)

A scratch probe on the packaged backend (geocentric apparent, optimised
build) measured the following before the plan was written.

**Resolved: the true node has a speed.** `position_at` reports a longitude
speed for `TrueNode`, `MeanNode` and `TrueApogee`. `MissingSpeed` does not
apply to them.

**Planet steps hold.** Over 1980–2020 the shortest interval between
consecutive stations was 19 days (Mercury, step 1), 41 days (Venus, step 1),
60 days (Mars, step 2) and 116–158 days (Jupiter–Pluto, step 2). The Sun, the
Moon and the mean node gave no stations; heliocentric Pluto's speed never
fell below 0.005 °/day.

**Cost.** About 0.3 ms per speed evaluation for a planet and 0.9 ms for the
true node. The gate as specified below is about 70 s for the eight planets
over the full window, about 55 s for the true node over 1990–2030, and a few
seconds for the `mean` and `sid` groups.

**Finding: the true node grazes zero.** The true node is retrograde on
average, and its speed rises to touch zero about every two weeks. Each touch
either just crosses, giving a pair of stations hours apart, or just misses.
Over two years the count was 96, 98, 98 and 102 at steps of 0.5, 0.25, 0.1
and 0.02 day. The speed is smooth; the pairs that appear only at fine steps
have a peak direct speed of 1e-7 to 4e-5 °/day, far below the difference
between pleiades and Swiss Ephemeris for this body. Whether such a pair
exists is model-dependent, so an exact count match cannot hold for the true
node at any step.

Decision (user, 2026-10-02): **match only well-separated true-node
stations.**

- A station is *separated* when its nearest neighbouring station in its own
  list is at least 2 days away.
- Every separated corpus station must have an engine station of the same
  kind within the true node's time ceiling, and its longitude residual must
  be within the longitude ceiling.
- Every separated engine station must have a corpus station of the same kind
  within the time ceiling.
- Stations in closer pairs are unconstrained on both sides.
- The engine's 0.25-day step for the true node stays. Both finders document
  that a pair of stations closer together than the step is not reported, and
  that for the true node such pairs are grazes whose existence depends on the
  ephemeris.
- The true node's corpus span is 1990–2030, not the full window.
- Planets keep the exact count-and-kind-sequence rule.

**Test backends.** No test backend in the workspace reports a speed
(`LinearSunMoon` does not), so the engine tests run on `packaged_backend()`
over short spans, and `LinearSunMoon` provides the `MissingSpeed` case. The
gate's comparison rules are pure functions over two station lists and are
unit-tested with synthetic lists.

**Non-finite speed.** A NaN or infinite longitude speed is treated as
missing and returns `MissingSpeed`; it is never compared against zero.

## Amendment (2026-10-02, implementation measurements)

- **Separation is 3 days, not 2.** The measurement found one-sided graze
  pairs about 2.5 days wide; 3.0 is the smallest swept value (2.0, 2.5, 3.0,
  3.5, 4.0, 5.0) at which every separated station has a same-kind
  counterpart. `SEPARATION_DAYS = 3.0`.
- **The true-node gate is a coarse existence-and-kind check, not a timing
  check.** Its ceilings are 270 000 s (about three days: the larger of the
  corpus-side residual 117 467.6 s and the engine-side distance 176 947 s,
  times 1.5) and 78″. The node's speed hovers near zero for days, so a
  station instant is ill-conditioned; a tighter comparison needs a different
  metric.
- **Swiss Ephemeris speed convention.** Moshier planet speed is a backward
  difference over `PLAN_SPEED_INTV` = 0.0001 day (`swemplan.c`), so its speed
  zero lands 4.32 s late. That explains Mercury's engine-minus-corpus mean
  of about -4.2 s (99.5 % of stations negative, both kinds); it is included
  in the measured maxima and is not an engine defect. For the slower planets
  the mean is dominated by scatter; Pluto's -56.6 s mean is not explained by
  the convention, is well inside its 2000 s ceiling, and is recorded as an
  open observation in FU-21 item (f).
- **Reference-tool NaN.** Swiss Ephemeris Moshier returns a NaN true-node
  longitude speed at isolated grid instants (jd_tt 2451544.9 and 2451545.1).
  `tools/se-stations-reference` skips an isolated non-finite grid sample
  (logged to stderr) and aborts on anything else non-finite. No station lies
  near them.
- **Gate runtime: the Risks-section fallback was taken.** The full gate takes
  173 s in release but 339 s in the dev profile (opt-level 2) that
  `release-smoke` uses, above the plan's 300 s limit; it made up most of
  `release-smoke`'s 553 s against blocking CI's 10-minute target. The release
  battery (`run_all_numeric_gates`, so `release-smoke` and the battery tests)
  now runs `validate_stations_corpus_subset`: the whole-corpus checksum and
  row count, then the `mean` and `sid` series only (734 stations, floor 734,
  about 24 s in the dev profile). `release-smoke` fell to 150 s. The full gate
  (5542 stations) is unchanged and runs as `validate-stations` and as the
  `stations_gate_passes_within_ceilings` test in nightly `test-full`, which
  `release-gate` depends on.
