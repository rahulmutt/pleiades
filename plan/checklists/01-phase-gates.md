# Checklist 1 — Phase Gates

Use this checklist for active implementation only. Completed historical work should not be re-added.

## Phase 1: Production reference/source corpus

- [x] Production source strategy is documented and pure-Rust compatible.
- [x] Provenance, redistribution posture, frame, time scale, schema, source revision, generation command, and checksums are recorded.
- [ ] Corpus covers every release-claimed body/channel/frame/date range.
- [ ] Fitting/reference, independent hold-out, boundary overlay, fixture-exactness, and provenance-only evidence are separated.
- [x] Corpus validation fails on body, epoch, channel, frame, apparentness, or checksum drift.

## Phase 2: Production compressed ephemeris

- [ ] Artifact generation consumes only validated Phase 1 source inputs.
- [ ] Stored, derived, approximated, and unsupported outputs are explicit in the profile.
- [ ] Body/channel errors pass published production thresholds against reference and hold-out corpora.
- [ ] Lookup, batch, decode, size, and chart-style benchmarks are current.
- [ ] Regeneration is deterministic and sidecars/manifests/checksums verify from a clean checkout.

## Phase 3: Body and backend claim completion

- [ ] Pluto status is source-backed, artifact-backed, approximate, constrained, or excluded in every release surface.
- [ ] Lunar theory and lunar-point claims match implemented formulas and validation windows.
- [ ] Selected asteroid claims match source-backed evidence and backend metadata.
- [ ] Custom/numbered body support is extensible without overstating backend coverage.
- [ ] Backend capability matrices agree with actual supported bodies, dates, frames, time scales, and request modes.

## Phase 4: Advanced request modes

- [ ] UTC/Delta-T behavior is implemented with evidence or explicitly deferred.
- [ ] Apparent-place behavior is implemented with evidence or rejected with structured errors.
- [ ] Topocentric body-position behavior is implemented with evidence or rejected with structured errors.
- [ ] Native sidereal backend output is implemented with evidence or explicitly unsupported.
- [ ] Backend matrices, rustdoc, CLI output, release reports, and tests agree.

## Phase 5: Compatibility and release readiness

- [ ] House formulas, aliases, and latitude/numerical constraints are audited for release-claimed entries.
- [ ] Ayanamsa reference epochs, offsets, formulas, aliases, and provenance are audited for release-claimed entries.
- [ ] Descriptor-only, custom-only, constrained, approximate, and unsupported entries are not advertised as fully implemented.
- [ ] Release bundles contain current profiles, reports, manifests, checksums, source revisions, tool versions, and notes.
- [ ] Gates fail on stale generated outputs, native-dependency drift, artifact threshold failures, unsupported-mode claim drift, or profile mismatches.
