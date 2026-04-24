# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Production Ephemeris Accuracy**.

The repository has a complete architectural foundation and broad planning/reporting scaffolding. The next material risk is correctness: the backends that drive chart outputs, validation comparisons, and future compressed artifacts must move from preliminary/sample behavior to source-backed astronomical implementations with measured accuracy.

## Why this is first

Several downstream requirements depend on trusted ephemeris outputs:

- compressed artifacts must be generated from validated source data;
- release compatibility profiles must not imply unsupported accuracy;
- chart APIs need deterministic body positions across supported bodies and time ranges;
- validation reports need real error envelopes rather than placeholder comparisons.

## Current repo state summary

Completed foundations:

- per-body comparison summaries and expected tolerance status sections in validation reports, so measured backend deltas are visible by body and checked against explicit interim evidence thresholds;
- managed Rust workspace and mandatory crate layout;
- typed domain vocabulary in `pleiades-types`;
- backend contract, metadata, errors, batch fallback, and composite routing in `pleiades-backend`;
- house and ayanamsa catalogs with descriptors and aliases;
- chart façade, sign/house/aspect summaries, sidereal conversion, and profile exports in `pleiades-core`;
- preliminary `pleiades-vsop87`, `pleiades-elp`, and `pleiades-data` crates, including finite-difference mean-motion estimates in the VSOP87 path and compact ELP Moon/lunar-point path, generated binary coefficient-table slices for the vendored IMCCE/CELMECH VSOP87B source files across the Sun-through-Neptune paths, per-body VSOP87 source profiles that distinguish generated-binary, vendored full-file source-backed, and fallback mean-element paths in validation reports, explicit geocentric-only rejection of direct topocentric requests in the VSOP87/ELP placeholder backends, plus a small JPL Horizons derivative-fixture backend with exact lookup, linear interpolation, and a denser hold-out corpus for interpolation transparency;
- compression model and sample artifact lookup;
- CLI, validation reports, backend matrix, release notes/checklists, bundle generation, and bundle verification;
- tests and doctests for current behavior.

Latest progress (2026-04-24): the JPL snapshot backend now exposes coarse leave-one-out interpolation quality samples derived from the checked-in fixture, and backend matrix release artifacts render those expanded public-input error checks. The JPL fixture itself also now includes an additional 2500000.0 TDB epoch across the comparison-body set, which widens the hold-out coverage while keeping the backend geocentric and pure Rust. The compact validation report summary also now carries a JPL interpolation-quality envelope, so the current leave-one-out evidence is visible alongside the broader comparison summaries. The VSOP87 backend matrix entry now also prints canonical J2000 source-backed evidence for Sun-through-Neptune at the same public IMCCE reference points used by the regression tests, so the vendored Earth-through-Neptune source-file paths now carry exact evidence from public IMCCE VSOP87B inputs while Pluto remains the remaining mean-element fallback. Compact validation summaries now also surface the same source-backed VSOP87 evidence line, now include a concise source-documentation count for the structured VSOP87 body profiles, and now add a compact body-profile evidence count so the release-facing summaries carry measured-error and provenance snapshots without duplicating the full backend matrix detail. The validation report and backend matrix summaries now also distinguish the generated-binary VSOP87 source-backed body paths from the Pluto mean-element fallback, making the current source state clearer without changing the public backend contract. The backend matrix output itself now pairs each source-backed body profile with its source file, provenance, and measured canonical deltas so the validation reports make the body/profile linkage explicit. Validation-side compatibility-profile tests now also pin several recent release-profile entries — including the Equal (MC) and Equal (1=Aries) house-table forms, Pullen SR, True Citra Paksha, P.V.R. Narasimha Rao, and B. V. Raman — so the release-facing catalog text stays anchored to the currently published breadth. The compatibility-profile command now also pins additional release-profile spellings such as Equal table of houses, Whole Sign system, Morinus house system, Galactic Equator (Fiorenza), Valens Moon, and Babylonian (House Obs), extending the release-facing breadth checks without changing the verification model. The VSOP87 source documentation is now also structured enough for release reports to show variant, frame, unit, reduction, and date-range notes per source-backed body, which gives the future generated-table work a machine-readable documentation hook rather than just freeform provenance text. The VSOP87 source-backed body catalog is now centralized in one internal table, and the public profile/spec/sample accessors derive from that single source of truth so future generated-table ingestion has one shared catalog to extend. UTC-tagged chart and instant inputs now also have explicit caller-supplied TT conversion helpers, closing the remaining UTC convenience gap in the current time-scale policy surface without adding built-in leap-second or Delta T modeling.

Previous progress (2026-04-24): backend matrix release artifacts now report implementation status separately from body/catalog presence. Each implemented backend has an explicit status label and note (fixture reference, partial source-backed, preliminary algorithm, prototype artifact, or routing façade), and the compact matrix summary counts those statuses so release artifacts do not imply production accuracy merely because a backend advertises body coverage.

Recent test progress (2026-04-24): the VSOP87 backend now also exercises its batch API over the full source-backed Sun-through-Neptune sample set, so the canonical J2000 evidence is verified both body-by-body and in batch form.

Active gaps:

- documented VSOP87 regeneration tooling and release-grade error envelopes;
- production lunar theory implementation and lunar point semantics;
- larger JPL-style reference corpus, interpolation validation, and documented tolerance envelopes beyond the current small fixture proof of concept;
- production Delta T conversion, TDB handling, apparent-place corrections, and validated frame-conversion error envelopes beyond the initial documented policy;
- source-backed evidence tables, reference-backed tolerance tables, and broader validation reports.

## Recommended next slice

Continue expanding the `pleiades-vsop87` source-data increment:

1. move the next phase-1 accuracy slice toward the lunar theory source-selection work, now that the source-backed VSOP87 generated-table path covers all current major planets;
2. preserve Pluto as an explicitly documented non-VSOP87 special case until a Pluto-specific source path is selected;
3. schedule a smaller follow-up slice for a documented VSOP87 regeneration tool and release-grade error-envelope audit if the release process needs it.

Small preparatory backend-semantics increments have landed: supported planetary results now include deterministic central-difference mean-motion estimates, the compact ELP Moon/node path now exposes matching finite-difference mean-motion estimates, mean lunar apogee and mean lunar perigee are now explicitly supported as mean-only lunar points with finite-difference longitude speeds, direct observer-bearing requests to the geocentric VSOP87/ELP placeholders now fail explicitly instead of implying topocentric support, unsupported apparent requests are rejected rather than silently returning mean values, the initial time/observer policy is documented, caller-supplied UT1-to-TT/scale-offset helpers now exist in the shared type layer without introducing a built-in Delta T model, and `pleiades-core::ChartRequest` now includes explicit caller-supplied time-scale offset conveniences so chart assembly can adopt a chosen conversion policy before houses or body positions are queried. The Sun through Neptune paths now use public IMCCE VSOP87B source files tested against full-file J2000 golden values, and the Earth, Mercury, Venus, Mars, Jupiter, Saturn, Uranus, and Neptune paths now have generated binary coefficient-table slices derived from vendored source files. The validation backend matrix now surfaces per-body VSOP87 source profiles. A parallel release-safety increment tightened compatibility-profile verification so descriptor metadata and repeated exact labels are checked before bundle publication. Documented VSOP87 regeneration tooling and release-grade error envelopes remain outstanding; Pluto remains a special case outside VSOP87's major-planet files; true lunar apogee/perigee remain deferred until a source-backed true-point model is selected. The VSOP87 crate now also publishes a deterministic source-audit manifest with byte counts, line counts, parsed term counts, and 64-bit fingerprints for each vendored full-file input, giving the eventual generated-table path a reproducibility hook without changing runtime behavior.

## Constraints

- Keep all implementation pure Rust.
- Do not move astrology-domain logic into source-specific backends.
- Do not expand release claims until validation evidence exists.
- Keep changes small enough to review and test independently.
