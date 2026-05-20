# Track 2 — Domain and Public API

## Role

Keep domain types and façade APIs stable while body coverage, request modes, and compatibility evidence mature.

## Standards

- Use strongly typed units, frames, time scales, observer data, bodies, houses, ayanamsas, and errors.
- Keep sidereal conversion, house logic, and chart assembly outside source-specific backend crates.
- Keep unsupported modes explicit and structured.

## Remaining domain/API concerns

- UTC/Delta-T convenience policy and any future built-in civil-time conversion.
- Apparent-place and topocentric body-position semantics.
- House and ayanamsa entries whose implementation/evidence status is weaker than their descriptor presence.
- Public docs and rustdoc examples for production-ready chart workflows once claims are settled.
