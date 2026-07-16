# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-07-16

### Highlights

This release adds six capability programs on top of the report-surface
relocation program completed this cycle: rise/set/transit and fixed-star
support, local (topocentric) eclipse circumstances, fictitious bodies
(first-published `pleiades-fict` crate), planetary nodes and apsides,
phase/phase-angle/magnitude, and lunar occultations. See **Breaking
Changes** below before upgrading.

### Breaking Changes

- **Report-surface relocation, slice D (final):** report prose for `pleiades-jpl`
  (JPL snapshot backend interpolation-quality and manifest summaries; comparison,
  independent-hold-out, reference-asteroid, reference-snapshot, selected-asteroid, and
  production-generation evidence renderers; and their inherent `summary_line`/
  `validated_summary_line`/`Display` rendering) relocated from `pleiades-jpl` into
  `pleiades-validate`'s `posture::jpl` modules; `pleiades-data`'s phase-2 corpus-alignment
  renderer and the dependent PackagedArtifact target-threshold / source-fit-hold-out-sync
  rendering chain relocated into `posture::jpl`/`posture::data`. Corpus/evidence accessors and
  `REFERENCE_SNAPSHOT_*_EPOCH_JD` constants were promoted `pub(crate)` → `pub` where a copied
  renderer reads them. The redundant text-integrity gate in
  `ProductionGenerationSourceSummary::validate()` (which re-rendered its own summary line to
  string-match documented provenance fragments) was dropped in favour of the retained
  structured field checks. **Scope note:** unlike slices A–C, `pleiades-jpl` does not reach a
  zero-prose end-state. `pleiades-data` sits below `pleiades-validate` in the dependency graph
  and cannot depend on it, yet it legitimately consumes three jpl renderers
  (`production_generation_source_summary_for_report`, `format_reference_snapshot_summary`,
  `format_production_generation_boundary_source_summary`) as stored provenance strings and
  integrity-gate oracles; those three plus their transitive render tree — a minimal island of
  six report functions — remain in `pleiades-jpl`, while the other ~88% of its report-render
  layer (138 report functions, ~195 inherent render methods, ~91 evidence `Display` impls) is
  deleted. Pure relocation: no output or behaviour change (byte-identical, verified by
  release-smoke checksum parity), no version bump — compatibility profile stays 0.7.13,
  API-stability profile stays 0.3.0. Completes the four-slice report-surface relocation
  program; the workspace 0.3.0 → 0.4.0 bump + release-plz cut is a separate follow-up.

- **Report-surface relocation, slice C:** report prose for packaged offline-data
  coverage, profile, fit, target, threshold, regen, generation/body summaries,
  lookup, regenerate, thresholds, and accuracy-baseline moved from
  `pleiades-data` into `pleiades-validate`'s `posture::data` modules.
  `pleiades-data`'s packaged-data backend metadata now rebuilds its
  `data_sources` summary inline from the retained `&'static str` accessors
  and `PackagedBodyCoverageSummary::validated_summary_line` instead of
  calling the (now-relocated) report helpers, decoupling the backend from
  the report surface. Two over-exposed no-consumer items demoted to
  `pub(crate)` (`packaged_mixed_frame_batch_parity_summary_for_report`,
  `eros_self_consistency_max_longitude_arcsec`); two others originally
  targeted for demotion instead stay `pub` because relocating the renderers
  gave them genuine cross-crate runtime consumers. Pure relocation: no
  output or behavior change (byte-identical, verified by release-smoke
  checksum parity), no version bump — compatibility profile stays 0.7.13,
  API-stability profile stays 0.3.0. Slice D (`pleiades-jpl`) remains before
  the workspace 0.4.0 release.

- **Report-surface relocation, slice B:** report prose for house-catalog
  validation, ayanamsa provenance, VSOP87 source-docs (batch parity,
  body-class evidence), ELP lunar-theory (source-family, equatorial/lunar
  reference evidence), and compression artifact coverage summaries moved
  from `pleiades-houses`, `pleiades-ayanamsa`, `pleiades-vsop87`,
  `pleiades-elp`, and `pleiades-compression` into `pleiades-validate`'s
  `posture` modules. `pleiades-elp`'s backend metadata now rebuilds its
  lunar-theory source-family summary line from the retained
  `LunarTheorySourceFamilySummary` struct methods instead of calling the
  (now-relocated) report helper, decoupling the backend from the report
  surface (coupling 3). Pure relocation: no output or behavior change
  (byte-identical, verified by release-smoke checksum parity), no version
  bump — compatibility profile stays 0.7.13, API-stability profile stays
  0.3.0. Slices C (`pleiades-data`) and D (`pleiades-jpl`) remain before the
  workspace 0.4.0 release.

- **Breaking (report-surface relocation, slice A):** `pleiades-core` no longer
  exports `*_summary_for_report` wrapper functions or re-exports
  `pleiades-backend` policy summaries — call the `CompatibilityProfile` /
  `ReleaseProfileIdentifiers` methods, or use `pleiades-validate` for report
  rendering. `pleiades-backend`'s policy-prose layer (constants, summary
  structs, report functions) moved to `pleiades-validate`;
  `FrameTreatmentSummary` and the request/metadata `validate_*` contract
  functions remain. `ChartSnapshot`'s `Display` no longer embeds the global
  time-scale/frame/apparentness policy lines. Dead report helpers deleted
  from `pleiades-time`, `pleiades-apparent`, `pleiades-ayanamsa`,
  `pleiades-houses`. API-stability profile 0.2.2 → 0.3.0.

### Added

- Bennett true->apparent atmospheric refraction + Atmosphere ([4fd7c0f](https://github.com/rahulmutt/pleiades/commit/4fd7c0fb958b945fe41429bcffafcc3316a0de5d))
- Saemundsson apparent->true refraction inverse ([737a4a8](https://github.com/rahulmutt/pleiades/commit/737a4a88817fee26972f0c38f0d469fdacf4298b))
- Swe_azalt horizontal coordinates on EventEngine ([97aeadb](https://github.com/rahulmutt/pleiades/commit/97aeadbf9e21b47d8337466984c99ff72bcf21cf))
- Swe_azalt_rev horizontal->equatorial inverse ([61d1a70](https://github.com/rahulmutt/pleiades/commit/61d1a707b75500da279cf935c11adb6504061d01))
- Curated fixed-star catalog + reader ([f5b2675](https://github.com/rahulmutt/pleiades/commit/f5b2675d62305897d70237ba14a4495907996e80))
- Fixed-star apparent equatorial of date ([7e20fcc](https://github.com/rahulmutt/pleiades/commit/7e20fcc41be02c3a1505d65b1be92322d9b98fe6))
- Rise/set/transit types + semidiameter table ([9ce20f2](https://github.com/rahulmutt/pleiades/commit/9ce20f2d625a36bcc11117021715cc02ccb5ed79))
- Topocentric RA/Dec for body/ecliptic-point/star targets ([b20fb1a](https://github.com/rahulmutt/pleiades/commit/b20fb1a05835f9a2739aeb52166f180e2eefd168))
- Rise/set standard-altitude assembly + apparent-altitude evaluator ([88660be](https://github.com/rahulmutt/pleiades/commit/88660be89140257beab5ce0d8e57e9cabf92d21c))
- Rise/set root-finding on apparent-altitude residual ([329806a](https://github.com/rahulmutt/pleiades/commit/329806a80f2977aa0c49577476c425a15f68e551))
- Upper/lower meridian transit via hour-angle root-finding ([b7fbd5d](https://github.com/rahulmutt/pleiades/commit/b7fbd5db99cbb603fa1ca9804916e78411f9e79e))
- Circumpolar None + atmosphere/star/window fail-closed + rise doctest ([0e14bc3](https://github.com/rahulmutt/pleiades/commit/0e14bc3addfe0e0b805a90ae23b73e031f849c78))
- Two-tier validate-rise-trans SE-parity gate ([16fd583](https://github.com/rahulmutt/pleiades/commit/16fd5837166b526f0b4a64d4490e34d69d7073a3))
- Wire validate-rise-trans into release-smoke/release-gate ([fe80d30](https://github.com/rahulmutt/pleiades/commit/fe80d30665f4de1f2c2585231fcee93bfe22ecd8))
- Rise-trans + azalt aliases via validate render layer ([47263db](https://github.com/rahulmutt/pleiades/commit/47263db579b4490a29055295f5dc1c0e6067918d))
- SP-2c local circumstance value types ([e9ca4f2](https://github.com/rahulmutt/pleiades/commit/e9ca4f260701d1884c64d6f001bc8fe3ad2c4294))
- SP-2c topocentric Sun/Moon sample helper ([c548dc2](https://github.com/rahulmutt/pleiades/commit/c548dc296da47829feac04b807497ae6b9f01f73))
- SP-2c solar two-circle geometry (magnitude + obscuration) ([70ddcbc](https://github.com/rahulmutt/pleiades/commit/70ddcbcabdb7e8daee594a4ae60232bb2c32ca41))
- SP-2c solar local maximum + C1-C4 contact times ([b3c2482](https://github.com/rahulmutt/pleiades/commit/b3c2482322171303a102aa2fdad71dda91124b84))
- SP-2c horizontal position + visibility helper ([c0ef094](https://github.com/rahulmutt/pleiades/commit/c0ef0945378146b6f3943724597ae71dc324df2f))
- SP-2c assemble solar local circumstances ([321d210](https://github.com/rahulmutt/pleiades/commit/321d210d67a956d54ae151e41f085b7b40a75768))
- SP-2c lunar shadow contacts + local circumstances ([048c7e9](https://github.com/rahulmutt/pleiades/commit/048c7e9a1e776ae59e58645e40b8064b164f8b4c))
- SP-2c EclipseEngine local_circumstances + when_loc search ([8d05d00](https://github.com/rahulmutt/pleiades/commit/8d05d006d1abd92c71efe791abebe3df2fbcdf77))
- SP-2c validate-eclipses-local two-tier SE-parity gate ([02ab379](https://github.com/rahulmutt/pleiades/commit/02ab37964e8c24ed8bc1ff2cca8d47e05357a990))
- SP-2c wire validate-eclipses-local into gate battery + eclipse-local alias ([541bdb6](https://github.com/rahulmutt/pleiades/commit/541bdb6772d661c7bcb35a5c52378a6edc6dbbb9))
- SP-3 fictitious body variants + Fictitious body class ([fd6c0c9](https://github.com/rahulmutt/pleiades/commit/fd6c0c9a4f4b0155759888e4bfea0f11cffac962))
- SP-3 pleiades-fict crate scaffold + Kepler solver core ([aa92624](https://github.com/rahulmutt/pleiades/commit/aa92624f953fae135ad4e055d3e390361df5c20e))
- SP-3 orbital-element model + J2000 frame rotation ([f661576](https://github.com/rahulmutt/pleiades/commit/f66157618901a9b47ac34addac09243200334f99))
- SP-3 committed seorbel.txt element table + parser ([7fd651c](https://github.com/rahulmutt/pleiades/commit/7fd651c487fce1c48e1c8d566b6f7281b8273d21))
- SP-3 FictitiousBackend EphemerisBackend impl + claims ([aeb6a85](https://github.com/rahulmutt/pleiades/commit/aeb6a850bb21f9cddc4f538b2ec5c843e64f7f99))
- SP-3 route fictitious bodies into the chart backend chain ([d00eb17](https://github.com/rahulmutt/pleiades/commit/d00eb17b526f27f2d252ceb54040a31a2e101d34))
- SP-3 validate-fictitious two-tier SE-parity gate ([772588e](https://github.com/rahulmutt/pleiades/commit/772588e80e6f940cb7874383a0533d24a16102f3))
- SP-3 wire validate-fictitious into gate battery + fictitious-gate alias ([8faa938](https://github.com/rahulmutt/pleiades/commit/8faa938de440e81330e4381adedd49400e60504c))
- SP-4 general element/point geometry (elements_from_state, points_from_elements) ([71ea2bb](https://github.com/rahulmutt/pleiades/commit/71ea2bb1707160f30ad0238b0350b51da6494366))
- SP-4 nod_aps public types + error variants + scaffold ([3ccb8b9](https://github.com/rahulmutt/pleiades/commit/3ccb8b96fa8da3abadac5633b990d25c0b4cd380))
- SP-4 SE VSOP87 mean-element tables + mass ratios + mu helpers ([b3aae40](https://github.com/rahulmutt/pleiades/commit/b3aae406390909c4b75412698bd24f099f67bb41))
- SP-4 nod_aps entry point + mean-element path (planets, Sun, Moon) ([56406e4](https://github.com/rahulmutt/pleiades/commit/56406e41031c1dabb5c5dcc791f48d06776f6af5))
- SP-4 osculating nod_aps path (helio/geo/EMB/SSB centers, per-body dt) ([3351e13](https://github.com/rahulmutt/pleiades/commit/3351e137bc69fca238e2cd45e0f769d280f24f97))
- SP-4 validate-nod-aps SE-parity gate (provisional ceilings) ([4b7c149](https://github.com/rahulmutt/pleiades/commit/4b7c149c8a4edcf5195fa48237cebc9eb39f512a))
- SP-4 zero Sun nodes (§R8) and pin nod-aps gate ceilings from measured residuals ([7a18c77](https://github.com/rahulmutt/pleiades/commit/7a18c775e1023fcd8cd20fa4c9fcbf48d9d4687f))
- SP-5 pheno public type + module scaffold ([a710cc1](https://github.com/rahulmutt/pleiades/commit/a710cc165f1f3465856e9930498b5c4a8a999f56))
- SP-5 SE 2.10.03 apparent-magnitude + disc-diameter models ([b5029a0](https://github.com/rahulmutt/pleiades/commit/b5029a0233772a23aff30d36de9cb3e5126343a8))
- SP-5 pheno illumination geometry + EventEngine::pheno ([82d6550](https://github.com/rahulmutt/pleiades/commit/82d6550863348bbec340fc3c79d8dc705fda123c))
- SP-5 validate-pheno SE-parity gate (provisional ceilings) ([23b69ac](https://github.com/rahulmutt/pleiades/commit/23b69ace9393542faf4be6d3a22ac9884cc57974))
- SP-5 pin pheno gate ceilings from measured residuals ([b72f4a1](https://github.com/rahulmutt/pleiades/commit/b72f4a174cef5536c8dad796ffd31c6c11c39d15))
- SP-6 occultation public types + module scaffold ([3e49870](https://github.com/rahulmutt/pleiades/commit/3e4987006d84b1dedd08a09e2fc9993c2acf4990))
- SP-6 two-circle occultation geometry + classification ([f0f77e3](https://github.com/rahulmutt/pleiades/commit/f0f77e3e28162ef17e1ea2c63d20ef44fbe943cc))
- SP-6 Moon/target RA-Dec + semidiameter sampler ([0849716](https://github.com/rahulmutt/pleiades/commit/084971678dba1c6886a579f42a1fb0afc902eac8))
- SP-6 occult target resolution + un-occultable-star reject ([6f278e8](https://github.com/rahulmutt/pleiades/commit/6f278e84da896af581c65c458ea01d7234f41e6f))
- SP-6 occultation how (local circumstances) ([6cad60b](https://github.com/rahulmutt/pleiades/commit/6cad60bd61de4aed98c58e6d2398e24a718274fc))
- SP-6 next/previous_occultation (when_loc) ([41fd9d2](https://github.com/rahulmutt/pleiades/commit/41fd9d223863b16c5437469d861332834903088b))
- SP-6 next_global_occultation (when_glob) + sub-lunar point ([78019bd](https://github.com/rahulmutt/pleiades/commit/78019bd0aff53a176e42212f75904c299385760c))
- SP-6 occultation gate ceilings (provisional) ([1fb2c19](https://github.com/rahulmutt/pleiades/commit/1fb2c191588d878b1e28e2f806e2350ba9b64a99))
- SP-6 validate-occultations SE-parity gate + pinned ceilings ([abe4114](https://github.com/rahulmutt/pleiades/commit/abe41149838dfa4c2170eec5c046ac0fd09e4d4e))
- SP-6 gate occultation sub-lunar point + planet-grazing obscuration ([dc4aa5e](https://github.com/rahulmutt/pleiades/commit/dc4aa5e8170639010ee6c7c2bd691ad7ef0488d4))
- SP-6 wire validate-occultations into CLI + release battery ([3dc944c](https://github.com/rahulmutt/pleiades/commit/3dc944c1e54cd6f05898f865646224cf3fe592a0))
- SP-6-FU port SE eclipse_where axis-pierce central test (pure geometry + unit tests) ([72ae4fb](https://github.com/rahulmutt/pleiades/commit/72ae4fb3267daf2b76978d4387dc4439c35af43c))
- SP-6-FU hard-gate planet central exact-bool; measure star central; resolve KNOWN GAP 2 docs ([cfba21c](https://github.com/rahulmutt/pleiades/commit/cfba21ce4950c68ac4cf45d84ccada6ef2b02677))
- SP-6-FU occult_stage_diagnostics doc-hidden stage dump for graze-boundary differential analysis ([4fca48e](https://github.com/rahulmutt/pleiades/commit/4fca48e04408da6ed755162bd012d376bca84cc8))
- SP-6-FU tighten miss-classification pin to measured count; KNOWN GAP 3 resolution docs ([e941333](https://github.com/rahulmutt/pleiades/commit/e941333d916a34957aeffdf8b13badfd3e77ad3f))

### Fixed

- Last_crossing_before evaluates low anchor so backend errors fail closed ([921c03a](https://github.com/rahulmutt/pleiades/commit/921c03ad8273be742e4eceb1fba681cf5c75b5f2))
- Clamp azalt asin domain (never-NaN) + compute obliquity only for ecliptic input ([9762be4](https://github.com/rahulmutt/pleiades/commit/9762be499a244bc3255fd8ec043d6a08d5b40674))
- Rise/set uses SE apparent-altitude model (h0=-SD, drop double-counted refraction) ([f97584b](https://github.com/rahulmutt/pleiades/commit/f97584bcee3068be8f4bea2726e52f57f67ff5a2))
- Rise/set matches SE — drop elevation dip, bound search to ~1 day (circumpolar => None) ([8c40192](https://github.com/rahulmutt/pleiades/commit/8c401927ccdadf71f3b41dd41f369bd897616b28))
- Pin below-horizon refraction to SE; tighten rise-trans grazing/floor ceilings ([f00cdbe](https://github.com/rahulmutt/pleiades/commit/f00cdbed19e8328deef274835aae38578ce9edd1))
- SP-2c relabel 2013 hybrid observers as partial + correct corpus MANIFEST ([ea7582b](https://github.com/rahulmutt/pleiades/commit/ea7582bbfc180a4a973d9f3d4a5c13a8363ece25))
- SP-2c apply apparent-of-date to Sun/Moon in local topocentric path ([1b26748](https://github.com/rahulmutt/pleiades/commit/1b26748a6a934d4317fe8c45146454207d53c291))
- SP-2c emit local-eclipse corpus in TT to match engine time base ([c75752d](https://github.com/rahulmutt/pleiades/commit/c75752d85e1392c6d1526a4ba436dfbd08edc6c4))
- SP-2c next_local_eclipse must not report magnitude-0 non-eclipses as visible ([9c98790](https://github.com/rahulmutt/pleiades/commit/9c98790fcb844f8a01089b9f41eaceb1f9012b12))
- Drop rustdoc intra-doc links to private items ([34ce2a1](https://github.com/rahulmutt/pleiades/commit/34ce2a1c91377f7b1d49bc29cb5ab1af85969528))
- SP-4 nod_aps — inertial-frame osculating state, Sun recenter-then-negate, of-date lunar mean elements ([1b61c56](https://github.com/rahulmutt/pleiades/commit/1b61c56f274eb1952f9d44b5398318e164732b25))
- SP-5 silence clippy::approx_constant on SE's literal 2.7182818 (§E5) ([0c67c8f](https://github.com/rahulmutt/pleiades/commit/0c67c8f1d8b8c333aa5b212e5a04e5f0bc34bf93))
- SP-6 next_global_occultation reports the central-observation point, not the sub-Moon point ([ec0a6f0](https://github.com/rahulmutt/pleiades/commit/ec0a6f0b7df18135c602ca8a66375d917f0b0a05))
- SP-6 converge sub-lunar minimizer; tighten SUBLUNAR_ARCMIN gate ([33b9bc9](https://github.com/rahulmutt/pleiades/commit/33b9bc9687f7ad3822898869b7c483237325684a))
- SP-6-FU decouple GlobalOccultation::central from occ_type via SE axis-pierce test (Saturn 2/6 central mismatch -> 0) ([51c4935](https://github.com/rahulmutt/pleiades/commit/51c4935a69d16554a74348af94a557a2a761ee0a))
- SP-6-FU reconcile miss-row comparison with SE when_loc visibility semantics (SE-equivalent Miss = geometric Miss OR no phase visible) ([4bf919b](https://github.com/rahulmutt/pleiades/commit/4bf919bdd71201ce0479ac6b09e487a7b7bb4af5))
- Repair full-CI breakage surfaced by slice A close-out ([27467c9](https://github.com/rahulmutt/pleiades/commit/27467c9dd919001a69328f101fbb5315af779019))
- Repair rustfmt drift and rustdoc link surfaced by Slice B close-out ([770e4a1](https://github.com/rahulmutt/pleiades/commit/770e4a1e4c91f7d6c3f5d4e040bcc9012d0b774b))

### Performance

- Early-return previous_longitude_crossing via backward-scan twin (output-equivalent) ([ac60250](https://github.com/rahulmutt/pleiades/commit/ac60250f3bf7b443bbd844a1fec7639278deebbd))

## [0.3.0] - 2026-07-01

### Highlights

This release first-publishes five new crates — `pleiades-apparent`
(apparent-place corrections), `pleiades-apsides` (apsides/nodes),
`pleiades-eclipse` (eclipse computation), `pleiades-time` (civil/dynamical
time conversion), and `pleiades-data` (packaged offline ephemeris data and
backend) — and bumps the existing nine crates to 0.3.0. See **Breaking
Changes** below before upgrading.

### Breaking Changes

- Add `claim_tier` to `HouseSystemDescriptor`; mark 12 release-grade systems ([887371a](https://github.com/rahulmutt/pleiades/commit/887371a49af0dacd1c6a55bbc47175cb400cfd49))
- Add `claim_tier` to `AyanamsaDescriptor`; mark 6 release-grade modes ([7a22d13](https://github.com/rahulmutt/pleiades/commit/7a22d13f6bb6f4859b5b3a6cdfdaa6389a31b208))

### Added

- Scaffold spk module with error type and ReadAt ([d69cd52](https://github.com/rahulmutt/pleiades/commit/d69cd52f72b06bfead990a332d73419af8fe642c))
- Add endian-aware primitive byte readers ([0c3d323](https://github.com/rahulmutt/pleiades/commit/0c3d3238215a8efc1158773a337a32e256a6d5e1))
- Parse DAF file record and SPK segment descriptors ([a8ecfb9](https://github.com/rahulmutt/pleiades/commit/a8ecfb97d9e5ccc390cfde07bce8f16c2c576a8d))
- Add SPK type 2/3 Chebyshev segment decoder ([f5cfe7a](https://github.com/rahulmutt/pleiades/commit/f5cfe7aeea2d411f5601ef25d7c3a5b31d16b4d3))
- Add SPK type 1/21 modified-difference-array decoder ([60de957](https://github.com/rahulmutt/pleiades/commit/60de95730f89161cd1abd1100a1d3ccc2576f738))
- Add SPK kernel pool with routing and coverage ([c49deec](https://github.com/rahulmutt/pleiades/commit/c49deec53f17a42288204ccdd21994a0437a305b))
- Add NAIF-id mapping, geocentric chaining, ecliptic reduction ([5546e4b](https://github.com/rahulmutt/pleiades/commit/5546e4bbf6e8ca3a71b6ca93d91f5a73f51b1ac0))
- Add SpkBackend runtime EphemerisBackend over SPK kernels ([0cf22a8](https://github.com/rahulmutt/pleiades/commit/0cf22a82bfc0c6bc0105cebefdbc725f840b6b10))
- Add generate-spk-corpus command over SPK kernels ([b359610](https://github.com/rahulmutt/pleiades/commit/b3596101a976941245cc51ae8905d6822b5249a9))
- Label packaged range as 1600-2600 CE ([5a080ee](https://github.com/rahulmutt/pleiades/commit/5a080ee82ca49db0bdef35398be2b0e1cbf01c39))
- Scaffold checked-in corpus slice files and manifest ([b14f051](https://github.com/rahulmutt/pleiades/commit/b14f0517d8c29786a91bbb0994d00e1287f9566f))
- Add corpus spec roles and body sets ([1f67044](https://github.com/rahulmutt/pleiades/commit/1f6704492dfab093ddd3aced94ce0f332051eaf7))
- Add body-speed-scaled interior backbone epoch grid ([38658c3](https://github.com/rahulmutt/pleiades/commit/38658c3a7c2a9c2d14758d6e973176e1575aafe1))
- Add boundary, fast-cluster, and seeded hold-out epochs ([e921b4c](https://github.com/rahulmutt/pleiades/commit/e921b4ca6c2b50097adc7d802c070e11bbc5a8a9))
- Add corpus content-checksum helper ([d20ad12](https://github.com/rahulmutt/pleiades/commit/d20ad12ed27ef14fc3c1c57645378ef89786802e))
- Add corpus manifest render/parse round-trip ([f9f3d4c](https://github.com/rahulmutt/pleiades/commit/f9f3d4c61d2e469bf9be309a3d928a12709bf948))
- Generate corpus slices and manifest from the spec ([6209f18](https://github.com/rahulmutt/pleiades/commit/6209f1804a894802b10dd0ed9ac68fed96748f93))
- Add fail-closed corpus completeness gate ([cd17f48](https://github.com/rahulmutt/pleiades/commit/cd17f481bd1ea47eee39074193bc6e1d7e165e5d))
- Add corpus schema and checksum-drift checks ([925d882](https://github.com/rahulmutt/pleiades/commit/925d882f3016fc22e1d986bbe9318768cfbaf138))
- Add validate-corpus command over checked-in slices ([8eb72a8](https://github.com/rahulmutt/pleiades/commit/8eb72a8c2197eee293f75e757bfcf7ad61cefa38))
- Add generate-spk-corpus --emit-slices slice+manifest mode ([3e7fd8d](https://github.com/rahulmutt/pleiades/commit/3e7fd8dccc96c51006968a27331ed1f243eafc4f))
- Fixture-golden tolerance cross-check in corpus gate ([62138be](https://github.com/rahulmutt/pleiades/commit/62138be4e96a0ebaef3bd4583eca105346df5f11))
- Add anchor epochs + per-body interior epochs to corpus spec ([9381393](https://github.com/rahulmutt/pleiades/commit/93813932e6ff8d8cf97bec8e8dbc0242cd13bbc1))
- Sample interior backbone per body at spec cadence + anchors ([960ea69](https://github.com/rahulmutt/pleiades/commit/960ea691d494bc4149f3329d22288c1a07551568))
- Add generate-fixture-golden (independent Horizons source) ([32c1f8a](https://github.com/rahulmutt/pleiades/commit/32c1f8ac7d3fd20f6c1a30575c8e78b60468bf1d))
- Pin de440.bsp SHA-256 in corpus spec + sourcing doc ([e3018f3](https://github.com/rahulmutt/pleiades/commit/e3018f31c8a540b56257dc48a01ad0a4156a62f3))
- Regenerate real de440 corpus at full breadth (Task 11) ([f745323](https://github.com/rahulmutt/pleiades/commit/f74532384552e6d6f8ee81714e8803ca61b8e597))
- Scaffold ingest module with format-neutral IR ([d5c44d2](https://github.com/rahulmutt/pleiades/commit/d5c44d20072ac09f9d9360d7952f22553fff92e9))
- Add ingest error taxonomy and InputFormat enum ([7f190ed](https://github.com/rahulmutt/pleiades/commit/7f190ed33ab450aefb273b49161908dbb9bec78e))
- Add ExpectedProfile, Units/Center, and ingest provenance ([9ffed40](https://github.com/rahulmutt/pleiades/commit/9ffed403c4a91c093186570ece0f7f696fd136dc))
- Add fail-closed format detector ([e3006a5](https://github.com/rahulmutt/pleiades/commit/e3006a5d20f2c9223f9d4c3f1ae104e9835c92f2))
- Add fail-closed attribute resolution to ingest normalizer ([86110f6](https://github.com/rahulmutt/pleiades/commit/86110f6d379bdd7b39d58533390fd29553e56bca))
- Normalize ingest records into SnapshotCorpus with provenance ([9c25a1a](https://github.com/rahulmutt/pleiades/commit/9c25a1a795d554d6ded79fce0ba1511c8b51f852))
- Add Horizons vector-table front-end ([9865f69](https://github.com/rahulmutt/pleiades/commit/9865f6982eaa055d6f2c6998e39537a19302f597))
- Add Horizons API JSON front-end delegating to vector-table ([736fefa](https://github.com/rahulmutt/pleiades/commit/736fefaf2338282b5786db59ec2d78da7ff043a4))
- Add tolerant generic-CSV front-end with column aliasing ([ad791df](https://github.com/rahulmutt/pleiades/commit/ad791df50702d1a794fe7cc903fd72b837452d63))
- Wire public read_public_corpus API end-to-end ([79036cf](https://github.com/rahulmutt/pleiades/commit/79036cfc86c88c704c18e80c17cb64cff3decbca))
- Add quarantined live Horizons fetch behind horizons-fetch feature ([aeda046](https://github.com/rahulmutt/pleiades/commit/aeda0460f343bfbfc6fc54db39589dd1a9de534a))
- Add offline ingest-public subcommand ([3b0e69d](https://github.com/rahulmutt/pleiades/commit/3b0e69d26bb0d806c822be2504e15509e45c9e16))
- Use pure-Rust rustls+graviola TLS for horizons-fetch ([26379bc](https://github.com/rahulmutt/pleiades/commit/26379bc5b578c7d06fd519fd2952fe27e609b517))
- Exempt never-compiled pure-Rust TLS phantoms from lockfile audit ([fc7de4d](https://github.com/rahulmutt/pleiades/commit/fc7de4d31c78d1ee45955986f48793b118a8b57d))
- Add asteroid window + sb441-n16 kernel identity constants ([26944ea](https://github.com/rahulmutt/pleiades/commit/26944ea308968b8f2a613b17b165fd9881d24d9c))
- Add curated asteroid roster with tier/class tags ([b7a6903](https://github.com/rahulmutt/pleiades/commit/b7a6903f6d1c66703e83f6824fffe235ca382792))
- Add speed-scaled asteroid epoch grid for 1900-2100 ([7fde632](https://github.com/rahulmutt/pleiades/commit/7fde63291a54a4c907de067fb4ded35abef807cf))
- Add asteroid_reference/asteroid_constrained slice roles ([6b7b332](https://github.com/rahulmutt/pleiades/commit/6b7b332ee7708c1ed353e30e12c932d41e9a158b))
- Generate Tier A asteroid_reference slice from pinned kernel ([b72804d](https://github.com/rahulmutt/pleiades/commit/b72804d5c4677f29820a92a4319946a0de37df0f))
- Fail-closed window/schema validation for asteroid slices ([9a4ea8a](https://github.com/rahulmutt/pleiades/commit/9a4ea8afb5874fc57094e6c9bd3d4eeb1f64c792))
- Load and gate asteroid slices in the corpus gate ([f946417](https://github.com/rahulmutt/pleiades/commit/f946417be848ecfeecf3bc237aaf84cde1749402))
- Report curated asteroids as a constrained 1900-2100 class ([65ae3ac](https://github.com/rahulmutt/pleiades/commit/65ae3acbc1dbd063f952ce1d468088d8a764cb7b))
- Commit Tier A asteroid corpus from pinned sb441-n16 kernel ([f2f1ade](https://github.com/rahulmutt/pleiades/commit/f2f1adeaf8c5f3b96b3308f7780b259cd8d67e17))
- Commit Tier B constrained asteroid corpus from JPL Horizons ([71b91ef](https://github.com/rahulmutt/pleiades/commit/71b91efc42ffbaa537b9a84ceb687aca91d804e0))
- Expose typed production corpus accessors ([d361472](https://github.com/rahulmutt/pleiades/commit/d361472b290eea170257180779630e5cd9385728))
- Add SnapshotCorpusBackend over explicit entries ([e42860c](https://github.com/rahulmutt/pleiades/commit/e42860ca0de6c96d0be6864480688bcb244b0e46))
- Add per-body fitting cadence model for dense generation ([21cb06d](https://github.com/rahulmutt/pleiades/commit/21cb06dcc2b39a1bc7c6d68b03c0125035861d29))
- Least-squares within-span segment fitter ([52cd622](https://github.com/rahulmutt/pleiades/commit/52cd622967b8f5b744f7842de9cd9238c76e6148))
- Assemble dense artifact from a reference backend ([2270c93](https://github.com/rahulmutt/pleiades/commit/2270c933097eab3bbe2953b69dd3d62b954bdb24))
- Kernel-gate packaged-artifact generation; decode committed bytes kernel-free ([de98eba](https://github.com/rahulmutt/pleiades/commit/de98ebac8d2bd3babce84da2785d64400f3ba802))
- Carry constrained asteroid (Eros) from committed artifact; fit majors from reference ([a7eaa45](https://github.com/rahulmutt/pleiades/commit/a7eaa45c76e01746f33cd6d4ab16a0e66495e429))
- Re-derive constrained asteroid (Eros) from reference snapshot, not committed artifact ([5a18a53](https://github.com/rahulmutt/pleiades/commit/5a18a53275e0c5748a154a9b003b293f1912d73a))
- Regenerate draft artifact from de440 within-span fits (kernel-gated) ([abfeb19](https://github.com/rahulmutt/pleiades/commit/abfeb198bd67b0be77558f0b22b4994902ae3daa))
- Per-body accuracy baseline vs committed hold-out ([d8578b9](https://github.com/rahulmutt/pleiades/commit/d8578b9b1d86c84a23c7b446d6d51b1c107b82a8))
- Regenerate Tt-tagged dense artifact (lookup-compatible) ([f7726d5](https://github.com/rahulmutt/pleiades/commit/f7726d54fca608baa9b814f4e0f4e53de498a5dd))
- Add CoverageWindow type for parameterized generation ([c83a7ce](https://github.com/rahulmutt/pleiades/commit/c83a7cef94e7c34af28a06d523f13e5b997b1cc3))
- Public window-parameterized kernel generation API ([0cafb1d](https://github.com/rahulmutt/pleiades/commit/0cafb1d50dc5623f1c9f1661d9a3cf3e57320004))
- Generate-artifact subcommand for custom coverage windows ([17b0678](https://github.com/rahulmutt/pleiades/commit/17b0678c209b9f7078c7c9bd2af63cc976a6a47d))
- Narrow default coverage window to 1900-2100; regenerate artifact, corpus, golden ([baadb88](https://github.com/rahulmutt/pleiades/commit/baadb88b062ecc60aad00c46fab5739dd5584893))
- Ecliptic Cartesian recombination helpers for frame reframe ([3b213ab](https://github.com/rahulmutt/pleiades/commit/3b213ab3de54dd4b0e465e4c7cf8b5b6b0bc76b7))
- Add per-body StoredFrame (geocentric default), no serialization yet ([e77352c](https://github.com/rahulmutt/pleiades/commit/e77352cdb3b0ffaf92b6e60e4e0896a2687d1374))
- Reconstruct geocentric for heliocentric bodies; require Sun reference ([b206357](https://github.com/rahulmutt/pleiades/commit/b206357c6d666297780681deabd974c2e9e2c9ce))
- Fit planets heliocentrically via Sun-subtraction; tag StoredFrame ([14b5806](https://github.com/rahulmutt/pleiades/commit/14b5806c27f78de16b444ce21db893d4ad72ea24))
- ARTIFACT_VERSION 6 with per-body frame byte; regenerate heliocentric-planet artifact ([d38505e](https://github.com/rahulmutt/pleiades/commit/d38505eb0446cb92493d7a3c3c2d495252b76668))
- Analytic derivative for polynomial channels and segments ([d5e85d8](https://github.com/rahulmutt/pleiades/commit/d5e85d8cf192d17016a2b7cb4dae32bac040198f))
- Spherical<->cartesian velocity recombination for motion ([34a26b7](https://github.com/rahulmutt/pleiades/commit/34a26b7ae3fa807669121ebfadbf9cc08948d0fd))
- CompressedArtifact::lookup_motion (geocentric direct + heliocentric recombination) ([192e1aa](https://github.com/rahulmutt/pleiades/commit/192e1aaea3fd981e51d1b54134dc6bcf1b6bc400))
- Declare packaged Motion=Derived, speed policy FittedDerivative ([cb87fc7](https://github.com/rahulmutt/pleiades/commit/cb87fc769f2409bd5b0998bf267b23fb5462ece1))
- Packaged backend returns derived motion in EphemerisResult ([0d0f2eb](https://github.com/rahulmutt/pleiades/commit/0d0f2ebe3d271b43a23da6590afb005aa8f56a8a))
- Optional velocity columns on SnapshotEntry; parser accepts 5- or 8-field rows ([c3634f2](https://github.com/rahulmutt/pleiades/commit/c3634f24505b0ffb76a556186a853164cb661c38))
- Hold-out slice carries de440 ecliptic velocity truth (kernel-gated regen) ([0b637e2](https://github.com/rahulmutt/pleiades/commit/0b637e2fd4bf3e25a667fbdfff3586f51827e3bb))
- Corpus gate accepts optional 8-field velocity rows, fail-closed on non-finite ([803e81d](https://github.com/rahulmutt/pleiades/commit/803e81d877251b5d92d7c788a423b184fa827bff))
- Thresholds.rs published accuracy ceilings + size/latency budgets (SSOT) ([cd546b6](https://github.com/rahulmutt/pleiades/commit/cd546b6e258d24d62505a1a2b0082fe9cffe4f83))
- Measure per-body speed error vs hold-out velocity truth ([3e4fc2e](https://github.com/rahulmutt/pleiades/commit/3e4fc2ea57ddbfb9d7ff2c499ece71b36db28616))
- Packaged-artifact-thresholds-summary command + help-sync + golden ([bc39e5d](https://github.com/rahulmutt/pleiades/commit/bc39e5d9a550c890df6802aaf9c6f1fba51065f6))
- Tracked latency-budget summary + opt-in enforcement gate ([4fd0cdf](https://github.com/rahulmutt/pleiades/commit/4fd0cdf1df10bf666b61e6fe60eb80dd0cf64332))
- Add BodyClaim/BodyClaimTier/ClaimEvidence types ([423dab7](https://github.com/rahulmutt/pleiades/commit/423dab7496d8d94acbc442c0c0ce89a8a20993a4))
- Packaged backend declares 11 release-grade body claims ([c59f782](https://github.com/rahulmutt/pleiades/commit/c59f782711b7fd4684f44009429c6014e78bc329))
- Declare constrained majors and approximate Pluto claims ([7b68eb7](https://github.com/rahulmutt/pleiades/commit/7b68eb741ea7d16090aac64c99a89db5bab492e9))
- Declare constrained lunar claims and unsupported true apsides ([5821a31](https://github.com/rahulmutt/pleiades/commit/5821a31ed93b29e3de5efae4c55a197ab4897f61))
- SPK declares sb441-n16 Tier-A release-grade asteroid claims ([7a84d9b](https://github.com/rahulmutt/pleiades/commit/7a84d9bd3886f7755e03ea09d56d5d816f0bd52c))
- Add ReleasePosture derived from backend metadata ([51316e0](https://github.com/rahulmutt/pleiades/commit/51316e04cb5be7b9cf1076ae4e9bd669195ecb1e))
- Derive release posture from canonical backends; retire global string claim layer ([c4b1f46](https://github.com/rahulmutt/pleiades/commit/c4b1f4618d657d21027a4fcac3550166f0dbe3e2))
- Add claim drift gate comparing surfaces to derived posture ([eab6cc4](https://github.com/rahulmutt/pleiades/commit/eab6cc4bdbd4f91d60c74f1944ac884778fa9198))
- Add fast structural claim audit ([ad249c7](https://github.com/rahulmutt/pleiades/commit/ad249c77bba387b8ef5cfc80c2869a23ad7b6403))
- Add slow corpus accuracy-ceiling audit for release-grade bodies ([25584c9](https://github.com/rahulmutt/pleiades/commit/25584c9211f8bd68b5542ee8fe4b4e026702697a))
- Add claims-audit command and wire into release-gate/ci ([47f275f](https://github.com/rahulmutt/pleiades/commit/47f275f8453ac897052d21300f635dad1b15e1bc))
- Scaffold pleiades-time crate with fnv1a64 helper ([6dbcabf](https://github.com/rahulmutt/pleiades/commit/6dbcabfc41d1a56d7b23fbfe280349b3ccfaeecf))
- Add CivilTimeError ([abd135b](https://github.com/rahulmutt/pleiades/commit/abd135b6c4ff04e4f2c9c1fa15a8839127da3c88))
- Add CivilDateTime calendar <-> JulianDay conversion ([f85d3be](https://github.com/rahulmutt/pleiades/commit/f85d3be34ad73124575a3be414e2fbd80c4606f4))
- Add checksum-pinned leap-second table and lookup ([2b51c6e](https://github.com/rahulmutt/pleiades/commit/2b51c6e59f913cdb9a47d7870b201d5e777076e2))
- Add checksum-pinned Delta-T table with extrapolation ([569f254](https://github.com/rahulmutt/pleiades/commit/569f254770f3eedb3245a1c42f5ef1741700856e))
- Add TT<->TDB periodic term ([b3b248c](https://github.com/rahulmutt/pleiades/commit/b3b248c60885863c80cdb8f84a30a4d0e57a576e))
- Add civil-time orchestrator with tiered provenance ([49adf3f](https://github.com/rahulmutt/pleiades/commit/49adf3faaa09983a409f9b80e17e9a87069e35ad))
- Add civil-time policy summary and reverse backend non-goal posture ([9033330](https://github.com/rahulmutt/pleiades/commit/9033330e544547596741b88866b69b888d6546b4))
- Add ChartRequest::from_civil via pleiades-time ([698478a](https://github.com/rahulmutt/pleiades/commit/698478a31f40704cfd7ada0c38b1dd75ee6d3e03))
- Add --civil chart input with conversion provenance output ([f52d458](https://github.com/rahulmutt/pleiades/commit/f52d458cc79f93f9aabc1aaaf06b500b871344d1))
- Scaffold pleiades-apparent crate with fnv1a64 helper ([9de7712](https://github.com/rahulmutt/pleiades/commit/9de7712eadf7140ffc3f08588602449578bd4223))
- Add ApparentPlaceError and ApparentLightTimeError ([0cb01c6](https://github.com/rahulmutt/pleiades/commit/0cb01c6934f72dc736087f9d62f1bb0645dee658))
- Add checksum-pinned IAU-1980 nutation series ([c0ccce6](https://github.com/rahulmutt/pleiades/commit/c0ccce641361da2c6264107eb9e1b2c610f3e9fe))
- Add annual aberration in ecliptic coordinates ([cd6631f](https://github.com/rahulmutt/pleiades/commit/cd6631f993dcfc0f21b6460446cc1e85b879cf25))
- Add light-time iterator ([8e5733d](https://github.com/rahulmutt/pleiades/commit/8e5733dca92a620536dcabc9f923abc92d851805))
- Add IAU-1976 precession (J2000 ecliptic to of date) ([53437c3](https://github.com/rahulmutt/pleiades/commit/53437c322cbcbb5199716377bef1c46d5e3f1139))
- Add ApparentProvenance and CorrectionSet ([bacb039](https://github.com/rahulmutt/pleiades/commit/bacb03991b64b6b0ac06ecc5d56dd442d177ca78))
- Add apparent_position orchestrator with precession ([ad5b32f](https://github.com/rahulmutt/pleiades/commit/ad5b32f0edb4a39d51a4a7406d83141ac602cc86))
- Add ApparentPlacePolicySummary ([1e47d0a](https://github.com/rahulmutt/pleiades/commit/1e47d0a067dc8c7fad9f707e220f768bc937c281))
- Compute apparent-place of date as the default chart output ([62084db](https://github.com/rahulmutt/pleiades/commit/62084dbdcf1b816f4493909f9c12ccdf38f5cc10))
- Apparent place of date by default; --mean diagnostic flag ([5f56603](https://github.com/rahulmutt/pleiades/commit/5f56603bac79157f665073451a89b2a9a044d19c))
- Expose UT1-from-TT and GMST scalars for sidereal time ([ad50d17](https://github.com/rahulmutt/pleiades/commit/ad50d17fcf3b9f76f250b6d413fe86793a0f5dca))
- Observer geocentric vector for diurnal parallax (Meeus 11) ([27ff1f5](https://github.com/rahulmutt/pleiades/commit/27ff1f521fb8ec5a4b48e1ae2c517f13b3ce86dd))
- Topocentric place (diurnal parallax + diurnal aberration) ([d029350](https://github.com/rahulmutt/pleiades/commit/d0293502137aff462161ac99299e23d27b0ee0e8))
- Opt-in chart-layer topocentric correction ([1d974fa](https://github.com/rahulmutt/pleiades/commit/1d974fa3e7f9d53ea0a012bc0c4a6c70be328b24))
- --topocentric and --elevation flags with provenance output ([86e6c6b](https://github.com/rahulmutt/pleiades/commit/86e6c6bbd0c879aee182fa939746019c32e1809c))
- Validate-topocentric CLI gate for release rehearsal (parity with apparent gate) ([4ae1e26](https://github.com/rahulmutt/pleiades/commit/4ae1e264b66858269f1266ad830596c152f6538b))
- Add documented latitude bound to house-system descriptor ([c36e824](https://github.com/rahulmutt/pleiades/commit/c36e82408a3fe0faf2d11ebffe6a902b85342aed))
- Strict InvalidLatitude rejection beyond documented bound ([fe6bea4](https://github.com/rahulmutt/pleiades/commit/fe6bea46d477c481d4726c7f65c915a5015d5a56))
- Opt-in SE-compat high-latitude fallback policy ([258c51c](https://github.com/rahulmutt/pleiades/commit/258c51cb0e18b2bf5ca6e4d4bb0fb8331a86299a))
- Per-formula-family arcsecond ceiling module ([7132250](https://github.com/rahulmutt/pleiades/commit/7132250bc883220aeeb96f1fe5cd601187c9afb1))
- House corpus + manifest parsers (fail-closed) ([1f1bec1](https://github.com/rahulmutt/pleiades/commit/1f1bec10ffbf6b243a98bac3585b01d5a50daea2))
- House numeric-residual gate over SE reference corpus ([8f0b4d5](https://github.com/rahulmutt/pleiades/commit/8f0b4d5b01981d5be00c51901493ec00ba676f6a))
- Gate asserts strict-rejection and SE-compat fallback paths ([f60423d](https://github.com/rahulmutt/pleiades/commit/f60423dba609293aa0daabdef21a9b0365c1ae64))
- Tighten per-family ceilings from measured SE residuals ([6459a87](https://github.com/rahulmutt/pleiades/commit/6459a8788e300292ce5c4ba6a5b56c3325571704))
- Validate-houses CLI gate (wired like validate-topocentric) ([2141802](https://github.com/rahulmutt/pleiades/commit/2141802d7f3dc9297d9b24308a7e2a191a0fba0c))
- SE ayanamsa reference corpus + generator (Phase 5) ([e00741c](https://github.com/rahulmutt/pleiades/commit/e00741cd8c862b0a84d6a2fa61a4de9cebafefe0))
- IAU-2006 general precession in longitude ([51ba91c](https://github.com/rahulmutt/pleiades/commit/51ba91c4114e6978ed9670e7f8f0f89b77f51250))
- Per-mode-class arcsecond ceiling scaffold ([ff414aa](https://github.com/rahulmutt/pleiades/commit/ff414aa70cc83a19a157957f5fbfa35d218c689d))
- Validate_ayanamsa_corpus numeric-residual gate + measured ceilings ([0af2aea](https://github.com/rahulmutt/pleiades/commit/0af2aea320cd20b199745f9868af888f5c3e0e4e))
- Validate-ayanamsa CLI gate (wired like validate-houses) ([3986b90](https://github.com/rahulmutt/pleiades/commit/3986b90e1436f0732c86e79ae1a20a04ee177100))
- Add CompatibilityClaimTier enum ([41bf59f](https://github.com/rahulmutt/pleiades/commit/41bf59f85b0c36362d8f2572b93867f8974715b2))
- Expose validated-entry sets on corpus gate reports ([db02abb](https://github.com/rahulmutt/pleiades/commit/db02abbef750f947415f00209765cc6c29d6d234))
- Overclaim audit Check A (tier <-> corpus evidence) ([a610b7f](https://github.com/rahulmutt/pleiades/commit/a610b7f97a165b61c7baa8703657d444acbf7a87))
- Overclaim audit Check B + run it in verify-compatibility-profile ([303d123](https://github.com/rahulmutt/pleiades/commit/303d12320830c630c80875ed109df599be65d611))
- Overclaim audit Check C (README prose drift) ([d2935f5](https://github.com/rahulmutt/pleiades/commit/d2935f54bbe3da3a3b326787a7232e7c1e60153d))
- Compat-claims-audit CLI; run numeric gates + audit in release-smoke ([76d7bc3](https://github.com/rahulmutt/pleiades/commit/76d7bc3d278f22e5f0e03a6a762ea6c81094ba50))
- Emit 11 standard target house systems in SE reference generator ([54bd960](https://github.com/rahulmutt/pleiades/commit/54bd960004a31fa1adf12b288ad36ad06d21c298))
- Emit Gauquelin 36 sectors via raw FFI into sectors.csv ([facf0e1](https://github.com/rahulmutt/pleiades/commit/facf0e1f6cecec350b68d68776e71bfbbf1edb7f))
- Variable-length house-sector parser + two-slice manifest ([4a6df94](https://github.com/rahulmutt/pleiades/commit/4a6df94830d7298c78ccd8351a1af8899099e0d4))
- Promote 10 standard target house systems to release-grade ([6b06016](https://github.com/rahulmutt/pleiades/commit/6b060168293d3a5d8ca7f8452c0027e89d86a8ae))
- Add SE-sourced anchor table for offset-defined family ([524d50e](https://github.com/rahulmutt/pleiades/commit/524d50eef2126526feb1f490fd5982829875da8f))
- Emit + measure offset-defined ayanamsa modes; record P/D ([2ff0185](https://github.com/rahulmutt/pleiades/commit/2ff0185c44c2b5242f06d260a3bd4868a3defbcc))
- Gate offset-defined family in mode-class map ([4691349](https://github.com/rahulmutt/pleiades/commit/469134905abe398083c1772040d72718e9777317))
- Set SE anchors + promote offset-defined family to release-grade ([abbadc4](https://github.com/rahulmutt/pleiades/commit/abbadc48b121237a4476969c65d31f256ff8807a))
- Commit SE corpus + gate promoted offset-defined ayanamsa modes (23) ([2cbb129](https://github.com/rahulmutt/pleiades/commit/2cbb129160cdbc2eb1e6511802f03766fdfc188b))
- Emit/fit/measure fitted ayanamsa family (true-star + galactic) ([da85cc0](https://github.com/rahulmutt/pleiades/commit/da85cc0d88732045fe3019327398efe56619ac66))
- Add Galactic mode class + per-family fitted ceilings ([f95a7c2](https://github.com/rahulmutt/pleiades/commit/f95a7c2e659957625eb1908477afd567bee40dbb))
- Commit fitted-family cubics + route galactic offset path ([5dd7e4e](https://github.com/rahulmutt/pleiades/commit/5dd7e4ed8164e12d81a62703f30b7fe62d886024))
- Commit fitted-family corpus rows + gate promoted modes ([7ea777e](https://github.com/rahulmutt/pleiades/commit/7ea777e0c7b68c286f57e380c81e5403f44f1d96))
- Promote fitted family to release-grade in catalog ([34cbea8](https://github.com/rahulmutt/pleiades/commit/34cbea86ee2fff81f85fe86a8b599354e55ebd1b))
- Emit/fit/measure fitted-offset ayanamsa family (slice 3) ([288d57a](https://github.com/rahulmutt/pleiades/commit/288d57ac9d273bae9e1dcfaabfbc196c8bb233c3))
- Add FittedOffset mode class + measured ceiling (slice 3) ([44c8fe4](https://github.com/rahulmutt/pleiades/commit/44c8fe4703bfd56e669ed527a3deabe0befe7d25))
- Commit fitted-offset cubics + route offset path (slice 3) ([968a84e](https://github.com/rahulmutt/pleiades/commit/968a84e42457babfbed023adf1c4e24343982dfb))
- Commit fitted-offset corpus rows + gate promoted modes (slice 3) ([4047f9a](https://github.com/rahulmutt/pleiades/commit/4047f9a67b5e946023f1fbdd3c2a9895abcf6a25))
- Promote Eunomia + Cybele to Tier-A roster (slice 1) ([d637e16](https://github.com/rahulmutt/pleiades/commit/d637e169dbf4249e8a392254888a7202fe0f1565))
- Regenerate Tier-A asteroid reference corpus with promoted bodies (slice 1) ([df5ca34](https://github.com/rahulmutt/pleiades/commit/df5ca341e4b9c6174d550b229ff21257eb7a934d))
- Pin sb441-n373s asteroid kernel, retire sb441-n16 (slice 2) ([2bf94a0](https://github.com/rahulmutt/pleiades/commit/2bf94a00619a14c0e7183ab1588c9c1d757b12dd))
- Promote n373s-confirmed asteroids+TNOs to Tier-A roster (slice 2) ([d2cb45f](https://github.com/rahulmutt/pleiades/commit/d2cb45f67d5fdd9fc00616c6053dfccc25ab7b34))
- Regen tool filters promoted bodies out of constrained slice (slice 2) ([5e8f1c2](https://github.com/rahulmutt/pleiades/commit/5e8f1c2257f165f0c88c3373de6436a40eb6f5d0))
- Regenerate asteroid corpus from sb441-n373s, filter promoted bodies (slice 2) ([1cbe6f8](https://github.com/rahulmutt/pleiades/commit/1cbe6f80132617daff3da770d627023f324da792))
- Per-object SPK manifest for kernel-absent Tier-A asteroids (slice 3) ([08f12c5](https://github.com/rahulmutt/pleiades/commit/08f12c5e73cf82c351a565768c3c836f590e57e0))
- Regen path loads per-object SPK dir alongside de440+n373s (slice 3) ([f2a1584](https://github.com/rahulmutt/pleiades/commit/f2a1584f4cf60a207de15aca5ddf55b56ddc41ca))
- Promote per-object-SPK asteroids to Tier-A, regen corpus (slice 3) ([06e0cc9](https://github.com/rahulmutt/pleiades/commit/06e0cc983d02dee778beeda6e840edd68fbe49fb))
- Scaffold pleiades-eclipse crate ([fc28b01](https://github.com/rahulmutt/pleiades/commit/fc28b011f9ef0494c21c7cd2fea47072fa0424c2))
- Add eclipse domain types ([5de3aff](https://github.com/rahulmutt/pleiades/commit/5de3aff1600275e4a81c15c48ded11e3aef7725a))
- Add fail-closed eclipse error type ([27bccd2](https://github.com/rahulmutt/pleiades/commit/27bccd23ce5584e515957db365314d641c485d1f))
- Add Sun/Moon reader and analytic test backend ([593c74f](https://github.com/rahulmutt/pleiades/commit/593c74f4a6c1bc8566ca5dfcbcff9d3ce30d83ec))
- Add syzygy (new/full moon) search ([f9ed60e](https://github.com/rahulmutt/pleiades/commit/f9ed60e7ed19c7b3bd1098c0a9bb85518f03ba04))
- Add solar eclipse geometry and classification ([616a8fd](https://github.com/rahulmutt/pleiades/commit/616a8fd0e81ddc73103cfeb7ac270b84e759dd8e))
- Add lunar eclipse geometry and classification ([ca002ea](https://github.com/rahulmutt/pleiades/commit/ca002ead56ef7859ee86bb9950d5256371d44399))
- Add Saros series numbering ([402fe4b](https://github.com/rahulmutt/pleiades/commit/402fe4bdbe34753726f0754c29dcd93deb74f96a))
- Assemble EclipseEngine with range/next/previous queries ([ba1c8d4](https://github.com/rahulmutt/pleiades/commit/ba1c8d4f70059667e5fa1e713fd648486a9dab47))
- Add NASA-canon eclipse fixture (1900-2100) generated via Skyfield+DE440 ([460a792](https://github.com/rahulmutt/pleiades/commit/460a7923f2b1dd327d6579e7b1db05eb5c4b492d))
- Add exhaustive validate-eclipses gate against the NASA canon ([b3d5757](https://github.com/rahulmutt/pleiades/commit/b3d57576ef90c4cf1a9717c389b13f2126dc7802))
- Wire validate-eclipses + eclipses CLI and release gate ([f504127](https://github.com/rahulmutt/pleiades/commit/f504127f33c757af24340ee20b2e0f5d3bbc7149))
- Add apparent_sun_position (aberration applied once for the Sun) ([cc575c0](https://github.com/rahulmutt/pleiades/commit/cc575c041b31835cdd52a78123ef5745471235f0))
- Pure osculating lunar apsides crate (true apogee/perigee) ([63f2566](https://github.com/rahulmutt/pleiades/commit/63f256642e381a09f9a9535df7fedc06b8dc3221))
- Expose spherical_state_to_cartesian (symmetric with its inverse) ([e4d39bb](https://github.com/rahulmutt/pleiades/commit/e4d39bb5b1c616b8a2e1d108e6a3ab871036c4c3))
- Apparent_apsis_position (precession+nutation only, no aberration) ([08b354c](https://github.com/rahulmutt/pleiades/commit/08b354c20d6deb35ea862b0e44d0047f071796c6))
- Serve osculating true apogee/perigee from the packaged Moon state ([45d1783](https://github.com/rahulmutt/pleiades/commit/45d1783e25e7e0a351a5672981ff4ca5a42820da))
- Serve apparent true apsides via precession+nutation-only path ([f9b9125](https://github.com/rahulmutt/pleiades/commit/f9b912527f3e4c02ea6bdbd8fc68265e50daa44a))
- Add SE-lilith-reference tool + committed SE_OSCU_APOG reference corpus ([120abb4](https://github.com/rahulmutt/pleiades/commit/120abb479286b834eda2c4dad51d3f1e55bfe17c))
- Fail-closed validate-lilith gate vs SE_OSCU_APOG + release-gate wiring ([e152fea](https://github.com/rahulmutt/pleiades/commit/e152feaf4b35e6ef890906cc7f4d98c5527175b8))
- Apparent_equatorial_of_date helper (true obliquity of date) ([0a0729f](https://github.com/rahulmutt/pleiades/commit/0a0729fb0a196419cbc3c01c793dc39d769b29df))
- Apparent equatorial of date for release-grade bodies ([c395ea3](https://github.com/rahulmutt/pleiades/commit/c395ea3db2f9621c2fec8263aed7ef395f7ddd16))
- Validate-equatorial gate vs JPL Horizons apparent RA/Dec ([d8b5430](https://github.com/rahulmutt/pleiades/commit/d8b5430b6c65d17cf7710f7546e1537941275005))
- Emit J2000 ecliptic at the boundary via date->J2000 precession (B5) ([bce6a3a](https://github.com/rahulmutt/pleiades/commit/bce6a3aa6031418a106e630a14bf93ef7f1b3e75))
- Validate-equatorial-se convention-parity gate vs Swiss Ephemeris ([b9021cf](https://github.com/rahulmutt/pleiades/commit/b9021cfcb0266f2e426c4c6e4ecc319b7128f725))
- Wire validate-equatorial + validate-equatorial-se into CLI and release gate ([ae9c2db](https://github.com/rahulmutt/pleiades/commit/ae9c2db07a8e610511938abf66b8b377f7e86647))
- Public sidereal time (GMST/GAST, local) foundation ([963fe82](https://github.com/rahulmutt/pleiades/commit/963fe8257a71288cc02b3f749b7bcad5569d8a49))
- AscMc chart points + chart_points/chart_points_from_armc ([bfce65e](https://github.com/rahulmutt/pleiades/commit/bfce65e13c4c0fbd1edc79bb1dcc5b9b429600cc))
- Carry AscMc on HouseSnapshot; mark HouseSnapshot non_exhaustive ([9cceed7](https://github.com/rahulmutt/pleiades/commit/9cceed7d4dcbf87f2583f2777088a8eea3706e80))
- Expose AscMc + sidereal time; render new angles in chart report ([de5ed51](https://github.com/rahulmutt/pleiades/commit/de5ed5175f4b771d58fa2c44c59771e91089fccf))
- Emit ascmc[2..7] + sidereal time for the angles gate ([825e5a3](https://github.com/rahulmutt/pleiades/commit/825e5a3473ac3ebc9aba513cfa7625140ee3d38e))
- Validate-angles SE-parity gate for ascmc points + sidereal time ([77a4170](https://github.com/rahulmutt/pleiades/commit/77a41705c8704333d0627eb0cf8bea3e35613ead))

### Fixed

- Validate MDA record bounds, enforce zodiac policy, fix help sync ([031e064](https://github.com/rahulmutt/pleiades/commit/031e064a8b5996cfd12d9a7fa7cd940f93715136))
- Correct RANGE_END_JD to 2600-01-01 (2_670_690.5) ([74590b0](https://github.com/rahulmutt/pleiades/commit/74590b09c957cf9b1f07976c5bf9f57fdf5da54e))
- Fail closed on malformed corpus rows + add coverage ([4e68966](https://github.com/rahulmutt/pleiades/commit/4e6896637a903ea026a7b02c5be72c2026776e33))
- Emit complete 5-slice manifest incl. fixture_golden ([5363999](https://github.com/rahulmutt/pleiades/commit/5363999f4e717d636d476782f8b71ec9d224e1d6))
- Tolerate giant-planet barycenter offset in corpus gate ([d0b5610](https://github.com/rahulmutt/pleiades/commit/d0b5610d1341ab25a5aec8234aa81f9794833356))
- Distinct corpus BackendId and isolation-proving test ([5e7bacf](https://github.com/rahulmutt/pleiades/commit/5e7bacf7d78fb11bd0b14a7992632de4adccb7fa))
- Probe decode accuracy in within-span test; fail-closed distance ([cd4e539](https://github.com/rahulmutt/pleiades/commit/cd4e539e4a3b215a4338a9183212e55164ff6049))
- Widen per-body segment count to u32 (ARTIFACT_VERSION 5) for dense artifacts ([ac3f9d8](https://github.com/rahulmutt/pleiades/commit/ac3f9d8e7820d6b7910d0d322aaa3790ed94b784))
- Tag de440-fit segment boundaries Tt so packaged lookups match ([4555f81](https://github.com/rahulmutt/pleiades/commit/4555f81b4852683a4b0f893b9fcbeb1efcb176d5))
- Accuracy baseline must query artifact on Tt convention (was vacuous); add non-vacuity guard ([6da6cc9](https://github.com/rahulmutt/pleiades/commit/6da6cc90c6a6d429148a76ca811cf8d828053808))
- Reconcile cli help-sync and validate residual-bodies posture to 1900-2100 regen ([a924fd8](https://github.com/rahulmutt/pleiades/commit/a924fd8237ee1b580b95aa69b63ffd6b7b4e8579))
- Make frame_recombine module private; expose via re-export only ([139599e](https://github.com/rahulmutt/pleiades/commit/139599e74892624539f376d660b84d9c912de157))
- Bump ARTIFACT_VERSION 6->7 and regenerate packaged artifact from de440 (speed_policy serialized) ([055fb63](https://github.com/rahulmutt/pleiades/commit/055fb636587da1727586063cce895f6dc6b485d9))
- Include custom Tier-A asteroids in SPK covered_bodies so they are release-grade in live metadata ([261ca6d](https://github.com/rahulmutt/pleiades/commit/261ca6d85920edfa6421895b323fd6088683c19d))
- Honest drift errors (distinguish render failure from drift; clearer surface naming) ([551782f](https://github.com/rahulmutt/pleiades/commit/551782f4a5ea5daaeabe3ebefecd5afd548689ef))
- Fail audit closed when packaged-data release-grade bodies are not actually compared ([dbccc88](https://github.com/rahulmutt/pleiades/commit/dbccc888d650eb73f67b5836ca94cf454b4fcb89))
- Robust from_julian_day decomposition (seconds-of-day rounding) ([b4fec9e](https://github.com/rahulmutt/pleiades/commit/b4fec9ecf2fe97d61fba5f1209d4dd1ed31bc338))
- Use range contains for bound checks in pleiades-time ([f2ecf3b](https://github.com/rahulmutt/pleiades/commit/f2ecf3ba54b25d788e1b58175f60821c2b4aa6be))
- Rescope time-scale summary so release output no longer self-contradicts on built-in civil-time ([dd19c7c](https://github.com/rahulmutt/pleiades/commit/dd19c7ce85f5a50b27fb712cffa71c389de21df1))
- Correct sign of perihelion-longitude T^2 term (Meeus 25.4) ([08f7614](https://github.com/rahulmutt/pleiades/commit/08f76140117c0384af8213d2ea3533db99c7189d))
- Apply ayanamsa to apparent longitude for sidereal charts; thread body_observer through light-time re-query ([7a170ae](https://github.com/rahulmutt/pleiades/commit/7a170aecff90631d96b95e11c586ea46e4e06ca7))
- Graceful mean fallback + light-time sanity guard for release-grade bodies whose apparent place is unavailable (e.g. 433-Eros) ([4335229](https://github.com/rahulmutt/pleiades/commit/4335229658ad39a85fdb97522ebc094e27561223))
- Report true on-sky diurnal aberration magnitude (cos dec weighting) ([9a23660](https://github.com/rahulmutt/pleiades/commit/9a236605e1aad58bc0d657a8cd6691078d929cf1))
- Apply topocentric before sidereal ayanamsa (single application, correct frame) ([d38e399](https://github.com/rahulmutt/pleiades/commit/d38e399f886e943977e9042e9186bb832534c3bf))
- Record diurnal parallax/aberration in apparent provenance when topocentric applied ([02b66d4](https://github.com/rahulmutt/pleiades/commit/02b66d4e4cc2795816f379ca4bd6dc785d962260))
- Correct Ascendant/Midheaven formulas to match Swiss Ephemeris ([cc879b5](https://github.com/rahulmutt/pleiades/commit/cc879b5df32b3454223dc86ecce3660ee06aae9a))
- Correct Placidus semi-arc solver to match Swiss Ephemeris (also fixes Topocentric) ([752d001](https://github.com/rahulmutt/pleiades/commit/752d001d7d4c79f91358306ac99a077fe43e5082))
- Correct Koch house cusps to match Swiss Ephemeris ([6efe7b6](https://github.com/rahulmutt/pleiades/commit/6efe7b62e08aa83f5569ccaadc67c3279f145f28))
- Correct Campanus house cusps to match Swiss Ephemeris ([1101dea](https://github.com/rahulmutt/pleiades/commit/1101deaa00043be62d089ab8ed3ff1c6c10583b1))
- Correct Alcabitius house cusps to match Swiss Ephemeris ([78f30f8](https://github.com/rahulmutt/pleiades/commit/78f30f8e672036ab0bf7e09b958a17d733385ed9))
- Give Morinus its own Swiss-Ephemeris-correct implementation ([44902b4](https://github.com/rahulmutt/pleiades/commit/44902b46b853e9c9db57594f9bf5caaf0c31537b))
- Apply nutation (apparent sidereal time + true obliquity) to match SE to arcsec ([68431be](https://github.com/rahulmutt/pleiades/commit/68431beb4d6d06eed70fe58893825dd9e8354bf5))
- Close catalog test gap, correct fallback doc; attest house cross-check ([ba574bc](https://github.com/rahulmutt/pleiades/commit/ba574bc2af848b5f327183a6b8e59fcd6a90e94d))
- SE-correct gated modes (precession drift + true-star fit) ([5939558](https://github.com/rahulmutt/pleiades/commit/593955849bba1ea8c9f3b5551ebc0ec106530766))
- Regenerate ayanamsa corpus as SE mean ayanamsa (NONUT|NOABERR) for cubic-fittable true-star ([0ba68b7](https://github.com/rahulmutt/pleiades/commit/0ba68b7508fb6520df0816c561de6519dfb6cadf))
- Refit true-star cubic to SE mean ayanamsa ([2f11f03](https://github.com/rahulmutt/pleiades/commit/2f11f037c4e3ba0bc128e9cd89b4f8a919871629))
- Fail closed for true-star sidereal offset outside the 1900-2100 fit window ([d0f344f](https://github.com/rahulmutt/pleiades/commit/d0f344fc02c4415cb26160f709eca31f637bab69))
- Compute Gauquelin sectors via Placidus semi-arc; promote to release-grade ([c5744d8](https://github.com/rahulmutt/pleiades/commit/c5744d8355edc2e76a0c31464e9ecd8868cc9740))
- Correct Horizon azimuth convention to match Swiss-Ephemeris; promote to release-grade ([83da51b](https://github.com/rahulmutt/pleiades/commit/83da51bc8741c269b483598c7f66cff1c81dc6fc))
- Add pleiades-apparent + pleiades-time to package-check crate list ([5958977](https://github.com/rahulmutt/pleiades/commit/595897796303c36380848ad8a769dbafe27dd577))
- Reject Gauquelin SE high-latitude fallback cleanly ([36a7e75](https://github.com/rahulmutt/pleiades/commit/36a7e75990958893ce85995221bf215143b1741b))
- Correct false SE-label text on 6 custom-definition Babylonian descriptors (slice 4) ([21bcf5d](https://github.com/rahulmutt/pleiades/commit/21bcf5dde26c0a028df594a810b37a6cb57c7c2e))
- Correct backwards panic msg in roster guard test; note Hebe outcome deviation in plan ([3e3b740](https://github.com/rahulmutt/pleiades/commit/3e3b740f77916e9374e388300075448ec393baff))
- Scrub residual sb441-n16 from regen tool error msg + doc (slice 2) ([bf69e55](https://github.com/rahulmutt/pleiades/commit/bf69e55b00d4e07a9c3c0c9b6dc67d53cb8cf80b))
- Apparent eclipsed_longitude + task-9 review minors ([e4b92d9](https://github.com/rahulmutt/pleiades/commit/e4b92d9538eb1b0b170de700d8b9c2ebb334271b))
- Extend supported window to cover all of 2100 CE ([73caf58](https://github.com/rahulmutt/pleiades/commit/73caf58f74d870a3c90f90bbf5e896f15681e047))
- Add missing year-1900 eclipses to NASA corpus ([0c52e88](https://github.com/rahulmutt/pleiades/commit/0c52e8801b7b23c61b9647c57454b332c3d5cf38))
- Correct corpus generator reproducibility (deps, kernel provenance, window bound) ([48d45dd](https://github.com/rahulmutt/pleiades/commit/48d45dd58ad535ee1fea0b95768aca0b5aeeb554))
- Correct apparent-Sun aberration and topocentric magnitude so validate-eclipses gate passes ([07e05c6](https://github.com/rahulmutt/pleiades/commit/07e05c63e2670f48e66c05350547a91593b49388))
- Harden validate-eclipses gate to zero-drift + allowlist; trim to data-bound window ([1cdbbf9](https://github.com/rahulmutt/pleiades/commit/1cdbbf9a92631c13d54da2f26e0f5de0ef10ffe8))
- Satisfy workspace publish audit for pleiades-eclipse ([304877c](https://github.com/rahulmutt/pleiades/commit/304877c79ff786dab435e798dbe46004bc7fd8f3))
- Clamp eclipses_in_range scan to the ephemeris data bound ([057de9e](https://github.com/rahulmutt/pleiades/commit/057de9e57b19823b32c3c1065cd6fa7a9cfc98f2))
- Apply Sun apparent aberration once (use apparent_sun_position) ([a611370](https://github.com/rahulmutt/pleiades/commit/a61137055bf4cc149eaa96fc24eab463a4c12e6c))
- Floor lilith gate validated-row count to preserve fail-closed integrity ([ed7e0c1](https://github.com/rahulmutt/pleiades/commit/ed7e0c187e7d4ac880be12e806386ccbd9dd224e))
- Degrade osculating apsis motion to None at coverage edges (post-review hardening) ([bb0d956](https://github.com/rahulmutt/pleiades/commit/bb0d95626ad998941173f27ebdea2c602a97761c))
- Reduce ICRF to J2000 ecliptic with fixed ε₀, not of-date obliquity (B1) ([375b54d](https://github.com/rahulmutt/pleiades/commit/375b54d3927ee640e8895e721f3bf7efde41f20d))
- Drop the of-date latitude band-aid; backend now emits true J2000 (B4) ([44fa7f6](https://github.com/rahulmutt/pleiades/commit/44fa7f68477efc2fe5e0e297b869e3ba9ba9be3c))
- Scope Pluto fallback policy summary to the algorithmic path ([bebe967](https://github.com/rahulmutt/pleiades/commit/bebe9677177b18d68812f3a0d02565bec0123f22))
- Cover apsides and eclipse in package-check ([f4301a6](https://github.com/rahulmutt/pleiades/commit/f4301a6ad25705665123c5535da0a03933e52d7f))

### Performance

- Optimize test/dev build profiles (opt-level=2) ([ce74b8f](https://github.com/rahulmutt/pleiades/commit/ce74b8f7d87f8f50726ee1a96e9a4bc6fe0fa25a))
## [0.2.0] - 2026-06-13

### Fixed

- Fix broken intra-doc links after modularization ([01cb291](https://github.com/rahulmutt/pleiades/commit/01cb2910cd8500fd38f82340ad7ddd885063e939))
## [0.1.0] - 2026-06-11

### Added

- Add optional serde support to core data crates ([14c093c](https://github.com/rahulmutt/pleiades/commit/14c093c74edf03415c6083072e891d728623453e))
- Separate validation reference points from gaps ([3805b61](https://github.com/rahulmutt/pleiades/commit/3805b6199504d06447b520cce2662e0b38dc4c40))
- Add compact release summary command ([23290ad](https://github.com/rahulmutt/pleiades/commit/23290ad187bd0df9c4ae63b866fc65fc4038617e))
- Bundle release summary artifact ([cae959f](https://github.com/rahulmutt/pleiades/commit/cae959f2d870cf8239ca1a9dc198a7b426cea1c0))
- Add artifact summary release view ([90eefb8](https://github.com/rahulmutt/pleiades/commit/90eefb8088781954d0552456e2ddc7898952a4d8))
- Broaden lunar reference evidence slice ([ea38f68](https://github.com/rahulmutt/pleiades/commit/ea38f68edf28623a13a734f282cad76769da10a1))
- Expose typed lunar source family in summaries ([bfd4f83](https://github.com/rahulmutt/pleiades/commit/bfd4f83dd90e9e93b93be1c8374ee2cf0ab3af8d))
- Add backend time-scale helper parity ([48099e2](https://github.com/rahulmutt/pleiades/commit/48099e25b31da589a6edbec15d0424de82773139))
- Harden packaged batch parity summaries ([135a5e1](https://github.com/rahulmutt/pleiades/commit/135a5e15bb080dd4ada19482bc91d67c49e8d3f0))
