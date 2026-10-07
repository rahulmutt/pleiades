# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.8.0] - 2026-10-07

### Breaking Changes

- Swiss Ephemeris default parity for star-anchored ayanamsas ([#164](https://github.com/rahulmutt/pleiades/pull/164)) ([#222](https://github.com/rahulmutt/pleiades/pull/222)) ([37eacf4](https://github.com/rahulmutt/pleiades/commit/37eacf45600f62314c6a0df6ebc746723899dbfc))

## [0.7.1] - 2026-10-05

### Fixed

- Serve TrueNode as the osculating node of the ELP Moon ([#127](https://github.com/rahulmutt/pleiades/pull/127)) ([#133](https://github.com/rahulmutt/pleiades/pull/133)) ([8f5c2b7](https://github.com/rahulmutt/pleiades/commit/8f5c2b78edbfc23a225d542ebc3f4c3e6595f442))

### Performance

- Stop re-reading the Sun and the sampled body in apparent samples ([#128](https://github.com/rahulmutt/pleiades/pull/128)) ([#134](https://github.com/rahulmutt/pleiades/pull/134)) ([369caf4](https://github.com/rahulmutt/pleiades/commit/369caf427f6433605a656f024ffd32184d938b3b))
- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

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
