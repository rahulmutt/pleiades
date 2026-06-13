//! Workspace provenance records for release-facing benchmark and validation reports.

use std::fmt;
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceProvenance {
    /// Short git revision for the current workspace state.
    pub source_revision: String,
    /// Whether the workspace is clean, dirty, or unavailable.
    pub workspace_status: String,
    /// The current rustc version string.
    pub rustc_version: String,
    /// The current cargo version string.
    pub cargo_version: String,
    /// The current rustfmt version string.
    pub rustfmt_version: String,
    /// The current clippy version string.
    pub clippy_version: String,
}

/// Validation error for a workspace provenance record that drifted away from the compact report shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceProvenanceValidationError {
    /// A provenance field was blank, padded, or multi-line.
    FieldInvalid { field: &'static str },
}

impl fmt::Display for WorkspaceProvenanceValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldInvalid { field } => write!(
                f,
                "workspace provenance field `{field}` must be a single non-empty line"
            ),
        }
    }
}

impl std::error::Error for WorkspaceProvenanceValidationError {}

impl WorkspaceProvenance {
    /// Returns the compact release-facing benchmark provenance block.
    pub fn summary_line(&self) -> String {
        format!(
            "Benchmark provenance\n  source revision: {}\n  workspace status: {}\n  rustc version: {}\n  cargo version: {}\n  rustfmt version: {}\n  clippy version: {}",
            self.source_revision,
            self.workspace_status,
            self.rustc_version,
            self.cargo_version,
            self.rustfmt_version,
            self.clippy_version
        )
    }

    /// Returns `Ok(())` when the provenance fields are safe for release-facing rendering.
    pub fn validate(&self) -> Result<(), WorkspaceProvenanceValidationError> {
        validate_workspace_provenance_field(&self.source_revision, "source revision")?;
        validate_workspace_provenance_field(&self.workspace_status, "workspace status")?;
        validate_workspace_provenance_field(&self.rustc_version, "rustc version")?;
        validate_workspace_provenance_field(&self.cargo_version, "cargo version")?;
        validate_workspace_provenance_field(&self.rustfmt_version, "rustfmt version")?;
        validate_workspace_provenance_field(&self.clippy_version, "clippy version")?;
        Ok(())
    }
}

impl fmt::Display for WorkspaceProvenance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.summary_line())
    }
}

fn validate_workspace_provenance_field(
    value: &str,
    field: &'static str,
) -> Result<(), WorkspaceProvenanceValidationError> {
    if value.trim().is_empty() || value.contains('\n') || value.contains('\r') {
        return Err(WorkspaceProvenanceValidationError::FieldInvalid { field });
    }

    Ok(())
}

pub fn workspace_provenance() -> WorkspaceProvenance {
    let source_revision = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let workspace_status = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .map(|value| {
            if value.is_empty() {
                "clean".to_string()
            } else {
                "dirty".to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let cargo_version = Command::new("cargo")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let rustfmt_version = Command::new("rustfmt")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let clippy_version = Command::new("cargo")
        .args(["clippy", "--version"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    WorkspaceProvenance {
        source_revision,
        workspace_status,
        rustc_version,
        cargo_version,
        rustfmt_version,
        clippy_version,
    }
}

fn render_workspace_provenance_text(title: &str) -> String {
    let provenance = workspace_provenance();
    match provenance.validate() {
        Ok(()) => format!(
            "{title}\n  source revision: {}\n  workspace status: {}\n  rustc version: {}\n  cargo version: {}\n  rustfmt version: {}\n  clippy version: {}",
            provenance.source_revision,
            provenance.workspace_status,
            provenance.rustc_version,
            provenance.cargo_version,
            provenance.rustfmt_version,
            provenance.clippy_version
        ),
        Err(error) => format!("{title} unavailable ({error})"),
    }
}

pub fn benchmark_provenance_text() -> String {
    render_workspace_provenance_text("Benchmark provenance")
}

/// Returns the compact workspace provenance block used by validation and release tooling.
pub fn workspace_provenance_summary_for_report() -> String {
    render_workspace_provenance_text("Workspace provenance")
}
