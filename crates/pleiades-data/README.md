# pleiades-data

[![crates.io](https://img.shields.io/crates/v/pleiades-data.svg)](https://crates.io/crates/pleiades-data)
[![docs.rs](https://img.shields.io/docsrs/pleiades-data)](https://docs.rs/pleiades-data)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Packaged offline ephemeris data (precomputed Sun and Moon positions, derived
from JPL public-domain ephemerides) and its `EphemerisBackend` for the
[pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

The crate ships a compressed artifact covering 1900-01-01 through 2100-01-01,
regenerated from the checked-in JPL reference snapshot and validated against a
deterministic binary fixture. It covers the comparison-body planetary set plus
the source-backed custom asteroid `asteroid:433-Eros`, and falls back to other
providers when callers request bodies outside the packaged slice. Enable the
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

## Status

Experimental `0.2.x`. First-party backends expose mean geometric coordinates
only, and broader accuracy claims are still gated; see the
[workspace README](https://github.com/rahulmutt/pleiades#readme) for the full
maturity posture.

## License

MIT OR Apache-2.0
