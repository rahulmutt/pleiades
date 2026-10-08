# pleiades-data

[![crates.io](https://img.shields.io/crates/v/pleiades-data.svg)](https://crates.io/crates/pleiades-data)
[![docs.rs](https://img.shields.io/docsrs/pleiades-data)](https://docs.rs/pleiades-data)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Packaged offline ephemeris data (precomputed positions for the Sun, the Moon,
Mercury through Pluto and five asteroids, derived from JPL public-domain ephemerides) and its
`EphemerisBackend` for the
[pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

The crate ships a compressed artifact covering 1900-01-01 through 2100-01-01,
regenerated from the JPL de440 and `sb441-n373s` kernels (two kernels are required) and validated against a
deterministic binary fixture. The backend serves the Sun, the Moon, Mercury
through Pluto, and Ceres, Pallas, Juno, Vesta and `asteroid:433-Eros`, and
falls back to other providers when callers request bodies outside the packaged
slice.

The planets come from the JPL de440 planetary kernel; the five asteroids are
dense heliocentric fits to the JPL `sb441-n373s` small-body kernel and are
gated against the `sb441-n373s` rows of `asteroid_reference.csv` (row-gate
maxima 0.2938″ longitude, 0.2646″ latitude; dense 0.5-day sampling against the
kernel finds at most 0.52″ longitude and 0.37″ latitude). Enable the
`packaged-artifact-path` feature to load an explicit artifact file for larger
or externally distributed packaged datasets.

## Quick start

```rust
use pleiades_backend::{CelestialBody, Instant, JulianDay, TimeScale};
use pleiades_data::{packaged_backend, packaged_lookup};

let _backend = packaged_backend();
let instant = Instant::new(JulianDay::from_days(2_451_545.0), TimeScale::Tt);
let sun = packaged_lookup(&CelestialBody::Sun, instant)
    .expect("Sun should be in the packaged artifact");
assert!(sun.distance_au.is_some());
```

Besides the packaged bodies, the backend serves six derived lunar points: the
osculating `TrueNode`, `TrueApogee` and `TruePerigee` (from the packaged Moon
state) and the mean `MeanNode`, `MeanApogee` and `MeanPerigee` (from the mean
lunar elements in `pleiades-apsides`). The mean apogee and perigee are the
points on the inclined mean orbit, as Swiss Ephemeris reports `SE_MEAN_APOG`.
All six are served only inside the packaged window.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. This backend returns mean
geometric coordinates and rejects apparent requests; the chart layer in
`pleiades-core` and the event engines in `pleiades-events` apply apparent
place on top of it. See the
[workspace README](https://github.com/rahulmutt/pleiades#readme) for the full
maturity posture.

## License

MIT OR Apache-2.0
