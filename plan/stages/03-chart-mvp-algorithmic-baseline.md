# Stage 3 — Chart MVP with Algorithmic Baseline

## Goal
Ship the first end-to-end useful product: a pure-Rust chart workflow using algorithmic backends plus baseline house and ayanamsa support.

## Why this stage comes third
This is the earliest point where the project can become directly useful to consumers while still avoiding the complexity of large reference datasets and compression pipelines.

## Primary deliverables

### Backends
- `pleiades-vsop87` for Sun and major planets
- `pleiades-elp` for the Moon and documented lunar-derived quantities where justified
- optional simple composite backend that routes planets to VSOP87 and Moon to ELP

### Domain crates
- baseline ayanamsa catalog:
  - Lahiri
  - Raman
  - Krishnamurti
  - Fagan/Bradley
  - True Chitra
  - documented aliases/near-equivalents
- baseline house-system milestone:
  - Placidus
  - Koch
  - Porphyry
  - Regiomontanus
  - Campanus
  - Equal
  - Whole Sign
  - Alcabitius
  - Meridian / documented Axial variants
  - Topocentric (Polich-Page)
  - Morinus
- explicit error/status behavior for high-latitude and other known failure modes

### User-facing workflow
- `pleiades-core` chart assembly helpers for common workflows
- `pleiades-cli` command(s) to query positions and generate a basic chart report
- initial compatibility profile documenting built-ins, aliases, and current gaps

## Workable state at end of stage
A user can compute a practical astrology chart in pure Rust for common modern use cases with documented limits, even though full compatibility breadth and reference-data validation are not complete yet.

## Suggested tasks

1. Implement planetary and lunar query paths with documented provenance.
2. Add tropical-to-sidereal conversion in the domain layer.
3. Implement baseline house systems incrementally, starting with systems with simpler and more robust formulas.
4. Provide a chart object or report structure containing positions, sign placement, and house placement.
5. Add CLI examples and sample outputs.
6. Publish the first machine-readable or human-readable compatibility profile.

## Recommended validation

- golden tests for sample charts
- house cusp regression tests, especially edge latitudes
- CLI snapshot tests
- benchmark a single chart calculation path

## Exit criteria

- one documented chart generation workflow works end to end
- baseline house and ayanamsa milestone is materially implemented
- limits versus the full target compatibility catalog are explicit
- composite backend behavior is deterministic

## Risks to avoid

- over-optimizing before the correctness story is established
- claiming full compatibility before publishing exact implemented coverage
- hardwiring the MVP around one backend in a way that blocks later routing
