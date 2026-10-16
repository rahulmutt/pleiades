# Aspects at several angles in one scan (design)

**Status:** design approved, spec under review · **Opened:** 2026-10-16 ·
**Issue:** #168 (e) · **Crates:** `pleiades-events`, `pleiades-validate`,
`pleiades-cli`, `pleiades-core` (compatibility notes only)

## Context

`EventEngine::aspects_in_range`, `next_aspect` and `previous_aspect` take one
angle. A caller after several angles for one pair, such as the
`validate-aspects` gate (0, 60, 90, 120 and 180 degrees for each of its
pairs) or `pleiades-cli aspects --angle ... --angle ...`, scans the pair once
per angle, reading both longitudes at every grid sample each time. #168 (e)
leaves this open: "an engine-level batch call is still open ... the gate
scans each pair once per angle for that reason."

The level scanner already supports several levels in one pass. An angle
becomes the levels `±angle` (one level for 0 or 180), and
`root::scan_levels` brackets every level on one grid of samples and turning
points. What it does not do is say which level a root belongs to: it
returns bare instants.

## Goal

One scan per pair for any set of angles, through three new `EventEngine`
methods. The single-angle methods keep their behavior and results, and the
gate and the CLI's range search move to the new call.

Non-goals:

- **Batching across pairs.** Pairs share no separation, and sharing body
  reads between pairs with different steps would need a position cache. That
  is a separate change, not wanted now.
- **Any change to the gate's corpus, tolerances or counts,** or to the
  scanner's sampling, bracketing or bisection.

## Approach: the scanner tags each root with its level

`scan_levels` returns `(jd, level_index)` pairs instead of `jd`.
`level_roots_between` already loops over the levels with an index in reach,
and it sorts each bracket's new roots by instant. It now sorts the pairs by
instant and keeps each root's tag.

The grid, the turning points and so the brackets depend only on the sampled
function, the range, the step and the window, never on the levels. Each
level's root in a bracket is bisected on its own function, `wrap180(d − level)`.
So scanning the levels of several angles together yields, for each level,
exactly the instants a scan of that level alone yields, bit for bit. That is
the contract the tests check.

Rejected alternatives:

- **Classifying each root afterwards by its nearest level.** This leaves the
  scanner untouched, but is ambiguous for two angles closer together than the
  bisection noise (about 1e-4 degrees for a fast pair over the 0.5 s
  tolerance). It would also need a second separation read per root.
- **Looping over single-angle scans and merging.** This gives no speedup,
  which is the point of the change.

The `pub(crate)` wrappers change with it:

- `level_crossings_in_range` returns `Vec<(f64, usize)>`.
- `first_level_crossing_after` and `last_level_crossing_before` return
  `Option<(f64, usize)>`.

Their only callers are `aspects.rs` and `root/level_tests.rs`. The tests
compare bare instants at about 40 sites, so they strip the tags through one
`instants(...)` helper instead of being rewritten one by one.

## API

```rust
impl<B: EphemerisBackend> EventEngine<B> {
    pub fn aspects_in_range_for_angles(
        &self, first: CelestialBody, second: CelestialBody, angles: &[Angle],
        reference: impl Into<CrossingReference>, start: Instant, end: Instant,
    ) -> Result<Vec<AspectEvent>, EventError>;

    pub fn next_aspect_for_angles(
        &self, first: CelestialBody, second: CelestialBody, angles: &[Angle],
        reference: impl Into<CrossingReference>, after: Instant,
    ) -> Result<Option<AspectEvent>, EventError>;

    pub fn previous_aspect_for_angles(
        &self, first: CelestialBody, second: CelestialBody, angles: &[Angle],
        reference: impl Into<CrossingReference>, before: Instant,
    ) -> Result<Option<AspectEvent>, EventError>;
}
```

Each event's `AspectEvent::angle` is the requested angle it belongs to
(already a field), so no type changes.

### Semantics

- **Range.** All events of every angle in `[start, end]`, ascending by
  instant. For each angle, the events are exactly those of
  `aspects_in_range` for that angle, bit for bit. Two angles cannot be exact
  at the same instant: they are distinct separations, and the angles are
  required to be distinct.
- **Next / previous.** The earliest (latest) event of any angle, strictly
  after `after` (at or before `before`), under the same rules as the
  single-angle method. The result equals the earliest (latest) of the
  single-angle results, bit for bit. The search stops at the first chunk or
  bracket that holds an event of any angle.
- **Validation, in order.** An empty slice is `EventError::InvalidAspect`.
  Then each angle is validated as today (finite, 0–180 degrees). Then two
  equal angles (compared by degrees) are `InvalidAspect`. Then the bodies,
  window, frame and zodiac are checked once, as today.
- **Out-of-reach angles (#168 (g)).** An angle beyond the pair's
  `separation_bound` contributes no levels. If at least one angle is in
  reach, the scan runs over the angles in reach: the range result simply
  lacks the others, and next or previous finds the nearest event among them.
  If none is in reach, the call answers as a single unreachable angle does
  today: an empty range, or `EventError::OutOfWindow` one step past the
  window's end (before its start), after the same one read for backend
  errors.
- **Window end.** Next or previous with no event of any in-reach angle
  before the window's limit is `OutOfWindow`, as today. There is no partial
  result.

The single-angle methods become wrappers: each calls the batch method with
`&[angle]`. The levels, step and scan they produce are unchanged.

## Callers

- **`pleiades-validate` (`aspects_validation.rs`).** One
  `aspects_in_range_for_angles` call per pair over `ANGLES_DEG`. Its events
  are split by `event.angle` into the per-angle lists the existing
  `compare_exact` and counts consume, so the report lines are unchanged.
  The full gate (`mise run gate-aspects`) must pass with every count
  unchanged. Its wall-clock before and after is measured on one machine,
  with the load recorded, and stated in the PR. No speedup figure is
  claimed in advance. The expectation is up to about 5× fewer separation
  samples per pair, less where bisections dominate.
- **`pleiades-cli aspects`.** A range search makes one batch call per pair,
  and the events are merged across pairs in time order as now. The output is
  unchanged for distinct angles. `--next` and `--previous` keep their
  per-angle loop, because they print one event per angle and the batch call
  would print one in total. **Behavior change:** a repeated `--angle` is now
  a usage error ("--angle <degrees> given twice"). Before, it printed every
  event twice.

## Testing

- **Bit-identity property** (`pleiades-events` integration tests,
  packaged backend). Over a pair set that covers a fast pair (Sun–Moon), a
  slow pair with a turning separation (Jupiter–Saturn), a bounded pair with
  a mixed in-reach/out-of-reach set (Sun–Mercury with 20 and 60 degrees),
  and the heliocentric and sidereal references, and for an angle set mixing
  0, 180 and two-sided angles:
  - the range result equals the time-merge of the single-angle results,
    compared field for field with `==` on `f64` (bit-identical);
  - next and previous equal the earliest and latest of the single-angle
    answers, also from a returned event's instant, so chaining is covered.
- **Validation:** an empty slice, a repeated angle, an out-of-range angle
  among valid ones, and the order those are reported in.
- **All angles out of reach:** the same result and error as a single
  unreachable angle.
- **`root::level_tests`:** the synthetic tests check the tags. The
  turning-point split case gets a two-level variant, where both levels fall
  inside one step and each root carries its own index.
- **CLI:** the existing `aspects` tests pass unchanged, plus a test for the
  repeated-`--angle` error.
- **Gate:** `mise run gate-aspects` (full) and the `release-smoke` mean
  subset pass.

## Documentation and release

- Rustdoc on the three methods, with a runnable example (Sun–Moon quarters
  over one month: 0, 90 and 180 degrees, four events in time order). The
  single-angle docs link to the batch methods.
- The `pleiades-events` README aspects section.
- `pleiades-core` compatibility notes: an entry for the additions and the CLI
  behavior change, and the profile bumped `0.7.36 → 0.7.37`. The API stability
  profile is unchanged, since `pleiades-events` is outside the façade's
  stable surface.
- #168: item (e) checked off, with the measured gate timing.
