# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.5.7] - 2026-10-07

### Fixed

- Replace the snapshot rows that are not Horizons positions; serve the Moon only at a row ([#200](https://github.com/rahulmutt/pleiades/pull/200)) ([#205](https://github.com/rahulmutt/pleiades/pull/205)) ([6ce4f37](https://github.com/rahulmutt/pleiades/commit/6ce4f37e16dfa07c0e2ab79f3bf1e04616bc5cd4))
- Refuse non-finite instants; correct the snapshot notes and summaries; document ELP's equatorial frame (#201, #171) ([#211](https://github.com/rahulmutt/pleiades/pull/211)) ([0eea1bf](https://github.com/rahulmutt/pleiades/commit/0eea1bf23a00dedaf25e9bc4d559cbb9ecbc60e4))
- Put the equatorial channel and mean charts on J2000 ([#210](https://github.com/rahulmutt/pleiades/pull/210)) ([#217](https://github.com/rahulmutt/pleiades/pull/217)) ([215bc61](https://github.com/rahulmutt/pleiades/commit/215bc61e33dd82dd7b78150ead28f11e98df5681))

## [0.5.6] - 2026-10-06

### Fixed

- Refuse asteroid positions away from their sample rows ([#158](https://github.com/rahulmutt/pleiades/pull/158)) ([#202](https://github.com/rahulmutt/pleiades/pull/202)) ([45ae4e2](https://github.com/rahulmutt/pleiades/commit/45ae4e277ae9a618c3f247231ae3c4772a506d54))

## [0.5.5] - 2026-10-05

### Fixed

- Bound SPK segment evaluation by the segment's own size; add spk_evaluate fuzz target ([#29](https://github.com/rahulmutt/pleiades/pull/29)) ([#137](https://github.com/rahulmutt/pleiades/pull/137)) ([fb3b774](https://github.com/rahulmutt/pleiades/commit/fb3b774122eaef7e103a618efddf50e1b878f54c))

## [0.5.4] - 2026-10-03

_No user-facing changes; released for a dependency or manifest update._

## [0.5.3] - 2026-09-28

_No user-facing changes; released for a dependency or manifest update._
