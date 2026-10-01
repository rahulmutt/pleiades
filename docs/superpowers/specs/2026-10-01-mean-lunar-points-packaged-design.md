# Mean lunar points on the packaged backend (issue #90) — design

**Status:** implemented (2026-10-01) ·
**Opened:** 2026-10-01 · **Issue:** #90 ·
**Crates:** `pleiades-apsides`, `pleiades-events`, `pleiades-data`,
`pleiades-elp`, `pleiades-core`, `pleiades-validate`, `pleiades-cli` ·
**Tool:** `tools/se-mean-lunar-reference` (new)

## Context

Every `pleiades-events` doc example builds `EventEngine::new(packaged_backend())`.
On that backend `nod_aps(Moon, NodApsMethod::Mean, ..)` fails at run time:

```
backend error: UnsupportedBody: no packed data exists for MeanNode
```

`mean_points_at` (`pleiades-events` `src/nod_aps.rs`) reads `MeanNode` and
`MeanPerigee` from the backend, and `PackagedDataBackend` serves neither. It
serves `TrueNode`, `TrueApogee` and `TruePerigee` as derived points. Only
`ElpBackend` serves the mean points, so the failure is invisible in the routed
chart chain and in `validate-nod-aps`, which both include ELP.

Decisions taken on 2026-10-01:

1. Serve the mean points from the packaged backend **release-grade**, behind a
   new Swiss Ephemeris corpus gate (not `constrained`, not documentation only).
2. Single-source the mean lunar elements in **`pleiades-apsides`**; no
   backend-to-backend dependency and no duplicated polynomials.
3. Accept that charted `MeanApogee`/`MeanPerigee` move to the Swiss Ephemeris
   definition (see "Consumer-visible changes").

### What already exists

- **Polynomials.** `ElpBackend::mean_node_longitude` and
  `mean_perigee_longitude` (`pleiades-elp` `src/backend.rs`) are private Meeus
  Ch. 47 polynomials in days from J2000 TT, referred to the mean equinox of
  date. `mean_apogee_longitude` is perigee + 180°. ELP emits each as the point
  `(longitude, latitude 0, no distance)` precessed to the J2000 boundary, under
  a `constrained` claim.
- **Mean lunar orbit constants.** `MOON_MEAN_INCL_DEG`, `MOON_MEAN_ECC`,
  `MOON_MEAN_SEMA_AU` are `pub(crate)` in `pleiades-events`
  `src/mean_elements.rs`.
- **Point geometry.** `pleiades_apsides::points_from_elements` turns
  `KeplerianElements` into node and apsis points with longitude, latitude and
  distance. `nod_aps` uses it for the Moon's mean points and
  `validate-nod-aps` measures the result against Swiss Ephemeris at 0.561″
  longitude (ceiling 0.8″), 0.06″ latitude ceiling, 2.0e-6 relative distance
  ceiling, over 1900–2100.
- **Derived-point path.** `PackagedDataBackend::derived_point_position` builds
  an `EphemerisResult` (ecliptic, mean-obliquity equatorial, central-difference
  motion) from a closure returning J2000 ecliptic coordinates. `TrueNode` is
  formed in the mean ecliptic of date and precessed back to J2000.
- **Chart layer.** `pleiades-core` `src/chart/mod.rs` routes
  `TrueApogee | TruePerigee | TrueNode` through `apparent_apsis_position`
  (precession + nutation in longitude only) and exempts the same three from
  the topocentric correction. Every other body, including the mean lunar
  points, takes the general light-time path.
- **Gate precedent.** `validate-true-node`: `tools/se-true-node-reference`
  (Moshier, nutation on, 23-day grid over 1900–2100, 3177 rows), committed CSV
  plus manifest with an fnv1a64 checksum and pinned row count,
  `crates/pleiades-validate/src/true_node_validation.rs`, wired into
  `run_all_numeric_gates` and both CLIs.

### Finding: the Meeus apsis element is not the Swiss Ephemeris point

The Meeus mean longitude of perigee is an orbital element: the node longitude
plus the argument of perigee measured in the orbit plane. Swiss Ephemeris'
`SE_MEAN_APOG` is the apsis placed on the inclined mean orbit and projected on
the ecliptic. From the committed `nod-aps.csv` Moon mean row at JD 2451545.0:

| | Longitude | Latitude |
|---|---|---|
| Swiss Ephemeris mean apogee | 263.4643° | +3.4197° |
| Meeus perigee + 180° (ELP's channel) | 263.3532° | 0 |

The gap is about 6.7′ in longitude and up to the mean inclination (5.145°) in
latitude. A release-grade `MeanApogee` therefore cannot be the ELP channel
served as-is; it must be the projected point. The mean node lies on the
ecliptic under both definitions and is unaffected.

It follows that the backend's `MeanPerigee` channel cannot stay the input of
`nod_aps`: `nod_aps` needs the element, and the packaged backend will now
emit the projected point.

## Design

### 1. `pleiades-apsides` owns the mean lunar orbit

New public items, in a `mean` submodule re-exported from the crate root:

- `mean_lunar_node_longitude_of_date(jd_tt: f64) -> f64` and
  `mean_lunar_perigee_longitude_of_date(jd_tt: f64) -> f64`: the two Meeus
  polynomials moved verbatim from `pleiades-elp`, degrees in `[0, 360)`, mean
  equinox of date.
- `MOON_MEAN_INCLINATION_DEG`, `MOON_MEAN_ECCENTRICITY`,
  `MOON_MEAN_SEMI_MAJOR_AU`: moved from `pleiades-events` with their values
  unchanged.
- `mean_lunar_elements_of_date(jd_tt: f64) -> KeplerianElements`: the five
  elements above assembled for `points_from_elements`.

The functions are total over finite input; no new error type. The crate keeps
zero dependencies. Its `Cargo.toml` description and README widen from
"osculating lunar apsides" to "lunar orbit points: mean and osculating nodes
and apsides".

### 2. `pleiades-events`: `nod_aps` Mean for the Moon stops reading the backend

`mean_points_at` builds the Moon's elements from
`mean_lunar_elements_of_date(jd)`, the same way planets use the built-in mean
element table. `read_mean_longitude_of_date` and the three `MOON_MEAN_*`
constants are removed from `pleiades-events`.

This fixes the reported failure on every backend. It also removes a
J2000-and-back precession round trip, so `validate-nod-aps` `MEAN_MOON`
residuals may move at floating-point scale; the 0.8″ ceiling stays and the
measured maximum is re-recorded.

### 3. `pleiades-data` serves `MeanNode`, `MeanApogee`, `MeanPerigee`

A new `mean_lunar_point_ecliptic(body, instant)` evaluates
`mean_lunar_elements_of_date`, takes the ascending node, apogee or perigee
from `points_from_elements` (aphelion convention), and precesses the of-date
point to the J2000 boundary with `precess_ecliptic_date_to_j2000`, as
`osculating_node_ecliptic` does. It is served through
`derived_point_position`, so the result carries ecliptic, equatorial and
central-difference motion.

- `supports_body` and `position` accept the three bodies.
- The points do not read the artifact, but they are served only inside the
  packaged window: an instant outside it returns `OutOfRangeInstant`, like
  every other packaged body. A backend that answered outside its declared
  range for some bodies would break the range metadata.
- Distance: the corpus shows `SE_MEAN_NODE` distance is the constant mean
  distance `a` (0.002569555290 AU) and `SE_MEAN_APOG` is `a(1+e)`
  (0.002710625132 AU). The backend's `MeanNode` therefore reports
  `MOON_MEAN_SEMI_MAJOR_AU`, not the `points_from_elements` orbit radius at the
  node; apogee and perigee report the `points_from_elements` distances.
- New public `mean_lunar_point_body_claims()` returns three
  `BodyClaim::release_grade(.., AccuracyClass::High, CorpusValidated { source:
  "Swiss Ephemeris 2.10.03 SE_MEAN_NODE / SE_MEAN_APOG
  (validate-mean-lunar-points)" })` claims, added to `metadata().body_claims`
  next to the apsis and true-node claims.

`pleiades-data` already depends on `pleiades-apsides` and `pleiades-apparent`;
no new dependency.

### 4. `pleiades-elp` delegates and documents

`pleiades-elp` takes a dependency on `pleiades-apsides` and its three mean
functions delegate to it. Behaviour, claims and evidence rows are unchanged.
Its `MeanApogee`/`MeanPerigee` channels remain the raw element with latitude 0;
the rustdoc, README and specification note gain an honesty caveat (up to about
7′ in longitude and 5.1° in latitude from `SE_MEAN_APOG`), matching the caveat
already carried by its `TrueNode` channel.

Layering: `pleiades-apsides` is a dependency-free domain crate, so a backend
crate depending on it follows `spec/architecture.md`.

### 5. `pleiades-core`: mean lunar points are geometric directions

The two `matches!` lists in `src/chart/mod.rs` (apparent-place arm and
topocentric exemption) are replaced by one predicate covering all six lunar
points (`CelestialBodyClass::LunarPoint`). Mean points then get precession +
nutation in longitude only: no light-time re-query, no aberration, no diurnal
parallax. This is what Swiss Ephemeris does, and it is required now that the
packaged mean points carry a distance, which would otherwise send them down
the light-time path and through the topocentric parallax shift (the defect
fixed for True Lilith under #58). The exemption is documented at the predicate,
which closes issue #63.

### 6. Gate `validate-mean-lunar-points`

- `tools/se-mean-lunar-reference`: `swe_calc` with `SE_MEAN_NODE` (10) and
  `SE_MEAN_APOG` (12), `SEFLG_MOSEPH`, nutation on, on the true-node grid
  (JD 2415020.5 to 2488070.0, 23-day step, 3177 rows). Columns: `jd_tt`, node
  longitude/latitude/distance, apogee longitude/latitude/distance. Built in
  `devenv shell`, like the other `se-*-reference` tools; not needed to run the
  gate.
- Corpus at `crates/pleiades-validate/data/mean-lunar-corpus/` with a manifest
  (row count, fnv1a64 checksum).
- `mean_lunar_validation.rs` mirrors `true_node_validation.rs`: fail-closed on
  checksum, row count, a validated-row floor, and per-channel ceilings. It
  queries `PackagedDataBackend` and applies precession + nutation in longitude
  through the same `pleiades-apparent` call the chart uses.
- Channels: node longitude, latitude, distance; apogee longitude, latitude,
  distance. `MeanPerigee` is gated on the same rows as apogee longitude + 180°,
  negated latitude, and distance `a(1−e)` derived from the corpus apogee
  distance and the published eccentricity; Swiss Ephemeris has no separate
  mean-perigee body.
- Ceilings: measured maximum plus margin, recorded beside each constant.
  Expectation from `validate-nod-aps`: longitude under 1″, latitude under 0.1″,
  distance under 1e-5 relative. A measured longitude maximum above 5″ on any
  channel stops the work for a design revision rather than a looser ceiling.
- Wired into `run_all_numeric_gates`, `pleiades-validate` CLI
  (`validate-mean-lunar-points`, alias `mean-lunar-points-gate`), the help
  text, and the `pleiades-cli` passthrough.

### 7. Tests

- `pleiades-apsides`: polynomial values at J2000 and at the Meeus worked dates
  (1913-05-27 node 0°, 1959-12-07 node 180°, 2021-03-05 perigee 224.89194°);
  elements assembly; a property test that the functions stay in `[0, 360)`.
- `pleiades-events`: regression test `nod_aps(Moon, Mean)` on
  `packaged_backend()` at JD 2460000.5 TDB (the issue's reproduction), added
  red first; a test that the Moon mean path needs no lunar-point channel from
  the backend.
- `pleiades-data`: `supports_body` for the three points; position at J2000
  against the `nod-aps.csv` row above; perigee/apogee antipodal relation;
  out-of-window instant returns `OutOfRangeInstant`; motion sign and magnitude
  (node about −0.0530°/day, apogee about +0.111°/day).
- `pleiades-elp`: existing evidence tests pass unchanged; one test that ELP's
  and the packaged backend's `MeanNode` agree at fp scale (lives in
  `pleiades-validate` or `pleiades-events` tests, which already depend on
  both).
- `pleiades-core`: a mean chart point is unchanged by a topocentric request; a
  charted `MeanApogee` has the Swiss Ephemeris latitude sign and magnitude.
- Gate tests following `true_node_validation.rs`: tampered checksum and row
  count fail closed.

### 8. Documentation and release posture

- Compatibility profile: new entry, profile id 0.7.15 → 0.7.16.
- READMEs: `pleiades-apsides`, `pleiades-data`, `pleiades-elp`; `nod_aps`
  rustdoc drops the ELP-channel wording.
- `docs/follow-ups.md`: note the resolution; no new FU unless a ceiling or the
  distance convention forces one.
- Any committed golden or release-bundle fixture that renders body claims or
  the capability matrix is regenerated in the same PR.
- Commit type `feat`; versions are left to release-plz.

## Consumer-visible changes

- `packaged_backend()` serves three more bodies; `nod_aps(Moon, Mean)` works on
  any backend.
- In the routed chart chain the packaged backend is first, so charts asking for
  `MeanApogee`/`MeanPerigee` move from ELP's element to the Swiss Ephemeris
  point: up to about 7′ in longitude, a non-zero latitude, and a distance.
- Charted mean points no longer receive the general light-time treatment or a
  topocentric shift. For `MeanNode` this is expected to be far below 0.01″
  geocentric (it had no distance); the implementation measures and records it.
- Apparent charts previously returned the ELP-served (constrained-tier) mean
  lunar points un-rotated in the J2000 mean ecliptic, because the chart applies
  apparent-place only to release-grade bodies; they are now in the true
  ecliptic of date (`MeanNode` 341.806020° → 342.170759° at JD 2461041.5, about
  +0.36°). The light-time removal itself moved the packaged points by at most
  0.0063″.
- In a routed chain with `PackagedDataBackend` first, the mean lunar points are
  now served only inside the packaged window (1900–2100); a chart requesting
  them outside it returns an out-of-range error instead of the `ElpBackend`
  element it previously fell back to (the same fail-closed behaviour as
  `TrueNode` and the packaged planets).
- Direct `ElpBackend` consumers see no change.

## Out of scope

- Extending any lunar-point gate beyond 1900–2100.
- Changing ELP's mean apsis channel to the projected point.
- Interpolated ("natural") apogee/perigee, and issues #84–#89.

## Risks

- **Distance convention of `SE_MEAN_NODE`** — resolved: the corpus shows the
  constant mean distance `a` (section 3).
- **Reference tool build** needs `libclang` from `devenv shell`; if the
  environment cannot build it, the gate cannot be produced and the work stops
  there rather than shipping an ungated release-grade claim.
- **Fixture churn:** body-claim and matrix renderings appear in several
  release-bundle tests; expect coordinated fixture updates.
