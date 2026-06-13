# pleiades-core

[![crates.io](https://img.shields.io/crates/v/pleiades-core.svg)](https://crates.io/crates/pleiades-core)
[![docs.rs](https://img.shields.io/docsrs/pleiades-core)](https://docs.rs/pleiades-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

High-level chart façade for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: typed tropical/sidereal chart requests, request validation, compatibility and API-stability profiles, and re-exports for common consumers.

Sits at the top of the published `pleiades-*` library layering (types, backend, houses, ayanamsa, compression). Pair it with a backend crate such as `pleiades-vsop87` and `pleiades-elp` to compute positions.

## Status

Experimental `0.1.x`. First-party backends expose mean geometric coordinates; UTC/UT1 need caller-supplied conversion offsets, and apparent/topocentric requests are rejected. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
