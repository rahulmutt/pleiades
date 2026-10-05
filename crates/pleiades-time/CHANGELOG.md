# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.7.1] - 2026-10-05

### Added

- Civil_from_tt and civil_from_tdb pick UTC or UT1 by era ([#107](https://github.com/rahulmutt/pleiades/pull/107)) ([#153](https://github.com/rahulmutt/pleiades/pull/153)) ([e8fe718](https://github.com/rahulmutt/pleiades/commit/e8fe718a90e70facdce4e419f208e77a2eb3dedc))

## [0.7.0] - 2026-10-03

### Breaking Changes

- Civil datetime from a TT or TDB instant, and a correct leap-second forward ([#87](https://github.com/rahulmutt/pleiades/pull/87)) ([#100](https://github.com/rahulmutt/pleiades/pull/100)) ([96adcd5](https://github.com/rahulmutt/pleiades/commit/96adcd5ef1b67d2971f64d88bc33de624b7e9568))

### Fixed

- Leap-second horizon follows IERS Bulletin C 72, and UTC past it holds the last offset ([#105](https://github.com/rahulmutt/pleiades/pull/105)) ([#109](https://github.com/rahulmutt/pleiades/pull/109)) ([72383d4](https://github.com/rahulmutt/pleiades/commit/72383d47177151ee226e6f428d2633e2cdab7997))

## [0.6.0] - 2026-09-28

### Fixed

- Bound ΔT by the leap-second table past the observed nodes (FU-11 item 2) ([#77](https://github.com/rahulmutt/pleiades/pull/77)) ([f36c13c](https://github.com/rahulmutt/pleiades/commit/f36c13cc22db2efe818a82ed6a4298da820f5085))
