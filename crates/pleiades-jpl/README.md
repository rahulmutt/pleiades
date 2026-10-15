# pleiades-jpl

[![crates.io](https://img.shields.io/crates/v/pleiades-jpl.svg)](https://crates.io/crates/pleiades-jpl)
[![docs.rs](https://img.shields.io/docsrs/pleiades-jpl)](https://docs.rs/pleiades-jpl)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Checked-in JPL Horizons reference snapshots and snapshot-backed validation helpers for the [pleiades](https://github.com/rahulmutt/pleiades) astrology workspace, plus pure-Rust CSV parsing entry points for JPL-style manifest/row corpora.

Depends on `pleiades-types` and `pleiades-backend`. This is a reference/validation fixture crate, not a broad public-data reader.

## Status

Experimental, pre-1.0: breaking changes can land in any minor release. The checked-in corpus is regression evidence, sparse relative to production-coverage goals; see the [workspace README](https://github.com/rahulmutt/pleiades#readme) for the full maturity posture.

`JplSnapshotBackend` answers at an exact fixture row, or between rows that lie
on both sides of the instant and span at most five days. Any other instant
returns `OutOfRangeInstant`. Use `SpkBackend` with a JPL kernel for positions
at arbitrary dates. The five-day interpolation is validated to 0.05″ for
the six asteroids, Apophis included: each row in the January 2001 cluster is
held out in turn and the rows around it reproduce it. The Sun and the planets
are interpolated under the same rule, within 0.3″ (the worst is Mercury). The
Moon is served only at an exact row: a cubic through rows a day apart
misplaces it by up to 80″. Its metadata says so: the accuracy class is
`High` rather than `Exact`, and `nominal_range` is the envelope from the first
row to the last, not continuous coverage.

Every row of the two snapshot fixtures agrees with JPL Horizons within 0.05″
as seen from the Earth (fetched 2026-10-15; the worst row is 0.0017″). The
check requests each asteroid row on its own: Horizons integrates a small body
from one listed epoch to the next, so a request listing several epochs answers
along a path that depends on the list, and for Apophis, whose Earth encounters
amplify it, that moves its 2001 rows by up to 85000 km. Apophis's rows at
JD 2500000 and 2634167 follow its nominal orbit past the 2029 encounter, which
the next orbit solution can move. To repeat the check, with network access:

```sh
cargo run -p pleiades-jpl --features horizons-fetch --bin check-snapshot-fixtures
```

## License

MIT OR Apache-2.0
