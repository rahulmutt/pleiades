# Track 2 — Domain and Public API

## Role

Protect the typed, backend-agnostic public API while remaining implementation phases add accuracy, catalog breadth, and release guarantees.

## Standards

- Keep shared domain vocabulary in `pleiades-types`.
- Keep backend contracts in `pleiades-backend` and source-specific calculations in backend crates.
- Keep house, ayanamsa, sidereal conversion, sign placement, house placement, aspects, and other astrology-domain behavior out of source-specific backends.
- Prefer extensible identifiers, descriptors, and compatibility-profile metadata over breaking enum churn.
- Document units, frames, normalization, time scales, and failure modes for public APIs.

## Remaining domain concerns

- Clarify Delta T and time-scale policies.
- Ensure topocentric support is either implemented consistently or rejected explicitly.
- Validate every shipped house and ayanamsa formula/alias against references.
- Keep custom body, house, and ayanamsa identifiers distinguishable from built-ins in profiles and serialization.
- Ensure release profiles truthfully separate baseline guarantees, release additions, known gaps, and unsupported modes.
