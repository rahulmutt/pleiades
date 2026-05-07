# Checklist 1 — Phase Gates

## Phase 1: Production compressed data

- [ ] Artifact generator consumes documented validated inputs.
- [ ] Normalized intermediates are deterministic and checksumed.
- [ ] Artifact profile states stored, derived, and unsupported outputs.
- [ ] Body/channel fit errors pass published thresholds for advertised scopes.
- [ ] Reference and hold-out validation reports are current.
- [ ] Lookup, batch, decode, size, and chart benchmarks are current.

## Phase 2: Production reference inputs

- [ ] Source ingestion/generation path is pure Rust and documented.
- [ ] Public provenance, frame, time scale, and checksum expectations are recorded.
- [ ] Validation corpus covers release-claimed bodies and date ranges.
- [ ] Evidence classes remain separated in reports.
- [ ] Pluto/lunar/asteroid claims match source-backed evidence.

## Phase 3: Advanced request support

- [ ] UTC/Delta-T policy is either implemented or explicitly deferred.
- [ ] Apparent-place behavior is implemented or rejected with structured errors.
- [ ] Topocentric body-position behavior is implemented or rejected with structured errors.
- [ ] Native sidereal backend output is implemented or explicitly unsupported.
- [ ] Backend matrices, rustdoc, CLI output, and tests agree.

## Phase 4: Compatibility catalog evidence

- [ ] House formulas, aliases, and latitude/numerical constraints are audited for release-claimed entries.
- [ ] Ayanamsa reference epochs, offsets, formulas, aliases, and provenance are audited for release-claimed entries.
- [ ] Descriptor-only/custom/constrained entries are not advertised as fully implemented.
- [ ] Compatibility-profile verification fails closed on overstated claims.

## Phase 5: Release gate hardening

- [ ] Formatting, clippy, tests, audits, artifact validation, compatibility verification, benchmarks, and bundle verification are reproducible.
- [ ] Release bundle contains current profiles, reports, manifests, checksums, and notes.
- [ ] Gates fail on stale generated outputs, native-dependency drift, artifact threshold failures, or claim/profile mismatches.
