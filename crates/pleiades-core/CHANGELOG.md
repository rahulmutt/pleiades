# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.8.1] - 2026-10-06

### Fixed

- Take nutation off sidereal house cusps and angles ([#157](https://github.com/rahulmutt/pleiades/pull/157)) ([#175](https://github.com/rahulmutt/pleiades/pull/175)) ([b5aeaea](https://github.com/rahulmutt/pleiades/commit/b5aeaea8525ce01bc90f4aac7c8e3c8a0b449a8e))
- Report a failed apparent reduction at the snapshot level ([#170](https://github.com/rahulmutt/pleiades/pull/170)) ([#176](https://github.com/rahulmutt/pleiades/pull/176)) ([da25051](https://github.com/rahulmutt/pleiades/commit/da250512f50054eda685ebe6809bd6788c8c12fe))
- Put sidereal Whole Sign and Equal (1=Aries) cusps on sidereal sign boundaries ([#180](https://github.com/rahulmutt/pleiades/pull/180)) ([#183](https://github.com/rahulmutt/pleiades/pull/183)) ([f646409](https://github.com/rahulmutt/pleiades/commit/f646409c1647da36be0bf8adf4ff20fbaad995f5))
- Assign a body's house from the longitude its placement reports ([#182](https://github.com/rahulmutt/pleiades/pull/182)) ([#184](https://github.com/rahulmutt/pleiades/pull/184)) ([06d2984](https://github.com/rahulmutt/pleiades/commit/06d29848a508d7b305d58dbcc7840269c2a76780))
- Report the ascmc chart points in a sidereal chart's own zodiac ([#179](https://github.com/rahulmutt/pleiades/pull/179)) ([#185](https://github.com/rahulmutt/pleiades/pull/185)) ([2508022](https://github.com/rahulmutt/pleiades/commit/2508022dc71d904cb9428891a4730154b75cf678))

## [0.8.0] - 2026-10-05

### Breaking Changes

- Serve Pluto from the Meeus Table 37.A periodic-term fit ([#129](https://github.com/rahulmutt/pleiades/pull/129)) ([#136](https://github.com/rahulmutt/pleiades/pull/136)) ([daaae01](https://github.com/rahulmutt/pleiades/commit/daaae01183dd454aae2ca058cbe0b23bb3516d28))

### Added

- Serve a sidereal zodiac in the heliocentric frame ([#106](https://github.com/rahulmutt/pleiades/pull/106)) ([#156](https://github.com/rahulmutt/pleiades/pull/156)) ([3908371](https://github.com/rahulmutt/pleiades/commit/3908371c946d91271e975b4450d89dd380c2e664))

### Fixed

- Reduce every served body to apparent place, whatever its claim tier ([#113](https://github.com/rahulmutt/pleiades/pull/113)) ([#114](https://github.com/rahulmutt/pleiades/pull/114)) ([90c57c4](https://github.com/rahulmutt/pleiades/commit/90c57c4c785067a3d3af5198badfe5ee830cb1f2))
- Evaluate the Pluto mean-element orbit in radians and subtract the VSOP87B Earth ([#119](https://github.com/rahulmutt/pleiades/pull/119)) ([#122](https://github.com/rahulmutt/pleiades/pull/122)) ([b64d0e9](https://github.com/rahulmutt/pleiades/commit/b64d0e9edafed05d4c2ef223bf9ad9f3d0711243))
- Sidereal apparent placements kept nutation ([#120](https://github.com/rahulmutt/pleiades/pull/120)) ([#123](https://github.com/rahulmutt/pleiades/pull/123)) ([fdef5f6](https://github.com/rahulmutt/pleiades/commit/fdef5f698de303a4951c139a7477586060349a9b))
- Serve TrueNode as the osculating node of the ELP Moon ([#127](https://github.com/rahulmutt/pleiades/pull/127)) ([#133](https://github.com/rahulmutt/pleiades/pull/133)) ([8f5c2b7](https://github.com/rahulmutt/pleiades/commit/8f5c2b78edbfc23a225d542ebc3f4c3e6595f442))
- Difference the speed over a short step ([#140](https://github.com/rahulmutt/pleiades/pull/140)) ([#150](https://github.com/rahulmutt/pleiades/pull/150)) ([b487377](https://github.com/rahulmutt/pleiades/commit/b487377f4a049d7369783014ab73f03bf4760fb9))
- Report the sidereal speed in a sidereal chart ([#141](https://github.com/rahulmutt/pleiades/pull/141)) ([#151](https://github.com/rahulmutt/pleiades/pull/151)) ([500a1fb](https://github.com/rahulmutt/pleiades/commit/500a1fb22ef4aa1ae950fc75d26d58fd7e70567f))

### Performance

- Stop re-reading the Sun and the sampled body in apparent samples ([#128](https://github.com/rahulmutt/pleiades/pull/128)) ([#134](https://github.com/rahulmutt/pleiades/pull/134)) ([369caf4](https://github.com/rahulmutt/pleiades/commit/369caf427f6433605a656f024ffd32184d938b3b))
- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

## [0.7.0] - 2026-10-03

### Breaking Changes

- Civil datetime from a TT or TDB instant, and a correct leap-second forward ([#87](https://github.com/rahulmutt/pleiades/pull/87)) ([#100](https://github.com/rahulmutt/pleiades/pull/100)) ([96adcd5](https://github.com/rahulmutt/pleiades/commit/96adcd5ef1b67d2971f64d88bc33de624b7e9568))
- Planetary station finder, gated against a Swiss Ephemeris speed-zero corpus ([#85](https://github.com/rahulmutt/pleiades/pull/85)) ([#102](https://github.com/rahulmutt/pleiades/pull/102)) ([e6e3132](https://github.com/rahulmutt/pleiades/commit/e6e3132c4adf2ef4b5faadb87639ab2be890835e))

### Added

- Mean lunar points on the packaged backend ([#90](https://github.com/rahulmutt/pleiades/pull/90)) ([#97](https://github.com/rahulmutt/pleiades/pull/97)) ([ac4cb8c](https://github.com/rahulmutt/pleiades/commit/ac4cb8c99035b82b08d8d32fbe74f950d45149cb))
- Public ecliptic position with latitude and speed ([#89](https://github.com/rahulmutt/pleiades/pull/89)) ([#98](https://github.com/rahulmutt/pleiades/pull/98)) ([23dcb14](https://github.com/rahulmutt/pleiades/commit/23dcb1407bf8e81801647cfdfff6aaf3188a8d27))
- Longitude crossings in a mean place or a sidereal zodiac ([#88](https://github.com/rahulmutt/pleiades/pull/88)) ([#99](https://github.com/rahulmutt/pleiades/pull/99)) ([a617d35](https://github.com/rahulmutt/pleiades/commit/a617d353ad12e5b12782bbbdea05448c8bd31f51))
- Exact-aspect event finder, gated against a Swiss Ephemeris corpus ([#84](https://github.com/rahulmutt/pleiades/pull/84)) ([#103](https://github.com/rahulmutt/pleiades/pull/103)) ([06b35f0](https://github.com/rahulmutt/pleiades/commit/06b35f05154b864145b43807ec2b575bd6dc4f7a))

### Fixed

- Report the apparent-place speed for apparent chart placements ([#91](https://github.com/rahulmutt/pleiades/pull/91)) ([#92](https://github.com/rahulmutt/pleiades/pull/92)) ([401d631](https://github.com/rahulmutt/pleiades/commit/401d631de1dbf9b61169d0a5daffa60daef2a052))
- Stop double-counting annual aberration on the light-time path ([#93](https://github.com/rahulmutt/pleiades/pull/93)) ([#94](https://github.com/rahulmutt/pleiades/pull/94)) ([54688d0](https://github.com/rahulmutt/pleiades/commit/54688d0b290378e2f3ad7921e8dae8e3d3e83f44))

## [0.6.0] - 2026-09-28

### Breaking Changes

- Re-export pleiades-time 0.6 and pleiades-apparent 0.6 types ([71009ea](https://github.com/rahulmutt/pleiades/commit/71009ea1d64608cde9873facd1d62e7dd28dfd1f))

### Fixed

- Honour the TimeScale tag on rise/set/transit and horizontal instants ([#74](https://github.com/rahulmutt/pleiades/pull/74)) ([#75](https://github.com/rahulmutt/pleiades/pull/75)) ([e41fe87](https://github.com/rahulmutt/pleiades/commit/e41fe879c8c77e2e292429a3698620b091f84227))
