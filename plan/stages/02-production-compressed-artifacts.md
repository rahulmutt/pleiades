# Phase 2 — Production Compressed Artifacts

## Purpose

Replace the current prototype packaged-data fixture with a reproducible 1500-2500 CE compressed ephemeris artifact that satisfies `spec/data-compression.md` and can be shipped as `pleiades-data`.

This phase depends on Phase 1 accuracy closure because artifact fit error must be measured against trusted source outputs.

## Spec drivers

- `requirements.md` FR-8, FR-9, NFR-2, NFR-4
- `data-compression.md` artifact layout, segmented polynomial/residual strategy, access pattern, accuracy targets, generation pipeline
- `backends.md` `pleiades-data` responsibilities
- `validation-and-testing.md` artifact validation, benchmark, checksum, release-gate expectations

## Current baseline

Implemented and not re-planned here:

- `pleiades-compression` codec primitives, headers, checksums, body/segment validation, residual support, profile summaries, and decode helpers;
- `pleiades-data` prototype artifact backend, checked-in deterministic fixture, regeneration helper, request policy summaries, frame reconstruction, batch parity, checksum verification, benchmark/report integration, explicit regeneration profile identifiers for the prototype posture, compact body-class coverage summaries for the bundled prototype, explicit body-class target-envelope reporting for the current prototype posture, residual-body coverage validation that now fails closed if regeneration provenance claims a residual body outside the bundled body list, generator-parameter / manifest drift coverage that now also checks profile-id and label mismatches, and validated scope-envelope report lines that keep the per-body-class breakdown fail-closed;
- CLI/validation commands for artifact summaries, validation, regeneration, and release-report inclusion.

Known remaining gaps:

- The checked-in artifact is a small prototype, not a full production 1500-2500 data product.
- Current prototype fit error is not acceptable for release-grade packaged-data claims.
- Generation is tied to the checked-in reference snapshot rather than a complete documented public-input corpus.
- Body-specific segment strategy and residual density still need release-grade tuning, but the packaged-data crate now records the current measured fit envelope plus body-class scope envelopes in the target-threshold scaffold and validates the generator manifest against it.
- Optional external artifact loading is feature-gated and not yet a complete distribution story.

## Remaining implementation goals

### 1. Define production artifact profiles

- Define versioned artifact profile identifiers for release artifacts.
- List bundled bodies, time range, stored channels, derived outputs, unsupported outputs, speed policy, frame treatment, and lookup epoch policy. The packaged-data crate now exposes a dedicated production-profile skeleton summary that aggregates the current prototype posture, and the target-threshold scaffold now carries the release-profile identifier through the manifest surface; the remaining work is to turn that skeleton into a finalized release manifest.
- Define body-class-specific target error envelopes for luminaries, planets, Pluto, lunar points, selected asteroids, and custom/named bodies if shipped. The current prototype now exposes an explicit scope-envelope breakdown for the bundled body classes; the remaining work is to turn that posture into finalized release thresholds.
- Encode profile metadata in artifact headers and release summaries.

### 2. Build deterministic generation from public inputs

- Use Phase 1 validated source outputs as generation inputs.
- Document all generation parameters: input source revisions, sampling cadence, segment boundaries, polynomial/residual strategy, quantization scales, and checksums.
- Provide a maintainer command that regenerates the production artifact from documented inputs without native tooling.
- Keep raw source inputs, normalized intermediate data, and distributable compressed artifacts separated.

### 3. Improve fit and lookup accuracy

- Tune segment lengths, polynomial order, and residual corrections per body class.
- Add high-curvature lunar windows, boundary-date checks, and interval-interior validation.
- Support efficient random access by body and time over the full advertised range.
- Keep unsupported outputs fail-closed unless the artifact profile advertises deterministic reconstruction.

### 4. Validate and benchmark artifacts

- Validate checksum, header, profile, body index, segment directory, residual consistency, boundary lookups, interior samples, and unsupported-body behavior.
- Compare decoded results to generation sources and publish measured fit errors by body and body class.
- Benchmark decode, lookup, batch lookup, memory footprint, and full-chart packaged-data use.
- Make artifact validation fail on profile/claim drift.

### 5. Finalize distribution behavior

- Decide default bundled artifact size and optional external artifact loading policy.
- Document artifact path configuration, version compatibility, checksum verification, and expected failure modes.
- Include artifact metadata in release bundles, manifests, and release notes.

## Done criteria

Phase 2 is complete when:

- a deterministic production artifact covers the advertised 1500-2500 CE range and release body set;
- the artifact can be regenerated from documented public inputs and parameters;
- validation reports show measured fit errors within the published thresholds;
- runtime lookup supports efficient random access with checksum/profile validation and structured failures;
- release summaries and bundle manifests include artifact provenance, profile, checksums, size, and benchmark data.

## Work intentionally deferred

- Adding catalog entries whose formulas are not yet audited belongs to Phase 3.
- Final publication/signing policy and archival release process belong to Phase 4.
