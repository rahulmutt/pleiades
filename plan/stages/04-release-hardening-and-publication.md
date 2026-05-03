# Phase 4 — Release Hardening and Publication

## Purpose

Convert release rehearsal tooling into a publishable release process backed by current validation evidence, production artifacts, compatibility profiles, documentation, audits, checksums, and bundle verification.

## Spec drivers

- `SPEC.md` acceptance summary
- `requirements.md` NFR-1 through NFR-6 and FR-6
- `api-and-ergonomics.md` documentation and determinism expectations
- `validation-and-testing.md` release gates and validation tooling
- `backends.md` capability matrix expectations
- `data-compression.md` artifact provenance and reproducibility expectations

## Current baseline

Implemented and not re-planned here:

- CLI/validation commands for compatibility profiles, backend matrices, API stability posture, validation reports, artifact summaries, comparison-envelope summaries, release notes, release checklists, workspace audits, release summaries, and release bundle generation/verification;
- workspace-native audit for mandatory native build hooks;
- checksum manifests and release-bundle verification over current rehearsal artifacts;
- README and workflow documentation for local checks and release smoke commands.

Known remaining gaps:

- Release reports must be regenerated after Phase 1-3 production changes and must not retain prototype/interim claims.
- CI/release gates need to enforce the final validation thresholds, compatibility-profile truthfulness, artifact checksums, and pure-Rust audit.
- Public rustdoc/examples need final review for units, frames, time scales, error modes, and chart workflows.
- Versioning, archive layout, and publication checklist need final maintainer approval for first release.

## Remaining implementation goals

### 1. Finalize release gates

- Gate releases on `cargo fmt --all --check`, strict clippy, workspace tests, compatibility-profile verification, artifact validation, release-bundle verification, and pure-Rust audit.
- Ensure validation commands fail or clearly block publication on advertised accuracy/profile regressions.
- Capture tool versions, source revision, workspace cleanliness, and benchmark provenance.

### 2. Produce release artifacts

- Generate and archive the release compatibility profile.
- Generate backend capability matrix, API stability posture, comparison-envelope summary, validation report, benchmark report, artifact summary, release notes, release checklist, and release summary.
- Include production compressed artifacts, checksums, manifest, and manifest checksum sidecar when packaged data is claimed.
- Verify bundles from a clean checkout.

### 3. Harden documentation

- Update README, docs, and rustdoc examples for main workflows.
- Document time-scale, Delta T, observer, apparentness, frame, zodiac, house, ayanamsa, backend-selection, artifact, and release-profile semantics.
- Make known gaps explicit: unsupported apparent/topocentric/native-sidereal modes, approximations, date ranges, and catalog caveats.

### 4. Stabilize publication policy

- Decide crate versions, feature flags, minimum supported Rust/toolchain posture, and artifact version compatibility.
- Document reproducibility commands for validation reports and artifacts.
- Preserve pure-Rust requirements across all default build/test/release workflows.

## Done criteria

Phase 4 is complete when:

- a release bundle can be generated and verified from a clean checkout;
- all release reports reflect current code, current data, and current compatibility claims;
- artifact and validation checksums verify;
- pure-Rust/native-dependency audits pass;
- public docs and examples describe supported workflows and known limitations accurately;
- maintainers can publish or archive the release without manual, undocumented steps.

## Follow-on work after first hardened release

- Expand optional backend families and larger source corpora.
- Add additional asteroids and higher-level chart utilities.
- Refine topocentric/apparent/Delta T support if not included in the first release.
- Continue compatibility-catalog expansion under release-profile truthfulness rules.
