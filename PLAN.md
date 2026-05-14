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
- `pleiades-jpl` provides checked-in JPL Horizons snapshots and hold-out fixtures with provenance and validation helpers, not a broad production source reader.
- `pleiades-data` provides a deterministic stage-5 draft artifact with manifest, checksum, output-profile, body-class cadence, fit-outlier, lookup, batch-lookup, and decode reports, but its model error envelope is far outside production thresholds.
- Current first-party request policy is explicit: TT/TDB mean geometric geocentric tropical requests are supported where metadata says so; built-in Delta T/UTC modeling, apparent-place corrections, topocentric body positions, and native sidereal backend output remain unsupported unless a future backend advertises and validates them.

## Remaining specification gaps

1. **Production compressed ephemeris** — the artifact format and reporting exist, but the bundled 1500-2500 CE data product remains draft-grade and fails production accuracy expectations.
2. **Production reference inputs** — checked-in snapshots are valuable regression and comparison evidence, but the project still needs a production-suitable public source ingestion or generated-corpus path for artifact fitting and body claims.
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

The execution frontier is Phase 1, with Phase 2 as its main dependency. The next implementation slice should reduce the packaged-data error envelope using a trusted source corpus, not add more report surfaces around the current draft fixture.

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

Status: Updated 2026-05-14 after reviewing `SPEC.md`, `spec/*.md`, README status, CLI/report posture, and current plan documents.
