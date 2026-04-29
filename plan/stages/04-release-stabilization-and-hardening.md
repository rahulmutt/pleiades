# Phase 4 — Release Stabilization and Hardening

## Purpose

Prepare Pleiades for a release whose compatibility, accuracy, artifact, and API claims are reproducible and auditable. This phase packages the evidence produced by earlier phases and closes gaps in documentation, CI, and user-facing ergonomics.

## Spec drivers

- `SPEC.md`: acceptance summary and normative decisions
- `spec/requirements.md`: NFR-1 through NFR-6 and release compatibility profiles
- `spec/api-and-ergonomics.md`: public documentation, examples, deterministic behavior
- `spec/validation-and-testing.md`: release gates, reports, benchmarking, audits
- `spec/backends.md`: capability matrices

## Current baseline

The workspace already has CLI/report commands, compatibility-profile summaries, release notes/checklists, release-bundle generation and verification, workspace audit tests, and broad unit-test coverage.

## Remaining implementation goals

1. Make release profiles authoritative.
   - Version profile identifiers and archive the exact profile shipped with each release.
   - Include supported bodies, house systems, ayanamsas, aliases, backend matrices, time ranges, constraints, and known gaps.
   - Require profile verification in CI or release scripts.
   - Progress note: the compact release summary now also mirrors the release-specific house-system and ayanamsa canonical-name breakdowns from the compatibility profile, so the one-screen release posture now shows the explicit name lists alongside the aggregate catalog counts. The release bundle now also archives a dedicated `release-profile-identifiers.txt` artifact and verifies its checksum, which keeps the shipped compatibility/API profile pair explicit in the staged release bundle instead of only inside the summary files. The release-bundle verifier now also cross-checks the profile and API posture identifiers embedded in the release notes summary, release summary, release checklist, and dedicated release-profile-identifiers artifact against the manifest-backed identifiers, so the staged human-readable release artifacts stay internally consistent in addition to being checksummed. The compatibility-profile verification path is now also backed by a structured summary record, which keeps the release-facing catalog audit typed and reusable instead of rebuilding the verification line in each caller. The shared release-profile identifier pair now also has a backend-owned `summary_line()` helper, and the validation/release report renderers reuse that typed line directly so the staged release profile wording stays centralized alongside the existing compatibility/profile identifiers. The release-profile identifier helper now also rejects a self-colliding compatibility/API pair, and the compact identifiers line now carries an explicit `v1` schema prefix, which keeps the staged profile ids versioned as well as distinct before the bundle cross-checks compare them against the current release posture.

2. Harden validation and benchmark reports.
   - Generate reports from real backend and artifact evidence.
   - Include accuracy tables, regression summaries, benchmark methodology, and environment metadata.
   - Archive reports with release artifacts and checksums.
   - Progress note: the benchmark command and validation benchmark summaries now surface workspace provenance alongside the timing numbers, so the release-facing benchmark output records source revision, workspace status, and rustc version in addition to the measured latencies and throughput. The benchmark provenance line is now also owned by a typed summary helper in `pleiades-validate`, which keeps the compact report formatting co-located with the provenance record instead of rebuilding the same three-line block ad hoc. The dedicated benchmark report now also spells out the benchmark methodology for the backend and chart workloads, so the rounds, samples-per-round, and workload split are visible in the CLI benchmark output instead of only being implied by the surrounding section labels. The release bundle now also archives that benchmark report as `benchmark-report.txt` with its own checksum, so the staged release artifacts carry the dedicated benchmark evidence alongside the validation report instead of only embedding it indirectly. The elapsed timing fields in the benchmark, artifact, and chart reports now render with explicit second units instead of `Debug`-style `Duration` output, keeping the release-facing timing lines unit-stable across the detailed and compact summaries. The backend and chart benchmark constructors now also reject zero-round or zero-sample placeholder runs after preflighting backend metadata, so the benchmark-report path fails closed instead of formatting a placeholder workload. Latest progress (2026-04-28): the backend and chart benchmark reports now also expose compact typed summary lines, and the detailed report output reuses those summary helpers so the benchmark posture now has a stable one-line inventory in addition to the longer methodology breakdown. Latest progress (2026-04-29): the validation report now preflights its aggregate corpus, benchmark, chart, and regression archives before rendering, and the full report/summary paths fail closed if a drifted benchmark corpus or archive would otherwise have produced stale release text.

3. Improve public API documentation.
   - Add rustdoc examples for common chart workflows, backend selection, sidereal conversion, houses, and packaged data.
   - Document units, frames, time scales, Delta T policy, normalization, and failure modes.
   - Clarify unstable or experimental APIs before publishing semver promises.
   - Progress note: the core chart request docs now include a worked explicit UTC-to-TDB chart-assembly example, and the low-level time-scale helpers now also include doctested caller-supplied conversion examples, so the public API examples now exercise the explicit conversion policy rather than relying only on prose. The lower-level backend request docs now also show the default tropical/ecliptic/mean-geometric request shape explicitly, which keeps the bare-request semantics visible alongside the higher-level chart workflow example. The `ChartEngine::chart` docs now also include a worked snapshot example that shows UTC-to-TT staging, an explicit house observer, and a geocentric Sun placement, which makes the common chart workflow easier to follow from the façade entry point. The `ChartRequest::summary_line` and `ChartSnapshot::summary_line` docs now also carry worked observer-policy examples, so the report-facing chart summaries now spell out the geocentric-versus-house-only boundary directly instead of leaving it implicit. The `MotionSummary` docs now also have a worked example and a direct ordering regression, and `AspectSummary` now has the same doctested summary-line pattern plus a crate-root export, which keeps the major-aspect wording discoverable alongside the other public chart summaries. The packaged-data crate now also has doctested examples for loading the checked-in artifact from bytes and from a temporary file path, and the sidereal conversion helper now includes a worked doctest example, so the packaged-data and sidereal conversion workflows are covered in the public API docs too. The `sidereal_longitude` helper now also has a dedicated worked example that shows a Lahiri conversion landing in Pisces, and the packaged backend selection example now checks the backend id and supported frames directly so the backend-selection narrative is easier to follow from the crate docs. The remaining signed `tt_from_tdb_signed`, `tdb_from_ut1_signed`, and `tdb_from_utc_signed` helpers now also have doctested examples, which keeps the caller-supplied time-scale policy visible all the way down to the shared instant layer and exercises the signed alias variants explicitly in doc tests. The release-profile identifier helper now also has rustdoc examples in `pleiades-core`, which makes the versioned compatibility/API pair visible from the façade docs as well as the release-bundle reports. The house lookup docs now also include doctested wraparound and exact-boundary examples for the shared cusp helper plus a chart-facing `house_for_body` example, which makes the house-workflow portion of the public API a little easier to scan and cross-check. The ayanamsa metadata coverage helper now also has a doctested example for the compact reference-epoch/offset inventory line, so that release-facing catalog posture is discoverable from the crate docs too. The packaged-data policy helpers now also have doctested examples for the lookup-epoch posture, request-policy posture, and artifact-access posture, which keeps the release-facing packaged-data policy surface discoverable from the public crate docs as well as from the validation reports.

4. Strengthen CI and audit gates.
   - Run formatting, clippy, tests, doc tests, compatibility-profile verification, artifact validation, release-bundle verification, and native-dependency audits.
   - Reject mandatory C/C++ build hooks or FFI dependencies in first-party crates.
   - Keep tools declared in `mise.toml` unless they genuinely require `devenv.nix`.
   - Progress note: the compact workspace-audit summary now also reuses a typed summary record with rule counts and clean/dirty status, which keeps the pure-Rust release gate's compact output aligned with the detailed audit report instead of reconstructing the summary text ad hoc. The workspace audit now also flags direct `-sys` dependency names and renamed `package = "...-sys"` declarations in manifest dependency tables, which tightens the native-dependency gate before the lockfile is consulted. The workspace audit summary now also validates its own aggregated rule counts before rendering, so the compact release gate now fails closed if the summarized violation data ever drifts from the detailed audit report. The workspace CI task now also runs a rustdoc build with warnings denied, which keeps the public API documentation checks in the release gate alongside the existing formatting, lint, test, audit, and release-smoke steps. The latest doc-gate run now passes after fixing the broken intra-doc links in `pleiades-backend` and `pleiades-core`, so the rustdoc warning policy is green again rather than blocked on stale link syntax.

5. Finalize release artifacts.
   - Generate signed or checksummed bundles containing source revision, profile identifiers, validation reports, backend matrix, artifact summaries, and release notes.
   - Verify bundles from a clean checkout.
   - Document how downstream users reproduce environment, tests, validation, and artifact generation.
   - Progress note: the release-summary, validation-summary, and bundle-release CLI paths now use a minimal nonzero validation-report sample instead of a zero-round placeholder, so the compact comparison-tolerance policy and release-bundle outputs render again instead of failing on empty benchmark datasets. The public `ReleaseBundle` metadata now also validates the expected artifact paths, required provenance fields, and nonzero validation-round count before the verified bundle object is returned, so the staged release-bundle surface now fails closed even if a caller mutates the in-memory bundle after verification.

## Done criteria

- Release bundle verification passes from a clean checkout.
- Published compatibility claims match generated profiles and backend metadata.
- Validation and benchmark reports use production backends/artifacts, not placeholder data.
- Public docs include examples and failure-mode documentation for main workflows.
- CI gates enforce pure-Rust, formatting, linting, tests, profile verification, and artifact/report checks.

## Follow-on work after the first hardened release

- Expand optional backend families such as Moshier, Kepler, asteroid-specific, or composite crates.
- Increase asteroid and derived-point coverage beyond the baseline.
- Add optional higher-level chart utilities only after core accuracy and compatibility remain stable.
