# Roadmap

## Phase 1: Foundations

- create workspace and crate skeletons
- define shared domain/time/body types and compatibility-profile types
- define backend trait and metadata model
- implement basic façade in `pleiades-core`

## Phase 2: Algorithmic Baseline

- implement `pleiades-vsop87` for major planets and solar coordinates
- implement `pleiades-elp` for lunar coordinates
- implement the initial ayanamsa catalog and extensible registration model, with an explicit path to the target compatibility catalog
- implement the initial house-system milestone on top of an API designed for the target compatibility catalog
- keep sidereal conversion and house logic in domain crates rather than duplicating them in each backend

## Phase 3: Reference Data Backend

- implement `pleiades-jpl` parser/reader in pure Rust
- validate core bodies against source material
- add selected asteroid support through public data sources

## Phase 4: Compression and Packaged Distribution

- build fitting pipeline for 1500-2500
- implement artifact format in `pleiades-compression`
- ship `pleiades-data` packaged backend
- benchmark artifact size and lookup speed

## Phase 5: Validation and Hardening

- add broad regression corpus
- document backend capability matrices
- publish error statistics
- stabilize public API

## Phase 6: Expansion and Compatibility Completion

- composite backend routing
- more asteroid coverage
- complete the remaining house-system and ayanamsa entries needed to satisfy the target compatibility catalog
- publish and maintain the versioned compatibility profile for built-in systems, aliases, constraints, and milestone coverage
- topocentric refinements
- optional higher-level chart utilities
