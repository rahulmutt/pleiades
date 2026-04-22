# Track 2 — Domain and Public API

## Purpose
Build stable shared semantics before the project accumulates backend-specific behavior.

## Scope

- time, angle, coordinate, and observer types
- body identifiers and extensible catalog models
- backend request/result types
- house-system and ayanamsa domain APIs
- `pleiades-core` facade design
- rustdoc examples and public error taxonomy

## Primary stages

- Stage 2 establishes the base types and contracts
- Stage 3 adds chart-facing domain behavior
- Stage 6 stabilizes and documents long-term API posture

## Key milestones

1. Shared types define units, normalization rules, and failure modes clearly.
2. Backend contracts support single and batch queries without assuming one backend family.
3. Domain-layer sidereal and house calculations sit above backend tropical coordinates when practical.
4. Public APIs are documented with examples and compatibility notes.

## Done criteria for work in this track

- no source-specific logic leaks into shared crates
- units and reference assumptions are explicit
- unsupported features and range limits are modeled as structured errors
- compatibility expansion does not require redesign of core public types

## Common failure modes

- stringly typed identifiers for stable catalog concepts
- convenience APIs that hide key astronomical assumptions
- public types that are too narrow for the target compatibility catalog
