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
   - Progress note: the Sunshine release-system regression now also checks the documented axis anchors explicitly, keeping the release-specific Sunshine house formula covered as more formula-validation slices land.

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
   - Progress note: the validate-command compatibility-profile and release-notes regressions now also pin the Pullen SD (Neo-Porphyry) and Makransky Sunshine spellings, keeping the current release-facing Sunshine/Pullen label variants visible alongside the broader alias audit.
   - Progress note: the compatibility-profile smoke tests now also pin the long-form Sunshine alias `Sunshine table of houses, by Bob Makransky`, keeping that release-facing house-system spelling visible alongside the shorter Makransky and Treindl variants in the catalog audit.
   - Progress note: the latest compatibility-profile regression slice now also anchors the `Equal Midheaven house system`, `Vehlow Equal house system`, `Topocentric house system`, `Whole Sign house system`, `Gal. Center = 0 Sag`, and `Gal. Center = 0 Cap` spellings, extending the release-facing alias coverage for the still-visible house and reference-frame variants.
   - Progress note: the compatibility-profile and release-notes regressions now also pin `Babylonian (Eta Piscium)`, and the compatibility-profile command now keeps its documented alias spellings (`Babylonian/Eta Piscium`, `Babylonian Eta Piscium`, and `Eta Piscium`) visible in the release-facing catalog audit.
   - Progress note: the compatibility-profile verification command now also renders the latitude-sensitive house-system subset as structured evidence, so the release-facing catalog audit calls out the polar-failure-constraint systems explicitly instead of only surfacing them through the summary display.
   - Progress note: the compact compatibility-profile summary and verification output now also surface the exact release-specific house-system and ayanamsa canonical-name lists, so the current release slice names are visible directly in the audit output instead of only through count-based summaries.
   - Progress note: the compact compatibility-profile canonical-name summaries now preflight through a typed descriptor-name validator before formatting, so the release-facing house-system and ayanamsa name lists fail closed if a malformed summary ever slips in.
   - Progress note: `pleiades-types::CustomHouseSystem::validate()` now rejects aliases that case-insensitively collide with each other or with the canonical name, so user-defined house-system definitions fail closed before they can introduce ambiguous label drift into the release profile.
   - Progress note: the shared custom-definition validators now also run during chart and backend request preflight, so malformed custom bodies, house systems, and ayanamsas fail closed before request dispatch instead of only being caught by catalog/report formatting.
   - Progress note: the compatibility-profile and release-notes regressions now also pin the remaining Equal-from-MC label family — including `Equal from MC`, `Equal (from MC)`, and `Equal (from MC) table of houses` — so the equal-house appendix keeps those Midheaven-anchored forms explicit alongside the already-covered `Equal/MC` variants.
   - Progress note: the compatibility-profile and release-summary smoke tests now also pin `Albategnius` and `Gauquelin sectors`, keeping the remaining release-specific house-system additions visible in the compact release-facing catalog views as the broader alias audit continues.
   - Progress note: compatibility-profile and release-notes regressions now also pin the Dhruva/Gal.Center/Mula (Wilhelm), Mula Wilhelm, and Wilhelm spellings, keeping the remaining Mula/Wilhelm galactic-reference variants visible in the release-facing catalog audit.
   - Progress note: the compatibility-profile verification slice now also pins a few additional house-system spellings that still surface in the release catalog — `Polich Page`, `Poli-Equatorial`, `Equal Quadrant`, `Meridian table of houses`, and `Whole-sign` — so the alias audit keeps both the resolution helpers and the release-facing text anchored as the catalog grows.
   - Progress note: that same smoke coverage now also exercises the `Equal Midheaven house system` spelling in the CLI and validation release-note paths, which keeps another still-visible equal-house alias from drifting out of the user-facing compatibility audit.
   - Progress note: the CLI compatibility-profile smoke tests now also pin a few high-signal release-profile house labels — `Equal (cusp 1 = Asc)`, `Equal from MC`, `WvA`, and `Gauquelin table of sectors` — so the command-line release view keeps the current source-label appendix and interoperability aliases anchored alongside the validation-side catalog checks.
   - Progress note: the CLI and validation compatibility-profile smoke tests now also pin `Nick Anthony Fiorenza`, `Galactic Center (Cochrane)`, and `P.V.R. Narasimha Rao`, extending the release-facing galactic and equal-house label coverage a little further without changing the published catalog shape.
   - Progress note: the CLI compatibility-profile smoke tests now also pin the `Whole Sign system`, `Whole Sign house system`, and `Whole Sign (house 1 = Aries)` spellings, which keeps the whole-sign family anchored in the command-line catalog audit alongside the already-covered equal-house and topocentric variants.
   - Progress note: the compatibility-profile verification now also rejects case-insensitive duplicate labels that would collide across different house-system, ayanamsa, or custom-definition entries, while still allowing same-entry alias spellings such as the current `Vehlow Equal`/`Vehlow equal` pair to coexist, so future catalog additions cannot silently drift into ambiguous cross-entry alias collisions even when spellings differ only by ASCII case. The same verification path now also checks that the freeform release-note, validation-reference-point, and compatibility-caveat prose sections stay pairwise disjoint, which keeps duplicated narrative from drifting between the release-facing text blocks. The prose-section verifier now also rejects case-insensitive duplicate entries within a section and across sections, so release-note drift cannot hide behind capitalization changes alone when the compatibility profile is revalidated.
   - Progress note: compatibility-profile verification now also compares the canonical house-system and ayanamsa descriptor names across the total catalog and the baseline/release partitions, so descriptor-count parity can no longer hide a partition that quietly drops or renames one of the published built-ins.
   - Progress note: `pleiades-core::CompatibilityProfile` now also exposes a typed `validate()` helper for the profile text sections and baseline/release partitions, and the validation summary preflights that helper before rendering the release-facing compatibility-profile checks, so malformed profile metadata now fails closed at the core boundary instead of only in the report formatter.
   - Progress note: compatibility-profile verification now also rejects whitespace-padded canonical names, notes metadata, and labels, which closes another small catalog-drift gap before the release profile can absorb a malformed descriptor or alias string. The same verification pass now also checks the freeform release-note, validation-reference-point, and compatibility-caveat sections for blank, whitespace-padded, or duplicate entries, keeping the profile prose a little less likely to drift silently. The standalone text-section disjointness helper now also rejects blank and whitespace-padded entries directly, and the regression suite now exercises a whitespace-padded cross-section case explicitly, which keeps the prose drift guard robust even when the helper is reused outside the main profile verification path.
   - Progress note: the compatibility-profile smoke tests now also pin `Hipparchus`, `Djwhal Khul`, `Udayagiri`, and `True Mula`, keeping a few more release-profile ayanamsa labels explicitly covered in the CLI and validation audit paths as the catalog breadth pass continues.
   - Progress note: the compatibility-profile smoke tests now also pin the Raman appendix spellings `Raman Ayanamsha` and `Raman ayanamsa`, keeping that baseline ayanamsa alias group visible in the CLI and validation audit paths alongside the existing `B. V. Raman` coverage.
   - Progress note: the validation and release summary views now also surface the compact house-validation corpus line from the representative baseline scenarios, so the formula-validation slice is visible in the short release-facing reports instead of only in the detailed house report. The built-in house and ayanamsa descriptor records now also expose compact summary-line/Display helpers, and the compatibility-profile formatter reuses those typed records directly so the release-facing catalog sections stay co-located with the descriptor definitions.
   - Progress note: topocentric house calculations now have a regression that checks the geocentric latitude correction path against a matching Placidus reference, while keeping the returned snapshot angles tied to the original observer request; that makes the topocentric house wrapper's observer-correction contract explicit in the derived-domain test surface.
   - Progress note: the Gauquelin release-system regression now also checks the 36-sector spacing in the descending wraparound order in addition to the existing anchor points, so the special 36-cusp shape is exercised as part of the formula-validation slice instead of only as a sector-count check.
   - Progress note: `HouseSnapshot::validate()` now also enforces the expected cusp cardinality for each supported system, so malformed manual snapshots for Gauquelin and the 12-cusp systems now fail closed before derived-domain consumers can treat them as structurally sound.

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
