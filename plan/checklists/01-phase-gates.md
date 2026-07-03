# Checklist 1 — Phase Gates

Use this checklist for active implementation only. Completed historical work
should not be re-added.

## Phase 1: Production reference backend and corpus

Complete — no open gates.

## Phase 2: Release-grade compressed ephemeris

Complete — no open gates.

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

- [ ] Native sidereal backend output is implemented with evidence or explicitly
      unsupported.

## Phase 5: Compatibility and release gates

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
