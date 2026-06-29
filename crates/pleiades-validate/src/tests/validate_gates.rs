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
