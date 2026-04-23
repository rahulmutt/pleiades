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

- [x] `pleiades-core` now exposes helper accessors for the current compatibility-profile and API-stability identifiers, and the CLI / validation tests consume those shared helpers instead of repeating release-string literals.
- [x] The house-system source-label appendix now also exposes the remaining Albategnius / Pullen / Gauquelin search forms (`Savard-A`, `Savard A`, `Savard's Albategnius`, `Neo-Porphyry`, `Pullen sinusoidal delta`, `Pullen sinusoidal ratio`, and `G`), and the compatibility profile identifier has been bumped to `0.6.26` so the release artifact stays synchronized with the latest house-label interoperability batch.
- [x] The compatibility profile's APC source-label appendix now includes `APC houses` alongside `Ram school`, `Ramschool`, and `Ascendant Parallel Circle`, and the compatibility profile identifier has been bumped to `0.6.23` so the release artifact stays synchronized with the house-system interoperability labels.
- [x] The Raman ayanamsa resolver now accepts the `B. V. Raman`, `B.V. Raman`, and `B V Raman` spellings directly, and the compatibility profile identifier has been bumped to `0.6.24` so the release artifact stays synchronized with the ayanamsa source-label appendix.
- [x] The compatibility profile now also renders a source-label appendix for the built-in house systems, including the Swiss Ephemeris `Equal (cusp 1 = Asc)` spelling, keeping the release-facing house interoperability map searchable alongside the ayanamsa appendix.
- [x] The compatibility profile's source-label appendix now also surfaces the exact `Suryasiddhanta 499`, `Suryasiddhanta 499 CE`, `Aryabhata 499`, `Aryabhata 499 CE`, and `Aryabhata 522 CE` spellings, and the shared ayanamsa resolver accepts those forms too, keeping the release-facing historical/reference-frame interoperability map aligned with the latest breadth additions. The compatibility profile identifier has been bumped to `0.6.25` so the release artifact stays versioned with that appendix refinement.
- [x] `pleiades-houses` now recognizes `Topocentric house system` as an interoperability alias for `Topocentric`, and the compatibility profile now renders that variant explicitly alongside the existing Polich-Page spellings.
- [x] `pleiades-ayanamsa` now backfills explicit sidereal metadata for `True Sheoran`, `Galactic Center (Rgilbrand)`, and `Galactic Center (Mula/Wilhelm)`, using the published Swiss Ephemeris zero points so the compatibility profile no longer treats those historical/reference-frame entries as metadata gaps.
- [x] `pleiades-ayanamsa` now backfills explicit sidereal metadata for `Udayagiri` and `Lahiri (VP285)`, reusing the Lahiri-family 285 CE zero point so the compatibility profile no longer treats those variants as metadata gaps.
- [x] The compatibility profile now distinguishes target scope, baseline milestone, release-specific coverage, and known gaps.
- [x] The compatibility profile now also includes a compact coverage summary for house-system breadth, ayanamsa breadth, and sidereal-metadata coverage, and the CLI profile rendering follows the same wording.
- [x] Validation reports now include the release compatibility profile so the stage-6 release artifact bundle carries the current coverage summary.
- [x] `pleiades-validate bundle-release --out DIR` now writes the compatibility profile, release notes, release checklist, backend capability matrix, API stability posture, validation report, and a manifest for a reproducible release bundle.
- [x] The benchmark corpus now uses five epochs across the 1500-2500 target window instead of the earlier three-epoch slice, giving the validation report a stronger representative workload for Stage 6 release hardening.
- [x] The release bundle manifest now records deterministic FNV-1a checksums for the published text artifacts, and the CLI bundle summary surfaces those checksums for release verification.
- [x] `mise run release-smoke` now exercises the release bundle command locally and in CI so release artifacts stay under automation.
- [x] The compatibility profile's Valens Moon source-label appendix now also includes the plain `Moon` search term, keeping the release-facing interoperability labels aligned with the existing chart-layer alias resolution.
- [x] `mise run audit` now runs a workspace-native dependency audit, and `pleiades-validate workspace-audit` checks the workspace manifests and lockfile for mandatory native build hooks so the release gates surface pure-Rust regressions early.
- [x] The first release-specific house-system additions are now implemented: Equal (MC), Equal (1=Aries), Vehlow Equal, and Sripati are catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Equal (MC) now also accepts the common `Equal MC` and `Equal Midheaven` label variants, tightening the release-line house-system interoperability mapping.
- [x] The first release-specific ayanamsa additions are now implemented: Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), Sassanian, and the release-line True Citra variant are catalogued, resolved, rendered in the compatibility profile, and exposed through the shared ayanamsa resolution path.
- [x] The next ayanamsa breadth batch is now implemented too: DeLuce and Yukteshwar are catalogued, resolved, rendered in the compatibility profile, and available through the shared catalog path alongside the existing release-specific ayanamsas.
- [x] Udayagiri has now been added as the next historical/reference-frame ayanamsa batch entry, keeping the release profile synchronized with the catalog breadth increments.
- [x] Swiss Ephemeris reference-frame and true-nakshatra ayanamsa modes are now catalogued too: J2000, J1900, B1950, True Revati, True Mula, Suryasiddhanta (Revati), and Suryasiddhanta (Citra) are resolved, rendered in the compatibility profile, and available through the shared catalog path.
- [x] Dhruva Galactic Center (Middle Mula) now rounds out the galactic-reference ayanamsa batch, with catalog resolution, compatibility-profile rendering, and CLI/validation visibility aligned to the release-specific breadth notes.
- [x] The compatibility profile identifier has been bumped to `0.6.9` to reflect the new Dhruva Galactic Center (Middle Mula) catalog breadth, and the profile remains synchronized with the built-in ayanamsa catalog and validation output.
- [x] The PVR Pushya-paksha and Sheoran ayanamsa modes are now catalogued too: both are resolved, rendered in the compatibility profile, and available through the shared catalog path as the next Stage 6 ayanamsa breadth increment.
- [x] Swiss Ephemeris historical/reference-frame ayanamsa modes now include Hipparchus, Babylonian (Kugler 1/2/3), Babylonian (Huber), Babylonian (Eta Piscium), Babylonian (Aldebaran), Galactic Center, and Galactic Equator; they are catalogued, resolved, and surfaced through the compatibility profile as the next breadth batch.
- [x] The remaining named legacy ayanamsa modes and formula variants from the Swiss Ephemeris header are now catalogued too: True Pushya, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, and the remaining Galactic Center / Galactic Equator variants are resolved and visible in the compatibility profile.
- [x] `pleiades-ayanamsa` now also carries explicit sidereal metadata for `Krishnamurti (VP291)` in the built-in catalog, so the compatibility profile and metadata coverage summary no longer treat it as a lingering release-line gap.
- [x] Carter (poli-equatorial) is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Horizon/Azimuth and APC are now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Krusinski-Pisa-Goelzer is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] Albategnius and the Pullen sinusoidal house families are now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The Sunshine house family is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The Gauquelin-sector family is now catalogued, resolved, rendered in the compatibility profile, and calculated in `pleiades-houses`.
- [x] The public API stability posture is now published through `pleiades-core`, surfaced in the CLI, and included in validation reports so consumers can tell which surfaces are stable versus tooling-internal.
- [x] `pleiades-core::ApiStabilityProfile` now explicitly lists `ChartSnapshot` motion summaries among the stable chart-facade helpers, and the generic `placements_with_motion_direction` filter now appears there too, keeping the consumer-facing posture synchronized with the chart helper surface.
- [x] Catalog-alignment invariant tests now keep the built-in house-system and ayanamsa catalogs synchronized with the release compatibility profile, including the latest Valens Moon, SS Revati/Citra, and Dhruva Galactic Center (Middle Mula) release-note coverage, and the profile identifier was bumped to `0.6.9`.
- [x] The CLI chart workflow now routes selected asteroid bodies through the JPL snapshot fallback at supported comparison epochs, so the release-line composite backend can exercise the stage-4 asteroid coverage without changing the primary packaged/chart path.
- [x] The backend routing layer now supports an n-provider prioritized router, and the CLI chart path uses it to compose packaged, algorithmic, and reference-data backends without nesting binary composites.
- [x] The chart request façade now makes apparent-versus-mean selection explicit so callers can opt into backend position queries without the chart layer hiding that assumption.
- [x] The validation CLI help text now documents the `generate-report` alias alongside `report`, keeping the release tooling and command-line documentation aligned.
- [x] Babylonian (Huber), Galactic Center (Cochrane), Galactic Equator (IAU 1958), True Pushya, Djwhal Khul, Sheoran, and Valens Moon now carry explicit sidereal-offset reference metadata, and the compatibility profile notes them as exceptions to the broader ayanamsa-metadata gap.
- [x] Suryasiddhanta (Revati) and Suryasiddhanta (Citra) now carry explicit zero-point epoch metadata too, narrowing the remaining ayanamsa-metadata gap for the reference-frame compatibility batch.
- [x] Galactic Equator (Fiorenza) now carries explicit J2000.0 epoch/offset metadata too, using the published 25° reference value so the galactic-reference batch stays synchronized with the compatibility profile.
- [x] Release bundle generation now has a matching verification command that re-reads the staged artifacts, checks the manifest checksums, and is exercised by the release smoke task so the release artifact path is validated end to end.
- [x] The release checklist artifact now embeds the canonical bundle-generation and verification commands, plus a pointer back to the reproducibility guide, so the release bundle is self-describing for maintainers.
- [x] A release reproducibility guide now documents the canonical build, lint, test, smoke, bundle, and verification commands so maintainers can reproduce the release workflow from repository-managed tooling.
- [x] Topocentric (Polich-Page) now uses a geodetic-to-geocentric latitude correction with elevation-aware ellipsoid handling, and the catalog note reflects the refined implementation.
- [x] The chart façade now exposes a motion-direction helper for body placements, so future chart reports and consumers can surface retrograde/direct state from backend motion data without adding backend-specific logic.
- [x] `pleiades-core::ChartSnapshot` now also exposes a direct-placement helper and a generic motion-direction filter for chart consumers that want to partition placements without rescanning the chart manually.
- [x] `pleiades-core::ChartSnapshot` now also exposes a stationary-placement helper, and chart rendering now surfaces stationary bodies alongside the existing motion summary and retrograde list when that motion class is present.
- [x] `pleiades-core::ChartSnapshot` now also exposes an unknown-motion helper, and chart rendering now surfaces unknown-motion bodies alongside the existing motion summary, stationary bodies, and retrograde list when this motion class is present.
- [x] `pleiades-core::ChartSnapshot` now also exposes a sign-summary helper, so downstream reports can summarize occupied zodiac signs without rescanning the placement list manually.
- [x] `pleiades-core::ChartSnapshot` now also exposes a house-summary helper, and chart rendering now surfaces occupied houses alongside the sign and motion summaries when house data is present.
- [x] `pleiades-core::ChartSnapshot` now also exposes a major-aspect summary helper, and chart rendering now surfaces an `Aspect summary:` line ahead of the aspect list when major aspect matches are present.
- [x] `pleiades-core::ChartSnapshot` now offers direct body lookup, sign lookup, house lookup, sign-scoped and house-scoped placement iteration, motion-summary and retrograde-summary helpers, and aspect-aware angular separation / major-aspect matching helpers, and chart rendering emits aspect, sign, motion, and retrograde summaries when motion data is available.
- [x] `pleiades-cli` now exposes the implemented backend capability matrices directly via `backend-matrix` / `capability-matrix`, so maintainers can inspect coverage, range, and accuracy notes from the user-facing CLI.
- [x] The compatibility profile now renders an explicit alias-mapping appendix for built-in house systems and ayanamsas, making interoperability lookups easier to audit from the release artifact itself.
- [x] The compatibility profile now also captures a couple of Swiss-Ephemeris label variants that were still missing from the release notes layer: `Whole Sign (house 1 = Aries)` resolves to `Equal (1=Aries)`, and `Moon` resolves to `Valens Moon`.
- [x] The Babylonian house-family ayanamsa batch is now catalogued too: Babylonian (House), Babylonian (Sissy), Babylonian (True Geoc), Babylonian (True Topc), Babylonian (True Obs), and Babylonian (House Obs) are resolved, rendered in the compatibility profile, and available through the shared catalog path as the next breadth increment.
- [x] The compatibility profile now calls out the Babylonian house-family entries explicitly as the remaining open ayanamsa metadata gap in the release line, so release consumers can distinguish source-backed zero-point modes from catalogued-but-unresolved custom definitions at a glance.
- [x] The backend capability matrices now render each backend's nominal supported time range in addition to accuracy and body coverage, making the release-facing capability docs easier to audit.
- [x] The backend capability matrices now also call out expected error classes and required external data files, so the release-facing capability docs explicitly surface each backend's failure modes and data dependencies.
- [x] Shared `CelestialBody` display formatting now preserves custom body identifiers across chart, validation, and artifact reports, so future custom-body or asteroid-oriented extensions have a stable user-facing rendering path.
- [x] The compatibility profile now calls out non-standard ayanamsa labels like True Balarama, Aphoric, and Takra as custom-definition territory, and the ayanamsa crate has a regression test covering custom sidereal offsets for project-specific labels.
- [x] The equal-house interoperability notes now recognize `Wang` as an alias for `Equal`, and the profile/test surface now renders that mapping explicitly so downstream equal-house consumers can round-trip the label without ambiguity.
- [x] The equal-house interoperability aliases now also capture the exact Swiss Ephemeris spellings for `Equal (cusp 1 = Asc)`, `Equal/1=0 Aries`, `Equal (cusp 1 = 0° Aries)`, `Equal (MC)`, and `Equal/MC = 10th`, keeping the release artifact aligned with the source labels people actually see in upstream house-system tables.
- [x] The compatibility profile now includes an ayanamsa sidereal-metadata coverage summary, so release consumers can see which built-ins still lack epoch/offset metadata for chart-layer sidereal conversion at a glance.
- [x] Additional historical sidereal metadata has been filled in for Hipparchus, JN Bhasin, Babylonian (Eta Piscium), Babylonian (Aldebaran), and the Galactic Equator entries, reducing the remaining release-line ayanamsa metadata gaps called out by the profile.
- [x] Babylonian (Kugler 1) and Babylonian (Kugler 2) now carry explicit zero-point metadata too, extending the historical sidereal metadata batch and shrinking the compatibility-profile gap list a little further.
- [x] Babylonian (Kugler 3), Babylonian (Britton), Galactic Center (Mardyks), and Galactic Center (Cochrane) now also carry explicit Swiss Ephemeris zero-point metadata, and the compatibility profile / release notes have been updated to call out the narrower remaining historical-metadata gap.
- [x] Krishnamurti (VP291) now carries explicit zero-point metadata too, closing one of the remaining release-note ayanamsa gaps and keeping the compatibility profile aligned with the catalog.
- [x] Galactic Equator (Mula) now carries its explicit Swiss Ephemeris mid-Mula zero-point metadata too, and the compatibility profile now reflects that release-note correction.
- [x] The next historical/reference-frame metadata batch is now filled in as well: Galactic Center, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Aryabhata (522 CE), and Dhruva Galactic Center (Middle Mula) now carry explicit zero-point metadata, so the compatibility profile no longer treats those entries as sidereal-metadata gaps.
- [x] The ayanamsa interoperability map now also includes exact Swiss Ephemeris source-label aliases for the Babylonian/Kugler family, the galactic-reference entries, the mean-sun variants, and related Stage 6 breadth additions, so the release profile and CLI can accept the source names users actually see in Swiss Ephemeris docs.
- [x] The compatibility profile now renders a dedicated source-label appendix for those exact ayanamsa labels, keeping the source-form aliases visible separately from the broader alias map in release output.
- [x] The source-label appendix now also surfaces the PVR Pushya-paksha / True Pushya, J. N. Bhasin, Lahiri (VP285), Krishnamurti (VP291), Sheoran, remaining mean-sun / galactic-reference source forms, and the newer J2000/J1900/B1950, True Citra/Revati/Mula, Udayagiri, Lahiri (ICRC)/(1940), DeLuce, Yukteshwar, and Moon sign ayanamsa source spellings that users see in Swiss Ephemeris-style interoperability tables, keeping the release profile searchable by the labels that appear in upstream docs.
- [x] The README now calls out the broadened source-label appendix explicitly, so the release-hardening summary stays aligned with the compatibility profile text and the latest source-form spellings.
- [x] The compatibility profile identifier has now been bumped to `0.6.10` to capture the source-label alias interoperability batch while keeping the release profile versioned and explicit.
- [x] The Babylonian house-family source labels (`BABYL_HOUSE`, `BABYL_SISSY`, `BABYL_TRUE_GEOC`, `BABYL_TRUE_TOPC`, `BABYL_TRUE_OBS`, and `BABYL_HOUSE_OBS`) now resolve as exact aliases too, and the compatibility profile identifier has been bumped to `0.6.12` to keep the release artifact aligned with the new interoperability batch and the additional True Citra ayanamsa breadth entry.
- [x] The compatibility profile now renders the Babylonian house-family labels in a dedicated custom-definition section instead of mixing them into the unresolved-gap list, so the release artifact separates published gaps from interoperability-only labels more clearly.
- [x] The custom-definition section now renders each Babylonian house-family label with its documented aliases and source notes, making the interoperability story visible directly in the release artifact.
- [x] APC now resolves the `Ascendant Parallel Circle` interoperability label, and Horizon/Azimuth now resolves the `Horizontal`, `Azimuthal`, and exact `horizon/azimuth` label variants, with the compatibility profile identifier bumped to `0.6.14` to keep the release artifact versioned with the alias batch.
- [x] The compatibility profile source-label appendix now also includes the final exact `Galact. Center = 0 Sag`, `Gal. Eq.`, and `Vettius Valens` source spellings, and the compatibility profile identifier has been bumped to `0.6.15` so the release artifact stays versioned with that appendix refinement.
- [x] The ayanamsa metadata coverage summary now separates intentional Babylonian house-family custom definitions from unexpected sidereal-metadata gaps, so the release profile no longer reports those six labels as unresolved missing-metadata entries.
- [x] The compatibility profile source-label appendix now also includes the Sassanian / `Zij al-Shah` source spelling, keeping the release artifact searchable by that legacy table-reform label alongside the other Swiss Ephemeris ayanamsa spellings.
- [x] The compatibility profile identifier has now been bumped to `0.6.16` to reflect the Sassanian / `Zij al-Shah` appendix refinement and keep the release artifact versioned with the latest source-label batch.
- [x] The source-label appendix for `Valens Moon` now also includes the `Moon sign ayanamsa` source spelling, and the compatibility profile identifier has been bumped to `0.6.17` so the release artifact stays synchronized with the latest appendix refinement.
- [x] The compatibility profile now renders a dedicated source-label appendix for the built-in house systems, so common Placidus/Koch/Equal/Whole Sign/Topocentric spellings are searchable in the release profile alongside the existing ayanamsa appendix, and the compatibility profile identifier has been bumped to `0.6.18` to keep the release artifact versioned with the appendix refinement.
- [x] The compatibility profile now includes the Swiss Ephemeris `Equal (cusp 1 = Asc)` house-system spelling in the built-in house source-label appendix, so equal-house interoperability searches can hit the exact upstream label as well as the canonical name, and the compatibility profile identifier has been bumped to `0.6.19` to keep the release artifact versioned with the latest appendix refinement.
- [x] The compatibility profile's source-label appendix now also includes the `Equal/1=Aries` and `Equal Aries` spellings for `Equal (1=Aries)` alongside the existing `Equal (MC)` equal-house source labels, and the compatibility profile identifier has been bumped to `0.6.22` to keep the release artifact versioned with the latest appendix refinement.
- [x] `ChartSnapshot`'s unknown-motion helper surface is now called out explicitly in the API stability posture and README, so the release-facing chart-helper summary uses the same motion-class wording as the code.
- [x] The API stability posture summary now also names the sign-summary, house-summary, motion-summary, and motion-direction filter helpers explicitly, keeping the release-facing chart-helper wording synchronized with the stable `ChartSnapshot` surface.

- [x] The house-system interoperability labels now also include `WvA`, `Bob Makransky`, and `Vehlow-equal`, and the compatibility profile/source-label appendix rendering now exposes those spellings alongside the existing APC, Sunshine, and Vehlow entries.
- [x] The `True Citra` ayanamsa now also accepts the `True Citra Paksha` and `True Chitrapaksha` interoperability spellings, and the compatibility profile identifier has been bumped to `0.6.27` so the release profile stays synchronized with that alias batch.
- [x] The compatibility and API-stability profile identifiers are now centralized and publicly re-exported from `pleiades-core`, and the CLI / validation tests derive their expectations from those shared profile values instead of duplicating the release strings.

## Exit criteria

- release compatibility profile is published and current
- target compatibility catalog is fully implemented or remaining gaps are explicitly scheduled and justified
- release gates from `spec/validation-and-testing.md` are automated where practical
- maintainers can reproduce tools, builds, tests, validation, and artifacts from repo docs

## Risks to avoid

- adding catalog breadth without maintaining the compatibility profile
- expanding optional helpers in ways that blur crate boundaries
- declaring stability without validation, documentation, and release discipline
