# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.10.2] - 2026-10-23

### Fixed

- Route transcendentals through pure-Rust libm so pins don't depend on host glibc ([#288](https://github.com/rahulmutt/pleiades/pull/288)) ([#289](https://github.com/rahulmutt/pleiades/pull/289)) ([a5c5600](https://github.com/rahulmutt/pleiades/commit/a5c56008cd60deda01c455a0331a8803bdda708d))

## [0.10.1] - 2026-10-19

### Performance

- Truncate the VSOP87B series inside 1600–2600 ([#281](https://github.com/rahulmutt/pleiades/pull/281)) ([2c2a19a](https://github.com/rahulmutt/pleiades/commit/2c2a19a4d15c6b72e51b123f438a4ea22b181c16))

## [0.10.0] - 2026-10-18

### Breaking Changes

- Precess fictitious-body element equinoxes with Vondrák 2011; raise MSRV to 1.99.1 ([#279](https://github.com/rahulmutt/pleiades/pull/279)) ([4292c63](https://github.com/rahulmutt/pleiades/commit/4292c636e1a79c74521d294fbda906408ab8205f))

### Added

- Find aspects at several angles in one scan (#168 (e)) ([#266](https://github.com/rahulmutt/pleiades/pull/266)) ([18a0163](https://github.com/rahulmutt/pleiades/commit/18a016317bb4514d8bc46191bdc6091e10191a86))
- Gate true-node aspects against Swiss Ephemeris SWIEPH ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#269](https://github.com/rahulmutt/pleiades/pull/269)) ([0f7453f](https://github.com/rahulmutt/pleiades/commit/0f7453fa8f096c772593f0dcccc5b94e2423c5bd))
- Gate osculating-apogee and mean-point aspects against SWIEPH, with graze windows and their own nightly job ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#272](https://github.com/rahulmutt/pleiades/pull/272)) ([a12ec90](https://github.com/rahulmutt/pleiades/commit/a12ec909d9224ebe8af218db9fc60f9efaf97342))
- Gate fictitious-body aspects against SWIEPH in the mean-of-date frame ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#273](https://github.com/rahulmutt/pleiades/pull/273)) ([44e1f14](https://github.com/rahulmutt/pleiades/commit/44e1f14d65625efabe5c9965aa7cc20e82e7a40e))
- Check graze-window edge sides and require equal counts in non-tangent windows ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#277](https://github.com/rahulmutt/pleiades/pull/277)) ([3f11ed0](https://github.com/rahulmutt/pleiades/commit/3f11ed02d35dd991c62d529bb44aa4051104dff9))
- Gate osculating-apogee and fictitious-body stations against SWIEPH central-difference corpora ([#167](https://github.com/rahulmutt/pleiades/pull/167)) ([#278](https://github.com/rahulmutt/pleiades/pull/278)) ([b4ba022](https://github.com/rahulmutt/pleiades/commit/b4ba022815972b010ee7ce18227588df8a2fe8c2))

### Fixed

- Compute the Earth-Moon μ from its derivation; the osculating apogee now matches Swiss Ephemeris to 1.8″ ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#271](https://github.com/rahulmutt/pleiades/pull/271)) ([fbce23b](https://github.com/rahulmutt/pleiades/commit/fbce23b341aeeba1e610ae3473b754161e73450a))

### Performance

- Interpolate the third light-time step instead of querying the backend ([#247](https://github.com/rahulmutt/pleiades/pull/247)) ([#248](https://github.com/rahulmutt/pleiades/pull/248)) ([cc8e20b](https://github.com/rahulmutt/pleiades/commit/cc8e20bb3447d779002f82a16b76b5c65cc8a300))

## [0.9.0] - 2026-10-08

### Breaking Changes

- Dense packaged asteroid fits from sb441-n373s ([#201](https://github.com/rahulmutt/pleiades/pull/201)) ([#233](https://github.com/rahulmutt/pleiades/pull/233)) ([560d69c](https://github.com/rahulmutt/pleiades/commit/560d69cf9791eb61a6847a2dd18821d57b6269dc))
- Apply rise/set refraction as swe_rise_trans does; give Atmosphere SE's atpress/attemp meaning ([#242](https://github.com/rahulmutt/pleiades/pull/242)) ([#245](https://github.com/rahulmutt/pleiades/pull/245)) ([dfc2006](https://github.com/rahulmutt/pleiades/commit/dfc200674ebbf599c25b9aee4aa37dde6bcb423c))

### Fixed

- Place a no_ecl_lat body geocentrically, as SE_BIT_GEOCTR_NO_ECL_LAT does ([#241](https://github.com/rahulmutt/pleiades/pull/241)) ([#244](https://github.com/rahulmutt/pleiades/pull/244)) ([c8d10d8](https://github.com/rahulmutt/pleiades/commit/c8d10d8004551f162ab8be7aa688e2ae979d6b1e))

### Performance

- Answer at once for an aspect angle the pair cannot reach ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#231](https://github.com/rahulmutt/pleiades/pull/231)) ([0adf2b8](https://github.com/rahulmutt/pleiades/commit/0adf2b8ac8ae46560e9af421081a267b546f00cd))

## [0.8.0] - 2026-10-07

### Breaking Changes

- Swiss Ephemeris default parity for star-anchored ayanamsas ([#164](https://github.com/rahulmutt/pleiades/pull/164)) ([#222](https://github.com/rahulmutt/pleiades/pull/222)) ([37eacf4](https://github.com/rahulmutt/pleiades/commit/37eacf45600f62314c6a0df6ebc746723899dbfc))

### Added

- Let a station search take a finer scan step ([#167](https://github.com/rahulmutt/pleiades/pull/167)) ([#221](https://github.com/rahulmutt/pleiades/pull/221)) ([9fdb340](https://github.com/rahulmutt/pleiades/commit/9fdb34017c2ecc8e4b804a696851cb3c1b0e80b8))

### Fixed

- Search rise, set and transit up to the window's ends; report a search the window cuts short ([#203](https://github.com/rahulmutt/pleiades/pull/203)) ([#207](https://github.com/rahulmutt/pleiades/pull/207)) ([5e72ffb](https://github.com/rahulmutt/pleiades/commit/5e72ffbabc1cbdabb49fec1674ae1ee80e0ce275))
- Refuse non-finite instants; correct the snapshot notes and summaries; document ELP's equatorial frame (#201, #171) ([#211](https://github.com/rahulmutt/pleiades/pull/211)) ([0eea1bf](https://github.com/rahulmutt/pleiades/commit/0eea1bf23a00dedaf25e9bc4d559cbb9ecbc60e4))
- Search crossings, stations, aspects and occultations up to the window's ends; report a search the window cuts short ([#208](https://github.com/rahulmutt/pleiades/pull/208)) ([#212](https://github.com/rahulmutt/pleiades/pull/212)) ([5b23b1f](https://github.com/rahulmutt/pleiades/commit/5b23b1f9c1ed806793a8985ec34585854834a455))
- Answer a station search for a body that never stations at the window's start ([#213](https://github.com/rahulmutt/pleiades/pull/213)) ([#215](https://github.com/rahulmutt/pleiades/pull/215)) ([13595e3](https://github.com/rahulmutt/pleiades/commit/13595e335a597dad647d8095ef8573d578c8165d))

### Performance

- Read a rise/set search's body at a few lattice instants and interpolate between them ([#204](https://github.com/rahulmutt/pleiades/pull/204)) ([#209](https://github.com/rahulmutt/pleiades/pull/209)) ([a5bbc84](https://github.com/rahulmutt/pleiades/commit/a5bbc842f845f740a54a8d58a6760ec26034527c))
- Share rise/set samples across an engine's searches and refine with ITP ([#204](https://github.com/rahulmutt/pleiades/pull/204)) ([#220](https://github.com/rahulmutt/pleiades/pull/220)) ([49abca1](https://github.com/rahulmutt/pleiades/commit/49abca11489a0ed0e7ca9bae719f6d639c45055a))

## [0.7.3] - 2026-10-06

### Added

- Answer at once for bodies that never station; hold the true node's stations to half a day ([#167](https://github.com/rahulmutt/pleiades/pull/167)) ([#198](https://github.com/rahulmutt/pleiades/pull/198)) ([ba37e22](https://github.com/rahulmutt/pleiades/commit/ba37e22431aa4d6d1bfe362eccca4f63cc793d11))

### Fixed

- Decide aspect range ends by the separation there ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#193](https://github.com/rahulmutt/pleiades/pull/193)) ([9f69926](https://github.com/rahulmutt/pleiades/commit/9f69926c054742b5f6361fa9cd7c88e168a5ebc8))
- Select occultations by a maximum that does not move with the search ([#159](https://github.com/rahulmutt/pleiades/pull/159)) ([#194](https://github.com/rahulmutt/pleiades/pull/194)) ([5e3163b](https://github.com/rahulmutt/pleiades/commit/5e3163b30a3b96b888b3f7564d3a37db7554e401))
- Put a sidereal mean chart on the mean equinox of date ([#164](https://github.com/rahulmutt/pleiades/pull/164)) ([#199](https://github.com/rahulmutt/pleiades/pull/199)) ([030983c](https://github.com/rahulmutt/pleiades/commit/030983cd6fe5937521cc758667ecc10a196b8f6d))
- Refuse asteroid positions away from their sample rows ([#158](https://github.com/rahulmutt/pleiades/pull/158)) ([#202](https://github.com/rahulmutt/pleiades/pull/202)) ([45ae4e2](https://github.com/rahulmutt/pleiades/commit/45ae4e277ae9a618c3f247231ae3c4772a506d54))

## [0.7.2] - 2026-10-06

### Added

- Add previous_station ([#167](https://github.com/rahulmutt/pleiades/pull/167)) ([#187](https://github.com/rahulmutt/pleiades/pull/187)) ([daea37e](https://github.com/rahulmutt/pleiades/commit/daea37edd39f809ab71c1f925b6f840f77fb637f))
- Add previous_aspect ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#189](https://github.com/rahulmutt/pleiades/pull/189)) ([9bebf13](https://github.com/rahulmutt/pleiades/commit/9bebf1348f42d98f0341f29095fe7e31e0349346))
- Give event results a civil() datetime ([#165](https://github.com/rahulmutt/pleiades/pull/165)) ([#190](https://github.com/rahulmutt/pleiades/pull/190)) ([15eeb36](https://github.com/rahulmutt/pleiades/commit/15eeb36842a376ef86db2fa07b34762f6a055cf0))

### Fixed

- Fail nod_aps with a typed error for a point with no direction ([#161](https://github.com/rahulmutt/pleiades/pull/161)) ([#177](https://github.com/rahulmutt/pleiades/pull/177)) ([76be56b](https://github.com/rahulmutt/pleiades/commit/76be56bf9c7410db1c061a1077dabf350397dc56))
- Settle backward longitude crossings and range ends at the query instant ([#159](https://github.com/rahulmutt/pleiades/pull/159)) ([#181](https://github.com/rahulmutt/pleiades/pull/181)) ([0a29db9](https://github.com/rahulmutt/pleiades/commit/0a29db92c0e260dcf413c579146c932d7f0fcb9d))
- Sample a backward search's empty range inside the window ([#159](https://github.com/rahulmutt/pleiades/pull/159)) ([#186](https://github.com/rahulmutt/pleiades/pull/186)) ([de77718](https://github.com/rahulmutt/pleiades/commit/de777188cbabe9cbb95b35af2125985565c8fb53))
- Report an apparent read reaching before the window as OutOfWindow ([#163](https://github.com/rahulmutt/pleiades/pull/163)) ([#191](https://github.com/rahulmutt/pleiades/pull/191)) ([9e33099](https://github.com/rahulmutt/pleiades/commit/9e33099812568bfc7c9d5149460e0f64302e6833))

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
