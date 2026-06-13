# Checklist 1 — Phase Gates

Use this checklist for active implementation only. Completed historical work
should not be re-added.

## Phase 1: Production reference backend and corpus

- [ ] Production source strategy is implemented and pure-Rust compatible.
- [ ] Public source provenance, redistribution posture, schema, frame, time
      scale, source revision, generation command, and checksums are recorded.
- [ ] Reference, fitting, hold-out, boundary, fixture-exactness, and
      provenance-only evidence are separated.
- [ ] Corpus coverage satisfies release-claimed body/channel/frame/date needs.
- [ ] Corpus validation fails on body, epoch, channel, frame, apparentness,
      schema, source-revision, or checksum drift.
- [x] Backend matrices and release profiles derive body/date/channel claims from
      validated corpus evidence.

## Phase 2: Release-grade compressed ephemeris

- [ ] Artifact generation consumes only validated Phase 1 source inputs.
- [ ] Stored, derived, approximated, and unsupported outputs are explicit in the
      artifact profile.
- [ ] Body/channel errors pass published production thresholds against reference
      and hold-out corpora.
- [ ] Lookup, batch, decode, size, and chart-style benchmarks are current.
- [ ] Regeneration is deterministic and sidecars/manifests/checksums verify from
      a clean checkout.

## Phase 3: Body/backend claim closure

- [ ] Pluto status is source-backed, artifact-backed, approximate, constrained,
      or excluded in every release surface.
- [ ] Lunar theory and lunar-point claims match implemented formulas and
      validation windows.
- [ ] Selected asteroid claims match source-backed evidence and backend metadata.
- [ ] Custom/numbered body support is extensible without overstating backend
      coverage.
- [ ] Backend capability matrices agree with actual supported bodies, dates,
      frames, time scales, channels, and request modes.

## Phase 4: Request-mode semantics

- [ ] UTC/UT1 and Delta-T behavior is implemented with evidence or explicitly
      deferred.
- [ ] Apparent-place behavior is implemented with evidence or rejected with
      structured errors.
- [ ] Topocentric body-position behavior is implemented with evidence or rejected
      with structured errors.
- [ ] Native sidereal backend output is implemented with evidence or explicitly
      unsupported.
- [ ] Motion/speed/retrograde policies agree across backend matrices, rustdoc,
      CLI output, release reports, and tests.

## Phase 5: Compatibility and release gates

- [ ] House formulas, aliases, and latitude/numerical constraints are audited for
      release-claimed entries.
- [ ] Ayanamsa reference epochs, offsets, formulas, aliases, and provenance are
      audited for release-claimed entries.
- [ ] Descriptor-only, custom-only, constrained, approximate, and unsupported
      entries are not advertised as fully implemented.
- [ ] Release bundles contain current profiles, reports, manifests, checksums,
      source revisions, tool versions, and notes.
- [ ] Gates fail on stale generated outputs, native-dependency drift, artifact
      threshold failures, unsupported-mode claim drift, missing evidence, or
      profile mismatches.
- [ ] Build and test pass on supported platforms (Linux, macOS, Windows) per
      `requirements.md` NFR-6.

## Phase 6: Target catalog completion and expansion (end-state, post-first-release)

- [ ] Remaining `compatibility-catalog.md` house systems are implemented with
      formula, aliases, latitude/numerical constraints, and provenance, or listed
      as known gaps in the release profile.
- [ ] Ayanamsa catalog grows toward the full Swiss Ephemeris `SE_SIDM_*` set with
      epoch/offset/formula, aliases, and provenance, or lists known gaps.
- [ ] Selected-asteroid coverage expands only where source evidence and backend
      metadata support release-grade claims.
- [ ] Optional chart utilities (aspects/orb-ready separations, dignities) are
      shipped with tests and rustdoc, or not advertised.
- [ ] No expansion broadens public claims ahead of its supporting evidence, and
      the public API/enums absorb new entries without breaking redesign.
