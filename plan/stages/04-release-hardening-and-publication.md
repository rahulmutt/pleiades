# Phase 4 — Release Hardening and Publication

## Purpose

Convert release rehearsal tooling into a publishable process backed by current validation evidence, production artifacts, compatibility profiles, documentation, audits, checksums, and bundle verification.

## Spec drivers

- `SPEC.md` acceptance summary
- `requirements.md` NFR-1 through NFR-6 and FR-6
- `api-and-ergonomics.md` documentation and determinism expectations
- `validation-and-testing.md` release gates and validation tooling
- `backends.md` capability matrix expectations
- `data-compression.md` artifact provenance and reproducibility expectations

## Current baseline

Implemented and not re-planned here:

- CLI/validation commands for compatibility profiles, backend matrices, API stability posture, request policies, validation reports, artifact summaries, benchmarks, release notes, release checklists, release summaries, audits, release-bundle generation, and release-bundle verification; the `release-gate` / `release-gate-summary` front ends now also run compatibility-profile verification plus release-bundle generation/verification before they render the checklist text;
- workspace-native audit for mandatory native build hooks;
- checksum manifests and bundle verification over current rehearsal artifacts;
- README and documentation coverage for local checks, request posture, and release smoke commands.

## Remaining implementation goals

### 1. Finalize blocking release gates

- Gate releases on format, strict clippy, workspace tests, compatibility-profile verification, artifact validation, release-bundle verification, benchmark/report generation, and pure-Rust/native-dependency audit.
- Ensure validation commands fail or clearly block publication on advertised accuracy, artifact, profile, or compatibility regressions.
- Capture tool versions, source revision, workspace cleanliness, benchmark parameters, and artifact-generation parameters.

### 2. Produce current release artifacts

- Generate and archive the release compatibility profile.
- Generate backend capability matrix, API stability posture, request-policy summaries, comparison/tolerance reports, validation report, benchmark report, artifact summary, release notes, release checklist, and release summary.
- Include production compressed artifacts, checksums, manifests, and manifest checksum sidecars when packaged data is claimed.
- Verify bundles from a clean checkout.

### 3. Harden documentation

- Update README, docs, and rustdoc examples for main workflows.
- Document time-scale, Delta T, observer, apparentness, frame, zodiac, house, ayanamsa, backend-selection, artifact, and release-profile semantics.
- Make known gaps explicit, including unsupported advanced request modes, approximate paths, date ranges, catalog caveats, and prototype-only artifacts if any remain.

### 4. Stabilize publication policy

- Decide crate versions, feature flags, MSRV/toolchain posture, artifact version compatibility, and archive layout.
- Document reproducibility commands for validation reports and artifacts.
- Preserve pure-Rust requirements across default build/test/release workflows.

## Done criteria

Phase 4 is complete when:

- a release bundle can be generated and verified from a clean checkout;
- all release reports reflect current code, current data, current artifacts, and current compatibility claims;
- artifact and validation checksums verify;
- pure-Rust/native-dependency audits pass;
- public docs and examples describe supported workflows and known limitations accurately;
- maintainers can publish or archive the release without manual, undocumented steps.

## Follow-on work after first hardened release

- Expand optional backend families and larger source corpora.
- Add additional asteroids and higher-level chart utilities.
- Refine topocentric, apparent-place, UTC/Delta T, and native sidereal support if deferred.
- Continue compatibility-catalog expansion under release-profile truthfulness rules.
