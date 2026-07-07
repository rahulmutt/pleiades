# SP-4 — Planetary Nodes and Apsides (`swe_nod_aps` analogue)

Phase: Event-engine track (SP series), slice SP-4 — the first of the two
"astronomy-flavored" engine gaps deferred out of SP-3 (`swe_nod_aps`; the other,
`swe_pheno`, remains queued).

## Summary

Add a Swiss-Ephemeris `swe_nod_aps()` analogue: for a body and instant, return the
four orbital singular points — **ascending node, descending node, perihelion, and
aphelion (or the orbit's second focus)** — as ecliptic-of-date positions with
speeds, computed either from **mean orbital elements** (Moon + Mercury–Neptune) or
from the **osculating ellipse** of the body's instantaneous state (all bodies,
optionally barycentric for the outer planets).

Body coverage is **full SE parity by composition**: Sun/Earth (per SE's semantics),
Moon, Mercury–Pluto, the 36 Tier-A asteroids/TNOs (states via the JPL/SPK backend),
and the 19 committed `seorbel.txt` fictitious bodies (states via
`FictitiousBackend`) — any body the caller's backend chain serves. The surface is an
**engine function**, not new `CelestialBody` variants: SE made the same choice
(4 points × ~60 bodies × 2 methods does not belong in a body enum).

## SE function targeted

`swe_nod_aps()` / `swe_nod_aps_ut()`, including all four method bits:

| SE method bit | This design |
| --- | --- |
| `SE_NODBIT_MEAN` | `NodApsMethod::Mean` — polynomial mean elements; Moon + Mercury–Neptune only |
| `SE_NODBIT_OSCU` | `NodApsMethod::Osculating` — osculating ellipse from the sampled state |
| `SE_NODBIT_OSCU_BAR` | `NodApsMethod::OsculatingBarycentric` — element formation about the SSB |
| `SE_NODBIT_FOPOINT` | `ApsisConvention::SecondFocus` (default `ApsisConvention::Aphelion`) |

SE's `method = 0` default (mean where available, else osculating) is mirrored by a
`nod_aps_default` convenience; the explicit method call never falls back silently.

## Public API (in `pleiades-events`)

```rust
pub enum NodApsMethod { Mean, Osculating, OsculatingBarycentric }
pub enum ApsisConvention { Aphelion, SecondFocus }

pub struct NodApsPoint {
    pub longitude_deg: f64,      // geocentric ecliptic of date, [0, 360)
    pub latitude_deg: f64,
    pub distance_au: f64,
    pub longitude_speed_deg_per_day: f64,   // central-difference speeds
    pub latitude_speed_deg_per_day: f64,
    pub distance_speed_au_per_day: f64,
}

pub struct NodesApsides {
    pub ascending: NodApsPoint,
    pub descending: NodApsPoint,
    pub perihelion: NodApsPoint,
    pub aphelion: NodApsPoint,   // second focus when ApsisConvention::SecondFocus
    pub method: NodApsMethod,    // the method actually served
}

impl<B: EphemerisBackend> EventEngine<B> {
    pub fn nod_aps(
        &self,
        body: CelestialBody,
        jd_tdb: f64,
        method: NodApsMethod,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError>;

    /// SE `method = 0`: Mean for Moon + Mercury–Neptune, Osculating otherwise.
    pub fn nod_aps_default(
        &self,
        body: CelestialBody,
        jd_tdb: f64,
        convention: ApsisConvention,
    ) -> Result<NodesApsides, EventError>;
}
```

Output frame/center default is **geocentric ecliptic-of-date longitude /
latitude / distance**, matching how SE is typically called (mean vs true ecliptic
of date is pinned against the reference corpus — see Architecture). "Apsis" names are used
generically (perihelion/aphelion for heliocentric orbits; for the Moon's geocentric
orbit the same fields carry perigee/apogee).

## Architecture

Two crates change; none are added.

### `pleiades-apsides` — pure two-body geometry (generalized)

The crate generalizes from "lunar osculating apsides" to full node/apsis geometry,
staying **frame-agnostic** (outputs in the input state's frame) and pure:

- `nodes_and_apsides(pos_au, vel_au_per_day, mu, convention)` — forms the
  osculating ellipse (reusing the existing `apsides` internals), then:
  - **Nodes**: the points on the ellipse along the line of nodes (orbit ∩ the
    input frame's reference plane), each with its true orbital radius.
  - **Apsides**: perihelion and aphelion from the apse line (existing code path),
    or the **second focus** at distance `2ae` from the primary focus, opposite
    perihelion.
- A committed per-body **μ table**: `GM_sun · (1 + m_body/m_sun)` for
  heliocentric orbits (SE-matching mass ratios), the existing
  `MU_EARTH_MOON_AU3_PER_DAY2` for the Moon. Small-body masses are treated as
  zero (μ = GM_sun) unless SE's implementation is found to do otherwise during
  corpus pinning.
- The existing `ApsidesError` taxonomy (degenerate orbit, unbound orbit,
  non-finite) extends to the node path unchanged.

### `pleiades-events` — engine, mean-element tables, sampling

- **`mean_elements` module**: committed polynomial tables for the Moon's mean node
  and mean apogee and for Mercury–Neptune mean orbital elements, ported from the
  vendored Swiss Ephemeris source in `tools/` (the parity authority). Evaluated
  directly — the Mean path touches no backend.
- **Osculating path** (per body, per instant):
  1. Sample backend **positions** at `t − h, t, t + h` (SE itself derives orbital
     velocity from three position samples; finite differencing is the
     parity-faithful method, not a shortcut). The step `h` is a tuning knob set
     by measured gate residuals.
  2. Spherical → Cartesian; central-difference the velocity.
  3. Recenter for element formation: heliocentric for planets/asteroids/fictitious
     (subtract the packaged Sun the same way `pleiades-fict` geocentricizes),
     geocentric for the Moon; SSB recentering for `OsculatingBarycentric`.
  4. **Rotate the J2000 state into the mean ecliptic of date** before geometry —
     nodes are plane-relative, so this rotation is load-bearing.
  5. Call `pleiades-apsides::nodes_and_apsides`.
  6. Re-center each returned point to the output center per SE semantics
     (heliocentric node point → geocentric direction where SE does so).
- **Point speeds**: evaluate the full pipeline at `t ± h` and central-difference
  each point's coordinates — the same pattern `pleiades-data` already uses for
  `osculating_apsis_motion`.

The exact SE semantics per body class — nodes referred to mean vs true ecliptic of
date, whether SE applies light-time/aberration to the returned points, Sun/Earth
special-casing, and the OSCU_BAR body set — are **pinned empirically against the
reference corpus during implementation**, not guessed here. Where SE's choice is
ambiguous, the corpus decides; the spec commits to parity, not to a guessed
formula.

## Error handling

All failures are typed `EventError` variants; fail-closed, no NaNs escape:

- `Mean` requested for a body without mean elements (Pluto, asteroids, fictitious)
  → `EventError::UnsupportedMethod { body, method }` (new variant). Same for
  `OsculatingBarycentric` outside the body set SE supports it for.
- Degenerate geometry (near-circular apse/node ill-conditioning — Venus, Neptune,
  and near-circular fictitious orbits are the real risk; unbound; non-finite)
  → typed engine error wrapping `ApsidesError`. The eccentricity/inclination
  conditioning floors are **measured during corpus work**, not assumed.
- Out-of-coverage instants and backend misses → the existing
  `EventError::Backend` / missing-coordinate errors. Three-point sampling shrinks
  the usable window by `h` at each end of 1900–2100; documented, and the corpus
  stays inside it.

## Validation

The established fail-closed gate pattern, applied whole:

- **Reference tool** `tools/se-nodaps-reference`: bindgen against the vendored
  Swiss Ephemeris (build requires `libclang-dev` + `LIBCLANG_PATH`; running the
  gate never does — it reads the committed CSV via `include_str!`). Emits
  body × method × convention × epoch rows spanning 1900–2100, four points ×
  (lon, lat, dist, 3 speeds) per row.
- **Gate** `validate-nod-aps` (CLI aliases `nod-aps` / `nodaps-gate`), wired into
  `run_all_numeric_gates`; corpus checksum-guarded (fnv1a64) and pinned by row
  count. Per-body-class ceilings set from measured residuals (~1.4× measured
  maxima, house-gate style):
  - **Mean rows are definitional** (same polynomials) → sub-arcsecond expected,
    tight ceilings.
  - **Osculating planets/Moon** → packaged states are sub-arcsecond; expect
    arcsecond-class parity. Measured, not promised.
  - **Osculating asteroids/fictitious** → cross-theory floors (our JPL/SPK and
    Kepler states vs the SE reference build's ephemeris), the same honest framing
    as the `validate-crossings` Tier-2 ceilings. Nibiru keeps its documented
    ~370 AD reference-equinox carve-out.
- **Unit level** (`pleiades-apsides`): closed-form tests — an analytic Kepler
  orbit with known elements in a known plane must yield known node/apsis/focus
  positions — plus property tests: nodes lie in the reference plane
  (latitude ≈ 0) and 180° apart in the orbital plane, perihelion/aphelion 180°
  apart in true anomaly with `r_peri = a(1−e)`, `r_apo = a(1+e)`, second focus at
  `2ae` from the primary focus.

## Phasing (single spec, one branch)

1. **Geometry**: generalize `pleiades-apsides` (nodes, second focus, μ table) +
   closed-form/property unit tests.
2. **Mean path**: mean-element tables + `Mean` method for Moon + Mercury–Neptune.
3. **Osculating path**: planets/Moon first, then asteroids and fictitious bodies —
   pure composition (same code, more corpus rows) — plus `OsculatingBarycentric`
   and `SecondFocus`.
4. **Gate + closeout**: reference tool, committed corpus, `validate-nod-aps` wired
   into `run_all_numeric_gates`; README/PLAN/status-file updates; compatibility
   profile bump to **0.7.10**; API stability profile unchanged (`pleiades-events`
   is unpublished contributor-facing surface; `pleiades-apsides` changes are
   additive).

## Non-goals

- No heliocentric / J2000 / equatorial **output-flag matrix** (SE `iflag`
  variants): the default geocentric ecliptic-of-date output is what the gate
  pins; the chart layer already owns frame conversions.
- No user-supplied orbital elements (stays on the further-out list with
  occultations and central-path cartography).
- No new `CelestialBody` variants and no chart-layer routing of these points.
- No `swe_pheno` content (phase/magnitude) — that is the other queued slice.
- No change to the existing lunar `MeanNode`/`TrueNode`/`MeanApogee`/`TrueApogee`
  chart bodies; the engine's Moon rows validate against the same SE quantities
  but the chart surface is untouched.
