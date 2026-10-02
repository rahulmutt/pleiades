//! README drift checks run by the workspace audit.
//!
//! The READMEs are the first thing a new user reads, on GitHub and on
//! crates.io, and nothing else ties them to the manifests. Two rules keep them
//! from describing an older release than the one published (issue #86):
//!
//! - the workspace README's published and unpublished crate lists must equal
//!   the sets the manifests declare, and
//! - no README may pin a release series such as `0.2.x`, which goes stale at
//!   the next minor release; crates.io shows the current version.

use std::collections::BTreeSet;
use std::path::Path;

use super::workspace_audit::WorkspaceAuditViolation;

const PUBLISHED_CRATES_BLOCK: &str = "audit:published-crates";
const UNPUBLISHED_CRATES_BLOCK: &str = "audit:unpublished-crates";

/// The text between `<!-- {block} -->` and `<!-- /{block} -->`, or `None`
/// when either marker is absent or they are out of order.
fn marked_block<'a>(text: &'a str, block: &str) -> Option<&'a str> {
    let open = format!("<!-- {block} -->");
    let close = format!("<!-- /{block} -->");
    let start = text.find(&open)? + open.len();
    let end = start + text[start..].find(&close)?;
    Some(&text[start..end])
}

/// The `pleiades-*` crate names quoted in backticks in `text`.
fn backticked_crate_names(text: &str) -> BTreeSet<&str> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .filter(|name| name.starts_with("pleiades-"))
        .collect()
}

fn audit_crate_list_block(
    path: &Path,
    text: &str,
    block: &str,
    list_label: &str,
    expected: &[String],
) -> Vec<WorkspaceAuditViolation> {
    let Some(block_text) = marked_block(text, block) else {
        return vec![WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "readme.crate-list-block-missing",
            detail: format!(
                "README must list the {list_label} crates between `<!-- {block} -->` and `<!-- /{block} -->` so the list can be checked against the manifests"
            ),
        }];
    };

    let listed = backticked_crate_names(block_text);
    let expected: BTreeSet<&str> = expected.iter().map(String::as_str).collect();
    let mut violations = Vec::new();

    for name in expected.difference(&listed) {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "readme.crate-list-missing",
            detail: format!(
                "`{name}` is {list_label} according to its manifest but is absent from the README's {list_label} crate list"
            ),
        });
    }
    for name in listed.difference(&expected) {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "readme.crate-list-extra",
            detail: format!(
                "`{name}` is in the README's {list_label} crate list but its manifest does not make it {list_label}"
            ),
        });
    }

    violations
}

/// Checks the workspace README's marked crate lists against the manifests.
///
/// `publishable_names` are the workspace packages without `publish = false`;
/// `unpublished_names` are the ones with it.
pub(crate) fn audit_readme_crate_lists(
    path: &Path,
    text: &str,
    publishable_names: &[String],
    unpublished_names: &[String],
) -> Vec<WorkspaceAuditViolation> {
    let mut violations = audit_crate_list_block(
        path,
        text,
        PUBLISHED_CRATES_BLOCK,
        "published",
        publishable_names,
    );
    violations.extend(audit_crate_list_block(
        path,
        text,
        UNPUBLISHED_CRATES_BLOCK,
        "unpublished",
        unpublished_names,
    ));
    violations
}

/// The release-series literals (`<digits>.<digits>.x`) in one line.
fn release_series_literals(line: &str) -> Vec<&str> {
    let bytes = line.as_bytes();
    let is_word_byte = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    let mut literals = Vec::new();

    for (suffix_start, _) in line.match_indices(".x") {
        let end = suffix_start + 2;
        if bytes.get(end).is_some_and(|byte| is_word_byte(*byte)) {
            continue;
        }

        let digits_before = |mut index: usize| {
            let stop = index;
            while index > 0 && bytes[index - 1].is_ascii_digit() {
                index -= 1;
            }
            (index < stop).then_some(index)
        };
        let Some(minor_start) = digits_before(suffix_start) else {
            continue;
        };
        if minor_start == 0 || bytes[minor_start - 1] != b'.' {
            continue;
        }
        let Some(major_start) = digits_before(minor_start - 1) else {
            continue;
        };
        if major_start > 0
            && (is_word_byte(bytes[major_start - 1]) || bytes[major_start - 1] == b'.')
        {
            continue;
        }

        literals.push(&line[major_start..end]);
    }

    literals
}

/// Flags a README that pins a release series such as `0.2.x`.
pub(crate) fn audit_readme_release_series(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    let mut violations = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for literal in release_series_literals(line) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "readme.pinned-release-series",
                detail: format!(
                    "line {} pins the release series `{literal}`, which goes stale at the next release; describe the maturity without a version and let crates.io show the current one",
                    index + 1
                ),
            });
        }
    }
    violations
}
