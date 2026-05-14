# Checklist 1 — Phase Gates

## Phase 1: Artifact accuracy and packaged-data production

- [ ] Artifact generator consumes validated documented inputs.
- [ ] Normalized intermediates are deterministic and checksumed.
- [ ] Artifact profile states stored, derived, approximated, and unsupported outputs.
- [ ] Body/channel errors pass published thresholds for advertised scopes.
- [ ] Reference and hold-out validation reports are current.
- [ ] Lookup, batch, decode, size, and chart-style benchmarks are current.

## Phase 2: Reference/source corpus productionization

- [ ] Source ingestion/generation path is pure Rust and documented.
- [ ] Public provenance, frame, time scale, columns, source revision, and checksums are recorded.
- [ ] Validation corpus covers release-claimed bodies, frames, channels, and date ranges.
- [ ] Reference, hold-out, boundary, fixture-exactness, and provenance-only evidence stay separated.

## Phase 3: Body-model completion and claim boundaries

- [ ] Pluto status is source-backed, artifact-backed, approximate, constrained, or excluded in every release surface.
- [ ] Lunar theory and lunar-point claims match implemented formulas and validation windows.
- [ ] Selected asteroid claims match source-backed evidence and backend metadata.
- [ ] Custom/numbered body support is extensible without overstating backend coverage.

## Phase 4: Advanced request modes and policy

- [ ] UTC/Delta-T behavior is implemented with evidence or explicitly deferred.
- [ ] Apparent-place behavior is implemented with evidence or rejected with structured errors.
- [ ] Topocentric body-position behavior is implemented with evidence or rejected with structured errors.
- [ ] Native sidereal backend output is implemented with evidence or explicitly unsupported.
- [ ] Backend matrices, rustdoc, CLI output, release reports, and tests agree.

## Phase 5: Compatibility catalog evidence

- [ ] House formulas, aliases, and latitude/numerical constraints are audited for release-claimed entries.
- [ ] Ayanamsa reference epochs, offsets, formulas, aliases, and provenance are audited for release-claimed entries.
- [ ] Descriptor-only, custom-only, constrained, approximate, and unsupported entries are not advertised as fully implemented.
- [ ] Compatibility-profile verification fails closed on overstated claims.

## Phase 6: Release gate hardening

- [ ] Formatting, clippy, tests, audits, artifact validation, compatibility verification, benchmarks, and bundle verification are reproducible.
- [ ] Release bundle contains current profiles, reports, manifests, checksums, source revisions, tool versions, and notes.
- [ ] Gates fail on stale generated outputs, native-dependency drift, artifact threshold failures, unsupported-mode claim drift, or profile mismatches.
