# Phase 2 — Reproducible Compressed Artifacts

## Purpose

Turn the current compressed-artifact scaffolding into a reproducible packaged-data system for 1500-2500 CE. This phase depends on Phase 1 reference-quality source outputs because artifact fit errors must be measured against trustworthy inputs.

## Spec drivers

- `spec/requirements.md`: FR-8, FR-9, NFR-2, NFR-4
- `spec/data-compression.md`: segmented polynomial/residual format, stored-vs-derived outputs, generation pipeline
- `spec/backends.md`: `pleiades-data` responsibilities
- `spec/validation-and-testing.md`: artifact validation, benchmarks, release gates

## Current baseline

`pleiades-compression` defines an in-memory artifact model, deterministic binary codec, artifact capability profile metadata, and lookup path. `pleiades-data` exposes a packaged backend backed by static sample segments with an ecliptic-only/no-motion profile, and validation tooling can inspect and summarize artifacts.

## Remaining implementation goals

1. Define the artifact profile format.
   - Initial header fields, versioning, endian policy, checksums, provenance, and capability/profile sections are implemented in the codec.
   - Initial profile metadata records stored channels, derived outputs, unsupported outputs, and speed derivation policy.
   - Remaining work: refine body-specific profile semantics as generated artifacts become available and surface profile summaries in validation/release reports.
   - Keep decode deterministic and independent of platform-specific binary layout.

2. Build a deterministic generation pipeline.
   - Add pure-Rust tooling to sample a trusted backend over 1500-2500 CE.
   - Fit body/channel-specific polynomial or Chebyshev segments.
   - Support shorter or residual-corrected lunar segments.
   - Stamp source versions, generation parameters, and checksums.

3. Implement binary encode/decode and random access.
   - Move beyond sample in-memory construction to a stable serialized artifact representation.
   - Provide efficient lookup by body and time segment.
   - Validate boundary behavior around segment edges and unsupported bodies.
   - Keep artifact loading usable for desktop/server applications without mandatory native dependencies.

4. Measure compression quality.
   - Define initial per-body-class target error envelopes.
   - Compare decoded positions and speeds against generation sources.
   - Benchmark artifact size, lookup latency, batch throughput, and memory footprint.
   - Include measured limits in artifact metadata and validation reports.

5. Upgrade `pleiades-data` to consume generated artifacts.
   - Bundle or fixture a small deterministic artifact for tests.
   - Provide feature-gated paths for larger distributable artifacts when appropriate.
   - Report packaged-data capabilities honestly through backend metadata and release profiles.

## Done criteria

- A maintainer can regenerate the test artifact from documented public inputs and deterministic parameters.
- `pleiades-compression` can encode, decode, checksum, and query the artifact format.
- `pleiades-data` uses generated artifacts rather than hand-authored sample segments for claimed coverage.
- Validation reports include artifact error summaries, boundary checks, and benchmark numbers.
- Artifact metadata states exactly what is stored, derived, or unsupported.

## Work that belongs in later phases

- Expanding compatibility catalogs belongs to Phase 3.
- Archiving release artifacts and final release bundle signing/checksum policy belongs to Phase 4.
