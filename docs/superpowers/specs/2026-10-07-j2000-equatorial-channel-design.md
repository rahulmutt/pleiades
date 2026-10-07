# J2000 equatorial channel and mean-chart right ascension

Date: 2026-10-07. Issue: #210. Status: design, awaiting review.

## Problem

Every first-party backend except `ElpBackend` fills
`EphemerisResult::equatorial` by rotating its J2000 `ecliptic` with the mean
obliquity **of date**:

```rust
ecliptic.to_equatorial(req.instant.mean_obliquity())
```

The result is neither J2000 right ascension and declination nor right
ascension and declination of date: it pairs the J2000 equinox with the
obliquity of date. Measured for VSOP87 Mars, mean request, against the J2000
ecliptic rotated by the J2000 obliquity (2026-10-06):

| Date | RA | Dec |
|---|---|---|
| 1900-01-01 | 5.4″ | −44.8″ |
| J2000 | 0 | 0 |
| 2100-01-01 | 8.5″ | −20.4″ |

The obliquity changes by about 47″ a century, so the error grows linearly from
J2000.

The call sites are:

- `crates/pleiades-vsop87/src/backend.rs` (`to_equatorial`)
- `crates/pleiades-data/src/backend.rs` (the main lookup and
  `derived_point_position`)
- `crates/pleiades-jpl/src/backend.rs` (`JplSnapshotBackend`,
  `SnapshotCorpusBackend`, and the reference-asteroid evidence builder)
- `crates/pleiades-jpl/src/spk/backend.rs` (`SpkBackend`)

`ElpBackend` gives right ascension and declination of date, from its of-date
place (#171). That frame is self-consistent, but it is not J2000: it differs
from J2000 by the precession, about 5000″ in right ascension a century out.

`ChartRequest::with_apparentness` documents a mean placement's equatorial
coordinates as "the backend's J2000 right ascension and declination in every
zodiac". The chart passes the backend's channel through unchanged
(`crates/pleiades-core/src/chart/mod.rs`, "keep the backend's mean-obliquity
equatorial"), so the promise is broken by up to about 45″ on the four
backends above. It is broken by degrees when ELP serves the body.

The parity self-checks in `pleiades-jpl` (`reference_summary/comparison.rs`,
`reference_snapshot/core/parity.rs`, `selected_asteroid.rs`,
`reference_asteroid.rs`) assert the same of-date rotation, so they repeat the
defect rather than test for it.

## Decisions

1. **Backends give J2000.** `Vsop87Backend`, `PackagedDataBackend`,
   `JplSnapshotBackend`, `SnapshotCorpusBackend` and `SpkBackend` rotate their
   J2000 ecliptic by `pleiades_types::OBLIQUITY_J2000_DEG`. Their equatorial
   channel is then the J2000 mean equator and equinox. For `SpkBackend`, ε₀
   inverts its own ICRF-to-ecliptic reduction (`spk/chain.rs`), so the
   channel is again the kernel's ICRF direction, to round-off.
2. **ELP stays of date.** `ElpBackend`'s channel remains right ascension and
   declination of date, as #171 documented and its Meeus evidence checks.
3. **Each backend states its frame.** The rustdoc of
   `EphemerisResult::equatorial` (`crates/pleiades-backend/src/result.rs`) and
   `spec/backend-trait.md` say the channel's frame is part of each backend's
   documented contract. The first-party backends give J2000, and ELP is the
   named exception.
4. **The chart owns the J2000 promise.** When the backend returned an
   equatorial channel, the chart replaces it with the backend's J2000
   ecliptic rotated by ε₀, computed **before** any sidereal or apparent
   rewrite of the ecliptic. That keeps `with_apparentness` true on every
   backend, ELP included, and in every zodiac. A backend that returns no
   equatorial channel still yields none.
5. **Apparent placements are unchanged.** When the apparent reduction
   succeeds, it overwrites the channel with right ascension and declination
   of date on the true obliquity, as today. When
   `apparent_equatorial_of_date` fails (nutation unavailable), the placement
   keeps the J2000 value from decision 4. Today it keeps whatever the
   backend gave.
6. Chosen over moving ELP to J2000 as well. That would revert #171 and its
   Meeus of-date evidence, for a channel the chart no longer depends on. It
   was also chosen over keeping the rotation and calling it a convenience:
   that leaves a documented promise false.

## What moves and what does not

- **Moves.** Away from J2000, the right ascension and declination of:
  - every mean placement, which is an `Apparentness::Mean` chart or the mean
    fallback inside an apparent chart. On the four backends this is up to
    about 45″ a century from J2000. On ELP-served bodies it is the
    precession, up to degrees.
  - a direct read of the four backends' `equatorial` channel.
- **Does not move.**
  - Ecliptic longitude, latitude, distance and speeds, in every chart.
  - Apparent placements.
  - House cusps and angles.
  - The Horizons and Swiss Ephemeris equatorial gates
    (`pleiades-validate/src/equatorial_validation.rs`), their goldens and
    checksums. They drive apparent charts and fail on a mean fallback, so
    they never read the changed channel.
  - The packaged artifact. It stores only ecliptic channels, and
    equatorial is a derived output rotated at lookup, so nothing is
    regenerated.
- **At J2000** `Instant::mean_obliquity()` equals `OBLIQUITY_J2000_DEG` bit
  for bit, so every assertion at JD 2451545.0 keeps its value.

## Design

### Backends

Replace `req.instant.mean_obliquity()` (or `instant.mean_obliquity()`) with
`Angle::from_degrees(OBLIQUITY_J2000_DEG)` at each call site listed under
Problem. The compression crate's `lookup_equatorial` already takes the
obliquity from its caller and needs only its doc line changed.

### Chart (`crates/pleiades-core/src/chart/mod.rs`)

Right after the backend result is taken, and before the sidereal mean rewrite
and the apparent reduction:

```rust
if position.equatorial.is_some() {
    if let Some(ecliptic) = position.ecliptic {
        position.equatorial =
            Some(ecliptic.to_equatorial(Angle::from_degrees(OBLIQUITY_J2000_DEG)));
    }
}
```

The existing apparent block still overwrites the channel on success. Replace
the comment "Mean-fallback rows … keep the backend's mean-obliquity
equatorial" with the rule from decisions 4 and 5. Update
`ChartRequest::with_apparentness` to say the J2000 value is computed by the
chart from the backend's J2000 ecliptic, whatever the backend's own channel
frame.

### Documentation and report strings

"Mean-obliquity transform" no longer says which obliquity. Reword these to
"J2000 mean obliquity" and update the tests that pin them in the same change:

- `crates/pleiades-validate/src/posture/backend_policy.rs`
  (`CURRENT_FRAME_POLICY_SUMMARY_TEXT`)
- `crates/pleiades-vsop87/src/profiles.rs`,
  `crates/pleiades-vsop87/src/source_docs/spec.rs`,
  `crates/pleiades-vsop87/src/source_docs/request_corpus.rs` (doc)
- `crates/pleiades-jpl/src/reference_summary/jpl_posture.rs`
- `crates/pleiades-validate/src/posture/jpl/holdout.rs`,
  `crates/pleiades-validate/src/posture/jpl/reference_asteroid.rs`
- `crates/pleiades-data/src/lib.rs`, `crates/pleiades-data/src/lookup.rs`,
  `crates/pleiades-data/src/backend.rs`
- `crates/pleiades-compression/src/format.rs`,
  `crates/pleiades-compression/src/artifact.rs`
- `crates/pleiades-types/src/time.rs` (the `mean_obliquity` doc no longer
  claims the equatorial transforms)
- `docs/time-observer-policy.md`

The mean-obliquity round-trip evidence (`pleiades-validate/src/render/text/
evidence.rs`) stays as it is. It checks a rotation and its inverse, which
holds at any obliquity, and it makes no frame claim.

The ELP-only prose and the historical design docs stay as they are.
`docs/superpowers/specs/2026-10-06-sidereal-mean-chart-frame-design.md`'s
claim ("backend's J2000 RA/Dec") becomes true and needs no edit.

### Release bookkeeping

Add a release note to `crates/pleiades-core/src/compatibility/mod.rs`, bump
the compatibility profile id, and re-measure its content checksum, following
the #171 entry. Release-plz picks the version bumps up from the commit's
`fix` type.

## Testing

Every new assertion is against a reference independent of the code under test.

- **SPK.** At 1900, J2000 and 2100, the `equatorial` channel equals the
  direction of the kernel's ICRF vector, which is read before any of the
  crate's rotations, within 1e-9″.
- **VSOP87, packaged data, JPL snapshot and corpus.** At an off-J2000 epoch:
  - the channel equals the J2000 ecliptic rotated by ε₀.
  - For VSOP87 Mars at 1900-01-01, the channel minus the old of-date rotation
    is within 0.1″ of the issue's measurement (RA 5.4″, Dec −44.8″). That
    pins the sign, the epoch and the size.
- **Chart.** In `crates/pleiades-core/src/chart/tests.rs`, with a test
  backend whose `equatorial` channel is deliberately of date, as ELP's is:
  - a mean chart at 1900 gives the J2000 ecliptic rotated by ε₀, not the
    backend's channel.
  - a sidereal mean chart's right ascension and declination equal the
    tropical mean chart's.
  - the mean fallback inside an apparent chart gives the same J2000 value.
  - an apparent chart whose reduction fails keeps the J2000 value. This one
    is covered only if a test backend can reach that path without new
    production hooks; otherwise it is recorded in the plan as untested.
  - a backend with no equatorial channel still yields none.
- **Existing tests.**
  - The unit tests that assert `to_equatorial(instant.mean_obliquity())` move
    to ε₀:
    - `pleiades-vsop87/src/tests/backend.rs`
    - `pleiades-data/src/tests/lookup.rs`
    - `pleiades-jpl/src/backend/tests.rs`
    - `pleiades-jpl/src/reference_summary/{holdout,reference_asteroid,selected_asteroid}/tests.rs`
  - The four parity self-checks move to ε₀ as well.
  - The ELP tests pinning its of-date channel are unchanged.
- **Gates.**
  - `mise run ci`, including release-smoke, whose equatorial gates must pass
    unchanged.
  - `mise run test-full` before the PR, for `pleiades-validate`'s slow tier.

## Out of scope

- Moving `ElpBackend`'s channel to J2000.
- A J2000 or of-date choice offered to the caller of a chart.
- `spec/data-compression.md` describes the stored channels as
  "ecliptic-of-date Cartesian", which looks stale against the J2000 storage
  boundary. That is filed as its own issue.
