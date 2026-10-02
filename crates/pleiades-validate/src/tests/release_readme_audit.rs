//! README drift checks of the workspace audit (issue #86).

use super::*;
use std::path::Path;

fn names(entries: &[&str]) -> Vec<String> {
    entries.iter().map(|name| name.to_string()).collect()
}

const CLEAN_README: &str = "# pleiades\n\n## Published crates\n\n<!-- audit:published-crates -->\n`pleiades-types`, `pleiades-core`\n<!-- /audit:published-crates -->\nare published under `MIT OR Apache-2.0`.\n\n<!-- audit:unpublished-crates -->\n`pleiades-cli` stays unpublished.\n<!-- /audit:unpublished-crates -->\n\n## Next\n";

#[test]
fn readme_audit_accepts_a_matching_crate_list() {
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        CLEAN_README,
        &names(&["pleiades-core", "pleiades-types"]),
        &names(&["pleiades-cli"]),
    );
    assert_eq!(violations, Vec::new());
}

#[test]
fn readme_audit_flags_a_publishable_crate_missing_from_the_list() {
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        CLEAN_README,
        &names(&["pleiades-core", "pleiades-events", "pleiades-types"]),
        &names(&["pleiades-cli"]),
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "readme.crate-list-missing");
    assert!(violations[0].detail.contains("`pleiades-events`"));
    assert!(violations[0].detail.contains("published"));
}

#[test]
fn readme_audit_flags_an_unpublished_crate_listed_as_published() {
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        CLEAN_README,
        &names(&["pleiades-types"]),
        &names(&["pleiades-cli", "pleiades-core"]),
    );
    let rules: Vec<_> = violations.iter().map(|violation| violation.rule).collect();
    // `pleiades-core` is extra in the published block and missing from the
    // unpublished block.
    assert_eq!(
        rules,
        vec!["readme.crate-list-extra", "readme.crate-list-missing"]
    );
    assert!(violations
        .iter()
        .all(|violation| violation.detail.contains("`pleiades-core`")));
}

#[test]
fn readme_audit_flags_a_published_crate_listed_as_unpublished() {
    // The issue #86 defect: `pleiades-data` described as unpublished tooling.
    let readme = CLEAN_README.replace(
        "`pleiades-cli` stays",
        "`pleiades-cli` and `pleiades-data` stay",
    );
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        &readme,
        &names(&["pleiades-core", "pleiades-data", "pleiades-types"]),
        &names(&["pleiades-cli"]),
    );
    let rules: Vec<_> = violations.iter().map(|violation| violation.rule).collect();
    assert_eq!(
        rules,
        vec!["readme.crate-list-missing", "readme.crate-list-extra"]
    );
}

#[test]
fn readme_audit_flags_missing_crate_list_markers() {
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        "# pleiades\n\nThe crates `pleiades-types` and `pleiades-core` are published.\n",
        &names(&["pleiades-core", "pleiades-types"]),
        &names(&["pleiades-cli"]),
    );
    let rules: Vec<_> = violations.iter().map(|violation| violation.rule).collect();
    assert_eq!(
        rules,
        vec![
            "readme.crate-list-block-missing",
            "readme.crate-list-block-missing"
        ]
    );
}

#[test]
fn readme_audit_flags_an_unterminated_crate_list_block() {
    let readme = CLEAN_README.replace("<!-- /audit:published-crates -->\n", "");
    let violations = audit_readme_crate_lists(
        Path::new("/tmp/README.md"),
        &readme,
        &names(&["pleiades-core", "pleiades-types"]),
        &names(&["pleiades-cli"]),
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].rule, "readme.crate-list-block-missing");
}

#[test]
fn readme_audit_flags_a_pinned_release_series() {
    let violations = audit_readme_release_series(
        Path::new("/tmp/README.md"),
        "# crate\n\n## Status\n\nExperimental `0.2.x`. More text.\nAlso 12.34.x here.\n",
    );
    assert_eq!(violations.len(), 2);
    assert!(violations
        .iter()
        .all(|violation| violation.rule == "readme.pinned-release-series"));
    assert!(violations[0].detail.contains("`0.2.x`"));
    assert!(violations[0].detail.contains("line 5"));
    assert!(violations[1].detail.contains("`12.34.x`"));
    assert!(violations[1].detail.contains("line 6"));
}

#[test]
fn readme_audit_ignores_text_that_is_not_a_release_series() {
    let violations = audit_readme_release_series(
        Path::new("/tmp/README.md"),
        "Rust 1.99.0, `position.x`, v0.x, 1.2.xml, a1.2.x, 3.1.2.x and 0.6.0 are fine.\n",
    );
    assert_eq!(violations, Vec::new());
}
