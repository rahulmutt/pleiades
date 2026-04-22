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

## Progress update

Stage 3 is now underway as of 2026-04-22.

- [x] Baseline house-system catalog metadata is available in `pleiades-houses`, including aliases and latitude-sensitivity notes.
- [x] Baseline ayanamsa catalog metadata is available in `pleiades-ayanamsa`, including aliases and compatibility notes.
- [x] `pleiades-core` publishes a versioned compatibility profile that surfaces the current built-ins and known gaps.
- [x] `pleiades-cli` can print the compatibility profile for quick inspection.
- [x] A tropical chart MVP now exists: `pleiades-vsop87` and `pleiades-elp` provide approximate Sun/Moon/planet positions, `pleiades-core` assembles sign placements, and `pleiades-cli chart` renders a basic report.
- [ ] Sidereal conversion and house placement for the full baseline catalog still remain to be implemented.

## Workable state at end of stage
A user can compute a practical astrology chart in pure Rust for common modern use cases with documented limits, even though full compatibility breadth and reference-data validation are not complete yet.

## Suggested implementation slices

1. Implement the smallest useful backend path first: Sun/major planets via `pleiades-vsop87` and Moon via `pleiades-elp`, with documented provenance.
2. Add tropical-to-sidereal conversion in the domain layer so sidereal support does not become backend-specific.
3. Implement chart assembly in `pleiades-core` for a narrow but real workflow: positions, sign placement, and house placement.
4. Add baseline house systems incrementally, starting with simpler and more robust formulas before latitude-sensitive systems.
5. Add the baseline ayanamsa milestone and explicit aliases/near-equivalents.
6. Expose the workflow through `pleiades-cli` and publish the first compatibility profile describing what is actually implemented.

A good checkpoint partway through this stage is a tropical chart workflow even before the full baseline sidereal and house catalog is finished.

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
