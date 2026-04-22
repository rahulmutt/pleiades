# PLAN-MAILBOX

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
