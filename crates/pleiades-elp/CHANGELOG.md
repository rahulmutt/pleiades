# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.5.9] - 2026-10-23

### Fixed

- Route transcendentals through pure-Rust libm so pins don't depend on host glibc ([#288](https://github.com/rahulmutt/pleiades/pull/288)) ([#289](https://github.com/rahulmutt/pleiades/pull/289)) ([a5c5600](https://github.com/rahulmutt/pleiades/commit/a5c56008cd60deda01c455a0331a8803bdda708d))

## [0.5.8] - 2026-10-18

### Added

- Generate README capability blocks from backend metadata and audit them ([#166](https://github.com/rahulmutt/pleiades/pull/166)) ([#267](https://github.com/rahulmutt/pleiades/pull/267)) ([feb59f6](https://github.com/rahulmutt/pleiades/commit/feb59f628ebd0ccfe57a54b96e478de54bc6fa72))

### Fixed

- Compute the Earth-Moon μ from its derivation; the osculating apogee now matches Swiss Ephemeris to 1.8″ ([#168](https://github.com/rahulmutt/pleiades/pull/168)) ([#271](https://github.com/rahulmutt/pleiades/pull/271)) ([fbce23b](https://github.com/rahulmutt/pleiades/commit/fbce23b341aeeba1e610ae3473b754161e73450a))

## [0.5.7] - 2026-10-08

_No user-facing changes; released for a dependency or manifest update._

## [0.5.6] - 2026-10-07

### Fixed

- Refuse non-finite instants; correct the snapshot notes and summaries; document ELP's equatorial frame (#201, #171) ([#211](https://github.com/rahulmutt/pleiades/pull/211)) ([0eea1bf](https://github.com/rahulmutt/pleiades/commit/0eea1bf23a00dedaf25e9bc4d559cbb9ecbc60e4))

## [0.5.5] - 2026-10-05

### Fixed

- Serve TrueNode as the osculating node of the ELP Moon ([#127](https://github.com/rahulmutt/pleiades/pull/127)) ([#133](https://github.com/rahulmutt/pleiades/pull/133)) ([8f5c2b7](https://github.com/rahulmutt/pleiades/commit/8f5c2b78edbfc23a225d542ebc3f4c3e6595f442))
- Difference the speed over a short step ([#140](https://github.com/rahulmutt/pleiades/pull/140)) ([#150](https://github.com/rahulmutt/pleiades/pull/150)) ([b487377](https://github.com/rahulmutt/pleiades/commit/b487377f4a049d7369783014ab73f03bf4760fb9))

### Performance

- Motion-free mean-place reads and a backend-free chart aberration Sun (FU-25, #128) ([#149](https://github.com/rahulmutt/pleiades/pull/149)) ([457497d](https://github.com/rahulmutt/pleiades/commit/457497dd534e26862e8b0a81f7aa1c0ab741c85f))

## [0.5.4] - 2026-10-03

### Added

- Mean lunar points on the packaged backend ([#90](https://github.com/rahulmutt/pleiades/pull/90)) ([#97](https://github.com/rahulmutt/pleiades/pull/97)) ([ac4cb8c](https://github.com/rahulmutt/pleiades/commit/ac4cb8c99035b82b08d8d32fbe74f950d45149cb))

## [0.5.3] - 2026-09-28

_No user-facing changes; released for a dependency or manifest update._
