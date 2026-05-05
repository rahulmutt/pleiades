# Phase 3 — Compatibility Evidence and Catalog Truthfulness

## Purpose

Turn the broad house and ayanamsa catalogs into release-grade interoperability evidence. The catalogs already exceed the baseline milestone in several areas; the remaining work is to prove or accurately caveat formulas, aliases, sidereal metadata, custom-definition behavior, latitude constraints, and release-profile claims.

## Spec drivers

- `requirements.md` FR-4, FR-5, FR-6, FR-10, NFR-5
- `astrology-domain.md` house systems, ayanamsa model, catalog management, derived quantities, numerical rules
- `api-and-ergonomics.md` type safety, domain transforms, batch-oriented façade expectations
- `validation-and-testing.md` house, ayanamsa, golden, and regression tests

## Current baseline

Implemented and not re-planned here:

- typed house-system and ayanamsa identifiers with custom-definition support;
- baseline house and ayanamsa milestone entries plus additional release-specific catalog descriptors;
- alias resolution, Swiss Ephemeris-style house-code aliases, descriptor validation, formula-family summaries, catalog inventories, and compatibility-profile verification;
- chart façade support for sidereal conversion, house calculation, sign placement, house placement, aspects, and summaries;
- validation/reporting surfaces for house scenarios, ayanamsa reference offsets, metadata coverage, compatibility caveats, and release profile summaries.

## Remaining implementation goals

### 1. Audit house-system implementations

- For every release-advertised built-in house system, document formula family, assumptions, aliases, implementation status, and latitude/numerical constraints.
- Add golden tests for representative latitudes, hemispheres, equatorial cases, and polar/high-latitude failure cases.
- Confirm cusp normalization, cusp ordering, ASC/MC/IC/DSC behavior, Gauquelin-sector behavior where applicable, and house-placement semantics.
- Mark descriptor-only, approximate, or constrained systems honestly in compatibility profiles and docs.

### 2. Audit ayanamsa definitions

- For every release-advertised built-in ayanamsa, document reference epoch, offset/formula, aliases, provenance, sidereal metadata, and equivalence caveats.
- Add golden tests for baseline entries and representative release-specific entries.
- Keep custom ayanamsa definitions distinguishable from built-ins in reporting and serialization.
- Resolve or explicitly classify entries that are currently descriptor/custom-definition posture only.

### 3. Maintain truthful release compatibility profiles

- Generate profiles that state target scope, baseline milestone, shipped entries, aliases, constraints, custom-definition posture, and known gaps.
- Make verification fail if unsupported, descriptor-only, approximate, or constrained entries are advertised as fully implemented.
- Keep profile summaries synchronized with CLI/validation reports, release notes, and release bundles.
- Avoid claiming full target-catalog coverage until implementation and validation evidence match that claim.

### 4. Strengthen chart-domain ergonomics

- Keep sidereal conversion in the domain layer unless a backend explicitly advertises equivalent native behavior.
- Preserve typed errors for invalid observer locations, unsupported house systems, numerical failures, and unsupported catalog entries.
- Add batch-friendly façade helpers only where they reduce low-level orchestration without hiding assumptions.
- Extend higher-level derived quantities only with clear units, normalization rules, tests, and failure modes.

## Done criteria

Phase 3 is complete when:

- every release-advertised house and ayanamsa entry has descriptor metadata, aliases, implementation status, constraints, and validation evidence;
- latitude-sensitive and numerical failure modes are covered by tests and release-profile caveats;
- profile verification fails closed on unsupported, descriptor-only, approximate, or overstated catalog claims;
- public docs/rustdoc explain core house, ayanamsa, sidereal, and chart-domain behavior;
- standard format, lint, and test checks pass.

## Deferred to other phases

- Backend accuracy and source-reader changes belong to Phase 1.
- Production compressed artifact generation belongs to Phase 2.
- Final release-bundle publication belongs to Phase 4.
