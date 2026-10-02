# pleiades-types

[![crates.io](https://img.shields.io/crates/v/pleiades-types.svg)](https://crates.io/crates/pleiades-types)
[![docs.rs](https://img.shields.io/docsrs/pleiades-types)](https://docs.rs/pleiades-types)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Shared typed vocabulary for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: angles, bodies, time scales, observers, coordinates, zodiac modes, house systems, and ayanamsas.

This crate sits at the base of the `pleiades-*` layering and depends on no other pleiades crates. Enable the `serde` feature for serialization support.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. First-party backends return mean geometric coordinates; apparent place, topocentric correction, and civil-time conversion are applied above the backend boundary, by the chart layer in `pleiades-core` and the event engines. Each accuracy claim is tied to a numeric gate; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
