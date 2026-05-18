# Phase 2 — Reference/Source Corpus Productionization

## Goal

Provide documented public source inputs broad enough for backend validation, release-grade body claims, and production artifact fitting.

## Starting point

`pleiades-jpl` includes checked-in JPL Horizons snapshots, hold-out fixtures, provenance summaries, batch/single parity checks, source-window summaries, selected asteroid samples, selected-asteroid request corpora, and production-generation manifest reports. This is regression-quality evidence, but not yet a broad production reader or generated corpus covering all advertised body/date/channel claims.

## Implementation goals

- The production source strategy is documented as a hybrid fixture corpus: checked-in reference and hold-out fixtures plus a separate generation-input path.
- Record source provenance, frame, time scale, columns/channels, source revision, generation command, checksums, and redistribution posture.
- Completed: the production-generation source summary now records the generation command alongside the checked-in CSV provenance, frame, time scale, schema, source-window evidence, and checksum markers, and now validates the merged provenance block before rendering; the production-generation source-window summary now stages and verifies alongside it in release bundles, and the regression expectation now matches the source-window placement used by the report helper. The source-revision metadata is now structured as a dedicated checksum pair instead of a raw string, and the rendered summary now also fails closed on the required columns and redistribution-posture fragments. The source-summary regression now also locks the current source-window ordering against the report helper so the merged source block stays aligned with the source-window placement. The lunar-theory source-selection summary now also stages in the release bundle alongside the lunar source-family summary.
- Completed: the production-generation boundary source summary now stages and verifies in the release bundle alongside the reference snapshot source summary, and the production-generation boundary request corpus summary now rides along with the same staged provenance.
- Completed: the interpolation-quality sample request corpus summary now stages in the release bundle and the release-bundle verifier cross-checks it against the live renderer, so the sampled request corpus posture fails closed on semantic drift.
- Completed: the release bundle verifier now also cross-checks the staged production-generation boundary source and request-corpus summaries against the live renderers, so those phase-2 provenance files fail closed when semantic drift slips past checksum refreshes.
- Completed: the packaged-artifact phase-2 corpus alignment summary now includes the source/provenance lines for the reference, comparison, and hold-out corpora alongside the body-class coverage evidence, and now also keeps the selected-asteroid source evidence/windows and request corpus plus the production-generation boundary source evidence, production-generation source provenance, and body-class coverage in the same phase-2 posture, keeping the threshold posture aligned with the recorded corpus metadata and the separate generation-input corpus. The phase-2 corpus alignment summary now also uses its own dedicated validation error type, and the validation/report surfaces now route that block through a dedicated validated wrapper, so direct alignment validation failures stay explicit and the live renderer path follows the same fail-closed pattern as the target-threshold and source-fit sync summaries.
- Completed: the comparison body-class tolerance posture now has a compact release-facing summary surface with alias coverage, keeping the backend-path tolerance posture available outside the full validation report.
- Completed: the comparison snapshot source summary now validates the manifest-derived source, coverage, redistribution posture, columns, and checksum directly so the release-facing provenance line fails closed if the checked-in J2000 slice drifts, and the comparison snapshot source/body-class coverage/batch-parity CLI commands now use validated helpers instead of the fallback-string path. The release-bundle verifier now also cross-checks the staged comparison and reference snapshot source summaries against the live renderers, so corpus provenance drift fails closed even when the staged checksums still match.
- Completed: the selected-asteroid source request corpus summary now has a release-facing CLI surface, keeping the checked-in request slice visible alongside the source evidence and source-window summaries.
- Completed: the reference snapshot mixed TT/TDB batch parity summary now has a release-facing CLI surface, so the checked-in batch slice can report its time-scale mix alongside the existing ecliptic batch parity evidence.
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
