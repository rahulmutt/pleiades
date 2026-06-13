//! Workspace audit report types and their compact summaries.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

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
