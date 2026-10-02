# pleiades-vsop87

[![crates.io](https://img.shields.io/crates/v/pleiades-vsop87.svg)](https://crates.io/crates/pleiades-vsop87)
[![docs.rs](https://img.shields.io/docsrs/pleiades-vsop87)](https://docs.rs/pleiades-vsop87)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Pure-Rust VSOP87B planetary position backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, with generated binary coefficient tables and an approximate Pluto fallback.

Depends on `pleiades-types` and `pleiades-backend`. The crate ships its generated coefficient tables plus the raw VSOP87B source tables and the `regenerate-vsop87b-tables` tool used to rebuild them.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. This backend returns mean geometric heliocentric-derived geocentric positions and rejects apparent and topocentric requests; the chart layer in `pleiades-core` applies those corrections on top of a backend. Pluto is approximate and excluded from release-grade claims. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
