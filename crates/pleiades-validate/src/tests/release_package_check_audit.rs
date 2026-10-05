//! `package-check` crate-list drift check of the workspace audit (issue #166).

use super::*;
use std::path::Path;

const MISE: &str = "[tasks.audit]\nrun = \"cargo run -q -p pleiades-validate -- workspace-audit\"\n\n[tasks.package-check]\nshell = \"bash -c\"\nrun = '''\nset -euo pipefail\ncrates=\"pleiades-types pleiades-core\"\nfor crate in $crates; do\n  echo \"$crate\"\ndone\n'''\n\n[tasks.benchmark]\nrun = \"true\"\n";

fn audit(mise: &str, publishable: &[&str]) -> Vec<WorkspaceAuditViolation> {
    let publishable: Vec<String> = publishable.iter().map(|name| name.to_string()).collect();
    audit_package_check_crate_list(Path::new("/tmp/mise.toml"), mise, &publishable)
}

fn rules(violations: &[WorkspaceAuditViolation]) -> Vec<&'static str> {
    violations.iter().map(|violation| violation.rule).collect()
}

#[test]
fn package_check_audit_accepts_a_matching_list_in_any_order() {
    assert_eq!(
        audit(MISE, &["pleiades-core", "pleiades-types"]),
        Vec::new()
    );
}

#[test]
fn package_check_audit_flags_a_publishable_crate_missing_from_the_list() {
    let violations = audit(
        MISE,
        &["pleiades-core", "pleiades-events", "pleiades-types"],
    );
    assert_eq!(rules(&violations), ["package-check.crate-missing"]);
    assert_eq!(violations[0].path, Path::new("/tmp/mise.toml"));
    assert!(violations[0].detail.contains("`pleiades-events`"));
}

#[test]
fn package_check_audit_flags_a_listed_crate_that_is_not_publishable() {
    let violations = audit(MISE, &["pleiades-types"]);
    assert_eq!(rules(&violations), ["package-check.crate-extra"]);
    assert!(violations[0].detail.contains("`pleiades-core`"));
}

#[test]
fn package_check_audit_flags_a_missing_task_or_list_line() {
    let no_task = MISE.replace("[tasks.package-check]", "[tasks.package-size]");
    let no_line = MISE.replace("crates=\"", "packages=\"");
    for mise in [no_task, no_line] {
        let violations = audit(&mise, &["pleiades-core", "pleiades-types"]);
        assert_eq!(rules(&violations), ["package-check.crate-list-missing"]);
    }
}

#[test]
fn package_check_audit_reads_the_list_from_its_own_task_only() {
    // A `crates="..."` line in another task is not the package-check list.
    let elsewhere = MISE.replace("[tasks.benchmark]\nrun = \"true\"\n", "")
        + "[tasks.other]\nrun = '''\ncrates=\"pleiades-cli\"\n'''\n";
    assert_eq!(
        audit(&elsewhere, &["pleiades-core", "pleiades-types"]),
        Vec::new()
    );
    let only_elsewhere = elsewhere.replace("crates=\"pleiades-types pleiades-core\"\n", "");
    let violations = audit(&only_elsewhere, &["pleiades-core", "pleiades-types"]);
    assert_eq!(rules(&violations), ["package-check.crate-list-missing"]);
}

#[test]
fn package_check_audit_flags_an_unterminated_list() {
    let unterminated = MISE.replace("pleiades-core\"\n", "pleiades-core\n");
    let violations = audit(&unterminated, &["pleiades-core", "pleiades-types"]);
    assert_eq!(rules(&violations), ["package-check.crate-list-missing"]);
}
