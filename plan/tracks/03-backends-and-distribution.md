# Track 3 — Backends and Distribution

## Purpose
Deliver multiple interchangeable ephemeris sources and a practical packaged-data path.

## Scope

- algorithmic backends such as `pleiades-vsop87` and `pleiades-elp`
- source-backed backends such as `pleiades-jpl`
- composite routing strategies
- compressed artifact generation and decode
- packaged offline distribution in `pleiades-data`

## Primary stages

- Stage 3 delivers the first useful algorithmic chart path
- Stage 4 adds a stronger source-backed backend
- Stage 5 adds compressed artifacts and packaged distribution
- Stage 6 broadens catalog and backend coverage

## Key milestones

1. A pure-Rust algorithmic path can generate practical charts.
2. A source-backed backend provides validation-grade reference output.
3. A packaged backend serves the common 1500-2500 range efficiently.
4. Composite routing works without changing consumer-facing APIs.

## Done criteria for work in this track

- every backend declares capabilities and coverage explicitly
- provenance and expected accuracy are documented
- packaged artifacts are reproducible from public inputs
- fallback behavior is explicit when a backend lacks coverage

## Common failure modes

- coupling domain code to one backend implementation
- expanding body support faster than validation supports
- shipping packaged data without reproducible generation metadata
