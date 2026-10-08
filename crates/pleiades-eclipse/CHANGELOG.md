# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.8.0] - 2026-10-08

### Breaking Changes

- Apply rise/set refraction as swe_rise_trans does; give Atmosphere SE's atpress/attemp meaning ([#242](https://github.com/rahulmutt/pleiades/pull/242)) ([#245](https://github.com/rahulmutt/pleiades/pull/245)) ([dfc2006](https://github.com/rahulmutt/pleiades/commit/dfc200674ebbf599c25b9aee4aa37dde6bcb423c))

## [0.7.2] - 2026-10-07

_No user-facing changes; released for a dependency or manifest update._

## [0.7.1] - 2026-10-05

### Fixed

- Select eclipses_in_range by greatest eclipse, not syzygy ([#121](https://github.com/rahulmutt/pleiades/pull/121)) ([#125](https://github.com/rahulmutt/pleiades/pull/125)) ([e7f3b56](https://github.com/rahulmutt/pleiades/commit/e7f3b5678c309b684495e43f917dc678a23bb828))

### Performance

- Search outward from the query instant in next/previous_eclipse (FU-23 (q)) ([#143](https://github.com/rahulmutt/pleiades/pull/143)) ([83b91c6](https://github.com/rahulmutt/pleiades/commit/83b91c669bc393c984a01d1a41be02ac9ea50f03))
- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

## [0.7.0] - 2026-10-03

### Breaking Changes

- Public signatures carry pleiades-time 0.7 and pleiades-apparent 0.7 types ([#110](https://github.com/rahulmutt/pleiades/pull/110)) ([64dfc48](https://github.com/rahulmutt/pleiades/commit/64dfc48ac0bce0209445755b3a017de6c42e1994))

### Fixed

- Stop double-counting annual aberration on the light-time path ([#93](https://github.com/rahulmutt/pleiades/pull/93)) ([#94](https://github.com/rahulmutt/pleiades/pull/94)) ([54688d0](https://github.com/rahulmutt/pleiades/commit/54688d0b290378e2f3ad7921e8dae8e3d3e83f44))

## [0.6.0] - 2026-09-28

### Breaking Changes

- Take pleiades-apparent 0.6 Atmosphere in local_circumstances ([eca828b](https://github.com/rahulmutt/pleiades/commit/eca828b111536c3a0b85aa49e238d2ef7db0de49))
