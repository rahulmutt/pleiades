# Event finder for exact aspects between two bodies (issue #84) — design

**Status:** approved (2026-10-02), amended after the pre-plan probe ·
**Opened:** 2026-10-02 · **Issue:** #84 ·
**Crates:** `pleiades-events`, `pleiades-validate`, `pleiades-cli` (gate
command routing only), `pleiades-core` (compatibility profile entry only) ·
**Tool:** `tools/se-aspects-reference` (new)

## Context

`EventEngine` finds when one body crosses a fixed longitude
(`longitude_crossings_in_range`) and when a body stations
(`stations_in_range`). Nothing finds when the separation between two moving
bodies reaches a given angle, such as Mars square Saturn becoming exact. The
chart layer's aspect matchers (`pleiades-core` `chart/aspects.rs`) report
which aspects hold at one instant; none returns an instant.

Intended outcome: a caller asks `pleiades-events` for the exact moments of an
aspect between two bodies in a range, or the next one after an instant, and
gets instants validated once against an independent reference, instead of
writing a root-finder with its own bracketing and tolerance.

Decisions taken on 2026-10-02:

1. **Angle meaning:** an unsigned separation in [0°, 180°]. One call finds
   the aspect on both sides (for example the waxing and the waning square).
2. **Close pairs:** two exact moments inside one scan step are found, by
   splitting each step at the turning point of the separation.
3. **Turning points come from the separation itself**, not from longitude
   speed (approach A below).
4. **Reference authority:** a Swiss Ephemeris corpus. Swiss Ephemeris has no
   aspect finder, so a reference tool bisects its own longitudes.
5. **Scope:** the two finders the issue names, plus the gate. No
   `previous_aspect`, no user-facing CLI command.

### What already exists

- **Places.** `reference::ecliptic_in` returns a body's longitude in any
  `CrossingFrame` and zodiac; `check_supported` rejects a heliocentric Sun
  or Moon, a sidereal zodiac in the heliocentric frame, and an ayanamsa with
  no finite offset.
- **Root-finding.** `src/root.rs` brackets by stepping and refines by
  bisection, with a guard against the ±180° wrap seam. `bisect` returns the
  later end of its final bracket (issues #80, #81), so a returned instant is
  "settled": the event has already happened there, less than 0.5 s earlier.
- **Step table.** `stations::step_days`: 0.25 day for the Moon and the lunar
  points, 1 day for the Sun, Mercury and Venus, 2 days otherwise.
- **Gate pattern.** `validate-stations`: a `tools/se-*-reference` generator
  built only under `devenv shell`, a committed CSV with a manifest checksum,
  a validation module and a thresholds module in `pleiades-validate`, a
  validated-row floor, and a subset in the release battery.

### Approaches considered

- **A. Split each step at the turning point of the separation, found from
  the separation's own samples (chosen).** Three consecutive samples whose
  slope changes sign reveal a turning point; a short search locates it; each
  monotone piece gets the ordinary sign-change test. Needs no longitude
  speed, costs about what a crossings scan costs, and cannot disagree with
  the longitudes the events are defined on. Only the separation's *value* at
  the turning point matters, not its time, so the flat-extremum objection
  that ruled out an extremum search for stations does not apply.
- **B. Split at the zero of the relative longitude speed.** Requires a speed
  from the backend (a `MissingSpeed` failure mode), about three place
  samples per evaluation, and a speed that is not exactly the derivative of
  the longitudes.
- **C. Fixed step, documented limit.** Reuses `root.rs` unchanged, but two
  exact moments closer than the step are dropped silently, which is the
  tight end of the retrograde-loop case the issue names.

## Public API

In `pleiades-events`, re-exported from `lib.rs`:

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    /// All exact moments of the aspect in `[start, end]` (TDB), ascending.
    pub fn aspects_in_range(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        start: Instant,
        end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError>;

    /// The first exact moment strictly after `after`, or `None`.
    pub fn next_aspect(
        &self,
        first: CelestialBody,
        second: CelestialBody,
        angle: Angle,
        reference: impl Into<CrossingReference>,
        after: Instant,
    ) -> Result<Option<AspectEvent>, EventError>;
}

#[non_exhaustive]
pub struct AspectEvent {
    pub first: CelestialBody,
    pub second: CelestialBody,
    /// The requested separation, in [0°, 180°].
    pub angle: Angle,
    /// Instant the aspect is exact (TDB).
    pub instant: Instant,
    /// Longitude of `first` at `instant`, in `frame` and `zodiac`.
    pub first_longitude: Longitude,
    /// Longitude of `second` at `instant`, in `frame` and `zodiac`.
    pub second_longitude: Longitude,
    pub frame: CrossingFrame,
    pub zodiac: ZodiacMode,
}
```

`AspectEvent` derives `Clone`, `Debug` and `PartialEq`, and carries serde
under the existing `serde` feature, as `Crossing` and `Station` do. The
signatures mirror the crossings and stations finders: two instants rather
than a range type, and `impl Into<CrossingReference>`.

### Semantics

- **Definition.** Let `d(t) = wrap180(lon(first) − lon(second))`, both
  longitudes read in `reference` at the same instant. An event is a sign
  change of `wrap180(d − angle)` or of `wrap180(d + angle)`. For 0° and 180°
  the two conditions are the same, so there is one target.
- **Which side.** There is no side field. The two longitudes say which body
  is ahead; `first_longitude − second_longitude` is `+angle` or `−angle` to
  within the motion over the bisection tolerance.
- **No applying/separating flag.** At the exact instant an aspect is
  neither.
- **Settled instant.** The returned instant is the later end of the final
  bisection bracket. It trails the sign change by less than 0.5 s and never
  precedes it, so `next_aspect(after = e.instant)` returns the following
  event, not `e` again.
- **`next_aspect` and `aspects_in_range` agree.** `next_aspect(a, b, x, r,
  t)` equals the first element of `aspects_in_range(a, b, x, r, t,
  WINDOW_END)` that is strictly after `t`. Both run the same scanner.
- **Turn-back.** A pair that approaches the angle and turns back before
  reaching it returns an empty `Vec` or `None`. It is not an error.
- **Sidereal zodiac.** The ayanamsa is subtracted from both longitudes at
  the same instant and cancels in `d`. Sidereal events fall at the tropical
  instants (to within the bisection tolerance); only the reported longitudes
  differ.
- **Orbs** need no API: the moment a pair enters a 3° orb of a square is
  the exact moment of the 87° or 93° separation.
- **Instants** are read as TDB, as everywhere else on the crossings surface.

### Accuracy of an aspect instant

The 0.5 s tolerance bounds how well the engine locates the zero of its own
separation. How well that matches another ephemeris is the separation error
divided by the pair's relative speed, and the relative speed falls to zero at
a turning point. An aspect instant is therefore firm for a fast pair (seconds
for the Moon) and soft for a slow pair near a station of either body (hours
or more). The rustdoc states the per-pair figures the gate measures and does
not claim a blanket precision.

## Errors

One new variant, not breaking because `EventError` is `#[non_exhaustive]`
since #85:

```rust
/// An aspect request that is not defined: a non-finite angle, an angle
/// outside [0°, 180°], or the same body twice.
EventError::InvalidAspect { detail: String }
```

Checks run in this order: the angle, the two bodies being distinct, the
window (`check_window` on each instant), then `check_supported` for `first`
and for `second` with the label `"aspects are"`. All other failures map to
existing variants as in `longitude_at`: `OutOfWindow`, `UnsupportedFrame`,
`MissingCoordinates`, `Backend`. The change ships as `feat(events)`.

## Algorithm and layout

- **Module.** New `crates/pleiades-events/src/aspects.rs` holding
  `AspectEvent` and the two finders, with tests in `src/aspects/tests.rs`.
- **Scanner.** `root.rs` gains a level-crossing scanner beside the existing
  functions, which are not changed. It takes a wrapped-degree function
  `d(jd)`, a list of levels (one or two), a range and a step, and yields the
  instants at which `d` equals any level, ascending. `aspects_in_range`
  collects them; `next_aspect` stops at the first one. One implementation
  serves both, with one step of look-ahead.
  1. Sample `d` on the grid `lo + k·step`.
  2. For three consecutive samples, form the wrapped increments
     `wrap180(d1 − d0)` and `wrap180(d2 − d1)`. Opposite signs mean a turning
     point in `(t0, t2)`.
  3. Locate it by a ternary (or golden-section) search on the locally
     unwrapped `d` over `[t0, t2]`, down to `REFINE_TOLERANCE_DAYS`, and add
     it as a breakpoint.
  4. Between consecutive breakpoints (grid points and turning points), apply
     the existing test per level: a sign change of `wrap180(d − level)` with
     a jump under 180°, refined by `bisect`.
- **Properties.**
  - A falsely detected turning point (noise in a nearly flat separation)
    only adds a breakpoint. It cannot create an event, because an event
    still requires a sign change.
  - One, two or three exact moments in a step are all found when the step
    holds at most one turning point.
  - Zero counts as the negative side, as in `bisect`. An exact tangency
    (the turning value equal to the level in floating point) reports no
    event.
- **Step.** The smaller of `stations::step_days` for the two bodies (made
  `pub(crate)`). The station table is the right one because turning points
  of the separation come from the bodies' speed changes.
- **Window clamp.** The scan runs over `[max(start, WINDOW_START + step),
  min(end, WINDOW_END − step)]`, as crossings do.
- **Event construction.** Both longitudes are read by `ecliptic_in` at the
  settled instant.
- **Known limits**, documented on both finders:
  - two turning points of the separation inside one step are not resolved
    (this needs a separation that reverses twice within 0.25 to 2 days);
  - an overshoot of the angle smaller than the ephemeris noise is not seen;
  - an event within one step of either window end is not reported.
- **Cost.** Two place samples per step, plus a few dozen at each turning
  point, which are rare. Measured in the pre-plan probe and recorded.

### To verify first *(resolved — see the amendment at the end)*

A scratch probe on the packaged backend, before the plan is written,
measures:

1. the cost per step and of a full-window scan of a fast and a slow pair;
2. how often the three-sample test fires on smooth stretches (false turning
   points cost time, not correctness);
3. the difference between the pleiades and Swiss Ephemeris separations for
   the corpus pairs, which sets the graze margin and the ceilings below.

If a measurement contradicts this design, the design returns to the user as
an amendment before the plan, as the true-node finding did for #85.

## Validation

### Reference tool

`tools/se-aspects-reference`, modelled on `tools/se-stations-reference`
(vendored Swiss Ephemeris through `libswisseph-sys`, Moshier mode, built only
under `devenv shell`, never by the workspace build). For each group, pair and
angle it finds the exact moments by a mechanism independent of the engine's:
it scans on a fine grid (0.05 day; 0.01 day for a pair with the Moon), splits
at the zeros of Swiss Ephemeris's own relative longitude speed
(`SEFLG_SPEED`), and bisects each sign change to 1e-7 day.

It writes two kinds of row to one CSV *(superseded: no `graze` rows — see
the amendment at the end)*:

- **`event`**: group, first, second, angle, TT Julian day, both longitudes,
  and the relative longitude speed at the event.
- **`graze`**: group, first, second, angle, and a TT interval. One is
  written for every turning point of the separation that comes within the
  graze margin of a level, on either side of it. The interval is the stretch
  around the turning point during which the separation is within the margin
  of the level.

The graze margin is one constant shared by the tool and the gate, set from
the probe to several times the largest separation difference between the two
ephemerides (30″ is the starting value).

| Group | Pairs | Span | Flags |
|---|---|---|---|
| `geo` (geocentric apparent, tropical) | Sun–Mercury, Mercury–Venus, Venus–Mars, Mars–Jupiter, Mars–Saturn, Jupiter–Saturn, Saturn–Pluto | 1900–2100 | `MOSEPH \| SPEED` |
| `geo` | Sun–Moon | 1990–2030 | `MOSEPH \| SPEED` |
| `mean` (geocentric mean of date) | Mercury–Venus, Mars–Saturn | 1990–2030 | plus `TRUEPOS \| NOABERR \| NOGDEFL \| NONUT` |
| `helio` (heliocentric) | Mars–Jupiter | 1900–2100 | the flags `se-helio-reference` uses |

Angles: 0°, 60°, 90°, 120°, 180° for every pair. Sun–Mercury at 60° and
above, and Mercury–Venus at 90° and above, are the "never perfects" case:
the corpus has no rows and the engine must return none. There is no sidereal
group, because the ayanamsa cancels; a unit test covers it.

### Gate

`validate-aspects` in `pleiades-validate`: `aspects_validation.rs`,
`aspects_validation/tests.rs`, `aspects_thresholds.rs`, corpus under
`data/aspects-corpus/` with a manifest checksum, wired into `render/cli.rs`,
the `pleiades-cli` validate routing and the release posture alongside
`validate-stations`.

For each group, pair and angle it calls
`EventEngine::new(packaged_backend()).aspects_in_range(..)` over the corpus
span and applies:

1. *(Superseded — see the amendment at the end: the gate is strictly event
   for event.)* **Graze zones are unconstrained.** Engine events and corpus events that
   fall inside a `graze` interval for that pair and angle are set aside.
   Whether a near-tangent pair of events exists depends on the ephemeris, so
   neither side is required to have them.
2. **Outside graze zones, event for event.** The remaining engine and
   corpus lists must have the same length and, in order, the same side (the
   sign of `first − second`). Nothing is matched by nearest neighbour; a
   missed or spurious event fails the gate.
3. **Separation residual.** `|Δt| × |relative speed|` (the corpus row's
   speed), in arcseconds, within a per-pair ceiling. This is the time
   residual expressed as an angle, so one ceiling is meaningful for both a
   fast event and a slow one.
4. **Longitude residuals** of both bodies within per-pair ceilings.
5. **Validated-row floor** enforced on the release path.

The largest time residual per pair is reported and documented but has no
ceiling of its own. Ceilings are set from measured maxima with headroom and
recorded in the thresholds module and `docs/follow-ups.md`. A residual the
Moshier-versus-packaged difference does not explain is investigated before a
ceiling is set over it. The tool and the gate use the same spans, which start
and end five days inside the window.

**Tiers**, as for stations: the release battery (`run_all_numeric_gates`,
blocking `release-smoke`) verifies the whole corpus's checksum and row count
and compares the `mean` group only; the full gate runs under
`validate-aspects` and in nightly `test-full`, which `release-gate` depends
on. The subset's runtime is measured against the blocking budget and trimmed
if it does not fit.

The gate's comparison rules are pure functions over two event lists and a
list of graze intervals, unit-tested with synthetic lists.

### Unit and regression tests

`root.rs`, on synthetic functions: a single crossing; a tangent that does not
reach the level (no event); two crossings inside one step; three inside one
step; a turning point exactly on a grid point; a crossing at the ±180° seam;
two levels returned in ascending order; first-after equal to the first in
range.

`src/aspects/tests.rs`, on `packaged_backend()` over short spans:

- a known triple perfection through a retrograde loop: three events, in
  order, each with the separation at the angle;
- a turn-back: a span in which a slow pair approaches an angle and does not
  reach it returns empty and `None`;
- 0° and 180° across the wrap seam (a Sun–Moon lunation: one conjunction and
  one opposition per month, none doubled);
- an angle strictly between 0° and 180° returns both sides, ascending;
- `next_aspect` equals the first in range; chaining from a returned instant
  returns the following event;
- swapping `first` and `second` returns the same instants with the
  longitudes exchanged;
- sidereal and tropical instants agree within the tolerance, and the
  longitudes differ by the ayanamsa;
- `InvalidAspect` for NaN, a negative angle, an angle above 180° and the
  same body twice; an out-of-window range fails closed; a heliocentric Sun
  or Moon is `UnsupportedFrame`.

`validate-crossings`, `validate-stations` and the `crossings-golden`
manifest must be unchanged; no existing function in `root.rs` is touched. A
diff in any of them is a defect.

## Documentation

- Rustdoc on both finders and `AspectEvent`, with a doctest on the packaged
  backend, stating units, frame, the unsigned-angle definition, the
  settled-instant contract, the known limits, the measured accuracy and the
  failure modes.
- `crates/pleiades-events/README.md` and the workspace `README.md`
  capability table. `CHANGELOG.md` is written by release-plz from the
  `feat(events)` commit; it is not edited by hand.
- `docs/follow-ups.md`: a resolved FU-22 entry with the measured gate figures
  and the deferred items below.
- `spec/astrology-domain.md` and any `spec/*.md` list of event surfaces or
  validation gates gain the aspect finder.
- The `pleiades-core` compatibility profile gains an entry for the new
  surface, with its pinned checksum and the tests that pin the identifier.

## Out of scope

- `previous_aspect` (it would inherit the FU-13 backward-search caveat).
- A user-facing `pleiades-cli` aspects command.
- A batch call over several angles or several pairs.
- Aspects in declination (parallels), to fixed stars, or from a topocentric
  place.
- Asteroids, fictitious bodies and the lunar points in the gate. The finders
  accept them; their aspects are ungated and documented as such.

## Risks

- **Grazes** *(superseded — see the amendment at the end)*. The graze margin must be large enough that no event outside a
  zone is model-dependent, and small enough that the zones do not swallow
  the retrograde-loop cases the gate exists to check. The probe measures
  both; the count of events set aside is reported in the gate's summary.
- **Soft instants.** For slow pairs the time residual near a turning point
  may be hours. The gate relies on the separation residual there, and the
  documentation gives the measured figures.
- **Reference speed convention.** Swiss Ephemeris's Moshier speed is a
  backward difference (FU-21(f)). It enters the corpus only through the
  turning-point split and the residual's scale factor, not through the event
  instants.
- **Gate runtime.** Eight full-window pairs at five angles each. If the full
  gate does not fit the nightly budget, the scan is shared across the five
  angles of a pair inside the gate (one pass, five level sets) before any
  span is cut.

## Amendment (2026-10-02, pre-plan probe)

A scratch probe on the packaged backend (geocentric apparent, optimised
build) ran a prototype of the scanner over the eight `geo` pairs at the five
angles, with the spans in the table above.

**Cost.** 0.12 to 0.22 ms per separation sample (two place samples). One
full-window pass over a pair, serving all five angles, took 1.3 s
(Mars–Jupiter) to 14 s (Mercury–Venus). Each turning point costs about 64
further samples, which roughly doubles the samples for a pair with Mercury
(1260 turning points against 73 000 grid points).

**No false turning points.** The three-sample test fired 0 times for
Sun–Moon, and for every other pair exactly as often as the faster-stationing
member stations (Sun–Mercury 1260, Mercury–Venus 1252, Venus–Mars 250,
Mars–Jupiter and Mars–Saturn 188, Jupiter–Saturn 366, Saturn–Pluto 386).

**The split found no extra events in this corpus.** For all 40 pair–angle
cases the plain sign-change scan, the turning-point scan and a turning-point
scan at a quarter of the step returned the same number of events, and the
same instants. The closest two events were 1.59 days apart (Mercury–Venus
conjunctions) against a 1-day step. The split stays, as decided: the margin
is thin, and the scanner's close-pair behaviour is covered by the synthetic
`root.rs` tests, not by the corpus.

**No turning point comes near an angle.** The closest was 0.028° (102″,
Venus–Mars at 60°), then 0.047° (Mercury–Venus at 0°). The pleiades and
Swiss Ephemeris planet longitudes differ by a few arcseconds (the stations
gate's largest is 2.314″), so no event in the corpus is model-dependent.
The separation difference itself is measured once the reference tool
exists, before any ceiling is set.

Decision (user, 2026-10-02): **no graze zones.**

- The reference tool writes `event` rows only. If any turning point of a
  corpus pair's separation comes within the graze margin (30″) of one of its
  levels, the tool fails without writing a corpus and names the pair, angle
  and instant. A future corpus change that would introduce a
  model-dependent event is then caught at generation time and returns as a
  design question.
- The gate has no rule 1. For every group, pair and angle the engine and
  corpus lists must have the same length and, in order, the same side; the
  separation and longitude residuals and the validated-row floor apply as
  written.
- Expected `geo` event counts, for the plan's cross-check against the
  corpus (0°, 60°, 90°, 120°, 180°): Sun–Moon 494, 990, 990, 990, 495;
  Sun–Mercury 1261, 0, 0, 0, 0; Mercury–Venus 496, 476, 0, 0, 0; Venus–Mars
  167, 281, 268, 236, 94; Mars–Jupiter 91, 209, 209, 201, 132; Mars–Saturn
  101, 213, 219, 223, 116; Jupiter–Saturn 14, 42, 52, 66, 36; Saturn–Pluto
  10, 28, 26, 30, 15.
