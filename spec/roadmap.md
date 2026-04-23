# Roadmap

## Phase 1: Foundations

- create the workspace and crate skeletons
- define shared domain, time, body, and compatibility-profile types
- define the backend trait and metadata model
- implement a basic façade in `pleiades-core`

## Phase 2: Domain Baseline

- implement `pleiades-vsop87` for major planets and solar coordinates
- implement `pleiades-elp` for lunar coordinates
- implement the baseline ayanamsa milestone on top of an extensible catalog model
- implement the baseline house-system milestone on top of an extensible house API
- keep sidereal conversion and house logic in domain crates rather than duplicating them in each backend

## Phase 3: Reference Backend

- implement a pure-Rust `pleiades-jpl` reader or parser
- validate core bodies against source material
- add selected asteroid support through public data sources

## Phase 4: Compression and Distribution

- build the fitting pipeline for 1500-2500 CE
- implement the artifact format in `pleiades-compression`
- ship `pleiades-data` as the packaged backend
- benchmark artifact size and lookup speed

## Phase 5: Validation and Hardening

- add a broad regression corpus
- document backend capability matrices
- publish error statistics
- publish the first release compatibility profile
- stabilize the public API

## Phase 6: Compatibility Completion and Expansion

- add composite backend routing where useful
- expand asteroid coverage
- complete the remaining house-system and ayanamsa entries needed for the target compatibility catalog
- maintain release compatibility profiles with aliases, constraints, and known gaps
- refine topocentric support
- add optional higher-level chart utilities
