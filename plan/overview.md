# Plan Overview

`pleiades` has completed bootstrap and foundation work. The active plan tracks only remaining production gaps against the specification.

## Active phases

1. **Production reference/source corpus** — build or ingest public, reproducible source inputs broad enough for validation and artifact generation.
2. **Production compressed ephemeris** — replace the draft packaged-data fixture with a release-grade 1500-2500 CE artifact.
3. **Body/backend claim completion** — settle accuracy and release status for Pluto, lunar theory/lunar points, selected asteroids, and backend capability matrices.
4. **Advanced request modes** — implement or consistently reject UTC/Delta-T modeling, apparent-place corrections, topocentric body positions, and native sidereal output.
5. **Compatibility and release readiness** — audit house/ayanamsa evidence and make release gates fail closed on stale or overstated claims.

## Current priority

Phase 1 is the execution frontier. Phase 2 can improve generation algorithms in parallel, but the artifact must remain draft-grade until source coverage and hold-out validation are production-ready.

## Cross-cutting rules

- Preserve pure-Rust, layered crate boundaries from `spec/architecture.md`.
- Keep unsupported modes as structured errors until implemented and validated.
- Keep release compatibility profiles truthful about exact built-ins, aliases, constraints, and known gaps.
- Keep validation/report artifacts generated from current code and source inputs, not manually-maintained prose.
