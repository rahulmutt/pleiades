# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.7.1] - 2026-10-05

### Added

- Serve a sidereal zodiac in the heliocentric frame ([#106](https://github.com/rahulmutt/pleiades/pull/106)) ([#156](https://github.com/rahulmutt/pleiades/pull/156)) ([3908371](https://github.com/rahulmutt/pleiades/commit/3908371c946d91271e975b4450d89dd380c2e664))

### Fixed

- Sidereal apparent placements kept nutation ([#120](https://github.com/rahulmutt/pleiades/pull/120)) ([#123](https://github.com/rahulmutt/pleiades/pull/123)) ([fdef5f6](https://github.com/rahulmutt/pleiades/commit/fdef5f698de303a4951c139a7477586060349a9b))
- Evaluate lunar orbit points as directions in the geocentric frames ([#118](https://github.com/rahulmutt/pleiades/pull/118)) ([#124](https://github.com/rahulmutt/pleiades/pull/124)) ([f1aaece](https://github.com/rahulmutt/pleiades/commit/f1aaece876210e44a24a508432641ba619b49f8a))
- Serve TrueNode as the osculating node of the ELP Moon ([#127](https://github.com/rahulmutt/pleiades/pull/127)) ([#133](https://github.com/rahulmutt/pleiades/pull/133)) ([8f5c2b7](https://github.com/rahulmutt/pleiades/commit/8f5c2b78edbfc23a225d542ebc3f4c3e6595f442))
- Richardson-extrapolate the derived lunar points' speed ([#108](https://github.com/rahulmutt/pleiades/pull/108)) ([#145](https://github.com/rahulmutt/pleiades/pull/145)) ([91b1329](https://github.com/rahulmutt/pleiades/commit/91b13290d49cbd35cf0118607012f978b016423e))
- Difference the speed over a short step ([#140](https://github.com/rahulmutt/pleiades/pull/140)) ([#150](https://github.com/rahulmutt/pleiades/pull/150)) ([b487377](https://github.com/rahulmutt/pleiades/commit/b487377f4a049d7369783014ab73f03bf4760fb9))
- Report the sidereal speed in a sidereal chart ([#141](https://github.com/rahulmutt/pleiades/pull/141)) ([#151](https://github.com/rahulmutt/pleiades/pull/151)) ([500a1fb](https://github.com/rahulmutt/pleiades/commit/500a1fb22ef4aa1ae950fc75d26d58fd7e70567f))

### Performance

- Stop re-reading the Sun and the sampled body in apparent samples ([#128](https://github.com/rahulmutt/pleiades/pull/128)) ([#134](https://github.com/rahulmutt/pleiades/pull/134)) ([369caf4](https://github.com/rahulmutt/pleiades/commit/369caf427f6433605a656f024ffd32184d938b3b))
- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

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
