# Phase 3 — Compatibility Catalog Completion

## Purpose

Complete the interoperability work needed for the target house-system and ayanamsa catalogs while keeping release profiles truthful about what is implemented, aliased, approximate, constrained, or still missing.

## Spec drivers

- `spec/requirements.md`: FR-4, FR-5, FR-6, FR-10, NFR-5
- `spec/astrology-domain.md`: house systems, ayanamsa model, catalog management, numerical rules
- `spec/api-and-ergonomics.md`: extensible strongly typed identifiers and structured errors
- `spec/validation-and-testing.md`: house/ayanamsa unit, golden, regression, and compatibility-profile tests

## Current baseline

The repository already exposes baseline house systems and ayanamsas, a larger release catalog, aliases, custom identifiers, sidereal conversion helpers, house placement, and compatibility-profile output. Tests confirm catalog round-trips and representative calculations.

## Remaining implementation goals

1. Audit the target catalog against external interoperability expectations.
   - Maintain a source table for Swiss-Ephemeris-class house codes, names, aliases, and ayanamsa labels.
   - Mark each item as implemented, metadata-only, approximate, unsupported, or deferred.
   - Ensure release profiles distinguish baseline guarantees from expanded release coverage.

2. Validate house-system formulas.
   - Add reference-backed tests for each implemented house system across representative latitudes and dates.
   - Document formulas, required astronomical inputs, and numerical assumptions.
   - Return explicit errors for polar/high-latitude or numerical failure cases.
   - Verify special shapes such as Gauquelin sectors and equal/MC/Aries variants.
   - Progress note: the house-formula regression slice now also covers both invalid-latitude rejection and a Placidus polar numerical-failure path, so the current house-system implementation explicitly exercises representative invalid-input and numerical-failure reporting for the compatibility catalog.

3. Complete ayanamsa metadata and formula coverage.
   - Fill remaining reference epochs, offsets, drift/precession assumptions, and provenance notes.
   - Ensure aliases resolve without ambiguity and do not overclaim equivalence.
   - Validate sidereal offsets at canonical epochs.
   - Preserve custom ayanamsa support for user-defined formulas or offset tables.

4. Strengthen derived domain helpers.
   - Continue separating domain logic from backend crates.
   - Add tests for sign placement, house placement, aspects, retrograde/stationary classification, and speed summaries where public APIs expose them.
   - Document angle normalization and wrap behavior for all catalog-facing APIs.

5. Integrate catalog evidence with release profiles.
   - Add profile verification that every listed built-in has a descriptor, alias policy, and implementation status.
   - Include known gaps, latitude constraints, and naming differences in generated reports.
   - Avoid claiming full target coverage until all entries are implemented and validated.
   - Progress note: compatibility-profile verification now also pins remaining release-profile spellings for Suryasiddhanta (Revati), Suryasiddhanta (Citra), True Pushya (PVRN Rao), the Dhruva Galactic Center (Middle Mula) aliases, and several still-visible source-name forms such as Equal/MC = 10th, Equal Midheaven table of houses, Vehlow Equal table of houses, Nick Anthony Fiorenza, Bob Makransky, Treindl Sunshine, True galactic equator, and Galactic equator true, so the current ayanamsa/source-label surface stays anchored as the catalog continues to grow.

## Done criteria

- Every release-profile catalog entry is backed by descriptor metadata and tests.
- Implemented house systems have formula documentation, representative reference tests, and explicit failure modes.
- Implemented ayanamsas have reference metadata and canonical epoch tests.
- Compatibility-profile verification fails on missing descriptors, unverified aliases, or unsupported entries advertised as implemented.
- Adding future catalog entries remains non-breaking.

## Work that belongs in other phases

- Backend accuracy and source-reader work belongs to Phase 1.
- Compressed artifact generation belongs to Phase 2.
- Final release publication gates belong to Phase 4.
