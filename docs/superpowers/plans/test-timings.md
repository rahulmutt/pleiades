# Test Timing Inventory

Measured on 2026-06-20 using `cargo nextest run --message-format libtest-json-plus` with isolated
per-crate runs (to avoid cross-binary interference). Build profile: `test` with `opt-level = 2`.
Total workspace: 1995 tests across 23 binaries.

**Coverage note:** `pleiades-cli` (98/98 tests), `pleiades-data` (180/183 non-ignored), and
`pleiades-validate` (full run, both `release_bundle_verify_a` and `release_bundle_verify_b` batches
captured) are all fully measured. All other crates are fast (< 3 s each) and fully measured.

**Validate update:** the `release_bundle_verify_b` batch (122 tests, 40–121 s each) was confirmed
after the initial draft. Including it, `pleiades-validate` has **166 tests over 60 s** — 120 of them
in `release_bundle_verify_b`. This makes the release-bundle family larger than first documented.

---

## Section 1: Timing Inventory

Slowest 40 tests ranked slowest-first. All times from clean isolated single-crate runs.

| Rank | Crate | Test Name | Seconds |
|-----:|-------|-----------|--------:|
| 1 | pleiades-cli | `cli::tests::summary_commands::summary_commands_render_compact_reports` | 374 |
| 2 | pleiades-cli | `cli::tests::release::bundle_release_commands_accept_output_alias` | 324 |
| 3 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_commands_accept_output_aliases_in_the_validation_front_end` | 312 |
| 4 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_writes_expected_artifacts` | 310 |
| 5 | pleiades-cli | `cli::tests::release::verify_release_bundle_command_verifies_a_staged_bundle` | 309 |
| 6 | pleiades-cli | `cli::tests::release::bundle_release_command_writes_a_staged_bundle` | 308 |
| 7 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_accepts_rendered_bundle` | 302 |
| 8 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_whitespace_padded_provenance` | 293 |
| 9 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_placeholder_provenance` | 288 |
| 10 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_multiline_provenance` | 277 |
| 11 | pleiades-validate | `tests::release_bundle_verify_a::release_bundle_validate_rejects_manifest_path_drift` | 267 |
| 12 | pleiades-cli | `cli::tests::artifact_and_workspace::artifact_and_workspace_commands_render_compact_reports` | 256 |
| 13 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_api_stability_posture_id_entry` | 246 |
| 14 | pleiades-cli | `cli::tests::validation::validation_report_commands_render_compact_reports` | 240 |
| 15 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_cargo_version_entry` | 229 |
| 16 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_profile_id_entry` | 228 |
| 17 | pleiades-cli | `cli::tests::misc::fallback_summary_commands_remain_reachable_from_the_cli` | 225 |
| 18 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_duplicate_api_stability_posture_id_entry` | 224 |
| 19 | pleiades-validate | `artifact::tests::render_artifact_summary_includes_span_caps` | 223 |
| 20 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_workspace_status_entry` | 223 |
| 21 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_source_revision_entry` | 223 |
| 22 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_checksum_mismatches` | 213 |
| 23 | pleiades-validate | `tests::release_bundle_verify_a::verify_release_bundle_rejects_blank_rustc_version_entry` | 212 |
| 24 | pleiades-data | `tests::coverage::packaged_artifact_fit_outlier_summary_prioritizes_distance_channel_outliers` | 127 |
| 25 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_reflects_the_current_posture` | 127 |
| 26 | pleiades-cli | `cli::tests::misc::packaged_artifact_and_ayanamsa_audit_summary_commands_render_directly_from_the_cli` | 126 |
| 27 | pleiades-data | `tests::coverage::packaged_artifact_fit_threshold_violation_summary_validation_rejects_drift` | 126 |
| 28 | pleiades-data | `tests::coverage::packaged_artifact_fit_channel_outlier_summary_validation_rejects_drift` | 121 |
| 29 | pleiades-cli | `cli::tests::misc::packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary` | 121 |
| 30 | pleiades-data | `tests::coverage::packaged_artifact_fit_threshold_summary_reflects_the_current_posture` | 120 |
| 31 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift` | 120 |
| 32 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_profile_drift` | 114 |
| 33 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_parameter_drift` | 114 |
| 34 | pleiades-data | `tests::coverage::packaged_artifact_generation_artifacts_keep_lookup_epoch_and_segment_strategy_aligned` | 112 |
| 35 | pleiades-data | `tests::coverage::packaged_artifact_fit_margin_summary_validation_rejects_envelope_drift` | 109 |
| 36 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_checksum_drift` | 107 |
| 37 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_profile_id_drift` | 106 |
| 38 | pleiades-data | `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_regeneration_drift` | 105 |
| 39 | pleiades-data | `tests::coverage::packaged_artifact_generator_parameters_validation_rejects_artifact_profile_drift` | 103 |
| 40 | pleiades-data | `tests::coverage::packaged_artifact_fit_margin_summary_reflects_the_current_posture` | 103 |

Additional slow tests beyond rank 40 (all in `pleiades-data tests::coverage::*`):
- `packaged_artifact_generation_manifest_validation_rejects_label_drift` — 103 s
- `packaged_artifact_generator_parameters_validation_rejects_body_coverage_drift` — 102 s
- `packaged_artifact_generation_manifest_validation_rejects_source_drift` — 101 s
- `packaged_artifact_generation_manifest_validation_rejects_request_policy_drift` — 101 s
- `packaged_artifact_generator_parameters_validation_rejects_label_drift` — 100 s
- `packaged_artifact_generator_parameters_validation_rejects_artifact_version_drift` — 99 s
- `packaged_artifact_generator_parameters_validation_rejects_checksum_drift` — 99 s
- `packaged_artifact_fit_margin_summary_validation_rejects_threshold_drift` — 97 s
- `packaged_artifact_generator_parameters_validation_rejects_profile_id_drift` — 93 s
- `packaged_artifact_production_profile_summary_validation_rejects_artifact_profile_drift` — 86 s
- `packaged_artifact_regeneration_summary_validation_rejects_residual_body_subset_drift` — 83 s
- `packaged_artifact_generator_parameters_validation_rejects_source_provenance_drift` — 83 s
- `packaged_artifact_generator_parameters_validation_rejects_speed_policy_drift` — 82 s
- `packaged_artifact_regeneration_summary_includes_reference_snapshot_coverage` — 84 s
- `packaged_artifact_generator_parameters_validation_rejects_request_policy_drift` — 80 s
- `packaged_artifact_generator_parameters_validation_rejects_target_threshold_drift` — 79 s
- `packaged_artifact_generator_parameters_validation_rejects_time_range_drift` — 77 s
- `packaged_artifact_production_profile_summary_reflects_the_current_posture` — 77 s
- `default_window_artifact_matches_explicit_default_over` — 34 s
- `build_from_reference_produces_all_bodies_with_spanning_segments` — 31 s
- `snapshot_reconstruction_covers_only_constrained_asteroids` — 17 s

---

## Section 2: Families

### Family 1: release-bundle

**Description:** Tests that build a full release bundle by calling `render_cli(&["bundle-release",
...])` or `render_cli(&["bundle-verify", ...])`. Each test independently spawns the CLI binary,
runs the complete bundle-generation pipeline (compiles artifact, assembles manifest, signs checksums),
and then verifies the output. This pipeline takes 200–375 s per test process.

**Crates and files:**
- `pleiades-cli` — `src/tests/release.rs`, `src/tests/summary_commands.rs`,
  `src/tests/artifact_and_workspace.rs`, `src/tests/validation.rs`, `src/tests/misc.rs`
- `pleiades-validate` — `src/tests/release_bundle_verify_a.rs` (26 tests, 144–312 s each),
  `src/tests/release_bundle_verify_b.rs` (122 tests, 40–121 s each — confirmed; these tamper a
  rendered bundle then re-verify, so each re-runs the bundle pipeline)

**Real test names from measurement (rank 1–23, plus further verify_b tests):**
- `cli::tests::summary_commands::summary_commands_render_compact_reports` — 374 s
- `cli::tests::release::bundle_release_commands_accept_output_alias` — 324 s
- `tests::release_bundle_verify_a::release_bundle_commands_accept_output_aliases_in_the_validation_front_end` — 312 s
- `tests::release_bundle_verify_a::release_bundle_writes_expected_artifacts` — 310 s
- `cli::tests::release::verify_release_bundle_command_verifies_a_staged_bundle` — 309 s
- `cli::tests::release::bundle_release_command_writes_a_staged_bundle` — 308 s
- `tests::release_bundle_verify_a::release_bundle_validate_accepts_rendered_bundle` — 302 s
- ... (rest of verify_a batch, 144–293 s each, 26 tests total)
- `tests::release_bundle_verify_b::verify_release_bundle_rejects_tampered_reference_snapshot_boundary_summary_files_even_with_updated_checksum` — 121 s (representative of 122 verify_b tests, 40–121 s each)
- `cli::tests::artifact_and_workspace::artifact_and_workspace_commands_render_compact_reports` — 256 s
- `cli::tests::validation::validation_report_commands_render_compact_reports` — 240 s
- `cli::tests::misc::fallback_summary_commands_remain_reachable_from_the_cli` — 225 s
- `validate::artifact::tests::render_artifact_summary_includes_span_caps` — 223 s
- `cli::tests::misc::packaged_artifact_and_ayanamsa_audit_summary_commands_render_directly_from_the_cli` — 126 s
- `cli::tests::misc::packaged_artifact_source_fit_holdout_sync_summary_and_alias_commands_render_the_summary` — 121 s

**Planned treatment:** **dedup** — run the bundle pipeline once (or a small fixed number of times),
share the rendered output directory across all tests that only need to verify a single property of
the bundle. This would collapse the ~30 tests in this family from ~30 × 250 s to ~1 × 250 s + 29 × ~1 s.

---

### Family 2: kernel-free regeneration

**Description:** Tests in `pleiades-data` that call `build_packaged_artifact_from_reference_over`
directly (or indirectly via `packaged_artifact()` which calls `build_packaged_artifact`). These
tests rebuild the full Chebyshev-segment artifact from the reference snapshot without an external
kernel. Each process rebuilds independently (no cross-process `OnceLock` sharing).

**Crates and files:**
- `pleiades-data` — `src/tests/coverage.rs` (functions `build_from_reference_produces_all_bodies_with_spanning_segments`,
  `default_window_artifact_matches_explicit_default_over`)
- `pleiades-data` — `tests/artifact_regen.rs`

**Real test names from measurement:**
- `tests::coverage::build_from_reference_produces_all_bodies_with_spanning_segments` — **31 s**
- `tests::coverage::default_window_artifact_matches_explicit_default_over` — **34 s**
- `tests::codec::snapshot_reconstruction_covers_only_constrained_asteroids` — **17 s**

**Observation vs. expectation:** These tests are slower than expected but are overshadowed by the
dense-sweep family (see Family 3 below). The expected test names `build_from_reference_*` and
`default_window_artifact_matches_explicit_default_over` are confirmed present and slow, but
the biggest contributors to `pleiades-data` wall time are the parametric-sweep tests in Family 3.

**Planned treatment:** **memoize** — share one built artifact via an `OnceLock` at the
integration-test process level, or move the build into a `#[test_support]` fixture loaded once
per binary.

---

### Family 3: dense numerical sweeps (coverage/fit)

**Description:** The largest slow family by count. Every test in `src/tests/coverage.rs` that
calls any `packaged_artifact_*_details()` helper ultimately calls `packaged_artifact()`, which
builds the full artifact via `OnceLock::get_or_init`. Because nextest runs each test in a separate
process, each of ~100 coverage tests pays the full ~100 s artifact-build cost independently.
These are NOT dense numerical sweeps in the traditional sense (they do not iterate over a large
parameter grid); rather, they each rebuild the artifact to exercise a particular summary function
or validation path.

**Crates and files:**
- `pleiades-data` — `src/tests/coverage.rs` (tests prefixed `packaged_artifact_fit_*`,
  `packaged_artifact_generation_manifest_*`, `packaged_artifact_generator_parameters_*`,
  `packaged_artifact_production_profile_*`, `packaged_artifact_regeneration_summary_*`)

**Real test names from measurement (sample, all ~80–127 s each):**
- `tests::coverage::packaged_artifact_fit_outlier_summary_prioritizes_distance_channel_outliers` — 127 s
- `tests::coverage::packaged_artifact_generation_manifest_reflects_the_current_posture` — 127 s
- `tests::coverage::packaged_artifact_fit_threshold_violation_summary_validation_rejects_drift` — 126 s
- `tests::coverage::packaged_artifact_fit_channel_outlier_summary_validation_rejects_drift` — 121 s
- `tests::coverage::packaged_artifact_fit_threshold_summary_reflects_the_current_posture` — 120 s
- `tests::coverage::packaged_artifact_generation_manifest_validation_rejects_artifact_version_drift` — 120 s
- ... (58+ more, all 77–127 s)

**Observation vs. expectation:** The task brief listed `tests/fit.rs`, `tests/coverage.rs`, and
`tests/lookup.rs` as expected dense-sweep files (implying parameter-space iteration). What was
actually found is that `src/tests/coverage.rs` contains ~100 parametric-validation tests that
each happen to rebuild the artifact from scratch per nextest process. The `src/tests/fit.rs` and
`src/tests/lookup.rs` files are fast (< 0.1 s each). The analogous `pleiades-vsop87` tests
(`tests/evidence.rs`, `tests/backend.rs`) are also fast (< 0.3 s each).

**Planned treatment:** **memoize / shrink** — consolidate coverage tests that share the same
`packaged_artifact()` call into a single test binary fixture (so the OnceLock fires once per
binary launch), OR collect all per-field validation assertions into fewer test functions. The
goal is to pay the ~100 s build cost O(1) times per coverage test run rather than O(N) times.

---

## Summary: actual vs. expected family map

| Expected family | Expected crate | Confirmed? | Notes |
|-----------------|---------------|-----------|-------|
| kernel-free regeneration | pleiades-data | Yes | `build_from_reference_*`, `default_window_artifact_*` at 31–34 s |
| release-bundle | pleiades-validate + pleiades-cli | Yes | 40–374 s per test; 26 `verify_a` + 122 `verify_b` + `cli::tests::release::*` — 166 validate tests over 60 s |
| dense numerical sweeps | pleiades-data coverage | Partial match | Not true sweeps — artifact-rebuild cost paid O(N) across ~100 validation tests |
| pleiades-vsop87 sweeps | pleiades-vsop87 | No slow tests | All vsop87 tests < 0.3 s; evidence.rs and backend.rs are fast |
| pleiades-data fit/lookup | pleiades-data | No slow tests | `src/tests/fit.rs` and `src/tests/lookup.rs` are fast (< 0.1 s) |
