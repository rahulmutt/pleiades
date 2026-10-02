# Convert a TT or TDB instant back to civil UTC/UT1 (issue #87) — design

**Status:** approved design, not yet implemented ·
**Opened:** 2026-10-02 · **Issue:** #87 ·
**Crates:** `pleiades-time`, `pleiades-core` (re-export only)

## Context

`pleiades-time` converts a civil UTC or UT1 datetime into a TT or TDB
`Instant` (`to_terrestrial`, `src/convert.rs`). Nothing converts the other
way. Every event finder (crossings, eclipses, rise and set, occultations)
returns a TDB instant, so a caller working in civil time must write its own
inverse and repeat the leap-second table and the ΔT model to do it. Two
callers can then disagree with each other and with the forward conversion.

The goal is one validated inverse that agrees with the forward conversion by
construction and by test.

Decisions taken on 2026-10-02:

1. An instant inside an inserted leap second is returned as `23:59:60.x`.
2. The forward conversion's handling of `:60` is fixed in the same change, so
   the round trip holds through a leap second.
3. The inverse leap lookup is a direct lookup on the TAI axis, not an
   iteration through `leap::tai_minus_utc`.

### What already exists

- **Forward.** `to_terrestrial(civil, source, target)` turns the civil
  datetime into a Julian day, then `to_tt` adds an offset chosen by source
  scale and epoch:

  | Source | Epoch | Offset | Path / quality |
  |---|---|---|---|
  | UTC | before 1972 | error `UtcBeforeLeapEpoch` | — |
  | UTC | 1972 to `leap::VALID_THROUGH_JD` | `TAI−UTC + 32.184 s` | `UtcLeapSecond` / `Exact` |
  | UTC | past the leap horizon | `ΔT(jd_civil)` | `FutureExtrapolated` / `Predicted` |
  | UT1 | whole window | `ΔT(jd_civil)` | `Ut1DeltaT` / `Observed`, or `FutureExtrapolated` / `Predicted` |

  TDB is `TT + tdb_minus_tt_seconds(jd_tt)`. The window is
  `[SUPPORT_START_JD, SUPPORT_END_JD)`, checked on the civil Julian day.
- **Leap table.** `data/leap-seconds.csv`: 28 rows of
  `(effective_jd_utc, tai_minus_utc)`: the 1972 epoch at 10 s, then 27
  insertions up to 37 s on 2017-01-01. All insertions are positive.
- **Calendar.** `CivilDateTime::to_julian_day` validates `second` in
  `[0, 61)` on any day. `from_julian_day` rounds the time of day to the
  millisecond and never returns `second >= 60`.
- **Partial inverse.** `ut1_jd_from_tt` returns a bare UT1 Julian day using
  `ΔT` looked up at the TT epoch. It serves sidereal time and is unchanged.

### The forward `:60` defect

`CivilDateTime::new(2016, 12, 31, 23, 59, 60.5)` produces the same Julian day
as `2017-01-01T00:00:00.5`, because a Julian day cannot express the 86,401st
second of a day. `to_tt` then looks up the *new* offset (37 s) and returns the
TT of `00:00:00.5`. The correct TT is one second earlier: `23:59:60.5` is
`23:59:59.5` plus one second under the *old* offset (36 s). The same aliasing
silently accepts `:60` on days with no leap second and for UT1 input, where it
has no meaning.

## Design

### Public API

New in `pleiades-time`, re-exported from the crate root:

```rust
/// A civil datetime recovered from a TT/TDB instant, plus how it was produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CivilConversion {
    /// The civil datetime in `scale`. `second` is in `[60, 61)` only for a
    /// UTC result inside an inserted leap second.
    pub civil: CivilDateTime,
    /// The civil scale of `civil`: `Utc` or `Ut1`.
    pub scale: TimeScale,
    /// Same provenance vocabulary as the forward conversion.
    pub provenance: ConversionProvenance,
}

pub fn from_terrestrial(
    instant: Instant,
    target: TimeScale,
) -> Result<CivilConversion, CivilTimeError>;

pub fn utc_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError>;
pub fn utc_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError>;
pub fn ut1_civil_from_tt(instant: Instant) -> Result<CivilConversion, CivilTimeError>;
pub fn ut1_civil_from_tdb(instant: Instant) -> Result<CivilConversion, CivilTimeError>;
```

`from_terrestrial` reads the source scale from `instant.scale`. The four
conveniences fix the target and additionally require the named source scale,
returning `UnsupportedScale` on a mismatch, so a TT instant passed to
`utc_civil_from_tdb` is an error rather than a silent 1.7 ms shift.

The result is a `CivilDateTime`, not a UTC-tagged `Instant`: a Julian day
cannot hold a leap second, which is the defect described above.

`pleiades-core` adds `CivilConversion` and `from_terrestrial` to its existing
`pleiades_time` re-export line.

### Errors

No new `CivilTimeError` variants.

| Condition | Error |
|---|---|
| `instant.scale` is not TT or TDB, or `target` is not UTC or UT1 | `UnsupportedScale { source, target }` |
| instant's Julian day is not finite | `NonFiniteOffset` |
| UTC target and the instant is before 1972-01-01T00:00:00 UTC | `UtcBeforeLeapEpoch` |
| resulting civil Julian day outside `[SUPPORT_START_JD, SUPPORT_END_JD)` | `BeyondHorizon { jd }` (the civil Julian day) |
| a pinned table fails its checksum | `StaleTimeData` (propagated) |

For a UTC target, `BeyondHorizon` is checked before `UtcBeforeLeapEpoch`, so
an instant before 1900 reports the window, matching the forward's order.

### Algorithm

0. **Millisecond axis.** The inverse quantizes once, early, to a whole
   number of milliseconds (`ms_from_jd`: day number `floor(JD + 0.5)` times
   86 400 000 plus the rounded millisecond of day, as an `i64`). Leap
   thresholds are exact integers on that axis, so every comparison below is
   an integer comparison and a millisecond-aligned civil input round-trips to
   identical fields. The civil date comes from
   `CivilDateTime::from_julian_day` evaluated at an exact midnight; hour,
   minute and second come from the integer millisecond of day. Julian-day
   expressions below are shorthand for this integer arithmetic. An instant
   more than one day outside the support window is rejected as
   `BeyondHorizon` before quantizing, so the `i64` cannot overflow.

1. **To TT.** For a TDB instant, `jd_tt = jd_tdb − tdb_minus_tt_seconds(jd_tdb) / 86400`.
   The forward evaluates the periodic term at `jd_tt`; evaluating it at
   `jd_tdb` differs by under 1e-12 s and is not iterated.

2. **UTC target, leap-table era.** Let `jd_tai = jd_tt − 32.184 s`. For leap
   row `i` with `(effective_i, secs_i)`, define the TAI threshold
   `T_i = effective_i + secs_i / 86400` (the TAI instant of
   `00:00:00 UTC` on the effective day). The era ends at
   `T_end = VALID_THROUGH_JD + secs_last / 86400`, inclusive, matching the
   forward's inclusive `VALID_THROUGH_JD`.
   - `jd_tai < T_0` → before 1972: `BeyondHorizon { jd: jd_tt }` if
     `jd_tt < SUPPORT_START_JD`, otherwise `UtcBeforeLeapEpoch`.
   - Find the last row `i` with `jd_tai >= T_i`. Then
     `jd_utc = jd_tai − secs_i / 86400`, the civil datetime is built from
     that millisecond count, and `tai_minus_utc = Some(secs_i)`.
   - **Leap second.** If a next row exists and
     `jd_tai >= T_{i+1} − 1 s`, the instant is inside the inserted second.
     The fraction is `f = jd_tai − (T_{i+1} − 1 s)`, a whole number of
     milliseconds in `[0, 999]`. The civil datetime is the
     date of `effective_{i+1} − 1 day` at `23:59:(60 + f)`, with
     `tai_minus_utc = Some(secs_i)` (the offset in force during the leap
     second).
   - Provenance: `UtcLeapSecond` / `Exact`, `delta_t_seconds = None`.

   The leap-second test relies on every insertion being positive by one
   second. A table-construction test asserts consecutive rows differ by
   exactly `+1`, so a future negative leap second fails loudly instead of
   being mis-converted.

3. **UTC target, past the leap horizon** (`jd_tai > T_end`). Solve
   `jd_utc + ΔT(jd_utc) / 86400 = jd_tt` by fixed-point iteration starting
   from `jd_utc = jd_tt − ΔT(jd_tt) / 86400`. `ΔT` changes by under
   3e-8 s per second, so the iteration contracts by that factor; it runs a
   fixed three steps with no convergence test. Provenance:
   `FutureExtrapolated` / `Predicted`, `delta_t_seconds = Some(ΔT)`,
   `tai_minus_utc = None`. `ΔT` is anchored to the leap bound at the
   horizon, so this branch is continuous with step 2 at `T_end`.

4. **UT1 target.** The same fixed-point solve over `ΔT(jd_ut1)`, with path
   and quality mapped from `DeltaTQuality` exactly as the forward maps them.
   See "The 2020 node" below.

5. **Window.** The civil Julian day from step 2, 3 or 4 must lie in
   `[SUPPORT_START_JD, SUPPORT_END_JD)`.

Steps 3 and 4 share one private helper,
`civil_jd_from_tt_via_delta_t(jd_tt) -> Result<(f64, f64, DeltaTQuality), _>`.

### Forward fix

In `to_terrestrial`, before the Julian-day conversion:

- If `civil.second >= 60.0`:
  - source must be UTC, else `InvalidCivilDate { field: "second" }`;
  - hour and minute must be `23:59` and the *next* day `00:00:00` must be the
    effective Julian day of a leap row other than the first, else
    `InvalidCivilDate { field: "second" }`;
  - the conversion proceeds on the same datetime with `second − 1`, and one
    second is added to the resulting TT. The lookup then sees the old offset,
    which is the one in force.
  - Provenance reports the old `tai_minus_utc`.

A small `leap::is_insertion_day_end(jd_next_midnight) -> Result<bool, _>`
supports the check; it and the TAI thresholds are the only additions to
`leap.rs`.

`CivilDateTime::to_julian_day` keeps its `[0, 61)` range: it is scale-agnostic
and cannot know the leap table. Its rustdoc gains a sentence that `:60` aliases
the next day's first second in the bare Julian day and that `to_terrestrial`
is the leap-aware entry point.

### The 2020 node

`ΔT` is interpolated up to 69.4 s at the last observed node (2020-01-01) and
is 69.184 s (the leap-second bound) from the node on. The forward UT1
conversion therefore steps down 0.216 s there: TT values in
`[node + 69.184 s, node + 69.4 s)` have two UT1 preimages, one on each side of
the node. The inverse cannot be one-to-one where the forward is not.

The fixed-point start `jd_tt − ΔT(jd_tt)` evaluates `ΔT` at a TT epoch that is
already past the node for every instant in that window, so the iteration
settles on the post-node branch. This is the documented choice: in the
ambiguous 0.216 s, the inverse returns the UT1 at or after the node. The spec
does not remove the step; it predates this work and is described in
`deltat.rs`.

### Precision

- The round trip civil → TT/TDB → civil returns the starting datetime to
  within **1 ms**. `from_julian_day` rounds to the millisecond, and a Julian
  day near 2.46e6 resolves about 40 µs.
- Rounding happens once, on the millisecond axis, before any field is split,
  so a result never shows `second == 60.0` outside a leap second and never
  reaches `61.0` inside one.

Rustdoc on `from_terrestrial` states the 1 ms figure, the quality tiers, the
leap-second representation and the 2020-node choice, with a doctest that
recovers `2016-12-31T23:59:60.5` from its TT instant.

### Module layout

`src/convert.rs` is 271 lines. The inverse goes in a new submodule
`src/convert/inverse.rs` (with `src/convert/inverse/tests.rs`), so
`convert.rs` keeps the shared provenance types and the forward path. Shared
test setup (the list of insertion dates derived from the leap table) lives in
a `#[cfg(test)]` helper used by both test files.

### Testing

Unit tests (`pleiades-time`):

- **Every insertion.** For each of the 27 insertion rows: forward and inverse
  at `23:59:59.5`, `23:59:60.0`, `23:59:60.5` and next-day `00:00:00.0`,
  `00:00:00.5`. TT differences between consecutive samples are 0.5 s each
  (within Julian-day resolution), and each inverts to the datetime it came
  from with the expected `tai_minus_utc`.
- **Forward regression.** `2016-12-31T23:59:60.5 UTC` maps to the TT of
  `23:59:59.5` plus 1 s, not to the TT of `2017-01-01T00:00:00.5`.
- **Forward rejections.** `:60` on a non-insertion day, at a time other than
  `23:59`, on 1971-12-31 (the first row is an epoch, not an insertion), and
  with UT1 source: each `InvalidCivilDate { field: "second" }`.
- **Boundaries.** 1972-01-01T00:00:00 UTC round-trips; one millisecond
  earlier in TT gives `UtcBeforeLeapEpoch`. The leap horizon instant is
  `Exact`; one second later is `Predicted`, and the two civil results differ
  by one second to within 1 ms. Window start and end give `BeyondHorizon`
  just outside.
- **Scales.** Each unsupported source/target pair, and each convenience
  wrapper given the wrong source scale, gives `UnsupportedScale`.
- **2020 node.** An instant in the ambiguous window returns a UT1 at or after
  the node.
- **Table shape.** Consecutive leap rows differ by exactly `+1`.

Property tests. `proptest` is already a workspace dependency (root
`Cargo.toml`); `pleiades-time` has no dev-dependencies today and gains
`proptest = { workspace = true }` under `[dev-dependencies]`:

- UTC millisecond-aligned datetimes in `[1972, 2100]` → TT and TDB → UTC
  equal the input to 1 ms, with matching path, quality and `tai_minus_utc`.
- UT1 datetimes in `[1900, 2100]`, excluding 0.5 s either side of the 2020
  node → TT → UT1 equal the input to 1 ms.
- The inverse is monotonic: for `tt_a < tt_b` at least 2 ms apart, the UTC
  results compare in the same order (comparison on date, then time, with
  `:60` ordered after `:59`).

Blocking-tier commands: `cargo fmt --all --check`, strict clippy,
`mise run test`. The crossings golden is unaffected (no longitude moves).

### Documentation

- `crates/pleiades-time/README.md` and the workspace `README.md` civil-time
  row: mention the inverse.
- `spec/api-and-ergonomics.md` does not enumerate the civil-time surface and
  needs no change.
- `docs/follow-ups.md`: a resolved FU entry pointing at this spec, plus any
  deferred item from review.
- Changelog entries come from commit messages via release-plz.

## Consumer-visible changes

- **New:** `from_terrestrial`, four conveniences, `CivilConversion`.
- **Changed (fix):** a UTC `:60` input at a real leap second now converts to a
  TT one second earlier than before.
- **Changed (stricter):** `:60` on a day without an inserted leap second, at a
  time other than `23:59`, or with UT1 source is now
  `InvalidCivilDate { field: "second" }` instead of being read as the next
  minute. The commit carries a `fix!`/breaking marker so release-plz bumps the
  `pleiades-time` minor version; `pleiades-core`, which re-exports the types,
  follows per the per-crate versioning rules.

## Out of scope

- A UTC- or UT1-tagged `Instant` result.
- Sub-millisecond precision, or a (day, seconds-of-day) internal
  representation.
- Negative leap seconds (guarded by the table-shape test, not modelled).
- Removing the 0.216 s `ΔT` step at the 2020 node.
- Changing `ut1_jd_from_tt` or the event finders' return types; a
  civil-time convenience on `pleiades-events` results can follow separately.
- CLI output of civil times for event results.

## Risks

- **Stricter forward contract.** A caller passing `:60` loosely now gets an
  error. Mitigated by the breaking marker and a changelog note; the previous
  result was wrong by a second in the only case where `:60` is meaningful.
- **Julian-day resolution at the leap boundary.** The input Julian day
  resolves about 40 µs. Quantizing to the millisecond first means an instant
  that is millisecond-aligned up to that noise lands on its exact
  millisecond, so boundary samples (`23:59:60.000`, `00:00:00.000`) are
  deterministic and are tested directly.
- **Chart request path.** `pleiades-core` `chart/request.rs` calls the forward
  conversion; its tests are re-run to confirm no caller relied on the `:60`
  aliasing.
