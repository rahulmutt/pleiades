# FU-25 Apparent-Sample Cost Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Cut the remaining multipliers on an apparent-of-date sample (issue
#128 / FU-25) without moving any position or speed bit.

**Architecture:** A new provided trait method,
`EphemerisBackend::position_without_motion`, lets VSOP87, ELP and the
fictitious backend skip the ±0.5 d finite-difference speed that callers throw
away. Every mean-place-only read (light-time re-queries, chart speed-difference
mean places, events and eclipse samples) switches to it. The chart's
provenance-only aberration Sun moves to the backend-free Meeus Sun, which
moves from `pleiades-events` to `pleiades-apparent`. Precession/nutation
sharing is measured before anything is built.

**Tech Stack:** Rust (stable, edition per workspace), cargo-nextest via
`mise run test`, `mise run ci`.

**Spec:** `docs/superpowers/specs/2026-10-04-fu25-apparent-sample-cost-design.md`

## Global Constraints

- Positions and speeds stay **bit-identical** before and after. Only
  `provenance.aberration_longitude_arcsec` may move, by at most about 0.01″.
- The crossings golden (`crossings-golden` + manifest) stays byte-identical.
  **Do not regenerate it.** If it changes, a task broke bit-identity: stop and
  investigate.
- No new field on `EphemerisRequest`. It is a pub-field struct without
  `#[non_exhaustive]`, so a new field would be a breaking change.
- `position_without_motion` contract: the place is bit-identical to
  `position`'s, `motion` is `None`, and the method may succeed where
  `position` fails only because the motion could not be computed.
- The API-stability profile does not change. The new trait method is
  additive.
- Work only in the worktree `/workspace/.claude/worktrees/fu25-apparent-perf`,
  branch `worktree-fu25-apparent-perf`. Another agent shares `/workspace`.
- Run `cargo fmt --all` before every commit. The CI fmt gate is strict.
- Never edit source or commit while a background `cargo test` / nextest run is
  in progress. Release-bundle tests compare git provenance.
- Commit messages: conventional style, end with `(#128)`.

## Review Focus

1. **A user wrapper that overrides only `position`** (for example one that
   strips distances). The default `position_without_motion` must return the
   wrapper's transformed place, not the inner backend's. Pinned in Task 1.
2. **Composite fallback on the motion-free path.** A primary that fails with a
   retryable kind must fall back to the secondary exactly as `position` does.
   Pinned in Task 1.
3. **A fictitious body at its Sun source's window edge** (packaged data
   starts 1900). `position` returns a one-sided speed; `position_without_motion`
   must return the same place. Pinned in Task 3.
4. **An apparent chart at the start of a bounded backend's window.** The
   earlier speed neighbour is unavailable. Today the missing backend Sun forces
   the one-sided speed; after Task 6 the body's own failed query must force it,
   with an identical result. Pinned by the Task 0 checksum (packaged Mars at
   the window start).
5. **A topocentric sidereal apparent chart.** The observer path goes through
   `query_mean_ecliptic` too, and must stay bit-identical. Pinned by the Task 0
   checksum.

---

## File Map

| File | Change |
|---|---|
| `crates/pleiades-backend/src/traits.rs` | new provided method; shared dispatch helpers in `CompositeBackend` / `RoutingBackend` |
| `crates/pleiades-backend/src/traits_tests.rs` | default-method and forwarding tests |
| `crates/pleiades-vsop87/src/backend.rs` | `compute(req, with_motion)`; override |
| `crates/pleiades-vsop87/src/tests/` | equivalence test (new file `position_without_motion.rs`, registered in that directory's module root) |
| `crates/pleiades-elp/src/backend.rs` | `compute(req, with_motion)`; override; equivalence test in its existing test module |
| `crates/pleiades-fict/src/backend.rs` | `compute`; override; Sun source read motion-free; tests in the inline `mod tests` |
| `crates/pleiades-core/src/chart/mod.rs` | `query_mean_ecliptic` motion-free; Meeus Sun; delete `query_sun_longitude_of_date` |
| `crates/pleiades-core/src/chart/bit_identity_tests.rs` | **new**: pinned checksum of chart outputs |
| `crates/pleiades-core/src/chart/query_count_tests.rs` | count `position` vs `position_without_motion`, Sun reads |
| `crates/pleiades-core/src/chart/apparent_tier_tests.rs` | rewrite the fail-closed Sun test |
| `crates/pleiades-events/src/ephemeris.rs` | motion-free mean reads |
| `crates/pleiades-events/src/solar.rs` | **deleted** (function moves to `pleiades-apparent`) |
| `crates/pleiades-events/src/{lib.rs,fixstar.rs}` | import change |
| `crates/pleiades-events/src/reference/tests.rs` | count motion-free vs full reads |
| `crates/pleiades-events/tests/fu25_bit_identity.rs` | **new**: pinned checksum of events outputs |
| `crates/pleiades-eclipse/src/ephemeris.rs` | `read` motion-free |
| `crates/pleiades-apparent/src/aberration.rs` (+ `aberration/tests.rs`, `lib.rs`) | `sun_true_longitude_of_date_deg` |
| `docs/follow-ups.md`, `spec/architecture.md` | FU-25 outcome; trait note |

---

### Task 0: Pin bit-identity checksums on the unchanged code

These tests must be written and pinned **before** any behaviour change. They
are the arbiter for every later task.

**Files:**
- Create: `crates/pleiades-core/src/chart/bit_identity_tests.rs`
- Modify: `crates/pleiades-core/src/chart/mod.rs` (register the module next to
  `query_count_tests`)
- Create: `crates/pleiades-events/tests/fu25_bit_identity.rs`

**Interfaces:**
- Consumes: the existing public APIs only.
- Produces: two pinned constants, `CHART_CHECKSUM` and `EVENTS_CHECKSUM`.
  Later tasks must not change them.

- [ ] **Step 1: Write the chart checksum test**

`crates/pleiades-core/src/chart/bit_identity_tests.rs`:

```rust
//! Pins every position and speed bit an apparent chart reports, so the
//! FU-25 query-shape changes (issue #128) can prove they move none of them.
//! Provenance is deliberately excluded: its aberration estimate moves by
//! design when the chart switches to the Meeus Sun.

use pleiades_apparent::fnv1a64;
use pleiades_backend::{Apparentness, CompositeBackend, EphemerisBackend};
use pleiades_data::packaged_backend;
use pleiades_elp::ElpBackend;
use pleiades_types::{
    Ayanamsa, CelestialBody, Instant, JulianDay, Latitude, Longitude, ObserverLocation,
    TimeScale, ZodiacMode,
};
use pleiades_vsop87::Vsop87Backend;

use crate::chart::{ChartEngine, ChartRequest, ChartSnapshot};

/// Value the checksum had on `main` at 91b13290d, before FU-25's second round.
const CHART_CHECKSUM: u64 = 0;

fn eleven_bodies() -> Vec<CelestialBody> {
    vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Venus,
        CelestialBody::Mars,
        CelestialBody::Jupiter,
        CelestialBody::Saturn,
        CelestialBody::Uranus,
        CelestialBody::Neptune,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ]
}

fn tt(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tt)
}

fn push_bits(out: &mut String, value: Option<f64>) {
    match value {
        Some(v) => out.push_str(&format!("{:016x};", v.to_bits())),
        None => out.push_str("none;"),
    }
}

fn render(out: &mut String, label: &str, snapshot: &ChartSnapshot) {
    out.push_str(label);
    out.push('\n');
    for placement in &snapshot.placements {
        out.push_str(&format!("{:?}:{:?}:", placement.body, placement.position.apparent));
        let ecliptic = placement.position.ecliptic;
        push_bits(out, ecliptic.map(|e| e.longitude.degrees()));
        push_bits(out, ecliptic.map(|e| e.latitude.degrees()));
        push_bits(out, ecliptic.and_then(|e| e.distance_au));
        let motion = placement.position.motion;
        push_bits(out, motion.and_then(|m| m.longitude_deg_per_day));
        push_bits(out, motion.and_then(|m| m.latitude_deg_per_day));
        push_bits(out, motion.and_then(|m| m.distance_au_per_day));
        out.push('\n');
    }
}

fn chart<B: EphemerisBackend>(backend: B, request: &ChartRequest) -> ChartSnapshot {
    ChartEngine::new(backend).chart(request).expect("chart")
}

#[test]
fn apparent_chart_outputs_are_pinned() {
    let composite = || CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new());
    let mut out = String::new();

    // Issue #128's epoch and two more spread over the composite's range.
    for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5] {
        let request = ChartRequest::new(tt(jd))
            .with_bodies(eleven_bodies())
            .with_apparentness(Apparentness::Apparent);
        render(&mut out, &format!("composite {jd}"), &chart(composite(), &request));
    }

    // Topocentric sidereal (Review Focus 5).
    let observer = ObserverLocation::new(
        Latitude::from_degrees(51.4779),
        Longitude::from_degrees(-0.0015),
        None,
    );
    let request = ChartRequest::new(tt(2_460_763.5))
        .with_bodies(eleven_bodies())
        .with_apparentness(Apparentness::Apparent)
        .with_observer(observer)
        .with_zodiac_mode(ZodiacMode::Sidereal {
            ayanamsa: Ayanamsa::Lahiri,
        })
        .with_topocentric(true);
    render(&mut out, "composite topo lahiri", &chart(composite(), &request));

    // A bounded backend at its window start (Review Focus 4): the earlier
    // speed neighbour is out of range, so the speed is one-sided.
    let packaged = packaged_backend();
    let start = packaged
        .metadata()
        .nominal_range
        .start
        .expect("packaged window has a start");
    let request = ChartRequest::new(start)
        .with_bodies(vec![CelestialBody::Mars, CelestialBody::Moon])
        .with_apparentness(Apparentness::Apparent);
    render(&mut out, "packaged window start", &chart(packaged, &request));

    let checksum = fnv1a64(&out);
    assert_eq!(
        checksum, CHART_CHECKSUM,
        "chart outputs moved: got {checksum:#018x}\n{out}"
    );
}
```

Register it in `crates/pleiades-core/src/chart/mod.rs`, after the
`mod apparent_tier_tests;` lines:

```rust
#[cfg(test)]
mod bit_identity_tests;
```

If `Ayanamsa` is not re-exported from `pleiades_types`, import it from where
`chart/sidereal_tests.rs` imports it. If `ChartSnapshot` is not reachable at
`crate::chart::ChartSnapshot`, use the path `chart/tests.rs` uses.

- [ ] **Step 2: Run it to read the actual checksum**

Run: `cargo test -p pleiades-core --lib chart::bit_identity_tests -- --nocapture`
Expected: FAIL with `chart outputs moved: got 0x…`. Check the printed rows:
every placement should be `Apparent`, and the packaged-window Mars row should
have a non-`none` longitude speed. If any placement is `Mean`, or Mars has no
speed, stop and report: the fixture does not exercise what it should.

- [ ] **Step 3: Pin the checksum**

Replace `const CHART_CHECKSUM: u64 = 0;` with the printed value, for example
`const CHART_CHECKSUM: u64 = 0x1234_5678_9abc_def0;`, using the exact digits
printed.

- [ ] **Step 4: Run it to verify it passes**

Run: `cargo test -p pleiades-core --lib chart::bit_identity_tests`
Expected: PASS. Run it twice more to confirm it is deterministic.

- [ ] **Step 5: Write the events checksum test**

`crates/pleiades-events/tests/fu25_bit_identity.rs`:

```rust
//! Pins every longitude, latitude, distance and speed bit the event engine
//! reports on the algorithmic backends, so the FU-25 query-shape changes
//! (issue #128) can prove they move none of them.

use pleiades_apparent::fnv1a64;
use pleiades_backend::CompositeBackend;
use pleiades_elp::ElpBackend;
use pleiades_events::{CrossingFrame, EventEngine};
use pleiades_fict::FictitiousBackend;
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

/// Value the checksum had on `main` at 91b13290d, before FU-25's second round.
const EVENTS_CHECKSUM: u64 = 0;

fn tdb(jd: f64) -> Instant {
    Instant::new(JulianDay::from_days(jd), TimeScale::Tdb)
}

fn bits(out: &mut String, value: Option<f64>) {
    match value {
        Some(v) => out.push_str(&format!("{:016x};", v.to_bits())),
        None => out.push_str("none;"),
    }
}

#[test]
fn event_engine_outputs_are_pinned() {
    let mut out = String::new();
    let engine = EventEngine::new(CompositeBackend::new(
        ElpBackend::new(),
        Vsop87Backend::new(),
    ));
    let bodies = [
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mercury,
        CelestialBody::Mars,
        CelestialBody::Saturn,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ];
    let frames = [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
        CrossingFrame::Heliocentric,
    ];
    for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5] {
        for body in &bodies {
            for frame in frames {
                out.push_str(&format!("{jd}:{body:?}:{frame:?}:"));
                match engine.longitude_at(body.clone(), frame, tdb(jd)) {
                    Ok(lon) => bits(&mut out, Some(lon.degrees())),
                    Err(error) => out.push_str(&format!("err({error});")),
                }
                match engine.position_at(body.clone(), frame, tdb(jd)) {
                    Ok(p) => {
                        bits(&mut out, Some(p.ecliptic.longitude.degrees()));
                        bits(&mut out, Some(p.ecliptic.latitude.degrees()));
                        bits(&mut out, p.ecliptic.distance_au);
                        bits(&mut out, p.motion.longitude_deg_per_day);
                        bits(&mut out, p.motion.latitude_deg_per_day);
                        bits(&mut out, p.motion.distance_au_per_day);
                    }
                    Err(error) => out.push_str(&format!("err({error});")),
                }
                out.push('\n');
            }
        }
    }

    // A fictitious body over the VSOP87 Sun source (Task 3 changes how that
    // source is read).
    let fict = EventEngine::new(FictitiousBackend::new(Vsop87Backend::new()));
    for jd in [2_460_763.5, 2_451_545.0] {
        let p = fict
            .position_at(
                CelestialBody::Cupido,
                CrossingFrame::GeocentricApparentOfDate,
                tdb(jd),
            )
            .expect("Cupido");
        out.push_str(&format!("fict {jd}:"));
        bits(&mut out, Some(p.ecliptic.longitude.degrees()));
        bits(&mut out, Some(p.ecliptic.latitude.degrees()));
        bits(&mut out, p.ecliptic.distance_au);
        bits(&mut out, p.motion.longitude_deg_per_day);
        out.push('\n');
    }

    let checksum = fnv1a64(&out);
    assert_eq!(
        checksum, EVENTS_CHECKSUM,
        "event outputs moved: got {checksum:#018x}\n{out}"
    );
}
```

If `p.motion`'s fields are not `Option<f64>` (check `EclipticPosition` in
`crates/pleiades-events/src/position.rs`), wrap them in `Some(..)`. Error
rows are deliberate: an unsupported body/frame pair is pinned as an error
string, so any change in which pairs succeed also moves the checksum.

- [ ] **Step 6: Run it, read the checksum, pin it, re-run**

Run: `cargo test -p pleiades-events --test fu25_bit_identity -- --nocapture`
Expected: first FAIL with `event outputs moved: got 0x…`. Check that most rows
are numeric, not `err(…)`. Pin the printed value in `EVENTS_CHECKSUM`, then
re-run and expect PASS.

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
git add crates/pleiades-core/src/chart/bit_identity_tests.rs crates/pleiades-core/src/chart/mod.rs crates/pleiades-events/tests/fu25_bit_identity.rs
git commit -m "test(core,events): pin apparent-place output bits before FU-25 round two (#128)"
```

---

### Task 1: `position_without_motion` on the trait, forwarded by the composites

**Files:**
- Modify: `crates/pleiades-backend/src/traits.rs:15-34` (trait),
  `:117-138` (`CompositeBackend::position`), `:265-297`
  (`RoutingBackend::position`)
- Test: `crates/pleiades-backend/src/traits_tests.rs`

**Interfaces:**
- Produces:
  `fn position_without_motion(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError>`
  on `EphemerisBackend`, with a default implementation. Every later task calls
  it.

- [ ] **Step 1: Write the failing tests**

Append to `crates/pleiades-backend/src/traits_tests.rs`:

```rust
/// Reports a motion on every result and counts which entry point was used.
struct MotionBackend {
    body: CelestialBody,
    longitude: f64,
    fail_with: Option<EphemerisErrorKind>,
    full: AtomicUsize,
    motion_free: AtomicUsize,
}

impl MotionBackend {
    fn new(body: CelestialBody, longitude: f64) -> Self {
        Self {
            body,
            longitude,
            fail_with: None,
            full: AtomicUsize::new(0),
            motion_free: AtomicUsize::new(0),
        }
    }

    fn failing(body: CelestialBody, kind: EphemerisErrorKind) -> Self {
        Self {
            fail_with: Some(kind),
            ..Self::new(body, 0.0)
        }
    }

    fn result(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        if let Some(kind) = self.fail_with.clone() {
            return Err(EphemerisError::new(kind, "configured failure"));
        }
        let mut result = EphemerisResult::new(
            BackendId::new("motion"),
            req.body.clone(),
            req.instant,
            req.frame,
            req.zodiac_mode.clone(),
            req.apparent,
        );
        result.ecliptic = Some(EclipticCoordinates::new(
            Longitude::from_degrees(self.longitude),
            Latitude::from_degrees(0.0),
            Some(1.0),
        ));
        result.motion = Some(Motion::new(Some(1.0), Some(0.0), Some(0.0)));
        Ok(result)
    }
}

impl EphemerisBackend for MotionBackend {
    fn metadata(&self) -> BackendMetadata {
        ToyBackend.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        body == self.body
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.full.fetch_add(1, Ordering::SeqCst);
        self.result(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.motion_free.fetch_add(1, Ordering::SeqCst);
        let mut result = self.result(req)?;
        result.motion = None;
        Ok(result)
    }
}

fn sun_request() -> EphemerisRequest {
    EphemerisRequest::new(
        CelestialBody::Sun,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    )
}

#[test]
fn default_position_without_motion_is_position_with_motion_dropped() {
    // ToyBackend overrides only `position`.
    let full = ToyBackend.position(&sun_request()).unwrap();
    let free = ToyBackend.position_without_motion(&sun_request()).unwrap();
    assert_eq!(free.motion, None);
    assert_eq!(EphemerisResult { motion: None, ..full }, free);
}

#[test]
fn default_position_without_motion_keeps_a_wrappers_transformation() {
    // Review Focus 1: a wrapper that overrides only `position` must not be
    // bypassed by the default motion-free path.
    struct NoDistance<B>(B);
    impl<B: EphemerisBackend> EphemerisBackend for NoDistance<B> {
        fn metadata(&self) -> BackendMetadata {
            self.0.metadata()
        }
        fn supports_body(&self, body: CelestialBody) -> bool {
            self.0.supports_body(body)
        }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            let mut result = self.0.position(req)?;
            if let Some(ecliptic) = result.ecliptic.as_mut() {
                ecliptic.distance_au = None;
            }
            Ok(result)
        }
    }
    let wrapped = NoDistance(MotionBackend::new(CelestialBody::Sun, 10.0));
    let free = wrapped.position_without_motion(&sun_request()).unwrap();
    assert_eq!(free.ecliptic.unwrap().distance_au, None);
    assert_eq!(free.motion, None);
}

#[test]
fn composite_forwards_position_without_motion_to_the_serving_backend() {
    let composite = CompositeBackend::new(
        MotionBackend::new(CelestialBody::Moon, 1.0),
        MotionBackend::new(CelestialBody::Sun, 2.0),
    );
    let free = composite.position_without_motion(&sun_request()).unwrap();
    assert_eq!(free.ecliptic.unwrap().longitude.degrees(), 2.0);
    assert_eq!(free.motion, None);
    assert_eq!(composite.secondary().motion_free.load(Ordering::SeqCst), 1);
    assert_eq!(composite.secondary().full.load(Ordering::SeqCst), 0);
    assert_eq!(composite.primary().motion_free.load(Ordering::SeqCst), 0);
}

#[test]
fn composite_motion_free_path_falls_back_like_position() {
    // Review Focus 2.
    let composite = CompositeBackend::new(
        MotionBackend::failing(CelestialBody::Sun, EphemerisErrorKind::MissingDataset),
        MotionBackend::new(CelestialBody::Sun, 2.0),
    );
    let full = composite.position(&sun_request()).unwrap();
    let free = composite.position_without_motion(&sun_request()).unwrap();
    assert_eq!(EphemerisResult { motion: None, ..full }, free);
    assert_eq!(composite.secondary().motion_free.load(Ordering::SeqCst), 1);
    assert_eq!(composite.secondary().full.load(Ordering::SeqCst), 1);
}

#[test]
fn routing_forwards_position_without_motion_and_falls_back() {
    let routing = RoutingBackend::new(vec![
        Box::new(MotionBackend::failing(
            CelestialBody::Sun,
            EphemerisErrorKind::MissingDataset,
        )),
        Box::new(MotionBackend::new(CelestialBody::Moon, 1.0)),
        Box::new(MotionBackend::new(CelestialBody::Sun, 3.0)),
    ]);
    let full = routing.position(&sun_request()).unwrap();
    let free = routing.position_without_motion(&sun_request()).unwrap();
    assert_eq!(free.ecliptic.unwrap().longitude.degrees(), 3.0);
    assert_eq!(EphemerisResult { motion: None, ..full }, free);
}
```

The `RoutingBackend` test can't read the boxed backends' counters. Its
equality with `position` plus the `3.0` longitude proves that it routed and
fell back. Adjust the `use` lines at the top of the file if `Motion`,
`JulianDay`, `Latitude` or `Longitude` are not in scope through `crate::*`.
Import them from `pleiades_types`.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-backend --lib traits`
Expected: compile error, `no method named position_without_motion`.

- [ ] **Step 3: Add the trait method**

In `crates/pleiades-backend/src/traits.rs`, after `fn position(…)` in the
trait:

```rust
    /// Computes a single ephemeris result for a caller that does not need
    /// its motion.
    ///
    /// The returned place is bit-identical to [`Self::position`]'s and
    /// `motion` is `None`. A backend that derives motion from extra
    /// evaluations (a finite difference, say) overrides this to skip them;
    /// the light-time iteration of an apparent place re-queries a body and
    /// discards the motion each time. An override may succeed where
    /// [`Self::position`] fails only because the motion could not be
    /// computed. A wrapper that transforms results should override this too,
    /// or it falls back to its own [`Self::position`], which stays correct
    /// but pays for the motion.
    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        let mut result = self.position(req)?;
        result.motion = None;
        Ok(result)
    }
```

- [ ] **Step 4: Share the dispatch in `CompositeBackend`**

Replace the body of `CompositeBackend::position` with a call to a private
helper, and add the forwarding method:

```rust
impl<A: EphemerisBackend, B: EphemerisBackend> CompositeBackend<A, B> {
    /// Routes `req` to the primary or secondary backend through `query`, so
    /// every entry point shares one fallback policy.
    fn dispatch(
        &self,
        req: &EphemerisRequest,
        query: impl Fn(&dyn EphemerisBackend) -> Result<EphemerisResult, EphemerisError>,
    ) -> Result<EphemerisResult, EphemerisError> {
        let primary_supports = self.primary.supports_body(req.body.clone());
        let secondary_supports = self.secondary.supports_body(req.body.clone());

        if primary_supports {
            match query(&self.primary) {
                Ok(result) => Ok(result),
                Err(error) if secondary_supports && should_fallback_to_secondary(&error.kind) => {
                    query(&self.secondary)
                }
                Err(error) => Err(error),
            }
        } else if secondary_supports {
            query(&self.secondary)
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the composite router supports the requested body",
            ))
        }
    }
}
```

And in `impl EphemerisBackend for CompositeBackend<A, B>`:

```rust
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.dispatch(req, |backend| backend.position(req))
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.dispatch(req, |backend| backend.position_without_motion(req))
    }
```

`&self.primary` coerces to `&dyn EphemerisBackend` because `A: EphemerisBackend`
(the trait is object-safe, as `RoutingBackend`'s `Box<dyn EphemerisBackend>`
shows). If the existing `impl<A, B> CompositeBackend<A, B>` block with `new`
has no trait bounds, put `dispatch` in a separate bounded `impl` block as shown.

- [ ] **Step 5: Share the dispatch in `RoutingBackend`**

```rust
impl RoutingBackend {
    /// Walks the chain through `query`, so every entry point shares one
    /// fallback policy.
    fn dispatch(
        &self,
        req: &EphemerisRequest,
        query: impl Fn(&dyn EphemerisBackend) -> Result<EphemerisResult, EphemerisError>,
    ) -> Result<EphemerisResult, EphemerisError> {
        let mut saw_support = false;
        let mut last_retryable_error = None;

        for backend in &self.backends {
            if !backend.supports_body(req.body.clone()) {
                continue;
            }

            saw_support = true;
            match query(backend.as_ref()) {
                Ok(result) => return Ok(result),
                Err(error) if should_fallback_to_secondary(&error.kind) => {
                    last_retryable_error = Some(error);
                }
                Err(error) => return Err(error),
            }
        }

        if let Some(error) = last_retryable_error {
            Err(error)
        } else if saw_support {
            Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "configured providers could not satisfy the requested body and request shape",
            ))
        } else {
            Err(EphemerisError::new(
                EphemerisErrorKind::UnsupportedBody,
                "no backend in the routing chain supports the requested body",
            ))
        }
    }
}
```

In the trait impl, `position` calls `self.dispatch(req, |backend|
backend.position(req))` and `position_without_motion` calls
`self.dispatch(req, |backend| backend.position_without_motion(req))`. Put
`dispatch` in the existing inherent `impl RoutingBackend` block if there is
one.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p pleiades-backend`
Expected: PASS, both the new tests and every existing one (the dispatch
refactor must not change routing).

- [ ] **Step 7: Commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-backend --all-targets --all-features -- -D warnings
git add crates/pleiades-backend/src/traits.rs crates/pleiades-backend/src/traits_tests.rs
git commit -m "feat(backend): add EphemerisBackend::position_without_motion (#128)"
```

---

### Task 2: VSOP87 and ELP skip the finite-difference speed

**Files:**
- Modify: `crates/pleiades-vsop87/src/backend.rs:433-483`
- Modify: `crates/pleiades-elp/src/backend.rs:320-392`
- Test: `crates/pleiades-vsop87/src/tests/position_without_motion.rs` (new,
  registered in `crates/pleiades-vsop87/src/tests/mod.rs` or whatever file is
  that directory's module root; check with `ls crates/pleiades-vsop87/src/tests`)
- Test: the ELP backend's existing `#[cfg(test)]` module (find it with
  `grep -n "cfg(test)" crates/pleiades-elp/src/backend.rs`)

**Interfaces:**
- Consumes: `EphemerisBackend::position_without_motion` (Task 1).
- Produces: overrides on `Vsop87Backend` and `ElpBackend`; private
  `fn compute(&self, req: &EphemerisRequest, with_motion: bool) -> Result<EphemerisResult, EphemerisError>`
  in each.

- [ ] **Step 1: Write the failing VSOP87 test**

`crates/pleiades-vsop87/src/tests/position_without_motion.rs`:

```rust
//! `position_without_motion` returns `position`'s place bit for bit and no
//! motion (issue #128: the light-time re-queries discard the motion, which
//! cost two of every three series evaluations).

use pleiades_backend::{EphemerisBackend, EphemerisRequest, EphemerisResult};
use pleiades_types::{CelestialBody, CoordinateFrame, Instant, JulianDay, TimeScale};

use crate::Vsop87Backend;

#[test]
fn position_without_motion_is_position_minus_motion() {
    let backend = Vsop87Backend::new();
    // Pluto at 1885.0 / 2099.9 crosses its fit-window edges.
    for jd in [2_460_763.5, 2_451_545.0, 2_309_103.5, 2_488_069.5, 2_381_000.5] {
        for body in Vsop87Backend::supported_bodies() {
            for frame in [CoordinateFrame::Ecliptic, CoordinateFrame::Equatorial] {
                let mut req = EphemerisRequest::new(
                    body.clone(),
                    Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
                );
                req.frame = frame;
                let full = backend.position(&req).expect("position");
                let free = backend.position_without_motion(&req).expect("motion-free");
                assert!(full.motion.is_some(), "{body:?} {jd}");
                assert_eq!(
                    EphemerisResult { motion: None, ..full },
                    free,
                    "{body:?} {jd} {frame:?}"
                );
            }
        }
    }
}
```

`assert_eq!` on `EphemerisResult` compares the `f64` fields with `==`, which
is exact for finite values. That is the bit-identity check: the places come
from one code path, so they are expected to be exactly equal, not merely
close.

If `supported_bodies()` is not callable from the tests module or returns a
slice of references, adapt the iteration (`.iter().cloned()`). Register the
file: `mod position_without_motion;` in the tests directory's module root.

- [ ] **Step 2: Run it to verify it fails as expected**

Run: `cargo test -p pleiades-vsop87 --lib position_without_motion`
Expected: PASS already, through the default method. This test guards the
override rather than driving it. To prove it can catch a broken override,
temporarily add
`fn position_without_motion(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> { self.position(req) }`
(it keeps the motion) to `impl EphemerisBackend for Vsop87Backend`, re-run,
and expect FAIL on `motion`. Then delete that temporary method.

- [ ] **Step 3: Implement the VSOP87 override**

In `crates/pleiades-vsop87/src/backend.rs`, move the body of `position`
(lines 433-483) into an inherent method and make the motion conditional:

```rust
impl Vsop87Backend {
    /// One request's result. `with_motion` adds the finite-difference speed,
    /// which costs two more geocentric evaluations than the place itself.
    fn compute(
        &self,
        req: &EphemerisRequest,
        with_motion: bool,
    ) -> Result<EphemerisResult, EphemerisError> {
        // … the existing body of `position`, unchanged, except its last
        // statements become:
        result.ecliptic = Some(Self::to_ecliptic(geocentric));
        result.equatorial = Some(Self::to_equatorial(geocentric, req.instant));
        if with_motion {
            result.motion = Self::motion(req.body.clone(), days, pluto_path);
        }
        Ok(result)
    }
}
```

Move the existing validation, `days`, `pluto_path`, `geocentric` and
`result` construction into `compute` verbatim, in the same order. Then in
`impl EphemerisBackend for Vsop87Backend`:

```rust
    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.compute(req, true)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.compute(req, false)
    }
```

Put `compute` in the existing `impl Vsop87Backend` block that holds `motion`,
not in a new block.

- [ ] **Step 4: Run the VSOP87 tests**

Run: `cargo test -p pleiades-vsop87`
Expected: PASS.

- [ ] **Step 5: Write the ELP test**

In the ELP backend's test module:

```rust
#[test]
fn position_without_motion_is_position_minus_motion() {
    let backend = ElpBackend::new();
    for jd in [2_460_763.5, 2_451_545.0, 2_433_282.5, 2_488_069.5] {
        for body in lunar_theory_supported_bodies() {
            let req = EphemerisRequest::new(
                body.clone(),
                Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
            );
            let full = backend.position(&req);
            let free = backend.position_without_motion(&req);
            match (full, free) {
                (Ok(full), Ok(free)) => {
                    assert!(full.motion.is_some(), "{body:?} {jd}");
                    assert_eq!(EphemerisResult { motion: None, ..full }, free, "{body:?} {jd}");
                }
                (Err(full), Err(free)) => assert_eq!(full, free, "{body:?} {jd}"),
                (full, free) => panic!("{body:?} {jd}: {full:?} vs {free:?}"),
            }
        }
    }
}
```

`lunar_theory_supported_bodies()` is what `ElpBackend::supports_body` uses
(`backend.rs:317`). Adjust its call shape if it returns a slice. If
`EphemerisError` has no `PartialEq`, compare `.kind` and `.message`.

- [ ] **Step 6: Implement the ELP override**

Same shape as VSOP87: move the body of `ElpBackend::position`
(`backend.rs:320-392`) into
`fn compute(&self, req: &EphemerisRequest, with_motion: bool)` in the
`impl ElpBackend` block that holds `motion`, and replace its final
`result.motion = Self::motion(body, days);` with:

```rust
        if with_motion {
            result.motion = Self::motion(body, days);
        }
```

`position` calls `self.compute(req, true)` and `position_without_motion` calls
`self.compute(req, false)`.

- [ ] **Step 7: Run the ELP tests and the Task 0 checksums**

Run: `cargo test -p pleiades-elp && cargo test -p pleiades-core --lib chart::bit_identity_tests && cargo test -p pleiades-events --test fu25_bit_identity`
Expected: PASS. Nothing calls the new method yet, so the checksums cannot
have moved; this guards the refactor of `position` itself.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-vsop87 -p pleiades-elp --all-targets --all-features -- -D warnings
git add crates/pleiades-vsop87 crates/pleiades-elp
git commit -m "perf(vsop87,elp): skip the finite-difference speed for motion-free queries (#128)"
```

---

### Task 3: The fictitious backend skips its speed and reads its Sun motion-free

**Files:**
- Modify: `crates/pleiades-fict/src/backend.rs:53-64` (`earth_heliocentric`),
  `:202-225` (`position`)
- Test: the inline `#[cfg(test)] mod tests` in the same file (around line 230)

**Interfaces:**
- Consumes: `EphemerisBackend::position_without_motion` (Task 1).
- Produces: override on `FictitiousBackend<S>`.

- [ ] **Step 1: Write the failing tests**

In the inline test module (which already defines `StubSun` and
`WindowedSun`; reuse them):

```rust
#[test]
fn position_without_motion_is_position_minus_motion() {
    let backend = FictitiousBackend::new(StubSun);
    for body in [CelestialBody::Cupido, CelestialBody::WhiteMoon] {
        for jd in [2_451_545.0, 2_460_763.5] {
            let req = EphemerisRequest::new(
                body.clone(),
                Instant::new(JulianDay::from_days(jd), TimeScale::Tt),
            );
            let full = backend.position(&req).unwrap();
            let free = backend.position_without_motion(&req).unwrap();
            assert!(full.motion.is_some());
            assert_eq!(EphemerisResult { motion: None, ..full }, free, "{body:?} {jd}");
        }
    }
}

#[test]
fn position_without_motion_at_the_sun_sources_window_edge() {
    // Review Focus 3: at either edge of a bounded Sun source the speed is
    // one-sided; the motion-free place must equal position's.
    const LO: f64 = 2_415_020.5;
    const HI: f64 = 2_488_069.5;
    let b = FictitiousBackend::new(WindowedSun { lo: LO, hi: HI });
    for edge in [LO, HI] {
        let req = EphemerisRequest::new(
            CelestialBody::Cupido,
            Instant::new(JulianDay::from_days(edge), TimeScale::Tt),
        );
        let full = b.position(&req).unwrap();
        let free = b.position_without_motion(&req).unwrap();
        assert_eq!(EphemerisResult { motion: None, ..full }, free, "{edge}");
    }
}

#[test]
fn position_without_motion_succeeds_where_only_the_motion_fails() {
    // The documented widening: a Sun source covering only the instant
    // itself fails both speed probes, so `position` errors, but the place
    // alone is available.
    const AT: f64 = 2_451_545.0;
    let b = FictitiousBackend::new(WindowedSun { lo: AT, hi: AT });
    let req = EphemerisRequest::new(
        CelestialBody::Cupido,
        Instant::new(JulianDay::from_days(AT), TimeScale::Tt),
    );
    assert!(b.position(&req).is_err());
    let free = b.position_without_motion(&req).unwrap();
    assert!(free.ecliptic.is_some());
    assert_eq!(free.motion, None);
}

#[test]
fn the_sun_source_is_read_without_motion() {
    use core::sync::atomic::{AtomicUsize, Ordering};
    struct CountingSun {
        full: AtomicUsize,
        motion_free: AtomicUsize,
    }
    impl EphemerisBackend for CountingSun {
        fn metadata(&self) -> BackendMetadata {
            StubSun.metadata()
        }
        fn supports_body(&self, body: CelestialBody) -> bool {
            StubSun.supports_body(body)
        }
        fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
            self.full.fetch_add(1, Ordering::SeqCst);
            StubSun.position(req)
        }
        fn position_without_motion(
            &self,
            req: &EphemerisRequest,
        ) -> Result<EphemerisResult, EphemerisError> {
            self.motion_free.fetch_add(1, Ordering::SeqCst);
            StubSun.position_without_motion(req)
        }
    }
    let backend = FictitiousBackend::new(CountingSun {
        full: AtomicUsize::new(0),
        motion_free: AtomicUsize::new(0),
    });
    let req = EphemerisRequest::new(
        CelestialBody::Cupido,
        Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt),
    );
    backend.position(&req).unwrap();
    assert_eq!(backend.sun_source.full.load(Ordering::SeqCst), 0);
    // The place plus two speed probes.
    assert_eq!(backend.sun_source.motion_free.load(Ordering::SeqCst), 3);
}
```

`WindowedSun { lo, hi }` and the edge JDs are copied from
`motion_falls_back_to_one_sided_difference_at_a_data_window_edge` (line 335).
`WindowedSun` overrides only `position`, so after Step 3 the fict backend reaches
it through the default `position_without_motion`, which is the Review Focus 1
path. If `CelestialBody::WhiteMoon` is spelled
differently, use `grep -n "WhiteMoon\|White" crates/pleiades-fict/src/elements.rs`.

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p pleiades-fict --lib`
Expected: `the_sun_source_is_read_without_motion` FAILS (`full` is 3) and
`position_without_motion_succeeds_where_only_the_motion_fails` FAILS (the
default method propagates `position`'s error). The two equivalence tests
already pass through the default method; they guard the override.

- [ ] **Step 3: Implement**

In `earth_heliocentric`, change
`let sun = self.sun_source.position(&req)?;` to:

```rust
        // Only the Sun's place is used; its motion would cost a bounded or
        // finite-differencing source extra evaluations (issue #128).
        let sun = self.sun_source.position_without_motion(&req)?;
```

Move the body of `position` into an inherent
`fn compute(&self, req: &EphemerisRequest, with_motion: bool) -> Result<EphemerisResult, EphemerisError>`
in the existing `impl<S: EphemerisBackend> FictitiousBackend<S>` block, with
its last lines becoming:

```rust
        result.ecliptic = Some(self.geocentric_ecliptic(el, req.instant)?);
        if with_motion {
            result.motion = Some(self.motion(el, req.instant)?);
        }
        Ok(result)
```

`position` calls `self.compute(req, true)` and `position_without_motion` calls
`self.compute(req, false)`.

- [ ] **Step 4: Run the tests and checksums**

Run: `cargo test -p pleiades-fict && cargo test -p pleiades-events --test fu25_bit_identity`
Expected: PASS. The events checksum covers Cupido over the VSOP87 Sun, whose
read now goes motion-free, so this proves the place did not move.

- [ ] **Step 5: Commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-fict --all-targets --all-features -- -D warnings
git add crates/pleiades-fict
git commit -m "perf(fict): read the Sun source and serve motion-free queries without the speed (#128)"
```

---

### Task 4: Switch the mean-place-only reads to the motion-free path

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:745-767` (`query_mean_ecliptic`)
- Modify: `crates/pleiades-events/src/ephemeris.rs:44-92` and the callers at
  `:255` and `:291`
- Modify: `crates/pleiades-eclipse/src/ephemeris.rs:50-58` (`read`)
- Test: `crates/pleiades-core/src/chart/query_count_tests.rs`,
  `crates/pleiades-events/src/reference/tests.rs`

**Interfaces:**
- Consumes: `position_without_motion` (Task 1).
- Produces (events, `pub(crate)`):
  `fn read_mean_place_without_motion<B: EphemerisBackend>(backend: &B, body: CelestialBody, body_label: &'static str, julian_day: f64) -> Result<MeanPlace, EventError>`.
  `read_mean_place` and `read_mean_ecliptic_with_motion` keep their
  signatures and keep calling `position`.

- [ ] **Step 1: Make the core `Recording` backend tell the two paths apart**

In `crates/pleiades-core/src/chart/query_count_tests.rs`, change the log
entry to record which entry point was used:

```rust
/// Which backend entry point a query used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Entry {
    Full,
    MotionFree,
}

type QueryLog = Arc<Mutex<Vec<(CelestialBody, f64, Entry)>>>;

impl<B> Recording<B> {
    fn record(&self, req: &EphemerisRequest, entry: Entry) {
        self.queries
            .lock()
            .unwrap()
            .push((req.body.clone(), req.instant.julian_day.days(), entry));
    }
}

impl<B: EphemerisBackend> EphemerisBackend for Recording<B> {
    fn metadata(&self) -> BackendMetadata {
        self.inner.metadata()
    }

    fn supports_body(&self, body: CelestialBody) -> bool {
        self.inner.supports_body(body)
    }

    fn position(&self, req: &EphemerisRequest) -> Result<EphemerisResult, EphemerisError> {
        self.record(req, Entry::Full);
        self.inner.position(req)
    }

    fn position_without_motion(
        &self,
        req: &EphemerisRequest,
    ) -> Result<EphemerisResult, EphemerisError> {
        self.record(req, Entry::MotionFree);
        self.inner.position_without_motion(req)
    }
}
```

Update the existing filter in
`an_apparent_chart_reads_each_body_once_per_sampled_instant` to destructure
three fields: `.filter(|(b, j, _)| b == body && *j == jd)`.

- [ ] **Step 2: Write the failing core test**

Append:

```rust
#[test]
fn only_the_position_batch_asks_a_backend_for_motion() {
    // Issue #128: the light-time re-queries and the speed-difference mean
    // places discard the motion, so they use the motion-free entry point;
    // only the position batch, whose motion the chart reports, pays for it.
    let bodies = vec![
        CelestialBody::Sun,
        CelestialBody::Moon,
        CelestialBody::Mars,
        CelestialBody::Pluto,
        CelestialBody::TrueNode,
    ];
    let queries = QueryLog::default();
    let backend = Recording {
        inner: composite(),
        queries: Arc::clone(&queries),
    };
    ChartEngine::new(backend)
        .chart(&apparent_request(bodies.clone()))
        .expect("apparent chart");
    let queries = queries.lock().unwrap().clone();
    for body in &bodies {
        let full: Vec<_> = queries
            .iter()
            .filter(|(b, _, e)| b == body && *e == Entry::Full)
            .collect();
        assert_eq!(full.len(), 1, "{body:?}: {queries:?}");
        assert_eq!(full[0].1, ISSUE_128_JD, "{body:?}: the batch is at the chart instant");
    }
}
```

The Sun gets one `Full` read (the batch) at this stage, and its extra
aberration-argument reads are motion-free. Task 6 removes those.

- [ ] **Step 3: Run it to verify it fails**

Run: `cargo test -p pleiades-core --lib chart::query_count_tests`
Expected: FAIL, more than one `Full` read per body (the re-queries still use
`position`).

- [ ] **Step 4: Switch `query_mean_ecliptic`**

In `crates/pleiades-core/src/chart/mod.rs`, in `query_mean_ecliptic`, change
`let result = self.backend.position(&req)?;` to:

```rust
        // Every caller reads the place only: the light-time re-queries, the
        // speed-difference mean places and the geocentric Sun / lunar-point
        // reads. The motion-free entry point skips a finite-difference speed
        // the backend would compute and this discards (issue #128).
        let result = self.backend.position_without_motion(&req)?;
```

- [ ] **Step 5: Run the core tests and the chart checksum**

Run: `cargo test -p pleiades-core --lib chart::query_count_tests chart::bit_identity_tests`
Expected: PASS.

- [ ] **Step 6: Write the failing events test**

In `crates/pleiades-events/src/reference/tests.rs`, give `Recording` the same
treatment as Step 1: record an `Entry` tag (define the same `Entry` enum in
this file), override `position_without_motion` to record `Entry::MotionFree`
and delegate, and update `count` and `take` to the three-field tuple. `count`
keeps its signature and counts both entries. Then append:

```rust
#[test]
fn mean_only_reads_never_ask_for_motion() {
    // Issue #128: apparent re-queries and the mean frames discard motion.
    let backend = Recording::new(packaged_backend());
    for frame in [
        CrossingFrame::GeocentricApparentOfDate,
        CrossingFrame::GeocentricMeanOfDate,
    ] {
        for body in [CelestialBody::Moon, CelestialBody::Mars, CelestialBody::TrueNode] {
            let reference = CrossingReference::tropical(frame);
            ecliptic_in(&backend, &body, &reference, ISSUE_128_JD).unwrap();
            let queries = backend.take();
            assert!(!queries.is_empty());
            assert!(
                queries.iter().all(|(_, _, e)| *e == Entry::MotionFree),
                "{frame:?} {body:?}: {queries:?}"
            );
        }
    }
}

#[test]
fn sampled_place_asks_for_motion_once() {
    // The speed sample does need the backend's motion, at its instant only.
    let backend = Recording::new(packaged_backend());
    let reference = CrossingReference::tropical(CrossingFrame::GeocentricApparentOfDate);
    sampled_place(&backend, &CelestialBody::Mars, &reference, ISSUE_128_JD).unwrap();
    let queries = backend.take();
    let full: Vec<_> = queries.iter().filter(|(_, _, e)| *e == Entry::Full).collect();
    assert_eq!(full.len(), 1, "{queries:?}");
    assert_eq!(full[0].1, ISSUE_128_JD);
}
```

`NoDistance` in this file overrides only `position`. Leave it: the default
`position_without_motion` routes through it (Review Focus 1), which is what
`sampled_place_still_requires_a_distance_for_a_body` needs.

- [ ] **Step 7: Run it to verify it fails**

Run: `cargo test -p pleiades-events --lib reference`
Expected: `mean_only_reads_never_ask_for_motion` FAILS.

- [ ] **Step 8: Switch the events reads**

In `crates/pleiades-events/src/ephemeris.rs`, replace `read_mean_place` with a
shared private helper plus two public-in-crate entry points:

```rust
/// Reads `body` through `position` (with motion) or through
/// `position_without_motion` (without), so the two share one decode.
fn query_mean_place<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
    with_motion: bool,
) -> Result<(MeanPlace, Option<Motion>), EventError> {
    let request = request(body, julian_day);
    let result = if with_motion {
        backend.position(&request)
    } else {
        backend.position_without_motion(&request)
    }
    .map_err(|e| EventError::Backend(e.to_string()))?;
    let ecliptic = result.ecliptic.ok_or(EventError::MissingCoordinates {
        body_label,
        julian_day,
    })?;
    Ok((
        (
            ecliptic.longitude.degrees(),
            ecliptic.latitude.degrees(),
            ecliptic.distance_au,
        ),
        result.motion,
    ))
}

/// (existing doc comment of `read_mean_place`, unchanged)
pub(crate) fn read_mean_place<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<(MeanPlace, Option<Motion>), EventError> {
    query_mean_place(backend, body, body_label, julian_day, true)
}

/// [`read_mean_place`] for a caller that discards the motion: the place is
/// bit-identical, and the backend skips any extra evaluations its motion
/// would cost (issue #128).
pub(crate) fn read_mean_place_without_motion<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<MeanPlace, EventError> {
    Ok(query_mean_place(backend, body, body_label, julian_day, false)?.0)
}
```

Change `read_mean_ecliptic` so it no longer goes through
`read_mean_ecliptic_with_motion`:

```rust
pub(crate) fn read_mean_ecliptic<B: EphemerisBackend>(
    backend: &B,
    body: CelestialBody,
    body_label: &'static str,
    julian_day: f64,
) -> Result<EclipticTriple, EventError> {
    let (lon, lat, distance) =
        read_mean_place_without_motion(backend, body, body_label, julian_day)?;
    let distance = distance.ok_or(EventError::MissingDistance {
        body_label,
        julian_day,
    })?;
    Ok((lon, lat, distance))
}
```

At `ephemeris.rs:255` and `:291`, where the motion is bound to `_`, replace
`let (mean, _) = read_mean_place(backend, body, body_label, julian_day)?;` with
`let mean = read_mean_place_without_motion(backend, body, body_label, julian_day)?;`.
Leave `reference.rs:283/307` and `ephemeris.rs:328-330` alone: they use the
motion.

- [ ] **Step 9: Switch the eclipse read**

In `crates/pleiades-eclipse/src/ephemeris.rs`, `read` (line 50) returns only
`(lon, lat, distance)`. Change `.position(&request(body, julian_day))` to
`.position_without_motion(&request(body, julian_day))` and add above it:

```rust
    // The eclipse geometry reads the place only (issue #128).
```

Then confirm nothing else in that crate reads `.motion` from a `read` result:
`grep -n "motion" crates/pleiades-eclipse/src/*.rs` must show no use tied to
`read`.

- [ ] **Step 10: Run the affected crates and both checksums**

Run: `cargo test -p pleiades-events -p pleiades-eclipse -p pleiades-core`
Expected: PASS, including `fu25_bit_identity` and
`chart::bit_identity_tests`.

- [ ] **Step 11: Commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-core -p pleiades-events -p pleiades-eclipse --all-targets --all-features -- -D warnings
git add crates/pleiades-core crates/pleiades-events crates/pleiades-eclipse
git commit -m "perf(core,events,eclipse): read mean places without motion where it is discarded (#128)"
```

---

### Task 5: Move the Meeus Sun into `pleiades-apparent`

**Files:**
- Modify: `crates/pleiades-apparent/src/aberration.rs`
  (+ `crates/pleiades-apparent/src/aberration/tests.rs`)
- Modify: `crates/pleiades-apparent/src/lib.rs:99`
- Delete: `crates/pleiades-events/src/solar.rs`
- Modify: `crates/pleiades-events/src/lib.rs:134`,
  `crates/pleiades-events/src/ephemeris.rs:6`,
  `crates/pleiades-events/src/fixstar.rs:6`

**Interfaces:**
- Produces: `pub fn pleiades_apparent::sun_true_longitude_of_date_deg(jd: f64) -> f64`,
  re-exported at the crate root. Task 6 consumes it.

- [ ] **Step 1: Write the failing test**

Append to `crates/pleiades-apparent/src/aberration/tests.rs`:

```rust
#[test]
fn meeus_sun_matches_example_25a() {
    // Meeus, Astronomical Algorithms, example 25.a: 1992 October 13.0 TD,
    // JD 2448908.5, true longitude ☉ = 199°.90988 (L0 + C).
    let lon = super::sun_true_longitude_of_date_deg(2_448_908.5);
    assert!((lon - 199.909_88).abs() < 1e-5, "{lon}");
}

#[test]
fn meeus_sun_is_normalized() {
    for jd in [2_305_447.5, 2_451_545.0, 2_524_593.5] {
        let lon = super::sun_true_longitude_of_date_deg(jd);
        assert!((0.0..360.0).contains(&lon), "{jd}: {lon}");
    }
}
```

If the tests file reaches items through `use super::*;` rather than
`super::`, match that style.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p pleiades-apparent --lib aberration`
Expected: compile error, `sun_true_longitude_of_date_deg` not found.

- [ ] **Step 3: Move the function**

Append to `crates/pleiades-apparent/src/aberration.rs`, above the tests
module declaration, the function from `crates/pleiades-events/src/solar.rs`.
Keep the arithmetic character for character, so the output stays
bit-identical:

```rust
/// Sun's geometric (true) ecliptic longitude of date, degrees, via the Meeus
/// low-precision solar theory (Astronomical Algorithms, ch. 25).
///
/// Computes the geometric mean longitude `L0`, the mean anomaly `M`, and the
/// equation of the center `C`, then returns the true longitude `L0 + C`.
/// Accuracy is about 0.01°, which is far more than enough for the argument of
/// [`annual_aberration`]: the offset is at most κ ≈ 20.5″ and its sensitivity
/// to the Sun's longitude is a fraction of that per degree. Being
/// backend-free, it costs nothing next to an ephemeris query, so callers that
/// need the Sun only as the aberration argument (the fixed-star path, and the
/// provenance-only aberration estimate of an apparent body place, issue #128)
/// use it instead of querying a backend. `jd` is the (TT/TDB) Julian Day of
/// date.
pub fn sun_true_longitude_of_date_deg(jd: f64) -> f64 {
    let t = (jd - 2_451_545.0) / 36_525.0; // Julian centuries since J2000.0
                                           // Geometric mean longitude of the Sun (Meeus 25.2).
    let l0 = 280.466_46 + 36_000.769_83 * t + 0.000_303_2 * t * t;
    // Mean anomaly of the Sun (Meeus 25.3).
    let m = (357.529_11 + 35_999.050_29 * t - 0.000_153_7 * t * t).to_radians();
    // Equation of the center (Meeus, ch. 25).
    let c = (1.914_602 - 0.004_817 * t - 0.000_014 * t * t) * m.sin()
        + (0.019_993 - 0.000_101 * t) * (2.0 * m).sin()
        + 0.000_289 * (3.0 * m).sin();
    (l0 + c).rem_euclid(360.0)
}
```

`J2000_JD` in events is `2_451_545.0` (`mean_elements.rs:12`), so inlining the
literal is bit-identical. Also update the module doc on line 2-3 of
`aberration.rs`: replace "this crate has no ephemeris of its own." with "this
crate has no ephemeris of its own, apart from the low-precision Meeus Sun
below, which is accurate enough only for the aberration argument."

In `crates/pleiades-apparent/src/lib.rs` line 99:

```rust
pub use aberration::{sun_true_longitude_of_date_deg, AberrationOffset};
```

- [ ] **Step 4: Repoint events and delete `solar.rs`**

```bash
git rm crates/pleiades-events/src/solar.rs
```

Remove `mod solar;` from `crates/pleiades-events/src/lib.rs:134`. In
`ephemeris.rs:6` and `fixstar.rs:6`, replace
`use crate::solar::sun_true_longitude_of_date_deg;` with
`use pleiades_apparent::sun_true_longitude_of_date_deg;` (in `ephemeris.rs`,
add it to the existing `pleiades_apparent::{…}` import list instead).
`fixstar.rs:79` mentions the function in a doc comment and still resolves.

- [ ] **Step 5: Run the tests and the events checksum**

Run: `cargo test -p pleiades-apparent && cargo test -p pleiades-events`
Expected: PASS, including `fu25_bit_identity` and the fixed-star tests.

- [ ] **Step 6: Commit**

```bash
cargo fmt --all
cargo clippy -p pleiades-apparent -p pleiades-events --all-targets --all-features -- -D warnings
git add -A crates/pleiades-apparent crates/pleiades-events
git commit -m "refactor(apparent,events): move the Meeus aberration Sun into pleiades-apparent (#128)"
```

---

### Task 6: The chart's aberration Sun comes from Meeus

**Files:**
- Modify: `crates/pleiades-core/src/chart/mod.rs:51` (imports), `:96-110`
  (`SunSample`, `offset_instant`), `:379-419`, `:627-642` (doc of
  `apparent_place`), `:769-794` (`query_sun_longitude_of_date`, deleted)
- Test: `crates/pleiades-core/src/chart/query_count_tests.rs`,
  `crates/pleiades-core/src/chart/apparent_tier_tests.rs:249-276`

**Interfaces:**
- Consumes: `pleiades_apparent::sun_true_longitude_of_date_deg` (Task 5).

- [ ] **Step 1: Write the failing query-count test**

Append to `query_count_tests.rs`:

```rust
#[test]
fn an_apparent_chart_reads_the_sun_only_as_a_body() {
    // Issue #128: the aberration argument comes from the backend-free Meeus
    // Sun, so a chart without the Sun never queries it, and a chart with the
    // Sun queries it exactly as it queries any other body.
    let queries = QueryLog::default();
    let backend = Recording {
        inner: composite(),
        queries: Arc::clone(&queries),
    };
    let engine = ChartEngine::new(backend);
    engine
        .chart(&apparent_request(vec![CelestialBody::Moon, CelestialBody::Mars]))
        .expect("apparent chart");
    let sun_reads = |log: &[(CelestialBody, f64, Entry)]| {
        log.iter()
            .filter(|(b, _, _)| *b == CelestialBody::Sun)
            .map(|(_, jd, _)| *jd)
            .collect::<Vec<_>>()
    };
    assert_eq!(sun_reads(&queries.lock().unwrap()), Vec::<f64>::new());

    queries.lock().unwrap().clear();
    engine
        .chart(&apparent_request(vec![CelestialBody::Sun, CelestialBody::Moon]))
        .expect("apparent chart");
    let mut reads = sun_reads(&queries.lock().unwrap());
    reads.sort_by(f64::total_cmp);
    // The batch at the chart instant, and the two speed-difference instants.
    assert_eq!(reads, vec![ISSUE_128_JD - 0.5, ISSUE_128_JD, ISSUE_128_JD + 0.5]);
}
```

- [ ] **Step 2: Rewrite the fail-closed test**

In `apparent_tier_tests.rs`, replace
`apparent_chart_fails_closed_when_backend_cannot_serve_the_sun` with:

```rust
#[test]
fn apparent_chart_needs_no_sun_from_the_backend() {
    // The Sun's longitude feeds only the provenance's aberration estimate,
    // which the backend-free Meeus Sun now serves (issue #128), so a backend
    // without a Sun can still serve an apparent chart.
    let engine = ChartEngine::new(MoonOnlyChartBackend);
    let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);

    let snapshot = engine
        .chart(&ChartRequest::new(instant).with_bodies(vec![CelestialBody::Moon]))
        .expect("an apparent chart needs no Sun source");
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Apparent);
    assert!(placement.apparent.is_some());

    // The explicit mean request still works and is reported as mean.
    let snapshot = engine
        .chart(
            &ChartRequest::new(instant)
                .with_bodies(vec![CelestialBody::Moon])
                .with_apparentness(Apparentness::Mean),
        )
        .expect("mean chart succeeds without a Sun source");
    assert_eq!(snapshot.apparentness, Apparentness::Mean);
    let placement = snapshot.placement_for(&CelestialBody::Moon).unwrap();
    assert_eq!(placement.position.apparent, Apparentness::Mean);
    assert!(placement.apparent.is_none());
}
```

Remove imports this leaves unused (for example `EphemerisErrorKind`) if
clippy flags them.

- [ ] **Step 3: Run them to verify they fail**

Run: `cargo test -p pleiades-core --lib chart::query_count_tests chart::apparent_tier_tests`
Expected: both new tests FAIL (Sun reads present; the Moon-only chart errors).

- [ ] **Step 4: Implement**

In `chart/mod.rs`, replace the block from `let sun_true_longitude_of_date = if
apparent_requested …` through the end of the `speed_suns` statement
(lines 387-419) with:

```rust
        // The Sun's true longitude of date is the argument of the annual-
        // aberration term, which feeds only the provenance's aberration
        // estimate: the light-time re-query already carries aberration (#93).
        // The backend-free Meeus Sun serves it, so an apparent chart neither
        // queries the backend for the Sun nor needs a backend that serves it
        // (issue #128). Positions do not depend on it.
        let sun_true_longitude_of_date = (apparent_requested && !request.bodies.is_empty())
            .then(|| sun_true_longitude_of_date_deg(request.instant.julian_day.days()));
        // The instants the apparent speed is differenced over, shared by every
        // body in the chart. A neighbour the backend cannot serve for a body
        // fails that body's correction sample; the speed then falls back to a
        // one-sided difference.
        let speed_suns = sun_true_longitude_of_date.map(|_| {
            [-HALF_SPAN_DAYS, 0.0, HALF_SPAN_DAYS].map(|offset_days| {
                let instant = offset_instant(request.instant, offset_days);
                Some(SunSample {
                    instant,
                    sun_lon: sun_true_longitude_of_date_deg(instant.julian_day.days()),
                })
            })
        });
```

Keep `speed_suns` as `[Option<SunSample>; 3]`. `apparent_motion` and its
one-sided fallback are unchanged. Delete `query_sun_longitude_of_date`
(lines 769-794). Add `sun_true_longitude_of_date_deg` to the
`pleiades_apparent::{…}` import at line 51. Remove
`precess_ecliptic_j2000_to_date` from it if nothing else in the file uses it
(`grep -n precess_ecliptic_j2000_to_date crates/pleiades-core/src/chart/mod.rs`).
If `backend_id` was used only by the deleted error message, remove whatever
clippy reports unused.

Update the `SunSample` doc (line 96) to "An instant of the apparent-speed
difference and the Meeus Sun's true longitude of date there."

- [ ] **Step 5: Update the provenance expectation in the reuse test**

`reusing_the_batch_read_leaves_the_apparent_place_bit_identical`
(`query_count_tests.rs`) builds its expected place with the backend Sun.
Positions are unaffected, but the provenance assertion now needs the Meeus
argument. Replace its `let sun = …; let sun_lon = …;` block with:

```rust
    let sun_lon = pleiades_apparent::sun_true_longitude_of_date_deg(ISSUE_128_JD);
```

Remove `precess_ecliptic_j2000_to_date` from that file's imports if it is now
unused.

- [ ] **Step 6: Run the whole core suite and the checksum**

Run: `cargo test -p pleiades-core`
Expected: PASS, including `chart::bit_identity_tests` with the **unchanged**
pinned value. Two failure modes need investigation, not a re-pin:
- **The checksum moved:** positions or speeds changed, so stop and find out
  why.
- **A test fails because it expected the old Sun-driven one-sided fallback or
  an exact provenance aberration value:** for example in
  `apparent_motion_tests.rs`, or in a test whose backend serves no Sun at a
  neighbour. Re-point the test so the *body* is missing at the neighbour, or
  compare provenance against a Meeus-argument expectation, and say so in the
  commit message.

- [ ] **Step 7: Commit**

The message carries the behaviour change for the changelog:

```bash
cargo fmt --all
cargo clippy -p pleiades-core --all-targets --all-features -- -D warnings
git add crates/pleiades-core
git commit -m "perf(core)!: take the chart's aberration Sun from the Meeus theory (#128)

The Sun's longitude feeds only the provenance's aberration estimate, so an
apparent chart no longer queries the backend for it at the chart instant and
at both speed-difference neighbours. Positions and speeds are bit-identical.

Behaviour change: provenance.aberration_longitude_arcsec moves by at most
about 0.01 arcsec, and a backend that cannot serve the Sun now returns
apparent placements instead of an error."
```

Check `git log --oneline -20` for whether this repo uses `!` for behaviour
changes that are not API breaks. If it doesn't (release-plz would read `!` as
breaking), drop the `!` and keep the body.

---

### Task 7: Measure, decide item 3, and record FU-25

**Files:**
- Create (throwaway, never committed): a cargo project under the session
  scratchpad, for example `$SCRATCH/fu25-probe/`
- Modify: `docs/follow-ups.md` (the FU-25 section, around line 2902)
- Modify: `spec/architecture.md` (one sentence, if it describes the backend
  trait's methods: `grep -n "positions\|EphemerisBackend" spec/architecture.md`)

- [ ] **Step 1: Build the probe**

`$SCRATCH/fu25-probe/Cargo.toml`:

```toml
[package]
name = "fu25-probe"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
pleiades-apparent = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-apparent" }
pleiades-backend = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-backend" }
pleiades-core = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-core" }
pleiades-elp = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-elp" }
pleiades-events = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-events" }
pleiades-types = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-types" }
pleiades-vsop87 = { path = "/workspace/.claude/worktrees/fu25-apparent-perf/crates/pleiades-vsop87" }

[profile.release]
debug = false
```

`$SCRATCH/fu25-probe/src/main.rs`:

```rust
use std::hint::black_box;
use std::time::Instant as Clock;

use pleiades_apparent::nutation::nutation;
use pleiades_apparent::precess_ecliptic_j2000_to_date;
use pleiades_backend::{Apparentness, CompositeBackend, EphemerisBackend, EphemerisRequest};
use pleiades_core::{ChartEngine, ChartRequest};
use pleiades_elp::ElpBackend;
use pleiades_events::{CrossingFrame, EventEngine};
use pleiades_types::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_vsop87::Vsop87Backend;

const START: f64 = 2_460_763.5; // 2025-03-29, issue #128
const N: usize = 400;
const STEP: f64 = 0.01;

fn composite() -> CompositeBackend<ElpBackend, Vsop87Backend> {
    CompositeBackend::new(ElpBackend::new(), Vsop87Backend::new())
}

fn instant(i: usize, scale: TimeScale) -> Instant {
    Instant::new(JulianDay::from_days(START + i as f64 * STEP), scale)
}

fn mean_us(mut f: impl FnMut(usize)) -> f64 {
    let t = Clock::now();
    for i in 0..N {
        f(i);
    }
    t.elapsed().as_secs_f64() * 1e6 / N as f64
}

const BODIES: [CelestialBody; 7] = [
    CelestialBody::Sun,
    CelestialBody::Moon,
    CelestialBody::Mercury,
    CelestialBody::Mars,
    CelestialBody::Saturn,
    CelestialBody::Pluto,
    CelestialBody::TrueNode,
];

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("loadavg: {}", load.trim());
    let backend = composite();
    let events = EventEngine::new(composite());
    let charts = ChartEngine::new(composite());

    println!("body       raw_us  motionfree_us  mean_us  apparent_us  position_at_us  chart1_us");
    for body in BODIES {
        let raw = mean_us(|i| {
            black_box(backend.position(&EphemerisRequest::new(body.clone(), instant(i, TimeScale::Tt))).unwrap());
        });
        let free = mean_us(|i| {
            black_box(backend.position_without_motion(&EphemerisRequest::new(body.clone(), instant(i, TimeScale::Tt))).unwrap());
        });
        let mean = mean_us(|i| {
            black_box(events.longitude_at(body.clone(), CrossingFrame::GeocentricMeanOfDate, instant(i, TimeScale::Tdb)).unwrap());
        });
        let apparent = mean_us(|i| {
            black_box(events.longitude_at(body.clone(), CrossingFrame::GeocentricApparentOfDate, instant(i, TimeScale::Tdb)).unwrap());
        });
        let position_at = mean_us(|i| {
            black_box(events.position_at(body.clone(), CrossingFrame::GeocentricApparentOfDate, instant(i, TimeScale::Tdb)).unwrap());
        });
        let chart1 = mean_us(|i| {
            let req = ChartRequest::new(instant(i, TimeScale::Tt))
                .with_bodies(vec![body.clone()])
                .with_apparentness(Apparentness::Apparent);
            black_box(charts.chart(&req).unwrap());
        });
        println!("{body:<10?} {raw:>7.1} {free:>14.1} {mean:>8.1} {apparent:>12.1} {position_at:>15.1} {chart1:>10.1}");
    }

    let eleven = vec![
        CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mercury, CelestialBody::Venus,
        CelestialBody::Mars, CelestialBody::Jupiter, CelestialBody::Saturn, CelestialBody::Uranus,
        CelestialBody::Neptune, CelestialBody::Pluto, CelestialBody::TrueNode,
    ];
    let chart11 = mean_us(|i| {
        let req = ChartRequest::new(instant(i, TimeScale::Tt))
            .with_bodies(eleven.clone())
            .with_apparentness(Apparentness::Apparent);
        black_box(charts.chart(&req).unwrap());
    });
    let raw11 = mean_us(|i| {
        for body in &eleven {
            black_box(backend.position(&EphemerisRequest::new(body.clone(), instant(i, TimeScale::Tt))).unwrap());
        }
    });
    let reduction = mean_us(|i| {
        let jd = START + i as f64 * STEP;
        black_box(precess_ecliptic_j2000_to_date(black_box(123.4), black_box(1.2), jd).unwrap());
        black_box(nutation(jd).unwrap());
    });
    println!("11-body chart: {chart11:.1} us; 11 raw queries: {raw11:.1} us");
    println!("precession+nutation per call: {reduction:.3} us");
    // An 11-body apparent chart reduces each body at three instants.
    let share = 11.0 * 3.0 * reduction / chart11 * 100.0;
    println!("precession+nutation share of the 11-body chart: {share:.2}%");
}
```

Adjust the names if a symbol is not exported under that path. Check with
`grep -n "pub use\|pub mod" crates/pleiades-*/src/lib.rs`. Run
`cargo fmt` inside the probe directory if desired. It is not committed.

- [ ] **Step 2: Wait for a quiet machine, then run A/B**

```bash
cat /proc/loadavg
```

Wait until the 1-minute load is below 3. If it stays above 3 for a long time,
proceed and report the load next to every number.

```bash
cd $SCRATCH/fu25-probe && cargo run --release --offline
```

For the baseline, add a second worktree of `main` at 91b13290d. Name it
`git worktree add $SCRATCH/fu25-base 91b13290d` from inside this worktree;
it lives in the scratchpad and is removed afterwards. Copy the probe as
`fu25-probe-base`, pointing at that tree. In the base copy, replace the
`motionfree_us` measurement with `position` (the method doesn't exist
there). Alternate base/new runs three times each and keep the median per cell.
Afterwards: `git worktree remove $SCRATCH/fu25-base`.

- [ ] **Step 3: Apply the item-3 decision rule**

If `precession+nutation share` is **≥ 5%**: stop. Write a spec addendum for
a `ReductionContext` and ask the user before implementing. Otherwise record
the share as the reason item 3 closes without change.

- [ ] **Step 4: Update FU-25**

In `docs/follow-ups.md`, change the FU-25 status to resolved (2026-10-04),
or to "partly resolved" if item 3 is deferred. Move the three "Remaining"
bullets into "Done" with one line each, and add the new table:

```markdown
**Done in round two (bit-identical positions and speeds):**

- `EphemerisBackend::position_without_motion`: VSOP87, ELP and the
  fictitious backend skip the ±0.5 d finite-difference speed. The light-time
  re-queries, the chart's speed-difference mean places and the events and
  eclipse mean-place reads use it.
- The chart's aberration-argument Sun comes from the backend-free Meeus Sun
  (now `pleiades_apparent::sun_true_longitude_of_date_deg`): no backend Sun
  query beyond the Sun's own placement. Provenance aberration estimates move
  by ≤ ~0.01″; a backend without a Sun can serve an apparent chart.
- Precession + nutation: measured at <share>% of an 11-body chart; no
  per-instant context needed.

Measured (2025-03-29, 400 samples, release build, load <L>; medians of three
alternated runs):

| body | raw | motion-free | meanOfDate | apparent | position_at | 1-body chart |
|---|---|---|---|---|---|---|
| … one row per body, before → after … |

11-body chart: <before> → <after> ms (11 raw queries: <raw> ms).
```

Fill every `<…>` and `…` with the measured numbers. Don't commit a
placeholder.

If `spec/architecture.md` lists the backend trait's methods, add after
`positions`: "`position_without_motion` — the place without motion, for
callers that discard it (light-time re-queries); a backend whose motion costs
extra evaluations overrides it."

- [ ] **Step 5: Commit**

```bash
git add docs/follow-ups.md spec/architecture.md
git commit -m "docs(follow-ups): FU-25 round two measurements and outcome (#128)"
```

---

### Task 8: Full validation

- [ ] **Step 1: Blocking CI locally**

Run: `mise run ci` (foreground; it includes fmt, clippy, tests, the crossings
golden check and `mise run docs`).
Expected: PASS. If the crossings golden differs, a task broke bit-identity.
Do **not** regenerate it; find the task that moved it.

- [ ] **Step 2: Apparent-place gates**

Run the gates that consume apparent places (find exact task names with
`mise tasks | grep -i "crossing\|station\|rise"`), for example
`mise run gate-crossings`, `mise run gate-stations`,
`mise run gate-rise-trans`. Use the names the repo actually defines.
Expected: every gate passes, with measured maxima identical to `main`'s
(positions are bit-identical).

- [ ] **Step 3: Report**

Summarize for the user: the commits, the before/after table, the item-3
decision with its number, the gate results, and the one behaviour change.
Then offer to open the PR. Pushing and merging follow the
`finish-branch-auto-merge` memory: never `gh pr merge --auto`. Wait for green
checks first.
