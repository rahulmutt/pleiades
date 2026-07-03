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

- House and ayanamsa entries whose evidence status is weaker than descriptor
  presence.
- Completion of the ayanamsa `SE_SIDM_*` catalog and the remaining optional chart
  utility (dignities) is deferred end-state work tracked in Phase 6; the target
  house systems and aspects are already shipped. The public API/enums must absorb
  future additions without redesign.
- Public docs and rustdoc examples for production-ready chart workflows once
  claims are settled.
