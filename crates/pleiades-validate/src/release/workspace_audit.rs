//! Workspace audit report types and their compact summaries.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::*;

use pleiades_core::{EphemerisError, EphemerisErrorKind};

/// A deterministic workspace audit that checks for mandatory native build hooks
/// in the first-party crates, lockfile, and pinned tooling manifest, plus
/// publish metadata for publishable crates.
#[derive(Clone, Debug)]
pub struct WorkspaceAuditReport {
    /// Workspace root used for the scan.
    pub workspace_root: PathBuf,
    /// Workspace manifest files that were checked.
    pub manifest_paths: Vec<PathBuf>,
    /// Workspace tool manifest path that was checked.
    pub tool_manifest_path: PathBuf,
    /// Workspace lockfile path that was checked.
    pub lockfile_path: PathBuf,
    /// Detected policy violations.
    pub violations: Vec<WorkspaceAuditViolation>,
}

/// A single workspace-audit finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceAuditViolation {
    /// File that triggered the finding.
    pub path: PathBuf,
    /// Stable rule identifier for the finding.
    pub rule: &'static str,
    /// Human-readable explanation of the finding.
    pub detail: String,
}

impl WorkspaceAuditReport {
    /// Returns whether the workspace passed the audit cleanly.
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

/// A compact workspace-audit summary derived from the detailed report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceAuditSummary {
    /// Workspace root used for the scan.
    pub workspace_root: PathBuf,
    /// Number of manifests that were checked.
    pub manifest_count: usize,
    /// Workspace tool manifest path that was checked.
    pub tool_manifest_path: PathBuf,
    /// Workspace lockfile path that was checked.
    pub lockfile_path: PathBuf,
    /// Number of detected policy violations.
    pub violation_count: usize,
    /// Policy-violation counts grouped by rule in stable order.
    pub rule_counts: Vec<(&'static str, usize)>,
    /// Whether the workspace passed the audit cleanly.
    pub clean: bool,
}

impl WorkspaceAuditSummary {
    /// Validates the compact summary before it is rendered.
    pub fn validate(&self) -> Result<(), EphemerisError> {
        let mut counted_violations = 0usize;
        let mut seen_rules = BTreeSet::new();

        for (rule, count) in &self.rule_counts {
            if rule.trim().is_empty() {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "workspace audit summary contains a blank rule label".to_string(),
                ));
            }
            if *count == 0 {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!("workspace audit summary rule '{}' has a zero count", rule),
                ));
            }
            if !seen_rules.insert(*rule) {
                return Err(EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    format!(
                        "workspace audit summary contains a duplicate rule count for '{}'",
                        rule
                    ),
                ));
            }
            counted_violations = counted_violations.checked_add(*count).ok_or_else(|| {
                EphemerisError::new(
                    EphemerisErrorKind::InvalidRequest,
                    "workspace audit summary rule counts overflowed the aggregate violation count"
                        .to_string(),
                )
            })?;
        }

        if counted_violations != self.violation_count {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "workspace audit summary violation count mismatch: expected {}, found {}",
                    counted_violations, self.violation_count
                ),
            ));
        }

        if self.clean != (self.violation_count == 0) {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                format!(
                    "workspace audit summary clean flag mismatch: clean={}, violations={}",
                    self.clean, self.violation_count
                ),
            ));
        }

        if self.clean && !self.rule_counts.is_empty() {
            return Err(EphemerisError::new(
                EphemerisErrorKind::InvalidRequest,
                "workspace audit summary marked clean but still carries rule counts".to_string(),
            ));
        }

        Ok(())
    }

    /// Returns a compact summary line used in release-facing reporting.
    pub fn summary_line(&self) -> String {
        let mut text = format!(
            "workspace root: {}; manifests checked: {}; tool manifest: {}; lockfile: {}; violations: {}; result: {}",
            self.workspace_root.display(),
            self.manifest_count,
            self.tool_manifest_path.display(),
            self.lockfile_path.display(),
            self.violation_count,
            if self.clean {
                "no workspace policy violations detected"
            } else {
                "violations found"
            }
        );
        if !self.rule_counts.is_empty() {
            text.push_str("; rule counts: ");
            text.push_str(
                &self
                    .rule_counts
                    .iter()
                    .map(|(rule, count)| format!("{}: {}", rule, count))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
        }
        text
    }

    /// Validates and returns the compact summary line.
    pub fn validated_summary_line(&self) -> Result<String, EphemerisError> {
        self.validate()?;
        Ok(self.summary_line())
    }
}

impl fmt::Display for WorkspaceAuditSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

/// Returns the compact workspace-audit summary derived from the detailed report.
pub fn workspace_audit_summary(report: &WorkspaceAuditReport) -> WorkspaceAuditSummary {
    WorkspaceAuditSummary {
        workspace_root: report.workspace_root.clone(),
        manifest_count: report.manifest_paths.len(),
        tool_manifest_path: report.tool_manifest_path.clone(),
        lockfile_path: report.lockfile_path.clone(),
        violation_count: report.violations.len(),
        rule_counts: workspace_audit_rule_counts(&report.violations)
            .into_iter()
            .collect(),
        clean: report.is_clean(),
    }
}

fn workspace_audit_rule_counts(
    violations: &[WorkspaceAuditViolation],
) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for violation in violations {
        *counts.entry(violation.rule).or_insert(0) += 1;
    }
    counts
}

impl fmt::Display for WorkspaceAuditReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Workspace audit")?;
        writeln!(f, "Workspace root: {}", self.workspace_root.display())?;
        writeln!(f, "Checked manifests: {}", self.manifest_paths.len())?;
        writeln!(
            f,
            "Checked tool manifest: {}",
            self.tool_manifest_path.display()
        )?;
        writeln!(f, "Checked lockfile: {}", self.lockfile_path.display())?;
        if self.violations.is_empty() {
            writeln!(f, "Result: no workspace policy violations detected")?;
            return Ok(());
        }

        writeln!(f, "Result: violations found")?;
        writeln!(f, "Rule counts:")?;
        for (rule, count) in workspace_audit_rule_counts(&self.violations) {
            writeln!(f, "  {}: {}", rule, count)?;
        }
        for violation in &self.violations {
            writeln!(
                f,
                "- {} [{}]: {}",
                violation.path.display(),
                violation.rule,
                violation.detail
            )?;
        }
        Ok(())
    }
}

pub(crate) fn render_workspace_audit_summary_text(report: &WorkspaceAuditReport) -> String {
    let summary = workspace_audit_summary(report);
    match summary.validated_summary_line() {
        Ok(summary_line) => {
            let mut text = String::new();
            text.push_str("Workspace audit summary\n");
            text.push_str("Workspace root: ");
            text.push_str(&summary.workspace_root.display().to_string());
            text.push('\n');
            text.push_str("Checked manifests: ");
            text.push_str(&summary.manifest_count.to_string());
            text.push('\n');
            text.push_str("Checked tool manifest: ");
            text.push_str(&summary.tool_manifest_path.display().to_string());
            text.push('\n');
            text.push_str("Checked lockfile: ");
            text.push_str(&summary.lockfile_path.display().to_string());
            text.push('\n');
            text.push_str("Summary: ");
            text.push_str(&summary_line);
            text.push('\n');
            text.push_str("Violations: ");
            text.push_str(&summary.violation_count.to_string());
            text.push('\n');
            if !summary.clean {
                text.push_str("Rule counts:\n");
                for (rule, count) in &summary.rule_counts {
                    text.push_str("  ");
                    text.push_str(rule);
                    text.push_str(": ");
                    text.push_str(&count.to_string());
                    text.push('\n');
                }
            }
            text.push_str("Result: ");
            text.push_str(if summary.clean {
                "no workspace policy violations detected"
            } else {
                "violations found"
            });
            text.push('\n');
            text
        }
        Err(error) => format!("Workspace audit summary unavailable ({error})"),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn workspace_manifest_paths(root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut manifests = vec![root.join("Cargo.toml")];
    let crates_dir = root.join("crates");
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let manifest = entry.path().join("Cargo.toml");
        if manifest.is_file() {
            manifests.push(manifest);
        }
    }
    manifests.sort();
    Ok(manifests)
}

fn manifest_has_assignment(line: &str, key: &str) -> bool {
    let Some(rest) = line.strip_prefix(key) else {
        return false;
    };
    rest.trim_start().starts_with('=')
}

fn manifest_dependency_rule(line: &str, forbidden: &str) -> bool {
    if manifest_has_assignment(line, forbidden) {
        return true;
    }

    line.contains(&format!("package = \"{forbidden}\""))
}

fn manifest_dependency_name(line: &str) -> Option<&str> {
    let (name, _) = line.split_once('=')?;
    let name = name.trim().trim_matches('"');
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn manifest_dependency_package_name(line: &str) -> Option<&str> {
    let needle = "package = \"";
    let start = line.find(needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    let package_name = &rest[..end];
    if package_name.is_empty() {
        None
    } else {
        Some(package_name)
    }
}

fn workspace_rust_version(root: &Path) -> Option<String> {
    let text = fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let mut in_workspace_package = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_package = trimmed == "[workspace.package]";
            continue;
        }

        if in_workspace_package && trimmed.starts_with("rust-version") {
            let (_, value) = trimmed.split_once('=')?;
            let value = value.trim().trim_matches('"');
            if value.is_empty() {
                return None;
            }
            return Some(value.to_string());
        }
    }

    None
}

fn extract_inline_table_string<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{key} = \"");
    let start = text.find(&needle)? + needle.len();
    let rest = &text[start..];
    let end = rest.find('"')?;
    let value = &rest[..end];
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(crate) fn audit_manifest_text(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        Package,
        Dependencies,
    }

    const FORBIDDEN_DEPENDENCIES: [&str; 4] = ["cc", "bindgen", "cmake", "pkg-config"];

    let mut section = Section::Other;
    let mut violations = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = if line == "[package]" {
                Section::Package
            } else if line == "[dependencies]"
                || line == "[dev-dependencies]"
                || line == "[build-dependencies]"
                || line.contains(".dependencies]")
            {
                Section::Dependencies
            } else {
                Section::Other
            };
            continue;
        }

        match section {
            Section::Package => {
                if manifest_has_assignment(line, "build") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "package.build",
                        detail: "package declares a build script, which violates the pure-Rust workspace policy".to_string(),
                    });
                }
                if manifest_has_assignment(line, "links") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "package.links",
                        detail: "package declares a native links value, which indicates an external build requirement".to_string(),
                    });
                }
            }
            Section::Dependencies => {
                if let Some(native_package_name) = manifest_dependency_name(line)
                    .filter(|name| name.ends_with("-sys"))
                    .or_else(|| {
                        manifest_dependency_package_name(line).filter(|name| name.ends_with("-sys"))
                    })
                {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "dependency.native-package",
                        detail: format!(
                            "dependency table references `{native_package_name}`, which suggests a native build dependency"
                        ),
                    });
                }

                for forbidden in FORBIDDEN_DEPENDENCIES {
                    if manifest_dependency_rule(line, forbidden) {
                        violations.push(WorkspaceAuditViolation {
                            path: path.to_path_buf(),
                            rule: "dependency.native-tool",
                            detail: format!(
                                "dependency table references `{forbidden}`, which is reserved for native build tooling"
                            ),
                        });
                    }
                }
            }
            Section::Other => {}
        }
    }

    violations
}

pub(crate) fn audit_tool_manifest_text(
    path: &Path,
    text: &str,
    workspace_rust_version: Option<String>,
) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        Tools,
    }

    let mut section = Section::Other;
    let mut violations = Vec::new();
    let mut saw_tools_section = false;
    let mut saw_rust_entry = false;
    let mut saw_rustfmt = false;
    let mut saw_clippy = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = if trimmed == "[tools]" {
                saw_tools_section = true;
                Section::Tools
            } else {
                Section::Other
            };
            continue;
        }

        if section != Section::Tools {
            continue;
        }

        if trimmed.starts_with("rust =") {
            saw_rust_entry = true;
            let Some((_, value)) = trimmed.split_once('=') else {
                violations.push(WorkspaceAuditViolation {
                    path: path.to_path_buf(),
                    rule: "tool-manifest.rust-entry-invalid",
                    detail: "mise.toml rust tool entry is malformed".to_string(),
                });
                continue;
            };
            let value = value.trim();
            if !value.starts_with('{') || !value.ends_with('}') {
                violations.push(WorkspaceAuditViolation {
                    path: path.to_path_buf(),
                    rule: "tool-manifest.rust-entry-invalid",
                    detail: "mise.toml rust tool entry must use an inline table".to_string(),
                });
                continue;
            }

            let rust_version = extract_inline_table_string(value, "version");
            let components = extract_inline_table_string(value, "components");

            match rust_version {
                Some(version) => match workspace_rust_version.as_deref() {
                    Some(expected) if expected != version => violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "tool-manifest.rust-version-mismatch",
                        detail: format!(
                            "mise.toml pins rust {version}, but workspace Cargo.toml declares rust-version {expected}"
                        ),
                    }),
                    Some(_) => {}
                    None => violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "tool-manifest.workspace-rust-version-missing",
                        detail: "workspace Cargo.toml does not declare a rust-version to compare against the pinned toolchain".to_string(),
                    }),
                },
                None => violations.push(WorkspaceAuditViolation {
                    path: path.to_path_buf(),
                    rule: "tool-manifest.rust-version-missing",
                    detail: "mise.toml rust tool entry does not declare a version".to_string(),
                }),
            }

            if let Some(components) = components {
                saw_rustfmt |= components
                    .split(',')
                    .map(|item| item.trim())
                    .any(|item| item == "rustfmt");
                saw_clippy |= components
                    .split(',')
                    .map(|item| item.trim())
                    .any(|item| item == "clippy");
            }
        }
    }

    if !saw_tools_section {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "tool-manifest.tools-section-missing",
            detail: "mise.toml is missing a [tools] section".to_string(),
        });
    }

    if saw_tools_section && !saw_rust_entry {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "tool-manifest.rust-entry-missing",
            detail: "mise.toml is missing a pinned rust tool entry".to_string(),
        });
    }

    if saw_rust_entry && (!saw_rustfmt || !saw_clippy) {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "tool-manifest.rust-components-missing",
            detail: "mise.toml rust tool entry should include both rustfmt and clippy components"
                .to_string(),
        });
    }

    violations
}

pub(crate) fn audit_lockfile_text(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    const FORBIDDEN_LOCKFILE_PACKAGES: [&str; 4] = ["cc", "bindgen", "cmake", "pkg-config"];
    let mut violations = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        let Some(name) = line.strip_prefix("name = \"") else {
            continue;
        };
        let Some((package_name, _)) = name.split_once('"') else {
            continue;
        };
        if package_name.ends_with("-sys") || FORBIDDEN_LOCKFILE_PACKAGES.contains(&package_name) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "lockfile.native-package",
                detail: format!(
                    "lockfile package `{package_name}` suggests a native build dependency and should be reviewed"
                ),
            });
        }
    }

    violations
}

pub(crate) fn audit_build_script_path(manifest_path: &Path) -> Option<WorkspaceAuditViolation> {
    let build_script = manifest_path.parent()?.join("build.rs");
    if build_script.is_file() {
        Some(WorkspaceAuditViolation {
            path: build_script,
            rule: "package.build-script",
            detail:
                "package includes a build.rs script, which violates the pure-Rust workspace policy"
                    .to_string(),
        })
    } else {
        None
    }
}

const PUBLISH_WORKSPACE_INHERITED_FIELDS: [&str; 4] =
    ["repository", "homepage", "keywords", "categories"];

const PUBLISH_WORKSPACE_LICENSE: &str = "MIT OR Apache-2.0";

fn manifest_assignment_value(line: &str) -> Option<&str> {
    let (_, value) = line.split_once('=')?;
    Some(value.trim())
}

pub(crate) fn audit_workspace_manifest_publish_text(path: &Path, text: &str) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        WorkspacePackage,
        WorkspaceDependencies,
    }

    let mut section = Section::Other;
    let mut violations = Vec::new();
    let mut workspace_version: Option<String> = None;
    let mut saw_license = false;
    let mut inherited_fields: Vec<&str> = Vec::new();
    let mut internal_dependencies: Vec<(String, String)> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = match line {
                "[workspace.package]" => Section::WorkspacePackage,
                "[workspace.dependencies]" => Section::WorkspaceDependencies,
                _ => Section::Other,
            };
            continue;
        }

        match section {
            Section::WorkspacePackage => {
                if manifest_has_assignment(line, "version") {
                    workspace_version = manifest_assignment_value(line)
                        .map(|value| value.trim_matches('"').to_string());
                }
                if manifest_has_assignment(line, "license") {
                    saw_license = manifest_assignment_value(line)
                        .is_some_and(|value| value.trim_matches('"') == PUBLISH_WORKSPACE_LICENSE);
                }
                for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
                    if manifest_has_assignment(line, field) {
                        inherited_fields.push(field);
                    }
                }
            }
            Section::WorkspaceDependencies => {
                if let Some(name) = manifest_dependency_name(line) {
                    if name.starts_with("pleiades-") {
                        internal_dependencies.push((name.to_string(), line.to_string()));
                    }
                }
            }
            Section::Other => {}
        }
    }

    if !saw_license {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.workspace-license",
            detail: format!(
                "workspace package license must be `{PUBLISH_WORKSPACE_LICENSE}` so published crates inherit the dual license"
            ),
        });
    }

    for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
        if !inherited_fields.contains(&field) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-metadata-missing",
                detail: format!(
                    "workspace package is missing `{field}`, which publishable crates inherit"
                ),
            });
        }
    }

    if workspace_version.is_none() && !internal_dependencies.is_empty() {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.workspace-version-missing",
            detail: "workspace Cargo.toml does not declare a workspace package version to compare against pinned internal dependency versions"
                .to_string(),
        });
    }

    for (name, line) in &internal_dependencies {
        let expected_path = format!("path = \"crates/{name}\"");
        if !line.contains(expected_path.as_str()) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-dependency-path",
                detail: format!(
                    "workspace dependency `{name}` must declare `{expected_path}` so workspace builds use the local crate"
                ),
            });
        }
        match extract_inline_table_string(line, "version") {
            Some(version) => {
                if let Some(expected) = workspace_version.as_deref() {
                    if expected != version {
                        violations.push(WorkspaceAuditViolation {
                            path: path.to_path_buf(),
                            rule: "publish.workspace-dependency-version",
                            detail: format!(
                                "workspace dependency `{name}` pins version {version}, but the workspace package version is {expected}"
                            ),
                        });
                    }
                }
            }
            None => violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.workspace-dependency-version",
                detail: format!(
                    "workspace dependency `{name}` must pin a version equal to the workspace package version so published manifests carry a registry version"
                ),
            }),
        }
    }

    violations
}

pub(crate) fn manifest_is_package(text: &str) -> bool {
    text.lines().any(|line| line.trim() == "[package]")
}

pub(crate) fn manifest_declares_publish_false(text: &str) -> bool {
    let mut in_package = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package
            && manifest_has_assignment(line, "publish")
            && matches!(manifest_assignment_value(line), Some("false") | Some("[]"))
        {
            return true;
        }
    }
    false
}

pub(crate) fn manifest_package_name(text: &str) -> Option<String> {
    let mut in_package = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_package = line == "[package]";
            continue;
        }
        if in_package && manifest_has_assignment(line, "name") {
            return manifest_assignment_value(line)
                .map(|value| value.trim_matches('"').to_string());
        }
    }
    None
}

pub(crate) fn audit_publishable_manifest_text(
    path: &Path,
    text: &str,
    publishable_names: &[String],
) -> Vec<WorkspaceAuditViolation> {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Section {
        Other,
        Package,
        RuntimeDependencies,
        DevDependencies,
    }

    let mut section = Section::Other;
    let mut violations = Vec::new();
    let mut saw_description = false;
    let mut saw_license_inheritance = false;
    let mut saw_readme = false;
    let mut inherited_fields: Vec<&str> = Vec::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            section = match line {
                "[package]" => Section::Package,
                "[dependencies]" | "[build-dependencies]" => Section::RuntimeDependencies,
                "[dev-dependencies]" => Section::DevDependencies,
                _ => Section::Other,
            };
            continue;
        }

        match section {
            Section::Package => {
                if manifest_has_assignment(line, "description") {
                    saw_description |= manifest_assignment_value(line)
                        .is_some_and(|value| !value.trim_matches('"').trim().is_empty());
                }
                if line == "license.workspace = true" {
                    saw_license_inheritance = true;
                }
                if line == "readme = \"README.md\"" {
                    saw_readme = true;
                }
                for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
                    let needle = format!("{field}.workspace = true");
                    if line == needle.as_str() {
                        inherited_fields.push(field);
                    }
                }
            }
            Section::RuntimeDependencies | Section::DevDependencies => {
                let Some(name) = manifest_dependency_name(line) else {
                    continue;
                };
                let package_name = manifest_dependency_package_name(line).unwrap_or(name);
                if !package_name.starts_with("pleiades-") {
                    continue;
                }
                if !line.contains("workspace = true") {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "publish.internal-dependency-not-workspace",
                        detail: format!(
                            "internal dependency `{package_name}` must use `workspace = true` so it inherits the pinned path and version from the workspace manifest"
                        ),
                    });
                }
                if section == Section::RuntimeDependencies
                    && !publishable_names
                        .iter()
                        .any(|publishable| publishable.as_str() == package_name)
                {
                    violations.push(WorkspaceAuditViolation {
                        path: path.to_path_buf(),
                        rule: "publish.internal-dependency-unpublishable",
                        detail: format!(
                            "internal dependency `{package_name}` is not publishable, so this crate cannot list it as a runtime dependency"
                        ),
                    });
                }
            }
            Section::Other => {}
        }
    }

    if !saw_description {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.description-missing",
            detail: "publishable crate is missing a non-blank package description".to_string(),
        });
    }
    if !saw_license_inheritance {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.license-not-inherited",
            detail: "publishable crate must declare `license.workspace = true` so the dual license is inherited"
                .to_string(),
        });
    }
    if !saw_readme {
        violations.push(WorkspaceAuditViolation {
            path: path.to_path_buf(),
            rule: "publish.readme-field-missing",
            detail: "publishable crate must declare `readme = \"README.md\"`".to_string(),
        });
    }
    for field in PUBLISH_WORKSPACE_INHERITED_FIELDS {
        if !inherited_fields.contains(&field) {
            violations.push(WorkspaceAuditViolation {
                path: path.to_path_buf(),
                rule: "publish.metadata-field-missing",
                detail: format!("publishable crate must declare `{field}.workspace = true`"),
            });
        }
    }

    violations
}

pub(crate) fn audit_publishable_crate_files(
    manifest_path: &Path,
    workspace_root: &Path,
) -> Vec<WorkspaceAuditViolation> {
    const PUBLISH_LICENSE_FILES: [&str; 2] = ["LICENSE-APACHE", "LICENSE-MIT"];

    let mut violations = Vec::new();
    let Some(crate_dir) = manifest_path.parent() else {
        return violations;
    };

    let readme_path = crate_dir.join("README.md");
    if !readme_path.is_file() {
        violations.push(WorkspaceAuditViolation {
            path: readme_path,
            rule: "publish.readme-file-missing",
            detail: "publishable crate is missing its README.md".to_string(),
        });
    }

    for license_name in PUBLISH_LICENSE_FILES {
        let crate_copy_path = crate_dir.join(license_name);
        let root_copy_path = workspace_root.join(license_name);
        let Ok(root_bytes) = fs::read(&root_copy_path) else {
            violations.push(WorkspaceAuditViolation {
                path: root_copy_path,
                rule: "publish.license-file-missing",
                detail: format!("workspace root is missing {license_name}"),
            });
            continue;
        };
        match fs::read(&crate_copy_path) {
            Ok(crate_bytes) => {
                if crate_bytes != root_bytes {
                    violations.push(WorkspaceAuditViolation {
                        path: crate_copy_path,
                        rule: "publish.license-file-drift",
                        detail: format!(
                            "crate copy of {license_name} does not match the workspace root copy"
                        ),
                    });
                }
            }
            Err(_) => violations.push(WorkspaceAuditViolation {
                path: crate_copy_path,
                rule: "publish.license-file-missing",
                detail: format!("publishable crate is missing its {license_name} copy"),
            }),
        }
    }

    violations
}

/// Renders the workspace audit used by the CLI and release smoke checks.
pub fn workspace_audit_report() -> Result<WorkspaceAuditReport, std::io::Error> {
    static CACHE: OnceLock<WorkspaceAuditReport> = OnceLock::new();

    if let Some(report) = CACHE.get() {
        return Ok(report.clone());
    }

    let report = workspace_audit_report_uncached()?;
    let _ = CACHE.set(report.clone());
    Ok(report)
}

fn workspace_audit_report_uncached() -> Result<WorkspaceAuditReport, std::io::Error> {
    let workspace_root = fs::canonicalize(workspace_root())?;
    let manifest_paths = workspace_manifest_paths(&workspace_root)?;
    let tool_manifest_path = workspace_root.join("mise.toml");
    let lockfile_path = workspace_root.join("Cargo.lock");
    let mut violations = Vec::new();

    let mut manifests = Vec::new();
    for path in &manifest_paths {
        let text = fs::read_to_string(path)?;
        manifests.push((path.clone(), text));
    }

    let publishable_names: Vec<String> = manifests
        .iter()
        .filter(|(_, text)| manifest_is_package(text) && !manifest_declares_publish_false(text))
        .filter_map(|(_, text)| manifest_package_name(text))
        .collect();

    let root_manifest_path = workspace_root.join("Cargo.toml");
    for (path, text) in &manifests {
        violations.extend(audit_manifest_text(path, text));
        if let Some(violation) = audit_build_script_path(path) {
            violations.push(violation);
        }
        if *path == root_manifest_path {
            violations.extend(audit_workspace_manifest_publish_text(path, text));
        } else if manifest_is_package(text) && !manifest_declares_publish_false(text) {
            violations.extend(audit_publishable_manifest_text(
                path,
                text,
                &publishable_names,
            ));
            violations.extend(audit_publishable_crate_files(path, &workspace_root));
        }
    }

    if tool_manifest_path.is_file() {
        let text = fs::read_to_string(&tool_manifest_path)?;
        violations.extend(audit_tool_manifest_text(
            &tool_manifest_path,
            &text,
            workspace_rust_version(&workspace_root),
        ));
    } else {
        violations.push(WorkspaceAuditViolation {
            path: tool_manifest_path.clone(),
            rule: "tool-manifest.missing",
            detail: "mise.toml is missing from the workspace root".to_string(),
        });
    }

    if lockfile_path.is_file() {
        let text = fs::read_to_string(&lockfile_path)?;
        violations.extend(audit_lockfile_text(&lockfile_path, &text));
    } else {
        violations.push(WorkspaceAuditViolation {
            path: lockfile_path.clone(),
            rule: "lockfile.missing",
            detail: "Cargo.lock is missing from the workspace root".to_string(),
        });
    }

    Ok(WorkspaceAuditReport {
        workspace_root,
        manifest_paths,
        tool_manifest_path,
        lockfile_path,
        violations,
    })
}
