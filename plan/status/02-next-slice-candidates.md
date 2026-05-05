# Status 2 — Next Slice Candidates

This file lists focused implementation slices that map to the current phase ladder. It intentionally omits completed report-surface, alias, fixture-summary, and release-rehearsal cleanup work.

## Phase 1 candidates — Reference accuracy and request semantics

### 1. Representative 1500-2500 reference expansion

- Add source/reference coverage at target-range boundaries and representative interior epochs.
- Include high-curvature windows needed by future artifact fitting.
- Keep hold-out rows separate from fitting/reference rows.
- Update validation reports to classify evidence as release-tolerance, hold-out, fixture exactness, or provenance-only.

### 3. Pluto release-claim closure

- Either add a source-backed Pluto path with validation across the claimed range or keep Pluto explicitly approximate.
- Ensure comparison/tolerance reports exclude approximate Pluto from release-grade major-body claims unless thresholds are met.
- Keep backend metadata, compatibility/release summaries, and docs synchronized with the chosen posture.

### 4. Lunar source posture decision

- Decide compact lunar baseline versus fuller ELP-style coefficient implementation for the first release.
- If compact baseline remains, publish measured limitations and error envelopes by lunar channel.
- If expanding to coefficient data, add pure-Rust ingestion/evaluation, provenance, validation, and tests.

### 5. Request/time semantics closure

- Decide first-release behavior for built-in Delta T and UTC/UT1 convenience conversion.
- Decide whether apparent-place corrections and topocentric body positions are implemented or explicitly deferred.
- Keep native sidereal backend output deferred unless a backend advertises equivalent support through capabilities.
- Add rustdoc/docs and regression tests for whichever posture is chosen.

## Phase 2 candidates — Production compressed artifacts

### 1. Production artifact profile manifest

- Replace prototype profile wording with a versioned production profile draft.
- Specify body set, date range, channels, derived outputs, unsupported outputs, speed policy, and thresholds.
- Add validation that fails on profile/threshold drift.

### 2. Deterministic artifact generator

- Build a generation command that consumes validated public inputs and writes normalized intermediates plus compressed artifacts.
- Record source revisions, generator parameters, segment strategy, checksums, and output profile identifiers.
- Keep the prototype fixture path separate from production artifact generation.

### 3. Fit-error and benchmark matrix

- Add body-class fit-error reporting for boundary and interior samples.
- Benchmark single lookup, batch lookup, decode cost, artifact size, and full-chart packaged-data use.
- Fail validation when measured errors exceed profile thresholds.

## Phase 3 candidates — Compatibility catalog evidence

### 1. House formula evidence batch

- Pick a small group of release-advertised house systems.
- Add formula/provenance notes, representative golden tests, latitude/failure constraints, and profile caveats.
- Ensure descriptor-only or approximate entries cannot be advertised as fully implemented.

### 2. Ayanamsa provenance batch

- Pick a small group of release-advertised ayanamsas.
- Add reference epoch/offset/formula provenance, alias evidence, sidereal metadata checks, and golden offsets.
- Classify custom-definition-only entries explicitly.

### 3. Compatibility-profile claim audit

- Add verification that distinguishes baseline guarantees, release additions, descriptor-only entries, constrained entries, aliases, and known gaps.
- Update release notes/docs to match the verified profile output.

## Phase 4 candidates — Release hardening

### 1. Final release gate command

- Compose existing checks into a documented release gate.
- Include format, clippy, tests, compatibility verification, artifact validation, bundle verification, audits, and benchmark/report generation.
- Ensure the gate blocks publication on stale reports or claim drift.

### 2. Clean-checkout bundle rehearsal

- Generate a release bundle from a clean checkout after Phases 1-3 changes.
- Verify manifests, sidecar checksums, artifact metadata, and report contents.
- Update docs for the exact reproducibility commands.

## Selection guidance

Prefer slices that convert an unverified claim into one of three explicit states:

1. implemented and validated,
2. implemented with documented constraints,
3. deferred/unsupported with structured errors and release-profile caveats.
