# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.6.0] - 2026-10-08

### Breaking Changes

- Dense packaged asteroid fits from sb441-n373s ([#201](https://github.com/rahulmutt/pleiades/pull/201)) ([#233](https://github.com/rahulmutt/pleiades/pull/233)) ([560d69c](https://github.com/rahulmutt/pleiades/commit/560d69cf9791eb61a6847a2dd18821d57b6269dc))

### Fixed

- Return an error, not a panic, when a kernel does not cover the window ([#235](https://github.com/rahulmutt/pleiades/pull/235)) ([#238](https://github.com/rahulmutt/pleiades/pull/238)) ([e960482](https://github.com/rahulmutt/pleiades/commit/e9604826dc3f9b2788e53c3795edacc70c7156d1))
- Describe the dense per-body fits in the artifact header and span report ([#234](https://github.com/rahulmutt/pleiades/pull/234)) ([#243](https://github.com/rahulmutt/pleiades/pull/243)) ([b6ea7ef](https://github.com/rahulmutt/pleiades/commit/b6ea7ef81fba2634bf4f26cb00e5ae645ecaf8f6))

## [0.5.7] - 2026-10-07

### Fixed

- Put the equatorial channel and mean charts on J2000 ([#210](https://github.com/rahulmutt/pleiades/pull/210)) ([#217](https://github.com/rahulmutt/pleiades/pull/217)) ([215bc61](https://github.com/rahulmutt/pleiades/commit/215bc61e33dd82dd7b78150ead28f11e98df5681))

## [0.5.6] - 2026-10-06

### Fixed

- Refuse asteroid positions away from their sample rows ([#158](https://github.com/rahulmutt/pleiades/pull/158)) ([#202](https://github.com/rahulmutt/pleiades/pull/202)) ([45ae4e2](https://github.com/rahulmutt/pleiades/commit/45ae4e277ae9a618c3f247231ae3c4772a506d54))

## [0.5.5] - 2026-10-05

### Fixed

- Serve TrueNode as the osculating node of the ELP Moon ([#127](https://github.com/rahulmutt/pleiades/pull/127)) ([#133](https://github.com/rahulmutt/pleiades/pull/133)) ([8f5c2b7](https://github.com/rahulmutt/pleiades/commit/8f5c2b78edbfc23a225d542ebc3f4c3e6595f442))
- Richardson-extrapolate the derived lunar points' speed ([#108](https://github.com/rahulmutt/pleiades/pull/108)) ([#145](https://github.com/rahulmutt/pleiades/pull/145)) ([91b1329](https://github.com/rahulmutt/pleiades/commit/91b13290d49cbd35cf0118607012f978b016423e))

## [0.5.4] - 2026-10-03

### Added

- Mean lunar points on the packaged backend ([#90](https://github.com/rahulmutt/pleiades/pull/90)) ([#97](https://github.com/rahulmutt/pleiades/pull/97)) ([ac4cb8c](https://github.com/rahulmutt/pleiades/commit/ac4cb8c99035b82b08d8d32fbe74f950d45149cb))

## [0.5.3] - 2026-09-28

_No user-facing changes; released for a dependency or manifest update._
