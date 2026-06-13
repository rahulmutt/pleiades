# pleiades-types

[![crates.io](https://img.shields.io/crates/v/pleiades-types.svg)](https://crates.io/crates/pleiades-types)
[![docs.rs](https://img.shields.io/docsrs/pleiades-types)](https://docs.rs/pleiades-types)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Shared typed vocabulary for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas.

This crate sits at the base of the `pleiades-*` layering and depends on no other pleiades crates. Enable the `serde` feature for serialization support.

## Status

Experimental `0.2.x`. First-party backends expose mean geometric coordinates only, and broader accuracy claims are still gated; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
