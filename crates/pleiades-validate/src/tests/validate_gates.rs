//! CLI dispatch tests for validate-apparent and validate-topocentric goldens gates.

use super::*;

#[test]
fn validate_apparent_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-apparent"])
        .expect("validate-apparent should succeed on committed goldens");
    assert!(
        result.contains("Apparent goldens"),
        "validate-apparent output should contain 'Apparent goldens': {result}"
    );
}

#[test]
fn apparent_gate_alias_matches_validate_apparent() {
    let via_primary = render_cli(&["validate-apparent"]).expect("validate-apparent should succeed");
    let via_alias = render_cli(&["apparent-gate"]).expect("apparent-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_apparent_rejects_extra_args() {
    let error = render_cli(&["validate-apparent", "extra"])
        .expect_err("validate-apparent should reject extra arguments");
    assert!(
        error.contains("validate-apparent does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn apparent_gate_rejects_extra_args() {
    let error = render_cli(&["apparent-gate", "extra"])
        .expect_err("apparent-gate should reject extra arguments");
    assert!(
        error.contains("validate-apparent does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_topocentric_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-topocentric"])
        .expect("validate-topocentric should succeed on committed goldens");
    assert!(
        result.contains("Topocentric goldens"),
        "validate-topocentric output should contain 'Topocentric goldens': {result}"
    );
}

#[test]
fn topocentric_gate_alias_matches_validate_topocentric() {
    let via_primary =
        render_cli(&["validate-topocentric"]).expect("validate-topocentric should succeed");
    let via_alias =
        render_cli(&["topocentric-gate"]).expect("topocentric-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_topocentric_rejects_extra_args() {
    let error = render_cli(&["validate-topocentric", "extra"])
        .expect_err("validate-topocentric should reject extra arguments");
    assert!(
        error.contains("validate-topocentric does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn topocentric_gate_rejects_extra_args() {
    let error = render_cli(&["topocentric-gate", "extra"])
        .expect_err("topocentric-gate should reject extra arguments");
    assert!(
        error.contains("validate-topocentric does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_topocentric_and_validate_apparent() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-topocentric"),
        "help text should mention validate-topocentric"
    );
    assert!(
        help.contains("topocentric-gate"),
        "help text should mention topocentric-gate alias"
    );
    assert!(
        help.contains("validate-apparent"),
        "help text should mention validate-apparent"
    );
    assert!(
        help.contains("apparent-gate"),
        "help text should mention apparent-gate alias"
    );
}

#[test]
fn validate_angles_passes_over_committed_corpus() {
    let report = crate::angles_validation::run_angles_gate();
    assert!(report.passed(), "validate-angles failed: {report:?}");
}

#[test]
fn validate_angles_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-angles"])
        .expect("validate-angles should succeed on committed angles corpus");
    assert!(
        result.contains("Angles gate"),
        "validate-angles output should contain 'Angles gate': {result}"
    );
}

#[test]
fn angles_gate_alias_matches_validate_angles() {
    let via_primary = render_cli(&["validate-angles"]).expect("validate-angles should succeed");
    let via_alias = render_cli(&["angles-gate"]).expect("angles-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_angles_rejects_extra_args() {
    let error = render_cli(&["validate-angles", "extra"])
        .expect_err("validate-angles should reject extra arguments");
    assert!(
        error.contains("validate-angles does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_houses_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-houses"])
        .expect("validate-houses should succeed on committed house corpus");
    assert!(
        result.contains("House gate"),
        "validate-houses output should contain 'House gate': {result}"
    );
}

#[test]
fn houses_gate_alias_matches_validate_houses() {
    let via_primary = render_cli(&["validate-houses"]).expect("validate-houses should succeed");
    let via_alias = render_cli(&["houses-gate"]).expect("houses-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_houses_rejects_extra_args() {
    let error = render_cli(&["validate-houses", "extra"])
        .expect_err("validate-houses should reject extra arguments");
    assert!(
        error.contains("validate-houses does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn houses_gate_rejects_extra_args() {
    let error = render_cli(&["houses-gate", "extra"])
        .expect_err("houses-gate should reject extra arguments");
    assert!(
        error.contains("validate-houses does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_houses_and_houses_gate() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-houses"),
        "help text should mention validate-houses"
    );
    assert!(
        help.contains("houses-gate"),
        "help text should mention houses-gate alias"
    );
}

#[test]
fn validate_ayanamsa_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-ayanamsa"])
        .expect("validate-ayanamsa should succeed on committed ayanamsa corpus");
    assert!(
        result.contains("Ayanamsa gate"),
        "validate-ayanamsa output should contain 'Ayanamsa gate': {result}"
    );
}

#[test]
fn ayanamsa_gate_alias_matches_validate_ayanamsa() {
    let via_primary = render_cli(&["validate-ayanamsa"]).expect("validate-ayanamsa should succeed");
    let via_alias = render_cli(&["ayanamsa-gate"]).expect("ayanamsa-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_ayanamsa_rejects_extra_args() {
    let error = render_cli(&["validate-ayanamsa", "extra"])
        .expect_err("validate-ayanamsa should reject extra arguments");
    assert!(
        error.contains("validate-ayanamsa does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_ayanamsa() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-ayanamsa"),
        "help should mention validate-ayanamsa"
    );
    assert!(
        help.contains("ayanamsa-gate"),
        "help should mention ayanamsa-gate alias"
    );
}

#[test]
fn compat_claims_audit_passes_on_real_catalogs() {
    let out = render_cli(&["compat-claims-audit"]).expect("audit passes");
    assert!(out.contains("OK"));
}

#[test]
fn validate_eclipses_command_reports_a_summary() {
    let out = render_cli(&["validate-eclipses"]).expect("gate passes");
    assert!(
        out.contains("validate-eclipses"),
        "output should contain 'validate-eclipses': {out}"
    );
    assert!(
        out.contains("NASA-canon"),
        "output should contain 'NASA-canon': {out}"
    );
}

#[test]
fn eclipses_gate_alias_matches_validate_eclipses() {
    let via_primary = render_cli(&["validate-eclipses"]).expect("validate-eclipses should succeed");
    let via_alias = render_cli(&["eclipses-gate"]).expect("eclipses-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_eclipses_rejects_extra_args() {
    let error = render_cli(&["validate-eclipses", "extra"])
        .expect_err("validate-eclipses should reject extra arguments");
    assert!(
        error.contains("validate-eclipses does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn validate_crossings_passes_over_committed_corpus() {
    let report = crate::crossings_validation::run_crossings_gate();
    assert!(report.passed(), "validate-crossings failed: {report:?}");
    let checked = report.0.as_ref().expect("gate should pass").checked;
    assert_eq!(checked, 86, "expected all 86 committed fixtures checked");
}

#[test]
fn validate_crossings_command_reports_a_summary() {
    let out = render_cli(&["validate-crossings"]).expect("gate passes");
    assert!(
        out.contains("validate-crossings"),
        "output should contain 'validate-crossings': {out}"
    );
    assert!(
        out.contains("SE crossing fixtures"),
        "output should contain 'SE crossing fixtures': {out}"
    );
}

#[test]
fn crossings_gate_alias_matches_validate_crossings() {
    let via_primary =
        render_cli(&["validate-crossings"]).expect("validate-crossings should succeed");
    let via_alias = render_cli(&["crossings-gate"]).expect("crossings-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_crossings_rejects_extra_args() {
    let error = render_cli(&["validate-crossings", "extra"])
        .expect_err("validate-crossings should reject extra arguments");
    assert!(
        error.contains("validate-crossings does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_crossings() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-crossings"),
        "help text should mention validate-crossings"
    );
    assert!(
        help.contains("crossings-gate"),
        "help text should mention crossings-gate alias"
    );
}

#[test]
fn validate_rise_trans_passes_over_committed_corpus() {
    let report = crate::validate_rise_trans_corpus()
        .expect("validate-rise-trans should pass on the committed corpus");
    assert!(report.passed(), "validate-rise-trans failed: {report:?}");
}

#[test]
fn validate_rise_trans_command_reports_a_summary() {
    let out = render_cli(&["validate-rise-trans"]).expect("gate passes");
    assert!(
        out.contains("validate-rise-trans"),
        "output should contain 'validate-rise-trans': {out}"
    );
    assert!(
        out.contains("rise-trans + ") && out.contains("azalt SE fixtures"),
        "output should contain the rise-trans/azalt fixture summary: {out}"
    );
}

#[test]
fn rise_trans_gate_alias_matches_validate_rise_trans() {
    let via_primary =
        render_cli(&["validate-rise-trans"]).expect("validate-rise-trans should succeed");
    let via_alias = render_cli(&["rise-trans-gate"]).expect("rise-trans-gate alias should succeed");
    assert_eq!(via_primary, via_alias);
}

#[test]
fn validate_rise_trans_rejects_extra_args() {
    let error = render_cli(&["validate-rise-trans", "extra"])
        .expect_err("validate-rise-trans should reject extra arguments");
    assert!(
        error.contains("validate-rise-trans does not accept extra arguments"),
        "unexpected error: {error}"
    );
}

#[test]
fn help_text_mentions_validate_rise_trans() {
    let help = render_cli(&["help"]).expect("help command should render");
    assert!(
        help.contains("validate-rise-trans"),
        "help text should mention validate-rise-trans"
    );
    assert!(
        help.contains("rise-trans-gate"),
        "help text should mention rise-trans-gate alias"
    );
}

#[test]
fn eclipses_listing_returns_lines_for_narrow_window() {
    // The allowlisted eclipse JD 2432680.601 (1948-05-09 solar, Saros 137) is
    // guaranteed to be in the corpus window and returns exactly one eclipse.
    let out = render_cli(&["eclipses", "--start", "2432679.0", "--end", "2432682.0"])
        .expect("eclipses listing should succeed for a narrow window");
    // There should be at least one line.
    assert!(!out.is_empty(), "eclipses listing should not be empty");
    // The eclipse kind and type are in the output.
    assert!(
        out.contains("solar"),
        "output should mention solar eclipse: {out}"
    );
}

#[test]
fn eclipses_listing_with_end_at_window_boundary_does_not_error() {
    // `render_eclipses_listing` with no --end defaults to WINDOW_END_JD.
    // Before the `eclipses_in_range` scan-end clamp, this triggered a backend
    // OutOfRange error because the syzygy scanner probed one STEP_DAYS past the
    // data bound. This test exercises that exact path via the CLI surface.
    let out = render_cli(&["eclipses", "--start", "2488000.0"])
        .expect("eclipses listing with default end (WINDOW_END_JD) must not error");
    // The window has eclipses in this region.
    assert!(
        !out.is_empty(),
        "eclipses listing near window end should not be empty"
    );
}

#[test]
fn validate_equatorial_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-equatorial"])
        .expect("validate-equatorial should succeed on committed goldens");
    assert!(
        result.contains("Equatorial goldens"),
        "validate-equatorial output should contain 'Equatorial goldens': {result}"
    );
}

#[test]
fn equatorial_gate_alias_matches_validate_equatorial() {
    let primary = render_cli(&["validate-equatorial"]).expect("validate-equatorial should succeed");
    let alias = render_cli(&["equatorial-gate"]).expect("equatorial-gate should succeed");
    assert_eq!(primary, alias);
}

#[test]
fn validate_equatorial_rejects_extra_args() {
    let error = render_cli(&["validate-equatorial", "extra"])
        .expect_err("validate-equatorial should reject extra arguments");
    assert!(
        error.contains("validate-equatorial does not accept extra arguments"),
        "{error}"
    );
}

#[test]
fn validate_equatorial_se_command_dispatches_and_reports_pass() {
    let result = render_cli(&["validate-equatorial-se"])
        .expect("validate-equatorial-se should succeed on committed corpus");
    assert!(
        result.contains("Equatorial-SE parity"),
        "validate-equatorial-se output should contain 'Equatorial-SE parity': {result}"
    );
}

#[test]
fn help_text_mentions_validate_equatorial() {
    let help = render_cli(&["help"]).expect("help should render");
    assert!(
        help.contains("validate-equatorial"),
        "help should mention validate-equatorial"
    );
    assert!(
        help.contains("validate-equatorial-se"),
        "help should mention validate-equatorial-se"
    );
}

#[test]
fn validate_fictitious_and_alias_agree_and_reject_extra_args() {
    let primary = render_cli(&["validate-fictitious"]).expect("gate ok");
    let alias = render_cli(&["fictitious-gate"]).expect("alias ok");
    assert_eq!(primary, alias);
    let error = render_cli(&["validate-fictitious", "extra"])
        .expect_err("validate-fictitious should reject extra arguments");
    assert!(
        error.contains("validate-fictitious does not accept extra arguments"),
        "unexpected error: {error}"
    );
    let help = render_cli(&["help"]).expect("help ok");
    assert!(
        help.contains("validate-fictitious"),
        "help text should mention validate-fictitious"
    );
    assert!(
        help.contains("fictitious-gate"),
        "help text should mention fictitious-gate alias"
    );
}

#[test]
fn validate_nod_aps_and_alias_agree_and_reject_extra_args() {
    let primary = render_cli(&["validate-nod-aps"]).expect("gate ok");
    let alias = render_cli(&["nod-aps-gate"]).expect("alias ok");
    assert_eq!(primary, alias);
    let error = render_cli(&["validate-nod-aps", "extra"])
        .expect_err("validate-nod-aps should reject extra arguments");
    assert!(
        error.contains("validate-nod-aps does not accept extra arguments"),
        "unexpected error: {error}"
    );
    let help = render_cli(&["help"]).expect("help ok");
    assert!(
        help.contains("validate-nod-aps"),
        "help text should mention validate-nod-aps"
    );
    assert!(
        help.contains("nod-aps-gate"),
        "help text should mention nod-aps-gate alias"
    );
}
