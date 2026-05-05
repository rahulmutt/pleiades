# Track 2 — Domain and Public API

## Role

Protect the typed, backend-agnostic public API while remaining phases add accuracy evidence, catalog validation, artifact distribution, and release guarantees.

## Standards

- Keep shared domain vocabulary in `pleiades-types`.
- Keep backend contracts in `pleiades-backend` and source-specific calculations in backend crates.
- Keep house, ayanamsa, sidereal conversion, sign placement, house placement, aspects, and other astrology-domain behavior out of source-specific backends.
- Prefer extensible identifiers, descriptors, validation helpers, and compatibility-profile metadata over breaking enum churn.
- Document units, frames, normalization, time scales, coordinate assumptions, and failure modes for public APIs.
- Keep unsupported advanced request modes as structured errors until implemented and validated.

## Remaining domain concerns

- Finalize first-release posture for built-in Delta T, UTC/UT1 conversion, apparent-place corrections, topocentric body positions, native sidereal backend output, and frame precision.
- Validate every release-advertised house and ayanamsa formula/alias against references or mark it constrained, approximate, descriptor-only, or unsupported.
- Keep custom body, house, and ayanamsa identifiers distinguishable from built-ins in profiles, summaries, and serialization.
- Ensure release profiles truthfully separate baseline guarantees, release additions, known gaps, constraints, and unsupported modes.
- Add batch-friendly façade helpers only where they reduce low-level orchestration without hiding assumptions.
