# pleiades-eclipse

Global geocentric and per-observer local solar and lunar eclipse computation
for the `pleiades` astrology workspace, derived from validated Sun and Moon
positions.

## What it provides

For any time in the 1900-01-01 … 2100-01-01 window (the packaged ephemeris
boundary), `pleiades-eclipse` returns the solar and lunar eclipses that occur,
each carrying:

- **Eclipse type** — solar (total / annular / hybrid / partial) or lunar
  (penumbral / partial / total)
- **Instant of greatest eclipse** — the moment to cast a chart on
- **Magnitude** — fraction of the disk covered (geocentric)
- **Gamma** — least distance of the shadow axis from Earth's centre, in Earth
  radii
- **Saros series** number
- **Eclipsed longitude** — apparent tropical ecliptic longitude of date at
  greatest eclipse (no ayanamsa; sidereal conversion is the façade layer's job)
- **Greatest-eclipse location** (solar only) — geographic sub-shadow point;
  lunar eclipses have no location

**Local (per-observer) circumstances** — via `EclipseEngine::local_circumstances`
and `next_local_eclipse`/`previous_local_eclipse` (Swiss-Ephemeris
`swe_sol_eclipse_when_loc`/`swe_lun_eclipse_when_loc` analogues), for both
solar and lunar eclipses, given a geographic observer and atmosphere:

- **Contact times** — per-observer first/second/third/fourth contact (solar)
  or penumbral/umbral ingress-egress (lunar), as `LocalContact` values
- **Magnitude / obscuration** — observer-local eclipse magnitude, and (solar)
  obscured area fraction
- **Azimuth / altitude** — on-sky position of the eclipsed body at greatest
  local eclipse
- **Local visibility** — whether the eclipse is visible above the observer's
  horizon (with refraction) at all

`next_local_eclipse`/`previous_local_eclipse` reuse the existing global
`next_eclipse`/`previous_eclipse` walk, filtering to the first eclipse whose
local circumstances are computable for the given observer — no new external
dependency was introduced.

## Window and data-bound

Coverage is strictly 1900-01-01 through 2100-01-01 (JD 2 415 020.5 … JD 2 488 069.5,
TDB), bounded by the packaged Sun/Moon ephemeris. Four NASA-canon eclipses
falling in mid/late 2100 are excluded as uncomputable with the packaged data.

## Validation

The fail-closed `validate-eclipses` gate (in `pleiades-validate`) recomputes
every in-window eclipse from NASA's Five Millennium Canon of Solar/Lunar
Eclipses (Espenak/Meeus) and enforces:

| Metric | Tolerance |
|---|---|
| Greatest-eclipse time | ≤ 60 s |
| Magnitude | ≤ 0.01 |
| Eclipse type | exact match |
| Saros series | exact match |
| Eclipsed longitude | ≤ 1.0″ |

Approximately 908 eclipses pass all tolerances. One documented knife-edge
eclipse (1948-05-09, Saros 137, annular vs hybrid at magnitude ≈ 1.0) is
allowlisted for the exact-type check only; its time, magnitude, Saros, and
longitude are still enforced. The eclipsed-longitude reference is computed
independently (NASA canon + Skyfield/DE440), so the gate genuinely validates
accuracy.

The fail-closed `validate-eclipses-local` gate (CLI aliases `eclipses-local-gate`,
`eclipse-local`) recomputes local circumstances against a committed
Swiss-Ephemeris reference corpus (29 solar rows + 20 lunar rows) and enforces
measured, data-driven ceilings (~1.4x the observed maxima):

| Metric | Ceiling | Measured max |
|---|---|---|
| Solar contact/greatest-eclipse instant (well-conditioned) | ≤ 23.0 s | 16.1 s |
| Solar contact/greatest-eclipse instant (grazing/central-limit) | ≤ 95.0 s | 65.0 s |
| Lunar contact instant | ≤ 7.0 s | 5.0 s |
| Solar magnitude/obscuration | ≤ 0.002 | ~1.1e-3 |
| Lunar magnitude | ≤ 0.001 | ~7.1e-4 |
| On-sky azimuth | ≤ 130.0″ | 91.0″ |
| Apparent altitude | ≤ 120.0″ | 81.0″ |

This is **arcsecond-class** parity on azimuth/altitude, not a claim of
sub-arcsecond parity, and second-class parity on contact times, widening at
grazing/central-limit geometry where the contact-time derivative is
ill-conditioned.

## Quick start

```rust
use pleiades_data::packaged_backend;
use pleiades_eclipse::{EclipseEngine, EclipseFilter};
use pleiades_types::{Instant, JulianDay, TimeScale};

let engine = EclipseEngine::new(packaged_backend());
let after = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tdb);
let next = engine.next_eclipse(after, EclipseFilter::All).unwrap();
assert!(next.is_some());
```
