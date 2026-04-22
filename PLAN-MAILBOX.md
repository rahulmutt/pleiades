# PLAN-MAILBOX

## 2026-04-22

Implemented the first Stage 3 slice:

- baseline house-system catalog metadata now lives in `pleiades-houses`
- baseline ayanamsa catalog metadata now lives in `pleiades-ayanamsa`
- `pleiades-core` now publishes a versioned compatibility profile with known gaps
- `pleiades-cli` can print the compatibility profile for quick inspection

Next recommended slice: start the actual algorithmic chart workflow by wiring in a minimal Sun/Moon backend path, then layer tropical-to-sidereal and chart assembly helpers on top.

## 2026-04-22 — tropical chart MVP landed

Implemented the next Stage 3 slice:

- `pleiades-vsop87` now computes approximate tropical positions for the Sun and major planets with a pure-Rust orbital-elements model
- `pleiades-elp` now computes an approximate tropical Moon position with a pure-Rust analytical model
- `pleiades-backend` gained a simple composite router for Moon-plus-planets workflows
- `pleiades-core` can assemble a basic tropical chart snapshot with zodiac sign placements
- `pleiades-cli chart` renders the new chart report using the composite backend

Remaining Stage 3 work: sidereal conversion, fuller house placement, and any missing chart ergonomics needed to make the workflow feel production-ready.

## 2026-04-22 — sidereal chart conversion added

Implemented the next Stage 3 slice:

- `pleiades-ayanamsa` now carries baseline epoch/offset metadata for built-in sidereal catalog entries and exposes a deterministic offset helper for custom or built-in definitions
- `pleiades-core` now exposes `sidereal_longitude` and uses it inside chart assembly when a sidereal zodiac mode is requested
- `pleiades-cli chart` accepts `--ayanamsa <name>` and can render sidereal chart output on top of the tropical backends
- compatibility-profile output was updated to describe the current sidereal chart capability and the remaining house-placement gap

Remaining Stage 3 work: house placement for the baseline catalog, plus any chart ergonomics needed to polish the workflow.
