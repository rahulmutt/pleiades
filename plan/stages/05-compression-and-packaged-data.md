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

## Progress update

Stage 5 compression and packaged-data work is complete as of 2026-04-23.

- [x] `pleiades-compression` now defines a versioned artifact header, quantized polynomial channels, binary encode/decode logic, checksums, and artifact roundtrip tests.
- [x] `pleiades-data` now ships a bundled artifact generated from the checked-in JPL reference snapshot, covering the comparison-body planetary set with packaged lookup support and a compressed-data backend implementation.
- [x] The CLI now composes packaged data ahead of the algorithmic backends so chart queries automatically use packaged lookups when available and still fall back for bodies outside the bundled slice.
- [x] `pleiades-validate` now exposes `validate-artifact` to inspect the bundled compressed artifact, verify encode/decode and checksum behavior, and report segment-boundary continuity for the packaged bodies.
- [x] The validation report now also benchmarks the packaged-data backend against its bundled artifact corpus, so the compressed-data decode path has a reproducible throughput measurement alongside the algorithmic and reference-backend benchmarks.
- [x] `validate-artifact` now reports measured artifact error envelopes against the algorithmic baseline, including body-class-specific summaries for luminaries, major planets, lunar points, asteroids, and custom bodies, and the packaged artifact coverage now spans the comparison-body planetary set.

## Suggested implementation slices

1. Prototype artifact layout and decode logic with one or two bodies before generalizing across the full packaged set.
2. Experiment with segment sizing and polynomial/residual strategies by body class using reproducible benchmarks.
3. Measure file size, latency, and error tradeoffs before freezing the format.
4. Add `pleiades-data` lookup support and make fallback/composition behavior explicit where packaged coverage is incomplete.
5. Validate artifact edges and segment-boundary behavior thoroughly.
6. Add CLI tooling to inspect artifact metadata and query packaged results, then document regeneration from public inputs end to end.

Do not optimize for maximum compression first; optimize for deterministic generation and maintainable decoding, then tighten size/performance iteratively.

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
