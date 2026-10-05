# pleiades

[![crates.io](https://img.shields.io/crates/v/pleiades-core.svg)](https://crates.io/crates/pleiades-core)
[![docs.rs](https://img.shields.io/docsrs/pleiades-core)](https://docs.rs/pleiades-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

`pleiades` is a pure-Rust ephemeris and chart library for astrology software:
planetary positions, houses, ayanamsas, eclipses and astronomical events, with
no C dependencies and no data files to download.

It is experimental and pre-1.0: breaking changes can land in any minor release.
See [Status](#status) before relying on it.

## Quick start

```bash
cargo add pleiades-core pleiades-data
```

```rust
use pleiades_core::{
    CelestialBody, ChartEngine, ChartRequest, CivilDateTime, HouseSystem, Latitude, Longitude,
    ObserverLocation, TimeScale,
};
use pleiades_data::packaged_backend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The packaged backend ships its own ephemeris data (1900-2100).
    let engine = ChartEngine::new(packaged_backend());

    // 2000-01-01 12:00:00 UTC, converted to Terrestrial Time.
    let civil = CivilDateTime::new(2000, 1, 1, 12, 0, 0.0);
    let bodies = vec![CelestialBody::Sun, CelestialBody::Moon, CelestialBody::Mars];
    let request = ChartRequest::from_civil(civil, TimeScale::Utc, TimeScale::Tt, bodies)?
        .request
        .with_observer(ObserverLocation::new(
            Latitude::from_degrees(51.5074),
            Longitude::from_degrees(-0.1278),
            None,
        ))
        .with_house_system(HouseSystem::Placidus);

    let chart = engine.chart(&request)?;
    for placement in &chart.placements {
        println!("{}", placement.summary_line());
    }
    Ok(())
}
```

```text
Sun 280.36892400760064°  Capricorn    10  Direct      Interpolated
Moon 223.32379972410268°  Scorpio       7  Direct      Interpolated
Mars 327.96330497994586°  Aquarius     12  Direct      Interpolated
```

Charts are tropical and apparent-of-date by default. Add
`.with_zodiac_mode(...)` for a sidereal chart; the observer and house system
are optional. The full API is on [docs.rs](https://docs.rs/pleiades-core).

## What it computes

- **Charts**: body positions, signs, houses, motion and aspects through
  [`pleiades-core`](https://docs.rs/pleiades-core).
- **Houses and ayanamsas**: today 24 house systems pass the Swiss Ephemeris
  numeric gate and 48 ayanamsas pass theirs, of 25 and 59 catalogued.
- **Civil time**: UTC/UT1 to TT/TDB, leap-second-exact, 1900–2100
  ([`pleiades-time`](https://docs.rs/pleiades-time)).
- **Eclipses**: global and per-observer solar and lunar eclipses
  ([`pleiades-eclipse`](https://docs.rs/pleiades-eclipse)).
- **Events**: longitude crossings, stations, exact aspects, rise/set/transit,
  nodes and apsides, phase and magnitude, lunar occultations
  ([`pleiades-events`](https://docs.rs/pleiades-events)).

## Crates

Most applications need only `pleiades-core` plus a backend. The library crates
are published to crates.io, each with its own version:

<!-- audit:published-crates -->
| Crate | Role |
| --- | --- |
| `pleiades-core` | Chart façade and re-exports; start here. |
| `pleiades-data` | Packaged offline ephemeris backend, 1900–2100. |
| `pleiades-vsop87` | Algorithmic planetary backend (VSOP87B). |
| `pleiades-elp` | Compact lunar backend. |
| `pleiades-fict` | Fictitious and hypothetical bodies. |
| `pleiades-jpl` | JPL reference corpus and Horizons ingestion. |
| `pleiades-events` | Crossings, stations, aspects, rise/set and other events. |
| `pleiades-eclipse` | Solar and lunar eclipses. |
| `pleiades-houses` | House systems. |
| `pleiades-ayanamsa` | Ayanamsas and sidereal offsets. |
| `pleiades-time` | Civil-time conversion. |
| `pleiades-apparent` | Apparent-place corrections. |
| `pleiades-apsides` | Lunar nodes and apsides. |
| `pleiades-compression` | Compressed artifact format. |
| `pleiades-backend` | Backend trait and capability metadata. |
| `pleiades-types` | Shared types: angles, bodies, time scales, observers. |
<!-- /audit:published-crates -->

<!-- audit:unpublished-crates -->
`pleiades-cli` and `pleiades-validate` are contributor tooling and stay
unpublished; see [docs/cli.md](docs/cli.md).
<!-- /audit:unpublished-crates -->

**Minimum supported Rust version: 1.99.0.** Raising it is a breaking change
and is released as one.

## Status

Each computed surface is guarded by a numeric gate against a reference,
mostly Swiss Ephemeris or JPL DE440. The gate for each surface, its accuracy
class and the known limits are listed in [docs/status.md](docs/status.md).
Production-accuracy claims wait on the phases in [PLAN.md](PLAN.md).

## Documentation

- [docs/status.md](docs/status.md) — accuracy per surface and known limits.
- [docs/time-observer-policy.md](docs/time-observer-policy.md) — time scales, observers, apparentness and frames.
- [docs/lunar-theory-policy.md](docs/lunar-theory-policy.md) — lunar theory selection and limits.
- [docs/cli.md](docs/cli.md) — the inspection and validation command-line tools.
- [docs/development.md](docs/development.md) — building, testing, workspace layout and releasing.
- [docs/threat-model.md](docs/threat-model.md) — trust boundaries for ingested data.
- [SPEC.md](SPEC.md) and [`spec/`](spec/) — the design specification.

## License

`MIT OR Apache-2.0`. The full texts are in [`LICENSE-APACHE`](LICENSE-APACHE)
and [`LICENSE-MIT`](LICENSE-MIT).
