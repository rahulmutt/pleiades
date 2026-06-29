# pleiades-eclipse

Global geocentric solar and lunar eclipse computation for the `pleiades`
astrology workspace, derived from validated Sun and Moon positions.

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

**Not provided:** local / per-observer circumstances (local magnitude, contact
times, visibility path). This is a global/geocentric product.

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
