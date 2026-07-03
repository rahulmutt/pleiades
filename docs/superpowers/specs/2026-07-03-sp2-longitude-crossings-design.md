# SP-2a · Longitude Crossings — Design

Status: **draft — 2026-07-03**. Design proposed and reviewed in brainstorming;
awaiting written-spec review before handing to writing-plans.

This is the first slice of **SP-2**, the second sub-project in the engine-layer
arc opened by SP-1 (angles & sidereal time, landed 2026-07-01). SP-2 closes the
event-finding gaps where Swiss Ephemeris *ships a function*:

- **SP-2a (this spec):** longitude crossings — `swe_solcross`, `swe_mooncross`,
  general geocentric body crossings, and heliocentric `swe_helio_cross`.
- **SP-2b (later spec):** rise/set/meridian transit + horizontal coordinates
  (`swe_rise_trans`, `swe_azalt`).
- **SP-2c (later spec):** local eclipse circumstances (observer-local visibility
  and contact timing).

SP-2b and SP-2c are out of scope here; they get their own spec → plan → gate
cycles. They are expected to share the `pleiades-events` crate and the generic
root-finder this slice introduces.

## Context

pleiades is used as an **astronomy engine only**: it supplies
positions/houses/ayanamsas/eclipses/angles, and the application builds the
technique layer on top. Crossings are an engine-level quantity ("when does body
B reach ecliptic longitude λ?"), not an astrological technique.

Current state (verified in code):

- `pleiades-eclipse` already root-finds an ephemeris quantity over time: its
  private `syzygy.rs` brackets the Sun−Moon elongation by half-day stepping and
  bisects to a 0.5 s tolerance (`crates/pleiades-eclipse/src/syzygy.rs`). It also
  owns the apparent-longitude helpers `sample_sun_moon` and
  `apparent_sun_longitude_deg` (`crates/pleiades-eclipse/src/ephemeris.rs`) that
  correctly apply light-time/aberration once for the geocentric Sun (per FU-1).
- The eclipse **engine** exposes the exact API shape this slice mirrors:
  `eclipses_in_range(start, end, filter)`, `next_eclipse(after, filter)`,
  `previous_eclipse(before, filter)`, generic over `B: EphemerisBackend`, with a
  hard 1900–2100 TDB coverage-window clamp (`WINDOW_START_JD`/`WINDOW_END_JD`).
- The apparent-place pipeline (`pleiades-apparent`) already produces geocentric
  apparent of-date ecliptic longitude — the convention SE's `solcross`/
  `mooncross` use.
- Backend positions carry `distance_au`
  (`crates/pleiades-types/src/coordinates.rs`), so full geocentric vectors are
  available for the heliocentric reconstruction that `helio_cross` needs.
- The repo has a firm, repeated shape for a new engine feature: a domain crate
  over a backend → re-export through `pleiades-core` + a CLI subcommand → an
  isolated out-of-workspace `tools/se-*-reference` harness → a committed corpus →
  a fail-closed `validate-*` gate wired into `release-smoke`/`release-gate` →
  claim-tier/compatibility-profile bookkeeping. SP-2a follows it exactly.

So SP-2a is **a generalized root-finder + a small public API + a parity gate**,
reusing conventions and correction code that already exist.

## Decisions captured

| Topic | Decision |
| --- | --- |
| SE functions targeted | `swe_solcross`/`swe_solcross_ut`, `swe_mooncross`/`swe_mooncross_ut`, general geocentric crossing (the same primitive over any supported body), and `swe_helio_cross` |
| Home crate | **New `pleiades-events` crate**, generic over `B: EphemerisBackend`, mirroring `pleiades-eclipse`/`pleiades-apsides`. `crossings` is its first module; SP-2b/SP-2c join it later. |
| Root-finder | A small private generic `root` module in `pleiades-events`: bracket-by-stepping over a body-scaled step + bisect on an arbitrary `f(t)` to a sub-second tolerance. The eclipse crate is **left untouched** (its passing `validate-eclipses` gate must not be perturbed); a later cleanup slice may migrate it onto the shared root-finder. |
| Longitude convention | **Geocentric apparent of-date tropical** for `solcross`/`mooncross`/general bodies (SE default), computed via the existing `pleiades-apparent` pipeline; Sun light-time handled once via the eclipse crate's `apparent_sun_longitude_deg` (FU-1 fix). **Heliocentric** for `helio_cross`. |
| Heliocentric reconstruction | `P_helio = P_geo − S_geo` from the backend's geocentric planet and Sun vectors (both carry `distance_au`); longitude taken from the reconstructed vector. Exact apparent/geometric convention pinned to SE parity during implementation. `helio_cross` applies to the planets (Mercury–Pluto), not Sun/Moon. |
| Direction & multiplicity | `next`/`previous`/`in_range`. Sun and Moon are monotonic in longitude (one crossing per period); **general bodies may cross a target up to three times per synodic loop when retrograde** — the step-scan bracket enumerates all crossings in range. |
| Time base | Native **TDB** results (matching the eclipse engine). SE `_ut` corpus rows converted once at generation. Callers wanting civil/UT1 use the existing `pleiades-time` conversion. No change to the ΔT/UT1 policy. |
| Coverage window | Same hard **1900–2100 TDB** clamp the eclipse engine uses (packaged backend has no segments outside it). Out-of-window requests fail closed. |
| Surfacing | Re-export `CrossingEngine`, `Crossing`, `CrossingFrame`, `Direction` through `pleiades-core`; add a `crossings` CLI subcommand mirroring the eclipse render. |
| Validation | New isolated `tools/se-crossings-reference`; committed `crossings-corpus` + manifest; fail-closed **`validate-crossings`** gate (per-body time-residual ceilings) wired into `release-smoke`/`release-gate`; claim-tier ↔ evidence carried through the overclaim audit. |
| Versioning | New public surface → bump the compatibility profile (from `0.7.4`) and note the crossings API in README "current state". No breaking change to existing types, so the API-stability profile (`0.2.1`) is unaffected unless a shared type is touched. |

## Scope & boundaries

**In:**

- A public longitude-crossing API over `B: EphemerisBackend`: `next`, `previous`,
  and `in_range` for a given body + target ecliptic longitude.
- `solcross` (body = Sun) and `mooncross` (body = Moon) as the geocentric
  apparent-of-date special cases, plus **general geocentric body crossings**
  (any backend-supported body), with retrograde multiplicity handled.
- **Heliocentric `helio_cross`** for the planets, via geocentric→heliocentric
  vector reconstruction.
- Surfacing on `pleiades-core` re-exports and a `crossings` CLI subcommand.
- An SE-parity numeric gate (`validate-crossings`) over a committed corpus.

**Out (explicitly):**

- SP-2b/SP-2c material (rise/set/transit, `azalt`, local eclipse circumstances).
- `swe_mooncross_node` (Moon crossing its node — a latitude-zero crossing, not a
  longitude crossing); candidate for a later slice.
- Sidereal-zodiac (ayanamsa) variants of the target longitude — the façade
  applies ayanamsa downstream; crossings are produced tropical/of-date like the
  existing Asc/MC and chart positions. A caller wanting a sidereal target
  converts the target longitude before calling.
- Any change to the ΔT/UT1/time-scale policy beyond documenting the existing
  TDB-native behavior.
- Migrating the eclipse crate onto the shared root-finder (optional later
  cleanup; not required and deliberately avoided to keep `validate-eclipses`
  stable).

## Architecture

### 1. New crate `pleiades-events`

Generic over the backend, exactly like `pleiades-eclipse`. Workspace member
added to `Cargo.toml`; depends on `pleiades-backend`, `pleiades-types`,
`pleiades-apparent`, and (for the Sun apparent-longitude helper) either
`pleiades-eclipse` or a small shared extraction of `apparent_sun_longitude_deg`
(decide during planning; prefer not to create a cycle).

### 2. `root` module (generic bracket + bisect)

```rust
// f: monotone-agnostic target function of TDB Julian day; returns a signed
// residual whose zero marks the event. Wrapping handled by the caller's f.
pub(crate) fn crossings_in_range<F>(
    f: F, lo_jd: f64, hi_jd: f64, step_days: f64, tol_days: f64,
) -> Result<Vec<f64>, EventError>
where F: FnMut(f64) -> Result<f64, EventError>;
```

Step-scan detects sign changes of `f`; each bracket is bisected to `tol_days`
(≈ 0.5 s, as in syzygy). `next`/`previous` are thin wrappers that scan forward
/ backward from the requested instant and take the first result.

### 3. `crossings` module (public API)

```rust
pub struct CrossingEngine<B> { backend: B }

pub enum CrossingFrame {
    GeocentricApparentOfDate, // solcross / mooncross / general bodies
    Heliocentric,             // helio_cross (planets only)
}

#[non_exhaustive]
pub struct Crossing {
    pub body: CelestialBody,
    pub target_longitude: Longitude,
    pub instant: Instant,       // TDB
    pub frame: CrossingFrame,
}

impl<B: EphemerisBackend> CrossingEngine<B> {
    pub fn new(backend: B) -> Self;

    pub fn next_longitude_crossing(
        &self, body: CelestialBody, target: Longitude,
        frame: CrossingFrame, after: Instant,
    ) -> Result<Option<Crossing>, EventError>;

    pub fn previous_longitude_crossing(
        &self, body: CelestialBody, target: Longitude,
        frame: CrossingFrame, before: Instant,
    ) -> Result<Option<Crossing>, EventError>;

    pub fn longitude_crossings_in_range(
        &self, body: CelestialBody, target: Longitude,
        frame: CrossingFrame, start: Instant, end: Instant,
    ) -> Result<Vec<Crossing>, EventError>;

    // Thin conveniences (SE-named cases), geocentric apparent of-date:
    pub fn next_sun_crossing(&self, target: Longitude, after: Instant) -> Result<Option<Crossing>, EventError>;   // solcross
    pub fn next_moon_crossing(&self, target: Longitude, after: Instant) -> Result<Option<Crossing>, EventError>;  // mooncross
}
```

- **Target function** `g(t) = wrap180(longitude(body, frame, t) − target)`, where
  `longitude(..)` is:
  - geocentric apparent of-date via `pleiades-apparent` (Sun via
    `apparent_sun_longitude_deg`) for `GeocentricApparentOfDate`;
  - heliocentric via `P_helio = P_geo − S_geo` for `Heliocentric`.
- **Step size** is body-scaled (Moon ≈ 0.25 d, Sun ≈ 1 d, slower planets larger)
  so no crossing is skipped while keeping sample counts modest.
- **Frame/body guards:** `Heliocentric` requested for Sun/Moon → fail closed;
  a body the backend does not support, or a heliocentric reconstruction with a
  missing `distance_au`, → fail closed (no NaN/placeholder emitted).

### 4. Surfacing

- Re-export `CrossingEngine`, `Crossing`, `CrossingFrame`, and `EventError` from
  `pleiades-core`. (`next`/`previous` encode search direction, so no public
  `Direction` type is needed; `direction` appears only as a corpus fixture
  column matching `swe_helio_cross`'s `dir` argument.)
- Add a `crossings` CLI subcommand rendering next/previous/in-range results for a
  body + target longitude, mirroring the eclipse subcommand's output style.

### 5. SE reference extension (`tools/se-crossings-reference`)

New isolated, out-of-workspace crate (own `Cargo.lock`), like the existing four
SE harnesses. Calls `swe_solcross_ut`/`swe_mooncross_ut`/`swe_helio_cross` over a
fixture set of `(body, target_longitude, start_epoch, direction)` and emits the
crossing JD (converted to TDB for `_ut` rows), writing the `crossings-corpus` +
manifest (row counts, checksums, SE version 2.10.03).

### 6. Corpus (`crates/pleiades-validate/data/crossings-corpus`)

Committed CSV slice(s) covering: Sun and Moon crossings of cardinal + arbitrary
longitudes; at least one general geocentric body including a **retrograde
triple-crossing** case; and heliocentric planet crossings. Manifest records
checksums + SE version. A clean checkout stays tool-free and validates only
committed values, per the de440/SE-corpus precedent.

### 7. The gate: `validate-crossings`

Fail-closed command in `pleiades-validate` (`crossings_validation.rs`), wired
into `release-smoke`/`release-gate` next to `validate-eclipses`/`validate-angles`.
Checks:

- corpus checksum / schema / provenance drift;
- completeness (every fixture present, including the retrograde-multiplicity
  case's full crossing set);
- numeric residuals vs **per-body time ceilings** (tight for Sun/Moon — target
  a few seconds of time; documented looser bands for slow/retrograde planets
  near stations, where dλ/dt → 0 makes the time of a fixed-λ crossing
  ill-conditioned), living in a `thresholds` module mirroring the house/angles
  gates;
- claim-tier ↔ evidence alignment carried through the existing overclaim audit.

## Data flow

```
fixtures (body, target_longitude, start_epoch, direction)
  └─(offline, SE present)─> se-crossings-reference:
        swe_solcross_ut / swe_mooncross_ut / swe_helio_cross → crossing JD (→ TDB)
        └─ write crossings-corpus + manifest
              └─> committed corpus (source of truth)
                     └─(runtime gate: validate-crossings)─>
                           pleiades: CrossingEngine::longitude_crossings_in_range()
                           └─ compare each crossing time vs SE within its ceiling
                                 └─> pass / fail (fail-closed)
```

## Error handling / fail-closed conditions

- **Ill-conditioned crossings.** Near a retrograde station dλ/dt → 0, so a fixed
  longitude may be grazed or crossed three times in quick succession. The
  step-scan must use a step small enough to separate them; the gate ceilings for
  such bodies are documented as looser in *time* precisely because the crossing
  time is sensitive there. Return the bracketed roots, not a NaN.
- **Frame/body misuse.** `Heliocentric` for Sun/Moon, an unsupported body, or a
  missing `distance_au` for reconstruction → `EventError` (documented kind), not
  a placeholder value.
- **Out-of-window.** Requests or scans outside 1900–2100 TDB fail closed with a
  window error (mirrors `EclipseError`).
- **Gate fails** on: missing slice, checksum/schema/provenance drift, any time
  residual over its ceiling, or a missing crossing in a multi-crossing fixture.
- **Generation fails** only on SE-side problems (mirrors `validate-eclipses`).

## Constraints

- **C1 — Pure-Rust workspace audit (hard).** The SE binding must never enter the
  published workspace lockfile (`workspace-audit` fails closed on `-sys`/`links`/
  `build.rs`). Satisfied by making `tools/se-crossings-reference` a new isolated
  crate with its own `Cargo.lock`, outside the workspace — no new in-workspace
  FFI. (`libclang-dev` + `LIBCLANG_PATH` needed only to *build the harness* and
  regenerate the corpus, never to run the gate or build the workspace, per the
  se-lilith/se-equatorial precedent.)
- **C2 — SE license.** Verification-only, non-shipping harness; reuse the
  existing `LICENSE-NOTES.md` posture from the other SE reference tools.
- **C3 — Do not perturb passing gates.** The eclipse crate and `validate-eclipses`
  are left untouched; the shared root-finder is a fresh extraction in
  `pleiades-events`, not a refactor of `syzygy.rs`.

## Compatibility / versioning

- New public surface (crossings API + CLI subcommand): bump the compatibility
  profile from `0.7.4` and note the crossings capability in README "current
  state". Add the crossings entry to the compatibility profile with its claim
  tier tied to the `validate-crossings` evidence, and let the overclaim audit
  enforce tier ↔ evidence ↔ profile ↔ prose agreement.
- No breaking change to existing public types (crossings are additive), so the
  API-stability profile (`0.2.1`) is unchanged unless planning finds a shared
  type must be touched — in which case take the bump then.

## Testing

- Unit tests per case against inline goldens: Sun crossing 0°/90°/180°/270°;
  Moon crossing an arbitrary longitude; a **retrograde planet triple-crossing**
  with all three roots asserted; a heliocentric planet crossing.
- `next`/`previous`/`in_range` consistency (the first `in_range` result equals
  `next`; the last equals `previous`).
- Fail-closed tests: `Heliocentric` for Sun/Moon, unsupported body, missing
  `distance_au`, out-of-window instant — assert the documented error, not NaN.
- Doctest examples on every new public item (the crates doctest heavily).
- Integration: the `validate-crossings` gate over the committed corpus; a
  manifest checksum-drift test.
- Regression guard: `validate-eclipses` and `validate-angles` still pass
  unchanged (proves the new crate did not perturb existing engines/gates).

## Open items (confirm during planning/implementation)

1. **Per-body time ceilings** — set from measured SE-vs-pleiades residuals; tight
   for Sun/Moon, documented looser for slow/retrograde planets near stations.
2. **Sun apparent-longitude helper reuse** — depend on `pleiades-eclipse` for
   `apparent_sun_longitude_deg` vs. extract it into `pleiades-apparent` (or a
   shared spot) to avoid an events→eclipse dependency. Prefer extraction if it
   is clean; otherwise depend directly.
3. **Heliocentric apparent vs geometric convention** — pin `helio_cross` output
   (light-time from Sun? aberration?) to whatever `swe_helio_cross` produces
   under the chosen flags, proven by the parity gate.
4. **Step sizes per body class** — choose steps small enough to separate
   retrograde multiple crossings without excessive sampling.
5. **Corpus shape** — one `crossings.csv` with a `frame`/`body` column vs.
   per-frame slices; pick to keep the schema clean and the manifest simple.
