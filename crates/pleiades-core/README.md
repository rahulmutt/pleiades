# pleiades-core

[![crates.io](https://img.shields.io/crates/v/pleiades-core.svg)](https://crates.io/crates/pleiades-core)
[![docs.rs](https://img.shields.io/docsrs/pleiades-core)](https://docs.rs/pleiades-core)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

High-level chart façade for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace: typed tropical/sidereal chart requests, request validation, compatibility and API-stability profiles, and re-exports for common consumers.

Sits at the top of the published `pleiades-*` library layering (types, backend, time, apparent, houses, ayanamsa, compression). Pair it with a backend crate such as `pleiades-data` (packaged 1900-2100 artifact) or `pleiades-vsop87` and `pleiades-elp` to compute positions.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. Charts default to apparent place of date (`ChartRequest::new` sets `Apparentness::Apparent`) and carry apparent equatorial coordinates; the topocentric correction is opt-in, and `ChartRequest::from_civil` converts a civil UTC or UT1 datetime through `pleiades-time`. The first-party backends themselves return mean geometric coordinates and reject apparent requests: the chart layer applies the corrections on top of them. A mean chart (`Apparentness::Mean`) reports that geometric place on the J2000 equinox in the tropical zodiac; in a sidereal zodiac it is precessed to the mean equinox of date before the ayanamsa comes off, as Swiss Ephemeris and `pleiades-events` do. See the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

## Apparent-star ayanamsa

A star-anchored ayanamsa (True Citra, Revati, Pushya, Mula, Sheoran, the Galactic Center modes other than Mardyks) is read from the anchor star's mean place by default. `ChartRequest::with_sidereal_star_place(SiderealStarPlace::Apparent)` reads it from the star's apparent place (light deflection and annual aberration, up to about 22″), as Swiss Ephemeris's default sidereal convention does. The default is unchanged; `ChartRequest` and `ChartSnapshot` gain the public field `sidereal_star_place`.

## License

MIT OR Apache-2.0
