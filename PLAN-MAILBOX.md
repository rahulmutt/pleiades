# PLAN-MAILBOX

## 2026-04-23 — Motion-direction filter helper now appears in the API stability posture

Implemented a small Stage 6 release-hardening sync slice:

- `pleiades-core::ApiStabilityProfile` now names the generic `placements_with_motion_direction` helper explicitly, so the stable chart-surface wording matches the public `ChartSnapshot` API a little more closely
- the README now mentions the same motion-direction filter helper alongside the existing direct/stationary/unknown-motion/retrograde helpers, and the Stage 6 progress notes were updated to keep the release-hardening tracker synchronized
- regression coverage now checks that the API stability posture still mentions the motion-direction filter helper by name, keeping the release-facing stability text aligned with the code surface

Remaining Stage 6 work: keep the API posture, chart helpers, catalog breadth, and compatibility profile synchronized as release-hardening polish continues.

## 2026-04-23 — APC source-label appendix refined

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now includes `APC houses` in the built-in APC source-label appendix, matching the interoperability alias already recognized by `pleiades-houses`
- the compatibility profile identifier was bumped to `0.6.23`, and the CLI / validation bundle checks were updated to match the new release-profile version string
- regression coverage now checks the updated APC source-label rendering, and the Stage 6 progress notes were updated to record the appendix refinement

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Raman source-label appendix refined

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now includes the `B. V. Raman`, `B.V. Raman`, and `B V Raman` source spellings in the built-in ayanamsa source-label appendix, so the baseline Raman ayanamsa is searchable by the common author-name forms
- the compatibility profile identifier was bumped to `0.6.21`, and the CLI / validation bundle checks were updated to match the new release-profile version string
- regression coverage now checks the updated source-label rendering, and the README / Stage 6 progress notes were updated to record the appendix refinement

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Equal-house source-label appendix refined

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now includes the Swiss Ephemeris `Equal (cusp 1 = Asc)` house-system spelling in the built-in house source-label appendix, so equal-house interoperability searches can hit the exact upstream label as well as the canonical name
- the compatibility profile identifier was bumped to `0.6.19`, and the CLI / validation bundle checks were updated to match the new release-profile version string
- regression coverage now checks the updated house source-label rendering, and the Stage 6 progress notes / README were updated to record the appendix refinement

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as additional interoperability labels land.

## 2026-04-23 — House-system source-label appendix added

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now renders a dedicated source-label appendix for the built-in house systems, so common Placidus/Koch/Equal/Whole Sign/Topocentric spellings are searchable in the release profile alongside the existing ayanamsa appendix
- the compatibility profile identifier was bumped to `0.6.18`, and the CLI / validation bundle checks were updated to match the new release-profile version string
- regression coverage now checks the new house-system appendix section, and the Stage 6 progress notes and README were updated to record the appendix refinement

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Valens Moon source-label appendix refined

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now includes `Moon sign ayanamsa` in the `Valens Moon` source-label appendix entry, so the release profile is a little more searchable by the labels people see in upstream source tables
- the compatibility profile identifier was bumped to `0.6.17`, and the CLI / validation bundle checks were updated to match the new release-profile version string
- regression coverage now checks the updated source-label appendix line, and the Stage 6 progress notes were updated to record the appendix refinement

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Aspect-summary helper added to the chart façade

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes a major-aspect summary helper that counts the built-in aspect families in one pass, so chart consumers can summarize major-aspect coverage without rescanning the aspect list manually
- chart rendering now emits an `Aspect summary:` line ahead of the aspect list when major aspect matches are present, keeping the report output aligned with the new helper
- regression coverage now exercises the summary counts and the rendered report text, and the API stability posture / Stage 6 progress notes should mention the new aspect-summary surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — House-summary helper added to the chart façade

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes a `house_summary` helper that counts occupied houses in one pass, so chart consumers can summarize house occupancy without rescanning placements manually
- chart rendering now emits a `House summary:` line ahead of the motion summary when house data is present, keeping the report output aligned with the new helper
- regression coverage now exercises the summary counts and the rendered report text, and the API stability posture / README / Stage 6 progress notes should mention the new house-summary surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Sign-summary helper added to the chart façade

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes a `sign_summary` helper that counts occupied zodiac signs in one pass, so chart consumers can summarize sign occupancy without rescanning placements manually
- chart rendering now emits a `Sign summary:` line ahead of the motion summary when sign data is present, keeping the report output aligned with the new helper
- regression coverage now exercises the summary counts and the rendered report text, and the API stability posture / README / Stage 6 progress notes should mention the new sign-summary surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Stationary motion helper added to the chart façade

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes a `stationary_placements` helper alongside the existing direct and retrograde motion helpers, so consumers can inspect stationary bodies without rescanning placements manually
- chart rendering now emits a `Stationary bodies:` line when stationary motion data is present, keeping the report output aligned with the motion summary counts
- regression coverage now exercises the stationary-motion path end to end, including the rendered report text and the motion-direction partition helper
- the README, API stability posture, and Stage 6 progress notes should mention the new stationary motion helper surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Release profile appendix docs synced

Completed the small documentation-sync follow-up for the latest Stage 6 source-label appendix expansion:

- `README.md` now calls out the broader source-label appendix explicitly, including the latest J2000/J1900/B1950, True Citra, True Revati, True Mula, Udayagiri, Lahiri (ICRC), Lahiri (1940), DeLuce, and Yukteshwar source spellings
- the Stage 6 progress notes were updated with the same appendix wording so the plan tracker stays aligned with the release profile text
- no spec change was needed; this is a documentation alignment slice for the already-implemented compatibility-profile breadth

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Release profile source-label appendix widened again

Implemented another small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now expands the source-label appendix further to cover the latest release-specific ayanamsa forms, including J2000/J1900/B1950, True Citra, True Revati, True Mula, Udayagiri, Lahiri (ICRC), Lahiri (1940), DeLuce, and Yukteshwar source spellings
- regression coverage now checks those additional source-label appendix entries so the compatibility profile stays searchable by the labels that appear in upstream docs and interoperability tables
- the Stage 6 progress notes and README should mention the broader source-label appendix alongside the existing release-hardening notes

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Topocentric interoperability alias broadened

Implemented a small Stage 6 compatibility-alias slice:

- `pleiades-houses` now recognizes `Topocentric house system` as an interoperability alias for `Topocentric`, matching the label variant surfaced by common astrology software and documentation
- the compatibility profile now renders that alias alongside the existing `Polich-Page` / `Polich Page` spellings, and the release notes call out the broadened topocentric interoperability mapping explicitly
- regression coverage now checks the new alias in both the house catalog and the release-profile rendering path so the release artifact stays synchronized with the catalog

Remaining Stage 6 work: keep the house-system alias map, compatibility profile, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — Motion-summary helper now appears in the API stability posture

Implemented a small Stage 6 release-hardening sync slice:

- `pleiades-core::ApiStabilityProfile` now explicitly lists `ChartSnapshot` motion summaries alongside the other stable chart-facade helpers, keeping the consumer-facing stability posture synchronized with the helper surface that was already added to the chart API
- regression coverage now checks that the public stability profile still mentions motion summaries as part of the stable `ChartSnapshot` surface
- the Stage 6 progress notes should call out the API-posture alignment so the release-hardening tracker stays in sync with the chart-helper surface

Remaining Stage 6 work: keep the API posture, chart helpers, and release-facing docs synchronized as additional chart ergonomics land.

## 2026-04-23 — Release profile source-label appendix expanded

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now expands the source-label appendix to cover additional release-specific ayanamsa labels, including the PVR Pushya-paksha / True Pushya family, J. N. Bhasin, Lahiri (VP285), Krishnamurti (VP291), Sheoran, and the mean-sun / galactic-reference label variants already catalogued in `pleiades-ayanamsa`
- regression coverage should assert the new source-label appendix entries so the release profile stays searchable by the labels users actually see in source docs
- the Stage 6 progress notes and README should mention the broader source-label appendix as part of the ongoing release-hardening work

Remaining Stage 6 work: keep the release-profile appendix, catalog breadth, and release notes synchronized as additional interoperability labels land.

## 2026-04-23 — ChartSnapshot motion summary helper added

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes a `motion_summary` helper that counts direct, stationary, retrograde, and unknown placements in one pass
- chart rendering now emits a concise motion summary before the retrograde-body list when motion data is available, so user-facing chart output can report motion mix without rescanning placements manually
- regression coverage now asserts the summary counts and the rendered motion-summary line alongside the existing retrograde/direct helper checks
- the README and Stage 6 progress notes should be updated to mention the new motion-summary helper surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Release bundle now ships release notes

Implemented a small Stage 6 release-hardening slice:

- `pleiades-validate bundle-release --out DIR` now writes a derived `release-notes.txt` artifact alongside the compatibility profile, backend matrix, API posture, validation report, and manifest
- the release notes file summarizes the current release-specific coverage, custom-definition labels, and known gaps, and the bundle manifest now records deterministic checksums for that file too
- the release bundle verification path checks the new file and checksum, and the release-bundle tests now exercise both the happy path and a checksum-corruption regression for the new artifact
- the release reproducibility docs, release-artifact checklist, README, and Stage 6 progress notes were updated to mention the new bundle artifact explicitly

Remaining Stage 6 work: keep the release bundle contents synchronized with any future compatibility-profile or validation-report changes.

## 2026-04-23 — True Citra ayanamsa breadth added

Implemented a small Stage 6 release-hardening slice:

- `pleiades-types` now includes a dedicated `TrueCitra` ayanamsa variant so the shared API can distinguish the published True Citra / Chitra-based interoperability label from the existing True Chitra baseline entry
- `pleiades-ayanamsa` now catalogs `True Citra` with the published zero point from the Swiss-Ephemeris compatibility tables, and the shared resolution path accepts the `True Citra` label explicitly
- `pleiades-core`, the compatibility profile, the CLI profile rendering, and the validation profile checks were updated to surface the new release-specific breadth entry, and the profile identifier was bumped to `0.6.12`
- regression coverage now exercises both the descriptor metadata and the label-resolution path for the new ayanamsa entry

Remaining Stage 6 work: keep the compatibility profile synchronized as any further catalog breadth or metadata backfills land.

## 2026-04-23 — House validation corpus added to the validation report

Implemented a small Stage 4 validation slice:

- `pleiades-validate` now renders a compact house-validation corpus inside the main validation report, covering the baseline house systems on both a mid-latitude reference chart and a polar stress chart
- the report shows per-system success/failure status plus representative ascendant/MC and cusp summaries so house-formula regressions stay visible alongside backend comparison output
- regression coverage now asserts the new house-validation section appears in the validation report output

Remaining Stage 4 work: keep broadening source-backed coverage or house-validation fixtures when more public reference data becomes available.


## 2026-04-23 — Compatibility profile now includes a compact coverage summary

Implemented a small Stage 6 release-hardening refinement:

- `pleiades-core::CompatibilityProfile` now renders a compact coverage summary that reports house-system breadth, ayanamsa breadth, and sidereal-metadata coverage in one place before the detailed custom-definition and alias sections
- the CLI compatibility-profile output and the corresponding tests were updated to match the new wording, so maintainers now see the coverage summary consistently across the core and user-facing surfaces
- Stage 6 progress notes should mention the coverage-summary refinement as part of the ongoing release-hardening work

Remaining Stage 6 work: keep the compatibility profile synchronized as catalog breadth, metadata backfills, or release notes continue to evolve.

## 2026-04-23 — Babylonian house-family labels now render with custom-definition detail

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::CompatibilityProfile` now renders the Babylonian house-family labels in a dedicated custom-definition section instead of mixing them into the unresolved-gap list, so the release artifact separates published gaps from interoperability-only labels more clearly
- the custom-definition section now includes each label's documented aliases and source notes, making the interoperability story visible in the same place as the custom-definition grouping
- the release compatibility profile summary and release notes were updated to describe that split, while the remaining true gap list now focuses on source-backed metadata work that still needs to be scheduled
- regression coverage now asserts the custom-definition section renders with the Babylonian alias details and that the Babylonian house-family labels no longer appear in the explicit-gap list

Remaining Stage 6 work: keep the compatibility profile synchronized as additional breadth batches or metadata backfills land.

## 2026-04-23 — True Sheoran and Galactic Center historical metadata backfill added

Implemented another small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit zero-point metadata for `Ayanamsa::TrueSheoran`, `Ayanamsa::GalacticCenterRgilbrand`, and `Ayanamsa::GalacticCenterMulaWilhelm`, using the published Swiss Ephemeris root dates so the compatibility profile no longer treats those historical/reference-frame entries as gaps
- `pleiades-core` and the release compatibility profile now reflect the narrower ayanamsa-metadata gap in the release notes and known-gap text, and the remaining historical/reference-frame metadata notes now point at future breadth work rather than those three entries
- regression coverage now asserts the new metadata-backed entries remain on the chart-layer sidereal path

Remaining Stage 6 work: keep backfilling any remaining ayanamsa metadata gaps and catalog breadth while the release profile stays synchronized with the catalog.

## 2026-04-22 — Udayagiri and Lahiri VP285 ayanamsa metadata backfill added

Implemented another small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit zero-point metadata for `Ayanamsa::Udayagiri` and `Ayanamsa::LahiriVP285`, reusing the published Lahiri-family 285 CE reference point so the compatibility profile no longer lists those variants as gaps
- `pleiades-core` and the release compatibility profile now reflect the narrower ayanamsa-metadata gap in the release notes and known-gap text, and `Krishnamurti (VP291)` remains scheduled for later metadata/source work
- regression coverage now asserts the new metadata-backed entries remain on the chart-layer sidereal path

Remaining Stage 6 work: keep backfilling the remaining ayanamsa metadata gaps and catalog breadth while the release profile stays synchronized with the catalog.

## 2026-04-22 — Direct-placement chart helper added

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes `placements_with_motion_direction` plus a convenience `direct_placements` helper so callers can partition chart bodies by motion state without rescanning the placements vector manually
- the API stability posture, README status notes, and Stage 6 progress notes were updated to mention the new direct/retrograde helper surface explicitly
- regression coverage now exercises both the retrograde and direct motion paths in the chart façade tests

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-22 — Additional ayanamsa metadata backfill added

Implemented another small Stage 6 release-hardening slice:

- `pleiades-ayanamsa` now carries explicit zero-point epoch/offset metadata for `Ayanamsa::BabylonianKugler3`, `Ayanamsa::BabylonianBritton`, and `Ayanamsa::GalacticCenterMardyks`, matching the Swiss Ephemeris reference values now reflected in the catalog
- `pleiades-core` and the compatibility profile now reflect the narrower ayanamsa-metadata gap in the release notes and known-gap text, and the profile summary stays synchronized with the updated catalog breadth
- the new descriptor coverage is regression-tested alongside the existing historical/reference-frame metadata checks, so the release profile can keep treating those three entries as metadata-backed rather than unresolved

Remaining Stage 6 work: keep filling out any remaining ayanamsa metadata and catalog breadth while the release profile stays synchronized with the catalog.

## 2026-04-22 — Ayanamsa metadata coverage summary added

Implemented a small Stage 6 release-hardening slice:

- `pleiades-ayanamsa` now exposes a metadata-coverage summary for built-in ayanamsas, counting which entries carry both reference epoch and offset metadata and listing the remaining gaps
- `pleiades-core` and `pleiades-cli` now render that coverage summary in the compatibility profile so release consumers can see the sidereal-metadata gap at a glance
- the new coverage summary is regression-tested in both the ayanamsa crate and the profile rendering path

Remaining Stage 6 work: continue filling out ayanamsa metadata and catalog breadth while keeping the release profile synchronized with the catalog.

## 2026-04-22 — Cochrane ayanamsa metadata backfill added

Implemented a small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit zero-point epoch/offset metadata for `Ayanamsa::GalacticCenterCochrane`, based on the Swiss Ephemeris Cochrane zero epoch at JD 1662951.794251
- `pleiades-core` now reflects that narrower ayanamsa-metadata gap in the compatibility profile, and the Stage 6 progress notes should mention Galactic Center (Cochrane) alongside the other metadata-backed exceptions
- the new descriptor coverage is regression-tested alongside the existing Huber, Galactic Equator, True Pushya, Djwhal Khul, Sheoran, and Valens Moon metadata checks

Remaining Stage 6 work: keep filling out any remaining ayanamsa metadata and catalog breadth while the release profile stays synchronized with the catalog.


## 2026-04-22 — Wang alias recognized as equal-house interoperability coverage

Implemented a small Stage 6 compatibility-alias slice:

- `pleiades-houses` now treats `Wang` as an alias for `HouseSystem::Equal`, matching the equal-house-from-Ascendant convention used by downstream interoperability labels
- `pleiades-core` release notes and alias-rendering tests now surface that mapping in the compatibility profile so the release artifact shows the new equivalence explicitly
- no spec update was required; this is a catalog/interop refinement that stays within the existing Stage 6 compatibility-expansion scope

Remaining Stage 6 work: keep filling out any remaining catalog breadth and metadata while the release profile stays synchronized with the catalog.

## 2026-04-22 — aspect helper slice added to ChartSnapshot

Implemented a small Stage 6 optional-helper slice:

- `pleiades-core::ChartSnapshot` now exposes angular separation and built-in major-aspect matching helpers, plus chart rendering now includes a simple aspects section when ecliptic positions are available
- the API stability posture and README should be updated to mention the new aspect-oriented helper surface once the change is recorded
- tests now cover a sextile example end to end through the core façade and CLI rendering path

Remaining Stage 6 work: broader catalog breadth and any remaining optional helper polish that depends on additional coverage.


## 2026-04-22 — sign-scoped chart helper added

Implemented a small Stage 6 optional-helper slice:

- `pleiades-core::ChartSnapshot` now exposes `sign_for_body` and `placements_in_sign` alongside the existing body, house, motion, and retrograde lookup helpers, so downstream chart consumers can ask sign-scoped questions without re-scanning placements manually
- API stability wording, the README, and the Stage 6 progress notes should be updated to reflect the new sign-scoped helper surface once the change is recorded

Remaining Stage 6 work: broader catalog breadth and any remaining optional helper polish that depends on additional coverage.


## 2026-04-22 — Sheoran ayanamsa metadata backfill added

Implemented another small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit zero-point epoch/offset metadata for `Ayanamsa::Sheoran`, so the chart-layer sidereal helper can derive offsets for that published reference mode instead of using a placeholder J2000 anchor
- `pleiades-core` now reflects the narrower ayanamsa-metadata gap in the compatibility profile, and the Stage 6 progress notes should mention Sheoran alongside the other metadata-backed exceptions
- the new descriptor coverage is regression-tested alongside the existing Huber, Galactic Equator, True Pushya, Djwhal Khul, and Valens Moon metadata checks

Remaining Stage 6 work: continue filling out any remaining ayanamsa metadata and catalog breadth while keeping the release profile synchronized with the catalog.


## 2026-04-22 — True Pushya and Djwhal Khul metadata backfill added

Implemented a small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit zero-point epoch/offset metadata for `Ayanamsa::TruePushya` and `Ayanamsa::DjwhalKhul`, so the chart-layer sidereal helper can derive offsets for those published reference modes instead of treating them as unresolved catalog entries
- `pleiades-core` now reflects the narrower ayanamsa-metadata gap in the compatibility profile, and the Stage 6 progress notes should mention the new metadata-backed exceptions instead of the earlier Udayagiri placeholder
- the new descriptor coverage is regression-tested alongside the existing Huber, Galactic Equator, and Valens Moon metadata checks

Remaining Stage 6 work: continue filling out any remaining ayanamsa metadata and catalog breadth while keeping the release profile synchronized with the catalog.


## 2026-04-22 — house-scoped placement helper added to ChartSnapshot

Implemented a small Stage 6 optional-helper slice:

- `pleiades-core::ChartSnapshot` now exposes `placements_in_house` alongside the existing body lookup, house lookup, and motion-direction helpers so chart consumers can ask which bodies fall into a house without re-scanning placements manually
- API stability wording and README status notes now mention the new house-scoped placement helper, and the regression test exercises it alongside the retrograde summary

Remaining Stage 6 work: broader catalog breadth and any remaining optional helper polish that depends on additional coverage.

## 2026-04-22 — non-standard ayanamsa labels now treated as custom-definition territory

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core` now calls out non-standard ayanamsa labels such as True Balarama, Aphoric, and Takra in the compatibility profile as custom-definition territory instead of implying that they are built-in Swiss Ephemeris selections
- `pleiades-ayanamsa` now has a regression test that exercises the `Ayanamsa::Custom` sidereal-offset path with a project-specific label, so maintainers have a concrete example for custom compatibility mappings
- the Stage 6 plan progress notes should be updated to reflect the new custom-label guidance slice once the change is recorded

Remaining Stage 6 work: keep catalog breadth, release notes, and custom-label interoperability guidance synchronized as additional compatibility entries land.

## 2026-04-22 — body display formatting now preserves custom body identifiers

Implemented a small Stage 6 ergonomics slice:

- `pleiades-types` now gives `CelestialBody` a `Display` implementation that renders custom body identifiers via the shared `CustomBodyId` format instead of collapsing them to a generic label
- chart and validation/report formatting now use that shared display path, so custom bodies show up consistently in user-facing output and future custom-body backends have a stable rendering path
- the Stage 6 plan notes should reflect the new ergonomics slice once the progress update is recorded

Remaining Stage 6 work: keep the compatibility profile and release-facing ergonomics aligned as additional catalog breadth or custom-body support lands.

## 2026-04-22 — artifact validation now reports body-class error envelopes

Implemented a small Stage 5 reporting slice:

- `pleiades-validate validate-artifact` now groups the packaged-artifact comparison deltas into body-class envelopes so the report shows separate error summaries for luminaries, major planets, lunar points, asteroids, and custom bodies
- the artifact-validation test now checks for the new body-class section in addition to the existing boundary and checksum coverage
- the Stage 5 plan note was updated to reflect the more explicit error-envelope reporting

Remaining Stage 5 work: keep the packaged artifact coverage and validation story aligned if the artifact body set grows again.

## 2026-04-22 — Babylonian house-family ayanamsa breadth added

Implemented another Stage 6 catalog-breadth slice:

- `pleiades-types` now includes the Babylonian house-family ayanamsa variants for House, Sissy, True Geoc, True Topc, True Obs, and House Obs
- `pleiades-ayanamsa` now catalogs, resolves, and documents those modes with explicit Swiss Ephemeris-style identifiers and aliases
- `pleiades-core` and the release compatibility profile now surface the new breadth batch in the release summary and release-note text so the catalog stays synchronized with the profile

Remaining Stage 6 work: keep filling out any still-scheduled ayanamsa breadth while the release profile stays synchronized with the catalog.

## 2026-04-22 — Dhruva Galactic Center (Middle Mula) ayanamsa added

Implemented another Stage 6 catalog-breadth slice:

- `pleiades-types` now includes a dedicated `DhruvaGalacticCenterMula` ayanamsa variant for the middle-of-Mula galactic-center selection
- `pleiades-ayanamsa` now catalogs, resolves, and documents Dhruva Galactic Center (Middle Mula) with explicit aliases for the Swiss Ephemeris / Wilhelm naming family
- `pleiades-core`, the compatibility profile, the README, and the Stage 6 progress notes now surface the new breadth entry so release notes stay synchronized with the catalog

Remaining Stage 6 work: keep catalog breadth and the release profile synchronized as additional Swiss Ephemeris ayanamsa modes are scheduled.

## 2026-04-22 — backend capability matrices now include nominal ranges

Implemented a small Stage 6 release-hardening slice:

- `pleiades-validate` and the CLI backend-matrix output now render each backend's nominal supported time range alongside the existing accuracy, coverage, and capability notes
- this makes the public capability documentation more explicit for the JPL snapshot, algorithmic backends, and packaged-data backend without changing the underlying routing or query behavior
- the stage-6 plan notes now call out the range rendering so the release-hardening backlog stays synchronized with the current docs surface

Remaining Stage 6 work: keep the capability docs aligned with future backend additions or move on to any remaining release-hardening polish that depends on broader coverage.

## 2026-04-22

Implemented the first Stage 3 slice:

- baseline house-system catalog metadata now lives in `pleiades-houses`
- baseline ayanamsa catalog metadata now lives in `pleiades-ayanamsa`
- `pleiades-core` now publishes a versioned compatibility profile with known gaps
- `pleiades-cli` can print the compatibility profile for quick inspection

Next recommended slice: start the actual algorithmic chart workflow by wiring in a minimal Sun/Moon backend path, then layer tropical-to-sidereal and chart assembly helpers on top.

## 2026-04-22 — tropical chart MVP landed

Implemented the next Stage 3 slice:

- `pleiades-vsop87` now computes approximate tropical positions for the Sun and major planets with a pure-Rust orbital-elements model
- `pleiades-elp` now computes an approximate tropical Moon position with a pure-Rust analytical model
- `pleiades-backend` gained a simple composite router for Moon-plus-planets workflows
- `pleiades-core` can assemble a basic tropical chart snapshot with zodiac sign placements
- `pleiades-cli chart` renders the new chart report using the composite backend

Remaining Stage 3 work: sidereal conversion, fuller house placement, and any missing chart ergonomics needed to make the workflow feel production-ready.

## 2026-04-22 — sidereal chart conversion added

Implemented the next Stage 3 slice:

- `pleiades-ayanamsa` now carries baseline epoch/offset metadata for built-in sidereal catalog entries and exposes a deterministic offset helper for custom or built-in definitions
- `pleiades-core` now exposes `sidereal_longitude` and uses it inside chart assembly when a sidereal zodiac mode is requested
- `pleiades-cli chart` accepts `--ayanamsa <name>` and can render sidereal chart output on top of the tropical backends
- compatibility-profile output was updated to describe the current sidereal chart capability and the remaining house-placement gap

Remaining Stage 3 work: house placement for the baseline catalog, plus any chart ergonomics needed to polish the workflow.

## 2026-04-22 — baseline house placement started

Implemented the next Stage 3 slice:

- `pleiades-houses` now exposes a first-pass calculation API for Equal, Whole Sign, and Porphyry houses, with explicit unsupported errors for the remaining baseline systems
- `pleiades-core` can request house placement during chart assembly, surface the resulting cusps, and assign bodies to houses
- `pleiades-cli chart` accepts `--house-system <name>` and can print house cusps alongside the body report
- the compatibility profile and README now distinguish the implemented house-placement subset from the remaining quadrant-style systems

Remaining Stage 3 work: the more complex baseline house families (Placidus, Koch, Regiomontanus, Campanus, Alcabitius, Topocentric, Morinus, Meridian, and Axial variants) still need dedicated implementations.

## 2026-04-22 — baseline quadrant-house implementations completed

Implemented the next Stage 3 slice:

- `pleiades-houses` now implements the full baseline house catalog, including Placidus, Koch, Regiomontanus, Campanus, Alcabitius, Topocentric, Morinus, Meridian, and Axial variants
- the compatibility profile now reports those systems as implemented rather than pending
- Stage 3 progress notes and Stage 2 handoff text were updated to reflect the expanded baseline coverage

Stage 3 is now effectively complete at the baseline level; Stage 4 validation and later-stage hardening remain the next major follow-up.

## 2026-04-22 — Stage 4 validation slice landed

Implemented the first Stage 4 slice:

- `pleiades-jpl` now ships a narrow JPL Horizons reference snapshot backend keyed to the J2000.0 corpus, with checked-in source data and provenance metadata
- `pleiades-validate` now compares the JPL snapshot backend against the algorithmic composite backend, benchmarks the corpus, and renders reproducible report output
- validation reports include backend capability matrices, corpus metadata, and per-body delta summaries so later artifacts can stay comparable

Next recommended slice: broaden the validation corpus/time coverage, add archived report outputs, and preserve any discovered regressions in the test corpus.

## 2026-04-22 — benchmark corpus now spans the target window

Implemented the next Stage 4 slice:

- `pleiades-validate` now distinguishes the single-epoch JPL comparison corpus from a three-epoch representative benchmark corpus spanning 1500-2500 CE
- validation reports now print explicit corpus summaries so maintainers can see the comparison and benchmark time coverage at a glance
- benchmark command output now uses the representative window corpus, while the comparison report remains locked to the source-backed JPL snapshot

Remaining Stage 4 work: broaden time-range comparison coverage, add archived validation outputs, and capture any additional regression cases in the corpus.

## 2026-04-22 — archived regression cases preserved

Implemented the next Stage 4 slice:

- `pleiades-validate` now preserves notable regression findings as an explicit archived regression case set in the rendered validation report
- validation reports now distinguish the live comparison summary from the archived regression case list so previously observed deltas remain visible in the test corpus
- regression archive coverage is exercised by tests for both the comparison report and the full validation report

Remaining Stage 4 work: broaden time-range comparison coverage and add asteroid support.

## 2026-04-22 — multi-epoch comparison coverage added

Implemented the next Stage 4 slice:

- `pleiades-jpl` now loads a checked-in multi-epoch Horizons snapshot rather than a single-epoch corpus, which lets the validation layer compare several bodies across a broader date span
- `pleiades-validate` now builds its comparison corpus from the snapshot rows, so the validation report exercises multiple epochs instead of only J2000.0
- the Stage 4 plan now reflects that the broader comparison coverage is implemented, while selected asteroid support remains the next open slice

Remaining Stage 4 work: selected asteroid support.

## 2026-04-22 — artifact inspection tooling added

Implemented the next Stage 5 slice:

- `pleiades-validate` now exposes `validate-artifact`, which inspects the bundled compressed artifact, verifies encode/decode and checksum behavior, and reports body- and boundary-level coverage
- the validation report now calls out the packaged artifact’s segment continuity checks so edge behavior is visible in a dedicated command
- the stage-5 plan now reflects that artifact-inspection tooling is in place, leaving measured error envelopes and broader body coverage as the remaining follow-up

Remaining Stage 5 work: measured artifact error envelopes and broader body coverage.

## 2026-04-22 — stage 6 release profile slice landed

Implemented the first Stage 6 release-hardening slice:

- `pleiades-core` now renders a release-grade compatibility profile that explicitly separates target scope, the baseline milestone, release-specific coverage notes, and remaining gaps
- `pleiades-validate report` now includes that compatibility profile so the validation bundle carries the release-coverage summary by default
- CLI help text and plan notes were updated to describe the profile as a release artifact instead of only a stage-3 baseline note

Next recommended slice: keep the compatibility profile current as catalog breadth expands, then move on to the remaining Stage 6 release-hardening work (automation, API posture, and broader catalog coverage).

## 2026-04-22 — packaged artifact coverage broadened

Implemented the next Stage 5 slice:

- `pleiades-data` now generates its bundled artifact from the checked-in JPL reference snapshot instead of hardcoded Sun/Moon constants
- the packaged artifact now covers the full comparison-body planetary set (`Sun` through `Pluto`) with two interpolated segments for the inner bodies and point segments for the outer bodies at J2000
- `validate-artifact` now reports the broader body coverage automatically, and the codec roundtrip tests now assert the expanded packaged body count

Stage 5 now appears complete; the remaining planned work is the Stage 6 release-hardening backlog (automation, broader catalog breadth, and public API posture).

## 2026-04-22 — release bundle smoke test automated

Implemented the next Stage 6 slice:

- `mise.toml` now exposes a `release-smoke` task that runs `pleiades-validate bundle-release` into a temporary directory and verifies the profile, report, and manifest outputs
- the GitHub CI workflow now exercises that release smoke check through `mise run ci`, so release bundle regressions are caught automatically
- `README.md` documents the release smoke check for maintainers who want to exercise the release bundle path locally

Remaining Stage 6 work: catalog breadth expansion and any broader release-hardening polish that depends on additional coverage.

## 2026-04-22 — release catalog breadth expanded

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained new house-system variants for Equal (MC), Vehlow Equal, and Sripati
- `pleiades-houses` now catalogs, resolves, documents, and calculates those three release-specific house systems alongside the baseline milestone
- `pleiades-core` and the release compatibility profile now distinguish the new release-specific house-system coverage from the baseline milestone
- `pleiades-cli` and `pleiades-validate` now report the updated compatibility profile identifier and release-specific coverage sections

Remaining Stage 6 work: broader house-system breadth, API posture review, and any remaining release-hardening polish that depends on additional coverage.

## 2026-04-22 — fixed zodiac-sign house addition

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained `HouseSystem::EqualAries` for the fixed 0° Aries house variant
- `pleiades-houses` now catalogs, resolves, documents, and calculates Equal (1=Aries)
- the compatibility profile, CLI, and validation output now reflect the additional release-specific house-system coverage

Remaining Stage 6 work: broader catalog breadth, API posture review, and any remaining release-hardening polish that depends on additional coverage.

## 2026-04-22 — historical ayanamsa breadth expanded

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained additional ayanamsa variants for Lahiri (ICRC), Lahiri (1940), Usha Shashi, Suryasiddhanta (499 CE), Aryabhata (499 CE), and Sassanian
- `pleiades-ayanamsa` now catalogs, resolves, and exposes those historical anchor-point variants alongside the baseline milestone
- `pleiades-core` now surfaces the expanded ayanamsa catalog in the release compatibility profile, and the CLI/validation paths inherit the broader resolution set automatically

Remaining Stage 6 work: broader house-system breadth, API posture review, and any remaining release-hardening polish that depends on additional coverage.

## 2026-04-22 — Carter house-system breadth added

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained `HouseSystem::Carter` for the poli-equatorial house system
- `pleiades-houses` now catalogs, resolves, documents, and calculates Carter (poli-equatorial)
- `pleiades-core` and the release compatibility profile now expose Carter in the release-specific house-system coverage set

Remaining Stage 6 work: broader house-system breadth, API posture review, and any remaining release-hardening polish that depends on additional coverage.

## 2026-04-22 — Horizon/Azimuth and APC added

Implemented another Stage 6 catalog-breadth slice:

- `pleiades-types` gained `HouseSystem::Horizon` and `HouseSystem::Apc`
- `pleiades-houses` now catalogs, resolves, documents, and calculates Horizon/Azimuth and APC using the shared compatibility profile path
- `pleiades-core` and the release compatibility profile now expose the new release-specific coverage set entries

Remaining Stage 6 work: the rest of the specialized house-system breadth, plus any API posture review and release-hardening polish that depends on additional coverage.

## 2026-04-22 — Krusinski-Pisa-Goelzer house breadth added

Implemented the next Stage 6 breadth slice:

- `pleiades-types` gained `HouseSystem::KrusinskiPisaGoelzer`
- `pleiades-houses` now catalogs, resolves, documents, and calculates Krusinski-Pisa-Goelzer with a dedicated pure-Rust implementation and alias coverage
- `pleiades-core` and the compatibility profile now expose the new release-specific coverage entry, and the profile version was bumped to `0.5.0`
- plan notes were updated so the remaining breadth work now points at the sinusoidal, Albategnius, Sunshine, and Gauquelin-sector families

Next recommended slice: keep adding the remaining specialized house families in small batches, then revisit any remaining API-posture polish if those additions expose new constraints.

## 2026-04-22 — Sunshine house family added

Implemented the next Stage 6 breadth slice:

- `pleiades-types` gained `HouseSystem::Sunshine` for the Sunshine house family
- `pleiades-houses` now catalogs, resolves, documents, and calculates Sunshine using a pure-Rust Sun-declination and arc-segmentation implementation derived from the Swiss Ephemeris formula set
- `pleiades-core`, the compatibility profile, and the README now reflect Sunshine as release-specific coverage

Remaining Stage 6 work: the Gauquelin-sector family, plus any API posture review and release-hardening polish that depends on additional coverage.

## 2026-04-22 — Gauquelin sector family added

Implemented the next Stage 6 breadth slice:

- `pleiades-types` gained `HouseSystem::Gauquelin` for the 36-sector Gauquelin house family
- `pleiades-houses` now catalogs, resolves, documents, and calculates Gauquelin sectors with a pure-Rust 36-sector implementation anchored on the release profile’s house-angle model
- `pleiades-core`, the compatibility profile, and the README now reflect Gauquelin sectors as release-specific coverage

Remaining Stage 6 work: any API posture review and release-hardening polish that depends on additional coverage.

## 2026-04-22 — API stability posture published

Implemented the remaining Stage 6 release-hardening slice that makes the public API posture explicit:

- `pleiades-core` now publishes a versioned API stability profile alongside the compatibility profile
- `pleiades-cli` can print the API stability posture directly with `api-stability` / `api-posture`
- `pleiades-validate` includes the API stability posture in validation reports and exposes the same command-line view for release automation
- the README now states which surfaces are intended to be stable, which remain tooling-internal, and how deprecations will be handled

Stage 6 now has a clear consumer-facing API posture; the next maintenance work is to keep that profile aligned with future release changes.

## 2026-04-22 — release bundle now includes API stability posture

Implemented a release-hardening polish slice:

- `pleiades-validate bundle-release --out DIR` now writes `api-stability.txt` alongside the compatibility profile and validation report
- the bundle manifest now records the API stability posture identifier so release archives can be traced back to both public profiles
- release bundle output and documentation now call out the extra posture artifact explicitly

Next recommended slice: keep the release bundle manifest aligned with future profile changes, or move on to any remaining stage-6 maintenance work that depends on new catalog breadth.

## 2026-04-22 — Swiss Ephemeris ayanamsa breadth expanded

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained `Ayanamsa::J2000`, `Ayanamsa::J1900`, `Ayanamsa::B1950`, `Ayanamsa::TrueRevati`, and `Ayanamsa::TrueMula`
- `pleiades-ayanamsa` now catalogs, resolves, and exposes those Swiss Ephemeris reference-frame and true-nakshatra modes with compatibility metadata
- `pleiades-core`, the compatibility profile, the CLI, and validation output now surface the broadened ayanamsa catalog and version bump

## 2026-04-22 — DeLuce and Yukteshwar added

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` now includes `Ayanamsa::DeLuce` and `Ayanamsa::Yukteshwar`
- `pleiades-ayanamsa` catalogs, resolves, and documents DeLuce and Yukteshwar with aliases for De Luce / Yukteswar / Sri Yukteswar spellings
- `pleiades-core` publishes the new built-ins in the compatibility profile, and the release profile identifier was bumped to `0.6.3`

Remaining Stage 6 work: keep catalog breadth and release notes aligned as additional Swiss Ephemeris ayanamsa modes are scheduled.

## 2026-04-22 — release bundle checksums added

- The release bundle manifest now includes deterministic FNV-1a checksums for the compatibility profile, API stability posture, and validation report, and the CLI release-bundle summary surfaces them for verification.

## 2026-04-22 — Valens Moon ayanamsa metadata added

Implemented a small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries the Swiss Ephemeris reference epoch and offset metadata for `Ayanamsa::ValensMoon`, matching the values published in the upstream header
- `pleiades-core` now reflects that metadata-backed exception in the compatibility profile, so the remaining ayanamsa metadata gap is a little narrower
- tests now verify the Valens Moon descriptor and the sidereal-offset helper path alongside the existing Huber and Galactic Equator reference metadata checks

Remaining Stage 6 work: keep filling out any remaining ayanamsa metadata and catalog breadth while the release profile stays synchronized with the catalog.

## 2026-04-22 — historical/reference-frame ayanamsa batch added

Implemented another Stage 6 catalog-breadth slice:

- `pleiades-types` now includes `Hipparchus`, the Babylonian Kugler 1/2/3, Huber, Eta Piscium, Aldebaran, Galactic Center, and Galactic Equator ayanamsa variants
- `pleiades-ayanamsa` now catalogs, resolves, and documents those historical/reference-frame ayanamsa modes with compatibility aliases
- `pleiades-core`, the compatibility profile, the README, and the Stage 6 progress notes now surface the new breadth batch as release-specific catalog coverage

Remaining Stage 6 work: continue filling out the remaining Swiss Ephemeris ayanamsa breadth and keep the release profile synchronized.

## 2026-04-22 — selected asteroid coverage surfaced in validation reports

Implemented a small Stage 4 visibility slice:

- `pleiades-jpl` now exposes the source-backed asteroid subset from the checked-in Horizons snapshot as a dedicated helper alongside the reference corpus helpers
- `pleiades-validate` now renders a dedicated selected-asteroid coverage section in validation reports, so the JPL snapshot’s Ceres/Pallas/Juno/Vesta support is visible without disturbing the planetary comparison corpus
- tests now assert both the JPL asteroid helper and the validation-report section so the stage-4 asteroid visibility stays covered

Remaining Stage 4 work: if we continue expanding source-backed coverage, the next incremental step should be additional bodies or epochs that are justified by available public data.

## 2026-04-22 — remaining Swiss Ephemeris ayanamsa breadth batch added

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` gained the remaining named legacy ayanamsa variants and formula modes from the Swiss Ephemeris header: True Pushya, Djwhal Khul, JN Bhasin, Suryasiddhanta (Mean Sun), Aryabhata (Mean Sun), Babylonian (Britton), Aryabhata (522 CE), Lahiri (VP285), Krishnamurti (VP291), True Sheoran, and the remaining Galactic Center / Galactic Equator variants
- `pleiades-ayanamsa` now catalogs, resolves, and documents those newly added modes with explicit aliases where the header/naming family suggests them
- `pleiades-core`, the compatibility profile, the CLI, and validation output now surface the expanded ayanamsa breadth and the release profile version was bumped again

Remaining Stage 6 work: keep catalog breadth and release notes aligned as any final Swiss Ephemeris ayanamsa modes are scheduled, plus the rest of the release-hardening backlog.

## 2026-04-22 — catalog alignment invariants added

Implemented a release-hardening maintenance slice:

- `pleiades-houses` and `pleiades-ayanamsa` now have round-trip coverage tests that verify the built-in catalogs, baseline milestones, release-specific additions, and alias resolution stay internally aligned
- `pleiades-core` now checks that the published compatibility profile’s release-note text continues to mention every release-specific house-system and ayanamsa entry, reducing drift between the catalog data and the release summary, and the compatibility-profile identifier was bumped to `0.6.8` to reflect the latest content update
- the next recommended follow-up is to keep using the same invariant pattern whenever new catalog breadth lands, so the compatibility profile stays synchronized automatically

## 2026-04-22 — metadata-backed ayanamsa metadata added

Implemented a small Stage 6 maintenance slice:

- `pleiades-ayanamsa` now carries explicit epoch/offset metadata for Babylonian (Huber) and Galactic Equator (IAU 1958)
- `pleiades-core` now calls out those metadata-backed exceptions in the compatibility profile so the remaining ayanamsa metadata gap is more precise
- tests now verify both descriptors and their sidereal-offset availability, keeping the metadata and runtime helper behavior synchronized

Remaining Stage 6 work: continue filling out any remaining ayanamsa breadth while keeping metadata-backed modes and the release profile synchronized.

## 2026-04-22 — topocentric refinement landed

Implemented the next Stage 6 maintenance slice:

- `pleiades-houses` now converts geodetic observer latitude to geocentric latitude with an elevation-aware ellipsoid model for Topocentric (Polich-Page)
- the Topocentric catalog note now documents the latitude correction so the compatibility profile and crate docs reflect the refined implementation
- a regression test now pins the geocentric correction at 45° latitude and checks that elevation nudges the effective latitude as expected

## 2026-04-22 — motion-direction helper added

Implemented another small Stage 6 optional-helper slice:

- `pleiades-types::Motion` now exposes a sign-based longitude-direction helper and the shared `MotionDirection` enum
- `pleiades-core::BodyPlacement` re-exports that direction classification for chart consumers
- `pleiades-core` chart rendering now has a dedicated motion column when backend motion data is available, so future reports can surface retrograde/direct state without backend-specific logic

Remaining Stage 6 work: broader catalog breadth, additional optional helpers where they add real chart value, and any remaining release-hardening polish that depends on additional coverage.

## 2026-04-22 — chart lookup helpers and retrograde summary added

Implemented a small Stage 6 optional-helper slice:

- `pleiades-core::ChartSnapshot` now exposes direct body lookup and motion-direction helpers, plus a retrograde-placement iterator for downstream chart consumers
- `ChartSnapshot` rendering now emits a retrograde-body summary when motion data is available, making the higher-level chart report easier to scan without re-deriving the same classification elsewhere
- the API stability profile now names the lookup and retrograde helpers as part of the stable chart façade, and the README notes the new ergonomics

Remaining Stage 6 work: broader catalog breadth and any remaining optional helper polish that depends on additional coverage.

## 2026-04-22 — Udayagiri ayanamsa breadth added

Implemented a small Stage 6 catalog-breadth slice:

- `pleiades-types` gained `Ayanamsa::Udayagiri`
- `pleiades-ayanamsa` now catalogs, resolves, and surfaces Udayagiri in the release compatibility profile
- `pleiades-core`, the README, and the Stage 6 plan notes now reflect the new breadth entry

Next recommended slice: continue the remaining Swiss Ephemeris ayanamsa breadth or move to any additional release-hardening polish that depends on broader compatibility coverage.

## 2026-04-22 — SS Revati/Citra breadth identified

Implemented the next Stage 6 catalog-breadth slice:

- `pleiades-types` now has explicit ayanamsa variants for Suryasiddhanta (Revati) and Suryasiddhanta (Citra)
- `pleiades-ayanamsa` and the compatibility profile can now catalog, resolve, and display the Swiss Ephemeris SS Revati and SS Citra breadth entries alongside the existing true-nakshatra modes
- the stage-6 plan should keep this batch grouped with the other ayanamsa-breadth notes so the release profile stays synchronized with the catalog

Remaining Stage 6 work: keep filling out ayanamsa breadth and any release-hardening polish that depends on additional catalog coverage.

## 2026-04-22 — backend capability matrix surfaced in the CLI

Implemented a small Stage 6 release-hardening slice:

- `pleiades-cli` now exposes `backend-matrix` / `capability-matrix` and reuses the validation report renderer to print the implemented backend capability matrices directly from the user-facing CLI
- the CLI help text now advertises the backend matrix command alongside the existing compatibility and API-posture views
- README wording now points out that maintainers can inspect body coverage, time-range notes, and accuracy classes without leaving the repository

Remaining Stage 6 work: keep catalog breadth and release notes aligned as new compatibility entries land, and continue any release-hardening polish that depends on broader coverage or automation.

## 2026-04-22 — Galactic Equator (Fiorenza) ayanamsa metadata backfill added

Implemented a small Stage 6 metadata-backfill slice:

- `pleiades-ayanamsa` now carries explicit J2000.0 epoch/offset metadata for `Ayanamsa::GalacticEquatorFiorenza`, using the published 25° reference value
- `pleiades-core` now reflects that narrower ayanamsa-metadata gap in the compatibility profile, and the Stage 6 progress notes should mention Galactic Equator (Fiorenza) alongside the other metadata-backed exceptions
- the metadata-coverage summary and compatibility-profile rendering should be updated if any downstream expectations still assume the older missing-metadata set

Remaining Stage 6 work: continue filling out any remaining ayanamsa metadata and catalog breadth while keeping the release profile synchronized with the catalog.

- 2026-04-23: Implemented a Stage 6 ayanamsa interoperability slice that aligns several built-in aliases with the exact Swiss Ephemeris source labels (including the Babylonian/Kugler family, the galactic-reference entries, and the mean-sun variants). The compatibility profile and CLI/validation resolution paths now render a dedicated source-label appendix so the release-facing catalog makes those exact labels visible separately from the more general alias map.

- Expanded the house-system interoperability aliases so APC now resolves `Ascendant Parallel Circle` and Horizon/Azimuth now resolves `Horizontal` and `Azimuthal`; the compatibility profile was bumped to `0.6.13` to keep the release artifact versioned with the alias batch.

- Added a release-checklist artifact to `pleiades-validate bundle-release` / `verify-release-bundle`, and updated the release bundle manifest, docs, and Stage 6 notes so the maintained release bundle now carries the maintainer-facing release gate summary alongside the compatibility profile, release notes, capability matrix, API posture, and validation report.

## 2026-04-23 — Unknown-motion chart helper added

Implemented a small Stage 6 chart-ergonomics slice:

- `pleiades-core::ChartSnapshot` now exposes an `unknown_motion_placements` helper so consumers can inspect placements whose longitudinal motion cannot be classified from backend data
- chart rendering now emits an `Unknown motion bodies:` line when those placements are present, keeping the report output aligned with the motion summary counts
- regression coverage now exercises the unknown-motion path end to end, including the rendered report text and the motion-summary counts
- the README, API stability posture, and Stage 6 progress notes should mention the unknown-motion helper surface explicitly

Remaining Stage 6 work: keep the release-facing chart helpers, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Sassanian source-label appendix entry added

Implemented a small Stage 6 source-label refinement:

- `pleiades-core::CompatibilityProfile` now includes the `Zij al-Shah -> Sassanian` source-label mapping in the ayanamsa source-label appendix, so the release profile stays searchable by the legacy Sassanian table-reform label as well as the canonical built-in name
- the compatibility-profile tests and release-facing README summary were updated to reflect the new appendix entry, and the Stage 6 progress notes now call out the Sassanian / `Zij al-Shah` spelling explicitly
- no spec change was needed; this is a release-hardening interoperability refinement within the existing compatibility-profile scope

Remaining Stage 6 work: keep the source-label appendix, catalog breadth, and release notes synchronized as additional interoperability spellings land.

## 2026-04-23 — Unknown-motion chart-helper wording aligned

Implemented a small Stage 6 release-hardening slice:

- `pleiades-core::ApiStabilityProfile` now spells out the `ChartSnapshot` unknown-motion helper surface explicitly as part of the stable chart API, matching the release-facing README wording
- the API-stability regression test now checks for the unknown-motion phrasing, and the README chart summary uses the same motion-class terminology for consistency
- the Stage 6 progress notes were updated to record the helper-surface wording refinement as part of the release-hardening backlog

Remaining Stage 6 work: keep the release-facing chart-helper wording, catalog breadth, and compatibility profile synchronized as the release hardening work continues.

## 2026-04-23 — Valens Moon source-label appendix broadened

Implemented a small Stage 6 release-hardening refinement:

- `pleiades-core::CompatibilityProfile` now includes the plain `Moon` search term in the Valens Moon source-label appendix, aligning the release-facing interoperability labels with the existing chart-layer alias resolution
- the compatibility-profile regression test now checks the expanded source-label rendering, and the Stage 6 progress notes were updated to record the refinement

No spec change was needed; this is a small release-profile alignment within the existing Stage 6 compatibility/search-label scope.
