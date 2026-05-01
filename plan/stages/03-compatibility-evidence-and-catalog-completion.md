# Phase 3 — Compatibility Evidence and Catalog Completion

## Purpose

Turn the existing broad house and ayanamsa catalogs into release-grade interoperability evidence. The catalogs are already larger than the baseline milestone; the remaining work is to prove formulas, aliases, sidereal metadata, custom-definition behavior, latitude constraints, and release-profile claims.

## Spec drivers

- `requirements.md` FR-4, FR-5, FR-6, FR-10, NFR-5
- `astrology-domain.md` house systems, ayanamsa model, catalog management, derived quantities, numerical rules
- `api-and-ergonomics.md` type safety, domain transforms, batch-oriented façade expectations
- `validation-and-testing.md` house, ayanamsa, golden, and regression tests

## Current baseline

Implemented and not re-planned here:

- strongly typed house-system and ayanamsa identifiers with custom-definition support;
- baseline house milestone entries and additional release-specific house entries;
- baseline ayanamsa entries and many release-specific ayanamsa descriptors;
- alias resolution, Swiss Ephemeris-style house code aliases, descriptor validation, catalog summaries, and compatibility-profile verification;
- chart façade support for sidereal conversion, house calculation, sign placement, house placement, aspects, summaries, and report/CLI integration;
- validation corpus and summaries for current house scenarios and ayanamsa catalog integrity.

Known remaining gaps:

- Not every catalog entry has independent formula/reference validation strong enough for interoperability claims.
- Latitude-sensitive and numerical failure modes need fuller reference scenarios and release-profile wording.
- Some ayanamsa entries are descriptor/custom-definition posture only and need explicit formula/offset provenance or known-gap treatment.
- Full target compatibility catalog policy needs a maintained completeness audit against the intended Swiss-Ephemeris-class scope.
- Optional higher-level derived chart utilities remain incremental and should not obscure core catalog truthfulness.

## Remaining implementation goals

### 1. Audit house-system implementations

- For every advertised built-in house system, document formula family, assumptions, aliases, and latitude/numerical constraints.
- Add golden tests for representative latitudes, hemispheres, and polar/high-latitude failure cases.
- Confirm cusp normalization, angle ordering, ASC/MC/IC/DSC behavior, and house placement semantics.
- Mark descriptor-only, approximate, or constrained systems honestly in compatibility profiles.

### 2. Audit ayanamsa definitions

- For every advertised built-in ayanamsa, document reference epoch, offset/formula, aliases, provenance, and known equivalence caveats.
- Add golden tests for baseline entries and representative release-specific entries.
- Ensure custom ayanamsa definitions remain distinguishable from built-ins in reporting and serialization.
- Resolve or explicitly classify entries with custom-definition-only metadata.

### 3. Maintain release compatibility profiles

- Generate profiles that state current target scope, baseline milestone, shipped entries, aliases, constraints, and known gaps.
- Make verification fail if an unsupported or descriptor-only entry is advertised as fully implemented.
- Keep profile summaries synchronized with CLI, validation, release notes, and release bundles.
- Avoid claiming full target-catalog coverage until the catalog and validation evidence actually match.

### 4. Strengthen chart-domain ergonomics

- Keep sidereal conversion in the domain layer unless a backend explicitly advertises native equivalent behavior.
- Preserve typed errors for invalid observer locations, unsupported house systems, numerical failures, and unsupported catalog entries.
- Add batch-friendly APIs where chart workloads still require low-level orchestration.
- Extend derived quantities only when they are backed by clear units, normalization rules, tests, and failure modes.

## Done criteria

Phase 3 is complete when:

- every release-advertised house and ayanamsa entry has descriptor metadata, alias coverage, implementation status, and validation evidence;
- latitude-sensitive and numerical failure modes are covered by tests and release-profile caveats;
- profile verification fails closed on unsupported or overstated catalog claims;
- public docs/rustdoc explain core house, ayanamsa, sidereal, and chart-domain behavior;
- standard workspace format, lint, and test commands pass.

## Work intentionally deferred

- Backend accuracy and source-reader changes belong to Phase 1.
- Production compressed artifact generation belongs to Phase 2.
- Final release bundle publication belongs to Phase 4.
