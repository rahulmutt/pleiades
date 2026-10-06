# pleiades-jpl

[![crates.io](https://img.shields.io/crates/v/pleiades-jpl.svg)](https://crates.io/crates/pleiades-jpl)
[![docs.rs](https://img.shields.io/docsrs/pleiades-jpl)](https://docs.rs/pleiades-jpl)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Checked-in JPL Horizons reference snapshots and snapshot-backed validation helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, plus pure-Rust CSV parsing entry points for JPL-style manifest/row corpora.

Depends on `pleiades-types` and `pleiades-backend`. This is a reference/validation fixture crate, not a broad public-data reader.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. The checked-in corpus is regression evidence, sparse relative to production-coverage goals; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

`JplSnapshotBackend` answers at an exact fixture row, or between rows that lie
on both sides of the instant and span at most five days. Any other instant
returns `OutOfRangeInstant`. Use `SpkBackend` with a JPL kernel for positions
at arbitrary dates.

## License

MIT OR Apache-2.0
