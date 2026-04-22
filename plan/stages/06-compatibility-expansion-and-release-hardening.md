# Stage 6 — Compatibility Expansion and Release Hardening

## Goal
Close the gap between the baseline milestone and the full target compatibility catalog while making releases reliable, well-documented, and easy to integrate.

## Why this stage comes last
Breadth and polish should build on a proven foundation: stable types, useful MVP functionality, validated references, and packaged-data performance.

## Primary deliverables

### Compatibility completion
- remaining house systems needed for the target compatibility catalog
- remaining ayanamsas needed for the target compatibility catalog
- clear alias mapping and interoperability notes versus other astrology software
- maintained versioned compatibility profile per release

### Hardening
- stronger benchmark corpus
- wider regression corpus
- public capability and accuracy documentation for every backend
- API stabilization review and deprecation policy as needed
- release checklist spanning docs, artifacts, validation reports, and environment reproducibility

### Optional expansion
- richer composite backend routing
- more asteroid coverage
- topocentric refinements
- optional higher-level chart helpers beyond the core MVP

## Workable state at end of stage
The project is not just functional but dependable: consumers can tell exactly what compatibility they are getting in each release, performance and accuracy are characterized, and extension paths remain open.

## Suggested implementation slices

1. Turn the compatibility profile into a routine release artifact before adding substantial new catalog breadth.
2. Complete remaining house systems and ayanamsas in prioritized batches, grouped by shared formulas or interoperability value.
3. Add interoperability tests for naming, alias behavior, and documented constraints as each batch lands.
4. Harden CI and release automation around validation, report publication, and artifact publication.
5. Review public APIs for long-term stability, deprecations, and documented intentional limitations.
6. Expand optional higher-level helpers only after the compatibility and release story is already dependable.

This final stage should behave like a sequence of release-quality increments, not a catch-all bucket for unfinished foundational work.

## Progress update

Stage 6 release hardening has started as of 2026-04-22.

- [x] The compatibility profile now distinguishes target scope, baseline milestone, release-specific coverage, and known gaps.
- [x] Validation reports now include the release compatibility profile so the stage-6 release artifact bundle carries the current coverage summary.
- [ ] Remaining catalog breadth and release automation are still the next planned slices.

## Exit criteria

- release compatibility profile is published and current
- target compatibility catalog is fully implemented or remaining gaps are explicitly scheduled and justified
- release gates from `spec/validation-and-testing.md` are automated where practical
- maintainers can reproduce tools, builds, tests, validation, and artifacts from repo docs

## Risks to avoid

- adding catalog breadth without maintaining the compatibility profile
- expanding optional helpers in ways that blur crate boundaries
- declaring stability without validation, documentation, and release discipline
