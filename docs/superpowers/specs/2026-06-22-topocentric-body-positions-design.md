# Topocentric Body Positions — Design

**Status:** Approved design, pre-implementation.
**Phase:** 4 (Request-Mode Semantics).
**Date:** 2026-06-22.

## Goal

Implement topocentric body positions as an opt-in chart-layer correction, mirroring
the `pleiades-apparent` apparent-place-of-date pattern. Topocentric place is the
geocentric apparent place shifted to an observer on Earth's surface. This closes
the one remaining *implementation* item in Phase 4; native sidereal backend output
deliberately stays unsupported.

## Background

The codebase already establishes the relevant pattern and scaffolding:

- `pleiades-apparent` is a pure (`pleiades-types`-only) correction crate that turns
  mean geocentric J2000 positions into apparent ecliptic-of-date positions
  (light-time → precession → nutation → annual aberration), with a typed
  `ApparentProvenance`. It takes time-derived scalars (e.g. the Sun's longitude of
  date) as *parameters* rather than computing them itself.
- `pleiades-core` orchestrates the chart layer and already depends on both
  `pleiades-apparent` and `pleiades-time`.
- `pleiades-types` provides `ObserverLocation`, `EclipticCoordinates`,
  `EquatorialCoordinates`, and ecliptic↔equatorial frame machinery (`frames.rs`).
- `pleiades-backend` carries a `capabilities.topocentric` flag (currently `false`
  everywhere) and a structured "topocentric positions are not implemented"
  rejection at the backend boundary.
- The CLI `chart` command already accepts `--lat`/`--lon` (today used only to build
  the `ObserverLocation` for houses) and `--mean`/`--apparent` apparentness flags,
  with apparent-place-of-date as the default.

## Decisions (from brainstorming)

1. **Scope:** topocentric body positions only. Native sidereal stays unsupported.
2. **Correction terms:** diurnal (geocentric) parallax **and** diurnal aberration
   (full rigor). Atmospheric refraction is omitted and documented (as
   light-deflection is in the apparent crate).
3. **Activation:** opt-in via a new `--topocentric` flag. Geocentric apparent stays
   the default even when `--lat`/`--lon` are supplied for houses (matches Swiss
   Ephemeris `SEFLG_TOPOCTR`; preserves the established geocentric-apparent default).
4. **Placement (Approach A):** a pure module inside `pleiades-apparent`; all
   time-derived inputs supplied by the `pleiades-core` caller. No new crate.
5. **Non-release-grade / distance-less bodies:** an explicit `--topocentric`
   request that cannot be honored **errors** (no silent geocentric fallback),
   because topocentric is opt-in — distinct from the default apparent path, which
   falls back gracefully so default charts never break.

## Physics Model

Topocentric place = geocentric apparent place corrected for the observer's offset
from Earth's center:

1. **Diurnal parallax** (dominant, distance-driven): vector subtraction of the
   observer's geocentric position from the body's geocentric position, computed in
   **equatorial Cartesian** coordinates. Magnitude ~1° for the Moon, ~8.8″ for the
   Sun, arcseconds for planets, negligible for distant bodies.
2. **Diurnal aberration** (≤0.32″ at the equator): the observer's eastward
   rotational velocity `ω·ρ·cosφ′`.

The observer's geocentric vector is derived from geodetic latitude, longitude, and
elevation on the **IAU-76 / WGS84 reference ellipsoid** (the standard `ρ·sinφ′`,
`ρ·cosφ′` quantities), combined with the **local apparent sidereal time** (LAST).

## Architecture (Approach A)

### `pleiades-apparent` (stays `pleiades-types`-only pure)

- **`parallax.rs`** — observer geodetic→geocentric Cartesian vector
  (`ρ·sinφ′`, `ρ·cosφ′` on the IAU-76/WGS84 ellipsoid, elevation-aware) and the
  equatorial-frame subtraction helper. Pure geometry, fully unit-testable.
- **`topocentric.rs`** — orchestrator
  `topocentric_position(apparent_ecliptic, observer_geocentric_vec,
  local_sidereal_time, obliquity, …) -> TopocentricPosition`. Converts
  ecliptic→equatorial (reusing `frames.rs`), subtracts the observer vector, applies
  diurnal aberration, converts back to ecliptic-of-date. **Pure:** every
  time-derived scalar (LAST, obliquity) is a parameter, exactly like
  `apparent_position()` takes the Sun's longitude.

### Provenance (extend, do not fork)

- New optional `TopocentricProvenance { parallax_longitude_arcsec,
  parallax_latitude_arcsec, diurnal_aberration_arcsec, distance_au_used, observer }`,
  carried as an optional field on the result.
- Two new flags on `CorrectionSet`: `diurnal_parallax`, `diurnal_aberration`.
- Geocentric charts leave these `None`/`false`, so existing provenance output is
  byte-for-byte unchanged.
- Add topocentric model sources (ellipsoid model + diurnal aberration formula) to
  the model-sources string.

### `pleiades-time` (small additive change)

- Expose a **public ΔT lookup** (the table already exists internally) so
  `pleiades-core` can derive **UT1 = TT − ΔT** → GMST → local apparent sidereal
  time.

### `pleiades-core` (orchestration; already depends on both crates)

- Compute LAST from the instant + observer longitude + nutation, then call
  `topocentric_position()` immediately after `apparent_position()`.

## Data Flow

Per body, the chart assembly applies:

```
mean geocentric J2000 (backend)
  → apparent_position()      light-time, precession, nutation, annual aberration  (existing)
  → topocentric_position()   diurnal parallax + diurnal aberration                (NEW)
  → sidereal_longitude()     ayanamsa offset, if sidereal requested               (existing)
```

Topocentric slots in right after the apparent block, operating on the of-date
apparent position. The ayanamsa offset (a pure longitude shift) is re-applied
after, mirroring how the apparent path already re-applies it for sidereal charts.

**Two observers stay distinct.** `request.observer` (geographic, used for houses)
supplies the topocentric location. The backend request's `body_observer` **stays
`None`** — backends remain geocentric/mean. The existing backend-boundary rejection
("topocentric positions are not implemented") is therefore untouched and still
correct: it rejects anyone asking a *backend* for topocentric; the chart-layer
feature never sends an observer to the backend, so it never trips it. The
`capabilities.topocentric` flag stays `false` — it denotes *native backend*
topocentric, which remains unsupported (exactly as backends stay mean while
apparent is a chart-layer concept).

**Activation predicate:** apply topocentric only when *all* hold — `--topocentric`
set, `request.observer` present, apparent mode active, and the body has a usable
geocentric distance.

## Error Handling (structured, fail-closed)

| Condition | Behavior |
|---|---|
| `--topocentric` without `--lat`/`--lon` | CLI error: observer required |
| `--topocentric` combined with `--mean` | CLI error: topocentric builds on apparent place |
| `--topocentric` for a body lacking a known distance (e.g. non-release-grade) | structured error — explicit request cannot be honored (no silent fallback) |
| observer out of range / non-finite | existing `ObserverLocationValidationError` |
| non-finite parallax/aberration result | new `ApparentPlaceError::NonFiniteCorrection { stage: "topocentric" }` |
| instant outside ΔT/UT1 window | propagate existing civil-time error |

New error variants (e.g. `MissingDistance`) land in `pleiades-apparent`'s existing
error enum, keeping one error surface for the correction chain.

## CLI

- New `--topocentric` opt-in flag.
- New `--elevation <meters>` to populate `ObserverLocation.elevation_m` (currently
  always `None`).
- `--topocentric` requires `--lat`/`--lon`; incompatible with `--mean`.
- Append a per-body topocentric provenance line (parallax ″, diurnal aberration ″,
  observer) next to the apparent provenance line.
- Update usage/help text.

## Testing & Validation

### Unit tests (`pleiades-apparent`, pure)

- `parallax.rs`: observer geocentric `ρ·sinφ′`/`ρ·cosφ′` matches Meeus reference
  values; equator and pole edge cases.
- `topocentric.rs`: Moon-at-horizon shows ~1° shift; shift → 0 as distance → ∞;
  diurnal aberration peaks ~0.32″ at the equator.
- Property test: shift magnitude orders Moon ≫ Sun ≫ planets ≫ Eros.

### Horizons golden gate (fail-closed, mirrors apparent)

- Commit Horizons **topocentric** goldens for a fixed observer (lat/lon/elev) across
  a few epochs, covering the Moon (critical — largest parallax), the Sun, one
  planet, and one release-grade asteroid.
- Assert our ecliptic lon/lat match within published tolerances (Moon a few arcsec;
  Sun/planets sub-arcsec).
- Assert topocentric **≠** geocentric for the Moon (no silent geocentric fallback —
  same pattern as the apparent goldens gate).

### Regression & integration

- Existing geocentric goldens unchanged; new provenance fields absent in default
  output.
- `pleiades-core` integration: a `--topocentric` Moon chart differs from geocentric
  by the parallax, and a topocentric provenance line is present.
- CLI tests: flag parsing and each error case.

## Docs, Policy & Release Surfaces (updated in lockstep, all gated)

- `ObserverPolicySummary` canonical text → chart-layer topocentric now supported
  (opt-in); backends stay geocentric. (The `CurrentPolicyOutOfSync` gate forces this
  update.)
- `PLAN.md`, `README.md`, and the Phase 4 stage doc "Completed" section → topocentric
  done; Phase 4's only remaining item becomes native-sidereal (the deliberate
  non-goal).

## Out of Scope (YAGNI)

- Native backend topocentric (stays unsupported, rejected at the backend boundary).
- Atmospheric refraction (documented omitted, like light-deflection).
- Topocentric for non-release-grade / distance-less bodies.
- Topocentric houses (houses already consume the observer independently — unchanged).

## Exit Criteria

- Topocentric is implemented as an opt-in chart-layer correction with diurnal
  parallax + diurnal aberration, validated against committed Horizons topocentric
  goldens behind a fail-closed gate that proves genuine (non-geocentric) output.
- All unsupported request modes (native sidereal; backend-native topocentric) still
  produce structured, documented errors.
- CLI, provenance, policy summary, PLAN/README/stage-doc surfaces are updated and
  pass their gates; existing geocentric output is unchanged.
