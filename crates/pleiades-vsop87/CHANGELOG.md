# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.6.1] - 2026-10-07

### Fixed

- Put the equatorial channel and mean charts on J2000 ([#210](https://github.com/rahulmutt/pleiades/pull/210)) ([#217](https://github.com/rahulmutt/pleiades/pull/217)) ([215bc61](https://github.com/rahulmutt/pleiades/commit/215bc61e33dd82dd7b78150ead28f11e98df5681))

## [0.6.0] - 2026-10-05

### Breaking Changes

- Serve Pluto from the Meeus Table 37.A periodic-term fit ([#129](https://github.com/rahulmutt/pleiades/pull/129)) ([#136](https://github.com/rahulmutt/pleiades/pull/136)) ([daaae01](https://github.com/rahulmutt/pleiades/commit/daaae01183dd454aae2ca058cbe0b23bb3516d28))

### Fixed

- Evaluate the Pluto mean-element orbit in radians and subtract the VSOP87B Earth ([#119](https://github.com/rahulmutt/pleiades/pull/119)) ([#122](https://github.com/rahulmutt/pleiades/pull/122)) ([b64d0e9](https://github.com/rahulmutt/pleiades/commit/b64d0e9edafed05d4c2ef223bf9ad9f3d0711243))
- Difference the speed over a short step ([#140](https://github.com/rahulmutt/pleiades/pull/140)) ([#150](https://github.com/rahulmutt/pleiades/pull/150)) ([b487377](https://github.com/rahulmutt/pleiades/commit/b487377f4a049d7369783014ab73f03bf4760fb9))

### Performance

- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

## [0.5.4] - 2026-10-03

_No user-facing changes; released for a dependency or manifest update._

## [0.5.3] - 2026-09-28

_No user-facing changes; released for a dependency or manifest update._
