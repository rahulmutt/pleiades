# Stage 5 — Compression and Packaged Data

## Goal
Deliver the fast, offline-friendly 1500-2500 experience via compressed ephemeris artifacts and a packaged-data backend.

## Why this stage comes fifth
Compression is only worth stabilizing after the type system, algorithmic behavior, and reference-validation pipeline are mature enough to define artifact shape, generation inputs, and error thresholds confidently.

## Primary deliverables

### `pleiades-compression`
- artifact format definition
- deterministic fitting pipeline primitives
- coefficient quantization and decode logic
- checksums and versioning

### `pleiades-data`
- packaged backend that reads generated artifacts
- body/time segment lookup
- memory-efficient decode path
- fallback or composition story where packaged coverage is incomplete

### Build/release workflow
- artifact generation command or pipeline
- published generation metadata and validation summaries
- measured error envelopes per body class
- distribution strategy for bundling artifacts with applications

## Workable state at end of stage
Applications targeting the common 1500-2500 window can use a compact offline backend with predictable speed and documented error characteristics, while broader-range or validation workloads can still use other backends.

## Suggested tasks

1. Prototype segment sizing and polynomial/residual strategies by body class.
2. Measure file size, latency, and error tradeoffs.
3. Validate artifact edges and segment-boundary behavior thoroughly.
4. Add CLI tooling to inspect artifact metadata and query packaged results.
5. Document regeneration from public inputs end to end.

## Recommended validation

- encode/decode roundtrip and checksum tests
- artifact-vs-source error measurements
- boundary-date regression tests
- benchmark full-chart lookups against algorithmic and reference backends

## Exit criteria

- packaged backend is clearly faster or more deployable for its target use case
- artifact format is versioned and reproducible
- published errors are within documented thresholds
- fallback behavior outside packaged coverage is explicit

## Risks to avoid

- locking in a format before enough empirical validation exists
- optimizing artifact size at the expense of maintainability or determinism
- coupling artifact generation too tightly to one transient source layout
