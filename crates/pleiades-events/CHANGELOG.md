# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.7.0] - 2026-10-03

### Breaking Changes

- Planetary station finder, gated against a Swiss Ephemeris speed-zero corpus ([#85](https://github.com/rahulmutt/pleiades/pull/85)) ([#102](https://github.com/rahulmutt/pleiades/pull/102)) ([e6e3132](https://github.com/rahulmutt/pleiades/commit/e6e3132c4adf2ef4b5faadb87639ab2be890835e))

### Added

- Mean lunar points on the packaged backend ([#90](https://github.com/rahulmutt/pleiades/pull/90)) ([#97](https://github.com/rahulmutt/pleiades/pull/97)) ([ac4cb8c](https://github.com/rahulmutt/pleiades/commit/ac4cb8c99035b82b08d8d32fbe74f950d45149cb))
- Public ecliptic position with latitude and speed ([#89](https://github.com/rahulmutt/pleiades/pull/89)) ([#98](https://github.com/rahulmutt/pleiades/pull/98)) ([23dcb14](https://github.com/rahulmutt/pleiades/commit/23dcb1407bf8e81801647cfdfff6aaf3188a8d27))
- Longitude crossings in a mean place or a sidereal zodiac ([#88](https://github.com/rahulmutt/pleiades/pull/88)) ([#99](https://github.com/rahulmutt/pleiades/pull/99)) ([a617d35](https://github.com/rahulmutt/pleiades/commit/a617d353ad12e5b12782bbbdea05448c8bd31f51))
- Exact-aspect event finder, gated against a Swiss Ephemeris corpus ([#84](https://github.com/rahulmutt/pleiades/pull/84)) ([#103](https://github.com/rahulmutt/pleiades/pull/103)) ([06b35f0](https://github.com/rahulmutt/pleiades/commit/06b35f05154b864145b43807ec2b575bd6dc4f7a))

### Fixed

- Chained rise/set/transit searches step past returned events and find short nights (#80, #81) ([#82](https://github.com/rahulmutt/pleiades/pull/82)) ([9453c5d](https://github.com/rahulmutt/pleiades/commit/9453c5d968b29a9f376bd897e856acc1da144026))
- Stop double-counting annual aberration on the light-time path ([#93](https://github.com/rahulmutt/pleiades/pull/93)) ([#94](https://github.com/rahulmutt/pleiades/pull/94)) ([54688d0](https://github.com/rahulmutt/pleiades/commit/54688d0b290378e2f3ad7921e8dae8e3d3e83f44))

## [0.6.0] - 2026-09-28

### Fixed

- Honour the TimeScale tag on rise/set/transit and horizontal instants ([#74](https://github.com/rahulmutt/pleiades/pull/74)) ([#75](https://github.com/rahulmutt/pleiades/pull/75)) ([e41fe87](https://github.com/rahulmutt/pleiades/commit/e41fe879c8c77e2e292429a3698620b091f84227))
