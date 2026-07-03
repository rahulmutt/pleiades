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

- [x] All target `compatibility-catalog.md` house systems are implemented with
      formula, aliases, and provenance (25 built-ins, 24 gated). Only Albategnius
      corpus-gating remains (optional; beyond the SE 23-code target).
- [ ] Ayanamsa catalog: gate the 11 remaining descriptor-only modes and add any
      missing `SE_SIDM_*` modes with epoch/offset/formula, aliases, and provenance,
      or list known gaps (48 of 59 built-ins currently SE-gated).
- [ ] Selected-asteroid coverage expands only where source evidence and backend
      metadata support release-grade claims.
- [ ] Dignities are shipped with tests and rustdoc, or not advertised. (Aspects /
      orb-ready separations are already implemented in `pleiades-core::chart::aspects`.)
- [ ] No expansion broadens public claims ahead of its supporting evidence, and
      the public API/enums absorb new entries without breaking redesign.
