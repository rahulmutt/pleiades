# Status 2 — Next Slice Candidates

This file lists focused implementation slices that map to the current phase ladder. It intentionally omits completed report-surface, alias, fixture-summary, and release-rehearsal cleanup work.

## Phase 1 candidates — Reference accuracy and request semantics

### 1. Representative 1500-2500 reference expansion

- The 1500-01-01, 1600-01-11, and 1900-01-01 selected-body boundary slices are now also surfaced through the top-level reference snapshot summary, so this early-boundary sub-slice is complete.
- The lunar boundary evidence summary is now also surfaced through the top-level reference snapshot summary, and the major-body boundary-window aggregate is now surfaced there as well, so the next breadth slice can focus on another boundary epoch or a representative interior comparison row.
- A 2500-01-01 selected-body boundary slice for Mars, Mercury, Moon, Sun, and Venus is now checked in and now has direct CLI parity in `pleiades-cli`.
- A 1749 major-body boundary slice is now checked in and surfaced through the top-level reference snapshot summary.
- A 1750-01-01 interior boundary slice for Sun through Neptune is now checked in and now has a first-class 1750 major-body interior report surface.
- A dedicated 1800-01-03 major-body boundary slice is now checked in and now surfaces through the top-level reference snapshot summary.
- A 2451916.0 interior reference slice is now checked in and surfaced through a first-class report surface with a direct CLI alias.
- A 2451920.5 interior reference slice is now checked in, and the validation CLI now explicitly regression-tests it inside the combined reference snapshot summary.
- The 2451915.25/2451915.75 high-curvature hold-out window is now surfaced through the combined JPL evidence report, and the selected-asteroid boundary, bridge, and dense summaries are now surfaced there too, while the 2451910 through 2451915 major-body boundary summaries and the 2451916.0 interior / 2451916.5 dense-boundary slices are now also explicitly surfaced there; keep hold-out rows separate from fitting/reference rows while the next slice targets any remaining boundary breadth.
- A 2453000.5 major-body boundary summary is now checked in and surfaced through the top-level reference snapshot summary.
- A 2500 major-body boundary summary is now checked in and surfaced through the top-level reference snapshot summary.
- The Mars outer-boundary summary is now checked in and surfaced through the top-level reference snapshot summary alongside the late corpus edge at JD 2634167.0.
- A 2400000.0 major-body boundary summary is now checked in and surfaced through a first-class report surface, and the top-level reference snapshot regression now explicitly anchors that slice alongside the 2451545.0 J2000 boundary.
- A 2451545.0 major-body boundary summary for the J2000 major-body reference slice is now first-class with direct CLI/report aliases and top-level reference-snapshot regression coverage.
- The exact-J2000 evidence summary is now also surfaced through the top-level reference snapshot summary, so the next breadth slice can focus on another boundary epoch or representative interior row if more reference breadth is still needed.
- Dedicated 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 major-body boundary report slices are now first-class and validated, with direct CLI/report surfaces now exposed for 2451910.5, 2451911.5, 2451912.5, 2451913.5, and 2451915.5, plus the 2451917.0 bridge slice, the 2451918.5 Mars/Jupiter boundary slice and its epoch-specific 2451918 alias now has backend drift regression coverage, the 2451916.5 dense boundary day is now also promoted through the top-level reference snapshot summary and now has an epoch-specific 2451916 major-body dense-boundary alias, the CLI parity layer now mirrors the 2451912, 2451913, 2451914, 2451917, 2451918 boundary aliases plus the 2451914 pre-bridge, 2451914 bridge-day, and bridge-day aliases, the 2451915 bridge alias, the 2451917 bridge alias, and the 2451916 dense-boundary alias, while the JPL backend API now also exposes epoch-specific aliases for the 2451914 pre-bridge, 2451914 bridge, and 2451915 bridge report surfaces, and the top-level reference snapshot summary now surfaces the 1749, 1750-01-01, 1800-01-03, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451915.5, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5 slices alongside the earlier boundary summaries, plus the 2451914 pre-bridge and bridge-day report surfaces. The reference-snapshot summary regression now also explicitly anchors the 2451912, 2451913, 2451914, 2451916 dense, and 2451917 bridge slices. This alias-parity slice is now complete in the backend API, CLI, and validation front ends.
- The validation-report and release-summary layers now also surface the 2451918 boundary slice explicitly.
- The comparison-corpus guard now reflects the current 2451913.5 boundary-day coverage, so the next reference-breadth slice can target another boundary or representative interior epoch if breadth still needs to grow; the 2451914 pre-bridge boundary day now has an epoch-specific CLI alias, the 2451914 bridge day now has its own epoch-specific CLI alias, the 2451914 major-body bridge-day alias is now also exposed for naming parity and surfaced in the top-level reference snapshot summary, the 2451915 major-body boundary day now has a first-class report surface, the 2451917.0 bridge day now has its own epoch-specific CLI alias, the 2451918 Mars/Jupiter boundary day now has its epoch-specific CLI alias, and the 2451915 major-body bridge day retains its epoch-specific CLI alias, and the 2451915 bridge slice is now explicitly regression-anchored in the top-level summary and validation report, the boundary-window / boundary-epoch-coverage aggregate report surfaces now carry direct regression coverage, and the independent hold-out high-curvature window is now included in the combined evidence report without being folded into reference rows; the remaining breadth work can now move to a different epoch, a new source-backed comparison slice, or a concise request-policy cleanup if no further corpus row is needed.
- Keep hold-out rows separate from fitting/reference rows.
- The release-grade body-claims posture is now explicit in typed backend/core summaries and release reporting, so keep validation work focused on keeping per-class tolerances, claim status, and any newly advertised bodies aligned with that claim set.
- Update validation reports to classify evidence as release-tolerance, hold-out, fixture exactness, or provenance-only; the comparison audit surface now mirrors body-class tolerance posture for the current release-grade corpus.

### 3. Lunar source posture decision

- Decide compact lunar baseline versus fuller ELP-style coefficient implementation for the first release.
- If expanding to coefficient data, add pure-Rust ingestion/evaluation, provenance, validation, and tests.

### 4. Request/time semantics closure

- The first-release request-policy posture is now explicit: built-in Delta T and UTC/UT1 convenience conversion remain deferred, validation-report summaries surface that deferral explicitly, the backend/core façades re-export the UTC-convenience, Delta T, and native sidereal policy summaries plus their current constructors, and the request-surface inventory lists Delta T as a separate report entrypoint. Pluto fallback posture is now also surfaced through a shared typed backend/core summary and validation report line, so keep future cleanup focused on the remaining request/time gaps instead of restating the Pluto fallback prose.
- Apparent-place corrections and topocentric body positions remain explicitly deferred unless a later backend-capability decision changes that posture.
- Native sidereal backend output remains deferred unless a backend advertises equivalent support through capabilities.
- If no request-policy decision changes, prefer the next slice to expand reference breadth (for example by promoting another checked-in interior/boundary epoch to a first-class report surface) before revisiting more request-policy wording; the `request-semantics` help-text cleanup now keeps the alias wording aligned, so any further cleanup here should stay limited to report/help phrasing.

## Phase 2 candidates — Production compressed artifacts

### 1. Production artifact profile manifest

- Specify body set, date range, channels, derived outputs, unsupported outputs, speed policy, and thresholds; lookup-epoch policy and speed policy are now explicit in the production-profile draft and generator manifest, and the release bundle now carries the packaged lookup-epoch policy summary/checksum alongside the production-profile and target-threshold bundle outputs.
- Add validation that fails on profile/threshold drift; the production-profile and generator-parameter summaries now also fail closed when the encoded speed policy drifts from the bundled artifact profile.

### 2. Deterministic artifact generator

- Build a generation command that consumes validated public inputs and writes normalized intermediates plus compressed artifacts.
- Record source revisions, generator parameters, segment strategy, checksums, and output profile identifiers.
- Keep the prototype fixture path separate from production artifact generation.

### 3. Fit-error and benchmark matrix

- Add body-class fit-error reporting for boundary and interior samples.
- Benchmark single lookup, batch lookup, decode cost, artifact size, and full-chart packaged-data use.
- Fail validation when measured errors exceed profile thresholds.

## Phase 3 candidates — Compatibility catalog evidence

### 1. House formula evidence batch

- The current house-validation corpus already carries the release-facing formula families, latitude-sensitive systems, and documented constraints for the shipped catalog.
- Extend that evidence only when new release-advertised house systems are added or existing ones change status.
- Keep descriptor-only or approximate entries out of fully implemented claims.

### 2. Ayanamsa provenance batch

- The current validation and release summaries already surface representative provenance excerpts for the curated release-facing ayanamsa sample.
- Expand or refine the curated sample only if additional release-advertised ayanamsas need first-class provenance evidence.
- Continue classifying custom-definition-only entries explicitly.

### 3. Compatibility-profile claim audit

- The catalog inventory summary now carries an explicit claim-audit clause for baseline guarantees, release additions, custom-definition territory, and known gaps.
- Extend the same audit vocabulary to any future descriptor-only or constrained entries if those catalog categories are introduced later.
- Update release notes/docs to match the verified profile output.

## Phase 4 candidates — Release hardening

### 1. Final release gate command

- Compose existing checks into a documented release gate.
- `release-checklist` is now also reachable as `release-gate` / `release-gate-summary` in the CLI and validation front ends; the remaining work is the full blocking composition around format, clippy, tests, compatibility verification, artifact validation, bundle verification, audits, and benchmark/report generation.
- Ensure the gate blocks publication on stale reports or claim drift.

### 2. Clean-checkout bundle rehearsal

- Generate a release bundle from a clean checkout after Phases 1-3 changes.
- Verify manifests, sidecar checksums, artifact metadata, and report contents.
- Update docs for the exact reproducibility commands.

## Selection guidance

Prefer slices that convert an unverified claim into one of three explicit states:

1. implemented and validated,
2. implemented with documented constraints,
3. deferred/unsupported with structured errors and release-profile caveats.
