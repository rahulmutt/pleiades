# pleiades-vsop87

[![crates.io](https://img.shields.io/crates/v/pleiades-vsop87.svg)](https://crates.io/crates/pleiades-vsop87)
[![docs.rs](https://img.shields.io/docsrs/pleiades-vsop87)](https://docs.rs/pleiades-vsop87)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Pure-Rust VSOP87B planetary position backend for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, with generated binary coefficient tables and a Meeus Table 37.A Pluto path.

Depends on `pleiades-types` and `pleiades-backend`. The crate ships its generated coefficient tables plus the raw VSOP87B source tables and the `regenerate-vsop87b-tables` tool used to rebuild them.

## Capabilities

Generated from the backend's metadata (`mise run readme-capabilities`):

<!-- audit:capabilities pleiades-vsop87 -->
- Coordinates: mean geometric; apparent requests are rejected
- Observer: geocentric; topocentric requests are rejected
- Frames: ecliptic, equatorial · Time scales: TT, TDB
- Window: no nominal limit
- Release-grade bodies: none
- Constrained bodies (Moderate): Sun, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune, Pluto
<!-- /audit:capabilities -->

## Truncated series inside 1600–2600

The default `Vsop87Backend` evaluates truncated VSOP87B series for instants in 1600-01-01..2600-01-01 TT (start inclusive, end exclusive; the window is applied to the request's Julian day whatever its scale, TT or TDB): terms with `|A|·0.6^α < 1e-8` are dropped, which keeps every body within 0.3″ of the full series (measured on a 200,000-instant scan of the window: Mars is the worst, at 0.26″ in longitude near 2508) and makes each apparent sample 2.2–9.5× cheaper per body, a Sun–Pluto set 3.4× (measured 2026-10-19). Inside the window the VSOP87B bodies report `Approximate` quality; Pluto keeps its usual quality, although the Earth subtracted from it is truncated too (about 0.001″). Outside the window the full series is evaluated, so output steps at 1600-01-01 and 2600-01-01 by under 0.1″ (measured 0.085″, Venus, at 1600; 0.013″ at 2600).

`Vsop87FullSeriesBackend` evaluates every term everywhere. It reproduces the published series exactly and has no edge step, at the cost of speed inside the window; outside it both backends return identical results.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. Positions are geocentric, derived from the heliocentric VSOP87B series; the chart layer in `pleiades-core` applies apparent place and the topocentric correction on top. Pluto, which VSOP87 excludes, comes from the Meeus Table 37.A periodic-term fit over 1885–2099, within about 3″ of the packaged DE440-fitted Pluto; outside that window it falls back to mean Keplerian elements (about 0.6° from Swiss Ephemeris), with a jump of that size at each window edge. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## License

MIT OR Apache-2.0
