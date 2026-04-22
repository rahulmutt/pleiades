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
- [x] `pleiades-validate bundle-release --out DIR` now writes the compatibility profile, API stability posture, validation report, and a manifest for a reproducible release bundle.
- [x] The release bundle manifest now records deterministic FNV-1a checksums for the published text artifacts, and the CLI bundle summary surfaces those checksums for release verification.
- [x] `mise run release-smoke` now exercises the release bundle command locally and in CI so release artifacts stay under automation.
- [x] The first release-specific house-system additions are now implemented: Equal (MC), Equal (1=Aries), Vehlow Equal, and Sripati are catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The first release-specific ayanamsa additions are now implemented: Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), and Sassanian are catalogued, resolved, rendered in the compatibility profile, and exposed through the shared ayanamsa resolution path.
- [x] The next ayanamsa breadth batch is now implemented too: DeLuce and Yukteshwar are catalogued, resolved, rendered in the compatibility profile, and available through the shared catalog path alongside the existing release-specific ayanamsas.
- [x] Udayagiri has now been added as the next historical/reference-frame ayanamsa batch entry, keeping the release profile synchronized with the catalog breadth increments.
- [x] Swiss Ephemeris reference-frame and true-nakshatra ayanamsa modes are now catalogued too: J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), and Suryasiddhanta (Citra) are resolved, rendered in the compatibility profile, and available through the shared catalog path.
- [x] Dhruva Galactic Center (Middle Mula) now rounds out the galactic-reference ayanamsa batch, with catalog resolution, compatibility-profile rendering, and CLI/validation visibility aligned to the release-specific breadth notes.
- [x] The compatibility profile identifier has been bumped to `0.6.9` to reflect the new Dhruva Galactic Center (Middle Mula) catalog breadth, and the profile remains synchronized with the built-in ayanamsa catalog and validation output.
- [x] The PVR Pushya-paksha and Sheoran ayanamsa modes are now catalogued too: both are resolved, rendered in the compatibility profile, and available through the shared catalog path as the next Stage 6 ayanamsa breadth increment.
- [x] Swiss Ephemeris historical/reference-frame ayanamsa modes now include Hipparchus, Babylonian (Kugler 1/2/3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Galactic Center, and Galactic Equator; they are catalogued, resolved, and surfaced through the compatibility profile as the next breadth batch.
- [x] The remaining named legacy ayanamsa modes and formula variants from the Swiss Ephemeris header are now catalogued too: True Pushya, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, and the remaining Galactic Center / Galactic Equator variants are resolved and visible in the compatibility profile.
- [x] Carter (poli-equatorial) is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Horizon/Azimuth and APC are now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Krusinski-Pisa-Goelzer is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Albategnius and the Pullen sinusoidal house families are now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The Sunshine house family is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The Gauquelin-sector family is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The public API stability posture is now published through `pleiades-core`, surfaced in the CLI, and included in validation reports so consumers can tell which surfaces are stable versus tooling-internal.
- [x] Catalog-alignment invariant tests now keep the built-in house-system and ayanamsa catalogs synchronized with the release compatibility profile, including the latest Valens Moon, SS Revati/Citra, and Dhruva Galactic Center (Middle Mula) release-note coverage, and the profile identifier was bumped to `0.6.9`.
- [x] The CLI chart workflow now routes selected asteroid bodies through the JPL snapshot fallback at supported comparison epochs, so the release-line composite backend can exercise the stage-4 asteroid coverage without changing the primary packaged/chart path.
- [x] The backend routing layer now supports an n-provider prioritized router, and the CLI chart path uses it to compose packaged, algorithmic, and reference-data backends without nesting binary composites.
- [x] The chart request façade now makes apparent-versus-mean selection explicit so callers can opt into backend position queries without the chart layer hiding that assumption.
- [x] The validation CLI help text now documents the `generate-report` alias alongside `report`, keeping the release tooling and command-line documentation aligned.
- [x] Babylonian (Huber), Galactic Equator (IAU 1958), and Valens Moon now carry explicit sidereal-offset reference metadata, and the compatibility profile notes them as exceptions to the broader ayanamsa-metadata gap.
- [x] Release bundle generation now has a matching verification command that re-reads the staged artifacts, checks the manifest checksums, and is exercised by the release smoke task so the release artifact path is validated end to end.
- [x] Topocentric (Polich-Page) now uses a geodetic-to-geocentric latitude correction with elevation-aware ellipsoid handling, and the catalog note reflects the refined implementation.
- [x] The chart façade now exposes a motion-direction helper for body placements, so future chart reports and consumers can surface retrograde/direct state from backend motion data without adding backend-specific logic.
- [x] `pleiades-core::ChartSnapshot` now offers direct body lookup and retrograde-summary helpers, and chart rendering emits a retrograde-body summary when motion data is available.
- [x] `pleiades-cli` now exposes the implemented backend capability matrices directly via `backend-matrix` / `capability-matrix`, so maintainers can inspect coverage, range, and accuracy notes from the user-facing CLI.
- [x] The compatibility profile now renders an explicit alias-mapping appendix for built-in house systems and ayanamsas, making interoperability lookups easier to audit from the release artifact itself.
- [x] The compatibility profile now also captures a couple of Swiss-Ephemeris label variants that were still missing from the release notes layer: `Whole Sign (house 1 = Aries)` resolves to `Equal (1=Aries)`, and `Moon` resolves to `Valens Moon`.
- [x] The backend capability matrices now render each backend's nominal supported time range in addition to accuracy and body coverage, making the release-facing capability docs easier to audit.

## Exit criteria

- release compatibility profile is published and current
- target compatibility catalog is fully implemented or remaining gaps are explicitly scheduled and justified
- release gates from `spec/validation-and-testing.md` are automated where practical
- maintainers can reproduce tools, builds, tests, validation, and artifacts from repo docs

## Risks to avoid

- adding catalog breadth without maintaining the compatibility profile
- expanding optional helpers in ways that blur crate boundaries
- declaring stability without validation, documentation, and release discipline
