# Pleiades Development Plan

This is the active forward plan for `pleiades`. It intentionally omits completed bootstrap, catalog-scaffolding, report-alias, release-rehearsal, fixture-promotion, manifest-completeness, and benchmark-surface work. Those details remain in git history and generated reports, not in the active task list.

## Source material

This plan is derived from:

- [SPEC.md](SPEC.md)
- [spec/vision-and-scope.md](spec/vision-and-scope.md)
- [spec/requirements.md](spec/requirements.md)
- [spec/architecture.md](spec/architecture.md)
- [spec/backend-trait.md](spec/backend-trait.md)
- [spec/astrology-domain.md](spec/astrology-domain.md)
- [spec/data-compression.md](spec/data-compression.md)
- [spec/backends.md](spec/backends.md)
- [spec/api-and-ergonomics.md](spec/api-and-ergonomics.md)
- [spec/validation-and-testing.md](spec/validation-and-testing.md)
- [spec/roadmap.md](spec/roadmap.md)

## Current implementation baseline

The repository is past the original foundation roadmap. The current workspace includes all mandatory first-party crates, pure-Rust tooling checks, typed shared domain models, backend traits, capability metadata, batch helpers, routing/composite helpers, chart façade APIs, broad house and ayanamsa catalogs, compatibility-profile reporting, source/provenance summaries, validation tooling, release-bundle rehearsal, and a checked-in draft packaged-data artifact.

Current backend and data posture:

- `pleiades-vsop87` has source-backed VSOP87B tables for Sun through Neptune and keeps Pluto as approximate fallback evidence, not a full release-grade source-backed claim.
- `pleiades-elp` provides a compact Meeus-style lunar/lunar-point baseline with published-example evidence, not a full ELP coefficient implementation.
- `pleiades-jpl` provides checked-in JPL Horizons snapshots and hold-out fixtures with provenance and validation helpers; its release-facing source posture is now documented as a hybrid fixture corpus, not a broad production source reader, the reference snapshot source summary now carries redistribution posture directly, the reference and hold-out snapshot manifests now carry redistribution posture comments, the release-facing source summary now carries explicit per-fixture checksum revision markers, the hold-out source summary now surfaces redistribution posture directly in release-facing reports, and the interpolation-quality report now includes body-class error envelopes for the available luminary and major-planet slices.
- `pleiades-data` provides a deterministic stage-5 draft artifact with manifest, checksum, output-profile, body-class cadence, fit-outlier, lookup, batch-lookup, and decode reports; the current draft path now passes the calibrated fit thresholds, prefers the lower measured-fit candidate-versus-fallback reconstruction on span-limited windows, can add residual-correction channels across the bundled body set when they improve the measured fit, now explores residual-channel combinations rather than only greedy single-channel additions, now uses channel-specific residual sample lattices so dense angular bodies can keep the denser lattice while distance residuals stay on the sparse table, uses a denser residual lattice for luminaries and custom bodies, now applies body-specific validation and outlier lattices for luminaries, selected asteroids, Pluto, and custom bodies, now gives those sensitive bodies a denser fit-candidate lattice before the legacy fallback ladder, uses a denser report lattice for fit-outlier summaries, the packaged-artifact fixture has been resynced from the current regeneration path and refreshed to the current bytes, the regeneration helper now caches the deterministic rebuilt artifact in-process so repeated validation and report invocations avoid rebuilding the full artifact, the normalized-intermediate summary now carries a deterministic checksum, the release bundle now includes the release-house-validation summary and the production-generation boundary source summary in its manifest and verification, the house-code alias inventory plus the house and ayanamsa descriptor summary surfaces now have validated wrappers, the ayanamsa catalog validation summary now routes through a validated wrapper in release-facing report surfaces, the catalog inventory summary now also validates the release-profile identifiers before rendering in release-facing report surfaces, the comparison-corpus release-grade guard summary now validates before rendering in comparison/report surfaces, the body-class comparison-envelope formatter now validates before rendering, the production-generation source summary now records the generation command alongside the checked-in CSV provenance and checksums and now has a typed validated wrapper, the frame-policy summary surface now validates before rendering in CLI and validation report paths, the selected-asteroid coverage line now routes through a validated summary helper shared by backend-matrix and reference-asteroid report surfaces, the request-policy/request-semantics report titles now share a guarded validation helper before emitting release-facing text, the JPL snapshot evidence classification and source-posture report lines now also route through validated summary helpers before release rendering, the packaged-artifact fit sample classes summary now validates its boundary continuity component before rendering, the packaged-artifact phase-2 corpus alignment summary now carries source/provenance lines for the reference, comparison, hold-out, and production-generation corpora alongside the body-class coverage evidence and now also keeps the selected-asteroid source evidence/windows in the same phase-2 posture, the backend layer now exposes validated Pluto fallback and release-grade body-claims summary-line helpers so claim-boundary checks can be reused by release-facing surfaces, and the release-facing target-threshold posture now uses a typed production-ready state with explicit phase-2 reference/comparison/hold-out corpus alignment evidence while the broader source corpus alignment remains open, and the release-bundle verifier now cross-checks the Pluto fallback summary against the release-grade body claims posture before accepting a staged bundle, and the artifact/validation summaries now surface the packaged-artifact phase-2 corpus alignment line explicitly alongside the target-threshold posture.
- Packaged artifact profiles now fail closed when any built-in output would remain unlisted, keeping stored/derived/unsupported capability classifications explicit.
- The packaged-artifact generator now uses midpoint-aware quadratic distance reconstruction on pair-based spans, the generation-policy wording now matches its quadratic-window strategy, and the checked-in fixture has been regenerated to the new bytes.
- Current first-party request policy is explicit: TT/TDB mean geometric geocentric tropical requests are supported where metadata says so; built-in Delta T/UTC modeling, apparent-place corrections, topocentric body positions, and native sidereal backend output remain unsupported unless a future backend advertises and validates them.
- The release notes and release summary surfaces now route the compatibility profile's release note through a validated helper before rendering, so those release-facing entries fail closed on profile summary drift.

## Remaining specification gaps

1. **Production compressed ephemeris** — the artifact format and reporting exist, but the bundled 1500-2500 CE data product remains draft-grade and fails production accuracy expectations.
2. **Production reference inputs** — checked-in snapshots are valuable regression and comparison evidence, but the project still needs broader production-suitable corpus coverage for artifact fitting and body claims.
3. **Release-grade body coverage** — Pluto, full lunar theory, lunar points beyond the current compact baseline, and selected asteroids need source-backed validation or constrained/excluded release status.
4. **Advanced request implementation choices** — UTC/Delta-T convenience, apparent corrections, topocentric body positions, and native sidereal backend output must either be implemented with evidence or remain consistently rejected and documented.
5. **Compatibility evidence** — broad house and ayanamsa catalogs need continued formula/provenance/reference audits before entries are promoted beyond descriptor, constrained, custom, or approximate status.
6. **Fail-closed release gates** — production releases must block stale profiles, native-dependency drift, artifact threshold failures, inaccurate backend claims, and unreproducible bundles.

## Active implementation phases

| Phase | Focus | Workable-state promise | Detailed doc |
| --- | --- | --- | --- |
| 1 | Artifact accuracy and packaged-data production | Maintainers can regenerate and ship a deterministic 1500-2500 CE artifact whose measured errors are within the published profile | [plan/stages/01-production-compressed-data.md](plan/stages/01-production-compressed-data.md) |
| 2 | Reference/source corpus productionization | Maintainers have documented public inputs broad enough for release body claims, backend validation, and artifact fitting | [plan/stages/02-production-reference-inputs.md](plan/stages/02-production-reference-inputs.md) |
| 3 | Body-model completion and claim boundaries | Pluto, lunar theory/lunar points, and selected asteroid claims are either source-backed or explicitly constrained/excluded | [plan/stages/03-body-coverage-and-claims.md](plan/stages/03-body-coverage-and-claims.md) |
| 4 | Advanced request modes and policy | UTC/Delta-T, apparent, topocentric, and native-sidereal behavior is implemented with evidence or consistently rejected | [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md) |
| 5 | Compatibility catalog evidence | Release profiles truthfully classify house and ayanamsa built-ins, aliases, constraints, custom entries, and known gaps | [plan/stages/05-compatibility-catalog-evidence.md](plan/stages/05-compatibility-catalog-evidence.md) |
| 6 | Release gate hardening | A clean checkout can produce verified release artifacts whose claims match current generated evidence | [plan/stages/06-release-gate-hardening.md](plan/stages/06-release-gate-hardening.md) |

## Current priority

The execution frontier is Phase 1, with Phase 2 as its main dependency. The target-threshold policy is now recorded as production-ready with explicit phase-2 corpus alignment evidence, and the next implementation slice should keep the measured-fit path aligned with a trusted source corpus rather than add more report surfaces around the current draft fixture; the regeneration helper now writes regenerated bytes so the checked-in fixture can be refreshed from current code, sensitive bodies now get a denser fit-candidate lattice before the legacy fallback ladder, the dense lattice now includes 10-point, 12-point, 14-point, and 16-point Chebyshev-Lobatto options for luminaries, Pluto, selected asteroids, and custom bodies, the generator now also tries an 8-point Chebyshev-Lobatto baseline candidate for every body before that body-specific dense ladder, and the generator now selects the best dense candidate before falling back, and the release-bundle verification display now includes the packaged-artifact phase-2 corpus alignment summary alongside the other staged release artifacts.

For live execution guidance, see:

- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)

## Detailed plan index

- [plan/overview.md](plan/overview.md)
- [plan/stages/01-production-compressed-data.md](plan/stages/01-production-compressed-data.md)
- [plan/stages/02-production-reference-inputs.md](plan/stages/02-production-reference-inputs.md)
- [plan/stages/03-body-coverage-and-claims.md](plan/stages/03-body-coverage-and-claims.md)
- [plan/stages/04-advanced-request-modes.md](plan/stages/04-advanced-request-modes.md)
- [plan/stages/05-compatibility-catalog-evidence.md](plan/stages/05-compatibility-catalog-evidence.md)
- [plan/stages/06-release-gate-hardening.md](plan/stages/06-release-gate-hardening.md)
- [plan/status/01-current-execution-frontier.md](plan/status/01-current-execution-frontier.md)
- [plan/status/02-next-slice-candidates.md](plan/status/02-next-slice-candidates.md)
- [plan/tracks/01-workspace-and-tooling.md](plan/tracks/01-workspace-and-tooling.md)
- [plan/tracks/02-domain-and-public-api.md](plan/tracks/02-domain-and-public-api.md)
- [plan/tracks/03-backends-and-distribution.md](plan/tracks/03-backends-and-distribution.md)
- [plan/tracks/04-validation-and-release.md](plan/tracks/04-validation-and-release.md)
- [plan/checklists/01-phase-gates.md](plan/checklists/01-phase-gates.md)
- [plan/checklists/02-release-artifacts.md](plan/checklists/02-release-artifacts.md)
- [plan/appendices/01-phase-to-spec-map.md](plan/appendices/01-phase-to-spec-map.md)
- [plan/appendices/02-phase-workable-state-matrix.md](plan/appendices/02-phase-workable-state-matrix.md)

## Plan maintenance rules

When implementation closes a gap, remove it from the active phase/status docs and update the phase map. Do not keep completed report aliases, already-landed summaries, or historical phase notes as future work.

Status: Updated 2026-05-17 after reviewing `SPEC.md`, `spec/*.md`, README status, CLI/report posture, and current plan documents; descriptor-level validated summary wrappers were added for the house and ayanamsa catalogs, the ayanamsa catalog validation summary now has a validated wrapper in release-facing report surfaces, the comparison-corpus release-grade guard summary now validates before rendering, the compatibility-caveats, compatibility-profile, release-notes, and release-summary summary surfaces now use validated house-system, house-formula-family, and ayanamsa wrappers, the request-policy/request-semantics report titles now share a guarded validation helper, the release-specific house-system and ayanamsa canonical-name summary commands now route through explicit validated helpers, the release bundle now stages and verifies the production-generation boundary source summary alongside the existing source evidence files, the comparison snapshot source summary now also stages and verifies in the release bundle, the production-generation source summary now has a typed validated wrapper around the merged provenance block, the packaged-artifact phase-2 corpus alignment now has a standalone validated CLI surface with a faster sidecar-check regression and a public typed accessor for validation/report reuse, the packaged-artifact fit envelope now follows the body-specific dense validation cadence for sensitive body classes, the packaged-artifact fit sample classes summary now validates its boundary continuity component before rendering, the body-class span-cap summary now validates before rendering in CLI and validation-report paths, the validation report now renders archived regression findings through validated summary helpers, the backend layer now exposes validated Pluto fallback and release-grade body-claims summary-line helpers for the claim-boundary posture, and the packaged-artifact regeneration helper now caches the deterministic rebuilt artifact in-process so repeated validation and report invocations avoid rebuilding the full artifact; the packaged-artifact phase-2 corpus alignment summary now also carries the production-generation source provenance and body-class coverage so the threshold posture explicitly tracks the separate generation-input corpus.
