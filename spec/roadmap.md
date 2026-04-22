# Roadmap

## Phase 1: Foundations

- create workspace and crate skeletons
- define shared domain/time/body types
- define backend trait and metadata model
- implement basic façade in `pleiades-core`

## Phase 2: Algorithmic Baseline

- implement `pleiades-vsop87` for major planets and solar coordinates
- implement `pleiades-elp` for lunar coordinates
- implement first ayanamsa catalog
- implement the initial mandatory house-system catalog, leaving room for later expansion to the broader house-system ecosystem

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

## Phase 6: Expansion

- composite backend routing
- more asteroid coverage
- topocentric refinements
- optional higher-level chart utilities
