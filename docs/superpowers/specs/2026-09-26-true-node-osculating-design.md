# Osculating true lunar node for `TrueNode` (issue #58) — design

**Status:** design approved · **Opened:** 2026-09-26 · **Issue:** #58 ·
**Crates:** `pleiades-data`, `pleiades-core`, `pleiades-validate`, `pleiades-elp`
(docs only) · **Tool:** `tools/se-true-node-reference` (new)

## Context

`CelestialBody::TrueNode` is served today by `ElpBackend` as Meeus's
*periodic-term-corrected mean node* (Astronomical Algorithms Ch. 47: the mean
node polynomial plus five sine terms in D, M, M′, F). Swiss Ephemeris'
`SE_TRUE_NODE` is instead the *osculating* ascending node, formed from the
Moon's instantaneous position and velocity. Issue #58 measured the two against
each other in the same frame across 2026 and found a wander of −0.137° to
+0.141° (roughly ±8′), while `MeanNode` agrees to 0.00000°. The ELP evidence
rows cannot detect this: the 1913 sample is Meeus's own worked example, so the
approximation validates itself.

The issue leaves the repo a design call: serve `TrueNode` from an osculating
path, or keep the approximation with a documented ceiling. The decision
(2026-09-26) is the osculating path, with the Meeus channel retained as the
ELP backend's documented lower-tier fallback.

### What already exists

- **True Lilith precedent (FU-2, resolved 2026-06-30).** `PackagedDataBackend`
  serves `TrueApogee`/`TruePerigee` release-grade from the packaged Moon state
  via `pleiades-apsides`, gated by `validate-lilith` against a committed Swiss
  Ephemeris `SE_OSCU_APOG` corpus (3177 rows, 1900–2100). The chart layer
  treats these as geometric directions: precession + nutation-in-longitude
  only, no light-time, no aberration (`apparent_apsis_position`).
- **SP-4 osculating machinery.** `pleiades-apsides` exposes
  `elements_from_state` and `points_from_elements`, which return the ascending
  and descending node points. `pleiades-events::nod_aps` forms the lunar
  osculating ellipse from the packaged Moon state rotated into the of-date
  plane (precession + Δψ) and matches the 8 Moon `SE_NODBIT_OSCU` rows of the
  committed nod-aps corpus on the node points to ≤18″ (the apse residual on
  the same rows is ~3400″, which is why the shared `OSCU_MOON` ceiling is
  loose). Those 8 rows are, for the Moon, Swiss Ephemeris' `SE_TRUE_NODE`
  evaluated with nutation on.
- **Frame convention after issue #57.** Every ELP point channel is an of-date
  quantity precessed back to J2000 at the backend boundary; consumers apply
  the forward J2000→date precession exactly once.

## Goal and scope

Serve `CelestialBody::TrueNode` release-grade from `PackagedDataBackend` as the
osculating ascending node of the geocentric lunar orbit, at Swiss Ephemeris
parity measured and gated by a new committed reference corpus, without
changing the backend boundary contract, the routing chain, or any public type.

In scope:

1. Osculating-node path in `pleiades-data` beside the existing apsis path.
2. Chart-layer apparent treatment of `TrueNode` as a geometric direction.
3. Reference generator `tools/se-true-node-reference`, committed corpus
   `crates/pleiades-validate/data/true-node-corpus/`, and the fail-closed
   `validate-true-node` gate wired into the release gate set.
4. A permanent regression test for #58 in `pleiades-data`, pinned to the 8
   Swiss Ephemeris Moon osculating-node rows already committed in the nod-aps
   corpus, independent of the new dense corpus.
5. Documentation: ELP fallback ceiling, lunar theory policy, follow-ups
   entry, README validation table, compatibility summary + profile bump.

Out of scope:

- Changing `ElpBackend`'s `TrueNode` numerics or claim tier. It stays the
  constrained-tier Meeus channel for direct ELP consumers and is documented as
  such with the measured ±0.14° envelope.
- A descending-node body (no such `CelestialBody` variant exists; not needed).
- Gating node *speed* against Swiss Ephemeris (the Lilith gate does not gate
  speed either; motion stays a central difference, see §2).
- Modelling the nutation-in-obliquity tilt of the node separately. Whatever
  residual it contributes is measured by the gate, not modelled.
- Any change to `pleiades-apsides`. Its existing helpers suffice.

## 1. Semantics and frame

At the backend boundary, `TrueNode` is the ascending node of the osculating
Keplerian ellipse fitted to the Moon's geocentric position and velocity,
**formed in the mean ecliptic of date**, and then **precessed back to J2000**
(the same `precess_ecliptic_date_to_j2000` step the ELP point channels use
since #57). Expressed in J2000 the node carries a small non-zero latitude (the
tilt between the two ecliptics, ≈±0.003° in 2026) which the consumer's forward
precession removes exactly.

Why form the node in the date plane rather than form it in J2000 and rotate
the point: a node is the intersection of the orbit plane with the *reference*
plane, so tilting the reference plane by δ moves a low-inclination node by
≈δ/sin(i). For the Moon (i ≈ 5.1°) the J2000-vs-date tilt would misplace the
node by up to ~0.04°, the same class of error the issue reports. The events
engine documents and avoids exactly this; the packaged path does the same.

Why the *mean* ecliptic of date, not the true: the packaged backend is a
mean-only backend, `MeanNode` is a mean-equinox-of-date quantity, and the
chart layer adds nutation in longitude (Δψ) uniformly for apparent charts.
Adding Δψ to a node formed in the mean plane is identical to forming the node
in a plane rotated by Δψ about the ecliptic pole, and the SP-4 engine (which
does precisely precession + Δψ before forming the ellipse) already matches
Swiss Ephemeris' nutation-on node to ≤18″. So the mean-plane node plus the
existing chart-layer Δψ reproduces `SE_TRUE_NODE` (default flags) in apparent
charts, and the raw boundary value precessed forward reproduces `SE_TRUE_NODE`
with `SEFLG_NONUT`, which is the quantity the issue measured against.

Time scale: the backend accepts TT or TDB; the Julian day of the request is
used directly as the precession epoch, as the ELP and apsis paths do.

## 2. Backend: `pleiades-data`

`PackagedDataBackend` gains an osculating-node path next to
`osculating_apsis_position`:

1. Look up the packaged Moon ecliptic and motion at the normalized lookup
   instant, exactly as the apsis path does, and build the J2000 Cartesian
   state (`SphericalState` → `spherical_state_to_cartesian`).
2. Rotate position **and** velocity J2000 → mean ecliptic of date with
   `pleiades_apparent::precess_ecliptic_j2000_to_date` applied to each
   vector's (lon, lat) with its magnitude preserved. Precession is a rotation,
   so applying it to the velocity vector as a direction is valid; this is what
   the events engine does. (The differencing itself happens in the packaged
   motion channel, i.e. in J2000, so no spurious frame-rotation term enters
   the velocity.)
3. `elements_from_state(pos, vel, MU_EARTH_MOON_AU3_PER_DAY2)` then
   `points_from_elements(&elements, false)`; take `.ascending`.
4. Precess the ascending point back to J2000 with
   `precess_ecliptic_date_to_j2000`, keeping its distance. That is the
   boundary `EclipticCoordinates`.
5. Equatorial: from the J2000 ecliptic with the mean obliquity, as the apsis
   path does. Motion: central difference over ±0.5 day of step 1–4, with the
   same out-of-range fallback to `Motion::new(None, None, None)` as the apsis
   motion. Quality: `Interpolated`. Backend id: the packaged id.

Errors mirror the apsis path: missing Moon distance → `InvalidRequest`;
`ApsidesError` (including `DegenerateNode`) → `InvalidRequest` with a message
naming the osculating node; artifact lookup errors map through
`map_artifact_error` unchanged.

Surface changes, all additive:

- `supports_body` returns true for `TrueNode`.
- `position` dispatches `TrueNode` to the new path before the artifact lookup.
- A new public `true_node_body_claims()` returns one release-grade claim:
  `AccuracyClass::High`, `ClaimEvidence::CorpusValidated { source: "Swiss
  Ephemeris 2.10.03 SE_TRUE_NODE (validate-true-node)" }`. `metadata()` extends
  its claims with it beside `apsis_body_claims()`. The existing
  `apsis_body_claims()` is not renamed (public API, and its name stays true).
- `Cargo.toml` adds `pleiades-apparent` (workspace dep). `pleiades-apparent`
  depends only on `pleiades-types` and `pleiades-time`, and `pleiades-elp`
  already depends on it, so the layering precedent exists and there is no
  cycle.

Routing needs no change: the CLI chart chain already lists
`PackagedDataBackend` first, ahead of the composite that contains
`ElpBackend`, so a chart request for `TrueNode` now resolves to the packaged
osculating node, exactly as `Moon` already resolves to packaged data ahead of
ELP.

The precession-rotate helper is small and stays private to `pleiades-data`;
deduplicating it with the events engine's `rotate_j2000_to_true_of_date`
(which also adds Δψ) is not worth a shared module for two callers with
different frames.

## 3. Chart layer: `pleiades-core`

In the apparent-place branch for release-grade bodies, the match arm that
routes `TrueApogee | TruePerigee` through `apparent_apsis_position`
(precession + nutation only, no light-time re-query, no annual aberration)
extends to `TrueNode`. Nothing else changes: `TrueNode` becomes release-grade
through the packaged claim, so it enters this branch automatically, and the
mean-fallback behaviour on error is inherited.

## 4. Validation

### 4.1 Reference generator: `tools/se-true-node-reference`

A sibling of `tools/se-lilith-reference` (same `Cargo.toml` shape:
`swisseph = "0.1.1"`, `libswisseph-sys = "0.1.2"`, `publish = false`,
`version = "0.0.0"`, outside the workspace). It calls
`swe_calc(jd_tt, SE_TRUE_NODE = 11, SEFLG_MOSEPH)` on the identical
deterministic grid (JD_TT 2415020.5 to 2488070.0, step 23 days, 3177 rows)
and prints the same CSV shape: comment header naming the SE version, body,
flags and frame (true ecliptic of date, nutation on), then
`jd_tt,lon_deg,lat_deg,dist_au` rows. Nutation stays on so the gate
reproduces the chart path, as the Lilith gate does.

**Build environment:** the tool needs `libclang` and `LIBCLANG_PATH` to build
the Swiss Ephemeris bindings. Per `AGENTS.md`, that native library lives in
`devenv.nix`, which already declares `pkgs.clang`, `pkgs.libclang` and
`env.LIBCLANG_PATH` (verified 2026-09-26: `devenv shell` provides clang 21
with `LIBCLANG_PATH` set). The corpus is therefore generated on the feature
branch as an ordinary plan task:

```bash
devenv shell -- cargo run --release \
  --manifest-path tools/se-true-node-reference/Cargo.toml \
  > crates/pleiades-validate/data/true-node-corpus/true-node.csv
```

followed by writing the manifest line. Neither the gate nor the workspace
needs the tool afterwards; the gate reads the committed CSV via
`include_str!`, so the CSV and manifest are committed with the gate. The
FU-2 follow-up note's "requires `libclang-dev`" wording should be updated to
point at `devenv.nix` as the sanctioned way to get it.

### 4.2 Corpus: `crates/pleiades-validate/data/true-node-corpus/`

- `true-node.csv`: generator output.
- `manifest.txt`: one line in the Lilith format,
  `slice true-node file=true-node.csv role=true-node rows=<n> checksum=<fnv1a64>`.

### 4.3 Gate: `validate-true-node`

`crates/pleiades-validate/src/true_node_validation.rs`, a structural mirror of
`lilith_validation.rs`:

- Parses the CSV (fail on any malformed row), verifies the fnv1a64 checksum
  and row count against the manifest (checksum or row drift is a failure).
- For every row: `PackagedDataBackend` `TrueNode` at `Instant(jd_tt, Tt)` →
  `apparent_apsis_position` → compare of-date longitude (wrap-aware,
  arcsec), latitude (arcsec) and distance (relative) against the row.
- Ceilings are constants set to `ceil(measured max × 1.5)` from the first
  run over the committed corpus, with the measured maxima recorded in the
  source comment, following the Lilith convention. Expected class:
  arcsecond-level cross-theory floor (Moshier reference vs packaged DE440
  state; the 8-row bootstrap suggests ≲20″ longitude). Placeholder ceilings
  must not be merged; the plan sequences generation before ceiling-setting.
- Public surface: `validate_true_node_corpus()`, `TrueNodeCorpusError`,
  `TrueNodeCorpusReport` (with `summary_line()`), re-exported from the crate
  root like the Lilith items.
- Wiring: `run_all_numeric_gates` (so `release-gate` runs it); validate CLI
  subcommand `validate-true-node` with alias `true-node-gate`, plus the same
  dispatch line in `pleiades-cli`'s `cli.rs`; help text; a nightly-tier test
  `true_node_gate_passes_within_ceilings` that also prints measured maxima.

### 4.4 Bootstrap regression test for #58 (blocking tier)

`crates/pleiades-data/src/tests/lookup.rs` gains a test that embeds the 8
Moon `method=2` rows from `nod-aps-corpus/nod-aps.csv` (jd_tt, asc_lon,
asc_lat, asc_dist, as committed; the source row identity cited in a comment)
and asserts the packaged `TrueNode`, taken through `apparent_apsis_position`,
matches each within a ceiling of 40″ in longitude, 10″ in latitude, and
1e-3 relative in distance. These rows are Swiss Ephemeris 2.10.03 Moshier
with nutation on and `SEFLG_NOGDEFL` (deflection is not applied to node
points, so the flag is immaterial). The ELP Meeus value at J2000 sits 0.027°
(~97″) from the corresponding row, so the test discriminates the two models.
`pleiades-data` cannot depend on `pleiades-validate`, hence the literal
embedding rather than a shared reader.

### 4.5 Unit tests (blocking tier)

In `pleiades-data`, mirroring the existing `TrueApogee` tests:

- `supports_body(TrueNode)` is true; `position` succeeds inside the window
  with ecliptic, equatorial and motion populated and `Interpolated` quality.
- The release-grade `TrueNode` claim is present with `CorpusValidated`
  evidence naming the gate.
- Forward-precessing the boundary ecliptic to the request date restores
  latitude to ≈0 (the node lies in the mean ecliptic of date by
  construction).
- The osculating node stays within 2.5° of the ELP `MeanNode` at a few
  instants (the Meeus periodic terms sum to ≈2°), guarding against a frame or
  sign slip that the tight rows in §4.4 would report less legibly.
- Out-of-window instants fail with `OutOfRangeInstant` like the apsis path.

In `pleiades-core`, a chart test with the packaged backend asserts an
apparent-mode chart marks `TrueNode` `Apparent` and that its longitude equals
the mean boundary value precessed forward plus Δψ (i.e. no aberration or
light-time crept in), mirroring whatever the existing `TrueApogee` chart
coverage does.

## 5. ELP fallback

No numerics or claim changes in `pleiades-elp`. Documentation states the
truth:

- `backend.rs`: the `elp_body_claims` and `true_node_longitude` doc comments
  say the channel is Meeus's periodic-term-corrected mean node, not the
  osculating node; that it differs from Swiss Ephemeris `SE_TRUE_NODE` by up
  to ±0.14° (issue #58, measured across 2026); and that the routed chart path
  serves the osculating node from `PackagedDataBackend`, so this channel is
  reached only by direct ELP consumers; a routed chart that requests
  `TrueNode` outside the packaged 1900–2100 window fails with
  `OutOfRangeInstant` (the router does not fall back on that error kind), as
  the Moon itself already does.
- `docs/lunar-theory-policy.md`: a note beside the existing True Lilith note
  saying the same for the true node.

## 6. Docs and bookkeeping

- `docs/follow-ups.md`: new entry `FU-12: Osculating true lunar node (issue
  #58)` recording the design, the measured gate parity once known, and the
  ELP fallback envelope as documented-not-gated.
- `README.md` validation table: a row for the true (osculating) node,
  `validate-true-node`, with its measured accuracy class.
- `crates/pleiades-core/src/compatibility/mod.rs`: append an
  "SP-4-FU (osculating true node, issue #58)" sentence to the compatibility
  summary in the established pattern (what ships, gate name and aliases,
  corpus size and flags, measured maxima and ceilings, the ELP fallback
  caveat), bump `CURRENT_COMPATIBILITY_PROFILE_ID` to `0.7.14`, and add the
  matching entry in the additions list; API stability profile unchanged
  (currently `0.3.0`; the change is purely additive: no new types, one new
  public fn in `pleiades-data`).
- `docs/threat-model.md`: unchanged. The corpus is untrusted-input-shaped but
  parsed by the same fail-closed reader pattern as the existing corpora, and
  no trust boundary moves.
- Changelog: release-plz generates it from conventional commits; no manual
  edit.

## Error handling summary

| Condition | Behaviour |
|---|---|
| Instant outside packaged window | `OutOfRangeInstant`, as for the Moon |
| Moon distance missing in artifact | `InvalidRequest`, message names the osculating node |
| `elements_from_state` degenerate/unbound/non-finite | `InvalidRequest`, message names the osculating node |
| Central-difference neighbour out of window | motion `None` channels, position still served |
| Corpus malformed / checksum / row-count drift | gate error, fail-closed |
| Row residual over ceiling | gate error naming jd, metric, got/want/residual/ceiling |

## Success criteria

1. The 8-row bootstrap test passes in the blocking tier at ≤40″ longitude.
2. `validate-true-node` passes over the committed dense corpus with ceilings
   set from measured maxima, and `release-gate` includes it.
3. A chart for any 2026 instant reports `TrueNode` from the packaged backend
   within the gate's ceiling of `SE_TRUE_NODE`, replacing the ±0.14° wander.
4. `MeanNode`, `TrueApogee`, `TruePerigee` and all existing gates are
   unchanged (existing tests and `mise run ci` green).
5. ELP's `TrueNode` output is bit-identical to before; only its docs change.
