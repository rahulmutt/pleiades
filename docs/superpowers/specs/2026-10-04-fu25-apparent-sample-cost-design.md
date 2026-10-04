# FU-25: cost of an apparent-of-date sample, remaining work (issue #128)

**Status:** design approved in conversation 2026-10-04 · follow-up FU-25 in
`docs/follow-ups.md` · first round shipped in #134.

## Intent

Issue #128 measured an apparent-of-date sample on the VSOP87/ELP composite at
4–20× the raw backend query, and an 11-body chart at 12× its raw queries. #134
removed duplicate reads (Moon apparent 263 µs → 30 µs, 11-body chart 60.6 ms →
31–35 ms). FU-25 lists three remaining multipliers. This round removes them
where that is possible **without moving any position or speed bit**.

Success:

- Every light-time re-query and every mean-place-only sample on VSOP87, ELP
  and the fictitious backend costs one geocentric evaluation instead of three.
- An apparent chart makes one backend Sun read (its batch), not four.
- Precession/nutation sharing is decided by measurement, not assumed.
- Positions and speeds are bit-identical before and after. The crossings
  golden stays byte-identical and is not regenerated. All gates stay green.
- #128's table is re-measured and recorded in FU-25.

Out of scope (rejected under the bit-identical policy): analytic VSOP87
derivatives, and extrapolating the retarded place from the backend's own
velocity. Both move outputs. A field on `EphemerisRequest` is also out:
`EphemerisRequest` is a pub-field struct without `#[non_exhaustive]`, so adding
a field is a breaking change.

## 1. Backend contract: `position_without_motion`

`pleiades-backend/src/traits.rs` gains a provided method on `EphemerisBackend`:

```rust
/// Like [`position`](Self::position) for a caller that does not need
/// `motion`: the returned place is bit-identical to `position`'s and
/// `motion` is `None`. A backend that derives motion from extra
/// evaluations overrides this to skip them. It may succeed where
/// `position` fails only because the motion could not be computed.
fn position_without_motion(
    &self,
    req: &EphemerisRequest,
) -> Result<EphemerisResult, EphemerisError> {
    let mut result = self.position(req)?;
    result.motion = None;
    Ok(result)
}
```

The method is additive, so every existing implementation compiles unchanged
and the API-stability profile does not move.

**Forwarders.** `CompositeBackend` and `RoutingBackend` forward it through the
same primary/fallback (respectively routing) logic as `position`. That logic
moves into one private helper per wrapper, parameterised by the call to make,
so `position` and `position_without_motion` cannot drift apart.

**Overrides.** `Vsop87Backend`, `ElpBackend` and `FictitiousBackend<S>`. In
each, the body of `position` becomes a private `compute(req, with_motion:
bool)`. `position` calls `compute(req, true)` and the override calls
`compute(req, false)`. With `with_motion == false` the finite-difference
`motion(..)` call is skipped. Nothing else changes.

`FictitiousBackend<S>` also reads its Sun source through
`S::position_without_motion`, because it uses only the Sun's place. This
speeds up its own `position` too, and leaves its output bit-identical.

**Left on the default.** `PackagedDataBackend` (motion is a table lookup),
`JplSnapshotBackend`, `SnapshotCorpusBackend` and `SpkBackend` (no motion), and
test backends.

**Edge-case widening.** Fict's motion fails only when both ±0.5 d probes fail.
In that case `position_without_motion` returns the place while `position`
returns an error. That is the only behaviour difference, and it follows the
documented contract.

## 2. Callers that switch

Every one of these callers ignores `motion` today.

| Crate | Call site | Used for |
|---|---|---|
| `pleiades-core` | `ChartEngine::query_mean_ecliptic` (`chart/mod.rs`) | light-time re-queries, speed-difference mean place, Sun/lunar-point geocentric reads |
| `pleiades-events` | `read_mean_ecliptic` (`ephemeris.rs`) | light-time re-queries, crossing/bisection/mean samples |
| `pleiades-eclipse` | `read` (`ephemeris.rs`) | Sun/Moon mean places for the eclipse geometry |

These stay on `position`: the chart's position batch (`positions`, whose motion
the chart reports) and `read_mean_ecliptic_with_motion` in events. The plan
greps every other `.position(` call in events and eclipse and switches only
those that never read `.motion`.

## 3. The chart's aberration-estimate Sun

The Sun's true longitude of date feeds only the provenance's
`aberration_longitude_arcsec`: the light-time re-query already carries
aberration (#93). #134 gave `pleiades-events` a backend-free Meeus Sun for that
purpose. The chart still queries the backend for it at the chart instant and
at both speed-difference neighbours.

- `sun_true_longitude_of_date_deg` moves verbatim from
  `pleiades-events/src/solar.rs` into `pleiades-apparent` as a public function
  beside `annual_aberration`, its only consumer. `solar.rs` is deleted. Events
  and its fixed-star path import the function from `pleiades-apparent`. The
  code is identical, so events output is bit-identical.
- In the chart, `sun_true_longitude_of_date` and `speed_suns` come from the
  Meeus Sun. `query_sun_longitude_of_date` and its fail-closed error branch are
  deleted. Every `SunSample` neighbour is now always present. Whether a
  neighbour is usable is decided by the body's own `correction_sample`, whose
  failure already triggers `apparent_motion`'s one-sided fallback.
- The Sun's own placement keeps reading the backend Sun from the batch, so an
  apparent chart makes exactly one backend Sun read.

**Visible behaviour change** (stated in the commit message, which release-plz
turns into the changelog):

- `provenance.aberration_longitude_arcsec` shifts by at most about 0.01″. The
  Meeus Sun is good to about 0.01°, and the aberration term's sensitivity to
  the Sun's longitude is at most κ/cos β (κ ≈ 20.5″) per radian.
- A chart on a backend that cannot serve the Sun now returns apparent
  placements instead of an error.
  `apparent_chart_fails_closed_when_backend_cannot_serve_the_sun`
  (`chart/apparent_tier_tests.rs`) is rewritten to assert success, with
  placements equal to the same chart computed on a backend that does serve the
  Sun.
- Positions and speeds are bit-identical.

## 4. Precession/nutation context: measure first

Precession is a few trigonometric calls and nutation an IAU 1980 series of
about 106 terms, recomputed per body per sample. Next to VSOP87 evaluations of
about 0.4 ms that cost is expected to be negligible. A throwaway probe settles
it:

- The probe is a standalone cargo project in the scratchpad, with path deps,
  an empty `[workspace]`, `--offline` and a release build. It is never
  committed.
- It times `precess_ecliptic_j2000_to_date` + `nutation` per call, a VSOP87
  `position_without_motion`, and an 11-body apparent chart on the composite
  after sections 1–3.
- It runs only once `/proc/loadavg` is below 3. A/B runs alternate, and the
  load is reported with the numbers.

**Decision rule.** If precession plus nutation across a chart's bodies is ≥ 5%
of the 11-body chart time, a per-instant `ReductionContext` in
`pleiades-apparent` caches those results. It reuses the same functions, so
outputs stay bit-identical. It is specified in an addendum to this spec before
it is implemented. Otherwise FU-25 records the measured numbers as the reason
the item is closed without change.

## 5. Testing and validation

- **Override equivalence:** for VSOP87, ELP and fict, across several bodies
  and epochs (including fict at a Sun-source window edge),
  `position_without_motion` returns an `EphemerisResult` equal to `position`'s
  with `motion` set to `None`.
- **Forwarding:** `CompositeBackend` and `RoutingBackend` route
  `position_without_motion` to the same backend as `position`, including the
  fallback-to-secondary path.
- **Query counts:**
  - Extend `chart/query_count_tests.rs` and the events reference tests with a
    counting backend that counts `position` and `position_without_motion`
    separately, per body.
  - Pinned for an apparent chart: one Sun read (the batch); every light-time
    and speed-difference query uses the motion-free path.
  - Pinned for events: `read_mean_ecliptic` never calls `position`.
- **Bit-identity:**
  - A test pins a checksum (`fnv1a64`) over chart placements (longitude,
    latitude, distance, speeds) and over events' `longitude_at`/`position_at`
    values on the VSOP87/ELP composite.
  - The checksum is captured on `main` before any change and asserted after.
    Provenance is excluded because its aberration estimate moves by design.
- **CI:** `mise run ci`, which includes the crossings golden check
  (byte-identical, no regeneration) and `mise run docs`.
- **Gates:** before merge, the apparent-place gates (crossings, stations,
  rise-trans) are run locally.
- **Measurement:** #128's table is re-measured with the same probe shape
  (2025-03-29, 400 samples 0.01 d apart, release build, load reported). The
  table covers body × raw / meanOfDate / apparent / `position_at` / one-body
  chart, plus the 11-body chart.

## 6. Docs

- `docs/follow-ups.md` FU-25: the done items, the measurements and the item-3
  outcome. Mark FU-25 resolved, or narrow it to whatever remains.
- The trait's rustdoc carries the contract above. `spec/architecture.md` /
  `spec/api-and-ergonomics.md` get one sentence where the backend trait
  surface is described, if such a sentence exists to extend.
- Issue #128: a comment with the new table when the PR lands.
