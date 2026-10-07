# Rise/set Sample Cache and ITP Refinement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cut the cost of a daily sunrise bracket (issue #204) in two ways. First, the rise/set and transit searches of one `EventEngine` share the body samples they read. Second, the rise/set scanner refines its brackets with ITP instead of bisection.

**Architecture:** A bounded `PlaceCache` (a `Mutex`-guarded map from body and lattice index to place) becomes a private field of `EventEngine`. `BodyTrack` reads through it instead of owning a per-search vector. Because samples sit at absolute lattice instants, the cache is a pure memo and results are bit-identical. A new `root::refine_itp` keeps `root::bisect`'s "settled later end, bracket ≤ 0.5 s" contract. Only `rise_trans::scan::walk` switches to it.

**Tech Stack:** Rust (stable, per `mise.toml`), `std::sync::Mutex`, `std::collections::HashMap`. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-07-rise-set-sample-cache-design.md`

## Global Constraints

- No new crate dependencies; pure Rust; `pleiades-events` only (plus a scratch timing probe that is never committed).
- `EventEngine::new(backend)` keeps its signature; `EventEngine<B>` stays `Send + Sync` whenever `B` is.
- `PLACE_CACHE_CAPACITY = 8192` entries; reaching it clears the whole table.
- The cache lock is never held across a backend read. Errors other than `EventError::OutOfWindow` propagate and are not cached. A poisoned lock is recovered with `PoisonError::into_inner`.
- ITP parameters: `κ₁ = 0.2 / (hi − lo)` of the initial bracket, `κ₂ = 2`, `n₀ = 1`, `2ε = REFINE_TOLERANCE_DAYS` (0.5 s).
- `refine_itp` returns the later end of a final bracket no wider than `REFINE_TOLERANCE_DAYS`, using `bisect`'s sign rule: an evaluation `<= 0.0` moves the end whose value is `<= 0.0`.
- `root::bisect` and every caller other than `rise_trans::scan::walk` stay unchanged (crossings, stations, aspects, eclipses).
- Part 1 (Tasks 1–2) and Part 2 (Tasks 3–4) are separate commits, Part 1 first.
- Public rustdoc must not intra-doc-link to a private item: `mise run docs` runs rustdoc with `-D warnings`.
- Run `cargo fmt --all` before every commit; CI's fmt gate is blocking.
- Do not edit source files or commit while a background test run is in progress.

## Review Focus

- **One engine used from several threads at once.** It must return the same instants as a fresh engine per call: no torn entries, no deadlock (Task 2, `a_shared_engine_answers_the_same_from_several_threads`).
- **The cache clearing in the middle of a search.** A tiny capacity must give bit-identical brackets (Task 2, `a_tiny_cache_gives_the_same_brackets`).
- **Different bodies on one engine.** Interleaved Sun and Moon searches must not return each other's samples (Task 2, `interleaved_sun_and_moon_searches_match_fresh_engines`).
- **Repeated searches near the window's end on a reused engine.** The cached `None` samples must give the same `Ok`/`OutOfWindow` answer as a fresh engine (Task 2, `a_search_at_the_windows_end_answers_the_same_on_a_reused_engine`).
- **ITP on step-like or badly scaled residuals.** Graze brackets are like this. It must never take more evaluations than bisection plus one (Task 3, `step_and_cubic_residuals_cost_at_most_one_more_than_bisection`).

---

## File Structure

| File | Change | Responsibility |
|---|---|---|
| `crates/pleiades-events/src/rise_trans/track.rs` | modify | `PlaceCache` (new) and `BodyTrack` reading through it |
| `crates/pleiades-events/src/rise_trans/track/tests.rs` | modify | `BodyTrack` and `PlaceCache` unit tests |
| `crates/pleiades-events/src/rise_trans/mod.rs` | modify | re-export `PlaceCache` to the crate; `track()` passes the engine's cache; doc wording |
| `crates/pleiades-events/src/crossings.rs` | modify | `EventEngine` gains the `places` field and the reuse rustdoc |
| `crates/pleiades-events/src/rise_trans/test_support.rs` | modify | `chennai()`, `tt()`, `sun_bracket()`, `FailingFirstReads` helpers |
| `crates/pleiades-events/src/rise_trans/cost_tests.rs` | modify | bracket and sweep read-count tests; use shared helpers |
| `crates/pleiades-events/src/rise_trans/cache_tests.rs` | create | engine-level cache behaviour (bit identity, threads, capacity, bodies, window end) |
| `crates/pleiades-events/src/root.rs` | modify | `refine_itp`; declare `refine_tests` |
| `crates/pleiades-events/src/root/refine_tests.rs` | create | ITP unit tests |
| `crates/pleiades-events/src/rise_trans/scan.rs` | modify | `walk` calls `refine_itp`; module docs |
| `$SCRATCH/probe204/` (scratchpad, never committed) | create | timing probe |

`$SCRATCH` below means `/tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad`. `$WT` means the worktree root, `/workspace/.claude/worktrees/issue-204-event-engine-sample-cache`.

---

### Task 0: Baseline timing probe and gate figures (no commit)

**Files:**
- Create: `$SCRATCH/probe204/Cargo.toml`
- Create: `$SCRATCH/probe204/src/main.rs`
- Create: `$SCRATCH/probe204/run.sh`

**Interfaces:**
- Produces: `$SCRATCH/bin/bracket-baseline` (binary) and `$SCRATCH/gate-baseline.txt`. Tasks 2 and 4 rebuild the same probe into `bracket-cache` and `bracket-itp`.

- [ ] **Step 1: Write the probe manifest**

`$SCRATCH/probe204/Cargo.toml`:

```toml
[package]
name = "probe204"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
pleiades-events = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-events" }
pleiades-backend = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-backend" }
pleiades-apparent = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-apparent" }
pleiades-elp = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-elp" }
pleiades-vsop87 = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-vsop87" }
pleiades-types = { path = "/workspace/.claude/worktrees/issue-204-event-engine-sample-cache/crates/pleiades-types" }

[workspace]

[profile.release]
debug = false
```

- [ ] **Step 2: Write the probe**

`$SCRATCH/probe204/src/main.rs`:

```rust
//! Issue #204's fixture: the Sun at Chennai, 366 days of 2025 at 06:00 TT,
//! one bracket (previous Rise, next Set, next Rise) per day.
use std::time::Instant as Clock;

use pleiades_apparent::Atmosphere;
use pleiades_backend::CompositeBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{EventEngine, RiseSetEvent, RiseSetOptions, RiseSetTarget};
use pleiades_types::{
    CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale,
};
use pleiades_vsop87::Vsop87Backend;

type Composite = CompositeBackend<ElpBackend, Vsop87Backend>;

/// 2025-01-01 06:00 TT.
const START_JD: f64 = 2_460_676.75;

fn engine() -> EventEngine<Composite> {
    EventEngine::new(CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new()))
}

fn chennai() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(13.08),
        Longitude::from_degrees(80.27),
        Some(0.0),
    )
}

fn sun() -> RiseSetTarget {
    RiseSetTarget::Body(CelestialBody::Sun)
}

/// Sum of the bracket's three Julian days, so the work cannot be optimized
/// away and stages can be compared for equal answers.
fn bracket(engine: &EventEngine<Composite>, jd: f64) -> f64 {
    let at = Instant::new(JulianDay::from_days(jd), TimeScale::Tt);
    let atmos = Atmosphere::default();
    let opts = RiseSetOptions::default;
    let rise = engine
        .previous_rise_set(sun(), RiseSetEvent::Rise, chennai(), atmos, opts(), at)
        .unwrap()
        .unwrap()
        .instant;
    let set = engine
        .next_rise_set(sun(), RiseSetEvent::Set, chennai(), atmos, opts(), rise)
        .unwrap()
        .unwrap()
        .instant;
    let next = engine
        .next_rise_set(sun(), RiseSetEvent::Rise, chennai(), atmos, opts(), set)
        .unwrap()
        .unwrap()
        .instant;
    rise.julian_day.days() + set.julian_day.days() + next.julian_day.days()
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "fresh".to_owned());
    let mut checksum = 0.0;
    let clock = Clock::now();
    match mode.as_str() {
        "fresh" => {
            for day in 0..366 {
                checksum += bracket(&engine(), START_JD + f64::from(day));
            }
        }
        "reused" => {
            let engine = engine();
            for day in 0..366 {
                checksum += bracket(&engine, START_JD + f64::from(day));
            }
        }
        other => panic!("unknown mode {other}; use fresh or reused"),
    }
    let per_bracket_ms = clock.elapsed().as_secs_f64() * 1000.0 / 366.0;
    println!("{mode}: {per_bracket_ms:.3} ms per bracket (checksum {checksum:.9})");
}
```

- [ ] **Step 3: Build the baseline binary and keep a copy**

Run:

```bash
cargo build --release --offline --manifest-path /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204/Cargo.toml --target-dir /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target
mkdir -p /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin
cp /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target/release/probe204 /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin/bracket-baseline
```

Expected: build succeeds. Then run `$SCRATCH/bin/bracket-baseline fresh` once. Expected: a line like `fresh: 0.8xx ms per bracket (checksum …)`. Write the checksum down; Task 2's binaries must print the same checksum.

- [ ] **Step 4: Write the alternating-run script**

`$SCRATCH/probe204/run.sh`. It takes the binaries as arguments and runs each in both modes, seven rounds, alternating:

```bash
#!/usr/bin/env bash
set -euo pipefail
echo "load before: $(cat /proc/loadavg)"
for round in 1 2 3 4 5 6 7; do
  for bin in "$@"; do
    for mode in fresh reused; do
      echo "round $round $(basename "$bin") $("$bin" "$mode")"
    done
  done
done
echo "load after: $(cat /proc/loadavg)"
```

Run it as `bash $SCRATCH/probe204/run.sh <bins…>`. It is a script file because the worktree session refuses inline shell loops.

- [ ] **Step 5: Record the baseline gate figures**

Run: `cd $WT && cargo run --release -q -p pleiades-validate -- validate-rise-trans > $SCRATCH/gate-baseline.txt 2>&1; tail -30 $SCRATCH/gate-baseline.txt`
Expected: the gate passes and prints its Tier 1 and Tier 2 figures (at #209 they were 1.812″, 3.259 s rise/set, 111.209 s grazing, 0.383 s transit). If the subcommand name is rejected, use `cargo run --release -q -p pleiades-validate -- help` to find the rise-trans gate alias and use that.

---

### Task 1: `PlaceCache`, and `BodyTrack` reading through it

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans/track.rs`
- Modify: `crates/pleiades-events/src/rise_trans/track/tests.rs`
- Modify: `crates/pleiades-events/src/rise_trans/test_support.rs`
- Modify: `crates/pleiades-events/src/rise_trans/mod.rs:332-337` (the `track()` helper), plus a `pub(crate) use`
- Modify: `crates/pleiades-events/src/crossings.rs:52-63` (`EventEngine` struct and `new`)

**Interfaces:**
- Produces, in `rise_trans::track` (re-exported as `crate::rise_trans::PlaceCache`):
  - `pub(crate) struct PlaceCache`, implementing `Default`
  - `pub(crate) fn PlaceCache::new() -> PlaceCache` (capacity `PLACE_CACHE_CAPACITY`)
  - `pub(crate) fn PlaceCache::with_capacity(capacity: usize) -> PlaceCache`
  - `pub(crate) fn PlaceCache::get(&self, body: &CelestialBody, index: i64) -> Option<Option<Place>>`
  - `pub(crate) fn PlaceCache::insert(&self, body: &CelestialBody, index: i64, place: Option<Place>)`
  - `#[cfg(test)] pub(crate) fn PlaceCache::len(&self) -> usize`
  - `pub(crate) fn BodyTrack::new(backend: &'a B, cache: &'a PlaceCache, body: &CelestialBody) -> Option<BodyTrack<'a, B>>`
- Produces, in `crossings.rs`:
  - `EventEngine { pub(crate) backend: B, pub(crate) places: PlaceCache }`
  - `#[cfg(test)] pub(crate) fn EventEngine::with_place_cache_capacity(backend: B, capacity: usize) -> Self`
- Produces, in `rise_trans::test_support`: `pub(crate) struct FailingFirstReads<B>` with `pub(crate) fn new(inner: B, failures: usize) -> Self`

- [ ] **Step 1: Add the failing-backend helper to `test_support.rs`**

Append to `crates/pleiades-events/src/rise_trans/test_support.rs`:

```rust
/// A backend whose first `failures` reads fail with a numerical error, which
/// the engine reports as `EventError::Backend`, not as a window error.
pub(crate) struct FailingFirstReads<B> {
    inner: B,
    failures: AtomicUsize,
}

impl<B> FailingFirstReads<B> {
    pub(crate) fn new(inner: B, failures: usize) -> Self {
        Self {
            inner,
            failures: AtomicUsize::new(failures),
        }
    }

    fn fail_now(&self) -> bool {
        self.failures
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |left| left.checked_sub(1))
            .is_ok()
    }
}

impl<B: EphemerisBackend> EphemerisBackend for FailingFirstReads<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if self.fail_now() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "injected failure",
            ));
        }
        self.inner.position(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        if self.fail_now() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::NumericalFailure,
                "injected failure",
            ));
        }
        self.inner.position_without_motion(req)
    }
}
```

Add `EphemerisErrorKind` to that file's `pleiades_backend::{…}` import. If `EphemerisError::new` is not public with this signature (`crates/pleiades-backend/src/errors.rs:64`), use whatever public constructor that file provides.

- [ ] **Step 2: Write the failing `PlaceCache` and track tests**

In `crates/pleiades-events/src/rise_trans/track/tests.rs`:

(a) Change the import line to:

```rust
use crate::rise_trans::test_support::{composite, CountingBackend, FailingFirstReads};
```

(b) Every existing `BodyTrack::new(&backend, &body)` (and `&CelestialBody::Sun`, `&CelestialBody::Mars`, and so on) becomes `BodyTrack::new(&backend, &cache, &body)`, with `let cache = PlaceCache::new();` declared right after `let backend = …;` in each test. In `a_track_reproduces_a_direct_read_for_every_tracked_body`, declare the cache inside the `for body in TRACKED` loop, before the inner loop.

(c) Append these tests:

```rust
#[test]
fn a_cache_starts_over_when_it_is_full() {
    let cache = PlaceCache::with_capacity(2);
    let place = Some((1.0, 2.0, 3.0));
    cache.insert(&CelestialBody::Sun, 1, place);
    cache.insert(&CelestialBody::Sun, 2, place);
    assert_eq!(cache.len(), 2);
    cache.insert(&CelestialBody::Sun, 3, place);
    assert_eq!(cache.len(), 1, "the full table is cleared before the insert");
    assert_eq!(cache.get(&CelestialBody::Sun, 1), None);
    assert_eq!(cache.get(&CelestialBody::Sun, 3), Some(place));
}

#[test]
fn a_cache_keeps_bodies_apart() {
    let cache = PlaceCache::new();
    cache.insert(&CelestialBody::Sun, 7, Some((1.0, 0.0, 1.0)));
    assert_eq!(cache.get(&CelestialBody::Moon, 7), None);
    assert_eq!(
        cache.get(&CelestialBody::Sun, 7),
        Some(Some((1.0, 0.0, 1.0)))
    );
}

#[test]
fn a_cache_reinserting_a_key_does_not_grow_it() {
    let cache = PlaceCache::with_capacity(2);
    cache.insert(&CelestialBody::Sun, 1, None);
    cache.insert(&CelestialBody::Sun, 1, None);
    cache.insert(&CelestialBody::Sun, 2, None);
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(&CelestialBody::Sun, 1), Some(None));
}

#[test]
fn a_cache_survives_a_poisoned_lock() {
    let cache = PlaceCache::new();
    cache.insert(&CelestialBody::Sun, 1, Some((1.0, 2.0, 3.0)));
    let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = cache.lock();
        panic!("poison the lock");
    }));
    assert!(poisoned.is_err());
    assert_eq!(
        cache.get(&CelestialBody::Sun, 1),
        Some(Some((1.0, 2.0, 3.0)))
    );
    cache.insert(&CelestialBody::Sun, 2, None);
    assert_eq!(cache.get(&CelestialBody::Sun, 2), Some(None));
}

#[test]
fn tracks_sharing_a_cache_read_each_sample_once() {
    let backend = CountingBackend::new(composite());
    let cache = PlaceCache::new();
    let first = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    first.place(2_460_000.01).unwrap();
    assert_eq!(backend.take_reads(), 4);
    // A second track over the same cache, as the next search of the same
    // engine makes, reads nothing for the same interval.
    let second = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    assert_eq!(
        second.place(2_460_000.01).unwrap(),
        first.place(2_460_000.01).unwrap()
    );
    assert_eq!(backend.take_reads(), 0);
}

#[test]
fn a_sample_the_window_does_not_reach_is_remembered() {
    // Mars at the window's first lattice instant reads before the window
    // (light-time), so that sample is `None` and the place at an instant
    // next to it is read directly. Asked again, the `None` is not re-read:
    // only the direct read is repeated.
    let backend = CountingBackend::new(composite());
    let cache = PlaceCache::new();
    let jd = WINDOW_START_JD + 0.6;
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Mars).expect("tracked");
    let first = track.place(jd).unwrap();
    backend.take_reads();
    geocentric_apparent_ecliptic(&backend, CelestialBody::Mars, "body", jd).unwrap();
    let direct_reads = backend.take_reads();
    let again = track.place(jd).unwrap();
    assert_eq!(again, first);
    assert_eq!(backend.take_reads(), direct_reads);
}

#[test]
fn a_failed_read_is_not_remembered() {
    let backend = FailingFirstReads::new(composite(), 1);
    let cache = PlaceCache::new();
    let track = BodyTrack::new(&backend, &cache, &CelestialBody::Sun).expect("tracked");
    let jd = 2_460_000.2;
    assert!(matches!(track.place(jd), Err(EventError::Backend(_))));
    let fresh_backend = composite();
    let fresh_cache = PlaceCache::new();
    let fresh = BodyTrack::new(&fresh_backend, &fresh_cache, &CelestialBody::Sun).expect("tracked");
    assert_eq!(track.place(jd).unwrap(), fresh.place(jd).unwrap());
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cd $WT && cargo test -p pleiades-events --lib rise_trans::track 2>&1 | tail -20`
Expected: a compile error such as `cannot find type PlaceCache` / `this function takes 2 arguments but 3 were supplied`.

- [ ] **Step 4: Implement `PlaceCache` and rewire `BodyTrack`**

In `crates/pleiades-events/src/rise_trans/track.rs`:

(a) Replace `use std::cell::RefCell;` with:

```rust
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
```

(b) Append this paragraph to the module doc comment (after the paragraph ending "(issues #80, #81)."):

```rust
//!
//! The samples live in the engine's [`PlaceCache`], not in the track, so
//! every rise/set and transit search one engine makes shares them: a daily
//! bracket of three searches, or a sweep of brackets over consecutive days,
//! reads each lattice sample once. A sample depends on the body and the
//! lattice instant alone, never on the observer, the atmosphere or the
//! options, so the cache is a pure memo: what it holds changes how many
//! reads a search makes, never what the search returns.
```

(c) Insert after `fn lattice_step_days`:

```rust
/// Most samples a [`PlaceCache`] holds before it starts over: about eleven
/// years of the Sun on its 12-hour lattice, or two of the Moon on its 6-hour
/// one, in well under a megabyte.
const PLACE_CACHE_CAPACITY: usize = 8192;

/// The lattice samples an engine has read, shared by all of its rise/set
/// and transit searches (issue #204).
///
/// Keyed by body and lattice index. `None` records a sample the window does
/// not reach, as [`BodyTrack`] defines it. Only those two outcomes are kept;
/// a backend error is returned to the caller and read again next time.
///
/// When an insert would exceed the capacity, the whole table is cleared
/// first. Clearing changes only how many reads later searches make.
///
/// The lock is held only to look up or insert, never across a backend
/// read. Two searches that miss the same sample at once both read it and
/// insert the same value. Each insert writes one complete entry, so a panic
/// can never leave the table inconsistent, and a poisoned lock is simply
/// taken over.
pub(crate) struct PlaceCache {
    capacity: usize,
    entries: Mutex<Entries>,
}

#[derive(Default)]
struct Entries {
    len: usize,
    by_body: HashMap<CelestialBody, HashMap<i64, Option<Place>>>,
}

impl Default for PlaceCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceCache {
    pub(crate) fn new() -> Self {
        Self::with_capacity(PLACE_CACHE_CAPACITY)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Mutex::new(Entries::default()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Entries> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The remembered sample, or `None` when it has not been read.
    pub(crate) fn get(&self, body: &CelestialBody, index: i64) -> Option<Option<Place>> {
        self.lock().by_body.get(body)?.get(&index).copied()
    }

    pub(crate) fn insert(&self, body: &CelestialBody, index: i64, place: Option<Place>) {
        let mut guard = self.lock();
        let entries = &mut *guard;
        if entries
            .by_body
            .get(body)
            .is_some_and(|samples| samples.contains_key(&index))
        {
            return;
        }
        if entries.len >= self.capacity {
            entries.by_body.clear();
            entries.len = 0;
        }
        entries
            .by_body
            .entry(body.clone())
            .or_default()
            .insert(index, place);
        entries.len += 1;
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.lock().len
    }
}
```

(d) Replace the `BodyTrack` struct, its `new` and its `sample` with the following (`place`, `read` and `interpolate` stay as they are):

```rust
/// One body's apparent place for the length of one search, read through
/// the engine's [`PlaceCache`].
pub(crate) struct BodyTrack<'a, B> {
    backend: &'a B,
    cache: &'a PlaceCache,
    body: CelestialBody,
    step_days: f64,
}

impl<'a, B: EphemerisBackend> BodyTrack<'a, B> {
    /// A track for `body`, or `None` for a body that is not interpolated.
    pub(crate) fn new(backend: &'a B, cache: &'a PlaceCache, body: &CelestialBody) -> Option<Self> {
        lattice_step_days(body).map(|step_days| Self {
            backend,
            cache,
            body: body.clone(),
            step_days,
        })
    }
```

```rust
    /// The lattice sample at `index`, read on first use by any search of the
    /// engine. `None` where the window does not reach: the lattice instant
    /// lies outside it, or the read there reports `OutOfWindow` (a planet
    /// within light-time of the window's start).
    fn sample(&self, index: i64) -> Result<Option<Place>, EventError> {
        if let Some(place) = self.cache.get(&self.body, index) {
            return Ok(place);
        }
        let jd = index as f64 * self.step_days;
        let place = if (WINDOW_START_JD..=WINDOW_END_JD).contains(&jd) {
            match self.read(jd) {
                Ok(place) => Some(place),
                Err(EventError::OutOfWindow { .. }) => None,
                Err(error) => return Err(error),
            }
        } else {
            None
        };
        self.cache.insert(&self.body, index, place);
        Ok(place)
    }
```

(e) In `crates/pleiades-events/src/rise_trans/mod.rs`, add next to `use track::BodyTrack;` (line 22):

```rust
pub(crate) use track::PlaceCache;
```

and change `fn track` (line 332) to:

```rust
    fn track(&self, target: &RiseSetTarget) -> Option<BodyTrack<'_, B>> {
        match target {
            RiseSetTarget::Body(body) => BodyTrack::new(&self.backend, &self.places, body),
            _ => None,
        }
    }
```

(f) In `crates/pleiades-events/src/crossings.rs`, change the struct and `new` to:

```rust
pub struct EventEngine<B> {
    pub(crate) backend: B,
    /// Body samples shared by the rise/set and transit searches.
    pub(crate) places: PlaceCache,
}

impl<B: EphemerisBackend> EventEngine<B> {
    /// Wraps a backend.
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            places: PlaceCache::new(),
        }
    }

    /// An engine whose sample cache holds at most `capacity` entries.
    #[cfg(test)]
    pub(crate) fn with_place_cache_capacity(backend: B, capacity: usize) -> Self {
        Self {
            backend,
            places: PlaceCache::with_capacity(capacity),
        }
    }
```

Add `use crate::rise_trans::PlaceCache;` to `crossings.rs`'s imports. If `new` sits in a separate `impl` block from the one shown, put `with_place_cache_capacity` in the same block as `new`.

- [ ] **Step 5: Run the track tests**

Run: `cd $WT && cargo test -p pleiades-events --lib rise_trans::track 2>&1 | tail -20`
Expected: every test in `rise_trans::track::tests` passes, including the seven new ones.

- [ ] **Step 6: Run the whole crate's lib tests**

Run: `cd $WT && cargo test -p pleiades-events --lib 2>&1 | grep -E "test result|FAILED|panicked"`
Expected: `test result: ok`, with no failures. The cache must not change any result.

- [ ] **Step 7: Format, lint, commit**

```bash
cd $WT && cargo fmt --all && cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings
git add crates/pleiades-events/src
git commit -m "perf(events): share a rise/set search's body samples across the engine's searches (#204)"
```

---

### Task 2: Engine-level cache behaviour, read counts and rustdoc

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans/test_support.rs`
- Modify: `crates/pleiades-events/src/rise_trans/cost_tests.rs`
- Create: `crates/pleiades-events/src/rise_trans/cache_tests.rs`
- Modify: `crates/pleiades-events/src/rise_trans/mod.rs` (declare `cache_tests`)
- Modify: `crates/pleiades-events/src/crossings.rs` (`EventEngine` rustdoc)

**Interfaces:**
- Consumes: `EventEngine::with_place_cache_capacity(backend, capacity)` and `PlaceCache` (Task 1).
- Produces, in `rise_trans::test_support`:
  - `pub(crate) fn chennai() -> ObserverLocation`
  - `pub(crate) fn tt(jd: f64) -> Instant`
  - `pub(crate) const BRACKET_QUERY_JD: f64 = 2_460_827.75`
  - `pub(crate) fn sun_bracket<B: EphemerisBackend>(engine: &EventEngine<B>, at_jd: f64) -> [u64; 3]` (the bits of the three instants' Julian days)

- [ ] **Step 1: Add the shared helpers to `test_support.rs`**

Append:

```rust
use crate::crossings::EventEngine;
use crate::rise_trans::{RiseSetEvent, RiseSetOptions, RiseSetTarget};
use pleiades_apparent::Atmosphere;
use pleiades_types::{Instant, JulianDay, Latitude, Longitude, ObserverLocation, TimeScale};

/// 2025-06-01 06:00 TT, the cost tests' query instant.
pub(crate) const BRACKET_QUERY_JD: f64 = 2_460_827.75;

/// Issue #204's observer.
pub(crate) fn chennai() -> ObserverLocation {
    ObserverLocation::new(
        Latitude::from_degrees(13.08),
        Longitude::from_degrees(80.27),
        Some(0.0),
    )
}

pub(crate) fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

/// The daily bracket of issue #204, the sunrise at or before `at_jd`, the
/// sunset after it and the sunrise after that, as the bits of their
/// Julian days, so callers can compare brackets bit for bit.
pub(crate) fn sun_bracket<B: EphemerisBackend>(engine: &EventEngine<B>, at_jd: f64) -> [u64; 3] {
    let sun = || RiseSetTarget::Body(CelestialBody::Sun);
    let atmos = Atmosphere::default();
    let rise = engine
        .previous_rise_set(
            sun(),
            RiseSetEvent::Rise,
            chennai(),
            atmos,
            RiseSetOptions::default(),
            tt(at_jd),
        )
        .expect("engine ok")
        .expect("a sunrise")
        .instant;
    let set = engine
        .next_rise_set(sun(), RiseSetEvent::Set, chennai(), atmos, RiseSetOptions::default(), rise)
        .expect("engine ok")
        .expect("a sunset")
        .instant;
    let next = engine
        .next_rise_set(sun(), RiseSetEvent::Rise, chennai(), atmos, RiseSetOptions::default(), set)
        .expect("engine ok")
        .expect("a sunrise")
        .instant;
    [rise, set, next].map(|instant| instant.julian_day.days().to_bits())
}
```

Merge the new `use` lines into the file's existing imports, at the top of the file.

- [ ] **Step 2: Point `cost_tests.rs` at the shared helpers**

In `crates/pleiades-events/src/rise_trans/cost_tests.rs`:
- Remove its local `QUERY_JD`, `chennai()` and `tt()`.
- Change the import to `use super::test_support::{chennai, composite, sun_bracket, tt, Composite, CountingBackend, BRACKET_QUERY_JD};`.
- Replace each `QUERY_JD` with `BRACKET_QUERY_JD`.

Then append:

```rust
/// One engine's three searches of a daily bracket share their samples: the
/// bracket spans about a day and a quarter, three 12-hour lattice intervals
/// and their stencil, which is six samples. Before the shared cache each
/// search read its own, about 15 in all.
#[test]
fn a_daily_sunrise_bracket_on_one_engine_reads_each_sample_once() {
    let engine = engine();
    sun_bracket(&engine, BRACKET_QUERY_JD);
    let reads = engine.backend.take_reads();
    assert!(
        reads <= MAX_SUN_READS_PER_BRACKET,
        "{reads} reads of the Sun for one bracket"
    );
}

const MAX_SUN_READS_PER_BRACKET: usize = 6;

/// A sweep of daily brackets on one engine, as an electional search makes,
/// reads only the samples each new day adds: two on a 12-hour lattice.
#[test]
fn a_sweep_of_daily_brackets_on_one_engine_reads_about_two_samples_a_day() {
    let engine = engine();
    for day in 0..SWEEP_DAYS {
        sun_bracket(&engine, BRACKET_QUERY_JD + f64::from(day));
    }
    let reads = engine.backend.take_reads();
    assert!(
        reads <= MAX_SUN_READS_PER_SWEPT_DAY * SWEEP_DAYS as usize,
        "{reads} reads of the Sun over {SWEEP_DAYS} days"
    );
}

const SWEEP_DAYS: u32 = 30;
const MAX_SUN_READS_PER_SWEPT_DAY: usize = 3;
```

- [ ] **Step 3: Write `cache_tests.rs`**

Create `crates/pleiades-events/src/rise_trans/cache_tests.rs`:

```rust
//! The engine's shared sample cache (issue #204) changes how many reads a
//! search makes and never what it returns: every answer here is compared
//! bit for bit with one from a fresh engine.

use std::sync::Arc;

use super::test_support::{chennai, composite, sun_bracket, tt, Composite, BRACKET_QUERY_JD};
use super::*;
use crate::error::WINDOW_END_JD;
use pleiades_types::CelestialBody;

fn fresh() -> EventEngine<Composite> {
    EventEngine::new(composite())
}

fn fresh_brackets(days: u32) -> Vec<[u64; 3]> {
    (0..days)
        .map(|day| sun_bracket(&fresh(), BRACKET_QUERY_JD + f64::from(day)))
        .collect()
}

#[test]
fn a_reused_engine_returns_the_brackets_of_fresh_engines() {
    let reused = fresh();
    let got: Vec<_> = (0..60)
        .map(|day| sun_bracket(&reused, BRACKET_QUERY_JD + f64::from(day)))
        .collect();
    assert_eq!(got, fresh_brackets(60));
}

#[test]
fn a_tiny_cache_gives_the_same_brackets() {
    // Three entries cannot even hold one stencil, so the table is cleared
    // over and over inside each search.
    let tiny = EventEngine::with_place_cache_capacity(composite(), 3);
    let got: Vec<_> = (0..10)
        .map(|day| sun_bracket(&tiny, BRACKET_QUERY_JD + f64::from(day)))
        .collect();
    assert_eq!(got, fresh_brackets(10));
}

#[test]
fn a_shared_engine_answers_the_same_from_several_threads() {
    let engine = Arc::new(fresh());
    let want = fresh_brackets(10);
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let engine = Arc::clone(&engine);
            std::thread::spawn(move || {
                (0..10)
                    .map(|day| sun_bracket(&engine, BRACKET_QUERY_JD + f64::from(day)))
                    .collect::<Vec<_>>()
            })
        })
        .collect();
    for handle in handles {
        assert_eq!(handle.join().expect("no panic"), want);
    }
}

fn moonrise_bits(engine: &EventEngine<Composite>, at_jd: f64) -> u64 {
    engine
        .next_rise_set(
            RiseSetTarget::Body(CelestialBody::Moon),
            RiseSetEvent::Rise,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tt(at_jd),
        )
        .expect("engine ok")
        .expect("a moonrise")
        .instant
        .julian_day
        .days()
        .to_bits()
}

#[test]
fn interleaved_sun_and_moon_searches_match_fresh_engines() {
    let shared = fresh();
    for day in 0..10 {
        let jd = BRACKET_QUERY_JD + f64::from(day);
        assert_eq!(sun_bracket(&shared, jd), sun_bracket(&fresh(), jd), "day {day}");
        assert_eq!(moonrise_bits(&shared, jd), moonrise_bits(&fresh(), jd), "day {day}");
    }
}

fn sunrise_near_the_end(engine: &EventEngine<Composite>, at_jd: f64) -> String {
    format!(
        "{:?}",
        engine.next_rise_set(
            RiseSetTarget::Body(CelestialBody::Sun),
            RiseSetEvent::Rise,
            chennai(),
            Atmosphere::default(),
            RiseSetOptions::default(),
            tt(at_jd),
        )
    )
}

#[test]
fn a_search_at_the_windows_end_answers_the_same_on_a_reused_engine() {
    // Searches whose samples run past the window's end leave `None` entries
    // in the cache; asked again on the same engine, each answers as a fresh
    // engine does, whether that is an event or `OutOfWindow`.
    let reused = fresh();
    for offset in [3.0, 1.0, 0.6, 0.3, 0.1] {
        let jd = WINDOW_END_JD - offset;
        let first = sunrise_near_the_end(&reused, jd);
        assert_eq!(sunrise_near_the_end(&reused, jd), first, "offset {offset}");
        assert_eq!(sunrise_near_the_end(&fresh(), jd), first, "offset {offset}");
    }
}

#[test]
fn an_engine_stays_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<EventEngine<Composite>>();
}
```

`Atmosphere` reaches this file through `use super::*;` (mod.rs imports it). If the compiler says otherwise, add `use pleiades_apparent::Atmosphere;`.

Declare the module in `crates/pleiades-events/src/rise_trans/mod.rs` next to `mod cost_tests;` (line 748), using the same attribute the neighbouring test modules use:

```rust
#[cfg(test)]
mod cache_tests;
```

- [ ] **Step 4: Run the new tests**

Run: `cd $WT && cargo test -p pleiades-events --lib rise_trans::cache_tests rise_trans::cost_tests 2>&1 | grep -E "^test |test result"`
Expected: all pass. These tests run against Task 1's implementation, so they confirm it rather than drive it. Verify that the read-count tests guard something:
- Temporarily change `PLACE_CACHE_CAPACITY` to `0`. `PlaceCache::insert` then clears the table on every insert, so the cache holds one sample at a time.
- Run the cost tests again. Expected: `a_daily_sunrise_bracket_on_one_engine_reads_each_sample_once` and the sweep test FAIL with read counts above their bounds.
- Restore `8192` and re-run. Expected: PASS. Confirm the line reads `const PLACE_CACHE_CAPACITY: usize = 8192;` before moving on.

If a bound fails with the real capacity, do not raise the constant silently. Report the measured count. The spec's bounds are 6 per bracket and 3 per swept day.

- [ ] **Step 5: Add the reuse paragraph to `EventEngine`'s rustdoc**

In `crates/pleiades-events/src/crossings.rs`, extend the doc comment above `pub struct EventEngine<B>` to:

```rust
/// Finds ephemeris events (longitude crossings today; rise/set/transit and
/// horizontal coordinates in sibling modules) over the packaged 1900–2100 TDB
/// window.
///
/// # Reuse one engine across a sweep
///
/// An engine remembers the body places its rise, set and transit searches
/// have read, and later searches reuse them: the three searches of a daily
/// sunrise bracket share their samples, and a sweep of daily brackets on one
/// engine reads only the samples each new day adds. Build one engine for a
/// sweep and pass it by reference, rather than one per search.
///
/// What the engine remembers never changes an answer. A remembered place is
/// the one a fresh engine would read at the same instant, and it does not
/// depend on the observer, the atmosphere or the options. The memory is
/// bounded, and the engine can be shared between threads.
```

Do not link to `PlaceCache` (private) from this public doc.

- [ ] **Step 6: Check the rustdoc build**

Run: `cd $WT && mise run docs 2>&1 | tail -5`
Expected: success, with no `-D warnings` failure.

- [ ] **Step 7: Format, lint, run the crate's tests, commit**

```bash
cd $WT && cargo fmt --all && cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings
cargo test -p pleiades-events 2>&1 | grep -E "test result|FAILED|panicked"
git add crates/pleiades-events/src
git commit -m "test(events): hold the shared sample cache to fresh-engine answers and per-bracket read counts; document engine reuse (#204)"
```

Expected: every `test result: ok`.

- [ ] **Step 8: Build the Part 1 probe binary and compare checksums**

```bash
cargo build --release --offline --manifest-path /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204/Cargo.toml --target-dir /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target
cp /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target/release/probe204 /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin/bracket-cache
/tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin/bracket-cache fresh
/tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin/bracket-cache reused
```

Expected: both print the same checksum as `bracket-baseline` (Task 0, Step 3), digit for digit.

---

### Task 3: `root::refine_itp`

**Files:**
- Modify: `crates/pleiades-events/src/root.rs` (add `refine_itp` after `bisect`; declare `refine_tests`)
- Create: `crates/pleiades-events/src/root/refine_tests.rs`

**Interfaces:**
- Produces: `pub(crate) fn refine_itp<F>(f: &mut F, lo: f64, f_lo: f64, hi: f64, f_hi: f64) -> Result<f64, EventError> where F: FnMut(f64) -> Result<f64, EventError>`

- [ ] **Step 1: Write the failing tests**

Create `crates/pleiades-events/src/root/refine_tests.rs`:

```rust
//! `refine_itp` keeps `bisect`'s contract (the settled later end of a
//! bracket no wider than the tolerance) in fewer evaluations on smooth
//! residuals, and never more than one beyond bisection's on any residual.

use super::*;

const TOL: f64 = REFINE_TOLERANCE_DAYS;

/// An hour, the rise/set scanner's grid step and so its widest bracket.
const HOUR: f64 = 1.0 / 24.0;

/// Runs a refiner on `g` over `[lo, hi]` and returns its answer and how many
/// times it evaluated `g`.
fn counted(
    refiner: &dyn Fn(&mut dyn FnMut(f64) -> Result<f64, EventError>, f64, f64) -> Result<f64, EventError>,
    g: &dyn Fn(f64) -> f64,
    lo: f64,
    hi: f64,
) -> (f64, usize) {
    let mut calls = 0;
    let mut f = |t: f64| {
        calls += 1;
        Ok(g(t))
    };
    let root = refiner(&mut f, lo, hi).unwrap();
    (root, calls)
}

// `refine_itp` and `bisect` take a sized `F`; `&mut dyn FnMut` is one.
fn itp(mut f: &mut dyn FnMut(f64) -> Result<f64, EventError>, lo: f64, hi: f64) -> Result<f64, EventError> {
    let (f_lo, f_hi) = (f(lo)?, f(hi)?);
    refine_itp(&mut f, lo, f_lo, hi, f_hi)
}

fn bisection(mut f: &mut dyn FnMut(f64) -> Result<f64, EventError>, lo: f64, hi: f64) -> Result<f64, EventError> {
    let f_lo = f(lo)?;
    f(hi)?;
    bisect(&mut f, lo, f_lo, hi)
}

/// The answer is settled: `g` carries the post-crossing sign there and the
/// pre-crossing sign one tolerance earlier.
fn assert_settled(g: &dyn Fn(f64) -> f64, ascending: bool, got: f64) {
    let (before, at) = (g(got - TOL), g(got));
    if ascending {
        assert!(before <= 0.0 && at > 0.0, "not settled at {got}: {before}, {at}");
    } else {
        assert!(before > 0.0 && at <= 0.0, "not settled at {got}: {before}, {at}");
    }
}

const T0: f64 = 2_460_827.0;

#[test]
fn a_linear_residual_settles_in_a_few_evaluations() {
    let root = T0 + 0.37 * HOUR;
    let g = move |t: f64| t - root;
    let (got, calls) = counted(&itp, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
    assert!(calls - 2 <= 4, "{} evaluations past the ends", calls - 2);
}

#[test]
fn a_sinusoidal_residual_settles_well_under_bisection() {
    // An altitude-like residual: a day-period sine with its zero inside an
    // hour bracket, rising.
    let root = T0 + 0.61 * HOUR;
    let g = move |t: f64| (std::f64::consts::TAU * (t - root)).sin();
    let (got, calls) = counted(&itp, &g, T0, T0 + HOUR);
    let (_, bisect_calls) = counted(&bisection, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
    assert!(calls - 2 <= 6, "{} evaluations past the ends", calls - 2);
    assert!(calls < bisect_calls, "{calls} against bisection's {bisect_calls}");
}

#[test]
fn a_descending_residual_settles_on_its_post_crossing_side() {
    let root = T0 + 0.2 * HOUR;
    let g = move |t: f64| root - t;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert_settled(&g, false, got);
}

#[test]
fn step_and_cubic_residuals_cost_at_most_one_more_than_bisection() {
    let root = T0 + 0.731 * HOUR;
    let step = move |t: f64| if t < root { -1.0 } else { 1.0 };
    let cubic = move |t: f64| ((t - root) * 24.0).powi(3) * 1e6;
    let flat_then_steep = move |t: f64| if t < root { -1e-12 } else { 1e3 * (t - root) + 1e-12 };
    for (label, g) in [
        ("step", &step as &dyn Fn(f64) -> f64),
        ("cubic", &cubic),
        ("flat then steep", &flat_then_steep),
    ] {
        let (got, calls) = counted(&itp, g, T0, T0 + HOUR);
        let (_, bisect_calls) = counted(&bisection, g, T0, T0 + HOUR);
        assert_settled(g, true, got);
        assert!(
            calls <= bisect_calls + 1,
            "{label}: {calls} against bisection's {bisect_calls}"
        );
    }
}

#[test]
fn a_root_at_the_earlier_end_settles_within_a_tolerance_of_it() {
    // `f(lo) == 0` counts as the pre-crossing sign, as in `bisect`.
    let g = |t: f64| t - T0;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert!(got > T0 && got - T0 <= TOL, "{}", got - T0);
}

#[test]
fn a_root_just_before_the_later_end_settles_within_a_tolerance_of_it() {
    let root = T0 + HOUR - 1e-9;
    let g = move |t: f64| t - root;
    let (got, _) = counted(&itp, &g, T0, T0 + HOUR);
    assert_settled(&g, true, got);
}

#[test]
fn a_bracket_already_within_tolerance_is_returned_without_evaluating() {
    let mut calls = 0;
    let mut f = |_t: f64| {
        calls += 1;
        Ok(0.0)
    };
    let got = refine_itp(&mut f, T0, -1.0, T0 + 0.4 * TOL, 1.0).unwrap();
    assert_eq!(got, T0 + 0.4 * TOL);
    assert_eq!(calls, 0);
}

#[test]
fn itp_and_bisection_agree_to_the_tolerance() {
    for k in 0..200_u32 {
        let root = T0 + (f64::from(k) + 0.5) / 200.0 * HOUR;
        let g = move |t: f64| (std::f64::consts::TAU * (t - root)).sin();
        let (a, _) = counted(&itp, &g, T0, T0 + HOUR);
        let (b, _) = counted(&bisection, &g, T0, T0 + HOUR);
        assert!((a - b).abs() <= TOL, "root {k}: {} s apart", (a - b).abs() * 86_400.0);
    }
}

#[test]
fn an_error_from_the_residual_propagates() {
    let mut f = |_t: f64| Err(EventError::OutOfWindow { julian_day: T0 });
    assert!(matches!(
        refine_itp(&mut f, T0, -1.0, T0 + HOUR, 1.0),
        Err(EventError::OutOfWindow { .. })
    ));
}
```

Declare it in `root.rs` next to `mod level_tests;`:

```rust
#[cfg(test)]
mod refine_tests;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd $WT && cargo test -p pleiades-events --lib root::refine_tests 2>&1 | tail -10`
Expected: compile error `cannot find function refine_itp`.

- [ ] **Step 3: Implement `refine_itp`**

In `crates/pleiades-events/src/root.rs`, insert after `bisect` (ends at line 50):

```rust
/// `κ₁` of the ITP method as a fraction of the initial bracket's width
/// (Oliveira & Takahashi 2020 recommend 0.2 / (b − a)).
const ITP_KAPPA1_SCALE: f64 = 0.2;

/// `n₀` of the ITP method: the evaluations it may spend beyond bisection's
/// count in the worst case.
const ITP_N0: f64 = 1.0;

/// Refines a sign change of `f` across `[lo, hi]` with the ITP method
/// (interpolate, truncate, project: Oliveira & Takahashi, "An Enhancement of
/// the Bisection Method Average Performance Preserving Minmax Optimality",
/// ACM Trans. Math. Softw. 47(1), 2020), with `κ₂ = 2`.
///
/// The contract is [`bisect`]'s: the LATER end of a final bracket no wider
/// than [`REFINE_TOLERANCE_DAYS`] is returned, and an evaluation at or
/// below zero moves the end that is at or below zero, so the returned
/// instant is settled. A regula-falsi step aimed at the root lands close to
/// it on a smooth residual, so a rise or set settles in a few evaluations
/// where bisection takes thirteen from an hour. The projection keeps every
/// step within the bisection worst case, so no residual costs more than
/// `ITP_N0` evaluations beyond bisection's count (issue #204).
pub(crate) fn refine_itp<F>(
    f: &mut F,
    mut lo: f64,
    mut f_lo: f64,
    mut hi: f64,
    mut f_hi: f64,
) -> Result<f64, EventError>
where
    F: FnMut(f64) -> Result<f64, EventError>,
{
    let width = hi - lo;
    if width <= REFINE_TOLERANCE_DAYS {
        return Ok(hi);
    }
    let epsilon = 0.5 * REFINE_TOLERANCE_DAYS;
    let kappa1 = ITP_KAPPA1_SCALE / width;
    let n_max = (width / REFINE_TOLERANCE_DAYS).log2().ceil() + ITP_N0;
    let mut step = 0.0_f64;
    while (hi - lo) > REFINE_TOLERANCE_DAYS {
        let mid = 0.5 * (lo + hi);
        let span = hi - lo;
        let radius = (epsilon * (n_max - step).exp2() - 0.5 * span).max(0.0);
        let delta = kappa1 * span * span;
        // Interpolate: the regula-falsi point. The ends carry opposite sign
        // classes, so `f_lo != f_hi`.
        let falsi = (hi * f_lo - lo * f_hi) / (f_lo - f_hi);
        // Truncate: step `delta` past it towards the midpoint.
        let towards_mid = (mid - falsi).signum();
        let truncated = if delta <= (mid - falsi).abs() {
            falsi + towards_mid * delta
        } else {
            mid
        };
        // Project: stay within `radius` of the midpoint.
        let projected = if (truncated - mid).abs() <= radius {
            truncated
        } else {
            mid - towards_mid * radius
        };
        let x = if projected.is_finite() && projected > lo && projected < hi {
            projected
        } else {
            mid
        };
        let f_x = f(x)?;
        if (f_lo <= 0.0) == (f_x <= 0.0) {
            lo = x;
            f_lo = f_x;
        } else {
            hi = x;
            f_hi = f_x;
        }
        step += 1.0;
    }
    Ok(hi)
}
```

- [ ] **Step 4: Run the tests**

Run: `cd $WT && cargo test -p pleiades-events --lib root:: 2>&1 | grep -E "^test .*(FAILED|ok)$|test result"`
Expected: all of `root::refine_tests` pass, and `root::tests` and `root::level_tests` still pass.

If `a_linear_residual_settles_in_a_few_evaluations` or `a_sinusoidal_residual_settles_well_under_bisection` fails only on its count bound, print the measured counts and report them. Do not loosen the bound without that report. The contract tests (settled, within tolerance, at most bisection plus one) must pass as written.

- [ ] **Step 5: Format, lint, commit**

```bash
cd $WT && cargo fmt --all && cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings
git add crates/pleiades-events/src/root.rs crates/pleiades-events/src/root/refine_tests.rs
git commit -m "feat(events): add an ITP bracket refiner with bisect's settled-end contract (#204)"
```

---

### Task 4: The rise/set scanner refines with ITP

**Files:**
- Modify: `crates/pleiades-events/src/rise_trans/scan.rs:1-15` (module doc), `:43` (import), `:297-302` (walk doc), `:377` (call)
- Modify: `crates/pleiades-events/src/rise_trans/mod.rs:33-40` (the `RISE_SET_STEP_DAYS` doc wording), `:499`

**Interfaces:**
- Consumes: `root::refine_itp(f, lo, f_lo, hi, f_hi)` (Task 3); `Sample { jd, f }` in `scan.rs`.

- [ ] **Step 1: Switch the call**

In `scan.rs`, change `use crate::root::bisect;` to `use crate::root::refine_itp;`. In `walk`'s `emit` closure, change

```rust
        let root = bisect(f, earlier.jd, earlier.f, later.jd)?;
```

to

```rust
        let root = refine_itp(f, earlier.jd, earlier.f, later.jd, later.f)?;
```

- [ ] **Step 2: Correct the docs that name bisection for this scanner**

Read `scan.rs` lines 1–15 and 295–302, and `mod.rs` lines 33–40 and 499:
- Where they say a bracket is refined "by bisection", say it is refined "by ITP (`root::refine_itp`)".
- Where they cite `root::bisect` returning the settled end, cite `root::refine_itp`.
- Keep "the bisection tolerance" wording only where it names `REFINE_TOLERANCE_DAYS`, and rephrase it as "the refinement tolerance".

Leave every other statement as it is.

- [ ] **Step 3: Run the rise/set suites**

Run: `cd $WT && cargo test -p pleiades-events --lib rise_trans 2>&1 | grep -E "FAILED|panicked|test result"`
Expected: `test result: ok`. The chaining, window-edge, scanner, cost and cache tests must pass unchanged. If a scanner test pins an exact evaluation count or a bisection-specific instant, stop and report it with its assertion. Do not edit it.

- [ ] **Step 4: Run the crate's full tests**

Run: `cd $WT && cargo test -p pleiades-events 2>&1 | grep -E "FAILED|panicked|test result"`
Expected: every `test result: ok`.

- [ ] **Step 5: Run the rise-trans gate and compare with the baseline**

Run: `cd $WT && cargo run --release -q -p pleiades-validate -- validate-rise-trans > $SCRATCH/gate-itp.txt 2>&1; diff $SCRATCH/gate-baseline.txt $SCRATCH/gate-itp.txt`
Expected: the gate passes. Differences, if any, are confined to the last printed digits of the Tier 1 and Tier 2 figures. If a ceiling fails, stop and report. Ceilings are not to be changed.

Also run the gate's unit test: `cd $WT && cargo test -p pleiades-validate --lib validate_gates -- rise_trans 2>&1 | grep -E "test result|FAILED"`. Expected: ok.

- [ ] **Step 6: Format, lint, commit**

```bash
cd $WT && cargo fmt --all && cargo clippy -p pleiades-events --all-targets --all-features -- -D warnings
git add crates/pleiades-events/src/rise_trans
git commit -m "perf(events): refine rise, set and transit brackets with ITP instead of bisection (#204)"
```

In the commit body, give the gate's before and after figures from Step 5.

---

### Task 5: Measure, verify, hand off

**Files:**
- None committed. Measurements go into the PR description and an issue comment.

- [ ] **Step 1: Build the Part 2 probe binary**

```bash
cargo build --release --offline --manifest-path /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204/Cargo.toml --target-dir /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target
cp /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/probe204-target/release/probe204 /tmp/claude-1000/-workspace/c9ce15d3-b49a-44a0-bd60-0803344fdca8/scratchpad/bin/bracket-itp
```

- [ ] **Step 2: Run the alternating timings**

Check `cat /proc/loadavg`. If the 1-minute load is above 3, wait (use Monitor with an until-loop, not sleep) for up to 20 minutes for it to fall. Then run regardless and record the load.

Run: `bash $SCRATCH/probe204/run.sh $SCRATCH/bin/bracket-baseline $SCRATCH/bin/bracket-cache $SCRATCH/bin/bracket-itp > $SCRATCH/timings.txt 2>&1`

Then take the best and the median of the seven rounds per binary and mode. Expected:
- baseline fresh ≈ baseline reused (about 0.78 ms; the cache does not exist there);
- cache reused well below cache fresh;
- itp reused lowest;
- `bracket-cache`'s checksum equals the baseline's.

`bracket-itp`'s checksum may differ in the last digits, because instants move by under 0.5 s.

- [ ] **Step 3: Whole-workspace checks**

```bash
cd $WT && cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
mise run docs
cargo test -p pleiades-events
```

Expected: all clean.

- [ ] **Step 4: Hand off to finishing**

Use superpowers:finishing-a-development-branch. The PR description contains:
- a summary of both parts;
- the timing table (best and median per stage and mode, with load average);
- the read counts from the cost tests;
- the gate's before and after figures;
- `Refs #204`.

The issue comment on #204 gives the same table. It states whether the bracket is now "within a few times" Swiss Ephemeris's 0.34 ms, scaling by the reporter's machine as #209 did and labelled as an estimate. It also proposes closing the issue or keeping it open.
