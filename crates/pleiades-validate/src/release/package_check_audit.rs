//! `package-check` crate-list drift check run by the workspace audit.
//!
//! The `package-check` task in `mise.toml` packages every publishable crate
//! and holds each artifact to a size budget. Its `crates="..."` shell list is
//! a hand-kept copy of the publishable set: a crate added to the workspace
//! and left out of it is published without its size ever being checked. This
//! rule keeps the list equal to the set the manifests declare (issue #166).

use std::collections::BTreeSet;
use std::path::Path;

use super::workspace_audit::WorkspaceAuditViolation;

const TASK_HEADER: &str = "[tasks.package-check]";
const LIST_PREFIX: &str = "crates=\"";

/// The crate names in the `crates="..."` line of the `package-check` task, or
/// `None` when the task or the line is absent.
fn package_check_crates(tool_manifest_text: &str) -> Option<BTreeSet<&str>> {
    let mut in_task = false;
    for line in tool_manifest_text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_task = line == TASK_HEADER;
            continue;
        }
        if !in_task {
            continue;
        }
        if let Some(list) = line.strip_prefix(LIST_PREFIX) {
            let (names, _) = list.split_once('"')?;
            return Some(names.split_whitespace().collect());
        }
    }
    None
}

/// Violations when the `package-check` task's crate list differs from
/// `publishable_names`, the workspace packages without `publish = false`.
pub(crate) fn audit_package_check_crate_list(
    tool_manifest_path: &Path,
    tool_manifest_text: &str,
    publishable_names: &[String],
) -> Vec<WorkspaceAuditViolation> {
    let violation = |rule, detail| WorkspaceAuditViolation {
        path: tool_manifest_path.to_path_buf(),
        rule,
        detail,
    };
    let Some(listed) = package_check_crates(tool_manifest_text) else {
        return vec![violation(
            "package-check.crate-list-missing",
            format!(
                "mise.toml must list the publishable crates on a `{LIST_PREFIX}...\"` line under `{TASK_HEADER}` so the list can be checked against the manifests"
            ),
        )];
    };
    let expected: BTreeSet<&str> = publishable_names.iter().map(String::as_str).collect();

    let missing = expected.difference(&listed).map(|name| {
        violation(
            "package-check.crate-missing",
            format!(
                "`{name}` is publishable according to its manifest but is absent from the package-check crate list, so its artifact size is not checked"
            ),
        )
    });
    let extra = listed.difference(&expected).map(|name| {
        violation(
            "package-check.crate-extra",
            format!(
                "`{name}` is in the package-check crate list but is not a publishable workspace crate"
            ),
        )
    });
    missing.chain(extra).collect()
}
