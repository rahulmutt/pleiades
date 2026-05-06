# Status 1 — Current Execution Frontier

## Frontier

The active frontier is **Phase 1 — Reference Accuracy and Request Semantics**.

The repository is past bootstrap, MVP API work, catalog scaffolding, report-surface expansion, and release-bundle rehearsal. The next production blocker is not another CLI summary or fixture-archaeology path; it is the evidence needed to make truthful release-grade ephemeris and packaged-data claims.

## Evidence reviewed

Current implementation status shows:

- all mandatory first-party crates exist and respect the `pleiades-*` naming rule;
- the backend trait, metadata, batch APIs, composite/routing helpers, chart façade, compatibility profiles, and validation/report commands are in place;
- first-party backend request policy is explicit: mean geometric, tropical, geocentric TT/TDB requests are supported; unsupported time scales, apparent-place requests, topocentric body-position requests, and native sidereal backend requests fail with structured errors;
- chart APIs preserve the distinction between house observers and body observers and provide caller-supplied UTC/UT1/TT/TDB offset helpers, but built-in Delta T/UTC conversion policy remains a release decision; the validation-report summary now mirrors that UTC-convenience posture explicitly, the backend and core façades now re-export the typed UTC-convenience, Delta T, and native sidereal policy summaries plus their current policy constructors for consumers that want the same wording without a backend dependency, the backend and core façades also expose matching `request_semantics_summary_for_report()` aliases for the combined request-policy wording, the request-surface inventory now surfaces Delta T as a separate report entrypoint, and the shared CLI request-policy help block now prints the UTC convenience line alongside the time-scale and Delta T lines; frame precision is explicitly bounded by the shared mean-obliquity frame round-trip envelope;
- `pleiades-vsop87` is source-backed for Sun through Neptune via generated VSOP87B tables; Pluto remains an explicitly approximate mean-elements fallback excluded from release-grade major-body claims;
- `pleiades-elp` is a compact Meeus-style lunar baseline with validation evidence for supported lunar channels, and its limitations summary now publishes the measured reference and equatorial channel envelopes directly; it is not a full ELP coefficient implementation;
- `pleiades-jpl` is a checked-in JPL Horizons snapshot/hold-out fixture backend with provenance and selected asteroid evidence, including dedicated 1500-01-01, 1600-01-11, 1750-01-01, 1800-01-03, 1900-01-01, and 2200-01-01 selected-body boundary report surfaces, 1749 major-body boundary coverage, a 2451912.5 major-body boundary report surface plus 2451913.5/2451915.5/2451917.5/2451918.5/2451919.5 major-body boundary report surfaces and a 2451916.0 interior reference surface with a direct CLI/report alias, a direct CLI/report surface for the 2451913.5 boundary slice, a 2451918.5 Mars/Jupiter boundary report surface now promoted into the top-level reference snapshot summary and now also exposed via an epoch-specific 2451918 major-body boundary alias, a 2451916.5 dense boundary report surface now promoted into the top-level reference snapshot summary, a 2500-01-01 selected-body boundary slice for Mars, Mercury, Moon, Sun, and Venus, a new 1750-01-01 interior boundary slice for Sun through Neptune, a 2360234.5 interior comparison slice now exposed through a first-class report surface, a 2451910.5 major-body boundary report surface now surfaced through a first-class report surface, a first-class high-curvature hold-out summary for the 2451915.25/2451915.75 Sun/Moon/Mercury/Venus window now also surfaced through the combined JPL evidence report, a 2451920.5 interior reference slice, a 2400000.0 major-body boundary summary now surfaced through a first-class report surface, an added 2453000.5 major-body boundary summary promoted into the top-level reference snapshot summary, an epoch-specific 2451914 major-body pre-bridge alias, an epoch-specific 2451914 bridge-day alias, and an epoch-specific 2451915 major-body bridge alias, and a reference snapshot summary that now surfaces the 1749, 1750-01-01, 1800-01-03, 2451912.5, 2451913.5, 2451916.0, 2451916.5, 2451917.5, 2451918.5, 2451919.5, 2451920.5, 2400000.0, and 2453000.5 slices alongside the earlier boundary summaries, plus an updated comparison-corpus guard that now reflects the current 2451913.5 boundary-day coverage; the boundary-window and boundary-epoch-coverage aggregate report surfaces now have direct regression coverage, not a broad production reader/corpus;
- `pleiades-data` ships a deterministic prototype artifact with codec/profile/checksum/regeneration support, and the checked-in fixture tracks the current reference-snapshot slice, but it is not yet a production 1500-2500 CE artifact and current fit posture is not release-grade;
- house and ayanamsa catalogs are broad, but not every release-advertised entry has independent formula/provenance/reference evidence sufficient for full interoperability claims; the catalog inventory now carries an explicit claim-audit clause for baseline guarantees, release additions, custom-definition territory, and known gaps, and the release summary now includes representative ayanamsa provenance excerpts for curated built-in entries;
- release-bundle generation and verification exist, and validation/report surfaces now classify evidence as release-tolerance, hold-out, fixture exactness, or provenance-only; the comparison audit surface now also mirrors body-class tolerance posture for the release-grade corpus; final release gates still need to be rerun over production accuracy evidence, production artifacts, and truthful compatibility profiles.

## Why this frontier comes first

Phase 2 production artifacts require trusted generation inputs and target thresholds. Phase 4 release claims require the same evidence. Therefore maintainers should close source-backed accuracy and request-policy gaps before claiming production packaged-data coverage or broad compatibility.

## Immediate blockers

1. **Reference corpus breadth** — The production-generation input path is now explicitly documented as the checked-in CSV fixture pair, now widened with a 1750-01-01 interior boundary slice and a 2200-01-01 interior selected-body slice; continue expanding or replacing fixture evidence with production-suitable public source/reference data for validation and artifact fitting.
2. **Pluto release posture** — Pluto is now visibly approximate and excluded from release-grade major-body claims; a source-backed path remains optional if future release claims need it.
3. **Lunar release posture** — Decide whether the first release keeps the compact lunar baseline or implements fuller ELP-style coefficient support; the compact path now publishes measured reference and equatorial channel envelopes in the limitations summary.
4. **Advanced request semantics** — These request modes are now explicitly deferred in the first-party backend posture; keep the report wording, metadata, and rustdoc aligned while the next implementation slice shifts back to reference breadth. Frame precision remains stated via the shared mean-obliquity frame round-trip envelope.

## Recommended next slice

The representative early-boundary slice is now checked in for Sun, Moon, Mercury, and Venus at 1500-01-01, now surfaced through a dedicated boundary summary, and the 1600-01-11, 1750-01-01, and 1900-01-01 selected-body boundary slices plus a 2500-01-01 selected-body boundary slice now give additional release-facing boundary points for the same release corpus. A 2451915.25/2451915.75 high-curvature hold-out window supplements the validation corpus for Sun, Moon, Mercury, and Venus.

The reference corpus also now includes a 1749 major-body boundary slice, a 1750-01-01 interior selected-body slice, a dedicated 1800-01-03 major-body boundary slice, dedicated 2451910.5, 2451911.5, 2451912.5, 2451913.5, 2451914.5, 2451915.5, 2451916.5, 2451917.5, 2451918.5, and 2451919.5 boundary-day reports, a dedicated 2360234.5 interior comparison report surface, and a dedicated 2451920.5 interior report surface; the 2451914 pre-bridge boundary day now has an epoch-specific CLI alias, the 2451914.5 major-body boundary day now has a first-class report surface, the 2451915 major-body boundary day now has a first-class report surface, the 2451918 Mars/Jupiter boundary day now has an epoch-specific CLI alias, and the 2451915 major-body bridge day retains its epoch-specific CLI alias, so the next reviewable step can move to another reference-breadth slice or a clearly scoped request-policy doc/report cleanup if no additional corpus row is needed. The top-level reference snapshot summary now mirrors those slices as well.

Next, implement another small, reviewable slice:

- keep hold-out rows separate from fitting/reference rows;
- update the relevant backend metadata/report summaries and tests without broadening release claims prematurely;
- prefer a fresh interior/boundary reference slice over more request-policy wording unless a new implementation decision is made.

## Parallel safe work

The following can proceed without blocking Phase 1:

- house/ayanamsa formula, alias, latitude, and custom-definition audits;
- custom-definition ayanamsa example labels are now centralized in `pleiades-ayanamsa` and surfaced through the compatibility profile;
- documentation cleanup for already-explicit request policy and known gaps;
- artifact-profile metadata hardening that does not claim production fit accuracy;
- release-bundle smoke-test maintenance that keeps existing rehearsal tooling accurate.

## Constraints

- Preserve pure Rust and first-party crate layering.
- Do not make domain crates depend on concrete backends.
- Do not silently satisfy unsupported apparent/topocentric/native-sidereal requests.
- Do not publish broader accuracy, artifact, or compatibility claims until validation evidence supports them.
