//! Fuzz nightly drift check run by the workspace audit.
//!
//! The dated nightly the fuzz targets build with is written twice:
//! `FUZZ_NIGHTLY` under `[env]` in `mise.toml`, which the mise tasks and CI
//! consume, and `channel` in `fuzz/rust-toolchain.toml`, which rustup reads
//! for a bare `cargo fuzz` or an IDE. mise cannot template a file rustup
//! reads directly, so the two are kept equal by this rule instead (issue #30).

use std::path::Path;

use super::workspace_audit::WorkspaceAuditViolation;

/// The quoted string assigned to `key` inside `[section]`, ignoring comments.
fn section_string<'a>(text: &'a str, section: &str, key: &str) -> Option<&'a str> {
    let header = format!("[{section}]");
    let mut in_section = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == header;
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        if name.trim() != key {
            continue;
        }
        let value = value.trim().strip_prefix('"')?;
        return value.split_once('"').map(|(string, _)| string);
    }
    None
}

/// Violations when `fuzz/rust-toolchain.toml` does not name the nightly that
/// `mise.toml` pins as `FUZZ_NIGHTLY`.
pub(crate) fn audit_fuzz_nightly_text(
    tool_manifest_path: &Path,
    tool_manifest_text: &str,
    toolchain_path: &Path,
    toolchain_text: &str,
) -> Vec<WorkspaceAuditViolation> {
    let Some(pinned) = section_string(tool_manifest_text, "env", "FUZZ_NIGHTLY") else {
        return vec![WorkspaceAuditViolation {
            path: tool_manifest_path.to_path_buf(),
            rule: "fuzz-toolchain.nightly-missing",
            detail: "mise.toml does not set FUZZ_NIGHTLY under [env], but fuzz/rust-toolchain.toml exists".to_string(),
        }];
    };
    let Some(channel) = section_string(toolchain_text, "toolchain", "channel") else {
        return vec![WorkspaceAuditViolation {
            path: toolchain_path.to_path_buf(),
            rule: "fuzz-toolchain.channel-missing",
            detail: "fuzz/rust-toolchain.toml does not set channel under [toolchain]".to_string(),
        }];
    };
    if channel == pinned {
        return Vec::new();
    }
    vec![WorkspaceAuditViolation {
        path: toolchain_path.to_path_buf(),
        rule: "fuzz-toolchain.nightly-drift",
        detail: format!(
            "fuzz/rust-toolchain.toml pins channel {channel}, but mise.toml sets FUZZ_NIGHTLY to {pinned}; bump both together"
        ),
    }]
}
