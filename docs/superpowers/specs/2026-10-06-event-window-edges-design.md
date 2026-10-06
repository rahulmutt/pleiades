# Event searches at the window's ends (issue #208)

Status: approved design, 2026-10-06. Follow-up to #203 / #207, which made the
rise, set and transit searches reach the window's ends.

## Problem

The longitude-crossing, station, exact-aspect and occultation searches of
`pleiades-events` stop short of each end of the 1900–2100 window:

| Search | Margin at each end | Where |
|---|---|---|
| Crossings (`longitude_crossings_in_range`, `next_/previous_longitude_crossing`, `next_sun/moon_crossing`) | one step (0.25–2 d) | `crossings.rs` |
| Stations (`stations_in_range`, `next_/previous_station`) | one step | `stations.rs` |
| Aspects (`aspects_in_range`, `next_/previous_aspect`) | two steps (0.5–4 d) | `aspects.rs` (`Search::earliest/latest`) |
| Occultations (`next_/previous_occultation`, `next_global_occultation`) | one conjunction step (0.25 d) | `occult.rs` |

An event inside a margin is not returned, and `next_*`/`previous_*` return
`Ok(None)` both when the window ends before the next event and when no event
exists. A caller cannot tell the two apart.

## Decisions

1. Every search scans up to the window's limits themselves.
2. A `next_*`/`previous_*` search that reaches a window limit without an event
   returns `EventError::OutOfWindow`, as the rise/set searches do since #207:
   the event may exist past the window but cannot be computed. `julian_day`
   names the first instant past the window the search needed (the next grid
   instant beyond the limit).
3. `Ok(None)` keeps only the meaning "the engine knows there is no event":
   a body that never stations in the frame (`never_stations`), and a target
   that can never be occulted (`target_never_occultable`).
4. A range search is never cut short (both ends are checked to lie in the
   window), so it keeps returning a list. The one exception is the
   light-time limit below.
5. Chosen over keeping `Ok(None)` (leaves the ambiguity) and over a new
   `Found / NoEvent / WindowEnded` result type (breaks eight public
   signatures for a distinction the error already carries).

### Light-time at the window's start

An apparent place of a body other than the Sun is read a light-time before
the instant (up to about 0.3 d for Pluto), so it is `OutOfWindow` within a
light-time of the window's first instant (#163). A search that needs a sample
there is `OutOfWindow`, as `position_at` and `previous_rise_set` are. In
particular `longitude_crossings_in_range`, `stations_in_range` and
`aspects_in_range` with a start inside that sliver now return `OutOfWindow`
for such a body, where the old one-step clamp quietly started later. The Sun,
the lunar points and the mean-of-date and heliocentric frames are read from
the first instant on.

## Design

### Crossings, stations, occultations

Remove the `WINDOW_START_JD + step` / `WINDOW_END_JD - step` clamps. The
scanners `root::crossings_in_range`, `first_crossing_after` and
`last_crossing_before` already cut their last interval at the range end and
never sample outside `[lo, hi]`, so they need no change.

`first_crossing_after` / `last_crossing_before` return `Ok(None)` when the
range holds no root. The `next_*`/`previous_*` callers whose range runs to a
window limit turn that into `OutOfWindow` (decision 2). A small helper in
`root.rs` (or the callers) names the instant: `limit + step` for a forward
search, `limit - step` for a backward one.

Occultations: the conjunction scan runs on absolute multiples of
`OCC_CONJUNCTION_STEP_DAYS`; its limits become `WINDOW_START_JD` /
`WINDOW_END_JD` (both are multiples of 0.25 d, so the grid stays aligned).
Exhausting the scan returns `OutOfWindow`. The refinement around a
conjunction already clamps to the window (`occult.rs` 756, 835, 861, 1130).

### Aspects: `root::scan_levels`

The level scanner reads one step before its range and one step past its grid
to see turning points of the separation (`at(-1)`, `at(intervals + 1)`).
These two look-around samples become clamped to window limits passed in by
the caller (`scan_levels` gains `window: (f64, f64)`; the public wrappers
pass it through):

- a look-around instant outside the window is replaced by the limit; if that
  coincides with the neighbouring sample, there is no look-around sample;
- a look-around read that fails with `OutOfWindow` (light-time at the start)
  counts as no sample;
- with no look-around sample, no turning point is looked for in that step.

`turning_point` does not assume equal spacing (it uses the signs of the two
increments and a ternary search over `[p0, p2]`), so a short edge step is
sound. `aspects.rs` drops `earliest`/`latest` margins (the `Search` limits
become the window limits).

Remaining documented limit: when a range ends at the window's end, two exact
moments within the final step, either side of a turning point, are not seen
(no sample exists past the end). The same at the start. Today the blind zone
is two whole steps.

`last_level_crossing_before`'s chunked backward walk is unchanged apart from
passing the window through.

### Errors and docs

- `EventError::OutOfWindow` doc: the "search cut short" paragraph covers
  every search, not only rise, set and transit.
- Each affected method: remove the margin text; replace "runs to the end of
  the window before returning `None`" with the `OutOfWindow` answer; state
  when `Ok(None)` is returned.
- `pleiades-core` compatibility summary: new release note entry; the #203
  entry's "Stations, aspects, crossings and occultations keep their margins"
  stays as history. Bump the compatibility profile id and re-pin its
  checksum and the CLI/validate tests that pin the summary text.
- Crate README (`pleiades-events`) wherever the margins are mentioned.

### CLI (`pleiades-cli stations`, `aspects`)

`--next`/`--previous` loop over several bodies, pairs and angles. One that
exhausts the window (a pair that never reaches its angle) must not abort the
others. The CLI maps `OutOfWindow` from a `next_*`/`previous_*` call to a
note line for that entry, "none before the window's end (2100-01-01)" /
"none after the window's start (1900-01-01)", and continues. Range searches
and every other error still fail the command. `docs/cli.md` is updated.

## Testing

Regression tests, co-located per family (`*/window_edge_tests.rs`, sharing a
helper for "the `julian_day` an `OutOfWindow` names"):

- an event inside the old margin at each end is found by the range search
  and by `next`/`previous` (crossings: a Moon crossing within 0.25 d of
  2100-01-01; stations: the nearest station to each end; aspects: a Sun–Moon
  aspect within 0.5 d of an end; occultations: the last conjunction-step
  case if one exists, otherwise exhaustion only);
- an exhausted `next`/`previous` returns `OutOfWindow` with the expected
  instant; `never_stations` and never-occultable targets still return
  `Ok(None)`;
- a planet's apparent range starting at `WINDOW_START_JD` returns
  `OutOfWindow`; the Sun's does not.

Expected instants come from the repo's Swiss Ephemeris reference tools
(`tools/se-crossings-reference`, `se-stations-reference`,
`se-aspects-reference`) where they produce them; otherwise the test checks
the sign change of the engine's own evaluator (`position_at`,
`longitude_at`) across the returned instant, which tests the scanner's reach
and not the ephemeris.

Unit tests in `root.rs` for `scan_levels` with clamped look-around samples:
a turning point in the first and last full step next to a limit is still
split; a look-around read failing with `OutOfWindow` is tolerated; no sample
is ever requested outside the window.

Gates: `mise run gate-stations`, `mise run gate-aspects`, `release-smoke`,
`validate-crossings` must pass. If a corpus holds events inside the old
margins, the gate's matching (not its tolerances) is adjusted. Then
`mise run ci` and `mise run test-full`.

## Out of scope

- `pleiades-eclipse`'s own window handling.
- The occultation found from a conjunction just past the window's end whose
  maximum would fall inside it: the conjunction is out of reach, so the
  search reports `OutOfWindow`.
- #168 (a)–(c) (turning-point coverage of the corpus).
