# Track 2 — Domain and Public API

## Role

Protect the typed, backend-agnostic public API while remaining phases add production accuracy, catalog evidence, artifact distribution, and release guarantees.

## Standards

- Keep shared domain vocabulary in `pleiades-types`.
- Keep backend contracts in `pleiades-backend` and source-specific calculations in backend crates.
- Keep house, ayanamsa, sidereal conversion, sign placement, house placement, aspects, and chart assembly out of source-specific backends.
- Prefer extensible identifiers, descriptors, validation helpers, and compatibility-profile metadata over breaking enum churn.
- Document units, frames, normalization, time scales, coordinate assumptions, and failure modes for public APIs.
- Keep unsupported advanced request modes as structured errors until implemented and validated.

## Remaining domain/API concerns

- Resolve or explicitly defer built-in UTC/Delta-T convenience, apparent-place corrections, topocentric body positions, and native sidereal backend output.
- Validate release-advertised house and ayanamsa formulas/aliases or mark them constrained, descriptor-only, custom-only, approximate, or unsupported.
- Keep custom body, house, and ayanamsa identifiers distinguishable from built-ins in profiles, summaries, and serialization.
- Ensure release profiles truthfully separate baseline guarantees, release additions, known gaps, constraints, and unsupported modes.
