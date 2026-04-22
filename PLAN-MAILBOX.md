# PLAN-MAILBOX

## 2026-04-22

Implemented the first Stage 3 slice:

- baseline house-system catalog metadata now lives in `pleiades-houses`
- baseline ayanamsa catalog metadata now lives in `pleiades-ayanamsa`
- `pleiades-core` now publishes a versioned compatibility profile with known gaps
- `pleiades-cli` can print the compatibility profile for quick inspection

Next recommended slice: start the actual algorithmic chart workflow by wiring in a minimal Sun/Moon backend path, then layer tropical-to-sidereal and chart assembly helpers on top.
