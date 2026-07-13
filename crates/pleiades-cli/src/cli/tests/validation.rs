//! Tests for validation-report commands and release-profile-identifiers.

use pleiades_core::current_release_profile_identifiers;
use pleiades_validate::render_cli as validate_render_cli;

use crate::cli::render_cli;

#[ignore = "slow: run via `mise test-full` or `cargo test -- --include-ignored`"]
#[test]
fn validation_report_commands_render_compact_reports() {
    let release_profiles = current_release_profile_identifiers();

    let report = render_cli(&["report", "--rounds", "1"])
        .expect("report should render through the primary CLI");
    assert!(report.contains("Validation report"));
    assert!(report.contains("Comparison corpus"));
    assert!(report.contains("release-grade guard: Pluto excluded from tolerance evidence"));
    assert!(report.contains("Benchmark corpus"));
    assert!(report.contains("Packaged-data benchmark corpus"));

    let generate_report = render_cli(&["generate-report", "--rounds", "1"])
        .expect("generate-report should render through the primary CLI");
    assert!(generate_report.contains("Validation report"));
    assert!(generate_report.contains("Comparison corpus"));

    let validation_summary =
        render_cli(&["validation-summary"]).expect("validation summary should render");
    assert!(validation_summary.contains("Validation report summary"));
    assert!(validation_summary.contains("Comparison corpus"));
    assert!(
        validation_summary.contains("release-grade guard: Pluto excluded from tolerance evidence")
    );
    assert!(validation_summary.contains("Release bundle verification: verify-release-bundle"));
    assert!(
        validation_summary.contains("Compatibility profile summary: compatibility-profile-summary")
    );
    assert!(validation_summary.contains("Release notes summary: release-notes-summary"));
    assert!(validation_summary.contains("Release checklist summary: release-checklist-summary"));
    assert!(validation_summary.contains("Release summary: release-summary"));
    assert!(validation_summary.contains("House validation corpus"));
    assert!(validation_summary.contains("Benchmark summaries"));
    assert!(validation_summary.contains("Packaged-data benchmark"));

    let validation_summary_rounds = render_cli(&["validation-summary", "--rounds", "1"])
        .expect("validation summary should accept explicit rounds");
    let strip_benchmark_timings = |text: &str| -> String {
        text.lines()
            .filter(|line| {
                !line.contains("ns/")
                    && !line.contains("throughput")
                    && !line.contains("per second")
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let report_summary_rounds = render_cli(&["report-summary", "--rounds", "1"])
        .expect("report summary should mirror the validation-summary rounds output");
    let validation_report_summary_rounds =
        render_cli(&["validation-report-summary", "--rounds", "1"])
            .expect("validation-report-summary should mirror the validation-summary rounds output");
    assert_eq!(
        strip_benchmark_timings(&validation_summary_rounds),
        strip_benchmark_timings(&report_summary_rounds)
    );
    assert_eq!(
        strip_benchmark_timings(&validation_summary_rounds),
        strip_benchmark_timings(&validation_report_summary_rounds)
    );
    assert_eq!(
        render_cli(&["validation-summary", "extra"]).unwrap_err(),
        "unknown argument: extra"
    );

    let validation_report_summary = render_cli(&["validation-report-summary"])
        .expect("validation-report-summary should render");
    assert!(validation_report_summary.contains("Validation report summary"));
    assert!(validation_report_summary.contains("Comparison corpus"));
    assert!(
        validation_report_summary.contains("Release bundle verification: verify-release-bundle")
    );
    assert!(validation_report_summary
        .contains("Compatibility profile summary: compatibility-profile-summary"));
    assert!(validation_report_summary.contains("Release notes summary: release-notes-summary"));
    assert!(
        validation_report_summary.contains("Release checklist summary: release-checklist-summary")
    );
    assert!(validation_report_summary.contains("Release summary: release-summary"));
    assert!(validation_report_summary.contains(
        &pleiades_validate::reference_snapshot_2451917_major_body_bridge_summary_for_report()
    ));
    assert!(validation_report_summary.contains(
        &pleiades_validate::reference_snapshot_2451917_major_body_boundary_summary_for_report()
    ));
    assert!(validation_report_summary.contains("Comparison tolerance policy: backend family=Composite; scopes=6 (Luminaries, Major planets, Lunar points, Asteroids, Custom bodies, Pluto fallback (approximate)); limits="));
    assert!(validation_report_summary.lines().any(|line| {
        line == format!(
            "Release profile identifiers: v1 compatibility={}, api-stability={}",
            release_profiles.compatibility_profile_id, release_profiles.api_stability_profile_id
        )
    }));
    assert!(validation_report_summary.contains("Benchmark summaries"));
}

#[test]
fn frame_policy_summary_command_renders_the_shared_frame_semantics_block() {
    let rendered =
        render_cli(&["frame-policy-summary"]).expect("frame policy summary should render");
    assert!(rendered.contains("Frame policy summary"));
    assert!(rendered.contains("Frame policy: ecliptic body positions are the default request shape; at the backend boundary equatorial output is derived via mean-obliquity transforms when supported, while the chart layer reports apparent equatorial of date (true obliquity = mean obliquity + nutation-in-obliquity) for release-grade bodies; supported equatorial precision is bounded by the shared mean-obliquity frame round-trip envelope; native sidereal backend output remains unsupported unless a backend explicitly advertises it"));
    assert_eq!(
        render_cli(&["frame-policy"]).expect("frame policy alias should render"),
        rendered
    );

    for (args, expected) in [
        (
            ["time-scale-policy-summary", "extra"],
            "time-scale-policy-summary does not accept extra arguments",
        ),
        (
            ["time-scale-policy", "extra"],
            "time-scale-policy does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy-summary", "extra"],
            "utc-convenience-policy-summary does not accept extra arguments",
        ),
        (
            ["utc-convenience-policy", "extra"],
            "utc-convenience-policy does not accept extra arguments",
        ),
        (
            ["delta-t-policy-summary", "extra"],
            "delta-t-policy-summary does not accept extra arguments",
        ),
        (
            ["delta-t-policy", "extra"],
            "delta-t-policy does not accept extra arguments",
        ),
        (
            ["zodiac-policy-summary", "extra"],
            "zodiac-policy-summary does not accept extra arguments",
        ),
        (
            ["zodiac-policy", "extra"],
            "zodiac-policy does not accept extra arguments",
        ),
        (
            ["observer-policy-summary", "extra"],
            "observer-policy-summary does not accept extra arguments",
        ),
        (
            ["observer-policy", "extra"],
            "observer-policy does not accept extra arguments",
        ),
        (
            ["apparentness-policy-summary", "extra"],
            "apparentness-policy-summary does not accept extra arguments",
        ),
        (
            ["apparentness-policy", "extra"],
            "apparentness-policy does not accept extra arguments",
        ),
        (
            ["native-sidereal-policy-summary", "extra"],
            "native-sidereal-policy-summary does not accept extra arguments",
        ),
        (
            ["native-sidereal-policy", "extra"],
            "native-sidereal-policy does not accept extra arguments",
        ),
        (
            ["frame-policy-summary", "extra"],
            "frame-policy-summary does not accept extra arguments",
        ),
        (
            ["frame-policy", "extra"],
            "frame-policy does not accept extra arguments",
        ),
    ] {
        assert_eq!(
            render_cli(&args).expect_err("policy summary should reject extra arguments"),
            expected
        );
    }
}

#[test]
fn release_profile_identifiers_summary_command_renders_the_shared_release_profile_identifiers_block(
) {
    let rendered = render_cli(&["release-profile-identifiers-summary"])
        .expect("release-profile identifiers summary should render");
    assert!(rendered.contains("Release profile identifiers summary"));
    assert!(rendered.contains("Summary line: v1 compatibility="));
    assert!(rendered.contains("Compatibility profile: "));
    assert!(rendered.contains("API stability posture: "));
    assert_eq!(
        rendered,
        validate_render_cli(&["release-profile-identifiers-summary"]).unwrap()
    );
    assert_eq!(
        render_cli(&["release-profile-identifiers"])
            .expect("release-profile identifiers alias should render"),
        rendered
    );
    assert_eq!(
        render_cli(&["release-profile-identifiers-summary", "extra"])
            .expect_err("release-profile identifiers summary should reject extra arguments"),
        "release-profile-identifiers-summary does not accept extra arguments"
    );
    assert_eq!(
        render_cli(&["release-profile-identifiers", "extra"])
            .expect_err("release-profile identifiers alias should reject extra arguments"),
        "release-profile-identifiers does not accept extra arguments"
    );
}

#[test]
fn validate_eclipses_command_forwards_to_validate_crate() {
    let out = render_cli(&["validate-eclipses"])
        .expect("validate-eclipses should succeed through the pleiades-cli layer");
    assert!(
        out.contains("validate-eclipses"),
        "output should contain 'validate-eclipses': {out}"
    );
    assert!(
        out.contains("NASA-canon"),
        "output should contain 'NASA-canon': {out}"
    );
    // Alias should produce identical output.
    let via_alias = render_cli(&["eclipses-gate"]).expect("eclipses-gate alias should succeed");
    assert_eq!(out, via_alias);
}

#[test]
fn crossings_alias_dispatches_to_validate() {
    let out = render_cli(&["crossings"]).expect("crossings should dispatch");
    // NOTE: the validate layer's `validate-crossings` / `crossings-gate` arm
    // (crates/pleiades-validate/src/render/cli.rs) returns
    // `CrossingsCorpusReport::summary_line()` directly — it has no
    // "Crossings gate:" banner prefix (unlike the plan's original Task 9
    // sketch). Asserting the real output here, mirroring how the existing
    // `validate_eclipses_command_forwards_to_validate_crate` test above
    // checks for the "validate-eclipses" substring rather than an
    // "Eclipses gate" banner.
    assert!(
        out.contains("validate-crossings"),
        "output should contain 'validate-crossings': {out}"
    );
    assert!(
        out.contains("SE crossing fixtures"),
        "output should contain 'SE crossing fixtures': {out}"
    );

    // validate-crossings and crossings-gate should reach the validate layer
    // directly and match the bare "crossings" alias output exactly.
    let via_validate =
        render_cli(&["validate-crossings"]).expect("validate-crossings should succeed");
    assert_eq!(out, via_validate);
    let via_gate = render_cli(&["crossings-gate"]).expect("crossings-gate alias should succeed");
    assert_eq!(out, via_gate);
}

#[test]
fn rise_trans_alias_dispatches_to_validate() {
    let out = render_cli(&["rise-trans"]).expect("rise-trans should dispatch");
    // Mirrors `crossings_alias_dispatches_to_validate` above: the validate
    // layer's `validate-rise-trans` / `rise-trans-gate` arm
    // (crates/pleiades-validate/src/render/cli.rs) returns
    // `RiseTransReport::summary_line()` directly, so we assert on that
    // substring rather than inventing a banner that doesn't exist.
    assert!(
        out.contains("validate-rise-trans"),
        "output should contain 'validate-rise-trans': {out}"
    );
    assert!(
        out.contains("rise-trans + ") && out.contains("azalt SE fixtures"),
        "output should contain the rise-trans/azalt fixture summary: {out}"
    );

    // validate-rise-trans and rise-trans-gate should reach the validate layer
    // directly and match the bare "rise-trans" alias output exactly.
    let via_validate =
        render_cli(&["validate-rise-trans"]).expect("validate-rise-trans should succeed");
    assert_eq!(out, via_validate);
    let via_gate = render_cli(&["rise-trans-gate"]).expect("rise-trans-gate alias should succeed");
    assert_eq!(out, via_gate);
}

#[test]
fn azalt_alias_dispatches_to_the_same_rise_trans_gate() {
    // There is no separate azalt gate: `validate-rise-trans` validates BOTH
    // the rise-trans.csv AND azalt.csv corpora in a single pass (see
    // `crates/pleiades-validate/src/rise_trans_validation.rs`). So the
    // `azalt` alias intentionally routes to the exact same gate as
    // `rise-trans`, not a distinct command.
    let out = render_cli(&["azalt"]).expect("azalt should dispatch");
    assert!(
        out.contains("validate-rise-trans"),
        "output should contain 'validate-rise-trans': {out}"
    );

    let via_rise_trans = render_cli(&["rise-trans"]).expect("rise-trans should dispatch");
    assert_eq!(out, via_rise_trans);
}
