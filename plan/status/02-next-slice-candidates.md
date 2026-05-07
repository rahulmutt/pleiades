# Status 2 — Next Slice Candidates

This file lists focused implementation slices that map to the current phase ladder. It intentionally omits completed report-surface, alias, fixture-summary, and release-rehearsal cleanup work.

## Phase 1 candidates — Reference accuracy and request semantics

### 1. Representative 1500-2500 reference expansion

- The 1500-01-01, 1600-01-11, 1900-01-01, and 2200-01-01 selected-body boundary slices are now also surfaced through the top-level reference snapshot summary and release notes summary, so this early-boundary sub-slice is complete. The release-facing summaries now keep the 1600-01-11 and 1750-01-01 selected-body blocks on separate lines, and the CLI/validation fronts now also expose the 2268932 and 2305457 exact-JD aliases for the 1500 and 1600 selected-body boundary slices.
- The generic reference bridge-day summary and 2451914 major-body bridge-day alias are now also promoted through the top-level reference snapshot and release notes summaries, so any remaining reference-breadth work can move to another epoch.
- The lunar boundary evidence summary is now also surfaced through the top-level reference snapshot summary, and the major-body boundary-window aggregate is now surfaced there as well, while the release notes summary now mirrors the boundary-epoch coverage and boundary-window aggregates for the same frontier, so the next breadth slice can focus on another boundary epoch or a representative interior comparison row.
- A 2500-01-01 selected-body boundary slice for Mars, Mercury, Moon, Sun, and Venus is now checked in and now has direct CLI parity in `pleiades-cli`.
- A 1749 major-body boundary slice, the early major-body boundary slice, and the Mars/Jupiter boundary slice are now checked in and surfaced through the top-level reference snapshot summary. The early boundary slice now also has an exact 2378498 JD alias for naming parity.
- A 1750-01-01 interior boundary slice for Sun through Neptune is now checked in and now has a first-class 1750 major-body interior report surface.
- A dedicated 1800-01-03 major-body boundary slice is now checked in and now surfaces through the top-level reference snapshot summary.
- A 2451916.0 interior reference slice is now checked in and surfaced through a first-class report surface with a direct CLI alias.
- A 2451920.5 interior reference slice is now checked in, and the validation CLI now explicitly regression-tests it inside the combined reference snapshot summary; the release-summary and validation-report layers now also surface it explicitly.
- The 2451915.25/2451915.75 high-curvature hold-out window is now surfaced through the combined JPL evidence report, and the selected-asteroid boundary, bridge, dense, and source evidence/window summaries are now surfaced there too, while the 2451910 through 2451915 major-body boundary summaries plus the 2451914 major-body bridge-day summary, the 2451917 boundary and bridge summaries, and the 2451918/2451919 boundary slices with the 2451920 interior slice are now also explicitly surfaced there; keep hold-out rows separate from fitting/reference rows, and the reference snapshot summary regression now explicitly keeps the hold-out block out of the reference summary, while the next slice targets any remaining boundary breadth. The top-level reference snapshot summary now also includes the 2451917 major-body boundary and bridge summaries and now surfaces the major-body high-curvature evidence, window, and epoch-coverage summaries as well, but it still excludes the hold-out high-curvature block. The release-notes summary now also surfaces the 2451917 boundary and bridge summaries explicitly, and the 2451918 compatibility alias now renders explicit 2451918 wording in release-facing reports, so alias-parity cleanup for that slice is now complete.
- The major-body high-curvature summary/window/epoch-coverage surfaces now also expose snapshot-prefixed and major-body-prefixed aliases in the CLI and validation front ends for discoverability.
- A 2453000.5 major-body boundary summary is now checked in and surfaced through the top-level reference snapshot summary.
- A 2500000.0 major-body boundary summary is now checked in and surfaced through the top-level reference snapshot summary and CLI/help aliases.
- A 2600000.0 major-body boundary summary for the Mars outer-boundary anchor is now checked in and surfaced through the top-level reference snapshot summary and CLI/report/help aliases.
- A 2500 major-body boundary summary is now checked in and surfaced through the top-level reference snapshot summary.
- The 2200 selected-body boundary slice now also has a 2524593 JD-labeled alias in the CLI and validation front ends, the 1900 selected-body boundary slice now also has a 2415020 JD-labeled alias, and the 2500 selected-body boundary slice now also has a 2634167 JD-labeled alias, so those frontier points are covered by both year and epoch naming. The top-level reference snapshot summary now also surfaces both aliases alongside the year-based 2200 and 2500 boundary entries.
- The Mars outer-boundary summary is now checked in and surfaced through the top-level reference snapshot summary alongside the late corpus edge at JD 2634167.0.
- A 2400000.0 major-body boundary summary is now checked in and surfaced through a first-class report surface, and the top-level reference snapshot regression now explicitly anchors that slice alongside the 2451545.0 J2000 boundary.
- A 2451545.0 major-body boundary summary for the J2000 major-body reference slice is now first-class with direct CLI/report aliases and top-level reference-snapshot regression coverage.
- The exact-J2000 evidence summary is now also surfaced through the top-level reference snapshot summary, and the selected-asteroid boundary, bridge, dense, terminal, and source evidence/window slices are now surfaced there as well, along with the reference source and source-window provenance summaries plus the sparse boundary and pre-bridge aggregate summaries, so the next breadth slice can focus on another boundary epoch or representative interior row if more reference breadth is still needed. The release notes summary now also surfaces the 2451916 dense boundary slice explicitly, the release notes summary and release summary now also surface the 2451916 interior and boundary slices explicitly, and the release notes summary and validation report now also surface the terminal slice explicitly.
- Dedicated 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451915.5, 2451917.5, 2451918.5, and 2451919.5 major-body boundary report slices are now first-class and validated, with direct CLI/report surfaces now exposed for 2451910.5, 2451911.5, 2451912.5, 2451913.5, and 2451915.5, plus the 2451917.0 bridge slice, the 2451918.5 Mars/Jupiter boundary slice and its epoch-specific 2451918 alias now has backend drift regression coverage, the 2451916.5 dense boundary day is now also promoted through the top-level reference snapshot summary and now has an epoch-specific 2451916 major-body dense-boundary alias, plus a generic 2451916 major-body boundary alias for discoverability; the top-level reference snapshot summary now also surfaces that generic boundary alias, and the CLI parity layer now mirrors the 2451912, 2451913, 2451914, 2451917, 2451918 boundary aliases plus the 2451914 pre-bridge, 2451914 bridge-day, and generic bridge-day aliases, including the 2451914-bridge-day-summary alias, the 2451915 bridge alias now rendering explicit 2451915 wording in release-facing reports, the 2451917 bridge alias, and the 2451916 dense-boundary alias, while the JPL backend API now also exposes epoch-specific aliases for the 2451914 pre-bridge, 2451914 bridge, and 2451915 bridge report surfaces, and the top-level reference snapshot summary now surfaces the 1749, 1750-01-01, 1800-01-03, 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451915.5, 2451916.5, 2451917.0, 2451917.5, 2451918.5, 2451919.5, 2451920.5, and 2453000.5 slices alongside the earlier boundary summaries, plus the generic bridge-day summary and the 2451914 pre-bridge and bridge-day report surfaces. The reference-snapshot summary regression now also explicitly anchors the 2451912, 2451913, 2451914, 2451916 dense, and 2451917 bridge slices. This alias-parity slice is now complete in the backend API, CLI, validation front ends, and the release-notes summary; the CLI/validation help inventory now also pins the 2451914 bridge-day and 2451915 bridge aliases explicitly.
- The validation-report and release-summary layers now also surface the 2451918 and 2451919 boundary slices explicitly, and now also surface the 2451920 interior slice explicitly; the top-level reference snapshot summary also now carries the selected-asteroid source evidence/window slices alongside the already-advertised boundary, bridge, dense, and terminal evidence blocks, and those selected-asteroid source commands now also have reference-snapshot-prefixed CLI/validation aliases for naming parity. The CLI parity layer now also rejects stray arguments for the 2451919 boundary alias, matching the validation front end, and the CLI help inventory now also asserts the `2451919-major-body-boundary-summary` and `2451920-major-body-interior-summary` alias lines.
- The comparison-corpus guard stays aligned with the 26-epoch release-grade comparison corpus, while 2451913.5 remains reference-only evidence, and the generic major-body boundary summary is now also first-class in the top-level reference snapshot summary, so the next reference-breadth slice can target another boundary or representative interior epoch if breadth still needs to grow; the 2451914 pre-bridge boundary day now has an epoch-specific CLI alias, the 2451914 bridge day now has its own epoch-specific CLI alias, the 2451914 major-body bridge-day alias is now also exposed for naming parity and surfaced in the top-level reference snapshot summary, the 2451915 major-body boundary day now has a first-class report surface, the 2451917.0 bridge day now has its own epoch-specific CLI alias, the 2451918 Mars/Jupiter boundary day now has its epoch-specific CLI alias, the 2451915 major-body bridge alias now renders explicit 2451915 wording in release-facing reports, and the 2451915 bridge slice is now explicitly regression-anchored in the top-level summary and validation report; the 2451919 boundary slice is now also surfaced in the release-facing reports, the top-level reference snapshot summary regression now also pins the 2451918 and 2451919 boundary surfaces in CLI and validation coverage, the boundary-window / boundary-epoch-coverage aggregate report surfaces now carry direct regression coverage, and the boundary-epoch-coverage slice now widens through JD 2451912.5..JD 2451919.5; the independent hold-out high-curvature window is now included in the combined evidence report without being folded into reference rows; the remaining breadth work can now move to a different epoch, a new source-backed comparison slice, or a concise request-policy cleanup if no further corpus row is needed.
- The JPL backend API now also exposes explicit 2360233 and 2378499 major-body boundary report aliases for the 1749 and 1800 reference slices, while the CLI runtime now also covers the 1500 selected-body, 1750 major-body interior, 1900 selected-body, 2415020 selected-body, 2360233 major-body boundary, 2360234 major-body interior, 2378499 major-body boundary, and 2451911 major-body boundary aliases directly, so the current alias-parity slice is fully exercised in the CLI and validation fronts. The top-level reference snapshot summary now also surfaces the 2360233.5 and 2378499.0 alias views of those boundary slices, and the release notes summary now mirrors those alias views as well; the 2451914 bridge-day / major-body bridge-day typed aliases now also have direct regression coverage; the CLI render tests now also pin the `2451914-bridge-day-summary` and `2451914-major-body-bridge-day-summary` aliases alongside the base bridge-day command, so that bridge-day naming path is no longer an open follow-up.
- The combined JPL evidence report now makes the checked-in fixture backend / separate generation-input posture explicit, so the remaining breadth work is now about corpus coverage rather than source-role ambiguity; it now also surfaces the 1750 major-body interior comparison slice alongside the current boundary coverage and includes the generic bridge-day summary alongside the 2451914 major-body bridge-day slice, plus the 2451914 major-body bridge summary.
- Keep hold-out rows separate from fitting/reference rows.
- The release-grade body-claims posture is now explicit in typed backend/core summaries and release reporting, so keep validation work focused on keeping per-class tolerances, claim status, and any newly advertised bodies aligned with that claim set.
- Update validation reports to classify evidence as release-tolerance, hold-out, fixture exactness, or provenance-only; the comparison audit surface now mirrors body-class tolerance posture for the current release-grade corpus.

### 3. Lunar source posture

- The first release keeps the compact Meeus-style truncated lunar baseline.
- If future releases add coefficient data, they should bring pure-Rust ingestion/evaluation, provenance, validation, and tests with them.

### 4. Request/time semantics closure

- The first-release request-policy posture is now explicit: built-in Delta T and UTC/UT1 convenience conversion remain deferred, validation-report summaries surface that deferral explicitly, the backend/core façades re-export the UTC-convenience, Delta T, and native sidereal policy summaries plus their current constructors, the request-semantics aliases now render a distinct `Request semantics summary` title while keeping the same policy body, and the request-policy/request-semantics renderers now share the same formatter; the request-surface inventory lists Delta T as a separate report entrypoint. The release bundle now also carries a distinct `request-semantics-summary.txt` sidecar rendered from the request-semantics entrypoint alongside `request-policy-summary.txt`, so the request-policy/request-semantics cleanup is now reflected in the reproducibility outputs as well. Pluto fallback posture is now also surfaced through a shared typed backend/core summary and validation report line, and the release-grade body claims posture now has a standalone release-body-claims summary command, so keep future cleanup focused on the remaining request/time gaps instead of restating the Pluto fallback prose. The generic `bridge-summary` alias now covers the major-body bridge report, the 2451914 bridge-day slice is now also regression-anchored in the CLI/validation summary tests, and unsupported-time-scale precedence over malformed observer input is now regression-tested in direct metadata checks, so any further cleanup here should stay limited to report phrasing; the shared CLI request-policy help block now also names the request-semantics line explicitly.
- Apparent-place corrections and topocentric body positions remain explicitly deferred unless a later backend-capability decision changes that posture.
- Native sidereal backend output remains deferred unless a backend advertises equivalent support through capabilities.
- If no request-policy decision changes, prefer the next slice to expand reference breadth (for example by promoting another checked-in interior/boundary epoch to a first-class report surface) before revisiting more request-policy wording; the `request-semantics` help-text cleanup now keeps the alias wording aligned, so any further cleanup here should stay limited to report/help phrasing. The current reference help inventory is now synchronized with the advertised manifest/body-class/exact-J2000/boundary aliases, so future drift fixes should only follow new command additions.

## Phase 2 candidates — Production compressed artifacts

### 1. Production artifact profile manifest

- Specify body set, date range, channels, derived outputs, unsupported outputs, speed policy, and thresholds; lookup-epoch policy and speed policy are now explicit in the production-profile draft and generator manifest, and the production-profile/generator/manifest summaries now also call out the segment strategy explicitly; the packaged-data surfaces now expose a standalone speed-policy summary in CLI/validation reporting, the CLI/validation front ends now also mirror the packaged lookup-epoch policy with packaged-artifact-prefixed aliases, the packaged-artifact generation manifest summary now also has a direct `packaged-artifact-generation-manifest` alias, and the release bundle now carries the packaged lookup-epoch policy, packaged-artifact profile coverage, and packaged-artifact speed-policy summary/checksums alongside the production-profile and target-threshold bundle outputs.
- Add validation that fails on profile/threshold drift; the production-profile and generator-parameter summaries now also fail closed when the encoded speed policy drifts from the bundled artifact profile.

### 2. Deterministic artifact generator

- Build a generation command that consumes validated public inputs and writes normalized intermediates plus compressed artifacts; the current fixture workflow now exposes both generation and regenerate entrypoints, so follow-on work can focus on normalized intermediates and production artifact writes.
- Record generator parameters, checksums, output profile identifiers, and per-channel quantization scales; the packaged-artifact regeneration provenance now already exposes and validates the codec quantization-scale metadata, the checked-in reference snapshot summary equality is now also validated there, and the source-revision provenance slice is now closed, so the next slice can concentrate on the remaining drift-proof manifest updates.
- Keep the prototype fixture path separate from production artifact generation.

### 3. Fit-error and benchmark matrix

- The boundary/interior fit sample classes summary is now surfaced alongside the packaged-artifact fit envelope summary in the validation report and CLI, so the fit-error slice now has a first-class report path.
- Benchmark single lookup, batch lookup, decode cost, artifact size, and full-chart packaged-data use.
- Fail validation when measured errors exceed profile thresholds.

## Phase 3 candidates — Compatibility catalog evidence

### 1. House formula evidence batch

- The current house-validation corpus already carries the release-facing formula families, latitude-sensitive systems, and documented constraints for the shipped catalog.
- Extend that evidence only when new release-advertised house systems are added or existing ones change status.
- Keep descriptor-only or approximate entries out of fully implemented claims.
- The packaged-artifact generation manifest summary sidecar is now staged alongside the generation manifest, so the next production-artifact slice can focus on drift-proof manifest updates.

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
- `release-checklist` is now also reachable as `release-gate` / `release-gate-summary` in the CLI and validation front ends; those gate commands now also perform compatibility-profile verification plus release-bundle generation/verification before rendering the checklist text, and the remaining work is the full composition of format, strict clippy, workspace tests, and audit steps into the same blocking gate.
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
