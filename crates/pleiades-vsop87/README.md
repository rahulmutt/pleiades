# pleiades-vsop87

Pure-Rust VSOP87B planetary position backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, with generated binary coefficient tables and an approximate Pluto fallback.

Depends on `pleiades-types` and `pleiades-backend`. The crate ships its generated coefficient tables plus the raw VSOP87B source tables and the `regenerate-vsop87b-tables` tool used to rebuild them.

## Status

Experimental `0.1.x`. Output is mean geometric heliocentric-derived geocentric positions; apparent-place corrections and topocentric requests are rejected. Pluto is approximate and excluded from release-grade claims. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
