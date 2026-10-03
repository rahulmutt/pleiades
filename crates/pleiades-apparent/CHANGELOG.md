# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.7.0] - 2026-10-03

### Breaking Changes

- Public signatures carry pleiades-time 0.7 and pleiades-apparent 0.7 types ([#110](https://github.com/rahulmutt/pleiades/pull/110)) ([64dfc48](https://github.com/rahulmutt/pleiades/commit/64dfc48ac0bce0209445755b3a017de6c42e1994))

### Added

- Public ecliptic position with latitude and speed ([#89](https://github.com/rahulmutt/pleiades/pull/89)) ([#98](https://github.com/rahulmutt/pleiades/pull/98)) ([23dcb14](https://github.com/rahulmutt/pleiades/commit/23dcb1407bf8e81801647cfdfff6aaf3188a8d27))

### Fixed

- Stop double-counting annual aberration on the light-time path ([#93](https://github.com/rahulmutt/pleiades/pull/93)) ([#94](https://github.com/rahulmutt/pleiades/pull/94)) ([54688d0](https://github.com/rahulmutt/pleiades/commit/54688d0b290378e2f3ad7921e8dae8e3d3e83f44))

## [0.6.0] - 2026-09-28

### Breaking Changes

- Return pleiades-time 0.6 CivilTimeError from ut1_instant ([9fabbcb](https://github.com/rahulmutt/pleiades/commit/9fabbcbb5c7a729ceaa4155b43911c1961ca041b))
