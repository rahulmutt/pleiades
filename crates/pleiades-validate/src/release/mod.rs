//! Release-facing summaries: checklist, bundle, workspace audit, and notes.

pub(crate) mod bundle;
pub(crate) mod bundle_verify;
mod checklist;
pub(crate) mod notes;
mod workspace_audit;

pub(crate) use notes::{
    render_release_checklist_summary_text, render_release_checklist_text,
    render_release_notes_summary_text, render_release_notes_text, render_release_smoke_text,
    render_release_summary_text,
};

pub use bundle::{render_release_bundle, ReleaseBundle, ReleaseBundleError};
pub(crate) use bundle_verify::{
    validated_lunar_theory_catalog_validation_summary_for_report, verify_release_bundle,
};
pub(crate) use checklist::{
    release_checklist_bundle_contents, release_checklist_external_publishing_reminders,
    release_checklist_manual_bundle_workflow, release_checklist_repository_managed_release_gates,
};
pub use checklist::{release_checklist_summary, ReleaseChecklistSummary};
pub(crate) use workspace_audit::render_workspace_audit_summary_text;
pub use workspace_audit::{
    workspace_audit_report, workspace_audit_summary, WorkspaceAuditReport, WorkspaceAuditSummary,
    WorkspaceAuditViolation,
};
