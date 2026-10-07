# pleiades-ayanamsa

[![crates.io](https://img.shields.io/crates/v/pleiades-ayanamsa.svg)](https://crates.io/crates/pleiades-ayanamsa)
[![docs.rs](https://img.shields.io/docsrs/pleiades-ayanamsa)](https://docs.rs/pleiades-ayanamsa)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Ayanamsa catalog, aliases, reference offset metadata, and sidereal offset helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace.

Depends only on `pleiades-types`.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. Formula, provenance, and interoperability audits still gate stronger compatibility claims; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## Anchor stars

`star_anchor` names the anchor star of a star-anchored ayanamsa (or `None`), and `anchor_star_mean_place` gives that star's mean place at an instant. The apparent place (deflection and aberration) is applied by the callers through `pleiades_apparent::apparent_star_place`; this crate stays free of that dependency.

## License

MIT OR Apache-2.0
