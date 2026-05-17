# Phase 2 — Reference/Source Corpus Productionization

## Goal

Provide documented public source inputs broad enough for backend validation, release-grade body claims, and production artifact fitting.

## Starting point

`pleiades-jpl` includes checked-in JPL Horizons snapshots, hold-out fixtures, provenance summaries, batch/single parity checks, source-window summaries, selected asteroid samples, selected-asteroid request corpora, and production-generation manifest reports. This is regression-quality evidence, but not yet a broad production reader or generated corpus covering all advertised body/date/channel claims.

## Implementation goals

- The production source strategy is documented as a hybrid fixture corpus: checked-in reference and hold-out fixtures plus a separate generation-input path.
- Record source provenance, frame, time scale, columns/channels, source revision, generation command, checksums, and redistribution posture.
- Completed: the production-generation source summary now records the generation command alongside the checked-in CSV provenance, frame, time scale, schema, and checksum markers, and now validates the merged provenance block before rendering.
- Completed: the production-generation boundary source summary now stages and verifies in the release bundle alongside the reference snapshot source summary.
- Completed: the packaged-artifact phase-2 corpus alignment summary now includes the source/provenance lines for the reference, comparison, and hold-out corpora alongside the body-class coverage evidence, and now also keeps the selected-asteroid source evidence/windows plus the production-generation source provenance and body-class coverage in the same phase-2 posture, keeping the threshold posture aligned with the recorded corpus metadata and the separate generation-input corpus.
- Completed: the comparison body-class tolerance posture now has a compact release-facing summary surface with alias coverage, keeping the backend-path tolerance posture available outside the full validation report.
- Completed: the comparison snapshot source summary now validates the manifest-derived source, coverage, columns, and checksum directly so the release-facing provenance line fails closed if the checked-in J2000 slice drifts.
- Completed: the selected-asteroid source request corpus summary now has a release-facing CLI surface, keeping the checked-in request slice visible alongside the source evidence and source-window summaries.
- Completed: the packaged-artifact phase-2 corpus alignment details accessor is now public so typed validation/report consumers can reuse the same structured evidence as the release-facing CLI surface.
- Completed: the reference snapshot source summary now surfaces redistribution posture directly alongside the checked-in checksum-bearing source metadata, the independent hold-out source summary now surfaces redistribution posture in release-facing reports, and the reference and hold-out snapshot manifests now record redistribution posture comments so the checked-in fixture headers carry the same provenance posture as the generation summaries.
- Expand coverage only where it supports release claims or artifact fitting.
- Keep reference, hold-out, boundary-overlay, fixture-exactness, and provenance-only evidence classes separate.
- Publish body-class tolerance reports and empirical error envelopes for release-claimed backend paths.
- Preserve small golden fixtures for fast regression tests even if broader ingestion lands.
- Ensure all ingestion/build steps remain pure Rust and deterministic.

## Completion criteria

Phase 2 is complete when validation and artifact-generation inputs cover the advertised release body set and range with documented provenance, reproducible generation, checksums, and current tolerance reports.

## Out of scope

- Shipping the compressed artifact; that belongs to Phase 1.
- Deciding final body release status; that belongs to Phase 3.
- Promoting house or ayanamsa catalog claims; that belongs to Phase 5.
