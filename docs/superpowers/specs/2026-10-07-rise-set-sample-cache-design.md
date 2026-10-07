# Rise/set sample cache and ITP refinement (issue #204)

## Goal

Bring a daily sunrise bracket (previous Rise, next Set, next Rise for the Sun)
close to Swiss Ephemeris's 0.34 ms. That figure comes from the reporter's
machine and includes Python overhead. #209 cut the bracket from 5.80 ms to
0.78 ms on our machine and from 151.7 to 15.6 backend reads. What remains, per
#209's measurements:

- about 15.6 backend reads at about 32 µs each (about 0.5 ms);
- about 75 residual evaluations at about 3 µs each (about 0.23 ms). Each
  search walks an hourly grid out to the event, then bisects about 13 times
  from one hour down to 0.5 s.

## Consumer context

From starfold's reply, 2026-10-07:

- Today starfold builds a new `EventEngine` for every sunrise bracket. It will
  switch its 90-day muhurta sweep to one engine, which it already does for
  longitude work.
- No engine is shared across threads. Native builds are called synchronously
  from the JS thread. The web build is `wasm32-unknown-unknown`,
  single-threaded, with no atomics.
- Its preference is a cache inside `EventEngine`. A `Mutex` is acceptable,
  since `std::sync::Mutex` works on that target.

## Part 1: engine-level sample cache

### Design

- `EventEngine<B>` gains a private field `places: PlaceCache`
  (`rise_trans/track.rs`). `EventEngine::new` keeps its signature. The engine
  stays `Send + Sync` whenever `B` is.
- `PlaceCache` wraps `Mutex<HashMap<(CelestialBody, i64), Option<Place>>>`,
  keyed by body and lattice index. Each body's lattice step is fixed by
  `lattice_step_days(body)`, so the index alone determines the instant.
  `None` records a lattice instant the window does not reach: outside
  `WINDOW_START_JD..=WINDOW_END_JD`, or `OutOfWindow` from the read. That is
  exactly what `BodyTrack::sample` stores today.
- `BodyTrack` no longer owns its per-search `RefCell<Vec<…>>`. It holds
  `&PlaceCache` and looks samples up there. On a miss it drops the lock, reads
  the backend, then inserts. Two concurrent misses on the same key both read
  and insert the same value, which is harmless. The lock is never held across
  a backend call.
- Errors other than `OutOfWindow` propagate and are not cached.
- A poisoned lock is recovered with `PoisonError::into_inner`. Every entry is
  written by a single `insert` of a complete value, so the table is
  consistent at any panic point.
- Bound: `PLACE_CACHE_CAPACITY = 8192` entries, about 11 years of Sun at 12 h
  or about five and a half years of Moon at 6 h, and well under 1 MB. When an insert would
  exceed it, the table is cleared first. Clearing affects speed only.

### Why hidden state is acceptable here

A sample sits at an absolute lattice instant (#209). It is independent of
observer, atmosphere and options, and it comes from the engine's own backend.
The cache is therefore a pure memo: every search result is bit-identical
whatever the cache holds. The `EventEngine` rustdoc gets a "Reuse one engine
across a sweep" paragraph explaining that the cache only ever improves speed.

### Scope

Only `BodyTrack` users read the cache: rise/set and transit searches.
Crossings, stations, aspects, occultations and the rest are untouched.

## Part 2: ITP refinement for the rise/set scanner

### Design

- Add `root::refine_itp(f, lo, f_lo, hi, f_hi)`, the ITP method (Oliveira &
  Takahashi, 2020, ACM TOMS 47(1)), with `κ₁ = 0.2 / (hi − lo)`, `κ₂ = 2` and
  `n₀ = 1`. Its worst case is at most `⌈log₂((hi−lo)/2ε)⌉ + n₀` evaluations,
  so it is never worse than bisection by more than one evaluation. For a
  smooth residual it converges superlinearly.
- It keeps `bisect`'s contract exactly:
  - it returns the *later* end of the final bracket;
  - that bracket is no wider than `REFINE_TOLERANCE_DAYS`;
  - the bracket invariant (pre-crossing sign at `lo`, post-crossing sign at
    `hi`) holds at every step.

  So a returned instant is still settled, which #80, #81 and #159 rely on.
- `rise_trans::scan::walk` switches from `bisect` to `refine_itp`. It already
  has `later.f` at each bracket. `root::bisect` and every other caller
  (crossings, stations, aspects, eclipse-style scanners) are unchanged, so
  their gates and goldens cannot move.
- The culmination and graze refiners in `scan.rs` (golden-section search)
  are out of scope.

### Accepted consequence

Rise, set and transit instants move by less than `REFINE_TOLERANCE_DAYS`
(0.5 s). `validate-rise-trans` figures may change in their last printed digits,
but its ceilings do not. The PR reports the before and after figures.

### Commit order

Part 1 lands as its own commit before Part 2, so each can be measured and
reverted on its own. The bit-identity test (below) is run on the Part 1
commit.

## Testing

- **Bit identity (Part 1).** Brackets for 60 consecutive days at Chennai,
  computed on one reused engine and on a fresh engine per day, are equal bit
  for bit (`f64::to_bits` on every instant).
- **Read counts** (`CountingBackend`, in `rise_trans/cost_tests.rs`):
  - a cold bracket on one engine: at most 6 Sun reads (15.6 today);
  - a 30-day sweep on one engine: at most 3 Sun reads per day on average;
  - the existing per-search and moonrise bounds stay.
- **Cache unit tests** (`rise_trans/track/tests.rs`):
  - clears at capacity and keeps working;
  - stores and returns `None` samples without re-reading;
  - does not cache a non-`OutOfWindow` error, so the next call reads again;
  - survives a poisoned lock;
  - keys differ per body.
- **ITP unit tests** (`root/refine_tests.rs`):
  - returns the later end of a bracket within tolerance whose ends carry the
    pre- and post-crossing signs;
  - on step-like and badly scaled functions, uses at most bisection's count
    plus `n₀`;
  - on linear and smooth (sine) functions, uses well under bisection's count;
  - works with a root at either end.
- **Unchanged suites:** rise/set chaining, window-edge, scanner and cost tests,
  and `validate-rise-trans`, all with ceilings unchanged.
- **Timing.** The issue's fixture (Sun at Chennai, 366 days of 2025,
  `CompositeBackend<ElpBackend, Vsop87Backend>`, release). Baseline and each
  stage run alternately, best of seven, machine load recorded:
  - fresh engine per day;
  - one reused engine;
  - one reused engine plus ITP.

  The figures go in the PR description and in a comment on #204, which states
  whether the issue can close.

## Documentation

- `EventEngine` rustdoc: the reuse paragraph.
- Module docs of `rise_trans/track.rs` (the cache) and `rise_trans/scan.rs`
  (ITP replacing bisection there).
- Docs that state "bisection to 0.5 s" for rise/set are corrected where they
  describe this scanner.

## Out of scope

- A cache for crossings, stations or aspects searches.
- Changing the hourly grid step or the lattice steps.
- A batch "daily bracket" API.
