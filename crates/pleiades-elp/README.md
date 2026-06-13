# pleiades-elp

[![crates.io](https://img.shields.io/crates/v/pleiades-elp.svg)](https://crates.io/crates/pleiades-elp)
[![docs.rs](https://img.shields.io/docsrs/pleiades-elp)](https://docs.rs/pleiades-elp)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Compact Meeus-style lunar baseline backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: Moon, mean/true node, and mean apogee/perigee channels.

Depends on `pleiades-types` and `pleiades-backend`. This is a compact baseline, not a full ELP coefficient implementation.

## Status

Experimental `0.1.x`. Output is mean geometric coordinates; apparent-place corrections and topocentric requests are rejected. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
