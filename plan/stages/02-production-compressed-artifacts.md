# Phase 2 — Production Compressed Artifacts

## Purpose

Replace the prototype packaged-data fixture with a reproducible 1500-2500 CE compressed ephemeris artifact that satisfies `spec/data-compression.md` and can be shipped through `pleiades-data` with truthful production claims.

This phase depends on Phase 1 because artifact fit error must be measured against trusted public source outputs and published tolerances.

## Spec drivers

- `requirements.md` FR-8, FR-9, NFR-2, NFR-4
- `data-compression.md` artifact layout, segmented polynomial/residual strategy, access pattern, accuracy targets, generation pipeline
- `backends.md` `pleiades-data` responsibilities
- `validation-and-testing.md` artifact validation, benchmark, checksum, release-gate expectations

## Current baseline

Implemented and not re-planned here:

- `pleiades-compression` artifact headers, body/segment structures, polynomial channels, residual support, profile metadata, byte-order policy, checksums, validation helpers, and decode helpers;
- `pleiades-data` deterministic draft artifact backend, bundled fixture, regeneration helper, request-policy summaries, frame reconstruction, boundary behavior, checksums, versioned production-profile draft summaries with explicit lookup-epoch policy and source-provenance metadata, release-bundle emission of the packaged lookup-epoch policy, packaged-artifact profile coverage, output-support, speed-policy, generation policy, and frame-treatment summaries/checksums, generator-parameter summaries that now explicitly carry the packaged-artifact checksum, regeneration-summary source-revision and quantization-scale metadata validation, cadence details in the production-generation source summary, a deterministic production-generation manifest checksum over the rendered manifest payload, scope-specific target-threshold envelopes that now split Pluto into its own release-facing scope, optional manifest-sidecar emission for regeneration commands, a dedicated production-generation-manifest-checksum summary in the CLI/validation surfaces, and optional explicit artifact path loading behind a feature;
- CLI/validation commands for artifact inspection, validation, target-threshold summaries, fit sample class summaries, regeneration, benchmarks, and release-report inclusion.

## Remaining implementation goals

### 1. Define production artifact profiles

- Specify bundled bodies, time range, stored channels, derived outputs, unsupported outputs, speed policy, frame treatment, and compatibility with release profiles; source provenance is now explicit in the production-profile and generator-parameter summaries.
- Finalize the scope-specific target error envelopes for luminaries, major planets, Pluto, lunar points, selected asteroids, and any custom/named bodies.
- Encode profile metadata in artifact headers, manifests, validation summaries, and release reports.

### 2. Build deterministic generation from public inputs

- Use Phase 1 validated source outputs as generation inputs.
- Document sampling cadence, segment boundaries, polynomial/residual strategy, quantization scales, and checksums.
- Provide a maintainer command that regenerates production artifacts without native tooling.
- Keep raw source inputs, normalized intermediate data, generated manifests, and distributable compressed artifacts separated; the regeneration command now supports a manifest sidecar so the output split can be exercised directly.

### 3. Improve fit and lookup accuracy

- Tune segment lengths, polynomial order, quantization, and residual corrections by body class.
- Add high-curvature lunar windows, boundary-date checks, and interval-interior validation.
- Support efficient random access by body and time across the full advertised range.
- Keep unsupported outputs fail-closed unless the profile advertises deterministic reconstruction.

### 4. Validate and benchmark artifacts

- Validate checksum, header, profile, body index, segment directory, residual consistency, boundary lookups, interior samples, unsupported bodies, and unsupported outputs.
- Compare decoded results to generation sources and publish measured fit errors by body, body class, coordinate channel, and time slice.
- Benchmark decode, lookup, batch lookup, memory footprint, artifact size, and full-chart packaged-data use.
- Make validation fail on profile, threshold, checksum, or claim drift, and report field-specific measured-versus-threshold context when fit thresholds are exceeded.

### 5. Finalize distribution behavior

- Decide default bundled artifact size and external artifact loading policy.
- Document artifact path configuration, version compatibility, checksum verification, and failure modes.
- Include artifact metadata, checksums, manifests, and benchmark evidence in release bundles.

## Done criteria

Phase 2 is complete when:

- a deterministic production artifact covers the advertised 1500-2500 CE range and release body set;
- the artifact can be regenerated from documented public inputs and parameters;
- validation reports show measured fit errors inside published thresholds;
- runtime lookup supports efficient random access with checksum/profile validation and structured failures;
- release summaries and bundle manifests include artifact provenance, profile, checksums, size, and benchmark data.

## Deferred to later phases

- Adding unaudited catalog entries belongs to Phase 3.
- Final archive/signing/publication process belongs to Phase 4.
