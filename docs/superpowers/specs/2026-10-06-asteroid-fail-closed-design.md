# Asteroid positions fail closed away from their sample rows

Date: 2026-10-06. Issue: #158 (widened). Status: design, awaiting review.

## Problem

The offline backends return asteroid positions that are wrong on almost every
date, without an error.

Measured with `pleiades-cli chart --mean` against the JPL `sb441-n373s` rows
in `crates/pleiades-jpl/data/corpus/asteroid_reference.csv` (geocentric
ecliptic J2000, 180-day cadence, 407 rows per body over 1900–2100):

| JD | Date | Body | pleiades | Truth | Error |
|---|---|---|---|---|---|
| 2451380.5 | 1999-07-21 | Eros | 317.18° | 92.22° | 135° |
| 2451560.5 | 2000-01-17 | Eros | 216.55° | 247.49° | 31° |
| 2451740.5 | 2000-07-15 | Eros | 37.05° | 297.45° | 100° |
| 2451380.5 | 1999-07-21 | Ceres | 311.46° | 119.47° | 168° |
| 2451740.5 | 2000-07-15 | Vesta | 51.72° | 294.92° | 117° |

Over all 407 Eros rows the mean absolute longitude error is 76.2° (a uniform
random longitude would average 90°). 375 rows are off by more than 10° and 3
are within 1°. The figures come from a CLI binary built on 2026-10-01; no
commit since then touches asteroid data. The first test of this change
re-measures at HEAD.

Issue #158 records the symptom as non-physical asteroid *velocities*. The
positions are wrong for the same reason.

### Cause

Both paths rest on `crates/pleiades-jpl/data/reference_snapshot.csv`. It holds
17 rows for each of Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros`, and 10
for `asteroid:99942-Apophis`:

- JD 2378498.5 (1800), 2451545.0 (J2000), 2453000.5 (2003), 2500000.0 (2132),
  2634167.0 (2500);
- a cluster from 2451910.5 to 2451919.5 (January 2001), rows 0.5 to 1 day
  apart (up to 2 days for Apophis).

`JplSnapshotBackend` answers a request that is not exactly on a row with a
Lagrange cubic through the four rows nearest in time
(`interpolate_fixture_state`). It has no limit on how far those rows are from
the instant, and it does not require them to lie on both sides of it. A Ceres
request at JD 2451800 is extrapolated 110 days from four cluster rows. The
result is labelled `QualityAnnotation::Interpolated`.

`PackagedDataBackend` serves Eros from segments that the artifact generator
fitted to that same cubic, sampled between the same rows
(`body_segments_from_entries`). It declares Eros `BodyClaimTier::ReleaseGrade`
with `AccuracyClass::High`. Its only accuracy check compares the artifact with
the rows it was fitted to.

`JplSnapshotBackend` is described as a validation fixture and is excluded from
release claims, but it is third in the CLI's default chart chain
(`default_chart_backend`), which is how Ceres, Pallas, Juno, Vesta and Apophis
reach users. `stations` and `aspects` use the same chain.

### Placeholder Apophis rows

Four of the ten Apophis rows are not positions:

| JD | x (km) | y (km) | z (km) |
|---|---|---|---|
| 2451915.0 | 459.2 | 1.3766e8 | −6.0 |
| 2453000.5 | 418.6 | 1.2549e8 | 5.6 |
| 2500000.0 | 466.1 | 1.3974e8 | 3.7 |
| 2634167.0 | 457.3 | 1.3710e8 | 2.7 |

Each reads as longitude 89.9998°, latitude 0.000°. The six other rows are
ordinary vectors (232° to 290°, about 1.1° a day across the cluster). The
backend serves the four as `QualityAnnotation::Exact`. The rows at 2500000.0
and 2634167.0 are repeated in `independent_holdout_snapshot.csv`. Their origin
is not known; the 2451915.0 row arrived with commit `a77437d1d`.

## Decision

No published backend returns an asteroid position it cannot support. Away
from real sample rows the caller gets a typed error that says why and names
`SpkBackend`, which already serves these bodies correctly from a JPL kernel.

Correct asteroid positions offline, without a kernel, are a separate project
(see "Follow-up").

After this change the default chain serves an asteroid at its exact sample
epochs and inside the January 2001 cluster, and returns an error elsewhere.

## Design

### 1. Stencil guard in `JplSnapshotBackend` (`pleiades-jpl`)

`JplSnapshotBackend::position` keeps answering an exact row with
`QualityAnnotation::Exact`. For any other instant *t* it takes the stencil the
interpolation would use (the four rows nearest in time; three if the body has
only three) and answers only if both hold:

- the stencil has a row before *t* and a row after it;
- the stencil's last epoch minus its first is at most
  `MAX_STENCIL_SPAN_DAYS`, a named constant set to 5.0.

Otherwise it returns `EphemerisErrorKind::OutOfRangeInstant`. The message
names the body, the instant, the nearest row on each side, and
`pleiades_jpl::SpkBackend` as the way to serve the body at that instant.

The value 5.0 rests on the pinned leave-one-out report
(`interpolation_quality_sample_list`): where the held-out row has stencil rows
on both sides and the stencil is at most 4 days wide, the largest longitude
error is 0.0000013° (0.005″) across Ceres, Vesta and Eros. The next bracket up
is 366 days. Pallas and Juno have not been read yet; the implementation
confirms the value against the full report before fixing it, and the
constant's doc comment records the measurement.

The rule applies to every body the backend holds, planets included. No
default-chain request for a planet reaches this backend.

`OutOfRangeInstant` is reused and no error variant is added: the instant is
outside what the backend's rows cover for that body, which is what the kind
means, and `RoutingBackend` already treats it as final. A new variant would
change a public enum in `pleiades-backend` for no gain to callers.

The guard lives in `position`. The internal resolver
(`resolve_fixture_state_from_entries`) and `SnapshotCorpusBackend` stay
unguarded, because two internal callers need the unguarded cubic:

- the leave-one-out transparency report, whose purpose is to show how bad
  wide interpolation is;
- the artifact generator (see section 3).

### 2. Remove the placeholder Apophis rows

Delete the four rows from `reference_snapshot.csv` and the two copies from
`independent_holdout_snapshot.csv`, and correct the coverage comments in both
headers. Update the pinned sample counts and report lines that follow from
the row count.

Add a test over both files that rejects a row whose `x` and `z` are both
under 1000 km, so a placeholder cannot return.

Replacing the rows with real Horizons vectors needs a network fetch and new
provenance. That goes to the follow-up.

### 3. `PackagedDataBackend` stops serving Eros (`pleiades-data`)

- `supports_body` returns false for `asteroid:433-Eros`, and `position`
  returns `EphemerisErrorKind::UnsupportedBody` for it. `RoutingBackend`
  skips a backend that does not support a body, so Eros reaches the guarded
  snapshot backend like the other asteroids.
- Eros leaves `packaged_body_claims` and the backend's supported-body
  metadata. The release posture goes from 17 release-grade bodies to 16.
- The committed artifact (`tests/fixtures/packaged-artifact.bin`), its
  checksum and `packaged_bodies()` do not change. The artifact still carries
  the Eros segments; the backend does not serve them. The crate docs say so,
  and name the follow-up issue as where they are removed or replaced.
- The distinction between bodies the artifact carries and bodies the backend
  serves gets one named predicate in `pleiades-data`, used by
  `supports_body`, `position`, the claims and the metadata.

The artifact generator (`regenerate.rs`) and the fit-envelope check
(`FitTruthBackend` in `coverage/threshold.rs`) sample Eros between rows
through `JplSnapshotBackend.position` today. They switch to a
`SnapshotCorpusBackend` over the same reference-snapshot rows, which runs the
same interpolation code without the guard. The generated segments must not
change. No existing test proves that: the kernel-free regeneration path
returns the committed bytes. So the change starts with a new test that pins
the generator's Eros segments to those in the committed artifact, written and
passing before the generator is touched.

The validation-report benchmark corpus (`packaged_artifact_corpus`) stops
requesting Eros from the packaged backend.

### 4. Consumers

- `pleiades-events/tests/nod_aps.rs`: `snapshot_only_asteroids_fail_closed`
  and `packaged_fit_asteroids_fail_closed` now receive
  `EventError::Backend` carrying `OutOfRangeInstant`, not
  `DegenerateNodAps`. They still fail closed. The file's header comment and
  the test comments are rewritten to match.
- `pleiades-cli` `chart_command_routes_selected_asteroids_via_jpl_fallback`
  (Ceres at J2000) still renders. The apparent reduction's light-time
  re-query lands 0.013 day off the row and is refused, so the placement takes
  the chart's existing fallback to the mean place. The test gains an
  assertion on that.
- The `never_stations` comment in `pleiades-events/src/stations.rs` stays
  true: asteroids are still scanned.

### 5. Tests

**Served or refused.** For each of Ceres, Pallas, Juno, Vesta and Eros, and
each of the body's 407 rows in `asteroid_reference.csv`, a mean geocentric
request must either return an error or agree with the row within 5″ in
longitude (scaled by cos latitude) and in latitude. Two instances:

- in `pleiades-jpl`, against `JplSnapshotBackend`;
- in `pleiades-cli`, against `default_chart_backend`, which also proves Eros
  routes past the packaged backend.

Both run in the blocking tier. Each pins the outcome so it cannot pass
vacuously: per body, exactly one epoch is served (JD 2453000.5, the one epoch
a sample row and a truth row share) and 406 are refused. Written first, the
test fails on current `main`.

The 5″ tolerance is set by the two sources. At JD 2453000.5 the Horizons row
and the `sb441-n373s` row differ by up to 1.0″ in longitude (Pallas) and 1.9″
in latitude (Juno, Vesta, Ceres, Eros). The difference has the pattern of a
small rotation about the equinox axis and is not explained here; it is
recorded in the follow-up issue.

Apophis has no row in `asteroid_reference.csv`, so this gate does not cover
it. Its six remaining rows are covered by the placeholder test and the unit
tests below.

**Unit tests in `pleiades-jpl`:**

- a request between J2000 and the cluster is refused for every asteroid, with
  `OutOfRangeInstant` and a message naming `SpkBackend`;
- a request inside the cluster interpolates and matches a held-out row within
  0.05″;
- a request 0.1 day before the cluster's first row and 0.1 day after its last
  is refused (all four nearest rows lie on one side);
- a request 0.013 day from the J2000 row is refused;
- the existing beyond-the-last-row case still returns its existing message.

**Unit tests in `pleiades-data`:** the backend does not support Eros, returns
`UnsupportedBody` for it, and claims 16 release-grade bodies with Eros absent.
The Eros self-consistency test keeps reading the artifact directly.

### 6. Documentation and claims

- `docs/status.md`: the per-backend grades line names Pluto and the Moon, not
  Eros, and a known-limits entry states the offline asteroid rule.
- `crates/pleiades-data/README.md` and crate docs: Eros is carried by the
  artifact and not served.
- `crates/pleiades-jpl/README.md` and `JplSnapshotBackend` rustdoc: the
  stencil rule and the error.
- `spec/data-compression.md`: the Eros ceiling row and its note.
- `docs/cli.md`: what `--body Ceres` and `--body asteroid:433-Eros` return
  away from the sample epochs, and the `SpkBackend` pointer.
- The compatibility release note in `pleiades-core` and its pinned checksum.
- `crates/pleiades-events/README.md`: stations and aspects of asteroids
  return a backend error outside the sample windows.

### 7. Tracker

- File a follow-up issue: correct asteroid positions offline across
  1900–2100. Scope: dense fits from `sb441-n373s` in the packaged artifact,
  real Apophis rows, removal of the dead Eros segments, and the 1.9″
  Horizons-versus-`sb441` latitude difference.
- Comment on #158 with the position measurements, noting that this change is
  its "decline" option and the follow-up is its "densify" option.
- Comment on #167 (d) and #168 (f) pointing at the follow-up.

## Out of scope

- Dense asteroid data and any regeneration of the packaged artifact.
- `JplSnapshotBackend`'s backend-level `AccuracyClass::Exact` and its
  `nominal_range`, beyond what the rustdoc now says about the rule.
- `SpkBackend` and the Tier A asteroid roster.
- A CLI option to load a kernel.

## Risks

- **Pinned report strings.** Removing Apophis rows and dropping the Eros
  claim change sample counts and summary lines pinned across `pleiades-jpl`,
  `pleiades-data`, `pleiades-validate` and `pleiades-cli` tests. Each is
  updated to the new measured value, never loosened.
- **Artifact byte identity.** The generator must produce the same Eros
  segments through `SnapshotCorpusBackend`. The new pinning test covers this
  locally. `tests/artifact_regen.rs` needs the de440 kernel and may not run
  here; if it cannot, the PR says so.
- **Behaviour change.** A chart, station or aspect request for an asteroid
  that returned a value now returns an error on most dates. Those values were
  wrong; the changelog entry states the change plainly.

## Release effect

`fix:` commits to `pleiades-jpl` and `pleiades-data`. release-plz gives both
a patch release and re-releases their exact-pin dependents.

## Verification

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo nextest run` for `pleiades-jpl`, `pleiades-data`, `pleiades-events`,
  `pleiades-core`, `pleiades-cli` and `pleiades-validate`
- `mise run docs`
- `mise run ci` before the PR is opened
