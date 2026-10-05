//! Fuzz nightly drift check of the workspace audit (issue #30, item 5).

use super::*;
use std::path::Path;

const MISE: &str = "[env]\n# The dated fuzzing nightly.\nFUZZ_NIGHTLY = \"nightly-2026-10-01\"\n\n[tools]\nrust = { version = \"1.99.0\", components = \"rustfmt,clippy\" }\n";
const TOOLCHAIN: &str =
    "# Keep identical to FUZZ_NIGHTLY.\n[toolchain]\nchannel = \"nightly-2026-10-01\"\n";

fn audit(mise: &str, toolchain: &str) -> Vec<WorkspaceAuditViolation> {
    audit_fuzz_nightly_text(
        Path::new("/tmp/mise.toml"),
        mise,
        Path::new("/tmp/fuzz/rust-toolchain.toml"),
        toolchain,
    )
}

fn rules(violations: &[WorkspaceAuditViolation]) -> Vec<&'static str> {
    violations.iter().map(|violation| violation.rule).collect()
}

#[test]
fn fuzz_toolchain_audit_accepts_matching_dates() {
    assert_eq!(audit(MISE, TOOLCHAIN), Vec::new());
}

#[test]
fn fuzz_toolchain_audit_flags_a_date_bumped_on_one_side_only() {
    let bumped = MISE.replace("2026-10-01", "2026-11-01");
    let violations = audit(&bumped, TOOLCHAIN);
    assert_eq!(rules(&violations), ["fuzz-toolchain.nightly-drift"]);
    assert_eq!(
        violations[0].path,
        Path::new("/tmp/fuzz/rust-toolchain.toml")
    );
    assert!(violations[0].detail.contains("nightly-2026-10-01"));
    assert!(violations[0].detail.contains("nightly-2026-11-01"));
}

#[test]
fn fuzz_toolchain_audit_flags_a_missing_mise_pin() {
    // A `FUZZ_NIGHTLY` outside `[env]` is not the pin the tasks consume.
    let misplaced = "[tools]\nFUZZ_NIGHTLY = \"nightly-2026-10-01\"\n";
    let violations = audit(misplaced, TOOLCHAIN);
    assert_eq!(rules(&violations), ["fuzz-toolchain.nightly-missing"]);
    assert_eq!(violations[0].path, Path::new("/tmp/mise.toml"));
}

#[test]
fn fuzz_toolchain_audit_flags_a_missing_channel() {
    let violations = audit(MISE, "[toolchain]\nprofile = \"minimal\"\n");
    assert_eq!(rules(&violations), ["fuzz-toolchain.channel-missing"]);
    assert_eq!(
        violations[0].path,
        Path::new("/tmp/fuzz/rust-toolchain.toml")
    );
}

#[test]
fn fuzz_toolchain_audit_ignores_comments_that_quote_the_keys() {
    let mise =
        "[env]\n# FUZZ_NIGHTLY = \"nightly-1999-01-01\"\nFUZZ_NIGHTLY = \"nightly-2026-10-01\"\n";
    let toolchain = "[toolchain]\n# channel = \"nightly-1999-01-01\"\nchannel = \"nightly-2026-10-01\" # dated\n";
    assert_eq!(audit(mise, toolchain), Vec::new());
}

#[test]
fn the_workspace_pins_one_fuzz_nightly() {
    // The real files, through the report `mise run audit` renders.
    let report = workspace_audit_report().expect("workspace audit should render");
    assert!(
        !report
            .violations
            .iter()
            .any(|violation| violation.rule.starts_with("fuzz-toolchain.")),
        "{report}"
    );
}
