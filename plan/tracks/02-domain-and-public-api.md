# Track 2 — Domain and Public API

## Role

Keep domain types and facade APIs stable while body coverage, request modes, and
compatibility evidence mature.

## Standards

- Use strongly typed units, frames, time scales, observer data, bodies, houses,
  ayanamsas, zodiac modes, and errors.
- Keep sidereal conversion, house logic, and chart assembly outside
  source-specific backend crates.
- Keep unsupported modes explicit and structured.

## Remaining domain/API concerns

- UTC/UT1 and Delta-T convenience policy.
- Apparent-place and topocentric body-position semantics.
- Speed, retrograde/stationary, and motion-output policy.
- House and ayanamsa entries whose evidence status is weaker than descriptor
  presence.
- Completion of the full `compatibility-catalog.md` target set and the optional
  chart utilities (aspects, dignities) is deferred end-state work tracked in
  Phase 6; the public API/enums must absorb those additions without redesign.
- Public docs and rustdoc examples for production-ready chart workflows once
  claims are settled.
